#![forbid(unsafe_code)]

//! Authored grammar expectations from the pinned Values 4 sections 10.1–10.9.
//! Numeric execution, integer rounding and range clamping belong to resolution.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#math

use surgeist_css::parse_style_attribute;

fn accepted(declaration: &str) {
    let report = parse_style_attribute(declaration);
    assert!(
        report.is_clean(),
        "{declaration}: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.syntax().len(), 1, "{declaration}");
}

fn rejected(declaration: &str) {
    let report = parse_style_attribute(declaration);
    assert!(
        !report.is_clean(),
        "{declaration} must not match its grammar"
    );
    assert!(report.syntax().is_empty(), "{declaration}");
}

#[test]
fn intermediate_dimension_exponents_can_cancel_within_a_calculation() {
    for declaration in [
        "width: calc(1px * 2px / 1px)",
        "opacity: calc(1s / 1ms)",
        "width: calc((1px * 2s) / 1s)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn authored_numeric_spelling_is_not_limited_by_float_or_integer_storage() {
    for declaration in [
        "opacity: calc(1e99999)",
        "z-index: calc(2147483648)",
        "width: calc(12345678901234567890123456789012345678901234567890px)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn special_values_and_arithmetic_domains_are_not_parse_errors() {
    for declaration in [
        "width: calc(1px / 0)",
        "width: calc(infinity * 1px)",
        "opacity: calc(-infinity)",
        "opacity: calc(NaN)",
        "opacity: sqrt(-1)",
        "opacity: log(0)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn integer_positions_accept_number_valued_math_for_later_rounding() {
    for declaration in ["z-index: calc(1.5)", "z-index: calc(3 / 2)"] {
        accepted(declaration);
    }
    rejected("z-index: 1.5");
}

#[test]
fn comparison_functions_share_length_percentage_admission() {
    for declaration in [
        "width: min(1px, 2em)",
        "width: max(10%, 2px)",
        "width: clamp(1px, 10%, 100px)",
        "width: clamp(none, 10%, none)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn stepped_functions_preserve_their_authored_argument_forms() {
    for declaration in [
        "width: round(nearest, 1.2px, 1px)",
        "width: round(up, 1.2px, 1px)",
        "width: round(down, 1.2px, 1px)",
        "width: round(to-zero, 1.2px, 1px)",
        "opacity: round(1.25)",
        "width: mod(5px, 2px)",
        "width: rem(-5px, 2px)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn trigonometric_functions_have_number_or_angle_results() {
    for declaration in [
        "opacity: sin(45deg)",
        "opacity: cos(0)",
        "opacity: tan(0rad)",
        "transform: rotate(asin(0.5))",
        "transform: rotate(acos(0.5))",
        "transform: rotate(atan(1))",
        "transform: rotate(atan2(1px, 2px))",
    ] {
        accepted(declaration);
    }
}

#[test]
fn exponential_functions_and_hypot_keep_their_intrinsic_domains() {
    for declaration in [
        "opacity: pow(2, 3)",
        "opacity: sqrt(2)",
        "opacity: exp(1)",
        "opacity: log(8, 2)",
        "width: hypot(3px, 4%)",
    ] {
        accepted(declaration);
    }
}

#[test]
fn sign_related_functions_use_their_defined_output_types() {
    accepted("width: abs(-10px)");
    accepted("opacity: sign(-1px)");
}

#[test]
fn binary_signs_require_real_whitespace_and_unary_delimiters_are_not_values() {
    rejected("width: calc(1px/**/+/**/2px)");
    rejected("width: calc(-(1px))");
    accepted("width: calc(1px /* left */ + /* right */ 2px)");
}

#[test]
fn each_function_boundary_must_have_a_valid_numeric_type() {
    accepted("width: calc((1px * 2px) / 1px)");
    rejected("width: calc(calc(1px * 2px) / 1px)");
    rejected("width: calc(0 * 5px + 10s)");
    rejected("width: calc(0 + 5px)");
    rejected("opacity: calc(0.25 + 25%)");
}

#[test]
fn existing_literal_and_basic_calculation_forms_remain_accepted() {
    for declaration in [
        "width: 10px",
        "width: calc(1px + 2px)",
        "width: calc(10% + 2px)",
        "z-index: 1",
        "opacity: calc(100% / 3)",
    ] {
        accepted(declaration);
    }
}
