//! The single real-brace fragment front, `CssBlockFragment::<B>::parse`, and
//! the parser configuration it reads (#1353 section 6.2).
//!
//! A body reads only the part of `CssBlockContext` it names; a rule list reads
//! its namespace bindings and style ancestry, and every other body reads nothing.
//! A feature-value definition block takes its kind as an argument.
use surgeist_css::component::{CssComponentValueLimits, CssComponentValues};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::property::{
    CssKnownProperty, CssPropertyNameRef, CssPropertyTarget, CssPropertyTextTarget,
};
use surgeist_css::rule::font_feature_values::{CssFontFeatureValueBlock, CssFontFeatureValueKind};
use surgeist_css::rule::keyframes::CssKeyframeDeclarationList;
use surgeist_css::rule::page::CssPageBody;
use surgeist_css::rule::{
    CssBlockContext, CssBlockFragment, CssNamespaceName, CssNamespacePrefix, CssRuleList,
    CssStyleAncestor,
};
use surgeist_css::selector::CssNamespaceContext;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

fn parser() -> CssParser {
    CssParser::default()
}

#[test]
fn a_rule_list_without_a_context_is_an_ordinary_group_without_bindings() {
    let namespaces = CssNamespaceContext::default();
    let source = " { .a { color: red } @media print { .b {} } } ";
    let plain =
        CssBlockFragment::<CssRuleList>::parse(&parser(), source, &CssBlockContext::default());
    let group = CssBlockFragment::<CssRuleList>::parse(
        &parser(),
        source,
        &CssBlockContext::group(&namespaces),
    );
    assert!(plain.is_clean(), "{plain:?}");
    assert_eq!(plain, group);
    let fragment = plain.syntax().as_ref().expect("a genuine block");
    assert_eq!(fragment.body().rules().len(), 2);
    let origin = fragment.origin().span();
    assert_eq!(
        (
            origin.start().byte_offset().value(),
            origin.end().byte_offset().value()
        ),
        (1, source.len() - 1)
    );
}

#[test]
fn a_namespace_binding_in_the_context_reaches_the_rules_of_the_list() {
    let source = "{ svg|rect { color: red } }";
    let unbound =
        CssBlockFragment::<CssRuleList>::parse(&parser(), source, &CssBlockContext::default());
    assert!(!unbound.is_clean(), "an undeclared prefix is a diagnostic");
    let namespaces = CssNamespaceContext::from_bindings([(
        CssNamespacePrefix::try_new("svg"),
        CssNamespaceName::new("http://www.w3.org/2000/svg"),
    )]);
    let bound = CssBlockFragment::<CssRuleList>::parse(
        &parser(),
        source,
        &CssBlockContext::group(&namespaces),
    );
    assert!(bound.is_clean(), "{bound:?}");
    assert_eq!(bound.syntax().as_ref().unwrap().body().rules().len(), 1);
}

#[test]
fn scope_and_scoped_group_contexts_select_their_own_page_placement() {
    // Cascade 6 scope bodies exclude `@page`; an ordinary group inside a scope
    // without a style ancestor admits it.
    let namespaces = CssNamespaceContext::default();
    let source = "{ @page { margin: 1in } }";
    let scope = CssBlockFragment::<CssRuleList>::parse(
        &parser(),
        source,
        &CssBlockContext::scope(&namespaces, CssStyleAncestor::Absent),
    );
    assert!(!scope.is_clean(), "{scope:?}");
    assert!(scope.syntax().as_ref().unwrap().body().rules().is_empty());
    let scoped_group = CssBlockFragment::<CssRuleList>::parse(
        &parser(),
        source,
        &CssBlockContext::scoped_group(&namespaces, CssStyleAncestor::Absent),
    );
    assert!(scoped_group.is_clean(), "{scoped_group:?}");
    assert_eq!(
        scoped_group.syntax().as_ref().unwrap().body().rules().len(),
        1
    );
}

#[test]
fn a_context_part_a_body_does_not_read_is_ignored() {
    let namespaces = CssNamespaceContext::default();
    let source = " { size: A4; @top-left { content: 'x' } } ";
    let plain =
        CssBlockFragment::<CssPageBody>::parse(&parser(), source, &CssBlockContext::default());
    for context in [
        CssBlockContext::group(&namespaces),
        CssBlockContext::scope(&namespaces, CssStyleAncestor::Present),
        CssBlockContext::scoped_group(&namespaces, CssStyleAncestor::Absent),
    ] {
        assert_eq!(
            CssBlockFragment::<CssPageBody>::parse(&parser(), source, &context),
            plain
        );
    }
}

#[test]
fn a_feature_value_definition_block_is_read_under_the_kind_it_is_given() {
    let source = "{ alt: 1; other: 2 }";
    for kind in [
        CssFontFeatureValueKind::Stylistic,
        CssFontFeatureValueKind::Styleset,
    ] {
        let report =
            CssBlockFragment::<CssFontFeatureValueBlock>::parse_definition(&parser(), source, kind);
        let block = report.syntax().as_ref().expect("a genuine block").body();
        assert_eq!(block.kind(), kind);
    }
}

#[test]
fn a_feature_value_definition_block_has_no_context_front_to_leave_the_kind_out() {
    // The kind is an argument of `parse_definition`; `parse` does not accept the
    // body at all (the `compile_fail` doctest on `CssBlockFragment::parse`).
    for (kind, source) in [
        (CssFontFeatureValueKind::Swash, "{ a: 1 }"),
        (CssFontFeatureValueKind::Styleset, "{ a: 1 2 }"),
    ] {
        let report =
            CssBlockFragment::<CssFontFeatureValueBlock>::parse_definition(&parser(), source, kind);
        assert!(report.is_clean(), "{kind:?}: {:?}", report.diagnostics());
    }
}

#[test]
fn the_parser_context_reaches_the_declarations_of_a_body() {
    // Color 4 Appendix B: only quirks mode admits the quirky hex color `ABC`.
    let source = "{ color: ABC }";
    let standards = CssBlockFragment::<CssKeyframeDeclarationList>::parse(
        &parser(),
        source,
        &CssBlockContext::default(),
    );
    assert!(!standards.is_clean(), "{standards:?}");
    let quirks = CssBlockFragment::<CssKeyframeDeclarationList>::parse(
        &CssParser::new(
            CssParserContext::new(CssParserMode::Quirks),
            Default::default(),
        ),
        source,
        &CssBlockContext::default(),
    );
    assert!(quirks.is_clean(), "{quirks:?}");
}

#[test]
fn a_page_margin_value_is_checked_from_components_because_the_text_front_names_no_page_target() {
    // `CssPropertyTextTarget` has only `Property` and `Grammar` (the `compile_fail`
    // doctest on `CssParser::parse_property_value`); a page value is checked from
    // owned components, and the text front serves the ordinary margin grammar.
    let margin = CssPropertyNameRef::Known(CssKnownProperty::Margin);
    let page = CssDeclaration::try_from_components(
        &parser(),
        CssPropertyTarget::PageProperty(margin),
        CssComponentValues::parse(&parser(), "1in").unwrap(),
        CssImportance::Normal,
    );
    assert!(page.is_ok(), "{page:?}");
    let ordinary = parser().parse_property_value(
        "1in",
        CssPropertyTextTarget::Property(margin),
        CssImportance::Normal,
    );
    assert!(ordinary.is_clean(), "{ordinary:?}");
    assert!(ordinary.syntax().is_some());
}

#[test]
fn the_component_limits_of_the_parser_charge_the_fronts_that_have_an_allowance() {
    let tight = CssParser::new(
        CssParserContext::default(),
        CssComponentValueLimits::try_new(256, usize::MAX, 8).unwrap(),
    );
    let source = "color: red; width: 2px";
    assert!(parser().parse_block_contents(source).is_clean());
    let refused = tight.parse_block_contents(source);
    assert!(refused.syntax().is_empty(), "{refused:?}");
    assert!(refused.diagnostics()[0].error().is_resource_refusal());
    assert!(CssComponentValues::parse(&tight, source).is_err());
}
