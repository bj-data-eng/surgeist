//! Error text of the contextual geometry and gradient refusals, observed by rendering on the real GPU.
//!
//! The scenes are the ones `native_geometry.rs`, `ordinary_fields.rs` and `horizontal_distance.rs`
//! use to reach each refusal; the expected sentences are written from the meaning of the variant
//! (what native conversion the receiving operation could not perform) and do not come from the
//! message implementation. `Error` prints its message only.
//!
//! Not covered here: `GradientRealizationError::{DegenerateLinearField, UnrepresentableGeometry,
//! InsufficientNativeCapacity}`, which need a transformed or repeated field whose arithmetic fails
//! at a value that no short public scene produces; the tables in `error_display.rs` cover their text.
#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Error, ErrorCode, accounting::RenderAccountingContext, diagnostic::GeometryRealizationError,
    geometry::Point, geometry::Rect, geometry::Size, geometry::Transform, paint::Color,
    paint::Gradient, paint::GradientRealizationError, renderer::EffectQualityPolicy,
    renderer::Options, renderer::Renderer, scene::Scene, scene::Shadow, shape::Stroke,
    surface::Parameters, text::FontData, text::FontRef, text::GlyphRunLimits, text::GlyphSynthesis,
    text::TextGlyph, text::TextInstance, text::TextPaint, text::TextRun, text::TextRunBounds,
};

fn red() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
}

fn blue() -> Color {
    Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()
}

fn bounds() -> Rect {
    Rect::try_new(0.0, 0.0, 32.0, 16.0).unwrap()
}

fn render(scene: &Scene) -> Error {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    pollster::block_on(renderer.render(&mut surface, scene, Parameters::default())).unwrap_err()
}

#[track_caller]
fn assert_geometry(scene: &Scene, reason: GeometryRealizationError, text: &str) {
    let error = render(scene);
    assert_eq!(error.code(), ErrorCode::GeometryRealization);
    assert_eq!(error.geometry_realization_diagnostic(), Some(&reason));
    assert_eq!(error.message(), text);
    assert_eq!(error.to_string(), text);
}

#[track_caller]
fn assert_gradient(scene: &Scene, reason: GradientRealizationError, text: &str) {
    let error = render(scene);
    assert_eq!(error.code(), ErrorCode::GradientRealization);
    assert_eq!(error.gradient_realization_diagnostic(), Some(&reason));
    assert_eq!(error.message(), text);
    assert_eq!(error.to_string(), text);
}

#[test]
fn an_extreme_transform_coefficient_is_a_transform_refusal() {
    // Each finite local rectangle has width MAX; the scale coefficient is tiny but nonzero.
    let mut scene = Scene::new();
    for x in [-f64::MAX, 0.0] {
        scene.transform(
            Transform::try_new([16.0 * f64::MIN_POSITIVE, 0.0, 0.0, 1.0, 16.0, 0.0]).unwrap(),
            |scene| {
                scene.fill(Rect::try_new(x, 0.0, f64::MAX, 16.0).unwrap(), red());
            },
        );
    }
    assert_geometry(
        &scene,
        GeometryRealizationError::UnrepresentableTransform,
        "a transform coefficient cannot be represented natively",
    );
}

#[test]
fn a_path_edge_that_collapses_natively_names_its_element() {
    // The origin 2^24 is exactly representable; the width 0.125 is lost when added to it natively.
    let mut scene = Scene::new();
    let origin = 2.0_f64.powi(24);
    scene.transform(Transform::translation(-origin, 0.0).unwrap(), |scene| {
        scene.fill(Rect::try_new(origin, 0.0, 0.125, 16.0).unwrap(), red());
    });
    assert_geometry(
        &scene,
        GeometryRealizationError::UnrepresentablePath { element_index: 1 },
        "path element 1 cannot be converted or retained natively",
    );
}

#[test]
fn a_stroke_width_below_native_precision_is_a_stroke_refusal() {
    let mut scene = Scene::new();
    scene.stroke(bounds(), Stroke::try_new(f64::MIN_POSITIVE).unwrap(), red());
    assert_geometry(
        &scene,
        GeometryRealizationError::UnrepresentableStroke,
        "a stroke parameter cannot be used natively",
    );
}

#[test]
fn a_blur_below_native_precision_is_a_blur_refusal() {
    let mut scene = Scene::new();
    scene.shadow(
        Rect::try_new(2.0, 2.0, 28.0, 12.0).unwrap(),
        Shadow::try_new(
            Point::try_new(0.0, 0.0).unwrap(),
            f64::MIN_POSITIVE,
            0.0,
            red(),
        )
        .unwrap(),
    );
    assert_geometry(
        &scene,
        GeometryRealizationError::UnrepresentableBlur,
        "a blur parameter cannot be represented natively",
    );
}

#[test]
fn a_glyph_placement_below_native_precision_names_its_input_index() {
    let font = FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    // The authentic ink box of the visible glyph, translated by the tiny x placement below.
    let visible = [TextGlyph::try_new(58, 0.0, 16.0, 16.0).unwrap()];
    let canonical = font
        .prepare_glyph_run(
            &TextInstance::try_new(16.0, vec![], GlyphSynthesis::none()).unwrap(),
            &visible,
            GlyphRunLimits::new(1, 16, 1 << 20, 1 << 20),
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
    let ink = canonical.bounds().ink_rect().unwrap();
    let tiny_ink = TextRunBounds::try_ink(
        Rect::try_new(
            ink.x() + f64::MIN_POSITIVE,
            ink.y(),
            ink.width(),
            ink.height(),
        )
        .unwrap(),
    )
    .unwrap();
    // The empty first glyph contributes no ink; the diagnostic index stays in the input order.
    let glyphs = [
        TextGlyph::try_new(3, f64::MAX, f64::MAX, 16.0).unwrap(),
        TextGlyph::try_new(58, f64::MIN_POSITIVE, 16.0, 16.0).unwrap(),
    ];
    let mut scene = Scene::new();
    scene.text_run(
        TextRun::try_new(
            FontRef::new(2111).with_data(font),
            16.0,
            Transform::identity(),
            TextPaint::try_fill(red().into()).unwrap(),
            &glyphs,
            tiny_ink,
        )
        .unwrap(),
    );
    assert_geometry(
        &scene,
        GeometryRealizationError::UnrepresentableGlyph { glyph_index: 1 },
        "glyph 1 cannot be represented natively",
    );
}

fn stops(values: &[(f64, Color)]) -> Vec<surgeist_render::paint::GradientStop> {
    values
        .iter()
        .map(|&(position, color)| {
            surgeist_render::paint::GradientStop::try_new(position, color).unwrap()
        })
        .collect()
}

fn linear(values: &[(f64, Color)]) -> Scene {
    let gradient = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(16.0, 0.0).unwrap(),
        stops(values),
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.fill(bounds(), gradient);
    scene
}

#[test]
fn a_stop_that_loses_its_order_natively_names_that_stop() {
    // after_half is the next f64 above 0.5 but rounds to the same native f32, so the stop keeps
    // its source position only by crossing its neighbour.
    let after_half = f64::from_bits(0.5_f64.to_bits() + 1);
    assert_ne!(0.5, after_half);
    assert_eq!(0.5_f32, after_half as f32);
    let scene = linear(&[
        (0.0, red()),
        (0.5, red()),
        (after_half, blue()),
        (1.0, blue()),
    ]);
    assert_gradient(
        &scene,
        GradientRealizationError::UnrepresentableStop { stop_index: 2 },
        "stop 2 cannot keep its source position, ordering or separation natively",
    );
}

#[test]
fn a_nonzero_stop_position_that_flushes_to_zero_names_that_stop() {
    // The smallest subnormal is nonzero in the source but is flushed to zero natively: the stop would
    // no longer keep its source position, which is the case the sentence names first.
    let scene = linear(&[(0.0, red()), (f64::from_bits(1), blue()), (1.0, blue())]);
    let error = render(&scene);
    assert_eq!(error.code(), ErrorCode::GradientRealization);
    assert_eq!(
        error.gradient_realization_diagnostic(),
        Some(&GradientRealizationError::UnrepresentableStop { stop_index: 1 })
    );
    assert_eq!(
        error.to_string(),
        "stop 1 cannot keep its source position, ordering or separation natively"
    );
}
