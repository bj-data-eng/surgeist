//! Original parsed Style prefix; no inline-policy decision or sheet publication.
use crate::{
    DocumentId, DocumentString, DomError, DomParsedHtmlAssociationAdmissionError,
    DomParsedHtmlAssociationId, DomParsedHtmlHistoricalDocument,
};
use surgeist_html::{
    HtmlCspAssociation, HtmlCspProtocolError, HtmlNodeId, HtmlStyleProtocolError, HtmlStyleState,
    HtmlStyleUpdateAction,
};
use surgeist_html_parser::{
    HtmlParseAssociationContextEpochId, HtmlParseAssociationContextRequirement,
    HtmlParseAssociationContextSlotId, HtmlParseAssociationOccurrenceId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomParsedHtmlStylePhase {
    Unstarted,
    AwaitingInitialPolicy,
    InlinePolicyPending,
}
#[derive(Debug)]
pub enum DomParsedHtmlStyleError {
    Admission(DomParsedHtmlAssociationAdmissionError),
    Busy {
        contribution: DomParsedHtmlAssociationId,
    },
    Poisoned {
        contribution: DomParsedHtmlAssociationId,
    },
    UnsupportedProfile {
        contribution: DomParsedHtmlAssociationId,
    },
    WrongRequest,
    AlreadyBound,
    MissingPolicy,
    PolicyContext {
        requirement: HtmlParseAssociationContextRequirement,
    },
    Csp(HtmlCspProtocolError),
    Protocol(HtmlStyleProtocolError),
}
impl From<DomError> for DomParsedHtmlStyleError {
    fn from(e: DomError) -> Self {
        Self::Admission(e.into())
    }
}
impl From<DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlStyleError {
    fn from(e: DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Admission(e)
    }
}
impl From<HtmlCspProtocolError> for DomParsedHtmlStyleError {
    fn from(e: HtmlCspProtocolError) -> Self {
        Self::Csp(e)
    }
}
impl From<HtmlStyleProtocolError> for DomParsedHtmlStyleError {
    fn from(e: HtmlStyleProtocolError) -> Self {
        Self::Protocol(e)
    }
}
impl std::fmt::Display for DomParsedHtmlStyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parsed Style prefix refused: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlStyleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(e) => Some(e),
            Self::Csp(e) => Some(e),
            Self::Protocol(e) => Some(e),
            _ => None,
        }
    }
}

/// Immutable qualification of the actual original constructor-declared slot.
/// Copying it supplies no processing, policy or publication authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlStyleInitialPolicyRequest {
    pub(crate) contribution: DomParsedHtmlAssociationId,
    pub(crate) occurrence: HtmlParseAssociationOccurrenceId,
    pub(crate) subject: HtmlNodeId,
    pub(crate) original_document: DomParsedHtmlHistoricalDocument,
    pub(crate) document: DocumentId,
    pub(crate) context_epoch: HtmlParseAssociationContextEpochId,
    pub(crate) slot: HtmlParseAssociationContextSlotId,
}
impl DomParsedHtmlStyleInitialPolicyRequest {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
    pub fn occurrence(&self) -> &HtmlParseAssociationOccurrenceId {
        &self.occurrence
    }
    pub fn subject(&self) -> &HtmlNodeId {
        &self.subject
    }
    pub fn original_document(&self) -> &DomParsedHtmlHistoricalDocument {
        &self.original_document
    }
    pub fn document(&self) -> DocumentId {
        self.document
    }
    pub fn context_epoch(&self) -> &HtmlParseAssociationContextEpochId {
        &self.context_epoch
    }
    pub fn slot(&self) -> &HtmlParseAssociationContextSlotId {
        &self.slot
    }
}
/// One stored historical inline-policy request, not native current Style source
/// or a Meta CSP insertion request. There is deliberately no reply method.
pub struct DomParsedHtmlStyleInlinePolicyRequest {
    pub(crate) qualification: DomParsedHtmlStyleInitialPolicyRequest,
    pub(crate) text: DocumentString,
    pub(crate) nonce: Option<DocumentString>,
    pub(crate) state: HtmlStyleState,
    pub(crate) policy: HtmlCspAssociation,
    pub(crate) cursor: surgeist_html::HtmlStyleUpdateCursor,
}
impl DomParsedHtmlStyleInlinePolicyRequest {
    pub fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        &self.qualification
    }
    pub fn text(&self) -> &DocumentString {
        &self.text
    }
    pub fn nonce(&self) -> Option<&DocumentString> {
        self.nonce.as_ref()
    }
    pub fn state(&self) -> HtmlStyleState {
        self.state
    }
    pub fn policy(&self) -> &HtmlCspAssociation {
        &self.policy
    }
    pub fn action(&self) -> HtmlStyleUpdateAction {
        self.cursor.action()
    }
}
