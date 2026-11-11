use surgeist_color::{
    Color as ResolvedColor, ColorSpace, Finite, PredefinedColorSpace, UnitInterval,
};
use surgeist_render::{
    Color, RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind,
    RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderQuantity, RenderWork,
    prepare_sdr_srgb_color,
};

fn oklch(l: f64, c: f64, h: f64, alpha: f64) -> ResolvedColor {
    ResolvedColor::new(
        ColorSpace::Oklch,
        [l, c, h].map(Finite::new),
        UnitInterval::new(alpha),
    )
}

fn rgb(color: Color) -> [f32; 3] {
    [color.r(), color.g(), color.b()]
}

#[test]
fn sdr_color_preserves_srgb_and_treats_missing_components_as_zero() {
    let mut accounting = RenderAccountingContext::default();
    let resolved = ResolvedColor::from_srgba8([17, 128, 240, 63]);
    let output = prepare_sdr_srgb_color(resolved, &mut accounting).unwrap();
    assert_eq!(rgb(output), [17.0 / 255.0, 128.0 / 255.0, 240.0 / 255.0]);
    assert_eq!(output.a(), 63.0 / 255.0);

    let resolved = ResolvedColor::new(
        ColorSpace::Predefined(PredefinedColorSpace::Srgb),
        [Finite::new(0.25), None, Finite::new(0.75)],
        None,
    );
    let output = prepare_sdr_srgb_color(resolved, &mut accounting).unwrap();
    assert_eq!(rgb(output), [0.25, 0.0, 0.75]);
    assert_eq!(output.a(), 0.0);

    let output = prepare_sdr_srgb_color(oklch(0.5, 0.2, 270.0, 1.0), &mut accounting).unwrap();
    // Reference sRGB conversion from the WPT oklch-005 reftest.
    for (actual, expected) in rgb(output).into_iter().zip([0.23056, 0.3173, 0.82628]) {
        assert!((actual - expected).abs() < 0.0001);
    }
}

#[test]
fn sdr_color_renders_missing_oklch_hue_as_zero_degrees() {
    let mut accounting = RenderAccountingContext::default();
    let source = ResolvedColor::new(
        ColorSpace::Oklch,
        [Finite::new(0.5), Finite::new(0.1), None],
        UnitInterval::new(0.75),
    );
    let output = prepare_sdr_srgb_color(source, &mut accounting).unwrap();
    let explicit = prepare_sdr_srgb_color(oklch(0.5, 0.1, 0.0, 0.75), &mut accounting).unwrap();
    assert_eq!(output, explicit);
    assert!(output.r() - output.g() > 0.25);
}

#[test]
fn sdr_color_matches_independent_local_minde_vectors() {
    let mut accounting = RenderAccountingContext::default();
    // Local MINDE reference channels in units of 1/255 from the WPT
    // sRGB canvas cases, at 93903eabea14594dd96cf19694735e3144e20364.
    for ((l, c, h), expected) in [
        ((0.8, 0.35, 30.0), [255.0, 153.55, 135.55]),
        ((0.8, 0.35, 300.0), [204.30, 167.06, 255.0]),
        ((0.4, 0.35, 210.0), [0.0, 83.62, 97.90]),
    ] {
        let output = prepare_sdr_srgb_color(oklch(l, c, h, 0.375), &mut accounting).unwrap();
        for (actual, expected) in rgb(output).into_iter().zip(expected) {
            assert!((actual * 255.0 - expected).abs() <= 0.1);
        }
        assert_eq!(output.a(), 0.375);
    }
}

#[test]
fn sdr_color_handles_lightness_extremes_and_finite_search_inputs() {
    let mut accounting = RenderAccountingContext::default();
    for (lightness, expected) in [(-0.1, [0.0; 3]), (1.1, [1.0; 3])] {
        let output =
            prepare_sdr_srgb_color(oklch(lightness, 0.35, 30.0, 0.25), &mut accounting).unwrap();
        assert_eq!(rgb(output), expected);
        assert_eq!(output.a(), 0.25);
    }

    let positive = prepare_sdr_srgb_color(oklch(0.6, 0.35, 30.0, 1.0), &mut accounting).unwrap();
    let negative = prepare_sdr_srgb_color(oklch(0.6, -0.35, 210.0, 1.0), &mut accounting).unwrap();
    for (a, b) in rgb(positive).into_iter().zip(rgb(negative)) {
        assert!((a - b).abs() <= 1e-6);
    }
    let extreme = prepare_sdr_srgb_color(oklch(0.6, f64::MAX, 30.0, 1.0), &mut accounting).unwrap();
    assert!(
        rgb(extreme)
            .iter()
            .all(|channel| (0.0..=1.0).contains(channel))
    );
}

#[test]
fn sdr_color_reports_the_accepted_prefix_and_times_the_retry() {
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: true });
    let mut accounting = RenderAccountingContext::new(
        options.with_limits(
            RenderLimits::new()
                .with_work_limit(RenderWork::CoordinateVisits, 43)
                .unwrap(),
        ),
    );
    let source = oklch(0.8, 0.35, 30.0, 0.5);
    let error = prepare_sdr_srgb_color(source, &mut accounting).unwrap_err();
    assert!(error.accounting_diagnostic().is_some());
    let terminal = accounting.last_attempt().unwrap();
    let attempt = terminal.attempt().unwrap();
    assert_eq!(terminal.cleanup_fault(), None);
    assert_eq!(attempt.kind(), RenderAttemptKind::SdrColor);
    assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
    let refusal = attempt.refusal().unwrap();
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::CoordinateVisits)
    );
    assert_eq!((refusal.current(), refusal.amount()), (43, 3));

    accounting.set_options(options);
    let output = prepare_sdr_srgb_color(source, &mut accounting).unwrap();
    assert_eq!(output.a(), 0.5);
    let attempt = accounting.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(attempt.status(), RenderAttemptStatus::Succeeded);
    assert!(attempt.phase_elapsed(RenderPhase::Coordinates).is_some());
    let usage = attempt.phase_usage(RenderPhase::Coordinates);
    assert_eq!(usage.work(RenderWork::PaintVisits), 1);
    assert!(usage.work(RenderWork::CoordinateVisits) > 43);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn sdr_color_reaches_native_scene_paint_with_source_over_alpha() {
    use surgeist_render::{Layer, Options, Parameters, Rect, RenderRoute, Renderer, Scene, Size};

    let mut accounting = RenderAccountingContext::default();
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(2.0, 1.0).unwrap(), 1.0))
            .unwrap();
    let colors = [1.0, 0.5].map(|alpha| {
        prepare_sdr_srgb_color(oklch(0.8, 0.35, 30.0, alpha), &mut accounting).unwrap()
    });
    let fill = |scene: &mut Scene| {
        for (x, color) in [0.0, 1.0].into_iter().zip(colors) {
            scene.fill(Rect::try_new(x, 0.0, 1.0, 1.0).unwrap(), color);
        }
    };
    let parameters = Parameters {
        base_color: Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
        ..Parameters::default()
    };
    for route in [RenderRoute::DirectVello, RenderRoute::GpuGraph] {
        let mut scene = Scene::new();
        if route == RenderRoute::GpuGraph {
            scene.layer(Layer::new().isolated(), fill);
        } else {
            fill(&mut scene);
        }
        let stats = pollster::block_on(renderer.render(&mut surface, &scene, parameters)).unwrap();
        assert_eq!(stats.route, Some(route));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        // At half alpha, source-over onto opaque blue gives about [127.5, 76.8, 195.3].
        let expected = [255, 154, 136, 255, 128, 77, 195, 255];
        assert_eq!(image.rgba().len(), expected.len());
        for (&actual, expected) in image.rgba().iter().zip(expected) {
            assert!(
                actual.abs_diff(expected) <= 2,
                "{route:?} pixels: {:?}",
                image.rgba()
            );
        }
    }
}
