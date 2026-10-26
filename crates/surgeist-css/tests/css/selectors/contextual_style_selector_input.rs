//! New public raw-input boundary: CSSOM selectorText consumes retained syntax,
//! not clean-report validation. Expectations follow CSSOM's complete-input
//! selector algorithm, Nesting 1 anchors and Cascade 6 scope-body anchor meaning.
use crate::css::support::color_and_selector_fixtures::{
    assert_selector_input_rejected, assert_strict_validation_rejects,
};
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::*;

fn clean(source: &str, context: CssStyleSelectorContext) -> CssParsedStyleSelectors {
    CssParsedStyleSelectors::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
        context,
    )
    .into_validation_result()
    .unwrap()
    .expect("complete valid selector list")
}

fn ordinary(parsed: &CssParsedStyleSelectors) -> &CssStyleSelectorList {
    assert!(
        !matches!(parsed.context(), CssStyleSelectorContext::Scoped(_)),
        "ordinary nesting domain"
    );
    parsed.selectors()
}

fn scoped(parsed: &CssParsedStyleSelectors) -> &CssStyleSelectorList {
    assert!(
        matches!(parsed.context(), CssStyleSelectorContext::Scoped(_)),
        "scope anchor domain"
    );
    parsed.selectors()
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
    assert_selector_input_rejected(
        source,
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Ordinary,
        ),
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
            CssStyleSelector::Selector(first),
            CssStyleSelector::Relative(next),
            CssStyleSelector::Selector(implicit),
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
    let report = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        "> svg|leaf&",
        &namespaces,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
    );
    assert!(report.is_clean(), "{report:?}");
    let [CssStyleSelector::Relative(relative)] =
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
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            &input,
            &namespaces,
            CssStyleSelectorContext::Nested,
        )
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
    let default_type = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        "leaf",
        &namespaces,
        CssStyleSelectorContext::Nested,
    )
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
    assert_selector_input_rejected(
        RAW,
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            RAW,
            &CssNamespaceContext::default(),
            CssStyleSelectorContext::Nested,
        ),
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
            assert_selector_input_rejected(
                source,
                CssParsedStyleSelectors::parse(
                    &CssParser::default(),
                    source,
                    &CssNamespaceContext::default(),
                    context,
                ),
            );
        }
    }
    let source = "/*😀*/\r\n/*é*/:unknown(x)";
    let report = CssParsedStyleSelectors::parse(
        &CssParser::default(),
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
    assert!(error.code() == CssErrorCode::InvalidSelector);
}

#[test]
fn forgiving_and_implicit_eof_recovery_remain_admitted_with_diagnostics() {
    for context in [
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
    ] {
        let report = CssParsedStyleSelectors::parse(
            &CssParser::default(),
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
        assert_strict_validation_rejects(report);
        let eof = CssParsedStyleSelectors::parse(
            &CssParser::default(),
            ":is(.ok",
            &CssNamespaceContext::default(),
            context,
        );
        assert!(eof.syntax().is_some());
        assert!(
            eof.diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert_strict_validation_rejects(eof);
        // A bad inner string rejects the whole lexical envelope.
        let lexical = ":is(.ok, [x=\"bad\n\"])";
        assert_selector_input_rejected(
            lexical,
            CssParsedStyleSelectors::parse(
                &CssParser::default(),
                lexical,
                &CssNamespaceContext::default(),
                context,
            ),
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
                ComponentFailure::ByteLimit,
            ),
            (
                CssComponentValueLimits::try_new(256, 1, source.len()).unwrap(),
                ComponentFailure::ComponentLimit,
            ),
            (
                CssComponentValueLimits::try_new(0, usize::MAX, source.len()).unwrap(),
                ComponentFailure::NestingLimit,
            ),
        ] {
            let failure = CssParsedStyleSelectors::parse(
                &CssParser::new(CssParserContext::default(), limits),
                source,
                &namespaces,
                context,
            );
            assert!(failure.syntax().is_none());
            assert!(
                failure.diagnostics().iter().any(|d| d
                    .error()
                    .component_error()
                    .is_some_and(|error| component_failure(error) == expected)),
                "{failure:?}"
            );
            assert!(
                failure
                    .diagnostics()
                    .iter()
                    .all(|d| d.action() != CssRecoveryAction::RejectInput),
                "resource is not an ordinary selector syntax no-op: {failure:?}"
            );
            let retry = CssParsedStyleSelectors::parse(
                &CssParser::new(
                    CssParserContext::default(),
                    CssComponentValueLimits::try_new(256, 100, source.len()).unwrap(),
                ),
                source,
                &namespaces,
                context,
            );
            assert!(retry.is_clean(), "{retry:?}");
            assert_eq!(
                retry.syntax().as_ref().unwrap().origin().source().as_str(),
                source
            );
            assert_eq!(
                retry.syntax(),
                CssParsedStyleSelectors::parse(&CssParser::default(), source, &namespaces, context)
                    .syntax()
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
    let fail = CssParsedStyleSelectors::parse(
        &CssParser::new(
            CssParserContext::default(),
            CssComponentValueLimits::try_new(1, 9, source.len()).unwrap(),
        ),
        source,
        &namespaces,
        context,
    );
    assert!(fail.syntax().is_none());
    assert!(fail.diagnostics().iter().any(|d| {
        d.error()
            .component_error()
            .is_some_and(|error| error.is_component_limit())
    }));
    let pass = CssParsedStyleSelectors::parse(
        &CssParser::new(
            CssParserContext::default(),
            CssComponentValueLimits::try_new(1, 10, source.len()).unwrap(),
        ),
        source,
        &namespaces,
        context,
    );
    assert!(pass.is_clean(), "{pass:?}");
    assert_eq!(
        ordinary(pass.syntax().as_ref().unwrap()).selectors().len(),
        2
    );
}

#[test]
fn admitted_carriers_feed_rule_graph_inputs_without_constructing_authored_rules() {
    let declarations = CssSpecifiedDeclarationBlock::try_from_declarations(
        CssParser::default().parse_block_contents("").syntax(),
        Default::default(),
    )
    .unwrap();
    let namespaces = CssNamespaceContext::default();
    let parsed = clean(".Consumer > .leaf", CssStyleSelectorContext::Ordinary);
    let input = CssRuleGraphInput::style(ordinary(&parsed), &declarations, &[]);
    assert_eq!(
        input
            .serialize_cssom(&namespaces, CssStyleAncestor::Absent, Default::default())
            .unwrap(),
        ".Consumer > .leaf { }"
    );
    let parsed = clean(
        "&.scoped",
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    );
    let input = CssRuleGraphInput::style(scoped(&parsed), &declarations, &[]);
    assert_eq!(
        input.serialize_specified(Default::default()).unwrap(),
        "&.scoped { }"
    );
}
