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
    pub(crate) disposition: LineEdgeDisposition,
    pub(crate) atoms: std::ops::Range<usize>,
    pub(crate) excluded: f64,
    pub(crate) overlap: f64,
}

/// Text-owned measurements of this exact selected/finalized line. Intrinsic
/// contributions are not a complete paragraph's min/max-content result.
#[derive(Clone, Debug)]
pub struct LineMeasurements {
    pub(crate) fit: InlineMeasure,
    pub(crate) alignment: InlineMeasure,
    pub(crate) minimum: InlineMeasure,
    pub(crate) maximum: InlineMeasure,
    pub(crate) edges: Vec<EdgeRow>,
}
impl LineMeasurements {
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
    /// Observations of genuine finalized text rows; never admission or replay
    /// capabilities. Shared glyphs keep their actual per-contribution custody.
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
impl LineEdgeUnitView<'_> {
    pub fn finalized_line(&self) -> &FinalizedLine {
        self.line
    }
    pub fn disposition(&self) -> LineEdgeDisposition {
        self.line.measurements().edges[self.index].disposition
    }
    /// Final allocated signed exclusion; it can be negative.
    pub fn excluded_contribution(&self) -> f64 {
        self.line.measurements().edges[self.index].excluded
    }
    /// Nonnegative physical intersection of this classified edge row with the
    /// outward half-line in its prealignment frame. It is not omitted advance;
    /// even an unconditionally hanging row can be physically inside the frame.
    /// Ordinary/removed rows have no Phase II outward assignment.
    pub fn outward_overlap(&self) -> f64 {
        self.line.measurements().edges[self.index].overlap
    }
    /// Actual allocated signed contribution, before any measurement exclusion.
    pub fn advance(&self) -> f64 {
        self.line.inner.data.clusters[self.index].advance
    }
    /// Actual final line-local geometry after alignment, before placement.
    pub fn bounds(&self) -> crate::Rect {
        let cluster = &self.line.inner.data.clusters[self.index];
        let metrics = self.line.metric_bounds();
        crate::Rect::new(
            cluster.x.min(cluster.x + cluster.advance),
            metrics.origin.y,
            cluster.advance.abs(),
            metrics.size.height,
        )
    }
    pub fn presentation_range(&self) -> PresentationRange {
        let range = self.line.inner.data.clusters[self.index].presentation;
        PresentationRange {
            start: range.start,
            end: range.end,
        }
    }
    /// Actual positive original pieces; the enclosing hull grants no custody.
    pub fn source_pieces(&self) -> impl Iterator<Item = Range> + '_ {
        let row = &self.line.measurements().edges[self.index];
        self.line.inner.selected.presentation.projection.atoms[row.atoms.clone()]
            .iter()
            .map(|a| a.source)
            .filter(|r| !r.is_empty())
    }
    pub fn owners(&self) -> impl Iterator<Item = LineEdgeOwner> + '_ {
        let selected = &self.line.inner.selected;
        let atoms = &self.line.measurements().edges[self.index].atoms;
        selected
            .presentation
            .units
            .iter()
            .filter(move |unit| unit.atoms.start < atoms.end && atoms.start < unit.atoms.end)
            .filter_map(|unit| {
                if let super::prepared::PresentationUnitKind::Replacement { alternative } =
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
