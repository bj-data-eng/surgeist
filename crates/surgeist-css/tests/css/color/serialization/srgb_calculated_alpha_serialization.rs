#![forbid(unsafe_code)]
//! Ordinary RGB/HSL/HWB alpha follows the accepted Color 4 §15.1 phase
//! disposition: finalize a noncontextual scalar independently of its siblings.
//! Color 4 §16.1.2 and Color 5 Origin retention still govern protected roles.
//! Binary64 values are clamped before six-place, positive-tie formatting;
//! exact unity is tested before rounding. Graph and scratch work precede omission.

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
        panic!("checked color property")
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
    // Reentry asserts admission only. A rounded explicit unity is a direct
    // literal on reentry, and therefore need not be a serialization fixed point.
    let _ = declaration(expected);
}

fn assert_text(source: &str, expected: &str) {
    let checked = declaration(source);
    assert_eq!(checked.value_components(), &components(source));
    let report = parse_style_attribute(&format!("color:{source}!important"));
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    for value in [&checked, &report.syntax()[0]] {
        let before = value.clone();
        assert_eq!(value.importance(), CssImportance::Important);
        assert_eq!(
            value.value_components().serialize().unwrap().as_css(),
            source
        );
        assert!(matches!(
            value.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
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

text_case!(rgb_scalar_unity, "rgb(1 2 3 / calc(1))", "rgb(1, 2, 3)");
text_case!(rgb_above_unity, "rgb(1 2 3 / calc(2))", "rgb(1, 2, 3)");
text_case!(rgb_below_zero, "rgb(1 2 3 / calc(-1))", "rgba(1, 2, 3, 0)");
text_case!(
    rgb_scalar_interior,
    "rgb(1 2 3 / calc(.5))",
    "rgba(1, 2, 3, 0.5)"
);
text_case!(
    rgb_percentage_unity,
    "rgb(1 2 3 / calc(100%))",
    "rgb(1, 2, 3)"
);
text_case!(rgb_scalar_zero, "rgb(1 2 3 / calc(0))", "rgba(1, 2, 3, 0)");
text_case!(
    rgb_percentage_zero,
    "rgb(1 2 3 / calc(0%))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    hsl_scalar_zero,
    "hsl(0 100% 50% / calc(0))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hsl_scalar_above_unity,
    "hsl(0 100% 50% / calc(2))",
    "rgb(255, 0, 0)"
);
text_case!(
    hsl_scalar_interior,
    "hsl(0 100% 50% / calc(.5))",
    "rgba(255, 0, 0, 0.5)"
);
text_case!(
    hsl_percentage_zero,
    "hsl(0 100% 50% / calc(0%))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hsl_percentage_negative,
    "hsl(0 100% 50% / calc(-50%))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hsl_percentage_unity,
    "hsl(0 100% 50% / calc(100%))",
    "rgb(255, 0, 0)"
);
text_case!(
    hsl_percentage_above_unity,
    "hsl(0 100% 50% / calc(120%))",
    "rgb(255, 0, 0)"
);
text_case!(
    hwb_scalar_zero,
    "hwb(0 0% 0% / calc(0))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hwb_scalar_negative,
    "hwb(0 0% 0% / calc(-1))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hwb_exact_scalar_unity,
    "hwb(0 0% 0% / calc(1))",
    "rgb(255, 0, 0)"
);
text_case!(
    hwb_scalar_interior,
    "hwb(0 0% 0% / calc(.5))",
    "rgba(255, 0, 0, 0.5)"
);
text_case!(
    hwb_percentage_zero,
    "hwb(0 0% 0% / calc(0%))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hwb_percentage_unity,
    "hwb(0 0% 0% / calc(100%))",
    "rgb(255, 0, 0)"
);
text_case!(
    hwb_percentage_above_unity,
    "hwb(0 0% 0% / calc(120%))",
    "rgb(255, 0, 0)"
);
text_case!(
    rgb_percentage_above_unity,
    "rgb(1 2 3 / calc(2 * 60%))",
    "rgb(1, 2, 3)"
);
text_case!(
    rgb_percentage_below_zero,
    "rgb(1 2 3 / calc(-120%))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    rgba_modern_interior,
    "rgba(1 2 3 / calc(.5))",
    "rgba(1, 2, 3, 0.5)"
);
text_case!(
    rgba_legacy_interior,
    "rgba(1, 2, 3, calc(50%))",
    "rgba(1, 2, 3, 0.5)"
);
text_case!(
    hsl_scalar_unity,
    "hsl(0 100% 50% / calc(1))",
    "rgb(255, 0, 0)"
);
text_case!(
    hsl_scalar_negative,
    "hsl(0 100% 50% / calc(-1))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hsl_percentage_interior,
    "hsl(0 100% 50% / calc(50%))",
    "rgba(255, 0, 0, 0.5)"
);
text_case!(
    hsla_modern_unity,
    "hsla(0 100% 50% / calc(2))",
    "rgb(255, 0, 0)"
);
text_case!(
    hsla_legacy_interior,
    "hsla(0, 100%, 50%, calc(.5))",
    "rgba(255, 0, 0, 0.5)"
);
text_case!(hwb_scalar_unity, "hwb(0 0% 0% / calc(2))", "rgb(255, 0, 0)");
text_case!(
    hwb_percentage_negative,
    "hwb(0 0% 0% / calc(-50%))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    hwb_percentage_interior,
    "hwb(0 0% 0% / calc(50%))",
    "rgba(255, 0, 0, 0.5)"
);

// 1/128 is exact binary64. Its scaled micro-unit value is 7812.5;
// the decimal neighbors below straddle that value without a decimal tie.
text_case!(
    scalar_positive_tie,
    "rgb(1 2 3 / calc(1 / 128))",
    "rgba(1, 2, 3, 0.007813)"
);
text_case!(
    scalar_below_tie,
    "rgb(1 2 3 / calc(.007812499999999999))",
    "rgba(1, 2, 3, 0.007812)"
);
text_case!(
    scalar_above_tie,
    "rgb(1 2 3 / calc(.007812500000000002))",
    "rgba(1, 2, 3, 0.007813)"
);
text_case!(
    scaled_percentage_tie,
    "hsl(0 100% 50% / calc(.78125%))",
    "rgba(255, 0, 0, 0.007813)"
);
text_case!(
    negative_tie_clamps_first,
    "hwb(0 0% 0% / calc(-1 / 128))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    rounded_unity_stays_explicit,
    "rgb(1 2 3 / calc(.9999996))",
    "rgba(1, 2, 3, 1)"
);
text_case!(
    scaled_rounded_unity_stays_explicit,
    "hsl(0 100% 50% / calc(99.99996%))",
    "rgba(255, 0, 0, 1)"
);
text_case!(
    negative_zero_normalizes,
    "rgb(1 2 3 / calc(0 * -1))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    positive_subnormal_rounds_zero,
    "rgb(1 2 3 / calc(5e-324))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    negative_subnormal_clamps_zero,
    "rgb(1 2 3 / calc(-5e-324))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    binary_micro_below_tie,
    "rgb(1 2 3 / calc(5e-7))",
    "rgba(1, 2, 3, 0)"
);
text_case!(
    positive_infinity_omits,
    "rgb(1 2 3 / calc(infinity))",
    "rgb(1, 2, 3)"
);
text_case!(
    negative_infinity_clamps_zero,
    "hsl(0 100% 50% / calc(-infinity))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    nan_normalizes_zero,
    "hwb(0 0% 0% / calc(NaN))",
    "rgba(255, 0, 0, 0)"
);
text_case!(
    percentage_infinity_omits,
    "rgb(1 2 3 / calc(infinity * 1%))",
    "rgb(1, 2, 3)"
);
text_case!(
    nan_poisoned_context_becomes_scalar_zero,
    "rgb(1 2 3 / calc(NaN + 1em / 1px))",
    "rgba(1, 2, 3, 0)"
);

text_case!(
    missing_rgb_unity,
    "rgb(none none none / calc(2))",
    "color(srgb none none none)"
);
text_case!(
    missing_hsl_unity,
    "hsl(none 100% 50% / calc(2))",
    "hsl(none 100% 50%)"
);
text_case!(
    missing_hwb_zero,
    "hwb(none 20% 30% / calc(-1))",
    "hwb(none 20% 30% / 0)"
);
text_case!(
    contextual_rgb_sibling_unity,
    "rgb(calc(1em / 1px) 2 3 / calc(2))",
    "rgb(calc(1 * 1em / 1px) 2 3)"
);
text_case!(
    contextual_hsl_sibling_unity,
    "hsl(calc(1em / 1px) 100% 50% / calc(2))",
    "hsl(calc(1em / 1px) 100% 50%)"
);
text_case!(
    contextual_hwb_sibling_zero,
    "hwb(calc(1em / 1px) 20% 30% / calc(-1))",
    "hwb(calc(1em / 1px) 20% 30% / 0)"
);
text_case!(
    nested_mix_finalizes_descendants,
    "color-mix(color-mix(rgb(1 2 3 / calc(2)), blue), hwb(0 0% 0% / calc(.5)))",
    "color-mix(color-mix(rgb(1, 2, 3), blue), rgba(255, 0, 0, 0.5))"
);
text_case!(
    mix_as_origin_finalizes_descendants,
    "alpha(from color-mix(rgb(1 2 3 / calc(2)), blue))",
    "alpha(from color-mix(rgb(1, 2, 3), blue))"
);

text_case!(
    protected_rgb_origin,
    "alpha(from rgb(1 2 3 / calc(2)))",
    "alpha(from rgb(1 2 3 / calc(2)))"
);
text_case!(
    protected_hsl_origin_percentage,
    "alpha(from hsl(0 100% 50% / calc(.78125%)))",
    "alpha(from hsl(0 100% 50% / calc(0.78125%)))"
);
text_case!(
    protected_hwb_origin,
    "alpha(from hwb(0 0% 0% / calc(-1)))",
    "alpha(from hwb(0 0% 0% / calc(-1)))"
);
text_case!(
    protected_relative_literal_unity,
    "rgb(from red r g b / 1)",
    "rgb(from red r g b / 1)"
);
text_case!(
    protected_relative_calculated_alpha,
    "rgb(from red r g b / calc(2))",
    "rgb(from red r g b / calc(2))"
);
text_case!(
    protected_relative_symbolic_alpha,
    "rgb(from red r g b / calc(alpha + 1 / 128))",
    "rgb(from red r g b / calc(0.007813 + alpha))"
);
text_case!(
    protected_alpha_override,
    "alpha(from red / calc(2))",
    "alpha(from red / calc(2))"
);
text_case!(
    protected_custom_alpha,
    "color(--P 0 / calc(2 * 60%))",
    "color(--P 0 / calc(1.2))"
);
text_case!(
    protected_lab_alpha,
    "lab(20 0 0 / calc(2))",
    "lab(20 0 0 / calc(2))"
);
text_case!(
    protected_lch_alpha,
    "lch(20 30 0 / calc(-1))",
    "lch(20 30 0 / calc(-1))"
);
text_case!(
    protected_oklab_alpha,
    "oklab(.2 0 0 / calc(2))",
    "oklab(0.2 0 0 / calc(2))"
);
text_case!(
    protected_oklch_alpha,
    "oklch(.2 .3 0 / calc(-1))",
    "oklch(0.2 0.3 0 / calc(-1))"
);
text_case!(
    protected_predefined_alpha,
    "color(display-p3 0 0 0 / calc(2))",
    "color(display-p3 0 0 0 / calc(2))"
);
text_case!(
    protected_predefined_srgb_alpha,
    "color(srgb 0 0 0 / calc(2))",
    "color(srgb 0 0 0 / calc(2))"
);
text_case!(
    protected_weight_with_finalized_alpha,
    "color-mix(rgb(1 2 3 / calc(2)) calc(120%), blue)",
    "color-mix(rgb(1, 2, 3) calc(120%), blue)"
);
text_case!(
    protected_contextual_rgb_alpha,
    "rgb(1 2 3 / calc(.0078125 * 1em / 1px))",
    "rgba(1, 2, 3, calc(0.007813 * 1em / 1px))"
);
text_case!(
    protected_contextual_hsl_percentage_alpha,
    "hsl(0 100% 50% / calc(.0078125% * 1em / 1px))",
    "rgba(255, 0, 0, calc(0.007813% * 1em / 1px / 100%))"
);
text_case!(
    protected_contextual_hwb_nonfinite_alpha,
    "hwb(0 0% 0% / calc(infinity + 1em / 1px))",
    "rgba(255, 0, 0, calc(infinity + (1em / 1px)))"
);
text_case!(protected_absent_alpha, "rgb(1 2 3)", "rgb(1, 2, 3)");
text_case!(
    protected_none_alpha,
    "rgb(1 2 3 / none)",
    "color(srgb 0.003922 0.007843 0.011765 / none)"
);
text_case!(protected_direct_unity, "rgb(1 2 3 / 1)", "rgb(1, 2, 3)");

fn number(source: &str, programmatic: bool) -> CssColorComponent {
    let token = if programmatic {
        CssComponentValue::try_number(source).unwrap()
    } else {
        components(source).items()[0].clone()
    };
    CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
}

fn checked_color(family: &str, programmatic: bool) -> CssColor {
    let values = if programmatic {
        let arguments =
            CssComponentValues::try_new(vec![CssComponentValue::try_number("2").unwrap()]).unwrap();
        CssComponentValues::try_new(vec![
            CssComponentValue::try_function("calc", arguments).unwrap(),
        ])
        .unwrap()
    } else {
        components("calc(2)")
    };
    assert_eq!(
        values.items()[0].origin() == &CssValueOrigin::Programmatic,
        programmatic
    );
    let calculation = CssNumberCalculation::try_from_components(values.clone()).unwrap();
    assert_eq!(calculation.components(), &values);
    let alpha = Some(CssColorComponent::NumberCalculation(calculation));
    let CssColorComponent::Number(hue) = number("0", programmatic) else {
        unreachable!()
    };
    match family {
        "rgb" => CssColor::from_rgb(
            CssRgbColor::try_new(
                CssColorSyntax::Modern,
                [
                    number("255", programmatic),
                    number("0", programmatic),
                    number("0", programmatic),
                ],
                alpha,
            )
            .unwrap(),
        ),
        "hsl" => CssColor::from_hsl(
            CssHslColor::try_new(
                CssColorSyntax::Modern,
                CssColorHue::Number(hue),
                number("100", programmatic),
                number("50", programmatic),
                alpha,
            )
            .unwrap(),
        ),
        "hwb" => CssColor::from_hwb(
            CssHwbColor::try_new(
                CssColorHue::Number(hue),
                number("0", programmatic),
                number("0", programmatic),
                alpha,
            )
            .unwrap(),
        ),
        _ => unreachable!(),
    }
}

macro_rules! constructor_case {
    ($name:ident, $family:literal, $programmatic:literal) => {
        #[test]
        fn $name() {
            assert_value(&checked_color($family, $programmatic), "rgb(255, 0, 0)");
        }
    };
}
constructor_case!(checked_rgb_parsed_graph_retained, "rgb", false);
constructor_case!(checked_hsl_parsed_graph_retained, "hsl", false);
constructor_case!(checked_hwb_parsed_graph_retained, "hwb", false);
constructor_case!(checked_rgb_programmatic_graph_retained, "rgb", true);
constructor_case!(checked_hsl_programmatic_graph_retained, "hsl", true);
constructor_case!(checked_hwb_programmatic_graph_retained, "hwb", true);

fn assert_error(value: &CssColor, limits: Limits, expected: Kind) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    assert_eq!(result.unwrap_err().kind(), expected, "{limits:?}");
}

fn assert_limits(source: &str, expected: &str, inputs: usize, projections: usize, bytes: usize) {
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
        (Limits::new(inputs, projections, bytes - 1), Kind::ByteLimit),
    ] {
        assert_error(&value, limits, kind);
    }
    let limits = Limits::new(inputs, projections, bytes);
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn missing_rgb_scalar_tie_exact_cumulative_limits() {
    // Root plus three missing channels = 4/4. Wrapper/product/two leaves
    // = 4 input; leaves/inverse/resolved product = 4 projection. 37 UTF-8 bytes.
    assert_limits(
        "rgb(none none none / calc(1 / 128))",
        "color(srgb none none none / 0.007813)",
        8,
        8,
        37,
    );
}

#[test]
fn missing_rgb_percentage_tie_charges_scale_nodes() {
    // 4/4 base + 2/1 calc + 0/3 percentage conversion = 6/8.
    assert_limits(
        "rgb(none none none / calc(.78125%))",
        "color(srgb none none none / 0.007813)",
        6,
        8,
        37,
    );
}

#[test]
fn missing_rgb_omission_still_charges_capture() {
    // 4/4 base + wrapper/scalar 2/1, final text 26 bytes.
    assert_limits(
        "rgb(none none none / calc(2))",
        "color(srgb none none none)",
        6,
        5,
        26,
    );
}

#[test]
fn long_opaque_alpha_requires_scratch_before_omission() {
    // calc( + 21 integer digits + ) = 27 scratch bytes. The resulting opaque
    // color is 26 bytes; 26 must fail despite fitting that final output.
    assert_limits(
        "rgb(none none none / calc(100000000000000000000))",
        "color(srgb none none none)",
        6,
        5,
        27,
    );
}

#[test]
fn nested_mix_prefixes_share_input_projection_and_final_bytes() {
    // Two Mix roots + named siblings = 4/4, child 8/8, two calculated
    // weights 4/2. Prefixes consume 20 bytes before the child's capture.
    let expected = "color-mix(color-mix(color(srgb none none none / 0.007813) calc(50%), blue) calc(50%), red)";
    assert_limits(
        "color-mix(color-mix(rgb(none none none / calc(1 / 128)) calc(50%), blue) calc(50%), red)",
        expected,
        16,
        14,
        expected.len(),
    );
}

fn assert_pair_limits(
    pair: &CssBorderColorPair,
    expected: &str,
    inputs: usize,
    projections: usize,
    bytes: usize,
) {
    let before = pair.clone();
    for (limits, kind) in [
        (
            Limits::new(inputs - 1, usize::MAX, usize::MAX),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(usize::MAX, projections - 1, usize::MAX),
            Kind::ProjectionNodeLimit,
        ),
        (Limits::new(inputs, projections, bytes - 1), Kind::ByteLimit),
    ] {
        let result = pair.serialize_specified_with_limits(limits);
        assert_eq!(result, pair.serialize_specified_with_limits(limits));
        assert_eq!(pair, &before);
        assert_eq!(result.unwrap_err().kind(), kind);
    }
    let limits = Limits::new(inputs, projections, bytes);
    let result = pair.serialize_specified_with_limits(limits);
    assert_eq!(result, pair.serialize_specified_with_limits(limits));
    assert_eq!(pair, &before);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn sibling_calculations_share_all_three_limits() {
    // Pair root 1/1 + two child 8/8 graphs = 17/17; separator is one byte.
    let child = color("rgb(none none none / calc(1 / 128))");
    let pair = CssBorderColorPair::new(child.clone(), Some(child));
    let expected = "color(srgb none none none / 0.007813) color(srgb none none none / 0.007813)";
    assert_pair_limits(&pair, expected, 17, 17, expected.len());
}

#[test]
fn utf8_prefix_spends_bytes_before_long_opaque_capture() {
    // Pair 1/1 + custom color root/missing channel 2/2 + RGB 6/5 = 9/8.
    // UTF-8 prefix color(--é none) plus separator is 17 bytes. The
    // remaining scratch needs 27, one more than the child's final 26.
    let pair = CssBorderColorPair::new(
        color("color(--é none)"),
        Some(color("rgb(none none none / calc(100000000000000000000))")),
    );
    let expected = "color(--é none) color(srgb none none none)";
    assert_pair_limits(&pair, expected, 9, 8, expected.len() + 1);
}

#[test]
fn mix_prefix_spends_bytes_before_long_opaque_capture() {
    // Mix root + named sibling 2/2 + child 6/5 + weight 2/1 = 10/8. A 10-byte prefix
    // leaves 27 scratch bytes at limit 37, versus 26 at limit 36. Final
    // output is 53 bytes, so final output is the stronger overall bound.
    let value =
        color("color-mix(rgb(none none none / calc(100000000000000000000)) calc(50%), blue)");
    assert_error(&value, Limits::new(10, 8, 36), Kind::ByteLimit);
    assert_limits(
        "color-mix(rgb(none none none / calc(100000000000000000000)) calc(50%), blue)",
        "color-mix(color(srgb none none none) calc(50%), blue)",
        10,
        8,
        53,
    );
}

#[test]
fn zero_limits_preserve_root_precedence() {
    let value = color("rgb(none none none / calc(2))");
    assert_error(&value, Limits::new(0, 0, 0), Kind::InputNodeLimit);
    assert_error(&value, Limits::new(1, 0, 0), Kind::ProjectionNodeLimit);
}
