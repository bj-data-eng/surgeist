#![forbid(unsafe_code)]
//! Exceptional authored hue calculations under the selected Color 4 phases.
//! Color 4 CRD 2026-09-08 §§4.3, 15.1–15.3:
//! https://www.w3.org/TR/2026/CRD-css-color-4-20260908/#hue-syntax
//! SHA-256: bada647312a73c4cd4484dc177d67d22cf70df6c451b0e8c5dd5efc29c3415ba
//! Values 4 WD 2024-03-12 §§10.12–10.13 retain specified calculations and serialize
//! exceptional angles in the canonical degree unit:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-serialize
//! SHA-256: 7b6b68c34d00d7f6945e66e4e2efa913266299f597d5825393cae85dac1cc78c
//! The catalog's css-color-4-lab-ok-calculated-range-phase selects non-eager
//! LCH/OKLCH specified wrappers. Ordinary HSL/HWB resolve scalar hue to zero,
//! so HSL(0,100%,50%) and HWB(0,0%,0%) independently yield opaque sRGB red.
//! This characterizes Standalone/Mix projection, not computed LCH/OKLCH hue.

use surgeist_css::*;

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

fn literal(text: &str) -> CssColorComponent {
    let values = parse_component_values(text).unwrap();
    let token = values.items()[0].clone();
    if text.ends_with('%') {
        CssColorComponent::Percentage(CssColorPercentageLiteral::try_from_component(token).unwrap())
    } else {
        CssColorComponent::Number(CssColorNumberLiteral::try_from_component(token).unwrap())
    }
}

fn calculated_hue(text: &str, angle: bool) -> CssColorHue {
    let components = parse_component_values(text).unwrap();
    if angle {
        CssColorHue::AngleCalculation(CssAngleCalculation::try_from_components(components).unwrap())
    } else {
        CssColorHue::NumberCalculation(
            CssNumberCalculation::try_from_components(components).unwrap(),
        )
    }
}

fn constructed(family: &str, calculation: &str, angle: bool) -> CssColor {
    let hue = calculated_hue(calculation, angle);
    match family {
        "hsl" => CssColor::from_hsl(
            CssHslColor::try_new(
                CssColorSyntax::Modern,
                hue,
                literal("100%"),
                literal("50%"),
                None,
            )
            .unwrap(),
        ),
        "hwb" => CssColor::from_hwb(
            CssHwbColor::try_new(hue, literal("0%"), literal("0%"), None).unwrap(),
        ),
        "lch" => CssColor::from_lch(
            CssLchColor::try_new(literal("50"), literal("10"), hue, None).unwrap(),
        ),
        "oklch" => CssColor::from_oklch(
            CssLchColor::try_new(literal(".5"), literal(".1"), hue, None).unwrap(),
        ),
        _ => panic!("unknown test family"),
    }
}

fn hue_of<'a>(color: &'a CssColor, family: &str) -> &'a CssColorHue {
    match family {
        "hsl" => color.hsl_value().unwrap().hue(),
        "hwb" => color.hwb_value().unwrap().hue(),
        "lch" => color.lch_value().unwrap().hue(),
        "oklch" => color.oklch_value().unwrap().hue(),
        _ => panic!("unknown test family"),
    }
}

fn assert_raw(color: &CssColor, family: &str, calculation: &str, angle: bool) {
    let (components, origin) = match (hue_of(color, family), angle) {
        (CssColorHue::NumberCalculation(value), false) => {
            assert_eq!(value.result_type(), CssCalculationType::Number);
            (value.components(), value.origin())
        }
        (CssColorHue::AngleCalculation(value), true) => {
            assert_eq!(value.result_type(), CssCalculationType::Angle);
            (value.components(), value.origin())
        }
        other => panic!("authored calculation domain changed: {other:?}"),
    };
    assert_eq!(components.serialize().unwrap().as_css(), calculation);
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("retained calculation provenance")
    };
    let source = origin.source().as_str();
    let begin = origin.span().start().byte_offset().value();
    assert_eq!(&source[begin..begin + calculation.len()], calculation);
    match family {
        "hsl" => {
            let value = color.hsl_value().unwrap();
            assert_eq!(value.syntax(), CssColorSyntax::Modern);
            assert!(value.alpha().is_none());
            assert_scalar(value.saturation(), "100", true);
            assert_scalar(value.lightness(), "50", true);
        }
        "hwb" => {
            let value = color.hwb_value().unwrap();
            assert!(value.alpha().is_none());
            assert_scalar(value.whiteness(), "0", true);
            assert_scalar(value.blackness(), "0", true);
        }
        "lch" => {
            let value = color.lch_value().unwrap();
            assert!(value.alpha().is_none());
            assert_scalar(value.lightness(), "50", false);
            assert_scalar(value.chroma(), "10", false);
        }
        "oklch" => {
            let value = color.oklch_value().unwrap();
            assert!(value.alpha().is_none());
            assert_scalar(value.lightness(), ".5", false);
            assert_scalar(value.chroma(), ".1", false);
        }
        _ => unreachable!(),
    }
}

fn assert_scalar(component: &CssColorComponent, representation: &str, percentage: bool) {
    match (component, percentage) {
        (CssColorComponent::Number(value), false) => {
            assert_eq!(value.numeric().representation(), representation)
        }
        (CssColorComponent::Percentage(value), true) => {
            assert_eq!(value.numeric().representation(), representation)
        }
        other => panic!("sibling domain changed: {other:?}"),
    }
}

fn source(family: &str, calculation: &str) -> String {
    match family {
        "hsl" => format!("hsl({calculation} 100% 50%)"),
        "hwb" => format!("hwb({calculation} 0% 0%)"),
        "lch" => format!("lch(50 10 {calculation})"),
        "oklch" => format!("oklch(.5 .1 {calculation})"),
        _ => unreachable!(),
    }
}

fn assert_case(family: &str, calculation: &str, angle: bool, expected: &str) {
    let text = source(family, calculation);
    for color in [parsed(&text), constructed(family, calculation, angle)] {
        assert_raw(&color, family, calculation, angle);
        let before = color.clone();
        assert_eq!(color.to_specified_css().unwrap(), expected, "{text}");
        assert_eq!(color, before);
        assert_raw(&color, family, calculation, angle);
    }
    let mix_text = format!("color-mix({text}, blue)");
    let checked_mix = CssColor::from_color_mix(
        CssColorMix::try_new(
            None,
            vec![
                CssColorMixComponent::new(constructed(family, calculation, angle), None),
                CssColorMixComponent::new(
                    CssColor::from_named(CssNamedColor::try_new("blue").unwrap()),
                    None,
                ),
            ],
        )
        .unwrap(),
    );
    for mix in [parsed(&mix_text), checked_mix] {
        let child = mix.color_mix_value().unwrap().components()[0].color();
        assert_raw(child, family, calculation, angle);
        let before = mix.clone();
        assert_eq!(
            mix.to_specified_css().unwrap(),
            format!("color-mix({expected}, blue)"),
            "{mix_text}"
        );
        assert_eq!(mix, before);
        assert_raw(
            mix.color_mix_value().unwrap().components()[0].color(),
            family,
            calculation,
            angle,
        );
    }
}

#[test]
fn ordinary_hsl_hwb_number_infinities_resolve_to_zero_hue_without_mutating_calculations() {
    for family in ["hsl", "hwb"] {
        for calculation in ["calc(infinity)", "calc(-infinity)"] {
            assert_case(family, calculation, false, "rgb(255, 0, 0)");
        }
    }
}

#[test]
fn ordinary_hsl_hwb_angle_infinities_resolve_to_zero_hue_without_mutating_calculations() {
    for family in ["hsl", "hwb"] {
        for calculation in ["calc(infinity * 1deg)", "calc(-infinity * 1deg)"] {
            assert_case(family, calculation, true, "rgb(255, 0, 0)");
        }
    }
}

#[test]
fn specified_lch_oklch_number_infinities_keep_the_signed_calculation_wrappers() {
    for family in ["lch", "oklch"] {
        for calculation in ["calc(infinity)", "calc(-infinity)"] {
            let expected = if family == "lch" {
                format!("lch(50 10 {calculation})")
            } else {
                format!("oklch(0.5 0.1 {calculation})")
            };
            assert_case(family, calculation, false, &expected);
        }
    }
}

#[test]
fn specified_lch_oklch_angle_infinities_keep_the_signed_degree_calculation_wrappers() {
    for family in ["lch", "oklch"] {
        for calculation in ["calc(infinity * 1deg)", "calc(-infinity * 1deg)"] {
            let expected = if family == "lch" {
                format!("lch(50 10 {calculation})")
            } else {
                format!("oklch(0.5 0.1 {calculation})")
            };
            assert_case(family, calculation, true, &expected);
        }
    }
}
