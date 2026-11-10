#![allow(unused_crate_dependencies)]

use surgeist_js::{
    Context, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionTermination, ProfileMode},
};

#[test]
fn idle_context_move_and_swap_keep_committed_state_and_completed_report() {
    let mut original = Context::builder().build().unwrap();
    let completed = original
        .with_execution(
            ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            |mut access| access.eval(Source::from_bytes("globalThis.originalValue = 7")),
        )
        .unwrap();
    assert_eq!(completed.result.unwrap(), JsValue::new(7));
    let instructions = completed.report.instructions().unwrap();
    assert!(instructions > 0);
    let retained = completed.report.clone();
    let mut second = Context::builder().build().unwrap();
    second
        .eval(Source::from_bytes("globalThis.otherValue = 42"))
        .unwrap();
    let mut moved = original;
    std::mem::swap(&mut moved, &mut second);
    assert_eq!(
        second.eval(Source::from_bytes("originalValue")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        moved.eval(Source::from_bytes("otherValue")).unwrap(),
        JsValue::new(42)
    );

    let stopped = second
        .with_execution(ExecutionOptions::new(0), |mut access| {
            access.eval(Source::from_bytes("globalThis.afterStop = true"))
        })
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(stopped.report.instructions().unwrap(), 0);
    let selected = moved
        .with_execution(ExecutionOptions::new(64), |mut access| {
            access.eval(Source::from_bytes("otherValue"))
        })
        .unwrap();
    assert_eq!(selected.result.unwrap(), JsValue::new(42));
    assert!(selected.report.instructions().unwrap() > 0);
    assert_eq!(
        second
            .eval(Source::from_bytes("typeof afterStop === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(retained.instructions().unwrap(), instructions);
    assert!(matches!(
        retained.termination(),
        ExecutionTermination::Completed
    ));
    assert!(std::ptr::eq(
        completed.report.profile().unwrap().unwrap(),
        retained.profile().unwrap().unwrap()
    ));
}

#[test]
fn rebinding_local_access_to_independent_context_keeps_original_operation_custody() {
    let mut original = Context::builder().build().unwrap();
    let mut independent = Context::builder().build().unwrap();
    let outcome = original
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let mut selected = access.reborrow();
            let mut original_access = std::mem::replace(&mut selected, independent.access());
            assert_eq!(
                selected.eval(Source::from_bytes("globalThis.independentValue = 42"))?,
                JsValue::new(42)
            );
            original_access.eval(Source::from_bytes("globalThis.originalEffect = true"))
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert!(!outcome.report.effects_started());
    assert_eq!(
        original
            .eval(Source::from_bytes(
                "typeof originalEffect === 'undefined' && typeof independentValue === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        independent
            .eval(Source::from_bytes("independentValue"))
            .unwrap(),
        JsValue::new(42)
    );
    let fresh = original
        .with_execution(ExecutionOptions::new(64), |mut access| {
            access.eval(Source::from_bytes("42"))
        })
        .unwrap();
    assert_eq!(fresh.result.unwrap(), JsValue::new(42));
    assert!(fresh.report.instructions().unwrap() > 0);
}
