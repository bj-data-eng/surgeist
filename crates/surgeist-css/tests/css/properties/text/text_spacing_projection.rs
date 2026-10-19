#![forbid(unsafe_code)]

//! CSS Text 4 §§8.1–8.2 authored percentage context:
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-word-spacing
//! https://www.w3.org/TR/2026/WD-css-text-4-20260814/#propdef-letter-spacing
//! The authored calculation must preserve its
//! percentage basis, even when percentage dimensions cancel algebraically.

use surgeist_css::*;

fn authored(name: &str, value: &str) -> CssTextSpacingAdjustment {
    let source = format!("{name}: {value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one spacing declaration")
    };
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::WordSpacing(wrapper) => wrapper.spacing().clone(),
        CssKnownPropertyValueRef::LetterSpacing(wrapper) => wrapper.value().clone(),
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
fn word_spacing_percentage_cancellation_retains_its_percentage_context() {
    let value = authored("word-spacing", "calc(10% / 10% * 1px)");
    assert_percentage_context(&value);
}

#[test]
fn letter_spacing_percentage_cancellation_retains_its_percentage_context() {
    let value = authored("letter-spacing", "calc(10% / 10% * 1px)");
    assert_percentage_context(&value);
}

#[test]
fn pure_length_math_and_additive_percentage_zero_keep_distinct_contexts() {
    for name in ["word-spacing", "letter-spacing"] {
        let pure = authored(name, "calc(1px - 2px)");
        assert_eq!(pure.serialize_specified().unwrap(), "calc(-1px)");
        assert_eq!(
            pure.length_percentage()
                .unwrap()
                .calculation()
                .unwrap()
                .numeric_type()
                .percent_hint(),
            None
        );

        let percentage = authored(name, "calc(1px + 0%)");
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
    }
}
