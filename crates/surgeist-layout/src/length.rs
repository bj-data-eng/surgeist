//! Lengths, percentages and their resolution, finite and non-negative scalar types, and aspect ratio.
//!
//! A [`LengthPercentageOf`] is an absolute pixel part plus a percentage fraction; [`LengthOf`] adds the `normal`
//! keyword and [`LengthAutoOf`] adds `auto`. A percentage resolves against a [`PercentageBasisOf`], which is either
//! missing or a definite non-negative value, and the result is reported as a [`NumericResolutionOf`],
//! [`LengthResolutionOf`] or [`ResolvedLengthAutoOf`] with a [`LengthResolutionStatus`] or
//! [`UnresolvedLengthReason`] when it cannot be a number. [`NonNegativeFiniteOf`] and [`AspectRatioOf`] are
//! validated scalars. [`NonNegativeFiniteScalarErrorOf`] is the construction error of [`NonNegativeFiniteOf`],
//! [`FiniteScalarErrorOf`] rejects a non-finite pixel or percentage part of a [`LengthPercentageOf`], and
//! [`LengthPercentageErrorOf`] is the construction error of a [`LengthPercentageOf`] from both parts.

pub use crate::value::{
    AspectRatio, AspectRatioOf, FiniteScalarErrorOf, Length, LengthAuto, LengthAutoOf, LengthOf,
    LengthPercentageErrorOf, LengthPercentageOf, LengthResolution, LengthResolutionOf,
    LengthResolutionStatus, NonNegativeFiniteOf, NonNegativeFiniteScalarErrorOf,
    NumericResolutionOf, PercentageBasisOf, ResolvedLengthAuto, ResolvedLengthAutoOf,
    UnresolvedLengthReason,
};
