#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, FontData, FontRef, GlyphRunLimits, GlyphSynthesis, Image,
    ImageBuffer, Layer, Options, Parameters, Rect, RenderAccountingContext,
    RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderRoute, RenderWork, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, TextGlyph, TextInstance, TextPaint, TextRun,
    TextRunBounds, Transform,
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

fn assert_interior(image: &ImageBuffer, expected: [u8; 4], context: &str) {
    for x in [8, 16, 24] {
        for y in [4, 12] {
            let offset = usize::try_from((y * image.size().width() + x) * 4).unwrap();
            let actual: [u8; 4] = image.rgba()[offset..offset + 4].try_into().unwrap();
            assert_eq!(actual, expected, "{context}: pixel ({x},{y})");
        }
    }
}

fn tiny_nonempty_glyph_scene(graph: bool, red: Color) -> Scene {
    let font = FontData::try_from_bytes(
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/fonts/ahem/Ahem.ttf"
        ))
        .to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    // The authenticated Ahem X is a full-em square: UPEM1000 and source
    // ink (0,-200)..(1000,800). Prepare that real outline at canonical size1,
    // with baseline .8, to obtain its actual run-local ink proof.
    let canonical_glyphs = [TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap()];
    let canonical = font
        .prepare_glyph_run(
            &TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap(),
            &canonical_glyphs,
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let ink = canonical.bounds().ink_rect().expect("Ahem X has real ink");
    let size = 2.0_f32.powi(-43);
    let scale = f64::from(size);
    let declared_ink = TextRunBounds::try_ink(
        Rect::try_new(
            ink.x() * scale,
            ink.y() * scale,
            ink.width() * scale,
            ink.height() * scale,
        )
        .unwrap(),
    )
    .unwrap();
    let glyphs = [TextGlyph::try_new(58, 0.0, 0.8 * scale, size).unwrap()];
    // S=2^-43 and the final scale2^48 give a 32x32 device cell. Its
    // interior covers the entire 32x16 output; no tiny-provider output is
    // used to infer an empty glyph or to construct this ink rectangle.
    scene(graph, |scene| {
        scene.text_run(
            TextRun::try_new(
                FontRef::new(2111).with_data(font),
                size,
                Transform::scale(2.0_f64.powi(48), 2.0_f64.powi(48)).unwrap(),
                TextPaint::try_fill(red.into()).unwrap(),
                &glyphs,
                declared_ink,
            )
            .unwrap(),
        );
    })
}

fn check_native_glyph_precision(graph: bool) {
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
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    // Readback is straight RGBA: the graph's premultiplied red128/alpha128
    // becomes straight [255,0,0,128].
    let red_pixel = [255, 0, 0, if graph { 128 } else { 255 }];
    let calibration = scene(graph, |scene| {
        scene.fill(bounds(), red);
    });
    let stats =
        pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
            .unwrap();
    assert_eq!(stats.route, Some(route));
    let red_image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_interior(&red_image, red_pixel, "normal red calibration");
    let baseline = scene(graph, |scene| {
        scene.fill(bounds(), blue);
    });
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    assert_eq!(successful.route, Some(route));
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_interior(&before, [0, 0, 255, red_pixel[3]], "prior blue publication");

    let tiny = tiny_nonempty_glyph_scene(graph, red);
    match pollster::block_on(renderer.render(&mut surface, &tiny, Parameters::default())) {
        Ok(stats) => {
            assert_eq!(stats.route, Some(route));
            let report = renderer.take_last_attempt().unwrap();
            assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
            assert_eq!(report.cleanup_fault(), None);
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_interior(&image, red_pixel, "successful nonempty tiny-scale glyph");
        }
        Err(error) => {
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
fn direct_nonempty_glyph_survives_tiny_provider_scale_or_refuses_atomically() {
    check_native_glyph_precision(false);
}

#[test]
fn graph_nonempty_glyph_survives_tiny_provider_scale_or_refuses_atomically() {
    check_native_glyph_precision(true);
}

#[test]
fn tiny_nonempty_native_glyph_has_typed_atomic_refusal_on_both_routes() {
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
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    for graph in [false, true] {
        let route = if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        };
        let alpha = if graph { 128 } else { 255 };
        let calibration = scene(graph, |scene| {
            scene.fill(bounds(), red);
        });
        let calibrated =
            pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
                .unwrap();
        assert_eq!(calibrated.route, Some(route));
        let red_image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_interior(&red_image, [255, 0, 0, alpha], "typed red calibration");
        let baseline = scene(graph, |scene| {
            scene.fill(bounds(), blue);
        });
        let successful =
            pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
                .unwrap();
        assert_eq!(successful.route, Some(route));
        let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_interior(&before, [0, 0, 255, alpha], "typed prior blue publication");

        let tiny = tiny_nonempty_glyph_scene(graph, red);
        let error = pollster::block_on(renderer.render(&mut surface, &tiny, Parameters::default()))
            .unwrap_err();
        assert_eq!(
            error.code(),
            surgeist_render::ErrorCode::GeometryRealization
        );
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(
                &surgeist_render::GeometryRealizationError::UnrepresentableGlyph { glyph_index: 0 }
            )
        );
        assert!(error.invalid_value_diagnostic().is_none());
        assert!(error.unsupported_primitive().is_none());
        assert!(error.unresolved_resource_diagnostic().is_none());
        assert!(error.degraded_quality_diagnostic().is_none());
        assert!(error.runtime_capability_unavailable_diagnostic().is_none());
        assert!(error.accounting_diagnostic().is_none());
        assert!(error.gradient_realization_diagnostic().is_none());
        assert!(error.gradient_hint_diagnostic().is_none());
        assert!(error.glyph_preparation_diagnostic().is_none());
        #[cfg(feature = "render-paint")]
        assert!(error.paint_gradient_diagnostic().is_none());
        #[cfg(feature = "render-text")]
        assert!(error.text_layout_diagnostic().is_none());
        assert_eq!(renderer.stats(), successful);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(
            report.status(),
            RenderAttemptStatus::Failed(surgeist_render::ErrorCode::GeometryRealization)
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

        let retry =
            pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
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
