pub(crate) mod contracts;
mod root;
mod rounding;
mod session;
#[cfg(test)]
pub(crate) use session::ComputeSession;
pub(crate) use session::FloatChildPacket;
pub(crate) mod validation;

use crate::error::LayoutResultOf;
use crate::sizing::resolve::{ResolvedPreferredSize, resolve_preferred_sizing};
use crate::{
    BlockGeneration, CompletedLayoutBatchOf, Display, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation,
    LayoutRootContextOf, LayoutRootRequestOf, LayoutScalar, LayoutTree,
    LayoutUnsupportedCapability, Overflow, PhysicalAxis, Position, Size, SizingAlgorithm, Traverse,
};
pub(crate) use root::{compute_hidden, compute_root};
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
    let (input, _) = root::root_compute_input(&mut session, container, container_style, request)?;
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
    let mut invalidation_seeds = changed_nodes.to_vec();
    invalidation_seeds.extend(capture.invalidated.iter().copied());
    let invalidated = capture.graph.invalidation(&invalidation_seeds)?;
    let session = session::ComputeSession::with_flow(capture, invalidated);
    run_session(session, root, request)
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
            && session.formatted_text_valid()
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
        session.begin_text_pass(frames);
    }
}

#[cfg(test)]
mod atomic_size_tests;
