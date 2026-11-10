//! The `unicode-range` descriptor of `@font-face`, read as CSS Syntax 3 (2026
//! CRD) §5.5.11 reads it: the value is tokenized again with unicode ranges
//! allowed, and §4.3.14 (consume a unicode-range token) makes each range one
//! `<unicode-range-token>` that starts with `U` or `u` and a plus sign, holds up
//! to six hexadecimal digits or question marks, and ends at the first code
//! point that cannot continue it. The descriptor is a comma-separated list of
//! those tokens (Fonts 4 `<urange>#`); a range above U+10FFFF or one whose end
//! is before its start is not valid.

use surgeist_css::CssParser;
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind, CssFontFaceDescriptorValue,
};

/// The ranges `text` is as the value of the descriptor, or `None` when it is not valid.
fn ranges(text: &str) -> Option<Vec<(u32, u32)>> {
    let kind = CssFontFaceDescriptorKind::from_css_name("unicode-range").unwrap();
    let report = CssAuthoredFontFaceDescriptorValue::parse(&CssParser::default(), text, kind);
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(
        CssFontFaceDescriptorValue::UnicodeRange(list),
    )) = report.syntax()
    else {
        return None;
    };
    Some(
        list.ranges()
            .iter()
            .map(|range| (range.start(), range.end()))
            .collect(),
    )
}

#[test]
fn a_range_is_the_code_points_its_token_spells() {
    for (text, expected) in [
        // A single code point.
        ("U+26", vec![(0x26, 0x26)]),
        ("U+0025", vec![(0x25, 0x25)]),
        // The `u` may be lowercase (§4.3.1).
        ("u+0025", vec![(0x25, 0x25)]),
        // An interval, with the digits of each end.
        ("U+0-7F", vec![(0x0, 0x7f)]),
        ("U+0025-00FF", vec![(0x25, 0xff)]),
        ("u+2e-3F", vec![(0x2e, 0x3f)]),
        // Question marks stand for every digit: the lowest to the highest.
        ("U+4??", vec![(0x400, 0x4ff)]),
        ("U+1?", vec![(0x10, 0x1f)]),
        ("U+??", vec![(0x0, 0xff)]),
        // The largest code point.
        ("U+10FFFF", vec![(0x10_ffff, 0x10_ffff)]),
        // A list, with whitespace and comments around the commas.
        ("U+0025-00FF, U+4??", vec![(0x25, 0xff), (0x400, 0x4ff)]),
        ("U+25 , U+26", vec![(0x25, 0x25), (0x26, 0x26)]),
        ("U+25/**/,/**/U+26", vec![(0x25, 0x25), (0x26, 0x26)]),
        (" U+25 ", vec![(0x25, 0x25)]),
    ] {
        assert_eq!(ranges(text), Some(expected), "{text:?}");
    }
}

#[test]
fn a_value_that_is_not_a_list_of_range_tokens_is_not_valid() {
    for text in [
        // `U+` followed by no digit or question mark is an ident and a delimiter.
        "U+",
        "U",
        "u",
        "25",
        "'U+25'",
        "x",
        // A range above the largest code point (Fonts 4), and one that ends
        // before it starts.
        "U+110000",
        "U+0-110000",
        "U+??????",
        "U+7F-0",
        // Question marks end the token: a range end cannot follow them, and
        // another token is not a comma.
        "U+4??-5??",
        "U+25 U+26",
        "U+25 x",
        // A comment ends a token (§4.3.1), so what it separates is two tokens.
        "U+2/**/5",
        // An empty member.
        "U+25,",
        ",U+25",
        "U+25,,U+26",
        // A seventh digit is another token, not part of the range.
        "U+1234567",
    ] {
        assert_eq!(ranges(text), None, "{text:?}");
    }
}

/// The first rejected token and its byte offset in the diagnostic of `text`.
fn rejection(text: &str) -> (String, usize) {
    let kind = CssFontFaceDescriptorKind::from_css_name("unicode-range").unwrap();
    let report = CssAuthoredFontFaceDescriptorValue::parse(&CssParser::default(), text, kind);
    let [diagnostic] = report.diagnostics() else {
        panic!("{text:?}: {:?}", report.diagnostics());
    };
    let error = diagnostic.error();
    (
        error
            .encountered()
            .expect("the rejected token")
            .authored()
            .to_owned(),
        error.position().byte_offset().value(),
    )
}

#[test]
fn the_token_that_cannot_continue_a_range_is_responsible() {
    for (text, token, offset) in [
        // The range token ends at the first code point that cannot continue it
        // (§4.3.14): the next token is the one rejected.
        ("U+ab-", "-", 4),
        ("U+0g", "g", 3),
        ("U+1234567", "7", 8),
        ("U+25 x", "x", 5),
        // A `u` and a plus sign that no digit or question mark follows.
        ("U+g", "g", 2),
        ("U+-1", "-1", 2),
        // A token that is no range: the whole token is responsible.
        ("U+7F-0", "U+7F-0", 0),
        ("U+110000", "U+110000", 0),
    ] {
        assert_eq!(rejection(text), (token.to_owned(), offset), "{text:?}");
    }
}

#[test]
fn a_value_that_ends_inside_the_start_of_a_range_names_its_last_token() {
    for (text, token, offset) in [("u", "u", 0), ("U+", "+", 1), ("U+/**/", "+", 1)] {
        assert_eq!(rejection(text), (token.to_owned(), offset), "{text:?}");
    }
}
