//! Ordinary child observation retains its source across disposable execution.
use super::*;

/// The child placement of a Grid sized without layout, retained while the
/// request executor runs it in a disposable observation session.
pub(crate) enum GridObservation<'input, Node, S: LayoutScalar> {
    Ordinary(Box<OrdinaryGridObservation<'input, Node, S>>),
    Lanes(Box<LanesGridObservation<'input, Node, S>>),
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridObservation<'input, Node, S> {
    pub(crate) fn node(&self) -> Node {
        match self {
            Self::Ordinary(observation) => observation.node(),
            Self::Lanes(observation) => observation.node(),
        }
    }

    pub(crate) fn begin<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, GridObservationRun<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match self {
            Self::Ordinary(observation) => observation.begin(tree),
            Self::Lanes(observation) => observation.begin(tree),
        }
    }

    /// Computes the observation on the native stack, in the session that holds
    /// the tree.
    pub(super) fn compute<Tree, M>(self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: SynchronousCompute<'input, M, Node = Node, Scalar = S>,
    {
        match self {
            Self::Ordinary(observation) => observation.compute(tree),
            Self::Lanes(observation) => observation.compute(tree),
        }
    }
}

/// A Grid Lanes container sized without layout: its accepted sources and
/// tracks, from which its lanes place the children.
pub(crate) struct LanesGridObservation<'input, Node, S: LayoutScalar> {
    pub(super) source: GridAcceptedSource<'input, Node, S>,
    pub(super) tracks: FinalGridLayoutTracks<S>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> LanesGridObservation<'input, Node, S> {
    fn node(&self) -> Node {
        self.source.output.node
    }

    fn begin<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, GridObservationRun<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let work = match self.source.place(tree, self.tracks, None)? {
            GridPlaced::Layout(placed) => GridPass {
                children: Some(GridPassChildren::EmptyLanes(Box::new(placed.layout))),
                output: placed.output,
                scroll_box: placed.scroll_box,
            },
            GridPlaced::Pass(pass) => *pass,
            GridPlaced::Done(_) => unreachable!("a placement pass observes nothing synchronously"),
        };
        Ok(GridObservationRun {
            work: Some(GridPassWork::Placement(Box::new(work))),
        })
    }

    fn compute<Tree, M>(self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: SynchronousCompute<'input, M, Node = Node, Scalar = S>,
    {
        match self.source.place(
            tree,
            self.tracks,
            Some(observe_grid_child_layout::<Tree, M>),
        )? {
            GridPlaced::Layout(placed) => placed
                .output
                .finish(Some((placed.scroll_box, placed.layout))),
            GridPlaced::Pass(pass) => pass.compute(tree),
            GridPlaced::Done(result) => Ok(*result),
        }
    }
}

pub(crate) struct OrdinaryGridObservation<'input, Node, S: LayoutScalar> {
    pub(super) source_constants: Constants<S>,
    pub(super) source_stage: GridSizingStage,
    pub(super) source: GridAcceptedSource<'input, Node, S>,
    pub(super) resolved: GridTrackResolution<Node, S>,
    pub(super) prepared_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
}

pub(crate) struct GridObservationRun<'input, Node, S: LayoutScalar> {
    work: Option<GridPassWork<'input, Node, S>>,
}

pub(crate) struct GridObservationCompletion<'input, Node, S: LayoutScalar> {
    output: GridPassOutput<'input, Node, S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    children: GridChildrenLayout<Node, S>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> OrdinaryGridObservation<'input, Node, S> {
    fn node(&self) -> Node {
        self.source.output.node
    }

    // Rebuild the receipt's borrowed execution frame only while consuming it.
    fn prepare<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, GridPassWork<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            source_constants,
            source_stage,
            source,
            resolved,
            prepared_available,
        } = self;
        let GridAcceptedSource {
            output,
            scroll_box,
            context,
            column_tracks,
            row_tracks,
            parent_context,
            subgrid_report,
            placements,
            sizing_phases,
            stacking_operation,
            fully_contained,
        } = source;
        let minimum_functions = minimum_parent_functions(
            &column_tracks,
            &row_tracks,
            context.percent_basis,
            &parent_context,
        );
        let track_source = GridTrackSource {
            style: &output.style,
            constants: &source_constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            subgrid_report: &subgrid_report,
            sizing_flow_axes: output.constants.flow_axes,
            placements: &placements,
            stage: source_stage,
        };
        let frame = resolved.receipt.frame(track_source);
        let input = GridChildLayoutInput {
            execution_input: output.input,
            scroll_box,
            sizing_phases,
            source_frame: &frame,
            style: &output.style,
            constants: &output.constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            context,
            columns: &resolved.columns,
            rows: &resolved.rows,
            column_preparation: resolved.column_preparation,
            row_preparation: resolved.row_preparation,
            prepared_available,
            output_size: output.output_size,
            subgrid_report: &subgrid_report,
            parent_context: &parent_context,
            placements: &placements,
        };
        let FinalGridTrackPreparation { tracks, receiver } =
            begin_final_grid_tracks(tree, output.node, &input)?;
        let source = GridAcceptedSource {
            context: input.context,
            output,
            scroll_box,
            column_tracks,
            row_tracks,
            parent_context,
            subgrid_report,
            placements,
            sizing_phases,
            stacking_operation,
            fully_contained,
        };
        match receiver {
            Some(receiver) => Ok(GridPassWork::Accepted(Box::new(GridAcceptedSizing {
                source,
                tracks,
                receiver,
            }))),
            None => match source.finish(tree, tracks)? {
                ControlFlow::Continue(work) => Ok(work),
                ControlFlow::Break(_) => unreachable!("ordinary placement retains its child owner"),
            },
        }
    }

    pub(crate) fn begin<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, GridObservationRun<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        Ok(GridObservationRun {
            work: Some(self.prepare(tree)?),
        })
    }

    pub(super) fn compute<Tree, M>(self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: SynchronousCompute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            source_constants,
            source_stage,
            source,
            resolved,
            prepared_available,
        } = self;
        let GridAcceptedSource {
            output,
            scroll_box,
            context,
            column_tracks,
            row_tracks,
            parent_context,
            subgrid_report,
            placements,
            sizing_phases,
            ..
        } = source;
        let minimum_functions = minimum_parent_functions(
            &column_tracks,
            &row_tracks,
            context.percent_basis,
            &parent_context,
        );
        let track_source = GridTrackSource {
            style: &output.style,
            constants: &source_constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            subgrid_report: &subgrid_report,
            sizing_flow_axes: output.constants.flow_axes,
            placements: &placements,
            stage: source_stage,
        };
        let frame = resolved.receipt.frame(track_source);
        let input = GridChildLayoutInput {
            execution_input: output.input,
            scroll_box,
            sizing_phases,
            source_frame: &frame,
            style: &output.style,
            constants: &output.constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            context,
            columns: &resolved.columns,
            rows: &resolved.rows,
            column_preparation: resolved.column_preparation,
            row_preparation: resolved.row_preparation,
            prepared_available,
            output_size: output.output_size,
            subgrid_report: &subgrid_report,
            parent_context: &parent_context,
            placements: &placements,
        };
        let children =
            observe_grid_child_layout(tree, output.node, GridOverflowLayout::Ordinary(input))?;
        output.finish(Some((scroll_box, children)))
    }
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridObservationRun<'input, Node, S> {
    pub(crate) fn advance<Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, Option<GridChild<'input, Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match self.work.as_mut().expect("active ordinary observation") {
                GridPassWork::Accepted(accepted) => {
                    if let ControlFlow::Continue(request) = accepted.advance(tree)? {
                        return Ok(Some(GridChild {
                            phase: GridChildPhase::AcceptedTrack(request),
                        }));
                    }
                    let Some(GridPassWork::Accepted(accepted)) = self.work.take() else {
                        unreachable!()
                    };
                    self.work = Some(match accepted.finish(tree)? {
                        ControlFlow::Continue(work) => work,
                        ControlFlow::Break(_) => {
                            unreachable!("ordinary placement retains its child owner")
                        }
                    });
                }
                GridPassWork::Placement(pass) => return pass.next(tree),
                GridPassWork::Observation(_) | GridPassWork::Observed(_) => {
                    unreachable!("an observation initializes its ordinary placement")
                }
            }
        }
    }

    pub(crate) fn accept<Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: GridChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match self.work.as_mut().expect("active ordinary observation") {
            GridPassWork::Accepted(accepted) => accepted.accept(output),
            GridPassWork::Placement(pass) => pass.accept(tree, child, output),
            GridPassWork::Observation(_) | GridPassWork::Observed(_) => {
                unreachable!("active observation owns its child phase")
            }
        }
    }

    pub(crate) fn accept_context<Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: GridChild<'input, Node, S>,
        result: GridComputeResult<Node, S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match self.work.as_mut().expect("active ordinary observation") {
            GridPassWork::Accepted(accepted) => accepted.accept_context(result),
            GridPassWork::Placement(pass) => pass.accept_context(tree, child, result),
            GridPassWork::Observation(_) | GridPassWork::Observed(_) => {
                unreachable!("only placement and track sizing issue contextual requests")
            }
        }
    }

    /// Closes the effect captures the traversals of the observation's track
    /// sizing hold.
    pub(crate) fn take_captures(&mut self, starts: &mut Vec<usize>) {
        if let Some(GridPassWork::Accepted(accepted)) = self.work.as_mut() {
            accepted.take_captures(starts);
        }
    }

    pub(crate) fn finish<M>(
        self,
    ) -> LayoutResultOf<Node, GridObservationCompletion<'input, Node, S>, S, M> {
        let Some(GridPassWork::Placement(pass)) = self.work else {
            unreachable!("completed ordinary child placement")
        };
        let GridPass {
            output,
            scroll_box,
            children,
        } = *pass;
        let children = finish_grid_children(children)?;
        let observation = GridChildrenObservation {
            visible_content_size: children.visible_content_size,
            contributions: children.contributions,
            baselines: children.baselines,
        };
        Ok(GridObservationCompletion {
            output,
            scroll_box,
            children: observation.into_children(),
        })
    }
}

impl<Node: Copy, S: LayoutScalar> GridObservationCompletion<'_, Node, S> {
    pub(crate) fn node(&self) -> Node {
        self.output.node
    }

    pub(super) fn finish<M>(self) -> GridComputeOutcome<Node, S, M> {
        self.output.finish(Some((self.scroll_box, self.children)))
    }
}
