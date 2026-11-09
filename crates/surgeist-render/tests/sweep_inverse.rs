#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, ErrorCode, Gradient, GradientRealizationError, GradientStop, Image,
    ImageBuffer, Layer, Options, Parameters, Point, Rect, RenderAccountingOptions,
    RenderAttemptStatus, RenderProfile, RenderResource, RenderRoute, RenderWork, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, Transform,
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

fn pixel(image: &ImageBuffer, x: u32, y: u32) -> [u8; 4] {
    let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    image.rgba()[offset..offset + 4].try_into().unwrap()
}

fn assert_cardinal_colors(image: &ImageBuffer, graph: bool, context: &str) {
    let alpha = if graph { 128 } else { 255 };
    // A full turn starts on +x and increases toward +y. These interior
    // integer-grid samples are on the four cardinal rays from (16,8).
    // The stops keep each ray well inside a constant-color plateau, so this
    // oracle does not require an exact atan or native ramp interpolation.
    let actual = [(24, 8), (16, 12), (8, 8), (16, 4)].map(|(x, y)| pixel(image, x, y));
    let expected = [
        [255, 0, 0, alpha],
        [0, 255, 0, alpha],
        [0, 0, 255, alpha],
        [255, 255, 0, alpha],
    ];
    assert_eq!(actual, expected, "{context}: +x, +y, -x, -y");
}

fn sweep_scene(graph: bool, source_scale: f64) -> Scene {
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let green = Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let yellow = Color::try_rgba(1.0, 1.0, 0.0, 1.0).unwrap();
    let sweep = Gradient::try_sweep(
        Point::try_new(16.0 * source_scale, 8.0 * source_scale).unwrap(),
        [
            (0.0, red),
            (0.05, red),
            (0.20, green),
            (0.30, green),
            (0.45, blue),
            (0.55, blue),
            (0.70, yellow),
            (0.80, yellow),
            (0.95, red),
            (1.0, red),
        ]
        .map(|(position, color)| GradientStop::try_new(position, color).unwrap())
        .into(),
    )
    .unwrap();
    scene(graph, |scene| {
        scene.transform(
            Transform::scale(source_scale.recip(), source_scale.recip()).unwrap(),
            |scene| {
                scene.fill(
                    Rect::try_new(0.0, 0.0, 32.0 * source_scale, 16.0 * source_scale).unwrap(),
                    sweep,
                );
            },
        );
    })
}

fn check_sweep_inverse(graph: bool) {
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
    let control = sweep_scene(graph, 1.0);
    let calibrated =
        pollster::block_on(renderer.render(&mut surface, &control, Parameters::default())).unwrap();
    assert_eq!(calibrated.route, Some(route));
    let control_image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_cardinal_colors(&control_image, graph, "normal sweep control");

    // A different publication makes a refusal's atomicity observable.
    let baseline = scene(graph, |scene| {
        scene.fill(bounds(), Color::try_rgba(1.0, 0.0, 1.0, 1.0).unwrap());
    });
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    assert_eq!(successful.route, Some(route));
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(
        pixel(&before, 16, 8),
        [255, 0, 255, if graph { 128 } else { 255 }],
        "prior magenta publication"
    );

    // Exact positive binary rescaling preserves device coverage and every
    // angle: the source center and rectangle are multiplied by 2^80 while
    // the scene transform is diag(2^-80,2^-80). Native forward factors remain
    // nonzero, but their f32 determinant underflows in the sweep inverse.
    let target = sweep_scene(graph, 2.0_f64.powi(80));
    match pollster::block_on(renderer.render(&mut surface, &target, Parameters::default())) {
        Ok(stats) => {
            assert_eq!(stats.route, Some(route));
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_cardinal_colors(&image, graph, "successful rescaled sweep publication");
        }
        Err(error) => {
            assert_eq!(error.code(), ErrorCode::GradientRealization);
            assert_eq!(
                error.gradient_realization_diagnostic(),
                Some(&GradientRealizationError::InsufficientNativeCapacity)
            );
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
                RenderAttemptStatus::Failed(ErrorCode::GradientRealization)
            );
            assert_eq!(report.cleanup_fault(), None);
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::Submissions), 0);
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                assert_eq!(usage.live(resource), 0, "{resource:?}");
            }
        }
    }

    let retry =
        pollster::block_on(renderer.render(&mut surface, &control, Parameters::default())).unwrap();
    assert_eq!(retry.route, Some(route));
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        control_image
    );
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
fn direct_sweep_inverse_preserves_cardinal_colors_or_refuses_atomically() {
    check_sweep_inverse(false);
}

#[test]
fn graph_sweep_inverse_preserves_cardinal_colors_or_refuses_atomically() {
    check_sweep_inverse(true);
}
