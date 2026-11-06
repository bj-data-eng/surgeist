use super::scroll::retained_child_scroll_geometry;
use super::sizing::{
    maximum_size, minimum_size, preferred_size, resolve_auto_optional, resolve_length_or_zero,
};
use super::{Constants, with_block_scroll_projections};
use crate::error::layout_child_geometry_error;
use crate::geometry::{LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::inline::{InlineParticipantKindOf, InlineParticipantProjection};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinSizeClampExt, OptionalSizeExt,
    OptionalSizeMaxExt, absolute_inset_alignment_offsets, absolute_inset_child_axes,
    absolute_inset_extents, absolute_stretch_fit_sizes, floor_absolute_definite_insets,
    resolve_absolute_margins,
};
use crate::scroll::ScrollContributionAccumulatorOf;
use crate::sizing::resolve::{EdgesResultExt, SizeResultExt};
use crate::{
    AxisSizingRequestOf, BoxSizing, Compute, ComputeInputOf, ContainingLayoutContext, Direction,
    LayoutResultOf, LayoutScalar, NodeOutputOf, ParentFormattingContext, Point, Position,
    RequestedAxis, RunMode, Size, SizingAlgorithm, SizingMode, Traverse,
};

/// Pre-relative flow ports for one accepted absolute occurrence.
/// Native flow can supply equal ports; Text supplies both actual frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StaticPositionOf<S: LayoutScalar> {
    pub(crate) normal: Point<S>,
    pub(crate) visual: Point<S>,
}

pub(super) fn absolute_static_position<S: LayoutScalar>(
    cursor_block: S,
    constants: &Constants<S>,
    containing_size: Size<S>,
) -> StaticPositionOf<S> {
    let point = constants.flow_axes.physical_point(
        LogicalPointOf::new(
            constants.logical_content_box_inset().inline_start,
            cursor_block,
        ),
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        containing_size,
    );
    // This ordinary producer precedes relative ancestry/corner projection in
    // either channel. Neither port is a recovery from a missing Text port.
    StaticPositionOf {
        normal: point,
        visual: point,
    }
}

pub(super) fn layout_absolute_children<'input, Tree, S, M>(
    tree: &mut Tree,
    container_node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    static_positions: &[(<Tree as Traverse>::Node, StaticPositionOf<S>)],
    container: Size<S>,
    constants: &Constants<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    <Tree as Traverse>::Node: 'input,
    S: LayoutScalar,
{
    let area_start_x = constants.effective_border.left + constants.scrollbar_gutter.left;
    let max_area_start_x =
        (container.width - constants.effective_border.right).max(constants.effective_border.left);
    let area_start_y = constants.effective_border.top + constants.scrollbar_gutter.top;
    let max_area_start_y =
        (container.height - constants.effective_border.bottom).max(constants.effective_border.top);
    let area_offset = Point::new(
        area_start_x.min(max_area_start_x),
        area_start_y.min(max_area_start_y),
    );
    let area_size = Size::new(
        (container.width
            - constants.effective_border.horizontal_sum()
            - constants.scrollbar_gutter.horizontal_sum())
        .max(S::ZERO),
        (container.height
            - constants.effective_border.vertical_sum()
            - constants.scrollbar_gutter.vertical_sum())
        .max(S::ZERO),
    );
    let available = crate::layout_math::transpose_axis_requests_at_node(
        container_node,
        area_size.map(|value| {
            crate::layout_math::checked_available_space(value)
                .map(|space| AxisSizingRequestOf::new(space, crate::SizingOperation::Normal))
        }),
    )?;

    for (source_index, child) in children.iter().copied().enumerate() {
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let InlineParticipantKindOf::Box(style) = projection.kind() else {
            continue;
        };
        if style.common.position != Position::Absolute || style.display == crate::Display::None {
            continue;
        }

        let padding = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.padding,
                area_size.map(Some),
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.border,
                area_size.map(Some),
                resolve_length_or_zero,
            )
            .transpose_with_node(tree, child)?;
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *style.common.margin,
                area_size.map(Some),
                resolve_auto_optional,
            )
            .transpose_with_node(tree, child)?;
        let padding_border = padding + border;
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border.sum_axes()
        } else {
            Size::ZERO
        };
        let min_size = minimum_size(
            style.common.min_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .or(padding_border.sum_axes().map(Some))
        .max_optional(padding_border.sum_axes().map(Some));
        let max_size = maximum_size(
            style.common.max_size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let style_size = preferred_size(
            style.common.size,
            area_size.map(Some),
            SizingAlgorithm::Positioned,
            true,
        )
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment);
        let aspect_max_size = if style.common.aspect_ratio.is_some()
            && style_size.width.is_none()
            && style_size.height.is_none()
            && max_size.width.is_some()
            && max_size.height.is_some()
        {
            max_size
        } else {
            Size::NONE
        };
        let mut known_size = style_size
            .or(aspect_max_size)
            .clamp_max_before_min_optional(min_size, max_size);
        let resolved_inset = style
            .common
            .inset
            .zip_size(area_size.map(Some), resolve_auto_optional)
            .transpose_with_node(tree, child)?;
        let inset = floor_absolute_definite_insets(resolved_inset, area_size, constants.flow_axes);
        let child_input = tree.node_input(child);
        let alignment = LogicalSizeOf::new(child_input.justify_self, child_input.align_self);
        let physical_alignment = constants.flow_axes.physical_size(alignment);
        let inset_extents = absolute_inset_extents(inset, area_size, constants.flow_axes);
        let stretch = absolute_stretch_fit_sizes(
            inset,
            unresolved_margin,
            inset_extents,
            physical_alignment,
            !child_input.item_is_replaced && !child_input.item_is_table,
        );
        if known_size.width.is_none()
            && let Some(stretch_width) = stretch.width
        {
            known_size.width = Some(stretch_width);
            known_size = known_size
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .clamp_max_before_min_optional(min_size, max_size);
        }
        if known_size.height.is_none()
            && let Some(stretch_height) = stretch.height
        {
            known_size.height = Some(stretch_height);
            known_size = known_size
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .clamp_max_before_min_optional(min_size, max_size);
        }

        let available = crate::layout_math::transpose_axis_requests_at_node(
            child,
            absolute_inset_child_axes(
                available,
                inset_extents,
                unresolved_margin,
                tree.has_leaf_measurement(child),
            ),
        )?;
        let output = tree.compute_child(
            child,
            ComputeInputOf::<S>::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known_size,
                area_size.map(Some),
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                constants
                    .sizing_request
                    .with_axes(available)
                    .with_orthogonal_fallback(
                        constants.sizing_request.orthogonal_fallback().for_child(
                            constants.positioned_fallback_limits,
                            crate::ScrollportFallbackChangeOf::Preserve,
                        ),
                    ),
            )
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars),
        )?;
        if let Some(groups) = tree.group_computation() {
            groups.child(child);
        }
        let final_size = known_size
            .unwrap_or(output.size)
            .clamp_max_before_min_optional(min_size, max_size);
        let margin = resolve_absolute_margins(
            unresolved_margin,
            inset,
            final_size,
            area_size,
            constants.flow_axes,
        );
        let static_position = static_positions
            .iter()
            .find_map(|(node, position)| (*node == child).then_some(*position))
            .unwrap_or_else(|| {
                absolute_static_position(
                    constants.logical_content_box_inset().block_start,
                    constants,
                    container,
                )
            });
        let static_x_direction =
            static_axis_direction(constants.flow_axes, PhysicalAxis::Horizontal);
        let static_y_direction = static_axis_direction(constants.flow_axes, PhysicalAxis::Vertical);
        let horizontal_flow_anchor = (inset.left.is_none() && inset.right.is_none())
            || physical_alignment.width.is_some()
            || unresolved_margin.left.is_none()
            || unresolved_margin.right.is_none()
            || (inset.left, inset.right) != (resolved_inset.left, resolved_inset.right);
        let vertical_flow_anchor = (inset.top.is_none() && inset.bottom.is_none())
            || physical_alignment.height.is_some()
            || unresolved_margin.top.is_none()
            || unresolved_margin.bottom.is_none()
            || (inset.top, inset.bottom) != (resolved_inset.top, resolved_inset.bottom);
        let inset_alignment = absolute_inset_alignment_offsets(
            inset,
            unresolved_margin,
            final_size,
            area_size,
            constants.flow_axes,
            alignment,
        );
        let locate = |point: Point<S>| {
            Point::new(
                AbsoluteAxis {
                    start: inset.left,
                    end: inset.right,
                    direction: if horizontal_flow_anchor {
                        static_x_direction
                    } else {
                        constants.direction
                    },
                    area_start: area_offset.x,
                    area_size: area_size.width,
                    size: final_size.width,
                    margin_start: margin.left,
                    margin_end: margin.right,
                    static_position: point.x,
                }
                .location()
                    + inset_alignment.x,
                AbsoluteAxis {
                    start: inset.top,
                    end: inset.bottom,
                    direction: if vertical_flow_anchor {
                        static_y_direction
                    } else {
                        Direction::Ltr
                    },
                    area_start: area_offset.y,
                    area_size: area_size.height,
                    size: final_size.height,
                    margin_start: margin.top,
                    margin_end: margin.bottom,
                    static_position: point.y,
                }
                .location()
                    + inset_alignment.y,
            )
        };
        let normal_location = locate(static_position.normal);
        let location = locate(static_position.visual);
        let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
            tree,
            child,
            |box_projection, target_projection| {
                retained_child_scroll_geometry(
                    box_projection,
                    target_projection,
                    final_size,
                    output.content_size,
                    padding,
                    border,
                    output.scroll_geometry,
                )
            },
        )
        .map_err(|error| layout_child_geometry_error(container_node, child, error))?;
        contributions
            .include_current_out_of_flow_geometry(location, margin, scroll_geometry)
            .map_err(|error| layout_child_geometry_error(container_node, child, error))?;
        if let Some(state) = tree.text_computation()
            && state.paragraphs.iter().any(|paragraph| {
                paragraph.atomics.iter().any(|binding| {
                    binding.node == child
                        && binding.occurrence.view().original_box().kind()
                            == surgeist_text::InlineBoxKind::OutOfFlow
                })
            })
        {
            let selected = output
                .baselines()
                .first_or_synthesize_block(constants.flow_axes, final_size)
                .to_f64();
            state.record_absolute_baseline(child, selected);
        }
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            child,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                final_size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index),
                location,
                size: final_size,
                named_baselines: output.named_baselines,
                content_size: output.content_size,
                border,
                padding,
                margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        ));
    }

    Ok(())
}

fn static_axis_direction(flow_axes: crate::geometry::FlowAxes, axis: PhysicalAxis) -> Direction {
    let start = if flow_axes.inline_axis() == axis {
        flow_axes.inline_start()
    } else {
        flow_axes.block_start()
    };
    if matches!(start, PhysicalSide::Right | PhysicalSide::Bottom) {
        Direction::Rtl
    } else {
        Direction::Ltr
    }
}

struct AbsoluteAxis<S: LayoutScalar> {
    start: Option<S>,
    end: Option<S>,
    direction: Direction,
    area_start: S,
    area_size: S,
    size: S,
    margin_start: S,
    margin_end: S,
    static_position: S,
}

impl<S: LayoutScalar> AbsoluteAxis<S> {
    fn location(self) -> S {
        if self.direction == Direction::Rtl
            && let (Some(_), Some(end)) = (self.start, self.end)
        {
            return self.area_start + self.area_size - end - self.size - self.margin_end;
        }

        if let Some(start) = self.start {
            self.area_start + start + self.margin_start
        } else if let Some(end) = self.end {
            self.area_start + self.area_size - end - self.size - self.margin_end
        } else if self.direction == Direction::Rtl {
            self.static_position - self.size - self.margin_end
        } else {
            self.static_position + self.margin_start
        }
    }
}
