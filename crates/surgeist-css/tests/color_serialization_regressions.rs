#![forbid(unsafe_code)]

use surgeist_css::*;

fn color(text: &str) -> CssAuthoredColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected color");
    };
    value.current().clone()
}

#[test]
fn positive_fractional_rgb_channels_remain_above_zero() {
    for (source, expected) in [
        ("rgb(0.1 0.5 0.9)", "rgb(0.1, 0.5, 0.9)"),
        ("rgb(-0.1 0 0.1)", "rgb(0, 0, 0.1)"),
        ("rgb(0.01% 0 0)", "rgb(0.0255, 0, 0)"),
    ] {
        assert_eq!(
            color(source).to_specified_css().unwrap(),
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
            "color(srgb calc(0.5 * 1em / 1px) 0 0)",
        ),
        (
            "alpha(from red / calc(1em / 1px * 50%))",
            "alpha(from red / calc(0.5 * 1em / 1px))",
        ),
    ] {
        let output = color(source).to_specified_css().unwrap();
        // 50% of the unit reference is 0.5; contextual unit ratios stay symbolic.
        assert_eq!(
            color(&output).to_specified_css().unwrap(),
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
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn projection_budget_never_relaxes_color_rounding() {
    let value = color("hsl(0 0% 6.27451%)");
    // 255 * 6.27451 / 100 = 16.0000005: a tie rounds toward positive infinity.
    let expected = "rgb(16.000001, 16.000001, 16.000001)";
    assert_eq!(value.to_specified_css().unwrap(), expected);
    for budget in [50, 100, 150, 200, 300, 500, 1000, 10000] {
        match value.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
            10000, budget, 10000,
        )) {
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
            color(source).to_specified_css().unwrap(),
            expected,
            "{source}"
        );
    }
}
