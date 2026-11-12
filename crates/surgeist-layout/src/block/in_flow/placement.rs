use super::super::{fragment_margins, inline_run};
use super::*;

/// Amount and border have distinct floating-point associations. The common
/// BFC bound owns the border; signed clearance owns adjoining progression.
#[derive(Clone, Copy)]
struct ClearanceSettlement<S: LayoutScalar> {
    amount: S,
    border: S,
}

pub(super) struct InFlowChildMargins<'a, 'input, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) child: Node,
    pub(super) index: usize,
    pub(super) child_style: &'a BlockChildProjection<'input, S>,
    pub(super) output: &'a ComputeOutputOf<S>,
    pub(super) child_margin: Edges<S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) float_exclusions: &'a FloatExclusions<S, Node>,
    pub(super) clearance_funding:
        &'a mut Option<crate::text_layout::scope_alignment::EntryFunding<S>>,
    pub(super) introduced_cut: Option<entry::canonical::Separation<Node, S>>,
}

pub(super) struct InFlowMarginPlacement<S: LayoutScalar> {
    base_block: S,
    prior_internal: S,
    clearance: Option<ClearanceSettlement<S>>,
    margin_groups: Option<(Option<usize>, Option<usize>)>,
    top_margin_set: CollapsibleMarginOf<S>,
    bottom_margin_set: CollapsibleMarginOf<S>,
    collapses_through: bool,
    child_margin_can_collapse_with_parent: bool,
}

pub(super) fn begin_in_flow_child_margins<'input, Tree, S, M>(
    tree: &mut Tree,
    placement: InFlowChildMargins<'_, 'input, Tree::Node, S>,
    adjoining: &mut entry::AdjoiningFlow<S>,
    remaining_cut: &mut Option<entry::canonical::Separation<Tree::Node, S>>,
    fragment_margins: &mut Option<fragment_margins::Flow>,
) -> LayoutResultOf<Tree::Node, InFlowMarginPlacement<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowChildMargins {
        node,
        child,
        index,
        child_style,
        output,
        child_margin,
        constants,
        float_exclusions,
        clearance_funding,
        mut introduced_cut,
    } = placement;
    let top_margin_set = output
        .block_margin_collapse
        .at(constants.flow_axes.block_start())
        .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_start()));
    let bottom_margin_set = output
        .block_margin_collapse
        .at(constants.flow_axes.block_end())
        .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_end()));
    let child_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent(child_style);
    let collapses_through = output
        .block_margin_collapse
        .can_collapse_through(constants.flow_axes);
    if child_style.clear != Clear::None
        && !float_exclusions.is_empty()
        && clearance_funding.is_none()
        && let Some(state) = tree.text_computation()
    {
        state
            .prepare_entry()
            .map_err(|error| inline_run::text_run_error(child, error.into()))?;
        *clearance_funding = state
            .entry_funding()
            .map_err(|error| inline_run::text_run_error(child, error.into()))?;
    }
    if introduced_cut.is_none() && child_style.clear != Clear::None {
        let hypothetical = float_exclusions
            .common_border(
                adjoining.cursor
                    + adjoining
                        .reduce_start(
                            constants.collapse_top_margin,
                            child_margin_can_collapse_with_parent,
                            top_margin_set,
                        )
                        .offset,
            )
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        if let Some(bound) = float_exclusions.entry_relevant_bound(
            child,
            child_style.clear,
            constants.flow_axes,
            clearance_funding.as_ref(),
        )? && hypothetical < bound
        {
            introduced_cut = Some(entry::canonical::Separation::own(
                node,
                child,
                index,
                hypothetical,
                bound,
            ));
        }
    }
    let base_block = adjoining.cursor;
    let prior_internal = if adjoining.start_open {
        S::ZERO
    } else {
        adjoining.active.resolve()
    };
    let clearance = if let Some(cut) = introduced_cut {
        let uncollapsed = float_exclusions
            .common_border(base_block + prior_internal + top_margin_set.resolve())
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        let current = float_exclusions
            .entry_relevant_bound(
                child,
                child_style.clear,
                constants.flow_axes,
                clearance_funding.as_ref(),
            )?
            .ok_or_else(|| float_context::missing(child))?;
        let origin = float_exclusions
            .common_border(S::ZERO)
            .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
        Some(ClearanceSettlement {
            amount: (current.get() - uncollapsed.get())
                .max(cut.hypothetical.get() - uncollapsed.get()),
            border: current.get().max(cut.hypothetical.get()) - origin.get(),
        })
    } else {
        None
    };
    let margin_error =
        |error: surgeist_text::Error| inline_run::text_run_error(child, error.into());
    let margin_groups = tree
        .fragment_margin_state()
        .map(|state| {
            Ok::<_, surgeist_text::Error>((
                state.child(child, constants.flow_axes.block_start())?,
                state.child(child, constants.flow_axes.block_end())?,
            ))
        })
        .transpose()
        .map_err(margin_error)?;
    if let Some(settlement) = clearance {
        if let Some(previous) = *remaining_cut
            && introduced_cut.is_some_and(|cut| cut.same_cut(previous))
        {
            *remaining_cut = None;
        }
        adjoining.begin_clearance(settlement.border);
        if let Some(flow) = fragment_margins.as_mut() {
            flow.started = true;
            flow.active = None;
        }
    }
    if let (Some(flow), Some((top, bottom)), Some(state)) = (
        fragment_margins.as_mut(),
        margin_groups,
        tree.fragment_margin_state(),
    ) && !flow.started
    {
        flow.initial = state.join(flow.initial, top).map_err(margin_error)?;
        if collapses_through {
            flow.initial = state.join(flow.initial, bottom).map_err(margin_error)?;
        } else {
            flow.started = true;
        }
    }
    let start = adjoining.reduce_start(
        constants.collapse_top_margin,
        child_margin_can_collapse_with_parent,
        top_margin_set,
    );
    if let (Some(flow), Some((top, _)), Some(state)) = (
        fragment_margins.as_ref(),
        margin_groups,
        tree.fragment_margin_state(),
    ) {
        let group = if start.exposes_child {
            flow.active
        } else {
            state.join(flow.active, top).map_err(margin_error)?
        };
        state
            .record_entry(child, constants.flow_axes, group)
            .map_err(margin_error)?;
    }
    if adjoining.start_open
        && start.exposes_child
        && let (Some(flow), Some((top, bottom)), Some(state)) = (
            fragment_margins.as_mut(),
            margin_groups,
            tree.fragment_margin_state(),
        )
    {
        flow.top = state.join(flow.top, top).map_err(margin_error)?;
        if collapses_through {
            flow.top = state.join(flow.top, bottom).map_err(margin_error)?;
        }
    }
    if clearance.is_none() {
        adjoining.begin_child(&start, top_margin_set, bottom_margin_set, collapses_through);
    }
    Ok(InFlowMarginPlacement {
        base_block,
        prior_internal,
        clearance,
        margin_groups,
        top_margin_set,
        bottom_margin_set,
        collapses_through,
        child_margin_can_collapse_with_parent,
    })
}

impl<S: LayoutScalar> InFlowMarginPlacement<S> {
    pub(super) fn finish<'input, Tree, M>(
        self,
        tree: &mut Tree,
        child: Tree::Node,
        adjoining: &mut entry::AdjoiningFlow<S>,
        fragment_margins: &mut Option<fragment_margins::Flow>,
        child_block_size: S,
        child_block_end: S,
    ) -> LayoutResultOf<Tree::Node, (), S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
    {
        let Self {
            base_block,
            prior_internal,
            clearance,
            margin_groups,
            top_margin_set,
            bottom_margin_set,
            collapses_through,
            child_margin_can_collapse_with_parent,
        } = self;
        let margin_error =
            |error: surgeist_text::Error| inline_run::text_run_error(child, error.into());
        if collapses_through {
            if let (Some(flow), Some((top, bottom)), Some(state)) = (
                fragment_margins.as_mut(),
                margin_groups,
                tree.fragment_margin_state(),
            ) {
                let joined = state.join(flow.active, top).map_err(margin_error)?;
                flow.active = state.join(joined, bottom).map_err(margin_error)?;
            }
        } else if let (Some(flow), Some((_, bottom))) = (fragment_margins.as_mut(), margin_groups) {
            flow.active = bottom;
        }
        let next_cursor = if collapses_through {
            base_block
                + clearance.map_or(S::ZERO, |settlement| prior_internal + settlement.amount)
                + child_block_size
        } else {
            child_block_end
        };
        adjoining.finish_child(
            next_cursor,
            top_margin_set,
            bottom_margin_set,
            collapses_through,
            child_margin_can_collapse_with_parent,
        );
        Ok(())
    }
}

pub(super) struct SharedInFlowPlacement<Node, S: LayoutScalar> {
    pub(super) origin: LogicalPointOf<S>,
    pub(super) floats: float_context::Completed<Node, S>,
}

pub(super) struct InFlowBoxPlacement<'a, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) child: Node,
    pub(super) source_index: usize,
    pub(super) constants: &'a Constants<S>,
    pub(super) cursor: S,
    pub(super) containing_size: Size<S>,
    pub(super) child_size: LogicalSizeOf<S>,
    pub(super) logical_margin: LogicalEdgesOf<S>,
    pub(super) child_margin: Edges<S>,
    pub(super) child_border: Edges<S>,
    pub(super) child_padding: Edges<S>,
    pub(super) output: &'a ComputeOutputOf<S>,
    pub(super) clear: Clear,
    pub(super) avoids_float_exclusions: bool,
    pub(super) accepted_float_position: Option<LogicalPointOf<S>>,
    pub(super) shared_child: Option<SharedInFlowPlacement<Node, S>>,
    pub(super) inset_offset: Point<S>,
    pub(super) deferred_inset: bool,
    pub(super) set_layout: bool,
}

pub(super) struct InFlowBoxOrigins<S: LayoutScalar> {
    pub(super) logical: LogicalPointOf<S>,
    pub(super) normal: Point<S>,
}

pub(super) fn place_in_flow_box<'input, Tree, S, M>(
    tree: &mut Tree,
    placement: InFlowBoxPlacement<'_, Tree::Node, S>,
    float_exclusions: &mut FloatExclusions<S, Tree::Node>,
    owned_float_block_end: &mut S,
    contributions: &mut BlockScrollContributions<S>,
) -> LayoutResultOf<Tree::Node, InFlowBoxOrigins<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowBoxPlacement {
        node,
        child,
        source_index,
        constants,
        cursor,
        containing_size,
        child_size,
        logical_margin,
        child_margin,
        child_border,
        child_padding,
        output,
        clear,
        avoids_float_exclusions,
        accepted_float_position,
        shared_child,
        inset_offset,
        deferred_inset,
        set_layout,
    } = placement;
    let logical_location = LogicalPointOf::new(
        in_flow_child_inline_offset(child_size.inline, logical_margin, constants),
        cursor,
    );
    if shared_child
        .as_ref()
        .is_some_and(|shared| shared.origin != logical_location)
    {
        return Err(float_context::missing(child));
    }
    let fallback_location =
        constants
            .flow_axes
            .physical_point(logical_location, child_size, containing_size);
    let flow_location = if let Some(location) = accepted_float_position {
        constants
            .flow_axes
            .physical_point(location, child_size, containing_size)
    } else if avoids_float_exclusions {
        let location = float_exclusions.place_bfc_block(
            cursor,
            child_size,
            logical_margin,
            clear,
            logical_location.inline,
        );
        constants
            .flow_axes
            .physical_point(location, child_size, containing_size)
    } else {
        fallback_location
    };
    let location = Point::new(
        flow_location.x + inset_offset.x,
        flow_location.y + inset_offset.y,
    );
    if let Some(shared) = shared_child {
        if !float_exclusions
            .import_child(&shared.floats, logical_location)
            .map_err(|error| inline_run::text_run_error(child, error.into()))?
        {
            return Err(float_context::missing(child));
        }
        *owned_float_block_end = owned_float_block_end.max(float_exclusions.bottom());
    }
    if set_layout {
        let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
            tree,
            child,
            |box_projection, target_projection| {
                retained_child_scroll_geometry(
                    box_projection,
                    target_projection,
                    output.size,
                    output.content_size,
                    child_padding,
                    child_border,
                    output.scroll_geometry,
                )
            },
        )
        .map_err(|error| layout_child_geometry_error(node, child, error))?;
        if !deferred_inset {
            contributions
                .include_in_flow_geometry(location, child_margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(node, child, error))?;
        }
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            child,
            crate::output::NormalGeometryOf::box_geometry(
                flow_location,
                output.size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index),
                location,
                size: output.size,
                named_baselines: output.named_baselines,
                content_size: output.content_size,
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        ));
    }

    Ok(InFlowBoxOrigins {
        logical: logical_location,
        normal: flow_location,
    })
}

pub(super) struct InFlowFloatItem<'input, 'a, Node, S: LayoutScalar> {
    pub(super) child: Node,
    pub(super) source_index: usize,
    pub(super) child_style: &'a BlockChildProjection<'input, S>,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) child_border: Edges<S>,
    pub(super) child_padding: Edges<S>,
    pub(super) child_margin: Edges<S>,
    pub(super) logical_child_size: LogicalSizeOf<S>,
    pub(super) logical_child_margin: LogicalEdgesOf<S>,
}

pub(super) struct InFlowFloatPass<'a, S: LayoutScalar> {
    pub(super) constants: &'a Constants<S>,
    pub(super) adjoining: &'a entry::AdjoiningFlow<S>,
    pub(super) inline_operation: crate::SizingOperation,
    pub(super) physical_float_context: bool,
    pub(super) set_layout: bool,
}

pub(super) struct InFlowFloatState<'a, Node, S: LayoutScalar> {
    pub(super) float_exclusions: &'a mut FloatExclusions<S, Node>,
    pub(super) float_intrinsics: &'a mut FloatIntrinsics<S>,
    pub(super) pending_floats: &'a mut Vec<PendingFloat<Node, S>>,
    pub(super) intrinsic_inline_demand: &'a mut S,
    pub(super) owned_float_block_end: &'a mut S,
}

pub(super) fn place_in_flow_float<Node, S, M>(
    node: Node,
    item: InFlowFloatItem<'_, '_, Node, S>,
    pass: InFlowFloatPass<'_, S>,
    state: InFlowFloatState<'_, Node, S>,
) -> LayoutResultOf<Node, (), S, M>
where
    Node: Copy + Eq,
    S: LayoutScalar,
{
    let InFlowFloatItem {
        child,
        source_index,
        child_style,
        output,
        child_border,
        child_padding,
        child_margin,
        logical_child_size,
        logical_child_margin,
    } = item;
    let InFlowFloatPass {
        constants,
        adjoining,
        inline_operation,
        physical_float_context,
        set_layout,
    } = pass;
    let InFlowFloatState {
        float_exclusions,
        float_intrinsics,
        pending_floats,
        intrinsic_inline_demand,
        owned_float_block_end,
    } = state;
    let margin_box_inline = logical_child_size.inline + logical_child_margin.inline_sum();
    float_intrinsics.add(margin_box_inline, child_style.float, child_style.clear);
    *intrinsic_inline_demand = intrinsic_inline_demand.max(float_intrinsics.result());
    if !physical_float_context && inline_operation != crate::SizingOperation::MaxContent {
        return Ok(());
    }
    // Project the otherwise-empty anonymous parent's border without
    // consuming the component. Exported start groups stay outside;
    // internal groups precede its own zero top margin, not a later
    // in-flow sibling's margin.
    let anonymous_start = adjoining.reduce_start(
        constants.collapse_top_margin,
        true,
        CollapsibleMarginOf::ZERO,
    );
    let mut pending_float = PendingFloat {
        placement: super::floats::PendingPlacement::Unresolved,
        node: child,
        source_index,
        side: child_style.float,
        clear: child_style.clear,
        block_start: adjoining.cursor + anonymous_start.offset,
        size: output.size,
        named_baselines: output.named_baselines,
        content_size: output.content_size,
        border: child_border,
        padding: child_padding,
        margin: child_margin,
        float_exclusion: child_style.float_exclusion,
        child_compute_geometry: output.scroll_geometry,
    };
    // Intrinsic Max needs the actual rectangular cohort, without
    // querying Shape or pretending an indefinite line has zero width.
    let placement = if inline_operation == crate::SizingOperation::MaxContent {
        float_exclusions.place_float_intrinsic(node, &pending_float)?
    } else {
        float_exclusions.place_float_for_line(node, &pending_float, None)?
    };
    let logical_location = placement.logical;
    let intrinsic_float_demand = placement.intrinsic_inline_demand;
    if set_layout {
        float_exclusions.retain_native(&mut pending_float, placement);
        pending_floats.push(pending_float);
    }
    let content_box_inset = constants.logical_content_box_inset();
    let float_inline_end =
        logical_location.inline + logical_child_size.inline + logical_child_margin.inline_end
            - content_box_inset.inline_start;
    let float_block_end =
        logical_location.block + logical_child_size.block + logical_child_margin.block_end;
    *intrinsic_inline_demand =
        intrinsic_inline_demand.max(intrinsic_float_demand.unwrap_or(float_inline_end));
    *owned_float_block_end = owned_float_block_end.max(float_block_end);
    Ok(())
}
