#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, Image, ImageBuffer, Layer, Options, Parameters, Rect,
    RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderRoute, RenderWork, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, Transform,
};

#[derive(Clone, Copy, Debug)]
enum Geometry {
    ExtremeFiniteFactors,
    CollapsedUnitEdge,
    TinyLocalEdges,
}

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

fn geometry_scene(geometry: Geometry, graph: bool, red: Color) -> Scene {
    scene(graph, |scene| match geometry {
        Geometry::ExtremeFiniteFactors => {
            // Each finite local rectangle has width MAX. The separately
            // transformed halves cover approximately [-48,16] and [16,80].
            // The entire inverse-mapped 32-wide output domain remains finite:
            // width 2^1023 and inverse translation magnitude 2^1022.
            for x in [-f64::MAX, 0.0] {
                scene.transform(
                    Transform::try_new([16.0 * f64::MIN_POSITIVE, 0.0, 0.0, 1.0, 16.0, 0.0])
                        .unwrap(),
                    |scene| {
                        scene.fill(Rect::try_new(x, 0.0, f64::MAX, 16.0).unwrap(), red);
                    },
                );
            }
        }
        Geometry::CollapsedUnitEdge => {
            // Both endpoints are finite, but f32(2^24+1)==f32(2^24).
            // Applying the exact translation first preserves column [0,1].
            let origin = 2.0_f64.powi(24);
            scene.transform(Transform::translation(-origin, 0.0).unwrap(), |scene| {
                scene.fill(Rect::try_new(origin, 0.0, 1.0, 16.0).unwrap(), red);
            });
        }
        Geometry::TinyLocalEdges => {
            // These exact powers are nonzero even in f32. Their final device
            // rectangle is exactly 32x16; eliminating tiny local edges first
            // would erase real device coverage.
            scene.transform(
                Transform::scale(2.0_f64.powi(48), 2.0_f64.powi(48)).unwrap(),
                |scene| {
                    scene.fill(
                        Rect::try_new(0.0, 0.0, 2.0_f64.powi(-43), 2.0_f64.powi(-44)).unwrap(),
                        red,
                    );
                },
            );
        }
    })
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], context: &str) {
    let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[offset..offset + 4].try_into().unwrap();
    assert_eq!(actual, expected, "{context}: pixel ({x},{y})");
}

fn check_geometry(geometry: Geometry, graph: bool) {
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
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let route = if graph {
        RenderRoute::GpuGraph
    } else {
        RenderRoute::DirectVello
    };
    // Readback is straight RGBA. The graph's native premultiplied red/alpha
    // are 128/128; its straight readback is [255,0,0,128].
    let red_pixel = [255, 0, 0, if graph { 128 } else { 255 }];
    let calibration = scene(graph, |scene| {
        scene.fill(bounds(), red);
    });
    let calibration_stats =
        pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
            .unwrap();
    assert_eq!(calibration_stats.route, Some(route));
    let red_image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for x in [0, 16, 31] {
        assert_pixel(&red_image, x, 4, red_pixel, "normal red calibration");
    }
    // Preserve a different published color so a successful red outcome cannot
    // be satisfied by leaving the prior surface unchanged.
    let baseline = scene(graph, |scene| {
        scene.fill(bounds(), Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap());
    });
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    assert_eq!(successful.route, Some(route));
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for x in [0, 16, 31] {
        assert_pixel(
            &before,
            x,
            4,
            [0, 0, 255, red_pixel[3]],
            "prior blue publication",
        );
    }
    let extreme = geometry_scene(geometry, graph, red);
    match pollster::block_on(renderer.render(&mut surface, &extreme, Parameters::default())) {
        Ok(stats) => {
            assert_eq!(stats.route, Some(route));
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let context = format!("{geometry:?}, route={route:?}, successful publication");
            match geometry {
                Geometry::CollapsedUnitEdge => {
                    for y in [4, 12] {
                        assert_pixel(&image, 0, y, red_pixel, &context);
                        assert_pixel(&image, 2, y, [0, 0, 0, 0], &context);
                    }
                }
                Geometry::ExtremeFiniteFactors | Geometry::TinyLocalEdges => {
                    for x in [0, 8, 16, 24, 31] {
                        for y in [4, 12] {
                            assert_pixel(&image, x, y, red_pixel, &context);
                        }
                    }
                }
            }
        }
        Err(error) => {
            // Preservation and atomic refusal are both valid outcomes.
            // Contextual diagnostic details have their own focused checks.
            assert_eq!(renderer.stats(), successful);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                before
            );
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
fn direct_extreme_finite_factors_render_red_or_refuse_atomically() {
    check_geometry(Geometry::ExtremeFiniteFactors, false);
}

#[test]
fn graph_extreme_finite_factors_render_masked_red_or_refuse_atomically() {
    check_geometry(Geometry::ExtremeFiniteFactors, true);
}

#[test]
fn direct_unit_edge_survives_translation_or_refuses_atomically() {
    check_geometry(Geometry::CollapsedUnitEdge, false);
}

#[test]
fn graph_unit_edge_survives_translation_or_refuses_atomically() {
    check_geometry(Geometry::CollapsedUnitEdge, true);
}

#[test]
fn direct_tiny_local_edges_survive_final_scale_or_refuse_atomically() {
    check_geometry(Geometry::TinyLocalEdges, false);
}

#[test]
fn graph_tiny_local_edges_survive_final_scale_or_refuse_atomically() {
    check_geometry(Geometry::TinyLocalEdges, true);
}
