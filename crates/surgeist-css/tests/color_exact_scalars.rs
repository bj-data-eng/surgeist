#![forbid(unsafe_code)]
//! Exact authored color scalars under the selected Color4/Color5 grammars.
use surgeist_css::*;

fn color(declaration: &CssDeclaration) -> &CssColor {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    value.value()
}
fn both(text: &str) -> [CssDeclaration; 2] {
    let source = format!("color:{text}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [parsed] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    assert_eq!(parsed.importance(), CssImportance::Important);
    assert_eq!(
        parsed.value_components().serialize().unwrap().as_css(),
        text
    );
    let CssValueOrigin::Parsed(origin) = parsed.value_components().items()[0].origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    let values = parse_component_values(text).unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Color),
        values.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(checked.value_components(), &values);
    assert!(checked.position().is_none());
    assert!(checked.parsed_value().is_none());
    [parsed.clone(), checked]
}
fn number(value: &CssColorComponent, spelling: &str) {
    let CssColorComponent::Number(v) = value else {
        panic!("number: {value:?}")
    };
    assert_eq!(v.numeric().representation(), spelling);
    assert_eq!(v.component().origin(), v.origin());
}
fn percentage(value: &CssColorComponent, spelling: &str) {
    let CssColorComponent::Percentage(v) = value else {
        panic!("percentage: {value:?}")
    };
    assert_eq!(v.numeric().representation(), spelling);
    assert_eq!(v.component().origin(), v.origin());
}
fn mix_weight(mix: &CssColorMix, index: usize, spelling: &str) {
    let literal = mix.components()[index]
        .weight()
        .unwrap()
        .literal_value()
        .unwrap()
        .literal();
    assert_eq!(literal.numeric().representation(), spelling);
    assert_eq!(literal.component().origin(), literal.origin());
}

#[test]
fn number_percentage_and_angle_literals_keep_extreme_exact_coefficients() {
    for (text, spelling) in [("rgb(1e100 0 0)", "1e100"), ("rgb(1e-47 0 0)", "1e-47")] {
        for declaration in both(text) {
            number(
                &color(&declaration).rgb_value().unwrap().channels()[0],
                spelling,
            );
        }
    }
    for declaration in both("lab(1e100% 0 0)") {
        percentage(
            color(&declaration).lab_value().unwrap().lightness(),
            "1e100",
        );
    }
    for declaration in both("hsl(1e100deg 25% 50%)") {
        let hsl = color(&declaration).hsl_value().unwrap();
        let CssColorHue::Angle(hue) = hsl.hue() else {
            panic!("angle hue")
        };
        assert_eq!(hue.numeric().representation(), "1e100");
        assert_eq!(hue.unit(), CssAngleUnit::Degrees);
        assert_eq!(hue.component().origin(), hue.origin());
        percentage(hsl.saturation(), "25");
        percentage(hsl.lightness(), "50");
    }
}

#[test]
fn relative_literals_keep_exact_coefficients_and_channel_references() {
    for (text, spelling) in [
        ("rgb(from red 1e100 g b)", "1e100"),
        ("rgb(from red 0.1 g b)", "0.1"),
    ] {
        for declaration in both(text) {
            let relative = color(&declaration).relative_value().unwrap();
            assert_eq!(relative.source().named().unwrap().name(), "red");
            let CssRelativeColorExpressionValue::Number(v) = relative.channels()[0].value() else {
                panic!("number")
            };
            assert_eq!(v.numeric().representation(), spelling);
            assert_eq!(v.component().origin(), v.origin());
            assert!(matches!(
                relative.channels()[1].value(),
                CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::G)
            ));
            assert!(matches!(
                relative.channels()[2].value(),
                CssRelativeColorExpressionValue::Channel(CssRelativeColorChannel::B)
            ));
        }
    }
}

#[test]
fn predefined_none_math_and_exact_channel_remain_distinct() {
    for declaration in both("color(display-p3-linear none calc(0.5) 1e100)") {
        let value = color(&declaration).predefined_value().unwrap();
        assert_eq!(
            value.color_space(),
            CssPredefinedColorSpace::DisplayP3Linear
        );
        assert!(matches!(value.channels()[0], CssColorComponent::None));
        assert!(
            matches!(&value.channels()[1], CssColorComponent::NumberCalculation(calc) if calc.components().serialize().unwrap().as_css() == "calc(0.5)")
        );
        number(&value.channels()[2], "1e100");
    }
}

#[test]
fn mix_weights_keep_precision_sign_and_underflow_spelling() {
    for (text, first, second) in [
        (
            "color-mix(in srgb, red 50.000003814697265625%, blue)",
            "50.000003814697265625",
            None,
        ),
        ("color-mix(in srgb, red 0.1%, blue)", "0.1", None),
        ("color-mix(in srgb, red 1e-47%, blue)", "1e-47", None),
        ("color-mix(in srgb, red 25%, blue 50%)", "25", Some("50")),
        ("color-mix(in srgb, red -0%, blue)", "-0", None),
        ("color-mix(in srgb, red 100%, blue)", "100", None),
    ] {
        for declaration in both(text) {
            let mix = color(&declaration).color_mix_value().unwrap();
            assert_eq!(mix.components().len(), 2);
            assert_eq!(mix.components()[0].color().named().unwrap().name(), "red");
            assert_eq!(mix.components()[1].color().named().unwrap().name(), "blue");
            mix_weight(mix, 0, first);
            if let Some(second) = second {
                mix_weight(mix, 1, second);
            } else {
                assert!(mix.components()[1].weight().is_none());
            }
        }
    }
}

#[test]
fn nested_relative_and_mix_keep_exact_descendant_literals() {
    for declaration in both("rgb(from color-mix(in srgb, red 0.1%, blue) r g b)") {
        let relative = color(&declaration).relative_value().unwrap();
        mix_weight(relative.source().color_mix_value().unwrap(), 0, "0.1");
    }
    for declaration in both("color-mix(in srgb, rgb(from red 0.1 g b), blue)") {
        let mix = color(&declaration).color_mix_value().unwrap();
        let relative = mix.components()[0].color().relative_value().unwrap();
        let CssRelativeColorExpressionValue::Number(v) = relative.channels()[0].value() else {
            panic!("nested number")
        };
        assert_eq!(v.numeric().representation(), "0.1");
    }
}

#[test]
fn exact_mix_weight_range_rejects_nonzero_negative_and_over_100() {
    for text in [
        "color-mix(in srgb, red -1e-100%, blue)",
        "color-mix(in srgb, red 100.0000000000000000000001%, blue)",
    ] {
        let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
        assert_eq!(report.diagnostics().len(), 1, "{text}");
        assert_eq!(report.syntax().len(), 1, "{text}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
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
fn finite_number_hue_and_predefined_color_controls() {
    for declaration in both("hsl(0 25% 50%)") {
        let CssColorHue::Number(hue) = color(&declaration).hsl_value().unwrap().hue() else {
            panic!("number hue")
        };
        assert_eq!(hue.numeric().representation(), "0");
    }
    for declaration in both("color(srgb 0.5 0 1)") {
        let value = color(&declaration).predefined_value().unwrap();
        assert_eq!(value.color_space(), CssPredefinedColorSpace::Srgb);
        number(&value.channels()[0], "0.5");
    }
}
