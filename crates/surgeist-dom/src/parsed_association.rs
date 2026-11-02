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
    /// A history read that the installer itself set up wrongly. A reference the
    /// report cannot resolve is [`Self::UnresolvedReport`], never this.
    History(DomParsedHtmlHistoryReadError),
    /// A preferred-name change of the report retains no original preferred
    /// name, so its history cannot be read.
    MissingPreferredName {
        contribution: DomParsedHtmlAssociationId,
        occurrence: HtmlParseAssociationOccurrenceId,
    },
    /// The report holds a reference to a record its own tables cannot resolve,
    /// so the contribution cannot be classified.
    UnresolvedReport(surgeist_html_parser::HtmlParseUnresolvedReference),
}
/// How an installer's own history read was set up wrongly. The report cannot
/// cause either: the installer names a group of the report it reads and sizes
/// its scratch from that report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomParsedHtmlHistoryReadError {
    /// The group belongs to another report.
    ForeignOwner,
    /// The scratch frames supplied are fewer than the history needs.
    InsufficientScratch { required: usize, supplied: usize },
}
impl std::fmt::Display for DomParsedHtmlHistoryReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignOwner => f.write_str("the history group belongs to another report"),
            Self::InsufficientScratch { required, supplied } => write!(
                f,
                "the history read needs {required} scratch frames but {supplied} were supplied"
            ),
        }
    }
}
impl std::error::Error for DomParsedHtmlHistoryReadError {}
impl From<surgeist_html_parser::HtmlParseHistoryError> for DomParsedHtmlAssociationAdmissionError {
    /// A reference the report cannot resolve takes the one shape every other
    /// reader fault takes; only the installer's own misuse is a history error.
    fn from(value: surgeist_html_parser::HtmlParseHistoryError) -> Self {
        use surgeist_html_parser::HtmlParseHistoryError as Read;
        match value {
            Read::Unresolved(fault) => Self::UnresolvedReport(fault),
            Read::ForeignOwner => Self::History(DomParsedHtmlHistoryReadError::ForeignOwner),
            Read::InsufficientScratch { required, supplied } => {
                Self::History(DomParsedHtmlHistoryReadError::InsufficientScratch {
                    required,
                    supplied,
                })
            }
        }
    }
}
impl From<surgeist_html_parser::HtmlParseUnresolvedReference>
    for DomParsedHtmlAssociationAdmissionError
{
    fn from(value: surgeist_html_parser::HtmlParseUnresolvedReference) -> Self {
        Self::UnresolvedReport(value)
    }
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
            Self::History(error) => Some(error),
            Self::UnresolvedReport(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;
    use surgeist_html_parser::HtmlParseHistoryError as Read;

    #[test]
    fn a_history_read_misuse_is_a_history_error_with_a_source_and_a_message() {
        let foreign = DomParsedHtmlAssociationAdmissionError::from(Read::ForeignOwner);
        assert!(matches!(
            foreign,
            DomParsedHtmlAssociationAdmissionError::History(
                DomParsedHtmlHistoryReadError::ForeignOwner
            )
        ));
        assert_eq!(
            foreign.source().unwrap().to_string(),
            "the history group belongs to another report"
        );

        let scratch = DomParsedHtmlAssociationAdmissionError::from(Read::InsufficientScratch {
            required: 7,
            supplied: 3,
        });
        assert!(matches!(
            scratch,
            DomParsedHtmlAssociationAdmissionError::History(
                DomParsedHtmlHistoryReadError::InsufficientScratch {
                    required: 7,
                    supplied: 3
                }
            )
        ));
        assert_eq!(
            scratch.source().unwrap().to_string(),
            "the history read needs 7 scratch frames but 3 were supplied"
        );
    }
}
