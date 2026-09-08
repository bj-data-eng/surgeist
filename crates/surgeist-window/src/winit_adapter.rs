#[cfg(test)]
use super::normalization::{normalize_command, normalize_commands};
#[cfg(test)]
use super::planning::{projected_snapshot_from_request, take_single_planned_command};
use super::{
    capability::CapabilityProvider,
    command::Action,
    descriptor::WindowSnapshotSeed,
    event::EventKind,
    lease::{LeaseWeak, ReleaseNotifier},
    loop_::PreparedLoop,
    normalization::NormalizedCommand,
    planning::{
        CommandPlanner, ResolvedImeRequest, SizeApplicationResult, StagedBatchPlan,
        accepts_action_target, accepts_work_target, apply_batch_plan, take_resolved_host_command,
    },
    pump::{
        Callback as BackendCallback, CallbackCompletion, CallbackEnvironment, Pump, PumpBackend,
        WorkOrigin, callback_for_event,
        enqueue_ready_after_created as enqueue_shared_ready_after_created,
        enqueue_resume_callbacks as enqueue_shared_resume_callbacks,
        enqueue_suspend_callbacks as enqueue_shared_suspend_callbacks, invoke_handler_callback,
    },
    registry::{InnerSizeBounds, LifecycleState},
    winit_mapping, *,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    time::{Duration, Instant},
};

#[cfg(all(test, feature = "accessibility"))]
use std::{cell::Cell, rc::Rc};

const REDRAW_RETRY: Duration = Duration::from_millis(16);

#[cfg(feature = "accessibility")]
#[derive(Debug)]
pub(crate) enum AccessibilitySetupError {
    EventProxyUnavailable,
}

#[cfg(feature = "accessibility")]
impl std::fmt::Display for AccessibilitySetupError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventProxyUnavailable => {
                formatter.write_str("native event loop proxy unavailable")
            }
        }
    }
}

#[cfg(feature = "accessibility")]
impl std::error::Error for AccessibilitySetupError {}

#[cfg(feature = "accessibility")]
pub(crate) fn native_open_request(request: &WindowRequest, install_adapter: bool) -> WindowRequest {
    let mut native_request = request.clone();
    if install_adapter {
        native_request.set_visible(false);
    }
    native_request
}

#[cfg(feature = "accessibility")]
pub(crate) fn prepare_accessible_open<P, A>(
    id: Id,
    install_adapter: bool,
    event_proxy: Option<P>,
    authored_visible: bool,
    adapter_factory: impl FnOnce(P) -> A,
    restore_visibility: impl FnOnce(),
) -> Result<Option<A>> {
    if !install_adapter {
        return Ok(None);
    }

    let event_proxy = event_proxy.ok_or_else(|| {
        Error::new(
            ErrorCode::AccessibilityAdapterFailed,
            "native event loop proxy unavailable for accessibility adapter",
        )
        .with_id(id)
        .with_source(AccessibilitySetupError::EventProxyUnavailable)
    })?;
    let adapter = adapter_factory(event_proxy);
    if authored_visible {
        restore_visibility();
    }
    Ok(Some(adapter))
}

#[cfg(all(test, feature = "accessibility"))]
struct MissingAccessibilityProxyForTest {
    id: Id,
    local_window_drops: Rc<Cell<usize>>,
}

#[cfg(all(test, feature = "accessibility"))]
struct TestLocalNativeWindow(Rc<Cell<usize>>);

#[cfg(all(test, feature = "accessibility"))]
impl Drop for TestLocalNativeWindow {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

struct WinitCapabilityProvider<'a> {
    event_loop: &'a winit::event_loop::ActiveEventLoop,
}

impl CapabilityProvider for WinitCapabilityProvider<'_> {
    fn resolve(&mut self) -> HostCapabilities {
        winit_capabilities(self.event_loop)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WinitTarget {
    MacOs,
    Windows,
    X11,
    Wayland,
    Web,
    Ios,
    Android,
    Orbital,
    Other,
}

fn winit_capabilities(event_loop: &winit::event_loop::ActiveEventLoop) -> HostCapabilities {
    winit_capabilities_for_target(winit_target(event_loop))
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn winit_target(event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    use winit::platform::{wayland::ActiveEventLoopExtWayland, x11::ActiveEventLoopExtX11};

    winit_target_from_active_identity(event_loop.is_x11(), event_loop.is_wayland())
}

#[cfg(target_os = "macos")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::MacOs
}

#[cfg(target_os = "windows")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Windows
}

#[cfg(target_family = "wasm")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Web
}

#[cfg(target_os = "ios")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Ios
}

#[cfg(target_os = "android")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Android
}

#[cfg(target_os = "redox")]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Orbital
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "macos",
    target_os = "windows",
    target_family = "wasm",
    target_os = "ios",
    target_os = "android",
    target_os = "redox",
)))]
fn winit_target(_event_loop: &winit::event_loop::ActiveEventLoop) -> WinitTarget {
    WinitTarget::Other
}

#[cfg(any(
    test,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn winit_target_from_active_identity(is_x11: bool, is_wayland: bool) -> WinitTarget {
    match (is_x11, is_wayland) {
        (true, false) => WinitTarget::X11,
        (false, true) => WinitTarget::Wayland,
        (false, false) | (true, true) => WinitTarget::Other,
    }
}

fn winit_capabilities_for_target(target: WinitTarget) -> HostCapabilities {
    let mut capabilities = HostCapabilities::builder();

    for role in [
        RoleKind::Root,
        RoleKind::Dialog,
        RoleKind::Tool,
        RoleKind::Popup,
    ] {
        capabilities = capabilities.role(role, winit_role_support(target, role));
    }
    for fullscreen in [
        FullscreenMode::None,
        FullscreenMode::Borderless,
        FullscreenMode::Exclusive,
    ] {
        capabilities =
            capabilities.fullscreen(fullscreen, winit_fullscreen_support(target, fullscreen));
    }
    for cursor in [
        CursorCapability::Icon,
        CursorCapability::Hidden,
        CursorCapability::Custom,
    ] {
        capabilities = capabilities.cursor(cursor, winit_cursor_support(target, cursor));
    }
    for operation in [
        WindowOperation::SetTitle,
        WindowOperation::SetOuterPosition,
        WindowOperation::SetVisible,
        WindowOperation::SetResizable,
        WindowOperation::SetControls,
        WindowOperation::SetDecorations,
        WindowOperation::InitialTransparency,
        WindowOperation::SetTransparent,
        WindowOperation::RequestInnerSize,
        WindowOperation::SetInnerSizeBounds,
        WindowOperation::SetLevel,
        WindowOperation::SetExplicitTheme,
        WindowOperation::ResetTheme,
        WindowOperation::RequestUserAttention,
        WindowOperation::RequestDraw,
        WindowOperation::Destroy,
    ] {
        capabilities = capabilities.window(operation, winit_window_support(target, operation));
    }
    for grab in [CursorGrab::None, CursorGrab::Confined, CursorGrab::Locked] {
        capabilities = capabilities.cursor_grab(grab, winit_cursor_grab_support(target, grab));
    }
    for ime in [
        ImeCapability::Enablement,
        ImeCapability::CursorArea,
        ImeCapability::Purpose(ImePurpose::Normal),
        ImeCapability::Purpose(ImePurpose::Password),
        ImeCapability::Purpose(ImePurpose::Number),
        ImeCapability::Purpose(ImePurpose::Email),
        ImeCapability::Purpose(ImePurpose::Url),
        ImeCapability::Purpose(ImePurpose::Terminal),
        ImeCapability::Hint(ImeHint::None),
        ImeCapability::Hint(ImeHint::Spellcheck),
        ImeCapability::Hint(ImeHint::NoSpellcheck),
        ImeCapability::SurroundingText,
    ] {
        capabilities = capabilities.ime(ime, winit_ime_support(target, ime));
    }

    capabilities
        .accessibility(winit_accessibility_support(target))
        .build()
}

fn target_is(target: WinitTarget, supported: &[WinitTarget]) -> bool {
    supported.contains(&target)
}

fn supported_when(supported: bool) -> CapabilitySupport {
    if supported {
        CapabilitySupport::Supported
    } else {
        CapabilitySupport::Unsupported
    }
}

fn winit_role_support(target: WinitTarget, role: RoleKind) -> CapabilitySupport {
    supported_when(role == RoleKind::Root && target != WinitTarget::Other)
}

fn winit_fullscreen_support(target: WinitTarget, fullscreen: FullscreenMode) -> CapabilitySupport {
    match fullscreen {
        FullscreenMode::None => CapabilitySupport::Supported,
        FullscreenMode::Borderless => supported_when(target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Ios,
            ],
        )),
        FullscreenMode::Exclusive => CapabilitySupport::Unsupported,
    }
}

fn winit_cursor_support(target: WinitTarget, cursor: CursorCapability) -> CapabilitySupport {
    match cursor {
        CursorCapability::Icon | CursorCapability::Hidden => supported_when(target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Web,
            ],
        )),
        CursorCapability::Custom => CapabilitySupport::Unsupported,
    }
}

fn winit_window_support(target: WinitTarget, operation: WindowOperation) -> CapabilitySupport {
    let supported = match operation {
        WindowOperation::SetTitle => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Web,
                WinitTarget::Orbital,
            ],
        ),
        WindowOperation::SetOuterPosition => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Web,
                WinitTarget::Ios,
                WinitTarget::Orbital,
            ],
        ),
        WindowOperation::SetVisible => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Ios,
                WinitTarget::Orbital,
            ],
        ),
        WindowOperation::SetResizable => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::Wayland,
                WinitTarget::Orbital,
            ],
        ),
        WindowOperation::SetControls => {
            target_is(target, &[WinitTarget::MacOs, WinitTarget::Windows])
        }
        WindowOperation::SetDecorations => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Orbital,
            ],
        ),
        WindowOperation::InitialTransparency => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
            ],
        ),
        WindowOperation::SetTransparent => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::Wayland,
            ],
        ),
        WindowOperation::RequestInnerSize => target != WinitTarget::Other,
        WindowOperation::SetInnerSizeBounds => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Web,
            ],
        ),
        WindowOperation::SetLevel | WindowOperation::SetExplicitTheme => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
            ],
        ),
        WindowOperation::ResetTheme => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::Wayland,
            ],
        ),
        WindowOperation::RequestUserAttention => target_is(
            target,
            &[WinitTarget::MacOs, WinitTarget::Windows, WinitTarget::X11],
        ),
        WindowOperation::RequestDraw | WindowOperation::Destroy => true,
    };
    supported_when(supported)
}

fn winit_cursor_grab_support(target: WinitTarget, grab: CursorGrab) -> CapabilitySupport {
    match grab {
        CursorGrab::None => CapabilitySupport::Supported,
        CursorGrab::Confined if target == WinitTarget::Wayland => {
            CapabilitySupport::RuntimeDependent
        }
        CursorGrab::Confined => {
            supported_when(target_is(target, &[WinitTarget::Windows, WinitTarget::X11]))
        }
        CursorGrab::Locked => CapabilitySupport::Unsupported,
    }
}

fn winit_ime_support(target: WinitTarget, capability: ImeCapability) -> CapabilitySupport {
    let supported = match capability {
        ImeCapability::Enablement | ImeCapability::Purpose(ImePurpose::Normal) => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Ios,
                WinitTarget::Android,
            ],
        ),
        ImeCapability::CursorArea => target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::Wayland,
            ],
        ),
        ImeCapability::Purpose(ImePurpose::Password | ImePurpose::Terminal) => {
            target == WinitTarget::Wayland
        }
        ImeCapability::Hint(ImeHint::None) => true,
        ImeCapability::Purpose(ImePurpose::Number | ImePurpose::Email | ImePurpose::Url)
        | ImeCapability::Hint(ImeHint::Spellcheck | ImeHint::NoSpellcheck)
        | ImeCapability::SurroundingText => false,
    };
    supported_when(supported)
}

fn winit_accessibility_support(target: WinitTarget) -> CapabilitySupport {
    #[cfg(feature = "accessibility")]
    {
        supported_when(target_is(
            target,
            &[
                WinitTarget::MacOs,
                WinitTarget::Windows,
                WinitTarget::X11,
                WinitTarget::Wayland,
                WinitTarget::Ios,
            ],
        ))
    }
    #[cfg(not(feature = "accessibility"))]
    {
        let _ = target;
        CapabilitySupport::Unsupported
    }
}

fn remove_routed_window<K: Eq + std::hash::Hash>(routes: &mut HashMap<K, Id>, id: Id) {
    routes.retain(|_, routed| *routed != id);
}

fn remove_window_state<T>(state: &mut HashMap<Id, T>, id: Id) {
    state.remove(&id);
}

fn cleanup_native_window<E>(
    hide: impl FnOnce(),
    release_cursor_grab: impl FnOnce() -> std::result::Result<(), E>,
) {
    hide();
    let _ = release_cursor_grab();
}

struct ClosingWindow<R> {
    state: WindowSnapshot,
    lease: Option<LeaseWeak<R>>,
}

/// Private ownership retained after a native window leaves live lookup.
///
/// Every entry owns one terminal snapshot and, for a real native resource, one
/// weak lease only. The weak lease can invalidate external wrappers but never
/// extends their resource lifetime.
struct NativeOwnership<R> {
    closing: HashMap<Id, ClosingWindow<R>>,
    terminal_leases: Vec<LeaseWeak<R>>,
}

impl<R> NativeOwnership<R> {
    fn new() -> Self {
        Self {
            closing: HashMap::new(),
            terminal_leases: Vec::new(),
        }
    }

    fn begin_close(&mut self, id: Id, state: WindowSnapshot, lease: Option<LeaseWeak<R>>) -> bool {
        if self.closing.contains_key(&id) {
            return false;
        }
        self.closing.insert(id, ClosingWindow { state, lease });
        true
    }

    fn contains(&self, id: Id) -> bool {
        self.closing.contains_key(&id)
    }

    fn is_empty(&self) -> bool {
        self.closing.is_empty()
    }

    fn complete_close(&mut self, id: Id) -> Option<WindowSnapshot> {
        let entry = self.closing.remove(&id)?;
        if let Some(lease) = entry.lease {
            self.remember_terminal_lease(lease);
        }
        Some(entry.state)
    }

    fn mark_destroyed(&self, id: Id) {
        let Some(lease) = self.closing.get(&id).and_then(|entry| entry.lease.as_ref()) else {
            return;
        };
        lease.mark_destroyed();
    }

    fn mark_loop_exited(&self) {
        for entry in self.closing.values() {
            if let Some(lease) = entry.lease.as_ref() {
                lease.mark_loop_exited();
            }
        }
        for lease in &self.terminal_leases {
            lease.mark_loop_exited();
        }
    }

    fn remember_terminal_lease(&mut self, lease: LeaseWeak<R>) {
        self.terminal_leases.retain(|lease| !lease.is_released());
        if !lease.is_released() {
            self.terminal_leases.push(lease);
        }
    }

    fn has_outstanding_leases(&self) -> bool {
        self.terminal_leases
            .iter()
            .any(|lease| !lease.is_released())
            || self
                .closing
                .values()
                .filter_map(|entry| entry.lease.as_ref())
                .any(|lease| !lease.is_released())
    }

    fn closing_ids(&self) -> Vec<Id> {
        self.closing.keys().copied().collect()
    }
}

fn complete_native_close<R>(
    registry: &mut Registry,
    ownership: &mut NativeOwnership<R>,
    id: Id,
) -> Option<WindowSnapshot> {
    if !ownership.contains(id) || !registry.complete_close(id) {
        return None;
    }
    ownership.complete_close(id)
}

fn terminal_result(
    pump_result: Result<()>,
    native_result: Result<()>,
    has_outstanding_leases: bool,
) -> Result<()> {
    pump_result?;
    native_result?;
    if has_outstanding_leases {
        return Err(Error::new(
            ErrorCode::OutstandingHandle,
            "external native handle lease remains after event-loop teardown",
        ));
    }
    Ok(())
}

pub(crate) struct WinitRunner<H> {
    handler: H,
    registry: Registry,
    draw: DrawScheduler,
    capabilities: Option<HostCapabilities>,
    clipboard: Box<dyn Clipboard>,
    #[cfg(test)]
    pub(crate) commands: Vec<Command>,
    #[cfg(test)]
    applied_commands: Vec<Command>,
    pub(crate) startup: Vec<NormalizedCommand>,
    staged_startup: Option<StagedBatchPlan>,
    windows: HashMap<winit::window::WindowId, Id>,
    closing_windows: HashMap<winit::window::WindowId, Id>,
    native_ownership: NativeOwnership<winit::window::Window>,
    pending_releases: HashSet<Id>,
    hovered_files: HashMap<Id, Vec<PathBuf>>,
    modifiers: ModifierState,
    pub(crate) pointer_positions: HashMap<PointerPositionKey, Point>,
    cursor_state: HashMap<Id, Cursor>,
    cursor_grab: HashMap<Id, CursorGrab>,
    pending_draws: HashSet<Id>,
    pending_size_requests: HashSet<Id>,
    immediate_size_results: HashMap<Id, VecDeque<PhysicalSize>>,
    #[cfg(feature = "accessibility")]
    accessibility: HashMap<Id, accesskit_winit::Adapter>,
    #[cfg(all(test, feature = "accessibility"))]
    accessibility_updates_for_test: Vec<(Id, accesskit::TreeUpdate)>,
    #[cfg(all(test, feature = "accessibility"))]
    missing_accessibility_proxy_for_test: Option<MissingAccessibilityProxyForTest>,
    pub(crate) proxy: Option<Proxy>,
    startup_applied: bool,
    teardown_complete: bool,
    pump: Pump<BackendCallback>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeTransitionRoute {
    Event,
    Input,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PointerPositionKey {
    window: Id,
    pointer: Option<u64>,
}

impl PointerPositionKey {
    pub(crate) const fn mouse(window: Id) -> Self {
        Self {
            window,
            pointer: None,
        }
    }

    pub(crate) const fn touch(window: Id, pointer: u64) -> Self {
        Self {
            window,
            pointer: Some(pointer),
        }
    }
}

impl<H> WinitRunner<H> {
    pub(crate) fn from_prepared(prepared: PreparedLoop<H>) -> Self {
        let (window_loop, startup) = prepared.into_loop_and_startup();

        Self {
            handler: window_loop.handler,
            registry: window_loop.registry,
            draw: window_loop.draw,
            capabilities: None,
            clipboard: window_loop.clipboard,
            #[cfg(test)]
            commands: window_loop.commands,
            #[cfg(test)]
            applied_commands: Vec::new(),
            startup,
            staged_startup: None,
            windows: HashMap::new(),
            closing_windows: HashMap::new(),
            native_ownership: NativeOwnership::new(),
            pending_releases: HashSet::new(),
            hovered_files: HashMap::new(),
            modifiers: ModifierState::default(),
            pointer_positions: HashMap::new(),
            cursor_state: HashMap::new(),
            cursor_grab: HashMap::new(),
            pending_draws: HashSet::new(),
            pending_size_requests: HashSet::new(),
            immediate_size_results: HashMap::new(),
            #[cfg(feature = "accessibility")]
            accessibility: HashMap::new(),
            #[cfg(all(test, feature = "accessibility"))]
            accessibility_updates_for_test: Vec::new(),
            #[cfg(all(test, feature = "accessibility"))]
            missing_accessibility_proxy_for_test: None,
            proxy: None,
            startup_applied: false,
            teardown_complete: false,
            pump: Pump::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_loop(window_loop: Loop<H>) -> Self {
        let prepared = window_loop
            .prepare()
            .expect("test runner startup must satisfy intrinsic preparation");
        Self::from_prepared(prepared)
    }

    #[must_use]
    pub(crate) fn capabilities(&self) -> &HostCapabilities {
        self.capabilities
            .as_ref()
            .expect("winit runner capabilities are resolved before callback or command planning")
    }

    fn resolve_capabilities_once(&mut self, provider: &mut impl CapabilityProvider) {
        if self.capabilities.is_none() {
            self.capabilities = Some(provider.resolve());
        }
    }

    fn callbacks_are_enabled(&self) -> bool {
        self.capabilities.is_some() && self.pump.is_running()
    }

    #[cfg(test)]
    pub(crate) fn resolve_capabilities_for_test(&mut self, capabilities: HostCapabilities) {
        self.capabilities.get_or_insert(capabilities);
    }

    #[cfg(test)]
    pub(crate) fn plan_command_for_test(&self, command: Command) -> Result<HostCommandPlan> {
        let batch = CommandPlanner::from_registry(self.capabilities().clone(), &self.registry)
            .plan_normalized_batch(vec![normalize_command(command)?])?;
        Ok(take_single_planned_command(batch))
    }

    #[cfg(test)]
    pub(crate) fn apply_size_request_result_for_test(
        &mut self,
        id: Id,
        result: Option<PhysicalSize>,
    ) -> Result<()>
    where
        H: Handler,
    {
        let mut callbacks = Vec::new();
        self.apply_size_request_result(
            id,
            result.map_or(
                SizeApplicationResult::Deferred,
                SizeApplicationResult::Immediate,
            ),
            &mut callbacks,
        )?;
        for callback in callbacks {
            self.enqueue_callback(callback);
        }
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()
    }

    #[cfg(test)]
    pub(crate) fn observe_resized_for_test(&mut self, id: Id, size: PhysicalSize) -> Result<()>
    where
        H: Handler,
    {
        if self.observe_resized(id, size)? {
            self.enqueue_callback(BackendCallback::Resize(id));
        }
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()
    }
}

impl<H: Handler> WinitRunner<H> {
    fn stage_first_resume(&mut self) -> Result<bool> {
        if self.startup_applied {
            return Ok(false);
        }

        self.startup_applied = true;
        self.staged_startup = Some(
            CommandPlanner::from_registry(self.capabilities().clone(), &self.registry)
                .stage_normalized_batch(std::mem::take(&mut self.startup))?,
        );
        Ok(true)
    }

    #[cfg(test)]
    fn resume_with_capability_provider(
        &mut self,
        provider: &mut impl CapabilityProvider,
    ) -> Result<Action> {
        self.resolve_capabilities_once(provider);
        match self.stage_first_resume() {
            Ok(true) => self.enqueue_callback(BackendCallback::Resume),
            Ok(false) => self.enqueue_resume_callbacks(),
            Err(error) => self.fail_terminal(error),
        }
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()?;
        Ok(Action::Wait)
    }

    #[cfg(test)]
    pub(crate) fn resume_with_capability_provider_for_test(
        &mut self,
        provider: &mut impl CapabilityProvider,
    ) -> Result<Action> {
        self.resume_with_capability_provider(provider)
    }

    #[cfg(test)]
    pub(crate) fn suspend_for_test(&mut self) -> Result<Action> {
        self.enqueue_suspend_callbacks();
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()?;
        Ok(Action::Wait)
    }

    #[cfg(test)]
    pub(crate) fn idle_for_test(&mut self) -> Result<Option<Action>> {
        if !self.pump.is_running() {
            return Ok(None);
        }
        if !self.handler.wants_idle() {
            return Ok(None);
        }
        if !self.callbacks_are_enabled() {
            return Ok(Some(Action::Wait));
        }

        self.enqueue_callback(BackendCallback::Idle);
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()?;
        Ok(Some(Action::Wait))
    }

    #[cfg(test)]
    pub(crate) const fn startup_is_staged_for_test(&self) -> bool {
        self.startup_applied
    }

    fn apply_action_from_pump(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        action: Action,
        pump: &mut Pump<BackendCallback>,
    ) -> Result<()> {
        match action {
            Action::Exit => event_loop.exit(),
            Action::DrawNow(id) => {
                self.draw.next.remove(&id);
                self.draw.delayed.remove(&id);
                if let Ok(handle) = self.handle(id) {
                    self.pending_draws.insert(id);
                    let _ = handle.request_draw();
                }
            }
            Action::Batch(_) => unreachable!("the pump flattens nested action batches"),
            Action::CloseRequested(id) => {
                if self.registry.contains(id) {
                    pump.enqueue_callback(BackendCallback::Close(id));
                }
            }
            other => self.draw.request(&other),
        }
        Ok(())
    }

    fn request_ready_draws(&mut self) {
        for id in self.draw.take_ready(Instant::now()) {
            self.pending_draws.insert(id);
        }
        self.request_pending_draws();
    }

    fn request_pending_draws(&mut self) {
        for id in self.pending_draws.clone() {
            if !self.can_request_draw(id) {
                continue;
            }
            if let Ok(handle) = self.handle(id) {
                let _ = handle.request_draw();
            }
        }
    }

    fn can_request_draw(&self, id: Id) -> bool {
        self.registry
            .get(id)
            .map(|instance| {
                instance.instance.state.is_visible() && !instance.instance.state.is_occluded()
            })
            .unwrap_or(false)
    }

    fn control_flow(&self) -> winit::event_loop::ControlFlow {
        if self
            .pending_draws
            .iter()
            .any(|id| self.can_request_draw(*id))
        {
            return winit::event_loop::ControlFlow::WaitUntil(Instant::now() + REDRAW_RETRY);
        }
        winit_mapping::control_flow_from_draw_scheduler(&self.draw)
    }

    fn invoke_callback(
        &mut self,
        callback: BackendCallback,
        commands: &mut Vec<Command>,
        actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion> {
        let capabilities = self
            .capabilities
            .as_ref()
            .expect("winit runner capabilities are resolved before callback delivery");
        let proxy = self.proxy.clone();
        invoke_handler_callback(
            &mut self.handler,
            CallbackEnvironment::new(
                &mut self.registry,
                self.clipboard.as_mut(),
                capabilities,
                proxy,
            ),
            callback,
            commands,
            actions,
        )
    }

    fn enqueue_callback(&mut self, callback: BackendCallback) {
        if !self.callbacks_are_enabled() {
            return;
        }

        self.pump.enqueue_callback(callback);
    }

    fn enqueue_resume_callbacks(&mut self) {
        if !self.callbacks_are_enabled() {
            return;
        }
        enqueue_shared_resume_callbacks(&self.registry, &mut self.pump);
    }

    fn enqueue_suspend_callbacks(&mut self) {
        if !self.callbacks_are_enabled() {
            return;
        }
        enqueue_shared_suspend_callbacks(&self.registry, &mut self.pump);
    }

    fn dispatch_callback(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        callback: BackendCallback,
    ) {
        self.enqueue_callback(callback);
        self.drain_pump(event_loop);
    }

    #[cfg(test)]
    pub(crate) fn deliver_event_to_pump_for_test(&mut self, event: EventKind) {
        self.enqueue_callback(BackendCallback::Event(event));
    }

    #[cfg(test)]
    pub(crate) fn drain_pump_for_test(&mut self) {
        let applied = std::cell::RefCell::new(Vec::new());
        let mut pump = std::mem::take(&mut self.pump);
        {
            let mut backend = TestPumpBackend {
                runner: self,
                applied: &applied,
            };
            pump.drain(&mut backend);
        }
        self.pump = pump;
    }

    #[cfg(test)]
    fn ensure_test_pump_has_not_failed(&self) -> Result<()> {
        (!self.pump.has_failed()).then_some(()).ok_or_else(|| {
            Error::new(
                ErrorCode::CommandFailed,
                "native test pump reached a terminal state",
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn deliver_created_then_ready_for_test(
        &mut self,
        state: WindowSnapshot,
    ) -> Result<()> {
        self.enqueue_callback(BackendCallback::Event(EventKind::Created(state)));
        self.drain_pump_for_test();
        self.ensure_test_pump_has_not_failed()
    }

    #[cfg(test)]
    pub(crate) fn registry_contains_for_test(&self, id: Id) -> bool {
        self.registry.contains(id)
    }

    #[cfg(test)]
    pub(crate) fn registry_snapshot_for_test(&self, id: Id) -> Option<&WindowSnapshot> {
        self.registry.get(id).map(|window| window.instance.state())
    }

    #[cfg(test)]
    pub(crate) fn inner_size_bounds_for_test(&self, id: Id) -> Option<InnerSizeBounds> {
        self.registry
            .get(id)
            .map(|window| window.instance.inner_size_bounds())
    }

    #[cfg(test)]
    pub(crate) fn applied_commands_for_test(&self) -> &[Command] {
        &self.applied_commands
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn terminal_result_for_test(&self) -> Option<std::result::Result<(), &Error>> {
        self.pump.result()
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn take_ready_draws_for_test(&mut self, now: Instant) -> Vec<Id> {
        self.draw.take_ready(now)
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn lifecycle_state_for_test(&self, id: Id) -> Option<LifecycleState> {
        self.registry.lifecycle_state(id)
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn accessibility_open_state_for_test(
        &self,
        id: Id,
    ) -> (
        bool,
        bool,
        bool,
        Option<super::planning::AccessibilityPhase>,
    ) {
        (
            self.registry.contains(id),
            self.windows.values().any(|routed| *routed == id),
            self.accessibility.contains_key(&id),
            self.registry.accessibility_phase(id),
        )
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn accessibility_updates_for_test(&self) -> &[(Id, accesskit::TreeUpdate)] {
        &self.accessibility_updates_for_test
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn route_native_accessibility_event_for_test(
        &mut self,
        id: Id,
        event: accesskit_winit::WindowEvent,
    ) where
        H: Handler,
    {
        self.route_accessibility_event(id, accessibility_event_from_winit(id, event));
        self.drain_pump_for_test();
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn simulate_missing_accessibility_proxy_for_test(
        &mut self,
        id: Id,
    ) -> Rc<Cell<usize>> {
        let local_window_drops = Rc::new(Cell::new(0));
        self.missing_accessibility_proxy_for_test = Some(MissingAccessibilityProxyForTest {
            id,
            local_window_drops: local_window_drops.clone(),
        });
        local_window_drops
    }

    #[cfg(test)]
    pub(crate) fn allocator_high_water_for_test(&self) -> u64 {
        self.registry.planning_projection().next
    }

    #[cfg(test)]
    fn apply_action_for_test(&mut self, action: Action, pump: &mut Pump<BackendCallback>) {
        match action {
            Action::CloseRequested(id) => {
                if self.registry.contains(id) {
                    pump.enqueue_callback(BackendCallback::Close(id));
                }
            }
            Action::Batch(actions) => {
                for action in actions {
                    self.apply_action_for_test(action, pump);
                }
            }
            other => self.draw.request(&other),
        }
    }

    fn drain_pump(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut pump = std::mem::take(&mut self.pump);
        {
            let mut backend = WinitPumpBackend {
                runner: self,
                event_loop,
            };
            pump.drain(&mut backend);
        }
        let terminal = !pump.is_running();
        self.pump = pump;
        if terminal {
            event_loop.exit();
        } else {
            self.request_ready_draws();
        }
    }

    fn deliver_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: Id,
        event: EventKind,
    ) {
        debug_assert_eq!(event.id(), id);
        self.dispatch_callback(event_loop, BackendCallback::Event(event));
    }

    fn apply_native_transition(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: Id,
        transition: NativeEventTransition,
    ) {
        let event = match self.apply_native_transition_state(id, transition) {
            Ok(event) => event,
            Err(error) => {
                self.fail_terminal(error);
                self.drain_pump(event_loop);
                return;
            }
        };
        if let Some(event) = event {
            self.dispatch_callback(event_loop, callback_for_event(event));
        }
    }

    fn apply_native_transition_state(
        &mut self,
        id: Id,
        transition: NativeEventTransition,
    ) -> Result<Option<EventKind>> {
        let instance = self
            .registry
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        transition.apply(instance.state_mut()).map(Some)
    }

    #[cfg(test)]
    fn apply_metrics_transition_for_test(
        &mut self,
        id: Id,
        width: u32,
        height: u32,
        event: MetricsEvent,
    ) -> Result<()> {
        let result = self
            .metrics_transition(id, width, height, event)
            .and_then(|transition| self.apply_native_transition_patch_state(transition));
        self.retain_terminal_result(result)
    }

    #[cfg(test)]
    fn apply_scale_factor_transition_for_test(&mut self, id: Id, scale_factor: f64) -> Result<()> {
        let result = self
            .scale_factor_transition(id, scale_factor)
            .and_then(|transition| self.apply_native_transition_patch_state(transition));
        self.retain_terminal_result(result)
    }

    fn deliver_resize(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, id: Id) {
        self.dispatch_callback(event_loop, BackendCallback::Resize(id));
    }

    fn deliver_input(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        input: InputEvent,
    ) {
        self.dispatch_callback(event_loop, BackendCallback::Input(input));
    }

    fn deliver_close(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, id: Id) {
        if self.registry.contains(id) {
            self.dispatch_callback(event_loop, BackendCallback::Close(id));
        }
    }

    #[cfg(test)]
    pub(crate) fn apply_proxy_command_for_test(&mut self, command: NormalizedCommand) {
        self.pump.enqueue_queued_normalized_command(command);
        self.drain_pump_for_test();
    }

    #[cfg(test)]
    pub(crate) fn enqueue_open_commands_for_test(&mut self) -> Result<()> {
        let commands = normalize_commands(std::mem::take(&mut self.commands))?;
        self.pump.enqueue_normalized_commands(commands);
        Ok(())
    }

    #[cfg(all(test, feature = "accessibility"))]
    pub(crate) fn enqueue_action_to_pump_for_test(&mut self, action: Action) {
        self.pump.enqueue_queued_action(action);
    }

    #[cfg(test)]
    fn apply_open_commands_for_test(
        &mut self,
        applied: &std::cell::RefCell<Vec<Id>>,
    ) -> Result<()> {
        self.enqueue_open_commands_for_test()?;
        let mut pump = std::mem::take(&mut self.pump);
        {
            let mut backend = TestPumpBackend {
                runner: self,
                applied,
            };
            pump.drain(&mut backend);
        }
        let running = pump.is_running();
        self.pump = pump;
        if running {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::CommandFailed,
                "native open test pump reached a terminal state",
            ))
        }
    }

    fn plan_pump_batch(
        &mut self,
        commands: Vec<NormalizedCommand>,
    ) -> Result<super::planning::BatchPlan> {
        if let Some(staged_startup) = self.staged_startup.take() {
            staged_startup.finish_normalized_batch(commands)
        } else {
            CommandPlanner::from_registry(self.capabilities().clone(), &self.registry)
                .plan_normalized_batch(commands)
        }
    }

    fn enqueue_ready_after_created(
        &mut self,
        callback: BackendCallback,
        pump: &mut Pump<BackendCallback>,
    ) {
        enqueue_shared_ready_after_created(callback, &self.registry, pump);
    }

    fn apply_host_command_with_follow_ups(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        plan: HostCommandPlan,
        callbacks: &mut Vec<BackendCallback>,
    ) -> Result<()> {
        let resolved = take_resolved_host_command(plan);
        let ime_request = resolved.ime_request;
        #[cfg(feature = "accessibility")]
        let accessibility_update = resolved.accessibility_update;
        #[cfg(feature = "accessibility")]
        let accessibility_open_phase = resolved.accessibility_open_phase;
        match resolved.command.into_command() {
            Command::Open { request } => {
                #[cfg(feature = "accessibility")]
                let install_adapter =
                    self.capabilities().accessibility() == CapabilitySupport::Supported;
                #[cfg(feature = "accessibility")]
                let native_request = native_open_request(&request, install_adapter);
                #[cfg(not(feature = "accessibility"))]
                let native_request = request.clone();
                let id = resolved
                    .open_id
                    .expect("preflighted open commands carry a planned identity");
                let proxy = self
                    .proxy
                    .clone()
                    .expect("native runner proxy is initialized before window creation");
                let windows = &mut self.windows;
                #[cfg(feature = "accessibility")]
                let accessibility = &mut self.accessibility;
                let state =
                    with_planned_open_reservation(&mut self.registry, id, |id, registry| {
                        let attributes =
                            winit_mapping::window_attributes_from_request(&native_request)?;
                        let window = event_loop.create_window(attributes).map_err(|source| {
                            Error::new(ErrorCode::WindowCreateFailed, "failed to create window")
                                .with_source(source)
                        })?;
                        let state = state_from_winit(id, &request, &window)?;
                        let window_id = window.id();
                        #[cfg(feature = "accessibility")]
                        let adapter = prepare_accessible_open(
                            id,
                            install_adapter,
                            proxy.winit_proxy(),
                            request.visible(),
                            |event_proxy| {
                                accesskit_winit::Adapter::with_event_loop_proxy(
                                    event_loop,
                                    &window,
                                    event_proxy,
                                )
                            },
                            || window.set_visible(true),
                        )?;
                        let handle =
                            Handle::from_winit(id, window, ReleaseNotifier::new(proxy.clone()));
                        let mut instance = Instance::with_handle(state.clone(), handle);
                        instance.set_inner_size_bounds(InnerSizeBounds::new(
                            request.min_inner_size(),
                            request.max_inner_size(),
                        ));
                        registry.insert(instance)?;
                        windows.insert(window_id, id);
                        #[cfg(feature = "accessibility")]
                        if let Some(adapter) = adapter {
                            accessibility.insert(id, adapter);
                        }
                        #[cfg(feature = "accessibility")]
                        if let Some(phase) = accessibility_open_phase {
                            registry.insert_accessibility_phase(id, phase);
                        }
                        Ok(state)
                    })?;
                callbacks.push(BackendCallback::Event(EventKind::Created(state)));
            }
            Command::SetTitle { id, title } => {
                let handle = self.handle(id)?;
                handle.with_winit(|window| window.set_title(&title))?;
                self.apply_patch(WindowStatePatch::title(id, title))?;
            }
            Command::SetPosition { id, position } => {
                self.handle(id)?.with_winit(|window| {
                    window.set_outer_position(winit::dpi::LogicalPosition::new(
                        position.x, position.y,
                    ));
                })?;
            }
            Command::SetVisible { id, visible } => {
                let handle = self.handle(id)?;
                handle.with_winit(|window| window.set_visible(visible))?;
                self.apply_patch(WindowStatePatch::visible(id, visible))?;
                if visible {
                    self.request_pending_draws();
                }
            }
            Command::SetResizable { id, resizable } => {
                self.handle(id)?
                    .with_winit(|window| window.set_resizable(resizable))?;
            }
            Command::SetControls { id, controls } => {
                self.handle(id)?
                    .with_winit(|window| window.set_enabled_buttons(controls.into()))?;
            }
            Command::SetDecorations { id, decorations } => {
                self.handle(id)?
                    .with_winit(|window| window.set_decorations(decorations))?;
            }
            Command::SetTransparent { id, transparent } => {
                self.handle(id)?
                    .with_winit(|window| window.set_transparent(transparent))?;
            }
            Command::SetInnerSize { id, size } => {
                let result = self.handle(id)?.with_winit(|window| {
                    window.request_inner_size(winit::dpi::LogicalSize::new(size.width, size.height))
                })?;
                self.apply_size_request_result(
                    id,
                    size_application_result_from_winit(result),
                    callbacks,
                )?;
            }
            Command::SetMinInnerSize { id, size } => {
                self.handle(id)?.with_winit(|window| {
                    window.set_min_inner_size(
                        size.map(|size| winit::dpi::LogicalSize::new(size.width, size.height)),
                    );
                })?;
                self.apply_inner_size_bounds(
                    id,
                    InnerSizeBounds::new(size, self.inner_size_bounds(id)?.maximum),
                    resolved.resolved_size,
                    callbacks,
                )?;
            }
            Command::SetMaxInnerSize { id, size } => {
                self.handle(id)?.with_winit(|window| {
                    window.set_max_inner_size(
                        size.map(|size| winit::dpi::LogicalSize::new(size.width, size.height)),
                    );
                })?;
                self.apply_inner_size_bounds(
                    id,
                    InnerSizeBounds::new(self.inner_size_bounds(id)?.minimum, size),
                    resolved.resolved_size,
                    callbacks,
                )?;
            }
            Command::SetFullscreen { id, fullscreen } => {
                let fullscreen = match fullscreen {
                    Fullscreen::None => None,
                    Fullscreen::Borderless => Some(winit::window::Fullscreen::Borderless(None)),
                    Fullscreen::Exclusive => {
                        unreachable!("exclusive fullscreen is rejected during command planning")
                    }
                };
                let is_fullscreen = fullscreen.is_some();
                self.handle(id)?
                    .with_winit(|window| window.set_fullscreen(fullscreen))?;
                self.apply_patch(WindowStatePatch::Fullscreen {
                    id,
                    fullscreen: is_fullscreen,
                })?;
            }
            Command::SetLevel { id, level } => {
                self.handle(id)?
                    .with_winit(|window| window.set_window_level(level.into()))?;
            }
            Command::SetTheme { id, theme } => {
                self.handle(id)?
                    .with_winit(|window| window.set_theme(theme.map(Into::into)))?;
                if let Some(event) = self.apply_patch(WindowStatePatch::Theme { id, theme })? {
                    callbacks.push(BackendCallback::Event(event));
                }
            }
            Command::SetCursor { id, cursor } => {
                if self.cursor_state.get(&id) == Some(&cursor) {
                    return Ok(());
                }
                let window = self.handle(id)?;
                match cursor {
                    Cursor::Icon(icon) => {
                        window.with_winit(|native| {
                            native.set_cursor_visible(true);
                            native.set_cursor(icon);
                        })?;
                        self.cursor_state.insert(id, Cursor::Icon(icon));
                    }
                    Cursor::Hidden => {
                        window.with_winit(|native| native.set_cursor_visible(false))?;
                        self.cursor_state.insert(id, Cursor::Hidden);
                    }
                    Cursor::Custom(_) => {
                        unreachable!("custom cursors are rejected during command planning")
                    }
                }
            }
            Command::SetCursorGrab { id, grab } => {
                let mode = match grab {
                    CursorGrab::None => winit::window::CursorGrabMode::None,
                    CursorGrab::Confined => winit::window::CursorGrabMode::Confined,
                    CursorGrab::Locked => winit::window::CursorGrabMode::Locked,
                };
                self.handle(id)?
                    .with_winit(|window| window.set_cursor_grab(mode))?
                    .map_err(|source| cursor_grab_failed(id, source))?;
                if matches!(grab, CursorGrab::None) {
                    self.cursor_grab.remove(&id);
                } else {
                    self.cursor_grab.insert(id, grab);
                }
            }
            Command::SetIme { id, .. } => {
                self.apply_ime(
                    id,
                    ime_request.expect("planned IME command carries a resolved request"),
                )?;
            }
            #[cfg(feature = "accessibility")]
            Command::UpdateAccessibility { id, update } => {
                let resolved = accessibility_update
                    .expect("preflighted accessibility commands carry a resolved phase");
                let adapter = self.accessibility.get_mut(&id).ok_or_else(|| {
                    Error::new(
                        ErrorCode::AccessibilityAdapterFailed,
                        "native accessibility adapter is unavailable",
                    )
                    .with_id(id)
                })?;
                apply_accessibility_update(
                    &mut self.registry,
                    id,
                    update,
                    resolved,
                    adapter,
                    |adapter, update| adapter.update_if_active(|| update),
                );
            }
            Command::RequestUserAttention { id } => {
                self.handle(id)?.with_winit(|window| {
                    window.request_user_attention(Some(
                        winit::window::UserAttentionType::Informational,
                    ));
                })?;
            }
            Command::RequestDraw { id } => {
                self.pending_draws.insert(id);
                self.handle(id)?.request_draw()?;
            }
            Command::Destroy { .. } => {}
        }
        Ok(())
    }

    fn handle(&self, id: Id) -> Result<Handle> {
        self.registry
            .get(id)
            .ok_or_else(|| Error::new(ErrorCode::HandleUnavailable, "unknown window").with_id(id))?
            .handle()
    }

    fn begin_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        let Some(instance) = self.registry.begin_close(id) else {
            return Ok(());
        };

        self.cleanup_window_state(id, &instance, true);
        let lease = instance.handle.as_ref().map(Handle::downgrade);
        let has_native_lease = lease.is_some();
        debug_assert!(
            self.native_ownership
                .begin_close(id, instance.state().clone(), lease),
            "a live generation enters closing ownership exactly once"
        );
        let release_arrived_early = self.pending_releases.remove(&id);
        drop(instance);

        if !has_native_lease || release_arrived_early {
            pump.enqueue_close_completion(id);
        }
        Ok(())
    }

    fn complete_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        if let Some(state) =
            complete_native_close(&mut self.registry, &mut self.native_ownership, id)
        {
            remove_routed_window(&mut self.closing_windows, id);
            pump.enqueue_callback(BackendCallback::Closed(state));
        }
        Ok(())
    }

    fn handle_released(&mut self, id: Id) {
        if self.native_ownership.contains(id) {
            self.pump.enqueue_close_completion(id);
        } else if self.registry.lifecycle_state(id) == Some(LifecycleState::Closing) {
            self.pending_releases.insert(id);
        }
    }

    fn mark_native_destroyed(&mut self, id: Id) {
        if let Some(instance) = self.registry.get(id) {
            if let Some(handle) = instance.instance.handle.as_ref() {
                handle.mark_destroyed();
            }
            return;
        }
        self.native_ownership.mark_destroyed(id);
    }

    fn cleanup_window_state(&mut self, id: Id, instance: &Instance, retain_closing_route: bool) {
        self.draw.cancel(id);
        self.pending_draws.remove(&id);
        self.pending_size_requests.remove(&id);
        self.immediate_size_results.remove(&id);
        self.pointer_positions.retain(|key, _| key.window != id);
        remove_window_state(&mut self.hovered_files, id);
        remove_window_state(&mut self.cursor_state, id);
        remove_window_state(&mut self.cursor_grab, id);
        if retain_closing_route {
            let routes = self
                .windows
                .iter()
                .filter_map(|(window_id, routed)| (*routed == id).then_some(*window_id))
                .collect::<Vec<_>>();
            for window_id in routes {
                self.windows.remove(&window_id);
                self.closing_windows.insert(window_id, id);
            }
        } else {
            remove_routed_window(&mut self.windows, id);
            remove_routed_window(&mut self.closing_windows, id);
        }
        #[cfg(feature = "accessibility")]
        remove_window_state(&mut self.accessibility, id);

        if let Some(handle) = instance.handle.as_ref() {
            let _ = handle.with_winit(|window| {
                cleanup_native_window(
                    || window.set_visible(false),
                    || window.set_cursor_grab(winit::window::CursorGrabMode::None),
                );
            });
        }
    }

    fn close_ingress(&mut self) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.close_ingress();
        }
    }

    fn fail_terminal(&mut self, error: Error) {
        if self.pump.is_running() {
            self.close_ingress();
            self.teardown();
            self.pump.fail(error);
        }
    }

    fn teardown(&mut self) {
        if self.teardown_complete {
            return;
        }
        self.teardown_complete = true;
        self.close_ingress();
        self.native_ownership.mark_loop_exited();

        let instances = self.registry.take_live_for_teardown();
        for instance in &instances {
            let id = instance.id();
            self.cleanup_window_state(id, instance, false);
            if let Some(handle) = instance.handle.as_ref() {
                self.native_ownership
                    .remember_terminal_lease(handle.downgrade());
                handle.mark_loop_exited();
            }
        }
        drop(instances);

        self.native_ownership.mark_loop_exited();
        for id in self.native_ownership.closing_ids() {
            let _ = self.registry.complete_close(id);
        }
        self.windows.clear();
        self.closing_windows.clear();
        self.pending_releases.clear();
    }

    fn apply_ime(&mut self, id: Id, request: ResolvedImeRequest) -> Result<()> {
        let handle = self.handle(id)?;
        handle.with_winit(|window| winit_mapping::apply_native_ime_request(window, &request))?;
        Ok(())
    }

    fn apply_patch(&mut self, patch: WindowStatePatch) -> Result<Option<EventKind>> {
        let id = patch.id();
        let instance = self
            .registry
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        patch.apply(&mut instance.state)
    }

    fn inner_size_bounds(&self, id: Id) -> Result<InnerSizeBounds> {
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
        callbacks: &mut Vec<BackendCallback>,
    ) -> Result<()> {
        let instance = self
            .registry
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        instance.set_inner_size_bounds(bounds);
        if let Some(size) = resolved_size {
            let result = self.handle(id)?.with_winit(|window| {
                window.request_inner_size(winit::dpi::LogicalSize::new(size.width, size.height))
            })?;
            self.apply_size_request_result(
                id,
                size_application_result_from_winit(result),
                callbacks,
            )?;
        }
        Ok(())
    }

    #[cfg(test)]
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

    fn apply_size_request_result(
        &mut self,
        id: Id,
        result: SizeApplicationResult,
        callbacks: &mut Vec<BackendCallback>,
    ) -> Result<()> {
        let SizeApplicationResult::Immediate(size) = result else {
            self.pending_size_requests.insert(id);
            return Ok(());
        };
        let transition =
            self.metrics_transition(id, size.width, size.height, MetricsEvent::Resized)?;
        self.apply_native_transition_patch_state(transition)?;
        self.pending_size_requests.remove(&id);
        self.immediate_size_results
            .entry(id)
            .or_default()
            .push_back(size);
        callbacks.push(BackendCallback::Resize(id));
        Ok(())
    }

    fn observe_resized(&mut self, id: Id, size: PhysicalSize) -> Result<bool> {
        self.pending_size_requests.remove(&id);
        let matched_immediate_result =
            self.immediate_size_results
                .get_mut(&id)
                .and_then(|results| {
                    let index = results.iter().position(|result| *result == size)?;
                    results.remove(index);
                    Some(results.is_empty())
                });
        if let Some(remove_empty_results) = matched_immediate_result {
            if remove_empty_results {
                self.immediate_size_results.remove(&id);
            }
            return Ok(false);
        }
        let transition =
            self.metrics_transition(id, size.width, size.height, MetricsEvent::Resized)?;
        self.apply_native_transition_patch_state(transition)?;
        Ok(true)
    }

    #[cfg(test)]
    fn apply_inner_size_bounds_for_test(
        &mut self,
        id: Id,
        bounds: InnerSizeBounds,
        resolved_size: Option<Size>,
        callbacks: &mut Vec<BackendCallback>,
    ) -> Result<()> {
        let instance = self
            .registry
            .get_mut(id)
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        instance.set_inner_size_bounds(bounds);
        if let Some(size) = resolved_size {
            let result = self.immediate_size_result(id, size)?;
            self.apply_size_request_result(id, result, callbacks)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn require_test_window(&self, id: Id) -> Result<()> {
        self.registry
            .contains(id)
            .then_some(())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))
    }

    fn id_for_winit(&self, window_id: winit::window::WindowId) -> Option<Id> {
        self.windows.get(&window_id).copied()
    }

    fn id_for_closing_winit(&self, window_id: winit::window::WindowId) -> Option<Id> {
        self.closing_windows.get(&window_id).copied()
    }

    fn should_exit_after_closed(&self) -> bool {
        self.registry.is_empty() && self.native_ownership.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn into_terminal_result(self) -> Result<()> {
        self.finish_terminal_result(Ok(()))
    }

    pub(crate) fn finish_terminal_result(mut self, native_result: Result<()>) -> Result<()> {
        self.teardown();
        let has_outstanding_leases = self.native_ownership.has_outstanding_leases();
        let pump_result = self.pump.into_result();
        terminal_result(pump_result, native_result, has_outstanding_leases)
    }
}

pub(crate) fn cursor_grab_failed(
    id: Id,
    source: impl std::error::Error + Send + Sync + 'static,
) -> Error {
    Error::new(ErrorCode::CursorRequestFailed, "cursor grab failed")
        .with_id(id)
        .with_source(source)
}

struct WinitPumpBackend<'a, 'event_loop, H> {
    runner: &'a mut WinitRunner<H>,
    event_loop: &'event_loop winit::event_loop::ActiveEventLoop,
}

impl<H: Handler> PumpBackend<BackendCallback> for WinitPumpBackend<'_, '_, H> {
    fn invoke_callback(
        &mut self,
        callback: BackendCallback,
        commands: &mut Vec<Command>,
        actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion> {
        self.runner.invoke_callback(callback, commands, actions)
    }

    fn apply_commands(
        &mut self,
        mut commands: Vec<NormalizedCommand>,
        origin: WorkOrigin,
        pump: &mut Pump<BackendCallback>,
    ) -> Result<()> {
        commands.retain(|command| {
            accepts_work_target(origin, &self.runner.registry, command.command().target())
        });
        if commands.is_empty() && self.runner.staged_startup.is_none() {
            return Ok(());
        }
        let batch = self.runner.plan_pump_batch(commands)?;
        apply_batch_plan(batch, |plan| {
            let begin_close = (plan.kind() == CommandKind::Destroy)
                .then(|| plan.target().expect("destroy plans target one window"));
            let mut callbacks = Vec::new();
            self.runner.apply_host_command_with_follow_ups(
                self.event_loop,
                plan,
                &mut callbacks,
            )?;
            if let Some(id) = begin_close {
                pump.apply_begin_close(self, id)?;
            }
            for callback in callbacks {
                pump.enqueue_callback(callback);
            }
            Ok(())
        })
    }

    fn apply_action(
        &mut self,
        action: Action,
        origin: WorkOrigin,
        pump: &mut Pump<BackendCallback>,
    ) -> Result<()> {
        if !accepts_action_target(origin, &self.runner.registry, action.target())? {
            return Ok(());
        }
        self.runner
            .apply_action_from_pump(self.event_loop, action, pump)
    }

    fn begin_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        self.runner.begin_close(id, pump)
    }

    fn complete_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        self.runner.complete_close(id, pump)
    }

    fn after_callback(&mut self, callback: BackendCallback, pump: &mut Pump<BackendCallback>) {
        let closed = matches!(callback, BackendCallback::Closed(_));
        self.runner.enqueue_ready_after_created(callback, pump);
        if closed && self.runner.should_exit_after_closed() {
            self.event_loop.exit();
        }
    }

    fn close_ingress(&mut self) {
        self.runner.close_ingress();
    }

    fn teardown(&mut self) {
        self.runner.teardown();
    }
}

#[cfg(test)]
struct TestPumpBackend<'a, H> {
    runner: &'a mut WinitRunner<H>,
    applied: &'a std::cell::RefCell<Vec<Id>>,
}

#[cfg(test)]
impl<H: Handler> PumpBackend<BackendCallback> for TestPumpBackend<'_, H> {
    fn invoke_callback(
        &mut self,
        callback: BackendCallback,
        commands: &mut Vec<Command>,
        actions: &mut Vec<Action>,
    ) -> Result<CallbackCompletion> {
        self.runner.invoke_callback(callback, commands, actions)
    }

    fn apply_commands(
        &mut self,
        mut commands: Vec<NormalizedCommand>,
        origin: WorkOrigin,
        pump: &mut Pump<BackendCallback>,
    ) -> Result<()> {
        commands.retain(|command| {
            accepts_work_target(origin, &self.runner.registry, command.command().target())
        });
        if commands.is_empty() && self.runner.staged_startup.is_none() {
            return Ok(());
        }
        let batch = self.runner.plan_pump_batch(commands)?;
        apply_batch_plan(batch, |plan| {
            let begin_close = (plan.kind() == CommandKind::Destroy)
                .then(|| plan.target().expect("destroy plans target one window"));
            let mut callbacks = Vec::new();
            let resolved = take_resolved_host_command(plan);
            #[cfg(feature = "accessibility")]
            let accessibility_update = resolved.accessibility_update;
            #[cfg(feature = "accessibility")]
            let accessibility_open_phase = resolved.accessibility_open_phase;
            let command = resolved.command.into_command();
            let recorded = command.clone();
            match command {
                Command::Open { request } => {
                    let id = resolved
                        .open_id
                        .expect("preflighted open commands carry a planned identity");
                    #[cfg(all(test, feature = "accessibility"))]
                    let missing_proxy = self
                        .runner
                        .missing_accessibility_proxy_for_test
                        .as_ref()
                        .filter(|configuration| configuration.id == id)
                        .map(|configuration| configuration.local_window_drops.clone());
                    let state = with_planned_open_reservation(
                        &mut self.runner.registry,
                        id,
                        |_, registry| {
                            let state = projected_snapshot_from_request(id, &request)?;
                            #[cfg(all(test, feature = "accessibility"))]
                            if let Some(local_window_drops) = missing_proxy.as_ref() {
                                let _local_window =
                                    TestLocalNativeWindow(local_window_drops.clone());
                                prepare_accessible_open(
                                    id,
                                    true,
                                    None::<()>,
                                    request.visible(),
                                    |_| (),
                                    || panic!("a missing proxy must not restore visibility"),
                                )?;
                            }
                            let mut instance = Instance::new(state.clone());
                            instance.set_inner_size_bounds(InnerSizeBounds::new(
                                request.min_inner_size(),
                                request.max_inner_size(),
                            ));
                            registry.insert(instance)?;
                            #[cfg(feature = "accessibility")]
                            if let Some(phase) = accessibility_open_phase {
                                registry.insert_accessibility_phase(id, phase);
                            }
                            Ok(state)
                        },
                    )?;
                    self.applied.borrow_mut().push(id);
                    callbacks.push(BackendCallback::Event(EventKind::Created(state)));
                }
                Command::SetTitle { id, title } => {
                    self.runner
                        .apply_patch(WindowStatePatch::title(id, title))?;
                }
                Command::SetPosition { id, position } => {
                    self.runner
                        .apply_patch(WindowStatePatch::Position { id, position })?;
                }
                Command::SetVisible { id, visible } => {
                    self.runner
                        .apply_patch(WindowStatePatch::visible(id, visible))?;
                }
                Command::SetInnerSize { id, size } => {
                    let result = self.runner.immediate_size_result(id, size)?;
                    self.runner
                        .apply_size_request_result(id, result, &mut callbacks)?;
                }
                Command::SetMinInnerSize { id, size } => {
                    let bounds =
                        InnerSizeBounds::new(size, self.runner.inner_size_bounds(id)?.maximum);
                    self.runner.apply_inner_size_bounds_for_test(
                        id,
                        bounds,
                        resolved.resolved_size,
                        &mut callbacks,
                    )?;
                }
                Command::SetMaxInnerSize { id, size } => {
                    let bounds =
                        InnerSizeBounds::new(self.runner.inner_size_bounds(id)?.minimum, size);
                    self.runner.apply_inner_size_bounds_for_test(
                        id,
                        bounds,
                        resolved.resolved_size,
                        &mut callbacks,
                    )?;
                }
                Command::SetFullscreen { id, fullscreen } => {
                    self.runner.apply_patch(WindowStatePatch::Fullscreen {
                        id,
                        fullscreen: !matches!(fullscreen, Fullscreen::None),
                    })?;
                }
                Command::SetTheme { id, theme } => {
                    if let Some(event) = self
                        .runner
                        .apply_patch(WindowStatePatch::Theme { id, theme })?
                    {
                        callbacks.push(BackendCallback::Event(event));
                    }
                }
                Command::SetCursor { id, cursor } => {
                    self.runner.require_test_window(id)?;
                    self.runner.cursor_state.insert(id, cursor);
                }
                Command::SetCursorGrab { id, grab } => {
                    self.runner.require_test_window(id)?;
                    if matches!(grab, CursorGrab::None) {
                        self.runner.cursor_grab.remove(&id);
                    } else {
                        self.runner.cursor_grab.insert(id, grab);
                    }
                }
                #[cfg(feature = "accessibility")]
                Command::UpdateAccessibility { id, update } => {
                    let resolved = accessibility_update
                        .expect("preflighted accessibility commands carry a resolved phase");
                    let updates = &mut self.runner.accessibility_updates_for_test;
                    apply_accessibility_update(
                        &mut self.runner.registry,
                        id,
                        update,
                        resolved,
                        updates,
                        |updates, update| updates.push((id, update)),
                    );
                    self.runner.require_test_window(id)?;
                }
                Command::RequestDraw { id } => {
                    self.runner.require_test_window(id)?;
                    self.runner.pending_draws.insert(id);
                    self.runner.draw.request(&Action::DrawNext(id));
                }
                Command::Destroy { id } => {
                    self.runner.require_test_window(id)?;
                }
                Command::SetResizable { id, .. }
                | Command::SetControls { id, .. }
                | Command::SetDecorations { id, .. }
                | Command::SetTransparent { id, .. }
                | Command::SetLevel { id, .. }
                | Command::SetIme { id, .. }
                | Command::RequestUserAttention { id } => {
                    self.runner.require_test_window(id)?;
                }
            }
            self.runner.applied_commands.push(recorded);
            if let Some(id) = begin_close {
                pump.apply_begin_close(self, id)?;
            }
            for callback in callbacks {
                pump.enqueue_callback(callback);
            }
            Ok(())
        })
    }

    fn apply_action(
        &mut self,
        action: Action,
        origin: WorkOrigin,
        pump: &mut Pump<BackendCallback>,
    ) -> Result<()> {
        if !accepts_action_target(origin, &self.runner.registry, action.target())? {
            return Ok(());
        }
        self.runner.apply_action_for_test(action, pump);
        Ok(())
    }

    fn begin_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        self.runner.begin_close(id, pump)
    }

    fn complete_close(&mut self, id: Id, pump: &mut Pump<BackendCallback>) -> Result<()> {
        self.runner.complete_close(id, pump)
    }

    fn after_callback(&mut self, callback: BackendCallback, pump: &mut Pump<BackendCallback>) {
        self.runner.enqueue_ready_after_created(callback, pump);
    }
}

#[cfg(test)]
fn with_open_reservation<T>(
    registry: &mut Registry,
    operation: impl FnOnce(Id, &mut Registry) -> Result<T>,
) -> Result<T> {
    let id = registry.reserve_id()?;
    match operation(id, registry) {
        Ok(value) => Ok(value),
        Err(error) => {
            registry.retire_pending_reservation(id);
            Err(error)
        }
    }
}

fn with_planned_open_reservation<T>(
    registry: &mut Registry,
    id: Id,
    operation: impl FnOnce(Id, &mut Registry) -> Result<T>,
) -> Result<T> {
    registry.reserve_planned(id)?;
    match operation(id, registry) {
        Ok(value) => Ok(value),
        Err(error) => {
            registry.retire_pending_reservation(id);
            Err(error)
        }
    }
}

impl<H: Handler> winit::application::ApplicationHandler<UserEvent> for WinitRunner<H> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut provider = WinitCapabilityProvider { event_loop };
        self.resolve_capabilities_once(&mut provider);
        match self.stage_first_resume() {
            Ok(true) => self.dispatch_callback(event_loop, BackendCallback::Resume),
            Ok(false) => {
                self.enqueue_resume_callbacks();
                self.drain_pump(event_loop);
            }
            Err(error) => {
                self.fail_terminal(error);
                self.drain_pump(event_loop);
            }
        }
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.enqueue_suspend_callbacks();
        self.drain_pump(event_loop);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Action(action) => {
                self.pump.enqueue_queued_action(action);
                self.drain_pump(event_loop);
            }
            UserEvent::Command(command) => {
                self.pump.enqueue_queued_normalized_command(command);
                self.drain_pump(event_loop);
            }
            UserEvent::HandleReleased(release) => {
                self.handle_released(release.id());
                self.drain_pump(event_loop);
            }
            #[cfg(feature = "accessibility")]
            UserEvent::Accessibility(event) => {
                if let Some(id) = self.id_for_winit(event.window_id) {
                    self.route_accessibility_event(
                        id,
                        accessibility_event_from_winit(id, event.window_event),
                    );
                    self.drain_pump(event_loop);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let live_id = self.id_for_winit(window_id);
        let closing_id = self.id_for_closing_winit(window_id);
        if matches!(&event, winit::event::WindowEvent::Destroyed) {
            let Some(id) = live_id.or(closing_id) else {
                return;
            };
            self.mark_native_destroyed(id);
            self.pump.enqueue_begin_close(id);
            self.pump.enqueue_close_completion(id);
            self.drain_pump(event_loop);
            return;
        }
        let Some(id) = live_id else {
            return;
        };
        #[cfg(feature = "accessibility")]
        if let Ok(handle) = self.handle(id)
            && let Some(adapter) = self.accessibility.get_mut(&id)
        {
            let _ = handle.with_winit(|window| adapter.process_event(window, &event));
        }
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.deliver_close(event_loop, id);
            }
            winit::event::WindowEvent::Destroyed => unreachable!("destroyed events return above"),
            winit::event::WindowEvent::RedrawRequested => {
                self.pending_draws.remove(&id);
                self.dispatch_callback(event_loop, BackendCallback::Frame(id));
            }
            winit::event::WindowEvent::Resized(size) => {
                if self.apply_resize_observation(
                    event_loop,
                    id,
                    PhysicalSize {
                        width: size.width,
                        height: size.height,
                    },
                ) {
                    self.deliver_resize(event_loop, id);
                }
            }
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if self.apply_scale_factor_transition(event_loop, id, scale_factor) {
                    self.deliver_resize(event_loop, id);
                }
            }
            winit::event::WindowEvent::Moved(position) => {
                match self.moved_transition(
                    id,
                    PhysicalPoint {
                        x: position.x,
                        y: position.y,
                    },
                ) {
                    Ok(transition) => self.apply_native_transition(event_loop, id, transition),
                    Err(error) => {
                        self.fail_terminal(error);
                        self.drain_pump(event_loop);
                    }
                }
            }
            winit::event::WindowEvent::Focused(focused) => {
                let transition = NativeEventTransition::focused(id, focused);
                self.apply_native_transition(event_loop, id, transition);
            }
            winit::event::WindowEvent::ThemeChanged(theme) => {
                let theme = Some(theme.into());
                let transition = NativeEventTransition::theme_changed(id, theme);
                self.apply_native_transition(event_loop, id, transition);
            }
            winit::event::WindowEvent::Occluded(occluded) => {
                if !occluded {
                    self.request_pending_draws();
                }
                let transition = NativeEventTransition::occluded(id, occluded);
                self.apply_native_transition(event_loop, id, transition);
            }
            winit::event::WindowEvent::HoveredFile(path) => {
                let entered = !self.hovered_files.contains_key(&id);
                let paths = self.record_hovered_file(id, path);
                let position = self.last_mouse_position(id);
                let event = if entered {
                    FileDragEvent::Entered { id, paths }
                } else {
                    FileDragEvent::Hovered {
                        id,
                        paths,
                        position,
                    }
                };
                self.deliver_event(event_loop, id, EventKind::FileDrag(event));
            }
            winit::event::WindowEvent::DroppedFile(path) => {
                let position = self.last_mouse_position(id);
                self.hovered_files.remove(&id);
                self.deliver_event(
                    event_loop,
                    id,
                    EventKind::FileDrag(file_drag_dropped(id, path, position)),
                );
            }
            winit::event::WindowEvent::HoveredFileCancelled => {
                self.cancel_hovered_files(id);
                self.deliver_event(
                    event_loop,
                    id,
                    EventKind::FileDrag(FileDragEvent::Cancelled { id }),
                );
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state().into();
                self.deliver_input(
                    event_loop,
                    InputEvent::Modifiers {
                        id,
                        modifiers: self.modifiers,
                    },
                );
            }
            winit::event::WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                let input = InputEvent::Key(key_event_from_winit(
                    id,
                    &event,
                    self.modifiers,
                    is_synthetic,
                ));
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::Ime(ime) => {
                let input = InputEvent::Ime(ime_event_from_winit(id, ime));
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                let input = InputEvent::Pointer(PointerEvent {
                    id,
                    phase: PointerPhase::Entered,
                    kind: PointerKind::Mouse,
                    pointer_id: None,
                    position: self
                        .pointer_positions
                        .get(&PointerPositionKey::mouse(id))
                        .copied(),
                    physical_position: None,
                    delta: None,
                    button: None,
                    modifiers: self.modifiers,
                    device: PointerDeviceData::default(),
                    timestamp: Some(Instant::now()),
                });
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::CursorLeft { .. } => {
                let position = self
                    .pointer_positions
                    .remove(&PointerPositionKey::mouse(id));
                let input = InputEvent::Pointer(PointerEvent {
                    id,
                    phase: PointerPhase::Left,
                    kind: PointerKind::Mouse,
                    pointer_id: None,
                    position,
                    physical_position: None,
                    delta: None,
                    button: None,
                    modifiers: self.modifiers,
                    device: PointerDeviceData::default(),
                    timestamp: Some(Instant::now()),
                });
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let point = self.logical_point(id, position.x, position.y);
                let previous = self
                    .pointer_positions
                    .insert(PointerPositionKey::mouse(id), point);
                let physical_position = PhysicalPoint {
                    x: position.x.round() as i32,
                    y: position.y.round() as i32,
                };
                let delta = previous.map(|previous| Point {
                    x: point.x - previous.x,
                    y: point.y - previous.y,
                });
                let transition = NativeEventTransition::mouse_moved(
                    id,
                    point,
                    physical_position,
                    delta,
                    self.modifiers,
                );
                self.apply_native_transition(event_loop, id, transition);
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let input = InputEvent::Pointer(PointerEvent {
                    id,
                    phase: pointer_phase_from_element_state(state),
                    kind: PointerKind::Mouse,
                    pointer_id: None,
                    position: self
                        .pointer_positions
                        .get(&PointerPositionKey::mouse(id))
                        .copied(),
                    physical_position: None,
                    delta: None,
                    button: Some(button.into()),
                    modifiers: self.modifiers,
                    device: PointerDeviceData::default(),
                    timestamp: Some(Instant::now()),
                });
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                let input = InputEvent::Wheel(WheelEvent {
                    id,
                    delta: delta.into(),
                    phase: phase.into(),
                    position: self
                        .pointer_positions
                        .get(&PointerPositionKey::mouse(id))
                        .copied(),
                    modifiers: self.modifiers,
                    timestamp: Some(Instant::now()),
                });
                self.deliver_input(event_loop, input);
            }
            winit::event::WindowEvent::Touch(touch) => {
                let point = self.logical_point(id, touch.location.x, touch.location.y);
                let key = PointerPositionKey::touch(id, touch.id);
                let previous = self.pointer_positions.insert(key, point);
                if matches!(
                    touch.phase,
                    winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled
                ) {
                    self.pointer_positions.remove(&key);
                }
                let input = InputEvent::Pointer(PointerEvent {
                    id,
                    phase: touch_phase_as_pointer_phase(touch.phase),
                    kind: PointerKind::Touch,
                    pointer_id: Some(touch.id),
                    position: Some(point),
                    physical_position: Some(PhysicalPoint {
                        x: touch.location.x.round() as i32,
                        y: touch.location.y.round() as i32,
                    }),
                    delta: previous.map(|previous| Point {
                        x: point.x - previous.x,
                        y: point.y - previous.y,
                    }),
                    button: None,
                    modifiers: self.modifiers,
                    device: PointerDeviceData {
                        force: touch.force.map(|force| force.normalized()),
                        pressure: touch.force.map(|force| force.normalized()),
                        altitude: touch.force.and_then(|force| match force {
                            winit::event::Force::Calibrated { altitude_angle, .. } => {
                                altitude_angle
                            }
                            winit::event::Force::Normalized(_) => None,
                        }),
                        ..PointerDeviceData::default()
                    },
                    timestamp: Some(Instant::now()),
                });
                self.deliver_input(event_loop, input);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !self.pump.is_running() {
            event_loop.exit();
            return;
        }

        if self.handler.wants_idle() {
            self.dispatch_callback(event_loop, BackendCallback::Idle);
        }
        if !self.pump.is_running() {
            return;
        }

        self.request_ready_draws();
        event_loop.set_control_flow(self.control_flow());
    }
}

#[cfg(test)]
pub(crate) fn validate_name(registry: &Registry, name: Option<&str>) -> Result<()> {
    let Some(name) = name else {
        return Ok(());
    };
    if name.is_empty() || registry.window_id(name).is_none() {
        return Ok(());
    }
    Err(Error::new(
        ErrorCode::DuplicateIdentity,
        format!("duplicate window name '{name}'"),
    ))
}

#[cfg(test)]
pub(crate) const fn native_transition_route(event: &EventKind) -> NativeTransitionRoute {
    match event {
        EventKind::Input(_) => NativeTransitionRoute::Input,
        _ => NativeTransitionRoute::Event,
    }
}

#[cfg(feature = "accessibility")]
pub(crate) fn accessibility_event_from_winit(
    id: Id,
    event: accesskit_winit::WindowEvent,
) -> AccessibilityEvent {
    match event {
        accesskit_winit::WindowEvent::InitialTreeRequested => {
            AccessibilityEvent::InitialTreeRequested(id)
        }
        accesskit_winit::WindowEvent::ActionRequested(request) => {
            AccessibilityEvent::ActionRequested(AccessibilityActionRequest { id, request })
        }
        accesskit_winit::WindowEvent::AccessibilityDeactivated => {
            AccessibilityEvent::Deactivated(id)
        }
    }
}

#[cfg(feature = "accessibility")]
fn apply_accessibility_update<A>(
    registry: &mut Registry,
    id: Id,
    update: accesskit::TreeUpdate,
    resolved: super::planning::ResolvedAccessibilityUpdate,
    adapter: &mut A,
    update_adapter: impl FnOnce(&mut A, accesskit::TreeUpdate),
) {
    update_adapter(adapter, update);
    registry.apply_accessibility_phase(id, resolved.phase);
}

impl<H: Handler> WinitRunner<H> {
    #[cfg(feature = "accessibility")]
    fn route_accessibility_event(&mut self, id: Id, event: AccessibilityEvent) {
        if self.registry.accessibility_phase(id).is_none() {
            return;
        }

        match &event {
            AccessibilityEvent::InitialTreeRequested(_) => {
                self.registry.request_accessibility_tree(id);
            }
            AccessibilityEvent::Deactivated(_) => self.registry.deactivate_accessibility(id),
            AccessibilityEvent::ActionRequested(_) => {}
        }
        self.enqueue_callback(BackendCallback::Event(EventKind::Accessibility(event)));
    }

    fn apply_resize_observation(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: Id,
        size: PhysicalSize,
    ) -> bool {
        match self.observe_resized(id, size) {
            Ok(deliver) => deliver,
            Err(error) => {
                self.fail_terminal(error);
                self.drain_pump(event_loop);
                false
            }
        }
    }

    fn apply_scale_factor_transition(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        id: Id,
        scale_factor: f64,
    ) -> bool {
        let result = self
            .scale_factor_transition(id, scale_factor)
            .and_then(|transition| self.apply_native_transition_patch_state(transition));
        self.apply_native_transition_result(event_loop, result)
    }

    fn metrics_transition(
        &self,
        id: Id,
        width: u32,
        height: u32,
        event: MetricsEvent,
    ) -> Result<NativeEventTransition> {
        let existing = self
            .registry
            .get(id)
            .map(|window| window.metrics())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        let metrics = Metrics::from_physical_size(
            id,
            PhysicalSize { width, height },
            existing.scale_factor(),
        )?
        .with_outer_geometry(existing.outer_position(), existing.outer_size())?;
        Ok(match event {
            MetricsEvent::Resized => NativeEventTransition::resized(metrics),
            MetricsEvent::ScaleFactorChanged => {
                NativeEventTransition::scale_factor_changed(metrics)
            }
        })
    }

    fn scale_factor_transition(&self, id: Id, scale_factor: f64) -> Result<NativeEventTransition> {
        let existing = self
            .registry
            .get(id)
            .map(|window| window.metrics())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        let metrics = Metrics::from_physical_size(id, existing.physical_size(), scale_factor)?
            .with_outer_geometry(existing.outer_position(), existing.outer_size())?;
        Ok(NativeEventTransition::scale_factor_changed(metrics))
    }

    fn moved_transition(
        &self,
        id: Id,
        physical_position: PhysicalPoint,
    ) -> Result<NativeEventTransition> {
        let metrics = self
            .registry
            .get(id)
            .map(|window| window.metrics())
            .ok_or_else(|| Error::new(ErrorCode::CommandFailed, "unknown window").with_id(id))?;
        Ok(NativeEventTransition::moved(
            id,
            metrics.physical_to_logical_point(physical_position),
        ))
    }

    fn apply_native_transition_patch_state(
        &mut self,
        transition: NativeEventTransition,
    ) -> Result<()> {
        let id = transition.id();
        self.apply_native_transition_state(id, transition)
            .map(|_| ())
    }

    fn apply_native_transition_result(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        result: Result<()>,
    ) -> bool {
        match result {
            Ok(()) => true,
            Err(error) => {
                self.fail_terminal(error);
                self.drain_pump(event_loop);
                false
            }
        }
    }

    #[cfg(test)]
    fn retain_terminal_result(&mut self, result: Result<()>) -> Result<()> {
        if let Err(error) = result {
            self.fail_terminal(error);
        }
        Ok(())
    }

    fn record_hovered_file(&mut self, id: Id, path: PathBuf) -> Vec<PathBuf> {
        let files = self.hovered_files.entry(id).or_default();
        files.push(path);
        files.clone()
    }

    fn cancel_hovered_files(&mut self, id: Id) {
        self.hovered_files.remove(&id);
    }

    pub(crate) fn last_mouse_position(&self, id: Id) -> Option<Point> {
        self.pointer_positions
            .get(&PointerPositionKey::mouse(id))
            .copied()
    }

    fn logical_point(&self, id: Id, physical_x: f64, physical_y: f64) -> Point {
        let scale = self
            .registry
            .get(id)
            .map(|window| window.metrics().scale_factor())
            .unwrap_or(1.0);
        Point {
            x: physical_x / scale,
            y: physical_y / scale,
        }
    }
}

fn size_application_result_from_winit(
    result: Option<winit::dpi::PhysicalSize<u32>>,
) -> SizeApplicationResult {
    result.map_or(SizeApplicationResult::Deferred, |size| {
        SizeApplicationResult::Immediate(PhysicalSize {
            width: size.width,
            height: size.height,
        })
    })
}

pub(crate) fn state_from_winit(
    id: Id,
    request: &WindowRequest,
    window: &winit::window::Window,
) -> Result<WindowSnapshot> {
    let inner_size = window.inner_size();
    let outer_position = window.outer_position().ok().map(|position| PhysicalPoint {
        x: position.x,
        y: position.y,
    });
    let outer_size = window.outer_size();
    let metrics = metrics_from_winit_geometry(
        id,
        PhysicalSize {
            width: inner_size.width,
            height: inner_size.height,
        },
        outer_position,
        Some(PhysicalSize {
            width: outer_size.width,
            height: outer_size.height,
        }),
        window.scale_factor(),
    )?;
    Ok(WindowSnapshot::from_seed(WindowSnapshotSeed {
        title: request.title().to_owned(),
        name: request.name().map(str::to_owned),
        focused: window.has_focus(),
        visible: Some(request.visible()),
        minimized: None,
        maximized: window.is_maximized(),
        occluded: None,
        fullscreen: window.fullscreen().is_some(),
        theme: request.theme(),
        role: request.role().clone(),
        metrics,
    }))
}

fn metrics_from_winit_geometry(
    id: Id,
    inner_size: PhysicalSize,
    outer_position: Option<PhysicalPoint>,
    outer_size: Option<PhysicalSize>,
    scale_factor: f64,
) -> Result<Metrics> {
    let metrics = Metrics::from_physical_size(id, inner_size, scale_factor)?;
    let outer_position = outer_position.map(|position| metrics.physical_to_logical_point(position));
    let outer_size = outer_size.map(|size| Size {
        width: f64::from(size.width) / metrics.scale_factor(),
        height: f64::from(size.height) / metrics.scale_factor(),
    });
    metrics.with_outer_geometry(outer_position, outer_size)
}

fn file_drag_dropped(id: Id, path: PathBuf, position: Option<Point>) -> FileDragEvent {
    FileDragEvent::Dropped {
        id,
        paths: vec![path],
        position,
    }
}

impl From<winit::window::Theme> for Theme {
    fn from(theme: winit::window::Theme) -> Self {
        match theme {
            winit::window::Theme::Light => Self::Light,
            winit::window::Theme::Dark => Self::Dark,
        }
    }
}

impl From<winit::keyboard::ModifiersState> for ModifierState {
    fn from(modifiers: winit::keyboard::ModifiersState) -> Self {
        Self {
            shift: modifiers.shift_key(),
            control: modifiers.control_key(),
            alt: modifiers.alt_key(),
            super_key: modifiers.super_key(),
        }
    }
}

impl From<winit::event::MouseButton> for PointerButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => Self::Primary,
            winit::event::MouseButton::Right => Self::Secondary,
            winit::event::MouseButton::Middle => Self::Middle,
            winit::event::MouseButton::Back => Self::Back,
            winit::event::MouseButton::Forward => Self::Forward,
            winit::event::MouseButton::Other(button) => Self::Other(button),
        }
    }
}

impl From<winit::event::TouchPhase> for TouchPhase {
    fn from(phase: winit::event::TouchPhase) -> Self {
        match phase {
            winit::event::TouchPhase::Started => Self::Started,
            winit::event::TouchPhase::Moved => Self::Moved,
            winit::event::TouchPhase::Ended => Self::Ended,
            winit::event::TouchPhase::Cancelled => Self::Cancelled,
        }
    }
}

impl From<winit::event::MouseScrollDelta> for WheelDelta {
    fn from(delta: winit::event::MouseScrollDelta) -> Self {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => Self::Lines {
                x: f64::from(x),
                y: f64::from(y),
            },
            winit::event::MouseScrollDelta::PixelDelta(position) => Self::Pixels {
                x: position.x,
                y: position.y,
            },
        }
    }
}

fn pointer_phase_from_element_state(state: winit::event::ElementState) -> PointerPhase {
    match state {
        winit::event::ElementState::Pressed => PointerPhase::Pressed,
        winit::event::ElementState::Released => PointerPhase::Released,
    }
}

fn touch_phase_as_pointer_phase(phase: winit::event::TouchPhase) -> PointerPhase {
    match phase {
        winit::event::TouchPhase::Started => PointerPhase::Pressed,
        winit::event::TouchPhase::Moved => PointerPhase::Moved,
        winit::event::TouchPhase::Ended => PointerPhase::Released,
        winit::event::TouchPhase::Cancelled => PointerPhase::Cancelled,
    }
}

pub(crate) fn ime_event_from_winit(id: Id, ime: winit::event::Ime) -> ImeEvent {
    match ime {
        winit::event::Ime::Enabled => ImeEvent::Enabled { id },
        winit::event::Ime::Disabled => ImeEvent::Disabled { id },
        winit::event::Ime::Preedit(text, cursor) => ImeEvent::Preedit { id, text, cursor },
        winit::event::Ime::Commit(text) => ImeEvent::Commit { id, text },
    }
}

fn key_event_from_winit(
    id: Id,
    event: &winit::event::KeyEvent,
    modifiers: ModifierState,
    synthetic: bool,
) -> KeyEvent {
    KeyEvent {
        id,
        logical_key: key_from_winit(&event.logical_key),
        physical_key: code_from_winit(&event.physical_key),
        location: location_from_winit(event.location),
        state: key_state_from_winit(event.state),
        repeat: event.repeat,
        synthetic,
        modifiers,
        timestamp: Some(Instant::now()),
    }
}

fn key_state_from_winit(state: winit::event::ElementState) -> KeyState {
    match state {
        winit::event::ElementState::Pressed => KeyState::Pressed,
        winit::event::ElementState::Released => KeyState::Released,
    }
}

pub(crate) fn location_from_winit(
    location: winit::keyboard::KeyLocation,
) -> keyboard_types::Location {
    match location {
        winit::keyboard::KeyLocation::Standard => keyboard_types::Location::Standard,
        winit::keyboard::KeyLocation::Left => keyboard_types::Location::Left,
        winit::keyboard::KeyLocation::Right => keyboard_types::Location::Right,
        winit::keyboard::KeyLocation::Numpad => keyboard_types::Location::Numpad,
    }
}

macro_rules! map_identical_named_keys {
    ($named:expr; $($name:ident),+ $(,)?) => {
        match $named {
            $(winit::keyboard::NamedKey::$name => keyboard_types::Key::$name,)+
            _ => keyboard_types::Key::Unidentified,
        }
    };
}

macro_rules! map_identical_key_codes {
    ($code:expr; $($name:ident),+ $(,)?) => {
        match $code {
            $(winit::keyboard::KeyCode::$name => keyboard_types::Code::$name,)+
            _ => keyboard_types::Code::Unidentified,
        }
    };
}

pub(crate) fn key_from_winit(key: &winit::keyboard::Key) -> keyboard_types::Key {
    match key {
        winit::keyboard::Key::Character(character) => {
            keyboard_types::Key::Character(character.to_string())
        }
        winit::keyboard::Key::Dead(_) => keyboard_types::Key::Dead,
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space) => {
            keyboard_types::Key::Character(String::from(" "))
        }
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Meta) => keyboard_types::Key::Super,
        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Super) => keyboard_types::Key::Meta,
        winit::keyboard::Key::Named(named) => map_identical_named_keys!(
            *named;
            Alt,
        AltGraph,
        CapsLock,
        Control,
        Fn,
        FnLock,
        NumLock,
        ScrollLock,
        Shift,
        Symbol,
        SymbolLock,
        Hyper,
        Enter,
        Tab,
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        ArrowUp,
        End,
        Home,
        PageDown,
        PageUp,
        Backspace,
        Clear,
        Copy,
        CrSel,
        Cut,
        Delete,
        EraseEof,
        ExSel,
        Insert,
        Paste,
        Redo,
        Undo,
        Accept,
        Again,
        Attn,
        Cancel,
        ContextMenu,
        Escape,
        Execute,
        Find,
        Help,
        Pause,
        Play,
        Props,
        Select,
        ZoomIn,
        ZoomOut,
        BrightnessDown,
        BrightnessUp,
        Eject,
        LogOff,
        Power,
        PowerOff,
        PrintScreen,
        Hibernate,
        Standby,
        WakeUp,
        AllCandidates,
        Alphanumeric,
        CodeInput,
        Compose,
        Convert,
        FinalMode,
        GroupFirst,
        GroupLast,
        GroupNext,
        GroupPrevious,
        ModeChange,
        NextCandidate,
        NonConvert,
        PreviousCandidate,
        Process,
        SingleCandidate,
        HangulMode,
        HanjaMode,
        JunjaMode,
        Eisu,
        Hankaku,
        Hiragana,
        HiraganaKatakana,
        KanaMode,
        KanjiMode,
        Katakana,
        Romaji,
        Zenkaku,
        ZenkakuHankaku,
        Soft1,
        Soft2,
        Soft3,
        Soft4,
        ChannelDown,
        ChannelUp,
        Close,
        MailForward,
        MailReply,
        MailSend,
        MediaClose,
        MediaFastForward,
        MediaPause,
        MediaPlay,
        MediaPlayPause,
        MediaRecord,
        MediaRewind,
        MediaStop,
        MediaTrackNext,
        MediaTrackPrevious,
        New,
        Open,
        Print,
        Save,
        SpellCheck,
        Key11,
        Key12,
        AudioBalanceLeft,
        AudioBalanceRight,
        AudioBassBoostDown,
        AudioBassBoostToggle,
        AudioBassBoostUp,
        AudioFaderFront,
        AudioFaderRear,
        AudioSurroundModeNext,
        AudioTrebleDown,
        AudioTrebleUp,
        AudioVolumeDown,
        AudioVolumeUp,
        AudioVolumeMute,
        MicrophoneToggle,
        MicrophoneVolumeDown,
        MicrophoneVolumeUp,
        MicrophoneVolumeMute,
        SpeechCorrectionList,
        SpeechInputToggle,
        LaunchApplication1,
        LaunchApplication2,
        LaunchCalendar,
        LaunchContacts,
        LaunchMail,
        LaunchMediaPlayer,
        LaunchMusicPlayer,
        LaunchPhone,
        LaunchScreenSaver,
        LaunchSpreadsheet,
        LaunchWebBrowser,
        LaunchWebCam,
        LaunchWordProcessor,
        BrowserBack,
        BrowserFavorites,
        BrowserForward,
        BrowserHome,
        BrowserRefresh,
        BrowserSearch,
        BrowserStop,
        AppSwitch,
        Call,
        Camera,
        CameraFocus,
        EndCall,
        GoBack,
        GoHome,
        HeadsetHook,
        LastNumberRedial,
        Notification,
        MannerMode,
        VoiceDial,
        TV,
        TV3DMode,
        TVAntennaCable,
        TVAudioDescription,
        TVAudioDescriptionMixDown,
        TVAudioDescriptionMixUp,
        TVContentsMenu,
        TVDataService,
        TVInput,
        TVInputComponent1,
        TVInputComponent2,
        TVInputComposite1,
        TVInputComposite2,
        TVInputHDMI1,
        TVInputHDMI2,
        TVInputHDMI3,
        TVInputHDMI4,
        TVInputVGA1,
        TVMediaContext,
        TVNetwork,
        TVNumberEntry,
        TVPower,
        TVRadioService,
        TVSatellite,
        TVSatelliteBS,
        TVSatelliteCS,
        TVSatelliteToggle,
        TVTerrestrialAnalog,
        TVTerrestrialDigital,
        TVTimer,
        AVRInput,
        AVRPower,
        ColorF0Red,
        ColorF1Green,
        ColorF2Yellow,
        ColorF3Blue,
        ColorF4Grey,
        ColorF5Brown,
        ClosedCaptionToggle,
        Dimmer,
        DisplaySwap,
        DVR,
        Exit,
        FavoriteClear0,
        FavoriteClear1,
        FavoriteClear2,
        FavoriteClear3,
        FavoriteRecall0,
        FavoriteRecall1,
        FavoriteRecall2,
        FavoriteRecall3,
        FavoriteStore0,
        FavoriteStore1,
        FavoriteStore2,
        FavoriteStore3,
        Guide,
        GuideNextDay,
        GuidePreviousDay,
        Info,
        InstantReplay,
        Link,
        ListProgram,
        LiveContent,
        Lock,
        MediaApps,
        MediaAudioTrack,
        MediaLast,
        MediaSkipBackward,
        MediaSkipForward,
        MediaStepBackward,
        MediaStepForward,
        MediaTopMenu,
        NavigateIn,
        NavigateNext,
        NavigateOut,
        NavigatePrevious,
        NextFavoriteChannel,
        NextUserProfile,
        OnDemand,
        Pairing,
        PinPDown,
        PinPMove,
        PinPToggle,
        PinPUp,
        PlaySpeedDown,
        PlaySpeedReset,
        PlaySpeedUp,
        RandomToggle,
        RcLowBattery,
        RecordSpeedNext,
        RfBypass,
        ScanChannelsToggle,
        ScreenModeNext,
        Settings,
        SplitScreenToggle,
        STBInput,
        STBPower,
        Subtitle,
        Teletext,
        VideoModeNext,
        Wink,
        ZoomToggle,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,
        F16,
        F17,
        F18,
        F19,
        F20,
        F21,
        F22,
        F23,
        F24,
        F25,
        F26,
        F27,
        F28,
        F29,
        F30,
        F31,
        F32,
        F33,
        F34,
        F35
        ),
        winit::keyboard::Key::Unidentified(_) => keyboard_types::Key::Unidentified,
    }
}

pub(crate) fn code_from_winit(physical_key: &winit::keyboard::PhysicalKey) -> keyboard_types::Code {
    match physical_key {
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::SuperLeft) => {
            keyboard_types::Code::MetaLeft
        }
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::SuperRight) => {
            keyboard_types::Code::MetaRight
        }
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Meta) => {
            keyboard_types::Code::Super
        }
        winit::keyboard::PhysicalKey::Code(code) => map_identical_key_codes!(
            *code;
            Backquote,
        Backslash,
        BracketLeft,
        BracketRight,
        Comma,
        Digit0,
        Digit1,
        Digit2,
        Digit3,
        Digit4,
        Digit5,
        Digit6,
        Digit7,
        Digit8,
        Digit9,
        Equal,
        IntlBackslash,
        IntlRo,
        IntlYen,
        KeyA,
        KeyB,
        KeyC,
        KeyD,
        KeyE,
        KeyF,
        KeyG,
        KeyH,
        KeyI,
        KeyJ,
        KeyK,
        KeyL,
        KeyM,
        KeyN,
        KeyO,
        KeyP,
        KeyQ,
        KeyR,
        KeyS,
        KeyT,
        KeyU,
        KeyV,
        KeyW,
        KeyX,
        KeyY,
        KeyZ,
        Minus,
        Period,
        Quote,
        Semicolon,
        Slash,
        AltLeft,
        AltRight,
        Backspace,
        CapsLock,
        ContextMenu,
        ControlLeft,
        ControlRight,
        Enter,
        ShiftLeft,
        ShiftRight,
        Space,
        Tab,
        Convert,
        KanaMode,
        Lang1,
        Lang2,
        Lang3,
        Lang4,
        Lang5,
        NonConvert,
        Delete,
        End,
        Help,
        Home,
        Insert,
        PageDown,
        PageUp,
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        ArrowUp,
        NumLock,
        Numpad0,
        Numpad1,
        Numpad2,
        Numpad3,
        Numpad4,
        Numpad5,
        Numpad6,
        Numpad7,
        Numpad8,
        Numpad9,
        NumpadAdd,
        NumpadBackspace,
        NumpadClear,
        NumpadClearEntry,
        NumpadComma,
        NumpadDecimal,
        NumpadDivide,
        NumpadEnter,
        NumpadEqual,
        NumpadHash,
        NumpadMemoryAdd,
        NumpadMemoryClear,
        NumpadMemoryRecall,
        NumpadMemoryStore,
        NumpadMemorySubtract,
        NumpadMultiply,
        NumpadParenLeft,
        NumpadParenRight,
        NumpadStar,
        NumpadSubtract,
        Escape,
        Fn,
        FnLock,
        PrintScreen,
        ScrollLock,
        Pause,
        BrowserBack,
        BrowserFavorites,
        BrowserForward,
        BrowserHome,
        BrowserRefresh,
        BrowserSearch,
        BrowserStop,
        Eject,
        LaunchApp1,
        LaunchApp2,
        LaunchMail,
        MediaPlayPause,
        MediaSelect,
        MediaStop,
        MediaTrackNext,
        MediaTrackPrevious,
        Power,
        Sleep,
        AudioVolumeDown,
        AudioVolumeMute,
        AudioVolumeUp,
        WakeUp,
        Hyper,
        Turbo,
        Abort,
        Resume,
        Suspend,
        Again,
        Copy,
        Cut,
        Find,
        Open,
        Paste,
        Props,
        Select,
        Undo,
        Hiragana,
        Katakana,
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
        F10,
        F11,
        F12,
        F13,
        F14,
        F15,
        F16,
        F17,
        F18,
        F19,
        F20,
        F21,
        F22,
        F23,
        F24,
        F25,
        F26,
        F27,
        F28,
        F29,
        F30,
        F31,
        F32,
        F33,
        F34,
        F35
        ),
        winit::keyboard::PhysicalKey::Unidentified(_) => keyboard_types::Code::Unidentified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        lease::Lease,
        registry::{ProxyQueue, UserEvent},
    };
    use raw_window_handle::HandleError;
    #[cfg(unix)]
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::Arc,
    };

    struct NoopHandler;

    impl Handler for NoopHandler {}

    fn state(id: Id) -> WindowSnapshot {
        WindowSnapshot::new(
            "Test",
            Metrics::from_physical_size(
                id,
                PhysicalSize {
                    width: 640,
                    height: 480,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        )
    }

    fn state_with_scale(id: Id, scale_factor: f64) -> WindowSnapshot {
        WindowSnapshot::new(
            "Test",
            Metrics::from_physical_size(
                id,
                PhysicalSize {
                    width: 640,
                    height: 480,
                },
                scale_factor,
            )
            .expect("test metrics are valid"),
        )
    }

    fn runner_with_live_window<H: Handler>(handler: H, id: Id) -> WinitRunner<H> {
        let mut window_loop = Loop::new(handler);
        window_loop
            .registry
            .insert(Instance::new(state(id)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
                .window(WindowOperation::SetTitle, CapabilitySupport::Supported)
                .window(WindowOperation::Destroy, CapabilitySupport::Supported)
                .cursor_grab(CursorGrab::Locked, CapabilitySupport::Supported)
                .build(),
        );
        runner
    }

    #[cfg(feature = "accessibility")]
    fn runner_with_accessibility_window<H: Handler>(handler: H, id: Id) -> WinitRunner<H> {
        let mut runner = runner_with_live_window(handler, id);
        runner.capabilities = Some(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .accessibility(CapabilitySupport::Supported)
                .build(),
        );
        runner
            .registry
            .insert_accessibility_phase(id, super::planning::AccessibilityPhase::Absent);
        runner
    }

    #[cfg(feature = "accessibility")]
    fn initial_accessibility_tree() -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes: Vec::new(),
            tree: Some(accesskit::Tree::new(accesskit::NodeId(42))),
            tree_id: accesskit::TreeId::ROOT,
            focus: accesskit::NodeId(42),
        }
    }

    #[cfg(feature = "accessibility")]
    fn incremental_accessibility_tree() -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: accesskit::TreeId::ROOT,
            focus: accesskit::NodeId(42),
        }
    }

    fn request_close_for_test<H: Handler>(runner: &mut WinitRunner<H>, id: Id) {
        runner.pump.enqueue_callback(BackendCallback::Close(id));
        runner.drain_pump_for_test();
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_hover_and_drop_paths_round_trip_losslessly() {
        let id = Id::from_u64(1);
        let first = PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/first-\xff".to_vec()));
        let second = PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/second-\xfe".to_vec()));
        let dropped = PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/dropped-\xfd".to_vec()));
        let position = Some(Point { x: 8.0, y: 13.0 });
        let mut runner = WinitRunner::from_loop(Loop::new(NoopHandler));

        let entered = runner.record_hovered_file(id, first.clone());
        let hovered = runner.record_hovered_file(id, second.clone());
        let dropped_event = file_drag_dropped(id, dropped.clone(), position);

        assert_eq!(entered, vec![first.clone()]);
        assert_eq!(hovered, vec![first.clone(), second.clone()]);
        assert_eq!(entered[0].as_os_str().as_bytes(), b"/tmp/first-\xff");
        assert_eq!(hovered[1].as_os_str().as_bytes(), b"/tmp/second-\xfe");
        assert_eq!(
            dropped_event,
            FileDragEvent::Dropped {
                id,
                paths: vec![dropped],
                position,
            }
        );
    }

    #[test]
    fn file_drag_cancel_and_close_clear_only_target_paths() {
        let cancelled = Id::from_u64(1);
        let closing = Id::from_u64(2);
        let surviving = Id::from_u64(3);
        let mut runner = runner_with_live_window(NoopHandler, closing);
        runner
            .registry
            .insert(Instance::new(state(surviving)))
            .expect("surviving test identity inserts");
        runner
            .hovered_files
            .insert(cancelled, vec![PathBuf::from("/tmp/cancelled")]);
        runner
            .hovered_files
            .insert(closing, vec![PathBuf::from("/tmp/closing")]);
        runner
            .hovered_files
            .insert(surviving, vec![PathBuf::from("/tmp/surviving")]);

        runner.cancel_hovered_files(cancelled);
        request_close_for_test(&mut runner, closing);

        assert!(!runner.hovered_files.contains_key(&cancelled));
        assert!(!runner.hovered_files.contains_key(&closing));
        assert_eq!(
            runner.hovered_files.get(&surviving),
            Some(&vec![PathBuf::from("/tmp/surviving")])
        );
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn accessibility_initial_tree_reaches_active_adapter() {
        struct InitialTreeHandler {
            phase_before_callback: Option<super::planning::AccessibilityPhase>,
            update: accesskit::TreeUpdate,
            incremental_update: accesskit::TreeUpdate,
        }

        impl Handler for InitialTreeHandler {
            fn event(&mut self, event: &mut crate::dsl::Event<'_>) -> Result<()> {
                let EventKind::Accessibility(AccessibilityEvent::InitialTreeRequested(id)) =
                    event.event()
                else {
                    return Ok(());
                };
                let id = *id;
                self.phase_before_callback = event.context_mut().registry().accessibility_phase(id);
                event
                    .context_mut()
                    .update_accessibility(id, self.update.clone());
                event
                    .context_mut()
                    .update_accessibility(id, self.incremental_update.clone());
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let update = initial_accessibility_tree();
        let incremental_update = incremental_accessibility_tree();
        let mut runner = runner_with_accessibility_window(
            InitialTreeHandler {
                phase_before_callback: None,
                update: update.clone(),
                incremental_update: incremental_update.clone(),
            },
            id,
        );

        runner.route_native_accessibility_event_for_test(
            id,
            accesskit_winit::WindowEvent::InitialTreeRequested,
        );

        assert_eq!(
            runner.handler.phase_before_callback,
            Some(super::planning::AccessibilityPhase::WaitingForInitialTree)
        );
        assert_eq!(
            runner.registry.accessibility_phase(id),
            Some(super::planning::AccessibilityPhase::Active)
        );
        assert_eq!(
            runner.accessibility_updates_for_test(),
            &[(id, update), (id, incremental_update)]
        );
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn accessibility_incremental_update_requires_initialized_tree() {
        struct IncrementalTreeHandler;

        impl Handler for IncrementalTreeHandler {
            fn event(&mut self, event: &mut crate::dsl::Event<'_>) -> Result<()> {
                let EventKind::Accessibility(AccessibilityEvent::InitialTreeRequested(id)) =
                    event.event()
                else {
                    return Ok(());
                };
                let id = *id;
                event
                    .context_mut()
                    .update_accessibility(id, incremental_accessibility_tree());
                Ok(())
            }
        }

        let id = Id::from_u64(2);
        let mut runner = runner_with_accessibility_window(IncrementalTreeHandler, id);

        runner.route_native_accessibility_event_for_test(
            id,
            accesskit_winit::WindowEvent::InitialTreeRequested,
        );

        assert!(matches!(
            runner.terminal_result_for_test(),
            Some(Err(error)) if error.code == ErrorCode::InvalidRequest
        ));
        assert_eq!(
            runner.registry.accessibility_phase(id),
            Some(super::planning::AccessibilityPhase::WaitingForInitialTree)
        );
        assert!(runner.accessibility_updates_for_test().is_empty());
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn accessibility_action_preserves_complete_typed_request() {
        #[derive(Default)]
        struct ActionHandler {
            requests: Vec<AccessibilityActionRequest>,
        }

        impl Handler for ActionHandler {
            fn event(&mut self, event: &mut crate::dsl::Event<'_>) -> Result<()> {
                if let EventKind::Accessibility(AccessibilityEvent::ActionRequested(request)) =
                    event.event()
                {
                    self.requests.push(request.clone());
                }
                Ok(())
            }
        }

        let id = Id::from_u64(3);
        let request = accesskit::ActionRequest {
            action: accesskit::Action::SetValue,
            target_tree: accesskit::TreeId::ROOT,
            target_node: accesskit::NodeId(42),
            data: Some(accesskit::ActionData::NumericValue(7.5)),
        };
        let mut runner = runner_with_accessibility_window(ActionHandler::default(), id);

        runner.route_native_accessibility_event_for_test(
            id,
            accesskit_winit::WindowEvent::ActionRequested(request.clone()),
        );

        assert_eq!(
            runner.handler.requests,
            vec![AccessibilityActionRequest { id, request }]
        );
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn accessibility_deactivation_and_close_consume_state_once() {
        #[derive(Default)]
        struct DeactivationHandler {
            phases: Vec<Option<super::planning::AccessibilityPhase>>,
            closed: usize,
        }

        impl Handler for DeactivationHandler {
            fn event(&mut self, event: &mut crate::dsl::Event<'_>) -> Result<()> {
                let EventKind::Accessibility(AccessibilityEvent::Deactivated(id)) = event.event()
                else {
                    return Ok(());
                };
                let id = *id;
                self.phases
                    .push(event.context_mut().registry().accessibility_phase(id));
                Ok(())
            }

            fn closed(&mut self, _closed: &mut crate::dsl::Closed<'_>) -> Result<()> {
                self.closed += 1;
                Ok(())
            }
        }

        let id = Id::from_u64(4);
        let mut runner = runner_with_accessibility_window(DeactivationHandler::default(), id);
        runner
            .registry
            .apply_accessibility_phase(id, super::planning::AccessibilityPhase::Active);

        runner.route_native_accessibility_event_for_test(
            id,
            accesskit_winit::WindowEvent::AccessibilityDeactivated,
        );
        request_close_for_test(&mut runner, id);
        request_close_for_test(&mut runner, id);

        assert_eq!(
            runner.handler.phases,
            vec![Some(super::planning::AccessibilityPhase::Absent)]
        );
        assert_eq!(runner.handler.closed, 1);
        assert_eq!(runner.registry.accessibility_phase(id), None);
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn stale_accessibility_deactivation_after_close_is_ignored() {
        #[derive(Default)]
        struct Recorder {
            accessibility_events: usize,
        }

        impl Handler for Recorder {
            fn event(&mut self, event: &mut crate::dsl::Event<'_>) -> Result<()> {
                if matches!(event.event(), EventKind::Accessibility(_)) {
                    self.accessibility_events += 1;
                }
                Ok(())
            }
        }

        let id = Id::from_u64(5);
        let mut runner = runner_with_accessibility_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);
        runner.route_native_accessibility_event_for_test(
            id,
            accesskit_winit::WindowEvent::AccessibilityDeactivated,
        );

        assert_eq!(runner.handler.accessibility_events, 0);
        assert_eq!(runner.registry.accessibility_phase(id), None);
    }

    const TARGETS: [WinitTarget; 9] = [
        WinitTarget::MacOs,
        WinitTarget::Windows,
        WinitTarget::X11,
        WinitTarget::Wayland,
        WinitTarget::Web,
        WinitTarget::Ios,
        WinitTarget::Android,
        WinitTarget::Orbital,
        WinitTarget::Other,
    ];

    const ROLES: [RoleKind; 4] = [
        RoleKind::Root,
        RoleKind::Dialog,
        RoleKind::Tool,
        RoleKind::Popup,
    ];

    const FULLSCREENS: [FullscreenMode; 3] = [
        FullscreenMode::None,
        FullscreenMode::Borderless,
        FullscreenMode::Exclusive,
    ];

    const CURSORS: [CursorCapability; 3] = [
        CursorCapability::Icon,
        CursorCapability::Hidden,
        CursorCapability::Custom,
    ];

    const WINDOW_OPERATIONS: [WindowOperation; 16] = [
        WindowOperation::SetTitle,
        WindowOperation::SetOuterPosition,
        WindowOperation::SetVisible,
        WindowOperation::SetResizable,
        WindowOperation::SetControls,
        WindowOperation::SetDecorations,
        WindowOperation::InitialTransparency,
        WindowOperation::SetTransparent,
        WindowOperation::RequestInnerSize,
        WindowOperation::SetInnerSizeBounds,
        WindowOperation::SetLevel,
        WindowOperation::SetExplicitTheme,
        WindowOperation::ResetTheme,
        WindowOperation::RequestUserAttention,
        WindowOperation::RequestDraw,
        WindowOperation::Destroy,
    ];

    const CURSOR_GRABS: [CursorGrab; 3] =
        [CursorGrab::None, CursorGrab::Confined, CursorGrab::Locked];

    const IME_CAPABILITIES: [ImeCapability; 12] = [
        ImeCapability::Enablement,
        ImeCapability::CursorArea,
        ImeCapability::Purpose(ImePurpose::Normal),
        ImeCapability::Purpose(ImePurpose::Password),
        ImeCapability::Purpose(ImePurpose::Number),
        ImeCapability::Purpose(ImePurpose::Email),
        ImeCapability::Purpose(ImePurpose::Url),
        ImeCapability::Purpose(ImePurpose::Terminal),
        ImeCapability::Hint(ImeHint::None),
        ImeCapability::Hint(ImeHint::Spellcheck),
        ImeCapability::Hint(ImeHint::NoSpellcheck),
        ImeCapability::SurroundingText,
    ];

    fn is_named_target(target: WinitTarget, targets: &[WinitTarget]) -> bool {
        targets.contains(&target)
    }

    fn expected_role(target: WinitTarget, role: RoleKind) -> CapabilitySupport {
        match role {
            RoleKind::Root if target != WinitTarget::Other => CapabilitySupport::Supported,
            RoleKind::Root | RoleKind::Dialog | RoleKind::Tool | RoleKind::Popup => {
                CapabilitySupport::Unsupported
            }
        }
    }

    fn expected_fullscreen(target: WinitTarget, fullscreen: FullscreenMode) -> CapabilitySupport {
        match fullscreen {
            FullscreenMode::None => CapabilitySupport::Supported,
            FullscreenMode::Borderless
                if is_named_target(
                    target,
                    &[
                        WinitTarget::MacOs,
                        WinitTarget::Windows,
                        WinitTarget::X11,
                        WinitTarget::Wayland,
                        WinitTarget::Ios,
                    ],
                ) =>
            {
                CapabilitySupport::Supported
            }
            FullscreenMode::Borderless | FullscreenMode::Exclusive => {
                CapabilitySupport::Unsupported
            }
        }
    }

    fn expected_cursor(target: WinitTarget, cursor: CursorCapability) -> CapabilitySupport {
        match cursor {
            CursorCapability::Icon | CursorCapability::Hidden
                if is_named_target(
                    target,
                    &[
                        WinitTarget::MacOs,
                        WinitTarget::Windows,
                        WinitTarget::X11,
                        WinitTarget::Wayland,
                        WinitTarget::Web,
                    ],
                ) =>
            {
                CapabilitySupport::Supported
            }
            CursorCapability::Icon | CursorCapability::Hidden | CursorCapability::Custom => {
                CapabilitySupport::Unsupported
            }
        }
    }

    fn expected_window(target: WinitTarget, operation: WindowOperation) -> CapabilitySupport {
        let supported = match operation {
            WindowOperation::SetTitle => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                    WinitTarget::Web,
                    WinitTarget::Orbital,
                ],
            ),
            WindowOperation::SetOuterPosition => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Web,
                    WinitTarget::Ios,
                    WinitTarget::Orbital,
                ],
            ),
            WindowOperation::SetVisible => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Ios,
                    WinitTarget::Orbital,
                ],
            ),
            WindowOperation::SetResizable => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::Wayland,
                    WinitTarget::Orbital,
                ],
            ),
            WindowOperation::SetControls => {
                is_named_target(target, &[WinitTarget::MacOs, WinitTarget::Windows])
            }
            WindowOperation::SetDecorations => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                    WinitTarget::Orbital,
                ],
            ),
            WindowOperation::InitialTransparency => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                ],
            ),
            WindowOperation::SetTransparent => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::Wayland,
                ],
            ),
            WindowOperation::RequestInnerSize => target != WinitTarget::Other,
            WindowOperation::SetInnerSizeBounds => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                    WinitTarget::Web,
                ],
            ),
            WindowOperation::SetLevel | WindowOperation::SetExplicitTheme => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                ],
            ),
            WindowOperation::ResetTheme => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::Wayland,
                ],
            ),
            WindowOperation::RequestUserAttention => is_named_target(
                target,
                &[WinitTarget::MacOs, WinitTarget::Windows, WinitTarget::X11],
            ),
            WindowOperation::RequestDraw | WindowOperation::Destroy => true,
        };
        if supported {
            CapabilitySupport::Supported
        } else {
            CapabilitySupport::Unsupported
        }
    }

    fn expected_cursor_grab(target: WinitTarget, grab: CursorGrab) -> CapabilitySupport {
        match grab {
            CursorGrab::None => CapabilitySupport::Supported,
            CursorGrab::Confined if target == WinitTarget::Wayland => {
                CapabilitySupport::RuntimeDependent
            }
            CursorGrab::Confined
                if is_named_target(target, &[WinitTarget::Windows, WinitTarget::X11]) =>
            {
                CapabilitySupport::Supported
            }
            CursorGrab::Confined | CursorGrab::Locked => CapabilitySupport::Unsupported,
        }
    }

    fn expected_ime(target: WinitTarget, capability: ImeCapability) -> CapabilitySupport {
        let supported = match capability {
            ImeCapability::Enablement | ImeCapability::Purpose(ImePurpose::Normal) => {
                is_named_target(
                    target,
                    &[
                        WinitTarget::MacOs,
                        WinitTarget::Windows,
                        WinitTarget::X11,
                        WinitTarget::Wayland,
                        WinitTarget::Ios,
                        WinitTarget::Android,
                    ],
                )
            }
            ImeCapability::CursorArea => is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::Wayland,
                ],
            ),
            ImeCapability::Purpose(ImePurpose::Password | ImePurpose::Terminal) => {
                target == WinitTarget::Wayland
            }
            ImeCapability::Hint(ImeHint::None) => true,
            ImeCapability::Purpose(ImePurpose::Number | ImePurpose::Email | ImePurpose::Url)
            | ImeCapability::Hint(ImeHint::Spellcheck | ImeHint::NoSpellcheck)
            | ImeCapability::SurroundingText => false,
        };
        if supported {
            CapabilitySupport::Supported
        } else {
            CapabilitySupport::Unsupported
        }
    }

    fn expected_accessibility(target: WinitTarget) -> CapabilitySupport {
        #[cfg(feature = "accessibility")]
        {
            supported_when(is_named_target(
                target,
                &[
                    WinitTarget::MacOs,
                    WinitTarget::Windows,
                    WinitTarget::X11,
                    WinitTarget::Wayland,
                    WinitTarget::Ios,
                ],
            ))
        }
        #[cfg(not(feature = "accessibility"))]
        {
            let _ = target;
            CapabilitySupport::Unsupported
        }
    }

    #[test]
    fn winit_capability_matrix_matches_every_normative_cell() {
        for target in TARGETS {
            let report = winit_capabilities_for_target(target);
            for role in ROLES {
                assert_eq!(
                    report.role(role),
                    expected_role(target, role),
                    "{target:?} {role:?}"
                );
            }
            for fullscreen in FULLSCREENS {
                assert_eq!(
                    report.fullscreen(fullscreen),
                    expected_fullscreen(target, fullscreen),
                    "{target:?} {fullscreen:?}"
                );
            }
            for cursor in CURSORS {
                assert_eq!(
                    report.cursor(cursor),
                    expected_cursor(target, cursor),
                    "{target:?} {cursor:?}"
                );
            }
            for operation in WINDOW_OPERATIONS {
                assert_eq!(
                    report.window(operation),
                    expected_window(target, operation),
                    "{target:?} {operation:?}"
                );
            }
            for grab in CURSOR_GRABS {
                assert_eq!(
                    report.cursor_grab(grab),
                    expected_cursor_grab(target, grab),
                    "{target:?} {grab:?}"
                );
            }
            for capability in IME_CAPABILITIES {
                assert_eq!(
                    report.ime(capability),
                    expected_ime(target, capability),
                    "{target:?} {capability:?}"
                );
            }
            assert_eq!(
                report.accessibility(),
                expected_accessibility(target),
                "{target:?} accessibility"
            );
        }
    }

    #[test]
    fn winit_unix_target_uses_active_backend_identity() {
        assert_eq!(
            winit_target_from_active_identity(true, false),
            WinitTarget::X11
        );
        assert_eq!(
            winit_target_from_active_identity(false, true),
            WinitTarget::Wayland
        );
        assert_eq!(
            winit_target_from_active_identity(false, false),
            WinitTarget::Other
        );
        assert_eq!(
            winit_target_from_active_identity(true, true),
            WinitTarget::Other
        );
    }

    #[test]
    fn locked_cursor_is_unsupported_without_relative_motion() {
        for target in TARGETS {
            assert_eq!(
                winit_capabilities_for_target(target).cursor_grab(CursorGrab::Locked),
                CapabilitySupport::Unsupported,
                "{target:?}"
            );
        }
    }

    #[test]
    fn ime_none_hint_is_supported_without_native_operation() {
        for target in TARGETS {
            assert_eq!(
                winit_capabilities_for_target(target).ime(ImeCapability::Hint(ImeHint::None)),
                CapabilitySupport::Supported,
                "{target:?}"
            );
        }
    }

    #[test]
    fn android_accessibility_is_unsupported_without_android_adapter_feature() {
        assert_eq!(
            winit_capabilities_for_target(WinitTarget::Android).accessibility(),
            CapabilitySupport::Unsupported
        );
    }

    #[derive(Clone, Copy, Debug)]
    enum TargetDrawAction {
        Next,
        Now,
        At,
    }

    impl TargetDrawAction {
        fn request(self, context: &mut Context<'_>, id: Id) {
            match self {
                Self::Next => {
                    context.draw(id);
                }
                Self::Now => {
                    context.again(id);
                }
                Self::At => {
                    context.at(id, Instant::now() + Duration::from_secs(1));
                }
            }
        }
    }

    fn assert_rejected_target_action<H: Handler>(
        runner: WinitRunner<H>,
        id: Id,
        expected_code: ErrorCode,
    ) {
        assert!(!runner.draw.next.contains(&id));
        assert!(!runner.draw.delayed.contains_key(&id));
        assert!(!runner.pending_draws.contains(&id));

        let error = runner
            .pump
            .into_result()
            .expect_err("a strict target action must retain its lifecycle diagnostic");
        assert_eq!(error.code, expected_code);
        assert_eq!(error.id, Some(id));
        assert_eq!(error.command_kind, None);
    }

    #[test]
    fn authored_destroy_then_target_draw_actions_reject_without_stale_scheduler_state() {
        struct DestroyThenDraw {
            action: TargetDrawAction,
        }

        impl Handler for DestroyThenDraw {
            fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
                let id = ready.id();
                ready.context_mut().send(Command::Destroy { id });
                self.action.request(ready.context_mut(), id);
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        for action in [TargetDrawAction::Next, TargetDrawAction::At] {
            let mut runner = runner_with_live_window(DestroyThenDraw { action }, id);
            runner.pump.enqueue_callback(BackendCallback::Ready(id));
            runner.drain_pump_for_test();

            assert_rejected_target_action(runner, id, ErrorCode::StaleWindow);
        }
    }

    #[test]
    fn closed_callback_target_draw_actions_reject_without_stale_scheduler_state() {
        struct ClosedDraw {
            action: TargetDrawAction,
        }

        impl Handler for ClosedDraw {
            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                let id = closed.id();
                self.action.request(closed.context_mut(), id);
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        for action in [
            TargetDrawAction::Next,
            TargetDrawAction::Now,
            TargetDrawAction::At,
        ] {
            let mut runner = runner_with_live_window(ClosedDraw { action }, id);
            runner.pump.enqueue_begin_close(id);
            runner.drain_pump_for_test();

            assert_rejected_target_action(runner, id, ErrorCode::StaleWindow);
        }
    }

    #[test]
    fn strict_target_action_for_closing_window_returns_window_closing() {
        struct ClosingDraw {
            id: Id,
        }

        impl Handler for ClosingDraw {
            fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
                context.draw(self.id);
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(ClosingDraw { id }, id);
        let _ = runner
            .registry
            .begin_close(id)
            .expect("test window enters closing lifecycle");
        runner.pump.enqueue_callback(BackendCallback::Idle);
        runner.drain_pump_for_test();

        assert_rejected_target_action(runner, id, ErrorCode::WindowClosing);
    }

    #[test]
    fn strict_target_action_for_unknown_window_returns_stale_window() {
        struct UnknownDraw {
            id: Id,
        }

        impl Handler for UnknownDraw {
            fn idle(&mut self, context: &mut Context<'_>) -> Result<()> {
                context.at(self.id, Instant::now() + Duration::from_secs(1));
                Ok(())
            }
        }

        let live = Id::from_u64(1);
        let unknown = Id::from_u64(404);
        let mut runner = runner_with_live_window(UnknownDraw { id: unknown }, live);
        runner.pump.enqueue_callback(BackendCallback::Idle);
        runner.drain_pump_for_test();

        assert_rejected_target_action(runner, unknown, ErrorCode::StaleWindow);
    }

    #[test]
    fn queued_nested_stale_target_actions_are_noops_while_live_targets_schedule() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut runner = runner_with_live_window(NoopHandler, closing);
        runner
            .registry
            .insert(Instance::new(state(surviving)))
            .expect("surviving test identity inserts");
        runner.pump.enqueue_begin_close(closing);
        runner.drain_pump_for_test();

        runner.pump.enqueue_queued_action(Action::Batch(vec![
            Action::DrawNext(closing),
            Action::Batch(vec![
                Action::DrawAt {
                    id: closing,
                    time: Instant::now() + Duration::from_secs(1),
                },
                Action::DrawNext(surviving),
            ]),
        ]));
        runner.drain_pump_for_test();

        assert!(!runner.draw.next.contains(&closing));
        assert!(!runner.draw.delayed.contains_key(&closing));
        assert!(!runner.pending_draws.contains(&closing));
        assert!(runner.draw.next.contains(&surviving));
        assert!(runner.pump.into_result().is_ok());
    }

    struct NativeClosingCleanupProbe {
        operations: RefCell<Vec<&'static str>>,
        release_fails: bool,
    }

    impl NativeClosingCleanupProbe {
        fn new(release_fails: bool) -> Self {
            Self {
                operations: RefCell::new(Vec::new()),
                release_fails,
            }
        }

        fn hide(&self) {
            self.operations.borrow_mut().push("hide");
        }

        fn release_cursor_grab(&self) -> std::result::Result<(), ()> {
            self.operations.borrow_mut().push("release-grab");
            (!self.release_fails).then_some(()).ok_or(())
        }
    }

    #[test]
    fn native_closing_cleanup_hides_before_releasing_grab_once() {
        let probe = NativeClosingCleanupProbe::new(false);

        cleanup_native_window(|| probe.hide(), || probe.release_cursor_grab());

        assert_eq!(*probe.operations.borrow(), ["hide", "release-grab"]);
    }

    #[test]
    fn native_closing_cleanup_ignores_grab_release_failure() {
        let probe = NativeClosingCleanupProbe::new(true);

        cleanup_native_window(|| probe.hide(), || probe.release_cursor_grab());

        assert_eq!(*probe.operations.borrow(), ["hide", "release-grab"]);
    }

    #[test]
    fn closing_cancels_next_delayed_and_pending_draws() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut window_loop = Loop::new(NoopHandler);
        for id in [closing, surviving] {
            window_loop
                .registry
                .insert(Instance::new(state(id)))
                .expect("test identity inserts");
        }
        let mut runner = WinitRunner::from_loop(window_loop);
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .cursor_grab(CursorGrab::Locked, CapabilitySupport::Supported)
                .build(),
        );
        let deadline = Instant::now() + Duration::from_secs(1);

        runner.draw.next.extend([closing, surviving]);
        runner.draw.delayed.insert(closing, deadline);
        runner.draw.delayed.insert(surviving, deadline);
        runner.pending_draws.extend([closing, surviving]);

        request_close_for_test(&mut runner, closing);

        assert!(!runner.draw.next.contains(&closing));
        assert!(!runner.draw.delayed.contains_key(&closing));
        assert!(!runner.pending_draws.contains(&closing));
        assert!(runner.draw.next.contains(&surviving));
        assert_eq!(runner.draw.delayed.get(&surviving), Some(&deadline));
        assert!(runner.pending_draws.contains(&surviving));
    }

    #[test]
    fn closing_clears_pointer_drag_cursor_and_accessibility_state() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut window_loop = Loop::new(NoopHandler);
        for id in [closing, surviving] {
            window_loop
                .registry
                .insert(Instance::new(state(id)))
                .expect("test identity inserts");
        }
        let mut runner = WinitRunner::from_loop(window_loop);
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .cursor_grab(CursorGrab::Locked, CapabilitySupport::Supported)
                .build(),
        );
        runner.pump.enqueue_normalized_commands(vec![
            normalize_command(Command::SetCursorGrab {
                id: closing,
                grab: CursorGrab::Locked,
            })
            .expect("closing cursor grab normalizes"),
            normalize_command(Command::SetCursorGrab {
                id: surviving,
                grab: CursorGrab::Locked,
            })
            .expect("surviving cursor grab normalizes"),
        ]);
        runner.drain_pump_for_test();
        assert_eq!(runner.cursor_grab.get(&closing), Some(&CursorGrab::Locked));
        assert_eq!(
            runner.cursor_grab.get(&surviving),
            Some(&CursorGrab::Locked)
        );
        runner
            .pointer_positions
            .insert(PointerPositionKey::mouse(closing), Point { x: 1.0, y: 2.0 });
        runner.pointer_positions.insert(
            PointerPositionKey::touch(closing, 10),
            Point { x: 3.0, y: 4.0 },
        );
        runner.pointer_positions.insert(
            PointerPositionKey::touch(closing, 11),
            Point { x: 5.0, y: 6.0 },
        );
        runner.pointer_positions.insert(
            PointerPositionKey::mouse(surviving),
            Point { x: 7.0, y: 8.0 },
        );
        runner
            .hovered_files
            .insert(closing, vec![PathBuf::from("closing")]);
        runner
            .hovered_files
            .insert(surviving, vec![PathBuf::from("surviving")]);
        runner.cursor_state.insert(closing, Cursor::Hidden);
        runner
            .cursor_state
            .insert(surviving, Cursor::Icon(CursorIcon::Default));
        runner.modifiers = ModifierState {
            shift: true,
            ..ModifierState::default()
        };
        runner
            .windows
            .insert(winit::window::WindowId::dummy(), closing);

        request_close_for_test(&mut runner, closing);

        assert!(
            runner
                .pointer_positions
                .keys()
                .all(|key| key.window != closing)
        );
        assert!(
            runner
                .pointer_positions
                .contains_key(&PointerPositionKey::mouse(surviving))
        );
        assert!(!runner.hovered_files.contains_key(&closing));
        assert_eq!(
            runner.hovered_files.get(&surviving),
            Some(&vec![PathBuf::from("surviving")])
        );
        assert!(!runner.cursor_state.contains_key(&closing));
        assert_eq!(
            runner.cursor_state.get(&surviving),
            Some(&Cursor::Icon(CursorIcon::Default))
        );
        assert!(!runner.cursor_grab.contains_key(&closing));
        assert_eq!(
            runner.cursor_grab.get(&surviving),
            Some(&CursorGrab::Locked)
        );
        assert_eq!(
            runner.modifiers,
            ModifierState {
                shift: true,
                ..ModifierState::default()
            }
        );
        assert_eq!(runner.id_for_winit(winit::window::WindowId::dummy()), None);
    }

    #[test]
    fn closing_route_cleanup_retains_unrelated_routes() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut routes = HashMap::from([("closing", closing), ("surviving", surviving)]);

        remove_routed_window(&mut routes, closing);

        assert_eq!(routes.get("closing"), None);
        assert_eq!(routes.get("surviving"), Some(&surviving));
    }

    #[cfg(feature = "accessibility")]
    #[test]
    fn closing_accessibility_cleanup_removes_only_closing_entry() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut adapter_state = HashMap::from([(closing, ()), (surviving, ())]);

        remove_window_state(&mut adapter_state, closing);

        assert!(!adapter_state.contains_key(&closing));
        assert!(adapter_state.contains_key(&surviving));
    }

    #[test]
    fn closing_cleanup_is_idempotent_and_preserves_other_window_state() {
        let closing = Id::from_u64(1);
        let surviving = Id::from_u64(2);
        let mut window_loop = Loop::new(NoopHandler);
        for id in [closing, surviving] {
            window_loop
                .registry
                .insert(Instance::new(state(id)))
                .expect("test identity inserts");
        }
        let mut runner = WinitRunner::from_loop(window_loop);
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .build(),
        );
        runner.draw.next.insert(closing);
        runner.draw.next.insert(surviving);
        runner.pending_size_requests.insert(closing);
        runner.pending_size_requests.insert(surviving);
        runner
            .immediate_size_results
            .entry(closing)
            .or_default()
            .push_back(PhysicalSize {
                width: 400,
                height: 300,
            });
        runner
            .immediate_size_results
            .entry(surviving)
            .or_default()
            .push_back(PhysicalSize {
                width: 500,
                height: 400,
            });
        runner.pointer_positions.insert(
            PointerPositionKey::mouse(surviving),
            Point { x: 1.0, y: 1.0 },
        );
        runner.cursor_state.insert(surviving, Cursor::Hidden);

        runner.pump.enqueue_begin_close(closing);
        runner.drain_pump_for_test();
        runner.pump.enqueue_begin_close(closing);
        runner.drain_pump_for_test();

        assert!(!runner.draw.next.contains(&closing));
        assert!(runner.draw.next.contains(&surviving));
        assert!(!runner.pending_size_requests.contains(&closing));
        assert!(runner.pending_size_requests.contains(&surviving));
        assert!(!runner.immediate_size_results.contains_key(&closing));
        assert!(runner.immediate_size_results.contains_key(&surviving));
        assert!(
            runner
                .pointer_positions
                .contains_key(&PointerPositionKey::mouse(surviving))
        );
        assert_eq!(runner.cursor_state.get(&surviving), Some(&Cursor::Hidden));
        assert!(runner.registry.contains(surviving));
    }

    #[test]
    fn close_destroy_then_open_reuses_name_at_the_committed_transition() {
        #[derive(Default)]
        struct Recorder {
            callbacks: Vec<String>,
        }

        impl Handler for Recorder {
            fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
                if let EventKind::Created(state) = event.event() {
                    self.callbacks.push(format!(
                        "created:{}",
                        state.name().expect("replacement window is named")
                    ));
                }
                Ok(())
            }

            fn ready(&mut self, ready: &mut Ready<'_>) -> Result<()> {
                self.callbacks.push(format!(
                    "ready:{}",
                    ready.state().name().expect("replacement window is named")
                ));
                Ok(())
            }

            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                self.callbacks
                    .push(format!("closed:{}", closed.id().as_u64()));
                Ok(())
            }
        }

        let destroyed = Id::from_u64(1);
        let replacement = Id::from_u64(2);
        let mut window_loop = Loop::new(Recorder::default());
        window_loop
            .registry
            .insert(Instance::new(state(destroyed).named("shared")))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
                .window(WindowOperation::Destroy, CapabilitySupport::Supported)
                .build(),
        );
        runner.commands = vec![Command::Destroy { id: destroyed }, open("shared").into()];
        let applied = RefCell::new(Vec::new());

        runner
            .apply_open_commands_for_test(&applied)
            .expect("preflighted destroy then name reuse must commit without a partial error");

        assert_eq!(applied.into_inner(), [replacement]);
        assert_eq!(
            runner.applied_commands_for_test(),
            [
                Command::Destroy { id: destroyed },
                Command::Open {
                    request: WindowRequest::builder("shared").build(),
                },
            ]
        );
        assert_eq!(runner.registry.window_id("shared"), Some(replacement));
        assert_eq!(
            runner.handler.callbacks,
            ["closed:1", "created:shared", "ready:shared"]
        );
        assert!(runner.into_terminal_result().is_ok());
    }

    #[test]
    fn close_then_cancel_keeps_window_live() {
        struct CancelAfterClose;

        impl Handler for CancelAfterClose {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                close.close().cancel();
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(CancelAfterClose, id);

        request_close_for_test(&mut runner, id);

        assert!(runner.registry.contains(id));
        assert!(runner.applied_commands.is_empty());
    }

    #[test]
    fn close_cancel_then_close_closes_once() {
        #[derive(Default)]
        struct Recorder {
            closed: usize,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                close.cancel().close();
                Ok(())
            }

            fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
                self.closed += 1;
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);

        assert!(!runner.registry.contains(id));
        assert_eq!(runner.handler.closed, 1);
        assert!(runner.applied_commands.is_empty());
    }

    #[test]
    fn close_duplicate_calls_close_once() {
        #[derive(Default)]
        struct Recorder {
            closed: usize,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                close.close().close();
                Ok(())
            }

            fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
                self.closed += 1;
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);

        assert!(!runner.registry.contains(id));
        assert_eq!(runner.handler.closed, 1);
        assert!(runner.pump.into_result().is_ok());
    }

    #[test]
    fn close_callback_failure_commits_no_decision_or_work() {
        struct FailingClose;

        impl Handler for FailingClose {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                close.close().draw();
                let id = close.id();
                close.context_mut().send(Command::SetTitle {
                    id,
                    title: String::from("discarded"),
                });
                Err(Error::new(
                    ErrorCode::CommandFailed,
                    "close callback failure",
                ))
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(FailingClose, id);

        request_close_for_test(&mut runner, id);

        assert!(runner.registry.contains(id));
        assert!(runner.applied_commands.is_empty());
        assert!(!runner.pending_draws.contains(&id));
        let error = runner
            .pump
            .into_result()
            .expect_err("the close callback failure is terminal");
        assert_eq!(error.message, "close callback failure");
    }

    #[test]
    fn close_authored_destroy_and_accepted_decision_enqueue_same_transition() {
        #[derive(Default)]
        struct AcceptClose {
            closed: usize,
        }

        impl Handler for AcceptClose {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                let id = close.id();
                close.context_mut().send(Command::Destroy { id });
                close.close();
                Ok(())
            }

            fn closed(&mut self, _closed: &mut Closed<'_>) -> Result<()> {
                self.closed += 1;
                Ok(())
            }
        }

        let accepted_id = Id::from_u64(1);
        let mut accepted = runner_with_live_window(AcceptClose::default(), accepted_id);
        request_close_for_test(&mut accepted, accepted_id);

        assert!(!accepted.registry.contains(accepted_id));
        assert_eq!(accepted.handler.closed, 1);
        assert_eq!(
            accepted.applied_commands,
            [Command::Destroy { id: accepted_id }]
        );
    }

    #[test]
    fn close_callback_work_runs_before_the_begin_close_transition() {
        #[derive(Default)]
        struct Recorder {
            closed_title: Option<String>,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                let id = close.id();
                close.context_mut().send(Command::SetTitle {
                    id,
                    title: String::from("committed before close"),
                });
                close.draw().close();
                Ok(())
            }

            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                self.closed_title = Some(closed.state().title().to_owned());
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);

        assert_eq!(
            runner.handler.closed_title.as_deref(),
            Some("committed before close")
        );
        assert!(!runner.draw.next.contains(&id));
        assert!(!runner.registry.contains(id));
    }

    #[test]
    fn stale_close_request_skips_close_scope() {
        #[derive(Default)]
        struct Recorder {
            close_calls: usize,
        }

        impl Handler for Recorder {
            fn close(&mut self, _close: &mut Close<'_>) -> Result<()> {
                self.close_calls += 1;
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(Recorder::default(), id);
        runner.pump.enqueue_begin_close(id);
        runner.drain_pump_for_test();

        request_close_for_test(&mut runner, id);

        assert_eq!(runner.handler.close_calls, 0);
        assert!(!runner.registry.contains(id));
        assert!(runner.pump.into_result().is_ok());
    }

    #[test]
    fn closed_callback_nested_work_runs_after_the_close_transition() {
        #[derive(Default)]
        struct Recorder {
            callbacks: Vec<String>,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                self.callbacks.push(String::from("close"));
                close.close();
                Ok(())
            }

            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                self.callbacks.push(String::from("closed"));
                let id = closed.id();
                assert!(closed.context_mut().state(id).is_none());
                closed.context_mut().open(open("replacement"));
                Ok(())
            }

            fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
                if let EventKind::Created(state) = event.event() {
                    self.callbacks
                        .push(format!("created:{}", state.name().unwrap()));
                }
                Ok(())
            }
        }

        let id = Id::from_u64(1);
        let mut runner = runner_with_live_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);

        assert_eq!(
            runner.handler.callbacks,
            ["close", "closed", "created:replacement"]
        );
        assert_eq!(
            runner.registry.window_id("replacement"),
            Some(Id::from_u64(2))
        );
    }

    #[test]
    fn native_batch_defers_created_callback_open_until_preplanned_opens_commit() {
        struct QueueThirdOpen {
            created: Rc<RefCell<Vec<Id>>>,
        }

        impl Handler for QueueThirdOpen {
            fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
                let EventKind::Created(state) = event.event() else {
                    return Ok(());
                };
                self.created.borrow_mut().push(state.id());
                if state.id() == Id::from_u64(1) {
                    event.context_mut().open(open("third"));
                }
                Ok(())
            }
        }

        let created = Rc::new(RefCell::new(Vec::new()));
        let mut runner = WinitRunner::from_loop(Loop::new(QueueThirdOpen {
            created: created.clone(),
        }));
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .fullscreen(FullscreenMode::None, CapabilitySupport::Supported)
                .build(),
        );
        runner.commands = vec![open("first").into(), open("second").into()];
        let applied = RefCell::new(Vec::new());

        runner
            .apply_open_commands_for_test(&applied)
            .expect("the outer preflighted opens must apply before the Created callback open");

        assert_eq!(
            applied.into_inner(),
            [Id::from_u64(1), Id::from_u64(2), Id::from_u64(3)]
        );
        assert_eq!(
            created.borrow().as_slice(),
            [Id::from_u64(1), Id::from_u64(2), Id::from_u64(3)]
        );
        assert!(runner.registry.contains(Id::from_u64(1)));
        assert!(runner.registry.contains(Id::from_u64(2)));
        assert!(runner.registry.contains(Id::from_u64(3)));
    }

    #[test]
    fn native_open_reservation_exhaustion_skips_native_effect() {
        let exhausted_id = Id::from_u64(u64::MAX);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state(exhausted_id)))
            .expect("maximum fixture identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        let calls = Cell::new(0);

        let error = with_open_reservation(&mut runner.registry, |_, _| {
            calls.set(calls.get() + 1);
            Ok(())
        })
        .expect_err("identity exhaustion must prevent native work");

        assert_eq!(error.code, ErrorCode::IdentityExhausted);
        assert_eq!(calls.get(), 0);
        assert_eq!(runner.registry.len(), 1);
        assert!(runner.registry.contains(exhausted_id));
        assert_eq!(
            runner
                .registry
                .reserve_id()
                .expect_err("exhaustion leaves registry unchanged")
                .code,
            ErrorCode::IdentityExhausted
        );
    }

    #[test]
    fn native_open_duplicate_name_preflight_rejects_without_allocating_or_mutating_live_window() {
        let live_id = Id::from_u64(10);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state(live_id).named("live")))
            .expect("live fixture inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        let before = runner
            .registry
            .get(live_id)
            .expect("live fixture remains available")
            .instance
            .state()
            .clone();

        let error = validate_name(&runner.registry, Some("live"))
            .expect_err("duplicate native runtime names must fail before opening a window");

        assert_eq!(error.code, ErrorCode::DuplicateIdentity);
        assert!(runner.commands.is_empty());
        assert!(runner.windows.is_empty());
        assert_eq!(runner.registry.len(), 1);
        assert_eq!(runner.registry.window_id("live"), Some(live_id));
        assert_eq!(
            runner
                .registry
                .get(live_id)
                .expect("live fixture remains mapped")
                .instance
                .state(),
            &before
        );
        assert_eq!(
            runner
                .registry
                .reserve_id()
                .expect("duplicate rejection must not advance the allocator"),
            Id::from_u64(11)
        );
    }

    #[test]
    fn native_open_reservation_failure_before_effect_retires_identity() {
        let live_id = Id::from_u64(10);
        let mut registry = Registry::new();
        registry
            .insert(Instance::new(state(live_id).named("live")))
            .expect("live fixture inserts");
        let before = registry
            .get(live_id)
            .expect("live fixture remains available")
            .instance
            .state()
            .clone();
        let abandoned = Cell::new(None);
        let native_effect_calls = Cell::new(0);

        let error = with_open_reservation(&mut registry, |id, _| -> Result<()> {
            abandoned.set(Some(id));
            Err(Error::new(
                ErrorCode::InvalidRequest,
                "simulated native attribute mapping failure",
            )
            .with_id(id))
        })
        .expect_err("failure before the native effect must retire its reservation");

        let abandoned = abandoned.get().expect("reservation id is observed");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.id, Some(abandoned));
        assert_eq!(native_effect_calls.get(), 0);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.window_id("live"), Some(live_id));
        assert_eq!(
            registry
                .get(live_id)
                .expect("live fixture remains mapped")
                .instance
                .state(),
            &before
        );

        let insertion = registry
            .insert(Instance::new(state(abandoned)))
            .expect_err("an abandoned native-open identity remains issued");
        assert_eq!(insertion.code, ErrorCode::DuplicateIdentity);
        assert_eq!(insertion.id, Some(abandoned));
        assert_eq!(
            registry
                .reserve_id()
                .expect("the next allocation advances past the abandoned identity"),
            Id::from_u64(12)
        );
    }

    #[test]
    fn native_open_reservation_failure_after_state_preparation_retires_identity() {
        let live_id = Id::from_u64(20);
        let mut registry = Registry::new();
        registry
            .insert(Instance::new(state(live_id).named("live")))
            .expect("live fixture inserts");
        let before = registry
            .get(live_id)
            .expect("live fixture remains available")
            .instance
            .state()
            .clone();
        let abandoned = Cell::new(None);
        let native_effect_calls = Cell::new(0);

        let error = with_open_reservation(&mut registry, |id, _| -> Result<()> {
            native_effect_calls.set(native_effect_calls.get() + 1);
            abandoned.set(Some(id));
            let prepared_state = state(id);
            assert_eq!(prepared_state.id(), id);
            Err(Error::new(
                ErrorCode::InvalidRequest,
                "simulated post-create state preparation failure",
            )
            .with_id(id))
        })
        .expect_err("post-create failure must retire its reservation");

        let abandoned = abandoned.get().expect("reservation id is observed");
        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.id, Some(abandoned));
        assert_eq!(native_effect_calls.get(), 1);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.window_id("live"), Some(live_id));
        assert_eq!(
            registry
                .get(live_id)
                .expect("live fixture remains mapped")
                .instance
                .state(),
            &before
        );

        let insertion = registry
            .insert(Instance::new(state(abandoned)))
            .expect_err("a post-create failure leaves the identity permanently issued");
        assert_eq!(insertion.code, ErrorCode::DuplicateIdentity);
        assert_eq!(insertion.id, Some(abandoned));
        assert_eq!(
            registry
                .reserve_id()
                .expect("the next allocation advances past the abandoned identity"),
            Id::from_u64(22)
        );
    }

    #[test]
    fn native_metric_transition_rejects_mismatched_identity_without_delivery() {
        let id = Id::from_u64(1);
        let mismatched_id = Id::from_u64(2);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state(id)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        let before = runner
            .registry
            .get(id)
            .expect("test window is live")
            .instance
            .state()
            .clone();
        let transition = NativeEventTransition::resized(
            Metrics::from_physical_size(
                mismatched_id,
                PhysicalSize {
                    width: 800,
                    height: 600,
                },
                1.0,
            )
            .expect("test metrics are valid"),
        );

        let error = runner
            .apply_native_transition_state(id, transition)
            .expect_err("mismatched metrics must not commit or deliver a transition");

        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(error.id, Some(mismatched_id));
        assert_eq!(runner.registry.len(), 1);
        assert!(runner.registry.contains(id));
        assert!(!runner.registry.contains(mismatched_id));
        assert_eq!(
            runner
                .registry
                .get(id)
                .expect("test window remains live")
                .instance
                .state(),
            &before
        );
    }

    #[test]
    fn native_transition_patch_failure_preserves_state_without_delivery() {
        let id = Id::from_u64(1);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state(id)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);
        let before = runner
            .registry
            .get(id)
            .expect("test window is live")
            .instance
            .state()
            .clone();

        let error = runner
            .apply_native_transition_state(
                id,
                NativeEventTransition::moved(
                    id,
                    Point {
                        x: f64::NAN,
                        y: 2.0,
                    },
                ),
            )
            .expect_err("invalid native patch must not produce an event");

        assert_eq!(error.code, ErrorCode::InvalidRequest);
        assert_eq!(
            runner
                .registry
                .get(id)
                .expect("test window remains live")
                .instance
                .state(),
            &before
        );
    }

    #[test]
    fn native_callback_transaction_waits_for_pump_drain_and_discards_failed_local_work() {
        struct FailingHandler {
            calls: Rc<Cell<usize>>,
        }

        impl Handler for FailingHandler {
            fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
                self.calls.set(self.calls.get() + 1);
                event.context_mut().open(open("discarded"));
                event.context_mut().exit();
                Err(Error::new(ErrorCode::CommandFailed, "callback failure"))
            }
        }

        let calls = Rc::new(Cell::new(0));
        let mut runner = WinitRunner::from_loop(Loop::new(FailingHandler {
            calls: calls.clone(),
        }));
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .build(),
        );

        runner.deliver_event_to_pump_for_test(EventKind::Focused {
            id: Id::from_u64(1),
            focused: true,
        });

        assert_eq!(calls.get(), 0, "the pump must own callback invocation");

        runner.drain_pump_for_test();

        assert_eq!(calls.get(), 1);
        assert!(runner.registry.is_empty());
        let error = runner
            .into_terminal_result()
            .expect_err("the callback failure is the terminal result");
        assert_eq!(error.code, ErrorCode::CommandFailed);
        assert_eq!(error.message, "callback failure");
    }

    #[test]
    fn native_metric_and_scale_failures_reach_the_terminal_result_without_a_display() {
        let missing_id = Id::from_u64(7);
        let mut metric_runner = WinitRunner::from_loop(Loop::new(NoopHandler));

        metric_runner
            .apply_metrics_transition_for_test(missing_id, 800, 600, MetricsEvent::Resized)
            .expect("invalid metric transitions must become terminal pump failures");

        let metric_error = metric_runner
            .into_terminal_result()
            .expect_err("the original metric error must reach the loop result");
        assert_eq!(metric_error.code, ErrorCode::CommandFailed);
        assert_eq!(metric_error.id, Some(missing_id));
        assert_eq!(metric_error.message, "unknown window");

        let id = Id::from_u64(8);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state(id)))
            .expect("test identity inserts");
        let mut scale_runner = WinitRunner::from_loop(window_loop);

        scale_runner
            .apply_scale_factor_transition_for_test(id, f64::NAN)
            .expect("invalid scale transitions must become terminal pump failures");

        let scale_error = scale_runner
            .into_terminal_result()
            .expect_err("the original scale error must reach the loop result");
        assert_eq!(scale_error.code, ErrorCode::InvalidRequest);
        assert_eq!(
            scale_error.message,
            "observed scale factor must be finite and greater than zero"
        );
    }

    #[test]
    fn scale_two_creation_converts_outer_position_to_logical() {
        let metrics = metrics_from_winit_geometry(
            Id::from_u64(31),
            PhysicalSize {
                width: 640,
                height: 480,
            },
            Some(PhysicalPoint { x: 200, y: 100 }),
            None,
            2.0,
        )
        .expect("display-free native geometry is valid");

        assert_eq!(metrics.outer_position(), Some(Point { x: 100.0, y: 50.0 }));
    }

    #[test]
    fn scale_two_move_converts_outer_position_to_logical() {
        let id = Id::from_u64(32);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state_with_scale(id, 2.0)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);

        let event = runner
            .apply_native_transition_state(
                id,
                runner
                    .moved_transition(id, PhysicalPoint { x: 200, y: 100 })
                    .expect("native move transition builds"),
            )
            .expect("native move transition applies");

        assert_eq!(
            event,
            Some(EventKind::Moved {
                id,
                position: Point { x: 100.0, y: 50.0 },
            })
        );
        assert_eq!(
            runner
                .registry
                .get(id)
                .expect("window remains live")
                .metrics()
                .outer_position(),
            Some(Point { x: 100.0, y: 50.0 })
        );
    }

    #[test]
    fn moved_then_resized_preserves_canonical_logical_position() {
        let id = Id::from_u64(33);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state_with_scale(id, 2.0)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);

        runner
            .apply_native_transition_state(
                id,
                runner
                    .moved_transition(id, PhysicalPoint { x: 200, y: 100 })
                    .expect("native move transition builds"),
            )
            .expect("native move transition applies");
        let transition = runner
            .metrics_transition(id, 960, 540, MetricsEvent::Resized)
            .expect("native resize transition builds");
        runner
            .apply_native_transition_state(id, transition)
            .expect("native resize transition applies");

        assert_eq!(
            runner
                .registry
                .get(id)
                .expect("window remains live")
                .metrics()
                .outer_position(),
            Some(Point { x: 100.0, y: 50.0 })
        );
    }

    #[test]
    fn moved_then_scale_factor_change_preserves_canonical_logical_position() {
        let id = Id::from_u64(34);
        let mut window_loop = Loop::new(NoopHandler);
        window_loop
            .registry
            .insert(Instance::new(state_with_scale(id, 2.0)))
            .expect("test identity inserts");
        let mut runner = WinitRunner::from_loop(window_loop);

        runner
            .apply_native_transition_state(
                id,
                runner
                    .moved_transition(id, PhysicalPoint { x: 200, y: 100 })
                    .expect("native move transition builds"),
            )
            .expect("native move transition applies");
        let transition = runner
            .scale_factor_transition(id, 4.0)
            .expect("native scale transition builds");
        runner
            .apply_native_transition_state(id, transition)
            .expect("native scale transition applies");

        assert_eq!(
            runner
                .registry
                .get(id)
                .expect("window remains live")
                .metrics()
                .outer_position(),
            Some(Point { x: 100.0, y: 50.0 })
        );
    }

    #[test]
    fn native_idle_probe_is_not_called_after_terminal_failure() {
        struct IdleProbe {
            probes: Rc<Cell<usize>>,
        }

        impl Handler for IdleProbe {
            fn event(&mut self, _event: &mut Event<'_>) -> Result<()> {
                Err(Error::new(ErrorCode::CommandFailed, "callback failure"))
            }

            fn wants_idle(&self) -> bool {
                self.probes.set(self.probes.get() + 1);
                true
            }
        }

        let probes = Rc::new(Cell::new(0));
        let mut runner = WinitRunner::from_loop(Loop::new(IdleProbe {
            probes: probes.clone(),
        }));
        runner.resolve_capabilities_for_test(
            HostCapabilities::builder()
                .role(RoleKind::Root, CapabilitySupport::Supported)
                .build(),
        );
        runner.deliver_event_to_pump_for_test(EventKind::Focused {
            id: Id::from_u64(1),
            focused: true,
        });
        runner.drain_pump_for_test();

        assert_eq!(
            runner
                .idle_for_test()
                .expect("terminal idle probing must be a no-op"),
            None
        );
        assert_eq!(probes.get(), 0);
    }

    #[derive(Debug)]
    struct FakeNativeResource;

    fn fake_lease(id: Id) -> (Lease<FakeNativeResource>, Arc<ProxyQueue>) {
        let queue = Arc::new(ProxyQueue::new());
        let lease = Lease::new(
            id,
            FakeNativeResource,
            ReleaseNotifier::new(Proxy::with_queue(queue.clone())),
        );
        (lease, queue)
    }

    fn fake_raw_access(lease: &Lease<FakeNativeResource>) -> std::result::Result<(), HandleError> {
        lease.with_resource(|_| ())
    }

    fn released_id(queue: &ProxyQueue) -> Id {
        let UserEvent::HandleReleased(release) = queue
            .pop()
            .expect("the final fake native lease owner must notify the pump")
        else {
            panic!("the final fake native lease owner must notify the pump");
        };
        release.id()
    }

    struct NativeOwnershipPumpBackend {
        registry: Registry,
        ownership: NativeOwnership<FakeNativeResource>,
        closed: Vec<Id>,
    }

    impl NativeOwnershipPumpBackend {
        fn closing(id: Id, lease: &Lease<FakeNativeResource>) -> Self {
            let mut registry = Registry::new();
            registry
                .insert(Instance::new(state(id)))
                .expect("the display-free identity inserts");
            let instance = registry
                .begin_close(id)
                .expect("the display-free identity enters closing");
            lease.mark_closing();
            let mut ownership = NativeOwnership::new();
            assert!(ownership.begin_close(id, instance.state().clone(), Some(lease.downgrade())));
            drop(instance);

            Self {
                registry,
                ownership,
                closed: Vec::new(),
            }
        }
    }

    impl PumpBackend<()> for NativeOwnershipPumpBackend {
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
            _commands: Vec<NormalizedCommand>,
            _origin: WorkOrigin,
            _pump: &mut Pump<()>,
        ) -> Result<()> {
            Ok(())
        }

        fn apply_action(
            &mut self,
            _action: Action,
            _origin: WorkOrigin,
            _pump: &mut Pump<()>,
        ) -> Result<()> {
            Ok(())
        }

        fn begin_close(&mut self, _id: Id, _pump: &mut Pump<()>) -> Result<()> {
            Ok(())
        }

        fn complete_close(&mut self, id: Id, _pump: &mut Pump<()>) -> Result<()> {
            if let Some(state) = complete_native_close(&mut self.registry, &mut self.ownership, id)
            {
                self.closed.push(state.id());
            }
            Ok(())
        }
    }

    #[test]
    fn native_ownership_external_handle_delays_closed_until_lease_release() {
        let id = Id::from_u64(91);
        let (runner_lease, queue) = fake_lease(id);
        let external = runner_lease.clone();
        let mut backend = NativeOwnershipPumpBackend::closing(id, &runner_lease);
        let mut pump = Pump::new();

        drop(runner_lease);
        pump.enqueue_begin_close(id);
        pump.drain(&mut backend);
        assert!(backend.closed.is_empty());
        assert!(backend.ownership.contains(id));

        drop(external);
        pump.enqueue_close_completion(released_id(&queue));
        pump.drain(&mut backend);

        assert_eq!(backend.closed, [id]);
        assert!(!backend.registry.contains(id));
        assert!(!backend.ownership.contains(id));
    }

    #[test]
    fn native_ownership_closing_handle_retains_raw_access_until_last_drop() {
        let id = Id::from_u64(92);
        let (runner_lease, queue) = fake_lease(id);
        let external = runner_lease.clone();
        let backend = NativeOwnershipPumpBackend::closing(id, &runner_lease);

        drop(runner_lease);
        assert!(fake_raw_access(&external).is_ok());
        assert!(backend.ownership.contains(id));
        assert!(queue.pop().is_none());

        drop(external);
        let release = released_id(&queue);
        assert_eq!(release, id);
        assert!(backend.ownership.contains(id));
    }

    #[test]
    fn native_ownership_accepted_close_hides_and_rejects_commands() {
        let id = Id::from_u64(93);
        let mut runner = runner_with_live_window(NoopHandler, id);
        runner.draw.next.insert(id);
        runner.pending_draws.insert(id);
        runner.cursor_state.insert(id, Cursor::Hidden);
        runner.cursor_grab.insert(id, CursorGrab::Locked);

        request_close_for_test(&mut runner, id);

        assert!(!runner.registry.contains(id));
        assert!(!runner.draw.next.contains(&id));
        assert!(!runner.pending_draws.contains(&id));
        assert!(!runner.cursor_state.contains_key(&id));
        assert!(!runner.cursor_grab.contains_key(&id));
        let applied = runner.applied_commands_for_test().to_vec();
        runner.apply_proxy_command_for_test(
            normalize_command(Command::SetTitle {
                id,
                title: String::from("too late"),
            })
            .expect("the queued stale command normalizes"),
        );
        assert_eq!(runner.applied_commands_for_test(), applied);
    }

    #[test]
    fn native_ownership_destroy_command_and_accepted_close_share_cleanup_and_closed_order() {
        #[derive(Default)]
        struct Recorder {
            callbacks: Vec<&'static str>,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                self.callbacks.push("close");
                close.close();
                Ok(())
            }

            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                let id = closed.id();
                assert!(closed.context_mut().state(id).is_none());
                self.callbacks.push("closed");
                Ok(())
            }
        }

        let accepted_id = Id::from_u64(94);
        let mut accepted = runner_with_live_window(Recorder::default(), accepted_id);
        accepted.draw.next.insert(accepted_id);
        request_close_for_test(&mut accepted, accepted_id);

        let destroyed_id = Id::from_u64(95);
        let mut destroyed = runner_with_live_window(Recorder::default(), destroyed_id);
        destroyed.draw.next.insert(destroyed_id);
        destroyed.pump.enqueue_normalized_commands(vec![
            normalize_command(Command::Destroy { id: destroyed_id })
                .expect("destroy normalizes before the shared pump"),
        ]);
        destroyed.drain_pump_for_test();

        assert_eq!(accepted.handler.callbacks, ["close", "closed"]);
        assert_eq!(destroyed.handler.callbacks, ["closed"]);
        assert!(!accepted.draw.next.contains(&accepted_id));
        assert!(!destroyed.draw.next.contains(&destroyed_id));
        assert!(!accepted.registry.contains(accepted_id));
        assert!(!destroyed.registry.contains(destroyed_id));
    }

    #[test]
    fn native_ownership_destroy_invalidates_external_handle_before_closed() {
        let id = Id::from_u64(96);
        let (runner_lease, _queue) = fake_lease(id);
        let external = runner_lease.clone();
        let mut backend = NativeOwnershipPumpBackend::closing(id, &runner_lease);
        let mut pump = Pump::new();

        backend.ownership.mark_destroyed(id);
        assert!(matches!(
            fake_raw_access(&external),
            Err(HandleError::Unavailable)
        ));
        pump.enqueue_begin_close(id);
        pump.enqueue_close_completion(id);
        pump.drain(&mut backend);

        assert_eq!(backend.closed, [id]);
        drop(runner_lease);
        drop(external);
    }

    #[test]
    fn native_ownership_destroy_racing_last_drop_closes_once() {
        let id = Id::from_u64(97);
        let (runner_lease, queue) = fake_lease(id);
        let external = runner_lease.clone();
        let mut backend = NativeOwnershipPumpBackend::closing(id, &runner_lease);
        let mut pump = Pump::new();

        backend.ownership.mark_destroyed(id);
        pump.enqueue_begin_close(id);
        pump.enqueue_close_completion(id);
        drop(runner_lease);
        drop(external);
        pump.enqueue_close_completion(released_id(&queue));
        pump.drain(&mut backend);

        assert_eq!(backend.closed, [id]);
    }

    #[test]
    fn native_ownership_loop_exit_with_external_handle_returns_outstanding_handle() {
        let id = Id::from_u64(98);
        let (runner_lease, _queue) = fake_lease(id);
        let external = runner_lease.clone();
        let mut ownership = NativeOwnership::new();
        ownership.remember_terminal_lease(runner_lease.downgrade());
        runner_lease.mark_loop_exited();
        drop(runner_lease);

        let error = terminal_result(Ok(()), Ok(()), ownership.has_outstanding_leases())
            .expect_err("an external native lease prevents a successful terminal result");
        assert_eq!(error.code, ErrorCode::OutstandingHandle);

        drop(external);
    }

    #[test]
    fn native_ownership_loop_exit_invalidates_external_raw_access() {
        let id = Id::from_u64(99);
        let (runner_lease, _queue) = fake_lease(id);
        let external = runner_lease.clone();
        let mut ownership = NativeOwnership::new();
        ownership.begin_close(id, state(id), Some(runner_lease.downgrade()));

        ownership.mark_loop_exited();

        assert!(matches!(
            fake_raw_access(&external),
            Err(HandleError::Unavailable)
        ));
        drop(runner_lease);
        drop(external);
    }

    #[test]
    fn native_ownership_nested_closed_commands_run_after_coherent_close() {
        #[derive(Default)]
        struct Recorder {
            callbacks: Vec<String>,
        }

        impl Handler for Recorder {
            fn close(&mut self, close: &mut Close<'_>) -> Result<()> {
                self.callbacks.push(String::from("close"));
                close.close();
                Ok(())
            }

            fn closed(&mut self, closed: &mut Closed<'_>) -> Result<()> {
                let id = closed.id();
                assert!(closed.context_mut().state(id).is_none());
                self.callbacks.push(String::from("closed"));
                closed.context_mut().open(open("replacement"));
                Ok(())
            }

            fn event(&mut self, event: &mut Event<'_>) -> Result<()> {
                if let EventKind::Created(state) = event.event() {
                    self.callbacks
                        .push(format!("created:{}", state.name().unwrap()));
                }
                Ok(())
            }
        }

        let id = Id::from_u64(100);
        let mut runner = runner_with_live_window(Recorder::default(), id);

        request_close_for_test(&mut runner, id);

        assert_eq!(
            runner.handler.callbacks,
            ["close", "closed", "created:replacement"]
        );
        assert_eq!(
            runner.registry.window_id("replacement"),
            Some(Id::from_u64(101))
        );
    }
}
