#![forbid(unsafe_code)]
//! Checked authored position construction for Values 4, Backgrounds 3, and Transforms 1.
use surgeist_css::*;

#[test]
fn generic_position_checks_edge_pairing_and_keeps_offset_origins() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let ordinary =
        CssPhysicalPosition::try_new(H::Offset(signed_length_percentage("15%")), V::Bottom)
            .unwrap();
    assert!(
        matches!(ordinary.horizontal(), H::Offset(v) if exact_percentage(v.literal_component(), "15"))
    );
    assert!(matches!(ordinary.vertical(), V::Bottom));

    let paired = CssPhysicalPosition::try_new(
        H::RightOffset(signed_length_percentage("10px")),
        V::TopOffset(signed_length_percentage("20%")),
    )
    .unwrap();
    assert!(
        matches!(paired.horizontal(), H::RightOffset(v) if exact_dimension(v.literal_component(), "10", "px"))
    );
    assert!(
        matches!(paired.vertical(), V::TopOffset(v) if exact_percentage(v.literal_component(), "20"))
    );
    assert!(
        CssPhysicalPosition::try_new(H::LeftOffset(signed_length_percentage("10px")), V::Top)
            .is_err()
    );
    assert!(
        CssPhysicalPosition::try_new(H::Right, V::BottomOffset(signed_length_percentage("20px")))
            .is_err()
    );
    assert!(
        CssPhysicalPosition::try_new(
            H::LeftOffset(signed_length_percentage("10px")),
            V::Offset(signed_length_percentage("20px"))
        )
        .is_err()
    );
    assert!(
        CssPhysicalPosition::try_new(
            H::Offset(signed_length_percentage("10px")),
            V::BottomOffset(signed_length_percentage("20px"))
        )
        .is_err()
    );
}

#[test]
fn generic_position_retains_symbolic_offsets_and_nonempty_list_order() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let symbolic = CssPhysicalPosition::try_new(
        H::Offset(signed_length_percentage("calc(10px + 20%)")),
        V::Center,
    )
    .unwrap();
    assert!(
        matches!(symbolic.horizontal(), H::Offset(v) if v.calculation().is_some_and(|calc| calc.components().serialize().unwrap().as_css() == "calc(10px + 20%)"))
    );
    let keyword = CssPhysicalPosition::try_new(H::Left, V::Top).unwrap();
    let list = CssPhysicalPositionList::try_new(vec![symbolic, keyword]).unwrap();
    assert_eq!(list.positions().len(), 2);
    assert!(matches!(list.positions()[0].horizontal(), H::Offset(_)));
    assert!(matches!(list.positions()[0].vertical(), V::Center));
    assert!(matches!(list.positions()[1].horizontal(), H::Left));
    assert!(matches!(list.positions()[1].vertical(), V::Top));
    assert!(CssPhysicalPositionList::try_new(Vec::new()).is_none());
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_ident("auto").unwrap()
        )
        .is_err()
    );
}

#[test]
fn background_position_adds_only_its_three_component_edge_forms() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let ordinary =
        CssBackgroundPosition::try_new(H::Offset(signed_length_percentage("40%")), V::Bottom)
            .unwrap();
    assert!(
        matches!(ordinary.horizontal(), H::Offset(v) if exact_percentage(v.literal_component(), "40"))
    );
    assert!(matches!(ordinary.vertical(), V::Bottom));

    let horizontal_edge =
        CssBackgroundPosition::try_new(H::LeftOffset(signed_length_percentage("12px")), V::Top)
            .unwrap();
    assert!(
        matches!(horizontal_edge.horizontal(), H::LeftOffset(v) if exact_dimension(v.literal_component(), "12", "px"))
    );
    assert!(matches!(horizontal_edge.vertical(), V::Top));

    let vertical_edge =
        CssBackgroundPosition::try_new(H::Center, V::BottomOffset(signed_length_percentage("30%")))
            .unwrap();
    assert!(matches!(vertical_edge.horizontal(), H::Center));
    assert!(
        matches!(vertical_edge.vertical(), V::BottomOffset(v) if exact_percentage(v.literal_component(), "30"))
    );

    let paired = CssBackgroundPosition::try_new(
        H::RightOffset(signed_length_percentage("1px")),
        V::TopOffset(signed_length_percentage("2px")),
    )
    .unwrap();
    assert!(matches!(paired.horizontal(), H::RightOffset(_)));
    assert!(matches!(paired.vertical(), V::TopOffset(_)));
    assert!(
        CssBackgroundPosition::try_new(
            H::LeftOffset(signed_length_percentage("1px")),
            V::Offset(signed_length_percentage("2px"))
        )
        .is_none()
    );
    assert!(
        CssBackgroundPosition::try_new(
            H::Offset(signed_length_percentage("1px")),
            V::BottomOffset(signed_length_percentage("2px"))
        )
        .is_none()
    );

    let list = CssBackgroundPositionList::try_new(vec![vertical_edge, horizontal_edge]).unwrap();
    assert_eq!(list.positions().len(), 2);
    assert!(matches!(list.positions()[0].vertical(), V::BottomOffset(_)));
    assert!(matches!(list.positions()[1].horizontal(), H::LeftOffset(_)));
    assert!(CssBackgroundPositionList::try_new(Vec::new()).is_none());
}

#[test]
fn transform_origin_checks_planar_grammar_and_keeps_optional_pure_length_z() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let planar = CssPhysicalPosition::try_new(H::Center, V::Top).unwrap();
    let no_z = CssTransformOrigin::try_new(planar.clone(), None).unwrap();
    assert!(matches!(no_z.horizontal(), H::Center));
    assert!(matches!(no_z.vertical(), V::Top));
    assert!(no_z.z().is_none());

    // The two explicit semantic axes are valid with Z; the source spelling
    // `top 10px` is rejected by the selected parser grammar separately.
    let z = signed_length("10px");
    let with_z = CssTransformOrigin::try_new(planar, Some(z)).unwrap();
    assert!(matches!(with_z.horizontal(), H::Center));
    assert!(matches!(with_z.vertical(), V::Top));
    assert!(exact_dimension(
        with_z.z().unwrap().literal_component(),
        "10",
        "px"
    ));

    let pure_symbolic = signed_length("calc(2px + 3px)");
    let symbolic = CssTransformOrigin::try_new(
        CssPhysicalPosition::try_new(H::Left, V::Bottom).unwrap(),
        Some(pure_symbolic),
    )
    .unwrap();
    assert!(
        symbolic.z().unwrap().calculation().is_some_and(|calc| calc
            .components()
            .serialize()
            .unwrap()
            .as_css()
            == "calc(2px + 3px)")
    );

    let edge_pair = CssPhysicalPosition::try_new(
        H::LeftOffset(signed_length_percentage("1px")),
        V::TopOffset(signed_length_percentage("2px")),
    )
    .unwrap();
    assert!(CssTransformOrigin::try_new(edge_pair, None).is_none());
    assert!(
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("25%").unwrap())
            .is_err()
    );
    assert!(
        CssLengthCalculation::try_from_components(
            parse_component_values("calc(2px + 3%)").unwrap()
        )
        .is_err()
    );
}

#[test]
fn mask_and_shape_consumers_accept_the_generic_position_without_reinterpretation() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let position = CssPhysicalPosition::try_new(H::Right, V::Bottom).unwrap();
    assert!(CssMaskLayer::try_new(None, None, None, None).is_none());
    let mask = CssMaskLayer::try_new(None, Some(position.clone()), None, None).unwrap();
    assert!(matches!(mask.position().unwrap().horizontal(), H::Right));
    assert!(matches!(mask.position().unwrap().vertical(), V::Bottom));

    let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position.clone().into()));
    let CssPositionRef::Cartesian(circle_position) = circle.position().unwrap().view() else {
        panic!("Cartesian circle position")
    };
    assert!(matches!(circle_position.horizontal(), H::Right));
    assert!(matches!(circle_position.vertical(), V::Bottom));
    let ellipse = CssEllipseShape::new(None, Some(position.clone().into()));
    let CssPositionRef::Cartesian(ellipse_position) = ellipse.position().unwrap().view() else {
        panic!("Cartesian ellipse position")
    };
    assert!(matches!(ellipse_position.horizontal(), H::Right));
    assert!(matches!(ellipse_position.vertical(), V::Bottom));

    let red = CssColor::from_named(CssNamedColor::try_new("red").unwrap());
    let blue = CssColor::from_named(CssNamedColor::try_new("blue").unwrap());
    let stops = CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(red, None))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(blue, None))),
    ])
    .unwrap();
    let radial = CssRadialGradient::try_new(None, None, Some(position), stops).unwrap();
    assert!(matches!(radial.position().unwrap().horizontal(), H::Right));
    assert!(matches!(radial.position().unwrap().vertical(), V::Bottom));
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension { number, unit })) if number.representation() == representation && unit == expected_unit)
}
fn exact_percentage(
    component: Option<&surgeist_css::CssComponentValue>,
    representation: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view), Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Percentage(number))) if number.representation() == representation)
}

fn signed_length(css: &str) -> surgeist_css::CssSpecifiedLength {
    let components = surgeist_css::parse_component_values(css).unwrap();
    if css.contains('(') {
        surgeist_css::CssSpecifiedLength::try_from_calculation(
            surgeist_css::CssLengthCalculation::try_from_components(components).unwrap(),
        )
        .unwrap()
    } else {
        surgeist_css::CssSpecifiedLength::try_from_component(
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
