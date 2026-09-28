#![forbid(unsafe_code)]
//! Images 3 specified serialization: https://www.w3.org/TR/2023/CRD-css-images-3-20231218/#serialization

use surgeist_css::*;

fn images(css: &str) -> CssImageValueList {
    let report = parse_style_attribute(&format!("background-image: {css}"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BackgroundImage(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected typed background-image");
    };
    value.images().clone()
}

fn only(css: &str) -> CssImageValue {
    images(css).images()[0].clone()
}

fn color(css: &str) -> CssColor {
    let report = parse_style_attribute(&format!("color: {css}"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected typed color");
    };
    value.value().clone()
}

fn kind(
    value: &CssImageValueList,
    limits: CssSpecifiedValueSerializationLimits,
) -> CssSpecifiedValueSerializationErrorKind {
    value
        .serialize_specified_with_limits(limits)
        .unwrap_err()
        .kind()
}

#[test]
fn linear_default_direction_and_endpoint_positions_follow_images3_example() {
    let value = only("Linear-Gradient( to bottom, red 0%, yellow, black 100px)");
    assert_eq!(
        value.serialize_specified().unwrap(),
        "linear-gradient(red, yellow, black 100px)"
    );
    for css in [
        "linear-gradient(180deg, red 0px, blue 100%)",
        "linear-gradient(200grad, red 0em, blue 100%)",
        "linear-gradient(0.5turn, red 0%, blue 100%)",
    ] {
        assert_eq!(
            only(css).serialize_specified().unwrap(),
            "linear-gradient(red, blue)"
        );
    }
    assert_eq!(
        only("linear-gradient(3.1415927rad, red, blue)")
            .serialize_specified()
            .unwrap(),
        "linear-gradient(3.1415927rad, red, blue)"
    );
    assert_eq!(
        only("linear-gradient(to right top, red 10%, 30%, blue 100%)")
            .serialize_specified()
            .unwrap(),
        "linear-gradient(to right top, red 10%, 30%, blue)"
    );
}

#[test]
fn radial_shape_size_and_position_omit_only_context_independent_defaults() {
    for css in [
        "radial-gradient(ellipse farthest-corner at center, red, blue)",
        "radial-gradient(farthest-corner at 50% 50%, red, blue)",
    ] {
        assert_eq!(
            only(css).serialize_specified().unwrap(),
            "radial-gradient(red, blue)"
        );
    }
    assert_eq!(
        only("radial-gradient(circle farthest-corner at center, red, blue)")
            .serialize_specified()
            .unwrap(),
        "radial-gradient(circle, red, blue)"
    );
    assert_eq!(
        only("radial-gradient(circle 20px at left 10px top 20%, red, blue)")
            .serialize_specified()
            .unwrap(),
        "radial-gradient(20px at left 10px top 20%, red, blue)"
    );
    assert_eq!(
        only("radial-gradient(ellipse 20% 25% at center, red, blue)")
            .serialize_specified()
            .unwrap(),
        "radial-gradient(20% 25%, red, blue)"
    );
    assert_eq!(
        only("radial-gradient(at left 50% top 50%, red, blue)")
            .serialize_specified()
            .unwrap(),
        "radial-gradient(at left 50% top 50%, red, blue)"
    );
}

#[test]
fn repeat_identity_hints_color_and_symbolic_math_are_preserved() {
    assert_eq!(
        only("repeating-linear-gradient(calc(90deg + 5deg), currentcolor calc(1px + 5%), 25%, transparent 100%)")
            .serialize_specified()
            .unwrap(),
        "repeating-linear-gradient(calc(95deg), currentcolor calc(5% + 1px), 25%, transparent)"
    );
    assert_eq!(
        only("repeating-radial-gradient(circle closest-side at right bottom, #000, #fff)")
            .serialize_specified()
            .unwrap(),
        "repeating-radial-gradient(circle closest-side at right bottom, rgb(0, 0, 0), rgb(255, 255, 255))"
    );
}

#[test]
fn checked_gradient_construction_has_independent_specified_text() {
    let stops = CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            color("currentcolor"),
            None,
        ))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            color("transparent"),
            Some(CssGradientLinePosition::try_new(CssLength::try_percent(50.0).unwrap()).unwrap()),
        ))),
    ])
    .unwrap();
    let direction = CssLinearGradientDirection::SideOrCorner(
        CssSideOrCorner::try_new(Some(CssHorizontalGradientSide::Left), None).unwrap(),
    );
    let gradient = CssGradient::Linear(CssLinearGradient::new(Some(direction), stops));
    assert_eq!(
        gradient.serialize_specified().unwrap(),
        "linear-gradient(to left, currentcolor, transparent 50%)"
    );
    let image = CssImage::try_new(CssImageValue::Gradient(gradient)).unwrap();
    assert_eq!(
        image.serialize_specified().unwrap(),
        "linear-gradient(to left, currentcolor, transparent 50%)"
    );
}

#[test]
fn calculated_default_like_angles_and_positions_remain_explicit() {
    assert_eq!(
        only("linear-gradient(calc(180deg), red, blue)")
            .serialize_specified()
            .unwrap(),
        "linear-gradient(calc(180deg), red, blue)"
    );
    assert_eq!(
        only("radial-gradient(at calc(50%) center, red, blue)")
            .serialize_specified()
            .unwrap(),
        "radial-gradient(at calc(50%) center, red, blue)"
    );
}

#[test]
fn image_list_url_src_none_and_transparent_image_dispatch() {
    let list = images("none, url(\"a\\\"b\"), src(\"#id\"), linear-gradient(red, blue)");
    assert_eq!(
        list.serialize_specified().unwrap(),
        "none, url(\"a\\\"b\"), src(\"#id\"), linear-gradient(red, blue)"
    );
    for image in list.images().iter().skip(1) {
        let checked = CssImage::try_new(image.clone()).unwrap();
        assert_eq!(
            checked.serialize_specified().unwrap(),
            image.serialize_specified().unwrap()
        );
    }
    assert!(CssImage::try_new(CssImageValue::None).is_none());
}

#[test]
fn list_and_gradient_node_budgets_are_cumulative_even_for_omitted_defaults() {
    let list = images("linear-gradient(to bottom, red 0%, blue 100%), none");
    // list 1; gradient 1; direction 1 + bottom 1; stops 1;
    // each stop 1 + color 1 + omitted endpoint 1; none 1 = 12.
    let enough = CssSpecifiedValueSerializationLimits::new(12, 12, 100);
    assert_eq!(
        list.serialize_specified_with_limits(enough).unwrap(),
        "linear-gradient(red, blue), none"
    );
    assert_eq!(
        kind(
            &list,
            CssSpecifiedValueSerializationLimits::new(11, 12, 100)
        ),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        kind(
            &list,
            CssSpecifiedValueSerializationLimits::new(12, 11, 100)
        ),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        kind(&list, CssSpecifiedValueSerializationLimits::new(12, 12, 31)),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        list.serialize_specified().unwrap(),
        "linear-gradient(red, blue), none"
    );
}

#[test]
fn nested_numeric_and_color_work_share_limits_with_sibling_images() {
    let list = images(
        "linear-gradient(red calc(1px + 5%), blue), linear-gradient(currentcolor, transparent)",
    );
    let expected =
        "linear-gradient(red calc(5% + 1px), blue), linear-gradient(currentcolor, transparent)";
    assert_eq!(list.serialize_specified().unwrap(), expected);
    assert_eq!(
        kind(
            &list,
            CssSpecifiedValueSerializationLimits::new(0, 100, 200)
        ),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        kind(
            &list,
            CssSpecifiedValueSerializationLimits::new(100, 0, 200)
        ),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        list.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            100,
            100,
            expected.len()
        ))
        .unwrap(),
        expected
    );
    assert_eq!(
        kind(
            &list,
            CssSpecifiedValueSerializationLimits::new(100, 100, expected.len() - 1)
        ),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}

#[test]
fn radial_omissions_charge_shape_size_and_each_center_axis() {
    let value = only("radial-gradient(ellipse farthest-corner at 50% 50%, red, blue)");
    // Gradient 1; omitted shape 1; size aggregate + extent 2;
    // checked position aggregate + two axes + two literal offsets 5;
    // stop list 1; two stop aggregates and two color leaves 4 = 14.
    let expected = "radial-gradient(red, blue)";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                14,
                14,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(13, 14, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(14, 13, 100))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}

#[test]
fn two_calculation_graphs_share_one_image_list_budget() {
    let single = images("linear-gradient(red calc(1px + 5%), blue)");
    let pair = images(
        "linear-gradient(red calc(1px + 5%), blue), linear-gradient(red calc(1px + 5%), blue)",
    );
    let limits = CssSpecifiedValueSerializationLimits::new(20, 20, 200);
    assert!(single.serialize_specified_with_limits(limits).is_ok());
    assert_eq!(
        kind(
            &pair,
            CssSpecifiedValueSerializationLimits::new(20, 100, 200)
        ),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        kind(
            &pair,
            CssSpecifiedValueSerializationLimits::new(100, 20, 200)
        ),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
}
