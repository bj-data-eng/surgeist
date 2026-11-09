#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use std::io::Write;
use surgeist_render::{
    Color, Dash, EffectQualityPolicy, ErrorCode, GeometryRealizationError, Image, Layer, Options,
    Parameters, Rect, RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderResource,
    RenderRoute, RenderWork, Renderer, ResolvedLayerAlphaMask, Scene, Size, Stroke,
};

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new()
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, bounds()).unwrap()),
            draw,
        );
    } else {
        draw(&mut scene);
    }
    scene
}

fn check_overflow(graph: bool, offset: f64) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let calibration = scene(graph, |scene| {
        scene.fill(bounds(), blue);
    });
    let prior_stats =
        pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
            .unwrap();
    let prior_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let alpha = if graph { 128 } else { 255 };
    assert_eq!(
        prior_stats.route,
        Some(if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        })
    );
    assert_eq!(&prior_pixels.rgba()[0..4], [0, 0, 255, alpha]);

    // Each authored length is finite. The odd repeated native period is not.
    // A zero offset avoids the old initialization loop but still requires the
    // same contextual capacity certificate as a negative offset.
    let dash = Dash::try_new(offset, &[f64::MAX]).unwrap();
    let stroke = Stroke::try_new(2.0).unwrap().try_dash(dash).unwrap();
    let target = scene(graph, |scene| {
        scene.stroke(Rect::try_new(2.0, 2.0, 28.0, 12.0).unwrap(), stroke, red);
    });
    println!("published blue calibration; entering overflowed native dash cycle");
    std::io::stdout().flush().unwrap();
    let error = pollster::block_on(renderer.render(&mut surface, &target, Parameters::default()))
        .expect_err("a nonfinite required native cycle must refuse before initialization");
    assert_eq!(error.code(), ErrorCode::GeometryRealization);
    assert_eq!(
        error.geometry_realization_diagnostic(),
        Some(&GeometryRealizationError::InsufficientNativeCapacity)
    );
    assert_eq!(renderer.stats(), prior_stats);
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Failed(error.code()));
    assert_eq!(report.cleanup_fault(), None);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::DashInitializations),
        0
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .live(RenderResource::ScratchCpuBytes),
        0
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        0
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        0
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        prior_pixels
    );

    let retry = scene(graph, |scene| {
        scene.fill(bounds(), red);
        scene.stroke(
            Rect::try_new(2.0, 2.0, 28.0, 12.0).unwrap(),
            Stroke::try_new(2.0)
                .unwrap()
                .try_dash(Dash::try_new(-1.0, &[4.0, 3.0]).unwrap())
                .unwrap(),
            blue,
        );
    });
    pollster::block_on(renderer.render(&mut surface, &retry, Parameters::default())).unwrap();
    let retry_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let center = (8 * 32 + 16) * 4;
    assert_eq!(&retry_pixels.rgba()[center..center + 4], [255, 0, 0, alpha]);
}

#[test]
fn direct_overflowed_cycle_at_zero_offset_refuses_atomically() {
    check_overflow(false, 0.0);
}

#[test]
fn graph_overflowed_cycle_at_zero_offset_refuses_atomically() {
    check_overflow(true, 0.0);
}

#[test]
fn direct_overflowed_cycle_at_negative_offset_refuses_before_initialization() {
    check_overflow(false, -1.0);
}

#[test]
fn graph_overflowed_cycle_at_negative_offset_refuses_before_initialization() {
    check_overflow(true, -1.0);
}

fn line_scene(graph: bool, dash: Dash) -> Scene {
    use surgeist_render::{Path, Point, Shape};
    scene(graph, |scene| {
        scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
        let mut path = Path::new();
        path.move_to(Point::try_new(0.0, 8.0).unwrap())
            .line_to(Point::try_new(32.0, 8.0).unwrap());
        scene.stroke(
            Shape::path(path),
            Stroke::try_new(2.0).unwrap().try_dash(dash).unwrap(),
            Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
        );
    })
}

#[test]
fn open_dash_patterns_keep_lengths_offsets_and_solid_zero_semantics() {
    let cases: &[(f64, &'static [f64], [bool; 3], u64)] = &[
        (0.0, &[4.0, 4.0], [true, false, true], 1),
        (0.0, &[4.0], [true, false, true], 1),
        (2.0, &[4.0, 4.0], [true, false, true], 1),
        (-2.0, &[4.0, 4.0], [false, true, false], 1),
        (0.0, &[], [true, true, true], 0),
        (-2.0, &[0.0, 0.0, 0.0], [true, true, true], 0),
    ];
    for graph in [false, true] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        for &(offset, intervals, red, initializations) in cases {
            let input = line_scene(graph, Dash::try_new(offset, intervals).unwrap());
            pollster::block_on(renderer.render(&mut surface, &input, Parameters::default()))
                .unwrap();
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::DashInitializations),
                initializations
            );
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for (x, red) in [0, 4, 8].into_iter().zip(red) {
                let byte_offset = (8 * 32 + x) * 4;
                let color = if red { [255, 0, 0] } else { [0, 0, 255] };
                assert_eq!(
                    &pixels.rgba()[byte_offset..byte_offset + 3],
                    color,
                    "{intervals:?} phase {offset}"
                );
            }
        }
    }
}

#[test]
fn native_initializer_admission_preserves_profiles_prefix_custody_and_retry() {
    use surgeist_render::{
        RenderAccountingError, RenderLimits, RenderPhase, RenderPhaseFilter, RenderQuantity,
        RenderRefusalReason,
    };
    for graph in [false, true] {
        let mut expected_usage = None;
        for profile in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 1,
                filter: RenderPhaseFilter::Phase(RenderPhase::Encode),
            },
            RenderProfile::Detailed {
                timing: true,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            },
        ] {
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::DashInitializations, 0)
                .unwrap();
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
                    .with_accounting(
                        RenderAccountingOptions::new()
                            .with_profile(profile)
                            .with_limits(limits),
                    ),
            ))
            .unwrap();
            let mut surface = pollster::block_on(
                renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0),
            )
            .unwrap();
            let baseline = scene(graph, |scene| {
                scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
            });
            let previous =
                pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
                    .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let input = line_scene(graph, Dash::try_new(-1.0, &[4.0, 3.0]).unwrap());
            let error =
                pollster::block_on(renderer.render(&mut surface, &input, Parameters::default()))
                    .unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected the actual native initializer admission refusal");
            };
            assert_eq!(
                refusal.quantity(),
                RenderQuantity::Work(RenderWork::DashInitializations)
            );
            assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
            assert_eq!(refusal.current(), 0);
            assert_eq!(refusal.amount(), 1);
            assert!(error.geometry_realization_diagnostic().is_none());
            assert_eq!(renderer.stats(), previous);
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::DashInitializations), 0);
            assert_eq!(usage.work(RenderWork::Submissions), 0);
            assert!(usage.work(RenderWork::CoordinateVisits) > 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
            if let Some(expected) = expected_usage {
                assert_eq!(usage, expected);
            } else {
                expected_usage = Some(usage);
            }
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                pixels
            );
            let retry = line_scene(graph, Dash::try_new(0.0, &[0.0]).unwrap());
            pollster::block_on(renderer.render(&mut surface, &retry, Parameters::default()))
                .unwrap();
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::DashInitializations),
                0
            );
        }
    }
}

#[test]
fn rounded_full_cycle_phase_keeps_the_visible_gap_or_refuses_atomically() {
    // The authored odd pattern is [2^100, 2^100]. Offset -2 puts a two-unit
    // gap before the first mark, independently of the huge remaining lengths.
    const LONG_DASH: f64 = (1_u128 << 100) as f64;
    for graph in [false, true] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        let baseline = scene(graph, |scene| {
            scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
        });
        let stats =
            pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
                .unwrap();
        let previous = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let target = line_scene(graph, Dash::try_new(-2.0, &[LONG_DASH]).unwrap());
        match pollster::block_on(renderer.render(&mut surface, &target, Parameters::default())) {
            Ok(_) => {
                let actual = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                for (x, expected) in [(0, [0, 0, 255]), (4, [255, 0, 0])] {
                    let offset = (8 * 32 + x) * 4;
                    assert_eq!(&actual.rgba()[offset..offset + 3], expected, "gap at x{x}");
                }
            }
            Err(error) => {
                assert_eq!(
                    error.geometry_realization_diagnostic(),
                    Some(&GeometryRealizationError::InsufficientNativeCapacity)
                );
                assert_eq!(renderer.stats(), stats);
                let report = renderer.take_last_attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Failed(error.code()));
                assert_eq!(report.cleanup_fault(), None);
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::Submissions),
                    0
                );
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::DashInitializations),
                    0
                );
                assert_eq!(
                    pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                    previous
                );
            }
        }
    }
}
