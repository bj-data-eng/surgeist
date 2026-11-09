#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, FontData, GlyphRunLimits, GlyphSynthesis, Image, ImageBuffer,
    Layer, Options, Parameters, PathElement, PreparedGlyphRun, Rect, RenderAccountingContext,
    RenderAccountingOptions, RenderAttemptStatus, RenderProfile, RenderRoute, RenderWork, Renderer,
    ResolvedLayerAlphaMask, Scene, Size, TextGlyph, TextInstance, TextPaint, TextRunBoundsKind,
    Transform,
};

const AHEM: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");
const X: u32 = 58;

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn table_offset(bytes: &[u8], tag: &[u8; 4]) -> usize {
    (0..usize::from(u16_at(bytes, 4)))
        .find_map(|index| {
            let record = 12 + index * 16;
            (&bytes[record..record + 4] == tag).then(|| {
                usize::try_from(u32::from_be_bytes(
                    bytes[record + 8..record + 12].try_into().unwrap(),
                ))
                .unwrap()
            })
        })
        .expect("the authenticated Ahem fixture contains this table")
}

fn point_only_outline() -> PreparedGlyphRun {
    // Use the same in-memory Ahem glyf derivative convention as
    // prepared_outline.rs and prepared_outline/static_composite.rs. The licensed
    // original and every other glyph/table remain unchanged. This is a real X
    // outline with one contour and one ON point, rather than an empty space.
    assert_eq!(u16_at(AHEM, table_offset(AHEM, b"head") + 50), 0);
    let loca = table_offset(AHEM, b"loca");
    let glyf = table_offset(AHEM, b"glyf");
    let index = usize::try_from(X).unwrap();
    let start = glyf + usize::from(u16_at(AHEM, loca + index * 2)) * 2;
    let end = glyf + usize::from(u16_at(AHEM, loca + (index + 1) * 2)) * 2;
    // OpenType simple glyph: one contour; bbox (0,0)..(0,0); endpoint0;
    // instructionLength0; ON_CURVE|X_IS_SAME|Y_IS_SAME. No coordinate bytes.
    let mut point = [1i16, 0, 0, 0, 0]
        .into_iter()
        .flat_map(i16::to_be_bytes)
        .collect::<Vec<_>>();
    point.extend(0u16.to_be_bytes());
    point.extend(0u16.to_be_bytes());
    point.push(0x31);
    assert!(point.len() <= end - start);
    let mut bytes = AHEM.to_vec();
    bytes[start..end].fill(0);
    bytes[start..start + point.len()].copy_from_slice(&point);
    // The loca span retains padding after the complete declared point streams.
    // Like existing authored SFNT derivatives, public admission validates these
    // bytes; this fixture does not claim a general font-conformance certificate.
    let font = FontData::try_from_bytes(bytes, 0, surgeist_test::fixtures::FONT_LIMITS)
        .expect("admit the actual single-point Ahem derivative");
    let glyph = TextGlyph::try_new(X, 0.0, 0.0, 1000.0).unwrap();
    let prepared = font
        .prepare_glyph_run(
            &TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap(),
            &[glyph],
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .expect("prepare the point-only contour through the public owner");
    assert_eq!(prepared.glyphs(), [glyph]);
    assert_eq!(prepared.bounds().kind(), TextRunBoundsKind::Empty);
    assert!(prepared.path_elements().next().is_some());
    assert!(
        prepared
            .path_elements()
            .all(|element| matches!(element, PathElement::MoveTo(_) | PathElement::Close))
    );
    prepared
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

fn assert_solid(image: &ImageBuffer, expected: [u8; 4]) {
    let (pixels, remainder) = image.rgba().as_chunks::<4>();
    assert!(remainder.is_empty());
    for &pixel in pixels {
        assert_eq!(pixel, expected);
    }
}

fn pixel(image: &ImageBuffer, x: usize, y: usize) -> &[u8] {
    let offset = (y * usize::try_from(image.size().width()).unwrap() + x) * 4;
    &image.rgba()[offset..offset + 4]
}

fn check_point_only_placement(graph: bool, profile: RenderProfile) {
    let prepared = point_only_outline();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
            .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    let route = if graph {
        RenderRoute::GpuGraph
    } else {
        RenderRoute::DirectVello
    };
    let alpha = if graph { 128 } else { 255 };

    // A point-only occurrence between two real A contours must not discard the
    // earlier paint or the later contour. Ahem A is an exact em square; at size8
    // and baseline8 its ink covers y1.6..9.6, x0..8 and x16..24.
    let mixed = prepared
        .font()
        .prepare_glyph_run(
            &TextInstance::try_new(8.0, vec![], GlyphSynthesis::none()).unwrap(),
            &[
                TextGlyph::try_new(35, 0.0, 8.0, 8.0).unwrap(),
                TextGlyph::try_new(X, 12.0, 8.0, 8.0).unwrap(),
                TextGlyph::try_new(35, 16.0, 8.0, 8.0).unwrap(),
            ],
            GlyphRunLimits::new(3, 32, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let mixed_scene = scene(graph, |scene| {
        scene.fill(bounds(), blue);
        scene.prepared_glyph_run(
            mixed,
            Transform::identity(),
            TextPaint::try_fill(red.into()).unwrap(),
        );
    });
    let mixed_stats =
        pollster::block_on(renderer.render(&mut surface, &mixed_scene, Parameters::default()))
            .unwrap();
    assert_eq!(mixed_stats.route, Some(route));
    let mixed_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(pixel(&mixed_pixels, 4, 4), [255, 0, 0, alpha]);
    assert_eq!(pixel(&mixed_pixels, 12, 4), [0, 0, 255, alpha]);
    assert_eq!(pixel(&mixed_pixels, 20, 4), [255, 0, 0, alpha]);

    let calibration = scene(graph, |scene| {
        scene.fill(bounds(), red);
    });
    let prior_stats =
        pollster::block_on(renderer.render(&mut surface, &calibration, Parameters::default()))
            .unwrap();
    assert_eq!(prior_stats.route, Some(route));
    let prior_pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_solid(&prior_pixels, [255, 0, 0, alpha]);

    // A point-only fill has no ink and therefore no required native placement.
    // 2^-127 is finite/normal f64 but subnormal f32: a surviving drawable would
    // require a capacity refusal, while this absent ink must skip it.
    let target = scene(graph, |scene| {
        scene.fill(bounds(), blue);
        scene.prepared_glyph_run(
            prepared,
            Transform::scale(2.0_f64.powi(-127), 1.0).unwrap(),
            TextPaint::try_fill(red.into()).unwrap(),
        );
    });
    let result = pollster::block_on(renderer.render(&mut surface, &target, Parameters::default()));
    if let Err(error) = &result {
        // A refusal must preserve the original publication; unchanged prior
        // pixels do not count as the required successful blue frame.
        assert_eq!(renderer.stats(), prior_stats);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            prior_pixels
        );
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Failed(error.code()));
        assert_eq!(report.cleanup_fault(), None);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::Submissions), 0);
    }
    let stats = result.expect("a point-only prepared glyph must skip unused native placement");
    assert_eq!(stats.route, Some(route));
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.cleanup_fault(), None);
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_solid(&pixels, [0, 0, 255, alpha]);
    assert_ne!(pixels, prior_pixels);
}

#[test]
fn direct_point_only_prepared_glyph_skips_unused_native_transform_with_accounting_off() {
    check_point_only_placement(false, RenderProfile::Off);
}

#[test]
fn direct_point_only_prepared_glyph_skips_unused_native_transform_with_summary() {
    check_point_only_placement(false, RenderProfile::Summary { timing: false });
}

#[test]
fn graph_point_only_prepared_glyph_skips_unused_native_transform_with_accounting_off() {
    check_point_only_placement(true, RenderProfile::Off);
}

#[test]
fn graph_point_only_prepared_glyph_skips_unused_native_transform_with_summary() {
    check_point_only_placement(true, RenderProfile::Summary { timing: false });
}
