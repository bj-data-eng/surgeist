//! Automatic fallback and real SHY priority through public Layout computation.
//!
//! Mount in text_layout::tests. The chosen 4-letter automatic prefix follows
//! Surgeist's greedy furthest-fitting policy, not a CSS-mandated numeric choice.
use super::*;
use surgeist_text::{
    HyphenateCharacter, HyphenateLimitChars, HyphenationCounts, HyphenationMinimum, Hyphens,
    LineEnd, OverflowWrap, PreparedBreakKind, PreparedTermination, Range, Text,
};

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const SOURCE_ID: u64 = 6602;
const SOURCE_REVISION: u64 = 9;

fn automatic_paragraph(source: &str) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    // A known, specified 10px glyph isolates priority from auto-indication
    // convention, primary-font probing and fallback coverage.
    style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
    style.wrap = Wrap::Word;
    style.overflow_wrap = OverflowWrap::Normal;
    PreparedParagraph::plain(
        1,
        Source::identified(
            source,
            surgeist_text::Id::from_u64(SOURCE_ID),
            SOURCE_REVISION,
        ),
        style,
    )
}

fn expected_letters(source: &str, selected: Range) -> Vec<(Range, u32, f32)> {
    // Independent retained Ahem cmap/hmtx facts: units/em=1000, each of these
    // glyphs advances 1000 font units. The cmap maps A to 35, a to 67, c to 69,
    // e to 71, i to 75, o to 81, s to 85 and t to 86.
    // Original UTF16 positions include the genuine SHY; inserted indications
    // never appear in this authored-source inventory.
    source
        .encode_utf16()
        .enumerate()
        .filter(|(index, cp)| selected.start <= *index && *index < selected.end && *cp != 0xad)
        .map(|(index, cp)| {
            let glyph = match cp {
                0x41 => 35,
                0x61 => 67,
                0x63 => 69,
                0x65 => 71,
                0x69 => 75,
                0x6f => 81,
                0x73 => 85,
                0x74 => 86,
                _ => panic!("fixture uses only independently qualified Ahem letters"),
            };
            (Range::new(index, index + 1), glyph, 10.0)
        })
        .collect()
}

fn glyphs(line: &surgeist_text::FinalizedLine) -> Vec<(Range, u32, f32)> {
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

fn compute<S: LayoutScalar>(
    paragraph: &PreparedParagraph,
    width: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
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
        )
        .input(1, paragraph.text_input(1));
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
    let batch = compute_layout_with_text(&capture, 0, request(width)).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    assert_eq!(batch.placed_text()[0].nodes(), &[1]);
    batch
}

fn finalized_lines<S: LayoutScalar>(
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
        .map(|line| line.expect("each authored line retains its actual origin fragment"))
        .collect()
}

fn qualify_unbroken<S: LayoutScalar>(paragraph: &PreparedParagraph, source: &str, advance: f64) {
    let original = source.encode_utf16().collect::<Vec<_>>();
    let analyzed = paragraph.prepared.analyzed_source();
    assert_eq!(analyzed.source().text().code_units(), original);
    assert_eq!(analyzed.style().hyphens, Hyphens::Auto);
    assert_eq!(analyzed.style().locale.as_deref(), Some("en-US"));
    assert_eq!(analyzed.style().wrap, Wrap::Word);
    assert_eq!(analyzed.style().overflow_wrap, OverflowWrap::Normal);
    assert_eq!(
        analyzed.style().hyphenate_character,
        HyphenateCharacter::String(Text::from("A"))
    );
    let origin = paragraph.origin(1);
    assert_eq!(
        origin.view().source().id(),
        Some(surgeist_text::Id::from_u64(SOURCE_ID))
    );
    assert_eq!(origin.view().source().revision(), SOURCE_REVISION);
    assert_eq!(origin.view().source().text().code_units(), original);

    // Qualify actual whole-word finalization through the same public compute
    // front before any intended constrained-choice assertion can fail.
    let batch = compute::<S>(paragraph, 100.0);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), original);
    let placed_lines = placed.lines();
    assert_eq!(placed_lines.len(), 1);
    assert_eq!(placed_lines[0].range(), Range::new(0, original.len()));
    assert_eq!(placed_lines[0].baseline(), 8.0);
    let lines = finalized_lines(&batch, paragraph);
    let whole = &lines[0];
    assert_eq!(
        whole.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert!(whole.selected_line().request().alternative.is_none());
    assert_eq!(
        glyphs(whole),
        expected_letters(source, Range::new(0, original.len()))
    );
    assert_eq!(
        (
            whole.feedback().advance,
            whole.feedback().baseline,
            whole.feedback().line_height
        ),
        (advance, 8.0, 10.0)
    );
}

fn assert_choice<S: LayoutScalar>(source: &str, width: f64, cut: usize, manual: bool) {
    // Independently qualify the actual specified indication glyph before the
    // real-word constrained call. It is authored only in this separate control.
    let indication = automatic_paragraph("A");
    qualify_unbroken::<S>(&indication, "A", 10.0);
    let paragraph = automatic_paragraph(source);
    qualify_unbroken::<S>(&paragraph, source, 90.0);
    let batch = compute::<S>(&paragraph, width);
    let placed = batch.placed_text()[0].layout();
    let original = source.encode_utf16().collect::<Vec<_>>();
    assert_eq!(placed.source().text().code_units(), original);
    assert_eq!(
        paragraph.origin(1).view().source().text().code_units(),
        original
    );
    let placed_lines = placed.lines();
    // The source exception associate=>2,4 and Ahem's one-em advances make
    // the cut after four letters the furthest fitting automatic choice here.
    // A real SHY after as wins at width 80 because its seven-letter residual fits.
    // At width 60 that residual is 70px and the automatic cut is permitted.
    assert_eq!(placed_lines.len(), 2, "{source:?} at width{width}");
    let expected_ranges = [Range::new(0, cut), Range::new(cut, original.len())];
    for (index, line) in placed_lines.iter().enumerate() {
        assert_eq!(line.range(), expected_ranges[index]);
        assert_eq!(line.baseline(), 8.0 + index as f64 * 10.0);
        assert_eq!(
            line.bounds(),
            surgeist_text::Rect::new(0.0, index as f64 * 10.0, width, 10.0)
        );
    }
    let lines = finalized_lines(&batch, &paragraph);
    for (index, line) in lines.iter().enumerate() {
        let mut expected = expected_letters(source, expected_ranges[index]);
        if index == 0 {
            expected.push((Range::new(cut, cut), 35, 10.0));
        }
        assert_eq!(glyphs(line), expected);
        let advance = if index == 0 && manual { 30.0 } else { 50.0 };
        let advance = if index == 1 && manual { 70.0 } else { advance };
        assert_eq!(line.feedback().advance, advance);
        assert_eq!(
            (line.feedback().occupied_start, line.feedback().occupied_end),
            (0.0, advance)
        );
        assert_eq!(
            line.selected_line().request().context.available_inline,
            Some(width)
        );
    }
    let selected = lines[0].selected_line();
    assert_eq!(selected.request().context.end, LineEnd::Soft);
    let PreparedTermination::Ordinary(offer) = &selected.request().termination else {
        panic!("a genuine hyphenation offer precedes arbitrary emergency cuts")
    };
    assert_eq!(offer.view().kind(), PreparedBreakKind::Discretionary);
    assert_eq!(offer.view().boundary().view().source_before().get(), cut);
    let alternative = selected
        .request()
        .alternative
        .as_ref()
        .expect("selected real indication");
    assert_eq!(alternative.view().text(), &Text::from("A"));
    assert_eq!(alternative.view().insertion_anchor().get(), cut);
    assert_eq!(
        alternative.view().source_range(),
        if manual { Some(Range::new(2, 3)) } else { None },
        "automatic insertion consumes no authored interval; a real SHY owns its actual range"
    );
    let continuation = lines[1].selected_line();
    assert_eq!(continuation.request().context.end, LineEnd::Paragraph);
    assert!(continuation.request().alternative.is_none());
    let root = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 0)
        .unwrap()
        .output();
    assert_eq!(root.size, Size::new(S::from_f64(width), S::from_f64(20.0)));
}

#[test]
fn native_auto_prefers_real_shy_when_its_residual_fits_f32() {
    assert_choice::<f32>("as\u{ad}sociate", 80.0, 3, true);
}

#[test]
fn native_auto_prefers_real_shy_when_its_residual_fits_f64() {
    assert_choice::<f64>("as\u{ad}sociate", 80.0, 3, true);
}

#[test]
fn native_auto_uses_resource_cut_when_shy_residual_is_overlong_f32() {
    assert_choice::<f32>("as\u{ad}sociate", 60.0, 5, false);
}

#[test]
fn native_auto_uses_resource_cut_when_shy_residual_is_overlong_f64() {
    assert_choice::<f64>("as\u{ad}sociate", 60.0, 5, false);
}

#[test]
fn native_auto_without_shy_uses_actual_resource_cut_f32() {
    assert_choice::<f32>("associate", 80.0, 4, false);
}

#[test]
fn native_auto_without_shy_uses_actual_resource_cut_f64() {
    assert_choice::<f64>("associate", 80.0, 4, false);
}

#[test]
fn native_character_minima_change_the_admitted_partition_and_real_line_choice() {
    fn check<S: LayoutScalar>() {
        let source = "as\u{ad}sociate";
        // Nine genuine letters: initial 5/2/2 admits SHY after two letters,
        // whose indicated prefix is 30px and residual is 70px, both fitting 80px.
        // Requiring four before removes that manual partition. The unbroken
        // word is 90px; source exception cut4 gives 50/50 with its 10px indication.
        // Original UTF16 includes SHY, so those first endpoints are 3 and 5.
        for (before, first_end, advances, manual) in [
            (HyphenationMinimum::Auto, 3, [30.0, 70.0], true),
            (HyphenationMinimum::Count(4), 5, [50.0, 50.0], false),
        ] {
            let limits = HyphenateLimitChars {
                before,
                ..HyphenateLimitChars::default()
            };
            let mut style = ahem_style(10.0, 10.0);
            style.hyphens = Hyphens::Auto;
            style.locale = Some("en-US".into());
            style.hyphenate_character = HyphenateCharacter::String(Text::from("A"));
            style.wrap = Wrap::Word;
            style.overflow_wrap = OverflowWrap::Normal;
            style.hyphenate_limit_chars = limits;
            let paragraph = PreparedParagraph::plain(
                1,
                Source::identified(
                    source,
                    surgeist_text::Id::from_u64(SOURCE_ID),
                    SOURCE_REVISION,
                ),
                style,
            );
            qualify_unbroken::<S>(&paragraph, source, 90.0);
            assert_eq!(
                paragraph
                    .prepared
                    .analyzed_source()
                    .style()
                    .hyphenate_limit_chars,
                limits
            );
            let batch = compute::<S>(&paragraph, 80.0);
            let layout = batch.placed_text()[0].layout();
            let ranges = [Range::new(0, first_end), Range::new(first_end, 10)];
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                ranges
            );
            assert_eq!(
                layout.source().text().code_units(),
                source.encode_utf16().collect::<Vec<_>>()
            );
            let lines = finalized_lines(&batch, &paragraph);
            for (index, line) in lines.iter().enumerate() {
                assert_eq!(line.feedback().advance, advances[index]);
                assert_eq!(
                    (line.feedback().occupied_start, line.feedback().occupied_end),
                    (0.0, advances[index])
                );
                let mut expected = expected_letters(source, ranges[index]);
                if index == 0 {
                    expected.push((Range::new(first_end, first_end), 35, 10.0));
                }
                assert_eq!(glyphs(line), expected);
            }
            let selected = lines[0].selected_line();
            let alternative = selected.request().alternative.as_ref().unwrap();
            assert_eq!(alternative.view().text(), &Text::from("A"));
            match (manual, alternative.view().definition()) {
                (true, surgeist_text::PreparedAlternativeDefinition::Source { source, .. }) => {
                    assert_eq!(source, Range::new(2, 3));
                }
                (false, surgeist_text::PreparedAlternativeDefinition::Automatic { cut }) => {
                    assert_eq!(
                        cut.view().counts(),
                        Some(HyphenationCounts {
                            total: 9,
                            before: 4,
                            after: 5
                        })
                    );
                    assert_eq!(alternative.view().source_range(), None);
                }
                other => panic!("wrong admitted ending: {other:?}"),
            }
            assert!(lines[1].selected_line().request().alternative.is_none());
        }
    }
    check::<f32>();
    check::<f64>();
}

fn configured(source: &str, indication: &str, overflow: OverflowWrap) -> PreparedParagraph {
    let mut style = ahem_style(10.0, 10.0);
    style.hyphens = Hyphens::Auto;
    style.locale = Some("en-US".into());
    style.hyphenate_character = HyphenateCharacter::String(Text::from(indication));
    style.overflow_wrap = overflow;
    PreparedParagraph::plain(1, Source::new(source), style)
}

#[test]
fn automatic_manual_partition_excludes_unrelated_prefix_capacity_and_includes_indication() {
    fn check<S: LayoutScalar>() {
        for (source, indication, width, expected_cut, advance, automatic) in [
            ("XX as\u{ad}sociate", "A", 80.0, 6, 60.0, false),
            ("assoc\u{ad}iate", "AA", 60.0, 4, 60.0, true),
        ] {
            let paragraph = configured(source, indication, OverflowWrap::Normal);
            let batch = compute::<S>(&paragraph, width);
            let lines = finalized_lines(&batch, &paragraph);
            assert_eq!(
                lines[0]
                    .selected_line()
                    .request()
                    .end
                    .view()
                    .source_before()
                    .get(),
                expected_cut
            );
            assert_eq!(lines[0].feedback().advance, advance);
            let selection = lines[0].selected_line();
            let alternative = selection.request().alternative.as_ref().unwrap();
            assert_eq!(
                alternative.view().text().code_units(),
                indication.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                matches!(
                    alternative.view().definition(),
                    surgeist_text::PreparedAlternativeDefinition::Automatic { .. }
                ),
                automatic
            );
            assert_eq!(
                batch.placed_text()[0].layout().source().text().code_units(),
                source.encode_utf16().collect::<Vec<_>>()
            );
            assert!(
                lines
                    .iter()
                    .flat_map(|line| line.run_views())
                    .all(|run| run.face().bytes() == AHEM)
            );
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn automatic_exhausted_variants_do_not_block_one_egc_emergency_lines() {
    fn check<S: LayoutScalar>() {
        for overflow in [OverflowWrap::Anywhere, OverflowWrap::BreakWord] {
            let paragraph = configured("associate", "A", overflow);
            let batch = compute::<S>(&paragraph, 10.0);
            let lines = finalized_lines(&batch, &paragraph);
            assert_eq!(lines.len(), 9);
            for (index, line) in lines.iter().enumerate() {
                let selection = line.selected_line();
                assert_eq!(selection.request().start.view().source_after().get(), index);
                assert_eq!(
                    selection.request().end.view().source_before().get(),
                    index + 1
                );
                assert!(selection.request().alternative.is_none());
                assert_eq!(line.feedback().advance, 10.0);
                assert_eq!(
                    glyphs(line),
                    expected_letters("associate", Range::new(index, index + 1))
                );
            }
        }
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn automatic_intrinsic_refinement_is_distinct_from_finite_manual_priority() {
    fn check<S: LayoutScalar>() {
        for source in ["associate", "as\u{ad}sociate"] {
            for (overflow, minimum) in [
                (OverflowWrap::Normal, 50.0),
                (OverflowWrap::BreakWord, 50.0),
                (OverflowWrap::Anywhere, 10.0),
            ] {
                let paragraph = configured(source, "A", overflow);
                let tree = PublicLayoutTreeOf::<S>::new()
                    .children(0, [1])
                    .children(1, [])
                    .style(
                        0,
                        NodeInputOf {
                            display: Display::Block,
                            ..NodeInputOf::default()
                        },
                    )
                    .input(1, paragraph.text_input(1));
                let capture =
                    TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0)]).unwrap();
                for (available, expected) in [
                    (AvailableOf::MIN_CONTENT, minimum),
                    (AvailableOf::MAX_CONTENT, 90.0),
                ] {
                    let request = LayoutRootRequestOf::viewport(Size::new(
                        available,
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap();
                    let batch = compute_layout_with_text(&capture, 0, request).unwrap();
                    let width = batch
                        .final_entries()
                        .iter()
                        .find(|entry| entry.node() == 0)
                        .unwrap()
                        .output()
                        .size
                        .width
                        .to_f64();
                    assert_eq!(width, expected, "{source:?}/{overflow:?}/{available:?}");
                    assert_eq!(
                        batch.placed_text()[0].layout().source().text().code_units(),
                        source.encode_utf16().collect::<Vec<_>>()
                    );
                }
            }
        }
    }
    check::<f32>();
    check::<f64>();
}
