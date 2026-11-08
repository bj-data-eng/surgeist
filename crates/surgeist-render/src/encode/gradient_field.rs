// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// Modified native field correspondence is documented in ../../NOTICE-VELLO.md.
//! Shared native scalar-field geometry and permitted-WGSL arithmetic certificates.
use super::BrushTransform;
use crate::{
    Error, PhysicalSize, Result,
    accounting::{Account, RenderWork},
};

fn invalid_geometry(value: &'static str) -> Error {
    Error::invalid_value(
        "native gradient realization",
        value,
        "must have finite, representable native field geometry",
    )
}

pub(super) fn check_finite(transform: kurbo::Affine) -> Result<()> {
    if transform.as_coeffs().iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(invalid_geometry("non-finite affine geometry"))
    }
}

pub(super) fn narrow_transform(
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

#[derive(Clone, Copy)]
pub(super) struct RadialReference {
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) delta: f64,
}

pub(super) fn native_glyph_transform(
    brush_transform: BrushTransform,
    paint_to_device: kurbo::Affine,
    radial_radii: Option<(f32, f32)>,
    target_extent: PhysicalSize,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let local = narrow_transform(brush_transform.local, account)?;
    let shape = narrow_transform(paint_to_device, account)?;
    // GlyphRun's resolver composes already-narrowed transforms. Neither factor
    // is inverted independently for the paint, so check the actual product.
    account.work(RenderWork::CoordinateVisits, 6)?;
    let composed = shape * local;
    if let Some(radii) = radial_radii {
        qualify_radial(
            brush_transform.reference_device,
            composed,
            radii,
            brush_transform
                .radial
                .expect("radial reference accompanies native radii"),
            target_extent,
            brush_transform.comparison_only,
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
            brush_transform.comparison_only,
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
pub(super) struct ShaderScalar {
    pub(super) lower: f64,
    pub(super) upper: f64,
}

impl ShaderScalar {
    pub(super) const ZERO: Self = Self::exact(0.0);
    pub(super) const ONE: Self = Self::exact(1.0);

    pub(super) const fn exact(value: f64) -> Self {
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

    pub(super) fn div(self, other: Self, account: &Account) -> Result<Self> {
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

    pub(super) fn sqrt(self, account: &Account) -> Result<Self> {
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
    radii: (f32, f32),
    account: &Account,
) -> Result<([ShaderScalar; 6], ShaderScalar)> {
    account.work(RenderWork::CoordinateVisits, 8)?;
    let (start, end) = radii; // Actual CPU-encoded radii, not an assumed unit delta.
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

pub(super) fn qualify_linear(
    reference_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    target_extent: PhysicalSize,
    comparison_only: bool,
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
    if !comparison_only && (phase_error.upper > quantum || phase_ulp(phase_bound.upper) > quantum) {
        return Err(invalid_geometry(
            "insufficient native linear phase capacity",
        ));
    }
    Ok(())
}

/// Scalar bounds over the actual integer target rectangle. Setup intervals
/// enclose permitted GPU coefficients; reverse triangle inequality then bounds
/// their field error independently of target pixel count.
pub(super) fn qualify_radial(
    ideal_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    radii: (f32, f32),
    reference: RadialReference,
    target_extent: PhysicalSize,
    comparison_only: bool,
    account: &Account,
) -> Result<()> {
    let (mut field, focal) = circular_native_field(native_device, radii, account)?;
    // Fine arithmetic may flush a stored coefficient or focal input too.
    for value in &mut field {
        account.work(RenderWork::CoordinateVisits, 1)?;
        *value = value.flushed(account)?;
    }
    let focal = focal.flushed(account)?;
    let [x, y] = target_coordinates(target_extent, account)?;
    let mut ideal = circular_ideal_field(ideal_device, account)?;
    // The native phase is focal + q/delta, including nonunit and negative starts.
    for coefficient in &mut ideal {
        *coefficient = coefficient.real_div(ShaderScalar::exact(reference.delta), account)?;
    }
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
        .real_add(
            ShaderScalar::exact(reference.start / reference.delta),
            account,
        )?
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
    if !comparison_only && (phase_error.upper > quantum || phase_ulp(phase_bound.upper) > quantum) {
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

/// Vello's linear shader transforms its two endpoints and then projects onto
/// their device-space difference. It does not inverse-transform the field
/// normal. Choose equivalent perpendicular device endpoints explicitly, then
/// express that device map in the caller's local paint coordinates.
pub(super) fn linear_brush_transform(
    paint_to_device: kurbo::Affine,
    period_to_device: kurbo::Affine,
    repeat: bool,
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
    let projected = x * direction[0] + y * direction[1];
    let phase = if repeat {
        projected.rem_euclid(period)
    } else {
        projected
    };
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
        radial: None,
        comparison_only: false,
    })
}

/// Mirror the native linear shader's endpoint transformation, subtraction and
/// projection arithmetic. A true endpoint collapse chooses uniform replacement;
/// other non-finite field coefficients are an explicit geometry refusal.
pub(super) fn check_native_linear(
    transform: vello_encoding::Transform,
    account: &Account,
) -> Result<bool> {
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

#[cfg(test)]
mod accounting_tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderPhase, RenderResource};
    use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};

    #[test]
    fn identity_native_projection_accounts_each_executed_scalar_action() {
        let mut context = RenderAccountingContext::default();
        assert!(
            context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::GradientPrepare,
                    |account| check_native_linear(vello_encoding::Transform::IDENTITY, account)
                )
                .unwrap()
        );
        // Six coefficient captures, six initial finite checks, four endpoint
        // operations, one executed zero comparison, three squared-length
        // operations, one reciprocal, two normal products, four offset
        // operations, and seven final finite checks.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            34
        );
    }

    #[test]
    fn native_projection_refuses_before_the_last_finite_check_and_can_retry() {
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 33)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| check_native_linear(vello_encoding::Transform::IDENTITY, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 33);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(
            check_native_linear(vello_encoding::Transform::IDENTITY, &Account::for_test()).unwrap()
        );
    }
}
