//! Dense exclusive arenas and incremental linear hashing. One insertion splits
//! at most one bucket. Fingerprints only route; the caller compares exact text.
use super::super::bank::owned::{EmptyPath, OwnedBank};
use super::*;

pub(super) const CLOSING_STEPS: usize = 3;

pub(super) struct Pending {
    pub origin: Weak<Scope>,
    pub pins: usize,
    pub bytes: usize,
}
pub(super) struct Row {
    pub fingerprint: u64,
    pub len: usize,
    pub record: Weak<NameRecord>,
    pub key: custody::MaterializationId,
    pub next: Option<usize>,
    pub previous: Option<usize>,
    pub bucket: usize,
    pub pending: Option<Pending>,
    pub own: custody::Usage,
    pub _custody: bank::ObjectCustody,
}
enum Slot {
    Occupied(Row),
    Free { next: Option<usize> },
}
#[derive(Default)]
struct Bucket {
    head: Option<usize>,
    len: usize,
}
#[derive(Default)]
pub(super) struct Index {
    rows: OwnedBank<Slot>,
    buckets: OwnedBank<Bucket>,
    free: Option<usize>,
    entries: usize,
    level: u32,
    split: usize,
    count: usize,
    cursor: usize,
}

/// Empty Bank paths and the split's actual selected-row vector precede the
/// affine token. No resident row is moved until every fallible step succeeds.
pub(super) struct Prepared {
    row_path: Option<EmptyPath<Slot>>,
    bucket_path: Option<EmptyPath<Bucket>>,
    split_rows: Vec<usize>,
    slot: usize,
    bucket: Option<usize>,
    splitting: Option<(usize, u32, usize)>,
    own: custody::Usage,
    storage: custody::Debit,
    local: Attribution,
}
impl Prepared {
    pub(super) fn slot(&self) -> usize {
        self.slot
    }
    pub fn new(
        owner: &Registry,
        scope: &Arc<Scope>,
        own: custody::Usage,
        admission: &mut Admission<'_>,
    ) -> Result<Self> {
        let ledger = owner.lock().registry.ledger().clone();
        let local = Attribution::new(owner, scope, own.bytes())?;
        let storage = ledger.retain(custody::Usage::ZERO, &mut *admission.global)?;
        Ok(Self {
            row_path: None,
            bucket_path: None,
            split_rows: Vec::new(),
            slot: 0,
            bucket: None,
            splitting: None,
            own,
            storage,
            local,
        })
    }
    fn retain(&mut self, bytes: usize, admission: &mut Admission<'_>) -> Result<()> {
        // Reserve the exact local extent before the global callback or actual
        // allocation. On refusal, Prepared drops data before both receipts.
        self.local.grow(bytes)?;
        self.storage
            .grow(custody::Usage::new(bytes, 0), &mut *admission.global)?;
        Ok(())
    }
}
impl Index {
    pub fn entries(&self) -> usize {
        self.entries
    }
    fn bucket_for(&self, fingerprint: u64) -> usize {
        if self.count == 0 {
            return 0;
        }
        let base = 1usize << self.level;
        let low = fingerprint as usize & (base - 1);
        if low < self.split {
            fingerprint as usize & ((base << 1) - 1)
        } else {
            low
        }
    }
    pub fn first(&self, fingerprint: u64) -> Option<usize> {
        self.buckets
            .get(self.bucket_for(fingerprint))
            .and_then(|bucket| bucket.head)
    }
    pub fn row(&self, index: usize) -> &Row {
        match &self.rows[index] {
            Slot::Occupied(row) => row,
            Slot::Free { .. } => unreachable!("occupied name link"),
        }
    }
    pub fn row_mut(&mut self, index: usize) -> &mut Row {
        match &mut self.rows[index] {
            Slot::Occupied(row) => row,
            Slot::Free { .. } => unreachable!("occupied name link"),
        }
    }
    pub fn find_row(&self, index: usize, key: custody::MaterializationId) -> Option<&Row> {
        match self.rows.get(index)? {
            Slot::Occupied(row) if row.key == key => Some(row),
            _ => None,
        }
    }
    pub fn find_row_mut(
        &mut self,
        index: usize,
        key: custody::MaterializationId,
    ) -> Option<&mut Row> {
        match self.rows.get_mut(index)? {
            Slot::Occupied(row) if row.key == key => Some(row),
            _ => None,
        }
    }
    pub fn prepare(
        &self,
        prepared: &mut Prepared,
        scope: &Scope,
        admission: &mut Admission<'_>,
    ) -> Result<()> {
        scope.step(admission)?;
        prepared.slot = self.free.unwrap_or(self.rows.len());
        if self.free.is_none() {
            prepared.row_path = Some(self.rows.reserve(prepared.slot, |charge| {
                scope.step(admission)?;
                prepared.retain(charge.bytes, admission)?;
                Ok::<_, DomError>(None)
            })?);
        }
        if self.count == 0 || self.entries >= self.count {
            self.count
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            prepared.bucket = Some(self.count);
            prepared.bucket_path = Some(self.buckets.reserve(self.count, |charge| {
                scope.step(admission)?;
                prepared.retain(charge.bytes, admission)?;
                Ok::<_, DomError>(None)
            })?);
            if self.count != 0 {
                let base = 1usize
                    .checked_shl(self.level)
                    .ok_or(DomError::RepresentationExhausted)?;
                let next_split = self
                    .split
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                let (level, split) = if next_split == base {
                    let level = self
                        .level
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    1usize
                        .checked_shl(
                            level
                                .checked_add(1)
                                .ok_or(DomError::RepresentationExhausted)?,
                        )
                        .ok_or(DomError::RepresentationExhausted)?;
                    (level, 0)
                } else {
                    (self.level, next_split)
                };
                prepared.splitting = Some((self.split, level, split));
                let len = self.buckets[self.split].len;
                let requested = len
                    .checked_mul(size_of::<usize>())
                    .ok_or(DomError::RepresentationExhausted)?;
                prepared.retain(requested, admission)?;
                prepared
                    .split_rows
                    .try_reserve_exact(len)
                    .map_err(|_| DomError::AllocationFailed)?;
                let actual = prepared
                    .split_rows
                    .capacity()
                    .checked_mul(size_of::<usize>())
                    .ok_or(DomError::RepresentationExhausted)?;
                if actual != 0 {
                    #[cfg(test)]
                    metric(super::super::scale_metrics::Event::StructuralAllocation {
                        bytes: actual,
                    });
                }
                prepared.retain(
                    actual
                        .checked_sub(requested)
                        .expect("reserve_exact covers requested"),
                    admission,
                )?;
                #[cfg(test)]
                let mut observed = super::super::CopyObservation::default();
                let mut row = self.buckets[self.split].head;
                while let Some(index) = row {
                    scope.step(admission)?;
                    prepared.split_rows.push(index);
                    #[cfg(test)]
                    observed.pushed::<usize>();
                    row = self.row(index).next;
                }
                assert_eq!(prepared.split_rows.len(), len, "exact bucket cardinality");
            }
        }
        Ok(())
    }
    pub fn install(
        &mut self,
        mut prepared: Prepared,
        row: Row,
        storage: &mut custody::Debit,
    ) -> (usize, usize) {
        let mut moved = 0;
        if let Some(plan) = prepared.bucket_path.take() {
            moved += self.buckets.insert(plan, Bucket::default(), false).bytes;
            self.count += 1;
        }
        if let Some((old, level, split)) = prepared.splitting {
            self.level = level;
            self.split = split;
            self.buckets[old] = Bucket::default();
            for index in prepared.split_rows.iter().copied() {
                let bucket = self.bucket_for(self.row(index).fingerprint);
                self.link(index, bucket);
            }
        }
        let index = prepared.slot;
        let bucket = self.bucket_for(row.fingerprint);
        if let Some(plan) = prepared.row_path.take() {
            moved += self.rows.insert(plan, Slot::Occupied(row), false).bytes;
        } else {
            let Slot::Free { next } = self.rows[index] else {
                unreachable!("prepared free slot")
            };
            self.free = next;
            self.rows[index] = Slot::Occupied(row);
        }
        self.link(index, bucket);
        self.entries += 1;
        // All unused spares and the selected split vector die before credit is
        // transferred/shrunk. Only actual moved components enter this owner.
        drop(std::mem::take(&mut prepared.split_rows));
        let retained = prepared
            .storage
            .split(custody::Usage::new(moved, 0))
            .expect("actual moved subset");
        storage.absorb(retained).expect("same DOM ledger");
        // The row now owns the pending scratch obligation. The same global
        // debit moves to the exclusive index; only its local classification
        // changes, with no callback or fallible admission after installation.
        prepared.local.transfer_pending(prepared.own, moved);
        (index, moved)
    }
    fn link(&mut self, index: usize, bucket: usize) {
        let head = self.buckets[bucket].head;
        if let Some(head) = head {
            self.row_mut(head).previous = Some(index);
        }
        let row = self.row_mut(index);
        row.bucket = bucket;
        row.previous = None;
        row.next = head;
        self.buckets[bucket].head = Some(index);
        self.buckets[bucket].len += 1;
    }
    pub fn remove(&mut self, index: usize) -> Row {
        let row = std::mem::replace(&mut self.rows[index], Slot::Free { next: self.free });
        let Slot::Occupied(row) = row else {
            unreachable!("occupied removal")
        };
        if let Some(previous) = row.previous {
            self.row_mut(previous).next = row.next;
        } else {
            self.buckets[row.bucket].head = row.next;
        }
        if let Some(next) = row.next {
            self.row_mut(next).previous = row.previous;
        }
        self.buckets[row.bucket].len -= 1;
        self.free = Some(index);
        self.entries -= 1;
        row
    }
    pub fn cleanup_one(&mut self) -> Option<Row> {
        if self.rows.is_empty() {
            return None;
        }
        let index = self.cursor;
        self.cursor = if index + 1 == self.rows.len() {
            0
        } else {
            index + 1
        };
        match &self.rows[index] {
            Slot::Occupied(row) if row.pending.is_none() && row.record.upgrade().is_none() => {
                Some(self.remove(index))
            }
            _ => None,
        }
    }
    #[cfg(test)]
    pub fn inspect_storage_bytes(&self) -> usize {
        self.rows.inspect_storage_bytes() + self.buckets.inspect_storage_bytes()
    }
    #[cfg(test)]
    pub fn inspect_first_reservation_bytes() -> usize {
        OwnedBank::<Slot>::inspect_first_reservation_bytes()
            + OwnedBank::<Bucket>::inspect_first_reservation_bytes()
    }
    #[cfg(test)]
    pub fn inspect_row_extent(&self) -> usize {
        self.rows.len()
    }
    #[cfg(test)]
    pub fn inspect_rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.values().filter_map(|slot| match slot {
            Slot::Occupied(row) => Some(row),
            _ => None,
        })
    }
}

#[cfg(test)]
fn metric(event: super::super::scale_metrics::Event) {
    if super::super::scale_metrics::ENABLED {
        super::super::scale_metrics::record(event);
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::scale_metrics::{Hooks, Measurement};
    use super::*;

    #[test]
    fn splitting_an_empty_selected_bucket_allocates_no_split_buffer() {
        let ledger = custody::Ledger::default();
        let control = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        let mut admission = Admission {
            global: &mut |_| Ok(()),
            step: &mut || Ok(()),
        };
        let registry = Registry::new(
            DomOwnerId::allocate().unwrap(),
            &control,
            DomNameLimits::new(1_000_000, 16, 10_000, 1_000_000).unwrap(),
            &mut admission,
        )
        .unwrap();
        let operation = registry.begin(None, &mut admission).unwrap();
        let first = operation
            .reserve_hashed("first".into(), 1, &mut admission)
            .unwrap()
            .publish();

        // Positive control: the first split selects one actual row. Prepare
        // only the index; no NameRecord or generic registration is constructed
        // in this interval. The original leaf has room for both new slots.
        {
            let lease = IndexLease::take(&registry).unwrap();
            let measured =
                super::super::super::scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
            let mut prepared = Prepared::new(
                &registry,
                &operation.scope,
                custody::Usage::ZERO,
                &mut admission,
            )
            .unwrap();
            lease
                .index
                .prepare(&mut prepared, &operation.scope, &mut admission)
                .unwrap();
            assert_eq!(prepared.splitting.unwrap().0, 0);
            assert_eq!(prepared.split_rows, [0]);
            if let Some(measured) = measured {
                let counts = measured.finish(Hooks::default()).unwrap();
                assert_eq!(counts.structural_allocations, 1);
                assert_eq!(
                    counts.structural_bytes,
                    prepared.split_rows.capacity() * size_of::<usize>()
                );
                assert_eq!(counts.sequence_materialized_bytes, size_of::<usize>());
                assert_eq!(counts.record_materializations, 2);
            }
        }

        // Both odd fingerprints move to bucket 1. The next split selects the
        // now-empty bucket 0, even though the registry has live participants.
        let second = operation
            .reserve_hashed("second".into(), 3, &mut admission)
            .unwrap()
            .publish();
        {
            let lease = IndexLease::take(&registry).unwrap();
            assert_eq!(lease.index.count, 2);
            assert_eq!(lease.index.buckets[0].len, 0);
            assert_eq!(lease.index.buckets[1].len, 2);
            let measured =
                super::super::super::scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
            let mut prepared = Prepared::new(
                &registry,
                &operation.scope,
                custody::Usage::ZERO,
                &mut admission,
            )
            .unwrap();
            lease
                .index
                .prepare(&mut prepared, &operation.scope, &mut admission)
                .unwrap();
            assert_eq!(prepared.splitting.unwrap().0, 0);
            assert_eq!(prepared.split_rows.len(), 0);
            assert_eq!(prepared.split_rows.capacity(), 0);
            if let Some(measured) = measured {
                let counts = measured.finish(Hooks::default()).unwrap();
                assert_eq!(counts.structural_allocations, 0);
                assert_eq!(counts.structural_bytes, 0);
                assert_eq!(counts.sequence_materializations, 0);
                assert_eq!(
                    counts.record_materializations, 2,
                    "actual spare row and bucket targets were reached"
                );
            }
        }
        assert_eq!(
            operation
                .reserve_hashed("first".into(), 1, &mut admission)
                .unwrap()
                .publish(),
            first
        );
        assert_eq!(
            operation
                .reserve_hashed("second".into(), 3, &mut admission)
                .unwrap()
                .publish(),
            second
        );
    }
}
