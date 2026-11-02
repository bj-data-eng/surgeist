use super::absolute::absolute_static_position;
use super::inline_run::{
    InlineRunContext, InlineRunTransitionState, VisibleInlineRunTransition, inline_run_end,
    visible_inline_boundary_in_flow, visible_line_break_in_flow,
};
use super::scroll::{BlockScrollContributions, retained_child_scroll_geometry};
use super::sizing::{
    maximum_size, minimum_size, preferred_size, resolve_auto_optional, resolve_length_or_zero,
};
use super::*;

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
    pub(super) float_context: FloatExclusions<S, Node>,
    pub(super) inherited_floats: usize,
    pub(super) groups: Vec<crate::group_alignment::Member<Node, S>>,
    pub(super) formatted: Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    pub(super) intrinsic_inline_demand: S,
    pub(super) owned_float_block_end: S,
    pub(super) resolved_terminal_float_block_end: Option<S>,
    pub(super) contributions: BlockScrollContributions<S>,
    pub(super) baselines: super::baseline_frame::PendingBaselines,
    pub(super) static_positions: Vec<(Node, Point<S>)>,
    pub(super) pending_floats: Vec<PendingFloat<Node, S>>,
    pub(super) cursor_block: S,
    pub(super) top_margin: CollapsibleMarginOf<S>,
    pub(super) active_margin: CollapsibleMarginOf<S>,
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
        if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
            self.active_margin
        } else {
            constants.own_bottom_margin
        }
    }

    pub(super) fn auto_block(&self, constants: &Constants<S>, role: float_context::Role) -> S {
        let bottom_margin_offset =
            if constants.collapse_bottom_margin && self.active_margin_can_collapse_with_parent {
                S::ZERO
            } else {
                self.active_margin.resolve()
            };
        let content_box_inset = constants.logical_content_box_inset();
        let in_flow_block_end = self.cursor_block + bottom_margin_offset;
        let used_line_end = self
            .resolved_terminal_float_block_end
            .unwrap_or(in_flow_block_end);
        let content_end = if role == float_context::Role::Owner {
            in_flow_block_end
                .max(used_line_end)
                .max(self.owned_float_block_end)
        } else {
            in_flow_block_end.max(used_line_end)
        };
        (content_end + content_box_inset.block_end).max(content_box_inset.block_sum())
    }
}

pub(super) struct InFlowPassContext<'a, S: LayoutScalar, Node> {
    pub(super) inner_inline: Option<S>,
    pub(super) set_layout: bool,
    pub(super) inherited: Option<&'a InheritedFloatExclusions<S, Node>>,
}

enum InFlowChildStart<'a, S: LayoutScalar> {
    VisibleInlineRun(VisibleInlineRunTransition),
    FlowBox(BlockChildProjection<'a, S>),
}

pub(super) fn layout_in_flow_children<'input, Tree, S, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<S>,
    input: ComputeInputOf<S>,
    pass: InFlowPassContext<'_, S, <Tree as Traverse>::Node>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InFlowResult<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InFlowPassContext {
        inner_inline,
        set_layout,
        inherited,
    } = pass;
    let logical_node_inner_size =
        LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
    let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
    let mut cursor_block = constants.logical_content_box_inset().block_start;
    let mut intrinsic_inline_demand = S::ZERO;
    let mut owned_float_block_end = constants.logical_content_box_inset().block_start;
    let mut resolved_terminal_float_block_end = None;
    let mut baselines = super::baseline_frame::PendingBaselines::new();
    let mut static_positions = Vec::new();
    let mut active_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut top_margin = CollapsibleMarginOf::<S>::ZERO;
    let mut is_collapsing_first_margin = constants.collapse_top_margin;
    let mut all_in_flow_children_can_collapse_through = true;
    let mut active_margin_can_collapse_with_parent = constants.collapse_top_margin;
    let mut pending_floats = Vec::new();
    let mut float_intrinsics = FloatIntrinsics::new(
        inner_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(constants.available_content.width),
    );
    let finite_content_inline = inner_inline.or(constants
        .flow_axes
        .logical_size(constants.available_content)
        .inline
        .into_option());
    let content_inline_size = finite_content_inline.unwrap_or(S::ZERO);
    // A finite ordinary-child preview still needs the real float bottom of
    // an owning BFC. Its private ledger determines size without publishing
    // descendants. Intrinsic inline requests retain their separate frontiers.
    let sizes_float_context = float_context::sizes_float_context(
        float_context::Role::for_input(tree.node_input(node), input),
        input,
        finite_content_inline,
        inherited.is_some(),
        tree.block_float_work(node),
    );
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        containing_size,
        content_inline_size,
        constants.logical_content_box_inset(),
    );
    if (set_layout || sizes_float_context)
        && let Some(inherited) = inherited
    {
        float_exclusions.inherit_into_child(inherited, node);
    }
    let inherited_floats = float_exclusions.len();
    let content_box_size = Size::new(
        inner_inline
            .or(constants.node_inner_size.width)
            .or(constants.available_content.width.into_option())
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
        text.formatted_begin_flow(
            input,
            inner_inline.map(AvailableOf::definite).unwrap_or(
                constants
                    .flow_axes
                    .logical_size(constants.available_content)
                    .inline,
            ),
            set_layout,
        );
    }
    if let Some(groups) = tree.group_computation() {
        groups.begin_flow(
            input,
            inner_inline.map(AvailableOf::definite).unwrap_or(
                constants
                    .flow_axes
                    .logical_size(constants.available_content)
                    .inline,
            ),
            set_layout,
        );
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
        && matches!(
            inner_inline.map(AvailableOf::definite).unwrap_or(
                constants
                    .flow_axes
                    .logical_size(constants.available_content)
                    .inline
            ),
            AvailableOf::Definite(_)
        );
    let mut tail_ledger = crate::hyphenation::BlockTailLedger::default();
    let slots = match tree.text_computation() {
        Some(state) => state.semantic_slots(node)?,
        None => Vec::new(),
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
                index = inline_run_end(tree, node, children, constants, index + 1, semantic_end)?;
                InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                    run_start, index,
                ))
            } else {
                match projection.kind() {
                    InlineParticipantKindOf::Box(style) => {
                        if style.display == crate::Display::None {
                            if set_layout {
                                tree.set_unrounded(
                                    child,
                                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                        source_index,
                                    )),
                                );
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
                            static_positions.push((
                                child,
                                absolute_static_position(
                                    cursor_block + active_margin.resolve(),
                                    constants,
                                    constants.containing_size(logical_node_inner_size),
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
                        )?;
                        InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::new(
                            run_start, index,
                        ))
                    }
                    InlineParticipantKindOf::LineBreak(line_break) => {
                        if line_break.display().is_none() {
                            if set_layout {
                                tree.set_unrounded(
                                    child,
                                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(
                                        source_index,
                                    )),
                                );
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
                        cursor_block,
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
                        cursor_block: &mut cursor_block,
                        intrinsic_inline_demand: &mut intrinsic_inline_demand,
                        baselines: &mut baselines,
                        static_positions: &mut static_positions,
                        resolved_terminal_float_block_end: &mut resolved_terminal_float_block_end,
                        active_margin: &mut active_margin,
                        is_collapsing_first_margin: &mut is_collapsing_first_margin,
                        active_margin_can_collapse_with_parent:
                            &mut active_margin_can_collapse_with_parent,
                        all_in_flow_children_can_collapse_through:
                            &mut all_in_flow_children_can_collapse_through,
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
        let child_indent_context = crate::text_indent::BlockIndentContext {
            inherited: constants.text_indent,
            first_formatted: first_in_flow && constants.first_formatted,
        };
        if child_style.float.is_none() {
            if tail_active {
                tail_ledger.child_block();
            }
            first_in_flow = false;
            inline_indent_state = None;
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
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_border = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *child_style.common.border,
                node_inner_size,
                |length, basis| resolve_length_or_zero(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let parent_logical_unresolved_margin = constants.flow_axes.logical_edges(unresolved_margin);
        let parent_logical_available = constants
            .flow_axes
            .logical_size(constants.available_content);
        let child_flow_axes = child_style.common.flow_axes;
        let child_parent_size = constants.child_containing_block_size(child_flow_axes);
        let child_logical_node_inner_size = child_flow_axes.logical_size(child_parent_size);
        let child_logical_available = child_flow_axes.logical_size(constants.available_content);
        let child_non_auto_margin = child_flow_axes
            .logical_edges(unresolved_margin)
            .map(resolved_length_auto_fallback_zero);
        let available_child_inline = child_logical_node_inner_size
            .inline
            .or(child_logical_available.inline.into_option())
            .map(|inline| (inline - child_non_auto_margin.inline_sum()).max(S::ZERO));
        let child_known = in_flow_child_known_size::<Tree, M>(
            tree,
            child,
            &child_style,
            child_padding + child_border,
            child_flow_axes,
            child_logical_node_inner_size,
            available_child_inline,
        )?;
        let mut child_input = ComputeInputOf::<S>::for_child(
            input.run_mode().for_child(),
            SizingMode::InherentSize,
            RequestedAxis::Both,
            child_known,
            child_parent_size,
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            child_flow_axes.physical_size(LogicalSizeOf::new(
                in_flow_child_available_inline(
                    &child_style,
                    child_flow_axes,
                    available_child_inline,
                    child_logical_available.inline,
                ),
                AvailableOf::<S>::MAX_CONTENT,
            )),
        )
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
        .with_block_indent_context(child_indent_context)
        .with_named_baseline_demand(if child_style.float.is_none() {
            baselines.demand_for_following(input.named_baseline_demand())
        } else {
            crate::output::NamedBaselineDemand::None
        });
        let avoids_float_exclusions = block_child_avoids_float_exclusions(&child_style);
        let inherits_float_exclusions = (input.run_mode().is_perform_layout()
            || sizes_float_context)
            && float_context::shares_parent_context(
                tree.node_input(child),
                constants.flow_axes,
                ParentFormattingContext::BlockFlow,
            )
            && !tree.has_leaf_measurement(child)
            && (!float_exclusions.is_empty() || tree.block_float_work(child));
        let mut shared_child = None;
        let places_against_float_exclusions = (set_layout || sizes_float_context)
            && avoids_float_exclusions
            && !float_exclusions.is_empty();
        let mut output = tree.compute_child(
            child,
            if inherits_float_exclusions || places_against_float_exclusions {
                child_input.with_run_mode(RunMode::ComputeSize)
            } else {
                child_input
            },
        )?;

        let mut logical_child_size = constants.flow_axes.logical_size(output.size);
        let mut logical_child_margin = resolve_logical_in_flow_margin(
            parent_logical_unresolved_margin,
            logical_child_size,
            logical_node_inner_size
                .inline
                .or(parent_logical_available.inline.into_option()),
        );
        let mut child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        if inherits_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let child_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let child_logical_location = if child_style.clear == Clear::None {
                child_logical_location
            } else {
                LogicalPointOf::new(
                    child_logical_location.inline,
                    float_exclusions.clearance_block(preview_cursor_block, child_style.clear),
                )
            };
            let relative = relative_inset_offset(
                child_style
                    .common
                    .inset
                    .zip_size(node_inner_size, |length, basis| {
                        resolve_auto_optional(length, basis)
                    })
                    .transpose_with_node(tree, child)?,
                constants.flow_axes,
            );
            let inherited = float_exclusions
                .for_ordinary_child(child_logical_location, relative != Point::ZERO);
            debug_assert_eq!(child_style.display.inner_display(), crate::Display::Block);
            let accepted = tree.compute_child_with_float_context(
                child,
                child_input.with_settled_auto_scrollbars(
                    crate::scroll::SettledAutoScrollbarState::INITIAL,
                ),
                inherited,
            )?;
            output = accepted.output;
            shared_child = Some(accepted.floats);
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        } else if places_against_float_exclusions {
            let preview_top_margin = output
                .block_margin_collapse
                .at(constants.flow_axes.block_start())
                .collapse_with_margin(
                    child_margin.at_physical_side(constants.flow_axes.block_start()),
                );
            let child_margin_can_collapse_with_parent =
                child_margin_can_collapse_with_parent(&child_style);
            let collapsed_margin = if is_collapsing_first_margin {
                if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                    active_margin.resolve()
                } else {
                    active_margin.collapse_with(preview_top_margin).resolve()
                }
            } else {
                active_margin.collapse_with(preview_top_margin).resolve()
            };
            let preview_cursor_block = cursor_block + collapsed_margin;
            let containing_size = constants.containing_size(logical_node_inner_size);
            let preview_logical_location = LogicalPointOf::new(
                in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
                preview_cursor_block,
            );
            let preview_fallback = constants.flow_axes.physical_point(
                preview_logical_location,
                logical_child_size,
                containing_size,
            );
            let inline_size_can_shrink =
                parent_inline_size_can_shrink(&child_style, constants.flow_axes);
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: true,
                },
                BfcBandCandidate {
                    block_start: preview_cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: preview_fallback,
                    inline_size_can_shrink,
                },
            )?;

            if inline_size_can_shrink {
                let parent_non_auto_margin =
                    parent_logical_unresolved_margin.map(resolved_length_auto_fallback_zero);
                let band_child_inline =
                    (placement.available_inline - parent_non_auto_margin.inline_sum()).max(S::ZERO);
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
                    band_available_child_inline,
                )?;
                let mut band_available = child_flow_axes.physical_size(LogicalSizeOf::new(
                    in_flow_child_available_inline(
                        &child_style,
                        child_flow_axes,
                        band_available_child_inline,
                        child_logical_available.inline,
                    ),
                    AvailableOf::<S>::MAX_CONTENT,
                ));
                set_parent_inline_available(
                    &mut band_available,
                    constants.flow_axes,
                    band_child_inline,
                );
                child_input = ComputeInputOf::<S>::for_child(
                    input.run_mode().for_child(),
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    band_child_known,
                    child_parent_size,
                    ContainingLayoutContext::new(
                        constants.flow_axes,
                        ParentFormattingContext::BlockFlow,
                    ),
                    band_available,
                )
                .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
                .with_block_indent_context(child_indent_context)
                .with_named_baseline_demand(child_input.named_baseline_demand());
            }

            output = tree.compute_child(child, child_input)?;
            logical_child_size = constants.flow_axes.logical_size(output.size);
            logical_child_margin = resolve_logical_in_flow_margin(
                parent_logical_unresolved_margin,
                logical_child_size,
                logical_node_inner_size
                    .inline
                    .or(parent_logical_available.inline.into_option()),
            );
            child_margin = constants.flow_axes.physical_edges(logical_child_margin);
        }
        if !child_style.float.is_none() {
            let margin_box_inline = logical_child_size.inline + logical_child_margin.inline_sum();
            float_intrinsics.add(margin_box_inline, child_style.float, child_style.clear);
            intrinsic_inline_demand = intrinsic_inline_demand.max(float_intrinsics.result());
            if !input.run_mode().is_perform_layout() && !sizes_float_context {
                index += 1;
                continue;
            }
            let mut pending_float = PendingFloat {
                placement: super::floats::PendingPlacement::Unresolved,
                node: child,
                source_index,
                side: child_style.float,
                clear: child_style.clear,
                block_start: cursor_block,
                size: output.size,
                named_baselines: output.named_baselines,
                content_size: output.content_size,
                border: child_border,
                padding: child_padding,
                margin: child_margin,
                float_exclusion: child_style.float_exclusion,
                child_compute_geometry: output.scroll_geometry,
            };
            let placement = float_exclusions.place_float_for_line(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout || sizes_float_context,
                },
                &pending_float,
                None,
            )?;
            let logical_location = placement.logical;
            float_exclusions.retain_native(&mut pending_float, placement);
            if set_layout {
                pending_floats.push(pending_float);
            }
            let content_box_inset = constants.logical_content_box_inset();
            let float_inline_end = logical_location.inline
                + logical_child_size.inline
                + logical_child_margin.inline_end
                - content_box_inset.inline_start;
            let float_block_end =
                logical_location.block + logical_child_size.block + logical_child_margin.block_end;
            intrinsic_inline_demand = intrinsic_inline_demand.max(float_inline_end);
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
                .and_then(|text| text.formatted_child(child, child_input, anonymous))
            {
                flow.items.push(item);
            }
        }
        let inset_offset = relative_inset_offset(
            child_style
                .common
                .inset
                .zip_size(node_inner_size, |length, basis| {
                    resolve_auto_optional(length, basis)
                })
                .transpose_with_node(tree, child)?,
            constants.flow_axes,
        );
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
        let base_block = cursor_block;
        let collapsed_margin = if is_collapsing_first_margin {
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                top_margin = top_margin.collapse_with(top_margin_set);
            }
            is_collapsing_first_margin = false;
            if constants.collapse_top_margin && child_margin_can_collapse_with_parent {
                active_margin.resolve()
            } else {
                active_margin.collapse_with(top_margin_set).resolve()
            }
        } else {
            active_margin.collapse_with(top_margin_set).resolve()
        };
        cursor_block = cursor_block + collapsed_margin;
        let logical_location = LogicalPointOf::new(
            in_flow_child_inline_offset(logical_child_size, logical_child_margin, constants),
            cursor_block,
        );
        let containing_size = constants.containing_size(logical_node_inner_size);
        let fallback_location = constants.flow_axes.physical_point(
            logical_location,
            logical_child_size,
            containing_size,
        );
        let flow_location = if avoids_float_exclusions {
            let placement = float_exclusions.place_bfc_block(
                ProviderBandContext {
                    tree,
                    container: node,
                    enabled: set_layout || sizes_float_context,
                },
                BfcBandCandidate {
                    block_start: cursor_block,
                    size: output.size,
                    margin: child_margin,
                    clear: child_style.clear,
                    fallback: fallback_location,
                    // This is the actual computed child, not a width proposal.
                    // A narrower band would require another child computation;
                    // retain its used width and advance to a fitting transition.
                    inline_size_can_shrink: false,
                },
            )?;
            placement.location
        } else if child_style.clear != Clear::None {
            let cleared_logical_location = LogicalPointOf::new(
                logical_location.inline,
                float_exclusions.clearance_block(cursor_block, child_style.clear),
            );
            constants.flow_axes.physical_point(
                cleared_logical_location,
                logical_child_size,
                containing_size,
            )
        } else {
            fallback_location
        };
        let location = Point::new(
            flow_location.x + inset_offset.x,
            flow_location.y + inset_offset.y,
        );
        if let Some(shared) = shared_child {
            let flow_origin = LogicalPointOf::new(
                logical_location.inline,
                if child_style.clear == Clear::None {
                    logical_location.block
                } else {
                    float_exclusions.clearance_block(cursor_block, child_style.clear)
                },
            );
            if !float_exclusions.import_child(&shared, flow_origin) {
                return Err(float_context::missing(child));
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
            contributions
                .include_in_flow_geometry(location, child_margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(node, child, error))?;
            tree.set_unrounded(
                child,
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
            );
        }

        let child_block_end = if avoids_float_exclusions || child_style.clear != Clear::None {
            constants
                .flow_axes
                .logical_point(flow_location, output.size, containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        // Normal intrinsic demand uses only the accepted border geometry and
        // margins. Performed child content may include recursive visual
        // overflow, which belongs exclusively to the scroll contributions.
        let border_inline_start = match constants.flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => flow_location.x - constants.content_box_inset.left,
            PhysicalAxis::Vertical => flow_location.y - constants.content_box_inset.top,
        };
        let border_inline_span = (border_inline_start + logical_child_size.inline).max(S::ZERO)
            - border_inline_start.min(S::ZERO);
        let child_inline_demand = (logical_child_margin.inline_sum() + logical_child_size.inline)
            .max(border_inline_span + logical_child_margin.inline_end);
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
        if output
            .block_margin_collapse
            .can_collapse_through(constants.flow_axes)
        {
            cursor_block = if child_style.clear == Clear::None {
                base_block + logical_child_size.block
            } else {
                child_block_end
            };
            active_margin = active_margin
                .collapse_with(top_margin_set)
                .collapse_with(bottom_margin_set);
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        } else {
            all_in_flow_children_can_collapse_through = false;
            cursor_block = child_block_end;
            active_margin = bottom_margin_set;
            active_margin_can_collapse_with_parent = child_margin_can_collapse_with_parent;
        }
        index += 1;
    }

    if tail_active && tail_ledger.needs_continuation() {
        return Err(super::inline_run::text_run_error(
            node,
            crate::text_layout::fit::FitError::MissingBlockTail,
        ));
    }
    if let Some(text) = tree.text_computation() {
        text.formatted_record_traversal(&formatted);
    }
    let groups = tree
        .group_computation()
        .map_or_else(Vec::new, |groups| groups.end_flow());
    Ok(InFlowResult {
        float_context: float_exclusions,
        inherited_floats,
        groups,
        formatted,
        intrinsic_inline_demand,
        owned_float_block_end,
        resolved_terminal_float_block_end,
        contributions,
        baselines,
        static_positions,
        pending_floats,
        cursor_block,
        top_margin,
        active_margin,
        active_margin_can_collapse_with_parent,
        all_in_flow_children_can_collapse_through,
    })
}

fn child_margin_can_collapse_with_parent<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
) -> bool {
    style.display == crate::Display::Block && style.common.position == Position::Relative
}

fn block_child_avoids_float_exclusions<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
) -> bool {
    style.display != crate::Display::None
        && !style.display.is_inline_level()
        && style.common.position != Position::Absolute
        && style.float.is_none()
        && (style.block_flow_context == crate::BlockFlowContext::Independent
            || style.common.item_is_replaced
            || matches!(
                style.display,
                crate::Display::Flex | crate::Display::Grid | crate::Display::GridLanes
            )
            || style
                .common
                .overflow
                .establishes_independent_formatting_context())
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

fn set_parent_inline_available<S: LayoutScalar>(
    available: &mut Size<AvailableOf<S>>,
    parent_flow_axes: crate::geometry::FlowAxes,
    value: S,
) {
    match parent_flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => available.width = AvailableOf::definite(value),
        PhysicalAxis::Vertical => available.height = AvailableOf::definite(value),
    }
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
    let box_sizing_adjustment = if style.common.box_sizing == BoxSizing::ContentBox {
        padding_border.sum_axes()
    } else {
        Size::ZERO
    };
    let min_size = minimum_size(style.common.min_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment);
    let mut max_size = maximum_size(style.common.max_size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .add_optional(box_sizing_adjustment);
    let aspect_height_limit = style
        .common
        .aspect_ratio
        .zip(max_size.height)
        .and_then(|(ratio, height)| max_size.width.is_none().then_some(height * ratio.get()));
    if let Some(width) = aspect_height_limit {
        max_size.width = Some(width);
    }
    let known = preferred_size(style.common.size, parent, SizingAlgorithm::Block, true)
        .transpose_with_node(tree, child)?
        .apply_aspect_ratio(*style.common.aspect_ratio)
        .add_optional(box_sizing_adjustment)
        .clamp_max_before_min_optional(min_size, max_size);

    let mut known = child_flow_axes.logical_size(known);
    let min_size = child_flow_axes.logical_size(min_size);
    let max_size = child_flow_axes.logical_size(max_size);
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
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
                    .apply_aspect_ratio(*style.common.aspect_ratio)
                    .clamp_max_before_min_optional(
                        child_flow_axes.physical_size(min_size),
                        child_flow_axes.physical_size(max_size),
                    ),
            );
        }
    }

    Ok(child_flow_axes.physical_size(known))
}

pub(super) fn in_flow_child_available_inline<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
    child_flow_axes: crate::geometry::FlowAxes,
    available_inline: Option<S>,
    fallback: AvailableOf<S>,
) -> AvailableOf<S> {
    let inline_size = match child_flow_axes.inline_axis() {
        crate::PhysicalAxis::Horizontal => &style.common.size.width,
        crate::PhysicalAxis::Vertical => &style.common.size.height,
    };
    if inline_size.is_min_content() {
        AvailableOf::<S>::MIN_CONTENT
    } else if inline_size.is_max_content() {
        AvailableOf::<S>::MAX_CONTENT
    } else {
        available_inline
            .map(AvailableOf::<S>::definite)
            .unwrap_or(fallback)
    }
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
    let non_auto_inline = resolved_length_auto_fallback_zero(margin.inline_start)
        + resolved_length_auto_fallback_zero(margin.inline_end);
    let auto_count = usize::from(matches!(margin.inline_start, ResolvedLengthAutoOf::Auto))
        + usize::from(matches!(margin.inline_end, ResolvedLengthAutoOf::Auto));
    let auto_inline = if auto_count == 0 {
        S::ZERO
    } else {
        container_inline
            .map(|inline| (inline - child_size.inline - non_auto_inline).max(S::ZERO))
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

fn in_flow_child_inline_offset<S: LayoutScalar>(
    size: LogicalSizeOf<S>,
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
        .unwrap_or(size.inline + margin.inline_sum());
    let item_outer_inline = size.inline + margin.inline_sum();
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
