#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, ErrorCode, FontData, FontRef, Gradient, GradientRealizationError,
    GradientStop, Image, ImageBuffer, Layer, Options, Parameters, Path, Point, Rect,
    RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderResource, RenderRoute,
    RenderWork, Renderer, ResolvedLayerAlphaMask, Scene, Shape, Size, Stroke, TextGlyph, TextPaint,
    TextRun, TextRunBounds, Transform,
};

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn wrapped(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
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

fn gradient(scale: f64, center_x: f64) -> Gradient {
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    Gradient::try_sweep(
        Point::try_new(center_x * scale, 8.0 * scale).unwrap(),
        [
            (0.0, red),
            (0.05, red),
            (0.45, blue),
            (0.55, blue),
            (0.95, red),
            (1.0, red),
        ]
        .map(|(position, color)| GradientStop::try_new(position, color).unwrap())
        .into(),
    )
    .unwrap()
}

fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    image.rgba()[offset..offset + 4].try_into().unwrap()
}

#[derive(Clone, Copy, Debug)]
enum Receiver {
    Stroke,
    DeferredGlyph,
}

fn target(
    receiver: Receiver,
    graph: bool,
    scale: f64,
    invalid_center: bool,
    font: &FontData,
) -> Scene {
    let field = |center| {
        if invalid_center {
            let ordinary = gradient(1.0, center);
            Gradient::try_sweep(
                Point::try_new(f64::MAX, f64::MAX).unwrap(),
                ordinary.stops().to_vec(),
            )
            .unwrap()
        } else {
            gradient(scale, center)
        }
    };
    wrapped(graph, |scene| {
        scene.transform(
            Transform::scale(scale.recip(), scale.recip()).unwrap(),
            |scene| match receiver {
                Receiver::Stroke => {
                    let mut path = Path::new();
                    path.move_to(Point::try_new(2.0 * scale, 8.0 * scale).unwrap())
                        .line_to(Point::try_new(30.0 * scale, 8.0 * scale).unwrap());
                    scene.stroke(
                        Shape::path(path),
                        Stroke::try_new(4.0 * scale).unwrap(),
                        field(16.0),
                    );
                }
                Receiver::DeferredGlyph => {
                    let glyphs = [
                        TextGlyph::try_new(58, 0.0, 16.0 * scale, (16.0 * scale) as f32).unwrap(),
                    ];
                    scene.text_run(
                        TextRun::try_new(
                            FontRef::new(2123).with_data(font.clone()),
                            (16.0 * scale) as f32,
                            Transform::identity(),
                            TextPaint::try_fill(field(8.0).into()).unwrap(),
                            &glyphs,
                            TextRunBounds::try_ink(
                                Rect::try_new(0.0, 0.0, 20.0 * scale, 24.0 * scale).unwrap(),
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    );
                }
            },
        );
    })
}

#[test]
fn path_stroke_and_deferred_glyph_sweeps_qualify_their_actual_native_factors_on_both_routes() {
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
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
        let route = if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        };
        let alpha = if graph { 128 } else { 255 };
        for receiver in [Receiver::Stroke, Receiver::DeferredGlyph] {
            let control = target(receiver, graph, 1.0, false, &font);
            let successful =
                pollster::block_on(renderer.render(&mut surface, &control, Parameters::default()))
                    .unwrap();
            assert_eq!(successful.route, Some(route));
            let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            let (left, right) = match receiver {
                Receiver::Stroke => (8, 24),
                Receiver::DeferredGlyph => (4, 12),
            };
            assert_eq!(
                pixel(&before, left, 8),
                [0, 0, 255, alpha],
                "{receiver:?} normal -x"
            );
            assert_eq!(
                pixel(&before, right, 8),
                [255, 0, 0, alpha],
                "{receiver:?} normal +x"
            );
            let error = pollster::block_on(renderer.render(
                &mut surface,
                &target(receiver, graph, 1.0, true, &font),
                Parameters::default(),
            ))
            .unwrap_err();
            assert_eq!(
                error.code(),
                ErrorCode::GradientRealization,
                "{receiver:?}: {error:?}"
            );
            assert_eq!(
                error.gradient_realization_diagnostic(),
                Some(&GradientRealizationError::InsufficientNativeCapacity)
            );
            assert!(error.accounting_diagnostic().is_none());
            assert!(error.geometry_realization_diagnostic().is_none());
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
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::Submissions), 0);
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
            let retry =
                pollster::block_on(renderer.render(&mut surface, &control, Parameters::default()))
                    .unwrap();
            assert_eq!(retry.route, Some(route));
            assert_eq!(
                pollster::block_on(renderer.read_headless(&surface)).unwrap(),
                before
            );
            assert_eq!(renderer.take_last_attempt().unwrap().cleanup_fault(), None);
        }
    }
}

#[test]
fn one_stop_and_empty_sweeps_do_not_require_an_unused_inverse_or_center() {
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let one = Gradient::try_sweep(
        Point::try_new(f64::MAX, f64::MAX).unwrap(),
        vec![GradientStop::try_new(0.5, red).unwrap()],
    )
    .unwrap();
    let many = Gradient::try_sweep(
        Point::try_new(f64::MAX, f64::MAX).unwrap(),
        vec![
            GradientStop::try_new(0.0, red).unwrap(),
            GradientStop::try_new(1.0, Color::BLACK).unwrap(),
        ],
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
        let scene = wrapped(graph, |scene| {
            scene.transform(
                Transform::scale(2.0_f64.powi(-80), 2.0_f64.powi(-80)).unwrap(),
                |scene| {
                    scene.fill(
                        Rect::try_new(0.0, 0.0, 32.0 * 2.0_f64.powi(80), 16.0 * 2.0_f64.powi(80))
                            .unwrap(),
                        one.clone(),
                    );
                    scene.fill(Shape::path(Path::new()), many.clone());
                    scene.stroke(
                        Shape::path(Path::new()),
                        Stroke::try_new(4.0).unwrap(),
                        many.clone(),
                    );
                },
            );
        });
        let stats =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
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
        for (x, y) in [(4, 4), (16, 8), (28, 12)] {
            assert_eq!(
                pixel(&image, x, y),
                [255, 0, 0, if graph { 128 } else { 255 }]
            );
        }
    }
}
