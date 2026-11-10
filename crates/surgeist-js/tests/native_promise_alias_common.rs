use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsObject, JsResult, JsValue,
    error::{EngineError, ObjectAccessError},
    job::{
        ContextTask, Job, JobAcceptance, JobExecutor, JobPublicationError, JobStep, NativeAsyncJob,
    },
    object::builtins::{JsFuture, JsPromise},
};

#[derive(Default)]
struct ManualExecutor(RefCell<VecDeque<Job>>);
impl JobExecutor for ManualExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the real async wrapper is dispatched explicitly")
    }
}
impl ManualExecutor {
    fn take_async(&self) -> NativeAsyncJob {
        let queued = self.0.borrow_mut().pop_front();
        let Some(Job::AsyncJob(job)) = queued else {
            panic!("the authored native promise must publish its actual async job")
        };
        job
    }
}
fn context(executor: Rc<ManualExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}
fn poll(future: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn finish(job: &mut NativeAsyncJob, owner: &mut Context) {
    let host = owner.host_access();
    assert!(matches!(
        job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host),
        JobStep::Prepared
    ));
    let JobStep::Complete(result) =
        job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host)
    else {
        panic!("the released promise must accept its original resolver stage")
    };
    result.unwrap();
}

#[test]
fn native_registration_under_a_mutable_promise_alias_returns_a_host_error_and_keeps_the_job_viable()
{
    let executor = Rc::new(ManualExecutor::default());
    let mut owner = context(executor.clone());
    let value = JsObject::with_null_proto(owner.runtime()).unwrap();
    let supplied = value.clone();
    let factories = Rc::new(Cell::new(0));
    let made = factories.clone();
    let promise = JsPromise::from_task(
        move |_| {
            made.set(made.get() + 1);
            Ok(ContextTask::from_future(std::future::ready(Ok(
                supplied.into()
            ))))
        },
        owner.access(),
    )
    .unwrap();
    let mut job = executor.take_async();
    let alias = Deref::deref(&promise).clone();
    let guard = alias.borrow_mut().unwrap();
    let attempted = catch_unwind(AssertUnwindSafe(|| {
        promise.clone().into_js_future(owner.access())
    }));
    assert_eq!(factories.get(), 0);
    drop(guard);

    // Exercise actual recovery before inspecting the contention outcome. Both
    // revisions must preserve the original unstarted job and its authored value.
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    finish(&mut job, &mut owner);
    assert_eq!(factories.get(), 1);
    let Poll::Ready(Ok(actual)) = poll(&mut future) else {
        panic!("releasing the object alias must preserve actual promise settlement")
    };
    assert_eq!(actual.as_object().unwrap(), value);

    let returned = attempted.expect("ordinary alias contention must return through the fallible registration API without a host panic");
    let Err(error) = returned else {
        panic!("mutable alias contention must refuse registration")
    };
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed)),
        "contention must preserve its uncatchable typed host cause"
    );
}

#[test]
fn native_poll_under_a_mutable_promise_alias_returns_a_host_error_without_losing_settlement() {
    let executor = Rc::new(ManualExecutor::default());
    let mut owner = context(executor.clone());
    let value = JsObject::with_null_proto(owner.runtime()).unwrap();
    let supplied = value.clone();
    let factories = Rc::new(Cell::new(0));
    let made = factories.clone();
    let promise = JsPromise::from_task(
        move |_| {
            made.set(made.get() + 1);
            Ok(ContextTask::from_future(std::future::ready(Ok(
                supplied.into()
            ))))
        },
        owner.access(),
    )
    .unwrap();
    let mut job = executor.take_async();
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    let alias = Deref::deref(&promise).clone();
    let guard = alias.borrow_mut().unwrap();
    let attempted = catch_unwind(AssertUnwindSafe(|| poll(&mut future)));
    assert_eq!(factories.get(), 0);
    drop(guard);

    assert!(poll(&mut future).is_pending());
    finish(&mut job, &mut owner);
    assert_eq!(factories.get(), 1);
    let Poll::Ready(Ok(actual)) = poll(&mut future) else {
        panic!(
            "the same native future must retain its original promise after transient alias contention"
        )
    };
    assert_eq!(actual.as_object().unwrap(), value);

    let returned = attempted
        .expect("ordinary alias contention must return from native poll without a host panic");
    let Poll::Ready(Err(error)) = returned else {
        panic!("an inaccessible current promise state must return its host contention error")
    };
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed)),
        "contention must not be fabricated as guest rejection or Pending"
    );
}
