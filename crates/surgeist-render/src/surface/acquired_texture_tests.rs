//! Real display-free acquired-owner characterization, independent of frame execution.
use super::*;
use crate::{
    Options, RenderAccountingError, RenderAccountingOptions, RenderLimits, RenderPhase,
    RenderProfile, RenderQuantity, RenderRefusalReason, RenderResource, RenderWork, Renderer,
    accounting::{FrameFixture, RendererCpu},
    backend::configured_display_free_presented_surface_for_test,
};
use std::{sync::mpsc, time::Duration};

struct Fixture {
    _renderer: Renderer,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Surface,
}

impl Fixture {
    fn new() -> Self {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let surface = configured_display_free_presented_surface_for_test(&mut renderer);
        let (device, queue) = renderer.default_wgpu_device_queue().unwrap();
        Self {
            device: device.clone(),
            queue: queue.clone(),
            surface,
            _renderer: renderer,
        }
    }

    fn acquire(
        &mut self,
        outcome: PresentedAcquireOutcomeForTest,
    ) -> AcquiredPresentedSurfaceTexture {
        let presented = presented_mut(&mut self.surface);
        presented.set_acquire_outcome_for_test(outcome);
        match (outcome, presented.acquire_texture(&self.device)) {
            (
                PresentedAcquireOutcomeForTest::Success,
                PresentedSurfaceAcquire::Success(texture),
            )
            | (
                PresentedAcquireOutcomeForTest::Suboptimal,
                PresentedSurfaceAcquire::Suboptimal(texture),
            ) => texture,
            _ => panic!("the configured provider must return the selected actual acquired owner"),
        }
    }

    fn clear_black(&self, view: &wgpu::TextureView) {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        self.queue.submit([encoder.finish()]);
    }

    fn publish_black(&mut self) -> OwnedGpu<wgpu::TextureView> {
        let acquired = self.acquire(PresentedAcquireOutcomeForTest::Success);
        let view = acquired.create_view(&view_generation(), None).unwrap();
        self.clear_black(&view);
        acquired.present();
        view
    }
}

fn presented(surface: &Surface) -> &PresentedSurface {
    let SurfaceBackend::Presented { surface, .. } = &surface.backend else {
        panic!("the real configured fixture must be presented");
    };
    surface
}

fn presented_mut(surface: &mut Surface) -> &mut PresentedSurface {
    let SurfaceBackend::Presented { surface, .. } = &mut surface.backend else {
        panic!("the real configured fixture must be presented");
    };
    surface
}

fn view_generation() -> Generation {
    Generation::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )
    .unwrap()
}

fn assert_observation(
    observation: DisplayFreePresentedSurfaceObservationForTest,
    acquired: usize,
    presented: usize,
    discarded: usize,
) {
    assert_eq!(observation.acquire_attempt_count, acquired);
    assert_eq!(observation.acquire_count, acquired);
    assert_eq!(observation.present_count, presented);
    assert_eq!(observation.discarded_count, discarded);
}

fn assert_view_custody(generation: &Generation, live: u64, created: u64) {
    let usage = generation.usage().unwrap();
    assert_eq!(usage.live(RenderResource::GpuBytes), 0);
    assert_eq!(usage.peak(RenderResource::GpuBytes), 0);
    assert_eq!(usage.work(RenderWork::ResourceAllocations), 0);
    assert_eq!(usage.live(RenderResource::OpaqueGpuObjects), live);
    assert_eq!(usage.peak(RenderResource::OpaqueGpuObjects), created);
    assert_eq!(usage.work(RenderWork::GpuObjectsCreated), created);
    assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), created);
    generation.ensure_healthy().unwrap();
}

/// Copy the actual parent into a tiny mapped buffer. No parent handle is cloned.
/// Submission and map completion both have diagnostic deadlines.
fn assert_black_parent(device: &wgpu::Device, queue: &wgpu::Queue, parent: &wgpu::Texture) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Surgeist acquired-view parent lifetime readback"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        parent.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(2),
            },
        },
        wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
    );
    let submission = queue.submit([encoder.finish()]);
    let (sender, receiver) = mpsc::sync_channel(1);
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(10)),
        })
        .expect("the tiny parent copy must complete within ten seconds");
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the tiny parent map callback must complete within ten seconds")
        .expect("the surviving real parent must remain readable");
    let mapped = buffer.slice(..).get_mapped_range();
    for row in [0, 256] {
        assert_eq!(&mapped[row..row + 8], &[0, 0, 0, 255, 0, 0, 0, 255]);
    }
    drop(mapped);
    buffer.unmap();
}

#[test]
fn borrowed_view_then_present_transfers_its_exact_parent_once() {
    let mut fixture = Fixture::new();
    let generation = view_generation();
    let acquired = fixture.acquire(PresentedAcquireOutcomeForTest::Success);
    assert_view_custody(&generation, 0, 0);
    let view = acquired.create_view(&generation, None).unwrap();
    fixture.clear_black(&view);
    assert_view_custody(&generation, 1, 1);
    acquired.present();
    assert_observation(presented(&fixture.surface).observation_for_test(), 1, 1, 0);
    let texture = presented_mut(&mut fixture.surface)
        .take_last_presented_texture_for_test()
        .unwrap();
    assert_eq!(&texture, view.texture());
    assert_black_parent(&fixture.device, &fixture.queue, &texture);
    assert!(
        presented_mut(&mut fixture.surface)
            .take_last_presented_texture_for_test()
            .is_none()
    );
    drop(texture);
    assert_view_custody(&generation, 1, 1);
    drop(view);
    assert_view_custody(&generation, 0, 1);
    assert_observation(presented(&fixture.surface).observation_for_test(), 1, 1, 0);
}

#[test]
fn ordinary_and_suboptimal_discard_preserve_the_prior_real_publication() {
    for outcome in [
        PresentedAcquireOutcomeForTest::Success,
        PresentedAcquireOutcomeForTest::Suboptimal,
    ] {
        let mut fixture = Fixture::new();
        let prior_view = fixture.publish_black();
        let acquired = fixture.acquire(outcome);
        assert_observation(presented(&fixture.surface).observation_for_test(), 2, 1, 0);
        drop(acquired);
        assert_observation(presented(&fixture.surface).observation_for_test(), 2, 1, 1);
        let retained = presented_mut(&mut fixture.surface)
            .take_last_presented_texture_for_test()
            .unwrap();
        assert_eq!(&retained, prior_view.texture());
        assert_black_parent(&fixture.device, &fixture.queue, &retained);
    }
}

#[test]
fn surviving_real_view_releases_fixture_state_and_keeps_its_parent_readable() {
    for outcome in [
        PresentedAcquireOutcomeForTest::Success,
        PresentedAcquireOutcomeForTest::Suboptimal,
    ] {
        let mut fixture = Fixture::new();
        let generation = view_generation();
        let observation = presented(&fixture.surface).observation_handle_for_test();
        let weak = Arc::downgrade(&observation.0);
        let acquired = fixture.acquire(outcome);
        let view = acquired.create_view(&generation, None).unwrap();
        fixture.clear_black(&view);
        let Fixture {
            _renderer,
            device,
            queue,
            surface,
        } = fixture;
        drop(surface);
        drop(observation);
        let state = weak.upgrade().expect(
            "the actual acquired owner must retain fixture state after surface and observer drop",
        );
        assert_observation(state.lock().unwrap().observation, 1, 0, 0);
        drop(acquired);
        assert_observation(state.lock().unwrap().observation, 1, 0, 1);
        drop(state);
        assert!(
            weak.upgrade().is_none(),
            "a surviving actual WGPU view must not retain fixture state"
        );
        assert_view_custody(&generation, 1, 1);
        assert_black_parent(&device, &queue, view.texture());
        drop(view);
        assert_view_custody(&generation, 0, 1);
    }
}

#[test]
fn refused_view_admission_preserves_cause_publication_and_owner_until_drop() {
    let mut fixture = Fixture::new();
    let prior_view = fixture.publish_black();
    let generation = view_generation();
    let acquired = fixture.acquire(PresentedAcquireOutcomeForTest::Success);
    let options = RenderAccountingOptions::new()
        .with_profile(RenderProfile::Summary { timing: false })
        .with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::ResourceAllocationRequests, 0)
                .unwrap(),
        );
    let cpu = RendererCpu::new(options).unwrap();
    let mut frame = FrameFixture::new(&cpu, options);
    let finished = frame
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation)?;
            acquired
                .create_view(&generation, Some(attempt.account()))
                .map(drop)
        })
        .unwrap();
    let (result, _, report, _) = finished.into_parts();
    let diagnostic = result.unwrap_err().accounting_diagnostic().unwrap();
    let RenderAccountingError::Refused(refusal) = diagnostic else {
        panic!("view admission must return the real allocation-request refusal");
    };
    assert_eq!(refusal.phase(), RenderPhase::Attempt);
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::ResourceAllocationRequests)
    );
    assert_eq!(refusal.current(), 0);
    assert_eq!(refusal.amount(), 1);
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
    assert_eq!(report.accounting_diagnostic(), Some(diagnostic));
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceAllocationRequests),
        0
    );
    assert_view_custody(&generation, 0, 0);
    assert_observation(presented(&fixture.surface).observation_for_test(), 2, 1, 0);
    let observation = presented(&fixture.surface).observation_handle_for_test();
    {
        let state = observation.0.lock().unwrap();
        assert_eq!(
            state.last_presented_texture.as_ref().unwrap(),
            prior_view.texture()
        );
        assert_black_parent(
            &fixture.device,
            &fixture.queue,
            state.last_presented_texture.as_ref().unwrap(),
        );
    }
    drop(acquired);
    assert_observation(observation.snapshot_for_test(), 2, 1, 1);
    let retained = presented_mut(&mut fixture.surface)
        .take_last_presented_texture_for_test()
        .unwrap();
    assert_eq!(&retained, prior_view.texture());
    assert_black_parent(&fixture.device, &fixture.queue, &retained);
    assert_view_custody(&generation, 0, 0);
}
