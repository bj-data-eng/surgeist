//! Entry-local strong semantic roots and their exact Text resource boundary.
use super::*;
use crate::text_layout::scope_alignment::allocation_buffer;
use surgeist_text::{
    InlineBoundaryContextAllocation, SemanticAtomicAllocation, SemanticControlAllocation,
    SemanticWindowEvidence, SemanticWindowRequest,
};

/// This opt-in owner leaves ordinary scope-resource layouts and quotas alone.
#[derive(Debug)]
pub(crate) struct EntryScopeResources<S: LayoutScalar> {
    pub(super) base: ScopeAlignmentResources<S>,
    pub(super) store: Arc<SemanticStore>,
    _lease: Option<ScopeLease>,
}

#[derive(Debug)]
pub(super) struct SemanticStore {
    registry: Mutex<SemanticRegistry>,
    anchor: Option<PreparedText>,
    _lease: Option<ScopeLease>,
}

#[derive(Debug, Default)]
struct SemanticRegistry {
    roots: Option<ProofRows>,
    descriptors: Option<Descriptors>,
    credit: usize,
    loaned: bool,
}

#[derive(Debug, Default)]
struct ProofRows {
    values: Vec<SemanticWindowEvidence>,
    slots: usize,
    _lease: Option<ScopeLease>,
}

/// Only EMPTY buffers have this lifetime. A filled loan is always discarded.
#[derive(Debug)]
struct Descriptors {
    values: Vec<ScopeAlignmentProduct<'static>>,
    slots: usize,
    _lease: Option<ScopeLease>,
}

impl SemanticStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, SemanticRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    pub(super) fn credit(&self) -> usize {
        self.lock().credit
    }
}

/// Text's evidence owner carries identity; this handle also pins the entry
/// store and its Layout leases after the computation's view is released.
#[derive(Clone, Debug)]
pub(crate) struct SemanticWindowProof {
    _store: Arc<SemanticStore>,
    evidence: SemanticWindowEvidence,
}
impl SemanticWindowProof {
    pub(crate) fn evidence(&self) -> &SemanticWindowEvidence {
        &self.evidence
    }
}

/// The three actual request buffers are funded before a caller fills them.
/// Their live leases are surrendered before Text seeds those same allocations.
pub(crate) struct FundedSemanticRequest {
    request: SemanticWindowRequest,
    counts: [usize; 3],
    leases: [Option<ScopeLease>; 3],
    store: Arc<SemanticStore>,
    limits: Option<ScopeAlignmentLimits>,
}
impl FundedSemanticRequest {
    pub(crate) fn push_atomic(
        &mut self,
        value: SemanticAtomicAllocation,
    ) -> surgeist_text::Result<()> {
        self.check_room(self.request.atomics.len(), 0)?;
        self.request.atomics.push(value);
        Ok(())
    }
    pub(crate) fn push_control(
        &mut self,
        value: SemanticControlAllocation,
    ) -> surgeist_text::Result<()> {
        self.check_room(self.request.controls.len(), 1)?;
        self.request.controls.push(value);
        Ok(())
    }
    pub(crate) fn push_boundary_context(
        &mut self,
        value: InlineBoundaryContextAllocation,
    ) -> surgeist_text::Result<()> {
        self.check_room(self.request.boundary_context.len(), 2)?;
        self.request.boundary_context.push(value);
        Ok(())
    }
    fn check_room(&self, len: usize, kind: usize) -> surgeist_text::Result<()> {
        if len < self.counts[kind] {
            if let (Some(limits), Some(lease)) = (self.limits, &self.leases[0]) {
                visit_state(&lease.state, limits, 1)?;
            }
            Ok(())
        } else {
            Err(unrepresented(&self.request.segment.prepared_text()))
        }
    }
}

pub(super) struct SemanticLoan {
    store: Arc<SemanticStore>,
    roots: ProofRows,
    descriptors: Option<Descriptors>,
}
impl SemanticLoan {
    pub(super) fn credit(&self) -> usize {
        self.store.credit()
    }
    pub(super) fn take(store: &Arc<SemanticStore>) -> surgeist_text::Result<Self> {
        let mut registry = store.lock();
        if registry.loaned {
            return Err(refusal(ScopeAlignmentError::MissingCapture));
        }
        registry.loaned = true;
        Ok(Self {
            store: Arc::clone(store),
            roots: registry
                .roots
                .take()
                .expect("entry root rows are owned or loaned"),
            descriptors: registry.descriptors.take(),
        })
    }
}
impl Drop for SemanticLoan {
    fn drop(&mut self) {
        let mut registry = self.store.lock();
        registry.roots = Some(std::mem::take(&mut self.roots));
        registry.descriptors = self.descriptors.take();
        registry.loaned = false;
    }
}

fn unrepresented(prepared: &PreparedText) -> surgeist_text::Error {
    refusal(ScopeAlignmentError::UnrepresentedOwnership {
        scope: prepared
            .captured_owner()
            .map_or(0, |owner| owner.domain().capture_ordinal()),
    })
}

fn native_buffer<T>(count: usize) -> surgeist_text::Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| {
        refusal(ScopeAlignmentError::Allocation {
            kind: ScopeAlignmentLimitKind::PayloadBytes,
        })
    })?;
    Ok(values)
}

impl<S: LayoutScalar> ScopeAlignmentResources<S> {
    /// Install before cloning entry trials or independent body fitters.
    pub(crate) fn entry_view(&self, prepared: &PreparedText) -> surgeist_text::Result<Self> {
        if matches!(self, Self::Entry(_)) {
            return Ok(self.clone());
        }
        if self.active_owner().is_some() && prepared.scope_alignment_retention().is_none() {
            return Err(refusal(ScopeAlignmentError::MissingCapture));
        }
        let operation = self.begin(
            ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(prepared)),
            0,
        )?;
        let (roots, descriptors, lease, store_lease) = if let Some(operation) = &operation {
            self.visit(operation.limits(), 4)?;
            let count = operation
                .strong
                .len()
                .checked_add(self.inline_boxes().slots().len())
                .and_then(|n| n.checked_add(2))
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
            let (roots, roots_lease) = allocation_buffer(self, operation, 1)?;
            let (values, descriptor_lease) = allocation_buffer(self, operation, count)?;
            let lease = crate::text_layout::scope_alignment::allocation_header::<
                EntryScopeResources<S>,
            >(self, operation)?;
            let store_lease = crate::text_layout::scope_alignment::allocation_header::<
                SemanticStore,
            >(self, operation)?;
            (
                ProofRows {
                    values: roots,
                    slots: 1,
                    _lease: Some(roots_lease),
                },
                Descriptors {
                    values,
                    slots: count,
                    _lease: Some(descriptor_lease),
                },
                Some(lease),
                Some(store_lease),
            )
        } else {
            (
                ProofRows {
                    values: native_buffer(1)?,
                    slots: 1,
                    _lease: None,
                },
                Descriptors {
                    values: Vec::new(),
                    slots: 0,
                    _lease: None,
                },
                None,
                None,
            )
        };
        let anchor = operation.as_ref().map(|_| prepared.clone());
        drop(operation);
        Ok(Self::Entry(Arc::new(EntryScopeResources {
            base: self.clone(),
            store: Arc::new(SemanticStore {
                anchor,
                _lease: store_lease,
                registry: Mutex::new(SemanticRegistry {
                    roots: Some(roots),
                    descriptors: Some(descriptors),
                    ..Default::default()
                }),
            }),
            _lease: lease,
        })))
    }

    pub(crate) fn fund_semantic_request(
        &self,
        mut template: SemanticWindowRequest,
        atomic_count: usize,
        control_count: usize,
        boundary_count: usize,
    ) -> surgeist_text::Result<FundedSemanticRequest> {
        let store = self
            .semantic_store()
            .ok_or_else(|| unrepresented(&template.segment.prepared_text()))?;
        if template.atomics.capacity() != 0
            || template.controls.capacity() != 0
            || template.boundary_context.capacity() != 0
        {
            return Err(unrepresented(&template.segment.prepared_text()));
        }
        let request_prepared = template.segment.prepared_text();
        let prepared = store.anchor.as_ref().unwrap_or(&request_prepared);
        let mut operation = self.begin(
            ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(prepared)),
            0,
        )?;
        let leases = if let Some(operation) = &mut operation {
            self.visit(operation.limits(), 3)?;
            operation.reserve_semantic_root(self)?;
            let (atomics, a) = allocation_buffer(self, operation, atomic_count)?;
            let (controls, c) = allocation_buffer(self, operation, control_count)?;
            let (context, b) = allocation_buffer(self, operation, boundary_count)?;
            template.atomics = atomics;
            template.controls = controls;
            template.boundary_context = context;
            [Some(a), Some(c), Some(b)]
        } else {
            let mut registry = store.lock();
            let roots = registry
                .roots
                .as_mut()
                .ok_or_else(|| unrepresented(prepared))?;
            if roots.values.len() == roots.slots {
                roots.values.try_reserve_exact(1).map_err(|_| {
                    refusal(ScopeAlignmentError::Allocation {
                        kind: ScopeAlignmentLimitKind::PayloadBytes,
                    })
                })?;
                roots.slots = roots
                    .values
                    .len()
                    .checked_add(1)
                    .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
            }
            template.atomics = native_buffer(atomic_count)?;
            template.controls = native_buffer(control_count)?;
            template.boundary_context = native_buffer(boundary_count)?;
            [None, None, None]
        };
        let limits = operation.as_ref().map(ScopeOperation::limits);
        Ok(FundedSemanticRequest {
            request: template,
            counts: [atomic_count, control_count, boundary_count],
            leases,
            store: Arc::clone(store),
            limits,
        })
    }

    pub(crate) fn observe_semantic(
        &self,
        funded: FundedSemanticRequest,
    ) -> surgeist_text::Result<SemanticWindowProof> {
        let FundedSemanticRequest {
            request,
            leases,
            store,
            ..
        } = funded;
        if !self
            .semantic_store()
            .is_some_and(|actual| Arc::ptr_eq(actual, &store))
        {
            return Err(unrepresented(&request.segment.prepared_text()));
        }
        let request_rows = request
            .atomics
            .len()
            .checked_add(request.controls.len())
            .and_then(|n| n.checked_add(request.boundary_context.len()))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        // Exactly these buffers now become borrowed Text input. Their completed
        // construction remains materialization; their duplicate live loan ends.
        drop(leases);
        let prepared = request.segment.prepared_text();
        let operation = self.begin_with_families(
            ScopeAlignmentCensusInput::SemanticWindow {
                prepared: &prepared,
                request: &request,
            },
            0,
            self.inline_boxes().slots(),
            request_rows,
        )?;
        let result = match &operation {
            Some(operation) => {
                prepared.semantic_window_with_scope_allowance(&request, operation.allowance()?)
            }
            None => prepared.semantic_window(&request),
        };
        let evidence = self.admit_semantic(operation.as_ref(), request_rows, result)?;
        drop(request);
        // This is a Layout store write after the producer has completed. Debit
        // the producer first, then admit the actual write before publishing a
        // handle. A Native producer remains inactive even when this shared Layout
        // store has an actual captured funding anchor.
        let insertion_limits = operation.as_ref().map(ScopeOperation::limits).or_else(|| {
            store
                .anchor
                .as_ref()?
                .captured_owner()?
                .domain()
                .scope_alignment_limits()
        });
        if let Some(limits) = insertion_limits {
            self.visit(limits, 1)?;
        }
        if let Some(mut operation) = operation {
            let loan = operation
                .semantic
                .as_mut()
                .expect("entry operation retains its semantic loan");
            if loan.roots.values.len() == loan.roots.slots {
                return Err(unrepresented(&prepared));
            }
            let records = evidence.scope_alignment_retention().records();
            let mut registry = store.lock();
            registry.credit = registry
                .credit
                .checked_add(records)
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
            loan.roots.values.push(evidence.clone());
            drop(registry);
            drop(operation);
            // The newly retained product is now an exact Text root. This fresh
            // census funds the next descriptor buffer against its complete union.
            drop(self.begin(
                ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::SemanticWindow(
                    &evidence,
                )),
                0,
            )?);
        } else {
            let mut registry = store.lock();
            let roots = registry
                .roots
                .as_mut()
                .expect("native entry roots are not loaned");
            if roots.values.len() == roots.slots {
                return Err(unrepresented(&prepared));
            }
            roots.values.push(evidence.clone());
        }
        Ok(SemanticWindowProof {
            _store: store,
            evidence,
        })
    }
    /// Debit a completed producer transaction before returning its result. The
    /// exact preceding input census authenticates request-buffer seed credit.
    fn admit_semantic(
        &self,
        operation: Option<&ScopeOperation<'_>>,
        request_rows: usize,
        result: Result<SemanticWindowEvidence, surgeist_text::SemanticWindowFailure>,
    ) -> surgeist_text::Result<SemanticWindowEvidence> {
        match result {
            Ok(evidence) => {
                if let (Some(operation), Some(usage)) =
                    (operation, evidence.scope_alignment_usage())
                {
                    let input = evidence.input_retention().records();
                    self.record_completed(
                        operation.limits(),
                        usage,
                        usage
                            .records()
                            .checked_sub(input)
                            .expect("semantic success retains seeded input"),
                        input
                            .checked_sub(request_rows)
                            .expect("request credit is in exact input"),
                    )
                    .map_err(refusal)?;
                }
                Ok(evidence)
            }
            Err(failure) => {
                let (error, usage, admitted) = failure.into_parts();
                if let (Some(operation), Some(usage)) = (operation, usage) {
                    // Exact-R begin + immediate allowance guarantees both seeds.
                    // Keep the admitted-input distinction explicit: no partial
                    // union or guessed complete input is published on failure.
                    let input = admitted.records();
                    let seeded = if input != 0 || admitted.payload_bytes() != 0 {
                        debug_assert_eq!(ScopeCensus::from_retention(admitted), operation.input);
                        input
                    } else if usage.records() == 0 {
                        0
                    } else {
                        // The input prepass allocates no rows. Without admitted
                        // bytes, positive records are exactly the successful
                        // record seed before byte-seed refusal, not new output.
                        assert_eq!(usage.records(), operation.input.records);
                        operation.input.records
                    };
                    let request_credit = if seeded == 0 { 0 } else { request_rows };
                    let _ = self.record_completed(
                        operation.limits(),
                        usage,
                        usage
                            .records()
                            .checked_sub(seeded)
                            .expect("semantic seeded rows precede new materialization"),
                        seeded
                            .checked_sub(request_credit)
                            .expect("funded request credit belongs to seeded input"),
                    );
                }
                Err(error)
            }
        }
    }
}

impl ScopeOperation<'_> {
    pub(super) fn census(
        &mut self,
        input: ScopeAlignmentCensusInput<'_>,
        families: &[Option<surgeist_text::PreparedInlineBoxFamily>],
    ) -> surgeist_text::Result<
        std::result::Result<
            surgeist_text::ScopeAlignmentCensus,
            surgeist_text::ScopeAlignmentCensusFailure,
        >,
    > {
        let allowance = self.allowance()?;
        let Some(loan) = &mut self.semantic else {
            return Ok(surgeist_text::scope_alignment_census(
                input,
                ScopeAlignmentProducts::FinalizedAndInlineBoxFamilies {
                    finalized: &self.strong,
                    families,
                },
                allowance,
            ));
        };
        let descriptors = loan
            .descriptors
            .take()
            .ok_or_else(|| unrepresented(&self.prepared))?;
        let Descriptors {
            values,
            slots,
            _lease: lease,
        } = descriptors;
        let mut values: Vec<ScopeAlignmentProduct<'_>> = values;
        let count = loan
            .roots
            .values
            .len()
            .checked_add(self.strong.len())
            .and_then(|n| n.checked_add(families.len()))
            .and_then(|n| n.checked_add(usize::from(loan.store.anchor.is_some())))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        if count > slots {
            return Err(unrepresented(&self.prepared));
        }
        visit_state(&self.state, self.limits, count)?;
        if let Some(anchor) = &loan.store.anchor {
            values.push(ScopeAlignmentProduct::Prepared(anchor));
        }
        for root in &loan.roots.values {
            values.push(ScopeAlignmentProduct::SemanticWindow(root));
        }
        for line in self.strong.iter().flatten() {
            values.push(ScopeAlignmentProduct::Finalized(line));
        }
        for family in families.iter().flatten() {
            values.push(ScopeAlignmentProduct::InlineBoxFamily(family));
        }
        let result = surgeist_text::scope_alignment_census(
            input,
            ScopeAlignmentProducts::Mixed(&values),
            allowance,
        );
        drop(values);
        drop(lease);
        Ok(result)
    }

    pub(super) fn replenish_descriptors<S: LayoutScalar>(
        &mut self,
        resources: &ScopeAlignmentResources<S>,
        families: usize,
    ) -> surgeist_text::Result<()> {
        let Some(loan) = &self.semantic else {
            return Ok(());
        };
        let count = loan
            .roots
            .slots
            .checked_add(self.strong.len())
            .and_then(|n| n.checked_add(families))
            .and_then(|n| n.checked_add(usize::from(loan.store.anchor.is_some())))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        // The previous borrowed descriptor Vec has been discarded. Existing
        // NEXT backing, if any, remains leased until its replacement is funded.
        let (values, lease) = allocation_buffer(resources, self, count)?;
        self.semantic
            .as_mut()
            .expect("known semantic loan")
            .descriptors = Some(Descriptors {
            values,
            slots: count,
            _lease: Some(lease),
        });
        Ok(())
    }

    fn reserve_semantic_root<S: LayoutScalar>(
        &mut self,
        resources: &ScopeAlignmentResources<S>,
    ) -> surgeist_text::Result<()> {
        let loan = self
            .semantic
            .as_ref()
            .expect("funded entry operation has root custody");
        resources.visit(self.limits, 1)?;
        if loan.roots.values.len() < loan.roots.slots {
            return Ok(());
        }
        let count = loan
            .roots
            .values
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        resources.visit(self.limits, count)?;
        let (mut values, lease) = allocation_buffer(resources, self, count)?;
        let loan = self.semantic.as_mut().expect("known semantic loan");
        values.append(&mut loan.roots.values);
        loan.roots = ProofRows {
            values,
            slots: count,
            _lease: Some(lease),
        };
        self.replenish_descriptors(resources, resources.inline_boxes().slots().len())
    }
}

#[cfg(test)]
mod tests;
