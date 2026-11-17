//! Original notification authority and its canonical synchronous dispatch gate.
use super::*;
use crate::{DomStyleAssociationId, DomStyleNotificationToken, NodeId};

use surgeist_html_parser::{HtmlParseAssociationMissingKind, HtmlParseAssociationOccurrenceKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStyleNotificationUnavailable {
    OriginalEntryMissing,
    UnsupportedInventory,
    IncompleteCapture(HtmlParseAssociationMissingKind),
    InvocationNotExecuted,
    ConsumerFrontierUnavailable,
}
/// Actual retained synchronous work, never a positive dispatch witness. Capture
/// completeness alone does not assert that any consumer operation executed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStyleNotificationRequirement {
    Invocation {
        contribution: DomParsedHtmlAssociationId,
        occurrence: HtmlParseAssociationOccurrenceId,
        document: DocumentId,
        phase: DomParsedHtmlStylePhase,
    },
    Occurrence {
        contribution: DomParsedHtmlAssociationId,
        occurrence: HtmlParseAssociationOccurrenceId,
        document: DocumentId,
        kind: HtmlParseAssociationOccurrenceKind,
    },
    Unavailable {
        contribution: DomParsedHtmlAssociationId,
        reason: DomParsedHtmlStyleNotificationUnavailable,
    },
}
/// Canonical custody readback, not task issuance or resource readiness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStyleNotificationHandoffStatus {
    Retained,
    Transferred,
}
/// An explicit receiving owner for this completed invocation's affine work.
pub enum DomParsedHtmlStyleNotificationHandoff<'a> {
    Retain,
    Transfer(&'a mut Option<DomParsedHtmlStyleNotificationContinuation>),
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleNotificationDisposition {
    Token(DomStyleNotificationToken),
    OwnerAssociationRemoved { association: DomStyleAssociationId },
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleNotificationError {
    Style(DomParsedHtmlStyleError),
    MismatchedOwner { root: NodeId },
    OwnerBusy { root: NodeId },
    OwnerPoisoned { root: NodeId },
    Cssom(surgeist_cssom::CssomError),
    MismatchedAssociation { target: NodeId },
    MismatchedLabel { root: NodeId },
    TargetDestroyed { target: NodeId },
    MissingContinuation,
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlStyleNotificationError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Style(value)
    }
}
impl From<DomError> for DomParsedHtmlStyleNotificationError {
    fn from(value: DomError) -> Self {
        Self::Style(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleNotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Style(_) => "the original Style prefix refused the notification",
            Self::MismatchedOwner { .. } => {
                "the supplied CSSOM owner is not the owner bound to the destination root"
            }
            Self::OwnerBusy { .. } => "the CSSOM owner of the destination root is already borrowed",
            Self::OwnerPoisoned { .. } => {
                "the CSSOM owner of the destination root was poisoned by an earlier failure"
            }
            Self::Cssom(_) => "the CSSOM producer refused the Style notification",
            Self::MismatchedAssociation { .. } => {
                "the current association of the Style element does not match the notified sheet"
            }
            Self::MismatchedLabel { .. } => {
                "the sheet's CSSOM owner label differs from the expected owner label"
            }
            Self::TargetDestroyed { .. } => {
                "the Style element targeted by the notification no longer exists"
            }
            Self::MissingContinuation => "no notification continuation was supplied",
        })
    }
}
impl std::error::Error for DomParsedHtmlStyleNotificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(error) => Some(error),
            Self::Cssom(error) => Some(error),
            _ => None,
        }
    }
}
pub use crate::store::pending_associations::{
    DomParsedHtmlStyleDeliveryPublication, DomParsedHtmlStyleNotificationCompletion,
    DomParsedHtmlStyleNotificationContinuation, DomParsedHtmlStyleNotificationPreparation,
    DomParsedHtmlStyleNotificationPublished,
};
