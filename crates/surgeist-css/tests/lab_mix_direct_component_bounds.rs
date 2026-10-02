#![forbid(unsafe_code)]
//! Color4 CRD 2026-09-08 §§9.3/9.4 direct parsed L/C domains, applied to
//! individual ordinary Mix colors by Color5 WD 2026-09-08 §11.1. Actual
//! Origin/relative and Values4 WD 2024-03-12 calculated slots retain their
//! distinct roles. Expectations are exact domain/scale and resource oracles,
//! not browser output or values copied from the implementation under test.

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

fn assert_text(source: &str, expected: &str) {
    let checked = declaration(source);
    let source_attribute = format!("color:{source}!important;opacity:.5");
    let report = parse_style_attribute(&source_attribute);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let parsed = &report.syntax()[0];
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(checked.value_components(), &components(source));
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        source
    );
    let CssValueOrigin::Parsed(origin) = parsed.value_components().items()[0].origin() else {
        panic!("real parsed source origin")
    };
    assert_eq!(origin.source().as_str(), source_attribute);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    // A function component's public origin is its opening token, including
    // the authored name and '('; its child and closing tokens own their spans.
    assert_eq!(
        origin.span().end().byte_offset().value(),
        6 + source.find('(').unwrap() + 1
    );
    let checked_before = checked.clone();
    let parsed_before = parsed.clone();
    // Execute both actual entry routes before asserting either output, so RED
    // in one route does not prevent the other route's writer from executing.
    let checked_result = declared_color(&checked).to_specified_css();
    let parsed_result = declared_color(parsed).to_specified_css();
    assert_eq!(checked_result, declared_color(&checked).to_specified_css());
    assert_eq!(parsed_result, declared_color(parsed).to_specified_css());
    assert_eq!(checked, checked_before);
    assert_eq!(parsed, &parsed_before);
    assert_eq!(checked_result.unwrap(), expected, "{source}: checked entry");
    assert_eq!(parsed_result.unwrap(), expected, "{source}: parsed entry");
    // Reentry complements the explicit independent literal oracle.
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

// Both wrapper and expected child are independent literal syntax contracts.
macro_rules! mix_text_case {
    ($name:ident, $source:literal, $expected:literal) => {
        #[test]
        fn $name() {
            assert_text(
                concat!("color-mix(", $source, ", blue)"),
                concat!("color-mix(", $expected, ", blue)"),
            );
        }
    };
}

mix_text_case!(mix_lab_exact_zero, "lab(0 0 0)", "lab(0 0 0)");
mix_text_case!(mix_lab_percentage_exact_zero, "lab(0% 0 0)", "lab(0 0 0)");
mix_text_case!(mix_lch_percentage_exact_zero, "lch(0% 0% 0)", "lch(0 0 0)");
mix_text_case!(mix_oklab_number_exact_zero, "oklab(0 0 0)", "oklab(0 0 0)");
mix_text_case!(mix_oklch_number_exact_zero, "oklch(0 0 0)", "oklch(0 0 0)");
mix_text_case!(
    mix_lab_equal_upper_percentage,
    "lab(100% 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(mix_lch_equal_upper_number, "lch(100 0 0)", "lch(100 0 0)");
mix_text_case!(mix_oklch_equal_upper_number, "oklch(1 0 0)", "oklch(1 0 0)");
mix_text_case!(
    mix_lch_inrange_upper_neighbor_is_exact,
    "lch(99.999999999999999999 0 0)",
    "lch(99.999999999999999999 0 0)"
);
mix_text_case!(
    mix_oklch_inrange_upper_neighbor_is_exact,
    "oklch(.999999999999999999 0 0)",
    "oklch(0.999999999999999999 0 0)"
);
mix_text_case!(
    mix_lab_negative_binary_overflow_lightness_clips,
    "lab(-1e400 0 0)",
    "lab(0 0 0)"
);
mix_text_case!(
    mix_lch_negative_binary_overflow_lightness_clips,
    "lch(-1e400 0 30)",
    "lch(0 0 30)"
);
mix_text_case!(
    mix_oklab_negative_binary_overflow_lightness_clips,
    "oklab(-1e400 0 0)",
    "oklab(0 0 0)"
);
mix_text_case!(
    mix_oklch_negative_binary_overflow_lightness_clips,
    "oklch(-1e400 0 30)",
    "oklch(0 0 30)"
);
mix_text_case!(mix_lch_exact_zero, "lch(0 0 0)", "lch(0 0 0)");
mix_text_case!(mix_oklab_exact_zero, "oklab(0% 0 0)", "oklab(0 0 0)");
mix_text_case!(mix_oklch_exact_zero, "oklch(0% 0% 0)", "oklch(0 0 0)");
mix_text_case!(mix_lab_equal_upper_number, "lab(100 0 0)", "lab(100 0 0)");
mix_text_case!(
    mix_lch_equal_upper_percentage,
    "lch(100% 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(mix_oklab_equal_upper_number, "oklab(1 0 0)", "oklab(1 0 0)");
mix_text_case!(
    mix_oklch_equal_upper_percentage,
    "oklch(100% 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_lch_signed_zero_lightness,
    "lch(-0% -0 30)",
    "lch(0 0 30)"
);
mix_text_case!(
    mix_oklab_signed_zero_lightness,
    "oklab(-0% 0 0)",
    "oklab(0 0 0)"
);
mix_text_case!(
    mix_oklch_signed_zero_components,
    "oklch(-0 -0% 30)",
    "oklch(0 0 30)"
);
mix_text_case!(
    mix_calculated_axis_does_not_disable_direct_lightness,
    "lab(125 calc(0) 0)",
    "lab(100 calc(0) 0)"
);
mix_text_case!(
    mix_calculated_percentage_lightness_does_not_disable_direct_chroma,
    "oklch(calc(120%) -20% 30)",
    "oklch(calc(1.2) 0 30)"
);
text_case!(
    nested_mix_normalizes_each_ordinary_child,
    "color-mix(color-mix(lab(125 0 0), blue), red)",
    "color-mix(color-mix(lab(100 0 0), blue), red)"
);
text_case!(
    origin_mix_still_serializes_individual_ordinary_children,
    "alpha(from color-mix(lab(125 0 0), blue))",
    "alpha(from color-mix(lab(100 0 0), blue))"
);
text_case!(
    relative_color_mix_origin_normalizes_only_mix_arguments,
    "lab(from color-mix(lab(125% -20% 30%), blue) 125 a b)",
    "lab(from color-mix(lab(100 -25 37.5), blue) 125 a b)"
);
text_case!(
    mix_relative_child_preserves_its_true_origin_and_declared_channels,
    "color-mix(lab(from lab(125% -20% 30%) 125 a b), lab(125 0 0))",
    "color-mix(lab(from lab(125% -20% 30%) 125 a b), lab(100 0 0))"
);
text_case!(
    mix_alpha_child_preserves_true_origin_beside_ordinary_child,
    "color-mix(alpha(from oklch(-.2 -20% 30)), oklch(120% -20% 30))",
    "color-mix(alpha(from oklch(-0.2 -20% 30)), oklch(1 0 30))"
);
text_case!(
    mix_interpolation_and_literal_weights_survive_child_bounds,
    "color-mix(in lch longer hue, lch(125 -20% 30) 30%, blue 70%)",
    "color-mix(in lch longer hue, lch(100 0 30) 30%, blue 70%)"
);
text_case!(
    default_explicit_interpolation_does_not_disable_child_bounds,
    "color-mix(in oklab, oklab(120% 0 0), blue)",
    "color-mix(oklab(1 0 0), blue)"
);
text_case!(
    mix_retained_weight_does_not_disable_direct_bounds,
    "color-mix(lab(125 0 0) calc(25% + 10%), blue 65%)",
    "color-mix(lab(100 0 0) calc(35%), blue 65%)"
);
text_case!(
    mix_contextual_weight_does_not_disable_direct_bounds,
    "color-mix(lab(125 0 0) calc(50% * 1em / 1px), blue)",
    "color-mix(lab(100 0 0) calc(50% * 1em / 1px), blue)"
);
text_case!(
    identical_mix_children_remain_two_ordered_arguments,
    "color-mix(lab(125 0 0), lab(125 0 0))",
    "color-mix(lab(100 0 0), lab(100 0 0))"
);
text_case!(
    three_mix_arguments_keep_order_and_each_child_domain,
    "color-mix(lab(125 0 0), lch(-20 -20 30), oklab(120% 0 0))",
    "color-mix(lab(100 0 0), lch(0 0 30), oklab(1 0 0))"
);

mix_text_case!(mix_lab_number_upper_clips, "lab(125 0 0)", "lab(100 0 0)");
mix_text_case!(
    mix_lab_percentage_upper_clips,
    "lab(125% 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(mix_lab_number_lower_clips, "lab(-20 0 0)", "lab(0 0 0)");
mix_text_case!(
    mix_lab_percentage_lower_clips,
    "lab(-20% 0 0)",
    "lab(0 0 0)"
);
mix_text_case!(mix_lch_number_upper_clips, "lch(125 0 0)", "lch(100 0 0)");
mix_text_case!(
    mix_lch_percentage_upper_clips,
    "lch(125% 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(mix_lch_number_lower_clips, "lch(-20 0 0)", "lch(0 0 0)");
mix_text_case!(
    mix_lch_percentage_lower_clips,
    "lch(-20% 0 0)",
    "lch(0 0 0)"
);
mix_text_case!(
    mix_oklab_number_upper_clips,
    "oklab(1.2 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklab_percentage_upper_clips,
    "oklab(120% 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklab_number_lower_clips,
    "oklab(-0.2 0 0)",
    "oklab(0 0 0)"
);
mix_text_case!(
    mix_oklab_percentage_lower_clips,
    "oklab(-20% 0 0)",
    "oklab(0 0 0)"
);
mix_text_case!(
    mix_oklch_number_upper_clips,
    "oklch(1.2 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_oklch_percentage_upper_clips,
    "oklch(120% 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_oklch_number_lower_clips,
    "oklch(-0.2 0 0)",
    "oklch(0 0 0)"
);
mix_text_case!(
    mix_oklch_percentage_lower_clips,
    "oklch(-20% 0 0)",
    "oklch(0 0 0)"
);
mix_text_case!(
    mix_lch_number_chroma_lower_clips,
    "lch(50 -20 30)",
    "lch(50 0 30)"
);
mix_text_case!(
    mix_lch_percentage_chroma_lower_clips,
    "lch(50 -20% 30)",
    "lch(50 0 30)"
);
mix_text_case!(
    mix_oklch_number_chroma_lower_clips,
    "oklch(.5 -.2 30)",
    "oklch(0.5 0 30)"
);
mix_text_case!(
    mix_oklch_percentage_chroma_lower_clips,
    "oklch(50% -20% 30)",
    "oklch(0.5 0 30)"
);
mix_text_case!(
    mix_lab_lower_neighbor_is_exact,
    "lab(1e-20 0 0)",
    "lab(0.00000000000000000001 0 0)"
);
mix_text_case!(
    mix_lab_upper_neighbor_is_exact,
    "lab(99.999999999999999999999 0 0)",
    "lab(99.999999999999999999999 0 0)"
);
mix_text_case!(
    mix_lab_upper_number_neighbor_clips,
    "lab(100.000000000000000000001 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lab_upper_percentage_neighbor_clips,
    "lab(100.000000000000000000001% 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lch_upper_number_neighbor_clips,
    "lch(100.000000000000000000001 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(
    mix_lch_upper_percentage_neighbor_clips,
    "lch(100.000000000000000000001% 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(
    mix_oklab_upper_neighbor_is_exact,
    "oklab(.999999999999999999999 0 0)",
    "oklab(0.999999999999999999999 0 0)"
);
mix_text_case!(
    mix_oklab_upper_number_neighbor_clips,
    "oklab(1.000000000000000000001 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklab_upper_percentage_neighbor_clips,
    "oklab(100.000000000000000000001% 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklch_upper_number_neighbor_clips,
    "oklch(1.000000000000000000001 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_oklch_upper_percentage_neighbor_clips,
    "oklch(100.000000000000000000001% 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_lch_tiny_negative_chroma_clips,
    "lch(50 -1e-400 30)",
    "lch(50 0 30)"
);
mix_text_case!(
    mix_oklch_tiny_negative_chroma_percentage_clips,
    "oklch(.5 -1e-400% 30)",
    "oklch(0.5 0 30)"
);
mix_text_case!(
    mix_lab_tiny_negative_lightness_clips,
    "lab(-1e-400 0 0)",
    "lab(0 0 0)"
);
mix_text_case!(
    mix_oklab_tiny_negative_lightness_percentage_clips,
    "oklab(-1e-400% 0 0)",
    "oklab(0 0 0)"
);
mix_text_case!(
    mix_lab_axes_keep_exact_signed_scale,
    "lab(125% -20% 30%)",
    "lab(100 -25 37.5)"
);
mix_text_case!(
    mix_lch_two_direct_bounds_apply,
    "lch(-20% -20% 30)",
    "lch(0 0 30)"
);
mix_text_case!(
    mix_oklab_axes_keep_exact_signed_scale,
    "oklab(120% 150% -200%)",
    "oklab(1 0.6 -0.8)"
);
mix_text_case!(
    mix_oklch_two_direct_bounds_apply,
    "oklch(-.2 -20% 30)",
    "oklch(0 0 30)"
);
mix_text_case!(
    mix_lab_finite_binary_overflow_clips,
    "lab(1e400 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lch_finite_binary_overflow_percentage_clips,
    "lch(1e400% 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(
    mix_oklab_finite_binary_overflow_clips,
    "oklab(1e400 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklch_negative_finite_binary_overflow_chroma_clips,
    "oklch(.5 -1e400 30)",
    "oklch(0.5 0 30)"
);
mix_text_case!(
    mix_lab_discarded_million_digit_text_is_unneeded,
    "lab(1e2000000 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lab_exponent_beyond_i128_clips,
    "lab(1e999999999999999999999999999999999999999999 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lch_percentage_exponent_beyond_i128_clips,
    "lch(1e999999999999999999999999999999999999999999% 0 0)",
    "lch(100 0 0)"
);
mix_text_case!(
    mix_oklab_number_exponent_beyond_i128_clips,
    "oklab(1e999999999999999999999999999999999999999999 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_oklch_percentage_exponent_beyond_i128_clips,
    "oklch(1e999999999999999999999999999999999999999999% 0 0)",
    "oklch(1 0 0)"
);
mix_text_case!(
    mix_negative_infinitesimal_exponent_beyond_i128_clips,
    "lab(-1e-999999999999999999999999999999999999999999 0 0)",
    "lab(0 0 0)"
);
mix_text_case!(
    mix_oklch_negative_extreme_slots_clip,
    "oklch(-1e-999999999999999999999999999999999999999999 -1e400 30)",
    "oklch(0 0 30)"
);
mix_text_case!(
    mix_lch_negative_chroma_exponent_beyond_i128_clips,
    "lch(50 -1e-999999999999999999999999999999999999999999 30)",
    "lch(50 0 30)"
);
mix_text_case!(
    mix_lab_signed_zero_remains_zero,
    "lab(-0e999999999999999999999999999999999999999999 0 0)",
    "lab(0 0 0)"
);
mix_text_case!(
    mix_lch_chroma_signed_zero_remains_zero,
    "lch(50 -0e-999999999999999999999999999999999999999999% 30)",
    "lch(50 0 30)"
);
mix_text_case!(
    mix_oklab_equal_upper_spelling_normalizes,
    "oklab(100.000% 0 0)",
    "oklab(1 0 0)"
);
mix_text_case!(
    mix_lab_equal_upper_exponent_spelling_normalizes,
    "lab(1e2 0 0)",
    "lab(100 0 0)"
);
mix_text_case!(
    mix_lch_positive_chroma_has_no_ceiling,
    "lch(50 200% 30)",
    "lch(50 300 30)"
);
mix_text_case!(
    mix_oklch_positive_chroma_has_no_ceiling,
    "oklch(.5 200% 30)",
    "oklch(0.5 0.8 30)"
);
mix_text_case!(
    mix_lab_unbounded_axes_have_no_reference_ceiling,
    "lab(50 200% -300%)",
    "lab(50 250 -375)"
);
mix_text_case!(
    mix_oklab_unbounded_axes_have_no_reference_ceiling,
    "oklab(.5 200% -300%)",
    "oklab(0.5 0.8 -1.2)"
);
mix_text_case!(
    mix_lab_missing_lightness_stays_missing,
    "lab(none 0 0)",
    "lab(none 0 0)"
);
mix_text_case!(
    mix_lab_missing_axis_does_not_disable_lightness_bound,
    "lab(125 none 0)",
    "lab(100 none 0)"
);
mix_text_case!(
    mix_lch_missing_chroma_does_not_disable_lightness_bound,
    "lch(125 none 30)",
    "lch(100 none 30)"
);
mix_text_case!(
    mix_oklch_missing_hue_does_not_disable_direct_bounds,
    "oklch(120% -20% none)",
    "oklch(1 0 none)"
);
mix_text_case!(
    mix_oklab_missing_alpha_does_not_disable_lightness_bound,
    "oklab(120% 0 0 / none)",
    "oklab(1 0 0 / none)"
);
mix_text_case!(
    mix_lab_calculated_lightness_remains_unbounded_wrapper,
    "lab(calc(125) 0 0)",
    "lab(calc(125) 0 0)"
);
mix_text_case!(
    mix_lch_calculated_chroma_remains_negative_wrapper,
    "lch(50 calc(-20) 30)",
    "lch(50 calc(-20) 30)"
);
mix_text_case!(
    mix_oklab_calculated_lightness_remains_scaled_wrapper,
    "oklab(calc(120%) 0 0)",
    "oklab(calc(1.2) 0 0)"
);
mix_text_case!(
    mix_oklch_calculated_chroma_remains_scaled_wrapper,
    "oklch(.5 calc(-20%) 30)",
    "oklch(0.5 calc(-0.08) 30)"
);
mix_text_case!(
    mix_lch_direct_lightness_beside_calculated_chroma_clips,
    "lch(125 calc(-20) 30)",
    "lch(100 calc(-20) 30)"
);
mix_text_case!(
    mix_lab_direct_lightness_beside_numeric_axis_clips,
    "lab(125 calc(1 / 128) 0)",
    "lab(100 calc(0.0078125) 0)"
);
mix_text_case!(
    mix_lab_direct_lightness_beside_contextual_axis_clips,
    "lab(125 calc(1em / 1px) 0)",
    "lab(100 calc(1 * 1em / 1px) 0)"
);
mix_text_case!(
    mix_oklch_direct_chroma_beside_contextual_lightness_clips,
    "oklch(calc(1em / 1px) -20% 30)",
    "oklch(calc(1 * 1em / 1px) 0 30)"
);
mix_text_case!(
    mix_contextual_axis_explicit_identity_scale_control,
    "lab(50 calc(1em / 1px) 0)",
    "lab(50 calc(1 * 1em / 1px) 0)"
);
mix_text_case!(
    mix_lab_contextual_lightness_keeps_dimensions,
    "lab(calc(125 * 1em / 1px) 0 0)",
    "lab(calc(125 * 1em / 1px) 0 0)"
);
mix_text_case!(
    mix_lab_calculated_alpha_keeps_wrapper_beside_direct_bound,
    "lab(125 0 0 / calc(.5))",
    "lab(100 0 0 / calc(0.5))"
);
mix_text_case!(
    mix_lab_origin_preserves_unclamped_percentages,
    "alpha(from lab(125% -20% 30%))",
    "alpha(from lab(125% -20% 30%))"
);
mix_text_case!(
    mix_lch_origin_preserves_negative_chroma,
    "alpha(from lch(125% -20% 30))",
    "alpha(from lch(125% -20% 30))"
);
mix_text_case!(
    mix_oklab_origin_preserves_unclamped_lightness,
    "alpha(from oklab(120% 150% -200%))",
    "alpha(from oklab(120% 150% -200%))"
);
mix_text_case!(
    mix_oklch_origin_preserves_negative_chroma,
    "alpha(from oklch(-.2 -20% 30))",
    "alpha(from oklch(-0.2 -20% 30))"
);
mix_text_case!(
    mix_lab_relative_channel_keeps_125,
    "lab(from red 125 a b)",
    "lab(from red 125 a b)"
);
mix_text_case!(
    mix_lch_relative_chroma_keeps_negative,
    "lch(from red l -20 h)",
    "lch(from red l -20 h)"
);
mix_text_case!(
    mix_oklab_relative_percentage_keeps_out_of_range_number,
    "oklab(from red 120% a b)",
    "oklab(from red 1.2 a b)"
);
mix_text_case!(
    mix_oklch_relative_chroma_keeps_negative,
    "oklch(from red l -20% h)",
    "oklch(from red l -0.08 h)"
);
mix_text_case!(
    mix_predefined_rgb_keeps_out_of_range_channels,
    "color(display-p3 -20% 120% 50%)",
    "color(display-p3 -0.2 1.2 0.5)"
);
mix_text_case!(
    mix_predefined_xyz_keeps_out_of_range_channels,
    "color(xyz -20% 120% 50%)",
    "color(xyz-d65 -0.2 1.2 0.5)"
);
mix_text_case!(
    mix_custom_profile_keeps_unbounded_channels,
    "color(--P -20% 120%)",
    "color(--P -0.2 1.2)"
);
mix_text_case!(
    mix_ordinary_rgb_keeps_existing_bound,
    "rgb(300 -20 0)",
    "rgb(255, 0, 0)"
);

fn assert_color_result(value: &CssColor, limits: Limits, expected: Result<&str, Kind>) {
    let before = value.clone();
    let result = value.to_specified_css_with_limits(limits);
    assert_eq!(result, value.to_specified_css_with_limits(limits));
    assert_eq!(value, &before);
    match expected {
        Ok(text) => assert_eq!(result.unwrap(), text),
        Err(kind) => assert_eq!(result.unwrap_err().kind(), kind),
    }
}

fn assert_pair_result(value: &CssBorderColorPair, limits: Limits, expected: Result<&str, Kind>) {
    let before = value.clone();
    let result = value.serialize_specified_with_limits(limits);
    assert_eq!(result, value.serialize_specified_with_limits(limits));
    assert_eq!(value, &before);
    match expected {
        Ok(text) => assert_eq!(result.unwrap(), text),
        Err(kind) => assert_eq!(result.unwrap_err().kind(), kind),
    }
}

macro_rules! resources {
    ($success:ident, $input:ident, $projection:ident, $byte:ident,
     $check:ident, $value:expr, $inputs:expr, $projections:expr, $bytes:expr, $expected:literal) => {
        #[test]
        fn $success() {
            $check(
                &$value,
                Limits::new($inputs, $projections, $bytes),
                Ok($expected),
            );
        }
        #[test]
        fn $input() {
            $check(
                &$value,
                Limits::new($inputs - 1, usize::MAX, usize::MAX),
                Err(Kind::InputNodeLimit),
            );
        }
        #[test]
        fn $projection() {
            $check(
                &$value,
                Limits::new(usize::MAX, $projections - 1, usize::MAX),
                Err(Kind::ProjectionNodeLimit),
            );
        }
        #[test]
        fn $byte() {
            $check(
                &$value,
                Limits::new(usize::MAX, usize::MAX, $bytes - 1),
                Err(Kind::ByteLimit),
            );
        }
    };
}

// No explicit weights: generated100/2 costs6 projections, then each of two
// ratio comparisons costs7. Mix root1 + Lab root/classifier2 + blue root1.
// Input: Mix1 + Lab4 + blue1. Bytes10 +12 +2 +4 +1 =29.
resources!(
    mix_huge_endpoint_exact_resources,
    mix_huge_endpoint_one_short_input,
    mix_huge_endpoint_one_short_projection,
    mix_huge_endpoint_one_short_byte,
    assert_color_result,
    color("color-mix(lab(1e400 0 0), blue)"),
    6,
    24,
    29,
    "color-mix(lab(100 0 0), blue)"
);
resources!(
    mix_greater_endpoint_exact_resources,
    mix_greater_endpoint_one_short_input,
    mix_greater_endpoint_one_short_projection,
    mix_greater_endpoint_one_short_byte,
    assert_color_result,
    color("color-mix(lab(101 0 0), blue)"),
    6,
    24,
    29,
    "color-mix(lab(100 0 0), blue)"
);
resources!(
    mix_equal_endpoint_exact_resources,
    mix_equal_endpoint_one_short_input,
    mix_equal_endpoint_one_short_projection,
    mix_equal_endpoint_one_short_byte,
    assert_color_result,
    color("color-mix(lab(100 0 0), blue)"),
    6,
    24,
    29,
    "color-mix(lab(100 0 0), blue)"
);
resources!(
    mix_beyond_i128_endpoint_exact_resources,
    mix_beyond_i128_endpoint_one_short_input,
    mix_beyond_i128_endpoint_one_short_projection,
    mix_beyond_i128_endpoint_one_short_byte,
    assert_color_result,
    color("color-mix(lab(1e999999999999999999999999999999999999999999 0 0), blue)"),
    6,
    24,
    29,
    "color-mix(lab(100 0 0), blue)"
);
// In-range50 additionally retains lexical limb1, multiplication1 and clone1.
resources!(
    mix_inrange_exact_resources,
    mix_inrange_one_short_input,
    mix_inrange_one_short_projection,
    mix_inrange_one_short_byte,
    assert_color_result,
    color("color-mix(lab(50 0 0), blue)"),
    6,
    27,
    28,
    "color-mix(lab(50 0 0), blue)"
);
resources!(
    mix_negative_endpoint_exact_resources,
    mix_negative_endpoint_one_short_input,
    mix_negative_endpoint_one_short_projection,
    mix_negative_endpoint_one_short_byte,
    assert_color_result,
    color("color-mix(lab(-1e-400 0 0), blue)"),
    6,
    24,
    27,
    "color-mix(lab(0 0 0), blue)"
);
// Outer Mix1 + generated weights20 + inner24 + blue1; no child refunds.
resources!(
    nested_mix_exact_resources,
    nested_mix_one_short_input,
    nested_mix_one_short_projection,
    nested_mix_one_short_byte,
    assert_color_result,
    color("color-mix(color-mix(lab(1e400 0 0), blue), blue)"),
    8,
    46,
    46,
    "color-mix(color-mix(lab(100 0 0), blue), blue)"
);
resources!(
    origin_mix_exact_resources,
    origin_mix_one_short_input,
    origin_mix_one_short_projection,
    origin_mix_one_short_byte,
    assert_color_result,
    color("alpha(from color-mix(lab(1e400 0 0), blue))"),
    7,
    25,
    41,
    "alpha(from color-mix(lab(100 0 0), blue))"
);

fn endpoint_pair() -> CssBorderColorPair {
    let endpoint = color("color-mix(lab(1e400 0 0), blue)");
    CssBorderColorPair::new(endpoint.clone(), Some(endpoint))
}
// Pair root1 + two complete Mix operations: I13/P49;29 +1 +29 bytes.
resources!(
    mix_pair_exact_resources,
    mix_pair_one_short_input,
    mix_pair_one_short_projection,
    mix_pair_one_short_byte,
    assert_pair_result,
    endpoint_pair(),
    13,
    49,
    59,
    "color-mix(lab(100 0 0), blue) color-mix(lab(100 0 0), blue)"
);

fn prefixed_pair(second: &str) -> CssBorderColorPair {
    CssBorderColorPair::new(color("color(--é 0)"), Some(color(second)))
}
// Actual prefix is13 UTF8 bytes, separator1. Identifier adds no nodes; zero
// has no rational limbs. Pair1 + prefixI2/P1 + MixI6/P24 yieldsI9/P26.
resources!(
    utf8_prefixed_mix_exact_resources,
    utf8_prefixed_mix_one_short_input,
    utf8_prefixed_mix_one_short_projection,
    utf8_prefixed_mix_one_short_byte,
    assert_pair_result,
    prefixed_pair("color-mix(lab(1e400 0 0), blue)"),
    9,
    26,
    43,
    "color(--é 0) color-mix(lab(100 0 0), blue)"
);
// Calc function+leaf2 inputs; projected leaf1+number factor1+folded product1.
// Captured17-byte calc remains scratch; Lab final28 and Mix final45 bytes.
resources!(
    mix_calculated_axis_exact_resources,
    mix_calculated_axis_one_short_input,
    mix_calculated_axis_one_short_projection,
    mix_calculated_axis_one_short_byte,
    assert_color_result,
    color("color-mix(lab(1e400 calc(10000000000) 0), blue)"),
    7,
    27,
    45,
    "color-mix(lab(100 calc(10000000000) 0), blue)"
);

macro_rules! color_error {
    ($name:ident, $source:literal, $input:expr, $projection:expr, $bytes:expr, $kind:ident) => {
        #[test]
        fn $name() {
            assert_color_result(
                &color($source),
                Limits::new($input, $projection, $bytes),
                Err(Kind::$kind),
            );
        }
    };
}
color_error!(
    mix_zero_input_precedes_other_limits,
    "color-mix(lab(125 0 0), blue)",
    0,
    0,
    0,
    InputNodeLimit
);
color_error!(
    mix_zero_projection_precedes_bytes,
    "color-mix(lab(125 0 0), blue)",
    1,
    0,
    0,
    ProjectionNodeLimit
);
color_error!(
    mix_zero_output_is_typed,
    "color-mix(lab(125 0 0), blue)",
    usize::MAX,
    usize::MAX,
    0,
    ByteLimit
);
color_error!(
    mix_inrange_beyond_i128_tiny_remains_bounded,
    "color-mix(lab(1e-999999999999999999999999999999999999999999 0 0), blue)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
color_error!(
    mix_unbounded_positive_chroma_remains_bounded,
    "color-mix(lch(50 1e2000000 30), blue)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
color_error!(
    mix_unbounded_axis_remains_bounded,
    "color-mix(lab(50 1e2000000 0), blue)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
color_error!(
    mix_unbounded_axis_beyond_i128_remains_bounded,
    "color-mix(lab(50 1e999999999999999999999999999999999999999999 0), blue)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
color_error!(
    mix_true_origin_huge_coefficient_keeps_typed_failure,
    "color-mix(alpha(from lab(1e2000000 0 0)), blue)",
    usize::MAX,
    usize::MAX,
    64,
    ByteLimit
);
color_error!(
    mix_contextual_lightness_keeps_required_text,
    "color-mix(lab(calc(125 * 1em / 1px) 0 0), blue)",
    usize::MAX,
    usize::MAX,
    29,
    ByteLimit
);

#[test]
fn prefixed_calculated_mix_fits_exact_final_bytes() {
    assert_pair_result(
        &prefixed_pair("color-mix(lab(1e400 calc(10000000000) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 59),
        Ok("color(--é 0) color-mix(lab(100 calc(10000000000) 0), blue)"),
    );
}
#[test]
fn prefixed_calculated_mix_one_short_final_byte_is_atomic() {
    assert_pair_result(
        &prefixed_pair("color-mix(lab(1e400 calc(10000000000) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 58),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn prefixed_calculated_mix_cannot_skip_required_scratch() {
    // Prefix14+Mix header10 leaves16, insufficient for17-byte calc capture.
    assert_pair_result(
        &prefixed_pair("color-mix(lab(1e400 calc(10000000000) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 40),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn prefixed_contextual_mix_fits_exact_final_bytes() {
    assert_pair_result(
        &prefixed_pair("color-mix(lab(125 calc(1em / 1px) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 61),
        Ok("color(--é 0) color-mix(lab(100 calc(1 * 1em / 1px) 0), blue)"),
    );
}
#[test]
fn prefixed_contextual_mix_one_short_final_byte_is_atomic() {
    assert_pair_result(
        &prefixed_pair("color-mix(lab(125 calc(1em / 1px) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 60),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn prefixed_contextual_mix_cannot_skip_required_scratch() {
    // Prefix14+Mix header10 leaves18, insufficient for19-byte contextual text.
    assert_pair_result(
        &prefixed_pair("color-mix(lab(125 calc(1em / 1px) 0), blue)"),
        Limits::new(usize::MAX, usize::MAX, 42),
        Err(Kind::ByteLimit),
    );
}
#[test]
fn failing_unbounded_second_mix_keeps_entire_pair_unchanged() {
    let pair = CssBorderColorPair::new(
        color("color-mix(lab(1e400 0 0), blue)"),
        Some(color("color-mix(lab(50 1e2000000 0), blue)")),
    );
    assert_pair_result(
        &pair,
        Limits::new(usize::MAX, usize::MAX, 64),
        Err(Kind::ByteLimit),
    );
}

#[test]
fn mix_long_decimal_upper_neighbor_clips_from_real_public_token() {
    let source = format!("color-mix(lab(100.{}1 0 0), blue)", "0".repeat(4096));
    assert_text(&source, "color-mix(lab(100 0 0), blue)");
}
#[test]
fn mix_long_exponent_clips_without_decimal_expansion() {
    let source = format!("color-mix(lab(1e{} 0 0), blue)", "9".repeat(4096));
    assert_text(&source, "color-mix(lab(100 0 0), blue)");
}
#[test]
fn mix_long_negative_chroma_exponent_clips_from_real_public_token() {
    let source = format!("color-mix(lch(50 -1e-{} 30), blue)", "9".repeat(4096));
    assert_text(&source, "color-mix(lch(50 0 30), blue)");
}
#[test]
fn mix_positive_binary_underflow_keeps_exact_decimal_text() {
    let expected = format!("color-mix(lab(0.{}1 0 0), blue)", "0".repeat(399));
    assert_text("color-mix(lab(1e-400 0 0), blue)", &expected);
}
#[test]
fn mix_positive_huge_chroma_has_no_ceiling() {
    let expected = format!("color-mix(lch(50 1{} 30), blue)", "0".repeat(400));
    assert_text("color-mix(lch(50 1e400 30), blue)", &expected);
}

fn checked_literal(spelling: &str, programmatic: bool) -> CssColorComponent {
    let supplied = if programmatic {
        CssComponentValue::try_token(spelling).unwrap()
    } else {
        components(spelling).items()[0].clone()
    };
    let expected_origin = supplied.origin().clone();
    let (coefficient, origin, value) = if let Some(coefficient) = spelling.strip_suffix('%') {
        let literal = CssColorPercentageLiteral::try_from_component(supplied).unwrap();
        assert_eq!(literal.numeric().representation(), coefficient);
        (
            coefficient,
            literal.origin().clone(),
            CssColorComponent::Percentage(literal),
        )
    } else {
        let literal = CssColorNumberLiteral::try_from_component(supplied).unwrap();
        assert_eq!(literal.numeric().representation(), spelling);
        (
            spelling,
            literal.origin().clone(),
            CssColorComponent::Number(literal),
        )
    };
    assert!(!coefficient.is_empty());
    assert_eq!(origin, expected_origin);
    if programmatic {
        assert_eq!(origin, CssValueOrigin::Programmatic);
    } else {
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("parsed token origin");
        };
        assert_eq!(origin.source().as_str(), spelling);
        assert_eq!(origin.span().start().byte_offset().value(), 0);
        assert_eq!(origin.span().end().byte_offset().value(), spelling.len());
    }
    value
}

fn checked_family_mix(
    family: &str,
    spellings: [&str; 3],
    expected_child: &str,
    programmatic: bool,
) {
    let first = checked_literal(spellings[0], programmatic);
    let second = checked_literal(spellings[1], programmatic);
    let third = checked_literal(spellings[2], programmatic);
    let child = match family {
        "lab" | "oklab" => {
            let model =
                CssLabColor::try_new(first.clone(), second.clone(), third.clone(), None).unwrap();
            if family == "lab" {
                CssColor::from_lab(model)
            } else {
                CssColor::from_oklab(model)
            }
        }
        "lch" | "oklch" => {
            let CssColorComponent::Number(hue) = third.clone() else {
                panic!("number hue");
            };
            let model = CssLchColor::try_new(
                first.clone(),
                second.clone(),
                CssColorHue::Number(hue),
                None,
            )
            .unwrap();
            if family == "lch" {
                CssColor::from_lch(model)
            } else {
                CssColor::from_oklch(model)
            }
        }
        _ => panic!("test family"),
    };
    let child_before = child.clone();
    let weight = CssColorMixWeight::literal(
        CssColorMixPercentage::try_from_component(components("30%").items()[0].clone()).unwrap(),
    );
    let second_weight = CssColorMixWeight::try_calculation(
        CssPercentageCalculation::try_from_components(components("calc(70%)")).unwrap(),
    )
    .unwrap();
    let interpolation = CssColorInterpolation::from_predefined(
        CssColorInterpolationMethod::try_new(
            CssColorInterpolationSpace::Lch,
            Some(CssHueInterpolationMethod::Longer),
        )
        .unwrap(),
    );
    let blue = CssColor::from_named(CssNamedColor::try_new("blue").unwrap());
    let mix = CssColorMix::try_new(
        Some(interpolation.clone()),
        vec![
            CssColorMixComponent::new(child, Some(weight.clone())),
            CssColorMixComponent::new(blue.clone(), Some(second_weight.clone())),
        ],
    )
    .unwrap();
    let value = CssColor::from_color_mix(mix);
    let before = value.clone();
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    let retained_mix = value.color_mix_value().unwrap();
    assert_eq!(retained_mix.interpolation(), Some(&interpolation));
    assert_eq!(retained_mix.components().len(), 2);
    assert_eq!(retained_mix.components()[0].weight(), Some(&weight));
    assert_eq!(retained_mix.components()[1].weight(), Some(&second_weight));
    assert_eq!(retained_mix.components()[1].color(), &blue);
    let retained = retained_mix.components()[0].color();
    assert_eq!(retained, &child_before);
    match family {
        "lab" | "oklab" => {
            let model = if family == "lab" {
                retained.lab_value()
            } else {
                retained.oklab_value()
            }
            .unwrap();
            assert_eq!(model.lightness(), &first);
            assert_eq!(model.a(), &second);
            assert_eq!(model.b(), &third);
            assert!(model.alpha().is_none());
        }
        "lch" | "oklch" => {
            let model = if family == "lch" {
                retained.lch_value()
            } else {
                retained.oklch_value()
            }
            .unwrap();
            assert_eq!(model.lightness(), &first);
            assert_eq!(model.chroma(), &second);
            let CssColorComponent::Number(hue) = third else {
                panic!("number hue");
            };
            assert_eq!(model.hue(), &CssColorHue::Number(hue));
            assert!(model.alpha().is_none());
        }
        _ => unreachable!(),
    }
    let expected = format!("color-mix(in lch longer hue, {expected_child} 30%, blue calc(70%))");
    assert_eq!(result.unwrap(), expected);
    assert_eq!(color(&expected).to_specified_css().unwrap(), expected);
}

macro_rules! checked_cases {
    ($parsed:ident, $programmatic:ident, $family:literal, $spellings:expr, $expected:literal) => {
        #[test]
        fn $parsed() {
            checked_family_mix($family, $spellings, $expected, false);
        }
        #[test]
        fn $programmatic() {
            checked_family_mix($family, $spellings, $expected, true);
        }
    };
}
checked_cases!(
    checked_lab_parsed_mix_keeps_literal_kinds_and_origins,
    checked_lab_programmatic_mix_keeps_literal_kinds_and_origins,
    "lab",
    ["125%", "-20%", "30%"],
    "lab(100 -25 37.5)"
);
checked_cases!(
    checked_lch_parsed_mix_keeps_literal_kinds_and_origins,
    checked_lch_programmatic_mix_keeps_literal_kinds_and_origins,
    "lch",
    ["-20%", "-20%", "30"],
    "lch(0 0 30)"
);
checked_cases!(
    checked_oklab_parsed_mix_keeps_literal_kinds_and_origins,
    checked_oklab_programmatic_mix_keeps_literal_kinds_and_origins,
    "oklab",
    ["120%", "150%", "-200%"],
    "oklab(1 0.6 -0.8)"
);
checked_cases!(
    checked_oklch_parsed_mix_keeps_literal_kinds_and_origins,
    checked_oklch_programmatic_mix_keeps_literal_kinds_and_origins,
    "oklch",
    ["-.2", "-20%", "30"],
    "oklch(0 0 30)"
);
checked_cases!(
    checked_extreme_lab_parsed_mix_clips_without_mutation,
    checked_extreme_lab_programmatic_mix_clips_without_mutation,
    "lab",
    ["1e999999999999999999999999999999999999999999", "0", "0"],
    "lab(100 0 0)"
);

#[test]
fn checked_nested_mix_preserves_the_shared_depth_ceiling() {
    // Lab payload owns depth1.255 enclosing Mix nodes reach the public256 cap.
    let mut value = color("lab(125 0 0)");
    let mut expected = "lab(100 0 0)".to_owned();
    let blue = CssColor::from_named(CssNamedColor::try_new("blue").unwrap());
    for _ in 0..255 {
        value = CssColor::from_color_mix(
            CssColorMix::try_new(
                None,
                vec![
                    CssColorMixComponent::new(value, None),
                    CssColorMixComponent::new(blue.clone(), None),
                ],
            )
            .unwrap(),
        );
        expected = format!("color-mix({expected}, blue)");
    }
    let before = value.clone();
    let error = CssColorMix::try_new(
        None,
        vec![
            CssColorMixComponent::new(value.clone(), None),
            CssColorMixComponent::new(blue, None),
        ],
    )
    .unwrap_err();
    assert_eq!(error, CssColorMixConstructionError::NestingLimit);
    let result = value.to_specified_css();
    assert_eq!(result, value.to_specified_css());
    assert_eq!(value, before);
    assert_eq!(result.unwrap(), expected);
}
