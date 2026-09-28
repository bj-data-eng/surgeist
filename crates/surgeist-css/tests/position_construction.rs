#![forbid(unsafe_code)]
//! Checked authored position construction for Values 4, Backgrounds 3, and Transforms 1.
use surgeist_css::*;

fn px(value: f32) -> CssLength {
    CssLength::try_px(value).unwrap()
}
fn percent(value: f32) -> CssLength {
    CssLength::try_percent(value).unwrap()
}
fn offset(value: CssLength) -> CssPositionOffset {
    CssPositionOffset::try_new(value).unwrap()
}
fn calculation(text: &str) -> CssLength {
    CssLength::Calc(CssCalcLength::Typed(
        CssLengthPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    ))
}

#[test]
fn generic_position_checks_edge_pairing_and_keeps_offset_origins() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let ordinary = CssPosition::try_new(H::Offset(offset(percent(15.0))), V::Bottom).unwrap();
    assert!(
        matches!(ordinary.horizontal(), H::Offset(v) if matches!(v.value(), CssLength::Percent(n) if n.value() == 15.0))
    );
    assert!(matches!(ordinary.vertical(), V::Bottom));

    let paired = CssPosition::try_new(
        H::RightOffset(offset(px(10.0))),
        V::TopOffset(offset(percent(20.0))),
    )
    .unwrap();
    assert!(
        matches!(paired.horizontal(), H::RightOffset(v) if matches!(v.value(), CssLength::Px(n) if n.value() == 10.0))
    );
    assert!(
        matches!(paired.vertical(), V::TopOffset(v) if matches!(v.value(), CssLength::Percent(n) if n.value() == 20.0))
    );
    assert!(CssPosition::try_new(H::LeftOffset(offset(px(10.0))), V::Top).is_none());
    assert!(CssPosition::try_new(H::Right, V::BottomOffset(offset(px(20.0)))).is_none());
    assert!(
        CssPosition::try_new(H::LeftOffset(offset(px(10.0))), V::Offset(offset(px(20.0))))
            .is_none()
    );
    assert!(
        CssPosition::try_new(
            H::Offset(offset(px(10.0))),
            V::BottomOffset(offset(px(20.0)))
        )
        .is_none()
    );
}

#[test]
fn generic_position_retains_symbolic_offsets_and_nonempty_list_order() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let symbolic = CssPosition::try_new(
        H::Offset(offset(calculation("calc(10px + 20%)"))),
        V::Center,
    )
    .unwrap();
    assert!(
        matches!(symbolic.horizontal(), H::Offset(v) if matches!(v.value(), CssLength::Calc(CssCalcLength::Typed(calc)) if calc.components().serialize().unwrap().as_css() == "calc(10px + 20%)"))
    );
    let keyword = CssPosition::try_new(H::Left, V::Top).unwrap();
    let list = CssPositionList::try_new(vec![symbolic, keyword]).unwrap();
    assert_eq!(list.positions().len(), 2);
    assert!(matches!(list.positions()[0].horizontal(), H::Offset(_)));
    assert!(matches!(list.positions()[0].vertical(), V::Center));
    assert!(matches!(list.positions()[1].horizontal(), H::Left));
    assert!(matches!(list.positions()[1].vertical(), V::Top));
    assert!(CssPositionList::try_new(Vec::new()).is_none());
    assert!(CssPositionOffset::try_new(CssLength::Auto).is_none());
}

#[test]
fn background_position_adds_only_its_three_component_edge_forms() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let ordinary =
        CssBackgroundPosition::try_new(H::Offset(offset(percent(40.0))), V::Bottom).unwrap();
    assert!(
        matches!(ordinary.horizontal(), H::Offset(v) if matches!(v.value(), CssLength::Percent(n) if n.value() == 40.0))
    );
    assert!(matches!(ordinary.vertical(), V::Bottom));

    let horizontal_edge =
        CssBackgroundPosition::try_new(H::LeftOffset(offset(px(12.0))), V::Top).unwrap();
    assert!(
        matches!(horizontal_edge.horizontal(), H::LeftOffset(v) if matches!(v.value(), CssLength::Px(n) if n.value() == 12.0))
    );
    assert!(matches!(horizontal_edge.vertical(), V::Top));

    let vertical_edge =
        CssBackgroundPosition::try_new(H::Center, V::BottomOffset(offset(percent(30.0)))).unwrap();
    assert!(matches!(vertical_edge.horizontal(), H::Center));
    assert!(
        matches!(vertical_edge.vertical(), V::BottomOffset(v) if matches!(v.value(), CssLength::Percent(n) if n.value() == 30.0))
    );

    let paired = CssBackgroundPosition::try_new(
        H::RightOffset(offset(px(1.0))),
        V::TopOffset(offset(px(2.0))),
    )
    .unwrap();
    assert!(matches!(paired.horizontal(), H::RightOffset(_)));
    assert!(matches!(paired.vertical(), V::TopOffset(_)));
    assert!(
        CssBackgroundPosition::try_new(H::LeftOffset(offset(px(1.0))), V::Offset(offset(px(2.0))))
            .is_none()
    );
    assert!(
        CssBackgroundPosition::try_new(
            H::Offset(offset(px(1.0))),
            V::BottomOffset(offset(px(2.0)))
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

    let planar = CssPosition::try_new(H::Center, V::Top).unwrap();
    let no_z = CssTransformOrigin::try_new(planar.clone(), None).unwrap();
    assert!(matches!(no_z.horizontal(), H::Center));
    assert!(matches!(no_z.vertical(), V::Top));
    assert!(no_z.z().is_none());

    // The two explicit semantic axes are valid with Z; the source spelling
    // `top 10px` is rejected by the selected parser grammar separately.
    let z = CssTransformOriginZ::try_new(px(10.0)).unwrap();
    let with_z = CssTransformOrigin::try_new(planar, Some(z)).unwrap();
    assert!(matches!(with_z.horizontal(), H::Center));
    assert!(matches!(with_z.vertical(), V::Top));
    assert!(matches!(with_z.z().unwrap().value(), CssLength::Px(n) if n.value() == 10.0));

    let pure_symbolic = CssTransformOriginZ::try_new(calculation("calc(2px + 3px)")).unwrap();
    let symbolic = CssTransformOrigin::try_new(
        CssPosition::try_new(H::Left, V::Bottom).unwrap(),
        Some(pure_symbolic),
    )
    .unwrap();
    assert!(
        matches!(symbolic.z().unwrap().value(), CssLength::Calc(CssCalcLength::Typed(calc)) if calc.components().serialize().unwrap().as_css() == "calc(2px + 3px)")
    );

    let edge_pair = CssPosition::try_new(
        H::LeftOffset(offset(px(1.0))),
        V::TopOffset(offset(px(2.0))),
    )
    .unwrap();
    assert!(CssTransformOrigin::try_new(edge_pair, None).is_none());
    assert!(CssTransformOriginZ::try_new(percent(25.0)).is_none());
    assert!(CssTransformOriginZ::try_new(calculation("calc(2px + 3%)")).is_none());
}

#[test]
fn mask_and_shape_consumers_accept_the_generic_position_without_reinterpretation() {
    use CssHorizontalPosition as H;
    use CssVerticalPosition as V;

    let position = CssPosition::try_new(H::Right, V::Bottom).unwrap();
    assert!(CssMaskLayer::try_new(None, None, None, None).is_none());
    let mask = CssMaskLayer::try_new(None, Some(position.clone()), None, None).unwrap();
    assert!(matches!(mask.position().unwrap().horizontal(), H::Right));
    assert!(matches!(mask.position().unwrap().vertical(), V::Bottom));

    let circle = CssCircleShape::new(CssCircleRadius::Default, Some(position.clone()));
    assert!(matches!(circle.position().unwrap().horizontal(), H::Right));
    assert!(matches!(circle.position().unwrap().vertical(), V::Bottom));
    let ellipse = CssEllipseShape::new(CssEllipseRadius::Default, Some(position.clone()));
    assert!(matches!(ellipse.position().unwrap().horizontal(), H::Right));
    assert!(matches!(ellipse.position().unwrap().vertical(), V::Bottom));

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
