//! Color 4 CRD 2026-09-08 §16 serializes every hue "as a canonicalized
//! <number> in degrees", and CSSOM (editor capture 2026-10-09, serialize a CSS
//! component value) writes a <number> "rounding the value if necessary to not
//! produce more than 6 decimals". Values 4 fixes 1rad = 180/π deg and
//! 1grad = 0.9deg, so 1rad = 57.2957795130823208…deg and serializes as
//! 57.29578. Degree and turn literals are exact multiples and keep their
//! exact decimal.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::parsed_color;
use surgeist_css::*;

fn cssom(value: &str) -> String {
    let source = format!("color: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax(), Default::default())
        .unwrap()
        .serialize_cssom(Default::default())
        .unwrap()
}

fn assert_hue_text(source: &str, expected: &str) {
    assert_eq!(
        parsed_color(source)
            .serialize_specified(Default::default())
            .unwrap(),
        expected,
        "{source}"
    );
    assert_eq!(cssom(source), format!("color: {expected};"), "{source}");
}

#[test]
fn radian_hues_serialize_as_degrees_with_cssom_number_precision() {
    // 180/π = 57.295779513082320876…; -1rad normalizes to 302.704220486917679…;
    // 3rad = 171.887338539246962629…; 0.001rad = 0.057295779513082…
    for (source, expected) in [
        ("lch(50 20 1rad)", "lch(50 20 57.29578)"),
        ("oklch(0.5 0.1 1rad)", "oklch(0.5 0.1 57.29578)"),
        ("lch(50 20 -1rad)", "lch(50 20 302.70422)"),
        ("oklch(0.5 0.1 3rad)", "oklch(0.5 0.1 171.887339)"),
        ("lch(50 20 0.001rad)", "lch(50 20 0.057296)"),
        ("hsl(1rad none 50%)", "hsl(57.29578 none 50%)"),
        ("hwb(1rad none 10%)", "hwb(57.29578 none 10%)"),
        ("hsl(3rad 50% 50% / none)", "hsl(171.887339 50% 50% / none)"),
        // 6.28318530717rad = 359.99999999945…deg rounds to the excluded 360,
        // the same direction as the normalized 0.
        ("lch(50 20 6.28318530717rad)", "lch(50 20 0)"),
    ] {
        assert_hue_text(source, expected);
    }
}

#[test]
fn gradian_hues_serialize_as_degrees_with_cssom_number_precision() {
    // 123.456789123grad × 0.9 = 111.1111102107deg; 0.0000001grad = 0.00000009deg.
    for (source, expected) in [
        ("lch(50 20 123.456789123grad)", "lch(50 20 111.11111)"),
        ("oklch(0.5 0.1 0.0000001grad)", "oklch(0.5 0.1 0)"),
        ("hsl(123.456789123grad none 50%)", "hsl(111.11111 none 50%)"),
        ("hwb(123.456789123grad 10% none)", "hwb(111.11111 10% none)"),
        ("lch(50 20 100grad)", "lch(50 20 90)"),
        ("lch(50 20 1grad)", "lch(50 20 0.9)"),
        ("lch(50 20 399.9999999999grad)", "lch(50 20 0)"),
    ] {
        assert_hue_text(source, expected);
    }
}

#[test]
fn exact_degree_and_turn_hues_keep_their_exact_decimal() {
    for (source, expected) in [
        ("lch(50 20 57.2957795130823)", "lch(50 20 57.2957795130823)"),
        (
            "oklch(0.5 0.1 57.2957795130823deg)",
            "oklch(0.5 0.1 57.2957795130823)",
        ),
        ("lch(50 20 0.12345678912turn)", "lch(50 20 44.4444440832)"),
        ("hsl(0.5turn none 50%)", "hsl(180 none 50%)"),
    ] {
        assert_hue_text(source, expected);
    }
}

#[test]
fn rounded_hue_budgets_are_exact_at_the_limit_and_one_short() {
    // Characterizes the cumulative budget of the six-decimal hue: four input
    // nodes, the projection work of the exact conversion and its rounding,
    // and exactly the emitted UTF-8 bytes.
    type Limits = CssSerializationLimits;
    type Kind = SerializationFailure;
    for (source, expected, input, projection) in [
        ("hsl(1rad none 50%)", "hsl(57.29578 none 50%)", 4, 19),
        ("hsl(100grad none 50%)", "hsl(90 none 50%)", 4, 11),
    ] {
        let value = parsed_color(source);
        let bytes = expected.len();
        let serialize = |input, projection, bytes| {
            value.serialize_specified(Limits::new(input, projection, bytes))
        };
        assert_eq!(
            serialize(input, projection, bytes).unwrap(),
            expected,
            "{source}"
        );
        for (limits, kind) in [
            ((input - 1, projection, bytes), Kind::InputNodeLimit),
            ((input, projection - 1, bytes), Kind::ProjectionNodeLimit),
            ((input, projection, bytes - 1), Kind::ByteLimit),
        ] {
            assert_eq!(
                serialization_failure(&serialize(limits.0, limits.1, limits.2).unwrap_err()),
                kind,
                "{source}: {limits:?}"
            );
        }
    }
}
