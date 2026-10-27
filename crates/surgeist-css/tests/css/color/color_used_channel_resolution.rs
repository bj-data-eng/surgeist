//! Absolute and relative channel resolution through the production used-color
//! front door. Color 4 CRD 2026-09-08 declares the channel references (rgb
//! "100% = 255", lab "100% = 100" lightness and "100% = 125" a/b, oklch
//! "100% = 1" lightness and "100% = 0.4" chroma, color() "100% = 1"), clamps
//! absolute rgb() to [0, 255] and lightness to [0, 100%], resolves complete
//! hsl()/hwb() to sRGB and keeps missing components. Color 5 WD 2026-09-08
//! §4.1: relative channel values "are *not clamped*", and an omitted relative
//! alpha is the origin's alpha.

use surgeist_color::{Color, ColorSpace, PredefinedColorSpace, SerializationFamily};
use surgeist_css::color::{
    CssColorFactIdentity, CssColorFacts, CssColorGrammarRequest, CssColorInput,
    CssColorOrdinaryLimits, CssColorOrdinaryOutcome, CssColorPropertyFacts, CssColorSchemeFact,
    CssDeviceColorFact, parse_used_color,
};

const SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::Srgb);

fn facts() -> CssColorFacts<'static> {
    CssColorFacts::new(
        CssColorFactIdentity::new(11, 5),
        CssColorPropertyFacts::Unavailable,
        CssColorSchemeFact::Unavailable,
        &[],
        &[],
        &[],
        CssDeviceColorFact::Unknown,
    )
}

fn limits() -> CssColorOrdinaryLimits {
    CssColorOrdinaryLimits {
        input_units: 1024,
        algorithm_records: 16_384,
        arithmetic_records: 4096,
        evaluation_records: 4096,
        actions: 40_000_000,
    }
}

fn used(input: &str) -> Option<Color> {
    let facts = facts();
    let result = parse_used_color(
        CssColorInput::Utf8(input.as_bytes()),
        CssColorGrammarRequest::WholeColor,
        &facts,
        facts.identity(),
        limits(),
    );
    match result.map(|result| result.outcome()) {
        Ok(CssColorOrdinaryOutcome::Used(value)) => Some(value),
        _ => None,
    }
}

fn color(input: &str) -> Color {
    used(input).unwrap_or_else(|| panic!("used color for {input}"))
}

#[test]
fn absolute_channels_use_declared_references_and_parse_time_clamps() {
    let rgb = color("rgb(100% -1 128 / 50%)");
    assert_eq!(rgb.space(), SRGB);
    assert_eq!(rgb.channels(), [Some(1.0), Some(0.0), Some(128.0 / 255.0)]);
    assert_eq!(rgb.alpha(), Some(0.5));
    assert_eq!(rgb.srgb_bytes(), None);

    let lab = color("lab(120% 100% -100%)");
    assert_eq!(lab.space(), ColorSpace::Lab);
    assert_eq!(lab.channels(), [Some(100.0), Some(125.0), Some(-125.0)]);

    // 450deg normalizes to 90deg.
    let oklch = color("oklch(50% 100% 450deg)");
    assert_eq!(oklch.space(), ColorSpace::Oklch);
    assert_eq!(oklch.channels(), [Some(0.5), Some(0.4), Some(90.0)]);

    let wide = color("color(display-p3 1.5 -50% none / none)");
    assert_eq!(
        wide.space(),
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3)
    );
    assert_eq!(wide.channels(), [Some(1.5), Some(-0.5), None]);
    assert_eq!(wide.alpha(), None);

    // Color 4 §5.1: Infinity, -Infinity and NaN "clamp at 255, 0 and 0".
    let exceptional = color("rgb(calc(infinity) calc(NaN) calc(-infinity))");
    assert_eq!(exceptional.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
}

#[test]
fn legacy_domain_mixtures_are_invalid_colors() {
    // Legacy rgb() channels are all numbers or all percentages; legacy hsl()
    // saturation and lightness are percentages.
    for input in ["rgb(1, 2%, 3%)", "hsl(0deg, 100, 50%)"] {
        assert!(used(input).is_none(), "{input}");
    }
}

#[test]
fn complete_hsl_resolves_to_srgb_and_missing_channels_keep_the_natural_space() {
    let red = color("hsl(360deg 100% 50%)");
    assert_eq!(red.space(), SRGB);
    assert_eq!(red.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
    assert_eq!(red.serialization_family(), SerializationFamily::LegacySrgb);

    let missing = color("hsl(none 0% 50%)");
    assert_eq!(missing.space(), ColorSpace::Hsl);
    assert_eq!(missing.channels(), [None, Some(0.0), Some(50.0)]);
}

#[test]
fn relative_channels_are_unclamped_and_omitted_alpha_comes_from_the_origin() {
    let origin = "oklab(0.543 -0.09 -0.02 / 0.3)";
    let modified = color(&format!("oklab(from {origin} calc(1 - l) calc(a * 0.8) b)"));
    assert_eq!(modified.space(), ColorSpace::Oklab);
    let [l, a, b] = modified.channels();
    assert!((l.unwrap() - 0.457).abs() < 1e-12, "{l:?}");
    assert!((a.unwrap() + 0.072).abs() < 1e-12, "{a:?}");
    assert_eq!(b, Some(-0.02));
    assert_eq!(modified.alpha(), Some(0.3));

    // Relative rgb() channels are numbers of 255 and percentages of 255;
    // alpha alone is still clamped to [0, 1].
    let extended = color(&format!("rgb(from {origin} 510 -255 150% / 200%)"));
    assert_eq!(extended.space(), SRGB);
    assert_eq!(extended.channels(), [Some(2.0), Some(-1.0), Some(1.5)]);
    assert_eq!(extended.alpha(), Some(1.0));
    assert_eq!(extended.srgb_bytes(), None);

    let light = color(&format!("lab(from {origin} 120% 200% -200%)"));
    assert_eq!(light.channels(), [Some(120.0), Some(250.0), Some(-250.0)]);

    let signed = color("lch(from black 50 -20 0)");
    assert_eq!(signed.space(), ColorSpace::Lch);
    assert_eq!(signed.channels(), [Some(50.0), Some(-20.0), Some(0.0)]);
}

#[test]
fn complete_relative_hsl_resolves_to_extended_srgb_and_missing_keeps_hsl() {
    // HSL with saturation -100% at hue 0 is the 180-degree complement of red
    // at full saturation: cyan (0, 1, 1).
    let cyan = color("hsl(from black 0 -100 50)");
    assert_eq!(cyan.space(), SRGB);
    assert_eq!(cyan.channels(), [Some(0.0), Some(1.0), Some(1.0)]);
    assert_eq!(
        cyan.serialization_family(),
        SerializationFamily::ColorFunction
    );

    let missing = color("hsl(from black none -100 50 / none)");
    assert_eq!(missing.space(), ColorSpace::Hsl);
    assert_eq!(missing.channels(), [None, Some(-100.0), Some(50.0)]);
    assert_eq!(missing.alpha(), None);

    // Zero saturation is gray at the lightness, whatever the hue.
    let gray = color("hsl(from hsl(130 0% 50%) h s l)");
    assert_eq!(gray.space(), SRGB);
    assert_eq!(gray.channels(), [Some(0.5); 3]);
}

#[test]
fn infinite_lab_axis_is_the_largest_finite_value_at_the_used_boundary() {
    // Values 4 §10.9.1: an infinity does not escape a top-level calculation;
    // it clamps to the largest value allowed, and a Lab a/b axis has no
    // range clamp, so the axis is f64::MAX.
    let value = color("lab(50 calc(infinity) 0)");
    assert_eq!(value.space(), ColorSpace::Lab);
    assert_eq!(value.channels(), [Some(50.0), Some(f64::MAX), Some(0.0)]);
    assert_eq!(value.alpha(), Some(1.0));
}
