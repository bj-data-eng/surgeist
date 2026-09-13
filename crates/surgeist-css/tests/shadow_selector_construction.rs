#![forbid(unsafe_code)]
//! Checked authored shadow syntax; expectations follow the pinned sources cited
//! in shadow_selector_grammar.rs. These are new API tests without stub RED.
use surgeist_css::*;

fn selector(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().unwrap()
}

fn argument(source: &str) -> CssCompoundSelectorArgument {
    CssCompoundSelectorArgument::try_new(selector(source)).unwrap()
}

fn part(names: &[&str]) -> CssPseudoElement {
    CssPseudoElement::Part(
        CssPartNameList::try_new(
            names
                .iter()
                .map(|name| CssPartName::try_new(*name).unwrap())
                .collect(),
        )
        .unwrap(),
    )
}

fn pseudo_list(pseudo: CssPseudoClass) -> CssPseudoSelectorList {
    CssPseudoSelectorList::try_new(vec![CssSelector::PseudoClass(pseudo)]).unwrap()
}

fn has(selector: CssSelector) -> CssPseudoClass {
    CssPseudoClass::Has(
        CssRelativeSelectorList::try_new(vec![CssRelativeSelector::new(
            CssSelectorCombinator::Child,
            selector,
        )])
        .unwrap(),
    )
}

#[test]
fn compound_construction_preserves_primitive_and_namespace_arguments() {
    let constructed =
        CssCompoundSelectorArgument::try_new(CssSelector::Class("button".into())).unwrap();
    assert_eq!(constructed.compound().classes(), &["button"]);
    assert!(constructed.compound().type_selector().is_none());
    for (source, universal, namespace) in [
        ("*", true, CssNamespaceConstraint::Any),
        ("|button", false, CssNamespaceConstraint::ExplicitNone),
        ("*|button", false, CssNamespaceConstraint::Any),
    ] {
        let argument = argument(source);
        let name = argument.compound().type_selector().unwrap();
        assert_eq!(name.is_universal(), universal);
        assert_eq!(name.namespace(), &namespace);
        assert_eq!(name.local_name(), (!universal).then_some("button"));
    }
    for invalid in [
        CssSelector::Class(String::new()),
        CssSelector::Tag("a\0b".into()),
        selector(".a > .b"),
        selector("::before"),
    ] {
        assert!(CssCompoundSelectorArgument::try_new(invalid).is_none());
    }
}

#[test]
fn checked_arguments_reject_logical_complex_selectors_and_pseudo_elements() {
    for invalid in [
        CssPseudoClass::Not(CssPseudoSelectorList::try_new(vec![selector(".a > .b")]).unwrap()),
        CssPseudoClass::Is(CssPseudoSelectorList::try_new(vec![selector(".a > .b")]).unwrap()),
        CssPseudoClass::Where(CssPseudoSelectorList::try_new(vec![selector("::before")]).unwrap()),
        CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(CssPseudoSelectorList::try_new(vec![selector("::before")]).unwrap()),
        )),
    ] {
        assert!(CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(invalid)).is_none());
    }
    let valid = has(selector(".parent > .child"));
    assert!(CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(valid)).is_some());
}

#[test]
fn checked_arguments_reject_has_nested_through_host_and_logical_functions() {
    let inner = has(CssSelector::Class("child".into()));
    let host_argument =
        CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(inner.clone())).unwrap();
    for hidden in [
        CssPseudoClass::HostFunction(host_argument.clone()),
        CssPseudoClass::HostContext(host_argument),
        CssPseudoClass::Not(pseudo_list(inner.clone())),
        CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(pseudo_list(inner)),
        )),
    ] {
        let invalid = has(CssSelector::PseudoClass(hidden));
        assert!(
            CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(invalid.clone()))
                .is_none()
        );
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                CssPseudoElementSegment::PseudoElement(part(&["label"])),
                CssPseudoElementSegment::PseudoClass(invalid),
            ])
            .is_none()
        );
    }
}

#[test]
fn part_names_preserve_order_duplicates_and_programmatic_origins() {
    let list = CssPartNameList::try_new(vec![
        CssPartName::try_new("Label").unwrap(),
        CssPartName::try_new("1st").unwrap(),
        CssPartName::try_new("label:icon").unwrap(),
        CssPartName::try_new("Label").unwrap(),
        CssPartName::try_new("initial").unwrap(),
    ])
    .unwrap();
    assert_eq!(
        list.names()
            .iter()
            .map(CssPartName::as_str)
            .collect::<Vec<_>>(),
        ["Label", "1st", "label:icon", "Label", "initial"]
    );
    assert_eq!(
        list.to_css_string(),
        r"Label \31 st label\:icon Label initial"
    );
    assert!(
        list.names()
            .iter()
            .all(|name| name.origin() == &CssValueOrigin::Programmatic)
    );
    assert!(CssPartNameList::try_new(Vec::new()).is_none());
    assert!(CssPartName::try_new("").is_err());
    assert!(CssPartName::try_new("a\0b").is_err());
}

#[test]
fn parsed_part_names_keep_each_original_token_span() {
    let source = r"::part( Label /**/ \31 st label\:icon Label )";
    let CssSelector::Compound(compound) = selector(source) else {
        panic!("compound")
    };
    let [CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(list))] =
        compound.pseudo_elements().unwrap().segments()
    else {
        panic!("part")
    };
    assert_eq!(
        list.names()
            .iter()
            .map(CssPartName::as_str)
            .collect::<Vec<_>>(),
        ["Label", "1st", "label:icon", "Label"]
    );
    for (name, raw) in list
        .names()
        .iter()
        .zip(["Label", r"\31 st", r"label\:icon", "Label"])
    {
        let CssValueOrigin::Parsed(origin) = name.origin() else {
            panic!("parsed origin")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            &source[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
            raw
        );
    }
    assert!(list.names()[0].origin() != list.names()[3].origin());
}

#[test]
fn parsed_segments_preserve_which_pseudo_element_owns_each_condition() {
    let CssSelector::Compound(compound) =
        selector(".button:focus::part(label):hover::before:active")
    else {
        panic!("compound")
    };
    assert_eq!(compound.pseudo_classes(), &[CssPseudoClass::Focus]);
    let [
        CssPseudoElementSegment::PseudoElement(CssPseudoElement::Part(names)),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Hover),
        CssPseudoElementSegment::PseudoElement(CssPseudoElement::Before),
        CssPseudoElementSegment::PseudoClass(CssPseudoClass::Active),
    ] = compound.pseudo_elements().unwrap().segments()
    else {
        panic!("complete ordered attachment")
    };
    assert_eq!(names.names()[0].as_str(), "label");
}

#[test]
fn checked_segments_validate_transitions_at_the_most_recent_element() {
    use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
    let segments = vec![
        E(part(&["label"])),
        P(CssPseudoClass::FirstChild),
        E(CssPseudoElement::Before),
        P(CssPseudoClass::Hover),
        E(CssPseudoElement::Marker),
    ];
    assert_eq!(
        CssPseudoElementSequence::try_from_segments(segments.clone())
            .unwrap()
            .segments(),
        segments
    );
    for invalid in [
        vec![],
        vec![P(CssPseudoClass::Hover)],
        vec![E(CssPseudoElement::Before), P(CssPseudoClass::FirstChild)],
        vec![
            E(part(&["label"])),
            E(CssPseudoElement::Before),
            P(CssPseudoClass::FirstChild),
        ],
        vec![E(CssPseudoElement::Before), E(CssPseudoElement::After)],
        vec![
            E(CssPseudoElement::Slotted(argument(".a"))),
            E(CssPseudoElement::FirstLine),
        ],
        vec![
            E(CssPseudoElement::Slotted(argument(".a"))),
            E(CssPseudoElement::Slotted(argument(".b"))),
        ],
    ] {
        assert!(CssPseudoElementSequence::try_from_segments(invalid).is_none());
    }
    for tree_abiding in [
        part(&["label"]),
        CssPseudoElement::Backdrop,
        CssPseudoElement::Marker,
    ] {
        assert!(
            CssPseudoElementSequence::try_new(vec![
                CssPseudoElement::Slotted(argument(".a")),
                tree_abiding
            ])
            .is_some()
        );
    }
}

#[test]
fn checked_logical_suffix_arguments_inherit_position_and_remain_pseudo_only() {
    use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
    let structural = CssPseudoClass::Not(pseudo_list(CssPseudoClass::FirstChild));
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            E(part(&["label"])),
            P(structural.clone())
        ])
        .is_some()
    );
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            E(CssPseudoElement::Before),
            P(structural)
        ])
        .is_none()
    );
    for invalid in [
        selector(".class"),
        selector(":hover > :focus"),
        selector("::before"),
    ] {
        let logical = CssPseudoClass::Is(CssPseudoSelectorList::try_new(vec![invalid]).unwrap());
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![E(part(&["label"])), P(logical)])
                .is_none()
        );
    }
    let host = CssPseudoClass::HostFunction(argument(".inside"));
    assert!(
        CssPseudoElementSequence::try_from_segments(vec![
            E(part(&["label"])),
            P(CssPseudoClass::Not(pseudo_list(host)))
        ])
        .is_some()
    );
}

#[test]
fn empty_forgiving_lists_cannot_be_reused_for_nonempty_argument_grammars() {
    let report = parse_selector(":is()", &CssNamespaceContext::default());
    let Some(CssSelector::PseudoClass(CssPseudoClass::Is(empty))) = report.syntax().clone() else {
        panic!("retained empty forgiving list")
    };
    assert!(empty.selectors().is_empty());
    for invalid in [
        CssPseudoClass::Not(empty.clone()),
        CssPseudoClass::NthChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(empty.clone()),
        )),
        CssPseudoClass::NthLastChild(CssNthChildPattern::new(
            CssNthPattern::Odd,
            Some(empty.clone()),
        )),
    ] {
        assert!(
            CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(invalid.clone()))
                .is_none()
        );
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                CssPseudoElementSegment::PseudoElement(part(&["label"])),
                CssPseudoElementSegment::PseudoClass(invalid),
            ])
            .is_none()
        );
    }
    for valid in [
        CssPseudoClass::Is(empty.clone()),
        CssPseudoClass::Where(empty),
    ] {
        assert!(
            CssCompoundSelectorArgument::try_new(CssSelector::PseudoClass(valid.clone())).is_some()
        );
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                CssPseudoElementSegment::PseudoElement(CssPseudoElement::Before),
                CssPseudoElementSegment::PseudoClass(valid),
            ])
            .is_some()
        );
    }
}
