#![forbid(unsafe_code)]

use surgeist_css::{CssRecoveryAction, CssRule, ErrorKind, parse_sheet};

// The public nesting-limit contract identifies the enclosing grammar production.
// CSS Syntax token identity is case-insensitive for at-rule names, including
// escaped spellings: https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#at-rules
// EOF recovery must retain that identity and the original source coordinates.
fn assert_eof_limit(at_keyword: &str, condition: &str, production: &str) {
    let before = ".before{}\n";
    let prelude = format!("{at_keyword} ");
    let source = format!("{before}{prelude}{}{condition}", "(".repeat(257));
    let report = parse_sheet(&source);
    assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    let [diagnostic] = report.diagnostics() else {
        panic!("{at_keyword}: {:?}", report.diagnostics());
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
        "{at_keyword}"
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
