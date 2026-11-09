//! Owned native task preparation, affinity and action cleanup.
use super::{
    ContextId, ContextTask, JobFault, JobFaultCause, JobFaultOwner, JobObserver, JobRetirement,
    JobStatus, UnwindCleanup,
    identity::{OwnerRegistration, OwnerTask},
};
use crate::{
    Context, ContextAccess, HostContext, JsError, JsResult, JsValue, error::EngineError,
    execution::JobAction, realm::Realm,
};
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::{Rc, Weak},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
};
use surgeist_ledger::PhaseObservation;

type Factory = Box<dyn for<'call> FnOnce(ContextAccess<'call>) -> JsResult<ContextTask<JsValue>>>;
enum State {
    Factory(Factory),
    Task(ContextTask<JsValue>),
}

/// One explicit owned native job transition.
#[must_use = "a deferred or pending job remains owned and must not be mistaken for completion"]
#[derive(Debug)]
pub enum JobStep {
    /// No action began; the same job remains resumable.
    Deferred(JsError),
    /// Preparation completed without polling the task.
    Prepared,
    /// An admitted poll returned Pending.
    Pending,
    /// The first terminal result, moved to this caller exactly once.
    Complete(JsResult<JsValue>),
    /// A later rootless, result-free observation, including a structural fault.
    Finished(Result<JobStatus, JobFault>),
}

/// Owned native asynchronous work, independently retained between drains.
pub struct NativeAsyncJob {
    control: Rc<Control>,
    realm: Option<Realm>,
}
impl std::fmt::Debug for NativeAsyncJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeAsyncJob")
            .field("status", &self.control.observer.status())
            .finish_non_exhaustive()
    }
}
struct Control {
    state: Cell<Option<State>>,
    // A second physical custody slot is prepared for an unexpected restoration
    // collision. A quarantined control cannot create another action lease.
    retained: Cell<Option<State>>,
    affinity: Cell<Option<ContextId>>,
    registration: Cell<Option<OwnerRegistration>>,
    disposed: Cell<bool>,
    observer: JobObserver,
    delivered: Cell<bool>,
    wake: Arc<TaskWake>,
}
impl Control {
    fn owner(&self) -> Option<ContextId> {
        let owner = self.affinity.take();
        let result = owner.clone();
        self.affinity.set(owner);
        result
    }
    fn restore_state(&self, state: State) {
        if self.disposed.get() {
            drop(state);
            return;
        }
        if let Some(previous) = self.state.replace(Some(state)) {
            let extra = self.retained.replace(Some(previous));
            let mut cleanup = UnwindCleanup::default();
            cleanup.run(|| {
                self.fault(JobFault::new(
                    JobFaultOwner::Task,
                    JobFaultCause::StateInconsistency,
                ));
            });
            cleanup.run(|| drop(extra));
            cleanup.finish();
        }
    }
}
impl OwnerTask for Control {
    fn mark_fault(&self, fault: JobFault) {
        self.observer.latch_fault(fault);
    }
    fn fault(&self, fault: JobFault) {
        self.wake.fault(fault);
    }
    fn retire(&self, reason: JobRetirement) {
        self.disposed.set(true);
        let state = self.state.take();
        let retained = self.retained.take();
        let registration = self.registration.take();
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| self.observer.retire(reason));
        cleanup.run(|| {
            if let Some(fault) = self.observer.fault() {
                self.wake.fault(fault);
            }
        });
        cleanup.run(|| self.wake.retire());
        cleanup.run(|| drop((state, retained, registration)));
        cleanup.finish();
    }
}
impl Drop for Control {
    fn drop(&mut self) {
        let state = self.state.get_mut().take();
        let retained = self.retained.get_mut().take();
        let registration = self.registration.get_mut().take();
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| self.observer.retire(JobRetirement::Dropped));
        cleanup.run(|| {
            if let Some(fault) = self.observer.fault() {
                self.wake.fault(fault);
            }
        });
        cleanup.run(|| self.wake.retire());
        cleanup.run(|| drop((state, retained, registration)));
        cleanup.finish();
    }
}
enum LeaseState {
    Held(State),
    Dispatched,
}
struct StateLease {
    control: Rc<Control>,
    state: LeaseState,
}
impl StateLease {
    fn take(&mut self) -> Option<State> {
        match std::mem::replace(&mut self.state, LeaseState::Dispatched) {
            LeaseState::Held(state) => Some(state),
            LeaseState::Dispatched => None,
        }
    }
}
impl Drop for StateLease {
    fn drop(&mut self) {
        if let Some(state) = self.take() {
            self.control.restore_state(state);
        }
    }
}
pub(super) struct AsyncIdentity(Weak<Control>);
impl AsyncIdentity {
    pub(super) fn owner(&self) -> Option<ContextId> {
        self.0.upgrade().and_then(|control| control.owner())
    }
    pub(super) fn retire(&self, reason: JobRetirement) {
        if let Some(control) = self.0.upgrade() {
            control.retire(reason);
        }
    }
}
impl NativeAsyncJob {
    /// Creates an unbound request whose admitted factory returns owned work.
    pub fn new<F>(factory: F) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<ContextTask<JsValue>> + 'static,
    {
        let observer = JobObserver::new();
        let wake = Arc::new(TaskWake::new(observer.clone()));
        let driver: Arc<dyn super::observer::FaultWake> = wake.clone();
        observer.bind_task_driver(Arc::downgrade(&driver));
        Self {
            control: Rc::new(Control {
                state: Cell::new(Some(State::Factory(Box::new(factory)))),
                retained: Cell::new(None),
                affinity: Cell::new(None),
                registration: Cell::new(None),
                disposed: Cell::new(false),
                observer,
                delivered: Cell::new(false),
                wake,
            }),
            realm: None,
        }
    }
    /// Creates an unbound request with an explicit execution realm.
    pub fn with_realm<F>(factory: F, realm: Realm) -> Self
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<ContextTask<JsValue>> + 'static,
    {
        let mut job = Self::new(factory);
        job.realm = Some(realm);
        job
    }
    /// Returns the selected execution realm, if any.
    pub fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }
    /// Observes disposition without retaining the task or its Context.
    pub fn observer(&self) -> JobObserver {
        self.control.observer.clone()
    }
    pub(super) fn identity(&self) -> AsyncIdentity {
        AsyncIdentity(Rc::downgrade(&self.control))
    }
    pub(crate) fn owner(&self) -> Option<ContextId> {
        self.control.owner()
    }
    pub(crate) fn is_ready(&self) -> bool {
        self.control.wake.ready.load(Ordering::Acquire)
    }
    pub(crate) fn register_driver(&self, waker: &Waker) {
        self.control.wake.register(waker);
    }
    pub(crate) fn is_prepared(&self) -> bool {
        // Terminal observation must not be routed through live-task affinity or
        // readiness checks merely because quarantined storage is still owned.
        if self.control.observer.status() != Ok(JobStatus::Pending)
            || self.control.observer.fault().is_some()
        {
            return false;
        }
        let state = self.control.state.take();
        let prepared = matches!(&state, Some(State::Task(_)));
        self.control.state.set(state);
        prepared
    }
    fn fault_step(&self, fault: JobFault) -> JobStep {
        let first = self.control.observer.record_fault(fault);
        if self.control.delivered.replace(true) {
            JobStep::Finished(Err(first))
        } else {
            JobStep::Complete(Err(first.into()))
        }
    }
    /// Performs exactly one preparation or task poll with temporary engine access.
    ///
    /// Before-action refusal retains resumable state. Terminal observation is
    /// rootless and charges nothing. A fault quarantines owned payload storage
    /// until explicit retirement or final disposal, without polling it again.
    pub fn run_step(&mut self, cx: &mut TaskContext<'_>, host: &HostContext<'_>) -> JobStep {
        match self.control.observer.status() {
            Err(fault) => return self.fault_step(fault),
            Ok(JobStatus::Pending) => {}
            Ok(status) => {
                if self.control.delivered.replace(true) {
                    return JobStep::Finished(Ok(status));
                }
                return match status {
                    JobStatus::Retired(reason) => {
                        JobStep::Complete(Err(EngineError::JobRetired(reason).into()))
                    }
                    JobStatus::Completed | JobStatus::Pending => self.fault_step(JobFault::new(
                        JobFaultOwner::Task,
                        JobFaultCause::StateInconsistency,
                    )),
                };
            }
        }
        if let Some(fault) = self.control.observer.fault() {
            return self.fault_step(fault);
        }
        self.control.wake.register(cx.waker());
        if let Some(fault) = self.control.observer.fault() {
            return self.fault_step(fault);
        }
        let Some(state) = self.control.state.take() else {
            return JobStep::Deferred(EngineError::ContextAccessBusy.into());
        };
        let action_kind = match &state {
            State::Factory(_) => JobAction::Preparation,
            State::Task(_) => JobAction::Poll,
        };
        let mut lease = StateLease {
            control: self.control.clone(),
            state: LeaseState::Held(state),
        };
        let mut action = match Action::begin(host, self.realm.as_ref(), action_kind, &self.control)
        {
            Ok(action) => action,
            Err(error) => return JobStep::Deferred(error),
        };
        let dispatched = catch_unwind(AssertUnwindSafe(|| match &mut lease.state {
            LeaseState::Held(State::Task(task)) => {
                self.control.wake.ready.store(false, Ordering::Release);
                let waker = Waker::from(self.control.wake.clone());
                match task.poll(&mut TaskContext::from_waker(&waker), host) {
                    Poll::Pending => StepResult::Pending,
                    Poll::Ready(result) => StepResult::Complete(result),
                }
            }
            LeaseState::Held(State::Factory(_)) => match lease.take() {
                Some(State::Factory(factory)) => match host.with(factory) {
                    Ok(task) => {
                        lease.state = LeaseState::Held(State::Task(task));
                        StepResult::Prepared
                    }
                    Err(error) => StepResult::Complete(Err(error)),
                },
                Some(state) => {
                    lease.state = LeaseState::Held(state);
                    StepResult::Complete(Err(JobFault::new(
                        JobFaultOwner::Task,
                        JobFaultCause::StateInconsistency,
                    )
                    .into()))
                }
                None => StepResult::Complete(Err(JobFault::new(
                    JobFaultOwner::Task,
                    JobFaultCause::StateInconsistency,
                )
                .into())),
            },
            LeaseState::Dispatched => StepResult::Complete(Err(JobFault::new(
                JobFaultOwner::Task,
                JobFaultCause::StateInconsistency,
            )
            .into())),
        }));
        let mut result = match dispatched {
            Ok(result) => result,
            Err(payload) => {
                let mut cleanup = UnwindCleanup::default();
                cleanup.run(|| {
                    let _ = action.restore_and_check();
                });
                self.control.disposed.set(true);
                cleanup.run(|| drop(lease.take()));
                cleanup.run(|| drop(action));
                cleanup.run(|| self.control.observer.retire(JobRetirement::Interrupted));
                // The original callback unwind remains the initiating failure.
                cleanup.resume(payload)
            }
        };
        let stopped = match action.restore_and_check() {
            Ok(result) => result.err(),
            Err(fault) => {
                self.control.fault(fault);
                None
            }
        };
        if let Some(fault) = host.fault() {
            self.control.fault(fault);
        }
        let observed = self.control.observer.status();
        let fault = self.control.observer.fault().or_else(|| observed.err());
        let retired = match observed {
            Ok(JobStatus::Retired(reason)) => Some(reason),
            _ => None,
        };
        if stopped.is_some()
            || retired.is_some()
            || (fault.is_none() && matches!(result, StepResult::Complete(_)))
        {
            self.control.disposed.set(true);
            drop(lease.take());
        }
        let later_stop = match action.check_stop() {
            Ok(result) => result.err(),
            Err(fault) => {
                self.control.fault(fault);
                None
            }
        };
        let stopped = later_stop.or(stopped);
        let fault = self.control.observer.fault().or(fault);
        if let Some(stop) = stopped {
            result = StepResult::Complete(Err(stop));
        } else if let Some(fault) = fault
            && !matches!(&result, StepResult::Complete(Err(error)) if error.as_engine().is_some())
        {
            result = StepResult::Complete(Err(fault.into()));
        } else if let Some(reason) = retired {
            result = StepResult::Complete(Err(EngineError::JobRetired(reason).into()));
        }
        match action.finish() {
            Ok(Ok(())) => {}
            Ok(Err(stop)) => {
                self.control.disposed.set(true);
                drop(lease.take());
                result = StepResult::Complete(Err(stop));
            }
            Err(fault) => {
                self.control.fault(fault);
                if !matches!(&result, StepResult::Complete(Err(error)) if error.as_engine().is_some())
                {
                    result = StepResult::Complete(Err(fault.into()));
                }
            }
        }
        match result {
            StepResult::Prepared => {
                self.control.wake.ready.store(true, Ordering::Release);
                JobStep::Prepared
            }
            StepResult::Pending => JobStep::Pending,
            StepResult::Complete(mut result) => {
                if result
                    .as_ref()
                    .is_err_and(|error| error.as_engine().is_some())
                {
                    self.control.observer.retire(JobRetirement::Interrupted);
                } else {
                    self.control.observer.complete();
                }
                if self.control.disposed.get() {
                    drop(self.control.registration.take());
                }
                self.control.delivered.set(true);
                self.control.wake.retire();
                if let Some(fault) = self.control.observer.fault()
                    && !result
                        .as_ref()
                        .is_err_and(|error| error.as_engine().is_some())
                {
                    result = Err(fault.into());
                }
                match host.with_job_cleanup(|access| access.into_owner().check_execution_stop()) {
                    Ok(Err(stop)) => result = Err(stop),
                    Err(fault)
                        if !result
                            .as_ref()
                            .is_err_and(|error| error.as_engine().is_some()) =>
                    {
                        result = Err(fault.into())
                    }
                    _ => {}
                }
                JobStep::Complete(result)
            }
        }
    }
}
enum StepResult {
    Prepared,
    Pending,
    Complete(JsResult<JsValue>),
}
struct Action<'h, 'o> {
    host: &'h HostContext<'o>,
    observer: JobObserver,
    cleanup: Option<(crate::vm::CallCheckpoint, Option<Realm>)>,
    observation: Option<PhaseObservation>,
}
impl<'h, 'o> Action<'h, 'o> {
    fn begin(
        host: &'h HostContext<'o>,
        realm: Option<&Realm>,
        kind: JobAction,
        control: &Rc<Control>,
    ) -> JsResult<Self> {
        let (cleanup, observation) = host.with(|access| {
            let owner = access.into_owner();
            owner.check_execution_stop()?;
            if let Some(fault) = control.observer.fault() {
                return Err(fault.into());
            }
            if let Some(required) = control.owner()
                && required != owner.id()
            {
                return Err(EngineError::JobContextRequired { required }.into());
            }
            if let Some(realm) = realm {
                owner.check_realm(realm)?;
            }
            let registration = if control.owner().is_none() {
                match owner.job_lifetime().register(control.clone()) {
                    Ok(registration) => Some(registration),
                    Err(error) => {
                        let mut error = JsError::from(error);
                        owner.retain_preparation_error(&mut error);
                        return Err(error);
                    }
                }
            } else {
                None
            };
            owner.admit_job_action(kind)?;
            if let Some(registration) = registration {
                control.affinity.set(Some(owner.id()));
                control.registration.set(Some(registration));
            }
            let checkpoint = owner.vm.call_checkpoint();
            let previous = realm
                .map(|realm| owner.enter_realm(realm.clone()))
                .transpose()?;
            let observation = owner.begin_execution_phase(kind.phase());
            if let Err(stop) = owner.check_execution_stop() {
                owner.vm.restore_call(checkpoint);
                if let Some(mut realm) = previous {
                    owner.swap_realm(&mut realm);
                }
                owner.end_execution_phase(observation);
                return Err(stop);
            }
            owner.note_execution_effect();
            Ok(((checkpoint, previous), observation))
        })?;
        Ok(Self {
            host,
            observer: control.observer.clone(),
            cleanup: Some(cleanup),
            observation,
        })
    }
    fn restore(&mut self, owner: &mut Context) {
        if let Some((checkpoint, previous)) = self.cleanup.take() {
            owner.vm.restore_call(checkpoint);
            if let Some(mut realm) = previous {
                owner.swap_realm(&mut realm);
            }
        }
    }
    fn restore_and_check(&mut self) -> Result<JsResult<()>, JobFault> {
        self.host.with_job_cleanup(|access| {
            let owner = access.into_owner();
            self.restore(owner);
            owner.check_execution_stop()
        })
    }
    fn check_stop(&self) -> Result<JsResult<()>, JobFault> {
        self.host
            .with_job_cleanup(|access| access.into_owner().check_execution_stop())
    }
    fn finish(mut self) -> Result<JsResult<()>, JobFault> {
        self.host.with_job_cleanup(|access| {
            let owner = access.into_owner();
            self.restore(owner);
            owner.end_execution_phase(self.observation.take());
            owner.check_execution_stop()
        })
    }
}
impl Drop for Action<'_, '_> {
    fn drop(&mut self) {
        if self.cleanup.is_none() && self.observation.is_none() {
            return;
        }
        if self
            .host
            .with_job_cleanup(|access| {
                let owner = access.into_owner();
                self.restore(owner);
                owner.end_execution_phase(self.observation.take());
            })
            .is_err()
        {
            let fault = self.host.fault().unwrap_or_else(|| {
                JobFault::new(
                    JobFaultOwner::HostContext,
                    JobFaultCause::StateInconsistency,
                )
            });
            self.observer.record_fault(fault);
        }
    }
}
struct TaskWake {
    ready: AtomicBool,
    retired: AtomicBool,
    driver: Mutex<Option<Waker>>,
    observer: JobObserver,
}
impl TaskWake {
    fn new(observer: JobObserver) -> Self {
        Self {
            ready: AtomicBool::new(true),
            retired: AtomicBool::new(false),
            driver: Mutex::new(None),
            observer,
        }
    }
    fn fault(&self, fault: JobFault) {
        let first = self.observer.latch_fault(fault);
        self.observer.record_fault(first);
    }
    fn register(&self, waker: &Waker) {
        if let Some(fault) = self.observer.fault() {
            self.fault(fault);
            return;
        }
        let waker = waker.clone();
        let (old, fault) = match self.driver.lock() {
            Ok(mut slot) => (slot.replace(waker), None),
            Err(poisoned) => {
                let fault = self.observer.latch_fault(JobFault::new(
                    JobFaultOwner::TaskWake,
                    JobFaultCause::PoisonedWaker,
                ));
                let old = poisoned.into_inner().take();
                drop(waker);
                (old, Some(fault))
            }
        };
        if let Some(fault) = fault {
            self.ready.store(true, Ordering::Release);
            let mut cleanup = UnwindCleanup::default();
            cleanup.run(|| {
                self.observer.record_fault(fault);
            });
            cleanup.run(|| {
                if let Some(old) = old {
                    old.wake();
                }
            });
            cleanup.finish();
        } else {
            drop(old);
        }
    }
    fn retire(&self) {
        self.retired.store(true, Ordering::Release);
        let (old, fault) = match self.driver.lock() {
            Ok(mut slot) => (slot.take(), None),
            Err(poisoned) => {
                let fault = self.observer.latch_fault(JobFault::new(
                    JobFaultOwner::TaskWake,
                    JobFaultCause::PoisonedWaker,
                ));
                (poisoned.into_inner().take(), Some(fault))
            }
        };
        if let Some(fault) = fault {
            let mut cleanup = UnwindCleanup::default();
            cleanup.run(|| {
                self.observer.record_fault(fault);
            });
            cleanup.run(|| {
                if let Some(old) = old {
                    old.wake();
                }
            });
            cleanup.finish();
        } else {
            drop(old);
        }
    }
}
impl super::observer::FaultWake for TaskWake {
    fn faulted(&self, fault: JobFault) {
        self.ready.store(true, Ordering::Release);
        let driver = match self.driver.lock() {
            Ok(mut slot) => slot.take(),
            Err(poisoned) => {
                self.observer.latch_fault(fault);
                poisoned.into_inner().take()
            }
        };
        if let Some(driver) = driver {
            driver.wake();
        }
    }
}
impl Wake for TaskWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        if self.retired.load(Ordering::Acquire) {
            return;
        }
        self.ready.store(true, Ordering::Release);
        let (driver, fault) = match self.driver.lock() {
            Ok(mut slot) => (slot.take(), None),
            Err(poisoned) => {
                let fault = self.observer.latch_fault(JobFault::new(
                    JobFaultOwner::TaskWake,
                    JobFaultCause::PoisonedWaker,
                ));
                (poisoned.into_inner().take(), Some(fault))
            }
        };
        let mut cleanup = UnwindCleanup::default();
        if let Some(fault) = fault {
            cleanup.run(|| {
                self.observer.record_fault(fault);
            });
        }
        cleanup.run(|| {
            if let Some(driver) = driver {
                driver.wake();
            }
        });
        cleanup.finish();
    }
}

#[cfg(test)]
mod fault_tests;
