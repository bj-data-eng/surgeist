//! Pending Meta associations of a parsed report.
//!
//! The `DomParsedHtmlMetaOccurrence*` and `DomParsedHtmlMeta*` types prepare, publish,
//! complete and retire the Meta associations of a parsed report:
//! `DomParsedHtmlMetaOccurrence` is one original reached operation with its historical
//! and destination identity, and the whole-contribution types
//! (`DomParsedHtmlMetaPreparation`, `DomParsedHtmlMetaCompletion` and related) consume
//! the report's Meta occurrences together.
//!
//! `docs/html-installation.md` owns the contract.

pub use crate::internal::parsed_meta::{
    DomParsedHtmlMetaCompletion, DomParsedHtmlMetaError, DomParsedHtmlMetaOccurrence,
};
pub use crate::internal::store::pending_associations::{
    DomParsedHtmlMetaOccurrenceCompletion, DomParsedHtmlMetaOccurrenceError,
    DomParsedHtmlMetaOccurrencePreparation, DomParsedHtmlMetaOccurrencePublished,
    DomParsedHtmlMetaOccurrenceRetirement, DomParsedHtmlMetaPreparation,
    DomParsedHtmlMetaPublished, DomParsedHtmlMetaRetirement, DomParsedHtmlMetaRuntimeHold,
};
