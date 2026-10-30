//! Prepared authority for actual structural source sections.

use super::{
    PreparedBoundary, PreparedOpportunity, PreparedText, ProtocolError, ProtocolHandleKind,
    protocol_error,
};
use crate::{CapturedEventPosition, Result};
use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Clone, Debug)]
pub(crate) struct SegmentData {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) end_opportunity: usize,
    pub(crate) opportunities: std::ops::Range<usize>,
}

/// One actual prepared domain/ordinary-child-Block interval. Identity includes
/// the immutable preparation and section ordinal, including empty sections at
/// equal numeric cuts. A storage seam or forced break creates no such authority.
#[derive(Clone)]
pub struct PreparedSourceSegment {
    pub(crate) prepared: Arc<crate::engine::PreparedData>,
    pub(crate) index: usize,
}
impl PartialEq for PreparedSourceSegment {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Arc::ptr_eq(&self.prepared, &other.prepared)
    }
}
impl Eq for PreparedSourceSegment {}
impl Hash for PreparedSourceSegment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.prepared).hash(state);
        self.index.hash(state);
    }
}
impl fmt::Debug for PreparedSourceSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedSourceSegment")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl PreparedSourceSegment {
    /// Borrows the actual ordinary offers owned by this structural segment.
    pub fn opportunity_views(
        &self,
    ) -> impl ExactSizeIterator<Item = super::readback::PreparedOpportunityView<'_>> + '_ {
        self.prepared.tables.source_segments[self.index]
            .opportunities
            .clone()
            .map(move |index| super::readback::PreparedOpportunityView::new(&self.prepared, index))
    }
    #[must_use]
    pub fn prepared_text(&self) -> PreparedText {
        PreparedText {
            inner: Arc::clone(&self.prepared),
        }
    }
    #[must_use]
    pub fn start(&self) -> PreparedBoundary {
        self.boundary(self.prepared.tables.source_segments[self.index].start)
    }
    #[must_use]
    pub fn end(&self) -> PreparedBoundary {
        self.boundary(self.prepared.tables.source_segments[self.index].end)
    }
    #[must_use]
    pub fn entry_position(&self) -> Option<CapturedEventPosition> {
        self.position(self.prepared.analyzed.sections[self.index].entry)
    }
    #[must_use]
    pub fn exit_position(&self) -> Option<CapturedEventPosition> {
        self.position(self.prepared.analyzed.sections[self.index].exit)
    }
    #[must_use]
    pub fn end_opportunity(&self) -> PreparedOpportunity {
        PreparedOpportunity {
            prepared: Arc::clone(&self.prepared),
            index: self.prepared.tables.source_segments[self.index].end_opportunity,
        }
    }
    fn boundary(&self, index: usize) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(&self.prepared),
            index,
        }
    }
    fn position(&self, index: Option<usize>) -> Option<CapturedEventPosition> {
        Some(CapturedEventPosition {
            capture: Arc::clone(
                &self
                    .prepared
                    .analyzed
                    .paragraph
                    .scope_topology
                    .as_ref()?
                    .capture,
            ),
            index: index?,
        })
    }
}
impl PreparedText {
    #[must_use]
    pub fn source_segments(&self) -> impl ExactSizeIterator<Item = PreparedSourceSegment> + '_ {
        (0..self.inner.tables.source_segments.len()).map(|index| PreparedSourceSegment {
            prepared: Arc::clone(&self.inner),
            index,
        })
    }
}

pub(crate) fn admit<'a>(
    prepared: &PreparedText,
    segment: &'a PreparedSourceSegment,
) -> Result<&'a SegmentData> {
    super::selection::require_prepared(
        prepared,
        &segment.prepared,
        ProtocolHandleKind::SourceSegment,
        segment.index,
    )?;
    segment
        .prepared
        .tables
        .source_segments
        .get(segment.index)
        .ok_or_else(|| {
            protocol_error(ProtocolError::InvalidSourceSegment {
                segment: segment.index,
            })
        })
}

/// Called before any prepared handles exist. Exact local atom/event cuts are
/// supplied by the completed-analysis owner; unavailable cuts are never clamped.
pub(crate) fn prepare(
    tables: &mut super::prepared::PreparedTables,
    analyzed: &crate::engine::AnalyzedData,
) -> Result<()> {
    use super::prepared::{OpportunityData, PreparedBreakKind};
    let native_count = tables.boundaries.len();
    for (index, section) in analyzed.sections.iter().enumerate() {
        let (start, end) = if analyzed.paragraph.scope_topology.is_none() {
            // The uncaptured source has its genuine single native section.
            // Its incident affinities are independent correspondence facts;
            // whole-source coverage does not replace them with extent values.
            if analyzed.sections.len() != 1 {
                return Err(protocol_error(ProtocolError::UnsupportedStandaloneAnalysis));
            }
            let end = native_count.checked_sub(1).ok_or_else(|| {
                protocol_error(ProtocolError::InvalidSourceSegment { segment: index })
            })?;
            let first = &tables.boundaries[0];
            let last = &tables.boundaries[end];
            if first.participant != section.participants.start
                || first.atom != section.atoms.start
                || last.participant != section.participants.end
                || last.atom != section.atoms.end
            {
                return Err(protocol_error(ProtocolError::InvalidSourceSegment {
                    segment: index,
                }));
            }
            (0, end)
        } else {
            if section.entry.is_none() || section.exit.is_none() {
                return Err(protocol_error(ProtocolError::InvalidSourceSegment {
                    segment: index,
                }));
            }
            (
                frontier(
                    tables,
                    native_count,
                    index,
                    section.participants.start,
                    section.atoms.start,
                    section.events.start,
                    section.entry_cursor,
                )?,
                frontier(
                    tables,
                    native_count,
                    index,
                    section.participants.end,
                    section.atoms.end,
                    section.events.end,
                    section.exit_cursor,
                )?,
            )
        };
        let end_opportunity = tables.opportunities.len();
        tables.opportunities.push(OpportunityData {
            boundary: end,
            kind: PreparedBreakKind::ParagraphEnd,
            alternatives: Vec::new(),
            source_segment: Some(index),
        });
        tables.source_segments.push(SegmentData {
            start,
            end,
            end_opportunity,
            opportunities: 0..0,
        });
    }
    // Native opportunities were analyzed on native cuts only. Frontier aliases
    // cannot invent another soft/forced opportunity. All handles are still
    // private; put genuine cuts in order and remap every existing cut link.
    let mut boundaries = std::mem::take(&mut tables.boundaries)
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
    boundaries.sort_unstable_by_key(|(old, row)| (row.participant, row.relation_cut, *old));
    let mut boundary_map = vec![0; boundaries.len()];
    for (new, (old, _)) in boundaries.iter().enumerate() {
        boundary_map[*old] = new;
    }
    tables.boundaries = boundaries.into_iter().map(|(_, row)| row).collect();
    for row in &mut tables.opportunities {
        row.boundary = boundary_map[row.boundary];
    }
    for row in &mut tables.alternatives {
        row.boundary = boundary_map[row.boundary];
    }
    for row in &mut tables.hyphenation_cuts {
        row.boundary = boundary_map[row.boundary];
    }
    for row in &mut tables.suppressed {
        row.boundary = boundary_map[row.boundary];
    }
    for row in &mut tables.raw_conditional {
        *row = boundary_map[*row];
    }
    for row in &mut tables.source_segments {
        row.start = boundary_map[row.start];
        row.end = boundary_map[row.end];
    }
    let boundaries = &tables.boundaries;
    let source_segments = &tables.source_segments;
    let native_owner =
        |boundary: usize| preceding_section(analyzed, boundaries[boundary].participant);
    let native_endpoint = |boundary: usize| -> Result<usize> {
        let Some(owner) = native_owner(boundary) else {
            return Ok(boundary);
        };
        let section = &analyzed.sections[owner];
        let row = &boundaries[boundary];
        if row.relation_cut <= section.events.end {
            return Ok(boundary);
        }
        // A canonical participant cut can absorb later filtered removals.
        // The actual preceding participant still ends at its own proved edge.
        if row.participant == section.participants.end && row.atom == section.atoms.end {
            Ok(source_segments[owner].end)
        } else {
            Err(protocol_error(ProtocolError::InvalidSourceSegment {
                segment: owner,
            }))
        }
    };
    for row in &mut tables.opportunities {
        if row.kind == PreparedBreakKind::ParagraphEnd {
            continue;
        }
        row.source_segment = native_owner(row.boundary);
        row.boundary = native_endpoint(row.boundary)?;
    }
    for row in &mut tables.alternatives {
        row.boundary = native_endpoint(row.boundary)?;
    }
    for row in &mut tables.hyphenation_cuts {
        row.boundary = native_endpoint(row.boundary)?;
    }
    for row in &mut tables.suppressed {
        row.boundary = native_endpoint(row.boundary)?;
    }
    for row in &mut tables.raw_conditional {
        *row = native_endpoint(*row)?;
    }
    for (index, row) in tables.opportunities.iter().enumerate() {
        if row.source_segment.is_none() {
            return Err(protocol_error(ProtocolError::IllegalOpportunity {
                opportunity: index,
            }));
        }
    }
    // Real paragraph endpoints may share a cut, and forced endings stay before
    // those endpoints. Remap all internal opportunity links before publication.
    let mut rows = std::mem::take(&mut tables.opportunities)
        .into_iter()
        .enumerate()
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|(old, row)| {
        (
            row.source_segment,
            tables.boundaries[row.boundary].participant,
            row.boundary,
            match row.kind {
                PreparedBreakKind::Forced => 0,
                PreparedBreakKind::ParagraphEnd => 2,
                _ => 1,
            },
            *old,
        )
    });
    let mut mapping = vec![0; rows.len()];
    for (new, (old, _)) in rows.iter().enumerate() {
        mapping[*old] = new;
    }
    for row in &mut tables.alternatives {
        if let Some(index) = &mut row.opportunity {
            *index = mapping[*index];
        }
    }
    for row in &mut tables.hyphenation_cuts {
        if let Some(index) = &mut row.opportunity {
            *index = mapping[*index];
        }
    }
    for row in &mut tables.source_segments {
        row.end_opportunity = mapping[row.end_opportunity];
    }
    tables.opportunities = rows.into_iter().map(|(_, row)| row).collect();
    // One forward pass over the actual owner-sorted rows builds borrowed
    // inventories. A consumer never reconstructs ownership from equal cuts.
    let mut next = 0;
    for (owner, segment) in tables.source_segments.iter_mut().enumerate() {
        let start = next;
        while tables
            .opportunities
            .get(next)
            .is_some_and(|row| row.source_segment == Some(owner))
        {
            next += 1;
        }
        if start == next || !(start..next).contains(&segment.end_opportunity) {
            return Err(protocol_error(ProtocolError::IllegalOpportunity {
                opportunity: segment.end_opportunity,
            }));
        }
        segment.opportunities = start..next;
    }
    if next != tables.opportunities.len() {
        return Err(protocol_error(ProtocolError::IllegalOpportunity {
            opportunity: next,
        }));
    }
    Ok(())
}

fn frontier(
    tables: &mut super::prepared::PreparedTables,
    native_count: usize,
    segment: usize,
    participant: usize,
    atom: usize,
    cut: usize,
    cursor: crate::engine::analysis::structural::Cursor,
) -> Result<usize> {
    // Native typography-safe cuts are ordered but can omit participant prefixes
    // inside an indivisible unit. Aliases never supply that missing safety proof.
    let native =
        tables.boundaries[..native_count].partition_point(|row| row.participant < participant);
    let mut row = *tables
        .boundaries
        .get(native)
        .filter(|row| native < native_count && row.participant == participant && row.atom == atom)
        .ok_or_else(|| protocol_error(ProtocolError::InvalidSourceSegment { segment }))?;
    let matches = |row: &super::prepared::BoundaryData| {
        row.participant == participant
            && row.atom == atom
            && row.relation_cut == cut
            && row.processed == cursor.processed
            && row.source_before == cursor.source_before
            && row.source_after == cursor.source_after
    };
    if matches(&row) {
        return Ok(native);
    }
    // Adjacent empty structural sections can share their actual numeric cut.
    // Reuse that boundary, while their SegmentData rows remain distinct.
    if tables.boundaries.last().is_some_and(matches) {
        return Ok(tables.boundaries.len() - 1);
    }
    row.relation_cut = cut;
    row.processed = cursor.processed;
    row.source_before = cursor.source_before;
    row.source_after = cursor.source_after;
    let index = tables.boundaries.len();
    tables.boundaries.push(row);
    Ok(index)
}

pub(super) fn preceding_section(
    analyzed: &crate::engine::AnalyzedData,
    participant: usize,
) -> Option<usize> {
    let index = analyzed
        .sections
        .partition_point(|section| section.participants.end < participant);
    analyzed
        .sections
        .get(index)
        .filter(|section| {
            section.participants.start < participant && participant <= section.participants.end
        })
        .map(|_| index)
}

pub(crate) fn admit_boundaries(
    prepared: &PreparedText,
    segment: &PreparedSourceSegment,
    start: &PreparedBoundary,
    end: &PreparedBoundary,
) -> Result<()> {
    admit_boundary_range(prepared, segment, start, end, BoundaryUse::Range)
}

/// Observations authenticate each point's inward source context. They do not
/// consume an outward incident merely because their executed cuts coincide.
pub(crate) fn admit_points(
    prepared: &PreparedText,
    segment: &PreparedSourceSegment,
    start: &PreparedBoundary,
    end: &PreparedBoundary,
) -> Result<()> {
    admit_boundary_range(prepared, segment, start, end, BoundaryUse::Points)
}

#[derive(Clone, Copy)]
enum BoundaryUse {
    Range,
    Points,
}

fn admit_boundary_range(
    prepared: &PreparedText,
    segment: &PreparedSourceSegment,
    start: &PreparedBoundary,
    end: &PreparedBoundary,
    usage: BoundaryUse,
) -> Result<()> {
    let row = admit(prepared, segment)?;
    super::selection::require_prepared(
        prepared,
        &start.prepared,
        ProtocolHandleKind::Boundary,
        start.index,
    )?;
    super::selection::require_prepared(
        prepared,
        &end.prepared,
        ProtocolHandleKind::Boundary,
        end.index,
    )?;
    let invalid = || {
        protocol_error(ProtocolError::InvalidSourceSegment {
            segment: segment.index,
        })
    };
    let tables = &prepared.inner.tables;
    let first = tables.boundaries.get(start.index).ok_or_else(invalid)?;
    let last = tables.boundaries.get(end.index).ok_or_else(invalid)?;
    let section = &prepared.inner.analyzed.sections[segment.index];
    if first.participant > last.participant
        || first.participant < tables.boundaries[row.start].participant
        || last.participant > tables.boundaries[row.end].participant
        || first.atom < section.atoms.start
        || last.atom > section.atoms.end
        || first.atom > last.atom
        || first.relation_cut < section.events.start
        || last.relation_cut > section.events.end
        || first.relation_cut > last.relation_cut
    {
        return Err(invalid());
    }
    match (section.entry, section.exit) {
        (None, None) => {}
        (Some(_), Some(_)) => {
            // Capture admits aggregate origins in cursor order. These are the
            // genuine section frontiers, not an envelope of external pieces or
            // transformed emission order. Removed/expanded material can retain
            // either incident anywhere in this real inclusive source interval.
            let lower = section.entry_cursor.source_after;
            let upper = section.exit_cursor.source_before;
            if lower > upper {
                return Err(invalid());
            }
            let within = |position| lower <= position && position <= upper;
            let point = |boundary: &super::prepared::BoundaryData| {
                let entry = boundary.participant == section.participants.start
                    && boundary.atom == section.atoms.start
                    && boundary.relation_cut == section.events.start;
                let exit = boundary.participant == section.participants.end
                    && boundary.atom == section.atoms.end
                    && boundary.relation_cut == section.events.end;
                let after = within(boundary.source_after);
                let before = within(boundary.source_before);
                match (entry, exit) {
                    (true, true) => after || before,
                    (true, false) => after,
                    (false, true) => before,
                    (false, false) => after && before,
                }
            };
            let represented = match usage {
                BoundaryUse::Range => within(first.source_after) && within(last.source_before),
                BoundaryUse::Points => point(first) && point(last),
            };
            if !represented {
                return Err(invalid());
            }
            // Native processed positions follow the right/next atom, which can
            // be outside an ending slice. Do not turn that outward observation
            // into an extra source-consumption or full-cursor equality demand.
        }
        _ => return Err(invalid()),
    }
    Ok(())
}

pub(crate) fn admit_opportunity(
    prepared: &PreparedText,
    segment: &PreparedSourceSegment,
    opportunity: &PreparedOpportunity,
) -> Result<()> {
    admit(prepared, segment)?;
    super::selection::require_prepared(
        prepared,
        &opportunity.prepared,
        ProtocolHandleKind::Opportunity,
        opportunity.index,
    )?;
    let row = prepared
        .inner
        .tables
        .opportunities
        .get(opportunity.index)
        .ok_or_else(|| {
            protocol_error(ProtocolError::InvalidSourceSegment {
                segment: segment.index,
            })
        })?;
    match row.source_segment {
        Some(actual) if actual == segment.index => Ok(()),
        Some(actual) => Err(protocol_error(ProtocolError::MismatchedSourceSegment {
            expected: segment.index,
            actual,
        })),
        None => Err(protocol_error(ProtocolError::InvalidSourceSegment {
            segment: segment.index,
        })),
    }
}
