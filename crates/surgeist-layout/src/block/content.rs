//! Descendant flow production, independent of principal-box materialization.
use super::*;

type ReceiptResult<Tree, M> = ComputeResult<
    <Tree as Traverse>::Node,
    Receipt<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Limits<S: LayoutScalar> {
    pub(crate) minimum: S,
    pub(crate) maximum: S,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Projection<S: LayoutScalar> {
    pub(crate) block_reference: S,
    pub(crate) content_origin: S,
}

/// Canonical child sizing authority. Projection references never enter this
/// context's known size or percentage-parent classification.
#[derive(Clone, Copy)]
pub(crate) struct Context<S: LayoutScalar> {
    pub(super) constants: Constants<S>,
    input: ComputeInputOf<S>,
    pub(crate) inline_outer: S,
    pub(crate) limits: Option<Limits<S>>,
    original_insets: LogicalEdgesOf<S>,
}

impl<S: LayoutScalar> Context<S> {
    pub(super) fn input(&self) -> ComputeInputOf<S> {
        self.input
    }
    pub(crate) fn axes(&self) -> crate::FlowAxes {
        self.constants.flow_axes
    }
    pub(crate) fn opening(&self) -> S {
        self.constants.logical_content_box_inset().block_start
    }
    pub(crate) fn original_closing(&self) -> S {
        self.original_insets.block_end
    }
}

/// Actual accepted content allocation, independent of a principal cap.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AcceptedContent<S: LayoutScalar> {
    pub(crate) frame: usize,
    pub(crate) natural: S,
    pub(crate) allocated: S,
    pub(crate) opening: S,
    pub(crate) participated: bool,
    pub(crate) complete: bool,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct PrincipalSpan<S: LayoutScalar> {
    pub(crate) frame: usize,
    pub(crate) opening: S,
    pub(crate) content: S,
    pub(crate) gap: S,
    pub(crate) remaining: S,
}
pub(crate) struct PrincipalTimeline<S: LayoutScalar> {
    pub(crate) spans: Vec<PrincipalSpan<S>>,
    pub(crate) body_extent: S,
}

/// Project allocations only after the shared scheduler accepts them. Closing
/// edges are a separate parent-flow obligation and are absent from this body.
pub(crate) fn principal_timeline<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    limits: Limits<S>,
    accepted: &[AcceptedContent<S>],
) -> LayoutResultOf<Node, PrincipalTimeline<S>, S, M> {
    let mut remaining = limits.maximum;
    let mut opening_pending = true;
    let mut body_extent = S::ZERO;
    let mut spans = Vec::new();
    spans.try_reserve(accepted.len()).map_err(|_| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::Allocation,
        )
    })?;
    for frame in accepted {
        let participates = frame.participated || frame.complete;
        if !participates || remaining == S::ZERO && !opening_pending {
            continue;
        }
        let opening = if opening_pending {
            frame.opening
        } else {
            S::ZERO
        };
        let content = frame.allocated.min(remaining);
        let gap = (content - frame.natural).max(S::ZERO);
        remaining = (remaining - content).max(S::ZERO);
        opening_pending = false;
        body_extent = body_extent + opening + content;
        if !body_extent.is_finite() {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::BlockFragmentation(
                    crate::BlockFragmentationFailure::InvalidGeometry,
                ),
            ));
        }
        spans.push(PrincipalSpan {
            frame: frame.frame,
            opening,
            content,
            gap,
            remaining,
        });
        if remaining == S::ZERO || frame.complete {
            break;
        }
    }
    Ok(PrincipalTimeline { spans, body_extent })
}

pub(crate) struct Receipt<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) role: crate::BlockGeneration,
    pub(super) binding: Option<super::content_insets::ContentInsetBinding<Node, S>>,
    pub(super) normal_complete: bool,
    pub(super) accepted_inline_inner: S,
    pub(super) axes: crate::FlowAxes,
    pub(super) flow: in_flow::InFlowResult<Node, S>,
    pub(crate) block_extent: S,
    pub(crate) occupied: S,
    pub(crate) participated: bool,
    pub(crate) bounds: Option<crate::ScrollRectOf<S>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Facts<S: LayoutScalar> {
    pub(crate) block_extent: S,
    pub(crate) occupied: S,
    pub(crate) participated: bool,
}
impl<Node, S: LayoutScalar> Receipt<Node, S> {
    pub(crate) fn facts(&self) -> Facts<S> {
        Facts {
            block_extent: self.block_extent,
            occupied: self.occupied,
            participated: self.participated,
        }
    }
}

pub(crate) fn prepare<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    original: &crate::NodeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Context<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = BlockContainerProjection::from_node(original);
    let contained =
        containment_sizing(node, original, input, None)?.map_or(Size::NONE, |value| value.outer);
    let canonical = Constants::from_projection::<Tree, M>(tree, node, &style, input, contained)?;
    let maximum = match canonical.flow_axes.block_axis() {
        PhysicalAxis::Horizontal => &original.max_size.width,
        PhysicalAxis::Vertical => &original.max_size.height,
    };
    let fixed = crate::sizing::resolve::fixed_fallback_maximum(
        maximum,
        SizingAlgorithm::Block,
        canonical.flow_axes.block_axis(),
        None,
    )
    .map_err(|error| crate::error::sizing_resolution_error(node, error))?;
    let inset = canonical.logical_content_box_inset().block_sum();
    let minimum = canonical
        .logical_node_min_size()
        .block
        .map_or(Tree::Scalar::ZERO, |value| {
            (value - inset).max(Tree::Scalar::ZERO)
        });
    let limits = if matches!(fixed, crate::sizing::resolve::FixedFallbackSource::Fixed(_)) {
        canonical.logical_node_max_size().block.map(|maximum| {
            let outer = canonical.logical_node_outer_size().block.unwrap_or(
                maximum.max(
                    canonical
                        .logical_node_min_size()
                        .block
                        .unwrap_or(Tree::Scalar::ZERO),
                ),
            );
            Limits {
                minimum,
                maximum: (outer - inset).max(Tree::Scalar::ZERO).max(minimum),
            }
        })
    } else {
        None
    };
    let inline_outer =
        intrinsic::accepted_inline(tree, node, &style, &canonical, input, |tree, op| {
            if let Some(value) = tree.fragment_inline_content(node, input, op)? {
                Ok(value)
            } else {
                intrinsic::content(tree, node, &canonical, input, op)
            }
        })?;
    let inner = canonical.accepted_inline_inner(node, inline_outer)?;
    let current = block_container_projection::<Tree, M>(tree, node);
    let mut constants =
        Constants::from_projection::<Tree, M>(tree, node, &current, input, contained)?;
    constants.node_outer_size = canonical.node_outer_size;
    constants.node_inner_size = canonical.node_inner_size;
    constants.child_containing_block_extent = canonical.child_containing_block_extent;
    constants.node_min_size = canonical.node_min_size;
    constants.node_max_size = canonical.node_max_size;
    constants.sizing_request = canonical.sizing_request;
    constants.positioned_fallback_limits = canonical.positioned_fallback_limits;
    let constants = constants.with_logical_node_inner_size(
        LogicalSizeOf::new(Some(inner), constants.logical_node_inner_size().block),
        false,
    );
    Ok(Context {
        constants,
        input,
        inline_outer,
        limits,
        original_insets: canonical.logical_content_box_inset(),
    })
}

pub(crate) fn produce<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    context: &Context<Tree::Scalar>,
    projection: Projection<Tree::Scalar>,
    binding: Option<super::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar>>,
) -> ReceiptResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    let constants = &context.constants;
    let children = tree.children(node).collect::<Vec<_>>();
    let flow = in_flow::layout_in_flow_children(
        tree,
        node,
        &children,
        constants,
        context.input,
        InFlowPassContext {
            inline_operation: crate::SizingOperation::Normal,
            inner_inline: constants.logical_node_inner_size().inline,
            set_layout: context.input.run_mode().is_perform_layout(),
            inherited: None,
            projection: Some(projection),
        },
    )?;
    if let Some(flow_margins) = flow.fragment_margins
        && let Some(state) = tree.fragment_margin_state()
    {
        state
            .record(
                node,
                constants.flow_axes,
                flow_margins,
                constants.collapse_top_margin,
                constants.collapse_bottom_margin && flow.active_margin_can_collapse_with_parent,
            )
            .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
    }
    let block_extent = flow.content_end(constants, float_context::Role::Owner);
    let block_extent = (block_extent - projection.content_origin).max(Tree::Scalar::ZERO);
    let bounds = flow
        .contributions
        .geometry
        .descendant_bounds()
        .map_err(|error| layout_own_geometry_error(node, context.input.run_mode(), error))?;
    Ok(Receipt {
        node,
        role: tree.node_input(node).block_generation,
        binding,
        normal_complete: context.input.run_mode().is_perform_layout(),
        accepted_inline_inner: constants.accepted_inline_inner(node, context.inline_outer)?,
        axes: context.axes(),
        participated: flow.participated,
        occupied: flow.occupied_block_end,
        flow,
        block_extent,
        bounds,
    })
}

/// Complete visual placement only for an adopted content window that has no
/// principal box. The receipt proves inline extent, not a block-size basis.
pub(crate) fn settle_accepted_insets<'input, Tree, M>(
    tree: &mut Tree,
    receipt: &mut Receipt<Tree::Node, Tree::Scalar>,
    expected: &super::content_insets::ContentInsetBinding<Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let basis = super::content_insets::accepted_inline_basis::<_, _, M>(receipt, expected)?;
    let children = tree.children(receipt.node).collect::<Vec<_>>();
    super::relative::settle_content_children(
        tree,
        receipt.node,
        &children,
        &basis,
        expected,
        &mut receipt.flow.contributions,
    )?;
    receipt.bounds = receipt
        .flow
        .contributions
        .geometry
        .descendant_bounds()
        .map_err(|error| layout_own_geometry_error(receipt.node, RunMode::PerformLayout, error))?;
    Ok(())
}

/// Materialize a selected principal span from the same descendant production.
/// The content allocation is a selected span, separate from authored sizing
/// authority retained in the context used to lay out its descendants.
pub(crate) fn materialize<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    context: &Context<Tree::Scalar>,
    receipt: Receipt<Tree::Node, Tree::Scalar>,
    content_allocation: Tree::Scalar,
) -> ComputeResult<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let constants = &context.constants;
    let style = block_container_projection::<Tree, M>(tree, node);
    let children = tree.children(node).collect::<Vec<_>>();
    let block = content_allocation + constants.logical_content_box_inset().block_sum();
    let output_size = constants
        .flow_axes
        .physical_size(LogicalSizeOf::new(context.inline_outer, block));
    finish_flow(
        tree,
        node,
        context.input,
        &style,
        &children,
        constants,
        SizedFlow {
            size: output_size,
            flow: receipt.flow,
        },
    )
    .map(|result| result.output)
}

#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
