#![forbid(unsafe_code)]
//! Nesting 1 section 4: parentless & matches scope with zero anchor specificity.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nest-selector
use surgeist_css::{
    CssNamespaceContext, CssNormalizedItem, CssSelector, CssSelectorBinding, normalize_sheet,
    parse_selector, parse_selector_list, parse_sheet,
};

#[test]
fn ordinary_compounds_retain_unbound_anchors_without_scope_rewriting() {
    for (source, count) in [
        ("&", 1),
        ("&.a", 1),
        (".a&", 1),
        ("div&", 1),
        ("&&", 2),
        (".a&&", 2),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        let CssSelector::Compound(compound) = report.syntax().as_ref().unwrap() else {
            panic!("compound")
        };
        assert_eq!(compound.nesting_selectors(), count);
        assert!(!compound.has_scope_anchor());
        assert!(compound.pseudo_classes().is_empty());
        assert!(parse_selector_list(source, &CssNamespaceContext::default()).is_clean());
    }
}

#[test]
fn normalized_anchor_context_distinguishes_missing_and_real_parents() {
    for (source, has_parent) in [
        ("& {color:red}", false),
        (".parent { & {color:red} }", true),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 1);
        let context = declarations[0].selector_context();
        assert_eq!(context.parent().is_some(), has_parent);
        assert!(context.scope_context().is_none());
        if let Some(parent) = context.parent() {
            assert!(
                matches!(parent.selectors()[0].selector(), CssSelector::Class(name) if name == "parent")
            );
            assert!(!context.same_context(parent));
        }
        assert_eq!(
            context.selectors()[0].binding(),
            CssSelectorBinding::ExplicitAnchors
        );
        let CssSelector::Compound(compound) = context.selectors()[0].selector() else {
            panic!("compound")
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert!(!compound.has_scope_anchor());
    }
}

#[test]
fn unbound_anchor_admission_does_not_enable_relative_roots_or_trailing_tokens() {
    for source in ["&div", ">&", "&.a{}", "&.a;", ""] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
}

#[test]
fn supports_selector_anchor_is_typed_instead_of_general_enclosed() {
    let report = parse_sheet("@supports selector(&) { a { color:red } }");
    assert!(report.is_clean(), "{report:?}");
    let [surgeist_css::CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports")
    };
    let surgeist_css::CssSupportsConditionKind::Selector(CssSelector::Compound(compound)) =
        rule.condition().kind()
    else {
        panic!("typed selector, not opaque fallback")
    };
    assert_eq!(compound.nesting_selectors(), 1);
}

#[test]
fn functional_anchors_keep_their_function_structure_and_parentless_binding() {
    use surgeist_css::CssPseudoClass;
    for source in [
        ":is(&,.a)",
        ":where(&)",
        ":not(&)",
        ":has(> &)",
        ":nth-child(2n of &)",
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(
            report.clone().into_validation_result().unwrap(),
            *report.syntax()
        );
        let list = parse_selector_list(source, &CssNamespaceContext::default());
        assert!(list.is_clean(), "{source}: {list:?}");
        assert_eq!(
            list.clone().into_validation_result().unwrap(),
            *list.syntax()
        );
        let sheet = parse_sheet(&format!("{source} {{color:red}}"));
        assert!(sheet.is_clean(), "{source}: {sheet:?}");
        let normalized = normalize_sheet(sheet.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        let [declaration] = declarations.as_slice() else {
            panic!("one declaration: {source}");
        };
        let context = declaration.selector_context();
        assert!(context.parent().is_none());
        assert!(context.scope_context().is_none());
        assert_eq!(
            context.selectors()[0].binding(),
            CssSelectorBinding::ExplicitAnchors
        );
        let selector = context.selectors()[0].selector();
        let pseudo = match selector {
            CssSelector::PseudoClass(pseudo) => pseudo,
            CssSelector::Compound(compound) => {
                assert_eq!(compound.nesting_selectors(), 0);
                assert_eq!(compound.pseudo_classes().len(), 1);
                &compound.pseudo_classes()[0]
            }
            _ => panic!("preserved functional selector: {source}"),
        };
        let anchor = match (source, pseudo) {
            (":is(&,.a)", CssPseudoClass::Is(list)) => {
                assert_eq!(list.selectors().len(), 2);
                &list.selectors()[0]
            }
            (":where(&)", CssPseudoClass::Where(list)) | (":not(&)", CssPseudoClass::Not(list)) => {
                assert_eq!(list.selectors().len(), 1);
                &list.selectors()[0]
            }
            (":has(> &)", CssPseudoClass::Has(list)) => {
                assert_eq!(list.selectors().len(), 1);
                assert_eq!(
                    list.selectors()[0].combinator(),
                    surgeist_css::CssSelectorCombinator::Child
                );
                list.selectors()[0].selector()
            }
            (":nth-child(2n of &)", CssPseudoClass::NthChild(pattern)) => {
                let list = pattern.selector_list().unwrap();
                assert_eq!(list.selectors().len(), 1);
                &list.selectors()[0]
            }
            _ => panic!("function kind must remain authored: {source}"),
        };
        let CssSelector::Compound(compound) = anchor else {
            panic!("inner anchor")
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert!(!compound.has_scope_anchor());
        assert!(compound.pseudo_classes().is_empty());
    }
}

#[test]
fn repeated_anchors_and_scope_pseudo_class_remain_distinct_after_normalization() {
    let sheet = parse_sheet("&& {color:red} :scope {color:blue}");
    assert!(sheet.is_clean(), "{sheet:?}");
    let normalized = normalize_sheet(sheet.syntax()).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    let [anchors, scope] = declarations.as_slice() else {
        panic!("two declarations");
    };
    let anchors = anchors.selector_context();
    assert!(anchors.parent().is_none());
    assert_eq!(
        anchors.selectors()[0].binding(),
        CssSelectorBinding::ExplicitAnchors
    );
    let CssSelector::Compound(compound) = anchors.selectors()[0].selector() else {
        panic!("anchors")
    };
    assert_eq!(compound.nesting_selectors(), 2);
    assert!(!compound.has_scope_anchor());
    let scope = scope.selector_context();
    assert!(scope.parent().is_none());
    assert_eq!(scope.selectors()[0].binding(), CssSelectorBinding::Absolute);
    assert!(matches!(
        scope.selectors()[0].selector(),
        CssSelector::PseudoClass(surgeist_css::CssPseudoClass::Scope)
    ));
    assert!(!scope.same_context(anchors));
}

#[test]
fn anchors_preserve_original_rule_positions_and_error_spans() {
    let source = "/*😀*/\r\n& {color:red}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [surgeist_css::CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("style rule")
    };
    assert_eq!(rule.position().byte_offset().value(), 10);
    assert_eq!(rule.position().line().value(), 1);
    assert_eq!(rule.position().column().value(), 0);
    let name = rule.declarations()[0].parsed_name().unwrap();
    assert_eq!(name.source().as_str(), source);
    assert_eq!(name.span().start().byte_offset().value(), 13);
    let invalid = "/*😀*/\r\n&div";
    let report = parse_selector(invalid, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.action(),
        surgeist_css::CssRecoveryAction::RejectInput
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), 14);
}

#[test]
fn anchors_do_not_invent_types_or_discard_authored_namespace_constraints() {
    use surgeist_css::{CssNamespaceConstraint, CssNamespaceName, CssNamespacePrefix};
    let prefix = CssNamespacePrefix::try_new("svg").unwrap();
    let context = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (Some(prefix.clone()), CssNamespaceName::new("urn:svg")),
    ]);
    for context in [&context, &CssNamespaceContext::default()] {
        let report = parse_selector("&", context);
        assert!(report.is_clean(), "{report:?}");
        let CssSelector::Compound(compound) = report.syntax().as_ref().unwrap() else {
            panic!("anchor")
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert!(compound.type_selector().is_none());
    }
    for (source, namespace) in [
        ("svg|leaf&", CssNamespaceConstraint::Named(prefix)),
        ("|leaf&", CssNamespaceConstraint::ExplicitNone),
        ("*|leaf&", CssNamespaceConstraint::Any),
    ] {
        let report = parse_selector(source, &context);
        assert!(report.is_clean(), "{source}: {report:?}");
        let CssSelector::Compound(compound) = report.syntax().as_ref().unwrap() else {
            panic!("qualified anchor")
        };
        assert_eq!(compound.nesting_selectors(), 1);
        let name = compound.type_selector().unwrap();
        assert_eq!(name.namespace(), &namespace);
        assert_eq!(name.local_name(), Some("leaf"));
    }
    assert!(
        parse_selector("svg|leaf&", &CssNamespaceContext::default())
            .syntax()
            .is_none()
    );
}
