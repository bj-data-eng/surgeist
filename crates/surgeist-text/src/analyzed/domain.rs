//! Atomic domain analysis issued only from an authentic processed member.
use crate::{
    AnalysisTypography, AnalyzedSource, CapturedScope, ProcessedParagraph, ProcessedSource, Result,
};
use std::sync::Arc;

/// One immutable, complete source-domain analysis batch.
/// Outputs borrow this batch; construction publishes no partial batch.
pub struct AnalyzedDomain {
    pub(crate) processed: Arc<ProcessedSource>,
    pub(crate) domain: CapturedScope,
    pub(crate) outputs: Vec<AnalyzedSource>,
}
impl AnalyzedDomain {
    pub fn from_processed_paragraph(
        member: ProcessedParagraph,
        typography: AnalysisTypography,
    ) -> Result<Self> {
        let processed = Arc::clone(member.processed_source());
        processed.admit_paragraph(&member)?;
        crate::engine::analyze_domain(processed, member, typography)
    }
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = &AnalyzedSource> {
        self.outputs.iter()
    }
    #[must_use]
    pub fn processed_source(&self) -> &Arc<ProcessedSource> {
        &self.processed
    }
    #[must_use]
    pub fn domain(&self) -> CapturedScope {
        self.domain.clone()
    }
}
impl std::fmt::Debug for AnalyzedDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyzedDomain")
            .field("domain", &self.domain)
            .field("outputs", &self.outputs.len())
            .finish()
    }
}
