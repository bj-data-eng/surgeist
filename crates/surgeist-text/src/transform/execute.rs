use std::{ops::Range as IndexRange, sync::Arc};

use icu_normalizer::properties::CanonicalCombiningClassMap;
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, GeneralCategoryGroup, GraphemeClusterBreak, Script},
    script::ScriptWithExtensions,
};
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};

use super::{
    casing::{self, Locale, Mapper, MappingError, Operation},
    irish,
    relation::{ComponentOwner, Emission, RelationEvent, TransformMap},
};
use crate::{
    CaseTransform, Error, ErrorCode, ErrorDetail, ProcessedSource, Range, Result, TextTransform,
    TextTransformError,
    engine::{OriginalAtom, analysis::wrapping::Event, resolve::AnalyzedStyleRun},
    projection::SourceProjection,
    protocol::paragraph::ParagraphData,
};

fn failed(reason: MappingError) -> Error {
    let (code, reason) = match reason {
        MappingError::DataUnavailable => (
            ErrorCode::UnsupportedFeature,
            TextTransformError::CasingDataUnavailable,
        ),
        MappingError::InvalidException | MappingError::InvalidScalar => (
            ErrorCode::InvalidSourceProcessing,
            TextTransformError::InvalidCasingData,
        ),
        MappingError::InvalidLocale => (
            ErrorCode::InvalidStyle,
            TextTransformError::InvalidContentLocale,
        ),
        MappingError::Storage => (
            ErrorCode::SourceProcessingCapacity,
            TextTransformError::Capacity,
        ),
    };
    Error::new(
        code,
        "text transformation could not retain its actual mapping",
    )
    .with_detail(ErrorDetail::TextTransform { reason })
}

#[derive(Clone, Copy, Default)]
struct Initial {
    title: bool,
    context: bool,
}

#[derive(Clone, Copy, Default)]
struct PrefixEffect {
    preserve_upper: bool,
    remove_upper: bool,
    insert_lower: bool,
}

fn base(input: &SourceProjection, unit: IndexRange<usize>) -> Option<usize> {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    if input.atoms[unit.clone()].iter().any(|atom| {
        categories.get32(atom.code_point) == GeneralCategory::EnclosingMark
            && scripts.has_script32(atom.code_point, Script::Common)
    }) {
        return None;
    }
    let gcb = CodePointMapData::<GraphemeClusterBreak>::new();
    let base = unit
        .clone()
        .find(|&row| gcb.get32(input.atoms[row].code_point) != GraphemeClusterBreak::Prepend)?;
    let category = categories.get32(input.atoms[base].code_point);
    if category == GeneralCategory::SpaceSeparator && unit.len() > 1 {
        return None;
    }
    (GeneralCategoryGroup::Letter.contains(category)
        || GeneralCategoryGroup::Number.contains(category))
    .then_some(base)
}

fn classify(
    input: &SourceProjection,
    values: &[u32],
    domain: IndexRange<usize>,
    mapper: &Mapper,
    initial: &mut [Initial],
    prefixes: &mut [PrefixEffect],
) -> Result<()> {
    let point = |atom: usize| {
        input
            .atoms
            .get(atom)
            .map_or(input.presentation_end(), |atom| atom.presentation.start)
    };
    let byte = input.segmentation_byte(point(domain.start));
    let end = input.segmentation_byte(point(domain.end));
    let atom_at = |byte| -> Result<usize> {
        let point = input.segmentation_unit(byte)?;
        Ok(input
            .atoms
            .partition_point(|atom| atom.presentation.start < point))
    };
    let mut previous = domain.start;
    for (offset, kind) in
        WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default())
            .segment_str(&input.segmentation[byte..end])
            .iter_with_word_type()
    {
        let next = atom_at(byte + offset)?;
        if previous < next && kind.is_word_like() {
            let word = previous..next;
            let prefix = irish::prefix(&values[word.start..domain.end]);
            for effect in &mut prefixes[word.start..(word.start + prefix.preserve).min(domain.end)]
            {
                effect.preserve_upper = true;
            }
            if let Some(row) = prefix
                .remove
                .and_then(|row| word.start.checked_add(row))
                .filter(|&row| row < domain.end)
            {
                prefixes[row].remove_upper = true;
            }
            if let Some(row) = prefix
                .insert_before
                .and_then(|row| word.start.checked_add(row))
                .filter(|&row| row < domain.end)
            {
                prefixes[row].insert_lower = true;
            }
            // Seek the first EGC whose end is after this word start. Keep
            // a unit beginning before the word: its base/context still owns
            // the initial-unit decision when the ranges genuinely overlap.
            let word_start = point(word.start);
            let word_end = point(word.end);
            let grapheme = input
                .graphemes
                .partition_point(|&edge| edge <= word_start)
                .saturating_sub(1);
            for pair in input.graphemes[grapheme..]
                .windows(2)
                .take_while(|pair| pair[0] < word_end)
            {
                let first = input
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[0]);
                let last = input
                    .atoms
                    .partition_point(|atom| atom.presentation.start < pair[1]);
                if last <= word.start || first >= word.end {
                    continue;
                }
                let Some(base) = base(input, first..last) else {
                    continue;
                };
                // The first Letter/Number unit occupies this slot, even when
                // it is a number, uppercase or caseless letter.
                if mapper.case_type(values[base])
                    == Some(icu_casemap::provider::data::CaseType::Lower)
                {
                    initial[base].title = true;
                    for (row, unit) in initial.iter_mut().enumerate().take(last).skip(first) {
                        if row != base {
                            unit.context = true;
                        }
                    }
                    for row in base + 1..word.end {
                        if matches!(values[row], 0x6a | 0x4a)
                            && mapper
                                .dutch_j(&values[word.start..row], &values[row + 1..word.end])
                                .map_err(failed)?
                        {
                            initial[row].context = true;
                        }
                    }
                }
                break;
            }
        }
        previous = next;
    }
    Ok(())
}

pub(super) fn apply(
    input: SourceProjection,
    styles: &mut Vec<AnalyzedStyleRun>,
    processed: &ProcessedSource,
    paragraph: &ParagraphData,
) -> Result<(SourceProjection, Arc<TransformMap>)> {
    let captured =
        crate::engine::analysis::wrapping::transform_events(processed, paragraph, &input);
    let mut map = TransformMap::identity(&input, styles, processed, paragraph, captured.clone());
    let proposals =
        super::separators::automatic::proposals(&input, styles, processed, paragraph, &mut map)?;
    let input = super::separators::apply(input, &mut map, &captured, styles, processed, paragraph)?;
    let (input, captured) = super::separators::wbr(
        input, &mut map, captured, styles, processed, paragraph, proposals,
    )?;
    let values: Vec<_> = input.atoms.iter().map(|atom| atom.code_point).collect();
    let casing = map
        .components
        .iter()
        .any(|row| matches!(styles[row.style].style.text_transform(), TextTransform::Set(set) if set.case() != CaseTransform::None));
    let mapper = casing.then(Mapper::new).transpose().map_err(failed)?;
    let mut stops = input.barrier_atoms.clone();
    stops.extend([0, input.atoms.len()]);
    for (row, &cp) in values.iter().enumerate() {
        if crate::engine::analysis::is_hard_separator(cp) {
            stops.extend([row, row + 1]);
        }
    }
    stops.sort_unstable();
    stops.dedup();
    let mut initial = vec![Initial::default(); values.len()];
    let mut prefixes = vec![PrefixEffect::default(); values.len()];
    if let Some(mapper) = &mapper {
        for domain in stops.windows(2) {
            classify(
                &input,
                &values,
                domain[0]..domain[1],
                mapper,
                &mut initial,
                &mut prefixes,
            )?;
        }
    }
    let mut locales = vec![None; styles.len()];
    let mut operations = Vec::new();
    let mut output = Vec::new();
    let mut spans = Vec::new();
    for domain in stops.windows(2) {
        for row in domain[0]..domain[1] {
            let style = &styles[map.components[row].style].style;
            let transform = style.text_transform();
            let operation = match transform {
                TextTransform::MathAuto => None,
                TextTransform::Set(set) => match set.case() {
                    CaseTransform::None => None,
                    CaseTransform::Uppercase => Some(Operation::Upper),
                    CaseTransform::Lowercase => Some(Operation::Lower),
                    CaseTransform::Capitalize if initial[row].title => Some(Operation::Title),
                    CaseTransform::Capitalize if initial[row].context => {
                        Some(Operation::TitleContext)
                    }
                    CaseTransform::Capitalize => None,
                },
            };
            operations.push(operation);
            let start = output.len();
            let locale = if operation.is_some() {
                let slot = &mut locales[map.components[row].style];
                if slot.is_none() {
                    *slot = Some(Locale::from_tag(style.locale_tag()).map_err(failed)?);
                }
                slot.expect("resolved actual component locale")
            } else {
                Locale::Root
            };
            output
                .try_reserve(1)
                .map_err(|_| failed(MappingError::Storage))?;
            if locale == Locale::Irish
                && operation == Some(Operation::Upper)
                && prefixes[row].remove_upper
            {
                // The genuine hyphen row has a zero-output correspondence.
            } else if operation.is_none()
                || locale == Locale::Irish
                    && operation == Some(Operation::Upper)
                    && prefixes[row].preserve_upper
            {
                output.push(values[row]);
            } else {
                if locale == Locale::Irish
                    && operation == Some(Operation::Lower)
                    && prefixes[row].insert_lower
                {
                    output.push(0x2d);
                }
                mapper
                    .as_ref()
                    .expect("actual casing operation")
                    .map_into(
                        &values[domain[0]..domain[1]],
                        row - domain[0],
                        operation.expect("casing operation"),
                        locale,
                        &mut output,
                    )
                    .map_err(failed)?;
            }
            if transform == TextTransform::MathAuto {
                let pieces = match map.components[row].owner {
                    ComponentOwner::Original(original) => map.original_pieces(original),
                    ComponentOwner::Wbr(_) | ComponentOwner::Virtual(_) => &[],
                };
                if !pieces.is_empty()
                    && pieces.iter().all(|piece| {
                        paragraph.origins[piece.origin]
                            .text_node
                            .as_ref()
                            .is_some_and(|node| super::math::eligible(node.text()))
                    })
                    && let Some(target) = super::math::map(values[row])
                {
                    output[start] = target;
                }
            }
            if let TextTransform::Set(set) = transform {
                if set.full_width() {
                    for cp in &mut output[start..] {
                        if (*cp != 0x20
                            || map.components[row].separator_space
                            || processed.preserves_space_at(map.components[row].processed.start))
                            && let Some(target) = super::width::map(*cp)
                        {
                            *cp = target;
                        }
                    }
                }
                if set.full_size_kana() {
                    for cp in &mut output[start..] {
                        if let Some(target) = super::full_size_kana(*cp) {
                            *cp = u32::from(target);
                        }
                    }
                }
            }
            spans.push(start..output.len());
        }
    }
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
    for (row, span) in spans.iter().enumerate() {
        let start = map.emissions.len();
        for (subposition, &code_point) in output[span.clone()].iter().enumerate() {
            map.emissions.push(Emission {
                component: row,
                subposition,
                code_point,
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
                    range.clone().last().map(|id| map.emissions[id].code_point),
                );
                for id in range.clone() {
                    if postpone && id + 1 == range.end {
                        pending.push(id);
                    } else {
                        emit(id, &mut atoms, &mut map)?;
                    }
                }
            }
            Event::Deleted(source, processed) => map
                .events
                .push(RelationEvent::PhaseIDeleted { source, processed }),
            Event::Anchor(anchor) => map.events.push(RelationEvent::Anchor {
                order: paragraph
                    .anchor_order
                    .iter()
                    .position(|value| *value == anchor)
                    .expect("captured actual anchor"),
            }),
        }
    }
    flush(&mut pending, &mut atoms, &mut map)?;
    map.finish_cuts();
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
    projection.anchor_atoms = vec![None; paragraph.anchor_order.len()];
    let mut at = 0;
    for event in &map.events {
        match *event {
            RelationEvent::Text { atom } => {
                if let ComponentOwner::Wbr(control) = map.component_for_atom(atom).owner {
                    let order = paragraph
                        .anchor_order
                        .iter()
                        .position(|anchor| {
                            anchor.item == crate::ParagraphAnchorItem::Control(control)
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
    let map = Arc::new(map);
    projection.transform = Some(Arc::clone(&map));
    Ok((projection, map))
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
