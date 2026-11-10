#![allow(unused_crate_dependencies)]

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    task::{Context as TaskContext, Waker},
};

use surgeist_js::{
    Context, JsValue, NativeFunction, Source,
    job::{ContextTask, JobStep, NativeAsyncJob},
    js_string,
};

#[test]
fn realm_free_async_job_unwind_releases_nested_guest_frames_before_reentry() {
    // Keep this owner unscoped: only the async job boundary can restore its
    // nested invocation state before the host inspects it.
    let mut context = Context::builder().build().unwrap();
    let original_realm = context.realm().clone();
    context
        .register_global_builtin_callable(
            js_string!("panicHost"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                panic!("nested realm-free async job callback panic");
            }),
        )
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.beforePanic = 0; globalThis.finallyRan = false;
             globalThis.panicWrapper = function panicWrapper() {
                 globalThis.beforePanic = 7;
                 try { panicHost(); } finally { globalThis.finallyRan = true; }
             };",
        ))
        .unwrap();
    let baseline_frames = context.stack_trace().count();
    assert_eq!(baseline_frames, 0);
    let mut job = NativeAsyncJob::new(|_| {
        Ok(ContextTask::from_poll(|_, host| {
            std::task::Poll::Ready(
                host.with(|mut access| access.eval(Source::from_bytes("panicWrapper()"))),
            )
        }))
    });

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let holder = context.host_access();
        let mut task_context = TaskContext::from_waker(Waker::noop());
        assert!(matches!(
            job.run_step(&mut task_context, &holder),
            JobStep::Prepared
        ));
        job.run_step(&mut task_context, &holder)
    }))
    .expect_err("the nested native panic must propagate outside the job future and holder");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"nested realm-free async job callback panic")
    );
    // Snapshot both immediately, before a later evaluator could hide residue.
    let remaining_frames = context.stack_trace().count();
    let restored_realm = context.realm().clone();
    assert_eq!(
        remaining_frames, baseline_frames,
        "a realm-free async job must release its own nested guest invocation on unwind"
    );
    assert_eq!(restored_realm, original_realm);
    assert_eq!(
        context.eval(Source::from_bytes("beforePanic")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        context.eval(Source::from_bytes("finallyRan")).unwrap(),
        JsValue::new(false)
    );
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}
