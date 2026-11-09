use crate::test::{TestAction, run_test_actions};

#[test]
fn expired_listener_finishes_committed_abort_without_publishing_prepared_prefix() {
    use crate::publication_test_support::PublicationExecutor;
    use std::rc::Rc;
    use surgeist_js::{
        Context, JsObject, JsValue, Runtime, Source,
        error::{EngineError, RootingError},
        js_string,
        native_function::NativeCapture,
        object::builtins::JsFunction,
    };

    let executor = Rc::new(PublicationExecutor::new(Some(0)));
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let signal = super::JsAbortSignal::default();
    let first = context.eval(Source::from_bytes("(() => {})")).unwrap();
    let first = JsFunction::from_object(first.as_object().unwrap()).unwrap();
    signal
        .add_event_listener(js_string!("abort"), first.clone(), context.access())
        .unwrap();

    let expired_runtime = Runtime::default();
    {
        let mut expired_context = Context::builder()
            .runtime(expired_runtime.clone())
            .build()
            .unwrap();
        let later = expired_context
            .eval(Source::from_bytes("(() => {})"))
            .unwrap();
        let later = JsFunction::from_object(later.as_object().unwrap()).unwrap();
        signal
            .add_event_listener(js_string!("abort"), later, expired_context.access())
            .unwrap();
    }
    drop(expired_runtime);

    let token = signal.cancellation_token();
    let error = signal
        .signal_abort(JsValue::new(42), context.access())
        .unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Rooting(RootingError::Expired))
    );
    assert!(signal.is_aborted());
    assert!(token.is_cancelled());
    assert_eq!(
        signal.abort_reason(context.access()).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(executor.attempts.get(), 0);
    assert!(executor.observations.borrow().is_empty());
    {
        let listeners = signal.listeners.borrow();
        assert_eq!(listeners.len(), 2);
        assert!(JsObject::equals(
            &JsFunction::restore(&listeners[0].callback).unwrap(),
            &first,
        ));
        assert!(matches!(
            JsFunction::restore(&listeners[1].callback),
            Err(RootingError::Expired)
        ));
    }

    signal
        .signal_abort(JsValue::new(43), context.access())
        .unwrap();
    assert!(signal.is_aborted());
    assert!(token.is_cancelled());
    assert_eq!(
        signal.abort_reason(context.access()).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(executor.attempts.get(), 0);
    assert!(executor.observations.borrow().is_empty());
    assert_eq!(signal.listeners.borrow().len(), 2);
}

#[test]
fn publication_failure_finishes_abort_and_every_listener_tail() {
    use crate::publication_test_support::PublicationExecutor;
    use std::rc::Rc;
    use surgeist_js::{
        Context, JsValue, Source,
        error::{EngineError, EngineResourceError},
        job::{JobRetirement, JobStatus},
        js_string,
        object::builtins::JsFunction,
    };

    for accepted_attempt in [None, Some(0), Some(1)] {
        let executor = Rc::new(PublicationExecutor::new(accepted_attempt));
        let mut context = Context::builder()
            .job_executor(executor.clone())
            .build()
            .unwrap();
        let signal = super::JsAbortSignal::default();
        for _ in 0..2 {
            let callback = context.eval(Source::from_bytes("(() => {})")).unwrap();
            let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
            signal
                .add_event_listener(js_string!("abort"), callback, context.access())
                .unwrap();
        }
        let token = signal.cancellation_token();
        let error = signal
            .signal_abort(JsValue::new(42), context.access())
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::Resource(EngineResourceError::Allocation))
        );
        assert!(signal.is_aborted());
        assert!(token.is_cancelled());
        assert_eq!(
            signal.abort_reason(context.access()).unwrap(),
            JsValue::new(42)
        );
        assert_eq!(executor.attempts.get(), 2);
        let observations = executor.observations.borrow();
        let dropped = JobStatus::Retired(JobRetirement::Dropped);
        for (index, observer) in observations.iter().enumerate() {
            assert_eq!(
                observer.status().unwrap(),
                if accepted_attempt == Some(index) {
                    JobStatus::Pending
                } else {
                    dropped
                }
            );
        }
        drop(observations);
        signal
            .signal_abort(JsValue::new(43), context.access())
            .unwrap();
        assert_eq!(executor.attempts.get(), 2);
        assert_eq!(
            signal.abort_reason(context.access()).unwrap(),
            JsValue::new(42)
        );
        if let Some(index) = accepted_attempt {
            context.run_jobs().unwrap();
            assert_eq!(
                executor.observations.borrow()[index].status().unwrap(),
                JobStatus::Completed
            );
        }
    }
}

#[test]
fn abort_controller_exists() {
    run_test_actions([
        TestAction::run("let ctrl = new AbortController()"),
        TestAction::run("let sig = ctrl.signal"),
    ]);
}

#[test]
fn signal_not_aborted_initially() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        if (ctrl.signal.aborted !== false) {
            throw new Error('signal should not be aborted initially');
        }
        ",
    )]);
}

#[test]
fn abort_sets_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        ctrl.abort();
        if (ctrl.signal.aborted !== true) {
            throw new Error('signal should be aborted after abort()');
        }
        ",
    )]);
}

#[test]
fn abort_with_custom_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort("custom reason");
        if (ctrl.signal.reason !== "custom reason") {
            throw new Error('reason should be "custom reason", got: ' + ctrl.signal.reason);
        }
        "#,
    )]);
}

#[test]
fn abort_default_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort();
        if (ctrl.signal.reason.name !== "AbortError") {
            throw new Error('default reason.name should be "AbortError", got: ' + ctrl.signal.reason.name);
        }
        "#,
    )]);
}

#[test]
fn throw_if_aborted_does_nothing_when_not_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        ctrl.signal.throwIfAborted();
        ",
    )]);
}

#[test]
fn throw_if_aborted_throws_when_aborted() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort("test error");
        let threw = false;
        try {
            ctrl.signal.throwIfAborted();
        } catch (e) {
            threw = true;
            if (e !== "test error") {
                throw new Error('expected "test error", got: ' + e);
            }
        }
        if (!threw) {
            throw new Error('throwIfAborted should have thrown');
        }
        "#,
    )]);
}

#[test]
fn add_event_listener_fires_on_abort() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let called = false;
            ctrl.signal.addEventListener('abort', function() {
                called = true;
            });
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (!called) {
                throw new Error('abort listener was not called');
            }
            ",
        ),
    ]);
}

#[test]
fn add_event_listener_ignores_unknown_event_names() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let called = false;
            ctrl.signal.addEventListener('nope', function() {
                called = true;
            });
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (called) {
                throw new Error('unknown event listener should not fire');
            }
            ",
        ),
    ]);
}

#[test]
fn add_event_listener_does_not_fire_abort_listener_added_after_abort() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            ctrl.abort();

            let called = false;
            ctrl.signal.addEventListener('abort', function() {
                called = true;
            });
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (called) {
                throw new Error('abort listener should not fire when added after abort');
            }
            ",
        ),
    ]);
}

#[test]
fn add_event_listener_deduplicates_same_type_and_callback() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;

            function onAbort() {
                count += 1;
            }

            ctrl.signal.addEventListener('abort', onAbort);
            ctrl.signal.addEventListener('abort', onAbort);
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 1) {
                throw new Error('expected duplicate abort listener to fire once, got: ' + count);
            }
            ",
        ),
    ]);
}

#[test]
fn remove_event_listener_uses_event_type() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;

            function listener() {
                count += 1;
            }

            ctrl.signal.addEventListener('nope', listener);
            ctrl.signal.addEventListener('abort', listener);
            ctrl.signal.removeEventListener('nope', listener);
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 1) {
                throw new Error('removing a different event type should not remove the abort listener');
            }
            ",
        ),
    ]);
}

#[test]
fn multiple_listeners() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 3) {
                throw new Error('expected 3 listeners called, got: ' + count);
            }
            ",
        ),
    ]);
}

#[test]
fn repeated_abort_is_idempotent() {
    run_test_actions([
        TestAction::run(
            r"
            let ctrl = new AbortController();
            let count = 0;
            ctrl.signal.addEventListener('abort', function() { count += 1; });
            ctrl.abort();
            ctrl.abort();
            ctrl.abort();
            ",
        ),
        TestAction::inspect_context(|ctx| {
            ctx.run_jobs().unwrap();
        }),
        TestAction::run(
            r"
            if (count !== 1) {
                throw new Error('listeners should fire only once, got: ' + count);
            }
            ",
        ),
    ]);
}

#[test]
fn signal_reuse_across_references() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        let sig1 = ctrl.signal;
        let sig2 = ctrl.signal;
        ctrl.abort();
        if (sig1.aborted !== true || sig2.aborted !== true) {
            throw new Error('both signal references should be aborted');
        }
        ",
    )]);
}

#[test]
fn reason_undefined_when_not_aborted() {
    run_test_actions([TestAction::run(
        r"
        let ctrl = new AbortController();
        if (ctrl.signal.reason !== undefined) {
            throw new Error('reason should be undefined before abort, got: ' + ctrl.signal.reason);
        }
        ",
    )]);
}

#[test]
fn reason_abort_error_when_aborted_without_reason() {
    run_test_actions([TestAction::run(
        r#"
        let ctrl = new AbortController();
        ctrl.abort();
        if (!(ctrl.signal.reason instanceof Error)) {
            throw new Error('reason should be an instance of Error');
        }
        if (ctrl.signal.reason.name !== "AbortError") {
            throw new Error('reason.name should be "AbortError", got: ' + ctrl.signal.reason.name);
        }
        "#,
    )]);
}
