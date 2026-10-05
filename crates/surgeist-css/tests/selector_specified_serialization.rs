#![forbid(unsafe_code)]
//! Independent authored selector-provider expectations. Rule traversal is separate.
use surgeist_css::{
    CssNamespaceContext, CssPseudoClass, CssPseudoSelectorList, CssRule, CssSelector,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, CssStyleSelector, parse_selector, parse_sheet,
};
fn parsed(source: &str) -> CssSelector {
    let report = parse_sheet(&format!("{source} {{ }}"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("style")
    };
    let [CssStyleSelector::Selector(selector)] = rule.selectors().selectors() else {
        panic!("selector")
    };
    selector.clone()
}
fn exact(source: &str, expected: &str) {
    let selector = parsed(source);
    let before = selector.clone();
    assert_eq!(selector.to_specified_css().unwrap(), expected);
    assert_eq!(selector, before);
    assert_eq!(parsed(expected).to_specified_css().unwrap(), expected);
}
#[test]
fn identifiers_attributes_and_combinators_have_meaningful_boundaries() {
    exact(
        r#"a#X.c[data-x='v' i] > b + c ~ d e"#,
        r#"a#X.c[data-x="v" i] > b + c ~ d e"#,
    );
    exact(
        r#"[lang|='en'][tag~='x'][name^='a'][name$='b'][name*='c' s]"#,
        r#"[lang|="en"][tag~="x"][name^="a"][name$="b"][name*="c" s]"#,
    );
}
#[test]
fn qualified_names_reuse_authored_prefix_identity() {
    let report = parse_sheet("@namespace n 'urn:x'; n|leaf[n|key='v']{} *|*[|key]{} |leaf{}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let expected = [r#"n|leaf[n|key="v"]"#, "*|*[|key]", "|leaf"];
    for (rule, expected) in report.syntax().rules()[1..].iter().zip(expected) {
        let CssRule::Style(rule) = rule else {
            panic!("style")
        };
        let CssStyleSelector::Selector(selector) = &rule.selectors().selectors()[0] else {
            panic!("selector")
        };
        assert_eq!(selector.to_specified_css().unwrap(), expected);
    }
}
#[test]
fn logical_lists_and_relative_has_preserve_order() {
    exact(
        ":is(.a,#b):not(.c):where(.d):has(>.x,+.y,~.z,.w)",
        ":is(.a, #b):not(.c):where(.d):has(> .x, + .y, ~ .z, .w)",
    );
}
#[test]
fn nth_patterns_follow_syntax_anb_serialization() {
    exact(
        ":nth-child(odd of .a,#b):nth-last-child(even):nth-of-type(-n + 3):nth-last-of-type(0n + 2)",
        ":nth-child(2n+1 of .a, #b):nth-last-child(2n):nth-of-type(-n+3):nth-last-of-type(2)",
    );
    let selector = CssSelector::PseudoClass(CssPseudoClass::NthOfType(
        surgeist_css::CssNthPattern::AnPlusB(surgeist_css::CssNthAnPlusB::new(-1, i32::MIN)),
    ));
    assert_eq!(
        selector.to_specified_css().unwrap(),
        ":nth-of-type(-n-2147483648)"
    );
}
#[test]
fn language_uses_quoted_cssom_arguments_and_dir_keeps_checked_identifier() {
    exact(
        r#":lang(en,"fr-CA"):dir(rtl)"#,
        r#":lang("en", "fr-CA"):dir(rtl)"#,
    );
}
#[test]
fn shadow_arguments_and_pseudo_element_attachment_survive() {
    exact(
        ":host(.a):host-context(.b)::part(foo bar):hover::before::marker",
        ":host(.a):host-context(.b)::part(foo bar):hover::before::marker",
    );
    exact("slot::slotted(.a)::before", "slot::slotted(.a)::before");
}
#[test]
fn constructed_names_escape_without_mutating_identity() {
    for (selector, expected) in [
        (CssSelector::Class("日本 x".into()), ".日本\\ x"),
        (CssSelector::Key("123".into()), "#\\31 23"),
        (CssSelector::Tag("a:b".into()), "a\\:b"),
    ] {
        let before = selector.clone();
        assert_eq!(selector.to_specified_css().unwrap(), expected);
        assert_eq!(selector, before);
    }
}
#[test]
fn limits_count_final_utf8_and_leave_the_graph_available_for_retry() {
    let selector = CssSelector::Class("日本".into());
    let expected = ".日本";
    let before = selector.clone();
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(1, 1, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
        (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            selector
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(selector, before);
    }
    assert_eq!(selector.to_specified_css().unwrap(), expected);
}
#[test]
fn finite_deep_and_wide_lists_use_incremental_checked_work() {
    let mut selector = CssSelector::Class("x".into());
    for _ in 0..255 {
        selector = CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![selector]).unwrap(),
        ));
    }
    let expected = format!("{}.x{}", ":is(".repeat(255), ")".repeat(255));
    assert_eq!(selector.to_specified_css().unwrap(), expected);
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(10, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    let wide = CssSelector::PseudoClass(CssPseudoClass::Is(
        CssPseudoSelectorList::try_new(vec![CssSelector::Class("x".into()); 8192]).unwrap(),
    ));
    assert_eq!(
        wide.to_specified_css_with_limits(Limits::new(2, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
}

#[test]
fn unchecked_scalar_names_cannot_silently_change_identity() {
    for selector in [
        CssSelector::Class(String::new()),
        CssSelector::Key("a\0b".into()),
        CssSelector::Tag(String::new()),
    ] {
        assert_eq!(
            selector.to_specified_css().unwrap_err().kind(),
            Kind::UnrepresentableValue
        );
    }
}

#[test]
fn enum_carrier_does_not_duplicate_pseudo_class_payload_work() {
    let selector = CssSelector::PseudoClass(CssPseudoClass::Root);
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(1, 1, 6))
            .unwrap(),
        ":root"
    );
}

#[test]
fn compound_attribute_and_logical_children_share_exact_semantic_node_budget() {
    // Compound, attribute, :is, its class, :nth-child, its An+B pattern,
    // and its `of` class each contribute one semantic visit.
    let expected = "[key]:is(.a):nth-child(2n+1 of .b)";
    let selector = parsed(expected);
    let before = selector.clone();
    assert_eq!(
        selector
            .to_specified_css_with_limits(Limits::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(6, 7, expected.len()), Kind::InputNodeLimit),
        (Limits::new(7, 6, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(7, 7, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            selector
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(selector, before);
    }
}

#[test]
fn symbolic_nesting_anchor_and_scope_pseudo_keep_distinct_authored_identity() {
    let report = parse_selector("& :scope", &CssNamespaceContext::default());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let selector = report.syntax().as_ref().unwrap();
    let before = selector.clone();
    assert_eq!(selector.to_specified_css().unwrap(), "& :scope");
    assert_eq!(selector, &before);
}
