//! Session history entries, navigables and queued history jobs.
//!
//! `DomHostedEntry`, `DomHostedEntryId` and `DomHostedNavigable` read the captured
//! entries and navigables, `DomHostedEntryRequest` and `DomHostedEntryInitialization`
//! create entries, and the `DomHostedHistory*` and `DomHostedQueuedHistory` types carry
//! queued history jobs.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedChildHistoryPublication, DomHostedChildHistoryRequest, DomHostedEntry,
    DomHostedEntryId, DomHostedEntryInitialization, DomHostedEntryRequest,
    DomHostedHistoryJobReceipt, DomHostedHistoryJobRequest, DomHostedHistoryStep,
    DomHostedNavigable, DomHostedQueuedHistory,
};
