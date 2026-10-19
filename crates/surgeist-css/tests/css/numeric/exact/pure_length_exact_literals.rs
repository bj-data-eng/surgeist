#![forbid(unsafe_code)]

//! Border spacing's ordinary numeric grammar is exact before contextual resolution.
//!
//! CSS2 §4.3.2 permits omission of a length unit only for zero, and §17.6.1
//! requires one or two nonnegative lengths:
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#length-units
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/tables.html#propdef-border-spacing
//! Values 4 §5 and §10 distinguish ordinary numeric range checks from deferred
//! math and define specified calculation serialization:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#lengths
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-range
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-serialize
//! These publication identities are pinned in `specs/catalog.json`. The public
//! exact-length contract is also exercised independently by `specified_lengths`.

use surgeist_css::{CssKnownPropertyValueRef, CssRecoveryAction, parse_style_attribute};

fn specified_spacing(authored: &str) -> String {
    let source = format!("border-spacing:{authored}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained border-spacing declaration: {source}");
    };
    let CssKnownPropertyValueRef::BorderSpacing(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed border-spacing: {source}");
    };
    value.spacing().serialize_specified().unwrap()
}

fn assert_spacing_rejected(authored: &str) {
    let source = format!("border-spacing:{authored}");
    let report = parse_style_attribute(&source);
    assert!(!report.is_clean(), "invalid length was accepted: {source}");
    assert!(
        report.syntax().is_empty(),
        "invalid border-spacing was retained: {source}"
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropDeclaration),
        "invalid border-spacing must be dropped: {source}"
    );
}

#[test]
fn huge_finite_decimal_length_is_accepted_and_serialized_exactly() {
    // Decimal 1e999 is exactly one followed by 999 zeroes. Authored CSS has
    // no f32 representability requirement; no used-value resolution occurs here.
    let expected_length = format!("1{}px", "0".repeat(999));
    assert_eq!(
        specified_spacing("1e999px"),
        format!("{expected_length} {expected_length}")
    );
}

#[test]
fn tiny_nonzero_unitless_number_is_rejected_instead_of_becoming_zero() {
    assert_spacing_rejected("1e-999");
}

#[test]
fn tiny_negative_length_is_rejected_by_the_nonnegative_property() {
    assert_spacing_rejected("-1e-999px");
}

#[test]
fn lexical_negative_zero_lengths_remain_valid_zero() {
    for (authored, expected) in [
        ("-0", "0 0"),
        ("-0.000e999", "0 0"),
        ("-0px", "0px 0px"),
        ("-0.000e-999px", "0px 0px"),
    ] {
        assert_eq!(specified_spacing(authored), expected, "{authored}");
    }
}

#[test]
fn only_true_unitless_zero_is_accepted_without_a_length_unit() {
    for authored in ["0", "+0.000e999", "0e-999"] {
        assert_eq!(specified_spacing(authored), "0 0", "{authored}");
    }
    assert_spacing_rejected("1");
}

#[test]
fn ordinary_pixel_lengths_serialize_in_horizontal_then_vertical_order() {
    assert_eq!(specified_spacing("+002.50PX"), "2.5px 2.5px");
    assert_eq!(specified_spacing("2.5px 3px"), "2.5px 3px");
}

#[test]
fn pure_length_math_keeps_its_deferred_range_and_percentage_cancellation() {
    // A nonnegative property's math range is enforced downstream, so this
    // authored calculation remains valid even though its result is negative.
    assert_eq!(
        specified_spacing("calc(1px - 2px)"),
        "calc(-1px) calc(-1px)"
    );
    // Percentages may appear in pure-length math when their dimensions cancel.
    let report = parse_style_attribute("border-spacing:calc(10% / 10% * 1px)");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    assert_spacing_rejected("calc(1px + 10%)");
}
