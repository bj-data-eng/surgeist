//! The authored rule tree is a public semantic contract: one authored rule remains one
//! node, independent of the number of selectors or declarations surrounding child rules.
//!
//! CSS Nesting 1 §3 and §5 retain nested rules and declaration runs in their parent:
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/

use surgeist_css::{CssRule, parse_sheet};

#[test]
fn selector_list_retains_one_authored_rule() {
    let source = ".card, #featured { color: red; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "a selector list belongs to one authored style rule"
    );
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("expected the authored style rule");
    };
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn nested_rule_and_surrounding_declarations_stay_in_the_authored_parent() {
    let source = ".card { color: red; & .title { color: blue; } color: green; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "nested syntax must not become independent stylesheet rules"
    );
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("expected the authored style rule");
    };
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn nested_conditional_remains_in_its_authored_parent() {
    let source = ".card { color: red; @media screen { color: blue; } color: green; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "a nested conditional retains its authored parent and order"
    );
    assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
}
