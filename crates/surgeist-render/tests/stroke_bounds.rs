#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, ErrorCode, Image, ImageBuffer, Layer, LineCap, LineJoin, Options,
    Parameters, Path, Point, Rect, RenderAccountingOptions, RenderAttemptStatus, RenderProfile,
    RenderResource, RenderRoute, RenderWork, Renderer, ResolvedLayerAlphaMask, Scene, Shape, Size,
    Stroke, Surface, Transform,
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

fn renderer() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    (renderer, surface)
}

fn route(graph: bool) -> RenderRoute {
    if graph {
        RenderRoute::GpuGraph
    } else {
        RenderRoute::DirectVello
    }
}

fn render_read(
    renderer: &mut Renderer,
    surface: &mut Surface,
    scene: &Scene,
    graph: bool,
) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(route(graph)));
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn path(points: &[(f64, f64)]) -> Shape {
    let mut path = Path::new();
    let &(x, y) = points.first().unwrap();
    path.move_to(Point::try_new(x, y).unwrap());
    for &(x, y) in &points[1..] {
        path.line_to(Point::try_new(x, y).unwrap());
    }
    Shape::path(path)
}

fn red() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4]) {
    let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    assert_eq!(
        &image.rgba()[offset..offset + 4],
        &expected,
        "pixel ({x},{y})"
    );
}

fn red_pixel(graph: bool) -> [u8; 4] {
    // Headless readback is straight RGBA; the graph mask halves opacity.
    [255, 0, 0, if graph { 128 } else { 255 }]
}

fn unused_miter_preserves_stroke(join: LineJoin, graph: bool) {
    let (mut renderer, mut surface) = renderer();
    let target = |miter| {
        scene(graph, |scene| {
            scene.stroke(
                path(&[(2.1, 8.1), (29.9, 8.1)]),
                Stroke::try_new(4.2)
                    .unwrap()
                    .join(join)
                    .try_miter_limit(miter)
                    .unwrap(),
                red(),
            );
        })
    };
    let ordinary = render_read(&mut renderer, &mut surface, &target(4.0), graph);
    // The center pixel is wholly inside the horizontal width-4.2 band;
    // (0,0) is outside it. This rules out an empty calibration oracle.
    assert_pixel(&ordinary, 16, 8, red_pixel(graph));
    assert_pixel(&ordinary, 0, 0, [0, 0, 0, 0]);
    let baseline = scene(graph, |scene| {
        scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
    });
    render_read(&mut renderer, &mut surface, &baseline, graph);
    // Changing an unused miter limit cannot change the stroke's ink.
    let unused_max = render_read(&mut renderer, &mut surface, &target(f64::MAX), graph);
    assert_eq!(unused_max, ordinary);
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.cleanup_fault(), None);
    assert!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions)
            > 0
    );
}

#[test]
fn direct_round_join_ignores_unused_max_miter_and_preserves_visible_stroke() {
    unused_miter_preserves_stroke(LineJoin::Round, false);
}

#[test]
fn graph_round_join_ignores_unused_max_miter_and_preserves_visible_stroke() {
    unused_miter_preserves_stroke(LineJoin::Round, true);
}

#[test]
fn direct_bevel_join_ignores_unused_max_miter_and_preserves_visible_stroke() {
    unused_miter_preserves_stroke(LineJoin::Bevel, false);
}

#[test]
fn graph_bevel_join_ignores_unused_max_miter_and_preserves_visible_stroke() {
    unused_miter_preserves_stroke(LineJoin::Bevel, true);
}

fn square_caps_preserve_corners(graph: bool, miter_limit: f64) {
    let (mut renderer, mut surface) = renderer();
    for reversed in [false, true] {
        for (transform, pixel) in [
            (Transform::translation(38.25, 0.0).unwrap(), (31, 0)),
            (
                Transform::try_new([0.0, 2.0, -1.0, 0.0, 16.0, 29.0]).unwrap(),
                (16, 15),
            ),
        ] {
            // For width12 and tangent(1,1)/sqrt2, the square corner is
            // (-6*sqrt2,0). A half-width-only local bound is entirely
            // outside the viewport in both transforms above. The chosen
            // whole pixel lies inside |(x+y)/sqrt2|<=6 and
            // |(-x+y)/sqrt2|<=6 after inverse transformation.
            let (points, caps) = if reversed {
                ([(8.0, 8.0), (0.0, 0.0)], (LineCap::Butt, LineCap::Square))
            } else {
                ([(0.0, 0.0), (8.0, 8.0)], (LineCap::Square, LineCap::Butt))
            };
            let target = scene(graph, |scene| {
                // Keep the graph observable if incorrect stroke bounds
                // cull all cap ink. This mark is away from every sample.
                scene.fill(Rect::try_new(0.0, 14.0, 1.0, 1.0).unwrap(), red());
                scene.transform(transform, |scene| {
                    scene.stroke(
                        path(&points),
                        Stroke::try_new(12.0)
                            .unwrap()
                            .join(LineJoin::Bevel)
                            .caps(caps.0, caps.1)
                            .try_miter_limit(miter_limit)
                            .unwrap(),
                        red(),
                    );
                });
            });
            let image = render_read(&mut renderer, &mut surface, &target, graph);
            assert_pixel(&image, pixel.0, pixel.1, red_pixel(graph));
            assert_pixel(&image, 0, 0, [0, 0, 0, 0]);
        }
    }
}

#[test]
fn square_caps_keep_visible_corners_beyond_half_width_under_rotation_and_nonuniform_scale() {
    for graph in [false, true] {
        square_caps_preserve_corners(graph, 4.0);
    }
}

#[test]
fn direct_square_caps_preserve_visible_corners_with_unused_miter_limit_one() {
    square_caps_preserve_corners(false, 1.0);
}

#[test]
fn graph_square_caps_preserve_visible_corners_with_unused_miter_limit_one() {
    square_caps_preserve_corners(true, 1.0);
}

#[test]
fn genuine_miter_join_keeps_visible_tip_and_obeys_limit() {
    let (mut renderer, mut surface) = renderer();
    for graph in [false, true] {
        let target = |limit| {
            scene(graph, |scene| {
                // Keep the masked graph observable when limit1 legitimately
                // removes all of the tested stroke's viewport coverage.
                scene.fill(Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap(), red());
                // The 90-degree outer miter is (41-8*sqrt2,8). All
                // centerline points are x>=41; half-width8 alone would cull
                // the entire stroke. Pixel(31,7) lies wholly in its tip.
                scene.stroke(
                    path(&[(49.0, 16.0), (41.0, 8.0), (49.0, 0.0)]),
                    Stroke::try_new(16.0)
                        .unwrap()
                        .join(LineJoin::Miter)
                        .try_miter_limit(limit)
                        .unwrap(),
                    red(),
                );
            })
        };
        let miter = render_read(&mut renderer, &mut surface, &target(2.0), graph);
        assert_pixel(&miter, 31, 7, red_pixel(graph));
        // Ratio sqrt2 exceeds limit1, so the join becomes a bevel with x>32.
        let bevel = render_read(&mut renderer, &mut surface, &target(1.0), graph);
        assert_pixel(&bevel, 31, 7, [0, 0, 0, 0]);
    }
}

#[test]
fn required_nonfinite_stroke_bounds_refuse_with_typed_error_preserving_publication_and_retry() {
    let (mut renderer, mut surface) = renderer();
    for graph in [false, true] {
        let baseline = scene(graph, |scene| {
            scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
        });
        let before = render_read(&mut renderer, &mut surface, &baseline, graph);
        let successful = renderer.stats();
        let invalid = scene(graph, |scene| {
            // The actual right-hand stroke edge is 0.75*MAX + 0.5*MAX,
            // outside the finite f64 domain even with a round join and no
            // square cap. This is required ink, not an unused miter product.
            scene.stroke(
                path(&[(f64::MAX * 0.75, 0.0), (f64::MAX * 0.75, 1.0)]),
                Stroke::try_new(f64::MAX).unwrap().join(LineJoin::Round),
                red(),
            );
        });
        let error =
            pollster::block_on(renderer.render(&mut surface, &invalid, Parameters::default()))
                .unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
        assert!(error.geometry_realization_diagnostic().is_none());
        assert!(error.accounting_diagnostic().is_none());
        assert_eq!(renderer.stats(), successful);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(
            report.status(),
            RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
        );
        assert_eq!(report.cleanup_fault(), None);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::Submissions), 0);
        assert!(usage.work(RenderWork::CommandVisits) > 0);
        for resource in [
            RenderResource::RetainedCpuBytes,
            RenderResource::ScratchCpuBytes,
        ] {
            assert_eq!(usage.live(resource), 0, "{resource:?}");
        }
        let retry = scene(graph, |scene| {
            scene.fill(bounds(), red());
        });
        let after = render_read(&mut renderer, &mut surface, &retry, graph);
        assert_pixel(&after, 16, 8, red_pixel(graph));
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.cleanup_fault(), None);
        assert!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::Submissions)
                > 0
        );
    }
}
