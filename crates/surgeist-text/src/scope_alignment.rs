//! Captured-scope alignment: limits, metrics, allocations and census.
//!
//! [`ScopeAlignmentLimits`] holds the explicit feature-local admission limits, [`ScopeAlignmentLimitKind`] names
//! the resource whose admission failed, [`ScopeAlignmentUsage`] holds the executed accounting of one complete
//! transaction, [`ScopeAlignmentAllowance`] is the caller-reduced allowance for one operation and
//! [`ScopeAlignmentError`] reports a typed refusal. [`ScopeAlignmentBoundary`] is the effective formatting
//! classification at a captured inline boundary, [`ScopeFontReference`] the scope-font query context,
//! [`PreparedScopeMetrics`] the retained same-instance metric context and [`ScopeMetricContribution`] one signed
//! local metric contribution. [`ScopeAllocation`], [`ScopeAllocations`], [`ScopeFrameAllocation`] and
//! [`LineAllocations`] hold the allocations, [`SelectedScope`] is an actual captured scope fragment in a
//! selected presentation, and [`ScopeAlignmentRetention`] holds the feature records and live payload an
//! immutable result retains. [`scope_alignment_census`] measures the input and related products as one live allocation
//! union, with [`ScopeAlignmentCensus`], [`ScopeAlignmentCensusInput`], [`ScopeAlignmentCensusAccounting`],
//! [`ScopeAlignmentCensusFailure`], [`ScopeAlignmentProduct`], [`ScopeAlignmentProducts`] and
//! [`ScopeAlignmentWeakProduct`].

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
