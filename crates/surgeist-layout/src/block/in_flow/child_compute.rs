use super::super::{baseline_frame, flow_root, inline_run};
use super::*;

pub(super) struct InFlowBoxComputation<'a, 'input, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) child: Node,
    pub(super) index: usize,
    pub(super) child_style: &'a BlockChildProjection<'input, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) prepared: &'a PreparedChild<S>,
    pub(super) adjoining: &'a entry::AdjoiningFlow<S>,
    pub(super) baselines: &'a baseline_frame::PendingBaselines,
    pub(super) float_exclusions: &'a FloatExclusions<S, Node>,
    pub(super) input: ComputeInputOf<S>,
    pub(super) pass_request: crate::SizingRequestOf<S>,
    pub(super) child_run_mode: RunMode,
    pub(super) normal_operation: bool,
    pub(super) physical_float_context: bool,
    pub(super) containing_size: Size<S>,
    pub(super) inner_inline: Option<S>,
    pub(super) content_inline_size: S,
    pub(super) occupied_block_end: S,
}

pub(super) struct InFlowComputedBox<Node, S: LayoutScalar> {
    pub(super) output: ComputeOutputOf<S>,
    pub(super) child_input: ComputeInputOf<S>,
    pub(super) child_margin: Edges<S>,
    pub(super) child_padding: Edges<S>,
    pub(super) child_border: Edges<S>,
    pub(super) logical_child_margin: LogicalEdgesOf<S>,
    pub(super) logical_child_size: LogicalSizeOf<S>,
    pub(super) avoids_float_exclusions: bool,
    pub(super) accepted_float_position: Option<LogicalPointOf<S>>,
    pub(super) shared_child: Option<SharedInFlowPlacement<Node, S>>,
    pub(super) clearance_funding: Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    pub(super) introduced_cut: Option<entry::canonical::Separation<Node, S>>,
}

type InFlowBoxResult<Node, S, M> = ComputeResult<Node, InFlowComputedBox<Node, S>, S, M>;

impl<Node, S: LayoutScalar> InFlowBoxComputation<'_, '_, Node, S> {
    fn parent_entry(&self) -> scope::ParentEntry<S> {
        scope::ParentEntry {
            reference: self.containing_size,
            cursor: self.adjoining.cursor,
            occupied: self.occupied_block_end,
            active: self.adjoining.active,
            top: self.adjoining.top,
            first: self.adjoining.start_open,
        }
    }
}

pub(super) struct InFlowBoxPreparation<S: LayoutScalar> {
    child_input: ComputeInputOf<S>,
    child_indent_context: crate::text_indent::BlockIndentContext<S>,
    parent_logical_unresolved_margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    parent_inline_basis: Option<S>,
    avoids_float_exclusions: bool,
    pub(super) inherits_float_exclusions: bool,
    pub(super) places_against_float_exclusions: bool,
}

impl<S: LayoutScalar> InFlowBoxPreparation<S> {
    pub(super) fn shared_input(&self) -> ComputeInputOf<S> {
        self.child_input
            .with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL)
    }
}

struct InFlowBoxOutput<Node, S: LayoutScalar> {
    output: ComputeOutputOf<S>,
    shared_child: Option<SharedInFlowPlacement<Node, S>>,
    clearance_funding: Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    introduced_cut: Option<entry::canonical::Separation<Node, S>>,
}

type InFlowBoxRequestResult<Node, S, M> =
    ComputeResult<Node, (scope::ChildRequest<Node, S>, InFlowBoxPreparation<S>), S, M>;

pub(super) fn finish_shared_box<'input, Tree, S, M>(
    tree: &mut Tree,
    computation: InFlowBoxComputation<'_, 'input, Tree::Node, S>,
    preparation: InFlowBoxPreparation<S>,
    pending: PendingSharedBox<Tree::Node, S>,
    output: ComputeOutputOf<S>,
    remaining_cut: &mut Option<entry::canonical::Separation<Tree::Node, S>>,
) -> InFlowBoxResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let child = computation.child;
    let floats = tree
        .end_child_float_context(child, preparation.shared_input())
        .ok_or_else(|| float_context::missing(child))?;
    if floats.output != output {
        return Err(float_context::missing(child).into());
    }
    if let Some(previous) = *remaining_cut
        && pending.child_cut.is_some_and(|cut| cut.same_cut(previous))
        && floats.remaining_cut.is_none()
    {
        *remaining_cut = None;
    }
    // Entry qualification retains its funding until the child receipt is consumed.
    drop(pending.entry);
    let output = InFlowBoxOutput {
        output,
        shared_child: Some(SharedInFlowPlacement {
            origin: pending.location,
            floats,
        }),
        clearance_funding: pending.clearance_funding,
        introduced_cut: pending.introduced_cut,
    };
    Ok(finish_in_flow_box(&computation, preparation, output, None))
}

pub(super) fn finish_requested_box<Node: Copy + Eq, S: LayoutScalar>(
    computation: InFlowBoxComputation<'_, '_, Node, S>,
    preparation: InFlowBoxPreparation<S>,
    output: ComputeOutputOf<S>,
    measured: Option<MeasuredAgainstFloats<S>>,
    resident_cut: Option<entry::canonical::Separation<Node, S>>,
) -> InFlowComputedBox<Node, S> {
    let output = InFlowBoxOutput {
        output,
        shared_child: None,
        clearance_funding: None,
        introduced_cut: resident_cut.filter(|cut| {
            cut.container == computation.node
                && cut.node == computation.child
                && cut.index == computation.index
        }),
    };
    finish_in_flow_box(&computation, preparation, output, measured)
}

pub(super) fn prepare_in_flow_box<'input, Tree, S, M>(
    tree: &mut Tree,
    computation: &InFlowBoxComputation<'_, 'input, Tree::Node, S>,
) -> InFlowBoxRequestResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowBoxComputation {
        node,
        child,
        child_style,
        constants,
        prepared,
        baselines,
        float_exclusions,
        input,
        pass_request,
        normal_operation,
        physical_float_context,
        inner_inline,
        ..
    } = *computation;
    let child_indent_context = prepared
        .input
        .block_indent_context()
        .expect("shared child preparation supplies generating context");
    let unresolved_margin = prepared.unresolved_margin;
    let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
    let parent_logical_available = constants.flow_axes.logical_size(pass_request.axes());
    let child_input = prepared
        .input
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
        .with_physical_baseline_demand(
            input.physical_baseline_demand() && child_style.float.is_none(),
        )
        .with_named_baseline_demand(if child_style.float.is_none() {
            baselines.demand_for_following(input.named_baseline_demand())
        } else {
            crate::output::NamedBaselineDemand::None
        });
    let avoids_float_exclusions =
        block_child_avoids_float_exclusions(child_style, constants.flow_axes.writing_mode());
    let inherits_float_exclusions = physical_float_context
        && float_context::shares_parent_context(
            tree.node_input(child),
            constants.flow_axes,
            ParentFormattingContext::BlockFlow,
        )
        && !tree.has_leaf_measurement(child)
        && (!float_exclusions.is_empty() || tree.block_float_work(child));
    let places_against_float_exclusions =
        physical_float_context && avoids_float_exclusions && !float_exclusions.is_empty();
    let child_request = scope::ChildRequest {
        node: child,
        entry: None,
        input: if inherits_float_exclusions || places_against_float_exclusions {
            child_input.with_run_mode(RunMode::ComputeSize)
        } else {
            child_input
        },
    };
    let child_request = if normal_operation
        && input.run_mode().is_perform_layout()
        && tree.block_scope_entry_required(child)
    {
        let pending = scope::prepare(
            tree,
            node,
            child_request,
            child_style,
            constants,
            prepared,
            computation.parent_entry(),
        )?;
        scope::ChildRequest {
            node: child,
            input: pending.input,
            entry: Some(pending),
        }
    } else {
        child_request
    };
    let parent_inline_basis = inner_inline.or(parent_logical_available
        .inline
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get));
    Ok((
        child_request,
        InFlowBoxPreparation {
            child_input,
            child_indent_context,
            parent_logical_unresolved_margin,
            parent_inline_basis,
            avoids_float_exclusions,
            inherits_float_exclusions,
            places_against_float_exclusions,
        },
    ))
}

fn finish_in_flow_box<Node, S: LayoutScalar>(
    computation: &InFlowBoxComputation<'_, '_, Node, S>,
    preparation: InFlowBoxPreparation<S>,
    result: InFlowBoxOutput<Node, S>,
    measured: Option<MeasuredAgainstFloats<S>>,
) -> InFlowComputedBox<Node, S> {
    let InFlowBoxComputation {
        constants,
        prepared,
        content_inline_size,
        ..
    } = *computation;
    let InFlowBoxPreparation {
        child_input,
        parent_logical_unresolved_margin,
        parent_inline_basis,
        avoids_float_exclusions,
        ..
    } = preparation;
    let InFlowBoxOutput {
        output,
        shared_child,
        clearance_funding,
        introduced_cut,
    } = result;
    let (output, child_input, accepted_float_position) = measured
        .map_or((output, child_input, None), |measured| {
            (measured.output, measured.input, measured.position)
        });
    let logical_child_size = constants.flow_axes.logical_size(output.size);
    let mut logical_child_margin = resolve_logical_in_flow_margin(
        parent_logical_unresolved_margin,
        logical_child_size,
        parent_inline_basis,
    );
    if let Some(position) = accepted_float_position {
        let inline_offset = position.inline - constants.logical_content_box_inset().inline_start;
        if matches!(
            parent_logical_unresolved_margin.inline_start,
            ResolvedLengthAutoOf::Auto
        ) {
            logical_child_margin.inline_start = inline_offset;
        }
        if matches!(
            parent_logical_unresolved_margin.inline_end,
            ResolvedLengthAutoOf::Auto
        ) {
            logical_child_margin.inline_end =
                content_inline_size - inline_offset - logical_child_size.inline;
        }
    }
    let child_margin = constants.flow_axes.physical_edges(logical_child_margin);
    InFlowComputedBox {
        output,
        child_input,
        child_margin,
        child_padding: prepared.padding,
        child_border: prepared.border,
        logical_child_margin,
        logical_child_size,
        avoids_float_exclusions,
        accepted_float_position,
        shared_child,
        clearance_funding,
        introduced_cut,
    }
}

fn block_child_avoids_float_exclusions<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    parent_writing_mode: crate::WritingMode,
) -> bool {
    style.display != crate::Display::None
        && !style.display.is_inline_level()
        && style.common.position != Position::Absolute
        && style.float.is_none()
        && flow_root::establishes_flow_root(
            style.display,
            style.float,
            style.block_flow_context,
            &style.common,
            parent_writing_mode,
        )
}

fn parent_inline_size_can_shrink<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    parent_flow_axes: crate::geometry::FlowAxes,
) -> bool {
    !style.common.item_is_replaced
        && match parent_flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => style.common.size.width.is_auto(),
            PhysicalAxis::Vertical => style.common.size.height.is_auto(),
        }
}

fn set_parent_inline_space<S: LayoutScalar>(
    requests: &mut Size<AxisSizingRequestOf<S>>,
    parent_flow_axes: crate::geometry::FlowAxes,
    value: S,
) -> Result<(), crate::LengthResolutionStatus<S>> {
    let space = crate::layout_math::checked_available_space(value)?;
    let request = match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => &mut requests.width,
        PhysicalAxis::Vertical => &mut requests.height,
    };
    *request = request.with_space(space);
    Ok(())
}

pub(super) struct FloatAvoidance<S: LayoutScalar> {
    block: Option<S>,
}

pub(crate) struct FloatBandRequest<S: LayoutScalar> {
    pub(super) input: ComputeInputOf<S>,
    pub(super) trial: Option<crate::engine::FloatChildTrial>,
    next: Option<S>,
}

pub(super) struct MeasuredAgainstFloats<S: LayoutScalar> {
    output: ComputeOutputOf<S>,
    input: ComputeInputOf<S>,
    position: Option<LogicalPointOf<S>>,
}

fn float_child_block_start<Node, S: LayoutScalar>(
    computation: &InFlowBoxComputation<'_, '_, Node, S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
) -> S {
    let InFlowBoxComputation {
        constants,
        adjoining,
        child_style,
        ..
    } = *computation;
    let preview_top_margin = output
        .block_margin_collapse
        .at(constants.flow_axes.block_start())
        .collapse_with_margin(margin.at_physical_side(constants.flow_axes.block_start()));
    let start = adjoining.reduce_start(
        constants.collapse_top_margin,
        child_margin_can_collapse_with_parent(child_style),
        preview_top_margin,
    );
    adjoining.cursor + start.offset
}

impl<S: LayoutScalar> FloatAvoidance<S> {
    pub(super) fn begin<Node: Copy>(
        computation: &InFlowBoxComputation<'_, '_, Node, S>,
        preparation: &InFlowBoxPreparation<S>,
        preview: ComputeOutputOf<S>,
    ) -> Self {
        let InFlowBoxComputation {
            child_style,
            constants,
            float_exclusions,
            ..
        } = *computation;
        let block = parent_inline_size_can_shrink(child_style, constants.flow_axes).then(|| {
            let size = constants.flow_axes.logical_size(preview.size);
            let margin = resolve_logical_in_flow_margin(
                preparation.parent_logical_unresolved_margin,
                size,
                preparation.parent_inline_basis,
            );
            float_exclusions.clearance_block(
                float_child_block_start(
                    computation,
                    preview,
                    constants.flow_axes.physical_edges(margin),
                ),
                child_style.clear,
            )
        });
        Self { block }
    }

    pub(super) fn request<'input, Tree, M>(
        &self,
        tree: &Tree,
        computation: &InFlowBoxComputation<'_, 'input, Tree::Node, S>,
        preparation: &InFlowBoxPreparation<S>,
    ) -> ComputeResult<Tree::Node, FloatBandRequest<S>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let Some(block) = self.block else {
            return Ok(FloatBandRequest {
                input: preparation.child_input,
                trial: None,
                next: None,
            });
        };
        let (available_inline, next) = computation
            .float_exclusions
            .rectangular_bfc_opportunity(block, preparation.parent_logical_unresolved_margin);
        let InFlowBoxComputation {
            child,
            child_style,
            constants,
            prepared,
            pass_request,
            child_run_mode,
            input,
            ..
        } = *computation;
        let child_flow_axes = child_style.common.flow_axes;
        let child_parent_size = prepared.parent_size;
        let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
        let child_logical_available = prepared.logical_available;
        let available_child_inline = prepared.available_inline;
        let child_padding = prepared.padding;
        let child_border = prepared.border;
        let band_child_inline =
            crate::layout_math::remaining_available_extent(available_inline, S::ZERO)
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        let band_available_child_inline =
            if child_flow_axes.inline_axis() == constants.flow_axes.inline_axis() {
                Some(band_child_inline)
            } else {
                available_child_inline
            };
        let band_child_known = in_flow_child_known_size::<Tree, M>(
            tree,
            child,
            child_style,
            child_padding + child_border,
            child_flow_axes,
            child_logical_node_inner_size,
            (child_logical_available.inline.operation() == crate::SizingOperation::Normal)
                .then_some(band_available_child_inline)
                .flatten(),
        )?;
        let mut band_available = child_flow_axes.physical_size(LogicalSizeOf::new(
            in_flow_child_inline_request(
                child_style,
                tree.has_leaf_measurement(child),
                child_flow_axes,
                band_available_child_inline,
                child_logical_available.inline,
            )
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?,
            in_flow_child_block_request(child_logical_available.block),
        ));
        set_parent_inline_space(&mut band_available, constants.flow_axes, band_child_inline)
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        let request = ComputeInputOf::<S>::for_child(
            child_run_mode,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            band_child_known,
            child_parent_size,
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            pass_request.with_axes(band_available),
        )
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
        .with_block_indent_context(preparation.child_indent_context)
        .with_baseline_demand_from(preparation.child_input);
        Ok(FloatBandRequest {
            input: request,
            trial: request
                .run_mode()
                .is_perform_layout()
                .then_some(crate::engine::FloatChildTrial::InFlow),
            next,
        })
    }

    pub(super) fn accept<Node: Copy>(
        &mut self,
        computation: &InFlowBoxComputation<'_, '_, Node, S>,
        preparation: &InFlowBoxPreparation<S>,
        request: FloatBandRequest<S>,
        output: ComputeOutputOf<S>,
    ) -> Option<MeasuredAgainstFloats<S>> {
        let position = if let Some(block) = self.block {
            let constants = computation.constants;
            let size = constants.flow_axes.logical_size(output.size);
            let margin = resolve_logical_in_flow_margin(
                preparation.parent_logical_unresolved_margin,
                size,
                preparation.parent_inline_basis,
            );
            let minimum_block = float_child_block_start(
                computation,
                output,
                constants.flow_axes.physical_edges(margin),
            );
            if minimum_block > block {
                self.block = Some(minimum_block);
                return None;
            }
            let Some(location) = computation.float_exclusions.rectangular_bfc_position(
                block,
                size,
                preparation.parent_logical_unresolved_margin,
                in_flow_child_inline_offset(size.inline, margin, constants),
            ) else {
                self.block = Some(
                    request
                        .next
                        .expect("an intersecting float has a later boundary"),
                );
                return None;
            };
            Some(location)
        } else {
            None
        };
        Some(MeasuredAgainstFloats {
            output,
            input: request.input,
            position,
        })
    }
}

pub(super) struct PendingSharedBox<Node, S: LayoutScalar> {
    entry: entry::EntrySummary<Node, S>,
    location: LogicalPointOf<S>,
    clearance_funding: Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    introduced_cut: Option<entry::canonical::Separation<Node, S>>,
    child_cut: Option<entry::canonical::Separation<Node, S>>,
}

type SharedBoxPreparation<Node, S, M> = LayoutResultOf<
    Node,
    (
        float_context::ChildFrame<Node, S>,
        PendingSharedBox<Node, S>,
    ),
    S,
    M,
>;

type SharedBoxQualificationResult<Node, S, M> =
    LayoutResultOf<Node, (entry::EntryQuery<Node, S>, SharedBoxQualification<Node, S>), S, M>;

pub(super) fn prepare_shared_box<'input, Tree, S, M>(
    tree: &mut Tree,
    computation: &InFlowBoxComputation<'_, 'input, Tree::Node, S>,
    preparation: &InFlowBoxPreparation<S>,
    resident_cut: Option<entry::canonical::Separation<Tree::Node, S>>,
) -> SharedBoxQualificationResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowBoxComputation {
        node,
        child,
        child_style,
        constants,
        prepared,
        float_exclusions,
        ..
    } = *computation;
    let child_input = preparation.child_input;
    let parent_entry = computation.parent_entry();
    let cursor = parent_entry.cursor;
    let active = parent_entry.active;
    let start_open = parent_entry.first;
    // Resolve the complete counterfactual component before committing
    // this child's origin or producing its actual body.
    let pending = scope::prepare(
        tree,
        node,
        scope::ChildRequest {
            node: child,
            input: child_input,
            entry: None,
        },
        child_style,
        constants,
        prepared,
        parent_entry,
    )?;
    let entry_input = child_input
        .with_run_mode(RunMode::ComputeSize)
        .with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL);
    let clearance_funding = match tree.text_computation() {
        Some(state) => {
            state
                .prepare_entry()
                .map_err(|error| inline_run::text_run_error(child, error.into()))?;
            state
                .entry_funding()
                .map_err(|error| inline_run::text_run_error(child, error.into()))?
        }
        None => None,
    };
    let funding = clearance_funding.as_ref();
    let frame = std::sync::Arc::new(
        float_exclusions
            .for_entry_child(LogicalPointOf::new(S::ZERO, S::ZERO), funding)
            .map_err(|error| inline_run::text_run_error(child, error.into()))?,
    );
    let position = entry::canonical::EntryPosition {
        cursor,
        active,
        start_open,
        collapses_start: constants.collapse_top_margin,
        child_adjoins_parent: pending.child_collapses,
        own_start: CollapsibleMarginOf::ZERO.collapse_with_margin(pending.margin.block_start),
        inline_offset: pending.inline_offset,
    };
    let query = entry::EntryQuery {
        node: child,
        input: entry_input,
        reporting_side: constants.flow_axes.block_start(),
        purpose: entry::EntryPurpose::Component,
        incoming: Some(float_context::Incoming {
            node: child,
            request: entry_input,
            frame: frame.clone(),
            cut: resident_cut,
        }),
        position: Some(position),
    };
    Ok((
        query,
        SharedBoxQualification {
            pending,
            frame,
            position,
            clearance_funding,
            resident_cut,
        },
    ))
}

pub(super) struct SharedBoxQualification<Node, S: LayoutScalar> {
    pending: scope::Pending<Node, S>,
    frame: std::sync::Arc<InheritedFloatExclusions<S, Node>>,
    position: entry::canonical::EntryPosition<S>,
    clearance_funding: Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    resident_cut: Option<entry::canonical::Separation<Node, S>>,
}

pub(super) fn complete_shared_preparation<'input, Node: Copy + Eq, S: LayoutScalar, M>(
    computation: &InFlowBoxComputation<'_, 'input, Node, S>,
    qualification: SharedBoxQualification<Node, S>,
    observation: entry::EntryObservation<Node, S>,
) -> SharedBoxPreparation<Node, S, M> {
    let InFlowBoxComputation {
        node,
        child,
        index,
        child_style,
        constants,
        float_exclusions,
        ..
    } = *computation;
    let SharedBoxQualification {
        pending,
        frame,
        position,
        clearance_funding,
        resident_cut,
    } = qualification;
    let funding = clearance_funding.as_ref();
    let summary = match observation {
        entry::EntryObservation::Summary(summary) => summary,
        entry::EntryObservation::Demand(demand) => return Err(float_context::missing(demand.node)),
    };
    let hypothetical = frame
        .common_border(position.hypothetical(summary.start_fact().margin))
        .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
    let bound =
        frame.entry_relevant_bound(child, child_style.clear, constants.flow_axes, funding)?;
    let mut introduced_cut =
        resident_cut.filter(|cut| cut.container == node && cut.node == child && cut.index == index);
    if introduced_cut.is_none()
        && let Some(bound) = bound
        && hypothetical < bound
    {
        introduced_cut = Some(entry::canonical::Separation::own(
            node,
            child,
            index,
            hypothetical,
            bound,
        ));
    }
    let location = if let Some(cut) = introduced_cut {
        let current = bound.ok_or_else(|| float_context::missing(child))?;
        let origin = frame
            .common_border(S::ZERO)
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        LogicalPointOf::new(
            pending.inline_offset,
            current.get().max(cut.hypothetical.get()) - origin.get(),
        )
    } else {
        pending.resolve::<M>(summary.start_fact())?.location
    };
    let child_cut = if introduced_cut.is_some() {
        None
    } else {
        summary.separation.or(resident_cut)
    };
    let exclusions = float_exclusions
        .for_entry_child(location, funding)
        .map_err(|error| inline_run::text_run_error(child, error.into()))?;
    Ok((
        float_context::ChildFrame {
            exclusions,
            cut: child_cut,
        },
        PendingSharedBox {
            entry: summary,
            location,
            clearance_funding,
            introduced_cut,
            child_cut,
        },
    ))
}
