//! Box generation, display, position, float and clear, containment, and box alignment.
//!
//! [`Display`], [`Position`], [`BoxSizing`], [`Float`], [`Clear`] and [`SizeContainment`] are keywords of
//! [`NodeInputOf`](crate::node::NodeInputOf). [`BlockFlowContext`] and [`BlockGeneration`]
//! describe block formatting-context establishment and the generation role of a normalized block box. [`FloatExclusion`]
//! selects the inline-wrapping exclusion geometry of a float; for a shape, the host answers a
//! [`FloatExclusionQueryOf`] with a [`FloatExclusionIntervalOf`] through
//! [`LayoutTree::float_exclusion_interval`](crate::tree::LayoutTree::float_exclusion_interval). [`AlignContent`],
//! [`AlignItems`] and [`ItemAlignmentPosition`] are the box-alignment keywords.

pub use crate::node_input::{
    AlignContent, AlignItems, BlockFlowContext, BlockGeneration, BoxSizing, Clear, Display, Float,
    FloatExclusion, FloatExclusionInterval, FloatExclusionIntervalError,
    FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQuery,
    FloatExclusionQueryOf, ItemAlignmentPosition, Position, SizeContainment,
};
