//! Failure atomicity through the admitted public Layer receiver and real queue
//! submission. The gate pauses only after the submitted graph reaches its owner.
#![cfg(not(target_arch = "wasm32"))]

use super::convolution::{bounds, image};
use super::gaussian::{finite_scene, operation, render, source_scene};
use crate::{
    ErrorCode, FilterColorSpace, FilterDomains, FilterEdgeMode, GpuFaultKind, Options, Parameters,
    RenderAttemptStatus, RenderResource, RenderRoute, Renderer, ResourceCacheBudget,
    RuntimeCapabilityUnavailableReason, Scene, Size, Surface,
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
    let effect = operation(4.0, 1.0, FilterColorSpace::LinearSrgb);
    let domains = FilterDomains::try_new(domain, domain, domain).unwrap();
    let build = |rgba: Vec<u8>| {
        finite_scene(
            source_scene(image(2, 1, rgba), domain),
            domains,
            FilterEdgeMode::Duplicate,
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
        _ => panic!("a completed public gaussian frame must retain its publication"),
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
            "public gaussian did not reach real graph submission"
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
            "public gaussian transaction did not resolve before its diagnostic deadline"
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
fn public_gaussian_post_submit_cancellation_preserves_pixels_stats_and_custody_then_retries() {
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
fn public_gaussian_scoped_validation_failure_preserves_publication_and_can_retry() {
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
        // scope, after its real gaussian command buffer was submitted.
        let _invalid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Surgeist public gaussian scoped failure fixture"),
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
fn public_gaussian_first_terminal_fault_aborts_publication_and_retires_once() {
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
            "first gaussian terminal fault",
        );
        signal.record_uncaptured_fault_for_test(
            GpuFaultKind::Internal,
            "later gaussian terminal fault",
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
    assert!(error.to_string().contains("first gaussian terminal fault"));
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
    // this observation does not execute or replace the gaussian receiver.
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
