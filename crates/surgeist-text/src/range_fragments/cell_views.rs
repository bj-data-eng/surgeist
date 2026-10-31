//! Frame-specific immutable Range geometry and authentic Cell-unit identity.
use super::*;

/// Geometry retains the metric domain that actually produced it.
#[derive(Clone, Copy, Debug)]
pub enum SourceRangeGeometry<'a> {
    /// Ordinary text in the surrounding line's metric frame.
    Line(SourceRangePiece<'a>),
    /// Font-backed text in an unrotated horizontal Cell frame.
    CellFont(SourceRangeCellFontPiece<'a>),
    /// A genuine native TAB span without a body font or glyph baseline.
    CellTab(SourceRangeCellTabPiece<'a>),
}
impl SourceRangeGeometry<'_> {
    pub fn rect(&self) -> Rect {
        match self {
            Self::Line(piece) => piece.rect(),
            Self::CellFont(piece) => piece.rect(),
            Self::CellTab(piece) => piece.rect(),
        }
    }
}

/// The retained source partition, independent of provider or glyph cardinality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRangeCellUnitKind {
    /// An actual EGC ordinal in the Cell's completed semantic used sequence.
    Semantic { ordinal: usize },
    /// An actual pre-used selection-unit ordinal with no retained geometry unit.
    Removed { ordinal: usize },
}

/// One issued inner unit of an actual Cell in this exact immutable placement.
/// This observation cannot authorize selection, allocation or replay.
#[derive(Clone, Copy)]
pub struct SourceRangeCellUnit<'a> {
    pub(super) layout: &'a Layout,
    pub(super) unit: &'a Unit<'a>,
    pub(super) cell: usize,
    pub(super) kind: SourceRangeCellUnitKind,
}
impl fmt::Debug for SourceRangeCellUnit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceRangeCellUnit")
            .field("line", &self.unit.line)
            .field("cell", &self.cell)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
impl<'a> SourceRangeCellUnit<'a> {
    pub fn same_unit(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.layout.inner, &other.layout.inner)
            && self.unit.line == other.unit.line
            && self.cell == other.cell
            && self.kind == other.kind
    }
    pub fn line_index(&self) -> usize {
        self.unit.line
    }
    /// Actual finalized Cell ordinal within this placed line.
    pub fn cell_index(&self) -> usize {
        self.cell
    }
    pub fn kind(&self) -> SourceRangeCellUnitKind {
        self.kind
    }
    pub fn source_cell(&self) -> crate::AnalyzedCombinedCell<'a> {
        self.layout.inner.lines[self.unit.line]
            .finalized
            .source_cell(self.cell)
    }
    pub fn presentation_range(&self) -> PresentationRange {
        PresentationRange {
            start: self.unit.presentation.start,
            end: self.unit.presentation.end,
        }
    }
    pub fn selected_intersections(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedOriginIntersectionView<'a>> + 'a {
        let prepared = &self.layout.inner.prepared;
        self.unit
            .selected
            .iter()
            .map(move |&association| PreparedOriginIntersectionView {
                prepared,
                association,
            })
    }
    pub fn expanded_intersections(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedOriginIntersectionView<'a>> + 'a {
        let prepared = &self.layout.inner.prepared;
        self.unit
            .expanded
            .iter()
            .map(move |&association| PreparedOriginIntersectionView {
                prepared,
                association,
            })
    }
}

#[derive(Debug)]
pub(super) struct CellFontPiece<'a> {
    pub(super) clusters: std::ops::Range<usize>,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) bounds: Rect,
    pub(super) instance: EffectiveFontInstance<'a>,
    pub(super) ascent: f64,
    pub(super) descent: f64,
    pub(super) frame: SourceRangeCellMetricFrame,
}

/// Real horizontal font metrics mapped through the executed Cell metric frame.
#[derive(Clone, Copy, Debug)]
pub struct SourceRangeCellFontPiece<'a> {
    pub(super) unit: SourceRangeCellUnit<'a>,
    pub(super) piece: &'a CellFontPiece<'a>,
}
impl<'a> SourceRangeCellFontPiece<'a> {
    pub fn rect(&self) -> Rect {
        self.piece.bounds
    }
    pub fn cell_unit(&self) -> SourceRangeCellUnit<'a> {
        self.unit
    }
    /// Directed metric coordinates before horizontal compression and placement.
    pub fn inner_advance_edges(&self) -> (f64, f64) {
        (self.piece.start, self.piece.end)
    }
    /// Actual Cell-internal finalized cluster ordinals, scoped to the owning line.
    pub fn inner_cluster_indices(&self) -> impl ExactSizeIterator<Item = usize> {
        self.piece.clusters.clone()
    }
    pub fn effective_instance(&self) -> EffectiveFontInstance<'a> {
        self.piece.instance
    }
    pub fn font_ascent(&self) -> f64 {
        self.piece.ascent
    }
    /// The retained run's signed extent toward font-under from its metric reference.
    pub fn font_descent(&self) -> f64 {
        self.piece.descent
    }
    pub fn metric_frame(&self) -> SourceRangeCellMetricFrame {
        self.piece.frame
    }
}

/// The actual common horizontal metric reference, distinct from a font-design
/// paint origin. Retained ascent/descent are measured from this reference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceRangeCellMetricFrame {
    pub(super) origin: crate::Point,
    pub(super) x_scale: f64,
}
impl SourceRangeCellMetricFrame {
    pub fn origin(&self) -> crate::Point {
        self.origin
    }
    pub fn x_scale(&self) -> f64 {
        self.x_scale
    }
}

/// Executed native horizontal layout, without a provider-baseline interpretation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceRangeCellLayoutFrame {
    pub(super) origin: crate::Point,
    pub(super) x_scale: f64,
    pub(super) extent: f64,
}
impl SourceRangeCellLayoutFrame {
    /// Physical layout origin after actual inner centering and scope placement.
    pub fn origin(&self) -> crate::Point {
        self.origin
    }
    pub fn x_scale(&self) -> f64 {
        self.x_scale
    }
    /// Actual uncompressed native Cell-em band, not body-font ascent/descent.
    pub fn em_band_extent(&self) -> f64 {
        self.extent
    }
}

#[derive(Debug)]
pub(super) struct CellTabPiece {
    pub(super) span: usize,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) amount: f64,
    pub(super) bounds: Rect,
    pub(super) frame: SourceRangeCellLayoutFrame,
    pub(super) metrics: usize,
}

/// A genuine retained CSS TAB execution; no body font or glyph baseline is lent.
#[derive(Clone, Copy, Debug)]
pub struct SourceRangeCellTabPiece<'a> {
    pub(super) unit: SourceRangeCellUnit<'a>,
    pub(super) piece: &'a CellTabPiece,
}
impl<'a> SourceRangeCellTabPiece<'a> {
    pub fn rect(&self) -> Rect {
        self.piece.bounds
    }
    pub fn cell_unit(&self) -> SourceRangeCellUnit<'a> {
        self.unit
    }
    /// Actual retained span ordinal within the finalized Cell.
    pub fn span_index(&self) -> usize {
        self.piece.span
    }
    pub fn inner_advance_edges(&self) -> (f64, f64) {
        (self.piece.start, self.piece.end)
    }
    pub fn advance(&self) -> f64 {
        self.piece.amount
    }
    pub fn layout_frame(&self) -> SourceRangeCellLayoutFrame {
        self.piece.frame
    }
    /// Genuine native container reference; this is not a body-font metric lender.
    pub fn container_metrics(&self) -> crate::PreparedScopeMetrics<'a> {
        crate::PreparedScopeMetrics {
            prepared: &self.unit.layout.inner.prepared,
            index: self.piece.metrics,
        }
    }
}
