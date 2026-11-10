use crate::{
    Context, JsError,
    error::{EngineError, EngineResourceError},
    execution::ExecutionOptions,
};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
};

fn first() -> JsError {
    EngineResourceError::SizeOverflow.into()
}
fn assert_first(error: &JsError) {
    assert!(matches!(
        error.as_engine(),
        Some(EngineError::Resource(EngineResourceError::SizeOverflow))
    ));
}

#[test]
fn nested_module_runs_reuse_first_cause_and_release_it_before_fresh_work() {
    let mut context = Context::builder().build().unwrap();
    let metadata_closed = Cell::new(false);
    let result = context.with_module_run(|context| {
        let mut error = first();
        assert!(context.retain_committed_reaction_failure(&mut error));
        context.with_module_run(|context| {
            let mut later = EngineResourceError::Allocation.into();
            assert!(context.retain_committed_reaction_failure(&mut later));
            assert_first(&later);
            Ok(())
        })?;
        metadata_closed.set(true);
        Ok(17)
    });
    assert_first(&result.unwrap_err());
    assert!(metadata_closed.get());
    assert!(context.module_run_failure.is_none());
    context.check_execution_stop().unwrap();
    assert_eq!(context.with_module_run(|_| Ok(29)).unwrap(), 29);
    let mut outside = first();
    assert!(!context.retain_committed_reaction_failure(&mut outside));
    context.check_execution_stop().unwrap();
}

#[derive(Debug)]
struct HostPanic;

#[test]
fn unwinding_a_module_run_removes_its_temporary_stop() {
    let mut context = Context::builder().build().unwrap();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        context.with_module_run::<()>(|context| {
            let mut error = first();
            assert!(context.retain_committed_reaction_failure(&mut error));
            panic_any(HostPanic)
        })
    }))
    .unwrap_err();
    assert!(panic.is::<HostPanic>());
    assert!(context.module_run_failure.is_none());
    context.check_execution_stop().unwrap();
    assert_eq!(context.with_module_run(|_| Ok(41)).unwrap(), 41);
}

#[test]
fn selected_execution_retains_the_cause_without_a_temporary_module_owner() {
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .with_execution(ExecutionOptions::new(4), |access| {
            let context = access.into_owner();
            let direct = context.with_module_run(|context| {
                assert!(context.module_run_failure.is_none());
                let mut error = first();
                assert!(context.retain_committed_reaction_failure(&mut error));
                assert_first(&context.check_execution_stop().unwrap_err());
                Ok(53)
            });
            assert_first(direct.as_ref().unwrap_err());
            direct
        })
        .unwrap();
    assert_first(&outcome.result.unwrap_err());
    assert!(context.module_run_failure.is_none());
    context.check_execution_stop().unwrap();
}
