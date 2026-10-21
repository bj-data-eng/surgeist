//! Exact original inputs and genuine results of the first sheet publication.
use super::*;
use crate::NodeId;
use surgeist_css::{CssMediaQueryList, CssParseReport, CssSheet};
use surgeist_cssom::CssomInputVersion;

/// Actual historical inputs and installation mappings, not native Source facts.
pub struct DomParsedHtmlStyleCreateSheetRequest {
    pub(crate) qualification: DomParsedHtmlStyleInitialPolicyRequest,
    pub(crate) element: NodeId,
    pub(crate) text: DocumentString,
    pub(crate) media: Option<DocumentString>,
    pub(crate) title: Option<DocumentString>,
    pub(crate) parser_document: Option<DocumentId>,
    pub(crate) mode: surgeist_html::HtmlDocumentMode,
}
impl DomParsedHtmlStyleCreateSheetRequest {
    pub fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        &self.qualification
    }
    pub fn element(&self) -> NodeId {
        self.element
    }
    pub fn document(&self) -> DocumentId {
        self.qualification.document()
    }
    pub fn text(&self) -> &DocumentString {
        &self.text
    }
    pub fn media(&self) -> Option<&DocumentString> {
        self.media.as_ref()
    }
    pub fn title(&self) -> Option<&DocumentString> {
        self.title.as_ref()
    }
    pub fn parser_document(&self) -> Option<DocumentId> {
        self.parser_document
    }
    pub fn document_mode(&self) -> surgeist_html::HtmlDocumentMode {
        self.mode
    }
}
/// CSS-owned reports cross into the actual private CSSOM batch. Their grammar
/// work and AST storage remain governed by the CSS/CSSOM producer contracts.
pub struct DomParsedHtmlStyleSheetInput {
    pub(crate) sheet: CssParseReport<CssSheet>,
    pub(crate) media: CssParseReport<CssMediaQueryList>,
    pub(crate) version: CssomInputVersion,
    pub(crate) owner: String,
    pub(crate) disabled: bool,
}
impl DomParsedHtmlStyleSheetInput {
    pub fn new(
        sheet: CssParseReport<CssSheet>,
        media: CssParseReport<CssMediaQueryList>,
        version: CssomInputVersion,
        owner: String,
        disabled: bool,
    ) -> Self {
        Self {
            sheet,
            media,
            version,
            owner,
            disabled,
        }
    }
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleSheetError {
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
    Cssom {
        root: NodeId,
        error: surgeist_cssom::CssomError,
    },
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlStyleSheetError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Style(value)
    }
}
impl From<DomError> for DomParsedHtmlStyleSheetError {
    fn from(value: DomError) -> Self {
        Self::Style(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlStyleSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsed Style sheet preparation refused: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlStyleSheetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(e) => Some(e),
            Self::Cssom { error, .. } => Some(error),
            _ => None,
        }
    }
}
