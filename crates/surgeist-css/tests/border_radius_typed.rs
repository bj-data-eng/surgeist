#![forbid(unsafe_code)]

use surgeist_css::*;

fn scalar(text: &str) -> CssSpecifiedNonNegativeLengthPercentage {
    let components = parse_component_values(text).unwrap();
    if text.starts_with("calc(") {
        CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        let [component] = components.items() else {
            panic!("one radius scalar")
        };
        CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone()).unwrap()
    }
}

fn corner(name: &str, text: &str) -> CssCornerRadiusValue {
    let report = parse_style_attribute(&format!("{name}:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let value = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap();
    match value {
        CssKnownPropertyValueRef::BorderTopLeftRadius(v) => v.current().clone(),
        CssKnownPropertyValueRef::BorderStartStartRadius(v) => v.current().clone(),
        CssKnownPropertyValueRef::BorderStartEndRadius(v) => v.current().clone(),
        CssKnownPropertyValueRef::BorderEndStartRadius(v) => v.current().clone(),
        CssKnownPropertyValueRef::BorderEndEndRadius(v) => v.current().clone(),
        _ => panic!("corner value"),
    }
}

#[test]
fn checked_corner_preserves_omission_axes_origins_and_exact_literals() {
    let horizontal = CssSpecifiedNonNegativeLengthPercentage::try_from_component(
        CssComponentValue::try_dimension("1", "px").unwrap(),
    )
    .unwrap();
    let programmatic = CssCornerRadiusValue::new(horizontal, Some(scalar("2%")));
    let parsed = corner("border-top-left-radius", "1px 2%");
    assert_eq!(programmatic, parsed);
    assert!(matches!(
        programmatic.horizontal().origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(matches!(
        parsed.horizontal().origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        programmatic.horizontal().serialize_specified().unwrap(),
        "1px"
    );
    assert_eq!(programmatic.vertical().serialize_specified().unwrap(), "2%");
    assert_eq!(programmatic.serialize_specified().unwrap(), "1px 2%");
    let omitted = CssCornerRadiusValue::new(scalar("1px"), None);
    assert_eq!(omitted.horizontal(), omitted.vertical());
    assert!(omitted.authored_vertical().is_none());
    assert_ne!(
        omitted,
        CssCornerRadiusValue::new(scalar("1px"), Some(scalar("1px")))
    );
    assert_ne!(omitted, CssCornerRadiusValue::new(scalar("1.0px"), None));
    for text in ["1e100px", "1e100%", "1e-100px", "1e-100%", "-0px", "-0%"] {
        assert!(scalar(text).literal_component().is_some(), "{text}");
    }
    for text in ["-1e-100px", "-1e-100%", "1fr", "1"] {
        let values = parse_component_values(text).unwrap();
        let [component] = values.items() else {
            panic!("one invalid scalar")
        };
        assert!(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component.clone()).is_err(),
            "{text}"
        );
    }
}

#[test]
fn every_logical_name_keeps_horizontal_then_vertical_without_mapping() {
    for name in [
        "border-start-start-radius",
        "border-start-end-radius",
        "border-end-start-radius",
        "border-end-end-radius",
    ] {
        let value = corner(name, "1px 2%");
        assert_eq!(
            value.horizontal().serialize_specified().unwrap(),
            "1px",
            "{name}"
        );
        assert_eq!(
            value.vertical().serialize_specified().unwrap(),
            "2%",
            "{name}"
        );
        assert_eq!(value.serialize_specified().unwrap(), "1px 2%");
    }
}

#[test]
fn shorthand_retains_lists_slash_and_corner_assignment_for_every_arity() {
    assert!(CssBorderRadiusShorthand::try_new(vec![], None).is_none());
    assert!(CssBorderRadiusShorthand::try_new(vec![scalar("0"); 5], None).is_none());
    assert!(CssBorderRadiusShorthand::try_new(vec![scalar("0")], Some(vec![])).is_none());
    assert!(
        CssBorderRadiusShorthand::try_new(vec![scalar("0")], Some(vec![scalar("0"); 5])).is_none()
    );
    for (horizontal, expected) in [
        (vec![scalar("1px")], ["1px", "1px", "1px", "1px"]),
        (
            vec![scalar("1px"), scalar("2px")],
            ["1px", "2px", "1px", "2px"],
        ),
        (
            vec![scalar("1px"), scalar("2px"), scalar("3px")],
            ["1px", "2px", "3px", "2px"],
        ),
        (
            vec![scalar("1px"), scalar("2px"), scalar("3px"), scalar("4px")],
            ["1px", "2px", "3px", "4px"],
        ),
    ] {
        let value = CssBorderRadiusShorthand::try_new(horizontal, None).unwrap();
        assert!(value.authored_vertical_values().is_none());
        assert_eq!(value.horizontal_values(), value.vertical_values());
        assert_eq!(
            [
                value.top_left(),
                value.top_right(),
                value.bottom_right(),
                value.bottom_left(),
            ]
            .map(|corner| corner.serialize_specified().unwrap()),
            expected
        );
    }
    let value = CssBorderRadiusShorthand::try_new(
        vec![scalar("1px"), scalar("2px"), scalar("3px")],
        Some(vec![scalar("4%"), scalar("5%")]),
    )
    .unwrap();
    assert_eq!(value.horizontal_values().len(), 3);
    assert_eq!(value.authored_vertical_values().unwrap().len(), 2);
    assert_eq!(
        [
            value.top_left(),
            value.top_right(),
            value.bottom_right(),
            value.bottom_left(),
        ]
        .map(|corner| corner.serialize_specified().unwrap()),
        ["1px 4%", "2px 5%", "3px 4%", "2px 5%"]
    );
    assert_eq!(value.serialize_specified().unwrap(), "1px 2px 3px / 4% 5%");
    assert_ne!(
        CssBorderRadiusShorthand::try_new(vec![scalar("1px")], None).unwrap(),
        CssBorderRadiusShorthand::try_new(vec![scalar("1px")], Some(vec![scalar("1px")])).unwrap()
    );
}

#[test]
fn specified_serialization_charges_one_aggregate_and_every_authored_child() {
    let corner = CssCornerRadiusValue::new(scalar("1px"), Some(scalar("2%")));
    assert_eq!(
        corner
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 6))
            .unwrap(),
        "1px 2%"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            corner
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let shorthand = CssBorderRadiusShorthand::try_new(
        vec![scalar("1px"), scalar("2px")],
        Some(vec![scalar("3%"), scalar("4%")]),
    )
    .unwrap();
    assert_eq!(
        shorthand
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(5, 5, 15))
            .unwrap(),
        "1px 2px / 3% 4%"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, 15),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, 15),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, 14),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            shorthand
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn symbolic_math_remains_typed_and_shares_aggregate_limits() {
    let math = scalar("calc(1px + 2%)");
    let original = math.calculation().unwrap().clone();
    let value = CssCornerRadiusValue::new(math, Some(scalar("3px")));
    assert_eq!(value.serialize_specified().unwrap(), "calc(2% + 1px) 3px");
    assert_eq!(
        value.horizontal().calculation().unwrap().components(),
        original.components()
    );
    assert_eq!(
        value.horizontal().calculation().unwrap().numeric_type(),
        original.numeric_type()
    );
    let rendered = "calc(2% + 1px) 3px";
    // The numeric projector retains the function and both operands; the corner
    // additionally charges its aggregate and second authored literal.
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                6,
                7,
                rendered.len()
            ))
            .unwrap(),
        rendered
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 7, rendered.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 6, rendered.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 7, rendered.len() - 1),
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
    }
}

#[test]
fn physical_compatibility_projection_is_exact_and_preserves_math_type() {
    for (text, expected) in [
        ("1px", true),
        ("16777216px", true),
        ("-0%", true),
        ("1e100px", false),
        ("1e-100%", false),
        ("16777216.00000000001px", false),
    ] {
        let report = parse_style_attribute(&format!("border-top-left-radius:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderTopLeftRadius(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("physical corner")
        };
        assert_eq!(wrapper.i01_subset().is_some(), expected, "{text}");
    }
    let report = parse_style_attribute("border-top-left-radius:calc(1px + 2%)");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderTopLeftRadius(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("math corner")
    };
    let original = wrapper.current().horizontal().calculation().unwrap();
    let CssLength::Calc(CssCalcLength::Typed(legacy)) = wrapper.i01_subset().unwrap().horizontal()
    else {
        panic!("legacy typed math")
    };
    assert_eq!(legacy.components(), original.components());
    assert_eq!(legacy.numeric_type(), original.numeric_type());
    assert_eq!(legacy.origin(), original.origin());
    let report = parse_style_attribute("border-radius:1e100px / 2%");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderRadius(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("physical shorthand")
    };
    assert!(wrapper.i01_subset().is_none());
    assert_eq!(
        wrapper
            .current()
            .top_left()
            .horizontal()
            .serialize_specified()
            .unwrap(),
        format!("1{}px", "0".repeat(100))
    );
}
