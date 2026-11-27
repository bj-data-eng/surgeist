//! Actual terminal LF retains distinct Forced and Paragraph completion receipts.
use super::*;
use crate::test_support::text::{PreparedParagraph, ahem_style};
use crate::{
    geometry::Size,
    node_input::{NodeInputOf, TextWrapStyle},
    sizing::PreferredSizeOf,
    value::{AvailableSpaceOf, AxisSizingRequestOf, SizingOperation},
};
use surgeist_text as text;

fn paragraph(raw: &str) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.white_space = text::style::WhiteSpace::PreserveBreaks;
    style.wrap = text::style::Wrap::Word;
    style.hyphens = text::hyphenation::Hyphens::None;
    PreparedParagraph::plain(1, text::source::Source::new(raw), style)
}

fn fit_terminal(paragraph: &PreparedParagraph, style: TextWrapStyle) -> Vec<FittedLine> {
    fit_with_terminal_context(paragraph, style, 60.0, TailContext::Unrestricted)
}

pub(in super::super) fn fit_with_terminal_context(
    paragraph: &PreparedParagraph,
    style: TextWrapStyle,
    available: f64,
    tail: TailContext,
) -> Vec<FittedLine> {
    fit_with_allocation(paragraph, style, available, tail, |_, _, _, struts| {
        Ok(LineAllocations {
            line_box_struts: struts.to_vec(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        })
    })
    .unwrap_or_else(|error| match error {
        FitError::Text(error) => panic!("actual Text preparation/finalization failed: {error:?}"),
        FitError::Wrapping(error) => panic!("actual wrapping input failed: {error:?}"),
        other => panic!("owning fit failed: {:?}", std::mem::discriminant(&other)),
    })
}

pub(in super::super) fn fit_with_allocation(
    paragraph: &PreparedParagraph,
    style: TextWrapStyle,
    available: f64,
    tail: TailContext,
    allocate: impl FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::allocation::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
) -> Result<Vec<FittedLine>, FitError> {
    let scope_alignments = crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
        crate::text_layout::scope_alignment::ScopeAlignmentSnapshot::<f64>::admit::<()>(
            Vec::new(),
            &[],
            false,
        )
        .unwrap()
        .freeze(),
        crate::text_layout::fragmentation::InlineBoxFamilies::default(),
    );
    super::super::fit(
        FitInput {
            meter: crate::accounting::Meter::new(),
            entry_funding: None,
            adjoining_displacement: 0.0,
            inline_boxes: None,
            groups: crate::group_alignment::Snapshot::default(),
            wrapping: None,
            formatted: &FormattedTailSnapshot::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::node_input::LineBoxDirectionPolicy::Normal,
                crate::node_input::Direction::Ltr,
            ),
            prepared: &paragraph.prepared,
            segment: paragraph.prepared.source_segments().next().unwrap(),
            scope_alignments: &scope_alignments,
            available: AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(
                    crate::value::NonNegativeFiniteOf::new(available).unwrap(),
                ),
                SizingOperation::Normal,
            ),
            base_direction: ParagraphBaseDirection::LeftToRight,
            block_direction: ParagraphBaseDirection::LeftToRight,
            alignment: TextAlign::Start,
            wrap_style: style,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: crate::value::LengthPercentageOf::ZERO,
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
        allocate,
        |_, _| Ok(None),
        EncounterCallbacks {
            replay: |_: &FinalizedLine, _: EncounterReplay| Ok(Vec::new()),
            accept: || Ok(()),
            reset: |_| Ok(()),
            checkpoint_group: || Ok(()),
        },
    )
}

fn terminal_pair(
    paragraph: &PreparedParagraph,
    length: usize,
) -> [text::prepared::PreparedOpportunity; 2] {
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
    expected: &[(text::source::Range, f64, LineEnd)],
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
                (line.flow.frame.logical_start, line.flow.end, line.flow.next),
                (top, top + 10.0, top + 10.0)
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
                        reason: text::prepared::ProtocolError::NonFormattedForcedLine
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
                (line.flow.frame.logical_start, line.flow.end, line.flow.next),
                (top, top, top)
            );
        }
        for run in line.finalized.run_views() {
            assert_eq!(
                run.face().bytes(),
                include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf")
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
            .map(|line| {
                line.flow.frame.logical_start + line.disposition.baseline(&line.finalized)
            })
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
    let auto = fit_terminal(&paragraph, TextWrapStyle::Auto);
    let auto_expected = [
        (text::source::Range::new(0, 7), 60.0, LineEnd::Soft),
        (text::source::Range::new(7, 9), 10.0, LineEnd::Forced),
        (text::source::Range::new(9, 9), 0.0, LineEnd::Paragraph),
    ];
    assert_completion(&paragraph, &auto, &auto_expected, 9);
    let balanced = fit_terminal(&paragraph, TextWrapStyle::Balance);
    let balanced_expected = [
        (text::source::Range::new(0, 4), 30.0, LineEnd::Soft),
        (text::source::Range::new(4, 9), 40.0, LineEnd::Forced),
        (text::source::Range::new(9, 9), 0.0, LineEnd::Paragraph),
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
        (text::source::Range::new(0, 1), 0.0, LineEnd::Forced),
        (text::source::Range::new(1, 1), 0.0, LineEnd::Paragraph),
    ];
    for style in [
        TextWrapStyle::Auto,
        TextWrapStyle::Balance,
        TextWrapStyle::AvoidShortLastLine,
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
        (text::source::Range::new(0, 7), 60.0, LineEnd::Soft),
        (text::source::Range::new(7, 9), 10.0, LineEnd::Forced),
        (text::source::Range::new(9, 9), 0.0, LineEnd::Paragraph),
    ];
    let auto = fit_terminal(&paragraph, TextWrapStyle::Auto);
    let repaired = fit_terminal(&paragraph, TextWrapStyle::AvoidShortLastLine);
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
    let style = text::style::Style {
        white_space: text::style::WhiteSpace::Collapse,
        wrap: text::style::Wrap::Word,
        hyphens: text::hyphenation::Hyphens::Manual,
        hyphenate_character: text::hyphenation::HyphenateCharacter::String("A".into()),
        ..ahem_style(10.0, 10.0)
    };
    // Real source SHY after XXXX SPACE as: Before indication A supplies10.
    // Ordinary permits80/70; complete-block Always requires40/90.
    let paragraph =
        PreparedParagraph::plain(1, text::source::Source::new("XXXX as\u{ad}sociate"), style);
    let manual = paragraph
        .prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::Discretionary)
        .unwrap();
    assert_eq!(manual.boundary().view().source_before().get(), 8);
    assert_eq!(manual.alternatives().count(), 1);
    for mode in [TextWrapStyle::Auto, TextWrapStyle::AvoidShortLastLine] {
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

    fn check<S: crate::scalar::LayoutScalar>() {
        let style = text::style::Style {
            white_space: text::style::WhiteSpace::PreserveBreaks,
            wrap: text::style::Wrap::Word,
            hyphens: text::hyphenation::Hyphens::None,
            ..ahem_style(10.0, 10.0)
        };
        for in_flow in [false, true] {
            let atomic = if in_flow {
                ParagraphPart::Atomic {
                    node: 2,
                    size: text::geometry::Size::new(0.0, 0.0),
                }
            } else {
                ParagraphPart::OutOfFlow {
                    node: 2,
                    size: text::geometry::Size::new(0.0, 0.0),
                }
            };
            let paragraph = PreparedParagraph::new(
                vec![
                    ParagraphPart::text(1, text::source::Source::new("AAA BB C\n"), style.clone()),
                    atomic,
                ],
                style.clone(),
                ParagraphBaseDirection::LeftToRight,
            );
            let binding = paragraph.binding(0);
            let occurrence = binding.atomics[0].occurrence.clone();
            assert_eq!(
                occurrence.view().original_box().id(),
                text::source::Id::from_u64(2)
            );
            assert_eq!(
                occurrence.view().original_box().size(),
                text::geometry::Size::new(0.0, 0.0)
            );
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, [1, 2])
                .children(1, [])
                .children(2, [])
                .style(
                    0,
                    NodeInputOf {
                        display: crate::node_input::Display::Block,
                        text_wrap_style: TextWrapStyle::AvoidShortLastLine,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(60.0)),
                            PreferredSizeOf::AUTO,
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .input(1, paragraph.text_input(1))
                .style(
                    2,
                    NodeInputOf {
                        display: crate::node_input::Display::InlineBlock,
                        position: if in_flow {
                            crate::node_input::Position::Relative
                        } else {
                            crate::node_input::Position::Absolute
                        },
                        size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
                        ..NodeInputOf::default()
                    },
                );
            let flow = crate::text_layout::LayoutFlowInput::try_new(
                &tree,
                0,
                vec![binding],
                crate::inline_wrapping::WrappingInput::default(),
                Vec::new(),
            )
            .unwrap();
            let mut session =
                crate::engine::ComputeSession::with_flow_unobserved(&flow, Vec::new());
            crate::engine::compute_root(&mut session, 0, {
                let width = crate::value::NonNegativeFiniteOf::new(S::from_f64(60.0)).unwrap();
                let root_input = crate::tree::LayoutTree::node_input(&tree, 0);
                // This is the actual fixture's Native NoParent frame. Its
                // declared percentage width supplies no independent ICB size.
                crate::output::LayoutRootRequestOf::standalone(
                    crate::value::SizingRequestOf::new(
                        Size::new(
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(width),
                                SizingOperation::Normal,
                            ),
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Indefinite,
                                SizingOperation::Normal,
                            ),
                        ),
                        crate::value::OrthogonalFallbackOf::new(
                            Size::NONE,
                            crate::value::NearestScrollportOf::Absent,
                            Size::splat(crate::value::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                    crate::geometry::FlowAxes::new(root_input.writing_mode, root_input.direction),
                    Size::new(Some(width), None),
                )
            })
            .unwrap();
            let state = session.text_computation().unwrap();
            assert_eq!(state.pending.len(), 1);
            let entries = &state.pending[0].entries;
            assert_eq!(entries.len(), 3);
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| matches!(entry, text::placement::PlacementEntry::Formatted(_)))
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
                matches!(receiver, text::placement::PlacementEntry::Formatted(_)),
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
                    text::allocation::AtomicLineAllocation::InFlow { .. }
                ),
                in_flow
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
