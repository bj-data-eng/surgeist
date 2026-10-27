//! Ordinary resolution ranges and signed authored calculation/media domains.
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::value_assertions::{assert_numeric_kind, literal_specified};
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueError, CssComponentValueLimits, CssComponentValueRef,
    CssComponentValues, CssNumericTokenKind, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::numeric::{
    CssCalculation, CssCalculationExpressionRef, CssCalculationType, CssFrequency,
    CssFrequencyUnit, CssNonNegative, CssNumeric, CssNumericLiteral, CssResolution,
    CssResolutionUnit,
};
use surgeist_css::query::CssConditionKind;
use surgeist_css::query::media::{
    CssMediaFeatureQuery, CssMediaLeaf, CssMediaQuery, CssMediaRangeRef, CssMediaResolution,
    CssMediaResolutionRef,
};
use surgeist_css::serialization::CssSerializationLimits;

#[test]
fn ordinary_resolution_zero_is_valid() {
    // Values 4 (2024-03-12), #resolution, excludes negative values, not zero.
    assert!(CssNumericLiteral::<CssResolution>::try_new("0", CssResolutionUnit::Dpi).is_ok());
}

#[test]
fn ordinary_resolution_preserves_positive_units_and_rejects_negative_nonfinite_values() {
    for unit in [
        CssResolutionUnit::Dpi,
        CssResolutionUnit::Dpcm,
        CssResolutionUnit::Dppx,
    ] {
        for number in [f32::MIN_POSITIVE, 0.5, 96.0, f32::MAX] {
            let resolution =
                CssNumericLiteral::<CssResolution>::try_new(&number.to_string(), unit).unwrap();
            assert_eq!(resolution.numeric().representation(), number.to_string());
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
            match CssNumericLiteral::<CssResolution>::try_new(&number.to_string(), unit) {
                Ok(literal) => {
                    assert!(number.is_finite());
                    assert_numeric_kind(
                        CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(literal),
                        NumericFailure::OutOfRange,
                    );
                }
                Err(error) => {
                    assert!(!number.is_finite());
                    assert!(error.is_invalid_dimension(), "{error:?}");
                }
            }
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
            let calculation =
                CssCalculation::<CssFrequency>::try_literal(&number.to_string(), unit).unwrap();
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
            assert_numeric_kind(
                CssCalculation::<CssFrequency>::try_literal(&number.to_string(), unit),
                NumericFailure::Component(ComponentFailure::InvalidDimension),
            );
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
        let calculation = CssCalculation::<CssResolution>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(calculation.result_type(), CssCalculationType::Resolution);
        let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
            panic!("ordinary signed resolution leaf");
        };
        assert_eq!(value.literal().representation(), number);
        assert_eq!(value.literal().unit(), Some(unit));
        assert_eq!(
            calculation
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
    }
    for source in ["calc(-1dppx)", "calc(-1dppx / 0)"] {
        let calculation = CssCalculation::<CssResolution>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap();
        assert!(matches!(
            calculation.expression(),
            CssCalculationExpressionRef::NestedCalc(_)
        ));
        assert_eq!(
            calculation
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            source
        );
    }
}

fn resolution_feature(source: &str) -> CssMediaFeatureQuery {
    let report = CssMediaQuery::parse(&CssParser::default(), source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        report
            .syntax()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("condition query");
    };
    let CssConditionKind::Leaf(CssMediaLeaf::Feature(
        feature @ CssMediaFeatureQuery::Resolution(_),
    )) = condition.kind()
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

// Functional new-API evidence; the existing zero test above supplies RED ancestry.
mod construction {
    use super::*;
    use crate::css::support::failure_classes::SerializationFailure as S;
    use CssSerializationLimits as L;
    use std::error::Error;

    fn token(source: &str) -> CssComponentValue {
        CssComponentValues::parse(&CssParser::default(), source)
            .unwrap()
            .items()
            .last()
            .unwrap()
            .clone()
    }
    fn frequency(source: &str) -> CssNumeric<CssFrequency> {
        CssNumeric::<CssFrequency>::try_from_calculation(
            CssCalculation::<CssFrequency>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn resolution(source: &str) -> CssNumeric<CssResolution, CssNonNegative> {
        CssNumeric::<CssResolution, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssResolution>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn decoded_unit(component: &CssComponentValue) -> &str {
        let CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) =
            component.view()
        else {
            panic!("dimension token")
        };
        unit
    }

    #[test]
    fn signed_frequency_and_nonnegative_resolution_retain_exact_ordinary_coefficients() {
        for unit in [CssFrequencyUnit::Hertz, CssFrequencyUnit::Kilohertz] {
            for number in [
                "-1e999",
                "-1e-999",
                "-0e999",
                "-0.000",
                "+0e-999",
                ".5",
                "+1.234567890123456789",
                "1.0000000000000001",
                "1.0000000000000002",
            ] {
                let literal = CssNumericLiteral::<CssFrequency>::try_new(number, unit).unwrap();
                assert_eq!(literal.numeric().representation(), number);
                assert_eq!(literal.unit(), unit);
                assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
                assert!(std::ptr::eq(literal.origin(), literal.component().origin()));
                let value = CssNumeric::<CssFrequency>::try_from_literal(literal.clone()).unwrap();
                assert_eq!(value.literal(), Some(&literal));
                assert!(value.calculation().is_none());
                assert_eq!(value.origin(), literal.origin());
            }
        }
        for unit in [
            CssResolutionUnit::Dpi,
            CssResolutionUnit::Dpcm,
            CssResolutionUnit::Dppx,
        ] {
            for number in [
                "0",
                "-0",
                "-0e999",
                "-0.000",
                "+0e-999",
                "1e999",
                "1e-999",
                ".5",
                "1.0000000000000001",
            ] {
                let literal = CssNumericLiteral::<CssResolution>::try_new(number, unit).unwrap();
                assert_eq!(literal.numeric().representation(), number);
                assert_eq!(literal.unit(), unit);
                assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
                let value =
                    CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(literal.clone())
                        .unwrap();
                assert_eq!(value.literal(), Some(&literal));
                assert!(value.calculation().is_none());
                assert_eq!(value.origin(), literal.origin());
            }
            for number in ["-1", "-1e999", "-1e-999", "-0.00000000000000000000001"] {
                let error = CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(
                    CssNumericLiteral::<CssResolution>::try_new(number, unit).unwrap(),
                )
                .unwrap_err();
                assert!(error.is_out_of_range(), "{error:?}");
                assert!(matches!(error.origin(), Some(CssValueOrigin::Programmatic)));
                assert!(error.source().is_none());
                assert_eq!(error.path(), None);
            }
        }
    }

    #[test]
    fn cased_escaped_units_and_resolution_alias_keep_original_components_and_origins() {
        for (source, unit, decoded) in [
            ("/* 😀 */\n1Hz", CssFrequencyUnit::Hertz, "Hz"),
            ("1KHZ", CssFrequencyUnit::Kilohertz, "KHZ"),
            (r"1k\48 z", CssFrequencyUnit::Kilohertz, "kHz"),
        ] {
            let component = token(source);
            let literal =
                CssNumericLiteral::<CssFrequency>::try_from_component(component.clone()).unwrap();
            assert_eq!(literal.component(), &component);
            assert_eq!(literal.origin(), component.origin());
            assert_eq!(literal.unit(), unit);
            assert_eq!(decoded_unit(literal.component()), decoded);
            let CssValueOrigin::Parsed(origin) = literal.origin() else {
                panic!("parsed token")
            };
            assert_eq!(origin.source().as_str(), source);
        }
        for (source, unit, decoded) in [
            ("1DPI", CssResolutionUnit::Dpi, "DPI"),
            ("1DpCm", CssResolutionUnit::Dpcm, "DpCm"),
            ("1DPPX", CssResolutionUnit::Dppx, "DPPX"),
            ("1x", CssResolutionUnit::Dppx, "x"),
            (r"1\78", CssResolutionUnit::Dppx, "x"),
        ] {
            let component = token(source);
            let literal =
                CssNumericLiteral::<CssResolution>::try_from_component(component.clone()).unwrap();
            assert_eq!(literal.component(), &component);
            assert_eq!(literal.origin(), component.origin());
            assert_eq!(literal.unit(), unit);
            assert_eq!(decoded_unit(literal.component()), decoded);
        }
    }

    #[test]
    fn intrinsic_literal_errors_preserve_component_sources_and_distinct_range_failures() {
        for source in ["0", "1%", "1s", "infinity", "NaN", "\"1hz\"", "calc(1hz)"] {
            let component = token(source);
            let error = CssNumericLiteral::<CssFrequency>::try_from_component(component.clone())
                .unwrap_err();
            assert!(error.is_root_domain_mismatch(), "{error:?}");
            assert_eq!(error.origin(), Some(component.origin()));
            assert!(error.source().is_none());
        }
        for source in [
            "0",
            "1%",
            "1hz",
            "infinity",
            "NaN",
            "\"1dpi\"",
            "calc(-1dppx)",
        ] {
            let component = token(source);
            let error = CssNumericLiteral::<CssResolution>::try_from_component(component.clone())
                .unwrap_err();
            assert!(error.is_root_domain_mismatch(), "{error:?}");
            assert_eq!(error.origin(), Some(component.origin()));
            assert_eq!(error.path(), None);
            assert!(error.source().is_none());
        }
        // The range belongs to the value, not to the literal.
        let component = token("/* independently parsed 😀 */\n-1e-999DPCM");
        assert!(CssNumericLiteral::<CssResolution>::try_from_component(component.clone()).is_ok());
        let error =
            CssNumeric::<CssResolution, CssNonNegative>::try_from_component(component.clone())
                .unwrap_err();
        assert!(error.is_out_of_range(), "{error:?}");
        assert_eq!(error.origin(), Some(component.origin()));
        assert!(error.source().is_none());
        assert_eq!(error.path(), None);
        for number in ["", " ", "1 2", "1e", "NaN", "infinity", "-inf", "1.", "1hz"] {
            assert!(
                CssNumericLiteral::<CssFrequency>::try_new(number, CssFrequencyUnit::Hertz)
                    .unwrap_err()
                    .is_invalid_dimension()
            );
            let error = CssNumericLiteral::<CssResolution>::try_new(number, CssResolutionUnit::Dpi)
                .unwrap_err();
            assert!(error.is_invalid_dimension(), "{error:?}");
            assert!(matches!(error.origin(), CssValueOrigin::Programmatic));
        }
    }

    #[test]
    fn ordinary_calculation_normalization_preserves_exact_origin_and_rechecks_resolution_range() {
        let input = token("/* 😀 */\n1.0000000000000001KHZ");
        let calculation = CssCalculation::<CssFrequency>::try_from_components(
            CssComponentValues::try_new(vec![input.clone()], Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
        let value = CssNumeric::<CssFrequency>::try_from_calculation(calculation).unwrap();
        assert_eq!(value.literal().unwrap().component(), &input);
        assert_eq!(value.origin(), input.origin());
        assert!(value.calculation().is_none());
        let input = token("/* distinct snapshot */\n-0e999x");
        let calculation = CssCalculation::<CssResolution>::try_from_components(
            CssComponentValues::try_new(vec![input.clone()], Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
        let value =
            CssNumeric::<CssResolution, CssNonNegative>::try_from_calculation(calculation).unwrap();
        assert_eq!(value.literal().unwrap().component(), &input);
        assert_eq!(value.literal().unwrap().unit(), CssResolutionUnit::Dppx);
        let input = token("/* original later token */\n-1e-999dpi");
        let calculation = CssCalculation::<CssResolution>::try_from_components(
            CssComponentValues::try_new(vec![input.clone()], Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
        let original = calculation.clone();
        let error =
            CssNumeric::<CssResolution, CssNonNegative>::try_from_calculation(calculation.clone())
                .unwrap_err();
        assert!(error.is_out_of_range(), "{error:?}");
        assert_eq!(error.origin(), Some(input.origin()));
        assert_eq!(error.path(), None);
        assert!(error.source().is_none());
        assert_eq!(calculation, original);
    }

    #[test]
    fn actual_math_grouping_and_symbolic_constants_remain_distinct_from_ordinary_values() {
        for source in [
            "calc(-1hz)",
            "calc((1khz))",
            "min(1hz, 2hz)",
            "calc(infinity * 1hz)",
            "calc(NaN * 1hz)",
        ] {
            let value = frequency(source);
            assert!(value.literal().is_none());
            let calculation = value.calculation().unwrap();
            assert_eq!(
                calculation
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            assert_eq!(value.origin(), calculation.origin());
        }
        for source in [
            "calc(-1dppx)",
            "calc((1dpi))",
            "min(-1dpcm, 2dpcm)",
            "calc(infinity * 1dppx)",
            "calc(NaN * 1dppx)",
        ] {
            let value = resolution(source);
            assert!(value.literal().is_none());
            let calculation = value.calculation().unwrap();
            assert_eq!(
                calculation
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            assert_eq!(value.origin(), calculation.origin());
        }
        for source in ["0", "1s", "(1hz)"] {
            let error = CssCalculation::<CssFrequency>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap_err();
            assert!(error.is_root_domain_mismatch(), "{error:?}");
            assert_eq!(error.path(), Some(&[0][..]));
            assert!(error.source().is_none());
        }
        for source in ["0", "1hz", "(1dpi)"] {
            let error = CssCalculation::<CssResolution>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap_err();
            assert!(error.is_root_domain_mismatch(), "{error:?}");
            assert_eq!(error.path(), Some(&[0][..]));
        }
    }

    #[test]
    fn exact_frequency_convenience_has_checked_component_and_numeric_limit_errors() {
        for (number, unit, suffix) in [
            ("-1e999", CssFrequencyUnit::Hertz, "hz"),
            ("1.1234567890123456789", CssFrequencyUnit::Kilohertz, "khz"),
        ] {
            let calculation = CssCalculation::<CssFrequency>::try_literal(number, unit).unwrap();
            let CssCalculationExpressionRef::Value(value) = calculation.expression() else {
                panic!("ordinary root")
            };
            assert_eq!(value.literal().representation(), number);
            assert_eq!(value.literal().unit(), Some(suffix));
            assert!(matches!(
                value.literal().origin(),
                CssValueOrigin::Programmatic
            ));
        }
        let error = CssCalculation::<CssFrequency>::try_literal("NaN", CssFrequencyUnit::Hertz)
            .unwrap_err();
        assert!(
            error
                .component_error()
                .is_some_and(|error| error.is_invalid_dimension()),
            "{error:?}"
        );
        assert!(error.source().is_some());
        assert_eq!(error.path(), None);
        for components in [
            CssComponentValues::parse(&CssParser::default(), "1hz").unwrap(),
            CssComponentValues::parse(&CssParser::default(), "1dppx").unwrap(),
        ] {
            let limits = CssComponentValueLimits::try_new(1, 1, 0).unwrap();
            let error = if decoded_unit(&components.items()[0]) == "hz" {
                CssCalculation::<CssFrequency>::try_from_components(components, limits).unwrap_err()
            } else {
                CssCalculation::<CssResolution>::try_from_components(components, limits)
                    .unwrap_err()
            };
            assert!(error.is_resource_limit(), "{error:?}");
            let source = error
                .source()
                .unwrap()
                .downcast_ref::<CssComponentValueError>()
                .unwrap();
            assert!(source.is_byte_limit(), "{source:?}");
        }
    }

    #[test]
    fn strict_calculation_frontdoors_reject_original_recovery_without_admitting_a_clean_prefix() {
        for source in ["calc(1hz", "calc(min(-1hz, 2hz"] {
            let error = CssCalculation::<CssFrequency>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap_err();
            assert!(error.is_recovered_component(), "{error:?}");
            assert!(error.source().is_none());
        }
        for source in ["calc(-1dppx", "calc(min(-1dppx, 2dppx"] {
            let error = CssCalculation::<CssResolution>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), source).unwrap(),
                Default::default(),
            )
            .unwrap_err();
            assert!(error.is_recovered_component(), "{error:?}");
            assert!(error.source().is_none());
        }
        assert!(
            CssCalculation::<CssFrequency>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), "1hz 2hz").unwrap(),
                Default::default()
            )
            .unwrap_err()
            .is_multiple_values()
        );
        assert!(
            CssCalculation::<CssResolution>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), "1dpi 2dpi").unwrap(),
                Default::default()
            )
            .unwrap_err()
            .is_multiple_values()
        );
    }

    #[test]
    fn raw_identity_compares_values_units_and_branches_without_provenance() {
        let first = CssNumericLiteral::<CssFrequency>::try_from_component(token("1hz")).unwrap();
        let second =
            CssNumericLiteral::<CssFrequency>::try_from_component(token("/* another origin */1hz"))
                .unwrap();
        assert_eq!(first, second);
        assert_eq!(first, first.clone());
        assert_eq!(
            CssNumeric::<CssFrequency>::try_from_literal(first).unwrap(),
            CssNumeric::<CssFrequency>::try_from_literal(second).unwrap()
        );
        for (a, b, equal) in [
            ("1hz", "1.0hz", true),
            ("1hz", "1Hz", true),
            ("1000hz", "1khz", false),
            ("1e-999hz", "2e-999hz", false),
            ("1.0000000000000001hz", "1.0000000000000002hz", false),
            ("0hz", "calc(0hz)", false),
        ] {
            assert_eq!(frequency(a) == frequency(b), equal, "{a} versus {b}");
        }
        let first = CssNumericLiteral::<CssResolution>::try_from_component(token("1x")).unwrap();
        let second =
            CssNumericLiteral::<CssResolution>::try_from_component(token("/* another origin */1x"))
                .unwrap();
        assert_eq!(first, second);
        assert_eq!(first, first.clone());
        assert_eq!(
            CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(first).unwrap(),
            CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(second).unwrap()
        );
        // `x` is Values 4's alias spelling of the dppx unit.
        for (a, b, equal) in [
            ("1x", "1dppx", true),
            ("1dpi", "1DPI", true),
            ("1dpi", "1.0dpi", true),
            ("1e-999dpi", "2e-999dpi", false),
            ("0dpi", "-0dpi", true),
            ("1dppx", "1dpi", false),
            ("1dppx", "calc(1dppx)", false),
        ] {
            assert_eq!(resolution(a) == resolution(b), equal, "{a} versus {b}");
        }
    }

    #[test]
    fn programmatic_identity_compares_exact_values_across_spellings() {
        for (a, b, equal) in [
            ("1", "1.0", true),
            ("1", "1e0", true),
            ("0", "-0", true),
            ("1e-999", "2e-999", false),
            ("1.0000000000000001", "1.0000000000000002", false),
        ] {
            let left =
                CssNumericLiteral::<CssFrequency>::try_new(a, CssFrequencyUnit::Hertz).unwrap();
            let right =
                CssNumericLiteral::<CssFrequency>::try_new(b, CssFrequencyUnit::Hertz).unwrap();
            assert_eq!(left.origin(), right.origin());
            assert_eq!(left == right, equal, "{a} versus {b}");
            assert_eq!(
                CssNumeric::<CssFrequency>::try_from_literal(left).unwrap()
                    == CssNumeric::<CssFrequency>::try_from_literal(right).unwrap(),
                equal,
                "{a} versus {b}"
            );
            let left =
                CssNumericLiteral::<CssResolution>::try_new(a, CssResolutionUnit::Dpi).unwrap();
            let right =
                CssNumericLiteral::<CssResolution>::try_new(b, CssResolutionUnit::Dpi).unwrap();
            assert_eq!(left.origin(), right.origin());
            assert_eq!(left == right, equal, "{a} versus {b}");
            assert_eq!(
                CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(left).unwrap()
                    == CssNumeric::<CssResolution, CssNonNegative>::try_from_literal(right)
                        .unwrap(),
                equal,
                "{a} versus {b}"
            );
        }
        for (number, kind) in [
            ("1", CssNumericTokenKind::Integer),
            ("1e0", CssNumericTokenKind::Number),
            (".5", CssNumericTokenKind::Number),
        ] {
            assert_eq!(
                CssNumericLiteral::<CssFrequency>::try_new(number, CssFrequencyUnit::Hertz)
                    .unwrap()
                    .numeric()
                    .kind(),
                kind
            );
            assert_eq!(
                CssNumericLiteral::<CssResolution>::try_new(number, CssResolutionUnit::Dpi)
                    .unwrap()
                    .numeric()
                    .kind(),
                kind
            );
        }
        let x = CssNumericLiteral::<CssResolution>::try_from_component(
            CssComponentValue::try_dimension("1", "x").unwrap(),
        )
        .unwrap();
        let dppx =
            CssNumericLiteral::<CssResolution>::try_new("1", CssResolutionUnit::Dppx).unwrap();
        assert_eq!(x.origin(), dppx.origin());
        assert_eq!(x.unit(), dppx.unit());
        assert_eq!(x, dppx);
        let literal =
            CssNumericLiteral::<CssFrequency>::try_new("1", CssFrequencyUnit::Hertz).unwrap();
        let args =
            CssComponentValues::try_new(vec![literal.component().clone()], Default::default())
                .unwrap();
        let root = CssComponentValue::try_function("calc", args).unwrap();
        let calculation = CssCalculation::<CssFrequency>::try_from_components(
            CssComponentValues::try_new(vec![root], Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
        let ordinary = CssNumeric::<CssFrequency>::try_from_literal(literal).unwrap();
        let math = CssNumeric::<CssFrequency>::try_from_calculation(calculation).unwrap();
        assert_eq!(ordinary.origin(), math.origin());
        assert_ne!(ordinary, math);
    }

    #[test]
    fn media_resolution_preservation_includes_decoded_escaped_negative_and_alias_units() {
        for (source, number, unit) in [
            (r"(resolution: -1\64 pi)", "-1", "dpi"),
            (r"(resolution >= -2\78)", "-2", "x"),
        ] {
            let report = CssMediaQuery::parse(&CssParser::default(), source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let CssMediaQuery::Condition(condition) = report.syntax() else {
                panic!("condition")
            };
            let CssConditionKind::Leaf(CssMediaLeaf::Feature(CssMediaFeatureQuery::Resolution(
                range,
            ))) = condition.kind()
            else {
                panic!("known resolution")
            };
            let value = match range.view() {
                CssMediaRangeRef::Plain { value }
                | CssMediaRangeRef::FeatureFirst { value, .. } => value,
                _ => panic!("selected bound"),
            };
            super::assert_media_number(value, number, unit);
            let CssMediaResolutionRef::Numeric(calculation) = value.view() else {
                unreachable!()
            };
            let CssValueOrigin::Parsed(origin) = calculation.origin() else {
                panic!("original operand")
            };
            assert_eq!(origin.source().as_str(), source);
        }
    }

    #[test]
    fn frequency_ordinary_output_rounds_six_places_and_preserves_selected_units() {
        for (source, expected) in [
            ("1kHz", "1khz"),
            ("1000Hz", "1000hz"),
            (r"1k\48 z", "1khz"),
            ("-2.5KHZ", "-2.5khz"),
            ("+1.2300Hz", "1.23hz"),
            ("-0e999Hz", "0hz"),
            ("-0.000kHz", "0khz"),
            ("1.1234567890123456789Hz", "1.123457hz"),
            ("0.0000001Hz", "0hz"),
            ("1e-8Hz", "0hz"),
        ] {
            let literal =
                CssNumericLiteral::<CssFrequency>::try_from_component(token(source)).unwrap();
            let original = literal.clone();
            assert_eq!(
                literal_specified(&literal, Default::default()).unwrap(),
                expected
            );
            assert_eq!(
                CssNumeric::<CssFrequency>::try_from_literal(literal.clone())
                    .unwrap()
                    .serialize_specified(Default::default())
                    .unwrap(),
                expected
            );
            assert_eq!(literal, original);
        }
        let literal =
            CssNumericLiteral::<CssFrequency>::try_new("1e40", CssFrequencyUnit::Kilohertz)
                .unwrap();
        assert_eq!(
            literal_specified(&literal, Default::default()).unwrap(),
            format!("1{}khz", "0".repeat(40))
        );
    }

    #[test]
    fn ordinary_frequency_resource_limits_have_independent_atomic_boundaries() {
        for (number, unit, expected) in [
            ("1", CssFrequencyUnit::Kilohertz, "1khz".to_owned()),
            ("-0e999", CssFrequencyUnit::Hertz, "0hz".to_owned()),
            ("1e-8", CssFrequencyUnit::Hertz, "0hz".to_owned()),
            (
                "1e40",
                CssFrequencyUnit::Kilohertz,
                format!("1{}khz", "0".repeat(40)),
            ),
        ] {
            let literal = CssNumericLiteral::<CssFrequency>::try_new(number, unit).unwrap();
            let value = CssNumeric::<CssFrequency>::try_from_literal(literal.clone()).unwrap();
            let original_literal = literal.clone();
            let original_value = value.clone();
            let boundary = L::new(1, 1, expected.len());
            assert_eq!(literal_specified(&literal, boundary).unwrap(), expected);
            assert_eq!(value.serialize_specified(boundary).unwrap(), expected);
            for (limit, kind) in [
                (L::new(0, 1, expected.len()), S::InputNodeLimit),
                (L::new(1, 0, expected.len()), S::ProjectionNodeLimit),
                (L::new(1, 1, expected.len() - 1), S::ByteLimit),
            ] {
                let error = literal_specified(&literal, limit).unwrap_err();
                assert_eq!(serialization_failure(&error), kind);
                assert!(error.source().is_none());
                assert_eq!(
                    serialization_failure(&value.serialize_specified(limit).unwrap_err()),
                    kind
                );
                assert_eq!(literal, original_literal);
                assert_eq!(value, original_value);
            }
        }
        let literal =
            CssNumericLiteral::<CssFrequency>::try_new("1e999999", CssFrequencyUnit::Hertz)
                .unwrap();
        let original = literal.clone();
        assert!(
            literal_specified(&literal, L::new(1, 1, 16))
                .unwrap_err()
                .is_byte_limit()
        );
        assert_eq!(literal, original);
        assert_eq!(L::default(), L::new(65_536, 262_144, 1_048_576));
    }

    #[test]
    fn actual_frequency_math_keeps_simplification_and_cumulative_limits() {
        let value = frequency("calc(1hz + 2hz)");
        let original = value.clone();
        assert_eq!(
            value
                .calculation()
                .unwrap()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            "calc(1hz + 2hz)"
        );
        assert_eq!(
            value.serialize_specified(L::new(4, 3, 9)).unwrap(),
            "calc(3hz)"
        );
        for (limit, kind) in [
            (L::new(3, 3, 9), S::InputNodeLimit),
            (L::new(4, 2, 9), S::ProjectionNodeLimit),
            (L::new(4, 3, 8), S::ByteLimit),
        ] {
            assert_eq!(
                serialization_failure(&value.serialize_specified(limit).unwrap_err()),
                kind
            );
            assert_eq!(value, original);
        }
        for (source, expected) in [
            ("calc(1khz)", "calc(1000hz)".to_owned()),
            // An operation-free leaf keeps its exact coefficient; conversion
            // overflow converts to the closest finite value (#1297).
            ("calc(1e999hz)", format!("calc(1{}hz)", "0".repeat(999))),
            ("calc(1e999khz)", format!("calc({:.0}hz)", f64::MAX)),
            ("calc(1e-999hz)", "calc(0hz)".to_owned()),
            ("calc((1hz))", "calc(1hz)".to_owned()),
        ] {
            let value = frequency(source);
            assert_eq!(
                value.serialize_specified(Default::default()).unwrap(),
                expected
            );
            assert_eq!(
                value
                    .calculation()
                    .unwrap()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
        }
    }

    #[test]
    fn mixed_numeric_children_keep_independent_snapshots_through_value_construction() {
        let left = token("/* first 😀 */\n1hz");
        let right = token("/* second */\n2hz");
        let values = CssComponentValues::try_new(
            vec![
                left.clone(),
                CssComponentValue::try_token(" ").unwrap(),
                CssComponentValue::try_token("+").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                right.clone(),
            ],
            Default::default(),
        )
        .unwrap();
        let function = CssComponentValue::try_function("calc", values).unwrap();
        let calculation = CssCalculation::<CssFrequency>::try_from_components(
            CssComponentValues::try_new(vec![function], Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
        let value = CssNumeric::<CssFrequency>::try_from_calculation(calculation).unwrap();
        let CssCalculationExpressionRef::NestedCalc(outer) =
            value.calculation().unwrap().expression()
        else {
            panic!("calc root")
        };
        let CssCalculationExpressionRef::Sum(sum) = outer.operand() else {
            panic!("sum")
        };
        assert_eq!(sum.term(0).unwrap().expression().origin(), left.origin());
        assert_eq!(sum.term(1).unwrap().expression().origin(), right.origin());
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "calc(3hz)"
        );
        assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    }
}
