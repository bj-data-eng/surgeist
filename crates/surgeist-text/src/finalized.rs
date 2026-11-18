//! Finalized lines, fragments, feedback, measurements, prefixes and readback views.
//!
//! A [`FinalizedLine`] is one immutable selected-line result, retaining its exact request and preparation;
//! [`FinalizedRunView`], [`FinalizedGlyphView`], [`FinalizedCellView`], [`FinalizedControlView`],
//! [`FinalizedFragmentView`] and [`SelectedMappingView`] read it back, and [`FinalizedFragment`] and
//! [`FinalizedFragmentOwner`] identify a fragment and its owner. [`LineMeasurements`], [`InlineMeasure`],
//! [`MarginEdgeFit`] and the `LineEdge*` types hold its measurements, [`LineFeedback`], [`AtomicSlotFeedback`]
//! and [`FormattedLineUsage`] hold feedback and usage, and [`FinalizedPrefixView`],
//! [`FinalizedSharedUnitView`] and [`TextSeam`] observe prefixes. [`LineOutlineInk`] is same-instance
//! glyph-outline ink and [`PresentationMappingKind`] says what happened to a selected source or presentation
//! interval.

pub use crate::protocol::prefix::{FinalizedPrefixView, FinalizedSharedUnitView, TextSeam};
pub use crate::protocol::readback::{
    FinalizedCellView, FinalizedControlView, FinalizedFragmentOwner, FinalizedFragmentView,
    FinalizedGlyphView, FinalizedRunView, LineOutlineInk, PresentationMappingKind,
    SelectedMappingView,
};
pub use crate::protocol::{
    AtomicSlotFeedback, FinalizedFragment, FinalizedLine, FormattedLineUsage, InlineMeasure,
    LineEdgeDisposition, LineEdgeOwner, LineEdgeSide, LineEdgeUnitView, LineFeedback,
    LineMeasurements, MarginEdgeFit, MarginEdgeFitAuthority,
};
