use std::{collections::TryReserveError, ops::ControlFlow};

use super::memo::Capture;
use super::{ChildAdmission, ChildCacheRequest, ComputeSession, SessionResult};
use crate::block::intrinsic::{BlockAxisDemand, InlineContentChild};
use crate::block::{BlockComputation, BlockResult, PendingBlockChild, begin_block_computation};
use crate::engine::block_scope::ordinary;
use crate::engine::hidden::{HiddenChildren, HiddenRequest};
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
    Complete(CompletedRequest<S>),
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

enum CompletedRequest<S: LayoutScalar> {
    Layout(ComputeOutputOf<S>),
    Axis(S),
}

impl<S: LayoutScalar> CompletedRequest<S> {
    fn layout(self) -> ComputeOutputOf<S> {
        match self {
            Self::Layout(output) => output,
            Self::Axis(_) => unreachable!("a layout request completes with layout geometry"),
        }
    }

    fn axis(self) -> S {
        match self {
            Self::Axis(output) => output,
            Self::Layout(_) => unreachable!("an axis request completes with a scalar"),
        }
    }
}

enum PendingRequest {
    HiddenSiblings,
    BlockAxis,
    Block,
}

struct BlockRequest<'input, Node, S: LayoutScalar> {
    computation: BlockComputation<'input, Node, S>,
    completion: ChildCompletion<Node, S>,
    projection: OutputProjection,
    child: Option<PendingBlockChild<'input, Node, S>>,
}

struct PendingRequests<'input, Node, S: LayoutScalar> {
    order: Vec<PendingRequest>,
    hidden: Vec<(HiddenChildren<Node>, OutputProjection)>,
    block_axes: Vec<(BlockAxisDemand<Node, S>, InlineContentChild<Node, S>)>,
    blocks: Vec<BlockRequest<'input, Node, S>>,
}

impl<Node, S: LayoutScalar> PendingRequests<'_, Node, S> {
    fn new() -> Self {
        Self {
            order: Vec::new(),
            hidden: Vec::new(),
            block_axes: Vec::new(),
            blocks: Vec::new(),
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
struct ChildCompletion<Node, S: LayoutScalar> {
    cache: Option<ChildCacheRequest<Node, S>>,
    capture: Option<Capture<S>>,
}
type SessionBlock<'input, Tree> =
    BlockRequest<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

fn reserve_error<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::Allocation,
    )
}

impl<'input, Tree: LayoutTree> ComputeSession<'input, Tree> {
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
    ) -> SessionResult<Tree, CompletedRequest<Tree::Scalar>> {
        let mut pending = PendingRequests::new();
        self.run_requests(request, &mut pending)
            .map_err(|error| self.unwind_blocks(pending.blocks, error))
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
    ) -> SessionResult<Tree, CompletedRequest<Tree::Scalar>> {
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
                        if self.block_scopes().is_none()
                            && !initial.leaf
                            && initial.style.display.inner_display() == Display::Block
                        {
                            self.begin_block(
                                node,
                                input,
                                ChildCompletion { cache, capture },
                                projection,
                                pending,
                            )?
                        } else {
                            let output =
                                ordinary(self.compute_uncached_layout_body(node, input, initial))?;
                            if let Some(cache) = cache {
                                cache.store(self, output)?;
                            }
                            self.complete_memo(node, capture, output)?;
                            projection.complete(output)
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
                Request::Complete(output) => match pending.order.pop() {
                    None => return Ok(output),
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
                    Some(PendingRequest::Block) => self.resume_block(output.layout(), pending)?,
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
                self.finish_request_block(node, input, completion, projection, output)
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
        output: ComputeOutputOf<Tree::Scalar>,
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
                debug_assert!(
                    request.entry.is_none(),
                    "ordinary request has no scope entry"
                );
                parent.child = Some(child);
                pending
                    .order
                    .try_reserve(1)
                    .map_err(|_| reserve_error(request.node))?;
                pending.order.push(PendingRequest::Block);
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
        self.finish_request_block(node, input, parent.completion, parent.projection, output)
    }

    fn finish_request_block(
        &mut self,
        node: Tree::Node,
        input: ComputeInputOf<Tree::Scalar>,
        completion: ChildCompletion<Tree::Node, Tree::Scalar>,
        projection: OutputProjection,
        output: ComputeOutputOf<Tree::Scalar>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        self.finish_uncached_layout(node, input, output)?;
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
