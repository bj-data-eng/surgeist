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

fn number<'a>(value: &'a CssAuthoredColorComponent, expected: &str) -> &'a CssColorNumberLiteral {
    let CssAuthoredColorComponent::ExactNumber(value) = value else {
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
        let value = CssColorAngleLiteral::try_from_component(supplied.clone()).unwrap();
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
        let error = CssColorAngleLiteral::try_from_component(supplied.clone()).unwrap_err();
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
        let rgb = wrapper(&declaration).current().rgb_value().unwrap();
        for (value, expected) in rgb.channels().iter().zip(["1e100", "1e-47", "0.1"]) {
            let value = number(value, expected);
            assert!(matches!(value.origin(), CssValueOrigin::Parsed(_)));
        }
        let CssAuthoredColorComponent::ExactPercentage(alpha) = rgb.alpha().unwrap() else {
            panic!("exact percentage alpha")
        };
        assert_eq!(alpha.numeric().representation(), "1e-100");
        assert!(wrapper(&declaration).i01_subset().is_none());
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
    let rgb = wrapper(&declaration).current().rgb_value().unwrap();
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
            let relative = wrapper(&declaration).current().relative_value().unwrap();
            assert_eq!(relative.environment(), environment);
            let expression = &relative.channels()[index];
            assert_eq!(expression.environment(), environment);
            let (numeric, origin) = match expression.value() {
                CssRelativeColorExpressionValue::ExactNumber(value)
                    if environment == CssRelativeColorEnvironment::Rgb && index == 0 =>
                {
                    (value.numeric(), value.origin())
                }
                CssRelativeColorExpressionValue::ExactPercentage(value) if index == 1 => {
                    (value.numeric(), value.origin())
                }
                CssRelativeColorExpressionValue::ExactAngle(value)
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
            assert!(wrapper(&declaration).i01_subset().is_none());
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
        let hsl = wrapper(&declaration).current().hsl_value().unwrap();
        let CssAuthoredHue::ExactAngle(hue) = hsl.hue() else {
            panic!("exact hue angle")
        };
        assert_eq!(hue.unit(), CssAngleUnit::Turns);
        assert_eq!(hue.numeric().representation(), "1e100");
        let CssAuthoredColorComponent::ExactPercentage(value) = hsl.saturation() else {
            panic!("exact coefficient")
        };
        assert_eq!(value.numeric().representation(), "0.1");
        assert!(
            matches!(hsl.lightness(), CssAuthoredColorComponent::Percentage(value) if value.value() == 25.0)
        );
    }
    let declaration = parsed("hsl(0.1 25% 50%)");
    let CssAuthoredHue::ExactNumber(hue) =
        wrapper(&declaration).current().hsl_value().unwrap().hue()
    else {
        panic!("unitless exact hue")
    };
    assert_eq!(hue.numeric().representation(), "0.1");
}

#[test]
fn mix_weights_distinguish_exact_range_errors_from_component_errors() {
    for text in ["0.1%", "1e-47%"] {
        let supplied = component(text);
        let weight = CssAuthoredColorMixPercentage::try_from_component(supplied.clone()).unwrap();
        assert_eq!(weight.value(), None);
        assert_eq!(weight.exact_literal().unwrap().component(), &supplied);
    }
    for text in [
        "-1e-100%".to_owned(),
        "100.0000000000000000000001%".to_owned(),
        format!("100.{}1%", "0".repeat(256)),
    ] {
        let supplied = component(&text);
        let error =
            CssAuthoredColorMixPercentage::try_from_component(supplied.clone()).unwrap_err();
        assert_eq!(error.kind(), CssColorScalarErrorKind::OutOfRange);
        assert_eq!(error.origin(), supplied.origin());
        assert!(error.source().is_none());
    }
    let supplied = CssComponentValue::try_number("25").unwrap();
    let error = CssAuthoredColorMixPercentage::try_from_component(supplied).unwrap_err();
    assert_eq!(error.kind(), CssColorScalarErrorKind::InvalidComponent);
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    assert!(error.source().is_some());
    for (text, expected) in [("-0%", 0.0), ("25%", 25.0), ("100%", 100.0)] {
        let weight = CssAuthoredColorMixPercentage::try_from_component(component(text)).unwrap();
        assert_eq!(weight.value(), Some(expected));
        assert!(weight.exact_literal().is_none());
    }
    let supplied_binary32 = CssAuthoredColorMixPercentage::try_new(0.1).unwrap();
    assert_eq!(supplied_binary32.value(), Some(0.1));
    assert!(supplied_binary32.exact_literal().is_none());
    assert!(CssAuthoredColorMixPercentage::try_new(f32::INFINITY).is_none());
}

#[test]
fn finite_coefficients_still_refuse_lossy_frozen_percentage_round_trips() {
    // These decimals denote exactly 0x42480001 and the smallest binary32 subnormal.
    for (coefficient, expected) in [
        ("50.000003814697265625", f32::from_bits(0x4248_0001)),
        (
            "1.40129846432481707092372958328991613128026194187651577175706828388979108268586060148663818836212158203125e-45",
            f32::from_bits(1),
        ),
    ] {
        let text = format!("color-mix(in srgb, red {coefficient}%, blue)");
        for declaration in [parsed(&text), checked(&text)] {
            let mix = wrapper(&declaration).current().color_mix_value().unwrap();
            let weight = mix.left().percentage().unwrap();
            assert_eq!(weight.value(), Some(expected));
            assert!(weight.exact_literal().is_none());
            assert!(wrapper(&declaration).i01_subset().is_none());
        }
    }
}

#[test]
fn projection_compares_the_actual_percentage_basis_in_each_slot() {
    // Every coefficient is binary32. The frozen ratio or perceptual result is
    // not exact: .3, .2, .04 and these scaled Lab/LCH coefficients
    // cannot be represented by binary32.
    for (text, expected) in [
        ("hsl(0 30% 50%)", 30.0),
        ("hwb(0 30% 25%)", 30.0),
        ("color(srgb 30% 0% 100%)", 30.0),
        ("rgb(0 0 0 / 30%)", 30.0),
        ("oklab(50% 50% 0%)", 50.0),
        ("oklch(50% 10% 0)", 10.0),
        (
            "lab(50% 50.000003814697265625% 0%)",
            f32::from_bits(0x4248_0001),
        ),
        (
            "lch(50% 50.000003814697265625% 0)",
            f32::from_bits(0x4248_0001),
        ),
    ] {
        for declaration in [parsed(text), checked(text)] {
            let current = wrapper(&declaration).current();
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
            assert!(
                matches!(slot, CssAuthoredColorComponent::Percentage(value) if value.value() == expected),
                "finite current coefficient: {text}"
            );
            assert!(wrapper(&declaration).i01_subset().is_none(), "{text}");
        }
    }
    for text in [
        "hsl(0 25% 50%)",
        "color(srgb 25% 0% 100%)",
        "color-mix(in srgb, red 25%, blue 50%)",
    ] {
        assert!(wrapper(&parsed(text)).i01_subset().is_some(), "{text}");
    }
    // Relative expressions store retained token text, not the frozen absolute
    // channel ratio. Its finite coefficient must remain 30, not 0.3 or 3000.
    let declaration = parsed("rgb(from red 30% g b)");
    let relative = wrapper(&declaration).current().relative_value().unwrap();
    assert!(
        matches!(relative.channels()[0].value(), CssRelativeColorExpressionValue::Percentage(value) if value.value() == 30.0)
    );
}

#[test]
fn aggregate_projection_cannot_reparse_an_unrepresentable_child() {
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
        assert!(value.i01_subset().is_none(), "{text}");
        let colors = value.current();
        assert_eq!(colors.top(), colors.right());
        assert_eq!(colors.top(), colors.bottom());
        assert_eq!(colors.top(), colors.left());
        if let Some(hsl) = colors.top().hsl_value() {
            assert!(
                matches!(hsl.saturation(), CssAuthoredColorComponent::Percentage(value) if value.value() == 30.0)
            );
        } else {
            let weight = colors
                .top()
                .color_mix_value()
                .unwrap()
                .left()
                .percentage()
                .unwrap();
            assert_eq!(
                weight.exact_literal().unwrap().numeric().representation(),
                "0.1"
            );
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
    assert!(value.i01_subset().is_some());
    assert!(
        matches!(value.current().top().hsl_value().unwrap().saturation(), CssAuthoredColorComponent::Percentage(value) if value.value() == 25.0)
    );
}

#[test]
fn ordinary_exact_literals_and_math_leaves_keep_distinct_phases() {
    let declaration = parsed("rgb(1e-47 calc(1e-47) 0.100000001490116119384765625)");
    let rgb = wrapper(&declaration).current().rgb_value().unwrap();
    number(&rgb.channels()[0], "1e-47");
    let CssAuthoredColorComponent::NumberCalculation(calculation) = &rgb.channels()[1] else {
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
    assert!(
        matches!(&rgb.channels()[2], CssAuthoredColorComponent::Number(value) if value.value() == f32::from_bits(0x3dcc_cccd))
    );
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
                .current()
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
            .current()
            .rgb_value()
            .unwrap()
            .channels()[0],
        "1e100",
    );
}

#[test]
fn perceptual_percentage_projection_keeps_exact_scaled_frozen_values() {
    // Color4 bases: Lab a=125, LCH C=150, Oklab L=1.
    // Therefore 20% maps exactly to 25 or 30, and 50% Oklab L to 0.5.
    for text in ["lab(50% 20% 0%)", "lch(50% 20% 0)", "oklab(50% 0% 0%)"] {
        for declaration in [parsed(text), checked(text)] {
            let value = wrapper(&declaration);
            match value
                .i01_subset()
                .expect("exact perceptual percentage projection")
            {
                CssColor::Lab(frozen) => {
                    assert_eq!(frozen.lightness(), Some(50.0));
                    assert_eq!(frozen.a(), Some(25.0));
                    assert_eq!(frozen.b(), Some(0.0));
                    assert!(
                        matches!(value.current().lab_value().unwrap().a(), CssAuthoredColorComponent::Percentage(value) if value.value() == 20.0)
                    );
                }
                CssColor::Lch(frozen) => {
                    assert_eq!(frozen.lightness(), Some(50.0));
                    assert_eq!(frozen.chroma(), Some(30.0));
                    assert_eq!(frozen.hue(), Some(0.0));
                    assert!(
                        matches!(value.current().lch_value().unwrap().chroma(), CssAuthoredColorComponent::Percentage(value) if value.value() == 20.0)
                    );
                }
                CssColor::Oklab(frozen) => {
                    assert_eq!(frozen.lightness(), Some(0.5));
                    assert_eq!(frozen.a(), Some(0.0));
                    assert_eq!(frozen.b(), Some(0.0));
                    assert!(
                        matches!(value.current().oklab_value().unwrap().lightness(), CssAuthoredColorComponent::Percentage(value) if value.value() == 50.0)
                    );
                }
                _ => panic!("same perceptual color family"),
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
            assert!(value.i01_subset().is_none());
            let mut current = value.current();
            let mut visited = 1;
            loop {
                if let Some(mix) = current.color_mix_value() {
                    current = mix.left().color();
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
