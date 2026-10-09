#![forbid(unsafe_code)]
//! Functional new-API contracts from Images 4 WD 2025-09-30 §§3.1–3.5,
//! the specific Values 5 WD 2024-11-11 position import, and shared Color 4/5.
//! New construction APIs have no executable preimplementation boundary.
use surgeist_css::*;

fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}
fn component(css: &str) -> CssComponentValue {
    let values = parse_component_values(css).unwrap();
    assert_eq!(values.items().len(), 1);
    values.items()[0].clone()
}
fn angular(css: &str) -> CssAngularColorStopPosition {
    CssAngularColorStopPosition::try_from_component(component(css)).unwrap()
}
fn lp(css: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(component(css)).unwrap()
}
fn angular_stop(positions: Vec<CssAngularColorStopPosition>) -> CssAngularColorStopListItem {
    CssAngularColorStopListItem::Stop(Box::new(
        CssAngularColorStop::try_new(named("red"), positions).unwrap(),
    ))
}
fn angular_stops() -> CssAngularColorStopList {
    CssAngularColorStopList::try_new(vec![angular_stop(vec![])]).unwrap()
}
fn line_stops() -> CssColorStopList {
    CssColorStopList::try_new(vec![CssColorStopListItem::Stop(Box::new(
        CssGradientColorStop::from_color(named("red"), None),
    ))])
    .unwrap()
}
fn interpolation() -> CssColorInterpolation {
    CssColorInterpolation::from_predefined(
        CssColorInterpolationMethod::try_new(
            CssColorInterpolationSpace::Oklch,
            Some(CssHueInterpolationMethod::Longer),
        )
        .unwrap(),
    )
}
fn conic(
    from: Option<CssAngleOrZero>,
    position: Option<CssPosition>,
    stops: CssAngularColorStopList,
) -> CssImageValue {
    CssImageValue::Gradient(CssGradient::Conic(CssConicGradient::new(
        from, position, None, stops,
    )))
}

#[test]
fn checked_stops_retain_zero_one_or_two_positions_and_reject_a_third() {
    for count in 0..=2 {
        let linear =
            CssGradientColorStop::try_from_positions(named("red"), vec![lp("25%"); count]).unwrap();
        assert_eq!(linear.positions().len(), count);
        assert_eq!(linear.position(), linear.positions().first());
        let conic =
            CssAngularColorStop::try_new(named("red"), vec![angular("-90deg"); count]).unwrap();
        assert_eq!(conic.positions().len(), count);
        assert_eq!(conic.color(), &named("red"));
    }
    assert_eq!(
        CssGradientColorStop::try_from_positions(named("red"), vec![lp("25%"); 3]).unwrap_err(),
        CssGradientStopConstructionError::TooManyPositions
    );
    assert_eq!(
        CssAngularColorStop::try_new(named("red"), vec![angular("25%"); 3]).unwrap_err(),
        CssGradientStopConstructionError::TooManyPositions
    );
}

#[test]
fn angular_lists_admit_a_single_stop_and_only_internal_separated_hints() {
    let stop = || angular_stop(vec![]);
    let hint = || CssAngularColorStopListItem::Hint(angular("40%"));
    assert!(CssAngularColorStopList::try_new(vec![]).is_none());
    assert!(CssAngularColorStopList::try_new(vec![stop()]).is_some());
    assert!(CssAngularColorStopList::try_new(vec![hint(), stop()]).is_none());
    assert!(CssAngularColorStopList::try_new(vec![stop(), hint()]).is_none());
    assert!(CssAngularColorStopList::try_new(vec![stop(), hint(), hint(), stop()]).is_none());
    assert!(CssAngularColorStopList::try_new(vec![stop(), hint(), stop()]).is_some());
}

#[test]
fn angular_literals_admit_angles_percentages_and_exact_zero_without_number_coercion() {
    for css in [
        "-90deg", "100grad", "2rad", ".25turn", "125%", "-20%", "0", "-0e99",
    ] {
        let value = angular(css);
        assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        assert!(
            parse_style_attribute(&format!(
                "background-image:conic-gradient(red {})",
                value.serialize_specified().unwrap()
            ))
            .is_clean()
        );
    }
    for css in ["1", "1e-99", "1px", "1s", "none", "calc(1deg)"] {
        assert!(
            CssAngularColorStopPosition::try_from_component(component(css)).is_err(),
            "{css}"
        );
    }
}

#[test]
fn angular_percentage_math_retains_the_contextual_angle_type_and_authored_origin() {
    let expression = CssAnglePercentageCalculation::try_from_components(
        parse_component_values("calc(1turn + 25%)").unwrap(),
    )
    .unwrap();
    assert_eq!(
        expression.result_type(),
        CssCalculationType::AnglePercentage
    );
    assert_eq!(
        expression.numeric_type().percent_hint(),
        Some(CssNumericDimension::Angle)
    );
    let origin = expression.origin().clone();
    let position = CssAngularColorStopPosition::from_calculation(expression);
    assert_eq!(position.origin(), &origin);
    assert_eq!(
        position.serialize_specified().unwrap(),
        "calc(25% + 360deg)"
    );
    for css in ["calc(1px + 25%)", "calc(1deg + 1s)", "calc(0)"] {
        assert!(
            CssAnglePercentageCalculation::try_from_components(
                parse_component_values(css).unwrap()
            )
            .is_err(),
            "{css}"
        );
    }
}

#[test]
fn conic_constructor_preserves_omission_and_explicit_full_flow_prelude() {
    let omitted = CssConicGradient::new(None, None, None, angular_stops());
    assert!(omitted.from().is_none());
    assert!(omitted.position().is_none());
    assert!(omitted.interpolation().is_none());
    let position =
        CssPosition::try_from_named_axes(CssBlockPosition::Start, CssInlinePosition::End).unwrap();
    let zero = CssAngleOrZero::Zero(CssZeroLiteral::try_from_component(component("0")).unwrap());
    let explicit = CssConicGradient::new(
        Some(zero.clone()),
        Some(position.clone()),
        Some(interpolation()),
        angular_stops(),
    );
    assert_eq!(explicit.from(), Some(&zero));
    assert_eq!(explicit.position(), Some(&position));
    assert_eq!(explicit.interpolation(), Some(&interpolation()));
    let image = CssImage::try_new(CssImageValue::Gradient(CssGradient::Conic(explicit))).unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        "conic-gradient(at block-start inline-end in oklch longer hue, red)"
    );
}

#[test]
fn linear_and_radial_interpolation_builders_preserve_explicit_method_and_full_position() {
    let linear =
        CssLinearGradient::new(None, line_stops()).with_interpolation(Some(interpolation()));
    assert_eq!(linear.interpolation(), Some(&interpolation()));
    assert!(
        linear
            .clone()
            .with_interpolation(None)
            .interpolation()
            .is_none()
    );
    let position = CssPosition::try_from_relative_axes(
        CssRelativeAxisPosition::Start,
        CssRelativeAxisPosition::End,
    )
    .unwrap();
    let radial = CssRadialGradient::try_new(None, None, Some(position.clone()), line_stops())
        .unwrap()
        .with_interpolation(Some(interpolation()));
    assert_eq!(radial.position(), Some(&position));
    assert_eq!(radial.interpolation(), Some(&interpolation()));
    assert_eq!(
        CssGradient::Linear(linear).serialize_specified().unwrap(),
        "linear-gradient(in oklch longer hue, red)"
    );
    assert_eq!(
        CssGradient::Radial(radial).serialize_specified().unwrap(),
        "radial-gradient(at start end in oklch longer hue, red)"
    );
}

#[test]
fn gradient_equality_observes_second_positions_and_explicit_interpolation() {
    let first =
        CssGradientColorStop::try_from_positions(named("red"), vec![lp("10%"), lp("20%")]).unwrap();
    let second =
        CssGradientColorStop::try_from_positions(named("red"), vec![lp("10%"), lp("30%")]).unwrap();
    assert_ne!(first, second);
    assert_ne!(
        CssAngularColorStop::try_new(named("red"), vec![angular("0"), angular("20%")]).unwrap(),
        CssAngularColorStop::try_new(named("red"), vec![angular("0"), angular("30%")]).unwrap()
    );
    let plain = CssLinearGradient::new(None, line_stops());
    assert_ne!(
        plain,
        plain.clone().with_interpolation(Some(interpolation()))
    );
    let conic_plain = CssConicGradient::new(None, None, None, angular_stops());
    assert_ne!(
        conic_plain,
        CssConicGradient::new(None, None, Some(interpolation()), angular_stops())
    );
}

#[test]
fn constructed_conic_budget_counts_both_positions_and_remains_atomic_on_failure() {
    let value = conic(
        None,
        None,
        CssAngularColorStopList::try_new(vec![angular_stop(vec![angular("0"), angular("50%")])])
            .unwrap(),
    );
    let before = value.clone();
    let expected = "conic-gradient(red 0 50%)";
    // Gradient + list + stop + named color + two numeric leaves = six nodes.
    let limits = CssSpecifiedValueSerializationLimits::new(6, 6, expected.len());
    assert_eq!(
        value.serialize_specified_with_limits(limits).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 6, expected.len() - 1),
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
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    6,
                    6,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
    }
}

fn math(functions: usize, leaf: &str) -> CssComponentValues {
    let mut css = leaf.to_owned();
    for _ in 0..functions {
        css = format!("calc({css})");
    }
    parse_component_values(&css).unwrap()
}
fn angular_math(functions: usize) -> CssAngularColorStopPosition {
    CssAngularColorStopPosition::from_calculation(
        CssAnglePercentageCalculation::try_from_components(math(functions, "1deg")).unwrap(),
    )
}
fn line_math(functions: usize) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(math(functions, "1px")).unwrap(),
    )
    .unwrap()
}
fn check_depth(build: impl Fn(usize) -> CssImageValue) {
    let valid = build(255);
    let invalid = build(256);
    assert_eq!(CssImage::try_new(valid.clone()).unwrap().value(), &valid);
    for _ in 0..2 {
        assert_eq!(
            CssImage::try_new(invalid.clone()).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        assert_eq!(CssImage::try_new(valid.clone()).unwrap().value(), &valid);
    }
}

#[test]
fn checked_image_depth_visits_conic_second_stop_position_and_hint_math() {
    check_depth(|depth| {
        conic(
            None,
            None,
            CssAngularColorStopList::try_new(vec![angular_stop(vec![
                angular("0"),
                angular_math(depth),
            ])])
            .unwrap(),
        )
    });
    check_depth(|depth| {
        conic(
            None,
            None,
            CssAngularColorStopList::try_new(vec![
                angular_stop(vec![]),
                CssAngularColorStopListItem::Hint(angular_math(depth)),
                angular_stop(vec![]),
            ])
            .unwrap(),
        )
    });
}

#[test]
fn checked_image_depth_visits_conic_from_and_full_flow_position() {
    check_depth(|depth| {
        conic(
            Some(CssAngleOrZero::Angle(
                CssAngleValue::try_from_calculation(
                    CssAngleCalculation::try_from_components(math(depth, "1deg")).unwrap(),
                )
                .unwrap(),
            )),
            None,
            angular_stops(),
        )
    });
    check_depth(|depth| {
        conic(
            None,
            Some(
                CssPosition::try_from_named_axes(
                    CssBlockPosition::StartOffset(line_math(depth)),
                    CssInlinePosition::EndOffset(lp("2px")),
                )
                .unwrap(),
            ),
            angular_stops(),
        )
    });
    check_depth(|depth| {
        CssImageValue::Gradient(CssGradient::Radial(
            CssRadialGradient::try_new(
                None,
                None,
                Some(
                    CssPosition::try_from_relative_axes(
                        CssRelativeAxisPosition::StartOffset(line_math(depth)),
                        CssRelativeAxisPosition::EndOffset(lp("2px")),
                    )
                    .unwrap(),
                ),
                line_stops(),
            )
            .unwrap(),
        ))
    });
}

#[test]
fn checked_image_depth_visits_second_linear_position_and_conic_color() {
    check_depth(|depth| {
        CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
            None,
            CssColorStopList::try_new(vec![CssColorStopListItem::Stop(Box::new(
                CssGradientColorStop::try_from_positions(
                    named("red"),
                    vec![lp("0"), line_math(depth)],
                )
                .unwrap(),
            ))])
            .unwrap(),
        )))
    });
    check_depth(|depth| {
        let mut color = named("red");
        for _ in 0..depth {
            color = CssColor::from_light_dark(
                CssLightDarkColor::try_new(color, named("blue")).unwrap(),
            );
        }
        conic(
            None,
            None,
            CssAngularColorStopList::try_new(vec![CssAngularColorStopListItem::Stop(Box::new(
                CssAngularColorStop::try_new(color, vec![]).unwrap(),
            ))])
            .unwrap(),
        )
    });
}

#[test]
fn selected_metadata_traces_the_exact_gradient_import_without_claiming_siblings() {
    for feature in [
        "ext.value.conic-gradient",
        "ext.value.gradient-interpolation",
        "ext.value.gradient-stop-list",
    ] {
        let metadata = feature_metadata(feature).unwrap();
        let source = metadata.source();
        assert_eq!(source.id().as_str(), "F-IMAGES4-GRADIENTS-20250930");
        assert_eq!(
            source.tier(),
            CssSpecificationTier::Snapshot2026PreCrException
        );
        assert_eq!(
            source.url(),
            Some("https://www.w3.org/TR/2025/WD-css-images-4-20250930/")
        );
        assert_eq!(source.module(), "CSS Images");
        assert_eq!(source.level(), "4");
        assert_eq!(specification_source(source.id().as_str()), Some(&source));
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
    }
    let position = feature_metadata("required.value.gradient-position").unwrap();
    assert_eq!(
        position.source().id().as_str(),
        "D-VALUES5-GRADIENT-POSITION"
    );
    assert_eq!(
        position.source().tier(),
        CssSpecificationTier::LaterStandard
    );
    assert_eq!(
        position.source().url(),
        Some("https://www.w3.org/TR/2024/WD-css-values-5-20241111/")
    );
    assert_eq!(position.status(), CssSupportStatus::Complete);
    for feature in ["ext.value.image-1d", "ext.value.stripes"] {
        let source = feature_metadata(feature).unwrap().source();
        assert_eq!(source.id().as_str(), "X-IMAGES4-20250930");
        assert_eq!(source.tier(), CssSpecificationTier::SurgeistExtension);
        assert_eq!(
            source.url(),
            Some("https://www.w3.org/TR/2025/WD-css-images-4-20250930/")
        );
    }
    assert!(
        !parse_style_attribute("background-image:repeating-conic-gradient(red,blue)").is_clean()
    );
}

#[test]
fn parsed_conic_second_position_and_from_keep_the_original_source_spans() {
    let source = "/*😀*/background-image:conic-gradient(from calc(15deg + 5deg) at block-start 3px inline-end 4% in lab, red 10deg calc(20deg + 25%))!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("background image")
    };
    let [CssImageValue::Gradient(CssGradient::Conic(gradient))] = images.images().images() else {
        panic!("conic")
    };
    let CssAngularColorStopListItem::Stop(stop) = &gradient.stops().items()[0] else {
        panic!("stop")
    };
    assert_eq!(stop.positions().len(), 2);
    for (origin, token) in [
        (gradient.from().unwrap().origin(), "calc(15deg + 5deg)"),
        (stop.positions()[0].origin(), "10deg"),
        (stop.positions()[1].origin(), "calc(20deg + 25%)"),
    ] {
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("parsed occurrence")
        };
        let start = source.find(token).unwrap();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start
                + if token.starts_with("calc(") {
                    5
                } else {
                    token.len()
                }
        );
    }
    let CssPositionRef::NamedFlow(axes) = gradient.position().unwrap().view() else {
        panic!("symbolic flow axes")
    };
    assert!(matches!(axes.block(), CssBlockPosition::StartOffset(_)));
    assert!(matches!(axes.inline(), CssInlinePosition::EndOffset(_)));
    assert_eq!(
        gradient
            .interpolation()
            .unwrap()
            .predefined()
            .unwrap()
            .space(),
        CssColorInterpolationSpace::Lab
    );
}

#[test]
fn intrinsic_conic_geometry_defaults_are_omitted_with_all_authored_nodes_still_charged() {
    // Images4 §§3.3.1, 3.3.3, 3.5.2 and §8: these spellings have the same
    // intrinsic meaning. The checked model still retains every authored field.
    for (start, end) in [
        ("0", "100%"),
        ("0deg", "360deg"),
        ("0grad", "400grad"),
        ("0turn", "1turn"),
    ] {
        let stops = CssAngularColorStopList::try_new(vec![
            angular_stop(vec![angular(start)]),
            CssAngularColorStopListItem::Stop(Box::new(
                CssAngularColorStop::try_new(named("blue"), vec![angular(end)]).unwrap(),
            )),
        ])
        .unwrap();
        let gradient = CssConicGradient::new(
            Some(CssAngleOrZero::Zero(
                CssZeroLiteral::try_from_component(component("0")).unwrap(),
            )),
            Some(
                CssPosition::try_from_named_axes(
                    CssBlockPosition::Center,
                    CssInlinePosition::Center,
                )
                .unwrap(),
            ),
            Some(CssColorInterpolation::from_predefined(
                CssColorInterpolationMethod::try_new(CssColorInterpolationSpace::Oklab, None)
                    .unwrap(),
            )),
            stops,
        );
        let image = CssImageValue::Gradient(CssGradient::Conic(gradient.clone()));
        let expected = "conic-gradient(in oklab, red, blue)";
        // Gradient/list + two stop/color/position triples + from + position
        // aggregate/two axes + interpolation = thirteen input/projection nodes.
        assert_eq!(
            image
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    13,
                    13,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            image
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    12,
                    13,
                    expected.len()
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        assert_eq!(
            image
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    13,
                    12,
                    expected.len()
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
        );
        assert!(gradient.from().is_some());
        assert!(gradient.position().is_some());
        assert!(gradient.interpolation().is_some());
        assert_eq!(image, CssImageValue::Gradient(CssGradient::Conic(gradient)));
    }
}

#[test]
fn singleton_first_zero_is_omitted_but_its_last_hundred_position_remains_authored() {
    for (position, expected) in [
        ("0", "conic-gradient(red)"),
        ("100%", "conic-gradient(red 100%)"),
    ] {
        let value = conic(
            None,
            None,
            CssAngularColorStopList::try_new(vec![angular_stop(vec![angular(position)])]).unwrap(),
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    5,
                    5,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    4,
                    5,
                    expected.len()
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
    }
    for (position, expected) in [
        ("0%", "linear-gradient(red)"),
        ("100%", "linear-gradient(red 100%)"),
    ] {
        let stops = CssColorStopList::try_new(vec![CssColorStopListItem::Stop(Box::new(
            CssGradientColorStop::from_color(named("red"), Some(lp(position))),
        ))])
        .unwrap();
        let value = CssGradient::Linear(CssLinearGradient::new(None, stops));
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    5,
                    5,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn matching_interpolation_omission_keeps_the_authored_node_budget() {
    // Color4 §13.2 makes ordinary legacy stops sRGB by default; Images4 §8
    // removes a matching method from output without removing its authored node.
    let gradient = CssConicGradient::new(
        None,
        None,
        Some(CssColorInterpolation::from_predefined(
            CssColorInterpolationMethod::try_new(
                CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
                None,
            )
            .unwrap(),
        )),
        CssAngularColorStopList::try_new(vec![
            angular_stop(vec![]),
            CssAngularColorStopListItem::Stop(Box::new(
                CssAngularColorStop::try_new(named("blue"), vec![]).unwrap(),
            )),
        ])
        .unwrap(),
    );
    let before = gradient.clone();
    let image = CssImageValue::Gradient(CssGradient::Conic(gradient.clone()));
    let expected = "conic-gradient(red, blue)";
    assert_eq!(
        image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                7,
                7,
                expected.len()
            ),)
            .unwrap(),
        expected
    );
    for (input, projection, cause) in [
        (
            6,
            7,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            7,
            6,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            image
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    input,
                    projection,
                    expected.len()
                ),)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(image.serialize_specified().unwrap(), expected);
    }
    assert_eq!(gradient, before);
    assert!(gradient.interpolation().is_some());
}

#[test]
fn generated_default_preservation_does_not_invent_an_authored_input_node() {
    // Color4 §13.2 and §16.2.2 require this generated method when ordinary
    // missing RGB projects into color(srgb). The new conic constructor retains
    // omission, even while specified output protects the original default.
    let color = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::None,
                CssColorComponent::Number(
                    CssColorNumberLiteral::try_from_component(component("0")).unwrap(),
                ),
                CssColorComponent::Number(
                    CssColorNumberLiteral::try_from_component(component("255")).unwrap(),
                ),
            ],
            None,
        )
        .unwrap(),
    );
    let stops = CssAngularColorStopList::try_new(vec![
        CssAngularColorStopListItem::Stop(Box::new(
            CssAngularColorStop::try_new(color, vec![]).unwrap(),
        )),
        CssAngularColorStopListItem::Stop(Box::new(
            CssAngularColorStop::try_new(named("blue"), vec![]).unwrap(),
        )),
    ])
    .unwrap();
    let gradient = CssConicGradient::new(None, None, None, stops.clone());
    let image = CssImageValue::Gradient(CssGradient::Conic(gradient.clone()));
    let expected = "conic-gradient(in srgb, color(srgb none 0 1), blue)";
    // Gradient/list + two stop nodes + RGB/three channels + named blue = 9.
    assert_eq!(
        image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                9,
                1000,
                expected.len()
            ),)
            .unwrap(),
        expected
    );
    assert_eq!(
        image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                8,
                1000,
                expected.len()
            ),)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert!(gradient.interpolation().is_none());
    assert_eq!(image.serialize_specified().unwrap(), expected);
    let authored = CssImageValue::Gradient(CssGradient::Conic(CssConicGradient::new(
        None,
        None,
        Some(CssColorInterpolation::from_predefined(
            CssColorInterpolationMethod::try_new(
                CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
                None,
            )
            .unwrap(),
        )),
        stops,
    )));
    assert_eq!(
        authored
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                9,
                1000,
                expected.len()
            ),)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        authored
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                10,
                1000,
                expected.len()
            ),)
            .unwrap(),
        expected
    );
}

#[test]
fn missing_rgb_sibling_defaults_follow_the_retained_color_carrier() {
    // Ordinary missing RGB remains legacy in the source. Keywords preserve
    // that source default; separately nonlegacy/mix and qualified relative
    // carriers establish Oklab without executing origins or mixing colors.
    for (sibling, emitted_sibling, method) in [
        ("currentcolor", "currentcolor", "in srgb, "),
        ("CanvasText", "canvastext", "in srgb, "),
        (
            "rgb(from currentcolor r g b)",
            "rgb(from currentcolor r g b)",
            "",
        ),
        (
            "color-mix(in srgb, currentcolor, blue)",
            "color-mix(in srgb, currentcolor, blue)",
            "",
        ),
    ] {
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
            parse_component_values(&format!("conic-gradient(rgb(none 0 255), {sibling})")).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::BackgroundImage(images) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("image");
        };
        let [CssImageValue::Gradient(CssGradient::Conic(gradient))] = images.images().images()
        else {
            panic!("conic");
        };
        assert!(gradient.interpolation().is_none());
        let expected = format!("conic-gradient({method}color(srgb none 0 1), {emitted_sibling})");
        assert_eq!(
            CssGradient::Conic(gradient.clone())
                .serialize_specified()
                .unwrap(),
            expected
        );
        let report = parse_style_attribute(&format!("background-image: {expected}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(
            report.syntax()[0].to_specified_css().unwrap(),
            declaration.to_specified_css().unwrap()
        );
    }
}

#[test]
fn matching_legacy_keyword_default_is_omitted_without_resolving_current_color() {
    // The selected keyword/SRGB default is structural. No used current color
    // is required to prove a matching method redundant in this new constructor.
    let gradient = CssConicGradient::new(
        None,
        None,
        Some(CssColorInterpolation::from_predefined(
            CssColorInterpolationMethod::try_new(
                CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
                None,
            )
            .unwrap(),
        )),
        CssAngularColorStopList::try_new(vec![
            CssAngularColorStopListItem::Stop(Box::new(
                CssAngularColorStop::try_new(CssColor::current_color(), vec![]).unwrap(),
            )),
            CssAngularColorStopListItem::Stop(Box::new(
                CssAngularColorStop::try_new(named("blue"), vec![]).unwrap(),
            )),
        ])
        .unwrap(),
    );
    let image = CssImageValue::Gradient(CssGradient::Conic(gradient.clone()));
    let expected = "conic-gradient(currentcolor, blue)";
    assert_eq!(
        image
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                7,
                7,
                expected.len()
            ),)
            .unwrap(),
        expected
    );
    assert!(gradient.interpolation().is_some());
    let report = parse_style_attribute(&format!("background-image: {expected}"));
    assert!(report.is_clean());
}

#[test]
fn matching_relative_legacy_function_default_preserves_the_declared_carrier() {
    // Qualified frozen nonkeyword default applies to these relative carriers;
    // their Color5 comma-free grammar alone does not settle the gradient space.
    for color in [
        "rgb(from currentcolor r g b)",
        "hsl(from currentcolor h s l)",
        "hwb(from currentcolor h w b)",
    ] {
        let declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::BackgroundImage),
            parse_component_values(&format!("conic-gradient(in oklab, {color}, blue)")).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::BackgroundImage(images) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("image");
        };
        let [CssImageValue::Gradient(CssGradient::Conic(gradient))] = images.images().images()
        else {
            panic!("conic");
        };
        assert!(gradient.interpolation().is_some());
        let expected = format!("conic-gradient({color}, blue)");
        assert_eq!(
            CssGradient::Conic(gradient.clone())
                .serialize_specified()
                .unwrap(),
            expected
        );
        let report = parse_style_attribute(&format!("background-image: {expected}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        assert_eq!(
            report.syntax()[0].to_specified_css().unwrap(),
            declaration.to_specified_css().unwrap()
        );
    }
}
