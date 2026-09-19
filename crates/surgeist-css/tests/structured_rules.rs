//! The authored rule tree is a public semantic contract: one authored rule remains one
//! node, independent of the number of selectors or declarations surrounding child rules.
//!
//! CSS Nesting 1 §3 and §5 retain nested rules and declaration runs in their parent:
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/

use surgeist_css::{CssRule, parse_sheet};

#[test]
fn selector_list_retains_one_authored_rule() {
    let source = ".card, #featured { color: red; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "a selector list belongs to one authored style rule"
    );
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("expected the authored style rule");
    };
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn nested_rule_and_surrounding_declarations_stay_in_the_authored_parent() {
    let source = ".card { color: red; & .title { color: blue; } color: green; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "nested syntax must not become independent stylesheet rules"
    );
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("expected the authored style rule");
    };
    assert_eq!(rule.position().byte_offset().value(), 0);
}

#[test]
fn nested_conditional_remains_in_its_authored_parent() {
    let source = ".card { color: red; @media screen { color: blue; } color: green; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(
        report.syntax().rules().len(),
        1,
        "a nested conditional retains its authored parent and order"
    );
    assert!(matches!(report.syntax().rules()[0], CssRule::Style(_)));
}

#[test]
fn nested_selector_context_preserves_the_entire_parent_list_without_multiplication() {
    let source = ".card, #featured { &:hover, &.selected { .title, .subtitle { color: red; } } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("expected one authored parent");
    };
    assert_eq!(parent.selectors().selectors().len(), 2);
    assert!(
        matches!(parent.selectors().selectors()[0].selector(), surgeist_css::CssSelector::Class(name) if name == "card")
    );
    assert!(
        matches!(parent.selectors().selectors()[1].selector(), surgeist_css::CssSelector::Key(name) if name == "featured")
    );
    let [CssRule::Style(child)] = parent.rules() else {
        panic!("the nested selector list remains one child");
    };
    assert_eq!(child.selectors().selectors().len(), 2);
    for selector in child.selectors().selectors() {
        let surgeist_css::CssSelector::Compound(compound) = selector.selector() else {
            panic!("expected symbolic parent anchor");
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert!(!compound.has_scope_anchor());
        assert!(
            compound.ids().is_empty(),
            "the parent ID stays on the parent list"
        );
    }
    let [CssRule::Style(grandchild)] = child.rules() else {
        panic!("the descendant selector list remains one grandchild");
    };
    assert_eq!(grandchild.selectors().selectors().len(), 2);
    assert_eq!(grandchild.declarations().len(), 1);
    assert_eq!(
        child.position().byte_offset().value(),
        source.find("&:hover").unwrap()
    );
    assert_eq!(
        grandchild.position().byte_offset().value(),
        source.find(".title").unwrap()
    );
}

#[test]
fn declaration_runs_preserve_parent_pseudo_elements_order_importance_and_provenance() {
    let source = "a, a::before { color: red; & .title { color: blue; } color: green !important; @media screen { color: black; } color: white; }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("expected authored parent");
    };
    assert_eq!(parent.selectors().selectors().len(), 2);
    assert!(
        parent.selectors().selectors()[1]
            .selector()
            .has_pseudo_elements()
    );
    assert_eq!(parent.declarations().len(), 1);
    let [
        CssRule::Style(child),
        CssRule::NestedDeclarations(green),
        CssRule::Media(media),
        CssRule::NestedDeclarations(white),
    ] = parent.rules()
    else {
        panic!("expected nested style, declaration run, media, declaration run");
    };
    assert_eq!(child.declarations().len(), 1);
    assert_eq!(
        green.declarations()[0].importance(),
        surgeist_css::CssImportance::Important
    );
    assert_eq!(
        green.position().byte_offset().value(),
        source.find("color: green").unwrap()
    );
    assert_eq!(
        white.position().byte_offset().value(),
        source.find("color: white").unwrap()
    );
    let [CssRule::NestedDeclarations(black)] = media.rules() else {
        panic!("conditional declarations inherit the same parent context");
    };
    assert_eq!(
        black.position().byte_offset().value(),
        source.find("color: black").unwrap()
    );
}

#[test]
fn invalid_nested_rules_do_not_hide_valid_declarations_or_later_children() {
    let source = ".card { color: red; .bad, { color: blue; } color: green; .child { opacity: 1; } color: black; }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("expected retained authored parent");
    };
    assert_eq!(parent.declarations().len(), 1);
    let [
        CssRule::NestedDeclarations(before),
        CssRule::Style(child),
        CssRule::NestedDeclarations(after),
    ] = parent.rules()
    else {
        panic!("expected separate runs around the valid child");
    };
    assert_eq!(before.declarations().len(), 1);
    assert_eq!(child.declarations().len(), 1);
    assert_eq!(after.declarations().len(), 1);
    for (declaration, expected) in [
        (&parent.declarations()[0], "red"),
        (&before.declarations()[0], "green"),
        (&child.declarations()[0], "1"),
        (&after.declarations()[0], "black"),
    ] {
        assert_eq!(
            declaration
                .value_components()
                .serialize()
                .unwrap()
                .as_css()
                .trim(),
            expected
        );
    }
    assert_eq!(
        before.position().byte_offset().value(),
        source.find("color: green").unwrap()
    );
    assert_eq!(
        after.position().byte_offset().value(),
        source.find("color: black").unwrap()
    );
}
