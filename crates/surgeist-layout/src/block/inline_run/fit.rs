use super::super::{float_text, floats};
use super::*;

pub(super) struct NumericRunContext<'a, Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) run: &'a [Node],
    pub(super) source_index_start: usize,
    pub(super) available_inline_extent: AxisSizingRequestOf<S>,
    pub(super) cursor_block: S,
    pub(super) uncollapsed_cursor_block: S,
    pub(super) constants: &'a Constants<S>,
    pub(super) provider_enabled: bool,
    pub(super) line_box_context: crate::line_box::LineBoxContext,
    pub(super) tail: crate::text_layout::fit::TailContext,
    pub(super) indent_state: crate::text_indent::IndentLineState,
    pub(super) participants: Vec<MixedInlineParticipantOf<S>>,
}

pub(super) struct NumericRunFit<S: LayoutScalar> {
    pub(super) report: crate::inline::MixedInlineRunReportOf<S>,
    pub(super) margin: InlineRunMargin,
    pub(super) cursor_block: S,
}

type ProviderError<Node, S, M> = std::cell::RefCell<Option<LayoutErrorOf<Node, S, M>>>;

pub(super) fn fit_numeric_run<'input, Tree, S, M>(
    tree: &mut Tree,
    context: NumericRunContext<'_, Tree::Node, S>,
    float_exclusions: &FloatExclusions<S, Tree::Node>,
    atomic_children: &mut [RetainedAtomicChild<'input, Tree::Node, S>],
    provider_error: &ProviderError<Tree::Node, S, M>,
) -> LayoutResultOf<Tree::Node, NumericRunFit<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let NumericRunContext {
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
    } = context;
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
                            MixedInlineParticipantOf::Boundary(control) => control.source_index(),
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
                        if available_inline_extent.operation() == crate::SizingOperation::MinContent
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
    let (margin, cursor_block) =
        if crate::inline::numeric_run_has_formatted_line(&participants, wrapping.as_ref()) {
            (InlineRunMargin::Separates, cursor_block)
        } else {
            (InlineRunMargin::Adjoins, uncollapsed_cursor_block)
        };
    let logical_content_box_inset = constants.logical_content_box_inset();
    let observe_formatted = matches!(
        constants.wrap_style,
        crate::TextWrapStyle::Balance
            | crate::TextWrapStyle::AvoidShortLastLine
            | crate::TextWrapStyle::Pretty
    ) && crate::layout_math::normal_available_size(available_inline_extent)
        .is_some()
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
            float_exclusions.clearance_for_line_intent(cursor_block + block, clear) - cursor_block
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
                atomic_children,
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
    let report = result.map_err(|error| match error {
        crate::inline::NumericRunError::Numeric(error) => {
            crate::inline_wrapping::numeric_wrapping_error(container, error)
        }
        crate::inline::NumericRunError::Band(error) => error,
    })?;
    Ok(NumericRunFit {
        report,
        margin,
        cursor_block,
    })
}

pub(super) struct CapturedRunContext<'a, 'input, Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) run: &'a [Node],
    pub(super) source_index_start: usize,
    pub(super) available_inline_extent: AxisSizingRequestOf<S>,
    pub(super) cursor_block: S,
    pub(super) adjoining_margin: S,
    pub(super) constants: &'a Constants<S>,
    pub(super) input: ComputeInputOf<S>,
    pub(super) orientation: surgeist_text::LineOrientation,
    pub(super) provider_enabled: bool,
    pub(super) line_box_context: crate::line_box::LineBoxContext,
    pub(super) tail: crate::text_layout::fit::TailContext,
    pub(super) indent_state: crate::text_indent::IndentLineState,
    pub(super) hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub(super) containing_size: Size<S>,
    pub(super) box_frontier: &'a Option<crate::text_layout::boxes::BoxFrontier>,
    pub(super) admitted: &'a crate::text_layout::AdmittedRun<'input, Node>,
    pub(super) text_atomics: &'a [crate::text_layout::run::Atomic<Node, S>],
    pub(super) float_events: float_text::FloatEvents<Node, S>,
}

pub(super) struct CapturedRunFit<Node, S: LayoutScalar> {
    pub(super) output: crate::text_layout::run::Output<Node, S>,
    pub(super) report: crate::inline::MixedInlineRunReportOf<S>,
    pub(super) formatted: Option<crate::text_layout::formatted::FormattedFlowItem<Node, S>>,
}

type CapturedRunFitResult<Node, S, M> = LayoutResultOf<Node, CapturedRunFit<Node, S>, S, M>;

pub(super) fn fit_captured_run<'input, Tree, S, M>(
    tree: &mut Tree,
    context: CapturedRunContext<'_, 'input, Tree::Node, S>,
    float_exclusions: &mut FloatExclusions<S, Tree::Node>,
    encounter_plans: &mut Vec<PendingFloat<Tree::Node, S>>,
    provider_error: &ProviderError<Tree::Node, S, M>,
) -> CapturedRunFitResult<Tree::Node, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let CapturedRunContext {
        container,
        run,
        source_index_start,
        available_inline_extent,
        cursor_block,
        adjoining_margin,
        constants,
        input,
        orientation,
        provider_enabled,
        line_box_context,
        tail,
        indent_state,
        hyphenation_state,
        containing_size,
        box_frontier,
        admitted,
        text_atomics,
        float_events,
    } = context;
    let binding = admitted.binding;
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
    let available = if crate::layout_math::normal_available_size(available_inline_extent).is_some()
    {
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
            state.formatted_snapshot(container, run, &admitted.segment, input, formatted_mode)
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
        orientation,
        inline_baseline_context: constants.inline_baseline_context,
        groups: group_snapshot,
        wrapping: tree.wrapping_context().and_then(|(context, graph)| {
            crate::text_layout::run::WrappingProjection::new(
                context,
                graph,
                admitted,
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
                    prepared: &admitted.binding.prepared,
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
    let intrinsic_max =
        !float_events.is_empty() && (available.operation() == crate::SizingOperation::MaxContent);
    let quality_replay = matches!(
        constants.wrap_style,
        crate::TextWrapStyle::Balance
            | crate::TextWrapStyle::AvoidShortLastLine
            | crate::TextWrapStyle::Pretty
    ) && crate::layout_math::normal_available_size(available).is_some();
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
                float_text::source_interval(admitted, &associations, context, graph, owner, funding)
            },
            funding,
        )
        .map_err(|error| text_run_error(container, error))?;
    let encounters = std::cell::RefCell::new(
        float_text::Encounters::new(
            bound_events,
            exclusions
                .try_clone()
                .map_err(|error| text_run_error(container, error.into()))?,
            cursor_block,
            if intrinsic_max {
                floats::FloatInlineSpace::IntrinsicMaximum
            } else {
                floats::FloatInlineSpace::Finite
            },
            quality_replay
                || tail == crate::text_layout::fit::TailContext::CompleteBlock
                    && crate::layout_math::normal_available_size(available).is_some(),
            quality_replay,
        )
        .map_err(|error| text_run_error(container, error))?,
    );
    let result = crate::text_layout::run::layout(
        admitted,
        &associations,
        text_atomics,
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
    let formatted = tree
        .text_computation()
        .map(|state| {
            state.formatted_merge_run(
                container,
                run,
                &admitted.segment,
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
    Ok(CapturedRunFit {
        report: output.report.clone(),
        output,
        formatted,
    })
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
