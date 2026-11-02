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
impl From<surgeist_html::HtmlStyleProtocolError> for DomParsedHtmlStyleSubjectError {
    fn from(e: surgeist_html::HtmlStyleProtocolError) -> Self {
        Self::Style(e.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleSubjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original Style subject refused: {self:?}")
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
