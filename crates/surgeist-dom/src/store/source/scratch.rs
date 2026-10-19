//! Mutable algorithm construction banks become immutable shared worklists once
//! parked in a continuation. Snapshotting never copies their retained extent.
use super::effects::ScratchRetained;
use super::*;

struct Mutable<T> {
    values: Vec<T>,
    debit: Option<custody::Debit>,
    scratch: ScratchRetained,
}
struct Shared<T> {
    values: Mutable<T>,
    _debit: custody::Debit,
    _scratch: ScratchRetained,
}
enum Storage<T> {
    Mutable(Mutable<T>),
    Shared(Arc<Shared<T>>),
}
pub(in crate::store) struct ScratchVec<T> {
    storage: Storage<T>,
    mirrors: formation::SourceMirrors,
    admit: bool,
}
pub(in crate::store) struct OwnedScratch<T>(Mutable<T>);

#[cfg(test)]
struct MutationObservation<'a, T> {
    values: &'a mut Vec<T>,
    length: usize,
    capacity: usize,
    pointer: *const T,
    shifted: usize,
}
// Reached bulk callers reserve their complete append extent first: tree
// worklists use exact-size child iterators, and subtree exposure reserves both
// optional owned roots. This observes the completed mutation, not allocator
// activity hidden inside an arbitrary, unreserved standard-library iterator.
#[cfg(test)]
impl<'a, T> MutationObservation<'a, T> {
    fn new(values: &'a mut Vec<T>) -> Self {
        Self {
            length: values.len(),
            capacity: values.capacity(),
            pointer: values.as_ptr(),
            values,
            shifted: 0,
        }
    }
}
#[cfg(test)]
impl<T> Drop for MutationObservation<'_, T> {
    fn drop(&mut self) {
        let mut copied = transaction::CopyObservation::default();
        if self.values.capacity() != self.capacity {
            if scale_metrics::ENABLED && size_of::<T>() != 0 {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                    bytes: self.values.capacity() * size_of::<T>(),
                });
            }
            if self.values.as_ptr() != self.pointer {
                copied.extended::<T>(self.length);
            }
        }
        copied.extended::<T>(self.values.len().saturating_sub(self.length));
        copied.extended::<T>(self.shifted);
    }
}
impl<T> OwnedScratch<T> {
    pub(in crate::store) fn values(&self) -> &[T] {
        &self.0.values
    }
    pub(in crate::store) fn into_held(
        self,
        ledger: &custody::Ledger,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<custody::Held<Vec<T>>, DomError> {
        let empty = if self.0.debit.is_none() {
            assert_eq!(self.0.values.capacity(), 0, "unallocated construction bank");
            Some(ledger.retain(custody::Usage::ZERO, admit)?)
        } else {
            None
        };
        let Mutable {
            values,
            debit,
            scratch,
        } = self.0;
        let held = custody::Held::new(
            values,
            debit.or(empty).expect("owned construction capacity"),
        );
        // The selected field now owns the same physical allocation and affine
        // debit. Its previous algorithm-scratch classification can end.
        drop(scratch);
        Ok(held)
    }
}

impl<T> std::ops::Deref for ScratchVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        match &self.storage {
            Storage::Mutable(value) => &value.values,
            Storage::Shared(value) => &value.values.values,
        }
    }
}
impl<T> std::ops::DerefMut for ScratchVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Storage::Mutable(value) = &mut self.storage else {
            unreachable!("parked immutable worklists use separate continuation cursors")
        };
        &mut value.values
    }
}
impl<T> ScratchVec<T> {
    #[cfg(test)]
    pub(in crate::store) fn push(&mut self, value: T) {
        let observed = MutationObservation::new(self);
        observed.values.push(value);
    }
    #[cfg(test)]
    pub(in crate::store) fn extend(&mut self, values: impl IntoIterator<Item = T>) {
        let observed = MutationObservation::new(self);
        observed.values.extend(values);
    }
    #[cfg(test)]
    pub(in crate::store) fn insert(&mut self, index: usize, value: T) {
        let mut observed = MutationObservation::new(self);
        observed.values.insert(index, value);
        observed.shifted = observed.length - index;
    }
    #[cfg(test)]
    pub(in crate::store) fn append(&mut self, values: &mut Vec<T>) {
        let observed = MutationObservation::new(self);
        observed.values.append(values);
    }
    #[cfg(test)]
    pub(in crate::store) fn resize(&mut self, length: usize, value: T)
    where
        T: Clone,
    {
        let observed = MutationObservation::new(self);
        observed.values.resize(length, value);
    }
    #[cfg(test)]
    pub(in crate::store) fn resize_with(&mut self, length: usize, value: impl FnMut() -> T) {
        let observed = MutationObservation::new(self);
        observed.values.resize_with(length, value);
    }
    #[cfg(test)]
    pub(in crate::store) fn extend_from_slice(&mut self, values: &[T])
    where
        T: Clone,
    {
        let observed = MutationObservation::new(self);
        observed.values.extend_from_slice(values);
    }
    #[cfg(test)]
    pub(in crate::store) fn remove(&mut self, index: usize) -> T {
        let mut observed = MutationObservation::new(self);
        let result = observed.values.remove(index);
        observed.shifted = observed.length - index - 1;
        result
    }
    #[cfg(test)]
    pub(in crate::store) fn swap_remove(&mut self, index: usize) -> T {
        let mut observed = MutationObservation::new(self);
        let result = observed.values.swap_remove(index);
        observed.shifted = usize::from(index != observed.values.len());
        result
    }
    pub(in crate::store) fn new(draft: &Draft, admit: bool) -> Self {
        let mirrors = draft.source_mirrors();
        Self {
            storage: Storage::Mutable(Mutable {
                values: Vec::new(),
                debit: None,
                scratch: ScratchRetained::empty(mirrors.clone()),
            }),
            mirrors,
            admit,
        }
    }
    pub(in crate::store) fn into_values(self) -> OwnedScratch<T> {
        let Storage::Mutable(value) = self.storage else {
            unreachable!("only a construction bank transfers directly into a selected field")
        };
        OwnedScratch(value)
    }
    pub(in crate::store) fn reserve(
        &mut self,
        draft: &Draft,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let required = self
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= self.capacity() {
            return Ok(());
        }
        let Storage::Mutable(current) = &mut self.storage else {
            unreachable!("continuation worklist storage is immutable")
        };
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let wanted = current
            .values
            .capacity()
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let requested = wanted
            .checked_mul(size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        let enforce = self.admit || draft.effect_scoped;
        if enforce {
            draft.select_scratch(requested)?;
        }
        let mut debit = draft
            .custody_ledger()
            .retain(custody::Usage::new(requested, 0), |usage| {
                draft.admit_custody_usage(usage, cancel)
            })?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(wanted)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes: actual });
        }
        if actual > requested {
            if enforce {
                draft.select_scratch(actual)?;
            }
            debit.grow(custody::Usage::new(actual - requested, 0), |usage| {
                draft.admit_custody_usage(usage, cancel)
            })?;
        }
        for _ in &current.values {
            draft.source_step(cancel)?;
        }
        let scratch = ScratchRetained::new(draft, actual, false)?;
        values.append(&mut current.values);
        #[cfg(test)]
        if scale_metrics::ENABLED && !values.is_empty() {
            scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
                bytes: values.len() * size_of::<T>(),
            });
        }
        let old = std::mem::replace(
            current,
            Mutable {
                values,
                debit: Some(debit),
                scratch,
            },
        );
        drop(old);
        Ok(())
    }
    /// Prepare a selected append's new actual allocation without moving or
    /// dropping the existing bank. The caller journals the old owner at commit.
    pub(in crate::store) fn prepare_growth_copy(
        &self,
        draft: &Draft,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<Self>, DomError>
    where
        T: Copy,
    {
        let required = self
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= self.capacity() {
            return Ok(None);
        }
        let maximum = isize::MAX as usize / size_of::<T>().max(1);
        let wanted = self
            .capacity()
            .checked_mul(2)
            .filter(|n| *n <= maximum)
            .unwrap_or(required)
            .max(required);
        let mut replacement = Self::new(draft, self.admit);
        replacement.reserve(draft, wanted, cancel)?;
        for value in self.iter() {
            draft.source_step(cancel)?;
            replacement.push(*value);
        }
        Ok(Some(replacement))
    }
    pub(in crate::store) fn freeze(
        &mut self,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if matches!(self.storage, Storage::Shared(_)) {
            return Ok(());
        }
        let bytes = size_of::<Shared<T>>();
        if self.admit || draft.effect_scoped {
            draft.select_scratch(bytes)?;
        }
        let debit = draft
            .custody_ledger()
            .retain(custody::Usage::new(bytes, 0), |usage| {
                draft.admit_custody_usage(usage, cancel)
            })?;
        let scratch = ScratchRetained::new(draft, bytes, false)?;
        let empty = Mutable {
            values: Vec::new(),
            debit: None,
            scratch: ScratchRetained::empty(self.mirrors.clone()),
        };
        let Storage::Mutable(values) =
            std::mem::replace(&mut self.storage, Storage::Mutable(empty))
        else {
            unreachable!()
        };
        self.storage = Storage::Shared(formation::allocated_arc(Shared {
            values,
            _debit: debit,
            _scratch: scratch,
        }));
        Ok(())
    }
    pub(in crate::store) fn checkpoint(
        &self,
        _admission: &mut checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        check_cancel(cancel)?;
        let Storage::Shared(value) = &self.storage else {
            unreachable!("worklists freeze before continuation capture")
        };
        Ok(Self {
            storage: Storage::Shared(value.clone()),
            mirrors: self.mirrors.clone(),
            admit: self.admit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_mutators_report_initialized_values_and_real_suffix_moves() {
        use scale_metrics::{Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let dom = Dom::new(
            DomLimits::new(
                8,
                4096,
                32,
                0,
                0,
                8,
                65_536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let batch = DomBatch::new();
        let draft = Draft::new_source(
            &dom,
            batch.owner,
            &batch.commands,
            crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 32, 128, 128, 1),
            &cancel,
        )
        .unwrap()
        .0;
        let mut values = ScratchVec::new(&draft, false);
        values.reserve(&draft, 16, &cancel).unwrap();
        let mut additional = vec![6_u64, 7];
        let measurement = Measurement::begin().unwrap();
        values.push(1);
        values.push(2);
        values.extend([3, 4]);
        values.insert(1, 9);
        values.append(&mut additional);
        values.resize(9, 0);
        values.resize_with(11, || 5);
        values.extend_from_slice(&[8, 9]);
        assert_eq!(values.remove(0), 1);
        assert_eq!(values.swap_remove(0), 9);
        values.truncate(3);
        values.resize(2, 0);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(values.as_slice(), &[9, 2]);
        assert!(additional.is_empty());
        assert_eq!(counters.structural_allocations, 0);
        // Thirteen new values, three insertion shifts, twelve removal
        // shifts and one last-element replacement were actually written.
        assert_eq!(counters.sequence_materialized_bytes, 29 * size_of::<u64>());
        assert_eq!(counters.sequence_materializations, 10);
    }
    #[test]
    fn scratch_growth_reports_initialized_moves_and_completed_copy_prefix() {
        use scale_metrics::{Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let dom = Dom::new(
            DomLimits::new(
                128,
                1_000_000,
                1024,
                128,
                128,
                32,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let batch = DomBatch::new();
        let cancel = AtomicBool::new(false);
        let limits =
            crate::DomSourceLimits::new(64_000_000, 1_000_000, 100_000_000, 64, 4096, 4096, 2);
        let mut draft =
            Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel)
                .unwrap()
                .0;
        let mut values = ScratchVec::new(&draft, false);
        values.reserve(&draft, 8, &cancel).unwrap();
        values.extend([3u64, 5, 8]);
        let additional = values.capacity() + 1;
        let measurement = Measurement::begin().unwrap();
        values.reserve(&draft, additional, &cancel).unwrap();
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(values.as_slice(), &[3, 5, 8]);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            values.capacity() * size_of::<u64>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u64>());

        // One original-Draft step admits the allocation and one copies the
        // first value. The next actual copy is refused by that same policy,
        // leaving exactly one completed element observed.
        draft.source_limits.as_mut().unwrap().work = draft.source_work.load(Ordering::Relaxed) + 2;
        let measurement = Measurement::begin().unwrap();
        let result = values.prepare_growth_copy(&draft, values.capacity() + 1, &cancel);
        assert!(matches!(result, Err(DomError::SourceWorkLimit { .. })));
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u64>());
        assert_eq!(values.as_slice(), &[3, 5, 8]);
    }

    #[test]
    fn parked_worklist_snapshot_shares_exact_backing_and_retained_custody() {
        let dom = Dom::new(
            DomLimits::new(
                128,
                1_000_000,
                1024,
                128,
                128,
                32,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let batch = DomBatch::new();
        let cancel = AtomicBool::new(false);
        let limits =
            crate::DomSourceLimits::new(64_000_000, 1_000_000, 100_000_000, 64, 4096, 4096, 2);
        let draft = Draft::new_source(&dom, batch.owner.clone(), &batch.commands, limits, &cancel)
            .unwrap()
            .0;
        let mut values = ScratchVec::new(&draft, false);
        for index in 0..96_u64 {
            values.reserve(&draft, 1, &cancel).unwrap();
            values.push(index);
        }
        values.freeze(&draft, &cancel).unwrap();
        let usage = draft.custody_ledger().total();
        let work = draft.source_work.load(Ordering::Relaxed);
        let mut admission =
            checkpoint::Admission::new(&draft, usage.bytes() + draft.source_other_bytes).unwrap();
        let alias = values.checkpoint(&mut admission, &cancel).unwrap();
        drop(admission.finish());
        assert_eq!(alias.as_slice(), (0..96_u64).collect::<Vec<_>>());
        assert_eq!(values.as_ptr(), alias.as_ptr());
        assert_eq!(draft.custody_ledger().total(), usage);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), work);
        let ledger = draft.custody_ledger().clone();
        let mirrors = draft.source_mirrors();
        let retained = mirrors.temporary().load(Ordering::Relaxed);
        drop(draft);
        drop(values);
        assert_eq!(alias[95], 95);
        assert_eq!(mirrors.temporary().load(Ordering::Relaxed), retained);
        let before_release = ledger.total();
        drop(alias);
        assert_eq!(mirrors.temporary().load(Ordering::Relaxed), 0);
        assert!(ledger.total().bytes() < before_release.bytes());
    }
}
