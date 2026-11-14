//! Existing native parameters stay owned while real encoded commands need them.
use super::convolution::{assert_pixel, bounds, image};
use crate::{
    Color, OffsetInput, Options, Parameters, RenderAccountingOptions, RenderGenerationScope,
    RenderProfile, RenderResource, RenderRoute, Renderer, ResourceCacheBudget, Scene, Size,
    gpu_transaction::test_support::{
        GraphBeforeSubmissionGateGuardForTest, GraphSubmissionGateGuardForTest,
    },
    pass::{
        ParameterFamilyForTest as Family, ParameterProbeForTest, ParameterStageForTest as Stage,
    },
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

fn offset_scene() -> Scene {
    let full = bounds(4, 1);
    let source = bounds(3, 1);
    super::offset::image_scene(
        image(3, 1, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]),
        source,
        super::offset::operation(OffsetInput::new().with_dx(1.0), source, full, full),
    )
}
fn wait<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device, reached: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending(),
            "real rendering must remain held at its selected submission boundary"
        );
        if reached() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native frame did not reach its submission boundary"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}
fn finish<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device) -> F::Output {
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
            "real native frame failed to resolve"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

#[test]
fn native_offset_parameter_backing_survives_encoded_commands_and_submission() {
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
    let mut baseline = Scene::new();
    baseline.fill(bounds(4, 1), Color::BLACK);
    let old_stats =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    let old_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let publications = surface.headless_publication_count_for_test();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let probe = ParameterProbeForTest::new(ready.generation_for_test());
    let read = probe.reader();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let submitted = GraphSubmissionGateGuardForTest::install(signal);
    let scene = offset_scene();
    {
        let mut future = Box::pin(renderer.render(&mut surface, &scene, Parameters::default()));
        wait(future.as_mut(), &device, || {
            before.reached_generation().is_some()
        });
        assert_eq!(before.submissions(), 0);
        before.release();
        wait(future.as_mut(), &device, || {
            submitted.submitted_generation().is_some()
        });
        assert_eq!(submitted.submissions(), 1);
        submitted.release();
        let stats = finish(future.as_mut(), &device).unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        assert_ne!(stats.route, old_stats.route);
    }
    drop((before, submitted, probe));
    let observed = read();
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for (x, rgba) in [
        (0, [0; 4]),
        (1, [255, 0, 0, 255]),
        (2, [0, 255, 0, 255]),
        (3, [0, 0, 255, 255]),
    ] {
        assert_pixel(&output, x, 0, rgba, 1);
    }
    assert_ne!(output, old_pixels);
    assert_eq!(
        surface.headless_publication_count_for_test(),
        publications + 1
    );
    assert_eq!(
        renderer
            .device_accounting_snapshot(Some(&surface))
            .unwrap()
            .scope_usage(RenderGenerationScope::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    let resources = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert_eq!(
        (resources.active_frame_count, resources.leased_count),
        (0, 0)
    );
    assert_eq!(resources.accounting_fault_for_test(), None);
    // The real scene still renders on an identical-input retry before the
    // retention assertion is evaluated; RED cannot be a setup/pixel failure.
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        output
    );

    let prepared = observed
        .iter()
        .find(|s| s.stage == Stage::Prepared)
        .unwrap();
    let prepared_buffers = prepared.buffers.as_ref().unwrap();
    let offset = prepared_buffers
        .iter()
        .filter(|b| b.family == Family::Offset)
        .collect::<Vec<_>>();
    assert!(
        !offset.is_empty(),
        "the nonzero native offset must create real parameters: {observed:?}"
    );
    assert!(offset.iter().all(|b| {
        b.bytes == 256
            && b.usage
                .contains(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }));
    let required = offset.iter().map(|b| b.bytes).sum::<u64>();
    assert!(prepared.pass_buffer_bytes >= required);
    let encoded = observed.iter().find(|s| s.stage == Stage::Encoded).unwrap();
    assert_eq!(encoded.command_encoder_objects, 1);
    assert_eq!(encoded.command_buffer_objects, 0);
    let mut expected = offset.iter().map(|b| b.native_identity).collect::<Vec<_>>();
    expected.sort_unstable();
    let mut alive = encoded
        .buffers
        .as_ref()
        .unwrap()
        .iter()
        .filter(|b| b.family == Family::Offset)
        .map(|b| b.native_identity)
        .collect::<Vec<_>>();
    alive.sort_unstable();
    assert_eq!(
        alive, expected,
        "the same actual native offset buffers must outlive logical use; preparedBytes={}, encodedBytes={}, encoderObjects={}; observations={observed:?}",
        prepared.pass_buffer_bytes, encoded.pass_buffer_bytes, encoded.command_encoder_objects
    );
    for stage in [Stage::Encoded, Stage::BeforeSubmit, Stage::Submitted] {
        let snapshot = observed.iter().find(|s| s.stage == stage).unwrap();
        assert!(
            snapshot.pass_buffer_bytes >= required,
            "native backing was refunded at{stage:?}: {observed:?}"
        );
    }
}
