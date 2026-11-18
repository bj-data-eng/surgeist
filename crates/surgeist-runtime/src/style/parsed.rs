//! Admission and consumption of parsed HTML Style and Meta associations.
//!
//! These items belong to one protocol, all named `DomStyleParsed*`. Each error type reports the refusal of one operation of
//! the protocol: Session intake ([`DomStyleParsedAdmissionError`]), structural installation
//! ([`DomStyleParsedInstallError`]), a parsed Style prefix command ([`DomStyleParsedPrefixError`]), a subject transition
//! ([`DomStyleParsedSubjectError`]), sheet processing ([`DomStyleParsedSheetError`], with
//! [`DomStyleParsedSheetOutcome`]), queueing the networking notification ([`DomStyleParsedNotificationError`], with
//! [`DomStyleParsedNotificationProgress`]), a whole parsed Meta operation ([`DomStyleParsedMetaError`], with
//! [`DomStyleParsedMetaOutcome`] and [`DomStyleParsedMetaPreference`]), per-occurrence Meta preparation
//! ([`DomStyleParsedMetaOccurrenceError`], with [`DomStyleParsedMetaOccurrenceOutcome`]), synchronous-end validation
//! ([`DomStyleParsedFrontierError`]), the successor ([`DomStyleParsedSuccessorError`]) and the final settlement
//! ([`DomStyleParsedTerminalError`]).

pub use crate::internal::dom_style::{
    DomStyleParsedAdmissionError, DomStyleParsedFrontierError, DomStyleParsedInstallError,
    DomStyleParsedMetaError, DomStyleParsedMetaOccurrenceError,
    DomStyleParsedMetaOccurrenceOutcome, DomStyleParsedMetaOutcome, DomStyleParsedMetaPreference,
    DomStyleParsedNotificationError, DomStyleParsedNotificationProgress, DomStyleParsedPrefixError,
    DomStyleParsedSheetError, DomStyleParsedSheetOutcome, DomStyleParsedSubjectError,
    DomStyleParsedSuccessorError, DomStyleParsedTerminalError,
};
