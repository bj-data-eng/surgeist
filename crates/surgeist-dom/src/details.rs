//! Details element state and its toggle intentions.
//!
//! `DomDetails` reads a Details element, whose open state derives solely from the
//! presence of the exact null-namespace `open` attribute. Changes record ordered toggle
//! intentions (`DomDetailsToggleIntention`, `DomDetailsToggleIntentionId`,
//! `DomDetailsToggleCancellationReason`); `DomDetailsToggleTransfer` hands the raw
//! transitions over before any task-owner coalescing or event dispatch, which the runtime
//! owns.
//!
//! `docs/details-tasks.md` owns the contract.

pub use crate::internal::details::{
    DomDetails, DomDetailsToggleCancellationReason, DomDetailsToggleIntention,
    DomDetailsToggleIntentionId, DomDetailsToggleTransfer,
};
