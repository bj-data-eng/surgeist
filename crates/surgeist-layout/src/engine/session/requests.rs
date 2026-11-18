use std::{collections::TryReserveError, ops::ControlFlow};

use super::memo::Capture;
use super::{
    ChildAdmission, ChildCacheRequest, ComputeSession, FloatChildTrial, ScopedChild,
    ScopedChildStart, SessionResult,
};
use crate::block::intrinsic::{BlockAxisDemand, InlineContentChild};
use crate::block::{
    BlockChildReceipt, BlockChildRequest, BlockComputation, BlockResult, PendingBlockChild,
    begin_block_computation,
};
use crate::engine::block_scope::{ComputeResult, ComputeStop, ordinary};
use crate::engine::hidden::{HiddenChildren, HiddenRequest};
use crate::flex::{FlexChild, FlexComputation, begin_flex_computation};
use crate::grid::{
    GridChild, GridComputeResult, GridContextRequest, GridContinuation,
    GridRequest as GridComputationRequest, OrdinaryGridObservation,
    OrdinaryGridObservationCompletion, OrdinaryGridObservationRun, begin_grid_computation,
    begin_grid_context_computation,
};
use crate::{
    Compute, ComputeInputOf, ComputeOutputOf, Display, FlowAxes, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutOperation, LayoutScalar, LayoutTree, PhysicalAxis, Traverse,
};

enum Request<'input, Node, S: LayoutScalar> {
    Child {
        node: Node,
        input: ComputeInputOf<S>,
        projection: OutputProjection,
    },
    Hidden(HiddenRequest<Node>, OutputProjection),
    HiddenSiblings(HiddenChildren<Node>, OutputProjection),
    Axis {
        node: Node,
        input: ComputeInputOf<S>,
        axis: PhysicalAxis,
    },
    ScopedChild(ScopedChild<Node, S>),
    AdvanceBlock,
    AdvanceFlex,
    AdvanceGrid,
    /// A subgrid item's contextual Grid computation; it bypasses child admission.
    ContextGrid(Box<GridContextRequest<'input, Node, S>>),
    GridObservation(Box<OrdinaryGridObservation<'input, Node, S>>),
    GridObservationRun(Box<OrdinaryGridObservationRun<'input, Node, S>>),
    Qualify(crate::block::entry::EntryQuery<Node, S>),
    Entry(Box<crate::block::entry::FramedEntry<Node, S>>),
    FloatTrial(FloatTrialRequest<Node, S>),
    Complete(CompletedRequest<'input, Node, S>),
}

struct FloatTrialRequest<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    trial: FloatChildTrial,
}

enum RequestProgress<'input, Node, S: LayoutScalar> {
    Complete(CompletedRequest<'input, Node, S>),
    GridObservation(Box<OrdinaryGridObservation<'input, Node, S>>),
    Qualification(crate::block::entry::EntryQuery<Node, S>),
    SourceFloat(Box<crate::block::entry::EntryFloat<'input, Node, S>>),
    FloatTrial(FloatTrialRequest<Node, S>),
}

#[derive(Clone, Copy)]
enum OutputProjection {
    Layout,
    Axis(PhysicalAxis),
}

impl OutputProjection {
    fn complete<'input, Node, S: LayoutScalar>(
        self,
        output: ComputeOutputOf<S>,
    ) -> Request<'input, Node, S> {
        let output = match self {
            Self::Layout => CompletedRequest::Layout(output),
            Self::Axis(PhysicalAxis::Horizontal) => CompletedRequest::Axis(output.size.width),
            Self::Axis(PhysicalAxis::Vertical) => CompletedRequest::Axis(output.size.height),
        };
        Request::Complete(output)
    }
}

enum CompletedRequest<'input, Node, S: LayoutScalar> {
    GridObservation(Box<OrdinaryGridObservationCompletion<'input, Node, S>>),
    Context(Box<GridComputeResult<Node, S>>),
    Entry(Box<crate::block::entry::EntryObservation<Node, S>>),
    Layout(ComputeOutputOf<S>),
    Axis(S),
}

impl<'input, Node, S: LayoutScalar> CompletedRequest<'input, Node, S> {
    fn layout(self) -> ComputeOutputOf<S> {
        match self {
            Self::Layout(output) => output,
            Self::GridObservation(_) | Self::Context(_) | Self::Entry(_) | Self::Axis(_) => {
                unreachable!("a layout request completes with layout geometry")
            }
        }
    }

    fn axis(self) -> S {
        match self {
            Self::Axis(output) => output,
            Self::GridObservation(_) | Self::Context(_) | Self::Entry(_) | Self::Layout(_) => {
                unreachable!("an axis request completes with a scalar")
            }
        }
    }
}

enum PendingRequest {
    GridObservation,
    HiddenSiblings,
    BlockAxis,
    Block,
    Flex,
    Grid,
}

struct BlockRequest<'input, Node, S: LayoutScalar> {
    computation: BlockComputation<'input, Node, S>,
    completion: ChildCompletion<Node, S>,
    projection: OutputProjection,
    child: Option<PendingBlockChild<'input, Node, S>>,
    /// A Block computed under installed fragmentation scopes: its scoped Block
    /// box children continue as scoped child requests.
    scoped: bool,
    /// The scope frame depth of a scoped Block box; `None` for any other.
    scope_depth: Option<usize>,
}

struct FlexRequest<'input, Node, S: LayoutScalar> {
    computation: FlexComputation<'input, Node, S>,
    completion: ChildCompletion<Node, S>,
    projection: OutputProjection,
    child: Option<FlexChild<'input, Node, S>>,
}

struct GridRequest<'input, Node, S: LayoutScalar> {
    computation: GridContinuation<'input, Node, S>,
    completion: GridCompletion<Node, S>,
    child: Option<GridChild<'input, Node, S>>,
}

enum GridCompletion<Node, S: LayoutScalar> {
    Child {
        completion: ChildCompletion<Node, S>,
        projection: OutputProjection,
    },
    /// A contextual computation caches and memoizes nothing; its receipt is
    /// the Grid result its parent consumes.
    Context {
        /// Where the writes of the computation begin in the session's capture.
        effects_start: usize,
    },
}

struct GridObservationRequest<'input, Node, S: LayoutScalar> {
    computation: Box<OrdinaryGridObservationRun<'input, Node, S>>,
    child: GridChild<'input, Node, S>,
}

struct PendingRequests<'input, Node, S: LayoutScalar> {
    grid_observations: Vec<GridObservationRequest<'input, Node, S>>,
    order: Vec<PendingRequest>,
    hidden: Vec<(HiddenChildren<Node>, OutputProjection)>,
    block_axes: Vec<(BlockAxisDemand<Node, S>, InlineContentChild<Node, S>)>,
    blocks: Vec<BlockRequest<'input, Node, S>>,
    flexes: Vec<FlexRequest<'input, Node, S>>,
    grids: Vec<GridRequest<'input, Node, S>>,
}

impl<Node, S: LayoutScalar> PendingRequests<'_, Node, S> {
    fn new() -> Self {
        Self {
            grid_observations: Vec::new(),
            order: Vec::new(),
            hidden: Vec::new(),
            block_axes: Vec::new(),
            blocks: Vec::new(),
            flexes: Vec::new(),
            grids: Vec::new(),
        }
    }

    fn keep_hidden(
        &mut self,
        children: HiddenChildren<Node>,
        projection: OutputProjection,
    ) -> Result<(), TryReserveError> {
        self.order.try_reserve(1)?;
        self.hidden.try_reserve(1)?;
        self.hidden.push((children, projection));
        self.order.push(PendingRequest::HiddenSiblings);
        Ok(())
    }

    fn keep_block_axis(
        &mut self,
        parent: BlockAxisDemand<Node, S>,
        child: InlineContentChild<Node, S>,
    ) -> Result<(), TryReserveError> {
        self.order.try_reserve(1)?;
        self.block_axes.try_reserve(1)?;
        self.block_axes.push((parent, child));
        self.order.push(PendingRequest::BlockAxis);
        Ok(())
    }
}

type StopResult<Tree, T> = ComputeResult<
    <Tree as Traverse>::Node,
    T,
    <Tree as Traverse>::Scalar,
    <Tree as LayoutTree>::MeasureError,
>;
type SessionRequest<'input, Tree> =
    Request<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;
type SessionPending<'input, Tree> =
    PendingRequests<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;
type StagedProgress<'input, Tree> = ControlFlow<
    CompletedRequest<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    SessionRequest<'input, Tree>,
>;
type EntryProgress<'input, Tree> = ControlFlow<
    RequestProgress<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    SessionRequest<'input, Tree>,
>;
struct ChildCompletion<Node, S: LayoutScalar> {
    cache: Option<ChildCacheRequest<Node, S>>,
    capture: Option<Capture<S>>,
}
type SessionBlock<'input, Tree> =
    BlockRequest<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

enum StagedDestination<'input, Node, S: LayoutScalar> {
    Qualification,
    GridObservation,
    FloatTrial(FloatTrialRequest<Node, S>),
    SourceFloat {
        request: FloatTrialRequest<Node, S>,
        pending: Box<crate::block::entry::EntryFloat<'input, Node, S>>,
        observation: Option<crate::engine::entry_sources::FloatReservation<Node, S>>,
    },
}

struct StagedTrial<'input, Tree: LayoutTree> {
    destination: StagedDestination<'input, Tree::Node, Tree::Scalar>,
    session: ComputeSession<'input, Tree>,
    pending: SessionPending<'input, Tree>,
}

fn reserve_error<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::Allocation,
    )
}

impl<'input, Tree: LayoutTree> ComputeSession<'input, Tree> {
    pub(super) fn drive_entry(
        &mut self,
        query: crate::block::entry::EntryQuery<Tree::Node, Tree::Scalar>,
    ) -> SessionResult<Tree, crate::block::entry::EntryObservation<Tree::Node, Tree::Scalar>> {
        match ordinary(self.drive_requests(Request::Qualify(query)))? {
            CompletedRequest::Entry(observation) => Ok(*observation),
            _ => unreachable!("entry operation returns entry facts"),
        }
    }

    pub(super) fn drive_child(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, ComputeOutputOf<Tree::Scalar>> {
        ordinary(self.drive_requests(Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        }))
        .map(CompletedRequest::layout)
    }

    /// Computes a scoped Block box and its scoped Block descendants on the
    /// request stack. A suspension keeps its scope frames in the scope request.
    pub(super) fn drive_scoped_child(
        &mut self,
        child: ScopedChild<Tree::Node, Tree::Scalar>,
    ) -> StopResult<Tree, ComputeOutputOf<Tree::Scalar>> {
        let depth = child.depth;
        let result = self.drive_requests(Request::ScopedChild(child));
        if !matches!(result, Err(ComputeStop::NeedScope(_))) {
            self.release_scope_frames(depth);
        }
        result.map(CompletedRequest::layout)
    }

    pub(super) fn drive_hidden(
        &mut self,
        request: HiddenRequest<Tree::Node>,
    ) -> SessionResult<Tree, ComputeOutputOf<Tree::Scalar>> {
        ordinary(self.drive_requests(Request::Hidden(request, OutputProjection::Layout)))
            .map(CompletedRequest::layout)
    }

    pub(super) fn drive_axis(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        axis: PhysicalAxis,
    ) -> SessionResult<Tree, Tree::Scalar> {
        ordinary(self.drive_requests(Request::Axis { node, input, axis }))
            .map(CompletedRequest::axis)
    }

    fn drive_requests(
        &mut self,
        request: SessionRequest<'input, Tree>,
    ) -> StopResult<Tree, CompletedRequest<'input, Tree::Node, Tree::Scalar>> {
        let mut pending = PendingRequests::new();
        let mut staged = Vec::new();
        let result = self.drive_staged_requests(request, &mut pending, &mut staged);
        result.map_err(|mut stop| {
            while let Some(mut trial) = staged.pop() {
                trial
                    .session
                    .close_context_captures(&mut trial.pending.grids);
                stop = trial.session.unwind_blocks(trial.pending.blocks, stop);
            }
            self.close_context_captures(&mut pending.grids);
            self.unwind_blocks(pending.blocks, stop)
        })
    }

    fn drive_staged_requests(
        &mut self,
        mut request: SessionRequest<'input, Tree>,
        pending: &mut SessionPending<'input, Tree>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> StopResult<Tree, CompletedRequest<'input, Tree::Node, Tree::Scalar>> {
        loop {
            let progress = if let Some(active) = staged.last_mut() {
                active.session.run_requests(request, &mut active.pending)?
            } else {
                self.run_requests(request, pending)?
            };
            match progress {
                RequestProgress::FloatTrial(trial) => {
                    staged
                        .try_reserve(1)
                        .map_err(|_| reserve_error(trial.node))?;
                    let parent = staged
                        .last_mut()
                        .map_or(&mut *self, |active| &mut active.session);
                    let session = parent.child_staging(trial.node, trial.trial, None)?;
                    request = Request::Child {
                        node: trial.node,
                        input: trial.input,
                        projection: OutputProjection::Layout,
                    };
                    staged.push(StagedTrial {
                        destination: StagedDestination::FloatTrial(trial),
                        session,
                        pending: PendingRequests::new(),
                    });
                }
                progress @ RequestProgress::GridObservation(_) => {
                    request = self.stage_grid_observation(progress, staged)?;
                }
                RequestProgress::Qualification(query) => {
                    request = self.stage_qualification(query, staged)?;
                }
                RequestProgress::SourceFloat(source) => {
                    request = self.stage_source_float(source, staged)?;
                }
                RequestProgress::Complete(output) => {
                    match self.complete_staged(output, pending, staged)? {
                        ControlFlow::Break(output) => return Ok(output),
                        ControlFlow::Continue(next) => request = next,
                    }
                }
            }
        }
    }

    // Phase-local construction finishes before the driver executes another
    // child. Ordinary request frames do not retain inactive entry owners.
    fn stage_qualification(
        &mut self,
        query: crate::block::entry::EntryQuery<Tree::Node, Tree::Scalar>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        staged
            .try_reserve(1)
            .map_err(|_| reserve_error(query.node))?;
        let parent = staged
            .last_mut()
            .map_or(&mut *self, |active| &mut active.session);
        let mut session = parent.prepare_entry_trial(&query)?;
        let entry = crate::block::entry::FramedEntry::begin(&mut session, query)?;
        let request = Request::Entry(Box::new(entry));
        staged.push(StagedTrial {
            destination: StagedDestination::Qualification,
            session,
            pending: PendingRequests::new(),
        });
        Ok(request)
    }

    fn stage_source_float(
        &mut self,
        source: Box<crate::block::entry::EntryFloat<'input, Tree::Node, Tree::Scalar>>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let (node, input) = source.request();
        staged.try_reserve(1).map_err(|_| reserve_error(node))?;
        let parent = staged
            .last_mut()
            .map_or(&mut *self, |active| &mut active.session);
        let observation = match parent.source_float_observation(node, input)? {
            Some(crate::engine::entry_sources::FloatObservation::Hit(output)) => {
                return Ok(Request::Entry(Box::new(
                    crate::block::entry::FramedEntry::after_float(parent, *source, output)?,
                )));
            }
            Some(crate::engine::entry_sources::FloatObservation::Miss(reservation)) => {
                Some(reservation)
            }
            None => None,
        };
        let session = parent.child_staging(node, FloatChildTrial::Detached, None)?;
        let request = Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        };
        staged.push(StagedTrial {
            destination: StagedDestination::SourceFloat {
                request: FloatTrialRequest {
                    node,
                    input,
                    trial: FloatChildTrial::Detached,
                },
                pending: source,
                observation,
            },
            session,
            pending: PendingRequests::new(),
        });
        Ok(request)
    }

    fn stage_grid_observation(
        &mut self,
        progress: RequestProgress<'input, Tree::Node, Tree::Scalar>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let RequestProgress::GridObservation(source) = progress else {
            unreachable!("the observation phase owns its source");
        };
        let node = source.node();
        staged.try_reserve(1).map_err(|_| reserve_error(node))?;
        let parent = staged
            .last_mut()
            .map_or(&mut *self, |active| &mut active.session);
        let mut session = parent.grid_observation_session(node)?;
        let run = source.begin(&mut session)?;
        let request = Request::GridObservationRun(Box::new(run));
        staged.push(StagedTrial {
            destination: StagedDestination::GridObservation,
            session,
            pending: PendingRequests::new(),
        });
        Ok(request)
    }

    fn advance_grid_observation_request(
        &mut self,
        request: SessionRequest<'input, Tree>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let Request::GridObservationRun(mut run) = request else {
            unreachable!()
        };
        if let Some(mut child) = run.advance(self)? {
            let (node, input) = child.request();
            pending
                .order
                .try_reserve(1)
                .map_err(|_| reserve_error(node))?;
            pending
                .grid_observations
                .try_reserve(1)
                .map_err(|_| reserve_error(node))?;
            let request = match child.take_context_request() {
                Some(request) => Request::ContextGrid(Box::new(request)),
                None => Request::Child {
                    node,
                    input,
                    projection: OutputProjection::Layout,
                },
            };
            pending.grid_observations.push(GridObservationRequest {
                computation: run,
                child,
            });
            pending.order.push(PendingRequest::GridObservation);
            Ok(request)
        } else {
            Ok(Request::Complete(CompletedRequest::GridObservation(
                Box::new(run.finish()?),
            )))
        }
    }

    fn complete_staged(
        &mut self,
        output: CompletedRequest<'input, Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> StopResult<Tree, StagedProgress<'input, Tree>> {
        let Some(finished) = staged.pop() else {
            return Ok(ControlFlow::Break(output));
        };
        let (parent, pending) = if let Some(active) = staged.last_mut() {
            (&mut active.session, &mut active.pending)
        } else {
            (&mut *self, &mut *pending)
        };
        let request = match finished.destination {
            StagedDestination::GridObservation => {
                let CompletedRequest::GridObservation(completion) = &output else {
                    unreachable!()
                };
                parent.accept_grid_observation(completion.node(), &finished.session)?;
                drop(finished.session);
                Request::Complete(output)
            }
            StagedDestination::Qualification => {
                drop(finished.session);
                Request::Complete(output)
            }
            StagedDestination::FloatTrial(trial) => {
                let packet = parent.seal_float_trial(
                    trial.node,
                    trial.input,
                    trial.trial,
                    finished.session,
                    output.layout(),
                )?;
                assert!(matches!(pending.order.pop(), Some(PendingRequest::Block)));
                parent.resume_block(BlockChildReceipt::FloatTrial(packet), pending)?
            }
            StagedDestination::SourceFloat {
                request: trial,
                pending: source,
                observation,
            } => {
                let packet = parent.seal_float_trial(
                    trial.node,
                    trial.input,
                    trial.trial,
                    finished.session,
                    output.layout(),
                )?;
                let output = packet.output;
                drop(packet);
                if let Some(reservation) = observation {
                    reservation.complete(output);
                }
                Request::Entry(Box::new(crate::block::entry::FramedEntry::after_float(
                    parent, *source, output,
                )?))
            }
        };
        Ok(ControlFlow::Continue(request))
    }

    fn advance_entry_request(
        &mut self,
        request: SessionRequest<'input, Tree>,
    ) -> StopResult<Tree, EntryProgress<'input, Tree>> {
        let Request::Entry(entry) = request else {
            unreachable!("entry advancement receives the retained entry request");
        };
        match entry.advance(self)? {
            ControlFlow::Break(observation) => Ok(ControlFlow::Continue(Request::Complete(
                CompletedRequest::Entry(Box::new(observation)),
            ))),
            ControlFlow::Continue(source) => Ok(ControlFlow::Break(RequestProgress::SourceFloat(
                Box::new(source),
            ))),
        }
    }

    /// Closes the effect captures of the contextual Grid requests a failure
    /// left suspended, innermost first. They have no writes to replay.
    fn close_context_captures(
        &mut self,
        grids: &mut Vec<GridRequest<'input, Tree::Node, Tree::Scalar>>,
    ) {
        while let Some(grid) = grids.pop() {
            if let GridCompletion::Context { effects_start } = grid.completion {
                self.end_effect_capture(effects_start);
            }
        }
    }

    fn unwind_blocks(
        &mut self,
        blocks: Vec<SessionBlock<'input, Tree>>,
        mut stop: ComputeStop<Tree::Node, Tree::Scalar, Tree::MeasureError>,
    ) -> ComputeStop<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        for frame in blocks.into_iter().rev() {
            if let Some((node, input)) = frame
                .child
                .as_ref()
                .and_then(PendingBlockChild::shared_request)
            {
                let _ = self.end_child_float_context(node, input);
            }
            stop = frame
                .computation
                .finish(self, Err(stop))
                .expect_err("failed child leaves its Block owner failed");
            // A suspension has already moved its frames into the scope request.
            if let Some(depth) = frame.scope_depth
                && !matches!(stop, ComputeStop::NeedScope(_))
            {
                self.release_scope_frames(depth);
            }
        }
        stop
    }

    fn run_requests(
        &mut self,
        mut request: SessionRequest<'input, Tree>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, RequestProgress<'input, Tree::Node, Tree::Scalar>> {
        loop {
            request = match request {
                Request::Child {
                    node,
                    input,
                    projection,
                } => match self.admit_child(node, input)? {
                    ChildAdmission::Ready(output) => projection.complete(output),
                    ChildAdmission::Hidden(request) => Request::Hidden(request, projection),
                    ChildAdmission::Uncached { input, cache, memo } => {
                        let capture = self.begin_memo(node, memo)?;
                        let initial = self.begin_uncached_layout(node, input);
                        let completion = ChildCompletion { cache, capture };
                        let container_display =
                            (!initial.leaf).then(|| initial.style.display.inner_display());
                        match container_display {
                            Some(Display::Block) => {
                                self.begin_block(node, input, completion, projection, pending)?
                            }
                            Some(Display::Flex) => {
                                self.begin_flex(node, input, completion, projection, pending)?
                            }
                            Some(Display::Grid | Display::GridLanes) => {
                                self.begin_grid(node, input, completion, projection, pending)?
                            }
                            _ => {
                                let output =
                                    self.compute_uncached_layout_body(node, input, initial)?;
                                self.complete_child_request(node, completion, projection, output)?
                            }
                        }
                    }
                },
                Request::ScopedChild(child) => self.begin_scoped_block(child, pending)?,
                Request::Hidden(request, projection) => {
                    let children = HiddenChildren::begin(self, request)?;
                    self.advance_hidden(children, projection, pending)?
                }
                Request::HiddenSiblings(children, projection) => {
                    self.advance_hidden(children, projection, pending)?
                }
                Request::Axis { node, input, axis } => {
                    self.begin_axis(node, input, axis, pending)?
                }
                Request::AdvanceBlock => self.advance_block(pending)?,
                Request::AdvanceFlex => self.advance_flex(pending)?,
                Request::AdvanceGrid => self.advance_grid(pending)?,
                Request::GridObservation(source) => {
                    return Ok(RequestProgress::GridObservation(source));
                }
                Request::ContextGrid(request) => self.begin_context_grid(*request, pending)?,
                request @ Request::GridObservationRun(_) => {
                    self.advance_grid_observation_request(request, pending)?
                }
                Request::Qualify(query) => return Ok(RequestProgress::Qualification(query)),
                entry @ Request::Entry(_) => match self.advance_entry_request(entry)? {
                    ControlFlow::Break(progress) => return Ok(progress),
                    ControlFlow::Continue(request) => request,
                },
                Request::FloatTrial(trial) => return Ok(RequestProgress::FloatTrial(trial)),
                Request::Complete(output) => match pending.order.pop() {
                    Some(PendingRequest::GridObservation) => {
                        self.resume_grid_observation(output, pending)?
                    }
                    None => return Ok(RequestProgress::Complete(output)),
                    Some(PendingRequest::HiddenSiblings) => {
                        let (children, projection) = pending
                            .hidden
                            .pop()
                            .expect("hidden sibling state follows request order");
                        Request::HiddenSiblings(children, projection)
                    }
                    Some(PendingRequest::BlockAxis) => {
                        let (mut parent, child) = pending
                            .block_axes
                            .pop()
                            .expect("Block axis state follows request order");
                        parent.accept_child(child, output.axis());
                        self.advance_block_axis(parent, pending)?
                    }
                    Some(PendingRequest::Block) => self.resume_block(
                        match output {
                            CompletedRequest::Layout(output) => BlockChildReceipt::Layout(output),
                            CompletedRequest::Entry(observation) => {
                                BlockChildReceipt::Entry(observation)
                            }
                            CompletedRequest::GridObservation(_)
                            | CompletedRequest::Context(_)
                            | CompletedRequest::Axis(_) => {
                                unreachable!("Block child has layout or entry result")
                            }
                        },
                        pending,
                    )?,
                    Some(PendingRequest::Flex) => {
                        let parent = pending.flexes.last_mut().expect("active Flex request");
                        let child = parent.child.take().expect("pending Flex child request");
                        parent
                            .computation
                            .accept_child(self, child, output.layout())?;
                        Request::AdvanceFlex
                    }
                    Some(PendingRequest::Grid) => self.resume_grid(output, pending)?,
                },
            };
        }
    }

    fn begin_block(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        match begin_block_computation(self, node, input)? {
            ControlFlow::Break(output) => {
                self.finish_container_request(node, input, completion, projection, output)
            }
            ControlFlow::Continue(computation) => {
                if pending.blocks.try_reserve(1).is_err() {
                    return computation
                        .finish(self, Err(reserve_error(node).into()))
                        .map(|output| projection.complete(output));
                }
                pending.blocks.push(BlockRequest {
                    computation,
                    completion,
                    projection,
                    child: None,
                    scoped: self.block_scopes().is_some(),
                    scope_depth: None,
                });
                Ok(Request::AdvanceBlock)
            }
        }
    }

    fn resume_block(
        &mut self,
        output: BlockChildReceipt<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.blocks.last_mut().expect("active Block request");
        let child = parent.child.take().expect("pending Block child request");
        parent.computation.accept_child(self, child, output)?;
        Ok(Request::AdvanceBlock)
    }

    fn advance_block(
        &mut self,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.blocks.last_mut().expect("active Block request");
        let scoped = parent.scoped;
        match parent.computation.advance(self)? {
            ControlFlow::Break(result) => self.complete_block(result, pending),
            ControlFlow::Continue(child) => {
                if scoped && let Some(request) = child.scoped_box_request() {
                    return self.advance_scoped_box(child, request, pending);
                }
                let (request, child) = child.into_request();
                let parent = pending.blocks.last_mut().expect("active Block request");
                if let BlockChildRequest::Entry(query) = request {
                    parent.child = Some(child);
                    pending
                        .order
                        .try_reserve(1)
                        .map_err(|_| reserve_error(query.node))?;
                    pending.order.push(PendingRequest::Block);
                    return Ok(Request::Qualify(*query));
                }
                let BlockChildRequest::Layout(request) = request else {
                    unreachable!()
                };
                debug_assert!(
                    request.entry.is_none(),
                    "ordinary request has no scope entry"
                );
                let trial = child.float_trial();
                parent.child = Some(child);
                pending
                    .order
                    .try_reserve(1)
                    .map_err(|_| reserve_error(request.node))?;
                pending.order.push(PendingRequest::Block);
                if let Some(trial) = trial {
                    return Ok(Request::FloatTrial(FloatTrialRequest {
                        node: request.node,
                        input: request.input,
                        trial,
                    }));
                }
                Ok(Request::Child {
                    node: request.node,
                    input: request.input,
                    projection: OutputProjection::Layout,
                })
            }
        }
    }

    /// Continues a scoped Block box child on the request stack, after its scope
    /// entry and input are settled.
    fn advance_scoped_box(
        &mut self,
        child: crate::block::BlockChild<'input, Tree::Node, Tree::Scalar>,
        request: crate::block::scope::ChildRequest<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let (_, accepted) = child.into_request();
        pending
            .order
            .try_reserve(1)
            .map_err(|_| reserve_error(request.node))?;
        let parent = pending.blocks.last_mut().expect("active Block request");
        match self.begin_scoped_child(request)? {
            ScopedChildStart::Output(output) => {
                parent.computation.accept_child(
                    self,
                    accepted,
                    BlockChildReceipt::Layout(output),
                )?;
                Ok(Request::AdvanceBlock)
            }
            ScopedChildStart::Fallback { node, input } => {
                parent.child = Some(accepted);
                pending.order.push(PendingRequest::Block);
                Ok(Request::Child {
                    node,
                    input,
                    projection: OutputProjection::Layout,
                })
            }
            ScopedChildStart::Descend(child) => {
                parent.child = Some(accepted);
                pending.order.push(PendingRequest::Block);
                Ok(Request::ScopedChild(child))
            }
        }
    }

    fn complete_block(
        &mut self,
        result: BlockResult<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.blocks.pop().expect("completed Block request");
        let (node, input) = parent.computation.request();
        let output = parent.computation.finish(self, Ok(result))?;
        if let Some(depth) = parent.scope_depth {
            self.finish_uncached_layout(node, input, output)?;
            self.release_scope_frames(depth);
            return Ok(parent.projection.complete(output));
        }
        self.finish_container_request(node, input, parent.completion, parent.projection, output)
    }

    /// Starts a scoped Block box whose scope frame is already pushed. A leaf
    /// completes at once; a Block container keeps its computation on the
    /// request stack, never on the native stack.
    fn begin_scoped_block(
        &mut self,
        child: ScopedChild<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let depth = child.depth;
        let result = self.start_scoped_block(child, pending);
        // Only a started Block request owns its frame, and a suspension has
        // moved its frames into the scope request.
        if matches!(result, Err(ComputeStop::Layout(_))) {
            self.release_scope_frames(depth);
        }
        result
    }

    fn start_scoped_block(
        &mut self,
        child: ScopedChild<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let ScopedChild { node, input, depth } = child;
        let initial = self.begin_uncached_layout(node, input);
        if initial.leaf || initial.style.display.inner_display() != Display::Block {
            let output = self.compute_uncached_layout_body(node, input, initial)?;
            self.release_scope_frames(depth);
            return Ok(Request::Complete(CompletedRequest::Layout(output)));
        }
        match begin_block_computation(self, node, input)? {
            ControlFlow::Break(output) => {
                self.finish_uncached_layout(node, input, output)?;
                self.release_scope_frames(depth);
                Ok(Request::Complete(CompletedRequest::Layout(output)))
            }
            ControlFlow::Continue(computation) => {
                if pending.blocks.try_reserve(1).is_err() {
                    return computation
                        .finish(self, Err(reserve_error(node).into()))
                        .map(|output| OutputProjection::Layout.complete(output));
                }
                pending.blocks.push(BlockRequest {
                    computation,
                    completion: ChildCompletion {
                        cache: None,
                        capture: None,
                    },
                    projection: OutputProjection::Layout,
                    child: None,
                    scoped: true,
                    scope_depth: Some(depth),
                });
                Ok(Request::AdvanceBlock)
            }
        }
    }

    fn begin_flex(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        match begin_flex_computation(self, node, input)? {
            ControlFlow::Break(output) => {
                self.finish_container_request(node, input, completion, projection, output)
            }
            ControlFlow::Continue(mut computation) => match computation.advance(self)? {
                ControlFlow::Break(output) => {
                    self.finish_container_request(node, input, completion, projection, output)
                }
                ControlFlow::Continue(child) => {
                    pending
                        .flexes
                        .try_reserve(1)
                        .map_err(|_| reserve_error(node))?;
                    let request = Self::schedule_flex_child(&child, &mut pending.order)?;
                    pending.flexes.push(FlexRequest {
                        computation,
                        completion,
                        projection,
                        child: Some(child),
                    });
                    Ok(request)
                }
            },
        }
    }

    fn advance_flex(
        &mut self,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.flexes.last_mut().expect("active Flex request");
        match parent.computation.advance(self)? {
            ControlFlow::Break(output) => {
                let parent = pending.flexes.pop().expect("completed Flex request");
                let (node, input) = parent.computation.request();
                self.finish_container_request(
                    node,
                    input,
                    parent.completion,
                    parent.projection,
                    output,
                )
            }
            ControlFlow::Continue(child) => {
                let request = Self::schedule_flex_child(&child, &mut pending.order)?;
                parent.child = Some(child);
                Ok(request)
            }
        }
    }

    fn schedule_flex_child(
        child: &FlexChild<'input, Tree::Node, Tree::Scalar>,
        order: &mut Vec<PendingRequest>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let (node, input) = child.request();
        order.try_reserve(1).map_err(|_| reserve_error(node))?;
        order.push(PendingRequest::Flex);
        Ok(Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        })
    }

    fn resume_grid_observation(
        &mut self,
        output: CompletedRequest<'input, Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let GridObservationRequest {
            computation: mut run,
            child,
        } = pending
            .grid_observations
            .pop()
            .expect("active observation child");
        match output {
            CompletedRequest::Context(result) => run.accept_context(self, child, *result)?,
            output => run.accept(self, child, output.layout())?,
        }
        Ok(Request::GridObservationRun(run))
    }

    fn resume_grid(
        &mut self,
        output: CompletedRequest<'input, Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.grids.last_mut().expect("active Grid request");
        match output {
            CompletedRequest::GridObservation(completion) => {
                parent.computation.accept_observation(*completion)?
            }
            CompletedRequest::Context(result) => {
                let child = parent.child.take().expect("pending Grid child request");
                parent.computation.accept_context(self, child, *result)?;
            }
            output => {
                let child = parent.child.take().expect("pending Grid child request");
                parent
                    .computation
                    .accept_child(self, child, output.layout())?;
            }
        }
        Ok(Request::AdvanceGrid)
    }

    fn begin_grid(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        match begin_grid_computation(self, node, input)? {
            ControlFlow::Break(output) => {
                self.finish_container_request(node, input, completion, projection, output)
            }
            ControlFlow::Continue(mut computation) => match computation.advance(self)? {
                ControlFlow::Break(result) => self.finish_container_request(
                    node,
                    input,
                    completion,
                    projection,
                    result.into_output(),
                ),
                ControlFlow::Continue(child) => {
                    pending
                        .grids
                        .try_reserve(1)
                        .map_err(|_| reserve_error(node))?;
                    let mut pending_child = None;
                    let request =
                        Self::schedule_grid_request(child, &mut pending_child, &mut pending.order)?;
                    pending.grids.push(GridRequest {
                        computation,
                        completion: GridCompletion::Child {
                            completion,
                            projection,
                        },
                        child: pending_child,
                    });
                    Ok(request)
                }
            },
        }
    }

    /// Begins a subgrid item's contextual Grid computation on the request
    /// stack. It takes no child admission, as its synchronous form took none.
    fn begin_context_grid(
        &mut self,
        request: GridContextRequest<'input, Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let node = request.node();
        let effects_start = self.begin_effect_capture(super::CaptureKind::Request);
        let started = begin_grid_context_computation(self, request)
            .and_then(|mut computation| computation.advance(self).map(|step| (computation, step)));
        let (computation, step) = match started {
            Ok(started) => started,
            Err(error) => {
                // The failed computation leaves no writes to replay.
                self.end_effect_capture(effects_start);
                return Err(error.into());
            }
        };
        match step {
            ControlFlow::Break(result) => {
                let result = result.with_effects(self.end_effect_capture(effects_start));
                Ok(Request::Complete(CompletedRequest::Context(Box::new(
                    result,
                ))))
            }
            ControlFlow::Continue(child) => {
                // The capture stays open for the computation to close; a
                // failure to schedule it closes the capture here.
                if pending.grids.try_reserve(1).is_err() {
                    self.end_effect_capture(effects_start);
                    return Err(reserve_error(node).into());
                }
                let mut pending_child = None;
                let request = match Self::schedule_grid_request(
                    child,
                    &mut pending_child,
                    &mut pending.order,
                ) {
                    Ok(request) => request,
                    Err(error) => {
                        self.end_effect_capture(effects_start);
                        return Err(error);
                    }
                };
                pending.grids.push(GridRequest {
                    computation,
                    completion: GridCompletion::Context { effects_start },
                    child: pending_child,
                });
                Ok(request)
            }
        }
    }

    fn advance_grid(
        &mut self,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let parent = pending.grids.last_mut().expect("active Grid request");
        match parent.computation.advance(self)? {
            ControlFlow::Break(result) => {
                let parent = pending.grids.pop().expect("completed Grid request");
                match parent.completion {
                    GridCompletion::Child {
                        completion,
                        projection,
                    } => {
                        let (node, input) = parent.computation.request();
                        self.finish_container_request(
                            node,
                            input,
                            completion,
                            projection,
                            result.into_output(),
                        )
                    }
                    GridCompletion::Context { effects_start } => {
                        let result = result.with_effects(self.end_effect_capture(effects_start));
                        Ok(Request::Complete(CompletedRequest::Context(Box::new(
                            result,
                        ))))
                    }
                }
            }
            ControlFlow::Continue(child) => {
                Self::schedule_grid_request(child, &mut parent.child, &mut pending.order)
            }
        }
    }

    fn schedule_grid_request(
        child: GridComputationRequest<'input, Tree::Node, Tree::Scalar>,
        pending_child: &mut Option<GridChild<'input, Tree::Node, Tree::Scalar>>,
        order: &mut Vec<PendingRequest>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        match child {
            GridComputationRequest::Child(mut child) => {
                let (node, input) = child.request();
                order.try_reserve(1).map_err(|_| reserve_error(node))?;
                order.push(PendingRequest::Grid);
                let context = child.take_context_request();
                *pending_child = Some(child);
                Ok(match context {
                    Some(request) => Request::ContextGrid(Box::new(request)),
                    None => Request::Child {
                        node,
                        input,
                        projection: OutputProjection::Layout,
                    },
                })
            }
            GridComputationRequest::Observation(source) => {
                order
                    .try_reserve(1)
                    .map_err(|_| reserve_error(source.node()))?;
                order.push(PendingRequest::Grid);
                *pending_child = None;
                Ok(Request::GridObservation(source))
            }
        }
    }

    fn finish_container_request(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        self.finish_uncached_layout(node, input, output)?;
        self.complete_child_request(node, completion, projection, output)
    }

    fn complete_child_request(
        &mut self,
        node: Tree::Node,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let ChildCompletion { cache, capture } = completion;
        if let Some(cache) = cache {
            cache.store(self, output)?;
        }
        self.complete_memo(node, capture, output)?;
        Ok(projection.complete(output))
    }

    fn advance_hidden(
        &mut self,
        mut children: HiddenChildren<Tree::Node>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        while let Some((node, input)) = children.next_box(self)? {
            match self.admit_child(node, input)? {
                ChildAdmission::Hidden(request) => {
                    if children.has_remaining() {
                        pending
                            .keep_hidden(children, projection)
                            .map_err(|_| reserve_error(node))?;
                    }
                    return Ok(Request::Hidden(request, projection));
                }
                ChildAdmission::Ready(_) => {}
                ChildAdmission::Uncached { input, cache, memo } => {
                    let capture = self.begin_memo(node, memo)?;
                    let output = self.complete_admitted_child(node, input, cache)?;
                    self.complete_memo(node, capture, output)?;
                }
            }
        }
        Ok(projection.complete(ComputeOutputOf::HIDDEN))
    }

    fn begin_axis(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        axis: PhysicalAxis,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        let style = self.tree.node_input(node);
        let own = FlowAxes::new(style.writing_mode, style.direction);
        if !self.tree.has_leaf_measurement(node) {
            if own.inline_axis() == axis && style.display.inner_display() == Display::Block {
                // This scalar demand has a separate completion from a full
                // child request and no host cache key.
                let demand = BlockAxisDemand::begin(self, node, input, axis)?;
                return self.advance_block_axis(demand, pending);
            }
            if self.bound.is_some() && self.child_count(node) != 0 {
                return Err(crate::block::intrinsic::profile_error(node).into());
            }
        }
        Ok(Request::Child {
            node,
            input,
            projection: OutputProjection::Axis(axis),
        })
    }

    fn advance_block_axis(
        &mut self,
        mut parent: BlockAxisDemand<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> StopResult<Tree, SessionRequest<'input, Tree>> {
        if let Some(child) = parent.next_child(self)? {
            let node = child.node;
            let input = child.input;
            let axis = parent.axis();
            pending
                .keep_block_axis(parent, child)
                .map_err(|_| reserve_error(node))?;
            return Ok(Request::Axis { node, input, axis });
        }
        Ok(Request::Complete(CompletedRequest::Axis(parent.finish())))
    }
}
