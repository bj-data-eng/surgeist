//! Actual Layout-owned scope resource custody.
use super::{ScopeAlignmentSnapshot, check, overflow, refusal, reserve_buffer};
use crate::LayoutScalar;
use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};
use surgeist_text::{
    CapturedScope, FinalizedLine, PreparedText, ScopeAlignmentCensusInput, ScopeAlignmentError,
    ScopeAlignmentLimitKind, ScopeAlignmentLimits, ScopeAlignmentProduct, ScopeAlignmentProducts,
    ScopeAlignmentWeakProduct,
};

/// Actual live Layout payload, separate from a phase's materialized-row history.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ScopeCensus {
    pub(super) records: usize,
    pub(super) bytes: usize,
}
impl ScopeCensus {
    pub(super) fn add(self, other: Self) -> surgeist_text::Result<Self> {
        Ok(Self {
            records: self
                .records
                .checked_add(other.records)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
            bytes: self
                .bytes
                .checked_add(other.bytes)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
        })
    }
}

#[derive(Debug)]
struct LiveScopeResources {
    state: Mutex<ScopeResourceState>,
    registry: Mutex<Registry>,
}
#[derive(Debug)]
struct ScopeResourceState {
    live: ScopeCensus,
    materialized: usize,
    work: usize,
}
#[derive(Debug, Default)]
struct Registry {
    weak: Vec<Option<ScopeAlignmentWeakProduct>>,
    strong: Option<Vec<Option<FinalizedLine>>>,
    loaned: bool,
}
impl LiveScopeResources {
    fn lock(&self) -> std::sync::MutexGuard<'_, ScopeResourceState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn retain(&self, census: ScopeCensus) -> surgeist_text::Result<()> {
        let mut state = self.lock();
        state.live = state.live.add(census)?;
        Ok(())
    }
    fn release(&self, census: ScopeCensus) {
        let mut state = self.lock();
        state.live = ScopeCensus {
            records: state
                .live
                .records
                .checked_sub(census.records)
                .expect("each scope payload releases its own retained rows once"),
            bytes: state
                .live
                .bytes
                .checked_sub(census.bytes)
                .expect("each scope payload releases its own retained capacity once"),
        };
    }
}

#[derive(Debug)]
pub(crate) struct ScopeLease {
    state: Arc<LiveScopeResources>,
    census: ScopeCensus,
}
impl Drop for ScopeLease {
    fn drop(&mut self) {
        self.state.release(self.census);
    }
}

#[derive(Debug)]
pub(crate) struct ScopeInputOwner<S: LayoutScalar> {
    snapshot: Arc<ScopeAlignmentSnapshot<S>>,
    state: Arc<LiveScopeResources>,
    census: ScopeCensus,
}
impl<S: LayoutScalar> Drop for ScopeInputOwner<S> {
    fn drop(&mut self) {
        self.state.release(self.census);
    }
}

/// One computation/session family's live feature custody. It never owns a
/// proof or a lease, so proof leases cannot form a strong ownership cycle.
#[derive(Clone, Debug)]
pub(crate) enum ScopeAlignmentResources<S: LayoutScalar> {
    Empty(Arc<ScopeAlignmentSnapshot<S>>),
    Active(Arc<ScopeInputOwner<S>>),
}
impl<S: LayoutScalar> ScopeAlignmentResources<S> {
    pub(crate) fn new(snapshot: Arc<ScopeAlignmentSnapshot<S>>) -> Self {
        if snapshot.retention.bytes == 0 {
            return Self::Empty(snapshot);
        }
        let census = snapshot
            .retention
            .add(ScopeCensus {
                records: 0,
                bytes: size_of::<ScopeInputOwner<S>>(),
            })
            .expect("scope admission checked input-owner header arithmetic");
        let live = census
            .add(ScopeCensus {
                records: 0,
                bytes: size_of::<LiveScopeResources>(),
            })
            .expect("scope admission checked resource-state header arithmetic");
        let state = Arc::new(LiveScopeResources {
            state: Mutex::new(ScopeResourceState {
                live,
                materialized: census.records,
                work: 0,
            }),
            registry: Mutex::new(Registry::default()),
        });
        Self::Active(Arc::new(ScopeInputOwner {
            snapshot,
            state,
            census,
        }))
    }
    pub(crate) fn snapshot(&self) -> &Arc<ScopeAlignmentSnapshot<S>> {
        match self {
            Self::Empty(snapshot) => snapshot,
            Self::Active(owner) => &owner.snapshot,
        }
    }
    /// Discover the actual constraining capture on the stack. Each normalized
    /// descriptor visit is admitted before continuing; no policy summary is
    /// retained beside the immutable input facts.
    pub(crate) fn key_limits(&self) -> surgeist_text::Result<Option<ScopeAlignmentLimits>> {
        let mut tightest: Option<ScopeAlignmentLimits> = None;
        for row in self.snapshot().rows() {
            let actual = row
                .scope
                .scope_alignment_limits()
                .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
            let limits = match tightest {
                Some(prior) if prior.structural_work() <= actual.structural_work() => prior,
                _ => actual,
            };
            self.visit(limits, 1)?;
            tightest = Some(limits);
        }
        Ok(tightest)
    }

    pub(super) fn census(&self) -> ScopeCensus {
        match self {
            Self::Empty(_) => ScopeCensus::default(),
            Self::Active(owner) => owner.state.lock().live,
        }
    }
    pub(super) fn lease(&self, census: ScopeCensus) -> surgeist_text::Result<ScopeLease> {
        let Self::Active(owner) = self else {
            return Err(refusal(ScopeAlignmentError::MissingCapture));
        };
        owner.state.retain(census)?;
        Ok(ScopeLease {
            state: Arc::clone(&owner.state),
            census,
        })
    }
    pub(super) fn materialize(
        &self,
        limits: ScopeAlignmentLimits,
        input_records: usize,
        count: usize,
    ) -> surgeist_text::Result<()> {
        let Self::Active(owner) = self else {
            return check(
                ScopeAlignmentLimitKind::Records,
                input_records
                    .checked_add(count)
                    .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
                limits.records(),
            );
        };
        let mut state = owner.state.lock();
        let total = state
            .materialized
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        let required = input_records
            .checked_add(total)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        check(ScopeAlignmentLimitKind::Records, required, limits.records())?;
        state.materialized = total;
        Ok(())
    }
    pub(super) fn visit(
        &self,
        limits: ScopeAlignmentLimits,
        count: usize,
    ) -> surgeist_text::Result<()> {
        let Self::Active(owner) = self else {
            return check(
                ScopeAlignmentLimitKind::StructuralWork,
                count,
                limits.structural_work(),
            );
        };
        let mut state = owner.state.lock();
        let total = state
            .work
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::StructuralWork))?;
        check(
            ScopeAlignmentLimitKind::StructuralWork,
            total,
            limits.structural_work(),
        )?;
        state.work = total;
        Ok(())
    }
    pub(super) fn begin<'a>(
        &'a self,
        input: ScopeAlignmentCensusInput<'a>,
        outputs: usize,
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>> {
        let (prepared, retained) = match input {
            ScopeAlignmentCensusInput::Product(product) => match product {
                ScopeAlignmentProduct::Prepared(value) => {
                    (value.clone(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::Selected(value) => {
                    (value.prepared_text(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::Finalized(value) => {
                    (value.prepared_text(), value.scope_alignment_retention())
                }
            },
            ScopeAlignmentCensusInput::Selection { prepared, .. }
            | ScopeAlignmentCensusInput::Flow { prepared, .. } => {
                (prepared.clone(), prepared.scope_alignment_retention())
            }
        };
        if retained.is_none() {
            return Ok(None);
        }
        let root = prepared
            .captured_owner()
            .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?
            .domain();
        let limits = root
            .scope_alignment_limits()
            .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
        let Self::Active(owner) = self else {
            return Err(refusal(ScopeAlignmentError::MissingCapture));
        };
        let mut registry = owner
            .state
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if registry.loaned {
            return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                scope: root.capture_ordinal(),
            }));
        }
        registry.loaned = true;
        let strong = registry.strong.take().unwrap_or_default();
        // The two initialized slot buffers grow together before any producer
        // may register a result. Upgrading and pruning therefore allocate none.
        if strong.len() != registry.weak.len() {
            registry.loaned = false;
            registry.strong = Some(strong);
            return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                scope: root.capture_ordinal(),
            }));
        }
        drop(registry);
        let mut operation = ScopeOperation {
            state: Arc::clone(&owner.state),
            strong,
            limits,
            prepared,
            remaining_outputs: outputs,
            input: ScopeCensus::default(),
            external: ScopeCensus::default(),
            _borrow: std::marker::PhantomData,
        };
        {
            let mut registry = owner
                .state
                .registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (slot, weak) in operation.strong.iter_mut().zip(&mut registry.weak) {
                self.visit(limits, 1)?;
                *slot = weak.as_ref().and_then(ScopeAlignmentWeakProduct::upgrade);
                if slot.is_none() {
                    *weak = None;
                }
            }
        }
        let census = surgeist_text::scope_alignment_census(
            input,
            ScopeAlignmentProducts::FinalizedSlots(&operation.strong),
            operation.allowance()?,
        )?;
        operation.input = ScopeCensus::from_retention(census.input_retention());
        operation.external = ScopeCensus::from_retention(census.external_retention());
        operation.reserve_outputs(outputs)?;
        Ok(Some(operation))
    }

    pub(crate) fn preparation_buffer<T>(
        &self,
        prepared: &PreparedText,
        scope: &CapturedScope,
        count: usize,
    ) -> surgeist_text::Result<(Vec<T>, ScopeLease)> {
        let limits = scope
            .scope_alignment_limits()
            .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
        let operation = self
            .begin(
                ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(prepared)),
                0,
            )?
            .ok_or_else(|| {
                refusal(ScopeAlignmentError::MissingMetrics {
                    scope: scope.capture_ordinal(),
                })
            })?;
        let retained = operation.input_union()?;
        self.materialize(limits, retained.records, count)?;
        let (values, census) = reserve_buffer::<T>(limits, retained, self.census(), count, 0)?;
        let lease = self.lease(census)?;
        Ok((values, lease))
    }
}

/// A stable, preallocated strong-slot loan. No same-family Layout callback is
/// invoked inside a Text operation. Mutex guards never cross that operation.
pub(crate) struct ScopeOperation<'a> {
    state: Arc<LiveScopeResources>,
    strong: Vec<Option<FinalizedLine>>,
    limits: ScopeAlignmentLimits,
    prepared: PreparedText,
    remaining_outputs: usize,
    input: ScopeCensus,
    external: ScopeCensus,
    _borrow: std::marker::PhantomData<&'a ()>,
}
impl ScopeCensus {
    pub(super) fn from_retention(value: surgeist_text::ScopeAlignmentRetention) -> Self {
        Self {
            records: value.records(),
            bytes: value.payload_bytes(),
        }
    }
}
impl ScopeOperation<'_> {
    pub(crate) fn limits(&self) -> ScopeAlignmentLimits {
        self.limits
    }
    pub(crate) fn input_union(&self) -> surgeist_text::Result<ScopeCensus> {
        self.input.add(self.external)
    }
    pub(crate) fn allowance(
        &self,
    ) -> surgeist_text::Result<surgeist_text::ScopeAlignmentAllowance> {
        let live = self.state.lock().live.add(self.external)?;
        check(
            ScopeAlignmentLimitKind::Records,
            live.records,
            self.limits.records(),
        )?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            live.bytes,
            self.limits.payload_bytes(),
        )?;
        Ok(surgeist_text::ScopeAlignmentAllowance::new(
            self.limits.records() - live.records,
            self.limits.payload_bytes() - live.bytes,
        ))
    }
    fn reserve_outputs(&mut self, outputs: usize) -> surgeist_text::Result<()> {
        if outputs == 0 {
            return Ok(());
        }
        let mut registry = self
            .state
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut live = 0usize;
        for slot in &registry.weak {
            visit_state(&self.state, self.limits, 1)?;
            live += usize::from(slot.is_some());
        }
        let required = live
            .checked_add(outputs)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        if required <= registry.weak.len() {
            return Ok(());
        }
        let union = self.input_union()?;
        grow_slots(
            &self.state,
            self.limits,
            union,
            &mut registry.weak,
            required,
        )?;
        if let Err(error) = grow_slots(&self.state, self.limits, union, &mut self.strong, required)
        {
            let extra = registry.weak.len() - self.strong.len();
            registry.weak.truncate(self.strong.len());
            self.state.release(ScopeCensus {
                records: extra,
                bytes: 0,
            });
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn register(&mut self, line: &FinalizedLine) -> surgeist_text::Result<()> {
        let Some(weak) = line.scope_alignment_weak_product() else {
            return Ok(());
        };
        if line.prepared_text() != self.prepared {
            return Err(refusal(ScopeAlignmentError::ForeignCapture));
        }
        let mut registry = self
            .state
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut free = None;
        for (index, slot) in registry.weak.iter().enumerate() {
            visit_state(&self.state, self.limits, 1)?;
            if slot.as_ref().is_some_and(|prior| prior.same_product(&weak)) {
                return Ok(());
            }
            if slot.is_none() && free.is_none() {
                free = Some(index);
            }
        }
        let scope = line
            .prepared_text()
            .captured_owner()
            .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?
            .domain()
            .capture_ordinal();
        if self.remaining_outputs == 0 {
            return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                scope,
            }));
        }
        let index =
            free.ok_or_else(|| refusal(ScopeAlignmentError::UnrepresentedOwnership { scope }))?;
        self.remaining_outputs -= 1;
        registry.weak[index] = Some(weak);
        self.strong[index] = Some(line.clone());
        Ok(())
    }
}
impl Drop for ScopeOperation<'_> {
    fn drop(&mut self) {
        // Drop strong native products without holding either shared mutex.
        for slot in &mut self.strong {
            *slot = None;
        }
        let mut registry = self
            .state
            .registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registry.strong = Some(std::mem::take(&mut self.strong));
        registry.loaned = false;
    }
}

fn visit_state(
    state: &LiveScopeResources,
    limits: ScopeAlignmentLimits,
    count: usize,
) -> surgeist_text::Result<()> {
    let mut counters = state.lock();
    let next = counters
        .work
        .checked_add(count)
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::StructuralWork))?;
    check(
        ScopeAlignmentLimitKind::StructuralWork,
        next,
        limits.structural_work(),
    )?;
    counters.work = next;
    Ok(())
}

fn grow_slots<T>(
    state: &LiveScopeResources,
    limits: ScopeAlignmentLimits,
    external: ScopeCensus,
    slots: &mut Vec<Option<T>>,
    required: usize,
) -> surgeist_text::Result<()> {
    if slots.len() >= required {
        return Ok(());
    }
    if slots.capacity() >= required {
        let additional = required - slots.len();
        visit_state(state, limits, additional)?;
        let mut counters = state.lock();
        let rows = counters
            .materialized
            .checked_add(additional)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        check(
            ScopeAlignmentLimitKind::Records,
            rows.checked_add(external.records)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
            limits.records(),
        )?;
        let live = counters.live.add(ScopeCensus {
            records: additional,
            bytes: 0,
        })?;
        check(
            ScopeAlignmentLimitKind::Records,
            live.records
                .checked_add(external.records)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
            limits.records(),
        )?;
        counters.materialized = rows;
        counters.live = live;
        drop(counters);
        slots.resize_with(required, || None);
        return Ok(());
    }
    visit_state(
        state,
        limits,
        required
            .checked_add(slots.len())
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::StructuralWork))?,
    )?;
    let current = state.lock().live;
    let bytes = required
        .checked_mul(size_of::<Option<T>>())
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
    let next = current.add(external)?.add(ScopeCensus {
        records: required,
        bytes,
    })?;
    check(
        ScopeAlignmentLimitKind::Records,
        next.records,
        limits.records(),
    )?;
    check(
        ScopeAlignmentLimitKind::PayloadBytes,
        next.bytes,
        limits.payload_bytes(),
    )?;
    {
        let mut counters = state.lock();
        let rows = counters
            .materialized
            .checked_add(required)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        check(
            ScopeAlignmentLimitKind::Records,
            rows.checked_add(external.records)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
            limits.records(),
        )?;
        counters.materialized = rows;
    }
    let mut replacement = Vec::new();
    replacement.try_reserve_exact(required).map_err(|_| {
        refusal(ScopeAlignmentError::Allocation {
            kind: ScopeAlignmentLimitKind::PayloadBytes,
        })
    })?;
    let census = ScopeCensus {
        records: required,
        bytes: replacement
            .capacity()
            .checked_mul(size_of::<Option<T>>())
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
    };
    let actual = current.add(external)?.add(census)?;
    check(
        ScopeAlignmentLimitKind::PayloadBytes,
        actual.bytes,
        limits.payload_bytes(),
    )?;
    replacement.resize_with(required, || None);
    state.retain(census)?;
    for (old, new) in slots.iter_mut().zip(&mut replacement) {
        *new = old.take();
    }
    let old = std::mem::replace(slots, replacement);
    let released = ScopeCensus {
        records: old.len(),
        bytes: old
            .capacity()
            .checked_mul(size_of::<Option<T>>())
            .expect("owned guard capacity was admitted"),
    };
    drop(old);
    state.release(released);
    Ok(())
}

pub(super) fn input_headers<S: LayoutScalar>() -> usize {
    size_of::<ScopeInputOwner<S>>() + size_of::<LiveScopeResources>()
}
