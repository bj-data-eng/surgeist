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

// One source-ordered track preparation, shared by completed layout and bounded
// subgrid observations. A row-first frame is the mapped orthogonal order.
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
        column_tracks: input.column_tracks,
        row_tracks: input.row_tracks,
        gap: input.context.gap,
        column_gutters: Some(&input.context.column_gutters),
        row_gutters: Some(&input.context.row_gutters),
        percent_basis: LogicalSizeOf::new(None, None),
        lines: input.context.lines,
        named_columns: &input.context.topology.named_columns,
        named_rows: &input.context.topology.named_rows,
        area_facts: input.context.topology.area_facts.as_ref(),
        subgrid_report: input.subgrid_report,
        placements: input.placements,
        query: IntrinsicGridQuery::complete(),
    };
    let first = input.stage.first_axis;
    let second = match first {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    };
    let mut columns = None;
    let mut rows = None;
    let mut column_min = Vec::new();
    let mut column_max = Vec::new();
    let mut row_min = Vec::new();
    let mut row_max = Vec::new();
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
        let stage = GridSizingStage {
            through: phase,
            ..input.stage
        };
        let minimum = evaluate_role(
            tree,
            node,
            &input,
            grid,
            IntrinsicRole {
                axis,
                stage,
                role: AvailableOf::MIN_CONTENT,
                columns: columns.as_deref(),
                rows: rows.as_deref(),
                lower_bound: None,
            },
        )?;
        let previous = match axis {
            GridAxisKind::Column => &column_receipts,
            GridAxisKind::Row => &row_receipts,
        };
        if phase >= GridSizingPhase::RevisitFirst && &minimum.receipts == previous {
            continue;
        }
        let maximum = evaluate_role(
            tree,
            node,
            &input,
            grid,
            IntrinsicRole {
                axis,
                stage,
                role: AvailableOf::MAX_CONTENT,
                columns: columns.as_deref(),
                rows: rows.as_deref(),
                lower_bound: Some(&minimum.sizes),
            },
        )?;
        let (minimum, receipts) = minimum.into_resolved();
        let (maximum, _) = maximum.into_resolved();
        let sizes = resolve_axis(
            &input,
            axis,
            &minimum,
            &maximum,
            axis_available(input.available, axis),
        );
        match axis {
            GridAxisKind::Column => {
                columns = Some(sizes);
                column_min = minimum;
                column_max = maximum;
                column_receipts = receipts;
            }
            GridAxisKind::Row => {
                rows = Some(sizes);
                row_min = minimum;
                row_max = maximum;
                row_receipts = receipts;
            }
        }
    }

    // These estimates are only present in an incomplete prepared frame. Their
    // unresolved provenance is retained by the frame's stage; they are never
    // published as a completed track allocation.
    let columns = columns.unwrap_or_else(|| {
        initial_estimate(input.column_tracks, input.context.percent_basis.inline)
    });
    let rows = rows
        .unwrap_or_else(|| initial_estimate(input.row_tracks, input.context.percent_basis.block));
    let row_intrinsic_sizes = if input.available.block == AvailableOf::MIN_CONTENT {
        row_min
    } else {
        track_resolution_intrinsic_sizes(input.row_tracks, &row_min, &row_max)
    };
    Ok(GridTrackResolution {
        sizing_phases: GridTrackSizingPhases { limit: input.stage },
        columns,
        rows,
        column_min_intrinsic_sizes: column_min,
        column_max_intrinsic_sizes: column_max,
        row_intrinsic_sizes,
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

fn resolve_axis<S: LayoutScalar, Node>(
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    minimum: &[S],
    maximum: &[S],
    available: AvailableOf<S>,
) -> Vec<S> {
    let constants = input.constants;
    let tracks = match axis {
        GridAxisKind::Column => input.column_tracks,
        GridAxisKind::Row => input.row_tracks,
    };
    let flow = input.sizing_flow_axes;
    let inner = axis_available(flow.logical_size(constants.node_inner_size), axis);
    let available_inner = axis_available(flow.logical_size(constants.available_inner_size), axis);
    let track_available = match available {
        AvailableOf::Definite(_) if axis == GridAxisKind::Row && inner.is_none() => {
            AvailableOf::MAX_CONTENT
        }
        AvailableOf::Definite(_) => available_inner.map_or(available, AvailableOf::Definite),
        intrinsic => intrinsic,
    };
    let mixed = track_resolution_intrinsic_sizes(tracks, minimum, maximum);
    let selected = if available == AvailableOf::MIN_CONTENT {
        minimum
    } else {
        &mixed
    };
    let gap = axis_available(input.context.gap, axis);
    let gutters = match axis {
        GridAxisKind::Column => &input.context.column_gutters,
        GridAxisKind::Row => &input.context.row_gutters,
    };
    let alignment = match axis {
        GridAxisKind::Column => input.style.justify_content,
        GridAxisKind::Row => input.style.align_content,
    }
    .unwrap_or(AlignContent::Stretch);
    let mut axis_input = AxisTrackInput {
        tracks,
        basis: axis_available(input.context.percent_basis, axis),
        definite_size: inner,
        available_size: track_available,
        gap,
        gutters: Some(gutters),
        alignment,
        stretch_empty_auto_to_available: axis_available(input.intrinsic_max_available, axis)
            && inner.is_none()
            && axis_available(flow.logical_size(constants.node_max_size), axis).is_some(),
        min_intrinsic_sizes: minimum,
        max_intrinsic_sizes: selected,
    };
    let mut sizes = resolve_axis_tracks(axis_input);
    if inner.is_none()
        && let Some(maximum) = axis_available(flow.logical_size(constants.node_max_size), axis)
    {
        let maximum = (maximum
            - axis_available(
                flow.logical_size(constants.content_box_inset.sum_axes()),
                axis,
            ))
        .max(S::ZERO);
        if track_sum_with_gutters(&sizes, gap, Some(gutters)) > maximum {
            axis_input.available_size = AvailableOf::Definite(maximum);
            axis_input.stretch_empty_auto_to_available = false;
            sizes = resolve_axis_tracks(axis_input);
        }
    }
    sizes
}

struct IntrinsicRole<'a, S: LayoutScalar> {
    axis: GridAxisKind,
    stage: GridSizingStage,
    role: AvailableOf<S>,
    columns: Option<&'a [S]>,
    rows: Option<&'a [S]>,
    lower_bound: Option<&'a [S]>,
}

type RoleResult<Node, S, M> = LayoutResultOf<Node, IntrinsicAxisContributions<Node, S>, S, M>;

// Each role starts without automatic bars. One actual tentative content frame
// is then observed using the canonical intrinsic-add semantics. The child's
// own finite scrollbar settlement is separate from the four parent stages.
fn evaluate_role<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: &GridTrackResolutionInput<'input, '_, Tree::Node, Tree::Scalar>,
    grid: IntrinsicGrid<'input, '_, Tree::Node, Tree::Scalar>,
    role_input: IntrinsicRole<'_, Tree::Scalar>,
) -> RoleResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let IntrinsicRole {
        axis,
        stage,
        role,
        columns,
        rows,
        lower_bound,
    } = role_input;
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
    let grid = IntrinsicGrid {
        query: IntrinsicGridQuery {
            axis: Some(axis),
            stage,
            available,
            columns,
            rows,
            observe_auto: false,
        },
        ..grid
    };
    let bounds = match axis {
        GridAxisKind::Column => IntrinsicGridLowerBounds {
            columns: lower_bound,
            rows: None,
        },
        GridAxisKind::Row => IntrinsicGridLowerBounds {
            columns: None,
            rows: lower_bound,
        },
    };
    let initial = intrinsic_track_sizes(tree, node, grid, available, bounds)?;
    let mut contributions = initial.into_axis(axis);
    if grid
        .subgrid_report
        .items
        .iter()
        .any(|item| item_inherits_parent_axis(*item, axis))
    {
        // Layout observes the role's converted axis output, while a later
        // MAX distribution still borrows its unconverted MIN floors.
        let observation_sizes = contributions.observation_sizes();
        let tentative = resolve_axis(input, axis, &observation_sizes, &observation_sizes, role);
        let query = IntrinsicGridQuery {
            columns: if axis == GridAxisKind::Column {
                Some(&tentative)
            } else {
                columns
            },
            rows: if axis == GridAxisKind::Row {
                Some(&tentative)
            } else {
                rows
            },
            observe_auto: true,
            ..grid.query
        };
        let observed = intrinsic_track_sizes(
            tree,
            node,
            IntrinsicGrid { query, ..grid },
            available,
            bounds,
        )?;
        contributions = observed.into_axis(axis);
    }
    if input
        .style
        .display
        .establishes_grid_lanes_formatting_context()
        && grid_axis_for_grid_lanes(input.style) == axis
    {
        let tracks = match axis {
            GridAxisKind::Column => input.column_tracks,
            GridAxisKind::Row => input.row_tracks,
        };
        let lane_sizes = lane_intrinsic_track_sizes(
            tree,
            node,
            LaneIntrinsicTrackSizeInput {
                container_style: input.style,
                constants: input.constants,
                axis,
                tracks,
                gap: axis_available(input.context.gap, axis),
                available: role,
                available_basis: axis_available(input.context.percent_basis, axis),
                gutters: Some(match axis {
                    GridAxisKind::Column => &input.context.column_gutters,
                    GridAxisKind::Row => &input.context.row_gutters,
                }),
                lines: input.context.lines,
                placements: input.placements,
                subgrid_report: input.subgrid_report,
            },
        )?;
        merge_lane_intrinsic_lower_bounds(&mut contributions.sizes, lane_sizes);
    }
    Ok(contributions)
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
    gap: LogicalSizeOf<S>,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    baseline_groups: GridBaselineGroups<S>,
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
    ) -> Self {
        Self {
            item: source.item,
            child_style: source.child_style.clone(),
            container: source
                .child_container_style
                .expect("a retained intrinsic subgrid has its original container"),
            area: source.area,
            content_box_size: source.content_box_size,
            columns: source.columns.to_vec(),
            rows: source.rows.to_vec(),
            gap: source.gap,
            named_columns: source.parent_named_columns.clone(),
            named_rows: source.parent_named_rows.clone(),
            area_facts: source.parent_area_facts.cloned(),
            baseline_groups: source.parent_baseline_groups.clone(),
            margin: source.margin,
            border: source.border,
            padding: source.padding,
            column_geometry: column_geometry.cloned(),
            row_geometry: row_geometry.cloned(),
            parent_flow,
            stage,
        }
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
            gap: self.gap,
            parent_named_columns: &self.named_columns,
            parent_named_rows: &self.named_rows,
            parent_area_facts: self.area_facts.as_ref(),
            parent_baseline_groups: &self.baseline_groups,
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
