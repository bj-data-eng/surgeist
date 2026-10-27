use super::absolute::absolute_static_position;
use super::in_flow::{content_size_contribution, relative_inset_offset};
use super::scroll::{child_scrollbar_size, retained_child_scroll_geometry};
use super::sizing::{max_content_size, resolve_auto_optional, resolve_length_or_zero};
use super::*;
use crate::FlowAxes;

type TextFragmentGroup<Node, S> = (
    Node,
    Vec<InlineFragmentOutputOf<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
);

struct RetainedAtomicChild<'input, Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
    projection: InlineParticipantProjection<'input, S>,
    request: ComputeInputOf<S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
}

pub(super) fn inline_run_end<'input, Tree, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    children: &[<Tree as Traverse>::Node],
    constants: &Constants<<Tree as Traverse>::Scalar>,
    mut index: usize,
    semantic_end: usize,
) -> LayoutResultOf<<Tree as Traverse>::Node, usize, <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let captured_end = tree.text_computation().and_then(|state| {
        state
            .paragraph_starting(container, &children[index - 1..])
            .map(|binding| index - 1 + binding.nodes.len())
    });
    while index < captured_end.unwrap_or(children.len()).min(semantic_end) {
        if captured_end.is_none()
            && !crate::inline::continues_run(tree.layout_input(children[index]))
        {
            break;
        }
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, children[index]);
        match projection.kind() {
            InlineParticipantKindOf::Box(_) => {}
            InlineParticipantKindOf::LineBreak(input) => {
                if input.display().is_none() {
                    index += 1;
                    continue;
                }
                visible_line_break_in_flow(tree, container, children[index], constants.flow_axes)?;
            }
            InlineParticipantKindOf::InlineText(_) | InlineParticipantKindOf::TextControl(_) => {}
            InlineParticipantKindOf::InlineBoundary(_) => {
                visible_inline_boundary_in_flow(
                    tree,
                    container,
                    children[index],
                    constants.flow_axes,
                )?;
            }
        }
        index += 1;
    }
    Ok(index)
}

pub(super) fn visible_line_break_in_flow<'input, Tree, S, M>(
    tree: &Tree,
    container: <Tree as Traverse>::Node,
    child: <Tree as Traverse>::Node,
    flow: FlowAxes,
) -> LayoutResultOf<Tree::Node, Option<LineBreakInputOf<S>>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
    let InlineParticipantKindOf::LineBreak(line_break) = projection.kind() else {
        return Ok(None);
    };
    if line_break.display().is_none() {
        return Ok(None);
    }
    validate_inline_control_flow(
        container,
        child,
        flow,
        FlowAxes::new(line_break.writing_mode(), line_break.direction()),
    )?;
    Ok(Some(line_break))
}

pub(super) fn visible_inline_boundary_in_flow<'input, Tree, S, M>(
    tree: &Tree,
    container: <Tree as Traverse>::Node,
    child: <Tree as Traverse>::Node,
    flow: FlowAxes,
) -> LayoutResultOf<Tree::Node, Option<InlineBoundaryInputOf<S>>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
    let InlineParticipantKindOf::InlineBoundary(boundary) = projection.kind() else {
        return Ok(None);
    };
    validate_inline_control_flow(
        container,
        child,
        flow,
        FlowAxes::new(boundary.writing_mode(), boundary.direction()),
    )?;
    Ok(Some(boundary))
}

fn validate_inline_control_flow<Node, S: LayoutScalar, M>(
    container: Node,
    subject: Node,
    expected: FlowAxes,
    actual: FlowAxes,
) -> LayoutResultOf<Node, (), S, M> {
    if actual != expected {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineFlowMismatch {
                expected,
                actual,
            }),
        ));
    }
    Ok(())
}

#[derive(Clone)]
pub(super) struct VisibleInlineRunTransition {
    pub(super) start: usize,
    pub(super) end: usize,
    semantic: Option<surgeist_text::PreparedText>,
}

impl VisibleInlineRunTransition {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            semantic: None,
        }
    }
    pub(super) fn semantic<Node>(
        slot: &crate::text_layout::capture::SemanticParagraphSlot<Node>,
    ) -> Self {
        Self {
            start: slot.gap,
            end: slot.gap,
            semantic: Some(slot.prepared.clone()),
        }
    }

    pub(super) fn apply<'input, Tree, S, M>(
        self,
        tree: &mut Tree,
        node: <Tree as Traverse>::Node,
        children: &[<Tree as Traverse>::Node],
        mut context: InlineRunContext<'_, S>,
        state: InlineRunTransitionState<'_, <Tree as Traverse>::Node, S>,
    ) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
        S: LayoutScalar,
    {
        let InlineRunTransitionState {
            box_frontier,
            formatted,
            indent_state,
            line_box_context,
            hyphenation_state,
            tail_ledger,
            cursor_block,
            content_size,
            scroll_content_size,
            baselines,
            named_baselines,
            static_positions,
            resolved_terminal_float_block_end,
            active_margin,
            is_collapsing_first_margin,
            active_margin_can_collapse_with_parent,
            all_in_flow_children_can_collapse_through,
            float_exclusions,
            float_intrinsics,
            pending_floats,
            owned_float_block_end,
            contributions,
        } = state;
        let adjoining_margin = active_margin.resolve();
        context.cursor_block = *cursor_block;
        let constants = context.constants;
        let semantic = self.semantic.as_ref().map(|prepared| {
            tree.text_computation()
                .and_then(|state| state.semantic_paragraph(node, prepared))
                .expect("semantic transition retains its validated actual binding")
        });
        let mut placement = layout_inline_run_children(
            tree,
            node,
            &children[self.start..self.end],
            InlineRunRequest {
                semantic,
                context,
                adjoining_margin,
            },
            float_exclusions,
            contributions,
        )?;

        if placement.margin == InlineRunMargin::Separates {
            *cursor_block = *cursor_block + adjoining_margin;
            *is_collapsing_first_margin = false;
        }
        if let (Some(flow), Some(item)) = (formatted.as_mut(), placement.formatted.take()) {
            flow.items.push(item);
        }

        if !placement.lines.is_empty() {
            tail_ledger.run(core::mem::take(&mut placement.lines));
        }
        *hyphenation_state = placement.hyphenation_state;
        *line_box_context = placement.line_box_context;
        *box_frontier = placement.box_frontier.take();
        if let Some(committed) = placement.indent_state {
            *indent_state = Some(committed);
        }
        let placement_content_size = constants.flow_axes.logical_size(placement.content_size);
        // An individual float still contributes its actual margin width.
        // Captured Max cohorts use actual Text-line and float-query receipts;
        // the ordinary global sum would combine expired rows.
        for plan in &placement.floats {
            let size = constants.flow_axes.logical_size(plan.size);
            let margin = constants.flow_axes.logical_edges(plan.margin);
            float_intrinsics.add_captured(
                size.inline + margin.inline_sum(),
                plan.placement
                    .text()
                    .and_then(|accepted| accepted.intrinsic_inline_demand),
            );
        }
        content_size.inline = content_size
            .inline
            .max(placement_content_size.inline)
            .max(float_intrinsics.result());
        content_size.block = content_size.block.max(placement_content_size.block);
        let placement_scroll_content_size = constants
            .flow_axes
            .logical_size(placement.scroll_content_size);
        scroll_content_size.inline = scroll_content_size
            .inline
            .max(placement_scroll_content_size.inline);
        scroll_content_size.block = scroll_content_size
            .block
            .max(placement_scroll_content_size.block);
        *owned_float_block_end = (*owned_float_block_end).max(placement.owned_float_block_end);
        record_inline_run_baselines(baselines, &placement, *cursor_block, constants);
        let mut run_baselines = BaselinesOf::NONE;
        record_inline_run_baselines(&mut run_baselines, &placement, *cursor_block, constants);
        let facts = crate::NamedBaselinesOf::from_line(
            placement.inline_baseline_context,
            constants.flow_axes,
            run_baselines,
            placement.first_named_baseline,
            placement.last_named_baseline,
        )
        .map_err(|crate::FiniteScalarErrorOf::NonFinite { value }| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
            )
        })?;
        named_baselines.record(facts);
        *cursor_block = *cursor_block + placement.logical_block_extent(constants.flow_axes);
        pending_floats.extend(placement.floats);
        static_positions.extend(placement.static_positions);
        if placement.margin == InlineRunMargin::Separates {
            *resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
            *active_margin = CollapsibleMarginOf::<S>::ZERO;
            *active_margin_can_collapse_with_parent = false;
            *all_in_flow_children_can_collapse_through = false;
        }
        Ok(())
    }
}

pub(super) struct InlineRunTransitionState<'a, Node, S: LayoutScalar> {
    pub(super) box_frontier: &'a mut Option<crate::text_layout::boxes::BoxFrontier>,
    pub(super) formatted: &'a mut Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    pub(super) line_box_context: &'a mut crate::line_box::LineBoxContext,
    pub(super) indent_state: &'a mut Option<crate::text_indent::IndentLineState>,
    pub(super) hyphenation_state: &'a mut crate::hyphenation::LineHyphenationState,
    pub(super) tail_ledger: &'a mut crate::hyphenation::BlockTailLedger,
    pub(super) cursor_block: &'a mut S,
    pub(super) content_size: &'a mut LogicalSizeOf<S>,
    pub(super) scroll_content_size: &'a mut LogicalSizeOf<S>,
    pub(super) baselines: &'a mut BaselinesOf<S>,
    pub(super) named_baselines: &'a mut crate::NamedBaselinesOf<S>,
    pub(super) static_positions: &'a mut Vec<(Node, Point<S>)>,
    pub(super) resolved_terminal_float_block_end: &'a mut Option<S>,
    pub(super) active_margin: &'a mut CollapsibleMarginOf<S>,
    pub(super) is_collapsing_first_margin: &'a mut bool,
    pub(super) active_margin_can_collapse_with_parent: &'a mut bool,
    pub(super) all_in_flow_children_can_collapse_through: &'a mut bool,
    pub(super) float_exclusions: &'a mut FloatExclusions<S, Node>,
    pub(super) float_intrinsics: &'a mut super::floats::FloatIntrinsics<S>,
    pub(super) pending_floats: &'a mut Vec<PendingFloat<Node, S>>,
    pub(super) owned_float_block_end: &'a mut S,
    pub(super) contributions: &'a mut ScrollContributionAccumulatorOf<S>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InlineRunMargin {
    Adjoins,
    Separates,
}

struct InlineRunPlacement<Node, S: LayoutScalar> {
    inline_baseline_context: crate::InlineBaselineContext,
    first_named_baseline: bool,
    last_named_baseline: bool,
    margin: InlineRunMargin,
    box_frontier: Option<crate::text_layout::boxes::BoxFrontier>,
    formatted: Option<crate::text_layout::formatted::FormattedFlowItem<Node, S>>,
    line_box_context: crate::line_box::LineBoxContext,
    lines: Vec<crate::hyphenation::CommittedInlineLine>,
    floats: Vec<PendingFloat<Node, S>>,
    owned_float_block_end: S,
    indent_state: Option<crate::text_indent::IndentLineState>,
    hyphenation_state: crate::hyphenation::LineHyphenationState,
    size: Size<S>,
    content_size: Size<S>,
    scroll_content_size: Size<S>,
    static_positions: Vec<(Node, Point<S>)>,
    baselines: BaselinesOf<S>,
    first_baseline: Option<S>,
    last_baseline: Option<S>,
    resolved_float_terminal_block_end: Option<S>,
}

impl<Node, S: LayoutScalar> InlineRunPlacement<Node, S> {
    fn logical_block_extent(&self, flow_axes: crate::geometry::FlowAxes) -> S {
        flow_axes.logical_size(self.size).block
    }
}

pub(super) struct InlineRunContext<'a, S: LayoutScalar> {
    pub(super) box_frontier: Option<crate::text_layout::boxes::BoxFrontier>,
    pub(super) line_box_context: crate::line_box::LineBoxContext,
    pub(super) tail: crate::text_layout::fit::TailContext,
    pub(super) indent_state: crate::text_indent::IndentLineState,
    pub(super) hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub(super) source_index_start: usize,
    pub(super) cursor_block: S,
    pub(super) owned_float_block_end: S,
    pub(super) constants: &'a Constants<S>,
    pub(super) input: ComputeInputOf<S>,
    pub(super) node_inner_size: Size<Option<S>>,
    pub(super) set_layout: bool,
}

fn resolved_inline_float_terminal_block_end<S: LayoutScalar>(
    report: &crate::inline::MixedInlineRunReportOf<S>,
    cursor_block: S,
    owned_float_block_end: S,
) -> Option<S> {
    let terminal_line_block_end = cursor_block + report.block_extent;
    report.float_edge_phase.map(|phase| {
        if terminal_line_block_end > owned_float_block_end {
            terminal_line_block_end
        } else {
            (terminal_line_block_end + phase).max(owned_float_block_end)
        }
    })
}

fn forced_line_break_control<S: LayoutScalar>(
    source_index: usize,
    input: LineBreakInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        source_index,
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
        input.clear(),
    )
}

fn inline_boundary_control<S: LayoutScalar>(
    source_index: usize,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AvailableOf<S>,
) -> InlineBoundaryControlOf<S> {
    InlineBoundaryControlOf::new(
        source_index,
        input.kind(),
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        InlineControlAlignment::from(input.vertical_align()),
        input.edge(),
    )
}

struct InlineRunRequest<'context, 'input, Node, S: LayoutScalar> {
    semantic: Option<&'input crate::TextParagraphBinding<Node>>,
    context: InlineRunContext<'context, S>,
    adjoining_margin: S,
}

fn layout_inline_run_children<'input, Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    request: InlineRunRequest<'_, 'input, <Tree as Traverse>::Node, S>,
    float_exclusions: &mut FloatExclusions<S, <Tree as Traverse>::Node>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunRequest {
        semantic,
        context,
        adjoining_margin,
    } = request;
    let InlineRunContext {
        mut box_frontier,
        line_box_context,
        indent_state,
        hyphenation_state,
        tail,
        source_index_start,
        cursor_block,
        owned_float_block_end,
        constants,
        input,
        node_inner_size,
        set_layout,
    } = context;
    // The native inventory settles whether this run consumes adjoining
    // margins before any positioning, float-band query or published output.
    // Text-backed runs retain their existing placement contract here.
    let uncollapsed_cursor_block = cursor_block;
    let mut cursor_block = cursor_block + adjoining_margin;
    let mut margin = InlineRunMargin::Separates;
    let logical_node_inner_size = constants.flow_axes.logical_size(node_inner_size);
    let available_inline_extent = logical_node_inner_size
        .inline
        .map(AvailableOf::<S>::definite)
        .unwrap_or(
            constants
                .flow_axes
                .logical_size(constants.available_content)
                .inline,
        );
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut participants = Vec::new();
    let mut atomic_children = Vec::new();
    let mut control_children = Vec::new();
    let text_binding = semantic.or_else(|| {
        tree.text_computation()
            .and_then(|state| state.paragraph(container, run))
    });
    if text_binding.is_some()
        && constants.inline_baseline_context != crate::InlineBaselineContext::Physical
    {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(container),
            LayoutOperation::TextCapture,
            LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::NamedInlineBaseline),
        ));
    }
    if text_binding.is_none()
        && run.iter().any(|node| {
            let input = tree.layout_input(*node);
            input.as_inline_text().is_some() || input.as_text_control().is_some()
        })
    {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(container),
            LayoutOperation::TextCapture,
            LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::TextCapture),
        ));
    }
    let mut text_atomics = Vec::new();
    let mut float_events = Vec::new();
    let mut encounter_plans = Vec::new();
    let mut static_positions = Vec::new();
    let mut absolute_children = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let source_index = source_index_start + offset;
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let child_style = match projection.kind() {
            InlineParticipantKindOf::InlineText(_) => continue,
            InlineParticipantKindOf::TextControl(_) => {
                control_children.push((child, source_index));
                continue;
            }
            InlineParticipantKindOf::Box(style) => style,
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
                    continue;
                }
                let line_break =
                    visible_line_break_in_flow(tree, container, child, constants.flow_axes)?
                        .expect("visible line-break input remains visible after validation");
                validate_metric_reference(
                    container,
                    constants.inline_baseline_context,
                    line_break.metrics(),
                    line_break.vertical_align(),
                )?;
                participants.push(MixedInlineParticipantOf::ForcedLineBreak(
                    forced_line_break_control(source_index, line_break, available_inline_extent),
                ));
                control_children.push((child, source_index));
                continue;
            }
            InlineParticipantKindOf::InlineBoundary(_) => {
                let boundary =
                    visible_inline_boundary_in_flow(tree, container, child, constants.flow_axes)?
                        .expect("inline-boundary input remains present after validation");
                validate_metric_reference(
                    container,
                    constants.inline_baseline_context,
                    boundary.metrics(),
                    boundary.vertical_align(),
                )?;
                participants.push(MixedInlineParticipantOf::Boundary(inline_boundary_control(
                    source_index,
                    boundary,
                    available_inline_extent,
                )));
                control_children.push((child, source_index));
                continue;
            }
        };
        if child_style.display == crate::Display::None {
            if set_layout {
                tree.set_unrounded(
                    child,
                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
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
            continue;
        }
        if child_style.common.position == Position::Absolute {
            absolute_children.push(child);
            continue;
        }

        let participation = if text_binding.is_some() {
            None
        } else {
            Some(child_style.atomic_inline_participation.ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(child),
                    LayoutOperation::ChildLayout,
                    LayoutErrorKindOf::InvalidInput(
                        LayoutInvalidInputOf::AtomicInlineParticipation {
                            reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
                        },
                    ),
                )
            })?)
        };
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
        let named_demand = named_atomic_demand(&child_style, constants.inline_baseline_context);
        let mut child_request = ComputeInputOf::<S>::for_child(
            input.run_mode().for_child(),
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            constants.definite_child_containing_block_size(),
            ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
            constants.flow_axes.physical_size(LogicalSizeOf::new(
                available_inline_extent,
                AvailableOf::<S>::MAX_CONTENT,
            )),
        )
        .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
        .with_named_baseline_demand(named_demand);
        let packet = if !child_style.float.is_none() && text_binding.is_some() {
            // Use the ordinary block owner's complete float child request. The
            // paragraph capture changes encounter scheduling, not child sizing.
            let axes = child_style.common.flow_axes;
            let parent = constants.child_containing_block_size(axes);
            let logical_parent = axes.logical_size(parent);
            let available = axes.logical_size(constants.available_content);
            let margin = constants.flow_axes.zip_physical_edges_with_inline_extent(
                *child_style.common.margin,
                node_inner_size,
                |length, basis| length.resolve_auto_with_status(basis),
            );
            let margin = axes
                .logical_edges(margin)
                .map(super::in_flow::resolved_length_auto_fallback_zero);
            let inline = logical_parent
                .inline
                .or(available.inline.into_option())
                .map(|inline| (inline - margin.inline_sum()).max(S::ZERO));
            let known = super::in_flow::in_flow_child_known_size::<Tree, M>(
                tree,
                child,
                &child_style,
                child_padding + child_border,
                axes,
                logical_parent,
                inline,
            )?;
            child_request = ComputeInputOf::<S>::for_child(
                input.run_mode().for_child(),
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known,
                parent,
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                axes.physical_size(LogicalSizeOf::new(
                    super::in_flow::in_flow_child_available_inline(
                        &child_style,
                        axes,
                        inline,
                        available.inline,
                    ),
                    AvailableOf::<S>::MAX_CONTENT,
                )),
            )
            .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
            .with_block_indent_context(crate::text_indent::BlockIndentContext {
                inherited: constants.text_indent,
                first_formatted: indent_state.first_formatted() && constants.first_formatted,
            })
            .with_named_baseline_demand(named_demand);
            Some(tree.probe_float_child(child, child_request)?)
        } else {
            None
        };
        let output = if let Some(packet) = &packet {
            packet.output
        } else {
            let result = tree.compute_child(child, child_request)?;
            if let Some(groups) = tree.group_computation() {
                groups.child(child);
            }
            result
        };
        let unresolved_margin = constants
            .flow_axes
            .zip_physical_edges_with_inline_extent(
                *child_style.common.margin,
                node_inner_size,
                |length, basis| resolve_auto_optional(length, basis),
            )
            .transpose_with_node(tree, child)?;
        let child_margin = resolve_atomic_inline_margin(unresolved_margin);
        if let Some(packet) = packet {
            let pending = PendingFloat {
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
            float_events.push(
                super::float_text::FloatEvent::new(
                    pending,
                    packet,
                    text_binding.expect("actual float is in captured paragraph"),
                    &tree
                        .text_computation()
                        .expect("captured float keeps source registry")
                        .associations,
                    offset,
                    child_request,
                )
                .map_err(|e| text_run_error(container, e))?,
            );
            continue;
        }
        let item = atomic_inline_box_participant(
            source_index,
            child_style.clone(),
            output,
            child_margin,
            child_padding,
            child_border,
            AtomicInlineContext {
                axes: constants.flow_axes,
                baseline: constants.inline_baseline_context,
            },
        )
        .map_err(|kind| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(child),
                LayoutOperation::ChildLayout,
                kind,
            )
        })?;
        if let Some(participation) = participation {
            participants.push(MixedInlineParticipantOf::Atomic {
                item,
                participation,
            });
        }
        if text_binding.is_some() {
            let relative_offset = relative_inset_offset(
                constants
                    .flow_axes
                    .zip_physical_edges_with_inline_extent(
                        *child_style.common.inset,
                        node_inner_size,
                        |length, basis| resolve_auto_optional(length, basis),
                    )
                    .transpose_with_node(tree, child)?,
                constants.flow_axes,
            );
            let content_box = output
                .scroll_geometry
                .map(|geometry| geometry.content_box())
                .unwrap_or_else(|| {
                    crate::ScrollRectOf::try_new(
                        Point::new(
                            child_border.left + child_padding.left,
                            child_border.top + child_padding.top,
                        ),
                        Size::new(
                            (output.size.width
                                - child_border.left
                                - child_border.right
                                - child_padding.left
                                - child_padding.right)
                                .max(S::ZERO),
                            (output.size.height
                                - child_border.top
                                - child_border.bottom
                                - child_padding.top
                                - child_padding.bottom)
                                .max(S::ZERO),
                        ),
                    )
                    .expect("validated actual atomic box has a finite content rectangle")
                });
            text_atomics.push(crate::text_layout::run::Atomic {
                node: child,
                item,
                content_box,
                relative_offset,
            });
        }

        atomic_children.push(RetainedAtomicChild {
            node: child,
            source_index,
            projection,
            request: child_request,
            output,
            margin: child_margin,
        });
    }
    let logical_content_box_inset = constants.logical_content_box_inset();
    let provider_error = std::cell::RefCell::new(None);
    let mut text_output = None;
    let mut formatted_item = None;
    let mut report = if let Some(binding) = text_binding {
        let (frame, frames, reusable, associations) = {
            let state = tree
                .text_computation()
                .expect("capture-backed branch retains its session state");
            (
                state.frame(container),
                state.frames.clone(),
                state.reusable.clone(),
                state.associations.clone(),
            )
        };
        let canonical_exclusions = float_exclusions
            .canonical_for_text(container, &frames)
            .map_err(|error| text_run_error(container, error))?;
        let exclusions = canonical_exclusions.as_ref().unwrap_or(float_exclusions);
        let actual_content = frame
            .and_then(|frame| frame.output.scroll_geometry)
            .map(|geometry| geometry.content_box());
        let inset = actual_content.map_or(
            Point::new(
                constants.content_box_inset.left,
                constants.content_box_inset.top,
            ),
            |rect| rect.origin(),
        );
        let right_inset = frame
            .zip(actual_content)
            .map_or(constants.content_box_inset.right, |(frame, rect)| {
                frame.output.size.width - rect.origin().x - rect.size().width
            });
        let available = match available_inline_extent {
            AvailableOf::Definite(_) => actual_content.map_or(available_inline_extent, |rect| {
                AvailableOf::definite(constants.flow_axes.logical_size(rect.size()).inline)
            }),
            // An intrinsic result is not a newly authored definite constraint.
            AvailableOf::MinContent | AvailableOf::MaxContent => available_inline_extent,
        };
        let formatted_mode = match available {
            AvailableOf::Definite(value) => {
                crate::text_layout::fit::Available::Definite(value.to_f64())
            }
            AvailableOf::MinContent => crate::text_layout::fit::Available::Minimum,
            AvailableOf::MaxContent => crate::text_layout::fit::Available::Maximum,
        };
        let formatted = tree
            .text_computation()
            .map(|state| {
                state.formatted_snapshot(container, run, &binding.prepared, input, formatted_mode)
            })
            .unwrap_or_default();
        let group_snapshot = tree
            .group_computation()
            .map(|groups| groups.snapshot())
            .unwrap_or_default();
        let context = crate::text_layout::run::Context {
            groups: group_snapshot,
            wrapping: tree.wrapping_context().and_then(|(context, graph)| {
                crate::text_layout::run::WrappingProjection::new(
                    context,
                    graph,
                    binding,
                    &associations,
                )
                .map(|projection| projection.with_incoming(box_frontier.clone()))
            }),
            formatted,
            frame: frame.map_or(crate::text_layout::run::Frame::Measurement, |frame| {
                crate::text_layout::run::Frame::Canonical {
                    origin: frame.origin,
                    size: frame.output.size,
                }
            }),
            containing_size,
            inset,
            right_inset,
            cursor_block,
            axes: constants.flow_axes,
            available,
            wrap_style: constants.wrap_style,
            alignment: constants.text_align,
            last_alignment: constants.text_align_last,
            justification: constants.text_justification,
            indent: resolve_text_indent(container, constants.text_indent, available)?,
            indent_state,
            hyphenation_state,
            hyphenate_limit_zone: constants.hyphenate_limit_zone,
            line_box_context,
            tail,
        };
        let text_controls = binding
            .controls
            .iter()
            .map(|binding| {
                let node = binding.node;
                let source_index = source_index_start
                    + run
                        .iter()
                        .position(|child| *child == node)
                        .expect("capture admitted the actual control node");
                let (kind, strut, endpoint) = match tree.layout_input(node) {
                    crate::LayoutInputViewOf::LineBreak(control) => (
                        crate::inline::InlineParticipantLayoutKind::ForcedLineBreak,
                        Some((control.metrics(), control.vertical_align())),
                        None,
                    ),
                    crate::LayoutInputViewOf::InlineBoundary(boundary) => (
                        match boundary.kind() {
                            crate::InlineBoundaryKind::Start => {
                                crate::inline::InlineParticipantLayoutKind::InlineBoundaryStart
                            }
                            crate::InlineBoundaryKind::End => {
                                crate::inline::InlineParticipantLayoutKind::InlineBoundaryEnd
                            }
                        },
                        Some((boundary.metrics(), boundary.vertical_align())),
                        Some(crate::inline::InlineEndpointOf::from_input(boundary)),
                    ),
                    crate::LayoutInputViewOf::TextControl(_) => (
                        crate::inline::InlineParticipantLayoutKind::SemanticControl,
                        None,
                        None,
                    ),
                    _ => unreachable!("validated actual control retains its normalized role"),
                };
                let boundary = endpoint
                    .map(|endpoint| {
                        surgeist_text::InlineBoundaryAllocation::try_new(
                            endpoint.edge.advance().to_f64(),
                        )
                        .map_err(|error| text_run_error(container, error.into()))
                    })
                    .transpose()?;
                Ok(crate::text_layout::run::Control {
                    node,
                    source_index,
                    kind,
                    endpoint,
                    allocation: surgeist_text::ControlAllocation {
                        occurrence: binding.occurrence.clone(),
                        boundary,
                        strut: strut.map(|(metrics, alignment)| surgeist_text::ControlStrut {
                            baseline: metrics.baseline().to_f64(),
                            line_extent: metrics.line_extent().to_f64(),
                            alignment: match alignment {
                                crate::VerticalAlign::Baseline => {
                                    surgeist_text::LineBoxAlignment::Baseline
                                }
                                crate::VerticalAlign::Top => surgeist_text::LineBoxAlignment::Top,
                                crate::VerticalAlign::Bottom => {
                                    surgeist_text::LineBoxAlignment::Bottom
                                }
                            },
                        }),
                    },
                })
            })
            .collect::<Result<Vec<_>, LayoutErrorOf<Tree::Node, S, M>>>()?;
        let intrinsic_max =
            !float_events.is_empty() && matches!(available, AvailableOf::MaxContent);
        let quality_replay = matches!(
            constants.wrap_style,
            crate::TextWrapStyle::Balance | crate::TextWrapStyle::AvoidShortLastLine
        ) && matches!(available, AvailableOf::Definite(_));
        let encounters = std::cell::RefCell::new(super::float_text::Encounters::new(
            float_events,
            exclusions.clone(),
            cursor_block,
            if intrinsic_max {
                super::floats::FloatInlineSpace::IntrinsicMaximum
            } else {
                super::floats::FloatInlineSpace::Finite
            },
            quality_replay
                || tail == crate::text_layout::fit::TailContext::CompleteBlock
                    && matches!(available, AvailableOf::Definite(_)),
            quality_replay,
        ));
        let result = crate::text_layout::run::layout(
            binding,
            &associations,
            &text_atomics,
            &text_controls,
            context,
            &reusable,
            crate::text_layout::run::LineEnvironment {
                reset: |mode| match mode {
                    crate::text_layout::fit::EncounterReset::Candidate => {
                        encounters.borrow_mut().reset_candidate()
                    }
                    crate::text_layout::fit::EncounterReset::Sequence => {
                        encounters.borrow_mut().restart_sequence()
                    }
                    crate::text_layout::fit::EncounterReset::Group => {
                        encounters.borrow_mut().restart_group()
                    }
                },
                checkpoint_group: || encounters.borrow_mut().checkpoint_group(),
                band: |block_start, block_end| {
                    let block_start = crate::text_layout::run::number::<S>(block_start)?;
                    let block_end = crate::text_layout::run::number::<S>(block_end)?;
                    let start = cursor_block + block_start;
                    let end = cursor_block + block_end;
                    let band = if set_layout {
                        match encounters
                            .borrow()
                            .band()
                            .query_provider_band(tree, container, start, end)
                        {
                            Ok(band) => band,
                            Err(error) => {
                                *provider_error.borrow_mut() = Some(error);
                                return Err(crate::text_layout::fit::FitError::Dependency);
                            }
                        }
                    } else {
                        encounters
                            .borrow()
                            .band()
                            .query_rectangular_line_band(start, end)
                    };
                    let inset_inline = constants
                        .flow_axes
                        .logical_edges(Edges::new(inset.y, right_inset, S::ZERO, inset.x))
                        .inline_start;
                    let intrinsic_end_occupation = intrinsic_max
                        .then(|| encounters.borrow().band().end_occupation(&band).to_f64());
                    Ok(crate::text_layout::fit::Band {
                        start: (band.inline_start - inset_inline).to_f64(),
                        end: (band.inline_end - inset_inline).to_f64(),
                        intrinsic_end_occupation,
                        next_transition: band
                            .next_transition
                            .map(|next| (next - cursor_block).to_f64()),
                    })
                },
                clear: |block, request: &surgeist_text::LineSelectionRequest| {
                    let clear = request
                        .participants
                        .iter()
                        .rev()
                        .find_map(|participant| {
                            let surgeist_text::PreparedParticipant::Control(control) = participant
                            else {
                                return None;
                            };
                            let row = binding
                                .controls
                                .iter()
                                .find(|row| &row.occurrence == control)?;
                            tree.layout_input(row.node)
                                .as_line_break()
                                .map(|control| control.clear())
                        })
                        .unwrap_or(crate::Clear::None);
                    let block = crate::text_layout::run::number::<S>(block)?;
                    let query = cursor_block + block;
                    let cleared = encounters.borrow().band().clearance_block(query, clear);
                    // Only the actual same-scalar clearance operation can issue a
                    // minimum. Narrowing the queried position alone is not a
                    // clearance and must not replace the fitter's f64 position.
                    Ok((cleared > query).then(|| cleared.to_f64() - cursor_block.to_f64()))
                },
                replay: |line: &surgeist_text::FinalizedLine, replay| match encounters
                    .borrow_mut()
                    .replay(tree, container, line, replay, set_layout)
                {
                    Ok(basis) => Ok(basis),
                    Err(error) => {
                        *provider_error.borrow_mut() = Some(error);
                        Err(crate::text_layout::fit::FitError::Dependency)
                    }
                },
                accept: || {
                    encounters.borrow_mut().accept();
                    Ok(())
                },
            },
        );
        if let Some(error) = provider_error.borrow_mut().take() {
            return Err(error);
        }
        let output = result.map_err(|error| text_run_error(container, error))?;
        formatted_item = tree.text_computation().and_then(|state| {
            state.formatted_merge_run(
                container,
                run,
                &binding.prepared,
                input,
                formatted_mode,
                output.formatted.clone(),
            )
        });
        let accepted = encounters.into_inner().finish();
        for child in accepted.children {
            tree.accept_float_child(&child.packet)?;
            if let Some(groups) = tree.group_computation() {
                groups.child(child.packet.node);
            }
            encounter_plans.push(child.pending);
        }
        *float_exclusions = accepted.ledger;
        let report = output.report.clone();
        text_output = Some(output);
        report
    } else {
        let wrapping = {
            tree.wrapping_context()
                .map(|(context, graph)| {
                    // Wrapping cuts index emitted participants, not native slots
                    // omitted for hidden or out-of-flow children.
                    let nodes: Vec<_> = participants
                        .iter()
                        .map(|participant| {
                            let source_index = match participant {
                                MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
                                MixedInlineParticipantOf::ForcedLineBreak(control) => {
                                    control.source_index()
                                }
                                MixedInlineParticipantOf::Boundary(control) => {
                                    control.source_index()
                                }
                            };
                            run[source_index - source_index_start]
                        })
                        .collect();
                    context
                        .numeric_projection(graph, container, &nodes)
                        .map(|projection| {
                            if matches!(available_inline_extent, AvailableOf::MinContent) {
                                projection.without_avoidance()
                            } else {
                                projection
                            }
                        })
                })
                .transpose()
                .map_err(|error| crate::inline_wrapping::wrapping_error(container, error))?
        };
        if !crate::inline::numeric_run_has_formatted_line(&participants, wrapping.as_ref()) {
            margin = InlineRunMargin::Adjoins;
            cursor_block = uncollapsed_cursor_block;
        }
        let observe_formatted = matches!(
            constants.wrap_style,
            crate::TextWrapStyle::Balance | crate::TextWrapStyle::AvoidShortLastLine
        ) && matches!(available_inline_extent, AvailableOf::Definite(_))
            || tree
                .text_computation()
                .is_some_and(|state| state.formatted_active());
        let numeric_input = MixedInlineRunInputOf {
            inline_baseline_context: constants.inline_baseline_context,
            groups: tree
                .group_computation()
                .map(|groups| groups.snapshot())
                .unwrap_or_default(),
            wrapping,
            text_wrap_style: constants.wrap_style,
            line_box_context,
            observe_tail: tail != crate::text_layout::fit::TailContext::Unrestricted,
            available_inline_extent,
            flow_axes: constants.flow_axes,
            text_align: constants.text_align,
            text_align_last: constants.text_align_last,
            indent: resolve_text_indent(container, constants.text_indent, available_inline_extent)?,
            indent_state,
            participants,
        };
        // The callbacks are sequential. A receiving fact is resolved before
        // any candidate band query, without aliasing the mutable child owner.
        let numeric_tree = std::cell::RefCell::new(&mut *tree);
        let result = layout_mixed_inline_run_with_formatted_observation(
            numeric_input,
            |block_start, block_end| {
                let query_block_start = cursor_block + block_start;
                let query_block_end = cursor_block + block_end;
                let band = if !set_layout || provider_error.borrow().is_some() {
                    float_exclusions.query_rectangular_line_band(query_block_start, query_block_end)
                } else {
                    match float_exclusions.query_provider_band(
                        &**numeric_tree.borrow(),
                        container,
                        query_block_start,
                        query_block_end,
                    ) {
                        Ok(band) => band,
                        Err(error) => {
                            *provider_error.borrow_mut() = Some(error);
                            FloatBand {
                                inline_start: float_exclusions.containing_inline_start,
                                inline_end: float_exclusions.containing_inline_end,
                                next_transition: None,
                                #[cfg(test)]
                                evaluated: 0,
                            }
                        }
                    }
                };
                LogicalLineBandQueryResultOf {
                    inline_start: band.inline_start - logical_content_box_inset.inline_start,
                    inline_end: band.inline_end - logical_content_box_inset.inline_start,
                    next_transition: band
                        .next_transition
                        .map(|transition| transition - cursor_block),
                }
            },
            |block, clear| {
                float_exclusions.clearance_for_line_intent(cursor_block + block, clear)
                    - cursor_block
            },
            observe_formatted,
            |source_index, expected| {
                if provider_error.borrow().is_some() {
                    return Err(
                        crate::inline_wrapping::NumericWrappingError::InvalidAtomicReceiverBasis,
                    );
                }
                match resolve_atomic_receiver(
                    &mut **numeric_tree.borrow_mut(),
                    container,
                    &mut atomic_children,
                    source_index,
                    expected,
                    constants.flow_axes,
                ) {
                    Ok(strut) => Ok(strut),
                    Err(error) => {
                        *provider_error.borrow_mut() = Some(error);
                        Err(crate::inline_wrapping::NumericWrappingError::InvalidAtomicReceiverBasis)
                    }
                }
            },
        );
        // Child/provider errors retain their exact context even if provisional
        // geometry makes a rejected quality trial fail its private replay proof.
        if let Some(error) = provider_error.borrow_mut().take() {
            return Err(error);
        }
        result.map_err(|error| crate::inline_wrapping::numeric_wrapping_error(container, error))?
    };
    for child in absolute_children {
        static_positions.push((
            child,
            absolute_static_position(cursor_block, constants, containing_size),
        ));
    }
    if text_output.is_none() {
        if let Some(lines) = &report.formatted_lines {
            formatted_item = Some(crate::text_layout::formatted::FormattedFlowItem::Numeric(
                run.to_vec(),
                lines.clone(),
            ));
        }
        report.hyphenation_state = hyphenation_state;
        if matches!(available_inline_extent, AvailableOf::Definite(_))
            && !report.line_bands.is_empty()
        {
            // The numeric box/control path committed real nonhyphenated lines.
            // Empty wrappers and floats with no formatted line do not reset.
            report.hyphenation_state.commit(false);
        }
    }
    if let Some(error) = provider_error.into_inner() {
        return Err(error);
    }
    let resolved_float_terminal_block_end =
        resolved_inline_float_terminal_block_end(&report, cursor_block, owned_float_block_end);
    let report_logical_size = LogicalSizeOf::new(report.inline_extent, report.block_extent);
    let report_size = constants.flow_axes.physical_size(report_logical_size);
    let project_point = |inline: S, block: S, size: LogicalSizeOf<S>| {
        constants.flow_axes.physical_point(
            LogicalPointOf::new(
                logical_content_box_inset.inline_start + inline,
                cursor_block + block,
            ),
            size,
            containing_size,
        )
    };

    let mut text_content_size = Size::ZERO;
    if let Some(output) = text_output {
        if let Some(state) = tree.text_computation() {
            state.record_boxes(crate::text_layout::boxes::BoxRunUse {
                container,
                prepared: output.paragraph.prepared.clone(),
                nodes: run.to_vec(),
                input,
                available: match available_inline_extent {
                    AvailableOf::Definite(value) => {
                        crate::text_layout::fit::Available::Definite(value.to_f64())
                    }
                    AvailableOf::MinContent => crate::text_layout::fit::Available::Minimum,
                    AvailableOf::MaxContent => crate::text_layout::fit::Available::Maximum,
                },
                trials: output.box_uses.clone(),
            });
        }
        if let Some(Some(assignment)) = output.paragraph.boxes.last() {
            box_frontier = Some(assignment.outgoing.clone());
        }
        // Every normal-flow line allocation and actual placed metric bound
        // contributes independently, including atomic-only and terminal empty
        // lines, beside genuine source-fragment and child-box contributions.
        for rect in output
            .line_bounds
            .iter()
            .flat_map(|bounds| [bounds.allocation, bounds.metrics])
            .chain(
                output
                    .fragments
                    .iter()
                    .filter_map(|(_, fragment)| fragment.rect()),
            )
        {
            let location = rect.origin();
            text_content_size = max_content_size(
                text_content_size,
                content_size_contribution(
                    Point::new(
                        location.x - constants.content_box_inset.left,
                        location.y - constants.content_box_inset.top,
                    ),
                    rect.size(),
                    rect.size(),
                    ComputedOverflow::VISIBLE,
                    false,
                ),
            );
            if set_layout {
                contributions.include_direct_line(rect);
            }
        }
        let mut groups: Vec<TextFragmentGroup<Tree::Node, S>> = Vec::new();
        let mut scope_fragments = Vec::new();
        for (owner, fragment) in output.fragments {
            let crate::InlineFragmentOwner::Node(node) = owner else {
                scope_fragments.push(crate::InlineFragmentOutputEntryOf::new(owner, fragment));
                continue;
            };
            let index = groups
                .iter()
                .position(|(subject, _, _, _)| *subject == node);
            let index = index.unwrap_or_else(|| {
                groups.push((node, Vec::new(), None, None));
                groups.len() - 1
            });
            let (_, fragments, minimum, maximum) = &mut groups[index];
            if let Some(rect) = fragment.rect() {
                let location = rect.origin();
                let end = Point::new(
                    location.x + rect.size().width,
                    location.y + rect.size().height,
                );
                *minimum = Some(minimum.map_or(location, |point| {
                    Point::new(point.x.min(location.x), point.y.min(location.y))
                }));
                *maximum = Some(maximum.map_or(end, |point| {
                    Point::new(point.x.max(end.x), point.y.max(end.y))
                }));
            }
            fragments.push(fragment);
        }
        for (node, actual) in output.static_positions {
            if let Some((_, position)) = static_positions
                .iter_mut()
                .find(|(subject, _)| *subject == node)
            {
                *position = actual;
            } else {
                static_positions.push((node, actual));
            }
        }
        if set_layout {
            for (child, location) in output.nonformatted_controls {
                let source_index = source_index_start
                    + run
                        .iter()
                        .position(|node| *node == child)
                        .expect("nonformatted descendant is an actual bound control");
                let (border, padding) = match tree.layout_input(child) {
                    crate::LayoutInputViewOf::InlineBoundary(boundary) => {
                        crate::inline::InlineEndpointOf::from_input(boundary).physical_components()
                    }
                    _ => (Edges::ZERO, Edges::ZERO),
                };
                tree.set_unrounded(
                    child,
                    NodeOutputOf::<S> {
                        source_index: crate::SourceIndex::new(source_index),
                        location,
                        border,
                        padding,
                        ..NodeOutputOf::new()
                    },
                );
            }
        }
        if set_layout {
            for row in text_binding
                .into_iter()
                .flat_map(|binding| binding.origins.iter())
            {
                if !groups.iter().any(|(node, _, _, _)| *node == row.node) {
                    groups.push((row.node, Vec::new(), None, None));
                }
            }
            for (node, fragments, minimum, maximum) in groups {
                let (location, size) = match (minimum, maximum) {
                    (Some(minimum), Some(maximum)) => (
                        minimum,
                        Size::new(maximum.x - minimum.x, maximum.y - minimum.y),
                    ),
                    _ => (Point::ZERO, Size::ZERO),
                };
                let source_index = source_index_start
                    + run
                        .iter()
                        .position(|child| *child == node)
                        .expect("capture admitted this source node");
                tree.set_unrounded(
                    node,
                    NodeOutputOf {
                        source_index: crate::SourceIndex::new(source_index),
                        location,
                        size,
                        content_size: size,
                        ..NodeOutputOf::new()
                    },
                );
                tree.set_unrounded_inline_fragment_state(node, Some(fragments));
            }
            let state = tree
                .text_computation()
                .expect("capture branch retains actual Text state");
            state.record_scope_fragments(
                output.paragraph.container,
                output.paragraph.prepared.clone(),
                output.paragraph.nodes.clone(),
                scope_fragments,
            );
            state.record_atomic_origins(output.atomic_origins);
            state.record(output.paragraph);
        }
    }

    let atomic_sources = report
        .atomics
        .iter()
        .map(|source| (source.item.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let mut content_size = report_size;
    let mut scroll_content_size = text_content_size;
    for RetainedAtomicChild {
        node: child,
        source_index,
        projection: child_style,
        output,
        ..
    } in atomic_children
    {
        let source = atomic_sources[&source_index];
        let logical_size = constants.flow_axes.logical_size(source.item.size);
        let projected_location =
            project_point(source.inline_start, source.block_start, logical_size);
        let InlineParticipantKindOf::Box(child_projection) = child_style.kind() else {
            unreachable!("atomic child retains its settled box projection");
        };
        let inset_offset = if text_binding.is_some() {
            Point::ZERO
        } else {
            relative_inset_offset(
                constants
                    .flow_axes
                    .zip_physical_edges_with_inline_extent(
                        *child_projection.common.inset,
                        node_inner_size,
                        |length, basis| resolve_auto_optional(length, basis),
                    )
                    .transpose_with_node(tree, child)?,
                constants.flow_axes,
            )
        };
        let location = Point::new(
            projected_location.x + inset_offset.x,
            projected_location.y + inset_offset.y,
        );
        let contribution = content_size_contribution(
            Point::new(
                location.x - constants.content_box_inset.left,
                location.y - constants.content_box_inset.top,
            ),
            source.item.size,
            output.content_size,
            child_projection.common.overflow,
            child_projection.common.item_is_replaced,
        );
        content_size = max_content_size(content_size, contribution);
        scroll_content_size = max_content_size(scroll_content_size, contribution);

        if set_layout {
            let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
                tree,
                child,
                |box_projection, target_projection| {
                    retained_child_scroll_geometry(
                        box_projection,
                        target_projection,
                        source.item.size,
                        source.item.content_size,
                        source.item.padding,
                        source.item.border,
                        output.scroll_geometry,
                    )
                },
            )
            .map_err(|error| layout_child_geometry_error(container, child, error))?;
            contributions
                .include_in_flow_geometry(location, source.item.margin, scroll_geometry)
                .map_err(|error| layout_child_geometry_error(container, child, error))?;
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size: source.item.size,
                    named_baselines: output.named_baselines,
                    content_size: source.item.content_size,
                    border: source.item.border,
                    padding: source.item.padding,
                    margin: source.item.margin,
                    ..NodeOutputOf::new()
                }
                .with_scroll_geometry(Some(scroll_geometry)),
            );
        }
    }

    let control_sources = report
        .controls
        .iter()
        .map(|source| (source.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    if set_layout {
        let nonformatted_controls = report
            .nonformatted_controls
            .iter()
            .map(|source| (source.control.source_index(), source))
            .collect::<BTreeMap<_, _>>();
        for (child, source_index) in control_children {
            let (inline_start, block_start, inline_extent, endpoint) =
                if let Some(source) = control_sources.get(&source_index) {
                    (
                        source.inline_start,
                        source.block_start,
                        source.inline_extent,
                        source.endpoint,
                    )
                } else if let Some(source) = nonformatted_controls.get(&source_index) {
                    (
                        source.inline_start,
                        source.block_start,
                        source.control.edge().advance(),
                        Some(crate::inline::InlineEndpointOf::from_control(
                            source.control,
                        )),
                    )
                } else {
                    continue;
                };
            let logical_size = LogicalSizeOf::new(inline_extent, S::ZERO);
            let size = constants.flow_axes.physical_size(logical_size);
            let (border, padding) = endpoint.map_or((Edges::ZERO, Edges::ZERO), |endpoint| {
                endpoint.physical_components()
            });
            tree.set_unrounded(
                child,
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location: project_point(inline_start, block_start, logical_size),
                    size,
                    border,
                    padding,
                    ..NodeOutputOf::new()
                },
            );
        }
    }

    let projected_baseline = |baseline| {
        constants.flow_axes.block_axis_coordinate(project_point(
            S::ZERO,
            baseline,
            LogicalSizeOf::new(S::ZERO, S::ZERO),
        ))
    };
    let baselines = BaselinesOf::from_block_coordinates(
        constants.flow_axes,
        report.first_baseline.map(projected_baseline),
        report.last_baseline.map(projected_baseline),
    );

    if let Some(groups) = tree.group_computation() {
        for line in &report.group_lines {
            groups.observe(line.clone());
        }
    }
    let float_end = encounter_plans
        .iter()
        .filter_map(|plan| {
            plan.placement.text().map(|accepted| {
                let margin = constants.flow_axes.logical_edges(plan.margin);
                accepted.logical_origin.block
                    + constants.flow_axes.logical_size(plan.size).block
                    + margin.block_end
            })
        })
        .fold(owned_float_block_end, |a, b| a.max(b));
    Ok(InlineRunPlacement {
        inline_baseline_context: report.inline_baseline_context,
        first_named_baseline: report.first_named_baseline,
        last_named_baseline: report.last_named_baseline,
        margin,
        box_frontier,
        formatted: formatted_item,
        line_box_context: report.line_box_context,
        lines: report.lines,
        floats: encounter_plans,
        owned_float_block_end: float_end,
        indent_state: (!report.line_bands.is_empty()).then_some(report.indent_state),
        hyphenation_state: report.hyphenation_state,
        size: report_size,
        content_size,
        scroll_content_size,
        static_positions,
        baselines,
        first_baseline: report.first_baseline,
        last_baseline: report.last_baseline,
        resolved_float_terminal_block_end,
    })
}

fn record_inline_run_baselines<S: LayoutScalar>(
    baselines: &mut BaselinesOf<S>,
    placement: &InlineRunPlacement<impl Copy, S>,
    cursor_block: S,
    constants: &Constants<S>,
) {
    if constants.flow_axes.inline_axis() == PhysicalAxis::Vertical {
        baselines.record_first(placement.baselines.first);
        baselines.record_last(placement.baselines.last);
        return;
    }

    if let Some(baseline) = placement.first_baseline {
        baselines.record_first(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                Some(cursor_block + baseline),
                None,
            )
            .first,
        );
    }
    if let Some(baseline) = placement.last_baseline {
        baselines.record_last(
            BaselinesOf::from_block_coordinates(
                constants.flow_axes,
                None,
                Some(cursor_block + baseline),
            )
            .last,
        );
    }
}

struct AtomicInlineContext {
    axes: crate::FlowAxes,
    baseline: crate::InlineBaselineContext,
}

fn named_atomic_demand<S: LayoutScalar>(
    child: &BlockChildProjection<'_, S>,
    context: crate::InlineBaselineContext,
) -> crate::output::NamedBaselineDemand {
    match context {
        crate::InlineBaselineContext::Named(name)
            if child.vertical_align == VerticalAlign::Baseline && child.float.is_none() =>
        {
            crate::output::NamedBaselineDemand::for_source(
                name,
                child.baseline_source,
                child.display,
            )
        }
        _ => crate::output::NamedBaselineDemand::None,
    }
}

fn atomic_inline_box_participant<S: LayoutScalar, M>(
    source_index: usize,
    child_style: BlockChildProjection<'_, S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    padding: Edges<S>,
    border: Edges<S>,
    containing: AtomicInlineContext,
) -> Result<AtomicInlineBoxParticipant<S>, LayoutErrorKindOf<S, M>> {
    let AtomicInlineContext {
        axes: containing_flow_axes,
        baseline: context,
    } = containing;
    let logical_size = containing_flow_axes.logical_size(output.size);
    let used_overflow = UsedOverflow::from_computed(
        child_style.common.overflow,
        child_style.common.item_is_replaced,
    );
    let block_overflow = match containing_flow_axes.block_axis() {
        PhysicalAxis::Horizontal => used_overflow.x(),
        PhysicalAxis::Vertical => used_overflow.y(),
    };
    let mut selected_inner_baseline = (child_style.vertical_align == VerticalAlign::Baseline
        && block_overflow.value() == Overflow::Visible)
        .then(|| {
            let baselines = output.baselines();
            containing_flow_axes
                .block_axis_coordinate(baselines.first)
                .or_else(|| containing_flow_axes.block_axis_coordinate(baselines.last))
                .map(|physical| {
                    if containing_flow_axes
                        .logical_axis_progression(crate::LogicalAxis::Block)
                        .is_decreasing()
                    {
                        logical_size.block - physical
                    } else {
                        physical
                    }
                })
        })
        .flatten();
    if let crate::InlineBaselineContext::Named(name) = context
        && child_style.vertical_align == VerticalAlign::Baseline
    {
        selected_inner_baseline = Some(named_atomic_baseline(
            &child_style,
            output,
            margin,
            containing_flow_axes,
            name,
        )?);
    }
    Ok(AtomicInlineBoxParticipant {
        baseline_reference: if child_style.vertical_align == VerticalAlign::Baseline {
            context
        } else {
            crate::InlineBaselineContext::Physical
        },
        source_index,
        size: output.size,
        content_size: output.content_size,
        margin,
        padding,
        border,
        scrollbar_size: child_scrollbar_size(&child_style),
        first_baseline: selected_inner_baseline,
        alignment: child_style.vertical_align.into(),
    })
}

fn named_atomic_baseline<S: LayoutScalar, M>(
    child_style: &BlockChildProjection<'_, S>,
    output: ComputeOutputOf<S>,
    margin: Edges<S>,
    containing_flow_axes: FlowAxes,
    name: crate::BaselineName,
) -> Result<S, LayoutErrorKindOf<S, M>> {
    let logical_size = containing_flow_axes.logical_size(output.size);
    let used_overflow = UsedOverflow::from_computed(
        child_style.common.overflow,
        child_style.common.item_is_replaced,
    );
    let (_, fact) = crate::output::NamedBaselineDemand::for_source(
        name,
        child_style.baseline_source,
        child_style.display,
    )
    .selected_fact(output.named_baselines)
    .expect("a resolved named source requests exactly one fact");
    let synthesized = matches!(fact, crate::BaselineFactOf::Absent);
    let physical = match fact {
        crate::BaselineFactOf::Unknown => {
            return Err(LayoutErrorKindOf::MissingContext(
                crate::LayoutMissingContext::NamedInlineBaseline,
            ));
        }
        crate::BaselineFactOf::Available(value) => {
            if value.axis() != containing_flow_axes.block_axis() {
                return Err(LayoutErrorKindOf::InvalidInput(
                    LayoutInvalidInputOf::InlineBaseline(
                        crate::InlineBaselineError::AxisMismatch {
                            expected: containing_flow_axes.block_axis(),
                            actual: value.axis(),
                        },
                    ),
                ));
            }
            value.offset()
        }
        crate::BaselineFactOf::Absent => {
            let edge = |side| match side {
                crate::PhysicalSide::Top => -margin.top,
                crate::PhysicalSide::Bottom => output.size.height + margin.bottom,
                crate::PhysicalSide::Left => -margin.left,
                crate::PhysicalSide::Right => output.size.width + margin.right,
            };
            let under = edge(containing_flow_axes.line_under());
            match name {
                crate::BaselineName::Alphabetic => under,
                crate::BaselineName::Central => {
                    edge(containing_flow_axes.line_over()) / S::from_f64(2.0)
                        + under / S::from_f64(2.0)
                }
            }
        }
    };
    if !physical.is_finite() {
        return Err(LayoutErrorKindOf::InvalidInput(
            LayoutInvalidInputOf::InvalidNumeric { value: physical },
        ));
    }
    let physical = if synthesized && used_overflow.is_scroll_container() {
        crate::BaselineCoordinateOf::try_new(containing_flow_axes.block_axis(), physical)
            .and_then(|coordinate| coordinate.clamped_to_border(output.size))
            .map_err(|crate::FiniteScalarErrorOf::NonFinite { value }| {
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            })?
            .offset()
    } else {
        physical
    };
    let logical = if containing_flow_axes
        .logical_axis_progression(crate::LogicalAxis::Block)
        .is_decreasing()
    {
        logical_size.block - physical
    } else {
        physical
    };
    if !logical.is_finite() {
        return Err(LayoutErrorKindOf::InvalidInput(
            LayoutInvalidInputOf::InvalidNumeric { value: logical },
        ));
    }
    Ok(logical)
}

fn resolve_atomic_receiver<'input, Tree, S, M>(
    tree: &mut Tree,
    container: Tree::Node,
    children: &mut [RetainedAtomicChild<'input, Tree::Node, S>],
    source_index: usize,
    expected: crate::InlineBaselineContext,
    axes: FlowAxes,
) -> LayoutResultOf<Tree::Node, crate::inline::ReferencedInlineStrutOf<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let invalid = |node| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidAtomicReceiverBasis,
            ),
        )
    };
    let index = children
        .binary_search_by_key(&source_index, |row| row.source_index)
        .map_err(|_| invalid(container))?;
    let row = &mut children[index];
    let InlineParticipantKindOf::Box(style) = row.projection.kind() else {
        return Err(invalid(row.node));
    };
    let crate::InlineBaselineContext::Named(name) = expected else {
        return Err(invalid(row.node));
    };
    if row.request.containing_flow_axes() != axes
        || !matches!(
            style.vertical_align,
            VerticalAlign::Top | VerticalAlign::Bottom
        )
        || !style.float.is_none()
    {
        return Err(invalid(row.node));
    }
    let demand =
        crate::output::NamedBaselineDemand::for_source(name, style.baseline_source, style.display);
    let (_, fact) = demand
        .selected_fact(row.output.named_baselines)
        .expect("an actual named receiver selects one fact");
    if matches!(fact, crate::BaselineFactOf::Unknown)
        && row.request.run_mode() == RunMode::ComputeSize
        && row.request.named_baseline_demand() == crate::output::NamedBaselineDemand::None
    {
        // Refine only the missing fact under the original computation frame.
        // Complete performed outputs are never speculatively placed again.
        let refined =
            tree.compute_child(row.node, row.request.with_named_baseline_demand(demand))?;
        if refined.size != row.output.size {
            return Err(invalid(row.node));
        }
        let (_, fact) = demand
            .selected_fact(refined.named_baselines)
            .expect("the retained source still selects one named fact");
        let pair = row.output.named_baselines.get(name);
        let pair = if style.baseline_source.prefers_last(style.display) {
            crate::BaselinePairOf::new(pair.first(), fact)
        } else {
            crate::BaselinePairOf::new(fact, pair.last())
        };
        row.output.named_baselines = row.output.named_baselines.with(name, pair);
    }
    let inner =
        named_atomic_baseline(&style, row.output, row.margin, axes, name).map_err(|kind| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(row.node),
                LayoutOperation::ChildLayout,
                kind,
            )
        })?;
    let margin = axes.logical_edges(row.margin);
    let metrics = crate::inline::InlineMetricContributionOf {
        baseline: margin.block_start + inner,
        after_baseline: axes.logical_size(row.output.size).block - inner + margin.block_end,
    };
    for value in [metrics.baseline, metrics.after_baseline] {
        if !value.is_finite() {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(row.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value }),
            ));
        }
    }
    Ok(crate::inline::ReferencedInlineStrutOf {
        metrics,
        reference: expected,
        source: crate::inline::InlineStrutSource::Atomic { source_index },
    })
}

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
}

pub(super) fn text_run_error<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    error: crate::text_layout::fit::FitError,
) -> LayoutErrorOf<Node, S, M> {
    let kind = match error {
        crate::text_layout::fit::FitError::Wrapping(error) => {
            return crate::inline_wrapping::wrapping_error(node, error);
        }
        crate::text_layout::fit::FitError::Text(error) => LayoutErrorKindOf::Text(error),
        crate::text_layout::fit::FitError::FeedbackCycle => LayoutErrorKindOf::TextFeedbackCycle,
        crate::text_layout::fit::FitError::SpacingBudget { available, advance } => {
            LayoutErrorKindOf::TextSpacingBudget { available, advance }
        }
        crate::text_layout::fit::FitError::NonFinite(value) => LayoutErrorKindOf::TextNumeric {
            value,
            reason: crate::LayoutTextNumericError::NonFinite,
        },
        crate::text_layout::fit::FitError::Numeric(value, reason) => {
            LayoutErrorKindOf::TextNumeric { value, reason }
        }
        crate::text_layout::fit::FitError::InvalidGeometry => LayoutErrorKindOf::InternalInvariant(
            crate::LayoutInternalInvariant::InvalidRoundedInlineFragmentGeometry,
        ),
        crate::text_layout::fit::FitError::InvalidTailBasis => {
            LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidInlineTailSelection,
            )
        }
        crate::text_layout::fit::FitError::MissingBlockTail => {
            LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::TextBlockTail)
        }
        crate::text_layout::fit::FitError::MissingComputedBoundaryIncidence => {
            LayoutErrorKindOf::MissingContext(
                crate::LayoutMissingContext::ComputedLineBoundaryIncidence,
            )
        }
        crate::text_layout::fit::FitError::MissingZoneBasis => {
            LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis)
        }
        crate::text_layout::fit::FitError::Dependency => {
            unreachable!("the actual dependency error is returned before Text adaptation")
        }
    };
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::TextSelection,
        kind,
    )
}

fn resolve_text_indent<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    policy: crate::TextIndentOf<S>,
    own_available: AvailableOf<S>,
) -> LayoutResultOf<Node, crate::text_indent::ResolvedTextIndent<S>, S, M> {
    let amount =
        match own_available {
            AvailableOf::MinContent | AvailableOf::MaxContent => policy.amount.absolute_px(),
            AvailableOf::Definite(width) => match policy.amount.resolve_against(
                crate::PercentageBasisOf::definite(width).map_err(|_| {
                    LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ValueResolution,
                        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                            value: width,
                        }),
                    )
                })?,
            ) {
                crate::NumericResolutionOf::Resolved(value) => value,
                crate::NumericResolutionOf::MissingBasis { .. } => {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ValueResolution,
                        LayoutErrorKindOf::MissingContext(
                            crate::LayoutMissingContext::RequiredBasis,
                        ),
                    ));
                }
                crate::NumericResolutionOf::InvalidNumeric { resolved, .. } => {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::ValueResolution,
                        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                            value: resolved,
                        }),
                    ));
                }
            },
        };
    if let AvailableOf::Definite(width) = own_available {
        let capacity = width - amount;
        if !capacity.is_finite() {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ValueResolution,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric {
                    value: capacity,
                }),
            ));
        }
    }
    Ok(crate::text_indent::ResolvedTextIndent {
        amount,
        each_line: policy.each_line,
        hanging: policy.hanging,
    })
}

fn validate_metric_reference<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    expected: crate::InlineBaselineContext,
    metrics: crate::InlineMetricsOf<S>,
    alignment: VerticalAlign,
) -> LayoutResultOf<Node, (), S, M> {
    if alignment == VerticalAlign::Baseline && metrics.reference() != expected {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineBaseline(
                crate::InlineBaselineError::ReferenceMismatch {
                    expected,
                    actual: metrics.reference(),
                },
            )),
        ));
    }
    Ok(())
}
