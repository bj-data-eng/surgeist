//! Native manual-hyphenation admission under the initial CSS count profile.
//! The selected initial minima are total5/before2/after2, including manual SHY.
//! These native Layout cases use empty indications so glyph coverage and added
//! hyphen width cannot hide the intended count-admission behavior.
use super::*;
use surgeist_text::{
    HyphenateCharacter, Hyphens, LineEnd, OverflowWrap, PreparedBreakKind, PreparedTermination,
    Range, Text,
};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn manual_paragraph(source: &str) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Manual;
    style.locale = None; // Explicit opportunities need no dictionary/language.
    style.hyphenate_character = HyphenateCharacter::String(Text::from(""));
    style.wrap = Wrap::Word;
    style.overflow_wrap = OverflowWrap::Normal;
    PreparedParagraph::plain(
        1,
        Source::identified(source, surgeist_text::Id::from_u64(6601), 7),
        style,
    )
}

fn expected_letters(source: &str) -> Vec<(Range, u32, f32)> {
    // Independently read from the retained Ahem cmap: a..f=>67..72. Its hmtx
    // has one-em advances; no GSUB feature substitutes these lowercase words.
    source
        .encode_utf16()
        .enumerate()
        .filter(|(_, cp)| *cp != 0xad)
        .map(|(i, cp)| {
            assert!((0x61..=0x66).contains(&cp));
            (Range::new(i, i + 1), u32::from(cp) - 30, 10.0)
        })
        .collect()
}

fn manual_glyphs(line: &surgeist_text::FinalizedLine) -> Vec<(Range, u32, f32)> {
    let mut actual = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
        actual.extend(
            run.glyphs()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    actual.sort_by_key(|(range, _, _)| (range.start, range.end));
    actual
}

fn manual_batch<S: LayoutScalar>(
    source: &str,
) -> (PreparedParagraph, CompletedLayoutBatchOf<u32, S>) {
    let paragraph = manual_paragraph(source);
    let analyzed = paragraph.prepared.analyzed_source();
    assert_eq!(
        analyzed.source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(analyzed.style().hyphens, Hyphens::Manual);
    assert_eq!(analyzed.style().locale, None);
    assert_eq!(analyzed.style().wrap, Wrap::Word);
    assert_eq!(analyzed.style().overflow_wrap, OverflowWrap::Normal);
    assert_eq!(
        analyzed.style().hyphenate_character,
        HyphenateCharacter::String(Text::from(""))
    );
    let origin = paragraph.origin(1);
    assert_eq!(
        origin.view().source().id(),
        Some(surgeist_text::Id::from_u64(6601))
    );
    assert_eq!(origin.view().source().revision(), 7);
    assert_eq!(
        origin.view().source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    // Actual native unconstrained finalization qualifies source/font/whole word
    // before the constrained compute can reach its intended admission failure.
    let whole =
        crate::test_support::text_fragment::fragment_for(&paragraph.prepared).finalized_line();
    assert_eq!(whole.prepared_text(), paragraph.prepared);
    assert_eq!(
        whole.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert!(whole.selected_line().request().alternative.is_none());
    let letters = expected_letters(source);
    assert_eq!(manual_glyphs(&whole), letters);
    assert_eq!(
        (
            whole.feedback().advance,
            whole.feedback().baseline,
            whole.feedback().line_height
        ),
        (letters.len() as f64 * 10.0, 8.0, 10.0)
    );
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::AUTO,
                ),
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
    let batch = compute_layout_with_flow(&capture, 0, request(20.0)).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].nodes(), &[1]);
    assert_eq!(
        batch.placed_text()[0].layout().source().text().code_units(),
        source.encode_utf16().collect::<Vec<_>>()
    );
    (paragraph, batch)
}

fn actual_lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    paragraph: &PreparedParagraph,
) -> Vec<surgeist_text::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let fragment = entry.fragment();
        let finalized = fragment.finalized_fragment().finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        if let Some(prior) = &lines[fragment.line_index()] {
            assert_eq!(prior, &finalized);
        }
        lines[fragment.line_index()] = Some(finalized);
    }
    lines
        .into_iter()
        .map(|line| line.expect("each nonempty authored line has a real origin fragment"))
        .collect()
}

fn assert_manual_suppressed<S: LayoutScalar>(source: &str, advance: f64) {
    let (paragraph, batch) = manual_batch::<S>(source);
    let placed = batch.placed_text()[0].layout();
    let placed_lines = placed.lines();
    // Width20 is actually too narrow for the unbroken word. Before correction
    // the real SHY is selected; after admission there is no emergency policy.
    assert_eq!(placed_lines.len(), 1, "CSS5/2/2 must suppress {source:?}");
    assert_eq!(
        placed_lines[0].range(),
        Range::new(0, source.encode_utf16().count())
    );
    assert_eq!(placed_lines[0].bounds().size.width, 20.0);
    assert_eq!(placed_lines[0].baseline(), 8.0);
    let lines = actual_lines(&batch, &paragraph);
    let line = &lines[0];
    assert_eq!(
        line.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert!(line.selected_line().request().alternative.is_none());
    let selected = line.selected_line();
    let PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("no conditional emergency ending is allowed")
    };
    assert_eq!(offer.view().kind(), PreparedBreakKind::ParagraphEnd);
    assert_eq!(line.feedback().advance, advance);
    assert_eq!(
        (line.feedback().occupied_start, line.feedback().occupied_end),
        (0.0, advance)
    );
    assert_eq!(manual_glyphs(line), expected_letters(source));
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(20.0), S::from_f64(10.0)));
}

#[test]
fn native_css_manual_total_minimum_suppresses_ab_shy_cd_f32() {
    assert_manual_suppressed::<f32>("ab\u{ad}cd", 40.0);
}
#[test]
fn native_css_manual_total_minimum_suppresses_ab_shy_cd_f64() {
    assert_manual_suppressed::<f64>("ab\u{ad}cd", 40.0);
}
#[test]
fn native_css_manual_before_minimum_suppresses_a_shy_bcdef_f32() {
    assert_manual_suppressed::<f32>("a\u{ad}bcdef", 60.0);
}
#[test]
fn native_css_manual_before_minimum_suppresses_a_shy_bcdef_f64() {
    assert_manual_suppressed::<f64>("a\u{ad}bcdef", 60.0);
}
#[test]
fn native_css_manual_after_minimum_suppresses_abcd_shy_e_f32() {
    assert_manual_suppressed::<f32>("abcd\u{ad}e", 50.0);
}
#[test]
fn native_css_manual_after_minimum_suppresses_abcd_shy_e_f64() {
    assert_manual_suppressed::<f64>("abcd\u{ad}e", 50.0);
}

fn assert_manual_qualified<S: LayoutScalar>() {
    let source = "ab\u{ad}cdef";
    let (paragraph, batch) = manual_batch::<S>(source);
    let placed_lines = batch.placed_text()[0].layout().lines();
    assert_eq!(placed_lines.len(), 2);
    assert_eq!(placed_lines[0].range(), Range::new(0, 3));
    assert_eq!(placed_lines[1].range(), Range::new(3, 7));
    assert_eq!(
        (placed_lines[0].baseline(), placed_lines[1].baseline()),
        (8.0, 18.0)
    );
    let lines = actual_lines(&batch, &paragraph);
    let first = lines[0].selected_line();
    assert_eq!(first.request().context.end, LineEnd::Soft);
    let PreparedTermination::Ordinary(offer) = &first.request().termination else {
        panic!("qualified SHY uses its real ordinary opportunity")
    };
    assert_eq!(offer.view().kind(), PreparedBreakKind::Discretionary);
    assert_eq!(offer.view().boundary().view().source_before().get(), 3);
    let alternative = first
        .request()
        .alternative
        .as_ref()
        .expect("taken SHY retains its actual alternative");
    assert_eq!(alternative.view().source_range(), Some(Range::new(2, 3)));
    assert!(alternative.view().text().is_empty());
    assert_eq!(lines[0].feedback().advance, 20.0);
    assert_eq!(lines[1].feedback().advance, 40.0);
    assert_eq!(
        lines[1].selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert!(lines[1].selected_line().request().alternative.is_none());
    let mut glyphs = manual_glyphs(&lines[0]);
    glyphs.extend(manual_glyphs(&lines[1]));
    assert_eq!(glyphs, expected_letters(source));
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(20.0), S::from_f64(20.0)));
}

#[test]
fn native_css_manual_qualified_ab_shy_cdef_keeps_its_real_empty_indication_f32() {
    assert_manual_qualified::<f32>();
}
#[test]
fn native_css_manual_qualified_ab_shy_cdef_keeps_its_real_empty_indication_f64() {
    assert_manual_qualified::<f64>();
}
