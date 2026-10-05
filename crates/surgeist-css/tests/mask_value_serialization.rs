#![forbid(unsafe_code)]
//! Represented Masking 1 §7.9 fields; full mask lifecycle remains outside this provider.
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#the-mask

use surgeist_css::*;
type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn parsed(css: &str) -> CssMaskList {
    let report = parse_style_attribute(&format!("mask:{css}"));
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Mask(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("mask")
    };
    value.value().clone()
}

fn bounds(value: &CssMaskList, expected: &str, input: usize, projection: usize) {
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(input, projection, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            Limits::new(input - 1, projection, expected.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(input, projection - 1, expected.len()),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(input, projection, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(*value, before);
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(*value, before);
}

#[test]
fn parsed_fields_emit_in_canonical_layer_order() {
    for (input, expected) in [
        ("none", "none"),
        ("no-repeat", "no-repeat"),
        ("center", "center center"),
        ("center / contain", "center center / contain"),
        (
            "no-repeat url(mask.png) center / contain",
            "url(\"mask.png\") center center / contain no-repeat",
        ),
        ("linear-gradient(red, blue)", "linear-gradient(red, blue)"),
        (
            "none, url(\"a\\\"b\"), linear-gradient(red, blue)",
            "none, url(\"a\\\"b\"), linear-gradient(red, blue)",
        ),
    ] {
        let value = parsed(input);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(parsed(expected).serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
    }
}

#[test]
fn constructed_size_synthesizes_position_without_changing_omission() {
    let value = CssMaskList::try_new(vec![
        CssMaskLayer::try_new(None, None, Some(CssBackgroundSize::Contain), None).unwrap(),
    ])
    .unwrap();
    bounds(&value, "0% 0% / contain", 3, 5);
    assert!(value.layers()[0].position().is_none());
    let explicit = parsed("0% 0% / contain");
    bounds(&explicit, "0% 0% / contain", 8, 8);
    assert!(explicit.layers()[0].position().is_some());
    assert_ne!(value, explicit);
}

#[test]
fn list_layer_and_authored_child_costs_are_exact() {
    bounds(&parsed("none"), "none", 3, 3);
    bounds(&parsed("none, none"), "none, none", 5, 5);
    bounds(&parsed("url(mask.png)"), "url(\"mask.png\")", 4, 4);
    // Gradient + stop list + two stop aggregates + two color leaves,
    // plus the enclosing mask list and mask layer.
    bounds(
        &parsed("linear-gradient(red, blue)"),
        "linear-gradient(red, blue)",
        8,
        8,
    );
    bounds(
        &parsed("center / contain no-repeat"),
        "center center / contain no-repeat",
        9,
        9,
    );
    bounds(
        &parsed("center / auto auto repeat repeat"),
        "center center / auto repeat",
        11,
        11,
    );
    bounds(&parsed("center / 10px"), "center center / 10px auto", 8, 9);
}

#[test]
fn programmatic_numeric_position_keeps_exact_token_and_origin_after_rounding() {
    let offset = CssSpecifiedLengthPercentage::try_from_component(
        CssComponentValue::try_dimension(".12345641", "px").unwrap(),
    )
    .unwrap();
    let position = CssPhysicalPosition::try_new(
        CssHorizontalPosition::Offset(offset),
        CssVerticalPosition::Top,
    )
    .unwrap();
    let value = CssMaskList::try_new(vec![
        CssMaskLayer::try_new(None, Some(position), None, None).unwrap(),
    ])
    .unwrap();
    bounds(&value, "0.123456px top", 6, 6);
    let CssHorizontalPosition::Offset(offset) = value.layers()[0].position().unwrap().horizontal()
    else {
        panic!("offset")
    };
    assert!(matches!(offset.origin(), CssValueOrigin::Programmatic));
    assert!(matches!(offset.literal_component().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
        if number.representation() == ".12345641" && unit == "px"));
}

#[test]
fn constructors_preserve_order_and_omitted_image_position_size_repeat() {
    let center =
        CssPhysicalPosition::try_new(CssHorizontalPosition::Center, CssVerticalPosition::Center)
            .unwrap();
    let value = CssMaskList::try_new(vec![
        CssMaskLayer::try_new(
            Some(CssImageValue::Url(CssUrl::new("first.svg"))),
            None,
            Some(CssBackgroundSize::Cover),
            None,
        )
        .unwrap(),
        CssMaskLayer::try_new(None, Some(center), None, Some(CssBackgroundRepeat::RepeatX))
            .unwrap(),
        CssMaskLayer::try_new(Some(CssImageValue::None), None, None, None).unwrap(),
    ])
    .unwrap();
    let before = value.clone();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "url(\"first.svg\") 0% 0% / cover, center center repeat-x, none"
    );
    assert_eq!(value, before);
    assert!(value.layers()[0].position().is_none());
    assert!(CssMaskList::try_new(vec![]).is_none());
    assert!(CssMaskLayer::try_new(None, None, None, None).is_none());
}

#[test]
fn symbolic_math_keeps_source_graph_and_non_bmp_origin_after_failure_and_retry() {
    let source = "/*😀*/mask:calc(10px + 1em) top / contain";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Mask(wrapper) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("mask")
    };
    let value = wrapper.value();
    let before = value.clone();
    let CssHorizontalPosition::Offset(offset) = value.layers()[0].position().unwrap().horizontal()
    else {
        panic!("offset")
    };
    let calculation = offset.calculation().unwrap();
    let components = calculation.components().clone();
    let CssValueOrigin::Parsed(origin) = offset.origin() else {
        panic!("parsed")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("calc(").unwrap()
    );
    let expected = "calc(1em + 10px) top / contain";
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(100, 100, 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(*value, before);
    assert_eq!(offset.calculation().unwrap().components(), &components);
}

#[test]
fn failure_after_first_layer_or_separator_is_atomic_and_retryable() {
    let value = parsed("none, none");
    let before = value.clone();
    for bytes in [4, 5, 6, 9] {
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(5, 5, bytes))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, before);
    }
    bounds(&value, "none, none", 5, 5);
}
