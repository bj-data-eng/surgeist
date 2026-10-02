#![forbid(unsafe_code)]
//! Retained specified alpha/weight leaves: CSSOM WD 2021-08-26 component
//! serialization, Values 4 WD 2024-03-12 calculation serialization, Color 4
//! CRD 2026-09-08 modern alpha, and Color 5 WD 2026-09-08 color-mix serialization.
//! Finite projected binary64 coefficients round to six fractional places with
//! exact halfway away from zero (the adopted operational policy), keeping the
//! specified calculation, dimensions, authored graph and cumulative resources.

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

fn declared_color(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color property")
    };
    value.value()
}

fn color(source: &str) -> CssColor {
    declared_color(&declaration(source)).clone()
}

fn assert_color_text(value: &CssColor, expected: &str) {
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, &before);
    assert_eq!(result.unwrap(), expected);
}

fn assert_text(source: &str, expected: &str) {
    // Both the normal recovering parser and the checked declaration constructor
    // must retain the exact authored spelling while exposing the same writer.
    let checked = declaration(source);
    assert_eq!(checked.value_components(), &components(source));
    let text = format!("color:{source}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    for value in [&checked, &report.syntax()[0]] {
        let before = value.clone();
        assert_eq!(value.importance(), CssImportance::Important);
        assert_eq!(
            value.value_components().serialize().unwrap().as_css(),
            source
        );
        let result = declared_color(value).to_specified_css();
        assert_eq!(result, declared_color(value).to_specified_css());
        assert_eq!(value, &before);
        assert_eq!(result.unwrap(), expected, "{source}");
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

// 1/128 = 0.0078125 is exact binary64, hence its micro-unit value is 7812.5.
// The neighboring decimal spellings denote the immediately adjacent binary64s.
// Percentage scaling happens before rounding: .78125 / 100 = exactly 1/128.
text_case!(
    alpha_midpoint,
    "color(--P 0 / calc(1 / 128))",
    "color(--P 0 / calc(0.007813))"
);
text_case!(
    alpha_negative_midpoint,
    "color(--P 0 / calc(-1 / 128))",
    "color(--P 0 / calc(-0.007813))"
);
text_case!(
    alpha_below_midpoint,
    "color(--P 0 / calc(.007812499999999999))",
    "color(--P 0 / calc(0.007812))"
);
text_case!(
    alpha_above_midpoint,
    "color(--P 0 / calc(.007812500000000002))",
    "color(--P 0 / calc(0.007813))"
);
text_case!(
    alpha_negative_below_magnitude_midpoint,
    "color(--P 0 / calc(-.007812499999999999))",
    "color(--P 0 / calc(-0.007812))"
);
text_case!(
    alpha_negative_above_magnitude_midpoint,
    "color(--P 0 / calc(-.007812500000000002))",
    "color(--P 0 / calc(-0.007813))"
);
text_case!(
    alpha_scaled_percentage_midpoint,
    "color(--P 0 / calc(.78125%))",
    "color(--P 0 / calc(0.007813))"
);
text_case!(
    alpha_scaled_percentage_negative_midpoint,
    "color(--P 0 / calc(-.78125%))",
    "color(--P 0 / calc(-0.007813))"
);
text_case!(
    alpha_binary_micro_below,
    "color(--P 0 / calc(5e-7))",
    "color(--P 0 / calc(0))"
);
text_case!(
    alpha_negative_binary_micro_below,
    "color(--P 0 / calc(-5e-7))",
    "color(--P 0 / calc(0))"
);
text_case!(
    alpha_carry_to_explicit_unity,
    "color(--P 0 / calc(.9999996))",
    "color(--P 0 / calc(1))"
);
text_case!(
    alpha_positive_subnormal,
    "color(--P 0 / calc(5e-324))",
    "color(--P 0 / calc(0))"
);
text_case!(
    alpha_negative_subnormal,
    "color(--P 0 / calc(-5e-324))",
    "color(--P 0 / calc(0))"
);
// 7812500000000001 < 2^53 is exact; multiplying by 128 is an exact exponent
// shift. The expected full integer is 1000000000000000128, not a decimal alias.
text_case!(
    alpha_full_integer_digits,
    "color(--P 0 / calc(7812500000000001 * 128))",
    "color(--P 0 / calc(1000000000000000128))"
);
text_case!(
    alpha_contextual_midpoint,
    "color(--P 0 / calc(.0078125 * 1em / 1px))",
    "color(--P 0 / calc(0.007813 * 1em / 1px))"
);
text_case!(
    alpha_contextual_negative_midpoint,
    "color(--P 0 / calc(-.0078125 * 1em / 1px))",
    "color(--P 0 / calc(-0.007813 * 1em / 1px))"
);
text_case!(
    alpha_contextual_percentage_coefficient,
    "color(--P 0 / calc(.0078125% * 1em / 1px))",
    "color(--P 0 / calc(0.007813% * 1em / 1px / 100%))"
);
text_case!(
    alpha_contextual_same_units_remain_symbolic,
    "color(--P 0 / calc(.0078125 * 1em / 1em))",
    "color(--P 0 / calc(0.007813 * 1em / 1em))"
);
text_case!(
    alpha_absolute_ratio_becomes_numeric,
    "color(--P 0 / calc(1 / 128 * 1in / 96px))",
    "color(--P 0 / calc(0.007813))"
);

macro_rules! family_cases {
    ($number:ident, $percentage:ident, $contextual:ident, $contextual_percentage:ident,
     $prefix:literal, $separator:literal, $suffix:literal, $output_prefix:literal, $output_separator:literal, $output_suffix:literal) => {
        #[test]
        fn $number() {
            assert_text(
                concat!($prefix, $separator, "calc(1 / 128)", $suffix),
                concat!(
                    $output_prefix,
                    $output_separator,
                    "calc(0.007813)",
                    $output_suffix
                ),
            );
        }
        #[test]
        fn $percentage() {
            assert_text(
                concat!($prefix, $separator, "calc(.78125%)", $suffix),
                concat!(
                    $output_prefix,
                    $output_separator,
                    "calc(0.007813)",
                    $output_suffix
                ),
            );
        }
        #[test]
        fn $contextual() {
            assert_text(
                concat!($prefix, $separator, "calc(.0078125 * 1em / 1px)", $suffix),
                concat!(
                    $output_prefix,
                    $output_separator,
                    "calc(0.007813 * 1em / 1px)",
                    $output_suffix
                ),
            );
        }
        #[test]
        fn $contextual_percentage() {
            assert_text(
                concat!($prefix, $separator, "calc(.0078125% * 1em / 1px)", $suffix),
                concat!(
                    $output_prefix,
                    $output_separator,
                    "calc(0.007813% * 1em / 1px / 100%)",
                    $output_suffix
                ),
            );
        }
    };
}

family_cases!(
    rgb_number,
    rgb_percentage,
    rgb_contextual,
    rgb_contextual_percentage,
    "rgb(1 2 3",
    " / ",
    ")",
    "rgba(1, 2, 3",
    ", ",
    ")"
);
family_cases!(
    rgba_modern_number,
    rgba_modern_percentage,
    rgba_modern_contextual,
    rgba_modern_contextual_percentage,
    "rgba(1 2 3",
    " / ",
    ")",
    "rgba(1, 2, 3",
    ", ",
    ")"
);
family_cases!(
    rgb_legacy_number,
    rgb_legacy_percentage,
    rgb_legacy_contextual,
    rgb_legacy_contextual_percentage,
    "rgba(1, 2, 3",
    ", ",
    ")",
    "rgba(1, 2, 3",
    ", ",
    ")"
);
family_cases!(
    hsl_number,
    hsl_percentage,
    hsl_contextual,
    hsl_contextual_percentage,
    "hsl(0 100% 50%",
    " / ",
    ")",
    "rgba(255, 0, 0",
    ", ",
    ")"
);
family_cases!(
    hsla_modern_number,
    hsla_modern_percentage,
    hsla_modern_contextual,
    hsla_modern_contextual_percentage,
    "hsla(0 100% 50%",
    " / ",
    ")",
    "rgba(255, 0, 0",
    ", ",
    ")"
);
family_cases!(
    hsl_legacy_number,
    hsl_legacy_percentage,
    hsl_legacy_contextual,
    hsl_legacy_contextual_percentage,
    "hsla(0, 100%, 50%",
    ", ",
    ")",
    "rgba(255, 0, 0",
    ", ",
    ")"
);
family_cases!(
    hwb_number,
    hwb_percentage,
    hwb_contextual,
    hwb_contextual_percentage,
    "hwb(0 0% 0%",
    " / ",
    ")",
    "rgba(255, 0, 0",
    ", ",
    ")"
);
family_cases!(
    lab_number,
    lab_percentage,
    lab_contextual,
    lab_contextual_percentage,
    "lab(20 0 0",
    " / ",
    ")",
    "lab(20 0 0",
    " / ",
    ")"
);
family_cases!(
    lch_number,
    lch_percentage,
    lch_contextual,
    lch_contextual_percentage,
    "lch(20 30 0",
    " / ",
    ")",
    "lch(20 30 0",
    " / ",
    ")"
);
family_cases!(
    oklab_number,
    oklab_percentage,
    oklab_contextual,
    oklab_contextual_percentage,
    "oklab(.2 0 0",
    " / ",
    ")",
    "oklab(0.2 0 0",
    " / ",
    ")"
);
family_cases!(
    oklch_number,
    oklch_percentage,
    oklch_contextual,
    oklch_contextual_percentage,
    "oklch(.2 .3 0",
    " / ",
    ")",
    "oklch(0.2 0.3 0",
    " / ",
    ")"
);
family_cases!(
    predefined_number,
    predefined_percentage,
    predefined_contextual,
    predefined_contextual_percentage,
    "color(display-p3 0 0 0",
    " / ",
    ")",
    "color(display-p3 0 0 0",
    " / ",
    ")"
);
family_cases!(
    custom_many_channels_number,
    custom_many_channels_percentage,
    custom_many_channels_contextual,
    custom_many_channels_contextual_percentage,
    "color(--P 0 none 0 0",
    " / ",
    ")",
    "color(--P 0 none 0 0",
    " / ",
    ")"
);

text_case!(
    weight_midpoint,
    "color-mix(red calc(.0078125%), blue)",
    "color-mix(red calc(0.007813%), blue)"
);
text_case!(
    weight_negative_midpoint,
    "color-mix(red calc(-.0078125%), blue)",
    "color-mix(red calc(-0.007813%), blue)"
);
text_case!(
    weight_below_midpoint,
    "color-mix(red calc(.007812499999999999%), blue)",
    "color-mix(red calc(0.007812%), blue)"
);
text_case!(
    weight_above_midpoint,
    "color-mix(red calc(.007812500000000002%), blue)",
    "color-mix(red calc(0.007813%), blue)"
);
text_case!(
    weight_binary_micro_below,
    "color-mix(red calc(5e-7%), blue)",
    "color-mix(red calc(0%), blue)"
);
text_case!(
    weight_carry,
    "color-mix(red calc(49.9999996%), blue)",
    "color-mix(red calc(50%), blue)"
);
text_case!(
    weight_positive_subnormal,
    "color-mix(red calc(5e-324%), blue)",
    "color-mix(red calc(0%), blue)"
);
text_case!(
    weight_negative_subnormal,
    "color-mix(red calc(-5e-324%), blue)",
    "color-mix(red calc(0%), blue)"
);
text_case!(
    weight_full_integer_digits,
    "color-mix(red calc(7812500000000001% * 128), blue)",
    "color-mix(red calc(1000000000000000128%), blue)"
);
text_case!(
    weight_contextual,
    "color-mix(red calc(.0078125% * 1em / 1px), blue)",
    "color-mix(red calc(0.007813% * 1em / 1px), blue)"
);
text_case!(
    weight_contextual_negative,
    "color-mix(red calc(-.0078125% * 1em / 1px), blue)",
    "color-mix(red calc(-0.007813% * 1em / 1px), blue)"
);
text_case!(
    weight_authored_siblings_not_normalized,
    "color-mix(in srgb, red calc(.0078125%), blue 20%, green calc(.0078125%))",
    "color-mix(in srgb, red calc(0.007813%), blue 20%, green calc(0.007813%))"
);
text_case!(
    weight_singleton_not_collapsed,
    "color-mix(red calc(.0078125%))",
    "color-mix(red calc(0.007813%))"
);
text_case!(
    weight_identical_colors_not_collapsed,
    "color-mix(red calc(.0078125%), red)",
    "color-mix(red calc(0.007813%), red)"
);
text_case!(
    weight_second_slot_prevents_filling_first,
    "color-mix(red, blue calc(.0078125%))",
    "color-mix(red, blue calc(0.007813%))"
);
text_case!(
    ordinary_mix_child_alpha,
    "color-mix(color(--P 0 / calc(1 / 128)) calc(50%), blue)",
    "color-mix(color(--P 0 / calc(0.007813)) calc(50%), blue)"
);
text_case!(
    nested_mix_alpha_and_weight,
    "color-mix(color-mix(color(--P 0 / calc(.0078125 * 1em / 1px)) calc(.0078125%), blue), red calc(50%))",
    "color-mix(color-mix(color(--P 0 / calc(0.007813 * 1em / 1px)) calc(0.007813%), blue), red calc(50%))"
);
// The outer mix's Origin mode does not turn its own ordinary Mix children into
// origin colors: their percentage alpha still acquires /100 conversion.
text_case!(
    origin_mix_child_alpha_keeps_non_origin_scaling,
    "alpha(from color-mix(color(--P 0 / calc(.78125%)) calc(50%), blue))",
    "alpha(from color-mix(color(--P 0 / calc(0.007813)) calc(50%), blue))"
);
text_case!(
    origin_mix_weight_is_retained_generic,
    "rgb(from color-mix(red calc(.0078125%), blue) r g b)",
    "rgb(from color-mix(red calc(0.007813%), blue) r g b)"
);

#[test]
fn calculated_alpha_and_weight_keep_explicit_unclamped_values() {
    for (source, expected) in [
        ("color(--P 0 / calc(1))", "color(--P 0 / calc(1))"),
        ("color(--P 0 / calc(2 * 60%))", "color(--P 0 / calc(1.2))"),
        ("color(--P 0 / calc(-120%))", "color(--P 0 / calc(-1.2))"),
        ("rgb(1 2 3 / calc(2))", "rgba(1, 2, 3, calc(2))"),
        (
            "color-mix(red calc(0%), blue)",
            "color-mix(red calc(0%), blue)",
        ),
        (
            "color-mix(red calc(50%), blue)",
            "color-mix(red calc(50%), blue)",
        ),
        (
            "color-mix(red calc(50%), blue calc(50%))",
            "color-mix(red calc(50%), blue calc(50%))",
        ),
        (
            "color-mix(red calc(-5%), blue)",
            "color-mix(red calc(-5%), blue)",
        ),
        (
            "color-mix(red calc(120%), blue)",
            "color-mix(red calc(120%), blue)",
        ),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn nonfinite_and_signed_zero_preserve_actual_projection_wrappers() {
    for (source, expected) in [
        (
            "color(--P 0 / calc(infinity))",
            "color(--P 0 / calc(infinity))",
        ),
        (
            "color(--P 0 / calc(-infinity))",
            "color(--P 0 / calc(-infinity))",
        ),
        (
            "color(--P 0 / calc(NaN + 1em / 1px))",
            "color(--P 0 / calc(NaN))",
        ),
        (
            "color(--P 0 / calc(infinity * 1%))",
            "color(--P 0 / calc(infinity))",
        ),
        (
            "color(--P 0 / calc(infinity + 1em / 1px))",
            "color(--P 0 / calc(infinity + (1em / 1px)))",
        ),
        ("color(--P 0 / calc(0 * -1))", "color(--P 0 / calc(0))"),
        (
            "color(--P 0 / calc(0 * -1 + 1em / 1px))",
            "color(--P 0 / calc((0 * -1) + (1em / 1px)))",
        ),
        (
            "color-mix(red calc(infinity * 1%), blue)",
            "color-mix(red calc(infinity * 1%), blue)",
        ),
        (
            "color-mix(red calc(-infinity * 1%), blue)",
            "color-mix(red calc(-infinity * 1%), blue)",
        ),
        (
            "color-mix(red calc(NaN * 1%), blue)",
            "color-mix(red calc(NaN * 1%), blue)",
        ),
        (
            "color-mix(red calc(0% * -1), blue)",
            "color-mix(red calc(0%), blue)",
        ),
        // This percentage product has a separate Number infinity leaf and a
        // Percentage 1 leaf. It never becomes a scalar infinity-percentage
        // operand, so no invented '(infinity * 1%)' grouping is appropriate.
        (
            "color-mix(red calc(infinity * 1% * 1em / 1px), blue)",
            "color-mix(red calc(infinity * 1% * 1em / 1px), blue)",
        ),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn protected_direct_origin_relative_profile_and_non_alpha_policies() {
    for (source, expected) in [
        ("color(--P 0 / 120%)", "color(--P 0)"),
        ("color(--P 0 / -1)", "color(--P 0 / 0)"),
        ("color(--P 0 / .0078125)", "color(--P 0 / 0.007813)"),
        ("color(--P 0 / .1234567)", "color(--P 0 / 0.123457)"),
        (
            "alpha(from color(--P 0 / calc(.78125%)))",
            "alpha(from color(--P 0 / calc(0.78125%)))",
        ),
        (
            "alpha(from color(--P 0 / 120%))",
            "alpha(from color(--P 0 / 120%))",
        ),
        (
            "alpha(from red / calc(.78125%))",
            "alpha(from red / calc(0.007813))",
        ),
        (
            "rgb(from red r g b / calc(alpha + 1 / 128))",
            "rgb(from red r g b / calc(0.007813 + alpha))",
        ),
        (
            "color(from red --P Cyan / calc(alpha + 1 / 128))",
            "color(from red --P Cyan / calc(0.007813 + alpha))",
        ),
        (
            "color(srgb calc(1 / 128) 0 0)",
            "color(srgb calc(0.0078125) 0 0)",
        ),
        ("color(--P .1234567)", "color(--P 0.1234567)"),
        (
            "color(--P calc(.0078125 * 1em / 1px))",
            "color(--P calc(0.007813 * 1em / 1px))",
        ),
        ("color-mix(red 50%, blue)", "color-mix(red, blue)"),
        ("color-mix(red 25%, blue)", "color-mix(red 25%, blue 75%)"),
        (
            "color-mix(red .0078125%, blue calc(50%))",
            "color-mix(red 0.0078125%, blue calc(50%))",
        ),
    ] {
        assert_text(source, expected);
    }
    let value = color("rgb(calc(10000000000) 0 0)");
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 17))
            .unwrap(),
        "rgb(255, 0, 0)"
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, 16))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
}

fn assert_error(value: &CssColor, limits: Limits, expected: Kind) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap_err().kind(), expected, "{limits:?}");
}

fn assert_limits(source: &str, expected: &str, inputs: usize, projections: usize) {
    let value = color(source);
    for (limits, kind) in [
        (
            Limits::new(inputs - 1, usize::MAX, usize::MAX),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, projections - 1, usize::MAX),
            Kind::ProjectionNodeLimit,
        ),
        (
            Limits::new(inputs, projections, expected.len() - 1),
            Kind::ByteLimit,
        ),
    ] {
        assert_error(&value, limits, kind);
    }
    let limits = Limits::new(inputs, projections, expected.len());
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected, "{source}");
}

#[test]
fn alpha_numeric_exact_input_projection_and_final_bytes() {
    // Color + none channel = 2/2. calc(1 / 128) has wrapper/product/two
    // leaves = 4 inputs; two leaves/inverse/resolved product = 4 projections.
    assert_limits(
        "color(--P none / calc(1 / 128))",
        "color(--P none / calc(0.007813))",
        6,
        6,
    );
}

#[test]
fn alpha_percentage_exact_scale_counts_and_bytes() {
    // calc(.78125%) = 2/1. Percentage-to-number adds a percentage leaf,
    // its inverse and the resolved product = 3 projections, no input work.
    assert_limits(
        "color(--P none / calc(.78125%))",
        "color(--P none / calc(0.007813))",
        4,
        6,
    );
}

#[test]
fn alpha_contextual_exact_counts_and_bytes() {
    // calc wrapper/product/three leaves = 5 inputs. Three leaves, inverse,
    // combined Number and product = 6 projections. Alpha uses Identity.
    assert_limits(
        "color(--P none / calc(.0078125 * 1em / 1px))",
        "color(--P none / calc(0.007813 * 1em / 1px))",
        7,
        8,
    );
}

#[test]
fn alpha_contextual_percentage_suffix_and_scale_counts() {
    // A percentage product lacks the combined Number node (5 projections).
    // The /100% scale adds percentage leaf/inverse/product = 3 more.
    assert_limits(
        "color(--P none / calc(.0078125% * 1em / 1px))",
        "color(--P none / calc(0.007813% * 1em / 1px / 100%))",
        7,
        10,
    );
}

#[test]
fn weight_numeric_exact_counts_and_percent_suffix_bytes() {
    // Mix + two named colors = 3/3. calc percentage = 2/1, Identity scale.
    assert_limits(
        "color-mix(red calc(.0078125%), blue)",
        "color-mix(red calc(0.007813%), blue)",
        5,
        4,
    );
}

#[test]
fn weight_contextual_exact_counts_and_bytes() {
    // Three color roots plus wrapper/product/three leaves (5 inputs).
    // Percentage product: 3 leaves + inverse + product = 5 projections.
    assert_limits(
        "color-mix(red calc(.0078125% * 1em / 1px), blue)",
        "color-mix(red calc(0.007813% * 1em / 1px), blue)",
        8,
        8,
    );
}

#[test]
fn weights_charge_siblings_cumulatively() {
    // Three roots + two 2/1 percentage calculations = 7/5 total.
    assert_limits(
        "color-mix(red calc(.0078125%), blue calc(.0078125%))",
        "color-mix(red calc(0.007813%), blue calc(0.007813%))",
        7,
        5,
    );
}

#[test]
fn nested_mix_charges_alpha_weight_and_remaining_bytes() {
    // Five color roots + none + Number alpha 4/4 + two weights 2/1 = 14/12.
    // The ordinary child runs after the outer and inner mix prefixes have
    // already spent output bytes; its scratch/local writer has that remainder.
    assert_limits(
        "color-mix(color-mix(color(--P none / calc(1 / 128)) calc(.0078125%), blue) calc(50%), red)",
        "color-mix(color-mix(color(--P none / calc(0.007813)) calc(0.007813%), blue) calc(50%), red)",
        14,
        12,
    );
}

#[test]
fn unchanged_coefficients_prove_exact_graph_counts_independently_of_rounding() {
    assert_limits(
        "color(--P none / calc(1 / 2))",
        "color(--P none / calc(0.5))",
        6,
        6,
    );
    assert_limits(
        "color(--P none / calc(50%))",
        "color(--P none / calc(0.5))",
        4,
        6,
    );
    assert_limits(
        "color(--P none / calc(.5 * 1em / 1px))",
        "color(--P none / calc(0.5 * 1em / 1px))",
        7,
        8,
    );
    assert_limits(
        "color(--P none / calc(.5% * 1em / 1px))",
        "color(--P none / calc(0.5% * 1em / 1px / 100%))",
        7,
        10,
    );
    assert_limits(
        "color-mix(red calc(50%), blue)",
        "color-mix(red calc(50%), blue)",
        5,
        4,
    );
    assert_limits(
        "color-mix(red calc(.5% * 1em / 1px), blue)",
        "color-mix(red calc(0.5% * 1em / 1px), blue)",
        8,
        8,
    );
    assert_limits(
        "color-mix(red calc(50%), blue calc(50%))",
        "color-mix(red calc(50%), blue calc(50%))",
        7,
        5,
    );
    assert_limits(
        "color-mix(color-mix(color(--P none / calc(1 / 2)) calc(50%), blue) calc(50%), red)",
        "color-mix(color-mix(color(--P none / calc(0.5)) calc(50%), blue) calc(50%), red)",
        14,
        12,
    );
}

#[test]
fn unchanged_coefficients_prove_scratch_and_prefix_error_precedence() {
    // These unchanged zero coefficients establish the same selected traversal
    // and exact byte thresholds without relying on the new finite text policy.
    let weight = color("color-mix(red calc(0%), blue calc(50%))");
    assert_error(&weight, Limits::new(3, usize::MAX, 8), Kind::InputNodeLimit);
    assert_error(
        &weight,
        Limits::new(usize::MAX, 2, 8),
        Kind::ProjectionNodeLimit,
    );
    assert_error(&weight, Limits::new(3, usize::MAX, 7), Kind::ByteLimit);
    let alpha = color("color-mix(color(--P none / calc(0)) calc(50%), blue)");
    let prefix = "color-mix(color(--P none / calc(0)) calc(50%), ";
    assert_error(
        &alpha,
        Limits::new(7, usize::MAX, prefix.len()),
        Kind::InputNodeLimit,
    );
    assert_error(
        &alpha,
        Limits::new(usize::MAX, 5, prefix.len()),
        Kind::ProjectionNodeLimit,
    );
    assert_error(
        &alpha,
        Limits::new(7, usize::MAX, prefix.len() - 1),
        Kind::ByteLimit,
    );
}

// The selected percentage-to-number graph multiplies by the binary64 inverse
// of 100, rather than doing a new division or decimal rounding first. Actual
// binary64 .00005 * .01 is just ABOVE half a micro-unit, so it rounds up.
text_case!(
    alpha_percentage_scale_crosses_binary_micro_boundary,
    "color(--P 0 / calc(.00005%))",
    "color(--P 0 / calc(0.000001))"
);
text_case!(
    alpha_percentage_negative_scale_crosses_binary_micro_boundary,
    "color(--P 0 / calc(-.00005%))",
    "color(--P 0 / calc(-0.000001))"
);
text_case!(
    weight_negative_below_magnitude_midpoint,
    "color-mix(red calc(-.007812499999999999%), blue)",
    "color-mix(red calc(-0.007812%), blue)"
);
text_case!(
    weight_negative_above_magnitude_midpoint,
    "color-mix(red calc(-.007812500000000002%), blue)",
    "color-mix(red calc(-0.007813%), blue)"
);
text_case!(
    alpha_contextual_carry,
    "color(--P 0 / calc(.9999996 * 1em / 1px))",
    "color(--P 0 / calc(1 * 1em / 1px))"
);
text_case!(
    weight_contextual_binary_micro_below,
    "color-mix(red calc(5e-7% * 1em / 1px), blue)",
    "color-mix(red calc(0% * 1em / 1px), blue)"
);

#[test]
fn dimensional_nonfinite_and_negative_zero_operands_keep_their_wrappers() {
    // The Percentage infinities and negative zero are scalar operands of a
    // retained sum here, unlike the Number-infinity product in the other
    // control. Each scalar's dimensional multiplication belongs in parentheses.
    for (source, expected) in [
        (
            "color-mix(red calc(infinity * 1% + 1% * 1em / 1px), blue)",
            "color-mix(red calc((infinity * 1%) + (1% * 1em / 1px)), blue)",
        ),
        (
            "color-mix(red calc(0% * -1 + 1% * 1em / 1px), blue)",
            "color-mix(red calc((0% * -1) + (1% * 1em / 1px)), blue)",
        ),
    ] {
        assert_text(source, expected);
    }
}

#[test]
fn weight_short_scratch_reaches_next_input_limit() {
    // First weight scratch calc(0%) = eight bytes. Mix root + first calc
    // wrapper/leaf exhausts three inputs; the next weight then exceeds them.
    let value = color("color-mix(red calc(5e-324%), blue calc(50%))");
    assert_error(&value, Limits::new(3, usize::MAX, 8), Kind::InputNodeLimit);
}

#[test]
fn weight_short_scratch_reaches_next_projection_limit() {
    let value = color("color-mix(red calc(5e-324%), blue calc(50%))");
    // Mix projection + first percentage leaf exhausts two projections.
    assert_error(
        &value,
        Limits::new(usize::MAX, 2, 8),
        Kind::ProjectionNodeLimit,
    );
}

#[test]
fn weight_one_below_scratch_precedes_next_input_limit() {
    let value = color("color-mix(red calc(5e-324%), blue calc(50%))");
    assert_error(&value, Limits::new(3, usize::MAX, 7), Kind::ByteLimit);
}

#[test]
fn alpha_short_scratch_reaches_next_color_input_limit_with_spent_prefix() {
    let value = color("color-mix(color(--P none / calc(5e-324)) calc(50%), blue)");
    // Root mix + first weight 2 + custom root + none + alpha 2 = 7 inputs.
    // Before the blue root, output comprises prefix, complete first color,
    // its authored weight, and separator. Exactly this bound is sufficient
    // for the rounded alpha scratch and all text before the next input visit.
    let prefix = "color-mix(color(--P none / calc(0)) calc(50%), ";
    assert_error(
        &value,
        Limits::new(7, usize::MAX, prefix.len()),
        Kind::InputNodeLimit,
    );
}

#[test]
fn alpha_short_scratch_reaches_next_color_projection_limit() {
    let value = color("color-mix(color(--P none / calc(5e-324)) calc(50%), blue)");
    // mix + weight + custom + none + alpha leaf = five projections.
    let prefix = "color-mix(color(--P none / calc(0)) calc(50%), ";
    assert_error(
        &value,
        Limits::new(usize::MAX, 5, prefix.len()),
        Kind::ProjectionNodeLimit,
    );
}

#[test]
fn alpha_one_below_remaining_prefix_precedes_next_color_input_limit() {
    let value = color("color-mix(color(--P none / calc(5e-324)) calc(50%), blue)");
    let prefix = "color-mix(color(--P none / calc(0)) calc(50%), ";
    assert_error(
        &value,
        Limits::new(7, usize::MAX, prefix.len() - 1),
        Kind::ByteLimit,
    );
}

#[test]
fn zero_limits_and_failure_precedence_remain_typed_and_atomic() {
    let value = color("color(--P none / calc(1 / 128))");
    for (limits, kind) in [
        (Limits::new(0, usize::MAX, usize::MAX), Kind::InputNodeLimit),
        (
            Limits::new(usize::MAX, 0, usize::MAX),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(usize::MAX, usize::MAX, 0), Kind::ByteLimit),
        (Limits::new(0, 0, 0), Kind::InputNodeLimit),
        (Limits::new(1, 0, 0), Kind::ProjectionNodeLimit),
        (Limits::new(1, usize::MAX, 0), Kind::InputNodeLimit),
    ] {
        assert_error(&value, limits, kind);
    }
}

fn custom_alpha(calculation: CssColorComponent) -> CssColor {
    CssColor::from_custom(
        CssCustomColor::try_new(
            CssColorProfileName::try_new("--P").unwrap(),
            vec![CssColorComponent::None],
            Some(calculation),
        )
        .unwrap(),
    )
}

#[test]
fn checked_number_alpha_retains_parsed_graph_kind_and_source() {
    let raw = components("calc(.0078125 * 1em / 1px)");
    let calculation = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    assert_eq!(calculation.components(), &raw);
    assert_eq!(calculation.expression().origin(), raw.items()[0].origin());
    let CssValueOrigin::Parsed(origin) = calculation.expression().origin() else {
        panic!("parsed graph")
    };
    assert_eq!(origin.source().as_str(), "calc(.0078125 * 1em / 1px)");
    let value = custom_alpha(CssColorComponent::NumberCalculation(calculation.clone()));
    assert_eq!(
        value.custom_value().unwrap().alpha(),
        Some(&CssColorComponent::NumberCalculation(calculation))
    );
    assert_color_text(&value, "color(--P none / calc(0.007813 * 1em / 1px))");
    assert_eq!(
        raw.serialize().unwrap().as_css(),
        "calc(.0078125 * 1em / 1px)"
    );
}

#[test]
fn checked_percentage_alpha_retains_authored_dimension_and_scaling_graph() {
    let raw = components("calc(.78125%)");
    let calculation = CssPercentageCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calculation.result_type(), CssCalculationType::Percentage);
    assert_eq!(calculation.components(), &raw);
    let value = custom_alpha(CssColorComponent::PercentageCalculation(
        calculation.clone(),
    ));
    assert_color_text(&value, "color(--P none / calc(0.007813))");
    assert_eq!(
        value.custom_value().unwrap().alpha(),
        Some(&CssColorComponent::PercentageCalculation(calculation))
    );
    assert_eq!(raw.serialize().unwrap().as_css(), "calc(.78125%)");
}

#[test]
fn programmatic_number_alpha_has_the_same_retained_coefficient_policy() {
    let raw = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "calc",
            CssComponentValues::try_new(vec![CssComponentValue::try_number(".0078125").unwrap()])
                .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let calculation = CssNumberCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(
        calculation.expression().origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(calculation.components(), &raw);
    let value = custom_alpha(CssColorComponent::NumberCalculation(calculation));
    assert_color_text(&value, "color(--P none / calc(0.007813))");
    assert_eq!(raw.serialize().unwrap().as_css(), "calc(.0078125)");
}

#[test]
fn mixed_origin_calculated_weight_retains_the_actual_leaf_source() {
    let leaf = components(".0078125%").items()[0].clone();
    let raw = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "calc",
            CssComponentValues::try_new(vec![leaf.clone()]).unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let calculation = CssPercentageCalculation::try_from_components(raw.clone()).unwrap();
    assert_eq!(calculation.origin(), &CssValueOrigin::Programmatic);
    let mut expression = calculation.expression();
    loop {
        match expression {
            CssCalculationExpressionRef::NestedCalc(value)
            | CssCalculationExpressionRef::Group(value) => expression = value.operand(),
            CssCalculationExpressionRef::Value(value) => {
                assert_eq!(value.literal().representation(), ".0078125");
                assert_eq!(value.literal().origin(), leaf.origin());
                break;
            }
            _ => panic!("percentage leaf"),
        }
    }
    let weight = CssColorMixWeight::try_calculation(calculation.clone()).unwrap();
    let value = CssColor::from_color_mix(
        CssColorMix::try_new(
            None,
            vec![
                CssColorMixComponent::new(color("red"), Some(weight)),
                CssColorMixComponent::new(color("blue"), None),
            ],
        )
        .unwrap(),
    );
    let retained = value.color_mix_value().unwrap().components()[0]
        .weight()
        .unwrap();
    assert!(retained.literal_value().is_none());
    assert_eq!(retained.calculation(), Some(&calculation));
    assert_color_text(&value, "color-mix(red calc(0.007813%), blue)");
    assert!(
        value.color_mix_value().unwrap().components()[1]
            .weight()
            .is_none()
    );
    assert_eq!(calculation.components(), &raw);
}

#[test]
fn clean_validation_and_important_declarations_preserve_alpha_weight_and_siblings() {
    let source =
        "color:color-mix(color(--P 0 / calc(1 / 128)) calc(.0078125%), blue)!important;opacity:.5";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(&validate_style_attribute(source).unwrap(), report.syntax());
    assert_eq!(report.syntax().len(), 2);
    let before = report.syntax().clone();
    assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    let result = declared_color(&report.syntax()[0]).to_specified_css();
    assert_eq!(report.syntax(), &before);
    assert_eq!(
        result.unwrap(),
        "color-mix(color(--P 0 / calc(0.007813)) calc(0.007813%), blue)"
    );
}

#[test]
fn invalid_alpha_and_weight_dimensions_recover_without_losing_siblings() {
    for invalid in ["color(--P 0 / calc(1em))", "color-mix(red calc(1), blue)"] {
        let source = format!("color:{invalid};opacity:.5");
        let report = parse_style_attribute(&source);
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        assert!(validate_style_attribute(&source).is_err());
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                components(invalid),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn alpha_and_weight_substitutions_remain_pending_contributions() {
    for source in [
        "color(--P 0 / calc(var(--a) + .0078125))",
        "color-mix(red calc(var(--w) + .0078125%), blue)",
    ] {
        let value = declaration(source);
        let before = value.clone();
        let known = value.known().unwrap();
        assert!(known.property_value().is_none());
        assert_eq!(known.substitution_dependent().unwrap().as_css(), source);
        assert!(matches!(
            expand_declaration(&value).unwrap(),
            CssExpansion::Pending(_)
        ));
        assert_eq!(value, before);
    }
    let error =
        CssPercentageCalculation::try_from_components(components("calc(var(--w) + .0078125%)"))
            .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::SubstitutionRequired
    );
    assert!(error.origin().is_some());
}
