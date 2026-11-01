//! Generated indications preserve original-source bidi resolution.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn paragraph(control: bool, indication: &str) -> PreparedParagraph {
    let style = text::Style {
        hyphens: text::Hyphens::Manual,
        hyphenate_character: text::HyphenateCharacter::String(indication.into()),
        ..ahem_style(10.0, 10.0)
    };
    if control {
        PreparedParagraph::new(
            vec![
                ParagraphPart::text(10, Source::new("\u{202e}AB\u{202c}_"), style.clone()),
                ParagraphPart::Control {
                    node: 20,
                    kind: text::ParagraphControlKind::DiscretionaryBreak {
                        style: Box::new(style.clone()),
                    },
                },
                ParagraphPart::text(11, Source::new("CDE"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        )
    } else {
        PreparedParagraph::plain(10, Source::new("\u{202e}AB\u{202c}_\u{ad}CDE"), style)
    }
}

fn original(control: bool) -> Vec<u16> {
    if control {
        "\u{202e}AB\u{202c}_CDE"
    } else {
        "\u{202e}AB\u{202c}_\u{ad}CDE"
    }
    .encode_utf16()
    .collect()
}

fn qualify(p: &PreparedParagraph, control: bool, indication: &str) {
    let prepared = &p.prepared;
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        original(control)
    );
    let cuts = prepared.hyphenation_cut_views().collect::<Vec<_>>();
    assert_eq!(cuts.len(), 1);
    assert_eq!(
        cuts[0].provenance(),
        if control {
            text::HyphenationProvenance::Control
        } else {
            text::HyphenationProvenance::Source
        }
    );
    assert_eq!(
        cuts[0].counts(),
        Some(text::HyphenationCounts {
            total: 5,
            before: 2,
            after: 3
        })
    );
    assert_eq!(
        cuts[0].opportunity().unwrap().view().kind(),
        text::PreparedBreakKind::Discretionary
    );
    let alternatives = prepared.alternative_views().collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 1);
    assert_eq!(
        alternatives[0].text().code_units(),
        indication.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        alternatives[0].insertion_anchor().get(),
        if control { 5 } else { 6 }
    );
    if control {
        assert!(matches!(
            alternatives[0].definition(),
            text::PreparedAlternativeDefinition::Control { .. }
        ));
        assert_eq!(prepared.control_views().next().unwrap().level().get(), 0);
    } else {
        assert_eq!(alternatives[0].source_range(), Some(text::Range::new(5, 6)));
    }
    // Original UBA: RLO makes AB level1; PDF returns to embedding0. The
    // neutral underscore's original neighbors are R and L, so N2 resolves it to L0.
    let underscore = prepared
        .unit_views()
        .find(|unit| unit.source_range() == text::Range::new(4, 5))
        .unwrap();
    assert_eq!(underscore.level().get(), 0);
}

fn compute<S: LayoutScalar>(p: &PreparedParagraph, width: f64) -> CompletedLayoutBatchOf<u32, S> {
    let binding = p.binding(0);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.clone())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(width)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
    for row in &binding.origins {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::inline_text(TextParticipation::new(row.origin.clone())),
        );
    }
    for row in &binding.controls {
        tree = tree.children(row.node, []).input(
            row.node,
            LayoutInputOf::text_control(TextControlParticipation::new(row.occurrence.clone())),
        );
    }
    let captured = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let result = compute_layout_with_flow(&captured, 0, request(width)).unwrap();
    assert_eq!(
        result.placed_text()[0]
            .layout()
            .source()
            .text()
            .code_units(),
        p.prepared.analyzed_source().source().text().code_units()
    );
    result
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut out = vec![None; batch.placed_text()[0].layout().lines().len()];
    for row in batch.final_inline_fragments() {
        let fragment = row.fragment();
        let line = fragment.finalized_fragment().finalized_line();
        if let Some(prior) = &out[fragment.line_index()] {
            assert_eq!(prior, &line);
        }
        out[fragment.line_index()] = Some(line);
    }
    out.into_iter().map(|line| line.unwrap()).collect()
}

fn glyphs(line: &text::FinalizedLine) -> Vec<(text::Range, u32, f32, f64)> {
    let mut out = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        for glyph in run.glyphs() {
            if glyph.advance() != 0.0 {
                out.push((
                    glyph.range(),
                    glyph.id(),
                    glyph.advance(),
                    run.origin().x + glyph.pen() + f64::from(glyph.x()),
                ));
            }
        }
    }
    out.sort_by_key(|(range, _, _, _)| (range.start, range.end));
    out
}

fn check<S: LayoutScalar>(control: bool) {
    let empty = paragraph(control, "");
    let rlm = paragraph(control, "\u{200f}");
    qualify(&empty, control, "");
    qualify(&rlm, control, "\u{200f}");
    let first_end = if control { 5 } else { 6 };
    let end = if control { 8 } else { 9 };
    let prefix = vec![
        (text::Range::new(1, 2), 35, 10.0, 10.0),
        (text::Range::new(2, 3), 36, 10.0, 0.0),
        (text::Range::new(4, 5), 65, 10.0, 20.0),
    ];
    let tail = vec![
        (text::Range::new(first_end, first_end + 1), 37, 10.0, 0.0),
        (
            text::Range::new(first_end + 1, first_end + 2),
            38,
            10.0,
            10.0,
        ),
        (
            text::Range::new(first_end + 2, first_end + 3),
            39,
            10.0,
            20.0,
        ),
    ];
    let mut selected = Vec::new();
    for p in [&empty, &rlm] {
        let whole = compute::<S>(p, 100.0);
        let whole_lines = lines(&whole);
        assert_eq!(whole_lines.len(), 1);
        assert_eq!(whole_lines[0].feedback().advance, 60.0);
        assert!(
            whole_lines[0]
                .selected_line()
                .request()
                .alternative
                .is_none()
        );
        let mut whole_expected = prefix.clone();
        whole_expected.extend(
            tail.iter()
                .map(|(range, id, advance, x)| (*range, *id, *advance, x + 30.0)),
        );
        assert_eq!(glyphs(&whole_lines[0]), whole_expected);
        let actual = compute::<S>(p, 40.0);
        let actual_lines = lines(&actual);
        assert_eq!(actual_lines.len(), 2);
        for (i, line) in actual_lines.iter().enumerate() {
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().occupied_start,
                    line.feedback().occupied_end
                ),
                (30.0, 0.0, 30.0)
            );
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                (8.0, 10.0)
            );
            assert_eq!(
                actual.placed_text()[0].layout().lines()[i].range(),
                [
                    text::Range::new(0, first_end),
                    text::Range::new(first_end, end)
                ][i]
            );
        }
        let alternative = actual_lines[0]
            .selected_line()
            .request()
            .alternative
            .clone()
            .unwrap();
        assert_eq!(
            alternative.view().text().code_units(),
            p.prepared
                .alternative_views()
                .next()
                .unwrap()
                .text()
                .code_units()
        );
        assert!(
            actual_lines[1]
                .selected_line()
                .request()
                .alternative
                .is_none()
        );
        assert_eq!(glyphs(&actual_lines[1]), tail);
        selected.push(actual_lines);
    }
    // Empty computed indication proves actual original-source L1/L2 order and
    // independent Ahem positions before the RLM behavior under test.
    assert_eq!(glyphs(&selected[0][0]), prefix);
    // Adopted local-domain contract: a zero-flow generated RLM cannot alter
    // original paragraph neutral resolution or any original glyph's position.
    // Full-variant UBA instead lets RLM resolve original neutral underscore to R by N1.
    assert_eq!(
        glyphs(&selected[1][0]),
        prefix,
        "generated RLM must not re-resolve original neutral underscore or reorder original source"
    );
}

#[test]
fn source_before_rlm_does_not_change_original_neutral_f32() {
    check::<f32>(false);
}
#[test]
fn source_before_rlm_does_not_change_original_neutral_f64() {
    check::<f64>(false);
}
#[test]
fn control_before_rlm_does_not_change_original_neutral_f32() {
    check::<f32>(true);
}
#[test]
fn control_before_rlm_does_not_change_original_neutral_f64() {
    check::<f64>(true);
}
