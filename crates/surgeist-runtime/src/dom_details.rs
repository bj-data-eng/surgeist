//! HTML Details' actual DOM-manipulation task owner and shared event delivery.
//!
//! A raw DOM transfer is admitted as a whole before ownership is consumed.
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
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use surgeist_dom::{DomDetailsToggleTransfer, DomError, DomOwnerId, DomRevision, NodeId};
use surgeist_html::{HtmlDetailsState, HtmlDetailsToggleEvent};

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Finite bounds for this source task owner, independent of event-path limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDetailsRuntimeLimits {
    /// All queued, retained and executing tasks, including untracked tasks.
    pub tasks: usize,
    /// Actual task/tracker Vec capacities, including atomic intake scratch banks.
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
}
/// A refused owner operation. Pending task/tracker state is preserved.
#[derive(Debug)]
pub enum DomDetailsRuntimeError {
    /// A foreign canonical owner, replaced task owner or invalid target kind.
    Binding,
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
            _ => None,
        }
    }
}
/// Refused intake retains the exact unconsumed, non-Clone DOM transfer.
#[derive(Debug)]
pub struct DomDetailsTransferError {
    error: DomDetailsRuntimeError,
    transfer: DomDetailsToggleTransfer,
}
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
impl fmt::Display for DomDetailsTransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}
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
/// Owns Details FIFO tasks and source tracker associations for one actual DOM.
/// Dispatcher registration/path/nesting budgets remain with their shared owner.
pub struct DomDetailsRuntime {
    state: Rc<RefCell<DetailsState>>,
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
}
impl DomDetailsRuntime {
    /// Bind an empty finite task owner to the actual canonical DOM store.
    pub fn new(
        dom: &DomEventDom<'_>,
        limits: DomDetailsRuntimeLimits,
    ) -> Result<Self, DomDetailsRuntimeError> {
        let owner = NEXT_OWNER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| DomDetailsRuntimeError::IdentityExhausted)?;
        Ok(Self {
            state: Rc::new(RefCell::new(DetailsState {
                owner,
                dom: dom.owner(),
                limits,
                tasks: Vec::new(),
                trackers: Vec::new(),
                next: 1,
                work: 0,
                last_transfer: None,
            })),
        })
    }
    /// Number of queued, retained and executing tasks, including untracked tasks.
    pub fn pending_tasks(&self) -> usize {
        self.state.borrow().tasks.len()
    }
    /// One immutable pending task in actual FIFO order.
    pub fn pending_task(&self, index: usize) -> Option<DomDetailsTask> {
        self.state
            .borrow()
            .tasks
            .get(index)
            .map(|entry| entry.plan().value)
    }
    /// Change finite admission for later work without resetting identity/work.
    /// Existing task count and actual retained bank capacity must still fit.
    /// Change configuration between dispatch calls, so reserved callback cleanup
    /// cannot be invalidated by raising the task bound during execution.
    pub fn set_limits(
        &mut self,
        limits: DomDetailsRuntimeLimits,
    ) -> Result<(), DomDetailsRuntimeError> {
        let mut state = self.state.borrow_mut();
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
    pub fn work_used(&self) -> u64 {
        self.state.borrow().work
    }
    /// Consume actual raw transitions only after a complete atomic intake plan.
    /// Refusal retains task identities/order and returns unchanged ownership.
    #[expect(
        clippy::result_large_err,
        reason = "Return the exact unconsumed non-Clone DOM transfer without allocating on admission failure"
    )]
    pub fn ingest(
        &mut self,
        transfer: DomDetailsToggleTransfer,
        cancel: &AtomicBool,
    ) -> Result<usize, DomDetailsTransferError> {
        self.state.borrow_mut().ingest(transfer, cancel)
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
            .try_fold(0usize, |sum, (count, size)| {
                count.checked_mul(size).and_then(|n| sum.checked_add(n))
            })
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?;
        if total > self.limits.storage_bytes {
            return Err(DomDetailsRuntimeError::Admission);
        }
        Ok(())
    }
    /// Consume actual raw transitions only after a complete atomic intake plan.
    /// Refusal retains task identities/order and returns unchanged ownership.
    #[expect(
        clippy::result_large_err,
        reason = "Return the exact unconsumed non-Clone DOM transfer without allocating on admission failure"
    )]
    pub fn ingest(
        &mut self,
        transfer: DomDetailsToggleTransfer,
        cancel: &AtomicBool,
    ) -> Result<usize, DomDetailsTransferError> {
        match self.plan(&transfer, cancel) {
            Err(error) => Err(DomDetailsTransferError { error, transfer }),
            Ok(Intake {
                plan,
                trackers,
                mut output,
                next,
            }) => {
                let count = transfer.len();
                // All comparisons, storage and serials were preadmitted. No
                // cancellation/fallible publication occurs after ownership moves.
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
                Ok(count)
            }
        }
    }
    fn plan(
        &mut self,
        transfer: &DomDetailsToggleTransfer,
        cancel: &AtomicBool,
    ) -> Result<Intake, DomDetailsRuntimeError> {
        self.spend(1, cancel)?;
        if transfer.source().owner() != self.dom {
            return Err(DomDetailsRuntimeError::Binding);
        }
        if self
            .last_transfer
            .is_some_and(|r| transfer.source().revision() < r)
        {
            return Err(DomDetailsRuntimeError::OutOfOrderTransfer);
        }
        // Each raw record appends at most one task/tracker. Admit the whole
        // checked peak bound before allocating either empty scratch bank; no
        // scratch bank grows/reallocates during planning. This avoids retaining
        // an old allocation while allocating its replacement beyond admission.
        let plan_capacity = self
            .tasks
            .len()
            .checked_add(transfer.len())
            .ok_or(DomDetailsRuntimeError::IdentityExhausted)?
            .min(self.limits.tasks);
        let tracker_capacity = self
            .trackers
            .len()
            .checked_add(transfer.len())
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
        for intention in transfer.intentions() {
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
        let state = context.details().state.clone();
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
        let displaced = !Rc::ptr_eq(&state, &context.details().state);
        match result? {
            DetailsStep::Complete | DetailsStep::Destroyed => {
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
                if let Some(receipt) = task.receipt.take() {
                    Ok(DomDetailsProgress::Complete {
                        task: value,
                        receipt,
                        displaced_owner: displaced.then(|| Self {
                            state: state.clone(),
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
                    state: state.clone(),
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

#[cfg(test)]
mod tests;
