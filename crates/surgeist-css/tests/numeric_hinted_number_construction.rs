#![forbid(unsafe_code)]
//! Values 4 §10.9 retains dimensional percentage hints after powers cancel;
//! Typed OM §4.3.2 matches such Numbers only where percentages are permitted.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-type-checking
//! https://www.w3.org/TR/2024/WD-css-typed-om-1-20240321/#cssnumericvalue-match

use surgeist_css::*;

const SOURCE: &str = "calc((1px + 1%) / 1px)";
const SPECIFIED: &str = "calc((1% + 1px) / 1px)";

fn components(text: &str) -> CssComponentValues {
    parse_component_values(text).unwrap()
}
fn hinted() -> CssHintedNumberCalculation {
    CssHintedNumberCalculation::try_from_components(components(SOURCE)).unwrap()
}
fn zero() -> CssColorComponent {
    CssColorComponent::Number(
        CssColorNumberLiteral::try_from_component(CssComponentValue::try_token("0").unwrap())
            .unwrap(),
    )
}

#[test]
fn checked_root_retains_every_dimensional_hint_and_authored_identity() {
    for (dimension, unit) in [
        (CssNumericDimension::Length, "px"),
        (CssNumericDimension::Angle, "deg"),
        (CssNumericDimension::Time, "s"),
        (CssNumericDimension::Frequency, "hz"),
        (CssNumericDimension::Resolution, "dppx"),
        (CssNumericDimension::Flex, "fr"),
    ] {
        let source = format!("calc((+01{unit} + 1e0%) / 1{unit})");
        let original = components(&source);
        let value = CssHintedNumberCalculation::try_from_components(original.clone()).unwrap();
        assert_eq!(value.result_type(), CssCalculationType::Number);
        assert_eq!(value.numeric_type().percent_hint(), Some(dimension));
        for axis in [
            CssNumericDimension::Length,
            CssNumericDimension::Angle,
            CssNumericDimension::Time,
            CssNumericDimension::Frequency,
            CssNumericDimension::Resolution,
            CssNumericDimension::Flex,
            CssNumericDimension::Percentage,
        ] {
            assert_eq!(value.numeric_type().exponent(axis), 0);
        }
        assert_eq!(value.components(), &original);
        assert_eq!(value.origin(), original.items()[0].origin());
        assert!(value.position().is_some());
        let text = value.serialize().unwrap();
        let rebuilt =
            CssHintedNumberCalculation::try_from_components(components(text.as_css())).unwrap();
        assert_eq!(rebuilt.numeric_type(), value.numeric_type());
        assert_eq!(value.components().serialize().unwrap().as_css(), source);
    }
}

#[test]
fn checked_hinted_root_rejects_other_domains_and_cannot_reenter_pure_number_roots() {
    for source in ["1", "calc(1px / 1px)", "calc(1%)", "calc(1px)"] {
        let error =
            CssHintedNumberCalculation::try_from_components(components(source)).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RootDomainMismatch
        );
        assert!(error.origin().is_some());
    }
    let value = hinted();
    let error = CssNumberCalculation::try_from_components(value.components().clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RootDomainMismatch
    );
    let error = CssIntegerCalculation::try_from_components(value.components().clone()).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::RootDomainMismatch
    );
    for source in ["calc(1 + 1%)", "calc(1px + 1s)", "calc(1px * 1px)"] {
        assert!(CssHintedNumberCalculation::try_from_components(components(source)).is_err());
    }
}

#[test]
fn programmatic_root_preserves_child_origins_and_constructor_limits() {
    let arguments = components("(1px + 1%) / 1px");
    let original = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("CALC", arguments.clone()).unwrap(),
    ])
    .unwrap();
    let value = CssHintedNumberCalculation::try_from_components(original.clone()).unwrap();
    assert!(matches!(value.origin(), CssValueOrigin::Programmatic));
    assert!(value.position().is_none());
    assert_eq!(value.components(), &original);
    // Function + five body components + five grouped components = eleven.
    // Two enclosing blocks and the canonical authored text remain independent limits.
    let exact = CssComponentValueLimits::try_new(2, 11, SOURCE.len()).unwrap();
    assert!(
        CssHintedNumberCalculation::try_from_components_with_limits(original.clone(), exact)
            .is_ok()
    );
    for limits in [
        CssComponentValueLimits::try_new(1, 11, SOURCE.len()).unwrap(),
        CssComponentValueLimits::try_new(2, 10, SOURCE.len()).unwrap(),
        CssComponentValueLimits::try_new(2, 11, SOURCE.len() - 1).unwrap(),
    ] {
        assert!(
            CssHintedNumberCalculation::try_from_components_with_limits(original.clone(), limits)
                .is_err()
        );
    }
    assert_eq!(value.components(), &original);
    assert_eq!(value.serialize().unwrap().as_css(), SOURCE);
}

#[test]
fn opacity_keeps_unknown_basis_symbolic_and_charges_the_shared_projection() {
    let value = CssOpacityValue::HintedNumberCalculation(hinted());
    let before = value.clone();
    // Seven authored expression nodes; three leaves, two merged Sum scalars,
    // one Sum, one inverse and one Product produce eight projection nodes.
    let exact = CssSpecifiedValueSerializationLimits::new(7, 8, SPECIFIED.len());
    assert_eq!(
        value.serialize_specified_with_limits(exact).unwrap(),
        SPECIFIED
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(6, 8, SPECIFIED.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 7, SPECIFIED.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(7, 8, SPECIFIED.len() - 1),
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
    assert_eq!(value.serialize_specified().unwrap(), SPECIFIED);
}

#[test]
fn checked_color_consumers_keep_context_and_borrow_the_distinct_alpha_payload() {
    let calculation = hinted();
    let channel = CssColorComponent::HintedNumberCalculation(calculation.clone());
    let alpha_owner = CssPredefinedColor::try_new(
        CssPredefinedColorSpace::Srgb,
        [zero(), zero(), zero()],
        Some(channel.clone()),
    )
    .unwrap();
    let CssParsedColorAlphaRef::HintedNumberCalculation(alpha) = alpha_owner.parsed_alpha() else {
        panic!("borrowed contextual Number alpha")
    };
    assert_eq!(alpha, &calculation);
    let value = CssDeviceCmykColor::try_new(
        CssColorSyntax::Modern,
        [channel.clone(), zero(), zero(), zero()],
        Some(channel.clone()),
    )
    .unwrap();
    let color = CssColor::from_device_cmyk(value);
    let before = color.clone();
    assert_eq!(
        color.to_specified_css().unwrap(),
        format!("device-cmyk({SPECIFIED} 0 0 0 / {SPECIFIED})")
    );
    assert_eq!(color, before);
    assert!(
        CssDeviceCmykColor::try_new(
            CssColorSyntax::Legacy,
            [channel, zero(), zero(), zero()],
            None,
        )
        .is_err()
    );
}
