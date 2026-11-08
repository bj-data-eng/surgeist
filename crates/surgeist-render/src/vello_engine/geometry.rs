// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT OR Unlicense
// Modified native geometry correspondence is documented in ../../NOTICE-VELLO.md.
//! Qualification of the actual locked native factors and streaming path input.
//!
//! This is a conservative receiving proof, not a replacement tessellator or an
//! exact-f64 raster contract. Local retention follows PathEncoder's predicate;
//! composed points cover both the fused flatten and unfused transform forms.
use crate::accounting::{Account, RenderWork};
use crate::{GeometryRealizationError as Reason, Result};
use kurbo::{Affine, PathEl, Point, Rect, Stroke};
use vello_encoding::Transform;

fn action<T>(account: &Account, run: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(run())
}

/// A cast is allowed to round normally. Concrete overflow/zero loss is distinct
/// from a surviving subnormal whose execution is not portable across GPU FTZ.
pub(crate) fn scalar(value: f64, reason: Reason, account: &Account) -> Result<f32> {
    if !action(account, || value.is_finite())? {
        return Err(reason.into());
    }
    let native = action(account, || value as f32)?;
    if !action(account, || native.is_finite())? {
        return Err(reason.into());
    }
    if action(account, || value != 0.0)? {
        if action(account, || native == 0.0)? {
            return Err(reason.into());
        }
        if !action(account, || native.is_normal())? {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
    }
    Ok(native)
}

pub(crate) struct QualifiedTransform {
    native: Transform,
    nonsingular: bool,
}
impl QualifiedTransform {
    pub(crate) fn native(&self) -> Transform {
        self.native
    }
}

fn finite_native(value: f32, account: &Account) -> Result<f32> {
    if !action(account, || value.is_finite())? {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    if action(account, || value != 0.0)? && !action(account, || value.is_normal())? {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    Ok(value)
}

fn rank(source: [f64; 6], native: Transform, account: &Account) -> Result<bool> {
    // Source normalization is the existing scene's finite-affine rank policy.
    // Native 24-bit products widen exactly and cannot overflow/underflow f64.
    let mut scale = 0.0_f64;
    for v in &source[..4] {
        let v = action(account, || v.abs())?;
        scale = action(account, || scale.max(v))?;
    }
    if action(account, || scale == 0.0)? {
        return Ok(false);
    }
    let a = action(account, || source[0] / scale)?;
    let b = action(account, || source[1] / scale)?;
    let c = action(account, || source[2] / scale)?;
    let d = action(account, || source[3] / scale)?;
    let ad = action(account, || a * d)?;
    let bc = action(account, || b * c)?;
    let determinant = action(account, || ad - bc)?;
    if action(account, || determinant == 0.0)? {
        return Ok(false);
    }
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [a, b, c, d] = native.matrix;
    let a = action(account, || f64::from(a))?;
    let b = action(account, || f64::from(b))?;
    let c = action(account, || f64::from(c))?;
    let d = action(account, || f64::from(d))?;
    let ad = action(account, || a * d)?;
    let bc = action(account, || b * c)?;
    let determinant = action(account, || ad - bc)?;
    if action(account, || determinant == 0.0)? {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    Ok(true)
}

pub(crate) fn qualify_transform(source: Affine, account: &Account) -> Result<QualifiedTransform> {
    let mut values = [0.0; 6];
    let mut native = [0.0; 6];
    account.work(RenderWork::CoordinateVisits, 6)?;
    for (index, value) in source.as_coeffs().into_iter().enumerate() {
        values[index] = value;
        native[index] = scalar(value, Reason::UnrepresentableTransform, account)?;
    }
    let native = Transform {
        matrix: [native[0], native[1], native[2], native[3]],
        translation: [native[4], native[5]],
    };
    Ok(QualifiedTransform {
        native,
        nonsingular: rank(values, native, account)?,
    })
}

/// Qualify the actual product that deferred native glyph resolution will use.
pub(crate) fn qualify_native_transform(
    source: Affine,
    native: Transform,
    account: &Account,
) -> Result<QualifiedTransform> {
    let mut values = [0.0; 6];
    account.work(RenderWork::CoordinateVisits, 6)?;
    for (index, value) in source.as_coeffs().into_iter().enumerate() {
        values[index] = value;
        if !action(account, || value.is_finite())? {
            return Err(Reason::UnrepresentableTransform.into());
        }
    }
    account.work(RenderWork::CoordinateVisits, 6)?;
    for (index, value) in native
        .matrix
        .into_iter()
        .chain(native.translation)
        .enumerate()
    {
        if !action(account, || value.is_finite())? {
            return Err(Reason::UnrepresentableTransform.into());
        }
        if action(account, || values[index] != 0.0)? && action(account, || value == 0.0)? {
            return Err(Reason::UnrepresentableTransform.into());
        }
        finite_native(value, account)?;
    }
    Ok(QualifiedTransform {
        native,
        nonsingular: rank(values, native, account)?,
    })
}

/// Exact arithmetic association of the locked vello_encoding::Transform::mul.
pub(crate) fn native_product(
    left: Transform,
    right: Transform,
    account: &Account,
) -> Result<Transform> {
    account.work(RenderWork::CoordinateVisits, 12)?; // twelve actual factor captures
    let [a, b, c, d] = left.matrix;
    let [e, f] = left.translation;
    let [g, h, i, j] = right.matrix;
    let [k, l] = right.translation;
    let pair = |x, y, z, w| -> Result<f32> {
        let p = product(x, y, account)?;
        let q = product(z, w, account)?;
        let sum = action(account, || p + q)?;
        finite_native(sum, account)
    };
    let matrix = [
        pair(a, g, c, h)?,
        pair(b, g, d, h)?,
        pair(a, i, c, j)?,
        pair(b, i, d, j)?,
    ];
    let x = pair(a, k, c, l)?;
    let y = pair(b, k, d, l)?;
    let tx = action(account, || x + e)?;
    let ty = action(account, || y + f)?;
    let translation = [finite_native(tx, account)?, finite_native(ty, account)?];
    Ok(Transform {
        matrix,
        translation,
    })
}

/// Exact association of the locked kurbo affine product; no hidden operator walk.
pub(crate) fn affine_product(left: Affine, right: Affine, account: &Account) -> Result<Affine> {
    account.work(RenderWork::CoordinateVisits, 12)?; // twelve actual factor captures
    let [a, b, c, d, e, f] = left.as_coeffs();
    let [g, h, i, j, k, l] = right.as_coeffs();
    let pair = |x, y, z, w| -> Result<f64> {
        let p = action(account, || x * y)?;
        let q = action(account, || z * w)?;
        action(account, || p + q)
    };
    let x = pair(a, k, c, l)?;
    let y = pair(b, k, d, l)?;
    Ok(Affine::new([
        pair(a, g, c, h)?,
        pair(b, g, d, h)?,
        pair(a, i, c, j)?,
        pair(b, i, d, j)?,
        action(account, || x + e)?,
        action(account, || y + f)?,
    ]))
}

#[derive(Clone, Copy)]
struct MappedPoint {
    source: Point,
    native: [f32; 2],
    mapped: [f32; 2],
    error: [f64; 2],
}

// The arithmetic guard covers finite normal inputs, the actual nested fma form,
// and the separately evaluated products/sums in shared/transform.wgsl. A
// conservative enclosure covers permitted operation error, including possible
// FTZ near zero. Exactly representable axis operations retain a zero enclosure.
fn arithmetic(value: f32, account: &Account) -> Result<f32> {
    finite_native(value, account)?;
    let extent = action(account, || value.abs())?;
    // Sixteen ulps of relative headroom cover the retained short expression
    // chains even when the final value is cancelled by another term.
    const LIMIT: f32 = f32::MAX / (1.0 + 16.0 * f32::EPSILON);
    if !action(account, || extent <= LIMIT)? {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    Ok(value)
}
fn product(a: f32, b: f32, account: &Account) -> Result<f32> {
    let result = action(account, || a * b)?;
    arithmetic(result, account)?;
    if action(account, || result == 0.0)?
        && action(account, || a != 0.0)?
        && action(account, || b != 0.0)?
    {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    let extent = action(account, || result.abs())?;
    if action(account, || extent <= 2.0 * f32::MIN_POSITIVE)? && action(account, || result != 0.0)?
    {
        let aw = action(account, || f64::from(a))?;
        let bw = action(account, || f64::from(b))?;
        let exact = action(account, || aw * bw)?;
        let widened = action(account, || f64::from(result))?;
        if action(account, || exact != widened)? {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
    }
    Ok(result)
}
fn mapped_axis(a: f32, x: f32, c: f32, y: f32, e: f32, account: &Account) -> Result<(f32, f64)> {
    let ax = product(a, x, account)?;
    let cy = product(c, y, account)?;
    let sum = action(account, || ax + cy)?;
    arithmetic(sum, account)?;
    let unfused = action(account, || sum + e)?;
    arithmetic(unfused, account)?;
    let inner = action(account, || c.mul_add(y, e))?;
    arithmetic(inner, account)?;
    let fused = action(account, || a.mul_add(x, inner))?;
    arithmetic(fused, account)?;
    // Widening two f32 factors gives exact f64 products. A single nonzero
    // product plus an exactly represented translation is also exact if the
    // inverse subtraction recovers both operands (Sterbenz/TwoSum check).
    let ax64 = action(account, || f64::from(a))?;
    let x64 = action(account, || f64::from(x))?;
    let cy64 = action(account, || f64::from(c))?;
    let y64 = action(account, || f64::from(y))?;
    let e64 = action(account, || f64::from(e))?;
    let p = action(account, || ax64 * x64)?;
    let q = action(account, || cy64 * y64)?;
    let s = action(account, || p + q)?;
    let z = action(account, || s - p)?;
    let t = action(account, || s - z)?;
    let pe = action(account, || p - t)?;
    let qe = action(account, || q - z)?;
    let residual = action(account, || pe + qe)?;
    let total = action(account, || s + e64)?;
    let z = action(account, || total - s)?;
    let t = action(account, || total - z)?;
    let se = action(account, || s - t)?;
    let ee = action(account, || e64 - z)?;
    let residual2 = action(account, || se + ee)?;
    let alternate_total = action(account, || p + e64)?;
    let alternate_z = action(account, || alternate_total - p)?;
    let alternate_t = action(account, || alternate_total - alternate_z)?;
    let alternate_pe = action(account, || p - alternate_t)?;
    let alternate_ee = action(account, || e64 - alternate_z)?;
    let alternate_residual = action(account, || alternate_pe + alternate_ee)?;
    // Reassociation may choose any pairing of the three sum terms.
    let alternate_native = action(account, || ax + e)?;
    arithmetic(alternate_native, account)?;
    let alternate_value = action(account, || f64::from(alternate_native))?;
    let inner_total = action(account, || q + e64)?;
    let inner_z = action(account, || inner_total - q)?;
    let inner_t = action(account, || inner_total - inner_z)?;
    let inner_qe = action(account, || q - inner_t)?;
    let inner_ee = action(account, || e64 - inner_z)?;
    let inner_residual = action(account, || inner_qe + inner_ee)?;
    let ax_value = action(account, || f64::from(ax))?;
    let cy_value = action(account, || f64::from(cy))?;
    let sum_value = action(account, || f64::from(sum))?;
    let inner_value = action(account, || f64::from(inner))?;
    let f64value = action(account, || f64::from(fused))?;
    let u64value = action(account, || f64::from(unfused))?;
    // Exact add/product facts, rather than a zero-size global tolerance.
    let exact = action(account, || p == ax_value)?
        && action(account, || q == cy_value)?
        && action(account, || s == sum_value)?
        && action(account, || inner_total == inner_value)?
        && action(account, || inner_residual == 0.0)?
        && action(account, || alternate_residual == 0.0)?
        && action(account, || alternate_total == alternate_value)?
        && action(account, || residual == 0.0)?
        && action(account, || residual2 == 0.0)?
        && action(account, || total == f64value)?
        && action(account, || total == u64value)?;
    if exact {
        return Ok((fused, 0.0));
    }
    let p = action(account, || p.abs())?;
    let q = action(account, || q.abs())?;
    let e = action(account, || e64.abs())?;
    let magnitude = action(account, || p + q)?;
    let magnitude = action(account, || magnitude.next_up())?;
    let magnitude = action(account, || magnitude + e)?;
    let magnitude = action(account, || magnitude.next_up())?;
    // The relative-error factor is a power of two, so this f64 scaling
    // is exact on this finite normal magnitude domain. Bound additions widen.
    let error = action(account, || magnitude * (8.0 * f32::EPSILON as f64))?;
    let error = action(account, || error + (8.0 * f32::MIN_POSITIVE as f64))?;
    let error = action(account, || error.next_up())?;
    let extent = action(account, || total.abs())?;
    let extent = action(account, || extent + error)?;
    let extent = action(account, || extent.next_up())?;
    if !action(account, || extent < f64::from(f32::MAX))? {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    Ok((fused, error))
}

fn point(
    transform: &QualifiedTransform,
    source: Point,
    reason: Reason,
    account: &Account,
) -> Result<MappedPoint> {
    let x = action(account, || source.x)?;
    let y = action(account, || source.y)?;
    let native = [scalar(x, reason, account)?, scalar(y, reason, account)?];
    account.work(RenderWork::CoordinateVisits, 6)?;
    let [a, b, c, d] = transform.native.matrix;
    let [e, f] = transform.native.translation;
    let (mx, ex) = mapped_axis(a, native[0], c, native[1], e, account)?;
    let (my, ey) = mapped_axis(b, native[0], d, native[1], f, account)?;
    Ok(MappedPoint {
        source,
        native,
        mapped: [mx, my],
        error: [ex, ey],
    })
}
pub(crate) fn qualify_point(
    transform: &QualifiedTransform,
    source: Point,
    reason: Reason,
    account: &Account,
) -> Result<[f32; 2]> {
    Ok(point(transform, source, reason, account)?.native)
}
fn distinct(a: Point, b: Point, account: &Account) -> Result<bool> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let (ax, ay, bx, by) = (a.x, a.y, b.x, b.y);
    Ok(action(account, || ax != bx)? || action(account, || ay != by)?)
}
fn preserve_pair(
    a: MappedPoint,
    b: MappedPoint,
    transform: &QualifiedTransform,
    reason: Reason,
    account: &Account,
) -> Result<()> {
    let different = distinct(a.source, b.source, account)?;
    if different
        && action(account, || a.native[0] == b.native[0])?
        && action(account, || a.native[1] == b.native[1])?
    {
        return Err(reason.into());
    }
    if different && transform.nonsingular {
        let mut separated = false;
        for axis in 0..2 {
            let a64 = action(account, || f64::from(a.mapped[axis]))?;
            let b64 = action(account, || f64::from(b.mapped[axis]))?;
            let delta = action(account, || a64 - b64)?;
            let distance = action(account, || delta.abs())?;
            let distance = action(account, || distance.next_down())?;
            let error = action(account, || a.error[axis] + b.error[axis])?;
            let error = action(account, || error.next_up())?;
            if action(account, || distance > error)? {
                separated = true;
            }
        }
        if !separated {
            return Err(Reason::InsufficientNativeCapacity.into());
        }
    }
    Ok(())
}

pub(crate) fn qualify_bounds(
    transform: &QualifiedTransform,
    bounds: Rect,
    account: &Account,
) -> Result<()> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [x0, y0, x1, y1] = [bounds.x0, bounds.y0, bounds.x1, bounds.y1];
    let points = [
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ];
    let mut mapped = [None; 4];
    for (index, p) in points.into_iter().enumerate() {
        mapped[index] = Some(point(
            transform,
            p,
            Reason::InsufficientNativeCapacity,
            account,
        )?);
    }
    for index in 0..4 {
        preserve_pair(
            mapped[index].unwrap(),
            mapped[(index + 1) % 4].unwrap(),
            transform,
            Reason::InsufficientNativeCapacity,
            account,
        )?;
    }
    Ok(())
}

pub(crate) fn qualify_stroke(
    stroke: &Stroke,
    transform: &QualifiedTransform,
    account: &Account,
) -> Result<()> {
    let width = action(account, || stroke.width)?;
    if action(account, || width == 0.0)? {
        return Ok(());
    }
    let width = scalar(width, Reason::UnrepresentableStroke, account)?;
    if stroke.join == kurbo::Join::Miter {
        let miter = action(account, || stroke.miter_limit)?;
        let miter = scalar(miter, Reason::UnrepresentableStroke, account)?;
        // Half precision overflows at the rounding midpoint 65520. Positive
        // supported miter limits are >=1, so no half subnormal is required.
        if action(account, || miter >= 65520.0)? {
            return Err(Reason::UnrepresentableStroke.into());
        }
    }
    account.work(RenderWork::CoordinateVisits, 4)?;
    let [a, b, c, d] = transform.native.matrix;
    let apd = action(account, || a + d)?;
    let bmc = action(account, || b - c)?;
    let amd = action(account, || a - d)?;
    let bpc = action(account, || b + c)?;
    let length = |x: f32, y: f32| -> Result<f32> {
        finite_native(x, account)?;
        finite_native(y, account)?;
        let xx = product(x, x, account)?;
        let yy = product(y, y, account)?;
        let sum = action(account, || xx + yy)?;
        arithmetic(sum, account)?;
        let result = action(account, || sum.sqrt())?;
        arithmetic(result, account)
    };
    let l = length(apd, bmc)?;
    let r = length(amd, bpc)?;
    let sum = action(account, || l + r)?;
    finite_native(sum, account)?;
    let scale = action(account, || 0.5 * sum)?;
    finite_native(scale, account)?;
    let width = product(width, scale, account)?;
    if action(account, || width == 0.0)? && transform.nonsingular {
        return Err(Reason::InsufficientNativeCapacity.into());
    }
    Ok(())
}

/// Source interpretation used by both the empty guard and checked native feed.
/// Initial line/quad/cubic is a move to its endpoint, exactly as PathEncoder.
#[derive(Default)]
struct Cursor {
    first: Option<Point>,
    last: Option<Point>,
    ever: bool,
    at_start: bool,
}
impl Cursor {
    fn advance(&mut self, element: PathEl, account: &Account) -> Result<SourceElement> {
        let (points, count) = match element {
            PathEl::MoveTo(p) => {
                self.first = Some(p);
                self.last = Some(p);
                self.at_start = true;
                return Ok(SourceElement::Move(p));
            }
            PathEl::ClosePath => {
                self.last = if self.ever { self.first } else { None };
                self.at_start = true;
                return Ok(SourceElement::Close);
            }
            PathEl::LineTo(p) => ([p, p, p], 1),
            PathEl::QuadTo(p, q) => ([p, q, q], 2),
            PathEl::CurveTo(p, q, r) => ([p, q, r], 3),
        };
        let Some(start) = self.last else {
            let end = points[count - 1];
            self.first = Some(end);
            self.last = Some(end);
            self.at_start = true;
            return Ok(SourceElement::Move(end));
        };
        let mut nonzero = false;
        for p in &points[..count] {
            if distinct(start, *p, account)? {
                nonzero = true;
            }
        }
        if !nonzero {
            return Ok(SourceElement::Empty);
        }
        let initial = self.at_start;
        self.at_start = false;
        self.ever = true;
        self.last = Some(points[count - 1]);
        Ok(SourceElement::Segment {
            start,
            points,
            count,
            initial,
        })
    }
}
enum SourceElement {
    Move(Point),
    Close,
    Empty,
    Segment {
        start: Point,
        points: [Point; 3],
        count: usize,
        initial: bool,
    },
}

pub(crate) fn has_segments(elements: &[PathEl], account: &Account) -> Result<bool> {
    let mut cursor = Cursor::default();
    for element in elements {
        account.work(RenderWork::PathElementVisits, 1)?;
        if matches!(
            cursor.advance(*element, account)?,
            SourceElement::Segment { .. }
        ) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug)]
pub(super) enum NativeElement {
    Move([f32; 2]),
    Line([f32; 2]),
    Quad([f32; 2], [f32; 2]),
    Cubic([f32; 2], [f32; 2], [f32; 2]),
    Close,
    Empty,
}
#[derive(Default)]
pub(super) struct PathQualifier {
    cursor: Cursor,
    contour: Option<ContourExtent>,
    fill: bool,
}
// A fill requires its positive two-dimensional contour extent to survive.
// Inspect the completed contour, rather than imposing exact scalar-coordinate
// distinctions on individual controls (ordinary decimal arcs have residuals).
struct ContourExtent {
    source_min: [f64; 2],
    source_max: [f64; 2],
    native_min: [f32; 2],
    native_max: [f32; 2],
    index: usize,
}
impl ContourExtent {
    fn new(point: MappedPoint, index: usize, account: &Account) -> Result<Self> {
        account.work(RenderWork::CoordinateVisits, 4)?;
        let source = [point.source.x, point.source.y];
        let native = point.native;
        Ok(Self {
            source_min: source,
            source_max: source,
            native_min: native,
            native_max: native,
            index,
        })
    }
    fn add(&mut self, point: MappedPoint, index: usize, account: &Account) -> Result<()> {
        for axis in 0..2 {
            let source = action(account, || {
                if axis == 0 {
                    point.source.x
                } else {
                    point.source.y
                }
            })?;
            let native = action(account, || point.native[axis])?;
            self.source_min[axis] = action(account, || self.source_min[axis].min(source))?;
            self.source_max[axis] = action(account, || self.source_max[axis].max(source))?;
            self.native_min[axis] = action(account, || self.native_min[axis].min(native))?;
            self.native_max[axis] = action(account, || self.native_max[axis].max(native))?;
        }
        self.index = index;
        Ok(())
    }
    fn qualify(self, account: &Account) -> Result<()> {
        if action(account, || self.source_min[0] < self.source_max[0])?
            && action(account, || self.source_min[1] < self.source_max[1])?
            && (action(account, || self.native_min[0] == self.native_max[0])?
                || action(account, || self.native_min[1] == self.native_max[1])?)
        {
            return Err(Reason::UnrepresentablePath {
                element_index: self.index,
            }
            .into());
        }
        Ok(())
    }
}
impl PathQualifier {
    pub(super) fn new(fill: bool) -> Self {
        Self {
            fill,
            ..Self::default()
        }
    }
    pub(super) fn finish(&mut self, account: &Account) -> Result<()> {
        if let Some(contour) = self.contour.take() {
            contour.qualify(account)?;
        }
        Ok(())
    }
    pub(super) fn element(
        &mut self,
        element: PathEl,
        index: usize,
        transform: &QualifiedTransform,
        account: &Account,
    ) -> Result<NativeElement> {
        let reason = Reason::UnrepresentablePath {
            element_index: index,
        };
        match self.cursor.advance(element, account)? {
            SourceElement::Move(p) => {
                self.finish(account)?;
                Ok(NativeElement::Move(qualify_point(
                    transform, p, reason, account,
                )?))
            }
            SourceElement::Close => {
                self.finish(account)?;
                Ok(NativeElement::Close)
            }
            SourceElement::Empty => Ok(NativeElement::Empty),
            SourceElement::Segment {
                start,
                points,
                count,
                initial,
            } => {
                let start = point(transform, start, reason, account)?;
                let mut native = [start; 3];
                for i in 0..count {
                    native[i] = point(transform, points[i], reason, account)?;
                }
                // Preserve source-distinct vector controls. One coordinate may
                // round together while another still separates the points.
                for i in 0..count {
                    preserve_pair(start, native[i], transform, reason, account)?;
                    for j in 0..i {
                        preserve_pair(native[j], native[i], transform, reason, account)?;
                    }
                }
                if self.fill {
                    if self.contour.is_none() {
                        self.contour = Some(ContourExtent::new(start, index, account)?);
                    }
                    for p in &native[..count] {
                        self.contour.as_mut().unwrap().add(*p, index, account)?;
                    }
                }
                let mut xmin = start.native[0];
                let mut xmax = xmin;
                let mut ymin = start.native[1];
                let mut ymax = ymin;
                let mut tangent = false;
                let mut tangent_index = 0;
                for p in &native[..count] {
                    xmin = action(account, || xmin.min(p.native[0]))?;
                    xmax = action(account, || xmax.max(p.native[0]))?;
                    ymin = action(account, || ymin.min(p.native[1]))?;
                    ymax = action(account, || ymax.max(p.native[1]))?;
                    if initial && !tangent {
                        let dx = action(account, || p.native[0] - start.native[0])?;
                        let dx = action(account, || dx.abs())?;
                        tangent = action(account, || dx > 1e-12)?;
                        if !tangent {
                            let dy = action(account, || p.native[1] - start.native[1])?;
                            let dy = action(account, || dy.abs())?;
                            tangent = action(account, || dy > 1e-12)?;
                        }
                        if !tangent {
                            tangent_index += 1;
                        }
                    }
                }
                let dx = action(account, || xmax - xmin)?;
                finite_native(dx, account)?;
                let dy = action(account, || ymax - ymin)?;
                finite_native(dy, account)?;
                let retained = action(account, || dx > 1e-12)? || action(account, || dy > 1e-12)?;
                if !retained || (initial && !tangent) {
                    return Err(reason.into());
                }
                // The pinned first tangent interpolations use differences/thirds.
                // Bound their arithmetic domain without replacing their output.
                if initial && count < 3 {
                    for axis in 0..2 {
                        let (base, other) = if count == 1 {
                            (start.native[axis], native[0].native[axis])
                        } else {
                            (
                                native[0].native[axis],
                                if tangent_index == 0 {
                                    start.native[axis]
                                } else {
                                    native[1].native[axis]
                                },
                            )
                        };
                        let delta = action(account, || other - base)?;
                        finite_native(delta, account)?;
                        let third = action(account, || delta * (1.0 / 3.0))?;
                        finite_native(third, account)?;
                        let end = action(account, || base + third)?;
                        finite_native(end, account)?;
                    }
                }
                Ok(match count {
                    1 => NativeElement::Line(native[0].native),
                    2 => NativeElement::Quad(native[0].native, native[1].native),
                    _ => NativeElement::Cubic(native[0].native, native[1].native, native[2].native),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase, RenderResource,
    };
    fn account() -> Account {
        Account::for_test()
    }
    fn diagnostic(error: crate::Error) -> Reason {
        *error
            .geometry_realization_diagnostic()
            .expect("geometry diagnostic")
    }

    #[test]
    fn scalar_normal_action_count_and_one_short_prefix_are_independent() {
        let mut context = RenderAccountingContext::default();
        assert_eq!(
            context
                .run(RenderAttemptKind::Frame, RenderPhase::Encode, |a| scalar(
                    0.1,
                    Reason::UnrepresentableTransform,
                    a
                ))
                .unwrap(),
            0.1_f32
        );
        // finite source, cast, finite native, source nonzero, native zero,
        // native normal: six actual actions, including the successful branch.
        assert_eq!(
            context
                .last_attempt()
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
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |a| {
                scalar(0.1, Reason::UnrepresentableTransform, a)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), 5);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(
            scalar(0.1, Reason::UnrepresentableTransform, &account()).unwrap(),
            0.1_f32
        );
    }
    #[test]
    fn scalar_preserves_ordinary_rounding_and_classifies_loss_vs_ftz() {
        assert_eq!(
            scalar(0.1, Reason::UnrepresentableTransform, &account()).unwrap(),
            0.1_f32
        );
        assert_eq!(
            scalar(0.0, Reason::UnrepresentableTransform, &account()).unwrap(),
            0.0
        );
        for value in [f64::MAX, f64::MIN_POSITIVE] {
            assert_eq!(
                diagnostic(
                    scalar(value, Reason::UnrepresentableTransform, &account()).unwrap_err()
                ),
                Reason::UnrepresentableTransform
            );
        }
        assert_eq!(
            diagnostic(
                scalar(
                    f64::from(f32::from_bits(1)),
                    Reason::UnrepresentableTransform,
                    &account()
                )
                .unwrap_err()
            ),
            Reason::InsufficientNativeCapacity
        );
    }
    #[test]
    fn native_rank_cannot_drop_a_source_dimension() {
        let source = Affine::new([
            1.0,
            1.0,
            1.0,
            f64::from_bits(1.0_f64.to_bits() + 1),
            0.0,
            0.0,
        ]);
        assert_eq!(
            diagnostic(
                qualify_transform(source, &account())
                    .err()
                    .expect("lost rank")
            ),
            Reason::InsufficientNativeCapacity
        );
        assert!(qualify_transform(Affine::new([1.0, 1.0, 1.0, 1.0, 0.0, 0.0]), &account()).is_ok());
    }
    #[test]
    fn products_match_actual_locked_dependency_association() {
        for (left, right) in [
            (
                Affine::new([1.25, -0.5, 0.125, 2.0, 7.0, -3.0]),
                Affine::new([-2.0, 0.75, 1.5, 0.25, -4.0, 11.0]),
            ),
            (
                Affine::new([1.0, 0.0, 0.0, -1.0, 16_777_216.0, 0.0]),
                Affine::translate((-16_777_215.0, 0.125)),
            ),
        ] {
            let actual = affine_product(left, right, &account()).unwrap();
            assert_eq!(
                actual.as_coeffs().map(f64::to_bits),
                (left * right).as_coeffs().map(f64::to_bits)
            );
            let l = Transform::from_kurbo(&left);
            let r = Transform::from_kurbo(&right);
            let actual = native_product(l, r, &account()).unwrap();
            let expected = l * r;
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
    fn empty_interpretation_preserves_initial_move_and_true_points() {
        use PathEl::*;
        for elements in [
            vec![],
            vec![MoveTo(Point::new(f64::MAX, 0.0))],
            vec![QuadTo(Point::new(f64::MAX, 0.0), Point::ZERO)],
            vec![MoveTo(Point::ZERO), LineTo(Point::ZERO), ClosePath],
        ] {
            assert!(!has_segments(&elements, &account()).unwrap());
        }
        assert!(
            has_segments(
                &[LineTo(Point::ZERO), LineTo(Point::new(1.0, 0.0))],
                &account()
            )
            .unwrap()
        );
    }
    #[test]
    fn local_alias_and_locked_epsilon_retention_refuse_at_source_index() {
        use PathEl::*;
        for elements in [
            [
                MoveTo(Point::new(16_777_216.0, 0.0)),
                LineTo(Point::new(16_777_217.0, 0.0)),
            ],
            [
                MoveTo(Point::ZERO),
                LineTo(Point::new(2.0_f64.powi(-43), 0.0)),
            ],
            [MoveTo(Point::ZERO), LineTo(Point::new(f64::MAX, 0.0))],
        ] {
            let transform = qualify_transform(Affine::IDENTITY, &account()).unwrap();
            let mut q = PathQualifier::default();
            q.element(elements[0], 0, &transform, &account()).unwrap();
            let error = match q.element(elements[1], 1, &transform, &account()) {
                Err(error) => error,
                Ok(_) => panic!("retention loss"),
            };
            assert_eq!(
                diagnostic(error),
                Reason::UnrepresentablePath { element_index: 1 }
            );
        }
    }
    #[test]
    fn composed_point_domain_catches_overflow_and_underflow_even_with_finite_inputs() {
        let transform = qualify_transform(Affine::scale(1e30), &account()).unwrap();
        assert_eq!(
            diagnostic(
                qualify_point(
                    &transform,
                    Point::new(1e20, 1.0),
                    Reason::UnrepresentablePath { element_index: 0 },
                    &account()
                )
                .unwrap_err()
            ),
            Reason::InsufficientNativeCapacity
        );
        let transform =
            qualify_transform(Affine::scale(f64::from(f32::MIN_POSITIVE)), &account()).unwrap();
        assert_eq!(
            diagnostic(
                qualify_point(
                    &transform,
                    Point::new(f64::from(f32::MIN_POSITIVE), 1.0),
                    Reason::UnrepresentablePath { element_index: 0 },
                    &account()
                )
                .unwrap_err()
            ),
            Reason::InsufficientNativeCapacity
        );
    }
    #[test]
    fn checked_line_quad_cubic_close_streams_match_real_encoder() {
        use PathEl::*;
        let cases = [
            vec![LineTo(Point::new(3.0, 4.0)), LineTo(Point::new(5.0, 6.0))],
            vec![
                MoveTo(Point::new(0.1, 0.2)),
                QuadTo(Point::new(1.1, 2.2), Point::new(3.3, 0.4)),
                ClosePath,
            ],
            vec![
                CurveTo(
                    Point::new(9.0, 9.0),
                    Point::new(8.0, 8.0),
                    Point::new(1.0, 2.0),
                ),
                CurveTo(
                    Point::new(2.0, 3.0),
                    Point::new(4.0, 5.0),
                    Point::new(6.0, 7.0),
                ),
            ],
            vec![
                MoveTo(Point::ZERO),
                QuadTo(Point::new(1e-13, 0.0), Point::new(2.0, 3.0)),
                ClosePath,
                LineTo(Point::new(4.0, 2.0)),
            ],
        ];
        for elements in cases {
            for fill in [true, false] {
                for marker in [true, false] {
                    let mut expected = vello_encoding::Encoding::default();
                    let mut p = expected.encode_path(fill);
                    p.path_elements(elements.iter().copied());
                    p.finish(marker);
                    let mut actual = vello_encoding::Encoding::default();
                    let transform = qualify_transform(Affine::IDENTITY, &account()).unwrap();
                    super::super::bounded::encode_elements_with_marker(
                        &mut actual,
                        &elements,
                        fill,
                        marker,
                        &transform,
                        &account(),
                    )
                    .unwrap();
                    assert_eq!(actual.path_data, expected.path_data);
                    assert_eq!(
                        actual.path_tags.iter().map(|tag| tag.0).collect::<Vec<_>>(),
                        expected
                            .path_tags
                            .iter()
                            .map(|tag| tag.0)
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(actual.n_paths, expected.n_paths);
                    assert_eq!(actual.n_path_segments, expected.n_path_segments);
                }
            }
        }
    }
    #[test]
    fn rounded_decimal_contour_and_small_single_axis_residual_keep_native_streams() {
        use kurbo::Shape;
        let rounded = crate::command::kurbo_rounded_rect(
            crate::Rect::try_new(2.1, 2.1, 27.8, 11.8).unwrap(),
            crate::Radii::try_all(2.3).unwrap(),
        );
        let elements = rounded.path_elements(0.1).collect::<Vec<_>>();
        for fill in [true, false] {
            let mut expected = vello_encoding::Encoding::default();
            let mut path = expected.encode_path(fill);
            path.path_elements(elements.iter().copied());
            path.finish(true);
            let mut actual = vello_encoding::Encoding::default();
            let transform = qualify_transform(Affine::IDENTITY, &account()).unwrap();
            assert!(
                super::super::bounded::encode_elements(
                    &mut actual,
                    &elements,
                    fill,
                    &transform,
                    &account()
                )
                .unwrap()
            );
            assert_eq!(actual.path_data, expected.path_data);
            assert_eq!(
                actual.path_tags.iter().map(|t| t.0).collect::<Vec<_>>(),
                expected.path_tags.iter().map(|t| t.0).collect::<Vec<_>>()
            );
        }
        let transform = qualify_transform(Affine::IDENTITY, &account()).unwrap();
        // This is ordinary coordinate rounding of a nearly horizontal stroke,
        // rather than a collapsed vector endpoint or a filled contour extent.
        let a = point(
            &transform,
            Point::new(2.1, 2.0999999999999996),
            Reason::UnrepresentablePath { element_index: 0 },
            &account(),
        )
        .unwrap();
        let b = point(
            &transform,
            Point::new(4.4, 2.1),
            Reason::UnrepresentablePath { element_index: 1 },
            &account(),
        )
        .unwrap();
        assert_eq!(a.native[1], b.native[1]);
        preserve_pair(
            a,
            b,
            &transform,
            Reason::UnrepresentablePath { element_index: 1 },
            &account(),
        )
        .unwrap();
    }
    #[test]
    fn complete_filled_contour_cannot_lose_an_axis_when_vector_controls_survive() {
        let elements = [
            PathEl::MoveTo(Point::new(16_777_216.0, 0.0)),
            PathEl::LineTo(Point::new(16_777_217.0, 1.0)),
            PathEl::LineTo(Point::new(16_777_216.0, 2.0)),
            PathEl::ClosePath,
        ];
        let transform = qualify_transform(Affine::IDENTITY, &account()).unwrap();
        let mut encoding = vello_encoding::Encoding::default();
        let error = super::super::bounded::encode_elements(
            &mut encoding,
            &elements,
            true,
            &transform,
            &account(),
        )
        .unwrap_err();
        assert_eq!(
            diagnostic(error),
            Reason::UnrepresentablePath { element_index: 2 }
        );
    }
    #[test]
    fn stroke_qualifies_required_width_miter_and_actual_scale_domain() {
        let identity = qualify_transform(Affine::IDENTITY, &account()).unwrap();
        assert!(qualify_stroke(&Stroke::new(0.1), &identity, &account()).is_ok());
        assert_eq!(
            diagnostic(
                qualify_stroke(&Stroke::new(f64::MIN_POSITIVE), &identity, &account()).unwrap_err()
            ),
            Reason::UnrepresentableStroke
        );
        assert_eq!(
            diagnostic(
                qualify_stroke(
                    &Stroke::new(1.0)
                        .with_join(kurbo::Join::Miter)
                        .with_miter_limit(65520.0),
                    &identity,
                    &account()
                )
                .unwrap_err()
            ),
            Reason::UnrepresentableStroke
        );
        let huge = qualify_transform(Affine::scale(1e20), &account()).unwrap();
        assert_eq!(
            diagnostic(qualify_stroke(&Stroke::new(1.0), &huge, &account()).unwrap_err()),
            Reason::InsufficientNativeCapacity
        );
    }
}
