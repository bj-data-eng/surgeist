use super::super::{baseline_frame, inline_run};
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

pub(super) fn compute_in_flow_box<'input, Tree, S, M>(
    tree: &mut Tree,
    computation: InFlowBoxComputation<'_, 'input, Tree::Node, S>,
    remaining_cut: &mut Option<entry::canonical::Separation<Tree::Node, S>>,
) -> InFlowBoxResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowBoxComputation {
        node,
        child,
        index,
        child_style,
        constants,
        prepared,
        adjoining,
        baselines,
        float_exclusions,
        input,
        pass_request,
        child_run_mode,
        normal_operation,
        physical_float_context,
        containing_size,
        inner_inline,
        content_inline_size,
        occupied_block_end,
    } = computation;
    let child_indent_context = prepared
        .input
        .block_indent_context()
        .expect("shared child preparation supplies generating context");
    let unresolved_margin = prepared.unresolved_margin;
    let child_padding = prepared.padding;
    let child_border = prepared.border;
    let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
    let parent_logical_available = constants.flow_axes.logical_size(pass_request.axes());
    let mut child_input = prepared
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
    let mut shared_child = None;
    let mut clearance_funding = None;
    let resident_cut = *remaining_cut;
    let mut introduced_cut =
        resident_cut.filter(|cut| cut.container == node && cut.node == child && cut.index == index);
    let mut accepted_float_position = None;
    let places_against_float_exclusions =
        physical_float_context && avoids_float_exclusions && !float_exclusions.is_empty();
    let parent_entry = || scope::ParentEntry {
        reference: containing_size,
        cursor: adjoining.cursor,
        occupied: occupied_block_end,
        active: adjoining.active,
        top: adjoining.top,
        first: adjoining.start_open,
    };
    let child_request = scope::Ordinary {
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
        && (tree.block_scope_child(child) || tree.block_scope_path(child))
    {
        let pending = scope::prepare(
            tree,
            node,
            child_request,
            child_style,
            constants,
            prepared,
            parent_entry(),
        )?;
        if tree.block_scope_child(child) {
            scope::ChildRequest::Scope(pending)
        } else {
            scope::ChildRequest::Ordinary(scope::Ordinary {
                node: child,
                input: pending.input,
                entry: Some(pending),
            })
        }
    } else {
        scope::ChildRequest::Ordinary(child_request)
    };
    let mut output = if inherits_float_exclusions {
        let computed = compute_shared_in_flow_child(
            tree,
            SharedInFlowChild {
                node,
                child,
                index,
                child_input,
                child_style,
                constants,
                prepared,
                parent_entry: parent_entry(),
                resident_cut,
                float_exclusions,
            },
        )?;
        clearance_funding = computed.clearance_funding;
        introduced_cut = computed.introduced_cut;
        *remaining_cut = computed.remaining_cut;
        shared_child = Some(SharedInFlowPlacement {
            origin: computed.location,
            floats: computed.accepted.floats,
        });
        computed.accepted.output
    } else {
        tree.compute_block_child(child_request)?
    };

    let parent_inline_basis = inner_inline.or(parent_logical_available
        .inline
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get));
    let mut logical_child_size = constants.flow_axes.logical_size(output.size);
    let mut logical_child_margin = resolve_logical_in_flow_margin(
        parent_logical_unresolved_margin,
        logical_child_size,
        parent_inline_basis,
    );
    let mut child_margin = constants.flow_axes.physical_edges(logical_child_margin);
    if places_against_float_exclusions {
        let measured = compute_child_against_floats(
            tree,
            FloatAvoidingChild {
                child,
                child_style,
                constants,
                adjoining,
                prepared,
                pass_request,
                child_run_mode,
                child_indent_context,
                parent_inline_basis,
                containing_scrollbars: input.settled_auto_scrollbars(),
                float_exclusions,
                output,
                child_margin,
                child_input,
            },
        )?;
        output = measured.output;
        child_input = measured.input;
        accepted_float_position = measured.position;
        logical_child_size = constants.flow_axes.logical_size(output.size);
        logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            parent_inline_basis,
        );
        if let Some(position) = accepted_float_position {
            let inline_offset =
                position.inline - constants.logical_content_box_inset().inline_start;
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
        child_margin = constants.flow_axes.physical_edges(logical_child_margin);
    }
    Ok(InFlowComputedBox {
        output,
        child_input,
        child_margin,
        child_padding,
        child_border,
        logical_child_margin,
        logical_child_size,
        avoids_float_exclusions,
        accepted_float_position,
        shared_child,
        clearance_funding,
        introduced_cut,
    })
}

struct FloatAvoidingChild<'input, 'a, Node, S: LayoutScalar> {
    child: Node,
    child_style: &'a BlockChildProjection<'input, S>,
    constants: &'a Constants<S>,
    adjoining: &'a entry::AdjoiningFlow<S>,
    prepared: &'a PreparedChild<S>,
    pass_request: crate::SizingRequestOf<S>,
    child_run_mode: RunMode,
    child_indent_context: crate::text_indent::BlockIndentContext<S>,
    parent_inline_basis: Option<S>,
    containing_scrollbars: crate::scroll::SettledAutoScrollbarState,
    float_exclusions: &'a FloatExclusions<S, Node>,
    output: ComputeOutputOf<S>,
    child_margin: Edges<S>,
    child_input: ComputeInputOf<S>,
}

struct MeasuredAgainstFloats<S: LayoutScalar> {
    output: ComputeOutputOf<S>,
    input: ComputeInputOf<S>,
    position: Option<LogicalPointOf<S>>,
}

fn compute_child_against_floats<'input, Tree, S, M>(
    tree: &mut Tree,
    request: FloatAvoidingChild<'input, '_, Tree::Node, S>,
) -> ComputeResult<Tree::Node, MeasuredAgainstFloats<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let FloatAvoidingChild {
        child,
        child_style,
        constants,
        adjoining,
        prepared,
        pass_request,
        child_run_mode,
        child_indent_context,
        parent_inline_basis,
        containing_scrollbars,
        float_exclusions,
        mut output,
        child_margin,
        mut child_input,
    } = request;
    let parent_logical_unresolved_margin = constants
        .flow_axes
        .logical_edges(prepared.unresolved_margin);
    let child_flow_axes = child_style.common.flow_axes;
    let child_parent_size = prepared.parent_size;
    let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
    let child_logical_available = prepared.logical_available;
    let available_child_inline = prepared.available_inline;
    let child_padding = prepared.padding;
    let child_border = prepared.border;
    let mut accepted_float_position = None;
    let block_start_for = |output: ComputeOutputOf<S>, margin: Edges<S>| {
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
    };
    let preview_cursor_block = block_start_for(output, child_margin);
    let inline_size_can_shrink = parent_inline_size_can_shrink(child_style, constants.flow_axes);
    let band_request = |tree: &Tree,
                        available_inline|
     -> ComputeResult<Tree::Node, ComputeInputOf<S>, S, M> {
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
        Ok(ComputeInputOf::<S>::for_child(
            child_run_mode,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            band_child_known,
            child_parent_size,
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            pass_request.with_axes(band_available),
        )
        .with_containing_auto_scrollbar_pass(containing_scrollbars)
        .with_block_indent_context(child_indent_context)
        .with_baseline_demand_from(child_input))
    };

    if inline_size_can_shrink {
        let mut block = float_exclusions.clearance_block(preview_cursor_block, child_style.clear);
        loop {
            let (available_inline, next) = float_exclusions
                .rectangular_bfc_opportunity(block, parent_logical_unresolved_margin);
            let request = band_request(tree, available_inline)?;
            let packet = if request.run_mode().is_perform_layout() {
                Some(tree.probe_float_child(
                    child,
                    request,
                    crate::engine::FloatChildTrial::InFlow,
                )?)
            } else {
                None
            };
            let measured = match &packet {
                Some(packet) => packet.output,
                None => tree.compute_child(child, request)?,
            };
            let size = constants.flow_axes.logical_size(measured.size);
            let margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                size,
                parent_inline_basis,
            );
            let physical_margin = constants.flow_axes.physical_edges(margin);
            let minimum_block = block_start_for(measured, physical_margin);
            if minimum_block > block {
                block = minimum_block;
                continue;
            }
            let Some(location) = float_exclusions.rectangular_bfc_position(
                block,
                size,
                parent_logical_unresolved_margin,
                in_flow_child_inline_offset(size.inline, margin, constants),
            ) else {
                block = next.expect("an intersecting float has a later boundary");
                continue;
            };
            if let Some(packet) = packet {
                tree.accept_float_child(&packet)?;
            }
            output = measured;
            child_input = request;
            accepted_float_position = Some(location);
            break;
        }
    }

    if accepted_float_position.is_none() {
        output = tree.compute_child(child, child_input)?;
    }
    Ok(MeasuredAgainstFloats {
        output,
        input: child_input,
        position: accepted_float_position,
    })
}

struct SharedInFlowChild<'input, 'a, Node, S: LayoutScalar> {
    node: Node,
    child: Node,
    index: usize,
    child_input: ComputeInputOf<S>,
    child_style: &'a BlockChildProjection<'input, S>,
    constants: &'a Constants<S>,
    prepared: &'a PreparedChild<S>,
    parent_entry: scope::ParentEntry<S>,
    resident_cut: Option<entry::canonical::Separation<Node, S>>,
    float_exclusions: &'a FloatExclusions<S, Node>,
}

struct SharedInFlowComputation<Node, S: LayoutScalar> {
    accepted: float_context::ChildOutput<Node, S>,
    location: LogicalPointOf<S>,
    clearance_funding: Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    introduced_cut: Option<entry::canonical::Separation<Node, S>>,
    remaining_cut: Option<entry::canonical::Separation<Node, S>>,
}

type SharedInFlowResult<Node, S, M> = LayoutResultOf<Node, SharedInFlowComputation<Node, S>, S, M>;

fn compute_shared_in_flow_child<'input, Tree, S, M>(
    tree: &mut Tree,
    request: SharedInFlowChild<'input, '_, Tree::Node, S>,
) -> SharedInFlowResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let SharedInFlowChild {
        node,
        child,
        index,
        child_input,
        child_style,
        constants,
        prepared,
        parent_entry,
        resident_cut,
        float_exclusions,
    } = request;
    let cursor = parent_entry.cursor;
    let active = parent_entry.active;
    let start_open = parent_entry.first;
    // Resolve the complete counterfactual component before committing
    // this child's origin or producing its actual body.
    let pending = scope::prepare(
        tree,
        node,
        scope::Ordinary {
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
    let summary = match tree.qualify_entry(entry::EntryQuery {
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
    })? {
        entry::EntryObservation::Summary(summary) => summary,
        entry::EntryObservation::Demand(demand) => {
            return Err(float_context::missing(demand.node));
        }
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
    let accepted = tree.compute_child_with_float_context(
        child,
        child_input.with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL),
        float_exclusions
            .for_entry_child(location, funding)
            .map_err(|error| inline_run::text_run_error(child, error.into()))?,
        child_cut,
    )?;
    let mut remaining_cut = resident_cut;
    if let Some(previous) = remaining_cut
        && child_cut.is_some_and(|cut| cut.same_cut(previous))
        && accepted.floats.remaining_cut.is_none()
    {
        remaining_cut = None;
    }
    Ok(SharedInFlowComputation {
        accepted,
        location,
        clearance_funding,
        introduced_cut,
        remaining_cut,
    })
}
