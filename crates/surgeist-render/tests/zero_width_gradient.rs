#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

use surgeist_layout as layout;
use surgeist_paint as paint;
use surgeist_render::{
    Color, EffectQualityPolicy, Image, ImageBuffer, Layer, Options, Paint, Parameters, Rect,
    RenderAccountingContext, RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Size, Surface,
    prepare_radial_gradient, prepare_repeating_radial_gradient,
};

fn options() -> paint::PaintOptions {
    paint::PaintOptions::new(paint::PaintLimits::unlimited(), paint::ProfileMode::Off)
}

fn position(length: f64, fraction: f64) -> paint::GradientPosition {
    paint::GradientPosition { length, fraction }
}

fn field(repeating: bool, height: f64, center_y: f64) -> Paint {
    let geometry = paint::resolve_radial_gradient(
        layout::Size::new(16.0, 8.0),
        Some(layout::Point::new(8.0, center_y)),
        paint::RadialGradientSize::EllipseRadii(layout::Size::new(
            position(0.0, 0.0),
            position(height, 0.0),
        )),
        options(),
    )
    .result
    .expect("authentic Paint radial geometry preparation must succeed");
    assert_eq!(geometry.case(), paint::RadialGradientCase::ZeroWidth);
    assert_eq!(geometry.center(), layout::Point::new(8.0, center_y));
    let (first, last) = if repeating { (1.0, 5.0) } else { (2.0, 6.0) };
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let source = [
        paint::GradientStopItem::Stop {
            payload: red,
            positions: paint::GradientStopPositions::One(position(first, -1.0)),
        },
        paint::GradientStopItem::Stop {
            payload: blue,
            positions: paint::GradientStopPositions::One(position(last, 1.0)),
        },
    ];
    let used = paint::resolve_gradient_stops(0.0, &source, options())
        .result
        .expect("authentic Paint basis-zero used-stop preparation must succeed");
    assert_eq!(used.line_length(), 0.0);
    assert!(std::ptr::eq(used.source(), source.as_slice()));
    let records: Vec<_> = used
        .items()
        .map(|item| {
            let paint::UsedGradientItem::Stop {
                source_index,
                position_index,
                position,
                payload,
            } = item
            else {
                panic!("the two supplied colors must remain actual stop occurrences")
            };
            (source_index, position_index, position, *payload)
        })
        .collect();
    // Percentage contributions vanish at the authentic zero basis; finite
    // physical lengths and exact source associations remain unchanged.
    assert_eq!(records, [(0, 0, first, red), (1, 0, last, blue)]);
    let mut context = RenderAccountingContext::default();
    if repeating {
        let field = prepare_repeating_radial_gradient(&geometry, &used, &mut context)
            .expect("repeating radial consumer must accept authentic ZeroWidth with basis zero");
        assert_eq!(
            field
                .stops()
                .iter()
                .map(|stop| (stop.position(), stop.color()))
                .collect::<Vec<_>>(),
            [(first, red), (last, blue)]
        );
        field.into()
    } else {
        let field = prepare_radial_gradient(&geometry, &used, &mut context)
            .expect("ordinary radial consumer must accept authentic ZeroWidth with basis zero");
        assert_eq!(
            field
                .stops()
                .iter()
                .map(|stop| (stop.position(), stop.color()))
                .collect::<Vec<_>>(),
            [(first, red), (last, blue)]
        );
        field.into()
    }
}

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 16.0, 8.0).unwrap()
}

fn renderer_and_surface() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface =
        pollster::block_on(renderer.create_headless(Size::try_new(16.0, 8.0).unwrap(), 1.0))
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
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, bounds()).unwrap()),
            |scene| {
                scene.fill(bounds(), field);
            },
        );
    } else {
        scene.fill(bounds(), field);
    }
    let stats =
        pollster::block_on(renderer.render(surface, &scene, Parameters::default())).unwrap();
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

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, rgb: [u8; 3], graph: bool) {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
    let expected = [rgb[0], rgb[1], rgb[2], if graph { 128 } else { 255 }];
    let tolerance = if graph { 3 } else { 2 };
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "zero-width pixel({x},{y}), graph={graph}: actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[test]
fn ordinary_zero_width_consumer_folds_distance_before_offset_and_pads_both_sides() {
    // These owned fields must survive all geometry, used/source storage and
    // contexts in field(). Neither GPU creation nor readback precedes acceptance.
    let fields = [field(false, 8.0, 4.0), field(false, 0.0, -100.0)];
    let (mut renderer, mut surface) = renderer_and_surface();
    for field in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone(), graph);
            // r=abs(x-8), then t=clamp((r-2)/4,0,1). Opaque red/blue
            // premultiplied interpolation is (1-t,0,t,1). Folding after the
            // offset would incorrectly change both the center and left half.
            for (x, rgb) in [
                (8, [255, 0, 0]),
                (7, [255, 0, 0]),
                (9, [255, 0, 0]),
                (4, [128, 0, 128]),
                (12, [128, 0, 128]),
                (2, [0, 0, 255]),
                (14, [0, 0, 255]),
            ] {
                for y in [2, 6] {
                    assert_pixel(&image, x, y, rgb, graph);
                }
            }
        }
    }
}

#[test]
fn repeating_zero_width_consumer_folds_before_phase_and_preserves_both_seams() {
    let fields = [field(true, 8.0, 4.0), field(true, 0.0, -100.0)];
    let (mut renderer, mut surface) = renderer_and_surface();
    for field in fields {
        for graph in [false, true] {
            let image = read(&mut renderer, &mut surface, field.clone(), graph);
            // Physical first=1, last=5, T=4. Fold r=abs(x-8) before
            // u=fract((r-1)/4). Center u=.75; r1/r5 are exact red seams;
            // r2/r3/r4 have blue weights .25/.5/.75 on either half.
            for (x, rgb) in [
                (8, [64, 0, 191]),
                (7, [255, 0, 0]),
                (9, [255, 0, 0]),
                (6, [191, 0, 64]),
                (10, [191, 0, 64]),
                (5, [128, 0, 128]),
                (11, [128, 0, 128]),
                (4, [64, 0, 191]),
                (12, [64, 0, 191]),
                (3, [255, 0, 0]),
                (13, [255, 0, 0]),
            ] {
                for y in [2, 6] {
                    assert_pixel(&image, x, y, rgb, graph);
                }
            }
        }
    }
}

fn at(color: Color, length: f64, fraction: f64) -> paint::GradientStopItem<Color> {
    paint::GradientStopItem::Stop {
        payload: color,
        positions: paint::GradientStopPositions::One(position(length, fraction)),
    }
}

fn geometry(origin: f64) -> paint::RadialGradientGeometry {
    paint::resolve_radial_gradient(
        layout::Size::new(32.0, 16.0),
        Some(layout::Point::new(origin, -100.0)),
        paint::RadialGradientSize::EllipseRadii(layout::Size::new(
            position(0.0, 0.0),
            position(8.0, 0.0),
        )),
        options(),
    )
    .result
    .unwrap()
}

fn composed(origin: f64, source: Vec<paint::GradientStopItem<Color>>, repeating: bool) -> Paint {
    let geometry = geometry(origin);
    let used = paint::resolve_gradient_stops(0.0, &source, options())
        .result
        .unwrap();
    assert_eq!(geometry.case(), paint::RadialGradientCase::ZeroWidth);
    assert_eq!(used.line_length(), 0.0);
    let mut context = RenderAccountingContext::default();
    if repeating {
        let output = prepare_repeating_radial_gradient(&geometry, &used, &mut context).unwrap();
        assert_eq!(output.horizontal_distance_origin(), Some(origin));
        output.into()
    } else {
        let output = prepare_radial_gradient(&geometry, &used, &mut context).unwrap();
        assert_eq!(output.horizontal_distance_origin(), Some(origin));
        output.into()
    }
}

#[test]
fn authentic_basis_zero_omissions_percentages_two_positions_and_hints_keep_their_semantics() {
    let r = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let g = Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap();
    let b = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let (mut renderer, mut surface) = renderer_and_surface();
    for repeating in [false, true] {
        let omitted = vec![
            paint::GradientStopItem::Stop {
                payload: r,
                positions: paint::GradientStopPositions::Omitted,
            },
            paint::GradientStopItem::Stop {
                payload: b,
                positions: paint::GradientStopPositions::Omitted,
            },
        ];
        let fields = [
            composed(8.0, omitted, repeating),
            composed(8.0, vec![at(r, 0.0, -1.0), at(b, 0.0, 1.0)], repeating),
        ];
        for graph in [false, true] {
            for field in &fields {
                let image = read(&mut renderer, &mut surface, field.clone(), graph);
                for x in [2, 8, 14] {
                    assert_pixel(
                        &image,
                        x,
                        4,
                        if repeating {
                            [128, 0, 128]
                        } else {
                            [0, 0, 255]
                        },
                        graph,
                    );
                }
            }
        }
    }
    let two = composed(
        8.0,
        vec![
            paint::GradientStopItem::Stop {
                payload: r,
                positions: paint::GradientStopPositions::Two(
                    position(0.0, 0.0),
                    position(2.0, 0.0),
                ),
            },
            at(b, 6.0, 0.0),
        ],
        false,
    );
    let hinted = composed(
        8.0,
        vec![
            at(r, 0.0, 0.0),
            paint::GradientStopItem::Stop {
                payload: g,
                positions: paint::GradientStopPositions::Omitted,
            },
            paint::GradientStopItem::Hint {
                position: position(6.0, 0.0),
            },
            at(b, 8.0, 0.0),
        ],
        false,
    );
    for graph in [false, true] {
        let image = read(&mut renderer, &mut surface, two.clone(), graph);
        for x in [6, 8, 10] {
            assert_pixel(&image, x, 4, [255, 0, 0], graph);
        }
        for x in [4, 12] {
            assert_pixel(&image, x, 4, [128, 0, 128], graph);
        }
        let image = read(&mut renderer, &mut surface, hinted.clone(), graph);
        // Adopted hint6 bounds the omitted G at3; the outgoing G3/B8
        // interval reaches half-color at its physical hint6 on either half.
        for x in [5, 11] {
            assert_pixel(&image, x, 4, [0, 255, 0], graph);
        }
        for x in [2, 14] {
            assert_pixel(&image, x, 4, [0, 128, 128], graph);
        }
    }
}

#[test]
fn identity_conversion_preserves_exact_basis_and_existing_refusal_domains_with_real_caps() {
    use surgeist_render::{
        PaintGradientError, RenderAccountingOptions, RenderAttemptStatus, RenderClock,
        RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderResource, RenderWork,
    };
    let r = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let b = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let geometry = geometry(8.0);
    let source = [
        at(r, 2.0, 0.0),
        paint::GradientStopItem::Hint {
            position: position(3.0, 0.0),
        },
        at(b, 6.0, 0.0),
    ];
    let used = paint::resolve_gradient_stops(0.0, &source, options())
        .result
        .unwrap();
    let mut context = RenderAccountingContext::new(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    );
    let ordinary = prepare_radial_gradient(&geometry, &used, &mut context).unwrap();
    let pad_record = context.take_last_attempt().unwrap();
    let pad = pad_record.attempt().unwrap();
    assert_eq!(
        ordinary
            .stops()
            .iter()
            .map(|s| (s.position(), s.hint_after().map(|h| h.position())))
            .collect::<Vec<_>>(),
        [(2.0, Some(3.0)), (6.0, None)]
    );
    // Case inspection is PaintVisits. Coordinate actions are origin capture,
    // used-basis capture and equality (3), then one position capture for each
    // of three actual source items. Identity performs no division or y capture.
    assert_eq!(pad.usage().work(RenderWork::CoordinateVisits), 3 + 3);
    assert_eq!(pad.usage().work(RenderWork::GradientStopVisits), 2 * 3);
    assert_eq!(pad.usage().work(RenderWork::PaintVisits), 3);
    let repeated = prepare_repeating_radial_gradient(&geometry, &used, &mut context).unwrap();
    let repeat_record = context.take_last_attempt().unwrap();
    let repeat = repeat_record.attempt().unwrap();
    assert_eq!(repeated.stops(), ordinary.stops());
    // Repeat separately performs its required span subtraction and domain check.
    assert_eq!(repeat.usage().work(RenderWork::CoordinateVisits), 3 + 3 + 2);
    let mismatch = paint::resolve_gradient_stops(1.0, &source, options())
        .result
        .unwrap();
    for error in [
        prepare_radial_gradient(&geometry, &mismatch, &mut context).unwrap_err(),
        prepare_repeating_radial_gradient(&geometry, &mismatch, &mut context).unwrap_err(),
    ] {
        assert_eq!(
            error.paint_gradient_diagnostic(),
            Some(&PaintGradientError::BasisMismatch {
                expected: 0.0,
                actual: 1.0
            })
        );
    }
    assert_eq!(
        context
            .last_attempt()
            .unwrap()
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::GradientStopVisits),
        0
    );
    for geometry in [
        paint::resolve_radial_gradient(
            layout::Size::new(16.0, 8.0),
            None,
            paint::RadialGradientSize::CircleRadius(0.0),
            options(),
        )
        .result
        .unwrap(),
        paint::resolve_radial_gradient(
            layout::Size::new(16.0, 8.0),
            None,
            paint::RadialGradientSize::EllipseRadii(layout::Size::new(
                position(8.0, 0.0),
                position(0.0, 0.0),
            )),
            options(),
        )
        .result
        .unwrap(),
    ] {
        for error in [
            prepare_radial_gradient(&geometry, &used, &mut context).unwrap_err(),
            prepare_repeating_radial_gradient(&geometry, &used, &mut context).unwrap_err(),
        ] {
            assert_eq!(
                error.paint_gradient_diagnostic(),
                Some(&PaintGradientError::UnsupportedRadialCase {
                    case: geometry.case()
                })
            );
        }
    }
    let extremes = [at(r, -f64::MAX, 0.0), at(b, f64::MAX, 0.0)];
    let extremes = paint::resolve_gradient_stops(0.0, &extremes, options())
        .result
        .unwrap();
    assert!(prepare_radial_gradient(&geometry, &extremes, &mut context).is_ok());
    let error = prepare_repeating_radial_gradient(&geometry, &extremes, &mut context).unwrap_err();
    assert_eq!(
        error.paint_gradient_diagnostic(),
        Some(&PaintGradientError::UnrepresentableSpan {
            first_source_index: 0,
            last_source_index: 1
        })
    );
    for work in [
        RenderWork::CoordinateVisits,
        RenderWork::GradientStopVisits,
        RenderWork::PaintVisits,
    ] {
        context.set_options(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(work, pad.usage().work(work) - 1)
                    .unwrap(),
            ),
        );
        let error = prepare_radial_gradient(&geometry, &used, &mut context).unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
        for resource in [
            RenderResource::RetainedCpuBytes,
            RenderResource::ScratchCpuBytes,
        ] {
            assert_eq!(report.usage().live(resource), 0);
        }
        context.set_options(RenderAccountingOptions::new());
        assert_eq!(
            prepare_radial_gradient(&geometry, &used, &mut context).unwrap(),
            ordinary
        );
    }
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
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        let options = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(if timed {
                adapter_clock
            } else {
                forbidden_adapter_clock
            }));
        context.set_options(options);
        assert_eq!(
            prepare_radial_gradient(&geometry, &used, &mut context).unwrap(),
            ordinary
        );
        assert_eq!(
            context.last_attempt().unwrap().attempt().unwrap().usage(),
            pad.usage()
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .elapsed()
                .is_some(),
            timed
        );
        assert_eq!(
            prepare_repeating_radial_gradient(&geometry, &used, &mut context).unwrap(),
            repeated
        );
        assert_eq!(
            context.last_attempt().unwrap().attempt().unwrap().usage(),
            repeat.usage()
        );
        let bytes = pad.usage().peak(RenderResource::RetainedCpuBytes);
        context
            .set_options(options.with_limits(RenderLimits::new().with_retained_bytes(bytes - 1)));
        let error = prepare_radial_gradient(&geometry, &used, &mut context).unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::PreparedTransferredBytes),
            0
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        context.set_options(options);
        assert_eq!(
            prepare_radial_gradient(&geometry, &used, &mut context).unwrap(),
            ordinary
        );
    }
}

thread_local! {static ADAPTER_CLOCK:std::cell::Cell<u64>=const{std::cell::Cell::new(0)};}
fn adapter_clock() -> std::time::Duration {
    ADAPTER_CLOCK.with(|value| {
        let now = value.get();
        value.set(now + 1);
        std::time::Duration::from_nanos(now)
    })
}
fn forbidden_adapter_clock() -> std::time::Duration {
    panic!("untimed adapter must not read a clock")
}

#[test]
fn authentic_owned_distance_fields_survive_transformed_fill_stroke_and_both_glyph_routes() {
    use surgeist_render::{
        FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Shape, Stroke, TextGlyph, TextInstance,
        TextPaint, TextRun, TextRunBounds, Transform,
    };
    let r = Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap();
    let b = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let fields = |factor, repeating| {
        composed(
            0.75 * factor,
            vec![
                at(r, 0.0, 0.0),
                paint::GradientStopItem::Hint {
                    position: position(0.25 * factor, 0.0),
                },
                at(b, factor, 0.0),
            ],
            repeating,
        )
    };
    let ordinary_stroke = fields(4.0, false);
    let repeated_stroke = fields(4.0, true);
    let ordinary_glyph = fields(1.0, false);
    let repeated_glyph = fields(1.0, true);
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
    let bounds = Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap();
    for scale in [1_u32, 2] {
        let mut surface = pollster::block_on(
            renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), f64::from(scale)),
        )
        .unwrap();
        for graph in [false, true] {
            for repeating in [false, true] {
                for consumer in 0..4 {
                    let mut scene = Scene::new();
                    let draw = |scene: &mut Scene| {
                        scene.transform(Transform::translation(2.0, 4.0).unwrap(), |scene| {
                            if consumer < 2 {
                                scene.transform(
                                    Transform::try_new([2.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
                                    |scene| {
                                        let field = if repeating {
                                            repeated_stroke.clone()
                                        } else {
                                            ordinary_stroke.clone()
                                        };
                                        if consumer == 0 {
                                            scene.fill(
                                                Rect::try_new(0.0, 0.0, 12.0, 4.0).unwrap(),
                                                field,
                                            );
                                        } else {
                                            scene.stroke(
                                                Shape::rect(
                                                    Rect::try_new(0.0, 1.0, 12.0, 2.0).unwrap(),
                                                ),
                                                Stroke::try_new(2.0).unwrap(),
                                                field,
                                            );
                                        }
                                    },
                                );
                            } else {
                                let transform =
                                    Transform::try_new([8.0, 0.0, 4.0, 4.0, 0.0, 0.0]).unwrap();
                                let field = if repeating {
                                    repeated_glyph.clone()
                                } else {
                                    ordinary_glyph.clone()
                                };
                                let paint = TextPaint::try_fill(field).unwrap();
                                if consumer == 2 {
                                    scene.prepared_glyph_run(prepared.clone(), transform, paint);
                                } else {
                                    scene.text_run(
                                        TextRun::try_new(
                                            FontRef::new(2105).with_data(font.clone()),
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
                    };
                    if graph {
                        let mask = Image::from_rgba(
                            Size::try_new(1.0, 1.0).unwrap(),
                            vec![255, 255, 255, 128],
                        )
                        .unwrap();
                        scene.layer(
                            Layer::new().with_resolved_alpha_mask(
                                ResolvedLayerAlphaMask::try_new(mask, bounds).unwrap(),
                            ),
                            draw,
                        );
                    } else {
                        draw(&mut scene);
                    }
                    let stats = pollster::block_on(renderer.render(
                        &mut surface,
                        &scene,
                        Parameters::default(),
                    ))
                    .unwrap();
                    assert_eq!(
                        stats.route,
                        Some(if graph {
                            RenderRoute::GpuGraph
                        } else {
                            RenderRoute::DirectVello
                        })
                    );
                    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                    // Inverse composition at(8,6): stroke x2, origin3, radius1;
                    // glyph x.5, origin.75, radius.25. Both reach H=.25, weight.5.
                    // At(24,6) both have radius/base1.75: ordinary blue, Repeat
                    // weight sqrt(.75), yielding premul-to-straight [18,0,237,238].
                    for (x, mut expected) in [
                        (8, [85, 0, 170, 191]),
                        (
                            24,
                            if repeating {
                                [18, 0, 237, 238]
                            } else {
                                [0, 0, 255, 255]
                            },
                        ),
                    ] {
                        if graph {
                            expected[3] =
                                u8::try_from((u16::from(expected[3]) * 128 + 127) / 255).unwrap();
                        }
                        let index =
                            usize::try_from(((6 * scale) * image.size().width() + x * scale) * 4)
                                .unwrap();
                        let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
                        assert!(
                            actual
                                .into_iter()
                                .zip(expected)
                                .all(|(a, e)| a.abs_diff(e) <= 3),
                            "consumer{consumer}, repeat{repeating}, graph{graph}, scale{scale}: actual{actual:?}, expected{expected:?}"
                        );
                    }
                    if consumer >= 2 {
                        let index =
                            usize::try_from(((6 * scale) * image.size().width() + 16 * scale) * 4)
                                .unwrap();
                        assert_eq!(&image.rgba()[index..index + 4], &[0, 0, 0, 0]);
                    }
                }
            }
        }
    }
}
