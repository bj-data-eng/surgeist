//! Property operations, immutable effect stacks, and replacement decisions.
//!
//! The caller selects the property's animation type and its owned algorithms.
//! Operations consume computed values. Stack snapshots validate caller-supplied
//! ordering facts and fold keyframe effects against evolving underlying values.
//! Contextual conversion, final clamping, state publication and dispatch remain
//! caller-owned.

use crate::interpolation::{InterpolableValue, InterpolationProgress, PropertyKey};
use crate::keyframe_input::CompositeOperation;

mod operations;
mod stack;

#[doc(inline)]
pub use operations::{
    AnimationType, AnimationTypeOperations, PropertyOperationError, PropertyOperationErrorKind,
};
#[doc(inline)]
pub use stack::{
    AnimationClass, AnimationOrder, AnimationStackInput, CompositedProperty, EffectStackEffect,
    EffectStackError, EffectStackErrorKind, EffectStackSampleError, EffectStackSnapshot,
    EffectTargetId, ReplacementDecision,
};

/// Supplies deterministic property-specific operations without publishing state.
///
/// Implementations preserve whole-property discrete fallback and typed failures
/// according to the selected animation type. Unsupported accumulation must not
/// silently use addition. Existing computed and transform APIs supply their
/// implemented operations; this interface does not invent missing algorithms.
pub trait PropertyValueOperations {
    type Error;
    fn interpolate(
        &self,
        property: &PropertyKey,
        start: &InterpolableValue,
        end: &InterpolableValue,
        progress: InterpolationProgress,
    ) -> Result<InterpolableValue, Self::Error>;
    fn add(
        &self,
        property: &PropertyKey,
        underlying: &InterpolableValue,
        value: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error>;
    fn accumulate(
        &self,
        property: &PropertyKey,
        underlying: &InterpolableValue,
        value: &InterpolableValue,
    ) -> Result<InterpolableValue, Self::Error>;
}

/// Composes an endpoint with underlying value in the source-defined order.
///
/// Replace returns the endpoint without invoking an operation. Addition and
/// accumulation receive underlying first, including noncommutative operations.
/// No input is mutated and no partial result is returned on failure.
pub fn compose_value<P: PropertyValueOperations + ?Sized>(
    property: &PropertyKey,
    operation: CompositeOperation,
    underlying: &InterpolableValue,
    value: &InterpolableValue,
    operations: &P,
) -> Result<InterpolableValue, P::Error> {
    match operation {
        CompositeOperation::Replace => Ok(value.clone()),
        CompositeOperation::Add => operations.add(property, underlying, value),
        CompositeOperation::Accumulate => operations.accumulate(property, underlying, value),
    }
}
