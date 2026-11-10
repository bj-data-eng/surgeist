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
fn nested_guest_panic_in_async_job_restores_frames_and_realm_before_reentry() {
    let mut context = Context::builder().build().unwrap();
    let caller_realm = context.realm().clone();
    let selected_realm = context.create_realm().unwrap();
    assert_ne!(selected_realm, caller_realm);
    context.enter_realm(selected_realm.clone()).unwrap();
    context
        .register_global_builtin_callable(
            js_string!("panicHost"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                panic!("nested async native job callback panic");
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
    context.enter_realm(caller_realm.clone()).unwrap();
    let baseline_frames = context.stack_trace().count();
    let expected_realm = selected_realm.clone();
    let mut job = NativeAsyncJob::with_realm(
        move |_| {
            Ok(ContextTask::from_poll(move |_, holder| {
                std::task::Poll::Ready(holder.with(|mut access| {
                    assert_eq!(access.realm(), &expected_realm);
                    access.eval(Source::from_bytes("panicWrapper()"))
                }))
            }))
        },
        selected_realm.clone(),
    );

    let panic = {
        let holder = context.host_access();
        let mut task_context = TaskContext::from_waker(Waker::noop());
        assert!(matches!(
            job.run_step(&mut task_context, &holder),
            JobStep::Prepared
        ));
        catch_unwind(AssertUnwindSafe(|| {
            job.run_step(&mut task_context, &holder)
        }))
        .expect_err("the nested native panic must propagate to the job's host caller")
    };
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"nested async native job callback panic")
    );
    // Observe cleanup immediately: no fresh engine operation may repair it first.
    let remaining_frames = context.stack_trace().count();
    let restored_realm = context.realm().clone();
    assert_eq!(
        remaining_frames, baseline_frames,
        "unwinding the async poll must release its nested guest frames"
    );
    assert_eq!(restored_realm, caller_realm);

    context.enter_realm(selected_realm).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("beforePanic")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        context.eval(Source::from_bytes("finallyRan")).unwrap(),
        JsValue::new(false)
    );
    context.enter_realm(caller_realm).unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}
