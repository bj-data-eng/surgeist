//! One font-independent logical occurrence stream and influencing-neighbor map.

use crate::engine::{OriginalAtom, resolve::AnalyzedStyleRun};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{
    ParagraphBoundaryDefinition, ParagraphControlKind, ParagraphData, SemanticAnchor,
    SemanticAnchorItem,
};
use crate::{InlineBoxKind, ProcessedSource, Range, Wrap, WrappingNeighbor};

use super::{Analysis, anchor_atom, anchor_position, is_hard_separator};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Event {
    Text(usize),
    Deleted(Range, usize),
    Anchor(SemanticAnchor),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LogicalEvent {
    pub(crate) atom: usize,
    pub(crate) source: usize,
    pub(crate) kind: Event,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Influence {
    ComputedLineBoundary {
        participant: usize,
    },
    Text {
        participant: usize,
        atom: usize,
    },
    Atomic {
        participant: usize,
        occurrence: usize,
    },
    Control {
        participant: usize,
        occurrence: usize,
    },
}

impl Influence {
    fn participant(self) -> usize {
        match self {
            Self::Text { participant, .. }
            | Self::Atomic { participant, .. }
            | Self::Control { participant, .. }
            | Self::ComputedLineBoundary { participant } => participant,
        }
    }

    fn code_point(self, atoms: &[OriginalAtom]) -> Option<u32> {
        match self {
            Self::Text { atom, .. } => Some(atoms[atom].code_point),
            _ => None,
        }
    }

    fn starts_hard_separator(self, atoms: &[OriginalAtom], paragraph: &ParagraphData) -> bool {
        match self {
            Self::Text { atom, .. } => is_hard_separator(atoms[atom].code_point),
            Self::Control { occurrence, .. } => {
                paragraph.controls[occurrence].kind == ParagraphControlKind::ForcedBreak
            }
            Self::Atomic { .. } => false,
            Self::ComputedLineBoundary { .. } => true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Relation {
    pub(crate) before: Option<Influence>,
    pub(crate) after: Option<Influence>,
    /// Exact neighboring source contributions/atomic occurrences, when both
    /// sides can consume ancestor wrapping rather than explicit control rules.
    pub(crate) neighbors: Option<(WrappingNeighbor, WrappingNeighbor)>,
    pub(crate) before_hard_separator: bool,
    pub(crate) atomic: bool,
    pub(crate) neighbors_permit: bool,
    pub(crate) space_permits: bool,
    /// Existing capture normalized only on a proven exact incident pair.
    pub(crate) captured_wrap: Option<Wrap>,
}

#[derive(Debug)]
pub(crate) struct LogicalParagraph {
    pub(crate) events: Vec<LogicalEvent>,
    pub(crate) relations: Vec<Relation>,
    /// A relation for every exact mixed-sequence cut, not a source-offset map.
    pub(crate) cuts: Vec<usize>,
    pub(crate) relation_indices: Vec<usize>,
}

impl LogicalParagraph {
    pub(crate) fn new(
        processed: &ProcessedSource,
        paragraph: &ParagraphData,
        projection: &SourceProjection,
        styles: &[AnalyzedStyleRun],
        analysis: &Analysis,
    ) -> crate::Result<Self> {
        use icu_properties::{CodePointMapData, props::LineBreak};
        let events = events(processed, paragraph, projection, styles);
        let influences: Vec<_> = events
            .iter()
            .enumerate()
            .filter_map(|(participant, event)| match event.kind {
                Event::Text(atom) => Some(Influence::Text { participant, atom }),
                Event::Anchor(SemanticAnchor {
                    item: SemanticAnchorItem::Atomic(occurrence),
                    ..
                }) if processed.source().boxes()[occurrence].kind() == InlineBoxKind::InFlow => {
                    Some(Influence::Atomic {
                        participant,
                        occurrence,
                    })
                }
                Event::Anchor(SemanticAnchor {
                    item: SemanticAnchorItem::Control(control),
                    ..
                }) => {
                    let kind = &paragraph.controls[control].kind;
                    match kind {
                        ParagraphControlKind::ForcedBreak
                        | ParagraphControlKind::OptionalBreak { .. }
                        | ParagraphControlKind::DiscretionaryBreak { .. } => {
                            Some(Influence::Control {
                                participant,
                                occurrence: control,
                            })
                        }
                        _ => None,
                    }
                }
                Event::Anchor(SemanticAnchor {
                    item: SemanticAnchorItem::ComputedLineBoundary(_),
                    ..
                }) => Some(Influence::ComputedLineBoundary { participant }),
                _ => None,
            })
            .collect();
        let classes = CodePointMapData::<LineBreak>::new();
        let forbidden = |code_point| {
            code_point != 0xa0
                && matches!(
                    classes.get32(code_point),
                    LineBreak::Glue | LineBreak::WordJoiner | LineBreak::ZWJ
                )
        };
        let mut relations = Vec::with_capacity(influences.len() + 1);
        for right in 0..=influences.len() {
            let before = right.checked_sub(1).map(|index| influences[index]);
            let after = influences.get(right).copied();
            let neighbors = before.zip(after).and_then(|(before, after)| {
                Some((
                    neighbor(before, false, processed, paragraph, projection)?,
                    neighbor(after, true, processed, paragraph, projection)?,
                ))
            });
            let space_permits = match (before, after) {
                (Some(Influence::Text { atom, .. }), Some(Influence::Atomic { .. })) => {
                    let style = styles[analysis.infos[atom].1].style.authored();
                    projection.atoms[atom].code_point == 0x20
                        && style.white_space.collapses_spaces()
                        && style.wrap == Wrap::Word
                }
                _ => false,
            };
            relations.push(Relation {
                before,
                after,
                neighbors,
                before_hard_separator: after
                    .is_some_and(|value| value.starts_hard_separator(&projection.atoms, paragraph)),
                atomic: [before, after]
                    .into_iter()
                    .flatten()
                    .any(|value| matches!(value, Influence::Atomic { .. })),
                neighbors_permit: before.is_some()
                    && after.is_some()
                    && !before
                        .and_then(|value| value.code_point(&projection.atoms))
                        .is_some_and(forbidden)
                    && !after
                        .and_then(|value| value.code_point(&projection.atoms))
                        .is_some_and(forbidden),
                space_permits,
                captured_wrap: None,
            });
        }
        let mut right = 0;
        let cuts: Vec<_> = (0..=events.len())
            .map(|cut| {
                while influences
                    .get(right)
                    .is_some_and(|value| value.participant() < cut)
                {
                    right += 1;
                }
                right
            })
            .collect();
        // A seam proves this pair only when the original supplied semantic
        // occurrences are themselves its actual influencing neighbors.
        for (policy_index, policy) in paragraph.boundary_policies.iter().enumerate() {
            let cut = match policy.definition {
                ParagraphBoundaryDefinition::Connection(connection) => {
                    let origin = &paragraph.origins[connection];
                    events.iter().enumerate().filter(|(_, event)| match event.kind {
                        Event::Text(atom) => projection.atoms[atom].source.end == origin.aggregate.end,
                        Event::Deleted(range, _) => range.end == origin.aggregate.end,
                        Event::Anchor(SemanticAnchor { item: SemanticAnchorItem::EmptyOrigin(index), .. }) => index == connection,
                        _ => false,
                    }).map(|(index, _)| index + 1).next_back()
                }
                ParagraphBoundaryDefinition::AtomicBefore(occurrence) => events.iter().position(|event| matches!(event.kind, Event::Anchor(SemanticAnchor { item: SemanticAnchorItem::Atomic(index), .. }) if index == occurrence)),
                ParagraphBoundaryDefinition::AtomicAfter(occurrence) => events.iter().position(|event| matches!(event.kind, Event::Anchor(SemanticAnchor { item: SemanticAnchorItem::Atomic(index), .. }) if index == occurrence)).map(|index| index + 1),
            };
            let Some(cut) = cut else {
                continue;
            };
            let relation = &mut relations[cuts[cut]];
            let exact = match (policy.definition, relation.neighbors) {
                (
                    ParagraphBoundaryDefinition::Connection(index),
                    Some((
                        WrappingNeighbor::Text { origin: before, .. },
                        WrappingNeighbor::Text { origin: after, .. },
                    )),
                ) => before == index && after == index + 1,
                (
                    ParagraphBoundaryDefinition::AtomicBefore(index),
                    Some((_, WrappingNeighbor::Atomic { occurrence })),
                ) => index == occurrence,
                (
                    ParagraphBoundaryDefinition::AtomicAfter(index),
                    Some((WrappingNeighbor::Atomic { occurrence }, _)),
                ) => index == occurrence,
                _ => false,
            };
            if !exact || relation.neighbors.is_none() {
                continue;
            }
            if relation
                .captured_wrap
                .is_some_and(|wrap| wrap != policy.ancestor_wrap)
            {
                return Err(crate::protocol::protocol_error(
                    crate::ProtocolError::InvalidBoundaryPolicy {
                        policy: policy_index,
                        at: policy.at(paragraph, processed.source().boxes()),
                    },
                ));
            }
            relation.captured_wrap = Some(policy.ancestor_wrap);
        }
        let relation_indices = if paragraph.base_direction.is_some() {
            relations
                .iter()
                .enumerate()
                .filter_map(|(index, relation)| relation.neighbors.map(|_| index))
                .collect()
        } else {
            Vec::new()
        };
        Ok(Self {
            events,
            relations,
            cuts,
            relation_indices,
        })
    }
}

fn neighbor(
    influence: Influence,
    following: bool,
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
    projection: &SourceProjection,
) -> Option<WrappingNeighbor> {
    match influence {
        Influence::Atomic { occurrence, .. } => Some(WrappingNeighbor::Atomic { occurrence }),
        Influence::Text { atom, .. } => {
            let mut pieces = paragraph
                .origin_intersections(processed, projection.atoms[atom].source)
                .filter(|piece| !piece.aggregate.is_empty());
            let piece = if following {
                pieces.next()?
            } else {
                pieces.last()?
            };
            Some(WrappingNeighbor::Text {
                origin: piece.origin,
                local: piece.local,
            })
        }
        Influence::Control { .. } | Influence::ComputedLineBoundary { .. } => None,
    }
}

pub(crate) fn events(
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
) -> Vec<LogicalEvent> {
    if let Some(map) = &projection.transform {
        return map.logical_events(paragraph, processed.source());
    }
    transform_events(processed, paragraph, projection, styles)
}

/// Wbr expansion needs its real scalar seam before typography reconstruction.
pub(crate) fn transform_events(
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
) -> Vec<LogicalEvent> {
    capture_events(processed, paragraph, projection, styles, false)
}

/// Original scalar seams and actual anchor order used only for separator
/// admission/insertion, before typography is rebuilt. No source atoms split.
pub(crate) fn separator_events(
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
) -> Vec<LogicalEvent> {
    capture_events(processed, paragraph, projection, styles, true)
}

fn capture_events(
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    scalar_seams: bool,
) -> Vec<LogicalEvent> {
    use crate::processing::MappingKind;
    let source = processed.source();
    let mut events = Vec::new();
    for (atom, value) in projection.atoms.iter().enumerate() {
        events.push((atom, 1usize, value.source.start, 0usize, Event::Text(atom)));
    }
    let mut cuts: Vec<_> = paragraph
        .origins
        .iter()
        .flat_map(|origin| [origin.aggregate.start, origin.aggregate.end])
        .collect();
    cuts.extend(
        paragraph
            .anchor_order
            .iter()
            .map(|anchor| anchor_position(anchor.item, paragraph, source.boxes())),
    );
    cuts.sort_unstable();
    cuts.dedup();
    // Lexical correspondence may coalesce adjacent collapsed SPACE originals.
    // Keep their actual source-style owners' seams before the transformation
    // map records custody, so an interior owner's local gate remains callable.
    // Other deletion causes retain their existing native/anchor partition.
    let mut collapsed_cuts = cuts.clone();
    collapsed_cuts.extend(
        styles
            .iter()
            .filter_map(AnalyzedStyleRun::source_range)
            .flat_map(|range| [range.start, range.end]),
    );
    collapsed_cuts.sort_unstable();
    collapsed_cuts.dedup();
    for piece in processed.operations() {
        if !matches!(piece.kind, MappingKind::Delete(_)) {
            continue;
        }
        let cuts = if piece.kind
            == MappingKind::Delete(crate::processing::DeletionCause::CollapsedSpace)
        {
            &collapsed_cuts
        } else {
            &cuts
        };
        for piece_range in paragraph.intersections(piece.source) {
            let first = cuts.partition_point(|at| *at <= piece_range.start);
            let last = cuts.partition_point(|at| *at < piece_range.end);
            let mut start = piece_range.start;
            for end in cuts[first..last]
                .iter()
                .copied()
                .chain(std::iter::once(piece_range.end))
            {
                let atom = projection
                    .atoms
                    .partition_point(|value| value.source.end <= start);
                events.push((
                    atom,
                    0,
                    start,
                    usize::MAX,
                    Event::Deleted(Range::new(start, end), piece.processed),
                ));
                start = end;
            }
        }
    }
    for (order, &anchor) in paragraph.anchor_order.iter().enumerate() {
        events.push((
            if scalar_seams {
                let at = anchor_position(anchor.item, paragraph, source.boxes());
                projection
                    .atoms
                    .partition_point(|atom| atom.source.end <= at)
            } else {
                anchor_atom(order, paragraph, source.boxes(), projection)
            },
            0,
            anchor_position(anchor.item, paragraph, source.boxes()),
            order,
            Event::Anchor(anchor),
        ));
    }
    events.sort_by_key(|event| (event.0, event.1, event.2, event.3));
    events
        .into_iter()
        .map(|(atom, _, source, _, kind)| LogicalEvent { atom, source, kind })
        .collect()
}
