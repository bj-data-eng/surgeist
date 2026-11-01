//! A rule that looks like a custom property (`--name:` before its block).
//!
//! Syntax 3 (2026 CRD) §5.5.3, consume a qualified rule: a prelude whose first
//! two non-whitespace values are an `--` ident and a colon consumes its block
//! and returns nothing. §5.4.6 step 3, parse a rule: when consuming the
//! qualified rule returns nothing, the result is a syntax error. So the entry
//! points that parse exactly one rule yield no rule for such text, whatever
//! the prelude would mean as a selector. The diagnostic code, action and byte
//! offset are Surgeist's choice: an `InvalidQualifiedRule` rejection of the
//! input, reported at the `--` ident that begins the prelude.
use surgeist_css::CssParser;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::{CssRule, CssRuleSyntax};
use surgeist_css::selector::CssNamespaceContext;

const LOOK_ALIKES: [&str; 3] = ["--x:{a}", "--x:hover{a}", "--x : { a } "];

#[test]
fn a_custom_property_look_alike_given_to_the_single_rule_entry_is_a_syntax_error() {
    for source in LOOK_ALIKES {
        let report = CssRule::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection for {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidQualifiedRule
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn trivia_before_a_custom_property_look_alike_moves_the_reported_offset_to_the_ident() {
    let source = " /* c */ --x:{a}";
    let report = CssRule::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
    );
    assert!(report.syntax().is_none(), "{report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("--x").unwrap()
    );
}

#[test]
fn a_custom_property_look_alike_is_no_rule_candidate_either() {
    for source in LOOK_ALIKES {
        let report = CssRuleSyntax::classify(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejection for {source}: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidQualifiedRule
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    }
}

#[test]
fn in_a_stylesheet_the_look_alike_is_dropped_alone_and_the_next_rule_survives() {
    let source = "--x:{a} b{color:red}";
    let report = CssParser::default().parse_sheet(source);
    let [CssRule::Style(_)] = report.syntax().rules() else {
        panic!("the rule after the look-alike survives: {report:?}");
    };
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped unit: {report:?}");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        "--x:{a}".len()
    );
}

#[test]
fn in_a_style_rule_the_same_text_is_a_custom_property_declaration() {
    // §5.5.6: a declaration of a custom property keeps its `{}`-block value
    // whole (step 8), so no rule is read there.
    for source in [".k{--x:{a}}", ".k{--x:hover{a}; color:red}"] {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("a style rule for {source}");
        };
        assert!(rule.rules().is_empty(), "{source}");
        assert!(
            rule.declarations().as_slice()[0].custom().is_some(),
            "{source}"
        );
    }
}
