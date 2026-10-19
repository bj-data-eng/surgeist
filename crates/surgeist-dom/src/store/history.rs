//! Exclusive-owner revision history. Fixed chunks retain their actual backing;
//! preparing a publication examines only entries that it will evict.
use super::*;
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, Weak};

const CHUNK_ENTRIES: usize = 32;
type Link = Arc<Mutex<Chunk>>;

pub(super) struct HistoryEntry {
    from: DomRevision,
    changes: Arc<[DomChange]>,
    bytes: usize,
    // The inline header is owned by its chunk backing. This token owns the
    // separately retained changes field occurrence.
    _changes: custody::Debit,
}

struct Chunk {
    values: VecDeque<HistoryEntry>,
    older: Option<Link>,
    newer: Weak<Mutex<Chunk>>,
    _allocation: custody::Debit,
}

pub(super) struct History {
    newest: Option<Link>,
    oldest: Option<Link>,
    entries: usize,
    bytes: usize,
}

pub(super) struct Plan(Decision);

enum Decision {
    Keep,
    Clear,
    Insert {
        entry: HistoryEntry,
        chunk: Option<Link>,
        evictions: usize,
        entries: usize,
        bytes: usize,
    },
}

fn lock(chunk: &Link) -> MutexGuard<'_, Chunk> {
    chunk
        .lock()
        .expect("exclusive history owner has not panicked")
}

impl History {
    pub(super) fn new() -> Self {
        Self {
            newest: None,
            oldest: None,
            entries: 0,
            bytes: 0,
        }
    }
    pub(super) fn entries(&self) -> usize {
        self.entries
    }
    pub(super) fn bytes(&self) -> usize {
        self.bytes
    }

    pub(super) fn prepare(
        &self,
        dom: &Dom,
        changes: &Arc<[DomChange]>,
        changed: bool,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Plan, DomError> {
        let limits = dom.limits;
        let ledger = dom.custody_registry.ledger();
        if !changed {
            return Ok(Plan(Decision::Keep));
        }
        let payload = changes
            .len()
            .checked_mul(size_of::<DomChange>())
            .ok_or(DomError::RepresentationExhausted)?;
        let entry_bytes = size_of::<HistoryEntry>()
            .checked_add(payload)
            .ok_or(DomError::RepresentationExhausted)?;
        if limits.history_entries == 0 || entry_bytes > limits.history_bytes {
            // Each reached entry is a real impending eviction, not a survivor
            // recount. Nothing has changed if cancellation/work refuses.
            self.eviction_count(self.entries, self.bytes, &mut step)?;
            return Ok(Plan(Decision::Clear));
        }
        let entries = self
            .entries
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let bytes = self
            .bytes
            .checked_add(entry_bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let (evictions, evicted_bytes) = self.eviction_count(
            entries.saturating_sub(limits.history_entries),
            bytes.saturating_sub(limits.history_bytes),
            &mut step,
        )?;
        let entries = entries - evictions;
        let bytes = bytes - evicted_bytes;
        let debit = ledger.retain(custody::Usage::new(payload, 0), &mut admit)?;
        let entry = HistoryEntry {
            from: dom.revision,
            changes: changes.clone(),
            bytes: entry_bytes,
            _changes: debit,
        };
        // A pending entry and its token remain together across chunk admission.
        let mut plan = Plan(Decision::Insert {
            entry,
            chunk: None,
            evictions,
            entries,
            bytes,
        });
        let needs_chunk = match &self.newest {
            None => true,
            Some(newest) => {
                let len = lock(newest).values.len();
                let removed_here = evictions.saturating_sub(self.entries - len);
                len - removed_here == CHUNK_ENTRIES
            }
        };
        if needs_chunk {
            let requested = CHUNK_ENTRIES
                .checked_mul(size_of::<HistoryEntry>())
                .and_then(|bytes| bytes.checked_add(size_of::<Mutex<Chunk>>()))
                .ok_or(DomError::RepresentationExhausted)?;
            let mut debit = ledger.retain(custody::Usage::new(requested, 0), &mut admit)?;
            let mut values = VecDeque::new();
            values
                .try_reserve_exact(CHUNK_ENTRIES)
                .map_err(|_| DomError::AllocationFailed)?;
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                    bytes: values.capacity() * size_of::<HistoryEntry>(),
                });
            }
            let actual = values
                .capacity()
                .checked_mul(size_of::<HistoryEntry>())
                .and_then(|bytes| bytes.checked_add(size_of::<Mutex<Chunk>>()))
                .ok_or(DomError::RepresentationExhausted)?;
            if actual > requested {
                debit.grow(custody::Usage::new(actual - requested, 0), &mut admit)?;
            }
            let chunk = Arc::new(Mutex::new(Chunk {
                values,
                older: None,
                newer: Weak::new(),
                _allocation: debit,
            }));
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                    bytes: size_of::<Mutex<Chunk>>(),
                });
            }
            let Decision::Insert { chunk: slot, .. } = &mut plan.0 else {
                unreachable!("prepared insertion")
            };
            *slot = Some(chunk);
        }
        Ok(plan)
    }

    // Return the count and exact quota bytes of the impending oldest suffix.
    fn eviction_count(
        &self,
        mut excess_entries: usize,
        mut excess_bytes: usize,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(usize, usize), DomError> {
        let mut count = 0;
        let mut bytes = 0;
        let mut cursor = self.oldest.clone();
        while excess_entries != 0 || excess_bytes != 0 {
            let link = cursor.take().expect("cached history extents");
            let chunk = lock(&link);
            for entry in &chunk.values {
                if excess_entries == 0 && excess_bytes == 0 {
                    break;
                }
                step()?;
                #[cfg(test)]
                if scale_metrics::ENABLED {
                    scale_metrics::record(scale_metrics::Event::HistoryEvictionVisited);
                }
                count += 1;
                bytes += entry.bytes;
                excess_entries = excess_entries.saturating_sub(1);
                excess_bytes = excess_bytes.saturating_sub(entry.bytes);
            }
            cursor = chunk.newer.upgrade();
        }
        Ok((count, bytes))
    }

    pub(super) fn commit(&mut self, plan: Plan) {
        match plan.0 {
            Decision::Keep => {}
            Decision::Clear => self.clear(),
            Decision::Insert {
                entry,
                chunk,
                evictions,
                entries,
                bytes,
            } => {
                for _ in 0..evictions {
                    self.pop_oldest(true);
                }
                if let Some(chunk) = chunk {
                    if let Some(previous) = &self.newest {
                        lock(previous).newer = Arc::downgrade(&chunk);
                        lock(&chunk).older = Some(previous.clone());
                    } else {
                        self.oldest = Some(chunk.clone());
                    }
                    self.newest = Some(chunk);
                }
                let mut newest = lock(self.newest.as_ref().expect("prepared history chunk"));
                assert!(newest.values.len() < CHUNK_ENTRIES);
                newest.values.push_back(entry);
                #[cfg(test)]
                if scale_metrics::ENABLED {
                    scale_metrics::record(scale_metrics::Event::SequenceMaterialized {
                        bytes: size_of::<HistoryEntry>(),
                    });
                }
                drop(newest);
                self.entries = entries;
                self.bytes = bytes;
            }
        }
    }

    fn pop_oldest(&mut self, keep_empty: bool) {
        let oldest = self.oldest.as_ref().expect("retained oldest").clone();
        let mut chunk = lock(&oldest);
        let entry = chunk.values.pop_front().expect("retained history entry");
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::HistoryEntryEvicted);
        }
        self.entries -= 1;
        self.bytes -= entry.bytes;
        drop(entry);
        if chunk.values.is_empty() {
            if let Some(newer) = chunk.newer.upgrade() {
                // Release the older strong edge while the explicit oldest
                // holder keeps its allocation and token together.
                lock(&newer).older = None;
                self.oldest = Some(newer);
            } else if !keep_empty {
                self.oldest = None;
                self.newest = None;
            }
        }
        drop(chunk);
        drop(oldest);
    }
    pub(super) fn trim(&mut self, limits: DomLimits) {
        while self.entries > limits.history_entries || self.bytes > limits.history_bytes {
            self.pop_oldest(false);
        }
    }
    fn clear(&mut self) {
        while self.entries != 0 {
            self.pop_oldest(false);
        }
        self.oldest = None;
        self.newest = None;
    }
    pub(super) fn changes_since(
        &self,
        baseline: DomRevision,
        current: DomRevision,
    ) -> Result<Arc<[DomChange]>, DomHistoryError> {
        let available_from = self
            .oldest
            .as_ref()
            .and_then(|oldest| {
                lock(oldest).values.front().map(|entry| {
                    #[cfg(test)]
                    if scale_metrics::ENABLED {
                        scale_metrics::record(scale_metrics::Event::HistoryQueryEntryVisited);
                    }
                    entry.from
                })
            })
            .unwrap_or(current);
        if baseline < available_from {
            return Err(DomHistoryError::FullRecomputeRequired { available_from });
        }
        let mut selected = Vec::new();
        let mut cursor = self.newest.clone();
        'history: while let Some(link) = cursor {
            let chunk = lock(&link);
            for entry in chunk.values.iter().rev() {
                #[cfg(test)]
                if scale_metrics::ENABLED {
                    scale_metrics::record(scale_metrics::Event::HistoryQueryEntryVisited);
                }
                if entry.from < baseline {
                    break 'history;
                }
                selected
                    .try_reserve(1)
                    .map_err(|_| DomHistoryError::AllocationFailed)?;
                selected.push(entry.changes.clone());
            }
            cursor = chunk.older.clone();
        }
        let mut changes = Vec::new();
        for entry in selected.into_iter().rev() {
            changes
                .try_reserve(entry.len())
                .map_err(|_| DomHistoryError::AllocationFailed)?;
            changes.extend_from_slice(&entry);
            #[cfg(test)]
            if scale_metrics::ENABLED && !entry.is_empty() {
                scale_metrics::record(scale_metrics::Event::HistoryQueryChangesCopied {
                    records: entry.len(),
                });
            }
        }
        Ok(changes.into())
    }

    #[cfg(test)]
    pub(super) fn inspect_entries(&self, mut visit: impl FnMut(DomRevision, &[DomChange], usize)) {
        let mut cursor = self.newest.clone();
        while let Some(link) = cursor {
            let chunk = lock(&link);
            for entry in chunk.values.iter().rev() {
                visit(entry.from, &entry.changes, entry.bytes);
            }
            cursor = chunk.older.clone();
        }
    }
    #[cfg(test)]
    pub(super) fn inspect_storage(&self) -> Result<(custody::Usage, custody::Usage), DomError> {
        let mut actual = custody::Usage::default();
        let mut claimed = custody::Usage::default();
        let mut cursor = self.newest.clone();
        while let Some(link) = cursor {
            let chunk = lock(&link);
            let backing = chunk
                .values
                .capacity()
                .checked_mul(size_of::<HistoryEntry>())
                .and_then(|bytes| bytes.checked_add(size_of::<Mutex<Chunk>>()))
                .ok_or(DomError::RepresentationExhausted)?;
            actual = actual.checked_add(custody::Usage::new(backing, 0))?;
            claimed = claimed.checked_add(chunk._allocation.usage())?;
            for entry in &chunk.values {
                let payload = entry
                    .changes
                    .len()
                    .checked_mul(size_of::<DomChange>())
                    .ok_or(DomError::RepresentationExhausted)?;
                actual = actual.checked_add(custody::Usage::new(payload, 0))?;
                claimed = claimed.checked_add(entry._changes.usage())?;
            }
            cursor = chunk.older.clone();
        }
        Ok((actual, claimed))
    }
}
impl Drop for History {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::scale_metrics::{Counters, Hooks, Measurement};
    use std::cell::Cell;

    fn fixture(retained: usize, maximum: usize) -> (Dom, NodeId) {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                16_384,
                8,
                0,
                0,
                0,
                0,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let comment = batch.create_comment(&document, "a");
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let comment = commit.created(&comment).unwrap().unwrap();
        dom.set_limits(
            DomLimits::new(
                8,
                16_384,
                8,
                0,
                0,
                maximum,
                64_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        for index in 0..retained {
            let mut batch = DomBatch::new();
            batch.replace_data(comment, 0, 1, if index % 2 == 0 { "b" } else { "a" });
            dom.apply(batch, &AtomicBool::new(false)).unwrap();
        }
        assert_eq!(dom.usage().history_entries(), retained);
        (dom, comment)
    }

    fn measured<T>(body: impl FnOnce() -> T) -> (T, Option<Counters>) {
        let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
        let result = body();
        (
            result,
            measurement.map(|value| value.finish(Hooks::default()).unwrap()),
        )
    }

    #[test]
    fn ordinary_preparation_visits_only_actual_evictions_and_never_moves_survivors() {
        for retained in [64, 256] {
            for full in [false, true] {
                let (mut dom, node) = fixture(retained, retained + usize::from(!full));
                let changes = Arc::from([DomChange::Data(node)]);
                let visits = Cell::new(0);
                let (plan, counters) = measured(|| {
                    dom.history
                        .prepare(
                            &dom,
                            &changes,
                            true,
                            |_| Ok(()),
                            || {
                                visits.set(visits.get() + 1);
                                Ok(())
                            },
                        )
                        .unwrap()
                });
                assert_eq!(visits.get(), usize::from(full));
                if let Some(counters) = counters {
                    assert_eq!(counters.history_eviction_visits, usize::from(full));
                    assert_eq!(counters.history_entries_evicted, 0);
                    assert_eq!(counters.history_query_entry_visits, 0);
                    assert_eq!(counters.structural_allocations, 2);
                    assert_eq!(
                        counters.structural_bytes,
                        CHUNK_ENTRIES * size_of::<HistoryEntry>() + size_of::<Mutex<Chunk>>()
                    );
                    assert_eq!(counters.sequence_materialized_bytes, 0);
                }
                let ((), counters) = measured(|| dom.history.commit(plan));
                if let Some(counters) = counters {
                    assert_eq!(counters.history_eviction_visits, 0);
                    assert_eq!(counters.history_entries_evicted, usize::from(full));
                    assert_eq!(counters.structural_allocations, 0);
                    assert_eq!(
                        counters.sequence_materialized_bytes,
                        size_of::<HistoryEntry>()
                    );
                }
                assert_eq!(dom.history.entries(), retained + usize::from(!full));
            }
        }
    }

    fn source_data(dom: &mut Dom, node: NodeId, value: &str) -> DomChanges {
        let cancel = AtomicBool::new(false);
        let policy = crate::DomSourceLimits::new(256_000_000, 1_000_000, 20_000, 8, 128, 128, 1);
        let mut lease = dom
            .begin_source_edit(
                crate::DomSourceEdit::replace_data(node, 0, 1, value),
                policy,
                &cancel,
            )
            .unwrap();
        match lease.advance(&cancel).unwrap() {
            crate::DomSourceProgress::Complete(done) => done.commit().changes().clone(),
            _ => panic!("character-data edit requires no external service"),
        }
    }

    #[test]
    fn ordinary_and_source_history_length_ladder_discovers_zero_or_one_eviction() {
        for retained in [64, 256, 32_768] {
            let (mut dom, node) = fixture(retained, retained + 2);
            let witness = dom.snapshot();
            for (source, full, value) in [
                (false, false, "b"),
                (true, false, "c"),
                (false, true, "d"),
                (true, true, "e"),
            ] {
                let before = dom.snapshot().stamp();
                let (delta, counters) = measured(|| {
                    if source {
                        source_data(&mut dom, node, value)
                    } else {
                        let mut batch = DomBatch::new();
                        batch.replace_data(node, 0, 1, value);
                        dom.apply(batch, &AtomicBool::new(false))
                            .unwrap()
                            .changes()
                            .clone()
                    }
                });
                assert_eq!(delta.from(), before);
                assert_eq!(delta.to(), dom.snapshot().stamp());
                assert_eq!(delta.changes(), &[DomChange::Data(node)]);
                assert_eq!(
                    witness.node(node).unwrap().data(),
                    Some(&DocumentString::from("a"))
                );
                assert_eq!(
                    dom.snapshot().node(node).unwrap().data(),
                    Some(&DocumentString::from(value))
                );
                if let Some(counters) = counters {
                    assert_eq!(
                        counters.history_eviction_visits,
                        usize::from(full),
                        "{counters:?}"
                    );
                    assert_eq!(
                        counters.history_entries_evicted,
                        usize::from(full),
                        "{counters:?}"
                    );
                    assert_eq!(counters.history_query_entry_visits, 0, "{counters:?}");
                    assert_eq!(counters.history_query_changes_copied, 0, "{counters:?}");
                    assert_eq!(counters.published_roots, 1);
                    if source {
                        assert!(counters.source_steps > 0 && counters.source_steps <= 20_000);
                    }
                    eprintln!(
                        "history length={retained} source={source} full={full} counters={counters:?}"
                    );
                }
                let (report, counters) = measured(|| dom.changes_since(before).unwrap());
                assert_eq!(report.changes(), &[DomChange::Data(node)]);
                if let Some(counters) = counters {
                    // Oldest coverage, selected newest entry, one boundary.
                    assert_eq!(counters.history_query_entry_visits, 3);
                    assert_eq!(counters.history_query_changes_copied, 1);
                    assert_eq!(counters.history_eviction_visits, 0);
                    assert_eq!(counters.history_entries_evicted, 0);
                }
            }
            let before = dom.snapshot().stamp();
            let usage = dom.usage();
            let (noop, counters) = measured(|| source_data(&mut dom, node, "e"));
            assert!(noop.changes().is_empty());
            assert_eq!(dom.snapshot().stamp(), before);
            assert_eq!(dom.usage(), usage);
            if let Some(counters) = counters {
                assert_eq!(counters.history_eviction_visits, 0);
                assert_eq!(counters.history_entries_evicted, 0);
                assert_eq!(counters.history_query_entry_visits, 0);
                assert_eq!(counters.published_roots, 0);
            }
        }
    }

    #[test]
    fn byte_eviction_plan_and_cancelled_prefix_preserve_the_original_window() {
        let (mut dom, node) = fixture(8, 64);
        let one = size_of::<HistoryEntry>() + size_of::<DomChange>();
        dom.set_limits(
            DomLimits::new(
                8,
                16_384,
                8,
                0,
                0,
                64,
                8 * one,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let changes: Arc<[DomChange]> = vec![DomChange::Data(node); 5].into();
        let incoming = size_of::<HistoryEntry>() + 5 * size_of::<DomChange>();
        let removed = incoming.div_ceil(one);
        assert!(removed > 1);
        let baseline = dom.custody_registry.ledger().total();
        let original_oldest = dom.history.oldest.as_ref().unwrap().clone();
        let original_newest = dom.history.newest.as_ref().unwrap().clone();
        let visits = Cell::new(0);
        let (refused, counters) = measured(|| {
            dom.history.prepare(
                &dom,
                &changes,
                true,
                |_| Ok(()),
                || {
                    visits.set(visits.get() + 1);
                    if visits.get() == 2 {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
            )
        });
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(visits.get(), 2);
        if let Some(counters) = counters {
            assert_eq!(counters.history_eviction_visits, 1);
            assert_eq!(counters.history_entries_evicted, 0);
            assert_eq!(counters.structural_allocations, 0);
        }
        assert_eq!(dom.history.entries(), 8);
        assert_eq!(dom.history.bytes(), 8 * one);
        assert!(Arc::ptr_eq(
            &original_oldest,
            dom.history.oldest.as_ref().unwrap()
        ));
        assert!(Arc::ptr_eq(
            &original_newest,
            dom.history.newest.as_ref().unwrap()
        ));
        assert_eq!(dom.custody_registry.ledger().total(), baseline);
        visits.set(0);
        let plan = dom
            .history
            .prepare(
                &dom,
                &changes,
                true,
                |_| Ok(()),
                || {
                    visits.set(visits.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(visits.get(), removed);
        dom.history.commit(plan);
        assert_eq!(dom.history.entries(), 9 - removed);
        assert_eq!(dom.history.bytes(), (8 - removed) * one + incoming);
    }

    #[test]
    fn pending_chunk_exact_peak_and_one_short_leave_canonical_history_unchanged() {
        let (dom, node) = fixture(CHUNK_ENTRIES, CHUNK_ENTRIES + 1);
        let changes = Arc::from([DomChange::Data(node)]);
        let ledger = dom.custody_registry.ledger();
        let baseline = ledger.total();
        let extra = size_of::<DomChange>()
            + CHUNK_ENTRIES * size_of::<HistoryEntry>()
            + size_of::<Mutex<Chunk>>();
        for short in [true, false] {
            let limit = baseline.bytes() + extra - usize::from(short);
            let result = dom.history.prepare(
                &dom,
                &changes,
                true,
                |usage| admit(usage.bytes(), limit, DomResource::SourceRetainedBytes),
                || Ok(()),
            );
            if short {
                assert!(matches!(
                    result,
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        ..
                    })
                ));
            } else {
                let plan = result.unwrap();
                assert_eq!(ledger.total().bytes(), baseline.bytes() + extra);
                assert_eq!(dom.history.entries(), CHUNK_ENTRIES);
                drop(plan);
            }
            assert_eq!(ledger.total(), baseline);
            assert_eq!(dom.history.entries(), CHUNK_ENTRIES);
        }
    }
}
