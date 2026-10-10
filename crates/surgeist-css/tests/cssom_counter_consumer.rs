#![forbid(unsafe_code)]
//! Authored admission is distinct from definition validity in Counter Styles 2026.
use surgeist_css::{CssRule, parse_sheet};

#[test]
fn extends_symbol_conflict_remains_inspectable_authored_syntax() {
    let report = parse_sheet("@counter-style example { system: extends decimal; symbols: x; }");
    assert!(
        report.is_clean(),
        "definition conflict does not invalidate authored grammar: {report:?}"
    );
    let CssRule::CounterStyle(rule) = &report.syntax().rules()[0] else {
        panic!("retained counter rule");
    };
    assert!(rule.descriptors().symbols().is_some());
}
