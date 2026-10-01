#![forbid(unsafe_code)]
//! Ordinary CSSOM number text has at most six fractional places (CSSOM
//! WD 2021-08-26, component serialization). Decimal halfway direction is the
//! selected frozen WebKit FIXED policy: nearest, ties away from zero.
//! Goldens below count decimal places directly; no host float is an oracle.

use surgeist_css::*;

fn component(source: &str) -> CssComponentValue {
    let values = parse_component_values(source).unwrap();
    let [value] = values.items() else {
        panic!("one component: {source}")
    };
    value.clone()
}

fn number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(component(source)).unwrap()
}

#[test]
fn ordinary_fraction_is_rounded_and_shortened_without_changing_authored_identity() {
    for (source, expected) in [
        ("0.12345649", "0.123456"),
        ("+0001.230000000", "1.23"),
        (".5", "0.5"),
        ("12.3456789e-1", "1.234568"),
        ("9007199254740993.12345649", "9007199254740993.123456"),
    ] {
        for original in [
            component(source),
            CssComponentValue::try_number(source).unwrap(),
        ] {
            let value = CssSpecifiedNumber::try_from_component(original.clone()).unwrap();
            let before = value.clone();
            assert_eq!(value.serialize_specified().unwrap(), expected, "{source}");
            assert_eq!(value, before);
            assert_eq!(value.literal_component(), Some(&original));
            assert_eq!(value.origin(), original.origin());
            assert_eq!(
                CssComponentValues::try_new(vec![original.clone()])
                    .unwrap()
                    .serialize()
                    .unwrap()
                    .as_css(),
                source
            );
            assert_eq!(number(expected).serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn positive_tie_and_decimal_neighbors_round_to_nearest_with_away_ties() {
    for (source, expected) in [
        ("0.0078125", "0.007813"),
        ("0.123456499999999999999999", "0.123456"),
        ("0.1234565", "0.123457"),
        ("0.123456500000000000000001", "0.123457"),
    ] {
        assert_eq!(
            number(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn negative_tie_and_decimal_neighbors_round_to_nearest_with_away_ties() {
    // -0.0078125 = -1/128 is also an exact binary tie discriminator.
    for (source, expected) in [
        ("-0.0078125", "-0.007813"),
        ("-0.123456499999999999999999", "-0.123456"),
        ("-0.1234565", "-0.123457"),
        ("-0.123456500000000000000001", "-0.123457"),
    ] {
        assert_eq!(
            number(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn carry_propagates_across_all_nines_and_trims_the_fraction() {
    for (source, expected) in [
        ("0.9999996", "1"),
        ("99.9999995", "100"),
        ("-99.9999995", "-100"),
    ] {
        assert_eq!(
            number(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn rounded_zero_loses_its_sign_but_half_a_micro_unit_does_not() {
    for (source, expected) in [
        ("-0.0000004", "0"),
        ("-0.000000499999999999", "0"),
        ("-0.0000005", "-0.000001"),
        ("0.0000005", "0.000001"),
        ("-0e999999999999999999999999999999999999999999", "0"),
    ] {
        assert_eq!(
            number(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn enormous_negative_exponent_rounds_to_zero_with_a_one_byte_budget() {
    for source in [
        "1e-999999999999999999999999999999999999999999",
        "-1e-999999999999999999999999999999999999999999",
    ] {
        let value = number(source);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 1))
                .unwrap(),
            "0"
        );
        assert_eq!(value, before);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 0))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn budgets_charge_actual_rounded_output_and_preserve_input_on_every_failure() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    for (source, expected, bytes) in [("0.12345649", "0.123456", 8), ("0.9999996", "1", 1)] {
        let value = number(source);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, bytes))
                .unwrap(),
            expected
        );
        assert_eq!(value, before);
        for (limits, kind) in [
            (L::new(0, 1, bytes), K::InputNodeLimit),
            (L::new(1, 0, bytes), K::ProjectionNodeLimit),
            (L::new(1, 1, bytes - 1), K::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, before);
            assert_eq!(value.literal_component(), before.literal_component());
            assert_eq!(value.origin(), before.origin());
        }
    }
}

#[test]
fn suffix_bytes_follow_rounded_length_without_changing_the_raw_component() {
    use CssSpecifiedValueSerializationLimits as L;
    for (source, expected, bytes) in [
        ("0.12345649px", "0.123456px", 10),
        ("0.9999996px", "1px", 3),
    ] {
        let raw = component(source);
        let value = CssSpecifiedLength::try_from_component(raw.clone()).unwrap();
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, bytes))
                .unwrap(),
            expected
        );
        assert_eq!(value, before);
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, bytes - 1))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
        assert_eq!(value.literal_component(), Some(&raw));
        assert_eq!(value.origin(), raw.origin());
        assert_eq!(
            CssComponentValues::try_new(vec![raw.clone()])
                .unwrap()
                .serialize()
                .unwrap()
                .as_css(),
            source
        );
    }
}

#[test]
fn rounded_carry_fits_one_byte_without_budgeting_unrounded_text() {
    let value = number("0.9999996");
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 1))
            .unwrap(),
        "1"
    );
    assert_eq!(value, before);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 0))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value, before);
}

#[test]
fn tiny_milliseconds_round_to_seconds_zero_with_its_two_byte_suffix_budget() {
    let value = CssTimeLiteral::try_new("0.0001", CssTimeUnit::Milliseconds).unwrap();
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 2))
            .unwrap(),
        "0s"
    );
    assert_eq!(value, before);
    assert_eq!(value.numeric().representation(), "0.0001");
    assert_eq!(value.unit(), CssTimeUnit::Milliseconds);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 1))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value, before);
}

#[test]
fn unit_normalization_precedes_rounding_and_keeps_authored_coefficients_and_units() {
    // 123.45649 ms / 1000 = .12345649 s; .0001 ms / 1000 = .0000001 s.
    for (coefficient, expected) in [
        ("123.45649", "0.123456s"),
        ("0.0001", "0s"),
        ("0.0005", "0.000001s"),
    ] {
        let value = CssTimeLiteral::try_new(coefficient, CssTimeUnit::Milliseconds).unwrap();
        let before = value.clone();
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
        assert_eq!(value, before);
        assert_eq!(value.numeric().representation(), coefficient);
        assert_eq!(value.unit(), CssTimeUnit::Milliseconds);
        assert_eq!(value.origin(), before.origin());
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn opacity_percentage_rounds_only_after_its_exact_division_by_one_hundred() {
    // 12.34565% / 100 = .1234565, a tie only after the exact percentage shift.
    let raw = component("12.34565%");
    let scalar = CssOpacityScalar::try_from_component(raw.clone()).unwrap();
    let value = CssOpacityValue::Scalar(scalar);
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), "0.123457");
    assert_eq!(value, before);
    let CssOpacityValue::Scalar(scalar) = &value else {
        panic!("scalar")
    };
    assert_eq!(scalar.component(), &raw);
}

#[test]
fn exact_admission_precedes_lossy_text_and_huge_positive_output_fails_atomically() {
    for source in ["-1e-999px", "-0.0000004%"] {
        let raw = component(source);
        let before = raw.clone();
        let error =
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(raw.clone()).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(raw.origin()));
        assert_eq!(raw, before);
    }
    assert!(
        CssDuration::try_new(CssTimeValue::from_literal(
            CssTimeLiteral::try_new("-1e-999", CssTimeUnit::Seconds).unwrap()
        ))
        .is_err()
    );
    for source in ["1e999", "1e999999999999999999999999999999999999999999"] {
        let value = number(source);
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 1))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn calculation_text_and_node_and_byte_budgets_keep_the_existing_contract() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    let calculation = CssFrequencyCalculation::try_from_components(
        parse_component_values("calc(1hz + 2hz)").unwrap(),
    )
    .unwrap();
    let raw = calculation.serialize().unwrap();
    let value = CssFrequencyValue::try_from_calculation(calculation).unwrap();
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(4, 3, 9))
            .unwrap(),
        "calc(3hz)"
    );
    for (limits, kind) in [
        (L::new(3, 3, 9), K::InputNodeLimit),
        (L::new(4, 2, 9), K::ProjectionNodeLimit),
        (L::new(4, 3, 8), K::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
        assert_eq!(
            value.calculation().unwrap().serialize().unwrap().as_css(),
            raw.as_css()
        );
    }
    let math = CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 / 3)").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        math.serialize_specified().unwrap(),
        "calc(0.3333333333333333)"
    );
}

#[test]
fn integer_color_and_signed_media_keep_their_distinct_public_text_contracts() {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Order),
        parse_component_values("round(-2.5)").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Order(order) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("order")
    };
    assert_eq!(order.value().serialize_specified().unwrap(), "calc(-2)");
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values("rgb(99.999999999999999999999999999999% 0 0)").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(color) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    assert_eq!(
        color.value().to_specified_css().unwrap(),
        "rgb(254.99999999999999999999999999999745, 0, 0)"
    );
    let report = parse_media_query("(width: -0.12345649px)");
    assert!(report.is_clean());
    let query = report.syntax();
    let before = query.clone();
    assert_eq!(
        query.serialize().unwrap().as_css(),
        "(width: -0.12345649px)"
    );
    assert_eq!(query, &before);
}

// Canonical ordinary output through composed specified-value writers.
mod composed_ordinary_values {
    use super::*;
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;

    fn declaration(property: CssKnownProperty, authored: &str) -> CssDeclaration {
        let report = parse_style_attribute(&format!(
            "{}:{authored}!important",
            property.canonical_name()
        ));
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        let parsed = report.syntax()[0].clone();
        let components = parse_component_values(authored).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        assert_eq!(checked.importance(), CssImportance::Important);
        assert_eq!(components.serialize().unwrap().as_css(), authored);
        assert_eq!(specified(&parsed), specified(&checked));
        checked
    }

    fn specified(source: &CssDeclaration) -> String {
        match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::ClipPath(v) => v.value().serialize_specified().unwrap(),
            CssKnownPropertyValueRef::BackgroundPosition(v) => {
                v.positions().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::BackgroundImage(v) => {
                v.images().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::Filter(v) => v.value().serialize_specified().unwrap(),
            CssKnownPropertyValueRef::GridTemplateColumns(v) => {
                v.value().serialize_specified().unwrap()
            }
            CssKnownPropertyValueRef::FontStyle(v) => v.value().serialize_specified().unwrap(),
            CssKnownPropertyValueRef::FontWeight(v) => v.value().serialize_specified().unwrap(),
            CssKnownPropertyValueRef::AspectRatio(v) => v.ratio().serialize_specified().unwrap(),
            _ => panic!("fixture has an existing specified emitter"),
        }
    }

    #[test]
    fn composed_frontdoors_round_children_and_preserve_raw_reentry() {
        for (property, authored, expected) in [
            (
                CssKnownProperty::BackgroundPosition,
                "right 0.12345649px bottom 0.1234565%",
                "right 0.123456px bottom 0.123457%",
            ),
            (
                CssKnownProperty::ClipPath,
                "circle(at 0.12345649px 0.1234565%)",
                "circle(at 0.123456px 0.123457%)",
            ),
            (
                CssKnownProperty::ClipPath,
                "shape(from 0.12345649px 0.12345649%, curve by 0.9999996px -0.0000004% with 0.1234565px 0.12345649% / -0.1234565px 1e-9%, arc by 2px 3px of 0.9999996px -0.1234565% rotate 0.1234565deg)",
                "shape(from 0.123456px 0.123456%, curve by 1px 0% with 0.123457px 0.123456% / -0.123457px 0%, arc by 2px 3px of 1px -0.123457% rotate 0.123457deg)",
            ),
            (
                CssKnownProperty::ClipPath,
                "xywh(0.12345649px -0.1234565px 0.9999996px 1e-9%)",
                "xywh(0.123456px -0.123457px 1px 0%)",
            ),
            (
                CssKnownProperty::BackgroundImage,
                "linear-gradient(180.0000004deg, red 0.12345649%, blue 0.1234565%)",
                "linear-gradient(180deg, red 0.123456%, blue 0.123457%)",
            ),
            (
                CssKnownProperty::Filter,
                "hue-rotate(0.1234565deg) opacity(0.12345649) blur(0.9999996px)",
                "hue-rotate(0.123457deg) opacity(0.123456) blur(1px)",
            ),
            (
                CssKnownProperty::GridTemplateColumns,
                "0.12345649fr minmax(0.9999996px,0.1234565%)",
                "0.123456fr minmax(1px, 0.123457%)",
            ),
            (CssKnownProperty::FontWeight, "725.1234565", "725.123457"),
            (
                CssKnownProperty::FontStyle,
                "oblique 14.1234565deg",
                "oblique 14.123457deg",
            ),
            (
                CssKnownProperty::AspectRatio,
                "auto .12345649 / .1234565",
                "auto 0.123456 / 0.123457",
            ),
        ] {
            let source = declaration(property, authored);
            let before = source.clone();
            assert_eq!(specified(&source), expected, "{authored}");
            assert_eq!(source, before);
        }
    }

    #[test]
    fn exact_gradient_default_omission_does_not_use_rounded_text() {
        let near = declaration(
            CssKnownProperty::BackgroundImage,
            "linear-gradient(180.0000004deg, red, blue)",
        );
        let exact = declaration(
            CssKnownProperty::BackgroundImage,
            "linear-gradient(180deg, red, blue)",
        );
        assert_eq!(specified(&near), "linear-gradient(180deg, red, blue)");
        assert_eq!(specified(&exact), "linear-gradient(red, blue)");
        assert_ne!(near.value_components(), exact.value_components());
    }

    #[test]
    fn ratio_and_filter_count_existing_child_visits_and_actual_rounded_bytes() {
        let ratio = CssSpecifiedRatio::new(
            CssRatioOperand::try_from_component(component(".12345649")).unwrap(),
            Some(CssRatioOperand::try_from_component(component(".1234565")).unwrap()),
        );
        let expected = "0.123456 / 0.123457";
        // No aggregate visit: the existing pair writer visits two scalars.
        assert_eq!(
            ratio
                .serialize_specified_with_limits(L::new(2, 2, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(1, 2, expected.len()), K::InputNodeLimit),
            (L::new(2, 1, expected.len()), K::ProjectionNodeLimit),
            (L::new(2, 2, expected.len() - 1), K::ByteLimit),
        ] {
            let before = ratio.clone();
            assert_eq!(
                ratio
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(ratio, before);
        }
        let source = declaration(
            CssKnownProperty::Filter,
            "hue-rotate(0.1234565deg) blur(0.9999996px)",
        );
        let CssKnownPropertyValueRef::Filter(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("filter")
        };
        let filter = value.value();
        let expected = "hue-rotate(0.123457deg) blur(1px)";
        // One list, two function wrappers, two ordinary scalars = five visits.
        assert_eq!(
            filter
                .serialize_specified_with_limits(L::new(5, 5, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(4, 5, expected.len()), K::InputNodeLimit),
            (L::new(5, 4, expected.len()), K::ProjectionNodeLimit),
            (L::new(5, 5, expected.len() - 1), K::ByteLimit),
        ] {
            let before = filter.clone();
            assert_eq!(
                filter
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(filter, &before);
        }
    }

    #[test]
    fn timing_and_easing_preserve_list_identity_while_existing_children_round() {
        let report = parse_style_attribute(
            "transition-duration:0.0001ms,1.1234565s;transition-delay:-0.0001ms;transition-timing-function:cubic-bezier(.12345649,-.1234565,.9999996,.1234565)",
        );
        assert!(report.is_clean());
        let declarations = report.syntax();
        let before = declarations.clone();
        let CssKnownPropertyValueRef::TransitionDuration(value) =
            declarations[0].known().unwrap().property_value().unwrap()
        else {
            panic!("duration")
        };
        let values = value.durations();
        assert_eq!(values.values()[0].serialize_specified().unwrap(), "0s");
        assert_eq!(
            values.values()[1].serialize_specified().unwrap(),
            "1.123457s"
        );
        assert_eq!(
            values.values()[0]
                .time()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            "0.0001"
        );
        assert_eq!(
            CssDurationList::try_new(values.values().to_vec()).unwrap(),
            *values
        );
        let CssKnownPropertyValueRef::TransitionDelay(value) =
            declarations[1].known().unwrap().property_value().unwrap()
        else {
            panic!("delay")
        };
        assert_eq!(
            value.delays().values()[0].serialize_specified().unwrap(),
            "0s"
        );
        assert_eq!(
            CssDelayList::try_new(value.delays().values().to_vec()).unwrap(),
            *value.delays()
        );
        let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
            declarations[2].known().unwrap().property_value().unwrap()
        else {
            panic!("easing")
        };
        let CssEasing::CubicBezier(bezier) = &value.timing_functions().values()[0] else {
            panic!("bezier")
        };
        for (number, expected) in [
            (bezier.x1().value(), "0.123456"),
            (bezier.y1(), "-0.123457"),
            (bezier.x2().value(), "1"),
            (bezier.y2(), "0.123457"),
        ] {
            assert_eq!(number.serialize_specified().unwrap(), expected);
        }
        assert_eq!(
            CssEasingList::try_new(value.timing_functions().values().to_vec()).unwrap(),
            *value.timing_functions()
        );
        assert_eq!(declarations, &before);
    }
}
