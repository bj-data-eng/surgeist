//! Declared word-break:break-word through real native conditional fitting.
//! Ahem at 10px gives each authored A/SPACE a 10px raw advance.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const DECLARED_OVERFLOW: [text::OverflowWrap; 3] = [
    text::OverflowWrap::Normal,
    text::OverflowWrap::Anywhere,
    text::OverflowWrap::BreakWord,
];

#[derive(Clone, Copy)]
enum Ending {
    Conditional,
    Ordinary,
    Forced,
    Paragraph,
}

fn paragraph(
    source: &str,
    word_break: text::WordBreak,
    overflow_wrap: text::OverflowWrap,
    ancestor_wrap: Wrap,
    rtl: bool,
) -> PreparedParagraph {
    let child = text::Style {
        word_break,
        overflow_wrap,
        white_space: text::WhiteSpace::Preserve,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    let base = text::Style {
        wrap: ancestor_wrap,
        ..child.clone()
    };
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(
            1,
            Source::identified(source, text::Id::from_u64(65053), 7),
            child,
        )],
        base,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    );
    let analyzed = paragraph.prepared.analyzed_source();
    assert_eq!(analyzed.style().word_break, word_break);
    assert_eq!(analyzed.style().overflow_wrap, overflow_wrap);
    for span in analyzed.source().spans() {
        assert_eq!(span.style().word_break, word_break);
        assert_eq!(span.style().overflow_wrap, overflow_wrap);
    }
    paragraph
}

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: AvailableOf<S>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction: if rtl { Direction::Rtl } else { Direction::Ltr },
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn assert_output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
    source: &str,
    width: f64,
    expected: &[(text::Range, f64, f64, Ending)],
) {
    let root = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size.width, S::from_f64(width));
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    let units = source.encode_utf16().collect::<Vec<_>>();
    assert_eq!(placed.source().text().code_units(), units);
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65053)));
    assert_eq!(origin.view().source().revision(), 7);
    assert_eq!(origin.view().source().text().code_units(), units);
    assert_eq!(placed.lines().len(), expected.len());
    let mut finalized = vec![None; expected.len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        finalized[fragment.line_index()] = Some(line);
    }
    let mut baseline = 8.0;
    for (index, ((range, raw, fit, ending), line)) in expected.iter().zip(finalized).enumerate() {
        let line = line.expect("nonempty native line retains its actual origin fragment");
        assert_eq!(placed.lines()[index].range(), *range);
        assert_eq!(placed.lines()[index].baseline(), baseline);
        baseline += 10.0;
        assert_eq!(line.feedback().advance, *raw);
        assert_eq!(line.measurements().fit().advance(), *fit);
        let selected = line.selected_line();
        let request = selected.request();
        assert!(request.alternative.is_none());
        assert!(request.incoming.is_none());
        assert!(line.hyphenation_continuation().is_none());
        match (ending, &request.termination) {
            (Ending::Conditional, text::PreparedTermination::Conditional(opportunity)) => {
                assert_eq!(request.context.end, text::LineEnd::Soft);
                assert_eq!(opportunity.boundary(), request.end);
                assert_eq!(request.end.view().source_before().get(), range.end);
                assert_eq!(request.end.view().source_after().get(), range.end);
            }
            (ending, text::PreparedTermination::Ordinary(opportunity)) => {
                let (kind, end) = match ending {
                    Ending::Ordinary => (text::PreparedBreakKind::Allowed, text::LineEnd::Soft),
                    Ending::Forced => (text::PreparedBreakKind::Forced, text::LineEnd::Forced),
                    Ending::Paragraph => (
                        text::PreparedBreakKind::ParagraphEnd,
                        text::LineEnd::Paragraph,
                    ),
                    Ending::Conditional => panic!("emergency cut must retain conditional custody"),
                };
                assert_eq!(opportunity.view().kind(), kind);
                assert_eq!(request.context.end, end);
            }
            _ => panic!("selection must retain its ordinary or conditional authority"),
        }
        let mut finalized_advance = 0.0;
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            finalized_advance += run.advance();
            for glyph in run.glyphs() {
                assert_eq!(glyph.advance(), 10.0);
            }
        }
        assert_eq!(finalized_advance, *raw);
        let placed_advance: f64 = placed
            .inline_metric_facts()
            .runs()
            .iter()
            .filter(|run| run.line() == index)
            .map(|run| run.advance())
            .sum();
        assert_eq!(placed_advance, *raw);
    }
}

#[test]
fn break_word_uses_conditional_finite_and_min_content_cuts_for_every_declared_overflow() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let twenty = [
                (text::Range::new(0, 2), 20.0, 20.0, Ending::Conditional),
                (text::Range::new(2, 4), 20.0, 20.0, Ending::Paragraph),
            ];
            let forty = [(text::Range::new(0, 4), 40.0, 40.0, Ending::Paragraph)];
            let minimum = [
                (text::Range::new(0, 1), 10.0, 10.0, Ending::Conditional),
                (text::Range::new(1, 2), 10.0, 10.0, Ending::Conditional),
                (text::Range::new(2, 3), 10.0, 10.0, Ending::Conditional),
                (text::Range::new(3, 4), 10.0, 10.0, Ending::Paragraph),
            ];
            for overflow in DECLARED_OVERFLOW {
                let p = paragraph(
                    "AAAA",
                    text::WordBreak::BreakWord,
                    overflow,
                    Wrap::Word,
                    rtl,
                );
                // BreakWord preserves Normal ordinary analysis. Every interior
                // selection below must come from the actual conditional fitter.
                assert!(p.prepared.opportunity_views().all(|opportunity| {
                    opportunity.kind() == text::PreparedBreakKind::ParagraphEnd
                }));
                for (available, width, lines) in [
                    (AvailableOf::definite(S::from_f64(20.0)), 20.0, &twenty[..]),
                    (AvailableOf::definite(S::from_f64(40.0)), 40.0, &forty[..]),
                    (AvailableOf::MIN_CONTENT, 10.0, &minimum[..]),
                    (AvailableOf::MAX_CONTENT, 40.0, &forty[..]),
                ] {
                    let actual = compute::<S>(&p, available, rtl);
                    assert_output(&actual, &p, "AAAA", width, lines);
                }
            }
            // The two BreakWord properties differ: overflow-wrap:break-word
            // leaves min-content at the full word, while word-break:break-word
            // above contributes one 10px typographic unit.
            let overflow_only = paragraph(
                "AAAA",
                text::WordBreak::Normal,
                text::OverflowWrap::BreakWord,
                Wrap::Word,
                rtl,
            );
            let minimum = compute::<S>(&overflow_only, AvailableOf::MIN_CONTENT, rtl);
            assert_output(&minimum, &overflow_only, "AAAA", 40.0, &forty);
            let normal = paragraph(
                "AAAA",
                text::WordBreak::Normal,
                text::OverflowWrap::Normal,
                Wrap::Word,
                rtl,
            );
            let narrow = compute::<S>(&normal, AvailableOf::definite(S::from_f64(20.0)), rtl);
            assert_output(&narrow, &normal, "AAAA", 20.0, &forty);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn break_word_chooses_the_fitting_ordinary_space_before_emergency_interiors() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            for overflow in DECLARED_OVERFLOW {
                let p = paragraph(
                    "AA AA",
                    text::WordBreak::BreakWord,
                    overflow,
                    Wrap::Word,
                    rtl,
                );
                let narrow = compute::<S>(&p, AvailableOf::definite(S::from_f64(40.0)), rtl);
                assert_output(
                    &narrow,
                    &p,
                    "AA AA",
                    40.0,
                    &[
                        // The preserved trailing SPACE hangs: raw30, fit20.
                        (text::Range::new(0, 3), 30.0, 20.0, Ending::Ordinary),
                        (text::Range::new(3, 5), 20.0, 20.0, Ending::Paragraph),
                    ],
                );
                let wide = compute::<S>(&p, AvailableOf::definite(S::from_f64(50.0)), rtl);
                assert_output(
                    &wide,
                    &p,
                    "AA AA",
                    50.0,
                    &[(text::Range::new(0, 5), 50.0, 50.0, Ending::Paragraph)],
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn break_word_child_respects_ancestor_nowrap_and_preserves_forced_breaks() {
    fn lane<S: LayoutScalar>() {
        for rtl in [false, true] {
            let p = paragraph(
                "AAAA\nAA",
                text::WordBreak::BreakWord,
                text::OverflowWrap::Normal,
                Wrap::None,
                rtl,
            );
            let actual = compute::<S>(&p, AvailableOf::definite(S::from_f64(20.0)), rtl);
            assert_output(
                &actual,
                &p,
                "AAAA\nAA",
                20.0,
                &[
                    (text::Range::new(0, 5), 40.0, 40.0, Ending::Forced),
                    (text::Range::new(5, 7), 20.0, 20.0, Ending::Paragraph),
                ],
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}
