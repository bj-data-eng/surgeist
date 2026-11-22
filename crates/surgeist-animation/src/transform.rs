//! Resolved two-dimensional transform values and checked CSS list algebra.
//!
//! Inputs contain finite CSS-pixel lengths, scale factors and explicit angles.
//! Callers resolve percentages, reference boxes and authored calculations before
//! construction and refresh these immutable inputs when that context changes.
//! Function kind and arity remain available for list interpolation. Three-
//! dimensional transforms and accumulation are outside this module's capability.
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

/// The transform component rejected by construction or checked arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformComponent {
    TranslationX,
    TranslationY,
    ScaleX,
    ScaleY,
    Angle,
    MatrixCoefficient(usize),
}

/// A non-finite transform input or derived arithmetic component.
#[derive(Debug, Clone, PartialEq)]
pub struct TransformError {
    component: TransformComponent,
    value: f64,
    function_index: Option<usize>,
}

impl TransformError {
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
        write!(
            f,
            "transform {:?} must be finite, got {}",
            self.component, self.value
        )?;
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

/// A resolved 2D function retaining its CSS name and optional argument arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformFunction {
    Matrix(TransformMatrix),
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

/// An immutable resolved 2D list; an empty list represents CSS `none`.
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
        math::list_matrix(&self.functions, 0)
    }
}

pub(crate) enum TransformInterpolation {
    Value(ResolvedTransformList),
    Discrete,
}

enum PairPart {
    Functions(math::FunctionPair, usize),
    Matrix(math::Decomposed, math::Decomposed, usize),
}

/// Preflight decomposition before interpolating any prefix, so a singular
/// endpoint selects whole-value discrete fallback even if a prefix would overflow.
pub(crate) fn interpolate(
    from: &ResolvedTransformList,
    to: &ResolvedTransformList,
    progress: f64,
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
            Some((a, b, false))
        } else if let Some(pair) = math::function_pair(a, b) {
            parts.push(PairPart::Functions(pair, index));
            None
        } else {
            Some((
                math::list_matrix(from.functions.get(index..).unwrap_or_default(), index)?,
                math::list_matrix(to.functions.get(index..).unwrap_or_default(), index)?,
                true,
            ))
        };
        if let Some((a, b, suffix)) = matrices {
            // Check both endpoint singularities before either decomposition.
            if math::singular(a) || math::singular(b) {
                return Ok(TransformInterpolation::Discrete);
            }
            let a = math::decompose(a).map_err(|error| error.at_index(index))?;
            let b = math::decompose(b).map_err(|error| error.at_index(index))?;
            parts.push(PairPart::Matrix(a, b, index));
            if suffix {
                break;
            }
        }
    }
    let functions = parts
        .into_iter()
        .map(|part| {
            let (result, index) = match part {
                PairPart::Functions(pair, index) => (pair.sample(progress), index),
                PairPart::Matrix(a, b, index) => (
                    math::interpolate_matrix(a, b, progress).map(TransformFunction::Matrix),
                    index,
                ),
            };
            result.map_err(|error| error.at_index(index))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TransformInterpolation::Value(ResolvedTransformList::new(
        functions,
    )))
}
