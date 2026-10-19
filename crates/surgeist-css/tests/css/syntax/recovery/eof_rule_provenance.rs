#![forbid(unsafe_code)]

use surgeist_css::{CssKnownProperty, CssRecoveryAction, CssRule, ErrorKind, parse_sheet};

// The public nesting-limit contract identifies the enclosing grammar production.
// CSS Syntax token identity is case-insensitive for at-rule names, including
// escaped spellings: https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#at-rules
// EOF recovery must retain that identity and the original source coordinates.
fn assert_eof_limit(at_keyword: &str, condition: &str, production: &str) {
    let prelude = format!("{at_keyword} ");
    assert_eof_limit_after_prelude(&prelude, condition, production);
}

fn assert_eof_limit_after_prelude(prelude: &str, condition: &str, production: &str) {
    let before = ".before{}\n";
    let source = format!("{before}{prelude}{}{condition}", "(".repeat(257));
    let report = parse_sheet(&source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("{prelude}: {:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        before.len() + prelude.len() + 256,
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        before.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    let ErrorKind::NestingLimit(limit) = diagnostic.error().kind() else {
        panic!("expected a typed nesting limit");
    };
    assert_eq!(limit.limit(), 256);
    assert_eq!(
        limit.enclosing_production().as_str(),
        production,
        "{prelude}"
    );
}

#[test]
fn eof_container_limit_retains_its_rule_production() {
    for name in ["@container", "@CoNtAiNeR", r"@con\74 ainer"] {
        assert_eof_limit(name, "width > 1px", "baseline.rule.container");
    }
}

#[test]
fn eof_media_limit_uses_decoded_at_keyword_identity() {
    for name in ["@MeDiA", r"@m\65 dia"] {
        assert_eof_limit(name, "width > 1px", "baseline.media.query-list");
    }
}

#[test]
fn eof_supports_limit_uses_decoded_at_keyword_identity() {
    for name in ["@SuPpOrTs", r"@s\75 pports"] {
        assert_eof_limit(name, "display: grid", "baseline.rule.supports");
    }
}

#[test]
fn canonical_media_and_supports_eof_limits_keep_their_productions() {
    assert_eof_limit("@media", "width > 1px", "baseline.media.query-list");
    assert_eof_limit("@supports", "display: grid", "baseline.rule.supports");
}

#[test]
fn completed_container_operands_keep_the_following_operand_inside_the_rule() {
    for prelude in [
        "@container (width > 1px) and ",
        "@CoNtAiNeR style(--x) or ",
        r"@con\74 ainer (not style(--x)) or ",
    ] {
        assert_eof_limit_after_prelude(prelude, "height > 2px", "baseline.rule.container");
    }
}

#[test]
fn completed_media_and_supports_operands_retain_rule_provenance_at_eof() {
    assert_eof_limit_after_prelude(
        "@media (width > 1px) and ",
        "height > 2px",
        "baseline.media.query-list",
    );
    assert_eof_limit_after_prelude(
        r"@s\75 pports (display: grid) and ",
        "display: flex",
        "baseline.rule.supports",
    );
}

#[test]
fn completed_semicolon_and_curly_rules_survive_a_later_eof_limit() {
    let before = "@layer first; .before{}\n";
    let prelude = "@container (width > 1px) and ";
    let source = format!("{before}{prelude}{}height > 2px", "(".repeat(257));
    let report = parse_sheet(&source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::LayerStatement(_), CssRule::Style(_)]
    ));
    let [diagnostic] = report.diagnostics() else {
        panic!("{:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        before.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    let ErrorKind::NestingLimit(limit) = diagnostic.error().kind() else {
        panic!("typed limit");
    };
    assert_eq!(
        limit.enclosing_production().as_str(),
        "baseline.rule.container"
    );
}

fn assert_later_declaration_eof_limit(group: Option<&str>) {
    let group_start = group.unwrap_or("");
    let depth = if group.is_some() { 2 } else { 1 };
    let prefix = format!("{group_start}.before{{}}.target{{margin:1px;--bomb:");
    let source = format!("{prefix}{}x", "f(".repeat(257 - depth));
    let report = parse_sheet(&source);
    let children = if group.is_some() {
        let [parent] = report.syntax().rules() else {
            panic!("parent rule must survive: {report:?}");
        };
        match parent {
            CssRule::Media(parent) => parent.rules(),
            CssRule::Supports(parent) => parent.rules(),
            CssRule::Container(parent) => parent.rules(),
            _ => panic!("expected original conditional parent"),
        }
    } else {
        report.syntax().rules()
    };
    let [CssRule::Style(before), CssRule::Style(target)] = children else {
        panic!("earlier and enclosing style rules must survive: {report:?}");
    };
    assert!(before.declarations().is_empty());
    assert_eq!(target.declarations().len(), 1, "{report:?}");
    assert_eq!(
        target.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Margin
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("only the offending declaration diagnoses: {report:?}");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("--bomb:").unwrap()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        prefix.len() + 2 * (256 - depth)
    );
    let ErrorKind::NestingLimit(limit) = diagnostic.error().kind() else {
        panic!("typed limit");
    };
    assert_eq!(limit.limit(), 256);
    assert_eq!(limit.enclosing_production().as_str(), "css.declaration");
}

#[test]
fn a_later_declaration_eof_limit_keeps_the_earlier_declaration() {
    assert_later_declaration_eof_limit(None);
}

#[test]
fn conditional_preludes_do_not_claim_eof_limits_from_their_bodies() {
    for group in [
        "@media all{",
        "@supports (display: grid){",
        "@container (width > 1px){",
    ] {
        assert_later_declaration_eof_limit(Some(group));
    }
}
