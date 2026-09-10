//! CSS Nesting 1 §3 permits nested style rules in a style rule's body, including
//! when the containing style rule is scoped. Scope changes selector context,
//! not the style body's acceptance of nested rules and declaration runs.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting

use surgeist_css::parse_sheet;

#[test]
fn scoped_style_body_accepts_nested_rules_and_surrounding_declarations() {
    let source = "@scope { .parent { color: red; .child { width: 1px; } opacity: 1; } }";
    let report = parse_sheet(source);
    assert!(
        report.is_clean(),
        "valid scoped style nesting must not discard syntax during recovery: {:?}",
        report.diagnostics(),
    );
}
