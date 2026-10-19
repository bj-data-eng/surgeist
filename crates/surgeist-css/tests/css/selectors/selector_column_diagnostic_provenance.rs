#![forbid(unsafe_code)]
//! Preserve the invalid right-hand token's diagnostic while disambiguating `||`
//! from a descendant whose type selector has an explicit empty namespace.
//! Column is admitted, but its required right compound cannot begin with `!`;
//! Nesting 1 §3.1 requires ignoring the invalid child without losing its parent.

use surgeist_css::{
    CssErrorCode, CssNamespaceConstraint, CssNamespaceContext, CssRecoveryAction, CssRule,
    CssSelector, CssSelectorCombinator, CssSourcePosition, CssTokenKind, ErrorKind, parse_selector,
    parse_sheet,
};

fn assert_position(position: CssSourcePosition, source: &str, byte: usize) {
    assert_eq!(position.byte_offset().value(), byte, "{source}");
    assert_eq!(position.line().value(), 0, "{source}");
    assert_eq!(
        position.column().value() as usize,
        source[..byte].encode_utf16().count(),
        "{source}",
    );
}

fn rejected_column(source: &str) {
    let report = parse_sheet(source);
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("invalid nested rule must retain only its parent: {report:?}");
    };
    assert!(parent.declarations().is_empty());
    assert!(parent.rules().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid nested-rule diagnostic: {report:?}");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    // The pair is a valid Column relation. The actual `!` token cannot supply
    // its required right compound; lookahead must preserve that token's origin.
    let invalid_rhs = source.find('!').unwrap();
    assert_position(diagnostic.error().position(), source, invalid_rhs);
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("structured selector error");
    };
    let encountered = detail.encountered().expect("invalid right-hand delimiter");
    assert_eq!(encountered.kind(), CssTokenKind::Delim);
    assert_eq!(encountered.authored(), "!");
    // Preserve the established nested recovery unit as well as its coordinates.
    assert_position(diagnostic.span().start(), source, source.find('&').unwrap());
    let inner_rule_end = source.rfind("} }").unwrap() + 1;
    assert_position(diagnostic.span().end(), source, inner_rule_end);
}

#[test]
fn invalid_column_rhs_reports_responsible_delimiter_and_retains_parent() {
    rejected_column(".card { & || ! { color: red; } }");
}

#[test]
fn invalid_column_rhs_preserves_utf8_and_utf16_delimiter_coordinates() {
    rejected_column("/*😀*/ .card { & || ! { color: red; } }");
}

#[test]
fn single_namespace_separator_after_whitespace_remains_a_descendant() {
    let report = parse_selector("svg |leaf", &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let Some(CssSelector::Complex(selector)) = report.syntax() else {
        panic!("two descendant compounds: {report:?}");
    };
    assert_eq!(
        selector.first().type_selector().unwrap().local_name(),
        Some("svg"),
    );
    let [part] = selector.rest() else {
        panic!("one descendant relation");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Descendant);
    let name = part.selector().type_selector().unwrap();
    assert_eq!(name.local_name(), Some("leaf"));
    assert_eq!(name.namespace(), &CssNamespaceConstraint::ExplicitNone);
}
