//! Original parsed association processing admission, independent of native Meta joins.
use crate::{DocumentId, DomError, DomParsedHtmlAssociationId, DomParsedHtmlHistoricalDocument};
use surgeist_html_parser::{
    HtmlParseAssociationMissingKind, HtmlParseAssociationOccurrenceId,
    HtmlParseAssociationOccurrenceKind,
};

/// Finite additional holder, UTF-16, work and output admission for original parsed association processing.
/// This envelope does not replace canonical DOM, Runtime or producer limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlAssociationLimits {
    pub(crate) retained_bytes: usize,
    pub(crate) retained_units: usize,
    pub(crate) work: u64,
    pub(crate) outputs: usize,
}
impl DomParsedHtmlAssociationLimits {
    pub fn new(
        retained_bytes: usize,
        retained_units: usize,
        work: u64,
        outputs: usize,
    ) -> Result<Self, DomError> {
        retained_units
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Self {
            retained_bytes,
            retained_units,
            work,
            outputs,
        })
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub const fn retained_units(self) -> usize {
        self.retained_units
    }
    pub const fn work(self) -> u64 {
        self.work
    }
    pub const fn outputs(self) -> usize {
        self.outputs
    }
}
/// Actual simultaneous Runtime-owned bank/name/output overlap, excluding the
/// producer's private graph. Repeated reports replace the prior whole report;
/// the composition retains the checked high-water until its fence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomParsedHtmlAssociationRuntimeUsage {
    pub(crate) bytes: usize,
    pub(crate) units: usize,
    pub(crate) outputs: usize,
}
impl DomParsedHtmlAssociationRuntimeUsage {
    pub fn new(bytes: usize, units: usize, outputs: usize) -> Result<Self, DomError> {
        units
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(Self {
            bytes,
            units,
            outputs,
        })
    }
    pub const fn bytes(self) -> usize {
        self.bytes
    }
    pub const fn units(self) -> usize {
        self.units
    }
    pub const fn outputs(self) -> usize {
        self.outputs
    }
}
/// Pure admission, immutable-history classification and destination qualification.
/// No CSSOM owner is accessed by these fronts.
#[derive(Debug)]
pub enum DomParsedHtmlAssociationAdmissionError {
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
impl From<DomError> for DomParsedHtmlAssociationAdmissionError {
    fn from(value: DomError) -> Self {
        Self::Dom(value)
    }
}
impl std::fmt::Display for DomParsedHtmlAssociationAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dom(error) => error.fmt(f),
            other => write!(f, "parsed association admission refused: {other:?}"),
        }
    }
}
impl std::error::Error for DomParsedHtmlAssociationAdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            _ => None,
        }
    }
}
