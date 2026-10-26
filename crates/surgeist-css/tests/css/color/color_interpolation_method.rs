//! Characterization of existing interpolation grammar and checked wrapper behavior.
//! Color4 CRD 2026-09-08 §13.2 supplies the spaces and polar-only hue grammar;
//! §13.5 supplies the shorter baseline. Color5 WD 2026-09-08 §9.1 adds custom
//! dashed identifiers, whose profile binding remains contextual. Color5 §11.1
//! supplies canonical mix serialization. This suite establishes GREEN behavior,
//! not a preimplementation RED. Replacement checked-method API tests establish
//! new API behavior without executable preimplementation RED.

use crate::css::support::color_and_selector_fixtures::assert_color_grammar_failure;
use crate::css::support::declaration_color as color;
use surgeist_color::{ColorSpace, HueInterpolation, PredefinedColorSpace};
use surgeist_css::*;

const RECTANGULAR: [(ColorSpace, &str); 11] = [
    (ColorSpace::Predefined(PredefinedColorSpace::Srgb), "srgb"),
    (
        ColorSpace::Predefined(PredefinedColorSpace::SrgbLinear),
        "srgb-linear",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3),
        "display-p3",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::DisplayP3Linear),
        "display-p3-linear",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::A98Rgb),
        "a98-rgb",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::ProphotoRgb),
        "prophoto-rgb",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::Rec2020),
        "rec2020",
    ),
    (ColorSpace::Lab, "lab"),
    (ColorSpace::Oklab, "oklab"),
    (
        ColorSpace::Predefined(PredefinedColorSpace::XyzD50),
        "xyz-d50",
    ),
    (
        ColorSpace::Predefined(PredefinedColorSpace::XyzD65),
        "xyz-d65",
    ),
];
const POLAR: [(ColorSpace, &str); 4] = [
    (ColorSpace::Hsl, "hsl"),
    (ColorSpace::Hwb, "hwb"),
    (ColorSpace::Lch, "lch"),
    (ColorSpace::Oklch, "oklch"),
];
const HUES: [(HueInterpolation, &str); 4] = [
    (HueInterpolation::Shorter, "shorter"),
    (HueInterpolation::Longer, "longer"),
    (HueInterpolation::Increasing, "increasing"),
    (HueInterpolation::Decreasing, "decreasing"),
];

// Checked construction is const. Its error carries retained origins, so a
// constant can hold the checked result but cannot discard the error to
// unwrap it; the polar methods are unwrapped at run time.
const OMITTED_POLAR: Result<CssColorInterpolationMethod, CssColorConstructionError> =
    CssColorInterpolationMethod::try_new(ColorSpace::Hsl, None);
const EXPLICIT_POLAR: Result<CssColorInterpolationMethod, CssColorConstructionError> =
    CssColorInterpolationMethod::try_new(ColorSpace::Hsl, Some(HueInterpolation::Shorter));
const REJECTED_RECTANGULAR: Result<CssColorInterpolationMethod, CssColorConstructionError> =
    CssColorInterpolationMethod::try_new(ColorSpace::Lab, Some(HueInterpolation::Longer));

#[test]
fn const_checked_methods_preserve_authored_omission_and_supply_the_baseline_default() {
    let omitted_polar = OMITTED_POLAR.expect("polar omitted hue is valid");
    let explicit_polar = EXPLICIT_POLAR.expect("polar shorter hue is valid");
    assert_eq!(omitted_polar.space(), ColorSpace::Hsl);
    assert_eq!(omitted_polar.hue(), None);
    assert_eq!(
        omitted_polar.effective_hue(),
        Some(HueInterpolation::Shorter)
    );
    assert_eq!(
        CssColorInterpolation::from_predefined(omitted_polar).predefined(),
        Some(omitted_polar)
    );
    assert_ne!(omitted_polar, explicit_polar);
    assert_eq!(explicit_polar.hue(), Some(HueInterpolation::Shorter));
    assert_eq!(
        explicit_polar.effective_hue(),
        omitted_polar.effective_hue()
    );
    assert!(
        REJECTED_RECTANGULAR
            .as_ref()
            .unwrap_err()
            .is_hue_requires_polar_space()
    );
    for (space, _) in POLAR {
        let omitted = CssColorInterpolationMethod::try_new(space, None).unwrap();
        let explicit =
            CssColorInterpolationMethod::try_new(space, Some(HueInterpolation::Shorter)).unwrap();
        assert_ne!(omitted, explicit);
        assert_eq!(omitted.effective_hue(), explicit.effective_hue());
    }
}

#[test]
fn rejected_hue_construction_has_a_typed_error_and_display_contract() {
    let error =
        CssColorInterpolationMethod::try_new(ColorSpace::Lab, Some(HueInterpolation::Longer))
            .unwrap_err();
    assert!(error.is_hue_requires_polar_space());
    assert_eq!(
        error.to_string(),
        "hue interpolation requires a polar color space"
    );
    let public_error: &dyn std::error::Error = &error;
    assert!(public_error.source().is_none());
    // Downstream matches include a wildcard because construction errors can grow.
    assert!(
        error.is_hue_requires_polar_space(),
        "unexpected interpolation construction error"
    );
    assert_eq!(public_error.to_string(), error.to_string());
}

fn admitted(source: &str) -> [CssDeclaration; 2] {
    let style = format!("color:{source}!important");
    let report = CssParser::default().parse_block_contents(&style);
    assert!(report.is_clean(), "{style}: {:?}", report.diagnostics());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(parsed.importance(), CssImportance::Important);
    let components = CssComponentValues::parse(&CssParser::default(), source).unwrap();
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    assert_eq!(
        parsed
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        source
    );
    [parsed.clone(), checked]
}

fn rejected_with_recovery(value: &str, rectangular_hue: Option<(&str, &str)>) {
    // Include a multibyte identifier before the invalid declaration, so spans
    // exercise original byte coordinates rather than detached value offsets.
    let source = format!("--😀:kept;\ncolor:{value};opacity:.5");
    let report = CssParser::default().parse_block_contents(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejection for {value}: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidColorSyntax);
    if let Some((keyword, strategy)) = rectangular_hue {
        // Parsing saves the boundary after the space identifier. Public source
        // resolution advances across whitespace to the responsible strategy
        // token, as the established structured color diagnostic contract states.
        let responsible =
            source.find(&format!("in {keyword}")).unwrap() + "in ".len() + keyword.len() + 1;
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible
        );
        assert_eq!(diagnostic.error().position().line().value(), 1);
        let line_start = source.find('\n').unwrap() + 1;
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible - line_start
        );
        let detail = diagnostic.error();
        assert_eq!(
            detail.code(),
            CssErrorCode::InvalidColorSyntax,
            "structured hue-method rejection"
        );
        assert_eq!(
            detail.color_component().map(|component| component.as_str()),
            Some("hue interpolation")
        );
        let encountered = detail.encountered().expect("responsible hue strategy");
        assert_eq!(encountered.kind(), CssTokenKind::Ident);
        assert_eq!(encountered.authored(), strategy);
    }
    let begin = source.find("color:").unwrap();
    let end = begin + "color:".len() + value.len() + 1;
    assert_eq!(diagnostic.span().start().byte_offset().value(), begin);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(
        CssParser::default()
            .parse_block_contents(&source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
    assert_color_grammar_failure(
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
            CssImportance::Normal,
        ),
        value,
    );
}

#[test]
fn rectangular_methods_without_hue_are_accepted_by_the_checked_wrapper() {
    for (space, _) in RECTANGULAR {
        let method = CssColorInterpolationMethod::try_new(space, None).unwrap();
        let interpolation = CssColorInterpolation::from_predefined(method);
        assert_eq!(method.space(), space);
        assert_eq!(method.hue(), None);
        assert_eq!(method.effective_hue(), None);
        assert_eq!(interpolation.predefined(), Some(method));
        assert!(interpolation.custom_profile().is_none());
    }
}

#[test]
fn every_explicit_hue_strategy_is_rejected_for_every_rectangular_space() {
    for (space, keyword) in RECTANGULAR {
        for (hue, strategy) in HUES {
            assert!(
                CssColorInterpolationMethod::try_new(space, Some(hue))
                    .as_ref()
                    .unwrap_err()
                    .is_hue_requires_polar_space()
            );
            rejected_with_recovery(
                &format!("color-mix(in {keyword} {strategy} hue, red, blue)"),
                Some((keyword, strategy)),
            );
        }
    }
}

#[test]
fn polar_methods_preserve_omitted_and_all_explicit_hue_strategies() {
    for (space, keyword) in POLAR {
        for hue in [
            None,
            Some(HueInterpolation::Shorter),
            Some(HueInterpolation::Longer),
            Some(HueInterpolation::Increasing),
            Some(HueInterpolation::Decreasing),
        ] {
            let method = CssColorInterpolationMethod::try_new(space, hue).unwrap();
            let interpolation = CssColorInterpolation::from_predefined(method);
            assert_eq!(method.space(), space);
            assert_eq!(method.hue(), hue);
            assert_eq!(
                method.effective_hue(),
                Some(hue.unwrap_or(HueInterpolation::Shorter))
            );
            assert_eq!(interpolation.predefined(), Some(method));
            let suffix = match hue {
                None => String::new(),
                Some(value) => format!(
                    " {} hue",
                    HUES.iter().find(|(hue, _)| *hue == value).unwrap().1
                ),
            };
            let source = format!("color-mix(in {keyword}{suffix}, red, blue)");
            let canonical_suffix = if hue == Some(HueInterpolation::Shorter) {
                ""
            } else {
                &suffix
            };
            let expected = format!("color-mix(in {keyword}{canonical_suffix}, red, blue)");
            for declaration in admitted(&source) {
                let value = color(&declaration);
                assert_eq!(
                    value
                        .color_mix_value()
                        .unwrap()
                        .interpolation()
                        .unwrap()
                        .predefined(),
                    Some(method)
                );
                assert_eq!(
                    value.serialize_specified(Default::default()).unwrap(),
                    expected
                );
                // Omitting the default shorter spelling does not erase authored identity.
                assert_eq!(
                    value
                        .color_mix_value()
                        .unwrap()
                        .interpolation()
                        .unwrap()
                        .predefined()
                        .unwrap()
                        .hue(),
                    hue
                );
            }
        }
    }
}

#[test]
fn every_predefined_keyword_case_and_escape_preserves_space_and_canonical_output() {
    let alias = (ColorSpace::Predefined(PredefinedColorSpace::XyzD65), "xyz");
    for (space, keyword) in RECTANGULAR.into_iter().chain(POLAR).chain([alias]) {
        for spelling in [
            keyword.to_owned(),
            keyword.to_ascii_uppercase(),
            format!("\\{:x} {}", keyword.as_bytes()[0], &keyword[1..]),
        ] {
            let source = format!("color-mix(in {spelling}, red, blue)");
            let canonical = if keyword == "xyz" { "xyz-d65" } else { keyword };
            let expected = if space == ColorSpace::Oklab {
                "color-mix(red, blue)".to_owned()
            } else {
                format!("color-mix(in {canonical}, red, blue)")
            };
            for declaration in admitted(&source) {
                let value = color(&declaration);
                let method = value
                    .color_mix_value()
                    .unwrap()
                    .interpolation()
                    .unwrap()
                    .predefined()
                    .unwrap();
                assert_eq!(method.space(), space);
                assert_eq!(method.hue(), None);
                assert_eq!(
                    value.serialize_specified(Default::default()).unwrap(),
                    expected
                );
                for round_trip in admitted(&expected) {
                    assert_eq!(
                        color(&round_trip)
                            .serialize_specified(Default::default())
                            .unwrap(),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn custom_interpolation_profiles_preserve_decoded_case_sensitive_identity() {
    for (spelling, name, canonical) in [
        ("--Profile", "--Profile", "--Profile"),
        ("--profile", "--profile", "--profile"),
        ("--Pr\\6f file", "--Profile", "--Profile"),
        ("--a\\ b", "--a b", "--a\\ b"),
        ("--", "--", "--"),
    ] {
        let constructed =
            CssColorInterpolation::custom(CssColorProfileName::try_new(name).unwrap());
        assert!(constructed.predefined().is_none());
        assert_eq!(constructed.custom_profile().unwrap().as_str(), name);
        for declaration in admitted(&format!("color-mix(in {spelling}, red, blue)")) {
            let value = color(&declaration);
            let interpolation = value.color_mix_value().unwrap().interpolation().unwrap();
            assert_eq!(interpolation, &constructed);
            assert_eq!(
                value.serialize_specified(Default::default()).unwrap(),
                format!("color-mix(in {canonical}, red, blue)")
            );
        }
        for (_, strategy) in HUES {
            rejected_with_recovery(
                &format!("color-mix(in {spelling} {strategy} hue, red, blue)"),
                None,
            );
        }
    }
    assert_ne!(
        CssColorProfileName::try_new("--Profile"),
        CssColorProfileName::try_new("--profile")
    );
}

#[test]
fn malformed_hue_suffixes_and_method_order_recover_at_the_declaration_boundary() {
    for value in [
        "color-mix(in hsl shorter, red, blue)",
        "color-mix(in hsl hue, red, blue)",
        "color-mix(in hsl shorter hue longer hue, red, blue)",
        "color-mix(in shorter hue hsl, red, blue)",
        "color-mix(in hsl sideways hue, red, blue)",
        "color-mix(in, red, blue)",
        "color-mix(hsl, red, blue)",
    ] {
        rejected_with_recovery(value, None);
    }
}
