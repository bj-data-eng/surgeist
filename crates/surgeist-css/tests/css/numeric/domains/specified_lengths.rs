//! Exact CSS `<length>` and nonnegative `<length-percentage>` contracts.

use surgeist_css::{
    CssCalculation, CssComponentValue, CssComponentValues, CssLength, CssLengthPercentage,
    CssNonNegative, CssNumeric, CssParser, CssSerializationLimits, CssValueOrigin,
};

fn component(text: &str) -> CssComponentValue {
    let values =
        CssComponentValues::parse(&CssParser::default(), text).expect("one valid component value");
    let [value] = values.items() else {
        panic!("one component in {text}")
    };
    value.clone()
}

#[test]
fn signed_length_accepts_only_exact_length_literals_and_unitless_zero() {
    for (text, expected) in [
        ("-12px", "-12px"),
        ("+2EM", "2em"),
        ("2\\50 X", "2px"),
        ("0", "0"),
        ("-0", "0"),
        ("-0.000e999", "0"),
    ] {
        let value = CssNumeric::<CssLength>::try_from_component(component(text)).unwrap();
        assert!(
            value.literal().map(|literal| literal.component()).is_some(),
            "{text}"
        );
        assert!(value.calculation().is_none(), "{text}");
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected,
            "{text}"
        );
    }
    for text in ["1", "1e-999", "10%", "auto", "2deg", "2future"] {
        let error = CssNumeric::<CssLength>::try_from_component(component(text)).unwrap_err();
        assert!(error.is_root_domain_mismatch(), "{text}");
    }
    let initial = CssNumeric::<CssLength>::zero();
    assert!(matches!(initial.origin(), CssValueOrigin::Programmatic));
    assert_eq!(
        initial.serialize_specified(Default::default()).unwrap(),
        "0"
    );
}

#[test]
fn nonnegative_length_percentage_checks_exact_ordinary_sign_and_domain() {
    for (text, expected) in [
        ("0", "0"),
        ("-0", "0"),
        ("-0px", "0px"),
        ("-0%", "0%"),
        ("1.25px", "1.25px"),
        ("2\\50 X", "2px"),
        ("12.5%", "12.5%"),
    ] {
        let value =
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(text))
                .unwrap_or_else(|error| panic!("{text}: {error:?}"));
        assert!(
            value.literal().map(|literal| literal.component()).is_some(),
            "{text}"
        );
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected,
            "{text}"
        );
    }
    for text in ["-1px", "-1e-999px", "-1e-999%"] {
        let error =
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(text))
                .unwrap_err();
        assert!(error.is_out_of_range(), "{text}");
        assert_eq!(error.origin(), Some(component(text).origin()));
    }
    for text in ["1", "1e-999", "auto", "2deg", "2future"] {
        let error =
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(text))
                .unwrap_err();
        assert!(error.is_root_domain_mismatch(), "{text}");
    }
}

#[test]
fn extreme_ordinary_magnitudes_keep_exact_digits_and_bounded_serialization() {
    let huge = CssNumeric::<CssLength>::try_from_component(component("1e999px")).unwrap();
    let expected_huge = format!("1{}px", "0".repeat(999));
    assert_eq!(
        huge.serialize_specified(Default::default()).unwrap(),
        expected_huge
    );

    let tiny = CssNumeric::<CssLength>::try_from_component(component("1e-999px")).unwrap();
    let expected_tiny = "0px";
    assert_eq!(
        tiny.serialize_specified(Default::default()).unwrap(),
        expected_tiny
    );

    let huge_percent =
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component("1e999%"))
            .unwrap();
    assert_eq!(
        huge_percent
            .serialize_specified(Default::default())
            .unwrap(),
        format!("1{}%", "0".repeat(999))
    );

    let exact = CssSerializationLimits::new(1, 1, expected_huge.len());
    assert_eq!(huge.serialize_specified(exact).unwrap(), expected_huge);
    for limits in [
        CssSerializationLimits::new(0, 1, expected_huge.len()),
        CssSerializationLimits::new(1, 0, expected_huge.len()),
    ] {
        assert!(huge.serialize_specified(limits).is_err());
    }
    let one_byte_short = CssSerializationLimits::new(1, 1, expected_huge.len() - 1);
    assert!(
        huge.serialize_specified(one_byte_short)
            .unwrap_err()
            .is_byte_limit()
    );
}

#[test]
fn checked_calculation_entry_rechecks_bare_roots_and_defers_function_math() {
    let signed_bare = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "-4px").unwrap(),
        Default::default(),
    )
    .unwrap();
    let signed = CssNumeric::<CssLength>::try_from_calculation(signed_bare).unwrap();
    assert!(
        signed
            .literal()
            .map(|literal| literal.component())
            .is_some()
    );
    assert!(signed.calculation().is_none());
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "-4px"
    );

    let forbidden_bare = CssCalculation::<CssLengthPercentage>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "-1e-999px").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(forbidden_bare)
            .unwrap_err()
            .is_out_of_range()
    );
    let allowed_bare = CssCalculation::<CssLengthPercentage>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "1e999%").unwrap(),
        Default::default(),
    )
    .unwrap();
    let allowed =
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(allowed_bare)
            .unwrap();
    assert!(
        allowed
            .literal()
            .map(|literal| literal.component())
            .is_some()
    );
    assert!(allowed.calculation().is_none());

    let margin_math = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "calc(-2px)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let margin = CssNumeric::<CssLength>::try_from_calculation(margin_math).unwrap();
    assert!(
        margin
            .literal()
            .map(|literal| literal.component())
            .is_none()
    );
    assert!(margin.calculation().is_some());
    assert!(matches!(margin.origin(), CssValueOrigin::Parsed(_)));
    assert!(
        !margin
            .serialize_specified(Default::default())
            .unwrap()
            .is_empty()
    );

    let padding_math = CssCalculation::<CssLengthPercentage>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "calc(2px + 3%)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let padding =
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(padding_math)
            .unwrap();
    assert!(
        padding
            .literal()
            .map(|literal| literal.component())
            .is_none()
    );
    assert!(padding.calculation().is_some());
    assert!(
        !padding
            .serialize_specified(Default::default())
            .unwrap()
            .is_empty()
    );
    assert!(
        CssCalculation::<CssLength>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), "calc(2px + 3%)").unwrap(),
            Default::default()
        )
        .is_err()
    );
}

#[test]
fn programmatic_literals_retain_programmatic_origin_and_exact_value() {
    let component = CssComponentValue::try_dimension("1e999", "Px").unwrap();
    let length = CssNumeric::<CssLength>::try_from_component(component).unwrap();
    assert!(matches!(length.origin(), CssValueOrigin::Programmatic));
    assert_eq!(
        length.serialize_specified(Default::default()).unwrap(),
        format!("1{}px", "0".repeat(999))
    );
    let percent = CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
        CssComponentValue::try_token("1e-999%").unwrap(),
    )
    .unwrap();
    assert!(matches!(percent.origin(), CssValueOrigin::Programmatic));
    assert_eq!(
        percent.serialize_specified(Default::default()).unwrap(),
        "0%"
    );
}
