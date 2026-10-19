#![forbid(unsafe_code)]
//! Mechanical shared-angle construction and existing Images 3 serialization.
//! This retains exact ordinary coefficients, four units, grammar-selected zero, symbolic
//! Angle-root math, contextual direction omission, and cumulative budgets.
//! https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#serialization

use surgeist_css::*;

fn zero_angle() -> CssAngleOrZero {
    CssAngleOrZero::Zero(
        CssZeroLiteral::try_from_component(CssComponentValue::try_number("0").unwrap()).unwrap(),
    )
}

fn angle(value: f32, unit: CssAngleUnit) -> CssAngleOrZero {
    CssAngleOrZero::Angle(CssAngleValue::from_literal(
        CssAngleLiteral::try_new(&value.to_string(), unit).unwrap(),
    ))
}

fn angle_math(text: &str) -> CssAngleOrZero {
    CssAngleOrZero::Angle(
        CssAngleValue::try_from_calculation(
            CssAngleCalculation::try_from_components(parse_component_values(text).unwrap())
                .unwrap(),
        )
        .unwrap(),
    )
}

fn number(text: &str) -> CssSpecifiedNumber {
    CssSpecifiedNumber::try_from_component(CssComponentValue::try_number(text).unwrap()).unwrap()
}

fn color(text: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value().clone()
}

fn gradient(angle: CssAngleOrZero) -> CssGradient {
    let stops = CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            color("red"),
            None,
        ))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            color("blue"),
            None,
        ))),
    ])
    .unwrap();
    CssGradient::Linear(CssLinearGradient::new(
        Some(CssLinearGradientDirection::Angle(angle)),
        stops,
    ))
}

fn parsed_context_angles(text: &str) -> Vec<CssAngleOrZero> {
    let mut source = format!(
        "transform: rotate({text}); filter: hue-rotate({text}); backdrop-filter: hue-rotate({text}); background-image: linear-gradient({text}, red, blue)"
    );
    if text != "0" {
        source.push_str(&format!("; image-orientation: {text} flip"));
    }
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), if text == "0" { 4 } else { 5 });
    report
        .syntax()
        .iter()
        .map(
            |declaration| match declaration.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::Transform(wrapper) => {
                    let CssTransform::Functions(functions) = wrapper.value() else {
                        panic!("transform")
                    };
                    let [CssTransformFunction::Rotate(angle)] = functions.functions() else {
                        panic!("one rotation")
                    };
                    angle.clone()
                }
                CssKnownPropertyValueRef::Filter(wrapper) => {
                    let CssFilter::Functions(functions) = wrapper.value() else {
                        panic!("filter")
                    };
                    let [CssFilterFunction::HueRotate(angle)] = functions.functions() else {
                        panic!("one hue rotation")
                    };
                    angle.angle().clone()
                }
                CssKnownPropertyValueRef::BackdropFilter(wrapper) => {
                    let CssFilter::Functions(functions) = wrapper.value() else {
                        panic!("backdrop filter")
                    };
                    let [CssFilterFunction::HueRotate(angle)] = functions.functions() else {
                        panic!("one backdrop hue rotation")
                    };
                    angle.angle().clone()
                }
                CssKnownPropertyValueRef::ImageOrientation(wrapper) => {
                    let CssImageOrientation::Flip(Some(angle)) = wrapper.orientation() else {
                        panic!("flipped orientation")
                    };
                    CssAngleOrZero::Angle(angle.clone())
                }
                CssKnownPropertyValueRef::BackgroundImage(wrapper) => {
                    let [CssImageValue::Gradient(CssGradient::Linear(linear))] =
                        wrapper.images().images()
                    else {
                        panic!("one linear gradient")
                    };
                    let Some(CssLinearGradientDirection::Angle(angle)) = linear.direction() else {
                        panic!("angle direction")
                    };
                    angle.clone()
                }
                _ => panic!("angle context"),
            },
        )
        .collect()
}

#[test]
fn all_existing_angle_parser_contexts_share_literal_units_zero_and_symbolic_root() {
    for (text, unit) in [
        ("-25deg", CssAngleUnit::Degrees),
        ("-25grad", CssAngleUnit::Gradians),
        ("-25rad", CssAngleUnit::Radians),
        ("-25turn", CssAngleUnit::Turns),
    ] {
        for angle in parsed_context_angles(text) {
            let CssAngleOrZero::Angle(angle) = angle else {
                panic!("angle")
            };
            let value = angle.literal().expect("literal");
            assert_eq!(value.numeric().representation(), "-25");
            assert_eq!(value.unit(), unit);
        }
    }
    for angle in parsed_context_angles("0") {
        assert!(matches!(angle, CssAngleOrZero::Zero(_)));
    }
    for angle in parsed_context_angles("calc(25deg + 5deg)") {
        let CssAngleOrZero::Angle(angle) = angle else {
            panic!("angle")
        };
        let value = angle.calculation().expect("symbolic angle");
        assert_eq!(value.result_type(), CssCalculationType::Angle);
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
    }
}

#[test]
fn one_shared_angle_retains_four_literal_units_and_exact_coefficients() {
    for unit in [
        CssAngleUnit::Degrees,
        CssAngleUnit::Gradians,
        CssAngleUnit::Radians,
        CssAngleUnit::Turns,
    ] {
        let CssAngleOrZero::Angle(angle) = angle(-0.25, unit) else {
            panic!("angle")
        };
        let value = angle.literal().expect("literal angle");
        assert_eq!(value.numeric().representation(), "-0.25");
        assert_eq!(value.unit(), unit);
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(CssAngleLiteral::try_new(&invalid.to_string(), unit).is_err());
        }
    }
    assert_ne!(zero_angle(), angle(0.0, CssAngleUnit::Degrees));
    assert_ne!(
        angle(1.0, CssAngleUnit::Degrees),
        angle(1.0, CssAngleUnit::Turns)
    );
}

#[test]
fn shared_angle_constructs_transform_axis_and_skew_roles_without_omission_loss() {
    let rotation = CssTransformRotate3d::new(
        number("1e999"),
        number("0"),
        number("-1e999"),
        angle(45.0, CssAngleUnit::Degrees),
    );
    assert!(
        matches!(rotation.angle(), CssAngleOrZero::Angle(value) if value.literal().is_some_and(|literal| literal.numeric().representation() == "45" && literal.unit() == CssAngleUnit::Degrees))
    );
    let CssComponentValueRef::Token(CssValueTokenRef::Number(axis)) =
        rotation.x().literal_component().unwrap().view()
    else {
        panic!("number axis")
    };
    assert_eq!(axis.representation(), "1e999");
    let zero = CssTransformSkew::new(zero_angle(), None);
    assert!(matches!(zero.x(), CssAngleOrZero::Zero(_)));
    assert!(zero.y().is_none());
    let two = CssTransformSkew::new(
        angle(10.0, CssAngleUnit::Degrees),
        Some(angle(-20.0, CssAngleUnit::Gradians)),
    );
    assert!(
        matches!(two.y(), Some(CssAngleOrZero::Angle(value)) if value.literal().is_some_and(|literal| literal.numeric().representation() == "-20" && literal.unit() == CssAngleUnit::Gradians))
    );
    assert_ne!(
        zero,
        CssTransformSkew::new(zero_angle(), Some(zero_angle()))
    );
    for (function, expected) in [
        (
            CssTransformFunction::Rotate(zero_angle()),
            CssTransformFunctionKind::Rotate,
        ),
        (
            CssTransformFunction::RotateX(zero_angle()),
            CssTransformFunctionKind::RotateX,
        ),
        (
            CssTransformFunction::RotateY(zero_angle()),
            CssTransformFunctionKind::RotateY,
        ),
        (
            CssTransformFunction::RotateZ(zero_angle()),
            CssTransformFunctionKind::RotateZ,
        ),
        (
            CssTransformFunction::SkewX(zero_angle()),
            CssTransformFunctionKind::SkewX,
        ),
        (
            CssTransformFunction::SkewY(zero_angle()),
            CssTransformFunctionKind::SkewY,
        ),
    ] {
        assert_eq!(function.kind(), expected);
    }
}

#[test]
fn shared_angle_constructs_filter_gradient_and_orientation_outer_contexts() {
    let shared = angle(-0.25, CssAngleUnit::Turns);
    assert!(
        matches!(CssFilterFunction::HueRotate(CssFilterHueRotate::new(shared.clone())), CssFilterFunction::HueRotate(value) if matches!(value.angle(), CssAngleOrZero::Angle(angle) if angle.literal().is_some_and(|literal| literal.numeric().representation() == "-0.25" && literal.unit() == CssAngleUnit::Turns)))
    );
    let linear = gradient(shared.clone());
    let CssGradient::Linear(linear) = linear else {
        panic!("linear gradient")
    };
    assert!(
        matches!(linear.direction(), Some(CssLinearGradientDirection::Angle(CssAngleOrZero::Angle(value))) if value.literal().is_some_and(|literal| literal.numeric().representation() == "-0.25" && literal.unit() == CssAngleUnit::Turns))
    );
    let CssAngleOrZero::Angle(shared) = shared else {
        panic!("angle")
    };
    assert!(
        shared
            .literal()
            .is_some_and(|value| value.unit() == CssAngleUnit::Turns)
    );
    assert!(
        matches!(CssImageOrientation::Angle(shared.clone()), CssImageOrientation::Angle(value) if value.literal().is_some_and(|literal| literal.unit() == CssAngleUnit::Turns))
    );
    assert!(
        matches!(CssImageOrientation::Flip(Some(shared)), CssImageOrientation::Flip(Some(value)) if value.literal().is_some_and(|literal| literal.numeric().representation() == "-0.25"))
    );
    let strict_zero =
        CssAngleValue::from_literal(CssAngleLiteral::try_new("0", CssAngleUnit::Degrees).unwrap());
    assert_ne!(
        CssImageOrientation::FromImage,
        CssImageOrientation::Angle(strict_zero.clone())
    );
    assert_ne!(
        CssImageOrientation::Flip(None),
        CssImageOrientation::Flip(Some(strict_zero))
    );
}

#[test]
fn shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality() {
    let first = angle_math("calc(25deg + 5deg)");
    let second = angle_math("  calc(25deg + 5deg)");
    let CssAngleOrZero::Angle(angle) = &first else {
        panic!("angle")
    };
    let value = angle.calculation().expect("calculation");
    assert!(
        (value.result_type()) == (CssCalculationType::Angle),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed calculation origin")
    };
    assert!(
        (origin.source().as_str()) == ("calc(25deg + 5deg)"),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    assert!(
        (first) != (second),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    assert!(
        (CssTransformFunction::Rotate(first.clone()))
            == (CssTransformFunction::Rotate(second.clone())),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    assert!(
        (CssFilterFunction::HueRotate(CssFilterHueRotate::new(first.clone())))
            == (CssFilterFunction::HueRotate(CssFilterHueRotate::new(second.clone()))),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    assert!(
        (CssLinearGradientDirection::Angle(first.clone()))
            == (CssLinearGradientDirection::Angle(second.clone())),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    let CssAngleOrZero::Angle(first) = first else {
        panic!("angle")
    };
    let CssAngleOrZero::Angle(second) = second else {
        panic!("angle")
    };
    assert!(
        (CssImageOrientation::Angle(first)) == (CssImageOrientation::Angle(second)),
        "shared_angle_math_retains_raw_provenance_and_semantic_aggregate_equality: direct equality contract"
    );
    assert!(
        CssAngleCalculation::try_from_components(parse_component_values("calc(1px)").unwrap())
            .is_err()
    );
    assert!(
        CssAngleCalculation::try_from_components(parse_component_values("calc(1%)").unwrap())
            .is_err()
    );
}

#[test]
fn gradient_serializer_omits_only_existing_literal_default_directions() {
    for (value, unit) in [
        (180.0, CssAngleUnit::Degrees),
        (200.0, CssAngleUnit::Gradians),
        (0.5, CssAngleUnit::Turns),
    ] {
        assert_eq!(
            gradient(angle(value, unit)).serialize_specified().unwrap(),
            "linear-gradient(red, blue)"
        );
    }
    assert_eq!(
        gradient(zero_angle()).serialize_specified().unwrap(),
        "linear-gradient(0, red, blue)"
    );
    let radians = angle(std::f32::consts::PI, CssAngleUnit::Radians);
    let CssAngleOrZero::Angle(value) = &radians else {
        panic!("angle direction")
    };
    assert_eq!(
        value.literal().unwrap().numeric().representation(),
        "3.1415927"
    );
    assert_eq!(value.literal().unwrap().unit(), CssAngleUnit::Radians);
    assert_eq!(
        gradient(radians).serialize_specified().unwrap(),
        "linear-gradient(3.141593rad, red, blue)"
    );
    assert_eq!(
        gradient(angle_math("calc(180deg)"))
            .serialize_specified()
            .unwrap(),
        "linear-gradient(calc(180deg), red, blue)"
    );
    assert_eq!(
        gradient(angle_math("calc(90deg + 5deg)"))
            .serialize_specified()
            .unwrap(),
        "linear-gradient(calc(95deg), red, blue)"
    );
}

#[test]
fn omitted_shared_angle_still_counts_with_sibling_images_under_cumulative_limits() {
    let list = CssImageValueList::try_new(vec![
        CssImageValue::Gradient(gradient(angle(180.0, CssAngleUnit::Degrees))),
        CssImageValue::None,
    ])
    .unwrap();
    // list 1 + gradient 1 + direction 1 + angle 1 + stops 1
    // + two (stop 1 + named color 1) + sibling none 1 = 10.
    let expected = "linear-gradient(red, blue), none";
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            10,
            10,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    for (input, projection, bytes, kind) in [
        (
            9,
            10,
            expected.len(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            10,
            9,
            expected.len(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            10,
            10,
            expected.len() - 1,
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                input, projection, bytes
            ))
            .unwrap_err()
            .kind(),
            kind
        );
    }
    assert_eq!(list.serialize_specified().unwrap(), expected);
}
