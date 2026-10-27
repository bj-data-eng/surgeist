use crate::css::support::failure_classes::NumericFailure;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::property_admission::assert_component_rejected;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::numeric::{
    CssCalculation, CssLengthPercentage, CssNonNegative, CssNumeric, CssNumericDimension,
};
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::border::CssBorderRadiusShorthand;
use surgeist_css::values::common::CssSidePair;

fn scalar(text: &str) -> CssNumeric<CssLengthPercentage, CssNonNegative> {
    let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    if text.starts_with("calc(") {
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_calculation(
            CssCalculation::<CssLengthPercentage>::try_from_components(
                components,
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    } else {
        let [component] = components.items() else {
            panic!("one radius scalar")
        };
        CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component.clone())
            .unwrap()
    }
}

fn corner(name: &str, text: &str) -> CssSidePair<CssNumeric<CssLengthPercentage, CssNonNegative>> {
    let report = CssParser::default().parse_block_contents(format!("{name}:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let value = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap();
    match value {
        CssKnownPropertyValueRef::BorderTopLeftRadius(v) => v.value().clone(),
        CssKnownPropertyValueRef::BorderStartStartRadius(v) => v.value().clone(),
        CssKnownPropertyValueRef::BorderStartEndRadius(v) => v.value().clone(),
        CssKnownPropertyValueRef::BorderEndStartRadius(v) => v.value().clone(),
        CssKnownPropertyValueRef::BorderEndEndRadius(v) => v.value().clone(),
        _ => panic!("corner value"),
    }
}

#[test]
fn checked_corner_preserves_omission_axes_origins_and_exact_literals() {
    let horizontal = CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
        CssComponentValue::try_dimension("1", "px").unwrap(),
    )
    .unwrap();
    let programmatic = CssSidePair::new(horizontal, Some(scalar("2%")));
    let parsed = corner("border-top-left-radius", "1px 2%");
    assert_eq!(programmatic, parsed);
    assert!(matches!(
        programmatic.first().origin(),
        CssValueOrigin::Programmatic
    ));
    assert!(matches!(parsed.first().origin(), CssValueOrigin::Parsed(_)));
    assert_eq!(
        programmatic
            .first()
            .serialize_specified(Default::default())
            .unwrap(),
        "1px"
    );
    assert_eq!(
        programmatic
            .second()
            .serialize_specified(Default::default())
            .unwrap(),
        "2%"
    );
    assert_eq!(
        programmatic
            .serialize_specified(Default::default())
            .unwrap(),
        "1px 2%"
    );
    let omitted = CssSidePair::new(scalar("1px"), None);
    assert_eq!(omitted.first(), omitted.second());
    assert!(omitted.authored_second().is_none());
    assert_ne!(
        omitted,
        CssSidePair::new(scalar("1px"), Some(scalar("1px")))
    );
    assert_eq!(omitted, CssSidePair::new(scalar("1.0px"), None));
    for text in ["1e100px", "1e100%", "1e-100px", "1e-100%", "-0px", "-0%"] {
        assert!(
            scalar(text)
                .literal()
                .map(|literal| literal.component())
                .is_some(),
            "{text}"
        );
    }
    for (text, kind) in [
        ("-1e-100px", NumericFailure::OutOfRange),
        ("-1e-100%", NumericFailure::OutOfRange),
        ("1fr", NumericFailure::RootDomainMismatch),
        ("1", NumericFailure::RootDomainMismatch),
    ] {
        let values = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let [component] = values.items() else {
            panic!("one invalid scalar")
        };
        assert_component_rejected(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component,
            component.clone(),
            kind,
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
            value
                .first()
                .serialize_specified(Default::default())
                .unwrap(),
            "1px",
            "{name}"
        );
        assert_eq!(
            value
                .second()
                .serialize_specified(Default::default())
                .unwrap(),
            "2%",
            "{name}"
        );
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            "1px 2%"
        );
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
            .map(|corner| corner.serialize_specified(Default::default()).unwrap()),
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
        .map(|corner| corner.serialize_specified(Default::default()).unwrap()),
        ["1px 4%", "2px 5%", "3px 4%", "2px 5%"]
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "1px 2px 3px / 4% 5%"
    );
    assert_ne!(
        CssBorderRadiusShorthand::try_new(vec![scalar("1px")], None).unwrap(),
        CssBorderRadiusShorthand::try_new(vec![scalar("1px")], Some(vec![scalar("1px")])).unwrap()
    );
}

#[test]
fn specified_serialization_charges_one_aggregate_and_every_authored_child() {
    let corner = CssSidePair::new(scalar("1px"), Some(scalar("2%")));
    assert_eq!(
        corner
            .serialize_specified(CssSerializationLimits::new(3, 3, 6))
            .unwrap(),
        "1px 2%"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 6),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 6),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 5),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&corner.serialize_specified(limits).unwrap_err()),
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
            .serialize_specified(CssSerializationLimits::new(5, 5, 15))
            .unwrap(),
        "1px 2px / 3% 4%"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(4, 5, 15),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 4, 15),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 5, 14),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&shorthand.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn symbolic_math_remains_typed_and_shares_aggregate_limits() {
    let math = scalar("calc(1px + 2%)");
    let original = math.calculation().unwrap().clone();
    let value = CssSidePair::new(math, Some(scalar("3px")));
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "calc(2% + 1px) 3px"
    );
    assert_eq!(
        value.first().calculation().unwrap().components(),
        original.components()
    );
    assert_eq!(
        value.first().calculation().unwrap().numeric_type(),
        original.numeric_type()
    );
    let rendered = "calc(2% + 1px) 3px";
    // The numeric projector retains the function and both operands; the corner
    // additionally charges its aggregate and second authored literal.
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(6, 7, rendered.len()))
            .unwrap(),
        rendered
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(5, 7, rendered.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(6, 6, rendered.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(6, 7, rendered.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
}

#[test]
fn physical_corners_preserve_exact_magnitudes_and_math_type() {
    for text in [
        "1px",
        "16777216px",
        "-0%",
        "1e100px",
        "1e-100%",
        "16777216.00000000001px",
    ] {
        let report =
            CssParser::default().parse_block_contents(format!("border-top-left-radius:{text}"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderTopLeftRadius(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("physical corner")
        };
        let horizontal = wrapper.value().first();
        let component = horizontal
            .literal()
            .map(|literal| literal.component())
            .expect("literal radius");
        match component.view() {
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                assert_eq!(unit, "px");
                assert_eq!(number.representation(), text.strip_suffix("px").unwrap());
            }
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
                assert_eq!(number.representation(), text.strip_suffix('%').unwrap());
            }
            _ => panic!("exact radius token"),
        }
        assert!(wrapper.value().authored_second().is_none());
        let CssValueOrigin::Parsed(origin) = horizontal.origin() else {
            panic!("original declaration origin")
        };
        assert_eq!(
            origin.source().as_str(),
            format!("border-top-left-radius:{text}")
        );
    }
    let report = CssParser::default().parse_block_contents("border-top-left-radius:calc(1px + 2%)");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderTopLeftRadius(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("math corner")
    };
    let original = wrapper.value().first().calculation().unwrap();
    assert_eq!(
        original.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert!(matches!(original.origin(), CssValueOrigin::Parsed(_)));
    let report = CssParser::default().parse_block_contents("border-radius:1e100px / 2%");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::BorderRadius(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("physical shorthand")
    };
    assert_eq!(
        wrapper
            .value()
            .top_left()
            .first()
            .serialize_specified(Default::default())
            .unwrap(),
        format!("1{}px", "0".repeat(100))
    );
}
