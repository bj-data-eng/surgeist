#![forbid(unsafe_code)]
//! Checked construction expectations from Color 5 WD20260908 §7 and the
//! existing complete structural ceiling: function nodes add one; none adds zero.
use surgeist_css::*;

fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}

fn stops(color: CssColor, position: Option<CssSpecifiedLengthPercentage>) -> CssColorStopList {
    CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(color, position))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            named("blue"),
            None,
        ))),
    ])
    .unwrap()
}

fn math(depth: usize, leaf: &str) -> CssComponentValues {
    let mut value = leaf.to_owned();
    for _ in 0..depth {
        value = format!("calc({value})");
    }
    let value = parse_component_values(&value).unwrap();
    assert_eq!(value.nesting_depth() as usize, depth);
    value
}

fn length_percentage(depth: usize) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(math(depth, "1px")).unwrap(),
    )
    .unwrap()
}

fn nonnegative_length_percentage(depth: usize) -> CssSpecifiedNonNegativeLengthPercentage {
    CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
        CssLengthPercentageCalculation::try_from_components(math(depth, "1px")).unwrap(),
    )
    .unwrap()
}

fn wrap_image(value: CssImageValue) -> Result<CssLightDarkImage, CssImageConstructionError> {
    CssLightDarkImage::try_new(value, CssImageValue::None)
}

fn assert_image_admission_in_both_branches(value: CssImageValue, admitted: bool) {
    for (light, dark) in [
        (value.clone(), CssImageValue::None),
        (CssImageValue::None, value),
    ] {
        let result = CssLightDarkImage::try_new(light, dark);
        if admitted {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err(), CssImageConstructionError::NestingLimit);
        }
    }
}

#[test]
fn checked_color_pair_accessors_preserve_independently_constructed_children() {
    let light = named("red");
    let dark = CssColor::current_color();
    let pair = CssLightDarkColor::try_new(light.clone(), dark.clone()).unwrap();
    assert_eq!(pair.light(), &light);
    assert_eq!(pair.dark(), &dark);
    let value = CssColor::from_light_dark(pair.clone());
    assert_eq!(value.kind_name(), "light-dark");
    assert_eq!(value.light_dark_value(), Some(&pair));
    assert!(light.light_dark_value().is_none());
    assert_eq!(
        value.to_specified_css().unwrap(),
        "light-dark(red, currentcolor)"
    );
    assert_eq!(
        value.absolute_eligibility(),
        CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::LightDark)
    );
    assert_ne!(pair, CssLightDarkColor::try_new(dark, light).unwrap());
}

#[test]
fn checked_image_pairs_preserve_url_none_identity_and_are_real_images() {
    let url = CssImageValue::Url(CssUrl::new("a.svg"));
    let pair = CssLightDarkImage::try_new(url.clone(), CssImageValue::None).unwrap();
    assert_eq!(pair.light(), &url);
    assert_eq!(pair.dark(), &CssImageValue::None);
    assert_eq!(
        CssImageValue::LightDark(Box::new(pair))
            .serialize_specified()
            .unwrap(),
        "light-dark(url(\"a.svg\"), none)"
    );
    let pair = CssLightDarkImage::try_new(CssImageValue::None, CssImageValue::None).unwrap();
    let image = CssImage::try_new(CssImageValue::LightDark(Box::new(pair))).unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        "light-dark(none, none)"
    );
    assert!(CssImage::try_new(CssImageValue::None).is_none());
}

#[test]
fn checked_color_and_image_pairs_admit_256_functions_and_reject_the_next() {
    let mut color = named("red");
    let mut image = CssImageValue::None;
    for _ in 0..256 {
        color =
            CssColor::from_light_dark(CssLightDarkColor::try_new(color, named("blue")).unwrap());
        image = CssImageValue::LightDark(Box::new(
            CssLightDarkImage::try_new(image, CssImageValue::None).unwrap(),
        ));
    }
    assert_eq!(
        CssLightDarkColor::try_new(named("blue"), color.clone()).unwrap_err(),
        CssColorConstructionError::NestingLimit
    );
    assert_eq!(
        CssLightDarkColor::try_new(color, named("blue")).unwrap_err(),
        CssColorConstructionError::NestingLimit
    );
    assert_image_admission_in_both_branches(image, false);
    let mut image = CssImageValue::Url(CssUrl::new("a.svg"));
    for _ in 0..255 {
        image = CssImageValue::LightDark(Box::new(wrap_image(image).unwrap()));
    }
    assert_eq!(
        wrap_image(image).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
}

#[test]
fn checked_image_depth_includes_deep_gradient_color_subtrees_and_oversized_children() {
    for depth in [254, 255, 256] {
        let mut color = named("red");
        for _ in 0..depth {
            color = CssColor::from_light_dark(
                CssLightDarkColor::try_new(color, named("blue")).unwrap(),
            );
        }
        // Gradient construction is deliberately infallible and may produce a
        // subtree already beyond the envelope required by the new pair boundary.
        let image = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
            None,
            stops(color, None),
        )));
        assert_image_admission_in_both_branches(image, depth == 254);
    }
}

#[test]
fn checked_image_depth_includes_all_gradient_numeric_component_roles() {
    for depth in [254, 255] {
        let direction = CssLinearGradientDirection::Angle(CssAngleOrZero::Angle(
            CssAngleValue::try_from_calculation(
                CssAngleCalculation::try_from_components(math(depth, "1deg")).unwrap(),
            )
            .unwrap(),
        ));
        let direction_image = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
            Some(direction),
            stops(named("red"), None),
        )));
        let stop_image = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
            None,
            stops(named("red"), Some(length_percentage(depth))),
        )));
        let hint_stops = CssColorStopList::try_new(vec![
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                named("red"),
                None,
            ))),
            CssColorStopListItem::Hint(length_percentage(depth)),
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                named("blue"),
                None,
            ))),
        ])
        .unwrap();
        let hint_image = CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(
            None, hint_stops,
        )));
        let circle = CssRadialSize::Circle(
            CssSpecifiedNonNegativeLength::try_from_calculation(
                CssLengthCalculation::try_from_components(math(depth, "1px")).unwrap(),
            )
            .unwrap(),
        );
        let ellipse = CssRadialSize::Ellipse(CssRadialEllipseSize::new(
            nonnegative_length_percentage(depth),
            CssSpecifiedNonNegativeLengthPercentage::zero(),
        ));
        let reverse_ellipse = CssRadialSize::Ellipse(CssRadialEllipseSize::new(
            CssSpecifiedNonNegativeLengthPercentage::zero(),
            nonnegative_length_percentage(depth),
        ));
        let mut values = vec![direction_image, stop_image, hint_image];
        for size in [circle, ellipse, reverse_ellipse] {
            values.push(CssImageValue::Gradient(CssGradient::Radial(
                CssRadialGradient::try_new(None, Some(size), None, stops(named("red"), None))
                    .unwrap(),
            )));
        }
        for (horizontal, vertical) in [
            (
                CssHorizontalPosition::Offset(length_percentage(depth)),
                CssVerticalPosition::Center,
            ),
            (
                CssHorizontalPosition::Center,
                CssVerticalPosition::Offset(length_percentage(depth)),
            ),
        ] {
            let position = CssPhysicalPosition::try_new(horizontal, vertical).unwrap();
            values.push(CssImageValue::Gradient(CssGradient::Radial(
                CssRadialGradient::try_new(None, None, Some(position), stops(named("red"), None))
                    .unwrap(),
            )));
        }
        for value in values {
            let repeating = match &value {
                CssImageValue::Gradient(CssGradient::Linear(value)) => {
                    CssImageValue::Gradient(CssGradient::RepeatingLinear(value.clone()))
                }
                CssImageValue::Gradient(CssGradient::Radial(value)) => {
                    CssImageValue::Gradient(CssGradient::RepeatingRadial(value.clone()))
                }
                _ => panic!("independently constructed gradient"),
            };
            assert_image_admission_in_both_branches(value, depth == 254);
            assert_image_admission_in_both_branches(repeating, depth == 254);
        }
    }
}

#[test]
fn checked_image_depth_includes_url_modifier_function_and_argument_trees() {
    for depth in [253, 254, 255] {
        let modifier = CssUrlModifier::Function(
            CssUrlModifierFunction::try_new(CssIdent::try_new("hint").unwrap(), math(depth, "1"))
                .unwrap(),
        );
        let url = CssImageValue::Url(CssUrl::from_parts(
            CssUrlFunction::Src,
            "a.svg",
            vec![modifier],
        ));
        // Pair + URL + modifier function + argument functions.
        assert_image_admission_in_both_branches(url, depth == 253);
    }
}

#[test]
fn new_image_construction_errors_implement_the_semantic_error_contract() {
    for (value, text) in [
        (
            CssImageConstructionError::NestingLimit,
            "image nesting limit exceeded",
        ),
        (
            CssImageConstructionError::CapacityOverflow,
            "image capacity overflow",
        ),
    ] {
        assert_eq!(value.to_string(), text);
        let error: &dyn std::error::Error = &value;
        assert!(error.source().is_none());
    }
}

#[test]
fn light_dark_metadata_identifies_complete_authored_productions_and_the_dated_source() {
    for (id, production, source) in [
        (
            "interop.value.light-dark-color",
            "#typedef-light-dark-color",
            "color:light-dark(red, blue)",
        ),
        (
            "interop.value.light-dark-image",
            "#typedef-light-dark-image",
            "background-image:light-dark(none, none)",
        ),
    ] {
        let metadata = feature_metadata(id).unwrap();
        assert_eq!(metadata.kind(), CssFeatureKind::Value);
        assert_eq!(metadata.status(), CssSupportStatus::Complete);
        assert_eq!(metadata.production(), production);
        assert_eq!(metadata.source().id().as_str(), "I-COLOR5-20260908");
        assert_eq!(
            metadata.source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-color-5-20260908/")
        );
        assert!(metadata.supported_subset().is_none());
        assert!(metadata.unsupported_remainder().is_none());
        assert!(metadata.baseline_alias_targets().is_empty());
        assert!(validate_style_attribute(source).is_ok());
    }
    assert!(specification_source("I-COLOR5-20260908").is_some());
    assert_eq!(
        feature_metadata("ext.value.color-mix")
            .unwrap()
            .source()
            .id()
            .as_str(),
        "I-COLOR5"
    );
}
