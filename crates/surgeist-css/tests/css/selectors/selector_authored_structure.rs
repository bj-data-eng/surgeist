//! Selectors 4 pseudo-classes, pseudo-elements, combinators, attribute and
//! `:nth-*()` selectors through the public stylesheet parser and constructors.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

#[test]
fn parses_root_selector_for_custom_property_declarations() {
    let sheet = CssParser::default()
        .parse_sheet(":root { --space: 8px; }")
        .expect("clean parse report");

    assert_eq!(sheet.rules().len(), 1);
    assert_eq!(style_rule(&sheet.rules()[0]).declarations().len(), 1);
    assert_eq!(
        style_rule(&sheet.rules()[0]).declarations()[0]
            .custom()
            .map(|custom| custom.name()),
        Some(&CssCustomPropertyName::try_new("--space").unwrap())
    );
}

#[test]
fn root_selector_carries_root_pseudo_class_structurally() {
    let sheet = CssParser::default()
        .parse_sheet(":root { --space: 8px; }")
        .expect("clean parse report");

    assert_eq!(
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
        &CssSelector::PseudoClass(CssPseudoClass::Root)
    );
}

#[test]
fn compound_root_selector_carries_root_pseudo_class_structurally() {
    let sheet = CssParser::default()
        .parse_sheet("html:root { --space: 8px; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };

    assert_eq!(
        selector
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("html")
    );
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Root]);
}

#[test]
fn parses_tier_1_state_pseudo_classes_as_authored_selectors() {
    let cases = [
        (":hover { color: black; }", CssPseudoClass::Hover),
        (":active { color: black; }", CssPseudoClass::Active),
        (":focus { color: black; }", CssPseudoClass::Focus),
        (
            ":focus-visible { color: black; }",
            CssPseudoClass::FocusVisible,
        ),
        (
            ":focus-within { color: black; }",
            CssPseudoClass::FocusWithin,
        ),
        (":disabled { color: black; }", CssPseudoClass::Disabled),
        (":enabled { color: black; }", CssPseudoClass::Enabled),
        (":checked { color: black; }", CssPseudoClass::Checked),
        (":required { color: black; }", CssPseudoClass::Required),
        (":optional { color: black; }", CssPseudoClass::Optional),
        (":valid { color: black; }", CssPseudoClass::Valid),
        (":invalid { color: black; }", CssPseudoClass::Invalid),
        (
            ":placeholder-shown { color: black; }",
            CssPseudoClass::PlaceholderShown,
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .expect("clean parse report");
        assert_eq!(
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_tier_1_state_pseudo_classes() {
    let sheet = CssParser::default()
        .parse_sheet(".button:hover { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.classes(), &["button".to_owned()]);
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);
}

#[test]
fn rejects_function_syntax_for_simple_state_pseudo_classes() {
    assert_rejection(
        ":hover() { color: black; }",
        Rejection::selector(Some("hover(")),
    );
    assert_rejection(
        ":focus() { color: black; }",
        Rejection::selector(Some("focus(")),
    );
}

#[test]
fn parses_tier_2_structural_simple_pseudo_classes() {
    let cases = [
        (":first-child { color: black; }", CssPseudoClass::FirstChild),
        (":last-child { color: black; }", CssPseudoClass::LastChild),
        (":only-child { color: black; }", CssPseudoClass::OnlyChild),
        (":empty { color: black; }", CssPseudoClass::Empty),
        (
            ":first-of-type { color: black; }",
            CssPseudoClass::FirstOfType,
        ),
        (
            ":last-of-type { color: black; }",
            CssPseudoClass::LastOfType,
        ),
        (
            ":only-of-type { color: black; }",
            CssPseudoClass::OnlyOfType,
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .expect("clean parse report");
        assert_eq!(
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_structural_simple_pseudo_classes() {
    let sheet = CssParser::default()
        .parse_sheet("button:first-child { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("button")
    );
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::FirstChild]);
}

#[test]
fn rejects_function_syntax_for_non_functional_structural_pseudo_classes() {
    assert_rejection(
        ":first-child() { color: black; }",
        Rejection::selector(Some("first-child(")),
    );
    assert_rejection(
        ":empty() { color: black; }",
        Rejection::selector(Some("empty(")),
    );
}

#[test]
fn parses_requested_double_colon_pseudo_elements() {
    let cases = [
        ("::selection { color: black; }", CssPseudoElement::Selection),
        ("::before { color: black; }", CssPseudoElement::Before),
        ("::after { color: black; }", CssPseudoElement::After),
        ("::marker { color: black; }", CssPseudoElement::Marker),
        ("::backdrop { color: black; }", CssPseudoElement::Backdrop),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::Compound(selector) =
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
        else {
            panic!("expected compound selector for {css}");
        };
        assert_pseudo_elements(selector.pseudo_elements().unwrap(), &[expected]);
    }
}

#[test]
fn parses_compound_and_complex_terminal_pseudo_element_selectors() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"
            .button.primary:hover::before { color: black; }
            li[data-kind="task"]::marker { color: red; }
            .card > dialog::backdrop { color: blue; }
        "#,
        )
        .expect("clean parse report");

    let CssSelector::Compound(button) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound button selector");
    };
    assert_eq!(
        button.classes(),
        &["button".to_owned(), "primary".to_owned()]
    );
    assert_eq!(button.pseudo_classes(), &[CssPseudoClass::Hover]);
    assert_pseudo_elements(
        button.pseudo_elements().unwrap(),
        &[CssPseudoElement::Before],
    );

    let CssSelector::Compound(marker) =
        style_rule(&sheet.rules()[1]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound marker selector");
    };
    assert_eq!(
        marker
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("li")
    );
    assert_pseudo_elements(
        marker.pseudo_elements().unwrap(),
        &[CssPseudoElement::Marker],
    );

    let CssSelector::Complex(backdrop) =
        style_rule(&sheet.rules()[2]).selectors().selectors()[0].selector()
    else {
        panic!("expected complex backdrop selector");
    };
    assert_eq!(backdrop.first().classes(), &["card".to_owned()]);
    let [part] = backdrop.rest() else {
        panic!("expected one complex selector part");
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Child);
    assert_pseudo_elements(
        part.selector().pseudo_elements().unwrap(),
        &[CssPseudoElement::Backdrop],
    );
}

#[test]
fn parses_supported_generated_marker_pseudo_element_chains() {
    for css in [
        ".item::before::marker { color: black; }",
        ".item::after::marker { color: black; }",
    ] {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::Compound(selector) =
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
        else {
            panic!("expected compound selector for {css}");
        };
        assert_pseudo_elements(
            selector.pseudo_elements().unwrap(),
            if css.contains("before") {
                &[CssPseudoElement::Before, CssPseudoElement::Marker][..]
            } else {
                &[CssPseudoElement::After, CssPseudoElement::Marker][..]
            },
        );
    }
}

#[test]
fn rejects_invalid_pseudo_element_forms_and_non_terminal_positions() {
    for (css, rejection) in [
        (
            ":marker { color: black; }",
            Rejection::selector(Some("marker")),
        ),
        (
            "::unknown { color: black; }",
            Rejection::selector(Some("unknown")),
        ),
        (
            ".button::before.primary { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".button::before[data-x] { color: black; }",
            Rejection::selector(Some("[")),
        ),
        (
            ".button::before#icon { color: black; }",
            Rejection::selector(Some("#icon")),
        ),
        (
            ".button::before span { color: black; }",
            Rejection::selector(Some("span")),
        ),
        (
            ".button::before > span { color: black; }",
            Rejection::selector(Some(">")),
        ),
        (
            ".button::marker::before { color: black; }",
            Rejection::selector(Some("before")),
        ),
        (
            ".button::before::after { color: black; }",
            Rejection::selector(Some("after")),
        ),
        (
            ".button::selection::marker { color: black; }",
            Rejection::selector(Some("marker")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn part_and_user_action_pseudo_element_suffixes_are_retained() {
    for css in [
        "::part(foo) { color: black; }",
        ".button::before:hover { color: black; }",
    ] {
        let report = CssParser::default().parse_sheet(css);
        assert!(report.is_clean(), "{css}: {report:?}");
        assert_eq!(report.syntax().rules().len(), 1);
    }
}

#[test]
fn selector_list_constructor_rejects_empty_lists() {
    assert!(CssSelectorList::try_new(Vec::new(), Default::default()).is_err());
    let list = CssSelectorList::try_new(
        vec![CssSelector::Class("button".to_owned())],
        Default::default(),
    )
    .unwrap();
    assert_eq!(list.selectors(), &[CssSelector::Class("button".to_owned())]);
}

#[test]
fn pseudo_element_sequence_constructor_guards_supported_terminal_shapes() {
    let before = CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before]).unwrap();
    assert_pseudo_elements(&before, &[CssPseudoElement::Before]);

    let before_marker =
        CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before, CssPseudoElement::Marker])
            .unwrap();
    assert_pseudo_elements(
        &before_marker,
        &[CssPseudoElement::Before, CssPseudoElement::Marker],
    );

    assert_eq!(CssPseudoElementSequence::try_new(Vec::new()), None);
    assert_eq!(
        CssPseudoElementSequence::try_new(
            vec![CssPseudoElement::Marker, CssPseudoElement::Before,]
        ),
        None
    );
    assert_eq!(
        CssPseudoElementSequence::try_new(vec![CssPseudoElement::Before, CssPseudoElement::After,]),
        None
    );
}

#[test]
fn relative_selector_list_constructor_requires_selectors() {
    assert!(CssRelativeSelectorList::try_new(Vec::new(), Default::default()).is_err());
}

#[test]
fn relative_selector_preserves_combinator_and_selector() {
    let selector = CssRelativeSelector::new(
        CssSelectorCombinator::Child,
        CssSelector::Class("icon".to_owned()),
    );

    assert_eq!(selector.combinator(), CssSelectorCombinator::Child);
    assert_eq!(selector.selector(), &CssSelector::Class("icon".to_owned()));
}

#[test]
fn nth_child_pattern_preserves_optional_selector_list() {
    let list = CssPseudoSelectorList::try_new(
        vec![CssSelector::Class("important".to_owned())],
        Default::default(),
    )
    .unwrap();
    let pattern =
        CssNthChildPattern::new(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1)), Some(list));

    assert!(
        matches!(pattern.pattern(), CssNthPattern::AnPlusB(value) if value.a() == 2 && value.b() == 1)
    );
    assert_eq!(
        pattern.selector_list().unwrap().items(),
        &[CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "important".to_owned()
        ))]
    );
}

#[test]
fn nth_pattern_model_exposes_an_plus_b_coefficients() {
    let pattern = CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1));
    let CssNthPattern::AnPlusB(value) = pattern else {
        panic!("expected an+b pattern");
    };
    assert_eq!(value.a(), 2);
    assert_eq!(value.b(), 1);
}

#[test]
fn parses_nth_child_patterns() {
    let cases = [
        (":nth-child(odd) { color: black; }", CssNthPattern::Odd),
        (":nth-child(even) { color: black; }", CssNthPattern::Even),
        (":nth-child(3) { color: black; }", CssNthPattern::Integer(3)),
        (
            ":nth-child(-1) { color: black; }",
            CssNthPattern::Integer(-1),
        ),
        (
            ":nth-child(+3) { color: black; }",
            CssNthPattern::Integer(3),
        ),
        (
            ":nth-child(n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(1, 0)),
        ),
        (
            ":nth-child(-n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(-1, 0)),
        ),
        (
            ":nth-child(+n) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(1, 0)),
        ),
        (
            ":nth-child(2n+1) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 1)),
        ),
        (
            ":nth-child(2n-1) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, -1)),
        ),
        (
            ":nth-child(-n+3) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(-1, 3)),
        ),
        (
            ":nth-child(+3n-2) { color: black; }",
            CssNthPattern::AnPlusB(CssNthAnPlusB::new(3, -2)),
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .unwrap_or_else(|error| panic!("{css}: {error:?}"));
        let CssSelector::PseudoClass(CssPseudoClass::NthChild(pattern)) =
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
        else {
            panic!("expected nth-child selector");
        };
        assert_eq!(pattern.pattern(), expected);
        assert!(pattern.selector_list().is_none());
    }
}

#[test]
fn parses_all_nth_structural_pseudo_classes() {
    let cases = [
        (
            ":nth-child(2n) { color: black; }",
            CssPseudoClass::NthChild(CssNthChildPattern::new(
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0)),
                None,
            )),
        ),
        (
            ":nth-last-child(2n) { color: black; }",
            CssPseudoClass::NthLastChild(CssNthChildPattern::new(
                CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0)),
                None,
            )),
        ),
        (
            ":nth-of-type(2n) { color: black; }",
            CssPseudoClass::NthOfType(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0))),
        ),
        (
            ":nth-last-of-type(2n) { color: black; }",
            CssPseudoClass::NthLastOfType(CssNthPattern::AnPlusB(CssNthAnPlusB::new(2, 0))),
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .expect("clean parse report");
        assert_eq!(
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
            &CssSelector::PseudoClass(expected)
        );
        if let CssSelector::PseudoClass(
            CssPseudoClass::NthChild(pattern) | CssPseudoClass::NthLastChild(pattern),
        ) = style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
        {
            assert!(pattern.selector_list().is_none());
        }
    }
}

#[test]
fn nth_child_accepts_strict_of_selector_lists() {
    let sheet = CssParser::default()
        .parse_sheet("li:nth-child(2n+1 of li.important, .row[hidden]) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::NthChild(pattern)] = selector.pseudo_classes() else {
        panic!("expected nth-child pseudo-class");
    };
    assert!(
        matches!(pattern.pattern(), CssNthPattern::AnPlusB(value) if value.a() == 2 && value.b() == 1)
    );
    let selector_list = pattern.selector_list().expect("expected of selector list");
    assert_eq!(selector_list.items().len(), 2);
    assert!(matches!(
        selector_list.items()[0],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));
    assert!(matches!(
        selector_list.items()[1],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));

    let sheet = CssParser::default()
        .parse_sheet(".item:nth-last-child(even of .selected ~ .tail) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::NthLastChild(pattern)] = selector.pseudo_classes() else {
        panic!("expected nth-last-child pseudo-class");
    };
    assert_eq!(pattern.pattern(), CssNthPattern::Even);
    assert!(matches!(
        pattern.selector_list().unwrap().items()[0],
        CssPseudoSelectorListItem::Selector(CssSelector::Complex(_))
    ));
}

#[test]
fn nth_child_of_selector_lists_reject_invalid_entries_strictly() {
    assert_rejection(
        ":nth-child(odd of) { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ":nth-child(odd of .valid, .bad..selector) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ":nth-child(odd of ::before) { color: black; }",
        Rejection::selector(Some("before")),
    );
    assert_rejection(
        ":nth-of-type(odd of .item) { color: black; }",
        Rejection::selector(Some("of")),
    );
    assert_rejection(
        ":nth-last-of-type(even of .item) { color: black; }",
        Rejection::selector(Some("of")),
    );
}

#[test]
fn rejects_unsupported_nth_patterns_and_of_selector_forms() {
    assert_rejection(
        ":nth-child() { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ":nth-child(foo) { color: black; }",
        Rejection::selector(Some("foo")),
    );
    assert_rejection(
        ":nth-child(2n +) { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ":nth-child(2n + +1) { color: black; }",
        Rejection::selector(Some("+1")),
    );
    assert_rejection(
        ":nth-of-type(2n of .item) { color: black; }",
        Rejection::selector(Some("of")),
    );
}

#[test]
fn rejects_trailing_tokens_in_nth_functions() {
    assert_rejection(
        ":nth-child(odd even) { color: black; }",
        Rejection::selector(Some("even")),
    );
    assert_rejection(
        ":nth-child(1 2) { color: black; }",
        Rejection::selector(Some("2")),
    );
    assert_rejection(
        ":nth-child(2n+1 extra) { color: black; }",
        Rejection::selector(Some("extra")),
    );
}

#[test]
fn nth_pseudo_class_arguments_are_publicly_inspectable() {
    let sheet = CssParser::default()
        .parse_sheet(":nth-child(2n+1) { color: black; }")
        .expect("clean parse report");
    let CssSelector::PseudoClass(CssPseudoClass::NthChild(pattern)) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected nth-child an+b selector");
    };
    let CssNthPattern::AnPlusB(value) = pattern.pattern() else {
        panic!("expected nth-child an+b pattern");
    };
    assert_eq!(value.a(), 2);
    assert_eq!(value.b(), 1);
    assert!(pattern.selector_list().is_none());
}

#[test]
fn parses_selector_list_functional_pseudo_classes() {
    let sheet = CssParser::default()
        .parse_sheet(".button:not(.disabled, .loading) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.items(),
        &[
            CssPseudoSelectorListItem::Selector(CssSelector::Class("disabled".to_owned())),
            CssPseudoSelectorListItem::Selector(CssSelector::Class("loading".to_owned())),
        ]
    );

    let sheet = CssParser::default()
        .parse_sheet(":is(.primary, .secondary) { color: black; }")
        .expect("clean parse report");
    assert!(matches!(
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
        CssSelector::PseudoClass(CssPseudoClass::Is(_))
    ));

    let sheet = CssParser::default()
        .parse_sheet(":where(button, .link) { color: black; }")
        .expect("clean parse report");
    assert!(matches!(
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
        CssSelector::PseudoClass(CssPseudoClass::Where(_))
    ));

    let sheet = CssParser::default()
        .parse_sheet(".field:has(.error) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert!(matches!(
        selector.pseudo_classes(),
        [CssPseudoClass::Has(_)]
    ));
}

#[test]
fn parses_compound_selector_list_functional_pseudo_classes() {
    let sheet = CssParser::default()
        .parse_sheet(".field:not(:disabled, :focus) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.items(),
        &[
            CssPseudoSelectorListItem::Selector(CssSelector::PseudoClass(CssPseudoClass::Disabled)),
            CssPseudoSelectorListItem::Selector(CssSelector::PseudoClass(CssPseudoClass::Focus)),
        ]
    );
}

#[test]
fn functional_selector_lists_accept_supported_complex_selectors() {
    let sheet = CssParser::default().parse_sheet(".scope:is(.card > .title, button.primary:hover, [data-state=\"open\"].active) { color: black; }")
    .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Is(list)] = selector.pseudo_classes() else {
        panic!("expected :is selector list");
    };
    assert_eq!(list.items().len(), 3);
    assert!(matches!(
        list.items()[0],
        CssPseudoSelectorListItem::Selector(CssSelector::Complex(_))
    ));
    assert!(matches!(
        list.items()[1],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));
    assert!(matches!(
        list.items()[2],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));

    let sheet = CssParser::default()
        .parse_sheet(":not(.field .icon, button.primary:hover) { color: black; }")
        .expect("clean parse report");
    let CssSelector::PseudoClass(CssPseudoClass::Not(list)) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected :not selector list");
    };
    assert_eq!(list.items().len(), 2);
    assert!(matches!(
        list.items()[0],
        CssPseudoSelectorListItem::Selector(CssSelector::Complex(_))
    ));
    assert!(matches!(
        list.items()[1],
        CssPseudoSelectorListItem::Selector(CssSelector::Compound(_))
    ));

    let sheet = CssParser::default()
        .parse_sheet(":where(.toolbar + .panel, .stack ~ .item) { color: black; }")
        .expect("clean parse report");
    let CssSelector::PseudoClass(CssPseudoClass::Where(list)) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected :where selector list");
    };
    assert_eq!(list.items().len(), 2);
    assert!(list.items().iter().all(|selector| matches!(
        selector,
        CssPseudoSelectorListItem::Selector(CssSelector::Complex(_))
    )));
}

#[test]
fn functional_selector_lists_reject_invalid_entries_strictly() {
    assert_rejection(
        ":is(.valid, .bad..selector) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ":where(.valid, .col ||) { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ":not(.valid, ::before) { color: black; }",
        Rejection::selector(Some("before")),
    );
    // Selectors 4 "parse a list" splits with Syntax 3 (2026 CRD) §5.4.10, so a
    // final comma followed by no token adds no failing member; whitespace does.
    assert!(
        CssParser::default()
            .parse_sheet(":is(.valid,) { color: black; }")
            .is_clean()
    );
    assert_rejection(
        ":is(.valid, ) { color: black; }",
        Rejection::selector(Some(")")),
    );
}

#[test]
fn functional_pseudo_class_selector_arguments_reject_pseudo_elements() {
    for (css, rejection) in [
        (
            ":is(::before) { color: black; }",
            Rejection::selector(Some("before")),
        ),
        (
            ":where(.x::after) { color: black; }",
            Rejection::selector(Some("after")),
        ),
        (
            ":not(::marker) { color: black; }",
            Rejection::selector(Some("marker")),
        ),
        (
            ":has(::backdrop) { color: black; }",
            Rejection::selector(Some("backdrop")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn selector_argument_surface_accepts_full_supported_strict_forms() {
    for css in [
        ":not(.field .icon, button.primary:hover) { color: black; }",
        ":is(.card > .title, button.primary:hover, [data-state=\"open\"].active) { color: black; }",
        ":where(.toolbar + .panel, .stack ~ .item) { color: black; }",
        ".card:has(.field > .icon) { color: black; }",
        ".card:has(> .icon, + .error, ~ .warning) { color: black; }",
        "li:nth-child(2n+1 of li.important, .row[hidden]) { color: black; }",
        "li:nth-last-child(even of .item.selected) { color: black; }",
    ] {
        CssParser::default()
            .parse_sheet(css)
            .unwrap_or_else(|error| panic!("{css} should parse: {error:?}"));
    }
}

#[test]
fn selector_argument_surface_rejects_invalid_entries_without_recovery() {
    for (css, rejection) in [
        (
            ":is(.valid, .bad..selector) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ":where(.valid, .col ||) { color: black; }",
            Rejection::selector(Some(")")),
        ),
        (
            ":not(.valid, ::before) { color: black; }",
            Rejection::selector(Some("before")),
        ),
        (
            ".card:has() { color: black; }",
            Rejection::selector(Some(")")),
        ),
        (
            ".card:has(:has(.nested)) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".card:has(:is(:has(.nested))) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".card:has(:nth-child(odd of :has(.nested))) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".card:has(::before) { color: black; }",
            Rejection::selector(Some("before")),
        ),
        (
            ".card:has(.valid, .bad..selector) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ":nth-child(odd of) { color: black; }",
            Rejection::selector(Some(")")),
        ),
        (
            ":nth-child(odd of .valid, .bad..selector) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ":nth-of-type(odd of .item) { color: black; }",
            Rejection::selector(Some("of")),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}

#[test]
fn has_accepts_strict_relative_selector_lists() {
    let sheet = CssParser::default()
        .parse_sheet(".card:has(.field > .icon) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Has(list)] = selector.pseudo_classes() else {
        panic!("expected :has selector list");
    };
    assert_eq!(list.selectors().len(), 1);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Descendant
    );
    assert!(matches!(
        list.selectors()[0].selector(),
        CssSelector::Complex(_)
    ));

    let sheet = CssParser::default()
        .parse_sheet(".card:has(> .icon, + .error, ~ .warning) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Has(list)] = selector.pseudo_classes() else {
        panic!("expected :has selector list");
    };
    assert_eq!(list.selectors().len(), 3);
    assert_eq!(
        list.selectors()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        list.selectors()[1].combinator(),
        CssSelectorCombinator::NextSibling
    );
    assert_eq!(
        list.selectors()[2].combinator(),
        CssSelectorCombinator::SubsequentSibling
    );
}

#[test]
fn has_rejects_invalid_relative_selector_entries_strictly() {
    assert_rejection(
        ".card:has() { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ".card:has(.valid, .bad..selector) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ".card:has(:has(.nested)) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ".card:has(:is(:has(.nested))) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ".card:has(:nth-child(odd of :has(.nested))) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ".card:has(::before) { color: black; }",
        Rejection::selector(Some("before")),
    );
    assert_rejection(
        ".card:has(| .bad) { color: black; }",
        Rejection::selector(Some(".")),
    );
    assert_rejection(
        ".card:has(.valid,) { color: black; }",
        Rejection::selector(Some(")")),
    );
}

#[test]
fn functional_pseudo_class_arguments_are_publicly_inspectable() {
    let sheet = CssParser::default()
        .parse_sheet(".button:not(.disabled) { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    let [CssPseudoClass::Not(list)] = selector.pseudo_classes() else {
        panic!("expected :not selector list");
    };
    assert_eq!(
        list.items(),
        &[CssPseudoSelectorListItem::Selector(CssSelector::Class(
            "disabled".to_owned()
        ))]
    );
}

#[test]
fn rejects_empty_selector_list_functional_pseudo_classes() {
    assert_rejection(":not() { color: black; }", Rejection::selector(Some(")")));
    assert_rejection(":is() { color: black; }", Rejection::selector(Some(")")));
    assert_rejection(":where() { color: black; }", Rejection::selector(Some(")")));
    assert_rejection(":has() { color: black; }", Rejection::selector(Some(")")));
}

#[test]
fn parses_tier_4_runtime_state_pseudo_classes() {
    let cases = [
        (":modal { color: black; }", CssPseudoClass::Modal),
        (":fullscreen { color: black; }", CssPseudoClass::Fullscreen),
        (
            ":popover-open { color: black; }",
            CssPseudoClass::PopoverOpen,
        ),
        (":default { color: black; }", CssPseudoClass::Default),
        (
            ":indeterminate { color: black; }",
            CssPseudoClass::Indeterminate,
        ),
        (":read-only { color: black; }", CssPseudoClass::ReadOnly),
        (":read-write { color: black; }", CssPseudoClass::ReadWrite),
        (":in-range { color: black; }", CssPseudoClass::InRange),
        (
            ":out-of-range { color: black; }",
            CssPseudoClass::OutOfRange,
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .expect("clean parse report");
        assert_eq!(
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
            &CssSelector::PseudoClass(expected)
        );
    }
}

#[test]
fn parses_compound_runtime_state_pseudo_classes() {
    let sheet = CssParser::default()
        .parse_sheet(".dialog:modal:fullscreen { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(selector.classes(), &["dialog".to_owned()]);
    assert_eq!(
        selector.pseudo_classes(),
        &[CssPseudoClass::Modal, CssPseudoClass::Fullscreen]
    );
}

#[test]
fn rejects_function_syntax_for_runtime_state_pseudo_classes() {
    assert_rejection(
        ":modal() { color: black; }",
        Rejection::selector(Some("modal(")),
    );
    assert_rejection(
        ":fullscreen() { color: black; }",
        Rejection::selector(Some("fullscreen(")),
    );
    assert_rejection(
        ":read-only() { color: black; }",
        Rejection::selector(Some("read-only(")),
    );
}

#[test]
fn rejects_invalid_relative_or_combinator_selector_forms() {
    assert_rejection(
        ".field:has(.col ||) { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ".field:has(||) { color: black; }",
        Rejection::selector(Some(")")),
    );
    assert_rejection(
        ".field:has(::before) { color: black; }",
        Rejection::selector(Some("before")),
    );
    assert_rejection(
        ".field:has(| .icon) { color: black; }",
        Rejection::selector(Some(".")),
    );
}

#[test]
fn parses_combinator_selectors() {
    for css in [
        ".stack .item { color: black; }",
        ".toolbar > button { color: black; }",
        "label + input { color: black; }",
        "h2 ~ p { color: black; }",
        ".card[data-state=open] > .title:hover { color: black; }",
    ] {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .unwrap_or_else(|error| panic!("{css}: {error:?}"));
        assert!(matches!(
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector(),
            CssSelector::Complex(_)
        ));
    }
}

#[test]
fn combinator_selectors_are_structurally_inspectable() {
    let sheet = CssParser::default()
        .parse_sheet(".toolbar > button { color: black; }")
        .expect("clean parse report");
    let rule = style_rule(&sheet.rules()[0]);
    let CssSelector::Complex(selector) = rule.selectors().selectors()[0].selector() else {
        panic!("expected complex selector");
    };
    assert_eq!(
        selector.rest()[0].combinator(),
        CssSelectorCombinator::Child
    );
    assert_eq!(
        selector.rest()[0]
            .selector()
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("button")
    );
}

#[test]
fn rejects_invalid_combinator_selectors() {
    assert_rejection("> .item { color: black; }", Rejection::selector(Some(">")));
    assert_rejection(
        ".a > > .b { color: black; }",
        Rejection::selector(Some(">")),
    );
    assert_rejection(".a > { color: black; }", Rejection::selector(Some(">")));
    assert_rejection(".col || { color: black; }", Rejection::selector(Some("|")));
    assert_rejection(
        ".field:has(> > .icon) { color: black; }",
        Rejection::selector(Some(">")),
    );
    assert_rejection(
        ".field:has(.field > > .icon) { color: black; }",
        Rejection::selector(Some(">")),
    );
}

#[test]
fn parses_attribute_selector_matcher_forms() {
    let cases = [
        ("[disabled] { color: black; }", CssAttributeMatcher::Exists),
        (
            "[data-state=open] { color: black; }",
            CssAttributeMatcher::Equals("open".to_owned()),
        ),
        (
            r#"[data-role~="button"] { color: black; }"#,
            CssAttributeMatcher::Includes("button".to_owned()),
        ),
        (
            "[lang|=en] { color: black; }",
            CssAttributeMatcher::DashMatch("en".to_owned()),
        ),
        (
            r#"[href^="https"] { color: black; }"#,
            CssAttributeMatcher::Prefix("https".to_owned()),
        ),
        (
            r#"[src$=".svg"] { color: black; }"#,
            CssAttributeMatcher::Suffix(".svg".to_owned()),
        ),
        (
            r#"[data-id*="card"] { color: black; }"#,
            CssAttributeMatcher::Substring("card".to_owned()),
        ),
    ];

    for (css, expected) in cases {
        let sheet = CssParser::default()
            .parse_sheet(css)
            .expect("clean parse report");
        let CssSelector::Compound(selector) =
            style_rule(&sheet.rules()[0]).selectors().selectors()[0].selector()
        else {
            panic!("{css} should parse as a compound selector");
        };
        let [attribute] = selector.attributes() else {
            panic!("{css} should have one attribute selector");
        };
        assert_eq!(attribute.matcher(), &expected);
        assert_eq!(
            attribute.case_sensitivity(),
            CssAttributeCaseSensitivity::DocumentDefault
        );
    }
}

#[test]
fn attribute_selectors_are_structurally_inspectable() {
    let sheet = CssParser::default()
        .parse_sheet(r#"[data-state="open" i] { color: black; }"#)
        .expect("clean parse report");
    let rule = style_rule(&sheet.rules()[0]);
    let CssSelector::Compound(selector) = rule.selectors().selectors()[0].selector() else {
        panic!("expected compound selector");
    };
    let [attribute] = selector.attributes() else {
        panic!("expected one attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "data-state");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("open".to_owned())
    );
    assert_eq!(
        attribute.case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );
}

#[test]
fn parses_attribute_selector_case_modifiers_and_compound_position() {
    let insensitive = CssParser::default()
        .parse_sheet(r#"[data-state="OPEN" i] { color: black; }"#)
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&insensitive.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector.attributes()[0].case_sensitivity(),
        CssAttributeCaseSensitivity::AsciiCaseInsensitive
    );

    let sensitive = CssParser::default()
        .parse_sheet(r#"[data-state="open" s] { color: black; }"#)
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&sensitive.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector.attributes()[0].case_sensitivity(),
        CssAttributeCaseSensitivity::ExplicitSensitive
    );

    let mixed = CssParser::default()
        .parse_sheet("button.primary[aria-expanded=true]:hover { color: black; }")
        .expect("clean parse report");
    let CssSelector::Compound(selector) =
        style_rule(&mixed.rules()[0]).selectors().selectors()[0].selector()
    else {
        panic!("expected compound selector");
    };
    assert_eq!(
        selector
            .type_selector()
            .and_then(CssQualifiedSelectorName::local_name),
        Some("button")
    );
    assert_eq!(selector.classes(), &["primary".to_owned()]);
    assert_eq!(selector.pseudo_classes(), &[CssPseudoClass::Hover]);
    let [attribute] = selector.attributes() else {
        panic!("expected one attribute selector");
    };
    assert_eq!(attribute.name().as_str(), "aria-expanded");
    assert_eq!(
        attribute.matcher(),
        &CssAttributeMatcher::Equals("true".to_owned())
    );
}

#[test]
fn rejects_invalid_attribute_selectors() {
    assert_rejection(
        "[svg|href] { color: black; }",
        Rejection::selector(Some("svg")),
    );
    assert_rejection(
        "[data-state=] { color: black; }",
        Rejection::selector(Some("]")),
    );
    assert_rejection(
        "[data-state=open q] { color: black; }",
        Rejection::selector(Some("q")),
    );
    assert_rejection("[] { color: black; }", Rejection::selector(Some("]")));
    assert_rejection(
        "[data-state=open extra] { color: black; }",
        Rejection::selector(Some("extra")),
    );
}

#[test]
fn attribute_name_constructor_matches_parser_identifier_invariants() {
    assert_eq!(
        CssAttributeName::try_new("data-state").unwrap().as_str(),
        "data-state"
    );
    assert_eq!(CssAttributeName::try_new(""), None);
    assert_eq!(CssAttributeName::try_new("a\0b"), None);
    for (decoded, spelling) in [
        (" \t\n ", r"\20 \9 \a \20 "),
        ("data state", r"data\ state"),
        ("svg|href", r"svg\|href"),
        ("data-state extra", r"data-state\ extra"),
        ("data-state;", r"data-state\;"),
    ] {
        // These are decoded IDENT values, not unescaped selector spelling.
        let components = CssComponentValues::parse(&CssParser::default(), spelling).unwrap();
        let [identifier] = components.items() else {
            panic!("one independently supplied identifier spelling: {spelling}");
        };
        assert!(matches!(
            identifier.view(),
            CssComponentValueRef::Token(CssValueTokenRef::Ident(value)) if value == decoded
        ));
        assert_eq!(
            CssAttributeName::try_new(decoded).unwrap().as_str(),
            decoded
        );

        let source = format!("[{spelling}]");
        let parsed = CssSelector::parse(
            &CssParser::default(),
            &source,
            &CssNamespaceContext::default(),
        );
        assert!(parsed.is_clean(), "{source}: {parsed:?}");
        let Some(CssSelector::Compound(compound)) = parsed.syntax() else {
            panic!("one attribute selector: {source}");
        };
        let [attribute] = compound.attributes() else {
            panic!("one decoded attribute name: {source}");
        };
        assert_eq!(attribute.name().as_str(), decoded);
        assert_eq!(attribute.namespace(), &CssNamespaceConstraint::ExplicitNone);

        let unescaped = CssSelector::parse(
            &CssParser::default(),
            format!("[{decoded}]"),
            &CssNamespaceContext::default(),
        );
        assert!(!unescaped.is_clean());
        assert!(unescaped.syntax().is_none());
    }
}

#[test]
fn practical_pseudo_class_matrix_accepts_supported_and_rejects_unsupported_forms() {
    let accepted = [
        ":host { color: black; }",
        ":hover { color: black; }",
        ":focus-visible { color: black; }",
        ":disabled { color: black; }",
        ":first-child { color: black; }",
        ":nth-child(2n+1) { color: black; }",
        ":nth-child(2n of .item) { color: black; }",
        ":not(.disabled) { color: black; }",
        ".field:not(.field .icon) { color: black; }",
        ":is(.primary, .secondary) { color: black; }",
        ":where(button, .link) { color: black; }",
        ".field:has(.error) { color: black; }",
        ".field:has(> .icon) { color: black; }",
        ".field:has(.field > .icon) { color: black; }",
        ":modal { color: black; }",
        ":read-only { color: black; }",
        ":visited { color: black; }",
        ":target { color: black; }",
        ":lang(en) { color: black; }",
    ];

    for css in accepted {
        assert!(
            CssParser::default().parse_sheet(css).is_clean(),
            "{css} should parse"
        );
    }

    let rejected = [
        (
            ":state(open) { color: black; }",
            Rejection::selector(Some("state(")),
        ),
        (
            ":hover() { color: black; }",
            Rejection::selector(Some("hover(")),
        ),
        (":not() { color: black; }", Rejection::selector(Some(")"))),
        (
            ":nth-of-type(2n of .item) { color: black; }",
            Rejection::selector(Some("of")),
        ),
        (
            ".field:has(.col ||) { color: black; }",
            Rejection::selector(Some(")")),
        ),
        (
            ".field:has(:has(.nested)) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".field:has(.valid, .bad..selector) { color: black; }",
            Rejection::selector(Some(".")),
        ),
        (
            ".field:not(::before) { color: black; }",
            Rejection::selector(Some("before")),
        ),
    ];

    for (css, rejection) in rejected {
        assert_rejection(css, rejection);
    }
}

#[test]
fn selector_parse_failure_has_typed_error_kind() {
    let error = CssParser::default()
        .parse_sheet("??? { width: 10px; }")
        .expect_err("rejected parse report");

    assert!(error.code() == CssErrorCode::InvalidSelector);
}

#[test]
fn selector_missing_class_name_has_typed_error_kind() {
    let error = CssParser::default()
        .parse_sheet(". { width: 10px; }")
        .expect_err("rejected parse report");

    assert!(error.code() == CssErrorCode::InvalidSelector);
}
