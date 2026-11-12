use super::absolute::{StaticPositionGeometry, absolute_static_position};
use super::inline_run::{
    InlineRunContext, InlineRunTransitionState, VisibleInlineRunTransition, inline_run_end,
    visible_inline_boundary_in_flow, visible_line_break_in_flow,
};
use super::scroll::{BlockScrollContributions, retained_child_scroll_geometry};
use super::sizing::{maximum_size, minimum_size, preferred_size, resolve_length_or_zero};
use super::*;
use crate::AxisSizingRequestOf;

mod child_compute;
mod placement;
mod traversal;

pub(crate) use traversal::{InFlowChild, PendingBox};
pub(super) use traversal::{InFlowStep, InFlowTraversal};

use child_compute::{
    InFlowBoxComputation, InFlowBoxPreparation, InFlowComputedBox, PendingSharedBox,
    finish_requested_box, finish_shared_box, prepare_in_flow_box, prepare_shared_box,
};
use placement::{
    InFlowBoxOrigins, InFlowBoxPlacement, InFlowChildMargins, InFlowFloatItem, InFlowFloatPass,
    InFlowFloatState, SharedInFlowPlacement, begin_in_flow_child_margins, place_in_flow_box,
    place_in_flow_float,
};

/// Raw baseline or float work needed even by an otherwise definite size query.
/// This does not establish named absence or discharge a named-fact demand.
pub(super) fn normal_flow_children_need_physical_sizing_work<'input, Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute<'input, M>,
{
    children.iter().copied().any(|child| {
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let style = match projection.kind() {
            InlineParticipantKindOf::InlineText(_) => return true,
            InlineParticipantKindOf::TextControl(_) => return true,
            InlineParticipantKindOf::Box(style) => style,
            InlineParticipantKindOf::LineBreak(input) => {
                return !input.display().is_none()
                    && input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
            InlineParticipantKindOf::InlineBoundary(input) => {
                return input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
        };
        if style.display == crate::Display::None || style.common.position == Position::Absolute {
            return false;
        }
        !style.float.is_none()
            || style.display.is_inline_level()
            || style.display.inner_display() == crate::Display::Block
            || tree.child_count(child) > 0 && style.display.inner_display() == crate::Display::Flex
    })
}

pub(super) struct InFlowResult<Node, S: LayoutScalar> {
    pub(super) fragment_margins: Option<super::fragment_margins::Flow>,
    pub(super) float_context: FloatExclusions<S, Node>,
    pub(super) inherited_floats: usize,
    pub(super) remaining_cut: Option<entry::canonical::Separation<Node, S>>,
    pub(super) groups: Vec<crate::group_alignment::Member<Node, S>>,
    pub(super) formatted: Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    pub(super) intrinsic_inline_demand: S,
    pub(super) owned_float_block_end: S,
    pub(super) resolved_terminal_float_block_end: Option<S>,
    pub(super) contributions: BlockScrollContributions<S>,
    pub(super) baselines: super::baseline_frame::PendingBaselines,
    pub(super) static_positions: Vec<(Node, StaticPositionOf<S>)>,
    pub(super) pending_floats: Vec<PendingFloat<Node, S>>,
    pub(super) cursor_block: S,
    pub(super) occupied_block_end: S,
    pub(super) participated: bool,
    pub(super) top_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin_at_start: bool,
    pub(super) active_margin_can_collapse_with_parent: bool,
    pub(super) all_in_flow_children_can_collapse_through: bool,
}

impl<Node, S: LayoutScalar> InFlowResult<Node, S> {
    pub(super) fn top_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    pub(super) fn bottom_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if self.collapses_end_margin(constants) {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    pub(super) fn collapses_through(&self, constants: &Constants<S>) -> bool {
        constants.can_collapse_through && self.all_in_flow_children_can_collapse_through
    }

    pub(super) fn collapses_end_margin(&self, constants: &Constants<S>) -> bool {
        constants.collapse_bottom_margin
            && self.active_margin_can_collapse_with_parent
            && (!self.active_margin_at_start || self.collapses_through(constants))
    }

    pub(super) fn content_end(&self, constants: &Constants<S>, role: float_context::Role) -> S {
        let bottom_margin_offset =
            if self.active_margin_at_start || self.collapses_end_margin(constants) {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let in_flow_block_end = self.cursor_block + bottom_margin_offset;
        let used_line_end = self
            .resolved_terminal_float_block_end
            .unwrap_or(in_flow_block_end);
        if role == float_context::Role::Owner {
            in_flow_block_end
                .max(used_line_end)
                .max(self.owned_float_block_end)
        } else {
            in_flow_block_end.max(used_line_end)
        }
    }

    pub(super) fn auto_block(&self, constants: &Constants<S>, role: float_context::Role) -> S {
        let inset = constants.logical_content_box_inset();
        (self.content_end(constants, role) + inset.block_end).max(inset.block_sum())
    }
}

pub(super) struct InFlowPassContext<'a, S: LayoutScalar, Node> {
    pub(super) inline_operation: crate::SizingOperation,
    pub(super) inner_inline: Option<S>,
    pub(super) set_layout: bool,
    pub(super) inherited: Option<&'a InheritedFloatExclusions<S, Node>>,
    pub(super) projection: Option<content::Projection<S>>,
}

enum InFlowChildStart<'input, Node, S: LayoutScalar> {
    VisibleInlineRun(VisibleInlineRunTransition),
    FlowBox {
        child: Node,
        style: BlockChildProjection<'input, S>,
    },
    Hidden {
        child: Node,
        compute_box: bool,
    },
    Positioned {
        child: Node,
        geometry: StaticPositionGeometry,
    },
}

type InFlowChildStartResult<'input, Node, S, M> =
    LayoutResultOf<Node, InFlowChildStart<'input, Node, S>, S, M>;

fn start_native_child<'input, Tree, S, M>(
    tree: &mut Tree,
    node: Tree::Node,
    children: &[Tree::Node],
    constants: &Constants<S>,
    index: usize,
    semantic_end: usize,
) -> InFlowChildStartResult<'input, Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let child = children[index];
    let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
    // The captured run owns transparent-leading children and their effects.
    let captured_start = tree
        .text_computation()
        .is_some_and(|state| state.run_starting(node, &children[index..]).is_some());
    if !captured_start {
        match projection.kind() {
            InlineParticipantKindOf::Box(style) => {
                if style.display == crate::Display::None {
                    return Ok(InFlowChildStart::Hidden {
                        child,
                        compute_box: true,
                    });
                }
                if style.common.position == Position::Absolute {
                    let geometry = if style.display.is_inline_level() {
                        StaticPositionGeometry::Point
                    } else {
                        StaticPositionGeometry::BlockStart
                    };
                    return Ok(InFlowChildStart::Positioned { child, geometry });
                }
                if !crate::inline::starts_run(tree.layout_input(child)) {
                    return Ok(InFlowChildStart::FlowBox { child, style });
                }
            }
            InlineParticipantKindOf::InlineText(_) | InlineParticipantKindOf::TextControl(_) => {}
            InlineParticipantKindOf::LineBreak(line_break) => {
                if line_break.display().is_none() {
                    return Ok(InFlowChildStart::Hidden {
                        child,
                        compute_box: false,
                    });
                }
                visible_line_break_in_flow(tree, node, child, constants.flow_axes)?;
            }
            InlineParticipantKindOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(tree, node, child, constants.flow_axes)?;
            }
        }
    }
    let end = inline_run_end(
        tree,
        node,
        children,
        constants,
        index + 1,
        semantic_end,
        None,
    )?;
    Ok(InFlowChildStart::VisibleInlineRun(
        VisibleInlineRunTransition::new(index, end),
    ))
}

pub(super) fn layout_in_flow_children<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    pass: InFlowPassContext<'_, S, <Tree as Traverse>::Node>,
) -> ComputeResult<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let mut traversal = InFlowTraversal::begin(tree, node, constants, input, pass)?;
    loop {
        match traversal.step(tree, children, constants)? {
            InFlowStep::Continue => {}
            InFlowStep::Complete => return traversal.finish(tree),
            InFlowStep::Child(child) => {
                let (pending, output) = child.compute(tree)?;
                traversal.accept_child(tree, constants, pending, output)?;
            }
        }
    }
}

pub(super) fn child_margin_can_collapse_with_parent<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
) -> bool {
    style.display == crate::Display::Block && style.common.position == Position::Relative
}

pub(super) struct PreparedChild<S: LayoutScalar> {
    pub(super) unresolved_margin: Edges<ResolvedLengthAutoOf<S>>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) parent_size: Size<Option<S>>,
    pub(super) available_inline: Option<S>,
    pub(super) logical_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    pub(super) input: ComputeInputOf<S>,
}

pub(super) struct ChildSizingContext<'flow, S: LayoutScalar> {
    pub(super) inner_size: Size<Option<S>>,
    pub(super) request: crate::SizingRequestOf<S>,
    pub(super) run_mode: RunMode,
    pub(super) axis: RequestedAxis,
    pub(super) first_in_flow: &'flow mut bool,
}

pub(super) fn prepare_child<'input, Tree, M>(
    tree: &Tree,
    child: Tree::Node,
    child_style: &BlockChildProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: ChildSizingContext<'_, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, PreparedChild<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let ChildSizingContext {
        inner_size: node_inner_size,
        request: pass_request,
        run_mode,
        axis: requested_axis,
        first_in_flow,
    } = context;
    let indent = crate::text_indent::BlockIndentContext {
        inherited: constants.text_indent,
        first_formatted: *first_in_flow && constants.first_formatted,
    };
    // Hidden and absolute children never enter this flow-box preparation.
    // Floats retain the ordinary parent's first formatted-box opportunity.
    if child_style.float.is_none() {
        *first_in_flow = false;
    }
    let unresolved_margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
        *child_style.common.margin,
        node_inner_size,
        |length, basis| length.resolve_auto_with_status(basis),
    );
    let child_padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *child_style.common.padding,
            node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let child_border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *child_style.common.border,
            node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let child_flow_axes = child_style.common.flow_axes;
    let child_parent_size = constants.child_containing_block_size(child_flow_axes);
    let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
    let mut child_logical_available = child_flow_axes.logical_size(pass_request.axes());
    if child_style.float.is_none()
        && child_flow_axes.inline_axis() != constants.flow_axes.inline_axis()
        && child_logical_node_inner_size.inline.is_none()
    {
        // An auto block extent supplies indefinite inline space to an
        // orthogonal in-flow child. Its fallback stays on the request.
        child_logical_available.inline =
            in_flow_child_block_request(child_logical_available.inline);
    }
    let child_non_auto_margin = child_flow_axes
        .logical_edges(unresolved_margin)
        .map(resolved_length_auto_fallback_zero);
    let available_child_inline = child_logical_node_inner_size
        .inline
        .or(child_logical_available
            .inline
            .space()
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get))
        .map(|inline| {
            crate::layout_math::remaining_available_extent(
                inline,
                child_non_auto_margin.inline_sum(),
            )
        })
        .transpose()
        .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
    let child_known = in_flow_child_known_size::<Tree, M>(
        tree,
        child,
        child_style,
        child_padding + child_border,
        child_flow_axes,
        child_logical_node_inner_size,
        (child_logical_available.inline.operation() == crate::SizingOperation::Normal)
            .then_some(available_child_inline)
            .flatten(),
    )?;
    let input = ComputeInputOf::<Tree::Scalar>::for_child(
        run_mode,
        SizingMode::InherentSize,
        requested_axis,
        child_known,
        child_parent_size,
        ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
        pass_request.with_axes(
            child_flow_axes.physical_size(LogicalSizeOf::new(
                in_flow_child_inline_request(
                    child_style,
                    tree.has_leaf_measurement(child),
                    child_flow_axes,
                    available_child_inline,
                    child_logical_available.inline,
                )
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?,
                in_flow_child_block_request(child_logical_available.block),
            )),
        ),
    )
    .with_block_indent_context(indent);
    Ok(PreparedChild {
        unresolved_margin,
        padding: child_padding,
        border: child_border,
        parent_size: child_parent_size,
        available_inline: available_child_inline,
        logical_available: child_logical_available,
        input,
    })
}

#[expect(
    clippy::type_complexity,
    reason = "the private child-size helper preserves the session's generic error envelope"
)]
pub(super) fn in_flow_child_known_size<'input, Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    style: &BlockChildProjection<'_, Tree::Scalar>,
    padding_border: Edges<Tree::Scalar>,
    child_flow_axes: crate::geometry::FlowAxes,
    parent: LogicalSizeOf<Option<Tree::Scalar>>,
    available_inline: Option<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let parent = child_flow_axes.physical_size(parent);
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
    let measured = tree.has_leaf_measurement(child);
    let ratio = if !measured && style.flex_owns_preferred_inline {
        None
    } else {
        *style.common.aspect_ratio
    };
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = minimum_size(style.common.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?;
    let max_size = maximum_size(style.common.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?;
    let ratio_transferred_in_block =
        normal_block_inline_ratio(&style.common, style.display, style.float)
            .filter(|_| ratio.is_some() && available_inline.is_some() && !measured);
    let (min_size, max_size) = if let Some(ratio) = ratio_transferred_in_block {
        transfer_block_ratio_inline_limits(child_flow_axes, ratio, min_size, max_size)
            .map_err(|status| crate::error::sizing_resolution_error(child, status.into()))?
    } else {
        (min_size.apply_aspect_ratio(ratio), max_size)
    };
    let min_size = super::fragment_minimum::apply(
        tree.fragment_minimum(child),
        min_size.add_optional(box_sizing_adjustment),
        padding_border.sum_axes(),
    );
    let mut max_size = max_size.add_optional(box_sizing_adjustment);
    let aspect_height_limit = if ratio_transferred_in_block.is_some() {
        None
    } else {
        ratio
            .zip(max_size.height)
            .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()))
    };
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = preferred_size(style.common.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_max_before_min_optional(min_size, max_size);

    let mut known = child_flow_axes.logical_size(known);
    let min_size = child_flow_axes.logical_size(min_size);
    let max_size = child_flow_axes.logical_size(max_size);
    if !style.common.item_is_table
        && !style.common.item_is_replaced
        && known.inline.is_none()
        && !inline_size.is_min_content()
        && !inline_size.is_max_content()
    {
        known.inline = available_inline
            .map(|inline| inline.clamp_max_before_min_optional(min_size.inline, max_size.inline));
        if aspect_height_limit.is_some() {
            let physical_known = child_flow_axes.physical_size(known);
            known = child_flow_axes.logical_size(
                physical_known
                    .apply_aspect_ratio(ratio)
                    .clamp_max_before_min_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

pub(super) fn in_flow_child_block_request<S: LayoutScalar>(
    inherited: AxisSizingRequestOf<S>,
) -> AxisSizingRequestOf<S> {
    if inherited.operation() == crate::SizingOperation::Normal {
        inherited.with_space(crate::AvailableSpaceOf::Indefinite)
    } else {
        inherited
    }
}

pub(super) fn in_flow_child_inline_request<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    measured: bool,
    child_flow_axes: crate::geometry::FlowAxes,
    available_inline: Option<S>,
    inherited: AxisSizingRequestOf<S>,
) -> Result<AxisSizingRequestOf<S>, crate::LengthResolutionStatus<S>> {
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
    let request = if inline_size.is_min_content() {
        inherited.with_operation(crate::SizingOperation::MinContent)
    } else if inline_size.is_max_content() && (measured || !style.flex_owns_preferred_inline) {
        inherited.with_operation(crate::SizingOperation::MaxContent)
    } else {
        inherited
    };
    crate::layout_math::with_available_extent(request, available_inline)
}

pub(super) fn relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: crate::geometry::FlowAxes,
) -> Point<S> {
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_offset = LogicalPointOf::new(
        logical_inset
            .inline_start
            .or_else(|| logical_inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        logical_inset
            .block_start
            .or_else(|| logical_inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    );
    flow_axes.physical_point(
        logical_offset,
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        Size::ZERO,
    )
}

pub(crate) fn resolve_logical_in_flow_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_size: LogicalSizeOf<S>,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    resolve_logical_inline_margin(margin, child_size.inline, container_inline)
}

pub(super) fn resolve_logical_inline_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_inline: S,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    let non_auto_inline = resolved_length_auto_fallback_zero(margin.inline_start)
        + resolved_length_auto_fallback_zero(margin.inline_end);
    let auto_count = usize::from(matches!(margin.inline_start, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.inline_end, ResolvedLengthAutoOf::Auto));
    let auto_inline = if auto_count == 0 {
        S::ZERO
    } else {
        container_inline
            .map(|inline| (inline - child_inline - non_auto_inline).max(S::ZERO))
            .unwrap_or(S::ZERO)
            / S::from_usize(auto_count)
    };

    LogicalEdgesOf::new(
        resolved_length_auto_or(margin.inline_start, auto_inline),
        resolved_length_auto_or(margin.inline_end, auto_inline),
        resolved_length_auto_fallback_zero(margin.block_start),
        resolved_length_auto_fallback_zero(margin.block_end),
    )
}

fn resolved_length_auto_or<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>, auto_fallback: S) -> S {
    match value {
        ResolvedLengthAutoOf::Auto => auto_fallback,
        ResolvedLengthAutoOf::Resolved(value) => value,
        // Missing-basis symbolic margins keep the algorithm's historical
        // unresolved-as-zero fallback and do not participate in auto distribution.
        ResolvedLengthAutoOf::Unresolved(
            UnresolvedLengthReason::Basis | UnresolvedLengthReason::InvalidNumeric,
        ) => S::ZERO,
    }
}

pub(super) fn resolved_length_auto_fallback_zero<S: LayoutScalar>(
    value: ResolvedLengthAutoOf<S>,
) -> S {
    resolved_length_auto_or(value, S::ZERO)
}

pub(super) fn in_flow_child_inline_offset<S: LayoutScalar>(
    child_inline: S,
    margin: LogicalEdgesOf<S>,
    constants: &Constants<S>,
) -> S {
    let logical_content_box_inset = constants.logical_content_box_inset();
    let logical_inner_size = constants.logical_node_inner_size();
    let mut inline = logical_content_box_inset.inline_start + margin.inline_start;

    let container_inner_inline = logical_inner_size
        .inline
        .or_else(|| {
            constants
                .logical_node_outer_size()
                .inline
                .map(|inline| inline - logical_content_box_inset.inline_sum())
        })
        .unwrap_or(child_inline + margin.inline_sum());
    let item_outer_inline = child_inline + margin.inline_sum();
    if item_outer_inline < container_inner_inline {
        let free_space = container_inner_inline - item_outer_inline;
        match constants.text_align {
            TextAlign::Start
            | TextAlign::End
            | TextAlign::Left
            | TextAlign::Right
            | TextAlign::Center
            | TextAlign::Justify => {}
            TextAlign::LegacyCenter => inline = inline + free_space / S::from_f64(2.0),
            TextAlign::LegacyLeft
                if constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyRight
                if !constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyLeft | TextAlign::LegacyRight => {}
        }
    }

    inline
}

pub(super) fn signed_inline_demand<S: LayoutScalar>(
    inline: S,
    margin: LogicalEdgesOf<S>,
    margin_inline_end: S,
) -> S {
    (inline + margin.inline_sum()).max(margin_inline_end)
}
