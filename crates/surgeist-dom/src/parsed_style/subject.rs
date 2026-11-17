//! The exact next distinct original Style, without removing another subject.
use super::*;

#[derive(Debug)]
pub enum DomParsedHtmlStyleSubjectError {
    Style(DomParsedHtmlStyleError),
    Remaining(DomParsedHtmlStyleNotificationRequirement),
    WrongOccurrence,
    AlreadySelected,
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlStyleSubjectError {
    fn from(e: DomParsedHtmlStyleError) -> Self {
        Self::Style(e)
    }
}
impl From<surgeist_html_parser::HtmlParseUnresolvedReference> for DomParsedHtmlStyleSubjectError {
    fn from(e: surgeist_html_parser::HtmlParseUnresolvedReference) -> Self {
        Self::Style(e.into())
    }
}
impl From<DomError> for DomParsedHtmlStyleSubjectError {
    fn from(e: DomError) -> Self {
        Self::Style(e.into())
    }
}
impl From<DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlStyleSubjectError {
    fn from(e: DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Style(e.into())
    }
}
impl From<surgeist_html::elements::style::HtmlStyleProtocolError>
    for DomParsedHtmlStyleSubjectError
{
    fn from(e: surgeist_html::elements::style::HtmlStyleProtocolError) -> Self {
        Self::Style(e.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleSubjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Style(_) => "the original Style prefix refused the next subject",
            Self::Remaining(requirement) => match requirement {
                DomParsedHtmlStyleNotificationRequirement::Invocation { .. }
                | DomParsedHtmlStyleNotificationRequirement::Occurrence { .. } => {
                    "other influencing original work must run before the next Style subject"
                }
                DomParsedHtmlStyleNotificationRequirement::Unavailable { .. } => {
                    "the original inventory cannot establish the next Style subject"
                }
            },
            Self::WrongOccurrence => {
                "the supplied occurrence is not the next original Style subject"
            }
            Self::AlreadySelected => {
                "the supplied occurrence was already selected or passed by the original sequence"
            }
        })
    }
}
impl std::error::Error for DomParsedHtmlStyleSubjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(e) => Some(e),
            _ => None,
        }
    }
}
