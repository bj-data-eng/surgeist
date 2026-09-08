#[cfg(feature = "accessibility")]
use super::CapabilitySupport;
use super::{
    CapabilityDecision, CapabilityKind, Command, CommandKind, Error, ErrorCode, Fullscreen,
    HostCapabilities, Id, ImeCapability, ImeConfig, ImeHint, ImePurpose, ImeRequest, Metrics,
    PhysicalSize, Rect, Registry, Result, Size, WindowOperation, WindowRequest, WindowSnapshot,
    descriptor::WindowSnapshotSeed,
    normalization::{NormalizedCommand, normalize_command},
    pump::WorkOrigin,
    registry::{InnerSizeBounds, LifecycleState, RegistryProjection},
};
use std::collections::HashMap;
use std::fmt;

/// Opaque resolved host command for a backend applicator.
///
/// This is not a second authored command surface: [`Command`] preserves app
/// input, and normalization produces its crate-private canonical form. The
/// standalone [`Self::from_command`] constructor performs intrinsic
/// normalization and capability evaluation only; it has no registry and does
/// not preflight runtime lifecycle state. Crate-private staged and batch
/// planning performs virtual registry and lifecycle preflight before a backend
/// consumes those plans. Its public accessors expose diagnostics about the
/// resolved command.
#[derive(Clone, PartialEq)]
pub struct HostCommandPlan {
    payload: PlannedCommandPayload,
}

impl fmt::Debug for HostCommandPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostCommandPlan")
            .field("kind", &self.kind())
            .field("target", &self.target())
            .field("capability_decisions", &self.capability_decisions())
            .finish()
    }
}

#[derive(Clone, PartialEq)]
struct PlannedCommandPayload {
    command: NormalizedCommand,
    ime_request: Option<ResolvedImeRequest>,
    #[cfg(feature = "accessibility")]
    accessibility_update: Option<ResolvedAccessibilityUpdate>,
    #[cfg(feature = "accessibility")]
    accessibility_open_phase: Option<AccessibilityPhase>,
    kind: CommandKind,
    target: Option<Id>,
    failure_id: Option<Id>,
    capability_decisions: Vec<CapabilityDecision>,
    open_id: Option<Id>,
    resolved_size: Option<Size>,
}

/// A capability-resolved IME purpose that winit can represent exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResolvedImePurpose {
    Normal,
    Password,
    Terminal,
}

/// One capability-resolved IME configuration shared by backend applicators.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedImeConfig {
    pub(crate) purpose: ResolvedImePurpose,
    pub(crate) hint: ImeHint,
    pub(crate) cursor_area: Option<Rect>,
}

/// One capability-resolved IME request shared by backend applicators.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ResolvedImeRequest {
    Disable,
    Enable(ResolvedImeConfig),
    Update(ResolvedImeConfig),
    Restart(ResolvedImeConfig),
}

/// The crate-private lifecycle of one window's AccessKit tree.
#[cfg(feature = "accessibility")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccessibilityPhase {
    Absent,
    WaitingForInitialTree,
    Active,
}

/// A tree update whose required phase transition has been preflighted.
#[cfg(feature = "accessibility")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedAccessibilityUpdate {
    pub(crate) phase: AccessibilityPhase,
}

impl ResolvedImeRequest {
    #[cfg(test)]
    pub(crate) const fn has_none_hint(&self) -> bool {
        match self {
            Self::Disable => false,
            Self::Enable(config) | Self::Update(config) | Self::Restart(config) => {
                matches!(config.hint, ImeHint::None)
            }
        }
    }

    pub(crate) fn as_authored(&self) -> ImeRequest {
        match self {
            Self::Disable => ImeRequest::Disable,
            Self::Enable(config) => ImeRequest::Enable(config.as_authored()),
            Self::Update(config) => ImeRequest::Update(config.as_authored()),
            Self::Restart(config) => ImeRequest::Restart(config.as_authored()),
        }
    }
}

impl ResolvedImeConfig {
    fn as_authored(&self) -> ImeConfig {
        ImeConfig {
            purpose: match self.purpose {
                ResolvedImePurpose::Normal => ImePurpose::Normal,
                ResolvedImePurpose::Password => ImePurpose::Password,
                ResolvedImePurpose::Terminal => ImePurpose::Terminal,
            },
            hint: self.hint,
            cursor_area: self.cursor_area,
            surrounding_text: None,
        }
    }
}

impl HostCommandPlan {
    /// Normalizes an authored command and evaluates it against a host report.
    ///
    /// This standalone constructor has no registry, so it performs intrinsic
    /// normalization and capability evaluation only. It does not preflight
    /// lifecycle or other runtime state; crate-private staged and batch planning
    /// performs that virtual registry preflight. Capability rejection happens
    /// before backend effects. Successful planning records the host's decisions
    /// and leaves application to the backend.
    pub fn from_command(command: Command, capabilities: &HostCapabilities) -> Result<Self> {
        let command = normalize_command(command)?;
        Self::from_normalized(command, capabilities)
    }

    pub(crate) fn from_normalized(
        command: NormalizedCommand,
        capabilities: &HostCapabilities,
    ) -> Result<Self> {
        let capability_decisions = capability_decisions(&command, capabilities);
        let kind = command.command().kind();
        let target = command.command().target();
        require_capability_decisions(&capability_decisions, capabilities)
            .map_err(|error| command_error(error, kind, target))?;
        let ime_request = match command.command() {
            Command::SetIme { request, .. } => Some(
                resolve_ime_request(request).map_err(|error| command_error(error, kind, target))?,
            ),
            _ => None,
        };

        Ok(Self {
            payload: PlannedCommandPayload {
                kind,
                target,
                failure_id: target,
                command,
                ime_request,
                #[cfg(feature = "accessibility")]
                accessibility_update: None,
                #[cfg(feature = "accessibility")]
                accessibility_open_phase: None,
                capability_decisions,
                open_id: None,
                resolved_size: None,
            },
        })
    }

    /// Returns the stable classification of the originating authored command.
    #[must_use]
    pub const fn kind(&self) -> CommandKind {
        self.payload.kind
    }

    /// Returns the target ID authored by the originating command, when it has one.
    ///
    /// This is not a backend-resolved identity.
    #[must_use]
    pub const fn target(&self) -> Option<Id> {
        self.payload.target
    }

    /// Returns capability decisions recorded during planning.
    #[must_use]
    pub fn capability_decisions(&self) -> &[CapabilityDecision] {
        &self.payload.capability_decisions
    }
}

#[cfg(test)]
pub(crate) fn take_planned_command(plan: HostCommandPlan) -> NormalizedCommand {
    plan.payload.command
}

#[cfg(test)]
pub(crate) fn take_planned_ime_request(plan: HostCommandPlan) -> Option<ResolvedImeRequest> {
    plan.payload.ime_request
}

#[cfg(test)]
pub(crate) fn take_single_planned_command(batch: BatchPlan) -> HostCommandPlan {
    let mut commands = batch.commands;
    assert_eq!(
        commands.len(),
        1,
        "single-command planner test seams must receive exactly one plan"
    );
    commands.pop().expect("single-command plan must exist")
}

pub(crate) struct ResolvedHostCommand {
    pub(crate) command: NormalizedCommand,
    pub(crate) ime_request: Option<ResolvedImeRequest>,
    #[cfg(feature = "accessibility")]
    pub(crate) accessibility_update: Option<ResolvedAccessibilityUpdate>,
    #[cfg(feature = "accessibility")]
    pub(crate) accessibility_open_phase: Option<AccessibilityPhase>,
    pub(crate) open_id: Option<Id>,
    pub(crate) resolved_size: Option<Size>,
}

/// The backend-observed outcome of one resolved inner-size request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SizeApplicationResult {
    Immediate(PhysicalSize),
    Deferred,
}

pub(crate) fn take_resolved_host_command(plan: HostCommandPlan) -> ResolvedHostCommand {
    ResolvedHostCommand {
        command: plan.payload.command,
        ime_request: plan.payload.ime_request,
        #[cfg(feature = "accessibility")]
        accessibility_update: plan.payload.accessibility_update,
        #[cfg(feature = "accessibility")]
        accessibility_open_phase: plan.payload.accessibility_open_phase,
        open_id: plan.payload.open_id,
        resolved_size: plan.payload.resolved_size,
    }
}

/// Crate-private ordered backend input. Only this type may be drained by a host.
pub(crate) struct BatchPlan {
    commands: Vec<HostCommandPlan>,
}

impl BatchPlan {
    fn new(commands: Vec<HostCommandPlan>) -> Self {
        Self { commands }
    }
}

/// A preflighted prefix whose virtual runtime projection continues into later work.
pub(crate) struct StagedBatchPlan {
    planner: CommandPlanner,
    commands: Vec<HostCommandPlan>,
}

impl StagedBatchPlan {
    pub(crate) fn finish_normalized_batch(
        mut self,
        commands: Vec<NormalizedCommand>,
    ) -> Result<BatchPlan> {
        self.commands
            .extend(self.planner.plan_normalized_commands(commands)?);
        Ok(BatchPlan::new(self.commands))
    }
}

/// Applies an already-preflighted batch and preserves a successfully applied prefix.
pub(crate) fn apply_batch_plan(
    batch: BatchPlan,
    mut apply: impl FnMut(HostCommandPlan) -> Result<()>,
) -> Result<()> {
    for (completed_prefix, plan) in batch.commands.into_iter().enumerate() {
        let kind = plan.kind();
        let failure_id = plan.payload.failure_id;
        if let Err(source) = apply(plan) {
            let mut error = Error::new(
                ErrorCode::CommandBatchFailed,
                "backend command application failed after a committed prefix",
            )
            .with_command_kind(kind)
            .with_completed_prefix(completed_prefix)
            .with_source(source);
            if let Some(id) = failure_id {
                error = error.with_id(id);
            }
            return Err(error);
        }
    }
    Ok(())
}

pub(crate) struct CommandPlanner {
    capabilities: HostCapabilities,
    runtime: VirtualRuntime,
}

impl CommandPlanner {
    pub(crate) fn from_registry(capabilities: HostCapabilities, registry: &Registry) -> Self {
        Self {
            capabilities,
            runtime: VirtualRuntime::from_registry(registry),
        }
    }

    pub(crate) fn plan_normalized_batch(
        mut self,
        commands: Vec<NormalizedCommand>,
    ) -> Result<BatchPlan> {
        Ok(BatchPlan::new(self.plan_normalized_commands(commands)?))
    }

    pub(crate) fn stage_normalized_batch(
        mut self,
        commands: Vec<NormalizedCommand>,
    ) -> Result<StagedBatchPlan> {
        let commands = self.plan_normalized_commands(commands)?;
        Ok(StagedBatchPlan {
            planner: self,
            commands,
        })
    }

    fn plan_normalized_commands(
        &mut self,
        commands: Vec<NormalizedCommand>,
    ) -> Result<Vec<HostCommandPlan>> {
        let mut plans = Vec::with_capacity(commands.len());
        for command in commands {
            let resolved = self.runtime.resolve(command, &self.capabilities)?;
            let mut plan = HostCommandPlan::from_normalized(resolved.command, &self.capabilities)?;
            plan.payload.open_id = resolved.open_id;
            plan.payload.failure_id = resolved.open_id.or(plan.payload.target);
            plan.payload.resolved_size = resolved.resolved_size;
            #[cfg(feature = "accessibility")]
            {
                plan.payload.accessibility_update = resolved.accessibility_update;
                plan.payload.accessibility_open_phase = resolved.accessibility_open_phase;
            }
            plans.push(plan);
        }
        Ok(plans)
    }
}

fn command_error(mut error: Error, kind: CommandKind, target: Option<Id>) -> Error {
    error = error.with_command_kind(kind);
    if let Some(id) = target {
        error = error.with_id(id);
    }
    error
}

/// Applies the event-loop ingress policy for one optional target.
///
/// Strict work retains lifecycle diagnostics during planning. Queued proxy work
/// treats a target that is no longer live as an idempotent no-op before planning.
#[must_use]
pub(crate) fn accepts_work_target(
    origin: WorkOrigin,
    registry: &Registry,
    target: Option<Id>,
) -> bool {
    matches!(origin, WorkOrigin::Strict)
        || match target {
            Some(id) => registry.contains(id),
            None => true,
        }
}

/// Classifies one action target at backend application time.
///
/// Strict callback work retains lifecycle diagnostics, while accepted queued work
/// remains an idempotent no-op after its target leaves live lookup.
pub(crate) fn accepts_action_target(
    origin: WorkOrigin,
    registry: &Registry,
    target: Option<Id>,
) -> Result<bool> {
    let Some(id) = target else {
        return Ok(true);
    };
    if registry.contains(id) {
        return Ok(true);
    }
    if matches!(origin, WorkOrigin::Queued) {
        return Ok(false);
    }

    let lifecycle = registry.planning_projection().lifecycle;
    let (code, message) = match lifecycle.get(&id).copied() {
        Some(LifecycleState::Closing) => (ErrorCode::WindowClosing, "target window is closing"),
        Some(LifecycleState::Reserved | LifecycleState::Live | LifecycleState::Closed) | None => {
            (ErrorCode::StaleWindow, "target window is stale")
        }
    };
    Err(Error::new(code, message).with_id(id))
}

struct ResolvedCommand {
    command: NormalizedCommand,
    open_id: Option<Id>,
    resolved_size: Option<Size>,
    #[cfg(feature = "accessibility")]
    accessibility_update: Option<ResolvedAccessibilityUpdate>,
    #[cfg(feature = "accessibility")]
    accessibility_open_phase: Option<AccessibilityPhase>,
}

struct VirtualRuntime {
    next: u64,
    lifecycle: HashMap<Id, LifecycleState>,
    live: HashMap<Id, VirtualWindow>,
    #[cfg(feature = "accessibility")]
    accessibility: HashMap<Id, AccessibilityPhase>,
}

struct VirtualWindow {
    snapshot: WindowSnapshot,
    inner_size_bounds: InnerSizeBounds,
}

impl VirtualRuntime {
    fn from_registry(registry: &Registry) -> Self {
        let RegistryProjection {
            next,
            lifecycle,
            live,
            #[cfg(feature = "accessibility")]
            accessibility,
        } = registry.planning_projection();
        Self {
            next,
            lifecycle,
            live: live
                .into_iter()
                .map(|(id, instance)| {
                    (
                        id,
                        VirtualWindow {
                            snapshot: instance.state,
                            inner_size_bounds: instance.inner_size_bounds,
                        },
                    )
                })
                .collect(),
            #[cfg(feature = "accessibility")]
            accessibility,
        }
    }

    fn resolve(
        &mut self,
        command: NormalizedCommand,
        _capabilities: &HostCapabilities,
    ) -> Result<ResolvedCommand> {
        let command = command.into_command();
        let kind = command.kind();
        let target = command.target();
        let mut resolved_size = None;
        let mut open_id = None;
        #[cfg(feature = "accessibility")]
        let mut accessibility_update = None;
        #[cfg(feature = "accessibility")]
        let mut accessibility_open_phase = None;

        let command = match command {
            Command::Open { request } => {
                let id = self.allocate_open_id(kind)?;
                #[cfg(feature = "accessibility")]
                {
                    accessibility_open_phase =
                        self.insert_open(id, &request, kind, _capabilities)?;
                }
                #[cfg(not(feature = "accessibility"))]
                self.insert_open(id, &request, kind)?;
                open_id = Some(id);
                Command::Open { request }
            }
            Command::SetTitle { id, title } => {
                self.require_live_mut(id, kind)?
                    .snapshot
                    .set_title(title.clone());
                Command::SetTitle { id, title }
            }
            Command::SetPosition { id, position } => {
                self.require_live_mut(id, kind)?
                    .snapshot
                    .set_position(Some(position))
                    .map_err(|error| command_error(error, kind, Some(id)))?;
                Command::SetPosition { id, position }
            }
            Command::SetVisible { id, visible } => {
                self.require_live_mut(id, kind)?
                    .snapshot
                    .set_visible(Some(visible));
                Command::SetVisible { id, visible }
            }
            Command::SetInnerSize { id, size } => {
                let window = self.require_live_mut(id, kind)?;
                let size = clamp_inner_size(size, window.inner_size_bounds);
                set_snapshot_size(&mut window.snapshot, size)
                    .map_err(|error| command_error(error, kind, Some(id)))?;
                Command::SetInnerSize { id, size }
            }
            Command::SetMinInnerSize { id, size } => {
                let window = self.require_live_mut(id, kind)?;
                let bounds = InnerSizeBounds::new(size, window.inner_size_bounds.maximum);
                validate_inner_size_bounds(bounds, kind, id)?;
                window.inner_size_bounds = bounds;
                let size = clamp_inner_size(window.snapshot.metrics().logical_size(), bounds);
                if size != window.snapshot.metrics().logical_size() {
                    set_snapshot_size(&mut window.snapshot, size)
                        .map_err(|error| command_error(error, kind, Some(id)))?;
                    resolved_size = Some(size);
                }
                Command::SetMinInnerSize {
                    id,
                    size: bounds.minimum,
                }
            }
            Command::SetMaxInnerSize { id, size } => {
                let window = self.require_live_mut(id, kind)?;
                let bounds = InnerSizeBounds::new(window.inner_size_bounds.minimum, size);
                validate_inner_size_bounds(bounds, kind, id)?;
                window.inner_size_bounds = bounds;
                let size = clamp_inner_size(window.snapshot.metrics().logical_size(), bounds);
                if size != window.snapshot.metrics().logical_size() {
                    set_snapshot_size(&mut window.snapshot, size)
                        .map_err(|error| command_error(error, kind, Some(id)))?;
                    resolved_size = Some(size);
                }
                Command::SetMaxInnerSize {
                    id,
                    size: bounds.maximum,
                }
            }
            Command::SetFullscreen { id, fullscreen } => {
                self.require_live_mut(id, kind)?
                    .snapshot
                    .set_fullscreen(!matches!(fullscreen, Fullscreen::None));
                Command::SetFullscreen { id, fullscreen }
            }
            Command::SetTheme { id, theme } => {
                self.require_live_mut(id, kind)?.snapshot.set_theme(theme);
                Command::SetTheme { id, theme }
            }
            Command::Destroy { id } => {
                self.require_live(id, kind)?;
                self.live.remove(&id);
                self.lifecycle.insert(id, LifecycleState::Closing);
                #[cfg(feature = "accessibility")]
                self.accessibility.remove(&id);
                Command::Destroy { id }
            }
            #[cfg(feature = "accessibility")]
            Command::UpdateAccessibility { id, update } => {
                self.require_live(id, kind)?;
                _capabilities
                    .require_accessibility()
                    .map_err(|error| command_error(error, kind, Some(id)))?;
                let phase = self.accessibility.get_mut(&id).ok_or_else(|| {
                    command_error(
                        Error::new(
                            ErrorCode::InvalidRequest,
                            "accessibility phase is unavailable for the live window",
                        )
                        .with_id(id),
                        kind,
                        Some(id),
                    )
                })?;
                match phase {
                    AccessibilityPhase::WaitingForInitialTree if update.tree.is_some() => {
                        *phase = AccessibilityPhase::Active;
                    }
                    AccessibilityPhase::Active => {}
                    AccessibilityPhase::Absent | AccessibilityPhase::WaitingForInitialTree => {
                        return Err(command_error(
                            Error::new(
                                ErrorCode::InvalidRequest,
                                "accessibility update requires an initial tree",
                            )
                            .with_id(id),
                            kind,
                            Some(id),
                        ));
                    }
                }
                accessibility_update = Some(ResolvedAccessibilityUpdate { phase: *phase });
                Command::UpdateAccessibility { id, update }
            }
            command => {
                let id = target.expect("all remaining commands target one live window");
                self.require_live(id, kind)?;
                command
            }
        };

        Ok(ResolvedCommand {
            command: normalize_command(command)?,
            open_id,
            resolved_size,
            #[cfg(feature = "accessibility")]
            accessibility_update,
            #[cfg(feature = "accessibility")]
            accessibility_open_phase,
        })
    }

    fn allocate_open_id(&mut self, kind: CommandKind) -> Result<Id> {
        let next = self.next.checked_add(1).ok_or_else(|| {
            command_error(
                Error::new(
                    ErrorCode::IdentityExhausted,
                    "no runtime window identities remain",
                ),
                kind,
                None,
            )
        })?;
        let id = Id::from_u64(next);
        if self.lifecycle.contains_key(&id) {
            return Err(command_error(
                Error::new(
                    ErrorCode::DuplicateIdentity,
                    "virtual allocator selected an issued identity",
                )
                .with_id(id),
                kind,
                None,
            ));
        }
        self.next = next;
        self.lifecycle.insert(id, LifecycleState::Reserved);
        Ok(id)
    }

    #[cfg(feature = "accessibility")]
    fn insert_open(
        &mut self,
        id: Id,
        request: &WindowRequest,
        kind: CommandKind,
        capabilities: &HostCapabilities,
    ) -> Result<Option<AccessibilityPhase>> {
        self.insert_open_without_accessibility(id, request, kind)?;
        let phase = (capabilities.accessibility() != CapabilitySupport::Unsupported)
            .then_some(AccessibilityPhase::Absent);
        if let Some(phase) = phase {
            self.accessibility.insert(id, phase);
        }
        Ok(phase)
    }

    #[cfg(not(feature = "accessibility"))]
    fn insert_open(&mut self, id: Id, request: &WindowRequest, kind: CommandKind) -> Result<()> {
        self.insert_open_without_accessibility(id, request, kind)
    }

    fn insert_open_without_accessibility(
        &mut self,
        id: Id,
        request: &WindowRequest,
        kind: CommandKind,
    ) -> Result<()> {
        if let Some(name) = request.name()
            && self
                .live
                .values()
                .any(|window| window.snapshot.name() == Some(name))
        {
            return Err(command_error(
                Error::new(ErrorCode::DuplicateIdentity, "window name is already live"),
                kind,
                None,
            ));
        }

        let snapshot = projected_snapshot_from_request(id, request)
            .map_err(|error| command_error(error, kind, None))?;
        self.live.insert(
            id,
            VirtualWindow {
                snapshot,
                inner_size_bounds: InnerSizeBounds::new(
                    request.min_inner_size(),
                    request.max_inner_size(),
                ),
            },
        );
        self.lifecycle.insert(id, LifecycleState::Live);
        Ok(())
    }

    fn require_live(&self, id: Id, kind: CommandKind) -> Result<()> {
        self.live
            .contains_key(&id)
            .then_some(())
            .ok_or_else(|| self.target_lifecycle_error(id, kind))
    }

    fn require_live_mut(&mut self, id: Id, kind: CommandKind) -> Result<&mut VirtualWindow> {
        if !self.live.contains_key(&id) {
            return Err(self.target_lifecycle_error(id, kind));
        }
        Ok(self
            .live
            .get_mut(&id)
            .expect("a checked live target remains available during planning"))
    }

    fn target_lifecycle_error(&self, id: Id, kind: CommandKind) -> Error {
        let (code, message) = match self.lifecycle.get(&id) {
            Some(LifecycleState::Closing) => {
                (ErrorCode::WindowClosing, "command target is closing")
            }
            Some(LifecycleState::Reserved | LifecycleState::Closed) | None => {
                (ErrorCode::StaleWindow, "command target is stale")
            }
            Some(LifecycleState::Live) => {
                unreachable!("a live lifecycle entry must have a virtual live window")
            }
        };
        command_error(Error::new(code, message).with_id(id), kind, Some(id))
    }
}

fn validate_inner_size_bounds(bounds: InnerSizeBounds, kind: CommandKind, id: Id) -> Result<()> {
    if let (Some(minimum), Some(maximum)) = (bounds.minimum, bounds.maximum)
        && (minimum.width > maximum.width || minimum.height > maximum.height)
    {
        return Err(command_error(
            Error::new(
                ErrorCode::InvalidRequest,
                "minimum inner size must not exceed maximum inner size",
            )
            .with_id(id),
            kind,
            Some(id),
        ));
    }
    Ok(())
}

fn clamp_inner_size(size: Size, bounds: InnerSizeBounds) -> Size {
    let width = bounds
        .minimum
        .map_or(size.width, |minimum| size.width.max(minimum.width));
    let height = bounds
        .minimum
        .map_or(size.height, |minimum| size.height.max(minimum.height));
    Size {
        width: bounds
            .maximum
            .map_or(width, |maximum| width.min(maximum.width)),
        height: bounds
            .maximum
            .map_or(height, |maximum| height.min(maximum.height)),
    }
}

pub(crate) fn set_snapshot_size(snapshot: &mut WindowSnapshot, size: Size) -> Result<()> {
    let metrics = snapshot.metrics();
    let scale = metrics.scale_factor();
    let metrics = Metrics::from_physical_size(
        snapshot.id(),
        PhysicalSize {
            width: (size.width * scale).round().max(0.0) as u32,
            height: (size.height * scale).round().max(0.0) as u32,
        },
        scale,
    )?
    .with_outer_geometry(metrics.outer_position(), metrics.outer_size())?;
    snapshot.set_metrics(metrics)
}

pub(crate) fn projected_snapshot_from_request(
    id: Id,
    request: &WindowRequest,
) -> Result<WindowSnapshot> {
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
    Ok(WindowSnapshot::from_seed(WindowSnapshotSeed {
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
    }))
}

fn capability_decisions(
    command: &NormalizedCommand,
    capabilities: &HostCapabilities,
) -> Vec<CapabilityDecision> {
    match command.command() {
        Command::Open { request } => open_capability_decisions(request, capabilities),
        Command::SetFullscreen { fullscreen, .. } => vec![CapabilityDecision::new(
            CapabilityKind::Fullscreen(fullscreen.mode()),
            capabilities.fullscreen(fullscreen.mode()),
        )],
        Command::SetCursor { cursor, .. } => vec![CapabilityDecision::new(
            CapabilityKind::Cursor(cursor.capability()),
            capabilities.cursor(cursor.capability()),
        )],
        Command::SetTitle { .. } => {
            window_capability_decision(WindowOperation::SetTitle, capabilities)
        }
        Command::SetPosition { .. } => {
            window_capability_decision(WindowOperation::SetOuterPosition, capabilities)
        }
        Command::SetVisible { .. } => {
            window_capability_decision(WindowOperation::SetVisible, capabilities)
        }
        Command::SetResizable { .. } => {
            window_capability_decision(WindowOperation::SetResizable, capabilities)
        }
        Command::SetControls { .. } => {
            window_capability_decision(WindowOperation::SetControls, capabilities)
        }
        Command::SetDecorations { .. } => {
            window_capability_decision(WindowOperation::SetDecorations, capabilities)
        }
        Command::SetTransparent { .. } => {
            window_capability_decision(WindowOperation::SetTransparent, capabilities)
        }
        Command::SetInnerSize { .. } => {
            window_capability_decision(WindowOperation::RequestInnerSize, capabilities)
        }
        Command::SetMinInnerSize { .. } | Command::SetMaxInnerSize { .. } => {
            window_capability_decision(WindowOperation::SetInnerSizeBounds, capabilities)
        }
        Command::SetLevel { .. } => {
            window_capability_decision(WindowOperation::SetLevel, capabilities)
        }
        Command::SetTheme { theme: Some(_), .. } => {
            window_capability_decision(WindowOperation::SetExplicitTheme, capabilities)
        }
        Command::SetTheme { theme: None, .. } => {
            window_capability_decision(WindowOperation::ResetTheme, capabilities)
        }
        Command::SetCursorGrab { grab, .. } => vec![CapabilityDecision::new(
            CapabilityKind::CursorGrab(*grab),
            capabilities.cursor_grab(*grab),
        )],
        Command::SetIme { request, .. } => ime_capability_decisions(request, capabilities),
        #[cfg(feature = "accessibility")]
        Command::UpdateAccessibility { .. } => vec![CapabilityDecision::new(
            CapabilityKind::Accessibility,
            capabilities.accessibility(),
        )],
        Command::RequestUserAttention { .. } => {
            window_capability_decision(WindowOperation::RequestUserAttention, capabilities)
        }
        Command::RequestDraw { .. } => {
            window_capability_decision(WindowOperation::RequestDraw, capabilities)
        }
        Command::Destroy { .. } => {
            window_capability_decision(WindowOperation::Destroy, capabilities)
        }
    }
}

fn open_capability_decisions(
    request: &WindowRequest,
    capabilities: &HostCapabilities,
) -> Vec<CapabilityDecision> {
    let default = WindowRequest::default();
    let mut decisions = vec![
        CapabilityDecision::new(
            CapabilityKind::Role(request.role().kind()),
            capabilities.role(request.role().kind()),
        ),
        CapabilityDecision::new(
            CapabilityKind::Fullscreen(request.fullscreen().mode()),
            capabilities.fullscreen(request.fullscreen().mode()),
        ),
    ];
    let mut record = |operation| {
        decisions.push(CapabilityDecision::new(
            CapabilityKind::Window(operation),
            capabilities.window(operation),
        ));
    };

    if request.title() != default.title() {
        record(WindowOperation::SetTitle);
    }
    if request.position() != default.position() {
        record(WindowOperation::SetOuterPosition);
    }
    if request.visible() != default.visible() {
        record(WindowOperation::SetVisible);
    }
    if request.resizable() != default.resizable() {
        record(WindowOperation::SetResizable);
    }
    if request.controls() != default.controls() {
        record(WindowOperation::SetControls);
    }
    if request.decorations() != default.decorations() {
        record(WindowOperation::SetDecorations);
    }
    if request.transparent() != default.transparent() {
        record(WindowOperation::InitialTransparency);
    }
    if request.inner_size() != default.inner_size() {
        record(WindowOperation::RequestInnerSize);
    }
    if request.min_inner_size() != default.min_inner_size()
        || request.max_inner_size() != default.max_inner_size()
    {
        record(WindowOperation::SetInnerSizeBounds);
    }
    if request.level() != default.level() {
        record(WindowOperation::SetLevel);
    }
    if request.theme() != default.theme() {
        record(WindowOperation::SetExplicitTheme);
    }

    decisions
}

fn window_capability_decision(
    operation: WindowOperation,
    capabilities: &HostCapabilities,
) -> Vec<CapabilityDecision> {
    vec![CapabilityDecision::new(
        CapabilityKind::Window(operation),
        capabilities.window(operation),
    )]
}

fn ime_capability_decisions(
    request: &ImeRequest,
    capabilities: &HostCapabilities,
) -> Vec<CapabilityDecision> {
    let mut decisions = Vec::new();
    let mut record = |capability| {
        decisions.push(CapabilityDecision::new(
            CapabilityKind::Ime(capability),
            capabilities.ime(capability),
        ));
    };
    let (requires_enablement, config) = match request {
        ImeRequest::Disable => (true, None),
        ImeRequest::Enable(config) | ImeRequest::Restart(config) => (true, Some(config)),
        ImeRequest::Update(config) => (false, Some(config)),
    };

    if requires_enablement {
        record(ImeCapability::Enablement);
    }
    if let Some(config) = config {
        record(ImeCapability::Purpose(config.purpose));
        record(ImeCapability::Hint(config.hint));
        if config.cursor_area.is_some() {
            record(ImeCapability::CursorArea);
        }
        if config.surrounding_text.is_some() {
            record(ImeCapability::SurroundingText);
        }
    }

    decisions
}

fn resolve_ime_request(request: &ImeRequest) -> Result<ResolvedImeRequest> {
    match request {
        ImeRequest::Disable => Ok(ResolvedImeRequest::Disable),
        ImeRequest::Enable(config) => Ok(ResolvedImeRequest::Enable(resolve_ime_config(config)?)),
        ImeRequest::Update(config) => Ok(ResolvedImeRequest::Update(resolve_ime_config(config)?)),
        ImeRequest::Restart(config) => Ok(ResolvedImeRequest::Restart(resolve_ime_config(config)?)),
    }
}

fn resolve_ime_config(config: &ImeConfig) -> Result<ResolvedImeConfig> {
    let purpose = match config.purpose {
        ImePurpose::Normal => ResolvedImePurpose::Normal,
        ImePurpose::Password => ResolvedImePurpose::Password,
        ImePurpose::Terminal => ResolvedImePurpose::Terminal,
        ImePurpose::Number | ImePurpose::Email | ImePurpose::Url => {
            return Err(Error::new(
                ErrorCode::ImeUnsupported,
                "native IME does not represent the requested purpose exactly",
            ));
        }
    };
    if !matches!(config.hint, ImeHint::None) {
        return Err(Error::new(
            ErrorCode::ImeUnsupported,
            "native IME does not support the requested hint",
        ));
    }
    if config.surrounding_text.is_some() {
        return Err(Error::new(
            ErrorCode::ImeUnsupported,
            "native IME does not support complete surrounding-text semantics",
        ));
    }

    Ok(ResolvedImeConfig {
        purpose,
        hint: config.hint,
        cursor_area: config.cursor_area,
    })
}

fn require_capability_decisions(
    decisions: &[CapabilityDecision],
    capabilities: &HostCapabilities,
) -> Result<()> {
    for decision in decisions {
        match decision.kind() {
            CapabilityKind::Role(role) => capabilities.require_role(role)?,
            CapabilityKind::Fullscreen(fullscreen) => {
                capabilities.require_fullscreen(fullscreen)?
            }
            CapabilityKind::Cursor(cursor) => capabilities.require_cursor(cursor)?,
            CapabilityKind::Window(operation) => capabilities.require_window(operation)?,
            CapabilityKind::CursorGrab(grab) => capabilities.require_cursor_grab(grab)?,
            CapabilityKind::Ime(capability) => {
                if let Err(mut error) = capabilities.require_ime(capability) {
                    error.code = ErrorCode::ImeUnsupported;
                    return Err(error);
                }
            }
            CapabilityKind::Accessibility => capabilities.require_accessibility()?,
        }
    }

    Ok(())
}
