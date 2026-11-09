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
            drop(report);
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
                drop(report);
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
