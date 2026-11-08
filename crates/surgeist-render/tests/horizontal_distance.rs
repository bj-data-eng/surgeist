#![forbid(unsafe_code)]

use surgeist_render::{Color, Gradient, GradientHint, GradientStop, Point, RepeatingGradient};

fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}
fn records(values: &[(f64, Color, Option<f64>)]) -> Vec<GradientStop> {
    values
        .iter()
        .map(|&(position, color, hint)| {
            let stop = GradientStop::try_new(position, color).unwrap();
            hint.map_or(stop, |hint| {
                stop.with_hint_after(GradientHint::try_new(hint).unwrap())
            })
        })
        .collect()
}

#[test]
fn physical_distance_models_preserve_controls_and_expose_only_their_actual_geometry() {
    let red = color(1.0, 0.0, 0.0, 1.0);
    let blue = color(0.0, 0.0, 1.0, 1.0);
    let origin = f64::from_bits(8.0_f64.to_bits() + 1);
    let stops = records(&[(-4.0, red, Some(0.25)), (12.0, blue, None)]);
    let ordinary = Gradient::try_horizontal_distance(origin, stops.clone()).unwrap();
    let repeating = RepeatingGradient::try_horizontal_distance(origin, stops.clone()).unwrap();
    assert_eq!(ordinary.horizontal_distance_origin(), Some(origin));
    assert_eq!(repeating.horizontal_distance_origin(), Some(origin));
    assert_eq!(ordinary.stops(), stops);
    assert_eq!(repeating.stops(), stops);
    assert!(
        ordinary.linear_points().is_none()
            && ordinary.radial_geometry().is_none()
            && ordinary.sweep_center().is_none()
    );
    assert!(repeating.linear_points().is_none() && repeating.radial_geometry().is_none());
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for error in [
            Gradient::try_horizontal_distance(bad, stops.clone()).unwrap_err(),
            RepeatingGradient::try_horizontal_distance(bad, stops.clone()).unwrap_err(),
        ] {
            assert!(error.invalid_value_diagnostic().is_some());
        }
    }
    for stops in [vec![], records(&[(2.0, red, None), (1.0, blue, None)])] {
        assert!(Gradient::try_horizontal_distance(0.0, stops.clone()).is_err());
        assert!(RepeatingGradient::try_horizontal_distance(0.0, stops).is_err());
    }
    let overflow = records(&[(-f64::MAX, red, None), (f64::MAX, blue, None)]);
    assert!(Gradient::try_horizontal_distance(0.0, overflow.clone()).is_ok());
    assert!(RepeatingGradient::try_horizontal_distance(0.0, overflow).is_err());
    for (stops, expected) in [
        (
            records(&[(0.0, red, Some(3.0)), (2.0, blue, None)]),
            surgeist_render::GradientHintError::HintOutsideInterval { stop_index: 0 },
        ),
        (
            records(&[(0.0, red, None), (2.0, blue, Some(2.0))]),
            surgeist_render::GradientHintError::TrailingHint { stop_index: 1 },
        ),
    ] {
        for error in [
            Gradient::try_horizontal_distance(0.0, stops.clone()).unwrap_err(),
            RepeatingGradient::try_horizontal_distance(0.0, stops).unwrap_err(),
        ] {
            assert_eq!(error.gradient_hint_diagnostic(), Some(&expected));
        }
    }
    assert!(
        Gradient::try_linear(
            Point::try_new(0.0, 0.0).unwrap(),
            Point::try_new(1.0, 0.0).unwrap(),
            stops
        )
        .unwrap()
        .horizontal_distance_origin()
        .is_none()
    );
}

#[cfg(not(target_arch = "wasm32"))]
mod gpu {
    use super::*;
    use surgeist_render::{
        EffectQualityPolicy, Image, ImageBuffer, Layer, Options, Paint, Parameters, Rect,
        RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Size, Surface, Transform,
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
    fn renderer() -> (Renderer, Surface) {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let surface =
            pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
                .unwrap();
        (renderer, surface)
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
        let tolerance = if graph { 3 } else { 2 };
        assert!(
            actual
                .into_iter()
                .zip(expected)
                .all(|(a, e)| a.abs_diff(e) <= tolerance),
            "distance({x},{y}), graph={graph}: actual{actual:?}, expected{expected:?}, tolerance{tolerance}"
        );
    }
    fn ordinary(origin: f64, values: &[(f64, Color, Option<f64>)]) -> Paint {
        Gradient::try_horizontal_distance(origin, records(values))
            .unwrap()
            .into()
    }
    fn repeating(origin: f64, values: &[(f64, Color, Option<f64>)]) -> Paint {
        RepeatingGradient::try_horizontal_distance(origin, records(values))
            .unwrap()
            .into()
    }

    #[test]
    fn negative_controls_coincident_runs_endpoint_hints_and_tiny_steps_use_folded_distance() {
        let r = color(1.0, 0.0, 0.0, 1.0);
        let g = color(0.0, 1.0, 0.0, 1.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let y = color(1.0, 1.0, 0.0, 1.0);
        // Samples are (physical distance from origin16, independent RGBA).
        let cases = [
            (
                ordinary(16.0, &[(-4.0, r, None), (4.0, g, None), (12.0, b, None)]),
                vec![
                    (0, [128, 128, 0, 255]),
                    (4, [0, 255, 0, 255]),
                    (8, [0, 128, 128, 255]),
                    (12, [0, 0, 255, 255]),
                ],
            ),
            (
                ordinary(16.0, &[(4.0, r, None), (4.0, b, None)]),
                vec![
                    (0, [255, 0, 0, 255]),
                    (3, [255, 0, 0, 255]),
                    (4, [0, 0, 255, 255]),
                    (5, [0, 0, 255, 255]),
                ],
            ),
            (
                ordinary(
                    16.0,
                    &[(f64::from_bits(1), r, None), (f64::from_bits(1), b, None)],
                ),
                vec![(0, [255, 0, 0, 255]), (1, [0, 0, 255, 255])],
            ),
            (
                ordinary(16.0, &[(-3.0, r, None), (-1.0, b, None)]),
                vec![(0, [0, 0, 255, 255]), (4, [0, 0, 255, 255])],
            ),
            (
                ordinary(16.0, &[(0.0, r, None), (0.0, b, None)]),
                vec![(0, [0, 0, 255, 255]), (4, [0, 0, 255, 255])],
            ),
            (
                ordinary(
                    16.0,
                    &[
                        (0.0, r, None),
                        (4.0, g, None),
                        (4.0, b, None),
                        (8.0, y, None),
                    ],
                ),
                vec![
                    (2, [128, 128, 0, 255]),
                    (4, [0, 0, 255, 255]),
                    (6, [128, 128, 128, 255]),
                ],
            ),
            (
                ordinary(16.0, &[(0.0, r, Some(2.0)), (8.0, b, None)]),
                vec![(2, [128, 0, 128, 255]), (4, [75, 0, 180, 255])],
            ),
            (
                ordinary(16.0, &[(0.0, r, Some(0.0)), (8.0, b, None)]),
                vec![
                    (0, [255, 0, 0, 255]),
                    (1, [0, 0, 255, 255]),
                    (8, [0, 0, 255, 255]),
                ],
            ),
            (
                ordinary(16.0, &[(0.0, r, Some(8.0)), (8.0, b, None)]),
                vec![
                    (0, [255, 0, 0, 255]),
                    (7, [255, 0, 0, 255]),
                    (8, [0, 0, 255, 255]),
                ],
            ),
            (
                ordinary(16.0, &[(-4.0, r, Some(0.0)), (4.0, b, None)]),
                vec![(0, [128, 0, 128, 255]), (4, [0, 0, 255, 255])],
            ),
            // A single color bypasses unused origin/field qualification.
            (
                ordinary(f64::MAX, &[(f64::MAX, r, None)]),
                vec![(0, [255, 0, 0, 255]), (4, [255, 0, 0, 255])],
            ),
            (
                ordinary(16.0, &[(0.0, r, None), (8.0, r, None)]),
                vec![(0, [255, 0, 0, 255]), (4, [255, 0, 0, 255])],
            ),
        ];
        let (mut renderer, mut surface) = renderer();
        for graph in [false, true] {
            for (field, samples) in &cases {
                let scene = scene(graph, |scene| {
                    scene.fill(bounds(), field.clone());
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                for &(distance, expected) in samples {
                    for x in [16 - distance, 16 + distance] {
                        for y in [4, 12] {
                            pixel(&image, x, y, expected, graph);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn repeat_zero_span_weighted_average_and_two_pixel_threshold_remain_distinct() {
        let r = color(1.0, 0.0, 0.0, 1.0);
        let transparent = color(0.0, 1.0, 0.0, 0.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let cases = [
            // Trapezoid weights(.125,.5,.375), premul(.125,0,.375,.5).
            // The quarter-interval hint is nonlinear; integrating its power
            // curve would produce a different color and alpha.
            (
                repeating(
                    16.0,
                    &[
                        (0.0, r, Some(0.0625)),
                        (0.25, transparent, None),
                        (1.0, b, None),
                    ],
                ),
                vec![(0, [64, 0, 191, 128]), (3, [64, 0, 191, 128])],
            ),
            // True zero span replaces actual colors uniformly, excluding hints.
            (
                repeating(
                    16.0,
                    &[
                        (4.0, r, Some(4.0)),
                        (4.0, transparent, None),
                        (4.0, b, None),
                    ],
                ),
                vec![(0, [128, 0, 128, 128]), (3, [128, 0, 128, 128])],
            ),
            (
                repeating(16.0, &[(0.0, r, None), (1.99, b, None)]),
                vec![(0, [128, 0, 128, 255]), (1, [128, 0, 128, 255])],
            ),
            (
                repeating(16.0, &[(0.0, r, None), (2.0, b, None)]),
                vec![
                    (0, [255, 0, 0, 255]),
                    (1, [128, 0, 128, 255]),
                    (2, [255, 0, 0, 255]),
                ],
            ),
            (
                repeating(16.0, &[(-4.0, r, None), (4.0, b, None)]),
                vec![
                    (0, [128, 0, 128, 255]),
                    (2, [64, 0, 191, 255]),
                    (4, [255, 0, 0, 255]),
                ],
            ),
        ];
        let (mut renderer, mut surface) = renderer();
        for graph in [false, true] {
            for (field, samples) in &cases {
                let scene = scene(graph, |scene| {
                    scene.fill(bounds(), field.clone());
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                for &(d, expected) in samples {
                    for x in [16 - d, 16 + d] {
                        pixel(&image, x, 4, expected, graph);
                    }
                }
            }
        }
    }

    #[test]
    fn inverse_shear_rotation_reflection_and_anisotropy_preserve_distance_and_repeat_spacing() {
        let r = color(1.0, 0.0, 0.0, 1.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let field = ordinary(8.0, &[(2.0, r, None), (6.0, b, None)]);
        let cases = [
            (
                Transform::try_new([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
                (12, 4),
                [(8, 4), (16, 4)],
            ),
            (
                Transform::try_new([-1.0, 0.0, 0.0, 1.0, 24.0, 0.0]).unwrap(),
                (16, 4),
                [(12, 4), (20, 4)],
            ),
            (
                Transform::try_new([0.0, 1.0, -1.0, 0.0, 16.0, 0.0]).unwrap(),
                (12, 8),
                [(12, 4), (12, 12)],
            ),
            (
                Transform::scale(2.0, 1.0).unwrap(),
                (16, 4),
                [(8, 4), (24, 4)],
            ),
        ];
        let (mut renderer, mut surface) = renderer();
        for graph in [false, true] {
            for &(transform, center, sides) in &cases {
                let scene = scene(graph, |scene| {
                    scene.transform(transform, |scene| {
                        scene.fill(Rect::try_new(0.0, 0.0, 16.0, 8.0).unwrap(), field.clone());
                    });
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                pixel(&image, center.0, center.1, [255, 0, 0, 255], graph);
                for (x, y) in sides {
                    pixel(&image, x, y, [128, 0, 128, 255], graph);
                }
            }
            for (period, expected) in [(4.0, [191, 0, 64, 255]), (2.0, [128, 0, 128, 255])] {
                let field = repeating(8.0, &[(1.0, r, None), (1.0 + period, b, None)]);
                let scene = scene(graph, |scene| {
                    scene.transform(
                        Transform::try_new([1.0, 0.0, 1.0, 1.0, 0.0, 0.0]).unwrap(),
                        |scene| {
                            scene.fill(Rect::try_new(0.0, 0.0, 16.0, 8.0).unwrap(), field);
                        },
                    );
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                // q at device(14,4)/(10,4) has r2. Spacing T/sqrt2 makes
                // period4 patterned (u=.25), while period2 selects its average.
                for x in [10, 14] {
                    pixel(&image, x, 4, expected, graph);
                }
                // At r1, either period's unaveraged pattern would be an
                // exact red seam. Purple here therefore proves that the
                // period2 shear case actually selected the average.
                let seam = if period == 4.0 {
                    [255, 0, 0, 255]
                } else {
                    [128, 0, 128, 255]
                };
                for x in [11, 13] {
                    pixel(&image, x, 4, seam, graph);
                }
            }
        }
    }

    #[test]
    fn native_control_and_phase_losses_refuse_without_publication_and_allow_retry() {
        use surgeist_render::{
            ErrorCode, GradientRealizationError, RenderAccountingOptions, RenderAttemptStatus,
            RenderProfile, RenderWork,
        };
        let r = color(1.0, 0.0, 0.0, 1.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let next = f64::from_bits(4.0_f64.to_bits() + 1);
        let before = f64::from_bits(8.0_f64.to_bits() - 1);
        let cases = [
            (
                ordinary(
                    16.0,
                    &[
                        (4.0, r, None),
                        (next, color(0.0, 1.0, 0.0, 1.0), None),
                        (8.0, b, None),
                    ],
                ),
                GradientRealizationError::UnrepresentableStop { stop_index: 1 },
            ),
            (
                ordinary(16.0, &[(0.0, r, Some(before)), (8.0, b, None)]),
                GradientRealizationError::UnrepresentableHint { stop_index: 0 },
            ),
            (
                repeating(16.0, &[(f64::from_bits(1), r, None), (4.0, b, None)]),
                GradientRealizationError::InsufficientNativeCapacity,
            ),
            (
                repeating(16.0, &[(2.0_f64.powi(-127), r, None), (4.0, b, None)]),
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
            let baseline = scene(graph, |scene| {
                scene.fill(bounds(), color(0.0, 1.0, 0.0, 1.0));
            });
            let before = read(&mut renderer, &mut surface, &baseline, graph);
            let stats = renderer.stats();
            for (field, expected) in &cases {
                let refused = scene(graph, |scene| {
                    scene.fill(bounds(), field.clone());
                });
                let error = pollster::block_on(renderer.render(
                    &mut surface,
                    &refused,
                    Parameters::default(),
                ))
                .unwrap_err();
                assert_eq!(error.code(), ErrorCode::GradientRealization);
                assert_eq!(error.gradient_realization_diagnostic(), Some(expected));
                assert_eq!(renderer.stats(), stats);
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
            let retry = scene(graph, |scene| {
                scene.fill(bounds(), ordinary(16.0, &[(2.0, r, None), (6.0, b, None)]));
            });
            let image = read(&mut renderer, &mut surface, &retry, graph);
            pixel(&image, 12, 4, [128, 0, 128, 255], graph);
        }
    }

    #[test]
    fn distance_program_copy_caps_and_profiles_preserve_publication_and_fresh_retry() {
        use surgeist_render::{
            RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
            RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity, RenderWork,
        };
        let r = color(1.0, 0.0, 0.0, 1.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let positive = scene(false, |scene| {
            scene.fill(
                bounds(),
                Gradient::try_linear(
                    Point::try_new(0.0, 0.0).unwrap(),
                    Point::try_new(8.0, 0.0).unwrap(),
                    records(&[(0.0, r, None), (1.0, b, None)]),
                )
                .unwrap(),
            );
        });
        let distance = scene(false, |scene| {
            scene.fill(bounds(), ordinary(16.0, &[(1.0, r, None), (5.0, b, None)]));
        });
        let make = |accounting| {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default().with_accounting(accounting),
            ))
            .unwrap();
            let surface = pollster::block_on(
                renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0),
            )
            .unwrap();
            (renderer, surface)
        };
        let (mut renderer, mut surface) = make(
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
        );
        let before = read(&mut renderer, &mut surface, &positive, false);
        let prior = renderer.take_last_attempt().unwrap();
        let expected = read(&mut renderer, &mut surface, &distance, false);
        let full = renderer.take_last_attempt().unwrap();
        let expected_usage = full.attempt().unwrap().usage();
        let copies = prior
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::RasterResourceCopies);
        // The unchanged positive Pad field needs no owned program-record copy.
        // Distance owns its actual records before the common endpoint push.
        assert!(copies > 0);
        assert!(expected_usage.work(RenderWork::RasterResourceCopies) > copies);
        pixel(&expected, 18, 4, [191, 0, 64, 255], false);
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
        ] {
            let options = RenderAccountingOptions::new().with_profile(profile);
            let (mut renderer, mut surface) = make(options);
            read(&mut renderer, &mut surface, &positive, false);
            let image = read(&mut renderer, &mut surface, &distance, false);
            assert_eq!(image, expected);
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.attempt().unwrap().usage(), expected_usage);
            assert_eq!(
                report.attempt().unwrap().elapsed().is_some(),
                matches!(profile, RenderProfile::Summary { timing: true })
            );
            drop(surface);
            drop(renderer);
            let limits = RenderLimits::new()
                .with_work_limit(RenderWork::RasterResourceCopies, copies)
                .unwrap();
            let (mut renderer, mut surface) = make(options.with_limits(limits));
            assert_eq!(read(&mut renderer, &mut surface, &positive, false), before);
            let stats = renderer.stats();
            let error =
                pollster::block_on(renderer.render(&mut surface, &distance, Parameters::default()))
                    .unwrap_err();
            assert!(
                matches!(error.accounting_diagnostic(),Some(RenderAccountingError::Refused(refusal))if refusal.quantity()==RenderQuantity::Work(RenderWork::RasterResourceCopies))
            );
            assert_eq!(renderer.stats(), stats);
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
                    .work(RenderWork::RasterResourceCopies)
                    <= copies
            );
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions),
                0
            );
            assert_eq!(read(&mut renderer, &mut surface, &positive, false), before);
            assert!(
                renderer
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::Submissions)
                    > 0
            );
        }
    }

    #[test]
    fn step_power_and_phase_fixtures_execute_in_every_fine_antialiasing_variant() {
        use surgeist_render::Antialiasing;
        let r = color(1.0, 0.0, 0.0, 1.0);
        let b = color(0.0, 0.0, 1.0, 1.0);
        let cases = [
            (
                ordinary(16.0, &[(4.0, r, None), (4.0, b, None)]),
                vec![
                    (16, [255, 0, 0, 255]),
                    (12, [0, 0, 255, 255]),
                    (20, [0, 0, 255, 255]),
                ],
            ),
            (
                ordinary(16.0, &[(0.0, r, Some(2.0)), (8.0, b, None)]),
                vec![(14, [128, 0, 128, 255]), (18, [128, 0, 128, 255])],
            ),
            (
                repeating(16.0, &[(1.0, r, None), (5.0, b, None)]),
                vec![
                    (16, [64, 0, 191, 255]),
                    (15, [255, 0, 0, 255]),
                    (17, [255, 0, 0, 255]),
                ],
            ),
        ];
        for antialiasing in [
            Antialiasing::Area,
            Antialiasing::Msaa8,
            Antialiasing::Msaa16,
        ] {
            let mut renderer = pollster::block_on(Renderer::new(
                Options::default()
                    .with_antialiasing(antialiasing)
                    .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
            ))
            .unwrap();
            let mut surface = pollster::block_on(
                renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0),
            )
            .unwrap();
            for graph in [false, true] {
                for (field, samples) in &cases {
                    let scene = scene(graph, |scene| {
                        scene.fill(bounds(), field.clone());
                    });
                    let image = read(&mut renderer, &mut surface, &scene, graph);
                    for &(x, expected) in samples {
                        pixel(&image, x, 4, expected, graph);
                    }
                }
            }
        }
    }
}
