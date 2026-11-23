#![cfg(all(feature = "render-paint", not(target_arch = "wasm32")))]
#![forbid(unsafe_code)]

use surgeist_layout as layout;
use surgeist_ledger::ProfileMode;
use surgeist_paint::{
    accounting::{PaintLimits, PaintOptions},
    gradient_stops::{GradientPosition, GradientStopItem, GradientStopPositions},
    radial_gradient::{RadialGradientGeometry, RadialGradientSize, resolve_radial_gradient},
    zero_circle_gradient::resolve_zero_circle_gradient_stops,
};
use surgeist_render::{
    accounting::RenderAccountingContext,
    geometry::{Rect, Size},
    image::{Image, ImageBuffer},
    paint::{Color, Paint},
    paint_gradient::{prepare_repeating_zero_circle_gradient, prepare_zero_circle_gradient},
    renderer::{EffectQualityPolicy, Options, RenderRoute, Renderer},
    scene::{Layer, ResolvedLayerAlphaMask, Scene},
    surface::{Parameters, Surface},
};

const WIDTH: u32 = 32;
const HEIGHT: u32 = 24;
const CENTER_X: u32 = 12;
const CENTER_Y: u32 = 10;

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT)).unwrap()
}

fn options() -> PaintOptions {
    PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)
}

fn stop(rgb: [f32; 3], length: f64, fraction: f64) -> GradientStopItem<Color> {
    let [r, g, b] = rgb;
    GradientStopItem::Stop {
        payload: Color::try_rgba(r, g, b, 1.0).unwrap(),
        positions: GradientStopPositions::One(GradientPosition { length, fraction }),
    }
}

fn hint(length: f64, fraction: f64) -> GradientStopItem<Color> {
    GradientStopItem::Hint {
        position: GradientPosition { length, fraction },
    }
}

fn geometry() -> RadialGradientGeometry {
    geometry_at(layout::geometry::Point::new(
        f64::from(CENTER_X),
        f64::from(CENTER_Y),
    ))
}

fn geometry_at(center: layout::geometry::Point<f64>) -> RadialGradientGeometry {
    resolve_radial_gradient(
        layout::geometry::Size::new(f64::from(WIDTH), f64::from(HEIGHT)),
        Some(center),
        RadialGradientSize::CircleRadius(0.0),
        options(),
    )
    .result
    .unwrap()
}

fn field(source: &[GradientStopItem<Color>], repeat: bool) -> Paint {
    paint_for(&geometry(), source, repeat)
}

fn paint_for(
    geometry: &RadialGradientGeometry,
    source: &[GradientStopItem<Color>],
    repeat: bool,
) -> Paint {
    let stops = resolve_zero_circle_gradient_stops(geometry, source, options())
        .result
        .unwrap();
    let mut context = RenderAccountingContext::default();
    if repeat {
        prepare_repeating_zero_circle_gradient(&stops, &mut context).unwrap()
    } else {
        prepare_zero_circle_gradient(&stops, &mut context).unwrap()
    }
}

fn target() -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface = pollster::block_on(renderer.create_headless(
        Size::try_new(f64::from(WIDTH), f64::from(HEIGHT)).unwrap(),
        1.0,
    ))
    .unwrap();
    (renderer, surface)
}

fn read(renderer: &mut Renderer, surface: &mut Surface, paint: Paint, graph: bool) -> ImageBuffer {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new()
                .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, bounds()).unwrap()),
            |layer| {
                layer.fill(bounds(), paint);
            },
        );
    } else {
        scene.fill(bounds(), paint);
    }
    let stats =
        pollster::block_on(renderer.render(surface, &scene, Parameters::default())).unwrap();
    let route = if graph {
        RenderRoute::GpuGraph
    } else {
        RenderRoute::DirectVello
    };
    assert_eq!(stats.route, Some(route));
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn pixel(image: &ImageBuffer, x: u32, y: u32, [r, g, b]: [u8; 3], graph: bool) {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual = &image.rgba()[index..index + 4];
    let expected = [r, g, b, if graph { 128 } else { 255 }];
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= 3),
        "pixel ({x}, {y}), graph={graph}: actual {actual:?}, expected {expected:?}"
    );
}

const RED: [f32; 3] = [1.0, 0.0, 0.0];
const GREEN: [f32; 3] = [0.0, 1.0, 0.0];
const BLUE: [f32; 3] = [0.0, 0.0, 1.0];
const WHITE: [f32; 3] = [1.0; 3];

#[test]
fn paint_adapter_accounts_work_and_releases_storage_on_output_refusal() {
    use surgeist_render::accounting::{
        RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus, RenderLimits,
        RenderProfile, RenderResource, RenderWork,
    };

    let source = [stop(RED, 4.0, -2.0), hint(4.0, -1.0), stop(BLUE, 9.0, 0.0)];
    let used = resolve_zero_circle_gradient_stops(&geometry(), &source, options())
        .result
        .unwrap();
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let mut context = RenderAccountingContext::new(options);
    let expected = prepare_zero_circle_gradient(&used, &mut context).unwrap();
    let record = context.take_last_attempt().unwrap();
    let report = record.attempt().unwrap();
    assert_eq!(report.kind(), RenderAttemptKind::PaintGradient);
    assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 8);
    assert_eq!(report.usage().work(RenderWork::GradientStopVisits), 7);
    assert_eq!(report.usage().work(RenderWork::PaintVisits), 2);
    let held = report.usage().peak(RenderResource::RetainedCpuBytes);
    assert_eq!(
        report.usage().work(RenderWork::PreparedTransferredBytes),
        held
    );
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);

    context.set_options(options.with_limits(RenderLimits::new().with_retained_bytes(held - 1)));
    assert!(
        prepare_zero_circle_gradient(&used, &mut context)
            .unwrap_err()
            .accounting_diagnostic()
            .is_some()
    );
    let report = context.last_attempt().unwrap().attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.usage().work(RenderWork::PreparedTransferredBytes), 0);
    assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);

    context.set_options(options);
    assert_eq!(
        prepare_zero_circle_gradient(&used, &mut context).unwrap(),
        expected
    );
}

#[test]
fn circle_program_copy_refusal_keeps_the_last_published_pixels_and_stats() {
    use surgeist_render::accounting::{
        RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus, RenderLimits,
        RenderQuantity, RenderWork,
    };

    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::RasterResourceCopies, 1)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface = pollster::block_on(renderer.create_headless(
        Size::try_new(f64::from(WIDTH), f64::from(HEIGHT)).unwrap(),
        1.0,
    ))
    .unwrap();
    let solid = Paint::color(Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
    let before = read(&mut renderer, &mut surface, solid.clone(), false);
    let successful = renderer.stats();
    let mut scene = Scene::new();
    scene.fill(
        bounds(),
        field(&[stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)], false),
    );
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert!(
        matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(refusal))
        if refusal.quantity() == RenderQuantity::Work(RenderWork::RasterResourceCopies))
    );
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.cleanup_fault(), None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::RasterResourceCopies), 1);
    assert_eq!(usage.work(RenderWork::Submissions), 0);
    assert_eq!(renderer.stats(), successful);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert_eq!(read(&mut renderer, &mut surface, solid, false), before);
}

#[test]
fn zero_circle_preserves_percentage_blend_at_the_native_center_and_ring() {
    let center = field(&[stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)], false);
    let ring = field(&[stop(RED, 4.0, -1.0), stop(BLUE, 4.0, 1.0)], false);
    let (mut renderer, mut surface) = target();
    for graph in [false, true] {
        let image = read(&mut renderer, &mut surface, center.clone(), graph);
        pixel(&image, CENTER_X, CENTER_Y, [128, 0, 128], graph);
        pixel(&image, CENTER_X + 1, CENTER_Y, [0, 0, 255], graph);
        let image = read(&mut renderer, &mut surface, ring.clone(), graph);
        for (radius, rgb) in [(3, [255, 0, 0]), (4, [128, 0, 128]), (5, [0, 0, 255])] {
            pixel(&image, CENTER_X + radius, CENTER_Y, rgb, graph);
            pixel(&image, CENTER_X, CENTER_Y - radius, rgb, graph);
        }
    }
}

#[test]
fn zero_circle_preserves_the_nonzero_radius_when_its_unscaled_square_is_subnormal() {
    let paint = paint_for(
        &geometry_at(layout::geometry::Point::new(2.0_f64.powi(-64), 0.0)),
        &[stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)],
        false,
    );
    let (mut renderer, mut surface) = target();
    let image = read(&mut renderer, &mut surface, paint, false);
    // At device (0,0) the native local radius is 2^-64, so the sample is
    // outside the center blend even though its square is smaller than 2^-126.
    pixel(&image, 0, 0, [0, 0, 255], false);
}

#[test]
fn zero_circle_refuses_a_native_carrier_that_could_invent_center_coincidence() {
    let subnormal = 3.0 * 2.0_f64.powi(-128);
    let paint = paint_for(
        &geometry_at(layout::geometry::Point::new(subnormal, subnormal)),
        &[stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)],
        false,
    );
    let (mut renderer, mut surface) = target();
    let mut scene = Scene::new();
    scene.fill(bounds(), paint);
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert!(
        error.geometry_realization_diagnostic().is_some()
            || error.gradient_realization_diagnostic().is_some(),
        "{error:?}"
    );
}

#[test]
fn zero_circle_does_not_invent_a_center_sample_after_surface_scaling() {
    let subnormal = 3.0 * 2.0_f64.powi(-128);
    let paint = paint_for(
        &geometry_at(layout::geometry::Point::new(subnormal, subnormal)),
        &[stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)],
        false,
    );
    let (mut renderer, _) = target();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(2.0, 2.0).unwrap(), 16.0))
            .unwrap();
    let mut scene = Scene::new();
    scene.fill(Rect::try_new(0.0, 0.0, 2.0, 2.0).unwrap(), paint);
    match pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())) {
        Ok(_) => {
            let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            pixel(&image, 0, 0, [0, 0, 255], false);
        }
        Err(error) => assert!(
            error.gradient_realization_diagnostic().is_some(),
            "{error:?}"
        ),
    }
}

#[test]
fn zero_circle_hint_limits_preserve_rings_at_both_sides_of_a_nonzero_span() {
    let left = field(
        &[stop(RED, 4.0, -2.0), hint(4.0, -1.0), stop(BLUE, 9.0, 0.0)],
        false,
    );
    let right = field(
        &[stop(RED, 0.0, 0.0), hint(8.0, -2.0), stop(BLUE, 8.0, 2.0)],
        false,
    );
    let (mut renderer, mut surface) = target();
    // Left ring samples have the hint's halfway color. At the right ring the
    // remaining fraction divides to 2/4, producing a blue weight of 2^-0.5.
    for (paint, samples) in [
        (
            left,
            [(3, [255, 0, 0]), (4, [128, 0, 128]), (5, [0, 0, 255])],
        ),
        (
            right,
            [(7, [255, 0, 0]), (8, [75, 0, 180]), (9, [0, 0, 255])],
        ),
    ] {
        let image = read(&mut renderer, &mut surface, paint, false);
        for (radius, rgb) in samples {
            pixel(&image, CENTER_X + radius, CENTER_Y, rgb, false);
        }
    }
}

#[test]
fn repeating_zero_circle_preserves_seam_direction_and_fraction_per_cycle() {
    let positive = field(&[stop(RED, 0.0, 0.0), stop(BLUE, 8.0, 1.0)], true);
    let negative = field(&[stop(RED, 0.0, 0.0), stop(BLUE, 8.0, -1.0)], true);
    let ring = field(
        &[
            stop(RED, 0.0, 0.0),
            stop(GREEN, 2.0, -1.0),
            stop(BLUE, 2.0, 2.0),
            stop(WHITE, 8.0, 1.0),
        ],
        true,
    );
    let (mut renderer, mut surface) = target();
    for (paint, seam) in [(positive, [0, 0, 255]), (negative, [255, 0, 0])] {
        let image = read(&mut renderer, &mut surface, paint, false);
        pixel(&image, CENTER_X + 8, CENTER_Y, seam, false);
    }
    let image = read(&mut renderer, &mut surface, ring, false);
    // At radius 2 the interpolated fraction is (0-(-1))/3. At radius 10,
    // one period shifts the sampled fraction to -1, exactly the green stop.
    pixel(&image, CENTER_X + 2, CENTER_Y, [0, 170, 85], false);
    pixel(&image, CENTER_X + 10, CENTER_Y, [0, 255, 0], false);
}

#[test]
fn repeating_zero_circle_averages_collapsed_periods_and_the_negative_threshold_tie() {
    let fractional = field(
        &[
            stop(RED, 4.0, 0.0),
            stop(GREEN, 4.0, 0.25),
            stop(BLUE, 4.0, 1.0),
        ],
        true,
    );
    let coincident = field(
        &[
            stop(RED, 4.0, 0.0),
            stop(GREEN, 4.0, 0.0),
            stop(BLUE, 4.0, 0.0),
        ],
        true,
    );
    let below = field(&[stop(RED, 0.0, 0.0), stop(BLUE, 2.0, -1.0)], true);
    let rounded_below = field(
        &[stop(RED, 2.0_f64.powi(-54), 0.0), stop(BLUE, 2.0, 1.0)],
        true,
    );
    let above = field(&[stop(RED, 0.0, 0.0), stop(BLUE, 2.0, 1.0)], true);
    let (mut renderer, mut surface) = target();
    for (paint, rgb) in [
        (fractional, [32, 128, 96]),
        (coincident, [64, 128, 64]),
        (below, [128, 0, 128]),
        (rounded_below, [128, 0, 128]),
        (above, [0, 0, 255]),
    ] {
        let image = read(&mut renderer, &mut surface, paint, false);
        pixel(&image, CENTER_X + 2, CENTER_Y, rgb, false);
    }
}

#[test]
fn repeating_zero_circle_uses_the_thin_axis_of_an_almost_circular_transform() {
    use surgeist_render::geometry::Transform;

    let paint = field(&[stop(RED, 0.0, 0.0), stop(BLUE, 2.0, 0.0)], true);
    let (mut renderer, mut surface) = target();
    let mut scene = Scene::new();
    scene.transform(
        Transform::try_new([1.0, 0.0, 2.0_f64.powi(-27), 1.0, 0.0, 0.0]).unwrap(),
        |scene| {
            scene.fill(bounds(), paint);
        },
    );
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    // Determinant is one, but the larger singular value exceeds one for
    // every nonzero shear. The leading period is therefore below two pixels.
    pixel(&image, CENTER_X, CENTER_Y, [128, 0, 128], false);
}

#[test]
fn repeating_zero_circle_preserves_the_side_of_a_seam_after_large_phase_cancellation() {
    let paint = paint_for(
        &geometry_at(layout::geometry::Point::new(
            2.0_f64.powi(-14),
            f64::from(CENTER_Y),
        )),
        &[stop(RED, -4092.0, 0.0), stop(BLUE, -4088.0, 0.0)],
        true,
    );
    let (mut renderer, mut surface) = target();
    let image = read(&mut renderer, &mut surface, paint, false);
    // The native radius is 4 - 2^-14. Relative to the first stop, the
    // distance is 4096 - 2^-14. Rounding that subtraction to f32 would move
    // the sample to the red start of cycle 1024 instead of blue in cycle 1023.
    pixel(&image, 4, CENTER_Y, [0, 0, 255], false);
}

#[test]
fn repeating_zero_circle_keeps_exact_ring_and_fraction_sides_under_cancellation() {
    let center = layout::geometry::Point::new(2.0_f64.powi(-14), f64::from(CENTER_Y));
    let ring = paint_for(
        &geometry_at(center),
        &[
            stop(RED, -4092.0, 0.0),
            stop(GREEN, -4088.0, -1.0),
            stop(BLUE, -4088.0, 1.0),
            stop(WHITE, -4084.0, 0.0),
        ],
        true,
    );
    let fraction_span = 1.0 - 2.0_f64.powi(-24);
    let first_fraction = -1023.0 + 2.0_f64.powi(-14);
    let seam = field(
        &[
            stop(RED, -4092.0, first_fraction),
            stop(BLUE, -4088.0, first_fraction + fraction_span),
        ],
        true,
    );
    let (mut renderer, mut surface) = target();
    let image = read(&mut renderer, &mut surface, ring, false);
    // The radius is 2^-14, just past the translated green-to-blue ring
    // at radius zero. A rounded delta would place the sample on the ring.
    pixel(&image, 0, CENTER_Y, [0, 0, 255], false);
    let image = read(&mut renderer, &mut surface, seam, false);
    // At the leading seam the relative fraction is -2^-24. It belongs to
    // the prior cycle's blue end. Multiplying k=1023 and F in f32 before
    // subtracting the origin rounds that secondary coordinate to zero.
    pixel(&image, CENTER_X, CENTER_Y, [0, 0, 255], false);
}

#[test]
fn zero_circle_center_survives_stroke_and_both_glyph_routes() {
    use surgeist_render::{
        geometry::Transform,
        shape::{Shape, Stroke},
        text::{
            FontData, FontRef, GlyphRunLimits, GlyphSynthesis, TextGlyph, TextInstance, TextPaint,
            TextRun, TextRunBounds,
        },
    };

    let source = [stop(RED, 0.0, -1.0), stop(BLUE, 0.0, 1.0)];
    let stroke = field(&source, false);
    let glyph = paint_for(
        &geometry_at(layout::geometry::Point::new(0.5, 0.5)),
        &source,
        false,
    );
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let glyphs = [TextGlyph::try_new(58, 0.0, 0.8, 1.0).unwrap()];
    let prepared = font
        .prepare_glyph_run(
            &TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap(),
            &glyphs,
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let transform = Transform::try_new([8.0, 0.0, 0.0, 8.0, 4.0, 4.0]).unwrap();
    let mut scene = Scene::new();
    scene.stroke(
        Shape::rect(Rect::try_new(2.0, 10.0, 28.0, 12.0).unwrap()),
        Stroke::try_new(4.0).unwrap(),
        stroke,
    );
    scene.prepared_glyph_run(
        prepared,
        transform,
        TextPaint::try_fill(glyph.clone()).unwrap(),
    );
    scene.text_run(
        TextRun::try_new(
            FontRef::new(332).with_data(font),
            1.0,
            Transform::try_new([8.0, 0.0, 0.0, 8.0, 4.0, 12.0]).unwrap(),
            TextPaint::try_fill(glyph).unwrap(),
            &glyphs,
            TextRunBounds::try_ink(Rect::try_new(0.0, 0.0, 1.0, 1.0).unwrap()).unwrap(),
        )
        .unwrap(),
    );
    let (mut renderer, mut surface) = target();
    pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default())).unwrap();
    let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for (x, y) in [(CENTER_X, CENTER_Y), (8, 8), (8, 16)] {
        pixel(&image, x, y, [128, 0, 128], false);
        pixel(&image, x + 1, y, [0, 0, 255], false);
    }
}
