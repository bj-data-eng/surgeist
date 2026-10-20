//! Source-qualified raw declaration-block ingestion through CSS preprocessing.
use crate::{
    CssComponentValueLimits, CssDeclarationList, CssParseReport, CssParseSource, CssParserContext,
    CssRetainedSources, CssSourceRetention, CssSourceRetentionError, CssUtf16InputError,
    CssUtf16InputLimits,
};
use std::sync::Arc;

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

impl CssParserContext {
    /// Preprocesses original UTF16 and parses raw declaration-block contents.
    ///
    /// Uses the same declaration grammar and recovery as
    /// [`Self::parse_declaration_block_contents_with_limits`], without surrounding
    /// braces or a synthetic stylesheet. A root `}` stops declaration consumption;
    /// the component limits still admit the complete normalized input, including
    /// text after that stop. Resource diagnostics remain in the ordinary report.
    ///
    /// `input_limits` bounds preparation and its source backing. Polling occurs
    /// during preparation and immediately before/after grammar execution. General
    /// grammar execution is opaque between those observations: its component,
    /// depth and byte admission is separate from preparation work, and no
    /// per-action grammar work/live-storage accounting is promised. Preparation
    /// refusal or interruption returns no partial source or report.
    pub fn parse_declaration_block_contents_utf16_with_limits<E>(
        self,
        original: Arc<Vec<u16>>,
        input_limits: CssUtf16InputLimits,
        component_limits: CssComponentValueLimits,
        poll: impl FnMut() -> Result<(), E>,
    ) -> Result<CssDeclarationBlockParseReport, CssUtf16InputError<E>> {
        crate::utf16_input::parse_utf16(original, input_limits, poll, |source| {
            let report = crate::parser::parse_declaration_block_contents_snapshot(
                source,
                component_limits,
                self,
            );
            CssDeclarationBlockParseReport {
                source: CssParseSource::new(source.clone(), self),
                report,
            }
        })
    }
}
