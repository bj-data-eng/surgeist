use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    ops::Deref,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsObject, JsResult, JsValue,
    builtins::promise::PromiseState,
    error::{
        DiagnosticEngine, DiagnosticKind, EngineError, ErasedEngineError, ObjectAccessError,
        PanicError,
    },
    execution::ExecutionOptions,
    job::{
        ContextTask, Job, JobAcceptance, JobExecutor, JobPublicationError, JobStatus, JobStep,
        NativeAsyncJob,
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
        panic!("dispatch uses the actual captured job explicitly")
    }
}
impl ManualExecutor {
    fn take_async(&self) -> NativeAsyncJob {
        let job = self.0.borrow_mut().pop_front();
        let Some(Job::AsyncJob(job)) = job else {
            panic!("the authored promise owns a native async wrapper")
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
fn busy(error: &JsError) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::ObjectAccess(ObjectAccessError::Borrowed))
    );
}
fn complete(job: &mut NativeAsyncJob, owner: &mut Context) {
    let host = owner.host_access();
    assert!(matches!(
        job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host),
        JobStep::Prepared
    ));
    let JobStep::Complete(result) =
        job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host)
    else {
        panic!("the released promise must accept its real resolver stage")
    };
    result.unwrap();
}
fn representations(error: &JsError, owner: &mut Context) {
    busy(error);
    assert_eq!(
        error.clone().into_opaque(owner.access()).unwrap_err(),
        *error
    );
    let outer = PanicError::new("authored promise alias contention").with_source(error.clone());
    assert!(matches!(
        outer.diagnostic().unwrap().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::ObjectAccess(ObjectAccessError::Borrowed))
    ));
    assert_eq!(
        error.clone().into_erased(owner.access()).as_engine(),
        Some(&ErasedEngineError::ObjectAccess(
            ObjectAccessError::Borrowed
        ))
    );
}

#[test]
fn a_mutable_promise_alias_refuses_future_registration_without_faulting_or_consuming_the_job() {
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
    let observer = job.observer();
    let alias = Deref::deref(&promise).clone();
    let guard = alias.borrow_mut().unwrap();
    let refused = owner
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(0)
                .with_job_invocation_limit(0)
                .with_job_poll_limit(0),
            |access| promise.clone().into_js_future(access),
        )
        .unwrap();
    let error = refused.result.unwrap_err();
    busy(&error);
    busy(&promise.state().unwrap_err());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(factories.get(), 0);
    assert_eq!(refused.report.instructions().unwrap(), 0);
    assert_eq!(refused.report.job_preparations().unwrap(), 0);
    assert_eq!(refused.report.job_invocations().unwrap(), 0);
    assert_eq!(refused.report.job_polls().unwrap(), 0);
    assert!(!refused.report.effects_started());
    drop(guard);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    representations(&error, &mut owner);
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    complete(&mut job, &mut owner);
    assert_eq!(factories.get(), 1);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(value.clone().into())
    );
    let Poll::Ready(Ok(actual)) = poll(&mut future) else {
        panic!("registration after releasing the alias remains viable")
    };
    assert_eq!(actual.as_object().unwrap(), value);
}

#[test]
fn polling_under_a_mutable_promise_alias_returns_busy_and_recovers_the_original_settlement() {
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
    let observer = job.observer();
    let mut future = promise.clone().into_js_future(owner.access()).unwrap();
    assert!(poll(&mut future).is_pending());
    let alias = Deref::deref(&promise).clone();
    let guard = alias.borrow_mut().unwrap();
    let Poll::Ready(Err(error)) = poll(&mut future) else {
        panic!("the actual shared mutable object lease must return its contention error")
    };
    busy(&error);
    busy(&promise.state().unwrap_err());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(factories.get(), 0);
    drop(guard);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    representations(&error, &mut owner);
    assert!(poll(&mut future).is_pending());
    complete(&mut job, &mut owner);
    assert_eq!(factories.get(), 1);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(value.clone().into())
    );
    let Poll::Ready(Ok(actual)) = poll(&mut future) else {
        panic!("transient object contention cannot retire or replace the native observation")
    };
    assert_eq!(actual.as_object().unwrap(), value);
}
