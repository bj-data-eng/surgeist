#![forbid(unsafe_code)]
//! CSS Sizing 4 §4.1: `auto || <ratio>`; CSS Values 4 §5.7: a ratio is
//! `<number [0,∞]> [ / <number [0,∞]> ]?` and serializes both components.
//! The expected spellings below come from those grammars, not parser output.

use surgeist_css::{
    CssAspectRatio, CssImportance, CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef,
    CssRecoveryAction, parse_component_values, parse_property_value, parse_sheet,
    parse_style_attribute,
};

fn parsed(value: &str) -> surgeist_css::CssDeclaration {
    let source = format!("aspect-ratio:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{value}");
    let declaration = report.syntax()[0].clone();
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected an ordinary aspect-ratio: {value}")
    };
    assert_eq!(authored.as_css(), value);
    declaration
}

fn checked(value: &str) -> surgeist_css::CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::AspectRatio),
        components,
        CssImportance::Normal,
    )
    .unwrap_or_else(|error| panic!("checked aspect-ratio {value}: {error:?}"));
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected checked aspect-ratio: {value}")
    };
    assert_eq!(authored.as_css(), value);
    declaration
}

fn assert_no_i01_projection(declaration: &surgeist_css::CssDeclaration, value: &str) {
    let CssKnownPropertyValueRef::AspectRatio(authored) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("expected aspect-ratio: {value}")
    };
    assert!(authored.i01_subset().is_none(), "{value}");
}

#[test]
fn preferred_ratio_accepts_auto_optional_pair_and_both_keyword_orders() {
    for value in [
        "auto",
        "16/9",
        "auto 16/9",
        "16/9 auto",
        "AUTO 16 / 9",
        "16 / 9 AUTO",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
    let sheet = parse_sheet(".x{aspect-ratio:auto 16/9}");
    assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
}

#[test]
fn preferred_ratio_accepts_zero_and_exact_extreme_number_components() {
    for value in [
        "0",
        "0/0",
        "0/9",
        "16/0",
        "-0/2",
        "1e999",
        "1e-999",
        "0.1",
        "16777217",
        "1e999 / 1e-999",
        "auto 1e999 / 1e-999",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn preferred_ratio_accepts_numeric_math_in_each_component() {
    for value in [
        "calc(8 * 2) / 9",
        "16 / calc(3 * 3)",
        "auto calc(8 * 2) / calc(3 * 3)",
        "min(16, 9) / max(3, 2) auto",
        "calc(-1)",
    ] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn only_a_positive_ordinary_number_has_the_frozen_i01_projection() {
    for value in ["1.5", "16"] {
        for declaration in [parsed(value), checked(value)] {
            let CssKnownPropertyValueRef::AspectRatio(authored) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("expected aspect-ratio: {value}")
            };
            let expected = CssAspectRatio::try_new(value.parse().unwrap()).unwrap();
            assert_eq!(authored.i01_subset(), Some(&expected));
        }
    }
    for value in ["1.5/1", "auto 1.5", "calc(1.5)"] {
        assert_no_i01_projection(&parsed(value), value);
        assert_no_i01_projection(&checked(value), value);
    }
}

#[test]
fn malformed_ratio_drops_only_its_declaration() {
    for value in [
        "-1e-999",
        "-1e-999 / 2",
        "2 / -1e-999",
        "auto auto",
        "auto 16/9 auto",
        "16/9/2",
        "/9",
        "16/",
        "auto / 9",
        "16 / auto",
        "calc(1px) / 2",
    ] {
        let source = format!("color:red;aspect-ratio:{value};width:1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.diagnostics().len(), 1, "{value}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{value}"
        );
        let kept: Vec<_> = report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property())
            .collect();
        assert_eq!(
            kept,
            [CssKnownProperty::Color, CssKnownProperty::Width],
            "{value}"
        );
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::AspectRatio),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        );
        assert!(checked.is_err(), "checked invalid ratio: {value}");
    }
}
