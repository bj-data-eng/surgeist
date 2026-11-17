//! Error text of filter and displacement refusals reached through short public routes.
//!
//! Every expected sentence is written from the contract of the operation: the documented rule that
//! the operation enforces, or the number the test supplies. `Error` prints its message only.
//!
//! Not reachable through a public route, so covered by the tables in `error_display.rs`:
//! `DisplacementPreparationError::InvalidMapDomain` (a `Rect` cannot be built with a non-finite
//! origin or a negative extent, which are the only inputs that variant refuses), and the `Numeric`
//! variants of every filter, which need an internal grid or table calculation to fail (the variants
//! carry no value in their text for that reason).
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use surgeist_render::{
    Error, ErrorCode, accounting::RenderAccountingContext, displacement::DisplacementInput,
    displacement::DisplacementSource, displacement::prepare_displacement,
    displacement::prepare_native_displacement_map, effect::FilterColorSpace,
    gaussian::GaussianInput, gaussian::GaussianSource, gaussian::prepare_gaussian, geometry::Rect,
    geometry::Size, morphology::MorphologyInput, morphology::MorphologySource,
    morphology::prepare_morphology, native_shadow::DropShadowInput,
    native_shadow::DropShadowSource, native_shadow::prepare_drop_shadow, paint::Color,
    scene::Layer, scene::Scene, style::filter::BackdropCaptureBounds,
    style::filter::BackdropFilterInput, style::filter::FilterBlur, style::filter::FilterList,
    style::filter::FilterOp,
};

fn unit() -> Rect {
    Rect::try_new(0.0, 0.0, 8.0, 8.0).unwrap()
}

fn context() -> RenderAccountingContext {
    RenderAccountingContext::default()
}

#[track_caller]
fn assert_message(error: &Error, code: ErrorCode, text: &str) {
    assert_eq!(error.code(), code);
    assert_eq!(error.message(), text);
    assert_eq!(error.to_string(), text);
}

// ---- displacement map size ----

#[test]
fn a_displacement_map_size_that_is_not_an_integer_pair_is_named_in_pixels() {
    let error = prepare_native_displacement_map(
        Size::try_new(1.5, 2.0).unwrap(),
        Arc::from(vec![]),
        unit(),
        &mut context(),
    )
    .unwrap_err();
    assert_message(
        &error,
        ErrorCode::InvalidInput,
        "displacement map size 1.5 by 2 is not a pair of finite nonnegative integers that fit in u32",
    );
}

#[test]
fn an_astronomical_displacement_map_size_stays_short() {
    let error = prepare_native_displacement_map(
        Size::try_new(1e300, 1.0).unwrap(),
        Arc::from(vec![]),
        unit(),
        &mut context(),
    )
    .unwrap_err();
    assert_message(
        &error,
        ErrorCode::InvalidInput,
        "displacement map size 1e300 by 1 is not a pair of finite nonnegative integers that fit in u32",
    );
}

#[test]
fn a_displacement_map_whose_byte_length_exceeds_u64_names_its_size() {
    // 4294967295 by 4294967295 pixels at four bytes each is more than 2^64 bytes.
    let size = Size::try_new(4_294_967_295.0, 4_294_967_295.0).unwrap();
    let error = prepare_native_displacement_map(size, Arc::from(vec![]), unit(), &mut context())
        .unwrap_err();
    assert_message(
        &error,
        ErrorCode::InvalidInput,
        "the byte length of a displacement map of 4294967295 by 4294967295 pixels cannot be represented",
    );
}

// ---- periodic sources: the documented rule is a positive period width and height ----

#[test]
fn every_periodic_source_needs_a_positive_period() {
    let period = Rect::try_new(0.0, 0.0, 5.0, 0.0).unwrap();
    let text = "a periodic source must have positive width and height";
    assert_message(
        &MorphologySource::periodic(Scene::new(), period, unit(), unit()).unwrap_err(),
        ErrorCode::InvalidInput,
        text,
    );
    assert_message(
        &DropShadowSource::periodic(Scene::new(), period, unit(), unit()).unwrap_err(),
        ErrorCode::InvalidInput,
        text,
    );
    assert_message(
        &GaussianSource::periodic(Scene::new(), period, unit(), unit()).unwrap_err(),
        ErrorCode::InvalidInput,
        text,
    );
    assert_message(
        &DisplacementSource::periodic(Scene::new(), period, unit(), unit()).unwrap_err(),
        ErrorCode::InvalidInput,
        text,
    );
}

// ---- a constant or periodic source cannot share a layer with a backdrop filter ----

fn backdrop_layer() -> Layer {
    let blur = FilterBlur::try_new(1.0).unwrap();
    let filters = FilterList::try_ops(vec![FilterOp::blur(blur)]).unwrap();
    let capture = BackdropCaptureBounds::try_new(unit()).unwrap();
    Layer::new()
        .try_backdrop_filter(BackdropFilterInput::try_new(filters, capture, None).unwrap())
        .unwrap()
}

fn black() -> Color {
    Color::try_rgba(0.0, 0.0, 0.0, 1.0).unwrap()
}

#[test]
fn a_same_layer_backdrop_with_an_infinite_source_is_refused_by_every_filter_layer() {
    let text = "a same-layer backdrop cannot be combined with a declared infinite source";
    let mut scene = Scene::new();

    let gaussian = prepare_gaussian(
        GaussianInput::new().with_standard_deviation(1.0),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap();
    let source = GaussianSource::constant(black(), unit(), unit()).unwrap();
    let error = scene
        .gaussian_layer(backdrop_layer(), gaussian, source)
        .map(|_| ())
        .unwrap_err();
    assert_message(&error, ErrorCode::InvalidInput, text);

    let morphology = prepare_morphology(
        MorphologyInput::new().with_radii(1.0, 1.0),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap();
    let source = MorphologySource::constant(black(), unit(), unit()).unwrap();
    let error = scene
        .morphology_layer(backdrop_layer(), morphology, source)
        .map(|_| ())
        .unwrap_err();
    assert_message(&error, ErrorCode::InvalidInput, text);

    let shadow = prepare_drop_shadow(
        DropShadowInput::new().with_standard_deviations(1.0, 1.0),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap();
    let source = DropShadowSource::constant(black(), unit(), unit()).unwrap();
    let error = scene
        .drop_shadow_layer(backdrop_layer(), shadow, source)
        .map(|_| ())
        .unwrap_err();
    assert_message(&error, ErrorCode::InvalidInput, text);

    let displacement = prepare_displacement(
        DisplacementInput::new().with_scale(1.0),
        FilterColorSpace::Srgb,
        &mut context(),
    )
    .unwrap();
    let map = prepare_native_displacement_map(
        Size::try_new(1.0, 1.0).unwrap(),
        Arc::from(vec![0_u8; 4]),
        unit(),
        &mut context(),
    )
    .unwrap();
    let source = DisplacementSource::constant(black(), unit(), unit()).unwrap();
    let error = scene
        .displacement_layer(backdrop_layer(), displacement, source, map)
        .map(|_| ())
        .unwrap_err();
    assert_message(&error, ErrorCode::InvalidInput, text);
}
