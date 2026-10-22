//! Numeric properties keep property-specific authored models, and their public
//! constructors reject invalid values.

use surgeist_css::*;

use crate::css::support::sheet_cases::*;

#[test]
fn numeric_properties_use_property_specific_authored_models() {
    let opacity = property_payload!(
        single_declaration(".panel { opacity: 0.5; }"),
        Opacity,
        value
    );
    let CssOpacityValue::Scalar(opacity) = opacity else {
        panic!("ordinary opacity scalar")
    };
    assert_eq!(opacity.kind(), CssOpacityScalarKind::Number);
    assert_eq!(opacity.numeric().representation(), "0.5");
    let grow = property_payload!(
        single_declaration(".panel { flex-grow: 2; }"),
        FlexGrow,
        factor
    );
    assert_eq!(number_spelling(grow.literal_component().unwrap()), "2");
    assert!(grow.calculation().is_none());
    let shrink = property_payload!(
        single_declaration(".panel { flex-shrink: 0; }"),
        FlexShrink,
        factor
    );
    assert_eq!(number_spelling(shrink.literal_component().unwrap()), "0");
    let ratio = property_payload!(
        single_declaration(".panel { aspect-ratio: 1.5; }"),
        AspectRatio,
        ratio
    );
    let CssAspectRatioValue::Ratio(ratio) = ratio else {
        panic!("specified ratio")
    };
    assert_eq!(
        number_spelling(ratio.numerator().literal_component().unwrap()),
        "1.5"
    );
    assert!(ratio.denominator().is_none());
    assert_eq!(
        property_payload!(
            single_declaration(".panel { scrollbar-width: thin; }"),
            ScrollbarWidth,
            value
        ),
        CssScrollbarWidth::Thin
    );
    let exact = CssOpacityScalar::try_from_component(CssComponentValue::try_number("0.5").unwrap())
        .unwrap();
    assert_eq!(exact.numeric().representation(), "0.5");
    assert_eq!(
        number_spelling(nonnegative_number("2").literal_component().unwrap()),
        "2"
    );
    let operand =
        CssRatioOperand::try_from_component(CssComponentValue::try_number("1.5").unwrap()).unwrap();
    assert_eq!(number_spelling(operand.literal_component().unwrap()), "1.5");
}

#[test]
fn numeric_property_models_reject_invalid_authored_values() {
    assert_rejects_declarations(&[
        RejectedDeclarationCase {
            label: "flex-grow rejects negative values",
            property_name: "flex-grow",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "flex-grow",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "flex-shrink rejects negative values",
            property_name: "flex-shrink",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "flex-shrink",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "aspect-ratio rejects a negative operand",
            property_name: "aspect-ratio",
            authored_value: "-1e-999",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "aspect-ratio",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "aspect-ratio rejects negative values",
            property_name: "aspect-ratio",
            authored_value: "-1",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "aspect-ratio",
            },
            property_name_should_be_recognized: true,
        },
        RejectedDeclarationCase {
            label: "scrollbar-width rejects numbers",
            property_name: "scrollbar-width",
            authored_value: "8",
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: "scrollbar-width",
            },
            property_name_should_be_recognized: true,
        },
    ]);

    assert!(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number("-1").unwrap()
        )
        .is_err()
    );
    assert!(
        CssRatioOperand::try_from_component(CssComponentValue::try_number("-1").unwrap()).is_err()
    );
    assert!(CssComponentValue::try_number("Infinity").is_err());
    let zero =
        CssRatioOperand::try_from_component(CssComponentValue::try_number("0").unwrap()).unwrap();
    assert_eq!(number_spelling(zero.literal_component().unwrap()), "0");
}

#[test]
fn constructor_invariants_reject_invalid_public_numeric_values() {
    assert_eq!(CssFiniteNumber::try_new(1.25).unwrap().value(), 1.25);
    assert_eq!(CssFiniteNumber::try_new(f32::NAN), None);
    assert_eq!(CssFiniteNumber::try_new(f32::INFINITY), None);

    for number in ["NaN", "inf", "-inf"] {
        assert!(CssComponentValue::try_dimension(number, "rem").is_err());
        assert!(CssComponentValue::try_token(&format!("{number}%")).is_err());
    }
    let exact = signed_length("4px");
    assert!(exact_dimension(exact.literal_component(), "4", "px"));
    assert!(matches!(exact.origin(), CssValueOrigin::Programmatic));

    assert!(CssComponentValue::try_number("NaN").is_err());
    assert!(CssComponentValue::try_number("inf").is_err());
    assert!(
        CssSpecifiedNonNegativeNumber::try_from_component(
            CssComponentValue::try_number("-1").unwrap()
        )
        .is_err()
    );
    let grow = nonnegative_number("1");
    let shrink = nonnegative_number("0");
    let basis = CssFlexBasisValue::from(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            CssComponentValue::try_token("2px").unwrap(),
        )
        .unwrap(),
    )));
    let components = CssFlexComponents::try_new(Some(grow), Some(shrink), Some(basis)).unwrap();
    assert_eq!(
        number_spelling(components.grow().unwrap().literal_component().unwrap()),
        "1"
    );
    assert_eq!(
        number_spelling(components.shrink().unwrap().literal_component().unwrap()),
        "0"
    );
    let CssFlexBasisRef::Size(CssSizeValue::BoxSize(CssBoxSize::LengthPercentage(length))) =
        components.basis().unwrap().view()
    else {
        panic!("length flex basis")
    };
    assert_length_literal(length.literal_component().unwrap(), "2", "px");
}
