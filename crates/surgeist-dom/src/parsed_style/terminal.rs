//! Whole settlement is distinct from the original synchronous End.
use super::*;
#[derive(Debug)]
pub enum DomParsedHtmlTerminalError {
    Frontier(crate::DomParsedHtmlFrontierError),
    SynchronousEndRequired {
        contribution: DomParsedHtmlAssociationId,
    },
    CriticalResourcesUnresolved {
        contribution: DomParsedHtmlAssociationId,
    },
    NotificationsUnsettled {
        contribution: DomParsedHtmlAssociationId,
        remaining: std::num::NonZeroU64,
    },
}
impl From<surgeist_html_parser::HtmlParseUnresolvedReference> for DomParsedHtmlTerminalError {
    fn from(error: surgeist_html_parser::HtmlParseUnresolvedReference) -> Self {
        Self::Frontier(error.into())
    }
}
impl From<DomError> for DomParsedHtmlTerminalError {
    fn from(error: DomError) -> Self {
        Self::Frontier(error.into())
    }
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlTerminalError {
    fn from(error: DomParsedHtmlStyleError) -> Self {
        Self::Frontier(error.into())
    }
}
impl From<crate::DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlTerminalError {
    fn from(error: crate::DomParsedHtmlAssociationAdmissionError) -> Self {
        DomParsedHtmlStyleError::from(error).into()
    }
}
impl std::fmt::Display for DomParsedHtmlTerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original contribution settlement refused: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlTerminalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frontier(error) => Some(error),
            _ => None,
        }
    }
}
pub use crate::store::pending_associations::DomParsedHtmlContributionCompletion;
