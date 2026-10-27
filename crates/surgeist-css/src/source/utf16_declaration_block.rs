//! Source-qualified raw declaration-block ingestion through CSS preprocessing.
use crate::CssParseReport;
use crate::declaration::CssDeclarationList;
use crate::source::{
    CssParseSource, CssRetainedSources, CssSourceRetention, CssSourceRetentionError,
};

/// A CSS-owned raw declaration-block parse and its complete, actual input.
///
/// The root source is retained even when the input is empty or every declaration
/// is discarded. Declaration and diagnostic origins address this same normalized
/// UTF8 snapshot; the source separately preserves original UTF16 and its exact
/// preprocessing map. Occurrences remain authored data, without CSSOM selection.
///
/// Source storage can be admitted with [`CssSourceRetention`]. Its source receipt
/// does not measure the report's diagnostic or checked-value allocations.
#[derive(Clone, Debug)]
pub struct CssDeclarationBlockParseReport {
    source: CssParseSource,
    report: CssParseReport<CssDeclarationList>,
}

impl CssDeclarationBlockParseReport {
    pub(crate) const fn new(
        source: CssParseSource,
        report: CssParseReport<CssDeclarationList>,
    ) -> Self {
        Self { source, report }
    }

    #[must_use]
    pub const fn source(&self) -> &CssParseSource {
        &self.source
    }

    #[must_use]
    pub const fn report(&self) -> &CssParseReport<CssDeclarationList> {
        &self.report
    }

    #[must_use]
    pub fn into_parts(self) -> (CssParseSource, CssParseReport<CssDeclarationList>) {
        (self.source, self.report)
    }
}

impl CssRetainedSources for CssDeclarationBlockParseReport {
    fn visit_retained_sources(
        &self,
        sources: &mut CssSourceRetention,
    ) -> Result<(), CssSourceRetentionError> {
        // All parsed occurrences and diagnostics are issued from this root,
        // including resource diagnostics without a retained declaration.
        sources.source(self.source.source())
    }
}
