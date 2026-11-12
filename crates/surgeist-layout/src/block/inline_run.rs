use super::absolute::{StaticPositionGeometry, absolute_static_position};
use super::scroll::{BlockScrollContributions, child_scrollbar_size};
use super::*;
use crate::{AxisSizingRequestOf, FlowAxes};

mod fit;
mod placement;
mod prepare;

use fit::{CapturedRunContext, NumericRunContext, fit_captured_run, fit_numeric_run};
use placement::{InlineSourcePlacement, place_inline_sources};
use prepare::PreparedInlineRun;
pub(super) use prepare::{InlineRunPreparation, PendingInlineAtomic};

#[cfg(test)]
mod baseline_offset_tests;

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

    pub(super) fn begin<'input, Tree, S, M>(
        &self,
        tree: &mut Tree,
        node: Tree::Node,
        run: &[Tree::Node],
        constants: &Constants<S>,
        context: &InlineRunContext<S>,
        adjoining_margin: S,
    ) -> LayoutResultOf<Tree::Node, InlineRunPreparation<'input, Tree::Node, S>, S, M>
    where
        Tree: Compute<'input, M, Scalar = S>,
        S: LayoutScalar,
    {
        let semantic = self.semantic.as_ref().map(|segment| {
            tree.text_computation()
                .and_then(|state| state.semantic_run(node, segment))
                .expect("semantic transition retains its validated actual binding")
        });
        InlineRunPreparation::begin(
            tree,
            node,
            run,
            semantic,
            constants,
            context,
            adjoining_margin,
        )
    }

    pub(super) fn apply<'input, Tree, S, M>(
        self,
        tree: &mut Tree,
        node: Tree::Node,
        run: &[Tree::Node],
        constants: &Constants<S>,
        request: InlineRunRequest<'input, Tree::Node, S>,
        state: InlineRunTransitionState<'_, Tree::Node, S>,
    ) -> LayoutResultOf<Tree::Node, (), S, M>
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
        let publish_native = request.context.set_layout;
        let mut placement = layout_inline_run_children(
            tree,
            node,
            run,
            constants,
            request,
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

pub(super) struct InlineRunContext<S: LayoutScalar> {
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

pub(super) struct InlineRunRequest<'input, Node, S: LayoutScalar> {
    pub(super) prepared: PreparedInlineRun<'input, Node, S>,
    pub(super) context: InlineRunContext<S>,
    pub(super) adjoining_margin: S,
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
    constants: &Constants<S>,
    request: InlineRunRequest<'input, <Tree as Traverse>::Node, S>,
    float_exclusions: &mut FloatExclusions<S, <Tree as Traverse>::Node>,
    contributions: &mut BlockScrollContributions<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, InlineRunPlacement<<Tree as Traverse>::Node, S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunRequest {
        prepared,
        context,
        adjoining_margin,
    } = request;
    let PreparedInlineRun {
        mut cursor_block,
        containing_size,
        provider_enabled,
        participants,
        mut atomic_children,
        control_children,
        text_run,
        text_orientation,
        text_atomics,
        float_events,
        absolute_children,
    } = prepared;
    let InlineRunContext {
        inline_request: available_inline_extent,
        mut box_frontier,
        line_box_context,
        indent_state,
        hyphenation_state,
        tail,
        source_index_start,
        cursor_block: cursor_block_before_entry,
        owned_float_block_end,
        input,
        node_inner_size,
        set_layout,
        ..
    } = context;
    let uncollapsed_cursor_block = cursor_block_before_entry;
    let text_binding = text_run.as_ref().map(|run| run.binding);
    let margin;
    let mut encounter_plans = Vec::new();
    let mut static_positions = Vec::new();
    let provider_error = std::cell::RefCell::new(None);
    let mut text_output = None;
    let mut text_progress = None;
    let mut formatted_item = None;
    let mut report = if let Some(admitted) = text_run.as_ref() {
        let fit = fit_captured_run(
            tree,
            CapturedRunContext {
                container,
                run,
                source_index_start,
                available_inline_extent,
                cursor_block,
                adjoining_margin,
                constants,
                input,
                orientation: text_orientation
                    .expect("the actual Text binding has its resolved orientation"),
                provider_enabled,
                line_box_context,
                tail,
                indent_state,
                hyphenation_state,
                containing_size,
                box_frontier: &box_frontier,
                admitted,
                text_atomics: &text_atomics,
                float_events,
            },
            float_exclusions,
            &mut encounter_plans,
            &provider_error,
        )?;
        formatted_item = fit.formatted;
        text_progress = Some(fit.output.progress);
        margin = if fit.output.progress.phase == crate::text_layout::fit::EntryPhase::Consumed {
            InlineRunMargin::Separates
        } else {
            InlineRunMargin::Adjoins
        };
        text_output = Some(fit.output);
        fit.report
    } else {
        let fit = fit_numeric_run(
            tree,
            NumericRunContext {
                container,
                run,
                source_index_start,
                available_inline_extent,
                cursor_block,
                uncollapsed_cursor_block,
                constants,
                provider_enabled,
                line_box_context,
                tail,
                indent_state,
                participants,
            },
            float_exclusions,
            &mut atomic_children,
            &provider_error,
        )?;
        margin = fit.margin;
        cursor_block = fit.cursor_block;
        fit.report
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
    place_inline_sources(
        tree,
        InlineSourcePlacement {
            container,
            run,
            source_index_start,
            input,
            constants,
            available_inline_extent,
            containing_size,
            node_inner_size,
            cursor_block,
            set_layout,
            text_binding,
            text_output,
            atomic_children,
            control_children,
            report: &report,
            box_frontier: &mut box_frontier,
        },
        &mut static_positions,
        contributions,
    )?;
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
