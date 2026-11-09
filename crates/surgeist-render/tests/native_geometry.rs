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

#[test]
fn contextual_geometry_errors_preserve_their_closed_reason_and_local_indices() {
    use surgeist_render::{Error, ErrorCode, GeometryRealizationError as Reason, Point};
    for reason in [
        Reason::UnrepresentableTransform,
        Reason::UnrepresentablePath { element_index: 7 },
        Reason::UnrepresentableStroke,
        Reason::UnrepresentableGlyph { glyph_index: 3 },
        Reason::UnrepresentableBlur,
        Reason::InsufficientNativeCapacity,
    ] {
        let error: Error = reason.into();
        assert_eq!(error.code(), ErrorCode::GeometryRealization);
        assert_eq!(error.geometry_realization_diagnostic(), Some(&reason));
        assert!(error.invalid_value_diagnostic().is_none());
        assert!(error.gradient_realization_diagnostic().is_none());
        assert!(error.glyph_preparation_diagnostic().is_none());
        assert!(error.accounting_diagnostic().is_none());
    }
    let intrinsic = Point::try_new(f64::INFINITY, 0.0).unwrap_err();
    assert_eq!(intrinsic.code(), ErrorCode::InvalidInput);
    assert!(intrinsic.geometry_realization_diagnostic().is_none());
}

fn profiled_renderer(profile: RenderProfile) -> (Renderer, surgeist_render::Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
            .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    (renderer, surface)
}

fn render_read(
    renderer: &mut Renderer,
    surface: &mut surgeist_render::Surface,
    target: &Scene,
    graph: bool,
) -> ImageBuffer {
    let stats =
        pollster::block_on(renderer.render(surface, target, Parameters::default())).unwrap();
    assert_eq!(
        stats.route,
        Some(if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        })
    );
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn assert_contextual_refusal(
    error: &surgeist_render::Error,
    reason: surgeist_render::GeometryRealizationError,
) {
    assert_eq!(
        error.code(),
        surgeist_render::ErrorCode::GeometryRealization
    );
    assert_eq!(error.geometry_realization_diagnostic(), Some(&reason));
    assert!(error.invalid_value_diagnostic().is_none());
    assert!(error.gradient_realization_diagnostic().is_none());
    assert!(error.glyph_preparation_diagnostic().is_none());
    assert!(error.accounting_diagnostic().is_none());
}

#[test]
fn concrete_transform_path_stroke_blur_and_glyph_losses_have_contextual_receiving_diagnostics() {
    use surgeist_render::{
        FontData, FontRef, GeometryRealizationError as Reason, GlyphRunLimits, GlyphSynthesis,
        Point, RenderAccountingContext, Shadow, Stroke, TextGlyph, TextInstance, TextPaint,
        TextRun, TextRunBounds,
    };
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    // Use actual face geometry for the visible glyph. Its empty predecessor
    // contributes no ink; the tiny x placement translates this authentic box.
    let canonical_visible = [TextGlyph::try_new(58, 0.0, 16.0, 16.0).unwrap()];
    let canonical = font
        .prepare_glyph_run(
            &TextInstance::try_new(16.0, vec![], GlyphSynthesis::none()).unwrap(),
            &canonical_visible,
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let ink = canonical.bounds().ink_rect().unwrap();
    let tiny_ink = TextRunBounds::try_ink(
        Rect::try_new(
            ink.x() + f64::MIN_POSITIVE,
            ink.y(),
            ink.width(),
            ink.height(),
        )
        .unwrap(),
    )
    .unwrap();
    let (mut renderer, mut surface) = profiled_renderer(RenderProfile::Summary { timing: false });
    for graph in [false, true] {
        let tiny_stroke = scene(graph, |scene| {
            scene.stroke(bounds(), Stroke::try_new(f64::MIN_POSITIVE).unwrap(), red);
        });
        let tiny_blur = scene(graph, |scene| {
            scene.shadow(
                Rect::try_new(2.0, 2.0, 28.0, 12.0).unwrap(),
                Shadow::try_new(
                    Point::try_new(0.0, 0.0).unwrap(),
                    f64::MIN_POSITIVE,
                    0.0,
                    red,
                )
                .unwrap(),
            );
        });
        let glyphs = [
            TextGlyph::try_new(3, f64::MAX, f64::MAX, 16.0).unwrap(),
            TextGlyph::try_new(58, f64::MIN_POSITIVE, 16.0, 16.0).unwrap(),
        ];
        let tiny_glyph = scene(graph, |scene| {
            scene.text_run(
                TextRun::try_new(
                    FontRef::new(2111).with_data(font.clone()),
                    16.0,
                    Transform::identity(),
                    TextPaint::try_fill(red.into()).unwrap(),
                    &glyphs,
                    tiny_ink,
                )
                .unwrap(),
            );
        });
        let cases = [
            (
                geometry_scene(Geometry::ExtremeFiniteFactors, graph, red),
                Reason::UnrepresentableTransform,
            ),
            (
                // Source/native displacement now refuses before pair retention.
                geometry_scene(Geometry::CollapsedUnitEdge, graph, red),
                Reason::InsufficientNativeCapacity,
            ),
            (
                // A smaller displacement reaches the concrete alias receiver.
                scene(graph, |scene| {
                    let origin = 2.0_f64.powi(24);
                    scene.transform(Transform::translation(-origin, 0.0).unwrap(), |scene| {
                        scene.fill(Rect::try_new(origin, 0.0, 0.125, 16.0).unwrap(), red);
                    });
                }),
                Reason::UnrepresentablePath { element_index: 1 },
            ),
            (
                geometry_scene(Geometry::TinyLocalEdges, graph, red),
                Reason::UnrepresentablePath { element_index: 1 },
            ),
            (tiny_stroke, Reason::UnrepresentableStroke),
            (tiny_blur, Reason::UnrepresentableBlur),
            // The empty source glyph0 is skipped, but diagnostic indices stay
            // in the original input rather than the shortened native stream.
            (tiny_glyph, Reason::UnrepresentableGlyph { glyph_index: 1 }),
            (
                scene(graph, |scene| {
                    scene.transform(
                        Transform::try_new([
                            1.0,
                            1.0,
                            1.0,
                            f64::from_bits(1.0_f64.to_bits() + 1),
                            0.0,
                            0.0,
                        ])
                        .unwrap(),
                        |scene| {
                            scene.fill(bounds(), red);
                        },
                    );
                }),
                Reason::InsufficientNativeCapacity,
            ),
            (
                scene(graph, |scene| {
                    scene.transform(
                        Transform::scale(2.0_f64.powi(-127), 1.0).unwrap(),
                        |scene| {
                            scene.fill(bounds(), red);
                        },
                    );
                }),
                Reason::InsufficientNativeCapacity,
            ),
        ];
        let baseline = scene(graph, |scene| {
            scene.fill(bounds(), blue);
        });
        let retry = scene(graph, |scene| {
            scene.fill(bounds(), red);
        });
        for (target, reason) in cases {
            let before = render_read(&mut renderer, &mut surface, &baseline, graph);
            let stats = renderer.stats();
            let error =
                pollster::block_on(renderer.render(&mut surface, &target, Parameters::default()))
                    .unwrap_err();
            assert_contextual_refusal(&error, reason);
            assert_eq!(renderer.stats(), stats);
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
            let image = render_read(&mut renderer, &mut surface, &retry, graph);
            assert_pixel(
                &image,
                16,
                4,
                [255, 0, 0, if graph { 128 } else { 255 }],
                "fresh valid retry",
            );
        }
    }
}

fn decimal_curve() -> surgeist_render::Shape {
    use surgeist_render::{Path, Point, Shape};
    let mut path = Path::new();
    path.move_to(Point::try_new(2.1, 2.1).unwrap())
        .line_to(Point::try_new(29.9, 2.1).unwrap())
        .quad_to(
            Point::try_new(30.2, 2.4).unwrap(),
            Point::try_new(29.9, 13.8).unwrap(),
        )
        .cubic_to(
            Point::try_new(24.4, 15.1).unwrap(),
            Point::try_new(7.6, 15.1).unwrap(),
            Point::try_new(2.1, 13.8).unwrap(),
        )
        .close();
    Shape::path(path)
}

#[test]
fn ordinary_decimal_fill_stroke_clip_image_and_blur_receivers_keep_interior_coverage() {
    use surgeist_render::{ImageFit, LineJoin, Path, Point, Radii, Shadow, Shape, Stroke};
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let rounded = Shape::try_rounded_rect(
        Rect::try_new(2.1, 2.1, 27.8, 11.8).unwrap(),
        Radii::try_all(2.3).unwrap(),
    )
    .unwrap();
    let mut line = Path::new();
    line.move_to(Point::try_new(2.1, 8.1).unwrap())
        .line_to(Point::try_new(29.9, 8.1).unwrap());
    let image = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 0, 0, 255]).unwrap();
    let (mut renderer, mut surface) = profiled_renderer(RenderProfile::Summary { timing: false });
    for graph in [false, true] {
        let targets = [
            scene(graph, |scene| {
                scene.fill(decimal_curve(), red);
            }),
            scene(graph, |scene| {
                scene.fill(rounded.clone(), red);
            }),
            scene(graph, |scene| {
                scene.stroke(
                    Shape::path(line.clone()),
                    Stroke::try_new(4.2).unwrap().join(LineJoin::Round),
                    red,
                );
            }),
            scene(graph, |scene| {
                // The bottom band spans y=7.5..11.7, covering pixel(16,8).
                // Rect bounds do not use the round join's unused MAX miter;
                // the actual native stroke receiver is shared with paths.
                scene.stroke(
                    Rect::try_new(2.1, 6.6, 27.8, 3.0).unwrap(),
                    Stroke::try_new(4.2)
                        .unwrap()
                        .join(LineJoin::Round)
                        .try_miter_limit(f64::MAX)
                        .unwrap(),
                    red,
                );
            }),
            scene(graph, |scene| {
                scene.clip(rounded.clone(), |scene| {
                    scene.fill(bounds(), red);
                });
            }),
            scene(graph, |scene| {
                scene.image(
                    image.clone(),
                    Rect::try_new(2.1, 2.1, 27.8, 11.8).unwrap(),
                    ImageFit::Stretch,
                );
            }),
            scene(graph, |scene| {
                scene.shadow(
                    bounds(),
                    Shadow::try_new(Point::try_new(0.0, 0.0).unwrap(), 1.1, 0.0, red).unwrap(),
                );
            }),
        ];
        for (index, target) in targets.into_iter().enumerate() {
            let label = [
                "curve",
                "rounded",
                "path stroke",
                "rect stroke unused miter",
                "clip",
                "image",
                "blur",
            ][index];
            eprintln!("normal receiver case {index} ({label}), graph={graph}");
            let image = render_read(&mut renderer, &mut surface, &target, graph);
            // These samples lie well inside every shape and far from the blur
            // edge; nonbinary coordinate rounding cannot erase this coverage.
            assert_pixel(
                &image,
                16,
                8,
                [255, 0, 0, if graph { 128 } else { 255 }],
                "ordinary receiving path interior",
            );
            if index != 6 {
                assert_pixel(&image, 0, 0, [0, 0, 0, 0], "outside ordinary shape");
            }
        }
    }
}

#[test]
fn prepared_and_native_glyph_receivers_accept_decimal_placement_and_skip_empty_glyphs() {
    use surgeist_render::{
        FontData, FontRef, GlyphRunLimits, GlyphSynthesis, RenderAccountingContext, TextGlyph,
        TextInstance, TextPaint, TextRun, TextRunBounds,
    };
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let glyphs = [
        TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap(),
        TextGlyph::try_new(58, 2.0, 0.8, 1.0).unwrap(),
    ];
    let prepared = font
        .prepare_glyph_run(
            &TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap(),
            &glyphs,
            GlyphRunLimits::new(2, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let native_ink = prepared.bounds();
    let canonical_space = [TextGlyph::try_new(3, 0.0, 16.0, 16.0).unwrap()];
    let prepared_space = font
        .prepare_glyph_run(
            &TextInstance::try_new(16.0, vec![], GlyphSynthesis::none()).unwrap(),
            &canonical_space,
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let empty_bounds = prepared_space.bounds();
    assert_eq!(empty_bounds, TextRunBounds::empty());
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let (mut renderer, mut surface) = profiled_renderer(RenderProfile::Summary { timing: false });
    for graph in [false, true] {
        for native in [false, true] {
            let target = scene(graph, |scene| {
                scene.transform(Transform::translation(2.1, 4.1).unwrap(), |scene| {
                    let transform = Transform::try_new([8.0, 0.0, 4.0, 4.0, 0.0, 0.0]).unwrap();
                    let paint = TextPaint::try_fill(red.into()).unwrap();
                    if native {
                        scene.text_run(
                            TextRun::try_new(
                                FontRef::new(2112).with_data(font.clone()),
                                1.0,
                                transform,
                                paint,
                                &glyphs,
                                native_ink,
                            )
                            .unwrap(),
                        );
                    } else {
                        scene.prepared_glyph_run(prepared.clone(), transform, paint);
                    }
                });
            });
            let image = render_read(&mut renderer, &mut surface, &target, graph);
            for x in [8, 24] {
                assert_pixel(
                    &image,
                    x,
                    6,
                    [255, 0, 0, if graph { 128 } else { 255 }],
                    "decimal glyph placement",
                );
            }
            assert_pixel(&image, 16, 6, [0, 0, 0, 0], "between two glyphs");
        }
        let empty = [TextGlyph::try_new(3, f64::MAX, f64::MAX, 16.0).unwrap()];
        let target = scene(graph, |scene| {
            scene.fill(bounds(), blue);
            scene.text_run(
                TextRun::try_new(
                    FontRef::new(2113).with_data(font.clone()),
                    16.0,
                    Transform::identity(),
                    TextPaint::try_fill(red.into()).unwrap(),
                    &empty,
                    empty_bounds,
                )
                .unwrap(),
            );
        });
        let image = render_read(&mut renderer, &mut surface, &target, graph);
        assert_pixel(
            &image,
            16,
            4,
            [0, 0, 255, if graph { 128 } else { 255 }],
            "empty glyph has no required native placement",
        );
    }
}

#[test]
fn empty_point_only_singular_and_eliminated_geometry_keep_their_unused_inputs_unrequired() {
    use surgeist_render::{Path, Point, Shape};
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let mut point = Path::new();
    point.move_to(Point::try_new(f64::MAX, f64::MAX).unwrap());
    let (mut renderer, mut surface) = profiled_renderer(RenderProfile::Summary { timing: false });
    for graph in [false, true] {
        let target = scene(graph, |scene| {
            scene.fill(bounds(), blue);
            scene.fill(Shape::path(Path::new()), red);
            scene.fill(Shape::path(point.clone()), red);
            scene.transform(Transform::scale(0.0, 1.0).unwrap(), |scene| {
                scene.fill(bounds(), red);
            });
            scene.layer(Layer::new().try_opacity(0.0).unwrap(), |scene| {
                scene.transform(
                    Transform::scale(2.0_f64.powi(-127), 1.0).unwrap(),
                    |scene| {
                        scene.fill(bounds(), red);
                    },
                );
            });
            scene.clip(Rect::try_new(0.0, 0.0, 0.0, 0.0).unwrap(), |scene| {
                let origin = 2.0_f64.powi(24);
                scene.transform(Transform::translation(-origin, 0.0).unwrap(), |scene| {
                    scene.fill(Rect::try_new(origin, 0.0, 1.0, 16.0).unwrap(), red);
                });
            });
        });
        let image = render_read(&mut renderer, &mut surface, &target, graph);
        for x in [0, 16, 31] {
            assert_pixel(
                &image,
                x,
                4,
                [0, 0, 255, if graph { 128 } else { 255 }],
                "semantically absent geometry preserves blue",
            );
        }
    }
}

#[test]
fn receiving_profiles_preserve_values_custody_and_actual_scalar_cap_refusal_with_retry() {
    use surgeist_render::{
        RenderAccountingError, RenderLimits, RenderPhase, RenderPhaseFilter, RenderQuantity,
        RenderRefusalReason,
    };
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let baseline = scene(false, |scene| {
        scene.fill(bounds(), blue);
    });
    let retry = scene(false, |scene| {
        scene.fill(bounds(), red);
    });
    let ordinary = scene(false, |scene| {
        for _ in 0..4 {
            scene.fill(decimal_curve(), red);
        }
    });
    let (mut renderer, mut surface) = profiled_renderer(RenderProfile::Summary { timing: false });
    let before = render_read(&mut renderer, &mut surface, &baseline, false);
    let baseline_report = renderer.take_last_attempt().unwrap();
    let expected_pixels = render_read(&mut renderer, &mut surface, &ordinary, false);
    let full = renderer.take_last_attempt().unwrap();
    let expected_usage = full.attempt().unwrap().usage();
    assert_pixel(
        &expected_pixels,
        16,
        8,
        [255, 0, 0, 255],
        "ordinary decimal receiving calibration",
    );
    let cap = expected_usage.work(RenderWork::CoordinateVisits) - 1;
    assert!(
        baseline_report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::CoordinateVisits)
            < cap
    );
    drop(surface);
    drop(renderer);
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 2,
            filter: RenderPhaseFilter::Phase(RenderPhase::Encode),
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
    ] {
        let timed = matches!(
            profile,
            RenderProfile::Summary { timing: true } | RenderProfile::Detailed { timing: true, .. }
        );
        let (mut renderer, mut surface) = profiled_renderer(profile);
        assert_eq!(
            render_read(&mut renderer, &mut surface, &baseline, false),
            before
        );
        assert_eq!(
            render_read(&mut renderer, &mut surface, &ordinary, false),
            expected_pixels
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.cleanup_fault(), None);
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.usage(), expected_usage);
        assert_eq!(attempt.elapsed().is_some(), timed);
        if let RenderProfile::Detailed { event_limit, .. } = profile {
            assert!(attempt.events().count() <= event_limit);
            if event_limit == 0 {
                assert!(attempt.dropped_events() > 0);
            }
        }
        drop(surface);
        drop(renderer);
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, cap)
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
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        assert_eq!(
            render_read(&mut renderer, &mut surface, &baseline, false),
            before
        );
        let successful = renderer.stats();
        let error =
            pollster::block_on(renderer.render(&mut surface, &ordinary, Parameters::default()))
                .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected actual receiving scalar work cap")
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CoordinateVisits)
        );
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: cap });
        assert!(refusal.current().checked_add(refusal.amount()).unwrap() > cap);
        assert!(error.geometry_realization_diagnostic().is_none());
        assert_eq!(renderer.stats(), successful);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.cleanup_fault(), None);
        assert!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits)
                <= cap
        );
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::Submissions),
            0
        );
        let image = render_read(&mut renderer, &mut surface, &retry, false);
        assert_pixel(
            &image,
            16,
            4,
            [255, 0, 0, 255],
            "fresh valid retry under same scalar cap",
        );
    }
}
