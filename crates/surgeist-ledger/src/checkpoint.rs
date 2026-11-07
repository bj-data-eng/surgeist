use crate::{Change, Error, Operation, Report, Resource, Scope, Usage, Work, profile::Profile};

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
pub struct IntervalUsage<const W: usize, const R: usize> {
    opening_work: [u64; W],
    opening_live: [u64; R],
    usage: Usage<W, R>,
}

impl<const W: usize, const R: usize> IntervalUsage<W, R> {
    /// Work admitted after opening and before closing.
    pub const fn work(&self, dimension: Work<W>) -> u64 {
        self.usage.work(dimension)
    }
    pub const fn opening_live(&self, dimension: Resource<R>) -> u64 {
        self.opening_live[dimension.0]
    }
    /// Live custody at closing, including custody predating the interval.
    pub const fn live(&self, dimension: Resource<R>) -> u64 {
        self.usage.live(dimension)
    }
    /// Highest committed live custody, including the opening balance and
    /// intermediate acquisitions in an atomic batch.
    pub const fn peak(&self, dimension: Resource<R>) -> u64 {
        self.usage.peak(dimension)
    }
}

/// One scope's interval attribution; use the root for inclusive totals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntervalScopeReport<const W: usize, const R: usize> {
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
pub struct IntervalReport<const W: usize, const R: usize> {
    pub scopes: Vec<IntervalScopeReport<W, R>>,
    pub profile: Option<Report<W, R>>,
}

struct Active<const W: usize, const R: usize> {
    nonce: u64,
    scopes: Vec<IntervalScopeReport<W, R>>,
    profile: Option<Profile<W, R>>,
}

pub(crate) struct Checkpoints<const W: usize, const R: usize> {
    configured: bool,
    next_nonce: u64,
    slots: Vec<Option<Active<W, R>>>,
}

impl<const W: usize, const R: usize> Default for Checkpoints<W, R> {
    fn default() -> Self {
        Self {
            configured: false,
            next_nonce: 0,
            slots: Vec::new(),
        }
    }
}

impl<const W: usize, const R: usize> Operation<W, R> {
    /// Configures the fixed concurrent checkpoint bound once. The default bound
    /// is zero. Registry reservation is fallible; interval buffers are reserved
    /// at begin, never during admissions or valid checkpoint completion.
    pub fn with_checkpoint_limit(mut self, limit: usize) -> Result<Self, Error<W, R>> {
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
        for (index, entry) in self.scopes.iter().enumerate() {
            let scope = self.scope(index);
            let parent = entry.parent_index().map(|index| self.scope(index));
            scopes.push(IntervalScopeReport {
                scope,
                parent,
                name: entry.name,
                usage: IntervalUsage {
                    opening_work: entry.usage.work,
                    opening_live: entry.usage.live,
                    usage: Usage {
                        work: [0; W],
                        live: entry.usage.live,
                        peak: entry.usage.live,
                    },
                },
            });
            if let Some(profile) = &mut profile {
                profile.add_scope(scope, parent, entry.name);
            }
        }
        self.checkpoints.slots[slot] = Some(Active {
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

    /// Closes a valid interval by moving its pre-reserved buffers. This performs
    /// no allocation. Foreign/stale identities leave every active interval intact.
    pub fn end_checkpoint(
        &mut self,
        checkpoint: &Checkpoint,
    ) -> Result<IntervalReport<W, R>, Error<W, R>> {
        if checkpoint.operation != self.identity {
            return Err(Error::ForeignCheckpoint);
        }
        let slot = self
            .checkpoints
            .slots
            .get_mut(checkpoint.slot)
            .ok_or(Error::StaleCheckpoint)?;
        if slot
            .as_ref()
            .is_none_or(|active| active.nonce != checkpoint.nonce)
        {
            return Err(Error::StaleCheckpoint);
        }
        let mut active = slot.take().expect("validated active checkpoint");
        for (scope, entry) in active.scopes.iter_mut().zip(&self.scopes) {
            for (index, opening) in scope.usage.opening_work.iter().enumerate() {
                scope.usage.usage.work[index] = entry.usage.work[index]
                    .checked_sub(*opening)
                    .expect("cumulative accepted work never decreases");
            }
            // Baselines are private staging state, not part of a closed report.
            scope.usage.opening_work = [0; W];
            scope.usage.usage.live = entry.usage.live;
        }
        let profile = active
            .profile
            .map(|profile| profile.finish(active.scopes.iter().map(|scope| scope.usage.usage)));
        Ok(IntervalReport {
            scopes: active.scopes,
            profile,
        })
    }
}

impl<const W: usize, const R: usize> Checkpoints<W, R> {
    pub(crate) fn add_scope(&mut self, scope: Scope, parent: Option<Scope>, name: &'static str) {
        for active in self.slots.iter_mut().flatten() {
            active.scopes.push(IntervalScopeReport {
                scope,
                parent,
                name,
                usage: IntervalUsage {
                    opening_work: [0; W],
                    opening_live: [0; R],
                    usage: Usage::ZERO,
                },
            });
            if let Some(profile) = &mut active.profile {
                profile.add_scope(scope, parent, name);
            }
        }
    }

    pub(crate) fn commit_peak(&mut self, index: usize, peak: [u64; R]) {
        for active in self.slots.iter_mut().flatten() {
            for (current, peak) in active.scopes[index].usage.usage.peak.iter_mut().zip(peak) {
                *current = (*current).max(peak);
            }
        }
    }

    pub(crate) fn admission(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
        result: Result<(), Error<W, R>>,
    ) {
        for active in self.slots.iter_mut().flatten() {
            if let Some(profile) = &mut active.profile {
                profile.admission(scope, changes, result);
            }
        }
    }

    pub(crate) fn end_phase(
        &mut self,
        scope: Scope,
        start: Option<std::time::Duration>,
        end: Option<std::time::Duration>,
    ) {
        for active in self.slots.iter_mut().flatten() {
            if let Some(profile) = &mut active.profile {
                profile.end_checkpoint_phase(scope, start, end);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_nonce_does_not_open_an_interval() {
        let mut operation = Operation::<0, 0>::new("root", crate::Limits::unlimited(), 1)
            .unwrap()
            .with_checkpoint_limit(1)
            .unwrap();
        operation.checkpoints.next_nonce = u64::MAX;
        assert_eq!(operation.begin_checkpoint(), Err(Error::IdentityExhausted));
        assert!(operation.checkpoints.slots[0].is_none());
    }
    #[test]
    fn failed_buffer_reservation_leaves_slot_and_nonce_available() {
        let mut operation = Operation::<1, 1>::new("root", crate::Limits::unlimited(), usize::MAX)
            .unwrap()
            .with_checkpoint_limit(1)
            .unwrap();
        assert_eq!(operation.begin_checkpoint(), Err(Error::Allocation));
        assert!(operation.checkpoints.slots[0].is_none());
        assert_eq!(operation.checkpoints.next_nonce, 0);
    }

    #[test]
    fn valid_end_transfers_the_reserved_usage_buffer() {
        let mut operation = Operation::<1, 1>::new("root", crate::Limits::unlimited(), 2)
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
}
