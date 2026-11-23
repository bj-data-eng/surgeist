//! Overflow, overflow clip margin, scrollbar, scroll padding and margin, and scroll snap inputs.
//!
//! [`ComputedOverflow`] is the normalized pair of [`Overflow`] keywords of a box, and [`OverflowClipBox`] with
//! [`OverflowClipMarginOf`] describe the overflow clip edge. [`ScrollbarGutter`] is the normalized gutter reservation
//! policy and [`ScrollbarWidthOf`] the explicit finite thickness selected by the caller's scrollbar environment; layout
//! does not discover host scrollbar metrics. [`ScrollPaddingOf`], [`ScrollPaddingValueOf`] and [`ScrollMarginOf`] are
//! the physical padding and margin of a scroll container and its targets, and [`ScrollSnapType`], [`ScrollSnapAxis`],
//! [`ScrollSnapStrictness`], [`ScrollSnapAlign`], [`ScrollSnapAlignValue`] and [`ScrollSnapStop`] are the normalized
//! snap metadata. [`ComputedOverflowError`] and [`ScrollMarginErrorOf`] are the construction errors.

pub use crate::node_input::{
    ComputedOverflow, ComputedOverflowError, Overflow, OverflowClipBox, OverflowClipMargin,
    OverflowClipMarginOf, ScrollMargin, ScrollMarginError, ScrollMarginErrorOf, ScrollMarginOf,
    ScrollPadding, ScrollPaddingOf, ScrollPaddingValue, ScrollPaddingValueOf, ScrollSnapAlign,
    ScrollSnapAlignValue, ScrollSnapAxis, ScrollSnapStop, ScrollSnapStrictness, ScrollSnapType,
    ScrollbarGutter, ScrollbarWidth, ScrollbarWidthOf,
};
