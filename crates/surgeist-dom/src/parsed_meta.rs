//! Whole original parsed contributions consumed by the association owner.
use crate::{
    DocumentId, DomError, DomMetaRootCommit, DomParsedHtmlAssociationAdmissionError,
    DomParsedHtmlAssociationId, DomStamp, NodeId,
};
use surgeist_html::construction::HtmlNodeId;
use surgeist_html::values::DocumentString;
use surgeist_html_parser::HtmlParseAssociationOccurrenceId;

/// A complete original contribution or actual root-qualified producer failure.
/// The native producer diagnostic is moved intact without refusal-time allocation.
#[derive(Debug)]
pub enum DomParsedHtmlMetaError {
    Admission(DomParsedHtmlAssociationAdmissionError),
    OwnerBusy {
        root: NodeId,
    },
    OwnerPoisoned {
        root: NodeId,
    },
    MissingOwner {
        root: NodeId,
    },
    MismatchedOwner {
        root: NodeId,
    },
    StaleProducer {
        root: NodeId,
    },
    Cssom {
        root: NodeId,
        error: surgeist_cssom::CssomError,
    },
}
impl From<DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlMetaError {
    fn from(value: DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Admission(value)
    }
}
impl From<DomError> for DomParsedHtmlMetaError {
    fn from(value: DomError) -> Self {
        Self::Admission(value.into())
    }
}
impl std::fmt::Display for DomParsedHtmlMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Admission(_) => "the whole parsed Meta contribution was refused at admission",
            Self::OwnerBusy { .. } => "the CSSOM owner of the destination root is already borrowed",
            Self::OwnerPoisoned { .. } => {
                "the CSSOM owner of the destination root was poisoned by an earlier failure"
            }
            Self::MissingOwner { .. } => "no CSSOM owner is available for the destination root",
            Self::MismatchedOwner { .. } => {
                "the supplied CSSOM owner is not the owner bound to the destination root"
            }
            Self::StaleProducer { .. } => {
                "the sheet is not in the prepared CSSOM snapshot of the destination root"
            }
            Self::Cssom { .. } => {
                "the CSSOM producer refused the parsed Meta change for the destination root"
            }
        })
    }
}
impl std::error::Error for DomParsedHtmlMetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Admission(error) => Some(error),
            Self::Cssom { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// One original reached operation with exact historical and destination identity.
/// The subject need not have a live installed NodeId.
#[derive(Debug)]
pub struct DomParsedHtmlMetaOccurrence {
    pub(crate) contribution: DomParsedHtmlAssociationId,
    pub(crate) id: HtmlParseAssociationOccurrenceId,
    pub(crate) subject: HtmlNodeId,
    pub(crate) document: DocumentId,
    pub(crate) content: DocumentString,
}
impl DomParsedHtmlMetaOccurrence {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
    pub fn id(&self) -> &HtmlParseAssociationOccurrenceId {
        &self.id
    }
    pub fn subject(&self) -> &HtmlNodeId {
        &self.subject
    }
    pub fn document(&self) -> DocumentId {
        self.document
    }
    pub fn content(&self) -> &DocumentString {
        &self.content
    }
}

/// Actual acknowledgement and producer outputs from one complete contribution.
/// Sidecar completion does not invent a canonical tree revision.
pub struct DomParsedHtmlMetaCompletion {
    pub(crate) contribution: DomParsedHtmlAssociationId,
    pub(crate) stamp: DomStamp,
    pub(crate) roots: Vec<DomMetaRootCommit>,
}
impl std::fmt::Debug for DomParsedHtmlMetaCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlMetaCompletion")
            .field("contribution", &self.contribution)
            .field("stamp", &self.stamp)
            .field("root_count", &self.roots.len())
            .finish()
    }
}
impl DomParsedHtmlMetaCompletion {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
    pub fn stamp(&self) -> DomStamp {
        self.stamp
    }
    pub fn roots(&self) -> &[DomMetaRootCommit] {
        &self.roots
    }
    pub fn into_parts(self) -> (DomParsedHtmlAssociationId, DomStamp, Vec<DomMetaRootCommit>) {
        (self.contribution, self.stamp, self.roots)
    }
}
