use std::ops::ControlFlow;

use super::{
    AlignContent, AlignItems, AspectRatioOf, AxisSizingRequestOf, BaselinesOf, BoxSizing, Compute,
    ComputeInputOf, ComputeOutputOf, ContainingLayoutContext, Direction, Edges, FlexDirection,
    FlexWrap, LayoutResultOf, LayoutScalar, LengthAutoOf, LengthOf, LengthResolutionStatus,
    ParentFormattingContext, Point, RequestedAxis, RunMode, Size, SizingMode, Traverse,
};
use crate::engine::containment::{
    compute_empty_box, contained_outer_size, empty_sizing_input, size_containment,
};
use crate::error::{SizingAlgorithm, layout_own_geometry_error};
use crate::geometry::{FlowAxes, LogicalAxis, PhysicalAxis, PhysicalProgression, PhysicalSide};
use crate::layout_math::{
    MaxBeforeMinOptionalSizeClampExt, MaxBeforeMinScalarClampExt, MaxBeforeMinSizeClampExt,
    OptionalMinimumSizeFloorExt, OptionalSizeExt, OptionalSizeMaxExt, UncheckedOptionalSizeSubExt,
    resolution_optional, resolution_or_zero, resolve_containing_padding_border,
    transfer_aspect_ratio_limits,
};
use crate::scroll::{
    CanonicalScrollBoxOf, CanonicalScrollBoxSourceOf, ScrollOriginAxes, ScrollOriginProgression,
    canonical_scroll_box_from_source,
};
use crate::sizing::resolve::{
    EdgesResultExt, SizeResultExt, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};

mod absolute;
mod alignment;
mod cross_size;
mod distribution_number;
mod flexible_lengths;
mod input;
mod intrinsic;
mod items;
mod lines;
mod rounds;
mod scroll;

pub(crate) use absolute::AbsoluteChild;
use absolute::{AbsoluteChildren, layout_hidden_children};
use alignment::{
    BaselineChild, BaselineExport, align_items_on_cross_axis, align_lines_on_cross_axis,
    alignment_fallback_from_sign, first_final_vertical_baseline, last_final_vertical_baseline,
    line_cross_size, line_free_space, main_alignment_offset, resolve_main_axis_auto_margins,
    resolved_vertical_baselines, stretch_lines_on_cross_axis,
};
use cross_size::{CrossSizeChild, LineResolution};
use distribution_number::DistributionNumber;
use flexible_lengths::resolve_flexible_lengths;
use input::{FlexContainerProjection, with_flex_container_projection, with_flex_item_projection};
use intrinsic::{
    IntrinsicMainSize, begin_main_size, finish_main_size, intrinsic_content_main_size,
    resolved_cross_layout_constants,
};
use items::{
    CollectedFlexItem, FinalFlexItem, FinalItems, FlexItemRequest, ItemCollection,
    PendingFinalItem, ResolvedFlexItem, has_eligible_items,
};
use lines::{
    CollapsedFlexStruts, FlexLine, FlexLineCollectionRound, collect_flex_lines,
    flex_line_collection_size,
};
use rounds::{FlexRoundChild, FlexRounds, ResolvedFlexRound};

use scroll::{
    FlexChildContribution, flex_container_scroll_box, flex_container_scroll_geometry,
    flex_scroll_contribution_error, flex_scroll_contributions, retain_flex_scroll_geometry,
};

/// When a row owns its preferred inline size, retain any intrinsic cross
/// preference handled by the same formatter.
pub(crate) fn preferred_flex_axes<S: LayoutScalar>(
    display: crate::Display,
    direction: FlexDirection,
    flow_axes: FlowAxes,
    preferred: &Size<crate::PreferredSizeOf<S>>,
) -> Size<bool> {
    let preferred = flow_axes.logical_size(Size::new(&preferred.width, &preferred.height));
    let owns_inline =
        display == crate::Display::Flex && direction.is_row() && preferred.inline.is_max_content();
    flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        owns_inline,
        owns_inline && preferred.block.is_max_content(),
    ))
}

pub(crate) fn child_preferred_axes<'input, Tree, M>(tree: &Tree, child: Tree::Node) -> Size<bool>
where
    Tree: Compute<'input, M>,
{
    if tree.has_leaf_measurement(child) {
        return Size::splat(false);
    }
    let input = tree.node_input(child);
    preferred_flex_axes(
        input.display,
        input.flex_direction,
        FlowAxes::new(input.writing_mode, input.direction),
        &input.size,
    )
}

fn preferred_intrinsic_request<S: LayoutScalar>() -> AxisSizingRequestOf<S> {
    AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Indefinite,
        crate::SizingOperation::MaxContent,
    )
}

/// A ContentSize probe omits the child's authored sizes. Retain the operations
/// that its intrinsic preferences supply while preserving imposed dimensions
/// and the specific main-axis contribution requested by the parent.
fn child_preferred_probe_axes<S: LayoutScalar>(
    axes: FlexAxes,
    requests: Size<AxisSizingRequestOf<S>>,
    known: Size<Option<S>>,
    preferred: Size<bool>,
) -> Size<AxisSizingRequestOf<S>> {
    let mut main = axes.main_size(requests);
    let mut cross = axes.cross_size(requests);
    if axes.main_size(preferred) && axes.main_size(known).is_none() {
        main = main.with_space(crate::AvailableSpaceOf::Indefinite);
    }
    if axes.cross_size(preferred) && axes.cross_size(known).is_none() {
        cross = preferred_intrinsic_request();
    }
    axes.size_from_main_cross(main, cross)
}

pub(crate) fn compute_flex<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut computation = match begin_flex_computation(tree, node, input)? {
        ControlFlow::Break(output) => return Ok(output),
        ControlFlow::Continue(computation) => computation,
    };
    loop {
        match computation.advance(tree)? {
            ControlFlow::Break(output) => return Ok(output),
            ControlFlow::Continue(child) => {
                let output = {
                    let (node, input) = child.request();
                    tree.compute_child(node, input)?
                };
                computation.accept_child(tree, child, output)?;
            }
        }
    }
}

pub(crate) struct FlexComputation<'input, Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    pass_input: ComputeInputOf<S>,
    contained_size: Size<Option<S>>,
    pass: Option<Box<FlexPhase<'input, Node, S>>>,
}

type FlexStart<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, FlexComputation<'input, Node, S>>, S, M>;
type FlexStep<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, FlexChild<'input, Node, S>>, S, M>;
type FlexPassStart<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, Box<FlexPhase<'input, Node, S>>>, S, M>;

pub(crate) struct FlexChild<'input, Node, S: LayoutScalar> {
    phase: FlexChildPhase<'input, Node, S>,
}

enum FlexChildPhase<'input, Node, S: LayoutScalar> {
    Collect(FlexItemRequest<Node, S>),
    Resolve(FlexRoundChild<Node, S>),
    Baseline(BaselineChild<Node, S>),
    Place(PendingFinalItem<'input, Node, S>),
    Absolute(AbsoluteChild<Node, S>),
}

impl<Node: Copy, S: LayoutScalar> FlexChild<'_, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        match &self.phase {
            FlexChildPhase::Collect(child) => (child.node, child.input),
            FlexChildPhase::Resolve(child) => child.request(),
            FlexChildPhase::Baseline(child) => (child.request.node, child.request.input),
            FlexChildPhase::Place(child) => (child.request.node, child.request.input),
            FlexChildPhase::Absolute(child) => (child.node, child.input),
        }
    }
}

enum FlexPhase<'input, Node, S: LayoutScalar> {
    Collect(Box<FlexCollection<'input, Node, S>>),
    Resolve(Box<FlexResolution<'input, Node, S>>),
    Baseline(FlexBaselineExport<Node, S>),
    Place(FlexPlacement<Node, S>),
    Absolute(FlexPass<Node, S>),
}

struct FlexCollection<'input, Node, S: LayoutScalar> {
    phase: CollectionPhase<S>,
    items: ItemCollection<'input, Node, S>,
    constants: Constants<S>,
    used_overflow: crate::scroll::UsedOverflow,
}

enum CollectionPhase<S: LayoutScalar> {
    Initial {
        own_ratio_cross: Option<OwnRatioCross<S>>,
        contents_run_mode: RunMode,
        requested_inline_operation: crate::SizingOperation,
        inherited_indefinite_cross: bool,
        first_run_mode: RunMode,
    },
    Accepted {
        contents_run_mode: RunMode,
    },
    RatioMinimum(Box<FlexRatioMinimum<S>>),
}

struct FlexRatioMinimum<S: LayoutScalar> {
    contents: AcceptedFlexContents<S>,
    transferred: S,
}

struct AcceptedFlexContents<S: LayoutScalar> {
    constants: Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    run_mode: RunMode,
    qualify_ratio_cross: bool,
}

struct CollectedFlexItems<Node, S: LayoutScalar> {
    phase: CollectionPhase<S>,
    items: Vec<CollectedFlexItem<Node, S>>,
    constants: Constants<S>,
    used_overflow: crate::scroll::UsedOverflow,
}

struct FlexResolution<'input, Node, S: LayoutScalar> {
    collection: CollectedFlexItems<Node, S>,
    rounds: FlexRounds<'input, Node, S>,
}

struct FlexBaselineExport<Node, S: LayoutScalar> {
    baselines: BaselineExport<Node, S>,
    constants: Constants<S>,
    sizes: FlexContainerSizes<S>,
    used_overflow: crate::scroll::UsedOverflow,
    no_eligible_items: bool,
}

struct FlexPlacement<Node, S: LayoutScalar> {
    items: FinalItems<Node, S>,
    constants: Constants<S>,
    scroll_box: CanonicalScrollBoxOf<S>,
    lines: Vec<FlexLine<S>>,
    sizes: FlexContainerSizes<S>,
    used_overflow: crate::scroll::UsedOverflow,
    no_eligible_items: bool,
}

pub(crate) fn begin_flex_computation<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
) -> FlexStart<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let style = tree.node_input(node);
    let containment = size_containment(style);
    let contained_size = if containment != crate::SizeContainment::None {
        let empty = compute_empty_box(node, style, |probe| {
            let empty_input = empty_sizing_input(input);
            let mut computation = FlexComputation {
                node,
                input: empty_input,
                pass_input: empty_input,
                contained_size: Size::NONE,
                pass: None,
            };
            match computation.advance(probe)? {
                ControlFlow::Break(output) => Ok(output),
                ControlFlow::Continue(_) => unreachable!("empty sizing has no children"),
            }
        })?;
        if containment == crate::SizeContainment::BothAxes
            && !input.run_mode().is_perform_layout()
            && !input.requires_content_baseline()
        {
            return Ok(ControlFlow::Break(empty));
        }
        contained_outer_size(
            containment,
            crate::FlowAxes::new(style.writing_mode, style.direction),
            empty.size,
        )
    } else {
        Size::NONE
    };
    Ok(ControlFlow::Continue(FlexComputation {
        node,
        input,
        pass_input: input,
        contained_size,
        pass: None,
    }))
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> FlexComputation<'input, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        (self.node, self.input)
    }

    pub(crate) fn advance<Tree, M>(&mut self, tree: &mut Tree) -> FlexStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            let output = match self.pass.as_deref_mut() {
                Some(FlexPhase::Collect(collection)) => {
                    if let Some(child) = collection.items.next(tree, &collection.constants)? {
                        return Ok(ControlFlow::Continue(FlexChild {
                            phase: FlexChildPhase::Collect(child),
                        }));
                    }
                    let phase = self.pass.take().expect("active Flex collection");
                    let FlexPhase::Collect(collection) = *phase else {
                        unreachable!()
                    };
                    self.pass = Some(Box::new(FlexPhase::Resolve(Box::new(
                        collection.finish(self.node)?,
                    ))));
                    continue;
                }
                Some(FlexPhase::Resolve(resolution)) => {
                    let layout = match resolution.rounds.advance(
                        tree,
                        self.node,
                        &resolution.collection.items,
                    )? {
                        ControlFlow::Continue(child) => {
                            return Ok(ControlFlow::Continue(FlexChild {
                                phase: FlexChildPhase::Resolve(child),
                            }));
                        }
                        ControlFlow::Break(layout) => layout,
                    };
                    let phase = self.pass.take().expect("active Flex rounds");
                    let FlexPhase::Resolve(resolution) = *phase else {
                        unreachable!()
                    };
                    let FlexResolution { collection, .. } = *resolution;
                    match finish_flex_collection(
                        tree,
                        self.node,
                        self.pass_input,
                        collection,
                        layout,
                    )? {
                        ControlFlow::Break(output) => output,
                        ControlFlow::Continue(phase) => {
                            self.pass = Some(phase);
                            continue;
                        }
                    }
                }
                Some(FlexPhase::Baseline(export)) => {
                    let baselines = match export.baselines.advance(tree, &export.constants)? {
                        ControlFlow::Continue(child) => {
                            return Ok(ControlFlow::Continue(FlexChild {
                                phase: FlexChildPhase::Baseline(child),
                            }));
                        }
                        ControlFlow::Break(baselines) => baselines,
                    };
                    let output = finish_flex_size(
                        self.node,
                        &export.constants,
                        export.sizes,
                        baselines,
                        export.used_overflow,
                        export.no_eligible_items,
                    )?;
                    self.pass = None;
                    output
                }
                Some(FlexPhase::Place(placement)) => {
                    if let Some(child) = placement.items.next(tree, &placement.constants)? {
                        return Ok(ControlFlow::Continue(FlexChild {
                            phase: FlexChildPhase::Place(child),
                        }));
                    }
                    let phase = self.pass.take().expect("active Flex placement");
                    let FlexPhase::Place(placement) = *phase else {
                        unreachable!()
                    };
                    self.pass = Some(Box::new(FlexPhase::Absolute(
                        placement.finish(tree, self.node)?,
                    )));
                    continue;
                }
                Some(FlexPhase::Absolute(pass)) => {
                    if let Some(child) = pass.absolute.next_child(tree)? {
                        return Ok(ControlFlow::Continue(FlexChild {
                            phase: FlexChildPhase::Absolute(child),
                        }));
                    }
                    let phase = self.pass.take().expect("active Flex absolute phase");
                    let FlexPhase::Absolute(pass) = *phase else {
                        unreachable!()
                    };
                    pass.finish(tree, self.node, self.pass_input)?
                }
                None => {
                    match begin_flex_pass(tree, self.node, self.pass_input, self.contained_size)? {
                        ControlFlow::Break(output) => output,
                        ControlFlow::Continue(phase) => {
                            self.pass = Some(phase);
                            continue;
                        }
                    }
                }
            };
            if !self.input.run_mode().is_perform_layout() {
                return Ok(ControlFlow::Break(output));
            }
            let Some(geometry) = output.scroll_geometry else {
                return Ok(ControlFlow::Break(output));
            };
            let current = self.pass_input.settled_auto_scrollbars();
            let next = current.transition(geometry);
            if current == next
                || !crate::scroll::settled_auto_scrollbars_change_available_geometry(geometry, next)
                    .map_err(|error| {
                        layout_own_geometry_error(self.node, self.input.run_mode(), error)
                    })?
            {
                return Ok(ControlFlow::Break(output));
            }
            self.pass_input = self.pass_input.with_settled_auto_scrollbars(next);
        }
    }

    pub(crate) fn accept_child<Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: FlexChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match (self.pass.as_deref_mut(), child.phase) {
            (Some(FlexPhase::Collect(collection)), FlexChildPhase::Collect(_)) => {
                collection.items.accept(tree, &collection.constants, output)
            }
            (Some(FlexPhase::Resolve(resolution)), FlexChildPhase::Resolve(child)) => resolution
                .rounds
                .accept(tree, self.node, &resolution.collection.items, child, output),
            (Some(FlexPhase::Baseline(export)), FlexChildPhase::Baseline(child)) => {
                export.baselines.accept(&export.constants, child, output)
            }
            (Some(FlexPhase::Place(placement)), FlexChildPhase::Place(child)) => placement
                .items
                .accept(tree, &placement.constants, child, output),
            (Some(FlexPhase::Absolute(pass)), FlexChildPhase::Absolute(child)) => {
                let contribution = pass.absolute.accept_child(tree, self.node, child, output)?;
                pass.absolute_contributions.push(contribution);
                Ok(())
            }
            _ => unreachable!("Flex child receipt resumes its owning phase"),
        }
    }
}

impl<Node: Copy, S: LayoutScalar> FlexPlacement<Node, S> {
    fn finish<'input, Tree, M>(
        self,
        tree: &Tree,
        node: Node,
    ) -> LayoutResultOf<Node, FlexPass<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            items,
            constants,
            scroll_box,
            lines,
            sizes,
            used_overflow,
            no_eligible_items,
        } = self;
        let absolute = AbsoluteChildren::begin(tree, node, &constants, scroll_box)?;
        Ok(FlexPass {
            absolute,
            absolute_contributions: Vec::new(),
            layout_constants: constants,
            final_scroll_box: scroll_box,
            final_items: items.finish(),
            lines,
            sizes,
            used_overflow,
            no_eligible_items,
        })
    }
}

struct FlexPass<Node, S: LayoutScalar> {
    absolute: AbsoluteChildren<Node, S>,
    absolute_contributions: Vec<FlexChildContribution<S>>,
    layout_constants: Constants<S>,
    final_scroll_box: CanonicalScrollBoxOf<S>,
    final_items: Vec<FinalFlexItem<Node, S>>,
    lines: Vec<FlexLine<S>>,
    sizes: FlexContainerSizes<S>,
    used_overflow: crate::scroll::UsedOverflow,
    no_eligible_items: bool,
}

fn begin_flex_pass<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    input: ComputeInputOf<S>,
    contained_size: Size<Option<S>>,
) -> FlexPassStart<'input, Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let (constants, own_ratio_cross, used_overflow) =
        with_flex_container_projection::<Tree, M, _>(tree, node, |style| {
            self::input::with_flex_scroll_projections::<Tree, M, _>(tree, node, |scroll_box, _| {
                let constants = Constants::new::<Tree, M>(
                    tree,
                    node,
                    &style,
                    scroll_box,
                    input,
                    contained_size,
                )?;
                let own_ratio_cross = OwnRatioCross::for_preferred_inline(&constants, &style);
                Ok((constants, own_ratio_cross, scroll_box.used_overflow()))
            })
        })?;
    if input.run_mode() == RunMode::ComputeSize
        && !input.physical_baseline_demand()
        && let Size {
            width: Some(width),
            height: Some(height),
        } = constants.node_outer_size
    {
        let output = ComputeOutputOf::<S>::from_outer_size(Size::new(width, height));
        // A requested named fact needs the actual eligible item set even when
        // known dimensions make child layout unnecessary.
        return Ok(ControlFlow::Break(
            if input.named_baseline_demand() != crate::output::NamedBaselineDemand::None
                && !has_eligible_items::<Tree, M>(tree, node)
            {
                output.with_named_baselines(crate::NamedBaselinesOf::absent())
            } else {
                output
            },
        ));
    }

    let requested_inline_operation = constants
        .flow_axes
        .logical_size(input.sizing_request().axes())
        .inline
        .operation();
    let cross_axis = constants.axes.cross_physical_axis();
    let inherited_indefinite_cross = input.indefinite_percentage_axis() == Some(cross_axis);
    let measure_ratio_contents = own_ratio_cross.is_some()
        || (inherited_indefinite_cross
            && constants.preferred_inline_phase != PreferredInlinePhase::Ordinary
            && tree.node_input(node).aspect_ratio.is_some());
    let contents_run_mode = if measure_ratio_contents {
        // Final placement supplies the used item dimensions. Until then a
        // percentage with no qualified cross basis is still indefinite.
        RunMode::ComputeSize
    } else {
        input.run_mode()
    };
    let first_run_mode = if constants.preferred_inline_phase == PreferredInlinePhase::Discover {
        // Preferred-size discovery has no accepted percentage basis yet.
        RunMode::ComputeSize
    } else {
        contents_run_mode
    };
    Ok(ControlFlow::Continue(Box::new(FlexPhase::Collect(
        Box::new(FlexCollection {
            phase: CollectionPhase::Initial {
                own_ratio_cross,
                contents_run_mode,
                requested_inline_operation,
                inherited_indefinite_cross,
                first_run_mode,
            },
            items: ItemCollection::new(tree, node, first_run_mode),
            constants,
            used_overflow,
        }),
    ))))
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> FlexCollection<'input, Node, S> {
    fn finish<M>(self, node: Node) -> LayoutResultOf<Node, FlexResolution<'input, Node, S>, S, M> {
        let Self {
            phase,
            items,
            constants,
            used_overflow,
        } = self;
        let items = items.finish();
        let run_mode = match phase {
            CollectionPhase::Initial { first_run_mode, .. } => first_run_mode,
            CollectionPhase::Accepted { contents_run_mode } => contents_run_mode,
            CollectionPhase::RatioMinimum(_) => RunMode::ComputeSize,
        };
        let rounds = FlexRounds::begin(node, &items, constants, run_mode)?;
        let collection = CollectedFlexItems {
            phase,
            items,
            constants,
            used_overflow,
        };
        Ok(FlexResolution { collection, rounds })
    }
}

fn finish_flex_collection<'input, Tree, M, S>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<S>,
    CollectedFlexItems {
        phase,
        items: collected_items,
        constants,
        used_overflow,
    }: CollectedFlexItems<Tree::Node, S>,
    (layout_constants, resolved_items, lines): ResolvedFlexRound<Tree::Node, Tree::Scalar>,
) -> FlexPassStart<'input, Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let phase = match phase {
        CollectionPhase::RatioMinimum(ratio) => {
            let axes = ratio.contents.constants.axes;
            let measured_cross = axes
                .cross_size(layout_constants.node_outer_size)
                .expect("Flex cross measurement resolves the measured outer size");
            let cross = ratio.transferred.max(measured_cross);
            return ratio
                .contents
                .collect(tree, node, input, used_overflow, Some(cross))
                .map(ControlFlow::Continue);
        }
        phase => phase,
    };
    let sizes = container_sizes(input, &layout_constants, &resolved_items, &lines);
    // Normal size queries need the same accepted inline frame as publication.
    // Constructing this internal box does not stage output geometry.
    let final_scroll_box =
        resolved_flex_scroll_box(tree, node, input, &layout_constants, sizes.output)?;
    if let CollectionPhase::Initial {
        own_ratio_cross,
        contents_run_mode,
        requested_inline_operation,
        inherited_indefinite_cross,
        ..
    } = phase
        && (!collected_items.is_empty() || own_ratio_cross.is_some())
        && let Some(contents_constants) = constants
            .complete_inline_contents(
                final_scroll_box,
                requested_inline_operation,
                own_ratio_cross.is_some(),
            )
            .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?
    {
        let contents = AcceptedFlexContents {
            constants: contents_constants,
            scroll_box: final_scroll_box,
            run_mode: contents_run_mode,
            qualify_ratio_cross: !inherited_indefinite_cross
                && constants
                    .axes
                    .main_size(constants.entry_percentage_basis)
                    .is_some(),
        };
        if let Some(ratio) = own_ratio_cross {
            let axes = contents.constants.axes;
            let main = axes
                .main_size(contents.constants.node_outer_size)
                .expect("the ratio settles after the preferred inline size");
            let transferred = axes
                .cross_size_from_main_in_sizing_box(main, ratio.value, ratio.sizing_box_insets)
                .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
            if ratio.needs_content_minimum {
                // Measure the content floor at the accepted inline size with no
                // provisional cross size. Apply the authored maximum after measuring.
                let mut minimum = contents.constants;
                minimum.max_outer_size = axes.with_cross_size(minimum.max_outer_size, None);
                minimum.max_inner_size = axes.with_cross_size(minimum.max_inner_size, None);
                let cross_request = AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::MinContent,
                );
                minimum.sizing_request = minimum
                    .sizing_request
                    .with_axes(axes.with_cross_size(minimum.sizing_request.axes(), cross_request));
                return Ok(ControlFlow::Continue(Box::new(FlexPhase::Collect(
                    Box::new(FlexCollection {
                        phase: CollectionPhase::RatioMinimum(Box::new(FlexRatioMinimum {
                            contents,
                            transferred,
                        })),
                        items: ItemCollection::new(tree, node, RunMode::ComputeSize),
                        constants: minimum,
                        used_overflow,
                    }),
                ))));
            }
            return contents
                .collect(tree, node, input, used_overflow, Some(transferred))
                .map(ControlFlow::Continue);
        }
        return contents
            .collect(tree, node, input, used_overflow, None)
            .map(ControlFlow::Continue);
    }
    let no_eligible_items = collected_items.is_empty();
    if input.run_mode().is_perform_layout() {
        let inset_basis = crate::output::MeasuredInsetBasisOf::from_used_content_size(
            final_scroll_box.content_box().size(),
        );
        let items = FinalItems::begin(tree, node, &collected_items, resolved_items, inset_basis);
        return Ok(ControlFlow::Continue(Box::new(FlexPhase::Place(
            FlexPlacement {
                items,
                constants: layout_constants,
                scroll_box: final_scroll_box,
                lines,
                sizes,
                used_overflow,
                no_eligible_items,
            },
        ))));
    }
    if input.physical_baseline_demand() {
        return Ok(ControlFlow::Continue(Box::new(FlexPhase::Baseline(
            FlexBaselineExport {
                baselines: BaselineExport::new(resolved_items, lines),
                constants: layout_constants,
                sizes,
                used_overflow,
                no_eligible_items,
            },
        ))));
    }
    let baselines = resolved_vertical_baselines(&resolved_items, &lines, &layout_constants)?;
    finish_flex_size(
        node,
        &layout_constants,
        sizes,
        baselines,
        used_overflow,
        no_eligible_items,
    )
    .map(ControlFlow::Break)
}

fn finish_flex_size<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    layout_constants: &Constants<S>,
    sizes: FlexContainerSizes<S>,
    baselines: BaselinesOf<S>,
    used_overflow: crate::scroll::UsedOverflow,
    no_eligible_items: bool,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
    DistributionNumber::finite(layout_constants.axes.main_size(sizes.intrinsic_content)).map_err(
        |value| {
            crate::error::sizing_resolution_error(
                node,
                LengthResolutionStatus::InvalidNumeric { value }.into(),
            )
        },
    )?;
    flex_output(
        node,
        sizes.output,
        sizes.intrinsic_content,
        baselines,
        used_overflow.is_scroll_container(),
        no_eligible_items,
    )
}

impl<Node: Copy, S: LayoutScalar> FlexPass<Node, S> {
    fn finish<'input, Tree, M>(
        self,
        tree: &mut Tree,
        node: Node,
        input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            absolute: _,
            absolute_contributions,
            layout_constants,
            final_scroll_box,
            final_items,
            lines,
            sizes,
            used_overflow,
            no_eligible_items,
        } = self;
        layout_hidden_children(
            tree,
            node,
            layout_constants.axes.flow_axes(),
            layout_constants.settled_auto_scrollbars,
        )?;
        let scroll_box = final_scroll_box;
        let range_error = |error| {
            flex_scroll_contribution_error(
                node,
                input.run_mode(),
                layout_constants.axes.main_physical_axis(),
                error,
            )
        };
        let mut contributions = flex_scroll_contributions(
            &final_items,
            &absolute_contributions,
            &lines,
            &layout_constants,
            scroll_box,
        )
        .map_err(range_error)?;
        contributions.exclude_reserved_gutter_from_range();
        let scroll_geometry = self::input::with_flex_scroll_projections::<Tree, M, _>(
            tree,
            node,
            |scroll_box_projection, scroll_target_projection| {
                flex_container_scroll_geometry::<_, S, M>(
                    node,
                    input.run_mode(),
                    scroll_box_projection,
                    scroll_target_projection,
                    &layout_constants,
                    scroll_box,
                    contributions,
                )
            },
        )?;
        let content_size = contributions
            .content_size_from_anchor(scroll_geometry.content_box().origin())
            .map_err(range_error)?;
        let baselines = BaselinesOf {
            first: first_final_vertical_baseline(&final_items, &lines, &layout_constants)
                .unwrap_or(Point::NONE),
            last: last_final_vertical_baseline(&final_items, &lines, &layout_constants)
                .unwrap_or(Point::NONE),
        };
        let output = flex_output(
            node,
            sizes.output,
            content_size,
            baselines,
            used_overflow.is_scroll_container(),
            no_eligible_items,
        )?;
        Ok(retain_flex_scroll_geometry(output, scroll_geometry))
    }
}

fn flex_output<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    size: Size<S>,
    content_size: Size<S>,
    baselines: BaselinesOf<S>,
    is_scroll_container: bool,
    no_eligible_items: bool,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
    let baselines = if is_scroll_container {
        baselines.clamped_to_border(size).map_err(
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
    let output = ComputeOutputOf::from_sizes_and_baselines(size, content_size, baselines);
    // Inspect the original eligible collection, not the resolved lines: a
    // nonempty collection of collapsed items is not proof of named absence.
    let output = if no_eligible_items {
        output.with_named_baselines(crate::NamedBaselinesOf::absent())
    } else {
        output
    };
    Ok(output)
}

fn resolved_flex_scroll_box<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    size: Size<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, CanonicalScrollBoxOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    DistributionNumber::finite(constants.axes.main_size(size)).map_err(|value| {
        crate::error::sizing_resolution_error(
            node,
            LengthResolutionStatus::InvalidNumeric { value }.into(),
        )
    })?;
    self::input::with_flex_scroll_projections::<Tree, M, _>(tree, node, |projection, _| {
        flex_container_scroll_box::<_, Tree::Scalar, M>(
            node,
            input.run_mode(),
            projection,
            constants,
            size,
        )
    })
}

impl<S: LayoutScalar> AcceptedFlexContents<S> {
    fn collect<'input, Tree, M>(
        self,
        tree: &mut Tree,
        node: Tree::Node,
        input: ComputeInputOf<S>,
        used_overflow: crate::scroll::UsedOverflow,
        ratio_cross: Option<S>,
    ) -> LayoutResultOf<Tree::Node, Box<FlexPhase<'input, Tree::Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let Self {
            mut constants,
            scroll_box,
            run_mode,
            qualify_ratio_cross,
        } = self;
        if let Some(cross) = ratio_cross {
            let axes = constants.axes;
            let cross = cross
                .clamp_max_before_min_optional(
                    axes.cross_size(constants.min_outer_size),
                    axes.cross_size(constants.max_outer_size),
                )
                .max(axes.cross_size(constants.padding_border_size));
            let size = axes.with_cross_size(scroll_box.border_box().size(), cross);
            let settled = resolved_flex_scroll_box(tree, node, input, &constants, size)?;
            constants = constants.with_final_scroll_box(settled);
            if qualify_ratio_cross {
                // Only an independently definite determining axis grants
                // percentage authority to the size transferred by the ratio.
                let cross_basis = Some(axes.cross_size(settled.content_box().size()));
                constants.percentage_parent_size =
                    axes.with_cross_size(constants.percentage_parent_size, cross_basis);
                constants.entry_percentage_basis =
                    axes.with_cross_size(constants.entry_percentage_basis, cross_basis);
            }
            constants.max_inner_size = constants
                .max_outer_size
                .sub_optional_unchecked(constants.content_box_inset.sum_axes())
                .max_optional(Size::ZERO.map(Some));
        }
        Ok(Box::new(FlexPhase::Collect(Box::new(FlexCollection {
            phase: CollectionPhase::Accepted {
                contents_run_mode: run_mode,
            },
            items: ItemCollection::new(tree, node, run_mode),
            constants,
            used_overflow,
        }))))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreferredInlinePhase {
    Ordinary,
    Discover,
    Accepted,
}

#[derive(Clone, Copy)]
struct OwnRatioCross<S: LayoutScalar> {
    value: AspectRatioOf<S>,
    sizing_box_insets: Size<S>,
    needs_content_minimum: bool,
}

impl<S: LayoutScalar> OwnRatioCross<S> {
    fn for_preferred_inline(
        constants: &Constants<S>,
        style: &FlexContainerProjection<'_, S>,
    ) -> Option<Self> {
        let axes = constants.axes;
        let value = (*style.common.aspect_ratio)?;
        let preferred = axes.cross_size(Size::new(
            &style.common.size.width,
            &style.common.size.height,
        ));
        if constants.preferred_inline_phase == PreferredInlinePhase::Ordinary
            || preferred.is_max_content()
            || axes.cross_size(constants.node_outer_size).is_some()
        {
            return None;
        }

        let cross_overflow = axes.cross_size(Size::new(
            style.common.overflow.x(),
            style.common.overflow.y(),
        ));
        let minimum = axes.cross_size(Size::new(
            &style.common.min_size.width,
            &style.common.min_size.height,
        ));
        let sizing_box_insets = if style.common.box_sizing == BoxSizing::ContentBox {
            constants.padding_border_size
        } else {
            Size::ZERO
        };
        Some(Self {
            value,
            sizing_box_insets,
            needs_content_minimum: minimum.is_auto()
                && !style.common.item_is_replaced
                && !cross_overflow.is_scrollable(),
        })
    }
}

#[derive(Clone, Copy)]
struct Constants<S: LayoutScalar> {
    flow_axes: crate::geometry::FlowAxes,
    axes: FlexAxes,
    node_outer_size: Size<Option<S>>,
    node_inner_size: Size<Option<S>>,
    preferred_inline_phase: PreferredInlinePhase,
    cross_content_offer: Option<S>,
    percentage_parent_size: Size<Option<S>>,
    physical_baseline_demand: bool,
    min_outer_size: Size<Option<S>>,
    max_outer_size: Size<Option<S>>,
    max_inner_size: Size<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    padding_border_size: Size<S>,
    scrollport_inset: Edges<S>,
    non_gutter_box_inset: Edges<S>,
    content_box_inset: Edges<S>,
    settled_auto_scrollbars: crate::scroll::SettledAutoScrollbarState,
    gap: Size<S>,
    gap_input: Size<LengthOf<S>>,
    // Independently qualified dimensions for this scrollbar pass. Intrinsic
    // inline replay can supply a contents basis without changing these facts.
    entry_percentage_basis: Size<Option<S>>,
    align_items: AlignItems,
    authored_align_content: Option<AlignContent>,
    align_content: AlignContent,
    authored_justify_content: Option<AlignContent>,
    justify_content: AlignContent,
    wraps: bool,
    sizing_request: crate::SizingRequestOf<S>,
    positioned_fallback_limits: Size<crate::OrthogonalFallbackLimitOf<S>>,
    available_main: AxisSizingRequestOf<S>,
}

impl<S: LayoutScalar> Constants<S> {
    fn new<'input, Tree, M>(
        tree: &Tree,
        node: <Tree as Traverse>::Node,
        style: &FlexContainerProjection<'_, S>,
        scroll_box_projection: crate::scroll::ScrollBoxProjection<'_, S>,
        input: ComputeInputOf<S>,
        contained_size: Size<Option<S>>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, Self, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let flow_axes = style.common.flow_axes;
        let axes = FlexAxes::new(flow_axes, style.flex_direction, style.flex_wrap);
        let (padding, border) = resolve_containing_padding_border(
            input.containing_flow_axes(),
            input.parent(),
            *style.common.padding,
            *style.common.border,
            resolve_length_or_zero,
            |edges| edges.transpose_with_node(tree, node),
        )?;
        let padding_border = (padding + border).sum_axes();
        let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
            padding_border
        } else {
            Size::<S>::ZERO
        };

        let preferred = Size::new(&style.common.size.width, &style.common.size.height);
        let row_preferred_size = input.sizing_mode() == SizingMode::InherentSize
            && axes.main_logical_axis() == LogicalAxis::Inline;
        let max_content_main = row_preferred_size && axes.main_size(preferred).is_max_content();
        let max_content_cross = row_preferred_size && axes.cross_size(preferred).is_max_content();
        let (style_size, min_size, max_size) = match input.sizing_mode() {
            SizingMode::ContentSize => (Size::NONE, Size::NONE, Size::NONE),
            SizingMode::PositionedColumnFlexInline => {
                let main_axis = axes.main_physical_axis();
                let basis = axes.main_size(input.parent());
                let minimum = resolve_minimum_optional(
                    axes.main_size(Size::new(
                        &style.common.min_size.width,
                        &style.common.min_size.height,
                    )),
                    SizingAlgorithm::Flex,
                    main_axis,
                    basis,
                    true,
                )
                .map_err(|error| crate::error::sizing_resolution_error(node, error))?;
                let maximum = resolve_maximum_optional(
                    axes.main_size(Size::new(
                        &style.common.max_size.width,
                        &style.common.max_size.height,
                    )),
                    SizingAlgorithm::Flex,
                    main_axis,
                    basis,
                    true,
                )
                .map_err(|error| crate::error::sizing_resolution_error(node, error))?;
                (
                    Size::NONE,
                    axes.with_main_size(Size::NONE, minimum)
                        .add_optional(box_sizing_adjustment),
                    axes.with_main_size(Size::NONE, maximum)
                        .add_optional(box_sizing_adjustment),
                )
            }
            SizingMode::InherentSize => {
                let resolve_preferred = |value, axis, basis| {
                    if max_content_main && axis == axes.main_physical_axis()
                        || max_content_cross && axis == axes.cross_physical_axis()
                    {
                        Ok(None)
                    } else {
                        resolve_preferred_optional(value, SizingAlgorithm::Flex, axis, basis, true)
                    }
                };
                let style_size = Size::new(
                    resolve_preferred(
                        preferred.width,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                    ),
                    resolve_preferred(
                        preferred.height,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let min_size = Size::new(
                    resolve_minimum_optional(
                        &style.common.min_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_minimum_optional(
                        &style.common.min_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let max_size = Size::new(
                    resolve_maximum_optional(
                        &style.common.max_size.width,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Horizontal,
                        input.parent().width,
                        true,
                    ),
                    resolve_maximum_optional(
                        &style.common.max_size.height,
                        SizingAlgorithm::Flex,
                        PhysicalAxis::Vertical,
                        input.parent().height,
                        true,
                    ),
                )
                .transpose_with_node(tree, node)?;
                let ratio = *style.common.aspect_ratio;
                let (style_size, min_size, max_size) = if max_content_main || max_content_cross {
                    let (min_size, max_size) = ratio.map_or((min_size, max_size), |ratio| {
                        transfer_aspect_ratio_limits(style_size, min_size, max_size, ratio.get())
                    });
                    (style_size, min_size, max_size)
                } else {
                    (
                        style_size.apply_aspect_ratio(ratio),
                        min_size.apply_aspect_ratio(ratio),
                        max_size.apply_aspect_ratio(ratio),
                    )
                };
                (
                    style_size.add_optional(box_sizing_adjustment),
                    min_size.add_optional(box_sizing_adjustment),
                    max_size.add_optional(box_sizing_adjustment),
                )
            }
        };
        let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
            (Some(min), Some(max)) if max <= min => Some(min),
            _ => None,
        });
        let definite_outer_size = input
            .known()
            .or(min_max_definite_size
                .or(style_size.clamp_max_before_min_optional(min_size, max_size)))
            .max_optional(padding_border.map(Some));
        let node_outer_size = contained_size.or(definite_outer_size);
        let mut entry_axes = input.sizing_request().axes();
        // Preferred intrinsic sizes use Flex contributions independently of the
        // finite containing offer. An already imposed size retains its request.
        let intrinsic = preferred_intrinsic_request();
        let preferred_inline_phase = if !max_content_main {
            PreferredInlinePhase::Ordinary
        } else if axes.main_size(node_outer_size).is_some() {
            PreferredInlinePhase::Accepted
        } else {
            PreferredInlinePhase::Discover
        };
        if preferred_inline_phase == PreferredInlinePhase::Discover {
            entry_axes = axes.with_main_size(entry_axes, intrinsic);
        }
        if max_content_cross && axes.cross_size(node_outer_size).is_none() {
            entry_axes = axes.with_cross_size(entry_axes, intrinsic);
        }
        let margin_debit = if input.run_mode() == RunMode::PerformRootLayout
            && input.parent_formatting_context() == ParentFormattingContext::NoParent
        {
            let unresolved = node_outer_size.zip_map(entry_axes, |size, request| {
                size.is_none() && crate::layout_math::normal_available_size(request).is_some()
            });
            if unresolved.width || unresolved.height {
                input
                    .containing_flow_axes()
                    .zip_physical_edges_with_inline_extent(
                        *style.common.margin,
                        input.parent(),
                        resolve_auto_or_zero,
                    )
                    .transpose_with_node(tree, node)?
                    .sum_axes()
                    .zip_map(
                        unresolved,
                        |margin, unresolved| {
                            if unresolved { margin } else { S::ZERO }
                        },
                    )
            } else {
                Size::ZERO
            }
        } else {
            Size::ZERO
        };
        let border_box_axes = if margin_debit == Size::ZERO {
            entry_axes
        } else {
            crate::layout_math::transpose_axis_requests_at_node(
                node,
                entry_axes.zip_map(margin_debit, crate::layout_math::inset_axis_request),
            )?
        };
        let mut scroll_box_source = CanonicalScrollBoxSourceOf::from_projection(
            scroll_box_projection,
            Size::ZERO,
            border,
            padding,
            input.settled_auto_scrollbars(),
        );
        let unconstrained_scroll_box_size = padding_border
            + Size::splat(
                scroll_box_source.scrollbar_width.get() + scroll_box_source.scrollbar_width.get(),
            );
        let scroll_box_size = node_outer_size
            .or(border_box_axes.map(|request| {
                request
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get)
            }))
            .or(max_size)
            .unwrap_or(unconstrained_scroll_box_size)
            .zip_map(padding_border, |size, minimum| size.max(minimum));
        scroll_box_source.border_box_size = scroll_box_size;
        let scroll_box = canonical_scroll_box_from_source(scroll_box_source)
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        let non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
        let content_box_inset = scroll_box.content_box_inset();
        let content_box_inset_size = content_box_inset.sum_axes();
        let content_axes = crate::layout_math::transpose_axis_requests_at_node(
            node,
            entry_axes.zip_map(
                content_box_inset_size + margin_debit,
                crate::layout_math::inset_axis_request,
            ),
        )?;
        let node_inner_size = node_outer_size
            .sub_optional_unchecked(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let percentage_parent_size = input.descendant_percentage_basis(
            node_inner_size.zip_map(definite_outer_size, |inner, definite| definite.and(inner)),
        );
        let max_inner_size = max_size
            .sub_optional_unchecked(content_box_inset_size)
            .max_optional(Size::<S>::ZERO.map(Some));
        let gap = style
            .gap
            .zip_map(percentage_parent_size, |length, basis| {
                resolve_length_or_zero(length, Some(basis.unwrap_or(S::ZERO)))
            })
            .transpose_with_node(tree, node)?;

        let fallback_source = crate::scroll::MeasuredLeafContentBoxInsetSourceOf {
            flow_axes,
            computed_overflow: scroll_box_source.computed_overflow,
            item_is_replaced: scroll_box_source.item_is_replaced,
            scrollbar_gutter: scroll_box_source.scrollbar_gutter,
            scrollbar_width: scroll_box_source.scrollbar_width,
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
            SizingAlgorithm::Flex,
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
        Ok(Self {
            flow_axes,
            axes,
            node_outer_size,
            node_inner_size,
            preferred_inline_phase,
            cross_content_offer: None,
            percentage_parent_size,
            physical_baseline_demand: input.physical_baseline_demand(),
            min_outer_size: min_size,
            max_outer_size: max_size,
            max_inner_size,
            border,
            padding,
            padding_border_size: padding_border,
            scrollport_inset,
            non_gutter_box_inset,
            content_box_inset,
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            gap,
            gap_input: *style.gap,
            entry_percentage_basis: percentage_parent_size,
            align_items: style.align_items.unwrap_or(AlignItems::Stretch),
            authored_align_content: style.align_content,
            align_content: style.align_content.unwrap_or(AlignContent::Stretch),
            authored_justify_content: style.justify_content,
            justify_content: style.justify_content.unwrap_or(AlignContent::FlexStart),
            wraps: matches!(
                style.flex_wrap,
                super::FlexWrap::Wrap | super::FlexWrap::WrapReverse
            ),
            sizing_request: input
                .sizing_request()
                .with_axes(content_axes)
                .with_orthogonal_fallback(fallback),
            positioned_fallback_limits,
            available_main: axes.main_size(content_axes),
        })
    }

    fn is_wrapping_column(&self) -> bool {
        self.wraps && self.axes.main_logical_axis() == LogicalAxis::Block
    }

    fn cross_content_extent(&self) -> Option<S> {
        self.cross_content_offer
            .or(self.axes.cross_size(self.node_inner_size))
    }

    fn complete_inline_contents(
        self,
        accepted: CanonicalScrollBoxOf<S>,
        requested_inline_operation: crate::SizingOperation,
        settle_cross: bool,
    ) -> Result<Option<Self>, LengthResolutionStatus<S>> {
        let axes = self.flow_axes;
        let mut percentage_parent = axes.logical_size(self.percentage_parent_size);
        let normal = requested_inline_operation == crate::SizingOperation::Normal;
        if !settle_cross
            && self.preferred_inline_phase != PreferredInlinePhase::Discover
            && (!normal || percentage_parent.inline.is_some())
        {
            return Ok(None);
        }
        // Use the accepted unrounded inline content extent. All block sizing,
        // availability and maximum authority still come from the entry context.
        let mut outer = axes.logical_size(self.node_outer_size);
        let mut inner = axes.logical_size(self.node_inner_size);
        let accepted_inline = axes.logical_size(accepted.content_box().size()).inline;
        outer.inline = Some(axes.logical_size(accepted.border_box().size()).inline);
        inner.inline = Some(accepted_inline);
        if normal {
            percentage_parent.inline = inner.inline;
        }
        let inline_edges = |original, used| {
            let mut original = axes.logical_edges(original);
            let used = axes.logical_edges(used);
            original.inline_start = used.inline_start;
            original.inline_end = used.inline_end;
            axes.physical_edges(original)
        };
        let mut contents = self;
        contents.node_outer_size = axes.physical_size(outer);
        contents.node_inner_size = axes.physical_size(inner);
        contents.percentage_parent_size = axes.physical_size(percentage_parent);
        let mut requests = axes.logical_size(self.sizing_request.axes());
        if self.preferred_inline_phase == PreferredInlinePhase::Discover {
            // A completed property measurement does not select the caller's
            // sizing operation or establish a formal percentage basis.
            requests.inline = AxisSizingRequestOf::new(
                crate::layout_math::checked_available_space(accepted_inline)?,
                requested_inline_operation,
            );
            contents.sizing_request = self.sizing_request.with_axes(axes.physical_size(requests));
            contents.available_main = self.axes.main_size(contents.sizing_request.axes());
            contents.preferred_inline_phase = PreferredInlinePhase::Accepted;
        }
        contents.scrollport_inset = inline_edges(
            self.scrollport_inset,
            accepted.effective_border() + accepted.effective_gutter(),
        );
        contents.content_box_inset =
            inline_edges(self.content_box_inset, accepted.content_box_inset());
        contents.max_inner_size = self
            .max_outer_size
            .sub_optional_unchecked(contents.content_box_inset.sum_axes())
            .max_optional(Size::ZERO.map(Some));
        Ok(Some(contents))
    }

    fn with_final_scroll_box(mut self, scroll_box: CanonicalScrollBoxOf<S>) -> Self {
        self.node_outer_size = scroll_box.border_box().size().map(Some);
        self.node_inner_size = scroll_box.content_box().size().map(Some);
        self.percentage_parent_size = self
            .percentage_parent_size
            .zip_map(scroll_box.content_box().size(), |original, used| {
                original.map(|_| used)
            });
        self.scrollport_inset = scroll_box.effective_border() + scroll_box.effective_gutter();
        self.non_gutter_box_inset = scroll_box.effective_border() + scroll_box.effective_padding();
        self.content_box_inset = scroll_box.content_box_inset();
        self
    }
}

/// Resolved flex main/cross roles for one container.
///
/// This is the sole flex owner for translating a container's logical flow into
/// physical axes, sides, and progressions. It is derived only from the
/// container's `FlowAxes`, `FlexDirection`, and `FlexWrap`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FlexAxes {
    flow_axes: FlowAxes,
    main_logical_axis: LogicalAxis,
    cross_logical_axis: LogicalAxis,
    main_physical_axis: PhysicalAxis,
    cross_physical_axis: PhysicalAxis,
    main_start_side: PhysicalSide,
    main_end_side: PhysicalSide,
    cross_start_side: PhysicalSide,
    cross_end_side: PhysicalSide,
    main_reversed: bool,
    cross_reversed: bool,
    main_progression: PhysicalProgression,
    cross_progression: PhysicalProgression,
}

impl FlexAxes {
    #[must_use]
    pub(crate) const fn new(
        flow_axes: FlowAxes,
        flex_direction: FlexDirection,
        flex_wrap: FlexWrap,
    ) -> Self {
        let main_reversed = flex_direction.is_reverse();
        let cross_reversed = matches!(flex_wrap, FlexWrap::WrapReverse);
        let (
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            normal_main_start_side,
            normal_main_end_side,
            normal_cross_start_side,
            normal_cross_end_side,
        ) = if flex_direction.is_row() {
            (
                LogicalAxis::Inline,
                LogicalAxis::Block,
                flow_axes.inline_axis(),
                flow_axes.block_axis(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
                flow_axes.block_start(),
                flow_axes.block_end(),
            )
        } else {
            (
                LogicalAxis::Block,
                LogicalAxis::Inline,
                flow_axes.block_axis(),
                flow_axes.inline_axis(),
                flow_axes.block_start(),
                flow_axes.block_end(),
                flow_axes.inline_start(),
                flow_axes.inline_end(),
            )
        };
        let (main_start_side, main_end_side) = if main_reversed {
            (normal_main_end_side, normal_main_start_side)
        } else {
            (normal_main_start_side, normal_main_end_side)
        };
        let (cross_start_side, cross_end_side) = if cross_reversed {
            (normal_cross_end_side, normal_cross_start_side)
        } else {
            (normal_cross_start_side, normal_cross_end_side)
        };

        Self {
            flow_axes,
            main_logical_axis,
            cross_logical_axis,
            main_physical_axis,
            cross_physical_axis,
            main_start_side,
            main_end_side,
            cross_start_side,
            cross_end_side,
            main_reversed,
            cross_reversed,
            main_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(main_physical_axis),
                main_reversed,
            ),
            cross_progression: Self::reverse_progression(
                flow_axes.physical_axis_progression(cross_physical_axis),
                cross_reversed,
            ),
        }
    }

    #[must_use]
    pub(crate) const fn flow_axes(self) -> FlowAxes {
        self.flow_axes
    }

    #[must_use]
    pub(crate) const fn flow_direction(self) -> Direction {
        self.flow_axes.direction()
    }

    #[must_use]
    pub(crate) const fn scroll_origin_axes(self) -> ScrollOriginAxes {
        let main = if self.main_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        let cross = if self.cross_reversed {
            ScrollOriginProgression::FlowStartward
        } else {
            ScrollOriginProgression::FlowEndward
        };
        match self.main_logical_axis {
            LogicalAxis::Inline => ScrollOriginAxes::new(main, cross),
            LogicalAxis::Block => ScrollOriginAxes::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) const fn main_logical_axis(self) -> LogicalAxis {
        self.main_logical_axis
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_logical_axis(self) -> LogicalAxis {
        self.cross_logical_axis
    }

    #[must_use]
    pub(crate) const fn main_physical_axis(self) -> PhysicalAxis {
        self.main_physical_axis
    }

    #[must_use]
    pub(crate) const fn cross_physical_axis(self) -> PhysicalAxis {
        self.cross_physical_axis
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_start_side(self) -> PhysicalSide {
        self.main_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_end_side(self) -> PhysicalSide {
        self.main_end_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_start_side(self) -> PhysicalSide {
        self.cross_start_side
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_end_side(self) -> PhysicalSide {
        self.cross_end_side
    }

    #[must_use]
    pub(crate) const fn main_is_reversed(self) -> bool {
        self.main_reversed
    }

    #[must_use]
    pub(crate) const fn cross_is_reversed(self) -> bool {
        self.cross_reversed
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn main_progression(self) -> PhysicalProgression {
        self.main_progression
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) const fn cross_progression(self) -> PhysicalProgression {
        self.cross_progression
    }

    #[must_use]
    pub(crate) fn main_size<T>(self, size: Size<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn cross_size<T>(self, size: Size<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => size.width,
            PhysicalAxis::Vertical => size.height,
        }
    }

    #[must_use]
    pub(crate) fn size_from_main_cross<T>(self, main: T, cross: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(main, cross),
            PhysicalAxis::Vertical => Size::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn with_main_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    pub(crate) fn with_cross_size<T>(self, size: Size<T>, value: T) -> Size<T> {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => Size::new(value, size.height),
            PhysicalAxis::Vertical => Size::new(size.width, value),
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn main_point<T>(self, point: Point<T>) -> T {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn cross_point<T>(self, point: Point<T>) -> T {
        match self.cross_physical_axis {
            PhysicalAxis::Horizontal => point.x,
            PhysicalAxis::Vertical => point.y,
        }
    }

    #[must_use]
    pub(crate) fn point_from_main_cross<T>(self, main: T, cross: T) -> Point<T> {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => Point::new(main, cross),
            PhysicalAxis::Vertical => Point::new(cross, main),
        }
    }

    #[must_use]
    pub(crate) fn main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_start_side)
    }

    #[must_use]
    pub(crate) fn main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.main_end_side)
    }

    #[must_use]
    pub(crate) fn cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_start_side)
    }

    #[must_use]
    pub(crate) fn cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.cross_end_side)
    }

    #[must_use]
    pub(crate) fn normal_main_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_main_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.main_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_start_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_start_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn normal_cross_end_edge<T: Copy>(self, edges: Edges<T>) -> T {
        edges.at_physical_side(self.normal_axis_end_side(self.cross_logical_axis))
    }

    #[must_use]
    pub(crate) fn main_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.main_start_edge(edges) + self.main_end_edge(edges)
    }

    #[must_use]
    pub(crate) fn cross_edge_sum<S: LayoutScalar>(self, edges: Edges<S>) -> S {
        self.cross_start_edge(edges) + self.cross_end_edge(edges)
    }

    pub(crate) fn set_main_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_start_side, value);
    }

    pub(crate) fn set_main_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.main_end_side, value);
    }

    #[cfg(test)]
    pub(crate) fn set_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_start_side, value);
    }

    #[cfg(test)]
    pub(crate) fn set_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(edges, self.cross_end_side, value);
    }

    pub(crate) fn set_normal_cross_start_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_start_side(self.cross_logical_axis),
            value,
        );
    }

    pub(crate) fn set_normal_cross_end_edge<T>(self, edges: &mut Edges<T>, value: T) {
        self.set_edge_at_side(
            edges,
            self.normal_axis_end_side(self.cross_logical_axis),
            value,
        );
    }

    #[must_use]
    pub(crate) const fn main_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.main_physical_axis)
    }

    #[must_use]
    pub(crate) const fn cross_requested_axis(self) -> RequestedAxis {
        Self::requested_axis_for(self.cross_physical_axis)
    }

    #[must_use]
    pub(crate) fn main_size_from_cross_aspect<S: LayoutScalar>(
        self,
        cross: S,
        aspect_ratio: AspectRatioOf<S>,
    ) -> S {
        match self.main_physical_axis {
            PhysicalAxis::Horizontal => cross * aspect_ratio.get(),
            PhysicalAxis::Vertical => cross / aspect_ratio.get(),
        }
    }

    fn main_size_from_cross_in_sizing_box<S: LayoutScalar>(
        self,
        border_box_cross: S,
        aspect_ratio: AspectRatioOf<S>,
        sizing_box_insets: Size<S>,
    ) -> Result<S, crate::LengthResolutionStatus<S>> {
        let cross = (border_box_cross - self.cross_size(sizing_box_insets)).max(S::ZERO);
        let main = self.main_size_from_cross_aspect(cross, aspect_ratio)
            + self.main_size(sizing_box_insets);
        if main.is_finite() {
            Ok(main)
        } else {
            Err(crate::LengthResolutionStatus::InvalidNumeric { value: main })
        }
    }

    fn cross_size_from_main_in_sizing_box<S: LayoutScalar>(
        self,
        border_box_main: S,
        aspect_ratio: AspectRatioOf<S>,
        sizing_box_insets: Size<S>,
    ) -> Result<S, LengthResolutionStatus<S>> {
        let main = (border_box_main - self.main_size(sizing_box_insets)).max(S::ZERO);
        let cross = match self.main_physical_axis {
            PhysicalAxis::Horizontal => main / aspect_ratio.get(),
            PhysicalAxis::Vertical => main * aspect_ratio.get(),
        } + self.cross_size(sizing_box_insets);
        if cross.is_finite() {
            Ok(cross)
        } else {
            Err(LengthResolutionStatus::InvalidNumeric { value: cross })
        }
    }

    #[must_use]
    pub(crate) fn main_position_from_start<
        T: core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
    >(
        self,
        container: Size<T>,
        start_inset: T,
        offset: T,
        item_size: T,
        relative_offset: T,
    ) -> T {
        let coordinate = start_inset + offset + relative_offset;
        match self.main_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.main_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn cross_position_from_start<
        T: core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
    >(
        self,
        container: Size<T>,
        start_inset: T,
        offset: T,
        item_size: T,
        relative_offset: T,
    ) -> T {
        let coordinate = start_inset + offset + relative_offset;
        match self.cross_progression {
            PhysicalProgression::Increasing => coordinate,
            PhysicalProgression::Decreasing => self.cross_size(container) - coordinate - item_size,
        }
    }

    #[must_use]
    pub(crate) fn main_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.main_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn cross_offset_from_normal_flow<S: LayoutScalar>(self, offset: S) -> S {
        if self.cross_reversed { -offset } else { offset }
    }

    #[must_use]
    pub(crate) fn main_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.main_reversed {
            self.main_size(container) - offset - item_size
        } else {
            offset
        };
        self.main_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    pub(crate) fn cross_position_from_normal_start<S: LayoutScalar>(
        self,
        container: Size<S>,
        offset: S,
        item_size: S,
    ) -> S {
        let flex_offset = if self.cross_reversed {
            self.cross_size(container) - offset - item_size
        } else {
            offset
        };
        self.cross_position_from_start(container, S::ZERO, flex_offset, item_size, S::ZERO)
    }

    #[must_use]
    const fn normal_axis_start_side(self, axis: LogicalAxis) -> PhysicalSide {
        match axis {
            LogicalAxis::Inline => self.flow_axes.inline_start(),
            LogicalAxis::Block => self.flow_axes.block_start(),
        }
    }

    #[must_use]
    const fn normal_axis_end_side(self, axis: LogicalAxis) -> PhysicalSide {
        self.normal_axis_start_side(axis).opposite()
    }

    const fn reverse_progression(
        progression: PhysicalProgression,
        reversed: bool,
    ) -> PhysicalProgression {
        if !reversed {
            return progression;
        }

        match progression {
            PhysicalProgression::Increasing => PhysicalProgression::Decreasing,
            PhysicalProgression::Decreasing => PhysicalProgression::Increasing,
        }
    }

    fn set_edge_at_side<T>(self, edges: &mut Edges<T>, side: PhysicalSide, value: T) {
        match side {
            PhysicalSide::Top => edges.top = value,
            PhysicalSide::Right => edges.right = value,
            PhysicalSide::Bottom => edges.bottom = value,
            PhysicalSide::Left => edges.left = value,
        }
    }

    const fn requested_axis_for(axis: PhysicalAxis) -> RequestedAxis {
        match axis {
            PhysicalAxis::Horizontal => RequestedAxis::Horizontal,
            PhysicalAxis::Vertical => RequestedAxis::Vertical,
        }
    }
}

#[derive(Clone, Copy)]
struct FlexContainerSizes<S: LayoutScalar> {
    output: Size<S>,
    intrinsic_content: Size<S>,
}

fn container_sizes<Node, S: LayoutScalar>(
    input: ComputeInputOf<S>,
    constants: &Constants<S>,
    resolved_items: &[ResolvedFlexItem<Node, S>],
    lines: &[FlexLine<S>],
) -> FlexContainerSizes<S> {
    let line_cross_gap =
        constants.axes.cross_size(constants.gap) * S::from_usize(lines.len().saturating_sub(1));
    let content_main = intrinsic_content_main_size(input, constants, resolved_items, lines);
    let content_cross = lines
        .iter()
        .fold(S::ZERO, |sum, line| sum + line.cross_size)
        + line_cross_gap;
    let content_size = constants
        .axes
        .size_from_main_cross(content_main, content_cross);
    let outer_size = constants
        .node_outer_size
        .unwrap_or(content_size + constants.content_box_inset.sum_axes())
        .clamp_max_before_min_optional(constants.min_outer_size, constants.max_outer_size);
    let mut output_size = input
        .known()
        .or(constants.node_outer_size)
        .unwrap_or(outer_size)
        .max_optional(constants.padding_border_size.map(Some));
    if constants
        .axes
        .main_size(constants.node_outer_size)
        .is_none()
        && lines.len() > 1
        && let Some(available_main) = crate::layout_math::normal_available_size(
            constants.axes.main_size(input.sizing_request().axes()),
        )
    {
        output_size = constants.axes.with_main_size(
            output_size,
            constants.axes.main_size(output_size).max(available_main),
        );
    }

    FlexContainerSizes {
        output: output_size,
        intrinsic_content: content_size,
    }
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

#[cfg(test)]
mod fri08_c07_t03_optional_math_characterization_tests {
    use super::*;

    fn characterize<S: LayoutScalar>() {
        let scalar = S::from_f64;

        assert_eq!(
            Size::new(None, Some(scalar(12.0))).max_optional(Size::new(Some(scalar(9.0)), None)),
            Size::new(None, Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(Some(scalar(4.0)), Some(scalar(12.0)))
                .max_optional(Size::new(Some(scalar(9.0)), Some(scalar(3.0)))),
            Size::new(Some(scalar(9.0)), Some(scalar(12.0)))
        );
        assert_eq!(
            Size::new(scalar(4.0), scalar(12.0)).max_optional(Size::new(None, Some(scalar(15.0)))),
            Size::new(scalar(4.0), scalar(15.0))
        );
    }

    #[test]
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri08_c07_t03_optional_math_flex_componentwise_floors_preserve_f64() {
        characterize::<f64>();
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
        containing_flow_axes: FlowAxes,
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
            display: crate::Display::Flex,
            padding: percentage_edges(),
            border,
            ..NodeInputOf::default()
        };
        let mut owner =
            crate::test_support::layout_tree::OracleTreeOf::new().style(7, style.clone());
        let tree = owner.borrow_compute();

        for (flow, parent, expected_padding) in [
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(100.0)),
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), Some(S::from_f64(200.0))),
                expected_percentage_edges(S::from_f64(200.0)),
            ),
            (
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::new(None, Some(S::from_f64(200.0))),
                Edges::ZERO,
            ),
            (
                FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
                Size::new(Some(S::from_f64(100.0)), None),
                Edges::ZERO,
            ),
        ] {
            let constants = super::input::with_flex_container_projection::<
                _,
                core::convert::Infallible,
                _,
            >(&tree, 7, |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &tree,
                            7,
                            &projection,
                            scroll_box,
                            input(flow, parent),
                            Size::NONE,
                        )
                    },
                )
            })
            .expect("flex constants edge characterization must resolve");
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
            display: crate::Display::Flex,
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
        let result = super::input::with_flex_container_projection::<_, core::convert::Infallible, _>(
            &failing_tree,
            7,
            |projection| {
                super::input::with_flex_scroll_projections::<_, core::convert::Infallible, _>(
                    &failing_tree,
                    7,
                    |scroll_box, _| {
                        Constants::new::<_, core::convert::Infallible>(
                            &failing_tree,
                            7,
                            &projection,
                            scroll_box,
                            input(
                                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                                Size::splat(Some(largest)),
                            ),
                            Size::NONE,
                        )
                    },
                )
            },
        );
        let error = match result {
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
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f32() {
        characterize_constants::<f32>(f32::MAX);
    }

    #[test]
    fn fri06_c13_t06_flex_resolution_edges_and_error_order_preserve_f64() {
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
            Size::new(None, Some(scalar(9.0)))
                .sub_optional_unchecked(Size::new(scalar(5.0), scalar(4.0))),
            Size::new(None, Some(scalar(5.0)))
        );
        assert_eq!(
            Size::new(scalar(8.0), scalar(12.0)).clamp_max_before_min_optional(
                Size::new(Some(scalar(3.0)), None),
                Size::new(Some(scalar(10.0)), Some(scalar(11.0))),
            ),
            Size::new(scalar(8.0), scalar(11.0))
        );
        assert_eq!(
            Size::new(Some(scalar(5.0)), Some(scalar(5.0))).clamp_max_before_min_optional(
                Size::new(Some(scalar(10.0)), Some(scalar(10.0))),
                Size::new(Some(scalar(3.0)), Some(scalar(3.0))),
            ),
            Size::new(Some(scalar(10.0)), Some(scalar(10.0)))
        );
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f32() {
        characterize::<f32>();
    }

    #[test]
    fn fri06_c13_t05_flex_unchecked_subtraction_and_clamp_order_preserve_f64() {
        characterize::<f64>();
    }
}
