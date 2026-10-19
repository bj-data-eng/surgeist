#![forbid(unsafe_code)]
//! Filter Effects 1 section 6.1 defines an omitted blur radius as 0px.
//! https://www.w3.org/TR/2018/WD-filter-effects-1-20181218/#funcdef-filter-blur
use surgeist_css::*;

#[test]
fn omitted_blur_radius_retains_authored_input_and_specified_zero() {
    for source in [
        "filter:blur()",
        "filter:blur(/**/)",
        "backdrop-filter:blur()",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let declaration = &report.syntax()[0];
        let value = declaration.known().unwrap().property_value().unwrap();
        let current = match value {
            CssKnownPropertyValueRef::Filter(value) => value.value(),
            CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
            _ => panic!("filter property"),
        };
        let CssFilter::Functions(functions) = current else {
            panic!("filter functions");
        };
        let [CssFilterFunction::Blur(blur)] = functions.functions() else {
            panic!("one blur function");
        };
        assert!(exact_dimension(
            blur.length().literal_component(),
            "0",
            "px"
        ));
        assert!(blur.authored_length().is_none());
        let span = declaration.parsed_value().unwrap().span();
        assert_eq!(
            &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            source.split_once(':').unwrap().1
        );
    }
}

#[test]
fn omitted_blur_radius_survives_implicit_function_closure() {
    let report = parse_style_attribute("filter:blur(");
    assert_eq!(report.syntax().len(), 1, "{report:?}");
    assert!(!report.is_clean());
    assert!(
        report.diagnostics().iter().all(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
}

#[test]
fn explicit_blur_radii_keep_their_values_and_authored_spelling() {
    for (argument, expected) in [("0", "0"), ("0px", "0px"), ("2px", "2px")] {
        let report = parse_style_attribute(&format!("filter:blur({argument})"));
        assert!(report.is_clean(), "{report:?}");
        let CssKnownPropertyValueRef::Filter(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("filter")
        };
        let CssFilter::Functions(functions) = value.value() else {
            panic!("functions")
        };
        let [CssFilterFunction::Blur(blur)] = functions.functions() else {
            panic!("blur")
        };
        assert!(exact_literal(blur.length().literal_component(), expected));
        assert!(blur.authored_length().is_some());
        assert_eq!(value.as_css(), format!("blur({argument})"));
    }
    for argument in ["-1px", "1%", ",", "1px 2px"] {
        let report = parse_style_attribute(&format!("filter:blur({argument})"));
        assert!(report.syntax().is_empty(), "{argument}: {report:?}");
        assert!(!report.is_clean());
    }
}

fn exact_dimension(
    component: Option<&surgeist_css::CssComponentValue>,
    expected: &str,
    expected_unit: &str,
) -> bool {
    matches!(component.map(surgeist_css::CssComponentValue::view),Some(surgeist_css::CssComponentValueRef::Token(surgeist_css::CssValueTokenRef::Dimension {number,unit})) if number.representation()==expected && unit==expected_unit)
}

fn exact_literal(component: Option<&surgeist_css::CssComponentValue>, css: &str) -> bool {
    use surgeist_css::{CssComponentValueRef as Component, CssValueTokenRef as Token};
    let expected = surgeist_css::CssComponentValue::try_token(css).unwrap();
    match (
        component.map(surgeist_css::CssComponentValue::view),
        expected.view(),
    ) {
        (
            Some(Component::Token(Token::Number(actual))),
            Component::Token(Token::Number(expected)),
        )
        | (
            Some(Component::Token(Token::Percentage(actual))),
            Component::Token(Token::Percentage(expected)),
        ) => actual.representation() == expected.representation(),
        (
            Some(Component::Token(Token::Dimension {
                number: actual,
                unit: actual_unit,
            })),
            Component::Token(Token::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => actual.representation() == expected.representation() && actual_unit == expected_unit,
        _ => false,
    }
}
