//! BreakAll source rules retain complete TCUs, mandatory glue and strictness.
use super::*;
use crate::test_fonts as sfnt;

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));
const MAP: &[(u32, u32)] = &[
    (0x21, 35),
    (0x28, 35),
    (0x29, 36),
    (0x41, 35),
    (0x42, 36),
    (0xa0, 3),
    (0xb2, 58),
    (0x5b0, 100),
    (0x5d0, 58),
    (0x5d1, 58),
    (0x660, 35),
    (0x661, 36),
    (0xe01, 35),
    (0xe48, 100),
    (0x200d, 253),
    (0x2010, 15),
    (0x2013, 15),
    (0x202f, 3),
    (0x2060, 252),
    (0x2160, 35),
    (0x3041, 35),
    (0x4e00, 58),
];

fn source_system() -> System {
    // Only the actual cmap changes. These are licensed retained Ahem outlines,
    // not claims about Hebrew/Thai glyph designs. Real mapping coverage keeps
    // the public opportunity test independent of missing-glyph fallback.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend(12_u16.to_be_bytes());
    cmap.extend(0_u16.to_be_bytes());
    cmap.extend(u32::try_from(16 + 12 * MAP.len()).unwrap().to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(u32::try_from(MAP.len()).unwrap().to_be_bytes());
    for &(cp, glyph) in MAP {
        cmap.extend(cp.to_be_bytes());
        cmap.extend(cp.to_be_bytes());
        cmap.extend(glyph.to_be_bytes());
    }
    let bytes = sfnt::replace_tables(AHEM, vec![(*b"cmap", cmap)]);
    for tag in [*b"hhea", *b"hmtx", *b"glyf", *b"loca", *b"maxp"] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(AHEM, tag));
    }
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.maxp().unwrap().num_glyphs(), 281);
    assert!(parsed.glyf().is_ok());
    assert!(parsed.table_data(skrifa::Tag::new(b"fvar")).is_none());
    for &(cp, glyph) in MAP {
        assert_eq!(parsed.charmap().map(cp), Some(GlyphId::new(glyph)));
    }
    assert_eq!(
        parsed
            .glyph_metrics(FontSize::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(35)),
        Some(1000.0)
    );
    system_with(bytes)
}

fn prepare_source(
    system: &mut System,
    raw: &str,
    word_break: crate::WordBreak,
    line_break: crate::LineBreak,
) -> PreparedText {
    let prepared = capture(
        system,
        raw,
        Style {
            word_break,
            line_break,
            wrap: crate::Wrap::Word,
            ..own_style()
        },
        raw.contains('\u{5d0}'),
        CW,
    );
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        raw.encode_utf16().collect::<Vec<_>>()
    );
    prepared
}

fn cuts(prepared: &PreparedText) -> Vec<usize> {
    let mut result = prepared
        .opportunity_views()
        .filter(|offer| offer.kind() == PreparedBreakKind::Allowed)
        .map(|offer| offer.boundary().view().source_after().get())
        .collect::<Vec<_>>();
    result.sort_unstable();
    result
}

#[test]
fn hebrew_marks_stay_inside_the_letter_tcu_and_glue_joiners_stay_unbreakable() {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let classes = CodePointMapData::<UnicodeLineBreak>::new();
    assert_eq!(categories.get('\u{5b0}'), GeneralCategory::NonspacingMark);
    assert_eq!(classes.get('\u{5b0}'), UnicodeLineBreak::CombiningMark);
    assert_eq!(classes.get('\u{5d1}'), UnicodeLineBreak::HebrewLetter);
    assert_eq!(categories.get('\u{e48}'), GeneralCategory::NonspacingMark);
    assert_eq!(classes.get('\u{e48}'), UnicodeLineBreak::ComplexContext);
    assert_eq!(classes.get('\u{a0}'), UnicodeLineBreak::Glue);
    assert_eq!(classes.get('\u{202f}'), UnicodeLineBreak::Glue);
    assert_eq!(classes.get('\u{2060}'), UnicodeLineBreak::WordJoiner);
    let mut system = source_system();
    for (raw, graphemes, cut) in [
        (
            "\u{5d0}\u{5b0}\u{5d1}",
            vec![Range::new(0, 2), Range::new(2, 3)],
            2,
        ),
        (
            "\u{5d0}\u{5b0}\u{5b0}\u{5d0}",
            vec![Range::new(0, 3), Range::new(3, 4)],
            3,
        ),
    ] {
        for rule in [
            crate::WordBreak::Normal,
            crate::WordBreak::KeepAll,
            crate::WordBreak::BreakAll,
        ] {
            let prepared = prepare_source(&mut system, raw, rule, crate::LineBreak::Strict);
            assert_eq!(
                prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
                graphemes
            );
            assert_eq!(
                cuts(&prepared),
                if rule == crate::WordBreak::BreakAll {
                    vec![cut]
                } else {
                    vec![]
                },
                "{raw:?} {rule:?}"
            );
        }
    }
    // SA normal/KeepAll may use lexical resources. Their particular word
    // segmentation is not this oracle: every rule must preserve the marked
    // TCU, while BreakAll must additionally offer its following boundary.
    for rule in [
        crate::WordBreak::Normal,
        crate::WordBreak::KeepAll,
        crate::WordBreak::BreakAll,
    ] {
        let prepared = prepare_source(
            &mut system,
            "\u{e01}\u{e48}\u{e01}",
            rule,
            crate::LineBreak::Strict,
        );
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 2), Range::new(2, 3)]
        );
        assert!(!cuts(&prepared).contains(&1), "{rule:?}");
        if rule == crate::WordBreak::BreakAll {
            assert_eq!(cuts(&prepared), [2]);
        }
    }
    // LB8a/LB11/LB12 retain actual ZWJ/WJ/GL protection; BreakAll changes
    // letter classification, rather than becoming an Anywhere operation.
    for raw in [
        "\u{5d0}\u{200d}\u{5d0}",
        "\u{5d0}\u{2060}\u{5d0}",
        "\u{5d0}\u{a0}\u{5d0}",
        "\u{5d0}\u{202f}\u{5d0}",
        "(\u{5d0}",
        "\u{5d0})",
        "\u{5d0}!",
    ] {
        for strictness in [
            crate::LineBreak::Loose,
            crate::LineBreak::Normal,
            crate::LineBreak::Strict,
        ] {
            let prepared = prepare_source(&mut system, raw, crate::WordBreak::BreakAll, strictness);
            assert!(cuts(&prepared).is_empty(), "{raw:?} {strictness:?}");
        }
    }
}

#[test]
fn break_all_effective_id_before_unicode_hyphens_is_a_loose_only_opportunity() {
    // Pinned Text3 §5.2 explicitly permits these before-hyphen cuts only for
    // Loose after ID, including letters treated as ID by BreakAll. Raw Hebrew
    // HL and Latin AL do not independently satisfy that condition.
    let classes = CodePointMapData::<UnicodeLineBreak>::new();
    assert_eq!(classes.get('\u{5d0}'), UnicodeLineBreak::HebrewLetter);
    assert_eq!(classes.get('A'), UnicodeLineBreak::Alphabetic);
    let mut system = source_system();
    for prefix in ['\u{5d0}', 'A'] {
        for hyphen in ['\u{2010}', '\u{2013}'] {
            let raw = format!("{prefix}{hyphen}{prefix}");
            for rule in [
                crate::WordBreak::Normal,
                crate::WordBreak::KeepAll,
                crate::WordBreak::BreakAll,
            ] {
                for strictness in [
                    crate::LineBreak::Loose,
                    crate::LineBreak::Normal,
                    crate::LineBreak::Strict,
                ] {
                    let prepared = prepare_source(&mut system, &raw, rule, strictness);
                    assert_eq!(
                        cuts(&prepared).contains(&1),
                        rule == crate::WordBreak::BreakAll && strictness == crate::LineBreak::Loose,
                        "{raw:?} {rule:?} {strictness:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn latin_sa_and_general_letter_number_units_keep_source_based_break_all_semantics() {
    // Text3 §1.4/Appendix E defines letter units using all L/N categories,
    // rather than only ASCII letters or decimal digits. These classifications
    // are positively checked independently of glyph IDs and font advances.
    let categories = CodePointMapData::<GeneralCategory>::new();
    let classes = CodePointMapData::<UnicodeLineBreak>::new();
    assert_eq!(categories.get('\u{b2}'), GeneralCategory::OtherNumber);
    assert_eq!(categories.get('\u{2160}'), GeneralCategory::LetterNumber);
    assert_eq!(categories.get('\u{660}'), GeneralCategory::DecimalNumber);
    assert_eq!(categories.get('\u{3041}'), GeneralCategory::OtherLetter);
    assert_eq!(
        classes.get('\u{3041}'),
        UnicodeLineBreak::ConditionalJapaneseStarter
    );
    assert_eq!(classes.get('\u{e01}'), UnicodeLineBreak::ComplexContext);
    let mut system = source_system();
    for raw in [
        "AB",
        "\u{b2}\u{b2}",
        "\u{2160}\u{2160}",
        "\u{660}\u{661}",
        "\u{3041}\u{3041}",
    ] {
        for rule in [
            crate::WordBreak::Normal,
            crate::WordBreak::KeepAll,
            crate::WordBreak::BreakAll,
        ] {
            let prepared = prepare_source(&mut system, raw, rule, crate::LineBreak::Strict);
            assert_eq!(
                prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(1, 2)]
            );
            assert_eq!(
                cuts(&prepared),
                if rule == crate::WordBreak::BreakAll {
                    vec![1]
                } else {
                    vec![]
                },
                "{raw:?} {rule:?}"
            );
        }
    }
    let sa = prepare_source(
        &mut system,
        "\u{e01}\u{e01}",
        crate::WordBreak::BreakAll,
        crate::LineBreak::Strict,
    );
    assert_eq!(cuts(&sa), [1]);
    // Genuine ID letters retain Normal's ordinary cut and KeepAll's distinct
    // suppression. This is not a blanket replacement of every policy by ID.
    assert_eq!(classes.get('\u{4e00}'), UnicodeLineBreak::Ideographic);
    for (rule, expected) in [
        (crate::WordBreak::Normal, vec![1]),
        (crate::WordBreak::KeepAll, vec![]),
        (crate::WordBreak::BreakAll, vec![1]),
    ] {
        let prepared = prepare_source(
            &mut system,
            "\u{4e00}\u{4e00}",
            rule,
            crate::LineBreak::Strict,
        );
        assert_eq!(cuts(&prepared), expected);
    }
}
