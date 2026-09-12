use surgeist_css::{CssNamespaceContext, CssRule, parse_rule, parse_sheet};

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
fn layer_lists_keep_whitespace_around_commas() {
    let report = parse_rule(
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
    for invalid in ["@layer foo . bar;", "@layer foo. bar {}"] {
        let source = format!("{invalid} @layer valid.name {{ a {{ color: red; }} }}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        let [CssRule::LayerBlock(rule)] = report.syntax().rules() else {
            panic!("{source}: expected only the valid layer block");
        };
        assert_eq!(rule.name().unwrap().components(), &["valid", "name"]);
        assert_eq!(rule.rules().len(), 1);
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
