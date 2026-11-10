#![cfg(not(target_arch = "wasm32"))]

use crate::*;
use std::time::Duration;

async fn published(options: RenderAccountingOptions) -> (Renderer, Surface, Stats) {
    let mut renderer = Renderer::new(Options::default().with_accounting(options))
        .await
        .unwrap();
    let mut surface = renderer
        .create_headless(Size::try_new(3.0, 2.0).unwrap(), 1.0)
        .await
        .unwrap();
    let mut scene = Scene::new();
    scene.fill(Rect::try_new(0.0, 0.0, 3.0, 2.0).unwrap(), Color::BLACK);
    let stats = renderer
        .render(&mut surface, &scene, Parameters::default())
        .await
        .unwrap();
    (renderer, surface, stats)
}

fn no_clock() -> Duration {
    panic!("Off and untimed readback must not read a clock")
}

#[test]
fn simultaneous_report_borrows_keep_identity_and_taken_reports_outlive_the_renderer() {
    pollster::block_on(async {
        let (mut renderer, surface, _) = published(RenderAccountingOptions::new()).await;
        renderer.read_headless(&surface).await.unwrap();
        let (frame_usage, readback_usage) = {
            let frame = renderer.last_attempt().unwrap();
            let frame_again = renderer.last_attempt().unwrap();
            let readback = renderer.last_readback_attempt().unwrap();
            let readback_again = renderer.last_readback_attempt().unwrap();
            assert!(std::ptr::eq(&*frame, &*frame_again));
            assert!(std::ptr::eq(&*readback, &*readback_again));
            let frame_address = (&*frame) as *const RenderFrameReport;
            let readback_address = (&*readback) as *const RenderReadbackReport;
            let frame_usage = frame.attempt().unwrap().usage();
            let readback_usage = readback.attempt().unwrap().usage();
            let cpu_before = renderer.cpu_accounting_snapshot().unwrap();
            let _formatted = format!("{:?} {:?}", *frame, *readback);
            assert_eq!(renderer.cpu_accounting_snapshot().unwrap(), cpu_before);
            assert_eq!(frame_address, (&*frame_again) as *const RenderFrameReport);
            assert_eq!(
                readback_address,
                (&*readback_again) as *const RenderReadbackReport
            );
            (frame_usage, readback_usage)
        };
        let frame = renderer.take_last_attempt().unwrap();
        let readback = renderer.take_last_readback_attempt().unwrap();
        assert!(renderer.last_attempt().is_none());
        assert!(renderer.last_readback_attempt().is_none());
        drop(renderer);
        assert_eq!(frame.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(readback.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(frame.attempt().unwrap().usage(), frame_usage);
        assert_eq!(readback.attempt().unwrap().usage(), readback_usage);
        assert_eq!(readback_usage.work(RenderWork::ReadbackBytes), 24);
    });
}

#[test]
fn rejected_and_unpolled_renderer_operations_preserve_both_completed_reports() {
    pollster::block_on(async {
        let (mut renderer, mut surface, stats) = published(RenderAccountingOptions::new()).await;
        let pixels = renderer.read_headless(&surface).await.unwrap();
        let frame = renderer.last_attempt().unwrap().snapshot_for_test();
        let readback = renderer
            .last_readback_attempt()
            .unwrap()
            .snapshot_for_test();
        drop(renderer.render(&mut surface, &Scene::new(), Parameters::default()));
        drop(renderer.read_headless(&surface));
        for options in [
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::AuthoredCommands, 0)
                    .unwrap(),
            ),
            RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: usize::MAX,
                filter: RenderPhaseFilter::All,
            }),
        ] {
            renderer.options = Options::default().with_accounting(options);
            assert!(
                renderer
                    .render(&mut surface, &Scene::new(), Parameters::default())
                    .await
                    .is_err()
            );
            assert!(renderer.read_headless(&surface).await.is_err());
            assert_eq!(*renderer.last_attempt().unwrap(), frame);
            assert_eq!(*renderer.last_readback_attempt().unwrap(), readback);
            assert_eq!(renderer.stats(), stats);
        }
        renderer.options = Options::default();
        assert_eq!(renderer.read_headless(&surface).await.unwrap(), pixels);
    });
}

static FRAME_CLEANUP_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn frame_cleanup_clock() -> Duration {
    Duration::from_millis(FRAME_CLEANUP_TIME.load(std::sync::atomic::Ordering::SeqCst))
}

#[test]
fn pending_renderer_frame_drops_child_resources_before_closing_the_report() {
    use std::{
        future::Future,
        sync::atomic::Ordering,
        task::{Context, Poll, Waker},
    };
    struct Cleanup(u8);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::hint::black_box(self.0);
            FRAME_CLEANUP_TIME.fetch_add(7, Ordering::SeqCst);
        }
    }
    pollster::block_on(async {
        FRAME_CLEANUP_TIME.store(0, Ordering::SeqCst);
        let options = RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(frame_cleanup_clock))
            .with_profile(RenderProfile::Summary { timing: true });
        let mut renderer = Renderer::new(Options::default().with_accounting(options))
            .await
            .unwrap();
        let mut surface = renderer
            .create_headless(Size::new(2.0, 2.0), 1.0)
            .await
            .unwrap();
        {
            let future = renderer.frame_operation(&mut surface, async |_, _, account| {
                let mut values = account.temporary::<Cleanup>(1)?;
                values.push(Cleanup(0))?;
                FRAME_CLEANUP_TIME.store(5, Ordering::SeqCst);
                std::future::pending::<()>().await;
                drop(values);
                Err(RenderAccountingError::Invariant.into())
            });
            let mut future = std::pin::pin!(future);
            assert!(matches!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop())),
                Poll::Pending
            ));
        }
        {
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Canceled);
            assert_eq!(
                report.attempt().unwrap().elapsed(),
                Some(Duration::from_millis(12))
            );
            assert_eq!(
                report.attempt().unwrap().phase_elapsed(RenderPhase::Frame),
                Some(Duration::from_millis(12))
            );
            assert_eq!(
                report
                    .renderer_cpu()
                    .unwrap()
                    .usage()
                    .live(RenderResource::ScratchCpuBytes),
                0
            );
            assert_eq!(renderer.stats(), Stats::default());
        }

        renderer
            .render(&mut surface, &Scene::new(), Parameters::default())
            .await
            .unwrap();
        assert_eq!(
            renderer.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );
    });
}

#[test]
fn explicit_readback_reports_actual_work_without_replacing_frame_observations() {
    pollster::block_on(async {
        let mut baseline = None;
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 8,
                filter: RenderPhaseFilter::Phase(RenderPhase::ReadbackDecode),
            },
        ] {
            let (mut renderer, surface, stats) = published(
                RenderAccountingOptions::new()
                    .with_profile(profile)
                    .with_clock(RenderClock::Host(no_clock)),
            )
            .await;
            let frame = renderer.last_attempt().unwrap().snapshot_for_test();
            assert!(renderer.last_readback_attempt().is_none());
            drop(renderer.read_headless(&surface));
            assert!(
                renderer.last_readback_attempt().is_none(),
                "an unpolled future starts no attempt"
            );
            let image = renderer.read_headless(&surface).await.unwrap();
            let bytes = image.into_rgba();
            let capacity = bytes.capacity() as u64;
            assert_eq!(bytes, [0, 0, 0, 255].repeat(6));
            {
                let report = renderer.last_readback_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
                let forward = report.attempt().unwrap();
                assert_eq!(forward.kind(), RenderAttemptKind::Readback);
                let usage = forward.usage();
                for (work, expected) in [
                    (RenderWork::ResourceAllocationRequests, 3),
                    (RenderWork::ReadbackCopies, 1),
                    (RenderWork::Submissions, 1),
                    (RenderWork::ReadbackMapRequests, 1),
                    (RenderWork::ReadbackRows, 2),
                    (RenderWork::ReadbackBytes, 24),
                    (RenderWork::CopyOperations, 0),
                    (RenderWork::ReadbackTransferredBytes, 0),
                ] {
                    assert_eq!(usage.work(work), expected, "{work:?}");
                }
                assert_eq!(forward.elapsed(), None);
                let output = report
                    .renderer_cpu()
                    .unwrap()
                    .scope_usage(RenderCpuScope::ReadbackOutput);
                assert_eq!(output.live(RenderResource::RetainedCpuBytes), capacity);
                assert_eq!(output.peak(RenderResource::RetainedCpuBytes), capacity);
                let staging = report
                    .generation()
                    .unwrap()
                    .unwrap()
                    .scope_usage(RenderGenerationScope::ReadbackStaging);
                assert_eq!(staging.peak(RenderResource::GpuBytes), 512);
                // A callback may finish dropping its staging clone after waking us.
                assert!(matches!(staging.live(RenderResource::GpuBytes), 0 | 512));
                if let Some(expected) = baseline {
                    assert_eq!(
                        usage, expected,
                        "detail selection must not alter authoritative work"
                    );
                } else {
                    baseline = Some(usage);
                }
                if let RenderProfile::Detailed { event_limit, .. } = profile {
                    assert!(forward.events().count() <= event_limit);
                    if event_limit == 0 {
                        assert!(forward.dropped_events() > 0);
                    }
                }
            }

            assert_eq!(*renderer.last_attempt().unwrap(), frame);
            assert_eq!(renderer.stats(), stats);
            let cpu = renderer.cpu_accounting_snapshot().unwrap();
            assert_eq!(
                cpu.usage().work(RenderWork::ReadbackTransferredBytes),
                capacity
            );
            let taken = renderer.take_last_readback_attempt().unwrap();
            assert_eq!(taken.status(), RenderAttemptStatus::Succeeded);
            assert!(renderer.last_readback_attempt().is_none());
            assert_eq!(*renderer.last_attempt().unwrap(), frame);
        }
    });
}

#[test]
fn readback_limits_admit_only_actual_prefixes_and_preserve_the_published_frame() {
    pollster::block_on(async {
        for (work, limit, succeeds, requests, copies, submissions, maps, rows, bytes) in [
            (RenderWork::ReadbackCopies, 0, false, 2, 0, 0, 0, 0, 0),
            (RenderWork::ReadbackMapRequests, 0, false, 3, 1, 1, 0, 0, 0),
            (RenderWork::ReadbackRows, 1, false, 3, 1, 1, 1, 1, 12),
            (RenderWork::ReadbackRows, 2, true, 3, 1, 1, 1, 2, 24),
            (RenderWork::ReadbackBytes, 23, false, 3, 1, 1, 1, 2, 12),
            (RenderWork::ReadbackBytes, 24, true, 3, 1, 1, 1, 2, 24),
        ] {
            let limits = RenderLimits::new().with_work_limit(work, limit).unwrap();
            let (mut renderer, surface, stats) =
                published(RenderAccountingOptions::new().with_limits(limits)).await;
            let frame = renderer.last_attempt().unwrap().snapshot_for_test();
            for _ in 0..2 {
                let result = renderer.read_headless(&surface).await;
                assert_eq!(result.is_ok(), succeeds);
                if let Ok(image) = &result {
                    assert_eq!(image.rgba(), [0, 0, 0, 255].repeat(6));
                }
                {
                    let report = renderer.last_readback_attempt().unwrap();
                    let usage = report.attempt().unwrap().usage();
                    for (unit, amount) in [
                        (RenderWork::ResourceAllocationRequests, requests),
                        (RenderWork::ReadbackCopies, copies),
                        (RenderWork::Submissions, submissions),
                        (RenderWork::ReadbackMapRequests, maps),
                        (RenderWork::ReadbackRows, rows),
                        (RenderWork::ReadbackBytes, bytes),
                    ] {
                        assert_eq!(usage.work(unit), amount, "{work:?} cap {limit}: {unit:?}");
                    }
                    if !succeeds {
                        assert_eq!(report.status(), RenderAttemptStatus::Refused);
                        assert_eq!(
                            report.attempt().unwrap().refusal().unwrap().quantity(),
                            RenderQuantity::Work(work)
                        );
                        assert_eq!(
                            report
                                .renderer_cpu()
                                .unwrap()
                                .scope_usage(RenderCpuScope::ReadbackOutput)
                                .live(RenderResource::RetainedCpuBytes),
                            0
                        );
                    }
                }

                assert_eq!(*renderer.last_attempt().unwrap(), frame);
                assert_eq!(renderer.stats(), stats);
            }
        }
    });
}

#[test]
fn zero_and_unpublished_surfaces_start_readback_reports_without_fabricated_native_work() {
    pollster::block_on(async {
        let mut renderer = Renderer::new(Options::default()).await.unwrap();
        let zero = renderer
            .create_headless(Size::try_new(0.0, 2.0).unwrap(), 1.0)
            .await
            .unwrap();
        assert!(
            renderer
                .read_headless(&zero)
                .await
                .unwrap()
                .rgba()
                .is_empty()
        );
        let zero_report = renderer.take_last_readback_attempt().unwrap();
        assert_eq!(zero_report.status(), RenderAttemptStatus::Succeeded);
        assert!(zero_report.generation().is_none());
        let pending = renderer
            .create_headless(Size::try_new(3.0, 2.0).unwrap(), 1.0)
            .await
            .unwrap();
        let error = renderer.read_headless(&pending).await.unwrap_err();
        let failed = renderer.take_last_readback_attempt().unwrap();
        assert_eq!(failed.status(), RenderAttemptStatus::Failed(error.code()));
        for report in [zero_report, failed] {
            let usage = report.attempt().unwrap().usage();
            for work in [
                RenderWork::ResourceAllocationRequests,
                RenderWork::ReadbackCopies,
                RenderWork::Submissions,
                RenderWork::ReadbackMapRequests,
                RenderWork::ReadbackRows,
                RenderWork::ReadbackBytes,
                RenderWork::ReadbackTransferredBytes,
            ] {
                assert_eq!(usage.work(work), 0);
            }
        }
        assert!(renderer.last_attempt().is_none());
        assert_eq!(renderer.stats(), Stats::default());
    });
}

#[test]
fn direct_report_references_inherit_send_and_sync_from_their_reports() {
    fn send_sync<T: Send + Sync + Unpin + std::panic::UnwindSafe + std::panic::RefUnwindSafe>() {}
    send_sync::<RenderFrameReport>();
    send_sync::<RenderReadbackReport>();
    send_sync::<RenderFrameReportRef<'static>>();
    send_sync::<RenderReadbackReportRef<'static>>();
}

static PHASE_TICKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
fn phase_order_clock() -> Duration {
    Duration::from_millis(PHASE_TICKS.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

#[test]
fn successful_frame_closes_frame_before_publish_and_cleanup_observations() {
    pollster::block_on(async {
        let (renderer, _, _) = published(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Detailed {
                    timing: true,
                    event_limit: 4096,
                    filter: RenderPhaseFilter::All,
                })
                .with_clock(RenderClock::Host(phase_order_clock)),
        )
        .await;
        let report = renderer.last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        let phases: Vec<_> = attempt
            .events()
            .filter_map(|event| match event {
                RenderEvent::Phase { phase, .. } => Some(phase),
                _ => None,
            })
            .collect();
        assert_eq!(attempt.dropped_events(), 0);
        let ended = |phase| phases.iter().position(|actual| *actual == phase).unwrap();
        assert!(ended(RenderPhase::Frame) < ended(RenderPhase::Cleanup));
        assert!(ended(RenderPhase::Cleanup) < ended(RenderPhase::Publish));
        for phase in [
            RenderPhase::Frame,
            RenderPhase::Publish,
            RenderPhase::Cleanup,
        ] {
            assert_eq!(attempt.phase_count(phase), 1);
            assert!(attempt.phase_elapsed(phase).unwrap() > Duration::ZERO);
        }
    });
}

#[test]
fn busy_generation_reports_started_failures_without_publication_and_allows_retry() {
    pollster::block_on(async {
        let (mut renderer, mut surface, stats) = published(RenderAccountingOptions::new()).await;
        let pixels = renderer.read_headless(&surface).await.unwrap();
        let readback = renderer
            .last_readback_attempt()
            .unwrap()
            .snapshot_for_test();
        let generation = renderer
            .backend
            .as_ref()
            .unwrap()
            .generation(renderer.default_device.unwrap())
            .unwrap()
            .clone();
        let busy = generation.begin_interval().unwrap();
        let error = renderer
            .render(&mut surface, &Scene::new(), Parameters::default())
            .await
            .unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        assert_eq!(
            renderer.last_attempt().unwrap().status(),
            RenderAttemptStatus::Failed(error.code())
        );
        assert_eq!(*renderer.last_readback_attempt().unwrap(), readback);
        let frame = renderer.last_attempt().unwrap().snapshot_for_test();
        let error = renderer.read_headless(&surface).await.unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        assert_eq!(
            renderer.last_readback_attempt().unwrap().status(),
            RenderAttemptStatus::Failed(error.code())
        );
        assert_eq!(*renderer.last_attempt().unwrap(), frame);
        assert_eq!(renderer.stats(), stats);
        drop(busy);
        assert_eq!(renderer.read_headless(&surface).await.unwrap(), pixels);
        renderer
            .render(&mut surface, &Scene::new(), Parameters::default())
            .await
            .unwrap();
        assert_eq!(
            renderer.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );
    });
}
