use super::absolute::{StaticPositionGeometry, absolute_static_position};
use super::relative::{self, InsetReference};
use super::scroll::{
    BlockScrollContributions, child_scrollbar_size, retained_child_scroll_geometry,
};
use super::sizing::{resolve_auto_optional, resolve_length_or_zero};
use super::*;
use crate::inline::InlineSourcePositionOf;
use crate::{AxisSizingRequestOf, FlowAxes};

#[cfg(test)]
mod baseline_offset_tests;

type TextFragmentGroup<Node, S> = (
    Node,
    Vec<InlineFragmentOutputOf<S>>,
    Option<Point<S>>,
    Option<Point<S>>,
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
    funding: Option<&crate::text_layout::scope_alignment::EntryFunding<Tree::Scalar>>,
) -> LayoutResultOf<<Tree as Traverse>::Node, usize, <Tree as Traverse>::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let error = |error| text_run_error(container, crate::text_layout::fit::FitError::Text(error));
    let captured_end = match tree.text_computation() {
        Some(state) => state
            .entry_run_starting(container, &children[index - 1..], funding)
            .map_err(error)?
            .map(|run| index - 1 + run.nodes().len()),
        None => None,
    };
    while index < captured_end.unwrap_or(children.len()).min(semantic_end) {
        if let Some(funding) = funding {
            funding.visit(1).map_err(error)?;
        }
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
    if let Some(parent) = boundary.parent_edge_axes() {
        let compatible = matches!(
            (flow.inline_start(), parent.inline_start()),
            (
                crate::PhysicalSide::Left | crate::PhysicalSide::Right,
                crate::PhysicalSide::Left | crate::PhysicalSide::Right
            ) | (
                crate::PhysicalSide::Top | crate::PhysicalSide::Bottom,
                crate::PhysicalSide::Top | crate::PhysicalSide::Bottom
            )
        );
        if !compatible {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::ContainerSubject {
                    container,
                    subject: child,
                },
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineBoundaryParentAxis {
                    parent,
                    formatter: flow,
                }),
            ));
        }
    }
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
    semantic: Option<surgeist_text::PreparedSourceSegment>,
}

impl VisibleInlineRunTransition {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            semantic: None,
        }
    }
    pub(super) fn semantic(slot: &crate::text_layout::capture::NativeRunSlot) -> Self {
        Self {
            start: slot.gap,
            end: slot.gap,
            semantic: Some(slot.segment.clone()),
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
            adjoining,
            fragment_margins,
            intrinsic_inline_demand,
            baselines,
            static_positions,
            resolved_terminal_float_block_end,
            float_exclusions,
            float_intrinsics,
            pending_floats,
            owned_float_block_end,
            contributions,
        } = state;
        let adjoining_margin = adjoining.adjoining_displacement();
        context.cursor_block = adjoining.cursor;
        let constants = context.constants;
        let semantic = self.semantic.as_ref().map(|segment| {
            tree.text_computation()
                .and_then(|state| state.semantic_run(node, segment))
                .expect("semantic transition retains its validated actual binding")
        });
        let publish_native = context.set_layout;
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

        if publish_native {
            tree.publish_native_inline_run(
                node,
                self.start..self.end,
                placement.native_completion.take(),
                core::mem::take(&mut placement.native_lines),
            )?;
        }

        if let (Some(flow), Some(item)) = (formatted.as_mut(), placement.formatted.take()) {
            flow.push(item)
                .map_err(|error| text_run_error(node, error.into()))?;
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
        let placement_size = constants.flow_axes.logical_size(placement.size);
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
        // Intrinsic inline demand follows the accepted flow/fit report. Actual
        // atomic borders and visible overflow remain in the independent content
        // and scroll geometry; a negative margin does not shrink those boxes.
        *intrinsic_inline_demand = (*intrinsic_inline_demand)
            .max(placement_size.inline)
            .max(float_intrinsics.result());
        *owned_float_block_end = (*owned_float_block_end).max(placement.owned_float_block_end);
        let contribution = &placement.baselines;
        baselines
            .line(
                contribution.first,
                contribution.last,
                contribution.frame,
                contribution.context,
                contribution.first_named,
                contribution.last_named,
            )
            .map_err(|error| error.layout(node))?;
        adjoining.finish_run(
            placement.cursor_end,
            placement.margin == InlineRunMargin::Separates,
        );
        pending_floats.extend(placement.floats);
        static_positions.extend(placement.static_positions);
        if placement.margin == InlineRunMargin::Separates {
            // Contributor groups end at the same participating line as the
            // scalar adjoining component, including a line with zero extent.
            if let Some(flow) = fragment_margins {
                flow.started = true;
                flow.active = None;
            }
            *resolved_terminal_float_block_end = placement.resolved_float_terminal_block_end;
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
    pub(super) adjoining: &'a mut entry::AdjoiningFlow<S>,
    pub(super) fragment_margins: &'a mut Option<super::fragment_margins::Flow>,
    pub(super) intrinsic_inline_demand: &'a mut S,
    pub(super) baselines: &'a mut super::baseline_frame::PendingBaselines,
    pub(super) static_positions: &'a mut Vec<(Node, StaticPositionOf<S>)>,
    pub(super) resolved_terminal_float_block_end: &'a mut Option<S>,
    pub(super) float_exclusions: &'a mut FloatExclusions<S, Node>,
    pub(super) float_intrinsics: &'a mut super::floats::FloatIntrinsics<S>,
    pub(super) pending_floats: &'a mut Vec<PendingFloat<Node, S>>,
    pub(super) owned_float_block_end: &'a mut S,
    pub(super) contributions: &'a mut BlockScrollContributions<S>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InlineRunMargin {
    Adjoins,
    Separates,
}

struct InlineRunPlacement<Node, S: LayoutScalar> {
    native_completion: Option<std::sync::Arc<crate::inline_wrapping::NativeOrderingProjection>>,
    native_lines: Vec<crate::inline_wrapping::family::NativeLineReceipt<S>>,
    margin: InlineRunMargin,
    cursor_end: S,
    box_frontier: Option<crate::text_layout::boxes::BoxFrontier>,
    formatted: Option<crate::text_layout::formatted::FormattedFlowItem<Node, S>>,
    line_box_context: crate::line_box::LineBoxContext,
    lines: Vec<crate::hyphenation::CommittedInlineLine>,
    floats: Vec<PendingFloat<Node, S>>,
    owned_float_block_end: S,
    indent_state: Option<crate::text_indent::IndentLineState>,
    hyphenation_state: crate::hyphenation::LineHyphenationState,
    size: Size<S>,
    static_positions: Vec<(Node, StaticPositionOf<S>)>,
    baselines: crate::inline::PhysicalBaselineReport,
    resolved_float_terminal_block_end: Option<S>,
}

pub(super) struct InlineRunContext<'a, S: LayoutScalar> {
    pub(super) inline_request: AxisSizingRequestOf<S>,
    pub(super) child_request: crate::SizingRequestOf<S>,
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
    terminal_line_block_end: S,
    owned_float_block_end: S,
) -> Option<S> {
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
    available_inline_extent: AxisSizingRequestOf<S>,
) -> ForcedLineBreakControlOf<S> {
    ForcedLineBreakControlOf::new(
        source_index,
        InlineFlowOf::new(
            input.writing_mode(),
            input.direction(),
            available_inline_extent,
        ),
        input.metrics(),
        input.vertical_align(),
        input.clear(),
    )
}

fn inline_boundary_control<S: LayoutScalar>(
    source_index: usize,
    input: InlineBoundaryInputOf<S>,
    available_inline_extent: AxisSizingRequestOf<S>,
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
        input.vertical_align(),
        input.edge(),
        input.parent_edge_axes(),
    )
}

struct InlineRunRequest<'context, 'input, Node, S: LayoutScalar> {
    semantic: Option<crate::text_layout::AdmittedRun<'input, Node>>,
    context: InlineRunContext<'context, S>,
    adjoining_margin: S,
}

pub(super) fn inline_atomic_child_request<S: LayoutScalar>(
    constants: &Constants<S>,
    child_run_mode: RunMode,
    pass_sizing_request: crate::SizingRequestOf<S>,
    available_inline_extent: AxisSizingRequestOf<S>,
) -> ComputeInputOf<S> {
    ComputeInputOf::<S>::for_child(
        child_run_mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        constants.definite_child_containing_block_size(),
        ContainingLayoutContext::new(constants.flow_axes, ParentFormattingContext::BlockFlow),
        pass_sizing_request.with_axes(
            constants.flow_axes.physical_size(LogicalSizeOf::new(
                available_inline_extent,
                super::in_flow::in_flow_child_block_request(
                    constants
                        .flow_axes
                        .logical_size(pass_sizing_request.axes())
                        .block,
                ),
            )),
        ),
    )
    .with_containing_auto_scrollbar_pass(constants.settled_auto_scrollbars)
}

fn layout_inline_run_children<'input, Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    run: &[<Tree as Traverse>::Node],
    request: InlineRunRequest<'_, 'input, <Tree as Traverse>::Node, S>,
    float_exclusions: &mut FloatExclusions<S, <Tree as Traverse>::Node>,
    contributions: &mut BlockScrollContributions<S>,
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
        inline_request: available_inline_extent,
        child_request: pass_sizing_request,
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
    // Numeric participants establish line existence before placement. Text
    // binds its actual candidate disposition to entry geometry in the fitter.
    let uncollapsed_cursor_block = cursor_block;
    let mut cursor_block = cursor_block + adjoining_margin;
    let mut margin = InlineRunMargin::Separates;
    let logical_node_inner_size = constants.flow_axes.logical_size(node_inner_size);
    let provider_enabled = available_inline_extent.operation() == crate::SizingOperation::Normal;
    let child_run_mode = if provider_enabled {
        input.run_mode().for_child()
    } else {
        RunMode::ComputeSize
    };
    let containing_size = constants.containing_size(logical_node_inner_size);
    let mut participants = Vec::new();
    let mut atomic_children = Vec::new();
    let mut control_children = Vec::new();
    let text_run = semantic.or_else(|| {
        tree.text_computation()
            .and_then(|state| state.run(container, run))
    });
    let text_binding = text_run.as_ref().map(|run| run.binding);
    let inset_reference = tree
        .text_computation()
        .and_then(|state| state.frame(container))
        .map_or(InsetReference::Measurement(node_inner_size), |frame| {
            InsetReference::Accepted(frame.measured_inset_basis)
        });
    if text_binding.is_some() {
        // Text's actual candidate disposition owns entry consumption. Every
        // frame, source point and float query shares this unconsumed origin.
        cursor_block = uncollapsed_cursor_block;
    }
    let text_orientation = text_binding
        .map(|binding| {
            crate::text_layout::run::text_orientation(
                constants.flow_axes.writing_mode(),
                &binding.prepared,
            )
            .map_err(|error| text_run_error(container, error))
        })
        .transpose()?;
    if let Some(binding) = text_binding {
        let named = crate::text_layout::run::named_baseline(
            &binding.prepared,
            constants.flow_axes,
            constants.inline_baseline_context,
        );
        if constants.inline_baseline_context != crate::InlineBaselineContext::Physical && !named {
            return Err(LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(container),
                LayoutOperation::TextCapture,
                LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::NamedInlineBaseline),
            ));
        }
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
    let mut float_events = super::float_text::FloatEvents::new();
    let mut encounter_plans = Vec::new();
    let mut static_positions = Vec::new();
    let mut absolute_children = Vec::new();
    for (offset, child) in run.iter().copied().enumerate() {
        let source_index = source_index_start + offset;
        let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
        let child_style = match projection.kind() {
            InlineParticipantKindOf::InlineText(_) => continue,
            InlineParticipantKindOf::TextControl(_) => {
                control_children.push((child, source_index, None));
                continue;
            }
            InlineParticipantKindOf::Box(style) => style,
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
                control_children.push((child, source_index, Some(line_break.vertical_align())));
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
                control_children.push((child, source_index, Some(boundary.vertical_align())));
                continue;
            }
        };
        if child_style.display == crate::Display::None {
            if set_layout {
                tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                    child,
                    crate::output::NormalGeometryOf::metadata(
                        crate::Point::ZERO,
                        crate::Size::ZERO,
                    ),
                    NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
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
        let physical_demand = provider_enabled
            && child_style.float.is_none()
            && constants.inline_baseline_context == crate::InlineBaselineContext::Physical
            && uses_inner_physical_atomic_baseline(&child_style, constants.flow_axes);
        let mut child_request = inline_atomic_child_request(
            constants,
            child_run_mode,
            pass_sizing_request,
            available_inline_extent,
        )
        .with_named_baseline_demand(named_demand)
        .with_physical_baseline_demand(physical_demand);
        let packet = if !child_style.float.is_none() && text_binding.is_some() {
            // Use the ordinary block owner's complete float child request. The
            // paragraph capture changes encounter scheduling, not child sizing.
            let axes = child_style.common.flow_axes;
            let parent = constants.child_containing_block_size(axes);
            let logical_parent = axes.logical_size(parent);
            let available = axes.logical_size(pass_sizing_request.axes());
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
                .or(available
                    .inline
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get))
                .map(|inline| {
                    crate::layout_math::remaining_available_extent(inline, margin.inline_sum())
                })
                .transpose()
                .map_err(|error| crate::error::sizing_resolution_error(child, error.into()))?;
            let known = super::in_flow::in_flow_child_known_size::<Tree, M>(
                tree,
                child,
                &child_style,
                child_padding + child_border,
                axes,
                logical_parent,
                (available.inline.operation() == crate::SizingOperation::Normal)
                    .then_some(inline)
                    .flatten(),
            )?;
            child_request = ComputeInputOf::<S>::for_child(
                child_run_mode,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                known,
                parent,
                ContainingLayoutContext::new(
                    constants.flow_axes,
                    ParentFormattingContext::BlockFlow,
                ),
                pass_sizing_request.with_axes(
                    axes.physical_size(LogicalSizeOf::new(
                        super::in_flow::in_flow_child_inline_request(
                            &child_style,
                            axes,
                            inline,
                            available.inline,
                        )
                        .map_err(|error| {
                            crate::error::sizing_resolution_error(child, error.into())
                        })?,
                        super::in_flow::in_flow_child_block_request(available.block),
                    )),
                ),
            )
            .with_containing_auto_scrollbar_pass(input.settled_auto_scrollbars())
            .with_block_indent_context(crate::text_indent::BlockIndentContext {
                inherited: constants.text_indent,
                first_formatted: indent_state.first_formatted() && constants.first_formatted,
            })
            .with_named_baseline_demand(named_demand);
            Some(tree.probe_float_child(
                child,
                child_request,
                crate::engine::FloatChildTrial::Detached,
            )?)
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
            let state = tree
                .text_computation()
                .expect("captured float keeps source registry");
            float_events
                .push(
                    super::float_text::FloatEvent::new(
                        pending,
                        packet,
                        text_run
                            .as_ref()
                            .expect("actual float is in captured native run"),
                        &state.associations,
                        offset,
                        child_request,
                        state.entry_case_funding().map(|funding| funding.as_ref()),
                    )
                    .map_err(|e| text_run_error(container, e))?,
                    state.entry_case_funding().map(|funding| funding.as_ref()),
                )
                .map_err(|error| text_run_error(container, error))?;
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
            let relative_offset = relative::resolve(
                tree,
                child,
                *child_style.common.inset,
                inset_reference,
                constants.flow_axes,
            )?;
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
    let mut text_progress = None;
    let mut formatted_item = None;
    let mut report = if let Some(binding) = text_binding {
        let (frame, frames, reusable, associations, scope_alignments, entry_funding) = {
            let state = tree
                .text_computation()
                .expect("capture-backed branch retains its session state");
            (
                state.frame(container),
                state.frames.clone(),
                state.reusable.clone(),
                state.associations.clone(),
                state.scope_resources.clone(),
                state.entry_case_funding().cloned(),
            )
        };
        let canonical_exclusions = float_exclusions
            .canonical_for_text(container, &frames)
            .map_err(|error| text_run_error(container, error))?;
        let exclusions = canonical_exclusions.as_ref().unwrap_or(float_exclusions);
        let actual_content = frame.and_then(|frame| frame.normal.content_box());
        let content_insets =
            frame
                .zip(actual_content)
                .map_or(constants.content_box_inset, |(frame, rect)| {
                    Edges::new(
                        rect.origin().y,
                        frame.normal.size.width - rect.origin().x - rect.size().width,
                        frame.normal.size.height - rect.origin().y - rect.size().height,
                        rect.origin().x,
                    )
                });
        let available =
            if crate::layout_math::normal_available_size(available_inline_extent).is_some() {
                crate::layout_math::with_available_extent(
                    available_inline_extent,
                    actual_content.map(|rect| constants.flow_axes.logical_size(rect.size()).inline),
                )
                .map_err(|error| crate::error::sizing_resolution_error(container, error.into()))?
            } else {
                // A measured intrinsic result does not turn into a used constraint.
                available_inline_extent
            };
        let formatted_mode = crate::layout_math::text_axis_request(available);
        let formatted = tree
            .text_computation()
            .map(|state| {
                state.formatted_snapshot(
                    container,
                    run,
                    &text_run
                        .as_ref()
                        .expect("actual native source segment")
                        .segment,
                    input,
                    formatted_mode,
                )
            })
            .transpose()
            .map_err(|error| text_run_error(container, error.into()))?
            .unwrap_or_default();
        let group_snapshot = tree
            .group_computation()
            .map(|groups| groups.snapshot())
            .unwrap_or_default();
        let actual_frame = if let Some(frame) = frame {
            let visual_content = frame
                .output
                .scroll_geometry
                .map(|geometry| geometry.content_box())
                .ok_or_else(|| {
                    text_run_error(
                        container,
                        crate::text_layout::fit::FitError::InvalidGeometry,
                    )
                })?;
            if frame.normal.content_box().is_none() {
                return Err(text_run_error(
                    container,
                    crate::text_layout::fit::FitError::InvalidGeometry,
                ));
            }
            crate::text_layout::run::Frame::Canonical {
                normal: crate::text_layout::run::CanonicalLineFrame {
                    origin: frame.normal_origin,
                    size: frame.normal.size,
                    content_insets,
                },
                visual: crate::text_layout::run::CanonicalLineFrame {
                    origin: frame.origin,
                    size: frame.output.size,
                    content_insets: Edges::new(
                        visual_content.origin().y,
                        frame.output.size.width
                            - visual_content.origin().x
                            - visual_content.size().width,
                        frame.output.size.height
                            - visual_content.origin().y
                            - visual_content.size().height,
                        visual_content.origin().x,
                    ),
                },
            }
        } else {
            crate::text_layout::run::Frame::Measurement
        };
        let context = crate::text_layout::run::Context {
            scope_alignments,
            entry_funding: entry_funding.clone(),
            orientation: text_orientation
                .expect("the actual Text binding has its resolved orientation"),
            inline_baseline_context: constants.inline_baseline_context,
            groups: group_snapshot,
            wrapping: tree.wrapping_context().and_then(|(context, graph)| {
                crate::text_layout::run::WrappingProjection::new(
                    context,
                    graph,
                    text_run.as_ref().expect("actual admitted native run"),
                    &associations,
                )
                .map(|projection| projection.with_incoming(box_frontier.clone()))
            }),
            formatted,
            frame: actual_frame,
            containing_size,
            content_insets,
            cursor_block,
            adjoining_displacement: adjoining_margin,
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
                    .map(|endpoint| BoundaryAllocationContext {
                        container,
                        prepared: &text_binding.expect("the actual Text branch retains its binding").prepared,
                        associations: &associations,
                        resources: &context.scope_alignments,
                        orientation: context.orientation,
                        axes: constants.flow_axes,
                    }.allocate(tree, node, &binding.occurrence, endpoint))
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
                                crate::VerticalAlignOf::Baseline => {
                                    surgeist_text::LineBoxAlignment::Baseline
                                }
                                crate::VerticalAlignOf::Top => surgeist_text::LineBoxAlignment::Top,
                                crate::VerticalAlignOf::Bottom => {
                                    surgeist_text::LineBoxAlignment::Bottom
                                }
                                crate::VerticalAlignOf::Shift(offset) => {
                                    surgeist_text::LineBoxAlignment::Shift(
                                        surgeist_text::BaselineShift::try_new(offset.get().to_f64())
                                            .expect("a finite Layout scalar widens to finite Text coordinates"),
                                    )
                                }
                            },
                        }),
                    },
                })
            })
            .collect::<Result<Vec<_>, LayoutErrorOf<Tree::Node, S, M>>>()?;
        let intrinsic_max = !float_events.is_empty()
            && (available.operation() == crate::SizingOperation::MaxContent);
        let quality_replay = matches!(
            constants.wrap_style,
            crate::TextWrapStyle::Balance
                | crate::TextWrapStyle::AvoidShortLastLine
                | crate::TextWrapStyle::Pretty
        ) && crate::layout_math::normal_available_size(available).is_some();
        let admitted = text_run.as_ref().expect("actual admitted native run");
        let funding = entry_funding.as_ref().map(|funding| funding.as_ref());
        let bound_events = float_events
            .bind(
                admitted,
                &associations,
                text_controls.iter().map(|control| &control.allocation),
                |owner| {
                    let (context, graph) = tree
                        .wrapping_context()
                        .ok_or(crate::WrappingScopeError::MissingCapture)?;
                    super::float_text::source_interval(
                        admitted,
                        &associations,
                        context,
                        graph,
                        owner,
                        funding,
                    )
                },
                funding,
            )
            .map_err(|error| text_run_error(container, error))?;
        let encounters = std::cell::RefCell::new(
            super::float_text::Encounters::new(
                bound_events,
                exclusions
                    .try_clone()
                    .map_err(|error| text_run_error(container, error.into()))?,
                cursor_block,
                if intrinsic_max {
                    super::floats::FloatInlineSpace::IntrinsicMaximum
                } else {
                    super::floats::FloatInlineSpace::Finite
                },
                quality_replay
                    || tail == crate::text_layout::fit::TailContext::CompleteBlock
                        && crate::layout_math::normal_available_size(available).is_some(),
                quality_replay,
            )
            .map_err(|error| text_run_error(container, error))?,
        );
        let result = crate::text_layout::run::layout(
            text_run.as_ref().expect("actual admitted native run"),
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
                    let band = if provider_enabled {
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
                            .query_rectangular_line_band(start, end)?
                    };
                    let inset_inline = constants
                        .flow_axes
                        .logical_edges(content_insets)
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
                    .replay(
                        container,
                        line,
                        replay,
                        entry_funding.as_ref().map(|funding| funding.as_ref()),
                    ) {
                    Ok(basis) => Ok(basis),
                    Err(error) => {
                        *provider_error.borrow_mut() = Some(error);
                        Err(crate::text_layout::fit::FitError::Dependency)
                    }
                },
                accept: || encounters.borrow_mut().accept(),
            },
        );
        if let Some(error) = provider_error.borrow_mut().take() {
            return Err(error);
        }
        let mut output = result.map_err(|error| text_run_error(container, error))?;
        formatted_item = tree
            .text_computation()
            .map(|state| {
                state.formatted_merge_run(
                    container,
                    run,
                    &text_run
                        .as_ref()
                        .expect("actual native source segment")
                        .segment,
                    input,
                    formatted_mode,
                    core::mem::take(&mut output.formatted),
                )
            })
            .transpose()
            .map_err(|error| text_run_error(container, error.into()))?
            .flatten();
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
        text_progress = Some(output.progress);
        margin = if output.progress.phase == crate::text_layout::fit::EntryPhase::Consumed {
            InlineRunMargin::Separates
        } else {
            InlineRunMargin::Adjoins
        };
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
                        .numeric_projection(
                            graph,
                            container,
                            crate::inline_wrapping::NativeRunSource {
                                nodes: run,
                                start: source_index_start,
                            },
                            &nodes,
                        )
                        .map(|projection| {
                            if available_inline_extent.operation()
                                == crate::SizingOperation::MinContent
                            {
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
        let observe_formatted =
            matches!(
                constants.wrap_style,
                crate::TextWrapStyle::Balance
                    | crate::TextWrapStyle::AvoidShortLastLine
                    | crate::TextWrapStyle::Pretty
            ) && crate::layout_math::normal_available_size(available_inline_extent).is_some()
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
                if let Some(error) = provider_error.borrow_mut().take() {
                    return Err(error);
                }
                let query_block_start = cursor_block + block_start;
                let query_block_end = cursor_block + block_end;
                let query = if !provider_enabled {
                    float_exclusions
                        .query_rectangular_line_band(query_block_start, query_block_end)
                        .map_err(|error| text_run_error(container, error.into()))
                } else {
                    float_exclusions.query_provider_band(
                        &**numeric_tree.borrow(),
                        container,
                        query_block_start,
                        query_block_end,
                    )
                };
                let band = query?;
                Ok(LogicalLineBandQueryResultOf {
                    inline_start: band.inline_start - logical_content_box_inset.inline_start,
                    inline_end: band.inline_end - logical_content_box_inset.inline_start,
                    next_transition: band
                        .next_transition
                        .map(|transition| transition - cursor_block),
                })
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
        result.map_err(|error| match error {
            crate::inline::NumericRunError::Numeric(error) => {
                crate::inline_wrapping::numeric_wrapping_error(container, error)
            }
            crate::inline::NumericRunError::Band(error) => error,
        })?
    };
    // Native outflows without a Text occurrence use the accepted run entry.
    // Their static position consumes adjoining displacement only with an actual
    // formatted line; Text-owned atomic slots below retain their own authority.
    let static_entry = if text_progress
        .as_ref()
        .is_some_and(|progress| progress.phase == crate::text_layout::fit::EntryPhase::Consumed)
    {
        cursor_block + adjoining_margin
    } else {
        cursor_block
    };
    for child in absolute_children {
        static_positions.push((
            child,
            absolute_static_position(
                static_entry,
                constants,
                containing_size,
                StaticPositionGeometry::Point,
            ),
        ));
    }
    if text_output.is_none() {
        if let Some(lines) = &report.formatted_lines {
            let funding = tree
                .text_computation()
                .and_then(|state| state.entry_case_funding().cloned());
            formatted_item = Some(
                crate::text_layout::formatted::FormattedFlowItem::numeric(
                    run,
                    lines,
                    funding.as_ref(),
                )
                .map_err(|error| text_run_error(container, error.into()))?,
            );
        }
        report.hyphenation_state = hyphenation_state;
        if crate::layout_math::normal_available_size(available_inline_extent).is_some()
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
    // Retain the accepted signed cursor, not max(0, geometric extent). The
    // numeric association matches the original entered-cursor-plus-height fold.
    let cursor_end = match text_progress {
        Some(progress) => progress.cursor_end,
        None => cursor_block + report.block_extent,
    };
    let resolved_float_terminal_block_end =
        resolved_inline_float_terminal_block_end(&report, cursor_end, owned_float_block_end);
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
    let project_source =
        |position: InlineSourcePositionOf<S>, size: LogicalSizeOf<S>| match position {
            InlineSourcePositionOf::Logical(point) => {
                project_point(point.inline, point.block, size)
            }
            InlineSourcePositionOf::Physical { visual, .. } => visual,
        };

    let project_normal_source =
        |position: InlineSourcePositionOf<S>, size: LogicalSizeOf<S>| match position {
            InlineSourcePositionOf::Logical(point) => {
                project_point(point.inline, point.block, size)
            }
            InlineSourcePositionOf::Physical { normal, .. } => normal,
        };

    // Numeric geometry has no grid conversion. Validate every newly consumed
    // numeric displacement before publishing output, using retained input loans.
    // Captured Text owns its separate checked, once-applied grid conversion.
    let atomic_sources = report
        .atomics
        .iter()
        .map(|source| (source.item.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let control_sources = report
        .controls
        .iter()
        .map(|source| (source.source_index, *source))
        .collect::<BTreeMap<_, _>>();
    let mut shifted_atomic_locations = BTreeMap::new();
    let mut shifted_control_locations = BTreeMap::new();
    if text_binding.is_none() {
        for child in &atomic_children {
            let source = atomic_sources[&child.source_index];
            if !matches!(source.item.alignment, VerticalAlignOf::Shift(offset) if offset.get() != S::ZERO)
            {
                continue;
            }
            let InlineParticipantKindOf::Box(projection) = child.projection.kind() else {
                unreachable!("retained atomic projection is a box");
            };
            let inset = atomic_inset_offset::<Tree, S, M>(
                tree,
                child.node,
                &projection,
                constants.flow_axes,
                node_inner_size,
                text_binding.is_some(),
            )?;
            let logical_size = constants.flow_axes.logical_size(source.item.size);
            let InlineSourcePositionOf::Logical(position) = source.position else {
                unreachable!("the numeric fitter emits logical positions");
            };
            let location = checked_shift_projection(
                source.item.alignment,
                constants.flow_axes,
                position.block,
                |block| {
                    let point = project_point(position.inline, block, logical_size);
                    Point::new(point.x + inset.x, point.y + inset.y)
                },
            )
            .map_err(|error| error.layout::<Tree::Node, S, M>(child.node))?;
            shifted_atomic_locations.insert(child.source_index, location);
        }
        for (child, source_index, alignment) in &control_children {
            let Some(alignment) = alignment else {
                continue;
            };
            if !matches!(alignment, VerticalAlignOf::Shift(offset) if offset.get() != S::ZERO) {
                continue;
            }
            let Some(source) = control_sources.get(source_index) else {
                continue;
            };
            let size = LogicalSizeOf::new(source.inline_extent, S::ZERO);
            let InlineSourcePositionOf::Logical(position) = source.position else {
                unreachable!("the numeric fitter emits logical positions");
            };
            let location = checked_shift_projection(
                *alignment,
                constants.flow_axes,
                position.block,
                |block| project_point(position.inline, block, size),
            )
            .map_err(|error| error.layout::<Tree::Node, S, M>(*child))?;
            shifted_control_locations.insert(*source_index, location);
        }
    }

    if set_layout && text_output.is_none() {
        for band in &report.line_bands {
            let size = LogicalSizeOf::new(
                band.inline_end - band.inline_start,
                band.block_end - band.block_start,
            );
            let rect = crate::ScrollRectOf::try_new(
                project_point(band.inline_start, band.block_start, size),
                constants.flow_axes.physical_size(size),
            )
            .map_err(|error| layout_own_geometry_error(container, input.run_mode(), error))?;
            contributions
                .include_line(rect)
                .map_err(|error| layout_own_geometry_error(container, input.run_mode(), error))?;
        }
    }
    if let Some(output) = text_output {
        if let Some(state) = tree.text_computation() {
            state.record_boxes(crate::text_layout::boxes::BoxRunUse {
                container,
                prepared: output.paragraph.prepared.clone(),
                nodes: run.to_vec(),
                input,
                available: crate::layout_math::text_axis_request(available_inline_extent),
                trials: output.box_uses.clone(),
            });
        }
        if let Some(Some(assignment)) = output.paragraph.boxes.last() {
            box_frontier = Some(assignment.outgoing.clone());
        }
        if set_layout {
            // Only the actual line allocation supplies terminal padding edges.
            // Placed metrics and source fragments remain independent overflow.
            for bounds in &output.line_bounds {
                contributions
                    .include_line(bounds.allocation)
                    .map_err(|error| {
                        layout_own_geometry_error(container, input.run_mode(), error)
                    })?;
                contributions.geometry.include_direct_line(bounds.metrics);
            }
            for rect in output
                .fragments
                .iter()
                .filter_map(|source| source.fragment.rect())
            {
                contributions.geometry.include_direct_line(rect);
            }
        }
        let mut groups: Vec<TextFragmentGroup<Tree::Node, S>> = Vec::new();
        let mut scope_fragments = Vec::new();
        for crate::text_layout::run::SourceFragment {
            owner,
            normal_rect,
            fragment,
        } in output.fragments
        {
            let crate::InlineFragmentOwner::Node(node) = owner else {
                scope_fragments.push(crate::InlineFragmentOutputEntryOf::new(owner, fragment));
                continue;
            };
            let index = groups
                .iter()
                .position(|(subject, _, _, _, _, _)| *subject == node);
            let index = index.unwrap_or_else(|| {
                groups.push((node, Vec::new(), None, None, None, None));
                groups.len() - 1
            });
            let (_, fragments, minimum, maximum, normal_minimum, normal_maximum) =
                &mut groups[index];
            if let Some(rect) = normal_rect {
                let location = rect.origin();
                let end = Point::new(
                    location.x + rect.size().width,
                    location.y + rect.size().height,
                );
                *normal_minimum = Some(normal_minimum.map_or(location, |p| {
                    Point::new(p.x.min(location.x), p.y.min(location.y))
                }));
                *normal_maximum = Some(
                    normal_maximum.map_or(end, |p| Point::new(p.x.max(end.x), p.y.max(end.y))),
                );
            }
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
            for (child, normal_location, location) in output.nonformatted_controls {
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
                tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                    child,
                    crate::output::NormalGeometryOf::metadata(normal_location, crate::Size::ZERO),
                    NodeOutputOf::<S> {
                        source_index: crate::SourceIndex::new(source_index),
                        location,
                        border,
                        padding,
                        ..NodeOutputOf::new()
                    },
                ));
            }
        }
        if set_layout {
            for row in text_binding
                .into_iter()
                .flat_map(|binding| binding.origins.iter())
                .filter(|row| run.contains(&row.node))
            {
                if !groups.iter().any(|(node, _, _, _, _, _)| *node == row.node) {
                    groups.push((row.node, Vec::new(), None, None, None, None));
                }
            }
            for (node, fragments, minimum, maximum, normal_minimum, normal_maximum) in groups {
                let (normal_location, normal_size) = match (normal_minimum, normal_maximum) {
                    (Some(start), Some(end)) => {
                        (start, Size::new(end.x - start.x, end.y - start.y))
                    }
                    _ => (Point::ZERO, Size::ZERO),
                };
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
                tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                    node,
                    crate::output::NormalGeometryOf::metadata(normal_location, normal_size),
                    NodeOutputOf {
                        source_index: crate::SourceIndex::new(source_index),
                        location,
                        size,
                        content_size: size,
                        ..NodeOutputOf::new()
                    },
                ));
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
            state.record(output.paragraph);
        }
    }

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
        let InlineParticipantKindOf::Box(child_projection) = child_style.kind() else {
            unreachable!("atomic child retains its settled box projection");
        };
        let normal_location = project_normal_source(source.position, logical_size);
        let location = if let Some(location) = shifted_atomic_locations.get(&source_index) {
            *location
        } else {
            let point = project_source(source.position, logical_size);
            let inset = atomic_inset_offset::<Tree, S, M>(
                tree,
                child,
                &child_projection,
                constants.flow_axes,
                node_inner_size,
                text_binding.is_some(),
            )?;
            Point::new(point.x + inset.x, point.y + inset.y)
        };
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
            if text_binding.is_some()
                || !relative::needs_basis(*child_projection.common.inset, constants.flow_axes)
            {
                contributions
                    .include_in_flow_geometry(location, source.item.margin, scroll_geometry)
                    .map_err(|error| layout_child_geometry_error(container, child, error))?;
            }
            let named_baselines = match source.canonical_geometry {
                Some(canonical) => crate::text_layout::run::pending_visual_named_baselines(
                    output.named_baselines,
                    canonical.visual_readback_offset,
                )
                .map_err(|error| text_run_error(container, error))?,
                None => output.named_baselines,
            };
            let visual = NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(source_index),
                location,
                size: source.item.size,
                named_baselines,
                content_size: source.item.content_size,
                border: source.item.border,
                padding: source.item.padding,
                margin: source.item.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry));
            let entry = match source.canonical_geometry {
                Some(canonical) => crate::LayoutOutputEntryOf::canonical_atomic(
                    child,
                    canonical.measured_normal,
                    canonical.normal,
                    canonical.normal_origin,
                    canonical.visual_origin,
                    visual,
                ),
                None => crate::LayoutOutputEntryOf::ordinary_unrounded(
                    child,
                    crate::output::NormalGeometryOf::box_geometry(
                        normal_location,
                        source.item.size,
                        scroll_geometry.content_box(),
                    ),
                    visual,
                ),
            };
            tree.set_unrounded(entry);
        }
    }

    if set_layout {
        let nonformatted_controls = report
            .nonformatted_controls
            .iter()
            .map(|source| (source.control.source_index(), source))
            .collect::<BTreeMap<_, _>>();
        for (child, source_index, _) in control_children {
            let (position, inline_extent, endpoint, margin_flow, formatted) =
                if let Some(source) = control_sources.get(&source_index) {
                    (
                        source.position,
                        source.inline_extent,
                        source.endpoint,
                        source.margin_flow,
                        true,
                    )
                } else if let Some(source) = nonformatted_controls.get(&source_index) {
                    (
                        InlineSourcePositionOf::Logical(LogicalPointOf::new(
                            source.inline_start,
                            source.block_start,
                        )),
                        source.control.edge().border_padding_advance(),
                        Some(crate::inline::InlineEndpointOf::from_control(
                            source.control,
                        )),
                        None,
                        false,
                    )
                } else {
                    continue;
                };
            let logical_size = LogicalSizeOf::new(inline_extent, S::ZERO);
            let size = constants.flow_axes.physical_size(logical_size);
            let (border, padding) = endpoint.map_or((Edges::ZERO, Edges::ZERO), |endpoint| {
                endpoint.physical_components()
            });
            let normal_location = project_normal_source(position, logical_size);
            let location = shifted_control_locations
                .get(&source_index)
                .copied()
                .unwrap_or_else(|| project_source(position, logical_size));
            if let Some((before_flow, after_flow)) = margin_flow {
                let InlineSourcePositionOf::Logical(point) = position else {
                    unreachable!("numeric margin flow has the same logical phase as its strip");
                };
                let endpoint = endpoint.expect("numeric margin belongs to an endpoint");
                let flow_point = |inline| {
                    project_source(
                        InlineSourcePositionOf::Logical(LogicalPointOf::new(inline, point.block)),
                        LogicalSizeOf::new(S::ZERO, S::ZERO),
                    )
                };
                let before_flow = flow_point(before_flow);
                let after_flow = flow_point(after_flow);
                let positive =
                    FlowAxes::new(constants.flow_axes.writing_mode(), crate::Direction::Ltr)
                        .inline_start();
                let side = match endpoint.kind {
                    crate::InlineBoundaryKind::Start => endpoint.axes.inline_start(),
                    crate::InlineBoundaryKind::End => endpoint.axes.inline_end(),
                };
                let (before, after) = match (positive, side == positive) {
                    (crate::PhysicalSide::Left, true) => (before_flow.x, location.x),
                    (crate::PhysicalSide::Left, false) => (location.x + size.width, after_flow.x),
                    (crate::PhysicalSide::Top, true) => (before_flow.y, location.y),
                    (crate::PhysicalSide::Top, false) => (location.y + size.height, after_flow.y),
                    (crate::PhysicalSide::Bottom, true) => {
                        (before_flow.y, location.y + size.height)
                    }
                    (crate::PhysicalSide::Bottom, false) => (location.y, after_flow.y),
                    _ => unreachable!("the forward native inline basis is left, top or bottom"),
                };
                if !before.is_finite() || !after.is_finite() {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(child),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::TextNumeric {
                            value: if !after.is_finite() {
                                after.to_f64()
                            } else {
                                before.to_f64()
                            },
                            reason: crate::LayoutTextNumericError::NonFinite,
                        },
                    ));
                }
                if before == after {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(child),
                        LayoutOperation::ChildLayout,
                        LayoutErrorKindOf::TextNumeric {
                            value: endpoint.edge.margin().to_f64(),
                            reason: crate::LayoutTextNumericError::NonZeroLost,
                        },
                    ));
                }
            }
            if formatted {
                contributions
                    .include_inline_strip(
                        location,
                        size,
                        endpoint.map_or(
                            Edges::ZERO,
                            crate::inline::InlineEndpointOf::physical_margin,
                        ),
                    )
                    .map_err(|error| layout_child_geometry_error(container, child, error))?;
            }
            tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                child,
                crate::output::NormalGeometryOf::metadata(normal_location, size),
                NodeOutputOf::<S> {
                    source_index: crate::SourceIndex::new(source_index),
                    location,
                    size,
                    border,
                    padding,
                    margin: endpoint.map_or(
                        Edges::ZERO,
                        crate::inline::InlineEndpointOf::physical_margin,
                    ),
                    ..NodeOutputOf::new()
                },
            ));
        }
    }

    let baselines = match report.baselines {
        crate::inline::InlineBaselineReportOf::Native(physical) => physical,
        crate::inline::InlineBaselineReportOf::Numeric(numeric) => {
            // Numeric line facts are accepted run-local logical S values.
            // Project them once in f64, retaining this parent's actual used
            // frame until its final border-box extent is known.
            let project = |baseline: S| {
                let block =
                    crate::group_alignment::add::<f64>(cursor_block.to_f64(), baseline.to_f64())?;
                crate::group_alignment::add::<f64>(baseline.to_f64(), cursor_block.to_f64())?;
                match constants.flow_axes.block_start() {
                    PhysicalSide::Top | PhysicalSide::Left => Ok(block),
                    PhysicalSide::Right | PhysicalSide::Bottom => {
                        crate::group_alignment::add::<f64>(
                            -block,
                            constants
                                .flow_axes
                                .logical_size(containing_size)
                                .block
                                .to_f64(),
                        )?;
                        crate::group_alignment::add::<f64>(
                            constants
                                .flow_axes
                                .logical_size(containing_size)
                                .block
                                .to_f64(),
                            -block,
                        )
                    }
                }
            };
            crate::inline::PhysicalBaselineReport {
                context: numeric.context,
                first: numeric
                    .first
                    .map(project)
                    .transpose()
                    .map_err(|error| error.layout(container))?,
                last: numeric
                    .last
                    .map(project)
                    .transpose()
                    .map_err(|error| error.layout(container))?,
                first_named: numeric.first_named,
                last_named: numeric.last_named,
                frame: super::baseline_frame::BaselineFrame::Measured {
                    used_size: containing_size.map(LayoutScalar::to_f64),
                    axes: constants.flow_axes,
                },
            }
        }
    };

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
        native_completion: report.native_completion,
        native_lines: report.native_lines,
        margin,
        cursor_end,
        box_frontier,
        formatted: formatted_item,
        line_box_context: report.line_box_context,
        lines: report.lines,
        floats: encounter_plans,
        owned_float_block_end: float_end,
        indent_state: (!report.line_bands.is_empty()).then_some(report.indent_state),
        hyphenation_state: report.hyphenation_state,
        size: report_size,
        static_positions,
        baselines,
        resolved_float_terminal_block_end,
    })
}

struct AtomicInlineContext {
    axes: crate::FlowAxes,
    baseline: crate::InlineBaselineContext,
}

fn atomic_inset_offset<'input, Tree, S, M>(
    tree: &Tree,
    child: Tree::Node,
    projection: &BlockChildProjection<'_, S>,
    axes: FlowAxes,
    inner_size: Size<Option<S>>,
    captured: bool,
) -> LayoutResultOf<Tree::Node, Point<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    if captured || relative::needs_basis(*projection.common.inset, axes) {
        // Captured offsets are sealed by the real converter. Native percentage
        // offsets are deferred until this parent's performed normal size exists.
        return Ok(Point::ZERO);
    }
    relative::resolve(
        tree,
        child,
        *projection.common.inset,
        InsetReference::Measurement(inner_size),
        axes,
    )
}

/// Compare the actual post-match displacement inside the same retained frame.
/// Reconstructing the matched point does not refit or renormalize the line.
fn checked_shift_projection<S: LayoutScalar>(
    alignment: VerticalAlignOf<S>,
    axes: FlowAxes,
    block: S,
    project: impl Fn(S) -> Point<S>,
) -> Result<Point<S>, crate::group_alignment::NumericError> {
    let actual = project(block);
    if let VerticalAlignOf::Shift(offset) = alignment
        && offset.get() != S::ZERO
    {
        let inverse = if axes.line_over() == axes.block_start() {
            offset.get()
        } else {
            -offset.get()
        };
        let matched = project(crate::baseline_offset::displaced(block, inverse)?);
        let before = axes.block_axis_coordinate(matched);
        let after = axes.block_axis_coordinate(actual);
        let delta = match axes.line_over() {
            PhysicalSide::Top | PhysicalSide::Left => -offset.get(),
            PhysicalSide::Right | PhysicalSide::Bottom => offset.get(),
        };
        crate::baseline_offset::displaced(before, delta)?;
        if !after.is_finite() {
            return Err(crate::group_alignment::NumericError::new(
                after.to_f64(),
                crate::LayoutTextNumericError::NonFinite,
            ));
        }
        if after == before {
            return Err(crate::group_alignment::NumericError::new(
                offset.get().to_f64(),
                crate::LayoutTextNumericError::NonZeroLost,
            ));
        }
    }
    Ok(actual)
}

fn named_atomic_demand<S: LayoutScalar>(
    child: &BlockChildProjection<'_, S>,
    context: crate::InlineBaselineContext,
) -> crate::output::NamedBaselineDemand {
    match context {
        crate::InlineBaselineContext::Named(name)
            if child.vertical_align.is_baseline_relative() && child.float.is_none() =>
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

fn uses_inner_physical_atomic_baseline<S: LayoutScalar>(
    child: &BlockChildProjection<'_, S>,
    containing: FlowAxes,
) -> bool {
    let used_overflow =
        UsedOverflow::from_computed(child.common.overflow, child.common.item_is_replaced);
    let block_overflow = match containing.block_axis() {
        PhysicalAxis::Horizontal => used_overflow.x(),
        PhysicalAxis::Vertical => used_overflow.y(),
    };
    child.vertical_align.is_baseline_relative() && block_overflow.value() == Overflow::Visible
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
    let mut selected_inner_baseline =
        uses_inner_physical_atomic_baseline(&child_style, containing_flow_axes)
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
        && child_style.vertical_align.is_baseline_relative()
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
        baseline_reference: if child_style.vertical_align.is_baseline_relative() {
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
        alignment: child_style.vertical_align,
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
            VerticalAlignOf::Top | VerticalAlignOf::Bottom
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

/// One source-backed endpoint allocation shared by actual fitting and entry
/// semantics. Struts and metrics are deliberately outside this operation.
pub(super) struct BoundaryAllocationContext<'a, Node> {
    pub(super) container: Node,
    pub(super) prepared: &'a surgeist_text::PreparedText,
    pub(super) associations: &'a crate::text_layout::capture::ScopeAssociations<Node>,
    pub(super) resources: &'a dyn crate::text_layout::scope_alignment::ScopeAlignmentFacts,
    pub(super) orientation: surgeist_text::LineOrientation,
    pub(super) axes: crate::FlowAxes,
}
impl<Node: Copy + Eq> BoundaryAllocationContext<'_, Node> {
    pub(super) fn allocate<'input, Tree, S, M>(
        &self,
        tree: &Tree,
        node: Node,
        occurrence: &surgeist_text::PreparedControl,
        endpoint: crate::inline::InlineEndpointOf<S>,
    ) -> LayoutResultOf<Node, surgeist_text::InlineBoundaryAllocation, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
        S: LayoutScalar,
    {
        let advance = endpoint.edge.border_padding_advance().to_f64();
        let crate::LayoutInputViewOf::InlineBoundary(input) = tree.layout_input(node) else {
            unreachable!("an endpoint retains its normalized input");
        };
        let Some(parent_axes) = input.parent_edge_axes() else {
            return surgeist_text::InlineBoundaryAllocation::contribution(advance)
                .map_err(|error| text_run_error(self.container, error.into()));
        };
        let limits = occurrence
            .captured_control()
            .and_then(|control| control.scope().scope_alignment_limits());
        let inspect = || {
            if let Some(limits) = limits {
                crate::text_layout::scope_alignment::ScopeAlignmentFacts::visit(
                    self.resources,
                    limits,
                    1,
                )?;
            }
            Ok::<_, surgeist_text::Error>(())
        };
        let fallback = self
            .associations
            .box_for_endpoint(node, inspect)
            .map_err(|error| text_run_error(self.container, error.into()))?;
        let (wrapping, _) = tree.wrapping_context().ok_or_else(|| {
            crate::inline_wrapping::wrapping_error(
                self.container,
                crate::WrappingScopeError::MissingCapture,
            )
        })?;
        let (owner, parent) = wrapping
            .text_box_edge_sources(
                self.container,
                node,
                fallback.as_ref(),
                self.prepared,
                inspect,
            )
            .map_err(|error| text_run_error(self.container, error.into()))?
            .ok_or_else(|| {
                crate::inline_wrapping::wrapping_error(
                    self.container,
                    crate::WrappingScopeError::MissingCapture,
                )
            })?;
        if let Some(scope) = parent
            .captured_scope()
            .filter(|scope| scope.role() == surgeist_text::ScopeRole::Domain)
        {
            if let Some(limits) = scope.scope_alignment_limits() {
                crate::text_layout::scope_alignment::ScopeAlignmentFacts::visit(
                    self.resources,
                    limits,
                    self.associations.captured_binding_work(scope),
                )
                .map_err(|error| text_run_error(self.container, error.into()))?;
            }
            if let Some(crate::TextScopeAssociation::ParagraphContainer(parent_node)) =
                self.associations.captured_binding(scope)
            {
                let parent_input = tree.layout_input(parent_node).as_box().ok_or_else(|| {
                    crate::inline_wrapping::wrapping_error(
                        parent_node,
                        crate::WrappingScopeError::CaptureAssociation,
                    )
                })?;
                let expected =
                    crate::FlowAxes::new(parent_input.writing_mode, parent_input.direction);
                if parent_axes != expected {
                    return Err(LayoutErrorOf::new(
                        LayoutErrorSiteOf::Node(node),
                        LayoutOperation::TextCapture,
                        LayoutErrorKindOf::InvalidInput(
                            LayoutInvalidInputOf::InlineBoundaryParentMismatch {
                                expected: Some(expected),
                                actual: Some(parent_axes),
                            },
                        ),
                    ));
                }
            }
        }
        let side = match (self.orientation, parent_axes.inline_start()) {
            (surgeist_text::LineOrientation::Horizontal, crate::PhysicalSide::Left)
            | (
                surgeist_text::LineOrientation::Vertical
                | surgeist_text::LineOrientation::SidewaysClockwise
                | surgeist_text::LineOrientation::SidewaysCounterclockwise,
                crate::PhysicalSide::Top,
            ) => surgeist_text::LineEdgeSide::Start,
            (surgeist_text::LineOrientation::Horizontal, crate::PhysicalSide::Right)
            | (
                surgeist_text::LineOrientation::Vertical
                | surgeist_text::LineOrientation::SidewaysClockwise
                | surgeist_text::LineOrientation::SidewaysCounterclockwise,
                crate::PhysicalSide::Bottom,
            ) => surgeist_text::LineEdgeSide::End,
            _ => {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::TextCapture,
                    LayoutErrorKindOf::InvalidInput(
                        LayoutInvalidInputOf::InlineBoundaryParentAxis {
                            parent: parent_axes,
                            formatter: self.axes,
                        },
                    ),
                ));
            }
        };
        let context = self
            .prepared
            .inline_box_edge_context(owner, parent, side)
            .map_err(|error| text_run_error(self.container, error.into()))?;
        let components = crate::text_layout::edge_components::from_native(endpoint.edge)
            .map_err(|error| text_run_error(self.container, error.into()))?;
        Ok(surgeist_text::InlineBoundaryAllocation::box_edge(
            context, components,
        ))
    }
}

pub(super) fn text_run_error<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    error: crate::text_layout::fit::FitError,
) -> LayoutErrorOf<Node, S, M> {
    let kind = match error {
        crate::text_layout::fit::FitError::TextWritingMode(mode) => {
            return LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::TextCapture,
                LayoutErrorKindOf::UnsupportedCapability(
                    crate::LayoutUnsupportedCapability::TextWritingMode(mode),
                ),
            );
        }
        crate::text_layout::fit::FitError::Wrapping(error) => {
            return crate::inline_wrapping::wrapping_error(node, error);
        }
        crate::text_layout::fit::FitError::Text(error) => LayoutErrorKindOf::Text(error),
        crate::text_layout::fit::FitError::Allocation => LayoutErrorKindOf::Allocation,
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
        crate::text_layout::fit::FitError::InvalidPrettyBasis => {
            LayoutErrorKindOf::InternalInvariant(
                crate::LayoutInternalInvariant::InvalidInlinePrettySelection,
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
    own_available: AxisSizingRequestOf<S>,
) -> LayoutResultOf<Node, crate::text_indent::ResolvedTextIndent<S>, S, M> {
    let amount =
        match crate::layout_math::normal_available_size(own_available) {
            None => policy.amount.absolute_px(),
            Some(width) => match policy.amount.resolve_against(
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
    if let Some(width) = crate::layout_math::normal_available_size(own_available) {
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
    alignment: VerticalAlignOf<S>,
) -> LayoutResultOf<Node, (), S, M> {
    if alignment.is_baseline_relative() && metrics.reference() != expected {
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
