//! Canonical preparation tables; handles borrow these actual issued rows.

#[cfg(test)]
mod tests;

use std::ops::Range as IndexRange;

use crate::engine::{
    AnalyzedData,
    analysis::{Analysis, Boundary, anchor_atom, anchor_position, is_hard_separator},
    layout::data::{ShapedData, ShapedItem},
    resolve::ResolvedStyleRun,
};
use crate::{Range, projection::SourceProjection};

use super::paragraph::{ParagraphAnchor, ParagraphAnchorItem, ParagraphControlKind};

/// Immutable logical participants and their exact occurrence/boundary tables.
#[derive(Debug, Default)]
pub(crate) struct PreparedTables {
    pub(crate) participants: Vec<ParticipantData>,
    pub(crate) units: Vec<UnitData>,
    pub(crate) atomics: Vec<AtomicData>,
    pub(crate) controls: Vec<ControlData>,
    /// Exact captured control occurrence to its actual issued table row.
    pub(crate) control_rows: Vec<usize>,
    pub(crate) boundaries: Vec<BoundaryData>,
    pub(crate) opportunities: Vec<OpportunityData>,
    pub(crate) alternatives: Vec<AlternativeData>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParticipantData {
    Text(usize),
    Atomic(usize),
    Control(usize),
}

/// One original-owned logical unit; only issued boundaries permit selection.
#[derive(Debug)]
pub(crate) struct UnitData {
    /// Distinct zero-extent original occurrence; no synthetic glyph is implied.
    pub(crate) empty_origin: Option<usize>,
    /// Actual lexical deletion ownership, with no presentation glyph implied.
    pub(crate) deleted: bool,
    pub(crate) source: Range,
    pub(crate) processed: Range,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) participant: usize,
    pub(crate) advance: f64,
    pub(crate) ascent: f64,
    pub(crate) descent: f64,
    pub(crate) line_height: f64,
    pub(crate) level: u8,
    pub(crate) is_whitespace: bool,
}

/// Refers to the original box-vector occurrence; no copied geometry is authority.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AtomicData {
    pub(crate) occurrence: usize,
    pub(crate) participant: usize,
    pub(crate) source: usize,
    pub(crate) processed: usize,
    pub(crate) level: u8,
    pub(crate) affinity: crate::Affinity,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ControlData {
    pub(crate) occurrence: usize,
    pub(crate) participant: usize,
    pub(crate) source: usize,
    pub(crate) processed: usize,
    pub(crate) kind: ParagraphControlKind,
    pub(crate) level: u8,
}

/// A mixed-sequence boundary retains both original incident extrema.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BoundaryData {
    pub(crate) participant: usize,
    pub(crate) atom: usize,
    pub(crate) processed: usize,
    pub(crate) source_before: usize,
    pub(crate) source_after: usize,
    /// Exact old chooser endpoint, only when a real preliminary item boundary.
    pub(crate) preliminary_item: Option<usize>,
}

/// The actual terminating behavior issued by opportunity analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedBreakKind {
    Allowed,
    Discretionary,
    Forced,
    ParagraphEnd,
}

#[derive(Debug)]
pub(crate) struct OpportunityData {
    pub(crate) boundary: usize,
    pub(crate) kind: PreparedBreakKind,
    pub(crate) alternatives: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AlternativeData {
    pub(crate) occurrence: usize,
    pub(crate) opportunity: usize,
    pub(crate) definition: super::paragraph::ParagraphReplacementDefinition,
    pub(crate) insertion_anchor: usize,
}

/// Actual replacement-context preparation under the same catalogue capture.
///
/// This is private variant work in the same preparation, not a separately
/// issued public source/preparation. Every retained face/instance comes from
/// actual resolution before publication. Finalization never queries a newer
/// catalogue to reconstruct it.
pub(crate) struct PreparedAlternativeShape {
    pub(crate) alternative: usize,
    pub(crate) inserted_atoms: IndexRange<usize>,
    pub(crate) projection: SourceProjection,
    pub(crate) analysis: Analysis,
    pub(crate) shaped: ShapedData,
    /// Exact splice correspondence; inserted presentation has no base atom.
    pub(crate) original_atoms: Vec<Option<usize>>,
    pub(crate) base_insertion: usize,
    /// One retained original joining domain per actual shaped run.
    pub(crate) joining_domains: Vec<usize>,
}

/// Exact selected presentation correspondence, separate from authored text.
#[derive(Debug)]
pub(crate) struct SelectedMapping {
    pub(crate) pieces: Vec<SelectedMappingPiece>,
}

/// One validated selection's actual provider-independent presentation. It is
/// built once, then shared by spacing admission and retained-face shaping.
pub(crate) struct SelectedPresentation {
    pub(crate) source: Range,
    pub(crate) units: Vec<PresentationUnit>,
    pub(crate) projection: SourceProjection,
    pub(crate) analysis: Analysis,
    pub(crate) assignments: Vec<RetainedAssignment>,
    pub(crate) mapping: std::sync::Arc<SelectedMapping>,
}

#[derive(Clone, Debug)]
pub(crate) struct PresentationUnit {
    pub(crate) source: Range,
    pub(crate) processed: Range,
    pub(crate) presentation: Range,
    pub(crate) atoms: IndexRange<usize>,
    /// Selected-local ordinal in the exact request's participant vector.
    pub(crate) participant: Option<usize>,
    /// Some only when this semantic unit consists of exactly one code point.
    pub(crate) code_point: Option<u32>,
    pub(crate) bidi_class: Option<icu_properties::props::BidiClass>,
    pub(crate) eligible_space: bool,
    pub(crate) level: u8,
    pub(crate) kind: PresentationUnitKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresentationUnitKind {
    Text,
    Deleted,
    EmptyOrigin { origin: usize },
    Atomic { occurrence: usize },
    Control { occurrence: usize },
    Replacement { alternative: usize },
}

/// Exact strong-font row already resolved in the original preparation or one
/// captured alternative. Atom extents refer to this selected projection.
#[derive(Clone, Debug)]
pub(crate) struct RetainedAssignment {
    pub(crate) atoms: IndexRange<usize>,
    /// Exact atom interval in the retained base or alternative capture.
    pub(crate) captured_atoms: IndexRange<usize>,
    pub(crate) alternative: Option<usize>,
    pub(crate) run: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectedMappingPiece {
    pub(crate) source: Range,
    pub(crate) processed: Range,
    pub(crate) presentation: Range,
    pub(crate) kind: SelectedMappingKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectedMappingKind {
    Source,
    Deleted,
    Replacement { alternative: usize, anchor: usize },
}

/// Issues canonical units from actual atoms, executed deletions and captured
/// anchor occurrences. No preliminary item/source offset issues a public handle.
pub(crate) fn prepare_tables(
    analyzed: &AnalyzedData,
    shaped: &ShapedData,
) -> crate::Result<PreparedTables> {
    use crate::engine::analysis::wrapping::Event;
    let projection = &analyzed.projection;
    let source = analyzed.processed.source();
    let paragraph = &analyzed.paragraph;
    let mut item_for_atom = vec![None; projection.atoms.len()];
    let mut item_for_box = vec![None; source.boxes().len()];
    let mut cluster_for_atom = vec![None; projection.atoms.len()];
    for (item, value) in shaped.items.iter().enumerate() {
        match *value {
            ShapedItem::TextCluster(index) => {
                for atom in shaped.clusters[index].atoms.clone() {
                    item_for_atom[atom] = Some(item);
                    cluster_for_atom[atom] = Some(index);
                }
            }
            ShapedItem::InlineBox { occurrence, .. } => item_for_box[occurrence] = Some(item),
            ShapedItem::Anchor { .. } => {}
        }
    }
    let mut tables = PreparedTables::default();
    let mut seen_items = vec![false; shaped.items.len()];
    let mut item_prefix = 0;
    let mut item_count = 0;
    let mut owned_source = paragraph.extent(&analyzed.processed).start;
    let mut atom_cursor = 0;
    tables.boundaries.push(BoundaryData {
        participant: 0,
        atom: 0,
        processed: analyzed
            .processed
            .project(owned_source, crate::Affinity::Before),
        source_before: owned_source,
        source_after: owned_source,
        preliminary_item: Some(0),
    });
    let mut events = analyzed.logical.events.iter().copied().peekable();
    while let Some(event) = events.next() {
        let atom = event.atom;
        let source_at = event.source;
        let event = event.kind;
        owned_source = owned_source.max(source_at);
        let participant = tables.participants.len();
        let processed_at = projection.atoms.get(atom).map_or(
            analyzed.processed.project(
                paragraph.extent(&analyzed.processed).end,
                crate::Affinity::After,
            ),
            |value| value.processed.start,
        );
        let mut preliminary = None;
        match event {
            Event::Text(index) => {
                let value = &projection.atoms[index];
                let cluster = cluster_for_atom[index].map(|index| &shaped.clusters[index]);
                let metrics = cluster
                    .map(|cluster| shaped.runs[cluster.run].metrics)
                    .unwrap_or_default();
                let item = item_for_atom[index];
                let row = UnitData {
                    empty_origin: None,
                    deleted: false,
                    source: value.source,
                    processed: value.processed,
                    atoms: index..index + 1,
                    participant,
                    advance: cluster.map_or(0.0, |cluster| cluster.advance),
                    ascent: metrics.ascent,
                    descent: metrics.descent,
                    line_height: metrics.line_height,
                    level: analyzed.analysis.levels[index],
                    is_whitespace: cluster.is_some_and(|cluster| cluster.is_whitespace),
                };
                let unit = tables.units.len();
                tables.units.push(row);
                tables.participants.push(ParticipantData::Text(unit));
                owned_source = owned_source.max(value.source.end);
                atom_cursor = index + 1;
                preliminary = item;
            }
            Event::Deleted(range, at) => {
                let unit = tables.units.len();
                tables.units.push(UnitData {
                    empty_origin: None,
                    deleted: true,
                    source: range,
                    processed: Range::new(at, at),
                    atoms: atom..atom,
                    participant,
                    advance: 0.0,
                    ascent: 0.0,
                    descent: 0.0,
                    line_height: 0.0,
                    level: analyzed
                        .analysis
                        .levels
                        .get(atom)
                        .copied()
                        .unwrap_or(analyzed.analysis.base_level),
                    is_whitespace: false,
                });
                tables.participants.push(ParticipantData::Text(unit));
                owned_source = owned_source.max(range.end);
            }
            Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(origin),
                ..
            }) => {
                let unit = tables.units.len();
                let range = paragraph.origins[origin].aggregate;
                tables.units.push(UnitData {
                    empty_origin: Some(origin),
                    deleted: false,
                    source: range,
                    processed: Range::new(processed_at, processed_at),
                    atoms: atom..atom,
                    participant,
                    advance: 0.0,
                    ascent: 0.0,
                    descent: 0.0,
                    line_height: 0.0,
                    level: analyzed
                        .analysis
                        .levels
                        .get(atom)
                        .copied()
                        .unwrap_or(analyzed.analysis.base_level),
                    is_whitespace: false,
                });
                tables.participants.push(ParticipantData::Text(unit));
            }
            Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(occurrence),
                affinity,
            }) => {
                let row = tables.atomics.len();
                tables.atomics.push(AtomicData {
                    occurrence,
                    participant,
                    source: source.boxes()[occurrence].index(),
                    processed: processed_at,
                    level: analyzed.analysis.box_levels[occurrence],
                    affinity,
                });
                tables.participants.push(ParticipantData::Atomic(row));
                preliminary = item_for_box[occurrence];
            }
            Event::Anchor(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(occurrence),
                ..
            }) => {
                let row = tables.controls.len();
                let control = paragraph.controls[occurrence];
                tables.controls.push(ControlData {
                    occurrence,
                    participant,
                    source: control.at,
                    processed: processed_at,
                    kind: control.kind,
                    level: analyzed.analysis.control_levels[occurrence],
                });
                tables.participants.push(ParticipantData::Control(row));
            }
        }
        if let Some(item) = preliminary {
            if !seen_items[item] {
                seen_items[item] = true;
                item_count += 1;
            }
            while seen_items.get(item_prefix) == Some(&true) {
                item_prefix += 1;
            }
        }
        let next_atom = atom_cursor.max(atom);
        let at = projection.atoms.get(next_atom).map_or(
            analyzed.processed.project(
                paragraph.extent(&analyzed.processed).end,
                crate::Affinity::After,
            ),
            |value| value.presentation.start,
        );
        // The existing raw standalone chooser treats each actual CR and LF as
        // a forced item boundary, even within the original CRLF grapheme.
        // Native processed paragraphs retain their typography-safe boundaries.
        let raw_forced = paragraph.base_direction.is_none()
            && next_atom
                .checked_sub(1)
                .is_some_and(|i| matches!(projection.atoms[i].code_point, 0xd | 0xa));
        if projection.graphemes.binary_search(&at).is_ok() || raw_forced {
            let shared_ligature = cluster_for_atom
                .get(next_atom)
                .and_then(|row| *row)
                .is_some_and(|cluster| shaped.clusters[cluster].ligature_component);
            tables.boundaries.push(BoundaryData {
                participant: tables.participants.len(),
                atom: next_atom,
                processed: at,
                source_before: owned_source,
                source_after: events.peek().map_or(owned_source, |event| event.source),
                preliminary_item: (item_count == item_prefix && !shared_ligature)
                    .then_some(item_prefix),
            });
        }
    }
    tables.control_rows = vec![0; paragraph.controls.len()];
    for (row, control) in tables.controls.iter().enumerate() {
        tables.control_rows[control.occurrence] = row;
    }
    issue_opportunities(analyzed, &mut tables)?;
    Ok(tables)
}

fn issue_opportunities(analyzed: &AnalyzedData, tables: &mut PreparedTables) -> crate::Result<()> {
    use super::paragraph::ParagraphReplacementDefinition;
    let source_owners: std::collections::BTreeMap<_, _> = tables
        .units
        .iter()
        .filter(|unit| !unit.deleted && !unit.source.is_empty())
        .map(|unit| ((unit.source.start, unit.source.end), unit.participant + 1))
        .collect();
    let mut alternatives_by_owner: std::collections::BTreeMap<usize, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (occurrence, replacement) in analyzed.paragraph.replacements.iter().enumerate() {
        let owner = match replacement.definition {
            ParagraphReplacementDefinition::Source { source, .. } => {
                source_owners.get(&(source.start, source.end)).copied()
            }
            ParagraphReplacementDefinition::Control { control, .. } => {
                Some(tables.controls[tables.control_rows[control]].participant + 1)
            }
        };
        if let Some(owner) = owner {
            alternatives_by_owner
                .entry(owner)
                .or_default()
                .push(occurrence);
        }
    }
    for (index, boundary) in tables.boundaries.iter().enumerate().skip(1) {
        let previous = tables.participants.get(boundary.participant - 1);
        let control = previous.and_then(|item| match *item {
            ParticipantData::Control(i) => Some(tables.controls[i].kind),
            _ => None,
        });
        let source_break = previous.is_some_and(|item| match *item {
            ParticipantData::Text(i) => tables.units[i]
                .atoms
                .clone()
                .any(|atom| is_hard_separator(analyzed.projection.atoms[atom].code_point)),
            _ => false,
        });
        let alternative_inputs = alternatives_by_owner
            .get(&boundary.participant)
            .cloned()
            .unwrap_or_default();
        let kind = if control == Some(ParagraphControlKind::ForcedBreak) || source_break {
            Some(PreparedBreakKind::Forced)
        } else if analyzed.paragraph.base_direction.is_some()
            && analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]]
                .before_hard_separator
        {
            // UAX #14 LB4/LB5 mandatory-after rules precede LB6. Only
            // after preserving that mandatory offer may LB6 suppress every
            // soft owner before the next real hard separator, including CB.
            None
        } else if matches!(
            control,
            Some(ParagraphControlKind::DiscretionaryBreak {
                wrap: crate::Wrap::Word
            })
        ) || (!alternative_inputs.is_empty()
            && control.is_none()
            && wrapping_allowed(analyzed, boundary)?)
        {
            Some(PreparedBreakKind::Discretionary)
        } else if matches!(
            control,
            Some(ParagraphControlKind::OptionalBreak {
                wrap: crate::Wrap::Word
            })
        ) {
            Some(PreparedBreakKind::Allowed)
        } else if matches!(
            control,
            Some(
                ParagraphControlKind::OptionalBreak { .. }
                    | ParagraphControlKind::DiscretionaryBreak { .. }
            )
        ) {
            None
        } else if let Some(allowed) = atomic_permission(analyzed, boundary)? {
            allowed.then_some(PreparedBreakKind::Allowed)
        } else if standalone_opportunity(analyzed, tables, boundary)
            || (boundary.participant < tables.participants.len()
                && analyzed
                    .analysis
                    .infos
                    .get(boundary.atom)
                    .is_some_and(|info| info.0.boundary >= Boundary::Line)
                && wrapping_allowed(analyzed, boundary)?)
        {
            Some(PreparedBreakKind::Allowed)
        } else {
            None
        };
        if let Some(kind) = kind {
            let opportunity = tables.opportunities.len();
            let alternatives = alternative_inputs
                .into_iter()
                .map(|occurrence| {
                    let input = &analyzed.paragraph.replacements[occurrence];
                    let index = tables.alternatives.len();
                    tables.alternatives.push(AlternativeData {
                        occurrence,
                        opportunity,
                        definition: input.definition,
                        insertion_anchor: input.insertion_anchor,
                    });
                    index
                })
                .collect();
            tables.opportunities.push(OpportunityData {
                boundary: index,
                kind,
                alternatives,
            });
        }
    }
    let boundary = tables.boundaries.len() - 1;
    tables.opportunities.push(OpportunityData {
        boundary,
        kind: PreparedBreakKind::ParagraphEnd,
        alternatives: Vec::new(),
    });
    Ok(())
}

fn native_permission(analyzed: &AnalyzedData, boundary: &BoundaryData) -> crate::Result<bool> {
    let relation = analyzed.logical.cuts[boundary.participant];
    if analyzed.logical.relations[relation].neighbors.is_none() {
        return Ok(false);
    }
    analyzed.wrapping[relation]
        .map(|wrap| wrap == crate::Wrap::Word)
        .ok_or_else(|| {
            super::protocol_error(super::ProtocolError::MissingWrappingPolicy { relation })
        })
}

fn atomic_permission(
    analyzed: &AnalyzedData,
    boundary: &BoundaryData,
) -> crate::Result<Option<bool>> {
    if analyzed.paragraph.base_direction.is_none() {
        return Ok(None);
    }
    let relation = &analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]];
    if !relation.atomic {
        return Ok(None);
    }
    if !relation.neighbors_permit {
        return Ok(Some(false));
    }
    if relation.space_permits {
        return Ok(Some(true));
    }
    native_permission(analyzed, boundary).map(Some)
}

/// Every existing mixed cut retains its own custody; permission comes from the
/// single actual influencing-neighbor relation, never an incidental seam.
pub(crate) fn conditional_boundaries(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    start: usize,
    ceiling: usize,
    purpose: super::ConditionalPurpose,
) -> crate::Result<Vec<usize>> {
    use crate::engine::analysis::wrapping::Influence;
    if analyzed.paragraph.base_direction.is_none() {
        // Raw analysis has its own characterized ordinary inventory. This
        // native operation adds no cuts to that distinct producer model.
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for index in start + 1..ceiling {
        let boundary = &tables.boundaries[index];
        let relation = &analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]];
        let (
            Some(Influence::Text {
                participant: before_participant,
                atom: before,
            }),
            Some(Influence::Text { atom: after, .. }),
        ) = (relation.before, relation.after)
        else {
            continue;
        };
        if before_participant < tables.boundaries[start].participant
            || is_hard_separator(analyzed.projection.atoms[before].code_point)
            || is_hard_separator(analyzed.projection.atoms[after].code_point)
            || boundary.atom != after
        {
            continue;
        }
        let style = analyzed.styles[analyzed.analysis.infos[before].1]
            .style
            .authored();
        if !matches!(
            (style.overflow_wrap, purpose),
            (crate::OverflowWrap::Anywhere, _)
                | (
                    crate::OverflowWrap::BreakWord,
                    super::ConditionalPurpose::Overflow
                )
        ) {
            continue;
        }
        let before_source = analyzed.projection.atoms[before].source.end;
        let after_source = analyzed.projection.atoms[after].source.start;
        if analyzed
            .projection
            .barriers
            .iter()
            .any(|at| *at >= before_source && *at <= after_source)
        {
            continue;
        }
        if native_permission(analyzed, boundary)? {
            result.push(index);
        }
    }
    Ok(result)
}

fn standalone_opportunity(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
) -> bool {
    if analyzed.paragraph.base_direction.is_some() {
        return false;
    }
    let previous = boundary
        .participant
        .checked_sub(1)
        .and_then(|i| tables.participants.get(i));
    let next = tables.participants.get(boundary.participant);
    if matches!(previous, Some(ParticipantData::Atomic(_)))
        || matches!(next, Some(ParticipantData::Atomic(_)))
    {
        return true;
    }
    if previous.is_some_and(|row|matches!(*row,ParticipantData::Text(i)
        if tables.units[i].atoms.clone().any(|atom|matches!(analyzed.projection.atoms[atom].code_point,0x20|0xa0)))) {
        return true;
    }
    next.is_some_and(|row| match *row {
        ParticipantData::Text(i) => tables.units[i].atoms.clone().next().is_some_and(|atom| {
            let style = &analyzed.styles[analyzed.analysis.infos[atom].1]
                .style
                .authored();
            style.wrap == crate::Wrap::Word && style.overflow_wrap != crate::OverflowWrap::Normal
        }),
        _ => false,
    })
}

fn wrapping_allowed(analyzed: &AnalyzedData, boundary: &BoundaryData) -> crate::Result<bool> {
    if let Some(previous) = boundary.atom.checked_sub(1)
        && matches!(analyzed.projection.atoms[previous].code_point, 0x20 | 0xad)
    {
        // Raw standalone retains its characterized source policy. Native SHY
        // still owns the opportunity at which it disappears; a native SPACE
        // lends local authority only when its actual mode removes it there.
        if analyzed.paragraph.base_direction.is_none()
            || analyzed.projection.atoms[previous].code_point == 0xad
        {
            return Ok(analyzed.styles[analyzed.analysis.infos[previous].1]
                .style
                .authored()
                .wrap
                == crate::Wrap::Word);
        }
        if let Some(wrap) = disappearing_space_wrap(analyzed, previous) {
            return Ok(wrap == crate::Wrap::Word);
        }
    }
    if analyzed.paragraph.base_direction.is_some() {
        return native_permission(analyzed, boundary);
    }
    Ok(analyzed
        .styles
        .get(
            analyzed
                .analysis
                .infos
                .get(boundary.atom)
                .map_or(0, |info| info.1),
        )
        .is_some_and(|run| run.style.authored().wrap == crate::Wrap::Word))
}

/// The containing style owns a native SPACE opportunity only when that SPACE
/// actually disappears at the selected break. Preserved SPACE boundaries use
/// captured NCA permission instead.
fn disappearing_space_wrap(analyzed: &AnalyzedData, atom: usize) -> Option<crate::Wrap> {
    let style = analyzed.styles[analyzed.analysis.infos[atom].1]
        .style
        .authored();
    (analyzed.projection.atoms[atom].code_point == 0x20
        && style.white_space == crate::WhiteSpace::Collapse)
        .then_some(style.wrap)
}

/// Performs actual alternative font matching/shaping while the original
/// catalogue capture is still owned by preparation. No finalizer can requery it.
pub(crate) fn prepare_alternatives(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &std::sync::Arc<AnalyzedData>,
    styles: &[ResolvedStyleRun],
    tables: &PreparedTables,
    scratch: &mut crate::engine::shape::ShapeContext,
) -> crate::Result<Vec<PreparedAlternativeShape>> {
    let mut alternatives = Vec::with_capacity(tables.alternatives.len());
    for (index, row) in tables.alternatives.iter().enumerate() {
        let input = &analyzed.paragraph.replacements[row.occurrence];
        let ReplacementProjection {
            projection,
            inserted_atoms,
            original_atoms,
            base_insertion,
        } = replacement_projection(analyzed, input)?;
        let analysis = crate::engine::analysis::analyze(
            &projection,
            &analyzed.styles,
            analyzed.processed.source().boxes(),
            &analyzed.paragraph,
        )?;
        let catalogue_identity = catalogue.identity().clone();
        let shaped = crate::engine::shape::shape(
            catalogue.query(),
            crate::engine::ShapingInput {
                projection: &projection,
                analysis: &analysis,
                styles,
                boxes: analyzed.processed.source().boxes(),
                font_catalogue: &catalogue_identity,
                paragraph: &analyzed.paragraph,
            },
            scratch,
        )?;
        alternatives.push(PreparedAlternativeShape {
            alternative: index,
            inserted_atoms,
            projection,
            analysis,
            shaped,
            original_atoms,
            base_insertion,
            joining_domains: Vec::new(),
        });
    }
    Ok(alternatives)
}

struct ReplacementProjection {
    projection: SourceProjection,
    inserted_atoms: IndexRange<usize>,
    original_atoms: Vec<Option<usize>>,
    base_insertion: usize,
}

fn replacement_projection(
    analyzed: &AnalyzedData,
    replacement: &super::paragraph::ParagraphReplacement,
) -> crate::Result<ReplacementProjection> {
    let original = &analyzed.projection;
    let start = if let Some(control) = replacement.control() {
        let anchor = analyzed
            .paragraph
            .anchor_order
            .iter()
            .position(|anchor| anchor.item == ParagraphAnchorItem::Control(control))
            .expect("validated defining control");
        anchor_atom(
            anchor,
            &analyzed.paragraph,
            analyzed.processed.source().boxes(),
            original,
        )
    } else {
        original
            .atoms
            .partition_point(|atom| atom.source.end <= replacement.replaced_source().start)
    };
    let end = if replacement.replaced_source().is_empty() {
        start
    } else {
        original
            .atoms
            .partition_point(|atom| atom.source.start < replacement.replaced_source().end)
    };
    let processed_anchor = analyzed
        .processed
        .project(replacement.insertion_anchor, crate::Affinity::After);
    let mut atoms = original.atoms[..start].to_vec();
    let inserted_start = atoms.len();
    let mut unit: usize = 0;
    for decoded in char::decode_utf16(replacement.text.code_units().iter().copied()) {
        let (code_point, glyph, len) = match decoded {
            Ok(ch) => (u32::from(ch), ch, ch.len_utf16()),
            Err(error) => (u32::from(error.unpaired_surrogate()), '\u{fffd}', 1),
        };
        let next = unit
            .checked_add(len)
            .ok_or_else(crate::projection::length_overflow)?;
        atoms.push(crate::engine::OriginalAtom {
            source: Range::new(replacement.insertion_anchor, replacement.insertion_anchor),
            processed: Range::new(processed_anchor, processed_anchor),
            presentation: Range::new(unit, next),
            glyph_bytes: 0..0,
            code_point,
            glyph,
        });
        unit = next;
    }
    let inserted = inserted_start..atoms.len();
    atoms.extend_from_slice(&original.atoms[end..]);
    let mut presentation: usize = 0;
    for atom in &mut atoms {
        let next = presentation
            .checked_add(atom.presentation.len())
            .ok_or_else(crate::projection::length_overflow)?;
        atom.presentation = Range::new(presentation, next);
        presentation = next;
    }
    let mut projection =
        SourceProjection::from_atoms(atoms, presentation, original.barriers.clone())?;
    projection.style_anchors = vec![None; projection.atoms.len()];
    for atom in inserted.clone() {
        projection.style_anchors[atom] = Some(replacement.style_anchor);
    }
    projection.anchor_atoms = vec![None; analyzed.paragraph.anchor_order.len()];
    if let Some(control) = replacement.control() {
        let defining = analyzed
            .paragraph
            .anchor_order
            .iter()
            .position(|anchor| anchor.item == ParagraphAnchorItem::Control(control))
            .expect("validated defining control");
        for (order, anchor) in analyzed.paragraph.anchor_order.iter().enumerate() {
            if anchor_position(
                anchor.item,
                &analyzed.paragraph,
                analyzed.processed.source().boxes(),
            ) == replacement.insertion_anchor
            {
                projection.anchor_atoms[order] = Some(if order < defining {
                    inserted.start
                } else {
                    inserted.end
                });
            }
        }
    }
    let original_atoms = (0..start)
        .map(Some)
        .chain(inserted.clone().map(|_| None))
        .chain((end..original.atoms.len()).map(Some))
        .collect();
    Ok(ReplacementProjection {
        projection,
        inserted_atoms: inserted,
        original_atoms,
        base_insertion: start,
    })
}

/// Constructs the one selected semantic overlay after capability admission.
/// Original participant ordinals are selected-local; replacement-only units
/// have no fabricated original participant. The captured alternative supplies
/// both original-context analysis and the actual retained font assignments.
pub(crate) fn select_presentation(
    prepared: &crate::engine::PreparedData,
    request: &super::LineSelectionRequest,
) -> crate::Result<SelectedPresentation> {
    use super::paragraph::ParagraphReplacementDefinition;
    let tables = &prepared.tables;
    let start = tables.boundaries[request.start.index];
    let end = tables.boundaries[request.end.index];
    let alternative = request.alternative.as_ref().map(|value| value.index);
    let variant = alternative.map(|index| &prepared.alternative_shapes[index]);
    let projection = variant.map_or(&prepared.analyzed.projection, |value| &value.projection);
    let analysis = variant.map_or(&prepared.analyzed.analysis, |value| &value.analysis);
    let shaped = variant.map_or(&prepared.shaped, |value| &value.shaped);
    let definition = alternative.map(|i| tables.alternatives[i].definition);
    let mut output = PresentationOutput::default();
    let mut original_cursor = 0usize;
    for (local, participant) in tables.participants[start.participant..end.participant]
        .iter()
        .enumerate()
    {
        let defining = match (definition, *participant) {
            (
                Some(ParagraphReplacementDefinition::Source { source, .. }),
                ParticipantData::Text(i),
            ) => {
                !tables.units[i].deleted
                    && tables.units[i].source.start <= source.start
                    && tables.units[i].source.end >= source.end
            }
            (
                Some(ParagraphReplacementDefinition::Control { control, .. }),
                ParticipantData::Control(i),
            ) => tables.controls[i].occurrence == control,
            _ => false,
        };
        // Source-defined SHY owns its deleted authored interval even when its
        // computed replacement is empty. Native controls own no authored text.
        if defining && let ParticipantData::Text(i) = *participant {
            let row = &tables.units[i];
            push_presentation_unit(
                &mut output,
                projection,
                &[],
                UnitCapture {
                    source: row.source,
                    processed: row.processed,
                    participant: Some(local),
                    kind: PresentationUnitKind::Deleted,
                    level: row.level,
                    bidi_class: None,
                },
            )?;
        }
        if defining {
            let variant = variant.expect("a defining replacement has a captured variant");
            let row = &tables.alternatives[alternative.expect("selected alternative")];
            let processed = prepared
                .analyzed
                .processed
                .project(row.insertion_anchor, crate::Affinity::After);
            if variant.inserted_atoms.is_empty() {
                push_presentation_unit(
                    &mut output,
                    projection,
                    &[],
                    UnitCapture {
                        source: Range::new(row.insertion_anchor, row.insertion_anchor),
                        processed: Range::new(processed, processed),
                        participant: None,
                        kind: PresentationUnitKind::Replacement {
                            alternative: variant.alternative,
                        },
                        level: match *participant {
                            ParticipantData::Text(i) => tables.units[i].level,
                            ParticipantData::Control(i) => {
                                analysis.control_levels[tables.controls[i].occurrence]
                            }
                            ParticipantData::Atomic(_) => {
                                unreachable!("replacement definitions are text or controls")
                            }
                        },
                        bidi_class: None,
                    },
                )?;
            } else {
                for index in variant.inserted_atoms.clone() {
                    push_presentation_unit(
                        &mut output,
                        projection,
                        &[index],
                        UnitCapture {
                            source: Range::new(row.insertion_anchor, row.insertion_anchor),
                            processed: Range::new(processed, processed),
                            participant: None,
                            kind: PresentationUnitKind::Replacement {
                                alternative: variant.alternative,
                            },
                            level: analysis.levels[index],
                            bidi_class: Some(analysis.infos[index].0.bidi_class),
                        },
                    )?;
                }
            }
        }
        match *participant {
            ParticipantData::Text(i) if !defining => {
                let row = &tables.units[i];
                let kind = if let Some(origin) = row.empty_origin {
                    PresentationUnitKind::EmptyOrigin { origin }
                } else if row.deleted {
                    PresentationUnitKind::Deleted
                } else {
                    PresentationUnitKind::Text
                };
                let mut selected_atoms = Vec::new();
                if !row.atoms.is_empty() {
                    while original_cursor < projection.atoms.len()
                        && projection.atoms[original_cursor].source.end <= row.source.start
                    {
                        original_cursor += 1;
                    }
                    let mut index = original_cursor;
                    while index < projection.atoms.len()
                        && projection.atoms[index].source.start < row.source.end
                    {
                        if !variant.is_some_and(|v| v.inserted_atoms.contains(&index)) {
                            selected_atoms.push(index);
                        }
                        index += 1;
                    }
                    original_cursor = index;
                }
                let level = selected_atoms
                    .first()
                    .map_or(row.level, |i| analysis.levels[*i]);
                let class = selected_atoms
                    .first()
                    .map(|i| analysis.infos[*i].0.bidi_class);
                push_presentation_unit(
                    &mut output,
                    projection,
                    &selected_atoms,
                    UnitCapture {
                        source: row.source,
                        processed: row.processed,
                        participant: Some(local),
                        kind,
                        level,
                        bidi_class: class,
                    },
                )?;
            }
            ParticipantData::Text(_) => {}
            ParticipantData::Atomic(i) => {
                let row = tables.atomics[i];
                push_presentation_unit(
                    &mut output,
                    projection,
                    &[],
                    UnitCapture {
                        source: Range::new(row.source, row.source),
                        processed: Range::new(row.processed, row.processed),
                        participant: Some(local),
                        kind: PresentationUnitKind::Atomic {
                            occurrence: row.occurrence,
                        },
                        level: analysis.box_levels[row.occurrence],
                        bidi_class: None,
                    },
                )?;
            }
            ParticipantData::Control(i) => {
                let row = tables.controls[i];
                let cp = if row.kind == ParagraphControlKind::ForcedBreak {
                    Some(0xa)
                } else {
                    row.kind.code_point()
                };
                let class = cp.map(|cp| {
                    icu_properties::CodePointMapData::<icu_properties::props::BidiClass>::new()
                        .get32(cp)
                });
                push_presentation_unit(
                    &mut output,
                    projection,
                    &[],
                    UnitCapture {
                        source: Range::new(row.source, row.source),
                        processed: Range::new(row.processed, row.processed),
                        participant: Some(local),
                        kind: PresentationUnitKind::Control {
                            occurrence: row.occurrence,
                        },
                        level: analysis.control_levels[row.occurrence],
                        bidi_class: class,
                    },
                )?;
            }
        }
    }
    let PresentationOutput {
        atoms,
        indices,
        units,
        pieces,
        position: presentation,
    } = output;
    let selected_projection =
        SourceProjection::from_atoms(atoms, presentation, projection.barriers.clone())?;
    let infos = indices.iter().map(|i| analysis.infos[*i]).collect();
    let levels = indices.iter().map(|i| analysis.levels[*i]).collect();
    let graphemes = selected_projection
        .graphemes
        .windows(2)
        .filter_map(|pair| {
            let start = selected_projection
                .atoms
                .partition_point(|a| a.presentation.start < pair[0]);
            let end = selected_projection
                .atoms
                .partition_point(|a| a.presentation.start < pair[1]);
            (start < end).then_some(start..end)
        })
        .collect();
    let selected_analysis = Analysis {
        infos,
        levels,
        graphemes,
        base_level: analysis.base_level,
        box_levels: analysis.box_levels.clone(),
        control_levels: analysis.control_levels.clone(),
    };
    let mut assignments: Vec<RetainedAssignment> = Vec::new();
    for (selected_atom, original_atom) in indices.into_iter().enumerate() {
        let run = shaped
            .runs
            .partition_point(|run| run.atoms.end <= original_atom);
        if !shaped
            .runs
            .get(run)
            .is_some_and(|run| run.atoms.contains(&original_atom))
        {
            return Err(crate::Error::new(
                crate::ErrorCode::LayoutFailed,
                "selected atom has no captured font assignment",
            ));
        }
        if let Some(previous) = assignments.last_mut()
            && previous.run == run
            && previous.alternative == alternative
            && previous.atoms.end == selected_atom
            && previous.captured_atoms.end == original_atom
        {
            previous.atoms.end += 1;
            previous.captured_atoms.end += 1;
        } else {
            assignments.push(RetainedAssignment {
                atoms: selected_atom..selected_atom + 1,
                captured_atoms: original_atom..original_atom + 1,
                alternative,
                run,
            });
        }
    }
    Ok(SelectedPresentation {
        source: Range::new(start.source_after, end.source_before),
        units,
        projection: selected_projection,
        analysis: selected_analysis,
        assignments,
        mapping: std::sync::Arc::new(SelectedMapping { pieces }),
    })
}

#[derive(Default)]
struct PresentationOutput {
    units: Vec<PresentationUnit>,
    pieces: Vec<SelectedMappingPiece>,
    position: usize,
    atoms: Vec<crate::engine::OriginalAtom>,
    indices: Vec<usize>,
}
struct UnitCapture {
    source: Range,
    processed: Range,
    participant: Option<usize>,
    kind: PresentationUnitKind,
    level: u8,
    bidi_class: Option<icu_properties::props::BidiClass>,
}
fn push_presentation_unit(
    output: &mut PresentationOutput,
    projection: &SourceProjection,
    source_atoms: &[usize],
    capture: UnitCapture,
) -> crate::Result<()> {
    let UnitCapture {
        source,
        processed,
        participant,
        kind,
        level,
        bidi_class,
    } = capture;
    let start = output.position;
    let atom_start = output.atoms.len();
    for &index in source_atoms {
        let mut atom = projection.atoms[index].clone();
        let end = output
            .position
            .checked_add(atom.presentation.len())
            .ok_or_else(crate::projection::length_overflow)?;
        atom.presentation = Range::new(output.position, end);
        output.position = end;
        output.atoms.push(atom);
        output.indices.push(index);
    }
    let presentation = Range::new(start, output.position);
    let code_point =
        (source_atoms.len() == 1).then(|| projection.atoms[source_atoms[0]].code_point);
    output.units.push(PresentationUnit {
        source,
        processed,
        presentation,
        participant,
        code_point,
        bidi_class,
        level,
        atoms: atom_start..output.atoms.len(),
        eligible_space: code_point == Some(0x20),
        kind,
    });
    output.pieces.push(SelectedMappingPiece {
        source,
        processed,
        presentation,
        kind: match kind {
            PresentationUnitKind::Deleted => SelectedMappingKind::Deleted,
            PresentationUnitKind::Replacement { alternative } => SelectedMappingKind::Replacement {
                alternative,
                anchor: source.start,
            },
            _ => SelectedMappingKind::Source,
        },
    });
    Ok(())
}

/// Actual selected occurrence associations, including distinct selected empty
/// origins and source-less native controls/alternatives. No inclusive numeric
/// anchor query invents an occurrence on the adjacent line.
pub(crate) fn selected_origin_intersections(
    selected: &super::selection::SelectedData,
) -> Vec<super::paragraph::FragmentAssociation> {
    use super::paragraph::{FragmentAssociation, FragmentOwner, ParagraphReplacementDefinition};
    let paragraph = &selected.prepared.analyzed.paragraph;
    let processed = &selected.prepared.analyzed.processed;
    let mut fragments = Vec::new();
    for unit in &selected.presentation.units {
        match unit.kind {
            PresentationUnitKind::Control { occurrence } => fragments.push(FragmentAssociation {
                owner: FragmentOwner::Control(occurrence),
                source: None,
            }),
            PresentationUnitKind::Replacement { alternative } => {
                let row = selected.prepared.tables.alternatives[alternative];
                let origin = match row.definition {
                    ParagraphReplacementDefinition::Source { origin, .. } => Some(origin),
                    ParagraphReplacementDefinition::Control { origin, .. } => origin,
                };
                let source = origin.map(|origin| {
                    let row = &paragraph.origins[origin];
                    let at = unit.source.start;
                    let local = row.local.start + at - row.aggregate.start;
                    super::paragraph::OriginIntersection {
                        origin,
                        aggregate: unit.source,
                        local: Range::new(local, local),
                        processed: unit.processed,
                    }
                });
                let association = FragmentAssociation {
                    owner: FragmentOwner::Alternative(alternative),
                    source,
                };
                if !fragments
                    .last()
                    .is_some_and(|row: &FragmentAssociation| row.owner == association.owner)
                {
                    fragments.push(association);
                }
            }
            PresentationUnitKind::EmptyOrigin { origin } => {
                let row = &paragraph.origins[origin];
                fragments.push(FragmentAssociation {
                    owner: FragmentOwner::Origin(origin),
                    source: Some(super::paragraph::OriginIntersection {
                        origin,
                        aggregate: row.aggregate,
                        local: row.local,
                        processed: unit.processed,
                    }),
                });
            }
            PresentationUnitKind::Text | PresentationUnitKind::Deleted => {
                fragments.extend(
                    paragraph
                        .origin_intersections(processed, unit.source)
                        .filter(|i| !paragraph.origins[i.origin].aggregate.is_empty())
                        .map(|source| FragmentAssociation {
                            owner: FragmentOwner::Origin(source.origin),
                            source: Some(source),
                        }),
                );
            }
            PresentationUnitKind::Atomic { .. } => {}
        }
    }
    fragments
}
