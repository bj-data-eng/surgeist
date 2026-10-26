use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::property_admission::{assert_component_rejected, assert_numeric_rejected};
use surgeist_css::{
    CssCalculation, CssComponentValue, CssComponentValues, CssContainIntrinsicSizeFallback,
    CssContainIntrinsicSizeValue, CssLength, CssNonNegative, CssNumeric, CssParser,
    CssSerializationLimits, CssSidePair, CssValueOrigin,
};

fn component(text: &str) -> CssComponentValue {
    let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let [component] = values.items() else {
        panic!("one component: {text}")
    };
    component.clone()
}

fn length(text: &str) -> CssNumeric<CssLength, CssNonNegative> {
    if text.starts_with("calc(") {
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLength>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(component(text)).unwrap()
    }
}

fn length_value(text: &str) -> CssContainIntrinsicSizeValue {
    CssContainIntrinsicSizeValue::new(CssContainIntrinsicSizeFallback::Length(length(text)))
}

#[test]
fn checked_nonnegative_length_preserves_exact_literal_domain() {
    for text in ["0", "-0", "0px", "-0px", "1px", "1e999px", "1e-999px"] {
        let value = length(text);
        assert!(value.literal().map(|literal| literal.component()).is_some());
        assert_eq!(
            value
                .literal()
                .map(|literal| literal.component())
                .unwrap()
                .origin(),
            value.origin()
        );
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    }
    assert_eq!(
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_number("0").unwrap()
        )
        .unwrap()
        .serialize_specified(Default::default())
        .unwrap(),
        "0"
    );
    let negative_zero =
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(component("-0px")).unwrap();
    let surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension {
        number,
        ..
    }) = negative_zero
        .literal()
        .map(|literal| literal.component())
        .unwrap()
        .view()
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
            CssNumeric::<CssLength, CssNonNegative>::try_from_component,
            component(text),
            kind,
        );
    }
}

#[test]
fn bare_calculation_roots_reenter_ordinary_admission_but_negative_math_stays_deferred() {
    let bare = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "2px").unwrap(),
        Default::default(),
    )
    .unwrap();
    let value = CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(bare).unwrap();
    assert!(value.literal().map(|literal| literal.component()).is_some());
    assert!(value.calculation().is_none());
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "2px"
    );
    let negative = CssCalculation::<CssLength>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "-1px").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert_numeric_rejected(
        CssNumeric::<CssLength, CssNonNegative>::try_from_calculation(negative),
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
        CssContainIntrinsicSizeFallback::Length(value) if value.literal().map(|literal| literal.component()).is_some()
    ));
}

#[test]
fn semantic_and_raw_equality_ignore_numeric_origins() {
    let first = length("2px");
    let later = CssNumeric::<CssLength, CssNonNegative>::try_from_component(
        CssComponentValues::parse(&CssParser::default(), "  2px")
            .unwrap()
            .items()[1]
            .clone(),
    )
    .unwrap();
    assert_ne!(first.origin(), later.origin());
    assert_eq!(first, later, "numeric value equality ignores provenance");
    let first =
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(first));
    let later =
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(later));
    assert_eq!(first, later);
    assert_ne!(first, length_value("2px"));
    assert_eq!(
        first,
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::Length(length(
            "2PX"
        )))
    );
    assert_ne!(
        length_value("calc(1px + 2px)"),
        length_value("calc(1px - 2px)")
    );
    // Redundant grouping is not a calculation node (Values 4 parse a calculation).
    assert_eq!(
        length_value("calc(1px + 2px)"),
        length_value("calc((1px + 2px))")
    );
}

#[test]
fn shorthand_repeats_the_entire_first_value_and_retains_authored_height() {
    let auto_width = CssContainIntrinsicSizeValue::with_auto(
        CssContainIntrinsicSizeFallback::Length(length("3px")),
    );
    let one = CssSidePair::new(auto_width.clone(), None);
    assert_eq!(one.first(), one.second());
    assert!(one.second().uses_auto());
    assert!(one.authored_second().is_none());
    assert_eq!(
        one.serialize_specified(Default::default()).unwrap(),
        "auto 3px"
    );

    let two = CssSidePair::new(
        auto_width,
        Some(CssContainIntrinsicSizeValue::new(
            CssContainIntrinsicSizeFallback::None,
        )),
    );
    assert!(two.first().uses_auto());
    assert!(!two.second().uses_auto());
    assert!(matches!(
        two.second().fallback(),
        CssContainIntrinsicSizeFallback::None
    ));
    assert!(two.authored_second().is_some());
    assert_eq!(
        two.serialize_specified(Default::default()).unwrap(),
        "auto 3px none"
    );
    assert_ne!(one, two);

    let one_none = CssSidePair::new(
        CssContainIntrinsicSizeValue::with_auto(CssContainIntrinsicSizeFallback::None),
        None,
    );
    assert_eq!(
        one_none
            .second()
            .serialize_specified(Default::default())
            .unwrap(),
        "auto none"
    );
}

#[test]
fn serialization_shares_input_projection_and_byte_budgets_across_whole_values() {
    let shorthand = CssSidePair::new(
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
