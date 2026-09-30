#![forbid(unsafe_code)]
//! Exact authored numeric admission through the existing public parser.
//!
//! Exact decimal retention is the adopted Surgeist authored-value contract.
//! CSS Values 4 permits implementation-defined supported precision/range;
//! this contract does not claim a universal CSS ban on finite numeric domains.
//! Cubic-bezier ordinary x coordinates have the inclusive [0,1] grammar,
//! while range checking inside genuine math functions remains downstream.
//!
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! https://www.w3.org/TR/2023/CRD-css-easing-1-20230213/#cubic-bezier-easing-functions

use surgeist_css::{CssKnownProperty, CssRecoveryAction, parse_style_attribute};

fn assert_clean_typed_property(source: &str, expected: CssKnownProperty) {
    let report = parse_style_attribute(source);
    assert!(
        report.is_clean(),
        "{source}: expected clean authored admission, got {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().len(), 1, "{source}");
    let declaration = report.syntax()[0].known().expect("known declaration");
    assert_eq!(declaration.property(), expected, "{source}");
    assert!(
        declaration.property_value().is_some(),
        "{source}: expected an ordinary typed property value"
    );
}

fn assert_rejected_property_preserves_neighbor(declaration: &str) {
    let source = format!("{declaration}; color: red");
    let report = parse_style_attribute(&source);
    assert!(
        !report.is_clean(),
        "{source}: expected ordinary grammar rejection"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "{source}: expected exactly one diagnostic, got {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropDeclaration,
        "{source}"
    );
    assert_eq!(report.syntax().len(), 1, "{source}");
    let neighbor = report.syntax()[0]
        .known()
        .expect("retained color declaration");
    assert_eq!(neighbor.property(), CssKnownProperty::Color, "{source}");
    assert!(neighbor.property_value().is_some(), "{source}");
}

#[test]
fn matrix_admits_an_exact_ordinary_decimal_beyond_float_range() {
    assert_clean_typed_property(
        "transform: matrix(1, 0, 0, 1, 1e999, -1e999)",
        CssKnownProperty::Transform,
    );
}

#[test]
fn matrix3d_admits_exact_ordinary_decimals_in_its_sixteen_operands() {
    assert_clean_typed_property(
        "transform: matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1e999, -1e999, 0, 1)",
        CssKnownProperty::Transform,
    );
}

#[test]
fn scale_x_admits_signed_exact_ordinary_decimals_beyond_float_range() {
    for number in ["1e999", "-1e999"] {
        assert_clean_typed_property(
            &format!("transform: scaleX({number})"),
            CssKnownProperty::Transform,
        );
    }
}

#[test]
fn rotate3d_admits_exact_ordinary_decimals_in_each_axis_operand() {
    for value in [
        "rotate3d(1e999, 0, 0, 0deg)",
        "rotate3d(0, -1e999, 0, 0deg)",
        "rotate3d(0, 0, 1e999, 0deg)",
    ] {
        assert_clean_typed_property(&format!("transform: {value}"), CssKnownProperty::Transform);
    }
}

#[test]
fn independent_scale_admits_signed_exact_ordinary_decimals() {
    for value in ["1e999", "-1e999", "1e999 -1e999 2"] {
        assert_clean_typed_property(&format!("scale: {value}"), CssKnownProperty::Scale);
    }
}

#[test]
fn cubic_bezier_admits_unrestricted_exact_ordinary_y_coordinates() {
    for value in [
        "cubic-bezier(0, 1e999, 1, -1e999)",
        "cubic-bezier(0, -1e999, 1, 1e999)",
    ] {
        assert_clean_typed_property(
            &format!("transition-timing-function: {value}"),
            CssKnownProperty::TransitionTimingFunction,
        );
    }
}

#[test]
fn cubic_bezier_rejects_tiny_negative_ordinary_x_and_preserves_color() {
    for value in [
        "cubic-bezier(-1e-999, 0, 1, 1)",
        "cubic-bezier(0, 0, -1e-999, 1)",
    ] {
        assert_rejected_property_preserves_neighbor(&format!(
            "transition-timing-function: {value}"
        ));
    }
}

#[test]
fn cubic_bezier_rejects_exact_ordinary_x_above_one_and_preserves_color() {
    for value in [
        "cubic-bezier(1.0000000000000000001, 0, 1, 1)",
        "cubic-bezier(0, 0, 1.0000000000000000001, 1)",
    ] {
        assert_rejected_property_preserves_neighbor(&format!(
            "transition-timing-function: {value}"
        ));
    }
}

#[test]
fn scale_z_admits_signed_exact_ordinary_percentages_beyond_float_range() {
    for percentage in ["1e999%", "-1e999%"] {
        assert_clean_typed_property(
            &format!("transform: scaleZ({percentage})"),
            CssKnownProperty::Transform,
        );
    }
}

#[test]
fn scale3d_admits_signed_exact_percentages_in_each_operand() {
    for value in [
        "scale3d(1e999%, 2, 3)",
        "scale3d(1, -1e999%, 3)",
        "scale3d(1, 2, -1e999%)",
    ] {
        assert_clean_typed_property(&format!("transform: {value}"), CssKnownProperty::Transform);
    }
}

#[test]
fn cubic_bezier_admits_genuine_signed_zero_with_enormous_exponents() {
    // A zero coefficient denotes zero even when its exponent cannot fit i128.
    for zero in [
        "-0e9999999999999999999999999999999999999999",
        "+0e9999999999999999999999999999999999999999",
        "-0e-9999999999999999999999999999999999999999",
    ] {
        assert_clean_typed_property(
            &format!("transition-timing-function: cubic-bezier({zero}, 0, {zero}, 1)"),
            CssKnownProperty::TransitionTimingFunction,
        );
    }
}

#[test]
fn ordinary_number_and_percentage_controls_preserve_current_typed_admission() {
    for value in [
        "matrix(1, 0, 0, 1, -12.5, +8)",
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 12, -8, 0, 1)",
        "scale(1.5, -2) scaleX(.5) scaleY(-2)",
        "rotate3d(1, 0, -1, 45deg)",
        "scale3d(1, -25%, 2) scaleZ(-25%)",
    ] {
        assert_clean_typed_property(&format!("transform: {value}"), CssKnownProperty::Transform);
    }
    for value in ["1", "1.5 -2", "1 2 -3", "none"] {
        assert_clean_typed_property(&format!("scale: {value}"), CssKnownProperty::Scale);
    }
    assert_clean_typed_property(
        "transition-timing-function: cubic-bezier(0, -20, 1, 30)",
        CssKnownProperty::TransitionTimingFunction,
    );
}

#[test]
fn number_contexts_retain_exact_arity_separator_and_dimension_rejection() {
    for value in [
        "matrix(1 0 0 1 10 20)",
        "matrix(1, 0, 0, 1, 10)",
        "matrix(1, 0, 0, 1, 10, 20, 30)",
        "matrix(1%, 0, 0, 1, 10, 20)",
        "matrix3d(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 10, 20, 30)",
        "scaleX(1, 2)",
        "scaleX(1px)",
        "scale(1 2)",
        "rotate3d(1 0 0 45deg)",
        "rotate3d(1, 0, 45deg)",
        "rotate3d(1px, 0, 0, 45deg)",
        "scale3d(1, 2)",
        "scale3d(1 2 3)",
        "scaleZ(1px)",
    ] {
        assert_rejected_property_preserves_neighbor(&format!("transform: {value}"));
    }
    for value in [
        "cubic-bezier(0 0 1 1)",
        "cubic-bezier(0, 0, 1)",
        "cubic-bezier(0, 0, 1, 1, 2)",
        "cubic-bezier(0%, 0, 1, 1)",
        "cubic-bezier(0, 1px, 1, 1)",
        "cubic-bezier(-.1, 0, 1, 1)",
        "cubic-bezier(0, 0, 1.1, 1)",
    ] {
        assert_rejected_property_preserves_neighbor(&format!(
            "transition-timing-function: {value}"
        ));
    }
    assert_rejected_property_preserves_neighbor("scale: 1 2 3 4");
}

#[test]
fn cubic_bezier_defers_range_checks_inside_genuine_number_math() {
    for value in [
        "cubic-bezier(calc(0 - 1), 0, 1, 1)",
        "cubic-bezier(0, 0, calc(1 + 1), 1)",
        "cubic-bezier(calc(2), 0, calc(-1), 1)",
    ] {
        assert_clean_typed_property(
            &format!("transition-timing-function: {value}"),
            CssKnownProperty::TransitionTimingFunction,
        );
    }
}

#[test]
fn independent_scale_preserves_its_current_literal_only_admission_subset() {
    // Characterization of current incomplete support, not a normative CSS ban.
    // This corrective unit does not add independent-scale math or percentages.
    for value in ["calc(1)", "1 calc(2)", "50%", "1 50%"] {
        assert_rejected_property_preserves_neighbor(&format!("scale: {value}"));
    }
}
