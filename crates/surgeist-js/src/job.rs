//! Boa's API to create and customize `ECMAScript` jobs and job queues.
//!
//! [`Job`] is an ECMAScript [Job], or a closure that runs an `ECMAScript` computation when
//! there's no other computation running. The module defines several type of jobs:
//! - [`PromiseJob`] for Promise related jobs.
//! - [`TimeoutJob`] for jobs that run after a certain amount of time.
//! - [`NativeAsyncJob`] for jobs that support [`Future`].
//! - [`NativeJob`] for generic jobs that aren't related to Promises.
//!
//! [`JobCallback`] is an ECMAScript [`JobCallback`] record, containing an `ECMAScript` function
//! that is executed when a promise is either fulfilled or rejected.
//!
//! [`JobExecutor`] is a trait encompassing the required functionality for a job executor; this allows
//! implementing custom event loops, custom handling of Jobs or other fun things.
//! This trait is also accompanied by two implementors of the trait:
//! - [`IdleJobExecutor`], which is an executor that does nothing, and the default executor if no executor is
//!   provided. Useful for hosts that want to disable promises.
//! - [`SimpleJobExecutor`], which is a simple FIFO queue that runs all jobs to completion, bailing
//!   on the first error encountered. This simple executor will block on any async job queued.
//!
//! Queued jobs own the roots of their captured values. Callback records stored on the runtime heap
//! instead report their edges through the stored form.
//!
//! [Job]: https://tc39.es/ecma262/#sec-jobs
//! [JobCallback]: https://tc39.es/ecma262/#sec-jobcallback-records

use crate::context::time::{JsDuration, JsInstant};
use crate::sys::time;
use crate::{
    Context, JsNativeError, JsResult, JsValue,
    native_function::NativeCapture,
    object::{JsFunction, JsObject, ObjectEdge},
    realm::Realm,
};
use futures_concurrency::future::FutureGroup;
use futures_lite::{StreamExt, future};
use portable_atomic::AtomicBool;
use std::any::Any;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::{cell::RefCell, collections::VecDeque, fmt::Debug, future::Future, pin::Pin};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, ObjectId, Trace as HeapTrace};

pub use crate::builtins::finalization_registry::FinalizationRegistryCleanupJob;

/// An ECMAScript [Job Abstract Closure].
///
/// This is basically a synchronous task that needs to be run to progress [`Promise`] objects,
/// or unblock threads waiting on [`Atomics.waitAsync`].
///
/// [Job Abstract Closure]: https://tc39.es/ecma262/#sec-jobs
/// [`Promise`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
/// [`Atomics.waitAsync`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/waitAsync
pub struct NativeJob {
    #[allow(clippy::type_complexity)]
    f: Box<dyn FnOnce(&mut Context) -> JsResult<JsValue>>,
    realm: Option<Realm>,
}

/// Owns invocation cleanup and the optional realm boundary of native jobs.
fn invoke_native_job(
    context: &mut Context,
    realm: Option<Realm>,
    call: impl FnOnce(&mut Context) -> JsResult<JsValue>,
) -> JsResult<JsValue> {
    context.check_execution_stop()?;
    let checkpoint = context.vm.call_checkpoint();
    let previous_realm = realm.map(|realm| context.enter_realm(realm)).transpose()?;
    let mut active = context.guard(move |context| {
        // An unwinding callback may leave a child frame on top. Release its
        // invocation state before restoring the actual entry caller's realm.
        context.vm.restore_call(checkpoint);
        if let Some(mut realm) = previous_realm {
            // Entry has already validated that both realms belong to this runtime.
            context.swap_realm(&mut realm);
        }
    });
    active.note_execution_effect();
    let result = call(&mut active);
    // A host callback can swallow an inner error. Its accepted effects remain,
    // but neither success nor another error can replace the retained host stop.
    active.check_execution_stop()?;
    result
}

impl Debug for NativeJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeJob").finish_non_exhaustive()
    }
}

impl NativeJob {
    /// Creates a new `NativeJob` from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: None,
        }
    }

    /// Creates a new `NativeJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: Some(realm),
        }
    }

    /// Gets a reference to the execution realm of the job.
    #[must_use]
    pub const fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }

    /// Calls the native job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    /// A retained operation stop prevents invocation and takes precedence over the
    /// callback's result. Invocation-owned VM state is released on return or unwind.
    /// An explicit execution realm is restored after that cleanup; a job without
    /// one preserves a callback's deliberate realm change.
    pub fn call(self, context: &mut Context) -> JsResult<JsValue> {
        invoke_native_job(context, self.realm, self.f)
    }
}

/// An ECMAScript [Job Abstract Closure] that can be called multiple times.
///
/// This is basically a synchronous task that needs to be run to progress [`Promise`] objects,
/// or unblock threads waiting on [`Atomics.waitAsync`].
///
/// [Job Abstract Closure]: https://tc39.es/ecma262/#sec-jobs
/// [`Promise`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
/// [`Atomics.waitAsync`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/waitAsync
pub struct NativeJobFn {
    #[allow(clippy::type_complexity)]
    f: Box<dyn Fn(&mut Context) -> JsResult<JsValue>>,
    realm: Option<Realm>,
}

impl Debug for NativeJobFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeJobFn").finish_non_exhaustive()
    }
}

impl NativeJobFn {
    /// Creates a new `NativeJobFn` from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: None,
        }
    }

    /// Creates a new `NativeJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: Fn(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: Some(realm),
        }
    }

    /// Gets a reference to the execution realm of the job.
    #[must_use]
    pub const fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }

    /// Calls the native job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    /// A retained operation stop prevents invocation and takes precedence over the
    /// callback's result. Invocation-owned VM state is released on return or unwind.
    /// An explicit execution realm is restored after that cleanup; a job without
    /// one preserves a callback's deliberate realm change.
    pub fn call(&self, context: &mut Context) -> JsResult<JsValue> {
        invoke_native_job(context, self.realm.clone(), &self.f)
    }
}

type Callback = Box<dyn FnOnce(&mut Context)>;

/// Token to cancel a [`TimeoutJob`] and [`IntervalJob`].
#[derive(Clone)]
pub struct CancellationToken(Rc<Cell<Vec<Callback>>>);

impl Debug for CancellationToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CancellationToken")
            .field("cancelled", &self.revoked())
            .finish_non_exhaustive()
    }
}

impl CancellationToken {
    /// Creates a new cancellation token.
    pub(crate) fn new() -> Self {
        Self(Rc::new(Cell::new(vec![Box::new(|_| {})])))
    }

    /// Sets a callback to run when the cancellation token gets used.
    ///
    /// On debug builds, this will panic if the cancellation token was already
    /// used.
    pub fn push_callback(&self, f: impl FnOnce(&mut Context) + 'static) {
        let mut vec = self.0.take();
        debug_assert!(
            !vec.is_empty(),
            "setting a callback on an already used cancellation token"
        );
        vec.push(Box::new(f));
        self.0.set(vec);
    }

    /// Cancels the [`TimeoutJob`] or [`IntervalJob`] associated with this cancellation token.
    pub fn cancel(&self, context: &mut Context) {
        for job in self.0.take() {
            job(context);
        }
    }

    /// Revokes this cancellation token, making it unusable to cancel its associated job.
    pub(crate) fn revoke(&self) {
        self.0.take();
    }

    /// Returns `true` if this cancellation token has been revoked, either because
    /// `cancel` was called or because its associated job has completed.
    #[must_use]
    pub fn revoked(&self) -> bool {
        let callbacks = self.0.take();
        let cancelled = callbacks.is_empty();
        self.0.set(callbacks);
        cancelled
    }
}

/// An ECMAScript [Job] that runs after a certain amount of time.
///
/// This represents the [HostEnqueueTimeoutJob] operation from the specification.
///
/// [HostEnqueueTimeoutJob]: https://tc39.es/ecma262/#sec-hostenqueuetimeoutjob
#[derive(Debug)]
pub struct TimeoutJob {
    /// The distance in milliseconds in the future when the job should run.
    /// This will be added to the current time when the job is enqueued.
    timeout: JsDuration,
    /// The job to run after the specified timeout.
    job: Option<NativeJob>,
    /// Signals if the timeout job was cancelled.
    cancellation_token: CancellationToken,
}

impl Drop for TimeoutJob {
    fn drop(&mut self) {
        self.cancellation_token.revoke();
    }
}

impl TimeoutJob {
    /// Create a new `TimeoutJob` with a timeout and a job.
    #[must_use]
    pub fn new(job: NativeJob, timeout_in_millis: u64) -> Self {
        Self {
            timeout: JsDuration::from_millis(timeout_in_millis),
            job: Some(job),
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Creates a new `TimeoutJob` from a closure and a timeout as [`std::time::Duration`].
    #[must_use]
    pub fn from_duration<F>(f: F, timeout: impl Into<JsDuration>) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self::new(NativeJob::new(f), timeout.into().as_millis())
    }

    /// Creates a new `TimeoutJob` from a closure, a timeout, and an execution realm.
    #[must_use]
    pub fn with_realm<F>(f: F, realm: Realm, timeout: time::Duration) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self::new(NativeJob::with_realm(f, realm), timeout.as_millis() as u64)
    }

    /// Calls the native job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    pub fn call(mut self, context: &mut Context) -> JsResult<JsValue> {
        let result = self
            .job
            .take()
            .map_or_else(|| Ok(JsValue::undefined()), |job| job.call(context));
        self.cancellation_token.revoke();
        result
    }

    /// Returns the timeout value in milliseconds since epoch.
    #[inline]
    #[must_use]
    pub fn timeout(&self) -> JsDuration {
        self.timeout
    }

    /// Returns `true` if the timeout was cancelled, and its execution can be skipped.
    #[inline]
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.cancellation_token.revoked()
    }

    /// Returns the [`CancellationToken`] for this timeout job.
    #[must_use]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

/// An ECMAScript [Job] that runs at a certain interval of time.
///
/// This represents jobs enqueued by APIs such as [`setInterval`].
///
/// [`setInterval`]: https://developer.mozilla.org/en-US/docs/Web/API/Window/setInterval
#[derive(Debug)]
pub struct IntervalJob {
    /// The distance in milliseconds in the future when the job should run.
    /// This will be added to the current time when the job is enqueued.
    interval: JsDuration,
    /// The job to run after every interval of time.
    job: NativeJobFn,
    /// Signals if the timeout job was cancelled.
    cancellation_token: CancellationToken,
}

impl Drop for IntervalJob {
    fn drop(&mut self) {
        self.cancellation_token.revoke();
    }
}

impl IntervalJob {
    /// Create a new `IntervalJob` with an interval and a job.
    #[must_use]
    pub fn new(job: NativeJobFn, interval_in_millis: u64) -> Self {
        Self {
            interval: JsDuration::from_millis(interval_in_millis),
            job,
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Creates a new `IntervalJob` from a closure and an interval as [`std::time::Duration`].
    #[must_use]
    pub fn from_duration<F>(f: F, interval: impl Into<JsDuration>) -> Self
    where
        F: Fn(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self::new(NativeJobFn::new(f), interval.into().as_millis())
    }

    /// Creates a new `TimeoutJob` from a closure, an interval, and an execution realm.
    #[must_use]
    pub fn with_realm<F>(f: F, realm: Realm, interval: time::Duration) -> Self
    where
        F: Fn(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self::new(
            NativeJobFn::with_realm(f, realm),
            interval.as_millis() as u64,
        )
    }

    /// Calls the interval job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the interval job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    pub fn call(&self, context: &mut Context) -> JsResult<JsValue> {
        self.job.call(context)
    }

    /// Returns the interval value in milliseconds.
    #[inline]
    #[must_use]
    pub fn interval(&self) -> JsDuration {
        self.interval
    }

    /// Returns `true` if the interval job was cancelled, and its execution can be skipped.
    #[inline]
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.cancellation_token.revoked()
    }

    /// Returns the [`CancellationToken`] for this interval job.
    #[must_use]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
}

/// An ECMAScript Generic [Job].
///
/// This represents the [HostEnqueueGenericJob] operation from the specification, which
/// enqueues a job that is just like a [`PromiseJob`], but unconstrained in relation
/// to priority and ordering.
///
/// [HostEnqueueGenericJob]: https://tc39.es/ecma262/#sec-hostenqueuegenericjob
pub struct GenericJob(NativeJob);

impl Debug for GenericJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericJob").finish_non_exhaustive()
    }
}

impl GenericJob {
    /// Creates a new `GenericJob` from a closure and an execution realm.
    pub fn new<F>(f: F, realm: Realm) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self(NativeJob::with_realm(f, realm))
    }

    /// Gets a reference to the execution realm of the job.
    #[must_use]
    pub const fn realm(&self) -> &Realm {
        self.0
            .realm
            .as_ref()
            .expect("all generic jobs must have an execution realm")
    }

    /// Calls the `GenericJob` with the specified [`Context`], setting the execution
    /// context to the job's realm before calling the inner closure, and resets it after execution.
    pub fn call(self, context: &mut Context) -> JsResult<JsValue> {
        self.0.call(context)
    }
}

/// The [`Future`] job returned by a [`NativeAsyncJob`] operation.
pub type BoxedFuture<'a> = Pin<Box<dyn Future<Output = JsResult<JsValue>> + 'a>>;

/// An ECMAScript [Job] that can be run asynchronously.
///
/// This is an additional type of job that is not defined by the specification, enabling running `Future` tasks
/// created by ECMAScript code in an easier way.
#[allow(clippy::type_complexity)]
pub struct NativeAsyncJob {
    f: Box<dyn for<'a> FnOnce(&'a RefCell<&mut Context>) -> BoxedFuture<'a>>,
    realm: Option<Realm>,
}

impl Debug for NativeAsyncJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeAsyncJob")
            .field("f", &"Closure")
            .finish()
    }
}

impl NativeAsyncJob {
    /// Creates a new `NativeAsyncJob` from an async closure.
    pub fn new<F>(f: F) -> Self
    where
        F: AsyncFnOnce(&RefCell<&mut Context>) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(move |ctx| Box::pin(async move { f(ctx).await })),
            realm: None,
        }
    }

    /// Creates a new `NativeAsyncJob` from an async closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: AsyncFnOnce(&RefCell<&mut Context>) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(move |ctx| Box::pin(async move { f(ctx).await })),
            realm: Some(realm),
        }
    }

    /// Gets a reference to the execution realm of the job.
    #[must_use]
    pub const fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }

    /// Calls the native async job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native async job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    pub fn call<'a, 'b>(
        self,
        context: &'a RefCell<&'b mut Context>,
        // We can make our users assume `Unpin` because `self.f` is already boxed, so we shouldn't
        // need pin at all.
    ) -> impl Future<Output = JsResult<JsValue>> + Unpin + use<'a, 'b> {
        // If realm is not null, each time job is invoked the implementation must perform
        // implementation-defined steps such that execution is prepared to evaluate ECMAScript
        // code at the time of job's invocation.
        let realm = self.realm;

        let mut future = if let Some(realm) = &realm {
            let entered = context.borrow_mut().enter_realm(realm.clone());
            entered.and_then(|old_realm| {
                // Let scriptOrModule be GetActiveScriptOrModule() at the time HostEnqueuePromiseJob is
                // invoked. If realm is not null, each time job is invoked the implementation must
                // perform implementation-defined steps such that scriptOrModule is the active script or
                // module at the time of job's invocation.
                let result = (self.f)(context);

                context.borrow_mut().enter_realm(old_realm)?;
                Ok(result)
            })
        } else {
            Ok((self.f)(context))
        };

        std::future::poll_fn(move |cx| {
            let future = match &mut future {
                Ok(future) => future,
                Err(error) => return Poll::Ready(Err(error.clone())),
            };
            if let Some(realm) = &realm {
                let old_realm = context.borrow_mut().enter_realm(realm.clone())?;
                let result = future.as_mut().poll(cx);
                context.borrow_mut().enter_realm(old_realm)?;
                result
            } else {
                future.as_mut().poll(cx)
            }
        })
    }
}

/// An ECMAScript [Job Abstract Closure] executing code related to [`Promise`] objects.
///
/// This represents the [`HostEnqueuePromiseJob`] operation from the specification.
///
/// ### [Requirements]
///
/// - If realm is not null, each time job is invoked the implementation must perform implementation-defined
///   steps such that execution is prepared to evaluate ECMAScript code at the time of job's invocation.
/// - Let `scriptOrModule` be [`GetActiveScriptOrModule()`] at the time `HostEnqueuePromiseJob` is invoked.
///   If realm is not null, each time job is invoked the implementation must perform implementation-defined steps
///   such that `scriptOrModule` is the active script or module at the time of job's invocation.
/// - Jobs must run in the same order as the `HostEnqueuePromiseJob` invocations that scheduled them.
///
/// Of all the requirements, Boa guarantees the first two by its internal implementation of `NativeJob`, meaning
/// implementations of [`JobExecutor`] must only guarantee that jobs are run in the same order as they're enqueued.
///
/// [`Promise`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
/// [`HostEnqueuePromiseJob`]: https://tc39.es/ecma262/#sec-hostenqueuepromisejob
/// [Job Abstract Closure]: https://tc39.es/ecma262/#sec-jobs
/// [Requirements]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-hostenqueuepromisejob
/// [`GetActiveScriptOrModule()`]: https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-getactivescriptormodule
pub struct PromiseJob(NativeJob);

impl Debug for PromiseJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromiseJob").finish_non_exhaustive()
    }
}

impl PromiseJob {
    /// Creates a new `PromiseJob` from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self(NativeJob::new(f))
    }

    /// Creates a new `PromiseJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: FnOnce(&mut Context) -> JsResult<JsValue> + 'static,
    {
        Self(NativeJob::with_realm(f, realm))
    }

    /// Gets a reference to the execution realm of the `PromiseJob`.
    #[must_use]
    pub const fn realm(&self) -> Option<&Realm> {
        self.0.realm()
    }

    /// Calls the `PromiseJob` with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    pub fn call(self, context: &mut Context) -> JsResult<JsValue> {
        self.0.call(context)
    }
}

/// [`JobCallback`][spec] records.
///
/// [spec]: https://tc39.es/ecma262/#sec-jobcallback-records
pub struct JobCallback {
    callback: JsFunction,
    host_defined: Box<dyn JobCallbackHost>,
}

#[derive(Clone, HeapTrace)]
pub(crate) struct StoredJobCallback {
    callback: ObjectEdge,
    host_defined: Rc<dyn StoredJobCallbackHost>,
}

impl StoredJobCallback {
    pub(crate) fn to_callback(&self) -> JobCallback {
        JobCallback {
            callback: JsFunction::from_record(self.callback.clone()),
            host_defined: self.host_defined.restore(),
        }
    }
}

trait JobCallbackHost: Any {
    fn store(&self) -> Rc<dyn StoredJobCallbackHost>;
}

impl<T: NativeCapture> JobCallbackHost for T {
    fn store(&self) -> Rc<dyn StoredJobCallbackHost> {
        Rc::new(CapturedJobCallbackHost::<T>(NativeCapture::store(self)))
    }
}

trait StoredJobCallbackHost: HeapTrace {
    fn restore(&self) -> Box<dyn JobCallbackHost>;
}

struct CapturedJobCallbackHost<T: NativeCapture>(T::Stored);

impl<T: NativeCapture> HeapTrace for CapturedJobCallbackHost<T> {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.0.trace(edges)
    }
}

impl<T: NativeCapture> StoredJobCallbackHost for CapturedJobCallbackHost<T> {
    fn restore(&self) -> Box<dyn JobCallbackHost> {
        Box::new(T::restore(&self.0))
    }
}

impl Debug for JobCallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobCallback")
            .field("callback", &self.callback)
            .field("host_defined", &"dyn Any")
            .finish()
    }
}

impl JobCallback {
    /// Creates a new `JobCallback`.
    ///
    /// The host data must describe how to store and restore its captured values
    /// if the callback is retained by a JavaScript object.
    #[inline]
    pub fn new<T: NativeCapture>(callback: JsFunction, host_defined: T) -> Self {
        Self {
            callback,
            host_defined: Box::new(host_defined),
        }
    }

    pub(crate) fn from_host(callback: JsFunction, context: &mut Context) -> JsResult<Self> {
        callback.check_runtime(context.runtime())?;
        let job = context
            .host_hooks()
            .make_job_callback(callback.clone(), context);
        if !JsObject::equals(&callback, job.callback()) {
            return Err(JsNativeError::typ()
                .with_message("host job callback must retain the requested function")
                .into());
        }
        Ok(job)
    }

    pub(crate) fn call(
        &self,
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let result = context
            .host_hooks()
            .call_job_callback(self, this, args, context);
        context.runtime().admit_callback_result(result)
    }

    pub(crate) fn to_stored(&self) -> StoredJobCallback {
        StoredJobCallback {
            callback: self.callback.clone().into_record(),
            host_defined: self.host_defined.store(),
        }
    }

    /// Gets the inner callback of the job.
    #[inline]
    #[must_use]
    pub const fn callback(&self) -> &JsFunction {
        &self.callback
    }

    /// Gets a reference to the host defined additional field as a Rust [`Any`] value.
    #[inline]
    #[must_use]
    pub fn host_defined(&self) -> &dyn Any {
        &*self.host_defined
    }

    /// Gets a mutable reference to the host defined additional field as a Rust [`Any`] value.
    #[inline]
    pub fn host_defined_mut(&mut self) -> &mut dyn Any {
        &mut *self.host_defined
    }
}

/// A job that needs to be handled by a [`JobExecutor`].
///
/// # Requirements
///
/// The specification defines many types of jobs, but all of them must adhere to a set of requirements:
///
/// - At some future point in time, when there is no running execution context and the execution
///   context stack is empty, the implementation must:
///     - Perform any host-defined preparation steps.
///     - Invoke the Job Abstract Closure.
///     - Perform any host-defined cleanup steps, after which the execution context stack must be empty.
/// - Only one Job may be actively undergoing evaluation at any point in time.
/// - Once evaluation of a Job starts, it must run to completion before evaluation of any other Job starts.
/// - The Abstract Closure must return a normal completion, implementing its own handling of errors.
///
/// Boa is a little bit flexible on the last requirement, since it allows jobs to return either
/// values or errors, but the rest of the requirements must be followed for all conformant implementations.
///
/// Additionally, each job type can have additional requirements that must also be followed in addition
/// to the previous ones.
#[non_exhaustive]
#[derive(Debug)]
pub enum Job {
    /// A `Promise`-related job.
    ///
    /// See [`PromiseJob`] for more information.
    PromiseJob(PromiseJob),
    /// A [`Future`]-related job.
    ///
    /// See [`NativeAsyncJob`] for more information.
    AsyncJob(NativeAsyncJob),
    /// A generic job that is to be executed after a number of milliseconds.
    ///
    /// See [`TimeoutJob`] for more information.
    TimeoutJob(TimeoutJob),
    /// A generic job that is to be executed after intervals of a number of milliseconds.
    ///
    /// See [`TimeoutJob`] for more information.
    IntervalJob(IntervalJob),
    /// A generic job.
    ///
    /// See [`GenericJob`] for more information.
    GenericJob(GenericJob),
    /// A subscription that yields cleanup jobs for a `FinalizationRegistry`.
    ///
    /// The engine enqueues the subscription once when it creates the registry. The executor
    /// can retain it across calls to [`JobExecutor::run_jobs`] and take a cleanup job each
    /// time the engine signals that the registry needs cleanup.
    ///
    /// # Execution
    ///
    /// As described on the [spec's section about execution][execution],
    ///
    /// > Because calling `HostEnqueueFinalizationRegistryCleanupJob` is optional,
    /// > registered objects in a `FinalizationRegistry` do not necessarily hold
    /// > that `FinalizationRegistry` live. Implementations may omit `FinalizationRegistry`
    /// > callbacks for any reason, e.g., if the `FinalizationRegistry` itself becomes
    /// > dead, or if the application is shutting down.
    ///
    /// Waiting subscriptions should allow [`JobExecutor::run_jobs`] to return when idle.
    ///
    /// By the same token, it is recommended to execute `FinalizationRegistry` cleanup
    /// jobs separately from all other enqueued [`NativeAsyncJob`]s, prioritizing the
    /// execution of all other jobs if possible.
    ///
    /// [execution]: https://tc39.es/ecma262/#sec-weakref-execution
    FinalizationRegistryCleanupJob(FinalizationRegistryCleanupJob),
}

impl From<NativeAsyncJob> for Job {
    fn from(native_async_job: NativeAsyncJob) -> Self {
        Job::AsyncJob(native_async_job)
    }
}

impl From<PromiseJob> for Job {
    fn from(promise_job: PromiseJob) -> Self {
        Job::PromiseJob(promise_job)
    }
}

impl From<TimeoutJob> for Job {
    fn from(job: TimeoutJob) -> Self {
        Job::TimeoutJob(job)
    }
}

impl From<IntervalJob> for Job {
    fn from(job: IntervalJob) -> Self {
        Job::IntervalJob(job)
    }
}

impl From<GenericJob> for Job {
    fn from(job: GenericJob) -> Self {
        Job::GenericJob(job)
    }
}

/// An executor of `ECMAscript` [Jobs].
///
/// This is the main API that allows creating custom event loops.
///
/// [Jobs]: https://tc39.es/ecma262/#sec-jobs
pub trait JobExecutor: Any {
    /// Enqueues a `Job` on the executor.
    ///
    /// This method combines all the host-defined job enqueueing operations into a single method.
    /// See the [spec] for more information on the requirements that each operation must follow.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-jobs
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context);

    /// Runs all jobs in the executor.
    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()>;

    /// Asynchronously runs all jobs in the executor.
    ///
    /// By default forwards to [`JobExecutor::run_jobs`]. Implementors using async should override this
    /// with a proper algorithm to run jobs asynchronously.
    #[expect(async_fn_in_trait, reason = "all our APIs are single-threaded")]
    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()>
    where
        Self: Sized,
    {
        self.run_jobs(&mut context.borrow_mut())
    }
}

/// A job executor that does nothing.
///
/// This executor is mostly useful if you want to disable the promise capabilities of the engine. This
/// can be done by passing it to the [`ContextBuilder`]:
///
/// ```
/// use surgeist_js::{
///     context::ContextBuilder,
///     job::{IdleJobExecutor, JobExecutor},
/// };
/// use std::rc::Rc;
///
/// let executor = Rc::new(IdleJobExecutor);
/// let context = ContextBuilder::new().job_executor(executor).build();
/// ```
///
/// [`ContextBuilder`]: crate::context::ContextBuilder
#[derive(Debug, Clone, Copy)]
pub struct IdleJobExecutor;

impl JobExecutor for IdleJobExecutor {
    fn enqueue_job(self: Rc<Self>, _: Job, _: &mut Context) {}

    fn run_jobs(self: Rc<Self>, _: &mut Context) -> JsResult<()> {
        Ok(())
    }
}

#[derive(Debug)]
enum ClockJob {
    Timeout(TimeoutJob),
    Interval(IntervalJob),
}

impl ClockJob {
    fn cancelled(&self) -> bool {
        match self {
            ClockJob::Timeout(t) => t.cancelled(),
            ClockJob::Interval(i) => i.cancelled(),
        }
    }
}

#[derive(Default)]
pub(crate) struct RuntimeJobQueues {
    promise_jobs: RefCell<VecDeque<PromiseJob>>,
    async_jobs: RefCell<VecDeque<NativeAsyncJob>>,
    finalization_registry_subscriptions: RefCell<VecDeque<FinalizationRegistryCleanupJob>>,
    clock_jobs: RefCell<BTreeMap<JsInstant, Vec<ClockJob>>>,
    generic_jobs: RefCell<VecDeque<GenericJob>>,
}

impl RuntimeJobQueues {
    fn clear(&self) {
        // Dropping job captures can call back into the executor.
        let promise_jobs = self.promise_jobs.take();
        let async_jobs = self.async_jobs.take();
        let clock_jobs = self.clock_jobs.take();
        let generic_jobs = self.generic_jobs.take();
        drop((promise_jobs, async_jobs, clock_jobs, generic_jobs));
    }

    fn is_empty(&self) -> bool {
        self.promise_jobs.borrow().is_empty()
            && self.async_jobs.borrow().is_empty()
            && self.generic_jobs.borrow().is_empty()
            && self.clock_jobs.borrow().is_empty()
    }

    fn take_ready_cleanup(&self) -> Option<NativeJob> {
        let mut subscriptions = self.finalization_registry_subscriptions.borrow_mut();
        for _ in 0..subscriptions.len() {
            let subscription = subscriptions
                .pop_front()
                .expect("one subscription per iteration");
            match subscription.try_take() {
                Poll::Pending => subscriptions.push_back(subscription),
                Poll::Ready(Some(job)) => {
                    subscriptions.push_back(subscription);
                    return Some(job);
                }
                Poll::Ready(None) => {}
            }
        }
        None
    }
}

/// A simple FIFO executor that bails on the first error.
///
/// Jobs are queued by runtime. Contexts sharing both a runtime and this executor
/// can run each other's jobs. Independent runtimes have independent queues.
/// Dropping the last attached context discards its runtime's remaining jobs.
///
/// This is the default job executor for the [`Context`]. To disable running
/// promise jobs, see [`IdleJobExecutor`].
#[derive(Default)]
pub struct SimpleJobExecutor {
    queues: RefCell<Vec<(ObjectId, Weak<RuntimeJobQueues>)>>,
    stop: Arc<AtomicBool>,
}

impl Debug for SimpleJobExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleJobExecutor").finish_non_exhaustive()
    }
}

impl SimpleJobExecutor {
    /// Creates a new `SimpleJobExecutor`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the cancellation token for this executor.
    ///
    /// Setting the signal to `true` exits the event loop that observes it and
    /// discards that runtime's ordinary queued jobs. Finalization registry
    /// subscriptions remain available for later runs.
    pub fn get_cancellation_token(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }

    pub(crate) fn attach_context(&self, context: &mut Context) -> Rc<RuntimeJobQueues> {
        let queues = self.queues_for(context);
        context.retain_simple_job_queue(&queues);
        queues
    }

    fn queues_for(&self, context: &Context) -> Rc<RuntimeJobQueues> {
        // A realm identity names its heap without retaining the realm or runtime.
        let identity = context.realm().identity();
        let mut queues = self.queues.borrow_mut();
        if let Some(jobs) = queues
            .iter()
            .find(|(key, _)| key.same_heap(identity))
            .and_then(|(_, jobs)| jobs.upgrade())
        {
            return jobs;
        }
        queues.retain(|(_, jobs)| jobs.strong_count() != 0);
        let jobs = Rc::new(RuntimeJobQueues::default());
        queues.push((identity, Rc::downgrade(&jobs)));
        jobs
    }
}

impl Drop for SimpleJobExecutor {
    fn drop(&mut self) {
        // A context can attach through a direct call without owning the executor.
        let queues = mem::take(self.queues.get_mut());
        for (_, jobs) in queues {
            if let Some(jobs) = jobs.upgrade() {
                jobs.clear();
                drop(jobs.finalization_registry_subscriptions.take());
            }
        }
    }
}

impl JobExecutor for SimpleJobExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        let queues = self.attach_context(context);
        match job {
            Job::PromiseJob(p) => queues.promise_jobs.borrow_mut().push_back(p),
            Job::AsyncJob(a) => queues.async_jobs.borrow_mut().push_back(a),
            Job::TimeoutJob(t) => {
                let now = context.clock().now();
                queues
                    .clock_jobs
                    .borrow_mut()
                    .entry(now + t.timeout())
                    .or_default()
                    .push(ClockJob::Timeout(t));
            }
            Job::IntervalJob(i) => {
                let now = context.clock().now();
                queues
                    .clock_jobs
                    .borrow_mut()
                    .entry(now + i.interval())
                    .or_default()
                    .push(ClockJob::Interval(i));
            }
            Job::GenericJob(g) => queues.generic_jobs.borrow_mut().push_back(g),
            Job::FinalizationRegistryCleanupJob(subscription) => {
                queues
                    .finalization_registry_subscriptions
                    .borrow_mut()
                    .push_back(subscription);
            }
        }
    }

    fn run_jobs(self: Rc<Self>, context: &mut Context) -> JsResult<()> {
        future::block_on(self.run_jobs_async(&RefCell::new(context)))
    }

    async fn run_jobs_async(self: Rc<Self>, context: &RefCell<&mut Context>) -> JsResult<()>
    where
        Self: Sized,
    {
        let queues = self.attach_context(&mut context.borrow_mut());
        let mut group = FutureGroup::new();
        loop {
            if self.stop.load(Ordering::Relaxed) && self.stop.swap(false, Ordering::Relaxed) {
                queues.clear();
                return Ok(());
            }

            for job in queues.async_jobs.take() {
                group.insert(job.call(context));
            }

            // Dispatch all past-due timeout jobs before the termination check.
            {
                let now = context.borrow().clock().now();
                let jobs_to_run = {
                    let mut timeout_jobs = queues.clock_jobs.borrow_mut();
                    let mut jobs_to_keep = timeout_jobs.split_off(&now);
                    jobs_to_keep.retain(|_, jobs| {
                        jobs.retain(|job| !job.cancelled());
                        !jobs.is_empty()
                    });
                    mem::replace(&mut *timeout_jobs, jobs_to_keep)
                };

                for jobs in jobs_to_run.into_values() {
                    for job in jobs {
                        if !job.cancelled() {
                            match job {
                                ClockJob::Timeout(job) => {
                                    if let Err(err) = job.call(&mut context.borrow_mut()) {
                                        queues.clear();
                                        return Err(err);
                                    }
                                }
                                ClockJob::Interval(job) => {
                                    let context = &mut context.borrow_mut();
                                    let now = context.clock().now();
                                    if let Err(err) = job.call(context) {
                                        queues.clear();
                                        return Err(err);
                                    }
                                    queues
                                        .clock_jobs
                                        .borrow_mut()
                                        .entry(now + job.interval())
                                        .or_default()
                                        .push(ClockJob::Interval(job));
                                }
                            }
                        }
                    }
                }
            }

            if queues.is_empty() && group.is_empty() {
                let Some(job) = queues.take_ready_cleanup() else {
                    break;
                };
                if let Err(err) = job.call(&mut context.borrow_mut()) {
                    queues.clear();
                    return Err(err);
                }
            }

            if let Some(Err(err)) = future::poll_once(group.next()).await.flatten() {
                queues.clear();
                return Err(err);
            }

            let jobs = queues.promise_jobs.take();
            for job in jobs {
                if let Err(err) = job.call(&mut context.borrow_mut()) {
                    queues.clear();
                    return Err(err);
                }
            }

            let jobs = queues.generic_jobs.take();
            for job in jobs {
                if let Err(err) = job.call(&mut context.borrow_mut()) {
                    queues.clear();
                    return Err(err);
                }
            }
            context.borrow_mut().clear_kept_objects();
            future::yield_now().await;
        }

        Ok(())
    }
}
