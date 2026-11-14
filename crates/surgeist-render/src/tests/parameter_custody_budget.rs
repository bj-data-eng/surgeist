//! Descriptor-derived child/ancestor residency and actual preparation phase.
use super::convolution::{assert_pixel, bounds};
use crate::{
    Color, FilterColorSpace, GaussianSource, Layer, Options, Parameters, Rect,
    RenderAccountingError, RenderAccountingOptions, RenderGenerationScope as Scope, RenderProfile,
    RenderQuantity, RenderResource, Renderer, ResourceCacheBudget, Scene, Size,
    gpu_transaction::test_support::GraphSubmissionGateGuardForTest,
    pass::{
        ParameterFamilyForTest as Family, ParameterProbeForTest, ParameterStageForTest as Stage,
    },
    resource::WorkingFormat,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

fn constant_scene() -> Scene {
    let full = bounds(4, 1);
    let mut scene = Scene::new();
    scene
        .gaussian_layer(
            Layer::new(),
            super::gaussian::operation(1.0, 1.0, FilterColorSpace::Srgb),
            GaussianSource::constant(Color::BLACK, full, Rect::new(1.0, 0.0, 2.0, 1.0)).unwrap(),
        )
        .unwrap();
    scene
}
fn render(renderer: &mut Renderer, surface: &mut crate::Surface, scene: &Scene) -> crate::Stats {
    pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
        surface,
        scene,
        Parameters::default(),
        WorkingFormat::HighPrecision,
    ))
    .unwrap()
}
fn held<F: Future>(
    mut future: Pin<&mut F>,
    gate: &GraphSubmissionGateGuardForTest,
    device: &wgpu::Device,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        if gate.submitted_generation().is_some() {
            assert_eq!(gate.submissions(), 1);
            return;
        }
        assert!(
            Instant::now() < deadline,
            "real parameter graph did not submit"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}
fn complete<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device) -> F::Output {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Poll::Ready(value) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "real parameter graph did not resolve"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

#[test]
fn graph_parameters_use_exact_pass_external_and_combined_root_limits_with_cold_and_warm_cache() {
    // One Gaussian256, Layer spatial48+composite128 and Present spatial48.
    const PARAMETERS: u64 = 256 + 48 + 128 + 48;
    // High precision root4x1+constant result2x1+composite4x1, no Vello input.
    const EFFECTS: u64 = (4 + 2 + 4) * 8;
    const DRAFT: u64 = 4 * 4;
    let scene = constant_scene();
    for (retention, warm) in [
        (ResourceCacheBudget::DISABLED, false),
        (ResourceCacheBudget::DEFAULT, false),
        (ResourceCacheBudget::DEFAULT, true),
    ] {
        for scope in [Scope::PassBuffers, Scope::External, Scope::Root] {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_resource_cache_budget(retention)
                    .with_accounting(
                        RenderAccountingOptions::new()
                            .with_profile(RenderProfile::Summary { timing: false }),
                    ),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
            if warm {
                render(&mut renderer, &mut surface, &scene);
            }
            let stats = renderer.stats();
            let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let publications = surface.headless_publication_count_for_test();
            let ready = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap();
            let device = ready.device_for_test().clone();
            let reader = ready.resource_manager_observation_reader_for_test();
            let opening = reader();
            assert_eq!(opening.effects_gpu_bytes, if warm { EFFECTS } else { 0 });
            let snapshot = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
            assert_eq!(
                snapshot
                    .scope_usage(Scope::PassBuffers)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                0
            );
            let existing = snapshot
                .scope_usage(scope)
                .unwrap()
                .live(RenderResource::GpuBytes);
            let missing_cache = if warm { 0 } else { EFFECTS };
            let exact = match scope {
                Scope::PassBuffers => PARAMETERS,
                Scope::External => existing + DRAFT + PARAMETERS,
                Scope::Root => existing + DRAFT + missing_cache + PARAMETERS,
                _ => unreachable!(),
            };
            renderer
                .set_default_device_gpu_byte_limit_for_test(scope, exact - 1)
                .unwrap();
            let error =
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    WorkingFormat::HighPrecision,
                ))
                .unwrap_err();
            assert!(
                matches!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::GenerationRefused {
                        quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                        ..
                    })
                ),
                "{scope:?}: {error:?}"
            );
            let refused = reader();
            assert_eq!(
                refused.payload_creation_attempts,
                opening.payload_creation_attempts
            );
            assert_eq!(refused.effects_gpu_bytes, opening.effects_gpu_bytes);
            assert_eq!(renderer.stats(), stats);
            assert_eq!(surface.headless_publication_count_for_test(), publications);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                old_pixels
            );
            renderer
                .set_default_device_gpu_byte_limit_for_test(scope, exact)
                .unwrap();
            let probe = ParameterProbeForTest::new(
                renderer
                    .default_ready_device_state_borrow_for_test()
                    .unwrap()
                    .generation_for_test(),
            );
            let observe = probe.reader();
            let signal = renderer.default_device_signal_for_test().unwrap();
            let gate = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
            {
                let mut future =
                    Box::pin(renderer.render_with_exact_graph_working_format_for_test(
                        &mut surface,
                        &scene,
                        Parameters::default(),
                        WorkingFormat::HighPrecision,
                    ));
                held(future.as_mut(), &gate, &device);
                let observed = observe();
                let prepared = observed
                    .iter()
                    .find(|s| s.stage == Stage::Prepared)
                    .unwrap();
                let mut descriptors = prepared
                    .buffers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|b| (b.bytes, b.family))
                    .collect::<Vec<_>>();
                descriptors.sort_by_key(|b| b.0);
                assert_eq!(
                    descriptors,
                    [
                        (48, Family::Spatial),
                        (48, Family::Spatial),
                        (128, Family::Composite),
                        (256, Family::Gaussian)
                    ]
                );
                assert_eq!(prepared.pass_buffer_bytes, PARAMETERS);
                let submitted = observed
                    .iter()
                    .find(|s| s.stage == Stage::Submitted)
                    .unwrap();
                assert_eq!(submitted.pass_buffer_bytes, PARAMETERS);
                assert_eq!(reader().effects_gpu_bytes, EFFECTS);
                gate.release();
                complete(future.as_mut(), &device).unwrap();
            }
            drop((probe, gate));
            let final_snapshot = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
            assert_eq!(
                final_snapshot
                    .scope_usage(Scope::PassBuffers)
                    .unwrap()
                    .peak(RenderResource::GpuBytes),
                PARAMETERS
            );
            assert_eq!(
                final_snapshot
                    .scope_usage(Scope::PassBuffers)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                0
            );
            assert_eq!((reader().active_frame_count, reader().leased_count), (0, 0));
            if warm {
                assert_eq!(
                    reader().payload_creation_attempts,
                    opening.payload_creation_attempts
                );
            }
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for (x, value) in [
                (0, [0; 4]),
                (1, [0, 0, 0, 255]),
                (2, [0, 0, 0, 255]),
                (3, [0; 4]),
            ] {
                assert_pixel(&pixels, x, 0, value, 1);
            }
        }
    }
}

#[test]
fn graph_parameter_uploads_happen_once_in_gpu_prepare_across_profiles() {
    use crate::{RenderPhase, RenderPhaseFilter, RenderWork};
    let scene = constant_scene();
    let mut totals = Vec::new();
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 256,
            filter: RenderPhaseFilter::All,
        },
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        render(&mut renderer, &mut surface, &scene);
        let parameter_allocations = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .generation_for_test()
            .class_usage(crate::accounting::GenerationClass::PassBuffers)
            .unwrap()
            .work(RenderWork::ResourceAllocations);
        let attempt = renderer.last_attempt().unwrap();
        let report = attempt.attempt().unwrap();
        assert_eq!(
            report.usage().work(RenderWork::UploadBytes),
            256 + 48 + 128 + 48
        );
        if profile != RenderProfile::Off {
            assert_eq!(
                report
                    .phase_usage(RenderPhase::GpuPrepare)
                    .work(RenderWork::UploadBytes),
                480
            );
            assert_eq!(
                report
                    .phase_usage(RenderPhase::Encode)
                    .work(RenderWork::UploadBytes),
                0
            );
        }
        // Generation ResourceAllocations counts actual byte-backed allocation,
        // unlike attempt ResourceAllocationRequests which also includes native
        // views, shaders, bind groups and command buffers. The parameter leaf
        // owns exactly Gaussian256 + spatial48 + composite128 + Present48.
        assert_eq!(parameter_allocations, 4);
        totals.push(
            [
                RenderWork::UploadBytes,
                RenderWork::SerializedBytes,
                RenderWork::ScheduledGaussianSamples,
                RenderWork::ImagePasses,
                RenderWork::CompositePasses,
            ]
            .map(|w| report.usage().work(w)),
        );
    }
    assert_eq!(totals[0], totals[1]);
    assert_eq!(totals[1], totals[2]);
}

#[test]
fn native_generation_identity_clones_share_metadata_without_owning_buffer_backing() {
    use crate::accounting::GenerationClass;
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let _surface = pollster::block_on(renderer.create_headless(Size::new(1.0, 1.0), 1.0)).unwrap();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let generation = ready.generation_for_test().clone();
    let same = generation.clone();
    assert!(generation.same_owner(&same));
    let other = crate::accounting::Generation::new(RenderAccountingOptions::new()).unwrap();
    assert!(!generation.same_owner(&other));
    let opening = generation
        .class_usage(GenerationClass::PassBuffers)
        .unwrap()
        .live(RenderResource::GpuBytes);
    let buffer = generation
        .allocation(GenerationClass::PassBuffers, 256, None, || {
            ready
                .device_for_test()
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some("native generation identity lifetime control"),
                    size: 256,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
        })
        .unwrap();
    assert_eq!(buffer.size(), 256);
    assert_eq!(
        same.class_usage(GenerationClass::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        opening + 256
    );
    drop(buffer);
    assert_eq!(
        same.class_usage(GenerationClass::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        opening
    );
}

fn assert_terminal_parameter_disposal(observed: &[crate::pass::ParameterSnapshotForTest]) {
    let prepared = observed
        .iter()
        .find(|s| s.stage == Stage::Prepared)
        .unwrap();
    let mut expected = prepared
        .buffers
        .as_ref()
        .unwrap()
        .iter()
        .map(|b| b.native_identity)
        .collect::<Vec<_>>();
    expected.sort_unstable();
    let mut disposed = Vec::new();
    for snapshot in observed.iter().filter(|s| s.stage == Stage::Disposing) {
        let buffer = snapshot.disposing.as_ref().unwrap();
        assert!(buffer.consumed);
        assert!(snapshot.pass_buffer_bytes >= buffer.bytes);
        assert_eq!(
            (
                snapshot.command_encoder_objects,
                snapshot.command_buffer_objects
            ),
            (0, 0)
        );
        disposed.push(buffer.native_identity);
    }
    disposed.sort_unstable();
    assert_eq!(disposed, expected);
}

#[test]
fn all_graph_parameters_survive_pre_and_post_submit_cancellation_until_commands_dispose() {
    use crate::gpu_transaction::test_support::GraphBeforeSubmissionGateGuardForTest;
    for after_submit in [false, true] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(
                    RenderAccountingOptions::new()
                        .with_profile(RenderProfile::Summary { timing: false }),
                ),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let publications = surface.headless_publication_count_for_test();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let device = ready.device_for_test().clone();
        let reader = ready.resource_manager_observation_reader_for_test();
        let probe = ParameterProbeForTest::new(ready.generation_for_test());
        let observe = probe.reader();
        let signal = renderer.default_device_signal_for_test().unwrap();
        let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let submitted = GraphSubmissionGateGuardForTest::install(signal);
        let scene = constant_scene();
        {
            let mut future = Box::pin(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                WorkingFormat::HighPrecision,
            ));
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
                if before.reached_generation().is_some() {
                    break;
                }
                assert!(Instant::now() < deadline);
                device.poll(wgpu::PollType::Poll).unwrap();
            }
            if after_submit {
                before.release();
                held(future.as_mut(), &submitted, &device);
            }
            let snapshots = observe();
            let stage = if after_submit {
                Stage::Submitted
            } else {
                Stage::BeforeSubmit
            };
            assert_eq!(
                snapshots
                    .iter()
                    .find(|s| s.stage == stage)
                    .unwrap()
                    .pass_buffer_bytes,
                480
            );
            drop(future);
        }
        assert_terminal_parameter_disposal(&observe());
        drop((probe, before, submitted));
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            pixels
        );
        assert_eq!((reader().active_frame_count, reader().leased_count), (0, 0));
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .scope_usage(Scope::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0
        );
        render(&mut renderer, &mut surface, &scene);
        assert_pixel(
            &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            1,
            0,
            [0, 0, 0, 255],
            1,
        );
    }
}

#[test]
fn graph_parameter_backing_survives_scoped_gpu_failure_without_publishing_and_retries() {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let publications = surface.headless_publication_count_for_test();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let probe = ParameterProbeForTest::new(ready.generation_for_test());
    let observe = probe.reader();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let gate = GraphSubmissionGateGuardForTest::install(signal);
    let scene = constant_scene();
    let error = {
        let mut future = Box::pin(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ));
        held(future.as_mut(), &gate, &device);
        assert_eq!(
            observe()
                .iter()
                .find(|s| s.stage == Stage::Submitted)
                .unwrap()
                .pass_buffer_bytes,
            480
        );
        let _invalid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("graph parameter scoped validation control"),
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
        gate.release();
        complete(future.as_mut(), &device).unwrap_err()
    };
    assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
    assert_terminal_parameter_disposal(&observe());
    drop((gate, probe));
    assert_eq!(renderer.stats(), stats);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        pixels
    );
    assert_eq!(
        renderer
            .device_accounting_snapshot(Some(&surface))
            .unwrap()
            .scope_usage(Scope::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
    render(&mut renderer, &mut surface, &scene);
    assert_pixel(
        &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        1,
        0,
        [0, 0, 0, 255],
        1,
    );
}

#[test]
fn partial_parameter_upload_refusals_dispose_only_the_completed_ready_buffers() {
    use crate::{RenderLimits, RenderRefusalReason, RenderWork};
    // This constant route has no source upload. The ordered pass requests first
    // acquire Gaussian256, then the Layer's spatial48/composite128, then Present48.
    // Caps end after the listed real descriptors. The first cap permits the
    // direct baseline's four-byte initialization but refuses Gaussian256.
    for (limit, prefix, next, completed) in [
        (255, 0, 256, vec![]),
        (256, 256, 48, vec![(Family::Gaussian, 256)]),
        (
            304,
            304,
            128,
            vec![(Family::Gaussian, 256), (Family::Spatial, 48)],
        ),
        (
            432,
            432,
            48,
            vec![
                (Family::Gaussian, 256),
                (Family::Spatial, 48),
                (Family::Composite, 128),
            ],
        ),
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(
                    RenderAccountingOptions::new()
                        .with_profile(RenderProfile::Summary { timing: false })
                        .with_limits(
                            RenderLimits::new()
                                .with_work_limit(RenderWork::UploadBytes, limit)
                                .unwrap(),
                        ),
                ),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
        let old = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let publications = surface.headless_publication_count_for_test();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let probe = ParameterProbeForTest::new(ready.generation_for_test());
        let observe = probe.reader();
        let resources = ready.resource_manager_observation_reader_for_test();
        let error = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &constant_scene(),
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ))
        .unwrap_err();
        let attempt = renderer.last_attempt().unwrap();
        let refusal = attempt.attempt().unwrap().refusal().unwrap();
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::UploadBytes)
        );
        assert_eq!((refusal.current(), refusal.amount()), (prefix, next));
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        let snapshots = observe();
        assert!(snapshots.iter().all(|s| s.stage == Stage::Disposing));
        let disposed = snapshots
            .iter()
            .map(|s| {
                assert_eq!(
                    (s.command_encoder_objects, s.command_buffer_objects),
                    (0, 0)
                );
                let buffer = s.disposing.as_ref().unwrap();
                assert!(!buffer.consumed);
                assert!(s.pass_buffer_bytes >= buffer.bytes);
                (buffer.family, buffer.bytes)
            })
            .collect::<Vec<_>>();
        assert_eq!(disposed.len(), completed.len());
        for descriptor in completed {
            assert!(disposed.contains(&descriptor));
        }
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .scope_usage(Scope::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0
        );
        assert_eq!(
            (resources().active_frame_count, resources().leased_count),
            (0, 0)
        );
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            old
        );
    }
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    render(&mut renderer, &mut surface, &constant_scene());
    assert_pixel(
        &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        1,
        0,
        [0, 0, 0, 255],
        1,
    );
}

#[cfg(feature = "render-window")]
#[test]
fn presented_graph_keeps_actual_parameters_through_submission_and_accounting_ready() {
    use crate::{
        Format, PhysicalSize, SurfaceOptions,
        backend::{
            display_free_presented_surface_for_test, presented_observation_handle_for_test,
            take_last_presented_texture_for_test,
        },
    };
    for cancel in [false, true] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(
                    RenderAccountingOptions::new()
                        .with_profile(RenderProfile::Summary { timing: false }),
                ),
        ))
        .unwrap();
        let mut surface = display_free_presented_surface_for_test(
            &mut renderer,
            SurfaceOptions {
                size: Size::new(4.0, 1.0),
                format: Format::Rgba8,
                ..SurfaceOptions::default()
            },
        );
        pollster::block_on(renderer.configure_presented_surface_for_test(&mut surface)).unwrap();
        let presentation = presented_observation_handle_for_test(&surface);
        let stats = renderer.stats();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let device = ready.device_for_test().clone();
        let resources = ready.resource_manager_observation_reader_for_test();
        let probe = ParameterProbeForTest::new(ready.generation_for_test());
        let observe = probe.reader();
        let gate = GraphSubmissionGateGuardForTest::install(
            renderer.default_device_signal_for_test().unwrap(),
        );
        let scene = constant_scene();
        {
            let future = renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                WorkingFormat::HighPrecision,
            );
            let mut future = Box::pin(future);
            held(future.as_mut(), &gate, &device);
            let snapshots = observe();
            let prepared = snapshots
                .iter()
                .find(|s| s.stage == Stage::Prepared)
                .unwrap();
            let submitted = snapshots
                .iter()
                .find(|s| s.stage == Stage::Submitted)
                .unwrap();
            assert_eq!(prepared.pass_buffer_bytes, 480);
            assert_eq!(submitted.pass_buffer_bytes, 480);
            assert_eq!(presentation.snapshot_for_test().present_count_for_test(), 0);
            if !cancel {
                gate.release();
                complete(future.as_mut(), &device).unwrap();
            }
        }
        let snapshots = observe();
        assert_terminal_parameter_disposal(&snapshots);
        if cancel {
            assert!(snapshots.iter().all(|s| s.stage != Stage::AccountingReady));
            assert_eq!(renderer.stats(), stats);
            assert_eq!(presentation.snapshot_for_test().present_count_for_test(), 0);
            assert_eq!(
                presentation.snapshot_for_test().discarded_count_for_test(),
                1
            );
        } else {
            let ready = snapshots
                .iter()
                .find(|s| s.stage == Stage::AccountingReady)
                .unwrap();
            assert_eq!(ready.pass_buffer_bytes, 480);
            let expected = snapshots
                .iter()
                .find(|s| s.stage == Stage::Prepared)
                .unwrap();
            assert_eq!(
                ready
                    .buffers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|b| b.native_identity)
                    .collect::<Vec<_>>(),
                expected
                    .buffers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|b| b.native_identity)
                    .collect::<Vec<_>>()
            );
            assert_eq!(presentation.snapshot_for_test().present_count_for_test(), 1);
            let texture = take_last_presented_texture_for_test(&mut surface).unwrap();
            let pixels = pollster::block_on(
                renderer.read_render_texture_for_test(&texture, PhysicalSize::new(4, 1)),
            )
            .unwrap();
            for (x, pixel) in [
                (0, [0; 4]),
                (1, [0, 0, 0, 255]),
                (2, [0, 0, 0, 255]),
                (3, [0; 4]),
            ] {
                assert_pixel(&pixels, x, 0, pixel, 1);
            }
        }
        assert_eq!(
            (resources().active_frame_count, resources().leased_count),
            (0, 0)
        );
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .scope_usage(Scope::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0
        );
        drop((probe, gate));
        // The same public input remains usable after either terminal path.
        render(&mut renderer, &mut surface, &scene);
        assert_eq!(
            presentation.snapshot_for_test().present_count_for_test(),
            if cancel { 1 } else { 2 }
        );
    }
}
