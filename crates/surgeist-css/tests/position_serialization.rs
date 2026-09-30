#![forbid(unsafe_code)]
//! Values 4 (2024-03-12) §8.3.2, specified generic `<position>` serialization.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#position-serialization

use surgeist_css::{
    CssHorizontalPosition as H, CssSpecifiedValueSerializationErrorKind as ErrorKind,
    CssSpecifiedValueSerializationLimits as L, CssVerticalPosition as V, *,
};

fn parsed(input: &str) -> CssPosition {
    let report = parse_style_attribute(&format!("mask-position: {input}"));
    assert!(report.is_clean(), "{input}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::MaskPosition(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected mask-position");
    };
    value.positions().positions()[0].clone()
}

#[test]
fn specified_axes_are_explicit_and_horizontal_first() {
    for (authored, expected) in [
        ("top", "center top"),
        ("left", "left center"),
        ("center", "center center"),
        ("bottom right", "right bottom"),
        ("25%", "25% center"),
        ("25% 75%", "25% 75%"),
        ("bottom 2% right 1px", "right 1px bottom 2%"),
    ] {
        assert_eq!(parsed(authored).serialize_specified().unwrap(), expected);
    }
}

#[test]
fn typed_axes_preserve_offset_origin_and_symbolic_values() {
    let paired = CssPosition::try_new(
        H::RightOffset(signed_length_percentage("0px")),
        V::BottomOffset(signed_length_percentage("2%")),
    )
    .unwrap();
    assert!(matches!(paired.horizontal(), H::RightOffset(_)));
    assert!(matches!(paired.vertical(), V::BottomOffset(_)));
    assert_eq!(paired.serialize_specified().unwrap(), "right 0px bottom 2%");
    assert!(
        CssPosition::try_new(H::RightOffset(signed_length_percentage("0px")), V::Top).is_none()
    );

    let signed = CssPosition::try_new(
        H::Offset(signed_length_percentage("-1px")),
        V::Offset(signed_length_percentage("-2%")),
    )
    .unwrap();
    assert_eq!(signed.serialize_specified().unwrap(), "-1px -2%");

    let calculation = CssLengthPercentageCalculation::try_from_components(
        parse_component_values("calc(1px + 5%)").unwrap(),
    )
    .unwrap();
    let offset = CssSpecifiedLengthPercentage::try_from_calculation(calculation).unwrap();
    let symbolic = CssPosition::try_new(H::Offset(offset), V::Center).unwrap();
    assert_eq!(
        symbolic.serialize_specified().unwrap(),
        "calc(5% + 1px) center"
    );
}

#[test]
fn one_budget_covers_aggregate_axes_children_and_full_output() {
    let keywords = CssPosition::try_new(H::Left, V::Top).unwrap();
    assert_eq!(
        keywords
            .serialize_specified_with_limits(L::new(3, 3, 8))
            .unwrap(),
        "left top"
    );
    for (limits, expected) in [
        (L::new(2, 3, 8), ErrorKind::InputNodeLimit),
        (L::new(3, 2, 8), ErrorKind::ProjectionNodeLimit),
        (L::new(3, 3, 7), ErrorKind::ByteLimit),
        (L::new(0, 3, 8), ErrorKind::InputNodeLimit),
        (L::new(3, 0, 8), ErrorKind::ProjectionNodeLimit),
        (L::new(3, 3, 0), ErrorKind::ByteLimit),
    ] {
        assert_eq!(
            keywords
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }

    let offsets = CssPosition::try_new(
        H::RightOffset(signed_length_percentage("1px")),
        V::BottomOffset(signed_length_percentage("2%")),
    )
    .unwrap();
    let expected = "right 1px bottom 2%";
    assert_eq!(
        offsets
            .serialize_specified_with_limits(L::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(4, 5, expected.len()), ErrorKind::InputNodeLimit),
        (L::new(5, 4, expected.len()), ErrorKind::ProjectionNodeLimit),
        (L::new(5, 5, expected.len() - 1), ErrorKind::ByteLimit),
    ] {
        assert_eq!(
            offsets
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}

#[test]
fn typed_math_projection_is_cumulative_across_both_axes() {
    let axis = || signed_length_percentage("calc(1px + 2em)");
    let one = CssPosition::try_new(H::Offset(axis()), V::Center).unwrap();
    let two = CssPosition::try_new(H::Offset(axis()), V::Offset(axis())).unwrap();
    let one_css = one.serialize_specified().unwrap();
    let two_css = two.serialize_specified().unwrap();
    assert_eq!(one_css, "calc(2em + 1px) center");
    assert_eq!(two_css, "calc(2em + 1px) calc(2em + 1px)");

    // The position and its axes cost three nodes. Each checked calc sum has
    // a function, sum and two literal input nodes (four), then two literals,
    // two canonical scalar terms and a sum projection node (five).
    assert_eq!(
        one.serialize_specified_with_limits(L::new(7, 8, 1_000))
            .unwrap(),
        one_css
    );
    assert_eq!(
        two.serialize_specified_with_limits(L::new(11, 13, 1_000))
            .unwrap(),
        two_css
    );
    for (limits, kind) in [
        (L::new(7, 1_000, 1_000), ErrorKind::InputNodeLimit),
        (L::new(1_000, 8, 1_000), ErrorKind::ProjectionNodeLimit),
        (L::new(10, 13, 1_000), ErrorKind::InputNodeLimit),
        (L::new(11, 12, 1_000), ErrorKind::ProjectionNodeLimit),
    ] {
        assert_eq!(
            two.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    let one_limit = L::new(1_000, 1_000, one_css.len());
    assert!(one.serialize_specified_with_limits(one_limit).is_ok());
    assert_eq!(
        two.serialize_specified_with_limits(one_limit)
            .unwrap_err()
            .kind(),
        ErrorKind::ByteLimit
    );
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
