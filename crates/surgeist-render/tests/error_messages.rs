//! Error text observed through public operations.
//!
//! Every test obtains an error from a documented public operation and compares its text with a
//! sentence written from that operation's contract (the validation rule in the operation's
//! documentation, a font's own table, a limit the test sets) rather than from the message
//! implementation. `Error` prints its message and nothing else; the classification is `code()`.
#![cfg(not(target_arch = "wasm32"))]

use std::error::Error as _;
use std::sync::Arc;
use surgeist_render::{
    Color, ConvolutionInput, Dash, DisplacementInput, DisplacementSource, DropShadowInput,
    EffectQualityPolicy, Error, ErrorCode, FilterColorSpace, FilterDomains, GaussianInput,
    GaussianPreparationError, GaussianSource, GeometryRealizationError, GlyphRunLimits,
    GlyphSynthesis, Gradient, GradientHint, GradientStop, MorphologyInput, NormalizedCoordinate,
    OffsetInput, Options, Parameters, Point, Rect, RenderAccountingContext, RenderAccountingError,
    RenderAccountingOptions, RenderLimits, RenderWork, Renderer, Scene, Size, Stroke, TextGlyph,
    TextInstance, prepare_convolution, prepare_displacement, prepare_drop_shadow, prepare_gaussian,
    prepare_morphology, prepare_native_displacement_map, prepare_offset,
};

const FONT: &[u8] = include_bytes!("fixtures/fonts/ahem/Ahem.ttf");

fn context() -> RenderAccountingContext {
    RenderAccountingContext::default()
}

fn unit() -> Rect {
    Rect::try_new(0.0, 0.0, 8.0, 8.0).unwrap()
}

fn domains() -> FilterDomains {
    FilterDomains::try_new(unit(), unit(), unit()).unwrap()
}

#[track_caller]
fn assert_text(error: &Error, code: ErrorCode, message: &str) {
    assert_eq!(error.code(), code);
    assert_eq!(error.message(), message);
    assert_eq!(error.to_string(), message);
}

// ---- standalone filter preparation: the documented rule is "a reached resolved value must be finite" ----

#[test]
fn gaussian_refuses_a_nan_horizontal_deviation_by_name() {
    let input = GaussianInput::new().with_standard_deviation(f64::NAN);
    let error = prepare_gaussian(input, FilterColorSpace::Srgb, &mut context()).unwrap_err();
    assert_text(
        &error,
        ErrorCode::InvalidInput,
        "Gaussian parameter horizontal standard deviation has the invalid value NaN",
    );
    assert!(error.source().is_none());
}

#[test]
fn gaussian_refuses_an_infinite_vertical_deviation_by_name() {
    let input = GaussianInput::new().with_standard_deviations(1.0, f64::INFINITY);
    let error = prepare_gaussian(input, FilterColorSpace::LinearSrgb, &mut context()).unwrap_err();
    assert_eq!(
        error.message(),
        "Gaussian parameter vertical standard deviation has the invalid value inf"
    );
}

#[test]
fn offset_refuses_an_infinite_horizontal_displacement() {
    let error = prepare_offset(
        OffsetInput::new().with_dx(f64::INFINITY),
        domains(),
        &mut context(),
    )
    .unwrap_err();
    assert_text(
        &error,
        ErrorCode::InvalidInput,
        "offset parameter horizontal displacement has the invalid value inf",
    );
}

#[test]
fn offset_refuses_a_nan_vertical_displacement() {
    let error = prepare_offset(
        OffsetInput::new().with_dy(f64::NAN),
        domains(),
        &mut context(),
    )
    .unwrap_err();
    assert_eq!(
        error.message(),
        "offset parameter vertical displacement has the invalid value NaN"
    );
}

#[test]
fn morphology_refuses_a_nan_vertical_radius() {
    let error = prepare_morphology(
        MorphologyInput::new().with_radii(1.0, f64::NAN),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap_err();
    assert_text(
        &error,
        ErrorCode::InvalidInput,
        "morphology parameter vertical radius has the invalid value NaN",
    );
}

#[test]
fn convolution_refuses_an_order_below_one() {
    let kernel = [1.0];
    let input = ConvolutionInput::new(&kernel).with_order(0.0, Some(1.0));
    let error =
        prepare_convolution(input, domains(), FilterColorSpace::Srgb, &mut context()).unwrap_err();
    assert_eq!(
        error.message(),
        "convolution parameter horizontal order has the invalid value 0"
    );
}

#[test]
fn convolution_refuses_a_nan_kernel_coefficient() {
    let kernel = [1.0, f64::NAN, 1.0, 1.0];
    let input = ConvolutionInput::new(&kernel).with_order(2.0, Some(2.0));
    let error =
        prepare_convolution(input, domains(), FilterColorSpace::Srgb, &mut context()).unwrap_err();
    assert_eq!(
        error.message(),
        "convolution parameter kernel coefficient has the invalid value NaN"
    );
}

#[test]
fn drop_shadow_refuses_a_flood_opacity_above_one() {
    let error = prepare_drop_shadow(
        DropShadowInput::new().with_flood_opacity(2.0),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap_err();
    assert_text(
        &error,
        ErrorCode::InvalidInput,
        "drop shadow flood opacity 2 is not finite or is outside 0 to 1",
    );
}

#[test]
fn displacement_refuses_a_nan_scale() {
    let error = prepare_displacement(
        DisplacementInput::new().with_scale(f64::NAN),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap_err();
    assert_eq!(
        error.message(),
        "displacement parameter scale has the invalid value NaN"
    );
}

#[test]
fn native_displacement_map_states_the_required_byte_length() {
    // A 2 by 2 straight-byte RGBA map needs 2 * 2 * 4 bytes.
    let error = prepare_native_displacement_map(
        Size::try_new(2.0, 2.0).unwrap(),
        Arc::from(vec![0u8; 15]),
        unit(),
        &mut context(),
    )
    .unwrap_err();
    assert_text(
        &error,
        ErrorCode::InvalidInput,
        "displacement map has 15 bytes but its dimensions require 16",
    );
}

#[test]
fn native_displacement_map_refuses_a_fractional_pixel_size() {
    let error = prepare_native_displacement_map(
        Size::try_new(1.5, 2.0).unwrap(),
        Arc::from(vec![0u8; 12]),
        unit(),
        &mut context(),
    )
    .unwrap_err();
    assert_eq!(
        error.message(),
        "displacement map size 1.5 by 2 is not a pair of finite nonnegative integers that fit in u32"
    );
}

#[test]
fn periodic_sources_need_a_positive_period() {
    let period = Rect::try_new(0.0, 0.0, 0.0, 5.0).unwrap();
    let gaussian = GaussianSource::periodic(Scene::new(), period, unit(), unit()).unwrap_err();
    assert_eq!(
        gaussian.gaussian_diagnostic(),
        Some(&GaussianPreparationError::EmptyPeriod)
    );
    assert_text(
        &gaussian,
        ErrorCode::InvalidInput,
        "a periodic source must have positive width and height",
    );
    let displacement =
        DisplacementSource::periodic(Scene::new(), period, unit(), unit()).unwrap_err();
    assert_eq!(
        displacement.message(),
        "a periodic source must have positive width and height"
    );
}

// ---- glyph preparation ----

#[test]
fn a_coordinate_outside_the_normalized_domain_is_named_with_its_bits() {
    // Normalized coordinates are signed F2Dot14 values: -16384..=16384 are the domain.
    let error = NormalizedCoordinate::try_from_bits(20000).unwrap_err();
    assert_text(
        &error,
        ErrorCode::GlyphPreparation,
        "a variation coordinate is outside the normalized domain, which is -1 to 1 as a scalar and -16384 to 16384 as 2.14 bits",
    );
}

#[test]
fn a_scalar_off_the_f2dot14_lattice_is_refused_instead_of_quantized() {
    // 0.3 * 16384 = 4915.2 is not an integer, so the scalar would change.
    let error = NormalizedCoordinate::try_from_scalar(0.3).unwrap_err();
    assert_eq!(
        error.message(),
        "variation coordinate 0.3 would change during quantization"
    );
}

#[test]
fn synthesis_rejects_nonfinite_skew() {
    let error = GlyphSynthesis::try_new(f32::NAN, 0.0).unwrap_err();
    assert_text(
        &error,
        ErrorCode::GlyphPreparation,
        "synthesis skew or embolden is not finite, or embolden is negative",
    );
}

#[test]
fn synthesis_rejects_negative_embolden() {
    let error = GlyphSynthesis::try_new(0.0, -1.0).unwrap_err();
    assert_eq!(
        error.message(),
        "synthesis skew or embolden is not finite, or embolden is negative"
    );
}

#[test]
fn a_zero_text_size_fails_the_canonical_size_check() {
    let error = TextInstance::try_new(0.0, vec![], GlyphSynthesis::none()).unwrap_err();
    assert_text(
        &error,
        ErrorCode::GlyphPreparation,
        "glyph numeric check failed at canonical size",
    );
}

fn font() -> surgeist_render::FontData {
    surgeist_render::FontData::try_from_bytes(
        FONT.to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap()
}

/// The glyph count of the face, read from the font file's own `maxp` table.
fn ahem_glyph_count() -> u16 {
    let tables = u16::from_be_bytes([FONT[4], FONT[5]]) as usize;
    for index in 0..tables {
        let record = 12 + index * 16;
        if &FONT[record..record + 4] == b"maxp" {
            let offset =
                u32::from_be_bytes(FONT[record + 8..record + 12].try_into().unwrap()) as usize;
            return u16::from_be_bytes([FONT[offset + 4], FONT[offset + 5]]);
        }
    }
    panic!("Ahem has a maxp table");
}

#[test]
fn a_glyph_id_beyond_the_face_names_the_actual_glyph_count() {
    let instance = TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyph = TextGlyph::try_new(60_000, 0.0, 0.0, 1000.0).unwrap();
    let error = font()
        .prepare_glyph_run(
            &instance,
            &[glyph],
            GlyphRunLimits::new(1, 5, 1 << 20, 1 << 20),
            &mut context(),
        )
        .unwrap_err();
    let count = ahem_glyph_count();
    assert_text(
        &error,
        ErrorCode::GlyphPreparation,
        &format!("glyph 60000 is outside the face's {count} glyphs"),
    );
}

#[test]
fn a_caller_glyph_limit_of_zero_refuses_with_the_limit_in_the_text() {
    // Ahem's one rectangular glyph is one original input; the caller allows none.
    let instance = TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyph = TextGlyph::try_new(58, 0.0, 0.0, 1000.0).unwrap();
    let error = font()
        .prepare_glyph_run(
            &instance,
            &[glyph],
            GlyphRunLimits::new(0, 5, 1 << 20, 1 << 20),
            &mut context(),
        )
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::GlyphPreparation);
    assert_eq!(error.message(), "glyphs requires 1, above the limit of 0");
}

// ---- accounting ----

#[test]
fn a_work_limit_for_a_unit_no_operation_produces_is_refused() {
    // The accounting reference lists result transfer as checked-unlimited work.
    let error = RenderLimits::new()
        .with_work_limit(RenderWork::PreparedTransferredBytes, 5)
        .unwrap_err();
    assert_eq!(
        error,
        RenderAccountingError::InvalidLimit(RenderWork::PreparedTransferredBytes)
    );
    assert_eq!(
        error.to_string(),
        "prepared transferred bytes work is not produced by the operations this limit applies to and cannot be limited"
    );
    assert!(error.source().is_none());
}

#[test]
fn a_refused_work_limit_reports_the_refusal_as_the_cause() {
    // Glyph preparation visits each font table record once; capping the visits at the first
    // directory sweep refuses the next one.
    let records = u64::from(u16::from_be_bytes([FONT[4], FONT[5]]));
    let directory = records * (records + 1) / 2;
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::FontRecordVisits, directory)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let instance = TextInstance::try_new(1000.0, vec![], GlyphSynthesis::none()).unwrap();
    let glyph = TextGlyph::try_new(58, 0.0, 0.0, 1000.0).unwrap();
    let error = font()
        .prepare_glyph_run(
            &instance,
            &[glyph],
            GlyphRunLimits::new(1, 5, 1 << 20, 1 << 20),
            &mut context,
        )
        .unwrap_err();
    let diagnostic = error.accounting_diagnostic().unwrap();
    assert_eq!(error.code(), ErrorCode::Accounting);
    let refusal = context
        .last_attempt()
        .unwrap()
        .attempt()
        .unwrap()
        .refusal()
        .unwrap();
    assert_eq!(refusal.current(), directory);
    // The wrapper names what failed; the refusal, its cause, carries the figures.
    assert_eq!(diagnostic.to_string(), "accounting refused a change");
    assert_eq!(
        diagnostic.source().unwrap().to_string(),
        format!(
            "font record visits work refused 1 in the render.attempt phase at usage {directory}: the limit is {directory}"
        )
    );
    assert_eq!(
        error.to_string(),
        format!(
            "accounting refused a change: font record visits work refused 1 in the render.attempt phase at usage {directory}: the limit is {directory}"
        )
    );
}

// ---- gradient authoring ----

fn color() -> Color {
    Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()
}

#[test]
fn a_hint_on_the_last_stop_names_that_stop() {
    let stops = vec![
        GradientStop::try_new(0.0, color()).unwrap(),
        GradientStop::try_new(1.0, color())
            .unwrap()
            .with_hint_after(GradientHint::try_new(1.0).unwrap()),
    ];
    let error = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(1.0, 0.0).unwrap(),
        stops,
    )
    .unwrap_err();
    assert_text(
        &error,
        ErrorCode::GradientHint,
        "stop 1 is the last stop and cannot carry an outgoing hint",
    );
}

#[test]
fn a_hint_outside_its_neighbouring_stops_names_the_stop_carrying_it() {
    let stops = vec![
        GradientStop::try_new(0.0, color())
            .unwrap()
            .with_hint_after(GradientHint::try_new(0.75).unwrap()),
        GradientStop::try_new(0.5, color()).unwrap(),
    ];
    let error = Gradient::try_linear(
        Point::try_new(0.0, 0.0).unwrap(),
        Point::try_new(1.0, 0.0).unwrap(),
        stops,
    )
    .unwrap_err();
    assert_eq!(
        error.message(),
        "the hint after stop 0 lies outside the interval of its adjacent colors"
    );
}

#[test]
fn a_sweep_gradient_refuses_any_hint() {
    let stops = vec![
        GradientStop::try_new(0.0, color())
            .unwrap()
            .with_hint_after(GradientHint::try_new(0.5).unwrap()),
        GradientStop::try_new(1.0, color()).unwrap(),
    ];
    let error = Gradient::try_sweep(Point::try_new(0.0, 0.0).unwrap(), stops).unwrap_err();
    assert_text(
        &error,
        ErrorCode::GradientHint,
        "a hinted full-turn sweep after stop 0 is not supported",
    );
}

// ---- native geometry (renders on the real GPU, like tests/dash_cycle.rs) ----

#[test]
fn a_dash_cycle_longer_than_native_arithmetic_allows_is_a_geometry_refusal() {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::try_new(32.0, 16.0).unwrap(), 1.0))
            .unwrap();
    // Each interval is finite; the odd repeated native period overflows.
    let dash = Dash::try_new(0.0, &[f64::MAX]).unwrap();
    let stroke = Stroke::try_new(2.0).unwrap().try_dash(dash).unwrap();
    let mut scene = Scene::new();
    scene.stroke(
        Rect::try_new(2.0, 2.0, 28.0, 12.0).unwrap(),
        stroke,
        color(),
    );
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    assert_eq!(
        error.geometry_realization_diagnostic(),
        Some(&GeometryRealizationError::InsufficientNativeCapacity)
    );
    assert_text(
        &error,
        ErrorCode::GeometryRealization,
        "native geometry cannot meet the required rank, precision, range or dash behavior",
    );
}
