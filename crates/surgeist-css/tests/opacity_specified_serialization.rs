#![forbid(unsafe_code)]
//! Color 4 specified opacity formatting; exact dyadic goldens come from
//! (2^24-1)*2^104 and 5^149*10^-151, independently of the formatter.

use surgeist_css::*;

fn opacity(text: &str) -> CssOpacityValue {
    let report = parse_style_attribute(&format!("opacity:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Opacity(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("opacity")
    };
    value.value().clone()
}

fn assert_output(value: &CssOpacityValue, expected: &str) {
    let before = value.clone();
    let output = value.serialize_specified().unwrap();
    assert_eq!(output, expected);
    assert_eq!(*value, before);
    assert_eq!(opacity(&output).serialize_specified().unwrap(), expected);
    let exact = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len());
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    let short = CssSpecifiedValueSerializationLimits::new(65_536, 262_144, expected.len() - 1);
    assert_eq!(
        value
            .serialize_specified_with_limits(short)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(*value, before);
}

#[test]
fn authored_decimal_scalars_format_exactly_without_clamping() {
    for (input, output) in [
        (".1", "0.1"),
        ("1%", "0.01"),
        ("-25%", "-0.25"),
        ("150%", "1.5"),
        ("+000.1000e1", "1"),
        ("-0e999999999999999999999999", "0"),
        ("-2.5", "-2.5"),
    ] {
        assert_output(&opacity(input), output);
        let scalar =
            CssOpacityScalar::try_from_component(CssComponentValue::try_token(input).unwrap())
                .unwrap();
        assert_output(&CssOpacityValue::ExactScalar(scalar), output);
    }
    for percent in 0..=100 {
        let expected = if percent == 100 {
            "1".into()
        } else if percent == 0 {
            "0".into()
        } else {
            format!("0.{percent:02}").trim_end_matches('0').to_owned()
        };
        assert_output(&opacity(&format!("{percent}%")), &expected);
    }
    assert_output(&opacity("1e-47"), &format!("0.{}1", "0".repeat(46)));
    assert_output(&opacity("1e-47%"), &format!("0.{}1", "0".repeat(48)));
    assert_output(&opacity("1e100%"), &format!("1{}", "0".repeat(98)));
}

#[test]
fn constructed_binary32_magnitudes_are_not_replaced_by_shortest_approximations() {
    assert_output(
        &CssOpacityValue::Number(CssFiniteNumber::try_new(f32::MAX).unwrap()),
        "340282346638528859811704183484516925440",
    );
    assert_output(
        &CssOpacityValue::Percentage(CssFiniteNumber::try_new(f32::MAX).unwrap()),
        "3402823466385288598117041834845169254.4",
    );
    assert_output(
        &CssOpacityValue::Literal(CssOpacity::try_new(1.0 / 3.0).unwrap()),
        "0.3333333432674407958984375",
    );
    assert_output(&opacity("0.33333334"), "0.33333334");
    let expected = "0.0000000000000000000000000000000000000000000000140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125";
    for (value, expected) in [
        (f32::from_bits(1), expected.to_owned()),
        (-f32::from_bits(1), format!("-{expected}")),
    ] {
        assert_output(
            &CssOpacityValue::Percentage(CssFiniteNumber::try_new(value).unwrap()),
            &expected,
        );
    }
    for value in [
        CssOpacityValue::Literal(CssOpacity::try_new(-0.0).unwrap()),
        CssOpacityValue::Number(CssFiniteNumber::try_new(-0.0).unwrap()),
        CssOpacityValue::Percentage(CssFiniteNumber::try_new(-0.0).unwrap()),
    ] {
        assert_output(&value, "0");
    }
}

#[test]
fn scalar_limits_are_independent_atomic_and_precede_exponent_expansion() {
    let value = opacity(".5");
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 3),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 3),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 2),
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
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 3))
            .unwrap(),
        "0.5"
    );
    for text in [
        "1e999999999999999999999999999999999999999999999999999999",
        "1e-999999999999999999999999999999999999999999999999999999",
        "1e1000000000%",
    ] {
        let value = opacity(text);
        let before = value.clone();
        assert_eq!(
            value.serialize_specified().unwrap_err().kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn determinate_calculations_follow_specified_stage_not_computed_clamping() {
    for (input, expected) in [
        ("calc(1 / 2)", "calc(0.5)"),
        ("calc(25% + 25%)", "calc(50%)"),
        ("calc(2 - 3)", "calc(-1)"),
        ("min(25%,50%)", "calc(25%)"),
        ("max(-1,2)", "calc(2)"),
        ("clamp(2,1,0)", "calc(2)"),
        ("round(2.5)", "calc(3)"),
        ("round(-2.5)", "calc(-2)"),
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
        ("calc(1s / 1000ms)", "calc(1)"),
        ("calc(1in / 96px)", "calc(1)"),
        ("calc(asin(1) / 90deg)", "calc(1)"),
        ("calc(acos(1) / 1deg)", "calc(0)"),
        ("calc(atan(infinity) / 90deg)", "calc(1)"),
        ("calc(atan2(1,0) / 90deg)", "calc(1)"),
        ("calc(1 / 3)", "calc(0.3333333333333333)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("pow(NaN,0)", "calc(NaN)"),
        ("hypot(infinity,NaN)", "calc(NaN)"),
        ("sqrt(-1)", "calc(NaN)"),
        ("log(0)", "calc(-infinity)"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn percentage_subnormals_keep_exact_magnitude_across_former_underflow_boundary() {
    // 5^149 is an independent exact-integer oracle for n * 2^-149 / 100.
    let power = "140129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125";
    for n in [2u32, 3, 49, 50, 51, 99, 100, 101] {
        let mut carry = 0;
        let mut digits = Vec::new();
        for digit in power.bytes().rev() {
            let product = u32::from(digit - b'0') * n + carry;
            digits.push(char::from(b'0' + (product % 10) as u8));
            carry = product / 10;
        }
        while carry != 0 {
            digits.push(char::from(b'0' + (carry % 10) as u8));
            carry /= 10;
        }
        let coefficient: String = digits.into_iter().rev().collect();
        let expected = format!("0.{}{coefficient}", "0".repeat(151 - coefficient.len()))
            .trim_end_matches('0')
            .to_owned();
        for (value, expected) in [
            (f32::from_bits(n), expected.clone()),
            (-f32::from_bits(n), format!("-{expected}")),
        ] {
            assert_output(
                &CssOpacityValue::Percentage(CssFiniteNumber::try_new(value).unwrap()),
                &expected,
            );
        }
    }
}

#[test]
fn ieee_exception_rules_preserve_nested_zero_sign_and_nan() {
    for (input, expected) in [
        ("calc(1 / 0)", "calc(infinity)"),
        ("calc(infinity / infinity)", "calc(NaN)"),
        ("calc(infinity * 0)", "calc(NaN)"),
        ("calc(infinity + -infinity)", "calc(NaN)"),
        ("calc(infinity * 1%)", "calc(infinity * 1%)"),
        ("calc(NaN * 1%)", "calc(NaN * 1%)"),
        ("round(1,0)", "calc(NaN)"),
        ("mod(1,0)", "calc(NaN)"),
        ("rem(infinity,1)", "calc(NaN)"),
        ("mod(-1,infinity)", "calc(NaN)"),
        ("rem(-1,infinity)", "calc(-1)"),
        ("pow(-2,.5)", "calc(NaN)"),
        ("pow(0,-1)", "calc(infinity)"),
        ("pow(0 * -1,-3)", "calc(-infinity)"),
        ("calc(1 / sign(0 * -1))", "calc(-infinity)"),
        ("calc(1 / abs(0 * -1))", "calc(infinity)"),
        ("calc(1 / sqrt(0 * -1))", "calc(-infinity)"),
        ("calc(1 / min(0,0 * -1))", "calc(-infinity)"),
        ("calc(1 / max(0,0 * -1))", "calc(infinity)"),
        ("calc(1 / round(to-zero,-.5))", "calc(-infinity)"),
        ("calc(1 / round(-.5))", "calc(-infinity)"),
        ("exp(-infinity)", "calc(0)"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn contextual_magnitudes_remain_symbolic_without_assuming_a_nonzero_basis() {
    for (input, expected) in [
        ("sign(1em - 1px)", "sign(1em - 1px)"),
        ("calc(1em / 1px)", "calc(1em / 1px)"),
        ("calc(1em / 1em)", "calc(1em / 1em)"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn calculation_limits_are_atomic_and_not_source_coordinate_dependent() {
    let value = opacity("calc(1 + 2)");
    let before = value.clone();
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 100, 100),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 0, 100),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(100, 100, 6),
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
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(100, 100, 7))
            .unwrap(),
        "calc(3)"
    );
}

fn assert_math_scalar(input: &str, expected: &str) {
    assert_output(
        &opacity(input),
        &format!("calc({})", if expected == "-0" { "0" } else { expected }),
    );
    if matches!(expected, "0" | "-0") {
        assert_output(
            &opacity(&format!("calc(1 / {input})")),
            if expected == "-0" {
                "calc(-infinity)"
            } else {
                "calc(infinity)"
            },
        );
    }
}

#[test]
fn atan2_exception_table_preserves_each_quadrant_and_zero_sign() {
    // Selected Values 4 §10.4.1; normal finite/finite cells use a precision bound.
    let args = ["-infinity", "-1", "0 * -1", "0", "1", "infinity"];
    let expected = [
        ["-135", "-90", "-90", "-90", "-90", "-45"],
        ["-180", "-135", "-90", "-90", "-45", "-0"],
        ["-180", "-180", "-180", "-0", "-0", "-0"],
        ["180", "180", "180", "0", "0", "0"],
        ["180", "135", "90", "90", "45", "0"],
        ["135", "90", "90", "90", "90", "45"],
    ];
    for (y_index, y) in args.iter().enumerate() {
        for (x_index, x) in args.iter().enumerate() {
            let source = format!("calc(atan2({y},{x}) / 1deg)");
            if [1, 4].contains(&y_index) && [1, 4].contains(&x_index) {
                // These four cells are "normal" in the specification table.
                // Host transcendental last bits are not a portable contract.
                let output = opacity(&source).serialize_specified().unwrap();
                let actual = output
                    .strip_prefix("calc(")
                    .unwrap()
                    .strip_suffix(')')
                    .unwrap()
                    .parse::<f64>()
                    .unwrap();
                let expected = expected[y_index][x_index].parse::<f64>().unwrap();
                assert!((actual - expected).abs() <= 4.0 * f64::EPSILON * expected.abs());
                assert_eq!(opacity(&output).serialize_specified().unwrap(), output);
            } else {
                assert_math_scalar(&source, expected[y_index][x_index]);
            }
        }
    }
}

#[test]
fn pow_exception_tables_distinguish_zero_sign_parity_and_infinite_exponents() {
    // Selected Values 4 §10.5.1; no std::pow expectation is used.
    let bases = ["-infinity", "0 * -1", "0", "infinity"];
    for (exponent, expected) in [
        ("-3", ["-0", "-infinity", "infinity", "0"]),
        ("-2", ["0", "infinity", "infinity", "0"]),
        ("-.5", ["0", "infinity", "infinity", "0"]),
        ("0", ["1", "1", "1", "1"]),
        (".5", ["infinity", "0", "0", "infinity"]),
        ("2", ["infinity", "0", "0", "infinity"]),
        ("3", ["-infinity", "-0", "0", "infinity"]),
    ] {
        for (index, base) in bases.iter().enumerate() {
            assert_math_scalar(&format!("pow({base},{exponent})"), expected[index]);
        }
    }
    for (base, positive, negative) in [
        ("-2", "infinity", "0"),
        ("-1", "NaN", "NaN"),
        ("-.5", "0", "infinity"),
        ("0", "0", "infinity"),
        (".5", "0", "infinity"),
        ("1", "NaN", "NaN"),
        ("2", "infinity", "0"),
    ] {
        assert_math_scalar(&format!("pow({base},infinity)"), positive);
        assert_math_scalar(&format!("pow({base},-infinity)"), negative);
    }
}

#[test]
fn infinite_round_steps_follow_strategy_and_signed_zero_table() {
    // Selected Values 4 §10.3.1 applies to either sign of infinite step.
    for step in ["infinity", "-infinity"] {
        for (value, nearest, up, down) in [
            ("-1", "-0", "-0", "-infinity"),
            ("0 * -1", "-0", "-0", "-0"),
            ("0", "0", "0", "0"),
            ("1", "0", "infinity", "0"),
        ] {
            for (strategy, expected) in [
                ("nearest", nearest),
                ("to-zero", nearest),
                ("up", up),
                ("down", down),
            ] {
                assert_math_scalar(&format!("round({strategy},{value},{step})"), expected);
            }
        }
    }
}

#[test]
fn intermediate_dimension_products_resolve_before_dimension_cancellation() {
    for (input, expected) in [
        ("calc((2px * 3px) / (2px * 3px))", "calc(1)"),
        ("calc(1 / (2px * 3px) * 6px * 1px)", "calc(1)"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn checked_calculation_construction_and_deep_projection_share_specified_semantics() {
    for source in ["calc(1 / 2)", "min(25%,50%)", "sign(1em - 1px)"] {
        let components = parse_component_values(source).unwrap();
        let typed = if source.contains('%') {
            CssOpacityValue::PercentageCalculation(
                CssPercentageCalculation::try_from_components(components).unwrap(),
            )
        } else {
            CssOpacityValue::Calculation(
                CssNumberCalculation::try_from_components(components).unwrap(),
            )
        };
        assert_eq!(
            typed.serialize_specified().unwrap(),
            opacity(source).serialize_specified().unwrap()
        );
    }
    // Existing public numeric construction supports 256 nested functions.
    let source = format!("{}1{}", "calc(1 + 0 * ".repeat(256), ")".repeat(256));
    let calculation =
        CssNumberCalculation::try_from_components(parse_component_values(&source).unwrap())
            .unwrap();
    let original = calculation.clone();
    let value = CssOpacityValue::Calculation(calculation);
    assert_output(&value, "calc(1)");
    let CssOpacityValue::Calculation(calculation) = value else {
        unreachable!()
    };
    assert_eq!(calculation, original);
}

#[test]
fn flat_symbolic_graph_and_repeated_distribution_obey_projection_limits() {
    let term = "sign(2 * (1em - 1px))";
    let source = format!(
        "calc({})",
        std::iter::repeat_n(term, 128)
            .collect::<Vec<_>>()
            .join(" + ")
    );
    let value = opacity(&source);
    let before = value.clone();
    let error = value
        .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            65_536, 32, 1_048_576,
        ))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(value, before);
    let expected = format!(
        "calc({})",
        std::iter::repeat_n("sign(2em - 2px)", 128)
            .collect::<Vec<_>>()
            .join(" + ")
    );
    assert_output(&value, &expected);
}

#[test]
fn arithmetic_precision_handles_lexical_overflow_underflow_and_stable_hypot() {
    // Decimal inputs and the scaled hypot operations are rounded binary64.
    // Require stable finite evaluation near the mathematical result, not an
    // exact integer that the declared arithmetic precision does not promise.
    let output = opacity("calc(hypot(3e200,4e200) / 1e200)")
        .serialize_specified()
        .unwrap();
    let number = output
        .strip_prefix("calc(")
        .unwrap()
        .strip_suffix(')')
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(number.is_finite() && (number - 5.0).abs() <= 4.0 * f64::EPSILON * 5.0);
    assert_eq!(opacity(&output).serialize_specified().unwrap(), output);
    for (input, expected) in [
        ("calc(1e999)", "calc(infinity)"),
        ("calc(-1e999)", "calc(-infinity)"),
        ("calc(1 / -1e-999)", "calc(-infinity)"),
        ("calc(1 / -0e999)", "calc(infinity)"),
        (
            "round(1e300,1e-300)",
            &format!("calc(1{})", "0".repeat(300)),
        ),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn exceptional_arithmetic_and_unbounded_clamp_reduce_with_symbolic_children() {
    for (input, expected) in [
        ("calc(NaN + sign(1em - 1px))", "calc(NaN)"),
        ("calc(NaN * sign(1em - 1px))", "calc(NaN)"),
        ("clamp(none, sign(1em - 1px), none)", "sign(1em - 1px)"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn partial_comparisons_preserve_first_numeric_and_symbolic_argument_positions() {
    for (input, expected) in [
        ("sign(min(1px, 1em))", "sign(min(1px, 1em))"),
        ("sign(min(1px, 1em - 1px))", "sign(min(1px, 1em - 1px))"),
        ("sign(max(1px, 1em, 2px))", "sign(max(2px, 1em))"),
        ("sign(min(3em, 2px, 1em, 1px))", "sign(min(1em, 1px))"),
    ] {
        assert_output(&opacity(input), expected);
    }
}

#[test]
fn representational_capacity_overflow_is_typed_before_allocation() {
    let value = opacity(&format!("1e{}", isize::MAX));
    let before = value.clone();
    let limits = CssSpecifiedValueSerializationLimits::new(1, 1, usize::MAX);
    assert_eq!(
        value
            .serialize_specified_with_limits(limits)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::CapacityOverflow
    );
    assert_eq!(value, before);
}
