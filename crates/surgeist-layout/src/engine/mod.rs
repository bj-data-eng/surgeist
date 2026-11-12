pub(crate) mod block_scope;
pub(crate) mod containment;
pub(crate) mod contracts;
mod entry_sources;
mod entry_staging;
mod hidden;
mod root;
mod rounding;
pub(crate) use rounding::FragmentAnchor;
mod session;
#[cfg(test)]
pub(crate) use session::ComputeSession;
pub(crate) use session::{FloatChildPacket, FloatChildTrial};
pub(crate) mod validation;

use crate::error::LayoutResultOf;
use crate::sizing::resolve::{ResolvedPreferredSize, resolve_preferred_sizing};
use crate::{
    BlockGeneration, CompletedLayoutBatchOf, Display, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation,
    LayoutRootContextOf, LayoutRootRequestOf, LayoutScalar, LayoutTree,
    LayoutUnsupportedCapability, Overflow, PhysicalAxis, Position, Size, SizingAlgorithm, Traverse,
};
#[cfg(test)]
pub(crate) use hidden::compute_hidden;
pub(crate) use root::compute_root;
#[cfg(test)]
pub(crate) use rounding::round_layout;
#[cfg(test)]
pub(crate) use session::trace_hidden_compute_session_requests;
#[cfg(test)]
pub(crate) use session::{observe_grid_probe, seed_grid_leaf_requests};

type CompletedTreeBatch<Tree> =
    CompletedLayoutBatchOf<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

type TextPassBasis<Tree> =
    crate::text_layout::state::PassBasis<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

type ComputeLayoutResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    CompletedTreeBatch<Tree>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

/// Measures the continuous physical border-box size of an empty, unmeasured,
/// in-flow inline block before preparing its surrounding text.
///
/// The actual paragraph container must be the principal Block root of the
/// supplied root-local positioned request. Its authored preferred dimensions
/// must resolve without leaf measurement or a content-dependent scrollbar
/// reservation. The actual atomic
/// must be a direct child reachable by exactly one path in the container's
/// acyclic topology. Only the container and atomic need box inputs for this
/// query. The caller supplies the same immutable box inputs, topology and
/// authenticated root request that composition will use. Recompute after any
/// of those sizing inputs change.
///
/// The result carries no baseline, placement, or cache authority. Composition
/// computes those facts with the prepared Text and the full layout tree.
pub fn measure_definite_inline_atomic_size<Tree>(
    tree: &Tree,
    container: Tree::Node,
    atomic: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Size<Tree::Scalar>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let error = |kind| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject {
                container,
                subject: atomic,
            },
            LayoutOperation::RootLayout,
            kind,
        )
    };
    let unsupported = || {
        error(LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::InlineAtomicSizePreparation,
        ))
    };
    let LayoutRootContextOf::RootLocalPositioned(frame) = request.context() else {
        return Err(unsupported());
    };
    let graph = validation::discover_graph(tree, container, LayoutOperation::RootLayout)?;
    let mut direct_atomic = graph.children[0]
        .iter()
        .copied()
        .filter(|&index| graph.nodes[index] == atomic);
    let (Some(atomic_index), None) = (direct_atomic.next(), direct_atomic.next()) else {
        return Err(error(LayoutErrorKindOf::InvalidInput(
            LayoutInvalidInputOf::InlineAtomicNotDirectChild,
        )));
    };
    if graph.path_counts()[atomic_index] != 1 {
        return Err(error(LayoutErrorKindOf::InvalidInput(
            LayoutInvalidInputOf::InlineAtomicAmbiguousOccurrence,
        )));
    }
    let Some(container_style) = tree.layout_input(container).as_box() else {
        return Err(unsupported());
    };
    let Some(atomic_style) = tree.layout_input(atomic).as_box() else {
        return Err(unsupported());
    };
    if !core::ptr::eq(container_style, tree.node_input(container))
        || !core::ptr::eq(atomic_style, tree.node_input(atomic))
        || container_style.display != Display::Block
        || container_style.block_generation != BlockGeneration::Principal
        || container_style.item_is_table
        || tree.has_leaf_measurement(container)
        || (container_style.scrollbar_width.get() != Tree::Scalar::ZERO
            && [container_style.overflow.x(), container_style.overflow.y()]
                .contains(&Overflow::Auto))
        || atomic_style.display != Display::InlineBlock
        || atomic_style.block_generation != BlockGeneration::Principal
        || atomic_style.position != Position::Relative
        || !atomic_style.float.is_none()
        || atomic_style.item_is_replaced
        || atomic_style.item_is_table
        || !graph.children[atomic_index].is_empty()
        || tree.has_leaf_measurement(atomic)
    {
        return Err(unsupported());
    }

    // Reject content-derived frame axes before root normalization can ask for
    // intrinsic content. Use the production preferred-size dispatch to admit
    // the authored basis, then let Block own the actual used size and insets.
    let containing = frame.containing_size();
    for (axis, preferred, basis) in [
        (
            PhysicalAxis::Horizontal,
            &container_style.size.width,
            containing.width,
        ),
        (
            PhysicalAxis::Vertical,
            &container_style.size.height,
            containing.height,
        ),
    ] {
        let resolved = resolve_preferred_sizing(
            preferred,
            SizingAlgorithm::Block,
            axis,
            Some(basis.get()),
            false,
        )
        .map_err(|reason| crate::error::sizing_resolution_error(container, reason))?;
        if !matches!(resolved, ResolvedPreferredSize::Definite(_)) {
            return Err(error(LayoutErrorKindOf::MissingContext(
                LayoutMissingContext::RequiredBasis,
            )));
        }
    }
    let mut session = session::ComputeSession::new(tree, Vec::new());
    let input = root::root_compute_input(&mut session, container, container_style, request)?.input;
    crate::block::measure_definite_inline_atomic(&mut session, container, atomic, input)
}

pub fn compute_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    compute_layout_invalidated(tree, root, request, &[])
}

pub fn compute_layout_invalidated<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    changed_nodes: &[Tree::Node],
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    let (graph, invalidated_nodes) =
        validation::validate_layout_request_with_text_graph(tree, root, changed_nodes, &[])?;
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        &graph,
        &crate::WrappingInput::default(),
        &[],
        |_| false,
    )
    .map_err(|reason| crate::inline_wrapping::wrapping_error(root, reason))?;
    compute_validated_layout(
        tree,
        root,
        request,
        invalidated_nodes,
        &graph,
        std::sync::Arc::new(wrapping),
    )
}

fn compute_validated_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    invalidated_nodes: Vec<Tree::Node>,
    graph: &validation::ValidationGraph<Tree::Node>,
    wrapping: std::sync::Arc<crate::inline_wrapping::WrappingContext<Tree::Node>>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    let mut session = session::ComputeSession::new(tree, invalidated_nodes);
    session.install_wrapping(wrapping, graph);
    run_session(session, root, request)
}

/// Computes checked immutable inline flow, including numeric-only flow.
/// Text paragraphs retain their actual finalized layouts; old geometry caches
/// cannot stand in for Text finalization.
pub fn compute_layout_with_flow<Tree>(
    capture: &crate::LayoutFlowInput<'_, Tree>,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    compute_layout_with_flow_invalidated(capture, root, request, &[])
}

/// Checked inline-flow computation with the source-owned invalidation closure.
pub fn compute_layout_with_flow_invalidated<Tree>(
    capture: &crate::LayoutFlowInput<'_, Tree>,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    changed_nodes: &[Tree::Node],
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    if root != capture.root {
        return Err(crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(root),
            crate::LayoutOperation::TextCapture,
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::TextBinding(
                crate::TextBindingError::RootMismatch,
            )),
        ));
    }
    if let Some(bound) = &capture.bound {
        if request != bound.root_request() {
            return Err(LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(root),
                crate::LayoutOperation::RootLayout,
                LayoutErrorKindOf::InvalidInput(
                    crate::LayoutInvalidInputOf::BlockFragmentationRequest,
                ),
            ));
        }
        if matches!(
            bound.mode(),
            crate::block_fragmentation::BoundMode::Block { .. }
        ) {
            return crate::block_fragmentation::compute(capture, root, changed_nodes);
        }
    }
    let mut invalidation_seeds = changed_nodes.to_vec();
    invalidation_seeds.extend(capture.invalidated.iter().copied());
    let invalidated = capture.graph.invalidation(&invalidation_seeds)?;
    let session = session::ComputeSession::with_flow(capture, invalidated);
    run_session(session, root, request)
}

type FragmentIntrinsicResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    crate::block::intrinsic::FragmentContext<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

pub(crate) fn prepare_block_fragment_intrinsics<Tree: LayoutTree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    target: crate::LayoutTargetBasis,
    axes: crate::FlowAxes,
) -> FragmentIntrinsicResult<Tree> {
    let mut session = session::ComputeSession::new(tree, Vec::new());
    session.bind_fragment_frame(target.clone(), Vec::new());
    let input = root::root_compute_input(&mut session, root, tree.node_input(root), request)?
        .input
        .with_run_mode(crate::RunMode::ComputeSize);
    let mut entries = Vec::new();
    crate::block::intrinsic::prepare(&mut session, root, input, axes, &mut entries)?;
    Ok(crate::block::intrinsic::FragmentContext {
        target,
        request,
        entries,
    })
}

pub(crate) struct BlockContentFrame<Node, S: LayoutScalar> {
    pub(crate) context: crate::block::content::Context<S>,
    pub(crate) facts: crate::block::content::Facts<S>,
    pub(crate) principal: Option<crate::ComputeOutputOf<S>>,
    pub(crate) packet: block_scope::Packet<Node, S>,
    pub(crate) carrier: Option<crate::block::carrier::PerformedPath<Node, S>>,
}

pub(crate) struct BlockFrameBinding<Node, S: LayoutScalar> {
    pub(crate) allocations: Vec<crate::block_fragmentation::FragmentBoxAllocation<Node, S>>,
    pub(crate) intrinsics: std::sync::Arc<crate::block::intrinsic::FragmentContext<Node, S>>,
    pub(crate) dispatch: block_scope::Dispatch<Node, S>,
}

/// Issue parent-local carrier origins from the same canonical inline contexts
/// used by actual content production, without constructing principal geometry.
type CarrierPathResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    crate::block::carrier::PreparedPath<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;
pub(crate) fn prepare_block_carrier_path<Tree: LayoutTree>(
    tree: &Tree,
    path: &[Tree::Node],
    request: LayoutRootRequestOf<Tree::Scalar>,
    frame: crate::BlockFragmentainerOf<Tree::Scalar>,
    intrinsics: std::sync::Arc<crate::block::intrinsic::FragmentContext<Tree::Node, Tree::Scalar>>,
    binding: crate::block::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar>,
) -> CarrierPathResult<Tree> {
    let root = path[0];
    let mut session = session::ComputeSession::new(tree, Vec::new());
    session.bind_fragment_frame_with_intrinsics(intrinsics.clone(), Vec::new());
    let input = intrinsics
        .entries
        .iter()
        .find(|entry| entry.node == root)
        .map(|entry| entry.input.with_run_mode(crate::RunMode::PerformLayout))
        .ok_or_else(|| {
            crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(root),
                crate::LayoutOperation::RootLayout,
                crate::LayoutErrorKindOf::BlockFragmentation(
                    crate::BlockFragmentationFailure::InvalidSource,
                ),
            )
        })?;
    let (first, mut context) = crate::block::carrier::root(&mut session, root, input, frame.size)?;
    let mut facts = Vec::new();
    facts.try_reserve(path.len()).map_err(|_| {
        crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(root),
            crate::LayoutOperation::RootLayout,
            crate::LayoutErrorKindOf::Allocation,
        )
    })?;
    facts.push(first);
    for pair in path.windows(2) {
        let input = intrinsics
            .entries
            .iter()
            .find(|entry| entry.node == pair[1])
            .map(|entry| entry.input.with_run_mode(crate::RunMode::PerformLayout))
            .ok_or_else(|| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(pair[1]),
                    crate::LayoutOperation::RootLayout,
                    crate::LayoutErrorKindOf::BlockFragmentation(
                        crate::BlockFragmentationFailure::InvalidSource,
                    ),
                )
            })?;
        let (fact, child_context) =
            crate::block::carrier::child(&mut session, pair[0], pair[1], input, context)?;
        facts.push(fact);
        context = child_context;
    }
    crate::block::carrier::PreparedPath::new(binding, facts, request, frame, intrinsics)
}

pub(crate) fn settle_block_carrier_origin<Tree: LayoutTree>(
    tree: &Tree,
    path: &crate::block::carrier::AcceptedPath<Tree::Node, Tree::Scalar>,
    production: block_scope::ProductionId,
    binding: &crate::block::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar>,
    node: Tree::Node,
    principal_parent: Option<crate::LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
) -> LayoutResultOf<
    Tree::Node,
    crate::block::carrier::Origin<Tree::Scalar>,
    Tree::Scalar,
    Tree::MeasureError,
> {
    let session = session::ComputeSession::new(tree, Vec::new());
    path.origin(&session, production, binding, node, principal_parent)
}
pub(crate) struct BlockScopeEntryRequest<Node, S: LayoutScalar> {
    pub(crate) pending: crate::block::scope::Pending<Node, S>,
    pub(crate) binding: BlockFrameBinding<Node, S>,
}

type BlockScopeEntryFacts<S> = (
    crate::block::entry::Qualification<S>,
    crate::block::content::Context<S>,
);
type EntryQualificationResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    BlockScopeEntryFacts<<Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;
type BodyQualificationResult<Tree> = LayoutResultOf<
    <Tree as Traverse>::Node,
    BlockScopeBodyFrame<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;
type ContentFrameResult<Tree> = block_scope::ComputeResult<
    <Tree as Traverse>::Node,
    BlockContentFrame<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;
type ScopeRootFrameResult<Tree> = block_scope::ComputeResult<
    <Tree as Traverse>::Node,
    block_scope::Packet<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;

/// Qualify only the receiving-side entry margin and canonical content context.
/// Completing an opposite-side collapse chain remains a body demand.
pub(crate) fn qualify_block_scope_entry<Tree: LayoutTree>(
    tree: &Tree,
    request: BlockScopeEntryRequest<Tree::Node, Tree::Scalar>,
) -> EntryQualificationResult<Tree> {
    let BlockScopeEntryRequest { pending, binding } = request;
    let mut session = session::ComputeSession::new(tree, Vec::new());
    session.bind_fragment_frame_with_intrinsics(binding.intrinsics, binding.allocations);
    session.install_block_scopes(pending.node, binding.dispatch)?;
    let context = crate::block::content::prepare(
        &mut session,
        pending.node,
        pending.input,
        tree.node_input(pending.node),
    )?;
    let fact = crate::block::entry::qualify(
        &mut session,
        pending.node,
        pending.input,
        pending.axes.block_start(),
    )?;
    Ok((fact, context))
}

pub(crate) struct BlockScopeBodyFrame<Node, S: LayoutScalar> {
    pub(crate) output: crate::ComputeOutputOf<S>,
    pub(crate) packet: block_scope::Packet<Node, S>,
}
/// Non-consuming whole-body qualification uses the ordinary producer without
/// scope interception. Native has already qualified all interior controls.
pub(crate) fn qualify_block_scope_body<Tree: LayoutTree>(
    tree: &Tree,
    request: BlockScopeEntryRequest<Tree::Node, Tree::Scalar>,
) -> BodyQualificationResult<Tree> {
    let BlockScopeEntryRequest { pending, binding } = request;
    let mut session = session::ComputeSession::new(tree, Vec::new());
    session.bind_fragment_frame_with_intrinsics(binding.intrinsics, Vec::new());
    session.retain_block_bodies();
    let input = pending.input.with_run_mode(crate::RunMode::PerformLayout);
    let output = block_scope::ordinary(crate::block::compute_block_scoped(
        &mut session,
        pending.node,
        input,
    ))?;
    session.record_block_body(pending.node, input, output)?;
    Ok(BlockScopeBodyFrame {
        output,
        packet: session.block_packet()?,
    })
}

pub(crate) struct BlockContentFrameRequest<'a, Node, S: LayoutScalar> {
    pub(crate) root: Node,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) original: &'a crate::NodeInputOf<S>,
    pub(crate) reference: S,
    pub(crate) opening: bool,
    pub(crate) complete: bool,
    pub(crate) principal_content: Option<S>,
    pub(crate) accepted: Option<crate::block::content_insets::ContentInsetBinding<Node, S>>,
    pub(crate) carrier: Option<crate::block::carrier::PreparedPath<Node, S>>,
    pub(crate) binding: BlockFrameBinding<Node, S>,
}
pub(crate) struct BlockScopeRootFrameRequest<Node, S: LayoutScalar> {
    pub(crate) root: Node,
    pub(crate) request: LayoutRootRequestOf<S>,
    pub(crate) frame: crate::BlockFragmentainerOf<S>,
    pub(crate) axes: crate::FlowAxes,
    pub(crate) binding: BlockFrameBinding<Node, S>,
}

/// Execute one exact child-content query. Neither suspension nor measurement
/// adopts child work into its parent or publishes a completed host batch.
pub(crate) fn compute_block_content_frame<Tree: LayoutTree>(
    tree: &Tree,
    request: BlockContentFrameRequest<'_, Tree::Node, Tree::Scalar>,
) -> ContentFrameResult<Tree> {
    let BlockContentFrameRequest {
        root,
        input,
        original,
        reference,
        opening,
        complete,
        principal_content,
        accepted,
        carrier,
        binding,
    } = request;
    let (graph, invalidated) =
        validation::validate_layout_request_with_text_graph(tree, root, &[], &[])?;
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        &graph,
        &crate::WrappingInput::default(),
        &[],
        |_| false,
    )
    .map_err(|reason| crate::inline_wrapping::wrapping_error(root, reason))?;
    let carrier_snapshot = binding.intrinsics.clone();
    let mut session = session::ComputeSession::new(tree, invalidated);
    session.bind_fragment_frame_with_intrinsics(binding.intrinsics, binding.allocations);
    session.install_block_scopes(root, binding.dispatch)?;
    session.retain_block_bodies();
    session.install_wrapping(std::sync::Arc::new(wrapping), &graph);
    session.begin_content_scope(root, input)?;
    let context = crate::block::content::prepare(&mut session, root, input, original)?;
    let mut projection = crate::block::content::Projection {
        block_reference: reference,
        content_origin: if opening {
            context.opening()
        } else {
            Tree::Scalar::ZERO
        },
    };
    let mut receipt =
        crate::block::content::produce(&mut session, root, &context, projection, accepted.clone())?;
    if !receipt.participated && !complete && projection.content_origin != Tree::Scalar::ZERO {
        projection.content_origin = Tree::Scalar::ZERO;
        receipt = crate::block::content::produce(
            &mut session,
            root,
            &context,
            projection,
            accepted.clone(),
        )?;
    }
    if principal_content.is_none()
        && let Some(expected) = &accepted
    {
        crate::block::content::settle_accepted_insets(&mut session, &mut receipt, expected)?;
    }
    let carrier = match carrier {
        Some(candidate) => Some(candidate.performed(
            &receipt,
            context,
            accepted.as_ref().ok_or_else(|| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(root),
                    crate::LayoutOperation::ValueResolution,
                    crate::LayoutErrorKindOf::MissingContext(
                        crate::LayoutMissingContext::RequiredBasis,
                    ),
                )
            })?,
            &carrier_snapshot,
        )?),
        None => None,
    };
    let facts = receipt.facts();
    let principal = match principal_content {
        Some(content) => Some(crate::block::content::materialize(
            &mut session,
            root,
            &context,
            receipt,
            content,
        )?),
        None => None,
    };
    if let Some(output) = principal {
        session.record_block_body(root, input, output)?;
    }
    Ok(BlockContentFrame {
        context,
        facts,
        principal,
        carrier,
        packet: session.block_packet()?,
    })
}

pub(crate) fn compute_block_scope_root_frame<Tree: LayoutTree>(
    tree: &Tree,
    request: BlockScopeRootFrameRequest<Tree::Node, Tree::Scalar>,
) -> ScopeRootFrameResult<Tree> {
    let BlockScopeRootFrameRequest {
        root,
        request,
        frame,
        axes,
        binding,
    } = request;
    let (graph, invalidated) =
        validation::validate_layout_request_with_text_graph(tree, root, &[], &[])?;
    let wrapping = crate::inline_wrapping::WrappingContext::new(
        tree,
        &graph,
        &crate::WrappingInput::default(),
        &[],
        |_| false,
    )
    .map_err(|reason| crate::inline_wrapping::wrapping_error(root, reason))?;
    let mut session = session::ComputeSession::new(tree, invalidated);
    session.bind_fragment_frame_with_intrinsics(binding.intrinsics, binding.allocations);
    session.install_block_scopes(root, binding.dispatch)?;
    session.retain_block_bodies();
    session.install_wrapping(std::sync::Arc::new(wrapping), &graph);
    root::compute_root_scoped(&mut session, root, request)?;
    root::position_fragment_root(&mut session, root, axes, frame.size)?;
    Ok(session.block_packet()?)
}

fn run_session<Tree: LayoutTree>(
    mut session: session::ComputeSession<'_, Tree>,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> ComputeLayoutResult<Tree> {
    let replay = session.has_text() || session.groups_active();
    let mut rejected_bases: Vec<(_, Option<TextPassBasis<Tree>>, _)> = Vec::new();
    loop {
        compute_root(&mut session, root, request)?;
        match request.rounding_mode() {
            crate::LayoutRoundingMode::NearestCssPixel => {
                rounding::round_layout(&mut session, root)?
            }
        }
        if !replay {
            return Ok(session.complete_for_root(root));
        }
        let frames = session.canonical_frames(root)?;
        let text = if session.has_text() {
            Some(session.text_pass_basis(root, frames.clone())?)
        } else {
            None
        };
        let groups = session.group_basis(root)?;
        let basis = (frames.clone(), text, groups.clone());
        if let Some((_, previous, _)) = rejected_bases.last() {
            session.admit_text_pass_comparison(root, basis.1.as_ref(), previous.as_ref())?;
        }
        if rejected_bases.last() == Some(&basis)
            && (!session.has_text() || frames == session.text_basis())
            && session.formatted_text_valid(root)?
            && groups.as_ref().is_none_or(|groups| groups.is_reproduced())
        {
            return if session.has_text() {
                session.complete_with_text(root)
            } else {
                Ok(session.complete_for_root(root))
            };
        }
        // The last comparison and the history search are distinct executions.
        // Admit every possible new scope-key equality before pure contains;
        // native frame/group/history-container work keeps its existing owner.
        for (_, previous, _) in &rejected_bases {
            session.admit_text_pass_comparison(root, basis.1.as_ref(), previous.as_ref())?;
        }
        if rejected_bases.contains(&basis) {
            return Err(crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(root),
                crate::LayoutOperation::TextPlacement,
                crate::LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        rejected_bases.push(basis);
        session.next_groups(&groups);
        session.begin_text_pass(root, frames)?;
    }
}

#[cfg(test)]
mod atomic_size_tests;

#[cfg(test)]
mod depth_tests;
