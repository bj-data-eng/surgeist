#![forbid(unsafe_code)]

//! Functional expectations from Color Adjustment 1 CR 2025-12-16 §§2.1, 3.2, 4.1–4.2
//! and the shared CSSOM identifier serialization contract. Contextual execution
//! remains outside these authored models.

use surgeist_css::*;

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    source.clone()
}

fn scheme(source: &CssDeclaration) -> &CssColorScheme {
    let CssKnownPropertyValueRef::ColorScheme(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed scheme")
    };
    value.value()
}

fn custom(value: &str) -> CssColorSchemeKeyword {
    CssColorSchemeKeyword::Custom(
        CssColorSchemeName::try_new(CssIdent::try_new(value).unwrap()).unwrap(),
    )
}

#[test]
fn checked_scheme_names_apply_only_generic_and_local_exclusions() {
    for name in [
        "normal",
        "LIGHT",
        "Dark",
        "only",
        "default",
        "inherit",
        "initial",
        "unset",
        "revert",
        "REVERT-LAYER",
    ] {
        assert_eq!(
            CssColorSchemeName::try_new(CssIdent::try_new(name).unwrap()).unwrap_err(),
            CssColorSchemeConstructionError::ReservedName,
            "{name}"
        );
    }
    for name in [
        "none",
        "span",
        "auto",
        "Future",
        "future",
        "--Theme",
        "Future Theme",
        "9dawn",
    ] {
        let checked = CssColorSchemeName::try_new(CssIdent::try_new(name).unwrap()).unwrap();
        assert_eq!(checked.as_str(), name);
    }
    assert!(CssIdent::try_new("").is_err());
    assert!(CssIdent::try_new("bad\0name").is_err());
    for only in [false, true] {
        assert_eq!(
            CssColorScheme::try_new(vec![], only).unwrap_err(),
            CssColorSchemeConstructionError::EmptySchemes
        );
    }
}

#[test]
fn checked_scheme_lists_retain_order_repetition_custom_case_and_normal_identity() {
    let entries = vec![
        CssColorSchemeKeyword::Light,
        CssColorSchemeKeyword::Light,
        custom("Future"),
        custom("future"),
        CssColorSchemeKeyword::Dark,
    ];
    let value = CssColorScheme::try_new(entries.clone(), true).unwrap();
    assert_eq!(value.schemes(), &entries);
    assert!(value.only());
    assert!(!value.is_normal());
    assert_eq!(
        value.serialize_specified().unwrap(),
        "light light Future future dark only"
    );
    assert_ne!(value, CssColorScheme::try_new(entries, false).unwrap());
    let normal = CssColorScheme::normal();
    assert!(normal.is_normal());
    assert!(normal.schemes().is_empty());
    assert!(!normal.only());
    assert_eq!(normal.serialize_specified().unwrap(), "normal");
}

#[test]
fn parsed_and_checked_schemes_canonicalize_only_last_without_erasing_authored_data() {
    for (authored, expected) in [
        ("NORMAL", "normal"),
        ("only LIGHT Dark", "light dark only"),
        ("light dark ONLY", "light dark only"),
        ("LIGHT light Future future", "light light Future future"),
        ("none span auto", "none span auto"),
        (r"only f\75 ture\ dawn DARK", r"future\ dawn dark only"),
    ] {
        let source = declaration("color-scheme", authored);
        let original = source.value_components().clone();
        let value = scheme(&source);
        let snapshot = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{authored}");
        assert_eq!(value, &snapshot);
        assert_eq!(source.value_components(), &original);
        assert_eq!(source.importance(), CssImportance::Important);
        assert!(
            original
                .items()
                .iter()
                .all(|item| !matches!(item.origin(), CssValueOrigin::Programmatic))
        );
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ColorScheme),
            original.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(scheme(&checked), value);
        assert_eq!(checked.value_components(), &original);
        let roundtrip = declaration("color-scheme", expected);
        assert_eq!(scheme(&roundtrip), value);
    }
    let value =
        CssColorScheme::try_new(vec![custom("Future Theme"), custom("9dawn")], false).unwrap();
    assert_eq!(
        value.serialize_specified().unwrap(),
        r"Future\ Theme \39 dawn"
    );
}

#[test]
fn scheme_modifier_consumes_cumulative_nodes_and_late_bytes_atomically() {
    let source = declaration("color-scheme", "only light dark");
    let original = source.value_components().clone();
    let value = scheme(&source);
    let snapshot = value.clone();
    let exact = CssSpecifiedValueSerializationLimits::new(4, 4, 15);
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        "light dark only"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, 15),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, 15),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, 14),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &snapshot);
        assert_eq!(source.value_components(), &original);
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            "light dark only"
        );
    }
    let normal = CssColorScheme::normal();
    assert_eq!(
        normal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "normal"
    );
    assert_eq!(
        normal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 5))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn escaped_custom_scheme_bytes_share_the_aggregate_budget() {
    let value = CssColorScheme::try_new(
        vec![CssColorSchemeKeyword::Light, custom("Future Theme")],
        true,
    )
    .unwrap();
    let original = value.clone();
    let exact = CssSpecifiedValueSerializationLimits::new(4, 4, 24);
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        r"light Future\ Theme only"
    );
    for bytes in [12, 23] {
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    4, 4, bytes
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, original);
    }
}

#[test]
fn keyword_models_emit_source_derived_canonical_hints_with_exact_limits() {
    for (text, value) in [
        ("AUTO", CssForcedColorAdjust::Auto),
        ("NONE", CssForcedColorAdjust::None),
        (
            r"preserve-parent-c\6f lor",
            CssForcedColorAdjust::PreserveParentColor,
        ),
    ] {
        let source = declaration("forced-color-adjust", text);
        let CssKnownPropertyValueRef::ForcedColorAdjust(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("forced hint")
        };
        assert_eq!(wrapper.value(), &value);
        let expected = match value {
            CssForcedColorAdjust::Auto => "auto",
            CssForcedColorAdjust::None => "none",
            CssForcedColorAdjust::PreserveParentColor => "preserve-parent-color",
            _ => unreachable!(),
        };
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
    for (text, value, expected) in [
        ("ECONOMY", CssPrintColorAdjust::Economy, "economy"),
        (r"e\78 act", CssPrintColorAdjust::Exact, "exact"),
    ] {
        for name in ["print-color-adjust", "color-adjust"] {
            let source = declaration(name, text);
            let typed = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::PrintColorAdjust(wrapper) => wrapper.value(),
                CssKnownPropertyValueRef::ColorAdjust(wrapper) => wrapper.value(),
                _ => panic!("print hint"),
            };
            assert_eq!(typed, &value);
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
}

#[test]
fn reserved_scheme_failure_maps_to_the_original_late_mixed_source_token() {
    let head = parse_component_values("light").unwrap();
    let tail = parse_component_values("DEFAULT").unwrap();
    let original_tail = tail.items()[0].origin().clone();
    let components =
        CssComponentValues::try_new(vec![head.items()[0].clone(), tail.items()[0].clone()])
            .unwrap();
    let snapshot = components.clone();
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ColorScheme),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(_)
    ));
    assert_eq!(error.origin(), &CssSerializedOrigin::Token(original_tail));
    assert_eq!(components, snapshot);
}

#[test]
fn shorthand_expansion_keeps_one_typed_print_hint_and_original_identity() {
    let source = declaration("color-adjust", "EXACT");
    let original = source.value_components().clone();
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("one completed member")
    };
    let [item] = values.items() else {
        panic!("exactly one print member")
    };
    assert_eq!(item.property(), CssKnownProperty::PrintColorAdjust);
    let CssLonghandValueRef::PrintColorAdjust(value) = item.ordinary_value().unwrap().view() else {
        panic!("typed print contribution")
    };
    assert_eq!(*value, CssPrintColorAdjust::Exact);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(
        item.source().known().unwrap().property(),
        CssKnownProperty::ColorAdjust
    );
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(source.value_components(), &original);
    assert_eq!(value.serialize_specified().unwrap(), "exact");
    assert_ne!(
        CssKnownProperty::PrintColorAdjust,
        CssKnownProperty::ColorAdjust
    );
}

#[test]
fn adjustment_properties_expose_complete_selected_source_and_separate_identities() {
    for (property, id) in [
        (
            CssKnownProperty::ColorScheme,
            "official.property.color-scheme",
        ),
        (
            CssKnownProperty::ForcedColorAdjust,
            "official.property.forced-color-adjust",
        ),
        (
            CssKnownProperty::PrintColorAdjust,
            "official.property.print-color-adjust",
        ),
        (
            CssKnownProperty::ColorAdjust,
            "official.property.color-adjust",
        ),
    ] {
        let support = property_support_metadata(property.canonical_name()).unwrap();
        assert_eq!(support.property(), property);
        assert!(support.aliases().is_empty());
        let feature = support.feature();
        assert_eq!(feature.id().as_str(), id);
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        assert_eq!(feature.supported_subset(), None);
        assert_eq!(feature.unsupported_remainder(), None);
        assert_eq!(feature.source().id().as_str(), "R-COLORADJUST1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2025/CR-css-color-adjust-1-20251216/")
        );
    }
}
