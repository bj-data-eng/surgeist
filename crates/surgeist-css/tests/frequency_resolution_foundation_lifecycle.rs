//! Ordinary resolution ranges and signed authored calculation/media domains.
use surgeist_css::*;

#[test]
fn ordinary_resolution_zero_is_valid() {
    // Values 4 (2024-03-12), #resolution, excludes negative values, not zero.
    assert!(CssResolution::try_new(0.0, CssResolutionUnit::Dpi).is_some());
}

#[test]
fn ordinary_resolution_preserves_positive_units_and_rejects_negative_nonfinite_values() {
    for unit in [
        CssResolutionUnit::Dpi,
        CssResolutionUnit::Dpcm,
        CssResolutionUnit::Dppx,
    ] {
        for number in [f32::MIN_POSITIVE, 0.5, 96.0, f32::MAX] {
            let resolution = CssResolution::try_new(number, unit).unwrap();
            assert_eq!(resolution.value().value(), number);
            assert_eq!(resolution.unit(), unit);
        }
        for number in [
            -f32::MIN_POSITIVE,
            -1.0,
            f32::MIN,
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            assert!(CssResolution::try_new(number, unit).is_none());
        }
    }
}

#[test]
fn frequency_calculation_convenience_preserves_signed_finite_units() {
    // Values 4 #frequency has no intrinsic nonnegative range.
    for (unit, suffix) in [
        (CssFrequencyUnit::Hertz, "hz"),
        (CssFrequencyUnit::Kilohertz, "khz"),
    ] {
        for (number, representation) in [(-2.5, "-2.5"), (0.0, "0"), (1.25, "1.25")] {
            let calculation = CssFrequencyCalculation::try_literal(number, unit).unwrap();
            assert_eq!(calculation.result_type(), CssCalculationType::Frequency);
            let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
                panic!("ordinary frequency leaf");
            };
            assert_eq!(value.literal().representation(), representation);
            assert_eq!(value.literal().unit(), Some(suffix));
            assert!(matches!(
                value.literal().origin(),
                CssValueOrigin::Programmatic
            ));
        }
        for number in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(CssFrequencyCalculation::try_literal(number, unit).is_none());
        }
    }
}

#[test]
fn signed_resolution_roots_retain_exact_coefficients_and_math_structure() {
    for (source, number, unit) in [
        ("-1e999dpi", "-1e999", "dpi"),
        ("-1e-999dpcm", "-1e-999", "dpcm"),
        ("-0e999dppx", "-0e999", "dppx"),
        ("1.0000000000000001x", "1.0000000000000001", "x"),
    ] {
        let calculation =
            CssResolutionCalculation::try_from_components(parse_component_values(source).unwrap())
                .unwrap();
        assert_eq!(calculation.result_type(), CssCalculationType::Resolution);
        let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
            panic!("ordinary signed resolution leaf");
        };
        assert_eq!(value.literal().representation(), number);
        assert_eq!(value.literal().unit(), Some(unit));
        assert_eq!(calculation.serialize().unwrap().as_css(), source);
    }
    for source in ["calc(-1dppx)", "calc(-1dppx / 0)"] {
        let calculation =
            CssResolutionCalculation::try_from_components(parse_component_values(source).unwrap())
                .unwrap();
        assert!(matches!(
            calculation.expression(),
            CssCalculationExpressionRef::NestedCalc(_)
        ));
        assert_eq!(calculation.serialize().unwrap().as_css(), source);
    }
}

fn resolution_feature(source: &str) -> CssMediaFeatureQuery {
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().serialize().unwrap().as_css(), source);
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition query");
    };
    let CssMediaConditionKind::Feature(feature @ CssMediaFeatureQuery::Resolution(_)) =
        condition.kind()
    else {
        panic!("known resolution feature");
    };
    feature.clone()
}

fn assert_media_number(value: &CssMediaResolution, number: &str, unit: &str) {
    let CssMediaResolutionRef::Numeric(calculation) = value.view() else {
        panic!("numeric resolution distinct from infinite");
    };
    let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
        panic!("ordinary signed resolution leaf");
    };
    assert_eq!(value.literal().representation(), number);
    assert_eq!(value.literal().unit(), Some(unit));
}

#[test]
fn media_resolution_preserves_signed_exact_aliases_and_parsed_origins() {
    // MQ4 #false-in-the-negative-range requires signed syntax to parse.
    for (source, number, unit, spelling) in [
        ("(resolution: -1e999dpi)", "-1e999", "dpi", "-1e999dpi"),
        (
            "(resolution: -1e-999dpcm)",
            "-1e-999",
            "dpcm",
            "-1e-999dpcm",
        ),
        ("(resolution: -0e999dppx)", "-0e999", "dppx", "-0e999dppx"),
        ("(resolution: 2x)", "2", "x", "2x"),
        ("(resolution: 2DPPX)", "2", "DPPX", "2DPPX"),
    ] {
        let CssMediaFeatureQuery::Resolution(range) = resolution_feature(source) else {
            unreachable!();
        };
        let CssMediaRangeRef::Plain { value } = range.view() else {
            panic!("plain resolution");
        };
        assert_media_number(value, number, unit);
        let CssMediaResolutionRef::Numeric(calculation) = value.view() else {
            unreachable!();
        };
        let CssValueOrigin::Parsed(origin) = calculation.origin() else {
            panic!("original parsed operand origin");
        };
        assert_eq!(origin.source().as_str(), source);
        let start = source.find(spelling).unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + spelling.len()
        );
        assert_eq!(calculation, &calculation.clone());
    }
}

#[test]
fn media_resolution_keeps_infinite_and_chained_source_order() {
    let CssMediaFeatureQuery::Resolution(range) =
        resolution_feature("(infinite >= resolution > -2x)")
    else {
        unreachable!();
    };
    let CssMediaRangeRef::Descending {
        left,
        left_inclusive,
        right,
        right_inclusive,
    } = range.view()
    else {
        panic!("descending authored bounds");
    };
    assert!(left_inclusive);
    assert!(!right_inclusive);
    assert!(matches!(left.view(), CssMediaResolutionRef::Infinite(_)));
    assert_media_number(right, "-2", "x");

    let CssMediaFeatureQuery::Resolution(range) =
        resolution_feature("(-3dpi < resolution <= 0dppx)")
    else {
        unreachable!();
    };
    let CssMediaRangeRef::Ascending {
        left,
        left_inclusive,
        right,
        right_inclusive,
    } = range.view()
    else {
        panic!("ascending authored bounds");
    };
    assert!(!left_inclusive);
    assert!(right_inclusive);
    assert_media_number(left, "-3", "dpi");
    assert_media_number(right, "0", "dppx");
}
