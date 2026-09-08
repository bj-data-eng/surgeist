use super::{
    Command, Error, ErrorCode, Id, Metrics, Result, Size, WindowSnapshot,
    command::Action,
    lease::{Lease, LeaseWeak, ReleaseNotifier},
    normalization::{NormalizedCommand, normalize_command},
};
#[cfg(feature = "accessibility")]
use crate::planning::AccessibilityPhase;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// Owned registry entry prepared for one window generation.
///
/// The registry owns lifecycle and identity reservation. An `Instance` becomes
/// live only after successful [`Registry::insert`]; closing then moves it into
/// private resource-owning cleanup.
#[derive(Debug)]
pub struct Instance {
    pub(crate) state: WindowSnapshot,
    pub(crate) handle: Option<Handle>,
    inner_size_bounds: InnerSizeBounds,
}

impl Instance {
    /// Creates an instance prepared for insertion with no native handle attached.
    ///
    /// A successful [`Registry::insert`] transitions its identity to `Live`.
    #[must_use]
    pub fn new(state: WindowSnapshot) -> Self {
        Self {
            state,
            handle: None,
            inner_size_bounds: InnerSizeBounds::default(),
        }
    }

    /// Creates an instance prepared for insertion that owns a cloneable native handle lease.
    ///
    /// A successful [`Registry::insert`] transitions its identity to `Live`.
    #[must_use]
    pub fn with_handle(state: WindowSnapshot, handle: Handle) -> Self {
        Self {
            state,
            handle: Some(handle),
            inner_size_bounds: InnerSizeBounds::default(),
        }
    }

    /// Returns this entry's stable issued identity.
    #[must_use]
    pub const fn id(&self) -> Id {
        self.state.id()
    }

    /// Returns the currently observed runtime snapshot.
    #[must_use]
    pub const fn state(&self) -> &WindowSnapshot {
        &self.state
    }

    /// Returns the mutable runtime snapshot while this entry remains registry-owned.
    pub fn state_mut(&mut self) -> &mut WindowSnapshot {
        &mut self.state
    }

    #[must_use]
    pub(crate) const fn inner_size_bounds(&self) -> InnerSizeBounds {
        self.inner_size_bounds
    }

    pub(crate) const fn set_inner_size_bounds(&mut self, bounds: InnerSizeBounds) {
        self.inner_size_bounds = bounds;
    }

    /// Borrows this entry through the public read-only native access view.
    #[must_use]
    pub fn as_ref(&self) -> Ref<'_> {
        Ref { instance: self }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InnerSizeBounds {
    pub(crate) minimum: Option<Size>,
    pub(crate) maximum: Option<Size>,
}

impl InnerSizeBounds {
    #[must_use]
    pub(crate) const fn new(minimum: Option<Size>, maximum: Option<Size>) -> Self {
        Self { minimum, maximum }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RegistryProjection {
    pub(crate) next: u64,
    pub(crate) lifecycle: HashMap<Id, LifecycleState>,
    pub(crate) live: HashMap<Id, ProjectedInstance>,
    #[cfg(feature = "accessibility")]
    pub(crate) accessibility: HashMap<Id, AccessibilityPhase>,
}

/// Issued-generation state retained after an instance leaves live lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LifecycleState {
    Reserved,
    Live,
    Closing,
    Closed,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectedInstance {
    pub(crate) state: WindowSnapshot,
    pub(crate) inner_size_bounds: InnerSizeBounds,
}

/// Borrowed read-only access to one live registry entry.
///
/// This view cannot outlive its `Instance`. Use [`Access`] for identity,
/// metrics, cloned-handle, and borrowed raw-handle operations.
#[derive(Clone, Copy, Debug)]
pub struct Ref<'a> {
    pub(crate) instance: &'a Instance,
}

/// Cloneable native handle lease for renderer integrations.
///
/// Clones share ownership of one native window resource. [`HasWindowHandle`] and
/// [`HasDisplayHandle`] access remains available in `Live` and resource-owning
/// `Closing`. In `Destroyed` and `LoopExited`, both traits return
/// [`raw_window_handle::HandleError::Unavailable`]. Dropping the final owning
/// clone releases the native resource and attempts a best-effort release
/// notification to the event loop. The notification can be discarded after
/// event-loop ingress closes, so delivery is not guaranteed.
#[derive(Clone, Debug)]
pub struct Handle {
    lease: Lease<winit::window::Window>,
}

impl Handle {
    #[must_use]
    pub(crate) fn from_winit(
        id: Id,
        window: winit::window::Window,
        notifier: ReleaseNotifier,
    ) -> Self {
        Self {
            lease: Lease::new(id, window, notifier),
        }
    }

    pub(crate) fn with_winit<T>(
        &self,
        access: impl FnOnce(&winit::window::Window) -> T,
    ) -> Result<T> {
        self.lease.with_resource(access).map_err(|source| {
            Error::new(ErrorCode::HandleUnavailable, "native handle unavailable")
                .with_id(self.lease.id())
                .with_source(source)
        })
    }

    pub(crate) fn request_draw(&self) -> Result<()> {
        self.with_winit(winit::window::Window::request_redraw)
    }

    pub(crate) fn mark_closing(&self) {
        self.lease.mark_closing();
    }

    pub(crate) fn mark_destroyed(&self) {
        self.lease.mark_destroyed();
    }

    pub(crate) fn mark_loop_exited(&self) {
        self.lease.mark_loop_exited();
    }

    pub(crate) fn downgrade(&self) -> LeaseWeak<winit::window::Window> {
        self.lease.downgrade()
    }
}

impl raw_window_handle::HasWindowHandle for Handle {
    fn window_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError>
    {
        self.lease.with_resource(|window| window.window_handle())?
    }
}

impl raw_window_handle::HasDisplayHandle for Handle {
    fn display_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError>
    {
        self.lease.with_resource(|window| window.display_handle())?
    }
}

/// Cloneable ingress handle for sending typed window commands and event-loop actions.
///
/// Obtain a proxy from `Context::proxy()` inside a handler callback, clone it,
/// and move it to another thread when external work needs to wake the window
/// loop. Public command helpers are implemented on `Proxy` by the app-facing
/// DSL module: `send`, `open`, `close`, `draw`, `again`, `at`, and `exit`.
/// Their results report queue acceptance only; planning and application remain
/// on the event-loop thread. Accepted target work that reaches that thread after
/// its window is closing or stale is resolved later by lifecycle rules and is
/// ignored without changing terminal state. Closed native ingress and closed
/// deterministic queues return [`ErrorCode::CommandFailed`].
#[derive(Clone, Debug)]
pub struct Proxy {
    sender: ProxySender,
}

#[derive(Clone, Debug)]
enum ProxySender {
    Winit {
        sender: winit::event_loop::EventLoopProxy<UserEvent>,
        ingress: Arc<WinitIngress>,
    },
    Queue(Arc<ProxyQueue>),
}

/// Shared close gate for every clone of a native event-loop proxy.
#[derive(Debug, Default)]
struct WinitIngress {
    closed: Mutex<bool>,
}

impl WinitIngress {
    fn send(
        &self,
        sender: &winit::event_loop::EventLoopProxy<UserEvent>,
        event: UserEvent,
    ) -> Result<()> {
        let closed = self
            .closed
            .lock()
            .expect("native proxy ingress mutex must not be poisoned");
        if *closed {
            return Err(Error::new(ErrorCode::CommandFailed, "event loop is closed"));
        }
        sender
            .send_event(event)
            .map_err(|_| Error::new(ErrorCode::CommandFailed, "event loop is closed"))
    }

    fn close(&self) {
        *self
            .closed
            .lock()
            .expect("native proxy ingress mutex must not be poisoned") = true;
    }
}

/// Shared FIFO sender state for deterministic event-loop driving.
#[derive(Debug, Default)]
pub(crate) struct ProxyQueue {
    state: Mutex<ProxyQueueState>,
}

#[derive(Debug, Default)]
struct ProxyQueueState {
    events: VecDeque<UserEvent>,
    closed: bool,
}

impl ProxyQueue {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn send(&self, event: UserEvent) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .expect("proxy queue mutex must not be poisoned");
        if state.closed {
            return Err(Error::new(ErrorCode::CommandFailed, "event loop is closed"));
        }
        state.events.push_back(event);
        Ok(())
    }

    fn ensure_open(&self) -> Result<()> {
        let state = self
            .state
            .lock()
            .expect("proxy queue mutex must not be poisoned");
        if state.closed {
            return Err(Error::new(ErrorCode::CommandFailed, "event loop is closed"));
        }
        Ok(())
    }

    pub(crate) fn pop(&self) -> Option<UserEvent> {
        self.state
            .lock()
            .expect("proxy queue mutex must not be poisoned")
            .events
            .pop_front()
    }

    pub(crate) fn close(&self) {
        let mut state = self
            .state
            .lock()
            .expect("proxy queue mutex must not be poisoned");
        state.closed = true;
        state.events.clear();
    }
}

impl Proxy {
    pub(crate) fn from_winit(inner: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self {
            sender: ProxySender::Winit {
                sender: inner,
                ingress: Arc::new(WinitIngress::default()),
            },
        }
    }

    pub(crate) fn with_queue(queue: Arc<ProxyQueue>) -> Self {
        Self {
            sender: ProxySender::Queue(queue),
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn winit_proxy(&self) -> Option<winit::event_loop::EventLoopProxy<UserEvent>> {
        match &self.sender {
            ProxySender::Winit { sender, .. } => Some(sender.clone()),
            ProxySender::Queue(_) => None,
        }
    }

    pub(crate) fn command(&self, command: Command) -> Result<()> {
        if let ProxySender::Queue(queue) = &self.sender {
            queue.ensure_open()?;
        }
        self.send_user_event(UserEvent::Command(normalize_command(command)?))
    }

    pub(crate) fn request_action(&self, action: Action) -> Result<()> {
        self.send_user_event(UserEvent::Action(action))
    }

    /// Enqueue an event-loop exit action.
    ///
    /// Returns [`ErrorCode::CommandFailed`] if the event loop is closed.
    pub fn exit(&self) -> Result<()> {
        self.send_user_event(UserEvent::Action(Action::Exit))
    }

    fn send_user_event(&self, event: UserEvent) -> Result<()> {
        match &self.sender {
            ProxySender::Winit { sender, ingress } => ingress.send(sender, event),
            ProxySender::Queue(queue) => queue.send(event),
        }
    }

    pub(crate) fn send_release(&self, id: Id) -> Result<()> {
        self.send_user_event(UserEvent::HandleReleased(HandleRelease::new(id)))
    }

    /// Prevents all proxy clones from accepting additional ingress work.
    pub(crate) fn close_ingress(&self) {
        match &self.sender {
            ProxySender::Winit { ingress, .. } => ingress.close(),
            ProxySender::Queue(queue) => queue.close(),
        }
    }
}

/// Immutable native-handle release identity delivered to the event loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HandleRelease {
    id: Id,
}

impl HandleRelease {
    pub(crate) const fn new(id: Id) -> Self {
        Self { id }
    }

    pub(crate) const fn id(self) -> Id {
        self.id
    }
}

#[derive(Debug)]
pub(crate) enum UserEvent {
    Action(Action),
    Command(NormalizedCommand),
    HandleReleased(HandleRelease),
    #[cfg(feature = "accessibility")]
    Accessibility(accesskit_winit::Event),
}

#[cfg(feature = "accessibility")]
impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::Accessibility(event)
    }
}

/// Common access operations for a borrowed live entry.
pub trait Access {
    /// Returns the stable identity of the borrowed live entry.
    fn id(&self) -> Id;
    /// Returns a copy of the entry's current observed metrics.
    fn metrics(&self) -> Metrics;
    /// Returns a cloned native lease, or [`ErrorCode::HandleUnavailable`] when absent.
    fn handle(&self) -> Result<Handle>;
    /// Borrows the native window handle through the active lease.
    ///
    /// Availability follows the lease: `Live` and resource-owning `Closing`
    /// succeed, while `Destroyed` and `LoopExited` report unavailability.
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>>;
    /// Borrows the native display handle through the active lease.
    ///
    /// Availability follows the same lifecycle rules as [`Access::window_handle`].
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>>;
}

impl Access for Ref<'_> {
    fn id(&self) -> Id {
        self.instance.id()
    }

    fn metrics(&self) -> Metrics {
        self.instance.state.metrics().clone()
    }

    fn handle(&self) -> Result<Handle> {
        self.instance.handle.clone().ok_or_else(|| {
            Error::new(ErrorCode::HandleUnavailable, "native handle unavailable").with_id(self.id())
        })
    }

    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>> {
        self.instance
            .handle
            .as_ref()
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::HandleUnavailable,
                    "native window handle unavailable",
                )
                .with_id(self.id())
            })?
            .window_handle()
            .map_err(|source| {
                Error::new(
                    ErrorCode::HandleUnavailable,
                    "native window handle unavailable",
                )
                .with_id(self.id())
                .with_source(source)
            })
    }

    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>> {
        self.instance
            .handle
            .as_ref()
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::HandleUnavailable,
                    "native display handle unavailable",
                )
                .with_id(self.id())
            })?
            .display_handle()
            .map_err(|source| {
                Error::new(
                    ErrorCode::HandleUnavailable,
                    "native display handle unavailable",
                )
                .with_id(self.id())
                .with_source(source)
            })
    }
}

/// Runtime owner of issued identities and live native windows.
///
/// Identities are reserved monotonically and are never reused. Closing removes
/// an entry from live lookup before private cleanup completes, and closed
/// generations remain recorded to distinguish lifecycle collisions from a new
/// live entry.
#[derive(Debug, Default)]
pub struct Registry {
    next: u64,
    lifecycle: HashMap<Id, LifecycleState>,
    instances: HashMap<Id, Instance>,
    #[cfg(feature = "accessibility")]
    accessibility: HashMap<Id, AccessibilityPhase>,
}

impl Registry {
    /// Creates an empty registry with no issued identities.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserves one never-reused runtime identity.
    ///
    /// A reservation advances the allocation high-water mark even when no live
    /// entry is inserted. Returns [`ErrorCode::IdentityExhausted`] on overflow.
    pub fn reserve_id(&mut self) -> Result<Id> {
        let candidate = self.next.checked_add(1).ok_or_else(|| {
            Error::new(
                ErrorCode::IdentityExhausted,
                "no runtime window identities remain",
            )
        })?;
        let id = Id::from_u64(candidate);

        self.next = candidate;
        let was_new = self
            .lifecycle
            .insert(id, LifecycleState::Reserved)
            .is_none();
        debug_assert!(
            was_new,
            "allocation high water must not be previously issued"
        );
        Ok(id)
    }

    /// Reserves the exact next identity selected by a virtual command plan.
    pub(crate) fn reserve_planned(&mut self, id: Id) -> Result<()> {
        let expected = self.next.checked_add(1).ok_or_else(|| {
            Error::new(
                ErrorCode::IdentityExhausted,
                "no runtime window identities remain",
            )
        })?;

        if id != Id::from_u64(expected) {
            return Err(Error::new(
                ErrorCode::CommandFailed,
                "planned window identity is no longer the next allocatable identity",
            )
            .with_id(id));
        }

        let reserved = self.reserve_id()?;
        debug_assert_eq!(reserved, id, "planned identity must match allocator output");
        Ok(())
    }

    /// Retires a pending reservation while retaining its issued identity history.
    pub(crate) fn retire_pending_reservation(&mut self, id: Id) {
        let Some(state) = self.lifecycle.get_mut(&id) else {
            debug_assert!(false, "pending reservations must already be issued");
            return;
        };
        debug_assert_eq!(*state, LifecycleState::Reserved);
        if *state == LifecycleState::Reserved {
            *state = LifecycleState::Closed;
        }
    }

    /// Inserts a coherent live instance without replacing an existing identity.
    ///
    /// A pending `Reserved` identity is accepted and promoted to `Live`.
    /// Rejects empty names, mismatched snapshot/metrics identities, names held
    /// by a live entry, and identity collisions with `Live`, `Closing`, or
    /// closed/retired generations.
    pub fn insert(&mut self, instance: Instance) -> Result<()> {
        let id = instance.id();
        if instance.state().name() == Some("") {
            return Err(
                Error::new(ErrorCode::InvalidRequest, "window name must not be empty").with_id(id),
            );
        }

        if id != instance.state().id() || id != instance.state().metrics().id() {
            return Err(Error::new(
                ErrorCode::DuplicateIdentity,
                "window instance identity does not match its snapshot metrics",
            )
            .with_id(id));
        }

        if self.instances.contains_key(&id)
            || matches!(
                self.lifecycle.get(&id),
                Some(LifecycleState::Live | LifecycleState::Closing | LifecycleState::Closed)
            )
        {
            return Err(Error::new(
                ErrorCode::DuplicateIdentity,
                "window identity has already been issued",
            )
            .with_id(id));
        }

        if let Some(name) = instance.state().name()
            && self
                .instances
                .values()
                .any(|existing| existing.state().name() == Some(name))
        {
            return Err(
                Error::new(ErrorCode::DuplicateIdentity, "window name is already live").with_id(id),
            );
        }

        self.instances.insert(id, instance);
        self.lifecycle.insert(id, LifecycleState::Live);
        self.next = self.next.max(id.as_u64());
        Ok(())
    }

    /// Closes and removes a live entry, returning it after lifecycle completion.
    ///
    /// This convenience path is unavailable for non-live identities and records
    /// the issued generation as closed rather than releasing it for reuse.
    pub fn remove(&mut self, id: Id) -> Option<Instance> {
        let instance = self.begin_close(id)?;
        let completed = self.complete_close(id);
        debug_assert!(
            completed,
            "a live generation must complete closure exactly once"
        );
        Some(instance)
    }

    /// Moves one live instance into private closing ownership exactly once.
    pub(crate) fn begin_close(&mut self, id: Id) -> Option<Instance> {
        if self.lifecycle.get(&id) != Some(&LifecycleState::Live) {
            return None;
        }

        let instance = self.instances.remove(&id)?;
        self.lifecycle.insert(id, LifecycleState::Closing);
        #[cfg(feature = "accessibility")]
        self.accessibility.remove(&id);
        if let Some(handle) = instance.handle.as_ref() {
            handle.mark_closing();
        }
        Some(instance)
    }

    /// Records completion for a closing generation exactly once.
    pub(crate) fn complete_close(&mut self, id: Id) -> bool {
        let Some(state) = self.lifecycle.get_mut(&id) else {
            return false;
        };
        if *state != LifecycleState::Closing {
            return false;
        }
        *state = LifecycleState::Closed;
        true
    }

    /// Removes every live instance for terminal event-loop teardown.
    ///
    /// Teardown has no `Closed` delivery, so it advances lifecycle state while
    /// returning the runner-owned handles for final invalidation and release.
    pub(crate) fn take_live_for_teardown(&mut self) -> Vec<Instance> {
        self.live_ids()
            .into_iter()
            .filter_map(|id| {
                let instance = self.begin_close(id)?;
                debug_assert!(
                    self.complete_close(id),
                    "a live generation must become closed during terminal teardown"
                );
                Some(instance)
            })
            .collect()
    }

    #[must_use]
    /// Returns a borrowed view only while `id` remains live in registry lookup.
    pub fn get(&self, id: Id) -> Option<Ref<'_>> {
        self.instances.get(&id).map(Instance::as_ref)
    }

    #[must_use]
    /// Returns the live identity currently associated with a nonempty name.
    pub fn window_id(&self, name: impl AsRef<str>) -> Option<Id> {
        let name = name.as_ref();
        if name.is_empty() {
            return None;
        }

        self.instances
            .values()
            .find(|instance| instance.state.name() == Some(name))
            .map(Instance::id)
    }

    pub(crate) fn get_mut(&mut self, id: Id) -> Option<&mut Instance> {
        self.instances.get_mut(&id)
    }

    #[must_use]
    pub(crate) fn planning_projection(&self) -> RegistryProjection {
        RegistryProjection {
            next: self.next,
            lifecycle: self.lifecycle.clone(),
            live: self
                .instances
                .iter()
                .map(|(id, instance)| {
                    (
                        *id,
                        ProjectedInstance {
                            state: instance.state.clone(),
                            inner_size_bounds: instance.inner_size_bounds,
                        },
                    )
                })
                .collect(),
            #[cfg(feature = "accessibility")]
            accessibility: self.accessibility.clone(),
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn insert_accessibility_phase(&mut self, id: Id, phase: AccessibilityPhase) {
        debug_assert!(
            self.instances.contains_key(&id),
            "accessibility phases belong only to live windows"
        );
        let replaced = self.accessibility.insert(id, phase);
        debug_assert!(
            replaced.is_none(),
            "a live window receives its accessibility phase only once"
        );
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn request_accessibility_tree(&mut self, id: Id) {
        if let Some(phase) = self.accessibility.get_mut(&id) {
            *phase = AccessibilityPhase::WaitingForInitialTree;
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn deactivate_accessibility(&mut self, id: Id) {
        if let Some(phase) = self.accessibility.get_mut(&id) {
            *phase = AccessibilityPhase::Absent;
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn apply_accessibility_phase(&mut self, id: Id, phase: AccessibilityPhase) {
        if let Some(current) = self.accessibility.get_mut(&id) {
            *current = phase;
        }
    }

    #[cfg(feature = "accessibility")]
    pub(crate) fn accessibility_phase(&self, id: Id) -> Option<AccessibilityPhase> {
        self.accessibility.get(&id).copied()
    }

    /// Returns whether `id` is presently live, excluding closing and closed entries.
    #[must_use]
    pub fn contains(&self, id: Id) -> bool {
        self.instances.contains_key(&id)
    }

    #[must_use]
    pub(crate) fn lifecycle_state(&self, id: Id) -> Option<LifecycleState> {
        self.lifecycle.get(&id).copied()
    }

    pub(crate) fn live_ids(&self) -> Vec<Id> {
        let mut ids = self.instances.keys().copied().collect::<Vec<_>>();
        ids.sort_by_key(|id| id.as_u64());
        ids
    }

    /// Returns the number of entries presently live in registry lookup.
    #[must_use]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Returns whether no entries are presently live in registry lookup.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}
