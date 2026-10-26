//! Checked authored nonnegative numbers retain exact syntax and provenance.

use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::{
    CssCalculation, CssComponentValue, CssComponentValueRef, CssNonNegative, CssNumber, CssNumeric,
    CssSerializationLimits, CssValueOrigin, CssValueTokenRef, parse_component_values,
};

fn literal(text: &str) -> CssNumeric<CssNumber, CssNonNegative> {
    CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
        CssComponentValue::try_token(text).unwrap(),
    )
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
        let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = value
            .literal()
            .map(|literal| literal.component())
            .unwrap()
            .view()
        else {
            panic!("number token")
        };
        assert_eq!(number.representation(), authored);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            canonical
        );
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    }
    let huge = literal("1e999");
    assert_eq!(
        huge.serialize_specified(Default::default()).unwrap(),
        format!("1{}", "0".repeat(999))
    );
}

#[test]
fn exact_negative_and_wrong_domain_errors_retain_component_origin() {
    for text in ["-1e-999", "-0.000000000000000000001", "-1"] {
        let error = CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert!(error.is_out_of_range(), "{error:?}");
        assert_eq!(error.origin(), Some(&CssValueOrigin::Programmatic));
    }
    for text in ["1px", "1%", "1deg", "auto"] {
        let error = CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
            CssComponentValue::try_token(text).unwrap(),
        )
        .unwrap_err();
        assert!(error.is_root_domain_mismatch(), "{error:?}");
    }
    let parsed = parse_component_values("  -1e-999").unwrap();
    let error = CssNumeric::<CssNumber, CssNonNegative>::try_from_component(
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
        let calculation = CssCalculation::<CssNumber>::try_from_components(
            parse_component_values(text).unwrap(),
            Default::default(),
        )
        .unwrap();
        let result = CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(calculation);
        if text.starts_with('-') {
            assert!(result.unwrap_err().is_out_of_range());
        } else {
            let value = result.unwrap();
            assert!(value.literal().map(|literal| literal.component()).is_some());
            assert!(value.calculation().is_none());
        }
    }
    let math = CssNumeric::<CssNumber, CssNonNegative>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(
            parse_component_values("calc(-1)").unwrap(),
            Default::default(),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(math.literal().map(|literal| literal.component()).is_none());
    assert!(math.calculation().is_some());
    assert_eq!(
        math.serialize_specified(Default::default()).unwrap(),
        "calc(-1)"
    );
    crate::css::support::value_assertions::assert_calculation_rejected(
        CssCalculation::<CssNumber>::try_from_components(
            parse_component_values("calc(1 + 2%)").unwrap(),
            Default::default(),
        ),
        Some(NumericFailure::IncompatibleTypes),
    );
}

#[test]
fn shared_scalar_equality_ignores_provenance_and_budgets_are_atomic() {
    let parsed = parse_component_values("1.250").unwrap();
    let parsed =
        CssNumeric::<CssNumber, CssNonNegative>::try_from_component(parsed.items()[0].clone())
            .unwrap();
    let programmatic = literal("1.250");
    assert_eq!(parsed, programmatic);
    assert_eq!(
        parsed.serialize_specified(Default::default()).unwrap(),
        "1.25"
    );
    assert_eq!(
        programmatic
            .serialize_specified(Default::default())
            .unwrap(),
        "1.25"
    );
    let value = literal("12.5");
    for (limits, expected) in [
        (
            CssSerializationLimits::new(0, 10, 10),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(10, 0, 10),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(10, 10, 3),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            expected
        );
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "12.5"
        );
    }
}
