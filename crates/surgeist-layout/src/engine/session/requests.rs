use std::{collections::TryReserveError, ops::ControlFlow};

use super::memo::Capture;
use super::{ChildAdmission, ChildCacheRequest, ComputeSession, FloatChildTrial, SessionResult};
use crate::block::intrinsic::{BlockAxisDemand, InlineContentChild};
use crate::block::{
    BlockChildReceipt, BlockChildRequest, BlockComputation, BlockResult, PendingBlockChild,
    begin_block_computation,
};
use crate::engine::block_scope::ordinary;
use crate::engine::hidden::{HiddenChildren, HiddenRequest};
use crate::flex::{FlexChild, FlexComputation, begin_flex_computation};
use crate::grid::{GridChild, GridContinuation, begin_grid_computation};
use crate::{
    Compute, ComputeInputOf, ComputeOutputOf, Display, FlowAxes, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutOperation, LayoutScalar, LayoutTree, PhysicalAxis, Traverse,
};

enum Request<Node, S: LayoutScalar> {
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
    AdvanceBlock,
    AdvanceFlex,
    AdvanceGrid,
    Qualify(crate::block::entry::EntryQuery<Node, S>),
    Entry(Box<crate::block::entry::FramedEntry<Node, S>>),
    FloatTrial(FloatTrialRequest<Node, S>),
    Complete(CompletedRequest<Node, S>),
}

struct FloatTrialRequest<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    trial: FloatChildTrial,
}

enum RequestProgress<'input, Node, S: LayoutScalar> {
    Complete(CompletedRequest<Node, S>),
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
    fn complete<Node, S: LayoutScalar>(self, output: ComputeOutputOf<S>) -> Request<Node, S> {
        let output = match self {
            Self::Layout => CompletedRequest::Layout(output),
            Self::Axis(PhysicalAxis::Horizontal) => CompletedRequest::Axis(output.size.width),
            Self::Axis(PhysicalAxis::Vertical) => CompletedRequest::Axis(output.size.height),
        };
        Request::Complete(output)
    }
}

enum CompletedRequest<Node, S: LayoutScalar> {
    Entry(Box<crate::block::entry::EntryObservation<Node, S>>),
    Layout(ComputeOutputOf<S>),
    Axis(S),
}

impl<Node, S: LayoutScalar> CompletedRequest<Node, S> {
    fn layout(self) -> ComputeOutputOf<S> {
        match self {
            Self::Layout(output) => output,
            Self::Entry(_) | Self::Axis(_) => {
                unreachable!("a layout request completes with layout geometry")
            }
        }
    }

    fn axis(self) -> S {
        match self {
            Self::Axis(output) => output,
            Self::Entry(_) | Self::Layout(_) => {
                unreachable!("an axis request completes with a scalar")
            }
        }
    }
}

enum PendingRequest {
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
}

struct FlexRequest<'input, Node, S: LayoutScalar> {
    computation: FlexComputation<'input, Node, S>,
    completion: ChildCompletion<Node, S>,
    projection: OutputProjection,
    child: Option<FlexChild<'input, Node, S>>,
}

struct GridRequest<'input, Node, S: LayoutScalar> {
    computation: GridContinuation<'input, Node, S>,
    completion: ChildCompletion<Node, S>,
    projection: OutputProjection,
    child: Option<GridChild<'input, Node, S>>,
}

struct PendingRequests<'input, Node, S: LayoutScalar> {
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

type SessionRequest<Tree> = Request<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;
type SessionPending<'input, Tree> =
    PendingRequests<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;
type StagedProgress<Tree> = ControlFlow<
    CompletedRequest<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    SessionRequest<Tree>,
>;
type EntryProgress<'input, Tree> = ControlFlow<
    RequestProgress<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    SessionRequest<Tree>,
>;
struct ChildCompletion<Node, S: LayoutScalar> {
    cache: Option<ChildCacheRequest<Node, S>>,
    capture: Option<Capture<S>>,
}
type SessionBlock<'input, Tree> =
    BlockRequest<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

enum StagedDestination<'input, Node, S: LayoutScalar> {
    Qualification,
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
        match self.drive_requests(Request::Qualify(query))? {
            CompletedRequest::Entry(observation) => Ok(*observation),
            _ => unreachable!("entry operation returns entry facts"),
        }
    }

    pub(super) fn drive_child(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, ComputeOutputOf<Tree::Scalar>> {
        self.drive_requests(Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        })
        .map(CompletedRequest::layout)
    }

    pub(super) fn drive_hidden(
        &mut self,
        request: HiddenRequest<Tree::Node>,
    ) -> SessionResult<Tree, ComputeOutputOf<Tree::Scalar>> {
        self.drive_requests(Request::Hidden(request, OutputProjection::Layout))
            .map(CompletedRequest::layout)
    }

    pub(super) fn drive_axis(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        axis: PhysicalAxis,
    ) -> SessionResult<Tree, Tree::Scalar> {
        self.drive_requests(Request::Axis { node, input, axis })
            .map(CompletedRequest::axis)
    }

    fn drive_requests(
        &mut self,
        request: SessionRequest<Tree>,
    ) -> SessionResult<Tree, CompletedRequest<Tree::Node, Tree::Scalar>> {
        let mut pending = PendingRequests::new();
        let mut staged = Vec::new();
        let result = self.drive_staged_requests(request, &mut pending, &mut staged);
        result.map_err(|mut error| {
            while let Some(mut trial) = staged.pop() {
                error = trial.session.unwind_blocks(trial.pending.blocks, error);
            }
            self.unwind_blocks(pending.blocks, error)
        })
    }

    fn drive_staged_requests(
        &mut self,
        mut request: SessionRequest<Tree>,
        pending: &mut SessionPending<'input, Tree>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> SessionResult<Tree, CompletedRequest<Tree::Node, Tree::Scalar>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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

    fn complete_staged(
        &mut self,
        output: CompletedRequest<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
        staged: &mut Vec<StagedTrial<'input, Tree>>,
    ) -> SessionResult<Tree, StagedProgress<Tree>> {
        let Some(finished) = staged.pop() else {
            return Ok(ControlFlow::Break(output));
        };
        let (parent, pending) = if let Some(active) = staged.last_mut() {
            (&mut active.session, &mut active.pending)
        } else {
            (&mut *self, &mut *pending)
        };
        let request = match finished.destination {
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
        request: SessionRequest<Tree>,
    ) -> SessionResult<Tree, EntryProgress<'input, Tree>> {
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

    fn unwind_blocks(
        &mut self,
        blocks: Vec<SessionBlock<'input, Tree>>,
        mut error: crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError>,
    ) -> crate::LayoutErrorOf<Tree::Node, Tree::Scalar, Tree::MeasureError> {
        for frame in blocks.into_iter().rev() {
            if let Some((node, input)) = frame
                .child
                .as_ref()
                .and_then(PendingBlockChild::shared_request)
            {
                let _ = self.end_child_float_context(node, input);
            }
            error = ordinary(frame.computation.finish(self, Err(error.into())))
                .expect_err("failed child leaves its Block owner failed");
        }
        error
    }

    fn run_requests(
        &mut self,
        mut request: SessionRequest<Tree>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, RequestProgress<'input, Tree::Node, Tree::Scalar>> {
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
                            Some(Display::Block) if self.block_scopes().is_none() => {
                                self.begin_block(node, input, completion, projection, pending)?
                            }
                            Some(Display::Flex) => {
                                self.begin_flex(node, input, completion, projection, pending)?
                            }
                            Some(Display::Grid | Display::GridLanes) => {
                                self.begin_grid(node, input, completion, projection, pending)?
                            }
                            _ => {
                                let output = ordinary(
                                    self.compute_uncached_layout_body(node, input, initial),
                                )?;
                                self.complete_child_request(node, completion, projection, output)?
                            }
                        }
                    }
                },
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
                Request::Qualify(query) => return Ok(RequestProgress::Qualification(query)),
                entry @ Request::Entry(_) => match self.advance_entry_request(entry)? {
                    ControlFlow::Break(progress) => return Ok(progress),
                    ControlFlow::Continue(request) => request,
                },
                Request::FloatTrial(trial) => return Ok(RequestProgress::FloatTrial(trial)),
                Request::Complete(output) => match pending.order.pop() {
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
                            CompletedRequest::Axis(_) => {
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
                    Some(PendingRequest::Grid) => {
                        let parent = pending.grids.last_mut().expect("active Grid request");
                        let child = parent.child.take().expect("pending Grid child request");
                        parent
                            .computation
                            .accept_child(self, child, output.layout())?;
                        Request::AdvanceGrid
                    }
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        match ordinary(begin_block_computation(self, node, input))? {
            ControlFlow::Break(output) => {
                self.finish_container_request(node, input, completion, projection, output)
            }
            ControlFlow::Continue(computation) => {
                if pending.blocks.try_reserve(1).is_err() {
                    return ordinary(computation.finish(self, Err(reserve_error(node).into())))
                        .map(|output| projection.complete(output));
                }
                pending.blocks.push(BlockRequest {
                    computation,
                    completion,
                    projection,
                    child: None,
                });
                Ok(Request::AdvanceBlock)
            }
        }
    }

    fn resume_block(
        &mut self,
        output: BlockChildReceipt<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let parent = pending.blocks.last_mut().expect("active Block request");
        let child = parent.child.take().expect("pending Block child request");
        ordinary(parent.computation.accept_child(self, child, output))?;
        Ok(Request::AdvanceBlock)
    }

    fn advance_block(
        &mut self,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let parent = pending.blocks.last_mut().expect("active Block request");
        match ordinary(parent.computation.advance(self))? {
            ControlFlow::Break(result) => self.complete_block(result, pending),
            ControlFlow::Continue(child) => {
                let (request, child) = child.into_request();
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

    fn complete_block(
        &mut self,
        result: BlockResult<Tree::Node, Tree::Scalar>,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let parent = pending.blocks.pop().expect("completed Block request");
        let (node, input) = parent.computation.request();
        let output = ordinary(parent.computation.finish(self, Ok(result)))?;
        self.finish_container_request(node, input, parent.completion, parent.projection, output)
    }

    fn begin_flex(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let (node, input) = child.request();
        order.try_reserve(1).map_err(|_| reserve_error(node))?;
        order.push(PendingRequest::Flex);
        Ok(Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        })
    }

    fn begin_grid(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        match begin_grid_computation(self, node, input)? {
            ControlFlow::Break(output) => {
                self.finish_container_request(node, input, completion, projection, output)
            }
            ControlFlow::Continue(mut computation) => match computation.advance(self)? {
                ControlFlow::Break(output) => {
                    self.finish_container_request(node, input, completion, projection, output)
                }
                ControlFlow::Continue(child) => {
                    pending
                        .grids
                        .try_reserve(1)
                        .map_err(|_| reserve_error(node))?;
                    let request = Self::schedule_grid_child(&child, &mut pending.order)?;
                    pending.grids.push(GridRequest {
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

    fn advance_grid(
        &mut self,
        pending: &mut SessionPending<'input, Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let parent = pending.grids.last_mut().expect("active Grid request");
        match parent.computation.advance(self)? {
            ControlFlow::Break(output) => {
                let parent = pending.grids.pop().expect("completed Grid request");
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
                let request = Self::schedule_grid_child(&child, &mut pending.order)?;
                parent.child = Some(child);
                Ok(request)
            }
        }
    }

    fn schedule_grid_child(
        child: &GridChild<'input, Tree::Node, Tree::Scalar>,
        order: &mut Vec<PendingRequest>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let (node, input) = child.request();
        order.try_reserve(1).map_err(|_| reserve_error(node))?;
        order.push(PendingRequest::Grid);
        Ok(Request::Child {
            node,
            input,
            projection: OutputProjection::Layout,
        })
    }

    fn finish_container_request(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        self.finish_uncached_layout(node, input, output)?;
        self.complete_child_request(node, completion, projection, output)
    }

    fn complete_child_request(
        &mut self,
        node: Tree::Node,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
                return Err(crate::block::intrinsic::profile_error(node));
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
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
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
