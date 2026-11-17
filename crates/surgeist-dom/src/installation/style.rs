//! Pending Style stylesheet associations of a parsed report: prefix, notification, sheet,
//! subject, successor and terminal.
//!
//! These types carry the stages of the whole original Style settlement of a parsed
//! report, in order: the inline-policy prefix, the first sheet publication, the
//! notification permission and synchronous frontier, a successor and a subsequent
//! subject, and the terminal error. The sheet and successor stages have a preparation, a
//! published result, a retirement, a completion and a typed error; the notification stage
//! has a preparation, a published result, a completion and a typed error, plus its
//! handoff types; the subject stage has a completion and an error; the frontier has a
//! dispatch cursor, a synchronous end and an error; the terminal stage is one error.
//!
//! `docs/html-installation.md` (pending stylesheet associations) owns the contract.

pub use crate::internal::parsed_style::{
    DomParsedHtmlExecutionState, DomParsedHtmlFrontierError, DomParsedHtmlStyleCreateSheetRequest,
    DomParsedHtmlStyleError, DomParsedHtmlStyleInitialPolicyRequest,
    DomParsedHtmlStyleInlinePolicyObservation, DomParsedHtmlStyleInlinePolicyRequest,
    DomParsedHtmlStyleNotificationDisposition, DomParsedHtmlStyleNotificationError,
    DomParsedHtmlStyleNotificationHandoff, DomParsedHtmlStyleNotificationHandoffStatus,
    DomParsedHtmlStyleNotificationRequirement, DomParsedHtmlStyleNotificationUnavailable,
    DomParsedHtmlStylePhase, DomParsedHtmlStyleQualification, DomParsedHtmlStyleRenderingProgress,
    DomParsedHtmlStyleRenderingRequirement, DomParsedHtmlStyleResources,
    DomParsedHtmlStyleSheetError, DomParsedHtmlStyleSheetInput, DomParsedHtmlStyleSubjectError,
    DomParsedHtmlStyleSuccessorError, DomParsedHtmlStyleSuccessorRemoval,
    DomParsedHtmlTerminalError,
};
pub use crate::internal::store::pending_associations::{
    DomParsedHtmlContributionCompletion, DomParsedHtmlDispatchCursor, DomParsedHtmlDispatchEntry,
    DomParsedHtmlStyleDeliveryPublication, DomParsedHtmlStyleNotificationCompletion,
    DomParsedHtmlStyleNotificationContinuation, DomParsedHtmlStyleNotificationPreparation,
    DomParsedHtmlStyleNotificationPublished, DomParsedHtmlStylePrefixView,
    DomParsedHtmlStyleRenderingCompletion, DomParsedHtmlStyleRuntimeHold,
    DomParsedHtmlStyleSheetCompletion, DomParsedHtmlStyleSheetPreparation,
    DomParsedHtmlStyleSheetPublished, DomParsedHtmlStyleSheetRetirement,
    DomParsedHtmlStyleSubjectCompletion, DomParsedHtmlStyleSuccessorCompletion,
    DomParsedHtmlStyleSuccessorPreparation, DomParsedHtmlStyleSuccessorPublished,
    DomParsedHtmlStyleSuccessorRetirement, DomParsedHtmlSynchronousEnd,
    DomParsedHtmlSynchronousEndView,
};
