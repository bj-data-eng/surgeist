//! Fixed-partition SPACE replay and source-only horizontal inner edge use.
use super::*;
use crate::processing::lexical::{SpaceClass, SpaceCollapse};
use crate::transform::relation::{Component, ComponentOwner, Emission, InputRow, RelationEvent};
use icu_properties::{
    CodePointMapData, CodePointSetData,
    props::{BidiControl, BidiMirroringGlyph},
};

fn class(part: &Part) -> SpaceClass {
    if CodePointSetData::new::<BidiControl>().contains32(part.identity) {
        SpaceClass::Transparent
    } else if part.identity == 0x20 && part.mode.collapses_spaces() {
        SpaceClass::Collapsible
    } else {
        SpaceClass::Other
    }
}

fn inner_class(part: &Part) -> icu_properties::props::BidiClass {
    if crate::engine::analysis::is_hard_separator(part.identity) {
        icu_properties::props::BidiClass::BoundaryNeutral
    } else {
        crate::engine::analysis::AnalysisDataSources
            .original_properties(part.normal)
            .bidi_class()
    }
}

#[derive(Clone, Copy)]
enum InnerInput {
    Part(usize),
    Anchor(usize),
}

struct InnerInputs {
    inputs: Vec<InnerInput>,
    part_positions: Vec<Option<usize>>,
}

impl InnerInputs {
    fn new(canonical: &Canonical<'_>, plan: &CombinedSourcePlan) -> Result<Self> {
        let mut result = Self {
            inputs: Vec::new(),
            part_positions: context::filled(plan.parts.len(), None)?,
        };
        for event in &canonical.map.events {
            match event {
                RelationEvent::Text { atom } => {
                    if let Some(part) = canonical.map.emission_for_atom(*atom).selection_part {
                        result.part(part)?;
                    }
                }
                RelationEvent::PhaseIDeleted { lexical, .. } => {
                    for (record, _) in
                        lexical.indexed_occurrences(&canonical.context.processed.lexical)
                    {
                        if let Some(part) = plan.lexical_parts[record] {
                            result.part(part)?;
                        }
                    }
                }
                RelationEvent::Anchor { order } => {
                    context::push(&mut result.inputs, InnerInput::Anchor(*order))?;
                }
                _ => (),
            }
        }
        Ok(result)
    }

    fn part(&mut self, part: usize) -> Result<()> {
        if self.part_positions[part].is_none() {
            self.part_positions[part] = Some(self.inputs.len());
            context::push(&mut self.inputs, InnerInput::Part(part))?;
        }
        Ok(())
    }

    fn group(&self, group: &FixedGroup) -> &[InnerInput] {
        let first = self.part_positions[group.parts.start].expect("actual first cell part");
        let last = self.part_positions[group.parts.end - 1].expect("actual last cell part");
        &self.inputs[first..=last]
    }
}

impl InnerInput {
    fn properties(
        self,
        parts: &[Part],
        source: &context::SourceContext<'_>,
    ) -> Option<(icu_properties::props::BidiClass, Option<u32>)> {
        use crate::protocol::paragraph::SemanticAnchorItem;
        use icu_properties::props::BidiClass;
        match self {
            Self::Part(part) => Some((inner_class(&parts[part]), Some(parts[part].normal))),
            Self::Anchor(order) => match source.anchor_order[order].item {
                SemanticAnchorItem::Control(control) => {
                    let kind = &source.controls[control].kind;
                    if *kind == crate::ParagraphControlKind::ForcedBreak {
                        Some((BidiClass::BoundaryNeutral, None))
                    } else {
                        kind.code_point().map(|cp| {
                            (
                                crate::engine::analysis::AnalysisDataSources
                                    .original_properties(cp)
                                    .bidi_class(),
                                Some(cp),
                            )
                        })
                    }
                }
                SemanticAnchorItem::ComputedLineBoundary(_) => {
                    Some((BidiClass::BoundaryNeutral, None))
                }
                SemanticAnchorItem::Atomic(_) | SemanticAnchorItem::EmptyOrigin(_) => None,
            },
        }
    }
}

fn inner_levels(
    parts: &[Part],
    inputs: &[InnerInput],
    source: &context::SourceContext<'_>,
    direction: crate::ParagraphBaseDirection,
) -> Result<Vec<u8>> {
    let entries = context::collect_rows(inputs.iter().filter_map(|&input| {
        input
            .properties(parts, source)
            .map(|(class, cp)| (input, class, cp))
    }))?;
    let brackets = CodePointMapData::<BidiMirroringGlyph>::new();
    let mut bidi = crate::engine::bidi::BidiResolver::new();
    bidi.resolve(
        entries.iter().map(|&(_, class, cp)| {
            (
                class,
                cp.and_then(char::from_u32)
                    .map(|character| (character, brackets.get32(character as u32))),
            )
        }),
        Some(direction.level()),
    );
    let mut levels = context::collect_rows(bidi.levels().iter().copied())?;
    let reset = context::collect_rows(
        entries
            .iter()
            .map(|&(_, class, _)| crate::engine::bidi::LineReset::from_class(class)),
    )?;
    crate::engine::bidi::reset_line_levels(&mut levels, &reset, direction.level());
    context::collect_rows(
        entries
            .iter()
            .zip(levels)
            .filter_map(|((input, _, _), level)| {
                matches!(input, InnerInput::Part(_)).then_some(level)
            }),
    )
}

pub(super) fn complete(canonical: &mut Canonical<'_>, plan: &mut CombinedSourcePlan) -> Result<()> {
    // Rejected candidates never alter ordinary processing. In particular this
    // preserves exact Phase-I visibility when the complete plan has no cells.
    if plan.groups.is_empty() {
        return Ok(());
    }
    let mut group_at = context::filled(plan.parts.len(), None)?;
    for (index, group) in plan.groups.iter().enumerate() {
        for part in group.parts.clone() {
            group_at[part] = Some(index);
        }
    }
    let mut state = SpaceCollapse::new();
    let mut previous = None;
    let mut stops = 0;
    let mut unknown = 0;
    for (index, &group) in group_at.iter().enumerate() {
        if group != previous || plan.fences.binary_search(&index).is_ok() {
            state = SpaceCollapse::new();
        }
        while plan
            .replay_stops
            .get(stops)
            .is_some_and(|stop| stop.0 <= index)
        {
            let (_, suppressible) = plan.replay_stops[stops];
            if !(suppressible && group.is_some() && group == previous) {
                state = SpaceCollapse::new();
            }
            stops += 1;
        }
        while plan
            .replay_unknown
            .get(unknown)
            .is_some_and(|edge| edge.0 <= index)
        {
            let (_, before, after) = plan.replay_unknown[unknown];
            state.missing(before, after);
            unknown += 1;
        }
        previous = group;
        let class = if group.is_some()
            && crate::engine::analysis::is_hard_separator(plan.parts[index].identity)
        {
            SpaceClass::Transparent
        } else {
            class(&plan.parts[index])
        };
        if let Some(visible) = state.step(class)? {
            plan.visible[index] = visible;
        }
    }
    let inner = InnerInputs::new(canonical, plan)?;
    for group in &plan.groups {
        let mut ids = Vec::new();
        for part in group.parts.clone() {
            // A replay-zero SPACE remains a real semantic unit and takes part
            // in inner bidi/edges. Only presentation emission is absent.
            context::push(&mut ids, part)?;
        }
        // Canonical mixed-event incidence supplies controls strictly between
        // the first and last owned parts; exterior anchors stay exterior.
        let mut inputs = context::collect_rows(inner.group(group).iter().copied())?;
        let levels = inner_levels(
            &plan.parts,
            &inputs,
            &canonical.context,
            group.inner_direction,
        )?;
        let bidi_control = CodePointSetData::new::<BidiControl>();
        let mut slots = Vec::new();
        let mut slot_parts = Vec::new();
        let initial_units = selection::used_units(&plan.parts, ids.iter().copied())?;
        let mut ogham = context::filled(ids.len(), false)?;
        for unit in initial_units {
            if unit
                .clone()
                .all(|position| plan.parts[ids[position]].normal == 0x1680)
            {
                for position in unit {
                    ogham[position] = true;
                }
            }
        }
        for (position, &part) in ids.iter().enumerate() {
            let value = &plan.parts[part];
            if bidi_control.contains32(value.identity)
                || crate::engine::analysis::is_hard_separator(value.identity)
            {
                plan.visible[part] = false;
                plan.retained[part] = false;
                continue;
            }
            let edge = crate::engine::analysis::edge::whitespace_edge(
                value.identity == 0x20,
                ogham[position],
                value.mode,
                true,
            );
            context::push(
                &mut slots,
                crate::SuppliedInlineParticipant::Text {
                    level: crate::BidiLevel::try_new(levels[position])
                        .map_err(|_| context::capacity())?,
                    edge,
                },
            )?;
            context::push(&mut slot_parts, part)?;
        }
        drop(levels);
        drop(ids);
        let edges = crate::SuppliedLine::new(&slots).edges();
        for (slot, &part) in slot_parts.iter().enumerate() {
            if edges.is_discarded(slot) == Some(true) {
                plan.visible[part] = false;
                plan.retained[part] = false;
            }
        }
        let retained =
            context::collect_rows(group.parts.clone().filter(|&part| plan.retained[part]))?;
        let presentation_parts =
            context::collect_rows(retained.iter().copied().filter(|&part| plan.visible[part]))?;
        let units = selection::used_units(&plan.parts, retained.iter().copied())?;
        inverse_width(&units, &retained, &mut plan.parts);
        // Edge selection uses the pre-inversion sequence above. Final levels
        // describe the actual used scalars, retaining source bidi controls and
        // the ignored forced-source BN entries in their genuine order.
        inputs.retain(|input| match *input {
            InnerInput::Part(part) => {
                plan.retained[part]
                    || bidi_control.contains32(plan.parts[part].identity)
                    || crate::engine::analysis::is_hard_separator(plan.parts[part].identity)
            }
            InnerInput::Anchor(_) => true,
        });
        let levels = inner_levels(
            &plan.parts,
            &inputs,
            &canonical.context,
            group.inner_direction,
        )?;
        let used_levels = context::collect_rows(
            inputs
                .iter()
                .filter_map(|input| match *input {
                    InnerInput::Part(part) => Some(part),
                    InnerInput::Anchor(_) => None,
                })
                .zip(levels)
                .filter_map(|(part, level)| plan.retained[part].then_some(level)),
        )?;
        let payload = UsedPayload {
            parts: retained,
            presentation_parts,
            levels: used_levels,
            units,
        };
        context::push(&mut plan.used, payload)?;
    }
    rewrite(canonical, plan)
}

/// Cardinality comes from the actual post-edge used units, independently of
/// the earlier source-selection EGC count.
fn inverse_width(units: &[IndexRange<usize>], retained: &[usize], parts: &mut [Part]) {
    if units.len() <= 1 {
        return;
    }
    for &id in retained {
        if let Some(cp) = crate::transform::width::inverse(parts[id].normal) {
            parts[id].normal = cp;
        }
    }
}

fn rewrite(canonical: &mut Canonical<'_>, plan: &CombinedSourcePlan) -> Result<()> {
    let mut part_for_emission = context::filled(canonical.map.emissions.len(), None)?;
    let mut part_for_lexical = match &canonical.context.processed.lexical {
        crate::processing::lexical_inventory::LexicalState::Executed { records, .. } => {
            context::filled(records.len(), None)?
        }
        crate::processing::lexical_inventory::LexicalState::Untransformed => Vec::new(),
    };
    for (index, part) in plan.parts.iter().enumerate() {
        match part.reference {
            SelectionPart::Emission(id) => part_for_emission[id] = Some(index),
            SelectionPart::CollapsedSpace(id) => part_for_lexical[id] = Some(index),
        }
    }
    let mut emissions = Vec::new();
    let mut translated = context::filled(canonical.map.emissions.len(), None)?;
    for component in &mut canonical.map.components {
        let first = emissions.len();
        for id in component.outputs.clone() {
            let Some(part) = part_for_emission[id] else {
                continue;
            };
            if !plan.visible[part] {
                continue;
            }
            translated[id] = Some(emissions.len());
            let old = canonical.map.emissions[id];
            context::push(
                &mut emissions,
                Emission {
                    code_point: plan.parts[part].normal,
                    ..old
                },
            )?;
        }
        component.outputs = first..emissions.len();
        if let ComponentOwner::Original(row) = component.owner {
            canonical.map.rows[row].outputs = component.outputs.clone();
        }
    }
    let mut lexical_emissions = context::filled(part_for_lexical.len(), None)?;
    for (part_index, part) in plan.parts.iter().enumerate() {
        let SelectionPart::CollapsedSpace(record) = part.reference else {
            continue;
        };
        if !plan.visible[part_index] {
            continue;
        }
        let first_piece = canonical.map.pieces.len();
        for &piece in &part.pieces {
            context::push(&mut canonical.map.pieces, piece)?;
        }
        let row = canonical.map.rows.len();
        let component = canonical.map.components.len();
        let id = emissions.len();
        let style =
            crate::transform::context::source_style(&canonical.source_styles, part.source.start);
        context::push(
            &mut canonical.map.rows,
            InputRow {
                source: part.source,
                code_point: 0x20,
                separator_space: false,
                style,
                pieces: first_piece..canonical.map.pieces.len(),
                outputs: id..id + 1,
            },
        )?;
        context::push(
            &mut canonical.map.components,
            Component {
                owner: ComponentOwner::Original(row),
                source: part.source,
                processed: part.processed,
                style,
                outputs: id..id + 1,
            },
        )?;
        context::push(
            &mut emissions,
            Emission {
                selection_part: Some(part_index),
                component,
                subposition: 0,
                code_point: part.normal,
                selection_identity: 0x20,
                atom: 0,
                event: 0,
            },
        )?;
        lexical_emissions[record] = Some(id);
    }
    let old_events = std::mem::take(&mut canonical.map.events);
    let old_atoms = std::mem::take(&mut canonical.map.atoms);
    let old_emissions = std::mem::replace(&mut canonical.map.emissions, emissions);
    let mut lexical_done = context::filled(lexical_emissions.len(), false)?;
    let mut atoms = Vec::new();
    let mut anchors = context::filled(canonical.context.anchor_order.len(), None)?;
    let mut presentation = 0usize;
    for event in old_events {
        match event {
            RelationEvent::Text { atom } => {
                let old_id = old_atoms[atom];
                if let Some(id) = translated[old_id] {
                    emit(id, &mut atoms, &mut canonical.map, &mut presentation)?;
                } else if let ComponentOwner::Original(row) =
                    canonical.map.components[old_emissions[old_id].component].owner
                {
                    context::push(
                        &mut canonical.map.events,
                        RelationEvent::Removal {
                            row,
                            selection_part: part_for_emission[old_id],
                        },
                    )?;
                } else if let Some(selection_part) = part_for_emission[old_id] {
                    context::push(
                        &mut canonical.map.events,
                        RelationEvent::GeneratedSuppression {
                            component: old_emissions[old_id].component,
                            selection_part,
                        },
                    )?;
                }
            }
            RelationEvent::PhaseIDeleted { lexical, .. } => {
                context::push(&mut canonical.map.events, event)?;
                for (record, _) in lexical.indexed_occurrences(&canonical.context.processed.lexical)
                {
                    if lexical_done[record] {
                        continue;
                    }
                    lexical_done[record] = true;
                    if let Some(id) = lexical_emissions[record] {
                        emit(id, &mut atoms, &mut canonical.map, &mut presentation)?;
                    }
                }
            }
            RelationEvent::Anchor { order } => {
                anchors[order] = Some(atoms.len());
                context::push(&mut canonical.map.events, event)?;
            }
            _ => context::push(&mut canonical.map.events, event)?,
        }
    }
    let mut prefix = context::filled(
        plan.parts
            .len()
            .checked_add(1)
            .ok_or_else(context::capacity)?,
        0usize,
    )?;
    for index in 0..plan.parts.len() {
        prefix[index + 1] = prefix[index] + usize::from(plan.visible[index]);
    }
    let fences = context::collect_rows(
        plan.semantic_fences
            .iter()
            .copied()
            .filter(|&cut| {
                let within_cell = cut
                    .checked_sub(1)
                    .and_then(|before| plan.group_for_part[before])
                    .is_some_and(|group| {
                        plan.group_for_part.get(cut).copied().flatten() == Some(group)
                    });
                // A genuine disconnection, block or atomic fence is never consumed.
                // Other hard effects strictly inside this fixed cell are its actual
                // suppressed forced controls, not an exterior paragraph boundary.
                !within_cell || plan.fences.binary_search(&cut).is_ok()
            })
            .map(|cut| prefix[cut]),
    )?;
    let mut projection = crate::projection::SourceProjection::from_atoms_at_barriers(
        atoms,
        presentation,
        canonical.projection.barriers.clone(),
        fences,
    )?;
    projection.anchor_atoms = anchors;
    projection.style_rows = context::collect_rows(
        (0..projection.atoms.len()).map(|atom| Some(canonical.map.component_for_atom(atom).style)),
    )?;
    canonical.map.finish_cuts()?;
    canonical.projection = projection;
    Ok(())
}
fn emit(
    id: usize,
    atoms: &mut Vec<crate::engine::OriginalAtom>,
    map: &mut crate::transform::relation::TransformMap,
    end: &mut usize,
) -> Result<()> {
    let emission = &mut map.emissions[id];
    let component = &map.components[emission.component];
    let glyph = char::from_u32(emission.code_point).unwrap_or('\u{fffd}');
    let length = if (0xd800..=0xdfff).contains(&emission.code_point) {
        1
    } else {
        glyph.len_utf16()
    };
    let next = end
        .checked_add(length)
        .ok_or_else(crate::projection::length_overflow)?;
    emission.atom = atoms.len();
    emission.event = map.events.len();
    context::push(&mut map.atoms, id)?;
    context::push(&mut map.events, RelationEvent::Text { atom: atoms.len() })?;
    context::push(
        atoms,
        crate::engine::OriginalAtom {
            source: component.source,
            processed: component.processed,
            presentation: Range::new(*end, next),
            glyph_bytes: 0..0,
            code_point: emission.code_point,
            glyph,
        },
    )?;
    *end = next;
    Ok(())
}
