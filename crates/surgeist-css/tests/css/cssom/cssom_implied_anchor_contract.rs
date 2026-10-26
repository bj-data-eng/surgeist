//! Test-only runtime checkpoint for #1078. Selected Nesting20260122 §6
//! absolutizes relative nested selectors; §3.1 preserves selectors containing
//! an ampersand, including retained invalid forgiving members. Compact authored
//! and general-relative output remain distinct.
use surgeist_css::{
    CssNamespaceContext, CssParsedStyleSelectors, CssParser, CssRecoveryAction,
    CssRelativeSelectorList, CssRule, CssRuleGraphInput, CssSpecifiedDeclarationBlock,
    CssStyleAncestor, CssStyleSelectorContext,
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
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.is_clean(), "{prelude}: {report:?}");
        let before = report.clone();
        let value = &report.syntax().rules()[0];
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            format!(".parent {{ {prelude} {{ }} }}")
        );
        expected.push(format!(".parent {{\n  {literal} {{ }}\n}}"));
        actual.push(value.serialize_cssom(Default::default()).unwrap());
        assert_eq!(report, before);
    }
    assert_eq!(actual, expected);
}

#[test]
fn parsed_multilevel_children_add_their_own_implied_anchor_only_in_literal_output() {
    let report = CssParser::default().parse_sheet(".outer { .middle { > .inner {} } }");
    assert!(report.is_clean());
    let before = report.clone();
    assert_eq!(
        report
            .syntax()
            .serialize_specified(Default::default())
            .unwrap(),
        ".outer { .middle { > .inner { } } }"
    );
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        ".outer {\n  & .middle {\n  & > .inner { }\n}\n}"
    );
    assert_eq!(report, before);
}

#[test]
fn checked_scoped_nested_role_absolutizes_relative_payload_without_mutating_source() {
    let source = " /*😀*/ > .child, + .sibling ";
    let report = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        source,
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    );
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let parsed = report.syntax().as_ref().unwrap();
    assert!(matches!(
        parsed.context(),
        CssStyleSelectorContext::Scoped(_)
    ));
    let list = parsed.selectors();
    let declarations =
        CssSpecifiedDeclarationBlock::try_from_entries(&[], Default::default()).unwrap();
    let input = CssRuleGraphInput::style(list, &declarations, &[]);
    assert_eq!(
        input.serialize_specified(Default::default()).unwrap(),
        "> .child, + .sibling { }"
    );
    assert_eq!(
        input
            .serialize_cssom(
                &CssNamespaceContext::default(),
                CssStyleAncestor::Absent,
                Default::default()
            )
            .unwrap(),
        "& > .child, & + .sibling { }"
    );
    assert_eq!(parsed.origin().source().as_str(), source);
    assert_eq!(report, before);
}

#[test]
fn explicit_repeated_and_retained_invalid_anchors_are_controls_without_an_extra_anchor() {
    let report = CssParser::default().parse_sheet(".parent { &.one {} &&.two {} .before & {} }");
    assert!(report.is_clean());
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        ".parent {\n  &.one { }\n  &&.two { }\n  .before & { }\n}"
    );
    let raw = CssParsedStyleSelectors::parse(
        &CssParser::default(),
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
    let list = raw.syntax().as_ref().unwrap().selectors();
    let declarations =
        CssSpecifiedDeclarationBlock::try_from_entries(&[], Default::default()).unwrap();
    let input = CssRuleGraphInput::style(list, &declarations, &[]);
    assert_eq!(
        input
            .serialize_cssom(
                &CssNamespaceContext::default(),
                CssStyleAncestor::Absent,
                Default::default()
            )
            .unwrap(),
        "&:is(.ok,&.bad:unknown()) { }"
    );
    // Nesting §3.1 counts the ampersand even when it only occurs in an
    // invalid retained member: that selector is already non-relative.
    let raw = CssParsedStyleSelectors::parse(
        &CssParser::default(),
        ":is(.ok,&.bad:unknown())",
        &CssNamespaceContext::default(),
        CssStyleSelectorContext::Nested,
    );
    let list = raw.syntax().as_ref().unwrap().selectors();
    let input = CssRuleGraphInput::style(list, &declarations, &[]);
    assert_eq!(
        input
            .serialize_cssom(
                &CssNamespaceContext::default(),
                CssStyleAncestor::Absent,
                Default::default()
            )
            .unwrap(),
        ":is(.ok,&.bad:unknown()) { }"
    );
}

#[test]
fn general_relative_output_and_strict_rejection_are_unchanged_controls() {
    let namespaces = CssNamespaceContext::default();
    let report = CssRelativeSelectorList::parse(
        &CssParser::default(),
        ".implicit, > .child, + .next, ~ .later, || .column",
        &namespaces,
    );
    assert!(report.is_clean());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        ".implicit, > .child, + .next, ~ .later, || .column"
    );
    assert!(
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            "> .child",
            &namespaces,
            CssStyleSelectorContext::Ordinary
        )
        .syntax()
        .is_none()
    );
    assert!(
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            ":not(.ok,&.bad:unknown())",
            &namespaces,
            CssStyleSelectorContext::Nested
        )
        .syntax()
        .is_none()
    );
    let report = CssParser::default().parse_sheet(".plain {}");
    let [CssRule::Style(_)] = report.syntax().rules() else {
        panic!("ordinary style")
    };
    assert_eq!(
        report.syntax().serialize_cssom(Default::default()).unwrap(),
        ".plain { }"
    );
}
