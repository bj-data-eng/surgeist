use std::ops::ControlFlow;

use super::{
    AlignContent, AlignItems, AvailableSpaceOf, AxisSizingRequestOf, BaselinesOf, BoxSizing,
    Compute, ComputeInputOf, ComputeOutputOf, DefaultScalar, Direction, Display, Edges,
    GridAutoFlow, GridPlacement, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf,
    LayoutInternalInvariant, LayoutInvalidInputOf, LayoutOperation, LayoutResultOf, LayoutScalar,
    LengthAutoOf, LengthOf, LengthResolutionStatus, MaxTrackSizingOf, MinSizeOf, MinTrackSizingOf,
    NodeOutputOf, OrthogonalFallbackOf, Overflow, Point, Position, PreferredSizeOf, RequestedAxis,
    RunMode, Scalar, Size, SizingAlgorithm, SizingMode, SizingOperation, SizingRequestOf,
    TrackComponentOf, TrackRepeat, TrackSizingOf, Traverse,
};
use crate::engine::containment::{
    compute_empty_box, contained_outer_size, empty_sizing_input, size_containment,
};
use crate::error::{layout_own_geometry_error, sizing_resolution_error};
use crate::geometry::{LogicalSizeOf, PhysicalAxis};
use crate::layout_math::{
    OptionalSizeExt, UncheckedOptionalSizeSubExt, child_orthogonal_fallback, resolution_optional,
    resolution_or_zero, resolve_containing_padding_border,
};
use crate::output::{PhysicalBaseline, omit_indefinite_percentage_axis};
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, CanonicalScrollGeometrySourceOf,
    ClipMarginSourceOf, MeasuredLeafContentBoxInsetSourceOf, OptimalRegionInsetsOf,
    ScrollOriginAxes, canonical_scroll_box_from_source, canonical_scroll_geometry_from_source,
    measured_leaf_content_box_inset,
};
use crate::sizing::resolve::{
    EdgesResultExt, FixedFallbackSource, ResolvedPreferredSize, SizeResultExt,
    SizingResolutionError, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional, resolve_preferred_sizing,
};

mod alignment;
mod axis;
mod baseline_reference;
mod child;
mod input;
mod intrinsic_stage;
mod lane_repeat;
mod lanes;
mod named;
mod placement;
mod subgrid;
#[cfg(test)]
#[path = "../grid_tests.rs"]
mod tests;
mod topology;
mod tracks;

use alignment::*;
pub use axis::GridAxisKind;
use axis::{GridAxisMappingInput, GridAxisMappingReport, map_grid_axis};
use baseline_reference::*;
use child::*;
use input::{
    GridChildInput, GridContainerInput, GridContainerProjection, GridItemProjection,
    order_modified_indexes,
};
use intrinsic_stage::{
    GridIntrinsicProbe, GridIntrinsicProbeKind, IntrinsicSubgridSource, GridTrackSizing,
};
use lane_repeat::{LaneRepeatExpansion, LaneRepeatPlacement};
pub use lanes::{
    DefiniteLaneIntrinsicItem, DefiniteLaneIntrinsicItemOf, IndefiniteLaneContributionGroup,
    IndefiniteLaneContributionGroupOf, LaneContributionFacts, LaneContributionFactsOf,
    LaneIntrinsicItem, LaneIntrinsicItemKind, LaneIntrinsicItemOf, LaneIntrinsicSizingInput,
    LaneIntrinsicSizingInputOf, LaneIntrinsicSizingReport, LaneIntrinsicSizingReportOf, LaneItem,
    LaneItemOf, LaneItemOffset, LaneItemOffsetOf, LanePlacementError, LanePlacementInput,
    LanePlacementInputOf, LanePlacementReport, LanePlacementReportOf, LaneTrackSpan,
    LaneTrackSpanLength, lane_intrinsic_sizing, place_lanes,
};
use lanes::{
    GridLanesLayoutInput, LaneIntrinsicTrackSizeInput, LanePlacementGeometry,
    apply_grid_lanes_auto_fit_policy, column_flow_for_grid_lanes, grid_axis_for_grid_lanes,
    lane_axis_for_grid_lanes, layout_grid_lanes_children, prepare_grid_lanes_intrinsic_child_input,
    prepare_lane_placement, resolve_grid_lanes_placement_with_resolved_tracks,
};
use named::{
    GridAreaNameFacts, NamedGridError, NamedGridLines, build_grid_named_context,
    resolve_grid_placement_or_auto_with_report, resolve_positioned_grid_placement_with_report,
    resolve_subgrid_placement,
};
pub use named::{NamedGridErrorReport, NamedGridReport};
use placement::*;
use subgrid::*;
use topology::{ExpandedGridTopology, ExpandedGridTopologyInput, TemplateAreaExpandedAxes};
use tracks::*;

pub struct GridComputationOf<S: LayoutScalar = DefaultScalar> {
    output: ComputeOutputOf<S>,
    report: GridComputationReport,
}

pub type GridComputation = GridComputationOf<DefaultScalar>;

impl<S: LayoutScalar> GridComputationOf<S> {
    pub fn output(&self) -> &ComputeOutputOf<S> {
        &self.output
    }

    pub fn report(&self) -> &GridComputationReport {
        &self.report
    }

    pub fn into_parts(self) -> (ComputeOutputOf<S>, GridComputationReport) {
        (self.output, self.report)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GridComputationReport {
    named_grid: NamedGridReport,
}

impl GridComputationReport {
    pub fn named_grid(&self) -> &NamedGridReport {
        &self.named_grid
    }

    pub fn named_grid_errors(&self) -> &[NamedGridErrorReport] {
        self.named_grid.errors()
    }

    pub fn is_empty(&self) -> bool {
        self.named_grid.is_empty()
    }

    fn merge_named_grid(&mut self, report: NamedGridReport) {
        self.named_grid.extend(report);
    }
}

pub(crate) fn compute_grid<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    Ok(compute_grid_with_report(tree, node, input)?.into_parts().0)
}

pub(crate) fn compute_grid_with_report<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridComputationOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let invocation = GridInvocation {
        container: input::project_grid_container!(tree, node),
        boundary: GridMeasurementBoundary::Ordinary,
    };
    let result = compute_grid_with_scrollbar_settlement(
        tree,
        node,
        input,
        input.settled_auto_scrollbars(),
        |_| Ok(GridParentContext::none()),
        invocation,
        GridSizingStage::COMPLETE,
    )?;
    Ok(GridComputationOf {
        output: result.output,
        report: result.report,
    })
}

pub(crate) struct GridChild<'input, Node, S: LayoutScalar> {
    phase: GridChildPhase<'input, Node, S>,
}

enum GridChildPhase<'input, Node, S: LayoutScalar> {
    Track {
        node: Node,
        input: ComputeInputOf<S>,
    },
    InFlow(GridFlowChild<'input, Node, S>),
    Positioned(PositionedGridChild<'input, Node, S>),
}

impl<Node: Copy, S: LayoutScalar> GridChild<'_, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        match &self.phase {
            GridChildPhase::Track { node, input } => (*node, *input),
            GridChildPhase::InFlow(child) => child.request(),
            GridChildPhase::Positioned(child) => (child.node, child.input),
        }
    }
}

pub(crate) struct GridContinuation<'input, Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    pass_input: ComputeInputOf<S>,
    invocation: GridInvocation<'input, S>,
    contained_size: Size<Option<S>>,
    pass: Option<Box<GridPass<'input, Node, S>>>,
    tracks: Option<Box<GridTrackPass<'input, Node, S>>>,
}

type GridStart<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, GridContinuation<'input, Node, S>>, S, M>;
type GridStep<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, GridChild<'input, Node, S>>, S, M>;

pub(crate) fn begin_grid_computation<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> GridStart<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let invocation = GridInvocation {
        container: input::project_grid_container!(tree, node),
        boundary: GridMeasurementBoundary::Ordinary,
    };
    let contained_size = match grid_containment(
        node,
        input,
        |_| Ok(GridParentContext::none()),
        invocation,
        GridSizingStage::COMPLETE,
    )? {
        ControlFlow::Break(result) => return Ok(ControlFlow::Break(result.output)),
        ControlFlow::Continue(size) => size,
    };
    Ok(ControlFlow::Continue(GridContinuation {
        node,
        input,
        pass_input: input,
        invocation,
        contained_size,
        pass: None,
        tracks: None,
    }))
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridContinuation<'input, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        (self.node, self.input)
    }

    pub(crate) fn advance<Tree, M>(&mut self, tree: &mut Tree) -> GridStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            let result = if let Some(pass) = self.pass.as_mut() {
                if let Some(child) = pass.next(tree)? {
                    return Ok(ControlFlow::Continue(child));
                }
                self.pass.take().expect("active Grid pass").finish()?
            } else if let Some(tracks) = &mut self.tracks {
                let projected = tracks.source.track_input();
                if let ControlFlow::Continue((node, input)) =
                    tracks.sizing.advance(tree, self.node, &projected)?
                {
                    return Ok(ControlFlow::Continue(GridChild {
                        phase: GridChildPhase::Track { node, input },
                    }));
                }
                let GridTrackPass { source, sizing } =
                    *self.tracks.take().expect("resolved tracks");
                match source.finish(tree, self.node, sizing)? {
                    ControlFlow::Break(result) => result,
                    ControlFlow::Continue(pass) => {
                        self.pass = Some(pass);
                        continue;
                    }
                }
            } else {
                match prepare_grid_pass(
                    tree,
                    self.node,
                    self.pass_input,
                    |_| Ok(GridParentContext::none()),
                    self.invocation,
                    GridSizingStage::COMPLETE,
                    self.contained_size,
                )? {
                    ControlFlow::Break(result) => result,
                    ControlFlow::Continue(source) => {
                        let sizing = GridTrackSizing::intrinsic();
                        self.tracks = Some(Box::new(GridTrackPass { source, sizing }));
                        continue;
                    }
                }
            };
            let Some(next) = retry_grid_scrollbars(self.node, self.pass_input, &result)? else {
                return Ok(ControlFlow::Break(result.output));
            };
            self.pass_input = next;
        }
    }

    pub(crate) fn accept_child<Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: GridChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if let GridChildPhase::Track { .. } = child.phase {
            let tracks = self.tracks.as_mut().expect("active track request");
            return tracks
                .sizing
                .accept(self.node, &tracks.source.track_input(), output);
        }
        self.pass
            .as_mut()
            .expect("active Grid pass")
            .accept(tree, child, output)
    }
}

struct GridComputeResult<Node, S: LayoutScalar = Scalar> {
    output: ComputeOutputOf<S>,
    report: GridComputationReport,
    baseline_groups: GridBaselineGroups<S>,
    effective_gutter: Option<Edges<S>>,
    executed_baselines: Option<ExecutedBaselineInventory<Node, S>>,
}

type GridComputeOutcome<Node, S, M> = LayoutResultOf<Node, GridComputeResult<Node, S>, S, M>;

impl<Node, S: LayoutScalar> GridComputeResult<Node, S> {
    fn from_output(output: ComputeOutputOf<S>) -> Self {
        Self {
            output,
            report: GridComputationReport::default(),
            baseline_groups: GridBaselineGroups {
                rows: Vec::new(),
                columns: Vec::new(),
            },
            effective_gutter: None,
            executed_baselines: None,
        }
    }
}

fn compute_grid_with_scrollbar_settlement<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    initial_state: crate::scroll::SettledAutoScrollbarState,
    mut parent_context: impl FnMut(
        &Constants<Tree::Scalar>,
    ) -> LayoutResultOf<
        <Tree as Traverse>::Node,
        GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
        Tree::Scalar,
        M,
    >,
    invocation: GridInvocation<'input, Tree::Scalar>,
    stage: GridSizingStage,
) -> GridComputeOutcome<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let initial_input = input.with_settled_auto_scrollbars(initial_state);
    let contained_size =
        match grid_containment(node, initial_input, &mut parent_context, invocation, stage)? {
            ControlFlow::Break(result) => return Ok(result),
            ControlFlow::Continue(size) => size,
        };
    let mut pass_input = initial_input;
    loop {
        let result = compute_grid_with_context_result(
            tree,
            node,
            pass_input,
            &mut parent_context,
            invocation,
            stage,
            contained_size,
        )?;
        let Some(next) = retry_grid_scrollbars(node, pass_input, &result)? else {
            return Ok(result);
        };
        pass_input = next;
    }
}

type GridContainmentResult<Node, S, M> =
    LayoutResultOf<Node, ControlFlow<GridComputeResult<Node, S>, Size<Option<S>>>, S, M>;

fn grid_containment<'input, Node, S, M>(
    node: Node,
    input: ComputeInputOf<S>,
    parent_context: impl FnMut(&Constants<S>) -> LayoutResultOf<Node, GridParentContext<S, Node>, S, M>,
    invocation: GridInvocation<'input, S>,
    stage: GridSizingStage,
) -> GridContainmentResult<Node, S, M>
where
    Node: Copy + Eq + 'input,
    S: LayoutScalar,
{
    let style = invocation.container.source();
    let containment = size_containment(style);
    if containment == crate::SizeContainment::None {
        return Ok(ControlFlow::Continue(Size::NONE));
    }
    let flow_axes = crate::FlowAxes::new(style.writing_mode, style.direction);
    let empty = compute_empty_grid(node, input, parent_context, invocation, stage)?;
    if containment == crate::SizeContainment::BothAxes
        && !input.run_mode().is_perform_layout()
        && !input.physical_baseline_demand()
    {
        return Ok(ControlFlow::Break(empty));
    }
    Ok(ControlFlow::Continue(contained_outer_size(
        containment,
        flow_axes,
        empty.output.size,
    )))
}

fn retry_grid_scrollbars<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    input: ComputeInputOf<S>,
    result: &GridComputeResult<Node, S>,
) -> LayoutResultOf<Node, Option<ComputeInputOf<S>>, S, M> {
    let Some(geometry) = result.output.scroll_geometry else {
        return Ok(None);
    };
    let current = input.settled_auto_scrollbars();
    let next = current.transition(geometry);
    if next == current
        || !crate::scroll::settled_auto_scrollbars_change_available_geometry(geometry, next)
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
    {
        return Ok(None);
    }
    Ok(Some(input.with_settled_auto_scrollbars(next)))
}

fn compute_empty_grid<'input, Node, S, M>(
    node: Node,
    input: ComputeInputOf<S>,
    parent_context: impl FnMut(&Constants<S>) -> LayoutResultOf<Node, GridParentContext<S, Node>, S, M>,
    invocation: GridInvocation<'input, S>,
    stage: GridSizingStage,
) -> GridComputeOutcome<Node, S, M>
where
    Node: Copy + Eq + 'input,
    S: LayoutScalar,
{
    compute_empty_box(node, invocation.container.source(), |probe| {
        compute_grid_with_context_result(
            probe,
            node,
            empty_sizing_input(input),
            parent_context,
            invocation,
            stage,
            Size::NONE,
        )
    })
}

// The resolved child-layout source is borrowed for one observation only. It
// preserves contextual tracks, baselines and the child's own scrollbar pass.
pub(crate) struct GridOverflowProbe<'input, 'a, Node, S: LayoutScalar> {
    layout: GridOverflowLayout<'input, 'a, Node, S>,
}

enum GridOverflowLayout<'input, 'a, Node, S: LayoutScalar> {
    Ordinary(GridChildLayoutInput<'input, 'a, Node, S>),
    Lanes(
        GridLanesLayoutInput<'input, 'a, Node, S>,
        GridContainerContext<S>,
    ),
    Intrinsic(GridIntrinsicProbe<'input, 'a, Node, S>),
    #[cfg(test)]
    RootPrefix {
        input: ComputeInputOf<S>,
        stage: GridSizingStage,
    },
}

pub(crate) enum GridOverflowObservation<S: LayoutScalar> {
    Children(GridChildrenObservation<S>),
    Intrinsic(intrinsic_stage::GridIntrinsicObservation<S>),
}

pub(crate) struct GridChildrenObservation<S: LayoutScalar> {
    visible_content_size: Size<S>,
    contributions: crate::scroll::ScrollContributionAccumulatorOf<S>,
    baselines: BaselinesOf<S>,
}

impl<'input, Node, S: LayoutScalar> GridOverflowProbe<'input, '_, Node, S> {
    pub(crate) fn observe<Tree, M>(
        self,
        tree: &mut Tree,
        node: Node,
    ) -> LayoutResultOf<Node, GridOverflowObservation<S>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S> + Traverse<Node = Node>,
        Node: Copy + Eq,
    {
        let children = match self.layout {
            GridOverflowLayout::Ordinary(input) => {
                layout_grid_container_children(tree, node, input)?.compute(tree)?
            }
            GridOverflowLayout::Lanes(input, context) => {
                match layout_grid_lanes_children(tree, node, input, context)? {
                    ControlFlow::Break(layout) => layout,
                    ControlFlow::Continue(placement) => placement.compute(tree)?,
                }
            }
            GridOverflowLayout::Intrinsic(input) => {
                return input
                    .observe(tree, node)
                    .map(GridOverflowObservation::Intrinsic);
            }
            #[cfg(test)]
            GridOverflowLayout::RootPrefix { input, stage } => {
                let invocation = GridInvocation {
                    container: input::project_grid_container!(tree, node),
                    boundary: GridMeasurementBoundary::Ordinary,
                };
                let result = compute_grid_with_scrollbar_settlement(
                    tree,
                    node,
                    input,
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                    |_| Ok(GridParentContext::none()),
                    invocation,
                    stage,
                )?;
                return Ok(GridOverflowObservation::Intrinsic(
                    intrinsic_stage::GridIntrinsicObservation {
                        output: result.output,
                        effective_gutter: result.effective_gutter,
                    },
                ));
            }
        };
        Ok(GridOverflowObservation::Children(GridChildrenObservation {
            visible_content_size: children.visible_content_size,
            contributions: children.contributions,
            baselines: children.baselines,
        }))
    }
}

fn observe_grid_child_layout<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    layout: GridOverflowLayout<'input, '_, Tree::Node, Tree::Scalar>,
) -> GridChildrenLayoutResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let GridOverflowObservation::Children(observation) =
        tree.observe_grid_overflow(node, GridOverflowProbe { layout })?
    else {
        unreachable!("a child-layout probe returns its child observation");
    };
    Ok(GridChildrenLayout {
        visible_content_size: observation.visible_content_size,
        contributions: observation.contributions,
        baselines: observation.baselines,
        baseline_groups: GridBaselineGroups {
            rows: Vec::new(),
            columns: Vec::new(),
        },
        executed_baselines: None,
    })
}

fn needs_intrinsic_auto_observation<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    input: ComputeInputOf<S>,
) -> bool {
    input.run_mode() == RunMode::ComputeSize
        && style.scrollbar_width.get() > S::ZERO
        && (style.overflow.x() == Overflow::Auto || style.overflow.y() == Overflow::Auto)
}

fn sizing_algorithm_for_grid_display(display: Display) -> SizingAlgorithm {
    if display.inner_display() == Display::GridLanes {
        SizingAlgorithm::GridLanes
    } else {
        SizingAlgorithm::Grid
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StandaloneIntrinsicMinimum {
    MinContent,
    MaxContent,
}

impl StandaloneIntrinsicMinimum {
    fn from_minimum<S: LayoutScalar>(minimum: &MinSizeOf<S>) -> Option<Self> {
        if minimum.is_min_content() {
            Some(Self::MinContent)
        } else if minimum.is_max_content() {
            Some(Self::MaxContent)
        } else {
            None
        }
    }

    const fn operation(self) -> SizingOperation {
        match self {
            Self::MinContent => SizingOperation::MinContent,
            Self::MaxContent => SizingOperation::MaxContent,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GridInvocation<'input, S: LayoutScalar> {
    container: GridContainerInput<'input, S>,
    boundary: GridMeasurementBoundary<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GridMeasurementBoundary<S: LayoutScalar> {
    Ordinary,
    StandaloneIntrinsicMinimumProbe { axes: Size<bool> },
    Standalone { minimum: Size<Option<S>> },
}

fn standalone_intrinsic_minimum_phases<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    inherited_axes: LogicalSizeOf<bool>,
) -> Size<Option<StandaloneIntrinsicMinimum>> {
    let style_flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
    let standalone_physical_axes = style_flow_axes.physical_size(LogicalSizeOf::new(
        !inherited_axes.inline,
        !inherited_axes.block,
    ));
    Size::new(
        standalone_physical_axes
            .width
            .then(|| {
                style.with_minimum(
                    PhysicalAxis::Horizontal,
                    StandaloneIntrinsicMinimum::from_minimum,
                )
            })
            .flatten(),
        standalone_physical_axes
            .height
            .then(|| {
                style.with_minimum(
                    PhysicalAxis::Vertical,
                    StandaloneIntrinsicMinimum::from_minimum,
                )
            })
            .flatten(),
    )
}

fn intrinsic_container_available<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    parent: Size<Option<S>>,
    available: Size<AxisSizingRequestOf<S>>,
    sizing_mode: SizingMode,
) -> Result<LogicalSizeOf<AxisSizingRequestOf<S>>, SizingResolutionError<S>> {
    if sizing_mode != SizingMode::InherentSize {
        return Ok(sizing_flow_axes.logical_size(available));
    }
    let algorithm = sizing_algorithm_for_grid_display(style.display);
    let style_size = sizing_flow_axes.logical_size(Size::new(
        resolve_preferred_sizing(
            &style.size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            parent.width,
            true,
        )?,
        resolve_preferred_sizing(
            &style.size.height,
            algorithm,
            PhysicalAxis::Vertical,
            parent.height,
            true,
        )?,
    ));
    let available = sizing_flow_axes.logical_size(available);
    Ok(LogicalSizeOf::new(
        intrinsic_available_for_dimension(style_size.inline)
            .map_or(available.inline, |operation| {
                available.inline.with_operation(operation)
            }),
        intrinsic_available_for_dimension(style_size.block).map_or(available.block, |operation| {
            available.block.with_operation(operation)
        }),
    ))
}

fn intrinsic_available_for_dimension<S: LayoutScalar>(
    dimension: ResolvedPreferredSize<S>,
) -> Option<SizingOperation> {
    match dimension {
        ResolvedPreferredSize::MinContent => Some(SizingOperation::MinContent),
        ResolvedPreferredSize::MaxContent => Some(SizingOperation::MaxContent),
        ResolvedPreferredSize::Auto | ResolvedPreferredSize::Definite(_) => None,
    }
}

fn intrinsic_available_size_for_axis<S: LayoutScalar>(
    available: AxisSizingRequestOf<S>,
    max_size: Option<S>,
) -> Option<S> {
    available
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get)
        .or_else(|| {
            (available.operation() == SizingOperation::MaxContent)
                .then_some(max_size)
                .flatten()
        })
}

#[cfg(test)]
fn compute_grid_with_context<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let container =
        input::project_grid_container!(tree, node).with_inherited_axes(LogicalSizeOf::new(
            parent_context.columns.is_some(),
            parent_context.rows.is_some(),
        ));
    Ok(compute_grid_with_context_source(
        tree,
        node,
        input,
        Size::NONE,
        container,
        GridSizingStage::COMPLETE,
        |_| Ok(parent_context.clone()),
    )?
    .output)
}

// Rebuild inherited tracks from the unchanged parent source for each child pass.
// A parent's auto-scrollbar pass is not this child's settled scrollbar state.
fn compute_grid_with_context_source<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
    container: GridContainerInput<'input, Tree::Scalar>,
    stage: GridSizingStage,
    mut parent_context: impl FnMut(
        &Constants<Tree::Scalar>,
    ) -> LayoutResultOf<
        <Tree as Traverse>::Node,
        GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
        Tree::Scalar,
        M,
    >,
) -> GridComputeOutcome<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style_input = container;
    let inherited_axes = container.inherited_axes();
    let contextual_minimum =
        standalone_intrinsic_minimum_phases(&style_input.projection(), inherited_axes);
    let intrinsic_minimum = Size::new(
        intrinsic_minimum.width.or(contextual_minimum.width),
        intrinsic_minimum.height.or(contextual_minimum.height),
    );
    let intrinsic_axes = intrinsic_minimum.map(|minimum| minimum.is_some());
    let resolved_intrinsic_minimum = if intrinsic_axes.width || intrinsic_axes.height {
        let available = Size::new(
            intrinsic_minimum
                .width
                .map_or(input.sizing_request().axes().width, |phase| {
                    input
                        .sizing_request()
                        .axes()
                        .width
                        .with_operation(phase.operation())
                }),
            intrinsic_minimum
                .height
                .map_or(input.sizing_request().axes().height, |phase| {
                    input
                        .sizing_request()
                        .axes()
                        .height
                        .with_operation(phase.operation())
                }),
        );
        let known = Size::new(
            if intrinsic_axes.width {
                None
            } else {
                input.known().width
            },
            if intrinsic_axes.height {
                None
            } else {
                input.known().height
            },
        );
        let probe_input = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            input.sizing_mode(),
            input.requested_axis(),
            known,
            input.parent(),
            input.containing_layout_context(),
            input.sizing_request().with_axes(available),
        )
        .with_settled_auto_scrollbars(input.settled_auto_scrollbars())
        .with_containing_auto_scrollbar_pass(input.containing_auto_scrollbar_pass())
        .with_baseline_demand_from(input);
        let measured = compute_grid_with_scrollbar_settlement(
            tree,
            node,
            probe_input,
            crate::scroll::SettledAutoScrollbarState::INITIAL,
            &mut parent_context,
            GridInvocation {
                container,
                boundary: GridMeasurementBoundary::StandaloneIntrinsicMinimumProbe {
                    axes: intrinsic_axes,
                },
            },
            stage,
        )?
        .output
        .size;
        Size::new(
            intrinsic_axes.width.then_some(measured.width),
            intrinsic_axes.height.then_some(measured.height),
        )
    } else {
        Size::NONE
    };
    compute_grid_with_scrollbar_settlement(
        tree,
        node,
        input,
        crate::scroll::SettledAutoScrollbarState::INITIAL,
        parent_context,
        GridInvocation {
            container,
            boundary: GridMeasurementBoundary::Standalone {
                minimum: resolved_intrinsic_minimum,
            },
        },
        stage,
    )
}

struct GridPass<'input, Node, S: LayoutScalar> {
    children: Option<GridPassChildren<'input, Node, S>>,
    output: GridPassOutput<'input, Node, S>,
    scroll_box: CanonicalScrollBoxOf<S>,
}

enum GridPassChildren<'input, Node, S: LayoutScalar> {
    InFlowLanes(Box<GridLanesInFlow<'input, Node, S>>),
    InFlowOrdinary(Box<OrdinaryGridInFlow<'input, Node, S>>),
    Positioned(GridChildPlacement<'input, Node, S>),
}

struct OrdinaryGridInFlow<'input, Node, S: LayoutScalar> {
    placement: OrdinaryGridPlacement<Node, S>,
    source: OrdinaryGridSource<'input, Node, S>,
    collection: GridItemCollection<'input, Node, S>,
}

struct OrdinaryGridSource<'input, Node, S: LayoutScalar> {
    tracks: OrdinaryGridTracks<S>,
    column_tracks: Vec<TrackSizingOf<S>>,
    row_tracks: Vec<TrackSizingOf<S>>,
    parent_context: GridParentContext<S, Node>,
    subgrid_report: GridSubgridReport<Node>,
    placements: GridPlacementContext<'input, Node, S>,
    sizing_stage: GridSizingStage,
}

impl<'input, Node, S: LayoutScalar> OrdinaryGridSource<'input, Node, S> {
    fn context<'a>(
        &'a self,
        pass: &'a GridPassOutput<'input, Node, S>,
        scroll_box: CanonicalScrollBoxOf<S>,
    ) -> GridLayoutContext<'input, 'a, Node, S> {
        self.tracks.placement_context(OrdinaryGridPlacementContext {
            execution_input: pass.input,
            scroll_box,
            style: &pass.style,
            row_tracks: &self.row_tracks,
            minimum_functions: minimum_parent_functions(
                &self.column_tracks,
                &self.row_tracks,
                self.tracks.track_basis,
                &self.parent_context,
            ),
            subgrid_report: &self.subgrid_report,
            parent_context: &self.parent_context,
            placements: &self.placements,
            sizing_stage: self.sizing_stage,
        })
    }
}

struct GridLanesInFlow<'input, Node, S: LayoutScalar> {
    geometry: LanePlacementGeometry<S>,
    source: GridLanesPlacementSource<'input, Node, S>,
    collection: GridItemCollection<'input, Node, S>,
}

struct GridLanesPlacementSource<'input, Node, S: LayoutScalar> {
    children: Vec<Node>,
    columns: Vec<S>,
    rows: Vec<S>,
    column_tracks: Vec<TrackSizingOf<S>>,
    row_tracks: Vec<TrackSizingOf<S>>,
    parent_context: GridParentContext<S, Node>,
    subgrid_report: GridSubgridReport<Node>,
    placements: GridPlacementContext<'input, Node, S>,
    content_box_size: LogicalSizeOf<S>,
    track_basis: LogicalSizeOf<Option<S>>,
    gap: LogicalSizeOf<S>,
    sizing_stage: GridSizingStage,
    stacking_operation: SizingOperation,
}

impl<'input, Node, S: LayoutScalar> GridLanesPlacementSource<'input, Node, S> {
    fn input<'a>(
        &'a self,
        pass: &'a GridPassOutput<'input, Node, S>,
        scroll_box: CanonicalScrollBoxOf<S>,
    ) -> GridLanesLayoutInput<'input, 'a, Node, S> {
        GridLanesLayoutInput {
            scroll_box,
            style: &pass.style,
            constants: &pass.constants,
            minimum_functions: minimum_parent_functions(
                &self.column_tracks,
                &self.row_tracks,
                self.track_basis,
                &self.parent_context,
            ),
            container_content_box_size: self.content_box_size,
            columns: &self.columns,
            rows: &self.rows,
            row_tracks: &self.row_tracks,
            track_basis: self.track_basis,
            parent_context: &self.parent_context,
            gap: self.gap,
            subgrid_report: &self.subgrid_report,
            placements: &self.placements,
            containing_auto_scrollbar_pass: pass.input.settled_auto_scrollbars(),
            sizing_stage: self.sizing_stage,
            stacking_operation: self.stacking_operation,
            execution_input: pass.input,
        }
    }
}

type GridPassStart<'input, Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<GridComputeResult<Node, S>, Box<GridPass<'input, Node, S>>>,
    S,
    M,
>;

type GridPassStep<'input, Node, S, M> =
    LayoutResultOf<Node, Option<GridChild<'input, Node, S>>, S, M>;

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridPass<'input, Node, S> {
    fn compute<Tree, M>(mut self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        while let Some(child) = self.next(tree)? {
            let (node, input) = child.request();
            let output = tree.compute_child(node, input)?;
            self.accept(tree, child, output)?;
        }
        self.finish()
    }

    fn next<Tree, M>(&mut self, tree: &mut Tree) -> GridPassStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if let Some(GridPassChildren::InFlowLanes(lanes)) = &mut self.children {
            let input = lanes.source.input(&self.output, self.scroll_box);
            let geometry = &lanes.geometry;
            if let Some(child) = lanes.collection.next(
                tree,
                self.output.node,
                &geometry.item_input(&input, &lanes.source.children),
                |source_index, style| geometry.item_disposition(&input, source_index, style),
            )? {
                return Ok(Some(GridChild {
                    phase: GridChildPhase::InFlow(child),
                }));
            }
            let Some(GridPassChildren::InFlowLanes(lanes)) = self.children.take() else {
                unreachable!()
            };
            let GridLanesInFlow {
                geometry,
                source,
                collection,
            } = *lanes;
            let input = source.input(&self.output, self.scroll_box);
            let children = geometry.finish(tree, self.output.node, &input, collection.finish())?;
            self.children = Some(GridPassChildren::Positioned(children));
        }
        if let Some(GridPassChildren::InFlowOrdinary(grid)) = &mut self.children {
            let context = grid.source.context(&self.output, self.scroll_box);
            let placement = &grid.placement;
            if let Some(child) = grid.collection.next(
                tree,
                self.output.node,
                &placement.item_input(&context),
                |source_index, _| placement.item_disposition(&context, source_index),
            )? {
                return Ok(Some(GridChild {
                    phase: GridChildPhase::InFlow(child),
                }));
            }
            let Some(GridPassChildren::InFlowOrdinary(grid)) = self.children.take() else {
                unreachable!()
            };
            let OrdinaryGridInFlow {
                placement,
                source,
                collection,
            } = *grid;
            let context = source.context(&self.output, self.scroll_box);
            let children = source.tracks.finish(
                tree,
                self.output.node,
                placement,
                &context,
                collection.finish(),
            )?;
            self.children = Some(GridPassChildren::Positioned(children));
        }
        let Some(GridPassChildren::Positioned(children)) = &mut self.children else {
            unreachable!("Grid placement has one active child phase");
        };
        Ok(children.next(tree)?.map(|child| GridChild {
            phase: GridChildPhase::Positioned(child),
        }))
    }

    fn accept<Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: GridChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match (&mut self.children, child.phase) {
            (Some(GridPassChildren::InFlowLanes(lanes)), GridChildPhase::InFlow(child)) => {
                let input = lanes.source.input(&self.output, self.scroll_box);
                lanes.collection.accept(
                    tree,
                    self.output.node,
                    &lanes.geometry.item_input(&input, &lanes.source.children),
                    child,
                    output,
                )
            }
            (Some(GridPassChildren::InFlowOrdinary(grid)), GridChildPhase::InFlow(child)) => {
                let context = grid.source.context(&self.output, self.scroll_box);
                grid.collection.accept(
                    tree,
                    self.output.node,
                    &grid.placement.item_input(&context),
                    child,
                    output,
                )
            }
            (Some(GridPassChildren::Positioned(children)), GridChildPhase::Positioned(child)) => {
                children.accept(tree, child, output)
            }
            _ => unreachable!("Grid child receipt belongs to the active placement phase"),
        }
    }

    fn finish<M>(self) -> GridComputeOutcome<Node, S, M> {
        let Some(GridPassChildren::Positioned(children)) = self.children else {
            unreachable!("Grid finishes after all child placement");
        };
        let layout = children.finish()?;
        self.output.finish(Some((self.scroll_box, layout)))
    }
}

fn compute_grid_with_context_result<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: impl FnMut(
        &Constants<Tree::Scalar>,
    ) -> LayoutResultOf<
        Tree::Node,
        GridParentContext<Tree::Scalar, Tree::Node>,
        Tree::Scalar,
        M,
    >,
    invocation: GridInvocation<'input, Tree::Scalar>,
    stage: GridSizingStage,
    contained_size: Size<Option<Tree::Scalar>>,
) -> GridComputeOutcome<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let source = match prepare_grid_pass(
        tree,
        node,
        input,
        parent_context,
        invocation,
        stage,
        contained_size,
    )? {
        ControlFlow::Break(result) => return Ok(result),
        ControlFlow::Continue(source) => source,
    };
    let projected = source.track_input();
    let mut sizing = GridTrackSizing::intrinsic();
    while let ControlFlow::Continue((child, request)) = sizing.advance(tree, node, &projected)? {
        let output = tree.compute_child(child, request)?;
        sizing.accept(node, &projected, output)?;
    }
    match source.finish(tree, node, sizing)? {
        ControlFlow::Break(result) => Ok(result),
        ControlFlow::Continue(pass) => pass.compute(tree),
    }
}

struct GridTrackPass<'input, Node, S: LayoutScalar> {
    source: GridPassSource<'input, Node, S>,
    sizing: GridTrackSizing<'static, Node, S>,
}

struct GridPassSource<'input, Node, S: LayoutScalar> {
    input: ComputeInputOf<S>,
    stage: GridSizingStage,
    contained_size: Size<Option<S>>,
    style: GridContainerProjection<'input, S>,
    constants: Constants<S>,
    parent_context: GridParentContext<S, Node>,
    initialized: InitializedGridTracks<'input, Node, S>,
    prepared_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    observes_contained_baseline: bool,
}

type GridPassPreparation<'input, Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<GridComputeResult<Node, S>, GridPassSource<'input, Node, S>>,
    S,
    M,
>;

fn prepare_grid_pass<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
    mut parent_context: impl FnMut(
        &Constants<Tree::Scalar>,
    ) -> LayoutResultOf<
        <Tree as Traverse>::Node,
        GridParentContext<Tree::Scalar, <Tree as Traverse>::Node>,
        Tree::Scalar,
        M,
    >,
    invocation: GridInvocation<'input, Tree::Scalar>,
    stage: GridSizingStage,
    contained_size: Size<Option<Tree::Scalar>>,
) -> GridPassPreparation<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut style_input = invocation.container;
    let standalone_intrinsic_minimum = match invocation.boundary {
        GridMeasurementBoundary::Ordinary => Size::NONE,
        GridMeasurementBoundary::StandaloneIntrinsicMinimumProbe { axes } => {
            style_input.suppress_intrinsic_minimum(axes);
            Size::NONE
        }
        GridMeasurementBoundary::Standalone { minimum } => minimum,
    };
    let style = style_input.projection();
    let constants = Constants::with_intrinsic_minimum::<Tree, M>(
        tree,
        node,
        &style,
        input,
        standalone_intrinsic_minimum,
    )?;

    let parent_context = parent_context(&constants)?;
    let mut style = style;
    style.baseline_references =
        effective_grid_baseline_references(style.baseline_references, &parent_context);

    let observes_contained_baseline =
        contained_size != Size::NONE && input.physical_baseline_demand();
    if input.run_mode() == RunMode::ComputeSize
        && !observes_contained_baseline
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        return Ok(ControlFlow::Break(GridComputeResult::from_output(
            ComputeOutputOf::from_outer_size(Size::new(width, height)),
        )));
    }

    let stacking_axis = style
        .display
        .establishes_grid_lanes_formatting_context()
        .then(|| lane_axis_for_grid_lanes(&style));
    let contained_inline = constants.contained_inline_basis(contained_size, stage);
    let initialized_tracks = initialize_grid_tracks::<Tree, M>(
        tree,
        node,
        &style,
        &constants,
        &parent_context,
        input,
        contained_inline,
    )?;
    let constants = constants.for_inline_track_basis(contained_inline);
    let InitializedGridTracks {
        column_tracks,
        row_tracks,
        context,
        ..
    } = &initialized_tracks;
    if stacking_axis.is_none()
        && input.requires_fixed_grid_tracks()
        && !column_tracks
            .iter()
            .chain(row_tracks)
            .all(TrackSizingOf::is_fixed_without_basis)
    {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(
                crate::RootContextMismatch::WritingMode,
            )),
        ));
    }
    if stacking_axis.is_none() {
        debug_assert_eq!(
            initialized_tracks.subgrid_report.items.len(),
            tree.child_count(node)
        );
        let lines = &context.lines;
        debug_assert_eq!(lines.column_explicit_start, context.leading_columns);
        debug_assert_eq!(lines.column_explicit_count, context.explicit_columns);
        debug_assert_eq!(lines.row_explicit_start, context.leading_rows);
        debug_assert_eq!(lines.row_explicit_count, context.explicit_rows);
    }
    let mut source = GridPassSource {
        input,
        stage,
        contained_size,
        style,
        constants,
        parent_context,
        prepared_available: initialized_tracks.track_available,
        initialized: initialized_tracks,
        observes_contained_baseline,
    };
    let track_input = source.track_input();
    source.prepared_available = LogicalSizeOf::new(
        intrinsic_stage::axis_track_available(node, &track_input, GridAxisKind::Column)?,
        intrinsic_stage::axis_track_available(node, &track_input, GridAxisKind::Row)?,
    );
    Ok(ControlFlow::Continue(source))
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridPassSource<'input, Node, S> {
    fn track_input(&self) -> GridTrackResolutionInput<'input, '_, Node, S> {
        let InitializedGridTracks {
            column_tracks,
            row_tracks,
            track_available,
            context,
            placements,
            subgrid_report,
            ..
        } = &self.initialized;
        GridTrackResolutionInput {
            style: &self.style,
            constants: &self.constants,
            column_tracks,
            row_tracks,
            minimum_functions: minimum_parent_functions(
                column_tracks,
                row_tracks,
                context.percent_basis,
                &self.parent_context,
            ),
            context: std::borrow::Cow::Borrowed(context),
            subgrid_report,
            sizing_flow_axes: self.constants.flow_axes,
            available: *track_available,
            placements,
            stage: self.stage,
        }
    }

    fn finish<Tree, M>(
        self,
        tree: &mut Tree,
        node: Node,
        sizing: GridTrackSizing<'_, Node, S>,
    ) -> GridPassStart<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            input,
            stage,
            contained_size,
            style,
            constants,
            parent_context,
            initialized,
            prepared_available,
            observes_contained_baseline,
        } = self;
        let InitializedGridTracks {
            column_tracks,
            row_tracks,
            track_available,
            context,
            placements,
            subgrid_report,
            report,
        } = initialized;
        let stacking_axis = style
            .display
            .establishes_grid_lanes_formatting_context()
            .then(|| lane_axis_for_grid_lanes(&style));
        let sizing_flow_axes = constants.flow_axes;
        let gap = context.gap;
        let stacking = stacking_axis.map(|axis| {
            let fixed_body =
                sizing_flow_axes.logical_size(constants.explicit_definite_content_size);
            let operation = if axis.component(fixed_body).is_some() {
                SizingOperation::Normal
            } else {
                axis.component(track_available).operation()
            };
            (axis, operation)
        });
        let minimum_functions = minimum_parent_functions(
            &column_tracks,
            &row_tracks,
            context.percent_basis,
            &parent_context,
        );
        let track_input = GridTrackResolutionInput {
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            context: std::borrow::Cow::Owned(context.clone()),
            subgrid_report: &subgrid_report,
            sizing_flow_axes,
            available: track_available,
            placements: &placements,
            stage,
        };
        let track_resolution = sizing.finish(node, track_input)?;
        let GridTrackResolution {
            sizing_phases,
            columns,
            rows,
            column_preparation,
            row_preparation,
            source_frame,
        } = track_resolution;
        let track_content_size = LogicalSizeOf::new(
            track_sum_with_gutters(&columns, gap.inline, Some(&context.column_gutters)),
            track_sum_with_gutters(&rows, gap.block, Some(&context.row_gutters)),
        );
        let mut logical_content_size = track_content_size;
        let needs_child_layout = (input.run_mode().is_perform_layout()
            || needs_intrinsic_auto_observation(&style, input)
            || observes_contained_baseline)
            && intrinsic_stage::has_layout_basis(stage, &constants, &column_tracks, &row_tracks);
        // Grid-axis contribution sizing does not consume stacking placement. Its
        // range is needed only at the reached stacking sizing point or when the
        // qualified child-layout operation can consume an actual scrollport.
        if let Some((stacking_axis, stacking_operation)) = stacking
            && stacking_axis
                .component(sizing_flow_axes.logical_size(contained_size))
                .is_none()
            && (stage.has_resolved_axis(stacking_axis) || needs_child_layout)
            && let Ok(lane_report) = resolve_grid_lanes_placement_with_resolved_tracks(
                tree,
                node,
                &style,
                &constants,
                context.clone(),
                &columns,
                &rows,
                &placements,
                match grid_axis_for_grid_lanes(&style) {
                    GridAxisKind::Column => gap.inline,
                    GridAxisKind::Row => gap.block,
                },
                sizing_flow_axes.logical_size(constants.node_inner_size),
                stacking_operation,
                minimum_functions,
                &subgrid_report,
                stage,
            )?
        {
            let extent = match stacking_axis {
                GridAxisKind::Column => &mut logical_content_size.inline,
                GridAxisKind::Row => &mut logical_content_size.block,
            };
            *extent = lane_report.range_extent().max(S::ZERO);
        }
        let intrinsic_sizing_content_size = if stacking.is_some() {
            logical_content_size
        } else {
            let logical_style_size =
                sizing_flow_axes.logical_size(Size::new(&style.size.width, &style.size.height));
            let logical_node_inner_size = sizing_flow_axes.logical_size(constants.node_inner_size);
            let logical_available_inner_size =
                sizing_flow_axes.logical_size(constants.available_inner_size);
            LogicalSizeOf::new(
                intrinsic_sizing_axis_content_size(IntrinsicSizingAxisInput {
                    operation: prepared_available.inline.operation(),
                    run_mode: input.run_mode(),
                    style_size: logical_style_size.inline,
                    content_size: track_content_size.inline,
                    track_content_size: track_content_size.inline,
                    definite_size: logical_node_inner_size.inline,
                    available_size: logical_available_inner_size.inline,
                    tracks: &column_tracks,
                }),
                intrinsic_sizing_axis_content_size(IntrinsicSizingAxisInput {
                    operation: prepared_available.block.operation(),
                    run_mode: input.run_mode(),
                    style_size: logical_style_size.block,
                    content_size: track_content_size.block,
                    track_content_size: track_content_size.block,
                    definite_size: logical_node_inner_size.block,
                    available_size: logical_available_inner_size.block,
                    tracks: &row_tracks,
                }),
            )
        };
        let padding_border_size = (constants.padding + constants.border).sum_axes();
        let intrinsic_sizing_physical_size = constants
            .flow_axes
            .physical_size(intrinsic_sizing_content_size);
        let intrinsic_outer_size = (intrinsic_sizing_physical_size
            + constants.content_box_inset.sum_axes())
        .clamp_min_before_max_optional(constants.node_min_size, constants.node_max_size)
        .max(padding_border_size);
        let output_size = contained_size
            .or(input.known())
            .or(constants.node_outer_size)
            .unwrap_or(intrinsic_outer_size)
            .max(padding_border_size);
        let mut content_size = if contained_size.width.is_some() && contained_size.height.is_some()
        {
            Size::ZERO
        } else {
            sizing_flow_axes.physical_size(logical_content_size)
        };

        if !needs_child_layout {
            return GridPassOutput {
                node,
                input,
                style,
                constants,
                output_size,
                content_size,
                report,
                observes_contained_baseline,
            }
            .finish(None)
            .map(ControlFlow::Break);
        }

        let scroll_box = canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
            flow_axes: constants.flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            border_box_size: output_size,
            border: constants.border,
            padding: constants.padding,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
        })
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let constants = constants.for_used_scroll_box(scroll_box);
        let child_input = GridChildLayoutInput {
            execution_input: input,
            scroll_box,
            sizing_phases,
            source_frame: &source_frame,
            style: &style,
            constants: &constants,
            column_tracks: &column_tracks,
            row_tracks: &row_tracks,
            minimum_functions,
            context,
            columns: &columns,
            rows: &rows,
            column_preparation: column_preparation.as_deref(),
            row_preparation: row_preparation.as_deref(),
            prepared_available,
            output_size,
            subgrid_report: &subgrid_report,
            parent_context: &parent_context,
            placements: &placements,
        };
        let (child_layout, constants) = if let Some((_, stacking_operation)) = stacking {
            let layout_content_box_size =
                (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
            let logical_layout_content_box_size =
                sizing_flow_axes.logical_size(layout_content_box_size);
            let FinalGridLayoutTracks {
                columns: layout_columns,
                rows: layout_rows,
                gap: layout_gap,
                column_gutters: layout_column_gutters,
                row_gutters: layout_row_gutters,
                percent_basis: layout_percent_basis,
                minimum_functions,
            } = final_grid_layout_tracks(tree, node, &child_input)?;
            let context = child_input.context;
            let context = GridContainerContext {
                gap: layout_gap,
                percent_basis: layout_percent_basis,
                column_gutters: layout_column_gutters.unwrap_or(context.column_gutters),
                row_gutters: layout_row_gutters.unwrap_or(context.row_gutters),
                ..context
            };
            if contained_size.width.is_some() && contained_size.height.is_some() {
                content_size = sizing_flow_axes.physical_size(LogicalSizeOf::new(
                    track_sum_with_gutters(
                        &layout_columns,
                        layout_gap.inline,
                        Some(&context.column_gutters),
                    ),
                    track_sum_with_gutters(
                        &layout_rows,
                        layout_gap.block,
                        Some(&context.row_gutters),
                    ),
                ));
            }
            let constants = constants.for_contained_placement(
                size_containment(tree.node_input(node)),
                output_size,
                layout_content_box_size,
            );
            let lanes_input = GridLanesLayoutInput {
                scroll_box,
                execution_input: input,
                style: &style,
                constants: &constants,
                minimum_functions,
                container_content_box_size: logical_layout_content_box_size,
                columns: &layout_columns,
                rows: &layout_rows,
                row_tracks: &row_tracks,
                track_basis: layout_percent_basis,
                parent_context: &parent_context,
                gap: layout_gap,
                subgrid_report: &subgrid_report,
                placements: &placements,
                containing_auto_scrollbar_pass: input.settled_auto_scrollbars(),
                sizing_stage: sizing_phases.limit,
                stacking_operation,
            };
            if input.run_mode().is_perform_layout() {
                let prepared = prepare_lane_placement(tree, node, &lanes_input, context)?;
                let output = GridPassOutput {
                    node,
                    input,
                    style,
                    constants,
                    output_size,
                    content_size,
                    report,
                    observes_contained_baseline,
                };
                return match prepared {
                    ControlFlow::Break(layout) => output
                        .finish(Some((scroll_box, layout)))
                        .map(ControlFlow::Break),
                    ControlFlow::Continue(geometry) => {
                        let children = tree.children(node).collect::<Vec<_>>();
                        let collection =
                            GridItemCollection::new(&geometry.item_input(&lanes_input, &children));
                        let source = GridLanesPlacementSource {
                            children,
                            columns: layout_columns,
                            rows: layout_rows,
                            column_tracks,
                            row_tracks,
                            parent_context,
                            subgrid_report,
                            placements,
                            content_box_size: logical_layout_content_box_size,
                            track_basis: layout_percent_basis,
                            gap: layout_gap,
                            sizing_stage: sizing_phases.limit,
                            stacking_operation,
                        };
                        Ok(ControlFlow::Continue(Box::new(GridPass {
                            children: Some(GridPassChildren::InFlowLanes(Box::new(
                                GridLanesInFlow {
                                    geometry,
                                    source,
                                    collection,
                                },
                            ))),
                            output,
                            scroll_box,
                        })))
                    }
                };
            }
            (
                observe_grid_child_layout(
                    tree,
                    node,
                    GridOverflowLayout::Lanes(lanes_input, context),
                )?,
                constants,
            )
        } else if input.run_mode().is_perform_layout() {
            let (tracks, minimum_functions) =
                prepare_ordinary_grid_tracks(tree, node, child_input)?;
            let placement_context = tracks.placement_context(OrdinaryGridPlacementContext {
                execution_input: input,
                scroll_box,
                style: &style,
                row_tracks: &row_tracks,
                minimum_functions,
                subgrid_report: &subgrid_report,
                parent_context: &parent_context,
                placements: &placements,
                sizing_stage: sizing_phases.limit,
            });
            let placement = OrdinaryGridPlacement::prepare(tree, node, &placement_context);
            let collection = GridItemCollection::new(&placement.item_input(&placement_context));
            return Ok(ControlFlow::Continue(Box::new(GridPass {
                children: Some(GridPassChildren::InFlowOrdinary(Box::new(
                    OrdinaryGridInFlow {
                        placement,
                        collection,
                        source: OrdinaryGridSource {
                            tracks,
                            column_tracks,
                            row_tracks,
                            parent_context,
                            subgrid_report,
                            placements,
                            sizing_stage: sizing_phases.limit,
                        },
                    },
                ))),
                output: GridPassOutput {
                    node,
                    input,
                    style,
                    constants,
                    output_size,
                    content_size,
                    report,
                    observes_contained_baseline,
                },
                scroll_box,
            })));
        } else {
            (
                observe_grid_child_layout(tree, node, GridOverflowLayout::Ordinary(child_input))?,
                constants,
            )
        };
        let output = GridPassOutput {
            node,
            input,
            style,
            constants,
            output_size,
            content_size,
            report,
            observes_contained_baseline,
        };
        output
            .finish(Some((scroll_box, child_layout)))
            .map(ControlFlow::Break)
    }
}

struct GridPassOutput<'input, Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    style: GridContainerProjection<'input, S>,
    constants: Constants<S>,
    output_size: Size<S>,
    content_size: Size<S>,
    report: GridComputationReport,
    observes_contained_baseline: bool,
}

impl<Node: Copy, S: LayoutScalar> GridPassOutput<'_, Node, S> {
    fn finish<M>(
        self,
        children: Option<(CanonicalScrollBoxOf<S>, GridChildrenLayout<Node, S>)>,
    ) -> GridComputeOutcome<Node, S, M> {
        let Self {
            node,
            input,
            style,
            constants,
            output_size,
            mut content_size,
            report,
            observes_contained_baseline,
        } = self;
        let mut baselines = BaselinesOf::NONE;
        let mut baseline_groups = GridBaselineGroups {
            rows: Vec::new(),
            columns: Vec::new(),
        };
        let mut final_scroll_geometry = None;
        let mut effective_gutter = None;
        let mut executed_baselines = None;
        if let Some((scroll_box, mut child_layout)) = children {
            effective_gutter = Some(scroll_box.effective_gutter());
            if input.run_mode().is_perform_layout() || observes_contained_baseline {
                baselines = child_layout.baselines;
            }
            baseline_groups = child_layout.baseline_groups;
            executed_baselines = child_layout.executed_baselines;
            content_size = max_size(content_size, child_layout.visible_content_size);
            child_layout
                .contributions
                .replace_container_seed(scroll_box.padding_box());
            child_layout
                .contributions
                .exclude_reserved_gutter_from_range();
            let geometry = grid_container_scroll_geometry::<_, S, M>(
                node,
                input.run_mode(),
                &style,
                &constants,
                scroll_box,
                child_layout.contributions,
                input.settled_auto_scrollbars(),
            )?;
            content_size = max_size(
                content_size,
                geometry
                    .canonical_content_size()
                    .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?,
            );
            final_scroll_geometry = Some(geometry);
        }

        let used_overflow =
            crate::scroll::UsedOverflow::from_computed(style.overflow, style.item_is_replaced);
        let baselines = if used_overflow.is_scroll_container() {
            baselines.clamped_to_border(output_size).map_err(
                |crate::FiniteScalarErrorOf::NonFinite { value }| {
                    crate::error::value_resolution_error(
                        node,
                        LengthResolutionStatus::InvalidNumeric { value },
                    )
                },
            )?
        } else {
            baselines
        };
        let output = if input.run_mode() == RunMode::ComputeSize {
            ComputeOutputOf::from_sizes_and_baselines(output_size, Size::ZERO, baselines)
        } else {
            ComputeOutputOf::from_sizes_and_baselines(output_size, content_size, baselines)
        };
        Ok(GridComputeResult {
            output: ComputeOutputOf {
                scroll_geometry: final_scroll_geometry,
                ..output
            },
            report,
            baseline_groups,
            effective_gutter,
            executed_baselines,
        })
    }
}

fn layout_percent_track_floor<S: LayoutScalar>(
    definite_size: Option<S>,
    available_size: Option<S>,
    tracks: &[TrackSizingOf<S>],
) -> S {
    if definite_size.is_some() || tracks.is_empty() {
        return S::ZERO;
    }
    available_size
        .map(|available| track_basis_dependent_space(tracks, available))
        .unwrap_or(S::ZERO)
}

// Initial imposed/authored fixed body authority can override a formal request;
// an accepted Auto extent cannot. This is separate from internal intrinsic
// phase selection and from physical available space.
fn grid_requested_operations<S: LayoutScalar>(
    flow: crate::geometry::FlowAxes,
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
) -> LogicalSizeOf<SizingOperation> {
    let requested = flow.logical_size(input.sizing_request().axes());
    let fixed = flow.logical_size(constants.explicit_definite_content_size);
    LogicalSizeOf::new(
        if fixed.inline.is_some() {
            SizingOperation::Normal
        } else {
            requested.inline.operation()
        },
        if fixed.block.is_some() {
            SizingOperation::Normal
        } else {
            requested.block.operation()
        },
    )
}

struct IntrinsicSizingAxisInput<'a, S: LayoutScalar = Scalar> {
    operation: SizingOperation,
    run_mode: RunMode,
    style_size: &'a PreferredSizeOf<S>,
    content_size: S,
    track_content_size: S,
    definite_size: Option<S>,
    available_size: Option<S>,
    tracks: &'a [TrackSizingOf<S>],
}

fn intrinsic_sizing_axis_content_size<S: LayoutScalar>(
    input: IntrinsicSizingAxisInput<'_, S>,
) -> S {
    let IntrinsicSizingAxisInput {
        operation,
        run_mode,
        style_size,
        content_size,
        track_content_size,
        definite_size,
        available_size,
        tracks,
    } = input;
    // Use the effective prepared phase operation here, including an authored
    // intrinsic preference. The final receiver separately follows the original
    // requested operation after this intrinsic container extent is accepted.
    if operation != SizingOperation::Normal
        || run_mode == RunMode::ComputeSize
        || style_size.is_auto()
    {
        return content_size;
    }
    track_content_size.max(layout_percent_track_floor(
        definite_size,
        available_size,
        tracks,
    ))
}

#[derive(Clone, Debug, PartialEq)]
struct GridParentContext<S: LayoutScalar = Scalar, Node = ()> {
    columns: Option<InheritedGridAxis<S, Node>>,
    rows: Option<InheritedGridAxis<S, Node>>,
}

impl<S: LayoutScalar, Node> GridParentContext<S, Node> {
    fn none() -> Self {
        Self {
            columns: None,
            rows: None,
        }
    }

    fn has_inherited_axis(&self) -> bool {
        self.columns.is_some() || self.rows.is_some()
    }

    fn geometry_only(&self) -> GridParentContext<S> {
        GridParentContext {
            columns: self.columns.as_ref().map(InheritedGridAxis::geometry_only),
            rows: self.rows.as_ref().map(InheritedGridAxis::geometry_only),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct InheritedGridAxis<S: LayoutScalar = Scalar, Node = ()> {
    offset: S,
    gap: S,
    tracks: Vec<S>,
    minimum_functions: Option<Vec<GridMinimumTrackFact<S>>>,
    geometry: UsedGridAxisGeometryOf<S>,
    named_lines: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    template_area_expanded: bool,
    owner_baseline_targets: Option<InheritedGridOwnerBaselineTargets<Node, S>>,
    // Retained independently of whether any actual owner target exists yet.
    baseline_reference: crate::GridBaselineReference,
    parent_start: usize,
    parent_end: usize,
    reversed: bool,
}

impl<S: LayoutScalar, Node> InheritedGridAxis<S, Node> {
    fn geometry_only(&self) -> InheritedGridAxis<S> {
        InheritedGridAxis {
            offset: self.offset,
            gap: self.gap,
            tracks: self.tracks.clone(),
            minimum_functions: self.minimum_functions.clone(),
            geometry: self.geometry.clone(),
            named_lines: self.named_lines.clone(),
            area_facts: self.area_facts.clone(),
            template_area_expanded: self.template_area_expanded,
            owner_baseline_targets: None,
            baseline_reference: self.baseline_reference,
            parent_start: self.parent_start,
            parent_end: self.parent_end,
            reversed: self.reversed,
        }
    }
}

fn minimum_parent_functions<'a, Node, S: LayoutScalar>(
    columns: &'a [TrackSizingOf<S>],
    rows: &'a [TrackSizingOf<S>],
    basis: LogicalSizeOf<Option<S>>,
    parent_context: &'a GridParentContext<S, Node>,
) -> GridMinimumParentFunctions<'a, S> {
    GridMinimumParentFunctions {
        columns: match &parent_context.columns {
            Some(axis) => axis
                .minimum_functions
                .as_deref()
                .map(GridMinimumTrackFunctions::from_qualified),
            None => Some(GridMinimumTrackFunctions::new(columns, basis.inline)),
        },
        rows: match &parent_context.rows {
            Some(axis) => axis
                .minimum_functions
                .as_deref()
                .map(GridMinimumTrackFunctions::from_qualified),
            None => Some(GridMinimumTrackFunctions::new(rows, basis.block)),
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
struct InheritedGridOwnerBaselineTargets<Node, S: LayoutScalar = Scalar> {
    group: AncestorBaselineGroup<Node, S>,
    mapping: CheckedOwnerToCurrentPlacementMap<Node, S>,
}

#[derive(Clone, PartialEq)]
struct GridContainerContext<S: LayoutScalar = Scalar> {
    topology: ExpandedGridTopology<S>,
    gap: LogicalSizeOf<S>,
    column_gutters: OrdinaryGridAxisGuttersOf<S>,
    row_gutters: OrdinaryGridAxisGuttersOf<S>,
    percent_basis: LogicalSizeOf<Option<S>>,
    leading_columns: usize,
    leading_rows: usize,
    lines: GridLines,
    inherited_column_offset: Option<S>,
    inherited_row_offset: Option<S>,
}

impl<S: LayoutScalar> core::ops::Deref for GridContainerContext<S> {
    type Target = ExpandedGridTopology<S>;

    fn deref(&self) -> &Self::Target {
        &self.topology
    }
}

struct InitializedGridTracks<'input, Node, S: LayoutScalar = Scalar> {
    column_tracks: Vec<TrackSizingOf<S>>,
    row_tracks: Vec<TrackSizingOf<S>>,
    track_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    context: GridContainerContext<S>,
    placements: GridPlacementContext<'input, Node, S>,
    subgrid_report: GridSubgridReport<Node>,
    report: GridComputationReport,
}

fn local_grid_axis_gutters<S: LayoutScalar>(
    display: Display,
    track_count: usize,
    collapsed: &[bool],
    gap: S,
) -> OrdinaryGridAxisGuttersOf<S> {
    if display.establishes_grid_lanes_formatting_context() {
        OrdinaryGridAxisGuttersOf::new_zero_adjacent_to_collapsed_tracks(
            track_count,
            collapsed,
            gap,
        )
    } else {
        OrdinaryGridAxisGuttersOf::new(track_count, collapsed, gap)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ResolvedGridItemPlacement {
    pub(super) column: GridPlacement,
    pub(super) row: GridPlacement,
    pub(super) absolute_column: GridPlacement,
    pub(super) absolute_row: GridPlacement,
    pub(super) in_flow: bool,
}

#[derive(Clone, Debug)]
pub(super) struct GridPlacementContext<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) children: Vec<Node>,
    pub(super) items: Vec<ResolvedGridItemPlacement>,
    child_inputs: Vec<GridChildInput<'input, S>>,
    pub(super) order_modified_indexes: Vec<crate::SourceIndex>,
    settled_areas: Option<Vec<Option<PlacedGridArea>>>,
}

impl<'input, Node, S: LayoutScalar> GridPlacementContext<'input, Node, S> {
    fn new_with_child_inputs(
        children: Vec<Node>,
        items: Vec<ResolvedGridItemPlacement>,
        child_inputs: Vec<GridChildInput<'input, S>>,
    ) -> Self {
        assert_eq!(
            children.len(),
            items.len(),
            "grid placement context must preserve one placement per child"
        );
        assert_eq!(
            children.len(),
            child_inputs.len(),
            "grid placement context must preserve one settled child input per child"
        );
        let order_modified_indexes = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| item.in_flow.then_some(crate::SourceIndex::new(index)))
            .collect();
        Self {
            children,
            items,
            child_inputs,
            order_modified_indexes,
            settled_areas: None,
        }
    }

    #[cfg(test)]
    fn empty() -> Self {
        Self::new_with_child_inputs(Vec::new(), Vec::new(), Vec::new())
    }

    fn with_order_modified_indexes(
        mut self,
        order_modified_indexes: Vec<crate::SourceIndex>,
    ) -> Self {
        debug_assert_eq!(
            order_modified_indexes.len(),
            self.items.iter().filter(|item| item.in_flow).count()
        );
        self.order_modified_indexes = order_modified_indexes;
        self
    }

    fn settled_areas(&self) -> &[Option<PlacedGridArea>] {
        self.settled_areas.as_deref().unwrap_or(&[])
    }

    fn item_input(&self, source_index: usize) -> &GridItemProjection<'input, S> {
        self.child_inputs[source_index].item()
    }

    fn child_input(&self, source_index: usize) -> &GridChildInput<'input, S> {
        &self.child_inputs[source_index]
    }

    fn nested_container_input(
        &self,
        source_index: usize,
    ) -> Option<&GridContainerInput<'input, S>> {
        self.child_inputs[source_index].nested_container()
    }
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridPlacementContext<'input, Node, S> {
    fn checked_child_placements<'a>(
        &'a self,
        children: &'a [Node],
    ) -> impl ExactSizeIterator<Item = (Node, ResolvedGridItemPlacement)> + 'a {
        assert!(
            self.children.as_slice() == children,
            "grid placement context must be consumed with its original child order"
        );
        assert_eq!(
            children.len(),
            self.items.len(),
            "grid placement context must preserve one placement per child"
        );
        children.iter().copied().zip(self.items.iter().copied())
    }
}

type GridTrackInitialization<'input, Node, S, M> =
    LayoutResultOf<Node, InitializedGridTracks<'input, Node, S>, S, M>;

fn initialize_grid_tracks<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    style: &GridContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    parent_context: &GridParentContext<Tree::Scalar, Tree::Node>,
    input: ComputeInputOf<Tree::Scalar>,
    contained_inline: Option<Tree::Scalar>,
) -> GridTrackInitialization<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let track_available = intrinsic_container_available(
        style,
        constants.flow_axes,
        input.parent(),
        input.sizing_request().axes(),
        input.sizing_mode(),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let repeat = lane_repeat::resolve_intrinsic_repeat(
        tree,
        node,
        style,
        constants,
        parent_context,
        track_available,
        contained_inline,
    )?;
    let mut tracks = initialize_grid_tracks_with_repeat(
        tree,
        node,
        style,
        constants,
        parent_context,
        track_available,
        repeat,
    )?;
    if let Some(inline) = contained_inline {
        // Authored constraints own the track counts. The live block phase
        // uses inline tracks sized against the accepted content extent.
        tracks.context.percent_basis.inline = Some(inline);
        if parent_context.columns.is_none() {
            let gap = constants.flow_axes.logical_size(*style.gap).inline;
            let gap = resolve_grid_gap(gap, Some(inline))
                .map_err(|error| crate::error::value_resolution_error(node, error))?;
            tracks.context.gap.inline = gap;
            tracks.context.column_gutters = local_grid_axis_gutters(
                style.display,
                tracks.column_tracks.len(),
                tracks.context.column_gutters.collapsed(),
                gap,
            );
        }
    }
    Ok(tracks)
}

fn initialize_grid_tracks_with_repeat<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    style: &GridContainerProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    parent_context: &GridParentContext<Tree::Scalar, Tree::Node>,
    track_available: LogicalSizeOf<AxisSizingRequestOf<Tree::Scalar>>,
    repeat: Option<LaneRepeatExpansion<'_>>,
) -> GridTrackInitialization<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let sizing_flow_axes = constants.flow_axes;
    let container_style = *style;
    let initial_size = constants.initial_track_basis(constants.node_inner_size);
    let initial_available = constants.initial_track_basis(constants.available_inner_size);
    let gap_basis = initial_size.map(|extent| Some(extent.unwrap_or(Tree::Scalar::ZERO)));
    let resolved_gap = Size::new(
        resolve_grid_gap(container_style.gap.width, gap_basis.width),
        resolve_grid_gap(container_style.gap.height, gap_basis.height),
    )
    .transpose_with_node(tree, node)?;
    let mut gap = sizing_flow_axes.logical_size(resolved_gap);
    if let Some(columns) = &parent_context.columns {
        gap.inline = columns.gap;
    }
    if let Some(rows) = &parent_context.rows {
        gap.block = rows.gap;
    }
    let mut percent_basis = sizing_flow_axes.logical_size(initial_size);
    let children = tree.children(node).collect::<Vec<_>>();
    let column_expansion_basis = track_expansion_basis(
        container_style.grid_template_columns,
        sizing_flow_axes.logical_size(initial_size).inline,
        sizing_flow_axes.logical_size(initial_available).inline,
    );
    let row_expansion_basis = track_expansion_basis(
        container_style.grid_template_rows,
        sizing_flow_axes.logical_size(initial_size).block,
        sizing_flow_axes.logical_size(initial_available).block,
    );
    let auto_repeat_sizing = AutoRepeatSizing::for_container(container_style.display);
    let lane_axis = container_style
        .display
        .establishes_grid_lanes_formatting_context()
        .then(|| grid_axis_for_grid_lanes(&container_style));
    let expand = |axis, components: &[TrackComponentOf<Tree::Scalar>], basis, gap| {
        let count = if let Some(repeat) = repeat.filter(|repeat| repeat.axis == axis) {
            Some(repeat.count)
        } else if lane_axis.is_none_or(|grid_axis| grid_axis == axis)
            && auto_repeat_components(components)
        {
            let constraint = constants
                .auto_repeat_constraint(axis)
                .map_err(|status| crate::error::value_resolution_error(node, status))?;
            standalone_auto_repeat_count(components, basis, gap, constraint)
                .map_err(|error| grid_track_sizing_error(node, error))?
        } else {
            None
        };
        let capacity = match count {
            Some(count) => auto_repeat_track_capacity(components, count).map(Some),
            None => unresolved_repeat_track_capacity(components),
        }
        .map_err(|error| grid_track_sizing_error(node, error))?;
        let mut tracks = Vec::new();
        if let Some(capacity) = capacity {
            tracks.try_reserve_exact(capacity).map_err(|_| {
                grid_track_sizing_error(node, SpanningContributionError::Allocation)
            })?;
        }
        expand_track_components_into(
            components,
            basis,
            gap,
            None,
            count.map_or(auto_repeat_sizing, AutoRepeatSizing::ResolvedGroupCount),
            tracks,
        )
        .map_err(|status| crate::error::value_resolution_error(node, status))
    };
    let column_expansion = if let Some(columns) = &parent_context.columns {
        TrackExpansionOf::inherited(
            columns
                .tracks
                .iter()
                .copied()
                .map(TrackSizingOf::px)
                .collect(),
        )
    } else {
        expand(
            GridAxisKind::Column,
            container_style.grid_template_columns,
            column_expansion_basis,
            gap.inline,
        )?
    };
    let row_expansion = if let Some(rows) = &parent_context.rows {
        TrackExpansionOf::inherited(rows.tracks.iter().copied().map(TrackSizingOf::px).collect())
    } else {
        expand(
            GridAxisKind::Row,
            container_style.grid_template_rows,
            row_expansion_basis,
            gap.block,
        )?
    };
    let sized_column_tracks = column_expansion
        .tracks
        .iter()
        .map(|track| track.sizing.clone())
        .collect::<Vec<_>>();
    let sized_row_tracks = row_expansion
        .tracks
        .iter()
        .map(|track| track.sizing.clone())
        .collect::<Vec<_>>();
    if percent_basis.inline.is_none()
        && intrinsic_stage::unallocated_track_operation(
            GridAxisKind::Column,
            track_available.inline,
        ) == SizingOperation::Normal
        && tracks_need_available_basis(&sized_column_tracks)
    {
        percent_basis.inline = sizing_flow_axes.logical_size(initial_available).inline;
    }
    if percent_basis.block.is_none()
        && intrinsic_stage::unallocated_track_operation(GridAxisKind::Row, track_available.block)
            == SizingOperation::Normal
        && tracks_need_available_basis(&sized_row_tracks)
    {
        percent_basis.block = sizing_flow_axes.logical_size(initial_available).block;
    }
    let sized_explicit_columns = sized_column_tracks.len();
    let sized_explicit_rows = sized_row_tracks.len();
    let mut report = GridComputationReport::default();
    let geometry_parent_context = parent_context.geometry_only();
    let named_context = build_grid_named_context(
        style,
        sized_explicit_columns,
        sized_explicit_rows,
        &geometry_parent_context,
    )
    .map_err(|error| invalid_grid_names_error(node, error))?;
    let mut topology = ExpandedGridTopology::new(ExpandedGridTopologyInput {
        container: container_style,
        columns: column_expansion,
        rows: row_expansion,
        named: named_context,
        column_basis: percent_basis.inline,
        row_basis: percent_basis.block,
        column_gap: gap.inline,
        row_gap: gap.block,
        inherited_columns: parent_context.columns.is_some(),
        inherited_rows: parent_context.rows.is_some(),
    })
    .map_err(|status| crate::error::value_resolution_error(node, status))?;
    if let Some(columns) = &parent_context.columns {
        topology.collapsed_columns = columns.geometry.collapsed().to_vec();
    }
    if let Some(rows) = &parent_context.rows {
        topology.collapsed_rows = rows.geometry.collapsed().to_vec();
    }
    let explicit_columns = topology.explicit_columns;
    let explicit_rows = topology.explicit_rows;
    let (mut placements, placement_report) = resolve_grid_child_placements(
        &children,
        tree,
        &topology,
        style,
        parent_context.columns.is_some(),
        parent_context.rows.is_some(),
    )
    .map_err(|error| grid_placement_demand_error(node, error))?;
    report.merge_named_grid(placement_report);
    if let Some(LaneRepeatExpansion {
        axis,
        placement: LaneRepeatPlacement::Hypothetical(spans),
        ..
    }) = repeat
    {
        debug_assert_eq!(spans.len(), placements.items.len());
        for (placement, span) in placements.items.iter_mut().zip(spans) {
            if let Some(span) = span {
                let automatic =
                    GridPlacement::try_span(span.get()).expect("positive hypothetical span");
                match axis {
                    GridAxisKind::Column => {
                        placement.column = automatic;
                        placement.row = GridPlacement::default();
                    }
                    GridAxisKind::Row => {
                        placement.column = GridPlacement::default();
                        placement.row = automatic;
                    }
                }
            }
        }
    }
    let inherited_columns = parent_context.columns.is_some();
    let inherited_rows = parent_context.rows.is_some();
    let ordinary_settled_placement = !container_style
        .display
        .establishes_grid_lanes_formatting_context();
    let (column_tracks, row_tracks, leading_columns, leading_rows) = if ordinary_settled_placement {
        derive_grid_placement_demand(
            &mut topology,
            &mut placements,
            container_style.grid_auto_flow,
        )
        .map_err(|error| grid_placement_demand_error(node, error))?;
        topology.collapse_ordinary_auto_fit(placements.settled_areas());
        let mut column_tracks = topology.column_tracks.clone();
        let mut row_tracks = topology.row_tracks.clone();
        for (track, collapsed) in column_tracks.iter_mut().zip(&topology.collapsed_columns) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        for (track, collapsed) in row_tracks.iter_mut().zip(&topology.collapsed_rows) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        (
            column_tracks,
            row_tracks,
            topology.column_explicit_start,
            topology.row_explicit_start,
        )
    } else if repeat.is_some_and(|repeat| matches!(repeat.placement, LaneRepeatPlacement::Seed)) {
        (
            topology.column_tracks.clone(),
            topology.row_tracks.clone(),
            0,
            0,
        )
    } else {
        // Grid-lanes retains its separately sequenced pre-sizing and auto-fit policy.
        let mut column_tracks = topology.column_tracks.clone();
        let mut row_tracks = topology.row_tracks.clone();
        let mut leading_columns = 0;
        let mut leading_rows = 0;
        for (axis, tracks, inherited, explicit_count, leading_tracks, auto_tracks, basis, gap) in [
            (
                GridAxisKind::Column,
                &mut column_tracks,
                inherited_columns,
                explicit_columns,
                &mut leading_columns,
                style.grid_auto_columns,
                percent_basis.inline,
                gap.inline,
            ),
            (
                GridAxisKind::Row,
                &mut row_tracks,
                inherited_rows,
                explicit_rows,
                &mut leading_rows,
                style.grid_auto_rows,
                percent_basis.block,
                gap.block,
            ),
        ] {
            if inherited {
                continue;
            }
            let (leading, demand) =
                lanes_axis_track_demand(&placements.items, explicit_count, axis)
                    .map_err(|error| grid_placement_demand_error(node, error))?;
            *leading_tracks = leading;
            let existing = leading.checked_add(tracks.len()).ok_or_else(|| {
                grid_placement_demand_error(
                    node,
                    GridPlacementDemandError::AxisCapacity {
                        axis,
                        requested_tracks: usize::MAX,
                    },
                )
            })?;
            let required = demand.max(existing);
            tracks
                .try_reserve_exact(required - tracks.len())
                .map_err(|_| {
                    grid_placement_demand_error(
                        node,
                        GridPlacementDemandError::AxisCapacity {
                            axis,
                            requested_tracks: required,
                        },
                    )
                })?;
            prepend_auto_tracks(tracks, auto_tracks, basis, gap, leading, None)
                .map_err(|status| crate::error::value_resolution_error(node, status))?;
            extend_auto_tracks(tracks, auto_tracks, basis, gap, required)
                .map_err(|status| crate::error::value_resolution_error(node, status))?;
        }
        let grid_axis = grid_axis_for_grid_lanes(style);
        let (track_count, explicit_start) = match grid_axis {
            GridAxisKind::Column => (column_tracks.len(), leading_columns),
            GridAxisKind::Row => (row_tracks.len(), leading_rows),
        };
        if !repeat
            .is_some_and(|repeat| matches!(repeat.placement, LaneRepeatPlacement::Hypothetical(_)))
        {
            apply_grid_lanes_auto_fit_policy(
                style,
                &mut topology,
                &placements,
                track_count,
                explicit_start,
            )
            .map_err(|error| grid_placement_demand_error(node, error))?;
        }
        for (track, collapsed) in column_tracks.iter_mut().zip(&topology.collapsed_columns) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        for (track, collapsed) in row_tracks.iter_mut().zip(&topology.collapsed_rows) {
            if *collapsed {
                *track = TrackSizingOf::px(Tree::Scalar::ZERO);
            }
        }
        (column_tracks, row_tracks, leading_columns, leading_rows)
    };

    let lines = GridLines {
        column_explicit_start: leading_columns,
        column_explicit_count: explicit_columns,
        row_explicit_start: leading_rows,
        row_explicit_count: explicit_rows,
    };

    topology
        .named_columns
        .set_used_frame(leading_columns, column_tracks.len());
    topology
        .named_rows
        .set_used_frame(leading_rows, row_tracks.len());

    let subgrid_report = collect_subgrid_report(style, &placements);

    let column_gutters = parent_context.columns.as_ref().map_or_else(
        || {
            local_grid_axis_gutters(
                style.display,
                column_tracks.len(),
                &topology.collapsed_columns,
                gap.inline,
            )
        },
        |axis| axis.geometry.sizing_gutters(),
    );
    let row_gutters = parent_context.rows.as_ref().map_or_else(
        || {
            local_grid_axis_gutters(
                style.display,
                row_tracks.len(),
                &topology.collapsed_rows,
                gap.block,
            )
        },
        |axis| axis.geometry.sizing_gutters(),
    );

    Ok(InitializedGridTracks {
        column_tracks,
        row_tracks,
        track_available,
        context: GridContainerContext {
            topology,
            gap,
            column_gutters,
            row_gutters,
            percent_basis,
            leading_columns,
            leading_rows,
            lines,
            inherited_column_offset: parent_context.columns.as_ref().map(|axis| axis.offset),
            inherited_row_offset: parent_context.rows.as_ref().map(|axis| axis.offset),
        },
        placements,
        subgrid_report,
        report,
    })
}

fn invalid_grid_names_error<Node, S: LayoutScalar, M>(
    container: Node,
    error: NamedGridError,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(container),
        LayoutOperation::ChildLayout,
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::GridNames(error.into())),
    )
}

fn grid_placement_demand_error<Node, S: LayoutScalar, M>(
    container: Node,
    error: GridPlacementDemandError,
) -> LayoutErrorOf<Node, S, M> {
    match error {
        GridPlacementDemandError::AxisCapacity { .. }
        | GridPlacementDemandError::UnrepresentableAxisPlacement { .. }
        | GridPlacementDemandError::OccupancyCapacity { .. } => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(container),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::Allocation,
        ),
    }
}

type ResolvedGridChildren<'input, Node, S> =
    (GridPlacementContext<'input, Node, S>, NamedGridReport);

fn resolve_grid_child_placements<'input, Tree, M>(
    children: &[<Tree as Traverse>::Node],
    tree: &Tree,
    topology: &ExpandedGridTopology<Tree::Scalar>,
    parent_style: &GridContainerProjection<'_, Tree::Scalar>,
    subgrid_columns: bool,
    subgrid_rows: bool,
) -> Result<
    ResolvedGridChildren<'input, <Tree as Traverse>::Node, Tree::Scalar>,
    GridPlacementDemandError,
>
where
    Tree: Compute<'input, M>,
{
    let mut report = NamedGridReport::default();
    let mut items = Vec::with_capacity(children.len());
    let child_inputs = children
        .iter()
        .copied()
        .map(|child| {
            input::project_grid_child_input!(tree, child)
                .resolve_against(parent_style, tree.has_leaf_measurement(child))
        })
        .collect::<Vec<_>>();
    let positioned_stacking = (parent_style
        .display
        .establishes_grid_lanes_formatting_context()
        && child_inputs.iter().any(|child| {
            let item = child.item();
            item.display != Display::None && item.position == Position::Absolute
        }))
    .then(|| NamedGridLines::positioned_stacking(lane_axis_for_grid_lanes(parent_style)));
    for child_input in &child_inputs {
        let style = child_input.item();
        if style.display == Display::None {
            items.push(ResolvedGridItemPlacement {
                column: style.grid_column,
                row: style.grid_row,
                absolute_column: style.grid_column,
                absolute_row: style.grid_row,
                in_flow: false,
            });
            continue;
        }

        let in_flow = style.position != Position::Absolute;
        let positioned_axis = |axis, ordinary| {
            positioned_stacking
                .as_ref()
                .filter(|lines| !in_flow && lines.axis == axis)
                .unwrap_or(ordinary)
        };
        let automatic_spans = subgrid_automatic_spans(parent_style, child_input);
        let (column, absolute_column, column_report) =
            resolve_grid_item_axis_placements_with_report(
                positioned_axis(GridAxisKind::Column, &topology.named_columns),
                style.raw_grid_column,
                style.grid_column,
                subgrid_columns,
                in_flow,
                automatic_spans.inline,
            )?;
        report.extend(column_report);
        let (row, absolute_row, row_report) = resolve_grid_item_axis_placements_with_report(
            positioned_axis(GridAxisKind::Row, &topology.named_rows),
            style.raw_grid_row,
            style.grid_row,
            subgrid_rows,
            in_flow,
            automatic_spans.block,
        )?;
        report.extend(row_report);
        items.push(ResolvedGridItemPlacement {
            column,
            row,
            absolute_column,
            absolute_row,
            in_flow,
        });
    }
    let order_modified_indexes = order_modified_indexes(
        &items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.in_flow)
            .map(|(index, _)| {
                (
                    child_inputs[index].item().item_order,
                    crate::SourceIndex::new(index),
                )
            })
            .collect::<Vec<_>>(),
    );
    Ok((
        GridPlacementContext::new_with_child_inputs(children.to_vec(), items, child_inputs)
            .with_order_modified_indexes(order_modified_indexes),
        report,
    ))
}

fn subgrid_automatic_spans<S: LayoutScalar>(
    parent_style: &GridContainerProjection<'_, S>,
    child_input: &GridChildInput<'_, S>,
) -> LogicalSizeOf<Option<crate::GridSpan>> {
    let mut spans = LogicalSizeOf::new(None, None);
    let Some(child_container) = child_input.nested_container() else {
        return spans;
    };
    let child_container = child_container.projection();
    let child_lanes_grid_axis = child_container
        .display
        .establishes_grid_lanes_formatting_context()
        .then(|| grid_axis_for_grid_lanes(&child_container));
    for (axis, components) in [
        (GridAxisKind::Column, child_container.grid_template_columns),
        (GridAxisKind::Row, child_container.grid_template_rows),
    ] {
        let report = subgrid_axis_report(parent_style, child_input, axis);
        if report.can_inherit() && child_lanes_grid_axis.is_none_or(|grid_axis| grid_axis == axis) {
            let span = named::subgrid_automatic_span(axis, components);
            match report.mapping.parent_axis {
                GridAxisKind::Column => spans.inline = span,
                GridAxisKind::Row => spans.block = span,
            }
        }
    }
    spans
}

fn resolve_grid_item_axis_placements_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
    subgrid_axis: bool,
    in_flow: bool,
    automatic_span: Option<crate::GridSpan>,
) -> Result<(GridPlacement, GridPlacement, NamedGridReport), GridPlacementDemandError> {
    if !in_flow {
        let (placement, report) =
            resolve_absolute_grid_item_axis_placement_with_report(lines, raw, legacy, subgrid_axis);
        return Ok((GridPlacement::AUTO, placement, report));
    }

    let (placement, report) = if subgrid_axis {
        resolve_subgrid_item_axis_placement_with_report(lines, raw, legacy, automatic_span)
    } else {
        let (placement, report) =
            resolve_grid_item_axis_placement_with_report(lines, raw, legacy, automatic_span)?;
        (placement.with_automatic_span(automatic_span), report)
    };
    Ok((placement, GridPlacement::AUTO, report))
}

#[cfg(test)]
fn resolve_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    resolve_grid_item_axis_placement_with_report(lines, raw, legacy, None)
        .expect("test grid placement is representable")
        .0
}

fn resolve_grid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
    automatic_span: Option<crate::GridSpan>,
) -> Result<(GridPlacement, NamedGridReport), GridPlacementDemandError> {
    if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        return Ok((legacy, NamedGridReport::default()));
    }
    let (resolved, report) =
        resolve_grid_placement_or_auto_with_report(lines, raw, None, automatic_span)?;
    if resolved.is_auto() && raw_uses_only_numeric_grid_lines(raw) && !legacy.is_auto() {
        Ok((legacy, report))
    } else {
        Ok((resolved, report))
    }
}

#[cfg(test)]
fn resolve_absolute_grid_item_axis_placement(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
) -> GridPlacement {
    resolve_absolute_grid_item_axis_placement_with_report(lines, raw, legacy, false).0
}

fn resolve_absolute_grid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
    subgrid_axis: bool,
) -> (GridPlacement, NamedGridReport) {
    if legacy.is_auto() {
        return resolve_positioned_grid_placement_with_report(lines, raw);
    }

    // Typed placement supplies the geometry. Retain raw diagnostics without
    // making a shadowed placement's track demand part of the layout.
    let report = if subgrid_axis {
        resolve_subgrid_item_axis_placement_with_report(lines, raw, legacy, None).1
    } else {
        resolve_grid_item_axis_placement_with_report(lines, raw, legacy, None)
            .map(|(_, report)| report)
            .unwrap_or_default()
    };
    (legacy, report)
}

fn resolve_subgrid_item_axis_placement_with_report(
    lines: &named::NamedGridLines,
    raw: &super::RawGridPlacement,
    legacy: GridPlacement,
    automatic_span: Option<crate::GridSpan>,
) -> (GridPlacement, NamedGridReport) {
    let typed_anchor;
    let raw = if raw == &super::RawGridPlacement::AUTO && !legacy.is_auto() {
        let line = |line: Option<crate::GridLine>| {
            line.map_or(super::RawGridLine::Auto, |line| {
                super::RawGridLine::Line(line.get())
            })
        };
        let (start, end) = match (legacy.start(), legacy.end(), legacy.span()) {
            (None, end, Some(span)) => (super::RawGridLine::Span(span.get()), line(end)),
            (start, None, Some(span)) => (line(start), super::RawGridLine::Span(span.get())),
            (start, end, _) => (line(start), line(end)),
        };
        typed_anchor = super::RawGridPlacement::new(start, end);
        &typed_anchor
    } else {
        raw
    };
    match resolve_subgrid_placement(lines, raw, None, automatic_span) {
        Ok(placement) => (placement, NamedGridReport::default()),
        Err(error) => (GridPlacement::AUTO, NamedGridReport::from_error(error)),
    }
}

fn raw_uses_only_numeric_grid_lines(raw: &super::RawGridPlacement) -> bool {
    raw_grid_line_is_numeric(&raw.start) && raw_grid_line_is_numeric(&raw.end)
}

fn raw_grid_line_is_numeric(line: &super::RawGridLine) -> bool {
    matches!(
        line,
        super::RawGridLine::Auto | super::RawGridLine::Line(_) | super::RawGridLine::Span(_)
    )
}

struct GridTrackResolutionInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    style: &'a GridContainerProjection<'a, S>,
    constants: &'a Constants<S>,
    column_tracks: &'a [TrackSizingOf<S>],
    row_tracks: &'a [TrackSizingOf<S>],
    minimum_functions: GridMinimumParentFunctions<'a, S>,
    context: std::borrow::Cow<'a, GridContainerContext<S>>,
    subgrid_report: &'a GridSubgridReport<Node>,
    sizing_flow_axes: crate::geometry::FlowAxes,
    available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    placements: &'a GridPlacementContext<'input, Node, S>,
    stage: GridSizingStage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum GridSizingPhase {
    InitialFirst,
    InitialSecond,
    RevisitFirst,
    RevisitSecond,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GridSizingStage {
    first_axis: GridAxisKind,
    through: GridSizingPhase,
}

impl GridSizingStage {
    const COMPLETE: Self = Self {
        first_axis: GridAxisKind::Column,
        through: GridSizingPhase::Complete,
    };

    fn has_resolved_axis(self, axis: GridAxisKind) -> bool {
        self.through >= GridSizingPhase::InitialSecond || self.first_axis == axis
    }

    fn mapped(self, parent: crate::geometry::FlowAxes, child: crate::geometry::FlowAxes) -> Self {
        let first_physical = grid_axis_physical_axis(parent, self.first_axis);
        Self {
            first_axis: if child.inline_axis() == first_physical {
                GridAxisKind::Column
            } else {
                GridAxisKind::Row
            },
            ..self
        }
    }
}

#[derive(Clone, Copy)]
struct GridTrackSizingPhases {
    limit: GridSizingStage,
}

struct GridTrackResolution<'input, 'source, 'prepared, Node, S: LayoutScalar = Scalar> {
    sizing_phases: GridTrackSizingPhases,
    columns: Vec<S>,
    rows: Vec<S>,
    column_preparation: Option<intrinsic_stage::GridAxisPreparationOwner<'prepared, S>>,
    row_preparation: Option<intrinsic_stage::GridAxisPreparationOwner<'prepared, S>>,
    source_frame: intrinsic_stage::GridTrackExecutionFrame<'input, 'source, Node, S>,
}

type GridTrackResolutionResult<'input, 'source, 'prepared, Node, S, M> =
    LayoutResultOf<Node, GridTrackResolution<'input, 'source, 'prepared, Node, S>, S, M>;

struct GridChildLayoutInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    execution_input: ComputeInputOf<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    sizing_phases: GridTrackSizingPhases,
    source_frame: &'a intrinsic_stage::GridTrackExecutionFrame<'input, 'a, Node, S>,
    style: &'a GridContainerProjection<'input, S>,
    constants: &'a Constants<S>,
    column_tracks: &'a [TrackSizingOf<S>],
    row_tracks: &'a [TrackSizingOf<S>],
    minimum_functions: GridMinimumParentFunctions<'a, S>,
    context: GridContainerContext<S>,
    columns: &'a [S],
    rows: &'a [S],
    column_preparation: Option<&'a intrinsic_stage::GridAxisTrackPreparation<S>>,
    row_preparation: Option<&'a intrinsic_stage::GridAxisTrackPreparation<S>>,
    // Keep the original containing constraint separate from final stretch.
    prepared_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    output_size: Size<S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext<S, Node>,
    placements: &'a GridPlacementContext<'input, Node, S>,
}

struct OrdinaryGridTracks<S: LayoutScalar> {
    constants: Constants<S>,
    content_box_size: Size<S>,
    columns: Vec<S>,
    rows: Vec<S>,
    collapsed_columns: Vec<bool>,
    collapsed_rows: Vec<bool>,
    column_gutters: OrdinaryGridAxisGuttersOf<S>,
    row_gutters: OrdinaryGridAxisGuttersOf<S>,
    track_basis: LogicalSizeOf<Option<S>>,
    gap: LogicalSizeOf<S>,
    lines: GridLines,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    template_area_expanded_axes: TemplateAreaExpandedAxes,
    inherited_column_offset: Option<S>,
    inherited_row_offset: Option<S>,
    contained_track_size: Option<Size<S>>,
}

type OrdinaryGridPreparation<'a, Node, S, M> =
    LayoutResultOf<Node, (OrdinaryGridTracks<S>, GridMinimumParentFunctions<'a, S>), S, M>;

fn prepare_ordinary_grid_tracks<'input, 'a, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: GridChildLayoutInput<'input, 'a, Tree::Node, Tree::Scalar>,
) -> OrdinaryGridPreparation<'a, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let FinalGridLayoutTracks {
        columns: layout_columns,
        rows: layout_rows,
        gap: layout_gap,
        column_gutters: layout_column_gutters,
        row_gutters: layout_row_gutters,
        minimum_functions,
        percent_basis: track_basis,
    } = final_grid_layout_tracks(tree, node, &input)?;
    let GridChildLayoutInput {
        constants,
        context,
        output_size,
        parent_context,
        ..
    } = input;
    let GridContainerContext {
        topology,
        column_gutters,
        row_gutters,
        lines,
        inherited_column_offset,
        inherited_row_offset,
        ..
    } = context;
    let authored_template_area_expanded = topology.template_area_expanded_axes();
    let template_area_expanded_axes = TemplateAreaExpandedAxes::new(
        if topology.axis_is_inherited(GridAxisKind::Column) {
            parent_context
                .columns
                .as_ref()
                .is_some_and(|axis| axis.template_area_expanded)
        } else {
            authored_template_area_expanded.for_axis(GridAxisKind::Column)
        },
        if topology.axis_is_inherited(GridAxisKind::Row) {
            parent_context
                .rows
                .as_ref()
                .is_some_and(|axis| axis.template_area_expanded)
        } else {
            authored_template_area_expanded.for_axis(GridAxisKind::Row)
        },
    );
    let ExpandedGridTopology {
        named_columns,
        named_rows,
        area_facts,
        collapsed_columns,
        collapsed_rows,
        ..
    } = topology;
    let layout_content_box_size =
        (output_size - constants.content_box_inset.sum_axes()).max(Size::ZERO);
    let layout_column_gutters = layout_column_gutters.unwrap_or(column_gutters);
    let layout_row_gutters = layout_row_gutters.unwrap_or(row_gutters);
    let containment = size_containment(tree.node_input(node));
    let constants =
        constants.for_contained_placement(containment, output_size, layout_content_box_size);

    let contained_track_size = (containment != crate::SizeContainment::None).then(|| {
        constants.flow_axes.physical_size(LogicalSizeOf::new(
            track_sum_with_gutters(
                &layout_columns,
                layout_gap.inline,
                Some(&layout_column_gutters),
            ),
            track_sum_with_gutters(&layout_rows, layout_gap.block, Some(&layout_row_gutters)),
        ))
    });
    Ok((
        OrdinaryGridTracks {
            constants,
            content_box_size: layout_content_box_size,
            columns: layout_columns,
            rows: layout_rows,
            collapsed_columns,
            collapsed_rows,
            column_gutters: layout_column_gutters,
            row_gutters: layout_row_gutters,
            track_basis,
            gap: layout_gap,
            lines,
            named_columns,
            named_rows,
            area_facts,
            template_area_expanded_axes,
            inherited_column_offset,
            inherited_row_offset,
            contained_track_size,
        },
        minimum_functions,
    ))
}

impl<S: LayoutScalar> OrdinaryGridTracks<S> {
    fn placement_context<'input, 'a, Node>(
        &'a self,
        source: OrdinaryGridPlacementContext<'input, 'a, Node, S>,
    ) -> GridLayoutContext<'input, 'a, Node, S> {
        GridLayoutContext {
            execution_input: source.execution_input,
            scroll_box: source.scroll_box,
            style: source.style,
            constants: &self.constants,
            container_content_size: self.content_box_size,
            columns: &self.columns,
            rows: &self.rows,
            collapsed_columns: &self.collapsed_columns,
            collapsed_rows: &self.collapsed_rows,
            row_tracks: source.row_tracks,
            minimum_functions: source.minimum_functions,
            track_basis: self.track_basis,
            gap: self.gap,
            column_gutters: &self.column_gutters,
            row_gutters: &self.row_gutters,
            lines: self.lines,
            named_columns: &self.named_columns,
            named_rows: &self.named_rows,
            area_facts: self.area_facts.as_ref(),
            template_area_expanded_axes: self.template_area_expanded_axes,
            inherited_column_offset: self.inherited_column_offset,
            inherited_row_offset: self.inherited_row_offset,
            subgrid_report: source.subgrid_report,
            parent_context: source.parent_context,
            placements: source.placements,
            containing_auto_scrollbar_pass: source.execution_input.settled_auto_scrollbars(),
            sizing_stage: source.sizing_stage,
        }
    }

    fn finish<'input, Tree, M>(
        &self,
        tree: &mut Tree,
        node: Tree::Node,
        placement: OrdinaryGridPlacement<Tree::Node, S>,
        context: &GridLayoutContext<'input, '_, Tree::Node, S>,
        pending: PendingGridItems<'input, Tree::Node, S>,
    ) -> GridChildPlacementResult<'input, Tree::Node, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let mut layout = placement.finish(tree, node, context, pending)?;
        if let Some(size) = self.contained_track_size {
            layout = layout.with_contained_track_size(size);
        }
        Ok(layout)
    }
}

struct OrdinaryGridPlacementContext<'input, 'a, Node, S: LayoutScalar> {
    execution_input: ComputeInputOf<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    style: &'a GridContainerProjection<'input, S>,
    row_tracks: &'a [TrackSizingOf<S>],
    minimum_functions: GridMinimumParentFunctions<'a, S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext<S, Node>,
    placements: &'a GridPlacementContext<'input, Node, S>,
    sizing_stage: GridSizingStage,
}

fn layout_grid_container_children<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: GridChildLayoutInput<'input, '_, Tree::Node, Tree::Scalar>,
) -> GridChildPlacementResult<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let source = OrdinaryGridPlacementContext {
        execution_input: input.execution_input,
        scroll_box: input.scroll_box,
        style: input.style,
        row_tracks: input.row_tracks,
        minimum_functions: input.minimum_functions,
        subgrid_report: input.subgrid_report,
        parent_context: input.parent_context,
        placements: input.placements,
        sizing_stage: input.sizing_phases.limit,
    };
    let (tracks, minimum_functions) = prepare_ordinary_grid_tracks(tree, node, input)?;
    let context = tracks.placement_context(OrdinaryGridPlacementContext {
        minimum_functions,
        ..source
    });
    let placement = OrdinaryGridPlacement::prepare(tree, node, &context);
    let pending = layout_grid_child_items(
        tree,
        node,
        placement.item_input(&context),
        |source_index, _| placement.item_disposition(&context, source_index),
    )?;
    tracks.finish(tree, node, placement, &context, pending)
}

struct FinalGridLayoutTracks<'a, S: LayoutScalar> {
    columns: Vec<S>,
    rows: Vec<S>,
    gap: LogicalSizeOf<S>,
    column_gutters: Option<OrdinaryGridAxisGuttersOf<S>>,
    row_gutters: Option<OrdinaryGridAxisGuttersOf<S>>,
    percent_basis: LogicalSizeOf<Option<S>>,
    minimum_functions: GridMinimumParentFunctions<'a, S>,
}

fn final_grid_layout_tracks<'input, 'a, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: &GridChildLayoutInput<'input, 'a, Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, FinalGridLayoutTracks<'a, Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let context = &input.context;
    let flow = input.constants.flow_axes;
    let content_size =
        (input.output_size - input.constants.content_box_inset.sum_axes()).max(Size::ZERO);
    let logical_content_size = flow.logical_size(content_size);
    let operations = grid_requested_operations(flow, input.constants, input.execution_input);
    let normal_axes = operations.map(|operation| operation == SizingOperation::Normal);
    let gap = resolved_logical_layout_gap(tree, node, input, logical_content_size, normal_axes)?;
    let column_gutters =
        (input.parent_context.columns.is_none() && gap.inline != context.gap.inline).then(|| {
            local_grid_axis_gutters(
                input.style.display,
                input.column_tracks.len(),
                context.column_gutters.collapsed(),
                gap.inline,
            )
        });
    let row_gutters =
        (input.parent_context.rows.is_none() && gap.block != context.gap.block).then(|| {
            local_grid_axis_gutters(
                input.style.display,
                input.row_tracks.len(),
                context.row_gutters.collapsed(),
                gap.block,
            )
        });
    let mut result = FinalGridLayoutTracks {
        columns: input.columns.to_vec(),
        rows: input.rows.to_vec(),
        gap,
        column_gutters,
        row_gutters,
        percent_basis: context.percent_basis,
        minimum_functions: input.minimum_functions,
    };
    if input.sizing_phases.limit.through != GridSizingPhase::Complete {
        return Ok(result);
    }
    // Only this completed owner has accepted the normal content extent.
    let percent_basis = LogicalSizeOf::new(
        if normal_axes.inline {
            Some(logical_content_size.inline)
        } else {
            context.percent_basis.inline
        },
        if normal_axes.block {
            Some(logical_content_size.block)
        } else {
            context.percent_basis.block
        },
    );
    result.percent_basis = percent_basis;
    result.minimum_functions = input.minimum_functions.requalified(percent_basis);
    let mut final_context = context.clone();
    final_context.gap = gap;
    final_context.percent_basis = percent_basis;
    if let Some(gutters) = &result.column_gutters {
        final_context.column_gutters = gutters.clone();
    }
    if let Some(gutters) = &result.row_gutters {
        final_context.row_gutters = gutters.clone();
    }
    let final_input = GridTrackResolutionInput {
        style: input.style,
        constants: input.constants,
        column_tracks: input.column_tracks,
        row_tracks: input.row_tracks,
        minimum_functions: result.minimum_functions,
        context: std::borrow::Cow::Owned(final_context),
        subgrid_report: input.subgrid_report,
        sizing_flow_axes: flow,
        available: LogicalSizeOf::new(
            if normal_axes.inline {
                AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(logical_content_size.inline)
                        .map_err(|_| {
                            crate::error::value_resolution_error(
                                node,
                                LengthResolutionStatus::InvalidNumeric {
                                    value: logical_content_size.inline,
                                },
                            )
                        })?,
                    SizingOperation::Normal,
                )
            } else {
                input.prepared_available.inline
            },
            if normal_axes.block {
                AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(logical_content_size.block)
                        .map_err(|_| {
                            crate::error::value_resolution_error(
                                node,
                                LengthResolutionStatus::InvalidNumeric {
                                    value: logical_content_size.block,
                                },
                            )
                        })?,
                    SizingOperation::Normal,
                )
            } else {
                input.prepared_available.block
            },
        ),
        placements: input.placements,
        stage: input.sizing_phases.limit,
    };
    let column_preparation = input
        .column_preparation
        .expect("a completed Grid column axis retains its issued preparation");
    let row_preparation = input
        .row_preparation
        .expect("a completed Grid row axis retains its issued preparation");
    let resolution = intrinsic_stage::resolve_tracks(
        tree,
        node,
        final_input,
        intrinsic_stage::GridTrackRun::AcceptedReceiver {
            source: input.source_frame,
            columns: core::mem::take(&mut result.columns),
            rows: core::mem::take(&mut result.rows),
            column_preparation,
            row_preparation,
            spaces: LogicalSizeOf::new(
                if normal_axes.inline {
                    intrinsic_stage::GridAxisTrackSpace::FinalUsed(logical_content_size.inline)
                } else {
                    intrinsic_stage::GridAxisTrackSpace::Intrinsic
                },
                if normal_axes.block {
                    intrinsic_stage::GridAxisTrackSpace::FinalUsed(logical_content_size.block)
                } else {
                    intrinsic_stage::GridAxisTrackSpace::Intrinsic
                },
            ),
        },
    )?;
    result.columns = resolution.columns;
    result.rows = resolution.rows;
    Ok(result)
}

struct GridLayoutContext<'input, 'a, Node, S: LayoutScalar = Scalar> {
    execution_input: ComputeInputOf<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    style: &'a GridContainerProjection<'input, S>,
    constants: &'a Constants<S>,
    container_content_size: Size<S>,
    columns: &'a [S],
    rows: &'a [S],
    collapsed_columns: &'a [bool],
    collapsed_rows: &'a [bool],
    row_tracks: &'a [TrackSizingOf<S>],
    minimum_functions: GridMinimumParentFunctions<'a, S>,
    track_basis: LogicalSizeOf<Option<S>>,
    gap: LogicalSizeOf<S>,
    column_gutters: &'a OrdinaryGridAxisGuttersOf<S>,
    row_gutters: &'a OrdinaryGridAxisGuttersOf<S>,
    lines: GridLines,
    named_columns: &'a NamedGridLines,
    named_rows: &'a NamedGridLines,
    area_facts: Option<&'a GridAreaNameFacts>,
    template_area_expanded_axes: TemplateAreaExpandedAxes,
    inherited_column_offset: Option<S>,
    inherited_row_offset: Option<S>,
    subgrid_report: &'a GridSubgridReport<Node>,
    parent_context: &'a GridParentContext<S, Node>,
    placements: &'a GridPlacementContext<'input, Node, S>,
    containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    sizing_stage: GridSizingStage,
}

fn resolved_logical_layout_gap<'input, Tree, M>(
    tree: &Tree,
    node: <Tree as Traverse>::Node,
    input: &GridChildLayoutInput<'input, '_, Tree::Node, Tree::Scalar>,
    content_box_size: LogicalSizeOf<Tree::Scalar>,
    normal_axes: LogicalSizeOf<bool>,
) -> LayoutResultOf<<Tree as Traverse>::Node, LogicalSizeOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = input.style;
    let constants = input.constants;
    let sizing_flow_axes = constants.flow_axes;
    let intrinsic_gap = input.context.gap;
    let physical_content_box_size = sizing_flow_axes.physical_size(content_box_size);
    let physical_intrinsic_gap = sizing_flow_axes.physical_size(intrinsic_gap);
    let normal_axes = sizing_flow_axes.physical_size(normal_axes);
    let initial_basis = constants.initial_track_basis(constants.node_inner_size);
    let gap = Size::new(
        if !normal_axes.width {
            Ok(physical_intrinsic_gap.width)
        } else {
            initial_basis.width.map_or_else(
                || resolve_grid_gap(style.gap.width, Some(physical_content_box_size.width)),
                |_| Ok(physical_intrinsic_gap.width),
            )
        },
        if !normal_axes.height {
            Ok(physical_intrinsic_gap.height)
        } else {
            initial_basis.height.map_or_else(
                || resolve_grid_gap(style.gap.height, Some(physical_content_box_size.height)),
                |_| Ok(physical_intrinsic_gap.height),
            )
        },
    )
    .transpose_with_node(tree, node)?;
    Ok(sizing_flow_axes.logical_size(gap))
}

fn effective_content_box_left<S: LayoutScalar>(
    constants: &Constants<S>,
    content_box_size: Size<S>,
) -> S {
    let padding_border_width = (constants.padding + constants.border).horizontal_sum();
    let outer_width = constants
        .node_outer_size
        .width
        .or(constants.node_max_size.width)
        .map(|width| width.max(padding_border_width))
        .unwrap_or(content_box_size.width + constants.content_box_inset.horizontal_sum());
    constants
        .content_box_inset
        .left
        .min((outer_width - constants.content_box_inset.right).max(S::ZERO))
}

#[derive(Clone, Copy)]
struct GridFixedFallback<S: LayoutScalar> {
    descendants: OrthogonalFallbackOf<S>,
    minimum_outer: Size<FixedFallbackSource<S>>,
    maximum_outer: Size<FixedFallbackSource<S>>,
}

fn qualify_grid_fallback<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    style: &GridContainerProjection<'_, S>,
    input: ComputeInputOf<S>,
    source: MeasuredLeafContentBoxInsetSourceOf<S>,
) -> LayoutResultOf<Node, GridFixedFallback<S>, S, M> {
    let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
        (source.padding + source.border).sum_axes()
    } else {
        Size::ZERO
    };
    let sources = crate::layout_math::qualify_fixed_fallback_sources(
        input,
        sizing_algorithm_for_grid_display(style.display),
        &style.common,
        input.known(),
        box_sizing_adjustment,
        crate::scroll::UsedOverflow::from_computed(style.overflow, style.item_is_replaced)
            .is_scroll_container(),
    )
    .map_err(|error| sizing_resolution_error(node, error))?;
    let descendants = child_orthogonal_fallback(
        input.sizing_request().orthogonal_fallback(),
        sources.definite_outer,
        sources.minimum_outer,
        sources.maximum_outer,
        source,
    )
    .map_err(|status| crate::error::value_resolution_error(node, status))?;
    Ok(GridFixedFallback {
        descendants,
        minimum_outer: sources.minimum_outer,
        maximum_outer: sources.maximum_outer,
    })
}

#[derive(Clone, Copy, PartialEq)]
struct Constants<S: LayoutScalar = Scalar> {
    pub(super) flow_axes: crate::geometry::FlowAxes,
    explicit_definite_content_size: Size<Option<S>>,
    indefinite_percentage_axis: Option<PhysicalAxis>,
    child_orthogonal_fallback: OrthogonalFallbackOf<S>,
    fallback_minimum_outer: Size<FixedFallbackSource<S>>,
    fallback_maximum_outer: Size<FixedFallbackSource<S>>,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    node_min_size: Size<Option<S>>,
    node_definite_min_size: Size<Option<S>>,
    node_max_size: Size<Option<S>>,
    available_inner_size: Size<Option<S>>,
    content_box_inset: Edges<S>,
    scrollbar_gutter: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
}

impl<S: LayoutScalar> Constants<S> {
    fn initial_track_basis(&self, size: Size<Option<S>>) -> Size<Option<S>> {
        omit_indefinite_percentage_axis(size, self.indefinite_percentage_axis)
    }

    fn auto_repeat_constraint(
        &self,
        axis: GridAxisKind,
    ) -> Result<AutoRepeatConstraint<S>, LengthResolutionStatus<S>> {
        let flow = self.flow_axes;
        if let Some(preferred) =
            axis.component(flow.logical_size(self.initial_track_basis(self.node_inner_size)))
        {
            return Ok(AutoRepeatConstraint::Fit(
                crate::layout_math::remaining_available_extent(preferred, S::ZERO)?,
            ));
        }
        let inset = axis.component(flow.logical_size(self.content_box_inset.sum_axes()));
        if let Some(maximum) = axis.component(flow.logical_size(self.node_max_size)) {
            return Ok(AutoRepeatConstraint::Fit(
                crate::layout_math::remaining_available_extent(maximum, inset)?,
            ));
        }
        if let Some(minimum) = axis.component(flow.logical_size(self.node_definite_min_size)) {
            return Ok(AutoRepeatConstraint::FillMinimum(
                crate::layout_math::remaining_available_extent(minimum, inset)?,
            ));
        }
        Ok(AutoRepeatConstraint::One)
    }

    fn contained_inline_basis(&self, outer: Size<Option<S>>, stage: GridSizingStage) -> Option<S> {
        let logical = self.flow_axes.logical_size(outer);
        if logical.block.is_some() || !stage.has_resolved_axis(GridAxisKind::Column) {
            return None;
        }
        let inset = self
            .flow_axes
            .logical_size(self.content_box_inset.sum_axes())
            .inline;
        logical.inline.map(|outer| (outer - inset).max(S::ZERO))
    }

    fn for_inline_track_basis(mut self, basis: Option<S>) -> Self {
        let inline = self
            .flow_axes
            .physical_size(LogicalSizeOf::new(basis, None));
        self.node_inner_size = inline.or(self.node_inner_size);
        self
    }

    fn for_contained_placement(
        mut self,
        containment: crate::SizeContainment,
        outer: Size<S>,
        inner: Size<S>,
    ) -> Self {
        self.node_outer_size =
            contained_outer_size(containment, self.flow_axes, outer).or(self.node_outer_size);
        self.node_inner_size =
            contained_outer_size(containment, self.flow_axes, inner).or(self.node_inner_size);
        self
    }

    fn for_used_scroll_box(&self, scroll_box: CanonicalScrollBoxOf<S>) -> Self {
        let content_box_inset = scroll_box.content_box_inset();
        // Intrinsic sizing needs the requested inset, while placement uses the
        // final allocated box's saturated inset. Rebase the inner quantities
        // without changing which outer dimensions were originally definite.
        let inset_difference = self.content_box_inset.sum_axes() - content_box_inset.sum_axes();
        Self {
            explicit_definite_content_size: self
                .explicit_definite_content_size
                .add_optional(inset_difference),
            node_inner_size: self.node_inner_size.add_optional(inset_difference),
            available_inner_size: self.available_inner_size.add_optional(inset_difference),
            content_box_inset,
            scrollbar_gutter: scroll_box.effective_gutter(),
            ..*self
        }
    }

    #[cfg(test)]
    fn new<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &GridContainerProjection<'_, S>,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        Self::with_intrinsic_minimum::<Tree, M>(tree, node, style, input, Size::NONE)
    }

    fn with_intrinsic_minimum<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &GridContainerProjection<'_, S>,
        input: ComputeInputOf<S>,
        standalone_intrinsic_minimum: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        style.with_inherited_constraints(|style| {
            Self::from_box_constraints::<Tree, M>(
                tree,
                node,
                style,
                input,
                standalone_intrinsic_minimum,
            )
        })
    }

    fn from_box_constraints<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &GridContainerProjection<'_, S>,
        input: ComputeInputOf<S>,
        standalone_intrinsic_minimum: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            *style.padding,
            *style.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let flow_axes = crate::geometry::FlowAxes::new(style.writing_mode, style.direction);
        let padding_border = padding + border;
        let padding_border_size = padding_border.sum_axes();
        let box_sizing_adjustment = if style.box_sizing == BoxSizing::ContentBox {
            padding_border_size
        } else {
            Size::ZERO
        };
        let algorithm = sizing_algorithm_for_grid_display(style.display);
        let style_size = if input.sizing_mode() == SizingMode::InherentSize {
            Size::new(
                resolve_preferred_optional(
                    &style.size.width,
                    algorithm,
                    PhysicalAxis::Horizontal,
                    input.parent().width,
                    true,
                )
                .map_err(|error| sizing_resolution_error(node, error))?,
                resolve_preferred_optional(
                    &style.size.height,
                    algorithm,
                    PhysicalAxis::Vertical,
                    input.parent().height,
                    true,
                )
                .map_err(|error| sizing_resolution_error(node, error))?,
            )
        } else {
            Size::NONE
        };
        // The inherited allocation is a border box. Its ratio-determining box
        // follows box-sizing; ignored authored sizes never supply this seed.
        let inherited_known =
            input.known().zip_map(
                style.inherited_axes,
                |known, inherited| {
                    if inherited { known } else { None }
                },
            );
        let ratio_seed = inherited_known
            .sub_optional_unchecked(box_sizing_adjustment)
            .map(|known| known.map(|extent| extent.max(S::ZERO)));
        let style_size = ratio_seed
            .or(style_size)
            .apply_aspect_ratio(*style.aspect_ratio)
            .add_optional(box_sizing_adjustment)
            .zip_map(
                style.inherited_axes,
                |size, inherited| {
                    if inherited { None } else { size }
                },
            );
        let resolve_minimum = |axis, basis, intrinsic_minimum: Option<S>| {
            let resolve = |minimum: &MinSizeOf<S>| {
                resolve_minimum_optional(minimum, algorithm, axis, basis, true)
                    .map_err(|error| sizing_resolution_error(node, error))
            };
            if intrinsic_minimum.is_some() {
                resolve(&MinSizeOf::AUTO)
            } else {
                style.with_minimum(axis, resolve)
            }
        };
        let resolved_min_size = Size::new(
            resolve_minimum(
                PhysicalAxis::Horizontal,
                input.parent().width,
                standalone_intrinsic_minimum.width,
            )?,
            resolve_minimum(
                PhysicalAxis::Vertical,
                input.parent().height,
                standalone_intrinsic_minimum.height,
            )?,
        )
        .apply_aspect_ratio(*style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .zip_map(
            style.inherited_axes,
            |size, inherited| {
                if inherited { None } else { size }
            },
        );
        let min_size = Size::new(
            standalone_intrinsic_minimum
                .width
                .or(resolved_min_size.width),
            standalone_intrinsic_minimum
                .height
                .or(resolved_min_size.height),
        )
        .zip_map(
            style.inherited_axes,
            |size, inherited| {
                if inherited { None } else { size }
            },
        );
        // CSS Sizing 3 §3.1 gives the minimum precedence over a conflicting
        // maximum. Use the effective bound for track sizing and gutter insets.
        let max_size = Size::new(
            resolve_maximum_optional(
                &style.max_size.width,
                algorithm,
                PhysicalAxis::Horizontal,
                input.parent().width,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
            resolve_maximum_optional(
                &style.max_size.height,
                algorithm,
                PhysicalAxis::Vertical,
                input.parent().height,
                true,
            )
            .map_err(|error| sizing_resolution_error(node, error))?,
        )
        .apply_aspect_ratio(*style.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .zip_map(
            style.inherited_axes,
            |size, inherited| {
                if inherited { None } else { size }
            },
        )
        .max_optional(min_size);
        let fallback_source = MeasuredLeafContentBoxInsetSourceOf {
            flow_axes,
            computed_overflow: style.overflow,
            item_is_replaced: style.item_is_replaced,
            scrollbar_gutter: style.scrollbar_gutter,
            scrollbar_width: style.scrollbar_width,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            padding,
            border,
        };
        let scroll_box = {
            let provisional_outer_size = input
                .known()
                .or(style_size.clamp_min_before_max_optional(min_size, max_size))
                .max_optional(padding_border_size.map(Some));
            // An available or maximum constraint is not an allocated intrinsic
            // border box. For an unknown axis, qualify the full requested inset
            // against exactly the extent it requires. This intermediate basis
            // is used only for insets; final geometry uses the resolved output.
            let requested_inset = measured_leaf_content_box_inset(fallback_source);
            let scroll_box_size = provisional_outer_size
                .unwrap_or(requested_inset.sum_axes())
                .max(padding_border_size);
            canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf {
                flow_axes,
                computed_overflow: style.overflow,
                item_is_replaced: style.item_is_replaced,
                border_box_size: scroll_box_size,
                border,
                padding,
                scrollbar_gutter: style.scrollbar_gutter,
                scrollbar_width: style.scrollbar_width,
                settled_auto_scrollbars: input.settled_auto_scrollbars(),
            })
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?
        };
        let content_box_inset = scroll_box.content_box_inset();
        let scrollbar_gutter = scroll_box.effective_gutter();
        let explicit_definite_outer_size =
            input.descendant_percentage_basis(input.known().or(style_size));
        let explicit_definite_content_size =
            explicit_definite_outer_size.sub_optional_unchecked(content_box_inset.sum_axes());
        let GridFixedFallback {
            descendants: child_orthogonal_fallback,
            minimum_outer: fallback_minimum_outer,
            maximum_outer: fallback_maximum_outer,
        } = qualify_grid_fallback(node, style, input, fallback_source)?;
        let node_outer_size = input
            .known()
            .or(style_size.clamp_min_before_max_optional(min_size, max_size))
            .max_optional(padding_border_size.map(Some));
        let node_inner_size = node_outer_size.sub_optional_unchecked(content_box_inset.sum_axes());
        let available_size = input
            .sizing_request()
            .axes()
            .zip_map(max_size, intrinsic_available_size_for_axis)
            .clamp_min_before_max_optional(min_size, max_size)
            .max_optional(padding_border_size.map(Some));
        let available_inner_size =
            available_size.sub_optional_unchecked(content_box_inset.sum_axes());

        Ok(Self {
            flow_axes,
            explicit_definite_content_size,
            indefinite_percentage_axis: input.indefinite_percentage_axis(),
            child_orthogonal_fallback,
            fallback_minimum_outer,
            fallback_maximum_outer,
            node_outer_size,
            node_inner_size,
            node_min_size: min_size,
            node_definite_min_size: resolved_min_size,
            node_max_size: max_size,
            available_inner_size,
            content_box_inset,
            scrollbar_gutter,
            padding,
            border,
        })
    }
}

fn grid_container_scroll_geometry<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    style: &GridContainerProjection<'_, S>,
    constants: &Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    contributions: crate::scroll::ScrollContributionAccumulatorOf<S>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
) -> LayoutResultOf<Node, crate::ScrollGeometryOf<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    canonical_scroll_geometry_from_source(CanonicalScrollGeometrySourceOf {
        flow_axes: constants.flow_axes,
        computed_overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: scroll_box.border_box().size(),
        border: constants.border,
        padding: constants.padding,
        scrollbar_gutter: style.scrollbar_gutter,
        scrollbar_width: style.scrollbar_width,
        settled_auto_scrollbars,
        clip_margin: ClipMarginSourceOf::new(
            style.overflow_clip_margin.clip_box(),
            style.overflow_clip_margin.margin(),
        ),
        scroll_padding: OptimalRegionInsetsOf::from_scroll_padding(style.scroll_padding),
        contributions,
        origin_axes: ScrollOriginAxes::ENDWARD,
        scroll_snap_type: style.scroll_snap_type,
        target_border_box: scroll_box.border_box(),
        target_scroll_margin: style.scroll_margin,
        target_flow_axes: constants.flow_axes,
        target_snap_align: style.scroll_snap_align,
        target_snap_stop: style.scroll_snap_stop,
    })
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

fn resolve_grid_gap<S: LayoutScalar>(
    gap: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    let value = resolve_length_or_zero(gap, basis)?;
    Ok(if gap.depends_on_basis() {
        value.max(S::ZERO)
    } else {
        value
    })
}

fn resolve_length_or_zero<S: LayoutScalar>(
    length: LengthOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_or_zero<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<S, LengthResolutionStatus<S>> {
    resolution_or_zero(length.resolve_with_status(basis))
}

fn resolve_auto_optional<S: LayoutScalar>(
    length: LengthAutoOf<S>,
    basis: Option<S>,
) -> Result<Option<S>, LengthResolutionStatus<S>> {
    resolution_optional(length.resolve_with_status(basis))
}

trait SizeOptionExt {
    type Scalar: LayoutScalar;
    fn clamp_min_before_max_optional(self, min: Self, max: Self) -> Self;
    fn max_optional(self, other: Self) -> Self;
}

trait SizeExt {
    type Scalar: LayoutScalar;
    fn max(self, other: Self) -> Self;
    fn clamp_min_before_max_optional(
        self,
        min: Size<Option<Self::Scalar>>,
        max: Size<Option<Self::Scalar>>,
    ) -> Self;
}

impl<S: LayoutScalar> SizeExt for Size<S> {
    type Scalar = S;

    fn max(self, other: Self) -> Self {
        Size::new(self.width.max(other.width), self.height.max(other.height))
    }

    fn clamp_min_before_max_optional(self, min: Size<Option<S>>, max: Size<Option<S>>) -> Self {
        Size::new(
            self.width
                .clamp_min_before_max_optional(min.width, max.width),
            self.height
                .clamp_min_before_max_optional(min.height, max.height),
        )
    }
}

impl<S: LayoutScalar> SizeOptionExt for Size<Option<S>> {
    type Scalar = S;

    fn clamp_min_before_max_optional(self, min: Self, max: Self) -> Self {
        self.zip_map(min, |value, min| match (value, min) {
            (Some(value), Some(min)) => Some(value.max(min)),
            (value, _) => value,
        })
        .zip_map(max, |value, max| match (value, max) {
            (Some(value), Some(max)) => Some(value.min(max)),
            (value, _) => value,
        })
    }

    fn max_optional(self, other: Self) -> Self {
        self.zip_map(other, |value, other| match (value, other) {
            (Some(value), Some(other)) => Some(value.max(other)),
            (value, _) => value,
        })
    }
}

trait ScalarExt {
    fn clamp_min_before_max_optional(self, min: Option<Self>, max: Option<Self>) -> Self
    where
        Self: Sized;
}

impl<S: LayoutScalar> ScalarExt for S {
    fn clamp_min_before_max_optional(self, min: Option<Self>, max: Option<Self>) -> Self {
        let value = min.map_or(self, |min| self.max(min));
        max.map_or(value, |max| value.min(max))
    }
}

#[cfg(test)]
mod fri06_c13_t06_characterization_tests {
    use super::*;
    use crate::{
        LayoutErrorKindOf, LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutOperation,
        LengthPercentageOf, NodeInputOf, ParentFormattingContext, RequestedAxis, WritingMode,
    };

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
            crate::ContainingLayoutContext::new(
                containing_flow_axes,
                ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::splat(AxisSizingRequestOf::new(
                    AvailableSpaceOf::Indefinite,
                    SizingOperation::MaxContent,
                )),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
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
            display: crate::Display::Grid,
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
                &GridContainerProjection::from_node(&style),
                input(flow, parent),
            )
            .expect("grid constants edge characterization must resolve");
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
            display: crate::Display::Grid,
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
        let error = match Constants::new::<_, core::convert::Infallible>(
            &failing_tree,
            7,
            &GridContainerProjection::from_node(&failing_style),
            input(
                crate::geometry::FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::splat(Some(largest)),
            ),
        ) {
            Ok(_) => panic!("padding failure must precede the distinct border failure"),
            Err(error) => error,
        };
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
    fn fri06_c13_t06_grid_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_grid_resolution_edges_and_error_order_preserve_f64() {
        characterize_constants::<f64>(f64::MAX);
    }
}

#[cfg(test)]
mod fri06_c13_t05_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(Some(scalar(2.0)), Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(Some(scalar(-3.0)), Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_min_before_max_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            Size::new(Some(scalar(5.0)), Some(scalar(5.0))).clamp_min_before_max_optional(
                Size::new(Some(scalar(10.0)), Some(scalar(10.0))),
                Size::new(Some(scalar(3.0)), Some(scalar(3.0))),
            ),
            Size::new(Some(scalar(3.0)), Some(scalar(3.0)))
        );
    }

    #[test]
    fn fri06_c13_t05_grid_unchecked_subtraction_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_grid_unchecked_subtraction_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}
