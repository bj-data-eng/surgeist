#![forbid(unsafe_code)]
//! Functional tests for new typed fronts; no executable RED preceded these APIs.
//! Filter1 WD20181218 §§9.13,10,11.5 supplies identities and intrinsic initials;
//! Values4/CSSOM and the selected exact Color/opacity owners supply projection.
use surgeist_css::*;

fn declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("terminal contribution")
    };
    let [value] = values.items() else {
        panic!("one contribution")
    };
    assert!(value.source().same_occurrence(source));
    assert_eq!(value.source().importance(), CssImportance::Important);
    values
}

fn opacity(source: &CssDeclaration) -> &CssOpacityValue {
    let CssKnownPropertyValueRef::FloodOpacity(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("flood opacity wrapper")
    };
    value.value()
}

#[test]
fn interpolation_typed_choices_keep_auto_distinct_and_emit_lowercase_keywords() {
    assert_eq!(
        CssColorInterpolationFilters::default(),
        CssColorInterpolationFilters::LinearRgb
    );
    assert_ne!(
        CssColorInterpolationFilters::Auto,
        CssColorInterpolationFilters::Srgb
    );
    assert_ne!(
        CssColorInterpolationFilters::Auto,
        CssColorInterpolationFilters::LinearRgb
    );
    for (value, authored, canonical) in [
        (CssColorInterpolationFilters::Auto, "AuTo", "auto"),
        (CssColorInterpolationFilters::Srgb, "sRGB", "srgb"),
        (
            CssColorInterpolationFilters::LinearRgb,
            "linearRGB",
            "linearrgb",
        ),
    ] {
        let before = value;
        assert_eq!(value.serialize_specified().unwrap(), canonical);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    canonical.len()
                ))
                .unwrap(),
            canonical
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, canonical.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, canonical.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, canonical.len() - 1),
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
            assert_eq!(value, before);
            assert_eq!(value.serialize_specified().unwrap(), canonical);
        }
        for text in [authored, canonical] {
            let source = declaration(CssKnownProperty::ColorInterpolationFilters, text);
            let CssKnownPropertyValueRef::ColorInterpolationFilters(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("interpolation wrapper")
            };
            assert_eq!(*wrapper.value(), value);
            assert_eq!(wrapper.as_css(), text);
            let values = contributions(&source);
            let CssContributionValueRef::Ordinary(CssLonghandValueRef::ColorInterpolationFilters(
                contributed,
            )) = values.items()[0].value()
            else {
                panic!("interpolation payload")
            };
            assert_eq!(*contributed, value);
        }
    }
}

#[test]
fn typed_intrinsic_initials_are_black_number_one_white_and_linear_rgb() {
    for property in [
        CssKnownProperty::FloodColor,
        CssKnownProperty::FloodOpacity,
        CssKnownProperty::LightingColor,
        CssKnownProperty::ColorInterpolationFilters,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("longhand metadata")
        };
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("intrinsic initial")
        };
        match (property, value.view()) {
            (CssKnownProperty::FloodColor, CssLonghandValueRef::FloodColor(value)) => {
                assert_eq!(value.named().unwrap().name(), "black");
                assert_eq!(value.keyword_srgba8(), Some([0, 0, 0, 255]));
            }
            (CssKnownProperty::LightingColor, CssLonghandValueRef::LightingColor(value)) => {
                assert_eq!(value.named().unwrap().name(), "white");
                assert_eq!(value.keyword_srgba8(), Some([255, 255, 255, 255]));
            }
            (
                CssKnownProperty::FloodOpacity,
                CssLonghandValueRef::FloodOpacity(CssOpacityValue::Scalar(value)),
            ) => {
                assert_eq!(value.kind(), CssOpacityScalarKind::Number);
                assert_eq!(value.numeric().representation(), "1");
                assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
            }
            (
                CssKnownProperty::ColorInterpolationFilters,
                CssLonghandValueRef::ColorInterpolationFilters(value),
            ) => {
                assert_eq!(*value, CssColorInterpolationFilters::LinearRgb);
            }
            _ => panic!("property-coupled initial"),
        }
        assert_eq!(
            metadata.inherited_by_default(),
            property == CssKnownProperty::ColorInterpolationFilters
        );
    }
}

#[test]
fn flood_and_lighting_wrappers_contribute_the_same_unresolved_color_payload() {
    for property in [
        CssKnownProperty::FloodColor,
        CssKnownProperty::LightingColor,
    ] {
        for (text, specified) in [
            ("currentColor", "currentcolor"),
            ("Canvas", "canvas"),
            ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
            ("rgb(from red r g b / alpha)", "rgb(from red r g b / alpha)"),
        ] {
            let source = declaration(property, text);
            let value = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::FloodColor(wrapper) => {
                    assert_eq!(wrapper.as_css(), text);
                    wrapper.value()
                }
                CssKnownPropertyValueRef::LightingColor(wrapper) => {
                    assert_eq!(wrapper.as_css(), text);
                    wrapper.value()
                }
                _ => panic!("color wrapper"),
            };
            if text == "currentColor" {
                assert!(value.is_current_color());
            }
            if text == "Canvas" {
                assert_eq!(value.system(), Some(CssSystemColor::Canvas));
            }
            let values = contributions(&source);
            let contributed =
                match values.items()[0].value() {
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodColor(value))
                        if property == CssKnownProperty::FloodColor =>
                    {
                        value
                    }
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::LightingColor(
                        value,
                    )) if property == CssKnownProperty::LightingColor => value,
                    _ => panic!("property-coupled color contribution"),
                };
            assert_eq!(contributed, value);
            assert_eq!(contributed.to_specified_css().unwrap(), specified);
        }
    }
}

#[test]
fn flood_scalar_payload_preserves_exact_original_kind_token_and_origin() {
    for (text, kind, number, specified) in [
        (
            "+000.1000e1",
            CssOpacityScalarKind::Number,
            "+000.1000e1",
            "1",
        ),
        ("-2.5", CssOpacityScalarKind::Number, "-2.5", "-2.5"),
        ("150%", CssOpacityScalarKind::Percentage, "150", "1.5"),
        ("-25%", CssOpacityScalarKind::Percentage, "-25", "-0.25"),
        ("1e-47", CssOpacityScalarKind::Number, "1e-47", "0"),
    ] {
        let source = declaration(CssKnownProperty::FloodOpacity, text);
        let CssOpacityValue::Scalar(value) = opacity(&source) else {
            panic!("scalar")
        };
        assert_eq!(value.kind(), kind);
        assert_eq!(value.numeric().representation(), number);
        assert_eq!(value.component(), &source.value_components().items()[0]);
        assert_eq!(
            value.origin(),
            source.value_components().items()[0].origin()
        );
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        assert_eq!(opacity(&source).serialize_specified().unwrap(), specified);
        let values = contributions(&source);
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodOpacity(contributed)) =
            values.items()[0].value()
        else {
            panic!("alpha contribution")
        };
        assert_eq!(contributed, opacity(&source));
    }
}

#[test]
fn flood_math_payloads_retain_checked_roots_and_unresolved_percentage_hint() {
    for (text, specified) in [
        ("calc(2 - 3)", "calc(-1)"),
        ("calc(25% + 25%)", "calc(50%)"),
        ("calc((1px + 1%) / 1px)", "calc((1% + 1px) / 1px)"),
    ] {
        let source = declaration(CssKnownProperty::FloodOpacity, text);
        let value = opacity(&source);
        match value {
            CssOpacityValue::NumberCalculation(root) if text == "calc(2 - 3)" => {
                assert_eq!(root.result_type(), CssCalculationType::Number);
                assert_eq!(root.numeric_type().percent_hint(), None);
                assert_eq!(root.components(), source.value_components());
                assert_eq!(root.origin(), source.value_components().items()[0].origin());
            }
            CssOpacityValue::PercentageCalculation(root) if text == "calc(25% + 25%)" => {
                assert_eq!(root.result_type(), CssCalculationType::Percentage);
                assert_eq!(root.components(), source.value_components());
                assert_eq!(root.origin(), source.value_components().items()[0].origin());
            }
            CssOpacityValue::HintedNumberCalculation(root) if text == "calc((1px + 1%) / 1px)" => {
                assert_eq!(root.result_type(), CssCalculationType::Number);
                assert_eq!(
                    root.numeric_type().percent_hint(),
                    Some(CssNumericDimension::Length)
                );
                assert_eq!(root.numeric_type().exponent(CssNumericDimension::Length), 0);
                assert_eq!(root.components(), source.value_components());
                assert_eq!(root.origin(), source.value_components().items()[0].origin());
            }
            _ => panic!("matching exact numeric root"),
        }
        assert_eq!(value.serialize_specified().unwrap(), specified);
        let values = contributions(&source);
        let CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodOpacity(contributed)) =
            values.items()[0].value()
        else {
            panic!("alpha contribution")
        };
        assert_eq!(contributed, value);
    }
}

#[test]
fn normalization_and_pending_retry_expose_property_coupled_typed_payloads() {
    let report = parse_sheet(
        ".x{flood-color:red;flood-opacity:-25%;lighting-color:currentcolor;color-interpolation-filters:auto}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let mut order = 0;
    for item in normalized.items() {
        let CssNormalizedItem::Declaration(declaration) = item else {
            continue;
        };
        assert_eq!(declaration.order(), order);
        order += 1;
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("completed terminal")
        };
        let [item] = values.items() else {
            panic!("one typed terminal")
        };
        assert!(item.source().same_occurrence(declaration.source()));
        match (
            declaration.source().known().unwrap().property(),
            item.value(),
        ) {
            (
                CssKnownProperty::FloodColor,
                CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodColor(value)),
            ) => assert_eq!(value.named().unwrap().name(), "red"),
            (
                CssKnownProperty::FloodOpacity,
                CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodOpacity(
                    CssOpacityValue::Scalar(value),
                )),
            ) => {
                assert_eq!(value.kind(), CssOpacityScalarKind::Percentage);
                assert_eq!(value.numeric().representation(), "-25");
            }
            (
                CssKnownProperty::LightingColor,
                CssContributionValueRef::Ordinary(CssLonghandValueRef::LightingColor(value)),
            ) => assert!(value.is_current_color()),
            (
                CssKnownProperty::ColorInterpolationFilters,
                CssContributionValueRef::Ordinary(CssLonghandValueRef::ColorInterpolationFilters(
                    value,
                )),
            ) => assert_eq!(*value, CssColorInterpolationFilters::Auto),
            _ => panic!("property-coupled normalized payload"),
        }
    }
    assert_eq!(order, 4);

    let source = declaration(CssKnownProperty::FloodOpacity, "var(--alpha)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending alpha")
    };
    assert!(
        handle
            .reenter(parse_component_values("red").unwrap())
            .is_err()
    );
    let replacement = parse_component_values("-25%").unwrap();
    let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap() else {
        panic!("alpha terminal")
    };
    let [item] = values.items() else {
        panic!("one replacement")
    };
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::FloodOpacity(
        CssOpacityValue::Scalar(value),
    )) = item.value()
    else {
        panic!("typed replacement scalar")
    };
    assert_eq!(value.kind(), CssOpacityScalarKind::Percentage);
    assert_eq!(value.numeric().representation(), "-25");
    assert_eq!(value.component(), &replacement.items()[0]);
    assert_eq!(value.origin(), replacement.items()[0].origin());
    assert_eq!(item.replacement_components(), Some(&replacement));
    assert!(item.source().same_occurrence(&source));
}
