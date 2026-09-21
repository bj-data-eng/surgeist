#![forbid(unsafe_code)]
//! Independent expectations for Unit E canonical specified colors.

use surgeist_css::*;

fn color(text: &str) -> CssAuthoredColor {
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("{text}: {error:?}"));
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.current().clone()
}

#[test]
fn standalone_color4_branches_have_source_derived_canonical_text() {
    for (source, expected) in [
        ("pUrPlE", "purple"),
        ("hsl(0 100% 50%)", "rgb(255, 0, 0)"),
        ("hsl(1 50% 50%)", "rgb(191.25, 65.875, 63.75)"),
        ("hsl(359 50% 50%)", "rgb(191.25, 63.75, 65.875)"),
        ("hsl(-1 50% 50%)", "rgb(191.25, 63.75, 65.875)"),
        ("hwb(0 0% 0%)", "rgb(255, 0, 0)"),
        ("hwb(0 6.27451% 0%)", "rgb(255, 16.000001, 16.000001)"),
        ("hwb(0 20% 80%)", "rgb(51, 51, 51)"),
        ("rgb(calc(100 * 4) 127 calc(20 - 35))", "rgb(255, 127, 0)"),
        ("rgb(calc(50%) 0 0)", "rgb(127.5, 0, 0)"),
        (
            "rgb(calc(25%) calc(50%) calc(75%))",
            "rgb(63.75, 127.5, 191.25)",
        ),
        ("rgb(none 0 255)", "color(srgb none 0 1)"),
        ("hsl(none 50% 50%)", "hsl(none 50% 50%)"),
        ("hsl(none calc(50) 50%)", "hsl(none calc(50%) 50%)"),
        ("hwb(30 -20% -30%)", "rgb(255, 140.25, 0)"),
        ("rgb(1e400 -1e400 50%)", "rgb(255, 0, 127.5)"),
        ("hsl(0 1e400% 50%)", "rgb(255, 0, 0)"),
        ("hwb(0 1e400% 1e400%)", "rgb(127.5, 127.5, 127.5)"),
        (
            "hwb(0 1e400% 1e401%)",
            "rgb(23.181818, 23.181818, 23.181818)",
        ),
        ("hsl(0 0% 6.27451%)", "rgb(16.000001, 16.000001, 16.000001)"),
        (
            "hsl(0 50% 17.51634%)",
            "rgb(67.000001, 22.333334, 22.333334)",
        ),
        (
            "rgb(99.999999999999999999999999999999% 0 0)",
            "rgb(254.99999999999999999999999999999745, 0, 0)",
        ),
        ("rgb(calc(infinity) 0 0)", "rgb(255, 0, 0)"),
        ("hsl(0 calc(infinity) 50%)", "rgb(255, 0, 0)"),
        ("rgb(1e2000000 0 0)", "rgb(255, 0, 0)"),
        ("hsl(0 1e2000000% 50%)", "rgb(255, 0, 0)"),
    ] {
        assert_eq!(
            color(source)
                .to_specified_css()
                .unwrap_or_else(|error| panic!("{source}: {error:?}")),
            expected,
            "{source}"
        );
    }
}

#[test]
fn admitted_huge_finite_exponents_fail_atomically_instead_of_panicking() {
    let value = color("hsl(0 12345678901234567891e170141183460469231731687303715884105727% 50%)");
    let before = value.clone();
    assert_eq!(
        value.to_specified_css().unwrap_err().kind(),
        CssSpecifiedValueSerializationErrorKind::CapacityOverflow
    );
    assert_eq!(value, before);
}

#[test]
fn origins_relative_alpha_and_custom_profiles_keep_declared_identity() {
    for (source, expected) in [
        ("alpha(from red)", "alpha(from red)"),
        ("alpha(from red / 50%)", "alpha(from red / 0.5)"),
        ("alpha(from red / none)", "alpha(from red / none)"),
        (
            "alpha(from rgb(1 2 3 / .2) / 1)",
            "alpha(from rgb(1 2 3 / 0.2) / 1)",
        ),
        (
            "rgb(from rgb(1 2 3 / .2) r g b / 1)",
            "rgb(from rgb(1 2 3 / 0.2) r g b / 1)",
        ),
        ("color(--P 0% 70% 20% 0%)", "color(--P 0 0.7 0.2 0)"),
        (
            "color(from red --P Cyan / alpha)",
            "color(from red --P Cyan / alpha)",
        ),
        (
            "alpha(from RGB(300, 0, 0, 120%))",
            "alpha(from rgb(300 0 0 / 120%))",
        ),
        (
            "alpha(from hsl(1turn 50% 50%))",
            "alpha(from hsl(360deg 50% 50%))",
        ),
    ] {
        assert_eq!(
            color(source)
                .to_specified_css()
                .unwrap_or_else(|error| panic!("{source}: {error:?}")),
            expected,
            "{source}"
        );
    }
}

#[test]
fn mix_serialization_preserves_declared_weights_and_fills_only_known_omissions() {
    for (source, expected) in [
        (
            "color-mix(in oklab, teal 50%, peru 50%)",
            "color-mix(teal, peru)",
        ),
        (
            "color-mix(teal 70%, peru 70%)",
            "color-mix(teal 70%, peru 70%)",
        ),
        (
            "color-mix(red 70%, blue 70%, green)",
            "color-mix(red 70%, blue 70%, green 0%)",
        ),
        (
            "color-mix(red 50%, green, blue)",
            "color-mix(red 50%, green 25%, blue 25%)",
        ),
        (
            "color-mix(red calc(50%), blue)",
            "color-mix(red calc(50%), blue)",
        ),
        (
            "color-mix(in --P, red, blue)",
            "color-mix(in --P, red, blue)",
        ),
        (
            "color-mix(red 60%, green, blue, yellow)",
            "color-mix(red 60%, green 13.333333%, blue 13.333333%, yellow 13.333333%)",
        ),
    ] {
        assert_eq!(
            color(source)
                .to_specified_css()
                .unwrap_or_else(|error| panic!("{source}: {error:?}")),
            expected,
            "{source}"
        );
    }
}

#[test]
fn deferred_hsl_hwb_radian_and_alpha_order_follow_adopted_policies() {
    for (source, expected) in [
        (
            "hsl(calc(1em / 1px) -20% 50% / 120%)",
            "hsl(calc(1em / 1px) 0% 50%)",
        ),
        (
            "hwb(calc(1em / 1px) 20 30 / .5)",
            "hwb(calc(1em / 1px) 20 30 / 0.5)",
        ),
        ("hwb(0 calc(1em / 1px) 20%)", "hwb(0 calc(1em / 1px) 20%)"),
        ("hwb(0 20% calc(1em / 1px))", "hwb(0 20% calc(1em / 1px))"),
        (
            "hwb(calc(1em / 1px) 40% 80%)",
            "hwb(calc(1em / 1px) 40% 80%)",
        ),
        (
            "hwb(calc(1em / 1px) -20% 30)",
            "hwb(calc(1em / 1px) -20% 30)",
        ),
        (
            "hsl(0 100% 50% / calc(1em / 1px))",
            "rgba(255, 0, 0, calc(1em / 1px))",
        ),
        (
            "alpha(from hsl(1rad 50% 50%))",
            "alpha(from hsl(57.29577951308232286464772187173366546630859375deg 50% 50%))",
        ),
        (
            "hsl(calc(1em / 1px) calc(-20%) 50%)",
            "hsl(calc(1em / 1px) calc(-20%) 50%)",
        ),
        ("color(srgb 1 0 0 / 0.9999996)", "color(srgb 1 0 0 / 1)"),
        ("color(srgb 1 0 0 / 1)", "color(srgb 1 0 0)"),
    ] {
        assert_eq!(
            color(source)
                .to_specified_css()
                .unwrap_or_else(|error| panic!("{source}: {error:?}")),
            expected,
            "{source}"
        );
    }
}

#[test]
fn exact_scales_and_legacy_byte_alpha_follow_the_selected_numeric_phases() {
    for (source, expected) in [
        ("#000000ec", "rgba(0, 0, 0, 0.925)"),
        ("#000000ed", "rgba(0, 0, 0, 0.93)"),
        ("color(srgb 100% 50% 0%)", "color(srgb 1 0.5 0)"),
        ("color(srgb calc(50%) 0 0)", "color(srgb calc(0.5) 0 0)"),
        ("lab(100% 100% -100%)", "lab(100 125 -125)"),
        ("lab(calc(50%) 0 0)", "lab(calc(50) 0 0)"),
        ("lch(100% 100% 0)", "lch(100 150 0)"),
        ("oklab(100% 100% -100%)", "oklab(1 0.4 -0.4)"),
        ("oklch(100% 100% 0)", "oklch(1 0.4 0)"),
        ("alpha(from red / calc(50%))", "alpha(from red / calc(0.5))"),
    ] {
        assert_eq!(
            color(source)
                .to_specified_css()
                .unwrap_or_else(|error| panic!("{source}: {error:?}")),
            expected,
            "{source}"
        );
    }
}

#[test]
fn simple_keyword_limits_have_exact_resource_boundaries() {
    let value = color("purple");
    assert_eq!(
        value
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 6))
            .unwrap(),
        "purple"
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, 6),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, 6),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 1, 5),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }

    let explicit_default = color("color-mix(in oklab, red, blue)");
    assert_eq!(
        explicit_default
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                4,
                usize::MAX,
                usize::MAX,
            ))
            .unwrap(),
        "color-mix(red, blue)"
    );
    assert_eq!(
        explicit_default
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                usize::MAX,
                usize::MAX,
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
}

#[test]
fn limits_fail_atomically_without_mutating_the_authored_graph() {
    let value = color("color-mix(red 50%, blue 50%)");
    let before = value.clone();
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, usize::MAX, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, 0, usize::MAX),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(value, before);
    }
}

#[test]
fn canonical_outputs_reenter_the_color_grammar_without_mutating_successful_inputs() {
    for source in [
        "rgb(none 0 255)",
        "hsl(calc(1em / 1px) -20% 50% / 120%)",
        "color(from red --P Cyan / alpha)",
        "color-mix(red 60%, green, blue, yellow)",
    ] {
        let value = color(source);
        let before = value.clone();
        let first = value.to_specified_css().unwrap();
        assert_eq!(value, before, "successful serialization changed {source}");
        let reparsed = color(&first);
        assert_eq!(
            reparsed.to_specified_css().unwrap(),
            first,
            "stable canonical output for {source}"
        );
    }

    let rounded = color("color(srgb 1 0 0 / 0.9999996)")
        .to_specified_css()
        .unwrap();
    assert_eq!(rounded, "color(srgb 1 0 0 / 1)");
    assert_eq!(
        color(&rounded).to_specified_css().unwrap(),
        "color(srgb 1 0 0)"
    );
}

#[test]
fn wide_and_deep_graphs_share_one_cumulative_resource_context() {
    let wide_source = format!("color-mix({})", vec!["red"; 64].join(", "));
    let wide = color(&wide_source);
    assert!(wide.to_specified_css().unwrap().starts_with("color-mix("));
    assert_eq!(
        wide.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
            32,
            usize::MAX,
            usize::MAX,
        ))
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );

    let mut deep_source = "red".to_owned();
    for _ in 0..64 {
        deep_source = format!("color-mix(red, {deep_source})");
    }
    let deep = color(&deep_source);
    let before = deep.clone();
    assert!(deep.to_specified_css().unwrap().starts_with("color-mix("));
    assert_eq!(deep, before);
}
