//! `Error` text of the native Paint gradient preparation refusals, obtained through the public operations.
#![cfg(feature = "render-paint")]
#![cfg(not(target_arch = "wasm32"))]

use surgeist_layout as layout;
use surgeist_ledger::ProfileMode;
use surgeist_paint::{
    accounting::{PaintLimits, PaintOptions},
    gradient_stops::{
        GradientPosition, GradientStopItem, GradientStopPositions, GradientStops,
        resolve_gradient_stops,
    },
    linear_gradient::{LinearGradientDirection, LinearGradientLine, resolve_linear_gradient},
    radial_gradient::{RadialGradientGeometry, RadialGradientSize, resolve_radial_gradient},
};
use surgeist_render::{
    Color, ErrorCode, RenderAccountingContext, prepare_linear_gradient, prepare_radial_gradient,
    prepare_repeating_linear_gradient, prepare_repeating_radial_gradient,
};

fn options() -> PaintOptions {
    PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)
}

fn position(length: f64) -> GradientPosition {
    GradientPosition {
        length,
        fraction: 0.0,
    }
}

fn stops_of_length(length: f64) -> Vec<GradientStopItem<Color>> {
    let red = Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap();
    let blue = Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap();
    vec![
        GradientStopItem::Stop {
            payload: red,
            positions: GradientStopPositions::One(position(0.0)),
        },
        GradientStopItem::Stop {
            payload: blue,
            positions: GradientStopPositions::One(position(length)),
        },
    ]
}

fn resolved(source: &[GradientStopItem<Color>], length: f64) -> GradientStops<'_, Color> {
    resolve_gradient_stops(length, source, options())
        .result
        .unwrap()
}

fn line(length: f64) -> LinearGradientLine {
    resolve_linear_gradient(
        layout::Size::<f64>::new(length, 1.0),
        LinearGradientDirection::Right,
        options(),
    )
    .result
    .unwrap()
}

fn ellipse(x: f64, y: f64) -> RadialGradientGeometry {
    resolve_radial_gradient(
        layout::Size::<f64>::new(16.0, 8.0),
        Some(layout::Point::new(8.0, 4.0)),
        RadialGradientSize::EllipseRadii(layout::Size::new(position(x), position(y))),
        options(),
    )
    .result
    .unwrap()
}

#[test]
fn a_zero_length_line_is_refused_with_the_documented_zero_length_sentence() {
    let source = stops_of_length(10.0);
    let used = resolved(&source, 10.0);
    let error = prepare_linear_gradient(&line(0.0), &used, &mut RenderAccountingContext::default())
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::PaintGradient);
    assert_eq!(
        error.message(),
        "a linear gradient of zero length cannot be prepared"
    );
    assert_eq!(
        error.to_string(),
        "paint gradient preparation refused: a linear gradient of zero length cannot be prepared"
    );
    let repeating = prepare_repeating_linear_gradient(
        &line(0.0),
        &used,
        &mut RenderAccountingContext::default(),
    )
    .unwrap_err();
    assert_eq!(repeating.message(), error.message());
}

#[test]
fn zero_circle_and_zero_height_radial_geometry_name_their_case() {
    let source = stops_of_length(10.0);
    let used = resolved(&source, 10.0);
    let zero_circle = resolve_radial_gradient(
        layout::Size::new(16.0, 8.0),
        None,
        RadialGradientSize::CircleRadius(0.0),
        options(),
    )
    .result
    .unwrap();
    for (geometry, label) in [
        (zero_circle, "zero-circle"),
        (ellipse(2.0, 0.0), "zero-height"),
    ] {
        let error =
            prepare_radial_gradient(&geometry, &used, &mut RenderAccountingContext::default())
                .unwrap_err();
        assert_eq!(
            error.message(),
            format!("a radial gradient of the {label} case cannot be prepared as a native field")
        );
        assert_eq!(error.code(), ErrorCode::PaintGradient);
    }
    let error = prepare_repeating_radial_gradient(
        &ellipse(2.0, 0.0),
        &used,
        &mut RenderAccountingContext::default(),
    )
    .unwrap_err();
    assert!(error.message().contains("zero-height"));
}

#[test]
fn stops_measured_on_another_line_report_both_lengths() {
    // A zero-width radial geometry measures distance from the center: its basis is zero, so
    // stops resolved against a line of length 10 do not match.
    let source = stops_of_length(10.0);
    let used = resolved(&source, 10.0);
    let error = prepare_radial_gradient(
        &ellipse(0.0, 2.0),
        &used,
        &mut RenderAccountingContext::default(),
    )
    .unwrap_err();
    assert_eq!(
        error.message(),
        "the used stops are measured on a line of length 10, but the geometry's basis is 0"
    );
    assert_eq!(
        error.to_string(),
        "paint gradient preparation refused: the used stops are measured on a line of length 10, but the geometry's basis is 0"
    );
}
