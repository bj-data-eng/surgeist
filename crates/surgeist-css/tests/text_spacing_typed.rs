#![forbid(unsafe_code)]

//! Functional CSS Text 4 §§8.1–8.2 authored spacing contracts.
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-word-spacing
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing

use surgeist_css::*;

fn component(text: &str) -> CssComponentValue {
    let components = parse_component_values(text).unwrap();
    let [value] = components.items() else {
        panic!("expected one component: {text}")
    };
    value.clone()
}

fn numeric(text: &str) -> CssSpecifiedLengthPercentage {
    if text.starts_with("calc(") {
        CssSpecifiedLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedLengthPercentage::try_from_component(component(text)).unwrap()
    }
}

fn adjustment(text: &str) -> CssTextSpacingAdjustment {
    if text == "normal" {
        CssTextSpacingAdjustment::Normal
    } else {
        CssTextSpacingAdjustment::LengthPercentage(numeric(text))
    }
}

fn declared(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}: {value} !important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn expanded_value(declaration: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("spacing is one longhand")
    };
    let [item] = items.items() else {
        panic!("one spacing contribution")
    };
    item.ordinary_value().unwrap().clone()
}

#[test]
fn checked_numeric_domain_is_signed_length_percentage_and_origin_is_retained() {
    for (authored, canonical) in [
        ("normal", "normal"),
        ("0", "0"),
        ("0%", "0%"),
        ("10%", "10%"),
        ("-12.5%", "-12.5%"),
        ("-2em", "-2em"),
        ("calc(1px + 2%)", "calc(2% + 1px)"),
        ("calc(1px - 2px)", "calc(-1px)"),
    ] {
        assert_eq!(
            adjustment(authored).serialize_specified().unwrap(),
            canonical
        );
    }
    for invalid in ["1", "1fr", "auto", "stretch"] {
        assert!(
            CssSpecifiedLengthPercentage::try_from_component(component(invalid)).is_err(),
            "{invalid}"
        );
    }
    let programmatic = CssTextSpacingAdjustment::LengthPercentage(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_token("-12.5%").unwrap(),
        )
        .unwrap(),
    );
    assert_eq!(programmatic.serialize_specified().unwrap(), "-12.5%");
    let CssTextSpacingAdjustment::LengthPercentage(number) = &programmatic else {
        panic!("checked percentage adjustment")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Percentage(token)) =
        number.literal_component().unwrap().view()
    else {
        panic!("checked percentage token")
    };
    assert_eq!(token.representation(), "-12.5");
    assert!(matches!(
        programmatic.origin(),
        Some(CssValueOrigin::Programmatic)
    ));
    assert!(matches!(
        adjustment("-12.5%").origin(),
        Some(CssValueOrigin::Parsed(_))
    ));
    assert_eq!(programmatic, adjustment("-12.5%"));
    assert!(CssWordSpacingLength::try_new(CssLength::try_percent(0.0).unwrap()).is_none());
    assert!(CssLetterSpacingLength::try_new(CssLength::try_percent(0.0).unwrap()).is_none());
}

#[test]
fn exact_large_exponents_avoid_float_rounding() {
    for (authored, canonical) in [
        ("1e999px", format!("1{}px", "0".repeat(999))),
        ("-1e999%", format!("-1{}%", "0".repeat(999))),
        ("1e-999px", format!("0.{}1px", "0".repeat(998))),
    ] {
        assert_eq!(
            adjustment(authored).serialize_specified().unwrap(),
            canonical
        );
    }
}

#[test]
fn parsed_current_values_and_legacy_projection_remain_distinct() {
    for (name, value, canonical, legacy) in [
        ("word-spacing", "normal", "normal", true),
        ("word-spacing", "2px", "2px", true),
        ("word-spacing", "-0.25em", "-0.25em", true),
        ("word-spacing", "0.1em", "0.1em", false),
        ("word-spacing", "0%", "0%", false),
        ("word-spacing", "10%", "10%", false),
        ("word-spacing", "calc(1px + 2%)", "calc(2% + 1px)", false),
        ("word-spacing", "calc(1px + 0%)", "calc(0% + 1px)", false),
        ("word-spacing", "calc(1px - 2px)", "calc(-1px)", true),
        ("letter-spacing", "normal", "normal", true),
        ("letter-spacing", "2px", "2px", true),
        ("letter-spacing", "-0.25em", "-0.25em", true),
        ("letter-spacing", "0.1em", "0.1em", false),
        ("letter-spacing", "0%", "0%", false),
        ("letter-spacing", "-12.5%", "-12.5%", false),
        ("letter-spacing", "calc(1px + 2%)", "calc(2% + 1px)", false),
        ("letter-spacing", "calc(1px + 0%)", "calc(0% + 1px)", false),
        ("letter-spacing", "calc(1px - 2px)", "calc(-1px)", true),
    ] {
        let declaration = declared(name, value);
        let parsed = declaration.known().unwrap().property_value().unwrap();
        let (current, has_legacy) = match parsed {
            CssKnownPropertyValueRef::WordSpacing(wrapper) => {
                assert_eq!(wrapper.as_css(), value);
                (wrapper.spacing(), wrapper.i01_subset().is_some())
            }
            CssKnownPropertyValueRef::LetterSpacing(wrapper) => {
                assert_eq!(wrapper.as_css(), value);
                (wrapper.current(), wrapper.i01_subset().is_some())
            }
            _ => panic!("spacing property: {name}"),
        };
        assert_eq!(current.serialize_specified().unwrap(), canonical);
        assert_eq!(has_legacy, legacy, "{name}: {value}");
        if value != "normal" {
            assert!(matches!(current.origin(), Some(CssValueOrigin::Parsed(_))));
        }
        let value = expanded_value(&declaration);
        match value.view() {
            CssLonghandValueRef::WordSpacing(value) | CssLonghandValueRef::LetterSpacing(value) => {
                assert_eq!(value.serialize_specified().unwrap(), canonical)
            }
            _ => panic!("typed spacing contribution"),
        }
    }
}

#[test]
fn both_initials_are_symbolic_normal_values() {
    for property in [
        CssKnownProperty::WordSpacing,
        CssKnownProperty::LetterSpacing,
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("spacing longhand")
        };
        assert!(metadata.inherited_by_default());
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial_value.view() else {
            panic!("normal initial")
        };
        match initial.view() {
            CssLonghandValueRef::WordSpacing(value) | CssLonghandValueRef::LetterSpacing(value) => {
                assert_eq!(value, &CssTextSpacingAdjustment::Normal);
                assert_eq!(value.serialize_specified().unwrap(), "normal");
            }
            _ => panic!("typed spacing initial"),
        }
    }
}

#[test]
fn serialization_limits_fail_atomically_without_changing_value() {
    let normal = CssTextSpacingAdjustment::Normal;
    assert_eq!(
        normal
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "normal"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            normal
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let math = adjustment("calc(1px + 2%)");
    assert_eq!(math.serialize_specified().unwrap(), "calc(2% + 1px)");
    let bytes = "calc(2% + 1px)".len();
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            128,
            128,
            bytes - 1
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(math.serialize_specified().unwrap(), "calc(2% + 1px)");
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            128, 128, bytes
        ))
        .unwrap(),
        "calc(2% + 1px)"
    );
}
