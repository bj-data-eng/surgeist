#![forbid(unsafe_code)]

//! Fonts 4 section 13.2: equal retained endpoints use the shorter form.
//! Rounding two different retained numbers to the same text is not equality.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssFontFaceDescriptorKind as Kind,
    CssFontFaceDescriptorValue as Value, CssFontFaceStyle,
    CssSpecifiedValueSerializationErrorKind as ErrorKind,
    CssSpecifiedValueSerializationLimits as Limits, parse_font_face_descriptor_value,
};

fn ordinary(source: &str, kind: Kind) -> Value {
    let report = parse_font_face_descriptor_value(source, kind);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let Some(Authored::Ordinary(value)) = report.syntax() else {
        panic!("expected an ordinary descriptor: {source}")
    };
    value.clone()
}

// Both retained endpoints consume visits, including the endpoint suppressed in
// output. Only the returned shorter text consumes the final output byte budget.
macro_rules! assert_shortened {
    ($value:expr, $expected:expr, $nodes:expr) => {{
        let value = $value;
        let before = value.clone();
        let expected = $expected;
        let nodes = $nodes;
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(nodes, nodes, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, usize::MAX),
                ErrorKind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, usize::MAX),
                ErrorKind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                ErrorKind::ByteLimit,
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
    }};
}

#[test]
fn equal_weight_endpoints_emit_once_and_still_charge_both_visits() {
    for source in ["400 400", "400 4e2", "bold bold"] {
        let Value::FontWeight(value) = ordinary(source, Kind::FontWeight) else {
            panic!("weight")
        };
        let expected = if source == "bold bold" { "bold" } else { "400" };
        assert_shortened!(value, expected, 3);
    }
}

#[test]
fn equal_width_endpoints_emit_once_and_still_charge_both_visits() {
    for source in ["75% 75%", "75% 75.0%", "condensed condensed"] {
        let Value::FontWidth(value) = ordinary(source, Kind::FontWidth) else {
            panic!("width")
        };
        let expected = if source == "condensed condensed" {
            "condensed"
        } else {
            "75%"
        };
        assert_shortened!(value, expected, 3);
    }
}

#[test]
fn equal_oblique_endpoints_emit_once_and_still_charge_both_visits() {
    for source in ["oblique 10deg 10deg", "oblique 10deg 1e1deg"] {
        let Value::FontStyle(value) = ordinary(source, Kind::FontStyle) else {
            panic!("style")
        };
        assert_shortened!(value, "oblique 10deg", 3);
    }
}

#[test]
fn standalone_oblique_range_uses_the_same_shortening_contract() {
    let Value::FontStyle(CssFontFaceStyle::Oblique { range: Some(range) }) =
        ordinary("oblique 10deg 10deg", Kind::FontStyle)
    else {
        panic!("oblique range")
    };
    assert!(range.end().is_some());
    assert_shortened!(range, "10deg", 2);
}

#[test]
fn different_retained_endpoints_are_not_collapsed_after_rounding() {
    for (source, kind, expected) in [
        ("400.0000001 400.0000002", Kind::FontWeight, "400 400"),
        ("75.0000001% 75.0000002%", Kind::FontWidth, "75% 75%"),
        (
            "oblique 10.0000001deg 10.0000002deg",
            Kind::FontStyle,
            "oblique 10deg 10deg",
        ),
    ] {
        let value = ordinary(source, kind);
        let before = value.clone();
        let output = match &value {
            Value::FontWeight(value) => value.serialize_specified().unwrap(),
            Value::FontWidth(value) => value.serialize_specified().unwrap(),
            Value::FontStyle(value) => value.serialize_specified().unwrap(),
            _ => panic!("unexpected descriptor"),
        };
        assert_eq!(output, expected, "{source}");
        assert_eq!(value, before);
    }
}
