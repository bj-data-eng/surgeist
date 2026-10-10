#![forbid(unsafe_code)]
//! Public-consumer expectations from the adopted Blink Page/margin witness.
use surgeist_css::{CssRule, parse_sheet};

#[test]
fn named_page_is_admitted() {
    let report = parse_sheet("@page invoice {}");
    assert!(report.is_clean(), "named Page must be admitted: {report:?}");
    assert!(matches!(report.syntax().rules()[0], CssRule::Page(_)));
}

#[test]
fn page_size_and_margin_child_are_retained() {
    let report = parse_sheet("@page { size:A4; @top-left { content:\"X\"; } }");
    assert!(
        report.is_clean(),
        "supported Page payload must be retained: {report:?}"
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { size: a4; @top-left { content: \"X\"; } }"
    );
}

#[test]
fn declaration_only_page_has_adopted_cssom_output() {
    let report = parse_sheet("@page { margin:1px; }");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { margin: 1px; }"
    );
    assert_eq!(report, before);
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        "@page { margin: 1px; }"
    );
}
