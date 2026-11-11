// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
//
// Modified concentric circular-field setup correspondence is documented in
// ../../NOTICE-VELLO.md. Qualification and resolved paint policy are Render-owned.

//! Per-use realization of resolved repeating fields. This is scalar paint
//! preparation; the native raster engine still owns all pixel production.

use super::{
    BrushTransform, FrameBrush, PreparedSampling, gradient_field::*, gradient_with_stops,
    hint_program, ordinary_gradient,
};
use crate::vello_engine::gradient_endpoints::{PadEndpoints, ProgramCoordinates};
use crate::{
    Color, Error, GradientStop, PhysicalSize, Point, RenderAccountingError, RepeatingGradient,
    Result,
    accounting::{Account, RenderPhase, RenderWork},
    color_reduction::{PremultipliedSum, premultiplied},
    paint::GradientGeometry,
};

enum RepeatingStops<'a> {
    Single(&'a GradientStop),
    Multiple(MultipleStops<'a>),
}

struct MultipleStops<'a> {
    all: &'a [GradientStop],
    first: &'a GradientStop,
    middle: &'a [GradientStop],
    last: &'a GradientStop,
}

impl<'a> RepeatingStops<'a> {
    fn borrow(stops: &'a [GradientStop]) -> Result<Self> {
        match stops {
            [only] => Ok(Self::Single(only)),
            [first, middle @ .., last] => Ok(Self::Multiple(MultipleStops {
                all: stops,
                first,
                middle,
                last,
            })),
            [] => Err(RenderAccountingError::Invariant.into()),
        }
    }
}

pub(super) fn prepare(
    gradient: &RepeatingGradient,
    paint_to_device: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    let account = account.at(RenderPhase::GradientPrepare);
    account.phase(RenderPhase::GradientPrepare, || {
        let stops = match RepeatingStops::borrow(gradient.stops())? {
            RepeatingStops::Single(stop) => {
                account.work(RenderWork::GradientStopVisits, 1)?;
                return Ok(FrameBrush::solid(stop.color().into()));
            }
            RepeatingStops::Multiple(stops) => stops,
        };
        account.work(RenderWork::GradientStopVisits, 2)?;
        let first = scalar(&account, || stops.first.position())?;
        let last = scalar(&account, || stops.last.position())?;
        let span = scalar(&account, || last - first)?;
        if scalar(&account, || span == 0.0)? {
            return solid_average(&stops, Average::Uniform, &account);
        }
        let field_geometry = match gradient.geometry() {
            GradientGeometry::HorizontalDistance { origin_x } => {
                return prepare_horizontal_distance(
                    &stops,
                    origin_x,
                    first,
                    span,
                    paint_to_device,
                    target_extent,
                    &account,
                );
            }
            GradientGeometry::Linear { start, end } => PeriodFieldGeometry::Linear { start, end },
            GradientGeometry::Radial {
                center,
                radius_x,
                radius_y,
            } => PeriodFieldGeometry::Radial {
                center,
                radius_x,
                radius_y,
            },
        };
        let geometry = period_geometry(field_geometry, first, span, &account)?;
        let (field, local) = match geometry {
            PeriodGeometry::RoundedZero => {
                return solid_average(&stops, Average::Uniform, &account);
            }
            PeriodGeometry::ZeroHeight => {
                return solid_average(&stops, Average::Weighted(span), &account);
            }
            PeriodGeometry::Pattern { field, local } => (field, local),
        };
        let device = affine_product(paint_to_device, local, &account)?;
        check_finite(device, &account)?;
        let period = match field {
            NativeField::Linear => linear_device_period(device, &account)?,
            NativeField::Radial { .. } => radial_device_period(device, &account)?,
        };
        if !scalar(&account, || period.is_finite())? {
            return Err(invalid_geometry("non-finite device period"));
        }
        if scalar(&account, || period < 2.0)? {
            return solid_average(&stops, Average::Weighted(span), &account);
        }
        let mut brush_transform = BrushTransform {
            local,
            reference_device: device,
            radial: None,
            comparison_only: false,
        };
        let native = match field {
            NativeField::Linear => {
                brush_transform = linear_brush_transform(paint_to_device, device, true, &account)?;
                let composed = affine_product(paint_to_device, brush_transform.local, &account)?;
                let transform = narrow_transform(composed, &account)?;
                if !check_native_linear(transform, &account)? {
                    return solid_average(&stops, Average::Uniform, &account);
                }
                qualify_linear(
                    brush_transform.reference_device,
                    transform,
                    target_extent,
                    false,
                    &account,
                )?;
                peniko::Gradient::new_linear((0., 0.), (1., 0.))
            }
            NativeField::Radial { start_radius } => {
                let native = narrow_transform(device, &account)?;
                let start = scalar(&account, || start_radius as f32)?;
                let end = scalar(&account, || start + 1.0)?;
                let reference = RadialReference {
                    start: start_radius,
                    end: scalar(&account, || start_radius + 1.0)?,
                    delta: 1.0,
                };
                brush_transform.radial = Some(reference);
                qualify_radial(
                    device,
                    native,
                    (start, end),
                    reference,
                    target_extent,
                    false,
                    &account,
                )?;
                peniko::Gradient::new_two_point_radial((0., 0.), start, (0., 0.), end)
            }
        }
        .with_extend(peniko::Extend::Repeat);
        let mut normalized = account.temporary(stops.all.len())?;
        let mut hinted = false;
        for stop in stops.all {
            account.work(RenderWork::GradientStopVisits, 1)?;
            hinted |= stop.hint_after().is_some();
            let position = scalar(&account, || stop.position())?;
            let difference = scalar(&account, || position - first)?;
            let quotient = scalar(&account, || difference / span)?;
            let offset = scalar(&account, || quotient as f32)?;
            account.work(RenderWork::CoordinateVisits, 4)?;
            normalized.append_slot()?.write(peniko::ColorStop::from((
                offset,
                peniko::Color::from(stop.color()),
            )));
        }
        let program = if hinted {
            let endpoints = PadEndpoints::new(
                (0., stops.first.color()),
                (1., stops.last.color()),
                &account,
            )?;
            Some(hint_program::prepare(
                stops.all,
                &normalized,
                |position, account| {
                    let difference = scalar(account, || position - first)?;
                    let quotient = scalar(account, || difference / span)?;
                    scalar(account, || quotient as f32)
                },
                endpoints,
                ProgramCoordinates::Repeat,
                &account,
            )?)
        } else {
            None
        };
        let mut output = gradient_with_stops(native, normalized, Some(brush_transform))?;
        output.sampling = program.map(PreparedSampling::Program);
        Ok(output)
    })
}

fn horizontal_phase(first: f64, span: f64, account: &Account) -> Result<f32> {
    let remainder = scalar(account, || first.rem_euclid(span))?;
    let phase = scalar(account, || remainder / span)?;
    let native = scalar(account, || phase as f32)?;
    // Compare the remainder, not the potentially already-underflowed quotient.
    if scalar(account, || remainder != 0.0)? && scalar(account, || native == 0.0)? {
        return Err(crate::GradientRealizationError::InsufficientNativeCapacity.into());
    }
    Ok(native)
}

fn prepare_horizontal_distance(
    stops: &MultipleStops<'_>,
    origin_x: f64,
    first: f64,
    span: f64,
    transform: kurbo::Affine,
    target: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    // Period needs only the linear map. Do not construct an unused transformed
    // origin before a rounded-zero/unresolved average has been selected.
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let linear = kurbo::Affine::new([a, b, c, d, 0.0, 0.0]);
    let period_device = ordinary_gradient::horizontal_device(0.0, span, linear, account)
        .map_err(ordinary_gradient::geometry_error)?;
    let period = linear_device_period(period_device, account)?;
    if !scalar(account, || period.is_finite())? {
        return Err(crate::GradientRealizationError::UnrepresentableGeometry.into());
    }
    if scalar(account, || period == 0.0)? {
        return solid_average(stops, Average::Uniform, account);
    }
    if scalar(account, || period < 2.0)? {
        return solid_average(stops, Average::Weighted(span), account);
    }
    let origin_x = scalar(account, || origin_x)?;
    let phase = horizontal_phase(first, span, account)?;
    let device = ordinary_gradient::horizontal_device(origin_x, span, transform, account)
        .map_err(ordinary_gradient::geometry_error)?;
    let brush = ordinary_gradient::horizontal_brush(
        device,
        transform,
        target,
        LinearSampling::HorizontalDistanceRepeat { phase },
        account,
    )?;
    account.work(RenderWork::GradientStopVisits, 2)?;
    let endpoints = PadEndpoints::new(
        (0.0, stops.first.color()),
        (1.0, stops.last.color()),
        account,
    )?;
    let normalize = |position: f64, account: &Account| -> Result<f32> {
        let difference = scalar(account, || position - first)?;
        let quotient = scalar(account, || difference / span)?;
        scalar(account, || quotient as f32)
    };
    let program = hint_program::prepare_projected(
        stops.all,
        |_, position, account| normalize(position, account),
        normalize,
        endpoints,
        ProgramCoordinates::HorizontalDistanceRepeat { phase },
        account,
    )?;
    let carrier = ordinary_gradient::horizontal_carrier_stops(stops.all, account)?;
    let native =
        peniko::Gradient::new_linear((0.0, 0.0), (1.0, 0.0)).with_extend(peniko::Extend::Repeat);
    let mut output = gradient_with_stops(native, carrier, Some(brush))?;
    output.sampling = Some(PreparedSampling::Program(program));
    Ok(output)
}

fn invalid_geometry(value: &'static str) -> Error {
    Error::invalid_value(
        "repeating gradient realization",
        value,
        "must have finite, representable native period geometry",
    )
}

#[derive(Clone, Copy)]
enum NativeField {
    Linear,
    Radial { start_radius: f64 },
}

enum PeriodFieldGeometry {
    Linear {
        start: Point,
        end: Point,
    },
    Radial {
        center: Point,
        radius_x: f64,
        radius_y: f64,
    },
}

enum PeriodGeometry {
    RoundedZero,
    ZeroHeight,
    Pattern {
        field: NativeField,
        local: kurbo::Affine,
    },
}

fn period_geometry(
    geometry: PeriodFieldGeometry,
    first: f64,
    span: f64,
    account: &Account,
) -> Result<PeriodGeometry> {
    let phase = scalar(account, || first.rem_euclid(span))?;
    let (field, local) = match geometry {
        PeriodFieldGeometry::Linear { start, end } => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (sx, sy, ex, ey) = (start.x(), start.y(), end.x(), end.y());
            let dx = scalar(account, || ex - sx)?;
            let dy = scalar(account, || ey - sy)?;
            let x = scalar(account, || dx * span)?;
            let y = scalar(account, || dy * span)?;
            for value in [dx, dy, x, y] {
                if !scalar(account, || value.is_finite())? {
                    return Err(invalid_geometry("non-finite linear period"));
                }
            }
            if scalar(account, || x == 0.0)? && scalar(account, || y == 0.0)? {
                return Ok(PeriodGeometry::RoundedZero);
            }
            let minus_y = scalar(account, || -y)?;
            let shift_x = scalar(account, || dx * phase)?;
            let shift_y = scalar(account, || dy * phase)?;
            let ox = scalar(account, || sx + shift_x)?;
            let oy = scalar(account, || sy + shift_y)?;
            (
                NativeField::Linear,
                kurbo::Affine::new([x, y, minus_y, x, ox, oy]),
            )
        }
        PeriodFieldGeometry::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            let radius_y = scalar(account, || radius_y)?;
            if scalar(account, || radius_y == 0.0)? {
                return Ok(PeriodGeometry::ZeroHeight);
            }
            let radius_x = scalar(account, || radius_x)?;
            let x = scalar(account, || radius_x * span)?;
            let y = scalar(account, || radius_y * span)?;
            for value in [x, y] {
                if !scalar(account, || value.is_finite())? {
                    return Err(invalid_geometry("non-finite radial period"));
                }
            }
            if scalar(account, || x == 0.0)? {
                return Ok(PeriodGeometry::RoundedZero);
            }
            if scalar(account, || y == 0.0)? {
                return Ok(PeriodGeometry::ZeroHeight);
            }
            let radius = scalar(account, || phase / span)?;
            account.work(RenderWork::CoordinateVisits, 2)?;
            let (cx, cy) = (center.x(), center.y());
            (
                NativeField::Radial {
                    start_radius: radius,
                },
                kurbo::Affine::new([x, 0., 0., y, cx, cy]),
            )
        }
    };
    check_finite(local, account)?;
    Ok(PeriodGeometry::Pattern { field, local })
}

/// Stripe spacing is |det(M)|/|M(0,1)|, not the transformed endpoint length.
fn linear_device_period(transform: kurbo::Affine, account: &Account) -> Result<f64> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let ca = scalar(account, || c.abs())?;
    let da = scalar(account, || d.abs())?;
    let scale = scalar(account, || ca.max(da))?;
    if scalar(account, || scale == 0.0)? {
        return Ok(0.0);
    }
    let c = scalar(account, || c / scale)?;
    let d = scalar(account, || d / scale)?;
    let transverse = scalar(account, || c.hypot(d))?;
    let dx = scalar(account, || d / transverse)?;
    let first = scalar(account, || a * dx)?;
    let dy = scalar(account, || c / transverse)?;
    let second = scalar(account, || b * dy)?;
    let difference = scalar(account, || first - second)?;
    scalar(account, || difference.abs())
}

/// Stable smallest singular value, normalized before squaring its axes.
fn radial_device_period(transform: kurbo::Affine, account: &Account) -> Result<f64> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let aa = scalar(account, || a.abs())?;
    let ba = scalar(account, || b.abs())?;
    let ca = scalar(account, || c.abs())?;
    let da = scalar(account, || d.abs())?;
    let scale = scalar(account, || aa.max(ba))?;
    let scale = scalar(account, || scale.max(ca))?;
    let scale = scalar(account, || scale.max(da))?;
    if scalar(account, || scale == 0.0)? {
        return Ok(0.0);
    }
    let a = scalar(account, || a / scale)?;
    let b = scalar(account, || b / scale)?;
    let c = scalar(account, || c / scale)?;
    let d = scalar(account, || d / scale)?;
    let aa = scalar(account, || a * a)?;
    let bb = scalar(account, || b * b)?;
    let cc = scalar(account, || c * c)?;
    let dd = scalar(account, || d * d)?;
    let sum = scalar(account, || aa + bb)?;
    let sum = scalar(account, || sum + cc)?;
    let sum = scalar(account, || sum + dd)?;
    let ad = scalar(account, || a * d)?;
    let bc = scalar(account, || b * c)?;
    let det = scalar(account, || ad - bc)?;
    let determinant = scalar(account, || det.abs())?;
    let squared = scalar(account, || sum * sum)?;
    let four_det = scalar(account, || 4.0 * determinant)?;
    let product = scalar(account, || four_det * determinant)?;
    let difference = scalar(account, || squared - product)?;
    let discriminant = scalar(account, || difference.max(0.0))?;
    let root = scalar(account, || discriminant.sqrt())?;
    let sum = scalar(account, || sum + root)?;
    let half = scalar(account, || sum * 0.5)?;
    let largest = scalar(account, || half.sqrt())?;
    let quotient = scalar(account, || determinant / largest)?;
    scalar(account, || quotient * scale)
}

#[derive(Clone, Copy)]
enum Average {
    Uniform,
    Weighted(f64),
}

fn solid_average(
    stops: &MultipleStops<'_>,
    rule: Average,
    account: &Account,
) -> Result<FrameBrush> {
    Ok(FrameBrush::solid(average(stops, rule, account)?.into()))
}

fn average(stops: &MultipleStops<'_>, rule: Average, account: &Account) -> Result<Color> {
    let mut total = PremultipliedSum::new();
    match rule {
        Average::Uniform => {
            let intervals = scalar(account, || stops.middle.len() + 1)?;
            let denominator = scalar(account, || intervals as f64)?;
            let numerators = std::iter::once((stops.first, 0.5))
                .chain(stops.middle.iter().map(|stop| (stop, 1.0)))
                .chain(std::iter::once((stops.last, 0.5)));
            for (stop, numerator) in numerators {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let color = premultiplied(stop.color(), account)?;
                let weight = scalar(account, || numerator / denominator)?;
                total.add_weighted(color, weight, account)?;
            }
        }
        Average::Weighted(span) => {
            let mut previous = None;
            for stop in stops.all {
                account.work(RenderWork::GradientStopVisits, 1)?;
                let color = premultiplied(stop.color(), account)?;
                let position = scalar(account, || stop.position())?;
                if let Some((previous_position, previous_color)) = previous {
                    let difference = scalar(account, || position - previous_position)?;
                    let quotient = scalar(account, || difference / span)?;
                    let weight = scalar(account, || quotient * 0.5)?;
                    total.add_segment(previous_color, color, weight, account)?;
                }
                previous = Some((position, color));
            }
        }
    }
    total.finish(account)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ErrorCode, Point, RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptKind, RenderLimits, RenderQuantity, RenderResource,
    };

    fn prepare(
        gradient: &RepeatingGradient,
        transform: kurbo::Affine,
        account: &Account,
    ) -> Result<FrameBrush> {
        super::prepare(gradient, transform, PhysicalSize::new(2, 2), account)
    }

    fn linear_device_period(transform: kurbo::Affine) -> f64 {
        super::linear_device_period(transform, &Account::for_test()).unwrap()
    }
    fn radial_device_period(transform: kurbo::Affine) -> f64 {
        super::radial_device_period(transform, &Account::for_test()).unwrap()
    }

    fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::try_rgba(r, g, b, a).unwrap()
    }

    fn stop(position: f64, color: Color) -> GradientStop {
        GradientStop::try_new(position, color).unwrap()
    }

    fn weighted_stops() -> Vec<GradientStop> {
        vec![
            stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(1.0, color(0.0, 1.0, 0.0, 0.5)),
            stop(4.0, color(0.0, 0.0, 1.0, 0.0)),
        ]
    }

    fn linear(stops: Vec<GradientStop>) -> RepeatingGradient {
        RepeatingGradient::try_linear(Point::new(0.0, 0.0), Point::new(1.0, 0.0), stops).unwrap()
    }

    fn is_pattern(brush: &FrameBrush) -> bool {
        matches!(brush.as_ref(), peniko::BrushRef::Gradient(_))
    }

    fn solid(brush: &FrameBrush) -> peniko::Color {
        match brush.as_ref() {
            peniko::BrushRef::Solid(color) => color,
            _ => panic!("expected the native solid fallback"),
        }
    }

    fn assert_color(actual: Color, expected: [f32; 4]) {
        for (actual, expected) in [actual.r(), actual.g(), actual.b(), actual.a()]
            .into_iter()
            .zip(expected)
        {
            assert!((actual - expected).abs() < 1.0e-6, "{actual} != {expected}");
        }
    }

    fn multiple_stops(stops: &[GradientStop]) -> MultipleStops<'_> {
        let RepeatingStops::Multiple(stops) = RepeatingStops::borrow(stops).unwrap() else {
            panic!("expected multiple stops");
        };
        stops
    }

    #[test]
    fn qualified_stops_borrow_the_original_records() {
        assert_eq!(
            RepeatingStops::borrow(&[])
                .err()
                .unwrap()
                .accounting_diagnostic(),
            Some(RenderAccountingError::Invariant)
        );
        let original = [
            stop(7.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(7.0, color(1.0, 1.0, 1.0, 1.0)),
            stop(7.0, color(0.0, 0.0, 1.0, 1.0)),
            stop(7.0, color(0.0, 1.0, 0.0, 1.0)),
        ];
        let RepeatingStops::Single(only) = RepeatingStops::borrow(&original[..1]).unwrap() else {
            panic!("expected one stop");
        };
        assert!(std::ptr::eq(only, &original[0]));

        for length in [2, 4] {
            let records = &original[..length];
            let stops = multiple_stops(records);
            assert!(std::ptr::eq(stops.all, records));
            assert!(std::ptr::eq(stops.first, &records[0]));
            assert!(std::ptr::eq(stops.last, &records[length - 1]));
            assert!(std::ptr::eq(stops.middle, &records[1..length - 1]));
        }
    }

    #[test]
    fn averages_use_segment_integrals_and_premultiplied_srgba() {
        // Segment lengths 1,3 give endpoint weights 1/8,1/2,3/8.
        // Premultiplied sum is (1/8,1/4,0,3/8), with no transparent blue bleed.
        assert_color(
            average(
                &multiple_stops(&weighted_stops()),
                Average::Weighted(4.0),
                &Account::for_test(),
            )
            .unwrap(),
            [1.0 / 3.0, 2.0 / 3.0, 0.0, 3.0 / 8.0],
        );
        let stops = [
            stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(1.0, color(0.0, 1.0, 0.0, 1.0)),
            stop(1.0, color(0.0, 0.0, 1.0, 1.0)),
            stop(4.0, color(1.0, 1.0, 1.0, 1.0)),
        ];
        assert_color(
            average(
                &multiple_stops(&stops),
                Average::Weighted(4.0),
                &Account::for_test(),
            )
            .unwrap(),
            [0.5, 0.5, 0.75, 1.0],
        );
    }

    #[test]
    fn uniform_replacement_retains_original_color_count_and_alpha_zero() {
        let stops = [
            stop(7.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(7.0, color(1.0, 1.0, 1.0, 1.0)),
            stop(7.0, color(0.0, 0.0, 1.0, 1.0)),
        ];
        // Equal replacement spacing gives weights 1/4,1/2,1/4.
        assert_color(
            average(
                &multiple_stops(&stops),
                Average::Uniform,
                &Account::for_test(),
            )
            .unwrap(),
            [0.75, 0.5, 0.75, 1.0],
        );
        let four = [
            stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(0.0, color(0.0, 1.0, 0.0, 1.0)),
            stop(0.0, color(0.0, 0.0, 1.0, 1.0)),
            stop(0.0, color(1.0, 1.0, 1.0, 1.0)),
        ];
        assert_color(
            average(
                &multiple_stops(&four),
                Average::Uniform,
                &Account::for_test(),
            )
            .unwrap(),
            [1.0 / 3.0, 0.5, 0.5, 1.0],
        );
        let transparent = [
            stop(0.0, color(1.0, 0.0, 0.0, 0.0)),
            stop(1.0, color(0.0, 0.0, 1.0, 0.0)),
        ];
        assert_eq!(
            average(
                &multiple_stops(&transparent),
                Average::Weighted(1.0),
                &Account::for_test(),
            )
            .unwrap(),
            Color::TRANSPARENT,
        );
    }

    #[test]
    fn physical_resolution_uses_the_inverse_transpose_and_minimum_singular_value() {
        let shear = kurbo::Affine::new([2.0, 0.0, 1.5, 2.0, 0.0, 0.0]);
        // |det|/|second column| = 4/2.5 = 1.6, while the first column has length 2.
        assert!((linear_device_period(shear) - 1.6).abs() < 1.0e-14);
        let ellipse = kurbo::Affine::new([3.0, 0.0, 4.0, 3.0, 0.0, 0.0]);
        // The exact smaller singular value of this map is sqrt(13)-2.
        assert!((radial_device_period(ellipse) - (13.0_f64.sqrt() - 2.0)).abs() < 1.0e-14);
        assert_eq!(
            radial_device_period(kurbo::Affine::scale_non_uniform(100.0, 0.25)),
            0.25,
        );
    }

    #[test]
    fn linear_reference_survives_phase_loss_in_the_f64_inverse_solve() {
        let account = Account::for_test();
        for (center, period, intended_origin, recomposed_origin) in [
            (2.0_f64.powi(53), 19.0, 9.0, 8.0),
            (2.0_f64.powi(57), 17.0, 15.0, 0.0),
        ] {
            let output = kurbo::Affine::translate((-center, 0.0));
            let original = kurbo::Affine::scale(period);
            let paired = linear_brush_transform(output, output * original, true, &account).unwrap();
            // (-2^53) mod 19 = 9 and (-2^57) mod 17 = 15. These are the
            // intended device origins before adding the huge inverse translation.
            assert_eq!(paired.reference_device.as_coeffs()[4], intended_origin);
            assert_eq!((output * paired.local).as_coeffs()[4], recomposed_origin);
            assert!(intended_origin != recomposed_origin);
            let gradient = RepeatingGradient::try_linear(
                Point::new(0.0, 0.0),
                Point::new(period, 0.0),
                vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
            )
            .unwrap();
            let error = super::prepare(&gradient, output, PhysicalSize::new(32, 32), &account)
                .err()
                .expect("f64 recomposition is not the intended-field oracle");
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            assert!(error.invalid_value_diagnostic().is_some());
        }
    }

    #[test]
    fn linear_glyph_refuses_phase_loss_from_separately_narrowed_factors() {
        let center = 2.0_f64.powi(28);
        let output = kurbo::Affine::translate((-center, 0.0));
        let local = kurbo::Affine::new([16.0, 0.0, 0.0, 16.0, center + 1.0, 0.0]);
        let reference_device = kurbo::Affine::new([16.0, 0.0, 0.0, 16.0, 1.0, 0.0]);
        assert_eq!((output * local).as_coeffs(), reference_device.as_coeffs());
        let native = vello_encoding::Transform::from_kurbo(&output)
            * vello_encoding::Transform::from_kurbo(&local);
        assert_eq!(native.translation, [0.0, 0.0]);
        // Device x=1 should have phase0; the narrowed factors produce1/16.
        let intended = reference_device.as_coeffs();
        let intended_phase = (1.0 - intended[4]) / intended[0];
        let narrowed_phase = (1.0 - f64::from(native.translation[0])) / f64::from(native.matrix[0]);
        assert_eq!(intended_phase, 0.0);
        assert_eq!(narrowed_phase - intended_phase, 1.0 / 16.0);
        let error = native_glyph_transform(
            local,
            reference_device,
            false,
            output,
            PhysicalSize::new(32, 32),
            NativeGlyphField::Linear(LinearSampling::Direct),
            &Account::for_test(),
        )
        .expect_err("native glyph factors must preserve the intended linear origin");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }

    #[test]
    fn linear_projection_requires_a_guaranteed_division_domain() {
        let axis = 3.0 * 2.0_f64.powi(62);
        let native_squared = (axis as f32) * (axis as f32);
        assert!(native_squared.is_finite());
        assert!(f64::from(native_squared) > 2.0_f64.powi(126));
        assert!((1.0_f32 / native_squared).is_subnormal());
        let gradient = RepeatingGradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(axis, 0.0),
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let error = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test())
            .err()
            .expect("a finite CPU reciprocal does not certify shader projection");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }

    #[test]
    fn linear_large_translation_remains_usable_when_phase_is_exactly_preserved() {
        let center = 2.0_f64.powi(57);
        let output = kurbo::Affine::translate((-center, 0.0));
        let gradient = RepeatingGradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(16.0, 0.0),
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        let brush = prepare(&gradient, output, &account).unwrap();
        assert!(is_pattern(&brush));
        let paired = brush.transform.unwrap();
        assert_eq!(
            paired.reference_device.as_coeffs(),
            [16.0, 0.0, 0.0, 16.0, 0.0, 0.0]
        );
        let local = native_glyph_transform(
            paired.local,
            paired.reference_device,
            paired.comparison_only,
            output,
            PhysicalSize::new(32, 32),
            NativeGlyphField::Linear(LinearSampling::Direct),
            &account,
        )
        .unwrap();
        let native = vello_encoding::Transform::from_kurbo(&output) * local;
        assert_eq!(native.matrix, [16.0, 0.0, 0.0, 16.0]);
        assert_eq!(native.translation, [0.0, 0.0]);
    }

    #[test]
    fn native_linear_endpoints_reproduce_the_affine_covector_under_shear() {
        let gradient = RepeatingGradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        let transform = kurbo::Affine::new([1.0, 0.0, 0.75, 1.0, 3.0, 5.0]);
        let brush = prepare(&gradient, transform, &account).unwrap();
        let local = brush.transform().unwrap();
        let outline = vello_encoding::Transform::from_kurbo(&(transform * local));
        let brush_transform = brush.transform.unwrap();
        let glyph = vello_encoding::Transform::from_kurbo(&transform)
            * native_glyph_transform(
                brush_transform.local,
                brush_transform.reference_device,
                brush_transform.comparison_only,
                transform,
                PhysicalSize::new(2, 2),
                NativeGlyphField::Linear(LinearSampling::Direct),
                &account,
            )
            .unwrap();
        for native in [outline, glyph] {
            // Independently reproduce draw_leaf's transformed endpoint formula.
            let [a, b, _, _] = native.matrix;
            let [x, y] = native.translation;
            let dx = (x + a) - x;
            let dy = (y + b) - y;
            let inverse = 1.0 / (dx * dx + dy * dy);
            let nx = f64::from(dx * inverse);
            let ny = f64::from(dy * inverse);
            let offset = f64::from(-(x * (dx * inverse) + y * (dy * inverse)));
            assert!((nx - 0.25).abs() < 1.0e-7);
            assert!((ny + 0.1875).abs() < 1.0e-7);
            for (x, y) in [(0.0, 1.0), (1.0, 1.0), (3.0, 1.0), (9.0, 7.0)] {
                // A^-1 maps device (x,y) to (x-3-.75*(y-5), y-5).
                // The authored field increases once per four local x units.
                let expected = (x - 3.0 - 0.75 * (y - 5.0)) / 4.0;
                let error = (nx * x + ny * y + offset - expected).rem_euclid(1.0);
                assert!(error.min(1.0 - error) < 1.0e-6);
            }
        }
    }

    #[test]
    fn threshold_is_strict_and_the_same_input_reacts_to_device_scale_and_shear() {
        let gradient = linear(vec![stop(0.0, Color::BLACK), stop(1.0, Color::BLACK)]);
        let account = Account::for_test();
        for (transform, pattern) in [
            (kurbo::Affine::scale(1.999), false),
            (kurbo::Affine::scale(2.0), true),
            (kurbo::Affine::scale(100.0), true),
            (kurbo::Affine::new([2.0, 0.0, 1.5, 2.0, 0.0, 0.0]), false),
        ] {
            assert_eq!(
                is_pattern(&prepare(&gradient, transform, &account).unwrap()),
                pattern
            );
        }
    }

    #[test]
    fn canonical_patterns_preserve_negative_phase_and_duplicate_stops() {
        let stops = vec![
            stop(-1.0, Color::BLACK),
            stop(0.0, Color::TRANSPARENT),
            stop(0.0, Color::BLACK),
            stop(3.0, Color::TRANSPARENT),
        ];
        let gradient = RepeatingGradient::try_linear(
            Point::new(5.0, 7.0),
            Point::new(15.0, 7.0),
            stops.clone(),
        )
        .unwrap();
        let brush = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test()).unwrap();
        assert_eq!(
            brush.transform().unwrap().as_coeffs(),
            [40.0, 0.0, 0.0, 40.0, 35.0, 0.0]
        );
        let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
            panic!("pattern");
        };
        assert_eq!(native.extend, peniko::Extend::Repeat);
        assert_eq!(
            native
                .stops
                .0
                .iter()
                .map(|stop| stop.offset)
                .collect::<Vec<_>>(),
            [0.0, 0.25, 0.25, 1.0]
        );
        let gradient =
            RepeatingGradient::try_radial(Point::new(5.0, 7.0), 10.0, 4.0, stops).unwrap();
        let brush = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test()).unwrap();
        assert_eq!(
            brush.transform().unwrap().as_coeffs(),
            [40.0, 0.0, 0.0, 16.0, 5.0, 7.0]
        );
        let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
            panic!("pattern");
        };
        let peniko::GradientKind::Radial(radial) = native.kind else {
            panic!("radial");
        };
        assert_eq!(radial.start_radius, 0.75);
        assert_eq!(radial.end_radius, 1.75);
        assert_eq!(radial.start_center, radial.end_center);
    }

    #[test]
    fn tiny_radial_period_survives_zoom_without_the_native_radius_epsilon() {
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            1.0e-12,
            1.0e-12,
            vec![stop(0.0, Color::BLACK), stop(0.01, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        assert!(!is_pattern(
            &prepare(&gradient, kurbo::Affine::IDENTITY, &account).unwrap()
        ));
        let brush = prepare(&gradient, kurbo::Affine::scale(1.0e15), &account).unwrap();
        let peniko::BrushRef::Gradient(native) = brush.as_ref() else {
            panic!("magnified pattern");
        };
        let peniko::GradientKind::Radial(radial) = native.kind else {
            panic!("radial");
        };
        assert_eq!(radial.end_radius - radial.start_radius, 1.0);
    }

    #[test]
    fn circular_division_certifies_both_guaranteed_domain_endpoints() {
        let account = Account::for_test();
        let low = ShaderScalar::exact(f64::from(f32::MIN_POSITIVE));
        let high = ShaderScalar::exact(2.0_f64.powi(126));
        for denominator in [low, high] {
            let quotient = ShaderScalar::ONE.div(denominator, &account).unwrap();
            let exact = 1.0 / denominator.lower;
            assert!(quotient.lower <= exact && quotient.upper >= exact);
        }
        assert!(
            ShaderScalar::ONE
                .div(ShaderScalar::exact(low.lower.next_down()), &account,)
                .is_err()
        );
        assert!(
            ShaderScalar::ONE
                .div(ShaderScalar::exact(high.upper.next_up()), &account,)
                .is_err()
        );
    }

    #[test]
    fn circular_square_root_encloses_a_permitted_non_nearest_result() {
        // inverseSqrt(4) may be 1/2-2^-24 (two ULP at 1/2). Its reciprocal
        // may then be 2+3*2^-22 within the separate 2.5-ULP division allowance.
        let allowed = 2.0 + 3.0 * 2.0_f64.powi(-22);
        let root = ShaderScalar::exact(4.0).sqrt(&Account::for_test()).unwrap();
        assert!(root.lower <= allowed && root.upper >= allowed);
        assert!(allowed - 2.0 > 2.0 * 2.5 * 2.0_f64.powi(-24));
    }

    #[test]
    fn radial_inverse_refuses_a_determinant_with_uncertain_native_sign() {
        let scale = 2.0_f64.powi(26);
        let transform = kurbo::Affine::new([
            scale,
            scale,
            scale,
            scale * (1.0 + f64::from(f32::EPSILON)),
            0.0,
            0.0,
        ]);
        let native = vello_encoding::Transform::from_kurbo(&transform);
        let [a, b, c, d] = native.matrix;
        assert!((a * d - b * c).is_finite());
        assert_ne!(a * d - b * c, 0.0);
        // Its exact smaller singular value exceeds two physical pixels, but
        // permitted product rounding in the cancelling determinant spans zero.
        assert!(radial_device_period(transform) > 2.0);
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            1.0,
            1.0,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let error = super::prepare(
            &gradient,
            transform,
            PhysicalSize::new(2, 2),
            &Account::for_test(),
        )
        .err()
        .expect("a finite nominal inverse cannot certify an uncertain determinant");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }

    #[test]
    fn radial_certificate_refuses_coordinate_work_before_native_stop_storage() {
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            16.0,
            16.0,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 64)
                    .unwrap(),
            ),
        );
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                prepare(&gradient, kurbo::Affine::IDENTITY, account).map(drop)
            })
            .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("the certificate must use coordinate-work admission");
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CoordinateVisits)
        );
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert!(usage.work(RenderWork::CoordinateVisits) > 0);
        assert!(usage.work(RenderWork::CoordinateVisits) <= 64);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn radial_native_inverse_requires_a_guaranteed_division_domain() {
        // The native determinant is exactly 2^127: finite, but outside
        // WGSL's division accuracy guarantee |denominator| <= 2^126.
        // CPU evaluation's exact subnormal reciprocal is not a certificate
        // for every permitted shader evaluation. No GPU failure is asserted.
        let x = 2.0_f64.powi(63);
        let y = 2.0_f64.powi(64);
        let determinant = (x as f32) * (y as f32);
        assert!(determinant.is_finite());
        assert_eq!(f64::from(determinant), 2.0_f64.powi(127));
        assert!((1.0_f32 / determinant).is_subnormal());
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            x,
            y,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let error = super::prepare(
            &gradient,
            kurbo::Affine::IDENTITY,
            PhysicalSize::new(2, 1),
            &Account::for_test(),
        )
        .err()
        .expect("a native inverse outside the shader accuracy domain cannot certify phase");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }

    #[test]
    fn radial_phase_capacity_covers_both_target_dimensions() {
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            16.0,
            16.0,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        assert!(is_pattern(
            &prepare(&gradient, kurbo::Affine::IDENTITY, &account).unwrap()
        ));
        // On either axis the last integer sample has exact radial phase
        // (2^20 - 1)/16. Its binary32 spacing is 1/256, already larger than
        // one 512-entry ramp interval. The origin alone cannot expose this.
        let far_phase = f64::from((1_u32 << 20) - 1) / 16.0;
        assert_eq!(far_phase, 65_535.937_5);
        let native_phase = far_phase as f32;
        let spacing = native_phase.next_up() - native_phase;
        assert_eq!(spacing, 1.0 / 256.0);
        assert!(f64::from(spacing) > 1.0 / 511.0);
        // These are CPU qualification inputs; no native texture is allocated.
        for extent in [PhysicalSize::new(1 << 20, 2), PhysicalSize::new(2, 1 << 20)] {
            let error = super::prepare(&gradient, kurbo::Affine::IDENTITY, extent, &account)
                .err()
                .expect("the entire sampled rectangle needs phase capacity");
            assert_eq!(error.code(), ErrorCode::InvalidInput);
            assert!(error.invalid_value_diagnostic().is_some());
        }
    }

    #[test]
    fn radial_output_translation_cancels_a_far_logical_center() {
        let center = f64::from(1_u32 << 28);
        let gradient = RepeatingGradient::try_radial(
            Point::new(center, 0.0),
            16.0,
            16.0,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        let error = prepare(&gradient, kurbo::Affine::IDENTITY, &account)
            .err()
            .expect("the uncancelled far center loses native phase");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        let output = kurbo::Affine::translate((-center, 0.0));
        let brush = prepare(&gradient, output, &account).unwrap();
        assert!(is_pattern(&brush));
        assert_eq!(
            (output * brush.transform().unwrap()).as_coeffs(),
            [16.0, 0.0, 0.0, 16.0, 0.0, 0.0]
        );
        // Device x=1 maps back to logical center+1. The concentric distance
        // is therefore 1, independently of the large logical translation.
        assert_eq!(((center + 1.0) - center) / 16.0, 1.0 / 16.0);
        assert_eq!(((center - 1.0) / 16.0).fract(), 15.0 / 16.0);
    }

    #[test]
    fn radial_glyph_accepts_exact_cancellation_of_separately_narrowed_factors() {
        let center = f64::from(1_u32 << 28);
        let local = kurbo::Affine::new([16.0, 0.0, 0.0, 16.0, center, 0.0]);
        let output = kurbo::Affine::translate((-center, 0.0));
        let native_local = native_glyph_transform(
            local,
            output * local,
            false,
            output,
            PhysicalSize::new(2, 2),
            NativeGlyphField::Radial {
                radii: (0.0, 1.0),
                reference: RadialReference {
                    start: 0.0,
                    end: 1.0,
                    delta: 1.0,
                },
            },
            &Account::for_test(),
        )
        .unwrap();
        let native_device = vello_encoding::Transform::from_kurbo(&output) * native_local;
        // Both factors retain their exact power-of-two translations in f32;
        // the real glyph composition has the same centered 16-pixel period.
        assert_eq!(
            (output * local).as_coeffs(),
            [16.0, 0.0, 0.0, 16.0, 0.0, 0.0]
        );
        assert_eq!(native_device.matrix, [16.0, 0.0, 0.0, 16.0]);
        assert_eq!(native_device.translation, [0.0, 0.0]);
    }

    #[test]
    fn radial_glyph_refuses_phase_error_from_separately_narrowed_factors() {
        let center = f64::from(1_u32 << 28);
        let output = kurbo::Affine::translate((-center, 0.0));
        let gradient = RepeatingGradient::try_radial(
            Point::new(center + 1.0, 0.0),
            16.0,
            16.0,
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let account = Account::for_test();
        // Outline preparation narrows the complete product, retaining center 1.
        let brush = prepare(&gradient, output, &account).unwrap();
        assert!(is_pattern(&brush));
        let local = brush.transform().unwrap();
        let ideal_device = output * local;
        assert_eq!(ideal_device.as_coeffs(), [16.0, 0.0, 0.0, 16.0, 1.0, 0.0]);
        // Glyph factors narrow separately: center+1 rounds to center before
        // cancellation. At device (0,0), the ideal and actual concentric
        // distances consequently differ by 1/16 of a cycle, well over 1/511.
        assert_eq!((center + 1.0) as f32, center as f32);
        let native_device = vello_encoding::Transform::from_kurbo(&output)
            * vello_encoding::Transform::from_kurbo(&local);
        assert_eq!(native_device.translation, [0.0, 0.0]);
        let ideal_phase_at_origin = ideal_device.as_coeffs()[4] / 16.0;
        let native_phase_at_origin = f64::from(native_device.translation[0]) / 16.0;
        assert_eq!(ideal_phase_at_origin - native_phase_at_origin, 1.0 / 16.0);
        assert!(ideal_phase_at_origin - native_phase_at_origin > 1.0 / 511.0);
        let error = native_glyph_transform(
            local,
            ideal_device,
            false,
            output,
            PhysicalSize::new(2, 2),
            NativeGlyphField::Radial {
                radii: (0.0, 1.0),
                reference: RadialReference {
                    start: 0.0,
                    end: 1.0,
                    delta: 1.0,
                },
            },
            &account,
        )
        .expect_err("glyph qualification must compare the actual narrowed composition");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
    }

    #[test]
    fn prepared_radial_glyph_preserves_phase_refusal_and_hint_mapping() {
        use crate::{GradientHint, GradientRealizationError};

        let center = f64::from(1_u32 << 28);
        let output = kurbo::Affine::translate((-center, 0.0));
        let extent = PhysicalSize::new(2, 2);
        let account = Account::for_test();
        let first = stop(0.0, Color::BLACK);
        for (first, hinted) in [
            (first, false),
            (
                first.with_hint_after(GradientHint::try_new(0.25).unwrap()),
                true,
            ),
        ] {
            let gradient = RepeatingGradient::try_radial(
                Point::new(center + 1.0, 0.0),
                16.0,
                16.0,
                vec![first, stop(1.0, Color::TRANSPARENT)],
            )
            .unwrap();
            let brush =
                crate::encode::glyph_paint_brush(&gradient.into(), output, extent, &account)
                    .unwrap();
            // The exact translated center is 1. Separately narrowed factors
            // move it to 0, an error of 1/16 of a cycle at the origin.
            let ideal_center = (center + 1.0) - center;
            let native_center = f64::from((center + 1.0) as f32 - center as f32);
            let phase_loss = (ideal_center - native_center) / 16.0;
            assert_eq!(phase_loss, 1.0 / 16.0);
            assert!(phase_loss > 1.0 / 511.0);
            let error = brush
                .native_glyph_transform(output, extent, &account)
                .unwrap_err();
            if hinted {
                assert_eq!(error.code(), ErrorCode::GradientRealization);
                assert_eq!(
                    error.gradient_realization_diagnostic(),
                    Some(&GradientRealizationError::InsufficientNativeCapacity)
                );
            } else {
                assert_eq!(error.code(), ErrorCode::InvalidInput);
                let diagnostic = error.invalid_value_diagnostic().unwrap();
                assert_eq!(diagnostic.field(), "native gradient realization");
                assert_eq!(
                    diagnostic.value(),
                    "insufficient native circular phase capacity"
                );
            }
        }
    }

    #[test]
    fn prepared_repeating_radial_glyph_accepts_exact_cancellation_with_stored_radii() {
        use crate::{GradientHint, RenderClock, RenderProfile, RenderRefusalReason};
        use std::time::Duration;

        let center = f64::from(1_u32 << 28);
        let output = kurbo::Affine::translate((-center, 0.0));
        let extent = PhysicalSize::new(2, 2);
        let first = 0.75 + 2.0_f64.powi(-24) + 2.0_f64.powi(-27);
        let last = first + 1.0;
        let hint = GradientHint::try_new(first + 0.25).unwrap();
        let gradient = RepeatingGradient::try_radial(
            Point::new(center, 0.0),
            16.0,
            16.0,
            vec![
                stop(first, Color::BLACK).with_hint_after(hint),
                stop(last, Color::TRANSPARENT),
            ],
        )
        .unwrap();
        let account = Account::for_test();
        let brush =
            crate::encode::glyph_paint_brush(&gradient.into(), output, extent, &account).unwrap();
        let native = brush
            .native_glyph_transform(output, extent, &account)
            .unwrap()
            .unwrap();
        assert_eq!(native.matrix, [16.0, 0.0, 0.0, 16.0]);
        assert_eq!(native.translation, [268_435_456.0, 0.0]);
        let peniko::BrushRef::Gradient(gradient) = brush.as_ref() else {
            panic!("repeating gradient");
        };
        let peniko::GradientKind::Radial(radial) = gradient.kind else {
            panic!("radial field");
        };
        // Adding one to the rounded start lands on an even tie. The ideal
        // end is above that tie and would narrow to the next binary32 value.
        assert_eq!(radial.start_radius.to_bits(), 0x3f40_0001);
        assert_eq!(radial.end_radius.to_bits(), 0x3fe0_0000);
        assert_eq!((last as f32).to_bits(), 0x3fe0_0001);
        assert_eq!(gradient.extend, peniko::Extend::Repeat);

        // Each affine narrowing admits 18 visits. The native product admits
        // 20 more before the radial certificate's eight input captures.
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: true })
                .with_clock(RenderClock::Host(|| Duration::ZERO))
                .with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CoordinateVisits, 56)
                        .unwrap(),
                ),
        );
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                brush.native_glyph_transform(output, extent, account)
            })
            .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("native radial certificate input admission");
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::CoordinateVisits)
        );
        assert_eq!(refusal.current(), 56);
        assert_eq!(refusal.amount(), 8);
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 56 });
        let attempt = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(attempt.phase_count(RenderPhase::GradientPrepare), 1);
        assert_eq!(
            attempt.phase_elapsed(RenderPhase::GradientPrepare),
            Some(Duration::ZERO)
        );
        assert_eq!(
            attempt
                .phase_usage(RenderPhase::GradientPrepare)
                .work(RenderWork::CoordinateVisits),
            56
        );
    }

    #[test]
    fn radial_weighted_averages_precede_unused_native_pattern_capacity() {
        let account = Account::for_test();
        let expected: peniko::Color = color(1.0 / 3.0, 2.0 / 3.0, 0.0, 3.0 / 8.0).into();
        // The center cannot narrow to native coordinates. A positive period
        // of 4/16 pixel still selects the weighted integral, while selected
        // zero height does so even though the unused width*span overflows.
        assert!(!(f64::MAX as f32).is_finite());
        assert_eq!(4.0 * (1.0 / 16.0), 0.25);
        assert!(!(f64::MAX * 4.0).is_finite());
        for (width, height) in [(1.0 / 16.0, 1.0 / 16.0), (f64::MAX, 0.0)] {
            let gradient = RepeatingGradient::try_radial(
                Point::new(f64::MAX, -f64::MAX),
                width,
                height,
                weighted_stops(),
            )
            .unwrap();
            // Segment lengths 1,3 yield premul (1/8,1/4,0,3/8), rather
            // than the equal-spacing average (1/2,1/2,0,1/2).
            assert_eq!(
                solid(&prepare(&gradient, kurbo::Affine::IDENTITY, &account).unwrap()),
                expected
            );
        }
    }

    #[test]
    fn zero_height_and_unresolved_positive_cycles_keep_the_actual_weights() {
        let expected: peniko::Color = color(1.0 / 3.0, 2.0 / 3.0, 0.0, 3.0 / 8.0).into();
        let account = Account::for_test();
        for height in [0.0, 0.001] {
            let gradient =
                RepeatingGradient::try_radial(Point::new(0.0, 0.0), 10.0, height, weighted_stops())
                    .unwrap();
            assert_eq!(
                solid(&prepare(&gradient, kurbo::Affine::IDENTITY, &account).unwrap()),
                expected
            );
        }
        let huge_zero_height =
            RepeatingGradient::try_radial(Point::new(0.0, 0.0), f64::MAX, 0.0, weighted_stops())
                .unwrap();
        assert_eq!(
            solid(&prepare(&huge_zero_height, kurbo::Affine::IDENTITY, &account).unwrap()),
            expected
        );
        let gradient = linear(weighted_stops());
        assert_eq!(
            solid(&prepare(&gradient, kurbo::Affine::scale(0.1), &account).unwrap()),
            expected
        );
        assert!(
            RepeatingGradient::try_radial(Point::new(0.0, 0.0), 0.0, 1.0, weighted_stops())
                .is_err()
        );
    }

    #[test]
    fn real_period_construction_collapse_uses_uniform_original_colors() {
        let span = 1.0e-300;
        let axis = 1.0e-100;
        assert!(span > 0.0);
        assert_eq!(span * axis, 0.0); // Actual chosen f64 realization boundary.
        let gradient = RepeatingGradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(axis, 0.0),
            vec![
                stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
                stop(span * 0.25, color(0.0, 1.0, 0.0, 0.5)),
                stop(span, color(0.0, 0.0, 1.0, 0.0)),
            ],
        )
        .unwrap();
        // Uniform endpoint weights 1/4,1/2,1/4 give premul (1/4,1/4,0,1/2).
        assert_eq!(
            solid(&prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test()).unwrap()),
            peniko::Color::from(color(0.5, 0.5, 0.0, 0.5)),
        );
    }

    #[test]
    fn unrepresentable_nonzero_geometry_is_a_typed_diagnostic() {
        let gradient = RepeatingGradient::try_linear(
            Point::new(-f64::MAX, 0.0),
            Point::new(f64::MAX, 0.0),
            vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)],
        )
        .unwrap();
        let error = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test())
            .err()
            .expect("overflow must refuse");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
        assert!(
            prepare(
                &linear(vec![stop(0.0, Color::BLACK), stop(1.0, Color::TRANSPARENT)]),
                kurbo::Affine::scale(1.0e30),
                &Account::for_test(),
            )
            .is_err()
        );
        // Glyph paint inverts the composed native map, not either factor.
        assert!(
            native_glyph_transform(
                kurbo::Affine::scale(1.0e20),
                kurbo::Affine::scale(1.0e-19) * kurbo::Affine::scale(1.0e20),
                false,
                kurbo::Affine::scale(1.0e-19),
                PhysicalSize::new(2, 2),
                NativeGlyphField::Linear(LinearSampling::Direct),
                &Account::for_test(),
            )
            .is_ok()
        );
        assert!(
            native_glyph_transform(
                kurbo::Affine::scale(1.0e50),
                kurbo::Affine::scale(1.0e-49) * kurbo::Affine::scale(1.0e50),
                false,
                kurbo::Affine::scale(1.0e-49),
                PhysicalSize::new(2, 2),
                NativeGlyphField::Linear(LinearSampling::Direct),
                &Account::for_test(),
            )
            .is_err()
        );
    }

    #[test]
    fn repeating_average_keeps_its_exact_scalar_and_stop_prefixes() {
        use crate::RenderRefusalReason;

        let uniform = linear(vec![
            stop(7.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(7.0, color(1.0, 1.0, 1.0, 1.0)),
            stop(7.0, color(0.0, 0.0, 1.0, 1.0)),
        ]);
        let weighted =
            RepeatingGradient::try_radial(Point::new(0.0, 0.0), 1.0, 0.0, weighted_stops())
                .unwrap();
        let cases = [
            (
                uniform,
                // Four endpoint/span actions, two denominator actions,
                // three weighted colors at 20 each and 12 finish actions.
                [
                    (4, 2, Some(1)),
                    (5, 2, Some(1)),
                    (6, 3, Some(4)),
                    (78, 5, None),
                ],
                color(0.75, 0.5, 0.75, 1.0),
            ),
            (
                weighted,
                // Seven actions before reduction, 12 for the first color
                // and position, two segments at 27 each and 12 to finish.
                [
                    (7, 3, Some(4)),
                    (18, 3, Some(1)),
                    (19, 4, Some(4)),
                    (85, 5, None),
                ],
                color(1.0 / 3.0, 2.0 / 3.0, 0.0, 3.0 / 8.0),
            ),
        ];
        for (gradient, attempts, expected) in cases {
            for (limit, stop_visits, next_amount) in attempts {
                let options = RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CoordinateVisits, limit)
                        .unwrap(),
                );
                let mut context = RenderAccountingContext::new(options);
                let result =
                    context.run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        prepare(&gradient, kurbo::Affine::IDENTITY, account)
                    });
                let report = context.last_attempt().unwrap();
                assert_eq!(report.cleanup_fault(), None);
                let attempt = report.attempt().unwrap();
                if let Some(amount) = next_amount {
                    let error = result.err().expect("the next scalar admission must refuse");
                    let refusal = attempt.refusal().unwrap();
                    assert_eq!(refusal.phase(), RenderPhase::Attempt);
                    assert_eq!(
                        refusal.quantity(),
                        RenderQuantity::Work(RenderWork::CoordinateVisits)
                    );
                    assert_eq!((refusal.current(), refusal.amount()), (limit, amount));
                    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
                    assert_eq!(
                        error.accounting_diagnostic(),
                        Some(RenderAccountingError::Refused(refusal))
                    );
                } else {
                    assert_eq!(solid(&result.unwrap()), peniko::Color::from(expected));
                    assert_eq!(attempt.refusal(), None);
                }
                let usage = attempt.usage();
                assert_eq!(usage.work(RenderWork::CoordinateVisits), limit);
                assert_eq!(usage.work(RenderWork::GradientStopVisits), stop_visits);
                assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), 0);
                for resource in [
                    RenderResource::ScratchCpuBytes,
                    RenderResource::RetainedCpuBytes,
                ] {
                    assert_eq!(usage.live(resource), 0);
                    assert_eq!(usage.peak(resource), 0);
                }
            }
        }
    }

    #[test]
    fn average_preparation_charges_real_stop_walks_without_unused_stop_backing() {
        let gradient = linear(weighted_stops());
        let mut context = RenderAccountingContext::new(RenderAccountingOptions::new());
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                prepare(&gradient, kurbo::Affine::scale(0.1), account).map(drop)
            })
            .unwrap();
        let report = context.last_attempt().unwrap().attempt().unwrap();
        // Two period endpoints plus one complete scalar average traversal.
        assert_eq!(
            report
                .phase_usage(RenderPhase::GradientPrepare)
                .work(RenderWork::GradientStopVisits),
            5
        );
        assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::GradientStopVisits, 3)
                    .unwrap(),
            ),
        );
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                prepare(&gradient, kurbo::Affine::scale(0.1), account).map(drop)
            })
            .unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("expected stop-visit admission refusal");
        };
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::GradientStopVisits)
        );
        assert_eq!(refusal.current(), 3);
        assert_eq!(refusal.amount(), 1);
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }
    #[test]
    fn repeating_geometry_dispatch_preserves_the_first_accounting_refusal() {
        use crate::RenderRefusalReason;

        let stops = [stop(1.0, Color::BLACK), stop(5.0, Color::TRANSPARENT)];
        let cases = [
            (linear(stops.to_vec()), 1),
            (
                RepeatingGradient::try_radial(Point::new(0.0, 0.0), 1.0, 1.0, stops.to_vec())
                    .unwrap(),
                1,
            ),
            (
                RepeatingGradient::try_horizontal_distance(2.0, stops.to_vec()).unwrap(),
                4,
            ),
        ];
        for (gradient, geometry_amount) in cases {
            // Two endpoint reads and their difference precede the zero-span
            // comparison. Linear and radial next admit one remainder, while
            // horizontal distance admits four linear coefficients together.
            for (limit, amount) in [(3, 1), (4, geometry_amount)] {
                let options = RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CoordinateVisits, limit)
                        .unwrap(),
                );
                let mut context = RenderAccountingContext::new(options);
                let error = context
                    .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                        prepare(&gradient, kurbo::Affine::IDENTITY, account).map(drop)
                    })
                    .unwrap_err();
                let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
                else {
                    panic!("expected coordinate admission refusal for {gradient:?}");
                };
                assert_eq!(refusal.phase(), RenderPhase::Attempt);
                assert_eq!(
                    refusal.quantity(),
                    RenderQuantity::Work(RenderWork::CoordinateVisits)
                );
                assert_eq!((refusal.current(), refusal.amount()), (limit, amount));
                assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
                let report = context.last_attempt().unwrap();
                assert_eq!(report.cleanup_fault(), None);
                let attempt = report.attempt().unwrap();
                assert_eq!(attempt.refusal(), Some(refusal));
                let usage = attempt.usage();
                assert_eq!(usage.work(RenderWork::CoordinateVisits), limit);
                assert_eq!(usage.work(RenderWork::GradientStopVisits), 2);
                assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), 0);
                assert_eq!(usage.work(RenderWork::GpuObjectsCreated), 0);
                for resource in [
                    RenderResource::ScratchCpuBytes,
                    RenderResource::RetainedCpuBytes,
                    RenderResource::GpuBytes,
                ] {
                    assert_eq!(usage.live(resource), 0);
                    assert_eq!(usage.peak(resource), 0);
                }
            }
        }
    }

    #[test]
    fn distance_phase_actions_refuse_before_the_fifth_comparison_and_retry() {
        let account = Account::for_test();
        assert_eq!(horizontal_phase(1.0, 4.0, &account).unwrap(), 0.25);
        let mut baseline = RenderAccountingContext::new(RenderAccountingOptions::new());
        baseline
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| horizontal_phase(1.0, 4.0, account).map(drop),
            )
            .unwrap();
        // rem_euclid, division, narrowing, nonzero remainder, native-zero test.
        assert_eq!(
            baseline
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            5
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 4)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| horizontal_phase(1.0, 4.0, account).map(drop),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 4);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(horizontal_phase(1.0, 4.0, &account).unwrap(), 0.25);
        // The f64 quotient itself underflows; its source remainder still proves
        // a nonzero phase and must not silently turn the center into a seam.
        let error = horizontal_phase(f64::from_bits(1), 4.0, &account).unwrap_err();
        assert_eq!(
            error.gradient_realization_diagnostic(),
            Some(&crate::GradientRealizationError::InsufficientNativeCapacity)
        );
    }

    #[test]
    fn distance_repeat_keeps_its_origin_unshifted_and_its_phase_in_the_program() {
        let gradient = RepeatingGradient::try_horizontal_distance(
            8.0,
            vec![
                GradientStop::try_new(1.0, Color::BLACK).unwrap(),
                GradientStop::try_new(5.0, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test()).unwrap();
        let Some(PreparedSampling::Program(program)) = &brush.sampling else {
            panic!("programme")
        };
        assert!(matches!(program.coordinates,
            ProgramCoordinates::HorizontalDistanceRepeat { phase } if phase == 0.25));
        assert_eq!(
            program
                .records
                .iter()
                .map(|r| r.position)
                .collect::<Vec<_>>(),
            [0.0, 1.0]
        );
        let inverse = brush.transform.unwrap().reference_device.inverse();
        assert_eq!((inverse * kurbo::Point::new(8.0, 0.0)).x, 0.0);
        assert_eq!((inverse * kurbo::Point::new(4.0, 0.0)).x, -1.0);
    }

    #[test]
    fn distance_average_does_not_evaluate_an_unused_origin_or_phase() {
        let gradient = RepeatingGradient::try_horizontal_distance(
            f64::MAX,
            vec![
                GradientStop::try_new(f64::from_bits(1), Color::BLACK).unwrap(),
                GradientStop::try_new(1.0, Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let brush = prepare(&gradient, kurbo::Affine::IDENTITY, &Account::for_test()).unwrap();
        assert!(!is_pattern(&brush));
        assert!(brush.sampling.is_none());
    }
}
