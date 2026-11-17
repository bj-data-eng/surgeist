//! Canonical admission and end of original synchronous execution.
use super::*;

/// Scheduling state of the original contribution. Request preparation and
/// policy binding do not activate execution. End excludes asynchronous work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlExecutionState {
    Unstarted,
    Activated,
    SynchronousEnd,
}
#[derive(Debug)]
pub enum DomParsedHtmlFrontierError {
    Style(DomParsedHtmlStyleError),
    Remaining(DomParsedHtmlStyleNotificationRequirement),
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlFrontierError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Style(value)
    }
}
impl From<surgeist_html_parser::HtmlParseUnresolvedReference> for DomParsedHtmlFrontierError {
    fn from(value: surgeist_html_parser::HtmlParseUnresolvedReference) -> Self {
        Self::Style(value.into())
    }
}
impl From<DomError> for DomParsedHtmlFrontierError {
    fn from(value: DomError) -> Self {
        Self::Style(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlFrontierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Style(_) => {
                "the original Style prefix refused the original synchronous end or its settlement"
            }
            Self::Remaining(requirement) => match requirement {
                DomParsedHtmlStyleNotificationRequirement::Invocation { .. }
                | DomParsedHtmlStyleNotificationRequirement::Occurrence { .. } => {
                    "influencing original work remains before the synchronous end"
                }
                DomParsedHtmlStyleNotificationRequirement::Unavailable { .. } => {
                    "the original inventory cannot establish the synchronous end"
                }
            },
        })
    }
}
impl std::error::Error for DomParsedHtmlFrontierError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(error) => Some(error),
            Self::Remaining(_) => None,
        }
    }
}
pub use crate::internal::store::pending_associations::{
    DomParsedHtmlDispatchCursor, DomParsedHtmlDispatchEntry, DomParsedHtmlSynchronousEnd,
    DomParsedHtmlSynchronousEndView,
};
