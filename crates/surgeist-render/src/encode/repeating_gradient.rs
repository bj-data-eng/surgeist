// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
//
// Modified concentric circular-field setup correspondence is documented in
// ../../NOTICE-VELLO.md. Qualification and resolved paint policy are Render-owned.

//! Per-use realization of resolved repeating fields. This is scalar paint
//! preparation; the native raster engine still owns all pixel production.

use super::{BrushTransform, FrameBrush, gradient_with_stops};
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
        let (field, local) = match geometry {
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
        let mut brush_transform = BrushTransform {
            local,
            reference_device: device,
        };
        let native = match field {
            NativeField::Linear => {
                brush_transform = linear_brush_transform(paint_to_device, device, &account)?;
                account.work(RenderWork::CoordinateVisits, 6)?;
                let transform =
                    narrow_transform(paint_to_device * brush_transform.local, &account)?;
                if !check_native_linear(transform, &account)? {
                    return solid_average(stops, Average::Uniform, &account);
                }
                qualify_linear(
                    brush_transform.reference_device,
                    transform,
                    target_extent,
                    &account,
                )?;
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
        gradient_with_stops(native, normalized, Some(brush_transform))
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
    brush_transform: BrushTransform,
    paint_to_device: kurbo::Affine,
    radial_start: Option<f64>,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let local = narrow_transform(brush_transform.local, account)?;
    let shape = narrow_transform(paint_to_device, account)?;
    // GlyphRun's resolver composes already-narrowed transforms. Neither factor
    // is inverted independently for the paint, so check the actual product.
    account.work(RenderWork::CoordinateVisits, 6)?;
    let composed = shape * local;
    if let Some(start_radius) = radial_start {
        qualify_radial(
            brush_transform.reference_device,
            composed,
            start_radius,
            target_extent,
            account,
        )?;
    } else {
        if !check_native_linear(composed, account)? {
            return Err(invalid_geometry("rounded-zero native glyph period"));
        }
        qualify_linear(
            brush_transform.reference_device,
            composed,
            target_extent,
            account,
        )?;
    }
    Ok(local)
}

// Fixed certificates for vello_shaders 0.9.0 linear endpoint projection and
// concentric circular setup. Intervals describe permitted WGSL results, not
// one CPU evaluation. Preparation never visits target pixels.
const SHADER_EPSILON: f64 = f32::EPSILON as f64;
const SHADER_MIN_NORMAL: f64 = f32::MIN_POSITIVE as f64;
const SHADER_MAX_DIVISOR: f64 = f64::from_bits(1149_u64 << 52); // 2^126.

#[derive(Clone, Copy)]
struct ShaderScalar {
    lower: f64,
    upper: f64,
}

impl ShaderScalar {
    const ZERO: Self = Self::exact(0.0);
    const ONE: Self = Self::exact(1.0);

    const fn exact(value: f64) -> Self {
        Self {
            lower: value,
            upper: value,
        }
    }

    fn checked(lower: f64, upper: f64, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        if !lower.is_finite() || !upper.is_finite() || lower > upper {
            return Err(invalid_geometry(
                "unbounded native gradient arithmetic certificate",
            ));
        }
        Ok(Self { lower, upper })
    }

    fn magnitude(self, account: &Account) -> Result<f64> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        Ok(self.lower.abs().max(self.upper.abs()))
    }

    fn neg(self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        Self::checked(-self.upper, -self.lower, account)
    }

    fn abs(self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        let lower = if self.lower <= 0.0 && self.upper >= 0.0 {
            0.0
        } else {
            self.lower.abs().min(self.upper.abs())
        };
        Self::checked(lower, self.lower.abs().max(self.upper.abs()), account)
    }

    fn hull(self, other: Self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 4)?;
        Self::checked(
            self.lower.min(other.lower),
            self.upper.max(other.upper),
            account,
        )
    }

    // Include permitted input flush-to-zero. Exact zeros remain exact.
    fn flushed(self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        if self.lower < SHADER_MIN_NORMAL && self.upper > -SHADER_MIN_NORMAL {
            Self::checked(self.lower.min(0.0), self.upper.max(0.0), account)
        } else {
            Ok(self)
        }
    }

    fn real_add(self, other: Self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 4)?;
        Self::checked(
            (self.lower + other.lower).next_down(),
            (self.upper + other.upper).next_up(),
            account,
        )
    }

    fn real_mul(self, other: Self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 4)?;
        let products = [
            self.lower * other.lower,
            self.lower * other.upper,
            self.upper * other.lower,
            self.upper * other.upper,
        ];
        account.work(RenderWork::CoordinateVisits, 8)?;
        let lower = products
            .into_iter()
            .fold(f64::INFINITY, f64::min)
            .next_down();
        let upper = products
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max)
            .next_up();
        // Both traversals were admitted before inspecting the corner products.
        Self::checked(lower, upper, account)
    }

    fn real_div(self, other: Self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 4)?;
        if other.lower <= 0.0 && other.upper >= 0.0 {
            return Err(invalid_geometry("uncertain native gradient divisor"));
        }
        let quotients = [
            self.lower / other.lower,
            self.lower / other.upper,
            self.upper / other.lower,
            self.upper / other.upper,
        ];
        account.work(RenderWork::CoordinateVisits, 8)?;
        let lower = quotients
            .into_iter()
            .fold(f64::INFINITY, f64::min)
            .next_down();
        let upper = quotients
            .into_iter()
            .fold(f64::NEG_INFINITY, f64::max)
            .next_up();
        Self::checked(lower, upper, account)
    }

    fn real_sqrt(self, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 2)?;
        if self.lower < 0.0 {
            return Err(invalid_geometry("uncertain native gradient square root"));
        }
        Self::checked(
            self.lower.sqrt().next_down().max(0.0),
            self.upper.sqrt().next_up(),
            account,
        )
    }

    // e is a full binary32 epsilon: WGSL permits either rounding direction.
    // 2.5 ULP <= 2.5e*|exact result| for a normal result. One minimum-normal
    // absolute allowance also covers subnormal rounding and output FTZ.
    fn rounded(self, relative: f64, account: &Account) -> Result<Self> {
        let magnitude = self.magnitude(account)?;
        let error = ((relative * magnitude).next_up() + SHADER_MIN_NORMAL).next_up();
        let result = Self::checked(
            (self.lower - error).next_down(),
            (self.upper + error).next_up(),
            account,
        )?;
        account.work(RenderWork::CoordinateVisits, 2)?;
        if result.lower < -f64::from(f32::MAX) || result.upper > f64::from(f32::MAX) {
            return Err(invalid_geometry(
                "non-finite permitted gradient intermediate",
            ));
        }
        Ok(result)
    }

    fn add(self, other: Self, account: &Account) -> Result<Self> {
        self.flushed(account)?
            .real_add(other.flushed(account)?, account)?
            .rounded(SHADER_EPSILON, account)
    }

    fn mul(self, other: Self, account: &Account) -> Result<Self> {
        self.flushed(account)?
            .real_mul(other.flushed(account)?, account)?
            .rounded(SHADER_EPSILON, account)
    }

    fn div(self, other: Self, account: &Account) -> Result<Self> {
        let numerator = self.flushed(account)?;
        let denominator = other.flushed(account)?;
        let absolute = denominator.abs(account)?;
        account.work(RenderWork::CoordinateVisits, 2)?;
        if absolute.lower < SHADER_MIN_NORMAL || absolute.upper > SHADER_MAX_DIVISOR {
            return Err(invalid_geometry(
                "gradient divisor outside native accuracy domain",
            ));
        }
        numerator
            .real_div(denominator, account)?
            .rounded(2.5 * SHADER_EPSILON, account)
    }

    fn sqrt(self, account: &Account) -> Result<Self> {
        let value = self.flushed(account)?;
        account.work(RenderWork::CoordinateVisits, 2)?;
        if value.lower < SHADER_MIN_NORMAL {
            return Err(invalid_geometry("uncertifiable circular setup square root"));
        }
        // WGSL sqrt inherits 1/inverseSqrt. Enclose the 2-ULP
        // inverseSqrt first, then the separately rounded reciprocal.
        let root = value.real_sqrt(account)?;
        let inverse = Self::ONE
            .real_div(root, account)?
            .rounded(2.0 * SHADER_EPSILON, account)?;
        Self::ONE.div(inverse, account)
    }
}

// A three-term sum must cover either association, including fused evaluations.
fn circular_sum3(
    a: ShaderScalar,
    b: ShaderScalar,
    c: ShaderScalar,
    account: &Account,
) -> Result<ShaderScalar> {
    a.add(b, account)?
        .add(c, account)?
        .hull(a.add(b.add(c, account)?, account)?, account)?
        .hull(a.add(c, account)?.add(b, account)?, account)
}

// [a,b,c,d,x,y], with the same columns as vello_encoding::Transform.
fn circular_product(
    left: [ShaderScalar; 6],
    right: [ShaderScalar; 6],
    account: &Account,
) -> Result<[ShaderScalar; 6]> {
    let [a, b, c, d, x, y] = left;
    let [aa, ab, ac, ad, ax, ay] = right;
    Ok([
        a.mul(aa, account)?.add(c.mul(ab, account)?, account)?,
        b.mul(aa, account)?.add(d.mul(ab, account)?, account)?,
        a.mul(ac, account)?.add(c.mul(ad, account)?, account)?,
        b.mul(ac, account)?.add(d.mul(ad, account)?, account)?,
        circular_sum3(a.mul(ax, account)?, c.mul(ay, account)?, x, account)?,
        circular_sum3(b.mul(ax, account)?, d.mul(ay, account)?, y, account)?,
    ])
}

fn circular_inverse(transform: [ShaderScalar; 6], account: &Account) -> Result<[ShaderScalar; 6]> {
    let [a, b, c, d, x, y] = transform;
    let determinant = a
        .mul(d, account)?
        .add(b.mul(c, account)?.neg(account)?, account)?;
    let reciprocal = ShaderScalar::ONE.div(determinant, account)?;
    let inverse = [
        reciprocal.mul(d, account)?,
        reciprocal.mul(b.neg(account)?, account)?,
        reciprocal.mul(c.neg(account)?, account)?,
        reciprocal.mul(a, account)?,
    ];
    Ok([
        inverse[0],
        inverse[1],
        inverse[2],
        inverse[3],
        inverse[0]
            .mul(x, account)?
            .add(inverse[2].mul(y, account)?, account)?
            .neg(account)?,
        inverse[1]
            .mul(x, account)?
            .add(inverse[3].mul(y, account)?, account)?
            .neg(account)?,
    ])
}

fn circular_native_field(
    device: vello_encoding::Transform,
    start_radius: f64,
    account: &Account,
) -> Result<([ShaderScalar; 6], ShaderScalar)> {
    account.work(RenderWork::CoordinateVisits, 8)?;
    let start = start_radius as f32;
    let end = start + 1.0; // These are the actual CPU-encoded native radii.
    let native = [
        device.matrix[0],
        device.matrix[1],
        device.matrix[2],
        device.matrix[3],
        device.translation[0],
        device.translation[1],
    ];
    let inverse = circular_inverse(native.map(|v| ShaderScalar::exact(f64::from(v))), account)?;
    let start = ShaderScalar::exact(f64::from(start));
    let end = ShaderScalar::exact(f64::from(end));
    let epsilon = ShaderScalar::exact(1.0 / 4096.0);
    let difference = start.add(end.neg(account)?, account)?;
    let absolute_difference = difference.abs(account)?;
    account.work(RenderWork::CoordinateVisits, 3)?;
    if absolute_difference.lower <= epsilon.upper || end.lower <= 0.0 {
        return Err(invalid_geometry("uncertain native circular branch"));
    }
    let focal = start.div(difference, account)?;
    let one_minus_focal = ShaderScalar::ONE.add(focal.neg(account)?, account)?;
    let center = one_minus_focal.mul(epsilon, account)?;
    let squared_center = center.mul(center, account)?;
    let distance = squared_center.add(squared_center, account)?.sqrt(account)?;
    let radius = end.div(distance, account)?;
    let radius_difference = radius
        .add(ShaderScalar::ONE.neg(account)?, account)?
        .abs(account)?;
    account.work(RenderWork::CoordinateVisits, 3)?;
    if radius_difference.lower <= epsilon.upper
        || radius.lower <= 1.0
        || one_minus_focal.lower <= 0.0
    {
        return Err(invalid_geometry("uncertain native circular kind"));
    }
    let z = ShaderScalar::ZERO;
    let one = ShaderScalar::ONE;
    let minus_one = one.neg(account)?;
    let unit = [z, minus_one, one, z, z, z];
    let minus_center = center.neg(account)?;
    let points = [
        minus_center,
        center,
        minus_center,
        minus_center,
        center,
        center,
    ];
    let line = circular_product(unit, circular_inverse(points, account)?, account)?;
    let line = circular_product(line, inverse, account)?;
    let squared = radius.mul(radius, account)?.add(minus_one, account)?;
    let ratio = one_minus_focal.abs(account)?.div(squared, account)?;
    let scale_x = radius.mul(ratio, account)?;
    let scale_y = squared.abs(account)?.sqrt(account)?.mul(ratio, account)?;
    let field = circular_product([scale_x, z, z, scale_y, z, z], line, account)?;
    Ok((field, focal))
}

// Enclose the authored reference inverse with outward f64 arithmetic.
// Both field certificates use this same intended device map.
fn reference_inverse(device: kurbo::Affine, account: &Account) -> Result<[ShaderScalar; 6]> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d, x, y] = device.as_coeffs().map(ShaderScalar::exact);
    let determinant = a
        .real_mul(d, account)?
        .real_add(b.real_mul(c, account)?.neg(account)?, account)?;
    let reciprocal = ShaderScalar::ONE.real_div(determinant, account)?;
    let inverse = [
        reciprocal.real_mul(d, account)?,
        reciprocal.real_mul(b.neg(account)?, account)?,
        reciprocal.real_mul(c.neg(account)?, account)?,
        reciprocal.real_mul(a, account)?,
    ];
    let tx = inverse[0]
        .real_mul(x, account)?
        .real_add(inverse[2].real_mul(y, account)?, account)?
        .neg(account)?;
    let ty = inverse[1]
        .real_mul(x, account)?
        .real_add(inverse[3].real_mul(y, account)?, account)?
        .neg(account)?;
    Ok([inverse[0], inverse[1], inverse[2], inverse[3], tx, ty])
}

fn circular_ideal_field(device: kurbo::Affine, account: &Account) -> Result<[ShaderScalar; 6]> {
    let inverse = reference_inverse(device, account)?;
    let s = ShaderScalar::exact(0.5).real_sqrt(account)?;
    let minus_s = s.neg(account)?;
    let rotate = |x: ShaderScalar, y: ShaderScalar| -> Result<[ShaderScalar; 2]> {
        Ok([
            minus_s
                .real_mul(x, account)?
                .real_add(minus_s.real_mul(y, account)?, account)?,
            s.real_mul(x, account)?
                .real_add(minus_s.real_mul(y, account)?, account)?,
        ])
    };
    let first = rotate(inverse[0], inverse[1])?;
    let second = rotate(inverse[2], inverse[3])?;
    let translation = rotate(inverse[4], inverse[5])?;
    Ok([
        first[0],
        first[1],
        second[0],
        second[1],
        translation[0],
        translation[1],
    ])
}

fn target_coordinates(target_extent: PhysicalSize, account: &Account) -> Result<[ShaderScalar; 2]> {
    account.work(RenderWork::CoordinateVisits, 2)?;
    let width = target_extent.width();
    let height = target_extent.height();
    if width == 0 || height == 0 || width - 1 > 1 << 24 || height - 1 > 1 << 24 {
        return Err(invalid_geometry(
            "unrepresentable native target pixel coordinates",
        ));
    }
    account.work(RenderWork::CoordinateVisits, 2)?;
    let x = ShaderScalar::exact(f64::from(width - 1));
    let y = ShaderScalar::exact(f64::from(height - 1));
    Ok([x, y])
}

// draw_leaf.wgsl:137-149 applies the native map to canonical endpoints
// (0,0) and (1,0), then projects onto their device-space difference. With
// all six coefficients finite, the zero products are exact; only translation
// and first-column addition remain, including permitted input/output FTZ.
fn linear_native_field(
    native: vello_encoding::Transform,
    account: &Account,
) -> Result<[ShaderScalar; 3]> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    if !native
        .matrix
        .into_iter()
        .chain(native.translation)
        .all(f32::is_finite)
    {
        return Err(invalid_geometry("unrepresentable native linear transform"));
    }
    account.work(RenderWork::CoordinateVisits, 4)?;
    let a = ShaderScalar::exact(f64::from(native.matrix[0])).flushed(account)?;
    let b = ShaderScalar::exact(f64::from(native.matrix[1])).flushed(account)?;
    let x = ShaderScalar::exact(f64::from(native.translation[0])).flushed(account)?;
    let y = ShaderScalar::exact(f64::from(native.translation[1])).flushed(account)?;
    let dx = a.add(x, account)?.add(x.neg(account)?, account)?;
    let dy = b.add(y, account)?.add(y.neg(account)?, account)?;
    let squared = dx.mul(dx, account)?.add(dy.mul(dy, account)?, account)?;
    let reciprocal = ShaderScalar::ONE.div(squared, account)?;
    let nx = dx.mul(reciprocal, account)?;
    let ny = dy.mul(reciprocal, account)?;
    let offset = x
        .mul(nx, account)?
        .add(y.mul(ny, account)?, account)?
        .neg(account)?;
    Ok([
        nx.flushed(account)?,
        ny.flushed(account)?,
        offset.flushed(account)?,
    ])
}

fn linear_rectangle_bound(
    field: [ShaderScalar; 3],
    coordinates: [ShaderScalar; 2],
    account: &Account,
) -> Result<ShaderScalar> {
    field[0]
        .abs(account)?
        .real_mul(coordinates[0], account)?
        .real_add(
            field[1].abs(account)?.real_mul(coordinates[1], account)?,
            account,
        )?
        .real_add(field[2].abs(account)?, account)
}

fn qualify_linear(
    reference_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    let field = linear_native_field(native_device, account)?;
    let coordinates = target_coordinates(target_extent, account)?;
    let inverse = reference_inverse(reference_device, account)?;
    account.work(RenderWork::CoordinateVisits, 3)?;
    let expected = [inverse[0], inverse[2], inverse[4]];
    let mut difference = [ShaderScalar::ZERO; 3];
    for i in 0..3 {
        account.work(RenderWork::CoordinateVisits, 2)?;
        difference[i] = field[i].real_add(expected[i].neg(account)?, account)?;
    }
    let q = linear_rectangle_bound(field, coordinates, account)?;
    let setup_error = linear_rectangle_bound(difference, coordinates, account)?;
    let e = ShaderScalar::exact(SHADER_EPSILON);
    let gamma6 = ShaderScalar::exact(6.0).real_mul(e, account)?.real_div(
        ShaderScalar::ONE.real_add(ShaderScalar::exact(-6.0).real_mul(e, account)?, account)?,
        account,
    )?;
    // fine.wgsl:1227-1229 evaluates a four-pixel group's base d, then
    // d+nx*i. Across both expressions there are three products and three
    // additions. Since base_x+i is the sampled x, the absolute term bound
    // is still |nx|*X+|ny|*Y+|c|. gamma6 covers every allowed association;
    // 6m/(1-6e) covers the six output-FTZ losses, including near zero.
    let absolute = ShaderScalar::exact(6.0 * SHADER_MIN_NORMAL)
        .real_mul(ShaderScalar::ONE.real_add(gamma6, account)?, account)?;
    let fine_error = gamma6.real_mul(q, account)?.real_add(absolute, account)?;
    let phase_error = setup_error.real_add(fine_error, account)?;
    let phase_bound = q.real_add(fine_error, account)?;
    account.work(RenderWork::CoordinateVisits, 2)?;
    let quantum = (1.0_f64 / 511.0).next_down();
    if phase_error.upper > quantum || phase_ulp(phase_bound.upper) > quantum {
        return Err(invalid_geometry(
            "insufficient native linear phase capacity",
        ));
    }
    Ok(())
}

/// Scalar bounds over the actual integer target rectangle. Setup intervals
/// enclose permitted GPU coefficients; reverse triangle inequality then bounds
/// their field error independently of target pixel count.
fn qualify_radial(
    ideal_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    start_radius: f64,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<()> {
    let (mut field, focal) = circular_native_field(native_device, start_radius, account)?;
    // Fine arithmetic may flush a stored coefficient or focal input too.
    for value in &mut field {
        account.work(RenderWork::CoordinateVisits, 1)?;
        *value = value.flushed(account)?;
    }
    let focal = focal.flushed(account)?;
    let [x, y] = target_coordinates(target_extent, account)?;
    let ideal = circular_ideal_field(ideal_device, account)?;
    let bound = |values: [ShaderScalar; 6]| -> Result<[ShaderScalar; 2]> {
        let a = values[0]
            .abs(account)?
            .real_mul(x, account)?
            .real_add(values[2].abs(account)?.real_mul(y, account)?, account)?
            .real_add(values[4].abs(account)?, account)?;
        let b = values[1]
            .abs(account)?
            .real_mul(x, account)?
            .real_add(values[3].abs(account)?.real_mul(y, account)?, account)?
            .real_add(values[5].abs(account)?, account)?;
        Ok([a, b])
    };
    let norm = |values: [ShaderScalar; 2]| -> Result<ShaderScalar> {
        values[0]
            .real_mul(values[0], account)?
            .real_add(values[1].real_mul(values[1], account)?, account)?
            .abs(account)?
            .real_sqrt(account)
    };
    let q = norm(bound(field)?)?;
    let mut difference = [ShaderScalar::ZERO; 6];
    for i in 0..6 {
        account.work(RenderWork::CoordinateVisits, 2)?;
        difference[i] = field[i].real_add(ideal[i].neg(account)?, account)?;
    }
    let coefficient_error = norm(bound(difference)?)?;
    let e = ShaderScalar::exact(SHADER_EPSILON);
    // gamma4 covers every association of the two affine products plus xlat.
    let gamma4 = ShaderScalar::exact(4.0).real_mul(e, account)?.real_div(
        ShaderScalar::ONE.real_add(ShaderScalar::exact(-4.0).real_mul(e, account)?, account)?,
        account,
    )?;
    let affine_absolute = ShaderScalar::exact(2.0)
        .real_sqrt(account)?
        .real_mul(ShaderScalar::exact(4.0 * SHADER_MIN_NORMAL), account)?
        .real_mul(ShaderScalar::ONE.real_add(gamma4, account)?, account)?;
    let affine_error = gamma4
        .real_mul(q, account)?
        .real_add(affine_absolute, account)?;
    let radius = q.real_add(affine_error, account)?;
    let gamma3 = ShaderScalar::exact(3.0).real_mul(e, account)?.real_div(
        ShaderScalar::ONE.real_add(ShaderScalar::exact(-3.0).real_mul(e, account)?, account)?,
        account,
    )?;
    // For 0<=g<1, both sqrt(1+g)-1 and 1-sqrt(1-g) are <=g.
    // Thus 1+gamma3 bounds either direction of squared-length rounding.
    let squared_factor = ShaderScalar::ONE.real_add(gamma3, account)?;
    let sqrt_factor = ShaderScalar::ONE
        .real_add(ShaderScalar::exact(2.5).real_mul(e, account)?, account)?
        .real_div(
            ShaderScalar::ONE.real_add(ShaderScalar::exact(-2.0).real_mul(e, account)?, account)?,
            account,
        )?;
    let norm_factor = squared_factor.real_mul(sqrt_factor, account)?;
    // Depends on the checked native fine adaptation in vello_engine/shaders.rs:
    // circular sqrt evaluates max(xx+yy,m), then selects 0 when xx+yy<=m.
    // Three squared-length FTZ losses contribute D=3m/(1-3e). In the guarded
    // branch, r^2 <= (m+D)/(1-gamma3); the normal branch has smaller absolute
    // loss. This bounds zero without assuming sqrt(0) has specified accuracy.
    let squared_absolute = ShaderScalar::exact(3.0 * SHADER_MIN_NORMAL)
        .real_mul(ShaderScalar::ONE.real_add(gamma3, account)?, account)?;
    let norm_absolute = squared_absolute
        .real_add(ShaderScalar::exact(SHADER_MIN_NORMAL), account)?
        .real_div(
            ShaderScalar::ONE.real_add(gamma3.neg(account)?, account)?,
            account,
        )?
        .real_sqrt(account)?
        .real_mul(sqrt_factor, account)?;
    let norm_error = radius
        .real_mul(
            norm_factor.real_add(ShaderScalar::exact(-1.0), account)?,
            account,
        )?
        .real_add(norm_absolute, account)?;
    let native_norm = radius
        .real_mul(norm_factor, account)?
        .real_add(norm_absolute, account)?;
    let focal_magnitude = focal.abs(account)?;
    let pre_add = focal_magnitude.real_add(native_norm, account)?;
    let addition_error = e
        .real_mul(pre_add, account)?
        .real_add(ShaderScalar::exact(SHADER_MIN_NORMAL), account)?;
    let phase_bound = pre_add.real_add(addition_error, account)?;
    let focal_error = focal
        .real_add(ShaderScalar::exact(start_radius), account)?
        .abs(account)?;
    let phase_error = coefficient_error
        .real_add(focal_error, account)?
        .real_add(affine_error, account)?
        .real_add(norm_error, account)?
        .real_add(addition_error, account)?
        // Glyph brushes retain the original phase narrowed to binary32.
        .real_add(ShaderScalar::exact(SHADER_EPSILON), account)?;
    account.work(RenderWork::CoordinateVisits, 2)?;
    let quantum = (1.0_f64 / 511.0).next_down();
    if phase_error.upper > quantum || phase_ulp(phase_bound.upper) > quantum {
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
) -> Result<BrushTransform> {
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
    // Preserve the intended map before solving back into local coordinates.
    // A*local may lose a translation even in f64, so it cannot be the oracle.
    account.work(RenderWork::CoordinateVisits, 6)?;
    let reference_device = kurbo::Affine::new([
        vector[0], vector[1], -vector[1], vector[0], origin[0], origin[1],
    ]);
    check_finite(reference_device)?;
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
    Ok(BrushTransform {
        local,
        reference_device,
    })
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
    fn linear_reference_survives_phase_loss_in_the_f64_inverse_solve() {
        let account = Account::for_test();
        for (center, period, intended_origin, recomposed_origin) in [
            (2.0_f64.powi(53), 19.0, 9.0, 8.0),
            (2.0_f64.powi(57), 17.0, 15.0, 0.0),
        ] {
            let output = kurbo::Affine::translate((-center, 0.0));
            let original = kurbo::Affine::scale(period);
            let paired = linear_brush_transform(output, output * original, &account).unwrap();
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
            BrushTransform {
                local,
                reference_device,
            },
            output,
            None,
            PhysicalSize::new(32, 32),
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
        let local =
            native_glyph_transform(paired, output, None, PhysicalSize::new(32, 32), &account)
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
        let glyph = vello_encoding::Transform::from_kurbo(&transform)
            * native_glyph_transform(
                brush.transform.unwrap(),
                transform,
                None,
                PhysicalSize::new(2, 2),
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
        let usage = context.last_attempt().unwrap().usage();
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
            BrushTransform {
                local,
                reference_device: output * local,
            },
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
        let error = native_glyph_transform(
            BrushTransform {
                local,
                reference_device: ideal_device,
            },
            output,
            Some(0.0),
            PhysicalSize::new(2, 2),
            &account,
        )
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
                BrushTransform {
                    local: kurbo::Affine::scale(1.0e20),
                    reference_device: kurbo::Affine::scale(1.0e-19) * kurbo::Affine::scale(1.0e20),
                },
                kurbo::Affine::scale(1.0e-19),
                None,
                PhysicalSize::new(2, 2),
                &Account::for_test(),
            )
            .is_ok()
        );
        assert!(
            native_glyph_transform(
                BrushTransform {
                    local: kurbo::Affine::scale(1.0e50),
                    reference_device: kurbo::Affine::scale(1.0e-49) * kurbo::Affine::scale(1.0e50),
                },
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
