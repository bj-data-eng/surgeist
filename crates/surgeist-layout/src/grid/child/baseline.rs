use super::*;

pub(in crate::grid) type OwnerProgressionTrackFrameOrigins<S> = (Vec<S>, Vec<S>, Vec<S>, Vec<S>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::grid) enum BaselineGroupKind {
    Major,
    Minor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct BaselineParticipation {
    pub(in crate::grid) participates: bool,
    pub(in crate::grid) group: Option<BaselineGroupKind>,
    pub(in crate::grid) synthesized: bool,
    pub(in crate::grid) fallback_alignment: Option<AlignItems>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ExecutedBaselineBody<Node, S: LayoutScalar> {
    node: Node,
    parent: Node,
    local_span: GridTrackSpan,
    axes: FlowAxes,
    size: Size<S>,
    first: Option<S>,
    last: Option<S>,
    selected_port: S,
    margin: SubgridAxisEdges<S>,
    cycle: bool,
}

/// The source facts stay in their genuine immediate-parent frame. The
/// projection is explicitly normalized to this inventory's owning grid.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ExecutedBaselineConstraint<Node, S: LayoutScalar> {
    source: ExecutedBaselineBody<Node, S>,
    projection: AncestorBaselineMember<Node, S>,
    point: S,
}

#[derive(Debug, PartialEq)]
pub(in crate::grid) struct ExecutedBaselineInventory<Node, S: LayoutScalar> {
    owner: Node,
    axes: FlowAxes,
    size: Size<S>,
    execution_input: ComputeInputOf<S>,
    stage: GridSizingStage,
    counts: LogicalSizeOf<Option<usize>>,
    constraints: Vec<ExecutedBaselineConstraint<Node, S>>,
}

pub(in crate::grid) enum PendingExecutedBaselines<Node, S: LayoutScalar> {
    Uncaptured(Option<ExecutedBaselineInventory<Node, S>>),
    Captured(core::ops::Range<usize>),
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct ExecutedBaselineAxisFrame<'a, S: LayoutScalar> {
    pub(in crate::grid) starts: &'a [S],
    pub(in crate::grid) sizes: &'a [S],
}

impl<S: LayoutScalar> ExecutedBaselineAxisFrame<'_, S> {
    fn edges(self, index: usize) -> Result<(S, S), InheritedCurrentGridBaselinePlacementError> {
        let start = self
            .starts
            .get(index)
            .copied()
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        let size = self
            .sizes
            .get(index)
            .copied()
            .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
        let end = start + size;
        if !start.is_finite() || !end.is_finite() {
            return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
        }
        Ok((start, end))
    }
}

#[derive(Clone, Copy)]
pub(in crate::grid) enum ExecutedBaselineNomination<'a, S: LayoutScalar> {
    Grid {
        row_tracks: &'a [TrackSizingOf<S>],
        track_basis: LogicalSizeOf<Option<S>>,
    },
    // Lane exports only its real grid-axis candidates to an inheriting owner.
    // The established flattened synthesized-span exclusion applies there.
    LaneInherited,
}

#[derive(Clone, Copy)]
pub(in crate::grid) struct ExecutedBaselineFrame<'a, S: LayoutScalar> {
    pub(in crate::grid) style: &'a GridContainerProjection<'a, S>,
    pub(in crate::grid) size: Size<S>,
    pub(in crate::grid) execution_input: ComputeInputOf<S>,
    pub(in crate::grid) stage: GridSizingStage,
    pub(in crate::grid) columns: Option<ExecutedBaselineAxisFrame<'a, S>>,
    pub(in crate::grid) rows: Option<ExecutedBaselineAxisFrame<'a, S>>,
    pub(in crate::grid) nomination: ExecutedBaselineNomination<'a, S>,
}

impl<'a, S: LayoutScalar> ExecutedBaselineFrame<'a, S> {
    fn axes(self) -> FlowAxes {
        FlowAxes::new(self.style.writing_mode, self.style.direction)
    }
    fn axis(self, axis: GridAxisKind) -> Option<ExecutedBaselineAxisFrame<'a, S>> {
        match axis {
            GridAxisKind::Column => self.columns,
            GridAxisKind::Row => self.rows,
        }
    }
}

fn executed_baseline_error<Node: Copy, S: LayoutScalar, M>(
    owner: Node,
    subject: Node,
    error: InheritedCurrentGridBaselinePlacementError,
) -> LayoutErrorOf<Node, S, M> {
    subgrid_child_context_container_error(
        owner,
        subject,
        SubgridChildContextError::BaselineInheritance(error),
    )
}

impl<Node: Copy + PartialEq, S: LayoutScalar> ExecutedBaselineInventory<Node, S> {
    pub(in crate::grid) fn empty(owner: Node, frame: ExecutedBaselineFrame<'_, S>) -> Self {
        Self {
            owner,
            axes: frame.axes(),
            size: frame.size,
            execution_input: frame.execution_input,
            stage: frame.stage,
            counts: LogicalSizeOf::new(
                frame.columns.map(|axis| axis.sizes.len()),
                frame.rows.map(|axis| axis.sizes.len()),
            ),
            constraints: Vec::new(),
        }
    }

    fn count(&self, axis: GridAxisKind) -> Option<usize> {
        match axis {
            GridAxisKind::Column => self.counts.inline,
            GridAxisKind::Row => self.counts.block,
        }
    }

    pub(in crate::grid) fn validate_execution(
        &self,
        owner: Node,
        axes: FlowAxes,
        size: Size<S>,
        input: ComputeInputOf<S>,
        stage: GridSizingStage,
    ) -> Result<(), InheritedCurrentGridBaselinePlacementError> {
        // The child's own settled scrollbar pass is part of its actual result.
        // Every other canonical request operand must be the caller's request.
        let expected =
            input.with_settled_auto_scrollbars(self.execution_input.settled_auto_scrollbars());
        if self.owner != owner
            || self.axes != axes
            || self.size != size
            || self.stage != stage
            || self.execution_input != expected
        {
            return Err(InheritedCurrentGridBaselinePlacementError::ExecutedSourceMismatch);
        }
        Ok(())
    }
}

pub(in crate::grid) fn require_executed_baselines<Node: Copy + PartialEq, S: LayoutScalar, M>(
    owner: Node,
    child: Node,
    axes: FlowAxes,
    output: ComputeOutputOf<S>,
    input: ComputeInputOf<S>,
    stage: GridSizingStage,
    source: Option<ExecutedBaselineInventory<Node, S>>,
) -> LayoutResultOf<Node, ExecutedBaselineInventory<Node, S>, S, M> {
    let source = source.ok_or_else(|| {
        executed_baseline_error(
            owner,
            child,
            InheritedCurrentGridBaselinePlacementError::MissingExecutedSource,
        )
    })?;
    source
        .validate_execution(child, axes, output.size, input, stage)
        .map_err(|error| executed_baseline_error(owner, child, error))?;
    Ok(source)
}

fn push_executed_constraint<Node: Copy, S: LayoutScalar, M>(
    owner: Node,
    subject: Node,
    output: &mut Vec<ExecutedBaselineConstraint<Node, S>>,
    constraint: ExecutedBaselineConstraint<Node, S>,
) -> LayoutResultOf<Node, (), S, M> {
    output.try_reserve(1).map_err(|_| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject {
                container: owner,
                subject,
            },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::Allocation,
        )
    })?;
    output.push(constraint);
    Ok(())
}

fn append_executed_item<'input, Node: Copy + PartialEq, S: LayoutScalar, M>(
    owner: Node,
    frame: ExecutedBaselineFrame<'_, S>,
    item: &PendingGridItem<'input, Node, S>,
    report: SubgridItemReport<Node>,
    child_source: Option<&ExecutedBaselineInventory<Node, S>>,
    output: &mut Vec<ExecutedBaselineConstraint<Node, S>>,
) -> LayoutResultOf<Node, (), S, M> {
    let fail = |error| executed_baseline_error(owner, item.node, error);
    let parent_axes = frame.axes();
    for axis in [GridAxisKind::Column, GridAxisKind::Row] {
        let Some(geometry) = frame.axis(axis) else {
            continue;
        };
        let inherited = [
            (GridAxisKind::Column, report.column),
            (GridAxisKind::Row, report.row),
        ]
        .into_iter()
        .find(|(_, report)| report.can_inherit() && report.mapping.parent_axis == axis);
        if let Some((child_axis, inherited)) = inherited {
            let source = child_source.ok_or_else(|| {
                fail(InheritedCurrentGridBaselinePlacementError::MissingExecutedSource)
            })?;
            let child_count = source.count(child_axis).ok_or_else(|| {
                fail(InheritedCurrentGridBaselinePlacementError::MissingExecutedSource)
            })?;
            let (start, end, offset) = match axis {
                GridAxisKind::Column => (
                    item.area.column,
                    item.area.column_end,
                    item.horizontal_axis.offset,
                ),
                GridAxisKind::Row => (item.area.row, item.area.row_end, item.vertical_axis.offset),
            };
            if end.checked_sub(start) != Some(child_count) || end > geometry.sizes.len() {
                return Err(fail(
                    InheritedCurrentGridBaselinePlacementError::SpanOutOfRange,
                ));
            }
            let physical = grid_axis_physical_axis(parent_axes, axis);
            if physical != grid_axis_physical_axis(source.axes, child_axis) {
                return Err(fail(
                    InheritedCurrentGridBaselinePlacementError::PhysicalAxisMismatch,
                ));
            }
            let parent_progression = parent_axes.physical_axis_progression(physical);
            let child_progression = source.axes.physical_axis_progression(physical);
            if inherited.mapping.reversed != (parent_progression != child_progression) {
                return Err(fail(
                    InheritedCurrentGridBaselinePlacementError::UnusableInheritedMapping,
                ));
            }
            let border_start = geometry.edges(start).map_err(fail)?.0 + offset;
            let extent = grid_axis_size(parent_axes, item.output.size, axis);
            for constraint in source
                .constraints
                .iter()
                .filter(|constraint| constraint.projection.axis() == child_axis)
            {
                let span = constraint.projection.span();
                let absent_selected = match constraint.projection.selected_port() {
                    AncestorBaselineRole::First => constraint.source.first.is_none(),
                    AncestorBaselineRole::Last => constraint.source.last.is_none(),
                };
                // Preserve the existing flattened final-census cycle guard;
                // direct Column nomination remains its separate native lane.
                if span.end > span.start.saturating_add(1) && absent_selected {
                    continue;
                }
                let selected = constraint.projection.selected_track();
                if span.start == 0
                    || span.end <= span.start
                    || span.end > child_count + 1
                    || selected >= child_count
                {
                    return Err(fail(
                        InheritedCurrentGridBaselinePlacementError::SpanOutOfRange,
                    ));
                }
                let reversed = inherited.mapping.reversed;
                let mapped_span = if reversed {
                    GridTrackSpan::new(end + 2 - span.end, end + 2 - span.start)
                } else {
                    GridTrackSpan::new(start + span.start, start + span.end)
                };
                let mapped_track = if reversed {
                    end - 1 - selected
                } else {
                    start + selected
                };
                let point = border_start
                    + if reversed {
                        extent - constraint.point
                    } else {
                        constraint.point
                    };
                if !point.is_finite() {
                    return Err(fail(InheritedCurrentGridBaselinePlacementError::NonFinite));
                }
                let (selected_start, selected_end) = geometry.edges(mapped_track).map_err(fail)?;
                let projection = constraint
                    .projection
                    .project_final(FinalBaselineProjectionInput {
                        axis,
                        span: mapped_span,
                        reversed: constraint.projection.context_reversed() != reversed,
                        progression: parent_progression,
                        point,
                        selected_start,
                        selected_end,
                        extent: constraint
                            .source
                            .axes
                            .block_axis_extent(constraint.source.size),
                        physical_port: constraint.source.selected_port,
                    })
                    .map_err(fail)?;
                if projection.selected_track() != mapped_track {
                    return Err(fail(
                        InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange,
                    ));
                }
                push_executed_constraint(
                    owner,
                    item.node,
                    output,
                    ExecutedBaselineConstraint {
                        source: constraint.source,
                        projection,
                        point,
                    },
                )?;
            }
            continue;
        }
        let alignment = match axis {
            GridAxisKind::Column => item.style.justify_self.or(frame.style.justify_items),
            GridAxisKind::Row => item.style.align_self.or(frame.style.align_items),
        }
        .unwrap_or(AlignItems::Stretch);
        let (start, end) = match axis {
            GridAxisKind::Column => (item.area.column, item.area.column_end),
            GridAxisKind::Row => (item.area.row, item.area.row_end),
        };
        let auto = matches!(
            item.child_flow_axes.line_over_edge(item.style.margin),
            LengthAutoOf::Auto
        ) || matches!(
            item.child_flow_axes.line_under_edge(item.style.margin),
            LengthAutoOf::Auto
        );
        let cycle = match frame.nomination {
            ExecutedBaselineNomination::Grid {
                row_tracks,
                track_basis,
            } => {
                axis == GridAxisKind::Row
                    && synthesized_baseline_would_cycle(
                        alignment,
                        item.output.baselines(),
                        item.child_flow_axes,
                        row_tracks.get(start..end).unwrap_or(&[]),
                        track_basis.block,
                    )
            }
            ExecutedBaselineNomination::LaneInherited => {
                end > start.saturating_add(1)
                    && match alignment {
                        AlignItems::Baseline => item
                            .output
                            .baselines()
                            .first_block(item.child_flow_axes)
                            .is_none(),
                        AlignItems::LastBaseline => item
                            .output
                            .baselines()
                            .last_block(item.child_flow_axes)
                            .is_none(),
                        _ => false,
                    }
            }
        };
        let Some(projection) = ancestor_baseline_member(AncestorBaselineMemberInput {
            source: item.node,
            axis,
            ancestor_span: GridTrackSpan::new(start + 1, end + 1),
            alignment_context_reversed: false,
            alignment,
            block_auto_margins: auto,
            synthesized_baseline_cycle: cycle,
            output: item.output,
            margin: item.margin,
            child_flow_axes: item.child_flow_axes,
            containing_flow_axes: parent_axes,
            start_adjustment: S::ZERO,
            end_adjustment: S::ZERO,
        }) else {
            continue;
        };
        let (selected_start, selected_end) =
            geometry.edges(projection.selected_track()).map_err(fail)?;
        let point = match projection.role() {
            AncestorBaselineRole::First => selected_start + projection.distance(),
            AncestorBaselineRole::Last => selected_end - projection.distance(),
        };
        if !point.is_finite() {
            return Err(fail(InheritedCurrentGridBaselinePlacementError::NonFinite));
        }
        let baselines = item.output.baselines();
        let selected_port = match projection.selected_port() {
            AncestorBaselineRole::First => {
                baselines.first_or_synthesize_block(item.child_flow_axes, item.output.size)
            }
            AncestorBaselineRole::Last => {
                baselines.last_or_synthesize_block(item.child_flow_axes, item.output.size)
            }
        };
        let margin = match item.child_flow_axes.block_axis() {
            PhysicalAxis::Horizontal => SubgridAxisEdges {
                start: item.margin.left,
                end: item.margin.right,
            },
            PhysicalAxis::Vertical => SubgridAxisEdges {
                start: item.margin.top,
                end: item.margin.bottom,
            },
        };
        push_executed_constraint(
            owner,
            item.node,
            output,
            ExecutedBaselineConstraint {
                source: ExecutedBaselineBody {
                    node: item.node,
                    parent: owner,
                    local_span: GridTrackSpan::new(start + 1, end + 1),
                    axes: item.child_flow_axes,
                    size: item.output.size,
                    first: baselines.first_block(item.child_flow_axes),
                    last: baselines.last_block(item.child_flow_axes),
                    selected_port,
                    margin,
                    cycle,
                },
                projection,
                point,
            },
        )?;
    }
    Ok(())
}

pub(in crate::grid) fn capture_executed_baselines<
    'input,
    Node: Copy + PartialEq,
    S: LayoutScalar,
    M,
>(
    owner: Node,
    frame: ExecutedBaselineFrame<'_, S>,
    items: &mut [PendingGridItem<'input, Node, S>],
    report: &GridSubgridReport<Node>,
) -> LayoutResultOf<Node, ExecutedBaselineInventory<Node, S>, S, M> {
    let mut inventory = ExecutedBaselineInventory::empty(owner, frame);
    for item in items {
        let pending = core::mem::replace(
            &mut item.executed_baselines,
            PendingExecutedBaselines::Uncaptured(None),
        );
        let PendingExecutedBaselines::Uncaptured(source) = pending else {
            return Err(executed_baseline_error(
                owner,
                item.node,
                InheritedCurrentGridBaselinePlacementError::ExecutedSourceMismatch,
            ));
        };
        let child_report = report
            .items
            .get(item.source_index)
            .copied()
            .ok_or_else(|| {
                executed_baseline_error(
                    owner,
                    item.node,
                    InheritedCurrentGridBaselinePlacementError::SpanOutOfRange,
                )
            })?;
        let start = inventory.constraints.len();
        append_executed_item(
            owner,
            frame,
            item,
            child_report,
            source.as_ref(),
            &mut inventory.constraints,
        )?;
        item.executed_baselines =
            PendingExecutedBaselines::Captured(start..inventory.constraints.len());
    }
    Ok(inventory)
}

pub(in crate::grid) fn verify_refreshed_baselines<
    'input,
    Node: Copy + PartialEq,
    S: LayoutScalar,
    M,
>(
    owner: Node,
    frame: ExecutedBaselineFrame<'_, S>,
    item: &PendingGridItem<'input, Node, S>,
    report: SubgridItemReport<Node>,
    source: &ExecutedBaselineInventory<Node, S>,
    original: &ExecutedBaselineInventory<Node, S>,
) -> LayoutResultOf<Node, (), S, M> {
    let PendingExecutedBaselines::Captured(range) = &item.executed_baselines else {
        return Err(executed_baseline_error(
            owner,
            item.node,
            InheritedCurrentGridBaselinePlacementError::ExecutedSourceMismatch,
        ));
    };
    let mut refreshed = Vec::new();
    append_executed_item(owner, frame, item, report, Some(source), &mut refreshed)?;
    if original.constraints.get(range.clone()) != Some(refreshed.as_slice()) {
        return Err(executed_baseline_error(
            owner,
            item.node,
            InheritedCurrentGridBaselinePlacementError::ExecutedSourceMismatch,
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct BaselineGeometry<S: LayoutScalar = Scalar> {
    pub(in crate::grid) available_span_size: S,
    pub(in crate::grid) margin_box_size: S,
    // Border-box baselines are stored separately on PendingGridItem. These
    // fields are the margin-box contributions used by shared baseline groups:
    // block-start margin plus first baseline for major groups, and block-end
    // margin plus distance from last baseline to block-end for minor groups.
    pub(in crate::grid) major_baseline: PhysicalBaseline<S>,
    pub(in crate::grid) minor_baseline: PhysicalBaseline<S>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct TrackBaselineGroup<S: LayoutScalar = Scalar> {
    pub(in crate::grid) first: Option<PhysicalBaseline<S>>,
    pub(in crate::grid) last: Option<PhysicalBaseline<S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct GridBaselineGroups<S: LayoutScalar = Scalar> {
    pub(in crate::grid) rows: Vec<TrackBaselineGroup<S>>,
    pub(in crate::grid) columns: Vec<TrackBaselineGroup<S>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::grid) struct FinalAncestorBaselineGroups<Node, S: LayoutScalar = Scalar> {
    rows: AncestorBaselineGroup<Node, S>,
    columns: AncestorBaselineGroup<Node, S>,
    placement_rows: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
    placement_columns: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
}

impl<Node: Copy + PartialEq, S: LayoutScalar> FinalAncestorBaselineGroups<Node, S> {
    pub(super) fn with_parent_context(
        mut self,
        parent_context: &GridParentContext<S, Node>,
    ) -> Self {
        if let Some(targets) = parent_context
            .rows
            .as_ref()
            .and_then(|axis| axis.owner_baseline_targets.as_ref())
        {
            self.placement_rows = Some(targets.clone());
        }
        if let Some(targets) = parent_context
            .columns
            .as_ref()
            .and_then(|axis| axis.owner_baseline_targets.as_ref())
        {
            self.placement_columns = Some(targets.clone());
        }
        self
    }
    pub(super) fn placement_groups(&self) -> GridBaselineGroups<S> {
        GridBaselineGroups {
            rows: self.rows.track_groups(),
            columns: self
                .placement_columns
                .as_ref()
                .map(|targets| targets.group.track_groups())
                .unwrap_or_else(|| self.columns.track_groups()),
        }
    }
    pub(super) fn for_axis(&self, axis: GridAxisKind) -> &AncestorBaselineGroup<Node, S> {
        match axis {
            GridAxisKind::Column => &self.columns,
            GridAxisKind::Row => &self.rows,
        }
    }
    pub(super) fn inherited_targets_for_axis(
        &self,
        axis: GridAxisKind,
    ) -> Option<&InheritedGridOwnerBaselineTargets<Node, S>> {
        match axis {
            GridAxisKind::Column => self.placement_columns.as_ref(),
            GridAxisKind::Row => self.placement_rows.as_ref(),
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn final_ancestor_baseline_groups_for_transport_test<S: LayoutScalar>(
    rows: AncestorBaselineGroup<u32, S>,
    columns: AncestorBaselineGroup<u32, S>,
    owner_flow: FlowAxes,
) -> FinalAncestorBaselineGroups<u32, S> {
    let placement_rows = Some(InheritedGridOwnerBaselineTargets {
        group: rows.clone(),
        mapping: CheckedOwnerToCurrentPlacementMap::identity(
            rows.owner(),
            GridAxisKind::Row,
            rows.physical_axis(),
            owner_flow.physical_axis_progression(rows.physical_axis()),
            rows.track_count(),
        ),
    });
    let placement_columns = Some(InheritedGridOwnerBaselineTargets {
        group: columns.clone(),
        mapping: CheckedOwnerToCurrentPlacementMap::identity(
            columns.owner(),
            GridAxisKind::Column,
            columns.physical_axis(),
            owner_flow.physical_axis_progression(columns.physical_axis()),
            columns.track_count(),
        ),
    });
    FinalAncestorBaselineGroups {
        rows,
        columns,
        placement_rows,
        placement_columns,
    }
}

#[cfg(test)]
pub(in crate::grid) fn final_ancestor_baseline_groups_with_parent_context_for_transport_test<
    S: LayoutScalar,
>(
    groups: FinalAncestorBaselineGroups<u32, S>,
    parent_context: &GridParentContext<S, u32>,
) -> FinalAncestorBaselineGroups<u32, S> {
    groups.with_parent_context(parent_context)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridContainerBaselines<S: LayoutScalar = Scalar> {
    pub(in crate::grid) baselines: BaselinesOf<S>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::grid) struct BaselineShim<S: LayoutScalar = Scalar> {
    pub(in crate::grid) before: S,
    pub(in crate::grid) after: S,
}

#[cfg(test)]
impl<S: LayoutScalar> GridBaselineGroups<S> {
    fn shared_baseline(
        &self,
        group_kind: BaselineGroupKind,
        area: GridArea<S>,
    ) -> Option<PhysicalBaseline<S>> {
        match group_kind {
            BaselineGroupKind::Major => self.rows.get(area.row)?.first,
            BaselineGroupKind::Minor => {
                let row = area.row_end.checked_sub(1)?;
                self.rows.get(row)?.last
            }
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn baseline_offset<S: LayoutScalar>(
    group_kind: BaselineGroupKind,
    shared_baseline: PhysicalBaseline<S>,
    geometry: BaselineGeometry<S>,
    expected_axis: PhysicalAxis,
) -> Option<S> {
    let shared_baseline = shared_baseline.coordinate_on(expected_axis)?;
    match group_kind {
        BaselineGroupKind::Major => geometry
            .major_baseline
            .coordinate_on(expected_axis)
            .map(|baseline| shared_baseline - baseline),
        BaselineGroupKind::Minor => {
            geometry
                .minor_baseline
                .coordinate_on(expected_axis)
                .map(|baseline| {
                    let baseline_delta = shared_baseline - baseline;
                    geometry.available_span_size - baseline_delta - geometry.margin_box_size
                })
        }
    }
}

#[cfg(test)]
pub(in crate::grid) fn baseline_aligned_block_offset<'input, Node: Copy, S: LayoutScalar>(
    item: &PendingGridItem<'input, Node, S>,
    groups: &GridBaselineGroups<S>,
    rows: &[S],
    row_gap: S,
    container_flow_axes: FlowAxes,
) -> Option<S> {
    if !item.baseline_participation.participates || item.block_auto_margins {
        return None;
    }

    let group_kind = item.baseline_participation.group?;
    let shared = groups.shared_baseline(group_kind, item.area)?;
    let margin_box_offset = baseline_offset(
        group_kind,
        shared,
        item.logical_baseline_geometry(rows, row_gap, container_flow_axes),
        container_flow_axes.block_axis(),
    )?;
    Some(margin_box_offset + container_flow_axes.logical_edges(item.margin).block_start)
}

#[cfg(test)]
pub(in crate::grid) fn spanned_track_size<S: LayoutScalar>(
    tracks: &[S],
    start: usize,
    end: usize,
    gap: S,
) -> S {
    let track_sum = tracks[start..end]
        .iter()
        .copied()
        .fold(S::ZERO, |sum, track| sum + track);
    let gap_sum = gap * S::from_usize(end.saturating_sub(start + 1));
    track_sum + gap_sum
}

pub(super) struct OwnerProgressionTrackFrameInput<'a, S: LayoutScalar = Scalar> {
    pub(super) parent_tracks: &'a [S],
    pub(super) parent_boundary_gutters: &'a [S],
    pub(super) parent_line_offsets: Option<&'a [S]>,
    pub(super) parent_span: GridTrackSpan,
    pub(super) current_tracks: &'a [S],
    pub(super) current_boundary_gutters: &'a [S],
    pub(super) reversed: bool,
    pub(super) start_mbp: S,
}

pub(super) fn owner_progression_track_frame_origins<S: LayoutScalar>(
    input: OwnerProgressionTrackFrameInput<'_, S>,
) -> Option<OwnerProgressionTrackFrameOrigins<S>> {
    let OwnerProgressionTrackFrameInput {
        parent_tracks,
        parent_boundary_gutters,
        parent_line_offsets,
        parent_span,
        current_tracks,
        current_boundary_gutters,
        reversed,
        start_mbp,
    } = input;
    let track_positions = |tracks: &[S], boundary_gutters: &[S]| {
        if boundary_gutters.len() != tracks.len().saturating_sub(1) {
            return None;
        }
        let mut starts = Vec::with_capacity(tracks.len());
        let mut ends = Vec::with_capacity(tracks.len());
        let mut cursor = S::ZERO;
        for (index, track) in tracks.iter().copied().enumerate() {
            starts.push(cursor);
            cursor = cursor + track;
            ends.push(cursor);
            if let Some(gutter) = boundary_gutters.get(index) {
                cursor = cursor + *gutter;
            }
        }
        Some((starts, ends))
    };
    let (parent_starts, parent_ends) = if let Some(line_offsets) = parent_line_offsets {
        if line_offsets.len() != parent_tracks.len() + 1 {
            return None;
        }
        let starts = line_offsets[..parent_tracks.len()].to_vec();
        let ends = starts
            .iter()
            .copied()
            .zip(parent_tracks.iter().copied())
            .map(|(start, track)| start + track)
            .collect();
        (starts, ends)
    } else {
        track_positions(parent_tracks, parent_boundary_gutters)?
    };
    let track_count = parent_span.checked_len()?;
    if parent_span.end > parent_tracks.len() || current_tracks.len() != track_count {
        return None;
    }
    let span_start = *parent_starts.get(parent_span.start)?;
    let span_end = *parent_ends.get(parent_span.end.checked_sub(1)?)?;
    let (local_starts, local_ends) = track_positions(current_tracks, current_boundary_gutters)?;

    let (parent_first, parent_last) = if reversed {
        (
            parent_ends.clone(),
            parent_starts
                .iter()
                .copied()
                .map(|origin| -origin)
                .collect(),
        )
    } else {
        (
            parent_starts
                .iter()
                .copied()
                .map(|origin| -origin)
                .collect(),
            parent_ends.clone(),
        )
    };
    let (current_first, current_last) = if reversed {
        let content_start = span_end - start_mbp;
        (
            local_starts
                .iter()
                .copied()
                .map(|origin| content_start - origin)
                .collect(),
            local_ends
                .iter()
                .copied()
                .map(|origin| -(content_start - origin))
                .collect(),
        )
    } else {
        (
            local_starts
                .iter()
                .copied()
                .map(|origin| -(span_start + start_mbp + origin))
                .collect(),
            local_ends
                .iter()
                .copied()
                .map(|origin| span_start + start_mbp + origin)
                .collect(),
        )
    };
    Some((parent_first, parent_last, current_first, current_last))
}

pub(super) struct BaselineAlignedAxisInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) item: &'a PendingGridItem<'input, Node, S>,
    pub(super) child_style: &'a GridItemProjection<'input, S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) group: &'a AncestorBaselineGroup<Node, S>,
    pub(super) axis: GridAxisKind,
    pub(super) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) subgrid_item: Option<SubgridItemReport<Node>>,
    pub(super) container_flow_axes: FlowAxes,
    pub(super) inherited_owner_targets: Option<&'a InheritedGridOwnerBaselineTargets<Node, S>>,
    pub(super) current_grid: Node,
}

pub(super) fn baseline_aligned_axis_offset<'input, Node: Copy + PartialEq, S: LayoutScalar>(
    input: BaselineAlignedAxisInput<'input, '_, Node, S>,
) -> Result<Option<S>, InheritedCurrentGridBaselinePlacementError> {
    let BaselineAlignedAxisInput {
        item,
        child_style,
        container_style,
        group,
        axis,
        geometry,
        row_tracks,
        track_basis,
        subgrid_item,
        container_flow_axes,
        inherited_owner_targets,
        current_grid,
    } = input;
    if subgrid_item.is_some_and(|subgrid_item| {
        [subgrid_item.column, subgrid_item.row]
            .into_iter()
            .any(|report| report.can_inherit() && report.mapping.parent_axis == axis)
    }) {
        return Ok(None);
    }
    let alignment = match axis {
        GridAxisKind::Column => child_style.justify_self.or(container_style.justify_items),
        GridAxisKind::Row => child_style.align_self.or(container_style.align_items),
    }
    .unwrap_or(AlignItems::Stretch);
    let block_auto_margins = matches!(
        item.child_flow_axes.line_over_edge(child_style.margin),
        LengthAutoOf::Auto
    ) || matches!(
        item.child_flow_axes.line_under_edge(child_style.margin),
        LengthAutoOf::Auto
    );
    let (start, end) = match axis {
        GridAxisKind::Column => (item.area.column, item.area.column_end),
        GridAxisKind::Row => (item.area.row, item.area.row_end),
    };
    let synthesized_baseline_cycle = axis == GridAxisKind::Row
        && synthesized_baseline_would_cycle(
            alignment,
            item.output.baselines(),
            item.child_flow_axes,
            row_tracks.get(start..end).unwrap_or(&[]),
            track_basis.block,
        );
    let Some(member) = ancestor_baseline_member(AncestorBaselineMemberInput {
        source: item.node,
        axis,
        ancestor_span: GridTrackSpan::new(start + 1, end + 1),
        alignment_context_reversed: false,
        alignment,
        block_auto_margins,
        synthesized_baseline_cycle,
        output: item.output,
        margin: item.margin,
        child_flow_axes: item.child_flow_axes,
        containing_flow_axes: container_flow_axes,
        start_adjustment: S::ZERO,
        end_adjustment: S::ZERO,
    }) else {
        return Ok(None);
    };
    let selected_track = member.selected_track();
    let area_start = geometry
        .line_offsets()
        .get(start)
        .copied()
        .ok_or(InheritedCurrentGridBaselinePlacementError::SpanOutOfRange)?;
    let track_start = geometry
        .line_offsets()
        .get(selected_track)
        .copied()
        .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
    let track_size = geometry
        .sizes()
        .get(selected_track)
        .copied()
        .ok_or(InheritedCurrentGridBaselinePlacementError::SelectedTrackOutOfRange)?;
    // The selected alignment-context track is independent of the cohort edge.
    // Both coordinates come from the same actual geometry used for placement.
    let selected_start = track_start - area_start;
    let selected_end = track_start + track_size - area_start;
    let offset = if let Some(owner_targets) = inherited_owner_targets {
        let placement = InheritedCurrentGridBaselinePlacement::try_derive(
            &owner_targets.group,
            InheritedCurrentGridBaselinePlacementInput {
                axis,
                physical_axis: grid_axis_physical_axis(container_flow_axes, axis),
                mapping: owner_targets.mapping.clone(),
                direct_witness: CurrentGridDirectWitness::new(
                    current_grid,
                    item.node,
                    axis,
                    GridTrackSpan::new(start, end),
                    member.selected_port(),
                    member.subject_progression(),
                    member.containing_progression(),
                ),
                current_grid,
                item: item.node,
            },
        )?;
        group.placement_offset_for_target(
            member,
            placement.translated_target(),
            selected_start,
            selected_end,
        )
    } else {
        let Some(offset) = group.placement_offset(member, selected_start, selected_end) else {
            return Ok(None);
        };
        offset
    };
    if !offset.is_finite() {
        return Err(InheritedCurrentGridBaselinePlacementError::NonFinite);
    }
    Ok(Some(offset))
}

impl<'input, Node, S: LayoutScalar> PendingGridItem<'input, Node, S> {
    #[cfg(test)]
    fn logical_baseline_geometry(
        &self,
        rows: &[S],
        row_gap: S,
        container_flow_axes: FlowAxes,
    ) -> BaselineGeometry<S> {
        self.logical_baseline_geometry_for_span(
            spanned_track_size(rows, self.area.row, self.area.row_end, row_gap),
            container_flow_axes,
        )
    }

    fn logical_baseline_geometry_for_span(
        &self,
        available_span_size: S,
        container_flow_axes: FlowAxes,
    ) -> BaselineGeometry<S> {
        let logical_margin = container_flow_axes.logical_edges(self.margin);
        let logical_size = container_flow_axes.logical_size(self.output.size);
        let first_baseline = logical_block_coordinate(
            container_flow_axes,
            self.first_baseline.coordinate(),
            self.output.size,
        );
        let last_baseline = logical_block_coordinate(
            container_flow_axes,
            self.last_baseline.coordinate(),
            self.output.size,
        );
        BaselineGeometry {
            available_span_size,
            margin_box_size: logical_size.block
                + logical_margin.block_start
                + logical_margin.block_end,
            major_baseline: PhysicalBaseline::new(
                container_flow_axes.block_axis(),
                logical_margin.block_start + first_baseline,
            ),
            minor_baseline: PhysicalBaseline::new(
                container_flow_axes.block_axis(),
                logical_margin.block_end + logical_size.block - last_baseline,
            ),
        }
    }
}

fn logical_block_coordinate<S: LayoutScalar>(
    flow_axes: FlowAxes,
    coordinate: S,
    physical_size: Size<S>,
) -> S {
    if flow_axes
        .logical_axis_progression(LogicalAxis::Block)
        .is_decreasing()
    {
        flow_axes.block_axis_extent(physical_size) - coordinate
    } else {
        coordinate
    }
}

pub(super) fn final_ancestor_baseline_groups<Node: Copy + PartialEq, S: LayoutScalar>(
    inventory: &ExecutedBaselineInventory<Node, S>,
) -> FinalAncestorBaselineGroups<Node, S> {
    let make = |axis| {
        let physical = grid_axis_physical_axis(inventory.axes, axis);
        let group = AncestorBaselineGroup::reduce(
            inventory.owner,
            axis,
            physical,
            inventory.count(axis).unwrap_or(0),
            inventory
                .constraints
                .iter()
                .filter(|constraint| constraint.projection.axis() == axis)
                .map(|constraint| constraint.projection),
        );
        let placement = group
            .has_any_target()
            .then(|| InheritedGridOwnerBaselineTargets {
                mapping: CheckedOwnerToCurrentPlacementMap::identity(
                    inventory.owner,
                    axis,
                    physical,
                    inventory.axes.physical_axis_progression(physical),
                    group.track_count(),
                ),
                group: group.clone(),
            });
        (group, placement)
    };
    let (columns, placement_columns) = make(GridAxisKind::Column);
    let (rows, placement_rows) = make(GridAxisKind::Row);
    FinalAncestorBaselineGroups {
        rows,
        columns,
        placement_rows,
        placement_columns,
    }
}

pub(in crate::grid) fn baseline_groups<'input, Node, S: LayoutScalar>(
    items: &[PendingGridItem<'input, Node, S>],
    row_count: usize,
    column_count: usize,
    container_flow_axes: FlowAxes,
) -> GridBaselineGroups<S> {
    let expected_axis = container_flow_axes.block_axis();
    let mut groups = GridBaselineGroups {
        rows: vec![TrackBaselineGroup::default(); row_count],
        columns: vec![TrackBaselineGroup::default(); column_count],
    };
    for item in items {
        if !item.baseline_participation.participates || item.block_auto_margins {
            continue;
        }
        match item.baseline_participation.group {
            Some(BaselineGroupKind::Major) => {
                let Some(group) = groups
                    .rows
                    .get_mut(item.area.row)
                    .map(|group| &mut group.first)
                else {
                    continue;
                };
                merge_expected_baseline(
                    group,
                    item.logical_baseline_geometry_for_span(
                        item.area.size.block,
                        container_flow_axes,
                    )
                    .major_baseline,
                    expected_axis,
                );
            }
            Some(BaselineGroupKind::Minor) => {
                let Some(row) = item.area.row_end.checked_sub(1) else {
                    continue;
                };
                let Some(group) = groups.rows.get_mut(row).map(|group| &mut group.last) else {
                    continue;
                };
                merge_expected_baseline(
                    group,
                    item.logical_baseline_geometry_for_span(
                        item.area.size.block,
                        container_flow_axes,
                    )
                    .minor_baseline,
                    expected_axis,
                );
            }
            None => {}
        }
    }
    groups
}

pub(in crate::grid) fn merge_expected_baseline<S: LayoutScalar>(
    target: &mut Option<PhysicalBaseline<S>>,
    candidate: PhysicalBaseline<S>,
    expected_axis: PhysicalAxis,
) -> bool {
    if candidate.axis() == expected_axis {
        merge_baseline(target, candidate);
        true
    } else {
        false
    }
}

fn merge_baseline<S: LayoutScalar>(
    target: &mut Option<PhysicalBaseline<S>>,
    candidate: PhysicalBaseline<S>,
) {
    match target {
        Some(current) if current.axis() == candidate.axis() => {
            if candidate.coordinate() > current.coordinate() {
                *current = candidate;
            }
        }
        Some(_) => {}
        None => *target = Some(candidate),
    }
}

pub(in crate::grid) fn grid_container_baselines<'input, Node, S: LayoutScalar>(
    items: &[PendingGridItem<'input, Node, S>],
    groups: &GridBaselineGroups<S>,
    row_offsets: &[S],
    rows: &[S],
    flow_axes: FlowAxes,
) -> GridContainerBaselines<S> {
    let mut first_occupied_row = None;
    let mut last_occupied_row = None;
    for (row, group) in groups.rows.iter().enumerate() {
        if group.first.is_some() || group.last.is_some() {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }
    for item in items {
        include_occupied_row(
            &mut first_occupied_row,
            &mut last_occupied_row,
            item.area.row,
        );
        if let Some(row) = item.area.row_end.checked_sub(1) {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }

    let mut baselines = BaselinesOf::NONE;
    if let Some(point) = first_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row == row)
            .min_by_key(|item| grid_area_start_key(item.area))
            .map(|item| item.first_baseline.translated(item.location))
    }) {
        baselines.record_first(point);
    }
    if let Some(point) = last_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row_end.checked_sub(1) == Some(row))
            .max_by_key(|item| grid_area_end_key(item.area))
            .map(|item| {
                item.output
                    .baselines()
                    .last_or_synthesize_block_baseline(item.child_flow_axes, item.output.size)
                    .translated(item.location)
            })
    }) {
        baselines.record_last(point);
    }

    if let Some(first) = first_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .first
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| row_offsets[row] + baseline)
        })
    }) {
        baselines.replace_first_axis(
            BaselinesOf::from_block_coordinates(flow_axes, Some(first), None).first,
        );
    }
    if let Some(last) = last_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .last
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| row_offsets[row] + rows[row] - baseline)
        })
    }) {
        baselines.replace_last_axis(
            BaselinesOf::from_block_coordinates(flow_axes, None, Some(last)).last,
        );
    }

    GridContainerBaselines { baselines }
}

pub(in crate::grid) fn logical_grid_container_baselines<'input, Node, S: LayoutScalar>(
    items: &[PendingGridItem<'input, Node, S>],
    groups: &GridBaselineGroups<S>,
    row_offsets: &[S],
    rows: &[S],
    flow_axes: FlowAxes,
    containing_size: Size<S>,
) -> GridContainerBaselines<S> {
    let mut first_occupied_row = None;
    let mut last_occupied_row = None;
    for (row, group) in groups.rows.iter().enumerate() {
        if group.first.is_some() || group.last.is_some() {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }
    for item in items {
        include_occupied_row(
            &mut first_occupied_row,
            &mut last_occupied_row,
            item.area.row,
        );
        if let Some(row) = item.area.row_end.checked_sub(1) {
            include_occupied_row(&mut first_occupied_row, &mut last_occupied_row, row);
        }
    }

    let mut baselines = BaselinesOf::NONE;
    if let Some(point) = first_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row == row)
            .min_by_key(|item| grid_area_start_key(item.area))
            .map(|item| item.first_baseline.translated(item.location))
    }) {
        baselines.record_first(point);
    }
    if let Some(point) = last_occupied_row.and_then(|row| {
        items
            .iter()
            .filter(|item| item.area.row_end.checked_sub(1) == Some(row))
            .max_by_key(|item| grid_area_end_key(item.area))
            .map(|item| {
                item.output
                    .baselines()
                    .last_or_synthesize_block_baseline(item.child_flow_axes, item.output.size)
                    .translated(item.location)
            })
    }) {
        baselines.record_last(point);
    }

    if let Some(first) = first_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .first
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| {
                    flow_axes.block_axis_coordinate(flow_axes.physical_point(
                        LogicalPointOf::new(S::ZERO, row_offsets[row] + baseline),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                        containing_size,
                    ))
                })
        })
    }) {
        baselines.replace_first_axis(
            BaselinesOf::from_block_coordinates(flow_axes, Some(first), None).first,
        );
    }
    if let Some(last) = last_occupied_row.and_then(|row| {
        groups.rows.get(row).and_then(|group| {
            group
                .last
                .and_then(|baseline| baseline.coordinate_on(flow_axes.block_axis()))
                .map(|baseline| {
                    flow_axes.block_axis_coordinate(flow_axes.physical_point(
                        LogicalPointOf::new(S::ZERO, row_offsets[row] + rows[row] - baseline),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                        containing_size,
                    ))
                })
        })
    }) {
        baselines.replace_last_axis(
            BaselinesOf::from_block_coordinates(flow_axes, None, Some(last)).last,
        );
    }

    GridContainerBaselines { baselines }
}

fn include_occupied_row(first: &mut Option<usize>, last: &mut Option<usize>, row: usize) {
    *first = Some(first.map_or(row, |current| current.min(row)));
    *last = Some(last.map_or(row, |current| current.max(row)));
}

fn grid_area_start_key<S: LayoutScalar>(area: GridArea<S>) -> (usize, usize) {
    (area.row, area.column)
}

fn grid_area_end_key<S: LayoutScalar>(area: GridArea<S>) -> (usize, usize) {
    (
        area.row_end.saturating_sub(1),
        area.column_end.saturating_sub(1),
    )
}

pub(in crate::grid) fn baseline_participation<S: LayoutScalar>(
    align_self: AlignItems,
    block_auto_margins: bool,
    synthesized_baseline_would_cycle: bool,
    baselines: BaselinesOf<S>,
    flow_axes: FlowAxes,
) -> BaselineParticipation {
    let (mut group, synthesized, fallback_alignment) = match align_self {
        AlignItems::Baseline => (
            Some(BaselineGroupKind::Major),
            baselines.first_block(flow_axes).is_none(),
            Some(AlignItems::Start),
        ),
        AlignItems::LastBaseline => (
            Some(BaselineGroupKind::Minor),
            baselines.last_block(flow_axes).is_none(),
            Some(AlignItems::End),
        ),
        _ => (None, false, None),
    };
    if synthesized && synthesized_baseline_would_cycle {
        group = None;
    }

    BaselineParticipation {
        participates: group.is_some() && !block_auto_margins,
        group,
        synthesized,
        fallback_alignment,
    }
}

pub(in crate::grid) fn baseline_participation_for_container<S: LayoutScalar>(
    align_self: AlignItems,
    block_auto_margins: bool,
    synthesized_baseline_would_cycle: bool,
    baselines: BaselinesOf<S>,
    child_flow_axes: FlowAxes,
    container_flow_axes: FlowAxes,
) -> BaselineParticipation {
    let mut participation = baseline_participation(
        align_self,
        block_auto_margins,
        synthesized_baseline_would_cycle,
        baselines,
        child_flow_axes,
    );
    if child_flow_axes.block_axis() != container_flow_axes.block_axis() {
        participation.participates = false;
        participation.group = None;
    }
    participation
}

pub(in crate::grid) fn synthesized_baseline_would_cycle<S: LayoutScalar>(
    align_self: AlignItems,
    baselines: BaselinesOf<S>,
    flow_axes: FlowAxes,
    row_span_tracks: &[TrackSizingOf<S>],
    track_basis: Option<S>,
) -> bool {
    let synthesizes = match align_self {
        AlignItems::Baseline => baselines.first_block(flow_axes).is_none(),
        AlignItems::LastBaseline => baselines.last_block(flow_axes).is_none(),
        _ => false,
    };
    synthesizes
        && row_span_tracks.len() > 1
        && row_span_tracks
            .iter()
            .any(|track| track_accepts_intrinsic_contribution(track, track_basis))
}
