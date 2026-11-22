//! Default animation-type operations; property-specific policy stays caller-owned.

use super::PropertyValueOperations;
use crate::computed::ComputedError;
use crate::interpolation::{
    InterpolableNumber, InterpolablePercentage, InterpolableValue, InterpolationError,
    InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey, TransformValue,
};
use crate::keyframe_input::CompositeOperation;
use crate::transform::ResolvedTransformList;
use core::fmt;

/// Selects a generic animation type after the property's own rule is consulted.
///
/// Missing legacy type lines default to by-computed-value except applicable
/// Appendix A exceptions. The caller selects those exceptions and property-
/// specific operations; this enum is not a CSS property registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Discrete,
    ByComputedValue,
    RepeatableList,
}

/// Executes one selected generic animation type through the shared interface.
///
/// Use another [`PropertyValueOperations`] implementation for property-specific
/// algorithms. Not-animatable properties are excluded from stack target values;
/// exclusion does not alter their effect's timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationTypeOperations {
    animation_type: AnimationType,
}
impl AnimationTypeOperations {
    pub const fn new(animation_type: AnimationType) -> Self {
        Self { animation_type }
    }
    pub const fn animation_type(self) -> AnimationType {
        self.animation_type
    }
}

/// Distinguishes invalid payloads, unsupported operations and arithmetic limits.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyOperationErrorKind {
    WrongValueFamily,
    Unsupported(CompositeOperation),
    Capacity,
    Computed(ComputedError),
    Interpolation(Box<InterpolationError>),
}
/// A typed failure from a generic property operation.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyOperationError {
    kind: PropertyOperationErrorKind,
}
impl PropertyOperationError {
    pub const fn kind(&self) -> &PropertyOperationErrorKind {
        &self.kind
    }
    fn unsupported(operation: CompositeOperation) -> Self {
        Self {
            kind: PropertyOperationErrorKind::Unsupported(operation),
        }
    }
}
impl fmt::Display for PropertyOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "property operation: {:?}", self.kind)
    }
}
impl std::error::Error for PropertyOperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PropertyOperationErrorKind::Computed(error) => Some(error),
            PropertyOperationErrorKind::Interpolation(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}
impl From<ComputedError> for PropertyOperationError {
    fn from(error: ComputedError) -> Self {
        Self {
            kind: PropertyOperationErrorKind::Computed(error),
        }
    }
}
impl From<InterpolationError> for PropertyOperationError {
    fn from(error: InterpolationError) -> Self {
        Self {
            kind: PropertyOperationErrorKind::Interpolation(Box::new(error)),
        }
    }
}

impl PropertyValueOperations for AnimationTypeOperations {
    type Error = PropertyOperationError;
    fn interpolate(
        &self,
        property: &PropertyKey,
        start: &InterpolableValue,
        end: &InterpolableValue,
        progress: InterpolationProgress,
    ) -> Result<InterpolableValue, Self::Error> {
        if self.animation_type == AnimationType::Discrete {
            return Ok(if progress.value() < 0.5 { start } else { end }.clone());
        }
        if self.animation_type == AnimationType::RepeatableList {
            return match (start, end) {
                (
                    InterpolableValue::RepeatableList(start),
                    InterpolableValue::RepeatableList(end),
                ) => Ok(InterpolableValue::repeatable_list(
                    start.interpolate(end, progress)?,
                )),
                _ => Err(PropertyOperationError {
                    kind: PropertyOperationErrorKind::WrongValueFamily,
                }),
            };
        }
        let pair = match InterpolationPair::new(property.clone(), start.clone(), end.clone()) {
            Ok(pair) => pair,
            Err(
                InterpolationError::MismatchedFamilies { .. }
                | InterpolationError::MismatchedColorSpaces { .. },
            ) => return Ok(if progress.value() < 0.5 { start } else { end }.clone()),
            Err(error) => return Err(error.into()),
        };
        match pair.sample(progress) {
            InterpolationOutcome::Value(value) => Ok(value),
            InterpolationOutcome::ArithmeticFailure(error) => {
                Err(InterpolationError::NonFiniteInterpolationResult(error).into())
            }
            InterpolationOutcome::ComputedFailure(error) => {
                Err(InterpolationError::Computed(error).into())
            }
            InterpolationOutcome::ShapeFailure(error) => {
                Err(InterpolationError::Shape(error).into())
            }
            InterpolationOutcome::ShadowFailure(error) => {
                Err(InterpolationError::Shadow(error).into())
            }
            InterpolationOutcome::GridFailure(error) => Err(InterpolationError::Grid(error).into()),
            InterpolationOutcome::Unsupported(error) => Err(error.into()),
        }
    }
    fn add(
        &self,
        _: &PropertyKey,
        underlying: &InterpolableValue,
        value: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error> {
        self.combine(underlying, value, CompositeOperation::Add)
    }
    fn accumulate(
        &self,
        _: &PropertyKey,
        underlying: &InterpolableValue,
        value: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error> {
        self.combine(underlying, value, CompositeOperation::Accumulate)
    }
}

impl AnimationTypeOperations {
    fn combine(
        &self,
        underlying: &InterpolableValue,
        value: &InterpolableValue,
        operation: CompositeOperation,
    ) -> Result<InterpolableValue, PropertyOperationError> {
        if self.animation_type == AnimationType::Discrete {
            return Ok(value.clone());
        }
        if self.animation_type == AnimationType::RepeatableList {
            return match (underlying, value) {
                (InterpolableValue::RepeatableList(a), InterpolableValue::RepeatableList(b)) => Ok(
                    InterpolableValue::repeatable_list(if operation == CompositeOperation::Add {
                        a.add(b)?
                    } else {
                        a.accumulate(b)?
                    }),
                ),
                _ => Err(PropertyOperationError {
                    kind: PropertyOperationErrorKind::WrongValueFamily,
                }),
            };
        }
        match (underlying, value) {
            (InterpolableValue::Number(a), InterpolableValue::Number(b)) => Ok(
                InterpolableValue::number(InterpolableNumber::new(a.value() + b.value())?),
            ),
            (InterpolableValue::Percentage(a), InterpolableValue::Percentage(b)) => Ok(
                InterpolableValue::percentage(InterpolablePercentage::new(a.value() + b.value())?),
            ),
            // Selected Values 4: absent color addition selects B; unspecified
            // accumulation uses that same nonadditive operation.
            (InterpolableValue::Color(_), InterpolableValue::Color(_)) => Ok(value.clone()),
            (InterpolableValue::Computed(a), InterpolableValue::Computed(b)) => Ok(
                InterpolableValue::computed(if operation == CompositeOperation::Add {
                    a.add(b)?
                } else {
                    a.accumulate(b)?
                }),
            ),
            (InterpolableValue::Transform(a), InterpolableValue::Transform(b))
                if operation == CompositeOperation::Add =>
            {
                let (Some(a), Some(b)) = (a.resolved_list(), b.resolved_list()) else {
                    return Err(PropertyOperationError::unsupported(operation));
                };
                let length = a.functions().len().checked_add(b.functions().len()).ok_or(
                    PropertyOperationError {
                        kind: PropertyOperationErrorKind::Capacity,
                    },
                )?;
                let mut functions = Vec::new();
                functions
                    .try_reserve_exact(length)
                    .map_err(|_| PropertyOperationError {
                        kind: PropertyOperationErrorKind::Capacity,
                    })?;
                functions.extend_from_slice(a.functions());
                functions.extend_from_slice(b.functions());
                Ok(InterpolableValue::transform(TransformValue::resolved(
                    ResolvedTransformList::new(functions),
                )))
            }
            (InterpolableValue::Discrete(_), InterpolableValue::Discrete(_)) => Ok(value.clone()),
            (a, b) if a.family() != b.family() => Ok(value.clone()),
            _ => Err(PropertyOperationError::unsupported(operation)),
        }
    }
}
