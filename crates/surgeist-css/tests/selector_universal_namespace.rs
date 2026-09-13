#![forbid(unsafe_code)]
//! Selectors 4 section 5.2: a namespace-qualified universal selector restricts
//! the elements matched even when another simple selector follows it.
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#universal-selector

use surgeist_css::{
    CssNamespaceConstraint, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssPseudoClass, CssSelector, parse_selector,
};

fn context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (
            Some(CssNamespacePrefix::try_new("svg").unwrap()),
            CssNamespaceName::new("urn:svg"),
        ),
    ])
}

fn assert_universal(selector: &CssSelector, expected: &CssNamespaceConstraint, suffix: &str) {
    let CssSelector::Compound(compound) = selector else {
        panic!("universal namespace must survive compatibility conversion: {selector:?}");
    };
    let name = compound
        .type_selector()
        .expect("retained universal selector");
    assert_eq!(name.local_name(), None);
    assert_eq!(name.namespace(), expected);
    match suffix {
        ".card" => assert_eq!(compound.classes(), &["card"]),
        "#card" => assert_eq!(compound.ids(), &["card"]),
        ":hover" => assert!(matches!(compound.pseudo_classes(), [CssPseudoClass::Hover])),
        _ => panic!("unexpected test suffix"),
    }
}

fn assert_namespace(prefix: &str, expected: CssNamespaceConstraint) {
    for suffix in [".card", "#card", ":hover"] {
        let input = format!("{prefix}*{suffix}");
        let report = parse_selector(&input, &context());
        assert!(report.is_clean(), "{input}: {report:?}");
        assert_universal(report.syntax().as_ref().unwrap(), &expected, suffix);
    }
}

#[test]
fn named_universals_retain_namespace_before_class_id_and_pseudo() {
    assert_namespace(
        "svg|",
        CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap()),
    );
}

#[test]
fn empty_namespace_universals_retain_constraint_before_class_id_and_pseudo() {
    assert_namespace("|", CssNamespaceConstraint::ExplicitNone);
}

#[test]
fn default_namespace_universals_retain_constraint_before_class_id_and_pseudo() {
    assert_namespace("", CssNamespaceConstraint::Default);
}

#[test]
fn explicit_any_namespace_universals_remain_authored_before_class_id_and_pseudo() {
    assert_namespace("*|", CssNamespaceConstraint::Any);
}

#[test]
fn logical_selector_arguments_retain_universal_namespace_constraints() {
    let report = parse_selector(":is(svg|*.card, |*#card, *:hover)", &context());
    assert!(report.is_clean(), "{report:?}");
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(list))) = report.syntax() else {
        panic!("expected logical selector: {report:?}");
    };
    assert_eq!(list.selectors().len(), 3);
    for (selector, expected, suffix) in [
        (
            &list.selectors()[0],
            CssNamespaceConstraint::Named(CssNamespacePrefix::try_new("svg").unwrap()),
            ".card",
        ),
        (
            &list.selectors()[1],
            CssNamespaceConstraint::ExplicitNone,
            "#card",
        ),
        (
            &list.selectors()[2],
            CssNamespaceConstraint::Default,
            ":hover",
        ),
    ] {
        assert_universal(selector, &expected, suffix);
    }
}

#[test]
fn absent_type_selectors_keep_compatible_simple_variants() {
    let context = CssNamespaceContext::default();
    for (input, expected) in [
        (".card", CssSelector::Class("card".into())),
        ("#card", CssSelector::Key("card".into())),
        (":hover", CssSelector::PseudoClass(CssPseudoClass::Hover)),
    ] {
        let report = parse_selector(input, &context);
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(report.syntax(), &Some(expected));
    }
}
