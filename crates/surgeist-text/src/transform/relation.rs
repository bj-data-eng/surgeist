//! Immutable correspondence between actual transform input, emitted atoms and
//! ordered mixed events. Removed input is correspondence, never a participant.

use std::ops::Range as IndexRange;

use crate::engine::analysis::wrapping::{Event, LogicalEvent};
use crate::processing::lexical_inventory::LexicalDeletion;
use crate::protocol::paragraph::{FlowOriginKey, ParagraphData, SemanticAnchorItem};
use crate::{Range, Source};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OriginalPiece {
    pub(crate) origin: FlowOriginKey,
    pub(crate) aggregate: Range,
    pub(crate) local: Range,
}

#[derive(Clone, Debug)]
pub(crate) struct InputRow {
    pub(crate) source: Range,
    pub(crate) code_point: u32,
    /// Actual separator-produced SPACE, eligible for width independently of Phase I.
    pub(crate) separator_space: bool,
    pub(crate) style: usize,
    pub(crate) pieces: IndexRange<usize>,
    /// Actual output IDs in flat input-row storage, independent of event order.
    pub(crate) outputs: IndexRange<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentOwner {
    Original(usize),
    Wbr(usize),
    Virtual(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeneratedSeparatorOwner {
    Control(usize),
    Virtual(usize),
}

#[derive(Clone, Debug)]
pub(crate) struct ScopeData {
    pub(crate) ordinal: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct VirtualSeparatorData {
    /// Actual insertion cut in the canonical captured event stream.
    pub(crate) capture_cut: usize,
    pub(crate) scope: usize,
    pub(crate) source: usize,
    pub(crate) processed: usize,
    /// Number of actual paragraph controls before this semantic seam.
    pub(crate) control_cut: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Component {
    pub(crate) owner: ComponentOwner,
    pub(crate) source: Range,
    pub(crate) processed: Range,
    pub(crate) style: usize,
    pub(crate) outputs: IndexRange<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Emission {
    pub(crate) component: usize,
    pub(crate) subposition: usize,
    pub(crate) code_point: u32,
    /// Actual post-case/MathAuto non-width identity of this emission. Ordinary
    /// presentation remains independently width-then-kana transformed.
    pub(crate) selection_identity: u32,
    pub(crate) atom: usize,
    pub(crate) event: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RelationEvent {
    Text {
        atom: usize,
    },
    Removal {
        row: usize,
    },
    PhaseIDeleted {
        source: Range,
        processed: usize,
        lexical: LexicalDeletion,
    },
    Anchor {
        order: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TransformMap {
    pub(crate) phrases: Vec<super::separators::automatic::PhraseAnalysis>,
    pub(crate) scopes: Vec<ScopeData>,
    pub(crate) virtuals: Vec<VirtualSeparatorData>,
    pub(crate) rows: Vec<InputRow>,
    pub(crate) pieces: Vec<OriginalPiece>,
    pub(crate) components: Vec<Component>,
    pub(crate) emissions: Vec<Emission>,
    pub(crate) events: Vec<RelationEvent>,
    /// Base semantic atom -> emission ID; never an original-source search.
    pub(crate) atoms: Vec<usize>,
    /// Canonical cut after each real presentation participant, with only the
    /// consecutive removals in that exact local mixed seam consumed.
    pub(crate) participant_cuts: Vec<usize>,
}

impl TransformMap {
    /// Immutable original incidence only. No component is considered mapped
    /// until the request transaction records its actual result.
    pub(super) fn originals(
        projection: &crate::projection::SourceProjection,
        styles: &[super::context::StyleRow<'_>],
        paragraph: &super::context::SourceContext<'_>,
    ) -> crate::Result<Self> {
        let mut map = Self::default();
        map.rows
            .try_reserve_exact(projection.atoms.len())
            .map_err(|_| super::context::capacity())?;
        for atom in &projection.atoms {
            let first = map.pieces.len();
            for piece in paragraph.origins.iter().filter_map(|origin| {
                let start = origin.aggregate.start.max(atom.source.start);
                let end = origin.aggregate.end.min(atom.source.end);
                (start < end).then(|| OriginalPiece {
                    origin: origin.key,
                    aggregate: Range::new(start, end),
                    local: Range::new(
                        origin.local.start + (start - origin.aggregate.start),
                        origin.local.start + (end - origin.aggregate.start),
                    ),
                })
            }) {
                super::context::push(&mut map.pieces, piece)?;
            }
            map.rows.push(InputRow {
                source: atom.source,
                code_point: atom.code_point,
                separator_space: false,
                style: super::context::source_style(styles, atom.source.start),
                pieces: first..map.pieces.len(),
                outputs: 0..0,
            });
        }
        Ok(map)
    }

    pub(super) fn finish_cuts(&mut self) -> crate::Result<()> {
        self.participant_cuts.clear();
        let count = self
            .events
            .len()
            .checked_add(1)
            .ok_or_else(super::context::capacity)?;
        self.participant_cuts
            .try_reserve_exact(count)
            .map_err(|_| super::context::capacity())?;
        self.participant_cuts.push(0);
        let mut owns_following_removals = false;
        for (index, event) in self.events.iter().enumerate() {
            match event {
                RelationEvent::Removal { .. } => {
                    if owns_following_removals {
                        *self.participant_cuts.last_mut().expect("initial boundary") = index + 1;
                    }
                }
                _ => {
                    self.participant_cuts.push(index + 1);
                    owns_following_removals = !matches!(event, RelationEvent::Anchor { .. });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn component_for_atom(&self, atom: usize) -> &Component {
        &self.components[self.emissions[self.atoms[atom]].component]
    }
    pub(crate) fn original_row_for_atom(&self, atom: usize) -> Option<usize> {
        match self.component_for_atom(atom).owner {
            ComponentOwner::Original(row) => Some(row),
            ComponentOwner::Wbr(_) | ComponentOwner::Virtual(_) => None,
        }
    }
    pub(crate) fn controls_for_atoms(&self, atoms: IndexRange<usize>) -> Vec<usize> {
        let mut controls = Vec::new();
        for atom in atoms {
            if let ComponentOwner::Wbr(control) = self.component_for_atom(atom).owner
                && !controls.contains(&control)
            {
                controls.push(control);
            }
        }
        controls
    }

    pub(crate) fn virtuals_for_atoms(&self, atoms: IndexRange<usize>) -> Vec<usize> {
        let mut virtuals = Vec::new();
        for atom in atoms {
            if let ComponentOwner::Virtual(index) = self.component_for_atom(atom).owner
                && !virtuals.contains(&index)
            {
                virtuals.push(index);
            }
        }
        virtuals
    }

    pub(crate) fn emission_for_atom(&self, atom: usize) -> Emission {
        self.emissions[self.atoms[atom]]
    }

    pub(crate) fn original_pieces(&self, row: usize) -> &[OriginalPiece] {
        &self.pieces[self.rows[row].pieces.clone()]
    }

    pub(crate) fn events(&self) -> &[RelationEvent] {
        &self.events
    }

    /// Original custody of one authentic mixed relation event. This is not an
    /// ordering key: several emissions can refer to one row, and their semantic
    /// order can differ from the input row order.
    pub(crate) fn event_source(
        &self,
        event: usize,
        paragraph: &ParagraphData,
        source: &Source,
    ) -> Range {
        match self.events[event] {
            RelationEvent::Text { atom } => self.component_for_atom(atom).source,
            RelationEvent::Removal { row } => self.rows[row].source,
            RelationEvent::PhaseIDeleted { source, .. } => source,
            RelationEvent::Anchor { order } => {
                let at = match paragraph.anchor_order[order].item {
                    SemanticAnchorItem::EmptyOrigin(index) => {
                        paragraph.origins[index].aggregate.start
                    }
                    SemanticAnchorItem::Control(index) => paragraph.controls[index].at,
                    SemanticAnchorItem::ComputedLineBoundary(index) => {
                        paragraph.computed_boundaries[index].at
                    }
                    SemanticAnchorItem::Atomic(index) => source.boxes()[index].index(),
                };
                Range::new(at, at)
            }
        }
    }

    pub(crate) fn cut_for_participant(&self, boundary: usize) -> usize {
        self.participant_cuts[boundary]
    }

    pub(crate) fn logical_events(
        &self,
        paragraph: &ParagraphData,
        source: &Source,
    ) -> Vec<LogicalEvent> {
        let mut atom = 0;
        self.events
            .iter()
            .filter_map(|event| {
                let kind = match *event {
                    RelationEvent::Text { atom: index } => {
                        atom = index;
                        Event::Text(index)
                    }
                    RelationEvent::Removal { .. } => return None,
                    RelationEvent::PhaseIDeleted {
                        source,
                        processed,
                        lexical,
                    } => Event::Deleted(source, processed, lexical),
                    RelationEvent::Anchor { order } => Event::Anchor(paragraph.anchor_order[order]),
                };
                let source = match kind {
                    Event::Text(index) => self.component_for_atom(index).source.start,
                    Event::Deleted(source, _, _) => source.start,
                    Event::Anchor(anchor) => match anchor.item {
                        SemanticAnchorItem::EmptyOrigin(index) => {
                            paragraph.origins[index].aggregate.start
                        }
                        SemanticAnchorItem::Control(index) => paragraph.controls[index].at,
                        SemanticAnchorItem::ComputedLineBoundary(index) => {
                            paragraph.computed_boundaries[index].at
                        }
                        SemanticAnchorItem::Atomic(index) => source.boxes()[index].index(),
                    },
                };
                let event = LogicalEvent { atom, source, kind };
                if matches!(kind, Event::Text(_)) {
                    atom += 1;
                }
                Some(event)
            })
            .collect()
    }
}
