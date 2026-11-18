//! Prepared text, its issued capabilities and borrowed views.
//!
//! [`PreparedText`] is the actual immutable preparation and [`PreparationOptions`] its provider-dependent
//! preparation inputs. Its issued capabilities are the `Prepared*` handles ([`PreparedOrigin`],
//! [`PreparedScope`], [`PreparedBoundary`], [`PreparedOpportunity`], [`PreparedUnit`], [`PreparedCell`],
//! [`PreparedControl`], [`PreparedAtomic`] and others), read through their `*View` observations;
//! [`PreparedParticipant`] is the complete logical mixed sequence and [`PreparedBreakKind`] the terminating
//! behavior of an opportunity. [`ProtocolError`], [`ProtocolHandleKind`] and [`ProtocolContextField`] report an
//! invalid association or incomplete input.

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
