use super::*;

fn query<S: LayoutScalar>(widths: &[f64]) -> MixedInlineRunInputOf<S> {
    let mut query = tail_input(widths);
    query.text_wrap_style = crate::TextWrapStyle::Pretty;
    query
}

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

fn group<S: LayoutScalar>(events: &mut Vec<Event<S>>, counts: &[usize]) {
    events.push(Event::Receiver(1));
    for (index, &queries) in counts.iter().enumerate() {
        line(events, 14.0 + 7.0 * index as f64, queries);
    }
}

fn body_trace<S: LayoutScalar>() -> Vec<Event<S>> {
    let mut events = Vec::new();
    prefix(&mut events);
    group(&mut events, &[1, 1, 1]); // ordinary60/20/50; Tail ineligible
    // Fixed post-Tail min1/3/max1 produce17 interpolants, then mean2/3.
    // Adjacent-only dedup retains that appended repeated mean. Goals0..8
    // and mean choose30/30/20/50 and exceed count3; goal9 wins30/50/50;
    // goals10..16 retain60/20/50. The terminal formatted line is ordinary.
    // A quality line with4/3/2 candidate completions/prefixes queries each
    // twice and its selected band once, hence9/7/5 provider calls.
    for _ in 0..9 {
        group(&mut events, &[9, 7, 1, 1]);
    }
    group(&mut events, &[9, 7, 1]);
    for _ in 0..7 {
        group(&mut events, &[9, 5, 1]);
    }
    group(&mut events, &[9, 7, 1, 1]); // appended mean is not globally deduped
    group(&mut events, &[9, 7, 1]); // selected winner
    prefix(&mut events);
    group(&mut events, &[1, 1, 1]); // authentic ordinary program replay
    group(&mut events, &[9, 7, 1]); // selected Body program replay
    events
}

fn nested_tail_trace<S: LayoutScalar>() -> Vec<Event<S>> {
    let mut events = expected_tail_trace(true);
    // The repaired phase60/30/40 fixes all guards. Eighteen Body goals
    // either retain it or regress its terminal guards; none improves Body.
    for _ in 0..18 {
        group(&mut events, &[9, 7, 1]);
    }
    // Saved post-Tail phase is restored directly, with no fresh Auto walk.
    prefix(&mut events);
    group(&mut events, &[1, 1, 1]); // program ordinary60/60/10
    group(&mut events, &[1, 1, 1]); // nested Tail baseline with frozen60
    group(&mut events, &[1, 7, 5]); // nested Tail selected replay30/40
    events
}

fn check_report<S: LayoutScalar>(
    report: &MixedInlineRunReportOf<S>,
    atoms: &[(usize, f64, f64, f64)],
    cuts: &[(usize, surgeist_text::LineEnd)],
    extent: f64,
) {
    let height = 7.0 * cuts.len() as f64;
    assert_eq!(report.inline_extent, S::from_f64(extent));
    assert_eq!(report.block_extent, S::from_f64(height));
    assert_eq!(report.atomics.len(), atoms.len());
    for (actual, &(source, x, y, width)) in report.atomics.iter().zip(atoms) {
        assert_eq!(actual.item.source_index, source);
        assert_eq!(
            actual.item.size,
            Size::new(S::from_f64(width), S::from_f64(7.0))
        );
        assert_eq!(
            expect_logical_position(actual.position),
            LogicalPointOf::new(S::from_f64(x), S::from_f64(y))
        );
    }
    assert_eq!(
        report
            .formatted_lines
            .as_ref()
            .unwrap()
            .iter()
            .map(|line| (line.source_end, line.descriptor_end, line.ending))
            .collect::<Vec<_>>(),
        cuts.iter()
            .map(|&(source, ending)| (source, source, ending))
            .collect::<Vec<_>>()
    );
    assert_eq!(report.line_bands.len(), cuts.len());
    for (index, band) in report.line_bands.iter().enumerate() {
        assert_eq!(band.inline_start, S::ZERO);
        assert_eq!(band.inline_end, S::from_f64(60.0));
        assert_eq!(band.block_start, S::from_f64(7.0 * index as f64));
        assert_eq!(band.block_end, S::from_f64(7.0 * (index + 1) as f64));
    }
    let baselines = report.baselines.numeric();
    assert_eq!(baselines.first, Some(S::from_f64(7.0)));
    assert_eq!(baselines.last, Some(S::from_f64(height)));
    assert!(baselines.first_named && baselines.last_named);
}

fn run<S: LayoutScalar>(
    query: MixedInlineRunInputOf<S>,
) -> (MixedInlineRunReportOf<S>, Vec<Event<S>>) {
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
        assert!(demand.expected == named());
        trace.borrow_mut().push(Event::Receiver(1));
        run.accept_receiver(resolved()).unwrap();
    }
    (run.finish::<Infallible>().unwrap(), trace.into_inner())
}

#[test]
fn pretty_body_receiver_preserves_fixed_guards_winner_and_program_trace() {
    fn lane<S: LayoutScalar>() {
        let query = query::<S>(&[30.0, 30.0, 20.0, 50.0]);
        let (report, trace) = run(query.clone());
        use surgeist_text::LineEnd::{Forced, Paragraph, Soft};
        check_report(
            &report,
            &[
                (0, 0.0, 0.0, 10.0),
                (1, 0.0, 7.0, 10.0),
                (2, 0.0, 14.0, 30.0),
                (3, 0.0, 21.0, 30.0),
                (4, 30.0, 21.0, 20.0),
                (5, 0.0, 28.0, 50.0),
            ],
            &[
                (1, Forced),
                (2, Forced),
                (3, Soft),
                (5, Soft),
                (6, Paragraph),
            ],
            50.0,
        );
        assert_eq!(trace, body_trace());
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
        assert_eq!(synchronous_trace.into_inner(), body_trace());
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn pretty_nested_tail_receiver_retains_saved_phase_and_full_program_replay() {
    fn lane<S: LayoutScalar>() {
        let (report, trace) = run(query::<S>(&[60.0, 30.0, 30.0, 10.0]));
        use surgeist_text::LineEnd::{Forced, Paragraph, Soft};
        check_report(
            &report,
            &[
                (0, 0.0, 0.0, 10.0),
                (1, 0.0, 7.0, 10.0),
                (2, 0.0, 14.0, 60.0),
                (3, 0.0, 21.0, 30.0),
                (4, 0.0, 28.0, 30.0),
                (5, 30.0, 28.0, 10.0),
            ],
            &[
                (1, Forced),
                (2, Forced),
                (3, Soft),
                (4, Soft),
                (6, Paragraph),
            ],
            60.0,
        );
        assert_eq!(trace, nested_tail_trace());
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn losing_pretty_body_trial_preserves_exact_provider_error_after_receiver() {
    let trace = RefCell::new(Vec::new());
    let calls = Cell::new(0);
    let mut run = NumericRun::new(query::<f64>(&[30.0, 30.0, 20.0, 50.0]), true).unwrap();
    loop {
        match run.advance(
            |start, end| {
                trace.borrow_mut().push(Event::Band(start, end));
                calls.set(calls.get() + 1);
                // Five ordinary lines precede goal0, whose four-line candidate
                // cannot authenticate the real three-line reference count.
                if calls.get() == 6 {
                    return Err("losing Pretty Body band");
                }
                Ok(LogicalLineBandQueryResultOf {
                    inline_start: 0.0,
                    inline_end: 60.0,
                    next_transition: None,
                })
            },
            |block, intent| clear(&trace, block, intent),
        ) {
            Ok(Some(demand)) => {
                assert_eq!(demand.source_index, 1);
                trace.borrow_mut().push(Event::Receiver(1));
                run.accept_receiver(resolved()).unwrap();
            }
            Err(NumericRunError::Band(error)) => {
                assert_eq!(error, "losing Pretty Body band");
                break;
            }
            _ => panic!("the actual losing Body provider error must propagate"),
        }
    }
    assert_eq!(*trace.borrow(), body_trace::<f64>()[..13]);
}

#[test]
fn pretty_replay_basis_refusals_preserve_the_owning_pretty_or_tail_error() {
    // Actual provider-basis drift, not fabricated private owner states. Body
    // receiver20 is winner replay,21 program ordinary,22 program selected.
    for changed in [20, 21, 22] {
        let receivers = Cell::new(0);
        let mut run = NumericRun::new(query::<f64>(&[30.0, 30.0, 20.0, 50.0]), true).unwrap();
        loop {
            match run.advance(
                |_, _| {
                    Ok::<_, Infallible>(LogicalLineBandQueryResultOf {
                        inline_start: 0.0,
                        inline_end: if receivers.get() >= changed {
                            50.0
                        } else {
                            60.0
                        },
                        next_transition: None,
                    })
                },
                |block, _| block,
            ) {
                Ok(Some(demand)) => {
                    assert_eq!(demand.source_index, 1);
                    receivers.set(receivers.get() + 1);
                    run.accept_receiver(resolved()).unwrap();
                }
                Err(NumericRunError::Numeric(NumericWrappingError::InvalidPrettyBasis)) => {
                    assert_eq!(receivers.get(), changed);
                    break;
                }
                _ => panic!("a changed Pretty replay basis must be a typed refusal"),
            }
        }
    }
    // Nested Tail authentication remains Tail-owned. Its baseline is receiver40;
    // selected replay is41. A valid improved but changed phase is Pretty-owned.
    for (changed, phase_only) in [(40, false), (41, false), (41, true)] {
        let receivers = Cell::new(0);
        let mut run = NumericRun::new(query::<f64>(&[60.0, 30.0, 30.0, 10.0]), true).unwrap();
        loop {
            let next = run.advance(
                |start, _| {
                    Ok::<_, Infallible>(LogicalLineBandQueryResultOf {
                        inline_start: 0.0,
                        inline_end: if receivers.get() < changed {
                            60.0
                        } else if phase_only {
                            if start < 21.0 { 60.0 } else { 66.0 }
                        } else {
                            50.0
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
                Err(NumericRunError::Numeric(error)) => {
                    assert_eq!(receivers.get(), changed);
                    assert!(matches!(
                        (changed, error),
                        (40, NumericWrappingError::InvalidTailBasis)
                            | (41, NumericWrappingError::InvalidPrettyBasis)
                    ));
                    break;
                }
                _ => panic!("the changed nested Tail or phase basis must retain its owner error"),
            }
        }
    }
}
