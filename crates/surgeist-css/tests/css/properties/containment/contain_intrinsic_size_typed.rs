use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::property_admission::{assert_component_rejected, assert_numeric_rejected};
use surgeist_css::{
    CssComponentValue, CssContainIntrinsicSize, CssContainIntrinsicSizeFallback,
    CssContainIntrinsicSizeValue, CssLengthCalculation, CssSerializationLimits,
    CssSpecifiedNonNegativeLength, CssValueOrigin, parse_component_values,
};

fn component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("one component: {text}")
    };
    component.clone()
}

fn length(text: &str) -> CssSpecifiedNonNegativeLength {
    if text.starts_with("calc(") {
        CssSpecifiedNonNegativeLength::try_from_calculation(
            CssLengthCalculation::try_from_components(
                parse_component_values(text).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedNonNegativeLength::try_from_component(component(text)).unwrap()
    }
}

fn length_value(text: &str) -> CssContainIntrinsicSizeValue {
    CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::Length(length(text)))
}

#[test]
fn checked_nonnegative_length_preserves_exact_literal_domain() {
    for text in ["0", "-0", "0px", "-0px", "1px", "1e999px", "1e-999px"] {
        let value = length(text);
        assert!(value.literal_component().is_some());
        assert_eq!(value.literal_component().unwrap().origin(), value.origin());
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    }
    assert_eq!(
        CssSpecifiedNonNegativeLength::zero()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
    let negative_zero =
        CssSpecifiedNonNegativeLength::try_from_component(component("-0px")).unwrap();
    let surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension {
        number,
        ..
    }) = negative_zero.literal_component().unwrap().view()
    else {
        panic!("retained negative-zero dimension")
    };
    assert_eq!(number.representation(), "-0");
    let huge = length("1e999px")
        .serialize_specified(Default::default())
        .unwrap();
    assert_eq!(huge.len(), 1002); // 1 followed by 999 zeroes, then px.
    assert!(huge.starts_with('1') && huge.ends_with("px"));
    for (text, kind) in [
        ("1", NumericFailure::RootDomainMismatch),
        ("1%", NumericFailure::RootDomainMismatch),
        ("-1px", NumericFailure::OutOfRange),
        ("-1e-999px", NumericFailure::OutOfRange),
        ("1fr", NumericFailure::RootDomainMismatch),
    ] {
        assert_component_rejected(
            CssSpecifiedNonNegativeLength::try_from_component,
            component(text),
            kind,
        );
    }
}

#[test]
fn bare_calculation_roots_reenter_ordinary_admission_but_negative_math_stays_deferred() {
    let bare = CssLengthCalculation::try_from_components(
        parse_component_values("2px").unwrap(),
        Default::default(),
    )
    .unwrap();
    let value = CssSpecifiedNonNegativeLength::try_from_calculation(bare).unwrap();
    assert!(value.literal_component().is_some());
    assert!(value.calculation().is_none());
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "2px"
    );
    let negative = CssLengthCalculation::try_from_components(
        parse_component_values("-1px").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert_numeric_rejected(
        CssSpecifiedNonNegativeLength::try_from_calculation(negative),
        NumericFailure::OutOfRange,
    );

    let symbolic = length("calc(1px - 2px)");
    assert!(symbolic.calculation().is_some());
    assert_eq!(
        symbolic.serialize_specified(Default::default()).unwrap(),
        "calc(-1px)"
    );
}

#[test]
fn fallback_and_auto_are_explicit_checked_structure() {
    let none = CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::None);
    let auto_none = CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::None);
    assert!(!none.uses_auto());
    assert!(auto_none.uses_auto());
    assert!(matches!(
        none.fallback(),
        CssContainIntrinsicSizeFallback::None
    ));
    assert_eq!(
        none.serialize_specified(Default::default()).unwrap(),
        "none"
    );
    assert_eq!(
        auto_none.serialize_specified(Default::default()).unwrap(),
        "auto none"
    );
    assert_ne!(none, auto_none);

    let auto_length = CssContainIntrinsicSizeValue::with_auto(
        CssContainIntrinsicSizeFallback::Length(length("2px")),
    );
    assert_eq!(
        auto_length.serialize_specified(Default::default()).unwrap(),
        "auto 2px"
    );
    assert!(matches!(
        auto_length.fallback(),
        CssContainIntrinsicSizeFallback::Length(value) if value.literal_component().is_some()
    ));
}

#[test]
fn semantic_equality_ignores_numeric_origins_but_raw_equality_does_not() {
    let first = length("2px");
    let later = CssSpecifiedNonNegativeLength::try_from_component(
        parse_component_values("  2px").unwrap().items()[1].clone(),
    )
    .unwrap();
    assert_ne!(first.origin(), later.origin());
    assert_ne!(first, later, "raw numeric equality retains provenance");
    let first =
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(first));
    let later =
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(later));
    assert_eq!(first, later);
    assert_ne!(first, length_value("2px"));
    assert_ne!(
        first,
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(length(
            "2PX"
        )))
    );
    assert_ne!(
        length_value("calc(1px + 2px)"),
        length_value("calc(1px - 2px)")
    );
    assert_ne!(
        length_value("calc(1px + 2px)"),
        length_value("calc((1px + 2px))")
    );
}

#[test]
fn shorthand_repeats_the_entire_first_value_and_retains_authored_height() {
    let auto_width = CssContainIntrinsicSizeValue::with_auto(
        CssContainIntrinsicSizeFallback::Length(length("3px")),
    );
    let one = CssContainIntrinsicSize::new(auto_width.clone(), None);
    assert_eq!(one.width(), one.height());
    assert!(one.height().uses_auto());
    assert!(one.authored_height().is_none());
    assert_eq!(
        one.serialize_specified(Default::default()).unwrap(),
        "auto 3px"
    );

    let two = CssContainIntrinsicSize::new(
        auto_width,
        Some(CssContainIntrinsicSizeValue::new(
            CssContainIntrinsicSizeFallback::None,
        )),
    );
    assert!(two.width().uses_auto());
    assert!(!two.height().uses_auto());
    assert!(matches!(
        two.height().fallback(),
        CssContainIntrinsicSizeFallback::None
    ));
    assert!(two.authored_height().is_some());
    assert_eq!(
        two.serialize_specified(Default::default()).unwrap(),
        "auto 3px none"
    );
    assert_ne!(one, two);

    let one_none = CssContainIntrinsicSize::new(
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::None),
        None,
    );
    assert_eq!(
        one_none
            .height()
            .serialize_specified(Default::default())
            .unwrap(),
        "auto none"
    );
}

#[test]
fn serialization_shares_input_projection_and_byte_budgets_across_whole_values() {
    let shorthand = CssContainIntrinsicSize::new(
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::None),
        Some(length_value("2px")),
    );
    assert_eq!(
        shorthand.serialize_specified(Default::default()).unwrap(),
        "auto none 2px"
    );
    assert!(
        shorthand
            .serialize_specified(CssSerializationLimits::new(2, 9, 99))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        shorthand
            .serialize_specified(CssSerializationLimits::new(9, 2, 99))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        shorthand
            .serialize_specified(CssSerializationLimits::new(9, 9, 12))
            .unwrap_err()
            .is_byte_limit()
    );
}
