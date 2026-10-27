//! Opaque named colors against the retained CSS Color 4 section 6.1 table.
//!
//! The expected rows are parsed from the retained specification text, not from
//! the crate's table. Source: CSS Color 4 CRD 2026-09-08, original snapshot
//! SHA-256 bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba.

use std::collections::HashSet;
use surgeist_color::named::{self, NAMES};

const COLOR4: &str =
    include_str!("../../../references/css-color-4--CRD-css-color-4-20260908--bada647312a7.md");

/// One data row of the specification's opaque named-color table.
struct Row {
    name: &'static str,
    rgb: [u8; 3],
}

/// Parses the table that follows the `named-colors` anchor.
///
/// Each data row has five cells: two empty sample cells, the name cell with its
/// `valdef-color-NAME` anchor and the name as the definition text, the hex
/// column (`\#rrggbb`) and the decimal column (`r g b`). The hex and decimal
/// columns are cross-checked, so a row passes only when the specification
/// states one value twice.
fn specification_rows() -> Vec<Row> {
    let section = COLOR4
        .split_once("<a id=\"named-colors\"></a>")
        .expect("the retained specification has the named-colors anchor")
        .1
        .split("\n### ")
        .next()
        .expect("the section ends at the next heading or the document end");
    let table = section
        .split_once("\n| Named | Numeric |")
        .expect("the section has the opaque named-color table")
        .1;
    let mut rows = Vec::new();
    for line in table
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with('|'))
    {
        if line.starts_with("|---") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').collect();
        // A leading and trailing empty piece surround the five cells.
        assert_eq!(cells.len(), 7, "five cells per table row: {line}");
        let anchor = cells[3]
            .split_once("id=\"valdef-color-")
            .unwrap_or_else(|| panic!("name cell without a definition anchor: {line}"))
            .1
            .split_once('"')
            .expect("a closed anchor identifier")
            .0;
        let definition = cells[3]
            .split_once("</span>")
            .expect("anchor followed by the definition text")
            .1
            .split_once("</dfn>")
            .expect("a closed definition")
            .0;
        assert_eq!(anchor, definition, "anchor and definition text agree");
        assert!(
            definition.bytes().all(|byte| byte.is_ascii_lowercase()),
            "{definition}"
        );
        let hex = cells[4]
            .trim()
            .strip_prefix("\\#")
            .unwrap_or_else(|| panic!("hex column without the escaped hash: {line}"));
        assert_eq!(hex.len(), 6, "{line}");
        let from_hex: Vec<u8> = (0..3)
            .map(|channel| u8::from_str_radix(&hex[channel * 2..channel * 2 + 2], 16).unwrap())
            .collect();
        let from_decimal: Vec<u8> = cells[5]
            .split_ascii_whitespace()
            .map(|component| component.parse().expect("decimal channel"))
            .collect();
        assert_eq!(from_hex, from_decimal, "hex and decimal columns: {line}");
        rows.push(Row {
            name: definition,
            rgb: from_decimal.try_into().expect("three channels"),
        });
    }
    rows
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

#[test]
fn specification_table_has_148_distinct_rows_in_ascending_order() {
    let rows = specification_rows();
    assert_eq!(rows.len(), 148);
    assert_eq!(
        rows.iter()
            .map(|row| row.name)
            .collect::<HashSet<_>>()
            .len(),
        148
    );
    assert!(rows.windows(2).all(|pair| pair[0].name < pair[1].name));
}

#[test]
fn names_equals_the_retained_specification_table_row_for_row() {
    let rows = specification_rows();
    assert_eq!(NAMES.len(), rows.len());
    for (index, (row, (name, rgb))) in rows.iter().zip(NAMES.iter()).enumerate() {
        assert_eq!((row.name, row.rgb), (*name, *rgb), "row {index}");
    }
}

#[test]
fn lookup_returns_each_specification_value_for_every_ascii_case_variant() {
    for row in specification_rows() {
        let variants = [
            row.name.to_owned(),
            row.name.to_ascii_uppercase(),
            alternating_case(row.name, true),
            alternating_case(row.name, false),
        ];
        for spelling in variants {
            assert_eq!(named::lookup(&spelling), Some(row.rgb), "{spelling}");
        }
    }
}

#[test]
fn lookup_rejects_near_misses_of_every_specification_name() {
    let names: HashSet<&str> = specification_rows().iter().map(|row| row.name).collect();
    for name in &names {
        let mut candidates = vec![
            format!("{name}x"),
            format!("x{name}"),
            format!(" {name}"),
            format!("{name} "),
            format!("{name}\0"),
            format!("{name}-"),
            name.chars().rev().collect::<String>(),
        ];
        candidates.push(name[..name.len() - 1].to_owned());
        candidates.push(name[1..].to_owned());
        // Replace the middle byte with a neighboring letter.
        let middle = name.len() / 2;
        let mut bytes = name.as_bytes().to_vec();
        bytes[middle] = if bytes[middle] == b'z' {
            b'a'
        } else {
            bytes[middle] + 1
        };
        candidates.push(String::from_utf8(bytes).unwrap());
        for candidate in candidates {
            // A mutation can spell another real name, e.g. a prefix of a longer
            // name; the specification set decides.
            let expected = names.contains(candidate.as_str());
            assert_eq!(
                named::lookup(&candidate).is_some(),
                expected,
                "{candidate:?}"
            );
        }
    }
}

#[test]
fn lookup_rejects_empty_and_keywords_that_are_not_opaque_named_colors() {
    for spelling in [
        "",
        "transparent",
        "TRANSPARENT",
        "Transparent",
        "currentcolor",
        "CurrentColor",
        "inherit",
        "none",
        // System colors of CSS Color 4 section 6.2 and Appendix A.
        "canvas",
        "CanvasText",
        "LinkText",
        "ButtonFace",
        "Highlight",
        "AccentColor",
        "GrayText",
        "ActiveBorder",
        "WindowText",
        "Window",
    ] {
        assert_eq!(named::lookup(spelling), None, "{spelling:?}");
    }
}

#[test]
fn lookup_matches_only_ascii_case_folding() {
    // U+212A KELVIN SIGN lowercases to ASCII `k` under Unicode folding, and
    // U+017F LATIN SMALL LETTER LONG S uppercases to ASCII `S`; neither is an
    // ASCII case variant.
    for spelling in [
        "dark\u{212a}haki",
        "\u{17f}ilver",
        "red\u{200b}",
        "\u{ff52}\u{ff45}\u{ff44}",
        "r\u{e9}d",
        "r\u{65}\u{301}d",
        "\u{130}ndigo",
        "ind\u{131}go",
        "\u{1f3a8}",
    ] {
        assert_eq!(named::lookup(spelling), None, "{spelling:?}");
    }
    assert_eq!(named::lookup("silver"), Some([192, 192, 192]));
    assert_eq!(named::lookup("SILVER"), Some([192, 192, 192]));
}

#[test]
fn names_are_lowercase_ascii_in_strictly_ascending_order() {
    for (name, _) in NAMES {
        assert!(name.bytes().all(|byte| byte.is_ascii_lowercase()), "{name}");
    }
    assert!(NAMES.windows(2).all(|pair| pair[0].0 < pair[1].0));
}
