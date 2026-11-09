#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use std::io::Write;
use surgeist_render::{
    Color, Dash, EffectQualityPolicy, ErrorCode, GeometryRealizationError, Image, Layer, Options,
    Parameters, Rect, RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderRoute,
    RenderWork, Renderer, ResolvedLayerAlphaMask, Scene, Size, Stroke,
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
