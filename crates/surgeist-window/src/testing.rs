//! Deterministic non-native window testing utilities.
//!
//! Use [`Runner`] for handler callbacks, lifecycle ordering, and terminal-result
//! parity with the native pump. [`Host`] stays callback-free for focused command,
//! state, and effect recording assertions.

use super::{
    command::Action,
    loop_::prepare_startup,
    normalization::{NormalizedCommand, normalize_command, normalize_commands},
    planning::{
        BatchPlan, CommandPlanner, ResolvedImeConfig, ResolvedImeRequest, SizeApplicationResult,
        StagedBatchPlan, accepts_action_target, accepts_work_target, apply_batch_plan,
        take_resolved_host_command,
    },
    pump::{
        Callback, CallbackCompletion, CallbackEnvironment, Pump, PumpBackend, WorkOrigin,
        callback_for_event, enqueue_ready_after_created, enqueue_resume_callbacks,
        enqueue_suspend_callbacks, invoke_handler_callback,
    },
    registry::{InnerSizeBounds, LifecycleState, ProxyQueue},
    *,
};

#[cfg(test)]
use super::{context::resolve_actions_with, dsl::Event as EventScope};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

// Test utilities for exercising the window contract without opening native windows.

/// Low-level lifecycle fact recorded by [`Host`].
///
/// These records describe deterministic host state/effects, not generic
/// production callbacks. Use [`Runner`] when a test must exercise handler
/// dispatch and lifecycle ordering.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// A fake open inserted this observed snapshot.
    Created(WindowSnapshot),
    /// Fake close completion removed this window identity.
    Destroyed(Id),
    /// The host recorded suspension for this live window.
    Suspended(Id),
    /// The host recorded resumption for this live window.
    Resumed(Id),
    /// The host received a low-level native close request for this live window.
    CloseRequested(Id),
    /// The host observed a focus transition.
    Focused {
        /// The affected window identity.
        id: Id,
        /// Whether the window became focused.
        focused: bool,
    },
    /// The host applied observed resize metrics.
    Resized(Metrics),
    /// The host applied observed scale-factor metrics.
    ScaleFactorChanged(Metrics),
    /// The host applied an observed outer-position transition.
    Moved {
        /// The affected window identity.
        id: Id,
        /// The observed logical outer position.
        position: Point,
    },
    /// The host observed a change in occlusion.
    Occluded {
        /// The affected window identity.
        id: Id,
        /// Whether the window is occluded.
        occluded: bool,
    },
    /// The host observed an effective-theme change.
    ThemeChanged {
        /// The affected window identity.
        id: Id,
        /// The observed theme, or `None` when the host has no explicit value.
        theme: Option<Theme>,
    },
    /// The host recorded a normalized file-drag payload.
    FileDrag(FileDragEvent),
    /// The host recorded normalized input.
    Input(InputEvent),
    #[cfg(feature = "accessibility")]
    /// The host recorded a typed accessibility request or deactivation.
    Accessibility(AccessibilityEvent),
}

impl Event {
    #[must_use]
    /// Returns the window identity carried by this low-level record.
    pub fn id(&self) -> Id {
        match self {
            Self::Created(state) => state.id(),
            Self::Destroyed(id)
            | Self::Suspended(id)
            | Self::Resumed(id)
            | Self::CloseRequested(id) => *id,
            Self::Focused { id, .. }
            | Self::Moved { id, .. }
            | Self::Occluded { id, .. }
            | Self::ThemeChanged { id, .. } => *id,
            Self::Resized(metrics) | Self::ScaleFactorChanged(metrics) => metrics.id(),
            Self::FileDrag(event) => event.id(),
            Self::Input(event) => event.id(),
            #[cfg(feature = "accessibility")]
            Self::Accessibility(event) => event.id(),
        }
    }
}

impl From<EventKind> for Event {
    fn from(event: EventKind) -> Self {
        match event {
            EventKind::Created(state) => Self::Created(state),
            EventKind::Suspended(id) => Self::Suspended(id),
            EventKind::Resumed(id) => Self::Resumed(id),
            EventKind::Focused { id, focused } => Self::Focused { id, focused },
            EventKind::Resized(metrics) => Self::Resized(metrics),
            EventKind::ScaleFactorChanged(metrics) => Self::ScaleFactorChanged(metrics),
            EventKind::Moved { id, position } => Self::Moved { id, position },
            EventKind::Occluded { id, occluded } => Self::Occluded { id, occluded },
            EventKind::ThemeChanged { id, theme } => Self::ThemeChanged { id, theme },
            EventKind::FileDrag(event) => Self::FileDrag(event),
            EventKind::Input(event) => Self::Input(event),
            #[cfg(feature = "accessibility")]
            EventKind::Accessibility(event) => Self::Accessibility(event),
        }
    }
}

/// Low-level action record used by deterministic fixture assertions.
///
/// Effects are action-plan observations, not callbacks delivered by production.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// No lifecycle action was requested.
    Wait,
    /// A next-frame draw was requested for this window.
    Draw(Id),
    /// An immediate draw was requested for this window.
    Again(Id),
    /// A draw was requested for this window at an absolute instant.
    At {
        /// The target window identity.
        id: Id,
        /// The requested draw deadline.
        time: Instant,
    },
    /// A specialized close callback should be requested for this window.
    CloseRequested(Id),
    /// A successful terminal exit was requested.
    Exit,
    /// Ordered effects recursively preserved from a batch action.
    ///
    /// Each nested `Action::Batch` becomes a nested `Effect::Batch` in the same
    /// position; conversion deliberately does not flatten batch structure.
    Batch(Vec<Effect>),
}

impl From<Action> for Effect {
    fn from(action: Action) -> Self {
        match action {
            Action::Wait => Self::Wait,
            Action::DrawNow(id) => Self::Again(id),
            Action::DrawNext(id) => Self::Draw(id),
            Action::DrawAt { id, time } => Self::At { id, time },
            Action::CloseRequested(id) => Self::CloseRequested(id),
            Action::Exit => Self::Exit,
            Action::Batch(actions) => Self::Batch(actions.into_iter().map(Into::into).collect()),
        }
    }
}

/// Callback-free deterministic host for low-level command, state, and effect assertions.
///
/// Use [`Runner`] when a test needs lifecycle callbacks or terminal behavior that
/// matches the native pump. `Host` applies authored commands and records their
/// coherent low-level effects without invoking a handler.
#[derive(Debug)]
pub struct Host {
    registry: Registry,
    closing: HashMap<Id, WindowSnapshot>,
    draw: DrawScheduler,
    capabilities: HostCapabilities,
    events: Vec<Event>,
    commands: Vec<Command>,
    cursors: HashMap<Id, Cursor>,
    cursor_grab: HashMap<Id, CursorGrab>,
    cursor_updates: Vec<(Id, Cursor)>,
    ime_requests: Vec<(Id, ImeRequest)>,
    resolved_ime_requests: Vec<(Id, ResolvedImeRequest)>,
    ime_active: HashSet<Id>,
    ime_configurations: HashMap<Id, ResolvedImeConfig>,
    #[cfg(feature = "accessibility")]
    accessibility_updates: Vec<(Id, accesskit::TreeUpdate)>,
    #[cfg(test)]
    backend_failure_at: Option<usize>,
    #[cfg(test)]
    clipboard: MemoryClipboard,
}

fn deterministic_capabilities() -> HostCapabilities {
    HostCapabilities::builder()
        .role(RoleKind::Root, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
        .fullscreen(FullscreenMode::Borderless, CapabilitySupport::Supported)
        .cursor(CursorCapability::Icon, CapabilitySupport::Supported)
        .cursor(CursorCapability::Hidden, CapabilitySupport::Supported)
        .window(WindowOperation::SetTitle, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetOuterPosition,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::SetVisible, CapabilitySupport::Supported)
        .window(WindowOperation::SetResizable, CapabilitySupport::Supported)
        .window(WindowOperation::SetControls, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetDecorations,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::InitialTransparency,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::SetTransparent,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::RequestInnerSize,
            CapabilitySupport::Supported,
        )
        .window(
            WindowOperation::SetInnerSizeBounds,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::SetLevel, CapabilitySupport::Supported)
        .window(
            WindowOperation::SetExplicitTheme,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::ResetTheme, CapabilitySupport::Supported)
        .window(
            WindowOperation::RequestUserAttention,
            CapabilitySupport::Supported,
        )
        .window(WindowOperation::RequestDraw, CapabilitySupport::Supported)
        .window(WindowOperation::Destroy, CapabilitySupport::Supported)
        .cursor_grab(CursorGrab::None, CapabilitySupport::Supported)
        .ime(ImeCapability::Enablement, CapabilitySupport::Supported)
        .ime(ImeCapability::CursorArea, CapabilitySupport::Supported)
        .ime(
            ImeCapability::Purpose(ImePurpose::Normal),
            CapabilitySupport::Supported,
        )
        .ime(
            ImeCapability::Hint(ImeHint::None),
            CapabilitySupport::Supported,
        )
        .build()
}

impl Default for Host {
    fn default() -> Self {
        Self::with_capabilities(deterministic_capabilities())
    }
}

impl Host {
    /// Creates a callback-free host using the given immutable capability report.
    ///
    /// The report controls normalization and planning decisions for all later
    /// commands; this constructor neither opens native windows nor runs handlers.
    #[must_use]
    pub fn with_capabilities(capabilities: HostCapabilities) -> Self {
        Self {
            registry: Registry::default(),
            closing: HashMap::new(),
            draw: DrawScheduler::default(),
            capabilities,
            events: Vec::new(),
            commands: Vec::new(),
            cursors: HashMap::new(),
            cursor_grab: HashMap::new(),
            cursor_updates: Vec::new(),
            ime_requests: Vec::new(),
            resolved_ime_requests: Vec::new(),
            ime_active: HashSet::new(),
            ime_configurations: HashMap::new(),
            #[cfg(feature = "accessibility")]
            accessibility_updates: Vec::new(),
            #[cfg(test)]
            backend_failure_at: None,
            #[cfg(test)]
            clipboard: MemoryClipboard::new(),
        }
    }

    #[must_use]
    /// Creates a callback-free host with its deterministic capability report.
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Returns the host's current low-level registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    #[must_use]
    /// Returns the immutable capability report used for host planning.
    pub fn capabilities(&self) -> &HostCapabilities {
        &self.capabilities
    }

    #[must_use]
    /// Returns recorded low-level facts since construction or [`Host::clear`].
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    #[must_use]
    /// Returns successfully applied authored commands since the last clear.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    #[must_use]
    /// Returns cursor changes that changed the host's retained cursor state.
    pub fn cursor_updates(&self) -> &[(Id, Cursor)] {
        &self.cursor_updates
    }

    #[must_use]
    /// Returns authored IME requests accepted by the host in application order.
    pub fn ime_requests(&self) -> &[(Id, ImeRequest)] {
        &self.ime_requests
    }

    /// Clears retained recordings without changing registry, scheduler, or host state.
    pub fn clear(&mut self) {
        self.events.clear();
        self.commands.clear();
        self.cursor_updates.clear();
        self.ime_requests.clear();
        self.resolved_ime_requests.clear();
        #[cfg(feature = "accessibility")]
        self.accessibility_updates.clear();
    }

    /// Normalizes, plans, and applies one command without invoking a handler.
    ///
    /// Invalid or stale strict targets return an error; successful effects are
    /// retained in the low-level recordings.
    pub fn apply(&mut self, command: impl Into<Command>) -> Result<()> {
        self.apply_batch(vec![command.into()])
    }

    #[cfg(test)]
    pub(crate) fn apply_batch_for_test(&mut self, commands: Vec<Command>) -> Result<()> {
        self.apply_batch(commands)
    }

    #[cfg(test)]
    pub(crate) fn fail_command_at_for_test(&mut self, index: usize) {
        self.backend_failure_at = Some(index);
    }

    #[cfg(test)]
    pub(crate) fn ime_active_for_test(&self, id: Id) -> bool {
        self.ime_active.contains(&id)
    }

    #[cfg(test)]
    pub(crate) fn ime_configuration_for_test(&self, id: Id) -> Option<&ResolvedImeConfig> {
        self.ime_configurations.get(&id)
    }

    #[cfg(test)]
    pub(crate) fn resolved_ime_requests_for_test(&self) -> &[(Id, ResolvedImeRequest)] {
        &self.resolved_ime_requests
    }

    #[cfg(all(feature = "accessibility", test))]
    pub(crate) fn accessibility_phase_for_test(
        &self,
        id: Id,
    ) -> Option<super::planning::AccessibilityPhase> {
        self.registry.accessibility_phase(id)
    }

    #[cfg(all(feature = "accessibility", test))]
    pub(crate) fn accessibility_updates_for_test(&self) -> &[(Id, accesskit::TreeUpdate)] {
        &self.accessibility_updates
    }

    #[cfg(test)]
    pub(crate) fn begin_close_for_test(&mut self, id: Id) -> Result<()> {
        self.registry
            .begin_close(id)
            .map(|_| ())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown live window").with_id(id))
    }

    #[must_use]
    /// Takes windows whose recorded draw deadlines are due at `now`.
    pub fn take_ready_draws(&mut self, now: Instant) -> Vec<Id> {
        self.draw.take_ready(now)
    }

    /// Records resumption for a live window without invoking [`Handler::resume`].
    pub fn resume(&mut self, id: Id) -> Result<()> {
        self.require_window(id)?;
        self.events.push(Event::Resumed(id));
        Ok(())
    }

    /// Records suspension for a live window without invoking [`Handler::suspend`].
    pub fn suspend(&mut self, id: Id) -> Result<()> {
        self.require_window(id)?;
        self.events.push(Event::Suspended(id));
        Ok(())
    }

    #[cfg(feature = "accessibility")]
    /// Records and applies one accessibility host fact when the feature is enabled.
    ///
    /// The target must be live and the configured report must support accessibility.
    pub fn accessibility(&mut self, event: AccessibilityEvent) -> Result<()> {
        self.require_window(event.id())?;
        self.capabilities
            .require_accessibility()
            .map_err(|error| error.with_id(event.id()))?;
        match event {
            AccessibilityEvent::InitialTreeRequested(id) => {
                self.registry.request_accessibility_tree(id);
            }
            AccessibilityEvent::Deactivated(id) => self.registry.deactivate_accessibility(id),
            AccessibilityEvent::ActionRequested(_) => {}
        }
        self.events.push(Event::Accessibility(event));
        Ok(())
    }

    #[must_use]
    /// Looks up a currently live fake window by its authored name.
    pub fn window_id(&self, name: impl AsRef<str>) -> Option<Id> {
        self.registry.window_id(name)
    }

    fn record_close_request(&mut self, id: Id) -> bool {
        if !self.registry.contains(id) {
            return false;
        }

        self.events.push(Event::CloseRequested(id));
        true
    }

    #[cfg(test)]
    pub(crate) fn dispatch_ready<H: Handler>(&mut self, handler: &mut H, id: Id) -> Result<Effect> {
        self.require_window(id)?;

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        {
            let context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            let mut ready = Ready::new(id, context);
            handler.ready(&mut ready)?;
        }

        self.draw.request(&Action::DrawNext(id));
        self.apply_batch(commands)?;

        Ok(resolve_actions_with(&actions, Action::DrawNext(id)).into())
    }

    #[cfg(test)]
    pub(crate) fn dispatch_draw<H: Handler>(&mut self, handler: &mut H, id: Id) -> Result<Effect> {
        self.require_window(id)?;

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let action = {
            let context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            let mut frame = Frame::new(id, context);
            handler.draw(&mut frame)?;
            frame.action().clone()
        };

        self.apply_batch(commands)?;

        Ok(action.into())
    }

    #[cfg(test)]
    pub(crate) fn dispatch_resize<H: Handler>(
        &mut self,
        handler: &mut H,
        metrics: Metrics,
    ) -> Result<Effect> {
        let transition = NativeEventTransition::resized(metrics);
        let id = transition.id();
        let event = transition.apply(self.state_mut(id)?)?;
        if !matches!(event, EventKind::Resized(_)) {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "resize transition did not produce a resize event",
            )
            .with_id(id));
        }

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        {
            let context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            let mut resize = Resize::new(id, context);
            handler.resize(&mut resize)?;
        }

        self.apply_batch(commands)?;

        Ok(resolve_actions_with(&actions, Action::DrawNext(id)).into())
    }

    #[cfg(test)]
    pub(crate) fn dispatch_input<H: Handler>(
        &mut self,
        handler: &mut H,
        input: InputEvent,
    ) -> Result<Effect> {
        let id = input.id();
        self.require_window(id)?;

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let action = {
            let context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            let mut input = Input::new(input, context);
            handler.input(&mut input)?;
            input.context_mut().resolved_action()
        };

        self.apply_batch(commands)?;

        Ok(action.into())
    }

    #[cfg(test)]
    pub(crate) fn dispatch_native_transition<H: Handler>(
        &mut self,
        handler: &mut H,
        transition: NativeEventTransition,
    ) -> Result<Effect> {
        let id = transition.id();
        if transition.requires_specialized_dispatch() {
            return Err(Error::new(
                ErrorCode::UnsupportedFeature,
                "specialized native transitions must use specialized fake dispatch",
            )
            .with_id(id));
        }

        self.require_window(id)?;
        let event = transition.apply(self.state_mut(id)?)?;

        self.events.push(Event::from(event.clone()));

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let action = {
            let context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            let mut event = EventScope::new(event, context);
            handler.event(&mut event)?;
            event.context_mut().resolved_action()
        };

        self.apply_batch(commands)?;

        Ok(action.into())
    }

    #[cfg(test)]
    pub(crate) fn idle<H: Handler>(&mut self, handler: &mut H) -> Result<Option<Effect>> {
        if !handler.wants_idle() {
            return Ok(None);
        }

        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let action = {
            let mut context = Context::new(
                &mut self.registry,
                &mut commands,
                &mut actions,
                &mut self.clipboard,
                &self.capabilities,
                None,
            );
            handler.idle(&mut context)?;
            context.action().clone()
        };

        self.apply_batch(commands)?;

        Ok(Some(action.into()))
    }

    fn require_window(&self, id: Id) -> Result<()> {
        self.registry
            .contains(id)
            .then_some(())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))
    }

    fn state_mut(&mut self, id: Id) -> Result<&mut WindowSnapshot> {
        self.registry
            .get_mut(id)
            .map(Instance::state_mut)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))
    }

    pub(crate) fn apply_transition(
        &mut self,
        transition: NativeEventTransition,
    ) -> Result<EventKind> {
        let id = transition.id();
        self.require_window(id)?;
        let event = transition.apply(self.state_mut(id)?)?;
        self.events.push(Event::from(event.clone()));
        Ok(event)
    }

    fn apply_patch(&mut self, patch: WindowStatePatch) -> Result<()> {
        let id = patch.id();
        let event = {
            let state = self.state_mut(id)?;
            patch.apply(state)?
        };
        if let Some(event) = event {
            self.events.push(event.into());
        }
        Ok(())
    }

    fn apply_batch(&mut self, commands: Vec<Command>) -> Result<()> {
        self.apply_normalized_batch(normalize_commands(commands)?)
    }

    fn apply_normalized_batch(&mut self, commands: Vec<NormalizedCommand>) -> Result<()> {
        let mut pump = Pump::new();
        pump.enqueue_normalized_commands(commands);
        {
            let mut backend = HostPumpBackend { host: self };
            pump.drain(&mut backend);
        }
        pump.into_result()
    }

    pub(crate) fn apply_plan(&mut self, plan: HostCommandPlan) -> Result<()> {
        let resolved = take_resolved_host_command(plan);
        let ime_request = resolved.ime_request;
        #[cfg(feature = "accessibility")]
        let accessibility_update = resolved.accessibility_update;
        #[cfg(feature = "accessibility")]
        let accessibility_open_phase = resolved.accessibility_open_phase;
        let command = resolved.command.into_command();
        let recorded = command.clone();

        match command {
            Command::Open { request } => self.apply_open_request(
                resolved
                    .open_id
                    .expect("preflighted open commands carry a planned identity"),
                request,
                #[cfg(feature = "accessibility")]
                accessibility_open_phase,
            ),
            Command::SetTitle { id, title } => self.apply_patch(WindowStatePatch::title(id, title)),
            Command::SetPosition { id, position } => {
                self.apply_patch(WindowStatePatch::Position { id, position })
            }
            Command::SetVisible { id, visible } => {
                self.apply_patch(WindowStatePatch::visible(id, visible))
            }
            Command::SetInnerSize { id, size } => self.apply_inner_size(id, size),
            Command::SetMinInnerSize { id, size } => self.apply_inner_size_bounds(
                id,
                InnerSizeBounds::new(size, self.window_inner_size_bounds(id)?.maximum),
                resolved.resolved_size,
            ),
            Command::SetMaxInnerSize { id, size } => self.apply_inner_size_bounds(
                id,
                InnerSizeBounds::new(self.window_inner_size_bounds(id)?.minimum, size),
                resolved.resolved_size,
            ),
            Command::SetFullscreen { id, fullscreen } => {
                self.apply_patch(WindowStatePatch::Fullscreen {
                    id,
                    fullscreen: !matches!(fullscreen, Fullscreen::None),
                })
            }
            Command::SetTheme { id, theme } => {
                self.apply_patch(WindowStatePatch::Theme { id, theme })
            }
            Command::SetCursor { id, cursor } => self.apply_cursor(id, cursor),
            Command::SetCursorGrab { id, grab } => self.apply_cursor_grab(id, grab),
            Command::SetIme { id, .. } => self.apply_ime_request(
                id,
                ime_request.expect("planned IME command carries a resolved request"),
            ),
            #[cfg(feature = "accessibility")]
            Command::UpdateAccessibility { id, update } => {
                let resolved = accessibility_update
                    .expect("preflighted accessibility commands carry a resolved phase");
                self.apply_accessibility_update(id, update, resolved)
            }
            Command::RequestDraw { id } => self.apply_draw_request(id),
            Command::Destroy { .. } => Ok(()),
            Command::SetResizable { id, .. }
            | Command::SetControls { id, .. }
            | Command::SetDecorations { id, .. }
            | Command::SetTransparent { id, .. }
            | Command::SetLevel { id, .. }
            | Command::RequestUserAttention { id } => self.require_window(id),
        }?;
        self.commands.push(recorded);
        Ok(())
    }

    fn apply_open_request(
        &mut self,
        id: Id,
        request: WindowRequest,
        #[cfg(feature = "accessibility")] accessibility_open_phase: Option<
            super::planning::AccessibilityPhase,
        >,
    ) -> Result<()> {
        self.registry.reserve_planned(id)?;
        let result = (|| {
            let state = fake_state_from_request(id, &request)?;
            let mut instance = Instance::new(state.clone());
            instance.set_inner_size_bounds(InnerSizeBounds::new(
                request.min_inner_size(),
                request.max_inner_size(),
            ));
            self.registry.insert(instance)?;
            #[cfg(feature = "accessibility")]
            if let Some(phase) = accessibility_open_phase {
                self.registry.insert_accessibility_phase(id, phase);
            }
            self.events.push(Event::Created(state));
            Ok(())
        })();
        if result.is_err() {
            self.registry.retire_pending_reservation(id);
        }
        result
    }

    fn apply_inner_size(&mut self, id: Id, size: Size) -> Result<()> {
        let result = self.immediate_size_result(id, size)?;
        self.apply_size_result(id, result)
    }

    fn immediate_size_result(&self, id: Id, size: Size) -> Result<SizeApplicationResult> {
        let scale = self
            .registry
            .get(id)
            .map(|window| window.metrics().scale_factor())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        Ok(SizeApplicationResult::Immediate(PhysicalSize {
            width: (size.width * scale).round().max(0.0) as u32,
            height: (size.height * scale).round().max(0.0) as u32,
        }))
    }

    fn apply_size_result(&mut self, id: Id, result: SizeApplicationResult) -> Result<()> {
        let SizeApplicationResult::Immediate(size) = result else {
            return Ok(());
        };
        let existing = self.state_mut(id)?.metrics().clone();
        let metrics = Metrics::from_physical_size(id, size, existing.scale_factor())?
            .with_outer_geometry(existing.outer_position(), existing.outer_size())?;
        self.apply_patch(WindowStatePatch::metrics(metrics, MetricsEvent::Resized))
    }

    fn window_inner_size_bounds(&self, id: Id) -> Result<InnerSizeBounds> {
        self.registry
            .get(id)
            .map(|window| window.instance.inner_size_bounds())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))
    }

    fn apply_inner_size_bounds(
        &mut self,
        id: Id,
        bounds: InnerSizeBounds,
        resolved_size: Option<Size>,
    ) -> Result<()> {
        let instance = self
            .registry
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        instance.set_inner_size_bounds(bounds);
        if let Some(size) = resolved_size {
            self.apply_inner_size(id, size)?;
        }
        Ok(())
    }

    fn apply_cursor(&mut self, id: Id, cursor: Cursor) -> Result<()> {
        self.require_window(id)?;
        if self.cursors.get(&id) != Some(&cursor) {
            self.cursors.insert(id, cursor.clone());
            self.cursor_updates.push((id, cursor));
        }
        Ok(())
    }

    fn apply_cursor_grab(&mut self, id: Id, grab: CursorGrab) -> Result<()> {
        self.require_window(id)?;
        if matches!(grab, CursorGrab::None) {
            self.cursor_grab.remove(&id);
        } else {
            self.cursor_grab.insert(id, grab);
        }
        Ok(())
    }

    fn apply_ime_request(&mut self, id: Id, request: ResolvedImeRequest) -> Result<()> {
        self.require_window(id)?;
        match &request {
            ResolvedImeRequest::Disable => {
                self.ime_active.remove(&id);
                self.ime_configurations.remove(&id);
            }
            ResolvedImeRequest::Enable(config) | ResolvedImeRequest::Restart(config) => {
                self.ime_active.insert(id);
                self.ime_configurations.insert(id, config.clone());
            }
            ResolvedImeRequest::Update(config) => {
                self.ime_configurations.insert(id, config.clone());
            }
        }
        self.resolved_ime_requests.push((id, request.clone()));
        self.ime_requests.push((id, request.as_authored()));
        Ok(())
    }

    #[cfg(feature = "accessibility")]
    fn apply_accessibility_update(
        &mut self,
        id: Id,
        update: accesskit::TreeUpdate,
        resolved: super::planning::ResolvedAccessibilityUpdate,
    ) -> Result<()> {
        self.registry.apply_accessibility_phase(id, resolved.phase);
        debug_assert_eq!(
            self.registry.accessibility_phase(id),
            Some(resolved.phase),
            "applicators consume only a phase-resolved update"
        );
        self.require_window(id)?;
        self.accessibility_updates.push((id, update));
        Ok(())
    }

    fn apply_draw_request(&mut self, id: Id) -> Result<()> {
        self.require_window(id)?;
        self.draw.request(&Action::DrawNext(id));
        Ok(())
    }

    fn begin_close(&mut self, id: Id) -> Result<()> {
        let Some(instance) = self.registry.begin_close(id) else {
            return Ok(());
        };

        self.draw.cancel(id);
        self.cursors.remove(&id);
        self.cursor_grab.remove(&id);
        self.ime_active.remove(&id);
        self.ime_configurations.remove(&id);
        let replaced = self.closing.insert(id, instance.state);
        debug_assert!(
            replaced.is_none(),
            "a live generation retains one terminal snapshot while closing"
        );
        Ok(())
    }

    fn complete_close(&mut self, id: Id) -> Option<WindowSnapshot> {
        if !self.registry.complete_close(id) {
            return None;
        }

        let state = self.closing.remove(&id);
        debug_assert!(
            state.is_some(),
            "a closing generation consumes its terminal snapshot exactly once"
        );
        let state = state?;
        self.events.push(Event::Destroyed(id));
        Some(state)
    }
}

/// Deterministic lifecycle runner with the same callback pump as the native host.
///
/// Configure a complete startup batch with [`Runner::startup`] before the first
/// [`Runner::resume`]. Use [`Runner::host`] to inspect low-level Host recordings;
/// the host itself intentionally does not dispatch callbacks. Close requests and
/// one-shot `Closed` completion use the same shared lifecycle as the native pump.
/// [`Runner::proxy`] queues deterministic cross-thread work, and
/// [`Runner::flush_proxy`] processes it after the first resume.
pub struct Runner<H> {
    handler: H,
    host: Host,
    clipboard: Box<dyn Clipboard>,
    startup: Vec<NormalizedCommand>,
    staged_startup: Option<StagedBatchPlan>,
    resumed: bool,
    pump: Pump<Callback>,
    proxy_queue: Arc<ProxyQueue>,
}

impl<H> Runner<H> {
    /// Creates a runner with the conservative deterministic capability report used by [`Host::new`].
    #[must_use]
    pub fn new(handler: H) -> Self {
        Self::with_capabilities(handler, Host::new().capabilities().clone())
    }

    /// Creates a runner using one immutable capability report for Host, planning, and callbacks.
    #[must_use]
    pub fn with_capabilities(handler: H, report: HostCapabilities) -> Self {
        Self {
            handler,
            host: Host::with_capabilities(report),
            clipboard: Box::new(MemoryClipboard::new()),
            startup: Vec::new(),
            staged_startup: None,
            resumed: false,
            pump: Pump::new(),
            proxy_queue: Arc::new(ProxyQueue::new()),
        }
    }

    /// Replaces the loop-owned clipboard borrowed by every runner callback.
    ///
    /// Clipboard I/O occurs while a callback is running and is not rolled back if
    /// that callback later returns an error.
    #[must_use]
    pub fn with_clipboard(mut self, clipboard: Box<dyn Clipboard>) -> Self {
        self.clipboard = clipboard;
        self
    }

    /// Intrinsically validates and stores one complete startup batch before first resume.
    ///
    /// Replacing the batch is allowed until resume. Calling this after resume
    /// returns `InvalidRequest`; calling it after a terminal result is a no-op.
    pub fn startup(&mut self, commands: Vec<Command>) -> Result<()> {
        if !self.pump.is_running() {
            return Ok(());
        }

        if self.resumed {
            return Err(Error::new(
                ErrorCode::InvalidRequest,
                "startup configuration is closed after resume",
            ));
        }

        let (startup, _) = prepare_startup(commands)?;
        self.startup = startup;
        Ok(())
    }

    #[must_use]
    /// Returns the runner's callback handler.
    pub fn handler(&self) -> &H {
        &self.handler
    }

    /// Returns mutable access to the runner's callback handler.
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    /// Returns the callback-free low-level recorder owned by this runner.
    #[must_use]
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// Returns the immutable capability report used by this runner.
    #[must_use]
    pub fn capabilities(&self) -> &HostCapabilities {
        self.host.capabilities()
    }

    /// Returns the retained terminal outcome without consuming or resetting it.
    ///
    /// `None` means the shared pump is still running. The first failure remains
    /// retained; successful exit returns `Some(Ok(()))` and closes proxy ingress.
    #[must_use]
    pub fn result(&self) -> Option<std::result::Result<(), &Error>> {
        self.close_proxy_if_terminal();
        self.pump.result()
    }

    /// Returns a cloneable FIFO proxy owned by this deterministic runner.
    ///
    /// Its result is only queue acceptance. Use [`Runner::flush_proxy`] after
    /// resume to process accepted work; terminal completion closes every clone.
    #[must_use]
    pub fn proxy(&self) -> Proxy {
        Proxy::with_queue(self.proxy_queue.clone())
    }

    fn close_proxy_if_terminal(&self) {
        if !self.pump.is_running() {
            self.proxy_queue.close();
        }
    }

    fn fail(&mut self, error: Error) {
        self.proxy_queue.close();
        self.pump.fail(error);
    }

    #[cfg(test)]
    pub(crate) fn take_ready_draws_for_test(&mut self, now: Instant) -> Vec<Id> {
        self.host.take_ready_draws(now)
    }
}

impl<H> Drop for Runner<H> {
    fn drop(&mut self) {
        self.proxy_queue.close();
    }
}

impl<H: Handler> Runner<H> {
    /// Dispatches one authored command through normalization, planning, and the shared pump.
    ///
    /// Call this only after the first [`Runner::resume`]. A pre-resume dispatch does not
    /// normalize, plan, or apply the command, and records a terminal
    /// [`ErrorCode::InvalidRequest`] in [`Runner::result`].
    pub fn dispatch(&mut self, command: Command) {
        if !self.pump.is_running() {
            return;
        }

        if !self.resumed {
            self.fail(Error::new(
                ErrorCode::InvalidRequest,
                "dispatch requires first resume",
            ));
            return;
        }

        match normalize_command(command) {
            Ok(command) => self.pump.enqueue_normalized_commands(vec![command]),
            Err(error) => {
                self.fail(error);
                return;
            }
        }
        self.drain_pump();
    }

    /// Delivers the first or a later resume sequence through the shared pump.
    ///
    /// The first call validates the complete startup batch, runs global `resume`,
    /// then applies its committed work. Later calls record and deliver per-window
    /// `Resumed` events in deterministic id order before global `resume`. Calls
    /// after terminal completion are no-ops.
    pub fn resume(&mut self) {
        if !self.pump.is_running() {
            return;
        }

        if !self.resumed {
            self.resumed = true;
            match CommandPlanner::from_registry(self.capabilities().clone(), self.host.registry())
                .stage_normalized_batch(std::mem::take(&mut self.startup))
            {
                Ok(staged) => {
                    self.staged_startup = Some(staged);
                    self.pump.enqueue_callback(Callback::Resume);
                }
                Err(error) => {
                    self.fail(error);
                    return;
                }
            }
        } else {
            if let Err(error) = self.record_resumed_windows() {
                self.fail(error);
                return;
            }
            enqueue_resume_callbacks(self.host.registry(), &mut self.pump);
        }
        self.drain_pump();
    }

    /// Delivers deterministic per-window suspension followed by the global callback.
    ///
    /// Before first resume and after terminal completion this is a no-op. Otherwise
    /// each live window records and delivers `Suspended` in deterministic id order
    /// before the global `suspend` callback.
    pub fn suspend(&mut self) {
        if !self.pump.is_running() || !self.resumed {
            return;
        }

        if let Err(error) = self.record_suspended_windows() {
            self.fail(error);
            return;
        }
        enqueue_suspend_callbacks(self.host.registry(), &mut self.pump);
        self.drain_pump();
    }

    /// Delivers idle after first resume when the handler currently requests it.
    ///
    /// Pre-resume, terminal, and opt-out calls are no-ops; callback failure becomes
    /// the runner's retained terminal error.
    pub fn idle(&mut self) {
        if !self.pump.is_running() || !self.resumed || !self.handler.wants_idle() {
            return;
        }

        self.pump.enqueue_callback(Callback::Idle);
        self.drain_pump();
    }

    /// Atomically applies one native transition, then dispatches its native-equivalent callback.
    ///
    /// Transition failure becomes terminal. Calls after terminal completion are
    /// no-ops, and the host records the low-level fact separately from callbacks.
    pub fn transition(&mut self, transition: NativeEventTransition) {
        if !self.pump.is_running() {
            return;
        }

        match self.host.apply_transition(transition) {
            Ok(event) => self.pump.enqueue_callback(callback_for_event(event)),
            Err(error) => {
                self.fail(error);
                return;
            }
        }
        self.drain_pump();
    }

    /// Delivers one typed accessibility host event through the deterministic runner.
    ///
    /// This feature-gated testing parity ingress is not a native/runtime API.
    /// It must be called after the first [`Runner::resume`]; a pre-resume call
    /// retains [`ErrorCode::InvalidRequest`] and records no host fact, phase
    /// change, callback, command, or update. After resume, the runner first
    /// delegates live-target and accessibility-capability validation plus host
    /// recording to [`Host::accessibility`]. A rejected event retains that error
    /// without delivering a callback or applying an update.
    ///
    /// An accepted event advances its host accessibility phase before its
    /// [`EventKind::Accessibility`] callback and then uses the same non-reentrant
    /// pump and callback transaction as native parity: callback-local updates
    /// apply only after the callback succeeds. Callback, planning, or application
    /// failure becomes the retained terminal result and suppresses later
    /// callbacks. Calls after any terminal result are no-ops.
    #[cfg(feature = "accessibility")]
    pub fn accessibility(&mut self, event: AccessibilityEvent) {
        if !self.pump.is_running() {
            return;
        }

        if !self.resumed {
            self.fail(Error::new(
                ErrorCode::InvalidRequest,
                "accessibility requires first resume",
            ));
            return;
        }

        match self.host.accessibility(event.clone()) {
            Ok(()) => self
                .pump
                .enqueue_callback(Callback::Event(EventKind::Accessibility(event))),
            Err(error) => {
                self.fail(error);
                return;
            }
        }
        self.drain_pump();
    }

    /// Delivers one draw callback through the shared pump.
    ///
    /// An unknown target becomes terminal; calls after terminal completion are no-ops.
    pub fn draw(&mut self, id: Id) {
        if !self.pump.is_running() {
            return;
        }

        if let Err(error) = self.host.require_window(id) {
            self.fail(error);
            return;
        }
        self.pump.enqueue_callback(Callback::Frame(id));
        self.drain_pump();
    }

    /// Injects one native close request for a currently live window.
    ///
    /// The request records its low-level fact and invokes the specialized close
    /// callback. Its final close decision is consumed by the shared lifecycle
    /// pump: cancellation leaves the window live, while acceptance completes one
    /// close and delivers one specialized `Closed` callback. Unknown, closing,
    /// closed, and terminal requests are no-ops.
    pub fn request_close(&mut self, id: Id) {
        if !self.pump.is_running() || !self.host.record_close_request(id) {
            return;
        }

        self.pump.enqueue_callback(Callback::Close(id));
        self.drain_pump();
    }

    /// Injects one native destruction fact without invoking the close callback.
    ///
    /// Live and closing windows use the shared begin-close and completion items,
    /// so cleanup and the specialized `Closed` callback occur in the same order
    /// as an authored destroy command. Unknown, closed, duplicate, and terminal
    /// destruction facts are no-ops.
    pub fn destroy(&mut self, id: Id) {
        if !self.pump.is_running()
            || !matches!(
                self.host.registry().lifecycle_state(id),
                Some(LifecycleState::Live | LifecycleState::Closing)
            )
        {
            return;
        }

        self.pump.enqueue_begin_close(id);
        self.pump.enqueue_close_completion(id);
        self.drain_pump();
    }

    /// Drains accepted proxy work in FIFO order after the first resume.
    ///
    /// Before the first resume this leaves queued work untouched. Once resumed,
    /// each event drains through the same non-reentrant pump used by native ingress,
    /// including proxy work submitted by callbacks during this flush.
    pub fn flush_proxy(&mut self) {
        if !self.pump.is_running() {
            self.close_proxy_if_terminal();
            return;
        }
        if !self.resumed {
            return;
        }

        while self.pump.is_running() {
            let Some(event) = self.proxy_queue.pop() else {
                break;
            };
            self.enqueue_proxy_event(event);
            self.drain_pump();
        }
        self.close_proxy_if_terminal();
    }

    fn plan_pump_batch(&mut self, commands: Vec<NormalizedCommand>) -> Result<BatchPlan> {
        if let Some(staged_startup) = self.staged_startup.take() {
            staged_startup.finish_normalized_batch(commands)
        } else {
            CommandPlanner::from_registry(self.capabilities().clone(), self.host.registry())
                .plan_normalized_batch(commands)
        }
    }

    fn record_resumed_windows(&mut self) -> Result<()> {
        for id in self.host.registry().live_ids() {
            self.host.resume(id)?;
        }
        Ok(())
    }

    fn record_suspended_windows(&mut self) -> Result<()> {
        for id in self.host.registry().live_ids() {
            self.host.suspend(id)?;
        }
        Ok(())
    }

    fn drain_pump(&mut self) {
        let mut pump = std::mem::take(&mut self.pump);
        {
            let mut backend = RunnerPumpBackend { runner: self };
            pump.drain(&mut backend);
        }
        self.pump = pump;
        self.close_proxy_if_terminal();
    }

    fn enqueue_proxy_event(&mut self, event: UserEvent) {
        match event {
            UserEvent::Action(action) => self.pump.enqueue_queued_action(action),
            UserEvent::Command(command) => self.pump.enqueue_queued_normalized_command(command),
            UserEvent::HandleReleased(release) => {
                let _ = release;
            }
            #[cfg(feature = "accessibility")]
            UserEvent::Accessibility(_) => {}
        }
    }

    fn enqueue_recorded_follow_ups(
        &self,
        kind: CommandKind,
        event_start: usize,
        callbacks: &mut Vec<Callback>,
    ) {
        for event in &self.host.events[event_start..] {
            match event {
                Event::Created(state) => {
                    if kind == CommandKind::Open {
                        callbacks.push(Callback::Event(EventKind::Created(state.clone())));
                    }
                }
                Event::ThemeChanged { id, theme } => {
                    if kind == CommandKind::SetTheme {
                        callbacks.push(Callback::Event(EventKind::ThemeChanged {
                            id: *id,
                            theme: *theme,
                        }));
                    }
                }
                Event::Resized(metrics) => {
                    if matches!(
                        kind,
                        CommandKind::SetInnerSize
                            | CommandKind::SetMinInnerSize
                            | CommandKind::SetMaxInnerSize
                    ) {
                        callbacks.push(callback_for_event(EventKind::Resized(metrics.clone())));
                    }
                }
                _ => {}
            }
        }
    }
}

struct RunnerPumpBackend<'a, H> {
    runner: &'a mut Runner<H>,
}

impl<H: Handler> PumpBackend<Callback> for RunnerPumpBackend<'_, H> {
    fn invoke_callback(
        &mut self,
        callback: Callback,
        commands: &mut Vec<Command>,
        actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion> {
        let proxy = self.runner.proxy();
        let Runner {
            handler,
            host,
            clipboard,
            ..
        } = self.runner;
        let Host {
            registry,
            capabilities,
            ..
        } = host;
        invoke_handler_callback(
            handler,
            CallbackEnvironment::new(registry, clipboard.as_mut(), capabilities, Some(proxy)),
            callback,
            commands,
            actions,
        )
    }

    fn apply_commands(
        &mut self,
        mut commands: Vec<NormalizedCommand>,
        origin: WorkOrigin,
        pump: &mut Pump<Callback>,
    ) -> Result<()> {
        commands.retain(|command| {
            accepts_work_target(
                origin,
                self.runner.host.registry(),
                command.command().target(),
            )
        });
        if commands.is_empty() && self.runner.staged_startup.is_none() {
            return Ok(());
        }
        let batch = self.runner.plan_pump_batch(commands)?;
        let mut callbacks = Vec::new();
        apply_batch_plan(batch, |plan| {
            let kind = plan.kind();
            let begin_close = (kind == CommandKind::Destroy)
                .then(|| plan.target().expect("destroy plans target one window"));
            let event_start = self.runner.host.events.len();
            self.runner.host.apply_plan(plan)?;
            self.runner
                .enqueue_recorded_follow_ups(kind, event_start, &mut callbacks);
            if let Some(id) = begin_close {
                pump.apply_begin_close(self, id)?;
            }
            Ok(())
        })?;
        for callback in callbacks {
            pump.enqueue_callback(callback);
        }
        Ok(())
    }

    fn apply_action(
        &mut self,
        action: Action,
        origin: WorkOrigin,
        pump: &mut Pump<Callback>,
    ) -> Result<()> {
        if !accepts_action_target(origin, self.runner.host.registry(), action.target())? {
            return Ok(());
        }
        match action {
            Action::CloseRequested(id) => {
                if self.runner.host.registry().contains(id) {
                    pump.enqueue_callback(Callback::Close(id));
                }
            }
            Action::Exit => {}
            Action::Batch(_) => unreachable!("the pump flattens nested action batches"),
            action => self.runner.host.draw.request(&action),
        }
        Ok(())
    }

    fn begin_close(&mut self, id: Id, pump: &mut Pump<Callback>) -> Result<()> {
        self.runner.host.begin_close(id)?;
        pump.enqueue_close_completion(id);
        Ok(())
    }

    fn complete_close(&mut self, id: Id, pump: &mut Pump<Callback>) -> Result<()> {
        if let Some(state) = self.runner.host.complete_close(id) {
            pump.enqueue_callback(Callback::Closed(state));
        }
        Ok(())
    }

    fn after_callback(&mut self, callback: Callback, pump: &mut Pump<Callback>) {
        enqueue_ready_after_created(callback, self.runner.host.registry(), pump);
    }

    fn close_ingress(&mut self) {
        self.runner.proxy_queue.close();
    }
}

struct HostPumpBackend<'a> {
    host: &'a mut Host,
}

impl PumpBackend<()> for HostPumpBackend<'_> {
    fn invoke_callback(
        &mut self,
        _callback: (),
        _commands: &mut Vec<Command>,
        _actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion> {
        Ok(CallbackCompletion::None)
    }

    fn apply_commands(
        &mut self,
        commands: Vec<NormalizedCommand>,
        _origin: WorkOrigin,
        pump: &mut Pump<()>,
    ) -> Result<()> {
        let batch =
            CommandPlanner::from_registry(self.host.capabilities.clone(), &self.host.registry)
                .plan_normalized_batch(commands)?;
        let mut command_index = 0;
        apply_batch_plan(batch, |plan| {
            #[cfg(test)]
            {
                if self.host.backend_failure_at == Some(command_index) {
                    self.host.backend_failure_at = None;
                    let mut error =
                        Error::new(ErrorCode::CommandFailed, "injected backend failure");
                    if let Some(id) = plan.target() {
                        error = error.with_id(id);
                    }
                    return Err(error);
                }
            }
            command_index += 1;
            let begin_close = (plan.kind() == CommandKind::Destroy)
                .then(|| plan.target().expect("destroy plans target one window"));
            self.host.apply_plan(plan)?;
            if let Some(id) = begin_close {
                pump.apply_begin_close(self, id)?;
            }
            Ok(())
        })
    }

    fn apply_action(
        &mut self,
        action: Action,
        origin: WorkOrigin,
        _pump: &mut Pump<()>,
    ) -> Result<()> {
        let _ = accepts_action_target(origin, &self.host.registry, action.target())?;
        Ok(())
    }

    fn begin_close(&mut self, id: Id, pump: &mut Pump<()>) -> Result<()> {
        self.host.begin_close(id)?;
        pump.enqueue_close_completion(id);
        Ok(())
    }

    fn complete_close(&mut self, id: Id, _pump: &mut Pump<()>) -> Result<()> {
        let _ = self.host.complete_close(id);
        Ok(())
    }
}

fn fake_state_from_request(id: Id, request: &WindowRequest) -> Result<WindowSnapshot> {
    let logical_size = request.inner_size().unwrap_or(Size {
        width: 800.0,
        height: 600.0,
    });
    let metrics = Metrics::from_physical_size(
        id,
        PhysicalSize {
            width: logical_size.width.round().max(0.0) as u32,
            height: logical_size.height.round().max(0.0) as u32,
        },
        1.0,
    )?
    .with_outer_geometry(request.position(), None)?;
    Ok(WindowSnapshot::from_seed(
        crate::descriptor::WindowSnapshotSeed {
            title: request.title().to_owned(),
            name: request.name().map(str::to_owned),
            metrics,
            focused: false,
            visible: Some(request.visible()),
            minimized: Some(false),
            maximized: false,
            occluded: Some(false),
            fullscreen: !matches!(request.fullscreen(), Fullscreen::None),
            theme: request.theme(),
            role: request.role().clone(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn host_closing_cleanup_cancels_its_scheduler_cursor_and_grab_state() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut host = Host::new();
        for name in ["closing", "surviving"] {
            host.apply(Command::Open {
                request: WindowRequest::builder(name).build(),
            })
            .expect("test window opens");
        }
        assert_eq!(host.window_id("closing"), Some(closing));
        assert_eq!(host.window_id("surviving"), Some(surviving));
        let deadline = Instant::now() + Duration::from_secs(1);

        host.draw.next.extend([closing, surviving]);
        host.draw.delayed.insert(closing, deadline);
        host.draw.delayed.insert(surviving, deadline);
        host.cursors.insert(closing, Cursor::Hidden);
        host.cursors.insert(surviving, Cursor::Hidden);
        for id in [closing, surviving] {
            host.cursor_grab.insert(id, CursorGrab::Locked);
        }
        assert_eq!(host.cursor_grab.get(&closing), Some(&CursorGrab::Locked));
        assert_eq!(host.cursor_grab.get(&surviving), Some(&CursorGrab::Locked));

        host.apply(Command::Destroy { id: closing })
            .expect("live window closes");

        assert!(!host.draw.next.contains(&closing));
        assert!(!host.draw.delayed.contains_key(&closing));
        assert!(!host.cursors.contains_key(&closing));
        assert!(!host.cursor_grab.contains_key(&closing));
        assert!(host.draw.next.contains(&surviving));
        assert_eq!(host.draw.delayed.get(&surviving), Some(&deadline));
        assert_eq!(host.cursors.get(&surviving), Some(&Cursor::Hidden));
        assert_eq!(host.cursor_grab.get(&surviving), Some(&CursorGrab::Locked));
        assert_eq!(host.take_ready_draws(deadline), vec![surviving]);
    }
}
