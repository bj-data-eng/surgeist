#![forbid(unsafe_code)]
//! Color 4 CRD 2026-09-08: resolving-sRGB-values, css-serialization-of-srgb,
//! hue-syntax and sRGB-precision supply clipping, domains and positive ties.
//! Six places is the selected bounded precision. HSL eager saturation/NaN
//! normalization follows frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d.
//! Values 4 WD 2024-03-12 supplies dimensional infinity syntax; Color 5 WD
//! 2026-09-08 preserves Origin/relative/alpha/weight roles. Expected strings
//! below are independent mathematical/grammar oracles, never captured output.

use surgeist_css::*;
type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn components(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}
fn declaration(source: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components(source),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("{source}: {error:?}"))
}
fn declared_color(value: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) = value.known().unwrap().property_value().unwrap()
    else {
        panic!("checked color")
    };
    value.value()
}
fn color(source: &str) -> CssColor {
    declared_color(&declaration(source)).clone()
}
fn assert_value(value: &CssColor, expected: &str) {
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, &before);
    assert_eq!(result.unwrap(), expected);
    // Grammar reentry complements the explicit scalar oracle above.
    declaration(expected);
}
fn assert_text(source: &str, expected: &str) {
    let checked = declaration(source);
    let report = parse_style_attribute(&format!("color:{source}!important;opacity:.5"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    for value in [&checked, &report.syntax()[0]] {
        let before = value.clone();
        assert_eq!(value.importance(), CssImportance::Important);
        assert_eq!(
            value.value_components().serialize().unwrap().as_css(),
            source
        );
        let CssValueOrigin::Parsed(origin) = value.value_components().items()[0].origin() else {
            panic!("parsed function");
        };
        assert!(origin.source().as_str().contains(source));
        assert_value(declared_color(value), expected);
        assert_eq!(value, &before);
    }
}
macro_rules! text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text($source, $expected);
        }
    };
}

text_case!(
    rgb_missing_percent_scale,
    "rgb(none calc(50%) 0)",
    "color(srgb none 0.5 0)"
);
text_case!(
    rgb_missing_number_scale,
    "rgb(none calc(127.5) calc(63.75))",
    "color(srgb none 0.5 0.25)"
);
text_case!(
    rgb_missing_calculated_range,
    "rgb(none calc(200%) calc(-1))",
    "color(srgb none 1 0)"
);
text_case!(
    rgb_missing_nonfinite,
    "rgb(none calc(infinity) calc(NaN))",
    "color(srgb none 1 0)"
);
text_case!(
    rgb_missing_negative_infinity,
    "rgb(none calc(-infinity) calc(50%))",
    "color(srgb none 0 0.5)"
);
text_case!(
    rgb_missing_direct_range,
    "rgb(none 510 -255)",
    "color(srgb none 1 0)"
);
text_case!(
    rgb_missing_direct_percent_range,
    "rgb(none 200% -100%)",
    "color(srgb none 1 0)"
);
text_case!(
    rgb_missing_direct_repeating_scale,
    "rgb(none 1 254)",
    "color(srgb none 0.003922 0.996078)"
);
text_case!(
    rgb_alpha_missing_numeric_scale,
    "rgb(calc(50%) 0 0 / none)",
    "color(srgb 0.5 0 0 / none)"
);
text_case!(
    rgb_legacy_positive_tie,
    "rgb(calc(1 / 128) 0 0)",
    "rgb(0.007813, 0, 0)"
);
text_case!(
    rgb_legacy_binary_half_below,
    "rgb(calc(5e-7) 0 0)",
    "rgb(0, 0, 0)"
);
text_case!(
    rgb_legacy_fractional_percentage_control,
    "rgb(calc(0.30637254901960786%) 0 0)",
    "rgb(0.78125, 0, 0)"
);
text_case!(
    rgb_legacy_exceptional_channels,
    "rgb(calc(infinity) calc(-infinity) calc(NaN))",
    "rgb(255, 0, 0)"
);
text_case!(
    hsl_missing_number_becomes_percent,
    "hsl(none calc(50) 50%)",
    "hsl(none 50% 50%)"
);
text_case!(
    hsl_missing_percent_identity,
    "hsl(none calc(50%) calc(25%))",
    "hsl(none 50% 25%)"
);
text_case!(
    hsl_direct_negative_saturation,
    "hsl(none -50 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_direct_tiny_negative_saturation,
    "hsl(none -1e-400 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_calculated_negative_saturation,
    "hsl(none calc(-50) 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_calculated_negative_percent_saturation,
    "hsl(none calc(-50%) 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_calculated_nan_saturation,
    "hsl(none calc(NaN) 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_calculated_negative_infinite_saturation,
    "hsl(none calc(-infinity) 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hsl_positive_saturation_unbounded,
    "hsl(none calc(150) 50%)",
    "hsl(none 150% 50%)"
);
text_case!(
    hsl_positive_infinite_saturation,
    "hsl(none calc(infinity) 50%)",
    "hsl(none calc(infinity * 1%) 50%)"
);
text_case!(
    hsl_lightness_unbounded,
    "hsl(none 50% calc(-150))",
    "hsl(none 50% -150%)"
);
text_case!(
    hsl_lightness_above_hundred,
    "hsl(none 50% calc(250%))",
    "hsl(none 50% 250%)"
);
text_case!(
    hsl_lightness_infinity,
    "hsl(none 50% calc(infinity))",
    "hsl(none 50% calc(infinity * 1%))"
);
text_case!(
    hsl_lightness_negative_infinity,
    "hsl(none 50% calc(-infinity))",
    "hsl(none 50% calc(-infinity * 1%))"
);
text_case!(
    hsl_lightness_nan,
    "hsl(none 50% calc(NaN))",
    "hsl(none 50% 0%)"
);
text_case!(
    hwb_positive_tie,
    "hwb(none calc(1 / 128) 20%)",
    "hwb(none 0.007813% 20%)"
);
text_case!(
    hwb_negative_tie,
    "hwb(none calc(-1 / 128) 20%)",
    "hwb(none -0.007812% 20%)"
);
text_case!(
    hwb_percent_positive_tie,
    "hwb(none calc(1% / 128) 20%)",
    "hwb(none 0.007813% 20%)"
);
text_case!(
    hwb_unbounded_signed_percentages,
    "hwb(none calc(-20) calc(150))",
    "hwb(none -20% 150%)"
);
text_case!(
    hwb_unbounded_direct_percentages,
    "hwb(none -20% 150%)",
    "hwb(none -20% 150%)"
);
text_case!(
    hwb_negative_infinity_and_nan,
    "hwb(none calc(-infinity) calc(NaN))",
    "hwb(none calc(-infinity * 1%) 0%)"
);
text_case!(
    hwb_positive_infinity,
    "hwb(none calc(infinity) 20%)",
    "hwb(none calc(infinity * 1%) 20%)"
);
text_case!(
    hwb_finite_lexical_overflow,
    "hwb(none calc(1e400) 20%)",
    "hwb(none calc(infinity * 1%) 20%)"
);
text_case!(
    hwb_signed_zero,
    "hwb(none calc(0 * -1) calc(0))",
    "hwb(none 0% 0%)"
);
text_case!(
    hwb_positive_subnormal,
    "hwb(none calc(5e-324) 20%)",
    "hwb(none 0% 20%)"
);
text_case!(
    hwb_negative_subnormal,
    "hwb(none calc(-5e-324) 20%)",
    "hwb(none 0% 20%)"
);
text_case!(
    hwb_minimum_normal,
    "hwb(none calc(2.2250738585072014e-308) 20%)",
    "hwb(none 0% 20%)"
);
text_case!(
    hwb_binary_micro_below,
    "hwb(none calc(5e-7) 20%)",
    "hwb(none 0% 20%)"
);
text_case!(
    hwb_binary_micro_above,
    "hwb(none calc(0.0000005000000000000001) 20%)",
    "hwb(none 0.000001% 20%)"
);
text_case!(
    hwb_fractional_carry,
    "hwb(none calc(.9999996) 20%)",
    "hwb(none 1% 20%)"
);
text_case!(
    hwb_exact_large_integer,
    "hwb(none calc(7812500000000001 * 128) 20%)",
    "hwb(none 1000000000000000128% 20%)"
);
text_case!(
    hwb_hue_turn,
    "hwb(calc(.25turn) none 20%)",
    "hwb(90 none 20%)"
);
text_case!(
    hwb_hue_negative_turn,
    "hwb(calc(-.25turn) none 20%)",
    "hwb(270 none 20%)"
);
text_case!(
    hwb_hue_grad,
    "hwb(calc(100grad) none 20%)",
    "hwb(90 none 20%)"
);
text_case!(
    hwb_hue_degree,
    "hwb(calc(450deg) none 20%)",
    "hwb(90 none 20%)"
);
text_case!(
    hwb_hue_radian,
    "hwb(calc(1rad) none 20%)",
    "hwb(57.29578 none 20%)"
);
text_case!(
    hsl_hue_number_modulo,
    "hsl(calc(450) none 50%)",
    "hsl(90 none 50%)"
);
text_case!(
    hsl_hue_negative_multiple,
    "hsl(calc(-720) none 50%)",
    "hsl(0 none 50%)"
);
text_case!(
    hsl_hue_positive_multiple,
    "hsl(calc(1080) none 50%)",
    "hsl(0 none 50%)"
);
text_case!(
    hwb_hue_rounding_carry,
    "hwb(calc(359.9999996) none 20%)",
    "hwb(0 none 20%)"
);
text_case!(
    hwb_tiny_negative_hue,
    "hwb(calc(-5e-324) none 20%)",
    "hwb(0 none 20%)"
);
text_case!(
    hwb_infinite_hue,
    "hwb(calc(infinity) none 20%)",
    "hwb(0 none 20%)"
);
text_case!(
    hwb_negative_infinite_hue,
    "hwb(calc(-infinity) none 20%)",
    "hwb(0 none 20%)"
);
text_case!(hwb_nan_hue, "hwb(calc(NaN) none 20%)", "hwb(0 none 20%)");
text_case!(
    hwb_angle_infinite_hue,
    "hwb(calc(infinity * 1deg) none 20%)",
    "hwb(0 none 20%)"
);
text_case!(
    hsl_alpha_missing_all_numeric,
    "hsl(calc(450) calc(50) calc(25) / none)",
    "hsl(90 50% 25% / none)"
);
text_case!(
    hwb_alpha_missing_all_numeric,
    "hwb(calc(.25turn) calc(20) calc(30) / none)",
    "hwb(90 20% 30% / none)"
);
text_case!(
    rgb_contextual_red_numeric_green,
    "rgb(calc(1em / 1px) calc(300) calc(-1))",
    "rgb(calc(1 * 1em / 1px) 255 0)"
);
text_case!(
    rgb_contextual_missing_red,
    "rgb(none calc(1em / 1px) calc(510))",
    "color(srgb none calc(0.003922 * 1em / 1px) 1)"
);
text_case!(
    rgb_contextual_missing_green,
    "rgb(calc(510) none calc(1em / 1px))",
    "color(srgb 1 none calc(0.003922 * 1em / 1px))"
);
text_case!(
    rgb_contextual_missing_blue,
    "rgb(calc(1em / 1px) calc(-1) none)",
    "color(srgb calc(0.003922 * 1em / 1px) 0 none)"
);
text_case!(
    hsl_contextual_missing_hue,
    "hsl(none calc(-50) calc(1em / 1px))",
    "hsl(none 0% calc(1% * 1em / 1px))"
);
text_case!(
    hsl_contextual_missing_saturation,
    "hsl(calc(1em / 1px) none calc(-20))",
    "hsl(calc(1em / 1px) none -20%)"
);
text_case!(
    hsl_contextual_missing_lightness,
    "hsl(calc(450) calc(-.0078125 * 1em / 1px) none)",
    "hsl(90 calc(-0.007813 * 1% * 1em / 1px) none)"
);
text_case!(
    hsl_contextual_hue_numeric_saturation,
    "hsl(calc(1em / 1px) calc(-20%) 50%)",
    "hsl(calc(1em / 1px) 0% 50%)"
);
text_case!(
    hwb_contextual_missing_hue,
    "hwb(none calc(1em / 1px) calc(-20))",
    "hwb(none calc(1% * 1em / 1px) -20%)"
);
text_case!(
    hwb_contextual_missing_white,
    "hwb(calc(1em / 1px) none calc(-20))",
    "hwb(calc(1em / 1px) none -20%)"
);
text_case!(
    hwb_contextual_missing_black,
    "hwb(calc(450) calc(-.0078125 * 1em / 1px) none)",
    "hwb(90 calc(-0.007813 * 1% * 1em / 1px) none)"
);
text_case!(
    origin_retained_numeric_control,
    "alpha(from hwb(none calc(-1 / 128) 20%))",
    "alpha(from hwb(none calc(-0.007813) 20%))"
);
text_case!(
    relative_retained_numeric_control,
    "hwb(from red h calc(-1 / 128) b)",
    "hwb(from red h calc(-0.007813) b)"
);
text_case!(
    predefined_retained_control,
    "color(srgb calc(-1 / 128) 0 0)",
    "color(srgb calc(-0.0078125) 0 0)"
);
text_case!(
    custom_retained_control,
    "color(--P calc(-1 / 128))",
    "color(--P calc(-0.0078125))"
);
text_case!(
    lab_retained_control,
    "lab(none calc(-1 / 128) 0)",
    "lab(none calc(-0.0078125) 0)"
);
text_case!(
    oklab_retained_control,
    "oklab(none calc(-1 / 128) 0)",
    "oklab(none calc(-0.0078125) 0)"
);
text_case!(
    calculated_alpha_clamps_negative_scalar,
    "rgb(0 0 0 / calc(-1 / 128))",
    "rgba(0, 0, 0, 0)"
);
text_case!(
    calculated_weight_retained_control,
    "color-mix(red calc(1% / 128), blue)",
    "color-mix(red calc(0.007813%), blue)"
);
text_case!(
    generic_lexical_half_control,
    "rgb(from red 5e-7 g b)",
    "rgb(from red 0.000001 g b)"
);
text_case!(
    ordinary_mix_numeric_rgb_child,
    "color-mix(rgb(none calc(50%) 0), blue)",
    "color-mix(color(srgb none 0.5 0), blue)"
);
text_case!(
    ordinary_mix_numeric_hsl_child,
    "color-mix(hsl(none calc(-50) 50%), blue)",
    "color-mix(hsl(none 0% 50%), blue)"
);
text_case!(
    ordinary_mix_numeric_hwb_child,
    "color-mix(hwb(none calc(-1 / 128) 20%), blue)",
    "color-mix(hwb(none -0.007812% 20%), blue)"
);
text_case!(
    nested_mix_numeric_children,
    "color-mix(color-mix(hwb(none calc(1 / 128) 20%), blue), rgb(none calc(50%) 0))",
    "color-mix(color-mix(hwb(none 0.007813% 20%), blue), color(srgb none 0.5 0))"
);
text_case!(
    origin_mix_numeric_child,
    "alpha(from color-mix(hwb(none calc(1 / 128) 20%), blue))",
    "alpha(from color-mix(hwb(none 0.007813% 20%), blue))"
);
text_case!(
    rgb_numeric_neighbor_below,
    "rgb(calc(.007812499999999999) 0 0)",
    "rgb(0.007812, 0, 0)"
);
text_case!(
    rgb_numeric_neighbor_above,
    "rgb(calc(.007812500000000002) 0 0)",
    "rgb(0.007813, 0, 0)"
);
text_case!(
    hsl_numeric_hue_conversion_control,
    "hsl(calc(0) 100% 50%)",
    "rgb(255, 0, 0)"
);
text_case!(
    hwb_numeric_hue_conversion_control,
    "hwb(calc(0) 0% 0%)",
    "rgb(255, 0, 0)"
);
text_case!(
    hwb_positive_binary_neighbor_below,
    "hwb(none calc(0.007812499999999999) 20%)",
    "hwb(none 0.007812% 20%)"
);
text_case!(
    hwb_positive_binary_neighbor_tie,
    "hwb(none calc(0.0078125) 20%)",
    "hwb(none 0.007813% 20%)"
);
text_case!(
    hwb_positive_binary_neighbor_above,
    "hwb(none calc(0.007812500000000002) 20%)",
    "hwb(none 0.007813% 20%)"
);
text_case!(
    hwb_negative_binary_neighbor_below,
    "hwb(none calc(-0.007812499999999999) 20%)",
    "hwb(none -0.007812% 20%)"
);
text_case!(
    hwb_negative_binary_neighbor_tie,
    "hwb(none calc(-0.0078125) 20%)",
    "hwb(none -0.007812% 20%)"
);
text_case!(
    hwb_negative_binary_neighbor_above,
    "hwb(none calc(-0.007812500000000002) 20%)",
    "hwb(none -0.007813% 20%)"
);

// Post-scale outcomes are exact 1/128: 1.9921875/255 and .78125/100.
text_case!(
    rgb_missing_scaled_number_positive_tie,
    "rgb(none calc(1.9921875) 0)",
    "color(srgb none 0.007813 0)"
);
text_case!(
    rgb_missing_scaled_percentage_positive_tie,
    "rgb(none calc(.78125%) 0)",
    "color(srgb none 0.007813 0)"
);
text_case!(
    rgb_missing_direct_large_finite_range,
    "rgb(none 1e400 -1e400)",
    "color(srgb none 1 0)"
);
text_case!(
    hsl_direct_negative_percent_saturation,
    "hsl(none -50% 50%)",
    "hsl(none 0% 50%)"
);
text_case!(
    hwb_negative_binary_micro_below,
    "hwb(none calc(-5e-7) 20%)",
    "hwb(none 0% 20%)"
);
text_case!(
    hwb_negative_binary_micro_above,
    "hwb(none calc(-0.0000005000000000000001) 20%)",
    "hwb(none -0.000001% 20%)"
);
// Numeric HSL/HWB conversion consumes normalized, unrounded hue. The
// fractional colors are quarter/half channel vertices with independent RGB
// values, also exercising both post-conversion formatter call boundaries.
text_case!(
    hsl_numeric_quarter_hue_conversion,
    "hsl(calc(450) calc(100) calc(50))",
    "rgb(127.5, 255, 0)"
);
text_case!(
    hwb_numeric_quarter_hue_conversion,
    "hwb(calc(-.75turn) calc(0) calc(0))",
    "rgb(127.5, 255, 0)"
);

text_case!(
    hwb_contextual_black_numeric_white,
    "hwb(none calc(-20) calc(1em / 1px))",
    "hwb(none -20% calc(1% * 1em / 1px))"
);

// Independent exact expansion of (2^53 - 1) * 2^971, the largest finite
// binary64. Keep literal decimal digits as the oracle for the public formatter.
const MAX_FINITE_INTEGER: &str = "179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368";

#[test]
fn public_authored_lexical_seeds_have_the_independently_selected_bits() {
    for (source, bits) in [
        ("0.007812499999999999", 0x3f7fffffffffffff),
        ("0.0078125", 0x3f80000000000000),
        ("0.007812500000000002", 0x3f80000000000001),
        ("5e-7", 0x3ea0c6f7a0b5ed8d),
        ("0.0000005000000000000001", 0x3ea0c6f7a0b5ed8e),
    ] {
        assert_eq!(source.parse::<f64>().unwrap().to_bits(), bits);
    }
    // Radian conversion uses the existing binary64 180/PI factor, not an
    // invented exact-pi authority or a second angle conversion.
    assert_eq!((180.0 / std::f64::consts::PI).to_bits(), 0x404ca5dc1a63c1f8);
    assert_eq!(MAX_FINITE_INTEGER.len(), 309);
}

#[test]
fn maximum_finite_hwb_positive_value_keeps_every_whole_digit() {
    assert_text(
        "hwb(none calc(1.7976931348623157e308) 20%)",
        &format!("hwb(none {MAX_FINITE_INTEGER}% 20%)"),
    );
}
#[test]
fn maximum_finite_hwb_negative_value_keeps_every_whole_digit() {
    assert_text(
        "hwb(none calc(-1.7976931348623157e308) 20%)",
        &format!("hwb(none -{MAX_FINITE_INTEGER}% 20%)"),
    );
}
#[test]
fn maximum_finite_hsl_negative_lightness_keeps_every_whole_digit() {
    assert_text(
        "hsl(none 50% calc(-1.7976931348623157e308))",
        &format!("hsl(none 50% -{MAX_FINITE_INTEGER}%)"),
    );
}
#[test]
fn maximum_finite_hsl_lightness_keeps_every_whole_digit() {
    assert_text(
        "hsl(none 50% calc(1.7976931348623157e308))",
        &format!("hsl(none 50% {MAX_FINITE_INTEGER}%)"),
    );
}

fn assert_error(value: &CssColor, limits: Limits, kind: Kind) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap_err().kind(), kind);
}
fn assert_limited(value: &CssColor, limits: Limits, expected: &str) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap(), expected);
    declaration(expected);
}

#[test]
fn numeric_missing_rgb_has_source_derived_cumulative_visits() {
    let value = color("rgb(none calc(255) calc(0))");
    // Color root (1), none (1), two Calc/leaf trees (2 each): six inputs.
    // Root/none (2), each leaf + number scale + resolved product (3): eight
    // projections. Calc wrappers add no arena node. No exact literal work.
    let expected = "color(srgb none 1 0)";
    for (limits, kind) in [
        (Limits::new(0, usize::MAX, usize::MAX), Kind::InputNodeLimit),
        (Limits::new(1, 0, usize::MAX), Kind::ProjectionNodeLimit),
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
        (Limits::new(usize::MAX, usize::MAX, 0), Kind::ByteLimit),
        (Limits::new(5, 8, usize::MAX), Kind::InputNodeLimit),
        (Limits::new(6, 7, usize::MAX), Kind::ProjectionNodeLimit),
        (Limits::new(6, 8, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_error(&value, limits, kind);
    }
    assert_limited(&value, Limits::new(6, 8, expected.len()), expected);
}

#[test]
fn shrinking_hsl_scratch_still_requires_nineteen_remaining_bytes() {
    let value = color("hsl(none calc(-10000000000) 50%)");
    let scratch = "calc(-10000000000%)";
    let expected = "hsl(none 0% 50%)";
    assert_eq!(scratch.len(), 19);
    assert_eq!(expected.len(), 16);
    // Before capture: color/none + Calc/leaf = four inputs; root/none +
    // leaf/percentage scale/resolved product = five projections. The last
    // direct lightness adds one input only after successful scratch capture.
    for (limits, kind) in [
        (Limits::new(3, usize::MAX, 18), Kind::InputNodeLimit),
        (Limits::new(4, 4, 18), Kind::ProjectionNodeLimit),
        (Limits::new(4, 5, 18), Kind::ByteLimit),
        (Limits::new(usize::MAX, usize::MAX, 18), Kind::ByteLimit),
    ] {
        assert_error(&value, limits, kind);
    }
    assert_limited(&value, Limits::new(5, usize::MAX, 19), expected);
}

#[test]
fn discarded_rgb_scratch_remains_seventeen_bytes() {
    let value = color("rgb(calc(10000000000) 0 0)");
    assert_eq!("calc(10000000000)".len(), 17);
    assert_eq!("rgb(255, 0, 0)".len(), 14);
    // Before first capture: color + Calc/leaf = three inputs. Color +
    // leaf/number scale/resolved product = four projections.
    for (limits, kind) in [
        (Limits::new(2, usize::MAX, 16), Kind::InputNodeLimit),
        (Limits::new(3, 3, 16), Kind::ProjectionNodeLimit),
        (Limits::new(3, 4, 16), Kind::ByteLimit),
    ] {
        assert_error(&value, limits, kind);
    }
    assert_limited(
        &value,
        Limits::new(usize::MAX, usize::MAX, 17),
        "rgb(255, 0, 0)",
    );
}

#[test]
fn hwb_successful_capture_still_has_twenty_three_final_bytes() {
    let value = color("hwb(none calc(1 / 128) 20%)");
    assert_eq!("calc(0.0078125%)".len(), 16);
    let expected = "hwb(none 0.007813% 20%)";
    assert_eq!(expected.len(), 23);
    for bytes in [16, 22] {
        assert_error(
            &value,
            Limits::new(usize::MAX, usize::MAX, bytes),
            Kind::ByteLimit,
        );
    }
    assert_limited(&value, Limits::new(usize::MAX, usize::MAX, 23), expected);
}

fn maximum_finite_bytes(sign: &str) {
    let value = color(&format!("hwb(none calc({sign}1.7976931348623157e308) 20%)"));
    let expected = format!("hwb(none {sign}{MAX_FINITE_INTEGER}% 20%)");
    // 309 integer digits + 15 syntax bytes, plus one for negative sign.
    let bytes = 324 + sign.len();
    assert_eq!(expected.len(), bytes);
    assert_error(
        &value,
        Limits::new(usize::MAX, usize::MAX, bytes - 1),
        Kind::ByteLimit,
    );
    assert_limited(
        &value,
        Limits::new(usize::MAX, usize::MAX, bytes),
        &expected,
    );
}

#[test]
fn maximum_finite_positive_final_bytes() {
    maximum_finite_bytes("");
}
#[test]
fn maximum_finite_negative_final_bytes() {
    maximum_finite_bytes("-");
}

#[test]
fn sibling_numeric_colors_share_visits_and_output_atomically() {
    let value = color("rgb(none calc(255) calc(0))");
    let pair = CssBorderColorPair::new(value.clone(), Some(value));
    let before = pair.clone();
    let expected = "color(srgb none 1 0) color(srgb none 1 0)";
    // Pair root + two colors: inputs 1 + 2*6 = 13; projections 1 + 2*8 =17.
    for (limits, kind) in [
        (Limits::new(12, 17, usize::MAX), Kind::InputNodeLimit),
        (Limits::new(13, 16, usize::MAX), Kind::ProjectionNodeLimit),
        (Limits::new(13, 17, expected.len() - 1), Kind::ByteLimit),
    ] {
        let result = pair.serialize_specified_with_limits(limits);
        assert_eq!(result, pair.serialize_specified_with_limits(limits));
        assert_eq!(pair, before);
        assert_eq!(result.unwrap_err().kind(), kind);
    }
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(13, 17, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(pair, before);
}

#[test]
fn utf8_prefix_spends_bytes_before_shrinking_hsl_scratch() {
    let pair = CssBorderColorPair::new(
        color("color(--é 0)"),
        Some(color("hsl(none calc(-10000000000) 50%)")),
    );
    let before = pair.clone();
    let prefix = "color(--é 0) ";
    let expected = "color(--é 0) hsl(none 0% 50%)";
    assert_eq!(prefix.len(), 14);
    assert_eq!(expected.len(), 30);
    let limits = Limits::new(usize::MAX, usize::MAX, prefix.len() + 18);
    let result = pair.serialize_specified_with_limits(limits);
    assert_eq!(result, pair.serialize_specified_with_limits(limits));
    assert_eq!(pair, before);
    assert_eq!(result.unwrap_err().kind(), Kind::ByteLimit);
    assert_eq!(
        pair.serialize_specified_with_limits(Limits::new(
            usize::MAX,
            usize::MAX,
            prefix.len() + 19
        ))
        .unwrap(),
        expected
    );
    assert_eq!(pair, before);
}

fn number(value: &str, programmatic: bool) -> CssColorComponent {
    let token = if programmatic {
        CssComponentValue::try_number(value).unwrap()
    } else {
        components(value).items()[0].clone()
    };
    CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
}
fn number_calc(value: &str, programmatic: bool) -> CssNumberCalculation {
    let raw = if programmatic {
        let leaf = CssComponentValue::try_number(value).unwrap();
        let args = CssComponentValues::try_new(vec![leaf]).unwrap();
        let function = CssComponentValue::try_function("calc", args).unwrap();
        CssComponentValues::try_new(vec![function]).unwrap()
    } else {
        components(&format!("calc({value})"))
    };
    CssNumberCalculation::try_from_components(raw).unwrap()
}
fn checked_rgb_graph(programmatic: bool) {
    let calculation = number_calc("127.5", programmatic);
    let raw = calculation.components().clone();
    let raw_before = raw.clone();
    assert_eq!(calculation.expression().origin(), raw.items()[0].origin());
    if programmatic {
        assert_eq!(raw.items()[0].origin(), &CssValueOrigin::Programmatic);
    }
    let value = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [
                CssColorComponent::None,
                CssColorComponent::NumberCalculation(calculation.clone()),
                number("0", programmatic),
            ],
            None,
        )
        .unwrap(),
    );
    assert_error(&value, Limits::new(0, 0, 0), Kind::InputNodeLimit);
    assert_error(
        &value,
        Limits::new(usize::MAX, 0, usize::MAX),
        Kind::ProjectionNodeLimit,
    );
    assert_error(
        &value,
        Limits::new(usize::MAX, usize::MAX, 1),
        Kind::ByteLimit,
    );
    assert_eq!(
        value.rgb_value().unwrap().channels()[1],
        CssColorComponent::NumberCalculation(calculation)
    );
    assert_eq!(raw, raw_before);
    assert_eq!(raw.serialize().unwrap().as_css(), "calc(127.5)");
    assert_value(&value, "color(srgb none 0.5 0)");
}
fn checked_hsl_graph(programmatic: bool) {
    let calculation = number_calc("-50", programmatic);
    let value = CssColor::from_hsl(
        CssHslColor::try_new(
            CssColorSyntax::Modern,
            CssColorHue::None,
            CssColorComponent::NumberCalculation(calculation),
            number("50", programmatic),
            None,
        )
        .unwrap(),
    );
    assert_error(&value, Limits::new(0, 0, 0), Kind::InputNodeLimit);
    assert_error(
        &value,
        Limits::new(usize::MAX, 0, usize::MAX),
        Kind::ProjectionNodeLimit,
    );
    assert_error(
        &value,
        Limits::new(usize::MAX, usize::MAX, 1),
        Kind::ByteLimit,
    );
    assert_value(&value, "hsl(none 0% 50%)");
}
fn checked_hwb_graph(programmatic: bool) {
    let calculation = number_calc("-0.0078125", programmatic);
    let value = CssColor::from_hwb(
        CssHwbColor::try_new(
            CssColorHue::None,
            CssColorComponent::NumberCalculation(calculation),
            number("20", programmatic),
            None,
        )
        .unwrap(),
    );
    assert_error(&value, Limits::new(0, 0, 0), Kind::InputNodeLimit);
    assert_error(
        &value,
        Limits::new(usize::MAX, 0, usize::MAX),
        Kind::ProjectionNodeLimit,
    );
    assert_error(
        &value,
        Limits::new(usize::MAX, usize::MAX, 1),
        Kind::ByteLimit,
    );
    assert_value(&value, "hwb(none -0.007812% 20%)");
}

#[test]
fn parsed_rgb_numeric_graph_keeps_its_origin() {
    checked_rgb_graph(false);
}
#[test]
fn programmatic_rgb_numeric_graph_keeps_its_origin() {
    checked_rgb_graph(true);
}
#[test]
fn parsed_hsl_numeric_graph_keeps_its_origin() {
    checked_hsl_graph(false);
}
#[test]
fn programmatic_hsl_numeric_graph_keeps_its_origin() {
    checked_hsl_graph(true);
}
#[test]
fn parsed_hwb_numeric_graph_keeps_its_origin() {
    checked_hwb_graph(false);
}
#[test]
fn programmatic_hwb_numeric_graph_keeps_its_origin() {
    checked_hwb_graph(true);
}

fn composition_text(property: CssKnownProperty, source: &str) -> String {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(property),
        components(source),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.importance(), CssImportance::Important);
    let before = declaration.clone();
    let value = declaration.known().unwrap().property_value().unwrap();
    let text = match value {
        CssKnownPropertyValueRef::BackgroundImage(value) => {
            value.images().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::Background(value) => {
            value.background().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::BorderColor(value) => {
            value.value().serialize_specified().unwrap()
        }
        CssKnownPropertyValueRef::BoxShadow(value) => value.value().serialize_specified().unwrap(),
        _ => panic!("selected composition front door"),
    };
    assert_eq!(declaration, before);
    parse_property_value(
        CssPropertyNameRef::Known(property),
        components(&text),
        CssImportance::Important,
    )
    .unwrap();
    text
}
#[test]
fn gradient_numeric_stop_uses_ordinary_scalar_policy() {
    assert_eq!(
        composition_text(
            CssKnownProperty::BackgroundImage,
            "linear-gradient(rgb(none calc(50%) 0), blue)"
        ),
        "linear-gradient(color(srgb none 0.5 0), blue)"
    );
}
#[test]
fn background_final_color_uses_ordinary_scalar_policy() {
    assert_eq!(
        composition_text(CssKnownProperty::Background, "hsl(none calc(-50) 50%)"),
        "hsl(none 0% 50%)"
    );
}
#[test]
fn border_siblings_use_ordinary_scalar_policy() {
    assert_eq!(
        composition_text(
            CssKnownProperty::BorderColor,
            "rgb(none calc(50%) 0) hwb(none calc(-1 / 128) 20%)"
        ),
        "color(srgb none 0.5 0) hwb(none -0.007812% 20%)"
    );
}
#[test]
fn shadow_color_uses_ordinary_scalar_policy() {
    assert_eq!(
        composition_text(
            CssKnownProperty::BoxShadow,
            "hsl(none calc(50) 50%) 1px 2px"
        ),
        "hsl(none 50% 50%) 1px 2px"
    );
}
