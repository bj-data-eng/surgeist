#![forbid(unsafe_code)]
//! CSSOM WD 2021-08-26 §6.7.2 selects dppx, six-place Number text and
//! comma-separated legacy rect() edges; Values 4 supplies the unit relations.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#resolution
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#clip-property
//! Number halfway output follows the shared frozen WebKit FIXED policy;
//! CSSOM's normative decimal-halfway question remains open.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn literal(text: &str) -> CssResolutionLiteral {
    let values = parse_component_values(text).unwrap();
    let component = values
        .items()
        .iter()
        .find(|component| {
            matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { .. })
            )
        })
        .unwrap();
    CssResolutionLiteral::try_from_component(component.clone()).unwrap()
}

fn resolution(text: &str) -> CssResolutionValue {
    CssResolutionValue::try_from_calculation(
        CssResolutionCalculation::try_from_components(parse_component_values(text).unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn ordinary_resolution_converts_exact_units_before_six_place_rounding() {
    for (text, expected) in [
        ("192dpi", "2dppx"),
        ("1dpi", "0.010417dppx"),
        ("1dpcm", "0.026458dppx"),
        ("4800dpcm", "127dppx"),
        ("2X", "2dppx"),
        ("+0002.000DPPX", "2dppx"),
        ("0.000048dpi", "0.000001dppx"),
        ("0.000047999999999999999999999999dpi", "0dppx"),
        ("0.000024dpi", "0dppx"),
        ("0.0000005dppx", "0.000001dppx"),
        ("0.000000499999999999999999999999dppx", "0dppx"),
    ] {
        let value = literal(text);
        let original = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{text}");
        assert_eq!(value, original);
        assert_eq!(value.component(), original.component());
        let reparsed = literal(expected);
        assert_eq!(reparsed.unit(), CssResolutionUnit::Dppx);
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
        let value = CssResolutionValue::from_literal(value);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value.literal(), Some(&original));
    }
    let constructed = CssResolutionLiteral::try_new("192", CssResolutionUnit::Dpi).unwrap();
    let before = constructed.clone();
    assert_eq!(constructed.serialize_specified().unwrap(), "2dppx");
    assert_eq!(constructed, before);
}

#[test]
fn resolution_zero_tiny_and_huge_coefficients_remain_exact_authored_values() {
    for text in [
        "-0dpi",
        "-0e999999dpcm",
        "1e-999999dpi",
        "1e-9999999999999999999999999999999999999999dpcm",
    ] {
        let value = literal(text);
        let original = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 100, 5))
                .unwrap(),
            "0dppx"
        );
        assert_eq!(value, original);
    }
    let huge = literal("1e30dpi");
    let expected = "10416666666666666666666666666.666667dppx";
    assert_eq!(huge.serialize_specified().unwrap(), expected);
    assert_eq!(huge.numeric().representation(), "1e30");
    for text in [
        "1e1000000000dpi",
        "1e9999999999999999999999999999999999999999dppx",
    ] {
        let value = literal(text);
        let original = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(1, 100, 16))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, original);
    }
    // Adding emission does not weaken ordinary negative-range admission.
    for text in ["-1dpi", "-1e-999999dpcm"] {
        let component = parse_component_values(text).unwrap().items()[0].clone();
        assert_eq!(
            CssResolutionLiteral::try_from_component(component)
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::OutOfRange
        );
    }
}

#[test]
fn converted_resolution_bytes_and_shared_work_limits_fail_atomically() {
    let value = literal("1dpcm");
    let before = value.clone();
    let expected = "0.026458dppx";
    for (limits, kind) in [
        (
            Limits::new(0, usize::MAX, expected.len()),
            Kind::InputNodeLimit,
        ),
        (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
        (
            Limits::new(1, usize::MAX, expected.len() - 1),
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
        assert_eq!(value, before);
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(1, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
    let carry = literal("0.000048dpi");
    assert_eq!(
        carry
            .serialize_specified_with_limits(Limits::new(1, usize::MAX, 11))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        carry
            .serialize_specified_with_limits(Limits::new(1, usize::MAX, 12))
            .unwrap(),
        "0.000001dppx"
    );
    let zero = literal("-0dppx");
    assert_eq!(
        zero.serialize_specified_with_limits(Limits::new(1, 1, 5))
            .unwrap(),
        "0dppx"
    );
}

#[test]
fn escaped_resolution_units_and_crlf_non_bmp_provenance_survive_emission() {
    let source = "/*😀*/\r\n+0192D\\50 I";
    let value = literal(source);
    let component = value.component().clone();
    let origin = value.origin().clone();
    let CssValueOrigin::Parsed(parsed) = &origin else {
        panic!("parsed dimension")
    };
    assert_eq!(parsed.source().as_str(), source);
    assert_eq!(
        parsed.span().start().byte_offset().value(),
        "/*😀*/\r\n".len()
    );
    assert_eq!(parsed.span().start().line().value(), 1);
    assert_eq!(value.numeric().representation(), "+0192");
    assert_eq!(value.unit(), CssResolutionUnit::Dpi);
    assert_eq!(value.serialize_specified().unwrap(), "2dppx");
    assert_eq!(value.component(), &component);
    assert_eq!(value.origin(), &origin);
}

#[test]
fn resolution_calculations_share_existing_projection_without_clamping_or_mutating_storage() {
    for (text, expected) in [
        ("calc(192dpi + 1dppx)", "calc(3dppx)"),
        ("calc(-1dppx)", "calc(-1dppx)"),
        ("calc(1dppx * 1em / 1px)", "calc(1dppx * 1em / 1px)"),
    ] {
        let value = resolution(text);
        let original = value.clone();
        let calculation = value.calculation().unwrap();
        let components = calculation.components().clone();
        let ty = calculation.numeric_type();
        assert_eq!(value.serialize_specified().unwrap(), expected, "{text}");
        assert_eq!(value, original);
        assert_eq!(value.origin(), original.origin());
        assert_eq!(value.calculation().unwrap().components(), &components);
        assert_eq!(value.calculation().unwrap().numeric_type(), ty);
        assert_eq!(components.serialize().unwrap().as_css(), text);
        assert_eq!(
            resolution(expected).serialize_specified().unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(100, 100, expected.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, original);
    }
}

fn clip(text: &str) -> CssClip {
    let report = parse_style_attribute(&format!("clip:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Clip(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip property")
    };
    value.clip().clone()
}

#[test]
fn legacy_rect_scalar_components_keep_order_and_canonical_commas() {
    for (text, expected) in [
        (
            "rect(+01.2500PX auto -2EM 0)",
            "rect(1.25px, auto, -2em, 0)",
        ),
        (
            "rect(0.0000005px, auto, -0px, 1px)",
            "rect(0.000001px, auto, 0px, 1px)",
        ),
        (
            "rect(calc(1em + 2px) auto -1px 0)",
            "rect(calc(1em + 2px), auto, -1px, 0)",
        ),
    ] {
        let value = clip(text);
        let before = value.clone();
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(value, before);
        assert_eq!(clip(expected).serialize_specified().unwrap(), expected);
    }
    assert_eq!(CssClip::Auto.serialize_specified().unwrap(), "auto");
    // Modern clip-path rect() has its own whitespace grammar and provider.
    let report = parse_style_attribute("clip-path:rect(1px 2px 3px 4px)");
    assert!(report.is_clean());
    let CssKnownPropertyValueRef::ClipPath(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("clip-path property")
    };
    assert_eq!(
        value.value().serialize_specified().unwrap(),
        "rect(1px 2px 3px 4px)"
    );
}

#[test]
fn legacy_rect_children_spend_one_cumulative_budget_and_preserve_edge_origins() {
    let value = clip("rect(1px auto 2px 3px)");
    let before = value.clone();
    let expected = "rect(1px, auto, 2px, 3px)";
    for (limits, kind) in [
        (Limits::new(4, 100, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 100, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
    let CssClip::Rect(rect) = &value else {
        panic!("rectangle")
    };
    let CssClipEdge::Length(top) = rect.top() else {
        panic!("length edge")
    };
    let CssValueOrigin::Parsed(origin) = top.origin() else {
        panic!("retained edge origin")
    };
    assert_eq!(origin.source().as_str(), "clip:rect(1px auto 2px 3px)");
    assert_eq!(origin.span().start().byte_offset().value(), 10);
}
