#![forbid(unsafe_code)]
//! Exact authored integer fidelity and Values4 specified math projection.
use surgeist_css::*;

fn order(text: &str) -> CssIntegerValue {
    let report = parse_style_attribute(&format!("order:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Order(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("order")
    };
    value.value().clone()
}

fn exact(text: &str) -> CssIntegerValue {
    CssIntegerValue::ExactLiteral(
        CssIntegerLiteral::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap(),
    )
}

fn assert_output(value: &CssIntegerValue, expected: &str) {
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(order(expected).serialize_specified().unwrap(), expected);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                65_536,
                262_144,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                65_536,
                262_144,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value, &before);
}

#[test]
fn checked_integer_literals_preserve_components_and_reject_other_token_kinds() {
    for text in [
        "0",
        "-0",
        "+0002",
        "-2147483649",
        "9999999999999999999999999999999999999999",
    ] {
        let parsed = parse_component_values(text).unwrap();
        for component in [
            parsed.items()[0].clone(),
            CssComponentValue::try_number(text).unwrap(),
        ] {
            let literal = CssIntegerLiteral::try_from_component(component.clone()).unwrap();
            assert_eq!(literal.component(), &component);
            assert_eq!(literal.origin(), component.origin());
            assert_eq!(literal.numeric().representation(), text);
            assert_eq!(literal.numeric().kind(), CssNumericTokenKind::Integer);
        }
    }
    for text in ["1.0", "1e2", "1%", "1px", "auto", " ", "calc(1)", "[1]"] {
        let components = parse_component_values(text).unwrap();
        assert_eq!(components.items().len(), 1);
        let component = components.items()[0].clone();
        let error = CssIntegerLiteral::try_from_component(component.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), component.origin());
    }
}

#[test]
fn both_integer_properties_preserve_exact_payloads_and_original_origins() {
    for property in [CssKnownProperty::Order, CssKnownProperty::ZIndex] {
        for text in [
            "2147483648",
            "-2147483649",
            "+0009007199254740993",
            "-123456789012345678901234567890",
        ] {
            let components = parse_component_values(text).unwrap();
            for supplied in [
                components,
                CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                    .unwrap(),
            ] {
                let declaration = parse_property_value(
                    CssPropertyNameRef::Known(property),
                    supplied.clone(),
                    CssImportance::Normal,
                )
                .unwrap();
                let payload = match declaration.known().unwrap().property_value().unwrap() {
                    CssKnownPropertyValueRef::Order(v) => v.value(),
                    CssKnownPropertyValueRef::ZIndex(v) => match v.value() {
                        CssZIndexValue::Integer(v) => v,
                        _ => panic!("integer"),
                    },
                    _ => panic!("integer property"),
                };
                let CssIntegerValue::ExactLiteral(literal) = payload else {
                    panic!("exact payload")
                };
                assert_eq!(literal.component(), &supplied.items()[0]);
                assert_eq!(literal.origin(), supplied.items()[0].origin());
                assert_eq!(literal.numeric().representation(), text);
            }
        }
    }
    for (text, expected) in [
        ("+0002147483647", i32::MAX),
        ("-0002147483648", i32::MIN),
        ("-000", 0),
    ] {
        assert_eq!(order(text), CssIntegerValue::Literal(expected));
    }
}

#[test]
fn ordinary_integer_serialization_is_exact_and_canonical_without_magnitude_rounding() {
    for (input, expected) in [
        ("+0002", "2"),
        ("-0002", "-2"),
        ("-000", "0"),
        ("+0", "0"),
        ("2147483648", "2147483648"),
        ("-2147483649", "-2147483649"),
        ("9007199254740993", "9007199254740993"),
        (
            "+000123456789012345678901234567890",
            "123456789012345678901234567890",
        ),
    ] {
        assert_output(&exact(input), expected);
        assert_output(&order(input), expected);
    }
    for (input, expected) in [
        (i32::MIN, "-2147483648"),
        (i32::MAX, "2147483647"),
        (0, "0"),
    ] {
        assert_output(&CssIntegerValue::Literal(input), expected);
    }
    let digits = "9".repeat(4096);
    assert_output(&exact(&digits), &digits);
    assert_output(&order(&digits), &digits);
    assert_output(&exact(&format!("-{}2", "0".repeat(4096))), "-2");
}

#[test]
fn ordinary_integer_limits_charge_canonical_bytes_and_one_node_per_stage() {
    for value in [CssIntegerValue::Literal(-2), exact("-0002")] {
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 0, 0),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, 0),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                expected
            );
        }
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 2))
                .unwrap(),
            "-2"
        );
    }
}

#[test]
fn explicit_calculation_leaf_uses_shared_binary64_without_mutating_exact_input() {
    let components = parse_component_values("9007199254740993").unwrap();
    let calculation = CssIntegerCalculation::try_from_components(components.clone()).unwrap();
    let CssCalculationExpressionRef::Value(leaf) = calculation.expression() else {
        panic!("exact authored leaf")
    };
    assert_eq!(leaf.literal().representation(), "9007199254740993");
    assert_eq!(calculation.components(), &components);
    assert_output(
        &CssIntegerValue::Calculation(calculation.clone()),
        "calc(9007199254740992)",
    );
    assert_eq!(calculation.components(), &components);
    assert_output(&exact("9007199254740993"), "9007199254740993");
}

#[test]
fn integer_context_math_preserves_fractional_results_and_symbolic_requirements() {
    // Values4 function definitions independently determine these arithmetic
    // results; integer rounding belongs to computed/used values downstream.
    for (input, expected) in [
        ("calc(1.5)", "calc(1.5)"),
        ("calc(-1.5)", "calc(-1.5)"),
        ("calc(1 + 2)", "calc(3)"),
        ("min(3,2)", "calc(2)"),
        ("max(-1,2)", "calc(2)"),
        ("clamp(2,1,0)", "calc(2)"),
        ("round(2.5)", "calc(3)"),
        ("mod(-18,5)", "calc(2)"),
        ("rem(-18,5)", "calc(-3)"),
        ("sin(90deg)", "calc(1)"),
        ("cos(0)", "calc(1)"),
        ("tan(90deg)", "calc(infinity)"),
        ("pow(2,3)", "calc(8)"),
        ("sqrt(4)", "calc(2)"),
        ("hypot(3,4)", "calc(5)"),
        ("log(1,2)", "calc(0)"),
        ("exp(0)", "calc(1)"),
        ("abs(-2)", "calc(2)"),
        ("sign(-25%)", "calc(-1)"),
        ("calc(1in / 96px)", "calc(1)"),
        ("calc(asin(1) / 90deg)", "calc(1)"),
        ("calc(acos(1) / 1deg)", "calc(0)"),
        ("calc(atan(infinity) / 90deg)", "calc(1)"),
        ("calc(atan2(1,0) / 90deg)", "calc(1)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("sign(1em - 1px)", "sign(1em - 1px)"),
    ] {
        assert_output(&order(input), expected);
        let calculation =
            CssIntegerCalculation::try_from_components(parse_component_values(input).unwrap())
                .unwrap();
        assert_output(&CssIntegerValue::Calculation(calculation), expected);
    }
    let value = order("calc(1 + 2)");
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 100, 100),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 0, 100),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
}

#[test]
fn parsed_exact_payloads_retain_the_original_source_token() {
    let report = parse_style_attribute("order:+0002147483648;z-index:-2147483649");
    assert!(report.is_clean());
    assert_eq!(report.syntax().len(), 2);
    for (declaration, expected) in report
        .syntax()
        .iter()
        .zip(["+0002147483648", "-2147483649"])
    {
        let value = match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::Order(value) => value.value(),
            CssKnownPropertyValueRef::ZIndex(value) => match value.value() {
                CssZIndexValue::Integer(value) => value,
                _ => panic!("integer"),
            },
            _ => panic!("integer property"),
        };
        let CssIntegerValue::ExactLiteral(literal) = value else {
            panic!("exact")
        };
        assert_eq!(literal.numeric().representation(), expected);
        assert_eq!(
            literal.component(),
            &declaration.value_components().items()[0]
        );
        let CssValueOrigin::Parsed(origin) = literal.origin() else {
            panic!("parsed")
        };
        assert!(
            origin
                .source()
                .same_snapshot(declaration.parsed_value().unwrap().source())
        );
    }
}

#[test]
fn ordinary_huge_integers_stay_finite_while_calculation_leaves_follow_binary64() {
    let digits = "9".repeat(400);
    let components = parse_component_values(&digits).unwrap();
    let calculation = CssIntegerCalculation::try_from_components(components.clone()).unwrap();
    assert_output(
        &CssIntegerValue::Calculation(calculation.clone()),
        "calc(infinity)",
    );
    assert_eq!(calculation.components(), &components);
    assert_output(&exact(&digits), &digits);
    assert_output(&order("calc(1em / 1px)"), "calc(1em / 1px)");
}
