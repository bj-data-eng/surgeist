#![forbid(unsafe_code)]
//! Selected Color4 CRD 2026-09-08 #the-hsl-notation and #funcdef-hwb:
//! modern saturation/lightness/whiteness/blackness admit numbers or percentages.
//! Existing public APIs only; no canonical or exact-scalar completion claim.
use surgeist_css::*;

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("color:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    let declaration = report.syntax()[0].clone();
    assert_eq!(declaration.importance(), CssImportance::Important);
    let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin() else {
        panic!("original function origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 6);
    declaration
}
fn wrapper(declaration: &CssDeclaration) -> &CssColorPropertyValue {
    let CssKnownPropertyValueRef::Color(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("ordinary color")
    };
    value
}
fn color(declaration: &CssDeclaration) -> &CssAuthoredColor {
    wrapper(declaration).current()
}
fn number(component: &CssAuthoredColorComponent, expected: f32) {
    assert!(
        matches!(component, CssAuthoredColorComponent::Number(value) if value.value() == expected)
    );
}
fn percentage(component: &CssAuthoredColorComponent, expected: f32) {
    assert!(
        matches!(component, CssAuthoredColorComponent::Percentage(value) if value.value() == expected)
    );
}
fn slots(color: &CssAuthoredColor) -> (&CssAuthoredColorComponent, &CssAuthoredColorComponent) {
    if let Some(hsl) = color.hsl_value() {
        assert_eq!(hsl.syntax(), CssAuthoredColorSyntax::Modern);
        (hsl.saturation(), hsl.lightness())
    } else {
        let hwb = color.hwb_value().expect("HWB");
        (hwb.whiteness(), hwb.blackness())
    }
}

#[test]
fn modern_number_channels_keep_number_payloads_and_source_origins() {
    for (text, first_expected, second_expected) in [
        ("hsl(0 25 50)", 25.0, 50.0),
        ("hsla(0 25 50 / .5)", 25.0, 50.0),
        ("hwb(0 25 50)", 25.0, 50.0),
        ("hsl(0 -25 150)", -25.0, 150.0),
        ("hwb(0 -25 150)", -25.0, 150.0),
    ] {
        let declaration = parsed(text);
        assert!(
            wrapper(&declaration).i01_subset().is_none(),
            "new number-channel form {text}"
        );
        let (first, second) = slots(color(&declaration));
        number(first, first_expected);
        number(second, second_expected);
    }
}

#[test]
fn checked_mixed_channels_keep_original_parsed_and_programmatic_components() {
    for function in ["hsl", "hsla", "hwb"] {
        for (arguments, first_is_number) in [("0 25 50%", true), ("0 25% 50", false)] {
            let text = format!("{function}({arguments})");
            for programmatic in [false, true] {
                let values = if programmatic {
                    CssComponentValues::try_new(vec![
                        CssComponentValue::try_function(
                            function,
                            parse_component_values(arguments).unwrap(),
                        )
                        .unwrap(),
                    ])
                    .unwrap()
                } else {
                    parse_component_values(&text).unwrap()
                };
                let declaration = parse_property_value(
                    CssPropertyNameRef::Known(CssKnownProperty::Color),
                    values.clone(),
                    CssImportance::Normal,
                )
                .expect("selected modern mixed component grammar");
                assert_eq!(declaration.value_components(), &values);
                assert!(declaration.position().is_none());
                assert!(declaration.parsed_value().is_none());
                assert_eq!(
                    matches!(
                        declaration.value_components().items()[0].origin(),
                        CssValueOrigin::Programmatic
                    ),
                    programmatic
                );
                assert!(wrapper(&declaration).i01_subset().is_none());
                let (first, second) = slots(color(&declaration));
                if first_is_number {
                    number(first, 25.0);
                    percentage(second, 50.0);
                } else {
                    percentage(first, 25.0);
                    number(second, 50.0);
                }
            }
        }
    }
}

fn literal(mut expression: CssCalculationExpressionRef<'_>, expected: &str) {
    loop {
        match expression {
            CssCalculationExpressionRef::NestedCalc(value)
            | CssCalculationExpressionRef::Group(value) => expression = value.operand(),
            CssCalculationExpressionRef::Value(value) => {
                assert_eq!(value.literal().representation(), expected);
                return;
            }
            _ => panic!("one independently expected literal math leaf"),
        }
    }
}
#[test]
fn modern_number_and_percentage_calculations_keep_distinct_types_and_leaves() {
    for text in [
        "hsl(0 calc(25) calc(50%))",
        "hsla(0 calc(25) calc(50%))",
        "hwb(0 calc(25) calc(50%))",
    ] {
        for declaration in [
            parsed(text),
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Color),
                parse_component_values(text).unwrap(),
                CssImportance::Normal,
            )
            .expect("checked modern math channels"),
        ] {
            let (first, second) = slots(color(&declaration));
            let CssAuthoredColorComponent::NumberCalculation(first) = first else {
                panic!("Number calculation")
            };
            let CssAuthoredColorComponent::PercentageCalculation(second) = second else {
                panic!("Percentage calculation")
            };
            assert_eq!(first.result_type(), CssCalculationType::Number);
            assert_eq!(second.result_type(), CssCalculationType::Percentage);
            literal(first.expression(), "25");
            literal(second.expression(), "50");
        }
    }
}

#[test]
fn border_color_aggregate_uses_the_same_modern_color_grammar() {
    let report = parse_style_attribute("border-color:hsl(0 25 50) hwb(0 10 20);opacity:.5");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 2);
    let CssKnownPropertyValueRef::BorderColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("border colors")
    };
    let values = value.current();
    number(values.top().hsl_value().unwrap().saturation(), 25.0);
    number(values.right().hwb_value().unwrap().whiteness(), 10.0);
    assert_eq!(values.top(), values.bottom());
    assert_eq!(values.right(), values.left());
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}

#[test]
fn legacy_hsl_stays_percentage_only_and_invalid_modern_domains_recover() {
    for text in [
        "hsl(0, 25, 50%)",
        "hsla(0, 25%, 50, .5)",
        "hsl(0, calc(25), 50%)",
        "hwb(0, 25%, 50%)",
        "hsl(0 25px 50)",
        "hwb(0 25 50deg)",
        "hsl(0 25)",
        "hwb(0 25 50 75)",
    ] {
        let report = parse_style_attribute(&format!("color:{text};opacity:.5"));
        assert!(!report.is_clean(), "{text}");
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
            .is_err(),
            "{text}"
        );
    }
}

#[test]
fn existing_legacy_percentages_and_modern_missing_components_stay_valid() {
    let declaration = parsed("hsla(0, 25%, 50%, .5)");
    assert!(
        wrapper(&declaration).i01_subset().is_some(),
        "existing legacy percentage projection"
    );
    let hsl = color(&declaration).hsl_value().unwrap();
    assert_eq!(hsl.syntax(), CssAuthoredColorSyntax::Legacy);
    percentage(hsl.saturation(), 25.0);
    percentage(hsl.lightness(), 50.0);
    for text in ["hsl(none none 50% / none)", "hwb(none none 50% / none)"] {
        let declaration = parsed(text);
        let (first, second) = slots(color(&declaration));
        assert!(matches!(first, CssAuthoredColorComponent::None));
        percentage(second, 50.0);
    }
}

// This exact fixture was incorrectly classified as invalid in color_grammars.rs.
#[test]
fn former_negative_modern_hsl_fixture_is_a_valid_mixed_domain_color() {
    let text = "hsl(20 30 40%)";
    for declaration in [
        parsed(text),
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            parse_component_values(text).unwrap(),
            CssImportance::Normal,
        )
        .expect("checked former negative fixture"),
    ] {
        let hsl = color(&declaration).hsl_value().unwrap();
        assert_eq!(hsl.syntax(), CssAuthoredColorSyntax::Modern);
        assert!(matches!(hsl.hue(), CssAuthoredHue::Number(value) if value.value() == 20.0));
        number(hsl.saturation(), 30.0);
        percentage(hsl.lightness(), 40.0);
        assert!(hsl.alpha().is_none());
        assert!(wrapper(&declaration).i01_subset().is_none());
    }
}
