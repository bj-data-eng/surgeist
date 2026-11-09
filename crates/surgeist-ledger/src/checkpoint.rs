use crate::{
    Change, Defect, Dimension, Error, Operation, Report, Scope, Usage, profile::Profile,
    push_reserved,
};

/// An opaque interval identity. Ending it does not consume the token, but any
/// subsequent end returns [`Error::StaleCheckpoint`]. Dropping it does not end it.
#[derive(Debug, Eq, PartialEq)]
pub struct Checkpoint {
    operation: u64,
    slot: usize,
    nonce: u64,
}

/// Inclusive accepted interval usage in the scope's units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntervalUsage<W: Dimension, R: Dimension> {
    opening_work: W::Table<u64>,
    opening_live: R::Table<u64>,
    usage: Usage<W, R>,
}

impl<W: Dimension, R: Dimension> IntervalUsage<W, R> {
    /// Work admitted after opening and before closing.
    pub fn work(&self, dimension: W) -> u64 {
        self.usage.work(dimension)
    }
    pub fn opening_live(&self, dimension: R) -> u64 {
        *R::get(&self.opening_live, dimension)
    }
    /// Live custody at closing, including custody predating the interval.
    pub fn live(&self, dimension: R) -> u64 {
        self.usage.live(dimension)
    }
    /// Highest committed live custody, including the opening balance and
    /// intermediate acquisitions in an atomic batch.
    pub fn peak(&self, dimension: R) -> u64 {
        self.usage.peak(dimension)
    }
}

/// One scope's interval attribution; use the root for inclusive totals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntervalScopeReport<W: Dimension, R: Dimension> {
    pub scope: Scope,
    pub parent: Option<Scope>,
    pub name: &'static str,
    pub usage: IntervalUsage<W, R>,
}

/// A closed interval. Exact accounting is available even with profiling off.
/// The optional profile covers the same interval: its usage contains work
/// deltas, closing live balances and exact interval peaks, never lifetime peaks.
/// Events and elapsed time are relative to interval opening. Only phases ended
/// inside the interval contribute; their duration is clipped at interval opening.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntervalReport<W: Dimension, R: Dimension> {
    pub scopes: Vec<IntervalScopeReport<W, R>>,
    pub profile: Option<Report<W, R>>,
}

struct Active<W: Dimension, R: Dimension> {
    nonce: u64,
    scopes: Vec<IntervalScopeReport<W, R>>,
    profile: Option<Profile<W, R>>,
}

pub(crate) struct Checkpoints<W: Dimension, R: Dimension> {
    configured: bool,
    next_nonce: u64,
    slots: Vec<Option<Active<W, R>>>,
}

impl<W: Dimension, R: Dimension> Default for Checkpoints<W, R> {
    fn default() -> Self {
        Self {
            configured: false,
            next_nonce: 0,
            slots: Vec::new(),
        }
    }
}

impl<W: Dimension, R: Dimension> Operation<W, R> {
    /// Configures the fixed concurrent checkpoint bound once. The default bound
    /// is zero. Registry reservation is fallible; interval buffers are reserved
    /// at begin, never during admissions or valid checkpoint completion.
    pub fn with_checkpoint_limit(mut self, limit: usize) -> Result<Self, Error<W, R>> {
        self.healthy()?;
        if self.checkpoints.configured {
            return Err(Error::CheckpointsConfigured);
        }
        self.checkpoints
            .slots
            .try_reserve_exact(limit)
            .map_err(|_| Error::Allocation)?;
        self.checkpoints.slots.resize_with(limit, || None);
        self.checkpoints.configured = true;
        Ok(self)
    }

    /// Opens an independently bounded interval over all existing and future
    /// scopes. Unended tokens occupy their slot until ended or operation drop.
    pub fn begin_checkpoint(&mut self) -> Result<Checkpoint, Error<W, R>> {
        self.healthy()?;
        let slot = self
            .checkpoints
            .slots
            .iter()
            .position(Option::is_none)
            .ok_or(Error::CheckpointLimit {
                limit: self.checkpoints.slots.len(),
            })?;
        let nonce = self
            .checkpoints
            .next_nonce
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let mut scopes = Vec::new();
        scopes
            .try_reserve_exact(self.max_scopes)
            .map_err(|_| Error::Allocation)?;
        let mut profile = self
            .profile
            .first()
            .map(|profile| profile.checkpoint(self.max_scopes))
            .transpose()?;
        let opened = self.open_interval(&mut scopes, profile.as_mut());
        self.note(opened)?;
        let Some(free) = self.checkpoints.slots.get_mut(slot) else {
            return self.note(Err(Error::Defect(Defect::CheckpointSlot)));
        };
        *free = Some(Active {
            nonce,
            scopes,
            profile,
        });
        self.checkpoints.next_nonce = nonce;
        Ok(Checkpoint {
            operation: self.identity,
            slot,
            nonce,
        })
    }

    fn open_interval(
        &self,
        scopes: &mut Vec<IntervalScopeReport<W, R>>,
        mut profile: Option<&mut Profile<W, R>>,
    ) -> Result<(), Error<W, R>> {
        for (index, entry) in self.scopes.iter().enumerate() {
            let scope = self.scope(index);
            let parent = entry.parent_index().map(|index| self.scope(index));
            push_reserved(
                scopes,
                IntervalScopeReport {
                    scope,
                    parent,
                    name: entry.name,
                    usage: IntervalUsage {
                        opening_work: entry.usage.work,
                        opening_live: entry.usage.live,
                        usage: Usage {
                            work: W::from_fn(|_| 0),
                            live: entry.usage.live,
                            peak: entry.usage.live,
                        },
                    },
                },
            )
            .map_err(Error::Defect)?;
            if let Some(profile) = profile.as_deref_mut() {
                profile
                    .add_scope(scope, parent, entry.name)
                    .map_err(Error::Defect)?;
            }
        }
        Ok(())
    }

    /// Closes a valid interval by moving its pre-reserved buffers. This performs
    /// no allocation. Foreign/stale identities leave every active interval intact.
    pub fn end_checkpoint(
        &mut self,
        checkpoint: &Checkpoint,
    ) -> Result<IntervalReport<W, R>, Error<W, R>> {
        self.healthy()?;
        if checkpoint.operation != self.identity {
            return Err(Error::ForeignCheckpoint);
        }
        let slot = self
            .checkpoints
            .slots
            .get_mut(checkpoint.slot)
            .ok_or(Error::StaleCheckpoint)?;
        let active = match slot.take() {
            Some(active) if active.nonce == checkpoint.nonce => active,
            other => {
                *slot = other;
                return Err(Error::StaleCheckpoint);
            }
        };
        let closed = self.close_interval(active);
        self.note(closed)
    }

    fn close_interval(
        &self,
        mut active: Active<W, R>,
    ) -> Result<IntervalReport<W, R>, Error<W, R>> {
        if active.scopes.len() != self.scopes.len() {
            return Err(Error::Defect(Defect::ReportScope));
        }
        for (scope, entry) in active.scopes.iter_mut().zip(&self.scopes) {
            for &dimension in W::ALL {
                let opening = *W::get(&scope.usage.opening_work, dimension);
                let closing = *W::get(&entry.usage.work, dimension);
                *W::get_mut(&mut scope.usage.usage.work, dimension) = closing
                    .checked_sub(opening)
                    .ok_or(Error::Defect(Defect::IntervalWork))?;
            }
            // Baselines are private staging state, not part of a closed report.
            scope.usage.opening_work = W::from_fn(|_| 0);
            scope.usage.usage.live = entry.usage.live;
        }
        let profile = active
            .profile
            .map(|profile| profile.finish(active.scopes.iter().map(|scope| scope.usage.usage)))
            .transpose()
            .map_err(Error::Defect)?;
        Ok(IntervalReport {
            scopes: active.scopes,
            profile,
        })
    }
}

impl<W: Dimension, R: Dimension> Checkpoints<W, R> {
    /// Whether any interval slot exists. Without one, admissions skip
    /// interval bookkeeping.
    pub(crate) fn configured(&self) -> bool {
        !self.slots.is_empty()
    }

    pub(crate) fn add_scope(
        &mut self,
        scope: Scope,
        parent: Option<Scope>,
        name: &'static str,
    ) -> Result<(), Defect> {
        for active in self.slots.iter_mut().flatten() {
            push_reserved(
                &mut active.scopes,
                IntervalScopeReport {
                    scope,
                    parent,
                    name,
                    usage: IntervalUsage {
                        opening_work: W::from_fn(|_| 0),
                        opening_live: R::from_fn(|_| 0),
                        usage: Usage::zero(),
                    },
                },
            )?;
            if let Some(profile) = &mut active.profile {
                profile.add_scope(scope, parent, name)?;
            }
        }
        Ok(())
    }

    pub(crate) fn commit_peak(&mut self, index: usize, peak: R::Table<u64>) -> Result<(), Defect> {
        for active in self.slots.iter_mut().flatten() {
            let scope = active.scopes.get_mut(index).ok_or(Defect::ReportScope)?;
            for &dimension in R::ALL {
                let current = R::get_mut(&mut scope.usage.usage.peak, dimension);
                *current = (*current).max(*R::get(&peak, dimension));
            }
        }
        Ok(())
    }

    pub(crate) fn admission(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
        result: Result<(), Error<W, R>>,
    ) -> Result<(), Defect> {
        for active in self.slots.iter_mut().flatten() {
            if let Some(profile) = &mut active.profile {
                profile.admission(scope, changes, result)?;
            }
        }
        Ok(())
    }

    pub(crate) fn end_phase(
        &mut self,
        scope: Scope,
        start: Option<std::time::Duration>,
        end: Option<std::time::Duration>,
    ) -> Result<(), Defect> {
        for active in self.slots.iter_mut().flatten() {
            if let Some(profile) = &mut active.profile {
                profile.end_checkpoint_phase(scope, start, end)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoDimension;

    crate::dimensions! {
        enum TestWork => TestWorkTable { Visit }
    }

    crate::dimensions! {
        enum TestResource => TestResourceTable { Bytes }
    }

    #[test]
    fn exhausted_nonce_does_not_open_an_interval() {
        let mut operation =
            Operation::<NoDimension, NoDimension>::new("root", crate::Limits::unlimited(), 1)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
        operation.checkpoints.next_nonce = u64::MAX;
        assert_eq!(operation.begin_checkpoint(), Err(Error::IdentityExhausted));
        assert!(operation.checkpoints.slots[0].is_none());
    }

    #[test]
    fn failed_buffer_reservation_leaves_slot_and_nonce_available() {
        let mut operation = Operation::<TestWork, TestResource>::new(
            "root",
            crate::Limits::unlimited(),
            usize::MAX,
        )
        .unwrap()
        .with_checkpoint_limit(1)
        .unwrap();
        assert_eq!(operation.begin_checkpoint(), Err(Error::Allocation));
        assert!(operation.checkpoints.slots[0].is_none());
        assert_eq!(operation.checkpoints.next_nonce, 0);
    }

    #[test]
    fn valid_end_transfers_the_reserved_usage_buffer() {
        let mut operation =
            Operation::<TestWork, TestResource>::new("root", crate::Limits::unlimited(), 2)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
        let checkpoint = operation.begin_checkpoint().unwrap();
        let buffer = operation.checkpoints.slots[0]
            .as_ref()
            .unwrap()
            .scopes
            .as_ptr();
        let report = operation.end_checkpoint(&checkpoint).unwrap();
        assert_eq!(report.scopes.as_ptr(), buffer);
    }

    #[test]
    fn interval_work_below_its_opening_is_a_latched_defect() {
        let mut operation =
            Operation::<TestWork, TestResource>::new("root", crate::Limits::unlimited(), 1)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
        let root = operation.root();
        operation
            .apply(root, Change::Work(TestWork::Visit, 3))
            .unwrap();
        let checkpoint = operation.begin_checkpoint().unwrap();
        // Cumulative work never decreases through the public API; lower it to
        // reach the state the closing subtraction guards.
        operation.scopes[0].usage.work.Visit = 1;
        assert_eq!(
            operation.end_checkpoint(&checkpoint),
            Err(Error::Defect(Defect::IntervalWork))
        );
        assert_eq!(
            operation.apply(root, Change::Work(TestWork::Visit, 1)),
            Err(Error::Defect(Defect::IntervalWork))
        );
        assert_eq!(operation.finish(), Err(Error::Defect(Defect::IntervalWork)));
    }

    fn with_one_interval() -> (Operation<TestWork, TestResource>, Checkpoint) {
        let mut operation =
            Operation::<TestWork, TestResource>::new("root", crate::Limits::unlimited(), 2)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
        let checkpoint = operation.begin_checkpoint().unwrap();
        (operation, checkpoint)
    }

    #[test]
    fn interval_records_not_matching_the_scopes_are_a_latched_defect() {
        let (mut operation, checkpoint) = with_one_interval();
        operation.checkpoints.slots[0]
            .as_mut()
            .unwrap()
            .scopes
            .pop();
        assert_eq!(
            operation.end_checkpoint(&checkpoint),
            Err(Error::Defect(Defect::ReportScope))
        );
        crate::tests::assert_latched(operation, Defect::ReportScope);
    }

    #[test]
    fn admission_to_a_scope_without_an_interval_record_is_a_latched_defect() {
        let (mut operation, _checkpoint) = with_one_interval();
        let root = operation.root();
        operation.checkpoints.slots[0]
            .as_mut()
            .unwrap()
            .scopes
            .pop();
        assert_eq!(
            operation.apply(root, Change::Acquire(TestResource::Bytes, 1)),
            Err(Error::Defect(Defect::ReportScope))
        );
        crate::tests::assert_latched(operation, Defect::ReportScope);
    }

    #[test]
    fn scopes_beyond_the_interval_reservation_latch_a_reserved_capacity_defect() {
        let mut operation =
            Operation::<TestWork, TestResource>::new("root", crate::Limits::unlimited(), 1)
                .unwrap()
                .with_checkpoint_limit(1)
                .unwrap();
        // The interval reserves one record per permitted scope. Add scopes
        // past that bound so opening the interval has no room for them.
        for _ in 0..8 {
            operation.scopes.push(crate::ScopeEntry::new(
                "extra",
                None,
                crate::Limits::unlimited(),
            ));
        }
        assert_eq!(
            operation.begin_checkpoint(),
            Err(Error::Defect(Defect::ReservedCapacity))
        );
        assert!(operation.checkpoints.slots[0].is_none());
        crate::tests::assert_latched(operation, Defect::ReservedCapacity);
    }
}
