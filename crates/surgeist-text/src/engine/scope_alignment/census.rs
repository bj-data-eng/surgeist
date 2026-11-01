//! Owner-backed feature payload census; allocation identities never leave Text.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AllocationIdentity {
    address: usize,
    extent: usize,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct AllocationWitness {
    identity: AllocationIdentity,
    records: usize,
    payload_bytes: usize,
}
pub(crate) type AllocationVisitor<'a> = dyn FnMut(AllocationWitness) -> Result<()> + 'a;

impl AllocationWitness {
    pub(crate) fn object<T>(value: &T, records: usize) -> Self {
        let bytes = std::mem::size_of::<T>();
        Self {
            identity: AllocationIdentity {
                address: std::ptr::from_ref(value) as usize,
                extent: bytes,
            },
            records,
            payload_bytes: bytes,
        }
    }
    pub(crate) fn buffer<T>(values: &Vec<T>, records: usize) -> Result<Self> {
        let bytes = capacity_bytes(values)?;
        // Zero-sized rows have no backing allocation; their actual owner
        // header distinguishes logically distinct buffers when needed.
        let address = if bytes == 0 {
            std::ptr::from_ref(values) as usize
        } else {
            values.as_ptr() as usize
        };
        Ok(Self {
            identity: AllocationIdentity {
                address,
                extent: bytes,
            },
            records,
            payload_bytes: bytes,
        })
    }
    pub(crate) fn resource(bytes: &[u8]) -> Self {
        Self {
            identity: AllocationIdentity {
                address: bytes.as_ptr() as usize,
                extent: bytes.len(),
            },
            records: 0,
            payload_bytes: bytes.len(),
        }
    }
    pub(crate) const fn identity(self) -> AllocationIdentity {
        self.identity
    }
    pub(crate) const fn records(self) -> usize {
        self.records
    }
    pub(crate) const fn payload_bytes(self) -> usize {
        self.payload_bytes
    }
}

struct AllocationSet {
    seen: Vec<AllocationWitness>,
    retention: crate::ScopeAlignmentRetention,
}
impl AllocationSet {
    fn new() -> Self {
        Self {
            seen: Vec::new(),
            retention: crate::ScopeAlignmentRetention::default(),
        }
    }
    fn observe(
        &mut self,
        witness: AllocationWitness,
        budget: &mut ScopeAlignmentBudget,
    ) -> Result<bool> {
        if witness.records == 0 && witness.payload_bytes == 0 {
            return Ok(false);
        }
        let inspection = checked_sum(
            crate::ScopeAlignmentLimitKind::StructuralWork,
            witness.records,
            1,
        )?;
        budget.work(checked_sum(
            crate::ScopeAlignmentLimitKind::StructuralWork,
            self.seen.len(),
            inspection,
        )?)?;
        if self.seen.iter().any(|old| old.identity == witness.identity) {
            return Ok(false);
        }
        push(budget, &mut self.seen, witness)?;
        self.retention.records = checked_sum(
            crate::ScopeAlignmentLimitKind::Records,
            self.retention.records,
            witness.records,
        )?;
        self.retention.payload_bytes = checked_sum(
            crate::ScopeAlignmentLimitKind::PayloadBytes,
            self.retention.payload_bytes,
            witness.payload_bytes,
        )?;
        Ok(true)
    }
    fn finish(self, budget: &mut ScopeAlignmentBudget) -> Result<crate::ScopeAlignmentRetention> {
        let Self { seen, retention } = self;
        let bytes = capacity_bytes(&seen)?;
        drop(seen);
        budget.release_bytes(bytes)?;
        Ok(retention)
    }
}

pub(crate) fn retained_prepared(
    data: &PreparedScopes,
    analyzed: &AnalyzedData,
    shaped: &super::super::layout::data::ShapedData,
    tables: &crate::protocol::prepared::PreparedTables,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    let mut set = AllocationSet::new();
    preparation::visit_prepared_allocations(data, analyzed, shaped, tables, &mut |allocation| {
        set.observe(allocation, budget).map(|_| ())
    })?;
    set.finish(budget)
}

/// Emit the actual selected feature owner and its retained preparation. Body
/// font blobs are pinned by selection itself, before any finalizer can run.
fn visit_selected_own_allocations(
    prepared: &PreparedData,
    scopes: &SelectedScopes,
    basis: &crate::engine::SelectedShape,
    policy: crate::SpacingPolicy,
    slots: &Vec<crate::protocol::selection::SpacingSlotData>,
    request: &crate::protocol::LineSelectionRequest,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    crate::protocol::inline_box::visit_request_allocations(request, visit)?;
    let shaped = &basis.shaped;
    let characters = &basis.characters;
    let order = &basis.order;
    if crate::protocol::selection::spacing::funded(shaped, policy) {
        crate::protocol::selection::spacing::visit_allocations(slots, visit)?;
        crate::protocol::selection::spacing::visit_word_allocations(shaped, visit)?;
    }
    preparation::visit_prepared_allocations(
        &prepared.scope_alignment,
        &prepared.analyzed,
        &prepared.shaped,
        &prepared.tables,
        visit,
    )?;
    visit(AllocationWitness::object(scopes, 0))?;
    visit(AllocationWitness::buffer(&scopes.rows, scopes.rows.len())?)?;
    visit(AllocationWitness::buffer(
        &scopes.contributions,
        scopes.contributions.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.cluster_owners,
        scopes.cluster_owners.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.glyph_owners,
        scopes.glyph_owners.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.unit_owners,
        scopes.unit_owners.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.strut_owners,
        scopes.strut_owners.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.shared,
        scopes.shared.len(),
    )?)?;
    visit(AllocationWitness::buffer(
        &scopes.uncertain,
        scopes.uncertain.len(),
    )?)?;
    // Its fixed Vec header is already included in the SelectedScopes object.
    visit(AllocationWitness::buffer(
        &scopes.proof_frames,
        scopes.proof_frames.len(),
    )?)?;
    visit(AllocationWitness::object(&shaped.scope_glyph_custody, 0))?;
    visit(AllocationWitness::buffer(
        &shaped.scope_glyph_custody,
        shaped.scope_glyph_custody.len(),
    )?)?;
    visit(AllocationWitness::object(&shaped.scope_relations, 0))?;
    visit(AllocationWitness::buffer(
        &shaped.scope_relations,
        shaped.scope_relations.len(),
    )?)?;
    preparation::visit_cell_allocations(shaped, visit)?;
    crate::engine::spacing::character::visit_shaped_allocations(shaped, visit)?;
    characters.visit_allocations(visit)?;
    if !shaped.cells.is_empty() {
        // This is the actual retained selected result, even when zero tracking
        // leaves no retained character Graph. Its header is embedded in the
        // selected heap owner, not in a temporary ordering call's stack frame.
        visit(AllocationWitness::object(order, 0))?;
        visit(AllocationWitness::buffer(order, order.len())?)?;
        accounting::visit_cell_arenas(shaped, visit)?;
    }
    if shaped.cell_arenas == crate::engine::layout::data::CellArenaOwnership::Owned {
        accounting::visit_cell_exclusions(shaped, visit)?;
    }
    for run in shaped.runs.iter().chain(shaped.empty_run.iter()) {
        visit(AllocationWitness::resource(run.font.bytes()))?;
    }
    Ok(())
}

fn visit_finalized_own_allocations(
    frame: &Arc<crate::engine::FinalizedLine>,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    if frame.data.scope_retention.is_none() {
        return Ok(());
    }
    let selected = &frame.selected;
    visit_selected_own_allocations(
        &selected.prepared,
        &selected.scopes,
        &selected.basis,
        selected.request.context.spacing,
        &selected.spacing_slots,
        &selected.request,
        visit,
    )?;
    frame.visit_scope_allocations(visit)
}

/// The stored closure is already unique and transitive. Emit each actual
/// older frame's own parts only; never recurse through its older proof edges.
fn visit_selected_allocations(
    prepared: &PreparedData,
    scopes: &SelectedScopes,
    basis: &crate::engine::SelectedShape,
    policy: crate::SpacingPolicy,
    slots: &Vec<crate::protocol::selection::SpacingSlotData>,
    request: &crate::protocol::LineSelectionRequest,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    visit_selected_own_allocations(prepared, scopes, basis, policy, slots, request, visit)?;
    for frame in &scopes.proof_frames {
        visit_finalized_own_allocations(&frame.inner, visit)?;
    }
    Ok(())
}

/// Test a prospective body pin against the actual already closed proof input.
/// Use the same current own-allocation emission as retention and union census;
/// no font/resource identity is reconstructed from a scalar receipt.
pub(crate) fn proof_frames_contain_resource(
    frames: &[crate::protocol::FinalizedLine],
    bytes: &[u8],
    budget: &mut ScopeAlignmentBudget,
) -> Result<bool> {
    let work = crate::ScopeAlignmentLimitKind::StructuralWork;
    budget.work(checked_sum(work, frames.len(), 1)?)?;
    if bytes.is_empty() {
        return Ok(false);
    }
    let resource = AllocationWitness::resource(bytes).identity();
    let mut retained = false;
    for frame in frames {
        visit_finalized_own_allocations(&frame.inner, &mut |witness| {
            budget.work(checked_sum(work, witness.records(), 1)?)?;
            if witness.records() == 0 && witness.payload_bytes() == 0 {
                return Ok(());
            }
            budget.work(1)?;
            retained |= witness.identity() == resource;
            Ok(())
        })?;
    }
    Ok(retained)
}

pub(crate) fn retained_selected(
    prepared: &PreparedData,
    scopes: &SelectedScopes,
    basis: &crate::engine::SelectedShape,
    policy: crate::SpacingPolicy,
    slots: &Vec<crate::protocol::selection::SpacingSlotData>,
    request: &crate::protocol::LineSelectionRequest,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    let mut set = AllocationSet::new();
    visit_selected_allocations(
        prepared,
        scopes,
        basis,
        policy,
        slots,
        request,
        &mut |allocation| set.observe(allocation, budget).map(|_| ()),
    )?;
    set.finish(budget)
}

pub(crate) fn retained_finalized(
    selected: &crate::protocol::selection::SelectedData,
    data: &crate::engine::layout::data::FinalizedLineData,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    let mut set = AllocationSet::new();
    visit_selected_allocations(
        &selected.prepared,
        &selected.scopes,
        &selected.basis,
        selected.request.context.spacing,
        &selected.spacing_slots,
        &selected.request,
        &mut |allocation| set.observe(allocation, budget).map(|_| ()),
    )?;
    data.visit_scope_allocations(&mut |allocation| set.observe(allocation, budget).map(|_| ()))?;
    let mut retention = set.finish(budget)?;
    // The published Arc object does not exist yet. Add its actual fixed size
    // to this scalar receipt without fabricating an allocation identity. Its
    // embedded data and prefix headers are already part of this single header.
    retention.payload_bytes = checked_sum(
        crate::ScopeAlignmentLimitKind::PayloadBytes,
        retention.payload_bytes,
        std::mem::size_of::<crate::engine::FinalizedLine>(),
    )?;
    Ok(retention)
}

fn product_prepared<'a>(product: crate::protocol::ScopeAlignmentProduct<'a>) -> &'a PreparedData {
    use crate::protocol::ScopeAlignmentProduct as Product;
    match product {
        Product::Prepared(value) => &value.inner,
        Product::Selected(value) => &value.inner.prepared,
        Product::Finalized(value) => &value.inner.selected.prepared,
    }
}

#[derive(Clone, Copy)]
enum Input<'a> {
    Product(crate::protocol::ScopeAlignmentProduct<'a>),
    Placed(&'a crate::engine::layout::data::PlacedLayout),
    Selection {
        prepared: &'a PreparedData,
        request: &'a crate::protocol::LineSelectionRequest,
    },
    Flow {
        prepared: &'a PreparedData,
        segment: &'a crate::protocol::PreparedFlowSegment,
    },
}

impl<'a> Input<'a> {
    fn public(input: crate::protocol::ScopeAlignmentCensusInput<'a>) -> Self {
        use crate::protocol::ScopeAlignmentCensusInput as Public;
        match input {
            Public::Product(product) => Self::Product(product),
            Public::Selection { prepared, request } => Self::Selection {
                prepared: &prepared.inner,
                request,
            },
            // FlowObservationRequest is geometry-only and owns no frame edge.
            Public::Flow {
                prepared,
                segment,
                request: _,
            } => Self::Flow {
                prepared: &prepared.inner,
                segment,
            },
        }
    }
    fn prepared(self) -> &'a PreparedData {
        match self {
            Self::Product(product) => product_prepared(product),
            Self::Placed(layout) => &layout.prepared,
            Self::Selection { prepared, .. } | Self::Flow { prepared, .. } => prepared,
        }
    }
}

/// Proof owners also expose non-emitting rows. Keep their inspection work
/// separate from allocation identities instead of inventing zero-byte owners.
enum Visit {
    Work(usize),
    Allocation(AllocationWitness),
}
type Visitor<'a> = dyn FnMut(Visit) -> Result<()> + 'a;

fn visit_prepared(prepared: &PreparedData, visit: &mut AllocationVisitor<'_>) -> Result<()> {
    if prepared.scope_alignment.retention.is_none() {
        return Ok(());
    }
    preparation::visit_prepared_allocations(
        &prepared.scope_alignment,
        &prepared.analyzed,
        &prepared.shaped,
        &prepared.tables,
        visit,
    )
}

#[derive(Clone, Copy)]
enum Phase {
    Input,
    External,
}

/// Borrow the actual stable composite owners, not copied allocation IDs.
/// Their arenas can still overlap through shared preparations and fonts;
/// allocation-level deduplication remains responsible for those overlaps.
#[derive(Clone, Copy)]
enum Root<'a> {
    Prepared(&'a PreparedData),
    Request(&'a crate::protocol::LineSelectionRequest),
    Selected(&'a Arc<crate::protocol::selection::SelectedData>),
    Finalized(&'a Arc<crate::engine::FinalizedLine>),
}
impl Root<'_> {
    fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Prepared(a), Self::Prepared(b)) => std::ptr::eq(a, b),
            (Self::Request(a), Self::Request(b)) => std::ptr::eq(a, b),
            (Self::Selected(a), Self::Selected(b)) => Arc::ptr_eq(a, b),
            (Self::Finalized(a), Self::Finalized(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
    fn allocations(self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        match self {
            Self::Prepared(value) => visit_prepared(value, visit),
            Self::Request(value) => {
                crate::protocol::inline_box::visit_request_allocations(value, visit)
            }
            Self::Selected(value) => visit_selected_own_allocations(
                &value.prepared,
                &value.scopes,
                &value.basis,
                value.request.context.spacing,
                &value.spacing_slots,
                &value.request,
                visit,
            ),
            Self::Finalized(value) => visit_finalized_own_allocations(value, visit),
        }
    }
}

enum RootVisit<'a> {
    Work(usize),
    Owner { root: Root<'a>, phase: Phase },
}

fn related_slots(related: crate::protocol::ScopeAlignmentProducts<'_>) -> usize {
    use crate::protocol::ScopeAlignmentProducts as Products;
    match related {
        Products::Mixed(values) => values.len(),
        Products::FinalizedSlots(values) => values.len(),
    }
}

fn visit_closed_roots<'a>(
    frames: &'a [crate::protocol::FinalizedLine],
    phase: Phase,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    emit(RootVisit::Work(frames.len()))?;
    for frame in frames {
        emit(RootVisit::Owner {
            root: Root::Finalized(&frame.inner),
            phase,
        })?;
    }
    Ok(())
}

/// Open one actual finalized owner and its already transitive proof closure.
/// Placement receipts borrow the same Arc directly, including nonformatted rows.
fn visit_finalized_roots<'a>(
    frame: &'a Arc<crate::engine::FinalizedLine>,
    phase: Phase,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    if frame.data.scope_retention.is_none() {
        return Ok(());
    }
    emit(RootVisit::Owner {
        root: Root::Finalized(frame),
        phase,
    })?;
    visit_closed_roots(&frame.selected.scopes.proof_frames, phase, emit)
}

fn visit_product_roots<'a>(
    product: crate::protocol::ScopeAlignmentProduct<'a>,
    phase: Phase,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    use crate::protocol::ScopeAlignmentProduct as Product;
    match product {
        Product::Prepared(value) if value.inner.scope_alignment.retention.is_some() => {
            emit(RootVisit::Owner {
                root: Root::Prepared(&value.inner),
                phase,
            })
        }
        Product::Selected(value) if value.inner.scopes.retention.is_some() => {
            emit(RootVisit::Owner {
                root: Root::Selected(&value.inner),
                phase,
            })?;
            visit_closed_roots(&value.inner.scopes.proof_frames, phase, emit)
        }
        Product::Finalized(value) => visit_finalized_roots(&value.inner, phase, emit),
        _ => Ok(()),
    }
}

/// Active Cell preparation publishes retention at every subsequent selected and
/// finalized owner, even for ordinary or nonformatted selections. Missing facts
/// are an incomplete canonical product, not Native ownership or absent limits.
fn require_range_frame_retention(frame: &Arc<crate::engine::FinalizedLine>) -> Result<()> {
    if frame.selected.prepared.scope_alignment.retention.is_none() {
        return Err(crate::Error::new(
            crate::ErrorCode::LayoutFailed,
            "activated Cell Range frame has no prepared retention",
        ));
    }
    if frame.selected.scopes.retention.is_none() {
        return Err(crate::Error::new(
            crate::ErrorCode::LayoutFailed,
            "activated Cell Range frame has no selected retention",
        ));
    }
    if frame.data.scope_retention.is_none() {
        return Err(crate::Error::new(
            crate::ErrorCode::LayoutFailed,
            "activated Cell Range frame has no finalized retention",
        ));
    }
    Ok(())
}

fn visit_input_roots<'a>(
    input: Input<'a>,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    use crate::protocol::scope_alignment::ScopeProofVisit as Proof;
    if let Input::Product(product) = input {
        return visit_product_roots(product, Phase::Input, emit);
    }
    if matches!(input, Input::Placed(_)) && input.prepared().scope_alignment.retention.is_none() {
        return Err(crate::Error::new(
            crate::ErrorCode::LayoutFailed,
            "activated Cell Range placement has no prepared retention",
        ));
    }
    if input.prepared().scope_alignment.retention.is_some() {
        emit(RootVisit::Owner {
            root: Root::Prepared(input.prepared()),
            phase: Phase::Input,
        })?;
    }
    if let Input::Selection { request, .. } = input
        && crate::protocol::inline_box::contexts(&request.controls, &request.boundary_context)
            .next()
            .is_some()
    {
        emit(RootVisit::Work(request.controls.len()))?;
        emit(RootVisit::Work(request.boundary_context.len()))?;
        emit(RootVisit::Owner {
            root: Root::Request(request),
            phase: Phase::Input,
        })?;
    }
    if let Input::Placed(layout) = input {
        // Receipts are the complete semantic sequence. Visible lines alias these
        // same owners; iterating only lines would omit nonformatted custody.
        emit(RootVisit::Work(layout.receipts.len()))?;
        for frame in &layout.receipts {
            require_range_frame_retention(frame)?;
            // Validate the actual closed proof owners before the legacy generic
            // visitor can skip an owner without feature retention. These are
            // separate real inspections, in addition to their later emission.
            let proofs = &frame.selected.scopes.proof_frames;
            emit(RootVisit::Work(proofs.len()))?;
            for proof in proofs {
                require_range_frame_retention(&proof.inner)?;
            }
            visit_finalized_roots(frame, Phase::Input, emit)?;
        }
        return Ok(());
    }
    let mut proof = |edge: Proof<'a>| match edge {
        Proof::Work(count) => emit(RootVisit::Work(count)),
        Proof::Frame(frame) => {
            emit(RootVisit::Work(1))?;
            emit(RootVisit::Owner {
                root: Root::Finalized(&frame.inner),
                phase: Phase::Input,
            })?;
            // This list is already transitive. Do not follow older lists again.
            visit_closed_roots(
                &frame.inner.selected.scopes.proof_frames,
                Phase::Input,
                emit,
            )
        }
    };
    match input {
        Input::Selection { request, .. } => {
            crate::protocol::scope_alignment::visit_request_scope_frames(request, &mut proof)
        }
        Input::Flow { segment, .. } => segment.visit_scope_frames(&mut proof),
        Input::Product(_) | Input::Placed(_) => Ok(()),
    }
}

/// Inspect every supplied slot (including None) and every actual proof role.
/// The input is first in this stable stream, before any related owner.
fn visit_raw_roots<'a>(
    input: Input<'a>,
    related: crate::protocol::ScopeAlignmentProducts<'a>,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    use crate::protocol::ScopeAlignmentProducts as Products;
    emit(RootVisit::Work(checked_sum(
        crate::ScopeAlignmentLimitKind::StructuralWork,
        related_slots(related),
        1,
    )?))?;
    visit_input_roots(input, emit)?;
    match related {
        Products::Mixed(values) => {
            for &product in values {
                visit_product_roots(product, Phase::External, emit)?;
            }
        }
        Products::FinalizedSlots(values) => {
            for frame in values.iter().flatten() {
                visit_product_roots(
                    crate::protocol::ScopeAlignmentProduct::Finalized(frame),
                    Phase::External,
                    emit,
                )?;
            }
        }
    }
    Ok(())
}

/// Canonicalize composite owners across the whole input/related graph before
/// opening their arenas. Fixed-stack rescans compare real owner identities;
/// every raw alias, proof role, slot and comparison remains metered.
fn visit_roots<'a>(
    input: Input<'a>,
    related: crate::protocol::ScopeAlignmentProducts<'a>,
    emit: &mut dyn FnMut(RootVisit<'a>) -> Result<()>,
) -> Result<()> {
    let work = crate::ScopeAlignmentLimitKind::StructuralWork;
    let mut ordinal = 0;
    visit_raw_roots(input, related, &mut |event| {
        let (root, phase) = match event {
            RootVisit::Work(count) => return emit(RootVisit::Work(count)),
            RootVisit::Owner { root, phase } => (root, phase),
        };
        let mut prior_ordinal = 0;
        let mut duplicate = false;
        visit_raw_roots(input, related, &mut |prior| {
            match prior {
                RootVisit::Work(count) => emit(RootVisit::Work(count))?,
                RootVisit::Owner { root: prior, .. } => {
                    if prior_ordinal < ordinal {
                        emit(RootVisit::Work(1))?;
                        duplicate |= root.same(prior);
                    }
                    prior_ordinal = checked_sum(work, prior_ordinal, 1)?;
                }
            }
            Ok(())
        })?;
        if !duplicate {
            emit(RootVisit::Owner { root, phase })?;
        }
        ordinal = checked_sum(work, ordinal, 1)?;
        Ok(())
    })
}

fn visit_products(
    input: Input<'_>,
    related: crate::protocol::ScopeAlignmentProducts<'_>,
    visit: &mut Visitor<'_>,
) -> Result<()> {
    visit_roots(input, related, &mut |event| match event {
        RootVisit::Work(count) => visit(Visit::Work(count)),
        RootVisit::Owner { root, .. } => {
            root.allocations(&mut |allocation| visit(Visit::Allocation(allocation)))
        }
    })
}

fn add_retention(
    retention: &mut crate::ScopeAlignmentRetention,
    witness: AllocationWitness,
) -> Result<()> {
    retention.records = checked_sum(
        crate::ScopeAlignmentLimitKind::Records,
        retention.records,
        witness.records(),
    )?;
    retention.payload_bytes = checked_sum(
        crate::ScopeAlignmentLimitKind::PayloadBytes,
        retention.payload_bytes,
        witness.payload_bytes(),
    )?;
    Ok(())
}

/// Allocation-free exact union of already live owners. The immutable input
/// and flat typed proof closure make ordinal rescans stable. Every inspection
/// and identity comparison is metered, including non-emitting proof work.
fn retained_union(
    input: Input<'_>,
    related: crate::protocol::ScopeAlignmentProducts<'_>,
    budget: &mut ScopeAlignmentBudget,
) -> Result<crate::ScopeAlignmentRetention> {
    let work = crate::ScopeAlignmentLimitKind::StructuralWork;
    let mut ordinal = 0;
    let mut retention = crate::ScopeAlignmentRetention::default();
    visit_products(input, related, &mut |event| {
        let witness = match event {
            Visit::Work(count) => return budget.work(count),
            Visit::Allocation(witness) => witness,
        };
        budget.work(checked_sum(work, witness.records(), 1)?)?;
        if witness.records() == 0 && witness.payload_bytes() == 0 {
            return Ok(());
        }
        let mut prior_ordinal = 0;
        let mut duplicate = false;
        visit_products(input, related, &mut |event| {
            let prior = match event {
                Visit::Work(count) => return budget.work(count),
                Visit::Allocation(prior) => prior,
            };
            budget.work(checked_sum(work, prior.records(), 1)?)?;
            if prior.records() == 0 && prior.payload_bytes() == 0 {
                return Ok(());
            }
            if prior_ordinal < ordinal {
                budget.work(1)?;
                duplicate |= prior.identity() == witness.identity();
            }
            prior_ordinal = checked_sum(work, prior_ordinal, 1)?;
            Ok(())
        })?;
        if !duplicate {
            add_retention(&mut retention, witness)?;
        }
        ordinal = checked_sum(work, ordinal, 1)?;
        Ok(())
    })?;
    Ok(retention)
}

struct Census {
    input_retention: crate::ScopeAlignmentRetention,
    external_retention: crate::ScopeAlignmentRetention,
    budget: ScopeAlignmentBudget,
}

fn census(
    input: Input<'_>,
    related: crate::protocol::ScopeAlignmentProducts<'_>,
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<Option<Census>> {
    let Some(limits) = super::operation_limits(input.prepared(), allowance)? else {
        return Ok(None);
    };
    let mut budget = ScopeAlignmentBudget::new(limits, 0, 0)?;
    let retained = retained_union(input, related, &mut budget)?;
    // Seed the complete already live union before any fallible seen growth.
    // Neither the prepass work nor the observed peak is reset afterwards.
    budget.add_records(retained.records())?;
    budget.retain_bytes(retained.payload_bytes())?;
    let mut set = AllocationSet::new();
    let mut input_retention = crate::ScopeAlignmentRetention::default();
    let mut external_retention = crate::ScopeAlignmentRetention::default();
    visit_roots(input, related, &mut |event| match event {
        RootVisit::Work(count) => budget.work(count),
        RootVisit::Owner { root, phase } => root.allocations(&mut |allocation| {
            if set.observe(allocation, &mut budget)? {
                match phase {
                    Phase::Input => add_retention(&mut input_retention, allocation)?,
                    Phase::External => add_retention(&mut external_retention, allocation)?,
                }
            }
            Ok(())
        }),
    })?;
    set.finish(&mut budget)?;
    Ok(Some(Census {
        input_retention,
        external_retention,
        budget,
    }))
}

pub(crate) fn begin_selection(
    prepared: &PreparedData,
    request: &crate::protocol::LineSelectionRequest,
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<Option<ScopeAlignmentBudget>> {
    Ok(census(
        Input::Selection { prepared, request },
        crate::protocol::ScopeAlignmentProducts::Mixed(&[]),
        allowance,
    )?
    .map(|value| value.budget))
}

pub(crate) fn begin_flow(
    prepared: &PreparedData,
    segment: &crate::protocol::PreparedFlowSegment,
    _request: &crate::protocol::FlowObservationRequest,
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<Option<ScopeAlignmentBudget>> {
    Ok(census(
        Input::Flow { prepared, segment },
        crate::protocol::ScopeAlignmentProducts::Mixed(&[]),
        allowance,
    )?
    .map(|value| value.budget))
}

/// Start an activated Cell Range query from the actual complete placed input.
/// The immutable receipt/proof stream is borrowed through both census passes;
/// no wrapper storage or summed per-receipt retention seeds the transaction.
pub(crate) fn begin_range(
    layout: &crate::engine::layout::data::PlacedLayout,
) -> Result<ScopeAlignmentBudget> {
    let value = census(
        Input::Placed(layout),
        crate::protocol::ScopeAlignmentProducts::Mixed(&[]),
        None,
    )?
    .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    Ok(value.budget)
}

pub(crate) fn live_products(
    input: crate::protocol::ScopeAlignmentCensusInput<'_>,
    related: crate::protocol::ScopeAlignmentProducts<'_>,
    allowance: crate::ScopeAlignmentAllowance,
) -> Result<(
    crate::ScopeAlignmentRetention,
    crate::ScopeAlignmentRetention,
    crate::ScopeAlignmentUsage,
)> {
    let value = census(Input::public(input), related, Some(allowance))?
        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    Ok((
        value.input_retention,
        value.external_retention,
        value.budget.usage(),
    ))
}
