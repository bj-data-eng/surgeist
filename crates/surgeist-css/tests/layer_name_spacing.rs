use surgeist_css::{CssNamespaceContext, CssRule, parse_rule};

// Cascade 5 (13 January 2022), section 6.4.2 expressly forbids intervening
// whitespace in dotted layer names. Boundary whitespace and comments differ.
#[test]
fn layer_names_reject_whitespace_before_a_period() {
    for source in ["@layer foo .bar;", "@layer foo\n.bar {}"] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean(), "{source}");
    }
}

#[test]
fn layer_names_reject_whitespace_after_a_period() {
    for source in ["@layer foo. bar;", "@layer foo.\tbar {}"] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}");
        assert!(!report.is_clean(), "{source}");
    }
}

#[test]
fn layer_names_keep_comments_and_boundary_whitespace() {
    for source in [
        "@layer foo.bar ;",
        "@layer foo/**/.bar;",
        "@layer foo./**/bar;",
    ] {
        let report = parse_rule(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let Some(CssRule::LayerStatement(rule)) = report.syntax() else {
            panic!("{source}: expected layer statement");
        };
        assert_eq!(rule.names().names()[0].components(), &["foo", "bar"]);
    }
}
