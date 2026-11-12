use crate::engine::block_scope::{ComputeResult, ordinary};
use std::{collections::BTreeMap, ops::ControlFlow};

use super::inline::{
    AtomicInlineBoxParticipant, ForcedLineBreakControlOf, InlineBoundaryControlOf, InlineFlowOf,
    InlineParticipantKindOf, InlineParticipantProjection, LogicalLineBandQueryResultOf,
    MixedInlineParticipantOf, MixedInlineRunInputOf,
    layout_mixed_inline_run_with_formatted_observation,
};
use super::value::{ResolvedLengthAutoOf, UnresolvedLengthReason};
use super::{
    BoxSizing, Clear, CollapsibleMarginOf, Compute, ComputeInputOf, ComputeOutputOf,
    ContainingLayoutContext, Direction, Edges, InlineBoundaryInputOf, InlineFragmentOutputOf,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutOperation,
    LayoutResultOf, LayoutScalar, LengthAutoOf, LineBreakInputOf, NodeOutputOf, Overflow,
    ParentFormattingContext, PhysicalBlockMarginCollapseOf, Point, Position, RequestedAxis,
    RunMode, Size, SizingAlgorithm, SizingMode, TextAlign, TextAlignLast, TextJustification,
    Traverse, VerticalAlignOf,
};
use crate::engine::containment::{
    compute_empty_box, contained_outer_size, empty_sizing_input, size_containment,
};
use crate::error::{
    AtomicInlineParticipationRoleError, layout_child_geometry_error, layout_own_geometry_error,
};
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, OptionalSizeExt,
    OptionalSizeMaxExt, resolve_containing_padding_border,
};
use crate::output::MeasuredInsetBasisOf;
use crate::scroll::{CanonicalScrollBoxSourceOf, UsedOverflow, canonical_scroll_box_from_source};
use crate::sizing::resolve::{EdgesResultExt, SizeResultExt};

#[cfg(test)]
use super::{LengthOf, NodeInputOf, WritingMode};

mod absolute;
pub(crate) mod baseline_frame;
pub(crate) mod carrier;
pub(crate) mod content;
pub(crate) mod content_insets;
pub(crate) mod entry;
pub(crate) mod float_context;
mod float_text;
mod floats;
pub(crate) mod flow_root;
pub(crate) mod fragment_margins;
pub(crate) mod fragment_minimum;
mod in_flow;
mod inline_run;
mod input;
pub(crate) mod intrinsic;
mod relative;
pub(crate) mod scope;
mod scroll;
mod sizing;

pub(crate) use absolute::StaticPositionOf;
use absolute::{AbsoluteChild, AbsoluteChildren};
pub(crate) use input::BlockChildProjection;
use input::{
    BlockContainerProjection, automatic_block_depends_on_ratio, block_container_projection,
    normal_block_auto_inline, normal_block_inline_ratio, with_block_scroll_projections,
};
use scroll::{finish_scroll_geometry, prepare_scroll_contributions};
use sizing::{
    BlockOptionalSizeSubExt, maximum_size, minimum_size, preferred_size, resolve_auto_optional,
    resolve_length_or_zero, transfer_block_ratio_inline_limits,
};

#[cfg(test)]
pub(crate) use in_flow::resolve_logical_in_flow_margin;
use in_flow::{
    InFlowPassContext, InFlowStep, InFlowTraversal, normal_flow_children_need_physical_sizing_work,
};

#[cfg(test)]
pub(crate) use floats::FloatLedgerSide;
pub(crate) use floats::{FloatExclusions, InheritedFloatExclusions};
use floats::{FloatIntrinsics, PendingFloat, layout_floats};

pub(crate) fn measure_definite_inline_atomic<'input, Tree, M>(
    tree: &mut Tree,
    container: Tree::Node,
    atomic: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, Size<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = block_container_projection::<Tree, M>(tree, container);
    let constants =
        Constants::from_projection::<Tree, M>(tree, container, &style, input, Size::NONE)?;
    let logical_size = constants.logical_node_inner_size();
    let (Some(outer_inline), Some(inner_block)) =
        (constants.definite_inline_outer(), logical_size.block)
    else {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject {
                container,
                subject: atomic,
            },
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
        ));
    };
    let inner_inline = constants.accepted_inline_inner(container, outer_inline)?;
    crate::layout_math::checked_available_space(inner_block)
        .map_err(|error| crate::error::sizing_resolution_error(container, error.into()))?;
    let constants = constants.with_logical_node_inner_size(
        LogicalSizeOf::new(Some(inner_inline), Some(inner_block)),
        false,
    );
    let available_inline = crate::layout_math::with_available_extent(
        constants
            .flow_axes
            .logical_size(constants.sizing_request.axes())
            .inline,
        Some(inner_inline),
    )
    .map_err(|error| crate::error::sizing_resolution_error(container, error.into()))?;
    let request = inline_run::inline_atomic_child_request(
        &constants,
        RunMode::ComputeSize,
        constants.sizing_request,
        available_inline,
    );
    // Admission proves an empty, unmeasured InlineBlock. Its production box
    // owner needs no parent traversal or host cache to compute this size.
    let output = compute_block(tree, atomic, request)?;
    for value in [output.size.width, output.size.height] {
        crate::layout_math::checked_available_space(value)
            .map_err(|error| crate::error::sizing_resolution_error(atomic, error.into()))?;
    }
    Ok(output.size)
}

pub(crate) fn compute_block<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    ordinary(compute_block_scoped(tree, node, input))
}

pub(crate) fn compute_block_scoped<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> ComputeResult<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut computation = match begin_block_computation(tree, node, input)? {
        ControlFlow::Break(output) => return Ok(output),
        ControlFlow::Continue(computation) => computation,
    };
    let result = (|| loop {
        match computation.advance(tree)? {
            ControlFlow::Break(output) => return Ok(output),
            ControlFlow::Continue(child) => {
                let (pending, output) = child.compute(tree)?;
                computation.accept_child(tree, pending, output)?;
            }
        }
    })();
    computation.finish(tree, result)
}

pub(crate) struct BlockResult<Node, S: LayoutScalar> {
    output: ComputeOutputOf<S>,
    used: ComputeInputOf<S>,
    floats: Option<(FloatExclusions<S, Node>, usize)>,
    remaining_cut: Option<entry::canonical::Separation<Node, S>>,
}

pub(crate) struct BlockComputation<'input, Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    pass_input: ComputeInputOf<S>,
    contained_size: Size<Option<S>>,
    scrollbar_width: S,
    incoming: Option<float_context::Incoming<Node, S>>,
    anonymous: bool,
    formatted: bool,
    pass: BlockPass<'input, Node, S>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> BlockComputation<'input, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        (self.node, self.input)
    }

    pub(crate) fn advance<Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, BlockPassStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            let output = if self.pass.has_work() {
                let inherited = self
                    .incoming
                    .as_ref()
                    .map(|incoming| incoming.frame.as_ref());
                match self.pass.advance(tree, inherited)? {
                    ControlFlow::Break(output) => output,
                    ControlFlow::Continue(child) => return Ok(ControlFlow::Continue(child)),
                }
            } else if let Some(output) = self.start_pass(tree)? {
                output
            } else {
                continue;
            };
            if !self.retry_scrollbars(&output)? {
                return Ok(ControlFlow::Break(output));
            }
        }
    }

    fn start_pass<Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, Option<BlockResult<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let inherited = self
            .incoming
            .as_ref()
            .map(|incoming| incoming.frame.as_ref());
        match begin_block_pass(
            tree,
            self.node,
            self.pass_input,
            inherited,
            self.contained_size,
        )? {
            ControlFlow::Break(output) => Ok(Some(output)),
            ControlFlow::Continue(pass) => {
                self.pass = pass;
                Ok(None)
            }
        }
    }

    pub(crate) fn accept_child<Tree, M>(
        &mut self,
        tree: &mut Tree,
        pending: PendingBlockChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> ComputeResult<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.pass.accept_child(tree, pending, output)
    }

    fn retry_scrollbars<M>(
        &mut self,
        result: &BlockResult<Node, S>,
    ) -> ComputeResult<Node, bool, S, M> {
        if !self.input.run_mode().is_perform_layout() {
            return Ok(false);
        }
        let Some(geometry) = result.output.scroll_geometry else {
            return Ok(false);
        };
        let current = self.pass_input.settled_auto_scrollbars();
        let next = current.transition(geometry);
        if next == current
            || self.scrollbar_width == S::ZERO
            || !crate::scroll::settled_auto_scrollbars_change_available_geometry(geometry, next)
                .map_err(|error| {
                    layout_own_geometry_error(self.node, self.input.run_mode(), error)
                })?
        {
            return Ok(false);
        }
        self.pass_input = self.pass_input.with_settled_auto_scrollbars(next);
        Ok(true)
    }

    pub(crate) fn finish<Tree, M>(
        self,
        tree: &mut Tree,
        result: ComputeResult<Node, BlockResult<Node, S>, S, M>,
    ) -> ComputeResult<Node, ComputeOutputOf<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            node,
            input,
            incoming,
            anonymous,
            formatted,
            pass,
            ..
        } = self;
        drop(pass);
        let formatted_result = if formatted && let Some(text) = tree.text_computation() {
            text.formatted_end_block(node, input, anonymous, result.is_ok())
                .map_err(|error| inline_run::text_run_error(node, error.into()))
        } else {
            Ok(())
        };
        if let Some(groups) = tree.group_computation() {
            groups.end(result.is_ok());
        }
        // Preserve the body failure ahead of completion bookkeeping.
        let result = result?;
        formatted_result?;
        if incoming.is_none()
            && float_context::Role::for_input(tree.node_input(node), input)
                == float_context::Role::Shared
            && result
                .floats
                .as_ref()
                .is_some_and(|(ledger, _)| !ledger.is_empty())
        {
            return Err(float_context::missing(node).into());
        }
        if let Some((ledger, inherited)) = result.floats
            && let Some(state) = tree.block_float_state()
        {
            state.record(float_context::Completed {
                node,
                request: incoming.as_ref().map_or(input, |incoming| incoming.request),
                used: result.used,
                output: result.output,
                ledger,
                inherited,
                frame: incoming.map(|incoming| incoming.frame),
                remaining_cut: result.remaining_cut,
            });
        }
        Ok(result.output)
    }
}

/// Actual empty-box sizing supplied by the containment owner. Scalar demand
/// and canonical preparation consume the same authority as Normal Block layout.
struct ContainmentSizing<S: LayoutScalar> {
    kind: crate::SizeContainment,
    empty: ComputeOutputOf<S>,
    outer: Size<Option<S>>,
}

fn containment_sizing<'input, Node, S, M>(
    node: Node,
    style: &'input crate::NodeInputOf<S>,
    input: ComputeInputOf<S>,
    minimum: Option<fragment_minimum::Override<S>>,
) -> LayoutResultOf<Node, Option<ContainmentSizing<S>>, S, M>
where
    Node: Copy + Eq + 'input,
    S: LayoutScalar,
{
    let kind = size_containment(style);
    if kind == crate::SizeContainment::None {
        return Ok(None);
    }
    let empty = compute_empty_box(node, style, |probe| {
        probe.install_fragment_minimum(minimum);
        ordinary(compute_block_inner(
            probe,
            node,
            empty_sizing_input(input),
            None,
            Size::NONE,
        ))
    })?
    .output;
    let axes = crate::FlowAxes::new(style.writing_mode, style.direction);
    Ok(Some(ContainmentSizing {
        kind,
        outer: contained_outer_size(kind, axes, empty.size),
        empty,
    }))
}

pub(crate) type BlockComputationStart<'input, Tree, M> = ComputeResult<
    <Tree as Traverse>::Node,
    ControlFlow<
        ComputeOutputOf<<Tree as Traverse>::Scalar>,
        BlockComputation<'input, <Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    >,
    <Tree as Traverse>::Scalar,
    M,
>;

pub(crate) fn begin_block_computation<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> BlockComputationStart<'input, Tree, M>
where
    Tree: Compute<'input, M>,
{
    let incoming = tree
        .block_float_state()
        .and_then(|state| state.incoming(node, input));
    if incoming.is_some()
        && float_context::Role::for_input(tree.node_input(node), input)
            != float_context::Role::Shared
    {
        return Err(float_context::missing(node).into());
    }

    let inherited = incoming.as_ref().map(|incoming| incoming.frame.as_ref());
    let group_style = tree.node_input(node);
    let contained_size = if let Some(containment) =
        containment_sizing(node, group_style, input, tree.fragment_minimum(node))?
    {
        let flow_axes = crate::FlowAxes::new(group_style.writing_mode, group_style.direction);
        let shared_normal_flow = float_context::Role::for_input(group_style, input)
            == float_context::Role::Shared
            && flow_axes
                .logical_size(input.sizing_request().axes())
                .inline
                .operation()
                == crate::SizingOperation::Normal;
        if containment.kind == crate::SizeContainment::BothAxes
            && !input.run_mode().is_perform_layout()
            && inherited.is_none()
            && !shared_normal_flow
            && !input.requires_content_baseline()
            && !tree
                .text_computation()
                .is_some_and(|text| text.formatted_active())
        {
            return Ok(ControlFlow::Break(containment.empty));
        }
        containment.outer
    } else {
        Size::NONE
    };
    let anonymous = group_style.block_generation == crate::BlockGeneration::Anonymous;
    if let Some(groups) = tree.group_computation() {
        groups.begin(node, input, group_style);
    }
    let formatted = tree
        .text_computation()
        .map_or(Ok(false), |text| {
            text.formatted_begin_block(node, input, anonymous)
        })
        .map_err(|error| inline_run::text_run_error(node, error.into()))?;
    Ok(ControlFlow::Continue(BlockComputation {
        node,
        input,
        pass_input: input,
        contained_size,
        scrollbar_width: block_container_projection::<Tree, M>(tree, node)
            .scrollbar_width
            .get(),
        incoming,
        anonymous,
        formatted,
        pass: BlockPass {
            active: None,
            positioned: None,
        },
    }))
}

fn compute_block_inner<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
    inherited: Option<&InheritedFloatExclusions<S, <Tree as Traverse>::Node>>,
    contained_size: Size<Option<S>>,
) -> ComputeResult<<Tree as Traverse>::Node, BlockResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let mut pass = match begin_block_pass(tree, node, input, inherited, contained_size)? {
        ControlFlow::Break(output) => return Ok(output),
        ControlFlow::Continue(pass) => pass,
    };
    loop {
        match pass.advance(tree, inherited)? {
            ControlFlow::Break(output) => return Ok(output),
            ControlFlow::Continue(child) => {
                let (pending, output) = child.compute(tree)?;
                pass.accept_child(tree, pending, output)?;
            }
        }
    }
}

type BlockPassStart<'input, Node, S> =
    ControlFlow<BlockResult<Node, S>, BlockPass<'input, Node, S>>;
pub(crate) type BlockPassStep<'input, Node, S> =
    ControlFlow<BlockResult<Node, S>, BlockChild<'input, Node, S>>;

pub(crate) enum BlockChild<'input, Node, S: LayoutScalar> {
    InFlow(in_flow::InFlowChild<'input, Node, S>),
    Positioned(AbsoluteChild<Node, S>),
}

pub(crate) enum PendingBlockChild<'input, Node, S: LayoutScalar> {
    InFlow(Option<in_flow::PendingBox<'input, Node, S>>),
    Positioned(AbsoluteChild<Node, S>),
}

impl<Node: Copy, S: LayoutScalar> PendingBlockChild<'_, Node, S> {
    pub(crate) fn shared_request(&self) -> Option<(Node, ComputeInputOf<S>)> {
        match self {
            Self::InFlow(Some(child)) => child.shared_request(),
            _ => None,
        }
    }
}

type BlockChildOutput<'input, Node, S> = (PendingBlockChild<'input, Node, S>, ComputeOutputOf<S>);

impl<'input, Node: Copy, S: LayoutScalar> BlockChild<'input, Node, S> {
    pub(crate) fn into_request(
        self,
    ) -> (
        scope::ChildRequest<Node, S>,
        PendingBlockChild<'input, Node, S>,
    ) {
        match self {
            Self::InFlow(child) => {
                let (request, pending) = child.into_request();
                (request, PendingBlockChild::InFlow(pending))
            }
            Self::Positioned(child) => (
                scope::ChildRequest {
                    node: child.node,
                    input: child.input,
                    entry: None,
                },
                PendingBlockChild::Positioned(child),
            ),
        }
    }

    fn compute<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, BlockChildOutput<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match self {
            Self::InFlow(child) => {
                let (pending, output) = child.compute(tree)?;
                Ok((PendingBlockChild::InFlow(pending), output))
            }
            Self::Positioned(child) => {
                let output = tree.compute_child(child.node, child.input)?;
                Ok((PendingBlockChild::Positioned(child), output))
            }
        }
    }
}

fn begin_block_pass<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
    inherited: Option<&InheritedFloatExclusions<S, <Tree as Traverse>::Node>>,
    contained_size: Size<Option<S>>,
) -> ComputeResult<<Tree as Traverse>::Node, BlockPassStart<'input, Tree::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let float_role = float_context::Role::for_input(tree.node_input(node), input);
    let style = block_container_projection::<Tree, M>(tree, node);
    let constants =
        Constants::from_projection::<Tree, M>(tree, node, &style, input, contained_size)?;
    let children = tree.children(node).collect::<Vec<_>>();
    let observe_formatted = tree
        .text_computation()
        .is_some_and(|text| text.formatted_active());

    let semantic_work = tree
        .text_computation()
        .is_some_and(|text| text.has_semantic(node));
    if contained_size == Size::NONE
        && children.is_empty()
        && inherited.is_none()
        && !semantic_work
        && !observe_formatted
        && input.run_mode() == RunMode::ComputeSize
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(ControlFlow::Break(BlockResult {
            output: block_baseline_output(
                node,
                &style,
                &constants,
                ComputeOutputOf::<S>::from_outer_size(Size::new(width, height)),
                crate::NamedBaselinesOf::absent(),
            )?,
            used: input,
            floats: None,
            remaining_cut: None,
        }));
    }
    if contained_size == Size::NONE
        && input.run_mode() == RunMode::ComputeSize
        && inherited.is_none()
        && !semantic_work
        && !observe_formatted
        && input.named_baseline_demand() == crate::output::NamedBaselineDemand::None
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
        && !normal_flow_children_need_physical_sizing_work(tree, &children)
    {
        return Ok(ControlFlow::Break(BlockResult {
            output: block_baseline_output(
                node,
                &style,
                &constants,
                ComputeOutputOf::<S>::from_outer_size(Size::new(width, height)),
                crate::NamedBaselinesOf::unknown(),
            )?,
            used: input,
            floats: None,
            remaining_cut: None,
        }));
    }
    let normal_block_fill = intrinsic::normal_fill(&style, input, tree.node_input(node).float);
    let extent = intrinsic::begin_inline::<Tree, M>(tree, node, &style, &constants, input);
    let flow = BlockFlow {
        node,
        input,
        style,
        children,
        constants,
        float_role,
        contained_size,
        semantic_work,
        normal_block_fill,
        normal_preview: None,
    };
    let active = flow.begin_inline_or_normal(tree, inherited, extent)?;
    Ok(ControlFlow::Continue(BlockPass {
        active: Some(active),
        positioned: None,
    }))
}

struct BlockPass<'input, Node, S: LayoutScalar> {
    active: Option<ActiveBlockPass<'input, Node, S>>,
    positioned: Option<PositionedFlow<Node, S>>,
}

struct ActiveBlockPass<'input, Node, S: LayoutScalar> {
    flow: BlockFlow<'input, Node, S>,
    constants: Constants<S>,
    in_flow: Option<InFlowPass<Node, S>>,
}

struct InFlowPass<Node, S: LayoutScalar> {
    traversal: InFlowTraversal<Node, S>,
    phase: BlockPassPhase<S>,
}

enum BlockPassPhase<S: LayoutScalar> {
    Inline(intrinsic::InlineMeasurement<S>),
    Normal(BlockFlowSizing<S>),
    Final {
        provisional: LogicalSizeOf<S>,
        ratio_block_outer: Option<S>,
    },
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> BlockPass<'input, Node, S> {
    fn has_work(&self) -> bool {
        self.active.is_some()
    }

    fn advance<Tree, M>(
        &mut self,
        tree: &mut Tree,
        inherited: Option<&InheritedFloatExclusions<S, Node>>,
    ) -> ComputeResult<Node, BlockPassStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            if self.positioned.is_some() {
                return self.advance_positioned(tree);
            }
            let active = self.active.as_mut().expect("active Block pass");
            let in_flow = active.in_flow.as_mut().expect("in-flow Block pass");
            match in_flow
                .traversal
                .step(tree, &active.flow.children, &active.constants)?
            {
                InFlowStep::Continue => {}
                InFlowStep::Child(child) => {
                    return Ok(ControlFlow::Continue(BlockChild::InFlow(child)));
                }
                InFlowStep::Complete => {
                    if let Some(output) = self.complete_traversal(tree, inherited)? {
                        return Ok(ControlFlow::Break(output));
                    }
                }
            }
        }
    }

    fn advance_positioned<Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, BlockPassStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let active = self.active.as_ref().expect("active Block pass");
        let positioned = self.positioned.as_mut().expect("positioned Block pass");
        if let Some(child) =
            positioned.next_child(tree, &active.flow.children, &active.constants)?
        {
            return Ok(ControlFlow::Continue(BlockChild::Positioned(child)));
        }
        let ActiveBlockPass {
            flow, constants, ..
        } = self.active.take().expect("completed Block pass");
        let positioned = self
            .positioned
            .take()
            .expect("completed positioned Block pass");
        positioned
            .finish(tree, flow.node, flow.input, &flow.style, &constants)
            .map(ControlFlow::Break)
    }

    fn complete_traversal<Tree, M>(
        &mut self,
        tree: &mut Tree,
        inherited: Option<&InheritedFloatExclusions<S, Node>>,
    ) -> ComputeResult<Node, Option<BlockResult<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let active = self.active.as_mut().expect("active Block pass");
        let InFlowPass { traversal, phase } =
            active.in_flow.take().expect("completed in-flow pass");
        let completed = traversal.finish(tree)?;
        let next = match phase {
            BlockPassPhase::Inline(measurement) => {
                let extent =
                    measurement.accept(&active.flow.constants, completed.intrinsic_inline_demand);
                drop(completed);
                let active = self.active.take().expect("completed inline pass");
                active
                    .flow
                    .begin_inline_or_normal(tree, inherited, extent)?
            }
            BlockPassPhase::Normal(sizing) => {
                let provisional = sizing.provisional_output_size(
                    &active.flow.constants,
                    &active.constants,
                    active.flow.input,
                    &completed,
                    active.flow.float_role,
                );
                if sizing.needs_final_pass {
                    let active = self.active.take().expect("completed normal pass");
                    active.flow.begin_final(
                        tree,
                        inherited,
                        completed,
                        provisional,
                        sizing.ratio_block_outer,
                    )?
                } else {
                    return self.begin_completion(
                        tree,
                        completed,
                        provisional,
                        sizing.ratio_block_outer,
                    );
                }
            }
            BlockPassPhase::Final {
                provisional,
                ratio_block_outer,
            } => return self.begin_completion(tree, completed, provisional, ratio_block_outer),
        };
        self.active = Some(next);
        Ok(None)
    }

    fn begin_completion<Tree, M>(
        &mut self,
        tree: &mut Tree,
        completed: in_flow::InFlowResult<Node, S>,
        provisional: LogicalSizeOf<S>,
        ratio_block_outer: Option<S>,
    ) -> ComputeResult<Node, Option<BlockResult<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let active = self.active.as_ref().expect("active Block pass");
        match active.flow.prepare_completion(
            tree,
            &active.constants,
            completed,
            provisional,
            ratio_block_outer,
        )? {
            ControlFlow::Break(output) => {
                self.active = None;
                Ok(Some(output))
            }
            ControlFlow::Continue(positioned) => {
                self.positioned = Some(positioned);
                Ok(None)
            }
        }
    }

    fn accept_child<Tree, M>(
        &mut self,
        tree: &mut Tree,
        pending: PendingBlockChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> ComputeResult<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let active = self.active.as_mut().expect("active Block pass");
        match pending {
            PendingBlockChild::InFlow(child) => active
                .in_flow
                .as_mut()
                .expect("in-flow child resumes its owning pass")
                .traversal
                .accept_child(tree, &active.constants, child, output),
            PendingBlockChild::Positioned(child) => self
                .positioned
                .as_mut()
                .expect("positioned child resumes its owning pass")
                .accept_child(tree, active.flow.node, &active.constants, child, output)
                .map_err(Into::into),
        }
    }
}

struct BlockFlow<'input, Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    style: BlockContainerProjection<'input, S>,
    children: Vec<Node>,
    constants: Constants<S>,
    float_role: float_context::Role,
    contained_size: Size<Option<S>>,
    semantic_work: bool,
    normal_block_fill: bool,
    // Keep the measured flow alive through final placement.
    normal_preview: Option<in_flow::InFlowResult<Node, S>>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> BlockFlow<'input, Node, S> {
    fn begin_inline_or_normal<Tree, M>(
        self,
        tree: &mut Tree,
        inherited: Option<&InheritedFloatExclusions<S, Node>>,
        mut extent: intrinsic::InlineExtent<S>,
    ) -> ComputeResult<Node, ActiveBlockPass<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match extent {
                intrinsic::InlineExtent::Resolved(accepted_inline) => {
                    return self.begin_normal(tree, inherited, accepted_inline);
                }
                intrinsic::InlineExtent::Measure(measurement) => {
                    let operation = measurement.operation();
                    if let Some(content) =
                        tree.fragment_inline_content(self.node, self.input, operation)?
                    {
                        extent = measurement.accept(&self.constants, content);
                        continue;
                    }
                    let constants = self.constants;
                    let traversal = InFlowTraversal::begin(
                        tree,
                        self.node,
                        &constants,
                        self.input,
                        InFlowPassContext {
                            inline_operation: operation,
                            inner_inline: None,
                            set_layout: false,
                            inherited,
                            projection: None,
                        },
                    )?;
                    return Ok(ActiveBlockPass {
                        flow: self,
                        constants,
                        in_flow: Some(InFlowPass {
                            traversal,
                            phase: BlockPassPhase::Inline(measurement),
                        }),
                    });
                }
            }
        }
    }

    fn begin_normal<Tree, M>(
        self,
        tree: &mut Tree,
        inherited: Option<&InheritedFloatExclusions<S, Node>>,
        accepted_inline_outer: S,
    ) -> ComputeResult<Node, ActiveBlockPass<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let PreparedNormalFlow {
            sizing,
            constants,
            input,
            inner_inline,
            set_layout,
        } = BlockFlowSizing::prepare(
            tree,
            &self.style,
            &self.constants,
            &self.children,
            BlockFlowRequest {
                node: self.node,
                input: self.input,
                contained_size: self.contained_size,
                inherited: inherited.is_some(),
                semantic_work: self.semantic_work,
                float_role: self.float_role,
                normal_block_fill: self.normal_block_fill,
                accepted_inline_outer,
            },
        )?;
        let traversal = InFlowTraversal::begin(
            tree,
            self.node,
            &constants,
            input,
            InFlowPassContext {
                inline_operation: crate::SizingOperation::Normal,
                inner_inline: Some(inner_inline),
                set_layout,
                inherited,
                projection: None,
            },
        )?;
        Ok(ActiveBlockPass {
            flow: self,
            constants,
            in_flow: Some(InFlowPass {
                traversal,
                phase: BlockPassPhase::Normal(sizing),
            }),
        })
    }

    fn begin_final<Tree, M>(
        mut self,
        tree: &mut Tree,
        inherited: Option<&InheritedFloatExclusions<S, Node>>,
        normal_pass: in_flow::InFlowResult<Node, S>,
        provisional: LogicalSizeOf<S>,
        ratio_block_outer: Option<S>,
    ) -> ComputeResult<Node, ActiveBlockPass<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let inset = self.constants.logical_content_box_inset();
        let logical_inner_size = LogicalSizeOf::new(
            Some((provisional.inline - inset.inline_sum()).max(S::ZERO)),
            Some((provisional.block - inset.block_sum()).max(S::ZERO)),
        );
        let constants = self
            .constants
            .with_logical_node_inner_size(logical_inner_size, ratio_block_outer.is_some());
        let traversal = InFlowTraversal::begin(
            tree,
            self.node,
            &constants,
            self.input,
            InFlowPassContext {
                inline_operation: crate::SizingOperation::Normal,
                inner_inline: logical_inner_size.inline,
                set_layout: self.input.run_mode().is_perform_layout(),
                inherited,
                projection: None,
            },
        )?;
        self.normal_preview = Some(normal_pass);
        Ok(ActiveBlockPass {
            flow: self,
            constants,
            in_flow: Some(InFlowPass {
                traversal,
                phase: BlockPassPhase::Final {
                    provisional,
                    ratio_block_outer,
                },
            }),
        })
    }

    fn prepare_completion<Tree, M>(
        &self,
        tree: &mut Tree,
        final_constants: &Constants<S>,
        final_pass: in_flow::InFlowResult<Node, S>,
        provisional_logical_output_size: LogicalSizeOf<S>,
        ratio_block_outer: Option<S>,
    ) -> ComputeResult<Node, PreparedFlow<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            node,
            input,
            ref style,
            ref children,
            constants,
            float_role,
            ..
        } = *self;
        if let Some(text) = tree.text_computation() {
            text.formatted_record_flow(&final_pass.formatted)
                .map_err(|error| inline_run::text_run_error(node, error.into()))?;
        }
        if let Some(groups) = tree.group_computation() {
            groups.select(final_pass.groups.clone());
        }
        let logical_output_size = LogicalSizeOf::new(
            provisional_logical_output_size.inline,
            constants
                .flow_axes
                .logical_size(input.known())
                .block
                // Keep the resolved border extent when deriving the content
                // extent and adding back its inset would round it differently.
                .or(constants.logical_node_outer_size().block)
                .or(final_constants.logical_node_outer_size().block)
                .or(ratio_block_outer.map(|_| provisional_logical_output_size.block))
                .unwrap_or_else(|| final_pass.auto_block(final_constants, float_role))
                .clamp_max_before_min_optional(
                    final_constants.logical_node_min_size().block,
                    final_constants.logical_node_max_size().block,
                )
                .max(final_constants.logical_padding_border_size().block),
        );
        let output_size = final_constants.flow_axes.physical_size(logical_output_size);
        prepare_flow_finish(
            tree,
            node,
            input,
            style,
            children,
            final_constants,
            SizedFlow {
                size: output_size,
                flow: final_pass,
            },
        )
    }
}

struct BlockFlowRequest<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    contained_size: Size<Option<S>>,
    inherited: bool,
    semantic_work: bool,
    float_role: float_context::Role,
    normal_block_fill: bool,
    accepted_inline_outer: S,
}

struct BlockFlowSizing<S: LayoutScalar> {
    accepted_inline_outer: S,
    ratio_block_outer: Option<S>,
    ratio_needs_content_minimum: bool,
    needs_final_pass: bool,
}

struct PreparedNormalFlow<S: LayoutScalar> {
    sizing: BlockFlowSizing<S>,
    constants: Constants<S>,
    input: ComputeInputOf<S>,
    inner_inline: S,
    set_layout: bool,
}

impl<S: LayoutScalar> BlockFlowSizing<S> {
    fn prepare<'input, Tree, M>(
        tree: &Tree,
        style: &BlockContainerProjection<'input, S>,
        constants: &Constants<S>,
        children: &[Tree::Node],
        request: BlockFlowRequest<Tree::Node, S>,
    ) -> ComputeResult<Tree::Node, PreparedNormalFlow<S>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let BlockFlowRequest {
            node,
            input,
            contained_size,
            inherited,
            semantic_work,
            float_role,
            normal_block_fill,
            accepted_inline_outer,
        } = request;
        let logical_inner_size = constants.logical_node_inner_size();
        let inherited_inline_request = constants
            .flow_axes
            .logical_size(constants.sizing_request.axes())
            .inline;
        let accepted_inline_inner = constants.accepted_inline_inner(node, accepted_inline_outer)?;
        let definite_child_inner = constants
            .flow_axes
            .logical_size(constants.definite_child_containing_block_size());
        let ratio_block_outer = if constants.ratio_dependent_block
            && definite_child_inner.block.is_none()
            && normal_block_fill
            && inherited_inline_request.operation() == crate::SizingOperation::Normal
            && (logical_inner_size.inline.is_some()
                || inherited_inline_request.space().definite_size().is_some())
        {
            let adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
                constants.padding_border_size
            } else {
                Size::ZERO
            };
            let size = constants
                .flow_axes
                .physical_size(LogicalSizeOf::new(Some(accepted_inline_outer), None))
                .sub_optional_clamped_to_zero(adjustment)
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .add_optional(adjustment);
            let block = constants.flow_axes.logical_size(size).block;
            if let Some(block) = block
                && !block.is_finite()
            {
                return Err(crate::error::sizing_resolution_error(
                    node,
                    crate::LengthResolutionStatus::InvalidNumeric { value: block }.into(),
                )
                .into());
            }
            block.map(|block| {
                block
                    .clamp_max_before_min_optional(
                        constants.logical_node_min_size().block,
                        constants.logical_node_max_size().block,
                    )
                    .max(constants.logical_padding_border_size().block)
            })
        } else {
            None
        };
        let block_minimum = match constants.flow_axes.block_axis() {
            PhysicalAxis::Horizontal => &style.common.min_size.width,
            PhysicalAxis::Vertical => &style.common.min_size.height,
        };
        let ratio_needs_content_minimum = constants
            .flow_axes
            .logical_size(contained_size)
            .block
            .is_none()
            && ratio_block_outer.is_some()
            && block_minimum.is_auto()
            && !style
                .common
                .overflow
                .establishes_independent_formatting_context()
            && (!children.is_empty() || semantic_work);
        // Collect the automatic minimum before using the settled ratio height
        // as a containing basis for child percentages.
        let definite_ratio_inner = ratio_block_outer
            .filter(|_| !ratio_needs_content_minimum)
            .map(|outer| {
                logical_inner_size.block.unwrap_or_else(|| {
                    (outer - constants.logical_content_box_inset().block_sum()).max(S::ZERO)
                })
            });
        let sizes_float_context = float_context::sizes_float_context(
            float_role,
            input,
            Some(accepted_inline_inner),
            inherited,
            tree.block_float_work(node),
        );
        let needs_final_pass = (input.run_mode().is_perform_layout() || sizes_float_context)
            && (logical_inner_size.inline.is_none()
                || input.run_mode().is_perform_layout() && ratio_needs_content_minimum
                || constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Block)
                    .is_decreasing()
                    && logical_inner_size.block.is_none());
        crate::layout_math::checked_available_space(accepted_inline_inner)
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        let normal_constants = constants.with_logical_node_inner_size(
            LogicalSizeOf::new(
                Some(accepted_inline_inner),
                definite_ratio_inner.or(logical_inner_size.block),
            ),
            definite_ratio_inner.is_some(),
        );
        // Actual Normal traversal owns height, Shape encounters and normal facts
        // at the accepted width. RunMode does not turn a size query intrinsic.
        let measured_inset_demand = input.run_mode().is_perform_layout()
            && relative::has_pending_children::<Tree, M>(
                tree,
                children,
                normal_constants.flow_axes,
            );
        let normal_pass_input = if ratio_needs_content_minimum {
            input.with_run_mode(RunMode::ComputeSize)
        } else {
            input
        };
        let set_normal_layout = normal_pass_input.run_mode().is_perform_layout()
            && (!needs_final_pass || measured_inset_demand);
        Ok(PreparedNormalFlow {
            sizing: Self {
                accepted_inline_outer,
                ratio_block_outer,
                ratio_needs_content_minimum,
                needs_final_pass,
            },
            constants: normal_constants,
            input: normal_pass_input,
            inner_inline: accepted_inline_inner,
            set_layout: set_normal_layout,
        })
    }

    fn provisional_output_size<Node>(
        &self,
        constants: &Constants<S>,
        normal_constants: &Constants<S>,
        input: ComputeInputOf<S>,
        normal_pass: &in_flow::InFlowResult<Node, S>,
        float_role: float_context::Role,
    ) -> LogicalSizeOf<S> {
        let auto_block = normal_pass.auto_block(normal_constants, float_role);
        let block = self.ratio_block_outer.map_or(auto_block, |ratio_block| {
            if self.ratio_needs_content_minimum {
                ratio_block.max(auto_block)
            } else {
                ratio_block
            }
        });
        let logical_intrinsic_outer_size = LogicalSizeOf::new(self.accepted_inline_outer, block);
        let logical_intrinsic_outer_size = LogicalSizeOf::new(
            logical_intrinsic_outer_size
                .inline
                .clamp_max_before_min_optional(
                    constants.logical_node_min_size().inline,
                    constants.logical_node_max_size().inline,
                ),
            logical_intrinsic_outer_size
                .block
                .clamp_max_before_min_optional(
                    constants.logical_node_min_size().block,
                    constants.logical_node_max_size().block,
                ),
        )
        .max_optional(constants.logical_padding_border_size().map(Some));
        let logical_outer_size = constants
            .logical_node_outer_size()
            .unwrap_or(logical_intrinsic_outer_size)
            .max_optional(constants.logical_padding_border_size().map(Some));
        constants
            .flow_axes
            .logical_size(input.known())
            .or(constants.logical_node_outer_size())
            .unwrap_or(logical_outer_size)
            .max_optional(constants.logical_padding_border_size().map(Some))
    }
}

/// Final box extent paired with the actual descendant production it sizes.
/// Ordinary layout and accepted principal materialization share this boundary.
struct SizedFlow<Node, S: LayoutScalar> {
    size: Size<S>,
    flow: in_flow::InFlowResult<Node, S>,
}

fn finish_flow<'input, Tree, S, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<S>,
    style: &BlockContainerProjection<'_, S>,
    children: &[Tree::Node],
    final_constants: &Constants<S>,
    sized: SizedFlow<Tree::Node, S>,
) -> ComputeResult<Tree::Node, BlockResult<Tree::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let mut positioned =
        match prepare_flow_finish(tree, node, input, style, children, final_constants, sized)? {
            ControlFlow::Break(output) => return Ok(output),
            ControlFlow::Continue(positioned) => positioned,
        };
    while let Some(child) = positioned.next_child(tree, children, final_constants)? {
        let output = tree.compute_child(child.node, child.input)?;
        positioned.accept_child(tree, node, final_constants, child, output)?;
    }
    positioned.finish(tree, node, input, style, final_constants)
}

type PreparedFlow<Node, S> = ControlFlow<BlockResult<Node, S>, PositionedFlow<Node, S>>;

struct PositionedFlow<Node, S: LayoutScalar> {
    output_size: Size<S>,
    baselines: crate::BaselinesOf<S>,
    named_baselines: crate::NamedBaselinesOf<S>,
    block_margin_collapse: PhysicalBlockMarginCollapseOf<S>,
    contributions: scroll::BlockScrollContributions<S>,
    static_positions: Vec<(Node, StaticPositionOf<S>)>,
    absolute: AbsoluteChildren<S>,
    // Keep funded formatting and placement data alive through publication.
    _formatted: Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    _pending_floats: Vec<PendingFloat<Node, S>>,
    float_context: FloatExclusions<S, Node>,
    inherited_floats: usize,
    remaining_cut: Option<entry::canonical::Separation<Node, S>>,
}

impl<Node: Copy + Eq, S: LayoutScalar> PositionedFlow<Node, S> {
    fn next_child<'input, Tree, M>(
        &mut self,
        tree: &Tree,
        children: &[Node],
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, Option<AbsoluteChild<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.absolute
            .next_child(tree, children, &self.static_positions, constants)
    }

    fn accept_child<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        constants: &Constants<S>,
        child: AbsoluteChild<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.absolute.accept_child(
            tree,
            node,
            constants,
            &mut self.contributions.geometry,
            child,
            output,
        )
    }

    fn finish<'input, Tree, M>(
        self,
        tree: &Tree,
        node: Node,
        input: ComputeInputOf<S>,
        style: &BlockContainerProjection<'_, S>,
        final_constants: &Constants<S>,
    ) -> ComputeResult<Node, BlockResult<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            output_size,
            baselines,
            named_baselines,
            block_margin_collapse,
            contributions,
            float_context,
            inherited_floats,
            remaining_cut,
            ..
        } = self;
        let published_scroll = with_block_scroll_projections::<Tree, M, _>(
            tree,
            node,
            |scroll_box_projection, scroll_target_projection| {
                finish_scroll_geometry::<Tree, S, M>(
                    node,
                    input.run_mode(),
                    scroll_box_projection,
                    scroll_target_projection,
                    final_constants,
                    output_size,
                    contributions,
                )
            },
        )?;
        let mut output = block_baseline_output(
            node,
            style,
            final_constants,
            ComputeOutputOf::<S>::from_sizes_and_baselines(
                output_size,
                published_scroll.content_size,
                baselines,
            ),
            named_baselines,
        )?;
        output.scroll_geometry = Some(published_scroll.geometry);
        output.block_margin_collapse = block_margin_collapse;
        Ok(BlockResult {
            output,
            used: input,
            floats: Some((float_context, inherited_floats)),
            remaining_cut,
        })
    }
}

fn prepare_flow_finish<'input, Tree, S, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<S>,
    style: &BlockContainerProjection<'_, S>,
    children: &[Tree::Node],
    final_constants: &Constants<S>,
    sized: SizedFlow<Tree::Node, S>,
) -> ComputeResult<Tree::Node, PreparedFlow<Tree::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let SizedFlow {
        size: output_size,
        flow: final_pass,
    } = sized;
    let top_margin = final_pass.top_margin(final_constants);
    let bottom_margin = final_pass.bottom_margin(final_constants);
    let margins_can_collapse_through = final_pass.collapses_through(final_constants);
    let block_margin_collapse = PhysicalBlockMarginCollapseOf::from_block_flow(
        final_constants.flow_axes,
        top_margin,
        bottom_margin,
        margins_can_collapse_through,
    );

    if let Some(flow) = final_pass.fragment_margins
        && let Some(state) = tree.fragment_margin_state()
    {
        state
            .record(
                node,
                final_constants.flow_axes,
                flow,
                final_constants.collapse_top_margin,
                final_pass.collapses_end_margin(final_constants),
            )
            .map_err(|error| inline_run::text_run_error(node, error.into()))?;
    }

    let (baselines, named_baselines) =
        final_pass.baselines.resolve(output_size).map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::TextNumeric {
                    value: error.value(),
                    reason: error.reason(),
                },
            )
        })?;

    if input.run_mode() == RunMode::ComputeSize {
        let mut output = block_baseline_output(
            node,
            style,
            final_constants,
            ComputeOutputOf::<S>::from_sizes_and_baselines(output_size, Size::ZERO, baselines),
            named_baselines,
        )?;
        output.block_margin_collapse = block_margin_collapse;
        Ok(ControlFlow::Break(BlockResult {
            output,
            used: input,
            floats: Some((final_pass.float_context, final_pass.inherited_floats)),
            remaining_cut: final_pass.remaining_cut,
        }))
    } else {
        let mut accepted_contributions = final_pass.contributions;
        let measured_content_size = Size::new(
            (output_size.width - final_constants.content_box_inset.sum_axes().width).max(S::ZERO),
            (output_size.height - final_constants.content_box_inset.sum_axes().height).max(S::ZERO),
        );
        let inset_basis = MeasuredInsetBasisOf::from_used_content_size(measured_content_size);
        relative::settle_children(
            tree,
            node,
            children,
            inset_basis,
            final_constants.flow_axes,
            &mut accepted_contributions,
        )?;
        let mut contributions =
            with_block_scroll_projections::<Tree, M, _>(tree, node, |scroll_box_projection, _| {
                prepare_scroll_contributions::<_, _, M>(
                    node,
                    input.run_mode(),
                    scroll_box_projection,
                    final_constants,
                    output_size,
                    accepted_contributions,
                )
            })?;
        layout_floats(
            tree,
            node,
            &final_pass.pending_floats,
            output_size,
            inset_basis,
            final_constants,
            &mut contributions,
        )?;
        let absolute = AbsoluteChildren::begin(tree, node, output_size, final_constants)?;
        Ok(ControlFlow::Continue(PositionedFlow {
            output_size,
            baselines,
            named_baselines,
            block_margin_collapse,
            contributions,
            static_positions: final_pass.static_positions,
            absolute,
            _formatted: final_pass.formatted,
            _pending_floats: final_pass.pending_floats,
            float_context: final_pass.float_context,
            inherited_floats: final_pass.inherited_floats,
            remaining_cut: final_pass.remaining_cut,
        }))
    }
}

#[derive(Clone, Copy, Debug)]
enum ChildContainingBlockExtent<S: LayoutScalar> {
    Definite(S),
    FinalAutoDerived(S),
}

#[derive(Clone, Copy, Debug)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    child_containing_block_extent: Size<Option<ChildContainingBlockExtent<S>>>,
    node_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    direction: Direction,
    line_box_direction_policy: crate::LineBoxDirectionPolicy,
    inline_baseline_context: crate::InlineBaselineContext,
    text_indent: crate::TextIndentOf<S>,
    hyphenate_limit_lines: crate::HyphenateLimitLines,
    hyphenate_limit_last: crate::HyphenateLimitLast,
    hyphenate_limit_zone: crate::LengthPercentageOf<S>,
    first_formatted: bool,
    wrap_style: crate::TextWrapStyle,
    text_align: TextAlign,
    text_align_last: TextAlignLast,
    text_justification: TextJustification,
    border: Edges<S>,
    padding: Edges<S>,
    effective_border: Edges<S>,
    padding_border_size: Size<S>,
    scrollbar_gutter: Edges<S>,
    content_box_inset: Edges<S>,
    sizing_request: crate::SizingRequestOf<S>,
    positioned_fallback_limits: Size<crate::OrthogonalFallbackLimitOf<S>>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    own_top_margin: CollapsibleMarginOf<S>,
    own_bottom_margin: CollapsibleMarginOf<S>,
    collapse_top_margin: bool,
    collapse_bottom_margin: bool,
    can_collapse_through: bool,
    ratio_dependent_block: bool,
}

impl<S: LayoutScalar> Constants<S> {
    fn logical_node_outer_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_outer_size)
    }

    fn logical_node_inner_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_inner_size)
    }

    fn definite_inline_outer(&self) -> Option<S> {
        self.logical_node_inner_size()
            .inline
            .map(|inner| inner + self.logical_content_box_inset().inline_sum())
    }

    fn accepted_inline_inner<Node: Copy, M>(
        &self,
        node: Node,
        outer: S,
    ) -> LayoutResultOf<Node, S, S, M> {
        crate::layout_math::remaining_available_extent(
            outer,
            self.logical_content_box_inset().inline_sum(),
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))
    }

    fn logical_node_min_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_min_size)
    }

    fn logical_node_max_size(&self) -> LogicalSizeOf<Option<S>> {
        self.flow_axes.logical_size(self.node_max_size)
    }

    fn logical_padding_border_size(&self) -> LogicalSizeOf<S> {
        self.flow_axes.logical_size(self.padding_border_size)
    }

    fn logical_content_box_inset(&self) -> LogicalEdgesOf<S> {
        self.flow_axes.logical_edges(self.content_box_inset)
    }

    fn containing_size(&self, inner_size: LogicalSizeOf<Option<S>>) -> Size<S> {
        self.node_outer_size
            .unwrap_or(self.flow_axes.physical_size(LogicalSizeOf::new(
                inner_size.inline.unwrap_or(S::ZERO)
                    + self.logical_content_box_inset().inline_sum(),
                inner_size.block.unwrap_or(S::ZERO) + self.logical_content_box_inset().block_sum(),
            )))
    }

    fn child_containing_block_size(
        &self,
        child_flow_axes: crate::geometry::FlowAxes,
    ) -> Size<Option<S>> {
        let child_logical_extent = child_flow_axes.logical_size(self.child_containing_block_extent);
        let child_inline_extent = match child_logical_extent.inline {
            Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
            Some(ChildContainingBlockExtent::FinalAutoDerived(value))
                if self.flow_axes.inline_axis() == child_flow_axes.inline_axis() =>
            {
                Some(value)
            }
            Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
        };
        // A content-derived final block extent remains indefinite for an
        // ordinary child's size percentages. Its measured numeric extent is
        // retained separately for normal placement and relative insets.
        let child_block_extent = match child_logical_extent.block {
            Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
            Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
        };
        child_flow_axes.physical_size(LogicalSizeOf::new(child_inline_extent, child_block_extent))
    }

    fn definite_child_containing_block_size(&self) -> Size<Option<S>> {
        self.child_containing_block_extent
            .map(|extent| match extent {
                Some(ChildContainingBlockExtent::Definite(value)) => Some(value),
                Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => None,
            })
    }

    fn with_logical_node_inner_size(
        mut self,
        inner_size: LogicalSizeOf<Option<S>>,
        ratio_block_is_definite: bool,
    ) -> Self {
        let final_inner_size = self.flow_axes.physical_size(inner_size);
        self.child_containing_block_extent =
            final_inner_size.zip_map(self.child_containing_block_extent, |value, previous| {
                value.map(|value| match previous {
                    Some(ChildContainingBlockExtent::Definite(_)) => {
                        ChildContainingBlockExtent::Definite(value)
                    }
                    Some(ChildContainingBlockExtent::FinalAutoDerived(_)) | None => {
                        ChildContainingBlockExtent::FinalAutoDerived(value)
                    }
                })
            });
        if ratio_block_is_definite {
            let mut basis = self
                .flow_axes
                .logical_size(self.child_containing_block_extent);
            basis.block = inner_size.block.map(ChildContainingBlockExtent::Definite);
            self.child_containing_block_extent = self.flow_axes.physical_size(basis);
        }
        self.node_inner_size = final_inner_size;
        let content_box_inset = self.logical_content_box_inset();
        self.node_outer_size = self.flow_axes.physical_size(LogicalSizeOf::new(
            inner_size
                .inline
                .map(|inline| inline + content_box_inset.inline_sum()),
            inner_size
                .block
                .map(|block| block + content_box_inset.block_sum()),
        ));
        self
    }

    fn from_projection<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &BlockContainerProjection<'_, S>,
        input: ComputeInputOf<S>,
        contained_size: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let flow_axes = style.common.flow_axes;
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            *style.common.padding,
            *style.common.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let own_logical_margin = flow_axes.logical_edges(*style.common.margin);
        let collapsible_margin = flow_axes.physical_edges(LogicalEdgesOf::new(
            LengthAutoOf::ZERO,
            LengthAutoOf::ZERO,
            own_logical_margin.block_start,
            own_logical_margin.block_end,
        ));
        let margin = input
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(
                collapsible_margin,
                input.parent(),
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, node)?;
        let padding_border_size = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize | SizingMode::PositionedColumnFlexInline => {
                (Size::NONE, Size::NONE, Size::NONE)
            }
            SizingMode::InherentSize => {
                let style_size = preferred_size(
                    style.common.size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?
                .apply_aspect_ratio(*style.common.aspect_ratio)
                .add_optional(box_sizing_adjustment);
                let min_size = minimum_size(
                    style.common.min_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?;
                let max_size = maximum_size(
                    style.common.max_size,
                    input.parent(),
                    SizingAlgorithm::Block,
                    true,
                )
                .transpose_with_node(tree, node)?;
                let inline_request = flow_axes.logical_size(input.sizing_request().axes()).inline;
                let normal_inline_ratio = normal_block_inline_ratio(
                    &style.common,
                    style.display,
                    tree.node_input(node).float,
                )
                .filter(|_| {
                    matches!(
                        input.parent_formatting_context(),
                        ParentFormattingContext::BlockFlow | ParentFormattingContext::NoParent
                    ) && inline_request.operation() == crate::SizingOperation::Normal
                        && (flow_axes.logical_size(input.known()).inline.is_some()
                            || inline_request.space().definite_size().is_some())
                });
                let (min_size, max_size) = if let Some(ratio) = normal_inline_ratio {
                    transfer_block_ratio_inline_limits(flow_axes, ratio, min_size, max_size)
                        .map_err(|status| {
                            crate::error::sizing_resolution_error(node, status.into())
                        })?
                } else {
                    (
                        min_size.apply_aspect_ratio(*style.common.aspect_ratio),
                        max_size.apply_aspect_ratio(*style.common.aspect_ratio),
                    )
                };
                (
                    style_size,
                    min_size.add_optional(box_sizing_adjustment),
                    max_size.add_optional(box_sizing_adjustment),
                )
            }
        };
        // Fragment progress changes the used sizing floor, not the computed
        // minimum that determines whether this box can collapse through.
        let computed_min_size = min_size;
        let min_size = fragment_minimum::apply(
            tree.fragment_minimum(node),
            computed_min_size,
            padding_border_size,
        );
        let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
            (Some(min), Some(max)) if max <= min => Some(min),
            _ => None,
        });
        let is_root = input.run_mode() == RunMode::PerformRootLayout;
        let boundary_margins_can_collapse =
            input.parent_formatting_context() == ParentFormattingContext::BlockFlow;
        let blocks_margin_collapse = flow_root::establishes_flow_root(
            style.display,
            style.float,
            style.block_flow_context,
            &style.common,
            input.containing_flow_axes().writing_mode(),
        );
        let is_margin_collapsing_block = style.display == super::Display::Block;
        let ratio_dependent_block = input.sizing_mode() == SizingMode::InherentSize
            && automatic_block_depends_on_ratio(&style.common);
        let can_collapse_through = is_margin_collapsing_block
            && !ratio_dependent_block
            && boundary_margins_can_collapse
            && !is_root
            && !blocks_margin_collapse
            && style.common.position == Position::Relative
            && flow_axes.logical_edges(padding).block_start == S::ZERO
            && flow_axes.logical_edges(padding).block_end == S::ZERO
            && flow_axes.logical_edges(border).block_start == S::ZERO
            && flow_axes.logical_edges(border).block_end == S::ZERO
            && !matches!(flow_axes.logical_size(style_size).block, Some(block) if block > S::ZERO)
            && !matches!(flow_axes.logical_size(computed_min_size).block, Some(block) if block > S::ZERO);
        let definite_outer_size = input
            .known()
            .or(min_max_definite_size)
            .or(style_size.clamp_max_before_min_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let node_outer_size = contained_size.or(definite_outer_size);
        let unconstrained_scroll_box_size = padding_border_size
            + Size::splat(style.scrollbar_width.get() + style.scrollbar_width.get());
        let scroll_box_size = node_outer_size
            .or(input.sizing_request().axes().map(|request| {
                request
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get)
            }))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border_size, |size, minimum| size.max(minimum));
        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes,
            computed_overflow: style.common.overflow,
            item_is_replaced: style.common.item_is_replaced,
            border_box_size: scroll_box_size,
            border,
            padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
        })
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let effective_border = scroll_box.effective_border();
        let scrollbar_gutter = scroll_box.effective_gutter();
        let content_box_inset =
            effective_border + scrollbar_gutter + scroll_box.effective_padding();
        let content_box_inset_size = content_box_inset.sum_axes();
        let node_inner_size = node_outer_size.sub_optional_clamped_to_zero(content_box_inset_size);
        let content_axes = crate::layout_math::transpose_axis_requests_at_node(
            node,
            input.sizing_request().axes().zip_map(
                content_box_inset_size,
                crate::layout_math::inset_axis_request,
            ),
        )?;
        let fallback_source = crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
            flow_axes,
            computed_overflow: style.common.overflow,
            item_is_replaced: style.common.item_is_replaced,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            padding,
            border,
        };
        let fixed_outer = if input.sizing_mode() == SizingMode::InherentSize {
            definite_outer_size
        } else {
            input.known()
        };
        let fallback_sources = crate::layout_math::qualify_fixed_fallback_sources(
            input,
            SizingAlgorithm::Block,
            &style.common,
            fixed_outer,
            box_sizing_adjustment,
            crate::scroll::UsedOverflow::from_computed(
                style.common.overflow,
                style.common.item_is_replaced,
            )
            .is_scroll_container(),
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error))?;
        let fallback = crate::layout_math::child_orthogonal_fallback(
            input.sizing_request().orthogonal_fallback(),
            fallback_sources.definite_outer,
            fallback_sources.minimum_outer,
            fallback_sources.maximum_outer,
            fallback_source,
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        let positioned_fallback_limits = crate::layout_math::containing_orthogonal_fallback_limits(
            fallback_sources.minimum_outer,
            fallback_sources.maximum_outer,
            crate::scroll::measured_leaf_content_box_inset(
                crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
                    padding: Edges::ZERO,
                    ..fallback_source
                },
            )
            .sum_axes(),
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        let sizing_request = input
            .sizing_request()
            .with_axes(content_axes)
            .with_orthogonal_fallback(fallback);
        let logical_padding = flow_axes.logical_edges(padding);
        let logical_border = flow_axes.logical_edges(border);
        let logical_style_size = flow_axes.logical_size(style_size);
        let logical_margin = flow_axes.logical_edges(margin);

        let first_formatted = match style.block_generation {
            crate::BlockGeneration::Principal => true,
            crate::BlockGeneration::Anonymous => {
                let context = input.block_indent_context().ok_or_else(|| {
                    crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(node),
                        crate::LayoutOperation::ChildLayout,
                        crate::LayoutErrorKindOf::MissingContext(
                            crate::LayoutMissingContext::AnonymousGeneratingBlock,
                        ),
                    )
                })?;
                if context.inherited != style.text_indent {
                    return Err(crate::LayoutErrorOf::new(
                        crate::LayoutErrorSiteOf::Node(node),
                        crate::LayoutOperation::ChildLayout,
                        crate::LayoutErrorKindOf::InvalidInput(
                            crate::LayoutInvalidInputOf::AnonymousIndentMismatch,
                        ),
                    ));
                }
                context.first_formatted
            }
        };
        Ok(Self {
            text_indent: style.text_indent,
            hyphenate_limit_lines: style.hyphenate_limit_lines,
            hyphenate_limit_last: style.hyphenate_limit_last,
            hyphenate_limit_zone: style.hyphenate_limit_zone,
            first_formatted,
            flow_axes,
            node_outer_size,
            node_inner_size,
            child_containing_block_extent: node_inner_size.zip_map(
                input.descendant_percentage_basis(definite_outer_size),
                |inner, definite| {
                    inner.map(|value| {
                        if definite.is_some() {
                            ChildContainingBlockExtent::Definite(value)
                        } else {
                            ChildContainingBlockExtent::FinalAutoDerived(value)
                        }
                    })
                },
            ),
            node_min_size: min_size,
            node_max_size: max_size,
            direction: flow_axes.direction(),
            line_box_direction_policy: style.line_box_direction_policy,
            inline_baseline_context: style.inline_baseline_context,
            wrap_style: style.wrap_style,
            text_align: style.text_align,
            text_align_last: style.text_align_last,
            text_justification: style.text_justification,
            border,
            padding,
            effective_border,
            padding_border_size,
            scrollbar_gutter,
            content_box_inset,
            sizing_request,
            positioned_fallback_limits,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            own_top_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_start.unwrap_or(S::ZERO),
            ),
            own_bottom_margin: CollapsibleMarginOf::<S>::from_margin(
                logical_margin.block_end.unwrap_or(S::ZERO),
            ),
            collapse_top_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.common.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_start == S::ZERO
                && logical_border.block_start == S::ZERO,
            collapse_bottom_margin: is_margin_collapsing_block
                && boundary_margins_can_collapse
                && !is_root
                && style.common.position == Position::Relative
                && !blocks_margin_collapse
                && logical_padding.block_end == S::ZERO
                && logical_border.block_end == S::ZERO
                && logical_style_size.block.is_none()
                && !ratio_dependent_block,
            can_collapse_through,
            ratio_dependent_block,
        })
    }

    #[cfg(test)]
    fn new<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Self::from_projection(
            tree,
            node,
            &BlockContainerProjection::from_node(style),
            input,
            Size::NONE,
        )
    }
}

fn block_baseline_output<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    style: &BlockContainerProjection<'_, S>,
    constants: &Constants<S>,
    mut output: ComputeOutputOf<S>,
    mut facts: crate::NamedBaselinesOf<S>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
    let used = crate::scroll::UsedOverflow::from_computed(
        style.common.overflow,
        style.common.item_is_replaced,
    );
    if !used.is_scroll_container() {
        return Ok(output.with_named_baselines(facts));
    }
    let size = output.size;
    let mut baselines = output.baselines();
    let numeric_error = |crate::FiniteScalarErrorOf::NonFinite { value }| {
        crate::LayoutErrorOf::new(
            crate::LayoutErrorSiteOf::Node(node),
            crate::LayoutOperation::ChildLayout,
            crate::LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::InvalidNumeric {
                value,
            }),
        )
    };
    // Positional facts already use Layout's canonical ZERO INITIAL ANCHOR,
    // after actual content alignment. Range extrema/padding are not offsets.
    // The exceptional exterior edge is not translated by its own scroll.
    if style.baseline_source == crate::BaselineSource::Auto
        && style.display.inner_display() == crate::Display::Block
        && !style.common.item_is_replaced
    {
        let end_margin = constants.own_bottom_margin.resolve();
        let coordinate = match constants.flow_axes.block_end() {
            crate::PhysicalSide::Top | crate::PhysicalSide::Left => -end_margin,
            crate::PhysicalSide::Bottom => size.height + end_margin,
            crate::PhysicalSide::Right => size.width + end_margin,
        };
        let coordinate =
            crate::BaselineCoordinateOf::try_new(constants.flow_axes.block_axis(), coordinate)
                .map_err(numeric_error)?;
        baselines.record_last(
            crate::BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                None,
                Some(coordinate.offset()),
            )
            .last,
        );
        let edge = crate::BaselineFactOf::Available(coordinate);
        for name in [
            crate::BaselineName::Alphabetic,
            crate::BaselineName::Central,
        ] {
            facts = facts.with(
                name,
                crate::BaselinePairOf::new(facts.get(name).first(), edge),
            );
        }
    }
    // Clamp known content and exceptional coordinates in THIS border frame;
    // a parent subsequently translates them once. Unknown/Absent stay intact.
    let baselines = baselines.clamped_to_border(size).map_err(numeric_error)?;
    output.first_baselines = baselines.first;
    output.last_baselines = baselines.last;
    Ok(output.with_named_baselines(facts.clamped_to_border(size).map_err(numeric_error)?))
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::LengthPercentageOf;
    use crate::layout_math::{resolution_optional, resolution_or_zero};

    fn input<S: LayoutScalar>(
        containing_flow_axes: crate::geometry::FlowAxes,
        parent: Size<Option<S>>,
    ) -> ComputeInputOf<S> {
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::ContentSize,
            RequestedAxis::Both,
            Size::NONE,
            parent,
            ContainingLayoutContext::new(containing_flow_axes, ParentFormattingContext::NoParent),
            crate::SizingRequestOf::new(
                Size::splat(crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                )),
                crate::OrthogonalFallbackOf::new(
                    crate::Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        )
    }

    fn percentage_edges<S: LayoutScalar>() -> Edges<LengthOf<S>> {
        Edges::new(
            LengthOf::percent(S::from_f64(0.1)),
            LengthOf::percent(S::from_f64(0.2)),
            LengthOf::percent(S::from_f64(0.3)),
            LengthOf::percent(S::from_f64(0.4)),
        )
    }

    fn expected_percentage_edges<S: LayoutScalar>(basis: S) -> Edges<S> {
        Edges::new(
            S::from_f64(0.1) * basis,
            S::from_f64(0.2) * basis,
            S::from_f64(0.3) * basis,
            S::from_f64(0.4) * basis,
        )
    }

    fn characterize_constants<S: LayoutScalar>(largest: S) {
        crate::layout_math::assert_fri06_c13_t06_resolution_policy::<S>(
            resolution_or_zero,
            resolution_optional,
        );

        let border = Edges::new(
            LengthOf::px(S::from_f64(1.0)),
            LengthOf::px(S::from_f64(2.0)),
            LengthOf::px(S::from_f64(3.0)),
            LengthOf::px(S::from_f64(4.0)),
        );
        let expected_border = Edges::new(
            S::from_f64(1.0),
            S::from_f64(2.0),
            S::from_f64(3.0),
            S::from_f64(4.0),
        );
        let style = NodeInputOf {
            display: crate::Display::Block,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let mut owner =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());
        let tree = owner.borrow_compute();

        for (flow, parent, expected_padding) in [
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                crate::geometry::FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), None),
                Edges::ZERO,
            ),
        ] {
            let constants = Constants::new::<_, core::convert::Infallible>(
                &tree,
                7,
                &style,
                input(flow, parent),
            )
            .expect("block constants edge characterization must resolve");
            assert_eq!(constants.padding, expected_padding);
            assert_eq!(constants.border, expected_border);
        }

        let positive_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(largest, S::ONE)
                .expect("finite positive overflow coefficients"),
        );
        let negative_overflow = LengthOf::value(
            LengthPercentageOf::from_coefficients(-largest, -S::ONE)
                .expect("finite negative overflow coefficients"),
        );
        let failing_style = NodeInputOf {
            display: crate::Display::Block,
            padding: Edges::new(
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
                positive_overflow,
            ),
            border: Edges::new(
                negative_overflow,
                LengthOf::ZERO,
                LengthOf::ZERO,
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        let mut failing_owner =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, failing_style.clone());
        let failing_tree = failing_owner.borrow_compute();
        let error = Constants::new::<_, core::convert::Infallible>(
            &failing_tree,
            7,
            &failing_style,
            input(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(largest)),
            ),
        )
        .expect_err("padding failure must precede the distinct border failure");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(7));
        assert_eq!(error.operation(), LayoutOperation::ValueResolution);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                value: S::INFINITY,
            }),
        );
    }

    #[test]
    fn fri06_c13_t06_block_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_block_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}
