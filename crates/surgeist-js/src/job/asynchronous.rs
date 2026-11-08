//! Owned native task preparation, affinity and action cleanup.
use super::{
    ContextId, ContextTask, JobObserver, JobRetirement, JobStatus,
    identity::{OwnerRegistration, OwnerTask},
};
use crate::{
    Context, ContextAccess, HostContext, JsError, JsResult, JsValue, error::EngineError,
    execution::JobAction, realm::Realm,
};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
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
    /// A later result-free observation of terminal disposition.
    Finished(JobStatus),
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
    state: RefCell<Option<State>>,
    affinity: RefCell<Option<ContextId>>,
    registration: RefCell<Option<OwnerRegistration>>,
    observer: JobObserver,
    delivered: Cell<bool>,
    wake: Arc<TaskWake>,
}
impl OwnerTask for Control {
    fn retire(&self, reason: JobRetirement) {
        self.observer.retire(reason);
        self.wake.retire();
        let state = self.state.borrow_mut().take();
        let registration = self.registration.borrow_mut().take();
        drop((state, registration));
    }
}
impl Drop for Control {
    fn drop(&mut self) {
        self.observer.retire(JobRetirement::Dropped);
        self.wake.retire();
        drop(self.state.get_mut().take());
        drop(self.registration.get_mut().take());
    }
}
struct StateLease {
    control: Rc<Control>,
    state: Option<State>,
}
impl Drop for StateLease {
    fn drop(&mut self) {
        if self.control.observer.status() == JobStatus::Pending {
            *self.control.state.borrow_mut() = self.state.take();
        }
    }
}

pub(super) struct AsyncIdentity(Weak<Control>);
impl AsyncIdentity {
    pub(super) fn owner(&self) -> Option<ContextId> {
        self.0
            .upgrade()
            .and_then(|control| control.affinity.borrow().clone())
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
        Self {
            control: Rc::new(Control {
                state: RefCell::new(Some(State::Factory(Box::new(factory)))),
                affinity: RefCell::new(None),
                registration: RefCell::new(None),
                observer: JobObserver::new(),
                delivered: Cell::new(false),
                wake: Arc::new(TaskWake::new()),
            }),
            realm: None,
        }
    }
    /// Creates an owned request with a selected execution realm.
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
        self.control.affinity.borrow().clone()
    }
    pub(crate) fn is_ready(&self) -> bool {
        self.control.wake.ready.load(Ordering::Acquire)
    }
    pub(crate) fn register_driver(&self, waker: &Waker) {
        self.control.wake.register(waker);
    }
    pub(crate) fn is_prepared(&self) -> bool {
        matches!(&*self.control.state.borrow(), Some(State::Task(_)))
    }

    /// Performs exactly one preparation or task poll with temporary engine access.
    ///
    /// A before-action refusal returns Deferred and retains the same resumable
    /// state. Preparation binds the exact Context before dispatching the factory.
    /// Complete moves its terminal result once; later calls return result-free
    /// Finished status without another admission or dispatch.
    pub fn run_step(&mut self, cx: &mut TaskContext<'_>, host: &HostContext<'_>) -> JobStep {
        let status = self.control.observer.status();
        if status != JobStatus::Pending {
            if self.control.delivered.replace(true) {
                return JobStep::Finished(status);
            }
            return JobStep::Complete(Err(match status {
                JobStatus::Retired(reason) => EngineError::JobRetired(reason).into(),
                JobStatus::Completed => unreachable!("ordinary completion delivered its result"),
                JobStatus::Pending => unreachable!("checked terminal state"),
            }));
        }
        self.control.wake.register(cx.waker());
        let state = self.control.state.borrow_mut().take();
        let Some(state) = state else {
            return JobStep::Deferred(EngineError::ContextAccessBusy.into());
        };
        let mut lease = StateLease {
            control: self.control.clone(),
            state: Some(state),
        };
        let action_kind = match lease.state {
            Some(State::Factory(_)) => JobAction::Preparation,
            Some(State::Task(_)) => JobAction::Poll,
            None => unreachable!(),
        };
        let mut action = match Action::begin(host, self.realm.as_ref(), action_kind, &self.control)
        {
            Ok(action) => action,
            Err(error) => return JobStep::Deferred(error),
        };
        let dispatched = catch_unwind(AssertUnwindSafe(|| {
            match lease.state.take().expect("lease owns task state") {
                State::Factory(factory) => match host.with(factory) {
                    Ok(task) => {
                        lease.state = Some(State::Task(task));
                        StepResult::Prepared
                    }
                    Err(error) => StepResult::Complete(Err(error)),
                },
                State::Task(mut task) => {
                    self.control.wake.ready.store(false, Ordering::Release);
                    let waker = Waker::from(self.control.wake.clone());
                    let result = task.poll(&mut TaskContext::from_waker(&waker), host);
                    lease.state = Some(State::Task(task));
                    match result {
                        Poll::Pending => StepResult::Pending,
                        Poll::Ready(result) => StepResult::Complete(result),
                    }
                }
            }
        }));
        let mut result = match dispatched {
            Ok(result) => result,
            Err(payload) => {
                drop(lease.state.take());
                drop(action);
                self.control.observer.retire(JobRetirement::Interrupted);
                resume_unwind(payload)
            }
        };
        let stopped = action.restore_and_check().err();
        let retired = match self.control.observer.status() {
            JobStatus::Retired(reason) => Some(reason),
            _ => None,
        };
        if stopped.is_some() || retired.is_some() || matches!(result, StepResult::Complete(_)) {
            drop(lease.state.take());
        }
        let stopped = action.check_stop().err().or(stopped);
        if let Some(stop) = stopped {
            drop(result);
            result = StepResult::Complete(Err(stop));
        } else if let Some(reason) = retired {
            drop(result);
            result = StepResult::Complete(Err(EngineError::JobRetired(reason).into()));
        }
        if let Err(stop) = action.finish() {
            drop(lease.state.take());
            drop(result);
            result = StepResult::Complete(Err(stop));
        }
        match result {
            StepResult::Prepared => {
                self.control.wake.ready.store(true, Ordering::Release);
                JobStep::Prepared
            }
            StepResult::Pending => JobStep::Pending,
            StepResult::Complete(result) => {
                if result
                    .as_ref()
                    .is_err_and(|error| error.as_engine().is_some())
                {
                    self.control.observer.retire(JobRetirement::Interrupted);
                } else {
                    self.control.observer.complete();
                }
                drop(self.control.registration.borrow_mut().take());
                self.control.delivered.set(true);
                self.control.wake.retire();
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
            if let Some(required) = control.affinity.borrow().as_ref()
                && *required != owner.id()
            {
                return Err(EngineError::JobContextRequired {
                    required: required.clone(),
                }
                .into());
            }
            if let Some(realm) = realm {
                owner.check_realm(realm)?;
            }
            let registration = if control.affinity.borrow().is_none() {
                match owner.job_lifetime().register(control.clone()) {
                    Ok(registration) => Some(registration),
                    Err(mut error) => {
                        owner.retain_preparation_error(&mut error);
                        return Err(error);
                    }
                }
            } else {
                None
            };
            owner.admit_job_action(kind)?;
            if let Some(registration) = registration {
                *control.affinity.borrow_mut() = Some(owner.id());
                *control.registration.borrow_mut() = Some(registration);
            }
            let checkpoint = owner.vm.call_checkpoint();
            let previous = realm
                .map(|realm| owner.enter_realm(realm.clone()))
                .transpose()?;
            let observation = owner.begin_execution_phase(kind.phase());
            owner.note_execution_effect();
            Ok(((checkpoint, previous), observation))
        })?;
        Ok(Self {
            host,
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
    fn restore_and_check(&mut self) -> JsResult<()> {
        self.host.with(|access| {
            let owner = access.into_owner();
            self.restore(owner);
            owner.check_execution_stop()
        })
    }
    fn check_stop(&self) -> JsResult<()> {
        self.host
            .with(|access| access.into_owner().check_execution_stop())
    }
    fn finish(mut self) -> JsResult<()> {
        self.host.with(|access| {
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
        self.host
            .with(|access| {
                let owner = access.into_owner();
                self.restore(owner);
                owner.end_execution_phase(self.observation.take());
                Ok(())
            })
            .expect("admitted action released all temporary host leases");
    }
}

struct TaskWake {
    ready: AtomicBool,
    retired: AtomicBool,
    driver: Mutex<Option<Waker>>,
}
impl TaskWake {
    fn new() -> Self {
        Self {
            ready: AtomicBool::new(true),
            retired: AtomicBool::new(false),
            driver: Mutex::new(None),
        }
    }
    fn register(&self, waker: &Waker) {
        let waker = waker.clone();
        let old = self
            .driver
            .lock()
            .expect("task wake mutex poisoned")
            .replace(waker);
        drop(old);
    }
    fn retire(&self) {
        self.retired.store(true, Ordering::Release);
        let old = self.driver.lock().expect("task wake mutex poisoned").take();
        drop(old);
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
        let driver = self.driver.lock().expect("task wake mutex poisoned").take();
        if let Some(driver) = driver {
            driver.wake();
        }
    }
}
