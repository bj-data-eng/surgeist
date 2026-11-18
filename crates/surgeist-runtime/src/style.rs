//! Style orchestration inside a session.
//!
//! [`DomStyleRuntime`] is the Style composition owner; its limits are [`DomStyleRuntimeLimits`] and its failures
//! [`DomStyleRuntimeError`]. A caller's composition context supplies it through [`DomStyleEventContext`], and networking
//! tasks are reported as [`DomStyleNetworkingProgress`] with a [`DomStyleNetworkingTaskId`]. Native Meta preparation
//! returns a [`DomStyleMetaOrdinaryOutcome`] or [`DomStyleMetaSourceOutcome`], whose [`DomStyleMetaPreference`] values record
//! the preferred-name operations reached, or a [`DomStyleMetaError`]. A [`DomStyleSheetSetObservation`] is an immutable view of a Document's preference
//! history and a [`DomStyleSheetSetCommit`] one CSSOM commit with the preference state published with it.
//! [`DomStyleRuntime::construct_document_sheet`] constructs a sheet in a native top-level Window and returns a
//! [`DomConstructedSheet`]; a [`DomConstructedSheetOwnerInit`] supplies the Style policy for a Document without a producer yet.
//! [`DomStyleRootWithdrawal`] records the withdrawal of a root for hosted destruction, and
//! refusals of that withdrawal are [`DomStyleRootWithdrawalError`]. The protocol for parsed HTML
//! associations is in [`parsed`].

pub mod parsed;

pub use crate::internal::dom_style::{
    DomConstructedSheet, DomConstructedSheetError, DomConstructedSheetOwnerInit,
    DomStyleDisabledObservation, DomStyleEventContext, DomStyleMetaError,
    DomStyleMetaOrdinaryOutcome, DomStyleMetaPreference, DomStyleMetaSourceOutcome,
    DomStyleNetworkingProgress, DomStyleNetworkingTaskId, DomStyleProcessedSheet,
    DomStyleRootWithdrawal, DomStyleRootWithdrawalError, DomStyleRuntime, DomStyleRuntimeError,
    DomStyleRuntimeLimits, DomStyleSheetSetCommit, DomStyleSheetSetObservation,
    DomStyleUnavailable, DomStyleWithdrawnSheetSetHistory,
};
