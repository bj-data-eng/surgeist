//! Actual numeric fit receipts with explicit native scopes, without Text.
use super::*;
use crate::inline_wrapping::{WrapInside, WrappingContext, WrappingScopeInput};
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{InlineBoundaryInputOf, LayoutInputOf, NodeInputOf};

fn exercise<S: LayoutScalar>(
    first: InlineBreakOpportunity,
    second: InlineBreakOpportunity,
    expected: &[usize],
    ending: SelectedInlineLineEnd,
) {
    let metrics = InlineMetricsOf::from_line_height_and_baseline(
        crate::InlineBaselineContext::Physical,
        S::from_f64(10.0),
        S::from_f64(10.0),
    )
    .unwrap();
    let flow = InlineFlowOf::new(
        WritingMode::HorizontalTb,
        Direction::Ltr,
        AvailableOf::definite(S::from_f64(25.0)),
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(100, [1, 2, 3, 4, 5])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .style(100, NodeInputOf::default())
        .style(
            1,
            NodeInputOf {
                display: crate::Display::InlineBlock,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    first,
                )),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: crate::Display::InlineBlock,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    second,
                )),
                ..NodeInputOf::default()
            },
        )
        .style(
            4,
            NodeInputOf {
                display: crate::Display::InlineBlock,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                ..NodeInputOf::default()
            },
        )
        .input(
            2,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::Start,
                    metrics,
                    InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
        )
        .input(
            5,
            LayoutInputOf::inline_boundary(
                InlineBoundaryInputOf::contribution(
                    InlineBoundaryKind::End,
                    metrics,
                    InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
        );
    let (graph, _) =
        crate::engine::validation::validate_layout_request_with_text_graph(&tree, 100, &[], &[])
            .unwrap();
    let context = WrappingContext::new(
        &tree,
        &graph,
        &[WrappingScopeInput::NativePair {
            metric_contribution: None,
            edges: Default::default(),
            start: 2,
            end: 5,
            wrap_inside: WrapInside::Avoid,
        }],
        &[],
        &[],
        &[],
    )
    .unwrap();
    let projection = context
        .numeric_projection(&graph, 100, &[1, 2, 3, 4, 5])
        .unwrap();
    let atomic = |source_index, following| MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            baseline_reference: crate::InlineBaselineContext::Physical,
            source_index,
            size: Size::splat(S::from_f64(10.0)),
            content_size: Size::ZERO,
            margin: Edges::new(S::ZERO, S::from_f64(-5.0), S::ZERO, S::from_f64(5.0)),
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: Some(S::from_f64(10.0)),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: AtomicInlineParticipation::new(BidiLevel::try_new(0).unwrap(), following),
    };
    let participants = vec![
        atomic(0, first),
        MixedInlineParticipantOf::Boundary(InlineBoundaryControlOf::new(
            1,
            InlineBoundaryKind::Start,
            flow,
            metrics,
            InlineControlAlignment::Baseline,
            InlineBoundaryEdgeOf::ZERO,
            None,
        )),
        atomic(2, second),
        atomic(3, InlineBreakOpportunity::Allowed),
        MixedInlineParticipantOf::Boundary(InlineBoundaryControlOf::new(
            4,
            InlineBoundaryKind::End,
            flow,
            metrics,
            InlineControlAlignment::Baseline,
            InlineBoundaryEdgeOf::ZERO,
            None,
        )),
    ];
    let supplied = project_inline_participants(&participants);
    let run = ProjectedInlineRun {
        participants: &participants,
        supplied: &supplied,
    };
    let InlineSelectionStep::Selected(Some(selected)) = select_scope_inline_line(
        NumericLineQuery {
            goal: None,
            indent: S::ZERO,
            run,
            cursor: InlineSourceCursor::default(),
            next_boundary_descriptor: supplied.len(),
            available: S::from_f64(25.0),
            wraps: true,
            pending: None,
            axes: flow.flow_axes,
        },
        Some(&projection),
        |_, _| Ok(None),
        |receipt| Ok(receipt.summary.used_inline_extent <= S::from_f64(25.0)),
    )
    .unwrap() else {
        panic!("resolved metric-only selection does not retry its receiver")
    };
    assert_eq!(selected.end, ending);
    let boxes: Vec<_> = selected
        .line
        .units
        .iter()
        .filter_map(|u| match u.participant {
            MixedInlineParticipantOf::Atomic { item, .. } => Some(item.source_index),
            _ => None,
        })
        .collect();
    assert_eq!(boxes, expected);
    assert_eq!(
        selected.line.used_inline_extent.to_f64(),
        10.0 * expected.len() as f64
    );
    let InlineSelectionStep::Selected(Some(receiving)) = select_scope_inline_line(
        NumericLineQuery {
            goal: None,
            indent: S::ZERO,
            run,
            cursor: selected.next_cursor,
            next_boundary_descriptor: supplied.len(),
            available: S::from_f64(25.0),
            wraps: true,
            pending: selected.pending_strut,
            axes: flow.flow_axes,
        },
        Some(&projection),
        |_, _| Ok(None),
        |receipt| Ok(receipt.summary.used_inline_extent <= S::from_f64(25.0)),
    )
    .unwrap() else {
        panic!("resolved metric-only selection does not retry its receiver")
    };
    assert_eq!(receiving.end, SelectedInlineLineEnd::Paragraph);
    assert_eq!(
        receiving.line.used_inline_extent.to_f64(),
        30.0 - 10.0 * expected.len() as f64
    );
}
#[test]
fn actual_numeric_fit_prefers_outside_but_retains_necessary_and_forced_cuts() {
    fn lane<S: LayoutScalar>() {
        exercise::<S>(
            InlineBreakOpportunity::Allowed,
            InlineBreakOpportunity::Allowed,
            &[0],
            SelectedInlineLineEnd::Soft,
        );
        exercise::<S>(
            InlineBreakOpportunity::Prohibited,
            InlineBreakOpportunity::Allowed,
            &[0, 2],
            SelectedInlineLineEnd::Soft,
        );
        exercise::<S>(
            InlineBreakOpportunity::Allowed,
            InlineBreakOpportunity::Mandatory,
            &[0, 2],
            SelectedInlineLineEnd::Forced,
        );
    }
    lane::<f32>();
    lane::<f64>();
}

fn atomic_metric<S: LayoutScalar>(
    height: f64,
    baseline: Option<f64>,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            baseline_reference: crate::InlineBaselineContext::Physical,
            source_index: 0,
            size: Size::new(S::from_f64(10.0), S::from_f64(height)),
            content_size: Size::ZERO,
            margin: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: baseline.map(S::from_f64),
            alignment: InlineControlAlignment::Baseline,
        },
        participation: AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            InlineBreakOpportunity::Allowed,
        ),
    }
}
#[test]
fn actual_numeric_receipts_use_their_own_height_qualified_float_band() {
    fn lane<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let atomic = |source_index| {
            let mut participant = atomic_metric::<S>(10.0, Some(10.0));
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut participant else {
                unreachable!("atomic_metric constructs an atomic participant");
            };
            item.source_index = source_index;
            participant
        };
        let mut accumulator = NumericReceiptAccumulatorOf::new(None, axes);
        accumulator.push(atomic(0), axes);
        let mut first = accumulator.receipt(
            SelectedInlineLineEnd::Soft,
            InlineSourceCursor {
                edge_consumed: false,
                source: 1,
                descriptor: 1,
            },
            None,
            None,
            1,
        );
        let end = InlineBoundaryControlOf::new(
            1,
            InlineBoundaryKind::End,
            InlineFlowOf::new(
                WritingMode::HorizontalTb,
                Direction::Ltr,
                AvailableOf::MAX_CONTENT,
            ),
            InlineMetricsOf::from_line_height_and_baseline(
                crate::InlineBaselineContext::Physical,
                S::from_f64(30.0),
                S::from_f64(30.0),
            )
            .unwrap(),
            InlineControlAlignment::Baseline,
            InlineBoundaryEdgeOf::ZERO,
            None,
        );
        accumulator.push(MixedInlineParticipantOf::Boundary(end), axes);
        let second_atomic = atomic(2);
        accumulator.push(second_atomic, axes);
        let mut second = accumulator.receipt(
            SelectedInlineLineEnd::Soft,
            InlineSourceCursor {
                edge_consumed: false,
                source: 3,
                descriptor: 3,
            },
            None,
            None,
            3,
        );
        let band = |_: S, end: S| LogicalLineBandQueryResultOf {
            inline_start: S::ZERO,
            inline_end: S::from_f64(if end > S::from_f64(20.0) { 5.0 } else { 25.0 }),
            next_transition: (end > S::from_f64(20.0)).then_some(S::from_f64(40.0)),
        };
        let carried = Some(ActiveInlineStrutOf {
            metrics: InlineMetricContributionOf {
                baseline: S::from_f64(30.0),
                after_baseline: S::ZERO,
            },
            alignment: InlineControlAlignment::Baseline,
        });
        assert!(
            !resolve_numeric_candidate(
                &mut first,
                carried,
                S::ZERO,
                S::ZERO,
                S::from_f64(25.0),
                band
            )
            .unwrap()
        );
        assert!(
            !resolve_numeric_candidate(
                &mut second,
                carried,
                S::ZERO,
                S::ZERO,
                S::from_f64(25.0),
                band
            )
            .unwrap()
        );
        // Closing the carried scope does not erase the actual End strut's
        // 30/0 contribution. Both candidate intervals remain 30 units high.
        assert_eq!(
            (
                first.summary.used_inline_extent.to_f64(),
                first.summary.baseline.to_f64(),
                (first.summary.baseline + first.summary.after_baseline).to_f64(),
                first.band.unwrap().inline_end.to_f64()
            ),
            (10.0, 30.0, 30.0, 5.0)
        );
        assert_eq!(
            (
                second.summary.used_inline_extent.to_f64(),
                second.summary.baseline.to_f64(),
                (second.summary.baseline + second.summary.after_baseline).to_f64(),
                second.band.unwrap().inline_end.to_f64()
            ),
            (20.0, 30.0, 30.0, 5.0)
        );
        assert_eq!(
            (
                first.next_cursor.source,
                first.next_cursor.descriptor,
                first.next_cursor.edge_consumed
            ),
            (1, 1, false)
        );
        assert_eq!(
            (
                second.next_cursor.source,
                second.next_cursor.descriptor,
                second.next_cursor.edge_consumed
            ),
            (3, 3, false)
        );

        // An independent line starts at the legal Atomic break at 3,
        // beyond End at 1. Its actual source interval [3,5) contains two short
        // atomics and no boundary or carried strut from the earlier scope.
        assert_eq!(
            second_atomic.following_break(),
            Some(InlineBreakOpportunity::Allowed)
        );
        let short_start = second.next_cursor;
        let short_participants = [atomic(3), atomic(4)];
        let mut short_accumulator = NumericReceiptAccumulatorOf::new(None, axes);
        for participant in short_participants {
            short_accumulator.push(participant, axes);
        }
        let mut short = short_accumulator.receipt(
            SelectedInlineLineEnd::Soft,
            InlineSourceCursor {
                edge_consumed: false,
                source: short_start.source + short_participants.len(),
                descriptor: short_start.descriptor + short_participants.len(),
            },
            None,
            None,
            short_participants.len(),
        );
        assert!(
            resolve_numeric_candidate(&mut short, None, S::ZERO, S::ZERO, S::from_f64(25.0), band)
                .unwrap()
        );
        assert_eq!(
            (
                short.summary.used_inline_extent.to_f64(),
                short.summary.baseline.to_f64(),
                (short.summary.baseline + short.summary.after_baseline).to_f64(),
                short.band.unwrap().inline_end.to_f64()
            ),
            (20.0, 10.0, 10.0, 25.0)
        );
        assert_eq!(
            (
                short.next_cursor.source,
                short.next_cursor.descriptor,
                short.next_cursor.edge_consumed
            ),
            (5, 5, false)
        );
    }
    lane::<f32>();
    lane::<f64>();
}
#[test]
fn actual_numeric_fallback_envelope_cycle_refuses_without_a_retry_threshold() {
    fn lane<S: LayoutScalar>() {
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        // Actual fallback atomic15 has baseline15 and no descent. The carried
        // explicit 10+10 strut gives natural extent25; float fallback uses20.
        let strut = Some(ReferencedInlineStrutOf {
            source: InlineStrutSource::Direct,
            reference: crate::InlineBaselineContext::Physical,
            metrics: InlineMetricContributionOf {
                baseline: S::from_f64(10.0),
                after_baseline: S::from_f64(10.0),
            },
        });
        let mut accumulator = NumericReceiptAccumulatorOf::new(strut, axes);
        accumulator.push(atomic_metric(15.0, None), axes);
        let mut receipt = accumulator.receipt(
            SelectedInlineLineEnd::Soft,
            InlineSourceCursor {
                edge_consumed: false,
                source: 1,
                descriptor: 1,
            },
            None,
            strut,
            1,
        );
        let error = resolve_numeric_candidate(
            &mut receipt,
            None,
            S::ZERO,
            S::ZERO,
            S::from_f64(25.0),
            |_: S, end: S| LogicalLineBandQueryResultOf {
                inline_start: S::ZERO,
                inline_end: S::from_f64(25.0),
                next_transition: (end > S::from_f64(22.0)).then_some(S::from_f64(40.0)),
            },
        )
        .unwrap_err();
        assert_eq!(
            error,
            crate::inline_wrapping::NumericWrappingError::FeedbackCycle
        );
        let mapped: crate::LayoutErrorOf<u32, S> =
            crate::inline_wrapping::numeric_wrapping_error(7, error);
        assert!(matches!(
            mapped.kind(),
            crate::LayoutErrorKindOf::InlineFeedbackCycle
        ));
    }
    lane::<f32>();
    lane::<f64>();
}
