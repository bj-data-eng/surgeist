#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use super::SurfaceFrameCommit;
use super::{Backend, DeviceSlotIdentity};
#[cfg(all(test, feature = "render-window", not(target_arch = "wasm32")))]
mod tests;
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use crate::accounting::{Account, RenderPhase, RenderWork};
use crate::error::Result;
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use crate::{
    error::{BackendErrorCode, Error, RuntimeCapabilityUnavailable, RuntimeOperation},
    geometry::PhysicalSize,
    gpu_transaction::{GpuOperationStage, GpuOperationTransaction},
    pass::PreparedGraph,
    resource::WorkingFormat,
    surface::{
        AcquiredPresentedSurfaceTexture, Format, PendingPresentedConfiguration,
        PresentedConfigurationDraft, PresentedLifecycle, PresentedResourceBundle, PresentedSurface,
        PresentedSurfaceAcquire, PresentedSurfaceState, Surface, SurfaceBackend,
    },
};
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(super) fn require_presented_device_identity(
    identity: Option<DeviceSlotIdentity>,
) -> Result<DeviceSlotIdentity> {
    identity.ok_or_else(|| {
        Error::runtime_unavailable(
            RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
            "no compatible WGPU adapter is available for the presentation surface",
        )
    })
}

impl Backend {
    pub(super) async fn select_presented_device(
        &mut self,
        surface: &wgpu::Surface<'_>,
        preferred: Option<DeviceSlotIdentity>,
    ) -> Result<Option<DeviceSlotIdentity>> {
        if let Some(identity) = self.compatible_ready_device(preferred, |ready| {
            ready.adapter.is_surface_supported(surface)
        }) {
            return Ok(Some(identity));
        }
        self.new_device(Some(surface)).await
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(crate) async fn create_presented_surface(
        &mut self,
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        preferred: Option<DeviceSlotIdentity>,
        operation: RuntimeOperation,
    ) -> Result<(PresentedSurface, DeviceSlotIdentity)> {
        let surface = self
            .instance
            .create_surface(target.into())
            .map_err(|source| {
                Error::new(
                    BackendErrorCode::SurfaceCreateFailed,
                    "failed to create a WGPU presentation surface",
                )
                .with_source(source)
            })?;
        let identity = require_presented_device_identity(
            self.select_presented_device(&surface, preferred).await?,
        )?;
        let ready = self.ready_state_mut(
            identity,
            operation,
            BackendErrorCode::SurfaceCreateFailed,
            "the selected presentation device is unavailable",
        )?;
        let presented = PresentedSurface::new(surface, &ready.adapter)?;
        Ok((presented, identity))
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(crate) async fn configure_presented_surface(
        &mut self,
        pending: &PendingPresentedConfiguration<'_>,
        operation: RuntimeOperation,
        observer: Option<&Account>,
    ) -> Result<PresentedConfigurationDraft> {
        let identity = pending.device_identity();
        let phase_account = observer.map(|account| account.at(RenderPhase::Configure));
        let observer = phase_account.as_ref();
        let configure = async {
            let generation = self.generation(identity)?.clone();
            let transaction =
                self.begin_gpu_operation(identity, GpuOperationStage::Configure, operation)?;
            let ready = self.ready_state_mut(
                identity,
                operation,
                BackendErrorCode::SurfaceConfigureFailed,
                "presented device resources are unavailable before configuration",
            )?;
            let draft = pending.configure_draft(&ready.device, &generation, observer)?;
            let result = transaction.finish(operation, observer).await;
            result?;
            Ok(draft)
        };
        match observer {
            Some(account) => account.async_phase(RenderPhase::Configure, configure).await,
            None => configure.await,
        }
    }
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(super) fn exact_presented_graph_target(
    surface: &mut Surface,
    selected_working_format: WorkingFormat,
    graph_output_format: Format,
    known_output_extent: Option<PhysicalSize>,
) -> Result<(
    ExactPresentedGraphTarget<'_>,
    DeviceSlotIdentity,
    PhysicalSize,
    Format,
    WorkingFormat,
)> {
    let (native, state, device_identity, physical_size, output_format) = match &mut surface.backend
    {
        SurfaceBackend::Presented {
            surface: native,
            device_identity,
            state,
        } => {
            match state.lifecycle() {
                PresentedLifecycle::Ready { .. } => {}
                PresentedLifecycle::ResizePending { .. } => {
                    return Err(Error::new(
                        BackendErrorCode::SurfaceConfigureFailed,
                        "presented exact graph execution started before configuration committed",
                    ));
                }
                PresentedLifecycle::NonRenderable { .. } => {
                    return Err(Error::runtime_unavailable(
                        RuntimeCapabilityUnavailable::rendering_nonrenderable(),
                        "presented exact graph output is not renderable",
                    ));
                }
                PresentedLifecycle::Occluded { .. } => {
                    return Err(Error::runtime_unavailable(
                        RuntimeCapabilityUnavailable::rendering_occluded(),
                        "presented exact graph output is occluded",
                    ));
                }
                PresentedLifecycle::Lost => {
                    return Err(Error::runtime_unavailable(
                        RuntimeCapabilityUnavailable::rendering_lost(),
                        "presented exact graph output is lost",
                    ));
                }
            }
            let resources = native.committed().ok_or_else(|| {
                Error::new(
                    BackendErrorCode::SurfaceConfigureFailed,
                    "ready presented exact graph output has no committed configuration",
                )
            })?;
            let physical_size = PhysicalSize::new(resources.config.width, resources.config.height);
            if resources.config.format != native.format
                || state.requested_physical_size() != physical_size
            {
                return Err(Error::new(
                    BackendErrorCode::SurfaceConfigureFailed,
                    "presented exact graph output differs from its committed configuration",
                ));
            }
            let output_format = match native.format {
                wgpu::TextureFormat::Rgba8Unorm => Format::Rgba8,
                wgpu::TextureFormat::Bgra8Unorm => Format::Bgra8,
                _ => {
                    return Err(Error::new(
                        BackendErrorCode::PresentFailed,
                        "presented exact graph output is not an advertised RGBA8 or BGRA8 format",
                    ));
                }
            };
            (
                &**native,
                state,
                *device_identity,
                physical_size,
                output_format,
            )
        }
        SurfaceBackend::ContractOnly { .. } | SurfaceBackend::Headless { .. } => {
            return Err(Error::new(
                BackendErrorCode::UnsupportedBackend,
                "presented exact graph execution requires a presented surface",
            ));
        }
    };
    if physical_size.width() == 0
        || physical_size.height() == 0
        || graph_output_format != output_format
        || known_output_extent.is_some_and(|extent| extent != physical_size)
    {
        return Err(Error::new(
            BackendErrorCode::RenderFailed,
            "the presented graph differs from the exact eligible output",
        ));
    }
    Ok((
        ExactPresentedGraphTarget { native, state },
        device_identity,
        physical_size,
        output_format,
        selected_working_format,
    ))
}

/// Exclusive qualification retained until one exact presented acquisition.
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(super) struct ExactPresentedGraphTarget<'surface> {
    native: &'surface PresentedSurface,
    state: &'surface mut PresentedSurfaceState,
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
enum PresentedAcquireFailure {
    Suboptimal,
    Outdated,
    Occluded,
    Timeout,
    Lost,
    Validation,
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
async fn finish_presented_acquire_failure(
    transaction: GpuOperationTransaction,
    state: &mut PresentedSurfaceState,
    failure: PresentedAcquireFailure,
    account: &Account,
) -> Result<Error> {
    let scope_result = transaction
        .finish(RuntimeOperation::SurfaceRendering, Some(account))
        .await;
    match failure {
        PresentedAcquireFailure::Suboptimal | PresentedAcquireFailure::Outdated => {
            state.mark_configuration_pending();
        }
        PresentedAcquireFailure::Occluded => state.mark_occluded(),
        PresentedAcquireFailure::Lost => state.mark_lost(),
        PresentedAcquireFailure::Timeout | PresentedAcquireFailure::Validation => {}
    }
    scope_result?;
    Ok(match failure {
        PresentedAcquireFailure::Suboptimal => Error::new(
            BackendErrorCode::SurfaceOutdated,
            "surface is suboptimal and requires reconfiguration",
        ),
        PresentedAcquireFailure::Outdated => Error::new(
            BackendErrorCode::SurfaceOutdated,
            "surface is outdated and requires reconfiguration",
        ),
        PresentedAcquireFailure::Occluded => Error::runtime_unavailable(
            RuntimeCapabilityUnavailable::rendering_occluded(),
            "surface is occluded",
        ),
        PresentedAcquireFailure::Timeout => Error::new(
            BackendErrorCode::SurfaceTimeout,
            "timed out acquiring surface texture",
        ),
        PresentedAcquireFailure::Lost => Error::runtime_unavailable(
            RuntimeCapabilityUnavailable::rendering_lost(),
            "surface was lost",
        ),
        PresentedAcquireFailure::Validation => Error::new(
            BackendErrorCode::PresentFailed,
            "surface texture validation failed",
        ),
    })
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
impl ExactPresentedGraphTarget<'_> {
    pub(super) async fn acquire<'a>(
        self,
        device: &wgpu::Device,
        prepared_graph: PreparedGraph<'a>,
        gpu_transaction: GpuOperationTransaction,
        account: &Account,
    ) -> Result<(
        AcquiredPresentedSurfaceTexture,
        PreparedGraph<'a>,
        GpuOperationTransaction,
    )> {
        // Keep these initialized locals in this order: admission failure drops
        // the transaction before the prepared graph, unlike unpolled arguments.
        let prepared = prepared_graph;
        let transaction = gpu_transaction;
        let acquired = match account.phase(RenderPhase::Acquire, || {
            account
                .at(RenderPhase::Acquire)
                .work(RenderWork::SurfaceAcquisitions, 1)?;
            Ok(self.native.acquire_texture(device))
        })? {
            PresentedSurfaceAcquire::Success(acquired) => acquired,
            PresentedSurfaceAcquire::Suboptimal(acquired) => {
                drop(acquired);
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Suboptimal,
                    account,
                )
                .await?);
            }
            PresentedSurfaceAcquire::Outdated => {
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Outdated,
                    account,
                )
                .await?);
            }
            PresentedSurfaceAcquire::Occluded => {
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Occluded,
                    account,
                )
                .await?);
            }
            PresentedSurfaceAcquire::Timeout => {
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Timeout,
                    account,
                )
                .await?);
            }
            PresentedSurfaceAcquire::Lost => {
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Lost,
                    account,
                )
                .await?);
            }
            PresentedSurfaceAcquire::Validation => {
                drop(prepared);
                return Err(finish_presented_acquire_failure(
                    transaction,
                    self.state,
                    PresentedAcquireFailure::Validation,
                    account,
                )
                .await?);
            }
        };
        Ok((acquired, prepared, transaction))
    }
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(super) fn internal_vello_presented_resources<'a>(
    native: &'a PresentedSurface,
    state: &PresentedSurfaceState,
) -> Result<Option<&'a PresentedResourceBundle>> {
    match state.lifecycle() {
        PresentedLifecycle::NonRenderable { .. } | PresentedLifecycle::Lost => return Ok(None),
        PresentedLifecycle::ResizePending { .. } => {
            return Err(Error::new(
                BackendErrorCode::SurfaceConfigureFailed,
                "presented rendering started before configuration committed",
            ));
        }
        PresentedLifecycle::Ready { .. } | PresentedLifecycle::Occluded { .. } => {}
    }
    native.committed().map(Some).ok_or_else(|| {
        Error::new(
            BackendErrorCode::SurfaceConfigureFailed,
            "ready presented lifecycle has no committed target resources",
        )
    })
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
pub(super) async fn present_internal_vello_target(
    backend: &mut Backend,
    native: &PresentedSurface,
    device_identity: DeviceSlotIdentity,
    state: &mut PresentedSurfaceState,
    resources: &PresentedResourceBundle,
    account: &Account,
) -> Result<SurfaceFrameCommit> {
    let generation = backend.generation(device_identity)?.clone();
    let transaction = backend.begin_gpu_operation(
        device_identity,
        GpuOperationStage::Present,
        RuntimeOperation::SurfaceRendering,
    )?;
    let (device, queue) = backend.present_device_queue(device_identity)?;
    let (surface_texture, transaction) =
        acquire_internal_vello_surface_texture(native, device, state, transaction, account).await?;
    let mut encoder = super::execute::create_frame_encoder(
        device,
        &generation,
        "Surgeist surface blit",
        account,
    )?;
    let surface_view = account.phase(RenderPhase::GpuPrepare, || {
        surface_texture.create_view(&generation, Some(&account.at(RenderPhase::GpuPrepare)))
    })?;
    account.phase(RenderPhase::Present, || {
        account
            .at(RenderPhase::Present)
            .work(RenderWork::CustomPresentPasses, 1)?;
        resources
            .blitter
            .copy(device, &mut encoder, &resources.target_view, &surface_view);
        Ok(())
    })?;
    transaction
        .submit_command_buffer_with_host_effect(
            queue,
            super::execute::finish_frame_encoder(encoder, account)?,
            || surface_texture.present(),
            RuntimeOperation::SurfaceRendering,
            account,
        )
        .await?;
    Ok(SurfaceFrameCommit::without_headless_publication())
}

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
async fn acquire_internal_vello_surface_texture(
    native: &PresentedSurface,
    device: &wgpu::Device,
    state: &mut PresentedSurfaceState,
    gpu_transaction: GpuOperationTransaction,
    account: &Account,
) -> Result<(AcquiredPresentedSurfaceTexture, GpuOperationTransaction)> {
    let transaction = gpu_transaction;
    let surface_texture = match account.phase(RenderPhase::Acquire, || {
        account
            .at(RenderPhase::Acquire)
            .work(RenderWork::SurfaceAcquisitions, 1)?;
        Ok(native.acquire_texture(device))
    })? {
        PresentedSurfaceAcquire::Success(surface_texture) => surface_texture,
        PresentedSurfaceAcquire::Suboptimal(surface_texture) => {
            drop(surface_texture);
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Suboptimal,
                account,
            )
            .await?);
        }
        PresentedSurfaceAcquire::Outdated => {
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Outdated,
                account,
            )
            .await?);
        }
        PresentedSurfaceAcquire::Occluded => {
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Occluded,
                account,
            )
            .await?);
        }
        PresentedSurfaceAcquire::Timeout => {
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Timeout,
                account,
            )
            .await?);
        }
        PresentedSurfaceAcquire::Lost => {
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Lost,
                account,
            )
            .await?);
        }
        PresentedSurfaceAcquire::Validation => {
            return Err(finish_presented_acquire_failure(
                transaction,
                state,
                PresentedAcquireFailure::Validation,
                account,
            )
            .await?);
        }
    };
    Ok((surface_texture, transaction))
}
