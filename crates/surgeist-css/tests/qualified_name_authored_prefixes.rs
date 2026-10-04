#![forbid(unsafe_code)]
//! Namespaces 3 §4: preserve authored prefix forms independently of their
//! effective namespace constraints, while each host owns grammar and recovery.
//! https://www.w3.org/TR/2014/REC-css-namespaces-3-20140320/#css-qnames

use surgeist_css::{
    CssErrorCode, CssNamespaceConstraint, CssNamespaceContext, CssNamespaceName,
    CssNamespacePrefix, CssNormalizedItem, CssRecoveryAction, CssRule, CssRuleContextKindRef,
    CssSelector, CssSelectorCombinator, normalize_report, parse_selector, parse_selector_list,
    parse_sheet, validate_sheet,
};

fn context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("svg").unwrap()),
        CssNamespaceName::new("urn:svg"),
    )])
}

fn selector(source: &str, context: &CssNamespaceContext) -> CssSelector {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().expect("one retained selector")
}

#[test]
fn unqualified_and_explicit_any_universals_remain_distinct_without_a_default_binding() {
    let context = CssNamespaceContext::default();
    for suffix in ["", ".mark"] {
        let unqualified = selector(&format!("*{suffix}"), &context);
        let explicit_any = selector(&format!("*|*{suffix}"), &context);
        for parsed in [&unqualified, &explicit_any] {
            let CssSelector::Compound(compound) = parsed else {
                panic!("authored universal retained: {parsed:?}");
            };
            let name = compound.type_selector().unwrap();
            assert!(name.is_universal());
            assert_eq!(name.namespace(), &CssNamespaceConstraint::Any);
        }
        // Equal effective constraints must not erase the authored `*|` prefix.
        assert_ne!(unqualified, explicit_any);
    }
}

#[test]
fn unqualified_and_explicit_empty_attribute_prefixes_remain_distinct() {
    for context in [
        CssNamespaceContext::default(),
        CssNamespaceContext::from_bindings([(None, CssNamespaceName::new(""))]),
        CssNamespaceContext::from_bindings([(None, CssNamespaceName::new("urn:default"))]),
    ] {
        let unqualified = selector("[href]", &context);
        let explicit_none = selector("[|href]", &context);
        for parsed in [&unqualified, &explicit_none] {
            let CssSelector::Compound(compound) = parsed else {
                panic!("authored attribute retained: {parsed:?}");
            };
            let [attribute] = compound.attributes() else {
                panic!("one attribute retained: {parsed:?}");
            };
            assert_eq!(attribute.name().as_str(), "href");
            assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);
        }
        // The default never applies to attributes, but authored `|` remains observable.
        assert_ne!(unqualified, explicit_none);
    }
}

#[test]
fn normalization_keeps_authored_prefix_distinctions_with_invalid_rule_diagnostics_and_origins() {
    for (left, right) in [("*", "*|*"), ("[href]", "[|href]")] {
        let invalid = "missing|leaf{}";
        let source = format!("/*😀*/\r\n{invalid}\r\n{left}{{}}\r\n{right}{{}}");
        let report = parse_sheet(&source);
        let [CssRule::Style(first), CssRule::Style(second)] = report.syntax().rules() else {
            panic!("two valid authored siblings retained: {report:?}");
        };
        let [diagnostic] = report.diagnostics() else {
            panic!("only the undeclared-prefix rule is diagnosed: {report:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let invalid_start = source.find(invalid).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            invalid_start
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            invalid_start + invalid.len()
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );

        let normalized = normalize_report(&report).unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let styles = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Rule(rule) => match rule.kind() {
                    CssRuleContextKindRef::Style(selectors) => Some((rule, selectors)),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(styles.len(), 2);
        for ((normalized_rule, selectors), authored, line) in
            [(styles[0], first, 2), (styles[1], second, 3)]
        {
            assert_eq!(normalized_rule.position(), Some(authored.position()));
            assert_eq!(authored.position().line().value(), line);
            assert_eq!(authored.position().column().value(), 0);
            assert_eq!(selectors.selectors().len(), 1);
            assert_eq!(
                selectors.selectors()[0].selector(),
                authored.selectors().selectors()[0].selector()
            );
        }
        assert_ne!(
            styles[0].1.selectors()[0].selector(),
            styles[1].1.selectors()[0].selector()
        );
    }
}

#[test]
fn qualified_names_allow_comments_but_keep_whitespace_and_local_wildcards_host_specific() {
    let context = context();
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    for (source, expected_namespace, expected_local) in [
        (
            r"s\76 g/**/|/**/leaf.mark",
            CssNamespaceConstraint::Named(prefix.clone()),
            Some("leaf"),
        ),
        (
            "svg/**/|/**/*",
            CssNamespaceConstraint::Named(prefix.clone()),
            None,
        ),
        ("*/**/|/**/leaf", CssNamespaceConstraint::Any, Some("leaf")),
        (
            "|/**/leaf",
            CssNamespaceConstraint::ExplicitNone,
            Some("leaf"),
        ),
    ] {
        let parsed = selector(source, &context);
        let CssSelector::Compound(compound) = parsed else {
            panic!("qualified name: {source}");
        };
        let name = compound.type_selector().unwrap();
        assert_eq!(name.namespace(), &expected_namespace);
        assert_eq!(name.local_name(), expected_local);
    }
    for source in ["[svg/**/|/**/href]", "[*/**/|/**/href]", "[|/**/href]"] {
        let parsed = selector(source, &context);
        let CssSelector::Compound(compound) = parsed else {
            panic!("qualified attribute: {source}");
        };
        assert_eq!(compound.attributes()[0].name().as_str(), "href");
    }
    for source in [
        "svg| leaf",
        "*| *",
        "| leaf",
        "[svg |href]",
        "[svg| href]",
        "[* |href]",
        "[| href]",
        "[svg|*]",
        "[*|*]",
        "[|*]",
    ] {
        let report = parse_selector(source, &context);
        assert!(!report.is_clean(), "{source}: {report:?}");
        assert!(report.syntax().is_none(), "{source}: {report:?}");
    }
    // Whitespace may instead separate two valid compounds; it cannot be
    // diagnosed as whitespace inside one qualified name.
    let parsed = selector("svg |leaf", &context);
    let CssSelector::Complex(complex) = parsed else {
        panic!("descendant compounds");
    };
    assert_eq!(
        complex.first().type_selector().unwrap().local_name(),
        Some("svg")
    );
    let [part] = complex.rest() else {
        panic!("one descendant compound");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Descendant);
    assert_eq!(
        part.selector().type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
}

#[test]
fn undeclared_and_ignored_prefixes_recover_at_the_selected_host_unit() {
    let context = context();
    let strict = parse_selector_list("svg|leaf,missing|leaf", &context);
    assert!(strict.syntax().is_none());
    assert_eq!(strict.diagnostics().len(), 1);
    assert_eq!(
        strict.diagnostics()[0].action(),
        CssRecoveryAction::RejectInput
    );
    let forgiving = parse_selector(":is(svg|leaf,missing|leaf)", &context);
    assert_eq!(forgiving.diagnostics().len(), 1);
    assert_eq!(
        forgiving.diagnostics()[0].action(),
        CssRecoveryAction::DropSelectorListItem
    );
    let Some(CssSelector::PseudoClass(surgeist_css::CssPseudoClass::Is(list))) = forgiving.syntax()
    else {
        panic!("retained forgiving host: {forgiving:?}");
    };
    assert_eq!(list.selectors(), [selector("svg|leaf", &context)]);
    let ignored = parse_sheet("@namespace svg ident;svg|leaf{}.after{}");
    assert!(matches!(ignored.syntax().rules(), [CssRule::Style(_)]));
    assert_eq!(ignored.diagnostics().len(), 2);
    assert_eq!(
        ignored.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
    assert_eq!(
        ignored.diagnostics()[1].action(),
        CssRecoveryAction::DropQualifiedRule
    );
    assert!(
        CssNamespaceContext::from_sheet(ignored.syntax())
            .named_namespace(&CssNamespacePrefix::try_new("svg").unwrap())
            .is_none()
    );
}
