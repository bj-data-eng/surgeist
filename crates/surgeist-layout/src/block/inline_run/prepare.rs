use super::super::relative::{self, InsetReference};
use super::super::sizing::{resolve_auto_optional, resolve_length_or_zero};
use super::super::{float_text, in_flow};
use super::*;

type PreparedInlineRunResult<'input, Node, S, M> =
    LayoutResultOf<Node, PreparedInlineRun<'input, Node, S>, S, M>;
pub(super) type InlineRunControl<Node, S> = (Node, usize, Option<crate::VerticalAlignOf<S>>);

pub(super) struct PreparedInlineRun<'input, Node, S: LayoutScalar> {
    pub(super) cursor_block: S,
    pub(super) containing_size: Size<S>,
    pub(super) provider_enabled: bool,
    pub(super) participants: Vec<MixedInlineParticipantOf<S>>,
    pub(super) atomic_children: Vec<RetainedAtomicChild<'input, Node, S>>,
    pub(super) control_children: Vec<InlineRunControl<Node, S>>,
    pub(super) text_run: Option<crate::text_layout::AdmittedRun<'input, Node>>,
    pub(super) text_orientation: Option<surgeist_text::LineOrientation>,
    pub(super) text_atomics: Vec<crate::text_layout::run::Atomic<Node, S>>,
    pub(super) float_events: float_text::FloatEvents<Node, S>,
    pub(super) absolute_children: Vec<Node>,
}

pub(super) fn prepare_inline_run<'input, Tree, S, M>(
    tree: &mut Tree,
    container: Tree::Node,
    run: &[Tree::Node],
    semantic: Option<crate::text_layout::AdmittedRun<'input, Tree::Node>>,
    context: &InlineRunContext<'_, S>,
    adjoining_margin: S,
) -> PreparedInlineRunResult<'input, Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let InlineRunContext {
        inline_request: available_inline_extent,
        child_request: pass_sizing_request,
        indent_state,
        source_index_start,
        cursor_block: uncollapsed_cursor_block,
        constants,
        input,
        node_inner_size,
        set_layout,
        ..
    } = *context;
    // Numeric participants establish line existence before placement. Text
    // binds its actual candidate disposition to entry geometry in the fitter.
    let mut cursor_block = uncollapsed_cursor_block + adjoining_margin;
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
    let mut float_events = float_text::FloatEvents::new();
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
                .map(in_flow::resolved_length_auto_fallback_zero);
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
            let known = in_flow::in_flow_child_known_size::<Tree, M>(
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
                        in_flow::in_flow_child_inline_request(
                            &child_style,
                            tree.has_leaf_measurement(child),
                            axes,
                            inline,
                            available.inline,
                        )
                        .map_err(|error| {
                            crate::error::sizing_resolution_error(child, error.into())
                        })?,
                        in_flow::in_flow_child_block_request(available.block),
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
                    float_text::FloatEvent::new(
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
    Ok(PreparedInlineRun {
        cursor_block,
        containing_size,
        provider_enabled,
        participants,
        atomic_children,
        control_children,
        text_run,
        text_orientation,
        text_atomics,
        float_events,
        absolute_children,
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

fn resolve_atomic_inline_margin<S: LayoutScalar>(margin: Edges<Option<S>>) -> Edges<S> {
    margin.map(|value| value.unwrap_or(S::ZERO))
}
