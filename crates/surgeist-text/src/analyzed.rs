//! Font-independent analysis of exact original source and its logical units.

use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::engine::{self, AnalyzedData};
use crate::{Direction, Range, Result, Source, Style, ValidatedSource, ValidatedStyle};

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
    /// Validates authored source/styles and analyzes original Unicode facts.
    /// No font discovery, selection, glyph shaping or placement occurs.
    pub fn new(source: Source, style: Style) -> Result<Self> {
        let source = ValidatedSource::try_from(source)?;
        let style = ValidatedStyle::try_from(style)?;
        Ok(Self {
            inner: engine::analyze(source, style)?,
        })
    }

    pub(crate) fn from_inner(inner: Arc<AnalyzedData>) -> Self {
        Self { inner }
    }

    /// Exact retained original source, including isolated UTF-16 units.
    #[must_use]
    pub fn source(&self) -> &Source {
        &self.inner.source
    }

    /// Captured default authored style; actual span winners are retained too.
    #[must_use]
    pub fn style(&self) -> &Style {
        &self.inner.default_style
    }

    /// Opaque identity of this analysis allocation, independent of content equality.
    #[must_use]
    pub fn identity(&self) -> AnalysisIdentity {
        AnalysisIdentity {
            inner: Arc::clone(&self.inner),
        }
    }

    /// Original-source grapheme extents from the actual retained analysis.
    pub fn graphemes(&self) -> impl ExactSizeIterator<Item = Range> + '_ {
        self.inner
            .projection
            .graphemes
            .windows(2)
            .map(|pair| Range::new(pair[0], pair[1]))
    }

    /// Resolved original-source paragraph direction, before glyph substitution.
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
