//! Affine nested rollback marks with immediately released obsolete custody.

use std::{
    collections::VecDeque,
    mem::size_of,
    sync::{Arc, Mutex, MutexGuard},
};

use super::super::{
    DomError,
    custody::{Debit, Held, Ledger, Usage},
};

struct Marker {
    id: u64,
    position: usize,
}

struct Registry<U> {
    entries: Held<VecDeque<U>>,
    markers: Held<Vec<Marker>>,
    first: usize,
    next_mark: u64,
    epoch: u64,
    #[cfg(test)]
    measure: fn(&U) -> usize,
    #[cfg(test)]
    payload_bytes: usize,
}

struct Shared<U> {
    // Data and retained inverse values are destroyed before their holder debit.
    registry: Mutex<Registry<U>>,
    _debit: Debit,
    #[cfg(test)]
    _observation: JournalObservation,
}

#[cfg(test)]
struct JournalObservation(std::sync::atomic::AtomicUsize);

#[cfg(test)]
impl Drop for JournalObservation {
    fn drop(&mut self) {
        if super::super::scale_metrics::ENABLED {
            super::super::scale_metrics::record(
                super::super::scale_metrics::Event::SavepointBytes {
                    identity: self.0.load(std::sync::atomic::Ordering::Relaxed),
                    live: 0,
                },
            );
        }
    }
}

pub(in crate::store) struct Journal<U> {
    shared: Arc<Shared<U>>,
}

impl<U> Clone for Journal<U> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

/// A mark is affine. Its ordinary destruction accepts the writes and releases
/// inverse values that no surviving mark can use, including on early returns.
pub(in crate::store) struct Mark<U> {
    shared: Arc<Shared<U>>,
    id: u64,
    position: usize,
    epoch: u64,
    live: bool,
}

/// Checked before the publication fence. Its infallible commit seals every
/// private mark belonging to the preceding unpublished segment.
pub(in crate::store) struct Commit<U> {
    shared: Arc<Shared<U>>,
    before: u64,
    after: u64,
}

impl<U> Journal<U> {
    pub fn new(
        ledger: &Ledger,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut debit = ledger.retain(Usage::new(size_of::<Shared<U>>(), 0), admit)?;
        let entries = Held::new(VecDeque::new(), debit.split(Usage::ZERO)?);
        let markers = Held::new(Vec::new(), debit.split(Usage::ZERO)?);
        let journal = Self {
            shared: Arc::new(Shared {
                registry: Mutex::new(Registry {
                    entries,
                    markers,
                    first: 0,
                    next_mark: 1,
                    epoch: 0,
                    #[cfg(test)]
                    measure: |_| 0,
                    #[cfg(test)]
                    payload_bytes: 0,
                }),
                _debit: debit,
                #[cfg(test)]
                _observation: JournalObservation(std::sync::atomic::AtomicUsize::new(0)),
            }),
        };
        #[cfg(test)]
        if super::super::scale_metrics::ENABLED {
            super::super::scale_metrics::record(
                super::super::scale_metrics::Event::StructuralAllocation {
                    bytes: size_of::<Shared<U>>(),
                },
            );
        }
        #[cfg(test)]
        if super::super::scale_metrics::ENABLED {
            journal.shared._observation.0.store(
                Arc::as_ptr(&journal.shared) as usize,
                std::sync::atomic::Ordering::Relaxed,
            );
            observe(&journal.shared, &journal.lock());
        }
        Ok(journal)
    }

    #[cfg(test)]
    pub fn measure_payloads(&self, measure: fn(&U) -> usize) {
        self.lock().measure = measure;
    }

    fn lock(&self) -> MutexGuard<'_, Registry<U>> {
        lock(&self.shared)
    }

    pub fn latest_mark_id(&self) -> Option<u64> {
        self.lock().markers.value().last().map(|mark| mark.id)
    }

    pub fn has_marks(&self) -> bool {
        !self.lock().markers.value().is_empty()
    }

    pub fn mark(
        &self,
        ledger: &Ledger,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Mark<U>, DomError> {
        let mut registry = self.lock();
        let id = registry.next_mark;
        let next = id.checked_add(1).ok_or(DomError::RepresentationExhausted)?;
        let position = registry
            .first
            .checked_add(registry.entries.value().len())
            .ok_or(DomError::RepresentationExhausted)?;
        reserve_vec(&mut registry.markers, 1, ledger, &mut admit)?;
        registry.markers.value_mut().push(Marker { id, position });
        #[cfg(test)]
        super::sequence::record_copied_bytes(size_of::<Marker>());
        #[cfg(test)]
        if super::super::scale_metrics::ENABLED {
            super::super::scale_metrics::record(super::super::scale_metrics::Event::Savepoint);
        }
        registry.next_mark = next;
        let epoch = registry.epoch;
        #[cfg(test)]
        observe(&self.shared, &registry);
        Ok(Mark {
            shared: Arc::clone(&self.shared),
            id,
            position,
            epoch,
            live: true,
        })
    }

    /// Reserve an inverse slot before the owner moves an old field or mutates
    /// a sequence. The matching push is infallible and cannot allocate.
    pub fn prepare(
        &self,
        ledger: &Ledger,
        admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        self.prepare_slots(1, ledger, admit)
    }
    pub fn prepare_slots(
        &self,
        count: usize,
        ledger: &Ledger,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        let mut registry = self.lock();
        if registry.markers.value().is_empty() {
            return Ok(false);
        }
        registry
            .first
            .checked_add(registry.entries.value().len())
            .and_then(|position| position.checked_add(count))
            .ok_or(DomError::RepresentationExhausted)?;
        reserve_deque(&mut registry.entries, count, ledger, &mut admit)?;
        #[cfg(test)]
        observe(&self.shared, &registry);
        Ok(true)
    }

    pub fn push_prepared(&self, undo: U) {
        let mut registry = self.lock();
        if registry.markers.value().is_empty() {
            // Dropping an old field here ends its real obsolete custody.
            drop(undo);
            return;
        }
        assert!(
            registry.entries.value().len() < registry.entries.value().capacity(),
            "inverse slot was admitted before mutation"
        );
        #[cfg(test)]
        {
            registry.payload_bytes = registry
                .payload_bytes
                .checked_add((registry.measure)(&undo))
                .expect("admitted inverse payload receipt sum");
        }
        registry.entries.value_mut().push_back(undo);
        #[cfg(test)]
        super::sequence::record_copied_bytes(size_of::<U>());
        #[cfg(test)]
        observe(&self.shared, &registry);
        #[cfg(test)]
        if super::super::scale_metrics::ENABLED {
            super::super::scale_metrics::record(super::super::scale_metrics::Event::JournalEntry);
        }
    }

    /// Rollback cannot allocate or consult a refusal/cancellation gate. Each
    /// owner inverse was made reversible before its forward write. Work and
    /// external receipts are deliberately absent from this registry.
    pub fn rollback(&self, mut mark: Mark<U>, mut apply: impl FnMut(U)) {
        assert!(
            Arc::ptr_eq(&self.shared, &mark.shared),
            "rollback mark belongs to this transaction"
        );
        {
            let registry = self.lock();
            assert_eq!(
                registry.epoch, mark.epoch,
                "a published segment cannot roll back"
            );
            assert_eq!(
                registry.markers.value().last().map(|entry| entry.id),
                Some(mark.id),
                "nested rollback marks restore in reverse registration order"
            );
        }
        loop {
            let undo = {
                let mut registry = self.lock();
                let end = registry
                    .first
                    .checked_add(registry.entries.value().len())
                    .expect("admitted journal positions fit");
                if end == mark.position {
                    None
                } else {
                    assert!(end > mark.position, "live mark retains its inverse suffix");
                    let undo = registry.entries.value_mut().pop_back();
                    #[cfg(test)]
                    if let Some(value) = &undo {
                        registry.payload_bytes -= (registry.measure)(value);
                    }
                    undo
                }
            };
            let Some(undo) = undo else { break };
            // No registry lock crosses owner restoration or payload destruction.
            apply(undo);
            #[cfg(test)]
            observe(&self.shared, &self.lock());
        }
        release(&mark.shared, mark.id, mark.epoch);
        mark.live = false;
    }

    pub fn prepare_commit(&self) -> Result<Commit<U>, DomError> {
        let registry = self.lock();
        Ok(Commit {
            shared: Arc::clone(&self.shared),
            before: registry.epoch,
            after: registry
                .epoch
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
        })
    }

    pub fn commit(&self, commit: Commit<U>) {
        assert!(
            Arc::ptr_eq(&self.shared, &commit.shared),
            "commit belongs to this journal"
        );
        let mut registry = self.lock();
        assert_eq!(registry.epoch, commit.before, "prepared commit is current");
        registry.epoch = commit.after;
        registry.markers.value_mut().clear();
        while let Some(obsolete) = registry.entries.value_mut().pop_front() {
            #[cfg(test)]
            {
                registry.payload_bytes -= (registry.measure)(&obsolete);
            }
            registry.first = registry
                .first
                .checked_add(1)
                .expect("admitted journal positions fit");
            drop(registry);
            drop(obsolete);
            registry = self.lock();
        }
        let obsolete = empty_buffers(&mut registry);
        drop(registry);
        drop(obsolete);
        #[cfg(test)]
        observe(&self.shared, &self.lock());
    }

    #[cfg(test)]
    pub fn retained_entries(&self) -> usize {
        self.lock().entries.value().len()
    }
}

fn lock<U>(shared: &Shared<U>) -> MutexGuard<'_, Registry<U>> {
    shared
        .registry
        .lock()
        .expect("exclusive transaction journal invariant has not panicked")
}

fn release<U>(shared: &Shared<U>, id: u64, epoch: u64) {
    let mut registry = lock(shared);
    if registry.epoch != epoch {
        return;
    }
    let index = registry
        .markers
        .value()
        .iter()
        .position(|entry| entry.id == id)
        .expect("live journal mark remains registered");
    #[cfg(test)]
    let moved_bytes = (registry.markers.value().len() - index - 1) * size_of::<Marker>();
    registry.markers.value_mut().remove(index);
    #[cfg(test)]
    super::sequence::record_copied_bytes(moved_bytes);
    let retain_from = registry.markers.value().first().map(|entry| entry.position);
    let amount = retain_from.map_or(registry.entries.value().len(), |position| {
        position
            .checked_sub(registry.first)
            .expect("mark positions follow the retained journal prefix")
    });
    // Take obsolete inverses one at a time. Their destructors may release
    // custody, but never require a copied bank or a temporary drain allocation.
    for _ in 0..amount {
        let obsolete = registry.entries.value_mut().pop_front();
        #[cfg(test)]
        if let Some(value) = &obsolete {
            registry.payload_bytes -= (registry.measure)(value);
        }
        registry.first = registry
            .first
            .checked_add(1)
            .expect("admitted journal positions fit");
        drop(registry);
        drop(obsolete);
        registry = lock(shared);
    }
    let obsolete = if registry.markers.value().is_empty() {
        Some(empty_buffers(&mut registry))
    } else {
        None
    };
    drop(registry);
    drop(obsolete);
    #[cfg(test)]
    observe(shared, &lock(shared));
}

fn empty_buffers<U>(registry: &mut Registry<U>) -> (Held<VecDeque<U>>, Held<Vec<Marker>>) {
    assert!(
        registry.entries.value().is_empty() && registry.markers.value().is_empty(),
        "no live inverse is discarded"
    );
    let entries_zero = registry
        .entries
        .debit_mut()
        .split(Usage::ZERO)
        .expect("zero debit split");
    let markers_zero = registry
        .markers
        .debit_mut()
        .split(Usage::ZERO)
        .expect("zero debit split");
    (
        std::mem::replace(
            &mut registry.entries,
            Held::new(VecDeque::new(), entries_zero),
        ),
        std::mem::replace(&mut registry.markers, Held::new(Vec::new(), markers_zero)),
    )
}

#[cfg(test)]
fn observe<U>(shared: &Shared<U>, registry: &Registry<U>) {
    if super::super::scale_metrics::ENABLED {
        let live = shared
            ._debit
            .usage()
            .bytes()
            .checked_add(registry.entries.debit().usage().bytes())
            .and_then(|n| n.checked_add(registry.markers.debit().usage().bytes()))
            .and_then(|n| n.checked_add(registry.payload_bytes))
            .expect("admitted journal holder receipt sum");
        super::super::scale_metrics::record(super::super::scale_metrics::Event::SavepointBytes {
            identity: shared as *const Shared<U> as usize,
            live,
        });
    }
}

impl<U> Drop for Mark<U> {
    fn drop(&mut self) {
        if self.live {
            release(&self.shared, self.id, self.epoch);
            self.live = false;
        }
    }
}

fn allocation_bytes<T>(capacity: usize) -> Result<usize, DomError> {
    capacity
        .checked_mul(size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(DomError::RepresentationExhausted)
}

fn reserve_vec<T>(
    values: &mut Held<Vec<T>>,
    additional: usize,
    ledger: &Ledger,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let required = values
        .value()
        .len()
        .checked_add(additional)
        .ok_or(DomError::RepresentationExhausted)?;
    if required <= values.value().capacity() {
        return Ok(());
    }
    let capacity = required.max(values.value().capacity().saturating_mul(2));
    let bytes = allocation_bytes::<T>(capacity)?;
    let mut debit = ledger.retain(Usage::new(bytes, 0), &mut *admit)?;
    let mut replacement = Vec::new();
    replacement
        .try_reserve_exact(capacity)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if super::super::scale_metrics::ENABLED {
        super::super::scale_metrics::record(
            super::super::scale_metrics::Event::StructuralAllocation {
                bytes: replacement.capacity() * size_of::<T>(),
            },
        );
    }
    let actual = allocation_bytes::<T>(replacement.capacity())?;
    if actual > bytes {
        debit.grow(Usage::new(actual - bytes, 0), &mut *admit)?;
    }
    let mut old = values.replace(Held::new(replacement, debit));
    #[cfg(test)]
    let moved_bytes = old.value().len() * size_of::<T>();
    values.value_mut().append(old.value_mut());
    #[cfg(test)]
    super::sequence::record_copied_bytes(moved_bytes);
    drop(old);
    Ok(())
}

fn reserve_deque<T>(
    values: &mut Held<VecDeque<T>>,
    additional: usize,
    ledger: &Ledger,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let required = values
        .value()
        .len()
        .checked_add(additional)
        .ok_or(DomError::RepresentationExhausted)?;
    if required <= values.value().capacity() {
        return Ok(());
    }
    let capacity = required.max(values.value().capacity().saturating_mul(2));
    let bytes = allocation_bytes::<T>(capacity)?;
    let mut debit = ledger.retain(Usage::new(bytes, 0), &mut *admit)?;
    let mut replacement = VecDeque::new();
    replacement
        .try_reserve_exact(capacity)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if super::super::scale_metrics::ENABLED {
        super::super::scale_metrics::record(
            super::super::scale_metrics::Event::StructuralAllocation {
                bytes: replacement.capacity() * size_of::<T>(),
            },
        );
    }
    let actual = allocation_bytes::<T>(replacement.capacity())?;
    if actual > bytes {
        debit.grow(Usage::new(actual - bytes, 0), &mut *admit)?;
    }
    let mut old = values.replace(Held::new(replacement, debit));
    #[cfg(test)]
    let moved_bytes = old.value().len() * size_of::<T>();
    values.value_mut().append(old.value_mut());
    #[cfg(test)]
    super::sequence::record_copied_bytes(moved_bytes);
    drop(old);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::scale_metrics::{self, Counters, Hooks, Measurement};

    fn measured<R>(run: impl FnOnce() -> R) -> (R, Counters) {
        let measurement = Measurement::begin().unwrap();
        let result = run();
        (result, measurement.finish(Hooks::default()).unwrap())
    }

    fn accept(_: Usage) -> Result<(), DomError> {
        Ok(())
    }

    #[test]
    fn buffer_growth_counts_actual_vec_and_ring_capacities_and_live_moves() {
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let input = vec![11_u64, 22, 33];
        let bytes = input.capacity() * size_of::<u64>();
        let mut values = Held::new(input, ledger.retain(Usage::new(bytes, 0), accept).unwrap());
        let (result, counters) = measured(|| reserve_vec(&mut values, 1, &ledger, &mut accept));
        result.unwrap();
        assert_eq!(values.value(), &[11, 22, 33]);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            values.value().capacity() * size_of::<u64>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u64>());
        let (result, counters) = measured(|| reserve_vec(&mut values, 1, &ledger, &mut accept));
        result.unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);

        let mut input = VecDeque::from(vec![11_u64, 22, 33]);
        assert_eq!(input.pop_front(), Some(11));
        input.push_back(44);
        let bytes = input.capacity() * size_of::<u64>();
        let mut values = Held::new(input, ledger.retain(Usage::new(bytes, 0), accept).unwrap());
        let (result, counters) = measured(|| reserve_deque(&mut values, 1, &ledger, &mut accept));
        result.unwrap();
        assert_eq!(
            values.value().iter().copied().collect::<Vec<_>>(),
            [22, 33, 44]
        );
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            values.value().capacity() * size_of::<u64>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u64>());
        let before = ledger.total();
        let pointer = values.value().as_slices().0.as_ptr();
        let additional = values.value().capacity();
        let (result, counters) = measured(|| {
            reserve_deque(&mut values, additional, &ledger, &mut |_| {
                Err(DomError::Cancelled)
            })
        });
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
        assert_eq!(values.value().as_slices().0.as_ptr(), pointer);
        assert_eq!(ledger.total(), before);
    }

    #[test]
    fn journal_shared_body_aliases_and_actual_slot_writes_have_distinct_evidence() {
        if !scale_metrics::ENABLED {
            return;
        }
        let ledger = Ledger::default();
        let (journal, counters) = measured(|| Journal::<u64>::new(&ledger, accept).unwrap());
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(counters.structural_bytes, size_of::<Shared<u64>>());
        assert_eq!(counters.sequence_materialized_bytes, 0);
        let (alias, counters) = measured(|| journal.clone());
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
        let (outer, counters) = measured(|| journal.mark(&ledger, accept).unwrap());
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            journal.lock().markers.value().capacity() * size_of::<Marker>()
        );
        assert_eq!(counters.sequence_materialized_bytes, size_of::<Marker>());
        let (prepared, counters) = measured(|| journal.prepare(&ledger, accept).unwrap());
        assert!(prepared);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            journal.lock().entries.value().capacity() * size_of::<u64>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 0);
        let ((), counters) = measured(|| journal.push_prepared(99));
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u64>());
        let inner = journal.mark(&ledger, accept).unwrap();
        let ((), counters) = measured(|| drop(outer));
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, size_of::<Marker>());
        assert_eq!(journal.retained_entries(), 0);
        drop(inner);
        drop(alias);
    }

    #[test]
    fn simultaneous_journal_metrics_follow_actual_shared_holders_and_final_drop() {
        use super::super::super::scale_metrics::{self, Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let measurement = Measurement::begin().unwrap();
        let ledger = Ledger::default();
        let first: Journal<usize> = Journal::new(&ledger, accept).unwrap();
        let alias = first.clone();
        let outer = first.mark(&ledger, accept).unwrap();
        let second: Journal<usize> = Journal::new(&ledger, accept).unwrap();
        let inner = second.mark(&ledger, accept).unwrap();
        let expected = 2 * size_of::<Shared<usize>>()
            + size_of::<Marker>()
                * (first.lock().markers.value().capacity()
                    + second.lock().markers.value().capacity());
        drop(first);
        drop(outer);
        drop(alias);
        drop(inner);
        drop(second);
        let counters = measurement.finish(Hooks::default()).unwrap();
        assert_eq!(counters.savepoint_peak_bytes, expected);
        assert_eq!(counters.savepoint_live_bytes, 0);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn nested_rollback_restores_each_intermediate_value_and_drop_seals_the_outer_mark() {
        let ledger = Ledger::default();
        let journal = Journal::new(&ledger, accept).unwrap();
        let outer = journal.mark(&ledger, accept).unwrap();
        let mut value = 10;
        assert!(journal.prepare(&ledger, accept).unwrap());
        journal.push_prepared(value);
        value = 20;
        let inner = journal.mark(&ledger, accept).unwrap();
        journal.prepare(&ledger, accept).unwrap();
        journal.push_prepared(value);
        value = 30;
        journal.rollback(inner, |before| value = before);
        assert_eq!(value, 20);
        assert_eq!(journal.retained_entries(), 1);
        drop(outer);
        assert_eq!(journal.retained_entries(), 0);
        assert!(!journal.has_marks());
        drop(journal);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn dropping_inner_mark_keeps_its_values_for_the_outer_rollback() {
        let ledger = Ledger::default();
        let journal = Journal::new(&ledger, accept).unwrap();
        let outer = journal.mark(&ledger, accept).unwrap();
        journal.prepare(&ledger, accept).unwrap();
        journal.push_prepared(10);
        let inner = journal.mark(&ledger, accept).unwrap();
        journal.prepare(&ledger, accept).unwrap();
        journal.push_prepared(20);
        drop(inner);
        assert_eq!(journal.retained_entries(), 2);
        let mut restored = Vec::new();
        journal.rollback(outer, |value| restored.push(value));
        assert_eq!(restored, [20, 10]);
        assert_eq!(journal.retained_entries(), 0);
    }

    #[test]
    fn refused_inverse_capacity_preserves_the_mark_and_existing_values() {
        let ledger = Ledger::default();
        let journal: Journal<usize> = Journal::new(&ledger, accept).unwrap();
        let mark = journal.mark(&ledger, accept).unwrap();
        let before = ledger.total();
        assert!(
            journal
                .prepare(&ledger, |_| Err(DomError::Cancelled))
                .is_err()
        );
        assert_eq!(ledger.total(), before);
        assert_eq!(journal.retained_entries(), 0);
        assert!(journal.has_marks());
        journal.rollback(mark, |_| panic!("refused inverse was never retained"));
        assert!(!journal.has_marks());
    }

    #[test]
    fn last_mark_drop_releases_inverse_payload_before_any_later_transaction_work() {
        let ledger = Ledger::default();
        let journal = Journal::new(&ledger, accept).unwrap();
        let permanent = ledger.total();
        let mark = journal.mark(&ledger, accept).unwrap();
        journal.prepare(&ledger, accept).unwrap();
        let administration = ledger.total();
        let held = Held::new(7, ledger.retain(Usage::new(19, 5), accept).unwrap());
        journal.push_prepared(held);
        assert_eq!(
            ledger.total(),
            administration.checked_add(Usage::new(19, 5)).unwrap()
        );
        drop(mark);
        assert_eq!(ledger.total(), permanent);
    }
}
