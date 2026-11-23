//! Baseline references, offsets, named baseline outputs and their errors.
//!
//! The inputs choose a reference: [`InlineBaselineContext`] for a native inline formatting context,
//! [`GridBaselineReferences`] for the local column and row alignment contexts of a Grid container, and
//! [`BaselineSource`] for a child's own choice of baseline set. [`BaselineOffsetOf`] is a finite displacement toward
//! the line's over side. The outputs are the first and last physical baselines of a [`BaselinesOf`], and the
//! [`NamedBaselinesOf`] (an alphabetic and a central [`BaselinePairOf`], each a first and a last fact) that are
//! independent of them. A [`BaselineFactOf`] separates unknown authority from the affirmative absence that permits
//! synthesis, and an available one is a [`BaselineCoordinateOf`]. [`InlineBaselineError`] and
//! [`GridBaselineError`] report a supplied baseline fact that cannot be used, and
//! [`BaselineOffsetResolutionError`] a resolved baseline displacement that cannot be normalized.

pub use crate::baseline_offset::{BaselineOffset, BaselineOffsetOf, BaselineOffsetResolutionError};
pub use crate::node_input::{
    BaselineSource, GridBaselineError, GridBaselineReference, GridBaselineReferences,
    InlineBaselineContext, InlineBaselineError,
};
pub use crate::output::{
    BaselineCoordinateOf, BaselineFactOf, BaselinePairOf, Baselines, BaselinesOf, NamedBaselinesOf,
};
