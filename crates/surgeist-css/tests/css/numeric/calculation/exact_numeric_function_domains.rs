//! Domain and arity expectations from CSS Values 4 §10. Type checking does not
//! evaluate the real-number result or reject deferred arithmetic singularities.
use crate::css::support::value_assertions::assert_calculation_rejected;
use surgeist_css::{
    CssAngleCalculation, CssLengthCalculation, CssLengthPercentageCalculation,
    CssNumberCalculation, CssPercentageCalculation, parse_component_values,
};

#[test]
fn numeric_functions_enforce_argument_domains_and_arity() {
    for source in [
        "calc(1)",
        "min(1, 2, 3)",
        "max(1)",
        "clamp(none, 2, none)",
        "round(2.5)",
        "round(up, 2.5, 1)",
        "mod(1, 0)",
        "rem(1, 0)",
        "sin(90deg)",
        "cos(1)",
        "tan(0turn)",
        "pow(2, 3)",
        "sqrt(-1)",
        "hypot(3, 4)",
        "log(0)",
        "log(8, 2)",
        "exp(1)",
        "abs(-1)",
        "sign(-1px)",
        "calc(1px / 1px)",
    ] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_ok(),
            "{source}"
        );
    }
    for source in [
        "calc()",
        "calc(1, 2)",
        "min()",
        "max()",
        "clamp(1, 2)",
        "clamp(1, none, 3)",
        "round()",
        "round(sideways, 1, 2)",
        "mod(1)",
        "rem(1, 2, 3)",
        "sin(1px)",
        "cos(1s)",
        "tan(1%)",
        "pow(1px, 2)",
        "sqrt(1px)",
        "log(1px)",
        "exp(1px)",
        "abs(1, 2)",
        "sign()",
        "hypot(1, 1px)",
        "log(1, 2, 3)",
        "calc(1 + 1%)",
        "calc(min(1px * 1px) / 1px / 1px)",
    ] {
        // Values 4 §10: a wrong argument count is an arity failure, an
        // argument outside the function's accepted types an argument-type
        // failure, and arguments whose types cannot be made consistent are
        // incompatible. The remaining cases admit more than one reading.
        use surgeist_css::CssNumericConstructionErrorKind as N;
        let kind = match source {
            "min()" | "max()" | "clamp(1, 2)" | "round()" | "mod(1)" | "rem(1, 2, 3)"
            | "abs(1, 2)" | "sign()" | "log(1, 2, 3)" => Some(N::Arity),
            "sin(1px)" | "cos(1s)" | "tan(1%)" | "pow(1px, 2)" | "sqrt(1px)" | "log(1px)"
            | "exp(1px)" => Some(N::InvalidArgumentType),
            "hypot(1, 1px)" => Some(N::IncompatibleTypes),
            _ => None,
        };
        assert_calculation_rejected(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()),
            kind,
        );
    }
}

#[test]
fn inverse_trigonometry_returns_angles_and_checks_argument_types() {
    for source in [
        "asin(2)",
        "acos(-2)",
        "atan(1)",
        "atan2(1px, 2em)",
        "atan2(1, 2)",
    ] {
        assert!(
            CssAngleCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_ok(),
            "{source}"
        );
    }
    for source in [
        "asin(1deg)",
        "acos(1%)",
        "atan(1px)",
        "atan2(1px, 1s)",
        "atan2(1)",
    ] {
        // Values 4 §10.5: asin/acos/atan take a <number>; atan2 takes two
        // consistent arguments.
        use surgeist_css::CssNumericConstructionErrorKind as N;
        let kind = match source {
            "atan2(1px, 1s)" => N::IncompatibleTypes,
            "atan2(1)" => N::Arity,
            _ => N::InvalidArgumentType,
        };
        assert_calculation_rejected(
            CssAngleCalculation::try_from_components(parse_component_values(source).unwrap()),
            Some(kind),
        );
    }
}

#[test]
fn percentage_resolution_depends_on_the_explicit_root_context() {
    for source in ["min(1px, 2%)", "clamp(none, 2%, 3px)", "hypot(3px, 4%)"] {
        let values = parse_component_values(source).unwrap();
        assert_calculation_rejected(
            CssLengthCalculation::try_from_components(values.clone()),
            Some(surgeist_css::CssNumericConstructionErrorKind::RootDomainMismatch),
        );
        assert!(
            CssLengthPercentageCalculation::try_from_components(values).is_ok(),
            "{source}"
        );
    }
    for source in ["min(1%, 2%)", "abs(-2%)", "round(2%, 1%)"] {
        assert!(
            CssPercentageCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_ok(),
            "{source}"
        );
    }
    // Percentages can cancel without requiring a layout basis in a pure root.
    assert!(
        CssLengthCalculation::try_from_components(
            parse_component_values("calc(10% / 10% * 1px)").unwrap()
        )
        .is_ok()
    );
}

#[test]
fn only_actual_whitespace_satisfies_binary_sum_separators() {
    for source in [
        "calc(1/**/+/**/2)",
        "calc(1 +/**/2)",
        "calc(1/**/+ 2)",
        "calc(-(1))",
    ] {
        assert_calculation_rejected(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap()),
            Some(surgeist_css::CssNumericConstructionErrorKind::MalformedExpression),
        );
    }
    for source in ["calc(1\t+\n2)", "calc(1 /*a*/+ /*b*/2)", "calc(-1 + +2)"] {
        assert!(
            CssNumberCalculation::try_from_components(parse_component_values(source).unwrap())
                .is_ok(),
            "{source}"
        );
    }
}
