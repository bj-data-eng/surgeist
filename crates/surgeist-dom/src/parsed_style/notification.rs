//! Original queue authority and precise absence of a synchronous dispatch frontier.
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
#[derive(Debug)]
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
/// Canonical original permission disposition; Issued does not itself assert
/// that a composing Runtime installed or delivered an element task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStyleNotificationStatus {
    NotIssued,
    Issued,
    OwnerAssociationRemoved,
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleNotificationDisposition {
    Token(DomStyleNotificationToken),
    OwnerAssociationRemoved { association: DomStyleAssociationId },
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleNotificationError {
    Style(DomParsedHtmlStyleError),
    MismatchedOwner {
        root: NodeId,
    },
    OwnerBusy {
        root: NodeId,
    },
    OwnerPoisoned {
        root: NodeId,
    },
    Cssom(surgeist_cssom::CssomError),
    MismatchedAssociation {
        target: NodeId,
    },
    MismatchedLabel {
        root: NodeId,
    },
    TargetDestroyed {
        target: NodeId,
    },
    AlreadyIssued {
        contribution: DomParsedHtmlAssociationId,
    },
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
        write!(f, "original Style notification refused: {self:?}")
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
    DomParsedHtmlStyleNotificationPreparation, DomParsedHtmlStyleNotificationPublished,
};
