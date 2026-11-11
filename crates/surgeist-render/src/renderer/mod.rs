mod dispatch;
mod options;
mod publication;
mod readback;
#[cfg(test)]
mod test_support;
#[cfg(test)]
pub(crate) use test_support::{
    BoundedBackdropRenderResultForTest, ColorFilterRenderResultForTest,
    ForcedGraphRenderResultForTest, SpatialFilterRenderResultForTest,
    unsupported_graph_diagnostic_for_test,
};

use crate::error::{SurfaceDiagnosticOperation, SurfaceIoOperation};
use dispatch::runtime_surface_format;
pub use options::{Antialiasing, EffectQualityPolicy, Options, ResourceCacheBudget};

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use super::surface::{PresentedLifecycle, PresentedSurfaceState, ResizeState};
use super::{
    backend::*,
    geometry::physical_size,
    surface::{HeadlessResources, RendererIdentity, SurfaceBackend},
    validation::*,
    *,
};
use crate::accounting::{
    ImageHistory, RenderAttemptKind, RenderCpuSnapshot, RenderDeviceSnapshot, RenderFrameReport,
    RenderFrameReportRef, RenderReadbackReport, RenderReadbackReportRef, RendererCpu,
};

/// GPU-only renderer and owner of device-scoped resources and frame transactions.
///
/// Effect-free scenes select [`RenderRoute::DirectVello`]. Scenes requiring the
/// implemented resolved-alpha-mask, composition, or bounded-backdrop subset
/// select [`RenderRoute::GpuGraph`]. Both routes encode into one
/// transaction-owned submission. The renderer never retries pixels on a CPU
/// path and never performs implicit readback.
pub struct Renderer {
    identity: RendererIdentity,
    options: Options,
    stats: Stats,
    cpu: RendererCpu,
    image_history: ImageHistory,
    frame_reports: Option<RenderFrameReport>,
    readback_reports: Option<RenderReadbackReport>,
    backend: Option<Backend>,
    default_device: Option<DeviceSlotIdentity>,
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
fn ensure_presented_device_available_after_creation(
    backend: &mut Backend,
    device_identity: DeviceSlotIdentity,
    operation: RuntimeOperation,
) -> Result<()> {
    if let Some(error) = backend.terminal_error(device_identity, operation) {
        return Err(error);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PresentedConfigurationOperation {
    Rendering,
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    Resume,
}

impl PresentedConfigurationOperation {
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    const fn runtime_operation(self) -> RuntimeOperation {
        match self {
            Self::Rendering => RuntimeOperation::SurfaceRendering,
            #[cfg(any(
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::Resume => RuntimeOperation::SurfaceResume,
        }
    }
}

impl Renderer {
    /// Creates a GPU-only renderer with the supplied fixed [`Options`].
    ///
    /// Device selection is asynchronous. When no compatible adapter is
    /// available, construction retains a contract-only headless boundary so
    /// later nonzero operations return typed runtime capability failures.
    pub async fn new(options: Options) -> Result<Self> {
        options
            .accounting()
            .limits()
            .validate(RenderAttemptKind::Frame)?;
        let cpu = RendererCpu::new(options.accounting())?;
        let image_history = ImageHistory::new(&cpu)?;
        let mut backend = Backend::new(
            options.resource_cache_budget(),
            cpu.clone(),
            options.accounting(),
        )?;
        let default_device = backend.select_device(None).await?;
        let backend = default_device.map(|_| backend);

        Ok(Self {
            identity: RendererIdentity::new(),
            options,
            stats: Stats::default(),
            cpu,
            image_history,
            frame_reports: None,
            readback_reports: None,
            backend,
            default_device,
        })
    }

    /// Creates a surface and awaits any native-window or WebGPU host setup.
    ///
    /// The returned surface is ready for its next lifecycle operation when this
    /// future succeeds. Invalid options and unsupported attachments preserve
    /// their existing diagnostics when the future is awaited. Presented host
    /// lifecycle remains owned by the caller and its window or browser event
    /// loop. This future does not promise to be `Send`.
    pub async fn create_surface(
        &mut self,
        attachment: Attachment,
        options: SurfaceOptions,
    ) -> Result<Surface> {
        validate_surface_options(options)?;
        Self::create_surface_with_configuration_operation(
            self.backend.as_mut(),
            self.identity.clone(),
            self.default_device,
            attachment,
            options,
            PresentedConfigurationOperation::Rendering,
            None,
        )
        .await
    }

    async fn create_surface_with_configuration_operation(
        backend: Option<&mut Backend>,
        renderer_identity: RendererIdentity,
        default_device: Option<DeviceSlotIdentity>,
        attachment: Attachment,
        options: SurfaceOptions,
        configuration_operation: PresentedConfigurationOperation,
        preferred_device: Option<DeviceSlotIdentity>,
    ) -> Result<Surface> {
        match attachment {
            Attachment::Headless => {
                Self::create_headless_surface_with_backend(
                    backend,
                    default_device,
                    renderer_identity,
                    options,
                )
                .await
            }
            Attachment::WebCanvas(canvas) => {
                Self::create_web_canvas_surface(
                    backend,
                    renderer_identity,
                    canvas,
                    options,
                    configuration_operation,
                    preferred_device,
                )
                .await
            }
            #[cfg(feature = "render-window")]
            Attachment::Window(handle) => {
                let Some(backend) = backend else {
                    return Err(Error::runtime_unavailable(
                        RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
                        "no compatible wgpu adapter is available",
                    ));
                };
                let physical_size = physical_size(options.size, options.scale)?;
                let (surface, device_identity) = backend
                    .create_presented_surface(
                        handle.clone(),
                        preferred_device,
                        configuration_operation.runtime_operation(),
                    )
                    .await?;
                ensure_presented_device_available_after_creation(
                    backend,
                    device_identity,
                    configuration_operation.runtime_operation(),
                )?;
                let mut created = Surface::with_backend(
                    Attachment::Window(handle),
                    options,
                    SurfaceBackend::Presented {
                        surface: Box::new(surface),
                        device_identity,
                        state: PresentedSurfaceState::new(physical_size, ResizeState::Idle),
                    },
                    renderer_identity,
                );
                Self::configure_presented_surface_with_backend(
                    backend,
                    &mut created,
                    configuration_operation,
                    None,
                )
                .await?;
                Ok(created)
            }
        }
    }

    #[cfg(all(feature = "render-web", target_arch = "wasm32"))]
    async fn create_web_canvas_surface(
        backend: Option<&mut Backend>,
        renderer_identity: RendererIdentity,
        canvas: WebCanvas,
        options: SurfaceOptions,
        configuration_operation: PresentedConfigurationOperation,
        preferred_device: Option<DeviceSlotIdentity>,
    ) -> Result<Surface> {
        let Some(html_canvas) = canvas.html_canvas() else {
            return Err(Error::new(
                BackendErrorCode::SurfaceCreateFailed,
                format!("web canvas surface '{}' has no canvas handle", canvas.id()),
            ));
        };
        let Some(backend) = backend else {
            return Err(Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
                "no compatible WebGPU adapter is available",
            ));
        };
        let physical_size = physical_size(options.size, options.scale)?;
        let (surface, device_identity) = backend
            .create_presented_surface(
                wgpu::SurfaceTarget::Canvas(html_canvas),
                preferred_device,
                configuration_operation.runtime_operation(),
            )
            .await?;
        ensure_presented_device_available_after_creation(
            backend,
            device_identity,
            configuration_operation.runtime_operation(),
        )?;
        let mut created = Surface::with_backend(
            Attachment::WebCanvas(canvas),
            options,
            SurfaceBackend::Presented {
                surface: Box::new(surface),
                device_identity,
                state: PresentedSurfaceState::new(physical_size, ResizeState::Idle),
            },
            renderer_identity,
        );
        Self::configure_presented_surface_with_backend(
            backend,
            &mut created,
            configuration_operation,
            None,
        )
        .await?;
        Ok(created)
    }

    #[cfg(not(all(feature = "render-web", target_arch = "wasm32")))]
    async fn create_web_canvas_surface(
        _backend: Option<&mut Backend>,
        _renderer_identity: RendererIdentity,
        canvas: WebCanvas,
        _options: SurfaceOptions,
        _configuration_operation: PresentedConfigurationOperation,
        _preferred_device: Option<DeviceSlotIdentity>,
    ) -> Result<Surface> {
        let _ = canvas;
        Err(Error::unsupported_render_primitive(
            UnsupportedPrimitive::new(
                PrimitiveFamily::Surfaces,
                PrimitiveOperation::WebCanvasSurface,
            ),
        ))
    }

    /// Creates a headless surface for later asynchronous GPU operations.
    ///
    /// `size` is in logical units and `scale` converts it to physical pixels.
    /// Await this operation before using the surface. Input and `Rgba8` format
    /// failures are reported when the future is awaited; explicit readback is a
    /// separate asynchronous operation.
    pub async fn create_headless(&mut self, size: Size, scale: f64) -> Result<Surface> {
        let options = SurfaceOptions {
            size,
            scale,
            ..SurfaceOptions::default()
        };
        self.create_headless_surface(options).await
    }

    async fn create_headless_surface(&mut self, options: SurfaceOptions) -> Result<Surface> {
        Self::create_headless_surface_with_backend(
            self.backend.as_mut(),
            self.default_device,
            self.identity.clone(),
            options,
        )
        .await
    }

    async fn create_headless_surface_with_backend(
        backend: Option<&mut Backend>,
        default_device: Option<DeviceSlotIdentity>,
        renderer_identity: RendererIdentity,
        options: SurfaceOptions,
    ) -> Result<Surface> {
        validate_surface_options(options)?;
        if options.format != Format::Rgba8 {
            return Err(Error::new(
                BackendErrorCode::SurfaceCreateFailed,
                "headless surfaces require Rgba8 format for Vello storage rendering",
            ));
        }
        let physical_size = physical_size(options.size, options.scale)?;
        let backend = if let (Some(backend), Some(device_identity)) = (backend, default_device) {
            if let Some(error) =
                backend.terminal_error(device_identity, RuntimeOperation::AdapterSelection)
            {
                return Err(error);
            }
            SurfaceBackend::Headless {
                device_identity,
                resources: HeadlessResources::for_physical_size(physical_size),
                physical_size,
            }
        } else {
            SurfaceBackend::ContractOnly { physical_size }
        };

        Ok(Surface::with_backend(
            Attachment::Headless,
            options,
            backend,
            renderer_identity,
        ))
    }

    /// Updates native presented-resize intent after validating surface identity and lifecycle.
    ///
    /// The flag is host scheduling input, not a resize by itself. Repeating the
    /// same value is idempotent; invalid, unavailable, foreign, or stale surfaces
    /// return their typed diagnostic without changing committed resources.
    pub fn set_surface_resizing(&mut self, surface: &mut Surface, resizing: bool) -> Result<()> {
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        self.validate_surface_device_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        surface.ensure_available(SurfaceIoOperation::Rendering)?;

        #[cfg(not(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        )))]
        let _ = resizing;

        #[cfg(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        ))]
        if let SurfaceBackend::Presented { state, .. } = &mut surface.backend {
            let next = if resizing {
                ResizeState::Resizing
            } else {
                ResizeState::Idle
            };
            if state.lifecycle().resize_state() == next {
                return Ok(());
            }
            state.set_resizing(next);
        }

        Ok(())
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    async fn configure_presented_surface_if_needed(
        &mut self,
        surface: &mut Surface,
        observer: Option<&crate::accounting::Account>,
    ) -> Result<()> {
        let Some(pending) = surface.pending_presented_configuration() else {
            return Ok(());
        };
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "no compatible wgpu adapter is available",
            )
        })?;
        pending
            .configure(
                backend,
                PresentedConfigurationOperation::Rendering.runtime_operation(),
                observer,
            )
            .await
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    async fn configure_presented_surface_with_backend(
        backend: &mut Backend,
        surface: &mut Surface,
        operation: PresentedConfigurationOperation,
        observer: Option<&crate::accounting::Account>,
    ) -> Result<()> {
        let Some(pending) = surface.pending_presented_configuration() else {
            return Ok(());
        };
        pending
            .configure(backend, operation.runtime_operation(), observer)
            .await
    }

    #[cfg(not(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    )))]
    async fn configure_presented_surface_if_needed(
        &mut self,
        _surface: &mut Surface,
        _observer: Option<&crate::accounting::Account>,
    ) -> Result<()> {
        Ok(())
    }

    /// Submits one failure-atomic GPU render operation for an available surface.
    ///
    /// Awaiting this future returns [`Stats`] only after validation, resource
    /// cleanup, submission, and any presentation succeed. On validation,
    /// lifecycle, capability, backend, cancellation, or presentation failure,
    /// no draft frame or statistics publish: the surface and [`Self::stats`]
    /// retain their last successful values. There is no production CPU fallback.
    pub async fn render(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
    ) -> Result<Stats> {
        self.render_frame(surface, scene, parameters).await
    }

    /// Resumes a compatible surface, awaiting host-resource recreation when presented.
    ///
    /// Await this operation before rendering again. Incompatible attachments,
    /// foreign/stale identity, terminal-device, configuration, and host failures
    /// preserve the previously committed surface state and their typed ordering.
    pub async fn resume_surface(
        &mut self,
        surface: &mut Surface,
        attachment: Attachment,
    ) -> Result<()> {
        #[cfg(not(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        )))]
        let _ = attachment;
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Resume)?;
        match &surface.backend {
            #[cfg(any(
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            SurfaceBackend::Presented {
                state,
                device_identity,
                ..
            } => {
                let device_identity = *device_identity;
                let renderer_identity = self.identity.clone();
                let default_device = self.default_device;
                let backend = self.surface_backend_for_device(
                    device_identity,
                    SurfaceDiagnosticOperation::Resume,
                )?;
                let action = Surface::presented_resume_action(surface.state, state.lifecycle());
                let resizing = state.lifecycle().resize_state();
                if let Some(error) =
                    backend.terminal_error(device_identity, RuntimeOperation::SurfaceResume)
                {
                    return Err(error);
                }
                surface.ensure_attachment_compatible(&attachment)?;
                match action {
                    super::surface::PresentedResumeAction::NoOp => Ok(()),
                    super::surface::PresentedResumeAction::ConfigureExisting => {
                        Self::configure_presented_surface_with_backend(
                            backend,
                            surface,
                            PresentedConfigurationOperation::Resume,
                            None,
                        )
                        .await
                    }
                    action @ (super::surface::PresentedResumeAction::Configure
                    | super::surface::PresentedResumeAction::Recreate) => {
                        let mut next = Self::create_surface_with_configuration_operation(
                            Some(backend),
                            renderer_identity,
                            default_device,
                            attachment,
                            surface.options,
                            PresentedConfigurationOperation::Resume,
                            Some(device_identity),
                        )
                        .await?;
                        next.last_parameters = surface.last_parameters;
                        if matches!(action, super::surface::PresentedResumeAction::Configure) {
                            next.renderer_identity = surface.renderer_identity.clone();
                        }
                        if let SurfaceBackend::Presented { state, .. } = &mut next.backend {
                            state.set_resizing(resizing);
                        }
                        *surface = next;
                        Ok(())
                    }
                }
            }
            SurfaceBackend::ContractOnly { .. } | SurfaceBackend::Headless { .. } => {
                Err(unsupported_surface_backend())
            }
        }
    }

    /// Projects immutable runtime-phase capabilities of the device selected by `surface`.
    ///
    /// This query observes pending terminal device signals but performs no
    /// allocation, submission, mapping, polling, or Vello/WGPU resource call.
    /// It is separate from semantic [`Capabilities`] and from any Cargo feature:
    /// features select compiled host adapters, while this report describes the
    /// selected device/surface snapshot.
    #[must_use]
    pub fn runtime_capabilities(&mut self, surface: &Surface) -> RuntimeCapabilities {
        if !self.identity.matches(&surface.renderer_identity) {
            return RuntimeCapabilities::Unavailable(
                RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
                    kind: SurfaceIdentityMismatchKind::ForeignRenderer,
                },
            );
        }
        let Some(device_identity) = surface.device_identity() else {
            return RuntimeCapabilities::Unavailable(
                RuntimeCapabilityUnavailableReason::AdapterUnavailable,
            );
        };
        let Some(backend) = self.backend.as_mut() else {
            return RuntimeCapabilities::Unavailable(
                RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
                    kind: SurfaceIdentityMismatchKind::StaleDeviceGeneration,
                },
            );
        };
        if !backend.has_device_slot(device_identity) {
            return RuntimeCapabilities::Unavailable(
                RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
                    kind: SurfaceIdentityMismatchKind::StaleDeviceGeneration,
                },
            );
        }
        if let Some(reason) = backend.terminal_reason(device_identity) {
            return RuntimeCapabilities::Unavailable(reason);
        }
        if let Some(reason) = runtime_surface_unavailable_reason(surface) {
            return RuntimeCapabilities::Unavailable(reason);
        }
        let Some(capabilities) = backend.device_capabilities(device_identity) else {
            return RuntimeCapabilities::Unavailable(
                RuntimeCapabilityUnavailableReason::SurfaceIdentityMismatch {
                    kind: SurfaceIdentityMismatchKind::StaleDeviceGeneration,
                },
            );
        };
        RuntimeCapabilities::Available(capabilities.runtime_report(runtime_surface_format(surface)))
    }

    /// Borrows the latest started frame after cleanup; setup failure preserves it.
    pub fn last_attempt(&self) -> Option<RenderFrameReportRef<'_>> {
        self.frame_reports.as_ref().map(RenderFrameReportRef::new)
    }
    /// Moves the latest frame report into caller custody without allocating.
    pub fn take_last_attempt(&mut self) -> Option<RenderFrameReport> {
        self.frame_reports.take()
    }
    /// Borrows the latest started explicit-readback report, without changing
    /// frame observations or observing device lifecycle signals.
    pub fn last_readback_attempt(&self) -> Option<RenderReadbackReportRef<'_>> {
        self.readback_reports
            .as_ref()
            .map(RenderReadbackReportRef::new)
    }
    /// Moves the latest explicit-readback report into caller custody.
    pub fn take_last_readback_attempt(&mut self) -> Option<RenderReadbackReport> {
        self.readback_reports.take()
    }
    /// Exact Renderer CPU lifetime point. Profiling uses one coherent Ledger snapshot.
    pub fn cpu_accounting_snapshot(&self) -> Result<RenderCpuSnapshot> {
        self.cpu.snapshot()
    }
    /// Exact selected native generation, including its observed terminal custody.
    /// None selects the existing default; no adapter selection/retirement occurs.
    pub fn device_accounting_snapshot(
        &self,
        surface: Option<&Surface>,
    ) -> Result<RenderDeviceSnapshot> {
        let identity = if let Some(surface) = surface {
            self.validate_surface_renderer_identity(
                surface,
                SurfaceDiagnosticOperation::Rendering,
            )?;
            surface.device_identity()
        } else {
            self.default_device
        }
        .ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
                "no selected accounting device",
            )
        })?;
        let backend = self.backend.as_ref().ok_or_else(|| {
            surface_identity_mismatch(
                SurfaceDiagnosticOperation::Rendering,
                SurfaceIdentityMismatchKind::StaleDeviceGeneration,
            )
        })?;
        backend
            .generation(identity)
            .map_err(|_| {
                surface_identity_mismatch(
                    SurfaceDiagnosticOperation::Rendering,
                    SurfaceIdentityMismatchKind::StaleDeviceGeneration,
                )
            })?
            .snapshot()
    }
    #[must_use]
    /// Returns statistics for the last successful published frame.
    ///
    /// Failed and canceled render attempts do not replace this value.
    pub const fn stats(&self) -> Stats {
        self.stats
    }

    #[must_use]
    /// Returns the fixed configuration supplied when this renderer was created.
    pub const fn options(&self) -> Options {
        self.options
    }

    fn validate_surface_renderer_identity(
        &self,
        surface: &Surface,
        operation: SurfaceDiagnosticOperation,
    ) -> Result<()> {
        if self.identity.matches(&surface.renderer_identity) {
            return Ok(());
        }
        Err(surface_identity_mismatch(
            operation,
            SurfaceIdentityMismatchKind::ForeignRenderer,
        ))
    }

    fn validate_surface_device_identity(
        &mut self,
        surface: &Surface,
        operation: SurfaceDiagnosticOperation,
    ) -> Result<()> {
        let Some(device_identity) = surface.device_identity() else {
            return Ok(());
        };
        self.surface_backend_for_device(device_identity, operation)
            .map(|_| ())
    }

    fn surface_backend_for_device(
        &mut self,
        device_identity: DeviceSlotIdentity,
        operation: SurfaceDiagnosticOperation,
    ) -> Result<&mut Backend> {
        let backend = self.backend.as_mut().ok_or_else(|| {
            surface_identity_mismatch(
                operation,
                SurfaceIdentityMismatchKind::StaleDeviceGeneration,
            )
        })?;
        if !backend.has_device_slot(device_identity) {
            return Err(surface_identity_mismatch(
                operation,
                SurfaceIdentityMismatchKind::StaleDeviceGeneration,
            ));
        }
        Ok(backend)
    }

    fn validate_surface_device_terminal(
        &mut self,
        surface: &Surface,
        operation: RuntimeOperation,
    ) -> Result<()> {
        let Some(device_identity) = surface.device_identity() else {
            return Ok(());
        };
        if let Some(error) = self
            .backend
            .as_mut()
            .and_then(|backend| backend.terminal_error(device_identity, operation))
        {
            return Err(error);
        }
        Ok(())
    }

    #[must_use]
    /// Returns the crate's semantic authored-operation capability contract.
    ///
    /// This does not inspect a runtime device or surface; use
    /// [`Self::runtime_capabilities`] for runtime facts.
    pub const fn capabilities(&self) -> Capabilities {
        Capabilities::CURRENT
    }
}

fn runtime_surface_unavailable_reason(
    _surface: &Surface,
) -> Option<RuntimeCapabilityUnavailableReason> {
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    if let SurfaceBackend::Presented { state, .. } = &_surface.backend {
        let state = match state.lifecycle() {
            PresentedLifecycle::NonRenderable { .. } => RenderSurfaceAvailability::NonRenderable,
            PresentedLifecycle::Occluded { .. } => RenderSurfaceAvailability::Occluded,
            PresentedLifecycle::Lost => RenderSurfaceAvailability::Lost,
            PresentedLifecycle::Ready { .. } | PresentedLifecycle::ResizePending { .. } => {
                return None;
            }
        };
        return Some(RuntimeCapabilityUnavailableReason::SurfaceUnavailable { state });
    }
    None
}

fn unsupported_surface_backend() -> Error {
    Error::new(
        BackendErrorCode::UnsupportedBackend,
        "surface backend does not support this operation",
    )
}

fn surface_identity_mismatch(
    operation: SurfaceDiagnosticOperation,
    kind: SurfaceIdentityMismatchKind,
) -> Error {
    Error::runtime_capability_unavailable(RuntimeCapabilityUnavailable::surface_identity_mismatch(
        operation, kind,
    ))
}

#[cfg(test)]
mod configuration_operation_tests {
    use super::*;

    #[test]
    fn configuration_operations_project_to_their_literal_runtime_operations() {
        assert_eq!(
            PresentedConfigurationOperation::Rendering.runtime_operation(),
            RuntimeOperation::SurfaceRendering
        );
        #[cfg(any(
            feature = "render-window",
            all(feature = "render-web", target_arch = "wasm32")
        ))]
        assert_eq!(
            PresentedConfigurationOperation::Resume.runtime_operation(),
            RuntimeOperation::SurfaceResume
        );
    }
}
