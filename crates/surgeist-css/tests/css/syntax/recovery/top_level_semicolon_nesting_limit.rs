//! Syntax 3 (2026 CRD) §5.5.1 consumes a qualified rule for anything at the
//! stylesheet root other than whitespace, CDO/CDC or an at-keyword, and §5.5.3
//! has no stop token there: a root `;` is prelude content, so `a; b(((` is one
//! qualified rule ending at EOF. Only an at-rule (§5.5.2) ends at its own `;`,
//! and CDO/CDC before a unit are discarded (§5.5.1).
//! The structural nesting limit (256) therefore stops that whole unit.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-stylesheet-contents
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-qualified-rule
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-at-rule
use surgeist_css::{CssErrorCode, CssParser, CssRecoveryAction};

const LIMIT: usize = 256;

fn assert_limit_unit(source: &str, unit_start: usize, parens_start: usize, retained: usize) {
    let report = CssParser::default().parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), retained, "{report:?}");
    let limits = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
        .collect::<Vec<_>>();
    let [limit] = limits.as_slice() else {
        panic!("one nesting limit: {:?}", report.diagnostics());
    };
    assert_eq!(limit.error().code(), CssErrorCode::NestingLimit);
    // The first parenthesis beyond the limit is the responsible opening.
    assert_eq!(
        limit.error().position().byte_offset().value(),
        parens_start + LIMIT
    );
    assert_eq!(
        (
            limit.span().start().byte_offset().value(),
            limit.span().end().byte_offset().value()
        ),
        (unit_start, source.len())
    );
    // The unit is one rule: nothing before its `;` is a separate rule.
    let qualified = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
        .count();
    assert_eq!(qualified, 0, "{:?}", report.diagnostics());
}

#[test]
fn a_root_semicolon_stays_inside_the_qualified_rule_stopped_by_the_nesting_limit() {
    let parens = "(".repeat(LIMIT + 4);
    assert_limit_unit(&format!("b{parens}"), 0, 1, 0);
    assert_limit_unit(&format!("a; b{parens}"), 0, 4, 0);
    assert_limit_unit(&format!("a;;; b{parens}"), 0, 6, 0);
    assert_limit_unit(&format!("/**/ a; b{parens}"), 5, 9, 0);
    assert_limit_unit(&format!(".x{{}} a; b{parens}"), 5, 9, 1);
    // CDO and CDC before a unit are discarded, not part of it.
    assert_limit_unit(&format!("<!-- a; b{parens}"), 5, 9, 0);
    assert_limit_unit(&format!("--> <!-- b{parens}"), 9, 10, 0);
}

#[test]
fn an_at_rule_ends_at_its_own_semicolon_before_the_limited_unit() {
    let parens = "(".repeat(LIMIT + 4);
    let source = format!("@x; b{parens}");
    assert_limit_unit(&source, 4, 5, 0);
    let report = CssParser::default().parse_sheet(&source);
    let at_rules = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        .map(|diagnostic| {
            (
                diagnostic.span().start().byte_offset().value(),
                diagnostic.span().end().byte_offset().value(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(at_rules, [(0, 3)]);
}
