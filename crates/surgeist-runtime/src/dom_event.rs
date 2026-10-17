//! Synchronous native DOM event dispatch over the canonical DOM owner.
//!
//! Implements the pinned DOM3071 dispatch/invoke and parent algorithms for the
//! ordinary nonbubbling, noncomposed Style load/error, Details toggle and Dialog
//! beforetoggle events. Details preserves its queued state; Dialog supplies its
//! source-selected cancelability. Both admitted ToggleEvent sources are null.
//! Registration supports capture and once; JS, AbortSignal, passive listeners,
//! arbitrary event construction and activation are not offered. A hosted
//! Document's non-load parent is its actual native Global; this is no JS Window
//! or ECMAScript realm implementation. Family owners validate their actual
//! source operation before creating an event and advance only after a receipt.
//! Event creation/execution is crate-private: the public callback view cannot
//! consume a feature task's continuation or completion receipt out of its owner.
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

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use surgeist_dom::{
    Dom, DomDispatchCustody, DomError, DomHostedError, DomHostedGlobalId, DomNodeKind, DomSnapshot,
    HtmlShadowRootMode, NodeId,
};

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

/// One actual native event endpoint. A Global identity is never a tree node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventTarget {
    /// An exact canonical node, including a Document.
    Node(NodeId),
    /// The canonical identity of a real native hosted global endpoint.
    Global(DomHostedGlobalId),
}
impl DomEventTarget {
    /// Returns the canonical node when this endpoint is a node.
    pub const fn node(self) -> Option<NodeId> {
        match self {
            Self::Node(node) => Some(node),
            Self::Global(_) => None,
        }
    }
    /// Returns the real global identity when this endpoint is a global.
    pub const fn global(self) -> Option<DomHostedGlobalId> {
        match self {
            Self::Global(global) => Some(global),
            Self::Node(_) => None,
        }
    }
}
impl From<NodeId> for DomEventTarget {
    fn from(node: NodeId) -> Self {
        Self::Node(node)
    }
}
impl From<DomHostedGlobalId> for DomEventTarget {
    fn from(global: DomHostedGlobalId) -> Self {
        Self::Global(global)
    }
}

/// Native event types currently admitted by the source-qualified producers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomEventType {
    /// The Style completion load event.
    Load,
    /// The Style completion error event.
    Error,
    /// The Details notification toggle event with its exact queued payload.
    Toggle,
    /// The source-qualified Dialog beforetoggle event.
    BeforeToggle,
}
impl DomEventType {
    /// Exact source event type spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Error => "error",
            Self::Toggle => "toggle",
            Self::BeforeToggle => "beforetoggle",
        }
    }
}
/// Source invocation phase, separate from listener capture selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventPhase {
    /// No callback is currently being invoked.
    None,
    /// Ancestor capture.
    Capturing,
    /// A path item with a non-null shadow-adjusted target.
    AtTarget,
    /// Ancestor bubbling (reserved for a future source-qualified event).
    Bubbling,
}
/// All registration options this native boundary supports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomEventListenerOptions {
    /// Select the capture invocation.
    pub capture: bool,
    /// Remove this exact registration before calling its callback.
    pub once: bool,
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// This same event is already executing or retained.
    AlreadyDispatching,
    /// This event already produced its one completion receipt.
    AlreadyCompleted,
    /// The actual source task owner terminally abandoned this event.
    Abandoned,
    /// Resume requires an event retained by this exact registry.
    WrongContinuation,
    /// Registry/generation/order identity exhausted rather than wrapping.
    IdentityExhausted,
    /// A canonical event-parent chain repeated a node.
    CyclicPath,
    /// Current Document facts do not establish the supported native no-global parent.
    UnsupportedDocumentParent,
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
            Self::AlreadyDispatching => f.write_str("native DOM event is already dispatching"),
            Self::AlreadyCompleted => f.write_str("native DOM event already completed dispatch"),
            Self::Abandoned => f.write_str("native DOM event source task was abandoned"),
            Self::WrongContinuation => {
                f.write_str("native DOM event continuation is not retained by this dispatcher")
            }
            Self::IdentityExhausted => {
                f.write_str("native DOM event registration identity is exhausted")
            }
            Self::CyclicPath => f.write_str("native DOM event parent path contains a cycle"),
            Self::UnsupportedDocumentParent => {
                f.write_str("native DOM event Document parent is unsupported")
            }
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
/// Real completed delivery. This value is deliberately not cloneable.
#[derive(Debug)]
pub struct DomEventDispatchReceipt {
    target: NodeId,
    event_type: DomEventType,
    callbacks: usize,
    failures: Vec<DomEventListenerFailure>,
    default_prevented: bool,
    manipulation: Option<crate::dom_manipulation::TaskPayload>,
}
impl DomEventDispatchReceipt {
    pub(crate) fn release_manipulation_retention(&mut self) {
        self.manipulation.take();
    }
    pub(crate) fn retained_backing_bytes(&self) -> usize {
        self.failures.capacity() * std::mem::size_of::<DomEventListenerFailure>()
    }
    /// Original exact target, independent of later retirement/reparenting.
    pub fn original_target(&self) -> NodeId {
        self.target
    }
    /// Type that was actually dispatched.
    pub fn event_type(&self) -> DomEventType {
        self.event_type
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
    /// All source invoke steps and event cleanup completed.
    Complete(DomEventDispatchReceipt),
    /// The event retains its exact next step; resume it without replay.
    Retained(DomEventResource),
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
pub struct DomEventCallback<C>(Rc<Listener<C>>);
impl<C> Clone for DomEventCallback<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
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
        Self(Rc::new(callback))
    }
}
struct Registration<C> {
    target: DomEventTarget,
    event_type: DomEventType,
    options: DomEventListenerOptions,
    callback: DomEventCallback<C>,
    order: u64,
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
/// Runtime's one native listener registry and path/invocation owner.
/// `C` is the application's actual mutable composition state, e.g. its CSSOM
/// and Style coordinator; the dispatcher does not own those family semantics.
pub struct DomEventDispatcher<C> {
    owner: Rc<RegistryOwner>,
    limits: DomEventRegistryLimits,
    slots: Vec<Slot<C>>,
    next_order: u64,
}
impl<C> DomEventDispatcher<C> {
    /// Establish a distinct finite registry. Zero bounds intentionally admit no work.
    pub fn new(limits: DomEventRegistryLimits) -> Result<Self, DomEventDispatchError> {
        let id = NEXT_REGISTRY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
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
        })
    }
    /// Add in deterministic order. The target/type/callback identity/capture key
    /// deduplicates; changing `once` on a duplicate does not change the listener.
    pub fn add_listener(
        &mut self,
        dom: &DomEventDom<'_>,
        target: impl Into<DomEventTarget>,
        event_type: DomEventType,
        callback: DomEventCallback<C>,
        options: DomEventListenerOptions,
    ) -> Result<DomEventRegistrationId, DomEventDispatchError> {
        let target = target.into();
        match target {
            DomEventTarget::Node(node) => {
                dom.snapshot()
                    .node(node)
                    .map_err(DomEventDispatchError::Dom)?;
            }
            DomEventTarget::Global(global) => {
                dom.raw()
                    .global_document(global)
                    .map_err(DomEventDispatchError::Hosted)?;
            }
        }
        for (slot, entry) in self.slots.iter().enumerate() {
            if let Some(r) = &entry.registration
                && r.target == target
                && r.event_type == event_type
                && r.options.capture == options.capture
                && Rc::ptr_eq(&r.callback.0, &callback.0)
            {
                return Ok(DomEventRegistrationId {
                    registry: self.owner.id,
                    slot,
                    generation: entry.generation,
                });
            }
        }
        if self
            .slots
            .iter()
            .filter(|s| s.registration.is_some())
            .count()
            >= self.limits.registrations
        {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::Registrations,
            ));
        }
        let order = self
            .next_order
            .checked_add(1)
            .ok_or(DomEventDispatchError::IdentityExhausted)?;
        let reusable = self
            .slots
            .iter()
            .position(|s| s.registration.is_none() && s.generation < u64::MAX);
        let slot = if let Some(slot) = reusable {
            slot
        } else {
            if self.slots.len() >= self.limits.registrations {
                return Err(DomEventDispatchError::IdentityExhausted);
            }
            self.slots
                .try_reserve_exact(1)
                .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
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
            options,
            callback,
            order,
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
    /// Begin a fresh source event. Exact live target and the entire path are
    /// validated/admitted before any callback is entered.
    pub(crate) fn dispatch(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &NativeDomEvent,
        limits: DomEventDispatchLimits,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        match event.0.status.get() {
            Status::Ready => (),
            Status::Complete => return Err(DomEventDispatchError::AlreadyCompleted),
            Status::Abandoned => return Err(DomEventDispatchError::Abandoned),
            _ => return Err(DomEventDispatchError::AlreadyDispatching),
        }
        self.admit_running()?;
        if self.owner.pending.get() >= self.limits.pending_events {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::PendingEvents,
            ));
        }
        let mut budget = Budget::new(limits);
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
        let mut path = build_path(
            dom,
            &snapshot,
            event.0.original,
            event.0.event_type(),
            &mut budget,
        )?;
        let final_root = root(
            &snapshot,
            path.last().expect("nonempty path").target,
            &mut budget,
        )?;
        let clear_target = snapshot
            .node(final_root)
            .map_err(DomEventDispatchError::Dom)?
            .as_shadow_root()
            .is_some();
        // This bounded temporary is live alongside the path and producer
        // scope construction. Family maximum admission includes that overlap.
        let mut invocation_nodes = Vec::new();
        // Allocate once from empty: incremental growth would require admitting
        // an old/new gather overlap as well as the completed path and scope.
        invocation_nodes
            .try_reserve_exact(path.len())
            .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
        if invocation_nodes.capacity() > limits.path_items {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::PathItems,
            ));
        }
        for item in &mut path {
            budget.tick().map_err(DomEventDispatchError::Admission)?;
            if let DomEventTarget::Node(node) = item.invocation {
                item.custody_row = Some(invocation_nodes.len());
                invocation_nodes.push(node);
            }
        }
        let association_custody = dom
            .register_dispatch_custody(&invocation_nodes, &AtomicBool::new(false))
            .map_err(DomEventDispatchError::Dom)?;
        drop(invocation_nodes);
        self.owner.pending.set(self.owner.pending.get() + 1);
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
        let path_retained = event
            .0
            .continuation
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)?
            .manipulation
            .borrow_mut()
            .as_mut()
            .map(|retained| {
                retained
                    .split(path_bytes, path_bytes)
                    .expect("pre-admitted actual path capacity")
            });
        let mut dispatch = Dispatch {
            association_custody,
            _reservation: PendingReservation(self.owner.clone()),
            path: Rc::new(EventPath {
                items: path,
                _manipulation: path_retained,
            }),
            invoke: 0,
            snapshot: None,
            listener: 0,
            callbacks: 0,
            failures: Vec::new(),
            clear_target,
        };
        dispatch
            .prepare_failure_bank(&mut budget)
            .map_err(DomEventDispatchError::Admission)?;
        self.execute(dom, context, event, dispatch, budget)
    }
    /// Continue exactly where admission stopped. No already-delivered listener
    /// is called again, even after detach, adoption, retirement or callback failure.
    pub(crate) fn resume(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &NativeDomEvent,
        limits: DomEventDispatchLimits,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        match event.0.status.get() {
            Status::Running => return Err(DomEventDispatchError::AlreadyDispatching),
            Status::Complete => return Err(DomEventDispatchError::AlreadyCompleted),
            Status::Abandoned => return Err(DomEventDispatchError::Abandoned),
            Status::Ready => return Err(DomEventDispatchError::WrongContinuation),
            Status::Retained => (),
        }
        self.admit_running()?;
        if dom.owner() != event.0.original.owner() {
            return Err(DomEventDispatchError::Dom(DomError::ForeignOwner));
        }
        let custody = event
            .0
            .continuation
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)?;
        let mut state = custody.borrow_mut();
        let dispatch = state
            .as_mut()
            .ok_or(DomEventDispatchError::WrongContinuation)?;
        if !Rc::ptr_eq(&dispatch._reservation.0, &self.owner) {
            return Err(DomEventDispatchError::WrongContinuation);
        }
        let mut budget = Budget::new(limits);
        dispatch
            .prepare_failure_bank(&mut budget)
            .map_err(DomEventDispatchError::Admission)?;
        let dispatch = state.take().expect("checked continuation");
        drop(state);
        self.execute(dom, context, event, dispatch, budget)
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
        event: &NativeDomEvent,
        mut dispatch: Dispatch,
        mut budget: Budget,
    ) -> Result<DomEventDispatchOutcome, DomEventDispatchError> {
        let custody = event
            .0
            .continuation
            .upgrade()
            .ok_or(DomEventDispatchError::Abandoned)?;
        self.owner.running.set(self.owner.running.get() + 1);
        let _running = RunningReservation(self.owner.clone());
        event.0.status.set(Status::Running);
        *event.0.path.borrow_mut() = Some(dispatch.path.clone());
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.run(dom, context, event, &mut dispatch, &mut budget)
        }));
        event.0.current.set(None);
        event.0.phase.set(DomEventPhase::None);
        event.0.path.borrow_mut().take();
        if event.0.status.get() == Status::Abandoned {
            return Err(DomEventDispatchError::Abandoned);
        }
        match result {
            Ok(Ok(())) => {
                if dispatch.clear_target {
                    event.0.target.set(None);
                }
                event.0.stop.set(false);
                event.0.immediate.set(false);
                event.0.status.set(Status::Complete);
                let failure_bytes =
                    dispatch.failures.capacity() * std::mem::size_of::<DomEventListenerFailure>();
                let manipulation = custody.manipulation.borrow_mut().as_mut().map(|retained| {
                    retained
                        .split(failure_bytes, failure_bytes)
                        .expect("pre-admitted actual failure backing")
                });
                let receipt = DomEventDispatchReceipt {
                    target: event.0.original,
                    event_type: event.0.event_type(),
                    callbacks: dispatch.callbacks,
                    failures: std::mem::take(&mut dispatch.failures),
                    default_prevented: event.default_prevented(),
                    manipulation,
                };
                drop(dispatch);
                custody.settle_manipulation();
                Ok(DomEventDispatchOutcome::Complete(receipt))
            }
            other => {
                *custody.borrow_mut() = Some(dispatch);
                custody.settle_manipulation();
                event.0.status.set(Status::Retained);
                Ok(match other {
                    Ok(Err(RunStop::Resource(resource))) => {
                        DomEventDispatchOutcome::Retained(resource)
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
                    Ok(Err(RunStop::ForeignOwner)) => DomEventDispatchOutcome::RetainedForeignOwner,
                    Ok(Err(RunStop::ForeignRegistry)) => {
                        DomEventDispatchOutcome::RetainedForeignRegistry
                    }
                    Err(_) => DomEventDispatchOutcome::RetainedAfterUnwind,
                    Ok(Err(RunStop::Abandoned)) => return Err(DomEventDispatchError::Abandoned),
                    Ok(Ok(())) => unreachable!("completed branch"),
                })
            }
        }
    }
    fn run(
        &mut self,
        dom: &mut Dom,
        context: &mut C,
        event: &NativeDomEvent,
        dispatch: &mut Dispatch,
        budget: &mut Budget,
    ) -> Result<(), RunStop> {
        let count = dispatch.path.len();
        while dispatch.invoke < count * 2 {
            dispatch_ready(&self.owner, dispatch, dom, event)?;
            budget.tick()?;
            let capture = dispatch.invoke < count;
            let index = if capture {
                count - 1 - dispatch.invoke
            } else {
                dispatch.invoke - count
            };
            let item = dispatch.path[index];
            // Nonbubbling ancestor items are skipped by dispatch before invoke;
            // they must not perform invoke's target adjustment as a side effect.
            if !capture && !item.at_target {
                dispatch.next_invoke();
                continue;
            }
            event.0.target.set(Some(item.target));
            // Source invoke checks stopPropagation before initializing currentTarget
            // and cloning the list, including the second invocation at target.
            if (dispatch.snapshot.is_none() && event.0.stop.get()) || event.0.immediate.get() {
                dispatch.next_invoke();
                continue;
            }
            event.0.current.set(Some(item.invocation));
            event.0.phase.set(if item.at_target {
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
                            if self.registration(*id).expect("snapshot registration").order
                                > r.order
                            {
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
            while dispatch.listener < dispatch.snapshot.as_ref().expect("snapshot").len() {
                dispatch_ready(&self.owner, dispatch, dom, event)?;
                budget.tick()?;
                let id = dispatch.snapshot.as_ref().expect("snapshot")[dispatch.listener];
                let Some(r) = self.registration(id) else {
                    dispatch.listener += 1;
                    continue;
                };
                if r.event_type != event.0.event_type() || r.options.capture != capture {
                    dispatch.listener += 1;
                    continue;
                }
                // Native authority admission can refuse before source's
                // irreversible once removal. A known inactive target instead
                // follows inner invoke's skip, including its once removal.
                if !listener_target_active(dom, &dispatch.association_custody, item, budget)? {
                    let once = r.options.once;
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
                let once = r.options.once;
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
                if once {
                    self.remove_listener(id);
                }
                let result = catch_unwind(AssertUnwindSafe(|| {
                    let mut access = DomEventDom::borrow(dom);
                    (callback.0)(&mut access, self, context, event)
                }));
                let failure = match &result {
                    Ok(Ok(())) => None,
                    Ok(Err(error)) => Some(DomEventListenerFailureKind::Reported(*error)),
                    Err(_) => Some(DomEventListenerFailureKind::Panicked),
                };
                if let Some(kind) = failure {
                    dispatch.failures.push(DomEventListenerFailure {
                        registration: id,
                        current_target: item.invocation,
                        kind,
                    });
                }
                if event.0.immediate.get() {
                    break;
                }
            }
            dispatch.next_invoke();
        }
        dispatch_ready(&self.owner, dispatch, dom, event)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Abandoned,
    Ready,
    Running,
    Retained,
    Complete,
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
    DetailsToggle(DomToggleEvent),
    DialogToggle(DomToggleEvent),
    DialogBeforeToggle {
        payload: DomToggleEvent,
        cancelable: bool,
    },
}
struct Event {
    original: NodeId,
    data: EventData,
    status: Cell<Status>,
    target: Cell<Option<NodeId>>,
    current: Cell<Option<DomEventTarget>>,
    phase: Cell<DomEventPhase>,
    stop: Cell<bool>,
    immediate: Cell<bool>,
    canceled: Cell<bool>,
    path: RefCell<Option<Rc<EventPath>>>,
    continuation: std::rc::Weak<EventContinuation>,
    manipulation: RefCell<Option<crate::dom_manipulation::TaskPayload>>,
}
impl Event {
    fn event_type(&self) -> DomEventType {
        match self.data {
            EventData::StyleLoad => DomEventType::Load,
            EventData::StyleError => DomEventType::Error,
            EventData::DetailsToggle(_) | EventData::DialogToggle(_) => DomEventType::Toggle,
            EventData::DialogBeforeToggle { .. } => DomEventType::BeforeToggle,
        }
    }
}
/// A shared native event identity. Clones refer to the same event and cannot
/// redispatch it while running/retained or obtain a second completion receipt.
#[derive(Clone)]
pub struct NativeDomEvent(Rc<Event>);
/// The actual closed source task owns active dispatch custody. Observation
/// clones hold only a Weak reference and cannot extend its admission lifetime.
pub(crate) struct NativeDomEventOwner {
    event: NativeDomEvent,
    // Sole persistent strong custody; active private jobs upgrade temporarily.
    _custody: Rc<EventContinuation>,
}
pub(crate) struct DispatchFormation {
    pub(crate) source: usize,
    pub(crate) full: usize,
}
impl NativeDomEventOwner {
    fn new(original: NodeId, data: EventData) -> Self {
        let custody = Rc::new(EventContinuation {
            dispatch: RefCell::new(None),
            manipulation: RefCell::new(None),
        });
        let event = NativeDomEvent(Rc::new(Event {
            original,
            data,
            status: Cell::new(Status::Ready),
            target: Cell::new(None),
            current: Cell::new(None),
            phase: Cell::new(DomEventPhase::None),
            stop: Cell::new(false),
            immediate: Cell::new(false),
            canceled: Cell::new(false),
            path: RefCell::new(None),
            continuation: Rc::downgrade(&custody),
            manipulation: RefCell::new(None),
        }));
        Self {
            event,
            _custody: custody,
        }
    }

    pub(crate) fn settle_manipulation(&self) {
        self._custody.settle_manipulation();
    }
    pub(crate) fn retain_manipulation(&self, mut retained: crate::dom_manipulation::TaskPayload) {
        if self.event.0.manipulation.borrow().is_none() {
            let bytes = std::mem::size_of::<Event>();
            *self.event.0.manipulation.borrow_mut() = Some(
                retained
                    .split(bytes, bytes)
                    .expect("pre-admitted actual event body"),
            );
        }
        let mut continuation = self._custody.manipulation.borrow_mut();
        if let Some(existing) = continuation.as_mut() {
            existing.absorb(retained).expect("same original task owner");
        } else {
            *continuation = Some(retained);
        }
    }
    pub(crate) fn manipulation_extent(&self) -> (usize, usize) {
        let mut full = 0;
        let mut source = 0;
        for retained in [
            self.event.0.manipulation.borrow(),
            self._custody.manipulation.borrow(),
        ] {
            if let Some(retained) = retained.as_ref() {
                full += retained.full();
                source += retained.source();
            }
        }
        if let Some(dispatch) = self._custody.borrow().as_ref()
            && let Some(retained) = &dispatch.path._manipulation
        {
            full += retained.full();
            source += retained.source();
        }
        (full, source)
    }
    pub(crate) fn initial_payload_bytes() -> Option<usize> {
        std::mem::size_of::<Event>().checked_add(std::mem::size_of::<EventContinuation>())
    }
    pub(crate) fn fresh_dispatch_formation(
        limits: DomEventDispatchLimits,
    ) -> Option<DispatchFormation> {
        Self::formation(limits, None)
    }
    pub(crate) fn dispatch_formation(
        &self,
        limits: DomEventDispatchLimits,
    ) -> Option<DispatchFormation> {
        Self::formation(limits, self._custody.borrow().as_ref())
    }
    fn formation(
        limits: DomEventDispatchLimits,
        retained: Option<&Dispatch>,
    ) -> Option<DispatchFormation> {
        let fixed = Self::initial_payload_bytes()?;
        let new_snapshot = std::mem::size_of::<Vec<DomEventRegistrationId>>().checked_add(
            limits
                .snapshot_listeners
                .checked_mul(std::mem::size_of::<DomEventRegistrationId>())?,
        )?;
        let new_failures = limits
            .failure_records
            .checked_mul(std::mem::size_of::<DomEventListenerFailure>())?;
        let (source, scope) = if let Some(dispatch) = retained {
            let path = std::mem::size_of_val(dispatch.path.as_ref()).checked_add(
                dispatch
                    .path
                    .capacity()
                    .checked_mul(std::mem::size_of::<PathItem>())?,
            )?;
            let base = fixed.checked_add(path)?;
            let old_snapshot = dispatch.snapshot.as_ref().map_or(Some(0), |snapshot| {
                snapshot
                    .capacity()
                    .checked_mul(std::mem::size_of::<DomEventRegistrationId>())
            })?;
            let old_failures = dispatch
                .failures
                .capacity()
                .checked_mul(std::mem::size_of::<DomEventListenerFailure>())?;
            // A raised policy prepares a real replacement while the old bank
            // and actual retained snapshot are still owned. Commit drops the
            // old backing before any subsequent listener can run.
            let replacement = if limits.failure_records > dispatch.failures.capacity() {
                old_failures
                    .checked_add(new_failures)?
                    .checked_add(std::mem::size_of::<Vec<DomEventListenerFailure>>())?
            } else {
                old_failures
            };
            let preparation = base.checked_add(old_snapshot)?.checked_add(replacement)?;
            // Later invocation snapshots form from empty after the previous
            // one drops; its candidate descriptor is distinct from the inline
            // dispatch slot. A lowered path bound never replaces frozen path.
            let run = base
                .checked_add(old_snapshot.max(new_snapshot))?
                .checked_add(old_failures.max(new_failures))?;
            (
                preparation.max(run),
                dispatch.association_custody.retained_bytes(),
            )
        } else {
            let backing = limits
                .path_items
                .checked_mul(std::mem::size_of::<PathItem>())?;
            // Path and exact-node gather are independent formation owners; the
            // gather drops before the path moves into its shared EventPath.
            let path_formation = fixed
                .checked_add(std::mem::size_of::<Vec<PathItem>>())?
                .checked_add(backing)?
                .checked_add(std::mem::size_of::<Vec<NodeId>>())?
                .checked_add(
                    limits
                        .path_items
                        .checked_mul(std::mem::size_of::<NodeId>())?,
                )?;
            let base = fixed
                .checked_add(std::mem::size_of::<EventPath>())?
                .checked_add(backing)?;
            let failure_formation = base
                .checked_add(std::mem::size_of::<Vec<DomEventListenerFailure>>())?
                .checked_add(new_failures)?;
            let run = base.checked_add(new_snapshot)?.checked_add(new_failures)?;
            (
                path_formation.max(failure_formation).max(run),
                DomDispatchCustody::maximum_retained_bytes(limits.path_items).ok()?,
            )
        };
        Some(DispatchFormation {
            source,
            full: source.checked_add(scope)?,
        })
    }
    pub(crate) fn retained_payload_bytes(&self) -> Option<usize> {
        let own = self.retained_source_payload_bytes()?;
        own.checked_add(
            self._custody
                .borrow()
                .as_ref()
                .map_or(0, |dispatch| dispatch.association_custody.retained_bytes()),
        )
    }
    /// Actual caller payload only; canonical Source custody body is separate.
    pub(crate) fn retained_source_payload_bytes(&self) -> Option<usize> {
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
    let event = std::mem::size_of_val(event.0.as_ref());
    let path = path.path.as_ref().map_or(0, |path| {
        std::mem::size_of_val(path.as_ref())
            + path.items.capacity() * std::mem::size_of::<PathItem>()
    });
    (event, path)
}
impl std::ops::Deref for NativeDomEventOwner {
    type Target = NativeDomEvent;
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
impl Drop for NativeDomEventOwner {
    fn drop(&mut self) {
        if self.event.0.status.get() != Status::Complete {
            self.event.0.status.set(Status::Abandoned);
        }
        // Ordinary field drop releases the actual continuation directly; it
        // never borrows a RefCell, calls user code, or manufactures completion.
    }
}
impl std::fmt::Debug for NativeDomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeDomEvent")
            .field("original_target", &self.0.original)
            .field("event_type", &self.0.event_type())
            .field("status", &self.0.status.get())
            .finish()
    }
}
impl NativeDomEvent {
    pub(crate) fn has_started(&self) -> bool {
        self.0.status.get() != Status::Ready
    }
    /// Construct the load event for an actual owning Style completion operation.
    pub(crate) fn style_load(target: NodeId) -> NativeDomEventOwner {
        NativeDomEventOwner::new(target, EventData::StyleLoad)
    }
    /// Construct the error event for an actual owning Style completion operation.
    pub(crate) fn style_error(target: NodeId) -> NativeDomEventOwner {
        NativeDomEventOwner::new(target, EventData::StyleError)
    }
    /// The actual Details task owner supplies its exact queued/coalesced data.
    /// The selected HTML firing clause supplies no source Element. Equal states
    /// are preserved; this constructor performs no coalescing or live-state read.
    pub(crate) fn details_toggle(
        target: NodeId,
        payload: surgeist_html::HtmlDetailsToggleEvent,
    ) -> NativeDomEventOwner {
        NativeDomEventOwner::new(
            target,
            EventData::DetailsToggle(DomToggleEvent {
                old_state: payload.old_state().as_str(),
                new_state: payload.new_state().as_str(),
            }),
        )
    }

    /// Actual null-source Dialog toggle task payload, including equal states.
    pub(crate) fn dialog_toggle(
        target: NodeId,
        old: surgeist_html::HtmlDialogState,
        new: surgeist_html::HtmlDialogState,
    ) -> NativeDomEventOwner {
        NativeDomEventOwner::new(
            target,
            EventData::DialogToggle(DomToggleEvent {
                old_state: old.as_str(),
                new_state: new.as_str(),
            }),
        )
    }

    /// The actual Dialog cursor supplies the HTML-owned states and the
    /// cancelability selected by its reached firing clause. Show supplies true.
    pub(crate) fn dialog_before_toggle(
        target: NodeId,
        old: surgeist_html::HtmlDialogState,
        new: surgeist_html::HtmlDialogState,
        cancelable: bool,
    ) -> NativeDomEventOwner {
        NativeDomEventOwner::new(
            target,
            EventData::DialogBeforeToggle {
                payload: DomToggleEvent {
                    old_state: old.as_str(),
                    new_state: new.as_str(),
                },
                cancelable,
            },
        )
    }

    /// Exact source event type.
    pub fn event_type(&self) -> DomEventType {
        self.0.event_type()
    }
    /// Immutable state strings from the actual Details task or Dialog firing
    /// clause. Attribute edits during delivery cannot change this payload.
    pub fn toggle_event(&self) -> Option<&DomToggleEvent> {
        match &self.0.data {
            EventData::DetailsToggle(payload)
            | EventData::DialogToggle(payload)
            | EventData::DialogBeforeToggle { payload, .. } => Some(payload),
            EventData::StyleLoad | EventData::StyleError => None,
        }
    }
    /// Current shadow-adjusted target; cleanup clears shadow-internal targets.
    pub fn target(&self) -> Option<NodeId> {
        self.0.target.get()
    }
    /// Current invocation target, cleared between dispatch calls and at completion.
    pub fn current_target(&self) -> Option<DomEventTarget> {
        self.0.current.get()
    }
    /// Current source phase.
    pub fn phase(&self) -> DomEventPhase {
        self.0.phase.get()
    }
    /// Source default: these events do not bubble.
    pub const fn bubbles(&self) -> bool {
        false
    }
    /// The actual firing clause's cancelability. Style and Details are false.
    pub fn cancelable(&self) -> bool {
        matches!(
            self.0.data,
            EventData::DialogBeforeToggle {
                cancelable: true,
                ..
            }
        )
    }
    /// Source default: these events are noncomposed.
    pub const fn composed(&self) -> bool {
        false
    }
    /// Sticky canceled flag retained with the same actual event identity.
    pub fn default_prevented(&self) -> bool {
        self.0.canceled.get()
    }
    /// Set the canceled flag only when the actual firing clause is cancelable.
    pub fn prevent_default(&self) {
        if self.cancelable() {
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
    /// Outside a callback it is empty.
    pub fn composed_path(&self) -> DomEventComposedPath {
        let path = self.0.path.borrow().clone();
        let current = self.0.current.get();
        let current_index = path
            .as_ref()
            .and_then(|path| path.iter().position(|p| Some(p.invocation) == current));
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
    fn next(&mut self) -> Option<Self::Item> {
        let path = self.path.as_ref()?;
        let current = self.current_index?;
        while self.next < path.len() {
            let index = self.next;
            self.next += 1;
            if visible(path, current, index) {
                return Some(path[index].invocation);
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
#[derive(Clone, Copy)]
struct PathItem {
    invocation: DomEventTarget,
    target: NodeId,
    at_target: bool,
    closed_root: bool,
    closed_slot: bool,
    // Exact row in this Dispatch's original affine canonical association scope.
    // Global invocation targets use their own actual current authority instead.
    custody_row: Option<usize>,
}
struct EventPath {
    items: Vec<PathItem>,
    // Shared independently by composed_path iterators after Event/Task release.
    _manipulation: Option<crate::dom_manipulation::TaskPayload>,
}
impl std::ops::Deref for EventPath {
    type Target = Vec<PathItem>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
struct EventContinuation {
    dispatch: RefCell<Option<Dispatch>>,
    // Declared after the actual dispatch so its allocations drop first.
    manipulation: RefCell<Option<crate::dom_manipulation::TaskPayload>>,
}
impl std::ops::Deref for EventContinuation {
    type Target = RefCell<Option<Dispatch>>;
    fn deref(&self) -> &Self::Target {
        &self.dispatch
    }
}
impl EventContinuation {
    fn settle_manipulation(&self) {
        let dispatch = self.dispatch.borrow();
        let own = std::mem::size_of::<Self>()
            + dispatch.as_ref().map_or(0, |dispatch| {
                dispatch.snapshot.as_ref().map_or(0, |snapshot| {
                    snapshot.capacity() * std::mem::size_of::<DomEventRegistrationId>()
                }) + dispatch.failures.capacity() * std::mem::size_of::<DomEventListenerFailure>()
            });
        let full = own
            + dispatch
                .as_ref()
                .map_or(0, |dispatch| dispatch.association_custody.retained_bytes());
        if let Some(retained) = self.manipulation.borrow_mut().as_mut() {
            retained
                .shrink(full, own)
                .expect("actual continuation within pre-admitted maximum");
        }
    }
}
struct Dispatch {
    association_custody: DomDispatchCustody,
    _reservation: PendingReservation,
    path: Rc<EventPath>,
    invoke: usize,
    snapshot: Option<Vec<DomEventRegistrationId>>,
    listener: usize,
    callbacks: usize,
    failures: Vec<DomEventListenerFailure>,
    clear_target: bool,
}
impl Dispatch {
    fn prepare_failure_bank(&mut self, budget: &mut Budget) -> Result<(), DomEventResource> {
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
            replacement.push(*failure);
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
    Resource(DomEventResource),
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
fn listener_target_active(
    dom: &Dom,
    custody: &DomDispatchCustody,
    item: PathItem,
    budget: &mut Budget,
) -> Result<bool, RunStop> {
    budget.tick()?;
    let snapshot = dom.snapshot();
    let document = match item.invocation {
        DomEventTarget::Node(node) => dom
            .dispatch_custody_association(
                custody,
                item.custody_row.ok_or(RunStop::HostedAuthority(
                    DomError::InvalidDispatchCustody.into(),
                ))?,
                node,
            )
            .map_err(|error| RunStop::HostedAuthority(error.into()))?
            .document(),
        DomEventTarget::Global(global) => snapshot
            .global_document(global)
            .map_err(RunStop::HostedAuthority)?,
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
    if !document
        .is_fully_active()
        .map_err(RunStop::HostedAuthority)?
    {
        return Ok(false);
    }
    budget.tick()?;
    let global = match item.invocation {
        DomEventTarget::Node(_) => dom
            .relevant_global(document.id().node())
            .map_err(RunStop::HostedAuthority)?
            .ok_or(RunStop::HostedAuthority(DomHostedError::StaleIdentity))?,
        DomEventTarget::Global(global) => global,
    };
    let current = dom
        .global_document(global)
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
    event: &NativeDomEvent,
) -> Result<(), RunStop> {
    if event.0.status.get() == Status::Abandoned {
        return Err(RunStop::Abandoned);
    }
    if !Rc::ptr_eq(registry, &dispatch._reservation.0) {
        return Err(RunStop::ForeignRegistry);
    }
    if dom.owner() != event.0.original.owner() {
        return Err(RunStop::ForeignOwner);
    }
    if let Some(operation) = dom.source_dispatch_blocker() {
        return Err(RunStop::SourceOperation(operation));
    }
    if let Some(operation) = dom.snapshot().incomplete_hosted_operation() {
        return Err(RunStop::HostedOperation(operation));
    }
    Ok(())
}
struct Budget {
    limits: DomEventDispatchLimits,
    work: u64,
    callbacks: usize,
}
impl Budget {
    fn new(limits: DomEventDispatchLimits) -> Self {
        Self {
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
fn source_tick(budget: &mut Budget) -> Result<(), DomEventDispatchError> {
    budget.tick().map_err(DomEventDispatchError::Admission)
}
fn root(
    snapshot: &DomSnapshot,
    mut node: NodeId,
    budget: &mut Budget,
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
fn shadow_including_ancestor(
    snapshot: &DomSnapshot,
    ancestor: NodeId,
    mut node: NodeId,
    budget: &mut Budget,
) -> Result<bool, DomEventDispatchError> {
    loop {
        source_tick(budget)?;
        if ancestor == node {
            return Ok(true);
        }
        match snapshot
            .node(node)
            .map_err(DomEventDispatchError::Dom)?
            .shadow_including_parent()
        {
            Some(parent) => node = parent,
            None => return Ok(false),
        }
    }
}
fn assigned(snapshot: &DomSnapshot, node: NodeId) -> Result<Option<NodeId>, DomEventDispatchError> {
    let node = snapshot.node(node).map_err(DomEventDispatchError::Dom)?;
    if matches!(node.kind(), DomNodeKind::Element(_) | DomNodeKind::Text(_)) {
        node.stored_assigned_slot()
            .map_err(DomEventDispatchError::Dom)
    } else {
        Ok(None)
    }
}
fn build_path(
    dom: &Dom,
    snapshot: &DomSnapshot,
    original: NodeId,
    event_type: DomEventType,
    budget: &mut Budget,
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
            target,
            at_target,
            closed_root,
            closed_slot,
            custody_row: None,
        });
        let assigned = assigned(snapshot, node)?;
        let parent = if let Some(shadow) = view.as_shadow_root() {
            // EventInit.composed is false, but the root test is against the
            // *first invocation target*, not the first encountered ShadowRoot.
            if first_root == node {
                None
            } else {
                Some(shadow.host())
            }
        } else if let Some(document) = view.as_document() {
            // Selected native prestart admission: retired hosted provenance
            // cannot become a fresh closed proof when BC state clears. This is
            // not an extra requirement of Document's source parent algorithm.
            let global = if document
                .metadata()
                .document_environment()
                .has_ever_had_navigable()
            {
                source_tick(budget)?;
                Some(
                    dom.relevant_global(node)
                        .map_err(DomEventDispatchError::Hosted)?
                        .ok_or(DomEventDispatchError::UnsupportedDocumentParent)?,
                )
            } else {
                None
            };
            if event_type != DomEventType::Load
                && document
                    .metadata()
                    .document_environment()
                    .browsing_context()
                    != surgeist_html::HtmlBrowsingContextPresence::Absent
            {
                let global = global.ok_or(DomEventDispatchError::UnsupportedDocumentParent)?;
                source_tick(budget)?;
                if path.len() >= budget.limits.path_items {
                    return Err(DomEventDispatchError::Admission(
                        DomEventResource::PathItems,
                    ));
                }
                path.push(PathItem {
                    invocation: DomEventTarget::Global(global),
                    target,
                    at_target: false,
                    closed_root: false,
                    closed_slot: false,
                    custody_row: None,
                });
            }
            // Load stops at Document. A non-load hosted Document adds its
            // actual relevant Global, whose default EventTarget parent is null.
            None
        } else {
            assigned.or(view.parent())
        };
        let Some(parent) = parent else {
            break;
        };
        closed_slot = assigned.is_some()
            && snapshot
                .node(root(snapshot, parent, budget)?)
                .map_err(DomEventDispatchError::Dom)?
                .as_shadow_root()
                .is_some_and(|r| r.state().mode() == HtmlShadowRootMode::Closed);
        at_target =
            !shadow_including_ancestor(snapshot, root(snapshot, target, budget)?, parent, budget)?;
        if at_target {
            target = parent;
        }
        node = parent;
    }
    Ok(path)
}

#[cfg(test)]
mod tests;
