use surgeist_css::{CssMediaConditionKind, CssMediaQuery, parse_media_query};

// The selected MQ4/MQ5 grammar requires these to remain known feature syntax.
// In particular, MQ4 section 2.4.3 requires signed range operands; failure of
// an implementation's older value model must not turn them into unknowns.
// https://www.w3.org/TR/2026/CRD-mediaqueries-4-20260219/#mq-range-context
fn assert_known_feature(source: &str) {
    let report = parse_media_query(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let CssMediaQuery::Condition(condition) = report.syntax() else {
        panic!("{source}: expected a condition-only query");
    };
    assert!(
        matches!(condition.kind(), CssMediaConditionKind::Feature(_)),
        "{source}: expected a known feature, got {:?}",
        condition.kind()
    );
}

#[test]
fn signed_length_operands_remain_known_features() {
    for source in [
        "(width: -1px)",
        "(height <= -100px)",
        "(device-width: -0px)",
    ] {
        assert_known_feature(source);
    }
}

#[test]
fn signed_zero_infinite_and_alias_resolutions_remain_known_features() {
    for source in [
        "(resolution > -300dpi)",
        "(resolution: 0dppx)",
        "(resolution: infinite)",
        "(resolution >= 2x)",
    ] {
        assert_known_feature(source);
    }
}

#[test]
fn signed_integer_media_features_keep_their_known_domains() {
    for source in ["(color: -1)", "(monochrome: -1)", "(color-index: -1)"] {
        assert_known_feature(source);
    }
}

#[test]
fn value_first_and_chained_ranges_remain_known_without_evaluating_bounds() {
    for source in [
        "(-100px < width)",
        "(-100px < width <= 40em)",
        "(80em >= width > -100px)",
        "(100px < width < 10px)",
        "(-1 < monochrome)",
    ] {
        assert_known_feature(source);
    }
}

// Values 4 section 5.7 permits nonnegative numbers, an omitted denominator,
// and zero components; media parsing does not divide the ratio.
// https://www.w3.org/TR/2024/WD-css-values-4-20240312/#ratios
#[test]
fn media_ratios_admit_decimal_omitted_and_zero_components() {
    for source in [
        "(aspect-ratio: 1.5/2)",
        "(aspect-ratio: 2)",
        "(aspect-ratio: 0/0)",
        "(aspect-ratio: 1/0)",
        "(device-aspect-ratio: 0/2)",
    ] {
        assert_known_feature(source);
    }
}

// All unprefixed selected feature names admit the structural boolean form.
// https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#media-descriptor-table
#[test]
fn selected_media_feature_names_have_known_boolean_forms() {
    for name in [
        "hover",
        "any-hover",
        "pointer",
        "any-pointer",
        "width",
        "height",
        "device-width",
        "device-height",
        "aspect-ratio",
        "device-aspect-ratio",
        "resolution",
        "color",
        "color-index",
        "monochrome",
        "orientation",
        "scan",
        "grid",
        "horizontal-viewport-segments",
        "vertical-viewport-segments",
        "prefers-color-scheme",
        "prefers-reduced-motion",
        "prefers-reduced-transparency",
        "prefers-contrast",
        "forced-colors",
        "display-mode",
        "update",
        "overflow-block",
        "overflow-inline",
        "color-gamut",
        "video-color-gamut",
        "dynamic-range",
        "video-dynamic-range",
        "environment-blending",
        "inverted-colors",
        "nav-controls",
        "scripting",
        "prefers-reduced-data",
    ] {
        assert_known_feature(&format!("({name})"));
    }
}

#[test]
fn selected_keyword_families_are_known_media_features() {
    for source in [
        "(update: slow)",
        "(overflow-block: paged)",
        "(overflow-inline: scroll)",
        "(color-gamut: rec2020)",
        "(video-color-gamut: p3)",
        "(dynamic-range: high)",
        "(video-dynamic-range: standard)",
        "(environment-blending: additive)",
        "(inverted-colors: inverted)",
        "(nav-controls: back)",
        "(scripting: initial-only)",
        "(prefers-reduced-data: reduce)",
    ] {
        assert_known_feature(source);
    }
}

#[test]
fn intrinsic_numeric_expressions_remain_known_features() {
    for source in [
        "(width: calc(1px + 2em))",
        "(width: min(1px, 2em))",
        "(color: calc(1.5))",
        "(width: calc(1px * 2px / 1px))",
        "(width: calc(infinity * 1px))",
        "(resolution: calc(1dppx + 96dpi))",
    ] {
        assert_known_feature(source);
    }
}

#[test]
fn ordinary_known_features_remain_accepted() {
    for source in ["(width: 1px)", "(color: 8)", "(orientation: portrait)"] {
        assert_known_feature(source);
    }
}

#[test]
fn exact_media_literals_do_not_narrow_to_machine_numeric_ranges() {
    for source in [
        "(color: -123456789012345678901234567890)",
        "(width: 1e999px)",
        "(width: 1e-999px)",
        "(horizontal-viewport-segments >= 1)",
        "(vertical-viewport-segments: -1)",
        "(grid: -0)",
        "(width </**/= 10px)",
    ] {
        assert_known_feature(source);
    }
}

// Values 4 sections 10.9 and 10.12 admit correctly typed calculations before
// contextual range checks and integer rounding; no arithmetic is evaluated here.
#[test]
fn grid_and_ratio_math_remains_symbolic_until_resolution() {
    for source in [
        "(grid: calc(0.5))",
        "(grid: calc(2))",
        "(grid: calc(-1))",
        "(aspect-ratio: calc(1 + 1)/calc(2))",
        "(aspect-ratio: calc(-1)/2)",
        "(aspect-ratio: 2/calc(-1))",
        "(aspect-ratio: calc(2))",
    ] {
        assert_known_feature(source);
    }
}
