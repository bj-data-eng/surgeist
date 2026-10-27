//! Actual terminal LF retains distinct Forced and Paragraph completion receipts.
use super::*;
use crate::test_support::text::{PreparedParagraph, ahem_style};
use surgeist_text as text;

fn paragraph(raw: &str) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::WhiteSpace::PreserveBreaks;
    style.wrap = text::Wrap::Word;
    style.hyphens = text::Hyphens::None;
    PreparedParagraph::plain(1, text::Source::new(raw), style)
}

fn fit_terminal(paragraph: &PreparedParagraph, style: crate::TextWrapStyle) -> Vec<FittedLine> {
    fit_with_terminal_context(paragraph, style, 60.0, TailContext::Unrestricted)
}

fn fit_with_terminal_context(
    paragraph: &PreparedParagraph,
    style: crate::TextWrapStyle,
    available: f64,
    tail: TailContext,
) -> Vec<FittedLine> {
    super::super::fit(
        FitInput {
            groups: crate::group_alignment::Snapshot::default(),
            wrapping: None,
            formatted: &FormattedTailSnapshot::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                crate::Direction::Ltr,
            ),
            prepared: &paragraph.prepared,
            available: Available::Definite(available),
            base_direction: ParagraphBaseDirection::LeftToRight,
            block_direction: ParagraphBaseDirection::LeftToRight,
            alignment: TextAlign::Start,
            wrap_style: style,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::LengthPercentageOf::ZERO,
            tail,
            orientation: LineOrientation::Horizontal,
            atomics: &[],
            controls: &[],
            line_strut: None,
            reusable: &[],
        },
        |_, _| {
            Ok(Band {
                start: 0.0,
                end: available,
                intrinsic_end_occupation: None,
                next_transition: None,
            })
        },
        |position, _| {
            Ok(position
                .minimum
                .map_or(position.proposed, |minimum| position.proposed.max(minimum)))
        },
        |_, _, _, struts| {
            Ok(LineAllocations {
                line_box_struts: struts.to_vec(),
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            })
        },
        |_, _| Ok(None),
        EncounterCallbacks {
            replay: |_: &FinalizedLine, _: EncounterReplay| Ok(Vec::new()),
            accept: || Ok(()),
            reset: |_| {},
            checkpoint_group: || {},
        },
    )
    .unwrap_or_else(|error| match error {
        FitError::Text(error) => panic!("actual Text preparation/finalization failed: {error:?}"),
        FitError::Wrapping(error) => panic!("actual wrapping input failed: {error:?}"),
        other => panic!("owning fit failed: {:?}", std::mem::discriminant(&other)),
    })
}

fn terminal_pair(paragraph: &PreparedParagraph, length: usize) -> [text::PreparedOpportunity; 2] {
    let offers = paragraph
        .prepared
        .opportunity_views()
        .filter(|view| {
            matches!(
                view.kind(),
                PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
            )
        })
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(offers.len(), 2);
    assert_eq!(offers[0].view().kind(), PreparedBreakKind::Forced);
    assert_eq!(offers[1].view().kind(), PreparedBreakKind::ParagraphEnd);
    assert_ne!(offers[0], offers[1]);
    assert_eq!(offers[0].view().boundary(), offers[1].view().boundary());
    assert_eq!(
        offers[0].view().boundary().view().source_after().get(),
        length
    );
    assert_eq!(
        offers[0].view().boundary().prepared_text(),
        paragraph.prepared
    );
    [offers[0].clone(), offers[1].clone()]
}

fn assert_completion(
    paragraph: &PreparedParagraph,
    lines: &[FittedLine],
    expected: &[(text::Range, f64, LineEnd)],
    length: usize,
) {
    let [forced, paragraph_end] = terminal_pair(paragraph, length);
    let formatted_count = expected.len() - 1;
    assert_eq!(lines.len(), expected.len());
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.disposition.is_formatted())
            .count(),
        formatted_count
    );
    for (i, (line, (range, natural, ending))) in lines.iter().zip(expected).enumerate() {
        assert_eq!(line.finalized.prepared_text(), paragraph.prepared);
        let selected = line.finalized.selected_line();
        let request = selected.request();
        assert_eq!(request.context.end, *ending);
        assert_eq!(request.context.available_inline, Some(60.0));
        assert_eq!(request.start.view().source_after().get(), range.start);
        assert_eq!(request.end.view().source_after().get(), range.end);
        assert_eq!(line.natural_advance, *natural);
        assert_eq!(
            line.finalized.measurements().alignment().advance(),
            *natural
        );
        assert_eq!(line.finalized.feedback().advance, *natural);
        assert_eq!(line.finalized.feedback().baseline, 8.0);
        assert_eq!(line.finalized.feedback().line_height, 10.0);
        assert_eq!(line.first_formatted, i == 0);
        assert!(request.alternative.is_none());
        assert!(request.incoming.is_none());
        if i < formatted_count {
            assert!(matches!(line.disposition, LineDisposition::Formatted));
            assert_eq!(line.disposition.height(&line.finalized), 10.0);
            assert_eq!(line.disposition.baseline(&line.finalized), 8.0);
            let top = 10.0 * i as f64;
            assert_eq!(
                line.flow,
                LineFlow {
                    start: top,
                    end: top + 10.0,
                    next: top + 10.0
                }
            );
            if *ending == LineEnd::Forced {
                assert_eq!(
                    request.termination,
                    PreparedTermination::Ordinary(forced.clone())
                );
                let error = line.finalized.nonformatted().unwrap_err();
                assert!(matches!(
                    error.detail(),
                    Some(text::ErrorDetail::InvalidTextProtocol {
                        reason: text::ProtocolError::NonFormattedForcedLine
                    })
                ));
            } else {
                let PreparedTermination::Ordinary(offer) = &request.termination else {
                    panic!("actual soft endpoint capability required");
                };
                assert_eq!(offer.view().kind(), PreparedBreakKind::Allowed);
            }
        } else {
            let LineDisposition::NonFormatted(capability) = &line.disposition else {
                panic!("actual terminal Paragraph receipt must retain nonformatted eligibility");
            };
            assert_eq!(capability.finalized_line(), &line.finalized);
            assert_eq!(
                request.termination,
                PreparedTermination::Ordinary(paragraph_end.clone())
            );
            assert_eq!(request.start, forced.view().boundary());
            assert_eq!(request.end, paragraph_end.view().boundary());
            assert!(request.participants.is_empty());
            assert!(request.controls.is_empty());
            assert!(request.atomics.is_empty());
            assert!(!line.finalized.has_in_flow_content());
            assert_eq!(line.disposition.height(&line.finalized), 0.0);
            assert_eq!(line.disposition.baseline(&line.finalized), 0.0);
            let top = 10.0 * formatted_count as f64;
            assert_eq!(
                line.flow,
                LineFlow {
                    start: top,
                    end: top,
                    next: top
                }
            );
        }
        for run in line.finalized.run_views() {
            assert_eq!(
                run.face().bytes(),
                include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
            );
            assert_eq!(run.font_size(), 10.0);
            assert!(
                run.glyphs()
                    .all(|glyph| glyph.id() > 0 && glyph.advance() == 10.0)
            );
        }
    }
    assert_eq!(
        lines
            .iter()
            .map(|line| line.disposition.height(&line.finalized))
            .sum::<f64>(),
        10.0 * formatted_count as f64
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.disposition.is_formatted())
            .map(|line| { line.flow.start + line.disposition.baseline(&line.finalized) })
            .collect::<Vec<_>>(),
        (0..formatted_count)
            .map(|i| 8.0 + 10.0 * i as f64)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        lines.last().unwrap().flow.next,
        10.0 * formatted_count as f64
    );
}

#[test]
fn balance_terminal_source_lf_keeps_equal_boundary_paragraph_completion_nonformatted() {
    let paragraph = paragraph("AAA BB C\n");
    assert_eq!(
        paragraph
            .prepared
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        [65, 65, 65, 32, 66, 66, 32, 67, 10]
    );
    let auto = fit_terminal(&paragraph, crate::TextWrapStyle::Auto);
    let auto_expected = [
        (text::Range::new(0, 7), 60.0, LineEnd::Soft),
        (text::Range::new(7, 9), 10.0, LineEnd::Forced),
        (text::Range::new(9, 9), 0.0, LineEnd::Paragraph),
    ];
    assert_completion(&paragraph, &auto, &auto_expected, 9);
    let balanced = fit_terminal(&paragraph, crate::TextWrapStyle::Balance);
    let balanced_expected = [
        (text::Range::new(0, 4), 30.0, LineEnd::Soft),
        (text::Range::new(4, 9), 40.0, LineEnd::Forced),
        (text::Range::new(9, 9), 0.0, LineEnd::Paragraph),
    ];
    assert_completion(&paragraph, &balanced, &balanced_expected, 9);
    assert_completion(&paragraph, &auto, &auto_expected, 9);
}

#[test]
fn balance_source_lf_without_text_still_has_real_forced_line_and_nonformatted_tail() {
    let paragraph = paragraph("\n");
    assert_eq!(
        paragraph
            .prepared
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        [10]
    );
    let expected = [
        (text::Range::new(0, 1), 0.0, LineEnd::Forced),
        (text::Range::new(1, 1), 0.0, LineEnd::Paragraph),
    ];
    for style in [
        crate::TextWrapStyle::Auto,
        crate::TextWrapStyle::Balance,
        crate::TextWrapStyle::AvoidShortLastLine,
    ] {
        let lines = fit_terminal(&paragraph, style);
        assert_completion(&paragraph, &lines, &expected, 1);
        assert!(
            lines
                .iter()
                .all(|line| !line.finalized.has_in_flow_content())
        );
        assert_eq!(
            lines
                .iter()
                .map(|line| line
                    .finalized
                    .run_views()
                    .map(|run| run.glyphs().count())
                    .sum::<usize>())
                .sum::<usize>(),
            0
        );
    }
}

#[test]
fn paragraph_tail_leaves_terminal_forced_group_and_phantom_receipt_ordinary() {
    let paragraph = paragraph("AAA BB C\n");
    let expected = [
        (text::Range::new(0, 7), 60.0, LineEnd::Soft),
        (text::Range::new(7, 9), 10.0, LineEnd::Forced),
        (text::Range::new(9, 9), 0.0, LineEnd::Paragraph),
    ];
    let auto = fit_terminal(&paragraph, crate::TextWrapStyle::Auto);
    let repaired = fit_terminal(&paragraph, crate::TextWrapStyle::AvoidShortLastLine);
    assert_completion(&paragraph, &auto, &expected, 9);
    assert_completion(&paragraph, &repaired, &expected, 9);
    assert!(
        repaired
            .iter()
            .all(|line| matches!(line.program.as_deref(), Some(SelectionProgram::Tail(_))))
    );
}

#[test]
fn paragraph_tail_cannot_reactivate_manual_penultimate_indication_excluded_by_always() {
    let style = text::Style {
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        hyphens: text::Hyphens::Manual,
        hyphenate_character: text::HyphenateCharacter::String("A".into()),
        ..ahem_style(10.0, 10.0)
    };
    // Real source SHY after XXXX SPACE as: Before indication A supplies10.
    // Ordinary permits80/70; genuine complete-block Always requires40/90.
    let paragraph = PreparedParagraph::plain(1, text::Source::new("XXXX as\u{ad}sociate"), style);
    let manual = paragraph
        .prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    assert_eq!(manual.boundary().view().source_before().get(), 8);
    assert_eq!(manual.alternatives().count(), 1);
    for mode in [
        crate::TextWrapStyle::Auto,
        crate::TextWrapStyle::AvoidShortLastLine,
    ] {
        for (tail, expected) in [
            (
                TailContext::Unrestricted,
                [(0, 8, 80.0, true), (8, 15, 70.0, false)],
            ),
            (
                TailContext::CompleteBlock,
                [(0, 5, 40.0, false), (5, 15, 90.0, false)],
            ),
        ] {
            let lines = fit_with_terminal_context(&paragraph, mode, 100.0, tail);
            assert_eq!(lines.len(), 2);
            for (index, (line, (start, end, natural, indication))) in
                lines.iter().zip(expected).enumerate()
            {
                let selected = line.finalized.selected_line();
                let request = selected.request();
                assert_eq!(line.finalized.prepared_text(), paragraph.prepared);
                assert_eq!(request.start.view().source_after().get(), start);
                assert_eq!(request.end.view().source_after().get(), end);
                assert_eq!(request.context.available_inline, Some(100.0));
                assert_eq!(
                    request.context.end,
                    if index == 0 {
                        LineEnd::Soft
                    } else {
                        LineEnd::Paragraph
                    }
                );
                assert_eq!(line.natural_advance, natural);
                assert_eq!(request.alternative.is_some(), indication);
                assert!(request.incoming.is_none());
                assert!(line.disposition.is_formatted());
                let PreparedTermination::Ordinary(offer) = &request.termination else {
                    panic!("a quality goal cannot authorize an overflow-only stage");
                };
                assert_eq!(offer.view().boundary(), request.end);
                assert!(
                    paragraph
                        .prepared
                        .opportunity_views()
                        .any(|view| view.handle() == *offer)
                );
                if indication {
                    assert_eq!(*offer, manual.handle());
                }
            }
        }
    }
}

#[test]
fn paragraph_tail_retains_zero_atomic_existence_without_formatting_an_out_of_flow_receiver() {
    use crate::engine::contracts::Compute;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    use crate::test_support::text::ParagraphPart;

    fn check<S: crate::LayoutScalar>() {
        let style = text::Style {
            white_space: text::WhiteSpace::PreserveBreaks,
            wrap: text::Wrap::Word,
            hyphens: text::Hyphens::None,
            ..ahem_style(10.0, 10.0)
        };
        for in_flow in [false, true] {
            let atomic = if in_flow {
                ParagraphPart::Atomic {
                    node: 2,
                    size: text::Size::new(0.0, 0.0),
                }
            } else {
                ParagraphPart::OutOfFlow {
                    node: 2,
                    size: text::Size::new(0.0, 0.0),
                }
            };
            let paragraph = PreparedParagraph::new(
                vec![
                    ParagraphPart::text(1, text::Source::new("AAA BB C\n"), style.clone()),
                    atomic,
                ],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
            );
            let binding = paragraph.binding(0);
            let occurrence = binding.atomics[0].occurrence.clone();
            assert_eq!(occurrence.view().original_box().id(), text::Id::from_u64(2));
            assert_eq!(
                occurrence.view().original_box().size(),
                text::Size::new(0.0, 0.0)
            );
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2])
                .children(1, [])
                .children(2, [])
                .style(
                    0,
                    crate::NodeInputOf {
                        display: crate::Display::Block,
                        text_wrap_style: crate::TextWrapStyle::AvoidShortLastLine,
                        size: crate::Size::new(
                            crate::PreferredSizeOf::px(S::from_f64(60.0)),
                            crate::PreferredSizeOf::AUTO,
                        ),
                        ..crate::NodeInputOf::default()
                    },
                )
                .input(1, paragraph.text_input(1))
                .style(
                    2,
                    crate::NodeInputOf {
                        display: crate::Display::InlineBlock,
                        position: if in_flow {
                            crate::Position::Relative
                        } else {
                            crate::Position::Absolute
                        },
                        size: crate::Size::new(
                            crate::PreferredSizeOf::px(S::ZERO),
                            crate::PreferredSizeOf::px(S::ZERO),
                        ),
                        ..crate::NodeInputOf::default()
                    },
                );
            let flow = crate::LayoutFlowInput::try_new(
                &tree,
                0,
                vec![binding],
                crate::WrappingInput::default(),
            )
            .unwrap();
            let mut session = crate::engine::ComputeSession::with_flow(&flow, Vec::new());
            crate::engine::compute_root(
                &mut session,
                0,
                crate::Size::new(
                    crate::AvailableOf::Definite(S::from_f64(60.0)),
                    crate::AvailableOf::MAX_CONTENT,
                ),
            )
            .unwrap();
            let state = session.text_computation().unwrap();
            assert_eq!(state.pending.len(), 1);
            let entries = &state.pending[0].entries;
            assert_eq!(entries.len(), 3);
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| matches!(entry, text::PlacementEntry::Formatted(_)))
                    .count(),
                if in_flow { 3 } else { 2 }
            );
            for (entry, natural) in entries.iter().take(2).zip([60.0, 10.0]) {
                assert_eq!(
                    entry.finalized_line().measurements().alignment().advance(),
                    natural
                );
            }
            let receiver = entries.last().unwrap();
            assert_eq!(
                matches!(receiver, text::PlacementEntry::Formatted(_)),
                in_flow
            );
            let finalized = receiver.finalized_line();
            let selected = finalized.selected_line();
            let request = selected.request();
            assert_eq!(finalized.prepared_text(), paragraph.prepared);
            assert_eq!(request.context.end, LineEnd::Paragraph);
            assert_eq!(request.start.view().source_after().get(), 9);
            assert_eq!(request.end.view().source_after().get(), 9);
            assert!(
                request.end.view().participant_offset() > request.start.view().participant_offset()
            );
            assert_eq!(finalized.has_in_flow_content(), in_flow);
            assert_eq!(finalized.measurements().alignment().advance(), 0.0);
            assert_eq!(request.atomics.len(), 1);
            assert_eq!(request.atomics[0].occurrence, occurrence);
            assert_eq!(
                matches!(
                    request.atomics[0].allocation,
                    text::AtomicLineAllocation::InFlow { .. }
                ),
                in_flow
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
