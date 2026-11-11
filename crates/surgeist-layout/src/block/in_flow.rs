use super::absolute::{StaticPositionGeometry, absolute_static_position};
use super::inline_run::{
    InlineRunContext, InlineRunTransitionState, VisibleInlineRunTransition, inline_run_end,
    visible_inline_boundary_in_flow, visible_line_break_in_flow,
};
use super::scroll::{BlockScrollContributions, retained_child_scroll_geometry};
use super::sizing::{maximum_size, minimum_size, preferred_size, resolve_length_or_zero};
use super::*;
use crate::{AxisSizingRequestOf, Display};

/// Raw baseline or float work needed even by an otherwise definite size query.
/// This does not establish named absence or discharge a named-fact demand.
pub(super) fn normal_flow_children_need_physical_sizing_work<'input, Tree, M>(
    tree: &Tree,
    children: &[<Tree as Traverse>::Node],
) -> bool
where
    Tree: Compute<'input, M>,
{
    children.iter().copied().any(|child| {
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let style = match projection.kind() {
            InlineParticipantKindOf::InlineText(_) => return true,
            InlineParticipantKindOf::TextControl(_) => return true,
            InlineParticipantKindOf::Box(style) => style,
            InlineParticipantKindOf::LineBreak(input) => {
                return !input.display().is_none()
                    && input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
            InlineParticipantKindOf::InlineBoundary(input) => {
                return input.metrics().line_extent() > Tree::Scalar::ZERO;
            }
        };
        if style.display == crate::Display::None || style.common.position == Position::Absolute {
            return false;
        }
        !style.float.is_none()
            || style.display.is_inline_level()
            || style.display.inner_display() == crate::Display::Block
            || tree.child_count(child) > 0 && style.display.inner_display() == crate::Display::Flex
    })
}

pub(super) struct InFlowResult<Node, S: LayoutScalar> {
    pub(super) fragment_margins: Option<super::fragment_margins::Flow>,
    pub(super) float_context: FloatExclusions<S, Node>,
    pub(super) inherited_floats: usize,
    pub(super) remaining_cut: Option<entry::canonical::Separation<Node, S>>,
    pub(super) groups: Vec<crate::group_alignment::Member<Node, S>>,
    pub(super) formatted: Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    pub(super) intrinsic_inline_demand: S,
    pub(super) owned_float_block_end: S,
    pub(super) resolved_terminal_float_block_end: Option<S>,
    pub(super) contributions: BlockScrollContributions<S>,
    pub(super) baselines: super::baseline_frame::PendingBaselines,
    pub(super) static_positions: Vec<(Node, StaticPositionOf<S>)>,
    pub(super) pending_floats: Vec<PendingFloat<Node, S>>,
    pub(super) cursor_block: S,
    pub(super) occupied_block_end: S,
    pub(super) participated: bool,
    pub(super) top_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin_at_start: bool,
    pub(super) active_margin_can_collapse_with_parent: bool,
    pub(super) all_in_flow_children_can_collapse_through: bool,
}

impl<Node, S: LayoutScalar> InFlowResult<Node, S> {
    pub(super) fn top_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if constants.collapse_top_margin {
            self.top_margin
        } else {
            constants.own_top_margin
        }
    }

    pub(super) fn bottom_margin(&self, constants: &Constants<S>) -> CollapsibleMarginOf<S> {
        if self.collapses_end_margin(constants) {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    pub(super) fn collapses_through(&self, constants: &Constants<S>) -> bool {
        constants.can_collapse_through && self.all_in_flow_children_can_collapse_through
    }

    pub(super) fn collapses_end_margin(&self, constants: &Constants<S>) -> bool {
        constants.collapse_bottom_margin
            && self.active_margin_can_collapse_with_parent
            && (!self.active_margin_at_start || self.collapses_through(constants))
    }

    pub(super) fn content_end(&self, constants: &Constants<S>, role: float_context::Role) -> S {
        let bottom_margin_offset =
            if self.active_margin_at_start || self.collapses_end_margin(constants) {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let in_flow_block_end = self.cursor_block + bottom_margin_offset;
        let used_line_end = self
            .resolved_terminal_float_block_end
            .unwrap_or(in_flow_block_end);
        if role == float_context::Role::Owner {
            in_flow_block_end
                .max(used_line_end)
                .max(self.owned_float_block_end)
        } else {
            in_flow_block_end.max(used_line_end)
        }
    }

    pub(super) fn auto_block(&self, constants: &Constants<S>, role: float_context::Role) -> S {
        let inset = constants.logical_content_box_inset();
        (self.content_end(constants, role) + inset.block_end).max(inset.block_sum())
    }
}

pub(super) struct InFlowPassContext<'a, S: LayoutScalar, Node> {
    pub(super) inline_operation: crate::SizingOperation,
    pub(super) inner_inline: Option<S>,
    pub(super) set_layout: bool,
    pub(super) inherited: Option<&'a InheritedFloatExclusions<S, Node>>,
    pub(super) projection: Option<content::Projection<S>>,
}

enum InFlowChildStart<'a, S: LayoutScalar> {
    VisibleInlineRun(VisibleInlineRunTransition),
    FlowBox(BlockChildProjection<'a, S>),
}

/// Amount and border have distinct floating-point associations. The common
/// BFC bound owns the border; signed clearance owns adjoining progression.
#[derive(Clone, Copy)]
struct ClearanceSettlement<S: LayoutScalar> {
    amount: S,
    border: S,
}

pub(super) fn layout_in_flow_children<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    pass: InFlowPassContext<'_, S, <Tree as Traverse>::Node>,
) -> ComputeResult<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowPassContext {
        inline_operation,
        inner_inline,
        set_layout,
        inherited,
        projection,
    } = pass;
    let logical_node_inner_size =
        LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
    let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
    let cursor = projection.map_or(
        constants.logical_content_box_inset().block_start,
        |reference| reference.content_origin,
    );
    let mut adjoining = entry::AdjoiningFlow::new(cursor, constants.collapse_top_margin);
    let mut remaining_cut = tree
        .block_float_state()
        .and_then(|state| state.entry_cut(node));
    let mut occupied_block_end = S::ZERO;
    let mut intrinsic_inline_demand = S::ZERO;
    let mut owned_float_block_end = adjoining.cursor;
    let mut resolved_terminal_float_block_end = None;
    let mut baselines = super::baseline_frame::PendingBaselines::new();
    let mut static_positions = Vec::new();
    let mut fragment_margins = tree
        .fragment_margin_state()
        .map(|_| super::fragment_margins::Flow::default());
    let mut pending_floats = Vec::new();
    let mut requests = constants.sizing_request.axes();
    let inline = constants
        .flow_axes
        .logical_size(requests)
        .inline
        .with_operation(inline_operation);
    match constants.flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => requests.width = inline,
        PhysicalAxis::Vertical => requests.height = inline,
    }
    let pass_request = constants.sizing_request.with_axes(requests);
    let flow_request = crate::layout_math::with_available_extent(
        constants.flow_axes.logical_size(pass_request.axes()).inline,
        inner_inline,
    )
    .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
    let normal_operation = inline_operation == crate::SizingOperation::Normal;
    let child_run_mode = if normal_operation {
        input.run_mode().for_child()
    } else {
        RunMode::ComputeSize
    };
    let mut float_intrinsics = FloatIntrinsics::new(flow_request);
    let finite_content_inline = inner_inline.or(constants
        .flow_axes
        .logical_size(constants.sizing_request.axes())
        .inline
        .space()
        .definite_size()
        .map(crate::NonNegativeFiniteOf::get));
    // A Shared ComputeSize preview has no incoming BFC frame. Its ordinary
    // float rows belong to the subsequent framed computation, not this trial.
    let physical_float_context = normal_operation
        && (input.run_mode().is_perform_layout()
            || float_context::sizes_float_context(
                float_context::Role::for_input(tree.node_input(node), input),
                input,
                finite_content_inline,
                inherited.is_some(),
                tree.block_float_work(node),
            ));
    let content_inline_size = finite_content_inline.unwrap_or(S::ZERO);
    // A finite ordinary-child preview still needs the real float bottom of
    // an owning BFC. Its private ledger determines size without publishing
    // descendants. Intrinsic inline requests retain their separate frontiers.
    let containing_size = projection.map_or_else(
        || constants.containing_size(logical_node_inner_size),
        |reference| {
            let mut size = constants
                .flow_axes
                .logical_size(constants.containing_size(logical_node_inner_size));
            size.block = reference.block_reference;
            constants.flow_axes.physical_size(size)
        },
    );
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        containing_size,
        content_inline_size,
        constants.logical_content_box_inset(),
    );
    let entry_case_funding = tree
        .text_computation()
        .and_then(|text| text.entry_case_funding().cloned());
    float_exclusions.fund_entry(entry_case_funding.as_ref());
    if physical_float_context && let Some(inherited) = inherited {
        float_exclusions
            .inherit_into_child(inherited)
            .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
    }
    let inherited_floats = float_exclusions.len();
    let content_box_size = Size::new(
        inner_inline
            .or(constants.node_inner_size.width)
            .or(constants
                .sizing_request
                .axes()
                .width
                .space()
                .definite_size()
                .map(crate::NonNegativeFiniteOf::get))
            .unwrap_or(S::ZERO),
        constants.node_inner_size.height.unwrap_or(S::ZERO),
    );
    let content_box_origin = Point::new(
        constants.content_box_inset.left,
        constants.content_box_inset.top,
    );
    let contribution_seed = crate::ScrollRectOf::try_new(content_box_origin, content_box_size)
        .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
    let mut contributions = BlockScrollContributions::new(constants.flow_axes, contribution_seed);

    let mut index = 0;
    if let Some(text) = tree.text_computation() {
        text.formatted_begin_flow(input, flow_request, set_layout)
            .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
    }
    if let Some(groups) = tree.group_computation() {
        groups.begin_flow(input, flow_request, set_layout);
    }
    let mut formatted = tree
        .text_computation()
        .and_then(|text| text.formatted_new_flow());
    let mut first_in_flow = true;
    let mut inline_indent_state = None;
    let mut box_frontier = None;
    let mut line_box_context = crate::line_box::LineBoxContext::new(
        constants.line_box_direction_policy,
        constants.direction,
    );
    let mut hyphenation_state =
        crate::hyphenation::LineHyphenationState::new(constants.hyphenate_limit_lines);
    let tail_active = constants.hyphenate_limit_last == crate::HyphenateLimitLast::Always
        && crate::layout_math::normal_available_size(flow_request).is_some();
    let mut tail_ledger = crate::hyphenation::BlockTailLedger::default();
    let (slots, _slot_lease) = match tree.text_computation() {
        Some(state) => state.semantic_slots(node, None)?,
        None => (Vec::new(), None),
    };
    let mut slot_cursor = 0;
    while index < children.len() || slot_cursor < slots.len() {
        let source_index = index;
        let semantic_end = slots
            .get(slot_cursor)
            .map_or(children.len(), |slot| slot.gap);
        let (child_start, native_child) = if let Some(slot) =
            slots.get(slot_cursor).filter(|slot| slot.gap == index)
        {
            slot_cursor += 1;
            (
                InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::semantic(slot)),
                None,
            )
        } else {
            let child = children[index];
            let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
            // Dispatch the complete admitted capture before transparent-leading
            // nodes are handled individually. The run owner retains their normal
            // hidden computation and absolute static-position side effects.
            let captured_start = tree
                .text_computation()
                .is_some_and(|state| state.run_starting(node, &children[index..]).is_some());
            let child_start = if captured_start {
                let run_start = index;
                index = inline_run_end(
                    tree,
                    node,
                    children,
                    constants,
                    index + 1,
                    semantic_end,
                    None,
                )?;
                InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                    run_start, index,
                ))
            } else {
                match projection.kind() {
                    InlineParticipantKindOf::Box(style) => {
                        if style.display == crate::Display::None {
                            if set_layout {
                                tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                                    child,
                                    crate::output::NormalGeometryOf::metadata(
                                        crate::Point::ZERO,
                                        crate::Size::ZERO,
                                    ),
                                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                        source_index,
                                    )),
                                ));
                                tree.compute_child(
                                    child,
                                    ComputeInputOf::<S>::hidden_in_containing_pass(
                                        ContainingLayoutContext::new(
                                            constants.flow_axes,
                                            ParentFormattingContext::BlockFlow,
                                        ),
                                        input.settled_auto_scrollbars(),
                                    ),
                                )?;
                            }
                            index += 1;
                            continue;
                        }
                        if style.common.position == Position::Absolute {
                            let geometry = if style.display.is_inline_level() {
                                StaticPositionGeometry::Point
                            } else {
                                StaticPositionGeometry::BlockStart
                            };
                            static_positions.push((
                                child,
                                absolute_static_position(
                                    adjoining.cursor + adjoining.active.resolve(),
                                    constants,
                                    containing_size,
                                    geometry,
                                ),
                            ));
                            index += 1;
                            continue;
                        }
                        if crate::inline::starts_run(tree.layout_input(child)) {
                            let run_start = index;
                            index = inline_run_end(
                                tree,
                                node,
                                children,
                                constants,
                                index + 1,
                                semantic_end,
                                None,
                            )?;
                            InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                                run_start, index,
                            ))
                        } else {
                            InFlowChildStart::FlowBox(style)
                        }
                    }
                    InlineParticipantKindOf::InlineText(_)
                    | InlineParticipantKindOf::TextControl(_) => {
                        let run_start = index;
                        index = inline_run_end(
                            tree,
                            node,
                            children,
                            constants,
                            index + 1,
                            semantic_end,
                            None,
                        )?;
                        InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                            run_start, index,
                        ))
                    }
                    InlineParticipantKindOf::LineBreak(line_break) => {
                        if line_break.display().is_none() {
                            if set_layout {
                                tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                                    child,
                                    crate::output::NormalGeometryOf::metadata(
                                        crate::Point::ZERO,
                                        crate::Size::ZERO,
                                    ),
                                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                        source_index,
                                    )),
                                ));
                            }
                            index += 1;
                            continue;
                        }
                        visible_line_break_in_flow(tree, node, child, constants.flow_axes)?;

                        let run_start = index;
                        index = inline_run_end(
                            tree,
                            node,
                            children,
                            constants,
                            index + 1,
                            semantic_end,
                            None,
                        )?;
                        InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                            run_start, index,
                        ))
                    }
                    InlineParticipantKindOf::InlineBoundary(_) => {
                        visible_inline_boundary_in_flow(tree, node, child, constants.flow_axes)?;

                        let run_start = index;
                        index = inline_run_end(
                            tree,
                            node,
                            children,
                            constants,
                            index + 1,
                            semantic_end,
                            None,
                        )?;
                        InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                            run_start, index,
                        ))
                    }
                }
            };
            (child_start, Some(child))
        };
        let child_style = match child_start {
            InFlowChildStart::VisibleInlineRun(transition) => {
                let run_start = transition.start;
                let run_end = transition.end;
                transition.apply(
                    tree,
                    node,
                    children,
                    InlineRunContext {
                        inline_request: flow_request,
                        child_request: pass_request,
                        box_frontier: box_frontier.clone(),
                        tail: if !tail_active {
                            crate::text_layout::fit::TailContext::Unrestricted
                        } else if slot_cursor == slots.len()
                            && children[run_end..].iter().all(|&node| {
                                match InlineParticipantProjection::lookup::<Tree, M>(tree, node)
                                    .kind()
                                {
                                    InlineParticipantKindOf::Box(style) => {
                                        style.display == crate::Display::None
                                            || style.common.position == Position::Absolute
                                            || !style.float.is_none()
                                    }
                                    _ => false,
                                }
                            })
                        {
                            crate::text_layout::fit::TailContext::CompleteBlock
                        } else {
                            crate::text_layout::fit::TailContext::Unresolved
                        },
                        hyphenation_state,
                        line_box_context,
                        source_index_start: run_start,
                        cursor_block: adjoining.cursor,
                        owned_float_block_end,
                        constants,
                        input,
                        node_inner_size,
                        set_layout,
                        indent_state: inline_indent_state.unwrap_or_else(|| {
                            crate::text_indent::IndentLineState::new(
                                first_in_flow && constants.first_formatted,
                            )
                        }),
                    },
                    InlineRunTransitionState {
                        box_frontier: &mut box_frontier,
                        formatted: &mut formatted,
                        indent_state: &mut inline_indent_state,
                        line_box_context: &mut line_box_context,
                        hyphenation_state: &mut hyphenation_state,
                        tail_ledger: &mut tail_ledger,
                        adjoining: &mut adjoining,
                        fragment_margins: &mut fragment_margins,
                        intrinsic_inline_demand: &mut intrinsic_inline_demand,
                        baselines: &mut baselines,
                        static_positions: &mut static_positions,
                        resolved_terminal_float_block_end: &mut resolved_terminal_float_block_end,
                        float_exclusions: &mut float_exclusions,
                        float_intrinsics: &mut float_intrinsics,
                        pending_floats: &mut pending_floats,
                        owned_float_block_end: &mut owned_float_block_end,
                        contributions: &mut contributions,
                    },
                )?;
                first_in_flow &= inline_indent_state.is_none();
                continue;
            }
            InFlowChildStart::FlowBox(style) => style,
        };

        let child = native_child.expect("only a real native child reaches block-box dispatch");
        if child_style.float.is_none() {
            if tail_active {
                tail_ledger.child_block();
            }
            inline_indent_state = None;
        }

        let prepared = prepare_child(
            tree,
            child,
            &child_style,
            constants,
            ChildSizingContext {
                inner_size: node_inner_size,
                request: pass_request,
                run_mode: child_run_mode,
                axis: RequestedAxis::Both,
                first_in_flow: &mut first_in_flow,
            },
        )?;
        let child_indent_context = prepared
            .input
            .block_indent_context()
            .expect("shared child preparation supplies generating context");
        let unresolved_margin = prepared.unresolved_margin;
        let child_padding = prepared.padding;
        let child_border = prepared.border;
        let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
        let parent_logical_available = constants.flow_axes.logical_size(pass_request.axes());
        let child_flow_axes = child_style.common.flow_axes;
        let child_parent_size = prepared.parent_size;
        let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
        let child_logical_available = prepared.logical_available;
        let available_child_inline = prepared.available_inline;
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
            block_child_avoids_float_exclusions(&child_style, constants.flow_axes.writing_mode());
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
        let resident_cut = remaining_cut;
        let mut introduced_cut = resident_cut
            .filter(|cut| cut.container == node && cut.node == child && cut.index == index);
        let mut accepted_shared_origin = None;
        let mut accepted_float_position = None;
        let places_against_float_exclusions =
            physical_float_context && avoids_float_exclusions && !float_exclusions.is_empty();
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
                &child_style,
                constants,
                &prepared,
                scope::ParentEntry {
                    reference: containing_size,
                    cursor: adjoining.cursor,
                    occupied: occupied_block_end,
                    active: adjoining.active,
                    top: adjoining.top,
                    first: adjoining.start_open,
                },
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
                &child_style,
                constants,
                &prepared,
                scope::ParentEntry {
                    reference: containing_size,
                    cursor: adjoining.cursor,
                    occupied: occupied_block_end,
                    active: adjoining.active,
                    top: adjoining.top,
                    first: adjoining.start_open,
                },
            )?;
            let entry_input = child_input
                .with_run_mode(RunMode::ComputeSize)
                .with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL);
            clearance_funding = match tree.text_computation() {
                Some(state) => {
                    state
                        .prepare_entry()
                        .map_err(|error| super::inline_run::text_run_error(child, error.into()))?;
                    state
                        .entry_funding()
                        .map_err(|error| super::inline_run::text_run_error(child, error.into()))?
                }
                None => None,
            };
            let funding = clearance_funding.as_ref();
            let frame = std::sync::Arc::new(
                float_exclusions
                    .for_entry_child(LogicalPointOf::new(S::ZERO, S::ZERO), funding)
                    .map_err(|error| super::inline_run::text_run_error(child, error.into()))?,
            );
            let position = entry::canonical::EntryPosition {
                cursor: adjoining.cursor,
                active: adjoining.active,
                start_open: adjoining.start_open,
                collapses_start: constants.collapse_top_margin,
                child_adjoins_parent: pending.child_collapses,
                own_start: CollapsibleMarginOf::ZERO
                    .collapse_with_margin(pending.margin.block_start),
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
                    return Err(float_context::missing(demand.node).into());
                }
            };
            let hypothetical = frame
                .common_border(position.hypothetical(summary.start_fact().margin))
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
            let bound = frame.entry_relevant_bound(
                child,
                child_style.clear,
                constants.flow_axes,
                funding,
            )?;
            introduced_cut = resident_cut
                .filter(|cut| cut.container == node && cut.node == child && cut.index == index);
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
                child_input.with_settled_auto_scrollbars(
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                ),
                float_exclusions
                    .for_entry_child(location, funding)
                    .map_err(|error| super::inline_run::text_run_error(child, error.into()))?,
                child_cut,
            )?;
            if let Some(previous) = remaining_cut
                && child_cut.is_some_and(|cut| cut.same_cut(previous))
                && accepted.floats.remaining_cut.is_none()
            {
                remaining_cut = None;
            }
            accepted_shared_origin = Some(location);
            shared_child = Some(accepted.floats);
            accepted.output
        } else {
            tree.compute_block_child(child_request)?
        };

        let mut logical_child_size = constants.flow_axes.logical_size(output.size);
        let mut logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            logical_node_inner_size.inline.or(parent_logical_available
                .inline
                .space()
                .definite_size()
                .map(crate::NonNegativeFiniteOf::get)),
        );
        let mut child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        if places_against_float_exclusions {
            let block_start_for = |output: ComputeOutputOf<S>, margin: Edges<S>| {
                let preview_top_margin = output
                    .block_margin_collapse
                    .at(constants.flow_axes.block_start())
                    .collapse_with_margin(
                        margin.at_physical_side(constants.flow_axes.block_start()),
                    );
                let start = adjoining.reduce_start(
                    constants.collapse_top_margin,
                    child_margin_can_collapse_with_parent(&child_style),
                    preview_top_margin,
                );
                adjoining.cursor + start.offset
            };
            let preview_cursor_block = block_start_for(output, child_margin);
            let inline_size_can_shrink =
                parent_inline_size_can_shrink(&child_style, constants.flow_axes);
            let band_request = |tree: &Tree,
                                available_inline|
             -> ComputeResult<Tree::Node, ComputeInputOf<S>, S, M> {
                let band_child_inline =
                    crate::layout_math::remaining_available_extent(available_inline, S::ZERO)
                        .map_err(|error| {
                            crate::error::sizing_resolution_error(child, error.into())
                        })?;
                let band_available_child_inline =
                    if child_flow_axes.inline_axis() == constants.flow_axes.inline_axis() {
                        Some(band_child_inline)
                    } else {
                        available_child_inline
                    };
                let band_child_known = in_flow_child_known_size::<Tree, M>(
                    tree,
                    child,
                    &child_style,
                    child_padding + child_border,
                    child_flow_axes,
                    child_logical_node_inner_size,
                    (child_logical_available.inline.operation() == crate::SizingOperation::Normal)
                        .then_some(band_available_child_inline)
                        .flatten(),
                )?;
                let mut band_available = child_flow_axes.physical_size(LogicalSizeOf::new(
                    in_flow_child_inline_request(
                        &child_style,
                        tree.has_leaf_measurement(child),
                        child_flow_axes,
                        band_available_child_inline,
                        child_logical_available.inline,
                    )
                    .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?,
                    in_flow_child_block_request(child_logical_available.block),
                ));
                set_parent_inline_space(
                    &mut band_available,
                    constants.flow_axes,
                    band_child_inline,
                )
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
                Ok(ComputeInputOf::<S>::for_child(
                    child_run_mode,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    band_child_known,
                    child_parent_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    pass_request.with_axes(band_available),
                )
                .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
                .with_block_indent_context(child_indent_context)
                .with_baseline_demand_from(child_input))
            };

            if inline_size_can_shrink {
                let mut block =
                    float_exclusions.clearance_block(preview_cursor_block, child_style.clear);
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
                        logical_node_inner_size.inline.or(parent_logical_available
                            .inline
                            .space()
                            .definite_size()
                            .map(crate::NonNegativeFiniteOf::get)),
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
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size.inline.or(parent_logical_available
                    .inline
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get)),
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
        if !child_style.float.is_none() {
            let margin_box_inline = logical_child_size.inline + logical_child_margin.inline_sum();
            float_intrinsics.add(margin_box_inline, child_style.float, child_style.clear);
            intrinsic_inline_demand = intrinsic_inline_demand.max(float_intrinsics.result());
            if !physical_float_context && inline_operation != crate::SizingOperation::MaxContent {
                index += 1;
                continue;
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
            let float_inline_end = logical_location.inline
                + logical_child_size.inline
                + logical_child_margin.inline_end
                - content_box_inset.inline_start;
            let float_block_end =
                logical_location.block + logical_child_size.block + logical_child_margin.block_end;
            intrinsic_inline_demand =
                intrinsic_inline_demand.max(intrinsic_float_demand.unwrap_or(float_inline_end));
            owned_float_block_end = owned_float_block_end.max(float_block_end);
            index += 1;
            continue;
        }
        resolved_terminal_float_block_end = None;
        if let Some(groups) = tree.group_computation() {
            groups.child(child);
        }
        if child_style.display.inner_display() == crate::Display::Block
            && let Some(flow) = &mut formatted
        {
            let anonymous =
                tree.node_input(child).block_generation == crate::BlockGeneration::Anonymous;
            if let Some(item) = tree
                .text_computation()
                .map(|text| text.formatted_child(child, child_input, anonymous))
                .transpose()
                .map_err(|error| super::inline_run::text_run_error(child, error.into()))?
                .flatten()
            {
                flow.push(item)
                    .map_err(|error| super::inline_run::text_run_error(child, error.into()))?;
            }
        }
        let deferred_inset =
            super::relative::needs_basis(*child_style.common.inset, constants.flow_axes);
        let inset_offset = if deferred_inset {
            Point::ZERO
        } else {
            super::relative::resolve(
                tree,
                child,
                *child_style.common.inset,
                super::relative::InsetReference::Measurement(node_inner_size),
                constants.flow_axes,
            )?
        };
        let top_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_start())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_start()));
        let bottom_margin_set = output
            .block_margin_collapse
            .at(constants.flow_axes.block_end())
            .collapse_with_margin(child_margin.at_physical_side(constants.flow_axes.block_end()));
        let child_margin_can_collapse_with_parent =
            child_margin_can_collapse_with_parent(&child_style);
        let collapses_through = output
            .block_margin_collapse
            .can_collapse_through(constants.flow_axes);
        let bound_frame = &float_exclusions;
        if child_style.clear != Clear::None
            && !float_exclusions.is_empty()
            && clearance_funding.is_none()
            && let Some(state) = tree.text_computation()
        {
            state
                .prepare_entry()
                .map_err(|error| super::inline_run::text_run_error(child, error.into()))?;
            clearance_funding = state
                .entry_funding()
                .map_err(|error| super::inline_run::text_run_error(child, error.into()))?;
        }
        if introduced_cut.is_none() && child_style.clear != Clear::None {
            let hypothetical = bound_frame
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
            if let Some(bound) = bound_frame.entry_relevant_bound(
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
            let uncollapsed = bound_frame
                .common_border(base_block + prior_internal + top_margin_set.resolve())
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
            let current = bound_frame
                .entry_relevant_bound(
                    child,
                    child_style.clear,
                    constants.flow_axes,
                    clearance_funding.as_ref(),
                )?
                .ok_or_else(|| float_context::missing(child))?;
            let origin = bound_frame
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
            |error: surgeist_text::Error| super::inline_run::text_run_error(child, error.into());
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
            if let Some(previous) = remaining_cut
                && introduced_cut.is_some_and(|cut| cut.same_cut(previous))
            {
                remaining_cut = None;
            }
            adjoining.begin_clearance(settlement.border);
            if let Some(flow) = &mut fragment_margins {
                flow.started = true;
                flow.active = None;
            }
        }
        if let (Some(flow), Some((top, bottom)), Some(state)) = (
            &mut fragment_margins,
            margin_groups,
            tree.fragment_margin_state(),
        ) && !flow.started
        {
            flow.initial = state.join(flow.initial, top).map_err(margin_error)?;
            if output
                .block_margin_collapse
                .can_collapse_through(constants.flow_axes)
            {
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
            &fragment_margins,
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
                &mut fragment_margins,
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
        let logical_location = LogicalPointOf::new(
            in_flow_child_inline_offset(logical_child_size.inline, logical_child_margin, constants),
            adjoining.cursor,
        );
        if accepted_shared_origin.is_some_and(|origin| origin != logical_location) {
            return Err(float_context::missing(child).into());
        }
        let fallback_location = constants.flow_axes.physical_point(
            logical_location,
            logical_child_size,
            containing_size,
        );
        let flow_location = if let Some(location) = accepted_float_position {
            constants
                .flow_axes
                .physical_point(location, logical_child_size, containing_size)
        } else if avoids_float_exclusions {
            let location = float_exclusions.place_bfc_block(
                adjoining.cursor,
                logical_child_size,
                logical_child_margin,
                child_style.clear,
                logical_location.inline,
            );
            constants
                .flow_axes
                .physical_point(location, logical_child_size, containing_size)
        } else {
            fallback_location
        };
        let location = Point::new(
            flow_location.x + inset_offset.x,
            flow_location.y + inset_offset.y,
        );
        if let Some(shared) = shared_child {
            let flow_origin = logical_location;
            if !float_exclusions
                .import_child(&shared, flow_origin)
                .map_err(|error| super::inline_run::text_run_error(child, error.into()))?
            {
                return Err(float_context::missing(child).into());
            }
            owned_float_block_end = owned_float_block_end.max(float_exclusions.bottom());
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

        let child_block_end = if avoids_float_exclusions {
            constants
                .flow_axes
                .logical_point(flow_location, output.size, containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        occupied_block_end = occupied_block_end.max(child_block_end);
        // Intrinsic demand uses the signed margin box in logical flow.
        // The separate scroll contributions retain the full physical span.
        let margin_inline_end = constants
            .flow_axes
            .logical_point(flow_location, output.size, containing_size)
            .inline
            + logical_child_size.inline
            + logical_child_margin.inline_end
            - constants.logical_content_box_inset().inline_start;
        let child_inline_demand = signed_inline_demand(
            logical_child_size.inline,
            logical_child_margin,
            margin_inline_end,
        );
        intrinsic_inline_demand = intrinsic_inline_demand.max(child_inline_demand);
        // Export normal baseline facts; relative movement belongs to visual
        // placement, not to the baseline used by a parent's normal line.
        baselines
            .resolved_child(
                output,
                child_flow_axes,
                flow_location,
                super::baseline_frame::BaselineFrame::Measured {
                    used_size: containing_size.map(LayoutScalar::to_f64),
                    axes: constants.flow_axes,
                },
            )
            .map_err(|error| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(child),
                    crate::LayoutOperation::ChildLayout,
                    crate::LayoutErrorKindOf::TextNumeric {
                        value: error.value(),
                        reason: error.reason(),
                    },
                )
            })?;
        if collapses_through {
            if let (Some(flow), Some((top, bottom)), Some(state)) = (
                &mut fragment_margins,
                margin_groups,
                tree.fragment_margin_state(),
            ) {
                let joined = state.join(flow.active, top).map_err(margin_error)?;
                flow.active = state.join(joined, bottom).map_err(margin_error)?;
            }
        } else if let (Some(flow), Some((_, bottom))) = (&mut fragment_margins, margin_groups) {
            flow.active = bottom;
        }
        let next_cursor = if collapses_through {
            base_block
                + clearance.map_or(S::ZERO, |settlement| prior_internal + settlement.amount)
                + logical_child_size.block
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
        index += 1;
    }

    if tail_active && tail_ledger.needs_continuation() {
        return Err(super::inline_run::text_run_error(
            node,
            crate::text_layout::fit::FitError::MissingBlockTail,
        )
        .into());
    }
    if let Some(text) = tree.text_computation() {
        text.formatted_record_traversal(&formatted)
            .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
    }
    let groups = tree
        .group_computation()
        .map_or_else(Vec::new, |groups| groups.end_flow());
    Ok(InFlowResult {
        fragment_margins,
        float_context: float_exclusions,
        inherited_floats,
        remaining_cut,
        groups,
        formatted,
        intrinsic_inline_demand,
        owned_float_block_end,
        resolved_terminal_float_block_end,
        contributions,
        baselines,
        static_positions,
        pending_floats,
        cursor_block: adjoining.cursor,
        occupied_block_end,
        participated: !first_in_flow,
        top_margin: adjoining.top,
        active_margin: adjoining.active,
        active_margin_at_start: adjoining.start_open,
        active_margin_can_collapse_with_parent: adjoining.end_adjoins_parent,
        all_in_flow_children_can_collapse_through: adjoining.all_through,
    })
}

pub(super) fn child_margin_can_collapse_with_parent<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
) -> bool {
    style.display == crate::Display::Block && style.common.position == Position::Relative
}

fn block_child_avoids_float_exclusions<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    parent_writing_mode: crate::WritingMode,
) -> bool {
    style.display != crate::Display::None
        && !style.display.is_inline_level()
        && style.common.position != Position::Absolute
        && style.float.is_none()
        && super::flow_root::establishes_flow_root(
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

pub(super) struct PreparedChild<S: LayoutScalar> {
    pub(super) unresolved_margin: Edges<ResolvedLengthAutoOf<S>>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) parent_size: Size<Option<S>>,
    pub(super) available_inline: Option<S>,
    pub(super) logical_available: LogicalSizeOf<AxisSizingRequestOf<S>>,
    pub(super) input: ComputeInputOf<S>,
}

pub(super) struct ChildSizingContext<'flow, S: LayoutScalar> {
    pub(super) inner_size: Size<Option<S>>,
    pub(super) request: crate::SizingRequestOf<S>,
    pub(super) run_mode: RunMode,
    pub(super) axis: RequestedAxis,
    pub(super) first_in_flow: &'flow mut bool,
}

pub(super) fn prepare_child<'input, Tree, M>(
    tree: &Tree,
    child: Tree::Node,
    child_style: &BlockChildProjection<'_, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    context: ChildSizingContext<'_, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, PreparedChild<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let ChildSizingContext {
        inner_size: node_inner_size,
        request: pass_request,
        run_mode,
        axis: requested_axis,
        first_in_flow,
    } = context;
    let indent = crate::text_indent::BlockIndentContext {
        inherited: constants.text_indent,
        first_formatted: *first_in_flow && constants.first_formatted,
    };
    // Hidden and absolute children never enter this flow-box preparation.
    // Floats retain the ordinary parent's first formatted-box opportunity.
    if child_style.float.is_none() {
        *first_in_flow = false;
    }
    let unresolved_margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
        *child_style.common.margin,
        node_inner_size,
        |length, basis| length.resolve_auto_with_status(basis),
    );
    let child_padding = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *child_style.common.padding,
            node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let child_border = constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            *child_style.common.border,
            node_inner_size,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let child_flow_axes = child_style.common.flow_axes;
    let child_parent_size = constants.child_containing_block_size(child_flow_axes);
    let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
    let child_logical_available = child_flow_axes.logical_size(pass_request.axes());
    let child_non_auto_margin = child_flow_axes
        .logical_edges(unresolved_margin)
        .map(resolved_length_auto_fallback_zero);
    let available_child_inline = child_logical_node_inner_size
        .inline
        .or(child_logical_available
            .inline
            .space()
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get))
        .map(|inline| {
            crate::layout_math::remaining_available_extent(
                inline,
                child_non_auto_margin.inline_sum(),
            )
        })
        .transpose()
        .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
    let child_known = in_flow_child_known_size::<Tree, M>(
        tree,
        child,
        child_style,
        child_padding + child_border,
        child_flow_axes,
        child_logical_node_inner_size,
        (child_logical_available.inline.operation() == crate::SizingOperation::Normal)
            .then_some(available_child_inline)
            .flatten(),
    )?;
    let input = ComputeInputOf::<Tree::Scalar>::for_child(
        run_mode,
        SizingMode::InherentSize,
        requested_axis,
        child_known,
        child_parent_size,
        ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
        pass_request.with_axes(
            child_flow_axes.physical_size(LogicalSizeOf::new(
                in_flow_child_inline_request(
                    child_style,
                    tree.has_leaf_measurement(child),
                    child_flow_axes,
                    available_child_inline,
                    child_logical_available.inline,
                )
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?,
                in_flow_child_block_request(child_logical_available.block),
            )),
        ),
    )
    .with_block_indent_context(indent);
    Ok(PreparedChild {
        unresolved_margin,
        padding: child_padding,
        border: child_border,
        parent_size: child_parent_size,
        available_inline: available_child_inline,
        logical_available: child_logical_available,
        input,
    })
}

#[expect(
    clippy::type_complexity,
    reason = "the private child-size helper preserves the session's generic error envelope"
)]
pub(super) fn in_flow_child_known_size<'input, Tree, M>(
    tree: &Tree,
    child: <Tree as Traverse>::Node,
    style: &BlockChildProjection<'_, Tree::Scalar>,
    padding_border: Edges<Tree::Scalar>,
    child_flow_axes: crate::geometry::FlowAxes,
    parent: LogicalSizeOf<Option<Tree::Scalar>>,
    available_inline: Option<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, Size<Option<Tree::Scalar>>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let parent = child_flow_axes.physical_size(parent);
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
    let measured = tree.has_leaf_measurement(child);
    let ratio = if flex_owns_preferred_inline(style.display, inline_size, measured) {
        None
    } else {
        *style.common.aspect_ratio
    };
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = minimum_size(style.common.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?;
    let max_size = maximum_size(style.common.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?;
    let ratio_transferred_in_block =
        normal_block_inline_ratio(&style.common, style.display, style.float)
            .filter(|_| ratio.is_some() && available_inline.is_some() && !measured);
    let (min_size, max_size) = if let Some(ratio) = ratio_transferred_in_block {
        transfer_block_ratio_inline_limits(child_flow_axes, ratio, min_size, max_size)
            .map_err(|status| crate::error::sizing_resolution_error(child, status.into()))?
    } else {
        (min_size.apply_aspect_ratio(ratio), max_size)
    };
    let min_size = super::fragment_minimum::apply(
        tree.fragment_minimum(child),
        min_size.add_optional(box_sizing_adjustment),
        padding_border.sum_axes(),
    );
    let mut max_size = max_size.add_optional(box_sizing_adjustment);
    let aspect_height_limit = if ratio_transferred_in_block.is_some() {
        None
    } else {
        ratio
            .zip(max_size.height)
            .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()))
    };
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = preferred_size(style.common.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_max_before_min_optional(min_size, max_size);

    let mut known = child_flow_axes.logical_size(known);
    let min_size = child_flow_axes.logical_size(min_size);
    let max_size = child_flow_axes.logical_size(max_size);
    if !style.common.item_is_table
        && !style.common.item_is_replaced
        && known.inline.is_none()
        && !inline_size.is_min_content()
        && !inline_size.is_max_content()
    {
        known.inline = available_inline
            .map(|inline| inline.clamp_max_before_min_optional(min_size.inline, max_size.inline));
        if aspect_height_limit.is_some() {
            let physical_known = child_flow_axes.physical_size(known);
            known = child_flow_axes.logical_size(
                physical_known
                    .apply_aspect_ratio(ratio)
                    .clamp_max_before_min_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

pub(super) fn in_flow_child_block_request<S: LayoutScalar>(
    inherited: AxisSizingRequestOf<S>,
) -> AxisSizingRequestOf<S> {
    if inherited.operation() == crate::SizingOperation::Normal {
        inherited.with_space(crate::AvailableSpaceOf::Indefinite)
    } else {
        inherited
    }
}

fn flex_owns_preferred_inline<S: LayoutScalar>(
    display: Display,
    preferred: &crate::PreferredSizeOf<S>,
    measured: bool,
) -> bool {
    display == Display::Flex && !measured && preferred.is_max_content()
}

pub(super) fn in_flow_child_inline_request<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    measured: bool,
    child_flow_axes: crate::geometry::FlowAxes,
    available_inline: Option<S>,
    inherited: AxisSizingRequestOf<S>,
) -> Result<AxisSizingRequestOf<S>, crate::LengthResolutionStatus<S>> {
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
    let request = if inline_size.is_min_content() {
        inherited.with_operation(crate::SizingOperation::MinContent)
    } else if inline_size.is_max_content()
        && !flex_owns_preferred_inline(style.display, inline_size, measured)
    {
        inherited.with_operation(crate::SizingOperation::MaxContent)
    } else {
        inherited
    };
    crate::layout_math::with_available_extent(request, available_inline)
}

pub(super) fn relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: crate::geometry::FlowAxes,
) -> Point<S> {
    let logical_inset = flow_axes.logical_edges(inset);
    let logical_offset = LogicalPointOf::new(
        logical_inset
            .inline_start
            .or_else(|| logical_inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        logical_inset
            .block_start
            .or_else(|| logical_inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    );
    flow_axes.physical_point(
        logical_offset,
        LogicalSizeOf::new(S::ZERO, S::ZERO),
        Size::ZERO,
    )
}

pub(crate) fn resolve_logical_in_flow_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_size: LogicalSizeOf<S>,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    resolve_logical_inline_margin(margin, child_size.inline, container_inline)
}

pub(super) fn resolve_logical_inline_margin<S: LayoutScalar>(
    margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    child_inline: S,
    container_inline: Option<S>,
) -> LogicalEdgesOf<S> {
    let non_auto_inline = resolved_length_auto_fallback_zero(margin.inline_start)
        + resolved_length_auto_fallback_zero(margin.inline_end);
    let auto_count = usize::from(matches!(margin.inline_start, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.inline_end, ResolvedLengthAutoOf::Auto));
    let auto_inline = if auto_count == 0 {
        S::ZERO
    } else {
        container_inline
            .map(|inline| (inline - child_inline - non_auto_inline).max(S::ZERO))
            .unwrap_or(S::ZERO)
            / S::from_usize(auto_count)
    };

    LogicalEdgesOf::new(
        resolved_length_auto_or(margin.inline_start, auto_inline),
        resolved_length_auto_or(margin.inline_end, auto_inline),
        resolved_length_auto_fallback_zero(margin.block_start),
        resolved_length_auto_fallback_zero(margin.block_end),
    )
}

fn resolved_length_auto_or<S: LayoutScalar>(value: ResolvedLengthAutoOf<S>, auto_fallback: S) -> S {
    match value {
        ResolvedLengthAutoOf::Auto => auto_fallback,
        ResolvedLengthAutoOf::Resolved(value) => value,
        // Missing-basis symbolic margins keep the algorithm's historical
        // unresolved-as-zero fallback and do not participate in auto distribution.
        ResolvedLengthAutoOf::Unresolved(
            UnresolvedLengthReason::Basis | UnresolvedLengthReason::InvalidNumeric,
        ) => S::ZERO,
    }
}

pub(super) fn resolved_length_auto_fallback_zero<S: LayoutScalar>(
    value: ResolvedLengthAutoOf<S>,
) -> S {
    resolved_length_auto_or(value, S::ZERO)
}

pub(super) fn in_flow_child_inline_offset<S: LayoutScalar>(
    child_inline: S,
    margin: LogicalEdgesOf<S>,
    constants: &Constants<S>,
) -> S {
    let logical_content_box_inset = constants.logical_content_box_inset();
    let logical_inner_size = constants.logical_node_inner_size();
    let mut inline = logical_content_box_inset.inline_start + margin.inline_start;

    let container_inner_inline = logical_inner_size
        .inline
        .or_else(|| {
            constants
                .logical_node_outer_size()
                .inline
                .map(|inline| inline - logical_content_box_inset.inline_sum())
        })
        .unwrap_or(child_inline + margin.inline_sum());
    let item_outer_inline = child_inline + margin.inline_sum();
    if item_outer_inline < container_inner_inline {
        let free_space = container_inner_inline - item_outer_inline;
        match constants.text_align {
            TextAlign::Start
            | TextAlign::End
            | TextAlign::Left
            | TextAlign::Right
            | TextAlign::Center
            | TextAlign::Justify => {}
            TextAlign::LegacyCenter => inline = inline + free_space / S::from_f64(2.0),
            TextAlign::LegacyLeft
                if constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyRight
                if !constants
                    .flow_axes
                    .logical_axis_progression(crate::LogicalAxis::Inline)
                    .is_decreasing() =>
            {
                inline = inline + free_space;
            }
            TextAlign::LegacyLeft | TextAlign::LegacyRight => {}
        }
    }

    inline
}

pub(super) fn signed_inline_demand<S: LayoutScalar>(
    inline: S,
    margin: LogicalEdgesOf<S>,
    margin_inline_end: S,
) -> S {
    (inline + margin.inline_sum()).max(margin_inline_end)
}
