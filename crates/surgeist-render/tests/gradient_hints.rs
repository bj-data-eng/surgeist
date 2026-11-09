#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, EffectQualityPolicy, Gradient, GradientHint, GradientStop, Image, ImageBuffer, Layer,
    Options, Paint, Parameters, Point, Rect, RenderAccountingContext, RenderRoute, Renderer,
    RepeatingGradient, ResolvedLayerAlphaMask, Scene, Size, Surface, prepare_linear_gradient,
    prepare_radial_gradient, prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
};

fn options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64) -> paint::GradientPosition {
    paint::GradientPosition {
        length,
        fraction: 0.0,
    }
}

fn source(length: f64, alpha: f32) -> [paint::GradientStopItem<Color>; 3] {
    [
        paint::GradientStopItem::Stop {
            payload: Color::try_rgba(1.0, 0.0, 0.0, alpha).unwrap(),
            positions: paint::GradientStopPositions::One(position(0.0)),
        },
        paint::GradientStopItem::Hint {
            position: position(length / 4.0),
        },
        paint::GradientStopItem::Stop {
            payload: Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
            positions: paint::GradientStopPositions::One(position(length)),
        },
    ]
}

fn line(length: f64) -> paint::LinearGradientLine {
    paint::resolve_linear_gradient(
        layout::Size::new(length, 16.0),
        paint::LinearGradientDirection::Right,
        options(),
    )
    .result
    .expect("authentic Paint line preparation must succeed")
}

fn used(basis: f64, source: &[paint::GradientStopItem<Color>]) -> paint::GradientStops<'_, Color> {
    let used = paint::resolve_gradient_stops(basis, source, options())
        .result
        .expect("authentic Paint used-stop preparation must succeed");
    // Verify that the real producer retained the stimulus. The behavioral
    // assertion remains consumer acceptance followed by independently known pixels.
    assert!(
        used.items()
            .any(|item| matches!(item, paint::UsedGradientItem::Hint { .. }))
    );
    used
}

fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}

fn point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}

fn stop(position: f64, color: Color, hint: Option<f64>) -> GradientStop {
    let stop = GradientStop::try_new(position, color).unwrap();
    match hint {
        Some(hint) => stop.with_hint_after(GradientHint::try_new(hint).unwrap()),
        None => stop,
    }
}

fn hinted_line(hint: f64) -> Gradient {
    Gradient::try_linear(
        point(0.0, 0.0),
        point(16.0, 0.0),
        vec![
            stop(0.0, color(1.0, 0.0, 0.0, 1.0), Some(hint)),
            stop(1.0, color(0.0, 0.0, 1.0, 1.0), None),
        ],
    )
    .unwrap()
}

fn opaque_weight(weight: f64) -> [u8; 4] {
    [
        ((1.0 - weight) * 255.0).round() as u8,
        0,
        (weight * 255.0).round() as u8,
        255,
    ]
}

fn ordinary_line(alpha: f32) -> Gradient {
    let geometry = line(16.0);
    let source = source(16.0, alpha);
    let used = used(16.0, &source);
    let mut context = RenderAccountingContext::default();
    prepare_linear_gradient(&geometry, &used, &mut context)
        .expect("ordinary linear consumer must accept the authentic transition hint")
}

fn repeating_line(length: f64, alpha: f32) -> RepeatingGradient {
    let geometry = line(length);
    let source = source(length, alpha);
    let used = used(length, &source);
    let mut context = RenderAccountingContext::default();
    prepare_repeating_linear_gradient(&geometry, &used, &mut context)
        .expect("repeating linear consumer must accept the authentic transition hint")
}

fn rect() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn renderer_and_surface() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    (renderer, surface)
}

fn read(renderer: &mut Renderer, surface: &mut Surface, field: Paint, graph: bool) -> ImageBuffer {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new()
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, rect()).unwrap()),
            |scene| {
                scene.fill(rect(), field);
            },
        );
    } else {
        scene.fill(rect(), field);
    }
    read_scene(renderer, surface, &scene, graph)
}

fn read_scene(
    renderer: &mut Renderer,
    surface: &mut Surface,
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

fn draw_scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new()
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, rect()).unwrap()),
            draw,
        );
    } else {
        draw(&mut scene);
    }
    scene
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, mut expected: [u8; 4], graph: bool) {
    if graph {
        expected[3] = u8::try_from((u16::from(expected[3]) * 128 + 127) / 255).unwrap();
    }
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
    let tolerance = if graph { 3 } else { 2 };
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, e)| a.abs_diff(e) <= tolerance),
        "hint pixel ({x},{y}), graph={graph}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[test]
fn authentic_linear_hint_controls_half_and_three_quarter_premultiplied_colors() {
    // Each owned output survives the geometry, source array and context in
    // ordinary_line. No GPU is created until both consumers have accepted.
    let fields = [(1.0, ordinary_line(1.0)), (0.5, ordinary_line(0.5))];
    let (mut renderer, mut surface) = renderer_and_surface();
    for (alpha, field) in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone().into(), graph);
            // H=1/4 gives k=ln(.5)/ln(.25)=1/2. At integer x=4,
            // P=1/4 => w=1/2; at x=9, P=9/16 => w=3/4.
            // With alpha .5 red, the premultiplied results are (.25,0,.5,.75)
            // and (.125,0,.75,.875), hence straight ratios 1:2 and 1:6.
            let (half, three_quarters) = if alpha == 1.0 {
                ([128, 0, 128, 255], [64, 0, 191, 255])
            } else {
                ([85, 0, 170, 191], [36, 0, 219, 223])
            };
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 9, 4, three_quarters, graph);
        }
    }
}

#[test]
fn authentic_unequal_ellipse_hints_execute_for_ordinary_and_repeating_consumers() {
    let (ordinary, repeating) = {
        let geometry = paint::resolve_radial_gradient(
            layout::Size::new(32.0, 16.0),
            Some(layout::Point::new(8.0, 8.0)),
            paint::RadialGradientSize::EllipseRadii(layout::Size::new(
                position(16.0),
                position(8.0),
            )),
            options(),
        )
        .result
        .expect("authentic Paint ellipse preparation must succeed");
        let source = source(16.0, 1.0);
        let used = used(16.0, &source);
        let mut context = RenderAccountingContext::default();
        let ordinary = prepare_radial_gradient(&geometry, &used, &mut context)
            .expect("ordinary radial consumer must accept the authentic transition hint");
        let repeating = prepare_repeating_radial_gradient(&geometry, &used, &mut context)
            .expect("repeating radial consumer must accept the authentic transition hint");
        (ordinary, repeating)
    };
    let (mut renderer, mut surface) = renderer_and_surface();
    for graph in [false, true] {
        for (field, exterior) in [
            (Paint::from(ordinary.clone()), [0, 0, 255, 255]),
            (Paint::from(repeating.clone()), [128, 0, 128, 255]),
        ] {
            let image = read(&mut renderer, &mut surface, field, graph);
            // q=sqrt(((x-8)/16)^2+((y-8)/8)^2). The two quarter
            // rays share w=.5; q=9/16 gives w=.75. These are smooth
            // interior colors, with no assertion of an exact radial step tie.
            assert_pixel(&image, 12, 8, [128, 0, 128, 255], graph);
            assert_pixel(&image, 8, 10, [128, 0, 128, 255], graph);
            assert_pixel(&image, 17, 8, [64, 0, 191, 255], graph);
            // q=1.25 is Pad blue for ordinary, but repeats to P=.25.
            assert_pixel(&image, 28, 8, exterior, graph);
        }
    }
}

#[test]
fn repeating_hints_use_power_colors_when_resolved_and_stop_only_average_when_unresolved() {
    let fields = [
        (1.0, repeating_line(16.0, 1.0), repeating_line(1.0, 1.0)),
        (0.5, repeating_line(16.0, 0.5), repeating_line(1.0, 0.5)),
    ];
    let (mut renderer, mut surface) = renderer_and_surface();
    for (alpha, resolved, averaged) in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, resolved.clone().into(), graph);
            let (half, three_quarters) = if alpha == 1.0 {
                ([128, 0, 128, 255], [64, 0, 191, 255])
            } else {
                ([85, 0, 170, 191], [36, 0, 219, 223])
            };
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 9, 4, three_quarters, graph);
            assert_pixel(&image, 20, 4, half, graph);
            let image = read(&mut renderer, &mut surface, averaged.clone().into(), graph);
            // A one-device-pixel period is unresolved. Its specified average
            // is the adjacent-color trapezoid, excluding hints: w=.5. The
            // spatial integral of sqrt(P), w=2/3, would give different bytes.
            assert_pixel(&image, 4, 4, half, graph);
            assert_pixel(&image, 24, 12, half, graph);
        }
    }
}

#[test]
fn outgoing_hint_model_preserves_positions_and_reports_the_invalid_interval() {
    use surgeist_render::{ErrorCode, GradientHintError};

    let red = color(1.0, 0.0, 0.0, 1.0);
    let blue = color(0.0, 0.0, 1.0, 1.0);
    let precise = f64::from_bits(0.25_f64.to_bits() + 1);
    let first = stop(-1.0, red, Some(precise));
    assert_eq!(
        first.hint_after().unwrap().position().to_bits(),
        precise.to_bits()
    );
    let records = vec![first, stop(2.0, blue, None)];
    let gradient =
        Gradient::try_linear(point(0.0, 0.0), point(16.0, 0.0), records.clone()).unwrap();
    assert_eq!(gradient.stops(), records);
    assert_eq!(gradient.clone(), gradient);
    assert_eq!(
        RepeatingGradient::try_linear(point(0.0, 0.0), point(16.0, 0.0), records.clone())
            .unwrap()
            .stops(),
        records
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let error = GradientHint::try_new(value).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }
    for (records, expected) in [
        (
            vec![stop(0.0, red, None), stop(1.0, blue, Some(1.0))],
            GradientHintError::TrailingHint { stop_index: 1 },
        ),
        (
            vec![stop(0.0, red, Some(-0.25)), stop(1.0, blue, None)],
            GradientHintError::HintOutsideInterval { stop_index: 0 },
        ),
        (
            vec![stop(0.0, red, Some(1.25)), stop(1.0, blue, None)],
            GradientHintError::HintOutsideInterval { stop_index: 0 },
        ),
    ] {
        for error in [
            Gradient::try_linear(point(0.0, 0.0), point(16.0, 0.0), records.clone()).unwrap_err(),
            Gradient::try_radial(point(8.0, 8.0), 16.0, 8.0, records.clone()).unwrap_err(),
            RepeatingGradient::try_linear(point(0.0, 0.0), point(16.0, 0.0), records).unwrap_err(),
        ] {
            assert_eq!(error.code(), ErrorCode::GradientHint);
            assert_eq!(error.gradient_hint_diagnostic(), Some(&expected));
        }
    }
    let error = Gradient::try_sweep(
        point(0.0, 0.0),
        vec![stop(0.0, red, Some(0.25)), stop(1.0, blue, None)],
    )
    .unwrap_err();
    assert_eq!(
        error.gradient_hint_diagnostic(),
        Some(&GradientHintError::UnsupportedSweep { stop_index: 0 })
    );
}

#[test]
fn endpoint_hints_coincident_runs_and_midpoint_hints_keep_their_selected_colors() {
    let red = color(1.0, 0.0, 0.0, 1.0);
    let green = color(0.0, 1.0, 0.0, 1.0);
    let blue = color(0.0, 0.0, 1.0, 1.0);
    let yellow = color(1.0, 1.0, 0.0, 1.0);
    let gradient =
        |records| Gradient::try_linear(point(8.0, 0.0), point(24.0, 0.0), records).unwrap();
    let cases = [
        (
            gradient(vec![stop(0.0, red, Some(0.0)), stop(1.0, blue, None)]),
            vec![
                (4, [255, 0, 0, 255]),
                (8, [255, 0, 0, 255]),
                (9, [0, 0, 255, 255]),
                (23, [0, 0, 255, 255]),
                (24, [0, 0, 255, 255]),
                (28, [0, 0, 255, 255]),
            ],
        ),
        (
            gradient(vec![stop(0.0, red, Some(1.0)), stop(1.0, blue, None)]),
            vec![
                (4, [255, 0, 0, 255]),
                (8, [255, 0, 0, 255]),
                (9, [255, 0, 0, 255]),
                (23, [255, 0, 0, 255]),
                (24, [0, 0, 255, 255]),
                (28, [0, 0, 255, 255]),
            ],
        ),
        (
            gradient(vec![
                stop(0.0, red, Some(0.25)),
                stop(0.5, green, Some(0.5)),
                stop(0.5, blue, Some(0.75)),
                stop(1.0, yellow, None),
            ]),
            vec![
                (12, [128, 128, 0, 255]),
                (16, [0, 0, 255, 255]),
                (20, [128, 128, 128, 255]),
            ],
        ),
        (
            gradient(vec![
                stop(0.5, red, Some(0.5)),
                stop(0.5, green, Some(0.5)),
                stop(0.5, blue, None),
            ]),
            vec![
                (15, [255, 0, 0, 255]),
                (16, [0, 0, 255, 255]),
                (17, [0, 0, 255, 255]),
            ],
        ),
        (
            gradient(vec![stop(0.0, red, Some(0.5)), stop(1.0, blue, None)]),
            vec![
                (12, [191, 0, 64, 255]),
                (16, [128, 0, 128, 255]),
                (20, [64, 0, 191, 255]),
            ],
        ),
        (
            gradient(vec![stop(0.0, red, None), stop(1.0, blue, None)]),
            vec![
                (12, [191, 0, 64, 255]),
                (16, [128, 0, 128, 255]),
                (20, [64, 0, 191, 255]),
            ],
        ),
    ];
    let (mut renderer, mut surface) = renderer_and_surface();
    for graph in [false, true] {
        for (field, samples) in &cases {
            let image = read(&mut renderer, &mut surface, field.clone().into(), graph);
            for &(x, expected) in samples {
                assert_pixel(&image, x, 4, expected, graph);
            }
        }
    }
}

#[test]
fn admitted_hint_witnesses_and_negative_radial_colors_follow_the_power_reference() {
    let (mut renderer, mut surface) = renderer_and_surface();
    for graph in [false, true] {
        for hint in [0.001_f64, 0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99] {
            let field = hinted_line(hint);
            let image = read(&mut renderer, &mut surface, field.into(), graph);
            // The analytic contract uses accepted f32 controls viewed as real
            // numbers. Independent f64 libm evaluates their power reference.
            let accepted_hint = f64::from(hint as f32);
            let k = 0.5_f64.ln() / accepted_hint.ln();
            for (x, p) in [(4, 0.25_f64), (9, 0.5625), (12, 0.75)] {
                assert_pixel(&image, x, 4, opaque_weight(p.powf(k)), graph);
            }
        }
        let field = Gradient::try_radial(
            point(8.0, 8.0),
            16.0,
            8.0,
            vec![
                stop(-0.5, color(1.0, 0.0, 0.0, 1.0), Some(0.0)),
                stop(1.0, color(1.0, 1.0, 0.0, 1.0), None),
            ],
        )
        .unwrap();
        let image = read(&mut renderer, &mut surface, field.into(), graph);
        // The center is the interior hint: P=(0+.5)/1.5=1/3 => w=.5.
        // Negative red remains part of the interval rather than being dropped.
        assert_pixel(&image, 8, 8, [255, 128, 0, 255], graph);
        let k = 0.5_f64.ln() / (1.0_f64 / 3.0).ln();
        let green = ((2.0_f64 / 3.0).powf(k) * 255.0).round() as u8;
        assert_pixel(&image, 16, 8, [255, green, 0, 255], graph);
    }
}

#[test]
fn overflowing_source_spans_keep_finite_geometry_and_hint_ratios() {
    use surgeist_render::Transform;

    let field = Gradient::try_linear(
        point(0.0, 0.0),
        point(f64::MIN_POSITIVE, 0.0),
        vec![
            stop(-f64::MAX, color(1.0, 0.0, 0.0, 1.0), Some(-f64::MAX / 2.0)),
            stop(f64::MAX, color(0.0, 0.0, 1.0, 1.0), None),
        ],
    )
    .unwrap();
    let (mut renderer, mut surface) = renderer_and_surface();
    for graph in [false, true] {
        let scene = draw_scene(graph, |scene| {
            scene.transform(
                Transform::try_new([2.0, 0.0, 0.0, 1.0, 16.0, 0.0]).unwrap(),
                |scene| {
                    scene.fill(Rect::try_new(-8.0, 0.0, 16.0, 16.0).unwrap(), field.clone());
                },
            );
        });
        let image = read_scene(&mut renderer, &mut surface, &scene, graph);
        // MIN_POSITIVE*MAX is just below4. The final transform maps the
        // whole color interval near [8,24] with hint12. Thus x12 is P=.25
        // (weight.5), x17 is P=9/16 (weight.75), despite b-a overflowing.
        assert_pixel(&image, 12, 4, [128, 0, 128, 255], graph);
        assert_pixel(&image, 17, 4, [64, 0, 191, 255], graph);
    }
}

#[test]
fn lost_hint_controls_and_uncertified_arithmetic_refuse_atomically_then_retry() {
    use surgeist_render::{
        ErrorCode, GradientRealizationError, RenderAccountingOptions, RenderAttemptStatus,
        RenderProfile, RenderWork,
    };

    let before_one = f64::from_bits(1.0_f64.to_bits() - 1);
    assert!(before_one < 1.0);
    assert_eq!(before_one as f32, 1.0_f32);
    let cases = [
        (
            before_one,
            GradientRealizationError::UnrepresentableHint { stop_index: 0 },
        ),
        (
            f64::MIN_POSITIVE,
            GradientRealizationError::UnrepresentableHint { stop_index: 0 },
        ),
        // This H is an exact normal f32 control, but its power has too much
        // weight in the uncertain FTZ tail for the required 1/511 guarantee.
        (
            2.0_f64.powi(-100),
            GradientRealizationError::InsufficientNativeCapacity,
        ),
    ];
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
    for graph in [false, true] {
        let before = read(
            &mut renderer,
            &mut surface,
            color(0.0, 1.0, 0.0, 1.0).into(),
            graph,
        );
        let successful = renderer.stats();
        for (hint, expected) in &cases {
            let scene = draw_scene(graph, |scene| {
                scene.fill(rect(), hinted_line(*hint));
            });
            let error =
                pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                    .unwrap_err();
            assert_eq!(error.code(), ErrorCode::GradientRealization);
            assert_eq!(error.gradient_realization_diagnostic(), Some(expected));
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
        }
        let image = read(&mut renderer, &mut surface, hinted_line(0.25).into(), graph);
        assert_pixel(&image, 4, 4, [128, 0, 128, 255], graph);
        // The same control loss is irrelevant after the one-pixel Repeat
        // fallback has selected the stop-only average. No hint is executed.
        let averaged = RepeatingGradient::try_linear(
            point(0.0, 0.0),
            point(1.0, 0.0),
            vec![
                stop(0.0, color(1.0, 0.0, 0.0, 1.0), Some(before_one)),
                stop(1.0, color(0.0, 0.0, 1.0, 1.0), None),
            ],
        )
        .unwrap();
        let image = read(&mut renderer, &mut surface, averaged.into(), graph);
        assert_pixel(&image, 4, 4, [128, 0, 128, 255], graph);
    }
}

#[test]
fn analytic_hints_follow_strokes_prepared_and_native_glyphs_through_final_transforms() {
    use surgeist_render::{
        FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Shape, Stroke, TextGlyph, TextInstance,
        TextPaint, TextRun, TextRunBounds, Transform,
    };

    let field = |length, repeating| -> Paint {
        let records = vec![
            stop(0.0, color(1.0, 0.0, 0.0, 0.5), Some(0.25)),
            stop(1.0, color(0.0, 0.0, 1.0, 1.0), None),
        ];
        if repeating {
            RepeatingGradient::try_linear(point(0.0, 0.0), point(length, 0.0), records)
                .unwrap()
                .into()
        } else {
            Gradient::try_linear(point(0.0, 0.0), point(length, 0.0), records)
                .unwrap()
                .into()
        }
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
            for repeating in [false, true] {
                for consumer in 0..3 {
                    let scene = draw_scene(graph, |scene| {
                        scene.transform(Transform::translation(2.0, 4.0).unwrap(), |scene| {
                            if consumer == 0 {
                                scene.transform(
                                    Transform::try_new([2.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
                                    |scene| {
                                        scene.stroke(
                                            Shape::rect(
                                                Rect::try_new(0.0, 1.0, 12.0, 2.0).unwrap(),
                                            ),
                                            Stroke::try_new(2.0).unwrap(),
                                            field(8.0, repeating),
                                        );
                                    },
                                );
                            } else {
                                let transform =
                                    Transform::try_new([8.0, 0.0, 4.0, 4.0, 0.0, 0.0]).unwrap();
                                let paint = TextPaint::try_fill(field(2.0, repeating)).unwrap();
                                if consumer == 1 {
                                    scene.prepared_glyph_run(prepared.clone(), transform, paint);
                                } else {
                                    scene.text_run(
                                        TextRun::try_new(
                                            FontRef::new(2097).with_data(font.clone()),
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
                    let image = read_scene(&mut renderer, &mut surface, &scene, graph);
                    // Parent translation, anisotropic scale and shear put
                    // (8,6) at stroke x2/L8 or glyph x.5/L2: P=.25. H=.25
                    // gives weight.5, premul(.25,0,.5,.75), straight1:2.
                    assert_pixel(&image, 8 * scale, 6 * scale, [85, 0, 170, 191], graph);
                    // (24,6) is q1.25 in the same run-local field. Ordinary
                    // Pads blue; Repeat reaches the same halfblend in cycle2.
                    let expected = if repeating {
                        [85, 0, 170, 191]
                    } else {
                        [0, 0, 255, 255]
                    };
                    assert_pixel(&image, 24 * scale, 6 * scale, expected, graph);
                    if consumer != 0 {
                        assert_pixel(&image, 16 * scale, 6 * scale, [0, 0, 0, 0], graph);
                    }
                }
            }
        }
    }
}

thread_local! { static HINT_CLOCK: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }

fn hint_clock() -> std::time::Duration {
    HINT_CLOCK.with(|clock| {
        let now = clock.get();
        clock.set(now + 1);
        std::time::Duration::from_nanos(now)
    })
}

fn forbidden_hint_clock() -> std::time::Duration {
    panic!("untimed hint preparation must not read a clock")
}

#[test]
fn actual_hinted_paint_preparation_preserves_caps_cleanup_retry_and_profile_invariance() {
    use surgeist_render::{
        RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderClock,
        RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity,
        RenderResource, RenderWork,
    };

    let geometry = line(16.0);
    let source = source(16.0, 0.5);
    let used = used(16.0, &source);
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: false })
            .with_clock(RenderClock::Host(forbidden_hint_clock)),
    );
    let expected = prepare_linear_gradient(&geometry, &used, &mut context).unwrap();
    let baseline_record = context.take_last_attempt().unwrap();
    let baseline = baseline_record.attempt().unwrap();
    let coordinates = baseline.usage().work(RenderWork::CoordinateVisits);
    let retained = baseline.usage().peak(RenderResource::RetainedCpuBytes);
    assert!(coordinates > 0);
    assert!(retained > 0);
    let failures = [
        (
            RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, coordinates - 1)
                .unwrap(),
            RenderQuantity::Work(RenderWork::CoordinateVisits),
        ),
        (
            RenderLimits::new().with_retained_bytes(retained - 1),
            RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
        ),
    ];
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 2,
            filter: RenderPhaseFilter::Phase(RenderPhase::Coordinates),
        },
    ] {
        let timing = matches!(profile, RenderProfile::Summary { timing: true });
        let clock = if timing {
            hint_clock
        } else {
            forbidden_hint_clock
        };
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(clock));
        context.set_options(options);
        assert_eq!(
            prepare_linear_gradient(&geometry, &used, &mut context).unwrap(),
            expected
        );
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.usage(), baseline.usage());
        assert_eq!(report.elapsed().is_some(), timing);
        for phase in [
            RenderPhase::Preparation,
            RenderPhase::Coordinates,
            RenderPhase::Output,
            RenderPhase::Cleanup,
        ] {
            assert_eq!(report.phase_usage(phase), baseline.phase_usage(phase));
        }
        for (limits, quantity) in failures {
            context.set_options(options.with_limits(limits));
            let error = prepare_linear_gradient(&geometry, &used, &mut context).unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected the actual hinted consumer's Ledger cap")
            };
            assert_eq!(refusal.quantity(), quantity);
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                assert_eq!(report.usage().live(resource), 0);
            }
            context.set_options(options);
            assert_eq!(
                prepare_linear_gradient(&geometry, &used, &mut context).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn hinted_frames_preserve_publication_under_program_caps_and_profiles() {
    use surgeist_render::{
        RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderClock,
        RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderWork,
    };

    let make = |accounting| {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_accounting(accounting),
        ))
        .unwrap();
        let surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        (renderer, surface)
    };
    let solid = draw_scene(false, |scene| {
        scene.fill(rect(), color(0.0, 1.0, 0.0, 1.0));
    });
    let hinted = draw_scene(false, |scene| {
        scene.fill(rect(), hinted_line(0.25));
    });
    let unhinted = draw_scene(false, |scene| {
        scene.fill(
            rect(),
            Gradient::try_linear(
                point(0.0, 0.0),
                point(16.0, 0.0),
                vec![
                    stop(0.0, color(1.0, 0.0, 0.0, 1.0), None),
                    stop(1.0, color(0.0, 0.0, 1.0, 1.0), None),
                ],
            )
            .unwrap(),
        );
    });
    let untimed = RenderAccountingOptions::new()
        .with_profile(RenderProfile::Summary { timing: false })
        .with_clock(RenderClock::Host(forbidden_hint_clock));
    let (mut renderer, mut surface) = make(untimed);
    read_scene(&mut renderer, &mut surface, &solid, false);
    read_scene(&mut renderer, &mut surface, &unhinted, false);
    let unhinted_report = renderer.take_last_attempt().unwrap();
    let baseline_pixels = read_scene(&mut renderer, &mut surface, &hinted, false);
    let baseline = renderer.take_last_attempt().unwrap();
    assert_eq!(baseline.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(baseline.cleanup_fault(), None);
    assert_pixel(&baseline_pixels, 4, 4, [128, 0, 128, 255], false);
    let attempt = baseline.attempt().unwrap();
    let all_coordinates = attempt.usage().work(RenderWork::CoordinateVisits);
    // The complete same-geometry unhinted frame upper-bounds its shared
    // geometry/native-stop prefix. Allow all of that measured work plus one
    // scalar, so refusal inside GradientPrepare targets added hint work.
    let coordinate_cap = unhinted_report
        .attempt()
        .unwrap()
        .usage()
        .work(RenderWork::CoordinateVisits)
        + 1;
    assert!(all_coordinates > coordinate_cap);
    // Program capture owns one independently admitted copy per actual color
    // record before the common endpoint push. Pad has only that common push.
    // Unlike padded texture bytes, these real copies cannot replace padding.
    let unhinted_usage = unhinted_report.attempt().unwrap().usage();
    let copy_cap = unhinted_usage.work(RenderWork::RasterResourceCopies);
    assert!(attempt.usage().work(RenderWork::RasterResourceCopies) > copy_cap);
    println!(
        "hint frame calibration: unhinted coordinates={}, hinted coordinates={}, unhinted serialized={}, hinted serialized={}, unhinted copies={}, hinted copies={}",
        unhinted_usage.work(RenderWork::CoordinateVisits),
        all_coordinates,
        unhinted_usage.work(RenderWork::SerializedBytes),
        attempt.usage().work(RenderWork::SerializedBytes),
        copy_cap,
        attempt.usage().work(RenderWork::RasterResourceCopies),
    );
    let expected_usage = attempt.usage();
    drop(surface);
    drop(renderer);
    let caps = [
        (RenderWork::CoordinateVisits, coordinate_cap),
        (RenderWork::RasterResourceCopies, copy_cap),
    ];
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 4,
            filter: RenderPhaseFilter::Phase(RenderPhase::GradientPrepare),
        },
    ] {
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        let clock = if timed {
            hint_clock
        } else {
            forbidden_hint_clock
        };
        let accounting = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(clock));
        let (mut renderer, mut surface) = make(accounting);
        read_scene(&mut renderer, &mut surface, &solid, false);
        read_scene(&mut renderer, &mut surface, &unhinted, false);
        let pixels = read_scene(&mut renderer, &mut surface, &hinted, false);
        assert_eq!(pixels, baseline_pixels);
        let report = renderer.take_last_attempt().unwrap();
        let attempt = report.attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.cleanup_fault(), None);
        assert_eq!(attempt.usage(), expected_usage);
        assert_eq!(attempt.elapsed().is_some(), timed);
        if profile != RenderProfile::Off {
            assert!(attempt.phase_count(RenderPhase::GradientPrepare) > 0);
            assert_eq!(
                attempt
                    .phase_elapsed(RenderPhase::GradientPrepare)
                    .is_some(),
                timed
            );
        }
        drop(surface);
        drop(renderer);
        for (work, cap) in caps {
            let limits = RenderLimits::new().with_work_limit(work, cap).unwrap();
            let (mut renderer, mut surface) = make(accounting.with_limits(limits));
            read_scene(&mut renderer, &mut surface, &solid, false);
            let before = read_scene(&mut renderer, &mut surface, &unhinted, false);
            assert_pixel(&before, 4, 4, [191, 0, 64, 255], false);
            let successful = renderer.stats();
            let error =
                pollster::block_on(renderer.render(&mut surface, &hinted, Parameters::default()))
                    .unwrap_err();
            let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
            else {
                panic!("expected actual program work cap")
            };
            assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
            // Public frame limits belong to the Attempt ancestor; this
            // diagnostic names that refusing owner, not the action scope.
            assert_eq!(refusal.phase(), RenderPhase::Attempt);
            assert_eq!(renderer.stats(), successful);
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                before
            );
            let report = renderer.take_last_attempt().unwrap();
            let attempt = report.attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            assert_eq!(report.cleanup_fault(), None);
            assert!(attempt.usage().work(work) <= cap);
            if work == RenderWork::CoordinateVisits {
                assert!(
                    attempt.phase_usage(RenderPhase::GradientPrepare).work(work)
                        > unhinted_report
                            .attempt()
                            .unwrap()
                            .phase_usage(RenderPhase::GradientPrepare)
                            .work(work)
                );
            } else {
                assert_eq!(
                    attempt.phase_usage(RenderPhase::Encode).work(work),
                    copy_cap
                );
                assert!(copy_cap > 0);
            }
            println!(
                "hint frame refusal: profile={profile:?}, work={work:?}, cap={cap}, owner={:?}, admitted={}, preparation coordinates={}, encode copies={}",
                refusal.phase(),
                attempt.usage().work(work),
                attempt
                    .phase_usage(RenderPhase::GradientPrepare)
                    .work(RenderWork::CoordinateVisits),
                attempt
                    .phase_usage(RenderPhase::Encode)
                    .work(RenderWork::RasterResourceCopies),
            );
            assert_eq!(attempt.usage().work(RenderWork::Submissions), 0);
            assert_eq!(renderer.cpu_accounting_snapshot().unwrap().fault(), None);
            // A fresh real submission under the same cap succeeds after all
            // partial program work was refused and its custody was cleaned.
            assert_eq!(
                read_scene(&mut renderer, &mut surface, &unhinted, false),
                before
            );
            let retry = renderer.take_last_attempt().unwrap();
            assert_eq!(retry.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(retry.cleanup_fault(), None);
            assert!(
                retry
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions)
                    > 0
            );
        }
    }
}
