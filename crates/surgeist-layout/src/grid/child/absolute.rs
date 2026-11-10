use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{
    LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide, project_from_start,
};
use crate::layout_math::{
    AbsoluteInsetStartOf, MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinSizeClampExt,
    absolute_inset_alignment_offsets,
};
use crate::sizing::resolve::{
    resolve_maximum_optional, resolve_minimum_optional, resolve_preferred_optional,
};

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridContext<'a, S: LayoutScalar> {
    source: AbsoluteGridContextInput<'a, S>,
    stacking_range: Option<(S, S)>,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridContextInput<'a, S: LayoutScalar> {
    pub(in crate::grid) container_style: &'a GridContainerProjection<'a, S>,
    pub(in crate::grid) constants: &'a Constants<S>,
    pub(in crate::grid) containing_size: Size<S>,
    /// Frozen from qualified in-flow geometry before any direct absolute child.
    pub(in crate::grid) auto_area: crate::ScrollRectOf<S>,
    pub(in crate::grid) column: super::GridPlacement,
    pub(in crate::grid) row: super::GridPlacement,
    pub(in crate::grid) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) lines: GridLines,
}

impl<'a, S: LayoutScalar> AbsoluteGridContext<'a, S> {
    pub(in crate::grid) fn new(source: AbsoluteGridContextInput<'a, S>) -> Self {
        Self {
            source,
            stacking_range: None,
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        }
    }

    pub(in crate::grid) fn for_lanes(
        source: AbsoluteGridContextInput<'a, S>,
        stacking_start: S,
        stacking_end: S,
    ) -> Self {
        Self {
            source,
            stacking_range: Some((stacking_start, stacking_end)),
            containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState::INITIAL,
        }
    }

    pub(in crate::grid) fn with_containing_auto_scrollbar_pass(
        mut self,
        containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    ) -> Self {
        self.containing_auto_scrollbar_pass = containing_auto_scrollbar_pass;
        self
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAreaInput<'a, S: LayoutScalar> {
    pub(in crate::grid) column: super::GridPlacement,
    pub(in crate::grid) row: super::GridPlacement,
    pub(in crate::grid) auto_area: crate::ScrollRectOf<S>,
    pub(in crate::grid) columns: AbsoluteGridAxisLines<'a, S>,
    pub(in crate::grid) rows: AbsoluteGridAxisLines<'a, S>,
    pub(in crate::grid) containing_size: Size<S>,
    pub(in crate::grid) constants: &'a Constants<S>,
}

#[derive(Clone, Copy)]
pub(in crate::grid) enum AbsoluteGridAxisLines<'a, S: LayoutScalar> {
    Grid {
        geometry: &'a UsedGridAxisGeometryOf<S>,
        explicit_start: usize,
        explicit_count: usize,
    },
    Stacking {
        start: S,
        end: S,
    },
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAxisInput<'a, S: LayoutScalar = Scalar> {
    pub(in crate::grid) placement: super::GridPlacement,
    pub(in crate::grid) lines: AbsoluteGridAxisLines<'a, S>,
    pub(in crate::grid) padding_box_location: S,
    pub(in crate::grid) padding_box_size: S,
    pub(in crate::grid) is_reverse: bool,
}
pub(in crate::grid) fn layout_absolute_grid_child<'input, Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    source_index: usize,
    child_style: &GridItemProjection<'input, Tree::Scalar>,
    context: AbsoluteGridContext<'_, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridChildContribution<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let containing_auto_scrollbar_pass = context.containing_auto_scrollbar_pass;
    let stacking_range = context.stacking_range;
    let context = context.source;
    let container_style = context.container_style;
    let constants = context.constants;
    let containing_size = context.containing_size;
    let axis_lines = |axis, geometry, explicit_start, explicit_count| {
        if let Some((start, end)) = stacking_range
            && axis == lane_axis_for_grid_lanes(container_style)
        {
            AbsoluteGridAxisLines::Stacking { start, end }
        } else {
            AbsoluteGridAxisLines::Grid {
                geometry,
                explicit_start,
                explicit_count,
            }
        }
    };
    let area = absolute_grid_area(AbsoluteGridAreaInput {
        column: context.column,
        row: context.row,
        auto_area: context.auto_area,
        columns: axis_lines(
            GridAxisKind::Column,
            context.column_geometry,
            context.lines.column_explicit_start,
            context.lines.column_explicit_count,
        ),
        rows: axis_lines(
            GridAxisKind::Row,
            context.row_geometry,
            context.lines.row_explicit_start,
            context.lines.row_explicit_count,
        ),
        containing_size,
        constants,
    });
    let containing_flow_axes = constants.flow_axes;
    let physical_area_size = containing_flow_axes.physical_size(area.size);
    let area_parent = physical_area_size.map(Some);
    let unresolved_margin = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.margin, area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let non_auto_margin = unresolved_margin.map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let available_size = Size::new(
        crate::layout_math::remaining_available_extent(
            physical_area_size.width,
            non_auto_margin.horizontal_sum(),
        )
        .map_err(|status| crate::error::value_resolution_error(child, status))?,
        crate::layout_math::remaining_available_extent(
            physical_area_size.height,
            non_auto_margin.vertical_sum(),
        )
        .map_err(|status| crate::error::value_resolution_error(child, status))?,
    );
    let padding = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.padding, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let border = containing_flow_axes
        .zip_physical_edges_with_inline_extent(child_style.border, area_parent, |length, basis| {
            resolve_length_or_zero(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    let style_size = Size::new(
        resolve_preferred_optional(
            &child_style.size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_preferred_optional(
            &child_style.size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let padding_border_size = (padding + border).sum_axes();
    let min_size = Size::new(
        resolve_minimum_optional(
            &child_style.min_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_minimum_optional(
            &child_style.min_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .add_optional(box_sizing_adjustment)
    .or(padding_border_size.map(Some))
    .max_optional(padding_border_size.map(Some))
    .apply_aspect_ratio(child_style.aspect_ratio);
    let max_size = Size::new(
        resolve_maximum_optional(
            &child_style.max_size.width,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
        resolve_maximum_optional(
            &child_style.max_size.height,
            SizingAlgorithm::Positioned,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )
        .map_err(|error| sizing_resolution_error(child, error))?,
    )
    .apply_aspect_ratio(child_style.aspect_ratio)
    .add_optional(box_sizing_adjustment);
    let resolved_inset = child_style
        .inset
        .zip_size(area_parent, |length, basis| {
            resolve_auto_optional(length, basis)
        })
        .transpose_with_node(tree, child)?;
    let inset = crate::layout_math::floor_absolute_definite_insets(
        resolved_inset,
        physical_area_size,
        containing_flow_axes,
    );
    let inset_extents =
        crate::layout_math::absolute_inset_extents(inset, physical_area_size, containing_flow_axes);
    let alignment = LogicalSizeOf::new(child_style.justify_self, child_style.align_self);
    let (stretch, remaining_ratio) = crate::layout_math::absolute_stretch_fit_sizes(
        inset,
        unresolved_margin,
        inset_extents,
        containing_flow_axes.physical_size(alignment),
        tree.node_input(child),
        padding_border_size,
    );
    let mut known = style_size.or(stretch);
    if let (Some(ratio), Some(width)) = (remaining_ratio, known.width)
        && child_style.size.height.is_auto()
    {
        known.height = Some(width / ratio.get());
    } else if let (Some(ratio), Some(height)) = (remaining_ratio, known.height)
        && child_style.size.width.is_auto()
    {
        known.width = Some(height * ratio.get());
    }
    let known = known
        .apply_aspect_ratio(remaining_ratio)
        .clamp_max_before_min_optional(min_size, max_size);
    let fallback_limits = crate::layout_math::containing_orthogonal_fallback_limits(
        constants.fallback_minimum_outer,
        constants.fallback_maximum_outer,
        (constants.border + constants.scrollbar_gutter).sum_axes(),
    )
    .map_err(|status| crate::error::value_resolution_error(child, status))?;
    let orthogonal_fallback = constants
        .child_orthogonal_fallback
        .for_child(fallback_limits, crate::ScrollportFallbackChangeOf::Preserve);
    let available = crate::layout_math::transpose_axis_requests_at_node(
        child,
        available_size.map(|value| {
            crate::layout_math::checked_available_space(value)
                .map(|space| AxisSizingRequestOf::new(space, SizingOperation::Normal))
        }),
    )?;
    let available = crate::layout_math::transpose_axis_requests_at_node(
        child,
        crate::layout_math::absolute_inset_child_axes(
            available,
            inset_extents,
            unresolved_margin,
            tree.has_leaf_measurement(child),
        ),
    )?;
    let output = tree.compute_child(
        child,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            known,
            area_parent,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    container_style.writing_mode,
                    container_style.direction,
                ),
                crate::ParentFormattingContext::Grid,
            ),
            SizingRequestOf::new(available, orthogonal_fallback),
        )
        .with_containing_auto_scrollbar_pass(containing_auto_scrollbar_pass),
    )?;
    let final_size = known
        .unwrap_or(output.size)
        .clamp_max_before_min_optional(min_size, max_size);
    let justify = child_style
        .justify_self
        .unwrap_or(container_style.justify_items.unwrap_or(AlignItems::Start));
    let align = child_style
        .align_self
        .unwrap_or(container_style.align_items.unwrap_or(AlignItems::Start));
    let scroll_progressions = crate::scroll::ScrollOriginAxes::ENDWARD.scrollable_progressions(
        containing_flow_axes,
        crate::scroll::UsedOverflow::from_computed(
            container_style.overflow,
            container_style.item_is_replaced,
        ),
    );
    let (location, margin) = {
        let logical_size = containing_flow_axes.logical_size(final_size);
        let containing = containing_flow_axes.logical_size(containing_size);
        let logical_margin = containing_flow_axes.logical_edges(unresolved_margin);
        let logical_inset = containing_flow_axes.logical_edges(inset);
        let inset_alignment = containing_flow_axes.logical_edges(absolute_inset_alignment_offsets(
            resolved_inset,
            unresolved_margin,
            final_size,
            physical_area_size,
            containing_flow_axes,
            alignment,
            scroll_progressions,
        ));
        let inline_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.inline,
            static_area_location: area.static_location.inline,
            area_size: area.size.inline,
            static_area_size: area.static_size.inline,
            size: logical_size.inline,
            margin_start: logical_margin.inline_start,
            margin_end: logical_margin.inline_end,
            inset_start: logical_inset.inline_start,
            inset_end: logical_inset.inline_end,
            is_inline: true,
            inset_alignment: inset_alignment.inline_start,
            alignment: justify,
            containing_extent: containing.inline,
            flow_start: containing_flow_axes.inline_start(),
        });
        let block_axis = absolute_grid_axis(AbsoluteGridAxis {
            area_location: area.location.block,
            static_area_location: area.static_location.block,
            area_size: area.size.block,
            static_area_size: area.static_size.block,
            size: logical_size.block,
            margin_start: logical_margin.block_start,
            margin_end: logical_margin.block_end,
            inset_start: logical_inset.block_start,
            inset_end: logical_inset.block_end,
            is_inline: false,
            inset_alignment: inset_alignment.block_start,
            alignment: align,
            containing_extent: containing.block,
            flow_start: containing_flow_axes.block_start(),
        });
        let position = containing_flow_axes.physical_size(LogicalSizeOf::new(
            inline_axis.location,
            block_axis.location,
        ));
        (
            Point::new(position.width, position.height),
            containing_flow_axes.physical_edges(LogicalEdgesOf::new(
                inline_axis.margin_start,
                inline_axis.margin_end,
                block_axis.margin_start,
                block_axis.margin_end,
            )),
        )
    };

    let scroll_geometry = retained_grid_child_scroll_geometry(
        child_style,
        final_size,
        output.content_size,
        padding,
        border,
        output.scroll_geometry,
    )
    .map_err(|error| layout_child_geometry_error(child, child, error))?;
    tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
        child,
        crate::output::NormalGeometryOf::box_geometry(
            location,
            final_size,
            scroll_geometry.content_box(),
        ),
        NodeOutputOf {
            source_index: crate::SourceIndex::new(source_index),
            location,
            size: final_size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            scroll_geometry: Some(scroll_geometry),
            border,
            padding,
            margin,
        },
    ));

    Ok(GridChildContribution {
        source_index: crate::SourceIndex::new(source_index),
        location,
        margin,
        geometry: scroll_geometry,
        descendants: scroll_geometry.propagatable_descendant_intervals(),
        in_flow: false,
    })
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct LogicalAbsoluteGridArea<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: LogicalPointOf<S>,
    pub(in crate::grid) size: LogicalSizeOf<S>,
    pub(in crate::grid) static_location: LogicalPointOf<S>,
    pub(in crate::grid) static_size: LogicalSizeOf<S>,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct AbsoluteGridAxisArea<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: S,
    pub(in crate::grid) size: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct AbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(in crate::grid) area_location: S,
    pub(in crate::grid) static_area_location: S,
    pub(in crate::grid) area_size: S,
    pub(in crate::grid) static_area_size: S,
    pub(in crate::grid) size: S,
    pub(in crate::grid) margin_start: Option<S>,
    pub(in crate::grid) margin_end: Option<S>,
    pub(in crate::grid) inset_start: Option<S>,
    pub(in crate::grid) inset_end: Option<S>,
    pub(in crate::grid) is_inline: bool,
    pub(in crate::grid) inset_alignment: S,
    pub(in crate::grid) alignment: AlignItems,
    pub(in crate::grid) containing_extent: S,
    pub(in crate::grid) flow_start: PhysicalSide,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct ResolvedAbsoluteGridAxis<S: LayoutScalar = Scalar> {
    pub(in crate::grid) location: S,
    pub(in crate::grid) margin_start: S,
    pub(in crate::grid) margin_end: S,
}

pub(in crate::grid) fn absolute_grid_axis<S: LayoutScalar>(
    axis: AbsoluteGridAxis<S>,
) -> ResolvedAbsoluteGridAxis<S> {
    let AbsoluteGridAxis {
        area_location,
        static_area_location,
        area_size,
        static_area_size,
        size,
        margin_start,
        margin_end,
        inset_start,
        inset_end,
        is_inline,
        inset_alignment,
        alignment,
        containing_extent,
        flow_start,
    } = axis;
    let uses_static_area = inset_start.is_none() && inset_end.is_none();
    let non_auto_start = margin_start.unwrap_or(S::ZERO);
    let non_auto_end = margin_end.unwrap_or(S::ZERO);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let containing_free_space = area_size - size - non_auto_start - non_auto_end;
    let auto_margin = if !uses_static_area && auto_margin_count > 0 {
        containing_free_space.max(S::ZERO) / S::from_usize(auto_margin_count)
    } else {
        S::ZERO
    };
    let (resolved_start, resolved_end) = if uses_static_area {
        (
            margin_start.unwrap_or(auto_margin),
            margin_end.unwrap_or(auto_margin),
        )
    } else {
        crate::layout_math::resolve_absolute_axis_margins(
            margin_start,
            margin_end,
            inset_start,
            inset_end,
            size,
            area_size,
            is_inline,
        )
    };
    let alignment_free_space = if uses_static_area {
        static_area_size - size - resolved_start - resolved_end
    } else {
        containing_free_space
    };
    if inset_alignment != S::ZERO
        && let Some(start) = inset_start
    {
        return ResolvedAbsoluteGridAxis {
            location: AbsoluteInsetStartOf::new(start, resolved_start, inset_alignment).project(
                area_location,
                size,
                containing_extent,
                flow_start.progression(),
            ),
            margin_start: resolved_start,
            margin_end: resolved_end,
        };
    }
    let offset = match (inset_start, inset_end) {
        (Some(start), _) => start + resolved_start,
        (None, Some(end)) => area_size - end - size - resolved_end,
        (None, None) => match alignment.safe_fallback(alignment_free_space) {
            AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
                static_area_size - size - resolved_end
            }
            AlignItems::Center => {
                (static_area_size - size + resolved_start - resolved_end) / S::from_f64(2.0)
            }
            AlignItems::Start
            | AlignItems::FlexStart
            | AlignItems::Baseline
            | AlignItems::Stretch => resolved_start,
            AlignItems::Safe(_) | AlignItems::Unsafe(_) => {
                unreachable!("safe_fallback returns unqualified item alignment")
            }
        },
    };
    let base_location = if uses_static_area {
        static_area_location
    } else {
        area_location
    };
    ResolvedAbsoluteGridAxis {
        location: project_from_start(base_location + offset, size, containing_extent, flow_start),
        margin_start: resolved_start,
        margin_end: resolved_end,
    }
}
