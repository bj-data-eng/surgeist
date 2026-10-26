//! The supported component depth also applies to public owned-value operations,
//! even when every function contains both a sum and a product expression node.
use crate::css::support::isolated_stack;

use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::{
    CssCalculation, CssComponentValueError, CssComponentValueLimits, CssLengthPercentage,
    CssNumber, parse_component_values,
};

#[derive(Default)]
struct DebugSink {
    wrote: bool,
}

impl std::fmt::Write for DebugSink {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.wrote |= !text.is_empty();
        Ok(())
    }
}

fn exercise_debug(value: &impl std::fmt::Debug) {
    let mut sink = DebugSink::default();
    std::fmt::write(&mut sink, format_args!("{value:?}")).unwrap();
    assert!(sink.wrote, "Debug completed and emitted output");
}

#[test]
fn mixed_number_tree_owned_lifecycle_completes_at_supported_depth() {
    isolated(
        "css::numeric::calculation::numeric_lifecycle_stack::mixed_number_tree_owned_lifecycle_completes_at_supported_depth",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let value = CssCalculation::<CssNumber>::try_from_components(
                parse_component_values(&source).unwrap(),
                Default::default(),
            )
            .unwrap();
            let cloned = value.clone();
            assert_eq!(cloned, value);
            exercise_debug(&value);
            assert_eq!(
                cloned
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(value);
            assert_eq!(
                cloned
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(cloned);
        },
    );
}

#[test]
fn mixed_length_tree_owned_lifecycle_completes_at_supported_depth() {
    isolated(
        "css::numeric::calculation::numeric_lifecycle_stack::mixed_length_tree_owned_lifecycle_completes_at_supported_depth",
        || {
            let source = format!("{}1px{}", "calc(1px + 2 * ".repeat(256), ")".repeat(256));
            let value = CssCalculation::<CssLengthPercentage>::try_from_components(
                parse_component_values(&source).unwrap(),
                Default::default(),
            )
            .unwrap();
            let cloned = value.clone();
            assert_eq!(cloned, value);
            exercise_debug(&value);
            assert_eq!(
                cloned
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(value);
            assert_eq!(
                cloned
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(cloned);
        },
    );
}

#[test]
fn tighter_limits_reject_and_release_deep_inputs_without_aborting_the_caller() {
    isolated(
        "css::numeric::calculation::numeric_lifecycle_stack::tighter_limits_reject_and_release_deep_inputs_without_aborting_the_caller",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let components = parse_component_values(&source).unwrap();
            let admitted = CssCalculation::<CssNumber>::try_from_components(
                components.clone(),
                Default::default(),
            )
            .unwrap();
            for (limits, expected) in [
                (
                    CssComponentValueLimits::try_new(255, usize::MAX, usize::MAX).unwrap(),
                    ComponentFailure::NestingLimit,
                ),
                (
                    CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
                    ComponentFailure::ComponentLimit,
                ),
                (
                    CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap(),
                    ComponentFailure::ByteLimit,
                ),
            ] {
                let error =
                    CssCalculation::<CssNumber>::try_from_components(components.clone(), limits)
                        .unwrap_err();
                assert!(error.is_resource_limit(), "{error:?}");
                let component = std::error::Error::source(&error)
                    .and_then(|source| source.downcast_ref::<CssComponentValueError>())
                    .expect("original typed component resource failure");
                assert_eq!(component_failure(component), expected);
                assert!(error.origin().is_some());
                drop(error);
            }
            assert_eq!(
                admitted
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                source
            );
            drop(admitted);
            drop(components);
        },
    );
}

fn isolated(test: &str, operation: fn()) {
    isolated_stack::run(
        test,
        "SURGEIST_NUMERIC_LIFECYCLE_STACK_CHILD",
        "numeric owned lifecycle and destruction completed",
        None,
        operation,
    );
}
