//! Atomic publication through the public active native displacement receiver.
//! Existing gates expose an encoded graph before its real queue submission and
//! the same graph after submission; neither substitutes a pass or a fake device.
#![cfg(not(target_arch = "wasm32"))]

use super::convolution::{bounds, image};
use super::displacement::{active, finite_scene, native_map, source_scene};
use crate::{
    DisplacementChannel as Channel, ErrorCode, FilterDomains, GpuFaultKind, ImageBuffer, Layer,
    Options, Parameters, RenderAttemptStatus, RenderResource, RenderRoute, Renderer,
    ResourceCacheBudget, RuntimeCapabilityUnavailableReason, Scene, Size, Surface,
    gpu_transaction::test_support::{
        GraphBeforeSubmissionGateGuardForTest, GraphSubmissionGateGuardForTest,
    },
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

const PRIOR_PIXELS: [u8; 8] = [255, 0, 255, 255, 255, 0, 255, 255];
const DISPLACED_PIXELS: [u8; 8] = [0, 0, 255, 255, 255, 255, 0, 255];

fn active_scene() -> Scene {
    let full = bounds(4, 3);
    let output = bounds(2, 1);
    let row = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
    ];
    // R255 at scale4 gives +2, while G128 gives0. Each of the three
    // independently authored source rows is red, green, blue, yellow. Thus
    // output x0 and x1 copy blue/yellow, retaining opaque source alpha even
    // though the native straight map alpha is64.
    finite_scene(
        source_scene(image(4, 3, row.repeat(3)), full),
        FilterDomains::try_new(full, full, output).unwrap(),
        active(4.0, Channel::R, Channel::G),
        native_map(1, 1, vec![255, 128, 0, 64], output),
        Layer::new(),
    )
}

fn fresh_renderer() -> (Renderer, Surface) {
    let options = Options::default().with_resource_cache_budget(ResourceCacheBudget::DISABLED);
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let surface = pollster::block_on(renderer.create_headless(Size::new(2.0, 1.0), 1.0)).unwrap();
    (renderer, surface)
}

fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats, renderer.stats());
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn setup() -> (
    Renderer,
    Surface,
    Scene,
    wgpu::Device,
    wgpu::Queue,
    wgpu::Texture,
) {
    let (mut renderer, mut surface) = fresh_renderer();
    let mut prior = Scene::new();
    prior.image(
        image(2, 1, PRIOR_PIXELS.to_vec()),
        bounds(2, 1),
        crate::ImageFit::Stretch,
    );
    assert_eq!(
        render(&mut renderer, &mut surface, &prior).rgba(),
        PRIOR_PIXELS
    );
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let queue = ready.queue_for_test().clone();
    let texture = published(&surface);
    (renderer, surface, active_scene(), device, queue, texture)
}

fn published(surface: &Surface) -> wgpu::Texture {
    match &surface.backend {
        SurfaceBackend::Headless {
            resources: HeadlessResources::Ready { texture },
            ..
        } => texture.deref().clone(),
        _ => panic!("completed native displacement fixture requires a published headless texture"),
    }
}

fn reach_before_submission<F: Future>(
    mut future: Pin<&mut F>,
    gate: &GraphBeforeSubmissionGateGuardForTest,
    device: &wgpu::Device,
) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending(),
            "public native displacement must remain held at its encoded pre-submit gate"
        );
        if let Some(generation) = gate.reached_generation() {
            assert_eq!(gate.submissions(), 0);
            return generation;
        }
        assert!(
            Instant::now() < deadline,
            "native displacement did not reach its pre-submit gate"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
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
            "public native displacement must remain held at its real post-submit gate"
        );
        if let Some(generation) = gate.submitted_generation() {
            assert_eq!(gate.submissions(), 1);
            return generation;
        }
        assert!(
            Instant::now() < deadline,
            "native displacement did not reach real queue submission"
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
            "native displacement transaction did not resolve before deadline"
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
    let observed = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert_eq!(observed.leased_count, 0);
    assert_eq!(observed.active_frame_count, 0);
    assert_eq!(observed.resolved_lease_count, 0);
    assert_eq!(observed.accounting_fault_for_test(), None);
    assert_eq!(observed.retained_bytes, 0);
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
    assert_eq!(render(renderer, surface, next).rgba(), DISPLACED_PIXELS);
    assert_eq!(renderer.stats().route, Some(RenderRoute::GpuGraph));
    assert_ready_custody(renderer, surface);
}

fn canceled_at_submission_boundary(after_submit: bool) {
    let (mut renderer, mut surface, next, device, _queue, old_texture) = setup();
    let stats = renderer.stats();
    let publications = surface.headless_publication_count_for_test();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let cache = ready.device_pass_cache_counts_for_test();
    let reader = ready.resource_manager_observation_reader_for_test();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let after = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
    {
        let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
        let generation = reach_before_submission(future.as_mut(), &before, &device);
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        let encoded = reader();
        assert!(encoded.leased_count > 0);
        // Graph preparation owns one FrameResourceScope. This finite first
        // source has exactly one Image span, whose Vello capture allocates a
        // second scope retained by PendingVelloResourceCommit. The native map
        // and address/record buffers use the graph scope, not another capture.
        assert_eq!(encoded.active_frame_count, 1 + 1);
        assert!(encoded.resolved_lease_count > 0);
        assert!(encoded.effect_texture_count_for_test() >= 2);
        assert_eq!(encoded.accounting_fault_for_test(), None);
        let identities = encoded.entry_identities_for_test().to_vec();
        assert_eq!(after.submissions(), 0);
        if after_submit {
            before.release();
            assert_eq!(
                reach_submission(future.as_mut(), &after, &device),
                generation
            );
            assert_eq!(before.submissions(), 1);
            let submitted = reader();
            assert_eq!(submitted.entry_identities_for_test(), identities);
            assert_eq!(submitted.effects_gpu_bytes, encoded.effects_gpu_bytes);
        }
        drop(future);
    }
    drop((before, after));
    assert_eq!(signal.active_generation_for_test(), None);
    assert_eq!(renderer.stats(), stats);
    assert_eq!(surface.headless_publication_count_for_test(), publications);
    assert_eq!(published(&surface), old_texture);
    assert_eq!(
        renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .device_pass_cache_counts_for_test(),
        cache
    );
    assert_eq!(
        renderer.last_attempt().unwrap().attempt().unwrap().status(),
        RenderAttemptStatus::Canceled
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface))
            .unwrap()
            .rgba(),
        PRIOR_PIXELS
    );
    assert_ready_custody(&mut renderer, &surface);
    retry(&mut renderer, &mut surface, &next);
}

#[test]
fn public_displacement_pre_submit_cancellation_preserves_publication_and_releases_encoded_resources()
 {
    canceled_at_submission_boundary(false);
}

#[test]
fn public_displacement_post_submit_cancellation_preserves_publication_and_releases_submitted_resources()
 {
    canceled_at_submission_boundary(true);
}

#[test]
fn public_displacement_scoped_validation_fault_at_each_submission_boundary_preserves_cache_and_retries()
 {
    for after_submit in [false, true] {
        let (mut renderer, mut surface, next, device, _queue, old_texture) = setup();
        let stats = renderer.stats();
        let publications = surface.headless_publication_count_for_test();
        let cache = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .device_pass_cache_counts_for_test();
        let signal = renderer.default_device_signal_for_test().unwrap();
        let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let after = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let error = {
            let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
            let generation = reach_before_submission(future.as_mut(), &before, &device);
            if after_submit {
                before.release();
                assert_eq!(
                    reach_submission(future.as_mut(), &after, &device),
                    generation
                );
            }
            assert_eq!(signal.active_generation_for_test(), Some(generation));
            // Real WGPU validation error in the actual render's still-open
            // error scope, after source/map/table resources have been encoded.
            let _invalid = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Surgeist native displacement scoped validation fixture"),
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
            before.release();
            after.release();
            finish(future.as_mut(), &device)
                .expect_err("actual render scope must return injected validation failure")
        };
        drop((before, after));
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
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface))
                .unwrap()
                .rgba(),
            PRIOR_PIXELS
        );
        assert_ready_custody(&mut renderer, &surface);
        retry(&mut renderer, &mut surface, &next);
    }
}

#[test]
fn public_displacement_first_terminal_fault_preserves_publication_retires_once_and_explicit_recovery_retries()
 {
    for after_submit in [false, true] {
        let (mut renderer, mut surface, next, device, queue, old_texture) = setup();
        let stats = renderer.stats();
        let publications = surface.headless_publication_count_for_test();
        let signal = renderer.default_device_signal_for_test().unwrap();
        let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let after = GraphSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let (error, generation) = {
            let mut future = Box::pin(renderer.render(&mut surface, &next, Parameters::default()));
            let generation = reach_before_submission(future.as_mut(), &before, &device);
            if after_submit {
                before.release();
                assert_eq!(
                    reach_submission(future.as_mut(), &after, &device),
                    generation
                );
            }
            assert_eq!(signal.active_generation_for_test(), Some(generation));
            signal.record_uncaptured_fault_for_test(
                GpuFaultKind::Validation,
                "first native displacement terminal fault",
            );
            signal.record_uncaptured_fault_for_test(
                GpuFaultKind::Internal,
                "later native displacement terminal fault",
            );
            before.release();
            after.release();
            (
                finish(future.as_mut(), &device)
                    .expect_err("terminal signal must reject actual native displacement frame"),
                generation,
            )
        };
        drop((before, after));
        let first = signal.first_terminal().unwrap();
        assert_eq!(signal.active_generation_for_test(), None);
        assert_eq!(first.operation_generation_for_test(), Some(generation));
        assert_eq!(error.code(), ErrorCode::RenderFailed);
        assert!(
            error
                .to_string()
                .contains("first native displacement terminal fault")
        );
        assert_eq!(
            first
                .error(crate::RuntimeOperation::SurfaceRendering)
                .runtime_capability_unavailable_diagnostic()
                .unwrap()
                .reason(),
            RuntimeCapabilityUnavailableReason::DeviceFaulted {
                kind: GpuFaultKind::Validation
            }
        );
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        assert_eq!(published(&surface), old_texture);
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
        assert!(pollster::block_on(renderer.read_headless(&surface)).is_err());
        assert_eq!(
            native_publication_pixels(&device, &queue, &old_texture),
            PRIOR_PIXELS
        );
        let refusal =
            pollster::block_on(renderer.render(&mut surface, &next, Parameters::default()))
                .unwrap_err();
        assert_eq!(
            refusal
                .runtime_capability_unavailable_diagnostic()
                .unwrap()
                .reason(),
            RuntimeCapabilityUnavailableReason::DeviceFaulted {
                kind: GpuFaultKind::Validation
            }
        );
        assert!(Arc::ptr_eq(&first, &signal.first_terminal().unwrap()));
        let repeated = renderer.device_accounting_snapshot(Some(&surface)).unwrap();
        assert_eq!(repeated.identity(), retired.identity());
        assert_eq!(repeated.usage(), retired.usage());
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        // A fresh renderer is the explicit device-generation recovery boundary.
        // The exact same Scene and immutable native map must then execute.
        let (mut replacement, mut replacement_surface) = fresh_renderer();
        retry(&mut replacement, &mut replacement_surface, &next);
    }
}

fn native_publication_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> [u8; 8] {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Surgeist retained displacement publication observation"),
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
    let result = {
        let mapped = slice.get_mapped_range();
        <[u8; 8]>::try_from(&mapped[..8]).unwrap()
    };
    staging.unmap();
    result
}
