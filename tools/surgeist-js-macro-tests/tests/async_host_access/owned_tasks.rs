use std::task::{Context as TaskContext, Poll, Waker};

use surgeist_js::{
    Context, ContextAccess, JsResult, JsValue, NativeFunction, Source,
    job::{ContextTask, JobObserver, JobStatus, JobStep, NativeAsyncJob},
};

fn callback(
    this: JsValue,
    args: Vec<JsValue>,
    _: ContextAccess<'_>,
) -> JsResult<ContextTask<JsValue>> {
    Ok(ContextTask::from_future(async move {
        Ok(JsValue::new(
            this.as_number().unwrap_or_default()
                + args
                    .first()
                    .and_then(JsValue::as_number)
                    .unwrap_or_default(),
        ))
    }))
}

fn assert_observer_traits<T: Send + Sync + Clone>() {}

fn main() {
    assert_observer_traits::<JobObserver>();
    let _native = NativeFunction::from_async_fn(callback);
    let mut job = NativeAsyncJob::new(|mut access| {
        access.eval(Source::from_bytes("globalThis.prepared = true"))?;
        Ok(ContextTask::from_poll(|_, host| {
            Poll::Ready(host.with(|mut access| access.eval(Source::from_bytes("40 + 2"))))
        }))
    });
    let observer = job.observer();
    let mut owner = Context::builder().build().unwrap();
    let host = owner.host_access();
    let mut task = TaskContext::from_waker(Waker::noop());
    assert!(matches!(job.run_step(&mut task, &host), JobStep::Prepared));
    let JobStep::Complete(result) = job.run_step(&mut task, &host) else {
        panic!("ready owned task must complete on its first poll");
    };
    assert_eq!(result.unwrap().as_number(), Some(42.0));
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert!(matches!(
        job.run_step(&mut task, &host),
        JobStep::Finished(Ok(JobStatus::Completed))
    ));
}
