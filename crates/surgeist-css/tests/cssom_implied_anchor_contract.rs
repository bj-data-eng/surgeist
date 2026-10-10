#![forbid(unsafe_code)]
//! Test-only runtime checkpoint for #1078. Selected Nesting20260122 §6
//! absolutizes relative nested selectors; §3.1 preserves selectors containing
//! an ampersand, including retained invalid forgiving members. Compact authored
//! and general-relative output remain distinct.
use surgeist_css::{
    CssAdmittedStyleSelectors, CssEditedRuleView, CssNamespaceContext, CssRecoveryAction, CssRule,
    CssSpecifiedDeclarationBlock, CssStyleAncestor, CssStyleSelectorContext,
    parse_relative_selector_list, parse_sheet, parse_style_selector_list,
};

#[test]
fn nested_literal_implicit_descendant_and_each_leading_combinator_have_implied_anchor() {
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for (prelude, literal) in [
        (".child", "& .child"),
        ("> .child", "& > .child"),
        ("+ .child", "& + .child"),
        ("~ .child", "& ~ .child"),
        ("|| .child", "& || .child"),
    ] {
        let source = format!("/*😀*/ .parent {{{prelude}{{}}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{prelude}: {report:?}");
        let before = report.clone();
        let value = &report.syntax().rules()[0];
        assert_eq!(
            value.to_specified_css().unwrap(),
            format!(".parent {{ {prelude} {{ }} }}")
        );
        expected.push(format!(".parent {{\n  {literal} {{ }}\n}}"));
        actual.push(value.serialize_cssom().unwrap());
        assert_eq!(report, before);
    }
    assert_eq!(actual, expected);
}

#[test]
fn parsed_multilevel_children_add_their_own_implied_anchor_only_in_literal_output() {
    let report = parse_sheet(".outer { .middle { > .inner {} } }");
    assert!(report.is_clean());
    let before = report.clone();
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        ".outer { .middle { > .inner { } } }"
    );
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        ".outer {\n  & .middle {\n  & > .inner { }\n}\n}"
    );
    assert_eq!(report, before);
}

#[test]
fn checked_scoped_nested_role_absolutizes_relative_payload_without_mutating_source() {
    let source = " /*😀*/ > .child, + .sibling ";
    let report = parse_style_selector_list(
        source,
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    );
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let parsed = report.syntax().as_ref().unwrap();
    let CssAdmittedStyleSelectors::Scoped(list) = parsed.selectors() else {
        panic!("scoped")
    };
    let declarations = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    let view = CssEditedRuleView::try_scoped_style(list, &declarations, &[]).unwrap();
    assert_eq!(view.to_specified_css().unwrap(), "> .child, + .sibling { }");
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "& > .child, & + .sibling { }"
    );
    assert_eq!(parsed.origin().source().as_str(), source);
    assert_eq!(report, before);
}

#[test]
fn explicit_repeated_and_retained_invalid_anchors_are_controls_without_an_extra_anchor() {
    let report = parse_sheet(".parent { &.one {} &&.two {} .before & {} }");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        ".parent {\n  &.one { }\n  &&.two { }\n  .before & { }\n}"
    );
    let raw = parse_style_selector_list(
        "&:is(.ok,&.bad:unknown())",
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Nested,
    );
    assert!(raw.syntax().is_some());
    assert!(
        raw.diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::PreserveInvalidSelectorListItem)
    );
    let CssAdmittedStyleSelectors::Ordinary(list) = raw.syntax().as_ref().unwrap().selectors()
    else {
        panic!("ordinary")
    };
    let declarations = CssSpecifiedDeclarationBlock::try_from_entries(&[]).unwrap();
    let view = CssEditedRuleView::try_style(list, &declarations, &[]).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        "&:is(.ok,&.bad:unknown()) { }"
    );
    // Nesting §3.1 counts the ampersand even when it only occurs in an
    // invalid retained member: that selector is already non-relative.
    let raw = parse_style_selector_list(
        ":is(.ok,&.bad:unknown())",
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Nested,
    );
    let CssAdmittedStyleSelectors::Ordinary(list) = raw.syntax().as_ref().unwrap().selectors()
    else {
        panic!("ordinary")
    };
    let view = CssEditedRuleView::try_style(list, &declarations, &[]).unwrap();
    assert_eq!(
        view.serialize_cssom().unwrap(),
        ":is(.ok,&.bad:unknown()) { }"
    );
}

#[test]
fn general_relative_output_and_strict_rejection_are_unchanged_controls() {
    let namespaces = CssNamespaceContext::default();
    let report = parse_relative_selector_list(
        ".implicit, > .child, + .next, ~ .later, || .column",
        &namespaces,
    );
    assert!(report.is_clean());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        ".implicit, > .child, + .next, ~ .later, || .column"
    );
    assert!(
        parse_style_selector_list("> .child", &namespaces, CssStyleSelectorContext::Ordinary)
            .syntax()
            .is_none()
    );
    assert!(
        parse_style_selector_list(
            ":not(.ok,&.bad:unknown())",
            &namespaces,
            CssStyleSelectorContext::Nested
        )
        .syntax()
        .is_none()
    );
    let report = parse_sheet(".plain {}");
    let [CssRule::Style(_)] = report.syntax().rules() else {
        panic!("ordinary style")
    };
    assert_eq!(report.syntax().serialize_cssom().unwrap(), ".plain { }");
}
