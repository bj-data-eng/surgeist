//! One actual DOM-manipulation task owner for Details and Dialog toggles.
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
use std::cell::{Cell, RefCell};
use std::fmt;
use std::mem::size_of;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use surgeist_dom::{
    DomDetailsToggleIntention, DomDetailsToggleTransfer, DomError, DomOwnerId, DomRevision,
    DomSnapshot,
};
use surgeist_html::{HtmlDetailsState, HtmlDetailsToggleEvent};

/// Finite bounds for this source task owner, independent of event-path limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomManipulationRuntimeLimits {
    /// All queued, retained and executing tasks, including untracked tasks.
    pub tasks: usize,
    /// Actual owner header and task/tracker Vec capacities, including intake scratch.
    pub storage_bytes: usize,
    /// Monotone work admission, including failed attempts and reserved cleanup.
    pub work: u64,
}
/// Independently explicit dispatcher and canonical cleanup policy per call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomManipulationRunLimits {
    /// Actual shared event path/listener/callback work.
    pub dispatch: DomEventDispatchLimits,
    /// Original queued-task Source observation, rebase and mandatory cleanup.
    pub source: surgeist_dom::DomSourceLimits,
}
pub use surgeist_dom::DomManipulationTaskId;
/// One pending source task's immutable queued payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDetailsTask {
    id: DomManipulationTaskId,
    event: HtmlDetailsToggleEvent,
    association: Option<surgeist_dom::DomElementTaskAssociation>,
}
impl DomDetailsTask {
    /// Exact execution identity.
    pub fn id(self) -> DomManipulationTaskId {
        self.id
    }
    /// First-old/latest-new values retained when this task was queued.
    pub fn event(self) -> HtmlDetailsToggleEvent {
        self.event
    }
    /// Original task Document/Global captured when the source queued the event.
    /// Adoption changes the dispatch path, not this execution association.
    pub fn queue_association(self) -> Option<surgeist_dom::DomElementTaskAssociation> {
        self.association
    }
}
/// Closed payloads in the one DOM-manipulation FIFO.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomManipulationTask {
    /// A Details transition.
    Details(DomDetailsTask),
    /// A Dialog transition published by its original Source cursor.
    Dialog(surgeist_dom::DomDialogToggleTaskInfo),
}
impl DomManipulationTask {
    /// Exact original task occurrence and target generation.
    pub fn id(self) -> DomManipulationTaskId {
        match self {
            Self::Details(task) => task.id(),
            Self::Dialog(task) => task.id(),
        }
    }
    /// Original relevant Global and its associated Document.
    pub fn queue_association(self) -> Option<surgeist_dom::DomElementTaskAssociation> {
        match self {
            Self::Details(task) => task.queue_association(),
            Self::Dialog(task) => task.queue_association(),
        }
    }
    /// The Details payload, when this is a Details task.
    pub fn details(self) -> Option<DomDetailsTask> {
        if let Self::Details(task) = self {
            Some(task)
        } else {
            None
        }
    }
    /// The Dialog payload, when this is a Dialog task.
    pub fn dialog(self) -> Option<surgeist_dom::DomDialogToggleTaskInfo> {
        if let Self::Dialog(task) = self {
            Some(task)
        } else {
            None
        }
    }
}
/// A refused owner operation. Pending task/tracker state is preserved.
#[derive(Debug)]
pub enum DomManipulationRuntimeError {
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
    /// The original trusted Source event executor refused the task step.
    SourceEvent(crate::dom_dialog::DomSourceEventRuntimeError),
    /// The actual original native Global/loop owner refused task admission.
    Hosted(crate::dom_lifecycle::DomHostedRuntimeError),
}
impl fmt::Display for DomManipulationRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DOM-manipulation task operation refused: {self:?}")
    }
}
impl std::error::Error for DomManipulationRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::SourceEvent(error) => Some(error),
            Self::Hosted(error) => Some(error),
            _ => None,
        }
    }
}
/// Refused intake retains the exact unconsumed, non-Clone DOM transfer.
#[cfg(test)]
#[derive(Debug)]
struct DomDetailsTransferError {
    error: DomManipulationRuntimeError,
    transfer: DomDetailsToggleTransfer,
}
#[cfg(test)]
impl DomDetailsTransferError {
    /// Reason intake refused without changing tasks/trackers.
    pub fn error(&self) -> &DomManipulationRuntimeError {
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
pub enum DomManipulationProgress {
    /// No task is pending.
    Empty,
    /// The current FIFO head is already executing a listener.
    Running(DomManipulationTaskId),
    /// Actual shared dispatch completed, including callback failures if any.
    Complete {
        /// Exact task whose source event finished.
        task: DomManipulationTask,
        /// Actual shared-dispatch delivery and listener failure evidence.
        receipt: DomEventDispatchReceipt,
        /// Original bank when a callback replaced the context's coordinator.
        /// Retains other queued work even if the callback dropped its old handle.
        displaced_owner: Option<DomManipulationRuntime>,
    },
    /// The exact target was retired before dispatch started.
    SkippedTargetDestroyed(DomManipulationTask),
    /// Shared dispatch retains its next step; no delivered listener is replayed.
    Retained {
        /// Exact task whose event retains its next source step.
        task: DomManipulationTask,
        /// Shared dispatcher retention reason and continuation ownership.
        dispatch: DomEventDispatchOutcome,
        /// Restore this exact owner before servicing its retained task if a
        /// callback substituted another coordinator (even dropping the original).
        displaced_owner: Option<DomManipulationRuntime>,
    },
}
/// The caller's actual composition context exposes its single task owner.
/// No owner borrow is held while the shared dispatcher invokes listeners.
pub trait DomManipulationEventContext {
    /// Current source task owner. Moving/replacing it cannot redirect an already
    /// executing task; that task retains its original bank until cleanup.
    fn dom_manipulation(&mut self) -> &mut DomManipulationRuntime;
}
/// Relative custody of actual shared task-event allocations. The original
/// owner's existing storage policy reads this extent even after its row drops.
pub(crate) struct TaskPayload {
    live: Rc<Cell<usize>>,
    full: usize,
    source: usize,
    retained: surgeist_dom::DomManipulationTaskRetention,
}
impl fmt::Debug for TaskPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskPayload")
            .field("full", &self.full)
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}
impl TaskPayload {
    fn new(
        live: Rc<Cell<usize>>,
        full: usize,
        source: usize,
        retained: surgeist_dom::DomManipulationTaskRetention,
    ) -> Result<Self, DomManipulationRuntimeError> {
        live.set(
            live.get()
                .checked_add(full)
                .ok_or(DomManipulationRuntimeError::IdentityExhausted)?,
        );
        Ok(Self {
            live,
            full,
            source,
            retained,
        })
    }
    pub(crate) fn full(&self) -> usize {
        self.full
    }
    pub(crate) fn source(&self) -> usize {
        self.source
    }
    pub(crate) fn split(
        &mut self,
        full: usize,
        source: usize,
    ) -> Result<Self, DomManipulationRuntimeError> {
        let remaining = self
            .full
            .checked_sub(full)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let remaining_source = self
            .source
            .checked_sub(source)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let retained = self
            .retained
            .split(source)
            .map_err(DomManipulationRuntimeError::Dom)?;
        self.full = remaining;
        self.source = remaining_source;
        Ok(Self {
            live: self.live.clone(),
            full,
            source,
            retained,
        })
    }
    pub(crate) fn absorb(&mut self, mut other: Self) -> Result<(), DomManipulationRuntimeError> {
        if !Rc::ptr_eq(&self.live, &other.live) {
            return Err(DomManipulationRuntimeError::Binding);
        }
        let full = self
            .full
            .checked_add(other.full)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let source = self
            .source
            .checked_add(other.source)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        self.retained
            .absorb(
                other
                    .retained
                    .split(other.source)
                    .map_err(DomManipulationRuntimeError::Dom)?,
            )
            .map_err(DomManipulationRuntimeError::Dom)?;
        self.full = full;
        self.source = source;
        other.full = 0;
        other.source = 0;
        Ok(())
    }
    pub(crate) fn shrink(
        &mut self,
        full: usize,
        source: usize,
    ) -> Result<(), DomManipulationRuntimeError> {
        let remove_full = self
            .full
            .checked_sub(full)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let remove_source = self
            .source
            .checked_sub(source)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        drop(self.split(remove_full, remove_source)?);
        Ok(())
    }
}
impl Drop for TaskPayload {
    fn drop(&mut self) {
        self.live.set(self.live.get() - self.full);
    }
}
struct Task {
    value: DomManipulationTask,
    event: Option<NativeDomEventOwner>,
    receipt: Option<DomEventDispatchReceipt>,
    started: bool,
    running: bool,
    dialog: Option<surgeist_dom::DomDialogToggleTask>,
    publication: Option<surgeist_dom::DomDialogToggleQueuePublication>,
    task_operation: Option<surgeist_dom::DomSourceOperationId>,
    request: Option<surgeist_dom::DomSourceEventRequest>,
    permit: Option<surgeist_dom::DomSourceEventDispatchPermit>,
    event_publication: Option<surgeist_dom::DomSourceEventPublication>,
    event_accepted: bool,
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
    value: DomManipulationTask,
    started: bool,
}
#[derive(Clone, Copy)]
struct Tracker {
    task: DomManipulationTaskId,
    old: HtmlDetailsState,
}
struct Intake {
    plan: Vec<Plan>,
    trackers: Vec<Tracker>,
    output: Vec<Task>,
    next: u64,
    output_retained: surgeist_dom::DomManipulationTaskRetention,
    trackers_retained: surgeist_dom::DomManipulationTaskRetention,
    scratch_retained: surgeist_dom::DomManipulationTaskRetention,
}
enum TaskAdmission<'a, 'dom> {
    Idle(&'a mut DomEventDom<'dom>),
    Source(&'a mut surgeist_dom::DomSourceLease<'dom>),
}
impl TaskAdmission<'_, '_> {
    fn retain(
        &mut self,
        source: &surgeist_dom::DomManipulationTaskSource,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<surgeist_dom::DomManipulationTaskRetention, DomManipulationRuntimeError> {
        match self {
            Self::Idle(dom) => source.retain_payload(dom.raw_mut(), bytes, cancel),
            Self::Source(lease) => lease.retain_manipulation_payload(source, bytes, cancel),
        }
        .map_err(DomManipulationRuntimeError::Dom)
    }
}
/// Weak execution permission for the original session's shared task owner.
/// Dropping or replacing this value cannot release its tasks or source trackers.
#[derive(Clone)]
pub struct DomManipulationRuntime {
    state: Weak<RefCell<ManipulationState>>,
}
pub(crate) struct ManipulationOwner {
    state: Rc<RefCell<ManipulationState>>,
}
impl ManipulationOwner {
    pub(crate) fn new(
        dom: DomOwnerId,
        limits: DomManipulationRuntimeLimits,
    ) -> Result<Self, DomManipulationRuntimeError> {
        if ManipulationState::fixed_bytes()? > limits.storage_bytes {
            return Err(DomManipulationRuntimeError::Admission);
        }
        Ok(Self {
            state: Rc::new(RefCell::new(ManipulationState {
                dom,
                limits,
                tasks: Vec::new(),
                trackers: Vec::new(),
                next: 1,
                work: 0,
                last_transfer: None,
                hosted: None,
                source: None,
                tasks_retained: None,
                trackers_retained: None,
                owner_retained: None,
                payload_bytes: Rc::new(Cell::new(0)),
                events: None,
            })),
        })
    }
    pub(crate) fn bind_hosted(&self, hosted: crate::dom_lifecycle::HostedRuntime) {
        // Constructor-only join after both actual original owners exist. No
        // callback or externally retained bank borrow can run during binding.
        self.state.borrow_mut().hosted = Some(hosted);
    }
    pub(crate) fn bind_source(&self, dom: &mut surgeist_dom::Dom) {
        let cancel = AtomicBool::new(false);
        let source = Rc::new(
            dom.bind_manipulation_task_source(&cancel)
                .expect("exclusive preflighted task source binding"),
        );
        let retained = source
            .retain_payload(
                dom,
                size_of::<RefCell<ManipulationState>>()
                    + size_of::<surgeist_dom::DomManipulationTaskSource>()
                    + size_of::<Cell<usize>>(),
                &cancel,
            )
            .expect("preflighted fixed task owner");
        let mut state = self.state.borrow_mut();
        state.source = Some(source);
        state.owner_retained = Some(retained);
    }
    pub(crate) fn bind_events(&self, events: crate::dom_dialog::SourceEventRuntime) {
        self.state.borrow_mut().events = Some(events);
    }
    pub(crate) fn execution(&self) -> DomManipulationRuntime {
        DomManipulationRuntime {
            state: Rc::downgrade(&self.state),
        }
    }
    pub(crate) fn pending(
        &self,
    ) -> Result<Option<DomManipulationTaskId>, DomManipulationRuntimeError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        Ok(state.tasks.first().map(|task| task.value.id()))
    }
}
impl fmt::Debug for DomManipulationRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomManipulationRuntime")
            .finish_non_exhaustive()
    }
}
struct ManipulationState {
    dom: DomOwnerId,
    limits: DomManipulationRuntimeLimits,
    tasks: Vec<Task>,
    trackers: Vec<Tracker>,
    next: u64,
    work: u64,
    last_transfer: Option<DomRevision>,
    hosted: Option<crate::dom_lifecycle::HostedRuntime>,
    source: Option<Rc<surgeist_dom::DomManipulationTaskSource>>,
    tasks_retained: Option<surgeist_dom::DomManipulationTaskRetention>,
    trackers_retained: Option<surgeist_dom::DomManipulationTaskRetention>,
    owner_retained: Option<surgeist_dom::DomManipulationTaskRetention>,
    payload_bytes: Rc<Cell<usize>>,
    events: Option<crate::dom_dialog::SourceEventRuntime>,
}
impl DomManipulationRuntime {
    pub(crate) fn validate_view(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<(), DomManipulationRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        if state.dom != dom.owner() {
            return Err(DomManipulationRuntimeError::Binding);
        }
        Ok(())
    }

    fn state(&self) -> Result<Rc<RefCell<ManipulationState>>, DomManipulationRuntimeError> {
        self.state
            .upgrade()
            .ok_or(DomManipulationRuntimeError::OwnerDisposed)
    }
    /// Number of queued, retained and executing tasks, including untracked tasks.
    pub fn pending_tasks(&self) -> Result<usize, DomManipulationRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        Ok(state.tasks.len())
    }
    /// One immutable pending task in actual FIFO order.
    pub fn pending_task(
        &self,
        index: usize,
    ) -> Result<Option<DomManipulationTask>, DomManipulationRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        Ok(state.tasks.get(index).map(|entry| entry.plan().value))
    }
    /// Change finite admission for later work without resetting identity/work.
    /// Existing task count and actual retained bank capacity must still fit.
    /// Change configuration between dispatch calls, so reserved callback cleanup
    /// cannot be invalidated by raising the task bound during execution.
    pub fn set_limits(
        &mut self,
        limits: DomManipulationRuntimeLimits,
    ) -> Result<(), DomManipulationRuntimeError> {
        let state = self.state()?;
        let mut state = state
            .try_borrow_mut()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        if state.tasks.first().is_some_and(|entry| entry.running)
            || state.tasks.len() > limits.tasks
            || state.work > limits.work
        {
            return Err(DomManipulationRuntimeError::Admission);
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
    pub fn work_used(&self) -> Result<u64, DomManipulationRuntimeError> {
        let state = self.state()?;
        let state = state
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        Ok(state.work)
    }
    /// Admit actual pending canonical transitions before consuming ownership.
    /// On refusal, the original DOM retains every intention in source order.
    pub fn ingest(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<usize, DomManipulationRuntimeError> {
        let owner = self.state()?;
        let hosted = owner
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?
            .hosted
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        hosted.ingest_dom_manipulation(self, dom, cancel)
    }
    pub(crate) fn ingest_source(
        &self,
        dom: &mut DomEventDom<'_>,
        issuer: &surgeist_dom::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<usize, DomManipulationRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        let source = dom.snapshot();
        let intentions = source
            .details_toggle_intentions()
            .map_err(DomManipulationRuntimeError::Dom)?;
        let plan = state.plan(
            &source,
            intentions,
            0,
            &mut TaskAdmission::Idle(dom),
            cancel,
        )?;
        state.spend(1, cancel)?;
        if dom.snapshot().stamp() != source.stamp() {
            return Err(DomManipulationRuntimeError::Binding);
        }
        let transfer = dom
            .raw_mut()
            .take_hosted_details_toggle_intentions(issuer, intentions.len(), cancel)
            .map_err(DomManipulationRuntimeError::Dom)?;
        Ok(state.publish_intake(transfer, plan))
    }

    pub(crate) fn queue_dialog(
        &self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        request: &surgeist_dom::DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<(), DomManipulationRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        if state.dom != lease.snapshot().owner() {
            return Err(DomManipulationRuntimeError::Binding);
        }
        // A real queue effect may have committed before its exact acceptance
        // refused. The original row owns that affine publication across retries.
        for index in 0..state.tasks.len() {
            state.spend(1, cancel)?;
            if state.tasks[index]
                .publication
                .as_ref()
                .is_some_and(|publication| publication.operation() == request.operation())
            {
                return lease
                    .accept_dialog_toggle_queue(
                        request,
                        &mut state.tasks[index].publication,
                        cancel,
                    )
                    .map_err(DomManipulationRuntimeError::Dom);
            }
        }
        let index = state.publish_dialog_queue(lease, request, cancel)?;
        lease
            .accept_dialog_toggle_queue(request, &mut state.tasks[index].publication, cancel)
            .map_err(DomManipulationRuntimeError::Dom)
    }

    #[cfg(test)]
    #[expect(
        clippy::result_large_err,
        reason = "Private raw-transfer oracle retains the actual non-Clone input on refusal without allocating"
    )]
    fn ingest_transfer(
        &mut self,
        dom: &mut DomEventDom<'_>,
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
                    error: DomManipulationRuntimeError::Busy,
                    transfer,
                });
            }
        };
        match state.plan(
            transfer.source(),
            transfer.intentions(),
            0,
            &mut TaskAdmission::Idle(dom),
            cancel,
        ) {
            Err(error) => Err(DomDetailsTransferError { error, transfer }),
            Ok(plan) => Ok(state.publish_intake(transfer, plan)),
        }
    }
}
impl ManipulationState {
    fn publish_dialog_queue(
        &mut self,
        lease: &mut surgeist_dom::DomSourceLease<'_>,
        request: &surgeist_dom::DomSourceDialogRequest,
        cancel: &AtomicBool,
    ) -> Result<usize, DomManipulationRuntimeError> {
        let source = self
            .source
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        let snapshot = lease.snapshot();
        // This borrowed canonical slice is consumed only by preparation. No
        // caller code or publication occurs while this original bank is lent.
        let mut intake = {
            let actual = lease
                .dialog_toggle_prior_details(request, &snapshot)
                .map_err(DomManipulationRuntimeError::Dom)?;
            self.plan(
                &snapshot,
                actual,
                1,
                &mut TaskAdmission::Source(lease),
                cancel,
            )?
        };
        let queue = lease
            .dialog_toggle_queue_plan(request, &source, intake.next, cancel)
            .map_err(DomManipulationRuntimeError::Dom)?;
        intake.next = intake
            .next
            .checked_add(1)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        if let Some(replacing) = queue.replacing() {
            for index in 0..intake.plan.len() {
                self.spend(1, cancel)?;
                if intake.plan[index].value.id() == replacing {
                    if !intake.plan[index].started {
                        self.spend(
                            u64::try_from(intake.plan.len() - index - 1)
                                .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
                            cancel,
                        )?;
                        intake.plan.remove(index);
                    }
                    break;
                }
            }
        }
        if intake.plan.len() >= self.limits.tasks {
            return Err(DomManipulationRuntimeError::Admission);
        }
        intake.plan.push(Plan {
            value: DomManipulationTask::Dialog(queue.task()),
            started: false,
        });
        // Admit the complete infallible installation loop, including the new row.
        let comparisons = intake
            .plan
            .len()
            .checked_mul(self.tasks.len())
            .and_then(|n| n.checked_add(intake.plan.len()))
            .and_then(|n| n.checked_add(self.tasks.len()))
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        self.spend(
            u64::try_from(comparisons)
                .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
            cancel,
        )?;
        let commit = lease
            .prepare_dialog_toggle_queue(request, &source, queue, cancel)
            .map_err(DomManipulationRuntimeError::Dom)?;
        // Exclusive no-callback fence: all allocations, comparisons and work
        // have already been admitted. The new append is the known final row.
        self.install_intake(intake);
        let (task, publication) = commit.publish();
        let installed = self.tasks.last_mut().expect("prepared actual append");
        installed.dialog = Some(task);
        installed.publication = Some(publication);
        Ok(self.tasks.len() - 1)
    }
    fn fixed_bytes() -> Result<usize, DomManipulationRuntimeError> {
        size_of::<RefCell<Self>>()
            .checked_add(size_of::<Cell<usize>>())
            .and_then(|bytes| {
                bytes.checked_add(size_of::<surgeist_dom::DomManipulationTaskSource>())
            })
            .and_then(|bytes| {
                bytes.checked_add(surgeist_dom::DomManipulationTaskSource::binding_payload_bytes())
            })
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)
    }
    fn spend(
        &mut self,
        amount: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomManipulationRuntimeError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomManipulationRuntimeError::Dom(DomError::Cancelled));
        }
        let next = self
            .work
            .checked_add(amount)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        if next > self.limits.work {
            self.work = self.limits.work;
            return Err(DomManipulationRuntimeError::Admission);
        }
        self.work = next;
        Ok(())
    }
    fn bytes(
        &self,
        plans: usize,
        trackers: usize,
        output: usize,
    ) -> Result<(), DomManipulationRuntimeError> {
        let total = self.byte_total(plans, trackers, output)?;
        if total > self.limits.storage_bytes {
            return Err(DomManipulationRuntimeError::Admission);
        }
        Ok(())
    }
    fn intake_bytes(
        &self,
        plans: usize,
        trackers: usize,
        output: usize,
    ) -> Result<(), DomManipulationRuntimeError> {
        if self
            .byte_total(plans, trackers, output)?
            .checked_add(size_of::<Intake>())
            .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomManipulationRuntimeError::Admission);
        }
        Ok(())
    }
    fn byte_total(
        &self,
        plans: usize,
        trackers: usize,
        output: usize,
    ) -> Result<usize, DomManipulationRuntimeError> {
        let banks = [
            (self.tasks.capacity(), size_of::<Task>()),
            (self.trackers.capacity(), size_of::<Tracker>()),
            (plans, size_of::<Plan>()),
            (trackers, size_of::<Tracker>()),
            (output, size_of::<Task>()),
        ];
        let total = banks
            .into_iter()
            .try_fold(
                Self::fixed_bytes()?
                    .checked_add(self.payload_bytes.get())
                    .ok_or(DomManipulationRuntimeError::IdentityExhausted)?,
                |sum, (count, size)| count.checked_mul(size).and_then(|n| sum.checked_add(n)),
            )
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        Ok(total)
    }
    fn publish_intake(&mut self, transfer: DomDetailsToggleTransfer, intake: Intake) -> usize {
        let count = transfer.len();
        let revision = transfer.source().revision();
        self.install_intake(intake);
        self.last_transfer = Some(revision);
        drop(transfer);
        count
    }
    fn install_intake(&mut self, intake: Intake) {
        let Intake {
            plan,
            trackers,
            mut output,
            next,
            output_retained,
            trackers_retained,
            scratch_retained,
        } = intake;
        // Complete storage, serial, comparison and move work was admitted before
        // canonical take. Publication has no cancellation or fallible tail.
        for item in plan {
            if let Some(index) = self
                .tasks
                .iter()
                .position(|e| e.plan().value.id() == item.value.id())
            {
                output.push(self.tasks.swap_remove(index));
            } else {
                output.push(Task {
                    value: item.value,
                    event: None,
                    receipt: None,
                    started: false,
                    running: false,
                    dialog: None,
                    publication: None,
                    task_operation: None,
                    request: None,
                    permit: None,
                    event_publication: None,
                    event_accepted: false,
                });
            }
        }
        self.tasks = output;
        self.tasks_retained = Some(output_retained);
        self.trackers = trackers;
        self.trackers_retained = Some(trackers_retained);
        drop(scratch_retained);
        self.next = next;
    }
    fn plan(
        &mut self,
        source: &DomSnapshot,
        intentions: &[DomDetailsToggleIntention],
        additional: usize,
        admission: &mut TaskAdmission<'_, '_>,
        cancel: &AtomicBool,
    ) -> Result<Intake, DomManipulationRuntimeError> {
        self.spend(1, cancel)?;
        if source.owner() != self.dom {
            return Err(DomManipulationRuntimeError::Binding);
        }
        if self.last_transfer.is_some_and(|r| source.revision() < r) {
            return Err(DomManipulationRuntimeError::OutOfOrderTransfer);
        }
        // Each raw record appends at most one task/tracker. Admit the whole
        // checked peak bound before allocating either empty scratch bank; no
        // scratch bank grows/reallocates during planning. This avoids retaining
        // an old allocation while allocating its replacement beyond admission.
        let plan_capacity = self
            .tasks
            .len()
            .checked_add(intentions.len())
            .and_then(|n| n.checked_add(additional))
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?
            .min(self.limits.tasks);
        let tracker_capacity = self
            .trackers
            .len()
            .checked_add(intentions.len())
            .and_then(|n| n.checked_add(additional))
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?
            .min(self.limits.tasks);
        let source_owner = self
            .source
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        let formation_bytes = size_of::<Intake>()
            .checked_add(
                plan_capacity
                    .checked_mul(size_of::<Plan>())
                    .ok_or(DomManipulationRuntimeError::IdentityExhausted)?,
            )
            .and_then(|n| {
                tracker_capacity
                    .checked_mul(size_of::<Tracker>())
                    .and_then(|bytes| n.checked_add(bytes))
            })
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        self.intake_bytes(plan_capacity, tracker_capacity, 0)?;
        let mut retained = admission.retain(&source_owner, formation_bytes, cancel)?;
        let mut plan = Vec::new();
        let mut trackers = Vec::new();
        self.intake_bytes(plan_capacity, tracker_capacity, 0)?;
        plan.try_reserve_exact(plan_capacity)
            .map_err(|_| DomManipulationRuntimeError::Admission)?;
        self.intake_bytes(plan.capacity(), tracker_capacity, 0)?;
        trackers
            .try_reserve_exact(tracker_capacity)
            .map_err(|_| DomManipulationRuntimeError::Admission)?;
        self.intake_bytes(plan.capacity(), trackers.capacity(), 0)?;
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
                if tracker.task.target() == intention.target() {
                    tracked = Some(index);
                    break;
                }
            }
            let old = tracked.map(|index| trackers[index]);
            if let Some(old) = old {
                for (index, task) in plan.iter().enumerate() {
                    self.spend(1, cancel)?;
                    if task.value.id() == old.task {
                        if !task.started {
                            self.spend(
                                u64::try_from(plan.len() - index - 1)
                                    .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
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
                        .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
                    cancel,
                )?;
                trackers.remove(index);
            }
            if plan.len() >= self.limits.tasks {
                return Err(DomManipulationRuntimeError::Admission);
            }
            let details = DomDetailsTask {
                id: self
                    .source
                    .as_ref()
                    .ok_or(DomManipulationRuntimeError::Binding)?
                    .task_id(next, intention.target())
                    .map_err(DomManipulationRuntimeError::Dom)?,
                association: intention.queue_association(),
                event: HtmlDetailsToggleEvent::queued(
                    intention.old_state(),
                    intention.new_state(),
                    old.map(|t| t.old),
                ),
            };
            let value = DomManipulationTask::Details(details);
            next = next
                .checked_add(1)
                .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
            plan.push(Plan {
                value,
                started: false,
            });
            trackers.push(Tracker {
                task: value.id(),
                old: details.event.old_state(),
            });
        }
        let output_capacity = plan
            .len()
            .checked_add(additional)
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let output_bytes = output_capacity
            .checked_mul(size_of::<Task>())
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let mut output_retained = admission.retain(&source_owner, output_bytes, cancel)?;
        let mut output = Vec::new();
        self.intake_bytes(plan.capacity(), trackers.capacity(), output_capacity)?;
        output
            .try_reserve_exact(output_capacity)
            .map_err(|_| DomManipulationRuntimeError::Admission)?;
        self.intake_bytes(plan.capacity(), trackers.capacity(), output.capacity())?;
        let actual_scratch = size_of::<Intake>()
            .checked_add(
                plan.capacity()
                    .checked_mul(size_of::<Plan>())
                    .ok_or(DomManipulationRuntimeError::IdentityExhausted)?,
            )
            .and_then(|n| {
                trackers
                    .capacity()
                    .checked_mul(size_of::<Tracker>())
                    .and_then(|bytes| n.checked_add(bytes))
            })
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        if actual_scratch > formation_bytes {
            retained
                .absorb(admission.retain(
                    &source_owner,
                    actual_scratch - formation_bytes,
                    cancel,
                )?)
                .map_err(DomManipulationRuntimeError::Dom)?;
        }
        let actual_output = output
            .capacity()
            .checked_mul(size_of::<Task>())
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        if actual_output > output_bytes {
            output_retained
                .absorb(admission.retain(&source_owner, actual_output - output_bytes, cancel)?)
                .map_err(DomManipulationRuntimeError::Dom)?;
        }
        let trackers_retained = retained
            .split(trackers.capacity() * size_of::<Tracker>())
            .map_err(DomManipulationRuntimeError::Dom)?;
        // Publication searches at most this many original entries per planned
        // entry. Reserve that finite work before committing any ownership.
        let comparisons = plan
            .len()
            .checked_mul(self.tasks.len())
            .and_then(|n| n.checked_add(plan.len()))
            .and_then(|n| n.checked_add(self.tasks.len()))
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        self.spend(
            u64::try_from(comparisons)
                .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
            cancel,
        )?;
        Ok(Intake {
            plan,
            trackers,
            output,
            next,
            output_retained,
            trackers_retained,
            scratch_retained: retained,
        })
    }
}
enum PreparedEvent {
    Destroyed,
    Delivered,
    Dispatch(
        NativeDomEvent,
        Option<surgeist_dom::DomSourceEventDispatchGuard>,
    ),
}
impl ManipulationState {
    fn prepare_event(
        &mut self,
        dom: &mut DomEventDom<'_>,
        value: DomManipulationTask,
        limits: DomManipulationRunLimits,
        cancel: &AtomicBool,
    ) -> Result<PreparedEvent, DomManipulationRuntimeError> {
        let index = self
            .tasks
            .iter()
            .position(|task| task.value.id() == value.id())
            .ok_or(DomManipulationRuntimeError::Binding)?;
        if self.tasks[index].receipt.is_some() {
            return Ok(PreparedEvent::Delivered);
        }
        if !self.tasks[index].started {
            match dom.snapshot().node(value.id().target()) {
                Err(DomError::StaleNode) => return Ok(PreparedEvent::Destroyed),
                Err(error) => return Err(DomManipulationRuntimeError::Dom(error)),
                Ok(node) => {
                    let valid = match value {
                        DomManipulationTask::Details(_) => node.as_details().is_some(),
                        DomManipulationTask::Dialog(_) => node.as_dialog().is_some(),
                    };
                    if !valid {
                        return Err(DomManipulationRuntimeError::Binding);
                    }
                }
            }
        }
        let source = self
            .source
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        if matches!(value, DomManipulationTask::Dialog(_)) && self.tasks[index].request.is_none() {
            let mut lease = if let Some(operation) = self.tasks[index].task_operation {
                source
                    .resume_dialog_toggle_task(
                        dom.raw_mut(),
                        value.id(),
                        operation,
                        limits.source,
                        cancel,
                    )
                    .map_err(DomManipulationRuntimeError::Dom)?
            } else {
                source
                    .begin_dialog_toggle_task(
                        dom.raw_mut(),
                        &mut self.tasks[index].dialog,
                        limits.source,
                        cancel,
                    )
                    .map_err(DomManipulationRuntimeError::Dom)?
            };
            self.tasks[index].task_operation = Some(lease.operation());
            let request = match lease
                .advance(cancel)
                .map_err(DomManipulationRuntimeError::Dom)?
            {
                surgeist_dom::DomSourceProgress::Request(
                    surgeist_dom::DomSourceRequest::Event(request),
                ) if request.kind() == surgeist_dom::DomSourceEventKind::Toggle => request,
                _ => return Err(DomManipulationRuntimeError::Binding),
            };
            self.tasks[index].request = Some(request);
        }
        if let Some(operation) = self.tasks[index].task_operation {
            drop(
                source
                    .resume_dialog_toggle_task(
                        dom.raw_mut(),
                        value.id(),
                        operation,
                        limits.source,
                        cancel,
                    )
                    .map_err(DomManipulationRuntimeError::Dom)?,
            );
        }
        let previous = self.tasks[index]
            .event
            .as_ref()
            .map_or((0, 0), NativeDomEventOwner::manipulation_extent);
        let formation = self.tasks[index]
            .event
            .as_ref()
            .map_or_else(
                || NativeDomEventOwner::fresh_dispatch_formation(limits.dispatch),
                |event| event.dispatch_formation(limits.dispatch),
            )
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        let maximum = formation.full.max(previous.0);
        let source_maximum = formation.source.max(previous.1);
        let extra = maximum - previous.0;
        if self
            .byte_total(0, 0, 0)?
            .checked_add(extra)
            .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomManipulationRuntimeError::Admission);
        }
        let retained = source
            .retain_payload(dom.raw_mut(), source_maximum - previous.1, cancel)
            .map_err(DomManipulationRuntimeError::Dom)?;
        let retained = TaskPayload::new(
            self.payload_bytes.clone(),
            extra,
            source_maximum - previous.1,
            retained,
        )?;
        let task = &mut self.tasks[index];
        let event = task.event.get_or_insert_with(|| match value {
            DomManipulationTask::Details(details) => {
                NativeDomEvent::details_toggle(value.id().target(), details.event())
            }
            DomManipulationTask::Dialog(dialog) => NativeDomEvent::dialog_toggle(
                value.id().target(),
                dialog.old_state(),
                dialog.new_state(),
            ),
        });
        event.retain_manipulation(retained);
        let scope = if let Some(request) = &task.request {
            let Some(events) = self.events.as_ref() else {
                event.settle_manipulation();
                return Err(DomManipulationRuntimeError::Binding);
            };
            match events.task_dispatch_scope(dom, request, &mut task.permit, cancel) {
                Ok(scope) => Some(scope),
                Err(error) => {
                    event.settle_manipulation();
                    return Err(DomManipulationRuntimeError::SourceEvent(error));
                }
            }
        } else {
            None
        };
        Ok(PreparedEvent::Dispatch((**event).clone(), scope))
    }
    fn finish_dialog_event(
        &mut self,
        dom: &mut DomEventDom<'_>,
        id: DomManipulationTaskId,
        limits: surgeist_dom::DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomManipulationRuntimeError> {
        let source = self
            .source
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        let events = self
            .events
            .clone()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        let task = self
            .tasks
            .iter_mut()
            .find(|task| task.value.id() == id)
            .ok_or(DomManipulationRuntimeError::Binding)?;
        let Some(operation) = task.task_operation else {
            return Ok(());
        };
        let request = task
            .request
            .as_ref()
            .ok_or(DomManipulationRuntimeError::Binding)?;
        if !task.event_accepted {
            if task.event_publication.is_none() {
                task.event_publication = Some(
                    events
                        .seal_task_dispatch(dom, &mut task.permit, cancel)
                        .map_err(DomManipulationRuntimeError::SourceEvent)?,
                );
            }
            let mut lease = source
                .resume_dialog_toggle_task(dom.raw_mut(), id, operation, limits, cancel)
                .map_err(DomManipulationRuntimeError::Dom)?;
            lease
                .accept_source_event(request, &mut task.event_publication, cancel)
                .map_err(DomManipulationRuntimeError::Dom)?;
            task.event_accepted = true;
        }
        let mut lease = source
            .resume_dialog_toggle_task(dom.raw_mut(), id, operation, limits, cancel)
            .map_err(DomManipulationRuntimeError::Dom)?;
        match lease
            .advance(cancel)
            .map_err(DomManipulationRuntimeError::Dom)?
        {
            surgeist_dom::DomSourceProgress::Complete(_) => {
                task.task_operation = None;
                // The real canonical root has released its EventWork debit.
                // Keep no private request backing across a later checkpoint
                // refusal; actual event/path/receipt custody remains separate.
                task.request = None;
                Ok(())
            }
            _ => Err(DomManipulationRuntimeError::Binding),
        }
    }
}
impl DomManipulationRuntime {
    /// Execute or resume the FIFO head through the actual shared dispatcher.
    /// The source task and its actual event/receipt remain in their original
    /// bank across caller work and unexpected unwind.
    pub fn run_next<C: DomManipulationEventContext>(
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomManipulationRunLimits,
        cancel: &AtomicBool,
    ) -> Result<DomManipulationProgress, DomManipulationRuntimeError> {
        let state = context.dom_manipulation().state()?;
        let value = {
            let mut owner = state.borrow_mut();
            if owner.dom != dom.owner() {
                return Err(DomManipulationRuntimeError::Binding);
            }
            if cancel.load(Ordering::Relaxed) {
                return Err(DomManipulationRuntimeError::Dom(DomError::Cancelled));
            }
            let Some(head) = owner.tasks.first() else {
                return Ok(DomManipulationProgress::Empty);
            };
            let value = head.value;
            if dom
                .raw()
                .pending_source_edit()
                .is_some_and(|operation| head.task_operation != Some(operation.operation()))
            {
                return Err(DomManipulationRuntimeError::Dom(
                    DomError::SourceOperationPending,
                ));
            }
            if head.running {
                owner.spend(1, cancel)?;
                return Ok(DomManipulationProgress::Running(value.id()));
            }
            let cleanup = owner
                .limits
                .tasks
                .checked_mul(3)
                .and_then(|n| n.checked_add(1))
                .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
            owner.spend(
                u64::try_from(cleanup)
                    .map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
                cancel,
            )?;
            owner.tasks[0].running = true;
            value
        };
        let _running = ManipulationTaskRunning {
            state: state.clone(),
            id: value.id(),
        };
        let mut native_task = if let Some(association) = value.queue_association() {
            let (hosted, started) = {
                let owner = state.borrow();
                let head = owner
                    .tasks
                    .first()
                    .ok_or(DomManipulationRuntimeError::Binding)?;
                (
                    owner
                        .hosted
                        .clone()
                        .ok_or(DomManipulationRuntimeError::Binding)?,
                    head.started,
                )
            };
            let retired_target = !started
                && matches!(
                    dom.snapshot().node(value.id().target()),
                    Err(DomError::StaleNode)
                );
            Some(
                hosted
                    .enter_dom_manipulation(dom, association, value.id(), retired_target, cancel)
                    .map_err(DomManipulationRuntimeError::Hosted)?,
            )
        } else {
            None
        };
        let result = (|| {
            let prepared = {
                let mut owner = state.borrow_mut();
                owner.prepare_event(dom, value, limits, cancel)?
            };
            let (event, scope) = match prepared {
                PreparedEvent::Destroyed => return Ok(ManipulationStep::Destroyed),
                PreparedEvent::Delivered => return Ok(ManipulationStep::Complete),
                PreparedEvent::Dispatch(event, scope) => (event, scope),
            };
            let _scope = scope;
            let outcome = if event.has_started() {
                dispatcher.resume(dom.raw_mut(), context, &event, limits.dispatch)
            } else {
                dispatcher.dispatch(dom.raw_mut(), context, &event, limits.dispatch)
            };
            let mut owner = state.borrow_mut();
            owner
                .tasks
                .iter()
                .find(|task| task.value.id() == value.id())
                .and_then(|task| task.event.as_ref())
                .expect("original actual event owner")
                .settle_manipulation();
            let outcome = outcome.map_err(DomManipulationRuntimeError::Dispatch)?;
            let task = owner
                .tasks
                .iter_mut()
                .find(|entry| entry.value.id() == value.id())
                .ok_or(DomManipulationRuntimeError::Binding)?;
            task.started = true;
            match outcome {
                DomEventDispatchOutcome::Complete(receipt) => {
                    task.receipt = Some(receipt);
                    Ok(ManipulationStep::Complete)
                }
                retained => Ok(ManipulationStep::Retained(retained)),
            }
        })();
        // This is arbitrary caller code too. The bank already owns a completed
        // receipt if it unwinds, so subsequent execution cannot replay dispatch.
        let displaced = !Weak::ptr_eq(&Rc::downgrade(&state), &context.dom_manipulation().state);
        match result? {
            ManipulationStep::Complete | ManipulationStep::Destroyed => {
                if matches!(value, DomManipulationTask::Dialog(_))
                    && state
                        .borrow()
                        .tasks
                        .iter()
                        .any(|task| task.value.id() == value.id() && task.receipt.is_some())
                {
                    state.borrow_mut().finish_dialog_event(
                        dom,
                        value.id(),
                        limits.source,
                        cancel,
                    )?;
                }
                if let Some(native_task) = &mut native_task {
                    native_task
                        .checkpoint(dom, cancel)
                        .map_err(DomManipulationRuntimeError::Hosted)?;
                }
                let mut owner = state.borrow_mut();
                let index = owner
                    .tasks
                    .iter()
                    .position(|entry| entry.value.id() == value.id())
                    .ok_or(DomManipulationRuntimeError::Binding)?;
                let mut task = owner.tasks.remove(index);
                if let Some(index) = owner
                    .trackers
                    .iter()
                    .position(|tracker| tracker.task.target() == value.id().target())
                {
                    owner.trackers.remove(index);
                }
                if let Some(native_task) = &mut native_task {
                    native_task.complete();
                }
                if let Some(mut receipt) = task.receipt.take() {
                    // Ordinary caller output transfer ends product ownership of
                    // diagnostic failure backing; shared Event/Path stay charged.
                    receipt.release_manipulation_retention();
                    Ok(DomManipulationProgress::Complete {
                        task: value,
                        receipt,
                        displaced_owner: displaced.then(|| Self {
                            state: Rc::downgrade(&state),
                        }),
                    })
                } else {
                    Ok(DomManipulationProgress::SkippedTargetDestroyed(value))
                }
            }
            ManipulationStep::Retained(dispatch) => Ok(DomManipulationProgress::Retained {
                task: value,
                dispatch,
                displaced_owner: displaced.then(|| Self {
                    state: Rc::downgrade(&state),
                }),
            }),
        }
    }
}
enum ManipulationStep {
    Complete,
    Destroyed,
    Retained(DomEventDispatchOutcome),
}
struct ManipulationTaskRunning {
    state: Rc<RefCell<ManipulationState>>,
    id: DomManipulationTaskId,
}
impl Drop for ManipulationTaskRunning {
    fn drop(&mut self) {
        // All private bank borrows end before caller work, or unwind before this
        // guard. No caller can retain a RefMut, and cleanup invokes no user code.
        let mut owner = self.state.borrow_mut();
        if let Some(entry) = owner
            .tasks
            .iter_mut()
            .find(|entry| entry.value.id() == self.id)
        {
            entry.running = false;
        }
    }
}

impl DomManipulationRuntime {
    // Destroy-document removes only this original Document's not-started task
    // bodies. An adopted target's current Document is intentionally irrelevant.
    pub(crate) fn cancel_document(
        &self,
        dom: &DomEventDom<'_>,
        document: surgeist_dom::DocumentId,
        cancel: &AtomicBool,
    ) -> Result<bool, DomManipulationRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        if state.dom != dom.owner() {
            return Err(DomManipulationRuntimeError::Binding);
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
                    .ok_or(DomManipulationRuntimeError::IdentityExhausted)?,
            )
            .ok_or(DomManipulationRuntimeError::IdentityExhausted)?;
        state.spend(
            u64::try_from(upper).map_err(|_| DomManipulationRuntimeError::IdentityExhausted)?,
            cancel,
        )?;
        let mut retained_started = false;
        let mut index = 0;
        while index < state.tasks.len() {
            let task = &state.tasks[index];
            if task
                .value
                .queue_association()
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
                .position(|tracker| tracker.task == removed.value.id())
            {
                state.trackers.remove(index);
            }
            // The private event owner can abandon an unstarted path, never
            // manufacture a completion receipt or dispatch a cancellation event.
            drop(removed);
        }
        Ok(retained_started)
    }
    pub(crate) fn has_task(
        &self,
        task: DomManipulationTaskId,
    ) -> Result<bool, DomManipulationRuntimeError> {
        let owner = self.state()?;
        let state = owner
            .try_borrow()
            .map_err(|_| DomManipulationRuntimeError::Busy)?;
        // This scalar query is used only after the destruction caller prepaid
        // its complete task-source cleanup scan; it invokes no user code.
        Ok(state.tasks.iter().any(|entry| entry.value.id() == task))
    }
}

#[cfg(test)]
mod tests;
