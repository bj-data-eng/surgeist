//! Shared checked color-component fixtures and color recovery assertions.

use surgeist_css::*;

/// `color: {value}` is dropped with one diagnostic while the following
/// declaration survives, and strict validation rejects the attribute.
pub fn assert_recovered(value: &str) {
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

/// A parsed component keeps the expected authored spelling and its parsed
/// source provenance.
pub fn assert_component(value: &CssColorComponent, expected: &str) {
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

/// The origin is parsed source containing `text`.
pub fn assert_origin(origin: &CssValueOrigin, text: &str) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("actual token provenance: {text}")
    };
    assert!(
        origin.source().as_str().contains(text),
        "{text}: {origin:?}"
    );
}

/// A programmatic `none`, number, percentage or `calc()` color component.
pub fn component(text: &str) -> CssColorComponent {
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

/// The single component value of `text`.
pub fn token(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    assert_eq!(values.items().len(), 1, "{text}");
    values.items()[0].clone()
}
