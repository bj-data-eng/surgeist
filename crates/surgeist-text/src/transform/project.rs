//! Checked exact output construction from recorded canonical mappings.

use super::{
    context::{self, SourceContext, StyleRow},
    relation::{ComponentOwner, RelationEvent, TransformMap},
};
use crate::{
    Error, ErrorCode, ErrorDetail, Range, Result, SourceContinuityOperation, TextTransformError,
    engine::resolve::AnalyzedStyleRun, projection::SourceProjection,
};
use std::sync::Arc;

fn refused(source: Range) -> Error {
    Error::new(
        ErrorCode::UnsupportedFeature,
        "completed transformed unit exceeds exact output coverage",
    )
    .with_detail(ErrorDetail::TextTransform {
        reason: TextTransformError::IndivisibleCoverage { source },
    })
}

pub(super) fn owns_component(context: &SourceContext<'_>, map: &TransformMap, row: usize) -> bool {
    match map.components[row].owner {
        ComponentOwner::Original(original) => map.original_pieces(original).iter().any(|piece| {
            context
                .requested
                .origin_map
                .view_origin(piece.origin)
                .is_some()
        }),
        ComponentOwner::Wbr(control) => context.local_control(control).is_some(),
        ComponentOwner::Virtual(index) => virtual_owned(context, &map.virtuals[index]),
    }
}

fn virtual_owned(context: &SourceContext<'_>, row: &super::relation::VirtualSeparatorData) -> bool {
    context.owns_capture_cut(row.capture_cut)
}

/// Actual scalar EGCs seed work only. Final units are always segmented again
/// from recorded emissions after removals, expansions and delayed ordering.
pub(super) fn scalar_units(input: &SourceProjection) -> Result<Vec<std::ops::Range<usize>>> {
    let mut text = String::new();
    let mut offsets = Vec::new();
    for atom in &input.atoms {
        context::push(&mut offsets, text.len())?;
        let scalar = if (0xd800..=0xdfff).contains(&atom.code_point) {
            '\u{a6}'
        } else {
            char::from_u32(atom.code_point).ok_or_else(context::capacity)?
        };
        text.try_reserve(scalar.len_utf8())
            .map_err(|_| context::capacity())?;
        text.push(scalar);
    }
    context::push(&mut offsets, text.len())?;
    let mut cuts = Vec::new();
    context::push(&mut cuts, 0)?;
    for &cut in &input.barrier_atoms {
        context::push(&mut cuts, offsets[cut])?;
    }
    context::push(&mut cuts, text.len())?;
    cuts.sort_unstable();
    cuts.dedup();
    let mut boundaries = Vec::new();
    crate::projection::visit_grapheme_boundaries(&text, &cuts, |byte| {
        let row = offsets
            .binary_search(&byte)
            .map_err(|_| context::capacity())?;
        if boundaries.last() != Some(&row) {
            context::push(&mut boundaries, row)?;
        }
        Ok(())
    })?;
    context::collect_rows(boundaries.windows(2).map(|pair| pair[0]..pair[1]))
}

pub(super) fn check_units(
    context: &SourceContext<'_>,
    map: &TransformMap,
    input: &SourceProjection,
) -> Result<()> {
    for unit in scalar_units(input)? {
        let mut owns = false;
        let mut other = false;
        let mut first = usize::MAX;
        let mut last = 0;
        let mut original_indices = Vec::new();
        for atom in unit {
            let row = map.emissions[map.atoms[atom]].component;
            let component = &map.components[row];
            owns |= owns_component(context, map, row);
            other |= !owns_component(context, map, row);
            if let ComponentOwner::Original(original) = component.owner {
                for piece in map.original_pieces(original) {
                    first = first.min(piece.aggregate.start);
                    last = last.max(piece.aggregate.end);
                    other |= context
                        .requested
                        .origin_map
                        .view_origin(piece.origin)
                        .is_none();
                    context::push(
                        &mut original_indices,
                        context
                            .origin_index(piece.origin)
                            .expect("actual source original"),
                    )?;
                }
            } else {
                first = first.min(component.source.start);
                last = last.max(component.source.end);
            }
        }
        if !owns {
            continue;
        }
        for edge in &context.missing {
            let cut = context
                .origins
                .partition_point(|origin| origin.aggregate.start < edge.at);
            if original_indices.iter().any(|&index| index < cut)
                && original_indices.iter().any(|&index| index >= cut)
            {
                return Err(edge.unavailable(SourceContinuityOperation::Segmentation));
            }
        }
        if other {
            return Err(refused(Range::new(first.min(last), last)));
        }
    }
    Ok(())
}

pub(super) fn finish(
    context: &SourceContext<'_>,
    styles: &[StyleRow<'_>],
    output_styles: &mut Vec<AnalyzedStyleRun>,
    input: SourceProjection,
    map: TransformMap,
) -> Result<(SourceProjection, Arc<TransformMap>)> {
    check_units(context, &map, &input)?;
    let mut result = TransformMap::default();
    let mut rows = context::filled(map.rows.len(), None)?;
    let mut components = context::filled(map.components.len(), None)?;
    let mut scopes = context::filled(map.scopes.len(), None)?;
    for (index, row) in map.rows.iter().enumerate() {
        let start = result.pieces.len();
        for &piece in map.original_pieces(index) {
            if context
                .requested
                .origin_map
                .view_origin(piece.origin)
                .is_some()
            {
                context::push(&mut result.pieces, piece)?;
            }
        }
        if start == result.pieces.len() {
            continue;
        }
        let mut own = row.clone();
        own.pieces = start..result.pieces.len();
        own.outputs = 0..0;
        own.style = context.local_style(&styles[row.style], own.source, output_styles)?;
        rows[index] = Some(result.rows.len());
        context::push(&mut result.rows, own)?;
    }
    for (index, component) in map.components.iter().enumerate() {
        let owner = match component.owner {
            ComponentOwner::Original(row) => rows[row].map(ComponentOwner::Original),
            ComponentOwner::Wbr(control) => context.local_control(control).map(ComponentOwner::Wbr),
            ComponentOwner::Virtual(index) => {
                if !virtual_owned(context, &map.virtuals[index]) {
                    None
                } else {
                    let mut own = map.virtuals[index].clone();
                    let old_scope = own.scope;
                    let scope = if let Some(scope) = scopes[old_scope] {
                        scope
                    } else {
                        let scope = result.scopes.len();
                        context::push(&mut result.scopes, map.scopes[old_scope].clone())?;
                        scopes[old_scope] = Some(scope);
                        scope
                    };
                    own.scope = scope;
                    own.control_cut = context.requested.scope_topology.as_ref().map_or(
                        own.control_cut,
                        |topology| {
                            topology
                                .control_orders
                                .iter()
                                .filter(|&&cut| cut < own.capture_cut)
                                .count()
                        },
                    );
                    let local = result.virtuals.len();
                    context::push(&mut result.virtuals, own)?;
                    Some(ComponentOwner::Virtual(local))
                }
            }
        };
        let Some(owner) = owner else { continue };
        let mut own = component.clone();
        own.owner = owner;
        own.outputs = 0..0;
        own.style =
            context.local_style(&styles[component.style], component.source, output_styles)?;
        components[index] = Some(result.components.len());
        context::push(&mut result.components, own)?;
    }
    let mut emissions = context::filled(map.emissions.len(), None)?;
    for (index, emission) in map.emissions.iter().enumerate() {
        let Some(component) = components[emission.component] else {
            continue;
        };
        let mut own = *emission;
        own.component = component;
        own.atom = 0;
        own.event = 0;
        emissions[index] = Some(result.emissions.len());
        context::push(&mut result.emissions, own)?;
    }
    for (index, component) in map.components.iter().enumerate() {
        if let Some(local) = components[index] {
            let start = component
                .outputs
                .clone()
                .find_map(|id| emissions[id])
                .unwrap_or_else(|| {
                    emissions[..component.outputs.start]
                        .iter()
                        .filter(|id| id.is_some())
                        .count()
                });
            let end = start
                + component
                    .outputs
                    .clone()
                    .filter(|&id| emissions[id].is_some())
                    .count();
            result.components[local].outputs = start..end;
            if let ComponentOwner::Original(row) = result.components[local].owner {
                result.rows[row].outputs = start..end;
            }
        }
    }
    let mut atoms = Vec::new();
    let mut kept_atoms = Vec::new();
    for event in &map.events {
        match *event {
            RelationEvent::Text { atom } => {
                let old = map.atoms[atom];
                let Some(emission) = emissions[old] else {
                    continue;
                };
                let local = atoms.len();
                result.emissions[emission].atom = local;
                result.emissions[emission].event = result.events.len();
                context::push(&mut result.atoms, emission)?;
                context::push(&mut kept_atoms, atom)?;
                context::push(&mut atoms, input.atoms[atom].clone())?;
                context::push(&mut result.events, RelationEvent::Text { atom: local })?;
            }
            RelationEvent::Removal { row } => {
                if let Some(row) = rows[row] {
                    context::push(&mut result.events, RelationEvent::Removal { row })?;
                }
            }
            RelationEvent::PhaseIDeleted { source, .. } => {
                if context
                    .requested
                    .contains(context.processed.source(), source)
                {
                    context::push(&mut result.events, *event)?;
                }
            }
            RelationEvent::Anchor { order } => {
                if let Some(order) = context.local_anchor(order) {
                    context::push(&mut result.events, RelationEvent::Anchor { order })?;
                }
            }
        }
    }
    for phrase in map.phrases {
        context::push(&mut result.phrases, phrase)?;
    }
    result.finish_cuts()?;
    let end = atoms.last().map_or(0, |atom| atom.presentation.end);
    let mut barriers = Vec::new();
    let mut cuts = Vec::new();
    for &source in &input.barriers {
        if context
            .requested
            .contains(context.processed.source(), Range::new(source, source))
        {
            context::push(&mut barriers, source)?;
        }
    }
    for &cut in &input.barrier_atoms {
        let local = kept_atoms.partition_point(|&atom| atom < cut);
        context::push(&mut cuts, local)?;
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut projection = SourceProjection::from_atoms_at_barriers(atoms, end, barriers, cuts)?;
    projection.style_rows = context::collect_rows(
        (0..projection.atoms.len()).map(|atom| Some(result.component_for_atom(atom).style)),
    )?;
    projection.anchor_atoms = context::filled(context.requested.anchor_order.len(), None)?;
    for (order, atom) in input.anchor_atoms.iter().enumerate() {
        if let (Some(local), Some(atom)) = (context.local_anchor(order), atom) {
            projection.anchor_atoms[local] = Some(kept_atoms.partition_point(|&old| old < *atom));
        }
    }
    let result = Arc::new(result);
    projection.transform = Some(Arc::clone(&result));
    Ok((projection, result))
}
