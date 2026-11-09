#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use std::{cell::Cell, time::Duration};
use surgeist_render::{
    Antialiasing, Color, EffectQualityPolicy, FontData, FontRef, GlyphRunLimits, GlyphSynthesis,
    Gradient, GradientStop, Image, ImageBuffer, ImageFit, Layer, Options, Parameters, Point,
    PreparedGlyphRun, Rect, RenderAccountingContext, RenderAccountingError,
    RenderAccountingOptions, RenderAttemptStatus, RenderClock, RenderCpuScope, RenderEvent,
    RenderFrameReport, RenderLimits, RenderPhase, RenderPhaseFilter, RenderProfile, RenderQuantity,
    RenderResource, RenderRoute, RenderWork, Renderer, RepeatingGradient, ResolvedLayerAlphaMask,
    Scene, Shape, Size, Stroke, Surface, TextGlyph, TextInstance, TextPaint, TextRun,
    TextRunBounds, Transform,
};

fn color(r: f32, g: f32, b: f32) -> Color {
    Color::try_rgba(r, g, b, 1.0).unwrap()
}
fn red() -> Color {
    color(1.0, 0.0, 0.0)
}
fn green() -> Color {
    color(0.0, 1.0, 0.0)
}
fn blue() -> Color {
    color(0.0, 0.0, 1.0)
}
fn yellow() -> Color {
    color(1.0, 1.0, 0.0)
}
fn magenta() -> Color {
    color(1.0, 0.0, 1.0)
}
fn cyan() -> Color {
    color(0.0, 1.0, 1.0)
}
fn point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}
fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect::try_new(x, y, w, h).unwrap()
}
fn stop(offset: f32, color: Color) -> GradientStop {
    GradientStop::try_new(f64::from(offset), color).unwrap()
}

fn step(start: f64, end: f64, first: Color, last: Color) -> Gradient {
    Gradient::try_linear(
        point(start, 0.0),
        point(end, 0.0),
        vec![stop(0.5, first), stop(0.5, magenta()), stop(0.5, last)],
    )
    .unwrap()
}

fn terminal(first: Color, middle: Color, last: Color) -> Gradient {
    Gradient::try_linear(
        point(0.0, 0.0),
        point(4.0, 0.0),
        vec![stop(0.0, first), stop(1.0, middle), stop(1.0, last)],
    )
    .unwrap()
}

fn options() -> Options {
    Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
}

fn surface(renderer: &mut Renderer, width: f64, height: f64, scale: f64) -> Surface {
    pollster::block_on(renderer.create_headless(Size::try_new(width, height).unwrap(), scale))
        .unwrap()
}

fn wrapped(width: f64, height: f64, graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new().with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(mask, rect(0.0, 0.0, width, height)).unwrap(),
            ),
            draw,
        );
    } else {
        draw(&mut scene);
    }
    scene
}

fn read(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene, graph: bool) -> ImageBuffer {
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

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
    let actual = pixel(image, x, y);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "pixel ({x},{y}) actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

fn image(bytes: [u8; 4]) -> Image {
    Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), bytes.to_vec()).unwrap()
}

fn native_run<'a>(
    font: &FontData,
    transform: Transform,
    glyphs: &'a [TextGlyph],
    paint: TextPaint,
) -> TextRun<'a> {
    TextRun::try_new(
        FontRef::new(2089).with_data(font.clone()),
        1.0,
        transform,
        paint,
        glyphs,
        TextRunBounds::try_ink(rect(0.0, 0.0, 1.0, 1.0)).unwrap(),
    )
    .unwrap()
}

fn text_frame(
    font: &FontData,
    prepared: &PreparedGlyphRun,
    empty: &PreparedGlyphRun,
    graph: bool,
) -> Scene {
    let glyphs = [TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap()];
    wrapped(8.0, 4.0, graph, |scene| {
        scene.transform(Transform::scale(0.5, 0.5).unwrap(), |scene| {
            scene.image(
                image([255, 255, 255, 255]),
                rect(0.0, 6.0, 2.0, 2.0),
                ImageFit::Stretch,
            );
            scene.text_run(native_run(
                font,
                Transform::scale(4.0, 4.0).unwrap(),
                &glyphs,
                TextPaint::try_fill(step(0.0, 1.0, red(), blue()).into()).unwrap(),
            ));
            scene.image(
                image([0, 255, 0, 255]),
                rect(6.0, 6.0, 2.0, 2.0),
                ImageFit::Stretch,
            );
            scene.prepared_glyph_run(
                empty.clone(),
                Transform::identity(),
                TextPaint::try_fill(step(0.0, 1.0, blue(), red()).into()).unwrap(),
            );
            scene.text_run(native_run(
                font,
                Transform::try_new([4.0, 0.0, 0.0, 4.0, 6.0, 0.0]).unwrap(),
                &glyphs,
                TextPaint::try_fill(step(0.0, 1.0, green(), yellow()).into()).unwrap(),
            ));
            scene.image(
                image([255, 0, 255, 0]),
                rect(2.0, 6.0, 2.0, 2.0),
                ImageFit::Stretch,
            );
            scene.prepared_glyph_run(
                prepared.clone(),
                Transform::try_new([4.0, 0.0, 0.0, 4.0, 12.0, 0.0]).unwrap(),
                TextPaint::try_fill(step(0.0, 1.0, magenta(), cyan()).into()).unwrap(),
            );
            scene.text_run(native_run(
                font,
                Transform::identity(),
                &[],
                TextPaint::try_fill(step(0.0, 1.0, green(), red()).into()).unwrap(),
            ));
            scene.image(
                image([0, 0, 255, 255]),
                rect(12.0, 6.0, 2.0, 2.0),
                ImageFit::Stretch,
            );
        });
    })
}

fn shape_frame(graph: bool) -> Scene {
    let repeating = RepeatingGradient::try_linear(
        point(0.0, 0.0),
        point(8.0, 0.0),
        vec![
            GradientStop::try_new(0.0, red()).unwrap(),
            GradientStop::try_new(1.0, blue()).unwrap(),
            GradientStop::try_new(1.0, green()).unwrap(),
        ],
    )
    .unwrap();
    wrapped(8.0, 4.0, graph, |scene| {
        scene.transform(Transform::scale(0.5, 0.5).unwrap(), |scene| {
            // Pad and Repeat receive identical native stop tuples. Their
            // geometries differ, but sharing a ramp must not share Pad policy.
            scene.fill(rect(0.0, 0.0, 8.0, 4.0), terminal(red(), blue(), green()));
            scene.fill(rect(8.0, 0.0, 8.0, 4.0), repeating);
            scene.stroke(
                Shape::rect(rect(0.0, 5.0, 16.0, 2.0)),
                Stroke::try_new(2.0).unwrap(),
                terminal(blue(), green(), red()),
            );
            let solid =
                Gradient::try_linear(point(0.0, 0.0), point(0.0, 0.0), vec![stop(0.75, yellow())])
                    .unwrap();
            scene.fill(rect(14.0, 6.0, 2.0, 2.0), solid);
        });
    })
}

#[test]
fn endpoint_association_survives_interleaved_glyphs_images_shapes_repeat_and_final_scale() {
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let instance = TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyphs = [TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap()];
    let limits = GlyphRunLimits::new(2, 16, 1 << 20, 1 << 20);
    let prepared = font
        .prepare_glyph_run(
            &instance,
            &glyphs,
            limits,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let empty = font
        .prepare_glyph_run(
            &instance,
            &[],
            limits,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    assert_eq!(empty.bounds().ink_rect(), None);
    let mut renderer = pollster::block_on(Renderer::new(options())).unwrap();
    for scale in [1_u32, 2] {
        let mut surface = surface(&mut renderer, 8.0, 4.0, f64::from(scale));
        for graph in [false, true] {
            let alpha = if graph { 128 } else { 255 };
            let text = read(
                &mut renderer,
                &mut surface,
                &text_frame(&font, &prepared, &empty, graph),
                graph,
            );
            // Ahem X is a one-em square. Run scale 4 followed by parent scale .5
            // makes two logical pixels. Left samples are before .5; right
            // samples equal .5 and select each run's final supplied endpoint.
            for (x, rgb) in [
                (0, [255, 0, 0]),
                (1, [0, 0, 255]),
                (3, [0, 255, 0]),
                (4, [255, 255, 0]),
                (6, [255, 0, 255]),
                (7, [0, 255, 255]),
            ] {
                assert_pixel(&text, x * scale, scale, [rgb[0], rgb[1], rgb[2], alpha], 0);
            }
            for (x, rgb) in [(0, [255, 255, 255]), (3, [0, 255, 0]), (6, [0, 0, 255])] {
                assert_pixel(
                    &text,
                    x * scale,
                    3 * scale,
                    [rgb[0], rgb[1], rgb[2], alpha],
                    0,
                );
            }
            // Straight-alpha RGB is not canonical at zero opacity. The
            // transparent image must contribute no visible coverage.
            assert_eq!(pixel(&text, scale, 3 * scale)[3], 0);
            let shapes = read(&mut renderer, &mut surface, &shape_frame(graph), graph);
            assert_pixel(&shapes, 3 * scale, scale, [0, 255, 0, alpha], 0);
            assert_pixel(&shapes, 3 * scale, 2 * scale, [255, 0, 0, alpha], 0);
            assert_pixel(&shapes, 7 * scale, 3 * scale, [255, 255, 0, alpha], 0);
            assert_pixel(&shapes, 4 * scale, scale, [255, 0, 0, alpha], 0);
            // Repeat's period is four logical pixels: at x 5 and 6 the phases
            // are .25 and .5. Only these interior ramp values use byte bounds.
            let tolerance = if graph { 5 } else { 2 };
            assert_pixel(&shapes, 5 * scale, scale, [191, 0, 64, alpha], tolerance);
            assert_pixel(&shapes, 6 * scale, scale, [128, 0, 128, alpha], tolerance);
        }
    }
}

#[test]
fn one_hundred_twenty_eight_distinct_draws_keep_endpoint_identity_across_all_fine_modes() {
    for antialiasing in [
        Antialiasing::Area,
        Antialiasing::Msaa8,
        Antialiasing::Msaa16,
    ] {
        let mut renderer =
            pollster::block_on(Renderer::new(options().with_antialiasing(antialiasing))).unwrap();
        let mut surface = surface(&mut renderer, 128.0, 1.0, 1.0);
        for last_side in [false, true] {
            for graph in [false, true] {
                let scene = wrapped(128.0, 1.0, graph, |scene| {
                    for x in 0_u32..128 {
                        let first = if x % 2 == 0 { red() } else { green() };
                        let last = if x % 2 == 0 { blue() } else { yellow() };
                        // At pixel x, raw phase is .25 or .75, respectively.
                        // Distinct translated fields require actual separate
                        // gradient draws even when stop/ramp colors repeat.
                        let start = f64::from(x) - if last_side { 3.0 } else { 1.0 };
                        scene.fill(
                            rect(f64::from(x), 0.0, 1.0, 1.0),
                            step(start, start + 4.0, first, last),
                        );
                    }
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                let alpha = if graph { 128 } else { 255 };
                for x in 0_u32..128 {
                    let rgb = match (last_side, x % 2 == 0) {
                        (false, true) => [255, 0, 0],
                        (false, false) => [0, 255, 0],
                        (true, true) => [0, 0, 255],
                        (true, false) => [255, 255, 0],
                    };
                    assert_pixel(&image, x, 0, [rgb[0], rgb[1], rgb[2], alpha], 0);
                }
            }
        }
    }
}

fn descriptor_scene() -> Scene {
    let mut scene = Scene::new();
    scene.fill(rect(0.0, 0.0, 4.0, 4.0), step(0.0, 4.0, red(), blue()));
    scene.fill(rect(4.0, 0.0, 4.0, 4.0), step(4.0, 8.0, green(), red()));
    scene
}

thread_local! { static CLOCK_READS: Cell<u64> = const { Cell::new(0) }; }
fn clock() -> Duration {
    CLOCK_READS.with(|reads| {
        let now = reads.get();
        reads.set(now + 1);
        Duration::from_nanos(now)
    })
}
fn forbidden_clock() -> Duration {
    panic!("Off and untimed frame accounting must not read a clock")
}

#[test]
fn descriptor_profiles_preserve_pixels_usage_and_a_refused_frame_can_retry_without_publication_loss()
 {
    let target = descriptor_scene();
    let mut solid = Scene::new();
    solid.fill(rect(0.0, 0.0, 8.0, 4.0), green());
    let mut baseline: Option<RenderFrameReport> = None;
    let mut baseline_pixels = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 16,
            filter: RenderPhaseFilter::Phase(RenderPhase::GradientPrepare),
        },
    ] {
        CLOCK_READS.with(|reads| reads.set(0));
        let timing = matches!(
            profile,
            RenderProfile::Summary { timing: true } | RenderProfile::Detailed { timing: true, .. }
        );
        let selected_clock = if timing { clock } else { forbidden_clock };
        let accounting = RenderAccountingOptions::new()
            .with_profile(profile)
            .with_clock(RenderClock::Host(selected_clock));
        let mut renderer =
            pollster::block_on(Renderer::new(options().with_accounting(accounting))).unwrap();
        let mut surface = surface(&mut renderer, 8.0, 4.0, 1.0);
        // Identical warm-up removes first-frame device/pipeline differences
        // from the compared descriptor operation; observer storage is excluded.
        pollster::block_on(renderer.render(&mut surface, &solid, Parameters::default())).unwrap();
        pollster::block_on(renderer.render(&mut surface, &target, Parameters::default())).unwrap();
        let frame = renderer.take_last_attempt().unwrap();
        assert_eq!(frame.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(frame.cleanup_fault(), None);
        let attempt = frame.attempt().unwrap();
        assert_eq!(attempt.elapsed().is_some(), timing);
        assert_eq!(
            attempt
                .phase_elapsed(RenderPhase::GradientPrepare)
                .is_some(),
            timing
        );
        if profile != RenderProfile::Off {
            assert!(attempt.phase_count(RenderPhase::GradientPrepare) > 0);
        }
        if let Some(previous_frame) = &baseline {
            let expected = previous_frame.attempt().unwrap();
            assert_eq!(attempt.usage(), expected.usage());
            for phase in [
                RenderPhase::Normalize,
                RenderPhase::Encode,
                RenderPhase::GradientPrepare,
            ] {
                assert_eq!(attempt.phase_usage(phase), expected.phase_usage(phase));
            }
            let cpu = frame.renderer_cpu().unwrap();
            let previous = previous_frame.renderer_cpu().unwrap();
            for scope in [
                RenderCpuScope::FrameBacking,
                RenderCpuScope::DeviceBacking,
                RenderCpuScope::ImageHistory,
                RenderCpuScope::PreparedValues,
                RenderCpuScope::FontPayloads,
                RenderCpuScope::ImagePayloads,
                RenderCpuScope::ReadbackOutput,
            ] {
                assert_eq!(cpu.scope_usage(scope), previous.scope_usage(scope));
            }
            // Cleanup records actual lease-sort comparisons, whose count can
            // vary with fresh resource maps even when profiling is Off. Keep
            // those observations intact; backing custody remains invariant.
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                let actual = cpu.usage();
                let expected = previous.usage();
                assert_eq!(
                    actual.opening_live(resource),
                    expected.opening_live(resource)
                );
                assert_eq!(actual.live(resource), expected.live(resource));
                assert_eq!(actual.peak(resource), expected.peak(resource));
            }
        }
        if let RenderProfile::Detailed {
            event_limit,
            filter,
            ..
        } = profile
        {
            assert!(attempt.events().count() <= event_limit);
            if event_limit == 0 {
                assert!(attempt.dropped_events() > 0);
            }
            if let RenderPhaseFilter::Phase(expected) = filter {
                assert!(attempt.events().next().is_some());
                for event in attempt.events() {
                    let phase = match event {
                        RenderEvent::Change { phase, .. }
                        | RenderEvent::Phase { phase, .. }
                        | RenderEvent::Refused { phase, .. } => phase,
                    };
                    assert_eq!(phase, expected);
                }
            }
        }
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (x, rgb) in [
            (1, [255, 0, 0]),
            (3, [0, 0, 255]),
            (5, [0, 255, 0]),
            (7, [255, 0, 0]),
        ] {
            assert_pixel(&image, x, 2, [rgb[0], rgb[1], rgb[2], 255], 0);
        }
        if let Some(expected) = &baseline_pixels {
            assert_eq!(&image, expected);
        } else {
            baseline_pixels = Some(image);
        }
        if baseline.is_none() {
            baseline = Some(frame);
        }
    }
    let total = baseline
        .unwrap()
        .attempt()
        .unwrap()
        .usage()
        .work(RenderWork::GradientStopVisits);
    assert!(total > 1);
    let cap = total - 1;
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::GradientStopVisits, cap)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        options().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface = surface(&mut renderer, 8.0, 4.0, 1.0);
    let published_stats =
        pollster::block_on(renderer.render(&mut surface, &solid, Parameters::default())).unwrap();
    let published = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let error = pollster::block_on(renderer.render(&mut surface, &target, Parameters::default()))
        .unwrap_err();
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("expected a named stop-work refusal")
    };
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::GradientStopVisits)
    );
    assert_eq!(renderer.stats(), published_stats);
    let refused = renderer.take_last_attempt().unwrap();
    assert_eq!(refused.status(), RenderAttemptStatus::Refused);
    assert_eq!(refused.cleanup_fault(), None);
    let usage = refused.attempt().unwrap().usage();
    assert!(
        usage.work(RenderWork::GradientStopVisits) > 0
            && usage.work(RenderWork::GradientStopVisits) <= cap
    );
    assert_eq!(usage.work(RenderWork::Submissions), 0);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        published
    );
    let mut retry = Scene::new();
    retry.fill(rect(0.0, 0.0, 8.0, 4.0), step(0.0, 4.0, green(), yellow()));
    pollster::block_on(renderer.render(&mut surface, &retry, Parameters::default())).unwrap();
    assert_eq!(
        renderer.last_attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_ne!(image, published);
    assert_pixel(&image, 1, 2, [0, 255, 0, 255], 0);
    assert_pixel(&image, 3, 2, [255, 255, 0, 255], 0);
}
