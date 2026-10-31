//! Font-independent semantic analysis with exact original-source custody.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::engine::{self, AnalyzedData};
use crate::{Direction, Range, Result, Source, Style, ValidatedSource, ValidatedStyle};

pub(crate) mod captured;
mod combined;
mod domain;
pub use captured::CapturedAnalysisOwner;
pub use combined::{AnalyzedCombinedCell, AnalyzedCombinedSourcePiece};
pub use domain::AnalyzedDomain;

/// Resolved source interpretation, established before Unicode bidi analysis.
/// This is independent of font shaping and placed line geometry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AnalysisTypography {
    /// Ordinary Unicode interpretation for horizontal and both sideways profiles.
    /// No computed direction is asserted for the standalone automatic policy.
    HorizontalComposition,
    /// Mixed vertical typography with ordinary paragraph bidi and source-owned orientation.
    Mixed,
    /// Whole-upright strong-L interpretation with a separately retained computed authority.
    Upright {
        /// Actual computed paragraph direction; the used direction is LTR.
        /// This is not a replacement for an already-resolved captured used base.
        computed_direction: crate::ParagraphBaseDirection,
    },
}
impl AnalysisTypography {
    pub(crate) const fn is_upright(self) -> bool {
        matches!(self, Self::Upright { .. })
    }
}

/// Immutable original-source analysis, constructible without a font catalogue.
///
/// Clones retain one analysis identity. Font preparation and placed layouts
/// retain this same allocation; they do not rerun a parallel source analyzer.
#[derive(Clone)]
pub struct AnalyzedSource {
    pub(crate) inner: Arc<AnalyzedData>,
}

impl fmt::Debug for AnalyzedSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalyzedSource")
            .field("source", &self.source())
            .field("direction", &self.direction())
            .finish_non_exhaustive()
    }
}

impl AnalyzedSource {
    /// Completed source cells, before any font preparation or placed geometry.
    pub fn combined_cells(&self) -> impl ExactSizeIterator<Item = AnalyzedCombinedCell<'_>> + '_ {
        let output = self.inner.combination.as_deref();
        (0..output.map_or(0, |output| output.cells.len())).map(move |index| {
            let output = output.expect("nonempty cell iterator");
            AnalyzedCombinedCell {
                core: &self.inner.core,
                output,
                index,
            }
        })
    }
    /// Genuine captured Raw/Joint authority of this completed output.
    #[must_use]
    pub fn captured_owner(&self) -> Option<CapturedAnalysisOwner<'_>> {
        self.inner
            .paragraph
            .scope_topology
            .as_ref()
            .map(|topology| CapturedAnalysisOwner {
                owner: &topology.owner,
            })
    }
    /// Validates source/styles and analyzes the post-transform semantic stream.
    /// Exact original source and Phase-I correspondence remain separately owned.
    /// No font discovery, selection, glyph shaping or placement occurs. Upright
    /// retains its computed direction while executing used-LTR interpretation.
    /// Existing standalone Style admission remains unchanged.
    pub fn new(source: Source, style: Style, typography: AnalysisTypography) -> Result<Self> {
        let source = ValidatedSource::try_from(source)?;
        let style = ValidatedStyle::try_from(style)?;
        Ok(Self {
            inner: engine::analyze(source, style, typography)?,
        })
    }

    pub(crate) fn from_inner(inner: Arc<AnalyzedData>) -> Self {
        Self { inner }
    }

    /// Exact retained original source, including isolated UTF-16 units.
    #[must_use]
    pub fn source(&self) -> &Source {
        self.inner.processed.source()
    }

    /// The same retained executed source correspondence used by preparation.
    #[must_use]
    pub fn processed_source(&self) -> &crate::ProcessedSource {
        &self.inner.processed
    }

    /// The captured paragraph contribution to every native line's metrics.
    #[must_use]
    pub fn paragraph_strut(&self) -> crate::protocol::paragraph::ParagraphStrut {
        self.inner.paragraph.paragraph_strut
    }

    /// Captured default authored style; actual span winners are retained too.
    #[must_use]
    pub fn style(&self) -> &Style {
        self.inner
            .paragraph
            .domain_style
            .as_ref()
            .unwrap_or(self.inner.processed.base_style())
    }

    /// Opaque identity of this analysis allocation, independent of content equality.
    #[must_use]
    pub fn identity(&self) -> AnalysisIdentity {
        AnalysisIdentity {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Original-source grapheme extents from the actual retained analysis.
    pub fn graphemes(&self) -> impl Iterator<Item = Range> + '_ {
        self.inner
            .host_projection
            .graphemes
            .windows(2)
            .map(|pair| Range::new(pair[0], pair[1]))
            .filter(|range| {
                self.inner
                    .paragraph
                    .contains(self.inner.processed.source(), *range)
            })
    }

    /// Actual source interpretation and its separately retained computed authority.
    #[must_use]
    pub fn typography(&self) -> AnalysisTypography {
        self.inner.typography
    }

    /// Resolved used paragraph direction, before glyph substitution.
    /// Upright is LTR without changing its retained computed direction.
    #[must_use]
    pub fn direction(&self) -> Direction {
        if self.inner.analysis.base_level & 1 == 0 {
            Direction::LeftToRight
        } else {
            Direction::RightToLeft
        }
    }

    /// Publishes logical host text without claiming font or positioned geometry.
    #[cfg(feature = "text-accessibility")]
    pub fn accessibility_update(
        &self,
        state: &mut crate::Accessibility,
        parent: accesskit::NodeId,
        focus: accesskit::NodeId,
    ) -> Result<accesskit::TreeUpdate> {
        crate::accessibility::analyzed_update(&self.inner, state, parent, focus)
    }

    /// Maps a source cursor only through this analysis's source-only AX binding.
    #[cfg(feature = "text-accessibility")]
    pub fn access_position(
        &self,
        state: &crate::Accessibility,
        cursor: crate::Cursor,
    ) -> Result<Option<accesskit::TextPosition>> {
        crate::accessibility::analyzed_access_position(&self.inner, state, cursor)
    }

    /// Maps exact source endpoints only through this source-only AX binding.
    #[cfg(feature = "text-accessibility")]
    pub fn access_selection(
        &self,
        state: &crate::Accessibility,
        selection: crate::Selection,
    ) -> Result<Option<accesskit::TextSelection>> {
        crate::accessibility::analyzed_access_selection(&self.inner, state, selection)
    }
}

/// Strong opaque identity of an actual immutable source analysis.
/// Equal strings analyzed independently have different identities.
#[derive(Clone)]
pub struct AnalysisIdentity {
    inner: Arc<AnalyzedData>,
}

impl PartialEq for AnalysisIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for AnalysisIdentity {}
impl Hash for AnalysisIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.inner).hash(state);
    }
}
impl fmt::Debug for AnalysisIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisIdentity").finish_non_exhaustive()
    }
}
