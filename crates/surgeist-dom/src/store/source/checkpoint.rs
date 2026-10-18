//! Unpublished source-segment rollback storage. The canonical owner retains the
//! original external receipt separately, so restoring a segment cannot replay it.
use super::super::*;

#[cfg(test)]
mod tests;

pub(in crate::store) struct Charge {
    storage: super::formation::SourceStorage,
    debit: Option<custody::Debit>,
    live: Arc<AtomicUsize>,
    bytes: usize,
    live_units: Arc<AtomicUsize>,
    units: usize,
    _dependencies: super::formation::WorkDependencies,
}
impl Charge {
    pub(super) fn payload(
        storage: super::formation::SourceStorage,
        live: Arc<AtomicUsize>,
        bytes: usize,
        live_units: Arc<AtomicUsize>,
        units: usize,
        debit: Option<custody::Debit>,
        dependencies: super::formation::WorkDependencies,
    ) -> Self {
        Self {
            storage,
            debit,
            live,
            bytes,
            live_units,
            units,
            _dependencies: dependencies,
        }
    }
    pub(in crate::store) fn merge(&mut self, mut other: Self) {
        assert!(
            Arc::ptr_eq(&self.live, &other.live)
                && Arc::ptr_eq(&self.live_units, &other.live_units),
            "same Source payload owner"
        );
        self.bytes = self
            .bytes
            .checked_add(other.bytes)
            .expect("already admitted live bytes");
        self.units = self
            .units
            .checked_add(other.units)
            .expect("already admitted live units");
        self.absorb_debit(other.debit.take());
        other.bytes = 0;
        other.units = 0;
    }
    /// Move this already admitted payload occurrence into its final owner.
    /// The protocol mirrors end here; the affine debit remains live in the
    /// receiving owner throughout the transfer.
    pub(in crate::store) fn into_debit(mut self) -> Option<custody::Debit> {
        let debit = self.debit.take();
        self.live.fetch_sub(self.bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(self.units, Ordering::Relaxed);
        self.bytes = 0;
        self.units = 0;
        debit
    }
    #[cfg(test)]
    pub(in crate::store) fn journal_bytes(&self) -> usize {
        self.debit.as_ref().map_or(0, |debit| debit.usage().bytes())
    }
    fn absorb_debit(&mut self, incoming: Option<custody::Debit>) {
        if let Some(incoming) = incoming {
            if let Some(owned) = self.debit.as_mut() {
                owned.absorb(incoming).expect("same Source custody ledger");
            } else {
                self.debit = Some(incoming);
            }
        }
    }
    /// Reclassify the exact current graph of an original controlled producer.
    /// Its call must already have admitted all allocation/overlap against an
    /// envelope derived from this same ledger, with this charge excluded once.
    /// Nothing may allocate or change other custody between that call and sync.
    pub(in crate::store) fn replace_admitted_payload(&mut self, payload: super::storage::Payload) {
        let growth = custody::Usage::new(
            payload.bytes.saturating_sub(self.bytes),
            payload.units.saturating_sub(self.units),
        );
        let shrink = custody::Usage::new(
            self.bytes.saturating_sub(payload.bytes),
            self.units.saturating_sub(payload.units),
        );
        if growth.bytes() != 0 || growth.units() != 0 {
            let debit = self
                .storage
                .ledger()
                .retain(growth, |_| Ok(()))
                .expect("original producer preadmitted the exact surviving graph");
            self.absorb_debit(Some(debit));
        }
        if shrink.bytes() != 0 || shrink.units() != 0 {
            drop(
                self.debit
                    .as_mut()
                    .expect("owned producer graph")
                    .split(shrink)
                    .expect("owned producer graph component"),
            );
        }
        // Mirrors have no ownership role. Relative updates preserve releases
        // by independent guards while this original producer is executing.
        self.live.fetch_add(growth.bytes(), Ordering::Relaxed);
        self.live.fetch_sub(shrink.bytes(), Ordering::Relaxed);
        self.live_units.fetch_add(growth.units(), Ordering::Relaxed);
        self.live_units.fetch_sub(shrink.units(), Ordering::Relaxed);
        self.bytes = payload.bytes;
        self.units = payload.units;
    }
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.live.fetch_sub(self.bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(self.units, Ordering::Relaxed);
    }
}
pub(in crate::store) struct Admission<'a> {
    original: &'a Draft,
    charge: Charge,
}
impl<'a> Admission<'a> {
    pub(in crate::store) fn new(original: &'a Draft, live_bytes: usize) -> Result<Self, DomError> {
        let limits = original
            .source_limits
            .expect("source checkpoint has explicit admission");
        admit(
            live_bytes,
            limits.retained_bytes,
            DomResource::SourceRetainedBytes,
        )?;
        Ok(Self {
            original,
            charge: Charge {
                storage: super::formation::SourceStorage::new(original.custody_ledger().clone()),
                debit: None,
                live: original.select_temporary_bytes.clone(),
                bytes: 0,
                live_units: original.source_temporary_units.clone(),
                units: 0,
                _dependencies: super::formation::WorkDependencies::from_draft(original),
            },
        })
    }
    pub(in crate::store) fn step(&self) -> Result<(), DomError> {
        // The enclosing driver admits cancellation before checkpoint formation;
        // copies also use its exact cancellation token through copy methods.
        let limits = self
            .original
            .source_limits
            .expect("source checkpoint admission");
        let next = self
            .original
            .source_work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > limits.work {
            return Err(DomError::SourceWorkLimit { limit: limits.work });
        }
        self.original.source_work.store(next, Ordering::Relaxed);
        super::formation::SourceStorage::record_step();
        crate::source::event::retain_ancestor_work(self.original.source_ancestors.as_ref(), 1)?;
        Ok(())
    }
    pub(in crate::store) fn effect_payload(
        &mut self,
        payload: super::storage::Payload,
        cancel: &AtomicBool,
    ) -> Result<Option<Charge>, DomError> {
        if payload.bytes == 0 && payload.units == 0 {
            return Ok(None);
        }
        let budget =
            super::formation::Formation::new(self.original, cancel)?.expect("source checkpoint");
        budget.retain_payload(payload, cancel)?;
        Ok(Some(budget.into_charge()))
    }
    pub(in crate::store) fn finish(self) -> Charge {
        self.charge
    }
}

/// A private rollback boundary in the original sparse transaction. The affine
/// mark seals on drop; restoring it changes semantic state only. Spent work,
/// acknowledged external receipts and issued identities are never inverse data.
pub(in crate::store) struct Checkpoint {
    mark: transaction::JournalMark,
    control: super::control::Snapshot,
    _header: custody::Debit,
}
impl Checkpoint {
    pub(in crate::store) fn restore(self, draft: &mut Draft) {
        draft.rollback_journal(self.mark);
        self.control.restore(draft);
    }
}
impl Draft {
    pub(in crate::store) fn source_checkpoint(
        &mut self,
        other_live_bytes: usize,
        other_retained_units: usize,
        cancel: &AtomicBool,
    ) -> Result<Checkpoint, DomError> {
        super::formation::SourceStorage::record_entry();
        // Callers supply an absolute external envelope. Already classified
        // input is in source_other and must not be added a second time.
        let extra_bytes = other_live_bytes.saturating_sub(self.source_other_bytes);
        let extra_units = other_retained_units.saturating_sub(self.source_other_units);
        let extra = custody::Usage::new(extra_bytes, extra_units);
        let header = self
            .custody_ledger()
            .retain(custody::Usage::new(size_of::<Checkpoint>(), 0), |usage| {
                self.admit_custody_usage(usage.checked_add(extra)?, cancel)
            })?;
        let mark = self.journal_mark(extra_bytes, extra_units, cancel)?;
        let control = super::control::Snapshot::capture(self);
        self.source_other_bytes = self.source_other_bytes.max(other_live_bytes);
        self.source_other_units = self.source_other_units.max(other_retained_units);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::source_external_changed(
                self.custody_ledger().measurement_identity(),
                custody::Usage::new(self.source_other_bytes, self.source_other_units),
            );
        }
        Ok(Checkpoint {
            mark,
            control,
            _header: header,
        })
    }
    pub(in crate::store) fn source_storage_bytes(
        &self,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        Ok(self.controlled_storage_usage(cancel)?.bytes())
    }
}
