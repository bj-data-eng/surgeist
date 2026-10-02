#![forbid(unsafe_code)]
//! Ordinary predefined color() grammar from Color 4 CRD 2026-09-08 §§10.1–10.9,
//! with canonical projection from §16.5 and the selected authored numeric phases.
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#color-function
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#serializing-color-function-values
//! Source SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Independently explicit expected values use 100% = 1 for every RGB/XYZ channel,
//! no channel gamut clipping, and direct alpha clipping to [0, 1]. Calculations
//! retain their authored graph and the selected specified projection wrappers.

use surgeist_css::*;

// The source lists seven RGB keywords and three XYZ spellings; xyz denotes D65.
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
    assert_eq!(values.items().len(), 1, "{text}");
    values.items()[0].clone()
}

fn component(text: &str) -> CssColorComponent {
    if text.eq_ignore_ascii_case("none") {
        CssColorComponent::None
    } else if text.starts_with("calc(") {
        let values = parse_component_values(text).unwrap();
        if text.contains('%') {
            CssColorComponent::PercentageCalculation(
                CssPercentageCalculation::try_from_components(values).unwrap(),
            )
        } else {
            CssColorComponent::NumberCalculation(
                CssNumberCalculation::try_from_components(values).unwrap(),
            )
        }
    } else if text.ends_with('%') {
        CssColorComponent::Percentage(
            CssColorPercentageLiteral::try_from_component(token(text)).unwrap(),
        )
    } else {
        CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token(text)).unwrap())
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

fn assert_fields(
    color: &CssColor,
    space: CssPredefinedColorSpace,
    channels: [&str; 3],
    alpha: Option<&str>,
) {
    let value = color.predefined_value().expect("predefined owning branch");
    assert_eq!(value.color_space(), space);
    for (actual, expected) in value.channels().iter().zip(channels) {
        assert_component(actual, expected);
    }
    match (value.alpha(), alpha) {
        (Some(actual), Some(expected)) => assert_component(actual, expected),
        (None, None) => {}
        other => panic!("alpha domain mismatch: {other:?}"),
    }
}

fn assert_case(
    name: &str,
    space: CssPredefinedColorSpace,
    canonical: &str,
    channels: [&str; 3],
    alpha: Option<&str>,
    projected: &str,
) {
    let text = format!(
        "color({name} {} {} {}{})",
        channels[0],
        channels[1],
        channels[2],
        alpha.map_or(String::new(), |value| format!(" / {value}"))
    );
    let checked = CssColor::from_predefined(
        CssPredefinedColor::try_new(space, channels.map(component), alpha.map(component)).unwrap(),
    );
    for color in [parsed(&text), checked] {
        assert_fields(&color, space, channels, alpha);
        let before = color.clone();
        assert_eq!(
            color.to_specified_css().unwrap(),
            format!("color({canonical} {projected})"),
            "{text}"
        );
        assert_eq!(color, before, "projection must preserve authored graph");
        assert_fields(&color, space, channels, alpha);
    }
}

#[test]
fn every_space_accepts_number_percentage_and_mixed_missing_domains() {
    for (name, space, canonical) in SPACES {
        for (channels, output) in [
            (["1", ".5", "0"], "1 0.5 0"),
            (["100%", "50%", "0%"], "1 0.5 0"),
            (["-20%", "2", "none"], "-0.2 2 none"),
        ] {
            assert_case(name, space, canonical, channels, None, output);
        }
    }
}

#[test]
fn every_space_retains_raw_alpha_domain_before_direct_projection() {
    for (name, space, canonical) in SPACES {
        for (alpha, suffix) in [
            (None, ""),
            (Some(".25"), " / 0.25"),
            (Some("25%"), " / 0.25"),
            (Some("none"), " / none"),
            (Some("150%"), ""),
            (Some("-2"), " / 0"),
        ] {
            assert_case(
                name,
                space,
                canonical,
                ["1", "0", "0"],
                alpha,
                &format!("1 0 0{suffix}"),
            );
        }
    }
}

#[test]
fn extended_coordinates_are_not_gamut_clipped_in_any_space() {
    for (name, space, canonical) in SPACES {
        assert_case(
            name,
            space,
            canonical,
            ["-250%", "3000", "125%"],
            Some("1"),
            "-2.5 3000 1.25",
        );
    }
}

#[test]
fn xyz_is_the_d65_alias_and_d50_remains_distinct() {
    let alias = parsed("color(xyz .1 .2 .3)");
    let d65 = parsed("color(xyz-d65 .1 .2 .3)");
    let d50 = parsed("color(xyz-d50 .1 .2 .3)");
    assert_eq!(
        alias.predefined_value().unwrap().color_space(),
        CssPredefinedColorSpace::XyzD65
    );
    assert_eq!(
        d65.predefined_value().unwrap().color_space(),
        CssPredefinedColorSpace::XyzD65
    );
    assert_eq!(
        d50.predefined_value().unwrap().color_space(),
        CssPredefinedColorSpace::XyzD50
    );
    // Full authored equality includes provenance; semantic space identity does not.
    assert_eq!(
        alias.to_specified_css().unwrap(),
        "color(xyz-d65 0.1 0.2 0.3)"
    );
    assert_eq!(
        d65.to_specified_css().unwrap(),
        "color(xyz-d65 0.1 0.2 0.3)"
    );
    assert_eq!(
        d50.to_specified_css().unwrap(),
        "color(xyz-d50 0.1 0.2 0.3)"
    );
}

#[test]
fn function_space_and_missing_keywords_accept_ascii_case_and_escapes() {
    for (name, space, canonical) in SPACES {
        let escaped = format!("\\{:x} {}", name.as_bytes()[0], &name[1..]);
        for (function, spelling) in [("COLOR", name.to_ascii_uppercase()), ("c\\6f lor", escaped)] {
            let color = parsed(&format!("{function}({spelling} NoNe 50% -2 / NONE)"));
            assert_fields(&color, space, ["none", "50%", "-2"], Some("none"));
            assert_eq!(
                color.to_specified_css().unwrap(),
                format!("color({canonical} none 0.5 -2 / none)")
            );
        }
    }
}

#[test]
fn calculations_keep_wrappers_and_unbounded_channels_or_alpha() {
    for (name, space, canonical) in SPACES {
        for (channels, alpha, output) in [
            (
                ["calc(-250%)", "calc(3)", "none"],
                "calc(150%)",
                "calc(-2.5) calc(3) none / calc(1.5)",
            ),
            (
                ["calc(125%)", "calc(-2)", "calc(0)"],
                "calc(-2)",
                "calc(1.25) calc(-2) calc(0) / calc(-2)",
            ),
            (["calc(1)", "0", "0"], "calc(1)", "calc(1) 0 0 / calc(1)"),
        ] {
            assert_case(name, space, canonical, channels, Some(alpha), output);
        }
    }
}

#[test]
fn malformed_predefined_forms_recover_only_the_color_declaration() {
    for (name, _, _) in SPACES {
        for arguments in [
            "",
            "1",
            "1 2",
            "1 2 3 4",
            "1, 2, 3",
            "1 2 3, .5",
            "1 2 3 /",
            "1 2 3 / .5 .6",
            "1 2 3 / .5 / .6",
            "1px 2 3",
            "1 2deg 3",
            "1 2 3s",
            "1 2 3 / 1deg",
            "calc(1px) 2 3",
            "1 2 3 / calc(1s)",
        ] {
            assert_recovered(&format!("color({name} {arguments})"));
        }
    }
    for name in [
        "profoto-rgb",
        "display-p4",
        "rgb",
        "lab",
        "xyz-d60",
        "unknown",
    ] {
        assert_recovered(&format!("color({name} 1 2 3)"));
    }
    // Accepted Color 5 custom profiles are a separate branch, not unknown predefined spaces.
    let custom = parsed("color(--Profile 1 2 3)");
    assert!(custom.custom_value().is_some());
    assert!(custom.predefined_value().is_none());
    assert_eq!(custom.to_specified_css().unwrap(), "color(--Profile 1 2 3)");
}

#[test]
fn checked_component_front_doors_exclude_dimensions_not_in_the_grammar() {
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
    for text in ["calc(1px)", "calc(1deg)", "calc(1s)"] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err()
        );
        assert!(
            CssPercentageCalculation::try_from_components(parse_component_values(text).unwrap())
                .is_err()
        );
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
