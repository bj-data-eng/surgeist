//! Differential check of `surgeist_color::named` against the `cssparser` named-color
//! function, used here as a black-box oracle while surgeist-css still depends on it.
//!
//! The table's own expected values come from the retained Color 4 specification
//! text (see surgeist-color's `tests/named.rs`); this test shows that the shipped
//! lookup and the external function agree on all 148 names, their case variants
//! and a family of near misses, so the call-site change alters no result.

use std::collections::HashSet;
use surgeist_color::named::{self, NAMES};

fn external(name: &str) -> Option<[u8; 3]> {
    cssparser::color::parse_named_color(name)
        .ok()
        .map(|(red, green, blue)| [red, green, blue])
}

fn alternating_case(name: &str, upper_first: bool) -> String {
    name.chars()
        .enumerate()
        .map(|(index, character)| {
            if index.is_multiple_of(2) == upper_first {
                character.to_ascii_uppercase()
            } else {
                character
            }
        })
        .collect()
}

fn assert_agree(spelling: &str) {
    assert_eq!(
        named::lookup(spelling),
        external(spelling),
        "lookup and the external function disagree on {spelling:?}"
    );
}

#[test]
fn all_148_names_and_their_case_variants_agree_with_the_external_function() {
    assert_eq!(NAMES.len(), 148);
    for (name, rgb) in NAMES {
        assert_eq!(external(name), Some(rgb), "external value for {name}");
        for spelling in [
            name.to_owned(),
            name.to_ascii_uppercase(),
            alternating_case(name, true),
            alternating_case(name, false),
        ] {
            assert_eq!(named::lookup(&spelling), Some(rgb), "{spelling}");
            assert_agree(&spelling);
        }
    }
}

#[test]
fn near_misses_and_non_names_agree_with_the_external_function() {
    let names: HashSet<&str> = NAMES.iter().map(|(name, _)| *name).collect();
    for name in &names {
        let middle = name.len() / 2;
        let mut bytes = name.as_bytes().to_vec();
        bytes[middle] = if bytes[middle] == b'z' {
            b'a'
        } else {
            bytes[middle] + 1
        };
        for spelling in [
            format!("{name}x"),
            format!("x{name}"),
            format!(" {name}"),
            format!("{name} "),
            format!("{name}\0"),
            format!("{name}-"),
            name[..name.len() - 1].to_owned(),
            name[1..].to_owned(),
            name.chars().rev().collect::<String>(),
            String::from_utf8(bytes).unwrap(),
        ] {
            assert_agree(&spelling);
        }
    }
    for spelling in [
        "",
        "transparent",
        "TRANSPARENT",
        "currentcolor",
        "inherit",
        "none",
        "canvas",
        "CanvasText",
        "ButtonFace",
        "Highlight",
        "AccentColor",
        "ActiveBorder",
        "WindowText",
        "dark\u{212a}haki",
        "\u{17f}ilver",
        "\u{130}ndigo",
        "ind\u{131}go",
        "r\u{e9}d",
        "\u{ff52}\u{ff45}\u{ff44}",
        "\u{1f3a8}",
    ] {
        assert_agree(spelling);
    }
}

#[test]
fn every_lowercase_spelling_of_up_to_three_letters_agrees_with_the_external_function() {
    // The shortest names have three letters, so this covers every short
    // spelling, not only mutations of known names.
    let letters = b'a'..=b'z';
    let mut checked = 0_u32;
    for first in letters.clone() {
        assert_agree(&char::from(first).to_string());
        for second in letters.clone() {
            assert_agree(&String::from_utf8(vec![first, second]).unwrap());
            for third in letters.clone() {
                assert_agree(&String::from_utf8(vec![first, second, third]).unwrap());
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 26 * 26 * 26);
}
