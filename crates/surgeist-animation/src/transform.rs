//! Resolved transform values and checked CSS list algebra.
//!
//! Inputs contain finite CSS-pixel lengths, scale factors and explicit angles.
//! Callers resolve percentages, reference boxes and authored calculations before
//! construction and refresh these immutable inputs when that context changes.
//! Function kind and arity remain available for 2D/3D list interpolation,
//! addition and accumulation. Geometry visibility and execution remain caller-owned.
//!
//! Pixel lengths and scale factors cannot be substituted for one another:
//!
//! ```compile_fail
//! use surgeist_animation::transform::{ScaleFactor, TransformFunction};
//! let scale = ScaleFactor::new(2.0).unwrap();
//! let translation = TransformFunction::TranslateX(scale);
//! ```
//!
//! Lists expose only a shared slice, so sampled operations cannot be replaced:
//!
//! ```compile_fail
//! use surgeist_animation::transform::{ResolvedTransformList, TransformFunction, TransformMatrix};
//! let list = ResolvedTransformList::new(vec![TransformFunction::Matrix(TransformMatrix::IDENTITY)]);
//! list.functions()[0] = TransformFunction::Matrix(TransformMatrix::IDENTITY);
//! ```

use core::fmt;
use std::sync::Arc;

mod math;
mod spatial;

/// The transform component rejected by construction or checked arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformComponent {
    TranslationX,
    TranslationY,
    TranslationZ,
    ScaleX,
    ScaleY,
    ScaleZ,
    RotationAxis(usize),
    Quaternion(usize),
    Perspective(usize),
    Skew(usize),
    Angle,
    MatrixCoefficient(usize),
}

/// The reason a transform input or conversion was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformErrorKind {
    NonFinite,
    NotTwoDimensional,
}

/// A rejected transform input, conversion or derived arithmetic component.
#[derive(Debug, Clone, PartialEq)]
pub struct TransformError {
    kind: TransformErrorKind,
    component: TransformComponent,
    value: f64,
    function_index: Option<usize>,
}

impl TransformError {
    pub const fn kind(&self) -> TransformErrorKind {
        self.kind
    }
    pub const fn component(&self) -> TransformComponent {
        self.component
    }
    pub const fn value(&self) -> f64 {
        self.value
    }
    /// Returns the list position when the error arose from list arithmetic.
    pub const fn function_index(&self) -> Option<usize> {
        self.function_index
    }

    fn at_index(mut self, index: usize) -> Self {
        self.function_index = Some(index);
        self
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TransformErrorKind::NonFinite => write!(
                f,
                "transform {:?} must be finite, got {}",
                self.component, self.value
            )?,
            TransformErrorKind::NotTwoDimensional => write!(
                f,
                "transform {:?} prevents two-dimensional conversion: {}",
                self.component, self.value
            )?,
        }
        if let Some(index) = self.function_index {
            write!(f, " at function {index}")?;
        }
        Ok(())
    }
}
impl std::error::Error for TransformError {}

fn finite(value: f64, component: TransformComponent) -> Result<f64, TransformError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(TransformError {
            kind: TransformErrorKind::NonFinite,
            component,
            value,
            function_index: None,
        })
    }
}

/// A finite resolved length in CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CssPixels(f64);
// Finite construction excludes NaN, making numeric equality an equivalence.
impl Eq for CssPixels {}
impl CssPixels {
    pub const ZERO: Self = Self(0.0);
    pub fn new(value: f64) -> Result<Self, TransformError> {
        finite(value, TransformComponent::TranslationX).map(Self)
    }
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// A finite dimensionless scale, including negative and zero factors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor(f64);
impl Eq for ScaleFactor {}
impl ScaleFactor {
    pub const ONE: Self = Self(1.0);
    pub const ZERO: Self = Self(0.0);
    pub fn new(value: f64) -> Result<Self, TransformError> {
        finite(value, TransformComponent::ScaleX).map(Self)
    }
    pub const fn value(self) -> f64 {
        self.0
    }
}

/// A finite angle retaining complete turns for direct function interpolation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformAngle(f64);
impl Eq for TransformAngle {}
impl TransformAngle {
    pub const ZERO: Self = Self(0.0);
    pub fn from_radians(value: f64) -> Result<Self, TransformError> {
        finite(value, TransformComponent::Angle).map(Self)
    }
    pub fn from_degrees(value: f64) -> Result<Self, TransformError> {
        finite(value, TransformComponent::Angle)?;
        Self::from_radians(value.to_radians())
    }
    pub fn from_turns(value: f64) -> Result<Self, TransformError> {
        finite(value, TransformComponent::Angle)?;
        Self::from_radians(value * core::f64::consts::TAU)
    }
    pub const fn radians(self) -> f64 {
        self.0
    }
}

/// A finite 2D affine matrix in CSS `[a,b,c,d,e,f]` column-major order.
///
/// Singular matrices are valid values. Their visibility belongs to consumers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformMatrix([f64; 6]);
impl Eq for TransformMatrix {}
impl TransformMatrix {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    pub fn new(coefficients: [f64; 6]) -> Result<Self, TransformError> {
        for (index, value) in coefficients.into_iter().enumerate() {
            finite(value, TransformComponent::MatrixCoefficient(index))?;
        }
        Ok(Self(coefficients))
    }
    pub const fn coefficients(self) -> [f64; 6] {
        self.0
    }
}

/// A finite homogeneous 4×4 matrix, stored column first, then row.
/// Singular matrices and finite extrapolated perspective coefficients are valid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformMatrix3d([f64; 16]);
impl Eq for TransformMatrix3d {}
impl TransformMatrix3d {
    pub const IDENTITY: Self = Self([
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]);
    pub fn new(coefficients: [f64; 16]) -> Result<Self, TransformError> {
        for (index, value) in coefficients.into_iter().enumerate() {
            finite(value, TransformComponent::MatrixCoefficient(index))?;
        }
        Ok(Self(coefficients))
    }
    pub const fn coefficients(self) -> [f64; 16] {
        self.0
    }
    fn as_affine(self) -> Option<TransformMatrix> {
        self.require_affine().ok()
    }
    fn require_affine(self) -> Result<TransformMatrix, TransformError> {
        for (index, expected) in [
            (2, 0.),
            (3, 0.),
            (6, 0.),
            (7, 0.),
            (8, 0.),
            (9, 0.),
            (10, 1.),
            (11, 0.),
            (14, 0.),
            (15, 1.),
        ] {
            if self.0[index] != expected {
                return Err(TransformError {
                    kind: TransformErrorKind::NotTwoDimensional,
                    component: TransformComponent::MatrixCoefficient(index),
                    value: self.0[index],
                    function_index: None,
                });
            }
        }
        let m = self.0;
        Ok(TransformMatrix([m[0], m[1], m[4], m[5], m[12], m[13]]))
    }
}
impl From<TransformMatrix> for TransformMatrix3d {
    fn from(value: TransformMatrix) -> Self {
        let [a, b, c, d, e, f] = value.0;
        Self([a, b, 0., 0., c, d, 0., 0., 0., 0., 1., 0., e, f, 0., 1.])
    }
}

/// A normalized finite rotation direction; the zero direction is a valid no-op.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotationAxis([f64; 3]);
impl Eq for RotationAxis {}
impl RotationAxis {
    pub fn new(values: [f64; 3]) -> Result<Self, TransformError> {
        for (index, value) in values.into_iter().enumerate() {
            finite(value, TransformComponent::RotationAxis(index))?;
        }
        let magnitude = values.into_iter().map(f64::abs).fold(0., f64::max);
        if magnitude == 0. {
            return Ok(Self([0.; 3]));
        }
        let scaled = values.map(|value| value / magnitude);
        let norm = scaled[0].hypot(scaled[1]).hypot(scaled[2]);
        Ok(Self(scaled.map(|value| value / norm)))
    }
    pub const fn components(self) -> [f64; 3] {
        self.0
    }
}

/// A resolved transform function retaining its CSS name and argument arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformFunction {
    Matrix(TransformMatrix),
    Matrix3d(TransformMatrix3d),
    Translate3d {
        x: CssPixels,
        y: CssPixels,
        z: CssPixels,
    },
    TranslateZ(CssPixels),
    Scale3d {
        x: ScaleFactor,
        y: ScaleFactor,
        z: ScaleFactor,
    },
    ScaleZ(ScaleFactor),
    RotateX(TransformAngle),
    RotateY(TransformAngle),
    RotateZ(TransformAngle),
    Rotate3d {
        axis: RotationAxis,
        angle: TransformAngle,
    },
    /// `None` is perspective(none); finite depths are floored to 1px on formation.
    Perspective(Option<CssPixels>),
    Translate {
        x: CssPixels,
        y: Option<CssPixels>,
    },
    TranslateX(CssPixels),
    TranslateY(CssPixels),
    Scale {
        x: ScaleFactor,
        y: Option<ScaleFactor>,
    },
    ScaleX(ScaleFactor),
    ScaleY(ScaleFactor),
    Rotate(TransformAngle),
    Skew {
        x: TransformAngle,
        y: Option<TransformAngle>,
    },
    SkewX(TransformAngle),
    SkewY(TransformAngle),
}

impl TransformFunction {
    fn identity(self) -> Self {
        match self {
            Self::Matrix3d(_) => Self::Matrix3d(TransformMatrix3d::IDENTITY),
            Self::Translate3d { .. } => Self::Translate3d {
                x: CssPixels::ZERO,
                y: CssPixels::ZERO,
                z: CssPixels::ZERO,
            },
            Self::TranslateZ(_) => Self::TranslateZ(CssPixels::ZERO),
            Self::Scale3d { .. } => Self::Scale3d {
                x: ScaleFactor::ONE,
                y: ScaleFactor::ONE,
                z: ScaleFactor::ONE,
            },
            Self::ScaleZ(_) => Self::ScaleZ(ScaleFactor::ONE),
            Self::RotateX(_) => Self::RotateX(TransformAngle::ZERO),
            Self::RotateY(_) => Self::RotateY(TransformAngle::ZERO),
            Self::RotateZ(_) => Self::RotateZ(TransformAngle::ZERO),
            Self::Rotate3d { axis, .. } => Self::Rotate3d {
                axis,
                angle: TransformAngle::ZERO,
            },
            Self::Perspective(_) => Self::Perspective(None),
            Self::Matrix(_) => Self::Matrix(TransformMatrix::IDENTITY),
            Self::Translate { y, .. } => Self::Translate {
                x: CssPixels::ZERO,
                y: y.map(|_| CssPixels::ZERO),
            },
            Self::TranslateX(_) => Self::TranslateX(CssPixels::ZERO),
            Self::TranslateY(_) => Self::TranslateY(CssPixels::ZERO),
            Self::Scale { y, .. } => Self::Scale {
                x: ScaleFactor::ONE,
                y: y.map(|_| ScaleFactor::ONE),
            },
            Self::ScaleX(_) => Self::ScaleX(ScaleFactor::ONE),
            Self::ScaleY(_) => Self::ScaleY(ScaleFactor::ONE),
            Self::Rotate(_) => Self::Rotate(TransformAngle::ZERO),
            Self::Skew { y, .. } => Self::Skew {
                x: TransformAngle::ZERO,
                y: y.map(|_| TransformAngle::ZERO),
            },
            Self::SkewX(_) => Self::SkewX(TransformAngle::ZERO),
            Self::SkewY(_) => Self::SkewY(TransformAngle::ZERO),
        }
    }

    /// Returns the pin-selected additive neutral with this function's shape.
    /// This differs from identity padding for scale and matrix functions.
    pub fn neutral_for_addition(self) -> Self {
        match self {
            Self::Matrix(_) => Self::Matrix(TransformMatrix([0.; 6])),
            Self::Matrix3d(_) => Self::Matrix3d(TransformMatrix3d([0.; 16])),
            Self::Scale { x: _, y } => Self::Scale {
                x: ScaleFactor::ZERO,
                y: y.map(|_| ScaleFactor::ZERO),
            },
            Self::ScaleX(_) => Self::ScaleX(ScaleFactor::ZERO),
            Self::ScaleY(_) => Self::ScaleY(ScaleFactor::ZERO),
            Self::ScaleZ(_) => Self::ScaleZ(ScaleFactor::ZERO),
            Self::Scale3d { .. } => Self::Scale3d {
                x: ScaleFactor::ZERO,
                y: ScaleFactor::ZERO,
                z: ScaleFactor::ZERO,
            },
            _ => self.identity(),
        }
    }
    fn is_spatial(self) -> bool {
        matches!(
            self,
            Self::Matrix3d(_)
                | Self::Translate3d { .. }
                | Self::TranslateZ(_)
                | Self::Scale3d { .. }
                | Self::ScaleZ(_)
                | Self::RotateX(_)
                | Self::RotateY(_)
                | Self::RotateZ(_)
                | Self::Rotate3d { .. }
                | Self::Perspective(_)
        )
    }

    fn translation(self) -> Option<(CssPixels, CssPixels)> {
        match self {
            Self::Translate { x, y } => Some((x, y.unwrap_or(CssPixels::ZERO))),
            Self::TranslateX(x) => Some((x, CssPixels::ZERO)),
            Self::TranslateY(y) => Some((CssPixels::ZERO, y)),
            _ => None,
        }
    }
    fn scale(self) -> Option<(ScaleFactor, ScaleFactor)> {
        match self {
            Self::Scale { x, y } => Some((x, y.unwrap_or(x))),
            Self::ScaleX(x) => Some((x, ScaleFactor::ONE)),
            Self::ScaleY(y) => Some((ScaleFactor::ONE, y)),
            _ => None,
        }
    }
}

/// An immutable resolved transform list; an empty list represents CSS `none`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTransformList {
    functions: Arc<[TransformFunction]>,
}

impl ResolvedTransformList {
    pub fn new(functions: Vec<TransformFunction>) -> Self {
        Self {
            functions: functions.into(),
        }
    }
    pub fn functions(&self) -> &[TransformFunction] {
        &self.functions
    }

    /// Concatenates this list with `other` in CSS addition order.
    ///
    /// This is list addition, not parameter accumulation or matrix multiplication.
    pub fn added_to(&self, other: &Self) -> Self {
        Self::new(
            self.functions
                .iter()
                .chain(other.functions.iter())
                .copied()
                .collect(),
        )
    }

    /// Post-multiplies the functions into a finite affine matrix.
    ///
    /// Reports arithmetic overflow without changing the list or clipping values.
    pub fn to_matrix(&self) -> Result<TransformMatrix, TransformError> {
        if self.is_spatial() {
            self.to_matrix3d()?.require_affine()
        } else {
            math::list_matrix(&self.functions, 0)
        }
    }
    /// Forms a checked column-major homogeneous matrix without 2D narrowing.
    pub fn to_matrix3d(&self) -> Result<TransformMatrix3d, TransformError> {
        spatial::list_matrix(&self.functions, 0)
    }
    /// Accumulates this underlying value with an effect using CSS list matching.
    /// Non-decomposable endpoints select the complete effect value.
    pub fn accumulated_with(&self, effect: &Self) -> Result<Self, TransformError> {
        match combine(self, effect, Combination::Accumulate)? {
            TransformInterpolation::Value(value) => Ok(value),
            TransformInterpolation::Discrete => Ok(effect.clone()),
        }
    }
    /// Maps each function to its pin-selected scalar-zero additive neutral.
    /// Padding identities and the empty concatenation identity remain distinct.
    pub fn neutral_for_addition(&self) -> Self {
        Self::new(
            self.functions
                .iter()
                .map(|function| function.neutral_for_addition())
                .collect(),
        )
    }
    pub(crate) fn is_spatial(&self) -> bool {
        self.functions.iter().any(|function| function.is_spatial())
    }
}

pub(crate) enum TransformInterpolation {
    Value(ResolvedTransformList),
    Discrete,
}

#[derive(Clone, Copy)]
enum Combination {
    Interpolate(f64),
    Accumulate,
}
fn combine_number(
    a: f64,
    b: f64,
    operation: Combination,
    one_based: bool,
    component: TransformComponent,
) -> Result<f64, TransformError> {
    match operation {
        Combination::Interpolate(p) => {
            let delta = finite(b - a, component)?;
            finite(a + finite(delta * p, component)?, component)
        }
        Combination::Accumulate => finite(
            a + finite(if one_based { b - 1. } else { b }, component)?,
            component,
        ),
    }
}

enum MatrixPair {
    Affine(TransformMatrix, TransformMatrix, bool),
    Spatial(TransformMatrix3d, TransformMatrix3d),
}
impl MatrixPair {
    fn prepare(a: TransformMatrix3d, b: TransformMatrix3d, output_3d: bool) -> Option<Self> {
        if let (Some(a), Some(b)) = (a.as_affine(), b.as_affine()) {
            if math::singular(a) || math::singular(b) {
                None
            } else {
                Some(Self::Affine(a, b, output_3d))
            }
        } else if spatial::decomposable(a) && spatial::decomposable(b) {
            Some(Self::Spatial(a, b))
        } else {
            None
        }
    }
    fn sample(self, operation: Combination) -> Result<TransformFunction, TransformError> {
        match self {
            Self::Affine(a, b, output_3d) => {
                let result =
                    math::combine_matrix(math::decompose(a)?, math::decompose(b)?, operation)?;
                Ok(if output_3d {
                    TransformFunction::Matrix3d(result.into())
                } else {
                    TransformFunction::Matrix(result)
                })
            }
            Self::Spatial(a, b) => {
                spatial::combine_matrix(a, b, operation).map(TransformFunction::Matrix3d)
            }
        }
    }
}
enum PairPart {
    Functions(math::FunctionPair, usize),
    SpatialFunctions(spatial::FunctionPair, usize),
    Matrix(MatrixPair, usize),
}
pub(crate) fn interpolate(
    from: &ResolvedTransformList,
    to: &ResolvedTransformList,
    progress: f64,
) -> Result<TransformInterpolation, TransformError> {
    combine(from, to, Combination::Interpolate(progress))
}
fn combine(
    from: &ResolvedTransformList,
    to: &ResolvedTransformList,
    operation: Combination,
) -> Result<TransformInterpolation, TransformError> {
    let mut parts = Vec::new();
    for index in 0..from.functions.len().max(to.functions.len()) {
        let (a, b) = match (from.functions.get(index), to.functions.get(index)) {
            (Some(a), Some(b)) => (*a, *b),
            (Some(a), None) => (*a, a.identity()),
            (None, Some(b)) => (b.identity(), *b),
            (None, None) => break,
        };
        let matrices = if let (TransformFunction::Matrix(a), TransformFunction::Matrix(b)) = (a, b)
        {
            Some((a.into(), b.into(), false, false))
        } else if let Some(pair) = math::function_pair(a, b) {
            parts.push(PairPart::Functions(pair, index));
            None
        } else if let Some(pair) = spatial::function_pair(a, b) {
            parts.push(PairPart::SpatialFunctions(pair, index));
            None
        } else if spatial::matching_matrix_functions(a, b) {
            Some((
                spatial::function_matrix(a)?,
                spatial::function_matrix(b)?,
                true,
                false,
            ))
        } else {
            let from = from.functions.get(index..).unwrap_or_default();
            let to = to.functions.get(index..).unwrap_or_default();
            let output_3d = from.iter().chain(to).any(|function| function.is_spatial());
            let (a, b) = if output_3d {
                (
                    spatial::list_matrix(from, index)?,
                    spatial::list_matrix(to, index)?,
                )
            } else {
                (
                    math::list_matrix(from, index)?.into(),
                    math::list_matrix(to, index)?.into(),
                )
            };
            Some((a, b, output_3d, true))
        };
        if let Some((a, b, output_3d, suffix)) = matrices {
            let Some(pair) = MatrixPair::prepare(a, b, output_3d) else {
                return Ok(TransformInterpolation::Discrete);
            };
            parts.push(PairPart::Matrix(pair, index));
            if suffix {
                break;
            }
        }
    }
    let functions = parts
        .into_iter()
        .map(|part| {
            let (result, index) = match part {
                PairPart::Functions(pair, index) => (pair.sample(operation), index),
                PairPart::SpatialFunctions(pair, index) => (pair.sample(operation), index),
                PairPart::Matrix(pair, index) => (pair.sample(operation), index),
            };
            result.map_err(|error| error.at_index(index))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TransformInterpolation::Value(ResolvedTransformList::new(
        functions,
    )))
}
