//! Block-issued placement context for an independently scheduled child.
use super::*;

type PendingResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    Pending<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Pending<Node, S: LayoutScalar> {
    pub(crate) parent: Option<Node>,
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) axes: crate::FlowAxes,
    pub(crate) reference: Size<S>,
    pub(crate) inline_outer: S,
    pub(crate) inline_offset: S,
    pub(crate) cursor: S,
    pub(crate) occupied: S,
    pub(crate) active: CollapsibleMarginOf<S>,
    pub(crate) top: CollapsibleMarginOf<S>,
    pub(crate) first: bool,
    pub(crate) collapses_start: bool,
    pub(crate) child_collapses: bool,
    pub(crate) margin: LogicalEdgesOf<S>,
    pub(crate) padding: Edges<S>,
    pub(crate) border: Edges<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Placement<Node, S: LayoutScalar> {
    pub(crate) pending: Pending<Node, S>,
    pub(crate) reported_start: entry::StartFact<S>,
    pub(crate) location: LogicalPointOf<S>,
}

pub(crate) struct ChildRequest<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) entry: Option<Pending<Node, S>>,
}

impl<Node: Copy, S: LayoutScalar> Pending<Node, S> {
    /// Parent-local origin of a continuation carrying content without a
    /// principal. The continuation has no block extent or opening margin.
    pub(crate) fn overflow_origin(self, inline_outer: S, reference: Size<S>) -> Point<S> {
        self.axes.physical_point(
            LogicalPointOf::new(self.inline_offset, S::ZERO),
            LogicalSizeOf::new(inline_outer, S::ZERO),
            reference,
        )
    }
    pub(crate) fn resolve<M>(
        self,
        reported_start: entry::StartFact<S>,
    ) -> LayoutResultOf<Node, Placement<Node, S>, S, M> {
        let set = reported_start
            .margin
            .collapse_with_margin(self.margin.block_start);
        let start = entry::reduce_start(
            self.first,
            self.collapses_start,
            self.child_collapses,
            self.active,
            set,
        );
        let block = self.cursor + start.offset;
        if !block.is_finite() || !self.inline_offset.is_finite() {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(self.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                    value: if !block.is_finite() {
                        block
                    } else {
                        self.inline_offset
                    },
                }),
            ));
        }
        Ok(Placement {
            location: LogicalPointOf::new(self.inline_offset, block),
            pending: self,
            reported_start,
        })
    }
}

/// Reuse the ordinary inline placement reducer for a carrier in a parent with
/// no principal. Preparing its coordinate origin grants no measured block basis.
pub(crate) fn carrier_origin<'input, Tree, M>(
    tree: &mut Tree,
    parent: Tree::Node,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    context: content::Context<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Point<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let constants = &context.constants;
    let style = BlockChildProjection::from_node(tree.node_input(node));
    let mut first = false;
    let prepared = in_flow::prepare_child(
        tree,
        node,
        &style,
        constants,
        in_flow::ChildSizingContext {
            inner_size: constants
                .flow_axes
                .physical_size(constants.logical_node_inner_size()),
            request: constants.sizing_request,
            run_mode: RunMode::PerformLayout,
            axis: RequestedAxis::Both,
            first_in_flow: &mut first,
        },
    )?;
    let reference = context
        .axes()
        .physical_size(LogicalSizeOf::new(context.inline_outer, Tree::Scalar::ZERO));
    let pending = prepare(
        tree,
        parent,
        ChildRequest {
            node,
            input,
            entry: None,
        },
        &style,
        constants,
        &prepared,
        ParentEntry {
            reference,
            cursor: Tree::Scalar::ZERO,
            occupied: Tree::Scalar::ZERO,
            active: CollapsibleMarginOf::ZERO,
            top: CollapsibleMarginOf::ZERO,
            first: false,
        },
    )?;
    Ok(pending.overflow_origin(pending.inline_outer, reference))
}

pub(super) struct ParentEntry<S: LayoutScalar> {
    pub(super) reference: Size<S>,
    pub(super) cursor: S,
    pub(super) occupied: S,
    pub(super) active: CollapsibleMarginOf<S>,
    pub(super) top: CollapsibleMarginOf<S>,
    pub(super) first: bool,
}

pub(super) fn prepare<'input, Tree, M>(
    tree: &mut Tree,
    parent: Tree::Node,
    child: ChildRequest<Tree::Node, Tree::Scalar>,
    style: &BlockChildProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    prepared: &in_flow::PreparedChild<Tree::Scalar>,
    state: ParentEntry<Tree::Scalar>,
) -> PendingResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    let ChildRequest {
        node: child, input, ..
    } = child;
    let inline = tree.compute_axis_demand(
        child,
        input.with_run_mode(RunMode::ComputeSize),
        constants.flow_axes.inline_axis(),
    )?;
    let available = constants.logical_node_inner_size().inline.or(constants
        .flow_axes
        .logical_size(constants.sizing_request.axes())
        .inline
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get));
    let margin = in_flow::resolve_logical_inline_margin(
        constants
            .flow_axes
            .logical_edges(prepared.unresolved_margin),
        inline,
        available,
    );
    let inline_offset = in_flow::in_flow_child_inline_offset(inline, margin, constants);
    Ok(Pending {
        parent: Some(parent),
        node: child,
        input,
        axes: constants.flow_axes,
        reference: state.reference,
        inline_outer: inline,
        inline_offset,
        cursor: state.cursor,
        occupied: state.occupied,
        active: state.active,
        top: state.top,
        first: state.first,
        collapses_start: constants.collapse_top_margin,
        child_collapses: in_flow::child_margin_can_collapse_with_parent(style),
        margin,
        padding: prepared.padding,
        border: prepared.border,
    })
}

/// Root placement has no preceding sibling or collapsed external margin.
pub(crate) fn root<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> PendingResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    let context = content::prepare(tree, node, input, tree.node_input(node))?;
    let constants = context.constants;
    Ok(Pending {
        parent: None,
        node,
        input,
        axes: constants.flow_axes,
        reference: input
            .parent()
            .map(|value| value.unwrap_or(Tree::Scalar::ZERO)),
        inline_outer: context.inline_outer,
        inline_offset: Tree::Scalar::ZERO,
        cursor: Tree::Scalar::ZERO,
        occupied: Tree::Scalar::ZERO,
        active: CollapsibleMarginOf::ZERO,
        top: CollapsibleMarginOf::ZERO,
        first: false,
        collapses_start: false,
        child_collapses: false,
        margin: LogicalEdgesOf::new(
            Tree::Scalar::ZERO,
            Tree::Scalar::ZERO,
            Tree::Scalar::ZERO,
            Tree::Scalar::ZERO,
        ),
        padding: constants.padding,
        border: constants.effective_border,
    })
}
