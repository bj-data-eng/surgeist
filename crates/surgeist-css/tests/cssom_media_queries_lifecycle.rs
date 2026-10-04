#![forbid(unsafe_code)]
//! CSSOM1 sections 4.2–4.3 define these canonical bytes independently of
//! authored source spelling. Media Queries 5 remains the grammar owner.
//!
//! These cases use existing public parsing/serialization. The CSSOM single-query
//! nullable result and ignored-query projection need their own functional APIs;
//! a missing symbol or a changed authored recovery contract is not RED evidence.

use surgeist_css::{parse_media_query, parse_media_query_list};

fn query_text(source: &str) -> String {
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report
        .syntax()
        .serialize()
        .expect("a clean query can be serialized")
        .as_css()
        .to_owned()
}

fn list_text(source: &str) -> String {
    let report = parse_media_query_list(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report
        .syntax()
        .serialize()
        .expect("a clean list can be serialized")
        .as_css()
        .to_owned()
}

#[test]
fn known_orientation_and_scan_values_use_the_defined_lowercase_spelling() {
    // These four spellings are the defined cells in CSSOM1 section 4.2.1.
    // No expected value is supplied for a table cell containing an ellipsis.
    let actual = [
        "(ORIENTATION: PORTRAIT)",
        "(orientation: LANDSCAPE)",
        "(SCAN: PROGRESSIVE)",
        "(scan: INTERLACE)",
    ]
    .map(query_text);
    assert_eq!(
        actual,
        [
            "(orientation: portrait)",
            "(orientation: landscape)",
            "(scan: progressive)",
            "(scan: interlace)",
        ]
    );
}

#[test]
fn an_unnegated_all_type_is_elided_without_removing_repeated_features() {
    // This is CSSOM1 section 4.2's explicit example, including the repetition.
    assert_eq!(
        query_text("all and (color) and (color)"),
        "(color) and (color)"
    );
}

#[test]
fn unknown_media_types_are_lowercase_without_becoming_not_all() {
    // MQ5 treats an unknown type as nonmatching, but does not ignore its grammar.
    // In particular, negating an unknown media type differs from `not all`.
    assert_eq!(query_text("FuTuRe"), "future");
    assert_eq!(query_text("NOT FuTuRe"), "not future");
}

#[test]
fn query_comparison_uses_canonical_bytes_for_a_redundant_default_type() {
    // CSSOM1 section 4.3 compares serialization, not source or model identity.
    assert_eq!(query_text("all and (color)"), query_text("(color)"));
}

#[test]
fn media_lists_preserve_member_order_and_use_comma_space_separators() {
    assert_eq!(
        list_text("SCREEN,not PRINT and (COLOR), (color)"),
        "screen, not print and (color), (color)"
    );
    assert_eq!(list_text(""), "");
    assert_eq!(list_text(" /* empty */ "), "");
    assert_ne!(list_text("screen, print"), list_text("print, screen"));
}

#[test]
fn negated_all_and_explicit_negation_keep_their_required_type_and_spacing() {
    assert_eq!(query_text("NOT ALL AND (COLOR)"), "not all and (color)");
    assert_eq!(
        query_text("NOT SCREEN AND (COLOR)"),
        "not screen and (color)"
    );
    assert_eq!(query_text("not all"), "not all");
}

#[test]
fn byte_comparison_preserves_case_sensitive_opaque_arguments() {
    // MQ5 general-enclosed keeps future syntax; serialization cannot infer a
    // value format from CSSOM1's ellipsis cells or evaluate the condition.
    assert_eq!(query_text("SCREEN"), query_text("screen"));
    assert_ne!(query_text("future(\"A\")"), query_text("future(\"a\")"));
}
