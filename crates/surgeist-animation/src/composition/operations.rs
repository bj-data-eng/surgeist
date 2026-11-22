//! Default animation-type operations; property-specific policy stays caller-owned.

use super::PropertyValueOperations;
use crate::computed::ComputedError;
use crate::filter::FilterError;
use crate::grid::GridError;
use crate::interpolation::{
    InterpolableNumber, InterpolablePercentage, InterpolableValue, InterpolationError,
    InterpolationOutcome, InterpolationPair, InterpolationProgress, PropertyKey, TransformValue,
    ValueFamily,
};
use crate::keyframe_input::CompositeOperation;
use crate::shadow::ShadowError;
use crate::transform::TransformError;
use core::fmt;
use surgeist_color::InterpolationMethod;

/// Selects a generic animation type after the property's own rule is consulted.
///
/// Missing legacy type lines default to by-computed-value except applicable
/// Appendix A exceptions. The caller selects those exceptions and property-
/// specific operations; this enum is not a CSS property registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Discrete,
    /// Combines computed components or delegates to the selected property rules
    /// for transforms, shadows, filters, Grid tracks, font palettes and colors.
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
    color_method: Option<InterpolationMethod>,
}
impl AnimationTypeOperations {
    pub const fn new(animation_type: AnimationType) -> Self {
        Self {
            animation_type,
            color_method: None,
        }
    }

    /// Selects the supplied color method for resolved endpoints in
    /// by-computed-value operations.
    pub const fn for_color(method: InterpolationMethod) -> Self {
        Self {
            animation_type: AnimationType::ByComputedValue,
            color_method: Some(method),
        }
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
    Transform(Box<TransformError>),
    Shadow(Box<ShadowError>),
    Filter(Box<FilterError>),
    Grid(Box<GridError>),
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
            PropertyOperationErrorKind::Transform(error) => Some(error.as_ref()),
            PropertyOperationErrorKind::Shadow(error) => Some(error.as_ref()),
            PropertyOperationErrorKind::Filter(error) => Some(error.as_ref()),
            PropertyOperationErrorKind::Grid(error) => Some(error.as_ref()),
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
impl From<GridError> for PropertyOperationError {
    fn from(error: GridError) -> Self {
        Self {
            kind: PropertyOperationErrorKind::Grid(Box::new(error)),
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
        let pair = match (start, end, self.color_method) {
            (
                InterpolableValue::ResolvedColor(start),
                InterpolableValue::ResolvedColor(end),
                Some(method),
            ) => Ok(InterpolationPair::with_color_method(
                property.clone(),
                start.clone(),
                end.clone(),
                method,
            )),
            _ => InterpolationPair::new(property.clone(), start.clone(), end.clone()),
        };
        let pair = match pair {
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
            InterpolationOutcome::FilterFailure(error) => {
                Err(InterpolationError::Filter(error).into())
            }
            InterpolationOutcome::PaletteFailure(error) => {
                Err(InterpolationError::Palette(error).into())
            }
            InterpolationOutcome::ColorFailure(error) => {
                Err(InterpolationError::Color(error).into())
            }
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
            (a, b) if a.family() == ValueFamily::Color && b.family() == ValueFamily::Color => {
                Ok(value.clone())
            }
            // Palette addition belongs to its symbolic value owner. Unspecified
            // accumulation uses the same operation, retaining the effect palette.
            (InterpolableValue::Palette(a), InterpolableValue::Palette(b)) => {
                Ok(InterpolableValue::palette(a.add(b)))
            }
            (InterpolableValue::Computed(a), InterpolableValue::Computed(b)) => Ok(
                InterpolableValue::computed(if operation == CompositeOperation::Add {
                    a.add(b)?
                } else {
                    a.accumulate(b)?
                }),
            ),
            (InterpolableValue::Transform(a), InterpolableValue::Transform(b)) => {
                let (Some(a), Some(b)) = (a.resolved_list(), b.resolved_list()) else {
                    return Err(PropertyOperationError::unsupported(operation));
                };
                let value = if operation == CompositeOperation::Add {
                    a.try_added_to(b).map_err(|_| PropertyOperationError {
                        kind: PropertyOperationErrorKind::Capacity,
                    })?
                } else {
                    a.accumulated_with(b)
                        .map_err(|error| PropertyOperationError {
                            kind: PropertyOperationErrorKind::Transform(Box::new(error)),
                        })?
                };
                Ok(InterpolableValue::transform(TransformValue::resolved(
                    value,
                )))
            }
            (InterpolableValue::Shadow(a), InterpolableValue::Shadow(b)) => {
                let value = if operation == CompositeOperation::Add {
                    a.added_to(b)
                } else {
                    a.accumulated_with(b)
                }
                .map_err(|error| PropertyOperationError {
                    kind: PropertyOperationErrorKind::Shadow(Box::new(error)),
                })?;
                Ok(InterpolableValue::shadow(value))
            }
            (InterpolableValue::Filter(a), InterpolableValue::Filter(b)) => {
                let value = if operation == CompositeOperation::Add {
                    a.added_to(b).map_err(|error| PropertyOperationError {
                        kind: PropertyOperationErrorKind::Filter(Box::new(error)),
                    })?
                } else {
                    a.accumulated_with(b)
                };
                Ok(InterpolableValue::filter(value))
            }
            (InterpolableValue::Grid(a), InterpolableValue::Grid(b)) => {
                let value = if operation == CompositeOperation::Add {
                    a.add(b)
                } else {
                    a.accumulate(b)
                }?;
                Ok(InterpolableValue::grid(value))
            }
            (InterpolableValue::Discrete(_), InterpolableValue::Discrete(_)) => Ok(value.clone()),
            (a, b) if a.family() != b.family() => Ok(value.clone()),
            _ => Err(PropertyOperationError::unsupported(operation)),
        }
    }
}
