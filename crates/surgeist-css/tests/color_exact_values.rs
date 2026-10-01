#![forbid(unsafe_code)]
//! Exact authored literals follow the product fidelity contract; calculations
//! retain their separately owned numeric semantics. These tests use public APIs.
use std::error::Error as _;

use surgeist_css::*;

fn component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1);
    values.items()[0].clone()
}

fn parsed(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("color:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}

fn checked(text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}

fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color property")
    };
    value
}

fn number<'a>(value: &'a CssColorComponent, expected: &str) -> &'a CssColorNumberLiteral {
    let CssColorComponent::Number(value) = value else {
        panic!("exact ordinary number")
    };
    assert_eq!(value.numeric().representation(), expected);
    value
}

#[test]
fn checked_literal_wrappers_preserve_domains_units_and_origins() {
    let supplied = component("1e100");
    let value = CssColorNumberLiteral::try_from_component(supplied.clone()).unwrap();
    assert_eq!(value.component(), &supplied);
    assert_eq!(value.origin(), supplied.origin());
    assert_eq!(value.numeric().representation(), "1e100");

    let supplied = component("0.1%");
    let value = CssColorPercentageLiteral::try_from_component(supplied.clone()).unwrap();
    assert_eq!(value.component(), &supplied);
    assert_eq!(value.origin(), supplied.origin());
    assert_eq!(value.numeric().representation(), "0.1");

    for (text, unit) in [
        ("1e100DEG", CssAngleUnit::Degrees),
        ("1e100grad", CssAngleUnit::Gradians),
        ("1e100rad", CssAngleUnit::Radians),
        ("1e100turn", CssAngleUnit::Turns),
        ("1e100d\\65 g", CssAngleUnit::Degrees),
    ] {
        let supplied = component(text);
        let value = CssAngleLiteral::try_from_component(supplied.clone()).unwrap();
        assert_eq!(value.unit(), unit);
        assert_eq!(value.numeric().representation(), "1e100");
        assert_eq!(value.component(), &supplied);
        assert_eq!(value.origin(), supplied.origin());
    }
    let supplied = CssComponentValue::try_number("0.1").unwrap();
    let value = CssColorNumberLiteral::try_from_component(supplied.clone()).unwrap();
    assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(value.component(), &supplied);
}

#[test]
fn wrong_literal_domains_report_the_original_invalid_token() {
    for text in ["1%", "1deg", "red", "calc(1)"] {
        let supplied = component(text);
        let error = CssColorNumberLiteral::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), supplied.origin());
    }
    for text in ["1", "1deg", "none"] {
        let supplied = component(text);
        let error = CssColorPercentageLiteral::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), supplied.origin());
    }
    for text in ["1", "1%", "1px"] {
        let supplied = component(text);
        let error = CssAngleLiteral::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), supplied.origin());
    }
}

#[test]
fn ordinary_huge_and_nonzero_underflow_channels_remain_exact() {
    for declaration in [
        parsed("rgb(1e100 1e-47 0.1 / 1e-100%)"),
        checked("rgb(1e100 1e-47 0.1 / 1e-100%)"),
    ] {
        let rgb = wrapper(&declaration).value().rgb_value().unwrap();
        for (value, expected) in rgb.channels().iter().zip(["1e100", "1e-47", "0.1"]) {
            let value = number(value, expected);
            assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        }
        let CssColorComponent::Percentage(alpha) = rgb.alpha().unwrap() else {
            panic!("exact percentage alpha")
        };
        assert_eq!(alpha.numeric().representation(), "1e-100");
        assert!(matches!(alpha.origin(), CssValueOrigin::Parsed(_)));
    }
}

#[test]
fn checked_mixed_origin_graph_keeps_the_actual_scalar_sources() {
    let mut children = vec![CssComponentValue::try_number("0.1").unwrap()];
    children.extend(
        parse_component_values("1e100 0")
            .unwrap()
            .items()
            .iter()
            .cloned(),
    );
    let children = CssComponentValues::try_new(children).unwrap();
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("rgb", children).unwrap(),
    ])
    .unwrap();
    assert_eq!(values.serialize().unwrap().as_css(), "rgb(0.1/**/1e100 0)");
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &values);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_value().is_none());
    let rgb = wrapper(&declaration).value().rgb_value().unwrap();
    assert_eq!(
        number(&rgb.channels()[0], "0.1").origin(),
        &CssValueOrigin::Programmatic
    );
    let CssValueOrigin::Parsed(origin) = number(&rgb.channels()[1], "1e100").origin() else {
        panic!("second component's original parsed source")
    };
    assert_eq!(origin.source().as_str(), "1e100 0");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
}

#[test]
fn formerly_overflowing_relative_slots_keep_their_typed_environment() {
    for (text, index, environment) in [
        (
            "rgb(from red 1e999 g b)",
            0,
            CssRelativeColorEnvironment::Rgb,
        ),
        (
            "rgb(from red r 1e999% b)",
            1,
            CssRelativeColorEnvironment::Rgb,
        ),
        (
            "hsl(from red 1e999deg s l)",
            0,
            CssRelativeColorEnvironment::Hsl,
        ),
    ] {
        for declaration in [parsed(text), checked(text)] {
            let relative = wrapper(&declaration).value().relative_value().unwrap();
            assert_eq!(relative.environment(), environment);
            let expression = &relative.channels()[index];
            assert_eq!(expression.environment(), environment);
            let (numeric, origin) = match expression.value() {
                CssRelativeColorExpressionValue::Number(value)
                    if environment == CssRelativeColorEnvironment::Rgb && index == 0 =>
                {
                    (value.numeric(), value.origin())
                }
                CssRelativeColorExpressionValue::Percentage(value) if index == 1 => {
                    (value.numeric(), value.origin())
                }
                CssRelativeColorExpressionValue::Angle(value)
                    if environment == CssRelativeColorEnvironment::Hsl =>
                {
                    assert_eq!(value.unit(), CssAngleUnit::Degrees);
                    assert_eq!(
                        expression.result_domain(),
                        CssRelativeColorResultDomain::Hue
                    );
                    (value.numeric(), value.origin())
                }
                _ => panic!("literal must retain its slot domain"),
            };
            assert_eq!(numeric.representation(), "1e999");
            assert!(matches!(origin, CssValueOrigin::Parsed(_)));
            assert_eq!(expression.origin(), origin);
        }
    }
    for text in ["hsl(from red 1e999% s l)", "rgb(from red 1e999deg g b)"] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                parse_component_values(text).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn hue_and_percentage_coefficient_domains_survive_without_normalizing() {
    for declaration in [
        parsed("hsl(1e100turn 0.1% 25%)"),
        checked("hsl(1e100turn 0.1% 25%)"),
    ] {
        let hsl = wrapper(&declaration).value().hsl_value().unwrap();
        let CssColorHue::Angle(hue) = hsl.hue() else {
            panic!("exact hue angle")
        };
        assert_eq!(hue.unit(), CssAngleUnit::Turns);
        assert_eq!(hue.numeric().representation(), "1e100");
        let CssColorComponent::Percentage(value) = hsl.saturation() else {
            panic!("exact coefficient")
        };
        assert_eq!(value.numeric().representation(), "0.1");
        assert!(
            matches!(hsl.lightness(), CssColorComponent::Percentage(value) if value.numeric().representation() == "25")
        );
    }
    let declaration = parsed("hsl(0.1 25% 50%)");
    let CssColorHue::Number(hue) = wrapper(&declaration).value().hsl_value().unwrap().hue() else {
        panic!("unitless exact hue")
    };
    assert_eq!(hue.numeric().representation(), "0.1");
}

#[test]
fn mix_weights_distinguish_exact_range_errors_from_component_errors() {
    for text in ["0.1%", "1e-47%", "-0%", "25%", "100%"] {
        let supplied = component(text);
        let weight = CssColorMixPercentage::try_from_component(supplied.clone()).unwrap();
        assert_eq!(weight.literal().component(), &supplied);
        assert_eq!(weight.literal().origin(), supplied.origin());
        assert_eq!(
            weight.literal().numeric().representation(),
            text.trim_end_matches('%')
        );
    }
    for text in [
        "-1e-100%".to_owned(),
        "100.0000000000000000000001%".to_owned(),
        format!("100.{}1%", "0".repeat(256)),
    ] {
        let supplied = component(&text);
        let error = CssColorMixPercentage::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssColorScalarErrorKind::OutOfRange);
        assert_eq!(error.origin(), supplied.origin());
        assert!(error.source().is_none());
    }
    let supplied = CssComponentValue::try_number("25").unwrap();
    let error = CssColorMixPercentage::try_from_component(supplied).unwrap_err();
    assert_eq!(error.kind(), CssColorScalarErrorKind::InvalidComponent);
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    assert!(error.source().is_some());
}

#[test]
fn finite_high_precision_mix_weights_retain_original_decimal() {
    for coefficient in [
        "50.000003814697265625",
        "1.40129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125e-45",
    ] {
        let text = format!("color-mix(in srgb, red {coefficient}%, blue)");
        for declaration in [parsed(&text), checked(&text)] {
            let mix = wrapper(&declaration).value().color_mix_value().unwrap();
            let weight = mix.components()[0]
                .weight()
                .unwrap()
                .literal_value()
                .unwrap();
            assert_eq!(weight.literal().numeric().representation(), coefficient);
            assert!(matches!(
                weight.literal().origin(),
                CssValueOrigin::Parsed(_)
            ));
            assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
            assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
        }
    }
}

#[test]
fn percentage_slots_retain_the_authored_coefficient_without_scaling() {
    for (text, expected) in [
        ("hsl(0 30% 50%)", "30"),
        ("hwb(0 30% 25%)", "30"),
        ("color(srgb 30% 0% 100%)", "30"),
        ("rgb(0 0 0 / 30%)", "30"),
        ("oklab(50% 50% 0%)", "50"),
        ("oklch(50% 10% 0)", "10"),
        (
            "lab(50% 50.000003814697265625% 0%)",
            "50.000003814697265625",
        ),
        ("lch(50% 50.000003814697265625% 0)", "50.000003814697265625"),
    ] {
        for declaration in [parsed(text), checked(text)] {
            let current = wrapper(&declaration).value();
            let slot = if let Some(value) = current.hsl_value() {
                value.saturation()
            } else if let Some(value) = current.hwb_value() {
                value.whiteness()
            } else if let Some(value) = current.predefined_value() {
                &value.channels()[0]
            } else if let Some(value) = current.rgb_value() {
                value.alpha().unwrap()
            } else if let Some(value) = current.oklab_value() {
                value.a()
            } else if let Some(value) = current.oklch_value() {
                value.chroma()
            } else if let Some(value) = current.lab_value() {
                value.a()
            } else {
                current.lch_value().unwrap().chroma()
            };
            let CssColorComponent::Percentage(value) = slot else {
                panic!("percentage component: {text}")
            };
            assert_eq!(value.numeric().representation(), expected, "{text}");
            assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        }
    }
    // A relative expression likewise retains its authored coefficient rather
    // than applying a color-space percentage basis during parsing.
    let declaration = parsed("rgb(from red 30% g b)");
    let relative = wrapper(&declaration).value().relative_value().unwrap();
    assert!(
        matches!(relative.channels()[0].value(), CssRelativeColorExpressionValue::Percentage(value) if value.numeric().representation() == "30")
    );
}

#[test]
fn border_color_assignments_retain_exact_nested_colors() {
    for text in ["hsl(0 30% 50%)", "color-mix(in srgb, red 0.1%, blue)"] {
        let report = parse_style_attribute(&format!("border-color:{text}"));
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("border colors")
        };
        let colors = value.value();
        assert_eq!(colors.kind(), CssBoxSideKind::Physical);
        assert_eq!(colors.authored_values().len(), 1);
        let [top, right, bottom, left] = colors.assigned_values();
        assert_eq!(top, right);
        assert_eq!(top, bottom);
        assert_eq!(top, left);
        if let Some(hsl) = top.hsl_value() {
            let CssColorComponent::Percentage(saturation) = hsl.saturation() else {
                panic!("authored HSL saturation")
            };
            assert_eq!(saturation.numeric().representation(), "30");
        } else {
            let weight = top.color_mix_value().unwrap().components()[0]
                .weight()
                .unwrap()
                .literal_value()
                .unwrap();
            assert_eq!(weight.literal().numeric().representation(), "0.1");
        }
    }
    let report = parse_style_attribute("border-color:hsl(0 25% 50%)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("border colors")
    };
    assert!(
        matches!(value.value().assigned_values()[0].hsl_value().unwrap().saturation(), CssColorComponent::Percentage(value) if value.numeric().representation() == "25")
    );
    let report = parse_style_attribute(
        "border-color: red hsl(0 30% 50%) color-mix(in srgb, blue 0.1%, green) currentcolor !important",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("border colors")
    };
    let [top, right, bottom, left] = value.value().assigned_values();
    assert_eq!(top.named().unwrap().name(), "red");
    assert!(
        matches!(right.hsl_value().unwrap().saturation(), CssColorComponent::Percentage(value)
        if value.numeric().representation() == "30")
    );
    assert_eq!(
        bottom.color_mix_value().unwrap().components()[0]
            .weight()
            .unwrap()
            .literal_value()
            .unwrap()
            .literal()
            .numeric()
            .representation(),
        "0.1"
    );
    assert!(left.is_current_color());
}

#[test]
fn ordinary_exact_literals_and_math_leaves_keep_distinct_phases() {
    let declaration = parsed("rgb(1e-47 calc(1e-47) 0.100000001490116119384765625)");
    let rgb = wrapper(&declaration).value().rgb_value().unwrap();
    number(&rgb.channels()[0], "1e-47");
    let CssColorComponent::NumberCalculation(calculation) = &rgb.channels()[1] else {
        panic!("typed number calculation")
    };
    assert_eq!(calculation.result_type(), CssCalculationType::Number);
    let mut expression = calculation.expression();
    loop {
        match expression {
            CssCalculationExpressionRef::NestedCalc(value)
            | CssCalculationExpressionRef::Group(value) => expression = value.operand(),
            CssCalculationExpressionRef::Value(value) => {
                assert_eq!(value.literal().representation(), "1e-47");
                break;
            }
            _ => panic!("one literal calculation leaf"),
        }
    }
    number(&rgb.channels()[2], "0.100000001490116119384765625");
}

#[test]
fn decimal_proof_capacity_does_not_limit_exact_literal_admission() {
    let coefficient = format!("1{}1", "0".repeat(512));
    for text in [
        coefficient,
        "1e999999999999999999999999999999999999999".to_owned(),
        "1e-999999999999999999999999999999999999999".to_owned(),
    ] {
        let declaration = checked(&format!("rgb({text} 0 0)"));
        number(
            &wrapper(&declaration)
                .value()
                .rgb_value()
                .unwrap()
                .channels()[0],
            &text,
        );
    }
    let source = "rgb(1e100 0 0)";
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap();
    let error = parse_component_values_with_limits(source, limits).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
    let limits = CssComponentValueLimits::try_new(256, usize::MAX, source.len()).unwrap();
    let values = parse_component_values_with_limits(source, limits).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values,
        CssImportance::Normal,
    )
    .unwrap();
    number(
        &wrapper(&declaration)
            .value()
            .rgb_value()
            .unwrap()
            .channels()[0],
        "1e100",
    );
}

#[test]
fn perceptual_families_retain_exact_authored_percentages() {
    for text in ["lab(50% 20% 0%)", "lch(50% 20% 0)", "oklab(50% 0% 0%)"] {
        for declaration in [parsed(text), checked(text)] {
            let color = wrapper(&declaration).value();
            if let Some(lab) = color.lab_value() {
                let CssColorComponent::Percentage(lightness) = lab.lightness() else {
                    panic!("Lab lightness percentage")
                };
                let CssColorComponent::Percentage(a) = lab.a() else {
                    panic!("Lab a percentage")
                };
                let CssColorComponent::Percentage(b) = lab.b() else {
                    panic!("Lab b percentage")
                };
                assert_eq!(lightness.numeric().representation(), "50");
                assert_eq!(a.numeric().representation(), "20");
                assert_eq!(b.numeric().representation(), "0");
            } else if let Some(lch) = color.lch_value() {
                let CssColorComponent::Percentage(lightness) = lch.lightness() else {
                    panic!("LCH lightness percentage")
                };
                let CssColorComponent::Percentage(chroma) = lch.chroma() else {
                    panic!("LCH chroma percentage")
                };
                assert_eq!(lightness.numeric().representation(), "50");
                assert_eq!(chroma.numeric().representation(), "20");
                assert!(matches!(lch.hue(), CssColorHue::Number(value)
                    if value.numeric().representation() == "0"));
            } else {
                let oklab = color.oklab_value().expect("Oklab value");
                let CssColorComponent::Percentage(lightness) = oklab.lightness() else {
                    panic!("Oklab lightness percentage")
                };
                assert_eq!(lightness.numeric().representation(), "50");
                let CssColorComponent::Percentage(a) = oklab.a() else {
                    panic!("Oklab a percentage")
                };
                let CssColorComponent::Percentage(b) = oklab.b() else {
                    panic!("Oklab b percentage")
                };
                assert_eq!(a.numeric().representation(), "0");
                assert_eq!(b.numeric().representation(), "0");
            }
        }
    }
}

#[test]
fn deep_mixed_color_graph_retains_exact_leaf_at_the_shared_depth_boundary() {
    fn nested(depth: usize) -> String {
        let mut text = "rgb(1e100 0 0)".to_owned();
        for level in 1..depth {
            text = if level % 2 == 0 {
                format!("rgb(from {text} r g b)")
            } else {
                format!("color-mix(in srgb, {text}, blue)")
            };
        }
        text
    }
    for depth in [255, 256] {
        let text = nested(depth);
        for declaration in [parsed(&text), checked(&text)] {
            let value = wrapper(&declaration);
            let mut current = value.value();
            let mut visited = 1;
            loop {
                if let Some(mix) = current.color_mix_value() {
                    current = mix.components()[0].color();
                } else if let Some(relative) = current.relative_value() {
                    current = relative.source();
                } else {
                    break;
                }
                visited += 1;
            }
            assert_eq!(visited, depth);
            number(&current.rgb_value().unwrap().channels()[0], "1e100");
            // The owned declaration and its exact descendant drop normally here.
        }
    }
    let text = nested(257);
    let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
    let [diagnostic] = report.diagnostics() else {
        panic!("one depth failure")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    // Checked property input cannot bypass the component graph's same ceiling.
    assert_eq!(
        parse_component_values(&text).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
}
