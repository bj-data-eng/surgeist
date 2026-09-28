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

fn stop(css: &str, position: Option<CssGradientLinePosition>) -> CssGradientColorStop {
    CssGradientColorStop::from_color(color(css), position)
}

fn stops() -> CssColorStopList {
    CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(stop("red", None))),
        CssColorStopListItem::Stop(Box::new(stop("blue", None))),
    ])
    .unwrap()
}

fn px(value: f32) -> CssLength {
    CssLength::try_px(value).unwrap()
}

fn offset(value: f32) -> CssPositionOffset {
    CssPositionOffset::try_new(px(value)).unwrap()
}

#[test]
fn image_only_boundary_rejects_property_none_and_retains_all_published_families() {
    assert!(CssImage::try_new(CssImageValue::None).is_none());

    let url = CssUrl::try_new("figure.svg").unwrap();
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
    let negative = CssGradientLinePosition::try_new(px(-1.0)).unwrap();
    let authored = color("currentcolor");
    let built = CssGradientColorStop::from_color(authored.clone(), Some(negative));
    assert_eq!(built.color(), &authored);
    assert!(
        matches!(built.position().unwrap().value(), CssLength::Px(value) if value.value() == -1.0)
    );
    assert_eq!(built.color().to_specified_css().unwrap(), "currentcolor");
    assert!(CssGradientLinePosition::try_new(CssLength::Auto).is_none());

    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(1px + 5%)").unwrap(),
    )
    .unwrap();
    let symbolic = CssGradientLinePosition::try_new(CssLength::Calc(CssCalcLength::Typed(
        calculation.clone(),
    )))
    .unwrap();
    let stop = CssGradientColorStop::from_color(color("red"), Some(symbolic));
    assert!(
        matches!(stop.position().unwrap().value(), CssLength::Calc(CssCalcLength::Typed(value)) if value == &calculation)
    );
}

#[test]
fn current_gradient_stop_equality_ignores_unexposed_legacy_color_projection() {
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
    let hint = || {
        CssColorStopListItem::Hint(
            CssGradientLinePosition::try_new(CssLength::try_percent(40.0).unwrap()).unwrap(),
        )
    };

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
    let circle = CssRadialSize::Circle(CssRadialCircleSize::try_new(px(10.0)).unwrap());
    let ellipse = CssRadialSize::Ellipse(
        CssRadialEllipseSize::try_new(px(10.0), CssLength::try_percent(20.0).unwrap()).unwrap(),
    );
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

    assert!(CssRadialCircleSize::try_new(px(-1.0)).is_none());
    assert!(CssRadialCircleSize::try_new(CssLength::try_percent(10.0).unwrap()).is_none());

    let symbolic_radius = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(2px + 3px)").unwrap(),
    )
    .unwrap();
    let radius = CssRadialCircleSize::try_new(CssLength::Calc(CssCalcLength::Typed(
        symbolic_radius.clone(),
    )))
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
        matches!(constructed.size(), Some(CssRadialSize::Circle(radius)) if matches!(radius.radius(), CssLength::Calc(CssCalcLength::Typed(value)) if value == &symbolic_radius))
    );
}

#[test]
fn generic_position_requires_edge_offsets_on_both_axes_or_neither() {
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::LeftOffset(offset(10.0)),
            CssVerticalPosition::Center,
        )
        .is_none()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Center,
            CssVerticalPosition::BottomOffset(offset(20.0)),
        )
        .is_none()
    );

    let paired = CssPosition::try_new(
        CssHorizontalPosition::RightOffset(offset(10.0)),
        CssVerticalPosition::TopOffset(offset(20.0)),
    )
    .unwrap();
    assert!(
        matches!(paired.horizontal(), CssHorizontalPosition::RightOffset(value) if matches!(value.value(), CssLength::Px(n) if n.value() == 10.0))
    );
    assert!(
        matches!(paired.vertical(), CssVerticalPosition::TopOffset(value) if matches!(value.value(), CssLength::Px(n) if n.value() == 20.0))
    );

    assert!(
        CssPosition::try_new(CssHorizontalPosition::Left, CssVerticalPosition::Bottom).is_some()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Offset(offset(15.0)),
            CssVerticalPosition::Center
        )
        .is_some()
    );
    assert!(
        CssPosition::try_new(
            CssHorizontalPosition::Left,
            CssVerticalPosition::Offset(offset(25.0))
        )
        .is_some()
    );
}
