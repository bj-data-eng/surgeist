//! Native settlement custody, independent of guest reactions and task results.

use crate::{
    Context, JsResult,
    error::{EngineResourceError, PanicError},
    job::{JobObserver, JobRetirement, RetirementRegistration, RetirementRelay},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak as LocalWeak},
    sync::{
        Arc, Mutex, MutexGuard, OnceLock, Weak,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js_gc::heap::{RecordRef, Rooted, Trace};

fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn reserve<T>(values: &mut Vec<T>) -> JsResult<()> {
    values
        .try_reserve(1)
        .map_err(|_| EngineResourceError::Allocation.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettlementStatus {
    Pending,
    Settled,
    Retired(JobRetirement),
    ModuleInterrupted,
}

struct Waiter {
    live: bool,
    waker: Option<Waker>,
}
struct SignalState {
    module_epoch: Option<Arc<Epoch>>,
    status: SettlementStatus,
    waiters: Vec<Waiter>,
    dependents: Vec<Option<Weak<Dependency>>>,
}
struct NoticeQueue {
    head: Option<Arc<Signal>>,
    tail: Option<Arc<Signal>>,
    wake_head: Option<Arc<Signal>>,
    wake_tail: Option<Arc<Signal>>,
    driving: bool,
}
struct Domain {
    queue: Mutex<NoticeQueue>,
    #[cfg(test)]
    empty_notice_hook: Option<EmptyNoticeHook>,
}
#[cfg(test)]
struct EmptyNoticeHook {
    visits: std::sync::atomic::AtomicUsize,
    reached: std::sync::mpsc::Sender<()>,
    resume: Mutex<std::sync::mpsc::Receiver<()>>,
}
#[cfg(test)]
impl Domain {
    fn after_empty_notice(&self) {
        if let Some(hook) = &self.empty_notice_hook
            && hook.visits.fetch_add(1, Ordering::SeqCst) == 1
        {
            hook.reached
                .send(())
                .expect("second retirement thread is alive");
            lock(&hook.resume)
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("concurrent retirement must return while the first driver owns the domain");
        }
    }
}
fn domain() -> &'static Domain {
    static DOMAIN: OnceLock<Domain> = OnceLock::new();
    DOMAIN.get_or_init(|| Domain {
        queue: Mutex::new(NoticeQueue {
            head: None,
            tail: None,
            wake_head: None,
            wake_tail: None,
            driving: false,
        }),
        #[cfg(test)]
        empty_notice_hook: None,
    })
}

/// Status and prepared notification links only; never owns engine values.
pub(crate) struct Signal {
    #[cfg(test)]
    isolated_domain: Option<Arc<Domain>>,
    notified: AtomicBool,
    module_notified: AtomicBool,
    module_next: Mutex<Option<Arc<Signal>>>,
    state: Mutex<SignalState>,
    next: Mutex<Option<Arc<Signal>>>,
}
impl std::fmt::Debug for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeSettlementSignal")
            .field(&self.status())
            .finish()
    }
}
impl Signal {
    fn new() -> Arc<Self> {
        let _ = domain();
        Arc::new(Self {
            #[cfg(test)]
            isolated_domain: None,
            notified: AtomicBool::new(false),
            module_notified: AtomicBool::new(false),
            module_next: Mutex::new(None),
            state: Mutex::new(SignalState {
                module_epoch: None,
                status: SettlementStatus::Pending,
                waiters: Vec::new(),
                dependents: Vec::new(),
            }),
            next: Mutex::new(None),
        })
    }
    fn notification_domain(&self) -> &Domain {
        #[cfg(test)]
        if let Some(domain) = &self.isolated_domain {
            return domain;
        }
        domain()
    }
    #[cfg(test)]
    fn in_isolated_domain(domain: Arc<Domain>) -> Arc<Self> {
        let mut signal = Self::new();
        Arc::get_mut(&mut signal)
            .expect("new signal is uniquely owned")
            .isolated_domain = Some(domain);
        signal
    }
    pub(crate) fn status(&self) -> SettlementStatus {
        lock(&self.state).status
    }
    fn retire_pending(self: &Arc<Self>, reason: JobRetirement) {
        {
            let mut state = lock(&self.state);
            if state.status != SettlementStatus::Pending {
                return;
            }
            state.status = SettlementStatus::Retired(reason);
        }
        self.notify();
    }
    fn retire_epoch(self: &Arc<Self>, epoch: &Epoch, reason: JobRetirement) {
        {
            let mut state = lock(&self.state);
            if state.status != SettlementStatus::Pending || !epoch.active.load(Ordering::Acquire) {
                return;
            }
            state.status = SettlementStatus::Retired(reason);
        }
        self.notify();
    }
    fn notify(self: &Arc<Self>) {
        match self.status() {
            SettlementStatus::Retired(_) if !self.notified.swap(true, Ordering::AcqRel) => {
                self.notice()
            }
            SettlementStatus::Retired(_) => {}
            SettlementStatus::ModuleInterrupted => self.notify_module(),
            _ => self.wake(),
        }
    }
    pub(crate) fn settle(self: &Arc<Self>) {
        {
            let mut state = lock(&self.state);
            if state.status == SettlementStatus::ModuleInterrupted {
                return;
            }
            state.status = SettlementStatus::Settled;
        }
        self.wake();
    }
    fn wake(&self) {
        // Slots are reserved by observers. No Waker runs or is dropped under a lock.
        let count = lock(&self.state).waiters.len();
        let mut panic = None;
        for index in 0..count {
            let waker = lock(&self.state)
                .waiters
                .get_mut(index)
                .and_then(|slot| slot.waker.take());
            if let Some(waker) = waker
                && let Err(payload) =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| waker.wake()))
                && panic.is_none()
            {
                panic = Some(payload);
            }
        }
        if let Some(panic) = panic {
            std::panic::resume_unwind(panic);
        }
    }
    pub(crate) fn mark_module(&self) {
        let mut state = lock(&self.state);
        if state.status != SettlementStatus::Settled {
            state.status = SettlementStatus::ModuleInterrupted;
        }
    }
    fn mark_module_epoch(self: &Arc<Self>, epoch: &Arc<Epoch>) {
        let changed = {
            let mut state = lock(&self.state);
            if !epoch.active.load(Ordering::Acquire)
                || matches!(
                    state.status,
                    SettlementStatus::Settled | SettlementStatus::ModuleInterrupted
                )
            {
                false
            } else {
                state.status = SettlementStatus::ModuleInterrupted;
                state.module_epoch = Some(epoch.clone());
                true
            }
        };
        if changed {
            self.notify_module();
        }
    }
    pub(crate) fn notify_module(self: &Arc<Self>) {
        if self.status() != SettlementStatus::ModuleInterrupted
            || self.module_notified.swap(true, Ordering::AcqRel)
        {
            return;
        }
        static MODULE_DOMAIN: OnceLock<Domain> = OnceLock::new();
        let domain = MODULE_DOMAIN.get_or_init(|| Domain {
            queue: Mutex::new(NoticeQueue {
                head: None,
                tail: None,
                wake_head: None,
                wake_tail: None,
                driving: false,
            }),
            #[cfg(test)]
            empty_notice_hook: None,
        });
        self.notice_in(domain, true);
    }
    fn link(&self, module: bool) -> &Mutex<Option<Arc<Signal>>> {
        if module {
            &self.module_next
        } else {
            &self.next
        }
    }
    fn notice(self: &Arc<Self>) {
        self.notice_in(self.notification_domain(), false);
    }
    fn notice_in(self: &Arc<Self>, domain: &Domain, module: bool) {
        let drive = {
            let mut queue = lock(&domain.queue);
            if let Some(tail) = queue.tail.replace(self.clone()) {
                *lock(tail.link(module)) = Some(self.clone());
            } else {
                queue.head = Some(self.clone());
            }
            if queue.driving {
                false
            } else {
                queue.driving = true;
                true
            }
        };
        if !drive {
            return;
        }
        enum Work {
            Notice(Arc<Signal>),
            Wake(Arc<Signal>),
            Idle,
        }
        let mut panic = None;
        loop {
            // The test forces an arrival after an optimistic empty observation.
            // The authoritative selection below must see that arrival atomically.
            #[cfg(test)]
            {
                let observed_empty = lock(&domain.queue).head.is_none();
                if observed_empty {
                    domain.after_empty_notice();
                }
            }
            let work = {
                let mut queue = lock(&domain.queue);
                if let Some(next) = queue.head.take() {
                    queue.head = lock(next.link(module)).take();
                    if queue.head.is_none() {
                        queue.tail = None;
                    }
                    Work::Notice(next)
                } else if let Some(wake) = queue.wake_head.take() {
                    queue.wake_head = lock(wake.link(module)).take();
                    if queue.wake_head.is_none() {
                        queue.wake_tail = None;
                    }
                    Work::Wake(wake)
                } else {
                    queue.driving = false;
                    Work::Idle
                }
            };
            match work {
                Work::Notice(next) => {
                    let count = lock(&next.state).dependents.len();
                    for index in 0..count {
                        let dependent = lock(&next.state)
                            .dependents
                            .get(index)
                            .and_then(Clone::clone)
                            .and_then(|value| value.upgrade());
                        if let Some(dependent) = dependent {
                            if module {
                                dependent.module_interrupt();
                            } else if let SettlementStatus::Retired(reason) = next.status() {
                                dependent.retire(reason);
                            }
                        }
                    }
                    let mut queue = lock(&domain.queue);
                    if let Some(tail) = queue.wake_tail.replace(next.clone()) {
                        *lock(tail.link(module)) = Some(next);
                    } else {
                        queue.wake_head = Some(next);
                    }
                }
                Work::Wake(wake) => {
                    // Notice work has priority: all currently reachable dependents
                    // are marked before arbitrary native Wakers run.
                    if let Err(payload) =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake.wake()))
                        && panic.is_none()
                    {
                        panic = Some(payload);
                    }
                }
                Work::Idle => break,
            }
        }
        if let Some(panic) = panic {
            std::panic::resume_unwind(panic);
        }
    }
    pub(crate) fn wait(self: &Arc<Self>) -> JsResult<SettlementWait> {
        let index = {
            let mut state = lock(&self.state);
            if let Some(index) = state.waiters.iter().position(|slot| !slot.live) {
                state.waiters[index].live = true;
                index
            } else {
                reserve(&mut state.waiters)?;
                let index = state.waiters.len();
                state.waiters.push(Waiter {
                    live: true,
                    waker: None,
                });
                index
            }
        };
        Ok(SettlementWait {
            signal: self.clone(),
            index,
        })
    }
    fn subscribe(
        self: &Arc<Self>,
        target: &Arc<Signal>,
        epoch: Arc<Epoch>,
    ) -> JsResult<DependencyGuard> {
        let dependency = Arc::new(Dependency {
            target: Arc::downgrade(target),
            epoch,
        });
        let index = {
            let mut state = lock(&self.state);
            if let Some(index) = state.dependents.iter().position(Option::is_none) {
                state.dependents[index] = Some(Arc::downgrade(&dependency));
                index
            } else {
                reserve(&mut state.dependents)?;
                let index = state.dependents.len();
                state.dependents.push(Some(Arc::downgrade(&dependency)));
                index
            }
        };
        if let SettlementStatus::Retired(reason) = self.status() {
            dependency.retire(reason);
        }
        if self.status() == SettlementStatus::ModuleInterrupted {
            dependency.module_interrupt();
        }
        Ok(DependencyGuard {
            source: Arc::downgrade(self),
            index,
            _dependency: dependency,
        })
    }
}
pub(crate) struct SettlementWait {
    signal: Arc<Signal>,
    index: usize,
}
impl SettlementWait {
    pub(crate) fn poll(&mut self, cx: &mut TaskContext<'_>) -> Poll<SettlementStatus> {
        let status = self.signal.status();
        if status != SettlementStatus::Pending {
            return Poll::Ready(status);
        }
        let new_waker = cx.waker().clone();
        let (status, old) = {
            let mut state = lock(&self.signal.state);
            let status = state.status;
            let old = state.waiters[self.index].waker.replace(new_waker);
            (status, old)
        };
        drop(old);
        if status == SettlementStatus::Pending {
            Poll::Pending
        } else {
            Poll::Ready(status)
        }
    }
}
impl Drop for SettlementWait {
    fn drop(&mut self) {
        let old = {
            let mut state = lock(&self.signal.state);
            let slot = &mut state.waiters[self.index];
            slot.live = false;
            slot.waker.take()
        };
        drop(old);
    }
}
struct Epoch {
    module_pending: AtomicBool,
    active: AtomicBool,
    pending: Mutex<Option<JobRetirement>>,
    completed: AtomicBool,
}
impl Epoch {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            module_pending: AtomicBool::new(false),
            active: AtomicBool::new(false),
            pending: Mutex::new(None),
            completed: AtomicBool::new(false),
        })
    }
}
struct Relay {
    target: Weak<Signal>,
    epoch: Arc<Epoch>,
}
impl RetirementRelay for Relay {
    fn retire(&self, reason: JobRetirement) {
        *lock(&self.epoch.pending) = Some(reason);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.retire_epoch(&self.epoch, reason);
        }
    }
    fn completed(&self) {
        if let Some(target) = self.target.upgrade() {
            let _state = lock(&target.state);
            self.epoch.completed.store(true, Ordering::Release);
            self.epoch.active.store(false, Ordering::Release);
        }
    }
}
struct Dependency {
    target: Weak<Signal>,
    epoch: Arc<Epoch>,
}
impl Dependency {
    fn module_interrupt(&self) {
        self.epoch.module_pending.store(true, Ordering::Release);
        if let Some(target) = self.target.upgrade() {
            target.mark_module_epoch(&self.epoch);
        }
    }
    fn retire(&self, reason: JobRetirement) {
        *lock(&self.epoch.pending) = Some(reason);
        if self.epoch.active.load(Ordering::Acquire)
            && let Some(target) = self.target.upgrade()
        {
            target.retire_epoch(&self.epoch, reason);
        }
    }
}
struct DependencyGuard {
    source: Weak<Signal>,
    index: usize,
    _dependency: Arc<Dependency>,
}
impl Drop for DependencyGuard {
    fn drop(&mut self) {
        if let Some(source) = self.source.upgrade() {
            let mut state = lock(&source.state);
            if state.dependents[self.index]
                .as_ref()
                .is_some_and(|entry| entry.ptr_eq(&Arc::downgrade(&self._dependency)))
            {
                state.dependents[self.index] = None;
            }
        }
    }
}
struct Binding {
    epoch: Arc<Epoch>,
    _job: Option<RetirementRegistration>,
    _dependency: Option<DependencyGuard>,
}
impl Drop for Binding {
    fn drop(&mut self) {
        self.epoch.active.store(false, Ordering::Release);
    }
}

pub(crate) struct PreparedHandoff {
    owner: Rooted<Association>,
    binding: Binding,
    dependency: Option<AssociationEdge>,
}
impl PreparedHandoff {
    pub(crate) fn commit(self) -> Rooted<Association> {
        let Self {
            owner,
            binding,
            dependency,
        } = self;
        if owner.module_cause().is_some() {
            owner.signal.mark_module();
        }
        let epoch = binding.epoch.clone();
        let mut candidate = Some((binding, dependency));
        let previous = {
            let mut bindings = owner.bindings.borrow_mut();
            let mut local_dependency = owner.view.dependency.borrow_mut();
            let mut state = lock(&owner.signal.state);
            if state.status == SettlementStatus::ModuleInterrupted {
                None
            } else {
                let (binding, dependency) = candidate.take().expect("prepared handoff");
                let old = bindings.primary.replace(binding);
                if let Some(old) = &old {
                    old.epoch.active.store(false, Ordering::Release);
                }
                let old_dependency = std::mem::replace(&mut *local_dependency, dependency);
                let active = !epoch.completed.load(Ordering::Acquire);
                epoch.active.store(active, Ordering::Release);
                if active && epoch.module_pending.load(Ordering::Acquire) {
                    state.status = SettlementStatus::ModuleInterrupted;
                    state.module_epoch = Some(epoch.clone());
                } else if active
                    && state.status == SettlementStatus::Pending
                    && let Some(reason) = *lock(&epoch.pending)
                {
                    state.status = SettlementStatus::Retired(reason);
                }
                Some((old, old_dependency))
            }
        };
        drop(previous);
        drop(candidate);
        owner
    }
}

pub(crate) struct ResolutionLease(Option<Arc<Epoch>>);

#[derive(Default, Trace)]
struct AssociationView {
    #[heap_trace(skip)]
    completes_origin: Cell<bool>,
    origin: RefCell<Option<crate::module::lifecycle::EvaluationRef>>,
    dependency: RefCell<Option<AssociationEdge>>,
}
#[derive(Trace)]
pub(crate) struct Association {
    #[heap_trace(skip)]
    signal: Arc<Signal>,
    #[heap_trace(skip)]
    bindings: RefCell<Bindings>,
    view: Rc<AssociationView>,
}
struct Bindings {
    primary: Option<Binding>,
    required: Vec<Binding>,
}
impl std::fmt::Debug for Association {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NativeSettlementAssociation")
            .field(&self.signal)
            .finish()
    }
}
#[derive(Clone, Debug, Trace)]
pub(crate) struct AssociationEdge(
    RecordRef<Association>,
    #[heap_trace(skip)] LocalWeak<AssociationView>,
    #[heap_trace(skip)] Arc<Signal>,
);
impl AssociationEdge {
    pub(crate) fn root(&self) -> Rooted<Association> {
        self.0
            .upgrade()
            .expect("native association heap available")
            .expect("traced owner retains native association")
    }
}
impl Association {
    pub(crate) fn attach_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        self.view.completes_origin.set(true);
        *self.view.origin.borrow_mut() = Some(evaluation);
    }
    pub(crate) fn bind_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        let mut origin = self.view.origin.borrow_mut();
        if origin.is_none() {
            *origin = Some(evaluation);
        }
    }
    pub(crate) fn observe_module(&self, evaluation: crate::module::lifecycle::EvaluationRef) {
        self.bind_module(evaluation);
        if self.module_cause().is_some() {
            self.signal.mark_module();
            self.signal.notify_module();
        }
    }
    fn next_view(view: &Rc<AssociationView>) -> Option<(Rc<AssociationView>, Arc<Signal>)> {
        let next = view.dependency.borrow().clone()?;
        Some((next.1.upgrade()?, next.2.clone()))
    }
    pub(crate) fn module_cause(&self) -> Option<Rc<crate::error::RetainedInterruption>> {
        let mut cursor = (self.view.clone(), self.signal.clone());
        let mut slow = Some(self.view.clone());
        let mut fast = Some(self.view.clone());
        let mut collision = None;
        loop {
            if cursor.1.status() == SettlementStatus::Settled {
                return None;
            }
            if let Some(origin) = cursor.0.origin.borrow().as_ref()
                && let Some(cause) = origin.cause()
            {
                return Some(cause);
            }
            cursor = Self::next_view(&cursor.0)?;
            if let Some(collision) = &collision {
                if Rc::ptr_eq(&cursor.0, collision) {
                    return None;
                }
                continue;
            }
            slow = slow.and_then(|view| Self::next_view(&view).map(|next| next.0));
            fast = fast
                .and_then(|view| Self::next_view(&view))
                .and_then(|next| Self::next_view(&next.0))
                .map(|next| next.0);
            if let (Some(slow), Some(fast)) = (&slow, &fast)
                && Rc::ptr_eq(slow, fast)
            {
                // A collision can precede an unvisited origin. Inspect one full
                // lap from this cursor before concluding the cycle has no cause.
                collision = Some(cursor.0.clone());
            }
        }
    }
    pub(crate) fn module_error(owner: &Rooted<Self>) -> Option<crate::JsError> {
        owner.module_cause().map(|cause| {
            crate::error::EngineError::ModuleInterrupted(
                crate::error::ModuleInterruption::association(cause, owner.clone()),
            )
            .into()
        })
    }
    pub(crate) fn new(context: &Context) -> JsResult<Rooted<Self>> {
        context
            .runtime()
            .heap
            .allocate(Self {
                signal: Signal::new(),
                bindings: RefCell::new(Bindings {
                    primary: None,
                    required: Vec::new(),
                }),
                view: Rc::new(AssociationView::default()),
            })
            .map_err(|error| {
                PanicError::new(format!(
                    "native settlement association publication: {}",
                    error.reason
                ))
                .into()
            })
    }
    pub(crate) fn edge(owner: &Rooted<Self>) -> AssociationEdge {
        AssociationEdge(
            owner.record_ref(),
            Rc::downgrade(&owner.view),
            owner.signal.clone(),
        )
    }
    pub(crate) fn signal(&self) -> Arc<Signal> {
        self.signal.clone()
    }
    pub(crate) fn prepare_handoff(
        owner: &Rooted<Self>,
        observer: &JobObserver,
    ) -> JsResult<PreparedHandoff> {
        let epoch = Epoch::new();
        let relay: Arc<dyn RetirementRelay> = Arc::new(Relay {
            target: Arc::downgrade(&owner.signal),
            epoch: epoch.clone(),
        });
        let registration = observer.register_retirement(relay)?;
        Ok(PreparedHandoff {
            owner: owner.clone(),
            binding: Binding {
                epoch,
                _job: Some(registration),
                _dependency: None,
            },
            dependency: None,
        })
    }
    pub(crate) fn prepare_follow(
        owner: &Rooted<Self>,
        source: &Rooted<Self>,
    ) -> JsResult<PreparedHandoff> {
        let epoch = Epoch::new();
        let subscription = source.signal.subscribe(&owner.signal, epoch.clone())?;
        Ok(PreparedHandoff {
            owner: owner.clone(),
            binding: Binding {
                epoch,
                _job: None,
                _dependency: Some(subscription),
            },
            dependency: Some(Self::edge(source)),
        })
    }
    pub(crate) fn resolution_lease(&self) -> ResolutionLease {
        ResolutionLease(
            self.bindings
                .borrow()
                .primary
                .as_ref()
                .map(|binding| binding.epoch.clone()),
        )
    }
    pub(crate) fn interrupt_resolution(&self, expected: ResolutionLease) {
        let notify = {
            let bindings = self.bindings.borrow();
            let current = bindings.primary.as_ref().map(|binding| &binding.epoch);
            let same = match (current, expected.0.as_ref()) {
                (None, None) => true,
                (Some(current), Some(expected)) => Arc::ptr_eq(current, expected),
                _ => false,
            };
            let mut state = lock(&self.signal.state);
            if same && state.status == SettlementStatus::Pending {
                state.status = SettlementStatus::Retired(JobRetirement::Interrupted);
                true
            } else {
                false
            }
        };
        if notify {
            self.signal.notify();
        }
    }
    pub(crate) fn retire_pending(&self, reason: JobRetirement) {
        self.signal.retire_pending(reason);
    }
    pub(crate) fn notify(&self) {
        self.signal.notify();
    }
    pub(crate) fn bind_job(&self, observer: &JobObserver, replace: bool) -> JsResult<()> {
        let epoch = Epoch::new();
        let relay: Arc<dyn RetirementRelay> = Arc::new(Relay {
            target: Arc::downgrade(&self.signal),
            epoch: epoch.clone(),
        });
        let registration = observer.register_retirement(relay)?;
        let binding = Binding {
            epoch: epoch.clone(),
            _job: Some(registration),
            _dependency: None,
        };
        self.commit(binding, replace)?;
        if replace && self.signal.status() != SettlementStatus::ModuleInterrupted {
            self.view.dependency.borrow_mut().take();
        }
        if let Some(reason) = *lock(&epoch.pending) {
            self.signal.retire_epoch(&epoch, reason);
        }
        if matches!(observer.status(), crate::job::JobStatus::Completed) {
            epoch.active.store(false, Ordering::Release);
        }
        self.signal.notify();
        Ok(())
    }
    fn commit(&self, binding: Binding, replace: bool) -> JsResult<()> {
        let epoch = binding.epoch.clone();
        let old = {
            let mut bindings = self.bindings.borrow_mut();
            if !replace
                && !bindings
                    .required
                    .iter()
                    .any(|slot| !slot.epoch.active.load(Ordering::Acquire))
            {
                reserve(&mut bindings.required)?;
            }
            let _signal = lock(&self.signal.state);
            let old = if replace {
                bindings.primary.replace(binding)
            } else if let Some(slot) = bindings
                .required
                .iter_mut()
                .find(|slot| !slot.epoch.active.load(Ordering::Acquire))
            {
                Some(std::mem::replace(slot, binding))
            } else {
                bindings.required.push(binding);
                None
            };
            if let Some(old) = &old {
                old.epoch.active.store(false, Ordering::Release);
            }
            epoch
                .active
                .store(!epoch.completed.load(Ordering::Acquire), Ordering::Release);
            old
        };
        drop(old);
        Ok(())
    }
    pub(crate) fn settle(&self) {
        if self.module_cause().is_some()
            || self.signal.status() == SettlementStatus::ModuleInterrupted
        {
            self.signal.mark_module();
            self.signal.notify_module();
            return;
        }
        if self.view.completes_origin.get()
            && let Some(origin) = self.view.origin.borrow().as_ref()
        {
            origin.complete();
        }
        let bindings = std::mem::replace(
            &mut *self.bindings.borrow_mut(),
            Bindings {
                primary: None,
                required: Vec::new(),
            },
        );
        self.view.dependency.borrow_mut().take();
        self.signal.settle();
        drop(bindings);
    }
}

#[cfg(test)]
mod module_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::atomic::AtomicUsize, task::Wake};

    #[derive(Default)]
    struct WakeCount(AtomicUsize, Mutex<()>, std::sync::Condvar);
    impl WakeCount {
        fn signal(&self) {
            let _guard = lock(&self.1);
            self.0.fetch_add(1, Ordering::SeqCst);
            self.2.notify_all();
        }
        fn wait_for_fresh_wake(&self, previous: usize) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            let mut guard = lock(&self.1);
            while self.0.load(Ordering::SeqCst) <= previous {
                let now = std::time::Instant::now();
                assert!(
                    now < deadline,
                    "registered native waiter was not woken before its deadline; last count {}",
                    self.0.load(Ordering::SeqCst)
                );
                let (next, _) = self
                    .2
                    .wait_timeout(guard, deadline.saturating_duration_since(now))
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard = next;
            }
        }
    }
    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.signal();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.signal();
        }
    }
    fn pending(wait: &mut SettlementWait, count: &Arc<WakeCount>) {
        let waker = Waker::from(count.clone());
        assert_eq!(
            wait.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Pending
        );
    }

    #[test]
    fn abandoned_successor_preparation_keeps_the_predecessor_authoritative() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let successor = JobObserver::new();
        association.bind_job(&first, true).unwrap();
        let prepared = Association::prepare_handoff(&association, &successor).unwrap();
        successor.retire(JobRetirement::Dropped);
        assert_eq!(association.signal.status(), SettlementStatus::Pending);
        drop(prepared);
        first.retire(JobRetirement::OwnerDropped);
        assert_eq!(
            association.signal.status(),
            SettlementStatus::Retired(JobRetirement::OwnerDropped)
        );
    }

    #[test]
    fn committed_successor_retires_existing_waiters_and_ignores_the_old_epoch() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let successor = JobObserver::new();
        association.bind_job(&first, true).unwrap();
        let count = Arc::new(WakeCount::default());
        let mut wait = association.signal.wait().unwrap();
        pending(&mut wait, &count);
        let notification = Association::prepare_handoff(&association, &successor)
            .unwrap()
            .commit();
        notification.notify();
        count.wait_for_fresh_wake(0);
        assert!(count.0.load(Ordering::SeqCst) > 0);
        pending(&mut wait, &count);
        count.0.store(0, Ordering::SeqCst);
        first.retire(JobRetirement::OwnerDropped);
        assert_eq!(association.signal.status(), SettlementStatus::Pending);
        assert_eq!(count.0.load(Ordering::SeqCst), 0);
        successor.retire(JobRetirement::Cancelled);
        count.wait_for_fresh_wake(0);
        assert!(count.0.load(Ordering::SeqCst) > 0);
        let waker = Waker::from(count);
        assert_eq!(
            wait.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Ready(SettlementStatus::Retired(JobRetirement::Cancelled))
        );
    }

    #[test]
    fn completing_one_required_child_does_not_release_another_child() {
        let context = Context::default();
        let association = Association::new(&context).unwrap();
        let first = JobObserver::new();
        let second = JobObserver::new();
        association.bind_job(&first, false).unwrap();
        association.bind_job(&second, false).unwrap();
        first.complete();
        assert_eq!(association.signal.status(), SettlementStatus::Pending);
        second.retire(JobRetirement::OwnerDropped);
        assert_eq!(
            association.signal.status(),
            SettlementStatus::Retired(JobRetirement::OwnerDropped)
        );
        assert_eq!(first.status(), crate::job::JobStatus::Completed);
    }

    struct CheckDescendant {
        last: Arc<Signal>,
        observed: AtomicBool,
    }
    impl Wake for CheckDescendant {
        fn wake(self: Arc<Self>) {
            assert_eq!(
                self.last.status(),
                SettlementStatus::Retired(JobRetirement::Interrupted)
            );
            self.observed.store(true, Ordering::SeqCst);
        }
    }
    #[test]
    fn retirement_marks_the_complete_dependency_chain_before_waking() {
        let context = Context::default();
        let chain: Vec<_> = (0..1024)
            .map(|_| Association::new(&context).unwrap())
            .collect();
        for pair in chain.windows(2) {
            drop(
                Association::prepare_follow(&pair[1], &pair[0])
                    .unwrap()
                    .commit(),
            );
        }
        let wake = Arc::new(CheckDescendant {
            last: chain.last().unwrap().signal(),
            observed: AtomicBool::new(false),
        });
        let mut waiter = chain[0].signal.wait().unwrap();
        let waker = Waker::from(wake.clone());
        assert_eq!(
            waiter.poll(&mut TaskContext::from_waker(&waker)),
            Poll::Pending
        );
        chain[0].retire_pending(JobRetirement::Interrupted);
        assert!(wake.observed.load(Ordering::SeqCst));
        assert!(
            chain.iter().all(|value| value.signal.status()
                == SettlementStatus::Retired(JobRetirement::Interrupted))
        );
    }

    #[test]
    fn local_dependency_cycles_are_collectable_even_with_native_signals_alive() {
        let context = Context::default();
        let left = Association::new(&context).unwrap();
        let right = Association::new(&context).unwrap();
        let left_edge = Association::edge(&left);
        let right_edge = Association::edge(&right);
        let signals = (left.signal(), right.signal());
        drop(Association::prepare_follow(&left, &right).unwrap().commit());
        drop(Association::prepare_follow(&right, &left).unwrap().commit());
        context.runtime().collect().unwrap();
        assert!(left_edge.0.is_live().unwrap());
        assert!(right_edge.0.is_live().unwrap());
        drop((left, right));
        context.runtime().collect().unwrap();
        assert!(!left_edge.0.is_live().unwrap());
        assert!(!right_edge.0.is_live().unwrap());
        assert_eq!(signals.0.status(), SettlementStatus::Pending);
        assert_eq!(signals.1.status(), SettlementStatus::Pending);
    }
    #[test]
    fn concurrent_retirement_wakes_both_registered_native_waiters() {
        use std::{sync::mpsc, time::Duration};
        let (reached_tx, reached_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let domain = Arc::new(Domain {
            queue: Mutex::new(NoticeQueue {
                head: None,
                tail: None,
                wake_head: None,
                wake_tail: None,
                driving: false,
            }),
            empty_notice_hook: Some(EmptyNoticeHook {
                visits: AtomicUsize::new(0),
                reached: reached_tx,
                resume: Mutex::new(resume_rx),
            }),
        });
        let first = Signal::in_isolated_domain(domain.clone());
        let second = Signal::in_isolated_domain(domain);
        let first_count = Arc::new(WakeCount::default());
        let second_count = Arc::new(WakeCount::default());
        let mut first_wait = first.wait().unwrap();
        let mut second_wait = second.wait().unwrap();
        pending(&mut first_wait, &first_count);
        pending(&mut second_wait, &second_count);
        std::thread::scope(|threads| {
            let first_thread = threads.spawn(|| first.retire_pending(JobRetirement::Dropped));
            let second_signal = second.clone();
            let second_thread = threads.spawn(move || {
                reached_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("first driver reaches its final empty-notice boundary");
                second_signal.retire_pending(JobRetirement::Dropped);
                resume_tx
                    .send(())
                    .expect("first driver waits for the concurrent retirement");
            });
            first_thread
                .join()
                .expect("first retirement finishes without timeout");
            second_thread
                .join()
                .expect("second retirement finishes without timeout");
        });
        // Retire the isolated failed-run queue itself after preserving the
        // observable wake counts; no test-owned notice cycle survives failure.
        let residue = {
            let mut queue = lock(&second.notification_domain().queue);
            (
                queue.head.take(),
                queue.tail.take(),
                queue.wake_head.take(),
                queue.wake_tail.take(),
            )
        };
        drop(residue);
        assert_eq!(
            first.status(),
            SettlementStatus::Retired(JobRetirement::Dropped)
        );
        assert_eq!(
            second.status(),
            SettlementStatus::Retired(JobRetirement::Dropped)
        );
        assert_eq!(first_count.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            second_count.0.load(Ordering::SeqCst),
            1,
            "a concurrently queued native notice must wake its preexisting waiter without a later unrelated retirement"
        );
    }
}
