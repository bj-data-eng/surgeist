#![forbid(unsafe_code)]

//! Checked authored nonnegative numbers retain exact syntax and provenance.

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssNumberCalculation, CssNumericConstructionErrorKind,
    CssSpecifiedNonNegativeNumber, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, CssValueOrigin, CssValueTokenRef, parse_component_values,
};

fn literal(text: &str) -> CssSpecifiedNonNegativeNumber {
    CssSpecifiedNonNegativeNumber::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}

#[test]
fn exact_ordinary_numbers_survive_admission_without_float_narrowing() {
    for (authored, canonical) in [
        ("0", "0"),
        ("-0", "0"),
        ("1.250", "1.25"),
        ("1.0000000000000001", "1"),
        ("1e3", "1000"),
        ("1e-999", "0"),
    ] {
        let value = literal(authored);
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) =
            value.literal_component().unwrap().view()
        else {
            panic!("number token")
        };
        assert_eq!(number.representation(), authored);
        assert_eq!(value.serialize_specified().unwrap(), canonical);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    }
    let huge = literal("1e999");
    assert_eq!(
        huge.serialize_specified().unwrap(),
        format!("1{}", "0".repeat(999))
    );
}

#[test]
fn exact_negative_and_wrong_domain_errors_retain_component_origin() {
    for text in ["-1e-999", "-0.000000000000000000001", "-1"] {
        let error = CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.kind(), &CssNumericConstructionErrorKind::OutOfRange);
        assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
    }
    for text in ["1px", "1%", "1deg", "auto"] {
        let error = CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
    }
    let parsed = parse_component_values("  -1e-999").unwrap();
    let error = CssSpecifiedNonNegativeNumber::try_from_component(
        parsed
            .items()
            .iter()
            .find(|component| {
                matches!(
                    component.view(),
                    CssComponentValueRef::Token(CssValueTokenRef::Number(_))
                )
            })
            .unwrap()
            .clone(),
    )
    .unwrap_err();
    let CssValueOrigin::Parsed(origin) = error.origin().unwrap() else {
        panic!("parsed error")
    };
    assert_eq!(origin.source().as_str(), "  -1e-999");
    assert_eq!(origin.span().start().byte_offset().value(), 2);
}

#[test]
fn bare_calculation_roots_reenter_literals_and_actual_math_stays_symbolic() {
    for text in ["0", "1e999", "-1e-999"] {
        let calculation =
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap();
        let result = CssSpecifiedNonNegativeNumber::try_from_calculation(calculation);
        if text.starts_with('-') {
            assert_eq!(
                result.unwrap_err().kind(),
                &CssNumericConstructionErrorKind::OutOfRange
            );
        } else {
            let value = result.unwrap();
            assert!(value.literal_component().is_some());
            assert!(value.calculation().is_none());
        }
    }
    let math = CssSpecifiedNonNegativeNumber::try_from_calculation(
        CssNumberCalculation::try_from_components(parse_component_values("calc(-1)").unwrap())
            .unwrap(),
    )
    .unwrap();
    assert!(math.literal_component().is_none());
    assert!(math.calculation().is_some());
    assert_eq!(math.serialize_specified().unwrap(), "calc(-1)");
    assert!(
        CssNumberCalculation::try_from_components(parse_component_values("calc(1 + 2%)").unwrap())
            .is_err()
    );
}

#[test]
fn shared_scalar_equality_retains_provenance_and_budgets_are_atomic() {
    let parsed = parse_component_values("1.250").unwrap();
    let parsed =
        CssSpecifiedNonNegativeNumber::try_from_component(parsed.items()[0].clone()).unwrap();
    let programmatic = literal("1.250");
    assert_ne!(parsed, programmatic);
    assert_eq!(parsed.serialize_specified().unwrap(), "1.25");
    assert_eq!(programmatic.serialize_specified().unwrap(), "1.25");
    let value = literal("12.5");
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 10, 10),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(10, 0, 10),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(10, 10, 3),
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
        assert_eq!(value.serialize_specified().unwrap(), "12.5");
    }
}
