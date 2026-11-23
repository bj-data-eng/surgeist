//! Baseline references, offsets, named baseline outputs and their errors.

pub use crate::baseline_offset::{BaselineOffset, BaselineOffsetOf, BaselineOffsetResolutionError};
pub use crate::node_input::{
    BaselineSource, GridBaselineError, GridBaselineReference, GridBaselineReferences,
    InlineBaselineContext, InlineBaselineError,
};
pub use crate::output::{
    BaselineCoordinateOf, BaselineFactOf, BaselinePairOf, Baselines, BaselinesOf, NamedBaselinesOf,
};
