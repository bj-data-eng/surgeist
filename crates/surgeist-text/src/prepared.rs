//! Prepared text, its issued capabilities and borrowed views.

pub use crate::protocol::readback::{
    PreparedAlternativeDefinition, PreparedAlternativeView, PreparedAtomicView,
    PreparedBoundaryView, PreparedCellView, PreparedComputedLineBoundaryView, PreparedControlView,
    PreparedOpportunityView, PreparedOriginIntersectionView, PreparedOriginView, PreparedScopeView,
    PreparedUnitView, PreparedVirtualSeparatorView,
};
pub use crate::protocol::{
    LineDirectionFacts, PhaseTwoContext, PreparationOptions, PreparedAlternative,
    PreparedAlternativeAvailability, PreparedAtomic, PreparedBoundary, PreparedBreakKind,
    PreparedCell, PreparedComputedLineBoundary, PreparedControl, PreparedOpportunity,
    PreparedOrigin, PreparedParticipant, PreparedScope, PreparedSourceSegment, PreparedText,
    PreparedUnit, PreparedVirtualSeparator, PresentationRange, ProtocolContextField, ProtocolError,
    ProtocolHandleKind,
};
