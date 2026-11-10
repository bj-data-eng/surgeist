use crate::error::RuntimeLimitError;
use surgeist_js_parser::frontend::InternerLimitKind;

#[test]
fn direct_eval_admits_lookup_of_an_ambient_private_name_absent_from_the_source() {
    for limit in [15, 19] {
        let mut context = Context::builder().build().unwrap();
        context
            .eval(Source::from_bytes(
                "globalThis.probe = new (class { #café; m() { return eval('1'); } })().m;",
            ))
            .unwrap();
        // The caller hashes, converts and publishes the five ASCII bytes
        // of `probe`. The eval source is numeric, so its only encoded byte
        // work is hashing the four Latin1 bytes of the ambient description.
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes("probe()"),
                ExecutionOptions::new(128).with_interner_byte_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.interner_bytes().unwrap(), limit);
        assert!(evaluation.report.instructions().unwrap() > 0);
        if limit == 15 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
                    kind: InternerLimitKind::Bytes,
                    limit,
                }))
            );
        } else {
            assert_eq!(evaluation.result.unwrap(), JsValue::new(1));
        }
    }
}

#[test]
fn direct_eval_deduplicates_ambient_symbols_without_changing_shadowing() {
    let setup = "
        globalThis.probe = new (class {
            #𐐀 = 1;
            m() {
                return new (class {
                    #𐐀 = 2;
                    #unseen;
                    m() { return eval('this.#𐐀'); }
                })().m();
            }
        })().m;
    ";
    for limit in [1, 2] {
        let mut context = Context::builder().build().unwrap();
        context.eval(Source::from_bytes(setup)).unwrap();
        // One comparison deduplicates the resolved outer symbol. The next
        // checks the private access against the projected symbol.
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes("probe()"),
                ExecutionOptions::new(128).with_name_comparison_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.name_comparisons().unwrap(), limit);
        if limit == 1 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::NameComparison { limit }
                ))
            );
        } else {
            assert_eq!(evaluation.result.unwrap(), JsValue::new(2));
        }
    }
}
use crate::{Context, EngineError, JsValue, Source, execution::ExecutionOptions};

#[test]
fn direct_eval_private_validation_precedes_environment_poisoning() {
    let source = "eval('(class { #a; #b; m() { return this.#b; } }); 42')";
    for limit in [3, 4] {
        let mut context = Context::builder().build().unwrap();
        let environment = context.realm().environment();
        assert!(!environment.poisoned());

        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(128).with_name_comparison_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.name_comparisons().unwrap(), limit);
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
