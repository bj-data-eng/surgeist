//! One bounded weak index, independent of historical snapshots and change logs.
use super::*;
use crate::DomDocumentStamp;
use crate::dispatch::{
    DomDispatchAssociation, DomDispatchCustody, DomDispatchCustodyLimits, Ledger, Release,
    ReleaseNode, Row, Scope,
};
use source::formation::Formation;
use std::{
    cell::Cell,
    sync::{Mutex, OnceLock, Weak},
};

#[derive(Default)]
pub(super) struct CustodyBank {
    scopes: bank::Bank<ScopeRecord>,
    targets: bank::Bank<TargetBucket>,
    ledger: Option<Arc<Mutex<Ledger>>>,
    next_row: usize,
}

struct ScopeRecord {
    scope: Weak<Scope>,
    targets: Vec<CleanupRow>,
}
struct CleanupRow {
    target: NodeId,
    key: usize,
}
struct TargetBucket {
    generations: bank::Bank<Generation>,
}
enum Generation {
    Branch(bank::Bank<Generation>),
    Rows(bank::Bank<IndexedRow>),
}
struct IndexedRow {
    target: NodeId,
    scope: usize,
    index: usize,
}

/// Finite fresh-scope fixture policy, priced from owning types rather than a
/// measured peak. Only the flat, fresh generation-zero qualification uses it.
#[cfg(test)]
pub(in crate::store) fn scale_fresh_limits(n: usize) -> DomDispatchCustodyLimits {
    assert_eq!(usize::BITS, 64, "qualification word-width premise");
    assert_eq!(LAST_GENERATION_PART, 1, "two generation index parts");
    assert!(n > 0);
    let capacity = n
        .checked_next_power_of_two()
        .expect("fixture capacity fits");
    let depth = |mut index: usize| {
        let mut depth = 1usize;
        while index >= 16 {
            index /= 16;
            depth += 1;
        }
        depth
    };
    let row_depth = depth(n - 1);
    let target_depth = depth(n.checked_add(4).expect("last fresh slot fits"));
    let sum = |values: &[usize]| {
        values
            .iter()
            .try_fold(0usize, |total, value| total.checked_add(*value))
            .expect("typed fixture sum fits")
    };
    let mul = |a: usize, b: usize| a.checked_mul(b).expect("typed fixture product fits");
    // C covers actual CleanupRow and pre-box Row capacities only after the
    // scoped setup verifier confirms them on this host. It is not a Vec API
    // guarantee. Extra Row/ReleaseNode bodies cover boxing coexistence.
    let headers = sum(&[
        mul(capacity, size_of::<CleanupRow>()),
        mul(mul(2, capacity), size_of::<Row>()),
        size_of::<ScopeRecord>(),
        size_of::<Scope>(),
        mul(2, size_of::<ReleaseNode>()),
        mul(2, size_of::<Mutex<Ledger>>()),
    ]);
    let per_target = sum(&[
        size_of::<IndexedRow>(),
        size_of::<TargetBucket>(),
        mul(2, size_of::<Generation>()),
        mul(row_depth, bank::scale_full_page_bytes::<IndexedRow>()),
        mul(2, bank::scale_full_page_bytes::<Generation>()),
    ]);
    // Final target paths plus two simultaneous selected-path/root-growth
    // overlaps; the fresh original bank contributes no retained base.
    let bytes = sum(&[
        headers,
        bank::scale_full_page_bytes::<ScopeRecord>(),
        mul(
            mul(n.checked_add(2).unwrap(), target_depth),
            bank::scale_full_page_bytes::<TargetBucket>(),
        ),
        mul(n, per_target),
    ]);
    let work = u64::try_from(n).unwrap().checked_mul(128).unwrap();
    // register validation/rows/records plus edit_generation's two singleton
    // parts and old/new selected native paths; no whole-bank search is priced.
    let per_target_steps = sum(&[8, mul(2, sum(&[row_depth, 2, mul(2, target_depth)]))]);
    let work_bound = u64::try_from(sum(&[16, mul(n, per_target_steps)])).unwrap();
    assert!(
        work_bound <= work,
        "unchanged finite work covers the fresh callgraph"
    );
    eprintln!(
        "scale dispatch_setup_policy targets={n} capacity_premise={capacity} row_depth={row_depth} target_depth={target_depth} headers={headers} per_target={per_target} scratch_and_retained_bytes={bytes} work={work} source_derived_work_bound={work_bound} policy_not_oracle=true"
    );
    DomDispatchCustodyLimits::new(1, n, bytes, bytes, work).unwrap()
}

#[cfg(test)]
#[derive(Default)]
struct ScaleReserves {
    bound: usize,
    cleanup: Option<usize>,
    rows: Option<usize>,
    release: Option<usize>,
}
#[cfg(test)]
thread_local! {
    static SCALE_RESERVES: std::cell::RefCell<Option<ScaleReserves>> = const {
        std::cell::RefCell::new(None)
    };
}
#[cfg(test)]
pub(in crate::store) struct ScaleSetupVerification {
    pages: Option<bank::ScalePageCapacityVerification>,
}
#[cfg(test)]
impl ScaleSetupVerification {
    pub fn begin(dom: &Dom, targets: &[NodeId]) -> Self {
        let bank = &dom.dispatch_custody;
        assert_eq!(bank.scopes.record_count(), 0);
        assert_eq!(bank.targets.record_count(), 0);
        assert!(bank.ledger.is_none());
        assert_eq!(bank.next_row, 0);
        for (index, target) in targets.iter().enumerate() {
            assert_eq!(target.owner, dom.owner);
            assert_eq!(target.slot, index.checked_add(5).unwrap());
            assert_eq!(target.generation, 0);
        }
        if !scale_metrics::ENABLED {
            eprintln!(
                "scale dispatch_setup_capacity_verification=reuse_instrumented_same_types_toolchain_allocator public_admission_still_enforced=true"
            );
            return Self { pages: None };
        }
        SCALE_RESERVES.with(|state| {
            assert!(state.borrow().is_none());
        });
        let pages = bank::ScalePageCapacityVerification::begin();
        SCALE_RESERVES.with(|state| {
            *state.borrow_mut() = Some(ScaleReserves {
                bound: targets.len().checked_next_power_of_two().unwrap(),
                ..ScaleReserves::default()
            });
        });
        Self { pages: Some(pages) }
    }
    pub fn finish(mut self) {
        if let Some(pages) = self.pages.take() {
            let pages = pages.finish();
            let reserves = SCALE_RESERVES.with(|state| state.borrow_mut().take().unwrap());
            assert!(pages > 0);
            let cleanup = reserves.cleanup.expect("actual cleanup reserve observed");
            let rows = reserves.rows.expect("actual row reserve observed");
            let release = reserves.release.expect("actual release reserve observed");
            eprintln!(
                "scale dispatch_setup_capacity_verification=actual setup_only=true pages_verified={pages} cleanup_capacity={cleanup} row_capacity={rows} release_capacity={release} capacity_bound={} typed_page_charge_premise=true",
                reserves.bound
            );
        }
    }
}
#[cfg(test)]
impl Drop for ScaleSetupVerification {
    fn drop(&mut self) {
        if scale_metrics::ENABLED {
            SCALE_RESERVES.with(|state| {
                state.borrow_mut().take();
            });
        }
    }
}
#[cfg(test)]
fn scale_verify_reserve<T>(capacity: usize) {
    SCALE_RESERVES.with(|state| {
        let mut state = state.borrow_mut();
        let Some(state) = state.as_mut() else { return };
        let (observed, bound) = if std::any::type_name::<T>() == std::any::type_name::<CleanupRow>()
        {
            (&mut state.cleanup, state.bound)
        } else if std::any::type_name::<T>() == std::any::type_name::<Row>() {
            (&mut state.rows, state.bound)
        } else {
            assert_eq!(
                std::any::type_name::<T>(),
                std::any::type_name::<ReleaseNode>()
            );
            (&mut state.release, 1)
        };
        assert!(
            capacity <= bound,
            "actual setup reserve exceeds capacity premise"
        );
        assert!(
            observed.replace(capacity).is_none(),
            "one actual reserve of each owning type"
        );
    });
}

/// Actual staged association, resolved by the original Draft's final mapping.
pub(super) struct Retirement {
    pub target: NodeId,
    pub document: DocumentId,
}

struct Processing<'a> {
    limits: DomDispatchCustodyLimits,
    spent: Cell<u64>,
    formation: Option<&'a Formation>,
    cancel: &'a AtomicBool,
}
impl<'a> Processing<'a> {
    fn new(
        limits: DomDispatchCustodyLimits,
        formation: Option<&'a Formation>,
        cancel: &'a AtomicBool,
    ) -> Self {
        Self {
            limits,
            spent: Cell::new(0),
            formation,
            cancel,
        }
    }
    fn step(&self) -> Result<(), DomError> {
        check_cancel(self.cancel)?;
        if let Some(formation) = self.formation {
            formation.step(self.cancel)?;
        }
        let next = self
            .spent
            .get()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::DispatchCustodyWorkLimit {
                limit: self.limits.work,
            });
        }
        self.spent.set(next);
        Ok(())
    }
    fn scratch(&self, bytes: usize) -> Result<(), DomError> {
        admit(
            bytes,
            self.limits.scratch_bytes,
            DomResource::DispatchCustodyScratch,
        )
    }
}

#[derive(Default)]
struct Extent {
    scopes: usize,
    references: usize,
    bytes: usize,
}
impl Extent {
    fn admit(&self, limits: DomDispatchCustodyLimits) -> Result<(), DomError> {
        admit(
            self.scopes,
            limits.scopes,
            DomResource::DispatchCustodyScopes,
        )?;
        admit(
            self.references,
            limits.references,
            DomResource::DispatchCustodyReferences,
        )?;
        admit(
            self.bytes,
            limits.retained_bytes,
            DomResource::DispatchCustodyRetainedBytes,
        )
    }
}
fn bytes<T>(capacity: usize) -> Result<usize, DomError> {
    capacity
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)
}
fn add(a: usize, b: usize) -> Result<usize, DomError> {
    a.checked_add(b).ok_or(DomError::RepresentationExhausted)
}

/// All temporary backing is admitted before retention. A completed path update
/// releases superseded candidate objects; spent work itself remains monotonic.
struct Allocation<'a, 'p> {
    processing: &'a Processing<'p>,
    base: usize,
    held: usize,
    guard: Option<Formation>,
}
impl<'a, 'p> Allocation<'a, 'p> {
    fn new(processing: &'a Processing<'p>, base: usize) -> Self {
        Self {
            processing,
            base,
            held: 0,
            guard: processing.formation.map(Formation::fork),
        }
    }
    fn retain(&mut self, amount: usize) -> Result<(), DomError> {
        if amount == 0 {
            return Ok(());
        }
        self.processing.step()?;
        let held = add(self.held, amount)?;
        self.processing.scratch(held)?;
        admit(
            add(self.base, held)?,
            self.processing.limits.retained_bytes,
            DomResource::DispatchCustodyRetainedBytes,
        )?;
        if let Some(guard) = &self.guard {
            guard.retain_bytes(amount, self.processing.cancel)?;
        }
        self.held = held;
        Ok(())
    }
    fn release(&mut self, released: usize) {
        self.held = self
            .held
            .checked_sub(released)
            .expect("candidate backing was admitted before allocation");
        if let Some(guard) = &self.guard {
            guard.release_bytes(released);
        }
    }
    fn index(&mut self, charge: bank::IndexCharge) -> Result<(), DomError> {
        self.retain(charge.bytes)
    }
    /// Fields already own their separately admitted backing. Reserve this
    /// materialization's identity before admitting and allocating its header.
    fn record<T>(&mut self, value: T) -> Result<bank::Record<T>, DomError> {
        let reserved = bank::Record::reserve()?;
        self.retain(size_of::<T>())?;
        Ok(reserved.materialize(value))
    }
}
fn reserve<T>(count: usize, allocation: &mut Allocation<'_, '_>) -> Result<Vec<T>, DomError> {
    let requested = bytes::<T>(count)?;
    allocation.retain(requested)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| DomError::AllocationFailed)?;
    #[cfg(test)]
    if scale_metrics::ENABLED {
        scale_verify_reserve::<T>(values.capacity());
    }
    let actual = bytes::<T>(values.capacity())?;
    #[cfg(test)]
    record_allocation(actual);
    allocation.retain(actual - requested)?;
    Ok(values)
}
#[cfg(test)]
fn record_allocation(bytes: usize) {
    if scale_metrics::ENABLED && bytes != 0 {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes });
    }
}
#[cfg(test)]
fn record_sequence(bytes: usize) {
    if scale_metrics::ENABLED && bytes != 0 {
        scale_metrics::record(scale_metrics::Event::SequenceMaterialized { bytes });
    }
}
fn totals(bytes: usize) -> bank::Totals {
    bank::Totals {
        bytes,
        ..bank::Totals::default()
    }
}

// Preserve all u64 generation bits on every pointer width. Leaving one native
// bit clear keeps every key below usize::MAX, whose extent cannot be indexed.
// This requires two levels on 64-bit hosts and three on 32-bit hosts.
const GENERATION_PART_BITS: usize = usize::BITS as usize - 1;
const LAST_GENERATION_PART: usize = 64_usize.div_ceil(GENERATION_PART_BITS) - 1;
fn generation_part(generation: u64, part: usize) -> usize {
    ((generation >> (part * GENERATION_PART_BITS)) as usize)
        & ((1_usize << GENERATION_PART_BITS) - 1)
}
impl Generation {
    fn retained_bytes(&self) -> Result<usize, DomError> {
        let backing = match self {
            Self::Branch(children) => add(children.index_charge().bytes, children.totals().bytes)?,
            Self::Rows(rows) => add(rows.index_charge().bytes, rows.totals().bytes)?,
        };
        add(size_of::<Self>(), backing)
    }
}
impl TargetBucket {
    fn retained_bytes(&self) -> Result<usize, DomError> {
        add(
            size_of::<Self>(),
            add(
                self.generations.index_charge().bytes,
                self.generations.totals().bytes,
            )?,
        )
    }
    fn rows(
        &self,
        target: NodeId,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<&bank::Bank<IndexedRow>>, DomError> {
        let mut generations = &self.generations;
        for part in (0..=LAST_GENERATION_PART).rev() {
            step()?;
            let Some(generation) = generations.get(generation_part(target.generation, part)) else {
                return Ok(None);
            };
            match generation {
                Generation::Branch(children) => generations = children,
                Generation::Rows(rows) => {
                    assert_eq!(part, 0, "complete generation selects its rows");
                    return Ok(Some(rows));
                }
            }
        }
        unreachable!("complete generation index has a row leaf")
    }
}

fn replaced_generation_bytes(
    candidate: &bank::Bank<Generation>,
    base: &bank::Bank<Generation>,
    target: NodeId,
    part: usize,
    key: usize,
) -> Result<usize, DomError> {
    let index = generation_part(target.generation, part);
    let mut bytes = candidate
        .path_index_charge_not_shared_with(base, index)
        .bytes;
    let Some(generation) = candidate.get_shared(index) else {
        return Ok(bytes);
    };
    let original = base.get_shared(index);
    if !original.is_some_and(|original| Arc::ptr_eq(original, generation)) {
        bytes = add(bytes, size_of::<Generation>())?;
    }
    let backing = match generation.as_ref() {
        Generation::Rows(rows) => {
            let empty = bank::Bank::default();
            let original = original
                .and_then(|original| match original.as_ref() {
                    Generation::Rows(rows) => Some(rows),
                    Generation::Branch(_) => None,
                })
                .unwrap_or(&empty);
            rows.path_index_charge_not_shared_with(original, key).bytes
        }
        Generation::Branch(children) => {
            let empty = bank::Bank::default();
            let original = original
                .and_then(|original| match original.as_ref() {
                    Generation::Branch(children) => Some(children),
                    Generation::Rows(_) => None,
                })
                .unwrap_or(&empty);
            replaced_generation_bytes(children, original, target, part - 1, key)?
        }
    };
    add(bytes, backing)
}

/// Rebuild only one exact generation path and row path. Recursion is bounded
/// by the number of native words in u64, independent of participant count.
fn edit_generation(
    generations: &mut bank::Bank<Generation>,
    target: NodeId,
    part: usize,
    key: usize,
    row: Option<bank::Record<IndexedRow>>,
    allocation: &mut Allocation<'_, '_>,
) -> Result<(), DomError> {
    let index = generation_part(target.generation, part);
    let generation = if part == 0 {
        let mut rows = match generations.get(index) {
            Some(Generation::Rows(rows)) => rows.clone(),
            None => bank::Bank::default(),
            Some(Generation::Branch(_)) => unreachable!("final generation part has rows"),
        };
        if let Some(row) = row {
            rows.set_admitted(key, row, false, totals(size_of::<IndexedRow>()), |charge| {
                allocation.index(charge)
            })?;
        } else {
            rows.remove_admitted(key, |charge| allocation.index(charge))?;
        }
        (rows.record_count() != 0).then_some(Generation::Rows(rows))
    } else {
        let mut children = match generations.get(index) {
            Some(Generation::Branch(children)) => children.clone(),
            None => bank::Bank::default(),
            Some(Generation::Rows(_)) => unreachable!("generation prefix has children"),
        };
        edit_generation(&mut children, target, part - 1, key, row, allocation)?;
        (children.record_count() != 0).then_some(Generation::Branch(children))
    };
    if let Some(generation) = generation {
        let retained = totals(generation.retained_bytes()?);
        let generation = allocation.record(generation)?;
        generations.set_admitted(index, generation, false, retained, |charge| {
            allocation.index(charge)
        })?;
    } else {
        generations.remove_admitted(index, |charge| allocation.index(charge))?;
    }
    Ok(())
}
impl CustodyBank {
    pub(super) fn has_indexed_scopes(&self) -> bool {
        self.scopes.record_count() != 0
    }

    /// Both slot and generation select only their actual custody participants.
    pub(super) fn tracks(
        &self,
        target: NodeId,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        if self.targets.record_count() == 0 {
            return Ok(false);
        }
        step()?;
        let Some(bucket) = self.targets.get(target.slot) else {
            return Ok(false);
        };
        let Some(rows) = bucket.rows(target, &mut step)? else {
            return Ok(false);
        };
        for row in rows.values() {
            step()?;
            if row.target == target
                && self
                    .scopes
                    .get(row.scope)
                    .is_some_and(|scope| scope.scope.strong_count() != 0)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub(super) fn retained_bytes(
        &self,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        Ok(self.extent_with(step)?.bytes)
    }

    fn extent_for(&self, references: usize, live_scopes: usize) -> Result<Extent, DomError> {
        let mut bytes = add(
            add(self.scopes.index_charge().bytes, self.scopes.totals().bytes)?,
            add(
                self.targets.index_charge().bytes,
                self.targets.totals().bytes,
            )?,
        )?;
        bytes = add(bytes, crate::store::dispatch::bytes::<Row>(references)?)?;
        bytes = add(
            bytes,
            crate::store::dispatch::bytes::<Mutex<Ledger>>(live_scopes)?,
        )?;
        if self.ledger.is_some() {
            bytes = add(bytes, size_of::<Mutex<Ledger>>())?;
        }
        Ok(Extent {
            scopes: self.scopes.record_count(),
            references,
            bytes,
        })
    }
    fn extent(&self, processing: &Processing<'_>) -> Result<Extent, DomError> {
        self.extent_with(|| processing.step())
    }
    fn extent_with(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Extent, DomError> {
        let Some(ledger) = &self.ledger else {
            return self.extent_for(0, 0);
        };
        step()?;
        let ledger = ledger.lock().unwrap_or_else(|error| error.into_inner());
        self.extent_for(ledger.references, ledger.live_scopes)
    }

    /// Candidate-only objects replaced by this exact target-row edit. The old
    /// bucket keeps its row root until the outer target update succeeds.
    fn replaced_target_bytes(
        &self,
        candidate: &bank::Bank<TargetBucket>,
        target: NodeId,
        key: usize,
    ) -> Result<usize, DomError> {
        let slot = target.slot;
        let mut bytes = candidate
            .path_index_charge_not_shared_with(&self.targets, slot)
            .bytes;
        if let Some(bucket) = candidate.get_shared(slot) {
            let base = self.targets.get_shared(slot);
            if !base.is_some_and(|base| Arc::ptr_eq(base, bucket)) {
                bytes = add(bytes, size_of::<TargetBucket>())?;
            }
            let empty = bank::Bank::default();
            let original = base.map_or(&empty, |base| &base.generations);
            bytes = add(
                bytes,
                replaced_generation_bytes(
                    &bucket.generations,
                    original,
                    target,
                    LAST_GENERATION_PART,
                    key,
                )?,
            )?;
        }
        Ok(bytes)
    }

    /// Refusal leaves both original roots and the preallocated release list
    /// intact. Only the released scopes and their actual row paths are reached.
    fn prune(&mut self, processing: &Processing<'_>) -> Result<(), DomError> {
        let Some(shared) = self.ledger.clone() else {
            return Ok(());
        };
        let mut ledger = shared.lock().unwrap_or_else(|error| error.into_inner());
        if ledger.released.is_none() {
            return Ok(());
        }
        let mut allocation = Allocation::new(
            processing,
            self.extent_for(ledger.references, ledger.live_scopes)?
                .bytes,
        );
        let mut scopes = self.scopes.clone();
        let mut targets = self.targets.clone();
        let mut released = ledger.released.as_deref();
        while let Some(node) = released {
            processing.step()?;
            let slot = node[0].scope;
            let scope = self.scopes.get(slot).expect("queued registered scope");
            for cleanup in &scope.targets {
                processing.step()?;
                let replaced = self.replaced_target_bytes(&targets, cleanup.target, cleanup.key)?;
                let mut generations = targets
                    .get(cleanup.target.slot)
                    .expect("registered target bucket")
                    .generations
                    .clone();
                edit_generation(
                    &mut generations,
                    cleanup.target,
                    LAST_GENERATION_PART,
                    cleanup.key,
                    None,
                    &mut allocation,
                )?;
                if generations.record_count() == 0 {
                    targets
                        .remove_admitted(cleanup.target.slot, |charge| allocation.index(charge))?;
                } else {
                    let bucket = TargetBucket { generations };
                    let charge = totals(bucket.retained_bytes()?);
                    let bucket = allocation.record(bucket)?;
                    targets.set_admitted(cleanup.target.slot, bucket, false, charge, |charge| {
                        allocation.index(charge)
                    })?;
                }
                allocation.release(replaced);
            }
            let replaced = scopes
                .path_index_charge_not_shared_with(&self.scopes, slot)
                .bytes;
            scopes.remove_admitted(slot, |charge| allocation.index(charge))?;
            allocation.release(replaced);
            released = node[0].next.as_deref();
        }
        check_cancel(processing.cancel)?;
        self.scopes = scopes;
        self.targets = targets;
        let released = ledger.released.take();
        drop(ledger);
        Ledger::discard(released);
        if self.scopes.record_count() == 0 {
            self.ledger = None;
        }
        Ok(())
    }

    pub(super) fn admit_limits(
        &mut self,
        limits: DomDispatchCustodyLimits,
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let processing = Processing::new(limits, formation, cancel);
        self.prune(&processing)?;
        self.extent(&processing)?.admit(limits)
    }

    fn register(
        &mut self,
        owner: DomOwnerId,
        state: &State,
        targets: &[NodeId],
        processing: &Processing<'_>,
    ) -> Result<DomDispatchCustody, DomError> {
        self.prune(processing)?;
        let extent = self.extent(processing)?;
        let count = add(self.scopes.record_count(), 1)?;
        admit(
            count,
            processing.limits.scopes,
            DomResource::DispatchCustodyScopes,
        )?;
        admit(
            add(extent.references, targets.len())?,
            processing.limits.references,
            DomResource::DispatchCustodyReferences,
        )?;
        let next_row = add(self.next_row, targets.len())?;
        let slot = self.scopes.lowest_vacant().unwrap_or(self.scopes.len());
        processing.step()?; // Scope registration and eventual queued release.
        for target in targets {
            processing.step()?;
            state.node(owner, *target)?;
        }
        let mut allocation = Allocation::new(processing, extent.bytes);
        let mut cleanup = reserve::<CleanupRow>(targets.len(), &mut allocation)?;
        for (index, target) in targets.iter().enumerate() {
            cleanup.push(CleanupRow {
                target: *target,
                key: self.next_row + index,
            });
            #[cfg(test)]
            record_sequence(size_of::<CleanupRow>());
        }
        let mut rows = reserve::<Row>(targets.len(), &mut allocation)?;
        let construction = bytes::<Row>(rows.capacity())?;
        for target in targets {
            processing.step()?;
            processing.step()?; // Pre-admit eventual row release.
            rows.push(Row {
                target: *target,
                retired: OnceLock::new(),
            });
            #[cfg(test)]
            record_sequence(size_of::<Row>());
        }
        let row_bytes = bytes::<Row>(targets.len())?;
        if construction != row_bytes {
            allocation.retain(row_bytes)?; // Boxing may replace spare backing.
        }
        #[cfg(test)]
        let original_rows = rows.as_ptr();
        let rows = rows.into_boxed_slice();
        #[cfg(test)]
        if original_rows != rows.as_ptr() {
            record_allocation(row_bytes);
            record_sequence(row_bytes);
        }
        if construction != row_bytes {
            allocation.release(construction);
        }
        let mut release = reserve::<ReleaseNode>(1, &mut allocation)?;
        let construction = bytes::<ReleaseNode>(release.capacity())?;
        release.push(ReleaseNode {
            scope: slot,
            next: None,
        });
        #[cfg(test)]
        record_sequence(size_of::<ReleaseNode>());
        if construction != size_of::<ReleaseNode>() {
            allocation.retain(size_of::<ReleaseNode>())?;
        }
        #[cfg(test)]
        let original_release = release.as_ptr();
        let release = release.into_boxed_slice();
        #[cfg(test)]
        if original_release != release.as_ptr() {
            record_allocation(size_of::<ReleaseNode>());
            record_sequence(size_of::<ReleaseNode>());
        }
        if construction != size_of::<ReleaseNode>() {
            allocation.release(construction);
        }
        let ledger = match &self.ledger {
            Some(ledger) => ledger.clone(),
            None => {
                allocation.retain(size_of::<Mutex<Ledger>>())?;
                let ledger = Arc::new(Mutex::new(Ledger::default()));
                #[cfg(test)]
                record_allocation(size_of::<Mutex<Ledger>>());
                ledger
            }
        };
        // Each materialized Scope independently retains this weak-field
        // backing occurrence, even though the bank also owns the ledger.
        allocation.retain(size_of::<Mutex<Ledger>>())?;
        allocation.retain(size_of::<Scope>())?;
        let scope = Arc::new(Scope {
            owner,
            rows,
            release: OnceLock::new(),
        });
        #[cfg(test)]
        record_allocation(size_of::<Scope>());
        let record = allocation.record(ScopeRecord {
            scope: Arc::downgrade(&scope),
            targets: cleanup,
        })?;
        let scope_bytes = add(
            add(
                size_of::<ScopeRecord>(),
                bytes::<CleanupRow>(record.value().targets.capacity())?,
            )?,
            add(size_of::<Scope>(), size_of::<ReleaseNode>())?,
        )?;
        let mut scopes = self.scopes.clone();
        let mut target_index = self.targets.clone();
        scopes.set_admitted(slot, record, false, totals(scope_bytes), |charge| {
            allocation.index(charge)
        })?;
        for (index, target) in targets.iter().enumerate() {
            processing.step()?;
            let replaced =
                self.replaced_target_bytes(&target_index, *target, self.next_row + index)?;
            let row = allocation.record(IndexedRow {
                target: *target,
                scope: slot,
                index,
            })?;
            let mut generations = target_index
                .get(target.slot)
                .map_or_else(bank::Bank::default, |bucket| bucket.generations.clone());
            edit_generation(
                &mut generations,
                *target,
                LAST_GENERATION_PART,
                self.next_row + index,
                Some(row),
                &mut allocation,
            )?;
            let bucket = TargetBucket { generations };
            let charge = totals(bucket.retained_bytes()?);
            let bucket = allocation.record(bucket)?;
            target_index.set_admitted(target.slot, bucket, false, charge, |charge| {
                allocation.index(charge)
            })?;
            allocation.release(replaced);
        }
        check_cancel(processing.cancel)?;
        let mut live = ledger.lock().unwrap_or_else(|error| error.into_inner());
        live.references = live
            .references
            .checked_add(targets.len())
            .expect("admitted row count");
        live.live_scopes = live
            .live_scopes
            .checked_add(1)
            .expect("admitted scope count");
        assert!(
            scope
                .release
                .set(Release {
                    ledger: Arc::downgrade(&ledger),
                    node: release,
                })
                .is_ok(),
            "unpublished original scope"
        );
        self.scopes = scopes;
        self.targets = target_index;
        self.next_row = next_row;
        self.ledger = Some(ledger.clone());
        drop(live);
        Ok(DomDispatchCustody { scope })
    }

    pub(super) fn prepare_publication(
        &mut self,
        retirements: &[Retirement],
        stamp: DomStamp,
        limits: DomDispatchCustodyLimits,
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<PublicationPlan, DomError> {
        if !self.has_indexed_scopes() || retirements.is_empty() {
            return Ok(PublicationPlan::default());
        }
        let processing = Processing::new(limits, formation, cancel);
        self.prune(&processing)?;
        let extent = self.extent(&processing)?;
        extent.admit(limits)?;
        let mut allocation = Allocation::new(&processing, extent.bytes);
        let mut seen = bank::Bank::default();
        let empty = bank::Bank::default();
        let mut count = 0;
        for retirement in retirements {
            processing.step()?;
            let Some(bucket) = self.targets.get(retirement.target.slot) else {
                continue;
            };
            let Some(participants) = bucket.rows(retirement.target, || processing.step())? else {
                continue;
            };
            for (key, row) in participants.iter() {
                processing.step()?;
                if row.target != retirement.target {
                    continue;
                }
                let Some(scope) = self
                    .scopes
                    .get(row.scope)
                    .and_then(|row| row.scope.upgrade())
                else {
                    continue;
                };
                if scope.rows[row.index].retired.get().is_some() {
                    return Err(DomError::InvalidDispatchCustody);
                }
                processing.step()?;
                if seen.get(key).is_some() {
                    return Err(DomError::InvalidDispatchCustody);
                }
                let replaced = seen.path_index_charge_not_shared_with(&empty, key).bytes;
                let marker = allocation.record(key)?;
                seen.set_admitted(key, marker, false, totals(size_of::<usize>()), |charge| {
                    allocation.index(charge)
                })?;
                allocation.release(replaced);
                count = add(count, 1)?;
            }
        }
        let seen_bytes = add(seen.index_charge().bytes, seen.totals().bytes)?;
        #[cfg(test)]
        assert_eq!(
            allocation.held, seen_bytes,
            "actual matched-row scratch backing"
        );
        drop(seen);
        allocation.release(seen_bytes);
        let guard = allocation.guard.take();
        let requested = bytes::<PlannedRow>(count)?;
        processing.scratch(requested)?;
        if let Some(guard) = &guard {
            guard.retain_bytes(requested, cancel)?;
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = bytes::<PlannedRow>(rows.capacity())?;
        #[cfg(test)]
        record_allocation(actual);
        processing.scratch(actual)?;
        if let Some(guard) = &guard {
            guard.retain_bytes(actual - requested, cancel)?;
        }
        for retirement in retirements {
            processing.step()?;
            let Some(bucket) = self.targets.get(retirement.target.slot) else {
                continue;
            };
            let Some(participants) = bucket.rows(retirement.target, || processing.step())? else {
                continue;
            };
            for row in participants.values() {
                processing.step()?;
                if row.target != retirement.target {
                    continue;
                }
                let Some(scope) = self
                    .scopes
                    .get(row.scope)
                    .and_then(|row| row.scope.upgrade())
                else {
                    continue;
                };
                if retirement.document.node().owner() != stamp.owner()
                    || scope.rows[row.index].retired.get().is_some()
                {
                    return Err(DomError::InvalidDispatchCustody);
                }
                processing.step()?; // The infallible retirement-cell publication.
                rows.push(PlannedRow {
                    scope,
                    index: row.index,
                    stamp: DomDocumentStamp {
                        store: stamp,
                        document: retirement.document,
                    },
                });
                #[cfg(test)]
                record_sequence(size_of::<PlannedRow>());
            }
        }
        check_cancel(cancel)?;
        Ok(PublicationPlan {
            rows,
            _formation: guard,
        })
    }
}

struct PlannedRow {
    scope: Arc<Scope>,
    index: usize,
    stamp: DomDocumentStamp,
}
#[derive(Default)]
pub(super) struct PublicationPlan {
    rows: Vec<PlannedRow>,
    _formation: Option<Formation>,
}
impl PublicationPlan {
    /// Called only at the exclusive, infallible canonical publication fence.
    pub(super) fn publish(self) {
        for row in self.rows {
            assert!(
                row.scope.rows[row.index].retired.set(row.stamp).is_ok(),
                "preflighted unpublished retirement cell"
            );
        }
    }
}

impl Dom {
    /// Actual aggregate producer-owned payload across every registered scope
    /// and the canonical weak index, including retained physical capacity.
    pub fn dispatch_custody_retained_bytes(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        let formation = self.dispatch_registration_formation(cancel)?;
        let processing = Processing::new(self.limits.dispatch_custody, formation.as_ref(), cancel);
        Ok(self.dispatch_custody.extent(&processing)?.bytes)
    }
    /// Register all actual Node invocation targets before exposing a dispatch.
    pub fn register_dispatch_custody(
        &mut self,
        targets: &[NodeId],
        cancel: &AtomicBool,
    ) -> Result<DomDispatchCustody, DomError> {
        let formation = self.dispatch_registration_formation(cancel)?;
        self.register_dispatch_custody_with_formation(targets, formation.as_ref(), cancel)
    }
    pub(in crate::store) fn register_dispatch_custody_with_formation(
        &mut self,
        targets: &[NodeId],
        formation: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<DomDispatchCustody, DomError> {
        let processing = Processing::new(self.limits.dispatch_custody, formation, cancel);
        self.dispatch_custody
            .register(self.owner, &self.state, targets, &processing)
    }
    /// Qualify exactly one original row against this canonical owner. Live rows
    /// always use current association; a missing retirement cell remains refusal.
    pub fn dispatch_custody_association(
        &self,
        custody: &DomDispatchCustody,
        row: usize,
        target: NodeId,
    ) -> Result<DomDispatchAssociation, DomError> {
        if custody.scope.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let row = custody
            .scope
            .rows
            .get(row)
            .filter(|row| row.target == target)
            .ok_or(DomError::InvalidDispatchCustody)?;
        match self.state.node(self.owner, target) {
            Ok(node) => Ok(DomDispatchAssociation::Current(node.document())),
            Err(DomError::StaleNode) => row
                .retired
                .get()
                .copied()
                .map(DomDispatchAssociation::Retired)
                .ok_or(DomError::StaleNode),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod incremental_tests;
#[cfg(test)]
mod tests;
