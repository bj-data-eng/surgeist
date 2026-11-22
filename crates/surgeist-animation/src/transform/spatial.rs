//! Column-major homogeneous algebra and the selected quaternion policy.

use super::{
    Combination, CssPixels, RotationAxis, ScaleFactor, TransformAngle, TransformComponent as C,
    TransformError, TransformFunction as F, TransformMatrix3d as M, combine_number, finite,
};

fn check(x: f64, c: C) -> Result<f64, TransformError> {
    finite(x, c)
}
fn sum_products(a: &[f64], b: &[f64], c: C) -> Result<f64, TransformError> {
    let mut sum = 0.;
    for (a, b) in a.iter().zip(b) {
        sum = check(sum + check(a * b, c)?, c)?;
    }
    Ok(sum)
}
fn multiply(a: M, b: M) -> Result<M, TransformError> {
    let mut result = [0.; 16];
    for column in 0..4 {
        for row in 0..4 {
            result[column * 4 + row] = sum_products(
                &[a.0[row], a.0[4 + row], a.0[8 + row], a.0[12 + row]],
                &b.0[column * 4..column * 4 + 4],
                C::MatrixCoefficient(column * 4 + row),
            )?;
        }
    }
    M::new(result)
}
fn quaternion_matrix(q: [f64; 4]) -> Result<M, TransformError> {
    let [x, y, z, w] = q;
    M::new([
        1. - 2. * (y * y + z * z),
        2. * (x * y + z * w),
        2. * (x * z - y * w),
        0.,
        2. * (x * y - z * w),
        1. - 2. * (x * x + z * z),
        2. * (y * z + x * w),
        0.,
        2. * (x * z + y * w),
        2. * (y * z - x * w),
        1. - 2. * (x * x + y * y),
        0.,
        0.,
        0.,
        0.,
        1.,
    ])
}

pub(super) fn function_matrix(function: F) -> Result<M, TransformError> {
    let mut m = M::IDENTITY.0;
    match function {
        F::Matrix3d(value) => return Ok(value),
        F::Translate3d { x, y, z } => {
            m[12] = x.0;
            m[13] = y.0;
            m[14] = z.0;
        }
        F::TranslateZ(z) => m[14] = z.0,
        F::Scale3d { x, y, z } => {
            m[0] = x.0;
            m[5] = y.0;
            m[10] = z.0;
        }
        F::ScaleZ(z) => m[10] = z.0,
        F::Perspective(depth) => {
            if let Some(depth) = depth {
                m[11] = -1. / depth.0.max(1.);
            }
        }
        F::RotateX(angle) => return rotation_matrix(RotationAxis([1., 0., 0.]), angle),
        F::RotateY(angle) => return rotation_matrix(RotationAxis([0., 1., 0.]), angle),
        F::RotateZ(angle) => return rotation_matrix(RotationAxis([0., 0., 1.]), angle),
        F::Rotate3d { axis, angle } => return rotation_matrix(axis, angle),
        other => return super::math::function_matrix(other).map(M::from),
    }
    M::new(m)
}
fn rotation_matrix(axis: RotationAxis, angle: TransformAngle) -> Result<M, TransformError> {
    if axis.0 == [0.; 3] {
        return Ok(M::IDENTITY);
    }
    let (sin, cos) = (angle.0 / 2.).sin_cos();
    quaternion_matrix([axis.0[0] * sin, axis.0[1] * sin, axis.0[2] * sin, cos])
}
pub(super) fn list_matrix(functions: &[F], start: usize) -> Result<M, TransformError> {
    let mut result = M::IDENTITY;
    for (offset, function) in functions.iter().enumerate() {
        result = function_matrix(*function)
            .and_then(|next| multiply(result, next))
            .map_err(|error| error.at_index(start + offset))?;
    }
    Ok(result)
}

// Exact determinant signs use fixed-size unsigned accumulators. Four binary64
// significands need at most 212 product bits. Their exponents span -4296..3884;
// 132 limbs cover every shifted product and the carry from all 24 terms.
fn significand(value: f64) -> (u64, i32) {
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    if exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), exponent - 1075)
    }
}
fn add_word(sum: &mut [u64; 132], mut index: usize, mut value: u64) {
    while value != 0 && index < sum.len() {
        let (next, carry) = sum[index].overflowing_add(value);
        sum[index] = next;
        value = u64::from(carry);
        index += 1;
    }
}
fn exact_sign(matrix: M) -> core::cmp::Ordering {
    let mut positive = [0_u64; 132];
    let mut negative = [0_u64; 132];
    for a in 0..4 {
        for b in 0..4 {
            if a == b {
                continue;
            }
            for c in 0..4 {
                if c == a || c == b {
                    continue;
                }
                for d in 0..4 {
                    if d == a || d == b || d == c {
                        continue;
                    }
                    let permutation = [a, b, c, d];
                    let mut minus = false;
                    for i in 0..4 {
                        for j in i + 1..4 {
                            minus ^= permutation[i] > permutation[j];
                        }
                    }
                    let mut limbs = [1_u64, 0, 0, 0];
                    let mut exponent = 0;
                    for (row, column) in permutation.into_iter().enumerate() {
                        let value = matrix.0[column * 4 + row];
                        minus ^= value.to_bits() >> 63 != 0;
                        let (mantissa, power) = significand(value);
                        exponent += power;
                        let mut carry = 0_u128;
                        for limb in &mut limbs {
                            let product = u128::from(*limb) * u128::from(mantissa) + carry;
                            *limb = product as u64;
                            carry = product >> 64;
                        }
                    }
                    let shift = (exponent + 4296) as usize;
                    let index = shift / 64;
                    let bits = shift % 64;
                    let sum = if minus { &mut negative } else { &mut positive };
                    for (offset, limb) in limbs.into_iter().enumerate() {
                        add_word(sum, index + offset, limb << bits);
                        if bits != 0 {
                            add_word(sum, index + offset + 1, limb >> (64 - bits));
                        }
                    }
                }
            }
        }
    }
    positive.iter().rev().cmp(negative.iter().rev())
}
fn perspective_base(matrix: M) -> M {
    let mut p = matrix;
    p.0[3] = 0.;
    p.0[7] = 0.;
    p.0[11] = 0.;
    p.0[15] = 1.;
    p
}
pub(super) fn decomposable(matrix: M) -> bool {
    matrix.0[15] != 0.
        && !exact_sign(matrix).is_eq()
        && !exact_sign(perspective_base(matrix)).is_eq()
}

fn unit3(v: [f64; 3], component: C) -> Result<(f64, [f64; 3]), TransformError> {
    let magnitude = v.into_iter().map(f64::abs).fold(0., f64::max);
    let scaled = [
        check(v[0] / magnitude, component)?,
        check(v[1] / magnitude, component)?,
        check(v[2] / magnitude, component)?,
    ];
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
    Ok((
        check(magnitude * length, component)?,
        scaled.map(|x| x / length),
    ))
}
fn unit4(v: [f64; 4]) -> Result<[f64; 4], TransformError> {
    let magnitude = v.into_iter().map(f64::abs).fold(0., f64::max);
    let mut scaled = [0.; 4];
    for i in 0..4 {
        scaled[i] = check(v[i] / magnitude, C::Quaternion(i))?;
    }
    let length = scaled[0].hypot(scaled[1]).hypot(scaled[2]).hypot(scaled[3]);
    Ok(scaled.map(|x| x / length))
}
fn norm4(v: [f64; 4]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2]).hypot(v[3])
}
fn subtract_projection(
    v: [f64; 3],
    unit: [f64; 3],
    amount: f64,
    component: C,
) -> Result<[f64; 3], TransformError> {
    let mut result = [0.; 3];
    for i in 0..3 {
        result[i] = check(v[i] - check(unit[i] * amount, component)?, component)?;
    }
    Ok(result)
}
fn solve_perspective(matrix: M) -> Result<[f64; 4], TransformError> {
    let mut rows = [[0.; 4]; 3];
    for (i, row) in rows.iter_mut().enumerate() {
        *row = [
            matrix.0[i * 4],
            matrix.0[i * 4 + 1],
            matrix.0[i * 4 + 2],
            matrix.0[i * 4 + 3],
        ];
    }
    for column in 0..3 {
        let mut pivot = column;
        for row in column + 1..3 {
            if rows[row][column].abs() > rows[pivot][column].abs() {
                pivot = row;
            }
        }
        rows.swap(pivot, column);
        let divisor = rows[column][column];
        for (j, value) in rows[column].iter_mut().enumerate().skip(column) {
            *value = check(*value / divisor, C::Perspective(j))?;
        }
        let pivot_row = rows[column];
        for (row_index, row) in rows.iter_mut().enumerate() {
            if row_index == column {
                continue;
            }
            let factor = row[column];
            for (j, value) in row.iter_mut().enumerate().skip(column) {
                *value = check(
                    *value - check(factor * pivot_row[j], C::Perspective(j))?,
                    C::Perspective(j),
                )?;
            }
        }
    }
    let p = [rows[0][3], rows[1][3], rows[2][3]];
    let w = check(
        1. - sum_products(&matrix.0[12..15], &p, C::Perspective(3))?,
        C::Perspective(3),
    )?;
    Ok([p[0], p[1], p[2], w])
}

struct Decomposed {
    translation: [f64; 3],
    scale: [f64; 3],
    skew: [f64; 3],
    perspective: [f64; 4],
    quaternion: [f64; 4],
}
fn decompose(matrix: M) -> Result<Decomposed, TransformError> {
    let divisor = matrix.0[15];
    let mut normalized = [0.; 16];
    for (i, value) in normalized.iter_mut().enumerate() {
        *value = check(matrix.0[i] / divisor, C::MatrixCoefficient(i))?;
    }
    let matrix = M(normalized);
    let perspective = if matrix.0[3] == 0. && matrix.0[7] == 0. && matrix.0[11] == 0. {
        [0., 0., 0., 1.]
    } else {
        solve_perspective(matrix)?
    };
    let translation = [matrix.0[12], matrix.0[13], matrix.0[14]];
    let (sx, mut u0) = unit3([matrix.0[0], matrix.0[1], matrix.0[2]], C::ScaleX)?;
    let v1 = [matrix.0[4], matrix.0[5], matrix.0[6]];
    let xy = sum_products(&u0, &v1, C::Skew(0))?;
    let (sy, mut u1) = unit3(subtract_projection(v1, u0, xy, C::Skew(0))?, C::ScaleY)?;
    let v2 = [matrix.0[8], matrix.0[9], matrix.0[10]];
    let xz = sum_products(&u0, &v2, C::Skew(1))?;
    let v2 = subtract_projection(v2, u0, xz, C::Skew(1))?;
    let yz = sum_products(&u1, &v2, C::Skew(2))?;
    let (sz, mut u2) = unit3(subtract_projection(v2, u1, yz, C::Skew(2))?, C::ScaleZ)?;
    let mut scale = [sx, sy, sz];
    if exact_sign(perspective_base(matrix)).is_lt() {
        scale = scale.map(|x| -x);
        u0 = u0.map(|x| -x);
        u1 = u1.map(|x| -x);
        u2 = u2.map(|x| -x);
    }
    let skew = [
        check(xy / sy, C::Skew(0))?,
        check(xz / sz, C::Skew(1))?,
        check(yz / sz, C::Skew(2))?,
    ];
    Ok(Decomposed {
        translation,
        scale,
        skew,
        perspective,
        quaternion: extract_quaternion([u0, u1, u2])?,
    })
}
fn extract_quaternion(c: [[f64; 3]; 3]) -> Result<[f64; 4], TransformError> {
    let candidates = [
        1. + c[0][0] - c[1][1] - c[2][2],
        1. - c[0][0] + c[1][1] - c[2][2],
        1. - c[0][0] - c[1][1] + c[2][2],
        1. + c[0][0] + c[1][1] + c[2][2],
    ];
    let mut largest = 0;
    for i in 1..4 {
        if candidates[i] > candidates[largest] {
            largest = i;
        }
    }
    let main = 0.5 * candidates[largest].max(0.).sqrt();
    let denominator = 4. * main;
    let q = match largest {
        0 => [
            main,
            (c[0][1] + c[1][0]) / denominator,
            (c[0][2] + c[2][0]) / denominator,
            (c[1][2] - c[2][1]) / denominator,
        ],
        1 => [
            (c[0][1] + c[1][0]) / denominator,
            main,
            (c[1][2] + c[2][1]) / denominator,
            (c[2][0] - c[0][2]) / denominator,
        ],
        2 => [
            (c[0][2] + c[2][0]) / denominator,
            (c[1][2] + c[2][1]) / denominator,
            main,
            (c[0][1] - c[1][0]) / denominator,
        ],
        _ => [
            (c[1][2] - c[2][1]) / denominator,
            (c[2][0] - c[0][2]) / denominator,
            (c[0][1] - c[1][0]) / denominator,
            main,
        ],
    };
    let mut q = unit4(q)?;
    let flip = if q[3] != 0. {
        q[3] < 0.
    } else {
        let mut dominant = 0;
        for i in 1..3 {
            if q[i].abs() > q[dominant].abs() {
                dominant = i;
            }
        }
        q[dominant] < 0.
    };
    if flip {
        q = q.map(|x| -x);
    }
    if q[3] == 0. {
        q[3] = 0.;
    }
    Ok(q)
}
fn slerp(a: [f64; 4], b: [f64; 4], p: f64) -> Result<[f64; 4], TransformError> {
    if a == b || a == b.map(|x| -x) {
        return Ok(a);
    }
    let sum = [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
    let difference = [b[0] - a[0], b[1] - a[1], b[2] - a[2], b[3] - a[3]];
    // Half-sum and half-difference form the angular basis without relying on a
    // rounded dot product. The midpoint remains stable near antipodal inputs.
    let angle = 2. * norm4(difference).atan2(norm4(sum));
    let u = unit4(sum)?;
    let v = unit4(difference)?;
    let phase = check(check(p - 0.5, C::Quaternion(3))? * angle, C::Quaternion(3))?;
    let (sin, cos) = phase.sin_cos();
    unit4([
        u[0] * cos + v[0] * sin,
        u[1] * cos + v[1] * sin,
        u[2] * cos + v[2] * sin,
        u[3] * cos + v[3] * sin,
    ])
}
fn quaternion_product(a: [f64; 4], b: [f64; 4]) -> Result<[f64; 4], TransformError> {
    let [x, y, z, w] = a;
    let [i, j, k, l] = b;
    unit4([
        w * i + x * l + y * k - z * j,
        w * j - x * k + y * l + z * i,
        w * k + x * j - y * i + z * l,
        w * l - x * i - y * j - z * k,
    ])
}
fn recompose(value: Decomposed) -> Result<M, TransformError> {
    let mut m = M::IDENTITY;
    m.0[3] = value.perspective[0];
    m.0[7] = value.perspective[1];
    m.0[11] = value.perspective[2];
    m.0[15] = value.perspective[3];
    let mut translation = M::IDENTITY;
    translation.0[12..15].copy_from_slice(&value.translation);
    m = multiply(m, translation)?;
    m = multiply(m, quaternion_matrix(value.quaternion)?)?;
    for (index, slot) in [(2, 9), (1, 8), (0, 4)] {
        let mut skew = M::IDENTITY;
        skew.0[slot] = value.skew[index];
        m = multiply(m, skew)?;
    }
    for column in 0..3 {
        for row in 0..4 {
            let index = column * 4 + row;
            m.0[index] = check(
                m.0[index] * value.scale[column],
                C::MatrixCoefficient(index),
            )?;
        }
    }
    M::new(m.0)
}
pub(super) fn combine_matrix(a: M, b: M, operation: Combination) -> Result<M, TransformError> {
    let a = decompose(a)?;
    let b = decompose(b)?;
    let mut result = Decomposed {
        translation: [0.; 3],
        scale: [0.; 3],
        skew: [0.; 3],
        perspective: [0.; 4],
        quaternion: [0.; 4],
    };
    for i in 0..3 {
        result.translation[i] = combine_number(
            a.translation[i],
            b.translation[i],
            operation,
            false,
            [C::TranslationX, C::TranslationY, C::TranslationZ][i],
        )?;
        result.scale[i] = combine_number(
            a.scale[i],
            b.scale[i],
            operation,
            true,
            [C::ScaleX, C::ScaleY, C::ScaleZ][i],
        )?;
        result.skew[i] = combine_number(a.skew[i], b.skew[i], operation, false, C::Skew(i))?;
    }
    for i in 0..4 {
        result.perspective[i] = combine_number(
            a.perspective[i],
            b.perspective[i],
            operation,
            i == 3,
            C::Perspective(i),
        )?;
    }
    result.quaternion = match operation {
        Combination::Interpolate(p) => slerp(a.quaternion, b.quaternion, p)?,
        Combination::Accumulate => quaternion_product(a.quaternion, b.quaternion)?,
    };
    recompose(result)
}

enum Shape {
    Translate3d,
    TranslateZ,
    Scale3d,
    ScaleZ,
    RotateX,
    RotateY,
    RotateZ,
    Rotate3d(RotationAxis),
}
pub(super) struct FunctionPair {
    shape: Shape,
    a: [f64; 3],
    b: [f64; 3],
}
fn translation(function: F) -> Option<[f64; 3]> {
    match function {
        F::Translate3d { x, y, z } => Some([x.0, y.0, z.0]),
        F::TranslateZ(z) => Some([0., 0., z.0]),
        other => other.translation().map(|(x, y)| [x.0, y.0, 0.]),
    }
}
fn scale(function: F) -> Option<[f64; 3]> {
    match function {
        F::Scale3d { x, y, z } => Some([x.0, y.0, z.0]),
        F::ScaleZ(z) => Some([1., 1., z.0]),
        other => other.scale().map(|(x, y)| [x.0, y.0, 1.]),
    }
}
fn rotation(function: F) -> Option<(RotationAxis, TransformAngle)> {
    match function {
        F::Rotate(a) | F::RotateZ(a) => Some((RotationAxis([0., 0., 1.]), a)),
        F::RotateX(a) => Some((RotationAxis([1., 0., 0.]), a)),
        F::RotateY(a) => Some((RotationAxis([0., 1., 0.]), a)),
        F::Rotate3d { axis, angle } => Some(if axis.0 == [0.; 3] {
            (RotationAxis([0., 0., 1.]), TransformAngle::ZERO)
        } else {
            (axis, angle)
        }),
        _ => None,
    }
}
pub(super) fn function_pair(a: F, b: F) -> Option<FunctionPair> {
    let (shape, a, b) = match (a, b) {
        (F::TranslateZ(a), F::TranslateZ(b)) => (Shape::TranslateZ, [a.0, 0., 0.], [b.0, 0., 0.]),
        (F::ScaleZ(a), F::ScaleZ(b)) => (Shape::ScaleZ, [a.0, 0., 0.], [b.0, 0., 0.]),
        (F::RotateX(a), F::RotateX(b)) => (Shape::RotateX, [a.0, 0., 0.], [b.0, 0., 0.]),
        (F::RotateY(a), F::RotateY(b)) => (Shape::RotateY, [a.0, 0., 0.], [b.0, 0., 0.]),
        (F::RotateZ(a), F::RotateZ(b)) => (Shape::RotateZ, [a.0, 0., 0.], [b.0, 0., 0.]),
        _ => {
            if let (Some(a), Some(b)) = (translation(a), translation(b)) {
                (Shape::Translate3d, a, b)
            } else if let (Some(a), Some(b)) = (scale(a), scale(b)) {
                (Shape::Scale3d, a, b)
            } else if let (Some((aa, an)), Some((ba, bn))) = (rotation(a), rotation(b)) {
                if aa != ba && an.0 != 0. && bn.0 != 0. {
                    return None;
                }
                let axis = if an.0 != 0. {
                    aa
                } else if bn.0 != 0. {
                    ba
                } else {
                    RotationAxis([0., 0., 1.])
                };
                (Shape::Rotate3d(axis), [an.0, 0., 0.], [bn.0, 0., 0.])
            } else {
                return None;
            }
        }
    };
    Some(FunctionPair { shape, a, b })
}
pub(super) fn matching_matrix_functions(a: F, b: F) -> bool {
    matches!(
        (a, b),
        (F::Matrix3d(_), F::Matrix3d(_)) | (F::Perspective(_), F::Perspective(_))
    ) || (rotation(a).is_some() && rotation(b).is_some())
}
impl FunctionPair {
    pub(super) fn sample(self, operation: Combination) -> Result<F, TransformError> {
        let (components, one_based) = match self.shape {
            Shape::Translate3d => ([C::TranslationX, C::TranslationY, C::TranslationZ], false),
            Shape::TranslateZ => ([C::TranslationZ; 3], false),
            Shape::Scale3d => ([C::ScaleX, C::ScaleY, C::ScaleZ], true),
            Shape::ScaleZ => ([C::ScaleZ; 3], true),
            _ => ([C::Angle; 3], false),
        };
        let mut v = [0.; 3];
        for i in 0..3 {
            v[i] = combine_number(self.a[i], self.b[i], operation, one_based, components[i])?;
        }
        Ok(match self.shape {
            Shape::Translate3d => F::Translate3d {
                x: CssPixels(v[0]),
                y: CssPixels(v[1]),
                z: CssPixels(v[2]),
            },
            Shape::TranslateZ => F::TranslateZ(CssPixels(v[0])),
            Shape::Scale3d => F::Scale3d {
                x: ScaleFactor(v[0]),
                y: ScaleFactor(v[1]),
                z: ScaleFactor(v[2]),
            },
            Shape::ScaleZ => F::ScaleZ(ScaleFactor(v[0])),
            Shape::RotateX => F::RotateX(TransformAngle(v[0])),
            Shape::RotateY => F::RotateY(TransformAngle(v[0])),
            Shape::RotateZ => F::RotateZ(TransformAngle(v[0])),
            Shape::Rotate3d(axis) => F::Rotate3d {
                axis,
                angle: TransformAngle(v[0]),
            },
        })
    }
}
