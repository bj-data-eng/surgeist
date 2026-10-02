#![forbid(unsafe_code)]
//! Ordinary authored RGB/hex contracts from Color 4 CRD 2026-09-08 §§5.1/5.2,
//! §4.2 and §§16.1/16.2.2, independently of implementation output.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#rgb-functions
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hex-notation
//! Source SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Calculated alpha, Origin, missing-component output precision and opaque hex
//! output budgets have separate focused tests; this module characterizes literals.

use surgeist_css::{
    CssColor, CssColorComponent, CssColorConstructionError, CssColorNumberLiteral,
    CssColorPercentageLiteral, CssColorSyntax, CssHexColor, CssKnownProperty,
    CssKnownPropertyValueRef, CssRecoveryAction, CssRgbColor, parse_component_values,
    parse_style_attribute, validate_style_attribute,
};

fn parsed_color(text: &str) -> CssColor {
    let source = format!("color: {text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    let CssKnownPropertyValueRef::Color(color) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("typed color: {source}");
    };
    let color = color.value().clone();
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    color
}

fn component(text: &str) -> CssColorComponent {
    if text == "none" {
        return CssColorComponent::None;
    }
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1, "{text}");
    let token = values.items()[0].clone();
    if text.ends_with('%') {
        CssColorComponent::Percentage(CssColorPercentageLiteral::try_from_component(token).unwrap())
    } else {
        CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
    }
}

fn assert_component(value: &CssColorComponent, expected: &str) {
    match value {
        CssColorComponent::None => assert_eq!(expected, "none"),
        CssColorComponent::Number(value) => {
            assert!(!expected.ends_with('%'), "{expected}");
            assert_eq!(value.numeric().representation(), expected);
        }
        CssColorComponent::Percentage(value) => {
            assert_eq!(
                value.numeric().representation(),
                expected.strip_suffix('%').unwrap()
            );
        }
        other => panic!("expected direct literal {expected}, got {other:?}"),
    }
}

fn assert_rgb(color: &CssColor, syntax: CssColorSyntax, channels: [&str; 3], alpha: Option<&str>) {
    let rgb = color.rgb_value().unwrap();
    assert_eq!(rgb.syntax(), syntax);
    for (value, expected) in rgb.channels().iter().zip(channels) {
        assert_component(value, expected);
    }
    match (rgb.alpha(), alpha) {
        (None, None) => {}
        (Some(value), Some(expected)) => assert_component(value, expected),
        other => panic!("alpha omission/domain mismatch: {other:?}"),
    }
}

#[test]
fn rgb_and_rgba_aliases_accept_both_signatures_and_ordinary_alpha_domains() {
    for function in ["rgb", "rgba"] {
        for syntax in [CssColorSyntax::Legacy, CssColorSyntax::Modern] {
            for (channels, numeric) in [
                (["12.5", "20", "30"], "12.5, 20, 30"),
                (["10%", "20%", "30%"], "25.5, 51, 76.5"),
            ] {
                for alpha in [None, Some(".25"), Some("25%")] {
                    let separator = if syntax == CssColorSyntax::Legacy {
                        ", "
                    } else {
                        " "
                    };
                    let mut arguments = channels.join(separator);
                    if let Some(alpha) = alpha {
                        arguments.push_str(if syntax == CssColorSyntax::Legacy {
                            ", "
                        } else {
                            " / "
                        });
                        arguments.push_str(alpha);
                    }
                    let color = parsed_color(&format!("{function}({arguments})"));
                    assert_rgb(&color, syntax, channels, alpha);
                    let expected = if alpha.is_some() {
                        format!("rgba({numeric}, 0.25)")
                    } else {
                        format!("rgb({numeric})")
                    };
                    assert_eq!(color.to_specified_css().unwrap(), expected);
                }
            }
        }
    }
}

#[test]
fn checked_rgb_construction_retains_homogeneous_domains_and_alpha_before_output() {
    for syntax in [CssColorSyntax::Legacy, CssColorSyntax::Modern] {
        for (channels, numeric) in [
            (["12.5", "20", "30"], "12.5, 20, 30"),
            (["10%", "20%", "30%"], "25.5, 51, 76.5"),
        ] {
            for alpha in [None, Some(".25"), Some("25%")] {
                let rgb =
                    CssRgbColor::try_new(syntax, channels.map(component), alpha.map(component))
                        .unwrap();
                let color = CssColor::from_rgb(rgb);
                assert_rgb(&color, syntax, channels, alpha);
                let before = color.clone();
                let expected = if alpha.is_some() {
                    format!("rgba({numeric}, 0.25)")
                } else {
                    format!("rgb({numeric})")
                };
                assert_eq!(color.to_specified_css().unwrap(), expected);
                assert_eq!(color, before);
            }
        }
    }
}

#[test]
fn modern_alias_case_and_escapes_accept_mixed_domains_and_missing_components() {
    // 20% * 255 / 100 = 51; 12.5 remains fractional, without integer rounding.
    for function in ["rgb", "rgba", "RGB", "RgBa", "\\72 gb", "\\72 gba"] {
        let mixed = parsed_color(&format!("{function}(12.5 20% 30 / 25%)"));
        assert_rgb(
            &mixed,
            CssColorSyntax::Modern,
            ["12.5", "20%", "30"],
            Some("25%"),
        );
        assert_eq!(
            mixed.to_specified_css().unwrap(),
            "rgba(12.5, 51, 30, 0.25)"
        );
        let missing = parsed_color(&format!("{function}(none 20% 30 / none)"));
        assert_rgb(
            &missing,
            CssColorSyntax::Modern,
            ["none", "20%", "30"],
            Some("none"),
        );
    }
    let color = parsed_color("rgb(12.5 20% 30)");
    assert_rgb(&color, CssColorSyntax::Modern, ["12.5", "20%", "30"], None);
    assert_eq!(color.to_specified_css().unwrap(), "rgb(12.5, 51, 30)");
    let constructed = CssColor::from_rgb(
        CssRgbColor::try_new(
            CssColorSyntax::Modern,
            [component("none"), component("20%"), component("30")],
            Some(CssColorComponent::None),
        )
        .unwrap(),
    );
    assert_rgb(
        &constructed,
        CssColorSyntax::Modern,
        ["none", "20%", "30"],
        Some("none"),
    );
}

#[test]
fn ordinary_endpoint_projection_clamps_output_without_changing_literal_ranges() {
    for (text, syntax, channels, alpha, expected) in [
        (
            "rgb(-10 300 50% / 150%)",
            CssColorSyntax::Modern,
            ["-10", "300", "50%"],
            "150%",
            "rgb(0, 255, 127.5)",
        ),
        (
            "rgba(-10%, 120%, 50%, -.2)",
            CssColorSyntax::Legacy,
            ["-10%", "120%", "50%"],
            "-.2",
            "rgba(0, 255, 127.5, 0)",
        ),
        (
            "rgba(0 255 100% / 100%)",
            CssColorSyntax::Modern,
            ["0", "255", "100%"],
            "100%",
            "rgb(0, 255, 255)",
        ),
    ] {
        let constructed = CssColor::from_rgb(
            CssRgbColor::try_new(syntax, channels.map(component), Some(component(alpha))).unwrap(),
        );
        for color in [parsed_color(text), constructed] {
            assert_rgb(&color, syntax, channels, Some(alpha));
            let before = color.clone();
            assert_eq!(color.to_specified_css().unwrap(), expected);
            assert_eq!(color, before);
            assert_rgb(&color, syntax, channels, Some(alpha));
        }
    }
}

fn assert_recovered(value: &str) {
    let source = format!("color: {value}; opacity: 0.5");
    let report = parse_style_attribute(&source);
    assert_eq!(
        report.syntax().len(),
        1,
        "{source}: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.diagnostics().len(), 1, "{source}");
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration,
        "{source}"
    );
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity,
        "{source}"
    );
    assert!(validate_style_attribute(&source).is_err(), "{source}");
}

#[test]
fn malformed_rgb_signatures_drop_only_the_color_and_fail_strict_validation() {
    for function in ["rgb", "rgba"] {
        for arguments in [
            "",
            "1 2",
            "1 2 3 4",
            "1, 2",
            "1, 2, 3, 4, 5",
            "1, 2 3",
            "1 2, 3",
            "1,, 2, 3",
            "1, 2, 3 / .5",
            "1 2 3, .5",
            "1 2 3 /",
            "1 2 3 / .5 .6",
            "1 2 3 / .5 / .6",
            "none, 2, 3",
            "1, none, 3",
            "1, 2, none",
            "1, 2, 3, none",
            "1%, 2, 3",
            "1, 2%, 3",
            "1, 2, 3%",
            "1deg 2 3",
        ] {
            assert_recovered(&format!("{function}({arguments})"));
        }
    }
}

#[test]
fn checked_legacy_rgb_rejects_missing_and_mixed_channel_domains() {
    for channels in [
        ["none", "2", "3"],
        ["1", "none", "3"],
        ["1", "2", "none"],
        ["1%", "2", "3"],
        ["1", "2%", "3"],
        ["1", "2", "3%"],
        ["1", "2%", "3%"],
        ["1%", "2", "3%"],
        ["1%", "2%", "3"],
    ] {
        assert_eq!(
            CssRgbColor::try_new(CssColorSyntax::Legacy, channels.map(component), None)
                .unwrap_err(),
            CssColorConstructionError::InvalidSyntax
        );
    }
    for channels in [["1", "2", "3"], ["1%", "2%", "3%"]] {
        assert_eq!(
            CssRgbColor::try_new(
                CssColorSyntax::Legacy,
                channels.map(component),
                Some(CssColorComponent::None)
            )
            .unwrap_err(),
            CssColorConstructionError::InvalidSyntax
        );
    }
}

#[test]
fn all_hex_lengths_decode_mixed_case_channels_and_nonopaque_alpha() {
    // Repeated a/b/c nibbles give 170/187/204; a1/b2/c3 bytes give 161/178/195.
    // Alpha nibble 6 repeats to 0x66 = 102, and 102/255 is exactly 0.4.
    for (digits, expected) in [
        ("aBc", "rgb(170, 187, 204)"),
        ("aBc6", "rgba(170, 187, 204, 0.4)"),
        ("a1B2c3", "rgb(161, 178, 195)"),
        ("a1B2c366", "rgba(161, 178, 195, 0.4)"),
    ] {
        let constructed = CssColor::from_hex(CssHexColor::try_new(digits).unwrap());
        for color in [parsed_color(&format!("#{digits}")), constructed] {
            assert_eq!(color.hex_value().unwrap().digits(), digits);
            let before = color.clone();
            assert_eq!(color.to_specified_css().unwrap(), expected);
            assert_eq!(color, before);
        }
    }
}

#[test]
fn invalid_hex_lengths_and_digits_are_rejected_by_parser_and_constructor() {
    for digits in [
        "",
        "a",
        "ab",
        "abcde",
        "abcdefg",
        "abcdefghi",
        "abg",
        "a_bc",
        "abcé",
        "１２３",
    ] {
        assert!(CssHexColor::try_new(digits).is_none(), "{digits}");
        assert_recovered(&format!("#{digits}"));
    }
}
