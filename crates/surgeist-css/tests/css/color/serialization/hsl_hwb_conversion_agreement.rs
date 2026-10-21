//! Color 4 CRD 2026-09-08 §§7.1, 8.1 and 16.2.2: complete hsl() and hwb()
//! serialize as the corresponding sRGB channels in legacy rgb() form. Specified
//! serialization and used-value serialization share surgeist-color's single
//! conversion, so they agree byte for byte, including at decimal rounding ties
//! where an independent exact or CSS3 m1/m2 computation could differ by an ulp.

use crate::css::support::checked_color as color;
use surgeist_css::{
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorOrdinaryLimits, CssColorOrdinaryOutcome, CssColorPropertyFacts, CssColorSchemeFact,
    CssDeviceColorFact, parse_used_color,
};

fn used_css(input: &str) -> String {
    let facts = CssColorFacts::new(
        CssColorFactIdentity::new(13, 1),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    );
    let result = parse_used_color(
        CssColorInput::Utf8(input.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        CssColorOrdinaryLimits {
            input_units: 1024,
            algorithm_records: 16_384,
            arithmetic_records: 4096,
            evaluation_records: 4096,
            actions: 40_000_000,
        },
    )
    .unwrap_or_else(|error| panic!("{input}: {error:?}"));
    let CssColorOrdinaryOutcome::Used(value) = result.outcome() else {
        panic!("used color for {input}")
    };
    surgeist_color::serialize(
        value,
        surgeist_color::SerializationRequest::Css,
        surgeist_color::SerializationLimits {
            actions: 4_000_000,
            digits: 4096,
            output_bytes: 4096,
        },
    )
    .unwrap()
}

#[test]
fn hsl_and_hwb_vectors_serialize_their_srgb_channels() {
    // Each channel is 255 × the Color 4 §7.1/§8.1 value, written with at most
    // six decimals; all of these are exact in binary64.
    for (source, expected) in [
        ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
        ("hsl(120 100% 50%)", "rgb(0, 255, 0)"),
        ("hsl(240 100% 50%)", "rgb(0, 0, 255)"),
        ("hsl(30 100% 50%)", "rgb(255, 127.5, 0)"),
        ("hsl(120 100% 25%)", "rgb(0, 127.5, 0)"),
        ("hsl(0 0% 50%)", "rgb(127.5, 127.5, 127.5)"),
        ("hsl(120deg 100% 50% / 0.5)", "rgba(0, 255, 0, 0.5)"),
        ("hsl(0.5turn 100% 50%)", "rgb(0, 255, 255)"),
        ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
        ("hwb(120 0% 50%)", "rgb(0, 127.5, 0)"),
        ("hwb(0 20% 80%)", "rgb(51, 51, 51)"),
        ("hwb(0 60% 60%)", "rgb(127.5, 127.5, 127.5)"),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
        assert_eq!(used_css(source), expected, "{source}");
    }
}

#[test]
fn specified_and_used_hsl_hwb_agree_at_rounding_ties() {
    // 255 × 6.27451% = 16.0000005 and 255 × 17.51634% / 2 = 22.3333335 are
    // exact decimal ties; 93.72549% + 6.27451% = 100% puts HWB at its gray
    // boundary. The binary64 operands decide each tie once for both phases.
    for source in [
        "hsl(0 0% 6.27451%)",
        "hsl(calc(0) 0% 6.27451%)",
        "hsl(0 50% 17.51634%)",
        "hsl(0 50% calc(17.51634%))",
        "hwb(0 6.27451% 0%)",
        "hwb(0 6.27451% 93.72549%)",
        "hwb(calc(0deg) 6.27451% 93.72549%)",
        "hsl(1rad 37% 61%)",
        "hsl(100grad 12.5% 33.3%)",
        "hwb(3rad 12.345678% 7.654321%)",
        "hsl(1e30 100% 50%)",
        // A gradian overflowing binary64 before its exact 0.9 scaling is
        // 1.71e308deg, as used resolution converts it, not the 360deg-multiple
        // endpoint of an infinite product.
        "hsl(1.9e308grad 100% 50%)",
        "hsl(-1.9e308grad 100% 50%)",
        "hwb(1.9e308grad 0% 0%)",
        "hwb(1.9e308grad 10% 20% / 0.5)",
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            used_css(source),
            "{source}"
        );
    }
}

#[test]
fn binary64_ties_have_pinned_specified_and_used_text() {
    // With zero saturation every HSL channel is L, and at basis 0 an HWB
    // channel is W: 255 × binary64(6.27451 / 100) lies just below the exact
    // decimal tie 16.0000005, so six-decimal rounding gives 16.
    for (source, expected) in [
        ("hsl(0 0% 6.27451%)", "rgb(16, 16, 16)"),
        ("hsl(120 0% 6.27451% / 50%)", "rgba(16, 16, 16, 0.5)"),
        ("hwb(0 6.27451% 0%)", "rgb(255, 16, 16)"),
        ("hwb(120 6.27451% 0%)", "rgb(16, 255, 16)"),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
        assert_eq!(used_css(source), expected, "{source}");
    }
}

#[test]
fn extreme_lightness_and_whiteness_keep_exact_vertex_channels() {
    // Color 4 §7.1 with S = 100% and L > 50%: f(n) = L - (1 - L) t is exactly
    // 1 at t = -1 and 2L - 1 >= 1 at t = 1, so any huge lightness is white.
    // With S = L = 1e300% the channels are about ∓1e596, the signed closest
    // finite values, which clip to 0 and 255. Color 4 §8.1 at basis 1 is
    // 1 - B and at basis 0 is W, whatever the magnitude of W. A literal beyond
    // binary64 reads as the closest finite value (Values 4 §10.12).
    for (source, expected) in [
        ("hsl(0 100% 1e18%)", "rgb(255, 255, 255)"),
        ("hsl(360 100% 1e400%)", "rgb(255, 255, 255)"),
        ("hsl(0 1e300% 1e300%)", "rgb(0, 255, 255)"),
        ("hsl(0 1e400% 50%)", "rgb(255, 0, 0)"),
        ("hwb(0 -1e18% 0%)", "rgb(255, 0, 0)"),
        ("hwb(120 -1e400% 0%)", "rgb(0, 255, 0)"),
        ("hwb(0 0% -1e18%)", "rgb(255, 0, 0)"),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
        assert_eq!(used_css(source), expected, "{source}");
    }
}

#[test]
fn overflowing_number_hues_take_the_nearest_supported_multiple_of_360deg() {
    // Color 4 §4.3 interprets a <number> hue as degrees. Values 4 §10.12
    // converts an angle beyond the supported range to the nearest supported
    // multiple of 360deg, which normalizes to 0deg; a number hue overflowing
    // binary64 is therefore the same direction as an overflowing <angle>.
    for (source, expected) in [
        ("hsl(1e400 100% 50%)", "rgb(255, 0, 0)"),
        ("hsl(-1e400 100% 50%)", "rgb(255, 0, 0)"),
        ("hsl(1e400deg 100% 50%)", "rgb(255, 0, 0)"),
        ("hwb(1e400 0% 0%)", "rgb(255, 0, 0)"),
        ("hwb(-1e400 0% 50% / 0.5)", "rgba(127.5, 0, 0, 0.5)"),
        ("lch(50 20 1e400)", "lch(50 20 0)"),
        ("lch(50 20 1e400deg)", "lch(50 20 0)"),
        ("oklch(0.5 0.1 -1e400)", "oklch(0.5 0.1 0)"),
        ("oklch(0.5 0.1 1e400turn)", "oklch(0.5 0.1 0)"),
        ("hsl(1e400 none 50%)", "hsl(0 none 50%)"),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
        assert_eq!(used_css(source), expected, "{source}");
    }
}
