#![cfg(not(target_arch = "wasm32"))]

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use surgeist_render::*;

fn solid_scene() -> Scene {
    let mut scene = Scene::new();
    scene.fill(Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(), Color::BLACK);
    scene
}

static CLOCK_READS: AtomicU64 = AtomicU64::new(0);
fn counted_clock() -> Duration {
    Duration::from_nanos(CLOCK_READS.fetch_add(1, Ordering::SeqCst) + 1)
}

#[test]
fn selected_clock_covers_initialization_and_frame_but_off_and_untimed_never_read_it() {
    // This test owns the callback and counter; other tests never use either.
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        CLOCK_READS.store(0, Ordering::SeqCst);
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(profile)
                    .with_clock(RenderClock::Host(counted_clock)),
            ),
        ))
        .unwrap();
        let initialized = renderer.device_accounting_snapshot(None).unwrap();
        assert_eq!(initialized.elapsed().is_some(), timed);
        assert_eq!(
            initialized
                .scope_elapsed(RenderGenerationScope::Initialization)
                .is_some(),
            timed
        );
        assert_eq!(CLOCK_READS.load(Ordering::SeqCst) > 0, timed);

        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 1.0))
                .unwrap();
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            &solid_scene(),
            Parameters::default(),
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        assert_eq!(stats.commands, 1);
        assert_eq!(stats.fills, 1);
        assert_eq!(stats.vello_passes, 1);
        assert_eq!(stats.frame_time.is_some(), timed);
        assert_eq!(stats.preparation_time.is_some(), timed);
        assert_eq!(stats.execution_time.is_some(), timed);
        if timed {
            assert!(stats.frame_time.unwrap() > Duration::ZERO);
            assert!(stats.preparation_time.unwrap() > Duration::ZERO);
            assert!(stats.execution_time.unwrap() > Duration::ZERO);
        }
        {
            let report = renderer.last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.attempt().unwrap().elapsed(), stats.frame_time);
            assert_eq!(report.renderer_cpu().unwrap().elapsed().is_some(), timed);
            assert_eq!(
                report.generation().unwrap().unwrap().elapsed().is_some(),
                timed
            );
        }

        assert_eq!(
            renderer
                .cpu_accounting_snapshot()
                .unwrap()
                .elapsed()
                .is_some(),
            timed
        );
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .identity(),
            initialized.identity()
        );
        assert_eq!(renderer.stats(), stats);
        assert_eq!(
            renderer.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );
        assert_eq!(CLOCK_READS.load(Ordering::SeqCst) > 0, timed);
    }
}

#[test]
fn bounded_and_filtered_detail_preserve_frame_work_custody_and_published_statistics() {
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
            event_limit: 4,
            filter: RenderPhaseFilter::Phase(RenderPhase::Normalize),
        },
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 1.0))
                .unwrap();
        let stats = pollster::block_on(renderer.render(
            &mut surface,
            &solid_scene(),
            Parameters::default(),
        ))
        .unwrap();
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.usage().work(RenderWork::CommandsEmitted), 1);
        assert_eq!(attempt.usage().work(RenderWork::FillsEmitted), 1);
        assert_eq!(attempt.usage().work(RenderWork::ImagesEmitted), 0);
        assert_eq!(attempt.usage().work(RenderWork::VelloPasses), 1);
        let cpu = report.renderer_cpu().unwrap().usage();
        // Native resource maps have independent randomized traversal order in
        // each Renderer. Their cleanup sorting compares a different token
        // permutation, so truthful CPU probe work can differ even with Off.
        // Compare exact frame/native work and all CPU backing balances/peaks;
        // deterministic CPU-owner tests cover identical ordered CPU work.
        assert!(cpu.work(RenderWork::ResourceProbes) > 0);
        let cpu_custody = [
            RenderResource::RetainedCpuBytes,
            RenderResource::ScratchCpuBytes,
        ]
        .map(|resource| {
            (
                cpu.opening_live(resource),
                cpu.live(resource),
                cpu.peak(resource),
            )
        });
        let measured = (
            stats,
            attempt.usage(),
            cpu_custody,
            report.generation().unwrap().unwrap().usage(),
        );
        if let Some(expected) = &baseline {
            assert_eq!(&measured, expected);
        } else {
            baseline = Some(measured);
        }
        match profile {
            RenderProfile::Detailed { event_limit: 0, .. } => {
                assert_eq!(attempt.events().count(), 0);
                assert!(attempt.dropped_events() > 0);
                assert!(report.renderer_cpu().unwrap().dropped_events() > 0);
                assert!(report.generation().unwrap().unwrap().dropped_events() > 0);
            }
            RenderProfile::Detailed { event_limit: 4, .. } => {
                let events: Vec<_> = attempt.events().collect();
                assert!(!events.is_empty());
                assert!(events.len() <= 4);
                for event in events {
                    let phase = match event {
                        RenderEvent::Change { phase, .. }
                        | RenderEvent::Phase { phase, .. }
                        | RenderEvent::Refused { phase, .. } => phase,
                    };
                    assert_eq!(phase, RenderPhase::Normalize);
                }
                assert_eq!(report.renderer_cpu().unwrap().events().count(), 0);
                assert_eq!(report.generation().unwrap().unwrap().events().count(), 0);
            }
            _ => assert_eq!(attempt.events().count(), 0),
        }
        assert!(renderer.last_attempt().is_none());
        renderer.cpu_accounting_snapshot().unwrap();
        renderer.device_accounting_snapshot(Some(&surface)).unwrap();
        assert!(renderer.last_attempt().is_none());
        assert_eq!(renderer.stats(), stats);
    }
}

fn image_scene(colors: &[[u8; 4]]) -> Scene {
    let mut scene = Scene::new();
    for &pixel in colors {
        let image =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), Arc::<[u8]>::from(pixel)).unwrap();
        scene.image(
            image,
            Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(),
            ImageFit::Stretch,
        );
    }
    scene
}

#[test]
fn refused_new_image_history_preserves_pixels_and_retry_still_observes_a_miss() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ImageContentMisses, 1)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 1.0))
            .unwrap();
    let baseline = image_scene(&[[255, 0, 0, 255]]);
    let prior = pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
        .unwrap();
    let pixels = pollster::block_on(renderer.read_headless(&surface))
        .unwrap()
        .into_rgba();
    assert_eq!(pixels, [255, 0, 0, 255].repeat(4));
    let refused = image_scene(&[[0, 255, 0, 255], [0, 0, 255, 255]]);
    let error = pollster::block_on(renderer.render(&mut surface, &refused, Parameters::default()))
        .unwrap_err();
    assert!(matches!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Refused(_))
    ));
    assert_eq!(renderer.stats(), prior);
    {
        let report = renderer.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ImageContentMisses),
            1
        );
    }
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface))
            .unwrap()
            .rgba(),
        pixels
    );
    let retry_scene = image_scene(&[[0, 255, 0, 255]]);
    let retry =
        pollster::block_on(renderer.render(&mut surface, &retry_scene, Parameters::default()))
            .unwrap();
    assert_eq!(retry.image_content_misses, 1);
    assert_eq!(retry.image_content_hits, 0);
    let warm =
        pollster::block_on(renderer.render(&mut surface, &retry_scene, Parameters::default()))
            .unwrap();
    assert_eq!(warm.image_content_misses, 0);
    assert_eq!(warm.image_content_hits, 1);
}
