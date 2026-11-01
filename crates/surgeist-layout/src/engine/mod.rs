pub(crate) mod contracts;
mod root;
mod rounding;
mod session;
#[cfg(test)]
pub(crate) use session::ComputeSession;
pub(crate) use session::FloatChildPacket;
pub(crate) mod validation;

use crate::error::LayoutResultOf;
use crate::{
    CompletedLayoutBatchOf, LayoutRootContextOf, LayoutRootRequestOf, LayoutTree, Traverse,
};
pub(crate) use root::{compute_flex_item_root, compute_hidden, compute_root};
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
    let wrapping = crate::inline_wrapping::WrappingContext::new(tree, &graph, &[], &[], &[], &[])
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
        match request.context() {
            LayoutRootContextOf::Viewport => {
                compute_root(&mut session, root, request.available())?;
            }
            LayoutRootContextOf::FlexItemUnderViewport(context) => {
                compute_flex_item_root(&mut session, root, request.available(), context)?;
            }
        }
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
