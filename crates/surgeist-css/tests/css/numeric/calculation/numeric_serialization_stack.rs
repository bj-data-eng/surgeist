//! The public depth budget applies to complete mathematical expressions,
//! including operator nodes between nested function components.
use crate::css::support::isolated_stack;

use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::numeric::{CssCalculation, CssLengthPercentage, CssNumber};

#[test]
fn mixed_operator_calculations_serialize_at_the_supported_depth() {
    isolated(
        "css::numeric::calculation::numeric_serialization_stack::mixed_operator_calculations_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let calculation = CssCalculation::<CssNumber>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), &source).unwrap(),
                Default::default(),
            )
            .unwrap();
            assert_eq!(
                calculation
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(calculation);
        },
    );
}

#[test]
fn mixed_operator_length_fragments_serialize_at_the_supported_depth() {
    isolated(
        "css::numeric::calculation::numeric_serialization_stack::mixed_operator_length_fragments_serialize_at_the_supported_depth",
        || {
            let source = format!("{}1px{}", "calc(1px + 2 * ".repeat(256), ")".repeat(256));
            let calculation = CssCalculation::<CssLengthPercentage>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), &source).unwrap(),
                Default::default(),
            )
            .unwrap();
            assert_eq!(
                calculation
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
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
