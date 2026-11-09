use crate::test::{TestAction, run_test_actions_with};
use indoc::indoc;
use surgeist_js::{Context, js_str};

#[test]
fn publication_rejection_is_returned_and_disposes_the_unstarted_microtask() {
    use crate::publication_test_support::PublicationExecutor;
    use std::rc::Rc;
    use surgeist_js::{
        Source,
        error::{EngineError, EngineResourceError},
        job::{JobRetirement, JobStatus},
        object::builtins::JsFunction,
    };

    let executor = Rc::new(PublicationExecutor::new(false));
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let callback = context
        .eval(Source::from_bytes("(() => { throw 'must not run'; })"))
        .unwrap();
    let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
    let error = super::js_module::queue_microtask(callback, context.access()).unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::Resource(EngineResourceError::Allocation))
    );
    let observations = executor.observations.borrow();
    assert_eq!(observations.len(), 1);
    assert_eq!(
        observations[0].status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
}

#[test]
fn accepted_publication_error_keeps_microtask_custody() {
    use crate::publication_test_support::PublicationExecutor;
    use std::rc::Rc;
    use surgeist_js::{Source, job::JobStatus, object::builtins::JsFunction};

    let executor = Rc::new(PublicationExecutor::new(true));
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let callback = context.eval(Source::from_bytes("(() => {})")).unwrap();
    let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
    assert!(super::js_module::queue_microtask(callback, context.access()).is_err());
    assert_eq!(
        executor.observations.borrow()[0].status().unwrap(),
        JobStatus::Pending
    );
    context.run_jobs().unwrap();
    assert_eq!(
        executor.observations.borrow()[0].status().unwrap(),
        JobStatus::Completed
    );
}

#[test]
fn explicit_discard_does_not_execute_a_microtask() {
    use std::rc::Rc;
    use surgeist_js::{JsValue, Source, job::IdleJobExecutor, object::builtins::JsFunction};

    let mut context = Context::builder()
        .job_executor(Rc::new(IdleJobExecutor))
        .build()
        .unwrap();
    let callback = context
        .eval(Source::from_bytes(
            "globalThis.called = false; (() => { called = true; })",
        ))
        .unwrap();
    let callback = JsFunction::from_object(callback.as_object().unwrap()).unwrap();
    super::js_module::queue_microtask(callback, context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("called")).unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn queue_microtask() {
    let context = &mut Context::default();
    crate::microtask::register(None, context).unwrap();

    run_test_actions_with(
        [
            // Record execution order in a global array instead of relying on
            // `console`, which is not part of this crate's test surface.
            TestAction::run(indoc! {r#"
                order = [];
                order.push(1);
                queueMicrotask(() => order.push(2));
                order.push(3);
                queueMicrotask(() => {
                    order.push(4);
                    queueMicrotask(() => {
                        order.push(5);
                        queueMicrotask(() => order.push(6));
                        order.push(7);
                    });
                    order.push(8);
                });
                order.push(9);
            "#}),
            TestAction::inspect_context(|ctx| {
                ctx.run_jobs().unwrap();
            }),
            TestAction::inspect_context(|ctx| {
                let order = ctx
                    .global_object()
                    .get(js_str!("order"), ctx.access())
                    .unwrap();
                let order = order.as_object().unwrap();

                assert_eq!(
                    order.get(js_str!("length"), ctx.access()).unwrap().as_i32(),
                    Some(9),
                    "all nine pushes must run",
                );

                // Synchronous pushes run first (1, 3, 9); the queued microtasks
                // then drain in FIFO order, with nested microtasks appended as
                // they are enqueued.
                for (i, expected) in [1, 3, 9, 2, 4, 8, 5, 7, 6].into_iter().enumerate() {
                    assert_eq!(
                        order.get(i, ctx.access()).unwrap().as_i32(),
                        Some(expected),
                        "order[{i}] mismatch",
                    );
                }
            }),
        ],
        context,
    );
}
