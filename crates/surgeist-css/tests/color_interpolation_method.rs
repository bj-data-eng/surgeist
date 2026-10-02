#![forbid(unsafe_code)]
//! Characterization of existing interpolation grammar and checked wrapper behavior.
//! Color4 CRD 2026-09-08 §13.2 supplies the spaces and polar-only hue grammar;
//! §13.5 supplies the shorter baseline. Color5 WD 2026-09-08 §9.1 adds custom
//! dashed identifiers, whose profile binding remains contextual. Color5 §11.1
//! supplies canonical mix serialization. This suite establishes GREEN behavior,
//! not a preimplementation RED. Replacement checked-method API tests establish
//! new API behavior without executable preimplementation RED.

use surgeist_css::*;

const RECTANGULAR: [(CssColorInterpolationSpace, &str); 11] = [
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
        "srgb",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::SrgbLinear),
        "srgb-linear",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3),
        "display-p3",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3Linear),
        "display-p3-linear",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::A98Rgb),
        "a98-rgb",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::ProphotoRgb),
        "prophoto-rgb",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Rec2020),
        "rec2020",
    ),
    (CssColorInterpolationSpace::Lab, "lab"),
    (CssColorInterpolationSpace::Oklab, "oklab"),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD50),
        "xyz-d50",
    ),
    (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "xyz-d65",
    ),
];
const POLAR: [(CssColorInterpolationSpace, &str); 4] = [
    (CssColorInterpolationSpace::Hsl, "hsl"),
    (CssColorInterpolationSpace::Hwb, "hwb"),
    (CssColorInterpolationSpace::Lch, "lch"),
    (CssColorInterpolationSpace::Oklch, "oklch"),
];
const HUES: [(CssHueInterpolationMethod, &str); 4] = [
    (CssHueInterpolationMethod::Shorter, "shorter"),
    (CssHueInterpolationMethod::Longer, "longer"),
    (CssHueInterpolationMethod::Increasing, "increasing"),
    (CssHueInterpolationMethod::Decreasing, "decreasing"),
];

const OMITTED_POLAR: CssColorInterpolationMethod =
    match CssColorInterpolationMethod::try_new(CssColorInterpolationSpace::Hsl, None) {
        Ok(value) => value,
        Err(_) => panic!("polar omitted hue is valid"),
    };
const EXPLICIT_POLAR: CssColorInterpolationMethod = match CssColorInterpolationMethod::try_new(
    CssColorInterpolationSpace::Hsl,
    Some(CssHueInterpolationMethod::Shorter),
) {
    Ok(value) => value,
    Err(_) => panic!("polar shorter hue is valid"),
};
const REJECTED_RECTANGULAR: Result<
    CssColorInterpolationMethod,
    CssColorInterpolationMethodConstructionError,
> = CssColorInterpolationMethod::try_new(
    CssColorInterpolationSpace::Lab,
    Some(CssHueInterpolationMethod::Longer),
);
const CONST_WRAPPER: CssColorInterpolation = CssColorInterpolation::from_predefined(OMITTED_POLAR);
const CONST_DEFAULT_HUE: Option<CssHueInterpolationMethod> = OMITTED_POLAR.effective_hue();
const CONST_SPACE: CssColorInterpolationSpace = OMITTED_POLAR.space();
const CONST_AUTHORED_HUE: Option<CssHueInterpolationMethod> = OMITTED_POLAR.hue();

#[test]
fn const_checked_methods_preserve_authored_omission_and_supply_the_baseline_default() {
    assert_eq!(CONST_SPACE, CssColorInterpolationSpace::Hsl);
    assert_eq!(CONST_AUTHORED_HUE, None);
    assert_eq!(CONST_DEFAULT_HUE, Some(CssHueInterpolationMethod::Shorter));
    assert_eq!(CONST_WRAPPER.predefined(), Some(OMITTED_POLAR));
    assert_ne!(OMITTED_POLAR, EXPLICIT_POLAR);
    assert_eq!(
        EXPLICIT_POLAR.hue(),
        Some(CssHueInterpolationMethod::Shorter)
    );
    assert_eq!(
        EXPLICIT_POLAR.effective_hue(),
        OMITTED_POLAR.effective_hue()
    );
    assert_eq!(
        REJECTED_RECTANGULAR,
        Err(CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace)
    );
    for (space, _) in POLAR {
        let omitted = CssColorInterpolationMethod::try_new(space, None).unwrap();
        let explicit =
            CssColorInterpolationMethod::try_new(space, Some(CssHueInterpolationMethod::Shorter))
                .unwrap();
        assert_ne!(omitted, explicit);
        assert_eq!(omitted.effective_hue(), explicit.effective_hue());
    }
}

#[test]
fn rejected_hue_construction_has_a_typed_error_and_display_contract() {
    let error = CssColorInterpolationMethod::try_new(
        CssColorInterpolationSpace::Lab,
        Some(CssHueInterpolationMethod::Longer),
    )
    .unwrap_err();
    assert_eq!(
        error,
        CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace
    );
    assert_eq!(
        error.to_string(),
        "hue interpolation requires a polar color space"
    );
    let public_error: &dyn std::error::Error = &error;
    assert!(public_error.source().is_none());
    // Downstream matches include a wildcard because construction errors can grow.
    match error {
        CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace => {
            assert_eq!(public_error.to_string(), error.to_string());
        }
        _ => panic!("unexpected interpolation construction error"),
    }
}

fn color(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected typed color")
    };
    value.value()
}

fn admitted(source: &str) -> [CssDeclaration; 2] {
    let style = format!("color:{source}!important");
    let report = parse_style_attribute(&style);
    assert!(report.is_clean(), "{style}: {:?}", report.diagnostics());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(parsed.importance(), CssImportance::Important);
    let components = parse_component_values(source).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &components);
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        source
    );
    [parsed.clone(), checked]
}

fn rejected_with_recovery(value: &str, rectangular_hue: Option<(&str, &str)>) {
    // Include a multibyte identifier before the invalid declaration, so spans
    // exercise original byte coordinates rather than detached value offsets.
    let source = format!("--😀:kept;\ncolor:{value};opacity:.5");
    let report = parse_style_attribute(&source);
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
        let ErrorKind::InvalidColorSyntax(detail) = diagnostic.error().kind() else {
            panic!("structured hue-method rejection")
        };
        assert_eq!(
            detail.component().map(|component| component.as_str()),
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
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err()
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
            assert_eq!(
                CssColorInterpolationMethod::try_new(space, Some(hue)),
                Err(CssColorInterpolationMethodConstructionError::HueRequiresPolarSpace)
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
            Some(CssHueInterpolationMethod::Shorter),
            Some(CssHueInterpolationMethod::Longer),
            Some(CssHueInterpolationMethod::Increasing),
            Some(CssHueInterpolationMethod::Decreasing),
        ] {
            let method = CssColorInterpolationMethod::try_new(space, hue).unwrap();
            let interpolation = CssColorInterpolation::from_predefined(method);
            assert_eq!(method.space(), space);
            assert_eq!(method.hue(), hue);
            assert_eq!(
                method.effective_hue(),
                Some(hue.unwrap_or(CssHueInterpolationMethod::Shorter))
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
            let canonical_suffix = if hue == Some(CssHueInterpolationMethod::Shorter) {
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
                assert_eq!(value.to_specified_css().unwrap(), expected);
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
    let alias = (
        CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "xyz",
    );
    for (space, keyword) in RECTANGULAR.into_iter().chain(POLAR).chain([alias]) {
        for spelling in [
            keyword.to_owned(),
            keyword.to_ascii_uppercase(),
            format!("\\{:x} {}", keyword.as_bytes()[0], &keyword[1..]),
        ] {
            let source = format!("color-mix(in {spelling}, red, blue)");
            let canonical = if keyword == "xyz" { "xyz-d65" } else { keyword };
            let expected = if space == CssColorInterpolationSpace::Oklab {
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
                assert_eq!(value.to_specified_css().unwrap(), expected);
                for round_trip in admitted(&expected) {
                    assert_eq!(color(&round_trip).to_specified_css().unwrap(), expected);
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
                value.to_specified_css().unwrap(),
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
