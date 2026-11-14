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
