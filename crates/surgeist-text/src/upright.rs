//! Upright and vertical typography diagnostics vocabulary.
//!
//! These types describe the content and script context of an upright operation that failed:
//! [`UprightContent`], [`UprightContentPiece`], [`UprightPresentation`], [`UprightScriptContext`],
//! [`UprightScriptContextFailure`], [`UprightScriptFailure`], [`TextScript`], [`TextStyleOwner`] and
//! [`UprightDecorationRole`], with [`UprightTabMeasurement`], [`VerticalOriginReferenceReason`] and
//! [`VerticalPositionStage`] naming the measurement purpose and the vertical positioning limits.

pub use crate::error::{
    TextScript, TextStyleOwner, UprightContent, UprightContentPiece, UprightDecorationRole,
    UprightPresentation, UprightScriptContext, UprightScriptContextFailure, UprightScriptFailure,
    UprightTabMeasurement, VerticalOriginReferenceReason, VerticalPositionStage,
    VerticalShapingContextReason,
};
