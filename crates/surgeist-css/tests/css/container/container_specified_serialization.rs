//! Independent expectations: selected Conditional5 2025-10-30 §§5.1–5.3
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-type
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-name
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-shorthand
//! CSSOM identifier escaping and the specified graph's semantic-node policy.
//! New specified-text APIs are tested alongside their implementation.
use crate::css::support::authored_syntax::container_names as names;
use crate::css::support::serialization_limits::assert_node_limits as assert_limits;
use CssSerializationLimits as Limits;
use CssSpecifiedValueSerializationErrorKind as Kind;
use surgeist_css::*;

#[test]
fn six_type_states_preserve_semantics_and_use_grammar_order() {
    for (value, expected, authored, nodes) in [
        (CssContainerType::Normal, "normal", "NORMAL", 1),
        (CssContainerType::Size, "size", "SIZE", 1),
        (
            CssContainerType::InlineSize,
            "inline-size",
            "INLINE-SIZE",
            1,
        ),
        (
            CssContainerType::ScrollState,
            "scroll-state",
            "SCROLL-STATE",
            1,
        ),
        (
            CssContainerType::SizeScrollState,
            "size scroll-state",
            "SCROLL-STATE SIZE",
            2,
        ),
        (
            CssContainerType::InlineSizeScrollState,
            "inline-size scroll-state",
            "SCROLL-STATE INLINE-SIZE",
            2,
        ),
    ] {
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_limits(expected, nodes, |limits| value.serialize_specified(limits));
        let report = parse_declaration(&format!("container-type:{authored}!important"));
        assert!(report.is_clean());
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::ContainerType(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("container-type");
        };
        assert_eq!(*wrapper.container_type(), value);
        assert_eq!(
            wrapper
                .container_type()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
        assert_eq!(wrapper.as_css(), authored);
        assert_eq!(declaration.importance(), CssImportance::Important);
    }
}

#[test]
fn ordered_names_keep_case_duplicates_unicode_and_identifier_boundaries() {
    for (value, expected, nodes) in [
        (CssContainerNames::None, "none", 1),
        (
            names(&["Pane", "Pane", "pane", "café"]),
            "Pane Pane pane café",
            5,
        ),
        (names(&["1", "pane"]), r"\31  pane", 3),
        (names(&["1pane", "a b", "a,b"]), r"\31 pane a\ b a\,b", 4),
    ] {
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_limits(expected, nodes, |limits| value.serialize_specified(limits));
        let report = parse_declaration(&format!("container-name:{expected}"));
        assert!(report.is_clean());
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::ContainerName(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("container-name");
        };
        assert_eq!(wrapper.names(), &value);
        assert_eq!(
            wrapper
                .names()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}

#[test]
fn shorthand_omits_normal_and_counts_the_retained_type() {
    for (kind, expected, nodes) in [
        (CssContainerType::Normal, "Pane pane", 5),
        (CssContainerType::Size, "Pane pane / size", 5),
        (CssContainerType::InlineSize, "Pane pane / inline-size", 5),
        (CssContainerType::ScrollState, "Pane pane / scroll-state", 5),
        (
            CssContainerType::SizeScrollState,
            "Pane pane / size scroll-state",
            6,
        ),
        (
            CssContainerType::InlineSizeScrollState,
            "Pane pane / inline-size scroll-state",
            6,
        ),
    ] {
        let value = CssContainer::new(names(&["Pane", "pane"]), kind);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_limits(expected, nodes, |limits| value.serialize_specified(limits));
        let report = parse_declaration(&format!("container:{expected}"));
        assert!(report.is_clean());
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::Container(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("container");
        };
        assert_eq!(wrapper.container(), &value);
        assert_eq!(
            wrapper
                .container()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
    let none = CssContainer::new(CssContainerNames::None, CssContainerType::Normal);
    assert_limits("none", 3, |limits| none.serialize_specified(limits));
}

#[test]
fn parsed_reordered_shorthand_keeps_its_authored_occurrence() {
    let authored = r"\31 pane/**// SCROLL-STATE INLINE-SIZE";
    let report = parse_declaration(&format!("container:{authored}!important"));
    assert!(report.is_clean());
    let declaration = report.syntax().as_ref().unwrap();
    let CssKnownPropertyValueRef::Container(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("container");
    };
    let value = wrapper.container();
    let before = value.clone();
    let components = value.to_components(Default::default()).unwrap();
    assert_limits(r"\31 pane / inline-size scroll-state", 5, |limits| {
        value.serialize_specified(limits)
    });
    assert_eq!(value, &before);
    assert_eq!(value.to_components(Default::default()).unwrap(), components);
    assert_eq!(wrapper.as_css(), authored);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_some());
}

#[test]
fn component_token_limits_and_programmatic_origins_remain_distinct() {
    let value = names(&["Pane", "Pane", "pane"]);
    let expected = "Pane Pane pane";
    // Three names plus two spaces are five component tokens, but four semantic nodes.
    assert_limits(expected, 4, |limits| value.serialize_specified(limits));
    let limits = CssComponentValueLimits::try_new(0, 5, expected.len()).unwrap();
    let components = value.to_components(limits).unwrap();
    assert_eq!(components.items().len(), 5);
    for component in components.items() {
        assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    }
    assert_eq!(
        value
            .to_components(CssComponentValueLimits::try_new(0, 4, expected.len()).unwrap())
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ComponentLimit
    );
    assert_eq!(
        value
            .serialize_tokens(crate::css::support::byte_limits(expected.len() - 1))
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    let old_text = value
        .serialize_tokens(crate::css::support::byte_limits(expected.len()))
        .unwrap();
    assert_eq!(old_text.as_css(), expected);
    for segment in old_text.segments() {
        assert!(matches!(
            segment.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        ));
    }
}

#[test]
fn wide_names_fail_under_small_work_limits_and_retry_without_mutation() {
    let value = CssContainerNames::Names(
        CssContainerNameList::try_new(
            (0..2048)
                .map(|_| CssContainerName::try_from_decoded("pane").unwrap())
                .collect(),
        )
        .unwrap(),
    );
    let before = value.clone();
    for (limits, kind) in [
        (Limits::new(2, 2049, 10239), Kind::InputNodeLimit),
        (Limits::new(2049, 2, 10239), Kind::ProjectionNodeLimit),
        (Limits::new(2049, 2049, 4), Kind::ByteLimit),
    ] {
        assert_eq!(value.serialize_specified(limits).unwrap_err().kind(), kind);
        assert_eq!(value, before);
    }
    // 2048 four-byte names plus 2047 one-byte separators.
    let expected = std::iter::repeat_n("pane", 2048)
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(
        value
            .serialize_specified(Limits::new(2049, 2049, 10239))
            .unwrap(),
        expected
    );
}

#[test]
fn explicit_normal_and_programmatic_components_keep_the_same_shorthand_value() {
    for (value, authored, expected, nodes) in [
        (
            CssContainer::new(CssContainerNames::None, CssContainerType::Normal),
            "none / NORMAL",
            "none",
            3,
        ),
        (
            CssContainer::new(names(&["pane"]), CssContainerType::Normal),
            "pane / NORMAL",
            "pane",
            4,
        ),
    ] {
        let report = parse_declaration(&format!("container:{authored}!important"));
        assert!(report.is_clean());
        let declaration = report.syntax().as_ref().unwrap();
        let CssKnownPropertyValueRef::Container(wrapper) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("container");
        };
        assert_eq!(wrapper.container(), &value);
        assert_eq!(wrapper.as_css(), authored);
        assert_limits(expected, nodes, |limits| {
            wrapper.container().serialize_specified(limits)
        });

        let components = value.to_components(Default::default()).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Container),
            components,
            CssImportance::Important,
        )
        .unwrap();
        let CssKnownPropertyValueRef::Container(checked_value) =
            checked.known().unwrap().property_value().unwrap()
        else {
            panic!("checked container");
        };
        assert_eq!(checked_value.container(), &value);
        assert_eq!(
            checked_value
                .container()
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
        assert!(checked.position().is_none());
        assert_eq!(checked.importance(), CssImportance::Important);
    }
}
