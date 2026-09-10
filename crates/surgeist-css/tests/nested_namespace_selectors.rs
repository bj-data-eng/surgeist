//! Selectors 4 section 16 permits an empty namespace prefix before a type or
//! universal selector. Nesting 1 sections 3.1/4 permit a following parent anchor.
//! A single namespace bar is distinct from the two-bar column combinator.

use surgeist_css::{
    CssCompoundSelector, CssNamespaceConstraint, CssPseudoClass, CssRecoveryAction, CssRule,
    CssSelector, parse_sheet,
};

fn compound(selector: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(compound) = selector else {
        panic!("expected a retained namespace-aware compound: {selector:?}");
    };
    compound
}

fn assert_explicit_none(compound: &CssCompoundSelector, universal: bool, anchors: usize) {
    let name = compound.type_selector().expect("qualified type selector");
    assert_eq!(name.namespace(), &CssNamespaceConstraint::ExplicitNone);
    assert_eq!(name.is_universal(), universal);
    assert_eq!(
        name.local_name(),
        if universal { None } else { Some("div") }
    );
    assert_eq!(compound.nesting_selectors(), anchors);
}

#[test]
fn nested_type_and_universal_selectors_preserve_explicit_no_namespace() {
    for (selector, universal, anchors) in [
        ("|div&", false, 1),
        ("|*&&", true, 2),
        ("|div", false, 0),
        ("|*", true, 0),
    ] {
        let source = format!(".parent, #alternate {{ {selector} {{color:green}} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("expected one parent");
        };
        assert_eq!(parent.selectors().selectors().len(), 2);
        let [CssRule::Style(child)] = parent.rules() else {
            panic!("expected one authored child");
        };
        assert_explicit_none(
            compound(child.selectors().selectors()[0].selector()),
            universal,
            anchors,
        );
        assert_eq!(child.declarations().len(), 1);
    }
}

#[test]
fn relational_selectors_preserve_single_bar_namespace_prefixes() {
    for (selector, anchors) in [("|div", 0), ("|div&", 1)] {
        let source = format!(".parent {{ &:has({selector}) {{color:green}} }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("expected parent");
        };
        let [CssRule::Style(child)] = parent.rules() else {
            panic!("expected child");
        };
        let outer = compound(child.selectors().selectors()[0].selector());
        assert_eq!(outer.nesting_selectors(), 1);
        let [CssPseudoClass::Has(list)] = outer.pseudo_classes() else {
            panic!("expected relational selector");
        };
        assert_eq!(list.selectors().len(), 1);
        assert_explicit_none(compound(list.selectors()[0].selector()), false, anchors);
    }
}

#[test]
fn incomplete_namespace_names_still_discard_only_the_child_rule() {
    for selector in ["|&", "|.class&", "| div&"] {
        let source = format!(".parent {{ {selector} {{color:green}} color:red }}");
        let report = parse_sheet(&source);
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("expected retained parent: {source}");
        };
        assert!(!report.is_clean(), "{source}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropQualifiedRule })
        );
        assert!(
            parent
                .rules()
                .iter()
                .all(|rule| !matches!(rule, CssRule::Style(_)))
        );
        let declarations = parent.declarations().len()
            + parent
                .rules()
                .iter()
                .map(|rule| match rule {
                    CssRule::NestedDeclarations(run) => run.declarations().len(),
                    _ => 0,
                })
                .sum::<usize>();
        assert_eq!(
            declarations, 1,
            "following declaration must survive: {source}"
        );
    }
}
