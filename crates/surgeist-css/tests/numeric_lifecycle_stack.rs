#![forbid(unsafe_code)]
//! The supported component depth also applies to public owned-value operations,
//! even when every function contains both a sum and a product expression node.
#[path = "support/isolated_stack.rs"]
mod isolated_stack;

use surgeist_css::{
    CssComponentValueError, CssComponentValueErrorKind, CssComponentValueLimits,
    CssLengthPercentageCalculation, CssNumberCalculation, CssNumericConstructionErrorKind,
    parse_component_values,
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
        "mixed_number_tree_owned_lifecycle_completes_at_supported_depth",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let value =
                CssNumberCalculation::try_from_components(parse_component_values(&source).unwrap())
                    .unwrap();
            let cloned = value.clone();
            assert_eq!(cloned, value);
            exercise_debug(&value);
            assert_eq!(cloned.serialize().unwrap().as_css(), source);
            drop(value);
            assert_eq!(cloned.serialize().unwrap().as_css(), source);
            drop(cloned);
        },
    );
}

#[test]
fn mixed_length_tree_owned_lifecycle_completes_at_supported_depth() {
    isolated(
        "mixed_length_tree_owned_lifecycle_completes_at_supported_depth",
        || {
            let source = format!("{}1px{}", "calc(1px + 2 * ".repeat(256), ")".repeat(256));
            let value = CssLengthPercentageCalculation::try_from_components(
                parse_component_values(&source).unwrap(),
            )
            .unwrap();
            let cloned = value.clone();
            assert_eq!(cloned, value);
            exercise_debug(&value);
            assert_eq!(cloned.serialize().unwrap().as_css(), source);
            drop(value);
            assert_eq!(cloned.serialize().unwrap().as_css(), source);
            drop(cloned);
        },
    );
}

#[test]
fn tighter_limits_reject_and_release_deep_inputs_without_aborting_the_caller() {
    isolated(
        "tighter_limits_reject_and_release_deep_inputs_without_aborting_the_caller",
        || {
            let source = format!("{}1{}", "calc(1 + 2 * ".repeat(256), ")".repeat(256));
            let components = parse_component_values(&source).unwrap();
            let admitted = CssNumberCalculation::try_from_components(components.clone()).unwrap();
            for (limits, expected) in [
                (
                    CssComponentValueLimits::try_new(255, usize::MAX, usize::MAX).unwrap(),
                    CssComponentValueErrorKind::NestingLimit,
                ),
                (
                    CssComponentValueLimits::try_new(256, 1, usize::MAX).unwrap(),
                    CssComponentValueErrorKind::ComponentLimit,
                ),
                (
                    CssComponentValueLimits::try_new(256, usize::MAX, source.len() - 1).unwrap(),
                    CssComponentValueErrorKind::ByteLimit,
                ),
            ] {
                let error = CssNumberCalculation::try_from_components_with_limits(
                    components.clone(),
                    limits,
                )
                .unwrap_err();
                assert_eq!(
                    error.kind(),
                    &CssNumericConstructionErrorKind::ResourceLimit
                );
                let component = std::error::Error::source(&error)
                    .and_then(|source| source.downcast_ref::<CssComponentValueError>())
                    .expect("original typed component resource failure");
                assert_eq!(component.kind(), expected);
                assert!(error.origin().is_some());
                drop(error);
            }
            assert_eq!(admitted.serialize().unwrap().as_css(), source);
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
