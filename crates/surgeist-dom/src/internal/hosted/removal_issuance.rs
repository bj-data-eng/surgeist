//! Prepare the issuer rows that a private child-removal segment will transfer.

use super::*;
use crate::internal::store::{EmptyPath, check_cancel, source::formation::Formation};

pub(crate) struct Funding {
    pub(crate) work: Arc<Work>,
    pub(crate) existing_cleanup: Option<DomHostedOperationId>,
}

#[derive(Clone, Copy)]
pub(crate) struct Ids {
    pub(crate) history: DomHostedOperationId,
    pub(crate) invocation: DomHostedOperationId,
    pub(crate) cleanup: DomHostedOperationId,
}

struct Row {
    id: DomHostedOperationId,
    value: IdentitySlot,
    path: Option<EmptyPath<IdentitySlot>>,
}

pub(crate) struct IssuancePlan {
    rows: Vec<Row>,
    ids: Vec<Ids>,
    retained: Retained,
    scratch: usize,
    before: DomHostedSourceRevision,
    after: DomHostedSourceRevision,
    reserved: u64,
    live: usize,
}

impl IssuancePlan {
    pub(crate) fn ids(&self) -> &[Ids] {
        &self.ids
    }

    pub(crate) fn scratch_bytes(&self) -> usize {
        self.scratch
    }

    pub(crate) fn authorize<'a>(
        self,
        source: &'a DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<RemovalPermit<'a>, DomHostedError> {
        let state = source.lock()?;
        if state.revision != self.before
            || !std::ptr::eq(self.retained.shared.as_ptr(), Arc::as_ptr(&source.shared))
        {
            return Err(DomHostedError::StaleBasis);
        }
        check_cancel(cancel)?;
        Ok(RemovalPermit { plan: self, state })
    }
}

pub(crate) struct RemovalPermit<'a> {
    plan: IssuancePlan,
    state: std::sync::MutexGuard<'a, Issuance>,
}

impl RemovalPermit<'_> {
    pub(crate) fn commit(mut self) {
        let role = Role::Operation.index();
        let bank = &mut self.state.banks[role];
        for row in self.plan.rows {
            if let Some(path) = row.path {
                bank.insert(path, row.value, false);
            } else {
                bank[row.id.slot] = row.value;
                bank.set_reusable(row.id.slot, false);
            }
        }
        self.state.live[role] = self.plan.live;
        self.state.revision = self.plan.after;
        self.state.reserved_revisions = self.plan.reserved;
    }
}

impl DomHostedSource {
    pub(crate) fn refresh_private_removal_work(&self, work: &Work) -> Result<(), DomHostedError> {
        let limit = self.limits.work;
        let observation = WorkObservation::new(work);
        observation.validate_limit(limit)?;
        observation.set_limit(limit);
        Ok(())
    }

    pub(crate) fn prepare_child_removal_issuance(
        &self,
        funding: &[Funding],
        scratch_base: usize,
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<IssuancePlan, DomHostedError> {
        let mut state = self.lock()?;
        let role = Role::Operation;
        let role_index = role.index();
        let mut count = 0usize;
        let mut revisions = 0u64;
        let mut remaining = 0u64;
        for input in funding {
            input.work.step(cancel)?;
            let cleanup = input.existing_cleanup.is_none();
            count = count
                .checked_add(2 + usize::from(cleanup))
                .ok_or(DomHostedError::RepresentationExhausted)?;
            revisions = revisions
                .checked_add(3 + u64::from(cleanup))
                .ok_or(DomHostedError::RepresentationExhausted)?;
            remaining = remaining
                .checked_add(1 + 3 * u64::from(cleanup))
                .ok_or(DomHostedError::RepresentationExhausted)?;
        }
        let live = state.live[role_index]
            .checked_add(count)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let (limit, resource) = role.bound(self.limits.records);
        admit(live, limit, resource)?;
        let before = state.revision;
        let after = DomHostedSourceRevision(
            before
                .0
                .checked_add(revisions)
                .ok_or(DomHostedError::RepresentationExhausted)?,
        );
        let reserved = state
            .reserved_revisions
            .checked_add(remaining)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        after
            .0
            .checked_add(reserved)
            .ok_or(DomHostedError::RepresentationExhausted)?;

        let mut scratch = size_of::<IssuancePlan>()
            .checked_add(
                count
                    .checked_mul(size_of::<Row>())
                    .ok_or(DomHostedError::RepresentationExhausted)?,
            )
            .and_then(|n| n.checked_add(funding.len().checked_mul(size_of::<Ids>())?))
            .ok_or(DomHostedError::RepresentationExhausted)?;
        self.admit_removal_scratch(scratch_base, scratch)?;
        if let Some(formation) = formation {
            formation.retain_bytes(scratch, cancel)?;
        }
        let mut retained = self.retain_locked(scratch, cancel)?;
        let mut rows = Vec::new();
        let mut ids = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        ids.try_reserve_exact(funding.len())
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let extra = (rows.capacity() - count)
            .checked_mul(size_of::<Row>())
            .and_then(|n| {
                n.checked_add((ids.capacity() - funding.len()).checked_mul(size_of::<Ids>())?)
            })
            .ok_or(DomHostedError::RepresentationExhausted)?;
        scratch = scratch
            .checked_add(extra)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        self.admit_removal_scratch(scratch_base, scratch)?;
        if let Some(formation) = formation {
            formation.retain_bytes(extra, cancel)?;
        }
        retained.append(self.retain_locked(extra, cancel)?)?;

        let bank = &mut state.banks[role_index];
        let mut append = bank.len();
        let result: Result<(), DomHostedError> = (|| {
            for input in funding {
                let mut prepare = |phase, credit| {
                    input.work.step(cancel)?;
                    let vacant = bank.lowest_vacant();
                    let slot = vacant.unwrap_or(append);
                    let (generation, path) = if let Some(slot) = vacant {
                        let row = bank.get(slot).ok_or(DomHostedError::StaleBasis)?;
                        let generation = row.generation;
                        bank.set_reusable(slot, false);
                        (generation, None)
                    } else {
                        let path = bank.reserve(slot, |charge| {
                            input.work.step(cancel)?;
                            scratch = scratch
                                .checked_add(charge.bytes)
                                .ok_or(DomHostedError::RepresentationExhausted)?;
                            self.admit_removal_scratch(scratch_base, scratch)?;
                            if let Some(formation) = formation {
                                formation.retain_bytes(charge.bytes, cancel)?;
                            }
                            self.retain_locked(charge.bytes, cancel).map(Some)
                        })?;
                        append = append
                            .checked_add(1)
                            .ok_or(DomHostedError::RepresentationExhausted)?;
                        (1, Some(path))
                    };
                    let id = DomHostedOperationId {
                        source: self.id(),
                        slot,
                        generation,
                    };
                    rows.push(Row {
                        id,
                        value: IdentitySlot {
                            generation,
                            live: true,
                            retired: false,
                            phase,
                            issued: 0,
                            creator: None,
                            reserved_revisions: credit,
                            work: Some(WorkObservation::new(&input.work)),
                        },
                        path,
                    });
                    Ok::<_, DomHostedError>(id)
                };
                // A private removal never lends its intermediate phase-zero
                // history request. Its second revision brings it to queued.
                let history = prepare(1, 0)?;
                let invocation = prepare(0, 1)?;
                let cleanup = match input.existing_cleanup {
                    Some(cleanup) => cleanup,
                    None => prepare(0, 3)?,
                };
                ids.push(Ids {
                    history,
                    invocation,
                    cleanup,
                });
            }
            Ok(())
        })();
        // These temporary selection marks never escape the issuer mutex.
        for row in &rows {
            if row.path.is_none() {
                bank.set_reusable(row.id.slot, true);
            }
        }
        result?;
        Ok(IssuancePlan {
            rows,
            ids,
            retained,
            scratch,
            before,
            after,
            reserved,
            live,
        })
    }

    fn admit_removal_scratch(&self, base: usize, extra: usize) -> Result<(), DomHostedError> {
        admit(
            base.checked_add(extra)
                .ok_or(DomHostedError::RepresentationExhausted)?,
            self.limits.scratch_bytes,
            DomHostedResource::ScratchBytes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removal_plan_reuses_vacant_generations_and_composes_bank_growth() {
        let cancel = AtomicBool::new(false);
        let mut source = DomHostedSource::new(
            DomHostedSourceId {
                owner: DomOwnerId::allocate().unwrap(),
                serial: 0,
            },
            DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            &cancel,
        )
        .unwrap();
        let work = source.work(&cancel).unwrap();
        let first = source.begin_operation(&work, 1, 0, &cancel).unwrap();
        let kept = source.begin_operation(&work, 1, 0, &cancel).unwrap();
        source
            .retire(
                Role::Operation,
                first.source,
                first.slot,
                first.generation,
                &cancel,
            )
            .unwrap();
        let before = source.revision().unwrap();
        let available = source.remaining_bytes().unwrap();
        let funding: Vec<_> = (0..6)
            .map(|_| Funding {
                work: work.clone(),
                existing_cleanup: None,
            })
            .collect();
        let plan = source
            .prepare_child_removal_issuance(&funding, 0, None, &cancel)
            .unwrap();
        assert!(matches!(
            plan.authorize(&source, &AtomicBool::new(true)),
            Err(DomHostedError::Dom(DomError::Cancelled))
        ));
        assert_eq!(source.revision().unwrap(), before);
        assert_eq!(
            source.next_operation_slot(&work, &cancel).unwrap(),
            first.slot
        );
        assert_eq!(source.remaining_bytes().unwrap(), available);
        let plan = source
            .prepare_child_removal_issuance(&funding, 0, None, &cancel)
            .unwrap();
        let ids = plan.ids().to_vec();
        assert_eq!(ids[0].history.slot, first.slot);
        assert_eq!(ids[0].history.generation, first.generation + 1);
        assert!(ids.last().unwrap().cleanup.slot >= 16);
        plan.authorize(&source, &cancel).unwrap().commit();

        let state = source.lock().unwrap();
        assert_eq!(state.revision.0, before.0 + 24);
        assert_eq!(state.reserved_revisions, 24);
        let role = Role::Operation.index();
        assert_eq!(state.live[role], 19);
        let bank = &state.banks[role];
        assert_eq!(bank[kept.slot].generation, kept.generation);
        for ids in ids {
            for (id, phase, credit) in [
                (ids.history, 1, 0),
                (ids.invocation, 0, 1),
                (ids.cleanup, 0, 3),
            ] {
                let row = &bank[id.slot];
                assert!(row.live);
                assert_eq!(row.generation, id.generation);
                assert_eq!(row.phase, phase);
                assert_eq!(row.reserved_revisions, credit);
            }
        }
    }
}
