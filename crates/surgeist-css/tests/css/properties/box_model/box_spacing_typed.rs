use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::property_admission::assert_component_rejected;
use surgeist_css::{
    CssBoxSideKind, CssCalculation, CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssFourSides, CssLengthPercentage, CssMarginValue, CssNonNegative, CssNumeric, CssPaddingValue,
    CssParser, CssSerializationLimits, CssSidePair, CssValueOrigin, CssValueTokenRef,
};

fn component(text: &str) -> CssComponentValue {
    let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let [component] = values.items() else {
        panic!("one component: {text}")
    };
    component.clone()
}

fn margin(text: &str) -> CssMarginValue {
    if text.eq_ignore_ascii_case("auto") {
        CssMarginValue::Auto
    } else if text.starts_with("calc(") {
        CssMarginValue::LengthPercentage(
            CssNumeric::<CssLengthPercentage>::try_from_calculation(
                CssCalculation::<CssLengthPercentage>::try_from_components(
                    CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                    Default::default(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    } else {
        CssMarginValue::LengthPercentage(
            CssNumeric::<CssLengthPercentage>::try_from_component(component(text)).unwrap(),
        )
    }
}

fn padding(text: &str) -> CssPaddingValue {
    let value = if text.starts_with("calc(") {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(text))
            .unwrap()
    };
    CssPaddingValue::new(value)
}

#[test]
fn signed_length_percentage_keeps_exact_extremes_and_literal_domain() {
    for text in [
        "-1e999px",
        "-1e999%",
        "-1e-999px",
        "-1e-999%",
        "0",
        "0px",
        "0%",
    ] {
        let value = CssNumeric::<CssLengthPercentage>::try_from_component(component(text)).unwrap();
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
        CssNumeric::<CssLengthPercentage>::try_from_component(
            CssComponentValue::try_number("0").unwrap()
        )
        .unwrap()
        .serialize_specified(Default::default())
        .unwrap(),
        "0"
    );
    assert_eq!(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_number("0").unwrap()
        )
        .unwrap()
        .serialize_specified(Default::default())
        .unwrap(),
        "0"
    );
    let huge = margin("-1e999px")
        .serialize_specified(Default::default())
        .unwrap();
    assert_eq!(huge.len(), 1003); // Minus, 1 followed by 999 zeroes, px.
    assert!(huge.starts_with("-1") && huge.ends_with("px"));
    let tiny = margin("-1e-999%");
    let before = tiny.clone();
    assert_eq!(tiny.serialize_specified(Default::default()).unwrap(), "0%");
    assert!(matches!(
        tiny.length_percentage().unwrap().literal().map(|literal| literal.component()).unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "-1e-999"
    ));
    assert_eq!(tiny, before);
    for text in ["1", "1fr"] {
        assert_component_rejected(
            CssNumeric::<CssLengthPercentage>::try_from_component,
            component(text),
            NumericFailure::RootDomainMismatch,
        );
    }
    assert_component_rejected(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component,
        component("-1e-999%"),
        NumericFailure::OutOfRange,
    );
}

#[test]
fn signed_math_stays_deferred_and_padding_wraps_checked_nonnegative_math() {
    let signed = margin("calc(1px - 2%)");
    assert!(signed.length_percentage().unwrap().calculation().is_some());
    assert_eq!(
        signed.serialize_specified(Default::default()).unwrap(),
        "calc(-2% + 1px)"
    );
    let nonnegative = padding("calc(1px - 2%)");
    assert!(nonnegative.length_percentage().calculation().is_some());
    assert_eq!(
        nonnegative.serialize_specified(Default::default()).unwrap(),
        "calc(-2% + 1px)"
    );
    assert!(matches!(nonnegative.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        CssMarginValue::Auto
            .serialize_specified(Default::default())
            .unwrap(),
        "auto"
    );
    assert!(CssMarginValue::Auto.length_percentage().is_none());
    assert!(CssMarginValue::Auto.origin().is_none());
}

#[test]
fn typed_margin_and_padding_equality_excludes_origin_but_retains_structure() {
    let first = margin("1px");
    let later = CssComponentValues::parse(&CssParser::default(), "  1px")
        .unwrap()
        .items()[1]
        .clone();
    let later = CssMarginValue::LengthPercentage(
        CssNumeric::<CssLengthPercentage>::try_from_component(later).unwrap(),
    );
    assert_ne!(first.origin(), later.origin());
    assert_eq!(first, later);
    assert_eq!(first, margin("1PX"));
    assert_eq!(first, margin("1.0px"));
    assert_ne!(margin("calc(1px + 2%)"), margin("calc(1px - 2%)"));
    // Redundant grouping is not a calculation node (Values 4 parse a calculation).
    assert_eq!(margin("calc(1px + 2%)"), margin("calc((1px + 2%))"));
    assert_ne!(margin("auto"), margin("0"));

    assert_eq!(padding("1px"), padding("1px"));
    let shifted_padding = CssPaddingValue::new(
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
            CssComponentValues::parse(&CssParser::default(), "  1px")
                .unwrap()
                .items()[1]
                .clone(),
        )
        .unwrap(),
    );
    assert_ne!(padding("1px").origin(), shifted_padding.origin());
    assert_eq!(padding("1px"), shifted_padding);
    assert_eq!(padding("1px"), padding("1PX"));
    assert_ne!(padding("calc(1px + 2%)"), padding("calc(1px - 2%)"));
    let raw_left = CssNumeric::<CssLengthPercentage>::try_from_component(component("1px")).unwrap();
    let raw_right = CssNumeric::<CssLengthPercentage>::try_from_component(
        CssComponentValues::parse(&CssParser::default(), "  1px")
            .unwrap()
            .items()[1]
            .clone(),
    )
    .unwrap();
    assert_eq!(raw_left, raw_right, "numeric value equality ignores origin");
}

#[test]
fn logical_pairs_keep_authored_cardinality_and_repeat_start_semantically() {
    let one = CssSidePair::new(margin("auto"), None);
    assert_eq!(one.first(), one.second());
    assert!(one.authored_second().is_none());
    assert_eq!(one.serialize_specified(Default::default()).unwrap(), "auto");
    let two = CssSidePair::new(margin("auto"), Some(margin("auto")));
    assert_eq!(two.first(), two.second());
    assert_ne!(
        one, two,
        "authored one and two values are different structures"
    );
    assert_eq!(two.serialize_specified(Default::default()).unwrap(), "auto");

    let padding_one = CssSidePair::new(padding("1px"), None);
    assert_eq!(padding_one.second(), padding_one.first());
    assert_eq!(
        padding_one.serialize_specified(Default::default()).unwrap(),
        "1px"
    );
    let padding_two = CssSidePair::new(padding("1px"), Some(padding("2%")));
    assert_eq!(
        padding_two
            .authored_second()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "2%"
    );
    assert_eq!(
        padding_two.serialize_specified(Default::default()).unwrap(),
        "1px 2%"
    );
}

#[test]
fn four_side_shorthands_preserve_kind_cardinality_and_exact_role_order() {
    let one = CssFourSides::try_new(CssBoxSideKind::Physical, vec![margin("1px")]).unwrap();
    assert_eq!(
        one.assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["1px", "1px", "1px", "1px"]
    );
    let two = CssFourSides::try_new(CssBoxSideKind::Logical, ["1px", "2px"].map(margin).to_vec())
        .unwrap();
    assert_eq!(
        two.assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["1px", "2px", "1px", "2px"]
    );
    let physical = CssFourSides::try_new(
        CssBoxSideKind::Physical,
        ["1px", "2px", "3px", "4px"].map(margin).to_vec(),
    )
    .unwrap();
    let logical = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2px", "3px", "4px"].map(margin).to_vec(),
    )
    .unwrap();
    assert_eq!(physical.kind(), CssBoxSideKind::Physical);
    assert_eq!(logical.kind(), CssBoxSideKind::Logical);
    assert_eq!(physical.authored().len(), 4);
    assert_ne!(physical, logical);
    assert_eq!(
        physical.serialize_specified(Default::default()).unwrap(),
        "1px 2px 3px 4px"
    );
    assert_eq!(
        logical.serialize_specified(Default::default()).unwrap(),
        "logical 1px 2px 3px 4px"
    );
    // Physical assignments are top, right, bottom, left. Logical assignments
    // are block-start, inline-start, block-end, inline-end (§4.7).
    for assignments in [physical.assigned(), logical.assigned()] {
        assert_eq!(
            assignments.map(|value| value.serialize_specified(Default::default()).unwrap()),
            ["1px", "2px", "3px", "4px"]
        );
    }
    let three = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2px", "3px"].map(margin).to_vec(),
    )
    .unwrap();
    assert_eq!(
        three
            .assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["1px", "2px", "3px", "2px"]
    );
    assert_ne!(three, logical);
    assert!(CssFourSides::<CssMarginValue>::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssFourSides::try_new(CssBoxSideKind::Logical, vec![margin("0"); 5]).is_none());

    let padding_shorthand = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2%", "3px", "4%"].map(padding).to_vec(),
    )
    .unwrap();
    assert_eq!(padding_shorthand.kind(), CssBoxSideKind::Logical);
    assert_eq!(padding_shorthand.authored().len(), 4);
    assert_eq!(
        padding_shorthand
            .assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["1px", "2%", "3px", "4%"]
    );
    let one_padding = CssFourSides::try_new(CssBoxSideKind::Logical, vec![padding("5px")]).unwrap();
    assert_eq!(
        one_padding
            .assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["5px", "5px", "5px", "5px"]
    );
    let two_padding = CssFourSides::try_new(
        CssBoxSideKind::Physical,
        vec![padding("5px"), padding("6%")],
    )
    .unwrap();
    assert_eq!(
        two_padding
            .assigned()
            .map(|value| value.serialize_specified(Default::default()).unwrap()),
        ["5px", "6%", "5px", "6%"]
    );
    assert_eq!(
        padding_shorthand
            .serialize_specified(Default::default())
            .unwrap(),
        "logical 1px 2% 3px 4%"
    );
    assert!(CssFourSides::<CssMarginValue>::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssFourSides::try_new(CssBoxSideKind::Logical, vec![padding("0"); 5]).is_none());
}

#[test]
fn combined_serialization_budgets_cover_every_child_and_logical_switch() {
    let pair = CssSidePair::new(margin("1px"), Some(margin("2px")));
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(1, 9, 99))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        pair.serialize_specified(CssSerializationLimits::new(9, 1, 99))
            .unwrap_err()
            .is_projection_node_limit()
    );
    let logical = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        vec![padding("1px"), padding("2px")],
    )
    .unwrap();
    assert!(
        logical
            .serialize_specified(CssSerializationLimits::new(2, 9, 99))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        logical
            .serialize_specified(CssSerializationLimits::new(9, 2, 99))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        logical
            .serialize_specified(CssSerializationLimits::new(9, 9, 14))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(
        logical.serialize_specified(Default::default()).unwrap(),
        "logical 1px 2px"
    );
}
