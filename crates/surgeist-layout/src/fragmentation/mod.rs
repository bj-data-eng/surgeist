//! Deterministic scheduling over mode-owned fragmentation facts.
//!
//! This algorithm does not publish native boxes or authenticate Text. The mode
//! owner supplies actual legal opportunities and source correspondence under
//! each queried fragmentainer. All flows complete or the operation fails.

mod input;
mod output;
mod page;
mod policy;
mod scheduler;
mod validation;
pub use input::*;
pub use output::*;
pub use page::*;
pub use scheduler::resolve_fragmentation;
pub(crate) use scheduler::{
    AcceptedStep, CompletedExecution, Execution, ScopeEndpointOf, ScopeHeadProjectionOf,
    ScopeProjectionOf, ScopeTicket,
};

/// Tests contextual applicability through the same force-policy owner used by
/// scheduling, without exposing a selected Force or selecting a boundary.
pub(crate) fn controls_require_break(
    before: &[FragmentationBreakControl],
    after: &[FragmentationBreakControl],
    available: FragmentationBreakKinds,
) -> bool {
    policy::force_controls(before, after, available).is_some()
}
