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
