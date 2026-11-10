//! Characterizations of the real acquire owners and their pending scope finish.
use super::*;
use crate::{
    Antialiasing, Capabilities, Color, EffectQualityPolicy, ErrorCode, Options, Rect, Renderer,
    ResourceCacheBudget, Scene, Size,
    accounting::{
        FrameFixture, RenderAccountingOptions, RenderCpuScope, RenderLimits, RenderProfile,
        RendererCpu,
    },
    backend::{
        DeviceCapabilities, DeviceSignal, configured_display_free_presented_surface_for_test,
        presented_lifecycle_for_test, presented_observation_handle_for_test,
        set_presented_acquire_outcome_for_test,
    },
    frame::{FrameContext, forced_base_graph_for_test},
    gpu_transaction::test_support::{
        LeaseDropObserverGuardForTest, ScopePopGate, ScopePopPosition,
    },
    pass::LoweredGraphPlan,
    resource::ResourceManager,
    shader::DevicePassCache,
    surface::PresentedAcquireOutcomeForTest,
};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

struct AcquireFixture {
    renderer: Renderer,
    device: wgpu::Device,
    queue: wgpu::Queue,
    signal: Arc<DeviceSignal>,
    capabilities: DeviceCapabilities,
    resources: Rc<ResourceManager>,
    pass_cache: DevicePassCache,
}

impl AcquireFixture {
    fn new() -> Self {
        let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let device = ready.device_for_test().clone();
        let queue = ready.queue_for_test().clone();
        let capabilities = DeviceCapabilities::from_device(ready.adapter_for_test(), &device)
            .for_selected_working_format(WorkingFormat::ReducedPrecision)
            .unwrap();
        let generation = ready.generation_for_test().clone();
        let cpu = RendererCpu::new(RenderAccountingOptions::default()).unwrap();
        let resources = Rc::new(ResourceManager::with_generation(
            ResourceCacheBudget::DISABLED,
            generation.clone(),
            cpu.at(RenderCpuScope::DeviceBacking),
        ));
        let pass_cache = DevicePassCache::new(generation, cpu.at(RenderCpuScope::DeviceBacking));
        let signal = renderer.default_device_signal_for_test().unwrap();
        Self {
            renderer,
            device,
            queue,
            signal,
            capabilities,
            resources,
            pass_cache,
        }
    }

    fn surface(&mut self, outcome: PresentedAcquireOutcomeForTest) -> Surface {
        let mut surface = configured_display_free_presented_surface_for_test(&mut self.renderer);
        set_presented_acquire_outcome_for_test(&mut surface, outcome);
        // Enter the same valid eligibility boundary as the real exact caller.
        exact_presented_graph_target(
            &surface,
            WorkingFormat::ReducedPrecision,
            Format::Rgba8,
            Some(PhysicalSize::new(2, 2)),
        )
        .unwrap();
        surface
    }

    fn transaction(&self, stage: GpuOperationStage) -> (u64, GpuOperationTransaction) {
        let generation = self.signal.next_test_generation().unwrap();
        (
            generation,
            GpuOperationTransaction::begin(
                &self.device,
                Arc::clone(&self.signal),
                generation,
                stage,
            ),
        )
    }

    fn prepare<'a>(&'a self, account: &Account) -> PreparedGraph<'a> {
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
        let commands = scene.normalize(Capabilities::CURRENT, account).unwrap();
        let context = FrameContext::try_new(
            Size::new(2.0, 2.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let graph = forced_base_graph_for_test(commands, context).unwrap();
        let lowered = LoweredGraphPlan::try_lower_validated_graph(
            &graph,
            WorkingFormat::ReducedPrecision,
            Format::Rgba8,
            &self.capabilities,
            account,
        )
        .unwrap();
        let prepared = PreparedGraph::try_prepare(
            lowered,
            EffectQualityPolicy::AllowReducedPrecision,
            &self.capabilities,
            (&self.device, &self.queue),
            &self.resources,
            (&self.pass_cache, true),
            account,
        )
        .unwrap();
        let observation = self.resources.observation_for_test();
        assert_eq!(observation.active_frame_count, 1);
        assert!(
            observation.leased_count > 0,
            "the real graph must own prepared resource leases"
        );
        prepared
    }

    fn assert_resources_closed(&self) {
        let observation = self.resources.observation_for_test();
        assert_eq!(observation.active_frame_count, 0);
        assert_eq!(observation.leased_count, 0);
        assert_eq!(observation.entry_count, 0);
        assert_eq!(observation.accounting_fault_for_test(), None);
    }

    fn assert_fresh_retry(&self) {
        let (_, transaction) = self.transaction(GpuOperationStage::Render);
        pollster::block_on(transaction.finish(RuntimeOperation::SurfaceRendering, None)).unwrap();
        assert_eq!(self.signal.active_generation_for_test(), None);
        assert!(self.signal.first_terminal().is_none());
    }
}

fn reach_gate<F: Future + ?Sized>(
    mut future: Pin<&mut F>,
    gate: &ScopePopGate,
    device: &wgpu::Device,
) {
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
            "actual acquire finish did not reach its scope-pop gate"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
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

#[test]
fn unpolled_exact_acquire_drops_prepared_before_transaction_arguments() {
    let mut fixture = AcquireFixture::new();
    let mut surface = fixture.surface(PresentedAcquireOutcomeForTest::Success);
    let observation = presented_observation_handle_for_test(&surface);
    let account = Account::for_test();
    let (generation, transaction) = fixture.transaction(GpuOperationStage::Render);
    let prepared = fixture.prepare(&account);
    let at_drop = Rc::new(RefCell::new(Vec::new()));
    let recorded = Rc::clone(&at_drop);
    let resources = Rc::clone(&fixture.resources);
    let observer = LeaseDropObserverGuardForTest::install(
        Arc::clone(&fixture.signal),
        generation,
        move || recorded.borrow_mut().push(resources.observation_for_test()),
    );
    drop(acquire_exact_presented_graph_texture(
        &mut surface,
        &fixture.device,
        prepared,
        transaction,
        &account,
    ));
    let recorded = at_drop.borrow();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].active_frame_count, 0);
    assert_eq!(recorded[0].leased_count, 0);
    assert_eq!(
        observation
            .snapshot_for_test()
            .acquire_attempt_count_for_test(),
        0
    );
    fixture.assert_resources_closed();
    drop(recorded);
    drop(observer);
    fixture.assert_fresh_retry();
}

#[test]
fn refused_exact_acquire_drops_transaction_before_initialized_prepared_local() {
    let mut fixture = AcquireFixture::new();
    let mut surface = fixture.surface(PresentedAcquireOutcomeForTest::Success);
    let observation = presented_observation_handle_for_test(&surface);
    let options = RenderAccountingOptions::new()
        .with_profile(RenderProfile::Summary { timing: false })
        .with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::SurfaceAcquisitions, 0)
                .unwrap(),
        );
    let cpu = RendererCpu::new(options).unwrap();
    let mut frame = FrameFixture::new(&cpu, options);
    let at_drop = Rc::new(RefCell::new(Vec::new()));
    let finished = frame
        .run_sync(|mut attempt| {
            attempt.attach_generation(fixture.resources.generation())?;
            let account = attempt.account();
            let (generation, transaction) = fixture.transaction(GpuOperationStage::Render);
            let prepared = fixture.prepare(account);
            let recorded = Rc::clone(&at_drop);
            let resources = Rc::clone(&fixture.resources);
            let _observer = LeaseDropObserverGuardForTest::install(
                Arc::clone(&fixture.signal),
                generation,
                move || recorded.borrow_mut().push(resources.observation_for_test()),
            );
            pollster::block_on(acquire_exact_presented_graph_texture(
                &mut surface,
                &fixture.device,
                prepared,
                transaction,
                account,
            ))
            .map(drop)
        })
        .unwrap();
    let (result, _, report, _) = finished.into_parts();
    let error = result.unwrap_err();
    let refusal = report.attempt().unwrap().refusal().unwrap();
    assert_eq!(
        error.accounting_diagnostic(),
        report.accounting_diagnostic()
    );
    // The configured cap belongs to the Attempt ancestor; Acquire remains
    // the operation's observed phase, not the refusing Ledger scope.
    assert_eq!(refusal.phase(), RenderPhase::Attempt);
    assert_eq!(
        refusal.quantity(),
        crate::RenderQuantity::Work(RenderWork::SurfaceAcquisitions)
    );
    assert_eq!(
        refusal.reason(),
        crate::RenderRefusalReason::Limit { limit: 0 }
    );
    let recorded = at_drop.borrow();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].active_frame_count, 1);
    assert!(
        recorded[0].leased_count > 0,
        "the actual lease drops while prepared resources remain live"
    );
    assert_eq!(
        observation
            .snapshot_for_test()
            .acquire_attempt_count_for_test(),
        0
    );
    assert_eq!(
        report.attempt().unwrap().phase_count(RenderPhase::Acquire),
        1
    );
    assert_eq!(
        report.attempt().unwrap().phase_count(RenderPhase::Resolve),
        0
    );
    fixture.assert_resources_closed();
    fixture.assert_fresh_retry();
}

#[test]
fn canceled_acquire_finish_discards_resources_without_advancing_lifecycle_at_each_real_pop() {
    let mut fixture = AcquireFixture::new();
    for exact in [true, false] {
        for outcome in [
            PresentedAcquireOutcomeForTest::Suboptimal,
            PresentedAcquireOutcomeForTest::Outdated,
            PresentedAcquireOutcomeForTest::Occluded,
            PresentedAcquireOutcomeForTest::Timeout,
            PresentedAcquireOutcomeForTest::Lost,
            PresentedAcquireOutcomeForTest::Validation,
        ] {
            for position in [
                ScopePopPosition::Validation,
                ScopePopPosition::OutOfMemory,
                ScopePopPosition::Internal,
            ] {
                let mut surface = fixture.surface(outcome);
                let before = presented_lifecycle_for_test(&surface);
                let observation = presented_observation_handle_for_test(&surface);
                let options = RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: true });
                let cpu = RendererCpu::new(options).unwrap();
                let mut frame = FrameFixture::new(&cpu, options);
                let sentinel = fixture
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation);
                let stage = if exact {
                    GpuOperationStage::Render
                } else {
                    GpuOperationStage::Present
                };
                let (generation, transaction) = fixture.transaction(stage);
                let gate = ScopePopGate::new(position);
                let transaction = transaction.with_scope_pop_gate_for_test(Rc::clone(&gate));
                {
                    let mut future = Box::pin(frame.run(async |mut attempt| {
                        attempt.attach_generation(fixture.resources.generation())?;
                        let account = attempt.account();
                        if exact {
                            let prepared = fixture.prepare(account);
                            acquire_exact_presented_graph_texture(
                                &mut surface,
                                &fixture.device,
                                prepared,
                                transaction,
                                account,
                            )
                            .await
                            .map(drop)
                        } else {
                            let SurfaceBackend::Presented {
                                surface: native,
                                state,
                                ..
                            } = &mut surface.backend
                            else {
                                panic!("configured presented fixture")
                            };
                            acquire_internal_vello_surface_texture(
                                native,
                                &fixture.device,
                                state,
                                transaction,
                                account,
                            )
                            .await
                            .map(drop)
                        }
                    }));
                    reach_gate(future.as_mut(), &gate, &fixture.device);
                    assert_eq!(
                        fixture.signal.active_generation_for_test(),
                        Some(generation)
                    );
                    let observed = observation.snapshot_for_test();
                    assert_eq!(observed.acquire_attempt_count_for_test(), 1);
                    assert_eq!(observed.present_count_for_test(), 0);
                    assert_eq!(
                        observed.discarded_count_for_test(),
                        usize::from(outcome == PresentedAcquireOutcomeForTest::Suboptimal)
                    );
                    fixture.assert_resources_closed();
                    // Drop at the actual native pop, without ever releasing its gate.
                }
                assert_eq!(
                    presented_lifecycle_for_test(&surface),
                    before,
                    "canceling pending finish must not apply the deferred lifecycle transition"
                );
                assert_eq!(fixture.signal.active_generation_for_test(), None);
                let report = frame.report.take().unwrap();
                assert_eq!(report.cleanup_fault(), None);
                let attempt = report.attempt().unwrap();
                assert_eq!(attempt.phase_count(RenderPhase::Acquire), 1);
                assert_eq!(attempt.phase_count(RenderPhase::Resolve), 1);
                assert!(attempt.phase_elapsed(RenderPhase::Resolve).is_some());
                assert_eq!(attempt.usage().work(RenderWork::SurfaceAcquisitions), 1);
                assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
                assert_eq!(attempt.usage().work(RenderWork::Presentations), 0);
                invalid_texture(
                    &fixture.device,
                    "Surgeist acquire cancellation outer sentinel",
                );
                let captured = pollster::block_on(sentinel.pop()).unwrap();
                assert!(
                    captured
                        .to_string()
                        .contains("Surgeist acquire cancellation outer sentinel")
                );
                fixture.assert_fresh_retry();
            }
        }
    }
}

#[test]
fn acquire_scope_failure_keeps_first_cause_and_applies_lifecycle_only_after_finish() {
    let mut fixture = AcquireFixture::new();
    for exact in [true, false] {
        for terminal in [false, true] {
            let mut surface = fixture.surface(PresentedAcquireOutcomeForTest::Outdated);
            let account = Account::for_test();
            let stage = if exact {
                GpuOperationStage::Render
            } else {
                GpuOperationStage::Present
            };
            let (generation, transaction) = fixture.transaction(stage);
            let gate = ScopePopGate::new(ScopePopPosition::Validation);
            let transaction = transaction.with_scope_pop_gate_for_test(Rc::clone(&gate));
            let prepared = exact.then(|| fixture.prepare(&account));
            invalid_texture(
                &fixture.device,
                "Surgeist actual acquire first captured validation",
            );
            let first = if terminal {
                fixture.signal.record_uncaptured_fault_for_test(
                    crate::GpuFaultKind::Internal,
                    "actual acquire first terminal fault",
                );
                let first = fixture.signal.first_terminal().unwrap();
                fixture.signal.record_uncaptured_fault_for_test(
                    crate::GpuFaultKind::OutOfMemory,
                    "later acquire terminal fault",
                );
                Some(first)
            } else {
                None
            };
            let error = {
                let mut future: Pin<Box<dyn Future<Output = Result<()>> + '_>> =
                    if let Some(prepared) = prepared {
                        Box::pin(async {
                            acquire_exact_presented_graph_texture(
                                &mut surface,
                                &fixture.device,
                                prepared,
                                transaction,
                                &account,
                            )
                            .await
                            .map(drop)
                        })
                    } else {
                        let SurfaceBackend::Presented {
                            surface: native,
                            state,
                            ..
                        } = &mut surface.backend
                        else {
                            panic!("configured presented fixture")
                        };
                        Box::pin(async {
                            acquire_internal_vello_surface_texture(
                                native,
                                &fixture.device,
                                state,
                                transaction,
                                &account,
                            )
                            .await
                            .map(drop)
                        })
                    };
                reach_gate(future.as_mut(), &gate, &fixture.device);
                fixture.assert_resources_closed();
                gate.release();
                pollster::block_on(future).unwrap_err()
            };
            assert_eq!(
                error.code(),
                if exact {
                    ErrorCode::RenderFailed
                } else {
                    ErrorCode::PresentFailed
                }
            );
            assert!(matches!(
                presented_lifecycle_for_test(&surface),
                PresentedLifecycle::ResizePending { .. }
            ));
            if let Some(first) = first {
                assert!(
                    error
                        .to_string()
                        .contains("actual acquire first terminal fault")
                );
                assert!(Arc::ptr_eq(
                    &first,
                    &fixture.signal.first_terminal().unwrap()
                ));
                assert_eq!(first.operation_generation_for_test(), Some(generation));
                // A terminal device intentionally cannot provide a fresh retry.
                fixture = AcquireFixture::new();
            } else {
                use std::error::Error as _;
                assert!(
                    error
                        .to_string()
                        .contains("Surgeist actual acquire first captured validation")
                );
                assert!(error.source().is_some());
                fixture.assert_fresh_retry();
            }
        }
    }
}
