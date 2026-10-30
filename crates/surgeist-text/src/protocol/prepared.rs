//! Canonical preparation tables; handles borrow these actual issued rows.

mod fragments;
pub(crate) mod presentation;
#[cfg(test)]
pub(crate) use fragments::selected_origin_intersections;
pub(crate) use fragments::visit_selected_origin_intersections;
#[cfg(test)]
mod tests;

use std::ops::Range as IndexRange;

use crate::engine::{
    AnalyzedData,
    analysis::{Analysis, Boundary, anchor_atom, anchor_position, is_hard_separator},
    layout::data::{ShapedData, ShapedItem},
    resolve::ResolvedStyleRun,
};
use crate::style::OrdinaryWordBreak;
use crate::{Range, projection::SourceProjection, transform::relation::GeneratedSeparatorOwner};

use super::paragraph::{ParagraphControlKind, SemanticAnchor, SemanticAnchorItem};

/// Immutable logical participants and their exact occurrence/boundary tables.
#[derive(Debug)]
pub(crate) struct PreparedTables {
    pub(crate) profile: crate::engine::PreparationProfile,
    pub(crate) words: Vec<super::hyphenation::WordData>,
    pub(crate) hyphenation_cuts: Vec<super::hyphenation::CutData>,
    pub(crate) participants: Vec<ParticipantData>,
    pub(crate) units: Vec<UnitData>,
    pub(crate) atomics: Vec<AtomicData>,
    pub(crate) controls: Vec<ControlData>,
    pub(crate) computed_boundaries: Vec<ComputedBoundaryData>,
    /// Exact captured control occurrence to its actual issued table row.
    pub(crate) control_rows: Vec<usize>,
    pub(crate) boundaries: Vec<BoundaryData>,
    pub(crate) opportunities: Vec<OpportunityData>,
    pub(crate) alternatives: Vec<AlternativeData>,
    pub(crate) ordinary_alternatives: Vec<usize>,
    pub(crate) suppressed: Vec<SuppressedOpportunityData>,
    pub(crate) raw_conditional: Vec<usize>,
    pub(crate) latent_alternatives: Vec<usize>,
    /// Counts actual text and discretionary-control owners, in participant order.
    auto_phrase_prefix: Vec<usize>,
    /// Derived index of actual soft opportunities for the private raw chooser.
    pub(crate) soft_preliminary_items: std::collections::BTreeSet<usize>,
}

impl PreparedTables {
    pub(crate) fn auto_phrase_between(&self, start: usize, end: usize) -> bool {
        self.auto_phrase_prefix[self.boundaries[start].participant]
            != self.auto_phrase_prefix[self.boundaries[end].participant]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParticipantData {
    Text(usize),
    Atomic(usize),
    Control(usize),
    ComputedLineBoundary(usize),
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
    pub(crate) anchor: usize,
    pub(crate) semantic_atom: Option<usize>,
    /// Actual owning mixed participant, including a transformed Wbr text row.
    pub(crate) participant: Option<usize>,
    pub(crate) occurrence: usize,
    pub(crate) source: usize,
    pub(crate) processed: usize,
    pub(crate) level: u8,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ComputedBoundaryData {
    pub(crate) occurrence: usize,
    pub(crate) source: usize,
    pub(crate) processed: usize,
    pub(crate) level: u8,
}

/// A mixed-sequence boundary retains both original incident extrema.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BoundaryData {
    pub(crate) participant: usize,
    pub(crate) atom: usize,
    pub(crate) processed: usize,
    pub(crate) relation_cut: usize,
    pub(crate) source_before: usize,
    pub(crate) source_after: usize,
    /// Actual collapsed-follower gate at this cut. Transparent custody retains
    /// it; a new influencing participant ends that side's local authority.
    pub(crate) collapsed_space_wrap: Option<crate::Wrap>,
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

#[derive(Debug)]
pub(crate) struct SuppressedOpportunityData {
    pub(crate) boundary: usize,
    pub(crate) phrase: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct AlternativeData {
    pub(crate) text: crate::Text,
    pub(crate) placement: crate::HyphenationPlacement,
    pub(crate) resolution: IndicationResolution,
    pub(crate) style: usize,
    pub(crate) opportunity: Option<usize>,
    pub(crate) boundary: usize,
    pub(crate) definition: AlternativeDefinition,
    pub(crate) hyphenation_cut: usize,
    pub(crate) insertion_anchor: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum AlternativeDefinition {
    Source {
        origin: usize,
        source: Range,
    },
    Control {
        control: usize,
        origin: Option<usize>,
    },
    Automatic {
        cut: usize,
        atom: usize,
        origin: usize,
    },
}
impl AlternativeDefinition {
    fn manual(value: super::paragraph::ParagraphReplacementDefinition) -> Self {
        match value {
            super::paragraph::ParagraphReplacementDefinition::Source { origin, source } => {
                Self::Source { origin, source }
            }
            super::paragraph::ParagraphReplacementDefinition::Control { control, origin } => {
                Self::Control { control, origin }
            }
        }
    }
}

/// Resolution provenance is retained with the canonical owning style row.
#[derive(Clone, Debug)]
pub(crate) enum IndicationResolution {
    Specified,
    Convention,
    Primary(crate::FontData),
    ResolvedNative,
}

impl AlternativeData {
    fn control(&self) -> Option<usize> {
        match self.definition {
            AlternativeDefinition::Control { control, .. } => Some(control),
            _ => None,
        }
    }
    fn replaced_source(&self) -> Range {
        match self.definition {
            AlternativeDefinition::Source { source, .. } => source,
            _ => Range::new(self.insertion_anchor, self.insertion_anchor),
        }
    }
}

/// Actual replacement-context preparation under the same catalogue capture.
///
/// This is private variant work in the same preparation, not a separately
/// issued public source/preparation. Every retained face/instance comes from
/// actual resolution before publication. Finalization never queries a newer
/// catalogue to reconstruct it.
pub(crate) struct PreparedIndicationFonts {
    pub(crate) inserted_atoms: IndexRange<usize>,
    pub(crate) projection: SourceProjection,
    pub(crate) analysis: Analysis,
    pub(crate) font_queries: Vec<crate::engine::shape::FontQueryWitness>,
}

/// Exact selected presentation correspondence, separate from authored text.
#[derive(Debug)]
pub(crate) struct SelectedMapping {
    pub(crate) pieces: Vec<SelectedMappingPiece>,
}

/// One validated selection's actual provider-independent presentation. It is
/// built once, then shared by spacing admission and retained-face shaping.
pub(crate) struct SelectedPresentation {
    pub(crate) vertical_units: Vec<crate::engine::analysis::upright::VerticalUnit>,
    pub(crate) source: Range,
    pub(crate) selected_relation: SelectedRelation,
    pub(crate) units: Vec<PresentationUnit>,
    pub(crate) projection: SourceProjection,
    pub(crate) analysis: Analysis,
    pub(crate) assignments: Vec<RetainedAssignment>,
    pub(crate) originals: Vec<Option<usize>>,
    pub(crate) domains: Vec<presentation::IndicationDomain>,
    pub(crate) mapping: std::sync::Arc<SelectedMapping>,
}

/// Executed logical relation for one selection. Indication atoms retain their
/// selected-local identity and never pretend to be transformed authored atoms.
#[derive(Clone, Debug)]
pub(crate) enum SelectedRelationEvent {
    Base { event: usize },
    Indication { unit: usize, atom: usize },
}

#[derive(Default)]
pub(crate) struct SelectedRelation {
    pub(crate) events: Vec<SelectedRelationEvent>,
    pub(crate) atom_events: Vec<usize>,
    pub(crate) unit_events: Vec<IndexRange<usize>>,
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
    /// Selected-line L1 authority, distinct from original/provider levels.
    pub(crate) line_level: u8,
    pub(crate) kind: PresentationUnitKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresentationUnitKind {
    Text,
    Deleted,
    EmptyOrigin { origin: usize },
    Atomic { occurrence: usize },
    Control { occurrence: usize },
    ComputedLineBoundary { boundary: usize },
    Replacement { alternative: usize, local: Range },
    GeneratedSeparator { owner: GeneratedSeparatorOwner },
}

/// Exact strong-font row already resolved in the original preparation or one
/// captured alternative. Atom extents refer to this selected projection.
#[derive(Clone, Debug)]
pub(crate) struct RetainedAssignment {
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) run: crate::engine::layout::data::PreparedRun,
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
    ComputedLineBoundary { boundary: usize },
    Deleted,
    Replacement { alternative: usize, anchor: usize },
    GeneratedSeparator { owner: GeneratedSeparatorOwner },
}

/// Issues canonical units from actual atoms, executed deletions and captured
/// anchor occurrences. No preliminary item/source offset issues a public handle.
pub(crate) fn prepare_tables(
    catalogue: &mut crate::catalogue::Catalogue,
    styles: &[ResolvedStyleRun],
    analyzed: &AnalyzedData,
    shaped: &ShapedData,
    profile: crate::engine::PreparationProfile,
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
    let mut tables = PreparedTables {
        profile,
        words: super::hyphenation::words(analyzed)?,
        hyphenation_cuts: Vec::new(),
        participants: Vec::new(),
        units: Vec::new(),
        atomics: Vec::new(),
        controls: Vec::new(),
        computed_boundaries: Vec::new(),
        control_rows: Vec::new(),
        boundaries: Vec::new(),
        opportunities: Vec::new(),
        alternatives: Vec::new(),
        ordinary_alternatives: Vec::new(),
        suppressed: Vec::new(),
        raw_conditional: Vec::new(),
        latent_alternatives: Vec::new(),
        auto_phrase_prefix: vec![0],
        soft_preliminary_items: std::collections::BTreeSet::new(),
    };
    // Every genuine control retains an observation, including a Wbr consumed
    // by one text-bearing participant. Inventory order remains anchor order.
    tables.control_rows = vec![usize::MAX; paragraph.controls.len()];
    for (order, anchor) in paragraph.anchor_order.iter().enumerate() {
        if let SemanticAnchorItem::Control(occurrence) = anchor.item {
            tables.control_rows[occurrence] = tables.controls.len();
            let control = &paragraph.controls[occurrence];
            tables.controls.push(ControlData {
                anchor: order,
                semantic_atom: None,
                participant: None,
                occurrence,
                source: control.at,
                processed: analyzed
                    .processed
                    .project(control.at, crate::Affinity::After),
                level: analyzed.analysis.control_levels[occurrence],
            });
        }
    }
    let mut seen_items = vec![false; shaped.items.len()];
    let mut item_prefix = 0;
    let mut item_count = 0;
    let mut owned_source = paragraph.extent(&analyzed.processed).start;
    let mut atom_cursor = 0;
    let mut collapsed_space_wrap = None;
    tables.boundaries.push(BoundaryData {
        participant: 0,
        atom: 0,
        relation_cut: 0,
        processed: analyzed
            .processed
            .project(owned_source, crate::Affinity::Before),
        source_before: owned_source,
        source_after: owned_source,
        collapsed_space_wrap: None,
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
                if let crate::transform::relation::ComponentOwner::Wbr(control) =
                    analyzed.transform.component_for_atom(index).owner
                {
                    let row = &mut tables.controls[tables.control_rows[control]];
                    row.participant = Some(participant);
                    row.semantic_atom.get_or_insert(index);
                }
                collapsed_space_wrap = None;
                let value = &projection.atoms[index];
                let cluster = cluster_for_atom[index].map(|index| &shaped.clusters[index]);
                let metrics = cluster
                    .and_then(|cluster| {
                        cluster
                            .content
                            .font_run()
                            .map(|run| shaped.runs[run].metrics)
                    })
                    .unwrap_or_default();
                let item = item_for_atom[index];
                let row = UnitData {
                    empty_origin: None,
                    deleted: false,
                    source: value.source,
                    processed: value.processed,
                    atoms: index..index + 1,
                    advance: cluster.map_or(0.0, |cluster| cluster.advance),
                    ascent: metrics.ascent,
                    descent: metrics.descent,
                    line_height: cluster
                        .filter(|cluster| cluster.content.font_run().is_some())
                        .map_or(0.0, |cluster| cluster.line_height),
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
            Event::Deleted(range, at, _) => {
                if let Some(wrap) = collapsed_follower_wrap(analyzed, range) {
                    collapsed_space_wrap = Some(wrap);
                }
                let unit = tables.units.len();
                tables.units.push(UnitData {
                    empty_origin: None,
                    deleted: true,
                    source: range,
                    processed: Range::new(at, at),
                    atoms: atom..atom,
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
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::EmptyOrigin(origin),
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
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::Atomic(occurrence),
                affinity,
            }) => {
                if source.boxes()[occurrence].kind() == crate::InlineBoxKind::InFlow {
                    collapsed_space_wrap = None;
                }
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
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::ComputedLineBoundary(occurrence),
                ..
            }) => {
                collapsed_space_wrap = None;
                let row = tables.computed_boundaries.len();
                tables.computed_boundaries.push(ComputedBoundaryData {
                    occurrence,
                    source: paragraph.computed_boundaries[occurrence].at,
                    processed: processed_at,
                    level: analyzed.analysis.computed_levels[occurrence],
                });
                tables
                    .participants
                    .push(ParticipantData::ComputedLineBoundary(row));
            }
            Event::Anchor(SemanticAnchor {
                item: SemanticAnchorItem::Control(occurrence),
                ..
            }) => {
                let row = tables.control_rows[occurrence];
                let control = &paragraph.controls[occurrence];
                if matches!(
                    control.kind,
                    ParagraphControlKind::ForcedBreak
                        | ParagraphControlKind::OptionalBreak { .. }
                        | ParagraphControlKind::DiscretionaryBreak { .. }
                ) {
                    collapsed_space_wrap = None;
                }
                tables.controls[row] = ControlData {
                    anchor: tables.controls[row].anchor,
                    semantic_atom: Some(atom),
                    participant: Some(participant),
                    occurrence,
                    source: control.at,
                    processed: processed_at,
                    level: analyzed.analysis.control_levels[occurrence],
                };
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
        let at = projection
            .atoms
            .get(next_atom)
            .map_or(projection.presentation_end(), |value| {
                value.presentation.start
            });
        let processed = projection.atoms.get(next_atom).map_or_else(
            || {
                analyzed.processed.project(
                    paragraph.extent(&analyzed.processed).end,
                    crate::Affinity::After,
                )
            },
            |value| value.processed.start,
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
                relation_cut: analyzed
                    .transform
                    .cut_for_participant(tables.participants.len()),
                atom: next_atom,
                processed,
                source_before: owned_source,
                source_after: events.peek().map_or(owned_source, |event| event.source),
                collapsed_space_wrap,
                preliminary_item: (item_count == item_prefix && !shared_ligature)
                    .then_some(item_prefix),
            });
        }
    }
    for participant in &tables.participants {
        let owns = match *participant {
            ParticipantData::Text(index) => tables.units[index].atoms.clone().any(|atom| {
                analyzed.styles[analyzed.analysis.infos[atom].1]
                    .style
                    .authored()
                    .effective_breaking()
                    .auto_phrase
            }),
            ParticipantData::Control(index) => {
                matches!(&paragraph.controls[tables.controls[index].occurrence].kind,
                ParagraphControlKind::DiscretionaryBreak {style} if style.effective_breaking().auto_phrase)
            }
            ParticipantData::Atomic(_) | ParticipantData::ComputedLineBoundary(_) => false,
        };
        tables
            .auto_phrase_prefix
            .push(tables.auto_phrase_prefix.last().copied().unwrap() + usize::from(owns));
    }
    issue_opportunities(catalogue, analyzed, styles, &mut tables)?;
    tables
        .suppressed
        .sort_unstable_by_key(|row| (row.boundary, row.phrase));
    tables
        .suppressed
        .dedup_by_key(|row| (row.boundary, row.phrase));
    tables.raw_conditional.sort_unstable();
    tables.raw_conditional.dedup();
    tables.latent_alternatives = tables
        .alternatives
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.opportunity.is_none().then_some(index))
        .collect();
    tables
        .latent_alternatives
        .sort_by_key(|index| tables.alternatives[*index].boundary);
    if paragraph.base_direction.is_none() {
        tables.soft_preliminary_items = tables
            .opportunities
            .iter()
            .filter(|row| {
                matches!(
                    row.kind,
                    PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
                )
            })
            .filter_map(|row| tables.boundaries[row.boundary].preliminary_item)
            .collect();
    }
    Ok(tables)
}

fn issue_opportunities(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &AnalyzedData,
    styles: &[ResolvedStyleRun],
    tables: &mut PreparedTables,
) -> crate::Result<()> {
    let groups: std::collections::BTreeMap<_, _> = analyzed
        .paragraph
        .replacements
        .iter()
        .map(|row| (row.definition.owner(), row))
        .collect();
    let control_styles: std::collections::BTreeMap<_, _> = analyzed
        .styles
        .iter()
        .enumerate()
        .filter_map(|(index, row)| match row.owner {
            crate::engine::resolve::AnalyzedStyleOwner::Control(control) => Some((control, index)),
            _ => None,
        })
        .collect();
    for index in 1..tables.boundaries.len() {
        let boundary_value = tables.boundaries[index];
        let boundary = &boundary_value;
        let computed_break = matches!(
            tables.participants.get(boundary.participant - 1),
            Some(ParticipantData::ComputedLineBoundary(_))
        );
        let previous = tables.participants.get(boundary.participant - 1);
        let control = previous.and_then(|item| match *item {
            ParticipantData::Control(i) => {
                Some(&analyzed.paragraph.controls[tables.controls[i].occurrence].kind)
            }
            _ => None,
        });
        let source_break = previous.is_some_and(|item| match *item {
            ParticipantData::Text(i) => tables.units[i]
                .atoms
                .clone()
                .any(|atom| is_hard_separator(analyzed.projection.atoms[atom].code_point)),
            _ => false,
        });
        // Only actual surviving source SHY/control occurrences own indications.
        let owner = match previous {
            Some(ParticipantData::Control(i))
                if matches!(
                    control,
                    Some(ParagraphControlKind::DiscretionaryBreak { .. })
                ) =>
            {
                let occurrence = tables.controls[*i].occurrence;
                let style = control_styles[&occurrence];
                Some((
                    super::paragraph::ParagraphReplacementDefinition::Control {
                        control: occurrence,
                        origin: None,
                    },
                    tables.controls[*i].source,
                    style,
                ))
            }
            Some(ParticipantData::Text(i))
                if !tables.units[*i].deleted && tables.units[*i].atoms.len() == 1 =>
            {
                let atom = tables.units[*i].atoms.start;
                let value = &analyzed.projection.atoms[atom];
                if value.code_point == 0xad {
                    let origin = analyzed
                        .paragraph
                        .origin_intersections(&analyzed.processed, value.source)
                        .find(|intersection| intersection.aggregate == value.source)
                        .expect("validated positive original SHY contribution")
                        .origin;
                    Some((
                        super::paragraph::ParagraphReplacementDefinition::Source {
                            origin,
                            source: value.source,
                        },
                        value.source.end,
                        analyzed.analysis.infos[atom].1,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        };
        let hyphenation = owner.is_some();
        let mut cut = None;
        if let Some((_, _, style)) = owner {
            let word = analyzed.words.word_at(boundary.atom);
            let preceding = if control.is_some() {
                None
            } else {
                word.and_then(|w| {
                    analyzed.words.rows()[w]
                        .units
                        .iter()
                        .find(|unit| unit.end == boundary.atom)
                        .cloned()
                })
            };
            cut = Some(tables.hyphenation_cuts.len());
            tables.hyphenation_cuts.push(super::hyphenation::CutData {
                word,
                boundary: index,
                style,
                preceding,
                counts: analyzed.words.at(boundary.atom),
                opportunity: None,
                provenance: if control.is_some() {
                    super::HyphenationProvenance::Control
                } else {
                    super::HyphenationProvenance::Source
                },
            });
        }
        let mut alternatives = Vec::new();
        if let Some((definition, anchor, style)) = owner {
            let authored = analyzed.styles[style].style.authored();
            let before_hard = analyzed.paragraph.base_direction.is_some()
                && analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]]
                    .before_hard_separator;
            let count_admitted = tables.profile == crate::engine::PreparationProfile::Standalone
                || analyzed
                    .words
                    .at(boundary.atom)
                    .is_some_and(|counts| counts.admits(authored.hyphenate_limit_chars));
            let eligible = count_admitted
                && !before_hard
                && authored.hyphens != crate::Hyphens::None
                && authored.effective_breaking().word != OrdinaryWordBreak::BreakAll
                && authored.line_break != crate::LineBreak::Anywhere
                && authored.wrap == crate::Wrap::Word
                && (control.is_some() || wrapping_allowed(analyzed, boundary)?);
            if eligible {
                let supplied = groups.get(&definition.owner()).copied();
                let definition = supplied.map_or(definition, |row| row.definition);
                let (texts, resolution) = indication(catalogue, styles, authored, style, supplied)?;
                alternatives = texts
                    .into_iter()
                    .map(|text| AlternativeData {
                        text,
                        placement: used_placement(authored, &resolution),
                        resolution: resolution.clone(),
                        style,
                        opportunity: None,
                        boundary: index,
                        definition: AlternativeDefinition::manual(definition),
                        hyphenation_cut: cut.expect("manual owner issued a cut"),
                        insertion_anchor: anchor,
                    })
                    .collect();
            }
        }
        // Resource cuts and authored cuts remain different provenance. Omitted
        // SHY leaves two query endpoints; its authored identity wins that gap.
        let automatic = if !hyphenation && !source_break && !computed_break {
            automatic_cut(analyzed, tables, index)?
        } else {
            None
        };
        if let Some((automatic_cut, style, origin)) = automatic {
            let authored = analyzed.styles[style].style.authored();
            let (texts, resolution) = indication(catalogue, styles, authored, style, None)?;
            alternatives.extend(texts.into_iter().map(|text| AlternativeData {
                text,
                placement: used_placement(authored, &resolution),
                resolution: resolution.clone(),
                style,
                opportunity: None,
                boundary: index,
                definition: AlternativeDefinition::Automatic {
                    cut: automatic_cut,
                    atom: boundary.atom,
                    origin,
                },
                hyphenation_cut: automatic_cut,
                insertion_anchor: analyzed.projection.atoms[boundary.atom - 1].source.end,
            }));
            cut = Some(automatic_cut);
        }
        let kind = if computed_break
            || control == Some(&ParagraphControlKind::ForcedBreak)
            || source_break
        {
            Some(PreparedBreakKind::Forced)
        } else if analyzed.paragraph.base_direction.is_some()
            && analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]]
                .before_hard_separator
        {
            // UAX #14 LB4/LB5 mandatory-after rules precede LB6. Only
            // after preserving that mandatory offer may LB6 suppress every
            // soft owner before the next real hard separator, including CB.
            None
        } else if hyphenation {
            let atomic = atomic_permission(analyzed, boundary)?.unwrap_or_else(|| {
                analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]].atomic
                    && standalone_atomic_opportunity(analyzed, tables, boundary)
            });
            if atomic
                || ordinary_after_source_shy(analyzed, tables, boundary)?
                || ordinary_text_opportunity(analyzed, boundary)?
            {
                Some(PreparedBreakKind::Allowed)
            } else {
                (!alternatives.is_empty()).then_some(PreparedBreakKind::Discretionary)
            }
        } else if ordinary_text_opportunity(analyzed, boundary)?
            || matches!(
                control,
                Some(ParagraphControlKind::OptionalBreak {
                    wrap: crate::Wrap::Word
                })
            )
            || matches!(control, Some(ParagraphControlKind::Wbr {style}) if style.wrap==crate::Wrap::Word
            )
        {
            Some(PreparedBreakKind::Allowed)
        } else if matches!(
            control,
            Some(
                ParagraphControlKind::OptionalBreak { .. }
                    | ParagraphControlKind::Wbr { .. }
                    | ParagraphControlKind::DiscretionaryBreak { .. }
            )
        ) {
            None
        } else if let Some(allowed) = atomic_permission(analyzed, boundary)? {
            allowed.then_some(PreparedBreakKind::Allowed)
        } else if standalone_atomic_opportunity(analyzed, tables, boundary) {
            Some(PreparedBreakKind::Allowed)
        } else if boundary
            .atom
            .checked_sub(1)
            .is_some_and(|atom| analyzed.projection.atoms[atom].code_point == 0xad)
        {
            // Only the genuine defining SHY cut owns its indication ending.
            // A later transparent occurrence adds custody, never a second,
            // indication-free copy of the SHY's ICU permission. Independent
            // explicit controls and atomic permission were considered above.
            None
        } else if (boundary.participant < tables.participants.len()
            && break_spaces_opportunity(analyzed, tables, boundary)
            && wrapping_allowed(analyzed, boundary)?)
            || standalone_opportunity(analyzed, tables, boundary)
            || (boundary.participant < tables.participants.len()
                && analyzed.analysis.line_units.permits_edge(boundary.atom)
                && analyzed
                    .analysis
                    .infos
                    .get(boundary.atom)
                    .is_some_and(|info| {
                        matches!(info.0.boundary, Boundary::Line | Boundary::Mandatory)
                    })
                && wrapping_allowed(analyzed, boundary)?)
        {
            Some(PreparedBreakKind::Allowed)
        } else {
            None
        };
        let attached = soft_boundary(tables, analyzed, index)?;
        let raw_conditional = analyzed.paragraph.base_direction.is_none()
            && kind == Some(PreparedBreakKind::Allowed)
            && control.is_none()
            && !standalone_atomic_opportunity(analyzed, tables, boundary)
            && analyzed
                .analysis
                .infos
                .get(boundary.atom)
                .is_some_and(|(info, style)| {
                    !info.has_ordinary_line_candidate()
                        && analyzed.styles[*style]
                            .style
                            .authored()
                            .effective_breaking()
                            .auto_phrase
                });
        if raw_conditional {
            tables.raw_conditional.push(attached);
        }
        let suppressed = if kind == Some(PreparedBreakKind::Allowed) && !raw_conditional {
            suppressed_phrase(analyzed, tables, boundary, control)?
        } else {
            None
        };
        let mut visible_alternatives = Vec::new();
        for mut row in alternatives {
            row.boundary = attached;
            let ordinary = !analyzed.styles[row.style]
                .style
                .authored()
                .effective_breaking()
                .auto_phrase;
            let alternative = tables.alternatives.len();
            tables.alternatives.push(row);
            if ordinary {
                visible_alternatives.push(alternative);
            }
        }
        if let Some(phrase) = suppressed {
            tables.suppressed.push(SuppressedOpportunityData {
                boundary: attached,
                phrase,
            });
        }
        let kind = kind
            .filter(|kind| {
                *kind != PreparedBreakKind::Allowed || (suppressed.is_none() && !raw_conditional)
            })
            .filter(|kind| {
                *kind != PreparedBreakKind::Discretionary || !visible_alternatives.is_empty()
            })
            .or_else(|| {
                (!visible_alternatives.is_empty()).then_some(PreparedBreakKind::Discretionary)
            });
        if let Some(kind) = kind {
            let attached = if matches!(
                kind,
                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary
            ) {
                attached
            } else {
                index
            };
            let duplicate = (kind == PreparedBreakKind::Allowed)
                .then(|| {
                    tables
                        .opportunities
                        .iter()
                        .position(|row| row.kind == kind && row.boundary == attached)
                })
                .flatten();
            let opportunity = duplicate.unwrap_or(tables.opportunities.len());
            if let Some(cut) = cut {
                tables.hyphenation_cuts[cut].opportunity = Some(opportunity);
                tables.hyphenation_cuts[cut].boundary = attached;
            }
            for &alternative in &visible_alternatives {
                tables.alternatives[alternative].opportunity = Some(opportunity);
                tables.ordinary_alternatives.push(alternative);
            }
            if let Some(existing) = duplicate {
                tables.opportunities[existing]
                    .alternatives
                    .extend(visible_alternatives);
            } else {
                tables.opportunities.push(OpportunityData {
                    boundary: attached,
                    kind,
                    alternatives: visible_alternatives,
                });
            }
        } else if let Some(cut) = cut {
            tables.hyphenation_cuts[cut].boundary = attached;
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

fn suppressed_phrase(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
    control: Option<&ParagraphControlKind>,
) -> crate::Result<Option<usize>> {
    use crate::engine::analysis::{LineCause, wrapping::Influence};
    if control.is_some() || atomic_permission(analyzed, boundary)?.is_some() {
        return Ok(None);
    }
    let relation = &analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]];
    if let (Some(Influence::Text { atom: before, .. }), Some(Influence::Text { atom: after, .. })) =
        (relation.before, relation.after)
        && [before, after].into_iter().any(|atom| {
            analyzed.styles[analyzed.analysis.infos[atom].1]
                .style
                .authored()
                .line_break
                == crate::LineBreak::Anywhere
        })
    {
        return Ok(None);
    }
    let Some(phrase) = analyzed
        .analysis
        .phrases
        .phrase_containing_cut(boundary.atom)
    else {
        return Ok(None);
    };
    let candidates = &analyzed.analysis.line_candidates;
    let first = candidates.partition_point(|row| row.atom < boundary.atom);
    let mut suppressible = false;
    for row in candidates[first..]
        .iter()
        .take_while(|row| row.atom == boundary.atom)
    {
        if matches!(
            row.cause,
            LineCause::RequiredLineBreak | LineCause::SourceZwsp { .. }
        ) {
            return Ok(None);
        }
        let owner = match row.cause {
            LineCause::Whitespace { atom } if disappearing_space_wrap(analyzed, atom).is_some() => {
                analyzed.analysis.infos[atom].1
            }
            _ => row.style,
        };
        if !analyzed.styles[owner]
            .style
            .authored()
            .effective_breaking()
            .auto_phrase
        {
            return Ok(None);
        }
        suppressible = true;
    }
    // Prepared BreakSpaces/disappearing-SPACE can supply an independent cause.
    if (boundary.collapsed_space_wrap.is_some()
        || break_spaces_opportunity(analyzed, tables, boundary))
        && let Some(atom) = boundary.atom.checked_sub(1)
    {
        if !analyzed.styles[analyzed.analysis.infos[atom].1]
            .style
            .authored()
            .effective_breaking()
            .auto_phrase
        {
            return Ok(None);
        }
        suppressible = true;
    }
    Ok(suppressible.then_some(phrase))
}

/// Attach authentic endpoint sequences through source-free neutral anchors.
/// Another influencing owner, original occurrence, domain seam, explicit break
/// or atomic stops the walk; neutral anchors alone never move an opportunity.
fn soft_boundary(
    tables: &PreparedTables,
    analyzed: &AnalyzedData,
    index: usize,
) -> crate::Result<usize> {
    if analyzed.paragraph.scope_topology.is_none() {
        return Ok(index);
    }
    let mut participant = tables.boundaries[index].participant;
    let owner = analyzed.logical.cuts[participant];
    let seams = if analyzed
        .paragraph
        .connections
        .iter()
        .any(|row| row.kind == crate::ParagraphConnectionKind::Disconnected)
    {
        crate::transform::separators::connection_seams(
            &analyzed.projection,
            &analyzed.logical.events,
            &analyzed.paragraph,
        )?
    } else {
        Vec::new()
    };
    let mut lower = 0;
    let mut upper = tables.participants.len();
    for seam in seams {
        if seam.upper <= participant {
            lower = lower.max(seam.upper);
        }
        if seam.lower >= participant {
            upper = upper.min(seam.lower);
        }
        if seam.lower < participant && participant < seam.upper {
            lower = participant;
            upper = participant;
        }
    }
    let endpoint = |at: usize, kind: ParagraphControlKind| {
        let ParticipantData::Control(control) = tables.participants[at] else {
            return false;
        };
        let occurrence = tables.controls[control].occurrence;
        analyzed.paragraph.controls[occurrence].kind == kind
            && captured_endpoint(analyzed, occurrence)
    };
    let neutral = |at: usize| match tables.participants[at] {
        ParticipantData::Control(control) => analyzed.paragraph.controls
            [tables.controls[control].occurrence]
            .kind
            .code_point()
            .is_some(),
        ParticipantData::Text(unit) => tables.units[unit].empty_origin.is_some(),
        ParticipantData::Atomic(_) | ParticipantData::ComputedLineBoundary(_) => false,
    };
    let mut scan = participant;
    while scan > lower && analyzed.logical.cuts[scan - 1] == owner {
        scan -= 1;
        if endpoint(scan, ParagraphControlKind::InlineBoundaryStart) {
            participant = scan;
        } else if !neutral(scan) {
            break;
        }
    }
    scan = participant;
    while scan < upper && analyzed.logical.cuts[scan + 1] == owner {
        if endpoint(scan, ParagraphControlKind::InlineBoundaryEnd) {
            participant = scan + 1;
        } else if !neutral(scan) {
            break;
        }
        scan += 1;
    }
    Ok(tables
        .boundaries
        .iter()
        .position(|row| row.participant == participant)
        .unwrap_or(index))
}

fn captured_endpoint(analyzed: &AnalyzedData, occurrence: usize) -> bool {
    let Some(topology) = &analyzed.paragraph.scope_topology else {
        return false;
    };
    let crate::SourceProcessingEvent::Control { control, .. } =
        topology.capture.events[topology.control_orders[occurrence]].event
    else {
        return false;
    };
    topology.capture.scopes.iter().any(|scope| {
        scope
            .boundaries
            .is_some_and(|boundaries| boundaries.start == control || boundaries.end == control)
    })
}

/// BreakSpaces adds an offer immediately after an actual separator only at an
/// existing typography-safe cut. A suppressed interior cut is never relocated
/// past combining continuations. Unicode glue/joiner protections remain active.
fn break_spaces_opportunity(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
) -> bool {
    use icu_properties::{
        CodePointMapData,
        props::{GeneralCategory, LineBreak},
    };
    if !analyzed.analysis.line_units.permits_edge(boundary.atom)
        || analyzed.paragraph.base_direction.is_none()
    {
        return false;
    }
    let Some(ParticipantData::Text(unit)) = boundary
        .participant
        .checked_sub(1)
        .and_then(|index| tables.participants.get(index))
    else {
        return false;
    };
    if tables.units[*unit].atoms.is_empty() || tables.units[*unit].atoms.end != boundary.atom {
        return false;
    }
    let separator = |cp| {
        cp == 0x09
            || cp != 0xa0
                && CodePointMapData::<GeneralCategory>::new().get32(cp)
                    == GeneralCategory::SpaceSeparator
    };
    let previous = boundary.atom - 1;
    if !separator(analyzed.projection.atoms[previous].code_point)
        || analyzed.styles[analyzed.analysis.infos[previous].1]
            .style
            .authored()
            .white_space
            != crate::WhiteSpace::BreakSpaces
    {
        return false;
    }
    let classes = CodePointMapData::<LineBreak>::new();
    let previous = analyzed.projection.atoms[previous].code_point;
    if matches!(
        classes.get32(previous),
        LineBreak::Glue | LineBreak::WordJoiner | LineBreak::ZWJ
    ) {
        return false;
    }
    !analyzed
        .projection
        .atoms
        .get(boundary.atom)
        .is_some_and(|atom| {
            matches!(
                classes.get32(atom.code_point),
                LineBreak::WordJoiner | LineBreak::ZWJ // Use the adopted UAX boundary for a following GL. Its revision55
                                                       // LB12a exceptions include SP/BA/HY/HH, so TAB before NBSP remains
                                                       // legal; a blanket following-GL veto would erase that permission.
            ) || classes.get32(atom.code_point) == LineBreak::Glue
                && analyzed.analysis.infos[boundary.atom].0.boundary < Boundary::Line
        })
}

fn automatic_cut(
    analyzed: &AnalyzedData,
    tables: &mut PreparedTables,
    boundary_index: usize,
) -> crate::Result<Option<(usize, usize, usize)>> {
    let boundary = tables.boundaries[boundary_index];
    if !analyzed.analysis.line_units.permits_edge(boundary.atom) {
        return Ok(None);
    }
    let Some(word_index) = analyzed.words.word_at(boundary.atom) else {
        return Ok(None);
    };
    let lexical = &analyzed.words.rows()[word_index];
    if boundary.atom == lexical.atoms.end {
        return Ok(None);
    }
    let word = &tables.words[word_index];
    let endpoint = boundary.atom - lexical.atoms.start;
    let Some(&(byte, _)) = word.endpoints.get(endpoint) else {
        return Ok(None);
    };
    let Ok(proposal) = word.proposals.binary_search_by_key(&byte, |(at, _)| *at) else {
        return Ok(None);
    };
    let provenance = word.proposals[proposal].1;
    if endpoint
        .checked_sub(1)
        .is_some_and(|i| word.endpoints[i].0 == byte)
        || word
            .endpoints
            .get(endpoint + 1)
            .is_some_and(|value| value.0 == byte)
    {
        return Ok(None);
    }
    let preceding = lexical
        .units
        .get(
            lexical
                .units
                .partition_point(|unit| unit.end < boundary.atom),
        )
        .filter(|unit| unit.end == boundary.atom)
        .cloned();
    let Some(preceding) = preceding else {
        return Ok(None);
    };
    let atom = preceding.end - 1;
    let style = analyzed.analysis.infos[atom].1;
    let source = analyzed.projection.atoms[atom].source;
    if analyzed.transform.original_row_for_atom(atom).is_none() {
        return Ok(None);
    }
    let origin = analyzed
        .paragraph
        .origin_intersections(&analyzed.processed, source)
        .filter(|row| !row.aggregate.is_empty())
        .last()
        .ok_or_else(|| crate::hyphenation::error(crate::HyphenationError::InvalidResourceMapping))?
        .origin;
    let counts = analyzed.words.at(boundary.atom);
    let index = tables.hyphenation_cuts.len();
    tables.hyphenation_cuts.push(super::hyphenation::CutData {
        word: Some(word_index),
        boundary: boundary_index,
        provenance,
        style,
        preceding: Some(preceding),
        counts,
        opportunity: None,
    });
    let authored = analyzed.styles[style].style.authored();
    let count_admitted = tables.profile == crate::engine::PreparationProfile::Standalone
        || counts.is_some_and(|value| value.admits(authored.hyphenate_limit_chars));
    if !count_admitted
        || authored.hyphens != crate::Hyphens::Auto
        || authored.effective_breaking().word == OrdinaryWordBreak::BreakAll
        || authored.line_break == crate::LineBreak::Anywhere
        || authored.wrap != crate::Wrap::Word
        || analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]]
            .before_hard_separator
    {
        return Ok(None);
    }
    if analyzed.paragraph.base_direction.is_some() && !native_permission(analyzed, &boundary)? {
        return Ok(None);
    }
    Ok(Some((index, style, origin)))
}

/// This exact source SHY ending can independently lend an ordinary BreakAll
/// cut. Its following text owns the rule; the SHY's indication/local wrap
/// eligibility neither grants this permission nor transfers it to later anchors.
fn ordinary_after_source_shy(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
) -> crate::Result<bool> {
    if !analyzed.analysis.line_units.permits_edge(boundary.atom) {
        return Ok(false);
    }
    let Some(ParticipantData::Text(unit)) = boundary
        .participant
        .checked_sub(1)
        .and_then(|previous| tables.participants.get(previous))
    else {
        return Ok(false);
    };
    let unit = &tables.units[*unit];
    if unit.deleted
        || unit.atoms.len() != 1
        || unit.atoms.end != boundary.atom
        || analyzed.projection.atoms[unit.atoms.start].code_point != 0xad
    {
        return Ok(false);
    }
    let relation = &analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]];
    if !matches!(relation.after,
        Some(crate::engine::analysis::wrapping::Influence::Text { atom, .. }) if atom == boundary.atom)
    {
        return Ok(false);
    }
    let Some(info) = analyzed.analysis.infos.get(boundary.atom) else {
        return Ok(false);
    };
    let following = analyzed.styles[info.1].style.authored();
    if following.effective_breaking().word != OrdinaryWordBreak::BreakAll
        || info.0.boundary < Boundary::Line
    {
        return Ok(false);
    }
    if analyzed.paragraph.base_direction.is_some() {
        native_permission(analyzed, boundary)
    } else {
        Ok(following.wrap == crate::Wrap::Word)
    }
}

fn used_placement(
    style: &crate::Style,
    resolution: &IndicationResolution,
) -> crate::HyphenationPlacement {
    if matches!(resolution, IndicationResolution::ResolvedNative) {
        crate::HyphenationPlacement::LogicalBefore
    } else {
        style
            .native_hyphenation_convention
            .as_ref()
            .map_or(crate::HyphenationPlacement::LogicalBefore, |value| {
                value.placement()
            })
    }
}

fn indication(
    catalogue: &mut crate::catalogue::Catalogue,
    styles: &[ResolvedStyleRun],
    authored: &crate::Style,
    style: usize,
    supplied: Option<&super::paragraph::ParagraphReplacement>,
) -> crate::Result<(Vec<crate::Text>, IndicationResolution)> {
    let result = if let Some(super::paragraph::ReplacementPresentation::ResolvedNative(texts)) =
        supplied.map(|row| &row.presentation)
    {
        (texts.clone(), IndicationResolution::ResolvedNative)
    } else {
        let (text, resolution) = match &authored.hyphenate_character {
            crate::HyphenateCharacter::String(text) => {
                (text.clone(), IndicationResolution::Specified)
            }
            crate::HyphenateCharacter::Auto => match &authored.native_hyphenation_convention {
                Some(convention) => (
                    convention.automatic_text().clone(),
                    IndicationResolution::Convention,
                ),
                None => {
                    let identity = catalogue.identity().clone();
                    let (text, primary) = crate::engine::shape::automatic_hyphen(
                        catalogue.query(),
                        &styles[style].style,
                        &identity,
                    )?;
                    (text, IndicationResolution::Primary(primary))
                }
            },
        };
        (vec![text], resolution)
    };

    Ok(result)
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

/// Authenticates actual text-pair proposals: either-side Anywhere or a qualified
/// following-owned missing-lexical assumption. Their causes remain distinct.
/// Controls, atomics and transparent metadata never lend their captured Style.
fn ordinary_text_opportunity(
    analyzed: &AnalyzedData,
    boundary: &BoundaryData,
) -> crate::Result<bool> {
    use crate::engine::analysis::wrapping::Influence;
    if !analyzed.analysis.line_units.permits_edge(boundary.atom) {
        return Ok(false);
    }
    let relation = &analyzed.logical.relations[analyzed.logical.cuts[boundary.participant]];
    let (
        Some(Influence::Text {
            participant,
            atom: before,
        }),
        Some(Influence::Text { atom: after, .. }),
    ) = (relation.before, relation.after)
    else {
        return Ok(false);
    };
    // One real defining cut; later source-free custody does not clone it.
    if boundary.participant != participant + 1
        || boundary.atom != after
        || is_hard_separator(analyzed.projection.atoms[before].code_point)
        || is_hard_separator(analyzed.projection.atoms[after].code_point)
        || analyzed
            .projection
            .barrier_atoms
            .iter()
            .any(|at| *at > before && *at <= after)
    {
        return Ok(false);
    }
    let anywhere = [before, after].into_iter().any(|atom| {
        analyzed.styles[analyzed.analysis.infos[atom].1]
            .style
            .authored()
            .line_break
            == crate::LineBreak::Anywhere
    });
    let lexical_fallback = analyzed.analysis.infos[after].0.lexical_fallback_before();
    if !anywhere && !lexical_fallback {
        return Ok(false);
    }
    if analyzed.paragraph.base_direction.is_some() {
        native_permission(analyzed, boundary)
    } else {
        wrapping_allowed(analyzed, boundary)
    }
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
    let local_space_permits = match (relation.before, relation.after) {
        (
            Some(crate::engine::analysis::wrapping::Influence::Text { .. }),
            Some(crate::engine::analysis::wrapping::Influence::Atomic { .. }),
        ) => boundary
            .collapsed_space_wrap
            .map_or(relation.space_permits, |wrap| wrap == crate::Wrap::Word),
        _ => relation.space_permits,
    };
    if local_space_permits {
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
        // AutoPhrase moves the raw profile's genuine emergency cuts out of
        // ordinary inventory and into the same authenticated final tier.
        let first = tables
            .raw_conditional
            .partition_point(|index| *index <= start);
        let last = tables
            .raw_conditional
            .partition_point(|index| *index < ceiling);
        return Ok(tables.raw_conditional[first.min(last)..last]
            .iter()
            .copied()
            .filter(|index| {
                if *index <= start
                    || *index >= ceiling
                    || !analyzed
                        .analysis
                        .line_units
                        .permits_edge(tables.boundaries[*index].atom)
                {
                    return false;
                }
                let atom = tables.boundaries[*index].atom;
                let style = analyzed.styles[analyzed.analysis.infos[atom].1]
                    .style
                    .authored();
                matches!(
                    (style.effective_breaking().overflow, purpose),
                    (crate::OverflowWrap::Anywhere, _)
                        | (
                            crate::OverflowWrap::BreakWord,
                            super::ConditionalPurpose::Overflow
                        )
                )
            })
            .collect());
    }
    let mut result = Vec::new();
    for index in start + 1..ceiling {
        if soft_boundary(tables, analyzed, index)? != index {
            continue;
        }
        let boundary = &tables.boundaries[index];
        if !analyzed.analysis.line_units.permits_edge(boundary.atom) {
            continue;
        }
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
            (style.effective_breaking().overflow, purpose),
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

/// The raw source model's actual atomic sides remain independent of a
/// neighboring SHY. This retains its characterized rule, without lending
/// native ancestor permission from incidental participant adjacency.
fn standalone_atomic_opportunity(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
) -> bool {
    if !analyzed.analysis.line_units.permits_edge(boundary.atom)
        || analyzed.paragraph.base_direction.is_some()
    {
        return false;
    }
    let previous = boundary
        .participant
        .checked_sub(1)
        .and_then(|i| tables.participants.get(i));
    matches!(previous, Some(ParticipantData::Atomic(_)))
        || matches!(
            tables.participants.get(boundary.participant),
            Some(ParticipantData::Atomic(_))
        )
}

fn standalone_opportunity(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    boundary: &BoundaryData,
) -> bool {
    if !analyzed.analysis.line_units.permits_edge(boundary.atom)
        || analyzed.paragraph.base_direction.is_some()
    {
        return false;
    }
    let previous = boundary
        .participant
        .checked_sub(1)
        .and_then(|i| tables.participants.get(i));
    let next = tables.participants.get(boundary.participant);
    if previous.is_some_and(|row|matches!(*row,ParticipantData::Text(i)
        if tables.units[i].atoms.clone().any(|atom|matches!(analyzed.projection.atoms[atom].code_point,0x20|0xa0)))) {
        return true;
    }
    next.is_some_and(|row| match *row {
        ParticipantData::Text(i) => tables.units[i].atoms.clone().next().is_some_and(|atom| {
            let style = &analyzed.styles[analyzed.analysis.infos[atom].1]
                .style
                .authored();
            style.wrap == crate::Wrap::Word
                && style.effective_breaking().overflow != crate::OverflowWrap::Normal
        }),
        _ => false,
    })
}

fn wrapping_allowed(analyzed: &AnalyzedData, boundary: &BoundaryData) -> crate::Result<bool> {
    if analyzed.paragraph.base_direction.is_some()
        && let Some(wrap) = boundary.collapsed_space_wrap
    {
        return Ok(wrap == crate::Wrap::Word);
    }
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

/// Retain the executed lexical cause rather than treating every deleted source
/// range as a disappeared SPACE or borrowing the preceding processed atom.
fn collapsed_follower_wrap(analyzed: &AnalyzedData, source: Range) -> Option<crate::Wrap> {
    let operations = analyzed.processed.operations();
    let piece =
        operations.get(operations.partition_point(|piece| piece.source.end <= source.start))?;
    if piece.source.start > source.start
        || source.end > piece.source.end
        || piece.kind
            != crate::processing::MappingKind::Delete(
                crate::processing::DeletionCause::CollapsedSpace,
            )
    {
        return None;
    }
    let at = source.end.checked_sub(1)?;
    Some(
        analyzed.styles[crate::engine::resolve::source_style(&analyzed.styles, at)]
            .style
            .authored()
            .wrap,
    )
}

/// The containing style owns a native SPACE opportunity only when that SPACE
/// actually disappears at the selected break. Preserved SPACE boundaries use
/// captured NCA permission instead.
fn disappearing_space_wrap(analyzed: &AnalyzedData, atom: usize) -> Option<crate::Wrap> {
    let style = analyzed.styles[analyzed.analysis.infos[atom].1]
        .style
        .authored();
    (analyzed.projection.atoms[atom].code_point == 0x20 && style.white_space.collapses_spaces())
        .then_some(style.wrap)
}

/// Performs actual alternative font matching while the original
/// catalogue capture is still owned by preparation. No finalizer can requery it.
pub(crate) fn prepare_indication_fonts(
    catalogue: &mut crate::catalogue::Catalogue,
    analyzed: &std::sync::Arc<AnalyzedData>,
    styles: &[ResolvedStyleRun],
    tables: &PreparedTables,
    scratch: &mut crate::engine::shape::ShapeContext,
    options: crate::PreparationOptions,
) -> crate::Result<Vec<PreparedIndicationFonts>> {
    let css_tabs = options.tabs() == crate::TabPreparation::CssStops;
    let orientation = options.orientation();
    let baseline_reference = options.resolved_baseline_reference();
    let mut alternatives = Vec::with_capacity(tables.alternatives.len());
    for (alternative, row) in tables.alternatives.iter().enumerate() {
        if let IndicationResolution::Primary(font) = &row.resolution
            && catalogue.identity() != font.catalogue_identity()
        {
            return Err(crate::Error::new(
                crate::ErrorCode::FontCatalogueMismatch,
                "indication primary belongs to a different preparation catalogue",
            )
            .with_detail(crate::ErrorDetail::FontCatalogueMismatch {
                expected: catalogue.identity().clone(),
                actual: font.catalogue_identity().clone(),
            }));
        }
        let input = row;
        let ReplacementProjection {
            projection,
            inserted_atoms,
            original_atoms,
        } = replacement_projection(analyzed, tables, input)?;
        let (mut analysis, _, _) = crate::engine::analysis::analyze(
            &projection,
            &analyzed.styles,
            analyzed.processed.source().boxes(),
            &analyzed.paragraph,
            analyzed.typography,
        )?;
        for (atom, original) in original_atoms.iter().enumerate() {
            if let Some(original) = *original {
                analysis.levels[atom] = analyzed.analysis.levels[original];
            }
        }
        let base = presentation::before_attachment(analyzed, tables, row)?
            .original_level(analyzed, tables)?;
        presentation::local_levels(
            &projection,
            &mut analysis,
            inserted_atoms.clone(),
            base & 1,
            analyzed.typography,
        )?;
        // Capture only the genuine selected side. Unselected original text
        // cannot change an indication's query EGC or script inheritance.
        let mut capture_side = |after: bool| -> crate::Result<PreparedIndicationFonts> {
            let side = if after {
                inserted_atoms.start..projection.atoms.len()
            } else {
                0..inserted_atoms.end
            };
            let offset = side.start;
            let mut atoms = projection.atoms[side.clone()].to_vec();
            let mut position = 0usize;
            for atom in &mut atoms {
                let end = position
                    .checked_add(atom.presentation.len())
                    .ok_or_else(crate::projection::length_overflow)?;
                atom.presentation = Range::new(position, end);
                position = end;
            }
            let projection =
                SourceProjection::from_atoms(atoms, position, projection.barriers.clone())?;
            let graphemes = projection
                .graphemes
                .windows(2)
                .filter_map(|p| {
                    let a = projection
                        .atoms
                        .partition_point(|a| a.presentation.start < p[0]);
                    let b = projection
                        .atoms
                        .partition_point(|a| a.presentation.start < p[1]);
                    (a < b).then_some(a..b)
                })
                .collect();
            let mut analysis = Analysis {
                line_candidates: Vec::new(),
                phrases: crate::engine::analysis::phrase::PhraseCatalog::default(),
                infos: analysis.infos[side.clone()].to_vec(),
                levels: analysis.levels[side.clone()].to_vec(),
                base_level: analysis.base_level,
                box_levels: analysis.box_levels.clone(),
                control_levels: analysis.control_levels.clone(),
                computed_levels: analysis.computed_levels.clone(),
                graphemes,
                line_units: crate::engine::analysis::line_units::LineBreakUnits::new(&projection)?,
            };
            let inserted_atoms = inserted_atoms.start - offset..inserted_atoms.end - offset;
            let catalogue_identity = catalogue.identity().clone();
            let provenance = crate::engine::upright::UprightProvenance::Alternative {
                original: &analyzed.projection,
                originals: &original_atoms[side.clone()],
                inserted: inserted_atoms.clone(),
                alternative,
                insertion_anchor: row.insertion_anchor,
            };
            let upright_units = crate::engine::upright::units(
                analyzed,
                &projection,
                &analysis,
                &provenance,
                orientation,
            )?;
            let shaping_profile = || match orientation {
                crate::LineOrientation::Horizontal => {
                    crate::engine::upright::ShapingProfile::Horizontal
                }
                crate::LineOrientation::SidewaysClockwise => {
                    crate::engine::upright::ShapingProfile::SidewaysClockwise
                }
                crate::LineOrientation::SidewaysCounterclockwise => {
                    crate::engine::upright::ShapingProfile::SidewaysCounterclockwise
                }
                crate::LineOrientation::Vertical
                    if analyzed.typography == crate::AnalysisTypography::Mixed =>
                {
                    crate::engine::upright::ShapingProfile::Mixed {
                        owners: &analyzed.styles,
                        units: &upright_units,
                        provenance: crate::engine::upright::UprightProvenance::Alternative {
                            original: &analyzed.projection,
                            originals: &original_atoms[side.clone()],
                            inserted: inserted_atoms.clone(),
                            alternative,
                            insertion_anchor: row.insertion_anchor,
                        },
                    }
                }
                crate::LineOrientation::Vertical => {
                    crate::engine::upright::ShapingProfile::Upright {
                        units: &upright_units,
                        provenance: crate::engine::upright::UprightProvenance::Alternative {
                            original: &analyzed.projection,
                            originals: &original_atoms[side.clone()],
                            inserted: inserted_atoms.clone(),
                            alternative,
                            insertion_anchor: row.insertion_anchor,
                        },
                    }
                }
            };
            let mut font_queries = crate::engine::shape::capture_fonts(
                catalogue,
                crate::engine::ShapingInput {
                    formatting_breaks: &[],
                    scope_custody: None,
                    characters: None,
                    separator_geometry: false,
                    profile: shaping_profile(),
                    baseline_reference,
                    projection: &projection,
                    analysis: &analysis,
                    styles,
                    boxes: analyzed.processed.source().boxes(),
                    font_catalogue: &catalogue_identity,
                    paragraph: &analyzed.paragraph,
                    css_tabs,
                },
                scratch,
            )?;
            if after && !inserted_atoms.is_empty() {
                presentation::local_levels(
                    &projection,
                    &mut analysis,
                    inserted_atoms.clone(),
                    (base & 1) ^ 1,
                    analyzed.typography,
                )?;
                font_queries.extend(crate::engine::shape::capture_fonts(
                    catalogue,
                    crate::engine::ShapingInput {
                        formatting_breaks: &[],
                        scope_custody: None,
                        characters: None,
                        separator_geometry: false,
                        profile: shaping_profile(),
                        baseline_reference,
                        projection: &projection,
                        analysis: &analysis,
                        styles,
                        boxes: analyzed.processed.source().boxes(),
                        font_catalogue: &catalogue_identity,
                        paragraph: &analyzed.paragraph,
                        css_tabs,
                    },
                    scratch,
                )?);
                presentation::local_levels(
                    &projection,
                    &mut analysis,
                    inserted_atoms.clone(),
                    base & 1,
                    analyzed.typography,
                )?;
            }
            Ok(PreparedIndicationFonts {
                inserted_atoms,
                projection,
                analysis,
                font_queries,
            })
        };
        let mut captured = capture_side(!row.placement.includes_before())?;
        if row.placement == crate::HyphenationPlacement::BothSides && !row.text.is_empty() {
            // One exact chosen Text supplies both emissions. Only its raw
            // scalar/style metadata is shared; each genuine seam executes its
            // own full matching profiles, with both possible receiving parities.
            captured
                .font_queries
                .extend(capture_side(true)?.font_queries);
        }
        alternatives.push(captured);
    }
    Ok(alternatives)
}

struct ReplacementProjection {
    projection: SourceProjection,
    inserted_atoms: IndexRange<usize>,
    original_atoms: Vec<Option<usize>>,
}

fn replacement_projection(
    analyzed: &AnalyzedData,
    tables: &PreparedTables,
    replacement: &AlternativeData,
) -> crate::Result<ReplacementProjection> {
    let original = &analyzed.projection;
    let start = if let AlternativeDefinition::Automatic { atom, .. } = replacement.definition {
        atom
    } else if let Some(control) = replacement.control() {
        let anchor = analyzed
            .paragraph
            .anchor_order
            .iter()
            .position(|anchor| anchor.item == SemanticAnchorItem::Control(control))
            .expect("validated defining control");
        anchor_atom(
            anchor,
            &analyzed.paragraph,
            analyzed.processed.source().boxes(),
            original,
        )
    } else {
        // A source-defined alternative replaces its genuine retained SHY row.
        // Reordered or overlapping transform emissions are not source-sorted.
        original
            .atoms
            .iter()
            .position(|atom| {
                atom.source == replacement.replaced_source() && atom.code_point == 0x00ad
            })
            .expect("issued source alternative retains its original SHY atom")
    };
    let end = if replacement.replaced_source().is_empty() {
        start
    } else {
        start + 1
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
    let project_atom = |atom: usize| {
        if atom <= start {
            atom
        } else if atom >= end {
            inserted.end + (atom - end)
        } else {
            inserted.start
        }
    };
    let barrier_atoms = original
        .barrier_atoms
        .iter()
        .copied()
        .map(project_atom)
        .collect();
    let mut projection = SourceProjection::from_atoms_at_barriers(
        atoms,
        presentation,
        original.barriers.clone(),
        barrier_atoms,
    )?;
    projection.style_rows = vec![None; projection.atoms.len()];
    for (atom, slot) in projection.style_rows.iter_mut().enumerate() {
        if inserted.contains(&atom) {
            *slot = Some(replacement.style);
        } else {
            let original_atom = if atom < inserted.start {
                atom
            } else {
                end + atom - inserted.end
            };
            *slot = original.style_rows.get(original_atom).copied().flatten();
        }
    }
    projection.anchor_atoms = original
        .anchor_atoms
        .iter()
        .map(|atom| atom.map(project_atom))
        .collect();
    if let Some(control) = replacement.control() {
        let defining = analyzed
            .paragraph
            .anchor_order
            .iter()
            .position(|anchor| anchor.item == SemanticAnchorItem::Control(control))
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
    if let AlternativeDefinition::Automatic { cut, .. } = replacement.definition {
        let boundary = tables.boundaries[tables.hyphenation_cuts[cut].boundary];
        for (order, anchor) in analyzed.paragraph.anchor_order.iter().enumerate() {
            if let Some((participant, _)) = analyzed.logical.events.iter().enumerate().find(|(_, event)| {
                event.atom == start && matches!(event.kind, crate::engine::analysis::wrapping::Event::Anchor(actual) if actual == *anchor)
            }) {
                projection.anchor_atoms[order] = Some(if participant < boundary.participant { inserted.start } else { inserted.end });
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
    })
}

/// Constructs the one selected semantic overlay after capability admission.
/// Original participant ordinals are selected-local; replacement-only units
/// have no fabricated original participant. The captured alternative supplies
/// both original-context analysis and the actual retained font assignments.
pub(crate) fn select_presentation(
    prepared: &crate::engine::PreparedData,
    request: super::input::RequestInput<'_>,
) -> crate::Result<SelectedPresentation> {
    presentation::select(prepared, request)
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
        line_level: level,
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
            PresentationUnitKind::ComputedLineBoundary { boundary } => {
                SelectedMappingKind::ComputedLineBoundary { boundary }
            }
            PresentationUnitKind::GeneratedSeparator { owner } => {
                SelectedMappingKind::GeneratedSeparator { owner }
            }
            PresentationUnitKind::Replacement { alternative, .. } => {
                SelectedMappingKind::Replacement {
                    alternative,
                    anchor: source.start,
                }
            }
            _ => SelectedMappingKind::Source,
        },
    });
    Ok(())
}
