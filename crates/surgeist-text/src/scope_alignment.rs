//! Captured-scope alignment: limits, metrics, allocations and census.

pub use crate::processing::contextual::{
    ScopeAlignmentBoundary, ScopeAlignmentError, ScopeAlignmentLimitKind, ScopeAlignmentLimits,
    ScopeAlignmentUsage, ScopeFontReference,
};
pub use crate::protocol::{
    LineAllocations, PreparedScopeMetrics, ScopeAlignmentAllowance, ScopeAlignmentCensus,
    ScopeAlignmentCensusAccounting, ScopeAlignmentCensusFailure, ScopeAlignmentCensusInput,
    ScopeAlignmentProduct, ScopeAlignmentProducts, ScopeAlignmentRetention,
    ScopeAlignmentWeakProduct, ScopeAllocation, ScopeAllocations, ScopeFrameAllocation,
    ScopeMetricContribution, SelectedScope, scope_alignment_census,
};
