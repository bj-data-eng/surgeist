//! A dropped unit's diagnostic spans that unit. Syntax 3 (2026 CRD) §4.3.2
//! consumes comments before any token, and §§5.5.1, 5.5.2, 5.5.3 and 5.5.5
//! start a rule at its at-keyword or first prelude token; text that merely
//! looks like the unit earlier in the source is not part of it.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-comment
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-stylesheet-contents
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::{CssParseReport, CssParser};

// The unit is the last occurrence of its text: every earlier copy lies in a
// comment, a string or a preceding rule.
fn assert_unit<T: std::fmt::Debug>(
    report: &CssParseReport<T>,
    source: &str,
    unit: &str,
    action: CssRecoveryAction,
) {
    let start = source.rfind(unit).unwrap();
    let dropped = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == action)
        .collect::<Vec<_>>();
    let [diagnostic] = dropped.as_slice() else {
        panic!("{source:?}: one {action:?}: {report:?}");
    };
    assert_eq!(
        (
            diagnostic.span().start().byte_offset().value(),
            diagnostic.span().end().byte_offset().value()
        ),
        (start, start + unit.len()),
        "{source:?}"
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start,
        "{source:?}"
    );
}

#[test]
fn a_dropped_top_level_unit_starts_after_a_comment_holding_its_text() {
    for (source, unit, action) in [
        ("@x;", "@x;", CssRecoveryAction::DropAtRule),
        ("/*@x;*/@x;", "@x;", CssRecoveryAction::DropAtRule),
        ("/*@x;*/ /*@x;*/\n@x;", "@x;", CssRecoveryAction::DropAtRule),
        ("/*@x{}*/@x{}", "@x{}", CssRecoveryAction::DropAtRule),
        ("/*$ {}*/$ {}", "$ {}", CssRecoveryAction::DropQualifiedRule),
        (
            "a{content:\"@x;\"} /*@x;*/@x;",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
    ] {
        assert_unit(
            &CssParser::default().parse_sheet(source),
            source,
            unit,
            action,
        );
    }
}

#[test]
fn a_dropped_nested_unit_starts_after_a_comment_holding_its_text() {
    for (source, unit, action) in [
        (
            "@media screen { /*@x;*/@x; }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@media screen { a{} /*$ {}*/$ {} }",
            "$ {}",
            CssRecoveryAction::DropQualifiedRule,
        ),
        (".a { /*@x;*/@x; }", "@x;", CssRecoveryAction::DropAtRule),
        (
            ".a { --v: \"@x;\"; /*@x;*/@x; }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@scope (.a) { /*@x;*/@x; }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
        (
            ".b { @scope (.a) { /*@x;*/@x; } }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@font-face { /*@x;*/@x; }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
        (
            "@keyframes k { /*@x;*/@x; }",
            "@x;",
            CssRecoveryAction::DropKeyframeBlock,
        ),
        (
            "@keyframes k { from {} /*$ {}*/$ {} }",
            "$ {}",
            CssRecoveryAction::DropKeyframeBlock,
        ),
        (
            "@keyframes k { from { /*@x;*/@x; } }",
            "@x;",
            CssRecoveryAction::DropAtRule,
        ),
    ] {
        assert_unit(
            &CssParser::default().parse_sheet(source),
            source,
            unit,
            action,
        );
    }
}

#[test]
fn a_dropped_style_attribute_at_rule_starts_after_a_comment_holding_its_text() {
    let source = "/*@x;*/@x; width: 2px";
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(report.syntax().len(), 1);
    assert_unit(&report, source, "@x;", CssRecoveryAction::DropAtRule);
}
