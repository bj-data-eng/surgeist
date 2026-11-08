#![allow(unused_crate_dependencies)]

use std::{
    cell::RefCell,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    task::{Context as TaskContext, Waker},
};

use surgeist_js::{Context, JsValue, Source, job::NativeAsyncJob};

#[test]
fn a_panicking_async_job_poll_restores_its_caller_realm() {
    let mut context = Context::default();
    let original_realm = context.realm().clone();
    let selected_realm = context.create_realm().unwrap();
    assert_ne!(selected_realm, original_realm);
    let expected_realm = selected_realm.clone();
    let job = NativeAsyncJob::with_realm(
        async move |context| {
            assert_eq!(context.borrow().realm(), &expected_realm);
            panic!("async job poll panic");
        },
        selected_realm,
    );

    let panic = {
        let holder = RefCell::new(&mut context);
        let mut future = job.call(&holder);
        let mut task_context = TaskContext::from_waker(Waker::noop());
        catch_unwind(AssertUnwindSafe(|| {
            Pin::new(&mut future).poll(&mut task_context)
        }))
        .expect_err("the async callback panic must propagate to its host caller")
    };
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"async job poll panic"));
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(context.realm(), &original_realm);
}
