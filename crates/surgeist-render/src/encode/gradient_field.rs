// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// Modified native field correspondence is documented in ../../NOTICE-VELLO.md.
//! Shared native scalar-field geometry and permitted-WGSL arithmetic certificates.
use super::BrushTransform;
use crate::{
    Error, PhysicalSize, Result,
    accounting::{Account, RenderWork},
};

mod sweep;
pub(super) use sweep::qualify_sweep;

fn invalid_geometry(value: &'static str) -> Error {
    Error::invalid_value(
        "native gradient realization",
        value,
        "must have finite, representable native field geometry",
    )
}

/// Admit each executed scalar arithmetic, comparison, or conversion before it
/// runs. Coefficient captures are separately admitted at their owning reads.
pub(super) fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

pub(super) fn check_finite(transform: kurbo::Affine, account: &Account) -> Result<()> {
    for value in transform.as_coeffs() {
        if !scalar(account, || value.is_finite())? {
            return Err(invalid_geometry("non-finite affine geometry"));
        }
    }
    Ok(())
}

fn check_native_finite(transform: vello_encoding::Transform, account: &Account) -> Result<()> {
    for value in transform.matrix.into_iter().chain(transform.translation) {
        if !scalar(account, || value.is_finite())? {
            return Err(invalid_geometry(
                "unrepresentable native affine coefficients",
            ));
        }
    }
    Ok(())
}

pub(super) fn narrow_transform(
    transform: kurbo::Affine,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    check_finite(transform, account)?;
    let values = transform.as_coeffs();
    let mut native = [0.0; 6];
    for (output, value) in native.iter_mut().zip(values) {
        *output = scalar(account, || value as f32)?;
    }
    let native = vello_encoding::Transform {
        matrix: [native[0], native[1], native[2], native[3]],
        translation: [native[4], native[5]],
    };
    check_native_finite(native, account)?;
    Ok(native)
}

// Matches vello_encoding 0.9.0 Transform::mul, preserving each association.
fn native_product(
    left: vello_encoding::Transform,
    right: vello_encoding::Transform,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let pair = |a: usize, b: usize| -> Result<f32> {
        let first = scalar(account, || left.matrix[a] * right.matrix[b])?;
        let second = scalar(account, || left.matrix[a + 2] * right.matrix[b + 1])?;
        scalar(account, || first + second)
    };
    let translation = |a: usize| -> Result<f32> {
        let first = scalar(account, || left.matrix[a] * right.translation[0])?;
        let second = scalar(account, || left.matrix[a + 2] * right.translation[1])?;
        let sum = scalar(account, || first + second)?;
        scalar(account, || sum + left.translation[a])
    };
    Ok(vello_encoding::Transform {
        matrix: [pair(0, 0)?, pair(1, 0)?, pair(0, 2)?, pair(1, 2)?],
        translation: [translation(0)?, translation(1)?],
    })
}

// Matches locked kurbo 0.13.1 Affine::mul, including translation association.
pub(super) fn affine_product(
    left: kurbo::Affine,
    right: kurbo::Affine,
    account: &Account,
) -> Result<kurbo::Affine> {
    account.work(RenderWork::CoordinateVisits, 12)?;
    let left = left.as_coeffs();
    let right = right.as_coeffs();
    let pair = |a: usize, b: usize| -> Result<f64> {
        let first = scalar(account, || left[a] * right[b])?;
        let second = scalar(account, || left[a + 2] * right[b + 1])?;
        scalar(account, || first + second)
    };
    let translation = |a: usize| -> Result<f64> {
        let first = scalar(account, || left[a] * right[4])?;
        let second = scalar(account, || left[a + 2] * right[5])?;
        let sum = scalar(account, || first + second)?;
        scalar(account, || sum + left[a + 4])
    };
    Ok(kurbo::Affine::new([
        pair(0, 0)?,
        pair(1, 0)?,
        pair(0, 2)?,
        pair(1, 2)?,
        translation(0)?,
        translation(1)?,
    ]))
}

#[derive(Clone, Copy)]
pub(super) struct RadialReference {
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) delta: f64,
}

/// Extra sampling arithmetic is proved about the sole accepted control state.
#[derive(Clone, Copy)]
pub(super) enum LinearSampling {
    Direct,
    HorizontalDistancePad { span: f64 },
    HorizontalDistanceRepeat { phase: f32 },
}

pub(super) fn native_glyph_transform(
    brush_transform: BrushTransform,
    paint_to_device: kurbo::Affine,
    radial_radii: Option<(f32, f32)>,
    target_extent: PhysicalSize,
    sampling: LinearSampling,
    account: &Account,
) -> Result<vello_encoding::Transform> {
    let local = narrow_transform(brush_transform.local, account)?;
    let shape = narrow_transform(paint_to_device, account)?;
    // GlyphRun's resolver composes already-narrowed transforms. Neither factor
    // is inverted independently for the paint, so check the actual product.
    let composed = native_product(shape, local, account)?;
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
        qualify_linear_sampling(
            brush_transform.reference_device,
            composed,
            target_extent,
            brush_transform.comparison_only,
            sampling,
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

    pub(super) fn checked(lower: f64, upper: f64, account: &Account) -> Result<Self> {
        if !scalar(account, || lower.is_finite())?
            || !scalar(account, || upper.is_finite())?
            || scalar(account, || lower > upper)?
        {
            return Err(invalid_geometry(
                "unbounded native gradient arithmetic certificate",
            ));
        }
        Ok(Self { lower, upper })
    }
    fn magnitude(self, account: &Account) -> Result<f64> {
        let lower = scalar(account, || self.lower.abs())?;
        let upper = scalar(account, || self.upper.abs())?;
        scalar(account, || lower.max(upper))
    }
    pub(super) fn neg(self, account: &Account) -> Result<Self> {
        let lower = scalar(account, || -self.upper)?;
        let upper = scalar(account, || -self.lower)?;
        Self::checked(lower, upper, account)
    }
    pub(super) fn abs(self, account: &Account) -> Result<Self> {
        let lower =
            if scalar(account, || self.lower <= 0.0)? && scalar(account, || self.upper >= 0.0)? {
                0.0
            } else {
                let low = scalar(account, || self.lower.abs())?;
                let high = scalar(account, || self.upper.abs())?;
                scalar(account, || low.min(high))?
            };
        let low = scalar(account, || self.lower.abs())?;
        let high = scalar(account, || self.upper.abs())?;
        let upper = scalar(account, || low.max(high))?;
        Self::checked(lower, upper, account)
    }
    fn hull(self, other: Self, account: &Account) -> Result<Self> {
        let lower = scalar(account, || self.lower.min(other.lower))?;
        let upper = scalar(account, || self.upper.max(other.upper))?;
        Self::checked(lower, upper, account)
    }
    // Include permitted input flush-to-zero. Exact zeros remain exact.
    fn flushed(self, account: &Account) -> Result<Self> {
        if scalar(account, || self.lower < SHADER_MIN_NORMAL)?
            && scalar(account, || self.upper > -SHADER_MIN_NORMAL)?
        {
            let lower = scalar(account, || self.lower.min(0.0))?;
            let upper = scalar(account, || self.upper.max(0.0))?;
            Self::checked(lower, upper, account)
        } else {
            Ok(self)
        }
    }
    pub(super) fn real_add(self, other: Self, account: &Account) -> Result<Self> {
        let sum = scalar(account, || self.lower + other.lower)?;
        let lower = scalar(account, || sum.next_down())?;
        let sum = scalar(account, || self.upper + other.upper)?;
        let upper = scalar(account, || sum.next_up())?;
        Self::checked(lower, upper, account)
    }
    fn corner_bounds(values: [f64; 4], account: &Account) -> Result<Self> {
        let mut lower = f64::INFINITY;
        for value in values {
            lower = scalar(account, || lower.min(value))?;
        }
        let lower = scalar(account, || lower.next_down())?;
        let mut upper = f64::NEG_INFINITY;
        for value in values {
            upper = scalar(account, || upper.max(value))?;
        }
        let upper = scalar(account, || upper.next_up())?;
        Self::checked(lower, upper, account)
    }
    pub(super) fn real_mul(self, other: Self, account: &Account) -> Result<Self> {
        let products = [
            scalar(account, || self.lower * other.lower)?,
            scalar(account, || self.lower * other.upper)?,
            scalar(account, || self.upper * other.lower)?,
            scalar(account, || self.upper * other.upper)?,
        ];
        Self::corner_bounds(products, account)
    }
    pub(super) fn real_div(self, other: Self, account: &Account) -> Result<Self> {
        if scalar(account, || other.lower <= 0.0)? && scalar(account, || other.upper >= 0.0)? {
            return Err(invalid_geometry("uncertain native gradient divisor"));
        }
        let quotients = [
            scalar(account, || self.lower / other.lower)?,
            scalar(account, || self.lower / other.upper)?,
            scalar(account, || self.upper / other.lower)?,
            scalar(account, || self.upper / other.upper)?,
        ];
        Self::corner_bounds(quotients, account)
    }
    fn real_sqrt(self, account: &Account) -> Result<Self> {
        if scalar(account, || self.lower < 0.0)? {
            return Err(invalid_geometry("uncertain native gradient square root"));
        }
        let root = scalar(account, || self.lower.sqrt())?;
        let outward = scalar(account, || root.next_down())?;
        let lower = scalar(account, || outward.max(0.0))?;
        let root = scalar(account, || self.upper.sqrt())?;
        let upper = scalar(account, || root.next_up())?;
        Self::checked(lower, upper, account)
    }
    // e is a full binary32 epsilon: WGSL permits either rounding direction.
    // 2.5 ULP <= 2.5e*|exact result| for a normal result. One minimum-normal
    // absolute allowance also covers subnormal rounding and output FTZ.
    fn rounded(self, relative: f64, account: &Account) -> Result<Self> {
        let magnitude = self.magnitude(account)?;
        let product = scalar(account, || relative * magnitude)?;
        let outward = scalar(account, || product.next_up())?;
        let sum = scalar(account, || outward + SHADER_MIN_NORMAL)?;
        let error = scalar(account, || sum.next_up())?;
        let difference = scalar(account, || self.lower - error)?;
        let lower = scalar(account, || difference.next_down())?;
        let sum = scalar(account, || self.upper + error)?;
        let upper = scalar(account, || sum.next_up())?;
        let result = Self::checked(lower, upper, account)?;
        const MAX: f64 = f32::MAX as f64;
        if scalar(account, || result.lower < -MAX)? || scalar(account, || result.upper > MAX)? {
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
        if scalar(account, || absolute.lower < SHADER_MIN_NORMAL)?
            || scalar(account, || absolute.upper > SHADER_MAX_DIVISOR)?
        {
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
        if scalar(account, || value.lower < SHADER_MIN_NORMAL)? {
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
    let mut converted = [ShaderScalar::ZERO; 6];
    for (output, value) in converted.iter_mut().zip(native) {
        *output = ShaderScalar::exact(scalar(account, || f64::from(value))?);
    }
    let inverse = circular_inverse(converted, account)?;
    let start = ShaderScalar::exact(scalar(account, || f64::from(start))?);
    let end = ShaderScalar::exact(scalar(account, || f64::from(end))?);
    let epsilon = ShaderScalar::exact(1.0 / 4096.0);
    let difference = start.add(end.neg(account)?, account)?;
    let absolute_difference = difference.abs(account)?;
    if scalar(account, || absolute_difference.lower <= epsilon.upper)?
        || scalar(account, || end.lower <= 0.0)?
    {
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
    if scalar(account, || radius_difference.lower <= epsilon.upper)?
        || scalar(account, || radius.lower <= 1.0)?
        || scalar(account, || one_minus_focal.lower <= 0.0)?
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
    let width = target_extent.width();
    let height = target_extent.height();
    if width == 0 || height == 0 || width - 1 > 1 << 24 || height - 1 > 1 << 24 {
        return Err(invalid_geometry(
            "unrepresentable native target pixel coordinates",
        ));
    }
    let x = ShaderScalar::exact(scalar(account, || f64::from(width - 1))?);
    let y = ShaderScalar::exact(scalar(account, || f64::from(height - 1))?);
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
    check_native_finite(native, account)?;
    let a =
        ShaderScalar::exact(scalar(account, || f64::from(native.matrix[0]))?).flushed(account)?;
    let b =
        ShaderScalar::exact(scalar(account, || f64::from(native.matrix[1]))?).flushed(account)?;
    let x = ShaderScalar::exact(scalar(account, || f64::from(native.translation[0]))?)
        .flushed(account)?;
    let y = ShaderScalar::exact(scalar(account, || f64::from(native.translation[1]))?)
        .flushed(account)?;
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
    qualify_linear_sampling(
        reference_device,
        native_device,
        target_extent,
        comparison_only,
        LinearSampling::Direct,
        account,
    )
}

pub(super) fn qualify_linear_sampling(
    reference_device: kurbo::Affine,
    native_device: vello_encoding::Transform,
    target_extent: PhysicalSize,
    comparison_only: bool,
    sampling: LinearSampling,
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
    // A comparison field also requires finite fine-stage arithmetic. Infinite
    // raw coordinates would otherwise choose an unrelated endpoint color.
    finite_fine_bound(phase_bound, account)?;
    if comparison_only {
        // Sign-bit folding preserves the exact native coordinate magnitude;
        // step cutoffs retain the existing finite-only comparison contract.
        return Ok(());
    }
    let quantum = scalar(account, || 1.0_f64 / 511.0)?;
    let quantum = scalar(account, || quantum.next_down())?;
    let (error, magnitude, threshold) = match sampling {
        LinearSampling::Direct => (phase_error, phase_bound, quantum),
        LinearSampling::HorizontalDistancePad { span } => {
            if !scalar(account, || span.is_finite())? || !scalar(account, || span > 0.0)? {
                return Err(invalid_geometry("unrepresentable distance control span"));
            }
            let threshold = ShaderScalar::exact(span)
                .real_mul(ShaderScalar::exact(quantum), account)?
                .lower;
            (phase_error, phase_bound, threshold)
        }
        LinearSampling::HorizontalDistanceRepeat { phase } => {
            let (error, magnitude) =
                distance_repeat_bounds(phase_error, phase_bound, phase, account)?;
            (error, magnitude, quantum)
        }
    };
    if scalar(account, || error.upper > threshold)?
        || phase_exceeds_quantum(magnitude.upper, threshold, account)?
    {
        return Err(invalid_geometry(
            "insufficient native linear phase capacity",
        ));
    }
    Ok(())
}

/// Bound the exact shader fold/subtract/floor/subtract sequence. The sign-bit
/// fold is 1-Lipschitz, including the cusp. Floor is exact for the bounded f32
/// input; modulo changes the comparison to circular phase distance. The two
/// subtraction bounds include their input/output FTZ losses and hold across
/// negative values and every integer seam without visiting target pixels.
fn distance_repeat_bounds(
    field_error: ShaderScalar,
    field_magnitude: ShaderScalar,
    phase: f32,
    account: &Account,
) -> Result<(ShaderScalar, ShaderScalar)> {
    if !scalar(account, || phase.is_finite())?
        || scalar(account, || phase < 0.0)?
        || scalar(account, || phase > 1.0)?
        || (scalar(account, || phase != 0.0)? && !scalar(account, || phase.is_normal())?)
    {
        return Err(invalid_geometry("uncertifiable native distance phase"));
    }
    let phase = scalar(account, || f64::from(phase))?;
    let phase = ShaderScalar::exact(phase);
    let epsilon = ShaderScalar::exact(SHADER_EPSILON);
    let min = ShaderScalar::exact(SHADER_MIN_NORMAL);
    // The source projection has separately refused a nonzero phase lost to0.
    // Full epsilon is conservative for either permitted native rounding.
    let narrowing = phase.real_mul(epsilon, account)?;
    let terms = field_magnitude.real_add(phase, account)?;
    let subtraction = terms
        .real_mul(epsilon, account)?
        .real_add(min.real_mul(ShaderScalar::exact(2.0), account)?, account)?;
    let shifted = terms.real_add(subtraction, account)?;
    finite_fine_bound(shifted, account)?;
    if scalar(account, || shifted.upper >= 16_777_216.0)? {
        return Err(invalid_geometry(
            "distance phase lacks native fractional precision",
        ));
    }
    // |y| + |floor(y)| <= 2|y|+1. This term bound also covers permitted
    // association instead of relying on a CPU cancellation/Sterbenz shortcut.
    let wrap_terms = shifted
        .real_mul(ShaderScalar::exact(2.0), account)?
        .real_add(ShaderScalar::ONE, account)?;
    let wrap = wrap_terms
        .real_mul(epsilon, account)?
        .real_add(min.real_mul(ShaderScalar::exact(3.0), account)?, account)?;
    let error = field_error
        .real_add(narrowing, account)?
        .real_add(subtraction, account)?
        .real_add(wrap, account)?;
    Ok((error, shifted))
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
    // The guarded native sqrt still evaluates xx+yy first. Bound every square
    // and either sum association before accepting the circular fine branch.
    let squared_bound = radius
        .real_mul(radius, account)?
        .real_mul(squared_factor, account)?
        .real_add(squared_absolute, account)?;
    finite_fine_bound(squared_bound, account)?;
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
    // A comparison field also requires finite fine-stage arithmetic. Infinite
    // raw coordinates would otherwise choose an unrelated endpoint color.
    finite_fine_bound(phase_bound, account)?;
    let focal_error = focal
        .real_add(
            ShaderScalar::exact(scalar(account, || reference.start / reference.delta)?),
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
    let quantum = scalar(account, || 1.0_f64 / 511.0)?;
    let quantum = scalar(account, || quantum.next_down())?;
    if !comparison_only
        && (scalar(account, || phase_error.upper > quantum)?
            || phase_exceeds_quantum(phase_bound.upper, quantum, account)?)
    {
        return Err(invalid_geometry(
            "insufficient native circular phase capacity",
        ));
    }
    Ok(())
}

fn finite_fine_bound(bound: ShaderScalar, account: &Account) -> Result<()> {
    const MAX: f64 = f32::MAX as f64;
    if scalar(account, || bound.upper > MAX)? {
        return Err(invalid_geometry(
            "non-finite permitted gradient fine intermediate",
        ));
    }
    Ok(())
}

fn phase_exceeds_quantum(bound: f64, quantum: f64, account: &Account) -> Result<bool> {
    let ulp = phase_ulp(bound, account)?;
    scalar(account, || ulp > quantum)
}

fn phase_ulp(bound: f64, account: &Account) -> Result<f64> {
    let mut upper = scalar(account, || bound as f32)?;
    let widened = scalar(account, || f64::from(upper))?;
    if scalar(account, || widened < bound)? {
        upper = scalar(account, || upper.next_up())?;
    }
    let next = scalar(account, || upper.next_up())?;
    let difference = scalar(account, || next - upper)?;
    scalar(account, || f64::from(difference))
}

/// Vello projects transformed endpoints. Solve equivalent perpendicular device
/// endpoints back into the caller's paint coordinates, retaining the independent
/// intended map before recomposition can lose phase.
pub(super) fn linear_brush_transform(
    paint_to_device: kurbo::Affine,
    period_to_device: kurbo::Affine,
    repeat: bool,
    account: &Account,
) -> Result<BrushTransform> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d, x, y] = period_to_device.as_coeffs();
    let c_abs = scalar(account, || c.abs())?;
    let d_abs = scalar(account, || d.abs())?;
    let transverse_scale = scalar(account, || c_abs.max(d_abs))?;
    let c = scalar(account, || c / transverse_scale)?;
    let d = scalar(account, || d / transverse_scale)?;
    let transverse_length = scalar(account, || c.hypot(d))?;
    let normal_x = scalar(account, || d / transverse_length)?;
    let minus_c = scalar(account, || -c)?;
    let normal_y = scalar(account, || minus_c / transverse_length)?;
    let first = scalar(account, || a * normal_x)?;
    let second = scalar(account, || b * normal_y)?;
    let signed_period = scalar(account, || first + second)?;
    let period = scalar(account, || signed_period.abs())?;
    if scalar(account, || period == 0.0)? || !scalar(account, || period.is_finite())? {
        return Err(invalid_geometry("unrepresentable effective linear period"));
    }
    let sign = scalar(account, || signed_period.signum())?;
    let direction_x = scalar(account, || normal_x * sign)?;
    let sign = scalar(account, || signed_period.signum())?;
    let direction_y = scalar(account, || normal_y * sign)?;
    let vector_x = scalar(account, || normal_x * signed_period)?;
    let vector_y = scalar(account, || normal_y * signed_period)?;
    let first = scalar(account, || x * direction_x)?;
    let second = scalar(account, || y * direction_y)?;
    let projected = scalar(account, || first + second)?;
    // Only Repeat discards whole cycles; ordinary Pad retains absolute phase.
    let phase = if repeat {
        scalar(account, || projected.rem_euclid(period))?
    } else {
        projected
    };
    let origin_x = scalar(account, || direction_x * phase)?;
    let origin_y = scalar(account, || direction_y * phase)?;
    let minus_vector_y = scalar(account, || -vector_y)?;
    let reference_device = kurbo::Affine::new([
        vector_x,
        vector_y,
        minus_vector_y,
        vector_x,
        origin_x,
        origin_y,
    ]);
    check_finite(reference_device, account)?;
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [aa, ab, ac, ad, ax, ay] = paint_to_device.as_coeffs();
    let aa_abs = scalar(account, || aa.abs())?;
    let ab_abs = scalar(account, || ab.abs())?;
    let first_scale = scalar(account, || aa_abs.max(ab_abs))?;
    let ac_abs = scalar(account, || ac.abs())?;
    let ad_abs = scalar(account, || ad.abs())?;
    let second_scale = scalar(account, || ac_abs.max(ad_abs))?;
    let aa = scalar(account, || aa / first_scale)?;
    let ab = scalar(account, || ab / first_scale)?;
    let ac = scalar(account, || ac / second_scale)?;
    let ad = scalar(account, || ad / second_scale)?;
    let first = scalar(account, || aa * ad)?;
    let second = scalar(account, || ab * ac)?;
    let determinant = scalar(account, || first - second)?;
    if scalar(account, || determinant == 0.0)? || !scalar(account, || determinant.is_finite())? {
        return Err(invalid_geometry("unrepresentable paint-to-device inverse"));
    }
    let solve = |x: f64, y: f64| -> Result<[f64; 2]> {
        let first = scalar(account, || ad * x)?;
        let second = scalar(account, || ac * y)?;
        let difference = scalar(account, || first - second)?;
        let quotient = scalar(account, || difference / determinant)?;
        let u = scalar(account, || quotient / first_scale)?;
        let first = scalar(account, || aa * y)?;
        let second = scalar(account, || ab * x)?;
        let difference = scalar(account, || first - second)?;
        let quotient = scalar(account, || difference / determinant)?;
        let v = scalar(account, || quotient / second_scale)?;
        Ok([u, v])
    };
    let first = solve(vector_x, vector_y)?;
    let minus_vector_y = scalar(account, || -vector_y)?;
    let second = solve(minus_vector_y, vector_x)?;
    let x = scalar(account, || origin_x - ax)?;
    let y = scalar(account, || origin_y - ay)?;
    let origin = solve(x, y)?;
    let local = kurbo::Affine::new([
        first[0], first[1], second[0], second[1], origin[0], origin[1],
    ]);
    check_finite(local, account)?;
    Ok(BrushTransform {
        local,
        reference_device,
        radial: None,
        comparison_only: false,
    })
}

/// Mirror the native endpoint and projection arithmetic. Actual collapse is a
/// Repeat policy input; other failures remain contextual geometry refusals.
pub(super) fn check_native_linear(
    transform: vello_encoding::Transform,
    account: &Account,
) -> Result<bool> {
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, _, _] = transform.matrix;
    let [x, y] = transform.translation;
    check_native_finite(transform, account)?;
    let endpoint_x = scalar(account, || x + a)?;
    let dx = scalar(account, || endpoint_x - x)?;
    let endpoint_y = scalar(account, || y + b)?;
    let dy = scalar(account, || endpoint_y - y)?;
    if scalar(account, || dx == 0.0)? && scalar(account, || dy == 0.0)? {
        return Ok(false);
    }
    let xx = scalar(account, || dx * dx)?;
    let yy = scalar(account, || dy * dy)?;
    let squared = scalar(account, || xx + yy)?;
    let inverse = scalar(account, || 1.0 / squared)?;
    let nx = scalar(account, || dx * inverse)?;
    let ny = scalar(account, || dy * inverse)?;
    let first = scalar(account, || x * nx)?;
    let second = scalar(account, || y * ny)?;
    let sum = scalar(account, || first + second)?;
    let offset = scalar(account, || -sum)?;
    for value in [dx, dy, squared, inverse, nx, ny, offset] {
        if !scalar(account, || value.is_finite())? {
            return Err(invalid_geometry("unrepresentable native linear projection"));
        }
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
                .attempt()
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
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 33);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(
            check_native_linear(vello_encoding::Transform::IDENTITY, &Account::for_test()).unwrap()
        );
    }
    #[test]
    fn affine_finite_walk_admits_each_executed_check_and_preserves_refused_prefix() {
        let mut context = RenderAccountingContext::default();
        context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| check_finite(kurbo::Affine::IDENTITY, account),
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            6
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 5)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| check_finite(kurbo::Affine::IDENTITY, account),
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 5);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(check_finite(kurbo::Affine::IDENTITY, &Account::for_test()).is_ok());
    }
    #[test]
    fn linear_projection_admits_normalization_and_inverse_solve_actions() {
        let mut context = RenderAccountingContext::default();
        let brush = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    linear_brush_transform(
                        kurbo::Affine::IDENTITY,
                        kurbo::Affine::IDENTITY,
                        false,
                        account,
                    )
                },
            )
            .unwrap();
        assert_eq!(brush.local, kurbo::Affine::IDENTITY);
        // 12 input coefficient captures;27 initial projection actions;12
        // finite checks;15 inverse setup actions;30 solve actions; one second
        // vector negation and two translation differences:99 total.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            99
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 20)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| {
                    linear_brush_transform(
                        kurbo::Affine::IDENTITY,
                        kurbo::Affine::IDENTITY,
                        false,
                        account,
                    )
                },
            )
            .err()
            .unwrap();
        assert!(error.accounting_diagnostic().is_some());
        // Six coefficient captures plus14 normalization/projection actions;
        // the next finite-period check is refused before inverse work starts.
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 20);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert!(
            linear_brush_transform(
                kurbo::Affine::IDENTITY,
                kurbo::Affine::IDENTITY,
                false,
                &Account::for_test()
            )
            .is_ok()
        );
    }
    #[test]
    fn outward_rounding_accounts_arithmetic_and_refuses_before_the_last_range_check() {
        let mut context = RenderAccountingContext::default();
        let rounded = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| ShaderScalar::ONE.rounded(SHADER_EPSILON, account),
            )
            .unwrap();
        assert!(rounded.lower < 1.0 && rounded.upper > 1.0);
        // Three magnitude actions, four outward-error actions, four bound
        // actions, three checked-interval predicates, two native-range checks.
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            16
        );
        let mut context = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 15)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::Frame,
                RenderPhase::GradientPrepare,
                |account| ShaderScalar::ONE.rounded(SHADER_EPSILON, account),
            )
            .err()
            .unwrap();
        assert!(error.accounting_diagnostic().is_some());
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            15
        );
        assert!(
            ShaderScalar::ONE
                .rounded(SHADER_EPSILON, &Account::for_test())
                .is_ok()
        );
    }
    #[test]
    fn accounted_native_product_matches_locked_provider_bits() {
        use vello_encoding::Transform;
        let pairs = [
            (
                Transform {
                    matrix: [1.25, -0.5, 2.75, -3.5],
                    translation: [19.25, -11.5],
                },
                Transform {
                    matrix: [-4.5, 3.75, 0.125, 8.5],
                    translation: [-17.5, 23.25],
                },
            ),
            (
                Transform {
                    matrix: [2.0_f32.powi(20), -2.0_f32.powi(20), 0.5, -0.25],
                    translation: [-2.0_f32.powi(24), 2.0_f32.powi(24)],
                },
                Transform {
                    matrix: [0.125, 2.0, -4.0, 0.25],
                    translation: [16., -32.],
                },
            ),
        ];
        for (left, right) in pairs {
            let expected = left * right; // Locked provider protocol owner.
            let actual = native_product(left, right, &Account::for_test()).unwrap();
            assert_eq!(
                actual.matrix.map(f32::to_bits),
                expected.matrix.map(f32::to_bits)
            );
            assert_eq!(
                actual.translation.map(f32::to_bits),
                expected.translation.map(f32::to_bits)
            );
        }
    }
    #[test]
    fn accounted_affine_product_matches_locked_provider_and_admits_every_action() {
        let pairs = [
            (
                kurbo::Affine::new([1.25, -0.5, 2.75, -3.5, 19.25, -11.5]),
                kurbo::Affine::new([-4.5, 3.75, 0.125, 8.5, -17.5, 23.25]),
            ),
            (
                kurbo::Affine::new([
                    2.0_f64.powi(40),
                    -2.0_f64.powi(40),
                    0.5,
                    -0.25,
                    -2.0_f64.powi(44),
                    2.0_f64.powi(44),
                ]),
                kurbo::Affine::new([0.125, 2.0, -4.0, 0.25, 16., -32.]),
            ),
        ];
        for (left, right) in pairs {
            let expected = left * right; // Locked provider protocol owner.
            let mut context = RenderAccountingContext::default();
            let actual = context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::GradientPrepare,
                    |account| affine_product(left, right, account),
                )
                .unwrap();
            assert_eq!(
                actual.as_coeffs().map(f64::to_bits),
                expected.as_coeffs().map(f64::to_bits)
            );
            // Twelve coefficient captures, twelve products, eight additions.
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::CoordinateVisits),
                32
            );
            let mut context = RenderAccountingContext::new(
                RenderAccountingOptions::new().with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::CoordinateVisits, 31)
                        .unwrap(),
                ),
            );
            let error = context
                .run(
                    RenderAttemptKind::Frame,
                    RenderPhase::GradientPrepare,
                    |account| affine_product(left, right, account),
                )
                .err()
                .unwrap();
            assert!(error.accounting_diagnostic().is_some());
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap()
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::CoordinateVisits),
                31
            );
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
            assert_eq!(
                affine_product(left, right, &Account::for_test())
                    .unwrap()
                    .as_coeffs()
                    .map(f64::to_bits),
                expected.as_coeffs().map(f64::to_bits)
            );
        }
    }
}
