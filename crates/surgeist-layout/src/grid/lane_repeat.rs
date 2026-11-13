use super::*;
use crate::GridSpan;

#[derive(Clone, Copy)]
pub(super) struct LaneRepeatExpansion<'a> {
    pub(super) axis: GridAxisKind,
    pub(super) count: usize,
    pub(super) placement: LaneRepeatPlacement<'a>,
}

#[derive(Clone, Copy)]
pub(super) enum LaneRepeatPlacement<'a> {
    Seed,
    Hypothetical(&'a [Option<GridSpan>]),
    Actual,
}

pub(super) fn resolve_intrinsic_repeat<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    style: &GridContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    parent: &GridParentContext<Tree::Scalar, Tree::Node>,
    available: LogicalSizeOf<AxisSizingRequestOf<Tree::Scalar>>,
    contained_inline: Option<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Option<LaneRepeatExpansion<'static>>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if !style.display.establishes_grid_lanes_formatting_context() {
        return Ok(None);
    }
    let axis = grid_axis_for_grid_lanes(style);
    let components = match axis {
        GridAxisKind::Column if parent.columns.is_none() => style.grid_template_columns,
        GridAxisKind::Row if parent.rows.is_none() => style.grid_template_rows,
        _ => return Ok(None),
    };
    let Some(group_tracks) = intrinsic_auto_repeat_track_count(components)
        .map_err(|error| grid_track_sizing_error(node, error))?
    else {
        return Ok(None);
    };
    let used = |count| {
        Some(LaneRepeatExpansion {
            axis,
            count,
            placement: LaneRepeatPlacement::Actual,
        })
    };
    let flow = constants.flow_axes;
    let constraint = constants
        .auto_repeat_constraint(axis)
        .map_err(|status| crate::error::value_resolution_error(node, status))?;
    if matches!(constraint, AutoRepeatConstraint::One) {
        return Ok(used(1));
    }
    let initial_size = constants.initial_track_basis(constants.node_inner_size);
    let initial_available = constants.initial_track_basis(constants.available_inner_size);
    let basis = track_expansion_basis(
        components,
        axis.component(flow.logical_size(initial_size)),
        axis.component(flow.logical_size(initial_available)),
    );
    let seed = initialize_grid_tracks_with_repeat(
        tree,
        node,
        style,
        constants,
        parent,
        available,
        Some(LaneRepeatExpansion {
            axis,
            count: 1,
            placement: LaneRepeatPlacement::Seed,
        }),
    )?;
    let explicit_count = match axis {
        GridAxisKind::Column => seed.context.lines.column_explicit_count,
        GridAxisKind::Row => seed.context.lines.row_explicit_count,
    };
    let mut spans = Vec::new();
    spans
        .try_reserve_exact(seed.placements.items.len())
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    let mut largest = None;
    // One seed group resolves named and negative line endpoints. Its starts
    // only determine span length; every in-flow item is automatic in the probe.
    for item in seed.placements.items {
        let span = if item.in_flow {
            let placement = match axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            let range =
                signed_definite_axis_range(placement, explicit_count, axis).map_err(|_| {
                    grid_track_sizing_error(node, SpanningContributionError::Allocation)
                })?;
            let length = range.map_or_else(
                || placement.span().map_or(1, GridSpan::get),
                |(start, end)| start.abs_diff(end),
            );
            let span = GridSpan::new(length).ok_or_else(|| {
                grid_track_sizing_error(node, SpanningContributionError::InvalidIncidence)
            })?;
            largest = Some(largest.map_or(length, |current: usize| current.max(length)));
            Some(span)
        } else {
            None
        };
        spans.push(span);
    }
    let Some(largest) = largest else {
        return Ok(used(1));
    };
    let count = hypothetical_repeat_groups(largest, group_tracks)
        .map_err(|error| grid_track_sizing_error(node, error))?;
    let probe = initialize_grid_tracks_with_repeat(
        tree,
        node,
        style,
        constants,
        parent,
        available,
        Some(LaneRepeatExpansion {
            axis,
            count,
            placement: LaneRepeatPlacement::Hypothetical(&spans),
        }),
    )?;
    let gap = axis.component(probe.context.gap);
    let measurement_constants = if axis == GridAxisKind::Row {
        constants.for_inline_track_basis(contained_inline)
    } else {
        *constants
    };
    let input = GridTrackResolutionInput {
        style,
        constants: &measurement_constants,
        column_tracks: &probe.column_tracks,
        row_tracks: &probe.row_tracks,
        minimum_functions: minimum_parent_functions(
            &probe.column_tracks,
            &probe.row_tracks,
            probe.context.percent_basis,
            parent,
        ),
        context: std::borrow::Cow::Owned(probe.context),
        subgrid_report: &probe.subgrid_report,
        sizing_flow_axes: flow,
        available,
        placements: &probe.placements,
        stage: GridSizingStage {
            first_axis: axis,
            through: GridSizingPhase::InitialFirst,
        },
    };
    let phases = intrinsic_stage::intrinsic_repeat_growth_limits(tree, node, &input, axis)?;
    let (origins, tracks) = match axis {
        GridAxisKind::Column => (&input.context.topology.column_origins, input.column_tracks),
        GridAxisKind::Row => (&input.context.topology.row_origins, input.row_tracks),
    };
    let mut sizes = Vec::new();
    sizes
        .try_reserve_exact(group_tracks)
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    sizes.resize(group_tracks, Tree::Scalar::ZERO);
    for (((origin, track), base), limit) in origins
        .iter()
        .zip(tracks)
        .zip(phases.base_sizes)
        .zip(phases.growth_limits)
    {
        if let Some(repeated) = origin.auto_repeat {
            let operand =
                definite_repeat_track_size(track, basis).unwrap_or_else(|| limit.unwrap_or(base));
            sizes[repeated.track_index] = sizes[repeated.track_index].max(operand);
        }
    }
    let reserved = standalone_reserved_track_space(components, group_tracks, basis, gap)
        .map_err(|error| grid_track_sizing_error(node, error))?;
    let count = auto_repeat_count_from_hypothetical(&sizes, constraint, gap, &reserved)
        .map_err(|error| grid_track_sizing_error(node, error))?;
    Ok(used(count))
}
