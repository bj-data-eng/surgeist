//! Checked custody of materialized objects, separate from immutable semantics.
//!
//! `Usage` is a lease-free receipt and may accompany canonical values. Ledger,
//! debit and holder values belong only to controlled owner-side storage. A new
//! materialized field acquires a new debit even when its backing is shared;
//! aliases of the same materialized object clone its `SharedDebit` instead.
//!
//! Admission owns cancellation, finite limits and spent work. This module never
//! restores those facts when a debit is refused, transferred or released. It
//! does not discover object identity, scan roots or infer custody from Arc counts.
//! A bank/frontier owner pairs semantic objects with these tokens, reusing the
//! token for an identity-equal controlled subtree and acquiring only genuinely
//! new objects while adopting a borrowed historical input.

use std::{
    mem::{self, size_of},
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
};

use super::DomError;

#[cfg(test)]
use super::scale_metrics::{self, Domain, Event};

#[cfg(test)]
#[inline]
fn metric(event: Event) {
    if scale_metrics::ENABLED {
        scale_metrics::record(event);
    }
}

/// A private materialization identity, unrelated to slots or public identities.
/// One namespace spans all bank types/owners and is never rolled back/reused.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MaterializationId(u64);

struct IdentityAllocator(AtomicU64);

impl IdentityAllocator {
    const fn new(next: u64) -> Self {
        Self(AtomicU64::new(next))
    }

    fn allocate(&self) -> Result<MaterializationId, DomError> {
        self.0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map(MaterializationId)
            .map_err(|_| DomError::RepresentationExhausted)
    }
}

static IDENTITIES: IdentityAllocator = IdentityAllocator::new(1);

impl MaterializationId {
    pub fn fresh() -> Result<Self, DomError> {
        IDENTITIES.allocate()
    }

    pub const fn key(self) -> u64 {
        self.0
    }
}

/// Retained bytes and capacity-based UTF-16 extent, without ownership authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Usage {
    bytes: usize,
    units: usize,
}

impl Usage {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(bytes: usize, units: usize) -> Self {
        Self { bytes, units }
    }

    pub const fn bytes(self) -> usize {
        self.bytes
    }

    pub const fn units(self) -> usize {
        self.units
    }

    pub fn checked_add(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            units: self
                .units
                .checked_add(other.units)
                .ok_or(DomError::RepresentationExhausted)?,
        })
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, DomError> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_sub(other.bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            units: self
                .units
                .checked_sub(other.units)
                .ok_or(DomError::RepresentationExhausted)?,
        })
    }

    /// Update a component receipt. This does not admit replacement overlap or
    /// change an owned debit; retain the actual new object before its old one
    /// can be discarded or transferred into an inverse journal.
    pub fn checked_replace(self, old: Self, new: Self) -> Result<Self, DomError> {
        self.checked_sub(old)?.checked_add(new)
    }
}

/// One controlled custody domain. Cloning this handle creates no new domain.
#[derive(Clone, Debug)]
pub(crate) struct Ledger {
    total: Arc<Mutex<Usage>>,
}

impl Default for Ledger {
    fn default() -> Self {
        let total = Arc::new(Mutex::new(Usage::ZERO));
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<Mutex<Usage>>(),
        });
        Self { total }
    }
}

impl Ledger {
    /// The owner admits this actual allocation once as its administration. Its
    /// inline Ledger handle is already part of the containing owner's layout.
    /// Arc control-block and opaque allocator metadata remain excluded.
    pub const fn administration_usage() -> Usage {
        Usage::new(size_of::<Mutex<Usage>>(), 0)
    }

    fn lock(&self) -> MutexGuard<'_, Usage> {
        self.total
            .lock()
            .expect("controlled custody ledger invariant has not panicked")
    }

    pub fn total(&self) -> Usage {
        *self.lock()
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_raw_allocations(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
    ) {
        graph.target(&self.total, "control.ledger");
    }

    #[cfg(test)]
    pub fn measurement_identity(&self) -> usize {
        Arc::as_ptr(&self.total) as usize
    }

    /// Acquire one materialized occurrence before retaining its allocation.
    /// The callback receives the prospective domain total and owns all policy
    /// and work admission. Refusal changes neither retained component.
    ///
    /// The callback runs without a ledger lock. If another controlled owner
    /// changes custody while it runs, the new total is admitted again before
    /// committing. The exclusive DOM owner ordinarily calls it exactly once;
    /// any repeated admission work remains the caller's spent work.
    pub fn retain(
        &self,
        usage: Usage,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Debit, DomError> {
        #[cfg(test)]
        metric(Event::Entered(Domain::Accounting));
        loop {
            let before = self.total();
            let next = before.checked_add(usage)?;
            admit(next)?;
            let mut current = self.lock();
            if *current != before {
                continue;
            }
            *current = next;
            drop(current);
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::source_ledger_changed(self.measurement_identity(), next);
            }
            return Ok(Debit {
                ledger: self.clone(),
                usage,
            });
        }
    }

    /// Acquire an object shared only by controlled custodians. Include the
    /// actual out-of-line Debit metadata before allocating its Arc target.
    /// Each containing holder's inline SharedDebit handle belongs to that
    /// holder's own layout/capacity debit, rather than being added here again.
    pub fn retain_shared(
        &self,
        usage: Usage,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<SharedDebit, DomError> {
        let usage = usage.checked_add(SharedDebit::administration_usage())?;
        let retained = SharedDebit(Arc::new(self.retain(usage, admit)?));
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<Debit>(),
        });
        Ok(retained)
    }
}

/// Affine custody of an admitted extent. Move it with the represented storage.
/// It is deliberately not Clone: cloning a field requires a new admission.
#[derive(Debug)]
pub(crate) struct Debit {
    ledger: Ledger,
    usage: Usage,
}

impl Debit {
    pub fn usage(&self) -> Usage {
        self.usage
    }

    pub fn belongs_to(&self, ledger: &Ledger) -> bool {
        Arc::ptr_eq(&self.ledger.total, &ledger.total)
    }

    /// Transfer this admitted extent into shared metadata, acquiring only that
    /// metadata's additional storage. Refusal preserves the original affine
    /// debit beside its semantic holder; success leaves this debit empty.
    pub fn share(
        &mut self,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<SharedDebit, DomError> {
        self.grow(SharedDebit::administration_usage(), admit)?;
        let transferred = self.split(self.usage)?;
        let shared = SharedDebit(Arc::new(transferred));
        #[cfg(test)]
        metric(Event::StructuralAllocation {
            bytes: size_of::<Debit>(),
        });
        Ok(shared)
    }

    /// Transfer a component to its new real holder, without changing totals.
    /// Any newly allocated holder/control storage requires its own admission.
    pub fn split(&mut self, usage: Usage) -> Result<Self, DomError> {
        let remainder = self.usage.checked_sub(usage)?;
        self.usage = remainder;
        Ok(Self {
            ledger: self.ledger.clone(),
            usage,
        })
    }

    /// Transfer an already admitted extent from the same domain. A mismatched
    /// domain returns its original token intact; neither domain gains credit.
    pub fn absorb(&mut self, mut other: Self) -> Result<(), Self> {
        if !Arc::ptr_eq(&self.ledger.total, &other.ledger.total) {
            return Err(other);
        }
        // Both disjoint extents are already included in a checked domain sum.
        self.usage = self
            .usage
            .checked_add(other.usage)
            .expect("same-domain owned extents fit their admitted total");
        other.usage = Usage::ZERO;
        Ok(())
    }

    /// Acquire additional live storage. For replacement, this must include the
    /// full new backing while old backing remains represented by this debit;
    /// subtracting the old receipt first would omit the required peak.
    pub fn grow(
        &mut self,
        usage: Usage,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let added = self.ledger.retain(usage, admit)?;
        self.absorb(added)
            .expect("additional custody was acquired from this same ledger");
        Ok(())
    }
}

impl Drop for Debit {
    fn drop(&mut self) {
        #[cfg(test)]
        metric(Event::Entered(Domain::Accounting));
        let mut total = self.ledger.lock();
        *total = total
            .checked_sub(self.usage)
            .expect("released owned extent is included in the admitted total");
        #[cfg(test)]
        let next = *total;
        drop(total);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::source_ledger_changed(self.ledger.measurement_identity(), next);
        }
    }
}

/// Aliases one actual controlled record/page occurrence. Its last controlled
/// owner releases its debit, independently of external semantic Arc owners.
#[derive(Clone, Debug)]
pub(crate) struct SharedDebit(Arc<Debit>);

impl SharedDebit {
    pub const fn administration_usage() -> Usage {
        Usage::new(size_of::<Debit>(), 0)
    }

    pub fn usage(&self) -> Usage {
        self.0.usage()
    }

    pub fn belongs_to(&self, ledger: &Ledger) -> bool {
        Arc::ptr_eq(&self.0.ledger.total, &ledger.total)
    }

    pub fn shares_domain(&self, debit: &Debit) -> bool {
        Arc::ptr_eq(&self.0.ledger.total, &debit.ledger.total)
    }

    #[cfg(test)]
    pub fn allocation_identity(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }
}

/// Storage and its affine debit in destruction order. Moving this whole holder
/// into an undo entry or output transfers custody without releasing it. It has
/// no Clone implementation and does not attach tokens to immutable values.
pub(crate) struct Held<T> {
    value: T,
    debit: Debit,
}

impl<T> Held<T> {
    /// `debit` must already cover this value's actual occurrence and any newly
    /// allocated holder storage. The value is destroyed before debit release.
    pub fn new(value: T, debit: Debit) -> Self {
        Self { value, debit }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    /// Only the owning mutation barrier may change the represented storage;
    /// it preadmits growth/replacement before using this mutable reference.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn debit(&self) -> &Debit {
        &self.debit
    }

    pub fn debit_mut(&mut self) -> &mut Debit {
        &mut self.debit
    }

    /// Move the old value and its debit together into the actual undo owner.
    /// The replacement must already have admitted simultaneous retention.
    pub fn replace(&mut self, replacement: Self) -> Self {
        mem::replace(self, replacement)
    }

    /// Transfer both parts to an owner with a different private representation.
    /// That owner must retain destruction order: data before its debit.
    pub fn into_parts(self) -> (T, Debit) {
        (self.value, self.debit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DomResource;

    fn admit(limit: Usage) -> impl FnMut(Usage) -> Result<(), DomError> {
        move |usage| {
            if usage.bytes() > limit.bytes() {
                return Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit: limit.bytes(),
                });
            }
            if usage.units() > limit.units() {
                return Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedUnits,
                    limit: limit.units(),
                });
            }
            Ok(())
        }
    }

    fn unlimited(_: Usage) -> Result<(), DomError> {
        Ok(())
    }

    #[test]
    fn checked_materialization_exhaustion_does_not_wrap_or_recycle_ids() {
        let identities = IdentityAllocator::new(u64::MAX - 1);
        assert_eq!(identities.allocate().unwrap().key(), u64::MAX - 1);
        assert_eq!(
            identities.allocate(),
            Err(DomError::RepresentationExhausted)
        );
        assert_eq!(
            identities.allocate(),
            Err(DomError::RepresentationExhausted)
        );
    }

    #[test]
    fn component_receipts_check_both_dimensions_and_replace_only_the_changed_component() {
        let total = Usage::new(37, 19);
        assert_eq!(
            total
                .checked_replace(Usage::new(12, 8), Usage::new(5, 3))
                .unwrap(),
            Usage::new(30, 14)
        );
        for (left, right) in [
            (Usage::new(usize::MAX, 0), Usage::new(1, 0)),
            (Usage::new(0, usize::MAX), Usage::new(0, 1)),
        ] {
            assert_eq!(
                left.checked_add(right),
                Err(DomError::RepresentationExhausted)
            );
        }
        for missing in [Usage::new(38, 0), Usage::new(0, 20)] {
            assert_eq!(
                total.checked_sub(missing),
                Err(DomError::RepresentationExhausted)
            );
        }
    }

    #[test]
    fn one_short_peak_refusal_changes_neither_dimension_and_keeps_admission_work_spent() {
        let ledger = Ledger::default();
        let old = ledger.retain(Usage::new(13, 7), unlimited).unwrap();
        let work = AtomicU64::new(0);
        for limit in [Usage::new(23, 12), Usage::new(24, 11)] {
            let mut policy = admit(limit);
            let result = ledger.retain(Usage::new(11, 5), |next| {
                work.fetch_add(1, Ordering::Relaxed);
                policy(next)
            });
            assert!(matches!(result, Err(DomError::ResourceLimit { .. })));
            assert_eq!(ledger.total(), Usage::new(13, 7));
        }
        assert_eq!(work.load(Ordering::Relaxed), 2);
        let candidate = ledger
            .retain(Usage::new(11, 5), admit(Usage::new(24, 12)))
            .unwrap();
        assert_eq!(ledger.total(), Usage::new(24, 12));
        drop(candidate);
        assert_eq!(ledger.total(), Usage::new(13, 7));
        drop(old);
        assert_eq!(ledger.total(), Usage::ZERO);
        assert_eq!(work.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn arithmetic_exhaustion_does_not_call_admission_or_change_the_live_total() {
        let ledger = Ledger::default();
        let retained = ledger
            .retain(Usage::new(usize::MAX, usize::MAX), unlimited)
            .unwrap();
        let calls = AtomicU64::new(0);
        for addition in [Usage::new(1, 0), Usage::new(0, 1)] {
            assert!(matches!(
                ledger.retain(addition, |_| {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }),
                Err(DomError::RepresentationExhausted)
            ));
        }
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert_eq!(ledger.total(), Usage::new(usize::MAX, usize::MAX));
        drop(retained);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn controlled_root_aliases_release_one_occurrence_only_after_the_last_alias() {
        let ledger = Ledger::default();
        let record = ledger.retain_shared(Usage::new(41, 17), unlimited).unwrap();
        let expected = Usage::new(41 + size_of::<Debit>(), 17);
        assert_eq!(record.usage(), expected);
        let root = record.clone();
        let mark = root.clone();
        assert_eq!(ledger.total(), expected);
        drop(record);
        drop(root);
        assert_eq!(ledger.total(), expected);
        drop(mark);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn copied_leaf_handles_add_only_leaf_storage_and_keep_shared_records_live() {
        let ledger = Ledger::default();
        let record = ledger.retain_shared(Usage::new(29, 11), unlimited).unwrap();
        let record_bytes = 29 + size_of::<Debit>();
        let first_records = vec![record];
        let first_bytes =
            size_of::<Vec<SharedDebit>>() + first_records.capacity() * size_of::<SharedDebit>();
        let first = Held::new(
            first_records,
            ledger
                .retain(Usage::new(first_bytes, 0), unlimited)
                .unwrap(),
        );
        let second_records = first.value().clone();
        let second_bytes =
            size_of::<Vec<SharedDebit>>() + second_records.capacity() * size_of::<SharedDebit>();
        let second = Held::new(
            second_records,
            ledger
                .retain(Usage::new(second_bytes, 0), unlimited)
                .unwrap(),
        );
        assert_eq!(
            ledger.total(),
            Usage::new(record_bytes + first_bytes + second_bytes, 11)
        );
        drop(first);
        assert_eq!(ledger.total(), Usage::new(record_bytes + second_bytes, 11));
        drop(second);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn distinct_materialized_fields_charge_shared_backing_per_occurrence() {
        let ledger = Ledger::default();
        let mut units = Vec::with_capacity(31);
        units.extend([65u16, 66]);
        let external_snapshot = Arc::new(units);
        let field_bytes = size_of::<Vec<u16>>() + external_snapshot.capacity() * size_of::<u16>();
        let field_units = external_snapshot.capacity();
        let first = Held::new(
            external_snapshot.clone(),
            ledger
                .retain(Usage::new(field_bytes, field_units), unlimited)
                .unwrap(),
        );
        let second = Held::new(
            external_snapshot.clone(),
            ledger
                .retain(Usage::new(field_bytes, field_units), unlimited)
                .unwrap(),
        );
        assert!(Arc::ptr_eq(first.value(), second.value()));
        assert_eq!(ledger.total(), Usage::new(field_bytes * 2, field_units * 2));
        drop(first);
        assert_eq!(ledger.total(), Usage::new(field_bytes, field_units));
        drop(second);
        assert_eq!(ledger.total(), Usage::ZERO);
        assert_eq!(external_snapshot.as_slice(), &[65, 66]);
    }

    #[test]
    fn split_and_absorb_transfer_existing_custody_without_releasing_or_readmitting() {
        let ledger = Ledger::default();
        let mut whole = ledger.retain(Usage::new(23, 13), unlimited).unwrap();
        let part = whole.split(Usage::new(7, 5)).unwrap();
        assert_eq!(whole.usage(), Usage::new(16, 8));
        assert_eq!(part.usage(), Usage::new(7, 5));
        assert_eq!(ledger.total(), Usage::new(23, 13));
        whole.absorb(part).unwrap();
        assert_eq!(whole.usage(), Usage::new(23, 13));
        assert_eq!(ledger.total(), Usage::new(23, 13));
        assert!(whole.split(Usage::new(24, 0)).is_err());
        assert_eq!(whole.usage(), Usage::new(23, 13));
        drop(whole);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn mismatched_ledger_transfer_returns_original_debit_without_discharge() {
        let first = Ledger::default();
        let second = Ledger::default();
        let mut destination = first.retain(Usage::new(3, 2), unlimited).unwrap();
        let source = second.retain(Usage::new(7, 5), unlimited).unwrap();
        let returned = destination.absorb(source).unwrap_err();
        assert_eq!(first.total(), Usage::new(3, 2));
        assert_eq!(second.total(), Usage::new(7, 5));
        assert_eq!(returned.usage(), Usage::new(7, 5));
        drop(returned);
        assert_eq!(second.total(), Usage::ZERO);
        drop(destination);
        assert_eq!(first.total(), Usage::ZERO);
    }

    #[test]
    fn nested_inverse_holders_retain_old_fields_until_each_real_owner_releases_them() {
        let ledger = Ledger::default();
        let field = |value, usage| Held::new(value, ledger.retain(usage, unlimited).unwrap());
        let mut current = field(10, Usage::new(5, 2));
        let outer_undo = current.replace(field(20, Usage::new(7, 3)));
        let inner_undo = current.replace(field(30, Usage::new(11, 5)));
        assert_eq!(ledger.total(), Usage::new(23, 10));
        // Roll back only the inner mark; the outer field remains retained.
        drop(current.replace(inner_undo));
        assert_eq!(*current.value(), 20);
        assert_eq!(ledger.total(), Usage::new(12, 5));
        // An accepted outer change discards only its obsolete old field.
        drop(outer_undo);
        assert_eq!(ledger.total(), Usage::new(7, 3));
        current
            .debit_mut()
            .grow(Usage::new(0, 0), unlimited)
            .unwrap();
        *current.value_mut() = 21;
        assert_eq!(current.debit().usage(), Usage::new(7, 3));
        let (value, debit) = current.into_parts();
        assert_eq!(value, 21);
        assert_eq!(ledger.total(), Usage::new(7, 3));
        drop(debit);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn refused_growth_preserves_the_old_extent_and_admitted_growth_counts_overlap() {
        let ledger = Ledger::default();
        let mut debit = ledger.retain(Usage::new(5, 2), unlimited).unwrap();
        assert!(
            debit
                .grow(Usage::new(7, 3), admit(Usage::new(11, 5)))
                .is_err()
        );
        assert_eq!(debit.usage(), Usage::new(5, 2));
        assert_eq!(ledger.total(), Usage::new(5, 2));
        debit
            .grow(Usage::new(7, 3), admit(Usage::new(12, 5)))
            .unwrap();
        assert_eq!(debit.usage(), Usage::new(12, 5));
        assert_eq!(ledger.total(), Usage::new(12, 5));
        let obsolete = debit.split(Usage::new(5, 2)).unwrap();
        drop(obsolete);
        assert_eq!(ledger.total(), Usage::new(7, 3));
    }

    struct ObserveDrop {
        ledger: Ledger,
        observed: Arc<Mutex<Option<Usage>>>,
    }

    impl Drop for ObserveDrop {
        fn drop(&mut self) {
            *self.observed.lock().unwrap() = Some(self.ledger.total());
        }
    }

    #[test]
    fn moved_holder_destroys_data_before_releasing_its_debit() {
        let ledger = Ledger::default();
        let observed = Arc::new(Mutex::new(None));
        let held = Held::new(
            ObserveDrop {
                ledger: ledger.clone(),
                observed: observed.clone(),
            },
            ledger.retain(Usage::new(17, 7), unlimited).unwrap(),
        );
        std::thread::spawn(move || drop(held)).join().unwrap();
        assert_eq!(*observed.lock().unwrap(), Some(Usage::new(17, 7)));
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn admission_can_read_the_domain_and_rechecks_a_changed_prospective_total() {
        let ledger = Ledger::default();
        let mut original = Some(ledger.retain(Usage::new(13, 5), unlimited).unwrap());
        let mut attempts = Vec::new();
        let acquired = ledger
            .retain(Usage::new(7, 3), |prospective| {
                attempts.push(prospective);
                if let Some(old) = original.take() {
                    assert_eq!(ledger.total(), Usage::new(13, 5));
                    drop(old);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(attempts, [Usage::new(20, 8), Usage::new(7, 3)]);
        assert_eq!(ledger.total(), Usage::new(7, 3));
        drop(acquired);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn shared_metadata_and_owner_administration_are_explicit_conservative_extents() {
        assert_eq!(
            Ledger::administration_usage(),
            Usage::new(size_of::<Mutex<Usage>>(), 0)
        );
        let ledger = Ledger::default();
        let amount = Usage::new(19 + size_of::<Debit>(), 5);
        assert!(
            ledger
                .retain_shared(Usage::new(19, 5), admit(Usage::new(amount.bytes() - 1, 5)))
                .is_err()
        );
        assert_eq!(ledger.total(), Usage::ZERO);
        let shared = ledger
            .retain_shared(Usage::new(19, 5), admit(amount))
            .unwrap();
        assert_eq!(ledger.total(), amount);
        drop(shared);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn source_measurement_observes_committed_selected_ledger_changes_and_excludes_refused_peaks() {
        if !scale_metrics::ENABLED {
            return;
        }
        let measurement = scale_metrics::Measurement::begin().unwrap();
        let ledger = Ledger::default();
        scale_metrics::select_source_ledger(
            ledger.measurement_identity(),
            ledger.total(),
            Usage::new(7, 2),
        );
        let retained = ledger.retain(Usage::new(19, 5), unlimited).unwrap();
        assert!(
            ledger
                .retain(Usage::new(100, 0), |_| Err(DomError::Cancelled))
                .is_err()
        );
        let unrelated = Ledger::default();
        let unrelated = unrelated.retain(Usage::new(1000, 100), unlimited).unwrap();
        drop(retained);
        drop(unrelated);
        let counters = measurement
            .finish(scale_metrics::Hooks::one(Domain::Accounting))
            .unwrap();
        assert_eq!(
            (counters.source_peak_bytes, counters.source_peak_units),
            (26, 7)
        );
        assert_eq!(
            (counters.source_live_bytes, counters.source_live_units),
            (7, 2)
        );
    }

    #[test]
    fn sharing_an_existing_debit_preserves_it_on_refusal_and_transfers_without_payload_readmission()
    {
        let ledger = Ledger::default();
        let payload = Usage::new(19, 5);
        let mut debit = ledger.retain(payload, unlimited).unwrap();
        let exact = payload
            .checked_add(SharedDebit::administration_usage())
            .unwrap();
        assert!(
            debit
                .share(admit(Usage::new(exact.bytes() - 1, 5)))
                .is_err()
        );
        assert_eq!(debit.usage(), payload);
        assert_eq!(ledger.total(), payload);
        let mut attempts = Vec::new();
        let shared = debit
            .share(|next| {
                attempts.push(next);
                Ok(())
            })
            .unwrap();
        assert_eq!(attempts, [exact]);
        assert_eq!(debit.usage(), Usage::ZERO);
        assert_eq!(ledger.total(), exact);
        drop(debit);
        assert_eq!(ledger.total(), exact);
        drop(shared);
        assert_eq!(ledger.total(), Usage::ZERO);
    }
}
