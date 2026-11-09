#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, ErrorCode, GeometryRealizationError, Image, ImageBuffer, Layer,
    Options, Paint, Parameters, Rect, RenderAccountingOptions, RenderAttemptStatus, RenderProfile,
    RenderRoute, RenderWork, Renderer, ResolvedLayerAlphaMask, Scene, Size, Transform,
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

fn assert_coverage(image: &ImageBuffer, expected: [u8; 4], context: &str) {
    for x in [0, 8, 16, 24, 31] {
        for y in [4, 12] {
            let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
            let actual: [u8; 4] = image.rgba()[offset..offset + 4].try_into().unwrap();
            assert_eq!(actual, expected, "{context}: pixel ({x},{y})");
        }
    }
}

fn check_image_brush_inverse(graph: bool) {
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
    let route = if graph {
        RenderRoute::GpuGraph
    } else {
        RenderRoute::DirectVello
    };
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    // Readback is straight RGBA; the graph mask leaves red/alpha128 in
    // premultiplied storage and [255,0,0,128] in this explicit readback.
    let red_pixel = [255, 0, 0, if graph { 128 } else { 255 }];
    let calibration = scene(graph, |scene| {
        scene.fill(bounds(), red);
    });
    let calibrated =
        pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
            .unwrap();
    assert_eq!(calibrated.route, Some(route));
    let red_image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_coverage(&red_image, red_pixel, "normal red calibration");
    let baseline = scene(graph, |scene| {
        scene.fill(bounds(), blue);
    });
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    assert_eq!(successful.route, Some(route));
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_coverage(&before, [0, 0, 255, red_pixel[3]], "prior blue publication");

    let image = Image::from_rgba(
        Size::try_new(2.0, 1.0).unwrap(),
        vec![0, 0, 255, 255, 255, 0, 0, 255],
    )
    .unwrap();
    let target = scene(graph, |scene| {
        // The source shape is exactly (32*2^80)x(16*2^80). Scaling by
        // 2^-80 gives the entire 32x16 output, with finite nonzero native
        // diagonal coefficients. Every sampled positive device position maps far
        // past this two-pixel image, so Pad selects its last, red texel.
        // This uses fill(Paint::image), not the separate Scene::image owner.
        scene.transform(
            Transform::scale(2.0_f64.powi(-80), 2.0_f64.powi(-80)).unwrap(),
            |scene| {
                scene.fill(
                    Rect::try_new(0.0, 0.0, 32.0 * 2.0_f64.powi(80), 16.0 * 2.0_f64.powi(80))
                        .unwrap(),
                    Paint::image(image),
                );
            },
        );
    });
    match pollster::block_on(renderer.render(&mut surface, &target, Parameters::default())) {
        Ok(stats) => {
            assert_eq!(stats.route, Some(route));
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_coverage(
                &image,
                red_pixel,
                "successful image-brush inverse publication",
            );
        }
        Err(error) => {
            assert_eq!(error.code(), ErrorCode::GeometryRealization);
            assert_eq!(
                error.geometry_realization_diagnostic(),
                Some(&GeometryRealizationError::InsufficientNativeCapacity)
            );
            assert_eq!(renderer.stats(), successful);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                before
            );
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(
                report.status(),
                RenderAttemptStatus::Failed(ErrorCode::GeometryRealization)
            );
            assert_eq!(report.cleanup_fault(), None);
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions),
                0
            );
            let retry = pollster::block_on(renderer.render(
                &mut surface,
                &calibration,
                Parameters::default(),
            ))
            .unwrap();
            assert_eq!(retry.route, Some(route));
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                red_image
            );
            let retry_report = renderer.take_last_attempt().unwrap();
            assert_eq!(retry_report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(retry_report.cleanup_fault(), None);
            assert!(
                retry_report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions)
                    > 0
            );
        }
    }
}

#[test]
fn direct_image_brush_inverse_preserves_coverage_or_refuses_atomically() {
    check_image_brush_inverse(false);
}

#[test]
fn graph_image_brush_inverse_preserves_coverage_or_refuses_atomically() {
    check_image_brush_inverse(true);
}
