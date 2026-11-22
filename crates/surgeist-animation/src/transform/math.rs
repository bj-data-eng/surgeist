//! Checked affine arithmetic and the selected CSS Transforms 1 algorithms.

use super::{
    CssPixels, ScaleFactor, TransformAngle, TransformComponent as C, TransformError,
    TransformFunction as F, TransformMatrix, finite,
};

fn checked(value: f64, component: C) -> Result<f64, TransformError> {
    finite(value, component)
}

fn lerp(a: f64, b: f64, p: f64, component: C) -> Result<f64, TransformError> {
    let delta = checked(b - a, component)?;
    let scaled = checked(delta * p, component)?;
    checked(a + scaled, component)
}

fn dot(a: f64, b: f64, c: f64, d: f64, component: C) -> Result<f64, TransformError> {
    checked(
        checked(a * b, component)? + checked(c * d, component)?,
        component,
    )
}

fn multiply(a: TransformMatrix, b: TransformMatrix) -> Result<TransformMatrix, TransformError> {
    let [a, b0, c, d, e, f] = a.coefficients();
    let [g, h, i, j, k, l] = b.coefficients();
    TransformMatrix::new([
        dot(a, g, c, h, C::MatrixCoefficient(0))?,
        dot(b0, g, d, h, C::MatrixCoefficient(1))?,
        dot(a, i, c, j, C::MatrixCoefficient(2))?,
        dot(b0, i, d, j, C::MatrixCoefficient(3))?,
        checked(
            dot(a, k, c, l, C::MatrixCoefficient(4))? + e,
            C::MatrixCoefficient(4),
        )?,
        checked(
            dot(b0, k, d, l, C::MatrixCoefficient(5))? + f,
            C::MatrixCoefficient(5),
        )?,
    ])
}

fn function_matrix(function: F) -> Result<TransformMatrix, TransformError> {
    match function {
        F::Matrix(matrix) => Ok(matrix),
        F::Rotate(angle) => {
            let (sin, cos) = angle.radians().sin_cos();
            TransformMatrix::new([cos, sin, -sin, cos, 0., 0.])
        }
        F::Skew { x, y } => TransformMatrix::new([
            1.,
            y.unwrap_or(TransformAngle::ZERO).radians().tan(),
            x.radians().tan(),
            1.,
            0.,
            0.,
        ]),
        F::SkewX(x) => TransformMatrix::new([1., 0., x.radians().tan(), 1., 0., 0.]),
        F::SkewY(y) => TransformMatrix::new([1., y.radians().tan(), 0., 1., 0., 0.]),
        F::Translate { x, y } => TransformMatrix::new([
            1.,
            0.,
            0.,
            1.,
            x.value(),
            y.unwrap_or(CssPixels::ZERO).value(),
        ]),
        F::TranslateX(x) => TransformMatrix::new([1., 0., 0., 1., x.value(), 0.]),
        F::TranslateY(y) => TransformMatrix::new([1., 0., 0., 1., 0., y.value()]),
        F::Scale { x, y } => {
            TransformMatrix::new([x.value(), 0., 0., y.unwrap_or(x).value(), 0., 0.])
        }
        F::ScaleX(x) => TransformMatrix::new([x.value(), 0., 0., 1., 0., 0.]),
        F::ScaleY(y) => TransformMatrix::new([1., 0., 0., y.value(), 0., 0.]),
    }
}

pub(super) fn list_matrix(
    functions: &[F],
    start: usize,
) -> Result<TransformMatrix, TransformError> {
    let mut matrix = TransformMatrix::IDENTITY;
    for (offset, function) in functions.iter().enumerate() {
        matrix = function_matrix(*function)
            .and_then(|next| multiply(matrix, next))
            .map_err(|error| error.at_index(start + offset))?;
    }
    Ok(matrix)
}

// Every finite binary64 coefficient is an integer significand times a power of
// two. Products of two significands fit in 106 bits. Comparing their exact
// normalized integer products avoids determinant overflow, underflow and an
// epsilon that would turn a valid tiny scale into a singular endpoint.
fn product(a: f64, b: f64) -> (bool, u128, i32) {
    fn parts(value: f64) -> (u64, i32) {
        let bits = value.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as i32;
        let fraction = bits & ((1_u64 << 52) - 1);
        if exponent == 0 {
            (fraction, -1074)
        } else {
            (fraction | (1_u64 << 52), exponent - 1075)
        }
    }
    let (am, ae) = parts(a);
    let (bm, be) = parts(b);
    (
        (a.to_bits() ^ b.to_bits()) >> 63 != 0,
        u128::from(am) * u128::from(bm),
        ae + be,
    )
}

fn compare_product_magnitudes(a: (u128, i32), b: (u128, i32)) -> core::cmp::Ordering {
    let (am, ae) = a;
    let (bm, be) = b;
    if am == 0 || bm == 0 {
        return am.cmp(&bm);
    }
    let al = am.leading_zeros();
    let bl = bm.leading_zeros();
    let top = (ae + 128 - al as i32).cmp(&(be + 128 - bl as i32));
    if top.is_eq() {
        (am << al).cmp(&(bm << bl))
    } else {
        top
    }
}

fn determinant_sign(matrix: TransformMatrix) -> core::cmp::Ordering {
    use core::cmp::Ordering;
    let [a, b, c, d, _, _] = matrix.coefficients();
    let (an, am, ae) = product(a, d);
    let (bn, bm, be) = product(b, c);
    if am == 0 && bm == 0 {
        return Ordering::Equal;
    }
    if am == 0 {
        return if bn {
            Ordering::Greater
        } else {
            Ordering::Less
        };
    }
    if bm == 0 {
        return if an {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    if an != bn {
        return if an {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let magnitude = compare_product_magnitudes((am, ae), (bm, be));
    if an { magnitude.reverse() } else { magnitude }
}

pub(super) fn singular(matrix: TransformMatrix) -> bool {
    determinant_sign(matrix).is_eq()
}

#[derive(Clone, Copy)]
pub(super) struct Decomposed {
    translation: [f64; 2],
    scale: [f64; 2],
    angle: f64,
    residual: [f64; 4],
}

fn normalized_column(x: f64, y: f64, component: C) -> Result<(f64, f64, f64), TransformError> {
    let magnitude = x.abs().max(y.abs());
    let scaled_x = checked(x / magnitude, component)?;
    let scaled_y = checked(y / magnitude, component)?;
    let length = scaled_x.hypot(scaled_y);
    // Derive the unit direction before restoring the column's magnitude.
    // Dividing by its rounded subnormal length would no longer produce a unit
    // vector, distorting the other column when rotation is removed and restored.
    Ok((
        checked(magnitude * length, component)?,
        checked(scaled_x / length, component)?,
        checked(scaled_y / length, component)?,
    ))
}

pub(super) fn decompose(matrix: TransformMatrix) -> Result<Decomposed, TransformError> {
    let [original_a, original_b, original_c, original_d, x, y] = matrix.coefficients();
    let (mut sx, mut a, mut b) = normalized_column(original_a, original_b, C::ScaleX)?;
    let (mut sy, mut c, mut d) = normalized_column(original_c, original_d, C::ScaleY)?;
    if determinant_sign(matrix).is_lt() {
        if original_a < original_d {
            sx = -sx;
            a = -a;
            b = -b;
        } else {
            sy = -sy;
            c = -c;
            d = -d;
        }
    }
    let angle = b.atan2(a);
    let residual = if angle == 0. {
        [a, b, c, d]
    } else {
        [
            dot(a, a, -b, c, C::MatrixCoefficient(0))?,
            dot(a, b, -b, d, C::MatrixCoefficient(1))?,
            dot(b, a, a, c, C::MatrixCoefficient(2))?,
            dot(b, b, a, d, C::MatrixCoefficient(3))?,
        ]
    };
    Ok(Decomposed {
        translation: [x, y],
        scale: [sx, sy],
        angle,
        residual,
    })
}

pub(super) fn interpolate_matrix(
    mut a: Decomposed,
    mut b: Decomposed,
    p: f64,
) -> Result<TransformMatrix, TransformError> {
    use core::f64::consts::{PI, TAU};
    if (a.scale[0] < 0. && b.scale[1] < 0.) || (a.scale[1] < 0. && b.scale[0] < 0.) {
        a.scale[0] = -a.scale[0];
        a.scale[1] = -a.scale[1];
        a.angle += if a.angle < 0. { PI } else { -PI };
    }
    if a.angle == 0. {
        a.angle = TAU;
    }
    if b.angle == 0. {
        b.angle = TAU;
    }
    if (a.angle - b.angle).abs() > PI {
        if a.angle > b.angle {
            a.angle -= TAU;
        } else {
            b.angle -= TAU;
        }
    }
    let x = lerp(a.translation[0], b.translation[0], p, C::TranslationX)?;
    let y = lerp(a.translation[1], b.translation[1], p, C::TranslationY)?;
    let sx = lerp(a.scale[0], b.scale[0], p, C::ScaleX)?;
    let sy = lerp(a.scale[1], b.scale[1], p, C::ScaleY)?;
    let angle = lerp(a.angle, b.angle, p, C::Angle)?;
    let mut residual = [0.; 4];
    for (index, result) in residual.iter_mut().enumerate() {
        *result = lerp(
            a.residual[index],
            b.residual[index],
            p,
            C::MatrixCoefficient(index),
        )?;
    }
    // Zero was represented as one turn solely for shortest-rotation matching.
    let angle = if angle == TAU || angle == -TAU {
        0.
    } else {
        angle
    };
    let (sin, cos) = angle.sin_cos();
    let [a, b, c, d] = residual;
    // #552 selects direct placement of the extracted affine translation. The
    // pinned §13.2.3 residual-weighted translation does not reconstruct shear.
    TransformMatrix::new([
        checked(
            dot(a, cos, c, sin, C::MatrixCoefficient(0))? * sx,
            C::MatrixCoefficient(0),
        )?,
        checked(
            dot(b, cos, d, sin, C::MatrixCoefficient(1))? * sx,
            C::MatrixCoefficient(1),
        )?,
        checked(
            dot(-a, sin, c, cos, C::MatrixCoefficient(2))? * sy,
            C::MatrixCoefficient(2),
        )?,
        checked(
            dot(-b, sin, d, cos, C::MatrixCoefficient(3))? * sy,
            C::MatrixCoefficient(3),
        )?,
        x,
        y,
    ])
}

enum Shape {
    Translate(bool),
    TranslateX,
    TranslateY,
    Scale(bool),
    ScaleX,
    ScaleY,
    Rotate,
    Skew(bool),
    SkewX,
    SkewY,
}
pub(super) struct FunctionPair {
    shape: Shape,
    from: [f64; 2],
    to: [f64; 2],
}

pub(super) fn function_pair(a: F, b: F) -> Option<FunctionPair> {
    let (shape, from, to) = match (a, b) {
        (F::Translate { x: ax, y: ay }, F::Translate { x: bx, y: by })
            if ay.is_some() == by.is_some() =>
        {
            (
                Shape::Translate(ay.is_some()),
                [ax.value(), ay.unwrap_or(CssPixels::ZERO).value()],
                [bx.value(), by.unwrap_or(CssPixels::ZERO).value()],
            )
        }
        (F::TranslateX(a), F::TranslateX(b)) => {
            (Shape::TranslateX, [a.value(), 0.], [b.value(), 0.])
        }
        (F::TranslateY(a), F::TranslateY(b)) => {
            (Shape::TranslateY, [a.value(), 0.], [b.value(), 0.])
        }
        (F::Scale { x: ax, y: ay }, F::Scale { x: bx, y: by }) if ay.is_some() == by.is_some() => (
            Shape::Scale(ay.is_some()),
            [ax.value(), ay.unwrap_or(ax).value()],
            [bx.value(), by.unwrap_or(bx).value()],
        ),
        (F::ScaleX(a), F::ScaleX(b)) => (Shape::ScaleX, [a.value(), 0.], [b.value(), 0.]),
        (F::ScaleY(a), F::ScaleY(b)) => (Shape::ScaleY, [a.value(), 0.], [b.value(), 0.]),
        (F::Rotate(a), F::Rotate(b)) => (Shape::Rotate, [a.radians(), 0.], [b.radians(), 0.]),
        (F::Skew { x: ax, y: ay }, F::Skew { x: bx, y: by }) => (
            Shape::Skew(ay.is_some() || by.is_some()),
            [ax.radians(), ay.unwrap_or(TransformAngle::ZERO).radians()],
            [bx.radians(), by.unwrap_or(TransformAngle::ZERO).radians()],
        ),
        (F::SkewX(a), F::SkewX(b)) => (Shape::SkewX, [a.radians(), 0.], [b.radians(), 0.]),
        (F::SkewY(a), F::SkewY(b)) => (Shape::SkewY, [a.radians(), 0.], [b.radians(), 0.]),
        _ => {
            if let (Some((ax, ay)), Some((bx, by))) = (a.translation(), b.translation()) {
                (
                    Shape::Translate(true),
                    [ax.value(), ay.value()],
                    [bx.value(), by.value()],
                )
            } else if let (Some((ax, ay)), Some((bx, by))) = (a.scale(), b.scale()) {
                (
                    Shape::Scale(true),
                    [ax.value(), ay.value()],
                    [bx.value(), by.value()],
                )
            } else {
                return None;
            }
        }
    };
    Some(FunctionPair { shape, from, to })
}

impl FunctionPair {
    pub(super) fn sample(self, p: f64) -> Result<F, TransformError> {
        let components = match self.shape {
            Shape::Translate(_) => [C::TranslationX, C::TranslationY],
            Shape::TranslateX => [C::TranslationX, C::TranslationX],
            Shape::TranslateY => [C::TranslationY, C::TranslationY],
            Shape::Scale(_) => [C::ScaleX, C::ScaleY],
            Shape::ScaleX => [C::ScaleX, C::ScaleX],
            Shape::ScaleY => [C::ScaleY, C::ScaleY],
            Shape::Rotate | Shape::Skew(_) | Shape::SkewX | Shape::SkewY => [C::Angle, C::Angle],
        };
        let x = lerp(self.from[0], self.to[0], p, components[0])?;
        let y = lerp(self.from[1], self.to[1], p, components[1])?;
        // Both components crossed the finite boundary immediately above.
        Ok(match self.shape {
            Shape::Translate(two) => F::Translate {
                x: CssPixels(x),
                y: two.then_some(CssPixels(y)),
            },
            Shape::TranslateX => F::TranslateX(CssPixels(x)),
            Shape::TranslateY => F::TranslateY(CssPixels(x)),
            Shape::Scale(two) => F::Scale {
                x: ScaleFactor(x),
                y: two.then_some(ScaleFactor(y)),
            },
            Shape::ScaleX => F::ScaleX(ScaleFactor(x)),
            Shape::ScaleY => F::ScaleY(ScaleFactor(x)),
            Shape::Rotate => F::Rotate(TransformAngle(x)),
            Shape::Skew(two) => F::Skew {
                x: TransformAngle(x),
                y: two.then_some(TransformAngle(y)),
            },
            Shape::SkewX => F::SkewX(TransformAngle(x)),
            Shape::SkewY => F::SkewY(TransformAngle(x)),
        })
    }
}
