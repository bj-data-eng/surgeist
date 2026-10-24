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
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

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
}
fn failure(reason: SourceRangeFragmentError) -> Error {
    let code = match reason {
        SourceRangeFragmentError::MissingFontMetrics { .. }
        | SourceRangeFragmentError::UnsupportedPartition { .. } => ErrorCode::UnsupportedFeature,
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
}
#[derive(Debug)]
struct Unit<'a> {
    line: usize,
    group: usize,
    selected: Vec<OriginIntersection>,
    expanded: Vec<OriginIntersection>,
    fragments: Vec<usize>,
    pieces: Vec<Piece<'a>>,
    alternatives: Vec<usize>,
    controls: Vec<usize>,
    virtuals: Vec<usize>,
    defining: Vec<OriginIntersection>,
    attributed_cuts: Vec<usize>,
    presentation: Range,
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
            && self.unit.group == other.unit.group
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
    /// Real font pieces retain actual edges, or explicitly partitioned provider
    /// advance shares, without merging distinct effective fonts.
    pub fn pieces(&self) -> impl ExactSizeIterator<Item = SourceRangePiece<'a>> + 'a {
        self.unit
            .pieces
            .iter()
            .map(|piece| SourceRangePiece { piece })
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
fn intersections(layout: &Layout, range: Range) -> Vec<OriginIntersection> {
    layout
        .inner
        .prepared
        .analyzed
        .paragraph
        .origin_intersections(&layout.inner.prepared.analyzed.processed, range)
        .filter(|i| !i.aggregate.is_empty())
        .collect()
}
// Geometry is operation-specific: EGC membership is retained by selected
// analysis; provider membership only lends the actual finalized advance.
struct MetricPiece {
    clusters: std::ops::Range<usize>,
    start: f64,
    end: f64,
}
fn range_geometry(
    line: &crate::engine::FinalizedLine,
    line_index: usize,
    group: usize,
    providers: &[std::ops::Range<usize>],
    cached: &mut BTreeMap<usize, Vec<(f64, f64)>>,
) -> Result<Vec<MetricPiece>> {
    let graphemes = &line.selected.presentation.analysis.graphemes;
    let clusters = &line.data.clusters;
    let unsupported = || {
        failure(SourceRangeFragmentError::UnsupportedPartition {
            line: line_index,
            unit: group,
        })
    };
    let mut result = Vec::new();
    for indices in providers {
        let first = indices.start;
        let index = indices.end;
        let provider = &clusters[first].provider_atoms;
        if let Some(edges) = cached.get(&first) {
            let begin = graphemes.partition_point(|g| g.end <= provider.start);
            let (start, end) = edges[group - begin];
            result.push(MetricPiece {
                clusters: indices.clone(),
                start,
                end,
            });
            continue;
        }
        let begin = graphemes.partition_point(|g| g.end <= provider.start);
        let end = graphemes.partition_point(|g| g.start < provider.end);
        if end <= begin {
            return Err(unsupported());
        }
        if end - begin == 1 {
            for (offset, cluster) in clusters[first..index].iter().enumerate() {
                if cluster.disposition != SpacingSlotDisposition::DiscardedAtLineEdge {
                    result.push(MetricPiece {
                        clusters: first + offset..first + offset + 1,
                        start: cluster.x,
                        end: add("source range local advance end", cluster.x, cluster.advance)?,
                    });
                }
            }
            continue;
        }
        // Equal shares are Surgeist's explicit metric policy for an ordinary
        // monotone horizontal provider group spanning complete EGCs. They are
        // neither independent letter measurements nor font ligature carets.
        if graphemes[begin].start != provider.start
            || graphemes[end - 1].end != provider.end
            || clusters[first..index].iter().any(|c| {
                c.level != clusters[first].level
                    || c.advance < 0.0
                    || c.disposition == SpacingSlotDisposition::DiscardedAtLineEdge
            })
        {
            return Err(unsupported());
        }
        let rtl = !clusters[first].level.is_multiple_of(2);
        let mut previous = None;
        for cluster in &clusters[first..index] {
            let edge = add("source range provider edge", cluster.x, cluster.advance)?;
            if previous.is_some_and(|(x, right)| if rtl { edge > x } else { cluster.x < right }) {
                return Err(unsupported());
            }
            previous = Some((cluster.x, edge));
        }
        let count = end - begin;
        let ordinal = group - begin;
        let raw = clusters[first..index]
            .iter()
            .try_fold(0.0, |base, cluster| {
                add("source range shaped advance", base, cluster.shaped_advance)
            })?;
        let share = raw / count as f64;
        let mut adjustments = vec![0.0; count];
        for cluster in &clusters[first..index] {
            let egc = graphemes.partition_point(|g| {
                line.selected.presentation.projection.atoms[g.end - 1]
                    .presentation
                    .end
                    <= cluster.presentation.start
            });
            if egc < begin || egc >= end {
                return Err(unsupported());
            }
            for value in cluster.logical_adjustments {
                adjustments[egc - begin] = add(
                    "source range owned adjustment",
                    adjustments[egc - begin],
                    value,
                )?;
            }
        }
        let right = add(
            "source range provider right",
            clusters[first].x,
            clusters[first].advance,
        )?;
        let mut offset = 0.0;
        let mut edges = Vec::with_capacity(count);
        for (relative, adjustment) in adjustments.into_iter().enumerate() {
            let raw_part = if relative + 1 == count {
                raw - share * relative as f64
            } else {
                share
            };
            let width = add("source range adjusted share", raw_part, adjustment)?;
            if width < 0.0 {
                return Err(unsupported());
            }
            let pair = if rtl {
                let end = add("source range logical share end", right, -offset)?;
                (add("source range logical share start", end, -width)?, end)
            } else {
                let start = add(
                    "source range logical share start",
                    clusters[first].x,
                    offset,
                )?;
                (start, add("source range logical share end", start, width)?)
            };
            edges.push(pair);
            offset = add("source range preceding shares", offset, width)?;
        }
        let (start, end) = edges[ordinal];
        cached.insert(first, edges);
        result.push(MetricPiece {
            clusters: first..index,
            start,
            end,
        });
    }
    Ok(result)
}
impl Layout {
    /// Observe a nonempty original-local interval of this exact issued origin.
    /// This neither chooses a line nor resolves fonts. Partial grapheme units expand;
    /// removed custody and source-less occurrences do not acquire rectangles.
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
        // Custody is checked independently of eligibility. Removed units retain
        // their original intervals, while alternatives/control anchors own none.
        for placed in &self.inner.lines {
            for unit in &placed.finalized.selected.presentation.units {
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
        let mut seen = BTreeSet::new();
        let mut custody = Vec::new();
        for line in &self.inner.lines {
            for event in &line
                .finalized
                .selected
                .presentation
                .selected_relation
                .events
            {
                let SelectedRelationEvent::Base { event } = *event else {
                    continue;
                };
                let row = match map.events()[event] {
                    crate::transform::relation::RelationEvent::Text { atom } => {
                        map.original_row_for_atom(atom)
                    }
                    crate::transform::relation::RelationEvent::Removal { row } => Some(row),
                    _ => None,
                };
                if let Some(row) = row {
                    if seen.insert((0, row)) {
                        custody.extend(
                            map.original_pieces(row)
                                .iter()
                                .filter(|piece| piece.origin == origin.index)
                                .filter_map(|piece| overlap(piece.aggregate, aggregate)),
                        );
                    }
                } else if let crate::transform::relation::RelationEvent::PhaseIDeleted {
                    source,
                    ..
                } = map.events()[event]
                    && seen.insert((1, event))
                {
                    custody.extend(
                        analyzed
                            .paragraph
                            .origin_intersections(&analyzed.processed, source)
                            .filter(|piece| piece.origin == origin.index)
                            .filter_map(|piece| overlap(piece.aggregate, aggregate)),
                    );
                }
            }
        }
        custody.sort_unstable_by_key(|range| (range.start, range.end));
        let mut at = aggregate.start;
        for range in custody {
            if range.start > at {
                return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
                    aggregate: Range::new(at, range.start),
                }));
            }
            if range.start < at {
                return Err(failure(
                    SourceRangeFragmentError::AmbiguousOriginalMapping { aggregate: range },
                ));
            }
            at = range.end;
        }
        if at != aggregate.end {
            return Err(failure(SourceRangeFragmentError::MissingOriginalMapping {
                aggregate: Range::new(at, aggregate.end),
            }));
        }
        let mut units = Vec::new();
        for (line_index, placed) in self.inner.lines.iter().enumerate() {
            let line = &placed.finalized;
            let presentation = &line.selected.presentation;
            let mut atom_units = vec![0; presentation.projection.atoms.len()];
            for (index, unit) in presentation.units.iter().enumerate() {
                for atom in unit.atoms.clone() {
                    atom_units[atom] = index;
                }
            }
            let mut origin_fragments: BTreeMap<usize, Vec<(Range, usize)>> = BTreeMap::new();
            let mut alternatives: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            let mut control_fragments: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            let mut virtual_fragments: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            for (index, fragment) in line.data.fragments.iter().enumerate() {
                match fragment.association.owner {
                    FragmentOwner::Origin(origin) => {
                        if let Some(source) = fragment.association.source
                            && !source.aggregate.is_empty()
                        {
                            origin_fragments
                                .entry(origin)
                                .or_default()
                                .push((source.aggregate, index));
                        }
                    }
                    FragmentOwner::Alternative(alternative) => {
                        alternatives.entry(alternative).or_default().push(index)
                    }
                    FragmentOwner::Control(control) => {
                        control_fragments.entry(control).or_default().push(index);
                    }
                    FragmentOwner::VirtualSeparator(owner) => {
                        virtual_fragments.entry(owner).or_default().push(index);
                    }
                }
            }
            for rows in origin_fragments.values_mut() {
                rows.sort_unstable_by_key(|row| (row.0.start, row.0.end));
            }
            let mut providers = vec![Vec::new(); presentation.analysis.graphemes.len()];
            let mut index = 0;
            while index < line.data.clusters.len() {
                let first = index;
                let cluster = &line.data.clusters[first];
                index += 1;
                while index < line.data.clusters.len()
                    && line.data.clusters[index].provider_atoms == cluster.provider_atoms
                    && line.data.clusters[index].content == cluster.content
                {
                    index += 1;
                }
                let start = presentation
                    .analysis
                    .graphemes
                    .partition_point(|g| g.end <= cluster.provider_atoms.start);
                let end = presentation
                    .analysis
                    .graphemes
                    .partition_point(|g| g.start < cluster.provider_atoms.end);
                for rows in &mut providers[start..end] {
                    rows.push(first..index);
                }
            }
            let mut cached = BTreeMap::new();
            for (group, atoms) in presentation.analysis.graphemes.iter().enumerate() {
                let extent = Range::new(
                    presentation.projection.atoms[atoms.start]
                        .presentation
                        .start,
                    presentation.projection.atoms[atoms.end - 1]
                        .presentation
                        .end,
                );
                let begin = line
                    .data
                    .clusters
                    .partition_point(|c| c.presentation.end <= extent.start);
                let end = line
                    .data
                    .clusters
                    .partition_point(|c| c.presentation.start < extent.end);
                let own_clusters = &line.data.clusters[begin..end];
                let mut expanded = Vec::new();
                let mut selected = Vec::new();
                let mut group_units: Vec<_> = atom_units[atoms.clone()].to_vec();
                group_units.dedup();
                for &unit_index in &group_units {
                    let unit = &presentation.units[unit_index];
                    if !matches!(unit.kind, PresentationUnitKind::Text) {
                        continue;
                    }
                    let has_geometry = own_clusters
                        .iter()
                        .any(|cluster| overlap(cluster.presentation, unit.presentation).is_some());
                    let retained = own_clusters.iter().any(|cluster| {
                        overlap(cluster.presentation, extent).is_some()
                            && overlap(cluster.presentation, unit.presentation).is_some()
                            && cluster.disposition != SpacingSlotDisposition::DiscardedAtLineEdge
                    });
                    if !has_geometry && overlap(unit.source, aggregate).is_some() {
                        return Err(failure(SourceRangeFragmentError::MissingUnitGeometry {
                            line: line_index,
                            unit: group,
                        }));
                    }
                    if !retained {
                        continue;
                    }
                    expanded.extend(intersections(self, unit.source));
                    if let Some(range) = overlap(unit.source, aggregate) {
                        selected.extend(
                            intersections(self, range)
                                .into_iter()
                                .filter(|i| i.origin == origin.index),
                        );
                    }
                }
                let mut control_owners = Vec::new();
                let mut virtual_owners = Vec::new();
                for &unit in &group_units {
                    if let PresentationUnitKind::GeneratedSeparator {
                        owner: crate::transform::relation::GeneratedSeparatorOwner::Control(control),
                    } = presentation.units[unit].kind
                        && !control_owners.contains(&control)
                    {
                        control_owners.push(control);
                    }
                    if let PresentationUnitKind::GeneratedSeparator {
                        owner: crate::transform::relation::GeneratedSeparatorOwner::Virtual(index),
                    } = presentation.units[unit].kind
                        && !virtual_owners.contains(&index)
                    {
                        virtual_owners.push(index);
                    }
                }
                let mut alternative_owners = group_units
                    .iter()
                    .filter_map(|&i| match presentation.units[i].kind {
                        PresentationUnitKind::Replacement { alternative } => Some(alternative),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                alternative_owners.sort_unstable();
                alternative_owners.dedup();
                let defining = alternative_owners
                    .iter()
                    .filter_map(|&index| {
                        match self.inner.prepared.tables.alternatives[index].definition {
                            AlternativeDefinition::Source {
                                origin: owner,
                                source,
                            } => intersections(self, source)
                                .into_iter()
                                .find(|piece| piece.origin == owner),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();
                for definition in &defining {
                    if definition.origin == origin.index
                        && let Some(source) = overlap(definition.aggregate, aggregate)
                    {
                        selected.extend(
                            intersections(self, source)
                                .into_iter()
                                .filter(|piece| piece.origin == origin.index),
                        );
                    }
                }
                let attributed_cuts = alternative_owners
                    .iter()
                    .filter_map(|&index| {
                        match self.inner.prepared.tables.alternatives[index].definition {
                            AlternativeDefinition::Automatic { cut, .. } => Some(cut),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();
                for &cut in &attributed_cuts {
                    let row = &self.inner.prepared.tables.hyphenation_cuts[cut];
                    let atoms = row.preceding.clone().ok_or_else(|| {
                        crate::hyphenation::error(crate::HyphenationError::InvalidResourceMapping)
                    })?;
                    for atom in atoms {
                        let source = self.inner.prepared.analyzed.projection.atoms[atom].source;
                        if let Some(range) = overlap(source, aggregate) {
                            selected.extend(
                                intersections(self, range)
                                    .into_iter()
                                    .filter(|piece| piece.origin == origin.index),
                            );
                        }
                    }
                }
                // One authentic source association may be reached both through
                // the selected original row and automatic attribution in a shared
                // EGC. Deduplicate that SAME occurrence/correspondence, never
                // Range units, glyphs or geometry at equal numeric positions.
                let mut associations = std::collections::HashSet::new();
                selected.retain(|piece| {
                    associations.insert((
                        piece.origin,
                        piece.aggregate,
                        piece.local,
                        piece.processed,
                    ))
                });
                if selected.is_empty() {
                    continue;
                }
                let geometry =
                    range_geometry(line, line_index, group, &providers[group], &mut cached)?;
                let mut pieces = Vec::new();
                for metric in &geometry {
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
                    let instance = EffectiveFontInstance::qualify_synthesized(
                        font.font.face(),
                        font.font_size,
                        &font.coords,
                        font.synthesis,
                    )?;
                    let start = add("source range advance start", placed.anchor.x, metric.start)?;
                    let local_end = metric.end;
                    let end = add("source range advance end", placed.anchor.x, local_end)?;
                    let top = add(
                        "source range font top",
                        placed.anchor.y,
                        -font.metrics.ascent,
                    )?;
                    let bottom = add(
                        "source range font bottom",
                        placed.anchor.y,
                        font.metrics.descent,
                    )?;
                    let width = add("source range width", start.max(end), -start.min(end))?;
                    let height = add(
                        "source range height",
                        font.metrics.ascent,
                        font.metrics.descent,
                    )?;
                    let bounds = Rect::new(start.min(end), top, width, height);
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
                    if bottom < top {
                        return Err(crate::error::text_numeric_error(
                            "source range font extent",
                            TextNumericPhase::SourceRangeFragments,
                            crate::TextNumericError::OutOfRange,
                            height,
                        ));
                    }
                    pieces.push(Piece {
                        clusters: metric.clusters.clone(),
                        start,
                        end,
                        bounds,
                        instance,
                        ascent: font.metrics.ascent,
                        descent: font.metrics.descent,
                        baseline: placed.anchor.y,
                    });
                }
                if pieces.is_empty() {
                    return Err(failure(SourceRangeFragmentError::MissingFontMetrics {
                        line: line_index,
                        unit: group,
                    }));
                }
                let mut fragments = Vec::new();
                for piece in &expanded {
                    if let Some(rows) = origin_fragments.get(&piece.origin) {
                        let first = rows.partition_point(|row| row.0.end <= piece.aggregate.start);
                        fragments.extend(
                            rows[first..]
                                .iter()
                                .take_while(|row| row.0.start < piece.aggregate.end)
                                .map(|row| row.1),
                        );
                    }
                }
                for &index in &group_units {
                    if let PresentationUnitKind::Replacement { alternative } =
                        presentation.units[index].kind
                        && let Some(rows) = alternatives.get(&alternative)
                    {
                        fragments.extend(rows);
                    }
                }
                for control in &control_owners {
                    if let Some(rows) = control_fragments.get(control) {
                        fragments.extend(rows);
                    }
                }
                for owner in &virtual_owners {
                    if let Some(rows) = virtual_fragments.get(owner) {
                        fragments.extend(rows);
                    }
                }
                fragments.sort_unstable();
                fragments.dedup();
                units.push(Unit {
                    line: line_index,
                    group,
                    selected,
                    expanded,
                    fragments,
                    pieces,
                    controls: control_owners
                        .into_iter()
                        .map(|occurrence| self.inner.prepared.tables.control_rows[occurrence])
                        .collect(),
                    virtuals: virtual_owners,
                    alternatives: alternative_owners,
                    defining,
                    attributed_cuts,
                    presentation: extent,
                });
            }
        }
        Ok(SourceRangeFragments {
            layout: self,
            origin: origin.clone(),
            local,
            units,
        })
    }
}
#[cfg(test)]
mod tests;
