use super::*;
use std::cell::RefCell;
use std::convert::Infallible;

#[derive(Debug, PartialEq)]
enum Event<S> {
    Band(S, S),
    Clear(S, PostLineClearIntent),
    Receiver(usize),
}

fn atomic<S: LayoutScalar>(
    source_index: usize,
    top: bool,
    mandatory: bool,
) -> MixedInlineParticipantOf<S> {
    MixedInlineParticipantOf::Atomic {
        item: AtomicInlineBoxParticipant {
            baseline_reference: if top {
                crate::InlineBaselineContext::Physical
            } else {
                named()
            },
            source_index,
            size: Size::new(S::from_f64(10.0), S::from_f64(7.0)),
            content_size: Size::ZERO,
            margin: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
            scrollbar_size: Size::ZERO,
            first_baseline: (!top).then_some(S::from_f64(7.0)),
            alignment: if top {
                InlineControlAlignment::Top
            } else {
                InlineControlAlignment::Baseline
            },
        },
        participation: AtomicInlineParticipation::new(
            BidiLevel::try_new(0).unwrap(),
            if mandatory {
                InlineBreakOpportunity::Mandatory
            } else {
                InlineBreakOpportunity::Allowed
            },
        ),
    }
}

fn named() -> crate::InlineBaselineContext {
    crate::InlineBaselineContext::Named(crate::BaselineName::Alphabetic)
}

fn input<S: LayoutScalar>(
    participants: Vec<MixedInlineParticipantOf<S>>,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        inline_baseline_context: named(),
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            Direction::Ltr,
        ),
        observe_tail: true,
        wrapping: None,
        available_inline_extent: AxisSizingRequestOf::new(
            crate::AvailableSpaceOf::Definite(
                crate::NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap(),
            ),
            crate::SizingOperation::Normal,
        ),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants,
    }
}

fn band<S: LayoutScalar>(
    trace: &RefCell<Vec<Event<S>>>,
    start: S,
    end: S,
) -> Result<LogicalLineBandQueryResultOf<S>, Infallible> {
    trace.borrow_mut().push(Event::Band(start, end));
    Ok(LogicalLineBandQueryResultOf {
        inline_start: S::ZERO,
        inline_end: S::from_f64(100.0),
        next_transition: None,
    })
}

fn clear<S: LayoutScalar>(
    trace: &RefCell<Vec<Event<S>>>,
    block: S,
    intent: PostLineClearIntent,
) -> S {
    trace.borrow_mut().push(Event::Clear(block, intent));
    block
}

fn resolved<S: LayoutScalar>() -> ReferencedInlineStrutOf<S> {
    ReferencedInlineStrutOf {
        metrics: InlineMetricContributionOf {
            baseline: S::from_f64(7.0),
            after_baseline: S::ZERO,
        },
        reference: named(),
        source: InlineStrutSource::Atomic { source_index: 1 },
    }
}

fn resume_after_completed_lines<S: LayoutScalar>() {
    let query = input::<S>(vec![
        atomic(0, false, true),
        atomic(1, true, true),
        atomic(2, false, false),
    ]);
    let expected_trace = RefCell::new(Vec::new());
    let expected = layout_mixed_inline_run_with_formatted_observation(
        query.clone(),
        |start, end| band(&expected_trace, start, end),
        |block, intent| clear(&expected_trace, block, intent),
        true,
        |index, reference| {
            assert_eq!(index, 1);
            assert!(reference == named());
            expected_trace.borrow_mut().push(Event::Receiver(index));
            Ok(resolved())
        },
    )
    .unwrap();
    let trace = RefCell::new(Vec::new());
    let mut run = NumericRun::new(query, true).unwrap();
    let demand = run
        .advance(
            |start, end| band(&trace, start, end),
            |block, intent| clear(&trace, block, intent),
        )
        .unwrap()
        .expect("a real receiving line needs the earlier Top source");
    assert_eq!(demand.source_index, 1);
    assert!(demand.expected == named());
    assert!(
        trace
            .borrow()
            .iter()
            .any(|event| matches!(event, Event::Clear(..)))
    );
    trace
        .borrow_mut()
        .push(Event::Receiver(demand.source_index));
    run.accept_receiver(resolved()).unwrap();
    assert!(
        run.advance(
            |start, end| band(&trace, start, end),
            |block, intent| clear(&trace, block, intent)
        )
        .unwrap()
        .is_none()
    );
    let report = run.finish::<Infallible>().unwrap();
    assert_eq!(report.inline_extent, S::from_f64(10.0));
    assert_eq!(report.block_extent, S::from_f64(21.0));
    assert_eq!(report.line_bands.len(), 3);
    for (index, atomic) in report.atomics.iter().enumerate() {
        assert_eq!(atomic.item.source_index, index);
        assert_eq!(
            expect_logical_position(atomic.position),
            LogicalPointOf::new(S::ZERO, S::from_f64(7.0 * index as f64))
        );
    }
    let InlineBaselineReportOf::Numeric(baselines) = report.baselines else {
        panic!("numeric lines retain their logical baseline report")
    };
    assert_eq!(baselines.first, Some(S::from_f64(7.0)));
    assert_eq!(baselines.last, Some(S::from_f64(21.0)));
    assert!(baselines.first_named && baselines.last_named);
    assert_eq!(report, expected);
    let literal = vec![
        Event::Band(S::ZERO, S::from_f64(7.0)),
        Event::Clear(S::from_f64(7.0), PostLineClearIntent::None),
        Event::Band(S::from_f64(7.0), S::from_f64(14.0)),
        Event::Clear(S::from_f64(14.0), PostLineClearIntent::None),
        Event::Receiver(1),
        Event::Band(S::from_f64(14.0), S::from_f64(21.0)),
        Event::Clear(S::from_f64(21.0), PostLineClearIntent::None),
    ];
    assert_eq!(*trace.borrow(), literal);
    assert_eq!(trace.into_inner(), expected_trace.into_inner());
}

#[test]
fn receiver_resume_preserves_completed_provider_calls_and_report() {
    resume_after_completed_lines::<f32>();
    resume_after_completed_lines::<f64>();
}

#[test]
fn terminal_top_source_does_not_demand_a_receiver() {
    let mut run = NumericRun::new(input::<f32>(vec![atomic(1, true, true)]), false).unwrap();
    let trace = RefCell::new(Vec::new());
    assert!(
        run.advance(
            |start, end| band(&trace, start, end),
            |block, intent| clear(&trace, block, intent)
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(run.finish::<Infallible>().unwrap().line_bands.len(), 1);
}

#[test]
fn invalid_receiver_stops_before_the_receiving_lines_provider_calls() {
    let mut run = NumericRun::new(
        input::<f32>(vec![atomic(1, true, true), atomic(2, false, false)]),
        false,
    )
    .unwrap();
    let trace = RefCell::new(Vec::new());
    assert!(
        run.advance(
            |start, end| band(&trace, start, end),
            |block, intent| clear(&trace, block, intent)
        )
        .unwrap()
        .is_some()
    );
    let previous = vec![
        Event::Band(0.0, 7.0),
        Event::Clear(7.0, PostLineClearIntent::None),
    ];
    assert_eq!(*trace.borrow(), previous);
    let mut response = resolved();
    response.reference = crate::InlineBaselineContext::Physical;
    assert!(matches!(
        run.accept_receiver(response),
        Err(NumericWrappingError::InvalidAtomicReceiverBasis)
    ));
    assert_eq!(*trace.borrow(), previous);
}
