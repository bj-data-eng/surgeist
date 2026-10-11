//! HTML legacy-color behavior from the retained HTML and CSS Color editions.

use std::collections::HashSet;
use surgeist_html::{
    DocumentString, HtmlLegacyColor, HtmlValueAuthorState as Author, HtmlValueFailure as Failure,
    HtmlValueInterpreter, HtmlValueRecoveryState as Recovery, HtmlValueSyntax,
    HtmlValueSyntaxError,
};

// Independent normative input, not the provider's lookup table. Root-owned
// references retain the original publication, attribution and conversion notes.
// CSS Color 4 CRD 2026-09-08, original snapshot SHA-256:
// bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba.
const COLOR4: &str =
    include_str!("../../../references/css-color-4--CRD-css-color-4-20260908--bada647312a7.md");

fn named_color_oracle() -> Vec<(&'static str, [u8; 3])> {
    let section = COLOR4
        .split_once("<a id=\"named-colors\"></a>")
        .expect("retained normative named-color section")
        .1
        .split("\n### ")
        .next()
        .unwrap();
    let table = section
        .split_once("\n| Named | Numeric |")
        .expect("normative opaque named-color table")
        .1;
    let mut rows = Vec::new();
    let mut names = HashSet::new();
    for line in table
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with('|'))
    {
        let columns: Vec<_> = line.split('|').collect();
        let Some(name_cell) = columns.get(3) else {
            panic!("invalid retained named-color row: {line}")
        };
        let Some((_, name)) = name_cell.split_once("id=\"valdef-color-") else {
            // The table's separator row contains no data definition.
            assert!(line.contains("---"));
            continue;
        };
        assert_eq!(columns.len(), 7, "normative table column count");
        let name = name.split_once('"').unwrap().0;
        assert!(name.bytes().all(|byte| byte.is_ascii_lowercase()));
        assert!(names.insert(name), "duplicate normative color: {name}");
        let components: Vec<u8> = columns[5]
            .split_ascii_whitespace()
            .map(|component| component.parse().expect("normative RGB8 decimal component"))
            .collect();
        let rgb: [u8; 3] = components.try_into().expect("three normative RGB columns");
        rows.push((name, rgb));
    }
    assert_eq!(
        rows.len(),
        148,
        "complete selected opaque named-color domain"
    );
    assert_eq!(names.len(), 148);
    rows
}

fn recovered(raw: DocumentString) -> HtmlLegacyColor {
    let report = HtmlValueInterpreter::new(raw.len())
        .legacy_color(Some(&raw))
        .unwrap();
    assert_eq!(report.raw(), Some(&raw), "raw UTF-16 remains exact");
    assert_eq!(report.syntax(), HtmlValueSyntax::LegacyColor);
    assert_eq!(report.author(), &Author::NotDefined);
    let Recovery::Value(color) = report.recovery() else {
        panic!("expected legacy-color recovery for {raw:?}")
    };
    *color
}

#[test]
fn native_rgb8_values_serialize_to_an_equivalent_lowercase_legacy_spelling() {
    let color = HtmlLegacyColor::new(0, 161, 255);
    assert_eq!(color.channels(), [0, 161, 255]);
    assert_eq!(color.red(), 0);
    assert_eq!(color.green(), 161);
    assert_eq!(color.blue(), 255);
    assert_eq!(color.as_document_string(), DocumentString::from("#00a1ff"));
    assert_eq!(recovered(color.as_document_string()), color);
    assert_eq!(
        HtmlLegacyColor::new(255, 255, 255).as_document_string(),
        DocumentString::from("#ffffff")
    );
    assert_eq!(
        HtmlLegacyColor::new(0, 0, 0).as_document_string(),
        DocumentString::from("#000000")
    );
    let mut colors = HashSet::new();
    assert!(colors.insert(color));
    assert!(!colors.insert(HtmlLegacyColor::new(0, 161, 255)));
    assert!(colors.insert(HtmlLegacyColor::new(0, 161, 254)));
}

#[test]
fn every_standard_opaque_named_color_recovers_its_normative_rgb_in_ascii_case_variants() {
    for (name, expected) in named_color_oracle() {
        let mixed_case: String = name
            .chars()
            .enumerate()
            .map(|(index, character)| {
                if index.is_multiple_of(2) {
                    character.to_ascii_uppercase()
                } else {
                    character
                }
            })
            .collect();
        for spelling in [name.to_owned(), name.to_ascii_uppercase(), mixed_case] {
            assert_eq!(
                recovered(DocumentString::from(spelling)).channels(),
                expected,
                "normative named color {name}"
            );
        }
    }
}

#[test]
fn empty_and_transparent_fail_while_nonempty_ascii_whitespace_recovers_black() {
    let interpreter = HtmlValueInterpreter::new(32);
    for (spelling, expected) in [
        ("", HtmlValueSyntaxError::EmptyLegacyColor),
        ("transparent", HtmlValueSyntaxError::TransparentLegacyColor),
        (
            " \tTRANSPARENT\r",
            HtmlValueSyntaxError::TransparentLegacyColor,
        ),
    ] {
        let raw = DocumentString::from(spelling);
        let report = interpreter.legacy_color(Some(&raw)).unwrap();
        assert_eq!(report.author(), &Author::NotDefined);
        assert_eq!(report.raw(), Some(&raw));
        assert_eq!(
            report.recovery(),
            &Recovery::Failed(Failure::Syntax(expected))
        );
    }
    for spelling in [" ", "\t\n\u{c}\r ", "#"] {
        assert_eq!(
            recovered(DocumentString::from(spelling)).channels(),
            [0, 0, 0]
        );
    }
    assert_eq!(
        recovered(DocumentString::from(" \tReD\r")).channels(),
        [255, 0, 0]
    );
    assert_eq!(
        recovered(DocumentString::from("\u{a0}Red\u{a0}")).channels(),
        [0, 237, 0]
    );
}

#[test]
fn legacy_fallback_does_not_resolve_contextual_colors_css_functions_or_escapes() {
    for (spelling, expected) in [
        ("currentColor", [192, 224, 0]),
        ("ButtonFace", [176, 0, 206]),
        ("Canvas", [202, 0, 160]),
        ("rgb(1,2,3)", [0, 16, 48]),
        ("r\\65 d", [0, 101, 13]),
    ] {
        assert_eq!(
            recovered(DocumentString::from(spelling)).channels(),
            expected,
            "{spelling}"
        );
    }
}

#[test]
fn shorthand_expands_nibbles_only_in_the_exact_hash_three_digit_branch() {
    for (spelling, expected) in [
        ("#aBc", [170, 187, 204]),
        ("#123", [17, 34, 51]),
        ("#abcd", [171, 205, 0]),
        ("a", [10, 0, 0]),
        ("abc", [10, 11, 12]),
        ("#ggg", [0, 0, 0]),
    ] {
        assert_eq!(
            recovered(DocumentString::from(spelling)).channels(),
            expected,
            "{spelling}"
        );
    }
}

#[test]
fn utf16_supplementary_pairs_expand_to_two_zeroes_and_isolated_surrogates_to_one() {
    let paired = DocumentString::from_utf16(vec![0xd83d, 0xde00, 0x66, 0x66]);
    assert_eq!(recovered(paired).channels(), [0, 255, 0]);
    for surrogate in [0xd800, 0xdc00] {
        let isolated = DocumentString::from_utf16(vec![surrogate, 0x66, 0x66]);
        assert_eq!(recovered(isolated).channels(), [0, 15, 15]);
    }
    let null = DocumentString::from_utf16(vec![0, 0x66, 0x66]);
    assert_eq!(recovered(null).channels(), [0, 15, 15]);
}

#[test]
fn transformed_prefix_truncates_before_hash_removal_and_ignores_later_input() {
    // 128 transformed points pad to 129, with the final two blue nibbles f0.
    let at_limit = format!("{}f", "0".repeat(127));
    assert_eq!(
        recovered(DocumentString::from(at_limit.clone())).channels(),
        [0, 0, 240]
    );
    assert_eq!(
        recovered(DocumentString::from(format!("{at_limit}abcde"))).channels(),
        [0, 0, 240]
    );

    // The '#' counts in the 128-point prefix, so the following f is discarded
    // before removing the hash. Removing it first would incorrectly expose f.
    let hash_before_limit = format!("#{}f", "0".repeat(127));
    assert_eq!(
        recovered(DocumentString::from(hash_before_limit)).channels(),
        [0, 0, 0]
    );

    // 126 zeroes + a supplementary point + f has 128 source code points but
    // 129 transformed points. Replacement precedes truncation, discarding f.
    let pair_at_limit = format!("{}😀f", "0".repeat(126));
    assert_eq!(
        recovered(DocumentString::from(pair_at_limit)).channels(),
        [0, 0, 0]
    );
}

#[test]
fn long_components_keep_the_final_eight_before_synchronized_zero_removal() {
    for (spelling, expected) in [
        ("001200340056", [18, 52, 86]),
        ("001200341256", [0, 0, 18]),
        ("000000120000003400000056", [18, 52, 86]),
        ("f00000012e00000034d00000056", [18, 52, 86]),
        ("123456", [18, 52, 86]),
        ("123456789", [18, 69, 120]),
    ] {
        assert_eq!(
            recovered(DocumentString::from(spelling)).channels(),
            expected,
            "{spelling}"
        );
    }
}
