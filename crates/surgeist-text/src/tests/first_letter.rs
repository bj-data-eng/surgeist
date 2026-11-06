use crate::{ErrorCode, Range, Text};

type Partition = (Option<Range>, Range, Option<Range>);

fn partition(input: &str, language: Option<&str>) -> Option<Partition> {
    Text::from(input)
        .first_letter(language)
        .unwrap()
        .map(|selection| (selection.prefix(), selection.letter(), selection.suffix()))
}

#[test]
fn containing_block_language_tailors_dutch_ij_without_extending_the_line() {
    let input = "“\u{2009}IJ\u{2009}!”—A";
    for language in [Some("nl"), Some("nl-Latn"), Some("nl-NL")] {
        assert_eq!(
            partition(input, language),
            Some((
                Some(Range::new(0, 2)),
                Range::new(2, 4),
                Some(Range::new(4, 7))
            ))
        );
    }
    for language in [None, Some("en"), Some("nl-Cyrl")] {
        assert_eq!(
            partition(input, language),
            Some((Some(Range::new(0, 2)), Range::new(2, 3), None))
        );
    }
    assert_eq!(
        partition("ij?", Some("nl")),
        Some((None, Range::new(0, 2), Some(Range::new(2, 3))))
    );
    for input in ["Ij?", "iJ?"] {
        assert_eq!(
            partition(input, Some("nl")),
            Some((None, Range::new(0, 1), None))
        );
    }
    assert_eq!(
        partition("“\u{2009}IJ\u{2009}", Some("nl")),
        Some((Some(Range::new(0, 2)), Range::new(2, 4), None))
    );
}

#[test]
fn matches_whole_utf16_graphemes_for_letters_numbers_and_symbols() {
    for (input, letter, suffix) in [
        ("“A\u{0301}!”", Range::new(1, 3), Range::new(3, 5)),
        ("“𝟚!”", Range::new(1, 3), Range::new(3, 5)),
        ("“👩‍🚀!”", Range::new(1, 6), Range::new(6, 8)),
    ] {
        assert_eq!(
            partition(input, None),
            Some((Some(Range::new(0, 1)), letter, Some(suffix)))
        );
    }
    let text = Text::from_code_units(vec![0xd800, b'A'.into()]);
    assert!(text.first_letter(None).unwrap().is_none());
}

#[test]
fn prefix_and_suffix_apply_their_own_punctuation_and_space_rules() {
    for input in ["", "?!", "\u{0301}A", "\u{2009}A", "“\u{3000}A", "“\tA"] {
        assert_eq!(partition(input, None), None, "{input:?}");
    }
    assert_eq!(
        partition("—(\u{00a0}A!\u{2009}”\u{2009}", None),
        Some((
            Some(Range::new(0, 3)),
            Range::new(3, 4),
            Some(Range::new(4, 7))
        ))
    );
    for input in ["A(?!", "A—?!", "A !", "A\u{00a0}!", "A\u{3000}!"] {
        assert_eq!(
            partition(input, None),
            Some((None, Range::new(0, 1), None)),
            "{input:?}"
        );
    }
}

#[test]
fn dutch_stress_forms_keep_exact_source_boundaries() {
    for (input, end) in [("íj?", 2), ("i\u{0301}j?", 3), ("i\u{0301}j\u{0301}?", 4)] {
        assert_eq!(
            partition(input, Some("nl")),
            Some((None, Range::new(0, end), Some(Range::new(end, end + 1))))
        );
    }
    assert_eq!(
        partition("i\u{0308}j?", Some("nl")),
        Some((None, Range::new(0, 2), None))
    );
}

#[test]
fn malformed_locale_is_distinct_from_an_empty_selection() {
    let text = Text::from("!");
    assert_eq!(
        text.first_letter(Some("@")).unwrap_err().code,
        ErrorCode::InvalidStyle
    );
    assert_eq!(text.first_letter(Some("und")).unwrap(), None);
}

#[test]
fn dutch_stress_on_j_alone_does_not_extend_the_initial_unit() {
    assert_eq!(
        partition("ij\u{0301}?", Some("nl")),
        Some((None, Range::new(0, 1), None))
    );
}

#[test]
fn opaque_utf16_base_cannot_become_a_first_letter_symbol() {
    let opaque = Text::from_code_units(vec![0xd800, 0x20dd, u16::from(b'A')]);
    assert_eq!(opaque.first_letter(None).unwrap(), None);
    assert_eq!(
        partition("\u{fffd}\u{20dd}A", None),
        Some((None, Range::new(0, 2), None))
    );
}

#[test]
fn canonical_acute_on_dutch_i_keeps_the_original_range() {
    assert_eq!(
        partition("i\u{0341}j?", Some("nl")),
        Some((None, Range::new(0, 3), Some(Range::new(3, 4))))
    );
}

#[test]
fn canonical_acute_on_dutch_j_keeps_the_original_range() {
    assert_eq!(
        partition("i\u{0301}j\u{0341}?", Some("nl")),
        Some((None, Range::new(0, 4), Some(Range::new(4, 5))))
    );
}
