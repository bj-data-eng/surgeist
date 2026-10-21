//! Ordinary authored Lab/LCH/Oklab/Oklch from Color 4 CRD 2026-09-08 §§9.3–9.4,
//! with selected specified projection contracts from §§16.3–16.4 and the
//! repository's css-color-4-lab-ok-calculated-range-phase disposition.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#specifying-lab-lch
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#specifying-oklab-oklch
//! Source SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Explicit numeric oracles use Lab scales 100/125/150 and OK scales 1/0.4,
//! degree conversion and the selected direct bounds, never captured output.

use crate::css::support::color_components::{
    assert_component, assert_origin, assert_recovered, component, token,
};
use crate::css::support::parsed_color as parsed;
use surgeist_css::*;

const FAMILIES: [&str; 4] = ["lab", "lch", "oklab", "oklch"];

fn hue(text: &str) -> CssColorHue {
    if text.eq_ignore_ascii_case("none") {
        CssColorHue::None
    } else if text.starts_with("calc(") {
        let values = parse_component_values(text).unwrap();
        if ["deg", "grad", "rad", "turn"]
            .iter()
            .any(|unit| text.contains(unit))
        {
            CssColorHue::AngleCalculation(CssAngleCalculation::try_from_components(values).unwrap())
        } else {
            CssColorHue::NumberCalculation(
                CssNumberCalculation::try_from_components(values).unwrap(),
            )
        }
    } else if ["deg", "grad", "rad", "turn"]
        .iter()
        .any(|unit| text.ends_with(unit))
    {
        CssColorHue::Angle(CssAngleLiteral::try_from_component(token(text)).unwrap())
    } else {
        CssColorHue::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
    }
}

fn constructed(family: &str, channels: [&str; 3], alpha: Option<&str>) -> CssColor {
    let [lightness, second, third] = channels;
    match family {
        "lab" | "oklab" => {
            let value = CssLabColor::try_new(
                component(lightness),
                component(second),
                component(third),
                alpha.map(component),
            )
            .unwrap();
            if family == "lab" {
                CssColor::from_lab(value)
            } else {
                CssColor::from_oklab(value)
            }
        }
        "lch" | "oklch" => {
            let value = CssLchColor::try_new(
                component(lightness),
                component(second),
                hue(third),
                alpha.map(component),
            )
            .unwrap();
            if family == "lch" {
                CssColor::from_lch(value)
            } else {
                CssColor::from_oklch(value)
            }
        }
        _ => panic!("unknown test family {family}"),
    }
}

fn assert_hue(value: &CssColorHue, expected: &str) {
    match value {
        CssColorHue::None => assert!(expected.eq_ignore_ascii_case("none")),
        CssColorHue::Number(value) => {
            assert_eq!(value.numeric().representation(), expected);
            assert_origin(value.origin(), expected);
        }
        CssColorHue::Angle(value) => {
            let (coefficient, unit) = if let Some(value) = expected.strip_suffix("deg") {
                (value, CssAngleUnit::Degrees)
            } else if let Some(value) = expected.strip_suffix("grad") {
                (value, CssAngleUnit::Gradians)
            } else if let Some(value) = expected.strip_suffix("turn") {
                (value, CssAngleUnit::Turns)
            } else {
                (expected.strip_suffix("rad").unwrap(), CssAngleUnit::Radians)
            };
            assert_eq!(value.numeric().representation(), coefficient);
            assert_eq!(value.unit(), unit);
            assert_origin(value.origin(), expected);
        }
        CssColorHue::NumberCalculation(value) => {
            assert_eq!(value.components().serialize().unwrap().as_css(), expected);
            assert_origin(value.origin(), expected);
        }
        CssColorHue::AngleCalculation(value) => {
            assert_eq!(value.components().serialize().unwrap().as_css(), expected);
            assert_origin(value.origin(), expected);
        }
        other => panic!("unexpected hue {other:?}"),
    }
}

fn assert_fields(color: &CssColor, family: &str, channels: [&str; 3], alpha: Option<&str>) {
    let actual_alpha = match family {
        "lab" | "oklab" => {
            let value = if family == "lab" {
                color.lab_value()
            } else {
                color.oklab_value()
            }
            .unwrap();
            for (value, expected) in [value.lightness(), value.a(), value.b()]
                .into_iter()
                .zip(channels)
            {
                assert_component(value, expected);
            }
            value.alpha()
        }
        "lch" | "oklch" => {
            let value = if family == "lch" {
                color.lch_value()
            } else {
                color.oklch_value()
            }
            .unwrap();
            assert_component(value.lightness(), channels[0]);
            assert_component(value.chroma(), channels[1]);
            assert_hue(value.hue(), channels[2]);
            value.alpha()
        }
        _ => panic!("unknown test family {family}"),
    };
    match (actual_alpha, alpha) {
        (None, None) => {}
        (Some(value), Some(expected)) => assert_component(value, expected),
        other => panic!("alpha omission/domain mismatch: {other:?}"),
    }
}

fn assert_case(family: &str, channels: [&str; 3], alpha: Option<&str>, expected: &str) {
    let mut arguments = channels.join(" ");
    if let Some(alpha) = alpha {
        arguments.push_str(&format!(" / {alpha}"));
    }
    let source = format!("{family}({arguments})");
    for color in [parsed(&source), constructed(family, channels, alpha)] {
        assert_fields(&color, family, channels, alpha);
        let before = color.clone();
        assert_eq!(color.to_specified_css().unwrap(), expected, "{source}");
        assert_eq!(color, before);
        assert_fields(&color, family, channels, alpha);
    }
}

#[test]
fn each_family_uses_its_percentage_reference_ranges_and_retains_alpha_domains() {
    for (family, channels, scaled) in [
        ("lab", ["100%", "-100%", "100%"], "100 -125 125"),
        ("lch", ["100%", "100%", "-90deg"], "100 150 270"),
        ("oklab", ["100%", "-100%", "100%"], "1 -0.4 0.4"),
        ("oklch", ["100%", "100%", "-90deg"], "1 0.4 270"),
    ] {
        for (alpha, suffix) in [
            (None, ""),
            (Some(".25"), " / 0.25"),
            (Some("25%"), " / 0.25"),
            (Some("none"), " / none"),
        ] {
            assert_case(
                family,
                channels,
                alpha,
                &format!("{family}({scaled}{suffix})"),
            );
        }
    }
}

#[test]
fn mixed_number_percentage_and_missing_channels_remain_distinct() {
    for (family, channels, expected) in [
        ("lab", ["50%", "20", "none"], "lab(50 20 none)"),
        ("lab", ["none", "-20%", "30"], "lab(none -25 30)"),
        ("lch", ["50%", "20", "none"], "lch(50 20 none)"),
        ("lch", ["none", "-20%", "30"], "lch(none 0 30)"),
        ("oklab", ["50%", "20", "none"], "oklab(0.5 20 none)"),
        ("oklab", ["none", "-20%", "30"], "oklab(none -0.08 30)"),
        ("oklch", ["50%", "20", "none"], "oklch(0.5 20 none)"),
        ("oklch", ["none", "-20%", "30"], "oklch(none 0 30)"),
    ] {
        assert_case(family, channels, None, expected);
    }
}

#[test]
fn function_case_escapes_and_none_case_preserve_the_same_family() {
    for family in FAMILIES {
        let escaped = format!("\\{:x} {}", family.as_bytes()[0], &family[1..]);
        for spelling in [family.to_owned(), family.to_ascii_uppercase(), escaped] {
            let color = parsed(&format!("{spelling}(NoNe NONE nOnE / NoNe)"));
            assert_fields(&color, family, ["none", "none", "none"], Some("none"));
            assert_eq!(
                color.to_specified_css().unwrap(),
                format!("{family}(none none none / none)")
            );
        }
        assert_case(
            family,
            ["none", "none", "none"],
            Some("none"),
            &format!("{family}(none none none / none)"),
        );
    }
}

#[test]
fn direct_bounds_preserve_raw_coefficients_and_unbounded_axes_or_chroma() {
    for (family, channels, alpha, expected) in [
        (
            "lab",
            ["125%", "-200%", "3000"],
            "150%",
            "lab(100 -250 3000)",
        ),
        (
            "lab",
            ["-2", "2000", "-300%"],
            "-25%",
            "lab(0 2000 -375 / 0)",
        ),
        (
            "oklab",
            ["125%", "-200%", "3000"],
            "150%",
            "oklab(1 -0.8 3000)",
        ),
        (
            "oklab",
            ["-2", "2000", "-300%"],
            "-25%",
            "oklab(0 2000 -1.2 / 0)",
        ),
        ("lch", ["125%", "-20%", "-90deg"], "150%", "lch(100 0 270)"),
        ("lch", ["-2", "3000", "-90"], "-25%", "lch(0 3000 270 / 0)"),
        (
            "oklch",
            ["125%", "-20%", "-90deg"],
            "150%",
            "oklch(1 0 270)",
        ),
        (
            "oklch",
            ["-2", "3000", "-.25turn"],
            "-25%",
            "oklch(0 3000 270 / 0)",
        ),
    ] {
        assert_case(family, channels, Some(alpha), expected);
    }
}

#[test]
fn polar_hues_retain_units_and_wrap_direct_values_without_erasing_present_hue() {
    // Color 4 §16 serializes a hue as a canonicalized <number> of degrees and
    // CSSOM rounds it to six decimals: 1rad = 180/π deg = 57.2957795…deg.
    for family in ["lch", "oklch"] {
        for (hue, degrees) in [
            ("-90", "270"),
            ("450deg", "90"),
            ("-100grad", "270"),
            ("1.25turn", "90"),
            ("1rad", "57.29578"),
            ("none", "none"),
        ] {
            assert_case(
                family,
                ["50%", "0", hue],
                None,
                &format!(
                    "{family}({} 0 {degrees})",
                    if family == "lch" { "50" } else { "0.5" }
                ),
            );
        }
    }
}

#[test]
fn calculations_keep_wrappers_and_outlier_values_while_percentage_scales_apply() {
    for (family, channels, expected) in [
        (
            "lab",
            ["calc(125%)", "calc(-200%)", "calc(3000)"],
            "lab(calc(125) calc(-250) calc(3000) / calc(1.5))",
        ),
        (
            "oklab",
            ["calc(125%)", "calc(-200%)", "calc(3000)"],
            "oklab(calc(1.25) calc(-0.8) calc(3000) / calc(1.5))",
        ),
        (
            "lch",
            ["calc(125%)", "calc(-20%)", "calc(450)"],
            "lch(calc(125) calc(-30) calc(450) / calc(1.5))",
        ),
        (
            "oklch",
            ["calc(125%)", "calc(-20%)", "calc(450)"],
            "oklch(calc(1.25) calc(-0.08) calc(450) / calc(1.5))",
        ),
    ] {
        assert_case(family, channels, Some("calc(150%)"), expected);
    }
    for family in ["lch", "oklch"] {
        let channels = ["50%", "20%", "calc(1turn - 90deg)"];
        for color in [
            parsed(&format!("{family}(50% 20% calc(1turn - 90deg))")),
            constructed(family, channels, None),
        ] {
            assert_fields(&color, family, channels, None);
            let before = color.clone();
            assert!(color.to_specified_css().unwrap().contains("calc("));
            assert_eq!(color, before);
        }
    }
}

#[test]
fn malformed_perceptual_functions_drop_only_color_and_fail_strict_validation() {
    for family in FAMILIES {
        for arguments in [
            "",
            "1 2",
            "1 2 3 4",
            "1, 2, 3",
            "1 2 3, .25",
            "1 2 3 /",
            "1 2 3 / .25 .5",
            "1 2 3 / .25 / .5",
            "1px 2 3",
            "1 2px 3",
            "1 2 3 / 1deg",
            "calc(1px) 2 3",
        ] {
            assert_recovered(&format!("{family}({arguments})"));
        }
        for invalid_third in if family.ends_with("lch") {
            ["30%", "1px", "1s", "calc(1%)"]
        } else {
            ["30deg", "1px", "1s", "calc(1deg)"]
        } {
            assert_recovered(&format!("{family}(1 2 {invalid_third})"));
        }
    }
}

#[test]
fn checked_scalar_front_doors_reject_dimensions_that_cannot_enter_the_models() {
    for text in ["1deg", "1px", "1s"] {
        assert_eq!(
            CssColorNumberLiteral::try_from_component(token(text))
                .unwrap_err()
                .kind(),
            CssComponentValueErrorKind::InvalidToken
        );
        assert_eq!(
            CssColorPercentageLiteral::try_from_component(token(text))
                .unwrap_err()
                .kind(),
            CssComponentValueErrorKind::InvalidToken
        );
    }
    assert_eq!(
        CssColorNumberLiteral::try_from_component(token("1%"))
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::InvalidToken
    );
    assert_eq!(
        CssColorPercentageLiteral::try_from_component(token("1"))
            .unwrap_err()
            .kind(),
        CssComponentValueErrorKind::InvalidToken
    );
    for text in ["1", "1%", "1px", "1s"] {
        assert_eq!(
            CssAngleLiteral::try_from_component(token(text))
                .unwrap_err()
                .kind(),
            CssComponentValueErrorKind::InvalidToken
        );
    }
    for text in ["calc(1px)", "calc(1deg)", "calc(1s)"] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
        assert!(
            CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
    }
    for text in ["calc(1px)", "calc(1%)", "calc(1s)"] {
        assert!(
            CssAngleCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
    }
}
