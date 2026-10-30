use super::casing::MappingError;
use super::mapping::failed;
use super::{
    casing,
    relation::{ComponentOwner, Emission, RelationEvent, TransformMap},
};
use crate::{
    ProcessedSource, Range, Result,
    engine::{OriginalAtom, analysis::wrapping::Event, resolve::AnalyzedStyleRun},
    projection::SourceProjection,
    protocol::paragraph::ParagraphData,
};
use icu_normalizer::properties::CanonicalCombiningClassMap;
use std::sync::Arc;

pub(super) fn apply(
    input: SourceProjection,
    styles: &mut Vec<AnalyzedStyleRun>,
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
) -> Result<(SourceProjection, Arc<TransformMap>)> {
    let context = super::context::SourceContext::surrounding(processed, paragraph)?;
    let input = context.projection(input)?;
    let super::transaction::Completed {
        input,
        styles: source_styles,
        mut map,
        captured,
        mappings:
            super::mapping::Mappings {
                output,
                identities,
                spans,
                operations,
            },
    } = super::transaction::run(&context, input)?;
    let values: Vec<_> = input.atoms.iter().map(|atom| atom.code_point).collect();
    let mut stops = input.barrier_atoms.clone();
    stops.extend([0, input.atoms.len()]);
    for (row, &cp) in values.iter().enumerate() {
        if crate::engine::analysis::is_hard_separator(cp) {
            stops.extend([row, row + 1]);
        }
    }
    stops.sort_unstable();
    stops.dedup();
    let changed = spans.iter().enumerate().any(|(row, span)| {
        span.len() != 1
            || match map.components[row].owner {
                ComponentOwner::Original(original) => {
                    output[span.start] != map.rows[original].code_point
                }
                ComponentOwner::Wbr(_) | ComponentOwner::Virtual(_) => true,
            }
    });
    map.emissions.clear();
    map.atoms.clear();
    map.events.clear();
    map.emissions
        .try_reserve_exact(output.len())
        .map_err(|_| failed(MappingError::Storage))?;
    map.atoms
        .try_reserve_exact(output.len())
        .map_err(|_| failed(MappingError::Storage))?;
    let event_capacity = captured
        .len()
        .checked_add(output.len())
        .ok_or_else(|| failed(MappingError::Storage))?;
    map.events
        .try_reserve_exact(event_capacity)
        .map_err(|_| failed(MappingError::Storage))?;
    for (row, span) in spans.iter().enumerate() {
        let start = map.emissions.len();
        for (subposition, &code_point) in output[span.clone()].iter().enumerate() {
            map.emissions.push(Emission {
                component: row,
                subposition,
                code_point,
                selection_identity: identities[span.start + subposition],
                atom: 0,
                event: 0,
            });
        }
        map.components[row].outputs = start..map.emissions.len();
        if let ComponentOwner::Original(original) = map.components[row].owner {
            map.rows[original].outputs = start..map.emissions.len();
        }
    }
    let mut atoms = Vec::new();
    let mut pending = Vec::new();
    atoms
        .try_reserve_exact(output.len())
        .map_err(|_| failed(MappingError::Storage))?;
    pending
        .try_reserve_exact(output.len())
        .map_err(|_| failed(MappingError::Storage))?;
    let classes = CanonicalCombiningClassMap::new();
    for event in captured {
        if stops.binary_search(&event.atom).is_ok()
            || values
                .get(event.atom)
                .is_none_or(|&cp| classes.get32_u8(cp) == 0)
        {
            flush(&mut pending, &mut atoms, &mut map)?;
        }
        match event.kind {
            Event::Text(row) => {
                let range = map.components[row].outputs.clone();
                if range.is_empty() {
                    let ComponentOwner::Original(original) = map.components[row].owner else {
                        unreachable!("separator literals are not case removals")
                    };
                    map.events.push(RelationEvent::Removal { row: original });
                }
                let postpone = casing::postpones_iota(
                    values[row],
                    operations[row],
                    range
                        .clone()
                        .last()
                        .map(|id| map.emissions[id].selection_identity),
                );
                let immediate = range.len().saturating_sub(usize::from(postpone));
                if immediate > 0 && !pending.is_empty() {
                    for &id in &pending {
                        let before = &map.components[map.emissions[id].component];
                        if let Some(edge) = context.missing.iter().find(|edge| {
                            before.source.end <= edge.at
                                && edge.at <= map.components[row].source.start
                        }) {
                            return Err(
                                edge.unavailable(crate::SourceContinuityOperation::Transformation)
                            );
                        }
                    }
                }
                for id in range.clone() {
                    if postpone && id + 1 == range.end {
                        pending.push(id);
                    } else {
                        emit(id, &mut atoms, &mut map)?;
                    }
                }
            }
            Event::Deleted(source, processed, lexical) => {
                map.events.push(RelationEvent::PhaseIDeleted {
                    source,
                    processed,
                    lexical,
                })
            }
            Event::Anchor(anchor) => map.events.push(RelationEvent::Anchor {
                order: context
                    .anchor_order
                    .iter()
                    .position(|value| *value == anchor)
                    .expect("captured actual anchor"),
            }),
        }
    }
    flush(&mut pending, &mut atoms, &mut map)?;
    map.finish_cuts()?;
    let mut end = 0usize;
    for atom in &mut atoms {
        let length = if (0xd800..=0xdfff).contains(&atom.code_point) {
            1
        } else {
            atom.glyph.len_utf16()
        };
        let next = end
            .checked_add(length)
            .ok_or_else(crate::projection::length_overflow)?;
        atom.presentation = Range::new(end, next);
        end = next;
    }
    // Semantic barriers are component-prefix cuts, even when a casing row
    // emits delayed pieces. Preserve the furthest actual emitted endpoint.
    let mut component_ends = Vec::new();
    let edge_count = map
        .components
        .len()
        .checked_add(1)
        .ok_or_else(|| failed(MappingError::Storage))?;
    component_ends
        .try_reserve_exact(edge_count)
        .map_err(|_| failed(MappingError::Storage))?;
    component_ends.resize(edge_count, 0);
    for emission in &map.emissions {
        component_ends[emission.component + 1] =
            component_ends[emission.component + 1].max(emission.atom + 1);
    }
    for index in 1..component_ends.len() {
        component_ends[index] = component_ends[index].max(component_ends[index - 1]);
    }
    let barrier_atoms = input
        .barrier_atoms
        .iter()
        .map(|&edge| component_ends[edge])
        .collect();
    let projected_anchors = input.anchor_atoms.clone();
    let mut projection = if changed {
        SourceProjection::from_atoms_at_barriers(atoms, end, input.barriers, barrier_atoms)?
    } else {
        input
    };
    projection.style_rows = (0..projection.atoms.len())
        .map(|atom| Some(map.component_for_atom(atom).style))
        .collect();
    projection.anchor_atoms = vec![None; context.anchor_order.len()];
    let mut at = 0;
    for event in &map.events {
        match *event {
            RelationEvent::Text { atom } => {
                if let ComponentOwner::Wbr(control) = map.component_for_atom(atom).owner {
                    let order = context
                        .anchor_order
                        .iter()
                        .position(|anchor| {
                            anchor.item
                                == crate::protocol::paragraph::SemanticAnchorItem::Control(control)
                        })
                        .expect("actual wbr anchor");
                    projection.anchor_atoms[order] = Some(at);
                }
                at += 1;
            }
            RelationEvent::Anchor { order } => projection.anchor_atoms[order] = Some(at),
            _ => (),
        }
    }
    // Wbr staging retains ordinary typographic projections separately from
    // authentic semantic relation events. Carry their real component cuts
    // through later casing/width/kana emissions without sorting the events.
    for (order, cut) in projected_anchors.into_iter().enumerate() {
        if let Some(cut) = cut {
            projection.anchor_atoms[order] = Some(component_ends[cut]);
        }
    }
    super::project::finish(&context, &source_styles, styles, projection, map)
}

fn emit(id: usize, atoms: &mut Vec<OriginalAtom>, map: &mut TransformMap) -> Result<()> {
    let emission = &mut map.emissions[id];
    let row = &map.components[emission.component];
    let atom = OriginalAtom {
        source: row.source,
        processed: row.processed,
        presentation: Range::new(0, 0),
        glyph_bytes: 0..0,
        code_point: emission.code_point,
        glyph: if (0xd800..=0xdfff).contains(&emission.code_point) {
            '\u{fffd}'
        } else {
            char::from_u32(emission.code_point)
                .ok_or_else(|| failed(MappingError::InvalidScalar))?
        },
    };
    emission.atom = atoms.len();
    emission.event = map.events.len();
    map.atoms.push(id);
    map.events.push(RelationEvent::Text { atom: atoms.len() });
    atoms.push(atom);
    Ok(())
}

fn flush(
    pending: &mut Vec<usize>,
    atoms: &mut Vec<OriginalAtom>,
    map: &mut TransformMap,
) -> Result<()> {
    for id in pending.drain(..) {
        emit(id, atoms, map)?;
    }
    Ok(())
}
