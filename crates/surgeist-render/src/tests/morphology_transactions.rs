//! Public native morphology accounting and failure atomicity through real graph
//! execution. Post-submit gates leave production submission and custody intact.
#![cfg(not(target_arch = "wasm32"))]

use super::convolution::{assert_pixel, bounds, image, renderer_and_surface};
use super::morphology::{finite_scene, operation, render, source_scene};
use crate::{
    Color, ErrorCode, FilterColorSpace, FilterDomains, GpuFaultKind, Layer, MorphologyInput,
    MorphologyOperator, MorphologySource, Options, Parameters, RenderAccountingOptions,
    RenderAttemptStatus, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile,
    RenderQuantity, RenderResource, RenderRoute, RenderWork, Renderer, ResourceCacheBudget,
    RuntimeCapabilityUnavailableReason, RuntimeOperation, Scene, Size, Surface,
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
    let domain = bounds(5, 1);
    let effect = operation(
        MorphologyInput::new()
            .with_operator(MorphologyOperator::Dilate)
            .with_radii(2.0, 0.25),
        FilterColorSpace::LinearSrgb,
    );
    let domains = FilterDomains::try_new(domain, domain, domain).unwrap();
    let build = |rgba: Vec<u8>| {
        finite_scene(
            source_scene(image(5, 1, rgba), domain),
            domains,
            effect,
            crate::Layer::new(),
        )
    };
    (
        build([255, 0, 0, 255].repeat(5)),
        build([0, 0, 255, 255].repeat(5)),
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
        pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 1.0)).unwrap();
    let output = render(&mut renderer, &mut surface, &old);
    assert_eq!(output.rgba(), [255, 0, 0, 255].repeat(5));
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
        _ => panic!("a completed public morphology frame must retain its publication"),
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
            "public morphology did not reach real graph submission"
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
            "public morphology transaction did not resolve before its diagnostic deadline"
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
    assert_eq!(output.rgba(), [0, 0, 255, 255].repeat(5));
    assert_eq!(renderer.stats().route, Some(RenderRoute::GpuGraph));
    assert_ready_custody(renderer, surface);
}

#[test]
fn public_morphology_post_submit_cancellation_preserves_pixels_stats_and_custody_then_retries() {
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
    assert_eq!(output.rgba(), [255, 0, 0, 255].repeat(5));
    assert_ready_custody(&mut renderer, &surface);
    retry(&mut renderer, &mut surface, &next);
}

#[test]
fn public_morphology_scoped_validation_failure_preserves_publication_and_can_retry() {
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
        // scope, after its real morphology command buffer was submitted.
        let _invalid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surgeist public morphology scoped failure fixture"),
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
    assert_eq!(output.rgba(), [255, 0, 0, 255].repeat(5));
    assert_ready_custody(&mut renderer, &surface);
    retry(&mut renderer, &mut surface, &next);
}

#[test]
fn public_morphology_first_terminal_fault_aborts_publication_and_retires_once() {
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
            "first morphology terminal fault",
        );
        signal.record_uncaptured_fault_for_test(
            GpuFaultKind::Internal,
            "later morphology terminal fault",
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
            .contains("first morphology terminal fault")
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
    // this observation does not execute or replace the morphology receiver.
    assert!(pollster::block_on(renderer.read_headless(&surface)).is_err());
    assert_eq!(
        native_publication_pixels(&device, &queue, &old_texture).as_slice(),
        [255, 0, 0, 255].repeat(5).as_slice()
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
) -> [u8; 20] {
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
            width: 5,
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
        <[u8; 20]>::try_from(&mapped[..20]).unwrap()
    };
    staging.unmap();
    result
}

fn sample_fixture(x: f64, y: f64) -> Scene {
    let domain = bounds(5, 1);
    finite_scene(
        source_scene(
            image(
                5,
                1,
                [32, 192, 64, 128, 224]
                    .into_iter()
                    .flat_map(|r| [r, 0, 0, 255])
                    .collect(),
            ),
            domain,
        ),
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        operation(
            MorphologyInput::new()
                .with_operator(MorphologyOperator::Dilate)
                .with_radii(x, y),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    )
}

fn assert_sample_pixels(pixels: &crate::ImageBuffer) {
    // Literal R2 maxima of [32,192,64,128,224], with finite exterior zero.
    for (x, red) in [192, 192, 224, 224, 224].into_iter().enumerate() {
        assert_pixel(pixels, x as u32, 0, [red, 0, 0, 255], 2);
    }
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
fn public_morphology_profiles_charge_reached_queries_without_gaussian_work() {
    let full = bounds(5, 1);
    let mut constant = Scene::new();
    constant
        .morphology_layer(
            Layer::new(),
            operation(
                MorphologyInput::new().with_radius(1e300),
                FilterColorSpace::LinearSrgb,
            ),
            MorphologySource::constant(Color::BLACK, full, full).unwrap(),
        )
        .unwrap();
    let zero = finite_scene(
        source_scene(image(5, 1, [255, 0, 0, 255].repeat(5)), full),
        FilterDomains::try_new(full, full, full).unwrap(),
        operation(
            MorphologyInput::new().with_radii(1e300, 0.25),
            FilterColorSpace::LinearSrgb,
        ),
        Layer::new(),
    );
    let mut periodic = Scene::new();
    periodic
        .morphology_layer(
            Layer::new(),
            operation(
                MorphologyInput::new()
                    .with_operator(MorphologyOperator::Dilate)
                    .with_radii(2.0, 0.25),
                FilterColorSpace::Srgb,
            ),
            MorphologySource::periodic(
                source_scene(
                    image(
                        5,
                        1,
                        [32, 192, 64, 128, 224]
                            .into_iter()
                            .flat_map(|r| [r, 0, 0, 255])
                            .collect(),
                    ),
                    full,
                ),
                full,
                full,
                full,
            )
            .unwrap(),
        )
        .unwrap();
    // Variable X lengths3,4,5,4,3 reach levels1 and2. The independently
    // selected schedule is prepare5, First5, table8, refine15, table4,
    // refine15, Y point5, resolve5:62 samples,36 target cells,8 draws.
    // Disabled/grid-point copies have one fetch per cell; constants and the
    // proved finite-exterior erosion have no source-sample opportunity.
    // Periodic tables retain5 cells each; two builds cost20 samples,
    // constant XQuery10, YFirst5, prepare5 and wrapped resolve20.
    // ImagePasses also includes the surface-base clear. Captured finite and
    // periodic inputs additionally require the transparent source-parent clear
    // and capture canonicalization. Thus the active finite total is8+1+1+1.
    // Vello capture, source/final composites and presentation have their own
    // counters. Constant input has no capture; the always-exterior erosion
    // proves its source unreachable before capture, so both add only the root
    // clear to their one analytic morphology draw.
    for (scene, expected, alpha, samples, texels, morphology_draws, captured) in [
        (
            sample_fixture(2.0, 0.25),
            [192, 192, 224, 224, 224],
            255,
            62,
            36,
            8,
            true,
        ),
        (
            sample_fixture(0.0, 2.0),
            [32, 192, 64, 128, 224],
            255,
            5,
            5,
            1,
            true,
        ),
        (
            sample_fixture(0.25, 0.25),
            [32, 192, 64, 128, 224],
            255,
            5,
            5,
            1,
            true,
        ),
        (constant, [0; 5], 255, 0, 5, 1, false),
        (zero, [0; 5], 0, 0, 5, 1, false),
        (periodic, [224; 5], 255, 60, 30, 6, true),
    ] {
        let mut observations = Vec::new();
        for profile in profiles() {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
            ))
            .unwrap();
            let mut surface =
                pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 1.0)).unwrap();
            let pixels = render(&mut renderer, &mut surface, &scene);
            for (x, red) in expected.into_iter().enumerate() {
                assert_pixel(&pixels, x as u32, 0, [red, 0, 0, alpha], 2);
            }
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let attempt = report.attempt().unwrap();
            assert_eq!(attempt.elapsed(), None);
            assert_eq!(attempt.phase_elapsed(RenderPhase::Plan), None);
            // Captured sources execute real Vello dispatches in Kernel. Off
            // omits phase observations; analytic constant/exterior routes have
            // no source raster work. The default Area rasterizer needs no mask
            // LUT samples, and morphology creates no Gaussian coefficients.
            if captured && !matches!(profile, RenderProfile::Off) {
                assert!(attempt.phase_count(RenderPhase::Kernel) > 0);
            } else {
                assert_eq!(attempt.phase_count(RenderPhase::Kernel), 0);
            }
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::Kernel)
                    .work(RenderWork::UploadBytes),
                0
            );
            match profile {
                RenderProfile::Off => {
                    assert_eq!(attempt.phase_count(RenderPhase::Plan), 0);
                    assert_eq!(attempt.events().count(), 0);
                }
                RenderProfile::Summary { .. } => {
                    assert!(attempt.phase_count(RenderPhase::Plan) > 0);
                    assert_eq!(attempt.events().count(), 0);
                }
                RenderProfile::Detailed { event_limit, .. } => {
                    assert!(attempt.phase_count(RenderPhase::Plan) > 0);
                    assert!(attempt.events().count() > 0);
                    assert!(attempt.events().count() <= event_limit);
                }
            }
            let usage = attempt.usage();
            let counts = [
                RenderWork::ScheduledMorphologySamples,
                RenderWork::ScheduledFilterTexels,
                RenderWork::ImagePasses,
                RenderWork::KernelSampleVisits,
                RenderWork::ScheduledGaussianSamples,
                RenderWork::SerializedBytes,
                RenderWork::UploadBytes,
            ]
            .map(|work| usage.work(work));
            let image_passes = morphology_draws + 1 + if captured { 2 } else { 0 };
            assert_eq!(counts[..5], [samples, texels, image_passes, 0, 0]);
            observations.push((pixels, counts));
        }
        assert_eq!(observations[0], observations[1]);
        assert_eq!(observations[1], observations[2]);
    }
}

fn assert_frame_leases_released(renderer: &mut Renderer) {
    let resources = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert_eq!(resources.leased_count, 0);
    assert_eq!(resources.active_frame_count, 0);
    assert_eq!(resources.resolved_lease_count, 0);
    assert_eq!(resources.accounting_fault_for_test(), None);
}

fn budget_options(
    profile: RenderProfile,
    cache: ResourceCacheBudget,
    work: RenderWork,
    cap: u64,
) -> Options {
    Options::default()
        .with_resource_cache_budget(cache)
        .with_accounting(
            RenderAccountingOptions::new()
                .with_profile(profile)
                .with_limits(RenderLimits::new().with_work_limit(work, cap).unwrap()),
        )
}

fn assert_budget_case(scene: &Scene, options: Options, work: RenderWork, cap: u64, exact: u64) {
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 1.0)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let old_texture = published(&surface);
    let publications = surface.headless_publication_count_for_test();
    let outcome = pollster::block_on(renderer.render(&mut surface, scene, Parameters::default()));
    if cap < exact {
        let error = outcome.unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let report = renderer.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.cleanup_fault(), None);
        let attempt = report.attempt().unwrap();
        assert_eq!(
            attempt.refusal().unwrap().quantity(),
            RenderQuantity::Work(work)
        );
        assert!(attempt.usage().work(work) <= cap);
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
            exact
        );
        assert_eq!(
            surface.headless_publication_count_for_test(),
            publications + 1
        );
        assert_sample_pixels(&pollster::block_on(renderer.read_headless(&surface)).unwrap());
    }
    assert_frame_leases_released(&mut renderer);
}

#[test]
fn public_morphology_exact_and_one_under_samples_preserve_publication_then_raised_cap_retries() {
    let scene = sample_fixture(2.0, 0.25);
    for profile in profiles() {
        for cache in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
            for (work, exact) in [
                (RenderWork::ScheduledMorphologySamples, 62),
                (RenderWork::ScheduledFilterTexels, 36),
            ] {
                assert_budget_case(
                    &scene,
                    budget_options(profile, cache, work, exact - 1),
                    work,
                    exact - 1,
                    exact,
                );
                // Options are immutable. Raising a public budget requires a
                // fresh renderer; retry the original Scene/shared Images,
                // with the same initial publication/readback history.
                assert_budget_case(
                    &scene,
                    budget_options(profile, cache, work, exact),
                    work,
                    exact,
                    exact,
                );
            }
        }
    }
}

#[test]
fn public_morphology_actual_upload_serialization_and_allocation_caps_refuse_cleanly() {
    let scene = sample_fixture(2.0, 0.25);
    for work in [
        RenderWork::UploadBytes,
        RenderWork::SerializedBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 1.0)).unwrap();
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
        pollster::block_on(renderer.read_headless(&surface)).unwrap();
        render(&mut renderer, &mut surface, &scene);
        let measured = renderer
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(work);
        assert!(measured > 0, "morphology must reach actual {work:?}");
        assert_frame_leases_released(&mut renderer);
        drop((renderer, surface));
        // These measured caps characterize actual backing, uploads and key/
        // parameter serialization. They are not independent sample oracles.
        // Match the fresh-device history so cache reuse cannot change totals.
        for cap in [measured - 1, measured] {
            assert_budget_case(
                &scene,
                budget_options(RenderProfile::Off, ResourceCacheBudget::DEFAULT, work, cap),
                work,
                cap,
                measured,
            );
        }
    }
}

#[test]
fn public_morphology_float32_refusal_is_typed_and_bypassed_only_by_unused_extrema() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
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
    let mut constant = Scene::new();
    constant
        .morphology_layer(
            Layer::new(),
            operation(
                MorphologyInput::new().with_radius(1e300),
                FilterColorSpace::LinearSrgb,
            ),
            MorphologySource::constant(Color::BLACK, full, full).unwrap(),
        )
        .unwrap();
    let pixels = render(&mut renderer, &mut surface, &constant);
    for x in 0..5 {
        assert_pixel(&pixels, x, 0, [0, 0, 0, 255], 1);
    }
    let disabled = render(&mut renderer, &mut surface, &sample_fixture(0.0, 1e300));
    let before = render(&mut renderer, &mut surface, &sample_fixture(0.25, 0.25));
    assert_eq!(before, disabled);
    let stats = renderer.stats();
    let old_texture = published(&surface);
    let publications = surface.headless_publication_count_for_test();
    let active = sample_fixture(2.0, 0.25);
    let error = pollster::block_on(renderer.render(&mut surface, &active, Parameters::default()))
        .unwrap_err();
    let diagnostic = error.runtime_capability_unavailable_diagnostic().unwrap();
    assert_eq!(
        diagnostic.operation(),
        RuntimeOperation::MorphologyRendering
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
    assert_frame_leases_released(&mut renderer);
    assert!(renderer.override_default_device_filter_float32_fact_for_test(supports_float32));
    assert_sample_pixels(&render(&mut renderer, &mut surface, &active));
    assert_frame_leases_released(&mut renderer);
}

fn bounded_morphology_peak_witness(periodic: bool, retention: ResourceCacheBudget) {
    use crate::{EffectPrecision, EffectQualityPolicy, RenderGenerationScope};
    let width = if periodic { 64 } else { 65 };
    let domain = bounds(width, 1);
    let alpha: Vec<u8> = (0..width).map(|x| (32 + (37 * x) % 224) as u8).collect();
    let source = source_scene(
        image(
            width,
            1,
            alpha.iter().flat_map(|a| [255, 255, 255, *a]).collect(),
        ),
        domain,
    );
    let effect = operation(
        MorphologyInput::new()
            .with_operator(MorphologyOperator::Dilate)
            .with_radii(32.0, 0.25),
        FilterColorSpace::Srgb,
    );
    let source = if periodic {
        MorphologySource::periodic(source, domain, domain, domain).unwrap()
    } else {
        MorphologySource::finite(
            source,
            FilterDomains::try_new(domain, domain, domain).unwrap(),
        )
    };
    let mut scene = Scene::new();
    scene
        .morphology_layer(Layer::new(), effect, source)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::RequireHighPrecision)
            .with_resource_cache_budget(retention)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(f64::from(width), 1.0), 1.0))
            .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
    assert_eq!(stats.effect_precision, Some(EffectPrecision::High));
    assert_eq!(stats, renderer.stats());
    let report = renderer.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.cleanup_fault(), None);
    let generation = report.generation().unwrap().unwrap();
    let scope = RenderGenerationScope::Effects;
    assert_eq!(generation.scope_opening(scope, RenderResource::GpuBytes), 0);
    assert_eq!(
        generation.scope_opening(scope, RenderResource::RegisteredCacheGpuBytes),
        0
    );
    let effects = generation.scope_usage(scope);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for x in 0..width {
        // Literal inclusive-center maximum, independent of range tables and
        // resource assignment. A full periodic window visits every tile cell;
        // finite windows include only the nonnegative in-domain candidates.
        let expected = if periodic {
            *alpha.iter().max().unwrap()
        } else {
            alpha
                .iter()
                .enumerate()
                .filter(|(i, _)| (i64::from(x) - *i as i64).abs() <= 32)
                .map(|(_, a)| *a)
                .max()
                .unwrap()
        };
        assert_pixel(&pixels, x, 0, [255, 255, 255, expected], 2);
    }
    // Both source and result are width×1. Current/next table and current/next
    // query require at most four width×1 RGBA32Float physical capacities.
    // Separately owned non-float Effects backing is six RGBA16Float images:
    // root clear, isolated-source clear, capture canonicalization, source
    // composite, morphology resolve, and final layer composite; plus one
    // RGBA8 source capture. Vello workspaces, uniforms and headless targets
    // have distinct generation scopes and are excluded, not estimated here.
    // Therefore the complete Effects bound is width*(4*16+6*8+4):7424/7540.
    // Period64 reaches six table levels; finite65 has clipped lengths33..65
    // and six table/refinement levels. Neither may retain every level's image.
    let bound = u64::from(width) * (4 * 16 + 6 * 8 + 4);
    let gpu_peak = effects.peak(RenderResource::GpuBytes);
    let registered_peak = effects.peak(RenderResource::RegisteredCacheGpuBytes);
    let gpu_live = effects.live(RenderResource::GpuBytes);
    let registered_live = effects.live(RenderResource::RegisteredCacheGpuBytes);
    if retention == ResourceCacheBudget::DISABLED {
        assert_eq!(gpu_live, 0);
        assert_eq!(registered_live, 0);
    }
    assert!(
        gpu_peak <= bound
            && registered_peak <= bound
            && gpu_live <= bound
            && registered_live <= bound,
        "periodic={periodic}, retention={}bytes: Effects peak GPU={gpu_peak}, registered={registered_peak}; closing GPU={gpu_live}, registered={registered_live}; independently bounded backing={bound}",
        retention.bytes(),
    );
}

#[test]
fn public_morphology_many_level_period_peak_is_bounded_with_default_retention() {
    bounded_morphology_peak_witness(true, ResourceCacheBudget::DEFAULT);
}

#[test]
fn public_morphology_many_level_period_peak_is_bounded_with_disabled_retention() {
    bounded_morphology_peak_witness(true, ResourceCacheBudget::DISABLED);
}

#[test]
fn public_morphology_variable_finite_peak_is_bounded_with_default_retention() {
    bounded_morphology_peak_witness(false, ResourceCacheBudget::DEFAULT);
}

#[test]
fn public_morphology_variable_finite_peak_is_bounded_with_disabled_retention() {
    bounded_morphology_peak_witness(false, ResourceCacheBudget::DISABLED);
}

// Physical-custody acceptance cases supplement the committed peak regressions
// above. Expected descriptor bytes below come from fixture geometry, not an
// observed peak or the slot allocator's output.
fn physical_alpha(width: u32) -> Vec<u8> {
    (0..width).map(|x| (32 + (37 * x) % 224) as u8).collect()
}

fn physical_scene(width: u32, output: u32, periodic: bool, radius: f64) -> Scene {
    let input = bounds(width, 1);
    let result = bounds(output, 1);
    let source = source_scene(
        image(
            width,
            1,
            physical_alpha(width)
                .into_iter()
                .flat_map(|a| [255, 255, 255, a])
                .collect(),
        ),
        input,
    );
    let source = if periodic {
        MorphologySource::periodic(source, input, result, result).unwrap()
    } else {
        MorphologySource::finite(
            source,
            FilterDomains::try_new(input, input, result).unwrap(),
        )
    };
    let mut scene = Scene::new();
    scene
        .morphology_layer(
            Layer::new(),
            operation(
                MorphologyInput::new()
                    .with_operator(MorphologyOperator::Dilate)
                    .with_radii(radius, 0.25),
                FilterColorSpace::Srgb,
            ),
            source,
        )
        .unwrap();
    scene
}

fn physical_baseline(width: u32) -> Scene {
    let domain = bounds(width, 1);
    finite_scene(
        source_scene(
            image(width, 1, [0, 255, 0, 255].repeat(width as usize)),
            domain,
        ),
        FilterDomains::try_new(domain, domain, domain).unwrap(),
        operation(MorphologyInput::new(), FilterColorSpace::Srgb),
        Layer::new(),
    )
}

fn physical_renderer(width: u32, retention: ResourceCacheBudget) -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(crate::EffectQualityPolicy::RequireHighPrecision)
            .with_resource_cache_budget(retention)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::new(f64::from(width), 1.0), 1.0))
            .unwrap();
    (renderer, surface)
}

fn assert_physical_pixels(
    pixels: &crate::ImageBuffer,
    input: u32,
    output: u32,
    periodic: bool,
    radius: i64,
) {
    let alpha = physical_alpha(input);
    for x in 0..output {
        let expected = (-radius..=radius)
            .map(|dx| {
                let i = i64::from(x) + dx;
                if periodic {
                    alpha[i.rem_euclid(i64::from(input)) as usize]
                } else if (0..i64::from(input)).contains(&i) {
                    alpha[i as usize]
                } else {
                    0
                }
            })
            .max()
            .unwrap();
        assert_pixel(pixels, x, 0, [255, 255, 255, expected], 2);
    }
}

fn assert_provider_planes(
    observed: &crate::resource::ResourceManagerObservationForTest,
    float_classes: &[(u32, usize)],
    working_classes: &[(u32, usize)],
    capture_classes: &[(u32, usize)],
) {
    let classes = [
        (wgpu::TextureFormat::Rgba32Float, 16, float_classes),
        (wgpu::TextureFormat::Rgba16Float, 8, working_classes),
        (wgpu::TextureFormat::Rgba8Unorm, 4, capture_classes),
    ];
    let mut count = 0;
    let mut bytes = 0;
    for (format, stride, sizes) in classes {
        for &(width, expected) in sizes {
            let entries: Vec<_> = observed
                .effect_textures
                .iter()
                .filter(|entry| {
                    entry.format == format && entry.extent == crate::PhysicalSize::new(width, 1)
                })
                .collect();
            assert_eq!(
                entries.len(),
                expected,
                "actual {format:?} {width}x1 provider entries"
            );
            for entry in entries {
                assert_eq!(entry.bytes, u64::from(width) * stride);
                // Vello writes its RGBA8 capture through storage bindings;
                // working and morphology scratch planes are render targets.
                // All three descriptor owners retain sampled and copy access.
                let write_usage = if format == wgpu::TextureFormat::Rgba8Unorm {
                    wgpu::TextureUsages::STORAGE_BINDING
                } else {
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                };
                assert_eq!(
                    entry.usage,
                    write_usage
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST
                );
            }
            count += expected;
            bytes += u64::from(width) * stride * expected as u64;
        }
    }
    assert_eq!(
        observed.effect_textures.len(),
        count,
        "no unclassified Effects backing"
    );
    assert_eq!(observed.effects_gpu_bytes, bytes);
    assert_eq!(observed.effects_registered_bytes, bytes);
}

fn float_identities(
    observed: &crate::resource::ResourceManagerObservationForTest,
) -> Vec<(crate::resource::ResourceIdentity, u64)> {
    let mut identities: Vec<_> = observed
        .effect_textures
        .iter()
        .filter(|entry| entry.format == wgpu::TextureFormat::Rgba32Float)
        .map(|entry| (entry.resource, entry.native_identity))
        .collect();
    identities.sort_by_key(|(resource, _)| resource.get());
    identities
}

fn reach_before_submission<F: Future>(
    mut future: Pin<&mut F>,
    gate: &crate::gpu_transaction::test_support::GraphBeforeSubmissionGateGuardForTest,
    device: &wgpu::Device,
) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending(),
            "the real encoded graph must remain held before queue submission"
        );
        if let Some(generation) = gate.reached_generation() {
            assert_eq!(gate.submissions(), 0);
            return generation;
        }
        assert!(
            Instant::now() < deadline,
            "encoded graph did not reach its owned pre-submit gate"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

#[test]
fn public_morphology_exact_physical_effects_limits_refuse_before_acquisition_and_retry() {
    for (periodic, width, slots, exact) in [(true, 64, 2, 5376), (false, 65, 3, 6500)] {
        for retention in [ResourceCacheBudget::DEFAULT, ResourceCacheBudget::DISABLED] {
            let (mut renderer, mut surface) = physical_renderer(width, retention);
            let before = render(&mut renderer, &mut surface, &physical_baseline(width));
            let stats = renderer.stats();
            let texture = published(&surface);
            let publications = surface.headless_publication_count_for_test();
            let reader = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap()
                .resource_manager_observation_reader_for_test();
            let opening = reader();
            let opening_bytes = if retention == ResourceCacheBudget::DEFAULT {
                52 * u64::from(width)
            } else {
                0
            };
            assert_eq!(opening.effects_gpu_bytes, opening_bytes);
            assert_eq!(opening.effects_registered_bytes, opening_bytes);
            assert_eq!(opening.effects_idle_bytes, opening_bytes);
            assert!(float_identities(&opening).is_empty());
            renderer
                .set_default_device_effects_gpu_byte_limit_for_test(exact - 1)
                .unwrap();
            let scene = physical_scene(width, width, periodic, 32.0);
            let error =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                    .unwrap_err();
            assert!(
                matches!(
                    error.accounting_diagnostic(),
                    Some(crate::RenderAccountingError::GenerationRefused {
                        quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                        ..
                    })
                ),
                "{error:?}"
            );
            let refused = reader();
            assert_eq!(
                refused.payload_creation_attempts,
                opening.payload_creation_attempts
            );
            assert_eq!(refused.effects_allocations, opening.effects_allocations);
            assert_eq!(refused.effect_textures, opening.effect_textures);
            assert_eq!(refused.effects_gpu_bytes, opening_bytes);
            assert_eq!(renderer.stats(), stats);
            assert_eq!(published(&surface), texture);
            assert_eq!(surface.headless_publication_count_for_test(), publications);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                before
            );
            assert_frame_leases_released(&mut renderer);
            renderer
                .set_default_device_effects_gpu_byte_limit_for_test(exact)
                .unwrap();
            let signal = renderer.default_device_signal_for_test().unwrap();
            let device = renderer
                .default_ready_device_state_borrow_for_test()
                .unwrap()
                .device_for_test()
                .clone();
            let gate = GraphSubmissionGateGuardForTest::install(signal);
            {
                let mut future =
                    Box::pin(renderer.render(&mut surface, &scene, Parameters::default()));
                reach_submission(future.as_mut(), &gate, &device);
                let held = reader();
                assert_provider_planes(&held, &[(width, slots)], &[(width, 6)], &[(width, 1)]);
                assert_eq!(held.effects_gpu_bytes, exact);
                assert!(
                    held.effect_textures
                        .iter()
                        .all(|entry| entry.leased && entry.resolved)
                );
                gate.release();
                finish(future.as_mut(), &device).unwrap();
            }
            drop(gate);
            assert_eq!(
                renderer.stats().effect_precision,
                Some(crate::EffectPrecision::High)
            );
            assert_eq!(
                renderer
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions),
                1
            );
            assert_eq!(
                surface.headless_publication_count_for_test(),
                publications + 1
            );
            assert_physical_pixels(
                &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                width,
                width,
                periodic,
                32,
            );
            assert_frame_leases_released(&mut renderer);
        }
    }
}

#[test]
fn public_morphology_unsubmitted_and_submitted_cancellation_retain_fixed_physical_slots() {
    use crate::gpu_transaction::test_support::GraphBeforeSubmissionGateGuardForTest;
    for cancel_after_submit in [false, true] {
        let (mut renderer, mut surface) = physical_renderer(65, ResourceCacheBudget::DISABLED);
        let before = render(&mut renderer, &mut surface, &physical_baseline(65));
        let stats = renderer.stats();
        let texture = published(&surface);
        let publications = surface.headless_publication_count_for_test();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let device = ready.device_for_test().clone();
        let reader = ready.resource_manager_observation_reader_for_test();
        let signal = renderer.default_device_signal_for_test().unwrap();
        let before_gate = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let after_gate = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let scene = physical_scene(65, 65, false, 32.0);
        {
            let mut future = Box::pin(renderer.render(&mut surface, &scene, Parameters::default()));
            let generation = reach_before_submission(future.as_mut(), &before_gate, &device);
            assert_eq!(signal.active_generation_for_test(), Some(generation));
            let encoded = reader();
            assert_provider_planes(&encoded, &[(65, 3)], &[(65, 6)], &[(65, 1)]);
            assert!(
                encoded
                    .effect_textures
                    .iter()
                    .all(|entry| entry.leased && entry.resolved)
            );
            let identities = float_identities(&encoded);
            assert_eq!(identities.len(), 3);
            assert_eq!(after_gate.submissions(), 0);
            if cancel_after_submit {
                before_gate.release();
                assert_eq!(
                    reach_submission(future.as_mut(), &after_gate, &device),
                    generation
                );
                let submitted = reader();
                assert_provider_planes(&submitted, &[(65, 3)], &[(65, 6)], &[(65, 1)]);
                assert_eq!(float_identities(&submitted), identities);
                assert_eq!(submitted.effects_gpu_bytes, encoded.effects_gpu_bytes);
                assert_eq!(before_gate.submissions(), 1);
            }
            drop(future);
        }
        drop((before_gate, after_gate));
        assert_eq!(signal.active_generation_for_test(), None);
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        assert_eq!(published(&surface), texture);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        assert_frame_leases_released(&mut renderer);
        assert_eq!(reader().effects_gpu_bytes, 0);
        assert_physical_pixels(
            &render(&mut renderer, &mut surface, &scene),
            65,
            65,
            false,
            32,
        );
        assert_frame_leases_released(&mut renderer);
    }
}

fn verify_physical_plane_budget(
    scene: &Scene,
    output: u32,
    floats: &[(u32, usize)],
    working: &[(u32, usize)],
    captures: &[(u32, usize)],
    exact: u64,
) -> crate::ImageBuffer {
    let (mut renderer, mut surface) = physical_renderer(output, ResourceCacheBudget::DISABLED);
    let before = render(&mut renderer, &mut surface, &physical_baseline(output));
    let stats = renderer.stats();
    let texture = published(&surface);
    let publications = surface.headless_publication_count_for_test();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let reader = ready.resource_manager_observation_reader_for_test();
    let opening = reader();
    assert_eq!(opening.effects_gpu_bytes, 0);
    renderer
        .set_default_device_effects_gpu_byte_limit_for_test(exact - 1)
        .unwrap();
    let error = pollster::block_on(renderer.render(&mut surface, scene, Parameters::default()))
        .unwrap_err();
    assert!(
        matches!(
            error.accounting_diagnostic(),
            Some(crate::RenderAccountingError::GenerationRefused {
                quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                ..
            })
        ),
        "{error:?}"
    );
    let refused = reader();
    assert_eq!(
        refused.payload_creation_attempts,
        opening.payload_creation_attempts
    );
    assert_eq!(refused.effects_allocations, opening.effects_allocations);
    assert_eq!(renderer.stats(), stats);
    assert_eq!(published(&surface), texture);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert_frame_leases_released(&mut renderer);
    renderer
        .set_default_device_effects_gpu_byte_limit_for_test(exact)
        .unwrap();
    let gate = GraphSubmissionGateGuardForTest::install(
        renderer.default_device_signal_for_test().unwrap(),
    );
    {
        let mut future = Box::pin(renderer.render(&mut surface, scene, Parameters::default()));
        reach_submission(future.as_mut(), &gate, &device);
        let held = reader();
        assert_provider_planes(&held, floats, working, captures);
        assert_eq!(held.effects_gpu_bytes, exact);
        let ids = float_identities(&held);
        let mut hashes: Vec<_> = ids.iter().map(|(_, native)| *native).collect();
        hashes.sort_unstable();
        hashes.dedup();
        assert_eq!(
            hashes.len(),
            ids.len(),
            "physical slots must own distinct real native textures"
        );
        gate.release();
        finish(future.as_mut(), &device).unwrap();
    }
    drop(gate);
    assert_eq!(
        renderer.stats().effect_precision,
        Some(crate::EffectPrecision::High)
    );
    assert_eq!(
        renderer
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        1
    );
    assert_eq!(
        surface.headless_publication_count_for_test(),
        publications + 1
    );
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_frame_leases_released(&mut renderer);
    assert_eq!(reader().effects_gpu_bytes, 0);
    pixels
}

#[test]
fn public_morphology_distinct_table_query_capacities_acquire_four_exact_planes() {
    let scene = physical_scene(65, 33, false, 32.0);
    // Three source working planes65, three output/root working planes33,
    // one65-cell RGBA8 capture. T slots are65 and Q slots33; neither class
    // can borrow the other's capacity merely because its logical width fits.
    let nonfloat = (3 * 65 + 3 * 33) * 8 + 65 * 4;
    let float = 2 * 65 * 16 + 2 * 33 * 16;
    assert_eq!((nonfloat, float), (2612, 3136));
    let pixels = verify_physical_plane_budget(
        &scene,
        33,
        &[(65, 2), (33, 2)],
        &[(65, 3), (33, 3)],
        &[(65, 1)],
        nonfloat + float,
    );
    assert_physical_pixels(&pixels, 65, 33, false, 32);
}

fn physical_siblings(first: (u32, bool), second: (u32, bool), output: u32) -> Scene {
    let mut scene = Scene::new();
    // Ordinary wrappers are transparent. The two morphology receivers remain
    // ordered siblings in one frame and one actual graph submission.
    scene.layer(Layer::new(), |children| {
        *children = physical_scene(first.0, output, first.1, 32.0)
    });
    scene.layer(Layer::new(), |children| {
        *children = physical_scene(second.0, output, second.1, 32.0)
    });
    scene
}

fn assert_sibling_pixels(
    pixels: &crate::ImageBuffer,
    first: (u32, bool),
    second: (u32, bool),
    output: u32,
) {
    let maximum = |width: u32, periodic: bool, x: u32| {
        let alpha = physical_alpha(width);
        (-32_i64..=32)
            .map(|dx| {
                let i = i64::from(x) + dx;
                if periodic {
                    alpha[i.rem_euclid(i64::from(width)) as usize]
                } else if (0..i64::from(width)).contains(&i) {
                    alpha[i as usize]
                } else {
                    0
                }
            })
            .max()
            .unwrap()
    };
    for x in 0..output {
        let bottom = f64::from(maximum(first.0, first.1, x)) / 255.0;
        let top = f64::from(maximum(second.0, second.1, x)) / 255.0;
        let alpha = ((top + bottom * (1.0 - top)) * 255.0).round() as u8;
        assert_pixel(pixels, x, 0, [255, 255, 255, alpha], 3);
    }
}

#[test]
fn public_morphology_compatible_sibling_routes_share_two_frame_wide_native_slots() {
    let scene = physical_siblings((64, true), (64, true), 64);
    // Root1 + each chain's source3/resolve1/final-composite1 gives11 working
    // planes. Each chain captures once; its two float slots can be reassigned
    // to the second chain after the first's final morphology read.
    let nonfloat = 11 * 64 * 8 + 2 * 64 * 4;
    let float = 2 * 64 * 16;
    assert_eq!((nonfloat, float), (6144, 2048));
    let pixels = verify_physical_plane_budget(
        &scene,
        64,
        &[(64, 2)],
        &[(64, 11)],
        &[(64, 2)],
        nonfloat + float,
    );
    assert_sibling_pixels(&pixels, (64, true), (64, true), 64);
}

#[test]
fn public_morphology_different_sibling_capacity_classes_keep_their_actual_sum() {
    let scene = physical_siblings((64, true), (65, false), 65);
    // The64-cell period needs2 slots and the65-cell variable finite route3.
    // Their descriptor classes differ, so all five real backings remain
    // charged. Working planes: first source3 at64; root1, second source3,
    // and both resolve/final-composite pairs at65, totaling8 at65.
    let nonfloat = (3 * 64 + 8 * 65) * 8 + (64 + 65) * 4;
    let float = 2 * 64 * 16 + 3 * 65 * 16;
    assert_eq!((nonfloat, float), (6212, 5168));
    let pixels = verify_physical_plane_budget(
        &scene,
        65,
        &[(64, 2), (65, 3)],
        &[(64, 3), (65, 8)],
        &[(64, 1), (65, 1)],
        nonfloat + float,
    );
    assert_sibling_pixels(&pixels, (64, true), (65, false), 65);
}

#[test]
fn public_morphology_changed_radius_reuses_warm_capacity_without_level_backings() {
    let (mut renderer, mut surface) = physical_renderer(64, ResourceCacheBudget::DEFAULT);
    let first = physical_scene(64, 64, true, 32.0);
    assert_physical_pixels(
        &render(&mut renderer, &mut surface, &first),
        64,
        64,
        true,
        32,
    );
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let reader = ready.resource_manager_observation_reader_for_test();
    let warm = reader();
    assert_provider_planes(&warm, &[(64, 2)], &[(64, 6)], &[(64, 1)]);
    assert_eq!(warm.effects_gpu_bytes, 5376);
    assert_eq!(warm.effects_idle_bytes, 5376);
    let identities = float_identities(&warm);
    renderer
        .set_default_device_effects_gpu_byte_limit_for_test(5376)
        .unwrap();
    let scene = physical_scene(64, 64, true, 8.0);
    let gate = GraphSubmissionGateGuardForTest::install(
        renderer.default_device_signal_for_test().unwrap(),
    );
    {
        let mut future = Box::pin(renderer.render(&mut surface, &scene, Parameters::default()));
        reach_submission(future.as_mut(), &gate, &device);
        let held = reader();
        assert_provider_planes(&held, &[(64, 2)], &[(64, 6)], &[(64, 1)]);
        assert_eq!(float_identities(&held), identities);
        assert_eq!(held.effects_allocations, warm.effects_allocations);
        gate.release();
        finish(future.as_mut(), &device).unwrap();
    }
    drop(gate);
    let report = renderer.last_attempt().unwrap();
    let generation = report.generation().unwrap().unwrap();
    assert_eq!(
        generation.scope_opening(
            crate::RenderGenerationScope::Effects,
            RenderResource::GpuBytes
        ),
        5376
    );
    assert_eq!(
        generation.scope_opening(
            crate::RenderGenerationScope::Effects,
            RenderResource::RegisteredCacheGpuBytes
        ),
        5376
    );
    assert_physical_pixels(
        &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        64,
        64,
        true,
        8,
    );
    let closing = reader();
    assert_eq!(float_identities(&closing), identities);
    assert_eq!(closing.effects_allocations, warm.effects_allocations);
    assert_eq!(closing.effects_gpu_bytes, 5376);
    assert_eq!(closing.effects_idle_bytes, 5376);
    assert_frame_leases_released(&mut renderer);
}

#[test]
fn public_morphology_capacity_padding_canaries_cannot_enter_logical_extrema() {
    let input = bounds(65, 1);
    let output = bounds(33, 1);
    let (mut renderer, mut surface) = renderer_and_surface(33, 1);
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for operator in [MorphologyOperator::Dilate, MorphologyOperator::Erode] {
            let scene = finite_scene(
                source_scene(image(65, 1, [255, 255, 255, 128].repeat(65)), input),
                FilterDomains::try_new(input, input, output).unwrap(),
                operation(
                    MorphologyInput::new()
                        .with_operator(operator)
                        .with_radii(16.0, 0.25),
                    FilterColorSpace::Srgb,
                ),
                Layer::new(),
            );
            for sentinel in [-2.0, 2.0] {
                // The actual scratch attachment clear initializes capacity padding
                // to an impossible input value. Its logical viewport/scissor then
                // overwrites only the current valid table/query region.
                let _canary =
                    crate::pass::MorphologyPaddingClearGuardForTest::install(wgpu::Color {
                        r: sentinel,
                        g: sentinel,
                        b: sentinel,
                        a: sentinel,
                    });
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    format,
                ))
                .unwrap();
                let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                for x in 0..33 {
                    // Inclusive R16 erosion reaches finite exterior only for x<16;
                    // every dilation window intersects the uniform half-alpha row.
                    let expected = if operator == MorphologyOperator::Erode && x < 16 {
                        [0, 0, 0, 0]
                    } else {
                        [255, 255, 255, 128]
                    };
                    assert_pixel(&pixels, x, 0, expected, 2);
                }
                assert_frame_leases_released(&mut renderer);
            }
        }
    }
}
