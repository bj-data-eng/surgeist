//! Immutable captures for semantic queries and host observations of external state.
//!
//! `DomSnapshot::html_capture` returns a `DomHtmlCapture`, which adapts one snapshot into
//! the HTML semantic producer under `DomHtmlCaptureLimits`; `DomHtmlCaptureError` and
//! `DomHtmlCaptureResource` report refusals and `DomSlotSemanticLimits` bounds the query,
//! mapping and shared producers. `DomScriptingCapture` pairs a snapshot with an
//! owner-sampled scripting fact. `DomHostPublisher` publishes fixed focus, hover or
//! document-target observations from an external producer against one exact
//! `identity::DomDocumentStamp`; `DomHostCapture`, `DomHostObservations`,
//! `DomHostPublication`, `DomHostChange`, `DomHostChanges`, `DomHostChangeCoverage`,
//! `DomHostDomain`, `DomHostDomains`, `DomHostStamp`, `DomHostSourceId`,
//! `DomHostRecomputeReason` and `DomHostError` read and describe them (items whose names
//! begin with DomHosted belong to `hosted` and its submodules).
//!
//! Captures retain exact historical values and keep no live nodes; `HostStateRevision`
//! advances only when observations change. A refused publication preserves the complete
//! previous state. `docs/semantic-captures.md` and `docs/hosted-documents.md` own the
//! contracts.

pub use crate::internal::host::{
    DomHostCapture, DomHostChange, DomHostChangeCoverage, DomHostChanges, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher,
    DomHostRecomputeReason, DomHostSourceId, DomHostStamp, HostStateRevision, Observed,
};
pub use crate::internal::semantic::{
    DomHtmlCapture, DomHtmlCaptureError, DomHtmlCaptureLimits, DomHtmlCaptureResource,
    DomScriptingCapture, DomSlotSemanticLimits,
};
