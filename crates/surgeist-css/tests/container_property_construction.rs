#![forbid(unsafe_code)]
//! Independent ordinary values and construction contracts for Conditional5 §5.1–5.3.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/
use surgeist_css::*;

fn names(values: &[&str]) -> CssContainerNames {
    CssContainerNames::Names(
        CssContainerNameList::try_new(
            values
                .iter()
                .map(|name| CssContainerName::try_from_decoded(*name).unwrap())
                .collect(),
        )
        .unwrap(),
    )
}
fn declaration(property: CssKnownProperty, components: CssComponentValues) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components,
        CssImportance::Important,
    )
    .unwrap()
}
fn assert_programmatic(components: &CssComponentValues) {
    assert!(
        components
            .items()
            .iter()
            .all(|value| value.origin() == &CssValueOrigin::Programmatic)
    );
}

#[test]
fn six_type_states_have_exact_canonical_tokens_and_checked_declaration_values() {
    for (value, text, keywords) in [
        (CssContainerType::Normal, "normal", &["normal"][..]),
        (CssContainerType::Size, "size", &["size"][..]),
        (
            CssContainerType::InlineSize,
            "inline-size",
            &["inline-size"][..],
        ),
        (
            CssContainerType::ScrollState,
            "scroll-state",
            &["scroll-state"][..],
        ),
        (
            CssContainerType::SizeScrollState,
            "size scroll-state",
            &["size", "scroll-state"][..],
        ),
        (
            CssContainerType::InlineSizeScrollState,
            "inline-size scroll-state",
            &["inline-size", "scroll-state"][..],
        ),
    ] {
        let components = value.to_components().unwrap();
        assert_programmatic(&components);
        let actual: Vec<_> = components
            .items()
            .iter()
            .filter_map(|item| match item.view() {
                CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => Some(name),
                _ => None,
            })
            .collect();
        assert_eq!(actual, keywords);
        assert_eq!(value.serialize().unwrap().as_css(), text);
        let declaration = declaration(CssKnownProperty::ContainerType, components);
        let CssKnownPropertyValueRef::ContainerType(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("type wrapper")
        };
        assert_eq!(*wrapper.container_type(), value);
        assert_eq!(wrapper.as_css(), text);
        assert!(declaration.position().is_none());
        assert!(declaration.parsed_value().is_none());
    }
}

#[test]
fn decoded_names_preserve_old_spelling_contract_and_reject_reserved_values() {
    for decoded in ["1pane", "a b", "a,b", "-", "café"] {
        let name = CssContainerName::try_from_decoded(decoded).unwrap();
        assert_eq!(name.as_str(), decoded);
    }
    for escaped in ["1pane", "a b", "a,b", "-"] {
        assert!(CssContainerName::try_new(escaped).is_none());
    }
    assert!(CssContainerName::try_new("pane").is_some());
    for reserved in [
        "",
        "a\0b",
        "NoNe",
        "and",
        "OR",
        "not",
        "default",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert!(
            CssContainerName::try_from_decoded(reserved).is_none(),
            "{reserved:?}"
        );
    }
    assert!(CssContainerNameList::try_new(Vec::new()).is_none());
    let value = names(&["Pane", "Pane", "pane"]);
    let CssContainerNames::Names(list) = &value else {
        panic!("list")
    };
    assert_eq!(
        list.names()
            .iter()
            .map(CssContainerName::as_str)
            .collect::<Vec<_>>(),
        ["Pane", "Pane", "pane"]
    );
    assert_eq!(value.serialize().unwrap().as_css(), "Pane Pane pane");
    assert_eq!(
        CssContainerNames::None.serialize().unwrap().as_css(),
        "none"
    );
}

#[test]
fn shorthand_semantic_pair_omits_normal_and_expands_in_adopted_member_order() {
    for kind in [CssContainerType::Normal, CssContainerType::SizeScrollState] {
        let value = CssContainer::new(names(&["Pane", "Pane"]), kind);
        assert_eq!(value.names(), &names(&["Pane", "Pane"]));
        assert_eq!(value.container_type(), kind);
        let expected = if kind == CssContainerType::Normal {
            "Pane Pane"
        } else {
            "Pane Pane / size scroll-state"
        };
        assert_eq!(value.serialize().unwrap().as_css(), expected);
        let components = value.to_components().unwrap();
        assert_programmatic(&components);
        let declaration = declaration(CssKnownProperty::Container, components.clone());
        assert_eq!(declaration.value_components(), &components);
        let CssKnownPropertyValueRef::Container(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("container wrapper")
        };
        assert_eq!(wrapper.container(), &value);
        let CssExpansion::Contributions(CssContributions::Longhands(expanded)) =
            expand_declaration(&declaration).unwrap()
        else {
            panic!("expanded")
        };
        let [name, container_type] = expanded.items() else {
            panic!("two members")
        };
        assert!(
            matches!(name.ordinary_value().unwrap().view(), CssLonghandValueRef::ContainerName(v) if v == value.names())
        );
        assert!(
            matches!(container_type.ordinary_value().unwrap().view(), CssLonghandValueRef::ContainerType(v) if *v == kind)
        );
        for item in expanded.items() {
            assert!(item.source().same_occurrence(&declaration));
        }
    }
}

#[test]
fn canonical_escaping_and_byte_limits_are_checked_after_escape_expansion() {
    let hex_terminated = names(&["1", "pane"]);
    assert_eq!(hex_terminated.serialize().unwrap().as_css(), r"\31  pane");
    let constructed = declaration(
        CssKnownProperty::ContainerName,
        hex_terminated.to_components().unwrap(),
    );
    let CssKnownPropertyValueRef::ContainerName(wrapper) =
        constructed.known().unwrap().property_value().unwrap()
    else {
        panic!("names")
    };
    assert_eq!(wrapper.names(), &hex_terminated);

    let value = names(&["1pane", "a b", "a,b"]);
    let text = r"\31 pane a\ b a\,b";
    assert_eq!(value.serialize().unwrap().as_css(), text);
    let output = value.serialize_with_limit(text.len()).unwrap();
    assert_eq!(output.as_css(), text);
    let error = value.serialize_with_limit(text.len() - 1).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    let components = value
        .to_components_with_limits(CssComponentValueLimits::try_new(0, 5, text.len()).unwrap())
        .unwrap();
    assert_programmatic(&components);
    assert_eq!(components.items().len(), 5);
    assert_eq!(
        value
            .to_components_with_limits(CssComponentValueLimits::try_new(0, 4, text.len()).unwrap())
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ComponentLimit
    );
    assert_eq!(
        value
            .to_components_with_limits(
                CssComponentValueLimits::try_new(0, 5, text.len() - 1).unwrap()
            )
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    // The lower bound catches obviously oversized names before canonical escaping.
    assert_eq!(
        names(&["oversized"])
            .serialize_with_limit(1)
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    assert_eq!(
        CssContainerType::Normal
            .serialize_with_limit(0)
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    let result = declaration(CssKnownProperty::ContainerName, components);
    let CssKnownPropertyValueRef::ContainerName(wrapper) =
        result.known().unwrap().property_value().unwrap()
    else {
        panic!("name wrapper")
    };
    assert_eq!(wrapper.names(), &value);
}

#[test]
fn parsed_wrappers_keep_authored_text_components_and_origins_when_semantics_canonicalize() {
    let report = parse_declaration(r"container: P\61 ne/**/Pane / SCROLL-STATE SIZE!important");
    assert!(report.is_clean(), "{report:?}");
    let declaration = report.syntax().as_ref().unwrap();
    let original = declaration.value_components().clone();
    assert!(declaration.position().is_some());
    assert!(declaration.parsed_value().is_some());
    assert!(
        original
            .items()
            .iter()
            .all(|item| matches!(item.origin(), CssValueOrigin::Parsed(_)))
    );
    let CssKnownPropertyValueRef::Container(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("container")
    };
    assert_eq!(wrapper.as_css(), r"P\61 ne/**/Pane / SCROLL-STATE SIZE");
    assert_eq!(wrapper.container().names(), &names(&["Pane", "Pane"]));
    assert_eq!(
        wrapper.container().container_type(),
        CssContainerType::SizeScrollState
    );
    let canonical = wrapper.container().to_components().unwrap();
    assert_programmatic(&canonical);
    assert_eq!(
        wrapper.container().serialize().unwrap().as_css(),
        "Pane Pane / size scroll-state"
    );
    assert_eq!(declaration.value_components(), &original);
}

#[test]
fn parsed_none_and_omitted_type_have_checked_initial_semantics() {
    for css in ["container:none", "container:none / normal"] {
        let report = parse_declaration(css);
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::Container(value) = report
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("container")
        };
        assert_eq!(
            value.container(),
            &CssContainer::new(CssContainerNames::None, CssContainerType::Normal)
        );
        assert_eq!(value.container().serialize().unwrap().as_css(), "none");
    }
}

#[test]
fn semantic_pending_replacement_keeps_programmatic_origins_and_original_occurrence() {
    let report = parse_declaration("container:var(--layout)!important");
    let source = report.syntax().as_ref().unwrap();
    let CssExpansion::Pending(pending) = expand_declaration(source).unwrap() else {
        panic!("pending")
    };
    let semantic = CssContainer::new(names(&["1pane"]), CssContainerType::InlineSizeScrollState);
    let replacement = semantic.to_components().unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("members")
    };
    let [name, kind] = values.items() else {
        panic!("two members")
    };
    assert!(
        matches!(name.ordinary_value().unwrap().view(), CssLonghandValueRef::ContainerName(v) if v == semantic.names())
    );
    assert!(matches!(
        kind.ordinary_value().unwrap().view(),
        CssLonghandValueRef::ContainerType(CssContainerType::InlineSizeScrollState)
    ));
    for item in values.items() {
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), Some(&replacement));
        assert_programmatic(item.replacement_components().unwrap());
    }
    let serialized = semantic.serialize().unwrap();
    for segment in serialized.segments() {
        assert!(matches!(
            segment.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        ));
    }
}
