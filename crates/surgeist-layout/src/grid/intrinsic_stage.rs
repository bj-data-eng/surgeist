use super::lanes::{
    LaneIntrinsicChild, LaneIntrinsicContributions, LaneIntrinsicRole, LanePurposeContributions,
    combine_lane_purposes,
};
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
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisTrackPreparation<S: LayoutScalar> {
    // This effective request produced the reduction. Original caller operation and
    // initial fixed authority separately govern normal recomputation.
    pub(super) available: AxisSizingRequestOf<S>,
    // A maximum can constrain finishing without replacing that operation.
    pub(super) finishing_available: GridTrackSizingConstraint<S>,
}

pub(super) struct GridAxisTrackReduction<S: LayoutScalar> {
    pub(super) sizes: Vec<S>,
    pub(super) available: AxisSizingRequestOf<S>,
    pub(super) finishing_available: GridTrackSizingConstraint<S>,
}

// One returned bounded sizing execution owns this key. Unissued prefix axes remain
// absent even if the traversal returned provisional numeric estimates.
pub(super) struct GridTrackSizingReceipt<Node, S: LayoutScalar> {
    owner: Node,
    context: GridContainerContext<S>,
    columns: Option<Vec<S>>,
    rows: Option<Vec<S>>,
    requests: LogicalSizeOf<Option<AxisSizingRequestOf<S>>>,
    finishing: LogicalSizeOf<Option<GridTrackSizingConstraint<S>>>,
}

pub(super) struct GridTrackExecutionFrame<'input, 'source, Node, S: LayoutScalar> {
    receipt: &'source GridTrackSizingReceipt<Node, S>,
    constants: &'source Constants<S>,
    flow: crate::geometry::FlowAxes,
    stage: GridSizingStage,
    style: GridContainerProjection<'source, S>,
    column_tracks: &'source [TrackSizingOf<S>],
    row_tracks: &'source [TrackSizingOf<S>],
    minimum_functions: GridMinimumParentFunctions<'source, S>,
    placements: &'source GridPlacementContext<'input, Node, S>,
    subgrid_report: &'source GridSubgridReport<Node>,
}

impl<Node, S: LayoutScalar> GridTrackSizingReceipt<Node, S> {
    pub(super) fn frame<'input, 'source>(
        &'source self,
        source: GridTrackSource<'input, 'source, Node, S>,
    ) -> GridTrackExecutionFrame<'input, 'source, Node, S> {
        GridTrackExecutionFrame {
            receipt: self,
            constants: source.constants,
            flow: source.sizing_flow_axes,
            stage: source.stage,
            style: *source.style,
            column_tracks: source.column_tracks,
            row_tracks: source.row_tracks,
            minimum_functions: source.minimum_functions,
            placements: source.placements,
            subgrid_report: source.subgrid_report,
        }
    }
}

pub(super) enum GridTrackRun<'input, 'source, 'a, Node, S: LayoutScalar> {
    Intrinsic,
    AcceptedReceiver {
        source: &'a GridTrackExecutionFrame<'input, 'source, Node, S>,
        columns: Vec<S>,
        rows: Vec<S>,
        column_preparation: GridAxisTrackPreparation<S>,
        row_preparation: GridAxisTrackPreparation<S>,
        spaces: LogicalSizeOf<GridAxisTrackSpace<S>>,
    },
}

fn retain_allocation<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    values: &[S],
) -> LayoutResultOf<Node, Vec<S>, S, M> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    result.extend_from_slice(values);
    Ok(result)
}

struct MinContentObservation<Node, S: LayoutScalar> {
    grid: IntrinsicAxisContributions<Node, S>,
    lanes: LanePurposeContributions<Node, S>,
}

impl<Node: PartialEq, S: LayoutScalar> MinContentObservation<Node, S> {
    fn matches(&self, previous: &AcceptedMinContent<Node, S>) -> bool {
        self.grid.receipts == previous.receipts
            && self.grid.base_floors == previous.base_floors
            && self.lanes == previous.lanes
    }
}

enum StageMinContent<Node, S: LayoutScalar> {
    Discard,
    Retain,
    Observed(MinContentObservation<Node, S>),
}

struct AcceptedMinContent<Node, S: LayoutScalar> {
    receipts: Vec<GridContributionReceipt<Node, S>>,
    base_floors: Vec<S>,
    lanes: LanePurposeContributions<Node, S>,
    own_axis: Option<Vec<S>>,
    observe_auto: bool,
}

impl<Node, S: LayoutScalar> AcceptedMinContent<Node, S> {
    fn in_frame<'a>(&'a self, mut frame: IntrinsicStageFrame<'a, S>) -> IntrinsicStageFrame<'a, S> {
        let own_axis = self.own_axis.as_deref();
        match frame.axis {
            GridAxisKind::Column => frame.columns = own_axis,
            GridAxisKind::Row => frame.rows = own_axis,
        }
        frame.observed_axis = if self.observe_auto { own_axis } else { None };
        frame
    }
}

struct EvaluatedAxis<Node, S: LayoutScalar> {
    contributions: GridAxisTrackInputs<S>,
    min_content: MinContentObservation<Node, S>,
    observed_axis: Option<Vec<S>>,
}

type StageResult<Node, S, M> = LayoutResultOf<Node, EvaluatedAxis<Node, S>, S, M>;
type PreparedAxisResult<Node, S, M> =
    LayoutResultOf<Node, (Vec<S>, GridAxisTrackPreparation<S>), S, M>;

fn intrinsic_grid<'input, 'a, Node: Copy, S: LayoutScalar>(
    input: &'a GridTrackResolutionInput<'input, '_, Node, S>,
) -> IntrinsicGrid<'input, 'a, Node, S> {
    IntrinsicGrid {
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
    }
}

pub(super) struct IntrinsicRepeatSizing<Node, S: LayoutScalar> {
    collection: ContributionCollection<Node, S>,
    frame: RepeatSizingFrame<S>,
}

enum RepeatSizingFrame<S> {
    Initial,
    Tentative(Vec<S>),
}

type IntrinsicRepeatStep<Node, S, M> =
    LayoutResultOf<Node, ControlFlow<GridAxisTrackPhases<S>, (Node, ComputeInputOf<S>)>, S, M>;

impl<Node: Copy + Eq, S: LayoutScalar> IntrinsicRepeatSizing<Node, S> {
    pub(super) fn new() -> Self {
        Self {
            collection: ContributionCollection::new(StageMinContent::Discard),
            frame: RepeatSizingFrame::Initial,
        }
    }

    pub(super) fn advance<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        input: &GridTrackResolutionInput<'input, '_, Node, S>,
    ) -> IntrinsicRepeatStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let axis = input.stage.first_axis;
        loop {
            let source = self.frame.source(input);
            let evaluated = match self.collection.advance(tree, node, source)? {
                ControlFlow::Continue(request) => return Ok(ControlFlow::Continue(request)),
                ControlFlow::Break(CollectedContributions::Axis(evaluated)) => evaluated,
                ControlFlow::Break(CollectedContributions::Observation(_)) => {
                    unreachable!("repeat sizing collects all contribution roles")
                }
            };
            if matches!(self.frame, RepeatSizingFrame::Initial)
                && has_inherited_items(source.grid, axis)
            {
                let (tentative, _) = prepare_axis_track_inputs(
                    node,
                    input,
                    axis,
                    &evaluated.contributions,
                    axis_available(input.available, axis),
                    GridAxisTrackSpace::Intrinsic,
                )?;
                self.frame = RepeatSizingFrame::Tentative(tentative);
                self.collection = ContributionCollection::new(StageMinContent::Discard);
                continue;
            }
            return intrinsic_axis_phases(
                node,
                input,
                axis,
                &evaluated.contributions,
                phase_constraint(axis_track_available(node, input, axis)?),
            )
            .map(ControlFlow::Break);
        }
    }

    pub(super) fn accept<M>(
        &mut self,
        node: Node,
        input: &GridTrackResolutionInput<'_, '_, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        self.collection
            .accept(node, self.frame.source(input), output)
    }
}

impl<S: LayoutScalar> RepeatSizingFrame<S> {
    fn source<'input, 'a, Node: Copy>(
        &'a self,
        input: &'a GridTrackResolutionInput<'input, 'a, Node, S>,
    ) -> ContributionSource<'input, 'a, Node, S> {
        let observed_axis = match self {
            Self::Initial => None,
            Self::Tentative(values) => Some(values.as_slice()),
        };
        ContributionSource {
            input,
            grid: intrinsic_grid(input),
            frame: IntrinsicStageFrame {
                axis: input.stage.first_axis,
                stage: input.stage,
                columns: None,
                rows: None,
                observed_axis,
            },
        }
    }
}

// A single source-ordered preparation serves full layout and bounded probes.
pub(super) fn resolve_tracks<'input, 'source, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: GridTrackResolutionInput<'input, 'source, Tree::Node, Tree::Scalar>,
    run: GridTrackRun<'input, 'source, '_, Tree::Node, Tree::Scalar>,
) -> GridTrackResolutionResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut sizing = GridTrackSizing::begin(node, &input, run)?;
    loop {
        match sizing.advance(tree, node, &input)? {
            ControlFlow::Break(()) => return sizing.finish(node, input),
            ControlFlow::Continue((child, request)) => {
                let result = tree.compute_child(child, request)?;
                sizing.accept(node, &input, result)?;
            }
        }
    }
}

pub(super) type TrackSizingStep<Node, S, M> =
    LayoutResultOf<Node, ControlFlow<(), (Node, ComputeInputOf<S>)>, S, M>;

pub(super) struct GridTrackSizing<Node, S: LayoutScalar> {
    columns: Option<Vec<S>>,
    rows: Option<Vec<S>>,
    column_preparation: Option<GridAxisTrackPreparation<S>>,
    row_preparation: Option<GridAxisTrackPreparation<S>>,
    column_min_content: Option<AcceptedMinContent<Node, S>>,
    row_min_content: Option<AcceptedMinContent<Node, S>>,
    spaces: LogicalSizeOf<GridAxisTrackSpace<S>>,
    receiver: bool,
    phase: usize,
    active: Option<Box<TrackAxisWork<Node, S>>>,
}

struct TrackAxisWork<Node, S: LayoutScalar> {
    collection: ContributionCollection<Node, S>,
    mode: TrackAxisMode<S>,
}

enum TrackAxisMode<S> {
    Observe,
    Tentative {
        use_previous: bool,
    },
    Final {
        use_previous: bool,
        tentative: Option<Vec<S>>,
    },
}

const TRACK_PHASES: [GridSizingPhase; 4] = [
    GridSizingPhase::InitialFirst,
    GridSizingPhase::InitialSecond,
    GridSizingPhase::RevisitFirst,
    GridSizingPhase::RevisitSecond,
];

impl<Node: Copy + Eq, S: LayoutScalar> GridTrackSizing<Node, S> {
    pub(super) fn intrinsic() -> Self {
        Self {
            columns: None,
            rows: None,
            column_preparation: None,
            row_preparation: None,
            column_min_content: None,
            row_min_content: None,
            spaces: LogicalSizeOf::new(
                GridAxisTrackSpace::Intrinsic,
                GridAxisTrackSpace::Intrinsic,
            ),
            receiver: false,
            phase: 0,
            active: None,
        }
    }

    pub(super) fn begin<'input, 'source, M>(
        node: Node,
        input: &GridTrackResolutionInput<'input, 'source, Node, S>,
        run: GridTrackRun<'input, 'source, '_, Node, S>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let (columns, rows, column_preparation, row_preparation, spaces, receiver, unchanged) =
            match run {
                GridTrackRun::Intrinsic => return Ok(Self::intrinsic()),
                GridTrackRun::AcceptedReceiver {
                    source,
                    columns,
                    rows,
                    column_preparation,
                    row_preparation,
                    spaces,
                } => {
                    if source.receipt.owner != node
                        || source.stage != input.stage
                        || source.receipt.columns.as_deref() != Some(columns.as_slice())
                        || source.receipt.rows.as_deref() != Some(rows.as_slice())
                        || source.receipt.requests.inline != Some(column_preparation.available)
                        || source.receipt.requests.block != Some(row_preparation.available)
                        || source.receipt.finishing.inline
                            != Some(column_preparation.finishing_available)
                        || source.receipt.finishing.block
                            != Some(row_preparation.finishing_available)
                    {
                        return Err(grid_track_sizing_error(
                            node,
                            SpanningContributionError::InvalidIncidence,
                        ));
                    }
                    for (space, request) in [
                        (spaces.inline, input.available.inline),
                        (spaces.block, input.available.block),
                    ] {
                        let valid = match space {
                            GridAxisTrackSpace::Intrinsic => {
                                request.operation() != SizingOperation::Normal
                            }
                            GridAxisTrackSpace::FinalUsed(value) => {
                                request.operation() == SizingOperation::Normal
                                    && request
                                        .space()
                                        .definite_size()
                                        .is_some_and(|extent| extent.get() == value)
                            }
                        };
                        if !valid {
                            return Err(grid_track_sizing_error(
                                node,
                                SpanningContributionError::InvalidIncidence,
                            ));
                        }
                    }
                    let unchanged = source.constants == input.constants
                        && source.flow == input.sizing_flow_axes
                        && &source.receipt.context == input.context.as_ref()
                        && source.style == *input.style
                        && source.column_tracks == input.column_tracks
                        && source.row_tracks == input.row_tracks
                        && source.minimum_functions == input.minimum_functions
                        && core::ptr::eq(source.placements, input.placements)
                        && core::ptr::eq(source.subgrid_report, input.subgrid_report)
                        && source.receipt.requests.inline == Some(input.available.inline)
                        && source.receipt.requests.block == Some(input.available.block);
                    (
                        Some(columns),
                        Some(rows),
                        Some(column_preparation),
                        Some(row_preparation),
                        spaces,
                        true,
                        unchanged,
                    )
                }
            };
        Ok(Self {
            columns,
            rows,
            column_preparation,
            row_preparation,
            spaces,
            receiver,
            column_min_content: None,
            row_min_content: None,
            phase: if unchanged { TRACK_PHASES.len() } else { 0 },
            active: None,
        })
    }

    pub(super) fn advance<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        input: &GridTrackResolutionInput<'input, '_, Node, S>,
    ) -> TrackSizingStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        while let Some(&phase) = TRACK_PHASES
            .get(self.phase)
            .filter(|&&phase| phase <= input.stage.through)
        {
            let axis = self.axis(input.stage);
            let retain = self.retains_min_content(input.stage, phase);
            if self.active.is_none() {
                self.active = Some(Box::new(if phase >= GridSizingPhase::RevisitFirst {
                    TrackAxisWork {
                        mode: TrackAxisMode::Observe,
                        collection: ContributionCollection::empty(true),
                    }
                } else {
                    self.collect_axis(input, axis, false, None, retain)
                }));
            }
            let mut active = self
                .active
                .take()
                .expect("current axis owns its active work");
            let source = self.source(node, input, &active.mode)?;
            let result = match active.collection.advance(tree, node, source)? {
                ControlFlow::Continue(request) => {
                    self.active = Some(active);
                    return Ok(ControlFlow::Continue(request));
                }
                ControlFlow::Break(result) => result,
            };
            match (active.mode, result) {
                (TrackAxisMode::Observe, CollectedContributions::Observation(observation)) => {
                    let previous = self.previous(axis);
                    if observation.matches(previous) {
                        self.phase += 1;
                    } else {
                        let observation = (!previous.observe_auto).then_some(observation);
                        let use_previous = observation.is_some();
                        self.active = Some(Box::new(self.collect_axis(
                            input,
                            axis,
                            use_previous,
                            observation,
                            retain,
                        )));
                    }
                }
                (
                    TrackAxisMode::Tentative { use_previous },
                    CollectedContributions::Axis(evaluated),
                ) => {
                    let (tentative, _) = self.prepare_axis(node, input, axis, &evaluated)?;
                    let observation = active.collection.saved_observation.take();
                    self.active = Some(Box::new(TrackAxisWork {
                        mode: TrackAxisMode::Final {
                            use_previous,
                            tentative: Some(tentative),
                        },
                        collection: ContributionCollection::new(Self::min_content(
                            observation,
                            retain,
                        )),
                    }));
                }
                (
                    TrackAxisMode::Final {
                        use_previous,
                        tentative,
                    },
                    CollectedContributions::Axis(mut evaluated),
                ) => {
                    let (sizes, preparation) = self.prepare_axis(node, input, axis, &evaluated)?;
                    evaluated.observed_axis = tentative;
                    let min_content = if retain {
                        let frame = self.frame(input.stage, axis, phase, use_previous);
                        let observe_auto = evaluated.observed_axis.is_some();
                        let own_axis = match evaluated.observed_axis {
                            Some(tentative) => Some(tentative),
                            None => {
                                axis_available(LogicalSizeOf::new(frame.columns, frame.rows), axis)
                                    .map(|values| retain_allocation(node, values))
                                    .transpose()?
                            }
                        };
                        Some(AcceptedMinContent {
                            receipts: evaluated.min_content.grid.receipts,
                            base_floors: evaluated.min_content.grid.base_floors,
                            lanes: evaluated.min_content.lanes,
                            own_axis,
                            observe_auto,
                        })
                    } else {
                        None
                    };
                    match axis {
                        GridAxisKind::Column => {
                            self.columns = Some(sizes);
                            self.column_preparation = Some(preparation);
                            self.column_min_content = min_content;
                        }
                        GridAxisKind::Row => {
                            self.rows = Some(sizes);
                            self.row_preparation = Some(preparation);
                            self.row_min_content = min_content;
                        }
                    }
                    self.phase += 1;
                }
                _ => unreachable!("track result belongs to its active axis phase"),
            }
        }
        Ok(ControlFlow::Break(()))
    }

    pub(super) fn accept<M>(
        &mut self,
        node: Node,
        input: &GridTrackResolutionInput<'_, '_, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let mut active = self.active.take().expect("track child retains its axis");
        let source = self.source(node, input, &active.mode)?;
        active.collection.accept(node, source, output)?;
        self.active = Some(active);
        Ok(())
    }

    fn collect_axis(
        &self,
        input: &GridTrackResolutionInput<'_, '_, Node, S>,
        axis: GridAxisKind,
        use_previous: bool,
        observation: Option<MinContentObservation<Node, S>>,
        retain: bool,
    ) -> TrackAxisWork<Node, S> {
        if has_inherited_items(intrinsic_grid(input), axis) {
            let mut collection = ContributionCollection::new(StageMinContent::Discard);
            collection.saved_observation = observation;
            TrackAxisWork {
                mode: TrackAxisMode::Tentative { use_previous },
                collection,
            }
        } else {
            TrackAxisWork {
                mode: TrackAxisMode::Final {
                    use_previous,
                    tentative: None,
                },
                collection: ContributionCollection::new(Self::min_content(observation, retain)),
            }
        }
    }

    fn min_content(
        observation: Option<MinContentObservation<Node, S>>,
        retain: bool,
    ) -> StageMinContent<Node, S> {
        if let Some(observation) = observation {
            StageMinContent::Observed(observation)
        } else if retain {
            StageMinContent::Retain
        } else {
            StageMinContent::Discard
        }
    }

    fn axis(&self, stage: GridSizingStage) -> GridAxisKind {
        match (stage.first_axis, self.phase % 2) {
            (GridAxisKind::Column, 0) | (GridAxisKind::Row, 1) => GridAxisKind::Column,
            _ => GridAxisKind::Row,
        }
    }

    fn retains_min_content(&self, stage: GridSizingStage, phase: GridSizingPhase) -> bool {
        match phase {
            GridSizingPhase::InitialFirst => stage.through >= GridSizingPhase::RevisitFirst,
            GridSizingPhase::InitialSecond => stage.through >= GridSizingPhase::RevisitSecond,
            _ => false,
        }
    }

    fn previous(&self, axis: GridAxisKind) -> &AcceptedMinContent<Node, S> {
        match axis {
            GridAxisKind::Column => self.column_min_content.as_ref(),
            GridAxisKind::Row => self.row_min_content.as_ref(),
        }
        .expect("revisit follows initial sizing for that axis")
    }

    fn frame(
        &self,
        stage: GridSizingStage,
        axis: GridAxisKind,
        phase: GridSizingPhase,
        use_previous: bool,
    ) -> IntrinsicStageFrame<'_, S> {
        let frame = IntrinsicStageFrame {
            axis,
            stage: GridSizingStage {
                through: if self.receiver { stage.through } else { phase },
                ..stage
            },
            columns: self.columns.as_deref(),
            rows: self.rows.as_deref(),
            observed_axis: None,
        };
        if use_previous {
            self.previous(axis).in_frame(frame)
        } else {
            frame
        }
    }

    fn source<'input, 'a, M>(
        &'a self,
        node: Node,
        input: &'a GridTrackResolutionInput<'input, '_, Node, S>,
        mode: &'a TrackAxisMode<S>,
    ) -> LayoutResultOf<Node, ContributionSource<'input, 'a, Node, S>, S, M> {
        let axis = self.axis(input.stage);
        let mut grid = intrinsic_grid(input);
        grid.query.include_flexible_tracks = matches!(
            axis_available(self.spaces, axis),
            GridAxisTrackSpace::Intrinsic
        ) && axis_track_available(node, input, axis)?
            .operation()
            == SizingOperation::MaxContent;
        let use_previous = match mode {
            TrackAxisMode::Observe => true,
            TrackAxisMode::Tentative { use_previous }
            | TrackAxisMode::Final { use_previous, .. } => *use_previous,
        };
        let mut frame = self.frame(input.stage, axis, TRACK_PHASES[self.phase], use_previous);
        if let TrackAxisMode::Final {
            tentative: Some(tentative),
            ..
        } = mode
        {
            frame.observed_axis = Some(tentative);
        }
        Ok(ContributionSource { input, grid, frame })
    }

    fn prepare_axis<M>(
        &self,
        node: Node,
        input: &GridTrackResolutionInput<'_, '_, Node, S>,
        axis: GridAxisKind,
        evaluated: &EvaluatedAxis<Node, S>,
    ) -> PreparedAxisResult<Node, S, M> {
        prepare_axis_track_inputs(
            node,
            input,
            axis,
            &evaluated.contributions,
            axis_available(input.available, axis),
            axis_available(self.spaces, axis),
        )
    }

    pub(super) fn finish<'input, 'source, M>(
        self,
        node: Node,
        input: GridTrackResolutionInput<'input, 'source, Node, S>,
    ) -> GridTrackResolutionResult<Node, S, M> {
        let Self {
            columns,
            rows,
            column_preparation,
            row_preparation,
            ..
        } = self;
        // An unissued incomplete axis remains None even when it needs an estimate.
        let columns = columns.unwrap_or_else(|| {
            initial_estimate(input.column_tracks, input.context.percent_basis.inline)
        });
        let rows = rows.unwrap_or_else(|| {
            initial_estimate(input.row_tracks, input.context.percent_basis.block)
        });
        let receipt = GridTrackSizingReceipt {
            owner: node,
            columns: if column_preparation.is_some() {
                Some(retain_allocation(node, &columns)?)
            } else {
                None
            },
            rows: if row_preparation.is_some() {
                Some(retain_allocation(node, &rows)?)
            } else {
                None
            },
            requests: LogicalSizeOf::new(
                column_preparation.as_ref().map(|value| value.available),
                row_preparation.as_ref().map(|value| value.available),
            ),
            finishing: LogicalSizeOf::new(
                column_preparation
                    .as_ref()
                    .map(|value| value.finishing_available),
                row_preparation
                    .as_ref()
                    .map(|value| value.finishing_available),
            ),
            context: input.context.into_owned(),
        };
        Ok(GridTrackResolution {
            sizing_phases: GridTrackSizingPhases { limit: input.stage },
            columns,
            rows,
            column_preparation,
            row_preparation,
            receipt,
        })
    }
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
pub(super) fn unallocated_track_operation<S: LayoutScalar>(
    axis: GridAxisKind,
    available: AxisSizingRequestOf<S>,
) -> SizingOperation {
    match available.operation() {
        SizingOperation::Normal
            if axis == GridAxisKind::Row || available.space() == AvailableSpaceOf::Indefinite =>
        {
            SizingOperation::MaxContent
        }
        operation => operation,
    }
}

fn effective_available<Node, S: LayoutScalar>(
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    available: AxisSizingRequestOf<S>,
) -> Result<AxisSizingRequestOf<S>, SpanningContributionError<S>> {
    let flow = input.sizing_flow_axes;
    let inner = axis_available(flow.logical_size(input.constants.node_inner_size), axis);
    if let Some(inner) = inner {
        let space = crate::layout_math::checked_available_space(inner)
            .map_err(|_| SpanningContributionError::InvalidNumeric { value: inner })?;
        return Ok(AxisSizingRequestOf::new(space, SizingOperation::Normal));
    }
    let operation = unallocated_track_operation(axis, available);
    if operation != SizingOperation::Normal {
        return Ok(available.with_operation(operation));
    }
    let available_inner = axis_available(
        flow.logical_size(input.constants.available_inner_size),
        axis,
    );
    let Some(remaining) = available_inner else {
        return Ok(available);
    };
    let remaining = crate::layout_math::remaining_available_extent(remaining, S::ZERO)
        .map_err(|_| SpanningContributionError::InvalidNumeric { value: remaining })?;
    let space = crate::layout_math::checked_available_space(remaining)
        .map_err(|_| SpanningContributionError::InvalidNumeric { value: remaining })?;
    Ok(available.with_space(space))
}

pub(super) fn axis_track_available<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
) -> LayoutResultOf<Node, AxisSizingRequestOf<S>, S, M> {
    effective_available(input, axis, axis_available(input.available, axis))
        .map_err(|error| grid_track_sizing_error(node, error))
}

fn phase_constraint<S: LayoutScalar>(
    available: AxisSizingRequestOf<S>,
) -> GridTrackSizingConstraint<S> {
    GridTrackSizingConstraint {
        operation: available.operation(),
        available_extent: if available.operation() == SizingOperation::Normal {
            available
                .space()
                .definite_size()
                .map(crate::NonNegativeFiniteOf::get)
        } else {
            None
        },
    }
}

fn intrinsic_axis_phases<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    contributions: &GridAxisTrackInputs<S>,
    available: GridTrackSizingConstraint<S>,
) -> LayoutResultOf<Node, GridAxisTrackPhases<S>, S, M> {
    resolve_grid_contribution_rows(
        SpanningTrackContext {
            tracks: axis_tracks(input, axis),
            basis: axis_available(input.context.percent_basis, axis),
            available,
            gap: axis_available(input.context.gap, axis),
            gutters: Some(axis_gutters(input, axis)),
            base_floors: &contributions.base_floors,
        },
        &contributions.rows,
    )
    .map_err(|error| grid_track_sizing_error(node, error))
}

// A constraint change recomputes from retained normalized inputs. Any
// changed basis/gutter/opposite frame instead needs fresh owning collection.
pub(super) fn reduce_axis_track_inputs<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    contributions: &GridAxisTrackInputs<S>,
    mut available: AxisSizingRequestOf<S>,
    space: GridAxisTrackSpace<S>,
) -> LayoutResultOf<Node, GridAxisTrackReduction<S>, S, M> {
    if let GridAxisTrackSpace::FinalUsed(value) = space {
        let extent = crate::layout_math::checked_available_space(value)
            .map_err(|status| crate::error::value_resolution_error(node, status))?;
        available = AxisSizingRequestOf::new(extent, SizingOperation::Normal);
    }
    let tracks = axis_tracks(input, axis);
    let gap = axis_available(input.context.gap, axis);
    let gutters = axis_gutters(input, axis);
    let phase_constraint = phase_constraint(available);
    let phases = intrinsic_axis_phases(node, input, axis, contributions, phase_constraint)?;
    let flow = input.sizing_flow_axes;
    let inner = axis_available(flow.logical_size(input.constants.node_inner_size), axis);
    let intrinsic_bounds = matches!(space, GridAxisTrackSpace::Intrinsic) && inner.is_none();
    let content_bound = |bound: Size<Option<S>>| {
        if !intrinsic_bounds {
            return Ok(None);
        }
        axis_available(flow.logical_size(bound), axis)
            .map(|outer| {
                crate::layout_math::remaining_available_extent(
                    outer,
                    axis_available(
                        flow.logical_size(input.constants.content_box_inset.sum_axes()),
                        axis,
                    ),
                )
            })
            .transpose()
            .map_err(|status| crate::error::value_resolution_error(node, status))
    };
    let minimum = content_bound(input.constants.node_min_size)?;
    let maximum = content_bound(input.constants.node_max_size)?;
    let mut finishing_available = GridTrackSizingConstraint {
        operation: available.operation(),
        available_extent: if available.operation() == SizingOperation::MaxContent {
            maximum
        } else {
            phase_constraint.available_extent
        },
    };
    let stretch_size = match space {
        GridAxisTrackSpace::Intrinsic if phase_constraint.available_extent.is_none() => {
            inner.or(content_bound(input.constants.node_definite_min_size)?)
        }
        GridAxisTrackSpace::Intrinsic => inner,
        GridAxisTrackSpace::FinalUsed(value) => Some(value),
    };
    let finish = |available, stretch_size| {
        resolve_tracks_from_phases(ResolvedAxisTrackInput {
            tracks,
            phases: &phases,
            items: &contributions.rows,
            available,
            flex_bounds: GridTrackFinishingBounds { minimum, maximum },
            stretch_size,
            gap,
            alignment: axis_alignment(input, axis),
            gutters: Some(gutters),
        })
        .map_err(|error| grid_track_sizing_error(node, error))
    };
    let mut sizes = finish(phase_constraint, stretch_size)?;
    if let Some(maximum) = maximum
        && finishing_available.available_extent != Some(maximum)
        && track_sum_with_gutters(&sizes, gap, Some(gutters)) > maximum
    {
        finishing_available.available_extent = Some(maximum);
        sizes = finish(finishing_available, None)?;
    }
    Ok(GridAxisTrackReduction {
        sizes,
        available,
        finishing_available,
    })
}
fn prepare_axis_track_inputs<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: &GridTrackResolutionInput<'_, '_, Node, S>,
    axis: GridAxisKind,
    contributions: &GridAxisTrackInputs<S>,
    available: AxisSizingRequestOf<S>,
    space: GridAxisTrackSpace<S>,
) -> PreparedAxisResult<Node, S, M> {
    let reduced = reduce_axis_track_inputs(
        node,
        input,
        axis,
        contributions,
        match space {
            GridAxisTrackSpace::Intrinsic => effective_available(input, axis, available)
                .map_err(|error| grid_track_sizing_error(node, error))?,
            GridAxisTrackSpace::FinalUsed(value) => AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(value).map_err(
                    |_| {
                        grid_track_sizing_error(
                            node,
                            SpanningContributionError::InvalidNumeric { value },
                        )
                    },
                )?),
                SizingOperation::Normal,
            ),
        },
        space,
    )?;
    Ok((
        reduced.sizes,
        GridAxisTrackPreparation {
            available: reduced.available,
            finishing_available: reduced.finishing_available,
        },
    ))
}

fn lane_intrinsic_input<'input, 'a, Node: Copy, S: LayoutScalar>(
    input: &'a GridTrackResolutionInput<'input, '_, Node, S>,
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    frame: IntrinsicStageFrame<'a, S>,
) -> Option<LaneIntrinsicTrackSizeInput<'input, 'a, Node, S>> {
    if !input
        .style
        .display
        .establishes_grid_lanes_formatting_context()
        || grid_axis_for_grid_lanes(input.style) != frame.axis
    {
        return None;
    }
    Some(LaneIntrinsicTrackSizeInput {
        grid,
        #[cfg(test)]
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
    })
}

enum TrackContributionRequest<Node, S: LayoutScalar> {
    Grid(IntrinsicTrackChild<Node, S>),
    Lanes(LaneIntrinsicChild<Node, S>),
}

enum ContributionRole<Node, S: LayoutScalar> {
    Grid(IntrinsicTrackRole<Node, S>),
    Lanes(LaneIntrinsicRole<Node, S>),
}

struct ActiveContributionRole<Node, S: LayoutScalar> {
    role: ContributionRole<Node, S>,
    available: Size<AxisSizingRequestOf<S>>,
}

#[derive(Clone, Copy)]
enum TrackContributionKind {
    Grid,
    Lanes,
}

const CONTRIBUTION_PURPOSES: [GridContributionPurpose; 3] = [
    GridContributionPurpose::Minimum,
    GridContributionPurpose::MinContent,
    GridContributionPurpose::MaxContent,
];

struct CombinedGridContributions<Node, S: LayoutScalar> {
    group: GridAxisContributions<Node, S>,
    rows: Vec<GridIntrinsicContributionRow<S>>,
}

struct ContributionCollection<Node, S: LayoutScalar> {
    index: usize,
    observe_only: bool,
    retain_min_content: bool,
    grid: [Option<IntrinsicAxisContributions<Node, S>>; 3],
    lanes: [Option<LanePurposeContributions<Node, S>>; 3],
    combined: Option<CombinedGridContributions<Node, S>>,
    role: Option<ActiveContributionRole<Node, S>>,
    pending: Option<TrackContributionRequest<Node, S>>,
    saved_observation: Option<MinContentObservation<Node, S>>,
}

enum CollectedContributions<Node, S: LayoutScalar> {
    Observation(MinContentObservation<Node, S>),
    Axis(EvaluatedAxis<Node, S>),
}

type ContributionStep<Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<CollectedContributions<Node, S>, (Node, ComputeInputOf<S>)>,
    S,
    M,
>;

#[derive(Clone, Copy)]
struct ContributionSource<'input, 'a, Node, S: LayoutScalar> {
    input: &'a GridTrackResolutionInput<'input, 'a, Node, S>,
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    frame: IntrinsicStageFrame<'a, S>,
}

impl<'input, 'a, Node: Copy, S: LayoutScalar> ContributionSource<'input, 'a, Node, S> {
    fn grid_for<M>(
        self,
        node: Node,
        purpose: GridContributionPurpose,
    ) -> LayoutResultOf<Node, IntrinsicGrid<'input, 'a, Node, S>, S, M> {
        intrinsic_role_grid(self.grid, self.frame, purpose)
            .map_err(|error| grid_track_sizing_error(node, error))
    }

    fn project_grid(
        self,
        purpose: GridContributionPurpose,
        available: Size<AxisSizingRequestOf<S>>,
    ) -> IntrinsicGrid<'input, 'a, Node, S> {
        project_intrinsic_role_grid(self.grid, self.frame, purpose, available)
    }
}

impl<Node: Copy + Eq, S: LayoutScalar> ContributionCollection<Node, S> {
    fn new(min_content: StageMinContent<Node, S>) -> Self {
        let mut result = Self::empty(false);
        result.retain_min_content = matches!(min_content, StageMinContent::Retain);
        if let StageMinContent::Observed(observation) = min_content {
            result.grid[1] = Some(observation.grid);
            result.lanes[1] = Some(observation.lanes);
        }
        result
    }

    fn empty(observe_only: bool) -> Self {
        Self {
            index: 0,
            observe_only,
            retain_min_content: false,
            grid: [None, None, None],
            lanes: [None, None, None],
            combined: None,
            role: None,
            pending: None,
            saved_observation: None,
        }
    }

    fn step(&self) -> Option<(TrackContributionKind, usize)> {
        if self.observe_only {
            return match self.index {
                0 => Some((TrackContributionKind::Grid, 1)),
                1 => Some((TrackContributionKind::Lanes, 1)),
                _ => None,
            };
        }
        match self.index {
            0..=2 => Some((TrackContributionKind::Grid, self.index)),
            3..=5 => Some((TrackContributionKind::Lanes, self.index - 3)),
            _ => None,
        }
    }

    fn advance<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        source: ContributionSource<'input, '_, Node, S>,
    ) -> ContributionStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        while let Some((kind, slot)) = self.step() {
            let purpose = CONTRIBUTION_PURPOSES[slot];
            if self.index == 3 && self.combined.is_none() {
                let [minimum, min_content, max_content] = self.grid.each_mut().map(Option::take);
                let group = GridAxisContributions::new(
                    minimum.expect("minimum role completed"),
                    min_content.expect("min-content role completed"),
                    max_content.expect("max-content role completed"),
                );
                let rows = grid_contribution_rows(&group)
                    .map_err(|error| grid_track_sizing_error(node, error))?;
                self.combined = Some(CombinedGridContributions { group, rows });
            }
            match kind {
                TrackContributionKind::Grid if self.grid[slot].is_none() => {
                    if self.role.is_none() {
                        let grid = source.grid_for(node, purpose)?;
                        self.role = Some(ActiveContributionRole {
                            available: grid.query.available,
                            role: ContributionRole::Grid(IntrinsicTrackRole::begin(
                                tree,
                                node,
                                grid,
                                IntrinsicGridLowerBounds::default(),
                            )?),
                        });
                    }
                    let active = self.role.as_mut().expect("active ordinary role");
                    let grid = source.project_grid(purpose, active.available);
                    let ContributionRole::Grid(role) = &mut active.role else {
                        unreachable!("ordinary role owns its child cursor");
                    };
                    if self.pending.is_none() {
                        self.pending = role
                            .next(tree, node, grid)?
                            .map(TrackContributionRequest::Grid);
                    }
                    if let Some(TrackContributionRequest::Grid(child)) = &self.pending {
                        return Ok(ControlFlow::Continue((child.node, child.input(grid)?)));
                    }
                    let Some(ActiveContributionRole {
                        role: ContributionRole::Grid(role),
                        ..
                    }) = self.role.take()
                    else {
                        unreachable!()
                    };
                    self.grid[slot] = Some(role.finish(node, grid)?.into_axis(source.frame.axis));
                }
                TrackContributionKind::Lanes if self.lanes[slot].is_none() => {
                    let Some(mut input) =
                        lane_intrinsic_input(source.input, source.grid, source.frame)
                    else {
                        self.index += 1;
                        continue;
                    };
                    if self.role.is_none() {
                        input.grid = source.grid_for(node, purpose)?;
                        self.role = Some(ActiveContributionRole {
                            available: input.grid.query.available,
                            role: ContributionRole::Lanes(LaneIntrinsicRole::begin(
                                tree, node, input,
                            )),
                        });
                    }
                    let active = self.role.as_mut().expect("active lane role");
                    input.grid = source.project_grid(purpose, active.available);
                    let ContributionRole::Lanes(role) = &mut active.role else {
                        unreachable!("lane role owns its candidate cursor");
                    };
                    if self.pending.is_none() {
                        self.pending = role
                            .next(tree, node, input, purpose)?
                            .map_err(|_| {
                                grid_track_sizing_error(
                                    node,
                                    SpanningContributionError::InvalidIncidence,
                                )
                            })?
                            .map(TrackContributionRequest::Lanes);
                    }
                    if let Some(TrackContributionRequest::Lanes(child)) = &self.pending {
                        return Ok(ControlFlow::Continue((child.node, child.input())));
                    }
                    let Some(ActiveContributionRole {
                        role: ContributionRole::Lanes(role),
                        ..
                    }) = self.role.take()
                    else {
                        unreachable!()
                    };
                    self.lanes[slot] = Some(role.finish(input));
                }
                _ => {}
            }
            self.index += 1;
        }
        if self.observe_only {
            return Ok(ControlFlow::Break(CollectedContributions::Observation(
                MinContentObservation {
                    grid: self.grid[1]
                        .take()
                        .expect("min-content observation completed"),
                    lanes: self.lanes[1]
                        .take()
                        .unwrap_or_else(LanePurposeContributions::empty),
                },
            )));
        }
        let CombinedGridContributions { group, rows } = self
            .combined
            .take()
            .expect("ordinary track rows settled before lane roles");
        let lanes = match self.lanes.each_mut().map(Option::take) {
            [Some(minimum), Some(min_content), Some(max_content)] => Some(combine_lane_purposes(
                node,
                minimum,
                min_content,
                max_content,
                self.retain_min_content,
            )?),
            [None, _, None] => None,
            _ => unreachable!("lane contributions settle all three roles"),
        };
        finish_axis_contributions(node, group, rows, lanes)
            .map(|axis| ControlFlow::Break(CollectedContributions::Axis(axis)))
    }

    fn accept<M>(
        &mut self,
        node: Node,
        source: ContributionSource<'_, '_, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let (_, slot) = self.step().expect("active contribution purpose");
        let purpose = CONTRIBUTION_PURPOSES[slot];
        let active = self.role.as_mut().expect("active contribution role");
        let grid = source.project_grid(purpose, active.available);
        let complete = match (&mut active.role, &mut self.pending) {
            (ContributionRole::Grid(role), Some(TrackContributionRequest::Grid(child))) => {
                role.accept_child(node, grid, child, output)?
            }
            (ContributionRole::Lanes(role), Some(TrackContributionRequest::Lanes(child))) => {
                let input = lane_intrinsic_input(source.input, grid, source.frame)
                    .expect("lane request retains its input");
                role.accept_child(input, child, output)?
            }
            _ => unreachable!("intrinsic child receipt belongs to the active role"),
        };
        if complete {
            self.pending = None;
        }
        Ok(())
    }
}

fn finish_axis_contributions<Node, S: LayoutScalar, M>(
    node: Node,
    mut group: GridAxisContributions<Node, S>,
    mut rows: Vec<GridIntrinsicContributionRow<S>>,
    lanes: Option<LaneIntrinsicContributions<Node, S>>,
) -> StageResult<Node, S, M> {
    let lane_min_content = if let Some(lanes) = lanes {
        rows.try_reserve(lanes.rows.len())
            .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
        rows.extend(lanes.rows);
        for (floor, value) in group.base_floors.iter_mut().zip(lanes.base_floors) {
            *floor = floor.max(value);
        }
        lanes.min_content
    } else {
        LanePurposeContributions::empty()
    };
    Ok(EvaluatedAxis {
        contributions: GridAxisTrackInputs {
            rows,
            base_floors: group.base_floors,
        },
        min_content: MinContentObservation {
            grid: group.min_content,
            lanes: lane_min_content,
        },
        observed_axis: None,
    })
}

fn has_inherited_items<Node: Copy, S: LayoutScalar>(
    grid: IntrinsicGrid<'_, '_, Node, S>,
    axis: GridAxisKind,
) -> bool {
    grid.subgrid_report
        .items
        .iter()
        .any(|item| item_inherits_parent_axis(*item, axis))
}

// All adapters use the same per-role opposite-axis/source preparation.
pub(super) fn intrinsic_role_grid<'input, 'a, Node: Copy, S: LayoutScalar>(
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    frame: IntrinsicStageFrame<'a, S>,
    purpose: GridContributionPurpose,
) -> Result<IntrinsicGrid<'input, 'a, Node, S>, SpanningContributionError<S>> {
    let IntrinsicStageFrame {
        axis,
        stage,
        columns,
        rows,
        ..
    } = frame;
    let role = match purpose {
        GridContributionPurpose::Minimum | GridContributionPurpose::MinContent => {
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent)
        }
        GridContributionPurpose::MaxContent => {
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
        }
    };
    let mut available = LogicalSizeOf::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
    );
    match axis {
        GridAxisKind::Column => {
            available.inline = role;
            if let Some(rows) = rows {
                available.block = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(track_sum_with_gutters(
                        rows,
                        grid.gap.block,
                        grid.row_gutters,
                    ))
                    .map_err(|_| {
                        SpanningContributionError::InvalidNumeric {
                            value: track_sum_with_gutters(rows, grid.gap.block, grid.row_gutters),
                        }
                    })?,
                    SizingOperation::Normal,
                );
            }
        }
        GridAxisKind::Row => {
            available.block = role;
            if let Some(columns) = columns {
                available.inline = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(track_sum_with_gutters(
                        columns,
                        grid.gap.inline,
                        grid.column_gutters,
                    ))
                    .map_err(|_| {
                        SpanningContributionError::InvalidNumeric {
                            value: track_sum_with_gutters(
                                columns,
                                grid.gap.inline,
                                grid.column_gutters,
                            ),
                        }
                    })?,
                    SizingOperation::Normal,
                );
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
                available.block = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(extent)
                        .map_err(|_| SpanningContributionError::InvalidNumeric { value: extent })?,
                    SizingOperation::Normal,
                );
            }
        }
        GridAxisKind::Row if columns.is_none() => {
            if let Some(extent) = estimated_grid.initial_opposite_span(
                GridAxisKind::Column,
                0,
                grid.column_tracks.len(),
            ) {
                available.inline = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(extent)
                        .map_err(|_| SpanningContributionError::InvalidNumeric { value: extent })?,
                    SizingOperation::Normal,
                );
            }
        }
        GridAxisKind::Column | GridAxisKind::Row => {}
    }
    let available = grid.sizing_flow_axes.physical_size(available);
    Ok(project_intrinsic_role_grid(grid, frame, purpose, available))
}

fn project_intrinsic_role_grid<'input, 'a, Node: Copy, S: LayoutScalar>(
    grid: IntrinsicGrid<'input, 'a, Node, S>,
    frame: IntrinsicStageFrame<'a, S>,
    purpose: GridContributionPurpose,
    available: Size<AxisSizingRequestOf<S>>,
) -> IntrinsicGrid<'input, 'a, Node, S> {
    let IntrinsicStageFrame {
        axis,
        stage,
        columns,
        rows,
        observed_axis,
    } = frame;
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
            include_flexible_tracks: grid.query.include_flexible_tracks,
        },
        ..grid
    }
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
    parent_baseline_references: crate::GridBaselineReferences,
    child_style: GridItemProjection<'input, S>,
    container: GridContainerInput<'input, S>,
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
            parent_baseline_references: source.parent_baseline_references,
            child_style: source.child_style.clone(),
            container: source
                .child_container
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

    pub(super) fn container_input(&self) -> GridContainerInput<'input, S> {
        self.container
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
            parent_baseline_references: self.parent_baseline_references,
            child_style: &self.child_style,
            child_container: Some(self.container),
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
            source.container(),
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
