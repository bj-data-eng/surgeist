//! Qualified Source event requests and an explicitly trusted host executor.
//!
//! The executor attests actual dispatch. DOM validates association and one-time
//! consumption; it cannot independently prove an external host ran callbacks.
use super::callback::{CallbackWork, DispatchScope};
use super::*;
use crate::{Dom, DomError, DomOwnerId};
use std::sync::{
    Weak,
    atomic::{AtomicBool, Ordering},
};
use surgeist_html::elements::dialog::{HtmlDialogAction, HtmlDialogState};

/// One exact event occurrence of an original Source operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceEventId {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) occurrence: u64,
}
/// Selects whether an exact native advance can request an authenticated clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventClockMode {
    /// Context-free DOM execution can deliver selected events without timing.
    ContextFree,
    /// Return a sealed firing for the original hosted issuer to acknowledge.
    Hosted,
}

/// A native clock attempt that supplied no usable time for a selected event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventClockFailure {
    /// The original clock could not supply a monotonic time.
    Unavailable,
    /// The native reading preceded the original clock's latest reading.
    Regressed,
    /// The actual native time exceeded the representation.
    Overflow,
}

/// The original issuer's observation for an exact synchronous firing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceEventTime {
    pub(crate) global: crate::DomHostedGlobalId,
    pub(crate) origin: crate::DomHostedMoment,
    pub(crate) occurrence: crate::DomHostedMoment,
}
impl DomSourceEventTime {
    pub const fn global(self) -> crate::DomHostedGlobalId {
        self.global
    }

    pub const fn origin(self) -> crate::DomHostedMoment {
        self.origin
    }

    pub const fn occurrence(self) -> crate::DomHostedMoment {
        self.occurrence
    }
}

/// A reached synchronous firing whose real request is still private to DOM.
#[derive(Debug)]
pub struct DomSourceEventFiring {
    pub(crate) identity: DomSourceEventId,
    pub(crate) global: crate::DomHostedGlobalId,
    pub(crate) document: crate::DocumentId,
    pub(crate) origin: crate::DomHostedMoment,
}

/// Adapter advance before a reached firing can expose its actual request.
pub enum DomSourceAdvance<T> {
    Progress(T),
    EventFiring(DomSourceEventFiring),
}

/// Exact native event firing clause admitted by the original Source engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventKind {
    /// Synchronous beforetoggle with the reached method’s actual cancelability.
    BeforeToggle,
    /// Synchronous plain cancel fired by the actual original watcher request.
    Cancel,
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
    Cancel,
}
/// Immutable actual method/task event data, not dispatch or reply authority.
/// ToggleEvent data exists only for beforetoggle and toggle; Close is plain.
#[derive(Clone, Debug)]
pub struct DomSourceEventRequest {
    pub(crate) work: Arc<CallbackWork>,
    pub(crate) executor: Weak<ExecutorBinding>,
    pub(crate) identity: DomSourceEventId,
    pub(crate) parent: Option<DomSourceOperationId>,
    pub(crate) subject: NodeId,
    pub(crate) kind: DomSourceEventKind,
    pub(crate) payload: DomSourceEventPayload,
    pub(crate) cancelable: bool,
}
impl DomSourceEventRequest {
    pub(crate) fn validate_executor(
        &self,
        executor: &Arc<ExecutorBinding>,
    ) -> Result<(), DomError> {
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
    /// Actual producer firing clause, independent of payload state spellings.
    pub fn kind(&self) -> DomSourceEventKind {
        self.kind
    }
    pub fn operation(&self) -> DomSourceOperationId {
        self.identity.operation
    }
    pub fn parent_operation(&self) -> Option<DomSourceOperationId> {
        self.parent
    }
    pub fn identity(&self) -> DomSourceEventId {
        self.identity
    }
    pub fn subject(&self) -> NodeId {
        self.subject
    }
    /// Actual ToggleEvent data, absent for a plain Close notification.
    pub fn toggle_event(&self) -> Option<DomSourceToggleEvent> {
        match self.payload {
            DomSourceEventPayload::Toggle(payload) => Some(payload),
            DomSourceEventPayload::Close | DomSourceEventPayload::Cancel => None,
        }
    }
    pub fn cancelable(&self) -> bool {
        self.cancelable
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
    work: Arc<CallbackWork>,
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
    pub(crate) work: Arc<CallbackWork>,
    pub(crate) outcome: DomSourceEventOutcome,
}
/// Continuous charge for actual external job/event/receipt storage. Backing
/// owners must drop before this guard; acquire both guards across replacement.
#[derive(Debug)]
pub struct DomSourceEventRetention {
    pub(crate) work: Arc<CallbackWork>,
    pub(crate) custody: Option<crate::store::SourceRetention>,
    pub(crate) bytes: usize,
    pub(crate) units: usize,
}
/// The original event's admitted payload charge, independent of the event's
/// work, request, and execution scope. Drop the backing before this receipt.
#[derive(Debug)]
pub struct DomSourceEventPayloadRetention {
    debit: crate::store::custody::Debit,
}
impl DomSourceEventPayloadRetention {
    pub(crate) fn into_debit(self) -> crate::store::custody::Debit {
        self.debit
    }
}
impl DomSourceEventRetention {
    /// Bytes still held by this affine receipt.
    pub const fn retained_bytes(&self) -> usize {
        self.bytes
    }

    /// Transfer an admitted subextent to backing that can outlive the
    /// original event's work. The canonical storage debit remains live while
    /// the transferred portion leaves the request's diagnostic mirrors.
    pub fn split_payload(
        &mut self,
        bytes: usize,
        units: usize,
    ) -> Result<DomSourceEventPayloadRetention, DomError> {
        let remaining_bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let remaining_units = self
            .units
            .checked_sub(units)
            .ok_or(DomError::RepresentationExhausted)?;
        let custody = self
            .custody
            .as_mut()
            .ok_or(DomError::InvalidSourceEvent)?
            .split(crate::store::custody::Usage::new(bytes, units))?;
        self.bytes = remaining_bytes;
        self.units = remaining_units;
        self.work.bytes.fetch_sub(bytes, Ordering::Relaxed);
        self.work.units.fetch_sub(units, Ordering::Relaxed);
        Ok(DomSourceEventPayloadRetention {
            debit: custody.into_payload_debit(),
        })
    }

    /// Transfer a checked resource subextent from this same original request.
    /// This grants no dispatch, publication or Source execution permission.
    pub fn split(&mut self, bytes: usize, units: usize) -> Result<Self, DomError> {
        let remaining_bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let remaining_units = self
            .units
            .checked_sub(units)
            .ok_or(DomError::RepresentationExhausted)?;
        let custody = self
            .custody
            .as_mut()
            .map(|owned| owned.split(crate::store::custody::Usage::new(bytes, units)))
            .transpose()?;
        self.bytes = remaining_bytes;
        self.units = remaining_units;
        Ok(Self {
            work: self.work.clone(),
            custody,
            bytes,
            units,
        })
    }
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
        if let Some(incoming) = other.custody.take() {
            match self.custody.as_mut() {
                Some(owned) => owned.absorb(incoming)?,
                None => self.custody = Some(incoming),
            }
        }
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
        let request = dom.qualify_source_event(&request.work, 0)?;
        request.validate_executor(&self.binding)?;
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
        let request = dom.qualify_source_event(&permit.work, 1)?;
        request.validate_executor(&self.binding)?;
        permit.work.step(cancel)?;
        let retained = dom.retain_source_event_payload(
            request,
            std::mem::size_of::<DispatchScope>(),
            0,
            cancel,
        )?;
        let scope = permit
            .work
            .open_scope()
            .ok_or(DomError::InvalidSourceEvent)?;
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
        let request = dom.qualify_source_event(&actual.work, 1)?;
        request.validate_executor(&self.binding)?;
        actual.work.step(cancel)?;
        if !actual.work.scope_closed() {
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
