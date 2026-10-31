//! Nonempty original-source observations over placed, selected grapheme units.
use crate::protocol::{
    ProtocolError, ProtocolHandleKind,
    paragraph::{FragmentOwner, OriginIntersection},
    prepared::{AlternativeDefinition, PresentationUnitKind, SelectedRelationEvent},
    protocol_error,
    readback::PreparedOriginIntersectionView,
};
use crate::{
    EffectiveFontInstance, Error, ErrorCode, ErrorDetail, FinalizedFragment, FinalizedLine, Layout,
    PreparedAlternative, PreparedControl, PreparedOrigin, PreparedVirtualSeparator,
    PresentationRange, Range, Rect, Result, SpacingSlotDisposition, TextNumericPhase,
};
mod cell_views;
mod cells;
mod indexes;
mod metric_geometry;
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;
use cell_views::{CellFontPiece, CellTabPiece};
pub use cell_views::{
    SourceRangeCellFontPiece, SourceRangeCellLayoutFrame, SourceRangeCellMetricFrame,
    SourceRangeCellTabPiece, SourceRangeCellUnit, SourceRangeCellUnitKind, SourceRangeGeometry,
};
use indexes::LineIndexes;
use metric_geometry::{GeometryCache, MetricUnits, range_geometry};

use std::{fmt, sync::Arc};

/// A valid source request cannot invent a placed unit or an actual font lender.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRangeFragmentError {
    EmptyRange,
    OutsideOrigin {
        origin: usize,
        local: Range,
        captured: Range,
    },
    OutsidePlacement {
        aggregate: Range,
    },
    MissingOriginalMapping {
        aggregate: Range,
    },
    AmbiguousOriginalMapping {
        aggregate: Range,
    },
    MissingUnitGeometry {
        line: usize,
        unit: usize,
    },
    UnsupportedPartition {
        line: usize,
        unit: usize,
    },
    MissingFontMetrics {
        line: usize,
        unit: usize,
    },
    /// The actual retained font/native-layout reference for this inner EGC is missing.
    MissingCellReference {
        line: usize,
        cell: usize,
        unit: usize,
    },
    /// Actual provider members cannot be partitioned into this semantic EGC.
    UnsupportedCellPartition {
        line: usize,
        cell: usize,
        unit: usize,
    },
    /// A real pre-used source unit has no unique retained inner-unit closure.
    AmbiguousCellUnit {
        line: usize,
        cell: usize,
        selection_unit: usize,
    },
}
fn failure(reason: SourceRangeFragmentError) -> Error {
    let code = match reason {
        SourceRangeFragmentError::MissingFontMetrics { .. }
        | SourceRangeFragmentError::UnsupportedPartition { .. }
        | SourceRangeFragmentError::MissingCellReference { .. }
        | SourceRangeFragmentError::UnsupportedCellPartition { .. }
        | SourceRangeFragmentError::AmbiguousCellUnit { .. } => ErrorCode::UnsupportedFeature,
        SourceRangeFragmentError::MissingOriginalMapping { .. }
        | SourceRangeFragmentError::AmbiguousOriginalMapping { .. }
        | SourceRangeFragmentError::MissingUnitGeometry { .. } => ErrorCode::LayoutFailed,
        _ => ErrorCode::InvalidRange,
    };
    Error::new(
        code,
        "nonempty source range fragments could not be observed",
    )
    .with_detail(ErrorDetail::SourceRangeFragments { reason })
}

/// Complete eagerly validated observations, borrowing their actual placement.
/// Empty observations can still account for a fully covered removed range.
#[derive(Debug)]
pub struct SourceRangeFragments<'a> {
    layout: &'a Layout,
    origin: PreparedOrigin,
    local: Range,
    units: Vec<Unit<'a>>,
    usage: Option<crate::ScopeAlignmentUsage>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnitIdentity {
    Line(usize),
    Cell {
        cell: usize,
        kind: SourceRangeCellUnitKind,
    },
}
#[derive(Debug)]
struct Unit<'a> {
    line: usize,
    identity: UnitIdentity,
    order: (usize, usize),
    selected: Vec<OriginIntersection>,
    expanded: Vec<OriginIntersection>,
    fragments: Vec<usize>,
    pieces: Vec<GeometryPiece<'a>>,
    alternatives: Vec<usize>,
    controls: Vec<usize>,
    virtuals: Vec<usize>,
    defining: Vec<OriginIntersection>,
    attributed_cuts: Vec<usize>,
    presentation: Range,
}
#[derive(Debug)]
enum GeometryPiece<'a> {
    Line(Piece<'a>),
    CellFont(CellFontPiece<'a>),
    CellTab(CellTabPiece),
}
#[derive(Debug)]
struct Piece<'a> {
    clusters: std::ops::Range<usize>,
    start: f64,
    end: f64,
    bounds: Rect,
    instance: EffectiveFontInstance<'a>,
    ascent: f64,
    descent: f64,
    baseline: f64,
}
impl SourceRangeFragments<'_> {
    pub fn origin(&self) -> &PreparedOrigin {
        &self.origin
    }
    pub fn local_range(&self) -> Range {
        self.local
    }
    pub fn layout(&self) -> &Layout {
        self.layout
    }
    /// Actual operation-local accounting when a Cell activates the query ledger.
    pub fn scope_alignment_usage(&self) -> Option<crate::ScopeAlignmentUsage> {
        self.usage
    }
    pub fn units(&self) -> impl ExactSizeIterator<Item = SourceRangeUnit<'_>> {
        self.units.iter().map(|unit| SourceRangeUnit {
            layout: self.layout,
            unit,
        })
    }
}
/// One selected grapheme unit; equal geometry or original offsets are not identity.
#[derive(Clone, Copy)]
pub struct SourceRangeUnit<'a> {
    layout: &'a Layout,
    unit: &'a Unit<'a>,
}
impl fmt::Debug for SourceRangeUnit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceRangeUnit")
            .field("line", &self.unit.line)
            .field("pieces", &self.unit.pieces.len())
            .finish_non_exhaustive()
    }
}
impl<'a> SourceRangeUnit<'a> {
    pub fn same_unit(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.layout.inner, &other.layout.inner)
            && self.unit.line == other.unit.line
            && self.unit.identity == other.unit.identity
    }
    pub fn line_index(&self) -> usize {
        self.unit.line
    }
    /// All actual contributing alternatives, independent of equal anchors.
    pub fn alternatives(&self) -> impl ExactSizeIterator<Item = PreparedAlternative> + '_ {
        self.unit
            .alternatives
            .iter()
            .map(|&index| PreparedAlternative {
                prepared: Arc::clone(&self.layout.inner.prepared),
                index,
            })
    }
    /// All genuine generated controls contributing to this actual Range unit.
    pub fn controls(&self) -> impl ExactSizeIterator<Item = PreparedControl> + '_ {
        self.unit.controls.iter().map(|&index| PreparedControl {
            prepared: Arc::clone(&self.layout.inner.prepared),
            index,
        })
    }
    /// All actual virtual occurrences contributing to this selected grapheme.
    pub fn virtual_separators(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedVirtualSeparator> + '_ {
        self.unit
            .virtuals
            .iter()
            .map(|&index| PreparedVirtualSeparator {
                prepared: Arc::clone(&self.layout.inner.prepared),
                index,
            })
    }
    /// Complete Source definitions, distinct from source-empty associations.
    pub fn defining_sources(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedOriginIntersectionView<'a>> + '_ {
        self.unit
            .defining
            .iter()
            .map(|&association| PreparedOriginIntersectionView {
                prepared: &self.layout.inner.prepared,
                association,
            })
    }
    /// Exact automatic source-attribution relations of every contributing owner.
    pub fn hyphenation_cuts(
        &self,
    ) -> impl ExactSizeIterator<Item = crate::PreparedHyphenationCut> + '_ {
        self.unit
            .attributed_cuts
            .iter()
            .map(|&index| crate::PreparedHyphenationCut {
                prepared: Arc::clone(&self.layout.inner.prepared),
                index,
            })
    }
    pub fn presentation_range(&self) -> PresentationRange {
        PresentationRange {
            start: self.unit.presentation.start,
            end: self.unit.presentation.end,
        }
    }
    pub fn finalized_line(&self) -> FinalizedLine {
        FinalizedLine {
            inner: Arc::clone(&self.layout.inner.lines[self.unit.line].finalized),
        }
    }
    pub fn selected_intersections(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedOriginIntersectionView<'_>> {
        self.unit
            .selected
            .iter()
            .map(|&association| PreparedOriginIntersectionView {
                prepared: &self.layout.inner.prepared,
                association,
            })
    }
    pub fn expanded_intersections(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedOriginIntersectionView<'_>> {
        self.unit
            .expanded
            .iter()
            .map(|&association| PreparedOriginIntersectionView {
                prepared: &self.layout.inner.prepared,
                association,
            })
    }
    pub fn fragments(&self) -> impl ExactSizeIterator<Item = FinalizedFragment> + '_ {
        self.unit.fragments.iter().map(|&index| FinalizedFragment {
            finalized: Arc::clone(&self.layout.inner.lines[self.unit.line].finalized),
            index,
        })
    }
    /// The authentic inner Cell unit, when this is a Cell observation.
    pub fn cell_unit(&self) -> Option<SourceRangeCellUnit<'a>> {
        match self.unit.identity {
            UnitIdentity::Line(_) => None,
            UnitIdentity::Cell { cell, kind } => Some(SourceRangeCellUnit {
                layout: self.layout,
                unit: self.unit,
                cell,
                kind,
            }),
        }
    }
    /// Actual geometry retains its ordinary, inner-font or native-layout frame.
    pub fn pieces(&self) -> impl ExactSizeIterator<Item = SourceRangeGeometry<'a>> + 'a + use<'a> {
        let cell = self.cell_unit();
        self.unit.pieces.iter().map(move |piece| match piece {
            GeometryPiece::Line(piece) => SourceRangeGeometry::Line(SourceRangePiece { piece }),
            GeometryPiece::CellFont(piece) => {
                SourceRangeGeometry::CellFont(SourceRangeCellFontPiece {
                    unit: cell.expect("Cell font pieces retain their authentic unit"),
                    piece,
                })
            }
            GeometryPiece::CellTab(piece) => {
                SourceRangeGeometry::CellTab(SourceRangeCellTabPiece {
                    unit: cell.expect("Cell TAB pieces retain their authentic unit"),
                    piece,
                })
            }
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SourceRangePiece<'a> {
    piece: &'a Piece<'a>,
}
impl<'a> SourceRangePiece<'a> {
    pub fn rect(&self) -> Rect {
        self.piece.bounds
    }
    pub fn advance_edges(&self) -> (f64, f64) {
        (self.piece.start, self.piece.end)
    }
    pub fn font_ascent(&self) -> f64 {
        self.piece.ascent
    }
    pub fn font_descent(&self) -> f64 {
        self.piece.descent
    }
    pub fn baseline(&self) -> f64 {
        self.piece.baseline
    }
    pub fn effective_instance(&self) -> EffectiveFontInstance<'a> {
        self.piece.instance
    }
    /// Actual finalized cluster ordinals contributing these edges, scoped to the line.
    pub fn cluster_indices(&self) -> impl ExactSizeIterator<Item = usize> {
        self.piece.clusters.clone()
    }
}
fn overlap(a: Range, b: Range) -> Option<Range> {
    let start = a.start.max(b.start);
    let end = a.end.min(b.end);
    (start < end).then_some(Range::new(start, end))
}
fn add(field: &'static str, base: f64, delta: f64) -> Result<f64> {
    crate::engine::layout::add(field, TextNumericPhase::SourceRangeFragments, base, delta)
}
fn source_intersection(
    layout: &Layout,
    origin: usize,
    aggregate: Range,
    ledger: &mut Ledger,
) -> Result<OriginIntersection> {
    account::work(ledger, 1)?;
    let analyzed = &layout.inner.prepared.analyzed;
    let row = &analyzed.paragraph.origins[origin];
    if aggregate.start < row.aggregate.start || aggregate.end > row.aggregate.end {
        return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
            aggregate,
        }));
    }
    let local = Range::new(
        row.local.start + aggregate.start - row.aggregate.start,
        row.local.start + aggregate.end - row.aggregate.start,
    );
    let start =
        analyzed
            .processed
            .project_with(aggregate.start, crate::Affinity::Before, || {
                account::work(ledger, 1)
            })?;
    let end = analyzed
        .processed
        .project_with(aggregate.end, crate::Affinity::After, || {
            account::work(ledger, 1)
        })?;
    Ok(OriginIntersection {
        origin,
        aggregate,
        local,
        processed: Range::new(start, end),
    })
}
fn visit_intersections(
    layout: &Layout,
    range: Range,
    ledger: &mut Ledger,
    mut emit: impl FnMut(OriginIntersection, &mut Ledger) -> Result<()>,
) -> Result<()> {
    use crate::protocol::paragraph::OriginVisit;
    layout
        .inner
        .prepared
        .analyzed
        .paragraph
        .visit_origin_indices(range, |visit| match visit {
            OriginVisit::Inspect => account::work(ledger, 1),
            OriginVisit::Intersection { origin, aggregate } => {
                if !aggregate.is_empty() {
                    let piece = source_intersection(layout, origin, aggregate, ledger)?;
                    emit(piece, ledger)?;
                }
                Ok(())
            }
        })
}
fn append_intersections(
    layout: &Layout,
    range: Range,
    owner: Option<usize>,
    result: &mut Vec<OriginIntersection>,
    ledger: &mut Ledger,
) -> Result<()> {
    visit_intersections(layout, range, ledger, |piece, ledger| {
        if owner.is_none_or(|owner| piece.origin == owner) {
            account::push(ledger, result, piece)?;
        }
        Ok(())
    })
}
fn first_seen(rows: &mut Vec<(u8, usize)>, key: (u8, usize), ledger: &mut Ledger) -> Result<bool> {
    match operations::binary_search_by(rows, ledger.as_mut(), |row| row.cmp(&key))? {
        Ok(_) => Ok(false),
        Err(at) => {
            let moves = rows.len() - at;
            account::work(ledger, moves)?;
            account::push(ledger, rows, key)?;
            rows[at..].rotate_right(1);
            Ok(true)
        }
    }
}
fn dedup_sorted<T: PartialEq>(rows: &mut Vec<T>, ledger: &mut Ledger) -> Result<()> {
    account::work(ledger, rows.len())?;
    account::work(ledger, rows.len())?;
    rows.dedup();
    Ok(())
}
fn association_key(
    piece: &OriginIntersection,
) -> (usize, usize, usize, usize, usize, usize, usize) {
    (
        piece.origin,
        piece.aggregate.start,
        piece.aggregate.end,
        piece.local.start,
        piece.local.end,
        piece.processed.start,
        piece.processed.end,
    )
}
fn unique_associations(rows: &mut Vec<OriginIntersection>, ledger: &mut Ledger) -> Result<()> {
    operations::order_by(rows, ledger.as_mut(), |a, b| {
        association_key(a).cmp(&association_key(b))
    })?;
    account::work(ledger, rows.len())?;
    account::work(ledger, rows.len())?;
    rows.dedup_by_key(|row| association_key(row));
    Ok(())
}
// Geometry is operation-specific: EGC membership is retained by selected
// analysis; provider membership only lends the actual finalized advance.
struct MetricPiece {
    clusters: std::ops::Range<usize>,
    start: f64,
    end: f64,
}
impl Layout {
    /// Observe a nonempty original-local interval of this exact issued origin.
    /// This neither chooses a line nor resolves fonts. Partial grapheme units expand;
    /// removed custody and source-less occurrences do not acquire rectangles.
    /// Cell requests close over actual retained inner semantic EGCs, with
    /// distinct font-metric and native-layout geometry. Removed-only custody
    /// retains source metadata without lending a neighboring rectangle.
    pub fn source_range_fragments(
        &self,
        origin: &PreparedOrigin,
        local: Range,
    ) -> Result<SourceRangeFragments<'_>> {
        if !Arc::ptr_eq(&self.inner.prepared, &origin.prepared) {
            return Err(protocol_error(ProtocolError::ForeignPreparation {
                kind: ProtocolHandleKind::Origin,
                occurrence: origin.index,
            }));
        }
        let view = origin.view();
        crate::range::validate(view.source().text(), local)?;
        if local.is_empty() {
            return Err(failure(SourceRangeFragmentError::EmptyRange));
        }
        let captured = view.local_range();
        if local.start < captured.start || local.end > captured.end {
            return Err(failure(SourceRangeFragmentError::OutsideOrigin {
                origin: origin.index,
                local,
                captured,
            }));
        }
        let aggregate = Range::new(
            view.aggregate_range().start + (local.start - captured.start),
            view.aggregate_range().start + (local.end - captured.start),
        );
        let analyzed = &self.inner.prepared.analyzed;
        let origin_key = analyzed.paragraph.origin_map.flow_origin(
            analyzed
                .paragraph
                .origin_map
                .view(origin.index)
                .expect("admitted placed origin belongs to its actual view"),
        );
        analyzed
            .paragraph
            .admit(analyzed.processed.source(), aggregate)?;
        if aggregate.start < self.inner.coverage.source.start
            || aggregate.end > self.inner.coverage.source.end
        {
            return Err(failure(SourceRangeFragmentError::OutsidePlacement {
                aggregate,
            }));
        }
        // Only actual placed Cells participate. Original-local pieces retain
        // removed/zero custody; equal hulls, resources or glyph ranges do not.
        let mut active = false;
        'admission: for placed in &self.inner.lines {
            for (cell, _) in placed.finalized.data.cells.iter().enumerate() {
                if placed
                    .finalized
                    .source_cell(cell)
                    .intersects_original_range(origin_key, local)
                {
                    active = true;
                    break 'admission;
                }
            }
        }
        let mut ledger: Ledger = if active {
            Some(crate::engine::scope_alignment::census::begin_range(
                &self.inner,
            )?)
        } else {
            None
        };
        // Custody is checked independently of eligibility. Removed units retain
        // their original intervals, while alternatives/control anchors own none.
        for receipt in &self.inner.receipts {
            account::work(&mut ledger, 1)?;
            for unit in &receipt.selected.presentation.units {
                account::work(&mut ledger, 1)?;
                if matches!(unit.kind, PresentationUnitKind::Text)
                    && unit.atoms.is_empty()
                    && let Some(aggregate) = overlap(unit.source, aggregate)
                {
                    return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
                        aggregate,
                    }));
                }
            }
        }
        // One genuine input row can lend several expansion emissions, including
        // emissions on different lines. Deduplicate only that authenticated row,
        // not arbitrary coincident original intervals from different owners.
        let map = &analyzed.transform;
        let mut seen = Vec::new();
        let mut custody = Vec::new();
        for receipt in &self.inner.receipts {
            account::work(&mut ledger, 1)?;
            for event in &receipt.selected.presentation.selected_relation.events {
                account::work(&mut ledger, 1)?;
                let SelectedRelationEvent::Base { event } = *event else {
                    continue;
                };
                let row = match map.events()[event] {
                    crate::transform::relation::RelationEvent::Text { atom } => {
                        map.original_row_for_atom(atom)
                    }
                    crate::transform::relation::RelationEvent::Removal { row, .. } => Some(row),
                    _ => None,
                };
                if let Some(row) = row {
                    if first_seen(&mut seen, (0, row), &mut ledger)? {
                        for piece in map.original_pieces(row) {
                            account::work(&mut ledger, 1)?;
                            if piece.origin == origin_key
                                && let Some(range) = overlap(piece.aggregate, aggregate)
                            {
                                account::push(&mut ledger, &mut custody, range)?;
                            }
                        }
                    }
                } else if let crate::transform::relation::RelationEvent::PhaseIDeleted {
                    source,
                    ..
                } = map.events()[event]
                    && first_seen(&mut seen, (1, event), &mut ledger)?
                {
                    visit_intersections(self, source, &mut ledger, |piece, ledger| {
                        if piece.origin == origin.index
                            && let Some(range) = overlap(piece.aggregate, aggregate)
                        {
                            account::push(ledger, &mut custody, range)?;
                        }
                        Ok(())
                    })?;
                }
            }
        }
        operations::order_by(&mut custody, ledger.as_mut(), |a, b| {
            (a.start, a.end).cmp(&(b.start, b.end))
        })?;
        let mut at = aggregate.start;
        for range in &custody {
            account::work(&mut ledger, 1)?;
            if range.start > at {
                return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
                    aggregate: Range::new(at, range.start),
                }));
            }
            if range.start < at {
                return Err(failure(
                    SourceRangeFragmentError::AmbiguousOriginalMapping { aggregate: *range },
                ));
            }
            at = range.end;
        }
        if at != aggregate.end {
            return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
                aggregate: Range::new(at, aggregate.end),
            }));
        }
        account::drop_vec(&mut ledger, custody)?;
        account::drop_vec(&mut ledger, seen)?;
        let mut units = Vec::new();
        for (line_index, placed) in self.inner.lines.iter().enumerate() {
            account::work(&mut ledger, 1)?;
            let line = &placed.finalized;
            let presentation = &line.selected.presentation;
            let mut atom_units =
                account::filled(&mut ledger, presentation.projection.atoms.len(), 0usize)?;
            for (index, unit) in presentation.units.iter().enumerate() {
                account::work(&mut ledger, 1)?;
                for atom in unit.atoms.clone() {
                    account::work(&mut ledger, 1)?;
                    atom_units[atom] = index;
                }
            }
            let indexes = LineIndexes::new(line, &mut ledger)?;
            let mut providers = account::filled(
                &mut ledger,
                presentation.analysis.graphemes.len(),
                Vec::new(),
            )?;
            let mut index = 0;
            while index < line.data.clusters.len() {
                account::work(&mut ledger, 1)?;
                let first = index;
                let cluster = &line.data.clusters[first];
                index += 1;
                if cluster.frame != crate::engine::layout::data::FinalizedRunFrame::Line {
                    continue;
                }
                while index < line.data.clusters.len()
                    && line.data.clusters[index].provider_atoms == cluster.provider_atoms
                    && line.data.clusters[index].content == cluster.content
                    && line.data.clusters[index].frame == cluster.frame
                {
                    account::work(&mut ledger, 1)?;
                    index += 1;
                }
                let start = operations::partition_point(
                    &presentation.analysis.graphemes,
                    ledger.as_mut(),
                    |g| g.end <= cluster.provider_atoms.start,
                )?;
                let end = operations::partition_point(
                    &presentation.analysis.graphemes,
                    ledger.as_mut(),
                    |g| g.start < cluster.provider_atoms.end,
                )?;
                for rows in &mut providers[start..end] {
                    account::work(&mut ledger, 1)?;
                    account::push(&mut ledger, rows, first..index)?;
                }
            }
            let mut cached = GeometryCache::default();
            for (group, atoms) in presentation.analysis.graphemes.iter().enumerate() {
                account::work(&mut ledger, 1)?;
                let extent = Range::new(
                    presentation.projection.atoms[atoms.start]
                        .presentation
                        .start,
                    presentation.projection.atoms[atoms.end - 1]
                        .presentation
                        .end,
                );
                let begin =
                    operations::partition_point(&line.data.clusters, ledger.as_mut(), |c| {
                        c.presentation.end <= extent.start
                    })?;
                let end = operations::partition_point(&line.data.clusters, ledger.as_mut(), |c| {
                    c.presentation.start < extent.end
                })?;
                let own_clusters = &line.data.clusters[begin..end];
                let mut expanded = Vec::new();
                let mut selected = Vec::new();
                let mut group_units = account::clone_vec(&mut ledger, &atom_units[atoms.clone()])?;
                dedup_sorted(&mut group_units, &mut ledger)?;
                let mut cell_owned = false;
                for &unit in &group_units {
                    account::work(&mut ledger, 1)?;
                    cell_owned |= matches!(
                        presentation.units[unit].kind,
                        PresentationUnitKind::Cell { .. }
                    );
                }
                if cell_owned {
                    account::drop_vec(&mut ledger, group_units)?;
                    continue;
                }
                for &unit_index in &group_units {
                    account::work(&mut ledger, 1)?;
                    let unit = &presentation.units[unit_index];
                    if !matches!(unit.kind, PresentationUnitKind::Text) {
                        continue;
                    }
                    let mut has_geometry = false;
                    let mut retained = false;
                    for cluster in own_clusters {
                        account::work(&mut ledger, 1)?;
                        has_geometry |= overlap(cluster.presentation, unit.presentation).is_some();
                        retained |= overlap(cluster.presentation, extent).is_some()
                            && overlap(cluster.presentation, unit.presentation).is_some()
                            && cluster.disposition != SpacingSlotDisposition::DiscardedAtLineEdge;
                        if has_geometry && retained {
                            break;
                        }
                    }
                    if !has_geometry && overlap(unit.source, aggregate).is_some() {
                        return Err(failure(SourceRangeFragmentError::MissingUnitGeometry {
                            line: line_index,
                            unit: group,
                        }));
                    }
                    if !retained {
                        continue;
                    }
                    append_intersections(self, unit.source, None, &mut expanded, &mut ledger)?;
                    if let Some(range) = overlap(unit.source, aggregate) {
                        append_intersections(
                            self,
                            range,
                            Some(origin.index),
                            &mut selected,
                            &mut ledger,
                        )?;
                    }
                }
                let mut control_owners = Vec::new();
                let mut virtual_owners = Vec::new();
                let mut alternative_owners = Vec::new();
                for &unit in &group_units {
                    account::work(&mut ledger, 1)?;
                    match presentation.units[unit].kind {
                        PresentationUnitKind::GeneratedSeparator {
                            owner:
                                crate::transform::relation::GeneratedSeparatorOwner::Control(control),
                        } => account::push(&mut ledger, &mut control_owners, control)?,
                        PresentationUnitKind::GeneratedSeparator {
                            owner:
                                crate::transform::relation::GeneratedSeparatorOwner::Virtual(index),
                        } => account::push(&mut ledger, &mut virtual_owners, index)?,
                        PresentationUnitKind::Replacement { alternative, .. } => {
                            account::push(&mut ledger, &mut alternative_owners, alternative)?;
                        }
                        _ => (),
                    }
                }
                for rows in [
                    &mut control_owners,
                    &mut virtual_owners,
                    &mut alternative_owners,
                ] {
                    operations::order_by(rows, ledger.as_mut(), Ord::cmp)?;
                    dedup_sorted(rows, &mut ledger)?;
                }
                let mut defining = Vec::new();
                let mut attributed_cuts = Vec::new();
                for &index in &alternative_owners {
                    account::work(&mut ledger, 1)?;
                    match self.inner.prepared.tables.alternatives[index].definition {
                        AlternativeDefinition::Source {
                            origin: owner,
                            source,
                        } => {
                            let mut actual = None;
                            visit_intersections(self, source, &mut ledger, |piece, _| {
                                if actual.is_none() && piece.origin == owner {
                                    actual = Some(piece);
                                }
                                Ok(())
                            })?;
                            if let Some(piece) = actual {
                                account::push(&mut ledger, &mut defining, piece)?;
                            }
                        }
                        AlternativeDefinition::Automatic { cut, .. } => {
                            account::push(&mut ledger, &mut attributed_cuts, cut)?;
                        }
                        _ => (),
                    }
                }
                for definition in &defining {
                    account::work(&mut ledger, 1)?;
                    if definition.origin == origin.index
                        && let Some(source) = overlap(definition.aggregate, aggregate)
                    {
                        append_intersections(
                            self,
                            source,
                            Some(origin.index),
                            &mut selected,
                            &mut ledger,
                        )?;
                    }
                }
                for &cut in &attributed_cuts {
                    account::work(&mut ledger, 1)?;
                    let row = &self.inner.prepared.tables.hyphenation_cuts[cut];
                    let atoms = row.preceding.clone().ok_or_else(|| {
                        crate::hyphenation::error(crate::HyphenationError::InvalidResourceMapping)
                    })?;
                    for atom in atoms {
                        account::work(&mut ledger, 1)?;
                        let source = self.inner.prepared.analyzed.projection.atoms[atom].source;
                        if let Some(range) = overlap(source, aggregate) {
                            append_intersections(
                                self,
                                range,
                                Some(origin.index),
                                &mut selected,
                                &mut ledger,
                            )?;
                        }
                    }
                }
                unique_associations(&mut selected, &mut ledger)?;
                if selected.is_empty() {
                    account::drop_vec(&mut ledger, selected)?;
                    account::drop_vec(&mut ledger, expanded)?;
                    account::drop_vec(&mut ledger, group_units)?;
                    account::drop_vec(&mut ledger, control_owners)?;
                    account::drop_vec(&mut ledger, virtual_owners)?;
                    account::drop_vec(&mut ledger, alternative_owners)?;
                    account::drop_vec(&mut ledger, defining)?;
                    account::drop_vec(&mut ledger, attributed_cuts)?;
                    continue;
                }
                let geometry = range_geometry(
                    &line.data.clusters,
                    MetricUnits::Line(presentation),
                    group,
                    &providers[group],
                    &mut cached,
                    &mut ledger,
                    || {
                        failure(SourceRangeFragmentError::UnsupportedPartition {
                            line: line_index,
                            unit: group,
                        })
                    },
                )?;
                let mut pieces = Vec::new();
                for metric in &geometry {
                    account::work(&mut ledger, 1)?;
                    let cluster = &line.data.clusters[metric.clusters.start];
                    let font = cluster
                        .content
                        .font_run()
                        .and_then(|run| line.data.font_runs.get(run))
                        .ok_or_else(|| {
                            failure(SourceRangeFragmentError::MissingFontMetrics {
                                line: line_index,
                                unit: group,
                            })
                        })?;
                    account::work(&mut ledger, font.coords.len())?;
                    let instance = EffectiveFontInstance::qualify_synthesized(
                        font.font.face(),
                        font.font_size,
                        &font.coords,
                        font.synthesis,
                    )?;
                    let inline_anchor =
                        crate::layout::inline_coordinate(placed.orientation, placed.anchor);
                    let baseline = if let Some(reference) = cluster.reference {
                        crate::layout::cross_coordinate(
                            placed.orientation,
                            crate::layout::line_point(placed, crate::Point::new(0.0, reference)),
                        )
                    } else {
                        crate::layout::line_baseline(placed)
                    };
                    // Directed metric edges use the actual inline basis before
                    // checked translation; CCW advances decrease physical y.
                    let inline_delta = |inline| {
                        crate::layout::inline_coordinate(
                            placed.orientation,
                            crate::layout::line_vector(placed, crate::Point::new(inline, 0.0)),
                        )
                    };
                    let start = add(
                        "source range advance start",
                        inline_anchor,
                        inline_delta(metric.start),
                    )?;
                    let end = add(
                        "source range advance end",
                        inline_anchor,
                        inline_delta(metric.end),
                    )?;
                    let over = crate::layout::line_vector(
                        placed,
                        crate::Point::new(0.0, -font.metrics.ascent),
                    );
                    let under = crate::layout::line_vector(
                        placed,
                        crate::Point::new(0.0, font.metrics.descent),
                    );
                    let top = add(
                        "source range font top",
                        baseline,
                        crate::layout::cross_coordinate(placed.orientation, over),
                    )?;
                    let bottom = add(
                        "source range font bottom",
                        baseline,
                        crate::layout::cross_coordinate(placed.orientation, under),
                    )?;
                    let width = add("source range width", start.max(end), -start.min(end))?;
                    let height = add(
                        "source range height",
                        font.metrics.ascent,
                        font.metrics.descent,
                    )?;
                    let origin = crate::layout::physical_origin(
                        placed.orientation,
                        start.min(end),
                        top.min(bottom),
                    );
                    let size = crate::layout::physical_origin(placed.orientation, width, height);
                    let bounds = Rect::new(origin.x, origin.y, size.x, size.y);
                    add(
                        "source range rectangle right",
                        bounds.origin.x,
                        bounds.size.width,
                    )?;
                    add(
                        "source range rectangle bottom",
                        bounds.origin.y,
                        bounds.size.height,
                    )?;
                    if height < 0.0 {
                        return Err(crate::error::text_numeric_error(
                            "source range font extent",
                            TextNumericPhase::SourceRangeFragments,
                            crate::TextNumericError::OutOfRange,
                            height,
                        ));
                    }
                    account::push(
                        &mut ledger,
                        &mut pieces,
                        GeometryPiece::Line(Piece {
                            clusters: metric.clusters.clone(),
                            start,
                            end,
                            bounds,
                            instance,
                            ascent: font.metrics.ascent,
                            descent: font.metrics.descent,
                            baseline,
                        }),
                    )?;
                }
                if pieces.is_empty() {
                    return Err(failure(SourceRangeFragmentError::MissingFontMetrics {
                        line: line_index,
                        unit: group,
                    }));
                }
                let mut fragments = Vec::new();
                for piece in &expanded {
                    account::work(&mut ledger, 1)?;
                    indexes.append_original(
                        piece.origin,
                        piece.aggregate,
                        &mut fragments,
                        &mut ledger,
                    )?;
                }
                for &alternative in &alternative_owners {
                    account::work(&mut ledger, 1)?;
                    indexes.append_owner(
                        FragmentOwner::Alternative(alternative),
                        &mut fragments,
                        &mut ledger,
                    )?;
                }
                for &control in &control_owners {
                    account::work(&mut ledger, 1)?;
                    indexes.append_owner(
                        FragmentOwner::Control(control),
                        &mut fragments,
                        &mut ledger,
                    )?;
                }
                for &owner in &virtual_owners {
                    account::work(&mut ledger, 1)?;
                    indexes.append_owner(
                        FragmentOwner::VirtualSeparator(owner),
                        &mut fragments,
                        &mut ledger,
                    )?;
                }
                operations::order_by(&mut fragments, ledger.as_mut(), Ord::cmp)?;
                dedup_sorted(&mut fragments, &mut ledger)?;
                let controls = account::collect(
                    &mut ledger,
                    control_owners
                        .iter()
                        .map(|&occurrence| self.inner.prepared.tables.control_rows[occurrence]),
                )?;
                account::push(
                    &mut ledger,
                    &mut units,
                    Unit {
                        line: line_index,
                        identity: UnitIdentity::Line(group),
                        order: (group_units[0], group),
                        selected,
                        expanded,
                        fragments,
                        pieces,
                        controls,
                        virtuals: virtual_owners,
                        alternatives: alternative_owners,
                        defining,
                        attributed_cuts,
                        presentation: extent,
                    },
                )?;
                account::drop_vec(&mut ledger, geometry)?;
                account::drop_vec(&mut ledger, group_units)?;
                account::drop_vec(&mut ledger, control_owners)?;
            }
            account::drop_vec(&mut ledger, atom_units)?;
            for rows in providers.iter_mut() {
                account::drop_vec(&mut ledger, std::mem::take(rows))?;
            }
            account::drop_vec(&mut ledger, providers)?;
            cached.drop_scratch(&mut ledger)?;
            if active {
                cells::append(
                    self,
                    line_index,
                    cells::Query {
                        origin,
                        key: origin_key,
                        local,
                    },
                    &indexes,
                    &mut units,
                    &mut ledger,
                )?;
            }
            indexes.drop_scratch(&mut ledger)?;
        }
        operations::order_by(&mut units, ledger.as_mut(), |a, b| {
            (a.line, a.order).cmp(&(b.line, b.order))
        })?;
        Ok(SourceRangeFragments {
            layout: self,
            origin: origin.clone(),
            local,
            units,
            usage: ledger.as_ref().map(|budget| budget.usage()),
        })
    }
}
#[cfg(test)]
mod tests;
