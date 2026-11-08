// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
//
// Modified concentric circular-field setup correspondence is documented in
// ../../NOTICE-VELLO.md. Qualification and resolved paint policy are Render-owned.

//! Per-use realization of resolved repeating fields. This is scalar paint
//! preparation; the native raster engine still owns all pixel production.

use super::{FrameBrush, gradient_with_stops};
use crate::{
    Color, Error, PhysicalSize, RepeatingGradient, RepeatingGradientStop, Result,
    accounting::{Account, RenderPhase, RenderWork},
    paint::RepeatingGradientGeometry,
};

pub(super) fn prepare(
    gradient: &RepeatingGradient,
    paint_to_device: kurbo::Affine,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<FrameBrush> {
    let account = account.at(RenderPhase::GradientPrepare);
    account.phase(RenderPhase::GradientPrepare, || {
        let stops = gradient.stops();
        if stops.len() == 1 {
            account.work(RenderWork::GradientStopVisits, 1)?;
            return Ok(FrameBrush::solid(stops[0].color().into()));
        }
        account.work(RenderWork::GradientStopVisits, 2)?;
        let first = stops[0].position();
        let span = stops[stops.len() - 1].position() - first;
        if span == 0.0 {
            return solid_average(stops, Average::Uniform, &account);
        }
        let geometry = period_geometry(gradient.geometry(), first, span, &account)?;
        let (field, mut local) = match geometry {
            PeriodGeometry::RoundedZero => {
                return solid_average(stops, Average::Uniform, &account);
            }
            PeriodGeometry::ZeroHeight => {
                return solid_average(stops, Average::Weighted(span), &account);
            }
            PeriodGeometry::Pattern { field, local } => (field, local),
        };
        account.work(RenderWork::CoordinateVisits, 6)?;
        let device = paint_to_device * local;
        check_finite(device)?;
        let period = match field {
            NativeField::Linear => linear_device_period(device),
            NativeField::Radial { .. } => radial_device_period(device),
        };
        if !period.is_finite() {
            return Err(invalid_geometry("non-finite device period"));
        }
        // Exactly two physical pixels is resolved. A positive but unresolved
        // cycle retains the original segment weights, including zero lengths.
        if period < 2.0 {
            return solid_average(stops, Average::Weighted(span), &account);
        }
        let native = match field {
            NativeField::Linear => {
                local = linear_brush_transform(paint_to_device, device, &account)?;
                account.work(RenderWork::CoordinateVisits, 6)?;
                let transform = narrow_transform(paint_to_device * local, &account)?;
                if !check_native_linear(transform, &account)? {
                    return solid_average(stops, Average::Uniform, &account);
                }
                peniko::Gradient::new_linear((0.0, 0.0), (1.0, 0.0))
            }
            NativeField::Radial { start_radius } => {
                let native = narrow_transform(device, &account)?;
                qualify_radial(device, native, start_radius, target_extent, &account)?;
                let start_radius = start_radius as f32;
                peniko::Gradient::new_two_point_radial(
                    (0.0, 0.0),
                    start_radius,
                    (0.0, 0.0),
                    start_radius + 1.0,
                )
            }
        }
        .with_extend(peniko::Extend::Repeat);
        let mut normalized = account.temporary(stops.len())?;
        for stop in stops {
            account.work(RenderWork::GradientStopVisits, 1)?;
            account.work(RenderWork::CoordinateVisits, 1)?;
            normalized.push(peniko::ColorStop::from((
                ((stop.position() - first) / span) as f32,
                peniko::Color::from(stop.color()),
            )))?;
        }
        gradient_with_stops(native, normalized, Some(local))
    })
}

fn invalid_geometry(value: &'static str) -> Error {
    Error::invalid_value(
        "repeating gradient realization",
        value,
        "must have finite, representable native period geometry",
    )
}

fn check_finite(transform: kurbo::Affine) -> Result<()> {
    if transform.as_coeffs().iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(invalid_geometry("non-finite affine geometry"))
    }
}

/// Check the real native narrowing and inverse arithmetic. Large finite f64
/// geometry must not silently become native infinities or a singular transform.
/// Unit native gradient geometry keeps tiny logical radii out of Vello's fixed
/// radial-radius epsilon; only the complete map is narrowed on outline paths.
#[cfg(test)]
pub(super) fn native_transform(
    transform: kurbo::Affine,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let native = narrow_transform(transform, account)?;
    native_inverse(native, account)?;
    Ok(native)
}

fn native_inverse(
    native: vello_encoding::Transform,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d] = native.matrix;
    let [x, y] = native.translation;
    let determinant = a * d - b * c;
    let inverse_determinant = 1.0 / determinant;
    let inverse = [
        inverse_determinant * d,
        inverse_determinant * -b,
        inverse_determinant * -c,
        inverse_determinant * a,
    ];
    let inverse_translation = [
        -(inverse[0] * x + inverse[2] * y),
        -(inverse[1] * x + inverse[3] * y),
    ];
    if ![a, b, c, d, x, y, determinant, inverse_determinant]
        .into_iter()
        .chain(inverse)
        .chain(inverse_translation)
        .all(f32::is_finite)
        || determinant == 0.0
    {
        return Err(invalid_geometry("unrepresentable native affine inverse"));
    }
    Ok(vello_encoding::Transform {
        matrix: inverse,
        translation: inverse_translation,
    })
}

fn narrow_transform(
    transform: kurbo::Affine,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    check_finite(transform)?;
    let native = vello_encoding::Transform::from_kurbo(&transform);
    if native
        .matrix
        .into_iter()
        .chain(native.translation)
        .all(f32::is_finite)
    {
        Ok(native)
    } else {
        Err(invalid_geometry(
            "unrepresentable native affine coefficients",
        ))
    }
}

pub(super) fn native_glyph_transform(
    local: kurbo::Affine,
    paint_to_device: kurbo::Affine,
    radial_start: Option<f64>,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let ideal_device = paint_to_device * local;
    let local = narrow_transform(local, account)?;
    let shape = narrow_transform(paint_to_device, account)?;
    // GlyphRun's resolver composes already-narrowed transforms. Neither factor
    // is inverted independently for the paint, so check the actual product.
    account.work(RenderWork::CoordinateVisits, 6)?;
    let composed = shape * local;
    if let Some(start_radius) = radial_start {
        qualify_radial(ideal_device, composed, start_radius, target_extent, account)?;
    } else if !check_native_linear(composed, account)? {
        return Err(invalid_geometry("rounded-zero native glyph period"));
    }
    Ok(local)
}

// The concentric specialization below follows vello_shaders 0.9.0
// draw_leaf.wgsl:154-222. Only its circular branch is needed: our canonical
// centers coincide and the two native radii differ by approximately one.
fn circular_native_field(
    device: vello_encoding::Transform,
    start_radius: f64,
    account: &Account,
) -> Result<(vello_encoding::Transform, f64)> {
    let inverse = native_inverse(device, account)?;
    account.work(RenderWork::CoordinateVisits, 10)?;
    let start = start_radius as f32;
    let end = start + 1.0;
    let focal = start / (start - end);
    let epsilon = 1.0 / (1_u32 << 12) as f32;
    let center = (1.0 - focal) * epsilon;
    let radius = end / (center * center + center * center).sqrt();
    let unit = vello_encoding::Transform {
        matrix: [0.0, -1.0, 1.0, 0.0],
        translation: [0.0, 0.0],
    };
    let points = vello_encoding::Transform {
        matrix: [-center, center, -center, -center],
        translation: [center, center],
    };
    let line = native_product(unit, native_inverse(points, account)?, account)?;
    let line = native_product(line, inverse, account)?;
    let squared = radius * radius - 1.0;
    let ratio = (1.0 - focal).abs() / squared;
    let scale_x = radius * ratio;
    let scale_y = squared.abs().sqrt() * ratio;
    if ![focal, radius, squared, ratio, scale_x, scale_y]
        .into_iter()
        .all(f32::is_finite)
        || radius <= 1.0
    {
        return Err(invalid_geometry("unrepresentable native circular setup"));
    }
    let field = native_product(
        vello_encoding::Transform {
            matrix: [scale_x, 0.0, 0.0, scale_y],
            translation: [0.0, 0.0],
        },
        line,
        account,
    )?;
    Ok((field, f64::from(focal)))
}

fn native_product(
    left: vello_encoding::Transform,
    right: vello_encoding::Transform,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let product = left * right;
    if product
        .matrix
        .into_iter()
        .chain(product.translation)
        .all(f32::is_finite)
    {
        Ok(product)
    } else {
        Err(invalid_geometry("unrepresentable native circular field"))
    }
}

/// Conservative bounds over actual target integer pixel coordinates. The
/// reverse triangle inequality bounds the effect of native field coefficients
/// versus the ideal rotated field. Standard gamma_n bounds cover binary32
/// affine, squared-length, square-root and final focal-addition rounding.
/// Both that error and the final phase spacing must fit one existing 512-entry
/// ramp interval. This is an explicit native capacity limit, not CSS semantics.
fn qualify_radial(
    ideal_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    start_radius: f64,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    let (field, focal) = circular_native_field(native_device, start_radius, account)?;
    account.work(RenderWork::CoordinateVisits, 24)?;
    let width = target_extent.width();
    let height = target_extent.height();
    // Every used integer coordinate must be exactly representable in binary32.
    if width == 0 || height == 0 || width - 1 > 1 << 24 || height - 1 > 1 << 24 {
        return Err(invalid_geometry(
            "unrepresentable native target pixel coordinates",
        ));
    }
    let x = f64::from(width - 1);
    let y = f64::from(height - 1);
    let inverse = ideal_device.inverse();
    check_finite(inverse)?;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let ideal = kurbo::Affine::new([-s, s, -s, -s, 0.0, 0.0]) * inverse;
    check_finite(ideal)?;
    let actual = field.to_kurbo().as_coeffs();
    let expected = ideal.as_coeffs();
    let bound = |values: [f64; 6]| {
        [
            values[0].abs() * x + values[2].abs() * y + values[4].abs(),
            values[1].abs() * x + values[3].abs() * y + values[5].abs(),
        ]
    };
    let [qx, qy] = bound(actual);
    let difference = std::array::from_fn(|i| actual[i] - expected[i]);
    let [dx, dy] = bound(difference);
    let unit_roundoff = f64::from(f32::EPSILON) * 0.5;
    let gamma = |operations: f64| operations * unit_roundoff / (1.0 - operations * unit_roundoff);
    let affine_error = gamma(4.0) * qx.hypot(qy);
    let radius = qx.hypot(qy) + affine_error;
    let norm_factor = (1.0 + gamma(3.0)).sqrt() * (1.0 + unit_roundoff);
    let norm_error = radius * (norm_factor - 1.0);
    let phase_bound = (focal.abs() + radius * norm_factor) * (1.0 + unit_roundoff);
    let phase_error = dx.hypot(dy)
        + (focal + start_radius).abs()
        + affine_error
        + norm_error
        + unit_roundoff * (focal.abs() + radius * norm_factor)
        // Glyph brushes retain native radii; their original f64 phase narrowing
        // is bounded by one binary32 unit roundoff over the canonical [0,1].
        + unit_roundoff;
    let quantum = 1.0 / 511.0;
    if !phase_bound.is_finite()
        || !phase_error.is_finite()
        || phase_error > quantum
        || phase_ulp(phase_bound) > quantum
    {
        return Err(invalid_geometry(
            "insufficient native circular phase capacity",
        ));
    }
    Ok(())
}

fn phase_ulp(bound: f64) -> f64 {
    let mut upper = bound as f32;
    if f64::from(upper) < bound {
        upper = upper.next_up();
    }
    f64::from(upper.next_up() - upper)
}

#[derive(Clone, Copy)]
enum NativeField {
    Linear,
    Radial { start_radius: f64 },
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
    geometry: RepeatingGradientGeometry,
    first: f64,
    span: f64,
    account: &Account,
) -> Result<PeriodGeometry> {
    let phase = first.rem_euclid(span);
    let (field, local) = match geometry {
        RepeatingGradientGeometry::Linear { start, end } => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            let dx = end.x() - start.x();
            let dy = end.y() - start.y();
            let x = dx * span;
            let y = dy * span;
            if ![dx, dy, x, y].into_iter().all(f64::is_finite) {
                return Err(invalid_geometry("non-finite linear period"));
            }
            // These are actual f64 construction/narrowing facts, not a size
            // epsilon or the device-resolution decision above.
            if x == 0.0 && y == 0.0 {
                return Ok(PeriodGeometry::RoundedZero);
            }
            (
                NativeField::Linear,
                kurbo::Affine::new([x, y, -y, x, start.x() + dx * phase, start.y() + dy * phase]),
            )
        }
        RepeatingGradientGeometry::Radial {
            center,
            radius_x,
            radius_y,
        } => {
            account.work(RenderWork::CoordinateVisits, 4)?;
            // This selected semantic case needs no native ellipse backing or
            // period-axis arithmetic, even when that unused product overflows.
            if radius_y == 0.0 {
                return Ok(PeriodGeometry::ZeroHeight);
            }
            let x = radius_x * span;
            let y = radius_y * span;
            if ![x, y].into_iter().all(f64::is_finite) {
                return Err(invalid_geometry("non-finite radial period"));
            }
            if x == 0.0 {
                return Ok(PeriodGeometry::RoundedZero);
            }
            // A positive height that underflows in period-axis construction is
            // an unresolved direction, not a rounded-zero *whole* period.
            if y == 0.0 {
                return Ok(PeriodGeometry::ZeroHeight);
            }
            let radius = phase / span;
            (
                NativeField::Radial {
                    start_radius: radius,
                },
                kurbo::Affine::new([x, 0.0, 0.0, y, center.x(), center.y()]),
            )
        }
    };
    check_finite(local)?;
    Ok(PeriodGeometry::Pattern { field, local })
}

/// Vello's linear shader transforms its two endpoints and then projects onto
/// their device-space difference. It does not inverse-transform the field
/// normal. Choose equivalent perpendicular device endpoints explicitly, then
/// express that device map in the caller's local paint coordinates.
fn linear_brush_transform(
    paint_to_device: kurbo::Affine,
    period_to_device: kurbo::Affine,
    account: &Account,
) -> Result<kurbo::Affine> {
    account.work(RenderWork::CoordinateVisits, 12)?;
    let [a, b, c, d, x, y] = period_to_device.as_coeffs();
    let transverse_scale = c.abs().max(d.abs());
    let c = c / transverse_scale;
    let d = d / transverse_scale;
    let transverse_length = c.hypot(d);
    let normal = [d / transverse_length, -c / transverse_length];
    let signed_period = a * normal[0] + b * normal[1];
    let period = signed_period.abs();
    if period == 0.0 || !period.is_finite() {
        return Err(invalid_geometry("unrepresentable effective linear period"));
    }
    let direction = [
        normal[0] * signed_period.signum(),
        normal[1] * signed_period.signum(),
    ];
    let vector = [normal[0] * signed_period, normal[1] * signed_period];
    // Perpendicular origin displacement is immaterial. Subtract whole cycles
    // along the normal as well, retaining phase without large absolute native
    // endpoints that could unnecessarily round to the same device point.
    let phase = (x * direction[0] + y * direction[1]).rem_euclid(period);
    let origin = [direction[0] * phase, direction[1] * phase];
    let [aa, ab, ac, ad, ax, ay] = paint_to_device.as_coeffs();
    let first_scale = aa.abs().max(ab.abs());
    let second_scale = ac.abs().max(ad.abs());
    let [aa, ab, ac, ad] = [
        aa / first_scale,
        ab / first_scale,
        ac / second_scale,
        ad / second_scale,
    ];
    let determinant = aa * ad - ab * ac;
    if determinant == 0.0 || !determinant.is_finite() {
        return Err(invalid_geometry("unrepresentable paint-to-device inverse"));
    }
    let solve = |x, y| {
        [
            ((ad * x - ac * y) / determinant) / first_scale,
            ((aa * y - ab * x) / determinant) / second_scale,
        ]
    };
    let first = solve(vector[0], vector[1]);
    let second = solve(-vector[1], vector[0]);
    let origin = solve(origin[0] - ax, origin[1] - ay);
    let local = kurbo::Affine::new([
        first[0], first[1], second[0], second[1], origin[0], origin[1],
    ]);
    check_finite(local)?;
    Ok(local)
}

/// Mirror the native linear shader's endpoint transformation, subtraction and
/// projection arithmetic. A true endpoint collapse chooses uniform replacement;
/// other non-finite field coefficients are an explicit geometry refusal.
fn check_native_linear(transform: vello_encoding::Transform, account: &Account) -> Result<bool> {
    account.work(RenderWork::CoordinateVisits, 7)?;
    let [a, b, c, d] = transform.matrix;
    let [x, y] = transform.translation;
    if ![a, b, c, d, x, y].into_iter().all(f32::is_finite) {
        return Err(invalid_geometry("unrepresentable native linear transform"));
    }
    let dx = (x + a) - x;
    let dy = (y + b) - y;
    if dx == 0.0 && dy == 0.0 {
        return Ok(false);
    }
    let squared = dx * dx + dy * dy;
    let inverse = 1.0 / squared;
    let nx = dx * inverse;
    let ny = dy * inverse;
    let offset = -(x * nx + y * ny);
    if ![dx, dy, squared, inverse, nx, ny, offset]
        .into_iter()
        .all(f32::is_finite)
    {
        return Err(invalid_geometry("unrepresentable native linear projection"));
    }
    Ok(true)
}

/// The first canonical coordinate is the field phase. Its stripe spacing is
/// 1 / |M^-T (1,0)| = |det(M)| / |M (0,1)|, rather than |M (1,0)|.
fn linear_device_period(transform: kurbo::Affine) -> f64 {
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let scale = c.abs().max(d.abs());
    if scale == 0.0 {
        return 0.0;
    }
    let c = c / scale;
    let d = d / scale;
    let transverse = c.hypot(d);
    (a * (d / transverse) - b * (c / transverse)).abs()
}

/// Stable two-dimensional smallest singular value: |det(M)| / sigma_max.
/// Normalize first to avoid squaring the original large or tiny axes.
fn radial_device_period(transform: kurbo::Affine) -> f64 {
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let scale = a.abs().max(b.abs()).max(c.abs()).max(d.abs());
    if scale == 0.0 {
        return 0.0;
    }
    let [a, b, c, d] = [a / scale, b / scale, c / scale, d / scale];
    let sum = a * a + b * b + c * c + d * d;
    let determinant = (a * d - b * c).abs();
    let discriminant = (sum * sum - 4.0 * determinant * determinant).max(0.0);
    let largest = ((sum + discriminant.sqrt()) * 0.5).sqrt();
    (determinant / largest) * scale
}

#[derive(Clone, Copy)]
enum Average {
    Uniform,
    Weighted(f64),
}

fn solid_average(
    stops: &[RepeatingGradientStop],
    rule: Average,
    account: &Account,
) -> Result<FrameBrush> {
    Ok(FrameBrush::solid(average(stops, rule, account)?.into()))
}

fn premultiplied(color: Color) -> [f64; 4] {
    let alpha = f64::from(color.a());
    [
        f64::from(color.r()) * alpha,
        f64::from(color.g()) * alpha,
        f64::from(color.b()) * alpha,
        alpha,
    ]
}

fn average(stops: &[RepeatingGradientStop], rule: Average, account: &Account) -> Result<Color> {
    let mut total = [0.0; 4];
    let mut previous = None;
    for (index, stop) in stops.iter().enumerate() {
        account.work(RenderWork::GradientStopVisits, 1)?;
        let color = premultiplied(stop.color());
        match rule {
            Average::Uniform => {
                let weight = if index == 0 || index + 1 == stops.len() {
                    0.5
                } else {
                    1.0
                } / (stops.len() - 1) as f64;
                for (total, component) in total.iter_mut().zip(color) {
                    *total += component * weight;
                }
            }
            Average::Weighted(span) => {
                account.work(RenderWork::CoordinateVisits, 1)?;
                let position = stop.position();
                if let Some((previous_position, previous_color)) = previous {
                    let weight = ((position - previous_position) / span) * 0.5;
                    for ((total, previous), current) in
                        total.iter_mut().zip(previous_color).zip(color)
                    {
                        *total += (previous + current) * weight;
                    }
                }
                previous = Some((position, color));
            }
        }
    }
    let [r, g, b, alpha] = total;
    if alpha == 0.0 {
        return Ok(Color::TRANSPARENT);
    }
    // Floating summation can exceed one by its final rounding unit. This
    // bounds only the final color conversion, never positions or geometry.
    Color::try_rgba(
        (r / alpha).clamp(0.0, 1.0) as f32,
        (g / alpha).clamp(0.0, 1.0) as f32,
        (b / alpha).clamp(0.0, 1.0) as f32,
        alpha.clamp(0.0, 1.0) as f32,
    )
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

    fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color::try_rgba(r, g, b, a).unwrap()
    }

    fn stop(position: f64, color: Color) -> RepeatingGradientStop {
        RepeatingGradientStop::try_new(position, color).unwrap()
    }

    fn weighted_stops() -> Vec<RepeatingGradientStop> {
        vec![
            stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(1.0, color(0.0, 1.0, 0.0, 0.5)),
            stop(4.0, color(0.0, 0.0, 1.0, 0.0)),
        ]
    }

    fn linear(stops: Vec<RepeatingGradientStop>) -> RepeatingGradient {
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

    #[test]
    fn averages_use_segment_integrals_and_premultiplied_srgba() {
        // Segment lengths 1,3 give endpoint weights 1/8,1/2,3/8.
        // Premultiplied sum is (1/8,1/4,0,3/8), with no transparent blue bleed.
        assert_color(
            average(
                &weighted_stops(),
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
            average(&stops, Average::Weighted(4.0), &Account::for_test()).unwrap(),
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
            average(&stops, Average::Uniform, &Account::for_test()).unwrap(),
            [0.75, 0.5, 0.75, 1.0],
        );
        let four = [
            stop(0.0, color(1.0, 0.0, 0.0, 1.0)),
            stop(0.0, color(0.0, 1.0, 0.0, 1.0)),
            stop(0.0, color(0.0, 0.0, 1.0, 1.0)),
            stop(0.0, color(1.0, 1.0, 1.0, 1.0)),
        ];
        assert_color(
            average(&four, Average::Uniform, &Account::for_test()).unwrap(),
            [1.0 / 3.0, 0.5, 0.5, 1.0],
        );
        let transparent = [
            stop(0.0, color(1.0, 0.0, 0.0, 0.0)),
            stop(1.0, color(0.0, 0.0, 1.0, 0.0)),
        ];
        assert_eq!(
            average(&transparent, Average::Weighted(1.0), &Account::for_test()).unwrap(),
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
        let glyph = vello_encoding::Transform::from_kurbo(&transform)
            * native_glyph_transform(local, transform, None, PhysicalSize::new(2, 2), &account)
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
            output,
            Some(0.0),
            PhysicalSize::new(2, 2),
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
        let error =
            native_glyph_transform(local, output, Some(0.0), PhysicalSize::new(2, 2), &account)
                .expect_err("glyph qualification must compare the actual narrowed composition");
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
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
        assert!(native_transform(kurbo::Affine::scale(1.0e30), &Account::for_test()).is_err());
        // Glyph paint inverts the composed native map, not either factor.
        assert!(
            native_glyph_transform(
                kurbo::Affine::scale(1.0e20),
                kurbo::Affine::scale(1.0e-19),
                None,
                PhysicalSize::new(2, 2),
                &Account::for_test(),
            )
            .is_ok()
        );
        assert!(
            native_glyph_transform(
                kurbo::Affine::scale(1.0e50),
                kurbo::Affine::scale(1.0e-49),
                None,
                PhysicalSize::new(2, 2),
                &Account::for_test(),
            )
            .is_err()
        );
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
        let report = context.last_attempt().unwrap();
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
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }
}
