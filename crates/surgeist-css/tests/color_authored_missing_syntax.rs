#![forbid(unsafe_code)]
//! Focused ordinary authored syntax and missing identity from Color 4 CRD 2026-09-08
//! §§4.1.1–4.1.2, 4.4 and 7.1, supplementing the owning family grammar matrices.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-syntax-modern
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-syntax-legacy
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#missing
//! Source SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Zero-valued controls isolate missing identity from color conversion arithmetic.
//! Expected preserving text uses the source's none keyword and selected ordinary
//! family projections; no computed interpolation or calculated range phase is tested.

use surgeist_css::*;

#[derive(Clone, Copy)]
enum Family {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Predefined(CssPredefinedColorSpace),
}

const ORDINARY: [(&str, Family); 7] = [
    ("rgb", Family::Rgb),
    ("hsl", Family::Hsl),
    ("hwb", Family::Hwb),
    ("lab", Family::Lab),
    ("lch", Family::Lch),
    ("oklab", Family::Oklab),
    ("oklch", Family::Oklch),
];

const SPACES: [(&str, CssPredefinedColorSpace, &str); 10] = [
    ("srgb", CssPredefinedColorSpace::Srgb, "srgb"),
    (
        "srgb-linear",
        CssPredefinedColorSpace::SrgbLinear,
        "srgb-linear",
    ),
    (
        "display-p3",
        CssPredefinedColorSpace::DisplayP3,
        "display-p3",
    ),
    (
        "display-p3-linear",
        CssPredefinedColorSpace::DisplayP3Linear,
        "display-p3-linear",
    ),
    ("a98-rgb", CssPredefinedColorSpace::A98Rgb, "a98-rgb"),
    (
        "prophoto-rgb",
        CssPredefinedColorSpace::ProphotoRgb,
        "prophoto-rgb",
    ),
    ("rec2020", CssPredefinedColorSpace::Rec2020, "rec2020"),
    ("xyz", CssPredefinedColorSpace::XyzD65, "xyz-d65"),
    ("xyz-d50", CssPredefinedColorSpace::XyzD50, "xyz-d50"),
    ("xyz-d65", CssPredefinedColorSpace::XyzD65, "xyz-d65"),
];

fn token(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1);
    values.items()[0].clone()
}
fn component(text: &str) -> CssColorComponent {
    if text == "none" {
        CssColorComponent::None
    } else if text.ends_with('%') {
        CssColorComponent::Percentage(
            CssColorPercentageLiteral::try_from_component(token(text)).unwrap(),
        )
    } else {
        CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
    }
}
fn hue(text: &str) -> CssColorHue {
    if text == "none" {
        CssColorHue::None
    } else if text.ends_with("turn") {
        CssColorHue::Angle(CssAngleLiteral::try_from_component(token(text)).unwrap())
    } else {
        CssColorHue::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
    }
}

fn parsed(text: &str) -> CssColor {
    let source = format!("color:{text}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    assert_eq!(validate_style_attribute(&source).unwrap(), *report.syntax());
    let declaration = &report.syntax()[0];
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("parsed function origin: {source}");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed color: {source}");
    };
    value.value().clone()
}

fn assert_origin(origin: &CssValueOrigin, text: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("actual token provenance: {text}")
    };
    assert!(
        origin.source().as_str().contains(text),
        "{text}: {origin:?}"
    );
}

fn assert_component(value: &CssColorComponent, expected: &str) {
    match value {
        CssColorComponent::None => assert!(expected.eq_ignore_ascii_case("none")),
        CssColorComponent::Number(value) => {
            assert_eq!(value.numeric().representation(), expected);
            assert_origin(value.origin(), expected);
        }
        CssColorComponent::Percentage(value) => {
            assert_eq!(
                value.numeric().representation(),
                expected.strip_suffix('%').unwrap()
            );
            assert_origin(value.origin(), expected);
        }
        CssColorComponent::NumberCalculation(value) => {
            assert!(!expected.contains('%'));
            assert_eq!(value.components().serialize().unwrap().as_css(), expected);
            assert_origin(value.origin(), expected);
        }
        CssColorComponent::PercentageCalculation(value) => {
            assert!(expected.contains('%'));
            assert_eq!(value.components().serialize().unwrap().as_css(), expected);
            assert_origin(value.origin(), expected);
        }
        other => panic!("unexpected scalar {other:?}"),
    }
}

fn constructed(family: Family, channels: [&str; 3], alpha: Option<&str>) -> CssColor {
    let [a, b, c] = channels;
    let alpha = alpha.map(component);
    match family {
        Family::Rgb => CssColor::from_rgb(
            CssRgbColor::try_new(CssColorSyntax::Modern, channels.map(component), alpha).unwrap(),
        ),
        Family::Hsl => CssColor::from_hsl(
            CssHslColor::try_new(
                CssColorSyntax::Modern,
                hue(a),
                component(b),
                component(c),
                alpha,
            )
            .unwrap(),
        ),
        Family::Hwb => CssColor::from_hwb(
            CssHwbColor::try_new(hue(a), component(b), component(c), alpha).unwrap(),
        ),
        Family::Lab | Family::Oklab => {
            let value =
                CssLabColor::try_new(component(a), component(b), component(c), alpha).unwrap();
            if matches!(family, Family::Lab) {
                CssColor::from_lab(value)
            } else {
                CssColor::from_oklab(value)
            }
        }
        Family::Lch | Family::Oklch => {
            let value = CssLchColor::try_new(component(a), component(b), hue(c), alpha).unwrap();
            if matches!(family, Family::Lch) {
                CssColor::from_lch(value)
            } else {
                CssColor::from_oklch(value)
            }
        }
        Family::Predefined(space) => CssColor::from_predefined(
            CssPredefinedColor::try_new(space, channels.map(component), alpha).unwrap(),
        ),
    }
}

#[derive(Debug, PartialEq)]
enum Slot<'a> {
    Scalar(&'a CssColorComponent),
    Hue(&'a CssColorHue),
}
fn slots(color: &CssColor, family: Family) -> [Slot<'_>; 3] {
    match family {
        Family::Rgb => color
            .rgb_value()
            .unwrap()
            .channels()
            .each_ref()
            .map(Slot::Scalar),
        Family::Hsl => {
            let v = color.hsl_value().unwrap();
            [
                Slot::Hue(v.hue()),
                Slot::Scalar(v.saturation()),
                Slot::Scalar(v.lightness()),
            ]
        }
        Family::Hwb => {
            let v = color.hwb_value().unwrap();
            [
                Slot::Hue(v.hue()),
                Slot::Scalar(v.whiteness()),
                Slot::Scalar(v.blackness()),
            ]
        }
        Family::Lab | Family::Oklab => {
            let v = if matches!(family, Family::Lab) {
                color.lab_value()
            } else {
                color.oklab_value()
            }
            .unwrap();
            [
                Slot::Scalar(v.lightness()),
                Slot::Scalar(v.a()),
                Slot::Scalar(v.b()),
            ]
        }
        Family::Lch | Family::Oklch => {
            let v = if matches!(family, Family::Lch) {
                color.lch_value()
            } else {
                color.oklch_value()
            }
            .unwrap();
            [
                Slot::Scalar(v.lightness()),
                Slot::Scalar(v.chroma()),
                Slot::Hue(v.hue()),
            ]
        }
        Family::Predefined(space) => {
            let v = color.predefined_value().unwrap();
            assert_eq!(v.color_space(), space);
            v.channels().each_ref().map(Slot::Scalar)
        }
    }
}
fn alpha(color: &CssColor, family: Family) -> Option<&CssColorComponent> {
    match family {
        Family::Rgb => color.rgb_value().unwrap().alpha(),
        Family::Hsl => color.hsl_value().unwrap().alpha(),
        Family::Hwb => color.hwb_value().unwrap().alpha(),
        Family::Lab => color.lab_value().unwrap().alpha(),
        Family::Oklab => color.oklab_value().unwrap().alpha(),
        Family::Lch => color.lch_value().unwrap().alpha(),
        Family::Oklch => color.oklch_value().unwrap().alpha(),
        Family::Predefined(_) => color.predefined_value().unwrap().alpha(),
    }
}
fn assert_slot(value: &Slot<'_>, text: &str) {
    match value {
        Slot::Scalar(value) => assert_component(value, text),
        Slot::Hue(CssColorHue::None) => assert_eq!(text, "none"),
        Slot::Hue(CssColorHue::Number(value)) => {
            assert_eq!(value.numeric().representation(), text);
            assert_origin(value.origin(), text);
        }
        Slot::Hue(CssColorHue::Angle(value)) => {
            assert_eq!(value.unit(), CssAngleUnit::Turns);
            assert_eq!(
                value.numeric().representation(),
                text.strip_suffix("turn").unwrap()
            );
            assert_origin(value.origin(), text);
        }
        other => panic!("unexpected typed hue: {other:?}"),
    }
}
fn assert_fields(
    color: &CssColor,
    family: Family,
    channels: [&str; 3],
    expected_alpha: Option<&str>,
) {
    for (slot, text) in slots(color, family).iter().zip(channels) {
        assert_slot(slot, text);
    }
    match (alpha(color, family), expected_alpha) {
        (None, None) => {}
        (Some(value), Some(text)) => assert_component(value, text),
        other => panic!("alpha mismatch: {other:?}"),
    }
}
fn text(name: &str, channels: [&str; 3], alpha: Option<&str>, predefined: bool) -> String {
    format!(
        "{}({}{} {} {}{})",
        if predefined { "color" } else { name },
        if predefined {
            format!("{name} ")
        } else {
            String::new()
        },
        channels[0],
        channels[1],
        channels[2],
        alpha.map_or(String::new(), |v| format!(" / {v}"))
    )
}
fn expected(name: &str, channels: [&str; 3], alpha_missing: bool) -> String {
    let channels = match name {
        "hsl" | "hwb" => [
            channels[0].to_owned(),
            if channels[1] == "none" {
                "none".into()
            } else {
                "0%".into()
            },
            if channels[2] == "none" {
                "none".into()
            } else {
                "0%".into()
            },
        ],
        _ => channels.map(str::to_owned),
    };
    let head = if name == "rgb" {
        "color(srgb".into()
    } else if ORDINARY.iter().any(|(n, _)| *n == name) {
        format!("{name}(")
    } else {
        format!("color({name}")
    };
    let separator = if name == "rgb" || !ORDINARY.iter().any(|(n, _)| *n == name) {
        " "
    } else {
        ""
    };
    format!(
        "{head}{separator}{} {} {}{})",
        channels[0],
        channels[1],
        channels[2],
        if alpha_missing { " / none" } else { "" }
    )
}
fn families() -> Vec<(&'static str, Family)> {
    ORDINARY
        .into_iter()
        .chain(
            SPACES
                .into_iter()
                .filter(|(name, _, _)| *name != "xyz")
                .map(|(_, space, canonical)| (canonical, Family::Predefined(space))),
        )
        .collect()
}

#[test]
fn each_modern_missing_channel_keeps_none_distinct_from_zero_in_both_front_doors() {
    for (name, family) in families() {
        let zero = constructed(family, ["0"; 3], None);
        for missing in 0..3 {
            let mut channels = ["0"; 3];
            channels[missing] = "none";
            let source = text(
                name,
                channels,
                None,
                matches!(family, Family::Predefined(_)),
            );
            for color in [parsed(&source), constructed(family, channels, None)] {
                assert_fields(&color, family, channels, None);
                assert_ne!(
                    slots(&color, family)[missing],
                    slots(&zero, family)[missing],
                    "{source}"
                );
                let before = color.clone();
                assert_eq!(
                    color.to_specified_css().unwrap(),
                    expected(name, channels, false),
                    "{source}"
                );
                assert_eq!(color, before);
                assert_fields(&color, family, channels, None);
            }
        }
    }
}

#[test]
fn each_modern_missing_alpha_keeps_none_distinct_from_zero_and_omission() {
    for (name, family) in families() {
        let omitted = constructed(family, ["0"; 3], None);
        let zero = constructed(family, ["0"; 3], Some("0"));
        let source = text(
            name,
            ["0"; 3],
            Some("none"),
            matches!(family, Family::Predefined(_)),
        );
        for color in [parsed(&source), constructed(family, ["0"; 3], Some("none"))] {
            assert_fields(&color, family, ["0"; 3], Some("none"));
            assert_ne!(alpha(&color, family), alpha(&omitted, family));
            assert_ne!(alpha(&color, family), alpha(&zero, family));
            let before = color.clone();
            assert_eq!(
                color.to_specified_css().unwrap(),
                expected(name, ["0"; 3], true),
                "{source}"
            );
            assert_eq!(color, before);
        }
    }
}

#[test]
fn hsl_and_hsla_aliases_preserve_modern_or_legacy_syntax_with_number_or_angle_hue() {
    for name in ["hsl", "hsla"] {
        for syntax in [CssColorSyntax::Modern, CssColorSyntax::Legacy] {
            for h in ["0", "1turn"] {
                for (a, suffix) in [(None, ""), (Some(".25"), ", 0.25"), (Some("25%"), ", 0.25")] {
                    let legacy = syntax == CssColorSyntax::Legacy;
                    let channels = if legacy {
                        [h, "0%", "0%"]
                    } else {
                        [h, "0", "0%"]
                    };
                    let source = if legacy {
                        format!(
                            "{name}({h}, 0%, 0%{})",
                            a.map_or(String::new(), |v| format!(", {v}"))
                        )
                    } else {
                        text(name, channels, a, false)
                    };
                    let checked = CssColor::from_hsl(
                        CssHslColor::try_new(
                            syntax,
                            hue(h),
                            component(channels[1]),
                            component(channels[2]),
                            a.map(component),
                        )
                        .unwrap(),
                    );
                    for color in [parsed(&source), checked] {
                        assert_eq!(color.hsl_value().unwrap().syntax(), syntax);
                        assert_fields(&color, Family::Hsl, channels, a);
                        let before = color.clone();
                        let output = if a.is_some() {
                            format!("rgba(0, 0, 0{suffix})")
                        } else {
                            "rgb(0, 0, 0)".into()
                        };
                        assert_eq!(color.to_specified_css().unwrap(), output, "{source}");
                        assert_eq!(color, before);
                    }
                }
            }
        }
    }
}

#[test]
fn legacy_aliases_reject_none_in_each_channel_or_alpha_with_sibling_recovery() {
    for name in ["rgb", "rgba", "hsl", "hsla"] {
        let rgb = name.starts_with("rgb");
        for missing in 0..4 {
            let mut args = if rgb {
                ["0", "0", "0", ".25"]
            } else {
                ["0", "0%", "0%", ".25"]
            };
            args[missing] = "none";
            assert_recovered(&format!(
                "{name}({}, {}, {}, {})",
                args[0], args[1], args[2], args[3]
            ));
            let result = if rgb {
                CssRgbColor::try_new(
                    CssColorSyntax::Legacy,
                    [component(args[0]), component(args[1]), component(args[2])],
                    Some(component(args[3])),
                )
                .map(CssColor::from_rgb)
            } else {
                CssHslColor::try_new(
                    CssColorSyntax::Legacy,
                    hue(args[0]),
                    component(args[1]),
                    component(args[2]),
                    Some(component(args[3])),
                )
                .map(CssColor::from_hsl)
            };
            assert_eq!(
                result.unwrap_err(),
                CssColorConstructionError::InvalidSyntax
            );
        }
    }
}

#[test]
fn hsl_aliases_reject_mixed_separators_and_nonpercentage_legacy_channels() {
    for name in ["hsl", "hsla"] {
        for arguments in ["0, 0%, 0% / .25", "0 0% 0%, .25", "0, 0, 0%", "0, 0%, 0"] {
            assert_recovered(&format!("{name}({arguments})"));
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
