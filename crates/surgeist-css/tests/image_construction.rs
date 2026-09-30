#![forbid(unsafe_code)]
//! Checked construction for Images 3 §3 and Values 4 generic `<position>`.
//! https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#gradients
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#position

use surgeist_css::*;

fn color(css: &str) -> CssColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(css).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected checked color");
    };
    value.value().clone()
}

fn stop(css: &str, position: Option<CssSpecifiedLengthPercentage>) -> CssGradientColorStop {
    CssGradientColorStop::from_color(color(css), position)
}

fn stops() -> CssColorStopList {
    CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(stop("red", None))),
        CssColorStopListItem::Stop(Box::new(stop("blue", None))),
    ])
    .unwrap()
}

#[test]
fn image_only_boundary_rejects_property_none_and_retains_all_published_families() {
    assert!(CssImage::try_new(CssImageValue::None).is_none());

    let url = CssUrl::new("figure.svg");
    let image = CssImage::try_new(CssImageValue::Url(url)).unwrap();
    assert!(matches!(image.value(), CssImageValue::Url(value) if value.as_str() == "figure.svg"));

    let linear = CssLinearGradient::new(None, stops());
    let radial = CssRadialGradient::try_new(None, None, None, stops()).unwrap();
    let linear_image =
        CssImage::try_new(CssImageValue::Gradient(CssGradient::Linear(linear.clone()))).unwrap();
    assert!(
        matches!(linear_image.value(), CssImageValue::Gradient(CssGradient::Linear(payload)) if payload == &linear)
    );
    let repeating_linear = CssImage::try_new(CssImageValue::Gradient(
        CssGradient::RepeatingLinear(linear.clone()),
    ))
    .unwrap();
    assert!(
        matches!(repeating_linear.value(), CssImageValue::Gradient(CssGradient::RepeatingLinear(payload)) if payload == &linear)
    );
    let radial_image =
        CssImage::try_new(CssImageValue::Gradient(CssGradient::Radial(radial.clone()))).unwrap();
    assert!(
        matches!(radial_image.value(), CssImageValue::Gradient(CssGradient::Radial(payload)) if payload == &radial)
    );
    let repeating_radial = CssImage::try_new(CssImageValue::Gradient(
        CssGradient::RepeatingRadial(radial.clone()),
    ))
    .unwrap();
    assert!(
        matches!(repeating_radial.value(), CssImageValue::Gradient(CssGradient::RepeatingRadial(payload)) if payload == &radial)
    );
}

#[test]
fn constructed_color_stops_keep_authored_color_and_signed_line_positions() {
    let negative = signed_length_percentage("-1px");
    let authored = color("currentcolor");
    let built = CssGradientColorStop::from_color(authored.clone(), Some(negative));
    assert_eq!(built.color(), &authored);
    assert!(exact_dimension(
        built.position().unwrap().literal_component(),
        "-1",
        "px"
    ));
    assert_eq!(built.color().to_specified_css().unwrap(), "currentcolor");
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_ident("auto").unwrap()
        )
        .is_err()
    );

    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(1px + 5%)").unwrap(),
    )
    .unwrap();
    let symbolic = CssSpecifiedLengthPercentage::try_from_calculation(calculation.clone()).unwrap();
    let stop = CssGradientColorStop::from_color(color("red"), Some(symbolic));
    assert!(
        stop.position()
            .unwrap()
            .calculation()
            .is_some_and(|value| value.components() == calculation.components())
    );
}

#[test]
fn parsed_gradient_stop_equals_checked_reconstruction() {
    let report = parse_style_attribute("background-image: linear-gradient(red, blue)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundImage(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected background image");
    };
    let [CssImageValue::Gradient(CssGradient::Linear(gradient))] = value.images().images() else {
        panic!("expected a linear gradient");
    };
    let CssColorStopListItem::Stop(parsed) = &gradient.stops().items()[0] else {
        panic!("expected first color stop");
    };
    let rebuilt =
        CssGradientColorStop::from_color(parsed.color().clone(), parsed.position().cloned());
    assert_eq!(**parsed, rebuilt);
}

#[test]
fn stop_list_requires_two_stops_and_separates_hints() {
    let red = || CssColorStopListItem::Stop(Box::new(stop("red", None)));
    let blue = || CssColorStopListItem::Stop(Box::new(stop("blue", None)));
    let hint = || CssColorStopListItem::Hint(signed_length_percentage("40%"));

    assert!(CssColorStopList::try_new(vec![red()]).is_none());
    assert!(CssColorStopList::try_new(vec![hint(), red(), blue()]).is_none());
    assert!(CssColorStopList::try_new(vec![red(), blue(), hint()]).is_none());
    assert!(CssColorStopList::try_new(vec![red(), hint(), hint(), blue()]).is_none());
    assert!(CssColorStopList::try_new(vec![red(), hint(), blue()]).is_some());
}

#[test]
fn linear_constructor_preserves_omitted_angle_and_side_directions() {
    let omitted = CssLinearGradient::new(None, stops());
    assert!(omitted.direction().is_none());
    assert_eq!(omitted.stops().items().len(), 2);

    let angle = CssLinearGradient::new(
        Some(CssLinearGradientDirection::Angle(
            CssGradientAngle::Literal(
                CssAngleLiteral::try_new(25.0, CssAngleUnit::Degrees).unwrap(),
            ),
        )),
        stops(),
    );
    assert!(
        matches!(angle.direction(), Some(CssLinearGradientDirection::Angle(CssGradientAngle::Literal(value))) if value.value() == 25.0)
    );

    let side = CssSideOrCorner::try_new(Some(CssHorizontalGradientSide::Right), None).unwrap();
    let directed = CssLinearGradient::new(
        Some(CssLinearGradientDirection::SideOrCorner(side)),
        stops(),
    );
    assert!(
        matches!(directed.direction(), Some(CssLinearGradientDirection::SideOrCorner(value)) if value.horizontal() == Some(CssHorizontalGradientSide::Right))
    );
    assert!(CssSideOrCorner::try_new(None, None).is_none());

    let symbolic_angle = CssAngleCalculation::try_from_components(
        parse_component_values("calc(25deg + 5deg)").unwrap(),
    )
    .unwrap();
    let calculated = CssLinearGradient::new(
        Some(CssLinearGradientDirection::Angle(
            CssGradientAngle::Calculation(symbolic_angle.clone()),
        )),
        stops(),
    );
    assert!(
        matches!(calculated.direction(), Some(CssLinearGradientDirection::Angle(CssGradientAngle::Calculation(value))) if value == &symbolic_angle)
    );
}

#[test]
fn radial_constructor_checks_shape_size_matrix_without_inventing_defaults() {
    let circle = CssRadialSize::Circle(nonnegative_length("10px"));
    let ellipse = CssRadialSize::Ellipse(CssRadialEllipseSize::new(
        nonnegative_length_percentage("10px"),
        nonnegative_length_percentage("20%"),
    ));
    let extent = CssRadialSize::Extent(CssRadialExtent::ClosestSide);

    assert!(
        CssRadialGradient::try_new(
            Some(CssRadialShape::Circle),
            Some(ellipse.clone()),
            None,
            stops()
        )
        .is_none()
    );
    assert!(
        CssRadialGradient::try_new(
            Some(CssRadialShape::Ellipse),
            Some(circle.clone()),
            None,
            stops()
        )
        .is_none()
    );

    for (shape, size) in [
        (None, None),
        (Some(CssRadialShape::Circle), None),
        (Some(CssRadialShape::Ellipse), None),
        (None, Some(circle.clone())),
        (None, Some(ellipse.clone())),
        (Some(CssRadialShape::Circle), Some(circle)),
        (Some(CssRadialShape::Ellipse), Some(ellipse)),
        (Some(CssRadialShape::Circle), Some(extent.clone())),
        (Some(CssRadialShape::Ellipse), Some(extent)),
    ] {
        let built = CssRadialGradient::try_new(shape, size.clone(), None, stops()).unwrap();
        assert_eq!(built.shape(), shape);
        assert_eq!(built.size(), size.as_ref());
        assert!(built.position().is_none());
    }

    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("-1", "px").unwrap()
        )
        .is_err()
    );
    assert!(
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_token("10%").unwrap()
        )
        .is_err()
    );

    let symbolic_radius = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(2px + 3px)").unwrap(),
    )
    .unwrap();
    let radius = CssSpecifiedNonNegativeLength::try_from_calculation(
        CssLengthCalculation::try_from_components(symbolic_radius.components().clone()).unwrap(),
    )
    .unwrap();
    let position =
        CssPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::Top).unwrap();
    let constructed = CssRadialGradient::try_new(
        Some(CssRadialShape::Circle),
        Some(CssRadialSize::Circle(radius)),
        Some(position.clone()),
        stops(),
    )
    .unwrap();
    assert_eq!(constructed.position(), Some(&position));
    assert!(
        matches!(constructed.size(), Some(CssRadialSize::Circle(radius)) if radius.calculation().is_some_and(|value| value.components() == symbolic_radius.components()))
    );
}

#[test]
fn generic_position_requires_edge_offsets_on_both_axes_or_neither() {
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::LeftOffset(signed_length_percentage("10px")),
            CssVerticalPosition::Center,
        )
        .is_none()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Center,
            CssVerticalPosition::BottomOffset(signed_length_percentage("20px")),
        )
        .is_none()
    );

    let paired = CssPosition::try_new(
        CssHorizontalPosition::RightOffset(signed_length_percentage("10px")),
        CssVerticalPosition::TopOffset(signed_length_percentage("20px")),
    )
    .unwrap();
    assert!(
        matches!(paired.horizontal(), CssHorizontalPosition::RightOffset(value) if exact_dimension(value.literal_component(), "10", "px"))
    );
    assert!(
        matches!(paired.vertical(), CssVerticalPosition::TopOffset(value) if exact_dimension(value.literal_component(), "20", "px"))
    );

    assert!(
        CssPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::Bottom).is_some()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Offset(signed_length_percentage("15px")),
            CssVerticalPosition::Center
        )
        .is_some()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Left,
            CssVerticalPosition::Offset(signed_length_percentage("25px"))
        )
        .is_some()
    );
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn nonnegative_length(css: &str) -> surgeist_css::CssSpecifiedNonNegativeLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedNonNegativeLength::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
fn signed_length_percentage(css: &str) -> surgeist_css::CssSpecifiedLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
fn nonnegative_length_percentage(
    css: &str,
) -> surgeist_css::CssSpecifiedNonNegativeLengthPercentage {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
            surgeist_css::CssLengthPercentageCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedNonNegativeLengthPercentage::try_from_component(
            surgeist_css::CssComponentValue::try_token(css).unwrap(),
        )
        .unwrap()
    }
}
