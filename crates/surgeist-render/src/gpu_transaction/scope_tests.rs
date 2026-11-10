//! Real scope ownership characterizations. No backend OOM/internal fault is induced.
#![cfg(not(target_arch = "wasm32"))]

use super::test_support::{ScopePopGate, ScopePopPosition};
use super::*;
use crate::{
    Options, Renderer, ResourceCacheBudget,
    accounting::{
        FrameFixture, RenderAccountingOptions, RenderProfile, RenderResource, RenderWork,
        RendererCpu,
    },
    resource::{ResourceCacheKey, ResourceManager},
};
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

fn inputs() -> (Renderer, wgpu::Device, wgpu::Queue, Arc<DeviceSignal>) {
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let queue = ready.queue_for_test().clone();
    let signal = renderer.default_device_signal_for_test().unwrap();
    (renderer, device, queue, signal)
}

fn invalid_texture(device: &wgpu::Device, label: &'static str) {
    let _ = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
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
}

fn reach_gate<F: Future>(mut future: Pin<&mut F>, gate: &ScopePopGate, device: &wgpu::Device) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(matches!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        if gate.reached() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "scope gate was not reached before its diagnostic deadline"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

#[test]
fn canceled_scope_pop_closes_generation_resources_and_phase_without_publication() {
    for stage in [GpuOperationStage::Render, GpuOperationStage::Readback] {
        for position in [
            ScopePopPosition::Validation,
            ScopePopPosition::OutOfMemory,
            ScopePopPosition::Internal,
        ] {
            characterize_cancellation(stage, position, false);
        }
    }
}

#[test]
#[cfg(feature = "render-window")]
fn canceled_submission_scope_pop_closes_generation_resources_and_phase_without_publication() {
    for position in [
        ScopePopPosition::Validation,
        ScopePopPosition::OutOfMemory,
        ScopePopPosition::Internal,
    ] {
        characterize_cancellation(GpuOperationStage::Render, position, true);
    }
}

fn characterize_cancellation(
    stage: GpuOperationStage,
    position: ScopePopPosition,
    submission: bool,
) {
    let (mut renderer, device, queue, signal) = inputs();
    let stats = renderer.stats();
    let custody = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .generation_for_test()
        .clone();
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    let resources = ResourceManager::new(ResourceCacheBudget::DISABLED);
    let sentinel = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(position);
    let mut publication = Some(1);
    {
        let future = fixture.run(async |mut attempt| {
            attempt.attach_generation(&custody)?;
            let account = attempt.account().clone();
            let mut transaction =
                GpuOperationTransaction::begin(&device, Arc::clone(&signal), generation, stage);
            transaction.scopes.scope_gate = Some(Rc::clone(&gate));
            let mut frame = resources.begin_frame(Some(&account)).unwrap();
            let _resource = frame.acquire(ResourceCacheKey::VelloAtlas, 16).unwrap();
            frame.discard_on_drop();
            let _scratch = account.temporary::<u8>(5).unwrap();
            let draft = GpuOperationDraft::new(&mut publication, 2);
            let submit_phase = if stage == GpuOperationStage::Readback {
                RenderPhase::ReadbackSubmit
            } else {
                RenderPhase::Submit
            };
            account
                .phase(submit_phase, || {
                    custody
                        .opaque(
                            crate::accounting::GpuObjectKind::CommandEncoder,
                            Some(&account),
                            || {
                                device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor::default(),
                                )
                            },
                        )?
                        .finish(Some(&account))?
                        .submit(&queue, Some(&account.at(submit_phase)))
                        .map(|_| ())
                })
                .unwrap();
            #[cfg(feature = "render-window")]
            if submission {
                drop(
                    transaction
                        .resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account)
                        .await
                        .unwrap(),
                );
                draft.commit();
                return Ok(());
            }
            #[cfg(not(feature = "render-window"))]
            assert!(!submission);
            transaction
                .finish(RuntimeOperation::SurfaceRendering, Some(&account))
                .await
                .unwrap();
            draft.commit();
            Ok(())
        });
        let mut future = std::pin::pin!(future);
        reach_gate(future.as_mut(), &gate, &device);
        assert!(gate.reached());
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        assert_eq!(resources.observation_for_test().active_frame_count, 1);
    }
    assert_eq!(publication, Some(1));
    assert_eq!(renderer.stats(), stats);
    assert_eq!(signal.active_generation_for_test(), None);
    let resources = resources.observation_for_test();
    assert_eq!(resources.active_frame_count, 0);
    assert_eq!(resources.leased_count, 0);
    assert_eq!(resources.entry_count, 0);
    let report = fixture.report.take().unwrap();
    let fault = report.cleanup_fault();
    assert_eq!(fault, None);
    let phase = if stage == GpuOperationStage::Readback {
        RenderPhase::ReadbackResolve
    } else {
        RenderPhase::Resolve
    };
    let observed = report.attempt().unwrap();
    assert_eq!(observed.phase_count(phase), 1);
    assert!(observed.phase_elapsed(phase).is_some());
    assert_eq!(observed.usage().work(RenderWork::Submissions), 1);
    assert_eq!(
        report
            .renderer_cpu()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
    invalid_texture(&device, "Surgeist canceled scope sentinel");
    let captured = pollster::block_on(sentinel.pop())
        .expect("the outer scope must receive validation after cancellation");
    assert!(
        captured
            .to_string()
            .contains("Surgeist canceled scope sentinel")
    );
    let retry = GpuOperationTransaction::begin(
        &device,
        Arc::clone(&signal),
        signal.next_test_generation().unwrap(),
        stage,
    );
    pollster::block_on(retry.finish(RuntimeOperation::SurfaceRendering, None)).unwrap();
    assert_eq!(signal.active_generation_for_test(), None);
    assert!(signal.first_terminal().is_none());
}

#[test]
fn captured_validation_retains_source_and_associated_terminal_has_priority() {
    characterize_captured_and_terminal(false);
}

#[test]
#[cfg(feature = "render-window")]
fn submission_captured_validation_retains_source_and_associated_terminal_has_priority() {
    characterize_captured_and_terminal(true);
}

fn characterize_captured_and_terminal(submission: bool) {
    for terminal in [false, true] {
        let (_renderer, device, _queue, signal) = inputs();
        let generation = signal.next_test_generation().unwrap();
        let transaction = GpuOperationTransaction::begin(
            &device,
            Arc::clone(&signal),
            generation,
            GpuOperationStage::Render,
        );
        invalid_texture(&device, "Surgeist captured validation cause");
        let first = if terminal {
            signal.record_uncaptured_fault_for_test(
                GpuFaultKind::Internal,
                "associated terminal cause",
            );
            signal.first_terminal()
        } else {
            None
        };
        let error = pollster::block_on(async {
            #[cfg(feature = "render-window")]
            if submission {
                let account = Account::for_test();
                return transaction
                    .resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account)
                    .await
                    .map(|_| ());
            }
            #[cfg(not(feature = "render-window"))]
            assert!(!submission);
            transaction
                .finish(RuntimeOperation::SurfaceRendering, None)
                .await
        })
        .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        use std::error::Error as _;
        if let Some(first) = first {
            assert!(error.to_string().contains("associated terminal cause"));
            assert!(
                !error
                    .to_string()
                    .contains("Surgeist captured validation cause")
            );
            assert!(Arc::ptr_eq(&first, &signal.first_terminal().unwrap()));
            assert_eq!(first.operation_generation_for_test(), Some(generation));
        } else {
            assert!(
                error
                    .to_string()
                    .contains("Surgeist captured validation cause")
            );
            assert!(error.source().is_some());
            assert!(signal.first_terminal().is_none());
        }
        assert_eq!(signal.active_generation_for_test(), None);
    }
}

#[test]
fn explicit_scope_gate_release_allows_clean_finish() {
    let (_renderer, device, _queue, signal) = inputs();
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(ScopePopPosition::Validation);
    let mut transaction = GpuOperationTransaction::begin(
        &device,
        Arc::clone(&signal),
        generation,
        GpuOperationStage::Render,
    );
    transaction.scopes.scope_gate = Some(Rc::clone(&gate));
    let mut future = Box::pin(transaction.finish(RuntimeOperation::SurfaceRendering, None));
    reach_gate(future.as_mut(), &gate, &device);
    gate.release();
    pollster::block_on(future).unwrap();
    assert_eq!(signal.active_generation_for_test(), None);
}

#[cfg(feature = "render-window")]
thread_local! {
    static LEASE_CLEANUP_CLOCK: std::cell::RefCell<Option<Rc<std::cell::Cell<u64>>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(feature = "render-window")]
fn lease_cleanup_clock() -> Duration {
    LEASE_CLEANUP_CLOCK
        .with_borrow(|clock| Duration::from_millis(clock.as_ref().map_or(0, |clock| clock.get())))
}

#[cfg(feature = "render-window")]
struct LeaseCleanupClockReset;
#[cfg(feature = "render-window")]
impl Drop for LeaseCleanupClockReset {
    fn drop(&mut self) {
        LEASE_CLEANUP_CLOCK.with_borrow_mut(|clock| *clock = None);
    }
}

#[test]
#[cfg(feature = "render-window")]
fn canceled_submission_resolve_includes_actual_lease_cleanup_in_phase_elapsed() {
    let (_renderer, device, _queue, signal) = inputs();
    let time = Rc::new(std::cell::Cell::new(0));
    LEASE_CLEANUP_CLOCK.with_borrow_mut(|clock| *clock = Some(Rc::clone(&time)));
    let _reset = LeaseCleanupClockReset;
    let options = RenderAccountingOptions::new()
        .with_clock(crate::RenderClock::Host(lease_cleanup_clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(ScopePopPosition::Validation);
    {
        let future = fixture.run(async |attempt| {
            let account = attempt.account().clone();
            let mut transaction = GpuOperationTransaction::begin(
                &device,
                Arc::clone(&signal),
                generation,
                GpuOperationStage::Render,
            );
            transaction.scopes.scope_gate = Some(Rc::clone(&gate));
            // Seven fixture ticks are advanced only by this real lease's Drop.
            transaction.lease.cleanup_time = Some(Rc::clone(&time));
            transaction
                .resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account)
                .await
                .map(drop)
        });
        let mut future = std::pin::pin!(future);
        reach_gate(future.as_mut(), &gate, &device);
        time.set(5);
    }
    assert_eq!(signal.active_generation_for_test(), None);
    assert_eq!(
        time.get(),
        12,
        "the actual lease cleanup must have executed"
    );
    let report = fixture.report.take().unwrap();
    let fault = report.cleanup_fault();
    assert_eq!(fault, None);
    let observation = report.attempt().unwrap();
    assert_eq!(observation.elapsed(), Some(Duration::from_millis(12)));
    assert_eq!(observation.phase_count(RenderPhase::Resolve), 1);
    assert_eq!(
        observation.phase_elapsed(RenderPhase::Resolve),
        Some(Duration::from_millis(12)),
        "Resolve must include the five pending ticks and seven actual lease-cleanup ticks"
    );
}

#[test]
#[cfg(feature = "render-window")]
fn clean_submission_disposal_aborts_and_present_keeps_the_same_generation() {
    for dispose in [true, false] {
        let (_renderer, device, _queue, signal) = inputs();
        let generation = signal.next_test_generation().unwrap();
        let transaction = GpuOperationTransaction::begin(
            &device,
            Arc::clone(&signal),
            generation,
            GpuOperationStage::Render,
        );
        let account = Account::for_test();
        let clean = pollster::block_on(
            transaction.resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account),
        )
        .unwrap();
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        if dispose {
            drop(clean);
        } else {
            let present = clean
                .begin_present_phase(&device, RuntimeOperation::SurfaceRendering)
                .unwrap();
            assert_eq!(present.lease.generation(), generation);
            assert_eq!(signal.active_generation_for_test(), Some(generation));
            invalid_texture(&device, "Surgeist fresh Present validation scope");
            let error = pollster::block_on(
                present.finish(RuntimeOperation::SurfaceRendering, Some(&account)),
            )
            .unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::PresentFailed);
            assert!(
                error
                    .to_string()
                    .contains("Surgeist fresh Present validation scope")
            );
        }
        assert_eq!(signal.active_generation_for_test(), None);
        let retry = GpuOperationTransaction::begin(
            &device,
            Arc::clone(&signal),
            signal.next_test_generation().unwrap(),
            GpuOperationStage::Render,
        );
        pollster::block_on(retry.finish(RuntimeOperation::SurfaceRendering, None)).unwrap();
        assert!(signal.first_terminal().is_none());
    }
}

#[cfg(feature = "render-window")]
fn preserved_presented_pixel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> [u8; 4] {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Surgeist retained output verification"),
        size: 512,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
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
    let (sender, receiver) = std::sync::mpsc::channel();
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
        .unwrap();
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("retained-output map callback deadline")
        .unwrap();
    let pixel = buffer.slice(..).get_mapped_range()[..4].try_into().unwrap();
    buffer.unmap();
    pixel
}

#[test]
#[cfg(feature = "render-window")]
fn terminal_after_actual_graph_authorization_retains_prior_output_and_suppresses_present() {
    use crate::backend::{
        configured_display_free_presented_surface_for_test, presented_observation_handle_for_test,
        take_last_presented_texture_for_test,
    };
    use crate::{
        Color, EffectQualityPolicy, Parameters, Rect, RenderGenerationScope, RenderRoute, Scene,
    };
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            )
            .with_resource_cache_budget(ResourceCacheBudget::DISABLED),
    ))
    .unwrap();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let queue = ready.queue_for_test().clone();
    let custody = ready.generation_for_test().clone();
    let working_format = crate::resource::WorkingFormat::ReducedPrecision;
    let mut surface = configured_display_free_presented_surface_for_test(&mut renderer);
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    let parameters = Parameters::default();
    let first = pollster::block_on(renderer.render_forced_base_graph_for_test(
        &mut surface,
        &scene,
        parameters,
        working_format,
    ))
    .unwrap();
    assert_eq!(first.stats.route, Some(RenderRoute::GpuGraph));
    let before = renderer.stats();
    let parameters_before = surface.last_parameters;
    let observation = presented_observation_handle_for_test(&surface);
    let before_presentation = observation.snapshot_for_test();
    let mut replacement = Scene::new();
    replacement.fill(
        Rect::new(0.0, 0.0, 2.0, 2.0),
        Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
    );
    let probe = test_support::GraphAuthorizationProbeGuard::install(Arc::clone(&signal));
    let error = pollster::block_on(renderer.render_forced_base_graph_for_test(
        &mut surface,
        &replacement,
        parameters,
        working_format,
    ))
    .expect_err("the original-stage terminal recheck must reject after real authorization");
    assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
    assert!(
        error
            .to_string()
            .contains("terminal after actual graph authorization")
    );
    let generation = probe
        .generation()
        .expect("the real graph authorization boundary must execute");
    drop(probe);
    assert_eq!(
        signal
            .first_terminal()
            .unwrap()
            .operation_generation_for_test(),
        Some(generation)
    );
    assert_eq!(signal.active_generation_for_test(), None);
    let after = observation.snapshot_for_test();
    assert_eq!(
        after.acquire_count_for_test(),
        before_presentation.acquire_count_for_test() + 1
    );
    assert_eq!(
        after.present_count_for_test(),
        before_presentation.present_count_for_test()
    );
    assert_eq!(
        after.discarded_count_for_test(),
        before_presentation.discarded_count_for_test() + 1
    );
    assert_eq!(renderer.stats(), before);
    assert_eq!(surface.last_parameters, parameters_before);
    let report = renderer.last_attempt().unwrap();
    assert_eq!(
        report.status(),
        crate::RenderAttemptStatus::Failed(crate::ErrorCode::RenderFailed)
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Presentations),
        0
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        1
    );
    assert_eq!(
        custody
            .snapshot()
            .unwrap()
            .scope_usage(RenderGenerationScope::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    let retained = take_last_presented_texture_for_test(&mut surface)
        .expect("failed replacement must retain the prior presented output");
    assert_eq!(
        preserved_presented_pixel(&device, &queue, &retained),
        [0, 0, 0, 255]
    );
}

#[test]
#[cfg(feature = "render-window")]
fn graph_and_readback_profiles_preserve_work_custody_and_owning_phases() {
    use crate::backend::configured_display_free_presented_surface_for_test;
    use crate::{
        Color, EffectQualityPolicy, Parameters, Rect, RenderEvent, RenderGenerationScope,
        RenderPhaseFilter, Scene, Size,
    };
    let mut baseline = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 2048,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 128,
            filter: RenderPhaseFilter::Phase(RenderPhase::Present),
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
                .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
        ))
        .unwrap();
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
        let mut presented = configured_display_free_presented_surface_for_test(&mut renderer);
        pollster::block_on(renderer.render_forced_base_graph_for_test(
            &mut presented,
            &scene,
            Parameters::default(),
            crate::resource::WorkingFormat::ReducedPrecision,
        ))
        .unwrap();
        let report = renderer.take_last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        let usage = attempt.usage();
        assert_eq!(usage.work(RenderWork::Submissions), 1);
        assert_eq!(usage.work(RenderWork::Presentations), 1);
        let work = <RenderWork as surgeist_ledger::Dimension>::from_fn(|unit| usage.work(unit));
        let generation = report.generation().unwrap().unwrap();
        let custody = (
            generation.usage().peak(RenderResource::GpuBytes),
            generation.usage().live(RenderResource::GpuBytes),
            generation
                .scope_usage(RenderGenerationScope::PassBuffers)
                .live(RenderResource::GpuBytes),
        );
        assert_eq!(custody.2, 0);
        if let Some(expected) = &baseline {
            assert_eq!(&(work, custody), expected);
        } else {
            baseline = Some((work, custody));
        }
        let observed = profile != RenderProfile::Off;
        // One checked Vello capture encoding resolves its own scope, followed
        // by submission Resolve and final presentation-transaction Resolve.
        assert_eq!(
            attempt.phase_count(RenderPhase::Resolve),
            if observed { 3 } else { 0 }
        );
        assert_eq!(
            attempt.phase_count(RenderPhase::Present),
            u64::from(observed)
        );
        for phase in [RenderPhase::Resolve, RenderPhase::Present] {
            assert_eq!(attempt.phase_elapsed(phase).is_some(), observed);
        }
        if matches!(
            profile,
            RenderProfile::Detailed {
                event_limit: 128,
                ..
            }
        ) {
            let events: Vec<_> = attempt.events().collect();
            assert!(events.iter().any(|event| matches!(
                event,
                RenderEvent::Phase {
                    phase: RenderPhase::Present,
                    duration: Some(_),
                    ..
                }
            )));
            assert!(events.iter().all(|event| matches!(
                event,
                RenderEvent::Phase {
                    phase: RenderPhase::Present,
                    ..
                } | RenderEvent::Change {
                    phase: RenderPhase::Present,
                    ..
                } | RenderEvent::Refused {
                    phase: RenderPhase::Present,
                    ..
                }
            )));
        }
        if matches!(
            profile,
            RenderProfile::Off | RenderProfile::Detailed { event_limit: 0, .. }
        ) {
            assert_eq!(attempt.events().count(), 0);
        }
        let mut headless =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
        pollster::block_on(renderer.render(&mut headless, &scene, Parameters::default())).unwrap();
        assert_eq!(
            pollster::block_on(renderer.read_headless(&headless))
                .unwrap()
                .rgba(),
            &[0, 0, 0, 255].repeat(4)
        );
        let readback = renderer.last_readback_attempt().unwrap();
        let readback = readback.attempt().unwrap();
        assert_eq!(
            readback.phase_count(RenderPhase::ReadbackResolve),
            u64::from(observed)
        );
        assert_eq!(
            readback
                .phase_elapsed(RenderPhase::ReadbackResolve)
                .is_some(),
            observed
        );
        assert_eq!(readback.usage().work(RenderWork::Submissions), 1);
        assert_eq!(readback.usage().work(RenderWork::ReadbackCopies), 1);
    }
}
