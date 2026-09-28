#![forbid(unsafe_code)]

//! CSS Text 4 §§8.1–8.2 authored percentage context:
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-word-spacing
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing
//! A pure-length I01 projection must preserve the current calculation's
//! percentage basis, even when percentage dimensions cancel algebraically.

use surgeist_css::*;

fn current(name: &str, value: &str) -> (CssTextSpacingAdjustment, bool) {
    let source = format!("{name}: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one spacing declaration")
    };
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::WordSpacing(wrapper) => {
            (wrapper.spacing().clone(), wrapper.i01_subset().is_some())
        }
        CssKnownPropertyValueRef::LetterSpacing(wrapper) => {
            (wrapper.current().clone(), wrapper.i01_subset().is_some())
        }
        _ => panic!("word or letter spacing"),
    }
}

fn assert_percentage_context(value: &CssTextSpacingAdjustment) {
    let calculation = value
        .length_percentage()
        .unwrap()
        .calculation()
        .expect("authored calc remains symbolic");
    assert_eq!(
        calculation.numeric_type().percent_hint(),
        Some(CssNumericDimension::Length)
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "calc(10% * 1px / 10%)"
    );
}

#[test]
fn word_spacing_percentage_cancellation_has_no_legacy_pure_length_projection() {
    let (value, has_legacy) = current("word-spacing", "calc(10% / 10% * 1px)");
    assert_percentage_context(&value);
    assert!(!has_legacy, "percentage basis must not disappear in I01");
}

#[test]
fn letter_spacing_percentage_cancellation_has_no_legacy_pure_length_projection() {
    let (value, has_legacy) = current("letter-spacing", "calc(10% / 10% * 1px)");
    assert_percentage_context(&value);
    assert!(!has_legacy, "percentage basis must not disappear in I01");
}

#[test]
fn pure_length_math_still_projects_and_additive_percentage_zero_does_not() {
    for name in ["word-spacing", "letter-spacing"] {
        let (pure, has_legacy) = current(name, "calc(1px - 2px)");
        assert_eq!(pure.serialize_specified().unwrap(), "calc(-1px)");
        assert!(
            has_legacy,
            "pure-length calc should retain its I01 projection"
        );

        let (percentage, has_legacy) = current(name, "calc(1px + 0%)");
        assert_eq!(
            percentage
                .length_percentage()
                .unwrap()
                .calculation()
                .unwrap()
                .numeric_type()
                .percent_hint(),
            Some(CssNumericDimension::Length)
        );
        assert!(!has_legacy, "authored zero percent still carries a basis");
    }
}
