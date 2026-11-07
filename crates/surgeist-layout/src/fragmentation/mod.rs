//! Deterministic scheduling over mode-owned fragmentation facts.
//!
//! This algorithm does not publish native boxes or authenticate Text. The mode
//! owner supplies actual legal opportunities and source correspondence under
//! each queried fragmentainer. All flows complete or the operation fails.

mod input;
mod output;
mod policy;
mod scheduler;
mod validation;
pub use input::*;
pub use output::*;
pub use scheduler::resolve_fragmentation;
pub(crate) use scheduler::{AcceptedStep, resolve_with_feedback};
