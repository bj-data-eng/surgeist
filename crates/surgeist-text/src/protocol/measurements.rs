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

/// Authority of the operation-specific admission observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarginEdgeFitAuthority {
    /// The existing signed-advance and occupied-span Native contract.
    NativeOnly,
    /// Explicit CSS margin-box flow composed with positioned opaque requirements.
    CssComposed,
}

/// Text-owned indent-free admission, distinct from raw occupied geometry.
/// `CssComposed` is Surgeist's explicit native mixing policy: final signed
/// flow may retract, while independently positioned opaque endpoints remain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarginEdgeFit {
    data: MarginEdgeFitData,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MarginEdgeFitData {
    Native {
        raw: InlineMeasure,
        required: f64,
    },
    CssWithoutOpaque {
        advance: f64,
        required: f64,
    },
    CssWithOpaque {
        advance: f64,
        native_min: f64,
        native_max: f64,
        required: f64,
    },
}
impl MarginEdgeFit {
    pub const fn authority(self) -> MarginEdgeFitAuthority {
        match self.data {
            MarginEdgeFitData::Native { .. } => MarginEdgeFitAuthority::NativeOnly,
            MarginEdgeFitData::CssWithoutOpaque { .. }
            | MarginEdgeFitData::CssWithOpaque { .. } => MarginEdgeFitAuthority::CssComposed,
        }
    }
    pub const fn advance(self) -> f64 {
        match self.data {
            MarginEdgeFitData::Native { raw, .. } => raw.advance,
            MarginEdgeFitData::CssWithoutOpaque { advance, .. }
            | MarginEdgeFitData::CssWithOpaque { advance, .. } => advance,
        }
    }
    /// Signed indent-free policy endpoint, not a rendered or relocated origin.
    pub const fn constraint_start(self) -> f64 {
        match self.data {
            MarginEdgeFitData::Native { raw, .. } => raw.occupied_start,
            MarginEdgeFitData::CssWithoutOpaque { .. } => 0.0,
            MarginEdgeFitData::CssWithOpaque { native_min, .. } => {
                if native_min < 0.0 {
                    native_min
                } else {
                    0.0
                }
            }
        }
    }
    pub const fn constraint_end(self) -> f64 {
        match self.data {
            MarginEdgeFitData::Native { raw, .. } => raw.occupied_end,
            MarginEdgeFitData::CssWithoutOpaque { advance, .. } => advance,
            MarginEdgeFitData::CssWithOpaque {
                advance,
                native_max,
                ..
            } => {
                if native_max > advance {
                    native_max
                } else {
                    advance
                }
            }
        }
    }
    /// Requirement at zero indent. A consumer composing a nonzero indent must
    /// use the actual opaque extrema, not translate the artificial zero anchor.
    pub const fn required_inline_size(self) -> f64 {
        match self.data {
            MarginEdgeFitData::Native { required, .. }
            | MarginEdgeFitData::CssWithoutOpaque { required, .. }
            | MarginEdgeFitData::CssWithOpaque { required, .. } => required,
        }
    }
    /// Unanchored positioned `NativeOccupied` atomic extrema constraining a
    /// `CssComposed` selection. Genuine absence is distinct from a zero-width
    /// opaque interval. `NativeOnly` retains its separate width-only contract
    /// and returns `None` here.
    pub const fn native_occupied_bounds(self) -> Option<(f64, f64)> {
        match self.data {
            MarginEdgeFitData::CssWithOpaque {
                native_min,
                native_max,
                ..
            } => Some((native_min, native_max)),
            MarginEdgeFitData::Native { .. } | MarginEdgeFitData::CssWithoutOpaque { .. } => None,
        }
    }

    pub(crate) fn native(raw: InlineMeasure) -> crate::Result<Self> {
        // Preserve the existing Native width-only reduction, including its
        // original subtraction. Do not introduce an origin anchor or CSS policy.
        let span = raw.occupied_end - raw.occupied_start;
        let required = raw.advance.max(span).max(0.0);
        if !required.is_finite() {
            return Err(crate::error::text_numeric_error(
                "Native fit requirement",
                crate::TextNumericPhase::Finalization,
                crate::TextNumericError::NonFinite,
                required,
            ));
        }
        Ok(Self {
            data: MarginEdgeFitData::Native { raw, required },
        })
    }
    pub(crate) fn css(advance: f64, opaque: Option<(f64, f64)>) -> crate::Result<Self> {
        let (start, end) = opaque.map_or((0.0, advance), |(left, right)| {
            (left.min(0.0), right.max(advance))
        });
        // No implicit zero upper endpoint and no prefix maximum. Floor only
        // the final required size; retain the unanchored opaque operands.
        let required = crate::engine::layout::add(
            "CSS-composed fit width",
            crate::TextNumericPhase::Finalization,
            end,
            -start,
        )?
        .max(0.0);
        let data = match opaque {
            None => MarginEdgeFitData::CssWithoutOpaque { advance, required },
            Some((native_min, native_max)) => MarginEdgeFitData::CssWithOpaque {
                advance,
                native_min,
                native_max,
                required,
            },
        };
        Ok(Self { data })
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
    pub(crate) margin_fit: MarginEdgeFit,
    pub(crate) margin_minimum: MarginEdgeFit,
    pub(crate) margin_maximum: MarginEdgeFit,
    pub(crate) fit: InlineMeasure,
    pub(crate) alignment: InlineMeasure,
    pub(crate) minimum: InlineMeasure,
    pub(crate) maximum: InlineMeasure,
    pub(crate) edges: EdgeRows,
}
impl LineMeasurements {
    pub const fn margin_edge_fit(&self) -> MarginEdgeFit {
        self.margin_fit
    }
    pub const fn margin_edge_min_content_contribution(&self) -> MarginEdgeFit {
        self.margin_minimum
    }
    pub const fn margin_edge_max_content_contribution(&self) -> MarginEdgeFit {
        self.margin_maximum
    }
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

#[cfg(test)]
mod atomic_fit_tests {
    use super::*;

    #[test]
    fn positioned_opaque_envelope_and_final_css_flow_follow_the_native_mixing_policy() {
        // CSS100 + Native6/[0,10]; then CSS−100 + that Native; then
        // the latter selection followed by a genuine CSS100 restoration.
        for (flow, opaque, expected) in [
            (106.0, Some((100.0, 110.0)), (0.0, 110.0, 110.0)),
            (-94.0, Some((-100.0, -90.0)), (-100.0, -90.0, 10.0)),
            (6.0, Some((-100.0, -90.0)), (-100.0, 6.0, 106.0)),
            (-200.0, Some((-100.0, 10.0)), (-100.0, 10.0, 110.0)),
            (-5.0, None, (0.0, -5.0, 0.0)),
            (5.0, None, (0.0, 5.0, 5.0)),
        ] {
            let fit = MarginEdgeFit::css(flow, opaque).unwrap();
            assert_eq!(fit.authority(), MarginEdgeFitAuthority::CssComposed);
            assert_eq!(fit.advance(), flow);
            assert_eq!(
                (
                    fit.constraint_start(),
                    fit.constraint_end(),
                    fit.required_inline_size()
                ),
                expected
            );
        }
    }

    #[test]
    fn native_only_does_not_acquire_the_css_origin_anchor() {
        let raw = InlineMeasure {
            advance: 6.0,
            occupied_start: 100.0,
            occupied_end: 110.0,
        };
        let native = MarginEdgeFit::native(raw).unwrap();
        assert_eq!(native.authority(), MarginEdgeFitAuthority::NativeOnly);
        assert_eq!(
            (native.constraint_start(), native.constraint_end()),
            (100.0, 110.0)
        );
        assert_eq!(native.required_inline_size(), 10.0);
        let css = MarginEdgeFit::css(6.0, Some((100.0, 110.0))).unwrap();
        assert_eq!(css.required_inline_size(), 110.0);
        assert_ne!(native, css);
    }
}
