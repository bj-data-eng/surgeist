//! Boa's API to create and customize `ECMAScript` jobs and job queues.
//!
//! [`Job`] is an ECMAScript [Job], or a closure that runs an `ECMAScript` computation when
//! there's no other computation running. The module defines several type of jobs:
//! - [`PromiseJob`] for Promise related jobs.
//! - [`TimeoutJob`] for jobs that run after a certain amount of time.
//! - [`NativeAsyncJob`] for jobs that support [`std::future::Future`].
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

use crate::context::time::JsDuration;
use crate::execution::JobAction;
use crate::sys::time;
use crate::{
    Context, JsNativeError, JsResult, JsValue,
    native_function::NativeCapture,
    object::{JsFunction, JsObject, ObjectEdge},
    realm::Realm,
};
use crate::{ContextAccess, HostContext};
use std::any::Any;
use std::cell::Cell;
use std::fmt::Debug;
use std::rc::Rc;
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace as HeapTrace};

mod asynchronous;
mod fault;
mod identity;
mod observer;
mod publication;
mod queue;
mod task;

pub use asynchronous::{JobStep, NativeAsyncJob};
pub use fault::{JobFault, JobFaultCause, JobFaultOwner, JobRegistrationError, JobResourceError};
pub(crate) use fault::{UnwindCleanup, reserve_registration};
pub use identity::ContextId;
pub(crate) use identity::ContextLifetime;
pub use observer::{JobObserver, JobRetirement, JobStatus, JobWait};
pub(crate) use observer::{RetirementRegistration, RetirementRelay};
pub(crate) use publication::finish_publication;
pub use publication::{JobAcceptance, JobPublicationCause, JobPublicationError};
pub(crate) use queue::RuntimeJobQueues;
pub use queue::SimpleJobExecutor;
pub use task::ContextTask;

/// Custody-preserving result of a consuming synchronous job entry.
#[must_use = "retain the returned deferred job or explicitly dispose it"]
#[derive(Debug)]
pub enum JobCall<J> {
    /// The original uncalled job and its before-action refusal.
    Deferred { job: J, error: crate::JsError },
    /// An admitted invocation completed and moved its result to this caller.
    Complete(JsResult<JsValue>),
}

/// Result of one invocation of a reusable job callback.
#[must_use = "inspect whether the reusable callback was invoked or deferred"]
#[derive(Debug)]
pub enum JobInvocation {
    /// No callback ran; the reusable callback remains unchanged.
    Deferred(crate::JsError),
    /// The admitted callback returned this result.
    Complete(JsResult<JsValue>),
}

pub use crate::builtins::finalization_registry::FinalizationRegistryCleanupJob;

type NativeCallback = Box<dyn for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue>>;

/// An ECMAScript [Job Abstract Closure].
///
/// This is basically a synchronous task that needs to be run to progress [`Promise`] objects,
/// or unblock threads waiting on [`Atomics.waitAsync`].
///
/// [Job Abstract Closure]: https://tc39.es/ecma262/#sec-jobs
/// [`Promise`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
/// [`Atomics.waitAsync`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Atomics/waitAsync
pub struct NativeJob {
    custody: InvocationCustody,
    f: NativeCallback,
    realm: Option<Realm>,
}

/// Owns invocation cleanup and the optional realm boundary of native jobs.
fn admit_native_job(context: &mut Context, realm: Option<&Realm>) -> JsResult<()> {
    context.check_execution_stop()?;
    if let Some(realm) = realm {
        context.check_realm(realm)?;
    }
    context.admit_job_action(JobAction::Invocation)
}

fn invoke_native_job(
    context: &mut Context,
    realm: Option<Realm>,
    call: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue>,
) -> JsResult<JsValue> {
    let checkpoint = context.vm.call_checkpoint();
    let previous_realm = realm.map(|realm| context.enter_realm(realm)).transpose()?;
    let observation = context.begin_execution_phase(JobAction::Invocation.phase());
    let mut active = context.guard(move |context| {
        context.vm.restore_call(checkpoint);
        if let Some(mut realm) = previous_realm {
            context.swap_realm(&mut realm);
        }
        context.end_execution_phase(observation);
    });
    active.check_execution_stop()?;
    active.note_execution_effect();
    let result = call(active.access());
    drop(active);
    context.check_execution_stop()?;
    result
}

struct InvocationCustody {
    observer: JobObserver,
    entered: bool,
    lifecycle: crate::builtins::promise::ReactionJobLifecycle,
}
impl Drop for InvocationCustody {
    fn drop(&mut self) {
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            if let Err(fault) = self.lifecycle.dispose(self.entered) {
                self.observer.record_fault(fault);
            }
        });
        cleanup.run(|| {
            self.observer.retire(if self.entered {
                JobRetirement::Interrupted
            } else {
                JobRetirement::Dropped
            })
        });
        cleanup.finish();
    }
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
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self {
            custody: InvocationCustody {
                observer: JobObserver::new(),
                entered: false,
                lifecycle: crate::builtins::promise::ReactionJobLifecycle::untracked(),
            },
            f: Box::new(f),
            realm: None,
        }
    }

    /// Creates a new `NativeJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self {
            custody: InvocationCustody {
                observer: JobObserver::new(),
                entered: false,
                lifecycle: crate::builtins::promise::ReactionJobLifecycle::untracked(),
            },
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
    ///
    /// Before-action refusal returns [`JobCall::Deferred`] with this original
    /// uncalled wrapper. [`JobCall::Complete`] owns the admitted invocation result.
    pub fn call(self, context: ContextAccess<'_>) -> JobCall<Self> {
        let context = context.into_owner();
        if let Err(error) = context.check_execution_stop() {
            return JobCall::Deferred { job: self, error };
        }
        let fault =
            self.custody
                .observer
                .fault()
                .or_else(|| match self.custody.observer.status() {
                    Ok(JobStatus::Pending) => None,
                    Err(fault) => Some(fault),
                    Ok(_) => Some(self.custody.observer.record_fault(JobFault::new(
                        JobFaultOwner::Task,
                        JobFaultCause::StateInconsistency,
                    ))),
                });
        if let Some(fault) = fault {
            return JobCall::Deferred {
                job: self,
                error: fault.into(),
            };
        }
        if let Err(error) = admit_native_job(context, self.realm.as_ref()) {
            return JobCall::Deferred { job: self, error };
        }
        let Self {
            mut custody,
            f: call,
            realm,
        } = self;
        custody.entered = true;
        let is_module = custody.lifecycle.is_module();
        let run = |context: &mut Context| {
            let result = match custody.lifecycle.check() {
                Ok(()) => invoke_native_job(context, realm, call),
                Err(error) => Err(error),
            };
            if let Err(error) = &result
                && let Err(fault) = custody.lifecycle.finish(error)
            {
                custody.observer.record_fault(fault);
            }
            result
        };
        let mut result = if is_module {
            context.with_module_run(run)
        } else {
            run(context)
        };
        if result
            .as_ref()
            .is_err_and(|error| error.as_engine().is_some())
        {
            custody.observer.retire(JobRetirement::Interrupted);
        } else {
            custody.observer.complete();
        }
        // Observer failure is not authority to revoke a committed module successor.
        if let Some(fault) = custody.observer.fault()
            && !result
                .as_ref()
                .is_err_and(|error| error.as_engine().is_some())
        {
            result = Err(fault.into());
        }
        JobCall::Complete(context.check_execution_stop().and(result))
    }

    /// Observes this task without retaining its callback captures.
    pub fn observer(&self) -> JobObserver {
        self.custody.observer.clone()
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
    f: Box<dyn for<'call> Fn(ContextAccess<'call>) -> JsResult<JsValue>>,
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
        F: for<'call> Fn(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self {
            f: Box::new(f),
            realm: None,
        }
    }

    /// Creates a new `NativeJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: for<'call> Fn(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
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
    ///
    /// A deferred entry runs no callback; this reusable callback remains available.
    pub fn call(&self, context: ContextAccess<'_>) -> JobInvocation {
        let context = context.into_owner();
        if let Err(error) = admit_native_job(context, self.realm.as_ref()) {
            return JobInvocation::Deferred(error);
        }
        JobInvocation::Complete(invoke_native_job(context, self.realm.clone(), &self.f))
    }
}

type Callback = Box<dyn for<'call> FnOnce(ContextAccess<'call>)>;

/// Token to cancel a [`TimeoutJob`] and [`IntervalJob`].
#[derive(Clone)]
pub struct CancellationToken(Rc<Cell<CancellationState>>);

#[derive(Default)]
enum CancellationState {
    Live(Vec<Callback>),
    #[default]
    Revoked,
}

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
        Self(Rc::new(Cell::new(CancellationState::Live(Vec::new()))))
    }

    /// Sets a callback to run when the cancellation token gets used.
    ///
    /// # Errors
    /// Returns an uncatchable revoked-registration error after cancellation or
    /// revocation, or a resource error if callback storage cannot be reserved.
    /// Rejection never invokes the callback or revives the token. Existing live
    /// callbacks retain their custody if reservation fails.
    pub fn push_callback(
        &self,
        f: impl for<'call> FnOnce(ContextAccess<'call>) + 'static,
    ) -> JsResult<()> {
        let CancellationState::Live(mut callbacks) = self.0.take() else {
            return Err(crate::error::JobError::CancellationRevoked.into());
        };
        let reserve = callbacks
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_mul(std::mem::size_of::<Callback>()))
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .ok_or(crate::error::EngineResourceError::SizeOverflow)
            .and_then(|_| {
                callbacks
                    .try_reserve(1)
                    .map_err(|_| crate::error::EngineResourceError::Allocation)
            });
        if let Err(error) = reserve {
            self.0.set(CancellationState::Live(callbacks));
            return Err(error.into());
        }
        callbacks.push(Box::new(f));
        self.0.set(CancellationState::Live(callbacks));
        Ok(())
    }

    /// Cancels the [`TimeoutJob`] or [`IntervalJob`] associated with this cancellation token.
    pub fn cancel(&self, context: ContextAccess<'_>) {
        let mut context = context;
        if let CancellationState::Live(callbacks) = self.0.take() {
            for job in callbacks {
                job(context.reborrow());
            }
        }
    }

    /// Revokes this cancellation token, making it unusable to cancel its associated job.
    pub(crate) fn revoke(&self) {
        drop(self.0.take());
    }

    /// Returns `true` if this cancellation token has been revoked, either because
    /// `cancel` was called or because its associated job has completed.
    #[must_use]
    pub fn revoked(&self) -> bool {
        let state = self.0.take();
        let revoked = matches!(&state, CancellationState::Revoked);
        self.0.set(state);
        revoked
    }
}

/// An ECMAScript [Job] that runs after a certain amount of time.
///
/// This represents the [HostEnqueueTimeoutJob] operation from the specification.
///
/// [HostEnqueueTimeoutJob]: https://tc39.es/ecma262/#sec-hostenqueuetimeoutjob
#[derive(Debug)]
pub struct TimeoutJob {
    cancellation: CancellationLease,
    /// The delay added to the current clock when the job is enqueued.
    timeout: JsDuration,
    /// The job to run after the specified timeout.
    job: NativeJob,
}

#[derive(Debug)]
struct CancellationLease(CancellationToken);
impl Drop for CancellationLease {
    fn drop(&mut self) {
        self.0.revoke();
    }
}

impl TimeoutJob {
    /// Creates a timeout for an existing job with a delay in milliseconds.
    #[must_use]
    pub fn new(job: NativeJob, timeout_in_millis: u64) -> Self {
        Self::with_duration(job, JsDuration::from_millis(timeout_in_millis))
    }

    fn with_duration(job: NativeJob, timeout: JsDuration) -> Self {
        Self {
            cancellation: CancellationLease(CancellationToken::new()),
            timeout,
            job,
        }
    }

    /// Creates a new `TimeoutJob` from a closure and a timeout as [`std::time::Duration`].
    /// Preserves the supplied duration's nanosecond precision and range.
    #[must_use]
    pub fn from_duration<F>(f: F, timeout: impl Into<JsDuration>) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self::with_duration(NativeJob::new(f), timeout.into())
    }

    /// Creates a new `TimeoutJob` from a closure, a timeout, and an execution realm.
    #[must_use]
    pub fn with_realm<F>(f: F, realm: Realm, timeout: time::Duration) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self::with_duration(NativeJob::with_realm(f, realm), timeout.into())
    }

    /// Calls the native job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the native job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    ///
    /// Before-action refusal returns [`JobCall::Deferred`] with this original
    /// uncalled wrapper. [`JobCall::Complete`] owns the admitted invocation result.
    pub fn call(self, context: ContextAccess<'_>) -> JobCall<Self> {
        let Self {
            cancellation,
            timeout,
            job,
        } = self;
        match job.call(context) {
            JobCall::Deferred { job, error } => JobCall::Deferred {
                job: Self {
                    cancellation,
                    timeout,
                    job,
                },
                error,
            },
            JobCall::Complete(result) => JobCall::Complete(result),
        }
    }

    /// Observes the timeout callback's disposition.
    pub fn observer(&self) -> JobObserver {
        self.job.observer()
    }

    /// Returns the configured timeout duration.
    #[inline]
    #[must_use]
    pub fn timeout(&self) -> JsDuration {
        self.timeout
    }

    /// Returns `true` if the timeout was cancelled, and its execution can be skipped.
    #[inline]
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.cancellation.0.revoked()
    }

    /// Returns the [`CancellationToken`] for this timeout job.
    #[must_use]
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation.0
    }
}

/// An ECMAScript [Job] that runs at a certain interval of time.
///
/// This represents jobs enqueued by APIs such as [`setInterval`].
///
/// [`setInterval`]: https://developer.mozilla.org/en-US/docs/Web/API/Window/setInterval
#[derive(Debug)]
pub struct IntervalJob {
    /// The delay added to the current clock for each subsequent invocation.
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
    /// Creates an interval for an existing job with a period in milliseconds.
    #[must_use]
    pub fn new(job: NativeJobFn, interval_in_millis: u64) -> Self {
        Self::with_duration(job, JsDuration::from_millis(interval_in_millis))
    }

    fn with_duration(job: NativeJobFn, interval: JsDuration) -> Self {
        Self {
            interval,
            job,
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Creates a new `IntervalJob` from a closure and an interval as [`std::time::Duration`].
    /// Preserves the supplied duration's nanosecond precision and range.
    #[must_use]
    pub fn from_duration<F>(f: F, interval: impl Into<JsDuration>) -> Self
    where
        F: for<'call> Fn(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self::with_duration(NativeJobFn::new(f), interval.into())
    }

    /// Creates a new `IntervalJob` from a closure, an interval, and an execution realm.
    #[must_use]
    pub fn with_realm<F>(f: F, realm: Realm, interval: time::Duration) -> Self
    where
        F: for<'call> Fn(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self::with_duration(NativeJobFn::with_realm(f, realm), interval.into())
    }

    /// Calls the interval job with the specified [`Context`].
    ///
    /// # Note
    ///
    /// If the interval job has an execution realm defined, this sets the running execution
    /// context to the realm's before calling the inner closure, and resets it after execution.
    ///
    /// A deferred entry runs no callback; this reusable callback remains available.
    pub fn call(&self, context: ContextAccess<'_>) -> JobInvocation {
        let context = context.into_owner();
        self.job.call(context.access())
    }

    /// Returns the configured interval duration.
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
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
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
    ///
    /// A deferred entry returns the original wrapper without consuming its callback.
    pub fn call(self, context: ContextAccess<'_>) -> JobCall<Self> {
        match self.0.call(context) {
            JobCall::Deferred { job, error } => JobCall::Deferred {
                job: Self(job),
                error,
            },
            JobCall::Complete(result) => JobCall::Complete(result),
        }
    }

    /// Observes this callback's disposition.
    pub fn observer(&self) -> JobObserver {
        self.0.observer()
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
    pub(crate) fn with_reaction_lifecycle(
        mut self,
        lifecycle: crate::builtins::promise::ReactionJobLifecycle,
    ) -> Self {
        self.0.custody.lifecycle = lifecycle;
        self
    }
    /// Creates a new `PromiseJob` from a closure.
    pub fn new<F>(f: F) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
    {
        Self(NativeJob::new(f))
    }

    /// Creates a new `PromiseJob` from a closure and an execution realm.
    pub fn with_realm<F>(f: F, realm: Realm) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<JsValue> + 'static,
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
    ///
    /// A deferred entry returns the original wrapper without consuming its callback.
    pub fn call(self, context: ContextAccess<'_>) -> JobCall<Self> {
        match self.0.call(context) {
            JobCall::Deferred { job, error } => JobCall::Deferred {
                job: Self(job),
                error,
            },
            JobCall::Complete(result) => JobCall::Complete(result),
        }
    }

    /// Observes this callback's disposition.
    pub fn observer(&self) -> JobObserver {
        self.0.observer()
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
            .make_job_callback(callback.clone(), context.access());
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
            .call_job_callback(self, this, args, context.access());
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
    /// A [`std::future::Future`]-related job.
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

impl Job {
    /// Checks a selected realm against the actual receiving runtime.
    ///
    /// This does not invoke the job, charge invocation work or grant execution
    /// authority. A valid continuation can still be accepted after an enclosing
    /// operation stopped. Executors that deliberately discard inputs may apply
    /// their discard policy without performing this check.
    ///
    /// # Errors
    /// Returns a publication-specific foreign-realm cause, or an earlier
    /// retained engine stop, without changing this original input.
    pub fn check_publication(&self, context: ContextAccess<'_>) -> Result<(), JobPublicationCause> {
        let realm = match self {
            Self::PromiseJob(job) => job.realm(),
            Self::AsyncJob(job) => job.realm(),
            Self::GenericJob(job) => job.0.realm(),
            Self::TimeoutJob(job) => job.job.realm(),
            Self::IntervalJob(job) => job.job.realm(),
            Self::FinalizationRegistryCleanupJob(_) => None,
        };
        let context = context.into_owner();
        match realm.map_or(Ok(()), |realm| context.check_publication_realm(realm)) {
            Ok(()) => Ok(()),
            Err(error) => Err(publication::JobPublicationCause::from_receiving(
                error, context,
            )),
        }
    }
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
    ///
    /// Rejection returns the exact original unaccepted input. An inline failure
    /// after acceptance is [`JobAcceptance::AcceptedWithError`], never a
    /// replayable rejection. Explicit policy disposal is
    /// [`JobAcceptance::Discarded`]. Receiving executors check a selected realm
    /// with [`Job::check_publication`] before acceptance or invocation.
    ///
    /// # Errors
    /// Returns the original job and a validated engine cause before acceptance.
    #[expect(
        clippy::result_large_err,
        reason = "retain original job ownership without allocating to box the refused input"
    )]
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        context: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError>;

    /// Runs all jobs in the executor.
    fn run_jobs(self: Rc<Self>, context: ContextAccess<'_>) -> JsResult<()>;

    /// Asynchronously runs all jobs in the executor.
    ///
    /// By default forwards to [`JobExecutor::run_jobs`]. Implementors using async should override this
    /// with a proper algorithm to run jobs asynchronously.
    #[expect(async_fn_in_trait, reason = "all our APIs are single-threaded")]
    async fn run_jobs_async(self: Rc<Self>, context: &HostContext<'_>) -> JsResult<()>
    where
        Self: Sized,
    {
        context.with(|access| self.run_jobs(access))
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
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        _: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        drop(job);
        Ok(JobAcceptance::Discarded)
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        Ok(())
    }
}
