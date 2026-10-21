//! Whole original parsed contributions consumed by the association owner.
use crate::{
    DocumentId, DocumentString, DomError, DomMetaRootCommit, DomParsedHtmlAssociationId,
    DomParsedHtmlHistoricalDocument, DomStamp, NodeId,
};
use surgeist_html::HtmlNodeId;
use surgeist_html_parser::{
    HtmlParseAssociationMissingKind, HtmlParseAssociationOccurrenceId,
    HtmlParseAssociationOccurrenceKind,
};

/// Pure admission, immutable-history classification and destination qualification.
/// No CSSOM owner is accessed by these fronts.
#[derive(Debug)]
pub enum DomParsedHtmlMetaAdmissionError {
    Dom(DomError),
    /// The qualified destination no longer has this pending contribution.
    /// No unbounded completed-identity registry is retained.
    NotPending {
        contribution: DomParsedHtmlAssociationId,
    },
    /// The original finite policy can only be raised componentwise on retry.
    LimitReduction {
        contribution: DomParsedHtmlAssociationId,
    },
    /// Another original contribution must complete first for this Document.
    EarlierPending {
        document: DocumentId,
        pending: DomParsedHtmlAssociationId,
    },
    /// The whole contribution needs an influencing unsupported operation.
    Unsupported {
        contribution: DomParsedHtmlAssociationId,
        occurrence: HtmlParseAssociationOccurrenceId,
        kind: HtmlParseAssociationOccurrenceKind,
    },
    /// Original capture lacks required qualification, rather than an unused context.
    Incomplete {
        contribution: DomParsedHtmlAssociationId,
        missing: HtmlParseAssociationMissingKind,
    },
    /// The original report/Document mapping is not part of this destination.
    DocumentMismatch {
        contribution: DomParsedHtmlAssociationId,
        document: DomParsedHtmlHistoricalDocument,
    },
    /// A destination Document is no longer the actual canonical generation.
    StaleDocument {
        document: DocumentId,
    },
    History(surgeist_html_parser::HtmlParseHistoryError),
}
impl From<DomError> for DomParsedHtmlMetaAdmissionError {
    fn from(value: DomError) -> Self {
        Self::Dom(value)
    }
}
impl std::fmt::Display for DomParsedHtmlMetaAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dom(error) => error.fmt(f),
            other => write!(f, "parsed Meta admission refused: {other:?}"),
        }
    }
}
impl std::error::Error for DomParsedHtmlMetaAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            _ => None,
        }
    }
}
/// A complete original contribution or actual root-qualified producer failure.
/// The native producer diagnostic is moved intact without refusal-time allocation.
#[derive(Debug)]
pub enum DomParsedHtmlMetaError {
    Admission(DomParsedHtmlMetaAdmissionError),
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
impl From<DomParsedHtmlMetaAdmissionError> for DomParsedHtmlMetaError {
    fn from(value: DomParsedHtmlMetaAdmissionError) -> Self {
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
        match self {
            Self::Admission(error) => error.fmt(f),
            Self::Cssom { root, error } => {
                write!(f, "parsed Meta producer for {root:?} refused: {error}")
            }
            other => write!(f, "parsed Meta producer refused: {other:?}"),
        }
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
