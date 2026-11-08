#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{NativeAsyncJob, SimpleJobExecutor},
};

fn instruction_stop() -> EngineError {
    EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 0 })
}

fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}

async fn yield_once() {
    let mut yielded = false;
    std::future::poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}

#[test]
fn queued_async_job_refuses_callback_entry_after_a_retained_instruction_stop() {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap();
    let callbacks = Rc::new(Cell::new(0));
    let entered = callbacks.clone();
    context.enqueue_job(
        NativeAsyncJob::new(async move |_| {
            entered.set(entered.get() + 1);
            Ok(JsValue::new(42))
        })
        .into(),
    );
    let actual_drain = RefCell::new(None);
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let stopped = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(stopped.as_engine(), Some(&instruction_stop()));
            // Preserve the actual drain result separately. The outer scope's
            // first-stop precedence cannot prove that the driver refused entry.
            *actual_drain.borrow_mut() = Some(access.run_jobs());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&instruction_stop())
    );
    assert_eq!(outcome.report.instructions(), 0);
    let drained = actual_drain
        .into_inner()
        .expect("the host must actually invoke the queued driver after retaining the stop");
    assert_eq!(
        drained.as_ref().err().and_then(|error| error.as_engine()),
        Some(&instruction_stop()),
        "actual drain={drained:?}; callback entries={}",
        callbacks.get()
    );
    assert_eq!(
        callbacks.get(),
        0,
        "a refused job must not invoke its callback; actual drain={drained:?}"
    );
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn pending_async_job_refuses_its_next_poll_after_an_external_retained_stop() {
    let mut context = Context::default();
    let selected = context.create_realm().unwrap();
    let entered = Rc::new(Cell::new(0));
    let continued = Rc::new(Cell::new(0));
    let callback_entered = entered.clone();
    let callback_continued = continued.clone();
    let job = NativeAsyncJob::with_realm(
        async move |_| {
            callback_entered.set(callback_entered.get() + 1);
            yield_once().await;
            callback_continued.set(callback_continued.get() + 1);
            Ok(JsValue::new(42))
        },
        selected,
    );
    let actual_poll = RefCell::new(None);
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            let mut future = job.call(&holder);
            assert!(poll(&mut future).is_pending());
            assert_eq!(entered.get(), 1);
            assert_eq!(continued.get(), 0);
            // This is host code outside the pending job, using the same active
            // operation. It establishes a real stop between engine-owned polls.
            let stopped = holder
                .with(|mut access| access.eval(Source::from_bytes("0")))
                .unwrap_err();
            assert_eq!(stopped.as_engine(), Some(&instruction_stop()));
            *actual_poll.borrow_mut() = Some(poll(&mut future));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&instruction_stop())
    );
    assert_eq!(outcome.report.instructions(), 0);
    let polled = actual_poll
        .into_inner()
        .expect("the host must actually request the next engine-owned job poll");
    let stopped = match &polled {
        Poll::Ready(Err(error)) => error.as_engine(),
        _ => None,
    };
    assert_eq!(
        stopped,
        Some(&instruction_stop()),
        "actual poll={polled:?}; initial entries={}; resumed body entries={}",
        entered.get(),
        continued.get()
    );
    assert_eq!(entered.get(), 1);
    assert_eq!(
        continued.get(),
        0,
        "the refused job poll must not resume its host body; actual poll={polled:?}"
    );
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}
