//! Values 4 WD 2024-03-12 §5: numeric precision and range are
//! implementation-defined, but a value that "cannot be explicitly supported"
//! converts to the closest supported value; an angle converts to the nearest
//! supported multiple of 360deg.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#numeric-types
//! §10.9.1 separately defines genuine infinities: the infinity constants,
//! division by zero, infinite operands and argument combinations defined to
//! produce them, such as pow(0, -1), log(0) and tan() at an asymptote.
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#calc-ieee
//! Surgeist's selected projection is binary64 (#735), so the closest finite
//! value of a non-angle type is `f64::MAX` with the result's sign. A
//! calculation needing no conversion or arithmetic serializes its exact
//! authored coefficient, as ordinary literals do (#1297).

use surgeist_css::*;

fn specified(declaration: &str) -> String {
    let report = parse_declaration(declaration);
    assert!(
        report.is_clean(),
        "{declaration}: {:?}",
        report.diagnostics()
    );
    report
        .syntax()
        .as_ref()
        .expect("retained declaration")
        .serialize_specified(Default::default())
        .unwrap()
}

fn assert_declaration(property: &str, value: &str, expected: &str) {
    assert_eq!(
        specified(&format!("{property}:{value}")),
        format!("{property}: {expected};"),
        "{property}: {value}"
    );
}

/// Every digit of the integral binary64 value, as six-place output emits it.
fn integral(value: f64) -> String {
    format!("{value:.0}")
}

fn max() -> String {
    integral(f64::MAX)
}

fn angle_endpoint() -> String {
    // (2^53 - 1) mod 45 == 31: the largest finite binary64 multiple of 360.
    integral(f64::from_bits(f64::MAX.to_bits() - 31))
}

fn exact_power_of_ten(exponent: usize) -> String {
    format!("1{}", "0".repeat(exponent))
}

#[test]
fn finite_unit_conversion_overflow_converts_to_the_closest_finite_value() {
    // 1e308in is 9.6e309px: finite, above the binary64 range.
    assert_declaration("width", "calc(1e308in)", &format!("calc({}px)", max()));
    assert_declaration("width", "calc(-1e308in)", &format!("calc(-{}px)", max()));
    // A finite authored Number coefficient above the binary64 range.
    assert_declaration("opacity", "calc(1e999 * 2)", &format!("calc({})", max()));
}

#[test]
fn finite_compound_arithmetic_overflow_converts_to_the_closest_finite_value() {
    for (property, value, expected) in [
        ("width", "calc(1e308px * 10)", format!("calc({}px)", max())),
        (
            "width",
            "calc(-1e308px * 10)",
            format!("calc(-{}px)", max()),
        ),
        (
            "width",
            "calc(1e308px + 1e308px)",
            format!("calc({}px)", max()),
        ),
        ("width", "calc(1e308% * 10)", format!("calc({}%)", max())),
        (
            "width",
            "calc(1e300 * 1e300 * 1px)",
            format!("calc({}px)", max()),
        ),
        (
            "transition-duration",
            "calc(1e308s * 10)",
            format!("calc({}s)", max()),
        ),
        ("opacity", "calc(1e308 * 10)", format!("calc({})", max())),
        // 1 / 1e-320 is the finite 1e320, not a division by zero.
        ("opacity", "calc(1 / 1e-320)", format!("calc({})", max())),
        ("opacity", "exp(1000)", format!("calc({})", max())),
        ("opacity", "pow(10, 400)", format!("calc({})", max())),
        (
            "width",
            "hypot(1.5e308px, 1.5e308px)",
            format!("calc({}px)", max()),
        ),
    ] {
        assert_declaration(property, value, &expected);
    }
}

#[test]
fn finite_angle_overflow_keeps_the_nearest_supported_multiple_of_360deg() {
    for value in [
        "rotate(calc(1e308turn))",
        "rotate(calc(1e308deg + 1e308deg))",
        "rotate(calc(1e308deg * 10))",
    ] {
        assert_declaration(
            "transform",
            value,
            &format!("rotate(calc({}deg))", angle_endpoint()),
        );
    }
}

#[test]
fn coefficients_beyond_binary64_convert_when_the_canonical_value_fits() {
    // 1e310ms is the finite 1e307s; its coefficient alone exceeds binary64.
    let output = specified("transition-duration:calc(1e310ms)");
    let coefficient = output
        .strip_prefix("transition-duration: calc(")
        .and_then(|rest| rest.strip_suffix("s);"))
        .unwrap_or_else(|| panic!("{output}"));
    let value: f64 = coefficient.parse().unwrap();
    assert!(((value - 1e307) / 1e307).abs() < 1e-12, "{output}");
}

#[test]
fn a_single_unconverted_leaf_serializes_its_exact_authored_coefficient() {
    for (property, value, expected) in [
        (
            "width",
            "calc(1e999px)",
            format!("calc({}px)", exact_power_of_ten(999)),
        ),
        (
            "width",
            "calc((1e999px))",
            format!("calc({}px)", exact_power_of_ten(999)),
        ),
        (
            "width",
            "calc(-1e999px)",
            format!("calc(-{}px)", exact_power_of_ten(999)),
        ),
        (
            "width",
            "calc(1e999em)",
            format!("calc({}em)", exact_power_of_ten(999)),
        ),
        (
            "width",
            "calc(1e999%)",
            format!("calc({}%)", exact_power_of_ten(999)),
        ),
        (
            "transition-duration",
            "calc(1e999s)",
            format!("calc({}s)", exact_power_of_ten(999)),
        ),
        (
            "opacity",
            "calc(1e999)",
            format!("calc({})", exact_power_of_ten(999)),
        ),
        // The exact decimal tie rounds away from zero, like the literal.
        ("width", "calc(5e-7px)", "calc(0.000001px)".into()),
        (
            "width",
            "calc(12345678901234567890123px)",
            "calc(12345678901234567890123px)".into(),
        ),
    ] {
        assert_declaration(property, value, &expected);
    }
    // The ordinary literal path produces the same coefficient text.
    assert_declaration(
        "width",
        "1e999px",
        &format!("{}px", exact_power_of_ten(999)),
    );
    assert_declaration("width", "5e-7px", "0.000001px");
}

#[test]
fn conversion_or_arithmetic_keeps_the_binary64_projection() {
    for (property, value, expected) in [
        ("width", "calc(1e999px + 0px)", format!("calc({}px)", max())),
        ("width", "calc(1e999px * 1)", format!("calc({}px)", max())),
        ("width", "calc(5e-7px * 1)", "calc(0px)".into()),
        ("transition-duration", "calc(1000ms)", "calc(1s)".into()),
    ] {
        assert_declaration(property, value, &expected);
    }
}

#[test]
fn genuine_infinities_stay_infinite() {
    for (property, value, expected) in [
        ("width", "calc(infinity * 1px)", "calc(infinity * 1px)"),
        ("width", "calc(-infinity * 1px)", "calc(-infinity * 1px)"),
        ("width", "calc(1px / 0)", "calc(infinity * 1px)"),
        ("width", "calc(-1px / 0)", "calc(-infinity * 1px)"),
        ("width", "calc(1e308px * infinity)", "calc(infinity * 1px)"),
        (
            "width",
            "calc(1e308px + infinity * 1px)",
            "calc(infinity * 1px)",
        ),
        (
            "transition-duration",
            "calc(infinity * 1s)",
            "calc(infinity * 1s)",
        ),
        ("opacity", "calc(infinity)", "calc(infinity)"),
        ("opacity", "calc(1 / 0)", "calc(infinity)"),
        ("opacity", "pow(0, -1)", "calc(infinity)"),
        ("opacity", "log(0)", "calc(-infinity)"),
        ("opacity", "tan(90deg)", "calc(infinity)"),
        ("opacity", "exp(infinity)", "calc(infinity)"),
        (
            "transform",
            "rotate(calc(infinity * 1deg))",
            "rotate(calc(infinity * 1deg))",
        ),
        (
            "transform",
            "rotate(calc(1deg / 0))",
            "rotate(calc(infinity * 1deg))",
        ),
    ] {
        assert_declaration(property, value, expected);
    }
}

#[test]
fn every_accumulation_step_converts_finite_overflow_before_the_next_operand() {
    // Each finite intermediate value is a value the implementation must
    // support, so it converts before meeting the next operand: MAX * 0 is 0,
    // and MAX - infinity is the genuine -infinity rather than NaN (§10.9.1).
    // Grouping therefore cannot change the value class.
    for (property, value, expected) in [
        ("opacity", "calc(1e308 * 2 * 0)", "calc(0)".to_owned()),
        ("opacity", "calc((1e308 * 2) * 0)", "calc(0)".to_owned()),
        (
            "width",
            "calc(1e308 * 1e308 * 0 * 1px)",
            "calc(0px)".to_owned(),
        ),
        (
            "width",
            "calc(1e308px + 1e308px - infinity * 1px)",
            "calc(-infinity * 1px)".to_owned(),
        ),
        (
            "opacity",
            "calc(1e308 + 1e308 - 1e308)",
            format!("calc({})", integral(f64::MAX - 1e308)),
        ),
    ] {
        assert_declaration(property, value, &expected);
    }
}

#[test]
fn single_argument_comparisons_simplify_to_the_exact_leaf() {
    // Values 4 simplification replaces min()/max() of one argument with that
    // argument, so no arithmetic remains and the authored coefficient is kept.
    let exact = format!("calc({}px)", exact_power_of_ten(999));
    for value in ["min(1e999px)", "max(1e999px)", "calc(min(1e999px))"] {
        assert_declaration("width", value, &exact);
    }
    assert_declaration("width", "min(1e999px, 1px)", "calc(1px)");
}
