#![forbid(unsafe_code)]
//! Color 5 WD 2026-09-08 origin and relative serialization; CSSOM WD
//! 2021-08-26 generic numbers. Decimal ties use the adopted FIXED policy:
//! nearest, ties away from zero. Expectations use exact authored decimals,
//! destination percentage factors, and conversion before six-place rounding.
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
    .unwrap_or_else(|e| panic!("{source}: {e:?}"))
}
fn declaration_color(d: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(c) = d.known().unwrap().property_value().unwrap() else {
        panic!("color")
    };
    c.value()
}
fn color(source: &str) -> CssColor {
    declaration_color(&declaration(source)).clone()
}
fn assert_text(source: &str, expected: &str) {
    let d = declaration(source);
    let before = d.clone();
    assert_eq!(d.value_components(), &components(source));
    let first = declaration_color(&d).to_specified_css();
    assert_eq!(first, declaration_color(&d).to_specified_css());
    assert_eq!(d, before);
    assert_eq!(first.unwrap(), expected, "{source}");
    // Reparse supplements the independent literal oracle.
    assert_eq!(color(expected).to_specified_css().unwrap(), expected);
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
    rgb_percentage_slot_0,
    "rgb(from blue 20% g b)",
    "rgb(from blue 51 g b)"
);
text_case!(
    rgb_percentage_slot_1,
    "rgb(from blue r -20% b)",
    "rgb(from blue r -51 b)"
);
text_case!(
    rgb_percentage_slot_2,
    "rgb(from blue r g 120%)",
    "rgb(from blue r g 306)"
);
text_case!(
    hsl_percentage_slot_1,
    "hsl(from blue h 12.3456789% l)",
    "hsl(from blue h 12.345679 l)"
);
text_case!(
    hsl_percentage_slot_2,
    "hsl(from blue h s -12.3456789%)",
    "hsl(from blue h s -12.345679)"
);
text_case!(
    hwb_percentage_slot_1,
    "hwb(from blue h 12.3456789% b)",
    "hwb(from blue h 12.345679 b)"
);
text_case!(
    hwb_percentage_slot_2,
    "hwb(from blue h w 120.1234567%)",
    "hwb(from blue h w 120.123457)"
);
text_case!(
    lab_percentage_slot_0,
    "lab(from red 20% a b)",
    "lab(from red 20 a b)"
);
text_case!(
    lab_percentage_slot_1,
    "lab(from red l 20% b)",
    "lab(from red l 25 b)"
);
text_case!(
    lab_percentage_slot_2,
    "lab(from red l a -20%)",
    "lab(from red l a -25)"
);
text_case!(
    lch_percentage_slot_0,
    "lch(from red 20% c h)",
    "lch(from red 20 c h)"
);
text_case!(
    lch_percentage_slot_1,
    "lch(from red l 20% h)",
    "lch(from red l 30 h)"
);
text_case!(
    oklab_percentage_slot_0,
    "oklab(from red 20% a b)",
    "oklab(from red 0.2 a b)"
);
text_case!(
    oklab_percentage_slot_1,
    "oklab(from red l 20% b)",
    "oklab(from red l 0.08 b)"
);
text_case!(
    oklab_percentage_slot_2,
    "oklab(from red l a -20%)",
    "oklab(from red l a -0.08)"
);
text_case!(
    oklch_percentage_slot_0,
    "oklch(from red 20% c h)",
    "oklch(from red 0.2 c h)"
);
text_case!(
    oklch_percentage_slot_1,
    "oklch(from red l 20% h)",
    "oklch(from red l 0.08 h)"
);
text_case!(
    color_percentage_slot_0,
    "color(from red srgb 20% g b)",
    "color(from red srgb 0.2 g b)"
);
text_case!(
    color_percentage_slot_1,
    "color(from red srgb r -20% b)",
    "color(from red srgb r -0.2 b)"
);
text_case!(
    color_percentage_slot_2,
    "color(from red srgb r g 120%)",
    "color(from red srgb r g 1.2)"
);
text_case!(
    custom_percentage_scaling,
    "color(from red --P 12.34567%)",
    "color(from red --P 0.123457)"
);
text_case!(
    custom_percentage_negative_unclamped,
    "color(from red --P -120%)",
    "color(from red --P -1.2)"
);
text_case!(
    custom_reference_case_none,
    "color(from red --P Cyan none 20%)",
    "color(from red --P Cyan none 0.2)"
);
text_case!(
    relative_positive_tie,
    "rgb(from red .0078125 g b)",
    "rgb(from red 0.007813 g b)"
);
text_case!(
    custom_positive_tie,
    "color(from red --P .0078125)",
    "color(from red --P 0.007813)"
);
text_case!(
    origin_positive_tie,
    "alpha(from rgb(.0078125 0 0))",
    "alpha(from rgb(0.007813 0 0))"
);
text_case!(
    relative_negative_tie,
    "rgb(from red -.0078125 g b)",
    "rgb(from red -0.007813 g b)"
);
text_case!(
    custom_negative_tie,
    "color(from red --P -.0078125)",
    "color(from red --P -0.007813)"
);
text_case!(
    origin_negative_tie,
    "alpha(from rgb(-.0078125 0 0))",
    "alpha(from rgb(-0.007813 0 0))"
);
text_case!(
    relative_below_tie,
    "rgb(from red .00781249 g b)",
    "rgb(from red 0.007812 g b)"
);
text_case!(
    custom_below_tie,
    "color(from red --P .00781249)",
    "color(from red --P 0.007812)"
);
text_case!(
    origin_below_tie,
    "alpha(from rgb(.00781249 0 0))",
    "alpha(from rgb(0.007812 0 0))"
);
text_case!(
    relative_above_tie,
    "rgb(from red .00781251 g b)",
    "rgb(from red 0.007813 g b)"
);
text_case!(
    custom_above_tie,
    "color(from red --P .00781251)",
    "color(from red --P 0.007813)"
);
text_case!(
    origin_above_tie,
    "alpha(from rgb(.00781251 0 0))",
    "alpha(from rgb(0.007813 0 0))"
);
text_case!(
    relative_lexical_micro_tie,
    "rgb(from red 5e-7 g b)",
    "rgb(from red 0.000001 g b)"
);
text_case!(
    custom_lexical_micro_tie,
    "color(from red --P 5e-7)",
    "color(from red --P 0.000001)"
);
text_case!(
    origin_lexical_micro_tie,
    "alpha(from rgb(5e-7 0 0))",
    "alpha(from rgb(0.000001 0 0))"
);
text_case!(
    relative_negative_micro_tie,
    "rgb(from red -5e-7 g b)",
    "rgb(from red -0.000001 g b)"
);
text_case!(
    custom_negative_micro_tie,
    "color(from red --P -5e-7)",
    "color(from red --P -0.000001)"
);
text_case!(
    origin_negative_micro_tie,
    "alpha(from rgb(-5e-7 0 0))",
    "alpha(from rgb(-0.000001 0 0))"
);
text_case!(
    relative_rounded_zero,
    "rgb(from red -4e-7 g b)",
    "rgb(from red 0 g b)"
);
text_case!(
    custom_rounded_zero,
    "color(from red --P -4e-7)",
    "color(from red --P 0)"
);
text_case!(
    origin_rounded_zero,
    "alpha(from rgb(-4e-7 0 0))",
    "alpha(from rgb(0 0 0))"
);
text_case!(
    relative_signed_zero,
    "rgb(from red -0 g b)",
    "rgb(from red 0 g b)"
);
text_case!(
    custom_signed_zero,
    "color(from red --P -0)",
    "color(from red --P 0)"
);
text_case!(
    origin_signed_zero,
    "alpha(from rgb(-0 0 0))",
    "alpha(from rgb(0 0 0))"
);
text_case!(
    relative_carry,
    "rgb(from red .9999996 g b)",
    "rgb(from red 1 g b)"
);
text_case!(
    custom_carry,
    "color(from red --P .9999996)",
    "color(from red --P 1)"
);
text_case!(
    origin_carry,
    "alpha(from rgb(.9999996 0 0))",
    "alpha(from rgb(1 0 0))"
);
text_case!(
    relative_integer_digits,
    "rgb(from red 1000000000000000100 g b)",
    "rgb(from red 1000000000000000100 g b)"
);
text_case!(
    custom_integer_digits,
    "color(from red --P 1000000000000000100)",
    "color(from red --P 1000000000000000100)"
);
text_case!(
    origin_integer_digits,
    "alpha(from rgb(1000000000000000100 0 0))",
    "alpha(from rgb(1000000000000000100 0 0))"
);
text_case!(
    rgb_conversion_precedes_rounding,
    "rgb(from red .0000002% g b)",
    "rgb(from red 0.000001 g b)"
);
text_case!(
    lab_axis_conversion_precedes_rounding,
    "lab(from red l .0000004% b)",
    "lab(from red l 0.000001 b)"
);
text_case!(
    lch_chroma_conversion_precedes_rounding,
    "lch(from red l .000001% h)",
    "lch(from red l 0.000002 h)"
);
text_case!(
    oklab_axis_exact_positive_tie,
    "oklab(from red l .000125% b)",
    "oklab(from red l 0.000001 b)"
);
text_case!(
    oklch_chroma_exact_negative_tie,
    "oklch(from red l -.000125% h)",
    "oklch(from red l -0.000001 h)"
);
text_case!(
    custom_conversion_precedes_rounding,
    "color(from red --P .00005%)",
    "color(from red --P 0.000001)"
);
text_case!(
    origin_rgb_preserves_categories,
    "alpha(from rgb(.1234567 -1.1234567 300.1234567 / 120.1234567%))",
    "alpha(from rgb(0.123457 -1.123457 300.123457 / 120.123457%))"
);
text_case!(
    origin_hsl_preserves_categories,
    "alpha(from hsl(360 120.1234567% -1.1234567% / 1.1234567))",
    "alpha(from hsl(360 120.123457% -1.123457% / 1.123457))"
);
text_case!(
    origin_hwb_preserves_categories,
    "alpha(from hwb(360 120.1234567% -1.1234567%))",
    "alpha(from hwb(360 120.123457% -1.123457%))"
);
text_case!(
    origin_lab_preserves_categories,
    "alpha(from lab(120.1234567% -30.1234567% 40.1234567))",
    "alpha(from lab(120.123457% -30.123457% 40.123457))"
);
text_case!(
    origin_lch_preserves_categories,
    "alpha(from lch(120.1234567% -30.1234567% 360))",
    "alpha(from lch(120.123457% -30.123457% 360))"
);
text_case!(
    origin_oklab_preserves_categories,
    "alpha(from oklab(120.1234567% -30.1234567% 40.1234567))",
    "alpha(from oklab(120.123457% -30.123457% 40.123457))"
);
text_case!(
    origin_oklch_preserves_categories,
    "alpha(from oklch(120.1234567% -30.1234567% 360))",
    "alpha(from oklch(120.123457% -30.123457% 360))"
);
text_case!(
    origin_predefined_preserves_categories,
    "alpha(from color(srgb 120.1234567% -30.1234567% 40.1234567))",
    "alpha(from color(srgb 120.123457% -30.123457% 40.123457))"
);
text_case!(
    origin_custom_preserves_categories,
    "alpha(from color(--P 120.1234567% -30.1234567% 40.1234567 / 1.1234567))",
    "alpha(from color(--P 120.123457% -30.123457% 40.123457 / 1.123457))"
);
text_case!(
    origin_hsl_hue_number,
    "alpha(from hsl(360 20% 30%))",
    "alpha(from hsl(360 20% 30%))"
);
text_case!(
    origin_hsl_hue_turn,
    "alpha(from hsl(1turn 20% 30%))",
    "alpha(from hsl(360deg 20% 30%))"
);
text_case!(
    origin_hsl_hue_tiny_turn,
    "alpha(from hsl(.0000001turn 20% 30%))",
    "alpha(from hsl(0.000036deg 20% 30%))"
);
text_case!(
    origin_hsl_hue_radian,
    "alpha(from hsl(1rad 20% 30%))",
    "alpha(from hsl(57.29578deg 20% 30%))"
);
text_case!(
    origin_hsl_hue_negative,
    "alpha(from hsl(-720 20% 30%))",
    "alpha(from hsl(-720 20% 30%))"
);
text_case!(
    relative_hsl_hue_turn,
    "hsl(from red .0000001turn s l)",
    "hsl(from red 0.000036deg s l)"
);
text_case!(
    relative_hsl_hue_radian,
    "hsl(from red 1rad s l)",
    "hsl(from red 57.29578deg s l)"
);
text_case!(
    relative_hsl_hue_number,
    "hsl(from red 360.1234567 s l)",
    "hsl(from red 360.123457 s l)"
);
text_case!(
    origin_hwb_hue_number,
    "alpha(from hwb(360 20% 30%))",
    "alpha(from hwb(360 20% 30%))"
);
text_case!(
    origin_hwb_hue_turn,
    "alpha(from hwb(1turn 20% 30%))",
    "alpha(from hwb(360deg 20% 30%))"
);
text_case!(
    origin_hwb_hue_tiny_turn,
    "alpha(from hwb(.0000001turn 20% 30%))",
    "alpha(from hwb(0.000036deg 20% 30%))"
);
text_case!(
    origin_hwb_hue_radian,
    "alpha(from hwb(1rad 20% 30%))",
    "alpha(from hwb(57.29578deg 20% 30%))"
);
text_case!(
    origin_hwb_hue_negative,
    "alpha(from hwb(-720 20% 30%))",
    "alpha(from hwb(-720 20% 30%))"
);
text_case!(
    relative_hwb_hue_turn,
    "hwb(from red .0000001turn w b)",
    "hwb(from red 0.000036deg w b)"
);
text_case!(
    relative_hwb_hue_radian,
    "hwb(from red 1rad w b)",
    "hwb(from red 57.29578deg w b)"
);
text_case!(
    relative_hwb_hue_number,
    "hwb(from red 360.1234567 w b)",
    "hwb(from red 360.123457 w b)"
);
text_case!(
    origin_lch_hue_number,
    "alpha(from lch(20 30 360))",
    "alpha(from lch(20 30 360))"
);
text_case!(
    origin_lch_hue_turn,
    "alpha(from lch(20 30 1turn))",
    "alpha(from lch(20 30 360deg))"
);
text_case!(
    origin_lch_hue_tiny_turn,
    "alpha(from lch(20 30 .0000001turn))",
    "alpha(from lch(20 30 0.000036deg))"
);
text_case!(
    origin_lch_hue_radian,
    "alpha(from lch(20 30 1rad))",
    "alpha(from lch(20 30 57.29578deg))"
);
text_case!(
    origin_lch_hue_negative,
    "alpha(from lch(20 30 -720))",
    "alpha(from lch(20 30 -720))"
);
text_case!(
    relative_lch_hue_turn,
    "lch(from red l c .0000001turn)",
    "lch(from red l c 0.000036deg)"
);
text_case!(
    relative_lch_hue_radian,
    "lch(from red l c 1rad)",
    "lch(from red l c 57.29578deg)"
);
text_case!(
    relative_lch_hue_number,
    "lch(from red l c 360.1234567)",
    "lch(from red l c 360.123457)"
);
text_case!(
    origin_oklch_hue_number,
    "alpha(from oklch(.2 .3 360))",
    "alpha(from oklch(0.2 0.3 360))"
);
text_case!(
    origin_oklch_hue_turn,
    "alpha(from oklch(.2 .3 1turn))",
    "alpha(from oklch(0.2 0.3 360deg))"
);
text_case!(
    origin_oklch_hue_tiny_turn,
    "alpha(from oklch(.2 .3 .0000001turn))",
    "alpha(from oklch(0.2 0.3 0.000036deg))"
);
text_case!(
    origin_oklch_hue_radian,
    "alpha(from oklch(.2 .3 1rad))",
    "alpha(from oklch(0.2 0.3 57.29578deg))"
);
text_case!(
    origin_oklch_hue_negative,
    "alpha(from oklch(.2 .3 -720))",
    "alpha(from oklch(0.2 0.3 -720))"
);
text_case!(
    relative_oklch_hue_turn,
    "oklch(from red l c .0000001turn)",
    "oklch(from red l c 0.000036deg)"
);
text_case!(
    relative_oklch_hue_radian,
    "oklch(from red l c 1rad)",
    "oklch(from red l c 57.29578deg)"
);
text_case!(
    relative_oklch_hue_number,
    "oklch(from red l c 360.1234567)",
    "oklch(from red l c 360.123457)"
);
text_case!(
    nested_relative,
    "alpha(from rgb(from red 20% g b))",
    "alpha(from rgb(from red 51 g b))"
);
text_case!(
    nested_custom,
    "alpha(from color(from red --P 12.34567%))",
    "alpha(from color(from red --P 0.123457))"
);
text_case!(
    mix_relative,
    "color-mix(rgb(from red 20% g b), blue)",
    "color-mix(rgb(from red 51 g b), blue)"
);
text_case!(
    mix_custom,
    "color-mix(color(from red --P 12.34567%), blue)",
    "color-mix(color(from red --P 0.123457), blue)"
);
text_case!(
    origin_mix,
    "alpha(from color-mix(rgb(from red 20% g b), blue))",
    "alpha(from color-mix(rgb(from red 51 g b), blue))"
);
text_case!(
    binary_calculation_control,
    "rgb(from red calc(5e-7) g b)",
    "rgb(from red calc(0) g b)"
);
text_case!(
    relative_percentage_calculation_control,
    "rgb(from red calc(30%) g b)",
    "rgb(from red calc(30%) g b)"
);
text_case!(
    custom_calculation_control,
    "color(from red --P calc(.78125%))",
    "color(from red --P calc(0.007813))"
);
text_case!(
    ordinary_origin_calculation_control,
    "alpha(from rgb(calc(1 / 128) 0 0))",
    "alpha(from rgb(calc(0.007813) 0 0))"
);
text_case!(
    standalone_precision_control,
    "lab(50 .1234567 0)",
    "lab(50 0.1234567 0)"
);
text_case!(
    standalone_rgb_quantization_control,
    "rgb(300 0 0)",
    "rgb(255, 0, 0)"
);
text_case!(
    standalone_hue_modulo_control,
    "lch(20 30 720)",
    "lch(20 30 0)"
);
text_case!(
    literal_alpha_explicit_unity_control,
    "rgb(from red r g b / 120%)",
    "rgb(from red r g b / 1)"
);
text_case!(
    custom_alpha_control,
    "color(from red --P Cyan / -.1)",
    "color(from red --P Cyan / 0)"
);
text_case!(
    relative_alpha_control,
    "alpha(from red / .1234567)",
    "alpha(from red / 0.123457)"
);
text_case!(
    mix_weight_control,
    "color-mix(in srgb, red calc(.78125%), blue)",
    "color-mix(in srgb, red calc(0.78125%), blue)"
);

fn assert_bytes(source: &str, expected: &str) {
    let value = color(source);
    let before = value.clone();
    let n = expected.len();
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, n - 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, n))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, n))
            .unwrap(),
        expected
    );
    assert_eq!(value, before);
}
macro_rules! byte_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_bytes($source, $expected);
        }
    };
}
byte_case!(
    relative_final_byte_edge,
    "rgb(from red .1234567 g b)",
    "rgb(from red 0.123457 g b)"
);
byte_case!(
    origin_final_byte_edge,
    "alpha(from rgb(.1234567 0 0))",
    "alpha(from rgb(0.123457 0 0))"
);
byte_case!(
    rgb_scaled_final_byte_edge,
    "rgb(from red .0000002% g b)",
    "rgb(from red 0.000001 g b)"
);
byte_case!(
    custom_scaled_final_byte_edge,
    "color(from red --P 12.34567%)",
    "color(from red --P 0.123457)"
);
byte_case!(
    sibling_rounded_literals_share_final_bytes,
    "color(from red --P .1234567 .1234567)",
    "color(from red --P 0.123457 0.123457)"
);
byte_case!(
    huge_tiny_identity_needs_only_zero_bytes,
    "rgb(from red 1e-1000000000 g b)",
    "rgb(from red 0 g b)"
);
byte_case!(
    huge_tiny_rational_needs_only_zero_bytes,
    "rgb(from red 1e-1000000000% g b)",
    "rgb(from red 0 g b)"
);
byte_case!(
    huge_exponent_genuine_zero,
    "color(from red --P 0e1000000000)",
    "color(from red --P 0)"
);

#[test]
fn huge_positive_output_is_bounded_and_atomic() {
    let value = color("rgb(from red 1e1000000000% g b)");
    let before = value.clone();
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(10000, 10000, 128))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(value, before);
}

// Two color roots and three channel leaves cost five input visits. References
// and none each emit one projection; there are no numeric arithmetic limbs.
#[test]
fn symbolic_routes_have_independent_exact_cumulative_counts() {
    let value = color("rgb(from red none g b)");
    let before = value.clone();
    let text = "rgb(from red none g b)";
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(5, 5, text.len()))
            .unwrap(),
        text
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(4, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, 4, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(value, before);
}

#[test]
fn nested_roots_and_references_share_cumulative_counts() {
    // Three color roots, six reference/none leaves: nine visits in both budgets.
    let value = color("rgb(from rgb(from red none g b) r g b)");
    let text = "rgb(from rgb(from red none g b) r g b)";
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(9, 9, text.len()))
            .unwrap(),
        text
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(8, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(usize::MAX, 8, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
}

#[test]
fn literal_input_visits_are_charged_once() {
    // Two roots + two literal siblings + one reference = five input nodes.
    let value = color("rgb(from red .1234567 .1234567 b)");
    let before = value.clone();
    let text = "rgb(from red 0.123457 0.123457 b)";
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(4, usize::MAX, usize::MAX))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(Limits::new(5, usize::MAX, text.len()))
            .unwrap(),
        text
    );
    assert_eq!(value, before);
}

macro_rules! zero_limit_case {
    ($name:ident, $limits:expr, $kind:expr) => {
        #[test]
        fn $name() {
            let value = color("rgb(from red .1234567 g b)");
            let before = value.clone();
            assert_eq!(
                value
                    .to_specified_css_with_limits($limits)
                    .unwrap_err()
                    .kind(),
                $kind
            );
            assert_eq!(value, before);
        }
    };
}
zero_limit_case!(
    zero_input_budget,
    Limits::new(0, usize::MAX, usize::MAX),
    Kind::InputNodeLimit
);
zero_limit_case!(
    zero_projection_budget,
    Limits::new(usize::MAX, 0, usize::MAX),
    Kind::ProjectionNodeLimit
);
zero_limit_case!(
    zero_byte_budget,
    Limits::new(usize::MAX, usize::MAX, 0),
    Kind::ByteLimit
);

#[test]
fn checked_programmatic_graph_retains_literal_kind_spelling_and_origin() {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain as D;
    let raw =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("-.0078125").unwrap()])
            .unwrap();
    let expression =
        CssRelativeColorExpression::try_from_components(raw.clone(), E::Rgb, D::NumberPercentage)
            .unwrap();
    assert_eq!(expression.origin(), &CssValueOrigin::Programmatic);
    let CssRelativeColorExpressionValue::Number(literal) = expression.value() else {
        panic!("number")
    };
    assert_eq!(literal.numeric().representation(), "-.0078125");
    let channel = |name| {
        CssRelativeColorExpression::try_from_components(
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(name).unwrap()]).unwrap(),
            E::Rgb,
            D::NumberPercentage,
        )
        .unwrap()
    };
    let source = CssColor::from_named(CssNamedColor::try_new("red").unwrap());
    let relative = CssRelativeColor::try_new(
        CssRelativeColorFunction::Rgb,
        source.clone(),
        [expression.clone(), channel("g"), channel("b")],
        None,
    )
    .unwrap();
    assert_eq!(relative.source(), &source);
    assert_eq!(relative.channels()[0], expression);
    let value = CssColor::from_relative(relative);
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(value, before);
    assert_eq!(raw.items()[0].origin(), &CssValueOrigin::Programmatic);
    assert_eq!(result.unwrap(), "rgb(from red -0.007813 g b)");
}

#[test]
fn checked_programmatic_origin_retains_number_and_percentage_categories() {
    let number = CssColorNumberLiteral::try_from_component(
        CssComponentValue::try_number(".1234567").unwrap(),
    )
    .unwrap();
    // A checked Rust graph can retain a parsed percentage leaf alongside a
    // programmatic number; neither constructor invents a new source origin.
    let raw = components("120.1234567%");
    let percentage = CssColorPercentageLiteral::try_from_component(raw.items()[0].clone()).unwrap();
    assert_eq!(percentage.numeric().representation(), "120.1234567");
    assert_eq!(percentage.origin(), raw.items()[0].origin());
    let rgb = CssRgbColor::try_new(
        CssColorSyntax::Modern,
        [
            CssColorComponent::Number(number.clone()),
            CssColorComponent::Percentage(percentage.clone()),
            CssColorComponent::None,
        ],
        None,
    )
    .unwrap();
    let source = CssColor::from_rgb(rgb);
    let value = CssColor::from_alpha(CssAlphaColor::try_new(source.clone(), None).unwrap());
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(value, before);
    assert_eq!(
        source.rgb_value().unwrap().channels()[0],
        CssColorComponent::Number(number)
    );
    assert_eq!(
        source.rgb_value().unwrap().channels()[1],
        CssColorComponent::Percentage(percentage)
    );
    assert_eq!(
        result.unwrap(),
        "alpha(from rgb(0.123457 120.123457% none))"
    );
}

#[test]
fn declarations_validate_and_keep_raw_graph_importance_and_siblings() {
    let source = "color: rgb(from red .1234567 g b) !important; opacity: .5";
    let validated = validate_style_attribute(source).unwrap();
    let report = parse_style_attribute(source);
    assert_eq!(&validated, report.syntax());
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let before = report.syntax().clone();
    let d = &report.syntax()[0];
    assert_eq!(d.importance(), CssImportance::Important);
    assert_eq!(
        d.value_components().serialize().unwrap().as_css(),
        " rgb(from red .1234567 g b) "
    );
    let result = declaration_color(d).to_specified_css();
    assert_eq!(report.syntax(), &before);
    assert_eq!(result.unwrap(), "rgb(from red 0.123457 g b)");
}

#[test]
fn hue_percentages_stay_invalid_and_recovery_retains_sibling() {
    let report = parse_style_attribute("color: hsl(from red 20% s l); opacity: .5");
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}

#[test]
fn substitutions_remain_pending_and_checked_construction_reports_origin() {
    let source = "rgb(from red var(--n) g b)";
    let d = declaration(source);
    assert!(d.known().unwrap().property_value().is_none());
    assert_eq!(
        d.known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        source
    );
    assert!(matches!(
        expand_declaration(&d).unwrap(),
        CssExpansion::Pending(_)
    ));
    let raw = components("var(--n)");
    let before = raw.clone();
    let error = CssRelativeColorExpression::try_from_components(
        raw,
        CssRelativeColorEnvironment::Rgb,
        CssRelativeColorResultDomain::NumberPercentage,
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNumericConstructionErrorKind::SubstitutionRequired
    );
    assert!(error.origin().is_some());
    assert_eq!(before, components("var(--n)"));
}
