use super::*;

#[cfg(test)]
#[path = "intrinsic_stage_tests.rs"]
mod tests;

pub(super) fn has_layout_basis<S: LayoutScalar>(
    stage: GridSizingStage,
    constants: &Constants<S>,
    columns: &[TrackSizingOf<S>],
    rows: &[TrackSizingOf<S>],
) -> bool {
    if stage.through != GridSizingPhase::InitialFirst {
        return true;
    }
    let (opposite_outer, opposite_tracks) = match stage.first_axis {
        GridAxisKind::Column => (
            constants
                .flow_axes
                .logical_size(constants.node_outer_size)
                .block,
            rows,
        ),
        GridAxisKind::Row => (
            constants
                .flow_axes
                .logical_size(constants.node_outer_size)
                .inline,
            columns,
        ),
    };
    opposite_outer.is_some()
        || opposite_tracks
            .iter()
            .all(|track| track.max.definite(None).is_some())
}

// Intrinsic preparation and accepted final allocation have different authorities.
#[derive(Clone, Copy)]
pub(super) enum GridAxisTrackSpace<S: LayoutScalar> {
    Intrinsic,
    FinalUsed(S),
}

#[derive(Clone, Copy)]
pub(super) struct IntrinsicStageFrame<'a, S: LayoutScalar> {
    pub(super) axis: GridAxisKind,
    pub(super) stage: GridSizingStage,
    pub(super) columns: Option<&'a [S]>,
    pub(super) rows: Option<&'a [S]>,
    pub(super) observed_axis: Option<&'a [S]>,
}

pub(super) struct GridAxisTrackInputs<S: LayoutScalar> {
    pub(super) rows: Vec<GridIntrinsicContributionRow<S>>,
    pub(super) base_floors: Vec<S>,
    // The original MinContent floor backing remains separate from the
    // all-purpose aggregate used by reduction. Only this basis can admit a
    // source-defined min-content revisit.
    pub(super) min_content_base_floors: Vec<S>,
}

pub(super) struct GridAxisTrackPreparation<S: LayoutScalar> {
    pub(super) contributions: GridAxisTrackInputs<S>,
    pub(super) phases: GridAxisTrackPhases<S>,
    // The request operation owns the phases and normal-recomputation decision.
    pub(super) available: AvailableOf<S>,
    // A maximum can constrain finishing without replacing that operation.
    pub(super) finishing_available: AvailableOf<S>,
}

pub(super) struct GridAxisTrackReduction<S: LayoutScalar> {
    pub(super) sizes: Vec<S>,
    pub(super) phases: GridAxisTrackPhases<S>,
    pub(super) available: AvailableOf<S>,
    pub(super) finishing_available: AvailableOf<S>,
}

struct EvaluatedAxis<Node, S: LayoutScalar> {
    contributions: GridAxisTrackInputs<S>,
    receipts: Vec<GridContributionReceipt<Node, S>>,
}

type RoleResult<Node, S, M> = LayoutResultOf<Node, IntrinsicAxisContributions<Node, S>, S, M>;
type StageResult<Node, S, M> = LayoutResultOf<Node, EvaluatedAxis<Node, S>, S, M>;
type PreparedAxisResult<Node, S, M> =
    LayoutResultOf<Node, (Vec<S>, GridAxisTrackPreparation<S>), S, M>;

// A single source-ordered preparation serves full layout and bounded probes.
pub(super) fn resolve_tracks<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: GridTrackResolutionInput<'input, '_, Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, GridTrackResolution<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let grid = IntrinsicGrid {
        style: input.style,
        constants: input.constants,
        sizing_flow_axes: input.sizing_flow_axes,
        minimum_functions: input.minimum_functions,
        track_basis: input.context.percent_basis,
        column_tracks: input.column_tracks,
        row_tracks: input.row_tracks,
        gap: input.context.gap,
        column_gutters: Some(&input.context.column_gutters),
        row_gutters: Some(&input.context.row_gutters),
        lines: input.context.lines,
        named_columns: &input.context.topology.named_columns,
        named_rows: &input.context.topology.named_rows,
        area_facts: input.context.topology.area_facts.as_ref(),
        subgrid_report: input.subgrid_report,
        placements: input.placements,
        query: IntrinsicGridQuery::complete(GridContributionPurpose::Minimum),
    };
    let first = input.stage.first_axis;
    let second = match first {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    };
    let mut columns = None;
    let mut rows = None;
    let mut column_preparation: Option<GridAxisTrackPreparation<Tree::Scalar>> = None;
    let mut row_preparation: Option<GridAxisTrackPreparation<Tree::Scalar>> = None;
    let mut column_receipts = Vec::new();
    let mut row_receipts = Vec::new();
    for (phase, axis) in [
        (GridSizingPhase::InitialFirst, first),
        (GridSizingPhase::InitialSecond, second),
        (GridSizingPhase::RevisitFirst, first),
        (GridSizingPhase::RevisitSecond, second),
    ] {
        if phase > input.stage.through {
            break;
        }
        let evaluated = evaluate_stage(
            tree,
            node,
            &input,
            grid,
            IntrinsicStageFrame {
                axis,
                stage: GridSizingStage {
                    through: phase,
                    ..input.stage
                },
                columns: columns.as_deref(),
                rows: rows.as_deref(),
                observed_axis: None,
            },
            GridAxisTrackSpace::Intrinsic,
        )?;
        let (previous_receipts, previous) = match axis {
            GridAxisKind::Column => (&column_receipts, column_preparation.as_ref()),
            GridAxisKind::Row => (&row_receipts, row_preparation.as_ref()),
        };
        if phase >= GridSizingPhase::RevisitFirst
            && same_min_content_receipts(&evaluated.receipts, previous_receipts)
            && previous.is_some_and(|previous| {
                same_min_content_inputs(&evaluated.contributions, &previous.contributions)
            })
        {
            continue;
        }
        let (sizes, preparation) = prepare_axis_track_inputs(
            node,
            &input,
            axis,
            evaluated.contributions,
            axis_available(input.available, axis),
            GridAxisTrackSpace::Intrinsic,
        )?;
        match axis {
            GridAxisKind::Column => {
                columns = Some(sizes);
                column_preparation = Some(preparation);
                column_receipts = evaluated.receipts;
            }
            GridAxisKind::Row => {
                rows = Some(sizes);
                row_preparation = Some(preparation);
                row_receipts = evaluated.receipts;
            }
        }
    }
    // An unissued incomplete axis remains None even when it needs an estimate.
    let columns = columns.unwrap_or_else(|| {
        initial_estimate(input.column_tracks, input.context.percent_basis.inline)
    });
    let rows = rows
        .unwrap_or_else(|| initial_estimate(input.row_tracks, input.context.percent_basis.block));
    Ok(GridTrackResolution {
        sizing_phases: GridTrackSizingPhases { limit: input.stage },
        columns,
        rows,
        column_preparation,
        row_preparation,
    })
}

fn same_min_content_receipts<Node: PartialEq, S: LayoutScalar>(
    left: &[GridContributionReceipt<Node, S>],
    right: &[GridContributionReceipt<Node, S>],
) -> bool {
    left.iter()
        .filter(|receipt| receipt.purpose == GridContributionPurpose::MinContent)
        .map(|receipt| (&receipt.source, receipt.axis, receipt.span, receipt.value))
        .eq(right
            .iter()
            .filter(|receipt| receipt.purpose == GridContributionPurpose::MinContent)
            .map(|receipt| (&receipt.source, receipt.axis, receipt.span, receipt.value)))
}

fn same_min_content_inputs<S: LayoutScalar>(
    left: &GridAxisTrackInputs<S>,
    right: &GridAxisTrackInputs<S>,
) -> bool {
    left.min_content_base_floors == right.min_content_base_floors
        && left.rows.len() == right.rows.len()
        && left.rows.iter().zip(&right.rows).all(|(left, right)| {
            left.span == right.span
                && left.contributions.min_content == right.contributions.min_content
        })
}

fn initial_estimate<S: LayoutScalar>(tracks: &[TrackSizingOf<S>], basis: Option<S>) -> Vec<S> {
    tracks
        .iter()
        .map(|track| track.max.definite(basis).unwrap_or(S::ZERO))
        .collect()
}
fn axis_available<S: Copy>(available: LogicalSizeOf<S>, axis: GridAxisKind) -> S {
    match axis {
        GridAxisKind::Column => available.inline,
        GridAxisKind::Row => available.block,
    }
}
fn axis_tracks<'a, Node, S: LayoutScalar>(
    input: &GridTrackResolutionInput<'_, 'a, Node, S>,
    axis: GridAxisKind,
) -> &'a [TrackSizingOf<S>] {
    match axis {
        GridAxisKind::Column => input.column_tracks,
        GridAxisKind::Row => input.row_tracks,
    }
}
fn axis_alignment<Node, S: LayoutScalar>(
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
) -> AlignContent {
    match axis {
        GridAxisKind::Column => input.style.justify_content,
        GridAxisKind::Row => input.style.align_content,
    }
    .unwrap_or(AlignContent::Stretch)
}
fn axis_gutters<'a, Node, S: LayoutScalar>(
    input: &'a GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
) -> &'a OrdinaryGridAxisGuttersOf<S> {
    match axis {
        GridAxisKind::Column => &input.context.column_gutters,
        GridAxisKind::Row => &input.context.row_gutters,
    }
}
fn effective_available<Node, S: LayoutScalar>(
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    available: AvailableOf<S>,
) -> AvailableOf<S> {
    let flow = input.sizing_flow_axes;
    let inner = axis_available(flow.logical_size(input.constants.node_inner_size), axis);
    if let Some(inner) = inner {
        return AvailableOf::Definite(inner);
    }
    let available_inner = axis_available(
        flow.logical_size(input.constants.available_inner_size),
        axis,
    );
    match available {
        AvailableOf::Definite(_) if axis == GridAxisKind::Row && inner.is_none() => {
            AvailableOf::MAX_CONTENT
        }
        AvailableOf::Definite(_) => available_inner.map_or(available, |remaining| {
            // Requested scrollbar insets can exceed an unallocated max/available
            // outer extent. Track space saturates here; raw provisional facts
            // stay intact until the allocated scroll box qualifies its inset.
            // Nonfinite arithmetic must still reach typed numeric validation.
            AvailableOf::Definite(if remaining.is_finite() {
                remaining.max(S::ZERO)
            } else {
                remaining
            })
        }),
        intrinsic => intrinsic,
    }
}

pub(super) fn axis_track_available<Node, S: LayoutScalar>(
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
) -> AvailableOf<S> {
    effective_available(input, axis, axis_available(input.available, axis))
}

// Same-frame finishing only borrows the retained state. A receiver can supply
// an actual final stretch extent without changing percentage/constraint facts.
pub(super) fn finish_axis_track_preparation<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    preparation: &GridAxisTrackPreparation<S>,
    stretch_size: Option<S>,
) -> LayoutResultOf<Node, Vec<S>, S, M> {
    finish_axis_track_phases(
        node,
        input,
        axis,
        &preparation.phases,
        &preparation.contributions.rows,
        preparation.finishing_available,
        stretch_size,
    )
}
pub(super) fn finish_axis_track_phases<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    phases: &GridAxisTrackPhases<S>,
    items: &[GridIntrinsicContributionRow<S>],
    available: AvailableOf<S>,
    stretch_size: Option<S>,
) -> LayoutResultOf<Node, Vec<S>, S, M> {
    resolve_tracks_from_phases(ResolvedAxisTrackInput {
        tracks: axis_tracks(input, axis),
        phases,
        items,
        available,
        stretch_size,
        gap: axis_available(input.context.gap, axis),
        alignment: axis_alignment(input, axis),
        gutters: Some(axis_gutters(input, axis)),
    })
    .map_err(|error| grid_track_sizing_error(node, error))
}

// A genuine constraint change recomputes from retained normalized inputs. Any
// changed basis/gutter/opposite frame instead needs fresh owning collection.
pub(super) fn reduce_axis_track_inputs<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    contributions: &GridAxisTrackInputs<S>,
    mut available: AvailableOf<S>,
    space: GridAxisTrackSpace<S>,
) -> LayoutResultOf<Node, GridAxisTrackReduction<S>, S, M> {
    if let GridAxisTrackSpace::FinalUsed(value) = space {
        available = AvailableOf::Definite(value);
    }
    let tracks = axis_tracks(input, axis);
    let gap = axis_available(input.context.gap, axis);
    let gutters = axis_gutters(input, axis);
    let basis = axis_available(input.context.percent_basis, axis);
    let phases = resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks,
            basis,
            available,
            gap,
            gutters: Some(gutters),
            base_floors: &contributions.base_floors,
        },
        &contributions.rows,
    )
    .map_err(|error| grid_track_sizing_error(node, error))?;
    let flow = input.sizing_flow_axes;
    let inner = axis_available(flow.logical_size(input.constants.node_inner_size), axis);
    let maximum = if matches!(space, GridAxisTrackSpace::Intrinsic) && inner.is_none() {
        axis_available(flow.logical_size(input.constants.node_max_size), axis).map(|maximum| {
            let remaining = maximum
                - axis_available(
                    flow.logical_size(input.constants.content_box_inset.sum_axes()),
                    axis,
                );
            // Saturate a real exhausted budget; preserve nonfinite arithmetic
            // for the existing finisher's typed numeric validation.
            if remaining.is_finite() {
                remaining.max(S::ZERO)
            } else {
                remaining
            }
        })
    } else {
        None
    };
    let mut finishing_available = match (available, maximum) {
        (AvailableOf::MaxContent, Some(maximum)) => AvailableOf::Definite(maximum),
        _ => available,
    };
    let stretch = match space {
        GridAxisTrackSpace::Intrinsic => {
            let stretch_enabled = axis_available(input.intrinsic_max_available, axis)
                && inner.is_none()
                && axis_available(flow.logical_size(input.constants.node_max_size), axis).is_some();
            resolved_track_stretch_size(
                tracks,
                &phases,
                inner,
                finishing_available,
                axis_alignment(input, axis),
                stretch_enabled,
            )
            .map_err(|error| grid_track_sizing_error(node, error))?
        }
        GridAxisTrackSpace::FinalUsed(value) => Some(value),
    };
    let mut sizes = finish_axis_track_phases(
        node,
        input,
        axis,
        &phases,
        &contributions.rows,
        finishing_available,
        stretch,
    )?;
    if let Some(maximum) = maximum
        && finishing_available != AvailableOf::Definite(maximum)
        && track_sum_with_gutters(&sizes, gap, Some(gutters)) > maximum
    {
        finishing_available = AvailableOf::Definite(maximum);
        sizes = finish_axis_track_phases(
            node,
            input,
            axis,
            &phases,
            &contributions.rows,
            finishing_available,
            None,
        )?;
    }
    Ok(GridAxisTrackReduction {
        sizes,
        phases,
        available,
        finishing_available,
    })
}
fn prepare_axis_track_inputs<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    contributions: GridAxisTrackInputs<S>,
    available: AvailableOf<S>,
    space: GridAxisTrackSpace<S>,
) -> PreparedAxisResult<Node, S, M> {
    let reduced = reduce_axis_track_inputs(
        node,
        input,
        axis,
        &contributions,
        match space {
            GridAxisTrackSpace::Intrinsic => effective_available(input, axis, available),
            GridAxisTrackSpace::FinalUsed(value) => AvailableOf::Definite(value),
        },
        space,
    )?;
    Ok((
        reduced.sizes,
        GridAxisTrackPreparation {
            contributions,
            phases: reduced.phases,
            available: reduced.available,
            finishing_available: reduced.finishing_available,
        },
    ))
}

pub(super) fn collect_axis_track_preparation<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: &GridTrackResolutionInput<'input, '_, Tree::Node, Tree::Scalar>,
    grid: IntrinsicGrid<'input, '_, Tree::Node, Tree::Scalar>,
    frame: IntrinsicStageFrame<'_, Tree::Scalar>,
    space: GridAxisTrackSpace<Tree::Scalar>,
) -> PreparedAxisResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let evaluated = evaluate_stage(tree, node, input, grid, frame, space)?;
    prepare_axis_track_inputs(
        node,
        input,
        frame.axis,
        evaluated.contributions,
        axis_available(input.available, frame.axis),
        space,
    )
}

fn collect_purposes<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: &GridTrackResolutionInput<'input, '_, Tree::Node, Tree::Scalar>,
    grid: IntrinsicGrid<'input, '_, Tree::Node, Tree::Scalar>,
    frame: IntrinsicStageFrame<'_, Tree::Scalar>,
) -> StageResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let minimum = evaluate_role(tree, node, grid, frame, GridContributionPurpose::Minimum)?;
    let min_content = evaluate_role(tree, node, grid, frame, GridContributionPurpose::MinContent)?;
    let max_content = evaluate_role(tree, node, grid, frame, GridContributionPurpose::MaxContent)?;
    let group = GridAxisContributions::new(minimum, min_content, max_content);
    let mut rows =
        grid_contribution_rows(&group).map_err(|error| grid_track_sizing_error(node, error))?;
    if input
        .style
        .display
        .establishes_grid_lanes_formatting_context()
        && grid_axis_for_grid_lanes(input.style) == frame.axis
    {
        let lane_rows = lane_intrinsic_contribution_rows(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                grid,
                frame,
                container_style: input.style,
                constants: input.constants,
                axis: frame.axis,
                tracks: axis_tracks(input, frame.axis),
                gap: axis_available(input.context.gap, frame.axis),
                gutters: Some(axis_gutters(input, frame.axis)),
                lines: input.context.lines,
                placements: input.placements,
                subgrid_report: input.subgrid_report,
            },
        )?
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::InvalidIncidence))?;
        rows.try_reserve(lane_rows.len())
            .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
        rows.extend(lane_rows);
    }
    let GridAxisContributions {
        minimum,
        min_content,
        max_content,
        base_floors,
    } = group;
    let min_content_base_floors = min_content.base_floors;
    let mut receipts = minimum.receipts;
    let additional = min_content
        .receipts
        .len()
        .checked_add(max_content.receipts.len())
        .ok_or_else(|| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    receipts
        .try_reserve(additional)
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    receipts.extend(min_content.receipts);
    receipts.extend(max_content.receipts);
    Ok(EvaluatedAxis {
        contributions: GridAxisTrackInputs {
            rows,
            base_floors,
            min_content_base_floors,
        },
        receipts,
    })
}

fn evaluate_stage<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: &GridTrackResolutionInput<'input, '_, Tree::Node, Tree::Scalar>,
    grid: IntrinsicGrid<'input, '_, Tree::Node, Tree::Scalar>,
    frame: IntrinsicStageFrame<'_, Tree::Scalar>,
    space: GridAxisTrackSpace<Tree::Scalar>,
) -> StageResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let initial = collect_purposes(tree, node, input, grid, frame)?;
    if frame.observed_axis.is_some()
        || !grid
            .subgrid_report
            .items
            .iter()
            .any(|item| item_inherits_parent_axis(*item, frame.axis))
    {
        return Ok(initial);
    }
    let (tentative, _) = prepare_axis_track_inputs(
        node,
        input,
        frame.axis,
        initial.contributions,
        axis_available(input.available, frame.axis),
        space,
    )?;
    collect_purposes(
        tree,
        node,
        input,
        grid,
        IntrinsicStageFrame {
            observed_axis: Some(&tentative),
            ..frame
        },
    )
}

// All adapters use the same genuine per-role opposite-axis/source preparation.
pub(super) fn intrinsic_role_grid<'input, 'a, Node: Copy, S: LayoutScalar>(
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    frame: IntrinsicStageFrame<'a, S>,
    purpose: GridContributionPurpose,
) -> IntrinsicGrid<'input, 'a, Node, S> {
    let IntrinsicStageFrame {
        axis,
        stage,
        columns,
        rows,
        observed_axis,
    } = frame;
    let role = match purpose {
        GridContributionPurpose::Minimum | GridContributionPurpose::MinContent => {
            AvailableOf::MIN_CONTENT
        }
        GridContributionPurpose::MaxContent => AvailableOf::MAX_CONTENT,
    };
    let mut available = LogicalSizeOf::new(AvailableOf::MAX_CONTENT, AvailableOf::MAX_CONTENT);
    match axis {
        GridAxisKind::Column => {
            available.inline = role;
            if let Some(rows) = rows {
                available.block = AvailableOf::Definite(track_sum_with_gutters(
                    rows,
                    grid.gap.block,
                    grid.row_gutters,
                ));
            }
        }
        GridAxisKind::Row => {
            available.block = role;
            if let Some(columns) = columns {
                available.inline = AvailableOf::Definite(track_sum_with_gutters(
                    columns,
                    grid.gap.inline,
                    grid.column_gutters,
                ));
            }
        }
    }
    let estimated_grid = IntrinsicGrid {
        query: IntrinsicGridQuery {
            purpose,
            stage,
            ..grid.query
        },
        ..grid
    };
    match axis {
        GridAxisKind::Column if rows.is_none() => {
            if let Some(extent) =
                estimated_grid.initial_opposite_span(GridAxisKind::Row, 0, grid.row_tracks.len())
            {
                available.block = AvailableOf::Definite(extent);
            }
        }
        GridAxisKind::Row if columns.is_none() => {
            if let Some(extent) = estimated_grid.initial_opposite_span(
                GridAxisKind::Column,
                0,
                grid.column_tracks.len(),
            ) {
                available.inline = AvailableOf::Definite(extent);
            }
        }
        GridAxisKind::Column | GridAxisKind::Row => {}
    }
    let available = grid.sizing_flow_axes.physical_size(available);
    IntrinsicGrid {
        query: IntrinsicGridQuery {
            purpose,
            axis: Some(axis),
            stage,
            available,
            columns: if axis == GridAxisKind::Column {
                observed_axis.or(columns)
            } else {
                columns
            },
            rows: if axis == GridAxisKind::Row {
                observed_axis.or(rows)
            } else {
                rows
            },
            observe_auto: observed_axis.is_some(),
        },
        ..grid
    }
}

fn evaluate_role<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    grid: IntrinsicGrid<'input, '_, Tree::Node, Tree::Scalar>,
    frame: IntrinsicStageFrame<'_, Tree::Scalar>,
    purpose: GridContributionPurpose,
) -> RoleResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let grid = intrinsic_role_grid(grid, frame, purpose);
    intrinsic_track_sizes(
        tree,
        node,
        grid,
        grid.query.available,
        IntrinsicGridLowerBounds::default(),
    )
    .map(|result| result.into_axis(frame.axis))
}

pub(super) enum GridIntrinsicProbeKind {
    ContentContribution(GridAxisKind),
    AllocatedPrefix,
}

// A standalone axis reached while flattening the other axis still needs the
// original parent loan on each child scrollbar pass. Retaining only the first
// derived context would lose that source and permit a later full-size call.
#[derive(Clone, Debug)]
pub(super) struct IntrinsicSubgridSource<'input, Node, S: LayoutScalar> {
    item: SubgridItemReport<Node>,
    child_style: GridItemProjection<'input, S>,
    container: GridContainerProjection<'input, S>,
    area: GridArea<S>,
    content_box_size: Size<S>,
    columns: Vec<S>,
    rows: Vec<S>,
    column_minimum_functions: Option<Vec<GridMinimumTrackFact<S>>>,
    row_minimum_functions: Option<Vec<GridMinimumTrackFact<S>>>,
    gap: LogicalSizeOf<S>,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    column_geometry: Option<UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<UsedGridAxisGeometryOf<S>>,
    parent_flow: crate::geometry::FlowAxes,
    stage: GridSizingStage,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> IntrinsicSubgridSource<'input, Node, S> {
    pub(super) fn new(
        source: SubgridChildParentContextInput<'input, '_, Node, S>,
        column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        parent_flow: crate::geometry::FlowAxes,
        stage: GridSizingStage,
    ) -> Result<Self, SubgridChildContextError<S>> {
        let column_minimum_functions = source
            .minimum_functions
            .columns
            .map(GridMinimumTrackFunctions::retain)
            .transpose()
            .map_err(SubgridChildContextError::MinimumFunctions)?;
        let row_minimum_functions = source
            .minimum_functions
            .rows
            .map(GridMinimumTrackFunctions::retain)
            .transpose()
            .map_err(SubgridChildContextError::MinimumFunctions)?;
        Ok(Self {
            item: source.item,
            child_style: source.child_style.clone(),
            container: source
                .child_container_style
                .expect("a retained intrinsic subgrid has its original container"),
            area: source.area,
            content_box_size: source.content_box_size,
            columns: source.columns.to_vec(),
            rows: source.rows.to_vec(),
            column_minimum_functions,
            row_minimum_functions,
            gap: source.gap,
            named_columns: source.parent_named_columns.clone(),
            named_rows: source.parent_named_rows.clone(),
            area_facts: source.parent_area_facts.cloned(),
            margin: source.margin,
            border: source.border,
            padding: source.padding,
            column_geometry: column_geometry.cloned(),
            row_geometry: row_geometry.cloned(),
            parent_flow,
            stage,
        })
    }

    pub(super) fn measure<Tree, M>(
        &self,
        tree: &mut Tree,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let source = SubgridChildParentContextInput {
            item: self.item,
            child_style: &self.child_style,
            child_container_style: Some(self.container),
            area: self.area,
            content_box_size: self.content_box_size,
            columns: &self.columns,
            rows: &self.rows,
            minimum_functions: GridMinimumParentFunctions {
                columns: self
                    .column_minimum_functions
                    .as_deref()
                    .map(GridMinimumTrackFunctions::from_qualified),
                rows: self
                    .row_minimum_functions
                    .as_deref()
                    .map(GridMinimumTrackFunctions::from_qualified),
            },
            gap: self.gap,
            parent_named_columns: &self.named_columns,
            parent_named_rows: &self.named_rows,
            parent_area_facts: self.area_facts.as_ref(),
            margin: self.margin,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: Edges::ZERO,
        };
        match tree.observe_grid_overflow(
            self.item.node,
            GridOverflowProbe {
                layout: GridOverflowLayout::Intrinsic(GridIntrinsicProbe {
                    input,
                    stage: self.stage,
                    kind: GridIntrinsicProbeKind::AllocatedPrefix,
                    intrinsic_minimum: Size::NONE,
                    source,
                    parent_column_geometry: self.column_geometry.as_ref(),
                    parent_row_geometry: self.row_geometry.as_ref(),
                    parent_flow: self.parent_flow,
                }),
            },
        )? {
            GridOverflowObservation::Intrinsic(observation) => Ok(observation.output),
            GridOverflowObservation::Children(_) => {
                unreachable!("a standalone intrinsic source returns its prepared output")
            }
        }
    }
}

pub(crate) struct GridIntrinsicObservation<S: LayoutScalar> {
    pub(super) output: ComputeOutputOf<S>,
    pub(super) effective_gutter: Option<Edges<S>>,
}

pub(super) struct GridIntrinsicProbe<'input, 'a, Node, S: LayoutScalar> {
    pub(super) input: ComputeInputOf<S>,
    pub(super) stage: GridSizingStage,
    pub(super) kind: GridIntrinsicProbeKind,
    pub(super) intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
    pub(super) source: SubgridChildParentContextInput<'input, 'a, Node, S>,
    pub(super) parent_column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) parent_row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    pub(super) parent_flow: crate::geometry::FlowAxes,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridIntrinsicProbe<'input, '_, Node, S> {
    pub(super) fn observe<Tree, M>(
        self,
        tree: &mut Tree,
        node: Node,
    ) -> LayoutResultOf<Node, GridIntrinsicObservation<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let source = self.source;
        let child_flow = crate::geometry::FlowAxes::new(
            source.child_style.writing_mode,
            source.child_style.direction,
        );
        let stage = self.stage.mapped(self.parent_flow, child_flow);
        let result = compute_grid_with_context_source(
            tree,
            node,
            self.input,
            self.intrinsic_minimum,
            LogicalSizeOf::new(
                source.item.column.can_inherit(),
                source.item.row.can_inherit(),
            ),
            stage,
            |constants| {
                let mut columns = source.columns.to_vec();
                let mut rows = source.rows.to_vec();
                if let GridIntrinsicProbeKind::ContentContribution(queried_axis) = self.kind {
                    let report = match queried_axis {
                        GridAxisKind::Column => source.item.column,
                        GridAxisKind::Row => source.item.row,
                    };
                    let gutter = self.parent_flow.logical_edges(constants.scrollbar_gutter);
                    let (tracks, start, end, before, after) = match report.mapping.parent_axis {
                        GridAxisKind::Column => (
                            &mut columns,
                            source.area.column,
                            source.area.column_end,
                            gutter.inline_start,
                            gutter.inline_end,
                        ),
                        GridAxisKind::Row => (
                            &mut rows,
                            source.area.row,
                            source.area.row_end,
                            gutter.block_start,
                            gutter.block_end,
                        ),
                    };
                    if let Some(track) = tracks.get_mut(start) {
                        *track = *track + before;
                    }
                    if let Some(last) = end.checked_sub(1).and_then(|end| tracks.get_mut(end)) {
                        *last = *last + after;
                    }
                }
                let column_geometry = self.parent_column_geometry.map(|geometry| {
                    UsedGridAxisGeometryOf::from_sizing_gutters(
                        columns.clone(),
                        &geometry.sizing_gutters(),
                    )
                });
                let row_geometry = self.parent_row_geometry.map(|geometry| {
                    UsedGridAxisGeometryOf::from_sizing_gutters(
                        rows.clone(),
                        &geometry.sizing_gutters(),
                    )
                });
                let mut frame = source;
                frame.columns = &columns;
                frame.rows = &rows;
                let mut extra = Size::ZERO;
                if let GridIntrinsicProbeKind::ContentContribution(queried_axis) = self.kind {
                    let gutter_size = constants.scrollbar_gutter.sum_axes();
                    match grid_axis_physical_axis(child_flow, queried_axis) {
                        PhysicalAxis::Horizontal => extra.width = gutter_size.width,
                        PhysicalAxis::Vertical => extra.height = gutter_size.height,
                    }
                }
                frame.content_box_size = frame.content_box_size + extra;
                subgrid_child_parent_context_with_geometry(
                    frame.for_scrollbar_pass(constants),
                    column_geometry.as_ref(),
                    row_geometry.as_ref(),
                )
                .map_err(|error| subgrid_child_context_error(node, error))
            },
        )?;
        Ok(GridIntrinsicObservation {
            output: result.output,
            effective_gutter: result.effective_gutter,
        })
    }
}
