//! The public depth budget applies to complete mathematical expressions,
//! including operator nodes between nested function components.
#[path = "support/isolated_stack.rs"]
mod isolated_stack;

use surgeist_css::{CssLengthPercentageCalculation, CssNumberCalculation, parse_component_values};

#[test]
fn mixed_operator_calculations_serialize_at_the_supported_depth() {
    isolated(
        "mixed_operator_calculations_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let calculation =
                CssNumberCalculation::try_from_components(parse_component_values(&source).unwrap())
                    .unwrap();
            assert_eq!(calculation.serialize().unwrap().as_css(), source);
            drop(calculation);
        },
    );
}

#[test]
fn mixed_operator_length_fragments_serialize_at_the_supported_depth() {
    isolated(
        "mixed_operator_length_fragments_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1px{}", "calc(1px + 2 * ".repeat(256), ")".repeat(256));
            let calculation = CssLengthPercentageCalculation::try_from_components(
                parse_component_values(&source).unwrap(),
            )
            .unwrap();
            assert_eq!(calculation.serialize().unwrap().as_css(), source);
            drop(calculation);
        },
    );
}

fn isolated(test: &str, operation: fn()) {
    isolated_stack::run(
        test,
        "SURGEIST_NUMERIC_SERIALIZATION_STACK_CHILD",
        "mixed numeric serialization and destruction completed",
        None,
        operation,
    );
}
