//! Lazy canonical scope incidence at executed mixed-event cuts.
//!
//! Original contribution identities, rather than glyph or source-coordinate
//! hulls, carry enclosure. Projected endpoint controls retain their independent
//! mixed-prefix ownership; a snapped marker cannot move a logical scope edge.

use crate::engine::AnalyzedCore;
use crate::protocol::paragraph::{ParagraphControlKind, SemanticAnchorItem};
use crate::transform::relation::{ComponentOwner, RelationEvent};

mod incidence;
pub(crate) use incidence::{IncidenceIndex, Placement};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScopeIndexCapacity;

impl ScopeIndexCapacity {
    pub(crate) fn error(self) -> crate::Error {
        super::capacity()
    }
}

/// `false` is the genuine cut immediately before an event; `true` is its body.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Position(usize, bool);

#[derive(Clone, Copy)]
struct Witness {
    first: Position,
    last: Position,
}

enum EventWitness {
    Known(Witness),
    Transparent,
    Unknown,
}

pub(crate) struct ScopeCutIndex {
    left: Vec<Option<Position>>,
    right: Vec<Option<Position>>,
    pub(crate) incidences: std::sync::OnceLock<Result<IncidenceIndex, ScopeIndexCapacity>>,
}

/// Requested live payload of the actual lazy index buffers, excluding the
/// already allocated capture, analysis, font and preparation objects.
/// Allocation identity prevents counting a retained shared buffer twice.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct IndexPayloadCensus {
    allocations: std::collections::BTreeMap<usize, usize>,
    cores: std::collections::BTreeSet<usize>,
}

#[cfg(test)]
impl IndexPayloadCensus {
    pub(crate) fn shared<T>(&mut self, value: &std::sync::Arc<T>) -> bool {
        let identity = std::sync::Arc::as_ptr(value) as usize;
        let bytes = std::mem::size_of::<T>();
        match self.allocations.entry(identity) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(bytes);
                true
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                assert_eq!(*entry.get(), bytes);
                false
            }
        }
    }

    pub(crate) fn buffer<T>(&mut self, values: &Vec<T>) {
        let bytes = values.capacity() * std::mem::size_of::<T>();
        if bytes != 0 {
            let previous = self.allocations.insert(values.as_ptr() as usize, bytes);
            assert!(previous.is_none_or(|previous| previous == bytes));
        }
    }

    pub(crate) fn analyzed(&mut self, core: &AnalyzedCore) {
        if let Some(topology) = &core.paragraph.scope_topology {
            // Computed admission can initialize capture facts before any
            // local ScopeCutIndex exists. Include that pre-query allocation.
            topology.capture.canonical_payload(self);
        }
        if !self.cores.insert(std::ptr::from_ref(core) as usize) {
            return;
        }
        if let Some(index) = core.scope_cuts.get() {
            index
                .as_ref()
                .expect("successful factual query")
                .payload(self);
        }
    }

    pub(crate) fn bytes(&self) -> usize {
        self.allocations.values().sum()
    }
}

#[cfg(test)]
impl ScopeCutIndex {
    fn payload(&self, census: &mut IndexPayloadCensus) {
        census.buffer(&self.left);
        census.buffer(&self.right);
        if let Some(index) = self.incidences.get() {
            index
                .as_ref()
                .expect("successful incidence query")
                .payload(census);
        }
    }
}

impl ScopeCutIndex {
    pub(crate) fn new(core: &AnalyzedCore) -> Result<Self, ScopeIndexCapacity> {
        let topology = core
            .paragraph
            .scope_topology
            .as_ref()
            .expect("query authenticates capture");
        let paragraph = topology.capture.paragraphs[topology.paragraph];
        let len = core
            .transform
            .events
            .len()
            .checked_add(1)
            .ok_or(ScopeIndexCapacity)?;
        let mut witnesses = Vec::new();
        witnesses
            .try_reserve_exact(len - 1)
            .map_err(|_| ScopeIndexCapacity)?;
        let mut atom_cut = 0;
        for event in &core.transform.events {
            let witness = match *event {
                RelationEvent::Text { atom } => {
                    atom_cut += 1;
                    match core.transform.component_for_atom(atom).owner {
                        ComponentOwner::Original(row) => original(core, row),
                        ComponentOwner::Wbr(control) => known(topology.control_orders[control]),
                        ComponentOwner::Virtual(index) => {
                            let point = Position(core.transform.virtuals[index].capture_cut, false);
                            EventWitness::Known(Witness {
                                first: point,
                                last: point,
                            })
                        }
                    }
                }
                RelationEvent::Removal { row } => original(core, row),
                RelationEvent::PhaseIDeleted { source, .. } => {
                    let mut orders = core
                        .paragraph
                        .origin_intersections(&core.processed, source)
                        .filter(|piece| !piece.aggregate.is_empty())
                        .map(|piece| topology.origin_orders[piece.origin]);
                    witness_orders(&mut orders)
                }
                RelationEvent::Anchor { order } => match core.paragraph.anchor_order[order].item {
                    SemanticAnchorItem::EmptyOrigin(origin) => {
                        known(topology.origin_orders[origin])
                    }
                    SemanticAnchorItem::Atomic(atomic) => {
                        known(topology.capture.atomic_orders[atomic])
                    }
                    SemanticAnchorItem::ComputedLineBoundary(index) => {
                        let point =
                            Position(core.paragraph.computed_boundaries[index].capture_cut, false);
                        EventWitness::Known(Witness {
                            first: point,
                            last: point,
                        })
                    }
                    SemanticAnchorItem::Control(control) => {
                        let endpoint = matches!(
                            core.paragraph.controls[control].kind,
                            ParagraphControlKind::InlineBoundaryStart
                                | ParagraphControlKind::InlineBoundaryEnd
                        );
                        if endpoint && !anchor_exact(core, order, atom_cut) {
                            EventWitness::Transparent
                        } else {
                            known(topology.control_orders[control])
                        }
                    }
                },
            };
            witnesses.push(witness);
        }
        let mut left = Vec::new();
        left.try_reserve_exact(len)
            .map_err(|_| ScopeIndexCapacity)?;
        let mut value = Some(Position(paragraph.enter, true));
        left.push(value);
        for witness in &witnesses {
            match witness {
                EventWitness::Known(witness) => value = Some(witness.last),
                EventWitness::Unknown => value = None,
                EventWitness::Transparent => (),
            }
            left.push(value);
        }
        let mut right = Vec::new();
        right
            .try_reserve_exact(len)
            .map_err(|_| ScopeIndexCapacity)?;
        right.resize(len, None);
        value = Some(Position(paragraph.exit, true));
        right[len - 1] = value;
        for (index, witness) in witnesses.iter().enumerate().rev() {
            match witness {
                EventWitness::Known(witness) => value = Some(witness.first),
                EventWitness::Unknown => value = None,
                EventWitness::Transparent => (),
            }
            right[index] = value;
        }
        Ok(Self {
            left,
            right,
            incidences: std::sync::OnceLock::new(),
        })
    }

    /// Both actual incident contributions must be inside. At a genuine box
    /// seam the closing box has left and the opening box has not yet entered.
    pub(crate) fn inside(&self, cut: usize, enter: usize, exit: usize) -> Option<bool> {
        let left = (*self.left.get(cut)?)?;
        let right = (*self.right.get(cut)?)?;
        let inside = |position| Position(enter, true) < position && position < Position(exit, true);
        Some(inside(left) && inside(right))
    }
}

fn known(order: usize) -> EventWitness {
    let point = Position(order, true);
    EventWitness::Known(Witness {
        first: point,
        last: point,
    })
}

fn witness_orders(orders: &mut impl Iterator<Item = usize>) -> EventWitness {
    let Some(first) = orders.next() else {
        return EventWitness::Unknown;
    };
    let last = orders.last().unwrap_or(first);
    EventWitness::Known(Witness {
        first: Position(first, true),
        last: Position(last, true),
    })
}

fn original(core: &AnalyzedCore, row: usize) -> EventWitness {
    let topology = core
        .paragraph
        .scope_topology
        .as_ref()
        .expect("captured index");
    witness_orders(
        &mut core.transform.pieces[core.transform.rows[row].pieces.clone()]
            .iter()
            .map(|piece| topology.origin_orders[piece.origin]),
    )
}

/// Projection exactness is checked against the already executed host and final
/// typographic units. No Unicode classification or glyph position is consulted.
pub(crate) fn anchor_exact(core: &AnalyzedCore, order: usize, semantic_atom: usize) -> bool {
    let boxes = core.processed.source().boxes();
    let source = super::anchor_position(
        core.paragraph.anchor_order[order].item,
        &core.paragraph,
        boxes,
    );
    let host = &core.host_projection;
    let host_cut = host.atoms.partition_point(|atom| atom.source.end <= source);
    if host
        .atoms
        .get(host_cut)
        .is_some_and(|atom| atom.source.start < source)
        || super::anchor_atom(order, &core.paragraph, boxes, host) != host_cut
    {
        return false;
    }
    // Input rows are actual Phase-I output operations with their original
    // contributions, not source hulls inferred from a selected glyph range.
    let row = core
        .transform
        .rows
        .partition_point(|row| row.source.end <= source);
    if core
        .transform
        .rows
        .get(row)
        .is_some_and(|row| row.source.start < source)
    {
        return false;
    }
    let projection = &core.projection;
    let at = projection.atoms.get(semantic_atom).map_or_else(
        || {
            projection
                .atoms
                .last()
                .map_or(0, |atom| atom.presentation.end)
        },
        |atom| atom.presentation.start,
    );
    projection.graphemes.binary_search(&at).is_ok()
        && super::anchor_atom(order, &core.paragraph, boxes, projection) == semantic_atom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::sync::Arc;

    // Each origin is a genuine source contribution; the inline scope begins
    // before origin 1 and ends after origin 2, independently of typography.
    fn analyzed(parts: [&str; 4], mode: WhiteSpace) -> AnalyzedSource {
        let source = Source::new(parts.concat());
        let mut at = 0;
        let origins = parts
            .into_iter()
            .map(|part| {
                let end = at + part.encode_utf16().count();
                let range = Range::new(at, end);
                at = end;
                ParagraphOrigin {
                    source: source.clone(),
                    local: range,
                    aggregate: range,
                    kind: ParagraphOriginKind::Authored,
                    text_node: None,
                }
            })
            .collect();
        let input = ContextualSourceInput {
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins,
            connections: (0..3)
                .map(|i| SourceProcessingConnection {
                    before_origin: i,
                    after_origin: i + 1,
                    kind: ParagraphConnectionKind::Continuous,
                })
                .collect(),
            controls: vec![],
            boundary_policies: (0..3)
                .map(|i| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(i),
                    preceding_wrap: Wrap::Word,
                    following_wrap: Wrap::Word,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
            replacements: vec![],
            scopes: vec![
                SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    topology: SourceProcessingTopology::Unknown,
                    trim: WhiteSpaceTrim::default(),
                    base_style: Style {
                        white_space: mode,
                        ..Style::default()
                    },
                    used_direction: ParagraphBaseDirection::LeftToRight,
                    font_reference: None,
                },
                SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: WhiteSpaceTrim::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events: vec![
                SourceProcessingEvent::EnterScope(0),
                SourceProcessingEvent::EnterParagraph(0),
                SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::EnterScope(1),
                SourceProcessingEvent::Connection(0),
                SourceProcessingEvent::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::Connection(1),
                SourceProcessingEvent::Origin {
                    origin: 2,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitScope(1),
                SourceProcessingEvent::Connection(2),
                SourceProcessingEvent::Origin {
                    origin: 3,
                    affinity: Affinity::After,
                },
                SourceProcessingEvent::ExitParagraph(0),
                SourceProcessingEvent::ExitScope(0),
            ],
        };
        let processed = Arc::new(process_contextual_source(input).unwrap());
        AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap()
    }

    fn incidences(analyzed: &AnalyzedSource) -> Vec<bool> {
        let core = &analyzed.inner.core;
        let index = ScopeCutIndex::new(core).unwrap();
        let fact = &core
            .paragraph
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
            .scopes[1];
        core.transform
            .participant_cuts
            .iter()
            .map(|&cut| index.inside(cut, fact.enter, fact.exit).unwrap())
            .collect()
    }

    #[test]
    fn canonical_scope_edges_are_outside_and_body_seam_is_inside() {
        let analyzed = analyzed(["A", "B", "C", "D"], WhiteSpace::Preserve);
        assert_eq!(incidences(&analyzed), [false, false, true, false, false]);
        assert!(
            analyzed.inner.scope_cuts.get().is_none(),
            "unused query stays lazy"
        );
    }

    #[test]
    fn cross_origin_unit_uses_actual_last_contribution_after_whole_unit() {
        let analyzed = analyzed(["A", "\u{301}", "B", "C"], WhiteSpace::Preserve);
        assert_eq!(analyzed.inner.projection.graphemes, [0, 2, 3, 4]);
        // The interior scalar seam is not a prepared boundary. The whole first
        // TCU has consumed the genuine entering contribution at its trailing cut.
        assert_eq!(incidences(&analyzed), [false, false, true, false, false]);
    }

    #[test]
    fn executed_collapsing_correspondence_retains_scope_owner() {
        let analyzed = analyzed(["A", " ", " ", "B"], WhiteSpace::Collapse);
        assert_eq!(
            analyzed.inner.processed.processed_text(),
            &Text::from("A B")
        );
        let core = &analyzed.inner.core;
        let index = ScopeCutIndex::new(core).unwrap();
        let scope = &core
            .paragraph
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
            .scopes[1];
        // Output SPACE carries both original contributions, with the consumed
        // deleted SPACE separately retained at its actual relation seam.
        assert_eq!(index.inside(0, scope.enter, scope.exit), Some(false));
        assert_eq!(
            index.inside(core.transform.events.len(), scope.enter, scope.exit),
            Some(false)
        );
    }
}
