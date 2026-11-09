#![cfg(feature = "fetch")]
#![allow(unused_crate_dependencies)]

use futures_lite::future;
use std::{cell::Cell, rc::Rc, sync::atomic::Ordering, task::Poll};

use surgeist_js::{
    Context, ContextAccess, JsData, JsNativeError, JsObject, JsResult, JsValue, Source,
    builtins::promise::PromiseState,
    error::EngineError,
    gc::heap::Trace as HeapTrace,
    job::{ContextTask, JobExecutor, JobRetirement, SimpleJobExecutor},
    object::builtins::JsPromise,
};
use surgeist_js_test_host::fetch::{self, Fetcher, request::JsRequest, response::JsResponse};

#[derive(Debug, Clone, Copy)]
enum ResolveFailure {
    Busy,
    Catchable,
}

#[derive(Debug, JsData, HeapTrace)]
struct FailingResolver {
    #[heap_trace(skip)]
    failure: ResolveFailure,
    #[heap_trace(skip)]
    resolve_invoked: Rc<Cell<bool>>,
    #[heap_trace(skip)]
    fetch_invoked: Rc<Cell<bool>>,
}

impl Fetcher for FailingResolver {
    fn resolve_uri(&self, _: String, access: ContextAccess<'_>) -> JsResult<String> {
        self.resolve_invoked.set(true);
        match self.failure {
            ResolveFailure::Busy => {
                let host = access.into_host_access();
                // Compete for an actual active lease rather than synthesizing
                // the error that the fetch adapter is expected to preserve.
                host.with(|_| host.with(|_| Ok(String::new())))
            }
            ResolveFailure::Catchable => Err(JsNativeError::typ()
                .with_message("ordinary URI resolution failure")
                .into()),
        }
    }

    fn fetch(self: Rc<Self>, _: JsRequest, _: Option<JsObject>) -> ContextTask<JsResponse> {
        ContextTask::from_future(async move {
            self.fetch_invoked.set(true);
            Err(JsNativeError::error()
                .with_message("backend must not run after URI resolution fails")
                .into())
        })
    }
}

struct FetchCase {
    context: Context,
    promise: JsPromise,
    resolve_invoked: Rc<Cell<bool>>,
    fetch_invoked: Rc<Cell<bool>>,
}

fn schedule_fetch(failure: ResolveFailure) -> FetchCase {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap();
    let resolve_invoked = Rc::new(Cell::new(false));
    let fetch_invoked = Rc::new(Cell::new(false));
    fetch::register(
        FailingResolver {
            failure,
            resolve_invoked: resolve_invoked.clone(),
            fetch_invoked: fetch_invoked.clone(),
        },
        None,
        &mut context,
    )
    .unwrap();
    let value = context
        .eval(Source::from_bytes(
            "globalThis.caught = false; globalThis.rejectionMessage = '';
             globalThis.fetchPromise = fetch('http://unit.test/resource');
             fetchPromise.catch(error => {
                 caught = true;
                 rejectionMessage = error.message;
             });
             fetchPromise",
        ))
        .unwrap();
    let promise = JsPromise::from_object(value.as_object().unwrap()).unwrap();
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert!(!resolve_invoked.get());
    assert!(!fetch_invoked.get());
    FetchCase {
        context,
        promise,
        resolve_invoked,
        fetch_invoked,
    }
}

#[test]
fn fetch_uri_resolution_busy_reaches_the_driver_without_guest_rejection() {
    let mut case = schedule_fetch(ResolveFailure::Busy);
    let drained = case.context.run_jobs();
    assert!(case.resolve_invoked.get());
    assert!(!case.fetch_invoked.get());
    let error = drained.expect_err("a real URI resolver lease conflict must reach the host driver");
    assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
    assert_eq!(case.promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        case.context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::new(false)
    );
    case.context.run_jobs().unwrap();
    assert_eq!(case.promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        case.context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn fetch_uri_resolution_catchable_error_rejects_the_original_guest_promise() {
    let mut case = schedule_fetch(ResolveFailure::Catchable);
    case.context.run_jobs().unwrap();
    assert!(case.resolve_invoked.get());
    assert!(!case.fetch_invoked.get());
    assert!(matches!(
        case.promise.state().unwrap(),
        PromiseState::Rejected(_)
    ));
    assert_eq!(
        case.context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        case.context
            .eval(Source::from_bytes(
                "rejectionMessage.includes('ordinary URI resolution failure')",
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

struct FetchDrop(Rc<Cell<u32>>);

impl Drop for FetchDrop {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[derive(Debug, JsData, HeapTrace)]
struct PendingFetcher {
    #[heap_trace(skip)]
    polls: Rc<Cell<u32>>,
    #[heap_trace(skip)]
    drops: Rc<Cell<u32>>,
}

impl Fetcher for PendingFetcher {
    fn fetch(self: Rc<Self>, _: JsRequest, _: Option<JsObject>) -> ContextTask<JsResponse> {
        let probe = FetchDrop(self.drops.clone());
        ContextTask::from_poll(move |_, _| {
            let _retained = &probe;
            self.polls.set(self.polls.get() + 1);
            Poll::Pending
        })
    }
}

struct PendingFetchCase {
    context: Context,
    executor: Rc<SimpleJobExecutor>,
    promise: JsPromise,
    polls: Rc<Cell<u32>>,
    drops: Rc<Cell<u32>>,
}

fn pending_fetch() -> PendingFetchCase {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let polls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    fetch::register(
        PendingFetcher {
            polls: polls.clone(),
            drops: drops.clone(),
        },
        None,
        &mut context,
    )
    .unwrap();
    let value = context
        .eval(Source::from_bytes(
            "globalThis.caught = false;
             const response = fetch('http://unit.test/resource');
             response.catch(() => { caught = true; }); response",
        ))
        .unwrap();
    PendingFetchCase {
        context,
        executor,
        promise: JsPromise::from_object(value.as_object().unwrap()).unwrap(),
        polls,
        drops,
    }
}

fn poll_pending_fetch(case: &mut PendingFetchCase) {
    let host = case.context.host_access();
    assert!(
        future::block_on(future::poll_once(
            case.executor.clone().run_jobs_async(&host)
        ))
        .is_none()
    );
    assert_eq!(case.polls.get(), 1);
    assert_eq!(case.drops.get(), 0);
}

#[test]
fn cancelled_pending_fetch_notifies_its_native_waiter_without_guest_rejection() {
    let mut case = pending_fetch();
    let mut native = Box::pin(
        case.promise
            .clone()
            .into_js_future(case.context.access())
            .unwrap(),
    );
    assert!(future::block_on(future::poll_once(native.as_mut())).is_none());
    poll_pending_fetch(&mut case);
    assert!(future::block_on(future::poll_once(native.as_mut())).is_none());
    case.executor
        .get_cancellation_token()
        .store(true, Ordering::Relaxed);
    case.context.run_jobs().unwrap();
    let error = future::block_on(future::poll_once(native.as_mut()))
        .expect("native fetch wait must observe cancellation")
        .unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::Cancelled))
    );
    assert_eq!(case.polls.get(), 1);
    assert_eq!(case.drops.get(), 1);
    assert_eq!(case.promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        case.context.eval(Source::from_bytes("caught")).unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn pending_fetch_owner_loss_notifies_its_native_waiter_without_guest_rejection() {
    let mut case = pending_fetch();
    let mut native = Box::pin(
        case.promise
            .clone()
            .into_js_future(case.context.access())
            .unwrap(),
    );
    assert!(future::block_on(future::poll_once(native.as_mut())).is_none());
    poll_pending_fetch(&mut case);
    drop(case.context);
    let error = future::block_on(future::poll_once(native.as_mut()))
        .expect("native fetch wait must observe its owner's loss")
        .unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::OwnerDropped))
    );
    assert_eq!(case.polls.get(), 1);
    assert_eq!(case.drops.get(), 1);
    assert_eq!(case.promise.state().unwrap(), PromiseState::Pending);
}
