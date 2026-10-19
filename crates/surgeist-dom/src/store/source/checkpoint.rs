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
    /// Release an unused portion after its represented storage has already
    /// been destroyed. The surviving allocation remains beside this affine
    /// receipt. This subset transfer never allocates, admits, or grows custody,
    /// so it is valid in an infallible post-publication handoff.
    pub(in crate::store) fn shrink_to_payload(&mut self, payload: super::storage::Payload) {
        let bytes = self
            .bytes
            .checked_sub(payload.bytes)
            .expect("surviving Source bytes are a preadmitted subset");
        let units = self
            .units
            .checked_sub(payload.units)
            .expect("surviving Source units are a preadmitted subset");
        if bytes != 0 || units != 0 {
            drop(
                self.debit
                    .as_mut()
                    .expect("owned Source payload")
                    .split(custody::Usage::new(bytes, units))
                    .expect("unused Source payload component"),
            );
        }
        self.live.fetch_sub(bytes, Ordering::Relaxed);
        self.live_units.fetch_sub(units, Ordering::Relaxed);
        self.bytes = payload.bytes;
        self.units = payload.units;
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
    sharing: SharingCheckpoint,
    _header: custody::Debit,
}
enum SharingCheckpoint {
    Absent,
    Dormant,
    Marked(value_sharing::Checkpoint),
}
impl Checkpoint {
    pub(in crate::store) fn sharing(&self) -> Option<&value_sharing::Checkpoint> {
        match &self.sharing {
            SharingCheckpoint::Marked(mark) => Some(mark),
            SharingCheckpoint::Absent | SharingCheckpoint::Dormant => None,
        }
    }
    pub(in crate::store) fn restore(self, draft: &mut Draft) {
        // Journal rollback first validates this transaction, publication epoch,
        // and nested boundary before a cold boundary may reset its attachment.
        draft.rollback_journal(self.mark);
        self.control.restore(draft);
        match self.sharing {
            SharingCheckpoint::Absent => drop(draft.sharing.take()),
            SharingCheckpoint::Dormant => draft
                .sharing_session()
                .expect("a dormant boundary retains its attachment")
                .reset_dormant(),
            SharingCheckpoint::Marked(mark) => mark.restore(),
        }
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
        let sharing = match self.sharing_session() {
            None => SharingCheckpoint::Absent,
            Some(session) if session.is_dormant() => SharingCheckpoint::Dormant,
            Some(session) => {
                SharingCheckpoint::Marked(session.checkpoint(&mut value_sharing::Admission {
                    global: &mut |usage| {
                        self.admit_custody_usage(usage.checked_add(extra)?, cancel)
                    },
                    step: &mut || self.source_step(cancel),
                })?)
            }
        };
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
            sharing,
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

#[cfg(test)]
mod subset_tests {
    use super::*;

    #[test]
    fn dropped_spares_shrink_and_merge_without_admission_or_refunding_work() {
        let cancel = AtomicBool::new(false);
        let dom = Dom::new(
            DomLimits::new(
                8,
                4096,
                64,
                0,
                0,
                0,
                0,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let batch = DomBatch::new();
        let mut draft = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 16, 64, 64, 1),
            &cancel,
        )
        .unwrap()
        .0;
        draft.custody_registry().reconcile_releases();
        let ledger = draft.custody_ledger().clone();
        let baseline = ledger.total();
        let mirrors = (
            draft.source_formation_bytes.load(Ordering::Relaxed),
            draft.source_temporary_units.load(Ordering::Relaxed),
        );
        let first = super::super::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let second = super::super::formation::Formation::new(&draft, &cancel)
            .unwrap()
            .unwrap();
        let mut kept = Vec::new();
        let mut spare = Vec::new();
        let mut other = Vec::new();
        super::super::formation::reserve_utf16(&mut kept, 7, Some(&first), &cancel).unwrap();
        super::super::formation::reserve_utf16(&mut spare, 19, Some(&first), &cancel).unwrap();
        super::super::formation::reserve_utf16(&mut other, 3, Some(&second), &cancel).unwrap();
        kept.extend_from_slice(&[1, 2, 3]);
        other.push(4);
        let kept_units = kept.capacity();
        let other_units = other.capacity();
        struct Buffers {
            kept: Vec<u16>,
            spare: Vec<u16>,
            other: Vec<u16>,
            primary: Charge,
            secondary: Option<Charge>,
        }
        let mut buffers = Buffers {
            kept,
            spare,
            other,
            primary: first.into_charge(),
            secondary: Some(second.into_charge()),
        };
        let spent = draft.source_work.load(Ordering::Relaxed);
        draft.source_limits.as_mut().unwrap().work = spent;
        draft.source_limits.as_mut().unwrap().retained_bytes = baseline.bytes();
        draft.source_limits.as_mut().unwrap().retained_units = baseline.units();
        cancel.store(true, Ordering::Relaxed);
        // The unused allocation is destroyed while its original receipt still
        // owns it. Only then may the infallible subset handoff release surplus.
        drop(std::mem::take(&mut buffers.spare));
        buffers
            .primary
            .shrink_to_payload(super::super::storage::Payload {
                bytes: kept_units * size_of::<u16>(),
                units: kept_units,
            });
        let remaining = custody::Usage::new(
            (kept_units + other_units) * size_of::<u16>(),
            kept_units + other_units,
        );
        assert_eq!(ledger.total(), baseline.checked_add(remaining).unwrap());
        buffers.primary.merge(buffers.secondary.take().unwrap());
        assert_eq!(ledger.total(), baseline.checked_add(remaining).unwrap());
        assert_eq!(buffers.primary.bytes, remaining.bytes());
        assert_eq!(buffers.primary.units, remaining.units());
        assert_eq!(buffers.kept, [1, 2, 3]);
        assert_eq!(buffers.other, [4]);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
        drop(buffers);
        assert_eq!(ledger.total(), baseline);
        assert_eq!(
            draft.source_formation_bytes.load(Ordering::Relaxed),
            mirrors.0
        );
        assert_eq!(
            draft.source_temporary_units.load(Ordering::Relaxed),
            mirrors.1
        );
    }
}
