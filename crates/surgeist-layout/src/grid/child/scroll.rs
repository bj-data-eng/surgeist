use super::*;
use crate::scroll::{
    CanonicalScrollGeometryErrorOf, MeasuredLeafScrollGeometrySourceOf,
    OptionalPhysicalContributionIntervalsOf, ScrollContributionAccumulatorOf,
    canonical_measured_leaf_scroll_geometry, rebuild_canonical_scroll_geometry_for_border_box,
};

#[derive(Clone, Copy, Debug)]
pub(in crate::grid) struct GridChildContribution<S: LayoutScalar = Scalar> {
    pub(in crate::grid) source_index: crate::SourceIndex,
    pub(in crate::grid) location: Point<S>,
    pub(in crate::grid) margin: Edges<S>,
    pub(in crate::grid) geometry: crate::ScrollGeometryOf<S>,
    pub(in crate::grid) descendants: OptionalPhysicalContributionIntervalsOf<S>,
    pub(in crate::grid) in_flow: bool,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct GridTrackScrollBounds<S: LayoutScalar> {
    pub(in crate::grid) rect: crate::ScrollRectOf<S>,
    pub(in crate::grid) present: LogicalSizeOf<bool>,
}

pub(in crate::grid) struct GridTrackAlignmentGeometry<'a, S: LayoutScalar> {
    pub(in crate::grid) sizes: LogicalSizeOf<&'a [S]>,
    pub(in crate::grid) offsets: LogicalSizeOf<&'a [S]>,
    pub(in crate::grid) fallback_origin: LogicalPointOf<S>,
    pub(in crate::grid) flow_axes: FlowAxes,
    pub(in crate::grid) containing_size: Size<S>,
    pub(in crate::grid) stacking_range: Option<(LogicalAxis, S, S)>,
}

pub(in crate::grid) fn grid_track_alignment_subject<S: LayoutScalar>(
    geometry: GridTrackAlignmentGeometry<'_, S>,
) -> Result<crate::ScrollRectOf<S>, crate::ScrollRectErrorOf<S>> {
    let GridTrackAlignmentGeometry {
        sizes,
        offsets,
        fallback_origin,
        flow_axes,
        containing_size,
        stacking_range,
    } = geometry;
    let axis_extent = |offsets: &[S], sizes: &[S], fallback: S| {
        let start = offsets.iter().copied().reduce(S::min).unwrap_or(fallback);
        let end = offsets
            .iter()
            .copied()
            .zip(sizes.iter().copied())
            .map(|(offset, size)| offset + size)
            .reduce(S::max)
            .unwrap_or(start);
        (start, end - start)
    };
    let (inline_start, inline_extent) =
        axis_extent(offsets.inline, sizes.inline, fallback_origin.inline);
    let (block_start, block_extent) =
        axis_extent(offsets.block, sizes.block, fallback_origin.block);
    let mut origin = LogicalPointOf::new(inline_start, block_start);
    let mut size = LogicalSizeOf::new(inline_extent, block_extent);
    if let Some((axis, start, end)) = stacking_range {
        let stacking_origin = start.min(end);
        let stacking_extent = start.max(end) - stacking_origin;
        match axis {
            LogicalAxis::Inline => {
                origin.inline = stacking_origin;
                size.inline = stacking_extent;
            }
            LogicalAxis::Block => {
                origin.block = stacking_origin;
                size.block = stacking_extent;
            }
        }
    }
    crate::ScrollRectOf::try_new(
        flow_axes.physical_point(origin, size, containing_size),
        flow_axes.physical_size(size),
    )
}

pub(in crate::grid) struct GridLayoutReporting<Node, S: LayoutScalar> {
    pub(in crate::grid) active_alignment_axes: LogicalSizeOf<bool>,
    pub(in crate::grid) flow_axes: FlowAxes,
    pub(in crate::grid) baselines: BaselinesOf<S>,
    pub(in crate::grid) baseline_groups: GridBaselineGroups<S>,
    pub(in crate::grid) execution_input: ComputeInputOf<S>,
    pub(in crate::grid) executed_inventory: ExecutedBaselineInventory<Node, S>,
}

pub(in crate::grid) fn report_grid_children<Node, S, M>(
    node: Node,
    mut contributions: ScrollContributionAccumulatorOf<S>,
    track_subject: crate::ScrollRectOf<S>,
    reporting: GridLayoutReporting<Node, S>,
) -> GridChildrenLayoutResult<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let GridLayoutReporting {
        active_alignment_axes,
        flow_axes,
        baselines,
        baseline_groups,
        execution_input,
        executed_inventory,
    } = reporting;
    if active_alignment_axes.inline {
        contributions.set_active_alignment_subject(flow_axes.inline_axis(), track_subject);
    }
    if active_alignment_axes.block {
        contributions.set_active_alignment_subject(flow_axes.block_axis(), track_subject);
    }
    let visible_content_size = contributions
        .content_size_from_anchor(Point::ZERO)
        .map_err(|error| layout_child_geometry_error(node, node, error))?;
    Ok(GridChildrenLayout {
        visible_content_size,
        contributions,
        baselines,
        baseline_groups,
        executed_baselines: execution_input
            .run_mode()
            .is_perform_layout()
            .then_some(executed_inventory),
    })
}

pub(in crate::grid) fn empty_grid_contributions<S: LayoutScalar>()
-> ScrollContributionAccumulatorOf<S> {
    ScrollContributionAccumulatorOf::new(
        crate::ScrollRectOf::try_new(Point::ZERO, Size::ZERO)
            .expect("zero grid contribution seed is valid"),
    )
}

pub(in crate::grid) fn retained_grid_child_scroll_geometry<'input, S: LayoutScalar>(
    item: &GridItemProjection<'input, S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<crate::ScrollGeometryOf<S>>,
) -> Result<crate::ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    if let Some(geometry) = child_compute_geometry {
        if geometry.border_box().origin() == Point::ZERO && geometry.border_box().size() == size {
            return Ok(geometry);
        }
        return rebuild_canonical_scroll_geometry_for_border_box(geometry, size, border, padding);
    }

    item.with_scroll_projections(|box_projection, target_projection| {
        canonical_measured_leaf_scroll_geometry(MeasuredLeafScrollGeometrySourceOf {
            box_projection,
            target_projection,
            border_box_size: size,
            border,
            padding,
            settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState::INITIAL,
            measured_content_size: content_size,
        })
    })
}
pub(in crate::grid) fn grid_scroll_contributions<S: LayoutScalar>(
    mut children: Vec<GridChildContribution<S>>,
    tracks: Option<GridTrackScrollBounds<S>>,
    flow_axes: FlowAxes,
    padding: Edges<S>,
) -> Result<ScrollContributionAccumulatorOf<S>, crate::scroll::ScrollContributionErrorOf<S>> {
    children.sort_by_key(|child| child.source_index);
    let mut contributions = empty_grid_contributions();
    let mut inline_end = None;
    let mut block_end = None;

    if let Some(tracks) = tracks {
        for (present, side, end) in [
            (
                tracks.present.inline,
                flow_axes.inline_end(),
                &mut inline_end,
            ),
            (tracks.present.block, flow_axes.block_end(), &mut block_end),
        ] {
            if present {
                let interval =
                    contributions.include_established_grid_tracks(tracks.rect, side.axis());
                *end = Some(match side {
                    crate::PhysicalSide::Top | crate::PhysicalSide::Left => interval.minimum(),
                    crate::PhysicalSide::Right | crate::PhysicalSide::Bottom => interval.maximum(),
                });
            }
        }
    }

    for child in children {
        contributions.include_child_geometry(
            child.location,
            child.margin,
            child.geometry,
            child.descendants,
        )?;
        if child.in_flow {
            let border_size = child.geometry.border_box().size();
            if border_size.width > S::ZERO && border_size.height > S::ZERO {
                include_farthest_grid_flow_end(
                    &mut inline_end,
                    flow_axes.inline_end(),
                    grid_child_flow_end(child, flow_axes.inline_end()),
                );
                include_farthest_grid_flow_end(
                    &mut block_end,
                    flow_axes.block_end(),
                    grid_child_flow_end(child, flow_axes.block_end()),
                );
            }
        }
    }

    for (axis, coordinate) in [
        (LogicalAxis::Inline, inline_end),
        (LogicalAxis::Block, block_end),
    ] {
        if let Some(coordinate) = coordinate {
            contributions.record_final_in_flow_end(flow_axes, axis, coordinate)?;
        }
    }
    contributions.include_terminal_padding(padding)?;
    Ok(contributions)
}

/// Uses the complete signed in-flow overflow with the reserved-gutter-excluded
/// padding box as its seed. Direct absolute children are added only afterwards.
pub(in crate::grid) fn grid_absolute_auto_area<S: LayoutScalar>(
    mut in_flow: ScrollContributionAccumulatorOf<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
) -> Result<crate::ScrollRectOf<S>, crate::ScrollRectErrorOf<S>> {
    in_flow.replace_container_seed(scroll_box.scrollport());
    in_flow.complete_overflow_rect()
}

fn grid_child_flow_end<S: LayoutScalar>(
    child: GridChildContribution<S>,
    side: crate::PhysicalSide,
) -> S {
    let border_box = child.geometry.border_box();
    let origin = border_box.origin();
    let size = border_box.size();
    match side {
        crate::PhysicalSide::Top => child.location.y + origin.y - child.margin.top.max(S::ZERO),
        crate::PhysicalSide::Right => {
            child.location.x + origin.x + size.width + child.margin.right.max(S::ZERO)
        }
        crate::PhysicalSide::Bottom => {
            child.location.y + origin.y + size.height + child.margin.bottom.max(S::ZERO)
        }
        crate::PhysicalSide::Left => child.location.x + origin.x - child.margin.left.max(S::ZERO),
    }
}

fn include_farthest_grid_flow_end<S: LayoutScalar>(
    end: &mut Option<S>,
    side: crate::PhysicalSide,
    candidate: S,
) {
    *end = Some(end.map_or(candidate, |current| match side {
        crate::PhysicalSide::Top | crate::PhysicalSide::Left => current.min(candidate),
        crate::PhysicalSide::Right | crate::PhysicalSide::Bottom => current.max(candidate),
    }));
}

pub(in crate::grid) fn max_size<S: LayoutScalar>(a: Size<S>, b: Size<S>) -> Size<S> {
    Size::new(a.width.max(b.width), a.height.max(b.height))
}
