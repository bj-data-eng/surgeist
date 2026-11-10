#![cfg(feature = "render-paint")]
#![forbid(unsafe_code)]

use std::{cell::Cell, mem::size_of, time::Duration};
use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAttemptKind, RenderAttemptStatus, RenderClock, RenderLimits, RenderPhase,
    RenderPhaseFilter, RenderProfile, RenderResource, RenderWork,
    prepare_zero_height_gradient_color,
};

fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}
fn position(length: f64, fraction: f64) -> paint::GradientPosition {
    paint::GradientPosition { length, fraction }
}
fn stop(payload: Color, length: f64, fraction: f64) -> paint::GradientStopItem<Color> {
    paint::GradientStopItem::Stop {
        payload,
        positions: paint::GradientStopPositions::One(position(length, fraction)),
    }
}
fn source(colors: [Color; 3]) -> [paint::GradientStopItem<Color>; 3] {
    [
        stop(colors[0], 0.0, 0.0),
        stop(colors[1], 10.0, 0.0),
        stop(colors[2], 0.0, 1.0),
    ]
}
fn paint_options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}
fn geometry(radius: f64) -> paint::RadialGradientGeometry {
    paint::resolve_radial_gradient(
        layout::Size::new(32.0, 16.0),
        Some(layout::Point::new(8.0, 8.0)),
        paint::RadialGradientSize::EllipseRadii(layout::Size::new(
            position(radius, 0.0),
            position(0.0, 0.0),
        )),
        paint_options(),
    )
    .result
    .unwrap()
}
fn measure<'a>(
    geometry: &paint::RadialGradientGeometry,
    source: &'a [paint::GradientStopItem<Color>],
    repeat: paint::GradientRepeat,
) -> paint::ZeroHeightGradientColors<'a, Color> {
    paint::resolve_zero_height_gradient_colors(geometry, source, repeat, paint_options())
        .result
        .unwrap()
}
fn owned(radius: f64, colors: [Color; 3], repeat: paint::GradientRepeat) -> Color {
    let geometry = geometry(radius);
    let source = source(colors);
    let measure = measure(&geometry, &source, repeat);
    let mut context = RenderAccountingContext::default();
    prepare_zero_height_gradient_color(&measure, &mut context).unwrap()
}
fn assert_color(actual: Color, expected: [f32; 4]) {
    for (actual, expected) in [actual.r(), actual.g(), actual.b(), actual.a()]
        .into_iter()
        .zip(expected)
    {
        assert!(
            (actual - expected).abs() <= 2.0 * f32::EPSILON,
            "channel {actual:?}, expected {expected:?}"
        );
    }
}

#[test]
fn real_zero_height_measure_becomes_an_owned_premultiplied_color_before_quantization() {
    let rgb = [
        color(1.0, 0.0, 0.0, 1.0),
        color(0.0, 1.0, 0.0, 1.0),
        color(0.0, 0.0, 1.0, 1.0),
    ];
    for radius in [1.0, 20.0, 100.0] {
        // All producers, borrowed measures and preparation contexts are dropped
        // on return. The selected weights are (0,.5,.5), not (.25,.5,.25).
        assert_color(
            owned(radius, rgb, paint::GradientRepeat::Repeat),
            [0.0, 0.5, 0.5, 1.0],
        );
        assert_eq!(owned(radius, rgb, paint::GradientRepeat::None), rgb[2]);
    }
    let partial = [
        color(1.0, 0.0, 0.0, 0.0),
        color(0.0, 1.0, 0.0, 0.5),
        color(0.0, 0.0, 1.0, 1.0),
    ];
    // Weighted premultiplied (0,.25,.5,.75) unpremultiplies to (0,1/3,2/3,.75).
    assert_color(
        owned(20.0, partial, paint::GradientRepeat::Repeat),
        [0.0, 1.0 / 3.0, 2.0 / 3.0, 0.75],
    );
    let transparent = [
        color(1.0, 0.0, 0.0, 0.0),
        color(0.0, 1.0, 0.0, 0.0),
        color(0.0, 0.0, 1.0, 0.0),
    ];
    assert_eq!(
        owned(20.0, transparent, paint::GradientRepeat::Repeat),
        Color::TRANSPARENT
    );
    // Last is a direct copy, preserving even unused straight RGB at alpha0;
    // only an averaged zero-alpha result selects canonical transparent.
    assert_eq!(
        owned(20.0, transparent, paint::GradientRepeat::None),
        transparent[2]
    );
}

#[test]
fn actual_two_occurrences_and_hint_bounded_omissions_reduce_without_hint_colors() {
    let red = color(1.0, 0.0, 0.0, 1.0);
    let green = color(0.0, 1.0, 0.0, 1.0);
    let blue = color(0.0, 0.0, 1.0, 1.0);
    let geometry = geometry(20.0);
    for (positions, expected) in [
        (
            (position(0.0, 0.0), position(0.0, 0.1)),
            [0.55, 0.0, 0.45, 1.0],
        ),
        (
            (position(0.0, 1.0), position(0.0, 1.0)),
            [0.75, 0.0, 0.25, 1.0],
        ),
    ] {
        let source = [
            paint::GradientStopItem::Stop {
                payload: red,
                positions: paint::GradientStopPositions::Two(positions.0, positions.1),
            },
            stop(blue, 0.0, 1.0),
        ];
        let measure = measure(&geometry, &source, paint::GradientRepeat::Repeat);
        let result =
            prepare_zero_height_gradient_color(&measure, &mut RenderAccountingContext::default())
                .unwrap();
        assert_color(result, expected);
    }
    let source = [
        stop(red, 0.0, 0.0),
        paint::GradientStopItem::Stop {
            payload: green,
            positions: paint::GradientStopPositions::Omitted,
        },
        paint::GradientStopItem::Hint {
            position: position(0.0, 0.75),
        },
        stop(blue, 0.0, 1.0),
    ];
    let measure = measure(&geometry, &source, paint::GradientRepeat::Repeat);
    let result =
        prepare_zero_height_gradient_color(&measure, &mut RenderAccountingContext::default())
            .unwrap();
    assert_color(result, [3.0 / 16.0, 0.5, 5.0 / 16.0, 1.0]);
}

thread_local! { static CLOCK: Cell<u64> = const { Cell::new(0) }; }
fn clock() -> Duration {
    CLOCK.with(|value| {
        let now = value.get();
        value.set(now + 1);
        Duration::from_nanos(now)
    })
}
fn forbidden_clock() -> Duration {
    panic!("untimed color reduction must not read a clock")
}

#[test]
fn fixed_color_custody_actual_reduction_caps_and_profiles_preserve_values_and_retry() {
    let geometry = geometry(20.0);
    let source = source([
        color(1.0, 0.0, 0.0, 0.0),
        color(0.0, 1.0, 0.0, 0.5),
        color(0.0, 0.0, 1.0, 1.0),
    ]);
    for repeat in [paint::GradientRepeat::None, paint::GradientRepeat::Repeat] {
        let measure = measure(&geometry, &source, repeat);
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false })
                .with_clock(RenderClock::Host(forbidden_clock)),
        );
        let expected = prepare_zero_height_gradient_color(&measure, &mut context).unwrap();
        let baseline_record = context.take_last_attempt().unwrap();
        let baseline = baseline_record.attempt().unwrap();
        assert_eq!(baseline.kind(), RenderAttemptKind::PaintGradient);
        assert_eq!(baseline.status(), RenderAttemptStatus::Succeeded);
        let bytes = u64::try_from(size_of::<Color>()).unwrap();
        assert_eq!(
            baseline.usage().peak(RenderResource::RetainedCpuBytes),
            bytes
        );
        assert_eq!(baseline.usage().peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(
            baseline.usage().work(RenderWork::PreparedTransferredBytes),
            bytes
        );
        assert_eq!(
            baseline.usage().work(RenderWork::GradientStopVisits),
            if repeat == paint::GradientRepeat::None {
                1
            } else {
                3
            }
        );
        // Last copies directly. Average performs weight capture1,
        // premultiplication11 and weighted product/sum8 per occurrence, then
        // nonzero-alpha final conversion12, including the zero-weight record.
        let coordinates = if repeat == paint::GradientRepeat::None {
            0
        } else {
            3 * 20 + 12
        };
        assert_eq!(
            baseline.usage().work(RenderWork::CoordinateVisits),
            coordinates
        );
        assert_eq!(baseline.usage().work(RenderWork::PaintVisits), 2);
        let visits = if repeat == paint::GradientRepeat::None {
            1
        } else {
            3
        };
        let mut failure_oracles = Vec::new();
        for (profile_index, profile) in [
            RenderProfile::Off,
            RenderProfile::Summary { timing: false },
            RenderProfile::Summary { timing: true },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 2,
                filter: RenderPhaseFilter::Phase(RenderPhase::Coordinates),
            },
            RenderProfile::Detailed {
                timing: false,
                event_limit: 0,
                filter: RenderPhaseFilter::All,
            },
        ]
        .into_iter()
        .enumerate()
        {
            let timed = matches!(profile, RenderProfile::Summary { timing: true });
            let options = RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(if timed {
                    clock
                } else {
                    forbidden_clock
                }));
            context.set_options(options);
            assert_eq!(
                prepare_zero_height_gradient_color(&measure, &mut context).unwrap(),
                expected
            );
            let report = context.last_attempt().unwrap().attempt().unwrap();
            assert_eq!(report.usage(), baseline.usage());
            assert_eq!(report.elapsed().is_some(), timed);
            let mut limits = vec![RenderLimits::new().with_retained_bytes(bytes - 1)];
            for (work, count) in [
                (RenderWork::GradientStopVisits, visits),
                (RenderWork::PaintVisits, 2),
                (RenderWork::CoordinateVisits, coordinates),
            ] {
                if count > 0 {
                    limits.push(
                        RenderLimits::new()
                            .with_work_limit(work, count - 1)
                            .unwrap(),
                    );
                }
            }
            for (index, limits) in limits.into_iter().enumerate() {
                context.set_options(options.with_limits(limits));
                let error = prepare_zero_height_gradient_color(&measure, &mut context).unwrap_err();
                assert!(matches!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(_))
                ));
                let report = context.last_attempt().unwrap().attempt().unwrap();
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
                assert_eq!(report.usage().work(RenderWork::Submissions), 0);
                for resource in [
                    RenderResource::RetainedCpuBytes,
                    RenderResource::ScratchCpuBytes,
                ] {
                    assert_eq!(report.usage().live(resource), 0);
                }
                let phases = [
                    RenderPhase::Preparation,
                    RenderPhase::Coordinates,
                    RenderPhase::Output,
                    RenderPhase::Cleanup,
                ]
                .map(|phase| report.phase_usage(phase));
                if profile_index == 0 {
                    failure_oracles.push((
                        error.accounting_diagnostic().unwrap(),
                        report.usage(),
                        phases,
                    ));
                } else {
                    let (expected_error, expected_usage, expected_phases) = &failure_oracles[index];
                    assert_eq!(error.accounting_diagnostic(), Some(*expected_error));
                    assert_eq!(&report.usage(), expected_usage);
                    assert_eq!(&phases, expected_phases);
                }
                context.set_options(options);
                assert_eq!(
                    prepare_zero_height_gradient_color(&measure, &mut context).unwrap(),
                    expected
                );
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::*;
    use surgeist_render::{
        EffectQualityPolicy, FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Image, ImageBuffer,
        Layer, Options, Parameters, Rect, RenderQuantity, RenderRoute, Renderer,
        ResolvedLayerAlphaMask, Scene, Shape, Size, Stroke, Surface, TextGlyph, TextInstance,
        TextPaint, TextRun, TextRunBounds, Transform,
    };

    fn bounds() -> Rect {
        Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
    }
    fn scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
        let mut scene = Scene::new();
        if graph {
            let mask = Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128])
                .unwrap();
            scene.layer(
                Layer::new().with_resolved_alpha_mask(
                    ResolvedLayerAlphaMask::try_new(mask, bounds()).unwrap(),
                ),
                draw,
            );
        } else {
            draw(&mut scene);
        }
        scene
    }
    fn read(
        renderer: &mut Renderer,
        surface: &mut Surface,
        scene: &Scene,
        graph: bool,
    ) -> ImageBuffer {
        let stats =
            pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
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
    fn pixel(image: &ImageBuffer, x: u32, y: u32, mut expected: [u8; 4], graph: bool) {
        if graph {
            expected[3] = u8::try_from((u16::from(expected[3]) * 128 + 127) / 255).unwrap();
        }
        let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
        let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(a, b)| a.abs_diff(b) <= 2),
            "pixel({x},{y}) actual{actual:?}, expected{expected:?}, graph={graph}"
        );
    }

    #[test]
    fn owned_zero_height_colors_fill_center_lines_strokes_and_both_glyph_routes_at_final_scale() {
        let opaque = [
            color(1.0, 0.0, 0.0, 1.0),
            color(0.0, 1.0, 0.0, 1.0),
            color(0.0, 0.0, 1.0, 1.0),
        ];
        let partial = [
            color(1.0, 0.0, 0.0, 0.0),
            color(0.0, 1.0, 0.0, 0.5),
            color(0.0, 0.0, 1.0, 1.0),
        ];
        // These values are all that survive; no source/context/measure remains.
        let fields = [
            (
                owned(20.0, opaque, paint::GradientRepeat::None),
                [0, 0, 255, 255],
            ),
            (
                owned(20.0, opaque, paint::GradientRepeat::Repeat),
                [0, 128, 128, 255],
            ),
            (
                owned(20.0, partial, paint::GradientRepeat::Repeat),
                [0, 85, 170, 191],
            ),
        ];
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
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        for scale in [1_u32, 2] {
            let mut surface = pollster::block_on(
                renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), f64::from(scale)),
            )
            .unwrap();
            for graph in [false, true] {
                for (color, expected) in fields {
                    for consumer in 0..4 {
                        let scene = scene(graph, |scene| {
                            if consumer == 0 {
                                scene.fill(bounds(), color);
                            } else {
                                scene.transform(
                                    Transform::translation(2.0, 4.0).unwrap(),
                                    |scene| {
                                        if consumer == 1 {
                                            scene.transform(
                                                Transform::try_new([2.0, 0.0, 1.0, 1.0, 0.0, 0.0])
                                                    .unwrap(),
                                                |scene| {
                                                    scene.stroke(
                                                        Shape::rect(
                                                            Rect::try_new(0.0, 1.0, 12.0, 2.0)
                                                                .unwrap(),
                                                        ),
                                                        Stroke::try_new(2.0).unwrap(),
                                                        color,
                                                    );
                                                },
                                            );
                                        } else {
                                            let transform =
                                                Transform::try_new([8.0, 0.0, 4.0, 4.0, 0.0, 0.0])
                                                    .unwrap();
                                            let paint = TextPaint::try_fill(color.into()).unwrap();
                                            if consumer == 2 {
                                                scene.prepared_glyph_run(
                                                    prepared.clone(),
                                                    transform,
                                                    paint,
                                                );
                                            } else {
                                                scene.text_run(
                                                    TextRun::try_new(
                                                        FontRef::new(2100).with_data(font.clone()),
                                                        1.0,
                                                        transform,
                                                        paint,
                                                        &glyphs,
                                                        TextRunBounds::try_ink(
                                                            Rect::try_new(0.0, 0.0, 3.0, 1.0)
                                                                .unwrap(),
                                                        )
                                                        .unwrap(),
                                                    )
                                                    .unwrap(),
                                                );
                                            }
                                        }
                                    },
                                );
                            }
                        });
                        let image = read(&mut renderer, &mut surface, &scene, graph);
                        if consumer == 0 {
                            // The actual producer's center and center line are
                            // solid too; no vanishing-ellipse field is sampled.
                            for (x, y) in [(8, 8), (4, 8), (24, 12)] {
                                pixel(&image, x * scale, y * scale, expected, graph);
                            }
                        } else {
                            for x in [8, 24] {
                                pixel(&image, x * scale, 6 * scale, expected, graph);
                            }
                            if consumer >= 2 {
                                pixel(&image, 16 * scale, 6 * scale, [0, 0, 0, 0], graph);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn refused_solid_frame_preserves_pixels_and_stats_then_retries_the_owned_color() {
        let colors = [
            color(1.0, 0.0, 0.0, 1.0),
            color(0.0, 1.0, 0.0, 1.0),
            color(0.0, 0.0, 1.0, 1.0),
        ];
        let owned = owned(20.0, colors, paint::GradientRepeat::Repeat);
        // One actual normalized fill fits. The next forward emission refuses
        // before publication; authored lifecycle totals are not admission caps.
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::FillsEmitted, 1)
            .unwrap();
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        let baseline = scene(false, |scene| {
            scene.fill(bounds(), owned);
        });
        let before = read(&mut renderer, &mut surface, &baseline, false);
        let successful = renderer.stats();
        let refused = scene(false, |scene| {
            scene.fill(bounds(), owned);
            scene.fill(bounds(), colors[0]);
        });
        let error =
            pollster::block_on(renderer.render(&mut surface, &refused, Parameters::default()))
                .unwrap_err();
        assert!(
            matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(refusal)) if refusal.quantity() == RenderQuantity::Work(RenderWork::FillsEmitted))
        );
        assert_eq!(renderer.stats(), successful);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.cleanup_fault(), None);
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::FillsEmitted),
            1
        );
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::Submissions),
            0
        );
        assert_eq!(read(&mut renderer, &mut surface, &baseline, false), before);
        pixel(&before, 8, 8, [0, 128, 128, 255], false);
    }
}
