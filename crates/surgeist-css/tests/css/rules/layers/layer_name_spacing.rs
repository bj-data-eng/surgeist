use crate::css::support::rules::ExpectedRejection;
use surgeist_css::{CssErrorCode, CssNamespaceContext, CssParser, CssRule};

// Cascade 5 (13 January 2022), section 6.4.2 expressly forbids intervening
// whitespace in dotted layer names. Boundary whitespace and comments differ.

fn layer_prelude() -> ExpectedRejection {
    ExpectedRejection::new(CssErrorCode::InvalidAtRulePrelude).in_rule("layer")
}

#[test]
fn layer_names_reject_whitespace_before_a_period() {
    // Whitespace may precede a comma, so the period is the violating token.
    for source in ["@layer foo .bar;", "@layer foo\n.bar {}"] {
        let report = CssRule::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_none(), "{source}");
        layer_prelude().at(".").assert_first(&report);
    }
}

#[test]
fn layer_lists_keep_whitespace_around_commas() {
    let report = CssRule::parse(
        &CssParser::default(),
        "@layer foo.bar , baz.qux ;",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssRule::LayerStatement(rule)) = report.syntax() else {
        panic!("expected layer statement");
    };
    assert_eq!(rule.names().names().len(), 2);
    assert_eq!(rule.names().names()[0].components(), &["foo", "bar"]);
    assert_eq!(rule.names().names()[1].components(), &["baz", "qux"]);
}

#[test]
fn malformed_layer_names_do_not_discard_later_rules() {
    for (invalid, token) in [("@layer foo . bar;", "."), ("@layer foo. bar {}", " ")] {
        let source = format!("{invalid} @layer valid.name {{ a {{ color: red; }} }}");
        let report = CssParser::default().parse_sheet(&source);
        layer_prelude().at(token).assert_first(&report);
        let [CssRule::LayerBlock(rule)] = report.syntax().rules() else {
            panic!("{source}: expected only the valid layer block");
        };
        assert_eq!(rule.name().unwrap().components(), &["valid", "name"]);
        assert_eq!(rule.rules().len(), 1);
    }
}

#[test]
fn layer_names_reject_whitespace_after_a_period() {
    // An identifier must follow the period directly.
    for (source, token) in [("@layer foo. bar;", " "), ("@layer foo.\tbar {}", "\t")] {
        let report = CssRule::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.syntax().is_none(), "{source}");
        layer_prelude().at(token).assert_first(&report);
    }
}

#[test]
fn layer_names_keep_comments_and_boundary_whitespace() {
    for source in [
        "@layer foo.bar ;",
        "@layer foo/**/.bar;",
        "@layer foo./**/bar;",
    ] {
        let report = CssRule::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        );
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let Some(CssRule::LayerStatement(rule)) = report.syntax() else {
            panic!("{source}: expected layer statement");
        };
        assert_eq!(rule.names().names()[0].components(), &["foo", "bar"]);
    }
}
