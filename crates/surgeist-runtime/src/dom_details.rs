//! HTML Details' actual DOM-manipulation task owner and shared event delivery.
//!
//! Canonical pending transitions are admitted as a whole before ownership moves.
//! Intake refusal leaves the actual intentions in the original bound DOM.
//! The task bank and source tracker are distinct: post-dispatch cleanup can
//! leave a reentrantly queued task untracked. Dropping this owner abandons its
//! remaining tasks; it never reports those tasks as delivered.
use crate::dom_event::{
    DomEventDispatchError, DomEventDispatchLimits, DomEventDispatchOutcome,
    DomEventDispatchReceipt, DomEventDispatcher, DomEventDom, NativeDomEvent, NativeDomEventOwner,
};
use std::cell::RefCell;
use std::fmt;
use std::mem::size_of;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use surgeist_dom::{
    DomDetailsToggleIntention, DomDetailsToggleTransfer, DomError, DomOwnerId, DomRevision,
    DomSnapshot, NodeId,
};
use surgeist_html::{HtmlDetailsState, HtmlDetailsToggleEvent};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Finite bounds for this source task owner, independent of event-path limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDetailsRuntimeLimits {
    /// All queued, retained and executing tasks, including untracked tasks.
    pub tasks: usize,
    /// Actual owner header and task/tracker Vec capacities, including intake scratch.
    pub storage_bytes: usize,
    /// Monotone work admission, including failed attempts and reserved cleanup.
    pub work: u64,
}
/// Exact task owner, checked serial and generation-qualified target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDetailsTaskId {
    owner: u64,
    serial: u64,
    target: NodeId,
}
impl DomDetailsTaskId {
    /// The actual element, never a Document/surface fallback.
    pub fn target(self) -> NodeId {
        self.target
    }
}
/// One pending source task's immutable queued payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDetailsTask {
    id: DomDetailsTaskId,
    event: HtmlDetailsToggleEvent,
    association: Option<surgeist_dom::DomDetailsQueueAssociation>,
}
impl DomDetailsTask {
    /// Exact execution identity.
    pub fn id(self) -> DomDetailsTaskId {
        self.id
    }
    /// First-old/latest-new values retained when this task was queued.
    pub fn event(self) -> HtmlDetailsToggleEvent {
        self.event
    }
    /// Original task Document/Global captured when the source queued the event.
    /// Adoption changes the dispatch path, not this execution association.
    pub fn queue_association(self) -> Option<surgeist_dom::DomDetailsQueueAssociation> {
        self.association
    }
}
/// A refused owner operation. Pending task/tracker state is preserved.
#[derive(Debug)]
pub enum DomDetailsRuntimeError {
    /// Standalone intake already transferred unknown external task obligations.
    PretransferredSession,
    /// A foreign canonical owner, replaced task owner or invalid target kind.
    Binding,
    /// The original durable session has released this permission's owner.
    OwnerDisposed,
    /// The original task state is temporarily borrowed by another operation.
    Busy,
    /// A transfer predates another already consumed transfer from this DOM.
    OutOfOrderTransfer,
    /// Canonical source/liveness validation failed.
    Dom(DomError),
    /// Task count, actual bank capacity or monotone work admission failed.
    Admission,
    /// A checked owner/task/work/storage representation would overflow.
    IdentityExhausted,
    /// Shared dispatch refused before starting or resuming this task.
    Dispatch(DomEventDispatchError),
    /// The actual original native Global/loop owner refused task admission.
    Hosted(crate::dom_lifecycle::DomHostedRuntimeError),
}
impl fmt::Display for DomDetailsRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Details task operation refused: {self:?}")
    }
}
impl std::error::Error for DomDetailsRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Hosted(error) => Some(error),
            _ => None,
        }
    }
}
/// Refused intake retains the exact unconsumed, non-Clone DOM transfer.
#[cfg(test)]
#[derive(Debug)]
struct DomDetailsTransferError {
    error: DomDetailsRuntimeError,
    transfer: DomDetailsToggleTransfer,
}
#[cfg(test)]
impl DomDetailsTransferError {
    /// Reason intake refused without changing tasks/trackers.
    pub fn error(&self) -> &DomDetailsRuntimeError {
        &self.error
    }
    /// Recover the same transfer for an exact later retry or another owner.
    pub fn into_transfer(self) -> DomDetailsToggleTransfer {
        self.transfer
    }
}
#[cfg(test)]
impl fmt::Display for DomDetailsTransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
#[cfg(test)]
impl std::error::Error for DomDetailsTransferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
/// Delivery or a precise retained source task, never an acknowledgement proxy.
#[derive(Debug)]
pub enum DomDetailsProgress {
    /// No task is pending.
    Empty,
    /// The current FIFO head is already executing a listener.
    Running(DomDetailsTaskId),
    /// Actual shared dispatch completed, including callback failures if any.
    Complete {
        /// Exact task whose source event finished.
        task: DomDetailsTask,
        /// Actual shared-dispatch delivery and listener failure evidence.
        receipt: DomEventDispatchReceipt,
        /// Original bank when a callback replaced the context's coordinator.
        /// Retains other queued work even if the callback dropped its old handle.
        displaced_owner: Option<DomDetailsRuntime>,
    },
    /// The exact target was retired before dispatch started.
    SkippedTargetDestroyed(DomDetailsTask),
    /// Shared dispatch retains its next step; no delivered listener is replayed.
    Retained {
        /// Exact task whose event retains its next source step.
        task: DomDetailsTask,
        /// Shared dispatcher retention reason and continuation ownership.
        dispatch: DomEventDispatchOutcome,
        /// Restore this exact owner before servicing its retained task if a
        /// callback substituted another coordinator (even dropping the original).
        displaced_owner: Option<DomDetailsRuntime>,
    },
}
/// The caller's actual composition context exposes its single Details owner.
/// No owner borrow is held while the shared dispatcher invokes listeners.
pub trait DomDetailsEventContext {
    /// Current source task owner. Moving/replacing it cannot redirect an already
    /// executing task; that task retains its original bank until cleanup.
    fn details(&mut self) -> &mut DomDetailsRuntime;
}
struct Task {
    value: DomDetailsTask,
    event: Option<NativeDomEventOwner>,
    receipt: Option<DomEventDispatchReceipt>,
    started: bool,
    running: bool,
}
impl Task {
    fn plan(&self) -> Plan {
        Plan {
            value: self.value,
            started: self.started || self.running,
        }
    }
}
#[derive(Clone, Copy)]
struct Plan {
    value: DomDetailsTask,
    started: bool,
}
#[derive(Clone, Copy)]
struct Tracker {
    task: DomDetailsTaskId,
    old: HtmlDetailsState,
}
struct Intake {
    plan: Vec<Plan>,
    trackers: Vec<Tracker>,
    output: Vec<Task>,
    next: u64,
}
/// Weak execution permission for the original session's Details task owner.
/// Dropping or replacing this value cannot release its tasks or source trackers.
#[derive(Clone)]
pub struct DomDetailsRuntime {
    state: Weak<RefCell<DetailsState>>,
}
pub(crate) struct DetailsOwner {
    state: Rc<RefCell<DetailsState>>,
}
impl DetailsOwner {
    pub(crate) fn new(
        dom: DomOwnerId,
        limits: DomDetailsRuntimeLimits,
    ) -> Result<Self, DomDetailsRuntimeError> {
        if size_of::<RefCell<DetailsState>>() > limits.storage_bytes {
            return Err(DomDetailsRuntimeError::Admission);
        }
        let owner = NEXT_OWNER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?;
        Ok(Self {
            state: Rc::new(RefCell::new(DetailsState {
                owner,
                dom,
                limits,
                tasks: Vec::new(),
                trackers: Vec::new(),
                next: 1,
                work: 0,
                last_transfer: None,
                hosted: None,
            })),
        })
    }
    pub(crate) fn bind_hosted(&self, hosted: crate::dom_lifecycle::HostedRuntime) {
        // Constructor-only join after both actual original owners exist. No
        // callback or externally retained bank borrow can run during binding.
        self.state.borrow_mut().hosted = Some(hosted);
    }
    pub(crate) fn execution(&self) -> DomDetailsRuntime {
        DomDetailsRuntime {
            state: Rc::downgrade(&self.state),
        }
    }
    pub(crate) fn pending(&self) -> Result<Option<DomDetailsTaskId>, DomDetailsRuntimeError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        Ok(state.tasks.first().map(|task| task.value.id))
    }
}
impl fmt::Debug for DomDetailsRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomDetailsRuntime").finish_non_exhaustive()
    }
}
struct DetailsState {
    owner: u64,
    dom: DomOwnerId,
    limits: DomDetailsRuntimeLimits,
    tasks: Vec<Task>,
    trackers: Vec<Tracker>,
    next: u64,
    work: u64,
    last_transfer: Option<DomRevision>,
    hosted: Option<crate::dom_lifecycle::HostedRuntime>,
}
impl DomDetailsRuntime {
    fn state(&self) -> Result<Rc<RefCell<DetailsState>>, DomDetailsRuntimeError> {
        self.state
            .upgrade()
            .ok_or(DomDetailsRuntimeError::OwnerDisposed)
    }
    /// Number of queued, retained and executing tasks, including untracked tasks.
    pub fn pending_tasks(&self) -> Result<usize, DomDetailsRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        Ok(state.tasks.len())
    }
    /// One immutable pending task in actual FIFO order.
    pub fn pending_task(
        &self,
        index: usize,
    ) -> Result<Option<DomDetailsTask>, DomDetailsRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        Ok(state.tasks.get(index).map(|entry| entry.plan().value))
    }
    /// Change finite admission for later work without resetting identity/work.
    /// Existing task count and actual retained bank capacity must still fit.
    /// Change configuration between dispatch calls, so reserved callback cleanup
    /// cannot be invalidated by raising the task bound during execution.
    pub fn set_limits(
        &mut self,
        limits: DomDetailsRuntimeLimits,
    ) -> Result<(), DomDetailsRuntimeError> {
        let state = self.state()?;
        let mut state = state
            .try_borrow_mut()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        if state.tasks.first().is_some_and(|entry| entry.running)
            || state.tasks.len() > limits.tasks
            || state.work > limits.work
        {
            return Err(DomDetailsRuntimeError::Admission);
        }
        let previous = state.limits;
        state.limits = limits;
        if let Err(error) = state.bytes(0, 0, 0) {
            state.limits = previous;
            return Err(error);
        }
        Ok(())
    }
    /// Actual monotone work charged, including refused attempts.
    pub fn work_used(&self) -> Result<u64, DomDetailsRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        Ok(state.work)
    }
    /// Admit actual pending canonical transitions before consuming ownership.
    /// On refusal, the original DOM retains every intention in source order.
    pub fn ingest(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<usize, DomDetailsRuntimeError> {
        let owner = self.state()?;
        let hosted = owner
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?
            .hosted
            .clone()
            .ok_or(DomDetailsRuntimeError::Binding)?;
        hosted.ingest_details(self, dom, cancel)
    }
    pub(crate) fn ingest_source(
        &self,
        dom: &mut DomEventDom<'_>,
        issuer: &surgeist_dom::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<usize, DomDetailsRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        let source = dom.snapshot();
        let intentions = source
            .details_toggle_intentions()
            .map_err(DomDetailsRuntimeError::Dom)?;
        let plan = state.plan(&source, intentions, cancel)?;
        state.spend(1, cancel)?;
        if dom.snapshot().stamp() != source.stamp() {
            return Err(DomDetailsRuntimeError::Binding);
        }
        let transfer = dom
            .raw_mut()
            .take_hosted_details_toggle_intentions(issuer, intentions.len(), cancel)
            .map_err(DomDetailsRuntimeError::Dom)?;
        Ok(state.publish_intake(transfer, plan))
    }

    #[cfg(test)]
    #[expect(
        clippy::result_large_err,
        reason = "Private raw-transfer oracle retains the actual non-Clone input on refusal without allocating"
    )]
    fn ingest_transfer(
        &mut self,
        transfer: DomDetailsToggleTransfer,
        cancel: &AtomicBool,
    ) -> Result<usize, DomDetailsTransferError> {
        let state = match self.state() {
            Ok(state) => state,
            Err(error) => return Err(DomDetailsTransferError { error, transfer }),
        };
        let mut state = match state.try_borrow_mut() {
            Ok(state) => state,
            Err(_) => {
                return Err(DomDetailsTransferError {
                    error: DomDetailsRuntimeError::Busy,
                    transfer,
                });
            }
        };
        match state.plan(transfer.source(), transfer.intentions(), cancel) {
            Err(error) => Err(DomDetailsTransferError { error, transfer }),
            Ok(plan) => Ok(state.publish_intake(transfer, plan)),
        }
    }
}
impl DetailsState {
    fn spend(&mut self, amount: u64, cancel: &AtomicBool) -> Result<(), DomDetailsRuntimeError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomDetailsRuntimeError::Dom(DomError::Cancelled));
        }
        let next = self
            .work
            .checked_add(amount)
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
        if next > self.limits.work {
            self.work = self.limits.work;
            return Err(DomDetailsRuntimeError::Admission);
        }
        self.work = next;
        Ok(())
    }
    fn bytes(
        &self,
        plans: usize,
        trackers: usize,
        output: usize,
    ) -> Result<(), DomDetailsRuntimeError> {
        let banks = [
            (self.tasks.capacity(), size_of::<Task>()),
            (self.trackers.capacity(), size_of::<Tracker>()),
            (plans, size_of::<Plan>()),
            (trackers, size_of::<Tracker>()),
            (output, size_of::<Task>()),
        ];
        let total = banks
            .into_iter()
            .try_fold(size_of::<RefCell<Self>>(), |sum, (count, size)| {
                count.checked_mul(size).and_then(|n| sum.checked_add(n))
            })
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
        if total > self.limits.storage_bytes {
            return Err(DomDetailsRuntimeError::Admission);
        }
        Ok(())
    }
    fn publish_intake(&mut self, transfer: DomDetailsToggleTransfer, intake: Intake) -> usize {
        let Intake {
            plan,
            trackers,
            mut output,
            next,
        } = intake;
        let count = transfer.len();
        // Complete storage, serial, comparison and move work was admitted before
        // canonical take. Publication has no cancellation or fallible tail.
        for item in plan {
            if let Some(index) = self
                .tasks
                .iter()
                .position(|e| e.plan().value.id == item.value.id)
            {
                output.push(self.tasks.swap_remove(index));
            } else {
                output.push(Task {
                    value: item.value,
                    event: None,
                    receipt: None,
                    started: false,
                    running: false,
                });
            }
        }
        self.tasks = output;
        self.trackers = trackers;
        self.next = next;
        self.last_transfer = Some(transfer.source().revision());
        drop(transfer);
        count
    }
    fn plan(
        &mut self,
        source: &DomSnapshot,
        intentions: &[DomDetailsToggleIntention],
        cancel: &AtomicBool,
    ) -> Result<Intake, DomDetailsRuntimeError> {
        self.spend(1, cancel)?;
        if source.owner() != self.dom {
            return Err(DomDetailsRuntimeError::Binding);
        }
        if self.last_transfer.is_some_and(|r| source.revision() < r) {
            return Err(DomDetailsRuntimeError::OutOfOrderTransfer);
        }
        // Each raw record appends at most one task/tracker. Admit the whole
        // checked peak bound before allocating either empty scratch bank; no
        // scratch bank grows/reallocates during planning. This avoids retaining
        // an old allocation while allocating its replacement beyond admission.
        let plan_capacity = self
            .tasks
            .len()
            .checked_add(intentions.len())
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?
            .min(self.limits.tasks);
        let tracker_capacity = self
            .trackers
            .len()
            .checked_add(intentions.len())
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?
            .min(self.limits.tasks);
        let mut plan = Vec::new();
        let mut trackers = Vec::new();
        self.bytes(plan_capacity, tracker_capacity, 0)?;
        plan.try_reserve_exact(plan_capacity)
            .map_err(|_| DomDetailsRuntimeError::Admission)?;
        self.bytes(plan.capacity(), tracker_capacity, 0)?;
        trackers
            .try_reserve_exact(tracker_capacity)
            .map_err(|_| DomDetailsRuntimeError::Admission)?;
        self.bytes(plan.capacity(), trackers.capacity(), 0)?;
        for index in 0..self.tasks.len() {
            self.spend(1, cancel)?;
            plan.push(self.tasks[index].plan());
        }
        for index in 0..self.trackers.len() {
            self.spend(1, cancel)?;
            trackers.push(self.trackers[index]);
        }
        let mut next = self.next;
        for intention in intentions {
            self.spend(1, cancel)?;
            let mut tracked = None;
            for (index, tracker) in trackers.iter().enumerate() {
                self.spend(1, cancel)?;
                if tracker.task.target == intention.target() {
                    tracked = Some(index);
                    break;
                }
            }
            let old = tracked.map(|index| trackers[index]);
            if let Some(old) = old {
                for (index, task) in plan.iter().enumerate() {
                    self.spend(1, cancel)?;
                    if task.value.id == old.task {
                        if !task.started {
                            self.spend(
                                u64::try_from(plan.len() - index - 1)
                                    .map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?,
                                cancel,
                            )?;
                            plan.remove(index);
                        }
                        break;
                    }
                }
                let index = tracked.expect("tracked source association");
                self.spend(
                    u64::try_from(trackers.len() - index - 1)
                        .map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?,
                    cancel,
                )?;
                trackers.remove(index);
            }
            if plan.len() >= self.limits.tasks {
                return Err(DomDetailsRuntimeError::Admission);
            }
            let value = DomDetailsTask {
                id: DomDetailsTaskId {
                    owner: self.owner,
                    serial: next,
                    target: intention.target(),
                },
                association: intention.queue_association(),
                event: HtmlDetailsToggleEvent::queued(
                    intention.old_state(),
                    intention.new_state(),
                    old.map(|t| t.old),
                ),
            };
            next = next
                .checked_add(1)
                .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
            plan.push(Plan {
                value,
                started: false,
            });
            trackers.push(Tracker {
                task: value.id,
                old: value.event.old_state(),
            });
        }
        let mut output = Vec::new();
        self.bytes(plan.capacity(), trackers.capacity(), plan.len())?;
        output
            .try_reserve_exact(plan.len())
            .map_err(|_| DomDetailsRuntimeError::Admission)?;
        self.bytes(plan.capacity(), trackers.capacity(), output.capacity())?;
        // Publication searches at most this many original entries per planned
        // entry. Reserve that finite work before committing any ownership.
        let comparisons = plan
            .len()
            .checked_mul(self.tasks.len())
            .and_then(|n| n.checked_add(plan.len()))
            .and_then(|n| n.checked_add(self.tasks.len()))
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
        self.spend(
            u64::try_from(comparisons).map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?,
            cancel,
        )?;
        Ok(Intake {
            plan,
            trackers,
            output,
            next,
        })
    }
}
impl DomDetailsRuntime {
    /// Execute or resume the FIFO head through the actual shared dispatcher.
    /// The source task and its actual event/receipt remain in their original
    /// bank across caller work and unexpected unwind.
    pub fn run_next<C: DomDetailsEventContext>(
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
    ) -> Result<DomDetailsProgress, DomDetailsRuntimeError> {
        let state = context.details().state()?;
        let value = {
            let mut owner = state.borrow_mut();
            if owner.dom != dom.owner() {
                return Err(DomDetailsRuntimeError::Binding);
            }
            if cancel.load(Ordering::Relaxed) {
                return Err(DomDetailsRuntimeError::Dom(DomError::Cancelled));
            }
            let Some(head) = owner.tasks.first() else {
                return Ok(DomDetailsProgress::Empty);
            };
            let value = head.value;
            if head.running {
                owner.spend(1, cancel)?;
                return Ok(DomDetailsProgress::Running(value.id));
            }
            let cleanup = owner
                .limits
                .tasks
                .checked_mul(3)
                .and_then(|n| n.checked_add(1))
                .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
            owner.spend(
                u64::try_from(cleanup).map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?,
                cancel,
            )?;
            owner.tasks[0].running = true;
            value
        };
        let _running = DetailsTaskRunning {
            state: state.clone(),
            id: value.id,
        };
        let mut native_task = if let Some(association) = value.association {
            let (hosted, started) = {
                let owner = state.borrow();
                let head = owner.tasks.first().ok_or(DomDetailsRuntimeError::Binding)?;
                (
                    owner
                        .hosted
                        .clone()
                        .ok_or(DomDetailsRuntimeError::Binding)?,
                    head.started,
                )
            };
            let retired_target = !started
                && matches!(
                    dom.snapshot().node(value.id.target),
                    Err(DomError::StaleNode)
                );
            Some(
                hosted
                    .enter_details(dom, association, value.id, retired_target, cancel)
                    .map_err(DomDetailsRuntimeError::Hosted)?,
            )
        } else {
            None
        };
        let result = (|| {
            let event = {
                let mut owner = state.borrow_mut();
                let task = owner
                    .tasks
                    .iter_mut()
                    .find(|entry| entry.value.id == value.id)
                    .ok_or(DomDetailsRuntimeError::Binding)?;
                if !task.started {
                    match dom.snapshot().node(value.id.target) {
                        Err(DomError::StaleNode) => return Ok(DetailsStep::Destroyed),
                        Err(error) => return Err(DomDetailsRuntimeError::Dom(error)),
                        Ok(node) if node.as_details().is_none() => {
                            return Err(DomDetailsRuntimeError::Binding);
                        }
                        Ok(_) => {}
                    }
                }
                if task.receipt.is_some() {
                    return Ok(DetailsStep::Complete);
                }
                let event = task.event.get_or_insert_with(|| {
                    NativeDomEvent::details_toggle(value.id.target, value.event)
                });
                (**event).clone()
            };
            let outcome = if event.has_started() {
                dispatcher.resume(dom.raw_mut(), context, &event, limits)
            } else {
                dispatcher.dispatch(dom.raw_mut(), context, &event, limits)
            }
            .map_err(DomDetailsRuntimeError::Dispatch)?;
            let mut owner = state.borrow_mut();
            let task = owner
                .tasks
                .iter_mut()
                .find(|entry| entry.value.id == value.id)
                .ok_or(DomDetailsRuntimeError::Binding)?;
            task.started = true;
            match outcome {
                DomEventDispatchOutcome::Complete(receipt) => {
                    task.receipt = Some(receipt);
                    Ok(DetailsStep::Complete)
                }
                retained => Ok(DetailsStep::Retained(retained)),
            }
        })();
        // This is arbitrary caller code too. The bank already owns a completed
        // receipt if it unwinds, so subsequent execution cannot replay dispatch.
        let displaced = !Weak::ptr_eq(&Rc::downgrade(&state), &context.details().state);
        match result? {
            DetailsStep::Complete | DetailsStep::Destroyed => {
                if let Some(native_task) = &mut native_task {
                    native_task
                        .checkpoint(dom, cancel)
                        .map_err(DomDetailsRuntimeError::Hosted)?;
                }
                let mut owner = state.borrow_mut();
                let index = owner
                    .tasks
                    .iter()
                    .position(|entry| entry.value.id == value.id)
                    .ok_or(DomDetailsRuntimeError::Binding)?;
                let mut task = owner.tasks.remove(index);
                if let Some(index) = owner
                    .trackers
                    .iter()
                    .position(|tracker| tracker.task.target == value.id.target)
                {
                    owner.trackers.remove(index);
                }
                if let Some(native_task) = &mut native_task {
                    native_task.complete();
                }
                if let Some(receipt) = task.receipt.take() {
                    Ok(DomDetailsProgress::Complete {
                        task: value,
                        receipt,
                        displaced_owner: displaced.then(|| Self {
                            state: Rc::downgrade(&state),
                        }),
                    })
                } else {
                    Ok(DomDetailsProgress::SkippedTargetDestroyed(value))
                }
            }
            DetailsStep::Retained(dispatch) => Ok(DomDetailsProgress::Retained {
                task: value,
                dispatch,
                displaced_owner: displaced.then(|| Self {
                    state: Rc::downgrade(&state),
                }),
            }),
        }
    }
}
enum DetailsStep {
    Complete,
    Destroyed,
    Retained(DomEventDispatchOutcome),
}
struct DetailsTaskRunning {
    state: Rc<RefCell<DetailsState>>,
    id: DomDetailsTaskId,
}
impl Drop for DetailsTaskRunning {
    fn drop(&mut self) {
        // All private bank borrows end before caller work, or unwind before this
        // guard. No caller can retain a RefMut, and cleanup invokes no user code.
        let mut owner = self.state.borrow_mut();
        if let Some(entry) = owner
            .tasks
            .iter_mut()
            .find(|entry| entry.value.id == self.id)
        {
            entry.running = false;
        }
    }
}

impl DomDetailsRuntime {
    // Destroy-document removes only this original Document's not-started task
    // bodies. An adopted target's current Document is intentionally irrelevant.
    pub(crate) fn cancel_document(
        &self,
        dom: &DomEventDom<'_>,
        document: surgeist_dom::DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomDetailsRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        if state.dom != dom.owner() {
            return Err(DomDetailsRuntimeError::Binding);
        }
        let count = state.tasks.len();
        // Every source association read, removal move and tracker comparison is
        // admitted before any task is removed. Failure preserves the whole bank.
        let upper = count
            .checked_mul(
                state
                    .trackers
                    .len()
                    .checked_mul(2)
                    .and_then(|trackers| count.checked_add(trackers))
                    .and_then(|n| n.checked_add(3))
                    .ok_or(DomDetailsRuntimeError::IdentityExhausted)?,
            )
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
        state.spend(
            u64::try_from(upper).map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?,
            cancel,
        )?;
        let mut retained_started = false;
        let mut index = 0;
        while index < state.tasks.len() {
            let task = &state.tasks[index];
            if task
                .value
                .association
                .is_none_or(|association| association.document() != document)
            {
                index += 1;
                continue;
            }
            let started = task.running
                || task.started
                || task.event.as_ref().is_some_and(|event| event.has_started());
            if started {
                retained_started = true;
                index += 1;
                continue;
            }
            let removed = state.tasks.remove(index);
            if let Some(index) = state
                .trackers
                .iter()
                .position(|tracker| tracker.task == removed.value.id)
            {
                state.trackers.remove(index);
            }
            // The private event owner can abandon an unstarted path, never
            // manufacture a completion receipt or dispatch a cancellation event.
            drop(removed);
        }
        Ok(retained_started)
    }
    pub(crate) fn has_task(&self, task: DomDetailsTaskId) -> Result<bool, DomDetailsRuntimeError> {
        let owner = self.state()?;
        let state = owner
            .try_borrow()
            .map_err(|_| DomDetailsRuntimeError::Busy)?;
        // This scalar query is used only after the destruction caller prepaid
        // its complete task-source cleanup scan; it invokes no user code.
        Ok(state.tasks.iter().any(|entry| entry.value.id == task))
    }
}

#[cfg(test)]
mod tests;
