//! The Color 4 CRD 2026-09-08 §§9.3/9.4 parsed-value lightness and chroma
//! bounds for direct Lab-family literals. Each row is an authored color and
//! its independently derived specified text; consumers run the table
//! standalone and as the first `color-mix()` operand, through the same
//! [`assert_specified_text`] runner.

use super::{checked_color, checked_color_declaration, component_values, declaration_color};
use surgeist_css::CssParser;
use surgeist_css::component::CssValueOrigin;
use surgeist_css::declaration::CssImportance;
use surgeist_css::property::CssKnownProperty;

/// `source` serializes as `expected` through both the checked and the
/// parsed `color` entry, without mutating either declaration, and the
/// expected text re-enters as itself.
pub fn assert_specified_text(source: &str, expected: &str) {
    let checked = checked_color_declaration(source);
    let source_attribute = format!("color:{source}!important;opacity:.5");
    let report = CssParser::default().parse_block_contents(&source_attribute);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let parsed = &report.syntax()[0];
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(checked.importance(), CssImportance::Important);
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(checked.value_components(), &component_values(source));
    assert_eq!(
        parsed
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
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
    let checked_result = declaration_color(&checked).serialize_specified(Default::default());
    let parsed_result = declaration_color(parsed).serialize_specified(Default::default());
    assert_eq!(
        checked_result,
        declaration_color(&checked).serialize_specified(Default::default())
    );
    assert_eq!(
        parsed_result,
        declaration_color(parsed).serialize_specified(Default::default())
    );
    assert_eq!(checked, checked_before);
    assert_eq!(parsed, &parsed_before);
    assert_eq!(checked_result.unwrap(), expected, "{source}: checked entry");
    assert_eq!(parsed_result.unwrap(), expected, "{source}: parsed entry");
    // Reentry complements the explicit independent literal oracle.
    assert_eq!(
        checked_color(expected)
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
}

macro_rules! lab_direct_component_cases {
    ($case:ident) => {
        $case!(lab_number_upper_clips, "lab(125 0 0)", "lab(100 0 0)");
        $case!(lab_percentage_upper_clips, "lab(125% 0 0)", "lab(100 0 0)");
        $case!(lab_number_lower_clips, "lab(-20 0 0)", "lab(0 0 0)");
        $case!(lab_percentage_lower_clips, "lab(-20% 0 0)", "lab(0 0 0)");
        $case!(lch_number_upper_clips, "lch(125 0 0)", "lch(100 0 0)");
        $case!(lch_percentage_upper_clips, "lch(125% 0 0)", "lch(100 0 0)");
        $case!(lch_number_lower_clips, "lch(-20 0 0)", "lch(0 0 0)");
        $case!(lch_percentage_lower_clips, "lch(-20% 0 0)", "lch(0 0 0)");
        $case!(oklab_number_upper_clips, "oklab(1.2 0 0)", "oklab(1 0 0)");
        $case!(
            oklab_percentage_upper_clips,
            "oklab(120% 0 0)",
            "oklab(1 0 0)"
        );
        $case!(oklab_number_lower_clips, "oklab(-0.2 0 0)", "oklab(0 0 0)");
        $case!(
            oklab_percentage_lower_clips,
            "oklab(-20% 0 0)",
            "oklab(0 0 0)"
        );
        $case!(oklch_number_upper_clips, "oklch(1.2 0 0)", "oklch(1 0 0)");
        $case!(
            oklch_percentage_upper_clips,
            "oklch(120% 0 0)",
            "oklch(1 0 0)"
        );
        $case!(oklch_number_lower_clips, "oklch(-0.2 0 0)", "oklch(0 0 0)");
        $case!(
            oklch_percentage_lower_clips,
            "oklch(-20% 0 0)",
            "oklch(0 0 0)"
        );
        $case!(
            lch_number_chroma_lower_clips,
            "lch(50 -20 30)",
            "lch(50 0 30)"
        );
        $case!(
            lch_percentage_chroma_lower_clips,
            "lch(50 -20% 30)",
            "lch(50 0 30)"
        );
        $case!(
            oklch_number_chroma_lower_clips,
            "oklch(.5 -.2 30)",
            "oklch(0.5 0 30)"
        );
        $case!(
            oklch_percentage_chroma_lower_clips,
            "oklch(50% -20% 30)",
            "oklch(0.5 0 30)"
        );
        $case!(
            lab_lower_neighbor_is_exact,
            "lab(1e-20 0 0)",
            "lab(0.00000000000000000001 0 0)"
        );
        $case!(
            lab_upper_neighbor_is_exact,
            "lab(99.999999999999999999999 0 0)",
            "lab(99.999999999999999999999 0 0)"
        );
        $case!(
            lab_upper_number_neighbor_clips,
            "lab(100.000000000000000000001 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lab_upper_percentage_neighbor_clips,
            "lab(100.000000000000000000001% 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lch_upper_number_neighbor_clips,
            "lch(100.000000000000000000001 0 0)",
            "lch(100 0 0)"
        );
        $case!(
            lch_upper_percentage_neighbor_clips,
            "lch(100.000000000000000000001% 0 0)",
            "lch(100 0 0)"
        );
        $case!(
            oklab_upper_neighbor_is_exact,
            "oklab(.999999999999999999999 0 0)",
            "oklab(0.999999999999999999999 0 0)"
        );
        $case!(
            oklab_upper_number_neighbor_clips,
            "oklab(1.000000000000000000001 0 0)",
            "oklab(1 0 0)"
        );
        $case!(
            oklab_upper_percentage_neighbor_clips,
            "oklab(100.000000000000000000001% 0 0)",
            "oklab(1 0 0)"
        );
        $case!(
            oklch_upper_number_neighbor_clips,
            "oklch(1.000000000000000000001 0 0)",
            "oklch(1 0 0)"
        );
        $case!(
            oklch_upper_percentage_neighbor_clips,
            "oklch(100.000000000000000000001% 0 0)",
            "oklch(1 0 0)"
        );
        $case!(
            lch_tiny_negative_chroma_clips,
            "lch(50 -1e-400 30)",
            "lch(50 0 30)"
        );
        $case!(
            oklch_tiny_negative_chroma_percentage_clips,
            "oklch(.5 -1e-400% 30)",
            "oklch(0.5 0 30)"
        );
        $case!(
            lab_tiny_negative_lightness_clips,
            "lab(-1e-400 0 0)",
            "lab(0 0 0)"
        );
        $case!(
            oklab_tiny_negative_lightness_percentage_clips,
            "oklab(-1e-400% 0 0)",
            "oklab(0 0 0)"
        );
        $case!(
            lab_axes_keep_exact_signed_scale,
            "lab(125% -20% 30%)",
            "lab(100 -25 37.5)"
        );
        $case!(
            lch_two_direct_bounds_apply,
            "lch(-20% -20% 30)",
            "lch(0 0 30)"
        );
        $case!(
            oklab_axes_keep_exact_signed_scale,
            "oklab(120% 150% -200%)",
            "oklab(1 0.6 -0.8)"
        );
        $case!(
            oklch_two_direct_bounds_apply,
            "oklch(-.2 -20% 30)",
            "oklch(0 0 30)"
        );
        $case!(
            lab_finite_binary_overflow_clips,
            "lab(1e400 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lch_finite_binary_overflow_percentage_clips,
            "lch(1e400% 0 0)",
            "lch(100 0 0)"
        );
        $case!(
            oklab_finite_binary_overflow_clips,
            "oklab(1e400 0 0)",
            "oklab(1 0 0)"
        );
        $case!(
            oklch_negative_finite_binary_overflow_chroma_clips,
            "oklch(.5 -1e400 30)",
            "oklch(0.5 0 30)"
        );
        $case!(
            lab_discarded_million_digit_text_is_unneeded,
            "lab(1e2000000 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lab_exponent_beyond_i128_clips,
            "lab(1e999999999999999999999999999999999999999999 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lch_percentage_exponent_beyond_i128_clips,
            "lch(1e999999999999999999999999999999999999999999% 0 0)",
            "lch(100 0 0)"
        );
        $case!(
            oklab_number_exponent_beyond_i128_clips,
            "oklab(1e999999999999999999999999999999999999999999 0 0)",
            "oklab(1 0 0)"
        );
        $case!(
            oklch_percentage_exponent_beyond_i128_clips,
            "oklch(1e999999999999999999999999999999999999999999% 0 0)",
            "oklch(1 0 0)"
        );
        $case!(
            negative_infinitesimal_exponent_beyond_i128_clips,
            "lab(-1e-999999999999999999999999999999999999999999 0 0)",
            "lab(0 0 0)"
        );
        $case!(
            oklch_negative_extreme_slots_clip,
            "oklch(-1e-999999999999999999999999999999999999999999 -1e400 30)",
            "oklch(0 0 30)"
        );
        $case!(
            lch_negative_chroma_exponent_beyond_i128_clips,
            "lch(50 -1e-999999999999999999999999999999999999999999 30)",
            "lch(50 0 30)"
        );
        $case!(
            lab_signed_zero_remains_zero,
            "lab(-0e999999999999999999999999999999999999999999 0 0)",
            "lab(0 0 0)"
        );
        $case!(
            lch_chroma_signed_zero_remains_zero,
            "lch(50 -0e-999999999999999999999999999999999999999999% 30)",
            "lch(50 0 30)"
        );
        $case!(
            oklab_equal_upper_spelling_normalizes,
            "oklab(100.000% 0 0)",
            "oklab(1 0 0)"
        );
        $case!(
            lab_equal_upper_exponent_spelling_normalizes,
            "lab(1e2 0 0)",
            "lab(100 0 0)"
        );
        $case!(
            lch_positive_chroma_has_no_ceiling,
            "lch(50 200% 30)",
            "lch(50 300 30)"
        );
        $case!(
            oklch_positive_chroma_has_no_ceiling,
            "oklch(.5 200% 30)",
            "oklch(0.5 0.8 30)"
        );
        $case!(
            lab_unbounded_axes_have_no_reference_ceiling,
            "lab(50 200% -300%)",
            "lab(50 250 -375)"
        );
        $case!(
            oklab_unbounded_axes_have_no_reference_ceiling,
            "oklab(.5 200% -300%)",
            "oklab(0.5 0.8 -1.2)"
        );
        $case!(
            lab_missing_lightness_stays_missing,
            "lab(none 0 0)",
            "lab(none 0 0)"
        );
        $case!(
            lab_missing_axis_does_not_disable_lightness_bound,
            "lab(125 none 0)",
            "lab(100 none 0)"
        );
        $case!(
            lch_missing_chroma_does_not_disable_lightness_bound,
            "lch(125 none 30)",
            "lch(100 none 30)"
        );
        $case!(
            oklch_missing_hue_does_not_disable_direct_bounds,
            "oklch(120% -20% none)",
            "oklch(1 0 none)"
        );
        $case!(
            oklab_missing_alpha_does_not_disable_lightness_bound,
            "oklab(120% 0 0 / none)",
            "oklab(1 0 0 / none)"
        );
        $case!(
            lab_calculated_lightness_remains_unbounded_wrapper,
            "lab(calc(125) 0 0)",
            "lab(calc(125) 0 0)"
        );
        $case!(
            lch_calculated_chroma_remains_negative_wrapper,
            "lch(50 calc(-20) 30)",
            "lch(50 calc(-20) 30)"
        );
        $case!(
            oklab_calculated_lightness_remains_scaled_wrapper,
            "oklab(calc(120%) 0 0)",
            "oklab(calc(1.2) 0 0)"
        );
        $case!(
            oklch_calculated_chroma_remains_scaled_wrapper,
            "oklch(.5 calc(-20%) 30)",
            "oklch(0.5 calc(-0.08) 30)"
        );
        $case!(
            lch_direct_lightness_beside_calculated_chroma_clips,
            "lch(125 calc(-20) 30)",
            "lch(100 calc(-20) 30)"
        );
        $case!(
            lab_direct_lightness_beside_numeric_axis_clips,
            "lab(125 calc(1 / 128) 0)",
            "lab(100 calc(0.0078125) 0)"
        );
        $case!(
            lab_direct_lightness_beside_contextual_axis_clips,
            "lab(125 calc(1em / 1px) 0)",
            "lab(100 calc(1 * 1em / 1px) 0)"
        );
        $case!(
            oklch_direct_chroma_beside_contextual_lightness_clips,
            "oklch(calc(1em / 1px) -20% 30)",
            "oklch(calc(1 * 1em / 1px) 0 30)"
        );
        $case!(
            contextual_axis_explicit_identity_scale_control,
            "lab(50 calc(1em / 1px) 0)",
            "lab(50 calc(1 * 1em / 1px) 0)"
        );
        $case!(
            lab_contextual_lightness_keeps_dimensions,
            "lab(calc(125 * 1em / 1px) 0 0)",
            "lab(calc(125 * 1em / 1px) 0 0)"
        );
        $case!(
            lab_calculated_alpha_keeps_wrapper_beside_direct_bound,
            "lab(125 0 0 / calc(.5))",
            "lab(100 0 0 / calc(0.5))"
        );
        $case!(
            lab_origin_preserves_unclamped_percentages,
            "alpha(from lab(125% -20% 30%))",
            "alpha(from lab(125% -20% 30%))"
        );
        $case!(
            lch_origin_preserves_negative_chroma,
            "alpha(from lch(125% -20% 30))",
            "alpha(from lch(125% -20% 30))"
        );
        $case!(
            oklab_origin_preserves_unclamped_lightness,
            "alpha(from oklab(120% 150% -200%))",
            "alpha(from oklab(120% 150% -200%))"
        );
        $case!(
            oklch_origin_preserves_negative_chroma,
            "alpha(from oklch(-.2 -20% 30))",
            "alpha(from oklch(-0.2 -20% 30))"
        );
        $case!(
            lab_relative_channel_keeps_125,
            "lab(from red 125 a b)",
            "lab(from red 125 a b)"
        );
        $case!(
            lch_relative_chroma_keeps_negative,
            "lch(from red l -20 h)",
            "lch(from red l -20 h)"
        );
        $case!(
            oklab_relative_percentage_keeps_out_of_range_number,
            "oklab(from red 120% a b)",
            "oklab(from red 1.2 a b)"
        );
        $case!(
            oklch_relative_chroma_keeps_negative,
            "oklch(from red l -20% h)",
            "oklch(from red l -0.08 h)"
        );
        $case!(
            predefined_rgb_keeps_out_of_range_channels,
            "color(display-p3 -20% 120% 50%)",
            "color(display-p3 -0.2 1.2 0.5)"
        );
        $case!(
            predefined_xyz_keeps_out_of_range_channels,
            "color(xyz -20% 120% 50%)",
            "color(xyz-d65 -0.2 1.2 0.5)"
        );
        $case!(
            custom_profile_keeps_unbounded_channels,
            "color(--P -20% 120%)",
            "color(--P -0.2 1.2)"
        );
        $case!(
            ordinary_rgb_keeps_existing_bound,
            "rgb(300 -20 0)",
            "rgb(255, 0, 0)"
        );
    };
}
pub(crate) use lab_direct_component_cases;
