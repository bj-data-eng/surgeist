pub(crate) mod contracts;
mod root;
mod rounding;
mod session;
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
    let invalidated_nodes = validation::validate_layout_request(tree, root, changed_nodes)?;

    compute_validated_layout(tree, root, request, invalidated_nodes)
}

fn compute_validated_layout<Tree>(
    tree: &Tree,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
    invalidated_nodes: Vec<Tree::Node>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    let mut session = session::ComputeSession::new(tree, invalidated_nodes);
    match request.context() {
        LayoutRootContextOf::Viewport => {
            compute_root(&mut session, root, request.available())?;
        }
        LayoutRootContextOf::FlexItemUnderViewport(context) => {
            compute_flex_item_root(&mut session, root, request.available(), context)?;
        }
    }

    match request.rounding_mode() {
        crate::LayoutRoundingMode::NearestCssPixel => rounding::round_layout(&mut session, root)?,
    }

    Ok(session.complete_for_root(root))
}

/// Computes and atomically retains actual Text layouts for a checked immutable
/// capture. Old geometry caches cannot stand in for Text finalization.
pub fn compute_layout_with_text<Tree>(
    capture: &crate::TextLayoutInput<'_, Tree>,
    root: Tree::Node,
    request: LayoutRootRequestOf<Tree::Scalar>,
) -> ComputeLayoutResult<Tree>
where
    Tree: LayoutTree,
{
    compute_layout_with_text_invalidated(capture, root, request, &[])
}

/// Text computation with the ordinary source-owned invalidation closure.
pub fn compute_layout_with_text_invalidated<Tree>(
    capture: &crate::TextLayoutInput<'_, Tree>,
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
    let mut session = session::ComputeSession::with_text(capture, invalidated);
    let mut rejected_bases = Vec::new();
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
        let frames = session.canonical_frames(root)?;
        let basis = session.text_pass_basis(frames.clone());
        if frames == session.text_basis() && rejected_bases.last() == Some(&basis) {
            return session.complete_with_text(root);
        }
        if rejected_bases.contains(&basis) {
            return Err(crate::LayoutErrorOf::new(
                crate::LayoutErrorSiteOf::Node(root),
                crate::LayoutOperation::TextPlacement,
                crate::LayoutErrorKindOf::TextFeedbackCycle,
            ));
        }
        rejected_bases.push(basis);
        session.begin_text_pass(frames);
    }
}
