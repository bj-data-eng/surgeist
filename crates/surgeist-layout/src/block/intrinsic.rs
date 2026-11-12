//! Scalar Block inline demand, independent of Normal block progression.

use super::*;
use crate::{FlowAxes, SizingOperation, SizingRequestOf};

pub(super) fn normal_fill<S: LayoutScalar>(
    style: &BlockContainerProjection<'_, S>,
    input: ComputeInputOf<S>,
    float: crate::Float,
) -> bool {
    input.sizing_mode() == SizingMode::InherentSize
        && normal_block_auto_inline(&style.common, style.display, float)
        && matches!(
            input.parent_formatting_context(),
            ParentFormattingContext::BlockFlow | ParentFormattingContext::NoParent
        )
}

#[derive(Clone, Copy)]
pub(super) enum InlineExtent<S> {
    Resolved(S),
    Measure(InlineMeasurement<S>),
}

#[derive(Clone, Copy)]
pub(super) enum InlineMeasurement<S> {
    MinContent,
    MaxContent,
    FitMaximum { available: S },
    FitMinimum { available: S, maximum: S },
}

impl<S: LayoutScalar> InlineMeasurement<S> {
    pub(super) fn operation(&self) -> SizingOperation {
        match self {
            Self::MinContent | Self::FitMinimum { .. } => SizingOperation::MinContent,
            Self::MaxContent | Self::FitMaximum { .. } => SizingOperation::MaxContent,
        }
    }

    pub(super) fn accept(self, constants: &Constants<S>, measured: S) -> InlineExtent<S> {
        let content = match self {
            Self::MinContent | Self::MaxContent => measured,
            Self::FitMaximum { available } => {
                return InlineExtent::Measure(Self::FitMinimum {
                    available,
                    maximum: measured,
                });
            }
            Self::FitMinimum { available, maximum } => maximum.min(measured.max(available)),
        };
        InlineExtent::Resolved(outer_inline(constants, content))
    }
}

fn outer_inline<S: LayoutScalar>(constants: &Constants<S>, content: S) -> S {
    (content + constants.logical_content_box_inset().inline_sum())
        .clamp_max_before_min_optional(
            constants.logical_node_min_size().inline,
            constants.logical_node_max_size().inline,
        )
        .max(constants.logical_padding_border_size().inline)
}

pub(super) fn begin_inline<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    style: &BlockContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
) -> InlineExtent<Tree::Scalar>
where
    Tree: Compute<'input, M>,
{
    if let Some(outer) = constants.definite_inline_outer() {
        return InlineExtent::Resolved(outer);
    }
    let inherited = constants
        .flow_axes
        .logical_size(constants.sizing_request.axes())
        .inline;
    let preferred = constants
        .flow_axes
        .logical_size(style.common.size.clone())
        .inline;
    let operation = if input.sizing_mode() == SizingMode::InherentSize && preferred.is_min_content()
    {
        SizingOperation::MinContent
    } else if input.sizing_mode() == SizingMode::InherentSize && preferred.is_max_content() {
        SizingOperation::MaxContent
    } else {
        inherited.operation()
    };
    let measurement = match operation {
        SizingOperation::MinContent => InlineMeasurement::MinContent,
        SizingOperation::MaxContent => InlineMeasurement::MaxContent,
        SizingOperation::Normal => {
            let available = inherited
                .space()
                .definite_size()
                .map(crate::NonNegativeFiniteOf::get);
            if normal_fill(style, input, tree.node_input(node).float)
                && let Some(available) = available
            {
                return InlineExtent::Resolved(outer_inline(constants, available));
            }
            available.map_or(InlineMeasurement::MaxContent, |available| {
                InlineMeasurement::FitMaximum { available }
            })
        }
    };
    InlineExtent::Measure(measurement)
}

/// Resolve the same accepted border-box inline extent for scalar demand and
/// actual layout. ContentSize deliberately ignores authored preferred sizing.
pub(super) fn accepted_inline<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    style: &BlockContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
    mut component: impl FnMut(
        &mut Tree,
        SizingOperation,
    ) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>,
) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut extent = begin_inline::<Tree, M>(tree, node, style, constants, input);
    loop {
        match extent {
            InlineExtent::Resolved(outer) => return Ok(outer),
            InlineExtent::Measure(measurement) => {
                let measured = component(tree, measurement.operation())?;
                extent = measurement.accept(constants, measured);
            }
        }
    }
}

pub(crate) struct InlineContentChild<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    margin: Edges<ResolvedLengthAutoOf<S>>,
}

type NextInlineContentChild<Node, S, M> =
    LayoutResultOf<Node, Option<InlineContentChild<Node, S>>, S, M>;

struct InlineContentPass<Node, S: LayoutScalar> {
    children: std::vec::IntoIter<Node>,
    request: SizingRequestOf<S>,
    inner: Size<Option<S>>,
    containing_pass: crate::scroll::SettledAutoScrollbarState,
    first_in_flow: bool,
    demand: S,
}

impl<Node: Copy + Eq, S: LayoutScalar> InlineContentPass<Node, S> {
    fn new(
        children: Vec<Node>,
        constants: &Constants<S>,
        input: ComputeInputOf<S>,
        operation: SizingOperation,
    ) -> Self {
        let axes = constants.flow_axes;
        let mut requests = constants.sizing_request.axes();
        match axes.inline_axis() {
            PhysicalAxis::Horizontal => requests.width = requests.width.with_operation(operation),
            PhysicalAxis::Vertical => requests.height = requests.height.with_operation(operation),
        }
        let inner = LogicalSizeOf::new(None, constants.logical_node_inner_size().block);
        Self {
            children: children.into_iter(),
            request: constants.sizing_request.with_axes(requests),
            inner: axes.physical_size(inner),
            containing_pass: input.settled_auto_scrollbars(),
            first_in_flow: true,
            demand: S::ZERO,
        }
    }

    fn next_child<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
    ) -> NextInlineContentChild<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        for child in self.children.by_ref() {
            let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
            let InlineParticipantKindOf::Box(style) = projection.kind() else {
                return Err(profile_error(child));
            };
            if style.display == crate::Display::None || style.common.position == Position::Absolute
            {
                continue;
            }
            if !style.float.is_none() || style.display.is_inline_level() {
                return Err(profile_error(child));
            }
            let prepared = super::in_flow::prepare_child(
                tree,
                child,
                &style,
                constants,
                super::in_flow::ChildSizingContext {
                    inner_size: self.inner,
                    request: self.request,
                    run_mode: RunMode::ComputeSize,
                    axis: requested_axis(constants.flow_axes.inline_axis()),
                    first_in_flow: &mut self.first_in_flow,
                },
            )?;
            return Ok(Some(InlineContentChild {
                node: child,
                input: prepared
                    .input
                    .with_containing_auto_scrollbar_pass(self.containing_pass),
                margin: prepared.unresolved_margin,
            }));
        }
        Ok(None)
    }

    fn accept_child(&mut self, axes: FlowAxes, child: InlineContentChild<Node, S>, inline: S) {
        let margin = super::in_flow::resolve_logical_inline_margin(
            axes.logical_edges(child.margin),
            inline,
            None,
        );
        self.demand = self.demand.max(super::in_flow::signed_inline_demand(
            inline,
            margin,
            margin.inline_start + inline + margin.inline_end,
        ));
    }
}

/// Full-source content contribution. Only an inline scalar is produced.
pub(super) fn content<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    constants: &Constants<Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
    operation: SizingOperation,
) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut pass =
        InlineContentPass::new(tree.children(node).collect(), constants, input, operation);
    let axes = constants.flow_axes;
    while let Some(child) = pass.next_child(tree, constants)? {
        let inline = tree.compute_axis_demand(child.node, child.input, axes.inline_axis())?;
        pass.accept_child(axes, child, inline);
    }
    Ok(pass.demand)
}

pub(crate) struct BlockAxisDemand<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    constants: Constants<S>,
    extent: InlineExtent<S>,
    content: Option<InlineContentPass<Node, S>>,
}

impl<Node: Copy + Eq, S: LayoutScalar> BlockAxisDemand<Node, S> {
    pub(crate) fn begin<'input, Tree, M>(
        tree: &mut Tree,
        node: Node,
        input: ComputeInputOf<S>,
        axis: PhysicalAxis,
    ) -> LayoutResultOf<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let style = block_container_projection::<Tree, M>(tree, node);
        if style.common.flow_axes.inline_axis() != axis {
            return Err(profile_error(node));
        }
        let contained = containment_sizing(
            node,
            tree.node_input(node),
            input,
            tree.fragment_minimum(node),
        )?
        .map_or(Size::NONE, |sizing| sizing.outer);
        let constants =
            Constants::from_projection::<Tree, M>(tree, node, &style, input, contained)?;
        let extent = begin_inline::<Tree, M>(tree, node, &style, &constants, input);
        Ok(Self {
            node,
            input,
            constants,
            extent,
            content: None,
        })
    }

    pub(crate) fn axis(&self) -> PhysicalAxis {
        self.constants.flow_axes.inline_axis()
    }

    pub(crate) fn next_child<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> NextInlineContentChild<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        while let InlineExtent::Measure(measurement) = self.extent {
            let pass = if let Some(pass) = &mut self.content {
                pass
            } else {
                let operation = measurement.operation();
                if let Some(value) =
                    tree.fragment_inline_content(self.node, self.input, operation)?
                {
                    self.extent = measurement.accept(&self.constants, value);
                    continue;
                }
                self.content.insert(InlineContentPass::new(
                    tree.children(self.node).collect(),
                    &self.constants,
                    self.input,
                    operation,
                ))
            };
            if let Some(child) = pass.next_child(tree, &self.constants)? {
                return Ok(Some(child));
            }
            self.extent = measurement.accept(&self.constants, pass.demand);
            self.content = None;
        }
        Ok(None)
    }

    pub(crate) fn accept_child(&mut self, child: InlineContentChild<Node, S>, inline: S) {
        let axes = self.constants.flow_axes;
        self.content
            .as_mut()
            .expect("a yielded inline child belongs to the active content pass")
            .accept_child(axes, child, inline);
    }

    pub(crate) fn finish(self) -> S {
        let InlineExtent::Resolved(outer) = self.extent else {
            unreachable!("inline demand finishes after its last child");
        };
        outer
    }
}

pub(crate) fn axis_demand<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    axis: PhysicalAxis,
) -> LayoutResultOf<Tree::Node, Tree::Scalar, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut demand = BlockAxisDemand::begin(tree, node, input, axis)?;
    while let Some(child) = demand.next_child(tree)? {
        let inline = tree.compute_axis_demand(child.node, child.input, demand.axis())?;
        demand.accept_child(child, inline);
    }
    Ok(demand.finish())
}

pub(crate) fn requested_axis(axis: PhysicalAxis) -> RequestedAxis {
    match axis {
        PhysicalAxis::Horizontal => RequestedAxis::Horizontal,
        PhysicalAxis::Vertical => RequestedAxis::Vertical,
    }
}

pub(crate) fn profile_error<Node: Copy, S: LayoutScalar, M>(
    node: Node,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::UnsupportedCapability(
            crate::LayoutUnsupportedCapability::BlockFragmentationProfile,
        ),
    )
}

/// A canonical full-source context and its demanded CONTENT components. Actual
/// windows retain ownership of their own insets, clamps and Normal height.
#[derive(Clone)]
pub(crate) struct InlineComponents<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) values: Vec<(SizingOperation, S)>,
    pub(crate) minimum: Option<super::fragment_minimum::Receipt<S>>,
}

pub(crate) fn prepare<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    fragment_axes: FlowAxes,
    entries: &mut Vec<InlineComponents<Tree::Node, Tree::Scalar>>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let node_style = tree.node_input(node);
    let own = FlowAxes::new(node_style.writing_mode, node_style.direction);
    if node_style.display == crate::Display::None
        || tree.has_leaf_measurement(node)
        || tree.child_count(node) == 0
        || node_style.item_is_replaced
        || own.block_axis() != fragment_axes.block_axis()
    {
        return Ok(());
    }
    let style = block_container_projection::<Tree, M>(tree, node);
    let contained = containment_sizing(
        node,
        tree.node_input(node),
        input,
        tree.fragment_minimum(node),
    )?
    .map_or(Size::NONE, |sizing| sizing.outer);
    let constants = Constants::from_projection::<Tree, M>(tree, node, &style, input, contained)?;
    let mut values = Vec::new();
    let outer = accepted_inline(tree, node, &style, &constants, input, |tree, operation| {
        if let Some((_, value)) = values.iter().find(|(op, _)| *op == operation) {
            return Ok(*value);
        }
        let value = content(tree, node, &constants, input, operation)?;
        values.push((operation, value));
        Ok(value)
    })?;
    entries.push(InlineComponents {
        node,
        input,
        values,
        minimum: super::fragment_minimum::receipt(tree.node_input(node), &constants)
            .map_err(|error| crate::error::sizing_resolution_error(node, error))?,
    });
    let inner = constants.accepted_inline_inner(node, outer)?;
    let constants = constants.with_logical_node_inner_size(
        LogicalSizeOf::new(Some(inner), constants.logical_node_inner_size().block),
        false,
    );
    let children = tree.children(node).collect::<Vec<_>>();
    let mut first_in_flow = true;
    for child in children {
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let InlineParticipantKindOf::Box(style) = projection.kind() else {
            return Err(profile_error(child));
        };
        if style.display == crate::Display::None || style.common.position == Position::Absolute {
            continue;
        }
        let prepared = super::in_flow::prepare_child(
            tree,
            child,
            &style,
            &constants,
            super::in_flow::ChildSizingContext {
                inner_size: constants
                    .flow_axes
                    .physical_size(constants.logical_node_inner_size()),
                request: constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: RequestedAxis::Both,
                first_in_flow: &mut first_in_flow,
            },
        )?;
        prepare(tree, child, prepared.input, fragment_axes, entries)?;
    }
    Ok(())
}

/// Invocation-local full-source demand custody. Each instance is prepared from
/// the complete source tree under one effective root context, never a window.
pub(crate) struct FragmentContext<Node, S: LayoutScalar> {
    pub(crate) target: crate::LayoutTargetBasis,
    pub(crate) request: crate::LayoutRootRequestOf<S>,
    pub(crate) entries: Vec<InlineComponents<Node, S>>,
}
