//! Selectors 4 section 16 permits an empty namespace prefix before a type or
//! universal selector. Nesting 1 sections 3.1/4 permit a following parent anchor.
//! A single namespace bar is distinct from the two-bar column combinator.

use surgeist_css::{
    CssCompoundSelector, CssErrorCode, CssNamespaceConstraint, CssPseudoClass, CssRecoveryAction,
    CssRecoveryDiagnostic, CssRule, CssSelector, CssSelectorError, ErrorKind, parse_sheet,
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
fn incomplete_namespace_names_discard_only_the_child_rule() {
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

// Selectors 4 §16 (WD 2026-01-22): "a simple selector containing an undeclared
// namespace prefix is invalid", so its rule is an invalid selector at every
// level. Syntax 3 (2026 CRD) §5.5.5 "consume a block's contents" reparses a
// failed ident-started declaration as a nested qualified rule; that rule's
// prelude is the selector, so its failure is the one reported.
// https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
fn selector_failure(source: &str, unit: &str) -> (usize, CssRecoveryDiagnostic) {
    let report = parse_sheet(source);
    let failures = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
        .collect::<Vec<_>>();
    let [diagnostic] = failures.as_slice() else {
        panic!("{source}: one dropped rule: {:?}", report.diagnostics());
    };
    let start = source.find(unit).unwrap();
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        start,
        "{source}"
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len(),
        "{source}"
    );
    (start, (*diagnostic).clone())
}

fn selector_detail(diagnostic: &CssRecoveryDiagnostic) -> &CssSelectorError {
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("an invalid selector: {diagnostic:?}");
    };
    detail
}

#[test]
fn undeclared_namespace_prefix_is_the_same_invalid_selector_at_every_level() {
    for unit in ["zz|a {}", "zz|* {}", "[zz|href] {}", "a[zz|href] {}"] {
        let (top_start, top) = selector_failure(unit, unit);
        assert_eq!(top_start, 0);
        assert_eq!(top.error().code(), CssErrorCode::InvalidSelector, "{unit}");
        let top_detail = selector_detail(&top).clone();
        let relative = top.error().position().byte_offset().value();
        for context in [
            ".card { UNIT }",
            ".card { color: red; UNIT color: blue; }",
            ".card { @media screen { UNIT } }",
            ".card { @scope (.x) { UNIT } }",
            "@media screen { UNIT }",
            "@scope (.x) { UNIT }",
        ] {
            let source = context.replace("UNIT", unit);
            let (start, nested) = selector_failure(&source, unit);
            assert_eq!(
                nested.error().code(),
                CssErrorCode::InvalidSelector,
                "{source}: {nested:?}"
            );
            assert_eq!(selector_detail(&nested), &top_detail, "{source}");
            assert_eq!(
                nested.error().position().byte_offset().value(),
                start + relative,
                "{source}"
            );
        }
    }
}
