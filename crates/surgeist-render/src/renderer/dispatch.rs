use super::{Renderer, publication::RenderPublication};
use crate::accounting::{Account, OwnedBox, RenderPhase, RenderWork};
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
#[cfg(not(test))]
use crate::backend::render_exact_presented_graph_surface;
#[cfg(all(
    test,
    any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    )
))]
use crate::backend::render_renderer_exact_presented_graph_surface as render_exact_presented_graph_surface;
#[cfg(not(test))]
use crate::backend::{ExactSurfaceGraph, render_exact_headless_graph_surface};
#[cfg(test)]
use crate::backend::{
    RendererExactSurfaceGraph as ExactSurfaceGraph,
    render_renderer_exact_headless_graph_surface as render_exact_headless_graph_surface,
};
use crate::error::{SurfaceDiagnosticOperation, SurfaceIoOperation};
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use crate::surface::SurfaceBackend;
use crate::{
    BackendErrorCode, Error, ErrorCode, Format, Parameters, PrimitiveFamily, PrimitiveOperation,
    RenderRoute, Result, RuntimeCapabilityUnavailable, RuntimeOperation, Scene, Surface,
    UnsupportedPrimitive,
    backend::{
        DeviceCapabilities, DeviceSlotIdentity, SurfaceFrameCommit, render_internal_vello_surface,
    },
    command::RenderCommands,
    encode::encode_vello_scene,
    frame::{
        FrameContext, FramePlan, GpuRenderGraph, GraphLoweringCompositeKind, GraphLoweringPassKind,
    },
    gpu_transaction::GpuOperationStage,
    pass::{ExecutableGraphDispatchEligibility, ExecutableGraphWorkingFormatRequest},
    vello_engine::scene::VelloScene,
};

pub(super) fn runtime_surface_format(surface: &Surface) -> Format {
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    if let crate::surface::SurfaceBackend::Presented {
        surface: native, ..
    } = &surface.backend
    {
        return match native.format {
            wgpu::TextureFormat::Rgba8Unorm => Format::Rgba8,
            wgpu::TextureFormat::Bgra8Unorm => Format::Bgra8,
            _ => surface.options.format,
        };
    }
    surface.options.format
}

fn rendering_adapter_unavailable() -> Error {
    Error::runtime_unavailable(
        RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
        "no compatible wgpu adapter is available",
    )
}

#[must_use = "the renderer dispatch boundary must resolve to exactly one execution route"]
pub(super) enum RendererFrameDispatch {
    DirectVello(RenderCommands),
    ExactGraph(OwnedBox<ExactSurfaceGraph>),
    RejectedFutureGraph(Error),
}

#[must_use = "prepared renderer execution must reach its selected GPU transaction"]
enum PreparedRendererExecution {
    DirectVello(OwnedBox<VelloScene>),
    ExactGraph(OwnedBox<ExactSurfaceGraph>),
}

impl Renderer {
    pub(super) async fn dispatch_render_frame(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        account: &Account,
    ) -> Result<(DeviceSlotIdentity, RenderPublication)> {
        self.dispatch_render_frame_with_working_format(
            surface,
            scene,
            parameters,
            ExecutableGraphWorkingFormatRequest::ConfiguredPolicy(
                self.options.effect_quality_policy(),
            ),
            account,
        )
        .await
    }

    pub(super) async fn dispatch_render_frame_with_working_format(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: ExecutableGraphWorkingFormatRequest,
        account: &Account,
    ) -> Result<(DeviceSlotIdentity, RenderPublication)> {
        let device_identity = self.render_device_identity(surface)?;
        let (normalized, execution) = account.phase(RenderPhase::FramePrepare, || {
            self.prepare_render_execution(
                surface,
                scene,
                parameters,
                device_identity,
                working_format,
                &account.at(RenderPhase::FramePrepare),
            )
        })?;
        self.configure_presented_surface_if_needed(surface, Some(account))
            .await?;
        let route = match &execution {
            PreparedRendererExecution::DirectVello(_) => RenderRoute::DirectVello,
            PreparedRendererExecution::ExactGraph(_) => RenderRoute::GpuGraph,
        };
        let render_account = account.at(RenderPhase::Render);
        let frame = account
            .async_phase(
                RenderPhase::Render,
                self.execute_render_frame(
                    surface,
                    execution,
                    parameters,
                    device_identity,
                    &render_account,
                ),
            )
            .await?;
        drop(normalized);
        Ok((
            device_identity,
            RenderPublication::new(frame, parameters, route),
        ))
    }

    pub(super) fn render_device_identity(
        &mut self,
        surface: &Surface,
    ) -> Result<DeviceSlotIdentity> {
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        self.validate_surface_device_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        surface.ensure_available(SurfaceIoOperation::Rendering)?;
        surface.ensure_renderable()?;
        self.validate_surface_device_terminal(surface, RuntimeOperation::SurfaceRendering)?;

        let Some(device_identity) = surface.device_identity() else {
            return Err(rendering_adapter_unavailable());
        };
        if self.backend.is_none() {
            return Err(rendering_adapter_unavailable());
        }
        Ok(device_identity)
    }

    fn prepare_render_execution(
        &mut self,
        surface: &Surface,
        scene: &Scene,
        parameters: Parameters,
        device_identity: DeviceSlotIdentity,
        working_format: ExecutableGraphWorkingFormatRequest,
        account: &Account,
    ) -> Result<(RenderCommands, PreparedRendererExecution)> {
        let normalized = scene.normalize(self.capabilities(), account)?;
        let graph_source = normalized.clone();
        let frame_context = FrameContext::try_new(
            surface.size(),
            surface.scale(),
            self.options.antialiasing(),
            parameters.base_color,
        )?;
        let frame_plan = normalized.plan_for(frame_context, account)?;
        let capabilities = self
            .backend
            .as_mut()
            .and_then(|backend| backend.device_capabilities(device_identity))
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the renderer dispatch boundary lost immutable device capabilities",
                )
            })?;
        let dispatch = self.classify_frame_dispatch(
            frame_plan,
            runtime_surface_format(surface),
            working_format,
            &capabilities,
            account,
        )?;
        Ok(match dispatch {
            RendererFrameDispatch::DirectVello(normalized) => {
                let vello_scene = encode_vello_scene(
                    &normalized,
                    surface.scale(),
                    surface.physical_size(),
                    account,
                )?;
                (
                    normalized,
                    PreparedRendererExecution::DirectVello(OwnedBox::new(
                        vello_scene,
                        &account.storage(),
                    )?),
                )
            }
            RendererFrameDispatch::ExactGraph(graph) => {
                (graph_source, PreparedRendererExecution::ExactGraph(graph))
            }
            RendererFrameDispatch::RejectedFutureGraph(error) => return Err(error),
        })
    }

    #[cfg(test)]
    pub(super) async fn execute_exact_graph_for_test(
        &mut self,
        surface: &mut Surface,
        graph: OwnedBox<ExactSurfaceGraph>,
        parameters: Parameters,
        device_identity: DeviceSlotIdentity,
        account: &Account,
    ) -> Result<SurfaceFrameCommit> {
        let render_account = account.at(RenderPhase::Render);
        account
            .async_phase(
                RenderPhase::Render,
                self.execute_render_frame(
                    surface,
                    PreparedRendererExecution::ExactGraph(graph),
                    parameters,
                    device_identity,
                    &render_account,
                ),
            )
            .await
    }

    async fn execute_render_frame(
        &mut self,
        surface: &mut Surface,
        execution: PreparedRendererExecution,
        parameters: Parameters,
        device_identity: DeviceSlotIdentity,
        account: &Account,
    ) -> Result<SurfaceFrameCommit> {
        let frame = {
            let backend = self
                .backend
                .as_mut()
                .ok_or_else(rendering_adapter_unavailable)?;
            let frame = match execution {
                PreparedRendererExecution::DirectVello(vello_scene) => {
                    let transaction = backend.begin_gpu_operation(
                        device_identity,
                        GpuOperationStage::Render,
                        RuntimeOperation::SurfaceRendering,
                    )?;
                    render_internal_vello_surface(
                        backend,
                        transaction,
                        surface,
                        &vello_scene,
                        parameters,
                        self.options.antialiasing(),
                        account,
                    )
                    .await
                }
                PreparedRendererExecution::ExactGraph(graph) => {
                    #[cfg(any(
                        feature = "render-window",
                        all(feature = "render-web", target_arch = "wasm32")
                    ))]
                    {
                        if matches!(&surface.backend, SurfaceBackend::Presented { .. }) {
                            render_exact_presented_graph_surface(backend, surface, graph, account)
                                .await
                        } else {
                            render_exact_headless_graph_surface(backend, surface, graph, account)
                                .await
                        }
                    }
                    #[cfg(not(any(
                        feature = "render-window",
                        all(feature = "render-web", target_arch = "wasm32")
                    )))]
                    {
                        render_exact_headless_graph_surface(backend, surface, graph, account).await
                    }
                }
            };
            if frame.is_err() {
                backend.observe_device_terminal(device_identity);
            }
            frame
        };
        let frame = match frame {
            Err(error) if error.code() == ErrorCode::SurfaceOutdated => {
                self.configure_presented_surface_if_needed(surface, Some(account))
                    .await?;
                return Err(error);
            }
            Err(error) => return Err(error),
            Ok(frame) => frame,
        };
        Ok(frame)
    }

    pub(super) fn classify_frame_dispatch(
        &self,
        plan: FramePlan,
        output_format: Format,
        working_format: ExecutableGraphWorkingFormatRequest,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<RendererFrameDispatch> {
        match plan {
            FramePlan::DirectVello(plan) => {
                Ok(RendererFrameDispatch::DirectVello(plan.into_commands()))
            }
            FramePlan::GpuGraph(graph) => match ExecutableGraphDispatchEligibility::try_classify(
                &graph,
                output_format,
                working_format,
                capabilities,
                account,
            )? {
                ExecutableGraphDispatchEligibility::ExactBase(preparable) => {
                    Ok(RendererFrameDispatch::ExactGraph(OwnedBox::new(
                        ExactSurfaceGraph::Base(preparable),
                        &account.storage(),
                    )?))
                }
                ExecutableGraphDispatchEligibility::ExactComposition(preparable) => {
                    Ok(RendererFrameDispatch::ExactGraph(OwnedBox::new(
                        ExactSurfaceGraph::Composition(preparable),
                        &account.storage(),
                    )?))
                }
                ExecutableGraphDispatchEligibility::ExactBackdrop(preparable) => {
                    Ok(RendererFrameDispatch::ExactGraph(OwnedBox::new(
                        ExactSurfaceGraph::Backdrop(preparable),
                        &account.storage(),
                    )?))
                }
                ExecutableGraphDispatchEligibility::ExactForegroundFilters(preparable) => {
                    Ok(RendererFrameDispatch::ExactGraph(OwnedBox::new(
                        ExactSurfaceGraph::ForegroundFilters(preparable),
                        &account.storage(),
                    )?))
                }
                ExecutableGraphDispatchEligibility::FuturePasses => {
                    let error = match reject_future_graph_with_typed_diagnostic(&graph, account) {
                        Err(error) => error,
                        Ok(()) => Error::new(
                            BackendErrorCode::RenderFailed,
                            "a future GPU graph had no unavailable execution pass",
                        ),
                    };
                    Ok(RendererFrameDispatch::RejectedFutureGraph(error))
                }
            },
        }
    }
}

pub(super) fn reject_future_graph_with_typed_diagnostic(
    graph: &GpuRenderGraph,
    account: &Account,
) -> Result<()> {
    let mut has_copy_backdrop = false;
    let mut has_color_filter = false;
    let mut has_blur = false;
    let mut has_drop_shadow = false;
    let mut has_convolution = false;
    let mut has_offset = false;
    let mut has_gaussian = false;
    let mut has_native_shadow = false;
    let mut has_morphology = false;

    for pass in graph.lowering_view(account)?.passes(account)?.iter() {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        match pass.kind(account)? {
            GraphLoweringPassKind::ClearRoot { .. }
            | GraphLoweringPassKind::VelloCapture(Some(_))
            | GraphLoweringPassKind::CanonicalizeCapture
            | GraphLoweringPassKind::Present => {}
            GraphLoweringPassKind::CopyBackdrop => has_copy_backdrop = true,
            GraphLoweringPassKind::ColorFilter(Some(_)) => has_color_filter = true,
            GraphLoweringPassKind::Convolution(_) => has_convolution = true,
            GraphLoweringPassKind::Offset(_) => has_offset = true,
            GraphLoweringPassKind::Gaussian(_) => has_gaussian = true,
            GraphLoweringPassKind::NativeShadowMerge(_) => has_native_shadow = true,
            GraphLoweringPassKind::Morphology(_) => has_morphology = true,
            GraphLoweringPassKind::BlurHorizontal(Some(_))
            | GraphLoweringPassKind::BlurVertical(Some(_)) => has_blur = true,
            GraphLoweringPassKind::DropShadowColorize(Some(_)) => has_drop_shadow = true,
            GraphLoweringPassKind::Composite(Some(composite)) => match composite.kind() {
                GraphLoweringCompositeKind::SpanSourceOver => {}
                GraphLoweringCompositeKind::Layer { .. } => {}
                GraphLoweringCompositeKind::DropShadow => has_drop_shadow = true,
            },
            GraphLoweringPassKind::VelloCapture(None)
            | GraphLoweringPassKind::ColorFilter(None)
            | GraphLoweringPassKind::BlurHorizontal(None)
            | GraphLoweringPassKind::BlurVertical(None)
            | GraphLoweringPassKind::DropShadowColorize(None)
            | GraphLoweringPassKind::Composite(None) => {
                return Err(Error::new(
                    BackendErrorCode::RenderFailed,
                    "a malformed GPU graph reached production dispatch",
                ));
            }
        }
    }

    let unsupported = if has_copy_backdrop {
        Some((
            PrimitiveFamily::OffscreenPipeline,
            PrimitiveOperation::BroadBackdropExecution,
        ))
    } else if has_drop_shadow {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuDropShadowFilterExecution,
        ))
    } else if has_convolution {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuConvolutionFilterExecution,
        ))
    } else if has_morphology {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuMorphologyFilterExecution,
        ))
    } else if has_native_shadow {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuNativeDropShadowFilterExecution,
        ))
    } else if has_gaussian {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuGaussianFilterExecution,
        ))
    } else if has_offset {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuOffsetFilterExecution,
        ))
    } else if has_blur {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuBlurFilterExecution,
        ))
    } else if has_color_filter {
        Some((
            PrimitiveFamily::Filters,
            PrimitiveOperation::GpuColorFilterExecution,
        ))
    } else {
        None
    };
    if let Some((family, operation)) = unsupported {
        return Err(Error::unsupported_render_primitive(
            UnsupportedPrimitive::new(family, operation),
        ));
    }
    Err(Error::new(
        BackendErrorCode::RenderFailed,
        "a future GPU graph had no unavailable execution pass",
    ))
}
