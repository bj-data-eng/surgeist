use std::{
    cell::{Cell, RefCell},
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::{Duration, Instant},
};
use surgeist_js::{
    Context, ContextAccess, JsObject, JsResult, JsValue, NativeFunction,
    builtins::promise::PromiseState,
    context::HostHooks,
    error::EngineError,
    gc::heap::Trace,
    job::{Job, JobCallback, JobExecutor, JobRetirement},
    native_function::NativeCapture,
    object::builtins::{JsFunction, JsPromise},
};

#[derive(Debug)]
struct PublicationUnwind;
#[derive(Debug)]
struct TailCaptureUnwind;
#[derive(Default)]
struct Captures {
    armed: Cell<bool>,
    throw_once: Cell<bool>,
    drops: Cell<usize>,
}
struct LiveCapture {
    shared: Rc<Captures>,
    restored: bool,
}
#[derive(Trace)]
struct StoredCapture {
    #[heap_trace(skip)]
    shared: Rc<Captures>,
}
impl NativeCapture for LiveCapture {
    type Stored = StoredCapture;
    fn store(&self) -> StoredCapture {
        StoredCapture {
            shared: self.shared.clone(),
        }
    }
    fn restore(stored: &StoredCapture) -> Self {
        Self {
            shared: stored.shared.clone(),
            restored: true,
        }
    }
}
impl Drop for LiveCapture {
    fn drop(&mut self) {
        if self.restored && self.shared.armed.get() {
            self.shared.drops.set(self.shared.drops.get() + 1);
            if self.shared.throw_once.replace(false) {
                panic_any(TailCaptureUnwind);
            }
        }
    }
}
struct Hooks(Rc<Captures>);
impl HostHooks for Hooks {
    fn make_job_callback(&self, callback: JsFunction, _: ContextAccess<'_>) -> JobCallback {
        JobCallback::new(
            callback,
            LiveCapture {
                shared: self.0.clone(),
                restored: false,
            },
        )
    }
}
#[derive(Default)]
struct RetainThenUnwind {
    jobs: RefCell<Vec<Job>>,
    attempts: Cell<usize>,
}
impl JobExecutor for RetainThenUnwind {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        _: ContextAccess<'_>,
    ) -> Result<surgeist_js::job::JobAcceptance, surgeist_js::job::JobPublicationError> {
        self.attempts.set(self.attempts.get() + 1);
        // This already accepted job must survive the executor's native unwind.
        // The engine still owns every unattempted prepared reaction tail.
        self.jobs.borrow_mut().push(job);
        panic_any(PublicationUnwind);
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the probe does not dispatch jobs")
    }
}
#[derive(Default)]
struct Count {
    value: Mutex<usize>,
    changed: Condvar,
}
impl Count {
    fn record(&self) {
        *self.value.lock().unwrap() += 1;
        self.changed.notify_all();
    }
    fn await_fresh(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut value = self.value.lock().unwrap();
        while *value == 0 {
            let left = deadline.saturating_duration_since(Instant::now());
            assert!(
                !left.is_zero(),
                "native tail retirement was not notified; last count {value}"
            );
            value = self.changed.wait_timeout(value, left).unwrap().0;
        }
    }
}
impl Wake for Count {
    fn wake(self: Arc<Self>) {
        self.record();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.record();
    }
}

#[test]
fn publication_unwind_finishes_each_unattempted_tail_and_preserves_the_initiating_payload() {
    let captures = Rc::new(Captures::default());
    let executor = Rc::new(RetainThenUnwind::default());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .host_hooks(Rc::new(Hooks(captures.clone())));
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let (source, resolver) = JsPromise::new_pending(context.access());
    // Native observation installs the source association before the authored
    // reactions, so each result promise has its actual job-retirement binding.
    let _source_future = source.clone().into_js_future(context.access()).unwrap();
    let mut tails = Vec::new();
    for _ in 0..3 {
        let callback = NativeFunction::from_fn_ptr(|_, args, _| Ok(args[0].clone()))
            .to_js_function(context.realm())
            .unwrap();
        let tail = source.then(Some(callback), None, context.access()).unwrap();
        let mut future = tail.clone().into_js_future(context.access()).unwrap();
        let count = Arc::new(Count::default());
        assert!(
            Pin::new(&mut future)
                .poll(&mut TaskContext::from_waker(&Waker::from(count.clone())))
                .is_pending()
        );
        tails.push((tail, future, count));
    }
    let value = JsObject::with_null_proto(context.runtime());
    captures.armed.set(true);
    captures.throw_once.set(true);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        resolver
            .resolve
            .call(
                &JsValue::undefined(),
                &[value.clone().into()],
                context.access(),
            )
            .unwrap();
    }))
    .unwrap_err();
    // Before correction an independently owned tail capture can panic during
    // the initiating executor unwind, aborting the child before this boundary.
    assert!(unwind.is::<PublicationUnwind>());
    assert_eq!(
        executor.attempts.get(),
        1,
        "ordinary publication must stop at native unwind"
    );
    assert_eq!(
        executor.jobs.borrow().len(),
        1,
        "accepted custody must survive"
    );
    assert_eq!(
        captures.drops.get(),
        3,
        "all three unattempted restored callback captures must be disposed"
    );
    assert_eq!(
        source.state().unwrap(),
        PromiseState::Fulfilled(value.into())
    );
    for (tail, mut future, count) in tails {
        count.await_fresh();
        let Poll::Ready(Err(error)) =
            Pin::new(&mut future).poll(&mut TaskContext::from_waker(Waker::noop()))
        else {
            panic!("unpublished reaction tail must terminate native observation")
        };
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::JobRetired(JobRetirement::Dropped))
        );
        assert_eq!(tail.state().unwrap(), PromiseState::Pending);
    }
    captures.armed.set(false);
    drop(executor.jobs.borrow_mut().drain(..).collect::<Vec<_>>());
}
