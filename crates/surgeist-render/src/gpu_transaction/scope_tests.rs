//! Real scope ownership characterizations. No backend OOM/internal fault is induced.
#![cfg(not(target_arch = "wasm32"))]

use super::test_support::{ScopePopGate, ScopePopPosition};
use super::*;
use crate::{
    Options, Renderer, ResourceCacheBudget,
    accounting::{
        FrameAttempt, FrameReportSlot, RenderAccountingOptions, RenderAttemptStatus, RenderProfile,
        RenderResource, RenderWork, RendererCpu,
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

thread_local! {
    static CLOSURE_CLOCK: std::cell::RefCell<Option<(Arc<DeviceSignal>, Rc<std::cell::RefCell<Vec<Option<u64>>>>)>> = const { std::cell::RefCell::new(None) };
}

fn closure_clock() -> Duration {
    CLOSURE_CLOCK.with_borrow(|entry| {
        if let Some((signal, samples)) = entry {
            samples
                .borrow_mut()
                .push(signal.active_generation_for_test());
        }
    });
    Duration::ZERO
}

struct ClosureClockReset;
impl Drop for ClosureClockReset {
    fn drop(&mut self) {
        CLOSURE_CLOCK.with_borrow_mut(|entry| *entry = None);
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
    let options = RenderAccountingOptions::new()
        .with_clock(crate::RenderClock::Host(closure_clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let samples = Rc::new(std::cell::RefCell::new(Vec::new()));
    CLOSURE_CLOCK
        .with_borrow_mut(|entry| *entry = Some((Arc::clone(&signal), Rc::clone(&samples))));
    let _clock_reset = ClosureClockReset;
    let cpu = RendererCpu::new(options).unwrap();
    let mut attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
    attempt.attach_generation(&custody).unwrap();
    let account = attempt.account().clone();
    let resources = ResourceManager::new(ResourceCacheBudget::DISABLED);
    let sentinel = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(position);
    let mut publication = Some(1);
    {
        let future = async {
            let mut transaction =
                GpuOperationTransaction::begin(&device, Arc::clone(&signal), generation, stage);
            transaction.scope_gate = Some(Rc::clone(&gate));
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
                transaction
                    .resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account)
                    .await
                    .unwrap();
                draft.commit();
                return;
            }
            #[cfg(not(feature = "render-window"))]
            assert!(!submission);
            transaction
                .finish(RuntimeOperation::SurfaceRendering, Some(&account))
                .await
                .unwrap();
            draft.commit();
        };
        let mut future = std::pin::pin!(future);
        reach_gate(future.as_mut(), &gate, &device);
        assert!(gate.reached());
        assert_eq!(signal.active_generation_for_test(), Some(generation));
        assert_eq!(resources.observation_for_test().active_frame_count, 1);
        samples.borrow_mut().clear();
    }
    // The old borrowed submission resolver closes Resolve before its caller's
    // transaction drops. Consuming final finish owns cleanup inside its phase.
    assert_eq!(
        samples.borrow().first(),
        Some(&if submission { Some(generation) } else { None })
    );
    assert_eq!(publication, Some(1));
    assert_eq!(renderer.stats(), stats);
    assert_eq!(signal.active_generation_for_test(), None);
    let resources = resources.observation_for_test();
    assert_eq!(resources.active_frame_count, 0);
    assert_eq!(resources.leased_count, 0);
    assert_eq!(resources.entry_count, 0);
    let (report, fault) = attempt.finish(RenderAttemptStatus::Canceled, None);
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
        let mut transaction = GpuOperationTransaction::begin(
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
                    .await;
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
fn explicit_scope_gate_release_wakes_and_allows_clean_finish() {
    let (_renderer, device, _queue, signal) = inputs();
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(ScopePopPosition::Validation);
    let mut transaction = GpuOperationTransaction::begin(
        &device,
        Arc::clone(&signal),
        generation,
        GpuOperationStage::Render,
    );
    transaction.scope_gate = Some(Rc::clone(&gate));
    let mut future = Box::pin(transaction.finish(RuntimeOperation::SurfaceRendering, None));
    reach_gate(future.as_mut(), &gate, &device);
    gate.release();
    pollster::block_on(future).unwrap();
    assert_eq!(signal.active_generation_for_test(), None);
}

thread_local! {
    static LEASE_CLEANUP_CLOCK: std::cell::RefCell<Option<Rc<std::cell::Cell<u64>>>> = const { std::cell::RefCell::new(None) };
}

fn lease_cleanup_clock() -> Duration {
    LEASE_CLEANUP_CLOCK
        .with_borrow(|clock| Duration::from_millis(clock.as_ref().map_or(0, |clock| clock.get())))
}

struct LeaseCleanupClockReset;
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
    let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
    let account = attempt.account().clone();
    let generation = signal.next_test_generation().unwrap();
    let gate = ScopePopGate::new(ScopePopPosition::Validation);
    {
        let future = async {
            let mut transaction = GpuOperationTransaction::begin(
                &device,
                Arc::clone(&signal),
                generation,
                GpuOperationStage::Render,
            );
            transaction.scope_gate = Some(Rc::clone(&gate));
            // Seven fixture ticks are advanced only by this real lease's Drop.
            transaction.lease.cleanup_time = Some(Rc::clone(&time));
            transaction
                .resolve_submission_phase(RuntimeOperation::SurfaceRendering, &account)
                .await
        };
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
    let (report, fault) = attempt.finish(RenderAttemptStatus::Canceled, None);
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
