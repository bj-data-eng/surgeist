//! Synchronous native DOM event dispatch over the canonical DOM owner.
//!
//! Implements the pinned DOM3071 dispatch/invoke and parent algorithms for the
//! ordinary Node, Global and standalone dispatch, and selected Style, Details
//! and Dialog delivery.
//! Authored Event bodies retain their exact UTF-16 type and creation observation.
//! The affine dispatch owns one invocation's path, cursor and admission. Details
//! preserves its queued state and Dialog supplies its source-selected cancelability.
//! Both selected ToggleEvent sources are null. Registration supports capture and
//! once, passive listeners, borrowed Rust callbacks and owned host invocations
//! at the Source front.
//! A qualifying hosted Document's parent is its actual native Global. Selected
//! family owners validate the actual Source operation and advance only on a
//! matching one-shot receipt. Callback observations confer no producer authority.
//!
//! A started path retains exact identities even if callbacks detach, adopt or
//! destroy its nodes. Retirement does not redirect a listener to a reused slot;
//! canonical mutations using a retired currentTarget fail through DOM normally.
//! A retained source task owns the next invocation/listener, failures and
//! admission reservation. Dropping that private owner abandons dispatch without
//! claiming completion; public observation clones cannot retain its reservation.
//! Native listeners have no JavaScript callback realm. Their current target's
//! hosted Document activity is checked per listener; unavailable native authority
//! pauses before once removal. This does not implement listener-realm checks.
//! Replacing either mutable owner during a callback retains the dispatch; a
//! replacement registry cannot run its listeners on the original owner's path.
//! Unwinding callbacks are reported alongside ordinary callback failures and
//! later listeners continue. With panic=abort, Rust terminates the process.

use crate::dom_lifecycle::EventCreation;
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use surgeist_dom::{
    Dom, DomDispatchCustody, DomError, DomEventParent, DomEventParentQuery, DomEventRecipient,
    DomHostedError, DomParsedHtmlStyleNotificationRequirement, DomSnapshot, HtmlShadowRootMode,
    NodeId,
};
mod storage;
pub(crate) use storage::EventStorage;
pub use storage::{DomEventStorageLimits, DomEventType};
use storage::{EventAllocation, equal_units};
mod objects;
pub use objects::{
    DomEventConstructionError, DomEventCreationObservation, DomEventInit,
    DomEventTimestampUnavailable,
};
mod invocation;
pub use invocation::DomEventDispatch;
mod target;
pub use target::{DomEventTarget, DomStandaloneEventTarget};
mod passive;
use passive::{BorrowedPassive, PassiveScope};

/// Scoped access to the original canonical DOM. The mutable owner cannot be
/// extracted or replaced. A fresh reborrow is supplied to every listener.
pub struct DomEventDom<'a> {
    dom: &'a mut Dom,
}
impl<'a> DomEventDom<'a> {
    /// Borrows a standalone owner through the same controlled event view.
    pub fn borrow(dom: &'a mut Dom) -> Self {
        Self { dom }
    }
    /// Exact canonical owner identity.
    pub fn owner(&self) -> surgeist_dom::DomOwnerId {
        self.dom.owner()
    }
    /// Current finite canonical limits.
    pub fn limits(&self) -> surgeist_dom::DomLimits {
        self.dom.limits()
    }
    /// Current canonical usage.
    pub fn usage(&self) -> surgeist_dom::DomUsage {
        self.dom.usage()
    }
    /// An immutable historical capture, without mutable resource custody.
    pub fn snapshot(&self) -> DomSnapshot {
        self.dom.snapshot()
    }
    /// Performs an ordinary atomic edit. Edits needing source hooks use their
    /// Runtime family driver, which retains all source and task responsibility.
    pub fn apply(
        &mut self,
        batch: surgeist_dom::DomBatch,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<surgeist_dom::DomCommit, DomError> {
        self.dom.apply(batch, cancel)
    }
    /// Current outstanding Details intentions, without transferring them.
    pub fn pending_details_toggle_intentions(&self) -> usize {
        self.dom.pending_details_toggle_intentions()
    }
    /// Transfers actual Details intentions to their task owner.
    #[cfg(test)]
    pub(crate) fn take_details_toggle_intentions(
        &mut self,
        max_output: usize,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<surgeist_dom::DomDetailsToggleTransfer, DomError> {
        self.dom.take_details_toggle_intentions(max_output, cancel)
    }
    /// Actual parked source status, including unpublished formation. This read
    /// lends no source lease or notification token.
    pub fn pending_source_edit(&self) -> Option<surgeist_dom::DomSourceStatus> {
        self.dom.pending_source_edit()
    }
    /// Live canonical invocation observation, without execution or reply authority.
    pub fn fullscreen_exit_status(
        &self,
        invocation: surgeist_dom::DomFullscreenExitId,
    ) -> Result<surgeist_dom::DomFullscreenExitStatus, DomError> {
        self.dom.fullscreen_exit_status(invocation)
    }
    /// Atomically transfers the recorded Document's actual pending fullscreen
    /// tuples. This does not choose targets or deliver events.
    pub fn take_fullscreen_events(
        &mut self,
        document: surgeist_dom::DocumentId,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<surgeist_dom::DomFullscreenEventBatch, DomError> {
        self.dom
            .take_fullscreen_events(document, max_output, cancel)
    }
    /// Reissues an independently owned capability to an already retained current
    /// root; it cannot replace the store or establish a new association.
    pub fn style_cssom_root(
        &self,
        root: NodeId,
    ) -> Result<Option<surgeist_dom::DomStyleCssomRoot>, DomError> {
        self.dom.style_cssom_root(root)
    }
    pub(crate) fn raw_mut(&mut self) -> &mut Dom {
        self.dom
    }
    pub(crate) fn raw(&self) -> &Dom {
        self.dom
    }
    pub(crate) fn begin_source_edit(
        &mut self,
        edit: surgeist_dom::DomSourceEdit,
        limits: surgeist_dom::DomSourceLimits,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<surgeist_dom::DomSourceLease<'_>, DomError> {
        self.dom.begin_source_edit(edit, limits, cancel)
    }
    pub(crate) fn resume_source_edit(
        &mut self,
        id: surgeist_dom::DomSourceOperationId,
        limits: surgeist_dom::DomSourceLimits,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<surgeist_dom::DomSourceLease<'_>, DomError> {
        self.dom.resume_source_edit(id, limits, cancel)
    }
}

static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(1);

/// Source invocation phase, separate from listener capture selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventPhase {
    /// No callback is currently being invoked.
    None,
    /// Ancestor capture.
    Capturing,
    /// A path item with a non-null shadow-adjusted target.
    AtTarget,
    /// Ancestor bubbling.
    Bubbling,
}
/// All registration options this native boundary supports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomEventListenerOptions {
    /// Select the capture invocation.
    pub capture: bool,
    /// Remove this exact registration before calling its callback.
    pub once: bool,
    /// Explicit passive policy. None selects the default for this exact type
    /// and target when the listener is registered.
    pub passive: Option<bool>,
}
/// Fixed registry storage and simultaneously owned dispatch bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventRegistryLimits {
    /// Live registration slots; invocation snapshots retain exact IDs separately.
    pub registrations: usize,
    /// Simultaneously running or retained events owned by this registry.
    pub pending_events: usize,
    /// Simultaneously executing nested dispatch calls.
    pub nested_dispatches: usize,
    /// Actual listener, type, index and ordinary invocation backing.
    pub storage_bytes: usize,
    /// Cumulative registration and ordinary preparation work.
    pub work: u64,
}
/// Per-call admission. Resume may supply more capacity without rebuilding a path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventDispatchLimits {
    /// Frozen event path item count.
    pub path_items: usize,
    /// One invoke step's complete listener-list snapshot.
    pub snapshot_listeners: usize,
    /// Total retained callback failure records for this event.
    pub failure_records: usize,
    /// Callback invocations in this call, including failures and panics.
    pub callbacks: usize,
    /// Source traversal, registry scanning and listener processing in this call.
    pub work: u64,
}
/// Exact registry, slot and generation; a removed registration never aliases reuse.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomEventRegistrationId {
    registry: u64,
    slot: usize,
    generation: u64,
}
/// Callback-reported failure, independent from event cancellation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventCallbackError {
    /// Caller-owned diagnostic code, interpreted by the composing application.
    pub code: u64,
}
/// Why a listener invocation reported failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventListenerFailureKind {
    /// The callback returned a typed error.
    Reported(DomEventCallbackError),
    /// Rust unwound while executing the callback.
    Panicked,
}
/// One failure from an actual callback invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomEventListenerFailure {
    /// Exact registration that ran.
    pub registration: DomEventRegistrationId,
    /// Exact invocation target on the fixed path.
    pub current_target: DomEventTarget,
    /// Failure independently of the event's canceled flag.
    pub kind: DomEventListenerFailureKind,
}
/// A finite resource whose admission refused further work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventResource {
    /// Actual retained Event, EventTarget, listener and invocation backing.
    Storage,
    /// Listener registration storage.
    Registrations,
    /// Running and retained dispatch state.
    PendingEvents,
    /// Active nested execution.
    NestedDispatches,
    /// Event path storage.
    PathItems,
    /// An invoke step's listener snapshot.
    SnapshotListeners,
    /// Callback error output.
    FailureRecords,
    /// Callback execution count.
    Callbacks,
    /// Traversal and listener work.
    Work,
    /// Fallible allocation refused storage.
    Allocation,
}
/// Failure before starting delivery, or an invalid dispatch transition.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomEventDispatchError {
    /// Canonical identity or source-coherence validation failed before the next step.
    Dom(DomError),
    /// Actual hosted source or global association could not be qualified.
    Hosted(DomHostedError),
    /// Finite admission refused work; no callback ran for a new event.
    Admission(DomEventResource),
    /// An activated original parsed contribution has unfinished synchronous work.
    ParsedSynchronousPending {
        /// Exact original invocation, occurrence or unavailable frontier.
        requirement: DomParsedHtmlStyleNotificationRequirement,
    },
    /// This same event is already executing or retained.
    AlreadyDispatching,
    /// The receiving Document created this legacy Event before initialization.
    Uninitialized,
    /// This invocation already produced its one completion receipt.
    AlreadyCompleted,
    /// The affine invocation was terminally abandoned.
    Abandoned,
    /// Resume requires an event retained by this exact registry.
    WrongContinuation,
    /// The standalone target belongs to another dispatcher registry.
    WrongTargetRegistry,
    /// The selected host refused execution; this event cannot produce a receipt.
    HostRefused(DomEventHostError),
    /// Registry/generation/order identity exhausted rather than wrapping.
    IdentityExhausted,
    /// A canonical event-parent chain repeated a node.
    CyclicPath,
}
impl std::fmt::Display for DomEventDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dom(error) => write!(f, "native DOM event dispatch: {error}"),
            Self::Hosted(error) => write!(f, "native DOM event hosted endpoint: {error}"),
            Self::Admission(resource) => write!(
                f,
                "native DOM event dispatch admission refused {resource:?}"
            ),
            Self::ParsedSynchronousPending { requirement } => write!(
                f,
                "original parsed synchronous execution is pending: {requirement:?}"
            ),
            Self::AlreadyDispatching => f.write_str("native DOM event is already dispatching"),
            Self::Uninitialized => f.write_str("native DOM event is uninitialized"),
            Self::AlreadyCompleted => f.write_str("native DOM event already completed dispatch"),
            Self::Abandoned => f.write_str("native DOM event source task was abandoned"),
            Self::WrongContinuation => {
                f.write_str("native DOM event continuation is not retained by this dispatcher")
            }
            Self::WrongTargetRegistry => {
                f.write_str("standalone EventTarget belongs to another dispatcher")
            }
            Self::HostRefused(error) => write!(f, "native DOM event host refused: {}", error.code),
            Self::IdentityExhausted => {
                f.write_str("native DOM event registration identity is exhausted")
            }
            Self::CyclicPath => f.write_str("native DOM event parent path contains a cycle"),
        }
    }
}
impl std::error::Error for DomEventDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Hosted(error) => Some(error),
            _ => None,
        }
    }
}
impl From<DomError> for DomEventDispatchError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}
impl From<DomHostedError> for DomEventDispatchError {
    fn from(error: DomHostedError) -> Self {
        match error {
            DomHostedError::Dom(error) => Self::Dom(error),
            other => Self::Hosted(other),
        }
    }
}
/// Real completed delivery. This value is deliberately not cloneable.
#[derive(Debug)]
pub struct DomEventDispatchReceipt {
    origin: DispatchOrigin,
    event_type: DomEventType,
    callbacks: usize,
    failures: Vec<DomEventListenerFailure>,
    default_prevented: bool,
    funding: Option<DispatchFunding>,
}
impl DomEventDispatchReceipt {
    pub(crate) fn native_cause(&self) -> Option<NativeEventCause> {
        self.origin.cause()
    }
    pub(crate) fn release_producer_retention(&mut self) {
        self.funding.take();
    }
    /// Original exact target, independent of later retirement/reparenting.
    pub fn original_target(&self) -> DomEventTarget {
        self.origin.target()
    }
    /// Type that was actually dispatched.
    pub fn event_type(&self) -> DomEventType {
        self.event_type.clone()
    }
    /// Number of callbacks actually entered, including failures.
    pub fn callbacks(&self) -> usize {
        self.callbacks
    }
    /// Errors and unwinds from actual callbacks, in delivery order.
    pub fn failures(&self) -> &[DomEventListenerFailure] {
        &self.failures
    }
    /// Actual canceled flag at completion, independently of listener failures.
    pub const fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}
/// Result of executing as far as current admission permits.
#[derive(Debug)]
pub enum DomEventDispatchOutcome {
    /// An admitted host listener is active. Release all caller loans before
    /// executing it, report its result once, then service the original event.
    HostInvocation(DomEventHostInvocation),
    /// This producer does not offer the owned host invocation capability. The
    /// next host registration remains untouched, including its once option.
    RetainedHostUnavailable,
    /// All source invoke steps and event cleanup completed.
    Complete(DomEventDispatchReceipt),
    /// The event retains its exact next step; resume it without replay.
    Retained(DomEventResource),
    /// Activated original synchronous work blocks the next invocation. The
    /// exact continuation retains all previously entered callback progress.
    RetainedParsedSynchronous {
        /// Actual original invocation, occurrence or unavailable frontier.
        requirement: DomParsedHtmlStyleNotificationRequirement,
    },
    /// Canonical original-frontier inspection refused its actual guard, work
    /// or cancellation admission. The next listener remains retained.
    RetainedParsedAdmission(DomError),
    /// A callback published an unfinished canonical source hook tail. Finish
    /// that exact DOM operation before resuming; no listener is replayed.
    RetainedSourceOperation(surgeist_dom::DomSourceOperationId),
    /// A published native creator tail must complete before later invocations.
    /// Coherent queued destruction does not trigger this state.
    RetainedHostedOperation(surgeist_dom::DomHostedOperationId),
    /// The next listener's current hosted authority is unavailable. Its exact
    /// cursor remains retained; previously entered callbacks are not replayed.
    RetainedHostedAuthority(DomHostedError),
    /// The caller replaced the actual canonical owner during a callback. Restore
    /// the original owner before resuming this exact path.
    RetainedForeignOwner,
    /// A callback replaced the listener registry. Restore the exact original
    /// registry before resuming; replacement listeners have not been invoked.
    RetainedForeignRegistry,
    /// A non-callback Rust unwind interrupted dispatch; its next step is retained.
    /// Callback unwinds are normally recorded and execution continues instead.
    RetainedAfterUnwind,
}

type Listener<C> = dyn Fn(
    &mut DomEventDom<'_>,
    &mut DomEventDispatcher<C>,
    &mut C,
    &NativeDomEvent,
) -> Result<(), DomEventCallbackError>;
/// Shared callback identity. Cloning preserves duplicate-registration identity.
pub struct DomEventCallback<C>(CallbackKind<C>);
enum CallbackKind<C> {
    Native(Rc<Listener<C>>),
    Host(Rc<u64>),
    #[cfg(feature = "document-scripting")]
    Guest(Rc<crate::document_scripting::guest::RegisteredGuestListener>),
}
impl<C> Clone for DomEventCallback<C> {
    fn clone(&self) -> Self {
        Self(match &self.0 {
            CallbackKind::Native(callback) => CallbackKind::Native(callback.clone()),
            CallbackKind::Host(key) => CallbackKind::Host(key.clone()),
            #[cfg(feature = "document-scripting")]
            CallbackKind::Guest(callback) => CallbackKind::Guest(callback.clone()),
        })
    }
}
impl<C> DomEventCallback<C> {
    /// A synchronous callback with caller-owned composition context. No owner or
    /// registry cell guard is held while it runs, including during nested dispatch.
    pub fn new(
        callback: impl Fn(
            &mut DomEventDom<'_>,
            &mut DomEventDispatcher<C>,
            &mut C,
            &NativeDomEvent,
        ) -> Result<(), DomEventCallbackError>
        + 'static,
    ) -> Self {
        Self(CallbackKind::Native(Rc::new(callback)))
    }
}
impl<C> DomEventCallback<C> {
    /// An external synchronous host listener. The key is caller routing data,
    /// never identity or execution authority. Clones share callback identity;
    /// independently constructed callbacks remain distinct even with equal keys.
    /// Source event servicing offers this execution capability. Other native
    /// producers retain at a host listener without consuming it.
    pub fn host(key: u64) -> Self {
        Self(CallbackKind::Host(Rc::new(key)))
    }
    #[cfg(feature = "document-scripting")]
    pub(crate) fn guest(
        callback: Rc<crate::document_scripting::guest::RegisteredGuestListener>,
    ) -> Self {
        Self(CallbackKind::Guest(callback))
    }
    fn same_identity(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (CallbackKind::Native(a), CallbackKind::Native(b)) => Rc::ptr_eq(a, b),
            (CallbackKind::Host(a), CallbackKind::Host(b)) => Rc::ptr_eq(a, b),
            #[cfg(feature = "document-scripting")]
            (CallbackKind::Guest(a), CallbackKind::Guest(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
/// Host execution refusal, distinct from an ordinary guest callback error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventHostError {
    /// Caller-owned host diagnostic code.
    pub code: u64,
}
/// Affine permission to execute one already-selected external listener. It owns
/// no session, dispatcher or composition borrow. The original event owner keeps
/// its dispatch and Source scope alive; this token cannot outlive that authority.
/// Dropping an unreported token abandons delivery without a completion receipt.
pub struct DomEventHostInvocation {
    event: NativeDomEvent,
    custody: ContinuationObservation,
    sequence: usize,
    registration: DomEventRegistrationId,
    key: u64,
    #[cfg(feature = "document-scripting")]
    guest: Option<Rc<crate::document_scripting::guest::RegisteredGuestListener>>,
    #[cfg(feature = "document-scripting")]
    once_removed: bool,
    reported: bool,
}
impl std::fmt::Debug for DomEventHostInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomEventHostInvocation")
            .field("event", &self.event)
            .field("registration", &self.registration)
            .field("key", &self.key)
            .field("reported", &self.reported)
            .finish()
    }
}
impl DomEventHostInvocation {
    /// Caller routing data of this exact callback identity.
    pub const fn key(&self) -> u64 {
        self.key
    }
    /// The registration selected before once removal or any host execution.
    pub const fn registration(&self) -> DomEventRegistrationId {
        self.registration
    }
    /// Authentic live event observation and propagation/default controls.
    pub fn event(&self) -> &NativeDomEvent {
        &self.event
    }
    #[cfg(feature = "document-scripting")]
    pub(crate) fn selected_guest(
        &self,
    ) -> Option<&crate::document_scripting::guest::RegisteredGuestListener> {
        self.guest.as_deref()
    }
    #[cfg(feature = "document-scripting")]
    pub(crate) const fn once_removed(&self) -> bool {
        self.once_removed
    }

    /// Record ordinary guest completion once. A guest error joins normal
    /// callback failure reporting; later listeners remain eligible. This closes
    /// the active callback scope but runs neither another listener nor a Source
    /// tail. Service the original event again to continue its exact cursor.
    /// A foreign dispatcher leaves this token and the original scope untouched.
    pub fn complete<C>(
        &mut self,
        dispatcher: &DomEventDispatcher<C>,
        result: Result<(), DomEventCallbackError>,
    ) -> Result<(), DomEventDispatchError> {
        self.report(dispatcher, HostResult::Returned(result))
    }
    /// Record a host refusal once. It closes reentry and terminally
    /// prevents a successful event receipt. Subsequent servicing returns this
    /// same first refusal; it does not reinterpret it as a guest exception.
    pub fn refuse<C>(
        &mut self,
        dispatcher: &DomEventDispatcher<C>,
        error: DomEventHostError,
    ) -> Result<(), DomEventDispatchError> {
        self.report(dispatcher, HostResult::Refused(error))
    }
    fn report<C>(
        &mut self,
        dispatcher: &DomEventDispatcher<C>,
        result: HostResult,
    ) -> Result<(), DomEventDispatchError> {
        if self.reported {
            return Err(DomEventDispatchError::AlreadyCompleted);
        }
        let custody = self
            .custody
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)?;
        let state = &self.custody.state;
        if state.status.get() == Status::Abandoned {
            return Err(DomEventDispatchError::Abandoned);
        }
        let mut parked = custody
            .dispatch
            .try_borrow_mut()
            .map_err(|_| DomEventDispatchError::AlreadyDispatching)?;
        let dispatch = parked
            .as_ref()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        if !Rc::ptr_eq(&dispatch._reservation.0, &dispatcher.owner) {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        let mut frame = custody
            .host
            .take()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        if frame.sequence != self.sequence || frame.registration != self.registration {
            custody.host.set(Some(frame));
            return Err(DomEventDispatchError::WrongContinuation);
        }
        if frame.result.is_some() {
            custody.host.set(Some(frame));
            return Err(DomEventDispatchError::AlreadyCompleted);
        }
        frame.scope.take();
        frame.running.take();
        custody.clear_invocation();
        match result {
            HostResult::Returned(result) => {
                frame.result = Some(result);
                custody.host.set(Some(frame));
                state.status.set(Status::Retained);
            }
            HostResult::Refused(error) => {
                state.status.set(Status::HostRefused(error));
                drop(parked.take());
                custody.finish_dispatch(state);
            }
        }
        self.reported = true;
        Ok(())
    }
    pub(crate) fn retain_source_scope(
        &self,
        scope: surgeist_dom::DomSourceEventDispatchGuard,
    ) -> Result<(), DomEventDispatchError> {
        let custody = self
            .custody
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)?;
        let mut frame = custody
            .host
            .take()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        if frame.sequence != self.sequence || frame.scope.is_some() || frame.result.is_some() {
            custody.host.set(Some(frame));
            return Err(DomEventDispatchError::WrongContinuation);
        }
        frame.scope = Some(scope);
        custody.host.set(Some(frame));
        Ok(())
    }
}
impl Drop for DomEventHostInvocation {
    fn drop(&mut self) {
        if !self.reported
            && let Some(custody) = self.custody.upgrade()
        {
            let frame = custody.host.take();
            if frame
                .as_ref()
                .is_some_and(|frame| frame.sequence == self.sequence)
            {
                custody.abandon(&self.custody.state);
                // No continuation loan crosses a host boundary. If the
                // source owner is already disposing it, its own drop owns
                // the same dispatch cleanup.
                if let Ok(mut dispatch) = custody.dispatch.try_borrow_mut() {
                    drop(dispatch.take());
                }
                custody.finish_dispatch(&self.custody.state);
            } else {
                custody.host.set(frame);
            }
        }
    }
}
#[derive(Clone, Copy)]
enum HostResult {
    Returned(Result<(), DomEventCallbackError>),
    Refused(DomEventHostError),
}
struct HostFrame {
    sequence: usize,
    registration: DomEventRegistrationId,
    current_target: DomEventTarget,
    result: Option<Result<(), DomEventCallbackError>>,
    running: Option<RunningReservation>,
    scope: Option<surgeist_dom::DomSourceEventDispatchGuard>,
}
struct Registration<C> {
    target: DomEventTarget,
    event_type: DomEventType,
    capture: bool,
    once: bool,
    passive: bool,
    callback: DomEventCallback<C>,
    order: u64,
    // The actual callback occurrence drops before its positive DOM lease.
    _recipient: Option<DomEventRecipient>,
}
struct Slot<C> {
    generation: u64,
    registration: Option<Registration<C>>,
}
struct RegistryOwner {
    id: u64,
    pending: Cell<usize>,
    running: Cell<usize>,
}
struct PendingReservation(Rc<RegistryOwner>);
impl Drop for PendingReservation {
    fn drop(&mut self) {
        self.0.pending.set(self.0.pending.get() - 1);
    }
}
struct RunningReservation(Rc<RegistryOwner>);
impl Drop for RunningReservation {
    fn drop(&mut self) {
        self.0.running.set(self.0.running.get() - 1);
    }
}
enum RegistryAssociation {
    Unbound,
    Bound(surgeist_dom::DomEventRegistryAssociation),
}
struct PreparedRegistry {
    // Backing is released before its provisional canonical debit.
    pending: Vec<ContinuationObservation>,
    funding: EventAllocation,
    association: surgeist_dom::DomEventRegistryPreparation,
}
impl RegistryAssociation {
    fn validate(&self, dom: &Dom) -> Result<(), DomEventDispatchError> {
        match self {
            Self::Unbound => Ok(()),
            Self::Bound(association) => association.validate(dom).map_err(Into::into),
        }
    }
    fn prepare(
        &self,
        dom: &mut Dom,
        pending_events: usize,
        storage: &EventStorage,
        cancel: &AtomicBool,
    ) -> Result<Option<PreparedRegistry>, DomEventDispatchError> {
        match self {
            Self::Unbound => {
                let bytes = pending_events
                    .checked_mul(std::mem::size_of::<ContinuationObservation>())
                    .ok_or(DomEventDispatchError::Admission(
                        DomEventResource::Allocation,
                    ))?;
                let association = dom.prepare_event_registry(bytes, cancel)?;
                let funding =
                    storage.reserve(&mut DomEventDom::borrow(dom), bytes, 0, 0, cancel)?;
                let mut pending = Vec::new();
                pending
                    .try_reserve_exact(pending_events)
                    .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
                if pending.capacity() != pending_events {
                    return Err(DomEventDispatchError::Admission(
                        DomEventResource::PendingEvents,
                    ));
                }
                Ok(Some(PreparedRegistry {
                    pending,
                    funding,
                    association,
                }))
            }
            Self::Bound(_) => Ok(None),
        }
    }
    fn commit(
        &mut self,
        prepared: Option<PreparedRegistry>,
        pending: &mut Vec<ContinuationObservation>,
        pending_funding: &mut Option<EventAllocation>,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomEventDispatchError> {
        if let Some(prepared) = prepared {
            let PreparedRegistry {
                pending: backing,
                funding,
                association,
            } = prepared;
            let association = match association.commit(dom, cancel) {
                Ok(association) => association,
                Err((error, preparation)) => {
                    drop(backing);
                    drop(funding);
                    drop(preparation);
                    return Err(error.into());
                }
            };
            *pending = backing;
            *pending_funding = Some(funding);
            *self = Self::Bound(association);
        }
        Ok(())
    }
}
/// Runtime's one native listener registry and path/invocation owner.
/// `C` is the application's actual mutable composition state, e.g. its CSSOM
/// and Style coordinator; the dispatcher does not own those family semantics.
pub struct DomEventDispatcher<C> {
    owner: Rc<RegistryOwner>,
    limits: DomEventRegistryLimits,
    slots: Vec<Slot<C>>,
    next_order: u64,
    pending: Vec<ContinuationObservation>,
    storage: EventStorage,
    slots_funding: Option<EventAllocation>,
    pending_funding: Option<EventAllocation>,
    // Drops after the actual callback list. Dispatch reservations hold no lease:
    // abandoning this dispatcher withdraws its canonical association immediately.
    association: RegistryAssociation,
}
impl<C> DomEventDispatcher<C> {
    /// Establish an empty unbound registry. Its first accepted registration or
    /// dispatch associates it irreversibly with the canonical DOM's one live
    /// listener owner. A second live owner cannot select a parallel listener list.
    /// Zero bounds intentionally admit no work.
    pub fn new(limits: DomEventRegistryLimits) -> Result<Self, DomEventDispatchError> {
        let storage = EventStorage::registry(DomEventStorageLimits {
            storage_bytes: limits.storage_bytes,
            work: limits.work,
        })?;
        let id = NEXT_REGISTRY
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| DomEventDispatchError::IdentityExhausted)?;
        Ok(Self {
            owner: Rc::new(RegistryOwner {
                id,
                pending: Cell::new(0),
                running: Cell::new(0),
            }),
            limits,
            slots: Vec::new(),
            next_order: 0,
            pending: Vec::new(),
            storage,
            slots_funding: None,
            pending_funding: None,
            association: RegistryAssociation::Unbound,
        })
    }
    /// Claim this canonical DOM's listener registry without adding a listener
    /// or dispatching an Event. A refused claim leaves the dispatcher unbound
    /// and available for a later attempt. An already-bound dispatcher only
    /// validates its original authority and does not repeat admission.
    pub fn bind(
        &mut self,
        dom: &mut DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomEventDispatchError> {
        if let RegistryAssociation::Bound(association) = &self.association {
            return association.validate(dom.raw()).map_err(Into::into);
        }
        self.storage.step(1, cancel)?;
        let prepared = self.association.prepare(
            dom.raw_mut(),
            self.limits.pending_events,
            &self.storage,
            cancel,
        )?;
        self.association.commit(
            prepared,
            &mut self.pending,
            &mut self.pending_funding,
            dom.raw_mut(),
            cancel,
        )
    }

    /// Add in deterministic order. The target/type/callback identity/capture key
    /// deduplicates; changing `once` on a duplicate does not change the listener.
    /// Load/Error registrations retain a finite canonical recipient lease until
    /// this exact registration is removed, invoked once, or its dispatcher drops.
    pub fn add_listener(
        &mut self,
        dom: &mut DomEventDom<'_>,
        target: impl Into<DomEventTarget>,
        event_type: impl AsRef<[u16]>,
        callback: DomEventCallback<C>,
        options: DomEventListenerOptions,
    ) -> Result<DomEventRegistrationId, DomEventDispatchError> {
        self.association.validate(dom.raw())?;
        // Public registration is a synchronous noncancellable call.
        let synchronous = AtomicBool::new(false);
        let event_type = event_type.as_ref();
        let target = target.into();
        self.validate_target(dom.raw(), &target, &synchronous)?;
        let mut registrations = 0;
        let mut reusable = None;
        for (slot, entry) in self.slots.iter().enumerate() {
            self.storage.step(1, &synchronous)?;
            let Some(r) = &entry.registration else {
                if reusable.is_none() && entry.generation < u64::MAX {
                    reusable = Some(slot);
                }
                continue;
            };
            registrations += 1;
            if r.target == target
                && r.capture == options.capture
                && r.callback.same_identity(&callback)
                && equal_units(r.event_type.code_units(), event_type, || {
                    self.storage.step(1, &synchronous)
                })?
            {
                return Ok(DomEventRegistrationId {
                    registry: self.owner.id,
                    slot,
                    generation: entry.generation,
                });
            }
        }
        if registrations >= self.limits.registrations {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::Registrations,
            ));
        }
        let order = self
            .next_order
            .checked_add(1)
            .ok_or(DomEventDispatchError::IdentityExhausted)?;
        let passive = match options.passive {
            Some(value) => value,
            None => self.default_passive(dom.raw(), &target, event_type, &synchronous)?,
        };
        let event_type = DomEventType::retain(&self.storage, dom, event_type, &synchronous)?;
        let mut replacement = None;
        if reusable.is_none() {
            if self.slots.len() >= self.limits.registrations {
                return Err(DomEventDispatchError::IdentityExhausted);
            }
            if self.slots.len() == self.slots.capacity() {
                self.storage.step(self.slots.len(), &synchronous)?;
                let capacity = self
                    .slots
                    .len()
                    .checked_add(1)
                    .ok_or(DomEventDispatchError::IdentityExhausted)?;
                let bytes = capacity
                    .checked_mul(std::mem::size_of::<Slot<C>>())
                    .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
                let funding = self.storage.reserve(dom, bytes, bytes, 0, &synchronous)?;
                let mut slots = Vec::new();
                slots
                    .try_reserve_exact(capacity)
                    .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
                if slots.capacity() != capacity {
                    return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
                }
                replacement = Some((slots, funding));
            }
        }
        // Every fallible Runtime preparation precedes acquiring the canonical
        // registration. A refused first registration drops its provisional
        // association; duplicates retain their original association and lease.
        let association = self.association.prepare(
            dom.raw_mut(),
            self.limits.pending_events,
            &self.storage,
            &synchronous,
        )?;
        let recipient = if let Some(target) = target.recipient() {
            dom.raw_mut().retain_event_recipient(
                target,
                event_type.code_units(),
                options.capture,
                &synchronous,
            )?
        } else {
            None
        };
        self.association.commit(
            association,
            &mut self.pending,
            &mut self.pending_funding,
            dom.raw_mut(),
            &synchronous,
        )?;
        if let Some((mut slots, funding)) = replacement.take() {
            slots.append(&mut self.slots);
            drop(std::mem::replace(&mut self.slots, slots));
            self.slots_funding = Some(funding);
        }
        let slot = if let Some(slot) = reusable {
            slot
        } else {
            self.slots.push(Slot {
                generation: 0,
                registration: None,
            });
            self.slots.len() - 1
        };
        let entry = &mut self.slots[slot];
        entry.generation += 1;
        entry.registration = Some(Registration {
            target,
            event_type,
            capture: options.capture,
            once: options.once,
            passive,
            callback,
            order,
            _recipient: recipient,
        });
        self.next_order = order;
        Ok(DomEventRegistrationId {
            registry: self.owner.id,
            slot,
            generation: entry.generation,
        })
    }
    /// Remove exactly one generation. Repeated or stale removal returns false.
    pub fn remove_listener(&mut self, id: DomEventRegistrationId) -> bool {
        if id.registry != self.owner.id {
            return false;
        }
        let Some(slot) = self.slots.get_mut(id.slot) else {
            return false;
        };
        if slot.generation != id.generation {
            return false;
        }
        slot.registration.take().is_some()
    }
    fn registration(&self, id: DomEventRegistrationId) -> Option<&Registration<C>> {
        (id.registry == self.owner.id).then_some(())?;
        let slot = self.slots.get(id.slot)?;
        (slot.generation == id.generation).then_some(())?;
        slot.registration.as_ref()
    }
    #[cfg(all(test, feature = "document-scripting"))]
    pub(crate) fn has_registration(&self, id: DomEventRegistrationId) -> bool {
        self.registration(id).is_some()
    }

    #[cfg(feature = "document-scripting")]
    pub(crate) fn guest_registration(
        &self,
        id: DomEventRegistrationId,
    ) -> Option<Rc<crate::document_scripting::guest::RegisteredGuestListener>> {
        let CallbackKind::Guest(listener) = &self.registration(id)?.callback.0 else {
            return None;
        };
        Some(listener.clone())
    }

    #[cfg(feature = "document-scripting")]
    pub(crate) fn reap_guest_target_registrations(&mut self) {
        for slot in &mut self.slots {
            if slot.registration.as_ref().is_some_and(|entry| {
                matches!(&entry.callback.0, CallbackKind::Guest(listener) if listener.target_retired())
            }) {
                slot.registration.take();
            }
        }
    }

    #[cfg(feature = "document-scripting")]
    pub(crate) fn is_bound(&self) -> bool {
        matches!(self.association, RegistryAssociation::Bound(_))
    }
    #[cfg(feature = "document-scripting")]
    pub(crate) fn qualify_guest_registry(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<(), DomEventDispatchError> {
        self.association.validate(dom.raw())
    }
    #[cfg(feature = "document-scripting")]
    pub(crate) fn next_guest_registration(
        &self,
        target: &DomEventTarget,
        event_type: &[u16],
        capture: bool,
        cursor: &mut usize,
        cancel: &AtomicBool,
    ) -> Result<
        Option<(
            DomEventRegistrationId,
            Rc<crate::document_scripting::guest::RegisteredGuestListener>,
        )>,
        DomEventDispatchError,
    > {
        while let Some(slot) = self.slots.get(*cursor) {
            self.storage.step(1, cancel)?;
            let index = *cursor;
            *cursor += 1;
            if let Some(registration) = &slot.registration
                && let CallbackKind::Guest(callback) = &registration.callback.0
                && registration.target == *target
                && registration.capture == capture
                && equal_units(registration.event_type.code_units(), event_type, || {
                    self.storage.step(1, cancel)
                })?
            {
                return Ok(Some((
                    DomEventRegistrationId {
                        registry: self.owner.id,
                        slot: index,
                        generation: slot.generation,
                    },
                    callback.clone(),
                )));
            }
        }
        Ok(None)
    }
    pub(crate) fn deliver(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &EventDispatchObservation,
        call: DispatchCall<'_>,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        if event.state().status.get() != Status::Ready {
            self.continue_dispatch(dom, context, event, call)
        } else {
            self.start_dispatch(dom, context, event, call)
        }
    }

    pub(crate) fn qualify_terminal_error(
        &self,
        dom: &Dom,
        error: DomEventDispatchError,
    ) -> DomEventDispatchError {
        self.association.validate(dom).err().unwrap_or(error)
    }

    /// Validate and admit the entire path before the first callback.
    fn start_dispatch(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &EventDispatchObservation,
        call: DispatchCall<'_>,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        let DispatchCall {
            limits,
            cancel,
            host,
            mut funding,
        } = call;
        self.association.validate(dom)?;
        let _custody = event.custody()?;
        let state = event.state();
        match state.status.get() {
            Status::Ready => (),
            Status::Complete => return Err(DomEventDispatchError::AlreadyCompleted),
            Status::Abandoned => return Err(DomEventDispatchError::Abandoned),
            Status::HostRefused(error) => return Err(DomEventDispatchError::HostRefused(error)),
            _ => return Err(DomEventDispatchError::AlreadyDispatching),
        }
        event.check_dispatch_state()?;
        if event.0.owner != dom.owner() {
            return Err(DomError::ForeignOwner.into());
        }
        let event_type = event.event_type();
        self.admit_running()?;
        if self.owner.pending.get() >= self.limits.pending_events {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::PendingEvents,
            ));
        }
        let mut budget = Budget::new(limits, cancel);
        let snapshot = dom.snapshot();
        if dom.source_dispatch_blocker().is_some() {
            return Err(DomEventDispatchError::Dom(
                DomError::IncompleteSourceOperation,
            ));
        }
        if snapshot.incomplete_hosted_operation().is_some() {
            return Err(DomEventDispatchError::Hosted(
                DomHostedError::IncompleteOperation,
            ));
        }
        if let Some(requirement) = parsed_dispatch_requirement(dom, cancel, || {
            budget.tick().map_err(DomEventDispatchError::Admission)
        })? {
            return Err(DomEventDispatchError::ParsedSynchronousPending { requirement });
        }
        if matches!(self.association, RegistryAssociation::Unbound) {
            budget.tick().map_err(DomEventDispatchError::Admission)?;
        }
        let association =
            self.association
                .prepare(dom, self.limits.pending_events, &self.storage, cancel)?;
        let (path, clear_target, association_custody) = if let Some(node) = state.origin.node() {
            let mut path = build_node_path(
                dom,
                &snapshot,
                node,
                &event_type,
                event.composed(),
                state.origin.cause(),
                &mut budget,
            )?;
            let final_target = path
                .last()
                .and_then(|item| item.target.node())
                .ok_or(DomEventDispatchError::WrongContinuation)?;
            let final_root = root(&snapshot, final_target, &mut budget)?;
            let clear_target = snapshot.node(final_root)?.as_shadow_root().is_some();
            let custody = register_node_path_custody(dom, &mut path, &mut budget)?;
            (path, clear_target, Some(custody))
        } else {
            (
                build_direct_path(dom, state.origin.target(), &mut budget)?,
                false,
                None,
            )
        };
        let path_bytes = std::mem::size_of::<EventPath>()
            .checked_add(
                path.capacity()
                    .checked_mul(std::mem::size_of::<PathItem>())
                    .ok_or(DomEventDispatchError::Admission(
                        DomEventResource::Allocation,
                    ))?,
            )
            .ok_or(DomEventDispatchError::Admission(
                DomEventResource::Allocation,
            ))?;
        let path_funding = funding.split_path(path_bytes)?;
        self.owner.pending.set(self.owner.pending.get() + 1);
        let mut dispatch = Dispatch {
            association_custody,
            _reservation: PendingReservation(self.owner.clone()),
            event_type,
            path: Rc::new(EventPath {
                items: path,
                _funding: path_funding,
            }),
            invoke: 0,
            snapshot: None,
            listener: 0,
            callbacks: 0,
            failures: Vec::new(),
            funding,
        };
        dispatch
            .prepare_failure_bank(&mut budget)
            .map_err(DomEventDispatchError::Admission)?;
        dispatch.retain_running_funding(limits)?;
        // Only active original invocations remain indexed. This index is
        // bounded by pending_events and its backing was admitted with the
        // registry association. Reserve its next entry before execution.
        let mut pending = 0;
        while pending < self.pending.len() {
            budget.tick().map_err(DomEventDispatchError::Admission)?;
            let observation = &self.pending[pending];
            if observation.upgrade().is_some_and(|custody| {
                custody.event().is_some() && !observation.state.status.get().is_terminal()
            }) {
                pending += 1;
            } else {
                self.pending.swap_remove(pending);
            }
        }
        let capacity = association
            .as_ref()
            .map_or(self.pending.capacity(), |prepared| {
                prepared.pending.capacity()
            });
        if pending >= capacity {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::PendingEvents,
            ));
        }
        self.association.commit(
            association,
            &mut self.pending,
            &mut self.pending_funding,
            dom,
            cancel,
        )?;
        self.pending.push(event.continuation.clone());
        event.0.continuation.set(Some(event.continuation.clone()));
        _custody.clear_target.set(clear_target);
        if state.origin.cause().is_none() {
            event.0.trusted.set(false);
        }
        self.execute(dom, context, event, dispatch, budget, host)
    }
    /// Continue exactly where admission stopped, without entering a delivered listener again.
    fn continue_dispatch(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &EventDispatchObservation,
        call: DispatchCall<'_>,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        let DispatchCall {
            limits,
            cancel,
            host,
            funding,
        } = call;
        self.association.validate(dom)?;
        let custody = event.custody()?;
        let invocation = event.state();
        match invocation.status.get() {
            Status::Running | Status::HostPending => {
                return Err(DomEventDispatchError::AlreadyDispatching);
            }
            Status::Complete => return Err(DomEventDispatchError::AlreadyCompleted),
            Status::Abandoned => return Err(DomEventDispatchError::Abandoned),
            Status::HostRefused(error) => return Err(DomEventDispatchError::HostRefused(error)),
            Status::Ready => return Err(DomEventDispatchError::WrongContinuation),
            Status::Retained => (),
        }
        self.admit_running()?;
        if dom.owner() != invocation.owner {
            return Err(DomEventDispatchError::Dom(DomError::ForeignOwner));
        }
        let mut state = custody.borrow_mut();
        let dispatch = state
            .as_mut()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        if !Rc::ptr_eq(&dispatch._reservation.0, &self.owner) {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        let mut budget = Budget::new(limits, cancel);
        if dom.source_dispatch_blocker().is_none()
            && dom.snapshot().incomplete_hosted_operation().is_none()
        {
            match parsed_dispatch_requirement(dom, cancel, || {
                budget.tick().map_err(DomEventDispatchError::Admission)
            }) {
                Ok(None) => (),
                Ok(Some(requirement)) => {
                    return Ok(DomEventDispatchOutcome::RetainedParsedSynchronous { requirement });
                }
                Err(DomEventDispatchError::Dom(error)) => {
                    return Ok(DomEventDispatchOutcome::RetainedParsedAdmission(error));
                }
                Err(DomEventDispatchError::Admission(resource)) => {
                    return Ok(DomEventDispatchOutcome::Retained(resource));
                }
                Err(error) => return Err(error),
            }
        }
        // The parked receipt still covers the old banks. The newly admitted
        // margin stays live during replacement and the next callback sequence.
        dispatch.funding.absorb(funding)?;
        if let Err(error) = dispatch.prepare_failure_bank(&mut budget) {
            dispatch.release_unused_funding();
            return Err(DomEventDispatchError::Admission(error));
        }
        if let Some(frame) = custody.host.take() {
            match frame.result {
                Some(Err(error)) => {
                    dispatch.failures.push(DomEventListenerFailure {
                        registration: frame.registration,
                        current_target: frame.current_target,
                        kind: DomEventListenerFailureKind::Reported(error),
                    });
                }
                Some(Ok(())) => (),
                None => {
                    custody.host.set(Some(frame));
                    dispatch.release_unused_funding();
                    return Err(DomEventDispatchError::AlreadyDispatching);
                }
            }
        }
        dispatch.retain_running_funding(limits)?;
        let dispatch = state
            .take()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        drop(state);
        self.execute(dom, context, event, dispatch, budget, host)
    }
    fn admit_running(&self) -> Result<(), DomEventDispatchError> {
        if self.owner.running.get() >= self.limits.nested_dispatches {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::NestedDispatches,
            ));
        }
        Ok(())
    }
    fn execute(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &EventDispatchObservation,
        mut dispatch: Dispatch,
        mut budget: Budget<'_>,
        host: DispatchHost,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        let custody = event.custody()?;
        let state = event.state();
        self.owner.running.set(self.owner.running.get() + 1);
        let running = RunningReservation(self.owner.clone());
        state.status.set(Status::Running);
        custody.lifetime.set(InvocationLifetime::Executing);
        custody.path.set(Some(dispatch.path.clone()));
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.run(dom, context, event, &mut dispatch, &mut budget, host)
        }));
        if let Ok(Err(RunStop::HostInvocation {
            registration,
            current_target,
            key,
            #[cfg(feature = "document-scripting")]
            guest,
            passive,
            once,
        })) = result
        {
            let sequence = dispatch.callbacks;
            if let Err(error) = dispatch.settle_funding() {
                custody.abandon(state);
                custody.finish_dispatch(state);
                return Err(error.into());
            }
            if once {
                self.remove_listener(registration);
            }
            *custody.borrow_mut() = Some(dispatch);
            custody.host.set(Some(HostFrame {
                sequence,
                registration,
                current_target,
                result: None,
                running: Some(running),
                scope: None,
            }));
            state.status.set(Status::HostPending);
            custody.lifetime.set(InvocationLifetime::Parked);
            if passive {
                custody.passive.set(PassiveScope::Host);
            }
            return Ok(DomEventDispatchOutcome::HostInvocation(
                DomEventHostInvocation {
                    event: event.event.clone(),
                    custody: event.continuation.clone(),
                    sequence,
                    registration,
                    key,
                    #[cfg(feature = "document-scripting")]
                    guest,
                    #[cfg(feature = "document-scripting")]
                    once_removed: once,
                    reported: false,
                },
            ));
        }
        drop(running);
        custody.clear_invocation();
        if state.status.get() == Status::Abandoned {
            custody.finish_dispatch(state);
            return Err(DomEventDispatchError::Abandoned);
        }
        match result {
            Ok(Ok(())) => {
                let failure_bytes =
                    dispatch.failures.capacity() * std::mem::size_of::<DomEventListenerFailure>();
                let funding = match dispatch.funding.split(failure_bytes, failure_bytes) {
                    Ok(funding) => funding,
                    Err(error) => {
                        custody.abandon(state);
                        custody.finish_dispatch(state);
                        return Err(error.into());
                    }
                };
                state.status.set(Status::Complete);
                custody.finish_dispatch(state);
                let receipt = DomEventDispatchReceipt {
                    origin: state.origin.clone(),
                    event_type: dispatch.event_type.clone(),
                    callbacks: dispatch.callbacks,
                    failures: std::mem::take(&mut dispatch.failures),
                    default_prevented: event.default_prevented(),
                    funding: Some(funding),
                };
                drop(dispatch);
                Ok(DomEventDispatchOutcome::Complete(receipt))
            }
            Ok(Err(RunStop::WrongContinuation)) => {
                custody.abandon(state);
                custody.finish_dispatch(state);
                Err(DomEventDispatchError::WrongContinuation)
            }
            other => {
                if let Err(error) = dispatch.settle_funding() {
                    custody.abandon(state);
                    custody.finish_dispatch(state);
                    return Err(error.into());
                }
                *custody.borrow_mut() = Some(dispatch);
                state.status.set(Status::Retained);
                custody.lifetime.set(InvocationLifetime::Parked);
                Ok(match other {
                    Ok(Err(RunStop::Resource(resource))) => {
                        DomEventDispatchOutcome::Retained(resource)
                    }
                    Ok(Err(RunStop::ParsedSynchronous(requirement))) => {
                        DomEventDispatchOutcome::RetainedParsedSynchronous { requirement }
                    }
                    Ok(Err(RunStop::ParsedAdmission(error))) => {
                        DomEventDispatchOutcome::RetainedParsedAdmission(error)
                    }
                    Ok(Err(RunStop::SourceOperation(operation))) => {
                        DomEventDispatchOutcome::RetainedSourceOperation(operation)
                    }
                    Ok(Err(RunStop::HostedOperation(operation))) => {
                        DomEventDispatchOutcome::RetainedHostedOperation(operation)
                    }
                    Ok(Err(RunStop::HostedAuthority(error))) => {
                        DomEventDispatchOutcome::RetainedHostedAuthority(error)
                    }
                    Ok(Err(RunStop::HostUnavailable)) => {
                        DomEventDispatchOutcome::RetainedHostUnavailable
                    }
                    Ok(Err(RunStop::HostInvocation { .. } | RunStop::WrongContinuation))
                    | Ok(Ok(())) => {
                        return Err(DomEventDispatchError::WrongContinuation);
                    }
                    Ok(Err(RunStop::ForeignOwner)) => DomEventDispatchOutcome::RetainedForeignOwner,
                    Ok(Err(RunStop::ForeignRegistry)) => {
                        DomEventDispatchOutcome::RetainedForeignRegistry
                    }
                    Err(_) => DomEventDispatchOutcome::RetainedAfterUnwind,
                    Ok(Err(RunStop::Abandoned)) => return Err(DomEventDispatchError::Abandoned),
                })
            }
        }
    }
    fn run(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &EventDispatchObservation,
        dispatch: &mut Dispatch,
        budget: &mut Budget<'_>,
        host: DispatchHost,
    ) -> Result<(), RunStop> {
        let custody = event.custody().map_err(|_| RunStop::Abandoned)?;
        let state = event.state();
        let event_type = dispatch.event_type.clone();
        let path = dispatch.path.clone();
        let count = path.len();
        while dispatch.invoke < count * 2 {
            dispatch_ready(&self.owner, dispatch, dom, state, budget)?;
            budget.tick()?;
            let capture = dispatch.invoke < count;
            let index = if capture {
                count - 1 - dispatch.invoke
            } else {
                dispatch.invoke - count
            };
            let item = &path[index];
            // Nonbubbling ancestor items are skipped by dispatch before invoke;
            // they must not perform invoke's target adjustment as a side effect.
            if !capture && !item.at_target && !event.bubbles() {
                dispatch.next_invoke();
                continue;
            }
            event.0.target.replace(Some(item.target.clone()));
            // Source invoke checks stopPropagation before initializing currentTarget
            // and cloning the list, including the second invocation at target.
            if (dispatch.snapshot.is_none() && event.0.stop.get()) || event.0.immediate.get() {
                dispatch.next_invoke();
                continue;
            }
            custody.current.replace(Some(item.invocation.clone()));
            custody.phase.set(if item.at_target {
                DomEventPhase::AtTarget
            } else if capture {
                DomEventPhase::Capturing
            } else {
                DomEventPhase::Bubbling
            });
            if dispatch.snapshot.is_none() {
                let mut snapshot = Vec::new();
                budget.tick()?;
                snapshot
                    .try_reserve_exact(budget.limits.snapshot_listeners)
                    .map_err(|_| DomEventResource::Allocation)?;
                if snapshot.capacity() > budget.limits.snapshot_listeners {
                    return Err(DomEventResource::SnapshotListeners.into());
                }
                for (slot, entry) in self.slots.iter().enumerate() {
                    budget.tick()?;
                    if let Some(r) = &entry.registration
                        && r.target == item.invocation
                    {
                        if snapshot.len() >= budget.limits.snapshot_listeners {
                            return Err(DomEventResource::SnapshotListeners.into());
                        }
                        let mut position = snapshot.len();
                        for (index, id) in snapshot.iter().enumerate() {
                            budget.tick()?;
                            let registration =
                                self.registration(*id).ok_or(RunStop::WrongContinuation)?;
                            if registration.order > r.order {
                                position = index;
                                break;
                            }
                        }
                        // Admit the actual identity shifts before insertion.
                        for _ in position..snapshot.len() {
                            budget.tick()?;
                        }
                        snapshot.insert(
                            position,
                            DomEventRegistrationId {
                                registry: self.owner.id,
                                slot,
                                generation: entry.generation,
                            },
                        );
                    }
                }
                dispatch.snapshot = Some(snapshot);
            }
            while let Some(&id) = dispatch
                .snapshot
                .as_ref()
                .and_then(|s| s.get(dispatch.listener))
            {
                dispatch_ready(&self.owner, dispatch, dom, state, budget)?;
                budget.tick()?;
                let Some(r) = self.registration(id) else {
                    dispatch.listener += 1;
                    continue;
                };
                if !equal_units(r.event_type.code_units(), event_type.code_units(), || {
                    budget.tick()
                })? || r.capture != capture
                {
                    dispatch.listener += 1;
                    continue;
                }
                // Native authority admission can refuse before source's
                // irreversible once removal. A known inactive target instead
                // follows inner invoke's skip, including its once removal.
                if !listener_target_active(
                    dom,
                    dispatch.association_custody.as_ref(),
                    item,
                    budget,
                )? {
                    let once = r.once;
                    dispatch.listener += 1;
                    if once {
                        self.remove_listener(id);
                    }
                    continue;
                }
                if budget.callbacks >= budget.limits.callbacks {
                    return Err(DomEventResource::Callbacks.into());
                }
                if dispatch.failures.len() >= budget.limits.failure_records {
                    return Err(DomEventResource::FailureRecords.into());
                }
                match &r.callback.0 {
                    CallbackKind::Host(_) if host != DispatchHost::Source => {
                        return Err(RunStop::HostUnavailable);
                    }
                    #[cfg(feature = "document-scripting")]
                    CallbackKind::Guest(_) if host != DispatchHost::Guest => {
                        return Err(RunStop::HostUnavailable);
                    }
                    _ => (),
                }
                let once = r.once;
                let passive = r.passive;
                let callback = r.callback.clone();
                // Advance and remove once *before* calling user code; both survive
                // nested dispatch and unwind, and no exclusive registry borrow escapes.
                let callbacks = dispatch
                    .callbacks
                    .checked_add(1)
                    .ok_or(DomEventResource::Callbacks)?;
                dispatch.listener += 1;
                dispatch.callbacks = callbacks;
                budget.callbacks += 1;
                let callback = match &callback.0 {
                    CallbackKind::Native(callback) => {
                        if once {
                            self.remove_listener(id);
                        }
                        callback
                    }

                    CallbackKind::Host(key) => {
                        return Err(RunStop::HostInvocation {
                            registration: id,
                            current_target: item.invocation.clone(),
                            key: **key,
                            #[cfg(feature = "document-scripting")]
                            guest: None,
                            passive,
                            once,
                        });
                    }
                    #[cfg(feature = "document-scripting")]
                    CallbackKind::Guest(callback) => {
                        return Err(RunStop::HostInvocation {
                            registration: id,
                            current_target: item.invocation.clone(),
                            key: callback.code,
                            guest: Some(callback.clone()),
                            passive,
                            once,
                        });
                    }
                };
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let _passive = BorrowedPassive::begin(&custody.passive, passive);
                    let mut access = DomEventDom::borrow(dom);
                    callback(&mut access, self, context, event)
                }));
                let failure = match &result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(DomEventListenerFailureKind::Reported(*error)),
                    Err(_) => Some(DomEventListenerFailureKind::Panicked),
                };
                if let Some(kind) = failure {
                    dispatch.failures.push(DomEventListenerFailure {
                        registration: id,
                        current_target: item.invocation.clone(),
                        kind,
                    });
                }
                if event.0.immediate.get() {
                    break;
                }
            }
            dispatch.next_invoke();
        }
        dispatch_ready(&self.owner, dispatch, dom, state, budget)?;
        Ok(())
    }
}

impl<C> Drop for DomEventDispatcher<C> {
    fn drop(&mut self) {
        for entry in &self.pending {
            if let Some(custody) = entry.upgrade() {
                if !entry.state.status.get().is_terminal() {
                    custody.abandon(&entry.state);
                }
                // No host call retains a mutable continuation loan. Withdraw
                // its real Source reentry guard before releasing dispatch state.
                custody.host.take();
                if let Ok(mut slot) = custody.dispatch.try_borrow_mut() {
                    // Its receipts drop with the actual backing. A dispatch
                    // running on the stack still owns its own funding.
                    let parked = slot.take();
                    if parked.is_some() {
                        custody.finish_dispatch(&entry.state);
                    }
                    drop(parked);
                }
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Abandoned,
    Ready,
    Running,
    HostPending,
    HostRefused(DomEventHostError),
    Retained,
    Complete,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InvocationLifetime {
    Idle,
    Executing,
    Parked,
}

impl Status {
    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Complete | Self::Abandoned | Self::HostRefused(_)
        )
    }
}
/// Immutable ToggleEvent data projected from the actual HTML producer states.
/// No public constructor or arbitrary state/source injection is offered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomToggleEvent {
    old_state: &'static str,
    new_state: &'static str,
}
impl DomToggleEvent {
    /// Exact original state spelling supplied by the HTML producer.
    pub const fn old_state(&self) -> &'static str {
        self.old_state
    }
    /// Exact new state spelling supplied by the HTML producer.
    pub const fn new_state(&self) -> &'static str {
        self.new_state
    }
    /// Details notification and the admitted Dialog Show firing clause both
    /// supply null. This makes no claim about future event-producing clauses.
    pub const fn source(&self) -> Option<NodeId> {
        None
    }
}
enum EventData {
    StyleLoad,
    StyleError,
    DialogClose,
    DialogCancel {
        cancelable: bool,
    },
    DetailsToggle(DomToggleEvent),
    DialogToggle(DomToggleEvent),
    DialogBeforeToggle {
        payload: DomToggleEvent,
        cancelable: bool,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeEventCause {
    StyleLoad,
    StyleError,
    DetailsToggle,
    DialogBeforeToggle,
    DialogToggle,
    DialogCancel,
    DialogClose,
}
impl NativeEventCause {
    fn event_type(self) -> DomEventType {
        match self {
            Self::StyleLoad => DomEventType::LOAD,
            Self::StyleError => DomEventType::ERROR,
            Self::DetailsToggle | Self::DialogToggle => DomEventType::TOGGLE,
            Self::DialogBeforeToggle => DomEventType::BEFORE_TOGGLE,
            Self::DialogCancel => DomEventType::CANCEL,
            Self::DialogClose => DomEventType::CLOSE,
        }
    }
}
struct Event {
    owner: surgeist_dom::DomOwnerId,
    // No type means legacy initialization has not occurred. An initialized
    // Event may have the exact empty type.
    event_type: RefCell<Option<DomEventType>>,
    init: Cell<DomEventInit>,
    trusted: Cell<bool>,
    creation: Result<DomEventCreationObservation, DomEventTimestampUnavailable>,
    toggle: Option<DomToggleEvent>,
    storage: EventStorage,
    target: RefCell<Option<DomEventTarget>>,
    stop: Cell<bool>,
    immediate: Cell<bool>,
    canceled: Cell<bool>,
    continuation: Cell<Option<ContinuationObservation>>,
}
impl Event {
    fn event_type(&self) -> DomEventType {
        self.event_type
            .borrow()
            .clone()
            .unwrap_or(DomEventType::EMPTY)
    }
    fn invocation(&self) -> Option<ContinuationObservation> {
        let invocation = self.continuation.take();
        self.continuation.set(invocation.clone());
        invocation
    }
}
/// A shared native Event body. Clones refer to the same type, flags and target.
/// Ordinary reuse requires actual cleanup of its previous invocation. Each
/// selected producer's continuation and completion receipt remain affine.
#[derive(Clone)]
pub struct NativeDomEvent(Rc<Event>, Rc<BodyCustody>);
/// Accounting only. No event, continuation, guard or execution authority is
/// reachable through a receipt. Each actual allocation has one shared receipt.
struct BodyCustody {
    _storage: BodyStorage,
}
enum BodyStorage {
    Owned {
        _retained: EventAllocation,
    },
    Element {
        _retained: crate::element_tasks::TaskPayload,
    },
    Source {
        _retained: surgeist_dom::DomSourceEventRetention,
        _extent: BodyExtent,
    },
    Style {
        _extent: BodyExtent,
    },
    #[cfg(test)]
    Fixture,
}
struct BodyExtent {
    live: Rc<Cell<usize>>,
    bytes: usize,
}
impl BodyExtent {
    fn new(live: &Rc<Cell<usize>>, bytes: usize) -> Self {
        // The producer checked and admitted this exact extra extent before
        // construction. Keeping the counter grants no owner access.
        live.set(live.get() + bytes);
        Self {
            live: live.clone(),
            bytes,
        }
    }
}
impl Drop for BodyExtent {
    fn drop(&mut self) {
        self.live.set(self.live.get() - self.bytes);
    }
}

/// Accounting-only funding from the original producer. This owns no event,
/// dispatcher, callback, Source execution permission or task row.
pub(crate) struct DispatchFunding(Funding);

enum Funding {
    Owned(EventAllocation),
    Element(crate::element_tasks::TaskPayload),
    Source {
        extent: BodyExtent,
        retained: surgeist_dom::DomSourceEventRetention,
    },
    Style(BodyExtent),
    #[cfg(test)]
    Fixture,
}

enum PathFunding {
    Owned {
        _retained: EventAllocation,
    },
    Element {
        _retained: crate::element_tasks::TaskPayload,
    },
    Source {
        _extent: BodyExtent,
        _retained: surgeist_dom::DomSourceEventPayloadRetention,
    },
    Style {
        _extent: BodyExtent,
    },
    #[cfg(test)]
    Fixture,
}

impl DispatchFunding {
    pub(crate) fn element(retained: crate::element_tasks::TaskPayload) -> Self {
        Self(Funding::Element(retained))
    }

    pub(crate) fn source(
        live: &Rc<Cell<usize>>,
        retained: surgeist_dom::DomSourceEventRetention,
        bytes: usize,
    ) -> Self {
        Self(Funding::Source {
            extent: BodyExtent::new(live, bytes),
            retained,
        })
    }

    pub(crate) fn style(live: &Rc<Cell<usize>>, bytes: usize) -> Self {
        Self(Funding::Style(BodyExtent::new(live, bytes)))
    }

    fn split_path(&mut self, bytes: usize) -> Result<PathFunding, DomError> {
        Ok(match &mut self.0 {
            Funding::Owned(retained) => PathFunding::Owned {
                _retained: retained.split(bytes, bytes, 0)?,
            },
            Funding::Element(retained) => PathFunding::Element {
                _retained: retained.split(bytes, bytes).map_err(task_funding_error)?,
            },
            Funding::Source { extent, retained } => {
                let remainder = extent
                    .bytes
                    .checked_sub(bytes)
                    .ok_or(DomError::RepresentationExhausted)?;
                let path = retained.split_payload(bytes, 0)?;
                PathFunding::Source {
                    _extent: extent.transfer(bytes, remainder),
                    _retained: path,
                }
            }
            Funding::Style(extent) => {
                let remainder = extent
                    .bytes
                    .checked_sub(bytes)
                    .ok_or(DomError::RepresentationExhausted)?;
                PathFunding::Style {
                    _extent: extent.transfer(bytes, remainder),
                }
            }
            #[cfg(test)]
            Funding::Fixture => PathFunding::Fixture,
        })
    }

    fn split(&mut self, full: usize, source: usize) -> Result<Self, DomError> {
        match &mut self.0 {
            Funding::Owned(retained) => retained
                .split(full, source, 0)
                .map(|retained| Self(Funding::Owned(retained))),
            Funding::Element(retained) => retained
                .split(full, source)
                .map(Self::element)
                .map_err(task_funding_error),
            Funding::Source { extent, retained } => {
                let remainder = extent
                    .bytes
                    .checked_sub(full)
                    .ok_or(DomError::RepresentationExhausted)?;
                let retained = retained.split(source, 0)?;
                Ok(Self(Funding::Source {
                    extent: extent.transfer(full, remainder),
                    retained,
                }))
            }
            Funding::Style(extent) => {
                let remainder = extent
                    .bytes
                    .checked_sub(full)
                    .ok_or(DomError::RepresentationExhausted)?;
                Ok(Self(Funding::Style(extent.transfer(full, remainder))))
            }
            #[cfg(test)]
            Funding::Fixture => Ok(Self(Funding::Fixture)),
        }
    }

    fn absorb(&mut self, other: Self) -> Result<(), DomError> {
        match (&mut self.0, other.0) {
            (Funding::Owned(retained), Funding::Owned(other)) => retained.absorb(other),
            (Funding::Element(retained), Funding::Element(other)) => {
                retained.absorb(other).map_err(task_funding_error)
            }
            (
                Funding::Source { extent, retained },
                Funding::Source {
                    extent: mut incoming,
                    retained: other,
                },
            ) => {
                let total = extent.join_bytes(&incoming)?;
                retained.absorb(other)?;
                extent.bytes = total;
                incoming.bytes = 0;
                Ok(())
            }
            (Funding::Style(extent), Funding::Style(mut incoming)) => {
                extent.bytes = extent.join_bytes(&incoming)?;
                incoming.bytes = 0;
                Ok(())
            }
            #[cfg(test)]
            (Funding::Fixture, Funding::Fixture) => Ok(()),
            _ => Err(DomError::ForeignOwner),
        }
    }

    fn shrink(&mut self, full: usize, source: usize) -> Result<(), DomError> {
        let (old_full, old_source) = match &mut self.0 {
            Funding::Owned(retained) => return retained.shrink(full, source),
            Funding::Element(retained) => (retained.full(), retained.source()),
            Funding::Source { extent, retained } => (extent.bytes, retained.retained_bytes()),
            Funding::Style(extent) => (extent.bytes, source),
            #[cfg(test)]
            Funding::Fixture => return Ok(()),
        };
        let extra = old_full
            .checked_sub(full)
            .ok_or(DomError::RepresentationExhausted)?;
        let extra_source = old_source
            .checked_sub(source)
            .ok_or(DomError::RepresentationExhausted)?;
        drop(self.split(extra, extra_source)?);
        Ok(())
    }
}

impl std::fmt::Debug for DispatchFunding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DispatchFunding").finish_non_exhaustive()
    }
}

impl BodyExtent {
    fn transfer(&mut self, bytes: usize, remainder: usize) -> Self {
        self.bytes = remainder;
        Self {
            live: self.live.clone(),
            bytes,
        }
    }

    fn join_bytes(&self, other: &Self) -> Result<usize, DomError> {
        if !Rc::ptr_eq(&self.live, &other.live) {
            return Err(DomError::ForeignOwner);
        }
        self.bytes
            .checked_add(other.bytes)
            .ok_or(DomError::RepresentationExhausted)
    }
}

fn task_funding_error(error: crate::DomElementTaskRuntimeError) -> DomError {
    match error {
        crate::DomElementTaskRuntimeError::Dom(error) => error,
        crate::DomElementTaskRuntimeError::Binding => DomError::ForeignOwner,
        _ => DomError::RepresentationExhausted,
    }
}
pub(crate) struct EventBodyCustody {
    event: Rc<BodyCustody>,
    continuation: BodyCustody,
    storage: EventStorage,
}
impl EventBodyCustody {
    fn event_bytes() -> usize {
        std::mem::size_of::<Event>() + std::mem::size_of::<BodyCustody>()
    }
    fn continuation_bytes() -> usize {
        std::mem::size_of::<EventContinuation>() + std::mem::size_of::<ContinuationState>()
    }
    pub(crate) fn element(
        retained: &mut crate::element_tasks::TaskPayload,
        storage: EventStorage,
    ) -> Result<Self, crate::DomElementTaskRuntimeError> {
        let event = retained.split(Self::event_bytes(), Self::event_bytes())?;
        let continuation =
            retained.split(Self::continuation_bytes(), Self::continuation_bytes())?;
        Ok(Self {
            storage,
            event: Rc::new(BodyCustody {
                _storage: BodyStorage::Element { _retained: event },
            }),
            continuation: BodyCustody {
                _storage: BodyStorage::Element {
                    _retained: continuation,
                },
            },
        })
    }
    pub(crate) fn source(
        retained: &mut surgeist_dom::DomSourceEventRetention,
        live: &Rc<Cell<usize>>,
        storage: EventStorage,
    ) -> Result<Self, DomError> {
        let event = retained.split(Self::event_bytes(), 0)?;
        let continuation = retained.split(Self::continuation_bytes(), 0)?;
        Ok(Self {
            storage,
            event: Rc::new(BodyCustody {
                _storage: BodyStorage::Source {
                    _retained: event,
                    _extent: BodyExtent::new(live, Self::event_bytes()),
                },
            }),
            continuation: BodyCustody {
                _storage: BodyStorage::Source {
                    _retained: continuation,
                    _extent: BodyExtent::new(live, Self::continuation_bytes()),
                },
            },
        })
    }
    pub(crate) fn style(live: &Rc<Cell<usize>>, storage: EventStorage) -> Self {
        Self {
            storage,
            event: Rc::new(BodyCustody {
                _storage: BodyStorage::Style {
                    _extent: BodyExtent::new(live, Self::event_bytes()),
                },
            }),
            continuation: BodyCustody {
                _storage: BodyStorage::Style {
                    _extent: BodyExtent::new(live, Self::continuation_bytes()),
                },
            },
        }
    }
}
#[cfg(test)]
fn test_event_backing() -> EventBodyCustody {
    // Kernel fixtures supply host-formed inputs. Real producer/accounting
    // regressions obtain their receipts through Source, tasks or Style.
    EventBodyCustody {
        storage: EventStorage::registry(DomEventStorageLimits {
            storage_bytes: 16_000_000,
            work: 1_000_000,
        })
        .unwrap(),
        event: Rc::new(BodyCustody {
            _storage: BodyStorage::Fixture,
        }),
        continuation: BodyCustody {
            _storage: BodyStorage::Fixture,
        },
    }
}
/// The state of one invocation survives weak expiry for truthful diagnostics.
/// Its storage receipt retains no continuation, callback or execution scope.
struct ContinuationState {
    owner: surgeist_dom::DomOwnerId,
    origin: DispatchOrigin,
    status: Cell<Status>,
    _body: BodyCustody,
}

#[derive(Clone, Debug)]
enum DispatchOrigin {
    Node(NodeId),
    Global(surgeist_dom::DomHostedGlobalId),
    Standalone(DomStandaloneEventTarget),
    Selected {
        node: NodeId,
        cause: NativeEventCause,
    },
}

impl From<DomEventTarget> for DispatchOrigin {
    fn from(target: DomEventTarget) -> Self {
        match target {
            DomEventTarget::Node(node) => Self::Node(node),
            DomEventTarget::Global(global) => Self::Global(global),
            DomEventTarget::Standalone(target) => Self::Standalone(target),
        }
    }
}

impl DispatchOrigin {
    fn target(&self) -> DomEventTarget {
        match self {
            Self::Node(node) | Self::Selected { node, .. } => (*node).into(),
            Self::Global(global) => (*global).into(),
            Self::Standalone(target) => target.clone().into(),
        }
    }

    fn node(&self) -> Option<NodeId> {
        match self {
            Self::Node(node) | Self::Selected { node, .. } => Some(*node),
            Self::Global(_) | Self::Standalone(_) => None,
        }
    }

    fn cause(&self) -> Option<NativeEventCause> {
        match self {
            Self::Selected { cause, .. } => Some(*cause),
            Self::Node(_) | Self::Global(_) | Self::Standalone(_) => None,
        }
    }
}
impl ContinuationState {
    fn terminal_dispatch_error(&self) -> Option<DomEventDispatchError> {
        match self.status.get() {
            Status::Abandoned => Some(DomEventDispatchError::Abandoned),
            Status::HostRefused(error) => Some(DomEventDispatchError::HostRefused(error)),
            _ => None,
        }
    }
}

/// Body charge follows the weak allocation observation, independently of
/// whether the actual continuation still provides execution authority.
#[derive(Clone)]
struct ContinuationObservation {
    value: std::rc::Weak<EventContinuation>,
    state: Rc<ContinuationState>,
}
impl ContinuationObservation {
    fn upgrade(&self) -> Option<Rc<EventContinuation>> {
        self.value.upgrade()
    }
}
struct EventObservation {
    value: std::rc::Weak<Event>,
    body: Rc<BodyCustody>,
}
/// Observation of one exact native invocation, independent of the Event body's
/// latest invocation. The producer remains its sole persistent strong owner.
pub(crate) struct EventDispatchObservation {
    event: NativeDomEvent,
    continuation: ContinuationObservation,
}
impl EventDispatchObservation {
    fn state(&self) -> &ContinuationState {
        &self.continuation.state
    }

    fn custody(&self) -> Result<Rc<EventContinuation>, DomEventDispatchError> {
        self.continuation
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)
    }
}
impl std::ops::Deref for EventDispatchObservation {
    type Target = NativeDomEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
/// The actual source task owns active dispatch custody. Observation clones
/// retain fixed backing charges through weak continuation observations; the
/// active frame, guards and reservations end with this owner.
pub(crate) struct EventInvocationOwner {
    event: NativeDomEvent,
    // Sole persistent strong custody; active private jobs upgrade temporarily.
    _custody: Rc<EventContinuation>,
    // Keep the debit until after the actual continuation allocation is released.
    state: Rc<ContinuationState>,
}
/// Additional funding needed while the original parked backing stays live.
#[derive(Clone, Copy)]
pub(crate) struct DispatchFormation {
    pub(crate) source: usize,
    pub(crate) full: usize,
}
pub(crate) struct DispatchCall<'a> {
    pub(crate) limits: DomEventDispatchLimits,
    pub(crate) cancel: &'a AtomicBool,
    pub(crate) host: DispatchHost,
    pub(crate) funding: DispatchFunding,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DispatchHost {
    Unavailable,
    Source,
    #[cfg(feature = "document-scripting")]
    Guest,
}

fn snapshot_formation_bytes(limits: DomEventDispatchLimits) -> Option<usize> {
    std::mem::size_of::<Vec<DomEventRegistrationId>>().checked_add(
        limits
            .snapshot_listeners
            .checked_mul(std::mem::size_of::<DomEventRegistrationId>())?,
    )
}

impl EventInvocationOwner {
    fn new(
        original: NodeId,
        data: EventData,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> Self {
        let (cause, toggle, cancelable) = match data {
            EventData::StyleLoad => (NativeEventCause::StyleLoad, None, false),
            EventData::StyleError => (NativeEventCause::StyleError, None, false),
            EventData::DialogClose => (NativeEventCause::DialogClose, None, false),
            EventData::DialogCancel { cancelable } => {
                (NativeEventCause::DialogCancel, None, cancelable)
            }
            EventData::DetailsToggle(payload) => {
                (NativeEventCause::DetailsToggle, Some(payload), false)
            }
            EventData::DialogToggle(payload) => {
                (NativeEventCause::DialogToggle, Some(payload), false)
            }
            EventData::DialogBeforeToggle {
                payload,
                cancelable,
            } => (
                NativeEventCause::DialogBeforeToggle,
                Some(payload),
                cancelable,
            ),
        };
        let event = NativeDomEvent(
            Rc::new(Event {
                owner: original.owner(),
                event_type: RefCell::new(Some(cause.event_type())),
                init: Cell::new(DomEventInit {
                    bubbles: false,
                    cancelable,
                    composed: false,
                }),
                trusted: Cell::new(true),
                creation: creation.map(EventCreation::into_observation),
                toggle,
                storage: backing.storage,
                target: RefCell::new(None),
                stop: Cell::new(false),
                immediate: Cell::new(false),
                canceled: Cell::new(false),
                continuation: Cell::new(None),
            }),
            backing.event,
        );
        let owner = Self::from_event(
            event,
            DispatchOrigin::Selected {
                node: original,
                cause,
            },
            backing.continuation,
        );
        owner.attach();
        owner
    }

    fn from_event(event: NativeDomEvent, origin: DispatchOrigin, backing: BodyCustody) -> Self {
        let custody = Rc::new(EventContinuation {
            event: Cell::new(None),
            lifetime: Cell::new(InvocationLifetime::Idle),
            clear_target: Cell::new(false),
            current: RefCell::new(None),
            phase: Cell::new(DomEventPhase::None),
            passive: Cell::new(PassiveScope::None),
            path: Cell::new(None),
            dispatch: RefCell::new(None),
            host: Cell::new(None),
        });
        let state = Rc::new(ContinuationState {
            owner: event.0.owner,
            origin,
            status: Cell::new(Status::Ready),
            _body: backing,
        });
        custody.event.set(Some(EventObservation {
            value: Rc::downgrade(&event.0),
            body: event.1.clone(),
        }));
        Self {
            event,
            _custody: custody,
            state,
        }
    }

    fn attach(&self) {
        self.event.0.continuation.set(Some(ContinuationObservation {
            value: Rc::downgrade(&self._custody),
            state: self.state.clone(),
        }));
    }

    pub(crate) fn dispatch_observation(&self) -> EventDispatchObservation {
        EventDispatchObservation {
            event: self.event.clone(),
            continuation: ContinuationObservation {
                value: Rc::downgrade(&self._custody),
                state: self.state.clone(),
            },
        }
    }

    pub(crate) fn has_started(&self) -> bool {
        self.state.status.get() != Status::Ready
    }

    pub(crate) fn has_active_dispatch(&self) -> bool {
        matches!(
            self.state.status.get(),
            Status::Running | Status::HostPending
        )
    }

    pub(crate) fn terminal_dispatch_error(&self) -> Option<DomEventDispatchError> {
        self.state.terminal_dispatch_error()
    }

    pub(crate) fn initial_payload_bytes() -> Option<usize> {
        EventBodyCustody::event_bytes().checked_add(EventBodyCustody::continuation_bytes())
    }
    pub(crate) fn fresh_dispatch_formation(
        limits: DomEventDispatchLimits,
    ) -> Option<DispatchFormation> {
        Self::formation(limits, None, true)
    }
    pub(crate) fn dispatch_formation(
        &self,
        limits: DomEventDispatchLimits,
    ) -> Option<DispatchFormation> {
        Self::formation(
            limits,
            self._custody.borrow().as_ref(),
            self.state.origin.node().is_some(),
        )
    }
    fn formation(
        limits: DomEventDispatchLimits,
        retained: Option<&Dispatch>,
        node: bool,
    ) -> Option<DispatchFormation> {
        let new_snapshot = snapshot_formation_bytes(limits)?;
        let new_failures = limits
            .failure_records
            .checked_mul(std::mem::size_of::<DomEventListenerFailure>())?;
        let (source, scope) = if let Some(dispatch) = retained {
            // The frozen path, scope and both old banks are already funded.
            // Count the full replacement while the old failure bank survives,
            // then only the additional peak across later listener snapshots.
            let old_snapshot = dispatch.snapshot.as_ref().map_or(Some(0), |snapshot| {
                snapshot
                    .capacity()
                    .checked_mul(std::mem::size_of::<DomEventRegistrationId>())
            })?;
            let old_failures = dispatch
                .failures
                .capacity()
                .checked_mul(std::mem::size_of::<DomEventListenerFailure>())?;
            let replacement = if new_failures > old_failures {
                new_failures.checked_add(std::mem::size_of::<Vec<DomEventListenerFailure>>())?
            } else {
                0
            };
            let run = new_snapshot
                .saturating_sub(old_snapshot)
                .checked_add(new_failures.saturating_sub(old_failures))?;
            (replacement.max(run), 0)
        } else {
            let items = if node {
                limits.path_items
            } else {
                limits.path_items.min(1)
            };
            let backing = items.checked_mul(std::mem::size_of::<PathItem>())?;
            // Path and exact-node gather are independent formation owners; the
            // gather drops before the path moves into its shared EventPath.
            let mut path_formation = std::mem::size_of::<Vec<PathItem>>().checked_add(backing)?;
            let scope = if node {
                path_formation = path_formation
                    .checked_add(std::mem::size_of::<Vec<NodeId>>())?
                    .checked_add(items.checked_mul(std::mem::size_of::<NodeId>())?)?;
                DomDispatchCustody::maximum_retained_bytes(items).ok()?
            } else {
                0
            };
            let base = std::mem::size_of::<EventPath>().checked_add(backing)?;
            let failure_formation = base
                .checked_add(std::mem::size_of::<Vec<DomEventListenerFailure>>())?
                .checked_add(new_failures)?;
            let run = base.checked_add(new_snapshot)?.checked_add(new_failures)?;
            (path_formation.max(failure_formation).max(run), scope)
        };
        Some(DispatchFormation {
            source,
            full: source.checked_add(scope)?,
        })
    }
    #[cfg(test)]
    pub(crate) fn retained_payload_bytes(&self) -> Option<usize> {
        let own = self.retained_source_payload_bytes()?;
        own.checked_add(
            self._custody
                .borrow()
                .as_ref()
                .map_or(0, Dispatch::association_bytes),
        )
    }
    /// Actual caller payload only; canonical Source custody body is separate.
    #[cfg(test)]
    fn retained_source_payload_bytes(&self) -> Option<usize> {
        // Private observation only; no user code runs under this borrow.
        let mut bytes = Self::initial_payload_bytes()?;
        if let Some(dispatch) = self._custody.borrow().as_ref() {
            bytes = bytes
                .checked_add(std::mem::size_of::<EventPath>())?
                .checked_add(
                    dispatch
                        .path
                        .capacity()
                        .checked_mul(std::mem::size_of::<PathItem>())?,
                )?;
            bytes =
                bytes.checked_add(dispatch.snapshot.as_ref().map_or(Some(0), |snapshot| {
                    snapshot
                        .capacity()
                        .checked_mul(std::mem::size_of::<DomEventRegistrationId>())
                })?)?;
            bytes = bytes.checked_add(
                dispatch
                    .failures
                    .capacity()
                    .checked_mul(std::mem::size_of::<DomEventListenerFailure>())?,
            )?;
        }
        Some(bytes)
    }
}
#[cfg(test)]
pub(crate) fn manipulation_observation_layout(
    event: &NativeDomEvent,
    path: &DomEventComposedPath,
) -> (usize, usize) {
    // Read the actual independently retained allocation owners, not any
    // task/source admission counter or producer maximum reservation.
    let event = std::mem::size_of_val(event.0.as_ref()) + std::mem::size_of_val(event.1.as_ref());
    let path = path.path.as_ref().map_or(0, |path| {
        std::mem::size_of_val(path.as_ref())
            + path.items.capacity() * std::mem::size_of::<PathItem>()
    });
    (event, path)
}
impl std::ops::Deref for EventInvocationOwner {
    type Target = NativeDomEvent;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
impl Drop for EventInvocationOwner {
    fn drop(&mut self) {
        if self.state.status.get() != Status::Complete {
            self._custody.abandon(&self.state);
            self._custody.host.take();
            if self._custody.lifetime.get() != InvocationLifetime::Executing {
                self._custody.finish_dispatch(&self.state);
            }
        }
        // Ordinary field drop releases the actual continuation directly; it
        // never borrows a RefCell, calls user code, or manufactures completion.
    }
}
impl std::fmt::Debug for NativeDomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("NativeDomEvent");
        debug.field("event_type", &self.event_type());
        if let Some(invocation) = self.0.invocation() {
            debug
                .field("original_target", &invocation.state.origin.target())
                .field("status", &invocation.state.status.get());
        }
        debug.finish()
    }
}
impl NativeDomEvent {
    fn continuation(&self) -> Option<Rc<EventContinuation>> {
        self.0.invocation()?.upgrade()
    }
    /// Construct the actual reached Style networking notification.
    pub(crate) fn style_notification(
        target: NodeId,
        success: bool,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        let data = if success {
            EventData::StyleLoad
        } else {
            EventData::StyleError
        };
        EventInvocationOwner::new(target, data, creation, backing)
    }
    /// The actual Details task owner supplies its exact queued/coalesced data.
    /// The selected HTML firing clause supplies no source Element. Equal states
    /// are preserved; this constructor performs no coalescing or live-state read.
    pub(crate) fn details_toggle(
        target: NodeId,
        payload: surgeist_html::elements::details::HtmlDetailsToggleEvent,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        EventInvocationOwner::new(
            target,
            EventData::DetailsToggle(DomToggleEvent {
                old_state: payload.old_state().as_str(),
                new_state: payload.new_state().as_str(),
            }),
            creation,
            backing,
        )
    }

    pub(crate) fn dialog_cancel(
        target: NodeId,
        cancelable: bool,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        EventInvocationOwner::new(
            target,
            EventData::DialogCancel { cancelable },
            creation,
            backing,
        )
    }

    pub(crate) fn dialog_close(
        target: NodeId,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        EventInvocationOwner::new(target, EventData::DialogClose, creation, backing)
    }

    /// Actual null-source Dialog toggle task payload, including equal states.
    pub(crate) fn dialog_toggle(
        target: NodeId,
        old: surgeist_html::elements::dialog::HtmlDialogState,
        new: surgeist_html::elements::dialog::HtmlDialogState,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        EventInvocationOwner::new(
            target,
            EventData::DialogToggle(DomToggleEvent {
                old_state: old.as_str(),
                new_state: new.as_str(),
            }),
            creation,
            backing,
        )
    }

    /// The actual Dialog cursor supplies the HTML-owned states and the
    /// cancelability selected by its reached firing clause. Show supplies true.
    pub(crate) fn dialog_before_toggle(
        target: NodeId,
        old: surgeist_html::elements::dialog::HtmlDialogState,
        new: surgeist_html::elements::dialog::HtmlDialogState,
        cancelable: bool,
        creation: Result<EventCreation, DomEventTimestampUnavailable>,
        backing: EventBodyCustody,
    ) -> EventInvocationOwner {
        EventInvocationOwner::new(
            target,
            EventData::DialogBeforeToggle {
                payload: DomToggleEvent {
                    old_state: old.as_str(),
                    new_state: new.as_str(),
                },
                cancelable,
            },
            creation,
            backing,
        )
    }

    /// Exact source event type.
    pub fn event_type(&self) -> DomEventType {
        self.0.event_type()
    }
    /// Immutable state strings from the actual Details task or Dialog firing
    /// clause. Attribute edits during delivery cannot change this payload.
    pub fn toggle_event(&self) -> Option<&DomToggleEvent> {
        self.0.toggle.as_ref()
    }
    /// Current shadow-adjusted target; cleanup clears shadow-internal targets.
    pub fn target(&self) -> Option<DomEventTarget> {
        self.0.target.borrow().clone()
    }
    /// Current invocation target, retained across an owned host call and cleared
    /// when that callback reports completion or ordinary dispatch returns.
    pub fn current_target(&self) -> Option<DomEventTarget> {
        self.continuation()
            .and_then(|invocation| invocation.current.borrow().clone())
    }
    /// Current source phase.
    pub fn phase(&self) -> DomEventPhase {
        self.continuation()
            .map_or(DomEventPhase::None, |invocation| invocation.phase.get())
    }
    /// Whether the Event bubbles.
    pub fn bubbles(&self) -> bool {
        self.0.init.get().bubbles
    }
    /// Whether the Event is cancelable.
    pub fn cancelable(&self) -> bool {
        self.0.init.get().cancelable
    }
    /// Whether the Event is composed across shadow boundaries.
    pub fn composed(&self) -> bool {
        self.0.init.get().composed
    }
    /// Sticky canceled flag retained with the same actual event identity.
    pub fn default_prevented(&self) -> bool {
        self.0.canceled.get()
    }
    /// Set the canceled flag when the Event is cancelable and no passive
    /// listener is currently invoking it.
    pub fn prevent_default(&self) {
        if self.cancelable()
            && !self
                .continuation()
                .is_some_and(|invocation| invocation.passive.get() != PassiveScope::None)
        {
            self.0.canceled.set(true);
        }
    }
    /// Stop subsequent source invoke steps. Other listeners in the current
    /// snapshot may still run; the later target invocation checks this flag too.
    pub fn stop_propagation(&self) {
        self.0.stop.set(true);
    }
    /// Also stop remaining listeners in the current invocation snapshot.
    pub fn stop_immediate_propagation(&self) {
        self.0.immediate.set(true);
        self.0.stop.set(true);
    }
    /// Allocation-free closed-tree-filtered path in target-to-root order. The
    /// iterator retains only the already-admitted frozen path, never a DOM borrow.
    /// Outside a borrowed or pending host callback it is empty.
    pub fn composed_path(&self) -> DomEventComposedPath {
        let invocation = self.continuation();
        let path = invocation.as_ref().and_then(|invocation| {
            let path = invocation.path.take();
            invocation.path.set(path.clone());
            path
        });
        let current = invocation.and_then(|invocation| invocation.current.borrow().clone());
        let current_index = path.as_ref().and_then(|path| {
            path.iter()
                .position(|p| Some(&p.invocation) == current.as_ref())
        });
        DomEventComposedPath {
            path,
            current_index,
            next: 0,
        }
    }
}
/// An allocation-free iterator implementing DOM3071 composedPath's closed-tree
/// levels. Retaining it retains the bounded path, independent of canonical edits.
pub struct DomEventComposedPath {
    path: Option<Rc<EventPath>>,
    current_index: Option<usize>,
    next: usize,
}
impl Iterator for DomEventComposedPath {
    type Item = DomEventTarget;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self
            .current_index
            .zip(self.path.as_ref())
            .map_or(0, |(_, path)| path.len().saturating_sub(self.next));
        (0, Some(remaining))
    }

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.path.as_ref()?;
        let current = self.current_index?;
        while self.next < path.len() {
            let index = self.next;
            self.next += 1;
            if visible(path, current, index) {
                return Some(path[index].invocation.clone());
            }
        }
        None
    }
}
fn visible(path: &[PathItem], current: usize, desired: usize) -> bool {
    if desired == current {
        return true;
    }
    let mut level = 0_i128;
    for i in (current..path.len()).rev() {
        if path[i].closed_root {
            level += 1;
        }
        if i == current {
            break;
        }
        if path[i].closed_slot {
            level -= 1;
        }
    }
    let mut max_level = level;
    if desired < current {
        for i in (desired..current).rev() {
            if path[i].closed_root {
                level += 1;
            }
            if i == desired {
                return level <= max_level;
            }
            if path[i].closed_slot {
                level -= 1;
                max_level = max_level.min(level);
            }
        }
    } else {
        for (i, item) in path.iter().enumerate().take(desired + 1).skip(current + 1) {
            if item.closed_slot {
                level += 1;
            }
            if i == desired {
                return level <= max_level;
            }
            if item.closed_root {
                level -= 1;
                max_level = max_level.min(level);
            }
        }
    }
    false
}
struct PathItem {
    invocation: DomEventTarget,
    target: DomEventTarget,
    at_target: bool,
    closed_root: bool,
    closed_slot: bool,
    // Exact row in this Dispatch's original affine canonical association scope.
    // Non-Node targets use their own actual relevant Global authority.
    custody_row: Option<usize>,
}
struct EventPath {
    items: Vec<PathItem>,
    // Shared independently by composed_path iterators after Event/Task release.
    _funding: PathFunding,
}
impl std::ops::Deref for EventPath {
    type Target = Vec<PathItem>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
struct EventContinuation {
    event: Cell<Option<EventObservation>>,
    lifetime: Cell<InvocationLifetime>,
    clear_target: Cell<bool>,
    current: RefCell<Option<DomEventTarget>>,
    phase: Cell<DomEventPhase>,
    passive: Cell<PassiveScope>,
    path: Cell<Option<Rc<EventPath>>>,
    dispatch: RefCell<Option<Dispatch>>,
    host: Cell<Option<HostFrame>>,
}
impl std::ops::Deref for EventContinuation {
    type Target = RefCell<Option<Dispatch>>;
    fn deref(&self) -> &Self::Target {
        &self.dispatch
    }
}
impl EventContinuation {
    fn finish_dispatch(&self, state: &ContinuationState) {
        self.clear_invocation();
        if let Some(event) = self.event()
            && event
                .0
                .invocation()
                .is_some_and(|latest| std::ptr::eq(latest.state.as_ref(), state))
        {
            if self.clear_target.get() {
                event.0.target.replace(None);
            }
            event.0.stop.set(false);
            event.0.immediate.set(false);
        }
        self.lifetime.set(InvocationLifetime::Idle);
    }
    fn clear_invocation(&self) {
        self.current.replace(None);
        self.phase.set(DomEventPhase::None);
        self.path.take();
        // A borrowed callback still on the stack owns its passive guard.
        if self.passive.get() == PassiveScope::Host {
            self.passive.set(PassiveScope::None);
        }
    }

    fn abandon(&self, state: &ContinuationState) {
        self.clear_invocation();
        state.status.set(Status::Abandoned);
    }

    fn event(&self) -> Option<NativeDomEvent> {
        let observation = self.event.take();
        let event = observation.as_ref().and_then(|entry| {
            entry
                .value
                .upgrade()
                .map(|event| NativeDomEvent(event, entry.body.clone()))
        });
        self.event.set(observation);
        event
    }
}
struct Dispatch {
    association_custody: Option<DomDispatchCustody>,
    _reservation: PendingReservation,
    event_type: DomEventType,
    path: Rc<EventPath>,
    invoke: usize,
    snapshot: Option<Vec<DomEventRegistrationId>>,
    listener: usize,
    callbacks: usize,
    failures: Vec<DomEventListenerFailure>,
    // Declared after the banks and scope so their backing drops first.
    funding: DispatchFunding,
}
impl Dispatch {
    fn association_bytes(&self) -> usize {
        self.association_custody
            .as_ref()
            .map_or(0, DomDispatchCustody::retained_bytes)
    }
    fn snapshot_backing_bytes(&self) -> usize {
        self.snapshot.as_ref().map_or(0, |snapshot| {
            snapshot.capacity() * std::mem::size_of::<DomEventRegistrationId>()
        })
    }

    fn failure_backing_bytes(&self) -> usize {
        self.failures.capacity() * std::mem::size_of::<DomEventListenerFailure>()
    }

    fn retain_running_funding(&mut self, limits: DomEventDispatchLimits) -> Result<(), DomError> {
        // Path gathering and old/new failure replacement have ended. Keep the
        // live banks and the peak for the next snapshot across callback reentry.
        let snapshot = snapshot_formation_bytes(limits).ok_or(DomError::RepresentationExhausted)?;
        let source = snapshot
            .max(self.snapshot_backing_bytes())
            .checked_add(self.failure_backing_bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        self.retain_funding(source)
    }

    fn settle_funding(&mut self) -> Result<(), DomError> {
        let source = self
            .snapshot_backing_bytes()
            .checked_add(self.failure_backing_bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        self.retain_funding(source)
    }

    fn retain_funding(&mut self, source: usize) -> Result<(), DomError> {
        let full = source
            .checked_add(self.association_bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        self.funding.shrink(full, source)
    }

    fn release_unused_funding(&mut self) {
        // Preserve the first refusal. A failed shrink leaves the original
        // funding intact, so no surviving allocation becomes uncharged.
        let _ = self.settle_funding();
    }

    fn prepare_failure_bank(&mut self, budget: &mut Budget<'_>) -> Result<(), DomEventResource> {
        let requested = budget.limits.failure_records;
        if requested <= self.failures.capacity() {
            return Ok(());
        }
        budget.tick()?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(requested)
            .map_err(|_| DomEventResource::Allocation)?;
        if replacement.capacity() > requested {
            return Err(DomEventResource::FailureRecords);
        }
        for failure in &self.failures {
            budget.tick()?;
            replacement.push(failure.clone());
        }
        // All fallible formation/copy work precedes the original-bank transfer.
        // The displaced allocation physically drops before callbacks resume.
        let old = std::mem::replace(&mut self.failures, replacement);
        drop(old);
        Ok(())
    }
    fn next_invoke(&mut self) {
        self.invoke += 1;
        self.snapshot = None;
        self.listener = 0;
    }
}
enum RunStop {
    Abandoned,
    WrongContinuation,
    HostUnavailable,
    HostInvocation {
        registration: DomEventRegistrationId,
        current_target: DomEventTarget,
        key: u64,
        #[cfg(feature = "document-scripting")]
        guest: Option<Rc<crate::document_scripting::guest::RegisteredGuestListener>>,
        passive: bool,
        once: bool,
    },
    Resource(DomEventResource),
    ParsedSynchronous(DomParsedHtmlStyleNotificationRequirement),
    ParsedAdmission(DomError),
    SourceOperation(surgeist_dom::DomSourceOperationId),
    HostedOperation(surgeist_dom::DomHostedOperationId),
    HostedAuthority(DomHostedError),
    ForeignOwner,
    ForeignRegistry,
}
impl From<DomEventResource> for RunStop {
    fn from(resource: DomEventResource) -> Self {
        Self::Resource(resource)
    }
}
impl From<DomError> for RunStop {
    fn from(error: DomError) -> Self {
        Self::ParsedAdmission(error)
    }
}

/// Inspect the actual canonical contribution headers under the caller's work
/// budget. The lower owner meters activated-entry inspection independently.
pub(crate) fn parsed_dispatch_requirement<E: From<DomError>>(
    dom: &Dom,
    cancel: &AtomicBool,
    mut tick: impl FnMut() -> Result<(), E>,
) -> Result<Option<DomParsedHtmlStyleNotificationRequirement>, E> {
    let mut cursor = dom.parsed_html_dispatch_cursor();
    loop {
        tick()?;
        let Some(entry) = cursor.advance(cancel)? else {
            return Ok(None);
        };
        if let Some(requirement) = entry.requirement(cancel)? {
            return Ok(Some(requirement));
        }
    }
}

fn listener_target_active(
    dom: &Dom,
    custody: Option<&DomDispatchCustody>,
    item: &PathItem,
    budget: &mut Budget<'_>,
) -> Result<bool, RunStop> {
    budget.tick()?;
    let snapshot = dom.snapshot();
    let (document, global) = match &item.invocation {
        DomEventTarget::Node(node) => {
            let (custody, row) = custody
                .zip(item.custody_row)
                .ok_or(RunStop::HostedAuthority(
                    DomError::InvalidDispatchCustody.into(),
                ))?;
            let association = dom
                .dispatch_custody_association(custody, row, *node)
                .map_err(|error| RunStop::HostedAuthority(error.into()))?;
            (association.document(), None)
        }
        // Historical association is used only to reach the negative activity
        // check. Positive eligibility below still qualifies current authority.
        DomEventTarget::Global(global) => (
            snapshot
                .global_document(*global)
                .map_err(RunStop::HostedAuthority)?,
            Some(*global),
        ),
        DomEventTarget::Standalone(target) => (target.document(), Some(target.relevant_global())),
    };
    budget.tick()?;
    let document = snapshot
        .document(document)
        .map_err(|error| RunStop::HostedAuthority(error.into()))?;
    if !document
        .metadata()
        .document_environment()
        .has_ever_had_navigable()
    {
        return Ok(true);
    }
    // This current capture supplies negative activity evidence only. Positive
    // eligibility additionally requires the actual issuer-qualified owner.
    if !dom
        .dispatch_document_fully_active(document.id(), budget.cancel)
        .map_err(RunStop::HostedAuthority)?
    {
        return Ok(false);
    }
    budget.tick()?;
    let global = match global {
        None => dom
            .dispatch_relevant_global(document.id().node(), budget.cancel)
            .map_err(RunStop::HostedAuthority)?
            .ok_or(RunStop::HostedAuthority(DomHostedError::StaleIdentity))?,
        Some(global) => global,
    };
    let current = dom
        .dispatch_global_document(global, budget.cancel)
        .map_err(RunStop::HostedAuthority)?;
    if current != document.id() {
        return Err(RunStop::HostedAuthority(DomHostedError::StaleIdentity));
    }
    Ok(true)
}
fn dispatch_ready(
    registry: &Rc<RegistryOwner>,
    dispatch: &Dispatch,
    dom: &Dom,
    invocation: &ContinuationState,
    budget: &mut Budget<'_>,
) -> Result<(), RunStop> {
    if invocation.status.get() == Status::Abandoned {
        return Err(RunStop::Abandoned);
    }
    if !Rc::ptr_eq(registry, &dispatch._reservation.0) {
        return Err(RunStop::ForeignRegistry);
    }
    if dom.owner() != invocation.owner {
        return Err(RunStop::ForeignOwner);
    }
    if let Some(operation) = dom.source_dispatch_blocker() {
        return Err(RunStop::SourceOperation(operation));
    }
    if let Some(operation) = dom.snapshot().incomplete_hosted_operation() {
        return Err(RunStop::HostedOperation(operation));
    }
    if let Some(requirement) =
        parsed_dispatch_requirement(dom, budget.cancel, || budget.tick().map_err(RunStop::from))?
    {
        return Err(RunStop::ParsedSynchronous(requirement));
    }
    Ok(())
}
struct Budget<'a> {
    cancel: &'a AtomicBool,
    limits: DomEventDispatchLimits,
    work: u64,
    callbacks: usize,
}
impl<'a> Budget<'a> {
    fn new(limits: DomEventDispatchLimits, cancel: &'a AtomicBool) -> Self {
        Self {
            cancel,
            limits,
            work: 0,
            callbacks: 0,
        }
    }
    fn tick(&mut self) -> Result<(), DomEventResource> {
        if self.work >= self.limits.work {
            return Err(DomEventResource::Work);
        }
        self.work += 1;
        Ok(())
    }
}
fn source_tick(budget: &mut Budget<'_>) -> Result<(), DomEventDispatchError> {
    budget.tick().map_err(DomEventDispatchError::Admission)
}
fn root(
    snapshot: &DomSnapshot,
    mut node: NodeId,
    budget: &mut Budget<'_>,
) -> Result<NodeId, DomEventDispatchError> {
    loop {
        source_tick(budget)?;
        match snapshot
            .node(node)
            .map_err(DomEventDispatchError::Dom)?
            .parent()
        {
            Some(parent) => node = parent,
            None => return Ok(node),
        }
    }
}
fn build_direct_path(
    dom: &Dom,
    target: DomEventTarget,
    budget: &mut Budget<'_>,
) -> Result<Vec<PathItem>, DomEventDispatchError> {
    if budget.limits.path_items == 0 {
        return Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems,
        ));
    }
    if let DomEventTarget::Global(global) = &target {
        dom.dispatch_global_document_with(*global, budget.cancel, &mut || source_tick(budget))?;
    }
    source_tick(budget)?;
    let mut path = Vec::new();
    path.try_reserve_exact(1)
        .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
    if path.capacity() != 1 {
        return Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems,
        ));
    }
    path.push(PathItem {
        invocation: target.clone(),
        target,
        at_target: true,
        closed_root: false,
        closed_slot: false,
        custody_row: None,
    });
    Ok(path)
}

fn register_node_path_custody(
    dom: &mut Dom,
    path: &mut [PathItem],
    budget: &mut Budget<'_>,
) -> Result<DomDispatchCustody, DomEventDispatchError> {
    // The bounded gather is live alongside the path and scope construction.
    let mut invocation_nodes = Vec::new();
    invocation_nodes
        .try_reserve_exact(path.len())
        .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
    if invocation_nodes.capacity() > budget.limits.path_items {
        return Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems,
        ));
    }
    for item in path {
        source_tick(budget)?;
        if let Some(node) = item.invocation.node() {
            item.custody_row = Some(invocation_nodes.len());
            invocation_nodes.push(node);
        }
    }
    dom.register_dispatch_custody(&invocation_nodes, budget.cancel)
        .map_err(Into::into)
}

fn build_node_path(
    dom: &Dom,
    snapshot: &DomSnapshot,
    original: NodeId,
    event_type: &DomEventType,
    composed: bool,
    native: Option<NativeEventCause>,
    budget: &mut Budget<'_>,
) -> Result<Vec<PathItem>, DomEventDispatchError> {
    snapshot
        .node(original)
        .map_err(DomEventDispatchError::Dom)?;
    let first_root = root(snapshot, original, budget)?;
    let mut path: Vec<PathItem> = Vec::new();
    source_tick(budget)?;
    path.try_reserve_exact(budget.limits.path_items)
        .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
    if path.capacity() > budget.limits.path_items {
        return Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems,
        ));
    }
    let mut node = original;
    let mut target = original;
    let mut at_target = true;
    let mut closed_slot = false;
    loop {
        source_tick(budget)?;
        if path.len() >= budget.limits.path_items || path.len() >= usize::MAX / 2 {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::PathItems,
            ));
        }
        for item in &path {
            source_tick(budget)?;
            if item.invocation == DomEventTarget::Node(node) {
                return Err(DomEventDispatchError::CyclicPath);
            }
        }
        let view = snapshot.node(node).map_err(DomEventDispatchError::Dom)?;
        source_tick(budget)?;
        let closed_root = view
            .as_shadow_root()
            .is_some_and(|r| r.state().mode() == HtmlShadowRootMode::Closed);
        path.push(PathItem {
            invocation: DomEventTarget::Node(node),
            target: target.into(),
            at_target,
            closed_root,
            closed_slot,
            custody_row: None,
        });
        let cancel = budget.cancel;
        let next = dom.dispatch_event_parent(
            snapshot,
            DomEventParentQuery::new(node, first_root, target, event_type.code_units(), composed),
            cancel,
            &mut || source_tick(budget),
        )?;
        let (parent, via_assigned_slot, in_adjusted_target_tree) = match next {
            DomEventParent::Node {
                parent,
                via_assigned_slot,
                in_adjusted_target_tree,
            } => (parent, via_assigned_slot, in_adjusted_target_tree),
            DomEventParent::Global(global) => {
                source_tick(budget)?;
                if path.len() >= budget.limits.path_items {
                    return Err(DomEventDispatchError::Admission(
                        DomEventResource::PathItems,
                    ));
                }
                path.push(PathItem {
                    invocation: DomEventTarget::Global(global),
                    target: target.into(),
                    at_target: false,
                    closed_root: false,
                    closed_slot: false,
                    custody_row: None,
                });
                // A native Global's default EventTarget parent is null.
                break;
            }
            DomEventParent::None => {
                // Selected native delivery needs current hosted authority even
                // where the standard parent stops at this Document.
                if let Some(document) = view.as_document().filter(|document| {
                    native.is_some()
                        && document
                            .metadata()
                            .document_environment()
                            .has_ever_had_navigable()
                }) {
                    dom.qualify_native_dispatch_document(document.id(), cancel, &mut || {
                        source_tick(budget)
                    })?;
                }
                break;
            }
        };
        closed_slot = via_assigned_slot
            && snapshot
                .node(root(snapshot, parent, budget)?)
                .map_err(DomEventDispatchError::Dom)?
                .as_shadow_root()
                .is_some_and(|r| r.state().mode() == HtmlShadowRootMode::Closed);
        at_target = !in_adjusted_target_tree;
        if at_target {
            target = parent;
        }
        node = parent;
    }
    Ok(path)
}

#[cfg(test)]
mod tests;
