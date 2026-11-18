//! Physical placement of finalized lines and atomics.
//!
//! A [`PlacementRequest`] holds a prepared text, a [`PlacementCoverage`] and its [`PlacementEntry`] values. A
//! [`LinePlacement`] places a finalized line at a [`LineAnchor`] together with its [`AtomicPlacement`] values.
//! [`NonFormattedLine`] and [`NonFormattedLinePlacement`] cover nonformatted descendants, and
//! [`BreakFontMetricBasis`] and [`PlacedBreakFontMetrics`] retain the font metric reference of an admitted
//! forced break.

pub use crate::protocol::placement::{BreakFontMetricBasis, PlacedBreakFontMetrics};
pub use crate::protocol::{
    AtomicPlacement, LineAnchor, LinePlacement, NonFormattedLine, NonFormattedLinePlacement,
    PlacementCoverage, PlacementEntry, PlacementRequest,
};
