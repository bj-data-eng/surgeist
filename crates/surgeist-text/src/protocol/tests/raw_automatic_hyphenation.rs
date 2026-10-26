//! Actual raw automatic/manual hyphenation and residual-overflow choices.
//! Existing raw System::layout, not the native CSS count profile. Source cuts
//! come from the qualified en-US exception table. W60's furthest-fitting choice
//! is the selected Surgeist greedy policy, not a CSS-prescribed coordinate.
use super::*;
use crate::{HyphenateCharacter, Hyphens, LineHeight, PlaintextItem, Range};
use skrifa::MetadataProvider;

fn raw_auto_font(primary_hyphen: bool) -> Vec<u8> {
    // Author only cmap over the licensed Ahem outlines/hmtx. Letters map to
    // actual Ahem A35; U2010/B36 or U002D/15 supplies a real one-em indication.
    // The optional98+99 GSUB sequence cannot match any input mapped here.
    let mut mappings = vec![(0x20, 3), (0x2d, 15), (0xad, 3)];
    mappings.extend((b'a'..=b'z').map(|cp| (u32::from(cp), 35)));
    if primary_hyphen {
        mappings.push((0x2010, 36));
    }
    crate::shaping_test_fonts::range_ligature_ahem(&[98, 99], &mappings)
}

fn raw_auto_style(hyphens: Hyphens, locale: Option<&str>) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        hyphens,
        locale: locale.map(str::to_owned),
        hyphenate_character: HyphenateCharacter::Auto,
        wrap: crate::Wrap::Word,
        overflow_wrap: crate::OverflowWrap::Normal,
        ..Style::default()
    }
}

#[derive(Clone, Debug)]
struct ExpectedRawLine {
    source: Range,
    advance: f64,
    indication: Option<usize>,
}

fn expected_line(
    start: usize,
    end: usize,
    advance: f64,
    indication: Option<usize>,
) -> ExpectedRawLine {
    ExpectedRawLine {
        source: Range::new(start, end),
        advance,
        indication,
    }
}

fn raw_auto_glyphs(layout: &crate::Layout, bytes: &[u8]) -> Vec<(usize, Range, u32, f32)> {
    for run in layout.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            layout.key().font_catalogue()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
    }
    let mut rows = Vec::new();
    for run in layout.glyph_runs() {
        rows.extend(
            run.glyphs()
                .iter()
                .filter(|glyph| glyph.advance() != 0.0)
                .map(|glyph| (run.line(), glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    rows.sort_by_key(|(line, range, _, _)| (*line, range.start, range.end));
    rows
}

fn raw_auto_advances(layout: &crate::Layout) -> Vec<f64> {
    let mut advances = vec![0.0; layout.lines().len()];
    for run in layout.inline_metric_facts().runs() {
        advances[run.line()] += run.advance();
    }
    advances
}

fn expected_raw_glyphs(
    source: &[u16],
    lines: &[ExpectedRawLine],
    indication: u32,
) -> Vec<(usize, Range, u32, f32)> {
    let mut expected = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        for (local, &cp) in source[line.source.start..line.source.end]
            .iter()
            .enumerate()
        {
            let at = line.source.start + local;
            if cp != 0xad {
                assert!((u16::from(b'a')..=u16::from(b'z')).contains(&cp));
                expected.push((line_index, Range::new(at, at + 1), 35, 10.0));
            }
        }
        if let Some(at) = line.indication {
            expected.push((line_index, Range::new(at, at), indication, 10.0));
        }
    }
    expected
}

fn raw_auto_case(
    source_text: &str,
    hyphens: Hyphens,
    locale: Option<&str>,
    width: f64,
    primary_hyphen: bool,
    expected: &[ExpectedRawLine],
    overflow: bool,
) {
    let bytes = raw_auto_font(primary_hyphen);
    // Independent actual primary cmap qualification: no expected indication
    // comes from the implementation's chosen glyph or adjacent fallback face.
    let font = skrifa::FontRef::new(&bytes).unwrap();
    let cmap = font.charmap();
    for cp in source_text.chars().filter(|cp| *cp != '\u{ad}') {
        assert!(cp.is_ascii_lowercase());
        assert_eq!(cmap.map(u32::from(cp)).unwrap().to_u32(), 35);
    }
    assert_eq!(cmap.map(0x2d_u32).unwrap().to_u32(), 15);
    assert_eq!(
        cmap.map(0x2010_u32).map(|glyph| glyph.to_u32()),
        primary_hyphen.then_some(36)
    );
    let indication = if primary_hyphen { 36 } else { 15 };
    let source = Source::identified(source_text, crate::Id::from_u64(6602), 9);
    let original = source_text.encode_utf16().collect::<Vec<_>>();
    let own = raw_auto_style(hyphens, locale);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    // Shape the actual nominal indication as ordinary authored source. This
    // proves its real glyph/advance without requiring working SHY behavior as
    // a precondition for any Auto test.
    let indication_source = Source::new(if primary_hyphen { "\u{2010}" } else { "-" });
    let nominal = system
        .layout(
            indication_source.clone(),
            own.clone(),
            crate::Options {
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    assert_eq!(nominal.source(), &indication_source);
    assert_eq!(raw_auto_advances(&nominal), [10.0]);
    assert_eq!(
        raw_auto_glyphs(&nominal, &bytes),
        [(0, Range::new(0, 1), indication, 10.0)]
    );
    let unbroken = system
        .layout(
            source.clone(),
            own.clone(),
            crate::Options {
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    let count = original.iter().filter(|cp| **cp != 0xad).count();
    let whole = [expected_line(0, original.len(), count as f64 * 10.0, None)];
    let whole_glyphs = expected_raw_glyphs(&original, &whole, indication);
    // These actual unbroken source/resource/provider controls precede any
    // missing-automatic or SHY-priority assertion. No manual-layout precondition.
    assert_eq!(unbroken.source(), &source);
    assert_eq!(unbroken.source().id(), Some(crate::Id::from_u64(6602)));
    assert_eq!(unbroken.source().revision(), 9);
    assert_eq!(unbroken.source().text().code_units(), original);
    assert_eq!(
        unbroken
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, original.len())]
    );
    assert_eq!(raw_auto_advances(&unbroken), [count as f64 * 10.0]);
    assert_eq!(raw_auto_glyphs(&unbroken, &bytes), whole_glyphs);
    let processed = crate::process_source_whitespace(
        source.clone(),
        own.clone(),
        crate::DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    let mut plain = Vec::new();
    for item in processed.plaintext_items(crate::Affinity::Before) {
        match item {
            PlaintextItem::Text(text) => plain.extend_from_slice(text.code_units()),
            PlaintextItem::InlineBox(_) => panic!("fixture has no inline boxes"),
        }
    }
    assert_eq!(plain, original);
    let retained_key = unbroken.key();
    let wrapped = system
        .layout(
            source.clone(),
            own,
            crate::Options {
                width: Some(width),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    drop(system);
    assert_eq!(unbroken.key(), retained_key);
    assert_eq!(unbroken.source(), &source);
    assert_eq!(raw_auto_glyphs(&unbroken, &bytes), whole_glyphs);
    assert_eq!(raw_auto_advances(&unbroken), [count as f64 * 10.0]);
    assert_eq!(wrapped.source(), &source);
    assert_eq!(wrapped.source().text().code_units(), original);
    let actual = (
        wrapped
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        raw_auto_advances(&wrapped),
        raw_auto_glyphs(&wrapped, &bytes),
    );
    let expected_facts = (
        expected.iter().map(|line| line.source).collect::<Vec<_>>(),
        expected.iter().map(|line| line.advance).collect::<Vec<_>>(),
        expected_raw_glyphs(&original, expected, indication),
    );
    assert_eq!(
        actual, expected_facts,
        "{source_text:?}/{hyphens:?}/{locale:?}/W{width}/primary{primary_hyphen}"
    );
    assert_eq!(
        wrapped.metrics().full_width(),
        expected
            .iter()
            .map(|line| line.advance)
            .fold(0.0_f64, f64::max)
    );
    assert_eq!(wrapped.metrics().overflow(), overflow);
    assert_eq!(wrapped.source().text().code_units(), original);
}

#[test]
fn raw_auto_table_takes_the_real_source_exception_and_primary_indication() {
    raw_auto_case(
        "table",
        Hyphens::Auto,
        Some("en-US"),
        30.0,
        true,
        &[
            expected_line(0, 2, 30.0, Some(2)),
            expected_line(2, 5, 30.0, None),
        ],
        false,
    );
}

#[test]
fn raw_auto_table_takes_the_real_source_exception_and_fallback_indication() {
    raw_auto_case(
        "table",
        Hyphens::Auto,
        Some("en-US"),
        30.0,
        false,
        &[
            expected_line(0, 2, 30.0, Some(2)),
            expected_line(2, 5, 30.0, None),
        ],
        false,
    );
}

#[test]
fn raw_auto_associate_uses_the_furthest_fitting_exception_cut() {
    raw_auto_case(
        "associate",
        Hyphens::Auto,
        Some("en-US"),
        50.0,
        true,
        &[
            expected_line(0, 4, 50.0, Some(4)),
            expected_line(4, 9, 50.0, None),
        ],
        false,
    );
}

#[test]
fn raw_auto_prefers_explicit_shy_while_its_seventy_pixel_residual_fits() {
    for primary in [true, false] {
        raw_auto_case(
            "as\u{ad}sociate",
            Hyphens::Auto,
            Some("en-US"),
            80.0,
            primary,
            &[
                expected_line(0, 3, 30.0, Some(3)),
                expected_line(3, 10, 70.0, None),
            ],
            false,
        );
    }
}

#[test]
fn raw_auto_uses_a_residual_automatic_cut_only_after_the_shy_partition_is_too_wide() {
    raw_auto_case(
        "as\u{ad}sociate",
        Hyphens::Auto,
        Some("en-US"),
        60.0,
        true,
        &[
            expected_line(0, 5, 50.0, Some(5)),
            expected_line(5, 10, 50.0, None),
        ],
        false,
    );
}

#[test]
fn raw_auto_residual_fallback_keeps_hyphen_minus_in_the_actual_primary_font() {
    raw_auto_case(
        "as\u{ad}sociate",
        Hyphens::Auto,
        Some("en-US"),
        60.0,
        false,
        &[
            expected_line(0, 5, 50.0, Some(5)),
            expected_line(5, 10, 50.0, None),
        ],
        false,
    );
}

#[test]
fn raw_manual_does_not_gain_automatic_residual_cuts_from_an_en_us_locale() {
    raw_auto_case(
        "as\u{ad}sociate",
        Hyphens::Manual,
        Some("en-US"),
        60.0,
        true,
        &[
            expected_line(0, 3, 30.0, Some(3)),
            expected_line(3, 10, 70.0, None),
        ],
        true,
    );
}

#[test]
fn raw_unknown_language_keeps_explicit_shy_without_inventing_automatic_cuts() {
    raw_auto_case(
        "as\u{ad}sociate",
        Hyphens::Auto,
        None,
        60.0,
        true,
        &[
            expected_line(0, 3, 30.0, Some(3)),
            expected_line(3, 10, 70.0, None),
        ],
        true,
    );
}

#[test]
fn raw_none_suppresses_the_explicit_and_automatic_hyphenation_endings() {
    raw_auto_case(
        "as\u{ad}sociate",
        Hyphens::None,
        Some("en-US"),
        60.0,
        true,
        &[expected_line(0, 10, 90.0, None)],
        true,
    );
}

#[test]
fn raw_nonautomatic_or_unqualified_language_words_keep_honest_unbroken_overflow() {
    for (hyphens, locale) in [
        (Hyphens::Manual, Some("en-US")),
        (Hyphens::None, Some("en-US")),
        (Hyphens::Auto, None),
        (Hyphens::Auto, Some("en")),
        (Hyphens::Auto, Some("en-GB")),
    ] {
        raw_auto_case(
            "associate",
            hyphens,
            locale,
            50.0,
            true,
            &[expected_line(0, 9, 90.0, None)],
            true,
        );
    }
}

#[test]
fn raw_auto_does_not_hyphenate_an_ordinary_fitting_whole_word() {
    raw_auto_case(
        "associate",
        Hyphens::Auto,
        Some("en-US"),
        90.0,
        true,
        &[expected_line(0, 9, 90.0, None)],
        false,
    );
}

#[test]
fn raw_manual_retains_its_real_unconstrained_count_operation() {
    raw_auto_case(
        "a\u{ad}bc",
        Hyphens::Manual,
        None,
        20.0,
        true,
        &[
            expected_line(0, 2, 20.0, Some(2)),
            expected_line(2, 4, 20.0, None),
        ],
        false,
    );
}
