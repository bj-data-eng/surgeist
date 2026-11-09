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

pub(crate) mod semantic;

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
    exhausted: Option<ScopeAlignmentError>,
}

impl ScopeResourceState {
    fn check_exhaustion(&self) -> surgeist_text::Result<()> {
        self.exhausted.map_or(Ok(()), |reason| Err(refusal(reason)))
    }
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
        state.check_exhaustion()?;
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

/// One computation/session family's live feature custody. The opt-in entry
/// view owns strong proofs and Layout leases. The live counter state does not
/// own that entry store, so these leases cannot form a strong ownership cycle.
#[derive(Clone, Debug)]
pub(crate) enum ScopeAlignmentResources<S: LayoutScalar> {
    Empty(
        Arc<ScopeAlignmentSnapshot<S>>,
        super::super::fragmentation::InlineBoxFamilies,
    ),
    Active(
        Arc<ScopeInputOwner<S>>,
        super::super::fragmentation::InlineBoxFamilies,
    ),
    Entry(Arc<semantic::EntryScopeResources<S>>),
}
impl<S: LayoutScalar> ScopeAlignmentResources<S> {
    pub(crate) fn new(
        snapshot: Arc<ScopeAlignmentSnapshot<S>>,
        inline_boxes: super::super::fragmentation::InlineBoxFamilies,
    ) -> Self {
        if snapshot.retention.bytes == 0 {
            return Self::Empty(snapshot, inline_boxes);
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
                exhausted: None,
            }),
            registry: Mutex::new(Registry::default()),
        });
        Self::Active(
            Arc::new(ScopeInputOwner {
                snapshot,
                state,
                census,
            }),
            inline_boxes,
        )
    }
    pub(crate) fn snapshot(&self) -> &Arc<ScopeAlignmentSnapshot<S>> {
        match self {
            Self::Empty(snapshot, _) => snapshot,
            Self::Active(owner, _) => &owner.snapshot,
            Self::Entry(entry) => entry.base.snapshot(),
        }
    }
    fn inline_boxes(&self) -> &super::super::fragmentation::InlineBoxFamilies {
        match self {
            Self::Empty(_, inline_boxes) | Self::Active(_, inline_boxes) => inline_boxes,
            Self::Entry(entry) => entry.base.inline_boxes(),
        }
    }
    fn active_owner(&self) -> Option<&Arc<ScopeInputOwner<S>>> {
        match self {
            Self::Empty(..) => None,
            Self::Active(owner, _) => Some(owner),
            Self::Entry(entry) => entry.base.active_owner(),
        }
    }
    fn semantic_store(&self) -> Option<&Arc<semantic::SemanticStore>> {
        match self {
            Self::Entry(entry) => Some(&entry.store),
            _ => None,
        }
    }
    fn semantic_credit(&self) -> usize {
        self.semantic_store().map_or(0, |store| store.credit())
    }
    /// Discover the actual constraining capture on the stack. Each normalized
    /// descriptor visit is admitted before continuing; no policy summary is
    /// retained beside the immutable input facts.
    pub(crate) fn key_limits(&self) -> surgeist_text::Result<Option<ScopeAlignmentLimits>> {
        let mut tightest = self
            .inline_boxes()
            .policy_scope()
            .and_then(CapturedScope::scope_alignment_limits);
        if let Some(limits) = tightest {
            self.visit(limits, 1)?;
        }
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
        if let Some(initial) = tightest {
            let families = self.inline_boxes().slots();
            self.visit(initial, families.len())?;
            for family in families.iter().flatten() {
                if let Some(actual) = family
                    .prepared_text()
                    .captured_owner()
                    .and_then(|owner| owner.domain().scope_alignment_limits())
                    && actual.structural_work() < tightest.expect("known ceiling").structural_work()
                {
                    self.visit(actual, 0)?;
                    tightest = Some(actual);
                }
            }
        }
        Ok(tightest)
    }

    pub(super) fn census(&self) -> ScopeCensus {
        match self {
            Self::Empty(_, _) => ScopeCensus::default(),
            Self::Active(owner, _) => owner.state.lock().live,
            Self::Entry(entry) => entry.base.census(),
        }
    }
    pub(super) fn lease(&self, census: ScopeCensus) -> surgeist_text::Result<ScopeLease> {
        let Some(owner) = self.active_owner() else {
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
        let Some(owner) = self.active_owner() else {
            return check(
                ScopeAlignmentLimitKind::Records,
                input_records
                    .checked_add(count)
                    .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?,
                limits.records(),
            );
        };
        let mut state = owner.state.lock();
        state.check_exhaustion()?;
        let total = state
            .materialized
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        // Only entry-owned Text rows already admitted as materialization may
        // overlap the exact borrowed union. Ordinary finalized roots get none.
        let input_records = input_records.saturating_sub(self.semantic_credit());
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
        let Some(owner) = self.active_owner() else {
            return check(
                ScopeAlignmentLimitKind::StructuralWork,
                count,
                limits.structural_work(),
            );
        };
        let mut state = owner.state.lock();
        state.check_exhaustion()?;
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

    /// Census work has already executed. Record both dimensions before
    /// checking ceilings; a late refusal cannot refund those completed costs.
    fn record_census(
        &self,
        limits: ScopeAlignmentLimits,
        accounting: surgeist_text::ScopeAlignmentCensusAccounting,
        request_credit: usize,
    ) -> Result<(), ScopeAlignmentError> {
        let usage = accounting.usage();
        let new_records = accounting.newly_materialized_records();
        let borrowed = usage
            .records()
            .checked_sub(new_records)
            .expect("Text census accounting separates authenticated seeded rows");
        let credit = if borrowed == 0 {
            0
        } else {
            self.semantic_credit()
                .checked_add(request_credit)
                .expect("admitted entry credit arithmetic")
        };
        let borrowed = borrowed
            .checked_sub(credit)
            .expect("entry credits name distinct rows in the seeded complete union");
        self.record_completed(limits, usage, new_records, borrowed)
    }

    fn record_completed(
        &self,
        limits: ScopeAlignmentLimits,
        usage: surgeist_text::ScopeAlignmentUsage,
        new_records: usize,
        borrowed: usize,
    ) -> Result<(), ScopeAlignmentError> {
        let Some(owner) = self.active_owner() else {
            return Err(ScopeAlignmentError::MissingCapture);
        };
        let mut state = owner.state.lock();
        let mut failure = state.exhausted;
        let mut add = |value: &mut usize, count: usize, kind| {
            if let Some(total) = value.checked_add(count) {
                *value = total;
            } else {
                failure.get_or_insert(ScopeAlignmentError::Overflow { kind });
            }
        };
        add(
            &mut state.work,
            usage.structural_work(),
            ScopeAlignmentLimitKind::StructuralWork,
        );
        add(
            &mut state.materialized,
            new_records,
            ScopeAlignmentLimitKind::Records,
        );
        if state.work > limits.structural_work() {
            failure.get_or_insert(ScopeAlignmentError::Limit {
                kind: ScopeAlignmentLimitKind::StructuralWork,
                required: state.work,
                limit: limits.structural_work(),
            });
        }
        match state.materialized.checked_add(borrowed) {
            Some(required) if required > limits.records() => {
                failure.get_or_insert(ScopeAlignmentError::Limit {
                    kind: ScopeAlignmentLimitKind::Records,
                    required,
                    limit: limits.records(),
                });
            }
            None => {
                failure.get_or_insert(ScopeAlignmentError::Overflow {
                    kind: ScopeAlignmentLimitKind::Records,
                });
            }
            _ => {}
        }
        state.exhausted = failure;
        state.exhausted.map_or(Ok(()), Err)
    }
    pub(super) fn begin<'a>(
        &'a self,
        input: ScopeAlignmentCensusInput<'a>,
        outputs: usize,
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>> {
        self.begin_with_families(input, outputs, self.inline_boxes().slots(), 0)
    }

    /// Constructor input moves into Text after census; the operation owns the
    /// policy loan and pins, not a borrow of that transferred Vec. Provisional
    /// earlier slots stay alive in the constructor through this child call.
    pub(crate) fn begin_inline_box_family<'a>(
        &'a self,
        prepared: &PreparedText,
        native_boxes: &Vec<surgeist_text::InlineBoxSource>,
        families: &[Option<surgeist_text::PreparedInlineBoxFamily>],
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>> {
        self.begin_with_families(
            ScopeAlignmentCensusInput::InlineBoxFamily {
                prepared,
                native_boxes,
            },
            0,
            families,
            0,
        )
    }

    fn begin_with_families<'a>(
        &'a self,
        input: ScopeAlignmentCensusInput<'_>,
        outputs: usize,
        families: &[Option<surgeist_text::PreparedInlineBoxFamily>],
        request_credit: usize,
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>> {
        let (prepared, retained) = match input {
            ScopeAlignmentCensusInput::Product(product) => match product {
                ScopeAlignmentProduct::Prepared(value) => {
                    (value.clone(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::InlineBoxFamily(value) => {
                    (value.prepared_text(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::Selected(value) => {
                    (value.prepared_text(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::Finalized(value) => {
                    (value.prepared_text(), value.scope_alignment_retention())
                }
                ScopeAlignmentProduct::SemanticWindow(value) => {
                    let prepared = value.request().segment.prepared_text();
                    let retained = prepared.scope_alignment_retention();
                    (prepared, retained)
                }
            },
            ScopeAlignmentCensusInput::InlineBoxFamily { prepared, .. }
            | ScopeAlignmentCensusInput::Selection { prepared, .. }
            | ScopeAlignmentCensusInput::SemanticWindow { prepared, .. }
            | ScopeAlignmentCensusInput::Flow { prepared, .. } => {
                (prepared.clone(), prepared.scope_alignment_retention())
            }
        };
        // Captured ceilings already materialize retained Prepared input. A
        // no-limit family remains Native; there is no family-only activation.
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
        let Some(owner) = self.active_owner() else {
            return Err(refusal(ScopeAlignmentError::MissingCapture));
        };
        owner.state.lock().check_exhaustion()?;
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
            semantic: None,
            _borrow: std::marker::PhantomData,
        };
        operation.semantic = self
            .semantic_store()
            .map(semantic::SemanticLoan::take)
            .transpose()?;
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
        let census = operation.census(input, families)?;
        let census = match census {
            Ok(census) => {
                self.record_census(limits, census.accounting(), request_credit)
                    .map_err(refusal)?;
                census
            }
            Err(failure) => {
                let (error, accounting) = failure.into_parts();
                if let Some(accounting) = accounting {
                    // Exhaustion remains recorded in the owner. The original
                    // census cause has priority over this later admission.
                    let _ = self.record_census(limits, accounting, request_credit);
                }
                return Err(error);
            }
        };
        operation.input = ScopeCensus::from_retention(census.input_retention());
        operation.external = ScopeCensus::from_retention(census.external_retention());
        operation.reserve_outputs(outputs)?;
        operation.replenish_descriptors(self, families.len())?;
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
    semantic: Option<semantic::SemanticLoan>,
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
            self.semantic.as_ref().map_or(0, |loan| loan.credit()),
            &mut registry.weak,
            required,
        )?;
        if let Err(error) = grow_slots(
            &self.state,
            self.limits,
            union,
            self.semantic.as_ref().map_or(0, |loan| loan.credit()),
            &mut self.strong,
            required,
        ) {
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
    counters.check_exhaustion()?;
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
    history_credit: usize,
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
            rows.checked_add(external.records.saturating_sub(history_credit))
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
            rows.checked_add(external.records.saturating_sub(history_credit))
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
