//! Public native shadow accounting and failure atomicity through real graph
//! execution. Post-submit gates leave production submission and custody intact.
#![cfg(not(target_arch = "wasm32"))]

use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::native_shadow::{finite_scene, operation, render, sigma, source_scene};
use crate::{
    Color, DropShadowInput, DropShadowSource, ErrorCode, FilterColorSpace, FilterDomains,
    GpuFaultKind, Layer, Options, Parameters, RenderAccountingOptions, RenderAttemptStatus,
    RenderLimits, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderResource, RenderRoute,
    RenderWork, Renderer, ResourceCacheBudget, RuntimeCapabilityUnavailableReason,
    RuntimeOperation, Scene, Size, Surface,
    gpu_transaction::test_support::GraphSubmissionGateGuardForTest,
    surface::{HeadlessResources, SurfaceBackend},
};
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

fn scenes() -> (Scene, Scene) {
    let domain = bounds(2, 1);
    let effect = operation(
        DropShadowInput::new()
            .with_dx(0.0)
            .with_dy(0.0)
            .with_standard_deviations(4.0, 1.0),
        FilterColorSpace::LinearSrgb,
    );
    let domains = FilterDomains::try_new(domain, domain, domain).unwrap();
    let build = |rgba: Vec<u8>| {
        finite_scene(
            source_scene(image(2, 1, rgba), domain),
            domains,
            effect,
            crate::Layer::new(),
        )
    };
    (
        build(vec![255, 0, 0, 255, 255, 0, 0, 255]),
        build(vec![0, 0, 255, 255, 0, 0, 255, 255]),
    )
}

fn setup() -> (
    Renderer,
    Surface,
    Scene,
    wgpu::Device,
    wgpu::Queue,
    wgpu::Texture,
) {
    let (old, next) = scenes();
    // Zero idle retention makes cleanup custody observable without introducing
    // an unrelated cache-eviction policy into this transaction assertion.
    let options = Options::default().with_resource_cache_budget(ResourceCacheBudget::DISABLED);
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
    let output = render(&mut renderer, &mut surface, &old);
    assert_eq!(output.rgba(), &[255, 0, 0, 255, 255, 0, 0, 255]);
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let queue = ready.queue_for_test().clone();
    let texture = published(&surface);
    (renderer, surface, next, device, queue, texture)
}

fn published(surface: &Surface) -> wgpu::Texture {
    match &surface.backend {
        SurfaceBackend::Headless {
            resources: HeadlessResources::Ready { texture },
            ..
        } => texture.deref().clone(),
        _ => panic!("a completed public native shadow frame must retain its publication"),
    }
}

fn reach_submission<F: Future>(
    mut future: Pin<&mut F>,
    gate: &GraphSubmissionGateGuardForTest,
    device: &wgpu::Device,
) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending(),
            "public render must remain pending at its real post-submit gate"
        );
        if let Some(generation) = gate.submitted_generation() {
            assert_eq!(gate.submissions(), 1);
            return generation;
        }
        assert!(
            Instant::now() < deadline,
            "public native shadow did not reach real graph submission"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

fn finish<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device) -> F::Output {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Poll::Ready(result) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return result;
        }
        assert!(
            Instant::now() < deadline,
            "public native shadow transaction did not resolve before its diagnostic deadline"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

fn assert_ready_custody(renderer: &mut Renderer, surface: &Surface) {
    assert_eq!(
        renderer.default_device_active_operation_generation_for_test(),
        None
    );
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let resources = ready.internal_resource_manager_observation_for_test();
    assert_eq!(resources.leased_count, 0);
    assert_eq!(resources.active_frame_count, 0);
    assert_eq!(resources.resolved_lease_count, 0);
    assert_eq!(resources.accounting_fault_for_test(), None);
    assert_eq!(resources.retained_bytes, 0);
    let snapshot = renderer.device_accounting_snapshot(Some(surface)).unwrap();
    assert!(!snapshot.retired());
    assert_eq!(snapshot.fault(), None);
    assert_eq!(
        snapshot
            .usage()
            .live(RenderResource::RegisteredCacheGpuBytes),
        0
    );
    assert_eq!(snapshot.usage().live(RenderResource::IdleCacheGpuBytes), 0);
}

fn retry(renderer: &mut Renderer, surface: &mut Surface, next: &Scene) {
    let output = render(renderer, surface, next);
    assert_eq!(output.rgba(), &[0, 0, 255, 255, 0, 0, 255, 255]);
    assert_eq!(renderer.stats().route, Some(RenderRoute::GpuGraph));
    assert_ready_custody(renderer, surface);
}

#[test]
fn public_native_shadow_post_submit_cancellation_preserves_pixels_stats_and_custody_then_retries() {
    let (mut renderer, mut surface, next, device, _queue, old_texture) = setup();
    let stats = renderer.stats();
    let publications = surface.headless_publication_count_for_test();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let gate = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
    {
        let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
        let generation = reach_submission(future.as_mut(), &gate, &device);
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        drop(future);
    }
    drop(gate);
    assert_eq!(signal.active_generation_for_test(), None);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(published(&surface), old_texture);
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        renderer.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Canceled
    );
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(output.rgba(), &[255, 0, 0, 255, 255, 0, 0, 255]);
    assert_ready_custody(&mut renderer, &surface);
    retry(&mut renderer, &mut surface, &next);
}

#[test]
fn public_native_shadow_scoped_validation_failure_preserves_publication_and_can_retry() {
    let (mut renderer, mut surface, next, device, _queue, old_texture) = setup();
    let stats = renderer.stats();
    let publications = surface.headless_publication_count_for_test();
    let cache = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .device_pass_cache_counts_for_test();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let gate = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let error = {
        let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
        let generation = reach_submission(future.as_mut(), &gate, &device);
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        // Inject a real WGPU validation error into the render owner's still-open
        // scope, after its real native shadow command buffer was submitted.
        let _invalid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surgeist public native shadow scoped failure fixture"),
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
        finish(future.as_mut(), &device)
            .expect_err("the actual render scope must report its injected validation error")
    };
    drop(gate);
    assert_eq!(error.code(), ErrorCode::RenderFailed);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(published(&surface), old_texture);
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .device_pass_cache_counts_for_test(),
        cache
    );
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(output.rgba(), &[255, 0, 0, 255, 255, 0, 0, 255]);
    assert_ready_custody(&mut renderer, &surface);
    retry(&mut renderer, &mut surface, &next);
}

#[test]
fn public_native_shadow_first_terminal_fault_aborts_publication_and_retires_once() {
    let (mut renderer, mut surface, next, device, queue, old_texture) = setup();
    let stats = renderer.stats();
    let publications = surface.headless_publication_count_for_test();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let gate = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let (error, submitted_generation) = {
        let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
        let generation = reach_submission(future.as_mut(), &gate, &device);
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        signal.record_uncaptured_fault_for_test(
            GpuFaultKind::Validation,
            "first native shadow terminal fault",
        );
        signal.record_uncaptured_fault_for_test(
            GpuFaultKind::Internal,
            "later native shadow terminal fault",
        );
        gate.release();
        let error = finish(future.as_mut(), &device)
            .expect_err("terminal signal must reject the real submitted frame");
        (error, generation)
    };
    drop(gate);
    let first = signal.first_terminal().unwrap();
    assert_eq!(signal.active_generation_for_test(), None);
    // An uncaptured fault attributed to this active generation is classified
    // by its Render stage. Later operations receive the terminal capability
    // diagnostic instead (GpuOperationTransaction::terminal_result).
    assert_eq!(
        error.code(),
        ErrorCode::RenderFailed,
        "active-generation fault: {error:?}"
    );
    assert_eq!(
        first.operation_generation_for_test(),
        Some(submitted_generation)
    );
    assert_eq!(
        first
            .error(crate::RuntimeOperation::SurfaceRendering)
            .runtime_capability_unavailable_diagnostic()
            .expect("first signal retains its terminal reason")
            .reason(),
        RuntimeCapabilityUnavailableReason::DeviceFaulted {
            kind: GpuFaultKind::Validation
        }
    );
    assert!(
        error
            .to_string()
            .contains("first native shadow terminal fault")
    );
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(published(&surface), old_texture);
    assert_eq!(renderer.stats(), stats);
    assert!(renderer.default_device_renderer_released_for_test());
    assert!(
        renderer
            .default_ready_device_state_borrow_for_test()
            .is_none()
    );
    let retired = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
    assert!(retired.retired());
    assert_eq!(retired.fault(), None);
    for resource in [
        RenderResource::GpuBytes,
        RenderResource::OpaqueGpuObjects,
        RenderResource::RegisteredCacheGpuBytes,
        RenderResource::IdleCacheGpuBytes,
    ] {
        assert_eq!(
            retired.usage().live(resource),
            0,
            "retired {resource:?} custody"
        );
    }
    // Public readback correctly refuses the terminal device. Inspect the same
    // native published texture independently to prove its old pixels survived;
    // this observation does not execute or replace the native shadow receiver.
    assert!(pollster::block_on(renderer.read_headless(&surface)).is_err());
    assert_eq!(
        native_publication_pixels(&device, &queue, &old_texture),
        [255, 0, 0, 255, 255, 0, 0, 255]
    );
    let retry_error =
        pollster::block_on(renderer.render(&mut surface, &next, Parameters::default()))
            .unwrap_err();
    assert_eq!(
        retry_error
            .runtime_capability_unavailable_diagnostic()
            .unwrap()
            .reason(),
        RuntimeCapabilityUnavailableReason::DeviceFaulted {
            kind: GpuFaultKind::Validation
        }
    );
    assert!(Arc::ptr_eq(&first, &signal.first_terminal().unwrap()));
    let after_retry = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
    assert_eq!(after_retry.identity(), retired.identity());
    assert_eq!(after_retry.usage(), retired.usage());
    assert_eq!(renderer.stats(), stats);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
}

fn native_publication_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> [u8; 8] {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Surgeist retained publication observation"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: 2,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let submission = queue.submit([encoder.finish()]);
    let slice = staging.slice(..);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    let result = {
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(5)),
            })
            .unwrap();
        receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        let mapped = slice.get_mapped_range();
        <[u8; 8]>::try_from(&mapped[..8]).unwrap()
    };
    staging.unmap();
    result
}

fn sample_fixture(x: f64, y: f64, flood_opacity: f64) -> Scene {
    let domain = bounds(3, 1);
    finite_scene(
        source_scene(image(3, 1, [64, 128, 192, 255].repeat(3)), domain),
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        operation(
            DropShadowInput::new()
                .with_dx(0.0)
                .with_dy(0.0)
                .with_standard_deviations(x, y)
                .with_flood_opacity(flood_opacity),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    )
}

fn profiles() -> [RenderProfile; 3] {
    [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 256,
            filter: RenderPhaseFilter::All,
        },
    ]
}

#[test]
fn public_native_shadow_profiles_charge_actual_alpha_offset_and_merge_samples() {
    // Every final image has3 cells. The finite aligned merge reads its original
    // and shifted alpha once per cell; offset has one aligned fetch per cell.
    // Gaussian has3 identity copies, or its existing preparation/resolve plus
    // 9-tap axis work. Two axes add the required horizontal halo; sigma4 uses
    // 33 coefficients in two batches. Offset and merge add6 filter cells to
    // the independently derived Gaussian totals3/9/36/12.
    for (x, y, opacity, expected) in [
        (0.0, 0.0, 1.0, [3, 3, 6, 9]),
        (sigma(), 0.0, 1.0, [33, 3, 6, 15]),
        (sigma(), sigma(), 1.0, [276, 3, 6, 42]),
        (4.0, 0.0, 1.0, [108, 3, 6, 18]),
        // Transparent flood omits Gaussian and offset entirely, including the
        // coefficient preparation. Only3 original reads/merge cells remain.
        (4.0, sigma(), 0.0, [0, 0, 3, 3]),
    ] {
        let mut observations = Vec::new();
        for profile in profiles() {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
            let pixels = render(&mut renderer, &mut surface, &sample_fixture(x, y, opacity));
            // Opaque original is preserved above every shadow, independent of
            // the edge-alpha convolution and the profiling mode.
            for cell in 0..3 {
                assert_pixel(&pixels, cell, 0, [64, 128, 192, 255], 2);
            }
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            let counts = [
                RenderWork::ScheduledGaussianSamples,
                RenderWork::ScheduledOffsetSamples,
                RenderWork::ScheduledShadowSamples,
                RenderWork::ScheduledFilterTexels,
                RenderWork::KernelSampleVisits,
                RenderWork::SerializedBytes,
                RenderWork::UploadBytes,
            ]
            .map(|work| usage.work(work));
            assert_eq!(counts[..4], expected, "sigma=[{x},{y}], flood={opacity}");
            if opacity == 0.0 || (x == 0.0 && y == 0.0) {
                assert_eq!(counts[4], 0);
            }
            observations.push((pixels, counts));
        }
        assert_eq!(observations[0], observations[1]);
        assert_eq!(observations[1], observations[2]);
    }
}

#[test]
fn public_native_shadow_exact_and_one_under_sample_budgets_preserve_publication() {
    let scene = sample_fixture(sigma(), sigma(), 1.0);
    for profile in profiles() {
        for cache in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
            for (work, exact) in [
                (RenderWork::ScheduledGaussianSamples, 276),
                (RenderWork::ScheduledOffsetSamples, 3),
                (RenderWork::ScheduledShadowSamples, 6),
                (RenderWork::ScheduledFilterTexels, 42),
            ] {
                for cap in [exact - 1, exact] {
                    let accounting = RenderAccountingOptions::new()
                        .with_profile(profile)
                        .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap());
                    let mut renderer = pollster::block_on(Renderer::new(
                        Options::default()
                            .with_resource_cache_budget(cache)
                            .with_accounting(accounting),
                    ))
                    .unwrap();
                    let mut surface =
                        pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0))
                            .unwrap();
                    let mut old_scene = Scene::new();
                    old_scene.fill(bounds(3, 1), Color::BLACK);
                    let stats = pollster::block_on(renderer.render(
                        &mut surface,
                        &old_scene,
                        Parameters::default(),
                    ))
                    .unwrap();
                    let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                    let old_texture = published(&surface);
                    let publications = surface.headless_publication_count_for_test();
                    let outcome = pollster::block_on(renderer.render(
                        &mut surface,
                        &scene,
                        Parameters::default(),
                    ));
                    if cap < exact {
                        let error = outcome.unwrap_err();
                        let report = renderer.last_attempt().unwrap();
                        assert_eq!(report.status(), RenderAttemptStatus::Refused);
                        assert_eq!(report.cleanup_fault(), None);
                        let attempt = report.attempt().unwrap();
                        assert_eq!(
                            attempt.refusal().unwrap().quantity(),
                            RenderQuantity::Work(work)
                        );
                        assert!(attempt.usage().work(work) <= cap);
                        assert!(error.accounting_diagnostic().is_some());
                        assert_eq!(renderer.stats(), stats);
                        assert_eq!(surface.headless_publication_count_for_test(), publications);
                        assert_eq!(published(&surface), old_texture);
                        assert_eq!(
                            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                            old_pixels
                        );
                    } else {
                        outcome.unwrap();
                        assert_eq!(
                            renderer
                                .last_attempt()
                                .unwrap()
                                .attempt()
                                .unwrap()
                                .usage()
                                .work(work),
                            exact
                        );
                        assert_eq!(
                            surface.headless_publication_count_for_test(),
                            publications + 1
                        );
                        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                        assert_pixel(&pixels, 1, 0, [64, 128, 192, 255], 2);
                    }
                    let resources = renderer
                        .default_ready_device_state_borrow_for_test()
                        .unwrap()
                        .internal_resource_manager_observation_for_test();
                    assert_eq!(resources.leased_count, 0);
                    assert_eq!(resources.active_frame_count, 0);
                    assert_eq!(resources.resolved_lease_count, 0);
                    assert_eq!(resources.accounting_fault_for_test(), None);
                }
            }
        }
    }
}

#[test]
fn public_native_shadow_float32_refusal_is_typed_and_bypassed_only_by_unused_alpha_kernel() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let domain = bounds(3, 1);
    let supports_float32 = renderer
        .default_device_capabilities_for_test()
        .effect_precisions()
        .supports_filter_float32();
    assert!(supports_float32);
    assert!(renderer.override_default_device_filter_float32_fact_for_test(false));
    assert!(
        !renderer
            .default_device_capabilities_for_test()
            .effect_precisions()
            .supports_filter_float32()
    );
    // Constant alpha is analytic even for a huge admitted sigma. This does not
    // justify skipping the finite F resolve, displacement or original merge.
    let mut constant = Scene::new();
    constant
        .drop_shadow_layer(
            Layer::new(),
            operation(
                DropShadowInput::new()
                    .with_dx(0.0)
                    .with_dy(0.0)
                    .with_standard_deviations(1e300, 1e300),
                FilterColorSpace::Srgb,
            ),
            DropShadowSource::constant(Color::BLACK, domain, domain).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &constant);
    assert_pixel(&pixels, 1, 0, [0, 0, 0, 255], 1);
    let before = render(&mut renderer, &mut surface, &sample_fixture(0.0, 0.0, 1.0));
    // A real finite source with transparent flood reaches only original merge;
    // its otherwise unrepresentable kernel is neither prepared nor admitted.
    let transparent = render(
        &mut renderer,
        &mut surface,
        &sample_fixture(1e300, 1e300, 0.0),
    );
    assert_eq!(transparent, before);
    let stats = renderer.stats();
    let old_texture = published(&surface);
    let publications = surface.headless_publication_count_for_test();
    let active = sample_fixture(sigma(), 0.0, 1.0);
    let error = pollster::block_on(renderer.render(&mut surface, &active, Parameters::default()))
        .unwrap_err();
    let diagnostic = error.runtime_capability_unavailable_diagnostic().unwrap();
    assert_eq!(
        diagnostic.operation(),
        RuntimeOperation::NativeDropShadowRendering
    );
    assert_eq!(
        diagnostic.reason(),
        RuntimeCapabilityUnavailableReason::FilterFloat32Unavailable
    );
    assert_eq!(renderer.stats(), stats);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(published(&surface), old_texture);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    let resources = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert_eq!(resources.leased_count, 0);
    assert_eq!(resources.active_frame_count, 0);
    assert_eq!(resources.accounting_fault_for_test(), None);
    assert!(renderer.override_default_device_filter_float32_fact_for_test(supports_float32));
    assert_eq!(render(&mut renderer, &mut surface, &active), before);
}

#[test]
fn public_native_shadow_actual_uniform_upload_allocation_and_serialization_caps_refuse_cleanly() {
    let scene = sample_fixture(4.0, 1.0, 1.0);
    // These are calibrated actual backing/upload totals, not independent sample
    // oracles. Use the same fresh-device history for measurement and each cap so
    // shader/resource cache reuse cannot silently change the admission contract.
    for work in [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
        render(&mut renderer, &mut surface, &scene);
        let measured = renderer
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(work);
        assert!(measured > 0, "native shadow must reach actual {work:?}");
        drop((renderer, surface));
        for cap in [measured - 1, measured] {
            let limits = RenderLimits::new().with_work_limit(work, cap).unwrap();
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_limits(limits)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(3.0, 1.0), 1.0)).unwrap();
            let stats = pollster::block_on(renderer.render(
                &mut surface,
                &Scene::new(),
                Parameters::default(),
            ))
            .unwrap();
            let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let old_texture = published(&surface);
            let publications = surface.headless_publication_count_for_test();
            let outcome =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()));
            if cap < measured {
                let error = outcome.unwrap_err();
                assert!(error.accounting_diagnostic().is_some());
                let report = renderer.last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(
                    report.attempt().unwrap().refusal().unwrap().quantity(),
                    RenderQuantity::Work(work)
                );
                assert!(report.attempt().unwrap().usage().work(work) <= cap);
                assert_eq!(report.cleanup_fault(), None);
                assert_eq!(renderer.stats(), stats);
                assert_eq!(surface.headless_publication_count_for_test(), publications);
                assert_eq!(published(&surface), old_texture);
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    before
                );
            } else {
                outcome.unwrap();
                assert_eq!(
                    renderer
                        .last_attempt()
                        .unwrap()
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(work),
                    measured
                );
                let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                assert_pixel(&pixels, 1, 0, [64, 128, 192, 255], 2);
            }
            let resources = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap()
                .internal_resource_manager_observation_for_test();
            assert_eq!(resources.leased_count, 0);
            assert_eq!(resources.active_frame_count, 0);
            assert_eq!(resources.resolved_lease_count, 0);
            assert_eq!(resources.accounting_fault_for_test(), None);
        }
    }
}
