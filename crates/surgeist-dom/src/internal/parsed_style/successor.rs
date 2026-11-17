//! One authentic original successor, without recreation or whole-history readiness.
use super::*;
use crate::internal::identity::NodeId;

#[derive(Debug)]
pub enum DomParsedHtmlStyleSuccessorError {
    Style(DomParsedHtmlStyleError),
    MissingOwner {
        root: NodeId,
    },
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
    /// The exact next influencing work is outside this ParserPopped leaf.
    Remaining(DomParsedHtmlStyleNotificationRequirement),
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlStyleSuccessorError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Style(value)
    }
}
impl From<DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlStyleSuccessorError {
    fn from(value: DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Style(value.into())
    }
}
impl From<DomError> for DomParsedHtmlStyleSuccessorError {
    fn from(value: DomError) -> Self {
        Self::Style(value.into())
    }
}
impl From<surgeist_html::elements::style::HtmlStyleProtocolError>
    for DomParsedHtmlStyleSuccessorError
{
    fn from(value: surgeist_html::elements::style::HtmlStyleProtocolError) -> Self {
        Self::Style(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleSuccessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Style(_) => "the original Style prefix refused the successor",
            Self::MissingOwner { .. } => {
                "no CSSOM owner was supplied for the sheet that the predecessor created"
            }
            Self::MismatchedOwner { .. } => {
                "the supplied CSSOM owner does not match the owner of the destination root"
            }
            Self::OwnerBusy { .. } => "the CSSOM owner of the destination root is already borrowed",
            Self::OwnerPoisoned { .. } => {
                "the CSSOM owner of the destination root was poisoned by an earlier failure"
            }
            Self::Cssom(_) => "the CSSOM producer refused the Style successor",
            Self::Remaining(requirement) => match requirement {
                DomParsedHtmlStyleNotificationRequirement::Invocation { .. }
                | DomParsedHtmlStyleNotificationRequirement::Occurrence { .. } => {
                    "the next influencing original work is outside this ParserPopped successor"
                }
                DomParsedHtmlStyleNotificationRequirement::Unavailable { .. } => {
                    "the original inventory cannot establish the successor"
                }
            },
        })
    }
}
impl std::error::Error for DomParsedHtmlStyleSuccessorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(e) => Some(e),
            Self::Cssom(e) => Some(e),
            _ => None,
        }
    }
}
pub use crate::internal::store::pending_associations::{
    DomParsedHtmlStyleSuccessorCompletion, DomParsedHtmlStyleSuccessorPreparation,
    DomParsedHtmlStyleSuccessorPublished, DomParsedHtmlStyleSuccessorRetirement,
};

/// Borrowed evidence of the actual removal branch. The opaque completion owns
/// the coupled commits; a known-null removal has no producer or DOM publication.
pub enum DomParsedHtmlStyleSuccessorRemoval<'a> {
    Removed {
        dom: &'a crate::internal::store::DomCommit,
        cssom: &'a surgeist_cssom::CssomCommit,
    },
    NoAssociatedSheet,
}

impl std::fmt::Debug for DomParsedHtmlStyleSuccessorRemoval<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAssociatedSheet => f.write_str("NoAssociatedSheet"),
            Self::Removed { dom, cssom } => f
                .debug_struct("Removed")
                .field("dom", dom)
                .field("cssom", cssom.publication())
                .finish(),
        }
    }
}
