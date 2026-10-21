//! One authentic original successor, without recreation or whole-history readiness.
use super::*;
use crate::NodeId;

#[derive(Debug)]
pub enum DomParsedHtmlStyleSuccessorError {
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
impl From<surgeist_html::HtmlStyleProtocolError> for DomParsedHtmlStyleSuccessorError {
    fn from(value: surgeist_html::HtmlStyleProtocolError) -> Self {
        Self::Style(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleSuccessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original Style successor refused: {self:?}")
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
pub use crate::store::pending_associations::{
    DomParsedHtmlStyleSuccessorCompletion, DomParsedHtmlStyleSuccessorPreparation,
    DomParsedHtmlStyleSuccessorPublished, DomParsedHtmlStyleSuccessorRetirement,
};
