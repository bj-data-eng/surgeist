//! Qualified Source event requests and an explicitly trusted host executor.
//!
//! The executor attests actual dispatch. DOM validates association and one-time
//! consumption; it cannot independently prove an external host ran callbacks.
use super::*;
use crate::{Dom, DomError, DomOwnerId};
use std::sync::{
    Mutex, Weak,
    atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
};
use surgeist_html::{HtmlDialogAction, HtmlDialogState};

/// One exact event occurrence of an original Source operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceEventId {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) occurrence: u64,
}
/// Exact native event firing clause admitted by the original Source engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventKind {
    /// Synchronous beforetoggle with the reached method’s actual cancelability.
    BeforeToggle,
    /// A queued Dialog toggle task with mandatory canonical tracker cleanup.
    Toggle,
    /// Plain queued close notification, without ToggleEvent data.
    Close,
}
/// The original event's completed cancelation outcome, attested by its executor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventOutcome {
    Canceled,
    NotCanceled,
}
#[derive(Debug)]
pub(crate) struct ExecutorBinding {
    pub(crate) owner: DomOwnerId,
    pub(crate) live: AtomicBool,
}
#[derive(Debug)]
pub(crate) struct DispatchScope;
#[derive(Debug)]
pub(crate) struct EventWork {
    pub(crate) executor: Weak<ExecutorBinding>,
    pub(crate) identity: DomSourceEventId,
    pub(crate) parent: Option<DomSourceOperationId>,
    pub(crate) subject: NodeId,
    pub(crate) kind: DomSourceEventKind,
    pub(crate) payload: DomSourceEventPayload,
    pub(crate) cancelable: bool,
    pub(crate) stage: AtomicU8,
    pub(crate) work: Arc<AtomicU64>,
    pub(crate) work_limit: AtomicU64,
    pub(crate) scope: Mutex<Weak<DispatchScope>>,
    pub(crate) ancestors: Option<Arc<EventWork>>,
    pub(crate) bytes: Arc<AtomicUsize>,
    pub(crate) units: Arc<AtomicUsize>,
}
impl EventWork {
    pub(crate) fn step_own(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled);
        }
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let limit = self.work_limit.load(Ordering::Relaxed);
        if next > limit {
            return Err(DomError::SourceWorkLimit { limit });
        }
        self.work.store(next, Ordering::Relaxed);
        Ok(())
    }

    pub(crate) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.step_own(cancel)?;
        step_ancestors(self.ancestors.as_ref(), cancel)
    }
    pub(crate) fn active(&self) -> bool {
        self.stage.load(Ordering::Relaxed) == 1
            && self
                .scope
                .lock()
                .is_ok_and(|scope| scope.upgrade().is_some())
    }
    fn validate_executor(&self, executor: &Arc<ExecutorBinding>) -> Result<(), DomError> {
        if !executor.live.load(Ordering::Relaxed)
            || self.subject.owner() != executor.owner
            || !self
                .executor
                .upgrade()
                .is_some_and(|actual| Arc::ptr_eq(&actual, executor))
        {
            return Err(DomError::InvalidSourceEvent);
        }
        Ok(())
    }
}
/// Actual ToggleEvent data; plain events have no invented state strings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceToggleEvent {
    pub(crate) old: HtmlDialogState,
    pub(crate) new: HtmlDialogState,
}
impl DomSourceToggleEvent {
    /// Exact source-selected old state.
    pub const fn old_state(self) -> HtmlDialogState {
        self.old
    }
    /// Exact source-selected new state.
    pub const fn new_state(self) -> HtmlDialogState {
        self.new
    }
    /// Standard Show/Close methods and the admitted Toggle tasks supply null.
    pub const fn source(self) -> Option<NodeId> {
        None
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum DomSourceEventPayload {
    Toggle(DomSourceToggleEvent),
    Close,
}
/// Immutable actual method/task event data, not dispatch or reply authority.
/// ToggleEvent data exists only for beforetoggle and toggle; Close is plain.
#[derive(Clone, Debug)]
pub struct DomSourceEventRequest {
    pub(crate) work: Arc<EventWork>,
}
impl DomSourceEventRequest {
    /// Actual producer firing clause, independent of payload state spellings.
    pub fn kind(&self) -> DomSourceEventKind {
        self.work.kind
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.work.identity.operation
    }
    pub fn parent_operation(&self) -> Option<DomSourceOperationId> {
        self.work.parent
    }
    pub fn identity(&self) -> DomSourceEventId {
        self.work.identity
    }
    pub fn subject(&self) -> NodeId {
        self.work.subject
    }
    /// Actual ToggleEvent data, absent for a plain Close notification.
    pub fn toggle_event(&self) -> Option<DomSourceToggleEvent> {
        match self.work.payload {
            DomSourceEventPayload::Toggle(payload) => Some(payload),
            DomSourceEventPayload::Close => None,
        }
    }
    pub fn cancelable(&self) -> bool {
        self.work.cancelable
    }
}
/// Actual Dialog service occurrence reached by the original Source engine.
/// This immutable request grants no service or acknowledgement authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceDialogRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) subject: NodeId,
    pub(crate) action: HtmlDialogAction,
    pub(crate) occurrence: Option<super::watcher::DialogOccurrence>,
}
impl DomSourceDialogRequest {
    pub(crate) fn hook(&self) -> Option<super::watcher::HookOccurrence> {
        match self.occurrence {
            Some(super::watcher::DialogOccurrence::Hook(value)) => Some(value),
            _ => None,
        }
    }
    pub(crate) fn method(&self) -> Option<super::watcher::HookOccurrence> {
        match self.occurrence {
            Some(super::watcher::DialogOccurrence::Method(value)) => Some(value),
            _ => None,
        }
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub fn subject(&self) -> NodeId {
        self.subject
    }
    pub fn action(&self) -> HtmlDialogAction {
        self.action
    }
}
/// Sole trusted host-executor capability installed for this actual DOM owner.
/// Moving it transfers the capability; a request does not lend this authority.
#[derive(Debug)]
pub struct DomSourceEventExecutor {
    pub(crate) binding: Arc<ExecutorBinding>,
}
impl Drop for DomSourceEventExecutor {
    fn drop(&mut self) {
        self.binding.live.store(false, Ordering::Relaxed);
    }
}
/// Affine permission for exactly one request's actual retained dispatch.
#[derive(Debug)]
pub struct DomSourceEventDispatchPermit {
    work: Arc<EventWork>,
}
/// The exact actual invocation's reentry scope. Drop/unwind closes reentry;
/// retaining an event job alone does not leave unrelated edits enabled.
#[derive(Debug)]
pub struct DomSourceEventDispatchGuard {
    _scope: Arc<DispatchScope>,
    _retained: DomSourceEventRetention,
}
/// A completed trusted dispatch attestation, consumed only after DOM admission.
#[derive(Debug)]
pub struct DomSourceEventPublication {
    pub(crate) work: Arc<EventWork>,
    pub(crate) outcome: DomSourceEventOutcome,
}
/// Continuous charge for actual external job/event/receipt storage. Backing
/// owners must drop before this guard; acquire both guards across replacement.
#[derive(Debug)]
pub struct DomSourceEventRetention {
    pub(crate) work: Arc<EventWork>,
    pub(crate) bytes: usize,
    pub(crate) units: usize,
}
impl DomSourceEventRetention {
    /// Merge already admitted custody for the same request without allocating.
    pub fn absorb(&mut self, mut other: Self) -> Result<(), DomError> {
        if !Arc::ptr_eq(&self.work, &other.work) {
            return Err(DomError::InvalidSourceEvent);
        }
        let bytes = self
            .bytes
            .checked_add(other.bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .units
            .checked_add(other.units)
            .ok_or(DomError::RepresentationExhausted)?;
        self.bytes = bytes;
        self.units = units;
        other.bytes = 0;
        other.units = 0;
        Ok(())
    }
}
impl Drop for DomSourceEventRetention {
    fn drop(&mut self) {
        self.work.bytes.fetch_sub(self.bytes, Ordering::Relaxed);
        self.work.units.fetch_sub(self.units, Ordering::Relaxed);
    }
}
impl DomSourceEventExecutor {
    /// Actual logical binding payload owned by this capability. The inline
    /// handle belongs to the consumer header; allocator/control-block overhead
    /// is not part of the repository payload tariff.
    pub const fn binding_payload_bytes() -> usize {
        std::mem::size_of::<ExecutorBinding>()
    }

    /// Begin this current owner's exact top observation. The canonical borrow
    /// keeps original Source work exclusive with tree execution.
    ///
    /// The canonical owner can move between threads, but cannot be shared for
    /// parallel dispatch while another thread executes its Source segment:
    ///
    /// ```compile_fail
    /// fn parallel_dispatch(
    ///     dom: &surgeist_dom::Dom,
    ///     executor: &surgeist_dom::DomSourceEventExecutor,
    ///     request: &surgeist_dom::DomSourceEventRequest,
    /// ) {
    ///     std::thread::scope(|scope| {
    ///         scope.spawn(move || {
    ///             let cancel = std::sync::atomic::AtomicBool::new(false);
    ///             let _ = executor.begin_dispatch(dom, request, &cancel);
    ///         });
    ///     });
    /// }
    /// ```
    pub fn begin_dispatch(
        &self,
        dom: &Dom,
        request: &DomSourceEventRequest,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventDispatchPermit, DomError> {
        dom.qualify_source_event(request, 0)?;
        request.work.validate_executor(&self.binding)?;
        request.work.step(cancel)?;
        request
            .work
            .stage
            .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| DomError::InvalidSourceEvent)?;
        Ok(DomSourceEventDispatchPermit {
            work: request.work.clone(),
        })
    }
    pub fn enter_dispatch(
        &self,
        dom: &Dom,
        permit: &DomSourceEventDispatchPermit,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventDispatchGuard, DomError> {
        let request = DomSourceEventRequest {
            work: permit.work.clone(),
        };
        dom.qualify_source_event(&request, 1)?;
        permit.work.validate_executor(&self.binding)?;
        permit.work.step(cancel)?;
        let retained = dom.retain_source_event_payload(
            &request,
            std::mem::size_of::<DispatchScope>(),
            0,
            cancel,
        )?;
        let mut slot = permit
            .work
            .scope
            .lock()
            .map_err(|_| DomError::InvalidSourceEvent)?;
        if slot.upgrade().is_some() {
            return Err(DomError::InvalidSourceEvent);
        }
        let scope = Arc::new(DispatchScope);
        *slot = Arc::downgrade(&scope);
        Ok(DomSourceEventDispatchGuard {
            _scope: scope,
            _retained: retained,
        })
    }
    /// Trusted attestation only: the host must use its actual complete
    /// dispatcher receipt/canceled flag, never an incomplete invocation.
    pub fn seal_completed(
        &self,
        dom: &Dom,
        permit: &mut Option<DomSourceEventDispatchPermit>,
        outcome: DomSourceEventOutcome,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventPublication, DomError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled);
        }
        let actual = permit.as_ref().ok_or(DomError::InvalidSourceEvent)?;
        dom.qualify_source_event(
            &DomSourceEventRequest {
                work: actual.work.clone(),
            },
            1,
        )?;
        actual.work.validate_executor(&self.binding)?;
        actual.work.step(cancel)?;
        if actual
            .work
            .scope
            .lock()
            .map_err(|_| DomError::InvalidSourceEvent)?
            .upgrade()
            .is_some()
        {
            return Err(DomError::InvalidSourceEvent);
        }
        actual
            .work
            .stage
            .compare_exchange(1, 2, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| DomError::InvalidSourceEvent)?;
        Ok(DomSourceEventPublication {
            work: permit.take().expect("checked permit").work,
            outcome,
        })
    }
}

/// Charge each actual retained owner, without resetting or replacing its meter.
pub(crate) fn step_ancestors(
    mut ancestor: Option<&Arc<EventWork>>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    while let Some(work) = ancestor {
        work.step_own(cancel)?;
        ancestor = work.ancestors.as_ref();
    }
    Ok(())
}

pub(crate) fn remaining_ancestor_work(
    mut ancestor: Option<&Arc<EventWork>>,
    mut remaining: u64,
) -> Result<u64, DomError> {
    while let Some(work) = ancestor {
        let limit = work.work_limit.load(Ordering::Relaxed);
        remaining = remaining.min(
            limit
                .checked_sub(work.work.load(Ordering::Relaxed))
                .ok_or(DomError::SourceWorkLimit { limit })?,
        );
        ancestor = work.ancestors.as_ref();
    }
    Ok(remaining)
}
pub(crate) fn retain_ancestor_work(
    mut ancestor: Option<&Arc<EventWork>>,
    delta: u64,
) -> Result<(), DomError> {
    while let Some(work) = ancestor {
        let next = work
            .work
            .load(Ordering::Relaxed)
            .checked_add(delta)
            .ok_or(DomError::RepresentationExhausted)?;
        let limit = work.work_limit.load(Ordering::Relaxed);
        if next > limit {
            return Err(DomError::SourceWorkLimit { limit });
        }
        work.work.store(next, Ordering::Relaxed);
        ancestor = work.ancestors.as_ref();
    }
    Ok(())
}
