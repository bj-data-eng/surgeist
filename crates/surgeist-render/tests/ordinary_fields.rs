#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, Gradient, GradientStop, ImageBuffer, Options, Parameters, Point, Rect, RenderRoute,
    Renderer, Scene, Size, Transform,
};

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}

fn stops(values: &[(f64, Color)]) -> Vec<GradientStop> {
    values
        .iter()
        .map(|&(position, color)| GradientStop::try_new(position, color).unwrap())
        .collect()
}

fn point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
    let actual = pixel(image, x, y);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "pixel ({x},{y}): actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

fn field_scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
    use surgeist_render::{Image, Layer, ResolvedLayerAlphaMask};

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

fn read(
    renderer: &mut Renderer,
    surface: &mut surgeist_render::Surface,
    scene: &Scene,
    graph: bool,
) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
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

fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    image.rgba()[index..index + 4].try_into().unwrap()
}

#[test]
fn linear_gradient_preserves_its_local_field_under_shear() {
    let gradient = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(8.0, 0.0).unwrap(),
        vec![
            GradientStop::try_new(0.0, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()).unwrap(),
            GradientStop::try_new(1.0, Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()).unwrap(),
        ],
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.transform(
        Transform::try_new([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
        |scene| {
            scene.fill(Rect::try_new(0.0, 0.0, 8.0, 8.0).unwrap(), gradient);
        },
    );
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(16.0, 8.0).unwrap(), 1.0))
            .unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::DirectVello));
    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();

    // T(x,y)=(x+y,y), so T^-1(x',y')=(x'-y',y'). The local line
    // (0,0)->(8,0) therefore has q=(x'-y')/8. At these fully covered
    // integer physical samples q is 1/4, 1/2, 3/4, and 1/4. The opaque
    // endpoints have premultiplied RGBA (1-q,0,q,1), giving these rounded
    // bytes independently of any transformed endpoint or native paint map.
    let samples = [
        (4, 2, [191, 0, 64, 255]),
        (6, 2, [128, 0, 128, 255]),
        (8, 2, [64, 0, 191, 255]),
        (6, 4, [191, 0, 64, 255]),
    ];
    for &(x, y, expected) in &samples {
        println!(
            "sheared linear ({x},{y}): actual {:?}, expected {expected:?}",
            pixel(&image, x, y)
        );
    }
    for (x, y, expected) in samples {
        let actual = pixel(&image, x, y);
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(actual, expected)| actual.abs_diff(expected) <= 2),
            "sheared linear ({x},{y}): actual {actual:?}, expected {expected:?}, tolerance 2"
        );
    }
}

#[test]
fn finite_f64_positions_and_elliptical_axes_are_preserved_by_the_public_model() {
    use surgeist_render::{ErrorCode, RepeatingGradient};

    let red = rgba(1.0, 0.0, 0.0, 1.0);
    let blue = rgba(0.0, 0.0, 1.0, 1.0);
    let precise = f64::from_bits(1.0_f64.to_bits() + 1);
    let records = stops(&[
        (-f64::MAX, red),
        (-0.0, red),
        (precise, blue),
        (f64::MAX, blue),
    ]);
    let linear = Gradient::try_linear(
        point(0.0, 0.0),
        point(f64::MIN_POSITIVE, 0.0),
        records.clone(),
    )
    .unwrap();
    assert_eq!(linear.stops(), records);
    assert_eq!(linear.stops()[1].position().to_bits(), (-0.0_f64).to_bits());
    assert_eq!(linear.stops()[2].position().to_bits(), precise.to_bits());
    assert_eq!(linear.clone(), linear);
    let radial = Gradient::try_radial(point(3.0, 4.0), 8.0, 2.0, records.clone()).unwrap();
    assert_eq!(radial.radial_geometry(), Some((point(3.0, 4.0), 8.0, 2.0)));
    assert_eq!(radial.stops(), records);
    // Ordinary fields do not need a finite source span. Repetition does.
    assert!(RepeatingGradient::try_linear(point(0.0, 0.0), point(1.0, 0.0), records).is_err());
    let shared = stops(&[(-1.0, red), (2.0, blue)]);
    let repeating =
        RepeatingGradient::try_linear(point(0.0, 0.0), point(8.0, 0.0), shared.clone()).unwrap();
    assert_eq!(repeating.stops(), shared);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = GradientStop::try_new(value, red).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }
    for (x, y) in [(0.0, 2.0), (2.0, 0.0), (-1.0, 2.0), (2.0, f64::INFINITY)] {
        assert!(Gradient::try_radial(point(0.0, 0.0), x, y, shared.clone()).is_err());
    }
    let descending = stops(&[(1.0, red), (0.0, blue)]);
    assert!(Gradient::try_linear(point(0.0, 0.0), point(8.0, 0.0), descending.clone()).is_err());
    assert!(Gradient::try_radial(point(0.0, 0.0), 8.0, 2.0, descending).is_err());
    assert!(Gradient::try_sweep(point(0.0, 0.0), shared).is_err());
}

#[test]
fn outside_stops_ellipses_steps_and_overflow_safe_fields_execute_on_both_gpu_routes() {
    use surgeist_render::EffectQualityPolicy;

    let red = rgba(1.0, 0.0, 0.0, 1.0);
    let blue = rgba(0.0, 0.0, 1.0, 1.0);
    let yellow = rgba(1.0, 1.0, 0.0, 1.0);
    let line = |a, b| {
        Gradient::try_linear(
            point(16.0, 0.0),
            point(20.0, 0.0),
            stops(&[(a, red), (b, blue)]),
        )
        .unwrap()
    };
    let cases = [
        (
            line(-1.0, 2.0),
            vec![
                (4, 4, [255, 0, 0, 255]),
                (16, 4, [170, 0, 85, 255]),
                (18, 4, [128, 0, 128, 255]),
                (20, 4, [85, 0, 170, 255]),
                (28, 4, [0, 0, 255, 255]),
            ],
        ),
        (
            // Adding one full stop interval (3) shifts Pad, not a repeat phase.
            line(2.0, 5.0),
            vec![(18, 4, [255, 0, 0, 255]), (28, 4, [170, 0, 85, 255])],
        ),
        (
            Gradient::try_radial(
                point(16.0, 8.0),
                8.0,
                4.0,
                stops(&[(0.0, red), (1.0, blue)]),
            )
            .unwrap(),
            // q=sqrt(((x-16)/8)^2+((y-8)/4)^2). Matching horizontal
            // and vertical half/three-quarter rays must have matching colors.
            vec![
                (16, 8, [255, 0, 0, 255]),
                (20, 8, [128, 0, 128, 255]),
                (16, 10, [128, 0, 128, 255]),
                (22, 8, [64, 0, 191, 255]),
                (16, 11, [64, 0, 191, 255]),
                (28, 8, [0, 0, 255, 255]),
            ],
        ),
        (
            Gradient::try_radial(
                point(16.0, 8.0),
                8.0,
                4.0,
                stops(&[(-0.5, red), (1.0, yellow)]),
            )
            .unwrap(),
            // Negative red contributes at q=0: t=(0+.5)/1.5=1/3.
            // At q=.5, t=2/3. Clamping away negative stops would fail both.
            vec![
                (16, 8, [255, 85, 0, 255]),
                (20, 8, [255, 170, 0, 255]),
                (16, 10, [255, 170, 0, 255]),
                (28, 8, [255, 255, 0, 255]),
            ],
        ),
        (
            Gradient::try_linear(
                point(0.0, 0.0),
                point(16.0, 0.0),
                stops(&[(0.5, red), (0.5, yellow), (0.5, blue)]),
            )
            .unwrap(),
            // A real zero span selects the first color below q=.5 and
            // the last color at and above it, without an average/epsilon ramp.
            vec![
                (7, 4, [255, 0, 0, 255]),
                (8, 4, [0, 0, 255, 255]),
                (9, 4, [0, 0, 255, 255]),
            ],
        ),
        (
            Gradient::try_linear(
                point(0.0, 0.0),
                point(8.0, 0.0),
                stops(&[(0.0, rgba(1.0, 0.0, 0.0, 0.5)), (1.0, blue)]),
            )
            .unwrap(),
            // q=.5 gives premultiplied (.25,0,.5,.75), hence straight
            // (1/3,0,2/3,.75), independently of opaque-ramp expectations.
            vec![(4, 4, [85, 0, 170, 191])],
        ),
    ];
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    for graph in [false, true] {
        for (gradient, samples) in &cases {
            let scene = field_scene(graph, |scene| {
                scene.fill(bounds(), gradient.clone());
            });
            let image = read(&mut renderer, &mut surface, &scene, graph);
            for &(x, y, mut expected) in samples {
                if graph {
                    // The mask's normalized alpha is 128/255. Preserve
                    // straight RGB while rounding the premultiplied alpha.
                    expected[3] = u8::try_from((u16::from(expected[3]) * 128 + 127) / 255).unwrap();
                }
                assert_pixel(&image, x, y, expected, if graph { 4 } else { 2 });
            }
        }
        // 2^-1022 * MAX is just below 4. Although MAX-(-MAX) overflows,
        // the two physical endpoint records are finite near x=-4 and x=4.
        // Translation by 16 exposes quarter/mid/three-quarter colors at 14/16/18.
        let overflow = Gradient::try_linear(
            point(0.0, 0.0),
            point(f64::MIN_POSITIVE, 0.0),
            stops(&[(-f64::MAX, red), (f64::MAX, blue)]),
        )
        .unwrap();
        let scene = field_scene(graph, |scene| {
            scene.transform(Transform::translation(16.0, 0.0).unwrap(), |scene| {
                scene.fill(Rect::try_new(-16.0, 0.0, 32.0, 16.0).unwrap(), overflow);
            });
        });
        let image = read(&mut renderer, &mut surface, &scene, graph);
        let alpha = if graph { 128 } else { 255 };
        for (x, r, b) in [
            (10, 255, 0),
            (14, 191, 64),
            (16, 128, 128),
            (18, 64, 191),
            (22, 0, 255),
        ] {
            assert_pixel(&image, x, 4, [r, 0, b, alpha], if graph { 4 } else { 2 });
        }
    }
}

#[test]
fn distinct_f64_stop_collapse_refuses_without_publication_and_allows_retry() {
    use surgeist_render::{
        ErrorCode, GradientRealizationError, RenderAccountingOptions, RenderAttemptStatus,
        RenderProfile, RenderWork,
    };

    let red = rgba(1.0, 0.0, 0.0, 1.0);
    let blue = rgba(0.0, 0.0, 1.0, 1.0);
    let after_half = f64::from_bits(0.5_f64.to_bits() + 1);
    assert_ne!(0.5, after_half);
    assert_eq!(0.5_f32, after_half as f32);
    let rejected = Gradient::try_linear(
        point(0.0, 0.0),
        point(16.0, 0.0),
        stops(&[(0.0, red), (0.5, red), (after_half, blue), (1.0, blue)]),
    )
    .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(Options::default().with_accounting(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    let baseline = field_scene(false, |scene| {
        scene.fill(bounds(), rgba(0.0, 1.0, 0.0, 1.0));
    });
    let before = read(&mut renderer, &mut surface, &baseline, false);
    let successful = renderer.stats();
    let scene = field_scene(false, |scene| {
        scene.fill(bounds(), rejected);
    });
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::GradientRealization);
    assert_eq!(
        error.gradient_realization_diagnostic(),
        Some(&GradientRealizationError::UnrepresentableStop { stop_index: 2 })
    );
    assert_eq!(renderer.stats(), successful);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(ErrorCode::GradientRealization)
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
    let retry = Gradient::try_linear(
        point(0.0, 0.0),
        point(16.0, 0.0),
        stops(&[(0.0, red), (1.0, blue)]),
    )
    .unwrap();
    let scene = field_scene(false, |scene| {
        scene.fill(bounds(), retry);
    });
    let image = read(&mut renderer, &mut surface, &scene, false);
    assert_pixel(&image, 4, 4, [191, 0, 64, 255], 2);
}

#[test]
fn outside_fields_follow_transformed_strokes_and_both_glyph_routes_at_final_scale() {
    use surgeist_render::{
        EffectQualityPolicy, FontData, FontRef, GlyphRunLimits, GlyphSynthesis,
        RenderAccountingContext, Shape, Stroke, TextGlyph, TextInstance, TextPaint, TextRun,
        TextRunBounds,
    };

    let red = rgba(1.0, 0.0, 0.0, 0.5);
    let blue = rgba(0.0, 0.0, 1.0, 1.0);
    let field = |length| {
        Gradient::try_linear(
            point(0.0, 0.0),
            point(length, 0.0),
            stops(&[(-1.0, red), (2.0, blue)]),
        )
        .unwrap()
    };
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    // Ahem X is a one-em square, glyph58, with baseline .8. Both paths
    // consume one run-local field across the two disjoint glyphs.
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
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    for scale in [1_u32, 2] {
        let mut surface = pollster::block_on(
            renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), f64::from(scale)),
        )
        .unwrap();
        for graph in [false, true] {
            for consumer in 0..3 {
                let scene = field_scene(graph, |scene| {
                    scene.transform(Transform::translation(2.0, 4.0).unwrap(), |scene| {
                        if consumer == 0 {
                            scene.transform(
                                Transform::try_new([2.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
                                |scene| {
                                    scene.stroke(
                                        Shape::rect(Rect::try_new(0.0, 1.0, 12.0, 2.0).unwrap()),
                                        Stroke::try_new(2.0).unwrap(),
                                        field(4.0),
                                    );
                                },
                            );
                        } else {
                            let transform =
                                Transform::try_new([8.0, 0.0, 4.0, 4.0, 0.0, 0.0]).unwrap();
                            let paint = TextPaint::try_fill(field(1.0).into()).unwrap();
                            if consumer == 1 {
                                scene.prepared_glyph_run(prepared.clone(), transform, paint);
                            } else {
                                scene.text_run(
                                    TextRun::try_new(
                                        FontRef::new(2093).with_data(font.clone()),
                                        1.0,
                                        transform,
                                        paint,
                                        &glyphs,
                                        TextRunBounds::try_ink(
                                            Rect::try_new(0.0, 0.0, 3.0, 1.0).unwrap(),
                                        )
                                        .unwrap(),
                                    )
                                    .unwrap(),
                                );
                            }
                        }
                    });
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                // Inverse composed maps at logical (8,6) give stroke x=2
                // on its length4 line and glyph x=.5 on its length1 line.
                // Both have q=.5, hence t=(q+1)/3=.5. Premultiplied
                // red(.5 alpha)/blue gives (.25,0,.5,.75), or straight
                // [85,0,170,191]. At (24,6), q>2 selects Pad blue.
                assert_pixel(
                    &image,
                    8 * scale,
                    6 * scale,
                    [85, 0, 170, if graph { 96 } else { 191 }],
                    if graph { 4 } else { 2 },
                );
                assert_pixel(
                    &image,
                    24 * scale,
                    6 * scale,
                    [0, 0, 255, if graph { 128 } else { 255 }],
                    if graph { 4 } else { 2 },
                );
                if consumer != 0 {
                    assert_pixel(&image, 16 * scale, 6 * scale, [0, 0, 0, 0], 0);
                }
            }
        }
    }
}

#[cfg(feature = "render-paint")]
mod paint {
    use super::*;
    use surgeist_layout as layout;
    use surgeist_paint as producer;
    use surgeist_render::{
        EffectQualityPolicy, Error, ErrorCode, PaintGradientError, RenderAccountingContext,
        prepare_linear_gradient, prepare_radial_gradient,
    };

    fn options() -> producer::PaintOptions {
        producer::PaintOptions::new(
            producer::PaintLimits::unlimited(),
            producer::ProfileMode::Off,
        )
    }

    fn position(length: f64) -> producer::GradientPosition {
        producer::GradientPosition {
            length,
            fraction: 0.0,
        }
    }

    fn at(length: f64, color: Color) -> producer::GradientStopItem<Color> {
        producer::GradientStopItem::Stop {
            payload: color,
            positions: producer::GradientStopPositions::One(position(length)),
        }
    }

    fn line(length: f64) -> producer::LinearGradientLine {
        producer::resolve_linear_gradient(
            layout::Size::new(length, 16.0),
            producer::LinearGradientDirection::Right,
            options(),
        )
        .result
        .unwrap()
    }

    fn ellipse(x: f64, y: f64) -> producer::RadialGradientGeometry {
        producer::resolve_radial_gradient(
            layout::Size::new(32.0, 16.0),
            Some(layout::Point::new(16.0, 8.0)),
            producer::RadialGradientSize::EllipseRadii(layout::Size::new(position(x), position(y))),
            options(),
        )
        .result
        .unwrap()
    }

    fn used(
        basis: f64,
        source: &[producer::GradientStopItem<Color>],
    ) -> producer::GradientStops<'_, Color> {
        producer::resolve_gradient_stops(basis, source, options())
            .result
            .unwrap()
    }

    fn diagnostic(error: &Error) -> &PaintGradientError {
        assert_eq!(error.code(), ErrorCode::PaintGradient);
        error.paint_gradient_diagnostic().unwrap()
    }

    #[test]
    fn authentic_paint_line_and_ellipse_outlive_their_sources_and_execute_as_ordinary_fields() {
        let red = rgba(1.0, 0.0, 0.0, 1.0);
        let blue = rgba(0.0, 0.0, 1.0, 1.0);
        let yellow = rgba(1.0, 1.0, 0.0, 1.0);
        let linear = {
            let geometry = line(8.0);
            let source = vec![
                producer::GradientStopItem::Stop {
                    payload: red,
                    positions: producer::GradientStopPositions::Two(position(-8.0), position(0.0)),
                },
                at(16.0, blue),
            ];
            let stops = used(8.0, &source);
            let mut context = RenderAccountingContext::default();
            prepare_linear_gradient(&geometry, &stops, &mut context).unwrap()
        };
        assert_eq!(
            linear.linear_points(),
            Some((point(0.0, 8.0), point(8.0, 8.0)))
        );
        assert_eq!(
            linear.stops(),
            stops(&[(-1.0, red), (0.0, red), (2.0, blue)])
        );
        let radial = {
            let geometry = ellipse(8.0, 4.0);
            let source = vec![at(-4.0, red), at(8.0, yellow)];
            let stops = used(8.0, &source);
            let mut context = RenderAccountingContext::default();
            prepare_radial_gradient(&geometry, &stops, &mut context).unwrap()
        };
        assert_eq!(radial.radial_geometry(), Some((point(16.0, 8.0), 8.0, 4.0)));
        assert_eq!(radial.stops(), stops(&[(-0.5, red), (1.0, yellow)]));
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        for graph in [false, true] {
            let alpha = if graph { 128 } else { 255 };
            let scene = field_scene(graph, |scene| {
                scene.fill(bounds(), linear.clone());
            });
            let image = read(&mut renderer, &mut surface, &scene, graph);
            // Two red records retain their flat segment. For q=x/8 in [0,2],
            // interpolation from red0 to blue2 gives t=x/16; afterward Pad blue.
            for (x, r, b) in [(4, 191, 64), (8, 128, 128), (12, 64, 191), (24, 0, 255)] {
                assert_pixel(&image, x, 4, [r, 0, b, alpha], if graph { 4 } else { 2 });
            }
            let scene = field_scene(graph, |scene| {
                scene.fill(bounds(), radial.clone());
            });
            let image = read(&mut renderer, &mut surface, &scene, graph);
            for (x, y, g) in [(16, 8, 85), (20, 8, 170), (16, 10, 170), (28, 8, 255)] {
                assert_pixel(&image, x, y, [255, g, 0, alpha], if graph { 4 } else { 2 });
            }
        }
    }

    #[test]
    fn ordinary_paint_adapters_accept_hints_and_refuse_basis_or_two_position_quotient_loss() {
        let red = rgba(1.0, 0.0, 0.0, 1.0);
        let blue = rgba(0.0, 0.0, 1.0, 1.0);
        let source = [
            at(0.0, red),
            producer::GradientStopItem::Hint {
                position: position(4.0),
            },
            at(8.0, blue),
        ];
        let mut context = RenderAccountingContext::default();
        let changed_basis = f64::from_bits(8.0_f64.to_bits() + 1);
        let mismatch = used(changed_basis, &source);
        for error in [
            prepare_linear_gradient(&line(8.0), &mismatch, &mut context).unwrap_err(),
            prepare_radial_gradient(&ellipse(8.0, 4.0), &mismatch, &mut context).unwrap_err(),
        ] {
            assert_eq!(
                diagnostic(&error),
                &PaintGradientError::BasisMismatch {
                    expected: 8.0,
                    actual: changed_basis
                }
            );
        }
        let hinted = used(8.0, &source);
        for gradient in [
            prepare_linear_gradient(&line(8.0), &hinted, &mut context).unwrap(),
            prepare_radial_gradient(&ellipse(8.0, 4.0), &hinted, &mut context).unwrap(),
        ] {
            assert_eq!(gradient.stops().len(), 2);
            assert_eq!(gradient.stops()[0].position(), 0.0);
            assert_eq!(gradient.stops()[0].hint_after().unwrap().position(), 0.5);
            assert_eq!(gradient.stops()[1].position(), 1.0);
            assert!(gradient.stops()[1].hint_after().is_none());
        }
        // These adjacent physical f64 positions share the same quotient by 3.
        // Keeping their Two-position source identity requires refusing index1.
        let first = f64::from_bits(1.5_f64.to_bits() + 1);
        let second = f64::from_bits(1.5_f64.to_bits() + 2);
        assert_ne!(first, second);
        assert_eq!(first / 3.0, second / 3.0);
        let source = [
            at(0.0, red),
            producer::GradientStopItem::Stop {
                payload: blue,
                positions: producer::GradientStopPositions::Two(position(first), position(second)),
            },
        ];
        let collapsed = used(3.0, &source);
        for error in [
            prepare_linear_gradient(&line(3.0), &collapsed, &mut context).unwrap_err(),
            prepare_radial_gradient(&ellipse(3.0, 1.0), &collapsed, &mut context).unwrap_err(),
        ] {
            assert_eq!(
                diagnostic(&error),
                &PaintGradientError::UnrepresentablePosition {
                    source_index: 1,
                    position_index: 1
                }
            );
        }
        let geometry = ellipse(0.0, 4.0);
        let source = [at(0.0, red), at(8.0, blue)];
        let error =
            prepare_radial_gradient(&geometry, &used(8.0, &source), &mut context).unwrap_err();
        assert_eq!(
            diagnostic(&error),
            &PaintGradientError::UnsupportedRadialCase {
                case: geometry.case()
            }
        );
    }
}

#[test]
fn tiny_radial_step_keeps_the_first_color_far_below_its_threshold() {
    use surgeist_render::EffectQualityPolicy;
    let red = rgba(1.0, 0.0, 0.0, 1.0);
    let blue = rgba(0.0, 0.0, 1.0, 1.0);
    let gradient = Gradient::try_radial(
        point(0.0, 0.0),
        1.0e-18,
        1.0e-18,
        stops(&[(1.0e30, red), (1.0e30, blue)]),
    )
    .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    for graph in [false, true] {
        let scene = field_scene(graph, |scene| {
            scene.fill(bounds(), gradient.clone());
        });
        let image = read(&mut renderer, &mut surface, &scene, graph);
        // q=hypot(31,4)/1e-18 < 3.2e19, eleven orders of magnitude
        // below the 1e30 step. Native squared-norm overflow must not turn
        // this first-color region into the final color.
        assert_pixel(&image, 31, 4, [255, 0, 0, if graph { 128 } else { 255 }], 0);
    }
}
