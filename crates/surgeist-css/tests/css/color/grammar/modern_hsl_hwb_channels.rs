//! Selected Color4 CRD 2026-09-08 #the-hsl-notation and #funcdef-hwb:
//! modern saturation/lightness/whiteness/blackness admit numbers or percentages.
//! Existing public APIs only; no canonical or exact-scalar completion claim.
use crate::css::support::color_and_selector_fixtures::{
    assert_color_value_rejected, parsed_important_declaration,
};
use crate::css::support::declaration_color as color;
use surgeist_css::CssParser;
use surgeist_css::color::{CssColor, CssColorComponent, CssColorHue, CssColorSyntax};
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::numeric::{CssCalculationExpressionRef, CssCalculationType};
use surgeist_css::property::{
    CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef, CssPropertyTarget,
};
use surgeist_css::values::common::CssBoxSideKind;

fn number(component: &CssColorComponent, expected: f32) {
    assert!(
        matches!(component, CssColorComponent::Number(value) if value.numeric().representation() == format!("{expected}"))
    );
}
fn percentage(component: &CssColorComponent, expected: f32) {
    assert!(
        matches!(component, CssColorComponent::Percentage(value) if value.numeric().representation() == format!("{expected}"))
    );
}
fn slots(color: &CssColor) -> (&CssColorComponent, &CssColorComponent) {
    if let Some(hsl) = color.hsl_value() {
        assert_eq!(hsl.syntax(), CssColorSyntax::Modern);
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
        let declaration = parsed_important_declaration("color", text);
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
                    CssComponentValues::try_new(
                        vec![
                            CssComponentValue::try_function(
                                function,
                                CssComponentValues::parse(&CssParser::default(), arguments)
                                    .unwrap(),
                            )
                            .unwrap(),
                        ],
                        Default::default(),
                    )
                    .unwrap()
                } else {
                    CssComponentValues::parse(&CssParser::default(), &text).unwrap()
                };
                let declaration = CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
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
            parsed_important_declaration("color", text),
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                CssImportance::Normal,
            )
            .expect("checked modern math channels"),
        ] {
            let (first, second) = slots(color(&declaration));
            let CssColorComponent::NumberCalculation(first) = first else {
                panic!("Number calculation")
            };
            let CssColorComponent::PercentageCalculation(second) = second else {
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
    let report = CssParser::default()
        .parse_block_contents("border-color:hsl(0 25 50) hwb(0 10 20);opacity:.5");
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
    let values = value.value();
    assert_eq!(values.kind(), CssBoxSideKind::Physical);
    assert_eq!(values.authored().len(), 2);
    let [top, right, bottom, left] = values.assigned();
    number(top.hsl_value().unwrap().saturation(), 25.0);
    number(right.hwb_value().unwrap().whiteness(), 10.0);
    assert_eq!(top, bottom);
    assert_eq!(right, left);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
}

#[test]
fn legacy_hsl_stays_percentage_only_and_invalid_modern_domains_recover() {
    // Color 4 §7.1: legacy HSL saturation and lightness are percentages and
    // HWB has no legacy comma form; modern HSL/HWB channels are <number> or
    // <percentage> and take exactly three. A truncated argument list and a
    // calculation name no single responsible token, so those rows check only
    // the color grammar kind.
    for (text, responsible) in [
        ("hsl(0, 25, 50%)", Some("25")),
        ("hsla(0, 25%, 50, .5)", Some("50")),
        ("hsl(0, calc(25), 50%)", None),
        ("hwb(0, 25%, 50%)", Some(",")),
        ("hsl(0 25px 50)", Some("25px")),
        ("hwb(0 25 50deg)", Some("50deg")),
        ("hsl(0 25)", None),
        ("hwb(0 25 50 75)", Some("75")),
    ] {
        let report = CssParser::default().parse_block_contents(format!("color:{text};opacity:.5"));
        let [diagnostic] = report.diagnostics() else {
            panic!("{text}: one diagnostic: {report:?}")
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidColorSyntax,
            "{text}"
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 1, "{text}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Opacity
        );
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            CssImportance::Normal,
        );
        match responsible {
            Some(token) => {
                assert_color_value_rejected(checked, CssKnownProperty::Color, text, token)
            }
            None => assert!(
                checked.unwrap_err().code() == Some(CssErrorCode::InvalidColorSyntax),
                "{text}"
            ),
        }
    }
}

#[test]
fn legacy_percentages_and_modern_missing_components_stay_valid() {
    let declaration = parsed_important_declaration("color", "hsla(0, 25%, 50%, .5)");
    let hsl = color(&declaration).hsl_value().unwrap();
    assert_eq!(hsl.syntax(), CssColorSyntax::Legacy);
    percentage(hsl.saturation(), 25.0);
    percentage(hsl.lightness(), 50.0);
    for text in ["hsl(none none 50% / none)", "hwb(none none 50% / none)"] {
        let declaration = parsed_important_declaration("color", text);
        let (first, second) = slots(color(&declaration));
        assert!(matches!(first, CssColorComponent::None));
        percentage(second, 50.0);
    }
}

// This exact fixture was incorrectly classified as invalid in color_grammars.rs.
#[test]
fn former_negative_modern_hsl_fixture_is_a_valid_mixed_domain_color() {
    let text = "hsl(20 30 40%)";
    for declaration in [
        parsed_important_declaration("color", text),
        CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            CssImportance::Normal,
        )
        .expect("checked former negative fixture"),
    ] {
        let hsl = color(&declaration).hsl_value().unwrap();
        assert_eq!(hsl.syntax(), CssColorSyntax::Modern);
        assert!(
            matches!(hsl.hue(), CssColorHue::Number(value) if value.numeric().representation() == "20")
        );
        number(hsl.saturation(), 30.0);
        percentage(hsl.lightness(), 40.0);
        assert!(hsl.alpha().is_none());
    }
}
