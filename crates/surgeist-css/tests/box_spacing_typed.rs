#![forbid(unsafe_code)]

use surgeist_css::{
    CssBoxSideKind, CssComponentValue, CssLengthPercentageCalculation, CssMarginPair,
    CssMarginShorthand, CssMarginValue, CssPaddingPair, CssPaddingShorthand, CssPaddingValue,
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    parse_component_values,
};

fn component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
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
            CssSpecifiedLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::try_from_components(
                    parse_component_values(text).unwrap(),
                )
                .unwrap(),
            )
            .unwrap(),
        )
    } else {
        CssMarginValue::LengthPercentage(
            CssSpecifiedLengthPercentage::try_from_component(component(text)).unwrap(),
        )
    }
}

fn padding(text: &str) -> CssPaddingValue {
    let value = if text.starts_with("calc(") {
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(
                parse_component_values(text).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component(text)).unwrap()
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
        let value = CssSpecifiedLengthPercentage::try_from_component(component(text)).unwrap();
        assert_eq!(value.literal_component().unwrap().origin(), value.origin());
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    }
    assert_eq!(
        CssSpecifiedLengthPercentage::zero()
            .serialize_specified()
            .unwrap(),
        "0"
    );
    assert_eq!(
        CssSpecifiedNonNegativeLengthPercentage::zero()
            .serialize_specified()
            .unwrap(),
        "0"
    );
    let huge = margin("-1e999px").serialize_specified().unwrap();
    assert_eq!(huge.len(), 1003); // Minus, 1 followed by 999 zeroes, px.
    assert!(huge.starts_with("-1") && huge.ends_with("px"));
    let tiny = margin("-1e-999%").serialize_specified().unwrap();
    assert!(tiny.starts_with("-0.") && tiny.ends_with("1%"));
    assert!(CssSpecifiedLengthPercentage::try_from_component(component("1")).is_err());
    assert!(CssSpecifiedLengthPercentage::try_from_component(component("1fr")).is_err());
    assert!(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component("-1e-999%")).is_err()
    );
}

#[test]
fn signed_math_stays_deferred_and_padding_wraps_checked_nonnegative_math() {
    let signed = margin("calc(1px - 2%)");
    assert!(signed.length_percentage().unwrap().calculation().is_some());
    assert_eq!(signed.serialize_specified().unwrap(), "calc(-2% + 1px)");
    let nonnegative = padding("calc(1px - 2%)");
    assert!(nonnegative.length_percentage().calculation().is_some());
    assert_eq!(
        nonnegative.serialize_specified().unwrap(),
        "calc(-2% + 1px)"
    );
    assert!(matches!(nonnegative.origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(CssMarginValue::Auto.serialize_specified().unwrap(), "auto");
    assert!(CssMarginValue::Auto.length_percentage().is_none());
    assert!(CssMarginValue::Auto.origin().is_none());
}

#[test]
fn typed_margin_and_padding_equality_excludes_origin_but_retains_structure() {
    let first = margin("1px");
    let later = parse_component_values("  1px").unwrap().items()[1].clone();
    let later = CssMarginValue::LengthPercentage(
        CssSpecifiedLengthPercentage::try_from_component(later).unwrap(),
    );
    assert_ne!(first.origin(), later.origin());
    assert_eq!(first, later);
    assert_ne!(first, margin("1PX"));
    assert_ne!(first, margin("1.0px"));
    assert_ne!(margin("calc(1px + 2%)"), margin("calc(1px - 2%)"));
    assert_ne!(margin("calc(1px + 2%)"), margin("calc((1px + 2%))"));
    assert_ne!(margin("auto"), margin("0"));

    assert_eq!(padding("1px"), padding("1px"));
    let shifted_padding = CssPaddingValue::new(
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            parse_component_values("  1px").unwrap().items()[1].clone(),
        )
        .unwrap(),
    );
    assert_ne!(padding("1px").origin(), shifted_padding.origin());
    assert_eq!(padding("1px"), shifted_padding);
    assert_ne!(padding("1px"), padding("1PX"));
    assert_ne!(padding("calc(1px + 2%)"), padding("calc(1px - 2%)"));
    let raw_left = CssSpecifiedLengthPercentage::try_from_component(component("1px")).unwrap();
    let raw_right = CssSpecifiedLengthPercentage::try_from_component(
        parse_component_values("  1px").unwrap().items()[1].clone(),
    )
    .unwrap();
    assert_ne!(raw_left, raw_right, "ordinary raw equality retains origin");
}

#[test]
fn logical_pairs_keep_authored_cardinality_and_repeat_start_semantically() {
    let one = CssMarginPair::new(margin("auto"), None);
    assert_eq!(one.start(), one.end());
    assert!(one.authored_end().is_none());
    assert_eq!(one.serialize_specified().unwrap(), "auto");
    let two = CssMarginPair::new(margin("auto"), Some(margin("auto")));
    assert_eq!(two.start(), two.end());
    assert_ne!(
        one, two,
        "authored one and two values are different structures"
    );
    assert_eq!(two.serialize_specified().unwrap(), "auto auto");

    let padding_one = CssPaddingPair::new(padding("1px"), None);
    assert_eq!(padding_one.end(), padding_one.start());
    assert_eq!(padding_one.serialize_specified().unwrap(), "1px");
    let padding_two = CssPaddingPair::new(padding("1px"), Some(padding("2%")));
    assert_eq!(
        padding_two
            .authored_end()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "2%"
    );
    assert_eq!(padding_two.serialize_specified().unwrap(), "1px 2%");
}

#[test]
fn four_side_shorthands_preserve_kind_cardinality_and_exact_role_order() {
    let one = CssMarginShorthand::try_new(CssBoxSideKind::Physical, vec![margin("1px")]).unwrap();
    assert_eq!(
        one.assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["1px", "1px", "1px", "1px"]
    );
    let two =
        CssMarginShorthand::try_new(CssBoxSideKind::Logical, ["1px", "2px"].map(margin).to_vec())
            .unwrap();
    assert_eq!(
        two.assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["1px", "2px", "1px", "2px"]
    );
    let physical = CssMarginShorthand::try_new(
        CssBoxSideKind::Physical,
        ["1px", "2px", "3px", "4px"].map(margin).to_vec(),
    )
    .unwrap();
    let logical = CssMarginShorthand::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2px", "3px", "4px"].map(margin).to_vec(),
    )
    .unwrap();
    assert_eq!(physical.kind(), CssBoxSideKind::Physical);
    assert_eq!(logical.kind(), CssBoxSideKind::Logical);
    assert_eq!(physical.authored_values().len(), 4);
    assert_ne!(physical, logical);
    assert_eq!(physical.serialize_specified().unwrap(), "1px 2px 3px 4px");
    assert_eq!(
        logical.serialize_specified().unwrap(),
        "logical 1px 2px 3px 4px"
    );
    // Physical assignments are top, right, bottom, left. Logical assignments
    // are block-start, inline-start, block-end, inline-end (§4.7).
    for assignments in [physical.assigned_values(), logical.assigned_values()] {
        assert_eq!(
            assignments.map(|value| value.serialize_specified().unwrap()),
            ["1px", "2px", "3px", "4px"]
        );
    }
    let three = CssMarginShorthand::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2px", "3px"].map(margin).to_vec(),
    )
    .unwrap();
    assert_eq!(
        three
            .assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["1px", "2px", "3px", "2px"]
    );
    assert_ne!(three, logical);
    assert!(CssMarginShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssMarginShorthand::try_new(CssBoxSideKind::Logical, vec![margin("0"); 5]).is_none());

    let padding_shorthand = CssPaddingShorthand::try_new(
        CssBoxSideKind::Logical,
        ["1px", "2%", "3px", "4%"].map(padding).to_vec(),
    )
    .unwrap();
    assert_eq!(padding_shorthand.kind(), CssBoxSideKind::Logical);
    assert_eq!(padding_shorthand.authored_values().len(), 4);
    assert_eq!(
        padding_shorthand
            .assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["1px", "2%", "3px", "4%"]
    );
    let one_padding =
        CssPaddingShorthand::try_new(CssBoxSideKind::Logical, vec![padding("5px")]).unwrap();
    assert_eq!(
        one_padding
            .assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["5px", "5px", "5px", "5px"]
    );
    let two_padding = CssPaddingShorthand::try_new(
        CssBoxSideKind::Physical,
        vec![padding("5px"), padding("6%")],
    )
    .unwrap();
    assert_eq!(
        two_padding
            .assigned_values()
            .map(|value| value.serialize_specified().unwrap()),
        ["5px", "6%", "5px", "6%"]
    );
    assert_eq!(
        padding_shorthand.serialize_specified().unwrap(),
        "logical 1px 2% 3px 4%"
    );
    assert!(CssPaddingShorthand::try_new(CssBoxSideKind::Physical, vec![]).is_none());
    assert!(CssPaddingShorthand::try_new(CssBoxSideKind::Logical, vec![padding("0"); 5]).is_none());
}

#[test]
fn combined_serialization_budgets_cover_every_child_and_logical_switch() {
    let pair = CssMarginPair::new(margin("1px"), Some(margin("2px")));
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 9, 99))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        pair.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(9, 1, 99))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    let logical = CssPaddingShorthand::try_new(
        CssBoxSideKind::Logical,
        vec![padding("1px"), padding("2px")],
    )
    .unwrap();
    assert_eq!(
        logical
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 9, 99))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        logical
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(9, 2, 99))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        logical
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(9, 9, 14))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(logical.serialize_specified().unwrap(), "logical 1px 2px");
}
