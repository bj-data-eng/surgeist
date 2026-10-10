#![forbid(unsafe_code)]
//! New public raw-input boundary: CSSOM selectorText consumes retained syntax,
//! not clean-report validation. Expectations follow CSSOM's complete-input
//! selector algorithm, Nesting 1 anchors and Cascade 6 scope-body anchor meaning.
use surgeist_css::*;

fn clean(source: &str, context: CssStyleSelectorContext) -> CssParsedStyleSelectors {
    parse_style_selector_list(source, &CssNamespaceContext::default(), context)
        .into_validation_result()
        .unwrap()
        .expect("complete valid selector list")
}

fn ordinary(parsed: &CssParsedStyleSelectors) -> &CssStyleSelectorList {
    let CssAdmittedStyleSelectors::Ordinary(list) = parsed.selectors() else {
        panic!("ordinary nesting domain")
    };
    list
}

fn scoped(parsed: &CssParsedStyleSelectors) -> &CssScopedStyleSelectorList {
    let CssAdmittedStyleSelectors::Scoped(list) = parsed.selectors() else {
        panic!("scope anchor domain")
    };
    list
}

fn compound(selector: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(compound) = selector else {
        panic!("authored compound: {selector:?}")
    };
    compound
}

#[test]
fn ordinary_and_nested_inputs_preserve_explicit_and_implicit_anchor_meaning() {
    let input = ".implicit, &&.explicit, :is(&.child, .other)";
    for context in [
        CssStyleSelectorContext::Ordinary,
        CssStyleSelectorContext::Nested,
    ] {
        let parsed = clean(input, context);
        assert_eq!(parsed.context(), context);
        let [implicit, explicit, function] = ordinary(&parsed).selectors() else {
            panic!("all authored members")
        };
        assert_eq!(implicit.selector(), &CssSelector::Class("implicit".into()));
        assert_eq!(compound(explicit.selector()).nesting_selectors(), 2);
        assert_eq!(compound(explicit.selector()).scope_anchors(), 0);
        let CssSelector::PseudoClass(CssPseudoClass::Is(list)) = function.selector() else {
            panic!("typed inner list")
        };
        let CssPseudoSelectorListItem::Selector(inner) = &list.items()[0] else {
            panic!("valid explicit inner anchor")
        };
        assert_eq!(compound(inner).nesting_selectors(), 1);
        assert_eq!(compound(inner).scope_anchors(), 0);
    }
    // A leading relationship is legal only in the destination's nested/scoped
    // grammar. The implicit parent relationship remains symbolic, never bound
    // to a manufactured containing rule or selector list.
    let source = "> .child, + .sibling, .implicit";
    assert!(
        parse_style_selector_list(
            source,
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Ordinary
        )
        .syntax()
        .is_none()
    );
    let nested = clean(source, CssStyleSelectorContext::Nested);
    let [
        CssStyleSelector::Relative(child),
        CssStyleSelector::Relative(sibling),
        implicit,
    ] = ordinary(&nested).selectors()
    else {
        panic!("relative relationships and implicit member")
    };
    assert_eq!(child.combinator(), CssSelectorCombinator::Child);
    assert_eq!(sibling.combinator(), CssSelectorCombinator::NextSibling);
    assert_eq!(child.selector(), &CssSelector::Class("child".into()));
    assert_eq!(implicit.selector(), &CssSelector::Class("implicit".into()));
}

#[test]
fn scope_body_anchors_do_not_become_nesting_anchors_when_a_style_ancestor_exists() {
    for ancestry in [CssStyleAncestor::Absent, CssStyleAncestor::Present] {
        let context = CssStyleSelectorContext::Scoped(ancestry);
        let parsed = clean("&:is(&.child, .other), > &&.next, .implicit", context);
        assert_eq!(parsed.context(), context);
        let [
            CssScopedStyleSelector::Selector(first),
            CssScopedStyleSelector::Relative(next),
            CssScopedStyleSelector::Selector(implicit),
        ] = scoped(&parsed).selectors()
        else {
            panic!("scope-context members")
        };
        assert_eq!(compound(first).scope_anchors(), 1);
        assert_eq!(compound(first).nesting_selectors(), 0);
        let [CssPseudoClass::Is(list)] = compound(first).pseudo_classes() else {
            panic!("scope-context forgiving function")
        };
        let CssPseudoSelectorListItem::Selector(inner) = &list.items()[0] else {
            panic!("scoped inner member")
        };
        assert_eq!(compound(inner).scope_anchors(), 1);
        assert_eq!(compound(inner).nesting_selectors(), 0);
        assert_eq!(next.combinator(), CssSelectorCombinator::Child);
        assert_eq!(compound(next.selector()).scope_anchors(), 2);
        assert_eq!(compound(next.selector()).nesting_selectors(), 0);
        assert_eq!(implicit, &CssSelector::Class("implicit".into()));
    }
    // A style nested inside the scoped style creates ordinary nesting context.
    let nested = clean("&.child", CssStyleSelectorContext::Nested);
    assert_eq!(
        compound(ordinary(&nested).selectors()[0].selector()).nesting_selectors(),
        1
    );
    assert_eq!(
        compound(ordinary(&nested).selectors()[0].selector()).scope_anchors(),
        0
    );
}

#[test]
fn scoped_relative_names_reuse_the_supplied_namespace_environment() {
    let namespaces = CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("svg").unwrap()),
        CssNamespaceName::new("urn:svg"),
    )]);
    let report = parse_style_selector_list(
        "> svg|leaf&",
        &namespaces,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
    );
    assert!(report.is_clean(), "{report:?}");
    let [CssScopedStyleSelector::Relative(relative)] =
        scoped(report.syntax().as_ref().unwrap()).selectors()
    else {
        panic!("one scoped relative name")
    };
    let selector = compound(relative.selector());
    assert_eq!(
        selector.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    assert_eq!(selector.scope_anchors(), 1);
    assert_eq!(selector.nesting_selectors(), 0);
}

#[test]
fn namespace_aware_relative_payload_survives_input_drop_with_one_genuine_occurrence() {
    const RAW: &str = "/*😀*/\r\n> svg|leaf:lang(en), |leaf, *|leaf";
    let namespaces = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ]);
    let parsed = {
        let input = RAW.to_owned();
        parse_style_selector_list(&input, &namespaces, CssStyleSelectorContext::Nested)
            .into_validation_result()
            .unwrap()
            .unwrap()
    };
    assert_eq!(parsed.origin().source().as_str(), RAW);
    assert_eq!(parsed.origin().span().start().byte_offset().value(), 0);
    assert_eq!(
        parsed.origin().span().end().byte_offset().value(),
        RAW.len()
    );
    let [CssStyleSelector::Relative(first), second, third] = ordinary(&parsed).selectors() else {
        panic!("all namespace-aware members")
    };
    let first = compound(first.selector());
    assert_eq!(
        first.type_selector().unwrap().namespace(),
        &CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap())
    );
    assert_eq!(
        compound(second.selector())
            .type_selector()
            .unwrap()
            .namespace(),
        &CssNamespaceConstraint::ExplicitNone
    );
    assert_eq!(
        compound(third.selector())
            .type_selector()
            .unwrap()
            .namespace(),
        &CssNamespaceConstraint::Any
    );
    let [CssPseudoClass::Lang(language)] = first.pseudo_classes() else {
        panic!("parsed language token")
    };
    let CssValueOrigin::Parsed(origin) = language.ranges()[0].origin() else {
        panic!("original raw language origin")
    };
    assert!(origin.source().same_snapshot(parsed.origin().source()));
    assert_eq!(
        origin.span().start().byte_offset().value(),
        RAW.find("en").unwrap()
    );
    assert_eq!(origin.span().start().line().value(), 1);
    assert_eq!(origin.span().start().column().value(), 16);
    let default_type =
        parse_style_selector_list("leaf", &namespaces, CssStyleSelectorContext::Nested)
            .into_validation_result()
            .unwrap()
            .unwrap();
    assert_eq!(
        compound(ordinary(&default_type).selectors()[0].selector())
            .type_selector()
            .unwrap()
            .namespace(),
        &CssNamespaceConstraint::Default
    );
    assert!(
        parse_style_selector_list(
            RAW,
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Nested
        )
        .syntax()
        .is_none()
    );
}

#[test]
fn complete_input_failure_is_atomic_and_reports_original_responsible_positions() {
    for context in [
        CssStyleSelectorContext::Ordinary,
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    ] {
        for source in [
            "",
            " /**/ ",
            ".ok;",
            ".ok {}",
            ".ok,",
            ".ok, :unknown(x)",
            ".ok)",
            ".ok { } .later",
        ] {
            let report =
                parse_style_selector_list(source, &CssNamespaceContext::default(), context);
            assert!(report.syntax().is_none(), "{source}: {report:?}");
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|d| d.action() == CssRecoveryAction::RejectInput),
                "{report:?}"
            );
        }
    }
    let source = "/*😀*/\r\n/*é*/:unknown(x)";
    let report = parse_style_selector_list(
        source,
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Nested,
    );
    assert!(report.syntax().is_none());
    let error = report
        .diagnostics()
        .iter()
        .find(|d| d.action() == CssRecoveryAction::RejectInput)
        .unwrap()
        .error();
    assert_eq!(error.position().byte_offset().value(), 17);
    assert_eq!(error.position().line().value(), 1);
    assert_eq!(error.position().column().value(), 6);
    assert!(matches!(error.kind(), ErrorKind::InvalidSelector(_)));
}

#[test]
fn forgiving_and_implicit_eof_recovery_remain_admitted_with_diagnostics() {
    for context in [
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
    ] {
        let report = parse_style_selector_list(
            ":is(.ok, :unknown(x), &.kept:unknown())",
            &CssNamespaceContext::default(),
            context,
        );
        assert!(report.syntax().is_some());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::DropSelectorListItem)
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::PreserveInvalidSelectorListItem)
        );
        assert!(report.into_validation_result().is_err());
        let eof = parse_style_selector_list(":is(.ok", &CssNamespaceContext::default(), context);
        assert!(eof.syntax().is_some());
        assert!(
            eof.diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(eof.into_validation_result().is_err());
        let lexical = parse_style_selector_list(
            ":is(.ok, [x=\"bad\n\"])",
            &CssNamespaceContext::default(),
            context,
        );
        assert!(
            lexical.syntax().is_none(),
            "bad inner string rejects the whole lexical envelope"
        );
    }
}

#[test]
fn bounded_failure_distinguishes_resource_from_syntax_and_allows_unchanged_retry() {
    let source = "> :is(.a, &.b), .c /**/";
    let namespaces = CssNamespaceContext::default();
    for context in [
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    ] {
        for (limits, expected) in [
            (
                CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap(),
                CssComponentValueErrorKind::ByteLimit,
            ),
            (
                CssComponentValueLimits::try_new(256, 1, source.len()).unwrap(),
                CssComponentValueErrorKind::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(0, usize::MAX, source.len()).unwrap(),
                CssComponentValueErrorKind::NestingLimit,
            ),
        ] {
            let failure =
                parse_style_selector_list_with_limits(source, &namespaces, context, limits);
            assert!(failure.syntax().is_none());
            assert!(failure.diagnostics().iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(error) if error.kind() == expected)), "{failure:?}");
            assert!(
                failure
                    .diagnostics()
                    .iter()
                    .all(|d| d.action() != CssRecoveryAction::RejectInput),
                "resource is not an ordinary selector syntax no-op: {failure:?}"
            );
            let retry = parse_style_selector_list_with_limits(
                source,
                &namespaces,
                context,
                CssComponentValueLimits::try_new(256, 100, source.len()).unwrap(),
            );
            assert!(retry.is_clean(), "{retry:?}");
            assert_eq!(
                retry.syntax().as_ref().unwrap().origin().source().as_str(),
                source
            );
            assert_eq!(
                retry.syntax(),
                parse_style_selector_list(source, &namespaces, context).syntax()
            );
        }
    }
}

#[test]
fn component_allowance_covers_all_members_descendants_and_root_trivia_once() {
    // Public component units: two colon tokens, two function nodes, four
    // inner class tokens, one comma and one root whitespace node = ten.
    let source = ":is(.a), :is(.b)";
    let context = CssStyleSelectorContext::Nested;
    let namespaces = CssNamespaceContext::default();
    let fail = parse_style_selector_list_with_limits(
        source,
        &namespaces,
        context,
        CssComponentValueLimits::try_new(1, 9, source.len()).unwrap(),
    );
    assert!(fail.syntax().is_none());
    assert!(fail.diagnostics().iter().any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(error) if error.kind() == CssComponentValueErrorKind::ComponentLimit)));
    let pass = parse_style_selector_list_with_limits(
        source,
        &namespaces,
        context,
        CssComponentValueLimits::try_new(1, 10, source.len()).unwrap(),
    );
    assert!(pass.is_clean(), "{pass:?}");
    assert_eq!(
        ordinary(pass.syntax().as_ref().unwrap()).selectors().len(),
        2
    );
}

#[test]
fn admitted_carriers_feed_checked_edited_views_without_constructing_authored_rules() {
    let declarations = CssSpecifiedDeclarationBlock::try_from_declarations(
        parse_declaration_block_contents("").syntax(),
    )
    .unwrap();
    let parsed = clean(".Consumer > .leaf", CssStyleSelectorContext::Ordinary);
    let view = CssEditedRuleView::try_style(ordinary(&parsed), &declarations, &[]).unwrap();
    assert_eq!(view.serialize_cssom().unwrap(), ".Consumer > .leaf { }");
    assert!(view.parsed_rule().is_none());
    let parsed = clean(
        "&.scoped",
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    );
    let view = CssEditedRuleView::try_scoped_style(scoped(&parsed), &declarations, &[]).unwrap();
    assert_eq!(view.to_specified_css().unwrap(), "&.scoped { }");
    assert!(view.parsed_rule().is_none());
}
