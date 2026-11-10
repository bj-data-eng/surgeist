use super::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::{Color, Options, Parameters, Rect, Renderer, Scene, Size};

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn basic_composition_profiles_preserve_work_pixels_and_optional_timing() {
    use crate::{BlendMode, Layer, LayerComposition, PorterDuffOperator, RenderRoute};

    let mut observed_work = None;
    let mut observed_pixels = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
        let mut scene = Scene::new();
        scene.layer(
            Layer::new().with_composition(
                LayerComposition::try_porter_duff(PorterDuffOperator::Clear, BlendMode::Normal)
                    .unwrap(),
            ),
            |_| {},
        );
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            &scene,
            Parameters {
                base_color: Color::BLACK,
                ..Parameters::default()
            },
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        assert_eq!(surface.headless_publication_count_for_test(), 1);
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        let attempt = report.attempt().unwrap();
        let usage = attempt.usage();
        assert!(usage.work(RenderWork::GraphNodeVisits) > 0);
        assert_eq!(usage.work(RenderWork::CompositePasses), 1);
        assert_eq!(usage.work(RenderWork::VelloPasses), 0);
        assert_eq!(stats.vello_passes, 0);
        assert_eq!(usage.work(RenderWork::Submissions), 1);
        let work = <RenderWork as surgeist_ledger::Dimension>::from_fn(|unit| usage.work(unit));
        if let Some(expected) = observed_work {
            assert_eq!(work, expected, "observation must not change admitted work");
        } else {
            observed_work = Some(work);
        }
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        assert_eq!(attempt.elapsed().is_some(), timed);
        if !matches!(profile, RenderProfile::Off) {
            assert!(attempt.phase_count(RenderPhase::Plan) > 0);
            assert!(attempt.phase_count(RenderPhase::Encode) > 0);
            assert_eq!(attempt.phase_elapsed(RenderPhase::Encode).is_some(), timed);
        }
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert!(
            pixels
                .rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 0)
        );
        if let Some(expected) = observed_pixels.as_ref() {
            assert_eq!(&pixels, expected);
        } else {
            observed_pixels = Some(pixels);
        }
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn refused_destructive_composition_preserves_publication_and_allows_fresh_retry() {
    use crate::{BlendMode, Layer, LayerComposition, PorterDuffOperator};

    // The baseline direct frame needs no closed-graph composition draw. Refuse
    // the first such draw of the destructive frame before queue submission.
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CompositePasses, 0)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    let mut baseline = Scene::new();
    baseline.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let publication = surface.headless_publication_count_for_test();
    let mut destructive = Scene::new();
    destructive.layer(
        Layer::new().with_composition(
            LayerComposition::try_porter_duff(PorterDuffOperator::Clear, BlendMode::Normal)
                .unwrap(),
        ),
        |_| {},
    );
    let error = pollster::block_on(renderer.render(
        &mut surface,
        &destructive,
        Parameters {
            base_color: Color::BLACK,
            ..Parameters::default()
        },
    ))
    .unwrap_err();
    assert!(
        matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(refusal))
            if refusal.quantity() == RenderQuantity::Work(RenderWork::CompositePasses)
        ),
        "the frame must reach its composition-draw admission boundary: {error:?}"
    );
    assert_eq!(renderer.stats(), successful);
    assert_eq!(surface.headless_publication_count_for_test(), publication);
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::CompositePasses), 0);
    assert_eq!(usage.work(RenderWork::Submissions), 0);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default())).unwrap();
    assert_eq!(
        surface.headless_publication_count_for_test(),
        publication + 1
    );
    assert_eq!(
        renderer.last_attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn multiple_borrowed_report_readers_preserve_last_success_and_fresh_retry_budget() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CommandsEmitted, 2)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    let mut one = Scene::new();
    one.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    assert!(renderer.last_attempt().is_none());
    drop(renderer.render(&mut surface, &one, Parameters::default()));
    assert!(
        renderer.last_attempt().is_none(),
        "an unpolled future starts nothing"
    );
    let successful =
        pollster::block_on(renderer.render(&mut surface, &one, Parameters::default())).unwrap();
    {
        let first = renderer.last_attempt().unwrap();
        let second = renderer.last_attempt().unwrap();
        assert_eq!(first.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(second.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(
            first
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CommandsEmitted),
            1
        );
        assert!(first.renderer_cpu().is_ok());
        assert!(first.generation().unwrap().is_ok());
    }
    let mut three = Scene::new();
    for _ in 0..3 {
        three.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    }
    let error = pollster::block_on(renderer.render(&mut surface, &three, Parameters::default()))
        .unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(_))
    ));
    assert_eq!(renderer.stats(), successful);
    let refused = renderer.take_last_attempt().unwrap();
    assert_eq!(refused.status(), RenderAttemptStatus::Refused);
    assert_eq!(
        refused
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandsEmitted),
        2
    );
    assert!(renderer.last_attempt().is_none());
    let mut two = Scene::new();
    for _ in 0..2 {
        two.fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
    }
    let retry =
        pollster::block_on(renderer.render(&mut surface, &two, Parameters::default())).unwrap();
    assert_eq!(retry.commands, 2);
    assert_eq!(retry.frame_time, None);
    assert_eq!(retry.preparation_time, None);
    assert_eq!(retry.execution_time, None);
    assert_eq!(retry.presentation_time, None);
    assert_eq!(retry.surface_acquisition_time, None);
}

#[test]
fn polled_cancellation_closes_all_domains_and_reopens_the_same_lifetime_owners() {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    let options = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(RenderWork::CommandsEmitted, 2)
            .unwrap(),
    );
    let cpu = RendererCpu::new(options).unwrap();
    let generation = Generation::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    {
        let future = fixture.run(async |mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let scratch = account.temporary::<u8>(3).unwrap();
            let object = generation
                .opaque(GpuObjectKind::Sampler, Some(&account), || ())
                .unwrap();
            account.work(RenderWork::CommandsEmitted, 1).unwrap();
            std::future::pending::<()>().await;
            drop(object);
            drop(scratch);
            Ok(())
        });
        let mut future = std::pin::pin!(future);
        assert!(matches!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        // The lifecycle has not reached its terminal publication.
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            3
        );
        assert_eq!(
            generation
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::OpaqueGpuObjects),
            1
        );
    }
    let canceled = fixture.report.take().unwrap();
    assert_eq!(canceled.status(), RenderAttemptStatus::Canceled);
    assert_eq!(
        canceled
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandsEmitted),
        1
    );
    let cpu_interval = canceled.renderer_cpu().unwrap().usage();
    assert_eq!(cpu_interval.peak(RenderResource::ScratchCpuBytes), 3);
    assert_eq!(cpu_interval.live(RenderResource::ScratchCpuBytes), 0);
    let native = canceled.generation().unwrap().unwrap().usage();
    assert_eq!(native.peak(RenderResource::OpaqueGpuObjects), 1);
    assert_eq!(native.live(RenderResource::OpaqueGpuObjects), 0);
    assert_eq!(native.work(RenderWork::GpuObjectsCreated), 1);
    // Forward requests have one numeric owner (the attempt); the generation
    // observes returned handles and mandatory lifetime effects separately.
    assert_eq!(native.work(RenderWork::ResourceAllocationRequests), 0);
    assert_eq!(
        canceled
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ResourceAllocationRequests),
        1
    );
    let (result, _, retry, fault) = fixture
        .run_sync(|mut retry| {
            retry.attach_generation(&generation)?;
            retry.account().work(RenderWork::CommandsEmitted, 2)?;
            Ok(())
        })
        .unwrap()
        .into_parts();
    result.unwrap();
    assert_eq!(fault, None);
    assert_eq!(retry.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(
        retry
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandsEmitted),
        2
    );
    assert_eq!(
        retry
            .renderer_cpu()
            .unwrap()
            .usage()
            .peak(RenderResource::ScratchCpuBytes),
        0
    );
    assert_eq!(
        retry
            .generation()
            .unwrap()
            .unwrap()
            .usage()
            .peak(RenderResource::OpaqueGpuObjects),
        0
    );
}

#[test]
fn missing_interval_is_typed_unavailability_and_preserves_other_actual_boundaries() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let generation = Generation::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    let (_, _, report, fault) = fixture
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            attempt
                .account()
                .work(RenderWork::CommandsEmitted, 1)
                .unwrap();
            // Faithful private-owner invariant injection: close the real CPU interval,
            // then remove its token. Extraction must not fabricate a zero interval.
            drop(attempt.core.cpu_checkpoint.take());
            Ok(())
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, Some(RenderAccountingError::Invariant));
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
    );
    assert_eq!(report.renderer_cpu(), Err(RenderAccountingError::Invariant));
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CommandsEmitted),
        1
    );
    assert!(report.generation().unwrap().is_ok());
}

#[test]
fn mandatory_finalization_fault_does_not_replace_an_earlier_backend_cause() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let first = RenderAccountingError::CpuRefused {
        scope: RenderCpuScope::FrameBacking,
        quantity: RenderQuantity::Resource(RenderResource::ScratchCpuBytes),
        current: 0,
        amount: 1,
        reason: RenderRefusalReason::Underflow,
    };
    // Inject the same fixed diagnostic retained by a committed CPU transition
    // fault, independently of the prior causal backend failure.
    let status = RenderAttemptStatus::Failed(crate::ErrorCode::RenderFailed);
    let (_, _, mut report, fault) = FrameFixture::new(&cpu, options)
        .run_sync(|_| {
            cpu.at(RenderCpuScope::FrameBacking).note_fault(first);
            Err::<(), _>(Error::new(
                crate::BackendErrorCode::RenderFailed,
                "original backend failure",
            ))
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, Some(first));
    report.note_cleanup_fault(Some(RenderAccountingError::Invariant));
    assert_eq!(report.status(), status);
    assert_eq!(report.attempt().unwrap().status(), status);
    assert_eq!(report.cleanup_fault(), Some(first));
    assert_eq!(report.accounting_diagnostic(), Some(first));
}

#[test]
fn rejected_frame_setup_preserves_previous_report_and_cpu_lifetime_usage() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    let (_, _, prior, fault) = fixture
        .run_sync(|attempt| {
            attempt.account().work(RenderWork::CommandsEmitted, 1)?;
            Ok(())
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, None);
    fixture.report = Some(prior.snapshot_for_test());
    let before = cpu.snapshot().unwrap().usage();
    let invalid = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(RenderWork::AuthoredCommands, 0)
            .unwrap(),
    );
    let unavailable = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
        timing: false,
        event_limit: usize::MAX,
        filter: RenderPhaseFilter::All,
    });
    for (configuration, diagnostic) in [
        (
            invalid,
            RenderAccountingError::InvalidLimit(RenderWork::AuthoredCommands),
        ),
        // Event elements occupy multiple bytes: this capacity is unrepresentable
        // before allocator invocation, so no actual huge allocation is attempted.
        (
            unavailable,
            RenderAccountingError::Setup(RenderAccountingSetupError::Allocation),
        ),
    ] {
        fixture.options = configuration;
        let error = match fixture.run_sync(|_| Ok(())) {
            Err(error) => error,
            Ok(_) => panic!("invalid frame setup unexpectedly started"),
        };
        assert_eq!(error.accounting_diagnostic(), Some(diagnostic));
        assert_eq!(fixture.report.as_ref().unwrap(), &prior);
        assert_eq!(cpu.snapshot().unwrap().usage(), before);
        assert_eq!(cpu.fault(), None);
    }
}

#[test]
fn consumed_phase_slot_stays_inert_after_owned_cleanup() {
    use std::{
        future::Future,
        sync::{
            Arc,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        },
        task::{Context, Poll, Wake, Waker},
        time::Duration,
    };
    // These clocks belong only to this test, including its payload destructor.
    static TIME: AtomicU64 = AtomicU64::new(0);
    static CLOCK_READS: AtomicUsize = AtomicUsize::new(0);
    fn clock() -> Duration {
        CLOCK_READS.fetch_add(1, Ordering::SeqCst);
        Duration::from_millis(TIME.load(Ordering::SeqCst))
    }
    struct CleanupValue(Arc<AtomicUsize>);
    impl Drop for CleanupValue {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
            TIME.fetch_add(7, Ordering::SeqCst);
        }
    }
    #[derive(Default)]
    struct WakeCount(AtomicUsize);
    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let mut fixture = FrameFixture::new(&cpu, options);
    let drops = Arc::new(AtomicUsize::new(0));
    let polls = AtomicUsize::new(0);
    let wakes = Arc::new(WakeCount::default());
    let waker = Waker::from(wakes.clone());
    let finished = pollster::block_on(fixture.run(async |attempt| {
        let account = attempt.account();
        let inner_account = account.clone();
        let cleanup_count = drops.clone();
        // The real payload and its accounted backing live across suspension.
        let mut slot = std::pin::pin!(Some(async {
            let mut values = inner_account.temporary::<CleanupValue>(1)?;
            values.push(CleanupValue(cleanup_count))?;
            inner_account.work(RenderWork::CommandsEmitted, 1)?;
            TIME.store(5, Ordering::SeqCst);
            std::future::poll_fn(|_| {
                polls.fetch_add(1, Ordering::SeqCst);
                Poll::<()>::Pending
            })
            .await;
            drop(values);
            Ok::<_, Error>(())
        }));
        account
            .ledger
            .with_async_phase(
                account.scopes[RenderPhase::FramePrepare.index()],
                async || {
                    let mut context = Context::from_waker(&waker);
                    let baseline = cpu.snapshot().unwrap().usage();
                    let mut present = PhaseFuture {
                        future: slot.as_mut(),
                    };
                    assert!(
                        std::pin::Pin::new(&mut present)
                            .poll(&mut context)
                            .is_pending()
                    );
                    assert_eq!(polls.load(Ordering::SeqCst), 1);
                    assert_eq!(account.work_count(RenderWork::CommandsEmitted)?, 1);
                    assert_eq!(drops.load(Ordering::SeqCst), 0);
                    assert_eq!(
                        cpu.snapshot()
                            .unwrap()
                            .usage()
                            .live(RenderResource::ScratchCpuBytes),
                        baseline.live(RenderResource::ScratchCpuBytes)
                            + std::mem::size_of::<CleanupValue>() as u64
                    );
                    drop(present);
                    assert!(slot.as_ref().get_ref().is_none());
                    assert_eq!(drops.load(Ordering::SeqCst), 1);
                    assert_eq!(TIME.load(Ordering::SeqCst), 12);
                    let cleaned = cpu.snapshot().unwrap().usage();
                    assert_eq!(
                        cleaned.live(RenderResource::ScratchCpuBytes),
                        baseline.live(RenderResource::ScratchCpuBytes)
                    );
                    let work = <RenderWork as surgeist_ledger::Dimension>::from_fn(|unit| {
                        account.work_count(unit).unwrap()
                    });
                    let reads = CLOCK_READS.load(Ordering::SeqCst);
                    // This is the SAME slot after actual guard-owned destruction.
                    let mut consumed = PhaseFuture {
                        future: slot.as_mut(),
                    };
                    for _ in 0..2 {
                        assert!(
                            std::pin::Pin::new(&mut consumed)
                                .poll(&mut context)
                                .is_pending()
                        );
                    }
                    drop(consumed);
                    assert_eq!(CLOCK_READS.load(Ordering::SeqCst), reads);
                    assert_eq!(wakes.0.load(Ordering::SeqCst), 0);
                    assert_eq!(polls.load(Ordering::SeqCst), 1);
                    assert_eq!(drops.load(Ordering::SeqCst), 1);
                    assert_eq!(TIME.load(Ordering::SeqCst), 12);
                    assert_eq!(cpu.snapshot().unwrap().usage(), cleaned);
                    assert_eq!(
                        <RenderWork as surgeist_ledger::Dimension>::from_fn(|unit| {
                            account.work_count(unit).unwrap()
                        }),
                        work
                    );
                    assert_eq!(account.latched_fault(), None);
                    Ok::<_, Error>(())
                },
            )
            .await
            .map_err(|error| account.error(error))?
    }))
    .unwrap();
    let (result, _, report, fault) = finished.into_parts();
    result.unwrap();
    assert_eq!(fault, None);
    assert_eq!(cpu.fault(), None);
    assert_eq!(report.cleanup_fault(), None);
    assert_eq!(report.accounting_diagnostic(), None);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    let attempt = report.attempt().unwrap();
    assert_eq!(attempt.usage().work(RenderWork::CommandsEmitted), 1);
    assert_eq!(attempt.phase_count(RenderPhase::FramePrepare), 1);
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::FramePrepare),
        Some(Duration::from_millis(12))
    );
    assert_eq!(attempt.elapsed(), Some(Duration::from_millis(12)));
    let cpu_interval = report.renderer_cpu().unwrap();
    assert_eq!(
        cpu_interval.usage().live(RenderResource::ScratchCpuBytes),
        0
    );
    assert_eq!(
        cpu_interval.scope_elapsed(RenderCpuScope::FrameBacking),
        Some(Duration::from_millis(7))
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

static PHASE_CLEANUP_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn phase_cleanup_clock() -> std::time::Duration {
    std::time::Duration::from_millis(PHASE_CLEANUP_TIME.load(std::sync::atomic::Ordering::SeqCst))
}
#[test]
fn pending_phase_cancellation_includes_its_owned_future_cleanup_before_attempt_closure() {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
        time::Duration,
    };
    struct CleanupValue {
        _data: u8,
    }
    impl Drop for CleanupValue {
        fn drop(&mut self) {
            PHASE_CLEANUP_TIME.fetch_add(7, std::sync::atomic::Ordering::SeqCst);
        }
    }
    // Each owner is dropped before resetting the isolated clock for the next case.
    for canceled in [true, false] {
        PHASE_CLEANUP_TIME.store(0, std::sync::atomic::Ordering::SeqCst);
        let options = RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(phase_cleanup_clock))
            .with_profile(RenderProfile::Summary { timing: true });
        let cpu = RendererCpu::new(options).unwrap();
        let mut fixture = FrameFixture::new(&cpu, options);
        let mut completed = None;
        {
            let future = fixture.run(async |attempt| {
                let account = attempt.account().clone();
                let inner_account = account.clone();
                account
                    .async_phase(RenderPhase::FramePrepare, async move {
                        let mut values = inner_account.temporary::<CleanupValue>(1)?;
                        values.push(CleanupValue { _data: 0 })?;
                        PHASE_CLEANUP_TIME.store(5, std::sync::atomic::Ordering::SeqCst);
                        if canceled {
                            std::future::pending::<()>().await;
                        }
                        drop(values);
                        Ok(())
                    })
                    .await
            });
            let mut future = std::pin::pin!(future);
            match future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
            {
                Poll::Pending => assert!(canceled),
                Poll::Ready(result) => {
                    assert!(!canceled);
                    completed = Some(result.unwrap());
                }
            }
        }
        let (report, fault) = if let Some(finished) = completed {
            let (result, _, report, fault) = finished.into_parts();
            result.unwrap();
            (report, fault)
        } else {
            (fixture.report.take().unwrap(), None)
        };
        assert_eq!(fault, None);
        assert_eq!(
            report.attempt().unwrap().elapsed(),
            Some(Duration::from_millis(12))
        );
        assert_eq!(
            report
                .renderer_cpu()
                .unwrap()
                .scope_elapsed(RenderCpuScope::FrameBacking),
            Some(Duration::from_millis(7))
        );
        assert_eq!(
            report
                .renderer_cpu()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .phase_elapsed(RenderPhase::FramePrepare),
            Some(Duration::from_millis(12))
        );
    }
}
