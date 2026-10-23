use crate::css::support::checked_color as color;
use surgeist_css::*;

#[test]
fn positive_fractional_rgb_channels_remain_above_zero() {
    for (source, expected) in [
        ("rgb(0.1 0.5 0.9)", "rgb(0.1, 0.5, 0.9)"),
        ("rgb(-0.1 0 0.1)", "rgb(0, 0, 0.1)"),
        ("rgb(0.01% 0 0)", "rgb(0.0255, 0, 0)"),
    ] {
        assert_eq!(
            color(source)
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn symbolic_percentage_conversion_preserves_grammar_and_value() {
    for (source, expected) in [
        (
            "color(srgb calc(1em / 1px * 50%) 0 0)",
            "color(srgb calc(0.5) 0 0)",
        ),
        (
            "alpha(from red / calc(1em / 1px * 50%))",
            "alpha(from red / calc(0.5))",
        ),
    ] {
        let output = color(source)
            .serialize_specified(Default::default())
            .unwrap();
        // Reentry must succeed while the ratio stays symbolic. Independently
        // substitute equal units: 50% of the unit reference must then be 0.5.
        color(&output);
        assert!(output.contains("1em") && output.contains("1px"));
        assert_eq!(
            color(&output.replace("1em", "1px"))
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{source}: {output}"
        );
    }
}

#[test]
fn missing_rgb_channels_round_repeating_ratios_without_resource_errors() {
    for (source, expected) in [
        ("rgb(none 1 255)", "color(srgb none 0.003922 1)"),
        ("rgb(254 none 51)", "color(srgb 0.996078 none 0.2)"),
    ] {
        assert_eq!(
            color(source)
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn projection_budget_never_relaxes_color_rounding() {
    let value = color("hsl(0 0% 6.27451%)");
    // The exact product 255 * 6.27451 / 100 = 16.0000005 is a decimal tie, but
    // surgeist-color converts the binary64 lightness, whose product lies just
    // below it, exactly as used-value serialization does.
    let expected = "rgb(16, 16, 16)";
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        expected
    );
    for budget in [50, 100, 150, 200, 300, 500, 1000, 10000] {
        match value.serialize_specified(CssSerializationLimits::new(10000, budget, 10000)) {
            Ok(output) => assert_eq!(output, expected, "budget {budget}"),
            Err(error) => assert_eq!(
                error.kind(),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            ),
        }
    }
}

#[test]
fn missing_hsl_hwb_use_percentages_for_exact_number_channels() {
    for (source, expected) in [
        ("hsl(none 0.1 50)", "hsl(none 0.1% 50%)"),
        ("hwb(none 0.1 20)", "hwb(none 0.1% 20%)"),
        ("hsl(none 50 0.1)", "hsl(none 50% 0.1%)"),
    ] {
        assert_eq!(
            color(source)
                .serialize_specified(Default::default())
                .unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn huge_saturation_vertices_keep_exact_clipped_endpoints() {
    for (angle, expected) in [
        (0, "rgb(255, 0, 0)"),
        (60, "rgb(255, 255, 0)"),
        (120, "rgb(0, 255, 0)"),
        (180, "rgb(0, 255, 255)"),
        (240, "rgb(0, 0, 255)"),
        (300, "rgb(255, 0, 255)"),
    ] {
        let source = format!("hsl({angle} 1e2000000% 50%)");
        assert_eq!(
            color(&source)
                .serialize_specified(Default::default())
                .unwrap(),
            expected
        );
    }
}
