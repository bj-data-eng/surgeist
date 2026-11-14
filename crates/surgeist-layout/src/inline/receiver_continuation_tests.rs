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

fn balanced_input<S: LayoutScalar>() -> MixedInlineRunInputOf<S> {
    let mut query = input(vec![
        atomic(0, false, true),
        atomic(1, true, true),
        atomic(2, false, false),
        atomic(3, false, false),
        atomic(4, false, false),
    ]);
    for (participant, width) in query
        .participants
        .iter_mut()
        .zip([10.0, 10.0, 30.0, 30.0, 10.0])
    {
        let MixedInlineParticipantOf::Atomic { item, .. } = participant else {
            unreachable!()
        };
        item.size.width = S::from_f64(width);
    }
    query.text_wrap_style = crate::TextWrapStyle::Balance;
    query.available_inline_extent = AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Definite(
            crate::NonNegativeFiniteOf::new(S::from_f64(60.0)).unwrap(),
        ),
        crate::SizingOperation::Normal,
    );
    query
}

fn balanced_band<S: LayoutScalar>(
    trace: &RefCell<Vec<Event<S>>>,
    start: S,
    end: S,
) -> Result<LogicalLineBandQueryResultOf<S>, Infallible> {
    trace.borrow_mut().push(Event::Band(start, end));
    Ok(LogicalLineBandQueryResultOf {
        inline_start: S::ZERO,
        inline_end: S::from_f64(60.0),
        next_transition: None,
    })
}

fn expected_balanced_trace<S: LayoutScalar>() -> Vec<Event<S>> {
    fn line<S: LayoutScalar>(events: &mut Vec<Event<S>>, start: f64, queries: usize) {
        for _ in 0..queries {
            events.push(Event::Band(S::from_f64(start), S::from_f64(start + 7.0)));
        }
        events.push(Event::Clear(
            S::from_f64(start + 7.0),
            PostLineClearIntent::None,
        ));
    }
    fn prefix<S: LayoutScalar>(events: &mut Vec<Event<S>>) {
        line(events, 0.0, 1);
        line(events, 7.0, 1);
    }
    fn group<S: LayoutScalar>(events: &mut Vec<Event<S>>, queries: &[usize]) {
        events.push(Event::Receiver(1));
        for (index, &count) in queries.iter().enumerate() {
            line(events, 14.0 + 7.0 * index as f64, count);
        }
    }
    let mut events = Vec::new();
    prefix(&mut events);
    group(&mut events, &[1, 1]); // whole reference Auto
    prefix(&mut events);
    group(&mut events, &[1, 1]); // rolling local Auto
    // Residuals0/50 generate seventeen goals0..50 in steps of3.125.
    // Goals0..12.5 retain60/10;15.625..25 choose30/40; larger goals
    // choose30/30/10 and exceed the authentic two-line count. Each quality
    // line queries every legal completion/prefix twice and its selected band
    // once: three/two/one candidates therefore mean7/5/3 provider queries.
    for _ in 0..5 {
        group(&mut events, &[7, 3]);
    }
    for _ in 0..4 {
        group(&mut events, &[7, 5]);
    }
    for _ in 0..8 {
        group(&mut events, &[7, 5, 3]);
    }
    group(&mut events, &[7, 5]); // first best goal's winner replay
    prefix(&mut events);
    group(&mut events, &[7, 5]); // whole selected program replay
    events
}

#[test]
fn balanced_receiver_resume_preserves_eligible_search_and_selected_program() {
    fn lane<S: LayoutScalar>() {
        let query = balanced_input::<S>();
        let trace = RefCell::new(Vec::new());
        let mut run = NumericRun::new(query.clone(), true).unwrap();
        while let Some(demand) = run
            .advance(
                |start, end| balanced_band(&trace, start, end),
                |block, intent| clear(&trace, block, intent),
            )
            .unwrap()
        {
            assert_eq!(demand.source_index, 1);
            assert!(demand.expected == named());
            trace.borrow_mut().push(Event::Receiver(1));
            run.accept_receiver(resolved()).unwrap();
        }
        let report = run.finish::<Infallible>().unwrap();
        assert_eq!(report.inline_extent, S::from_f64(40.0));
        assert_eq!(report.block_extent, S::from_f64(28.0));
        assert_eq!(report.line_bands.len(), 4);
        assert_eq!(report.atomics.len(), 5);
        assert_eq!(
            report.lines.iter().map(|line| line.end).collect::<Vec<_>>(),
            [
                surgeist_text::LineEnd::Forced,
                surgeist_text::LineEnd::Forced,
                surgeist_text::LineEnd::Soft,
                surgeist_text::LineEnd::Paragraph,
            ]
        );
        for (actual, (source, x, y)) in report.atomics.iter().zip([
            (0, 0.0, 0.0),
            (1, 0.0, 7.0),
            (2, 0.0, 14.0),
            (3, 0.0, 21.0),
            (4, 30.0, 21.0),
        ]) {
            assert_eq!(actual.item.source_index, source);
            assert_eq!(
                expect_logical_position(actual.position),
                LogicalPointOf::new(S::from_f64(x), S::from_f64(y))
            );
        }
        let baselines = report.baselines.numeric();
        assert_eq!(baselines.first, Some(S::from_f64(7.0)));
        assert_eq!(baselines.last, Some(S::from_f64(28.0)));
        assert!(baselines.first_named && baselines.last_named);
        assert_eq!(*trace.borrow(), expected_balanced_trace());
        let synchronous_trace = RefCell::new(Vec::new());
        let synchronous = layout_mixed_inline_run_with_formatted_observation(
            query,
            |start, end| balanced_band(&synchronous_trace, start, end),
            |block, intent| clear(&synchronous_trace, block, intent),
            true,
            |index, reference| {
                assert_eq!(index, 1);
                assert!(reference == named());
                synchronous_trace.borrow_mut().push(Event::Receiver(index));
                Ok(resolved())
            },
        )
        .unwrap();
        assert_eq!(synchronous, report);
        assert_eq!(synchronous_trace.into_inner(), expected_balanced_trace());
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn balanced_losing_trial_preserves_exact_provider_error_after_receiver() {
    let trace = RefCell::new(Vec::new());
    let queries = Cell::new(0);
    let mut run = NumericRun::new(balanced_input::<f64>(), true).unwrap();
    loop {
        let next = run.advance(
            |start, end| {
                trace.borrow_mut().push(Event::Band(start, end));
                queries.set(queries.get() + 1);
                // Reference and local Auto each query four real lines. The first
                // finite goal is0, which would tie local Auto and lose its priority.
                if queries.get() == 9 {
                    return Err("losing Balance trial band");
                }
                Ok(LogicalLineBandQueryResultOf {
                    inline_start: 0.0,
                    inline_end: 60.0,
                    next_transition: None,
                })
            },
            |block, intent| clear(&trace, block, intent),
        );
        match next {
            Ok(Some(demand)) => {
                assert_eq!(demand.source_index, 1);
                trace.borrow_mut().push(Event::Receiver(1));
                run.accept_receiver(resolved()).unwrap();
            }
            Err(NumericRunError::Band(error)) => {
                assert_eq!(error, "losing Balance trial band");
                break;
            }
            _ => panic!("the real losing-trial provider error must propagate"),
        }
    }
    let literal = expected_balanced_trace::<f64>();
    // Each Auto pass has two prefix lines and two receiving-group lines:
    // nine events each. The next receiver precedes the failing provider call.
    assert_eq!(*trace.borrow(), literal[..20]);
}

#[test]
fn balanced_group_receipts_reject_real_count_and_terminal_mismatches() {
    fn walk(
        query: &MixedInlineRunInputOf<f64>,
        limit: Option<usize>,
        width: f64,
    ) -> (NumericSequence<f64>, NumericWalkOutcome) {
        let supplied = project_inline_participants(&query.participants);
        let mut sequence = NumericSequence::new(query, true);
        let boundaries = numeric_line_boundaries(query);
        let outcome = walk_numeric_sequence(
            NumericWalkInput {
                input: query,
                supplied: &supplied,
                available: 60.0,
                receiver_source: ReceiverSourceMode::Deferred,
            },
            &mut sequence,
            &mut boundaries.into_iter().peekable(),
            NumericWalkControl::group(None, limit),
            |_, _| {
                Ok::<_, Infallible>(LogicalLineBandQueryResultOf {
                    inline_start: 0.0,
                    inline_end: width,
                    next_transition: None,
                })
            },
            |block, _| block,
        )
        .unwrap();
        (sequence, outcome)
    }
    let query = super::balance_tests::input::<f64>(
        &[30.0, 30.0, 10.0, 10.0],
        &[],
        Direction::Ltr,
        crate::TextWrapStyle::Balance,
    );
    let (reference, _) = walk(&query, None, 60.0);
    let references = numeric_reference_groups(&reference.selected_lines).unwrap();
    assert_eq!(references[0].count, 2);
    let entry = NumericSequence::new(&query, true).checkpoint();
    let (count, outcome) = walk(&query, Some(2), 30.0);
    assert!(outcome == NumericWalkOutcome::CountExceeded);
    assert_eq!(count.selected_lines.len(), 3);
    assert!(
        finish_numeric_group_trial(&count, entry, &references[0], outcome)
            .unwrap()
            .is_none()
    );
    let forced = super::balance_tests::input::<f64>(
        &[30.0, 30.0, 10.0, 10.0],
        &[2],
        Direction::Ltr,
        crate::TextWrapStyle::Balance,
    );
    let (terminal, outcome) = walk(&forced, Some(2), 60.0);
    assert!(outcome == NumericWalkOutcome::GroupComplete);
    assert_eq!(terminal.selected_lines.len(), 2);
    assert_eq!(
        terminal.selected_lines.last().unwrap().ending,
        surgeist_text::LineEnd::Forced
    );
    assert!(
        finish_numeric_group_trial(&terminal, entry, &references[0], outcome)
            .unwrap()
            .is_none()
    );
}

fn tail_input<S: LayoutScalar>(widths: &[f64]) -> MixedInlineRunInputOf<S> {
    let participants = [10.0, 10.0]
        .into_iter()
        .chain(widths.iter().copied())
        .enumerate()
        .map(|(source, width)| {
            let mut participant = atomic(source, source == 1, source < 2);
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut participant else {
                unreachable!()
            };
            item.size.width = S::from_f64(width);
            participant
        })
        .collect();
    let mut query = input(participants);
    query.text_wrap_style = crate::TextWrapStyle::AvoidShortLastLine;
    query.available_inline_extent = AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Definite(
            crate::NonNegativeFiniteOf::new(S::from_f64(60.0)).unwrap(),
        ),
        crate::SizingOperation::Normal,
    );
    query
}

fn expected_tail_trace<S: LayoutScalar>(winner: bool) -> Vec<Event<S>> {
    fn line<S: LayoutScalar>(events: &mut Vec<Event<S>>, start: f64, queries: usize) {
        for _ in 0..queries {
            events.push(Event::Band(S::from_f64(start), S::from_f64(start + 7.0)));
        }
        events.push(Event::Clear(
            S::from_f64(start + 7.0),
            PostLineClearIntent::None,
        ));
    }
    fn group<S: LayoutScalar>(events: &mut Vec<Event<S>>, frozen: bool, counts: &[usize]) {
        events.push(Event::Receiver(1));
        if frozen {
            line(events, 14.0, 1);
        }
        for (index, &count) in counts.iter().enumerate() {
            line(
                events,
                14.0 + 7.0 * (index + usize::from(frozen)) as f64,
                count,
            );
        }
    }
    let mut events = Vec::new();
    line(&mut events, 0.0, 1);
    line(&mut events, 7.0, 1);
    group(&mut events, winner, &[1, 1]); // discovery
    group(&mut events, winner, &[1, 1]); // one baseline frontier replay
    if winner {
        // Fullness goals interpolate1/6..1 in seventeen steps. k0..7
        // choose30/30/10 and exceed count3 after the frozen60; k8..11
        // choose30/40; k12..16 retain60/10. Frozen60 stays ordinary.
        // Each quality line queries every completion/prefix twice and its
        // selected band once, giving7/5/3 calls for3/2/1 candidates.
        for _ in 0..8 {
            group(&mut events, true, &[7, 5, 3]);
        }
        for _ in 0..4 {
            group(&mut events, true, &[7, 5]);
        }
        for _ in 0..5 {
            group(&mut events, true, &[7, 3]);
        }
        group(&mut events, true, &[7, 5]); // first winning goal, replayed once
    } else {
        // Two indivisible atoms60/10 cannot improve. Every goal has only
        // completion70 and prefix60, then the single10 completion. Restoring
        // saved Auto adds no provider, clearance or receiver calls afterward.
        for _ in 0..17 {
            group(&mut events, false, &[5, 3]);
        }
    }
    events
}

#[test]
fn tail_receiver_resume_preserves_frozen_prefix_and_winner_trace() {
    fn lane<S: LayoutScalar>() {
        let query = tail_input::<S>(&[60.0, 30.0, 30.0, 10.0]);
        let trace = RefCell::new(Vec::new());
        let mut run = NumericRun::new(query.clone(), true).unwrap();
        while let Some(demand) = run
            .advance(
                |start, end| balanced_band(&trace, start, end),
                |block, intent| clear(&trace, block, intent),
            )
            .unwrap()
        {
            assert_eq!(demand.source_index, 1);
            assert!(demand.expected == named());
            trace.borrow_mut().push(Event::Receiver(1));
            run.accept_receiver(resolved()).unwrap();
        }
        let report = run.finish::<Infallible>().unwrap();
        assert_eq!(report.inline_extent, S::from_f64(60.0));
        assert_eq!(report.block_extent, S::from_f64(35.0));
        assert_eq!(report.atomics.len(), 6);
        for (actual, (source, x, y)) in report.atomics.iter().zip([
            (0, 0.0, 0.0),
            (1, 0.0, 7.0),
            (2, 0.0, 14.0),
            (3, 0.0, 21.0),
            (4, 0.0, 28.0),
            (5, 30.0, 28.0),
        ]) {
            assert_eq!(actual.item.source_index, source);
            assert_eq!(
                expect_logical_position(actual.position),
                LogicalPointOf::new(S::from_f64(x), S::from_f64(y))
            );
        }
        use surgeist_text::LineEnd::{Forced, Paragraph, Soft};
        assert_eq!(
            report
                .formatted_lines
                .as_ref()
                .unwrap()
                .iter()
                .map(|line| (line.source_end, line.descriptor_end, line.ending))
                .collect::<Vec<_>>(),
            [
                (1, 1, Forced),
                (2, 2, Forced),
                (3, 3, Soft),
                (4, 4, Soft),
                (6, 6, Paragraph)
            ]
        );
        let baselines = report.baselines.numeric();
        assert_eq!(baselines.first, Some(S::from_f64(7.0)));
        assert_eq!(baselines.last, Some(S::from_f64(35.0)));
        assert!(baselines.first_named && baselines.last_named);
        assert_eq!(*trace.borrow(), expected_tail_trace(true));
        let synchronous_trace = RefCell::new(Vec::new());
        let synchronous = layout_mixed_inline_run_with_formatted_observation(
            query,
            |start, end| balanced_band(&synchronous_trace, start, end),
            |block, intent| clear(&synchronous_trace, block, intent),
            true,
            |source, reference| {
                assert_eq!(source, 1);
                assert!(reference == named());
                synchronous_trace.borrow_mut().push(Event::Receiver(source));
                Ok(resolved())
            },
        )
        .unwrap();
        assert_eq!(synchronous, report);
        assert_eq!(synchronous_trace.into_inner(), expected_tail_trace(true));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn tail_without_winner_restores_auto_formatted_receipts_without_another_walk() {
    fn lane<S: LayoutScalar>() {
        let query = tail_input::<S>(&[60.0, 10.0]);
        let trace = RefCell::new(Vec::new());
        let mut run = NumericRun::new(query, true).unwrap();
        while let Some(demand) = run
            .advance(
                |start, end| balanced_band(&trace, start, end),
                |block, intent| clear(&trace, block, intent),
            )
            .unwrap()
        {
            assert_eq!(demand.source_index, 1);
            trace.borrow_mut().push(Event::Receiver(1));
            run.accept_receiver(resolved()).unwrap();
        }
        let report = run.finish::<Infallible>().unwrap();
        assert_eq!(report.inline_extent, S::from_f64(60.0));
        assert_eq!(report.block_extent, S::from_f64(28.0));
        assert_eq!(report.atomics.len(), 4);
        for (source, atomic) in report.atomics.iter().enumerate() {
            assert_eq!(atomic.item.source_index, source);
            assert_eq!(
                expect_logical_position(atomic.position),
                LogicalPointOf::new(S::ZERO, S::from_f64(7.0 * source as f64))
            );
        }
        use surgeist_text::LineEnd::{Forced, Paragraph, Soft};
        assert_eq!(
            report
                .formatted_lines
                .as_ref()
                .unwrap()
                .iter()
                .map(|line| (line.source_end, line.descriptor_end, line.ending))
                .collect::<Vec<_>>(),
            [
                (1, 1, Forced),
                (2, 2, Forced),
                (3, 3, Soft),
                (4, 4, Paragraph)
            ]
        );
        assert_eq!(report.baselines.numeric().first, Some(S::from_f64(7.0)));
        assert_eq!(report.baselines.numeric().last, Some(S::from_f64(28.0)));
        assert!(report.baselines.numeric().first_named && report.baselines.numeric().last_named);
        assert_eq!(*trace.borrow(), expected_tail_trace(false));
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn losing_tail_trial_preserves_provider_error_after_frozen_prefix() {
    let trace = RefCell::new(Vec::new());
    let queries = Cell::new(0);
    let mut run = NumericRun::new(tail_input::<f64>(&[60.0, 30.0, 30.0, 10.0]), true).unwrap();
    loop {
        let next = run.advance(
            |start, end| {
                trace.borrow_mut().push(Event::Band(start, end));
                queries.set(queries.get() + 1);
                // Five discovery lines, three baseline lines, then the first
                // goal's frozen prefix precede this actual pair-band query.
                if queries.get() == 10 {
                    return Err("losing Tail pair band");
                }
                Ok(LogicalLineBandQueryResultOf {
                    inline_start: 0.0,
                    inline_end: 60.0,
                    next_transition: None,
                })
            },
            |block, intent| clear(&trace, block, intent),
        );
        match next {
            Ok(Some(demand)) => {
                assert_eq!(demand.source_index, 1);
                trace.borrow_mut().push(Event::Receiver(1));
                run.accept_receiver(resolved()).unwrap();
            }
            Err(NumericRunError::Band(error)) => {
                assert_eq!(error, "losing Tail pair band");
                break;
            }
            _ => panic!("the actual losing Tail provider error must propagate"),
        }
    }
    assert_eq!(*trace.borrow(), expected_tail_trace::<f64>(true)[..22]);
}

#[test]
fn tail_baseline_and_winner_provider_basis_mismatches_are_typed_refusals() {
    // Deliberate provider basis drift exercises the defensive replay proof
    // through real requests, rather than fabricating a private owner state.
    // Receiver2 starts baseline; receiver20 starts the selected winner replay.
    for changed_receiver in [2, 20] {
        let receivers = Cell::new(0);
        let mut run = NumericRun::new(tail_input::<f64>(&[30.0, 30.0, 10.0]), true).unwrap();
        loop {
            let next = run.advance(
                |_, _| {
                    Ok::<_, Infallible>(LogicalLineBandQueryResultOf {
                        inline_start: 0.0,
                        inline_end: if receivers.get() >= changed_receiver {
                            50.0
                        } else {
                            60.0
                        },
                        next_transition: None,
                    })
                },
                |block, _| block,
            );
            match next {
                Ok(Some(demand)) => {
                    assert_eq!(demand.source_index, 1);
                    receivers.set(receivers.get() + 1);
                    run.accept_receiver(resolved()).unwrap();
                }
                Err(NumericRunError::Numeric(NumericWrappingError::InvalidTailBasis)) => {
                    assert_eq!(receivers.get(), changed_receiver);
                    break;
                }
                _ => panic!(
                    "a changed replay basis must refuse instead of becoming a candidate miss"
                ),
            }
        }
    }
}
