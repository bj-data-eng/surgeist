//! Independent checked authored-value cases for the selected Overflow 3 controls.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValue;
use surgeist_css::numeric::{CssLength, CssNumeric};
use surgeist_css::property::CssKnownPropertyValueRef;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::box_model::{
    CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter,
};
use surgeist_css::values::common::CssBoxEdgeKeyword;
use surgeist_css::values::text::CssTextOverflow;

fn length(number: &str, unit: &str) -> CssNumeric<CssLength> {
    CssNumeric::<CssLength>::try_from_component(
        CssComponentValue::try_dimension(number, unit).unwrap(),
    )
    .unwrap()
}

fn parsed_clip_margin(value: &str) -> CssOverflowClipMargin {
    let report =
        CssParser::default().parse_block_contents(format!("overflow-clip-margin: {value}"));
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
    assert_eq!(
        bare_box
            .offset()
            .serialize_specified(Default::default())
            .unwrap(),
        "0px"
    );
    assert_eq!(
        bare_box.serialize_specified(Default::default()).unwrap(),
        "content-box"
    );

    let authored_zero = parsed_clip_margin("0");
    assert_eq!(
        authored_zero
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
    assert_eq!(authored_zero.box_edge(), CssBoxEdgeKeyword::PaddingBox);
    assert_eq!(
        authored_zero
            .offset()
            .serialize_specified(Default::default())
            .unwrap(),
        "0"
    );
    let initial = CssOverflowClipMargin::initial();
    assert_eq!(initial.authored_box_edge(), None);
    assert_eq!(
        initial
            .authored_offset()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "0px"
    );
    assert_eq!(
        initial
            .offset()
            .serialize_specified(Default::default())
            .unwrap(),
        "0px"
    );
    assert_eq!(initial, authored_zero);
}

#[test]
fn clip_margin_equality_ignores_origins_and_spelling_but_keeps_omission() {
    let parsed = parsed_clip_margin("2px border-box");
    let programmatic =
        CssOverflowClipMargin::try_new(Some(CssBoxEdgeKeyword::BorderBox), Some(length("2", "px")))
            .unwrap();
    assert_eq!(parsed, programmatic);
    assert_eq!(
        parsed.serialize_specified(Default::default()).unwrap(),
        "border-box 2px"
    );
    assert_eq!(parsed, parsed_clip_margin("2.0px border-box"));
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
fn checked_clip_margin_admits_signed_offsets_and_matches_parsed_values() {
    // The checked constructor takes the same signed `<length>` owner the parser
    // does: negative literals (insets), zero and positive offsets are all valid.
    for (edge, text, number, unit, expected) in [
        (None, "-1px", "-1", "px", "-1px"),
        (None, "-0.5em", "-0.5", "em", "-0.5em"),
        (None, "1px", "1", "px", "1px"),
        (
            Some(CssBoxEdgeKeyword::ContentBox),
            "-2px",
            "-2",
            "px",
            "content-box -2px",
        ),
        (
            Some(CssBoxEdgeKeyword::BorderBox),
            "-3px",
            "-3",
            "px",
            "border-box -3px",
        ),
        (
            Some(CssBoxEdgeKeyword::PaddingBox),
            "-1e-999px",
            "-1e-999",
            "px",
            "padding-box 0px",
        ),
    ] {
        let authored = match edge {
            Some(edge) => format!("{} {text}", edge.as_str()),
            None => text.to_owned(),
        };
        let checked = CssOverflowClipMargin::try_new(edge, Some(length(number, unit))).unwrap();
        assert_eq!(checked, parsed_clip_margin(&authored), "{authored}");
        assert_eq!(
            checked.serialize_specified(Default::default()).unwrap(),
            expected,
            "{authored}"
        );
    }
    // The boxes are still limited to the visual boxes for a signed offset.
    for edge in [
        CssBoxEdgeKeyword::MarginBox,
        CssBoxEdgeKeyword::FillBox,
        CssBoxEdgeKeyword::StrokeBox,
        CssBoxEdgeKeyword::ViewBox,
    ] {
        assert!(CssOverflowClipMargin::try_new(Some(edge), Some(length("-2", "px"))).is_none());
    }
    // A signed offset alone is a complete value, and zero is not omission.
    assert!(CssOverflowClipMargin::try_new(None, Some(length("-2", "px"))).is_some());
    assert_ne!(
        CssOverflowClipMargin::try_new(None, Some(length("-2", "px"))),
        CssOverflowClipMargin::try_new(None, Some(length("2", "px")))
    );
    assert_ne!(
        CssOverflowClipMargin::try_new(Some(CssBoxEdgeKeyword::ContentBox), None),
        CssOverflowClipMargin::try_new(
            Some(CssBoxEdgeKeyword::ContentBox),
            Some(length("0", "px"))
        )
    );
}

#[test]
fn keyword_controls_serialize_exactly_and_reject_one_below_limits() {
    for (value, text) in [
        (CssScrollBehavior::Auto, "auto"),
        (CssScrollBehavior::Smooth, "smooth"),
    ] {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len()))
                .unwrap(),
            text
        );
        assert!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len() - 1))
                .unwrap_err()
                .is_byte_limit()
        );
    }
    for (value, text) in [
        (CssScrollbarGutter::Auto, "auto"),
        (CssScrollbarGutter::Stable, "stable"),
        (CssScrollbarGutter::StableBothEdges, "stable both-edges"),
    ] {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len()))
                .unwrap(),
            text
        );
    }
    for (value, text) in [
        (CssTextOverflow::Clip, "clip"),
        (CssTextOverflow::Ellipsis, "ellipsis"),
    ] {
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, text.len()))
                .unwrap(),
            text
        );
    }
    // One keyword is one input node and one projection node.
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, 99),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, 99),
            SerializationFailure::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(
                &CssScrollBehavior::Auto
                    .serialize_specified(limits)
                    .unwrap_err()
            ),
            kind
        );
        assert_eq!(
            serialization_failure(
                &CssScrollbarGutter::Stable
                    .serialize_specified(limits)
                    .unwrap_err()
            ),
            kind
        );
        assert_eq!(
            serialization_failure(
                &CssTextOverflow::Clip
                    .serialize_specified(limits)
                    .unwrap_err()
            ),
            kind
        );
    }
}

#[test]
fn parsed_keyword_controls_expose_exact_typed_values_and_authored_spelling() {
    for (name, input, expected) in [
        ("scroll-behavior", "AUTO", CssScrollBehavior::Auto),
        ("scroll-behavior", "smooth", CssScrollBehavior::Smooth),
    ] {
        let report = CssParser::default().parse_block_contents(format!("{name}: {input}"));
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
        let report =
            CssParser::default().parse_block_contents(format!("scrollbar-gutter: {input}"));
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
        let report = CssParser::default().parse_block_contents(format!("text-overflow: {input}"));
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
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "content-box 2px"
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 3, 15))
            .unwrap(),
        "content-box 2px"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 15),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 15),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 14),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }

    let math = parsed_clip_margin("border-box calc(1px + 2em)");
    let serialized = math.serialize_specified(Default::default()).unwrap();
    assert_eq!(serialized, "border-box calc(2em + 1px)");
    assert_eq!(
        math.serialize_specified(CssSerializationLimits::new(6, 7, serialized.len()))
            .unwrap(),
        serialized
    );
    assert_eq!(math.authored_box_edge(), Some(CssBoxEdgeKeyword::BorderBox));
    assert!(math.authored_offset().unwrap().calculation().is_some());
    assert!(
        math.serialize_specified(CssSerializationLimits::new(2, 256, 256))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        math.serialize_specified(CssSerializationLimits::new(256, 2, 256))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        math.serialize_specified(CssSerializationLimits::new(256, 256, serialized.len() - 1))
            .unwrap_err()
            .is_byte_limit()
    );
}

fn specified(value: &str) -> String {
    parsed_clip_margin(value)
        .serialize_specified(Default::default())
        .unwrap()
}

#[test]
fn signed_literal_offsets_serialize_canonically_in_box_first_order() {
    // Values 4 canonical serialization: the sign is kept, trailing zeros and
    // the plus sign are dropped, and a result that rounds to zero is `0`.
    for (authored, expected) in [
        ("-1px", "-1px"),
        ("+1px", "1px"),
        ("-2.50PX content-box", "content-box -2.5px"),
        ("-0.5em border-box", "border-box -0.5em"),
        ("padding-box -3px", "padding-box -3px"),
        ("-0px", "0px"),
        ("-0", "0"),
        ("padding-box -0.000e999px", "padding-box 0px"),
        ("-1e-999px", "0px"),
        ("-0.0000004px", "0px"),
        ("-0.0000005px", "-0.000001px"),
        ("-1.0000004px", "-1px"),
    ] {
        assert_eq!(specified(authored), expected, "{authored}");
    }
}

#[test]
fn signed_literal_offsets_keep_exact_sign_and_magnitude_until_serialization() {
    let inset = parsed_clip_margin("-1px");
    assert_eq!(inset.authored_box_edge(), None);
    assert_eq!(inset.box_edge(), CssBoxEdgeKeyword::PaddingBox);
    assert!(inset.authored_offset().unwrap().calculation().is_none());
    assert_eq!(
        inset
            .offset()
            .serialize_specified(Default::default())
            .unwrap(),
        "-1px"
    );
    assert_ne!(inset, parsed_clip_margin("1px"));
    assert_ne!(inset, parsed_clip_margin("padding-box -1px"));
    assert_eq!(inset, parsed_clip_margin("-1.0px"));
    assert_eq!(inset, parsed_clip_margin("-1PX"));
    // A tiny negative is exactly nonzero even though it serializes as zero.
    assert_ne!(parsed_clip_margin("-1e-999px"), parsed_clip_margin("0px"));
    assert_ne!(
        parsed_clip_margin("-1e-999px"),
        parsed_clip_margin("1e-999px")
    );
    // Negative zero is zero.
    assert_eq!(parsed_clip_margin("-0px"), parsed_clip_margin("0px"));
    assert_eq!(parsed_clip_margin("-0"), parsed_clip_margin("0"));
    assert_eq!(parsed_clip_margin("-0px"), CssOverflowClipMargin::initial());
}

#[test]
fn signed_symbolic_offsets_stay_unresolved_math_without_range_clamping() {
    // Values 4 sum canonicalization (as for `outline-offset: calc(1px - 2em)`
    // -> `calc(-2em + 1px)`) with no clamp of a negative result to zero.
    for (authored, expected) in [
        ("calc(-1px)", "calc(-1px)"),
        ("calc(1px - 2px)", "calc(-1px)"),
        ("border-box calc(1px - 2em)", "border-box calc(-2em + 1px)"),
        ("calc(2em - 3px)", "calc(2em - 3px)"),
        (
            "calc(-2em - 3px) content-box",
            "content-box calc(-2em - 3px)",
        ),
        ("calc(1px + 2em)", "calc(2em + 1px)"),
    ] {
        assert_eq!(specified(authored), expected, "{authored}");
        let value = parsed_clip_margin(authored);
        assert!(
            value.authored_offset().unwrap().calculation().is_some(),
            "{authored}: math stays a calculation, not a clamped literal"
        );
        assert!(
            value.authored_offset().unwrap().literal().is_none(),
            "{authored}"
        );
    }
    assert_ne!(
        parsed_clip_margin("calc(1px - 2px)"),
        parsed_clip_margin("0px")
    );
    assert_ne!(
        parsed_clip_margin("calc(1px - 2px)"),
        parsed_clip_margin("-1px")
    );
}

#[test]
fn signed_offsets_serialize_within_exact_byte_and_node_budgets() {
    let value = parsed_clip_margin("-2px content-box");
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 3, 16))
            .unwrap(),
        "content-box -2px"
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(2, 3, 16),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 16),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 3, 15),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            kind
        );
    }

    let math = parsed_clip_margin("border-box calc(1px - 2em)");
    let serialized = math.serialize_specified(Default::default()).unwrap();
    assert_eq!(serialized, "border-box calc(-2em + 1px)");
    assert_eq!(
        math.serialize_specified(CssSerializationLimits::new(256, 256, serialized.len()))
            .unwrap(),
        serialized
    );
    assert!(
        math.serialize_specified(CssSerializationLimits::new(256, 256, serialized.len() - 1))
            .unwrap_err()
            .is_byte_limit()
    );
    assert!(
        math.serialize_specified(CssSerializationLimits::new(2, 256, 256))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        math.serialize_specified(CssSerializationLimits::new(256, 2, 256))
            .unwrap_err()
            .is_projection_node_limit()
    );
}

#[test]
fn signed_offsets_in_a_block_leave_neighbors_and_recovery_unchanged() {
    let report = CssParser::default().parse_block_contents(
        "color:red;overflow-clip-margin:-1px content-box;overflow-clip-margin:-1%;color:blue",
    );
    // The accepted inset is retained, the percentage declaration is dropped
    // atomically, and the neighbors are untouched.
    assert_eq!(report.syntax().len(), 3);
    assert_eq!(report.diagnostics().len(), 1);
    let CssKnownPropertyValueRef::OverflowClipMargin(value) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed clip margin")
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "content-box -1px"
    );
}
