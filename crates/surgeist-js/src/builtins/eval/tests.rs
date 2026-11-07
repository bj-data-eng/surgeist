use crate::error::RuntimeLimitError;
use crate::{Context, EngineError, JsValue, Source, execution::ExecutionOptions};

#[test]
fn direct_eval_private_validation_precedes_environment_poisoning() {
    let source = "eval('(class { #a; #b; m() { return this.#b; } }); 42')";
    for limit in [3, 4] {
        let mut context = Context::default();
        let environment = context.realm().environment();
        assert!(!environment.poisoned());

        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(128).with_name_comparison_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.name_comparisons(), limit);
        if limit == 3 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::NameComparison { limit }
                ))
            );
            assert!(!environment.poisoned());
        } else {
            assert_eq!(evaluation.result.unwrap(), JsValue::new(42));
            assert!(environment.poisoned());
        }
    }
}
