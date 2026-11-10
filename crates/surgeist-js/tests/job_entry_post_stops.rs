#![allow(unused_crate_dependencies)]

use std::{
    cell::RefCell,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{ContextTask, JobStep, NativeAsyncJob},
};

fn assert_poll_returns_its_retained_stop(pending: bool) {
    let mut context = Context::builder().build().unwrap();
    let actual_poll = RefCell::new(None);
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, host| {
            let stopped = host
                .with(|mut access| access.eval(Source::from_bytes("0")))
                .unwrap_err();
            assert_eq!(
                stopped.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            if pending {
                Poll::Pending
            } else {
                Poll::Ready(Ok(JsValue::new(42)))
            }
        }))
    });
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let host = access.into_host_access();
            let mut cx = TaskContext::from_waker(Waker::noop());
            assert!(matches!(job.run_step(&mut cx, &host), JobStep::Prepared));
            *actual_poll.borrow_mut() = Some(job.run_step(&mut cx, &host));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);

    // The outer scope already preserves its first stop. The job's own actual
    // poll must expose that stop, without a success or another Pending escaping.
    let actual = actual_poll.into_inner().expect("the job was polled once");
    let stopped = match &actual {
        JobStep::Complete(Err(error)) => error.as_engine(),
        _ => None,
    };
    assert_eq!(
        stopped,
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        })),
        "the actual job poll must preserve its retained stop: {actual:?}"
    );
    assert_eq!(
        context.eval(Source::from_bytes("40 + 2")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn async_job_exposes_a_stop_retained_by_a_poll_that_returns_pending() {
    assert_poll_returns_its_retained_stop(true);
}

#[test]
fn async_job_exposes_a_stop_retained_by_a_poll_that_returns_success() {
    assert_poll_returns_its_retained_stop(false);
}
