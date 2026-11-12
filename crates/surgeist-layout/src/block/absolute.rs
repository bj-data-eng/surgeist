use super::scroll::retained_child_scroll_geometry;
use super::sizing::{maximum_size, minimum_size, resolve_auto_optional, resolve_length_or_zero};
use super::{Constants, with_block_scroll_projections};
use crate::error::layout_child_geometry_error;
use crate::geometry::{
    LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalProgression, PhysicalSide,
};
use crate::inline::{InlineParticipantKindOf, InlineParticipantProjection};
use crate::layout_math::{
    AbsoluteInsetStartOf, OptionalSizeExt, PositionedSizeDeclarations, PositionedUsedSize,
    absolute_inset_alignment_offsets, absolute_inset_child_axes, absolute_inset_extents,
    absolute_stretch_fit_sizes, floor_absolute_definite_insets, resolve_absolute_margins,
};
use crate::scroll::{ScrollContributionAccumulatorOf, ScrollOriginAxes, ScrollOriginProgression};
use crate::sizing::resolve::{EdgesResultExt, SizeResultExt, resolve_child_preferred_size};
use crate::{
    AlignItems, AxisSizingRequestOf, Compute, ComputeInputOf, ContainingLayoutContext, Direction,
    Edges, LayoutResultOf, LayoutScalar, NodeOutputOf, ParentFormattingContext, Point, Position,
    RequestedAxis, RunMode, Size, SizingAlgorithm, SizingMode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StaticPositionGeometry {
    BlockStart,
    Point,
}

/// Pre-relative flow ports for one accepted absolute occurrence.
/// Native flow can supply equal ports; Text supplies both actual frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StaticPositionOf<S: LayoutScalar> {
    pub(crate) normal: Point<S>,
    pub(crate) visual: Point<S>,
    geometry: StaticPositionGeometry,
}

impl<S: LayoutScalar> StaticPositionOf<S> {
    pub(crate) fn from_points(normal: Point<S>, visual: Point<S>) -> Self {
        Self {
            normal,
            visual,
            geometry: StaticPositionGeometry::Point,
        }
    }
}

pub(super) fn absolute_static_position<S: LayoutScalar>(
    cursor_block: S,
    constants: &Constants<S>,
    containing_size: Size<S>,
    geometry: StaticPositionGeometry,
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
        geometry,
    }
}

pub(super) struct AbsoluteChildren<S: LayoutScalar> {
    container: Size<S>,
    area_offset: Point<S>,
    area_size: Size<S>,
    available: Size<AxisSizingRequestOf<S>>,
    scroll_progressions: LogicalSizeOf<Option<ScrollOriginProgression>>,
    next: usize,
}

pub(crate) struct AbsoluteChild<Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) input: ComputeInputOf<S>,
    source_index: usize,
    padding: Edges<S>,
    border: Edges<S>,
    unresolved_margin: Edges<Option<S>>,
    used_size: PositionedUsedSize<S>,
    known_size: Size<Option<S>>,
    resolved_inset: Edges<Option<S>>,
    inset: Edges<Option<S>>,
    alignment: LogicalSizeOf<Option<AlignItems>>,
    physical_alignment: Size<Option<AlignItems>>,
    static_position: StaticPositionOf<S>,
    inset_extents: Size<Option<S>>,
}

type NextAbsoluteChild<Node, S, M> = LayoutResultOf<Node, Option<AbsoluteChild<Node, S>>, S, M>;

impl<S: LayoutScalar> AbsoluteChildren<S> {
    pub(super) fn begin<'input, Tree, M>(
        tree: &Tree,
        container_node: Tree::Node,
        container: Size<S>,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Tree::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let scroll_progressions =
            with_block_scroll_projections::<Tree, M, _>(tree, container_node, |projection, _| {
                ScrollOriginAxes::ENDWARD
                    .scrollable_progressions(constants.flow_axes, projection.used_overflow())
            });
        let area_start_x = constants.effective_border.left + constants.scrollbar_gutter.left;
        let max_area_start_x = (container.width - constants.effective_border.right)
            .max(constants.effective_border.left);
        let area_start_y = constants.effective_border.top + constants.scrollbar_gutter.top;
        let max_area_start_y = (container.height - constants.effective_border.bottom)
            .max(constants.effective_border.top);
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

        Ok(Self {
            container,
            area_offset,
            area_size,
            available,
            scroll_progressions,
            next: 0,
        })
    }

    pub(super) fn next_child<'input, Tree, M>(
        &mut self,
        tree: &Tree,
        children: &[Tree::Node],
        static_positions: &[(Tree::Node, StaticPositionOf<S>)],
        constants: &Constants<S>,
    ) -> NextAbsoluteChild<Tree::Node, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let Self {
            container,
            area_offset,
            area_size,
            available,
            ..
        } = *self;
        while let Some(&child) = children.get(self.next) {
            let source_index = self.next;
            self.next += 1;
            let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
            let InlineParticipantKindOf::Box(style) = projection.kind() else {
                continue;
            };
            if style.common.position != Position::Absolute || style.display == crate::Display::None
            {
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
            let padding_border_size = (padding + border).sum_axes();
            let child_owned_preferred_axes =
                crate::flex::child_preferred_axes::<Tree, M>(tree, child);
            let used_size = PositionedUsedSize::resolve(PositionedSizeDeclarations {
                preferred: resolve_child_preferred_size(
                    style.common.size,
                    area_size.map(Some),
                    SizingAlgorithm::Positioned,
                    child_owned_preferred_axes,
                )
                .transpose_with_node(tree, child)?,
                child_owned_preferred_axes,
                min: minimum_size(
                    style.common.min_size,
                    area_size.map(Some),
                    SizingAlgorithm::Positioned,
                    true,
                )
                .transpose_with_node(tree, child)?,
                max: maximum_size(
                    style.common.max_size,
                    area_size.map(Some),
                    SizingAlgorithm::Positioned,
                    true,
                )
                .transpose_with_node(tree, child)?,
                aspect_ratio: *style.common.aspect_ratio,
                box_sizing: style.common.box_sizing,
                padding_border: padding_border_size,
            });
            let mut known_size = used_size.clamp_optional(used_size.preferred);
            let resolved_inset = style
                .common
                .inset
                .zip_size(area_size.map(Some), resolve_auto_optional)
                .transpose_with_node(tree, child)?;
            let inset =
                floor_absolute_definite_insets(resolved_inset, area_size, constants.flow_axes);
            let child_input = tree.node_input(child);
            let alignment = LogicalSizeOf::new(child_input.justify_self, child_input.align_self);
            let physical_alignment = constants.flow_axes.physical_size(alignment);
            let static_position = static_positions
                .iter()
                .find_map(|(node, position)| (*node == child).then_some(*position))
                .unwrap_or_else(|| {
                    absolute_static_position(
                        constants.logical_content_box_inset().block_start,
                        constants,
                        container,
                        StaticPositionGeometry::Point,
                    )
                });
            let flow = constants.flow_axes;
            let mut logical_inset = flow.logical_edges(inset);
            let sizing_inset = if static_position.geometry == StaticPositionGeometry::BlockStart
                && logical_inset.block_start.is_none()
                && logical_inset.block_end.is_none()
                && matches!(
                    alignment.block.map(AlignItems::unsafe_position),
                    None | Some(AlignItems::Start)
                ) {
                let static_start = flow
                    .logical_point(static_position.normal, Size::ZERO, container)
                    .block;
                let padding_start = flow.logical_point(area_offset, area_size, container).block;
                logical_inset.block_start = Some(static_start - padding_start);
                logical_inset.block_end = Some(S::ZERO);
                flow.physical_edges(logical_inset)
            } else {
                inset
            };
            let inset_extents = absolute_inset_extents(sizing_inset, area_size, flow);
            let (stretch, remaining_ratio) = absolute_stretch_fit_sizes(
                inset,
                unresolved_margin,
                inset_extents,
                physical_alignment,
                child_input,
                padding_border_size,
            );
            let stretch = used_size.automatic_stretch(stretch);
            if known_size.width.is_none()
                && let Some(stretch_width) = stretch.width
            {
                known_size.width = Some(stretch_width);
                known_size =
                    used_size.clamp_optional(used_size.transfer_ratio(known_size, remaining_ratio));
            }
            if known_size.height.is_none()
                && let Some(stretch_height) = stretch.height
            {
                known_size.height = Some(stretch_height);
                known_size =
                    used_size.clamp_optional(used_size.transfer_ratio(known_size, remaining_ratio));
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
            let input = ComputeInputOf::<S>::for_child(
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
            .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars);
            return Ok(Some(AbsoluteChild {
                node: child,
                input,
                source_index,
                padding,
                border,
                unresolved_margin,
                used_size,
                known_size,
                resolved_inset,
                inset,
                alignment,
                physical_alignment,
                static_position,
                inset_extents,
            }));
        }
        Ok(None)
    }

    pub(super) fn accept_child<'input, Tree, M>(
        &self,
        tree: &mut Tree,
        container_node: Tree::Node,
        constants: &Constants<S>,
        contributions: &mut ScrollContributionAccumulatorOf<S>,
        child: AbsoluteChild<Tree::Node, S>,
        output: crate::ComputeOutputOf<S>,
    ) -> LayoutResultOf<Tree::Node, (), S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let AbsoluteChild {
            node: child,
            source_index,
            padding,
            border,
            unresolved_margin,
            used_size,
            known_size,
            resolved_inset,
            inset,
            alignment,
            physical_alignment,
            static_position,
            inset_extents,
            ..
        } = child;
        let Self {
            area_offset,
            area_size,
            scroll_progressions,
            ..
        } = *self;
        if let Some(groups) = tree.group_computation() {
            groups.child(child);
        }
        let final_size = used_size.clamp(known_size.unwrap_or(output.size));
        let margin = resolve_absolute_margins(
            unresolved_margin,
            inset,
            final_size,
            area_size,
            constants.flow_axes,
        );
        let static_x_direction =
            static_axis_direction(constants.flow_axes, PhysicalAxis::Horizontal);
        let static_y_direction = static_axis_direction(constants.flow_axes, PhysicalAxis::Vertical);
        let horizontal_flow_anchor = (inset.left.is_none() && inset.right.is_none())
            || physical_alignment.width.is_some()
            || unresolved_margin.left.is_none()
            || unresolved_margin.right.is_none()
            || inset_extents.width == Some(S::ZERO);
        let vertical_flow_anchor = (inset.top.is_none() && inset.bottom.is_none())
            || physical_alignment.height.is_some()
            || unresolved_margin.top.is_none()
            || unresolved_margin.bottom.is_none()
            || inset_extents.height == Some(S::ZERO);
        let inset_alignment = absolute_inset_alignment_offsets(
            resolved_inset,
            unresolved_margin,
            final_size,
            area_size,
            constants.flow_axes,
            alignment,
            scroll_progressions,
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
                    start_alignment: inset_alignment.left,
                    end_alignment: inset_alignment.right,
                }
                .location(),
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
                    start_alignment: inset_alignment.top,
                    end_alignment: inset_alignment.bottom,
                }
                .location(),
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
        Ok(())
    }
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
    start_alignment: S,
    end_alignment: S,
}

impl<S: LayoutScalar> AbsoluteAxis<S> {
    fn location(self) -> S {
        if self.direction == Direction::Rtl
            && let (Some(_), Some(end)) = (self.start, self.end)
        {
            return if self.end_alignment == S::ZERO {
                self.area_start + self.area_size - end - self.size - self.margin_end
            } else {
                AbsoluteInsetStartOf::new(end, self.margin_end, self.end_alignment).project(
                    -self.area_start,
                    self.size,
                    self.area_size,
                    PhysicalProgression::Decreasing,
                )
            };
        }

        if let Some(start) = self.start {
            if self.start_alignment == S::ZERO {
                self.area_start + start + self.margin_start
            } else {
                AbsoluteInsetStartOf::new(start, self.margin_start, self.start_alignment).project(
                    self.area_start,
                    self.size,
                    self.area_size,
                    PhysicalProgression::Increasing,
                )
            }
        } else if let Some(end) = self.end {
            self.area_start + self.area_size - end - self.size - self.margin_end
        } else if self.direction == Direction::Rtl {
            self.static_position - self.size - self.margin_end
        } else {
            self.static_position + self.margin_start
        }
    }
}
