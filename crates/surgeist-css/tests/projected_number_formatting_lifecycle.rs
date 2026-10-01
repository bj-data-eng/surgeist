#![forbid(unsafe_code)]
//! CSSOM WD 2021-08-26 §6.7.2 and component number serialization require
//! selection before lossy six-place fixed text. Values 4 WD 2024-03-12
//! §10.13 retains specified calculation trees and normal numeric leaf text.
//! Generic decimal ties follow frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d:
//! dtoa.cpp:130–146 calls ToFixedUncapped(d, 6); double-conversion.h:327–335
//! defines FIXED ties away from zero. Expected numbers below come from exact
//! integer/rational binary64 values, never the formatter's shortest spelling.
//! Color captures preserve their separately owned text and scratch contracts.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn components(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}
fn component(source: &str) -> CssComponentValue {
    let values = components(source);
    assert_eq!(values.items().len(), 1, "{source}");
    values.items()[0].clone()
}
fn number(source: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(components(source)).unwrap(),
    )
    .unwrap()
}
fn length(source: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(components(source)).unwrap(),
    )
    .unwrap()
}
fn padding(source: &str) -> CssScrollPaddingValue {
    CssScrollPaddingValue::LengthPercentage(nonnegative_lp(source))
}
fn nonnegative_lp(source: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(components(source)).unwrap(),
    )
    .unwrap()
}
fn slice(sources: &[&str]) -> CssBorderImageSlice {
    CssBorderImageSlice::try_new(
        sources
            .iter()
            .map(|source| {
                CssBorderImageSliceComponent::Number(
                    CssSpecifiedNonNegativeNumber::try_from_calculation(
                        CssNumberCalculation::try_from_components(components(source)).unwrap(),
                    )
                    .unwrap(),
                )
            })
            .collect(),
        false,
    )
    .unwrap()
}
fn declaration(property: CssKnownProperty, source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components(source),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{property:?}: {source}: {error:?}"))
}
fn color(source: &str) -> CssColor {
    let value = declaration(CssKnownProperty::Color, source);
    let CssKnownPropertyValueRef::Color(value) = value.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value().clone()
}
fn assert_number(source: &str, expected: &str) {
    let value = number(source);
    let before = value.clone();
    let calculation = value.calculation().expect("actual calculation");
    assert_eq!(calculation.serialize().unwrap().as_css(), source);
    assert_eq!(calculation.components(), &components(source));
    let origin = value.origin().clone();
    let result = value.serialize_specified();
    assert_eq!(value, before);
    assert_eq!(value.origin(), &origin);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn thirds_use_six_fractional_places_in_actual_calculations() {
    assert_number("calc(1 / 3)", "calc(0.333333)");
    assert_number("calc(-1 / 3)", "calc(-0.333333)");
}

#[test]
fn exact_positive_dyadic_halfway_rounds_away_from_zero() {
    // 1/128 * 10^6 = 7812 + 1/2 exactly.
    assert_number("calc(1 / 128)", "calc(0.007813)");
}

#[test]
fn exact_negative_dyadic_halfway_rounds_away_from_zero() {
    assert_number("calc(-1 / 128)", "calc(-0.007813)");
}

#[test]
fn neighboring_binary_values_straddle_dyadic_halfway() {
    for (decimal, bits, expected) in [
        ("0.007812499999999999", 0x3f7fffffffffffff, "calc(0.007812)"),
        ("0.0078125", 0x3f80000000000000, "calc(0.007813)"),
        ("0.007812500000000002", 0x3f80000000000001, "calc(0.007813)"),
    ] {
        // Fixture identity only; the expected rounding is rational, not f64 text.
        assert_eq!(decimal.parse::<f64>().unwrap().to_bits(), bits);
        assert_number(&format!("calc({decimal})"), expected);
        assert_number(
            &format!("calc(-{decimal})"),
            &expected.replace("calc(0.", "calc(-0."),
        );
    }
}

#[test]
fn shortest_decimal_halfway_is_below_halfway_in_the_actual_binary_value() {
    // 5e-7 = 4722366482869645 / 9444732965739290427392 < 1/2000000.
    assert_number("calc(5e-7)", "calc(0)");
    assert_number("calc(-5e-7)", "calc(0)");
    assert_number("calc(5.000000000000001e-7)", "calc(0.000001)");
    assert_number("calc(4.999999999999999e-7)", "calc(0)");
}

#[test]
fn rounded_zero_and_carry_use_actual_short_output() {
    for (source, expected) in [
        ("calc(-0.0000004)", "calc(0)"),
        ("calc(0.9999996)", "calc(1)"),
        ("calc(-0.9999996)", "calc(-1)"),
    ] {
        assert_number(source, expected);
        let value = number(source);
        // Calc wrapper + leaf: two inputs, one projection, no formatter visits.
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(2, 1, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(2, 1, expected.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
    }
}

#[test]
fn finite_extremes_have_fixed_text_from_exact_binary_values() {
    // Subnormal 2^-1074 and minimum normal 2^-1022 are below 1/2000000.
    for source in [
        "calc(5e-324)",
        "calc(-5e-324)",
        "calc(2.2250738585072014e-308)",
    ] {
        assert_number(source, "calc(0)");
    }
}

#[test]
fn large_integral_binary_value_emits_all_actual_integer_digits() {
    // Nearest binary64 is 7812500000000001 * 2^7, not decimal ...0100.
    assert_eq!(7812500000000001_u128 * 128, 1000000000000000128);
    assert_number("calc(1000000000000000100)", "calc(1000000000000000128)");
}

#[test]
fn maximum_finite_binary_value_emits_its_exact_309_digit_integer() {
    // (2^53 - 1) * 2^971, independently expanded with integer arithmetic.
    const INTEGER: &str = concat!(
        "179769313486231570814527423731704356798070567525844996598917476803157260",
        "780028538760589558632766878171540458953514382464234321326889464182768467",
        "546703537516986049910576551282076245490090389328944075868508455133942304",
        "583236903222948165808559332123348274797826204144723168738177180919299881",
        "250404026184124858368"
    );
    assert_eq!(INTEGER.len(), 309);
    assert_number("calc(1.7976931348623157e308)", &format!("calc({INTEGER})"));
}

#[test]
fn generic_byte_limits_keep_tree_visits_failure_order_and_input_identity() {
    let value = number("calc(1 / 3)");
    let before = value.clone();
    // Four inputs: Calc, product, two leaves. Four projections: leaves,
    // inverse, resolved product. Text calc(0.333333) has fourteen bytes.
    for (limits, kind) in [
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(3, 4, 14), Kind::InputNodeLimit),
        (Limits::new(4, 3, 14), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, 13), Kind::ByteLimit),
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
            .serialize_specified_with_limits(Limits::new(4, 4, 14))
            .unwrap(),
        "calc(0.333333)"
    );
}

#[test]
fn ordinary_roots_retain_exact_decimal_admission_and_authored_identity() {
    let raw = component("5e-7");
    let ordinary = CssSpecifiedNumber::try_from_component(raw.clone()).unwrap();
    assert_eq!(ordinary.serialize_specified().unwrap(), "0.000001");
    let normalized = number("5e-7");
    assert!(normalized.calculation().is_none());
    assert_eq!(normalized.literal_component(), Some(&raw));
    assert_eq!(normalized.serialize_specified().unwrap(), "0.000001");
    for source in ["-1e-999px", "-0.0000004px"] {
        let raw = component(source);
        let error = CssSpecifiedNonNegativeLength::try_from_component(raw.clone()).unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(raw.origin()));
    }
    assert_ne!(number("calc(1 / 2)"), number("calc(.5)"));
}

#[test]
fn original_recovery_and_substitution_errors_precede_projection() {
    for (source, kind) in [
        (
            "calc(1",
            CssNumericConstructionErrorKind::RecoveredComponent,
        ),
        (
            "calc(var(--n))",
            CssNumericConstructionErrorKind::SubstitutionRequired,
        ),
    ] {
        let original = components(source);
        let before = original.clone();
        let error = CssNumberCalculation::try_from_components(original.clone()).unwrap_err();
        assert_eq!(error.kind(), &kind);
        assert!(error.origin().is_some());
        assert_eq!(original, before);
    }
}

#[test]
fn integer_calculation_text_rounding_is_distinct_from_integer_arithmetic() {
    for (source, expected) in [
        ("calc(-1 / 128)", "calc(-0.007813)"),
        ("calc(1000000000000000100)", "calc(1000000000000000128)"),
        ("round(-2.5)", "calc(-2)"),
        ("sign(1em - 1px)", "sign(1em - 1px)"),
    ] {
        let calculation = CssIntegerCalculation::try_from_components(components(source)).unwrap();
        let value = CssIntegerValue::Calculation(calculation.clone());
        assert_eq!(value.serialize_specified().unwrap(), expected, "{source}");
        assert_eq!(calculation.serialize().unwrap().as_css(), source);
    }
    let literal = CssIntegerValue::Literal(
        CssIntegerLiteral::try_from_component(component("1000000000000000100")).unwrap(),
    );
    assert_eq!(
        literal.serialize_specified().unwrap(),
        "1000000000000000100"
    );
}

#[test]
fn nonfinite_roots_and_actual_negative_zero_keep_their_math_representation() {
    for (source, expected) in [
        ("calc(0 * -1)", "calc(0)"),
        ("calc(1 / (0 * -1))", "calc(-infinity)"),
        ("calc(0 / 0)", "calc(NaN)"),
        ("calc(infinity)", "calc(infinity)"),
    ] {
        assert_number(source, expected);
    }
    for (source, expected) in [
        ("calc(infinity * 1px)", "calc(infinity * 1px)"),
        ("calc(0px * -1 + 1em)", "calc(1em + (0px * -1))"),
    ] {
        assert_eq!(
            length(source).serialize_specified().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn symbolic_terms_keep_order_sign_and_units_while_finite_children_round() {
    for (source, expected) in [
        ("calc(1em + 0.12345641px)", "calc(1em + 0.123456px)"),
        ("calc(1em - 0.0078125px)", "calc(1em - 0.007813px)"),
        ("calc(1em - 0.0000004px)", "calc(1em - 0px)"),
    ] {
        let value = length(source);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{source}");
        assert_eq!(value, before);
        assert_eq!(
            value.calculation().unwrap().serialize().unwrap().as_css(),
            source
        );
    }
}

#[test]
fn dimensional_calculation_providers_convert_units_before_rounding_text() {
    let angle = CssAngleValue::try_from_calculation(
        CssAngleCalculation::try_from_components(components("calc(0.0078125turn)")).unwrap(),
    )
    .unwrap();
    let angle_declaration =
        declaration(CssKnownProperty::Filter, "hue-rotate(calc(0.0078125turn))");
    assert_eq!(
        composed_text(&angle_declaration),
        "hue-rotate(calc(2.8125deg))"
    );
    assert_eq!(
        angle.calculation().unwrap().serialize().unwrap().as_css(),
        "calc(0.0078125turn)"
    );
    let time = CssTimeValue::try_from_calculation(
        CssTimeCalculation::try_from_components(components("calc(123.45641ms)")).unwrap(),
    )
    .unwrap();
    assert_eq!(time.serialize_specified().unwrap(), "calc(0.123456s)");
    let frequency = CssFrequencyValue::try_from_calculation(
        CssFrequencyCalculation::try_from_components(components("calc(0.00012345641khz)")).unwrap(),
    )
    .unwrap();
    assert_eq!(frequency.serialize_specified().unwrap(), "calc(0.123456hz)");
    let percentage = CssSpecifiedPercentage::try_from_calculation(
        CssPercentageCalculation::try_from_components(components("calc(1% / 128)")).unwrap(),
    )
    .unwrap();
    assert_eq!(percentage.serialize_specified().unwrap(), "calc(0.007813%)");
}

#[test]
fn opacity_calculations_use_generic_emission_without_computed_clamping() {
    for (source, expected) in [
        ("calc(1 / 3)", "calc(0.333333)"),
        ("calc(-1 / 128)", "calc(-0.007813)"),
        ("calc(2)", "calc(2)"),
    ] {
        let declaration = declaration(CssKnownProperty::Opacity, source);
        let CssKnownPropertyValueRef::Opacity(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("opacity")
        };
        assert_eq!(value.value().serialize_specified().unwrap(), expected);
    }
}

const COLLIDING: [&str; 4] = [
    "calc(0.12345641)",
    "calc(0.12345642)",
    "calc(0.12345643)",
    "calc(0.12345644)",
];
const FOUR_NUMBERS: &str = "calc(0.123456) calc(0.123456) calc(0.123456) calc(0.123456)";

#[test]
fn unequal_projected_border_slice_components_survive_identical_rounded_text() {
    let value = slice(&COLLIDING);
    let before = value.clone();
    assert_eq!(value.serialize_specified().unwrap(), FOUR_NUMBERS);
    assert_eq!(value, before);
}

#[test]
fn border_collision_limits_charge_all_four_captures_before_selection() {
    let value = slice(&COLLIDING);
    let before = value.clone();
    // One group + four edge wrappers; each Calc has two inputs/one projection.
    assert_eq!(FOUR_NUMBERS.len(), 59);
    for (limits, kind) in [
        (Limits::new(12, 9, 59), Kind::InputNodeLimit),
        (Limits::new(13, 8, 59), Kind::ProjectionNodeLimit),
        (Limits::new(13, 9, 58), Kind::ByteLimit),
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
            .serialize_specified_with_limits(Limits::new(13, 9, 59))
            .unwrap(),
        FOUR_NUMBERS
    );
}

#[test]
fn equal_projected_border_edges_keep_canonical_compression_and_scratch_limits() {
    let value = slice(&["calc(1 / 2)", "calc(0.5)", "calc(2 / 4)", "calc(0.50)"]);
    // Five wrappers; trees contribute twelve inputs/ten projections.
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(17, 15, 9))
            .unwrap(),
        "calc(0.5)"
    );
    for (limits, kind) in [
        (Limits::new(16, 15, 9), Kind::InputNodeLimit),
        (Limits::new(17, 14, 9), Kind::ProjectionNodeLimit),
        (Limits::new(17, 15, 8), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    // A captured scalar tree needs fourteen bytes even when selected to one edge.
    let equal = slice(&["calc(0.12345641)"]);
    assert_eq!(
        equal
            .serialize_specified_with_limits(Limits::new(13, 9, 14))
            .unwrap(),
        "calc(0.123456)"
    );
    assert_eq!(
        equal
            .serialize_specified_with_limits(Limits::new(13, 9, 13))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
}

#[test]
fn border_width_outset_and_full_shorthand_preserve_colliding_edges() {
    let width = CssBorderImageWidth::try_new(
        COLLIDING
            .iter()
            .map(|s| {
                CssBorderImageWidthComponent::Number(
                    CssSpecifiedNonNegativeNumber::try_from_calculation(
                        CssNumberCalculation::try_from_components(components(s)).unwrap(),
                    )
                    .unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(width.serialize_specified().unwrap(), FOUR_NUMBERS);
    let outset = CssBorderImageOutset::try_new(
        COLLIDING
            .iter()
            .map(|s| {
                CssBorderImageOutsetComponent::Number(
                    CssSpecifiedNonNegativeNumber::try_from_calculation(
                        CssNumberCalculation::try_from_components(components(s)).unwrap(),
                    )
                    .unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(outset.serialize_specified().unwrap(), FOUR_NUMBERS);
    let image = CssBorderImage::try_new(
        None,
        Some(slice(&COLLIDING)),
        Some(width),
        Some(outset),
        None,
    )
    .unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        format!("{FOUR_NUMBERS} / {FOUR_NUMBERS} / {FOUR_NUMBERS}")
    );
}

#[test]
fn contextual_border_components_compare_unrounded_coefficients_and_canonical_syntax() {
    let make = |sources: &[&str]| {
        CssBorderImageWidth::try_new(
            sources
                .iter()
                .map(|s| CssBorderImageWidthComponent::LengthPercentage(nonnegative_lp(s)))
                .collect(),
        )
        .unwrap()
    };
    let equal = make(&[
        "calc(1em + 2px)",
        "calc(2px + 1em)",
        "calc(1.0em + 2.0px)",
        "calc(2px + 1em)",
    ]);
    assert_eq!(
        equal
            .serialize_specified_with_limits(Limits::new(21, 25, 15))
            .unwrap(),
        "calc(1em + 2px)"
    );
    let unequal = make(&["calc(1em + 0.12345641px)", "calc(0.12345642px + 1em)"]);
    assert_eq!(
        unequal.serialize_specified().unwrap(),
        "calc(1em + 0.123456px) calc(1em + 0.123456px)"
    );
    let units = make(&["calc(0.0000001em)", "calc(0.0000001px)"]);
    assert_eq!(units.serialize_specified().unwrap(), "calc(0em) calc(0px)");
}

#[test]
fn scroll_pairs_preserve_collision_omission_auto_and_signed_length_policies() {
    let pair = CssScrollMarginPair::new(
        length("calc(-0.12345641px)"),
        Some(length("calc(-0.12345642px)")),
    );
    let before = pair.clone();
    assert_eq!(
        pair.serialize_specified().unwrap(),
        "calc(-0.123456px) calc(-0.123456px)"
    );
    assert_eq!(pair, before);
    let pad = CssScrollPaddingPair::new(
        padding("calc(0.12345641%)"),
        Some(padding("calc(0.12345642%)")),
    );
    assert_eq!(
        pad.serialize_specified().unwrap(),
        "calc(0.123456%) calc(0.123456%)"
    );
    let absent = CssScrollMarginPair::new(length("calc(0.12345641px)"), None);
    assert!(absent.authored_end().is_none());
    assert_eq!(absent.serialize_specified().unwrap(), "calc(0.123456px)");
    let auto = CssScrollPaddingPair::new(
        CssScrollPaddingValue::Auto,
        Some(padding("calc(0.0000001px)")),
    );
    assert_eq!(auto.serialize_specified().unwrap(), "auto calc(0px)");
}

#[test]
fn scroll_authored_roles_keep_marker_arity_and_exact_cumulative_capture_visits() {
    let sources = [
        "calc(0.12345641px)",
        "calc(0.12345642px)",
        "calc(0.12345643px)",
        "calc(0.12345644px)",
    ];
    for kind in [CssScrollSideKind::Physical, CssScrollSideKind::Logical] {
        for count in 1..=4 {
            let authored: Vec<_> = sources[..count].iter().map(|s| length(s)).collect();
            let value = CssScrollMarginShorthand::try_new(kind, authored.clone()).unwrap();
            assert_eq!(value.authored_values(), authored);
            let indices = match count {
                1 => [0, 0, 0, 0],
                2 => [0, 1, 0, 1],
                3 => [0, 1, 2, 1],
                _ => [0, 1, 2, 3],
            };
            for (role, index) in indices.into_iter().enumerate() {
                assert_eq!(value.role(role), Some(&authored[index]));
            }
            let marker = usize::from(kind == CssScrollSideKind::Logical);
            let expected = format!(
                "{}{}",
                if marker == 1 { "logical " } else { "" },
                vec!["calc(0.123456px)"; count].join(" ")
            );
            // No physical aggregate charge; each authored Calc costs 2/1.
            assert_eq!(
                value
                    .serialize_specified_with_limits(Limits::new(
                        2 * count + marker,
                        count + marker,
                        expected.len()
                    ))
                    .unwrap(),
                expected
            );
            for (limits, error) in [
                (
                    Limits::new(2 * count + marker - 1, count + marker, expected.len()),
                    Kind::InputNodeLimit,
                ),
                (
                    Limits::new(2 * count + marker, count + marker - 1, expected.len()),
                    Kind::ProjectionNodeLimit,
                ),
                (
                    Limits::new(2 * count + marker, count + marker, expected.len() - 1),
                    Kind::ByteLimit,
                ),
            ] {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    error
                );
            }
            let padded = CssScrollPaddingShorthand::try_new(
                kind,
                sources[..count].iter().map(|s| padding(s)).collect(),
            )
            .unwrap();
            assert_eq!(padded.serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn canonical_scroll_equivalence_keeps_authored_capture_work_and_order() {
    let pair = CssScrollMarginPair::new(length("calc(1in)"), Some(length("calc(96px)")));
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(4, 2, 10))
            .unwrap(),
        "calc(96px)"
    );
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(3, 2, 10))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    let symbolic =
        CssScrollMarginPair::new(length("calc(1em + 2px)"), Some(length("calc(2px + 1em)")));
    assert_eq!(
        symbolic
            .serialize_specified_with_limits(Limits::new(8, 10, 15))
            .unwrap(),
        "calc(1em + 2px)"
    );
    // Eager capture: a failed first scratch precedes work on the next sibling.
    let first = CssScrollMarginPair::new(length("calc(1em + 2px)"), Some(length("calc(1px)")));
    let second = CssScrollMarginPair::new(length("calc(1px)"), Some(length("calc(1em + 2px)")));
    assert_eq!(
        first
            .serialize_specified_with_limits(Limits::new(4, 5, 14))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        second
            .serialize_specified_with_limits(Limits::new(4, 5, 14))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
}

#[test]
fn tiny_nonzero_math_does_not_compare_equal_to_actual_zero() {
    let pair = CssScrollMarginPair::new(length("calc(0px)"), Some(length("calc(0.0000001px)")));
    assert_eq!(pair.serialize_specified().unwrap(), "calc(0px) calc(0px)");
    let negative = CssScrollMarginPair::new(length("calc(0px * -1)"), Some(length("calc(0px)")));
    assert_eq!(negative.serialize_specified().unwrap(), "calc(0px)");
}

#[test]
fn calc_size_and_fit_content_round_shared_children_without_erasing_symbols() {
    let source = "calc-size(min-content, size + 0.12345641px)";
    let value = CssCalcSize::try_from_component(component(source)).unwrap();
    let before = value.clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc-size(min-content, 0.123456px + size)"
    );
    assert_eq!(value, before);
    let fit = CssBoxSize::FitContentFunction(nonnegative_lp("calc(0.12345641px)"));
    let expected = "fit-content(calc(0.123456px))";
    assert_eq!(
        fit.serialize_specified_with_limits(Limits::new(3, 2, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        fit.serialize_specified_with_limits(Limits::new(3, 2, expected.len() - 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
}

fn composed_text(value: &CssDeclaration) -> String {
    match value.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::BackgroundPosition(v) => {
            v.positions().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::BackgroundImage(v) => v.images().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::ClipPath(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Filter(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::GridTemplateColumns(v) => {
            v.value().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::FontWeight(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::FontStyle(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::AspectRatio(v) => v.ratio().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::TextCombineUpright(v) => {
            v.combine().serialize_specified().unwrap()
        }
        _ => panic!("fixture with existing specified emitter"),
    }
}

#[test]
fn representative_composed_writers_use_generic_projected_children() {
    for (property, source, expected) in [
        (
            CssKnownProperty::BackgroundPosition,
            "calc(0.12345641px) calc(0.12345642%)",
            "calc(0.123456px) calc(0.123456%)",
        ),
        (
            CssKnownProperty::BackgroundImage,
            "linear-gradient(calc(180.0000004deg), red calc(0.12345641%), blue)",
            "linear-gradient(calc(180deg), red calc(0.123456%), blue)",
        ),
        (
            CssKnownProperty::ClipPath,
            "xywh(calc(0.12345641px) 0px 1px 2px)",
            "xywh(calc(0.123456px) 0px 1px 2px)",
        ),
        (
            CssKnownProperty::Filter,
            "blur(calc(0.9999996px)) opacity(calc(1 / 3))",
            "blur(calc(1px)) opacity(calc(0.333333))",
        ),
        (
            CssKnownProperty::GridTemplateColumns,
            "calc(0.12345641fr)",
            "calc(0.123456fr)",
        ),
        (
            CssKnownProperty::FontWeight,
            "calc(725.12345641)",
            "calc(725.123456)",
        ),
        (
            CssKnownProperty::FontStyle,
            "oblique calc(14.12345641deg)",
            "oblique calc(14.123456deg)",
        ),
        (
            CssKnownProperty::AspectRatio,
            "calc(1 / 3) / calc(0.12345641)",
            "calc(0.333333) / calc(0.123456)",
        ),
        (
            CssKnownProperty::TextCombineUpright,
            "digits calc(2.12345641)",
            "digits calc(2.123456)",
        ),
    ] {
        let value = declaration(property, source);
        let before = value.clone();
        assert_eq!(
            value.value_components().serialize().unwrap().as_css(),
            source
        );
        assert_eq!(composed_text(&value), expected, "{property:?}: {source}");
        assert_eq!(value, before);
    }
}

#[test]
fn existing_timing_children_round_without_inventing_an_aggregate_writer() {
    let source = declaration(
        CssKnownProperty::TransitionDuration,
        "calc(123.45641ms),calc(1 / 128 * 1s)",
    );
    let CssKnownPropertyValueRef::TransitionDuration(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("duration")
    };
    assert_eq!(
        value.durations().values()[0].serialize_specified().unwrap(),
        "calc(0.123456s)"
    );
    assert_eq!(
        value.durations().values()[1].serialize_specified().unwrap(),
        "calc(0.007813s)"
    );
    let source = declaration(
        CssKnownProperty::TransitionTimingFunction,
        "cubic-bezier(calc(1 / 128),calc(-1 / 128),calc(0.9999996),calc(1 / 3))",
    );
    let CssKnownPropertyValueRef::TransitionTimingFunction(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("easing")
    };
    let CssEasing::CubicBezier(value) = &value.timing_functions().values()[0] else {
        panic!("bezier")
    };
    assert_eq!(
        value.x1().value().serialize_specified().unwrap(),
        "calc(0.007813)"
    );
    assert_eq!(value.y1().serialize_specified().unwrap(), "calc(-0.007813)");
}

#[test]
fn color_calculated_alpha_preserves_exact_dyadic_text_and_unclamped_values() {
    for (source, expected) in [
        (
            "rgb(1 2 3 / calc(1 / 128))",
            "rgba(1, 2, 3, calc(0.0078125))",
        ),
        (
            "rgb(1 2 3 / calc(-1 / 128))",
            "rgba(1, 2, 3, calc(-0.0078125))",
        ),
        ("rgb(1 2 3 / calc(2))", "rgba(1, 2, 3, calc(2))"),
        (
            "alpha(from red / calc(0.78125%))",
            "alpha(from red / calc(0.0078125))",
        ),
    ] {
        let value = color(source);
        let before = value.clone();
        assert_eq!(value.to_specified_css().unwrap(), expected, "{source}");
        assert_eq!(
            value
                .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .to_specified_css_with_limits(Limits::new(
                    usize::MAX,
                    usize::MAX,
                    expected.len() - 1
                ))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, before);
    }
}

#[test]
fn color_channel_origin_relative_profile_and_mix_captures_keep_their_text() {
    for (source, expected) in [
        (
            "color(srgb calc(1 / 128) 0 0)",
            "color(srgb calc(0.0078125) 0 0)",
        ),
        (
            "color(srgb calc(0.78125%) 0 0)",
            "color(srgb calc(0.0078125) 0 0)",
        ),
        (
            "rgb(from color(srgb calc(1 / 128) 0 0) r g b)",
            "rgb(from color(srgb calc(0.0078125) 0 0) r g b)",
        ),
        (
            "hsl(none calc(0.0078125) 50%)",
            "hsl(none calc(0.0078125%) 50%)",
        ),
        (
            "hsl(calc(1em / 1px + 0.0078125deg / 1deg) 50% 50%)",
            "hsl(calc(0.0078125 + (1em / 1px)) 50% 50%)",
        ),
        (
            "rgb(from red calc(r + 0.0078125) g b)",
            "rgb(from red calc(0.0078125 + r) g b)",
        ),
        (
            "color(from red --P calc(Cyan + 0.0078125) Magenta)",
            "color(from red --P calc(0.0078125 + Cyan) Magenta)",
        ),
        (
            "color-mix(in srgb, red calc(0.78125%), blue)",
            "color-mix(in srgb, red calc(0.78125%), blue)",
        ),
    ] {
        let value = color(source);
        let before = value.clone();
        assert_eq!(value.to_specified_css().unwrap(), expected, "{source}");
        assert_eq!(value, before);
    }
}

#[test]
fn discarded_color_channel_capture_keeps_its_unrounded_scratch_budget() {
    let value = color("rgb(calc(10000000000) 0 0)");
    let before = value.clone();
    // Capture occurs before final RGB output. calc(10000000000) needs seventeen
    // scratch bytes although the clamped final rgb(255, 0, 0) needs fourteen.
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 17))
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 16))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
}
