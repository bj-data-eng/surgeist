#![allow(unused_crate_dependencies)]

use std::{path::Path, rc::Rc};

use surgeist_js::{
    Context, JsError, JsNativeError, JsValue, NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{ContextTask, SimpleJobExecutor},
    js_string,
    object::builtins::JsPromise,
};

#[test]
fn retained_stop_prevents_async_error_materialization_from_mutating_saved_stack() {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap();
    let saved_object = JsNativeError::error()
        .with_message("saved error")
        .into_opaque(context.access())
        .unwrap();
    let saved_value = JsValue::from(saved_object.clone());
    context
        .global_object()
        .set(
            js_string!("savedError"),
            saved_value.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let stack_before = saved_object
        .get(js_string!("stack"), context.access())
        .unwrap();
    assert_eq!(stack_before, JsValue::undefined());

    // Obtain a real opaque throw completion with a captured guest backtrace.
    // Reading/formatting the completion must not itself materialize it again.
    let saved_error = context
        .eval(Source::from_bytes("throw savedError").with_path(Path::new("saved-error-origin.js")))
        .unwrap_err();
    assert_eq!(saved_error.as_opaque(), Some(&saved_value));
    assert!(saved_error.to_string().contains("saved-error-origin.js"));
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before
    );

    let promise = JsPromise::from_task(
        move |_| {
            Ok(ContextTask::from_poll(move |_, context| {
                let first = context
                    .with(|mut access| access.eval(Source::from_bytes("0")))
                    .unwrap_err();
                assert_eq!(
                    first.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 0
                    }))
                );
                // Host code swallows its work stop and returns a distinct ordinary
                // guest error whose object was already reachable before this scope.
                std::task::Poll::Ready(Err(saved_error.clone()))
            }))
        },
        context.access(),
    )
    .unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before,
        "retained host stop must precede writes to the preexisting guest Error"
    );
}

#[test]
fn retained_stop_prevents_sync_executor_error_materialization_from_mutating_saved_stack() {
    let mut context = Context::default();
    let saved_object = JsNativeError::error()
        .with_message("saved synchronous error")
        .into_opaque(context.access())
        .unwrap();
    let saved_value = JsValue::from(saved_object.clone());
    context
        .global_object()
        .set(
            js_string!("savedError"),
            saved_value.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let stack_before = saved_object
        .get(js_string!("stack"), context.access())
        .unwrap();
    assert_eq!(stack_before, JsValue::undefined());
    let saved_error = context
        .eval(
            Source::from_bytes("throw savedError")
                .with_path(Path::new("sync-saved-error-origin.js")),
        )
        .unwrap_err();
    assert_eq!(saved_error.as_opaque(), Some(&saved_value));
    assert!(
        saved_error
            .to_string()
            .contains("sync-saved-error-origin.js")
    );
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before
    );
    assert!(context.insert_data(saved_error).is_none());

    let invoke_executor = NativeFunction::from_fn_ptr(|_, _, mut context| {
        let saved_error = *context
            .remove_data::<JsError>()
            .expect("staged guest error");
        JsPromise::new(
            move |_, mut context| {
                let first = context.eval(Source::from_bytes("0")).unwrap_err();
                assert_eq!(
                    first.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 0
                    }))
                );
                Err(saved_error)
            },
            context,
        )
        .map(JsValue::from)
    });
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            // Direct host entry reaches the synchronous executor without admitting
            // an unrelated outer guest instruction before the selected zero limit.
            invoke_executor.call(&JsValue::undefined(), &[], access.reborrow())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(
        saved_object
            .get(js_string!("stack"), context.access())
            .unwrap(),
        stack_before,
        "retained host stop must precede synchronous executor error materialization"
    );
}
