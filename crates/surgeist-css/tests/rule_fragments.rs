#![forbid(unsafe_code)]
//! Syntax 3 exact-one admission, with CSS-owned typed grammar and inner recovery.
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix, CssRecoveryAction,
    CssRule, CssSelector, CssStyleSelector, parse_rule,
};

fn parse(source: &str) -> surgeist_css::CssParseReport<Option<CssRule>> {
    parse_rule(source, &CssNamespaceContext::default())
}

fn reject(source: &str) {
    let report = parse(source);
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    let [diagnostic] = report.diagnostics() else {
        panic!("one outer rejection: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn extra_rules_and_outer_delimiters_reject_the_complete_original_source() {
    for source in [
        "",
        " /**/ ",
        "a{} b{}",
        "a{} @unknown x;",
        "@unknown x; a{}",
        "a{} ;",
        "a{} }",
        "a{} )",
        "a{} ]",
        "a{} tail",
        "a{unknown:x} b{}",
        "@media{} ;",
        "@unknown x;",
        ">@a{}",
    ] {
        reject(source);
    }
    let source = "@media{} ;";
    let report = parse(source);
    let error = report.diagnostics()[0].error();
    assert_eq!(error.code(), CssErrorCode::UnexpectedToken);
    assert_eq!(error.position().byte_offset().value(), 9);
    let report = parse("@media;");
    assert!(report.syntax().is_none());
    assert_eq!(
        report.diagnostics()[0].error().code(),
        CssErrorCode::InvalidAtRuleBody
    );
}

#[test]
fn leading_comments_preserve_the_invalid_at_rule_owner() {
    for source in ["/**/@media;", "/*{*/@media;", " /*😀*/\r\n@media;"] {
        let report = parse(source);
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("one outer rejection")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidAtRuleBody);
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn isolated_top_level_grammar_retains_typed_rule_variants() {
    for (source, kind) in [
        ("a{color:red}", "style"),
        ("&{color:red}", "style"),
        ("@media screen{a{color:red}}", "media"),
        ("@supports (color:red){a{color:red}}", "supports"),
        ("@import 'theme.css';", "import"),
        ("@namespace svg 'urn:svg';", "namespace"),
        ("@layer reset,theme;", "layer_statement"),
        ("@layer theme{a{color:red}}", "layer_block"),
        ("@keyframes fade{from{opacity:0}to{opacity:1}}", "keyframes"),
    ] {
        let report = parse(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let actual = match report.syntax().as_ref().unwrap() {
            CssRule::Style(_) => "style",
            CssRule::Media(_) => "media",
            CssRule::Supports(_) => "supports",
            CssRule::Import(_) => "import",
            CssRule::Namespace(_) => "namespace",
            CssRule::LayerStatement(_) => "layer_statement",
            CssRule::LayerBlock(_) => "layer_block",
            CssRule::Keyframes(_) => "keyframes",
            _ => panic!("expected selected kind"),
        };
        assert_eq!(actual, kind);
    }
}

#[test]
fn inner_recovery_retains_the_parent_and_its_own_diagnostics() {
    let source = "a{unknown:x;color:red}";
    let report = parse(source);
    let Some(CssRule::Style(rule)) = report.syntax() else {
        panic!("style: {report:?}")
    };
    assert_eq!(rule.declarations().len(), 1);
    let [diagnostic] = report.diagnostics() else {
        panic!("one inner error")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 2);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 12);
    let report = parse("@media screen{@unknown x;a{color:red}}");
    let Some(CssRule::Media(rule)) = report.syntax() else {
        panic!("media: {report:?}")
    };
    assert!(matches!(rule.rules(), [CssRule::Style(_)]));
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn eof_closure_is_published_only_for_retained_outer_rules() {
    let source = "a{color:red";
    let report = parse(source);
    assert!(matches!(report.syntax(), Some(CssRule::Style(_))));
    let [diagnostic] = report.diagnostics() else {
        panic!("one EOF closure: {report:?}")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    reject("a{color:red} b{");
}

#[test]
fn namespaces_and_unicode_positions_come_from_the_supplied_context_and_source() {
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([(
        Some(prefix.clone()),
        CssNamespaceName::new("urn:svg"),
    )]);
    let source = "/*😀*/\r\nsvg|leaf&{color:red}";
    let report = parse_rule(source, &context);
    assert!(report.is_clean(), "{report:?}");
    let Some(CssRule::Style(rule)) = report.syntax() else {
        panic!("style")
    };
    assert_eq!(rule.position().byte_offset().value(), 10);
    assert_eq!(rule.position().line().value(), 1);
    assert_eq!(rule.position().column().value(), 0);
    let [CssStyleSelector::Selector(CssSelector::Compound(compound))] =
        rule.selectors().selectors()
    else {
        panic!("qualified compound")
    };
    assert_eq!(compound.nesting_selectors(), 1);
    assert_eq!(
        compound.type_selector().unwrap().namespace(),
        &surgeist_css::CssNamespaceConstraint::Named(prefix.clone())
    );
    assert_eq!(
        rule.declarations()[0]
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
    assert!(parse(source).syntax().is_none());
    let namespace = parse_rule("@namespace svg 'urn:replacement';", &context);
    assert!(namespace.is_clean());
    assert_eq!(
        context.named_namespace(&prefix).unwrap().as_str(),
        "urn:svg"
    );
}

#[test]
fn encoding_metadata_and_stylesheet_sentinels_are_not_stripped_from_raw_rules() {
    reject("@charset \"utf-8\";");
    reject("<!--a{}-->");
    reject("a{} -->");
    // A Unicode BOM in this str is an identifier character, not a byte-stream marker.
    let report = parse("\u{feff}a{}");
    assert!(report.is_clean(), "{report:?}");
    let Some(CssRule::Style(rule)) = report.syntax() else {
        panic!("style")
    };
    let [CssStyleSelector::Selector(CssSelector::Tag(name))] = rule.selectors().selectors() else {
        panic!("authored type selector")
    };
    assert_eq!(name, "\u{feff}a");
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn nested_rules_and_later_declarations_keep_their_owning_order() {
    let report = parse("a{color:red;&{width:1px}color:blue}");
    assert!(report.is_clean(), "{report:?}");
    let Some(CssRule::Style(rule)) = report.syntax() else {
        panic!("style")
    };
    assert_eq!(rule.declarations().len(), 1);
    assert!(matches!(
        rule.rules(),
        [CssRule::Style(_), CssRule::NestedDeclarations(_)]
    ));
}

#[test]
fn bounded_deep_rules_retain_inner_resource_failures_without_losing_the_parent() {
    for depth in [63, 64, 255] {
        let source = format!(
            "{}a{{color:red}}{}",
            "@media screen{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let mut rule = report.syntax().as_ref().unwrap();
        for _ in 0..depth {
            let CssRule::Media(media) = rule else {
                panic!("media chain")
            };
            let [child] = media.rules() else {
                panic!("one child")
            };
            rule = child;
        }
        assert!(matches!(rule, CssRule::Style(_)));
    }
    let source = format!(
        "{}a{{color:red}}{}",
        "@media screen{".repeat(256),
        "}".repeat(256)
    );
    let report = parse(&source);
    assert!(matches!(report.syntax(), Some(CssRule::Media(_))));
    assert!(!report.is_clean());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::StopAtNestingLimit)
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
}

#[test]
fn style_and_scope_depths_preserve_supported_content_and_typed_limits() {
    for depth in [63, 64, 255] {
        let source = format!("{}a{{color:red}}{}", "a{".repeat(depth), "}".repeat(depth));
        let report = parse(&source);
        assert!(
            report.is_clean(),
            "style depth {depth}: {:?}",
            report.diagnostics()
        );
        let mut rule = report.syntax().as_ref().unwrap();
        for _ in 0..depth {
            let CssRule::Style(style) = rule else {
                panic!("style chain")
            };
            let [child] = style.rules() else {
                panic!("one child")
            };
            rule = child;
        }
        let CssRule::Style(leaf) = rule else {
            panic!("style leaf")
        };
        assert_eq!(leaf.declarations().len(), 1);
        let source = format!(
            "{}a{{color:red}}{}",
            "@scope (.a){".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse(&source);
        assert!(
            report.is_clean(),
            "scope depth {depth}: {:?}",
            report.diagnostics()
        );
        let Some(CssRule::Scope(mut_scope)) = report.syntax() else {
            panic!("scope root")
        };
        let mut scope = mut_scope;
        for _ in 1..depth {
            let [surgeist_css::CssScopedRule::Scope(child)] = scope.rules().rules() else {
                panic!("scope chain")
            };
            scope = child;
        }
        let [surgeist_css::CssScopedRule::Style(leaf)] = scope.rules().rules() else {
            panic!("scope leaf")
        };
        assert_eq!(leaf.declarations().len(), 1);
    }
    for opening in ["a{", "@scope (.a){"] {
        let source = format!("{}a{{color:red}}{}", opening.repeat(256), "}".repeat(256));
        let report = parse(&source);
        assert!(report.syntax().is_some());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit),
            "{opening}: {:?}",
            report.diagnostics()
        );
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
    }
}

#[test]
fn scoped_and_non_ident_style_limits_keep_resource_diagnostics() {
    for opening in [".a{", "@scope (.a){"] {
        let source = format!(
            "{}.leaf{{color:red}}{}",
            opening.repeat(256),
            "}".repeat(256)
        );
        let report = parse(&source);
        assert!(report.syntax().is_some());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit),
            "{opening}: {:?}",
            report.diagnostics()
        );
    }
}

#[test]
fn qualified_resource_error_does_not_replace_the_next_declaration() {
    let source = format!(
        "{}leaf{{color:red}}color:blue{}",
        "a{".repeat(256),
        "}".repeat(256)
    );
    let report = parse(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one resource error: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 516);
    let mut rule = report.syntax().as_ref().unwrap();
    for _ in 1..256 {
        let CssRule::Style(style) = rule else {
            panic!("style parent")
        };
        let [child] = style.rules() else {
            panic!("one retained child")
        };
        rule = child;
    }
    let CssRule::Style(style) = rule else {
        panic!("innermost surviving style")
    };
    assert_eq!(style.declarations().len(), 1);
    assert!(style.rules().is_empty());
    assert_eq!(
        style.declarations()[0]
            .parsed_value()
            .unwrap()
            .span()
            .start()
            .byte_offset()
            .value(),
        533
    );
}
