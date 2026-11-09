use std::{
    cell::{Cell, RefCell},
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsResult, JsValue, NativeFunction,
    context::HostHooks,
    error::EngineError,
    job::{Job, JobAcceptance, JobCallback, JobExecutor, JobPublicationError, JobRetirement},
    native_function::NativeCapture,
    object::builtins::{JsFunction, JsPromise},
};
use surgeist_js_gc::heap::{ObjectId, Trace};

#[derive(Clone, Copy, Eq, PartialEq)]
enum Role {
    Earlier,
    Failing,
    Tail,
    Neutral,
}
impl Role {
    fn index(self) -> usize {
        self as usize
    }
}
#[derive(Debug, Eq, PartialEq)]
enum HostUnwind {
    OriginalRestore,
    SecondaryDrop,
}
#[derive(Default)]
struct Monitor {
    roles: RefCell<Vec<(ObjectId, Role)>>,
    armed: Cell<bool>,
    primary: Cell<bool>,
    secondary: Cell<bool>,
    restores: [Cell<usize>; 4],
    live: [Cell<usize>; 4],
    stored: [Cell<usize>; 4],
    restored_drops: [Cell<usize>; 4],
    callback_calls: Cell<usize>,
    publications: Cell<usize>,
}
struct Capture {
    monitor: Rc<Monitor>,
    role: Role,
    restored: bool,
}
impl Capture {
    fn new(monitor: Rc<Monitor>, role: Role, restored: bool) -> Self {
        let count = &monitor.live[role.index()];
        count.set(count.get() + 1);
        Self {
            monitor,
            role,
            restored,
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        let count = &self.monitor.live[self.role.index()];
        count.set(
            count
                .get()
                .checked_sub(1)
                .expect("one actual live capture is being disposed"),
        );
        if self.restored && self.monitor.armed.get() {
            let count = &self.monitor.restored_drops[self.role.index()];
            count.set(count.get() + 1);
            if self.role == Role::Earlier && !self.monitor.secondary.replace(true) {
                panic_any(HostUnwind::SecondaryDrop);
            }
        }
    }
}
#[derive(Trace)]
struct StoredCapture {
    #[heap_trace(skip)]
    monitor: Rc<Monitor>,
    #[heap_trace(skip)]
    role: Role,
}
impl Drop for StoredCapture {
    fn drop(&mut self) {
        let count = &self.monitor.stored[self.role.index()];
        count.set(
            count
                .get()
                .checked_sub(1)
                .expect("one actual stored host capture is being disposed"),
        );
    }
}
impl NativeCapture for Capture {
    type Stored = StoredCapture;
    fn store(&self) -> Self::Stored {
        let count = &self.monitor.stored[self.role.index()];
        count.set(count.get() + 1);
        StoredCapture {
            monitor: self.monitor.clone(),
            role: self.role,
        }
    }
    fn restore(stored: &Self::Stored) -> Result<Self, surgeist_js::error::RootingError> {
        if stored.monitor.armed.get() {
            let count = &stored.monitor.restores[stored.role.index()];
            count.set(count.get() + 1);
            if stored.role == Role::Failing {
                stored.monitor.primary.set(true);
                panic_any(HostUnwind::OriginalRestore);
            }
        }
        Ok(Capture::new(
            stored.monitor.clone(),
            stored.role,
            stored.monitor.armed.get(),
        ))
    }
}
struct Hooks(Rc<Monitor>);
impl HostHooks for Hooks {
    fn make_job_callback(&self, callback: JsFunction, _: ContextAccess<'_>) -> JobCallback {
        let role = self
            .0
            .roles
            .borrow()
            .iter()
            .find(|(id, _)| *id == callback.identity())
            .map_or(Role::Neutral, |(_, role)| *role);
        JobCallback::new(callback, Capture::new(self.0.clone(), role, false))
    }
    fn call_job_callback(
        &self,
        job: &JobCallback,
        this: &JsValue,
        args: &[JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        self.0.callback_calls.set(self.0.callback_calls.get() + 1);
        job.callback().call(this, args, context)
    }
}
struct Executor(Rc<Monitor>);
impl JobExecutor for Executor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.publications.set(self.0.publications.get() + 1);
        // Reaching publication is already a failure for this preparation probe.
        // Dispose the actual input using explicit policy, never execute it.
        drop(job);
        Ok(JobAcceptance::Discarded)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the restore-failure probe never dispatches guest work")
    }
}
fn restore_unwind(reject: bool) {
    let monitor = Rc::new(Monitor::default());
    let executor = Rc::new(Executor(monitor.clone()));
    let hooks = Rc::new(Hooks(monitor.clone()));
    let builder = Context::builder().job_executor(executor).host_hooks(hooks);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let runtime = context.runtime().clone();
    let (source, resolvers) = JsPromise::new_pending(context.access());
    let mut source_wait = source.clone().into_js_future(context.access()).unwrap();
    assert!(
        Pin::new(&mut source_wait)
            .poll(&mut TaskContext::from_waker(Waker::noop()))
            .is_pending()
    );
    let mut handlers = Vec::new();
    let mut targets = Vec::new();
    for role in [Role::Earlier, Role::Failing, Role::Tail] {
        let callback = NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined()))
            .to_js_function(context.realm())
            .unwrap();
        monitor.roles.borrow_mut().push((callback.identity(), role));
        let target = if reject {
            source
                .then(None, Some(callback.clone()), context.access())
                .unwrap()
        } else {
            source
                .then(Some(callback.clone()), None, context.access())
                .unwrap()
        };
        handlers.push(callback);
        targets.push(target);
    }
    assert_eq!(monitor.publications.get(), 0);
    assert_eq!(monitor.callback_calls.get(), 0);
    monitor.armed.set(true);
    let actual = catch_unwind(AssertUnwindSafe(|| {
        let resolver = if reject {
            &resolvers.reject
        } else {
            &resolvers.resolve
        };
        resolver.call(&JsValue::undefined(), &[JsValue::new(42)], context.access())
    }))
    .expect_err("authored public NativeCapture restoration must propagate its Rust unwind");
    // The prefix's independent final-drop unwind is secondary. The conversions
    // only touch outside-record Rust counters; no engine reentry occurs.
    assert_eq!(
        actual.downcast_ref::<HostUnwind>(),
        Some(&HostUnwind::OriginalRestore)
    );
    assert!(monitor.primary.get());
    assert!(monitor.secondary.get());
    assert_eq!(monitor.restores[Role::Earlier.index()].get(), 1);
    assert_eq!(monitor.restores[Role::Failing.index()].get(), 1);
    assert_eq!(monitor.restores[Role::Tail.index()].get(), 0);
    assert_eq!(monitor.restored_drops[Role::Earlier.index()].get(), 1);
    assert_eq!(monitor.publications.get(), 0);
    assert_eq!(monitor.callback_calls.get(), 0);
    monitor.armed.set(false);
    let Poll::Ready(Err(error)) =
        Pin::new(&mut source_wait).poll(&mut TaskContext::from_waker(Waker::noop()))
    else {
        panic!("consumed preparation must not abandon the original native source wait Pending")
    };
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::Interrupted))
    );
    // No consumed resolver retry or invented guest settlement. Drop every real
    // public owner, then collect the actual heap-stored remaining tails.
    drop(actual);
    drop(source_wait);
    drop(targets);
    drop(handlers);
    drop(resolvers);
    drop(source);
    drop(context);
    runtime.collect().unwrap();
    for role in [Role::Earlier, Role::Failing, Role::Tail, Role::Neutral] {
        assert_eq!(monitor.live[role.index()].get(), 0);
        assert_eq!(monitor.stored[role.index()].get(), 0);
    }
    assert_eq!(monitor.publications.get(), 0);
    assert_eq!(monitor.callback_calls.get(), 0);
}
#[test]
fn fulfillment_restore_unwind_preserves_the_original_despite_an_earlier_restored_capture_drop_unwind()
 {
    restore_unwind(false);
}
#[test]
fn rejection_restore_unwind_preserves_the_original_despite_an_earlier_restored_capture_drop_unwind()
{
    restore_unwind(true);
}
