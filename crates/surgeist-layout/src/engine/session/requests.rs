use std::collections::TryReserveError;

use super::{ChildAdmission, ComputeSession, SessionResult};
use crate::block::intrinsic::{BlockAxisDemand, InlineContentChild};
use crate::engine::hidden::{HiddenChildren, HiddenRequest};
use crate::{
    ComputeInputOf, ComputeOutputOf, Display, FlowAxes, LayoutErrorKindOf, LayoutErrorOf,
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
}

struct PendingRequests<Node, S: LayoutScalar> {
    order: Vec<PendingRequest>,
    hidden: Vec<(HiddenChildren<Node>, OutputProjection)>,
    block_axes: Vec<(BlockAxisDemand<Node, S>, InlineContentChild<Node, S>)>,
}

impl<Node, S: LayoutScalar> PendingRequests<Node, S> {
    fn new() -> Self {
        Self {
            order: Vec::new(),
            hidden: Vec::new(),
            block_axes: Vec::new(),
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
type SessionPending<Tree> = PendingRequests<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>;

fn reserve_error<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::Allocation,
    )
}

impl<Tree: LayoutTree> ComputeSession<'_, Tree> {
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
        mut request: SessionRequest<Tree>,
    ) -> SessionResult<Tree, CompletedRequest<Tree::Scalar>> {
        let mut pending = PendingRequests::new();
        loop {
            request = match request {
                Request::Child {
                    node,
                    input,
                    projection,
                } => match self.admit_child(node, input)? {
                    ChildAdmission::Ready(output) => projection.complete(output),
                    ChildAdmission::Hidden(request) => Request::Hidden(request, projection),
                    ChildAdmission::Uncached { input, cache } => {
                        let output = self.complete_admitted_child(node, input, cache)?;
                        projection.complete(output)
                    }
                },
                Request::Hidden(request, projection) => {
                    let children = HiddenChildren::begin(self, request)?;
                    self.advance_hidden(children, projection, &mut pending)?
                }
                Request::HiddenSiblings(children, projection) => {
                    self.advance_hidden(children, projection, &mut pending)?
                }
                Request::Axis { node, input, axis } => {
                    self.begin_axis(node, input, axis, &mut pending)?
                }
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
                        self.advance_block_axis(parent, &mut pending)?
                    }
                },
            };
        }
    }

    fn advance_hidden(
        &mut self,
        mut children: HiddenChildren<Tree::Node>,
        projection: OutputProjection,
        pending: &mut SessionPending<Tree>,
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
                ChildAdmission::Uncached { input, cache } => {
                    self.complete_admitted_child(node, input, cache)?;
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
        pending: &mut SessionPending<Tree>,
    ) -> SessionResult<Tree, SessionRequest<Tree>> {
        let style = self.tree.node_input(node);
        let own = FlowAxes::new(style.writing_mode, style.direction);
        if !self.tree.has_leaf_measurement(node) {
            if own.inline_axis() == axis && style.display.inner_display() == Display::Block {
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
        pending: &mut SessionPending<Tree>,
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
