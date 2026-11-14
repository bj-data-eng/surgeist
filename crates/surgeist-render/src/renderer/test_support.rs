#[cfg(feature = "render-window")]
use super::PresentedConfigurationOperation;
use super::{
    Renderer,
    dispatch::{RendererFrameDispatch, runtime_surface_format},
    publication::RenderPublication,
};
use super::{SurfaceDiagnosticOperation, SurfaceIoOperation};
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
use crate::surface::SurfaceBackend;
use crate::{
    accounting::{Account, GenerationClass, GpuObjectKind},
    backend::*,
    command::RenderCommands,
    frame::{FrameContext, FramePlan, GpuRenderGraph},
    gpu_transaction::GpuOperationStage,
    image::ImageContentIdentity,
    pass::{ExecutableGraphDispatchEligibility, ExecutableGraphWorkingFormatRequest},
    resource::{ResourceManagerObservationForTest, WorkingFormat},
    *,
};
#[cfg(feature = "render-window")]
use crate::{geometry::physical_size, validation::validate_surface_options};
use std::{collections::HashSet, sync::Arc, time::Duration};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ResourcePreparationObservationForTest {
    pub(crate) complete_resource_and_pass_handoff: bool,
    pub(crate) exact_capture_coverage_working_and_mask_allocations: bool,
    pub(crate) typed_bindings_and_last_use_releases: bool,
    pub(crate) spatial_bytes_and_cache_keys_preserved: bool,
    pub(crate) allocation_preflight_is_atomic: bool,
    pub(crate) failure_and_drop_cleanup: bool,
    pub(crate) repeated_reuse_is_exact_and_bounded: bool,
    pub(crate) populated_pass_cache_is_preserved: bool,
}

impl Renderer {
    pub(crate) fn set_default_device_effects_gpu_byte_limit_for_test(
        &mut self,
        limit: u64,
    ) -> Result<()> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "Effects byte-limit coverage requires a ready default device",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "Effects byte-limit coverage requires a renderer backend",
            )
        })?;
        backend
            .ready_device_state_borrow_for_test(device_identity)
            .ok_or_else(|| {
                Error::runtime_unavailable(
                    RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                    "Effects byte-limit coverage requires a ready selected device generation",
                )
            })?
            .generation_for_test()
            .set_effects_gpu_byte_limit_for_test(limit)
    }

    pub(crate) async fn submit_prepared_vello_pass_for_test(
        &mut self,
        prepared: &super::super::vello_engine::PreparedVelloPass,
        target_extent: PhysicalSize,
    ) -> Result<
        super::super::gpu_transaction::test_support::InternalVelloSubmissionObservationForTest,
    > {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello transaction coverage requires a ready default device",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello transaction coverage requires a renderer backend",
            )
        })?;
        backend
            .submit_prepared_vello_pass_for_test(device_identity, prepared, target_extent)
            .await
    }

    pub(crate) async fn fail_prepared_vello_pass_after_submit_for_test(
        &mut self,
        prepared: &super::super::vello_engine::PreparedVelloPass,
        target_extent: PhysicalSize,
        publication: &mut Option<u64>,
    ) -> Result<()> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello failure coverage requires a ready default device",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello failure coverage requires a renderer backend",
            )
        })?;
        backend
            .fail_prepared_vello_pass_after_submit_for_test(
                device_identity,
                prepared,
                target_extent,
                publication,
            )
            .await
    }

    pub(crate) async fn fault_prepared_vello_accounting_after_submit_for_test(
        &mut self,
        prepared: &super::super::vello_engine::PreparedVelloPass,
        target_extent: PhysicalSize,
    ) -> Result<()> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello accounting coverage requires a ready default device",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello accounting coverage requires a renderer backend",
            )
        })?;
        backend
            .fault_prepared_vello_accounting_after_submit_for_test(
                device_identity,
                prepared,
                target_extent,
            )
            .await
    }

    pub(crate) async fn cancel_prepared_vello_pass_after_submit_for_test(
        &mut self,
        prepared: &super::super::vello_engine::PreparedVelloPass,
        target_extent: PhysicalSize,
    ) -> Result<ResourceManagerObservationForTest> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello cancellation coverage requires a ready default device",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "internal Vello cancellation coverage requires a renderer backend",
            )
        })?;
        backend
            .cancel_prepared_vello_pass_after_submit_for_test(
                device_identity,
                prepared,
                target_extent,
            )
            .await
    }

    pub(crate) fn default_device_active_operation_generation_for_test(&mut self) -> Option<u64> {
        let device_identity = self.default_device?;
        self.backend
            .as_mut()?
            .active_operation_generation_for_test(device_identity)
    }

    pub(crate) fn default_device_has_no_terminal_signal_for_test(&mut self) -> bool {
        let Some(device_identity) = self.default_device else {
            return true;
        };
        self.backend
            .as_mut()
            .is_some_and(|backend| backend.terminal_reason(device_identity).is_none())
    }

    pub(crate) fn default_device_capabilities_for_test(&mut self) -> AvailableRuntimeCapabilities {
        let device_identity = self.default_device.expect("test requires a default device");
        self.backend
            .as_mut()
            .and_then(|backend| backend.device_capabilities(device_identity))
            .expect("test requires a ready default device")
            .runtime_report(Format::Rgba8)
    }

    pub(crate) fn override_default_device_filter_float32_fact_for_test(
        &mut self,
        supported: bool,
    ) -> bool {
        let Some(identity) = self.default_device else {
            return false;
        };
        self.backend.as_mut().is_some_and(|backend| {
            backend.override_device_filter_float32_fact_for_test(identity, supported)
        })
    }

    pub(crate) fn override_default_device_effect_precision_facts_for_test(
        &mut self,
        effect_precisions: EffectPrecisionCapabilities,
    ) -> bool {
        let Some(device_identity) = self.default_device else {
            return false;
        };
        self.backend.as_mut().is_some_and(|backend| {
            backend
                .override_device_effect_precision_facts_for_test(device_identity, effect_precisions)
        })
    }

    pub(crate) fn destroy_default_device_for_test(&mut self) -> bool {
        let Some(device_identity) = self.default_device else {
            return false;
        };
        let Some(backend) = self.backend.as_mut() else {
            return false;
        };
        backend.destroy_device_for_test(device_identity)
    }

    pub(crate) fn wait_for_default_terminal_signal_for_test(&mut self, timeout: Duration) -> bool {
        let Some(device_identity) = self.default_device else {
            return false;
        };
        self.backend
            .as_mut()
            .is_some_and(|backend| backend.wait_for_terminal_for_test(device_identity, timeout))
    }

    pub(crate) async fn add_donor_device_slot_for_test(&mut self) -> Result<DeviceSlotIdentity> {
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "the renderer has no backend to receive a donor wgpu device",
            )
        })?;
        backend.add_device_slot_for_test().await
    }

    pub(crate) async fn submit_scoped_wgpu_probe_for_test(
        &mut self,
        device_identity: DeviceSlotIdentity,
    ) -> Result<()> {
        let account = Account::for_test();
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "real second-slot WGPU coverage requires a renderer backend",
            )
        })?;
        let custody = backend.generation(device_identity)?.clone();
        let transaction = backend.begin_gpu_operation(
            device_identity,
            GpuOperationStage::Render,
            RuntimeOperation::SurfaceRendering,
        )?;
        let (device, queue) =
            backend.device_queue(device_identity, RuntimeOperation::SurfaceRendering)?;
        let command_buffer = {
            let texture =
                custody.allocation(GenerationClass::SurfaceTargets, 4, Some(&account), || {
                    device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Surgeist second-slot terminal test target"),
                        size: wgpu::Extent3d {
                            width: 1,
                            height: 1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    })
                })?;
            let view =
                texture.create_view(&wgpu::TextureViewDescriptor::default(), Some(&account))?;
            let mut encoder =
                custody.opaque(GpuObjectKind::CommandEncoder, Some(&account), || {
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Surgeist second-slot terminal test encoder"),
                    })
                })?;
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Surgeist second-slot terminal test pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
            }
            encoder.finish(Some(&account))
        };
        let scope_result = match command_buffer {
            Ok(command_buffer) => {
                super::super::gpu_transaction::test_support::submit_command_buffer_for_test(
                    transaction,
                    queue,
                    command_buffer,
                    RuntimeOperation::SurfaceRendering,
                )
                .await
            }
            Err(error) => match transaction
                .finish(RuntimeOperation::SurfaceRendering, Some(&account))
                .await
            {
                Ok(()) => Err(error),
                Err(scope_error) => Err(scope_error),
            },
        };
        backend.observe_device_terminal(device_identity);
        scope_result
    }
}

pub(crate) fn unsupported_graph_diagnostic_for_test(
    graph: &GpuRenderGraph,
    output_format: Format,
    capabilities: &DeviceCapabilities,
) -> Result<Option<UnsupportedPrimitive>> {
    match ExecutableGraphDispatchEligibility::try_classify(
        graph,
        output_format,
        ExecutableGraphWorkingFormatRequest::Exact(WorkingFormat::HighPrecision),
        capabilities,
        &Account::for_test(),
    )? {
        ExecutableGraphDispatchEligibility::FuturePasses => {
            let error = super::dispatch::reject_future_graph_with_typed_diagnostic(
                graph,
                &Account::for_test(),
            )
            .expect_err("an unsupported graph diagnostic probe must reject before execution");
            Ok(error.unsupported_primitive())
        }
        ExecutableGraphDispatchEligibility::ExactBase(_)
        | ExecutableGraphDispatchEligibility::ExactComposition(_)
        | ExecutableGraphDispatchEligibility::ExactBackdrop(_)
        | ExecutableGraphDispatchEligibility::ExactForegroundFilters(_) => Ok(None),
    }
}

impl Renderer {
    pub(crate) fn resource_preparation_observation_for_test(
        &mut self,
        commands: RenderCommands,
        surface_size: Size,
        surface_scale: f64,
        base_color: Color,
        output_format: Format,
    ) -> Result<ResourcePreparationObservationForTest> {
        let context = FrameContext::try_new(
            surface_size,
            surface_scale,
            self.options.antialiasing(),
            base_color,
        )?;
        let FramePlan::GpuGraph(graph) = commands.plan_for(context, &Account::for_test())? else {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the resource preparation fixture did not produce a GPU graph",
            ));
        };
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
                "resource preparation coverage requires a ready default device",
            )
        })?;
        let policy = self.options.effect_quality_policy();
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::adapter_selection_unavailable(),
                "resource preparation coverage requires a renderer backend",
            )
        })?;
        let capabilities = backend
            .device_capabilities(device_identity)
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "resource preparation coverage requires immutable device capabilities",
                )
            })?;
        let working_format = capabilities.resolve_effect_working_format(policy)?;
        let lowered = super::pass::LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            working_format,
            output_format,
            &capabilities,
            &Account::for_test(),
        )?;
        let pass_cache_before = backend
            .seed_device_pass_cache_sampler_for_test(device_identity)
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "ready device disappeared before pass-cache preservation coverage",
                )
            })?;

        let allocation_preflight_is_atomic =
            preparation_preflight_is_atomic(backend, device_identity, &lowered, policy)?;
        let (first_exercise, repeated_reuse_is_exact_and_bounded) =
            exercise_preparation_reuse(backend, device_identity, &lowered, policy)?;
        let failure_and_drop_cleanup =
            preparation_failure_cleanup(backend, device_identity, lowered, policy)?;
        let pass_cache_after = backend
            .ready_device_state_borrow_for_test(device_identity)
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "ready device disappeared after pass-cache preservation coverage",
                )
            })?
            .device_pass_cache_counts_for_test();
        let populated_pass_cache_is_preserved =
            pass_cache_before.has_exactly_one_sampler() && pass_cache_after == pass_cache_before;

        Ok(ResourcePreparationObservationForTest {
            complete_resource_and_pass_handoff: first_exercise.complete_resource_and_pass_handoff,
            exact_capture_coverage_working_and_mask_allocations: first_exercise
                .exact_capture_coverage_working_and_mask_allocations,
            typed_bindings_and_last_use_releases: first_exercise
                .typed_bindings_and_last_use_releases,
            spatial_bytes_and_cache_keys_preserved: first_exercise
                .spatial_bytes_and_cache_keys_preserved,
            allocation_preflight_is_atomic,
            failure_and_drop_cleanup,
            repeated_reuse_is_exact_and_bounded,
            populated_pass_cache_is_preserved,
        })
    }
}

impl Renderer {
    pub(crate) fn default_wgpu_device_queue(&mut self) -> Option<(&wgpu::Device, &wgpu::Queue)> {
        let backend = self.backend.as_mut()?;
        let device_identity = self.default_device?;
        backend
            .device_queue(device_identity, RuntimeOperation::SurfaceRendering)
            .ok()
    }

    pub(crate) fn default_offscreen_render_context(
        &mut self,
    ) -> Option<OffscreenRenderGpuContext<'_>> {
        let backend = self.backend.as_mut()?;
        let device_identity = self.default_device?;
        if backend
            .terminal_error(device_identity, RuntimeOperation::SurfaceRendering)
            .is_some()
        {
            return None;
        }
        Some(OffscreenRenderGpuContext::new(backend, device_identity))
    }

    pub(crate) async fn read_render_texture_for_test(
        &mut self,
        texture: &wgpu::Texture,
        physical_size: PhysicalSize,
    ) -> Result<ImageBuffer> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "required render-texture readback needs an available wgpu device",
            )
        })?;
        self.backend.as_ref().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "required render-texture readback needs an available wgpu backend",
            )
        })?;
        let finished = crate::accounting::readback_lifecycle(
            self,
            |renderer| (&renderer.cpu, renderer.options.accounting()),
            |renderer, report| renderer.readback_reports = Some(report),
            async |renderer, mut attempt, account| {
                let backend = renderer
                    .backend
                    .as_mut()
                    .ok_or(crate::RenderAccountingError::Invariant)?;
                attempt.attach_generation(backend.generation(device_identity)?)?;
                account
                    .async_phase(
                        crate::RenderPhase::Readback,
                        crate::readback::read_texture_rgba(
                            backend,
                            device_identity,
                            texture,
                            physical_size,
                            RuntimeOperation::SurfaceRendering,
                            account,
                        ),
                    )
                    .await
            },
        )
        .await?;
        self.finish_readback(finished)
    }

    pub(crate) fn signal_default_device_loss_for_test(&mut self, reason: DeviceLossReason) {
        if let Some(device_identity) = self.default_device {
            self.signal_device_loss_for_test(device_identity, reason);
        }
    }

    pub(crate) fn signal_device_loss_for_test(
        &mut self,
        device_identity: DeviceSlotIdentity,
        reason: DeviceLossReason,
    ) {
        if let Some(backend) = self.backend.as_mut() {
            backend.signal_loss_for_test(device_identity, reason);
        }
    }

    pub(crate) fn device_signal_for_test(
        &mut self,
        device_identity: DeviceSlotIdentity,
    ) -> Option<Arc<DeviceSignal>> {
        self.backend
            .as_mut()?
            .device_signal_for_test(device_identity)
    }

    pub(crate) fn default_device_signal_for_test(&mut self) -> Option<Arc<DeviceSignal>> {
        self.device_signal_for_test(self.default_device?)
    }

    pub(crate) fn signal_device_uncaptured_fault_for_test(
        &mut self,
        device_identity: DeviceSlotIdentity,
        kind: GpuFaultKind,
    ) {
        if let Some(backend) = self.backend.as_mut() {
            backend.signal_uncaptured_fault_for_test(device_identity, kind);
        }
    }

    pub(crate) fn default_device_renderer_released_for_test(&mut self) -> bool {
        match (self.backend.as_mut(), self.default_device) {
            (Some(backend), Some(device_identity)) => {
                backend.renderer_released_for_test(device_identity)
            }
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn device_renderer_released_for_test(
        &mut self,
        device_identity: DeviceSlotIdentity,
    ) -> bool {
        self.backend
            .as_mut()
            .is_some_and(|backend| backend.renderer_released_for_test(device_identity))
    }

    pub(crate) fn default_ready_device_state_borrow_for_test(
        &mut self,
    ) -> Option<ReadyDeviceStateBorrowForTest<'_>> {
        let device_identity = self.default_device?;
        self.backend
            .as_mut()?
            .ready_device_state_borrow_for_test(device_identity)
    }

    pub(crate) async fn deliberate_validation_error_for_test(&mut self) -> Result<Result<()>> {
        let account = Account::for_test();
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "real GPU error-scope coverage requires a host adapter",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "real GPU error-scope coverage requires a host adapter",
            )
        })?;
        let transaction = backend.begin_gpu_operation(
            device_identity,
            GpuOperationStage::Render,
            RuntimeOperation::SurfaceRendering,
        )?;
        let (device, _) =
            backend.device_queue(device_identity, RuntimeOperation::SurfaceRendering)?;
        let _ = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surgeist deliberate scoped validation failure"),
            size: wgpu::Extent3d {
                width: 0,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let result = transaction
            .finish(RuntimeOperation::SurfaceRendering, Some(&account))
            .await;
        backend.observe_device_terminal(device_identity);
        Ok(result)
    }

    pub(crate) async fn scoped_clear_fill_probe_for_test(&mut self) -> Result<ImageBuffer> {
        let account = Account::for_test();
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "real GPU clear/fill probe requires a host adapter",
            )
        })?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "real GPU clear/fill probe requires a host adapter",
            )
        })?;
        let custody = backend.generation(device_identity)?.clone();
        let transaction = backend.begin_gpu_operation(
            device_identity,
            GpuOperationStage::Render,
            RuntimeOperation::SurfaceRendering,
        )?;
        let (result, destination_texture) = {
            let destination = super::super::texture::TextureDescriptor::try_new(
                PhysicalSize::new(2, 2),
                Format::Rgba8,
                super::super::texture::TextureUsageIntent::IntermediatePass,
            )?;
            let (device, queue) =
                backend.device_queue(device_identity, RuntimeOperation::SurfaceRendering)?;
            let (destination_texture, destination_view) = create_texture(
                device,
                &custody,
                "Surgeist scoped clear destination",
                destination,
                Some(&account),
            )?;
            let mut encoder =
                custody.opaque(GpuObjectKind::CommandEncoder, Some(&account), || {
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Surgeist scoped test-only clear encoder"),
                    })
                })?;
            {
                let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Surgeist scoped test-only clear pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &destination_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.25,
                                g: 0.5,
                                b: 0.75,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
            }
            (
                super::super::gpu_transaction::test_support::submit_command_buffer_for_test(
                    transaction,
                    queue,
                    encoder.finish(Some(&account))?,
                    RuntimeOperation::SurfaceRendering,
                )
                .await,
                destination_texture,
            )
        };
        backend.observe_device_terminal(device_identity);
        result?;
        self.read_render_texture_for_test(&destination_texture, PhysicalSize::new(2, 2))
            .await
    }
}

#[cfg(all(feature = "render-window", not(target_arch = "wasm32")))]
mod presented_configuration_tests {
    use super::*;
    use crate::surface::{PresentedLifecycle, ResizeState};

    fn fixture() -> (Renderer, Surface) {
        let options = Options::default().with_accounting(
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
        );
        let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
        let mut surface = configured_display_free_presented_surface_for_test(&mut renderer);
        renderer.set_surface_resizing(&mut surface, true).unwrap();
        (renderer, surface)
    }

    fn measure_configure(
        renderer: &mut Renderer,
        surface: &mut Surface,
        limit: u64,
    ) -> (Result<()>, RenderStandaloneReport) {
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ResourceAllocationRequests, limit)
            .unwrap();
        let options = RenderAccountingOptions::new()
            .with_limits(limits)
            .with_clock(RenderClock::Host(|| Duration::ZERO))
            .with_profile(RenderProfile::Summary { timing: true });
        let mut context = RenderAccountingContext::new(options);
        let result = context.run(RenderAttemptKind::Frame, RenderPhase::Frame, |account| {
            pollster::block_on(
                renderer.configure_presented_surface_if_needed(surface, Some(account)),
            )
        });
        let report = context.take_last_attempt().unwrap();
        assert_eq!(report.cleanup_fault(), None);
        (result, report)
    }

    fn committed_extent(surface: &Surface) -> Option<PhysicalSize> {
        let SurfaceBackend::Presented { surface, .. } = &surface.backend else {
            panic!("the fixture must retain the presented surface");
        };
        surface.committed_physical_size()
    }

    #[test]
    fn positive_logical_sizes_rounded_to_zero_leave_the_committed_bundle_available_for_reuse() {
        let (mut renderer, mut surface) = fixture();
        let initial_resource = presented_resource_id_for_test(&surface).unwrap();
        let initial_target = presented_target_identity_for_test(&surface);
        let initial_device = presented_device_identity_for_test(&surface);
        let initial_custody = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
        let initial_observation = presented_observation_for_test(&surface);
        let initial_stats = renderer.stats();
        let committed = PhysicalSize::new(2, 2);
        assert_eq!(presented_configuration_count_for_test(&surface), 1);

        for (logical, rounded) in [
            (Size::new(0.49, 2.0), PhysicalSize::new(0, 2)),
            (Size::new(2.0, 0.49), PhysicalSize::new(2, 0)),
        ] {
            surface.resize(logical, 1.0).unwrap();
            assert_eq!(surface.size(), logical);
            assert_eq!(surface.physical_size(), rounded);
            assert_eq!(
                presented_lifecycle_for_test(&surface),
                PresentedLifecycle::NonRenderable {
                    physical_size: rounded,
                    resizing: ResizeState::Resizing,
                }
            );
            let (result, report) = measure_configure(&mut renderer, &mut surface, 0);
            result.unwrap();
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .phase_count(RenderPhase::Configure),
                0
            );
            assert_eq!(presented_configuration_count_for_test(&surface), 1);
            assert_eq!(
                presented_resource_id_for_test(&surface),
                Some(initial_resource)
            );
            assert_eq!(committed_extent(&surface), Some(committed));

            surface.resize(Size::new(2.0, 2.0), 1.0).unwrap();
            assert_eq!(surface.physical_size(), committed);
            assert_eq!(
                presented_lifecycle_for_test(&surface),
                PresentedLifecycle::Ready {
                    resizing: ResizeState::Resizing,
                }
            );
            let (result, report) = measure_configure(&mut renderer, &mut surface, 0);
            result.unwrap();
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .phase_count(RenderPhase::Configure),
                0
            );
            assert_eq!(presented_configuration_count_for_test(&surface), 1);
            assert_eq!(
                presented_resource_id_for_test(&surface),
                Some(initial_resource)
            );
            assert_eq!(committed_extent(&surface), Some(committed));
        }
        assert_eq!(presented_target_identity_for_test(&surface), initial_target);
        assert_eq!(presented_device_identity_for_test(&surface), initial_device);
        assert_eq!(
            presented_observation_for_test(&surface),
            initial_observation
        );
        assert_eq!(
            renderer.device_accounting_snapshot(Some(&surface)).unwrap(),
            initial_custody
        );
        assert_eq!(renderer.stats(), initial_stats);
        assert_eq!(surface.last_parameters, None);
    }

    #[test]
    fn presented_configure_admits_three_requests_before_the_host_effect() {
        let (mut renderer, mut surface) = fixture();
        let initial_resource = presented_resource_id_for_test(&surface).unwrap();
        let target = presented_target_identity_for_test(&surface);
        let device = presented_device_identity_for_test(&surface);
        let initial_observation = presented_observation_for_test(&surface);
        let initial_stats = renderer.stats();
        let initial_custody = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
        let extent = PhysicalSize::new(3, 2);
        let scope_usage =
            |snapshot: &RenderDeviceSnapshot, scope| snapshot.scope_usage(scope).unwrap();
        let surface_targets = RenderGenerationScope::SurfaceTargets;
        let views = RenderGenerationScope::TextureView;
        let blitters = RenderGenerationScope::DependencyUtility;
        let gpu_bytes = RenderResource::GpuBytes;
        let opaque = RenderResource::OpaqueGpuObjects;
        let requests = RenderWork::ResourceAllocationRequests;
        let allocations = RenderWork::ResourceAllocations;
        let creates = RenderWork::GpuObjectsCreated;
        assert_eq!(
            scope_usage(&initial_custody, surface_targets).live(gpu_bytes),
            16
        );
        let initial_views = scope_usage(&initial_custody, views);
        let initial_blitters = scope_usage(&initial_custody, blitters);
        surface.resize(Size::new(3.0, 2.0), 1.0).unwrap();
        let pending = presented_lifecycle_for_test(&surface);
        assert!(matches!(pending, PresentedLifecycle::ResizePending { .. }));
        assert_eq!(surface.physical_size(), extent);
        assert_eq!(pending.resize_state(), ResizeState::Resizing);

        for limit in [2, 3] {
            let (result, report) = measure_configure(&mut renderer, &mut surface, limit);
            let attempt = report.attempt().unwrap();
            assert_eq!(attempt.usage().work(requests), limit);
            assert_eq!(
                attempt.phase_usage(RenderPhase::Configure).work(requests),
                limit
            );
            assert_eq!(attempt.phase_count(RenderPhase::Configure), 1);
            assert_eq!(
                attempt.phase_elapsed(RenderPhase::Configure),
                Some(Duration::ZERO)
            );
            assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
            assert_eq!(attempt.usage().work(RenderWork::Presentations), 0);
            let custody = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
            assert_eq!(custody.identity(), initial_custody.identity());
            assert_eq!(custody.fault(), None);
            let targets = scope_usage(&custody, surface_targets);
            let current_views = scope_usage(&custody, views);
            let current_blitters = scope_usage(&custody, blitters);
            // The new 24-byte target overlaps the installed 16-byte target.
            assert_eq!(targets.peak(gpu_bytes), 40);
            assert_eq!(current_views.live(opaque), initial_views.live(opaque));
            assert_eq!(current_blitters.live(opaque), initial_blitters.live(opaque));

            if limit == 2 {
                let error = result.unwrap_err();
                let refusal = attempt.refusal().unwrap();
                assert_eq!(error.code(), ErrorCode::Accounting);
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(refusal))
                );
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(refusal.quantity(), RenderQuantity::Work(requests));
                assert_eq!((refusal.current(), refusal.amount()), (2, 1));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 2 });
                assert_eq!(targets.live(gpu_bytes), 16);
                assert_eq!(
                    targets.work(allocations),
                    scope_usage(&initial_custody, surface_targets).work(allocations) + 1
                );
                assert_eq!(current_views.work(creates), initial_views.work(creates) + 1);
                assert_eq!(
                    current_blitters.work(creates),
                    initial_blitters.work(creates)
                );
                assert_eq!(presented_configuration_count_for_test(&surface), 1);
                assert_eq!(
                    presented_resource_id_for_test(&surface),
                    Some(initial_resource)
                );
                assert_eq!(committed_extent(&surface), Some(PhysicalSize::new(2, 2)));
                assert_eq!(presented_lifecycle_for_test(&surface), pending);
            } else {
                result.unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
                assert_eq!(attempt.refusal(), None);
                assert_eq!(targets.live(gpu_bytes), 24);
                assert_eq!(
                    targets.work(allocations),
                    scope_usage(&initial_custody, surface_targets).work(allocations) + 2
                );
                assert_eq!(current_views.work(creates), initial_views.work(creates) + 2);
                assert_eq!(
                    current_blitters.work(creates),
                    initial_blitters.work(creates) + 1
                );
                assert_eq!(presented_configuration_count_for_test(&surface), 2);
                assert_ne!(
                    presented_resource_id_for_test(&surface).unwrap(),
                    initial_resource
                );
                assert_eq!(committed_extent(&surface), Some(extent));
                assert_eq!(
                    presented_lifecycle_for_test(&surface),
                    PresentedLifecycle::Ready {
                        resizing: ResizeState::Resizing,
                    }
                );
            }
            assert_eq!(presented_target_identity_for_test(&surface), target);
            assert_eq!(presented_device_identity_for_test(&surface), device);
            assert_eq!(
                presented_observation_for_test(&surface),
                initial_observation
            );
            assert_eq!(renderer.stats(), initial_stats);
            assert_eq!(surface.last_parameters, None);
            assert_eq!(
                renderer.default_device_active_operation_generation_for_test(),
                None
            );
        }
    }
}

#[cfg(feature = "render-window")]
impl Renderer {
    pub(crate) async fn configure_presented_surface_for_test(
        &mut self,
        surface: &mut Surface,
    ) -> Result<()> {
        self.configure_presented_surface_if_needed(surface, None)
            .await
    }

    pub(crate) fn display_free_presented_surface_for_test(
        &mut self,
        options: SurfaceOptions,
    ) -> Result<Surface> {
        let device_identity = self.default_device.ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "display-free presented configuration coverage requires a host adapter",
            )
        })?;
        self.display_free_presented_surface_on_device_for_test(
            options,
            device_identity,
            Attachment::from_web_canvas("display-free-presented-test-target"),
        )
    }

    pub(crate) fn display_free_presented_surface_on_device_for_test(
        &mut self,
        options: SurfaceOptions,
        device_identity: DeviceSlotIdentity,
        attachment: Attachment,
    ) -> Result<Surface> {
        validate_surface_options(options)?;
        if !matches!(&attachment, Attachment::WebCanvas(_)) {
            return Err(Error::new(
                BackendErrorCode::SurfaceCreateFailed,
                "the display-free presented fixture requires a web-canvas attachment",
            ));
        }
        let physical_size = physical_size(options.size, options.scale)?;
        let backend = self.backend.as_mut().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "display-free presented configuration coverage requires a renderer backend",
            )
        })?;
        if !backend.has_device_slot(device_identity) {
            return Err(Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "the display-free presented fixture requires a current device slot",
            ));
        }
        if let Some(error) =
            backend.terminal_error(device_identity, RuntimeOperation::SurfaceRendering)
        {
            return Err(error);
        }
        Ok(Surface::with_backend(
            attachment,
            options,
            SurfaceBackend::Presented {
                surface: Box::new(
                    super::super::surface::PresentedSurface::display_free_for_test(options.format),
                ),
                device_identity,
                state: super::super::surface::PresentedSurfaceState::new(
                    physical_size,
                    super::super::surface::ResizeState::Idle,
                ),
            },
            self.identity.clone(),
        ))
    }

    pub(crate) async fn resume_display_free_presented_surface_for_test(
        &mut self,
        surface: &mut Surface,
        attachment: Attachment,
    ) -> Result<()> {
        self.resume_display_free_presented_surface_with_loss_for_test(surface, attachment, None)
            .await
    }

    pub(crate) async fn resume_display_free_presented_surface_after_device_loss_for_test(
        &mut self,
        surface: &mut Surface,
        attachment: Attachment,
        reason: DeviceLossReason,
    ) -> Result<()> {
        self.resume_display_free_presented_surface_with_loss_for_test(
            surface,
            attachment,
            Some(reason),
        )
        .await
    }

    async fn resume_display_free_presented_surface_with_loss_for_test(
        &mut self,
        surface: &mut Surface,
        attachment: Attachment,
        loss_before_configuration: Option<DeviceLossReason>,
    ) -> Result<()> {
        if !surface.is_display_free_presented_for_test() {
            return Err(Error::new(
                BackendErrorCode::UnsupportedBackend,
                "display-free resume support requires its own presented fixture",
            ));
        }
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Resume)?;
        let SurfaceBackend::Presented {
            state,
            device_identity,
            ..
        } = &surface.backend
        else {
            unreachable!("display-free resume support requires a presented surface");
        };
        let device_identity = *device_identity;
        let renderer_identity = self.identity.clone();
        let backend =
            self.surface_backend_for_device(device_identity, SurfaceDiagnosticOperation::Resume)?;
        let action = Surface::presented_resume_action(surface.state, state.lifecycle());
        let resizing = state.lifecycle().resize_state();
        if let Some(error) =
            backend.terminal_error(device_identity, RuntimeOperation::SurfaceResume)
        {
            return Err(error);
        }
        surface.ensure_attachment_compatible(&attachment)?;
        match action {
            super::super::surface::PresentedResumeAction::NoOp => Ok(()),
            super::super::surface::PresentedResumeAction::ConfigureExisting => {
                if let Some(reason) = loss_before_configuration {
                    backend.signal_loss_for_test(device_identity, reason);
                }
                Self::configure_presented_surface_with_backend(
                    backend,
                    surface,
                    PresentedConfigurationOperation::Resume,
                    None,
                )
                .await
            }
            super::super::surface::PresentedResumeAction::Configure
            | super::super::surface::PresentedResumeAction::Recreate => {
                let mut next = Self::create_display_free_presented_surface_with_configuration_operation_for_test(
                    backend, renderer_identity, attachment, surface.options,
                    PresentedConfigurationOperation::Resume, Some(device_identity), loss_before_configuration,
                ).await?;
                next.last_parameters = surface.last_parameters;
                next.renderer_identity = surface.renderer_identity.clone();
                if let SurfaceBackend::Presented { state, .. } = &mut next.backend {
                    state.set_resizing(resizing);
                }
                *surface = next;
                Ok(())
            }
        }
    }

    async fn create_display_free_presented_surface_with_configuration_operation_for_test(
        backend: &mut Backend,
        renderer_identity: super::super::surface::RendererIdentity,
        attachment: Attachment,
        options: SurfaceOptions,
        configuration_operation: PresentedConfigurationOperation,
        preferred_device: Option<DeviceSlotIdentity>,
        loss_before_configuration: Option<DeviceLossReason>,
    ) -> Result<Surface> {
        validate_surface_options(options)?;
        if !matches!(&attachment, Attachment::WebCanvas(_)) {
            return Err(Error::new(
                BackendErrorCode::SurfaceCreateFailed,
                "the display-free presented fixture requires a web-canvas attachment",
            ));
        }
        let physical_size = physical_size(options.size, options.scale)?;
        let (surface, device_identity) = backend
            .create_display_free_presented_surface_for_test(
                preferred_device,
                configuration_operation.runtime_operation(),
                options.format,
            )
            .await?;
        if let Some(reason) = loss_before_configuration {
            backend.signal_loss_for_test(device_identity, reason);
        }
        super::ensure_presented_device_available_after_creation(
            backend,
            device_identity,
            configuration_operation.runtime_operation(),
        )?;
        let mut created = Surface::with_backend(
            attachment,
            options,
            SurfaceBackend::Presented {
                surface: Box::new(surface),
                device_identity,
                state: super::super::surface::PresentedSurfaceState::new(
                    physical_size,
                    super::super::surface::ResizeState::Idle,
                ),
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

#[derive(Debug)]
pub(crate) struct ForcedGraphRenderResultForTest {
    pub(crate) stats: Stats,
    pub(crate) working_format: WorkingFormat,
    pub(crate) output_extent: PhysicalSize,
    pub(crate) captures: Vec<ForcedGraphCaptureForTest>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ForcedGraphCaptureForTest {
    pub(crate) antialiasing: Antialiasing,
    pub(crate) capture_transform: Transform,
    pub(crate) parent_to_surface: Transform,
    pub(crate) device_origin: (i32, i32),
    pub(crate) texel_origin: Point,
    pub(crate) extent: PhysicalSize,
    pub(crate) raster_scale: f64,
}

impl From<super::frame::ForcedGraphCaptureObservationForTest> for ForcedGraphCaptureForTest {
    fn from(capture: super::frame::ForcedGraphCaptureObservationForTest) -> Self {
        Self {
            antialiasing: capture.antialiasing,
            capture_transform: capture.capture_transform,
            parent_to_surface: capture.parent_to_surface,
            device_origin: capture.device_origin,
            texel_origin: capture.texel_origin,
            extent: capture.extent,
            raster_scale: capture.raster_scale,
        }
    }
}

struct ForcedGraphPreparationForTest {
    device_identity: DeviceSlotIdentity,
    normalized: RenderCommands,
    graph: crate::accounting::OwnedBox<RendererExactSurfaceGraph>,
    output_extent: PhysicalSize,
    captures: Vec<ForcedGraphCaptureForTest>,
}

fn preparation_resource_observation(
    backend: &mut Backend,
    identity: DeviceSlotIdentity,
    missing: &'static str,
) -> Result<ResourceManagerObservationForTest> {
    backend
        .ready_device_state_borrow_for_test(identity)
        .ok_or_else(|| Error::new(BackendErrorCode::RenderFailed, missing))
        .map(|ready| ready.internal_resource_manager_observation_for_test())
}

fn preparation_preflight_is_atomic(
    backend: &mut Backend,
    identity: DeviceSlotIdentity,
    lowered: &super::pass::LoweredGraphPlan,
    policy: EffectQualityPolicy,
) -> Result<bool> {
    let before = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared before preparation preflight",
    )?;
    let rejected = backend
        .prepare_graph_resources(
            identity,
            lowered.with_duplicate_preparation_resource_for_test(),
            policy,
            &Account::for_test(),
        )
        .is_err();
    let after = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared after preparation preflight",
    )?;
    Ok(rejected && before == after)
}

fn exercise_preparation_reuse(
    backend: &mut Backend,
    identity: DeviceSlotIdentity,
    lowered: &super::pass::LoweredGraphPlan,
    policy: EffectQualityPolicy,
) -> Result<(super::pass::PreparedGraphExerciseObservationForTest, bool)> {
    let (first_exercise, first_identities) = {
        let mut prepared = backend.prepare_graph_resources(
            identity,
            lowered.clone(),
            policy,
            &Account::for_test(),
        )?;
        let identities = prepared.allocation_identities_for_test();
        let exercise = prepared.exercise_for_test()?;
        let _ = prepared.finish()?;
        (exercise, identities)
    };
    let after_first = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared after first complete preparation",
    )?;
    let second_identities = {
        let mut prepared = backend.prepare_graph_resources(
            identity,
            lowered.clone(),
            policy,
            &Account::for_test(),
        )?;
        let identities = prepared.allocation_identities_for_test();
        let _ = prepared.exercise_for_test()?;
        let _ = prepared.finish()?;
        identities
    };
    let after_second = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared after repeated complete preparation",
    )?;
    let reuse = first_identities == second_identities
        && after_second.payload_creation_attempts == after_first.payload_creation_attempts
        && after_second.entry_count == after_first.entry_count
        && after_second.retained_bytes == after_first.retained_bytes;
    Ok((first_exercise, reuse))
}

fn preparation_failure_cleanup(
    backend: &mut Backend,
    identity: DeviceSlotIdentity,
    lowered: super::pass::LoweredGraphPlan,
    policy: EffectQualityPolicy,
) -> Result<bool> {
    let early_finish_failed = {
        let prepared = backend.prepare_graph_resources(
            identity,
            lowered.clone(),
            policy,
            &Account::for_test(),
        )?;
        prepared.finish().is_err()
    };
    let after_finish = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared after failed prepared finish",
    )?;
    drop(backend.prepare_graph_resources(identity, lowered, policy, &Account::for_test())?);
    let after_drop = preparation_resource_observation(
        backend,
        identity,
        "ready device disappeared after prepared cancellation",
    )?;
    Ok(early_finish_failed && after_finish.leased_count == 0 && after_drop.leased_count == 0)
}

#[derive(Debug)]
pub(crate) struct ColorFilterRenderResultForTest {
    pub(crate) stats: Stats,
    pub(crate) working_format: WorkingFormat,
    pub(crate) output_extent: PhysicalSize,
    pub(crate) source_origin: (i32, i32),
    pub(crate) source_extent: PhysicalSize,
    pub(crate) source_texel_origin: Point,
    pub(crate) source_raster_scale: f64,
}

#[derive(Debug)]
pub(crate) struct SpatialFilterRenderResultForTest {
    pub(crate) stats: Stats,
    pub(crate) working_format: WorkingFormat,
    pub(crate) output_extent: PhysicalSize,
    pub(crate) source_spatial: super::pass::ColorFilterSpatialObservationForTest,
    pub(crate) result_spatial: super::pass::ColorFilterSpatialObservationForTest,
}

#[derive(Debug)]
pub(crate) struct BoundedBackdropRenderResultForTest {
    pub(crate) stats: Stats,
    pub(crate) working_format: WorkingFormat,
    pub(crate) output_extent: PhysicalSize,
    pub(crate) parent_spatial: super::pass::ColorFilterSpatialObservationForTest,
    pub(crate) capture_spatial: super::pass::ColorFilterSpatialObservationForTest,
}

struct ColorFilterFixturePreparationForTest {
    device_identity: DeviceSlotIdentity,
    normalized: RenderCommands,
    graph: ExactSurfaceGraph,
    output_extent: PhysicalSize,
    source_spatial: super::pass::ColorFilterSpatialObservationForTest,
}

struct SpatialFilterFixturePreparationForTest {
    device_identity: DeviceSlotIdentity,
    normalized: RenderCommands,
    graph: ExactSurfaceGraph,
    output_extent: PhysicalSize,
    source_spatial: super::pass::ColorFilterSpatialObservationForTest,
    result_spatial: super::pass::ColorFilterSpatialObservationForTest,
}

struct BoundedBackdropFixturePreparationForTest {
    device_identity: DeviceSlotIdentity,
    normalized: RenderCommands,
    graph: ExactSurfaceGraph,
    output_extent: PhysicalSize,
    parent_spatial: super::pass::ColorFilterSpatialObservationForTest,
    capture_spatial: super::pass::ColorFilterSpatialObservationForTest,
}

impl Renderer {
    pub(crate) fn uploaded_images_for_test(&self) -> HashSet<ImageContentIdentity> {
        self.image_history.identities_for_test()
    }
}

impl Renderer {
    pub(crate) async fn render_with_exact_graph_working_format_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
    ) -> Result<Stats> {
        self.frame_operation(surface, async |renderer, surface, account| {
            let (_, publication) = renderer
                .dispatch_render_frame_with_working_format(
                    surface,
                    scene,
                    parameters,
                    ExecutableGraphWorkingFormatRequest::Exact(working_format),
                    account,
                )
                .await?;
            Ok(publication)
        })
        .await
    }

    fn classify_color_filter_fixture_dispatch(
        &self,
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<super::pass::ColorFilterPreparableGraph> {
        let preparable = super::pass::color_filter_preparable_graph_for_test(
            graph,
            output_format,
            working_format,
            capabilities,
            account,
        )?;
        Ok(preparable)
    }

    fn classify_spatial_filter_fixture_dispatch(
        &self,
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<super::pass::SpatialFilterPreparableGraph> {
        let preparable = super::pass::spatial_filter_preparable_graph_from_graph_for_test(
            graph,
            output_format,
            working_format,
            capabilities,
            account,
        )?;
        Ok(preparable)
    }

    fn classify_bounded_backdrop_fixture_dispatch(
        &self,
        graph: &GpuRenderGraph,
        output_format: Format,
        working_format: WorkingFormat,
        capabilities: &DeviceCapabilities,
        account: &Account,
    ) -> Result<super::pass::BackdropPreparableGraph> {
        let preparable = super::pass::backdrop_preparable_graph_from_graph_for_test(
            graph,
            output_format,
            working_format,
            capabilities,
            account,
        )?;
        Ok(preparable)
    }

    /// Test-only entry for forcing ordinary commands through the exact
    /// production graph executor without adding a public route or option.
    pub(crate) async fn render_forced_base_graph_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
    ) -> Result<ForcedGraphRenderResultForTest> {
        self.render_forced_base_graph_with_capture_mapping_for_test(
            surface,
            scene,
            parameters,
            working_format,
            super::frame::ForcedVelloCaptureMappingForTest::identity(),
        )
        .await
    }

    /// Test-only entry that keeps capture and parent mappings distinct while
    /// executing the same production graph path.
    pub(crate) async fn render_forced_base_graph_with_capture_mapping_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
        capture_mapping: super::frame::ForcedVelloCaptureMappingForTest,
    ) -> Result<ForcedGraphRenderResultForTest> {
        let mut observation = None;
        let stats = self
            .frame_operation(surface, async |renderer, surface, account| {
                let ForcedGraphPreparationForTest {
                    device_identity,
                    normalized,
                    graph,
                    output_extent,
                    captures,
                } = account.phase(crate::RenderPhase::FramePrepare, || {
                    renderer.prepare_forced_base_graph_for_test(
                        surface,
                        scene,
                        parameters,
                        working_format,
                        capture_mapping,
                        account,
                    )
                })?;
                renderer
                    .configure_presented_surface_if_needed(surface, Some(account))
                    .await?;
                let frame = renderer
                    .execute_exact_graph_for_test(
                        surface,
                        graph,
                        parameters,
                        device_identity,
                        account,
                    )
                    .await?;
                drop(normalized);
                observation = Some((output_extent, captures));
                Ok(RenderPublication::new(
                    frame,
                    parameters,
                    RenderRoute::GpuGraph,
                ))
            })
            .await?;
        let (output_extent, captures) =
            observation.expect("a successful forced frame records its capture facts");
        Ok(ForcedGraphRenderResultForTest {
            stats,
            working_format,
            output_extent,
            captures,
        })
    }

    async fn execute_fixture_graph_for_test(
        &mut self,
        surface: &mut Surface,
        device_identity: DeviceSlotIdentity,
        graph: ExactSurfaceGraph,
        account: &Account,
        reconfigure_outdated: bool,
    ) -> Result<SurfaceFrameCommit> {
        let frame = account
            .async_phase(crate::RenderPhase::Render, async {
                let backend = self
                    .backend
                    .as_mut()
                    .expect("fixture preflight confirmed its renderer backend");
                #[cfg(any(
                    feature = "render-window",
                    all(feature = "render-web", target_arch = "wasm32")
                ))]
                {
                    if matches!(&surface.backend, SurfaceBackend::Presented { .. }) {
                        render_exact_presented_graph_surface(backend, surface, graph, account).await
                    } else {
                        render_exact_headless_graph_surface(backend, surface, graph, account).await
                    }
                }
                #[cfg(not(any(
                    feature = "render-window",
                    all(feature = "render-web", target_arch = "wasm32")
                )))]
                {
                    render_exact_headless_graph_surface(backend, surface, graph, account).await
                }
            })
            .await;
        if frame.is_err()
            && let Some(backend) = self.backend.as_mut()
        {
            backend.observe_device_terminal(device_identity);
        }
        if reconfigure_outdated
            && matches!(&frame, Err(error) if error.code() == ErrorCode::SurfaceOutdated)
        {
            self.configure_presented_surface_if_needed(surface, Some(account))
                .await?;
        }
        frame
    }

    fn prepare_forced_base_graph_for_test(
        &mut self,
        surface: &Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
        capture_mapping: super::frame::ForcedVelloCaptureMappingForTest,
        account: &Account,
    ) -> Result<ForcedGraphPreparationForTest> {
        let device_identity = self.validate_forced_graph_surface_for_test(surface)?;
        let normalized = scene.normalize(self.capabilities(), account)?;
        let context = FrameContext::try_new(
            surface.size(),
            surface.scale(),
            self.options.antialiasing(),
            parameters.base_color,
        )?;
        let graph = super::frame::forced_base_graph_with_capture_mapping_for_test(
            normalized.clone(),
            context,
            capture_mapping,
            account,
        )?;
        let captures = graph.forced_capture_observations_for_test();
        let capabilities = self
            .backend
            .as_mut()
            .and_then(|backend| backend.device_capabilities(device_identity))
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private base graph forced route lost immutable device capabilities",
                )
            })?;
        let dispatch = self.classify_frame_dispatch(
            FramePlan::GpuGraph(crate::accounting::OwnedBox::new(graph, &account.storage())?),
            runtime_surface_format(surface),
            ExecutableGraphWorkingFormatRequest::Exact(working_format),
            &capabilities,
            account,
        )?;
        let RendererFrameDispatch::ExactGraph(graph) = dispatch else {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the private forced graph is outside the exact executable base graph subset",
            ));
        };
        let RendererExactSurfaceGraph::Base(preparable) = &*graph else {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the private forced graph is outside the exact executable base graph subset",
            ));
        };
        let output_extent = preparable.output_extent(account)?;
        let prepared_grids = preparable.capture_grids_for_test();
        if captures.len() != prepared_grids.len()
            || captures
                .iter()
                .zip(&prepared_grids)
                .any(|(capture, prepared)| {
                    capture.texel_origin != prepared.texel_origin
                        || capture.extent != prepared.extent
                        || capture.raster_scale != prepared.raster_scale
                })
        {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the prepared Vello capture grid differs from the validated semantic graph",
            ));
        }
        Ok(ForcedGraphPreparationForTest {
            device_identity,
            normalized,
            graph,
            output_extent,
            captures: captures
                .into_iter()
                .map(ForcedGraphCaptureForTest::from)
                .collect(),
        })
    }

    fn validate_forced_graph_surface_for_test(
        &mut self,
        surface: &Surface,
    ) -> Result<DeviceSlotIdentity> {
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        self.validate_surface_device_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        surface.ensure_available(SurfaceIoOperation::Rendering)?;
        surface.ensure_renderable()?;
        self.validate_surface_device_terminal(surface, RuntimeOperation::SurfaceRendering)?;
        surface.device_identity().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "the private base graph forced route requires a device-backed surface",
            )
        })
    }

    /// Test-only color-filter ingress into the shared exact graph executor.
    pub(crate) async fn render_color_filter_fixture_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        filters: Vec<FilterList>,
        parameters: Parameters,
        working_format: WorkingFormat,
    ) -> Result<ColorFilterRenderResultForTest> {
        let mut observation = None;
        let stats = self
            .frame_operation(surface, async |renderer, surface, account| {
                let ColorFilterFixturePreparationForTest {
                    device_identity,
                    normalized,
                    graph,
                    output_extent,
                    source_spatial,
                } = account.phase(crate::RenderPhase::FramePrepare, || {
                    renderer.prepare_color_filter_fixture_for_test(
                        surface,
                        scene,
                        filters,
                        parameters,
                        working_format,
                        account,
                    )
                })?;
                renderer
                    .configure_presented_surface_if_needed(surface, Some(account))
                    .await?;
                let frame = renderer
                    .execute_fixture_graph_for_test(surface, device_identity, graph, account, true)
                    .await?;
                drop(normalized);
                observation = Some((output_extent, source_spatial));
                Ok(RenderPublication::new(
                    frame,
                    parameters,
                    RenderRoute::GpuGraph,
                ))
            })
            .await?;
        let (output_extent, source_spatial) =
            observation.expect("a successful fixture frame records its spatial facts");
        Ok(ColorFilterRenderResultForTest {
            stats,
            working_format,
            output_extent,
            source_origin: source_spatial.device_origin,
            source_extent: source_spatial.device_extent,
            source_texel_origin: source_spatial.texel_origin,
            source_raster_scale: source_spatial.raster_scale,
        })
    }

    fn prepare_color_filter_fixture_for_test(
        &mut self,
        surface: &Surface,
        scene: &Scene,
        filters: Vec<FilterList>,
        parameters: Parameters,
        working_format: WorkingFormat,
        account: &Account,
    ) -> Result<ColorFilterFixturePreparationForTest> {
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        self.validate_surface_device_identity(surface, SurfaceDiagnosticOperation::Rendering)?;
        surface.ensure_available(SurfaceIoOperation::Rendering)?;
        surface.ensure_renderable()?;
        self.validate_surface_device_terminal(surface, RuntimeOperation::SurfaceRendering)?;
        let device_identity = surface.device_identity().ok_or_else(|| {
            Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                "the private color-filter fixture requires a device-backed surface",
            )
        })?;
        let normalized = scene.normalize(self.capabilities(), account)?;
        let context = FrameContext::try_new(
            surface.size(),
            surface.scale(),
            self.options.antialiasing(),
            parameters.base_color,
        )?;
        let graph = super::frame::authored_filter_graph_for_test(
            filters,
            normalized.clone(),
            context,
            account,
        )?;
        let capabilities = self
            .backend
            .as_mut()
            .ok_or_else(|| {
                Error::runtime_unavailable(
                    RuntimeCapabilityUnavailable::rendering_adapter_unavailable(),
                    "the private color-filter fixture requires a renderer backend",
                )
            })?
            .device_capabilities(device_identity)
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private color-filter fixture lost immutable device capabilities",
                )
            })?;
        let preparable = self.classify_color_filter_fixture_dispatch(
            &graph,
            runtime_surface_format(surface),
            working_format,
            &capabilities,
            account,
        )?;
        let output_extent = preparable.output_extent(account)?;
        let source_spatial = preparable.first_color_spatial_for_test().ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "the private color-filter fixture lost its first exact color source",
            )
        })?;
        Ok(ColorFilterFixturePreparationForTest {
            device_identity,
            normalized,
            graph: ExactSurfaceGraph::ColorFilter(preparable),
            output_extent,
            source_spatial,
        })
    }

    /// Test-only spatial-filter ingress into the shared exact graph executor.
    pub(crate) async fn render_spatial_filter_fixture_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        filters: Vec<FilterList>,
        parameters: Parameters,
        working_format: WorkingFormat,
    ) -> Result<SpatialFilterRenderResultForTest> {
        let mut observation = None;
        let stats = self
            .frame_operation(surface, async |renderer, surface, account| {
                let SpatialFilterFixturePreparationForTest {
                    device_identity,
                    normalized,
                    graph,
                    output_extent,
                    source_spatial,
                    result_spatial,
                } = account.phase(crate::RenderPhase::FramePrepare, || {
                    renderer.prepare_spatial_filter_fixture_for_test(
                        surface,
                        scene,
                        filters,
                        parameters,
                        working_format,
                        account,
                    )
                })?;
                renderer
                    .configure_presented_surface_if_needed(surface, Some(account))
                    .await?;
                let frame = renderer
                    .execute_fixture_graph_for_test(surface, device_identity, graph, account, false)
                    .await?;
                drop(normalized);
                observation = Some((output_extent, source_spatial, result_spatial));
                Ok(RenderPublication::new(
                    frame,
                    parameters,
                    RenderRoute::GpuGraph,
                ))
            })
            .await?;
        let (output_extent, source_spatial, result_spatial) =
            observation.expect("a successful fixture frame records its spatial facts");
        Ok(SpatialFilterRenderResultForTest {
            stats,
            working_format,
            output_extent,
            source_spatial,
            result_spatial,
        })
    }

    fn prepare_spatial_filter_fixture_for_test(
        &mut self,
        surface: &Surface,
        scene: &Scene,
        filters: Vec<FilterList>,
        parameters: Parameters,
        working_format: WorkingFormat,
        account: &Account,
    ) -> Result<SpatialFilterFixturePreparationForTest> {
        let device_identity = self.validate_forced_graph_surface_for_test(surface)?;
        let normalized = scene.normalize(self.capabilities(), account)?;
        let context = FrameContext::try_new(
            surface.size(),
            surface.scale(),
            self.options.antialiasing(),
            parameters.base_color,
        )?;
        let graph = super::frame::authored_filter_graph_for_test(
            filters,
            normalized.clone(),
            context,
            account,
        )?;
        let capabilities = self
            .backend
            .as_mut()
            .and_then(|backend| backend.device_capabilities(device_identity))
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private spatial-filter fixture lost immutable device capabilities",
                )
            })?;
        let preparable = self.classify_spatial_filter_fixture_dispatch(
            &graph,
            runtime_surface_format(surface),
            working_format,
            &capabilities,
            account,
        )?;
        let output_extent = preparable.output_extent(account)?;
        let (source_spatial, result_spatial) =
            preparable.first_filter_spatial_for_test().ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private spatial-filter fixture lost its first spatial mapping",
                )
            })?;
        Ok(SpatialFilterFixturePreparationForTest {
            device_identity,
            normalized,
            graph: ExactSurfaceGraph::SpatialFilter(preparable),
            output_extent,
            source_spatial,
            result_spatial,
        })
    }

    /// Test-only bounded-backdrop ingress into the shared exact graph executor.
    pub(crate) async fn render_bounded_backdrop_fixture_for_test(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
    ) -> Result<BoundedBackdropRenderResultForTest> {
        let mut observation = None;
        let stats = self
            .frame_operation(surface, async |renderer, surface, account| {
                let BoundedBackdropFixturePreparationForTest {
                    device_identity,
                    normalized,
                    graph,
                    output_extent,
                    parent_spatial,
                    capture_spatial,
                } = account.phase(crate::RenderPhase::FramePrepare, || {
                    renderer.prepare_bounded_backdrop_fixture_for_test(
                        surface,
                        scene,
                        parameters,
                        working_format,
                        account,
                    )
                })?;
                renderer
                    .configure_presented_surface_if_needed(surface, Some(account))
                    .await?;
                let frame = renderer
                    .execute_fixture_graph_for_test(surface, device_identity, graph, account, false)
                    .await?;
                drop(normalized);
                observation = Some((output_extent, parent_spatial, capture_spatial));
                Ok(RenderPublication::new(
                    frame,
                    parameters,
                    RenderRoute::GpuGraph,
                ))
            })
            .await?;
        let (output_extent, parent_spatial, capture_spatial) =
            observation.expect("a successful fixture frame records its spatial facts");
        Ok(BoundedBackdropRenderResultForTest {
            stats,
            working_format,
            output_extent,
            parent_spatial,
            capture_spatial,
        })
    }

    fn prepare_bounded_backdrop_fixture_for_test(
        &mut self,
        surface: &Surface,
        scene: &Scene,
        parameters: Parameters,
        working_format: WorkingFormat,
        account: &Account,
    ) -> Result<BoundedBackdropFixturePreparationForTest> {
        let device_identity = self.validate_forced_graph_surface_for_test(surface)?;
        let normalized = scene.normalize(self.capabilities(), account)?;
        let context = FrameContext::try_new(
            surface.size(),
            surface.scale(),
            self.options.antialiasing(),
            parameters.base_color,
        )?;
        let FramePlan::GpuGraph(graph) = normalized.clone().plan_for(context, account)? else {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "the private bounded-backdrop fixture did not produce a bounded backdrop graph",
            ));
        };
        let capabilities = self
            .backend
            .as_mut()
            .and_then(|backend| backend.device_capabilities(device_identity))
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private bounded-backdrop fixture lost immutable device capabilities",
                )
            })?;
        let preparable = self.classify_bounded_backdrop_fixture_dispatch(
            &graph,
            runtime_surface_format(surface),
            working_format,
            &capabilities,
            account,
        )?;
        let output_extent = preparable.output_extent(account)?;
        let (parent_spatial, capture_spatial) =
            preparable.backdrop_spatial_for_test().ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "the private bounded-backdrop fixture lost its exact backdrop mapping",
                )
            })?;
        Ok(BoundedBackdropFixturePreparationForTest {
            device_identity,
            normalized,
            graph: ExactSurfaceGraph::Backdrop(preparable),
            output_extent,
            parent_spatial,
            capture_spatial,
        })
    }
}
