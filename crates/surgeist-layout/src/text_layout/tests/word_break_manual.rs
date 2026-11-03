//! Native Manual disables lexical fallback while preserving declared overflow.
//! Ahem has no Thai coverage: actual glyph0 has a1000-unit advance at1000 UPEM.
//! The real10px missing-glyph instance qualifies geometry, not script coverage.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ending {
    Ordinary,
    Conditional,
    Paragraph,
}

type LineFact = (text::Range, f64, f64, Ending);

fn paragraph(
    word_break: text::WordBreak,
    overflow_wrap: text::OverflowWrap,
    rtl: bool,
) -> PreparedParagraph {
    let style = text::Style {
        word_break,
        overflow_wrap,
        locale: Some("th".to_owned()),
        line_break: text::LineBreak::Auto,
        white_space: text::WhiteSpace::Preserve,
        hyphens: text::Hyphens::None,
        wrap: Wrap::Word,
        ..ahem_style(10.0, 10.0)
    };
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(
            1,
            Source::identified("กก", text::Id::from_u64(65406), 31),
            style.clone(),
        )],
        style,
        if rtl {
            ParagraphBaseDirection::RightToLeft
        } else {
            ParagraphBaseDirection::LeftToRight
        },
    );
    let analyzed = paragraph.prepared.analyzed_source();
    assert_eq!(analyzed.style().word_break, word_break);
    assert_eq!(analyzed.style().overflow_wrap, overflow_wrap);
    assert_eq!(analyzed.style().locale.as_deref(), Some("th"));
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
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

fn observe<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
) -> (f64, Vec<LineFact>) {
    let width = batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output()
        .size
        .width
        .to_f64();
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    let source = text::Text::from("กก");
    assert_eq!(placed.source().text(), &source);
    let origin = paragraph.origin(1);
    assert_eq!(origin.view().source().id(), Some(text::Id::from_u64(65406)));
    assert_eq!(origin.view().source().revision(), 31);
    assert_eq!(origin.view().source().text(), &source);
    let endpoint = |offset, affinity| {
        placed
            .positions()
            .find(|position| position.source_cursor() == text::Cursor::new(offset, affinity))
            .unwrap()
    };
    let copied = placed
        .position_selection(
            &endpoint(0, text::Affinity::After),
            &endpoint(2, text::Affinity::Before),
        )
        .unwrap();
    assert_eq!(copied.source_text(), &source);
    assert_eq!(copied.plaintext(), &source);
    let mut finalized = vec![None; placed.lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        assert_eq!(line.prepared_text(), paragraph.prepared);
        finalized[fragment.line_index()] = Some(line);
    }
    let mut glyphs = Vec::new();
    let lines = placed
        .lines()
        .iter()
        .zip(finalized)
        .enumerate()
        .map(|(index, (placed_line, line))| {
            let line = line.expect("each nonempty native line retains its actual origin");
            let mut raw = 0.0;
            for run in line.run_views() {
                let face = run.face();
                assert_eq!(face.bytes(), AHEM);
                assert_eq!(face.index(), 0);
                let instance = run.effective_instance().unwrap();
                assert_eq!(instance.font_size(), 10.0);
                assert_eq!(instance.axis_count(), 0);
                assert_eq!(
                    instance
                        .metrics(face.catalogue_identity())
                        .unwrap()
                        .units_per_em(),
                    1000
                );
                raw += run.advance();
                glyphs.extend(
                    run.glyphs()
                        .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
                );
            }
            assert_eq!(line.feedback().advance, raw);
            let placed_raw: f64 = placed
                .inline_metric_facts()
                .runs()
                .iter()
                .filter(|run| run.line() == index)
                .map(|run| run.advance())
                .sum();
            assert_eq!(placed_raw, raw);
            let selected = line.selected_line();
            let request = selected.request();
            assert!(request.alternative.is_none());
            assert!(request.incoming.is_none());
            assert!(line.hyphenation_continuation().is_none());
            let ending = match &request.termination {
                text::PreparedTermination::Conditional(opportunity) => {
                    assert_eq!(request.context.end, text::LineEnd::Soft);
                    assert_eq!(opportunity.start(), request.start);
                    assert_eq!(opportunity.boundary(), request.end);
                    Ending::Conditional
                }
                text::PreparedTermination::Ordinary(opportunity) => {
                    match opportunity.view().kind() {
                        text::PreparedBreakKind::Allowed => {
                            assert_eq!(request.context.end, text::LineEnd::Soft);
                            Ending::Ordinary
                        }
                        text::PreparedBreakKind::ParagraphEnd => {
                            assert_eq!(request.context.end, text::LineEnd::Paragraph);
                            Ending::Paragraph
                        }
                        _ => panic!("this source has no forced or hyphenation owner"),
                    }
                }
                text::PreparedTermination::Measurement(_)
                | text::PreparedTermination::PhraseMeasurement(_) => {
                    panic!("measurement-only trials cannot be placed")
                }
            };
            (
                placed_line.range(),
                raw,
                line.measurements().fit().advance(),
                ending,
            )
        })
        .collect();
    glyphs.sort_by_key(|glyph| glyph.0.start);
    assert_eq!(
        glyphs,
        [
            (text::Range::new(0, 1), 0, 10.0),
            (text::Range::new(1, 2), 0, 10.0),
        ]
    );
    (width, lines)
}

#[test]
fn manual_preserves_declared_overflow_for_real_finite_and_intrinsic_layout() {
    fn lane<S: LayoutScalar>() {
        use text::OverflowWrap::{Anywhere, BreakWord, Normal as OverflowNormal};
        use text::WordBreak::{Manual, Normal};
        let whole = vec![(text::Range::new(0, 2), 20.0, 20.0, Ending::Paragraph)];
        let split = |ending| {
            vec![
                (text::Range::new(0, 1), 10.0, 10.0, ending),
                (text::Range::new(1, 2), 10.0, 10.0, Ending::Paragraph),
            ]
        };
        for rtl in [false, true] {
            for (word, overflow, finite, minimum) in [
                (Normal, OverflowNormal, split(Ending::Ordinary), 10.0),
                (Manual, OverflowNormal, whole.clone(), 20.0),
                (Manual, Anywhere, split(Ending::Conditional), 10.0),
                (Manual, BreakWord, split(Ending::Conditional), 20.0),
            ] {
                let p = paragraph(word, overflow, rtl);
                let cuts = p
                    .prepared
                    .opportunity_views()
                    .filter(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
                    .map(|offer| offer.boundary().view().source_after().get())
                    .collect::<Vec<_>>();
                assert_eq!(cuts, if word == Normal { vec![1] } else { vec![] });
                let finite_batch = compute::<S>(&p, AvailableOf::definite(S::from_f64(10.0)), rtl);
                assert_eq!(observe(&finite_batch, &p), (10.0, finite.clone()));
                let minimum_batch = compute::<S>(&p, AvailableOf::MIN_CONTENT, rtl);
                assert_eq!(
                    observe(&minimum_batch, &p),
                    (
                        minimum,
                        if minimum == 10.0 {
                            finite
                        } else {
                            whole.clone()
                        }
                    )
                );
                let maximum_batch = compute::<S>(&p, AvailableOf::MAX_CONTENT, rtl);
                assert_eq!(observe(&maximum_batch, &p), (20.0, whole.clone()));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}
