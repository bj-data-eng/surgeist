#![forbid(unsafe_code)]

//! Independent checked authored-value cases for the selected Overflow 3 controls.

use surgeist_css::*;

fn length(number: &str, unit: &str) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(
        CssComponentValue::try_dimension(number, unit).unwrap(),
    )
    .unwrap()
}

fn parsed_clip_margin(value: &str) -> CssOverflowClipMargin {
    let report = parse_style_attribute(&format!("overflow-clip-margin: {value}"));
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::OverflowClipMargin(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed clip margin")
    };
    value.value().clone()
}

#[test]
fn checked_clip_margin_keeps_authored_components_and_exact_effective_defaults() {
    assert!(CssOverflowClipMargin::try_new(None, None).is_none());
    for edge in [
        CssBoxEdgeKeyword::MarginBox,
        CssBoxEdgeKeyword::FillBox,
        CssBoxEdgeKeyword::StrokeBox,
        CssBoxEdgeKeyword::ViewBox,
    ] {
        assert!(CssOverflowClipMargin::try_new(Some(edge), Some(length("2", "px"))).is_none());
    }
    for edge in [
        CssBoxEdgeKeyword::ContentBox,
        CssBoxEdgeKeyword::PaddingBox,
        CssBoxEdgeKeyword::BorderBox,
    ] {
        assert!(CssOverflowClipMargin::try_new(Some(edge), None).is_some());
    }

    let bare_box =
        CssOverflowClipMargin::try_new(Some(CssBoxEdgeKeyword::ContentBox), None).unwrap();
    assert_eq!(
        bare_box.authored_box_edge(),
        Some(CssBoxEdgeKeyword::ContentBox)
    );
    assert!(bare_box.authored_offset().is_none());
    assert_eq!(bare_box.offset().serialize_specified().unwrap(), "0px");
    assert_eq!(bare_box.serialize_specified().unwrap(), "content-box");

    let authored_zero = parsed_clip_margin("0");
    assert_eq!(authored_zero.serialize_specified().unwrap(), "0");
    assert_eq!(authored_zero.box_edge(), CssBoxEdgeKeyword::PaddingBox);
    assert_eq!(authored_zero.offset().serialize_specified().unwrap(), "0");
    let initial = CssOverflowClipMargin::initial();
    assert_eq!(initial.authored_box_edge(), None);
    assert_eq!(
        initial
            .authored_offset()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "0px"
    );
    assert_eq!(initial.offset().serialize_specified().unwrap(), "0px");
    assert_ne!(initial, authored_zero);
}

#[test]
fn clip_margin_equality_ignores_origins_but_keeps_spelling_and_omission() {
    let parsed = parsed_clip_margin("2px border-box");
    let programmatic =
        CssOverflowClipMargin::try_new(Some(CssBoxEdgeKeyword::BorderBox), Some(length("2", "px")))
            .unwrap();
    assert_eq!(parsed, programmatic);
    assert_eq!(parsed.serialize_specified().unwrap(), "border-box 2px");
    assert_ne!(parsed, parsed_clip_margin("2.0px border-box"));
    assert_ne!(
        parsed,
        CssOverflowClipMargin::try_new(None, Some(length("2", "px"))).unwrap()
    );
    assert_ne!(
        parsed_clip_margin("padding-box"),
        parsed_clip_margin("padding-box 0px")
    );
    assert_ne!(parsed_clip_margin("0px"), parsed_clip_margin("padding-box"));
}

#[test]
fn keyword_controls_serialize_exactly_and_reject_one_below_limits() {
    for (value, text) in [
        (CssScrollBehavior::Auto, "auto"),
        (CssScrollBehavior::Smooth, "smooth"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len()
                ))
                .unwrap(),
            text
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len() - 1
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }
    for (value, text) in [
        (CssScrollbarGutter::Auto, "auto"),
        (CssScrollbarGutter::Stable, "stable"),
        (CssScrollbarGutter::StableBothEdges, "stable both-edges"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len()
                ))
                .unwrap(),
            text
        );
    }
    for (value, text) in [
        (CssTextOverflow::Clip, "clip"),
        (CssTextOverflow::Ellipsis, "ellipsis"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), text);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    text.len()
                ))
                .unwrap(),
            text
        );
    }
    for limits in [
        CssSpecifiedValueSerializationLimits::new(0, 1, 99),
        CssSpecifiedValueSerializationLimits::new(1, 0, 99),
    ] {
        assert!(
            CssScrollBehavior::Auto
                .serialize_specified_with_limits(limits)
                .is_err()
        );
        assert!(
            CssScrollbarGutter::Stable
                .serialize_specified_with_limits(limits)
                .is_err()
        );
        assert!(
            CssTextOverflow::Clip
                .serialize_specified_with_limits(limits)
                .is_err()
        );
    }
}

#[test]
fn parsed_keyword_controls_expose_exact_typed_values_and_authored_spelling() {
    for (name, input, expected) in [
        ("scroll-behavior", "AUTO", CssScrollBehavior::Auto),
        ("scroll-behavior", "smooth", CssScrollBehavior::Smooth),
    ] {
        let report = parse_style_attribute(&format!("{name}: {input}"));
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::ScrollBehavior(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed scroll behavior")
        };
        assert_eq!(*value.value(), expected);
        assert_eq!(value.as_css(), input);
    }
    for (input, expected) in [
        ("auto", CssScrollbarGutter::Auto),
        ("stable", CssScrollbarGutter::Stable),
        ("both-edges STABLE", CssScrollbarGutter::StableBothEdges),
    ] {
        let report = parse_style_attribute(&format!("scrollbar-gutter: {input}"));
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::ScrollbarGutter(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed scrollbar gutter")
        };
        assert_eq!(*value.value(), expected);
        assert_eq!(value.as_css(), input);
    }
    for (input, expected) in [
        ("CLIP", CssTextOverflow::Clip),
        ("ellipsis", CssTextOverflow::Ellipsis),
    ] {
        let report = parse_style_attribute(&format!("text-overflow: {input}"));
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::TextOverflow(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed text overflow")
        };
        assert_eq!(*value.value(), expected);
        assert_eq!(value.value(), &expected);
        assert_eq!(value.as_css(), input);
    }
}

#[test]
fn clip_margin_serialization_uses_one_budget_for_outer_box_offset_and_math() {
    let value = parsed_clip_margin("2px content-box");
    assert_eq!(value.serialize_specified().unwrap(), "content-box 2px");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 15))
            .unwrap(),
        "content-box 2px"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 15),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 15),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 14),
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
    }

    let math = parsed_clip_margin("border-box calc(1px + 2em)");
    let serialized = math.serialize_specified().unwrap();
    assert_eq!(serialized, "border-box calc(2em + 1px)");
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            6,
            7,
            serialized.len()
        ))
        .unwrap(),
        serialized
    );
    assert_eq!(math.authored_box_edge(), Some(CssBoxEdgeKeyword::BorderBox));
    assert!(math.authored_offset().unwrap().calculation().is_some());
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            2, 256, 256
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            256, 2, 256
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(
        math.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
            256,
            256,
            serialized.len() - 1
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
}
