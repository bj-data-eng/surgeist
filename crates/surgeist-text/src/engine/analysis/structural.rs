//! Genuine domain/ordinary-child-Block source frontiers. Raw storage members,
//! forced controls, atomics and disconnections confer no section boundary.
use crate::protocol::paragraph::{ParagraphContext, SemanticAnchorItem};
use crate::transform::relation::{ComponentOwner, RelationEvent, TransformMap};
use crate::{Range, Result, SourceProcessingEvent};
use std::ops::Range as IndexRange;

#[derive(Clone, Debug)]
pub(crate) struct Section {
    pub(crate) entry: Option<usize>,
    pub(crate) exit: Option<usize>,
    pub(crate) atoms: IndexRange<usize>,
    pub(crate) events: IndexRange<usize>,
    pub(crate) participants: IndexRange<usize>,
    pub(crate) entry_cursor: Cursor,
    pub(crate) exit_cursor: Cursor,
    /// Genuine containing domain/child-Block bounds, independent of the
    /// output's owned horizon. Raw markers do not clip localized box edges.
    pub(crate) span: Option<StructuralSpan>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StructuralSpan {
    pub(crate) entry: usize,
    pub(crate) exit: usize,
}

#[derive(Clone, Copy)]
enum SourcePosition {
    Event(usize),
    Cut(usize),
}
impl SourcePosition {
    fn order(self) -> usize {
        match self {
            Self::Event(order) | Self::Cut(order) => order,
        }
    }
    fn end(self) -> Result<usize> {
        match self {
            Self::Event(order) => order.checked_add(1).ok_or_else(super::capacity),
            Self::Cut(cut) => Ok(cut),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    /// Captured-frontier cursor facts. Uncaptured standalone retains its native
    /// first/last boundary extrema instead; whole extent is not that cursor.
    pub(crate) source_before: usize,
    pub(crate) source_after: usize,
    pub(crate) processed: usize,
}

pub(crate) fn anchor_order(
    item: SemanticAnchorItem,
    paragraph: &ParagraphContext,
) -> Option<usize> {
    let topology = paragraph.scope_topology.as_ref()?;
    Some(match item {
        SemanticAnchorItem::EmptyOrigin(origin) => paragraph
            .captured_origin_event(origin)
            .expect("captured source original"),
        SemanticAnchorItem::Control(control) => topology.control_orders[control],
        SemanticAnchorItem::Atomic(atomic) => topology.capture.atomic_orders[atomic],
        SemanticAnchorItem::ComputedLineBoundary(computed) => {
            paragraph.computed_boundaries[computed].capture_cut
        }
    })
}

fn visit_event_positions(
    event: RelationEvent,
    paragraph: &ParagraphContext,
    map: &TransformMap,
    emit: &mut impl FnMut(SourcePosition),
) {
    let originals = |row: usize, emit: &mut dyn FnMut(SourcePosition)| {
        for piece in map.original_pieces(row) {
            emit(SourcePosition::Event(
                piece
                    .origin
                    .captured_event()
                    .expect("captured source event"),
            ));
        }
    };
    match event {
        RelationEvent::Text { atom } => match map.component_for_atom(atom).owner {
            ComponentOwner::Original(row) => originals(row, emit),
            ComponentOwner::Wbr(control) => emit(SourcePosition::Event(
                anchor_order(SemanticAnchorItem::Control(control), paragraph).expect("source Wbr"),
            )),
            ComponentOwner::Virtual(index) => {
                emit(SourcePosition::Cut(map.virtuals[index].capture_cut))
            }
        },
        RelationEvent::Removal { row, .. } => originals(row, emit),
        RelationEvent::GeneratedSuppression { component, .. } => match map.components[component]
            .owner
        {
            ComponentOwner::Original(row) => originals(row, emit),
            ComponentOwner::Wbr(control) => emit(SourcePosition::Event(
                anchor_order(SemanticAnchorItem::Control(control), paragraph).expect("source Wbr"),
            )),
            ComponentOwner::Virtual(index) => {
                emit(SourcePosition::Cut(map.virtuals[index].capture_cut))
            }
        },
        RelationEvent::Anchor { order } => match paragraph.anchor_order[order].item {
            SemanticAnchorItem::ComputedLineBoundary(index) => {
                let capture = &paragraph
                    .scope_topology
                    .as_ref()
                    .expect("captured source")
                    .capture;
                for &cause in &paragraph.computed_boundaries[index].causes {
                    emit(SourcePosition::Cut(capture.computed[cause].capture_cut));
                }
            }
            item => emit(SourcePosition::Event(
                anchor_order(item, paragraph).expect("captured source anchor"),
            )),
        },
        RelationEvent::PhaseIDeleted { source, .. } => {
            // These are actual admitted original pieces; neither an envelope
            // nor the used-facts member supplies deletion ownership.
            let first = paragraph
                .origins
                .partition_point(|original| original.aggregate.end <= source.start);
            for (offset, original) in paragraph.origins[first..]
                .iter()
                .enumerate()
                .take_while(|(_, original)| original.aggregate.start < source.end)
                .filter(|(_, original)| {
                    source.start < original.aggregate.end && original.aggregate.start < source.end
                })
            {
                let _ = original;
                emit(SourcePosition::Event(
                    paragraph
                        .captured_origin_event(first + offset)
                        .expect("captured deletion source"),
                ));
            }
        }
    }
}

pub(crate) fn sections(
    paragraph: &ParagraphContext,
    map: &TransformMap,
    projection: &crate::projection::SourceProjection,
    processed: &crate::ProcessedSource,
) -> Result<Vec<Section>> {
    let atom_count = projection.atoms.len();
    let Some(topology) = &paragraph.scope_topology else {
        let extent = paragraph.extent(processed);
        return super::storage::collect([Section {
            entry: None,
            exit: None,
            atoms: 0..atom_count,
            events: 0..map.events.len(),
            participants: 0..map
                .logical_prefix_at_cut(map.events.len())
                .expect("completed relation"),
            entry_cursor: Cursor {
                source_before: extent.start,
                source_after: extent.start,
                processed: processed.project(extent.start, crate::Affinity::Before),
            },
            exit_cursor: Cursor {
                source_before: extent.end,
                source_after: extent.end,
                processed: processed.project(extent.end, crate::Affinity::After),
            },
            span: None,
        }]);
    };
    let capture = &topology.capture;
    let first = topology.owner.member(0).index;
    let last = topology.owner.member(topology.owner.len() - 1).index;
    let mut begin = capture.paragraphs[first].enter + 1;
    let mut end = capture.paragraphs[last].exit;
    let domain = &capture.scopes[capture.domain_scopes[topology.domain]];
    let mut bounds = Vec::new();
    let mut start = domain.enter + 1;
    for &order in &capture.flow_orders[topology.domain] {
        if let SourceProcessingEvent::EnterScope(scope) = capture.events[order].event {
            let scope_data = &capture.scopes[scope];
            if scope_data.domain != topology.domain
                && matches!(
                    scope_data.definition,
                    Some(crate::processing::contextual::CapturedDomainData::Block)
                )
                && scope_data
                    .parent
                    .is_some_and(|parent| capture.scopes[parent].domain == topology.domain)
            {
                super::storage::push(
                    &mut bounds,
                    StructuralSpan {
                        entry: start,
                        exit: order,
                    },
                )?;
                start = scope_data.exit.checked_add(1).ok_or_else(super::capacity)?;
            }
        }
    }
    super::storage::push(
        &mut bounds,
        StructuralSpan {
            entry: start,
            exit: domain.exit,
        },
    )?;
    let mut atoms = super::storage::filled(bounds.len(), 0usize)?;
    let mut events = super::storage::filled(bounds.len(), 0usize)?;
    let mut previous = 0;
    for (event_index, &event) in map.events.iter().enumerate() {
        let mut owner = None;
        let mut coherent = true;
        let mut overflow = false;
        visit_event_positions(event, paragraph, map, &mut |position| {
            let order = position.order();
            begin = begin.min(order);
            match position.end() {
                Ok(after) => end = end.max(after),
                Err(_) => overflow = true,
            }
            let index = bounds.partition_point(|span| span.exit < order);
            if bounds.get(index).is_none_or(|span| order < span.entry) {
                coherent = false;
                return;
            }
            if owner.is_some_and(|earlier| earlier != index) {
                coherent = false;
            }
            owner = Some(index);
        });
        if overflow {
            return Err(super::capacity());
        }
        if !coherent || owner.is_none_or(|index| index < previous) {
            return Err(crate::Error::new(
                crate::ErrorCode::UnsupportedFeature,
                "source event crosses a genuine structural frontier",
            )
            .with_detail(crate::ErrorDetail::TextTransform {
                reason: crate::TextTransformError::IndivisibleCoverage {
                    source: map.event_source(event_index, paragraph, processed.source()),
                },
            }));
        }
        let index = owner.expect("checked source-event custody");
        previous = index;
        events[index] = event_index + 1;
        if let RelationEvent::Text { atom } = event {
            atoms[index] = atoms[index].max(atom + 1);
        }
    }
    for index in 1..bounds.len() {
        atoms[index] = atoms[index].max(atoms[index - 1]);
        events[index] = events[index].max(events[index - 1]);
    }
    let mut result = Vec::new();
    for (index, span) in bounds.into_iter().enumerate() {
        if span.exit < begin || end < span.entry {
            continue;
        }
        let entry = span.entry.max(begin);
        let exit = span.exit.min(end);
        if entry > exit {
            return Err(crate::processing::contextual::invalid(
                crate::SourceProcessingError::Domain,
            ));
        }
        let atom_start = index.checked_sub(1).map_or(0, |before| atoms[before]);
        let atom_end = atoms[index];
        // A structural source cut is not permission to split actual output.
        // Canonical source fences already close mapped/segmented dependencies.
        for cut in [atom_start, atom_end] {
            let point = projection
                .atoms
                .get(cut)
                .map_or(projection.presentation_end(), |atom| {
                    atom.presentation.start
                });
            if projection.graphemes.binary_search(&point).is_err() {
                return Err(crate::Error::new(
                    crate::ErrorCode::UnsupportedFeature,
                    "structural frontier cuts a completed source unit",
                )
                .with_detail(crate::ErrorDetail::TextTransform {
                    reason: crate::TextTransformError::IndivisibleCoverage {
                        source: source_envelope(paragraph, map, atom_start..atom_end),
                    },
                }));
            }
        }
        let cursor = |cut: usize, affinity| {
            let at = capture
                .events
                .get(cut)
                .map_or(processed.source().text().len(), |event| event.at);
            Cursor {
                source_before: at,
                source_after: at,
                processed: processed.project(at, affinity),
            }
        };
        let event_start = index.checked_sub(1).map_or(0, |before| events[before]);
        let event_end = events[index];
        super::storage::push(
            &mut result,
            Section {
                entry: Some(entry),
                exit: Some(exit),
                atoms: atom_start..atom_end,
                events: event_start..event_end,
                participants: map
                    .logical_prefix_at_cut(event_start)
                    .expect("completed source cut")
                    ..map
                        .logical_prefix_at_cut(event_end)
                        .expect("completed source cut"),
                entry_cursor: cursor(entry, crate::Affinity::Before),
                exit_cursor: cursor(exit, crate::Affinity::After),
                span: Some(span),
            },
        )?;
    }
    Ok(result)
}

pub(crate) fn owner_section(
    owner: super::BidiOwner,
    paragraph: &ParagraphContext,
    sections: &[Section],
    combination: Option<&crate::transform::combined::CombinedOutput>,
) -> usize {
    match owner {
        super::BidiOwner::Atom(atom) => sections
            .partition_point(|section| section.atoms.end <= atom)
            .min(sections.len() - 1),
        super::BidiOwner::Atomic(index) => order_section(
            anchor_order(SemanticAnchorItem::Atomic(index), paragraph),
            sections,
        ),
        super::BidiOwner::Control(index) => order_section(
            anchor_order(SemanticAnchorItem::Control(index), paragraph),
            sections,
        ),
        super::BidiOwner::ComputedLineBoundary(index) => order_section(
            anchor_order(SemanticAnchorItem::ComputedLineBoundary(index), paragraph),
            sections,
        ),
        super::BidiOwner::CombinedCell(index) => {
            let binding = &combination.expect("real completed cell").bindings[index];
            sections
                .iter()
                .position(|section| {
                    section.events.start <= binding.events.start
                        && binding.events.end <= section.events.end
                })
                .expect("completed cell belongs to one genuine structural section")
        }
    }
}
fn order_section(order: Option<usize>, sections: &[Section]) -> usize {
    order.map_or(0, |order| {
        sections
            .iter()
            .position(|section| {
                section
                    .span
                    .is_some_and(|span| span.entry <= order && order <= span.exit)
            })
            .expect("owned source occurrence belongs to one actual structural span")
    })
}

pub(crate) fn source_envelope(
    paragraph: &ParagraphContext,
    map: &TransformMap,
    atoms: IndexRange<usize>,
) -> Range {
    let mut start = usize::MAX;
    let mut end = 0;
    for atom in atoms {
        let component = map.component_for_atom(atom);
        start = start.min(component.source.start);
        end = end.max(component.source.end);
    }
    if start == usize::MAX {
        Range::new(
            paragraph.envelope.map_or(0, |range| range.start),
            paragraph.envelope.map_or(0, |range| range.start),
        )
    } else {
        Range::new(start, end)
    }
}
