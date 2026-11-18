//! Finalized lines, fragments, feedback, measurements, prefixes and readback views.

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
