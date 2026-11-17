//! Session history entries, navigables and queued history jobs.
//!
//! `DomHostedEntry`, `DomHostedEntryId` and `DomHostedNavigable` read the captured
//! entries and navigables, `DomHostedEntryRequest` and `DomHostedEntryInitialization`
//! create entries, `DomHostedChildHistoryRequest` and `DomHostedChildHistoryPublication`
//! carry the queued creation of a child's entry, and `DomHostedHistoryJobRequest`,
//! `DomHostedHistoryJobReceipt`, `DomHostedHistoryStep` and `DomHostedQueuedHistory`
//! carry queued history jobs.
//!
//! `docs/hosted-documents.md` owns the contract.

pub use crate::internal::hosted::{
    DomHostedChildHistoryPublication, DomHostedChildHistoryRequest, DomHostedEntry,
    DomHostedEntryId, DomHostedEntryInitialization, DomHostedEntryRequest,
    DomHostedHistoryJobReceipt, DomHostedHistoryJobRequest, DomHostedHistoryStep,
    DomHostedNavigable, DomHostedQueuedHistory,
};
