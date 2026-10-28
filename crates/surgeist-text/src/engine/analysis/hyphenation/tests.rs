use super::*;
use crate::engine::resolve::{AnalyzedStyleOwner, AnalyzedStyleRun};
use icu_properties::{props::Script, script::ScriptWithExtensions};

fn analyzed(text: &str) -> (SourceProjection, Words) {
    let source = crate::Source::new(text);
    let projection = SourceProjection::new(&source).unwrap();
    let styles = [AnalyzedStyleRun {
        owner: AnalyzedStyleOwner::Source(crate::Range::new(0, source.text().len())),
        style: crate::ValidatedStyle::try_from(crate::Style::default()).unwrap(),
    }];
    let (analysis, word_ranges, _) = super::super::analyze(
        &projection,
        &styles,
        &[],
        &crate::protocol::paragraph::ParagraphData::standalone(&source),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let words = Words::new(&projection, &analysis.graphemes, word_ranges);
    (projection, words)
}

#[test]
fn complete_word_uses_one_count_map_with_format_controls_marks_and_supplementary_bases() {
    for text in [
        "ab\u{ad}cde",
        "a\u{61c}\u{61c}b\u{ad}cde",
        "ab\u{ad}\u{61c}\u{61c}cde",
        "a\u{301}b\u{ad}cde",
        "a\u{903}b\u{ad}cde",
        "\u{1d49c}b\u{ad}cde",
    ] {
        let (projection, words) = analyzed(text);
        let cut = projection
            .atoms
            .iter()
            .position(|atom| atom.code_point == 0xad)
            .unwrap()
            + 1;
        assert_eq!(
            words.at(cut),
            Some(CutCounts {
                total: 5,
                before: 2,
                after: 3
            }),
            "{text:?}"
        );
        assert!(
            words
                .at(cut)
                .unwrap()
                .admits(crate::HyphenateLimitChars::default())
        );
    }
    for text in [
        "a\u{61c}\u{61c}\u{61c}\u{ad}\u{61c}\u{61c}\u{61c}b",
        "a'\u{ad}bcd",
        "a'\u{20dd}\u{ad}bcd",
    ] {
        let (projection, words) = analyzed(text);
        let cut = projection
            .atoms
            .iter()
            .position(|atom| atom.code_point == 0xad)
            .unwrap()
            + 1;
        let counts = words.at(cut).unwrap();
        assert_eq!(counts.before, 1);
        assert!(
            !counts.admits(crate::HyphenateLimitChars::default()),
            "{text:?}"
        );
    }
}

#[test]
fn effective_unit_categories_are_independent_of_lexical_membership_and_glyphs() {
    for (text, expected) in [
        ("\u{301}", false),
        ("\u{903}", true),
        ("\u{93e}", true),
        ("\u{61c}", false),
        ("\u{ad}", false),
        ("\u{1}", true),
        ("\u{600}a", true),
        ("a\u{903}", true),
        ("a\u{301}", true),
        ("'", false),
        ("'\u{20dd}", false),
        (" \u{301}", true),
        ("1", true),
        ("$", true),
        ("\u{1f469}\u{200d}\u{1f4bb}", true),
    ] {
        let (projection, _) = analyzed(text);
        assert_eq!(projection.graphemes.len(), 2, "one real EGC: {text:?}");
        assert_eq!(
            counts(&projection, 0..projection.atoms.len()),
            expected,
            "{text:?}"
        );
    }
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(0x20dd),
        GeneralCategory::EnclosingMark
    );
    assert_eq!(
        CodePointMapData::<Script>::new().get32(0x20dd),
        Script::Inherited
    );
    assert!(!ScriptWithExtensions::new().has_script32(0x20dd, Script::Common));
    let (_, words) = analyzed("$");
    assert_eq!(
        words.at(0),
        None,
        "countable symbol alone is not a lexical word"
    );
}

#[test]
fn opaque_original_unit_keeps_its_raw_category_for_counting() {
    let source = crate::Source::new(crate::Text::from_code_units(vec![0xd800, 0x301]));
    let projection = SourceProjection::new(&source).unwrap();
    assert_eq!(projection.graphemes, [0, 2]);
    assert_eq!(projection.atoms[0].code_point, 0xd800);
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(0xd800),
        GeneralCategory::Surrogate
    );
    assert!(counts(&projection, 0..2));
}

#[test]
fn counting_does_not_authorize_interior_grapheme_cuts_or_split_base_marks() {
    let (_, words) = analyzed("ab\u{301}cde");
    assert_eq!(words.at(2), None, "between base b and its actual Mn");
    assert_eq!(
        words.at(3),
        Some(CutCounts {
            total: 5,
            before: 2,
            after: 3
        })
    );
}

#[test]
fn prepend_format_belongs_to_the_actual_countable_grapheme() {
    use icu_properties::props::GraphemeClusterBreak;
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(0x600),
        GeneralCategory::Format
    );
    assert_eq!(
        CodePointMapData::<GraphemeClusterBreak>::new().get32(0x600),
        GraphemeClusterBreak::Prepend
    );
    let (projection, words) = analyzed("\u{600}ab\u{ad}ide");
    assert_eq!(projection.graphemes[0..2], [0, 2]);
    assert_eq!(
        words.at(4),
        Some(CutCounts {
            total: 5,
            before: 2,
            after: 3
        })
    );
}

#[test]
fn independent_unicode_seventeen_vectors_keep_complete_egcs_and_both_cut_sides() {
    for (text, expected, vector) in [
        ("ab\u{ad}ide", (5, 2, 3), vec![1, 1, 0, 1, 1, 1]),
        ("ab\u{61c}\u{ad}ide", (5, 2, 3), vec![1, 1, 0, 0, 1, 1, 1]),
        ("ab\u{ad}\u{61c}ide", (5, 2, 3), vec![1, 1, 0, 0, 1, 1, 1]),
        (
            "كت\u{61c}\u{61c}\u{61c}\u{ad}\u{61c}\u{61c}\u{61c}ب",
            (3, 2, 1),
            vec![1, 1, 0, 0, 0, 0, 0, 0, 0, 1],
        ),
        ("na\u{308}\u{ad}ive", (5, 2, 3), vec![1, 1, 0, 1, 1, 1]),
        ("कार\u{ad}खाना", (4, 2, 2), vec![1, 1, 0, 1, 1]),
        ("can\u{ad}'t", (4, 3, 1), vec![1, 1, 1, 0, 0, 1]),
        (
            "\u{10400}\u{10401}\u{ad}\u{10402}\u{10403}\u{10404}",
            (5, 2, 3),
            vec![1, 1, 0, 1, 1, 1],
        ),
        ("ab\u{200d}\u{ad}ide", (5, 2, 3), vec![1, 1, 0, 1, 1, 1]),
    ] {
        let (projection, words) = analyzed(text);
        let actual: Vec<_> = projection
            .graphemes
            .windows(2)
            .map(|pair| {
                let a = projection
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[0]);
                let b = projection
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[1]);
                usize::from(counts(&projection, a..b))
            })
            .collect();
        assert_eq!(actual, vector, "{text:?}");
        let cut = projection
            .atoms
            .iter()
            .position(|atom| atom.code_point == 0xad)
            .unwrap()
            + 1;
        let result = words
            .at(cut)
            .expect("actual letter-bearing ICU lexical span");
        assert_eq!(
            (result.total, result.before, result.after),
            expected,
            "{text:?}"
        );
    }
    let (projection, words) = analyzed("\u{10400}\u{10401}\u{ad}\u{10402}\u{10403}\u{10404}");
    assert_eq!(projection.atoms[2].source, crate::Range::new(4, 5));
    assert_eq!(words.at(3).unwrap().before, 2);
}
