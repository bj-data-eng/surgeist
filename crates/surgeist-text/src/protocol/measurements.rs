//! Borrowed measurements of one exact finalized selection, not paragraph sizing.
use super::{FinalizedLine, PreparedAlternative, PreparedParticipant, PresentationRange};
use crate::Range;

/// Signed flow and an ordered physical occupied interval in the line's
/// prealignment frame. Indentation is excluded; the consumer adds it once.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InlineMeasure {
    pub(crate) advance: f64,
    pub(crate) occupied_start: f64,
    pub(crate) occupied_end: f64,
}
impl InlineMeasure {
    pub const fn advance(self) -> f64 {
        self.advance
    }
    pub const fn occupied_start(self) -> f64 {
        self.occupied_start
    }
    pub const fn occupied_end(self) -> f64 {
        self.occupied_end
    }
}

/// Actual physical edge, independent of source order and signed flow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEdgeSide {
    Start,
    End,
}

/// Natural classification, frozen before supplied spacing, or an explicit
/// observation that terminal authority remains unconsumed for neutral rows.
/// `eligible` records actual natural overflow, not a frozen pixel length or
/// advance fraction. Spacing cannot reclassify a known row.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineEdgeDisposition {
    #[default]
    Ordinary,
    /// Structural terminal authority was not consumed because this row and
    /// every dependent edge contribution remain neutral. This observation
    /// proves neither an ordinary classification nor a CSS block end.
    UnconsumedTerminal {
        side: LineEdgeSide,
    },
    Removed,
    Unconditional {
        side: LineEdgeSide,
        terminal: bool,
    },
    Conditional {
        side: LineEdgeSide,
        eligible: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EdgeRow {
    pub(crate) punctuation_unit: Option<super::PresentationRange>,
    pub(crate) disposition: LineEdgeDisposition,
    pub(crate) atoms: std::ops::Range<usize>,
    pub(crate) excluded: f64,
    pub(crate) overlap: f64,
}

/// Published targets are real outer contributions. The ordinary branch retains
/// its original cluster-indexed allocation; Cell lines exclude inner rows.
#[derive(Clone, Debug)]
pub(crate) enum EdgeRows {
    Clusters(Vec<EdgeRow>),
    Outer(Vec<OuterEdge>),
}
#[derive(Clone, Debug)]
pub(crate) enum OuterEdge {
    Cluster {
        cluster: usize,
        row: EdgeRow,
    },
    /// The actual opaque Cell result issued by Phase II and sealed by finalization.
    Cell {
        cell: usize,
    },
}
impl EdgeRows {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Clusters(rows) => rows.len(),
            Self::Outer(rows) => rows.len(),
        }
    }
    pub(crate) fn visit_scope_allocations(
        &self,
        visit: &mut crate::engine::scope_alignment::census::AllocationVisitor<'_>,
    ) -> crate::Result<()> {
        use crate::engine::scope_alignment::census::AllocationWitness as W;
        match self {
            Self::Clusters(rows) => visit(W::buffer(rows, rows.len())?),
            Self::Outer(rows) => visit(W::buffer(rows, rows.len())?),
        }
    }
}

/// Text-owned measurements of this exact selected/finalized line. Intrinsic
/// contributions are not a complete paragraph's min/max-content result.
#[derive(Clone, Debug)]
pub struct LineMeasurements {
    pub(crate) formatted_usage: super::FormattedLineUsage,
    pub(crate) fit: InlineMeasure,
    pub(crate) alignment: InlineMeasure,
    pub(crate) minimum: InlineMeasure,
    pub(crate) maximum: InlineMeasure,
    pub(crate) edges: EdgeRows,
}
impl LineMeasurements {
    pub const fn formatted_line_usage(&self) -> super::FormattedLineUsage {
        self.formatted_usage
    }
    pub const fn fit(&self) -> InlineMeasure {
        self.fit
    }
    pub const fn alignment(&self) -> InlineMeasure {
        self.alignment
    }
    pub const fn min_content_contribution(&self) -> InlineMeasure {
        self.minimum
    }
    pub const fn max_content_contribution(&self) -> InlineMeasure {
        self.maximum
    }
}

impl FinalizedLine {
    pub fn measurements(&self) -> &LineMeasurements {
        self.inner
            .data
            .measurements
            .as_ref()
            .expect("finalization installed actual measurements")
    }
    /// Actual Text metric frame, independent of a consumer's intrinsic
    /// allocation. This is line-local and excludes the later placement anchor.
    pub fn metric_bounds(&self) -> crate::Rect {
        let metrics = self.inner.data.metrics;
        crate::Rect::new(
            metrics.inline_min,
            metrics.block_min,
            metrics.inline_max - metrics.inline_min,
            metrics.block_max - metrics.block_min,
        )
    }
    /// Observations of genuine outer text contributions; never admission or replay
    /// capabilities. A Cell is one opaque contribution, including provider-empty Cells.
    /// Shared ordinary glyphs keep their actual per-contribution custody.
    pub fn edge_units(&self) -> impl ExactSizeIterator<Item = LineEdgeUnitView<'_>> {
        (0..self.measurements().edges.len()).map(|index| LineEdgeUnitView { line: self, index })
    }
}

/// An actual participant or computed alternative contributing to this row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineEdgeOwner {
    Participant(PreparedParticipant),
    Alternative(PreparedAlternative),
}

pub struct LineEdgeUnitView<'a> {
    line: &'a FinalizedLine,
    index: usize,
}
enum EdgeTarget<'a> {
    Cluster(usize, &'a EdgeRow),
    Cell(usize, &'a crate::engine::layout::data::FinalizedCell),
}
impl LineEdgeUnitView<'_> {
    fn target(&self) -> EdgeTarget<'_> {
        match &self.line.measurements().edges {
            EdgeRows::Clusters(rows) => EdgeTarget::Cluster(self.index, &rows[self.index]),
            EdgeRows::Outer(rows) => match &rows[self.index] {
                OuterEdge::Cluster { cluster, row } => EdgeTarget::Cluster(*cluster, row),
                OuterEdge::Cell { cell } => {
                    EdgeTarget::Cell(*cell, &self.line.inner.data.cells[*cell])
                }
            },
        }
    }
    pub fn punctuation_unit(&self) -> Option<super::PresentationRange> {
        match self.target() {
            EdgeTarget::Cluster(_, row) => row.punctuation_unit,
            EdgeTarget::Cell(..) => None,
        }
    }
    pub fn finalized_line(&self) -> &FinalizedLine {
        self.line
    }
    pub fn disposition(&self) -> LineEdgeDisposition {
        match self.target() {
            EdgeTarget::Cluster(_, row) => row.disposition,
            EdgeTarget::Cell(..) => LineEdgeDisposition::Ordinary,
        }
    }
    /// Final allocated signed exclusion; it can be negative.
    pub fn excluded_contribution(&self) -> f64 {
        match self.target() {
            EdgeTarget::Cluster(_, row) => row.excluded,
            EdgeTarget::Cell(..) => 0.0,
        }
    }
    /// Nonnegative physical intersection of this classified edge row with the
    /// outward half-line in its prealignment frame. It is not omitted advance.
    /// Ordinary/removed rows and opaque Cells have no Phase II outward assignment.
    pub fn outward_overlap(&self) -> f64 {
        match self.target() {
            EdgeTarget::Cluster(_, row) => row.overlap,
            EdgeTarget::Cell(..) => 0.0,
        }
    }
    /// Actual allocated signed contribution, before any measurement exclusion.
    /// Cell tracking is included; the fixed Cell square is a separate observation.
    pub fn advance(&self) -> f64 {
        match self.target() {
            EdgeTarget::Cluster(index, _) => self.line.inner.data.clusters[index].advance,
            EdgeTarget::Cell(_, cell) => cell.advance,
        }
    }
    /// Allocated contribution across the full final line metric band, after
    /// alignment and before placement. This is neither inner ink nor the Cell square.
    pub fn bounds(&self) -> crate::Rect {
        match self.target() {
            EdgeTarget::Cluster(index, _) => {
                let cluster = &self.line.inner.data.clusters[index];
                let metrics = self.line.metric_bounds();
                crate::Rect::new(
                    cluster.x.min(cluster.x + cluster.advance),
                    metrics.origin.y,
                    cluster.advance.abs(),
                    metrics.size.height,
                )
            }
            EdgeTarget::Cell(_, cell) => crate::engine::layout::data::cell_geometry::edge_rect(
                cell,
                &self.line.inner.data.metrics,
            ),
        }
    }
    pub fn presentation_range(&self) -> PresentationRange {
        let range = match self.target() {
            EdgeTarget::Cluster(index, _) => self.line.inner.data.clusters[index].presentation,
            EdgeTarget::Cell(_, cell) => {
                self.line.inner.selected.presentation.units[cell.unit].presentation
            }
        };
        PresentationRange {
            start: range.start,
            end: range.end,
        }
    }
    /// Actual positive original pieces; the enclosing hull grants no custody.
    pub fn source_pieces(&self) -> impl Iterator<Item = Range> + '_ {
        let (atoms, cell) = match self.target() {
            EdgeTarget::Cluster(_, row) => (row.atoms.clone(), None),
            EdgeTarget::Cell(index, _) => (0..0, Some(self.line.inner.source_cell(index))),
        };
        self.line.inner.selected.presentation.projection.atoms[atoms]
            .iter()
            .map(|atom| atom.source)
            .filter(|range| !range.is_empty())
            .chain(
                cell.into_iter()
                    .flat_map(|cell| cell.source_pieces().map(|piece| piece.aggregate()))
                    .filter(|range| !range.is_empty()),
            )
    }
    pub fn owners(&self) -> impl Iterator<Item = LineEdgeOwner> + '_ {
        let selected = &self.line.inner.selected;
        let (atoms, cell_unit) = match self.target() {
            EdgeTarget::Cluster(_, row) => (row.atoms.clone(), None),
            EdgeTarget::Cell(_, cell) => (0..0, Some(cell.unit)),
        };
        let units = match cell_unit {
            Some(index) => &selected.presentation.units[index..index + 1],
            None => &selected.presentation.units[..],
        };
        units
            .iter()
            .filter(move |unit| {
                cell_unit.is_some() || unit.atoms.start < atoms.end && atoms.start < unit.atoms.end
            })
            .filter_map(|unit| {
                if let super::prepared::PresentationUnitKind::Replacement { alternative, .. } =
                    unit.kind
                {
                    Some(LineEdgeOwner::Alternative(PreparedAlternative {
                        prepared: selected.prepared.clone(),
                        index: alternative,
                    }))
                } else {
                    unit.participant.map(|index| {
                        LineEdgeOwner::Participant(selected.request.participants[index].clone())
                    })
                }
            })
    }
}
