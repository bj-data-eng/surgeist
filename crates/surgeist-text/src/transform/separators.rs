//! Original U+200B expansion over the actual post-Phase-I mixed stream.
use super::relation::TransformMap;
pub(super) mod automatic;
use crate::engine::{
    analysis::wrapping::{Event, LogicalEvent},
    resolve::AnalyzedStyleRun,
};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{ParagraphAnchorItem, ParagraphControlKind, ParagraphData};
use crate::{
    Error, ErrorCode, ErrorDetail, InlineBoxKind, Result, TextTransformError, WordSpaceTransform,
};

/// None is genuinely transparent; Some records the next actual item's forced role.
fn forced(
    event: &LogicalEvent,
    input: &SourceProjection,
    source: &crate::Source,
    paragraph: &ParagraphData,
) -> Option<bool> {
    match event.kind {
        Event::Text(atom) => Some(crate::engine::analysis::is_hard_separator(
            input.atoms[atom].code_point,
        )),
        Event::Deleted(..) => None,
        Event::Anchor(anchor) => match anchor.item {
            ParagraphAnchorItem::EmptyOrigin(_) => None,
            ParagraphAnchorItem::Atomic(i) => {
                (source.boxes()[i].kind() == InlineBoxKind::InFlow).then_some(false)
            }
            ParagraphAnchorItem::Control(i) => match paragraph.controls[i].kind {
                ParagraphControlKind::InlineBoundaryStart
                | ParagraphControlKind::InlineBoundaryEnd => None,
                ParagraphControlKind::ForcedBreak => Some(true),
                _ => Some(false),
            },
        },
    }
}

pub(super) fn apply(
    mut input: SourceProjection,
    map: &mut TransformMap,
    events: &[LogicalEvent],
    styles: &[AnalyzedStyleRun],
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphData,
) -> Result<SourceProjection> {
    if !map.rows.iter().any(|row| {
        row.code_point == 0x200b
            && styles[row.style].style.authored().word_space_transform != WordSpaceTransform::None
    }) {
        return Ok(input);
    }
    let mut adjacent = Vec::new();
    adjacent.try_reserve_exact(input.atoms.len()).map_err(|_| {
        Error::new(
            ErrorCode::SourceProcessingCapacity,
            "separator adjacency storage unavailable",
        )
        .with_detail(ErrorDetail::TextTransform {
            reason: TextTransformError::Capacity,
        })
    })?;
    adjacent.resize(input.atoms.len(), false);
    let mut before = false;
    for event in events {
        if let Event::Text(atom) = event.kind {
            adjacent[atom] |= before;
        }
        if let Some(is_forced) = forced(event, &input, processed.source(), paragraph) {
            before = is_forced;
        }
    }
    let mut after = false;
    for event in events.iter().rev() {
        if let Event::Text(atom) = event.kind {
            adjacent[atom] |= after;
        }
        if let Some(is_forced) = forced(event, &input, processed.source(), paragraph) {
            after = is_forced;
        }
    }
    let mut changed = false;
    for (row, atom) in input.atoms.iter_mut().enumerate() {
        if atom.code_point != 0x200b || adjacent[row] {
            continue;
        }
        let Some(cp) = styles[map.rows[row].style]
            .style
            .authored()
            .word_space_transform
            .replacement()
        else {
            continue;
        };
        map.rows[row].separator_space = cp == 0x20;
        let cp = char::from_u32(cp).expect("separator replacement is a scalar");
        atom.code_point = u32::from(cp);
        atom.glyph = cp;
        changed = true;
    }
    if !changed {
        return Ok(input);
    }
    let end = input.presentation_end();
    // Both replacements retain one UTF-16 unit and one genuine input row.
    // Rebuild semantic strings/graphemes before any casing decision is made.
    let mut result = SourceProjection::from_atoms_at_barriers(
        input.atoms,
        end,
        input.barriers,
        input.barrier_atoms,
    )?;
    result.anchor_atoms = input.anchor_atoms;
    result.style_rows = input.style_rows;
    Ok(result)
}

fn capacity() -> Error {
    Error::new(
        ErrorCode::SourceProcessingCapacity,
        "wbr semantic storage unavailable",
    )
    .with_detail(ErrorDetail::TextTransform {
        reason: TextTransformError::Capacity,
    })
}

/// Retain actual contextual connection order independently from numeric decode
/// barriers. Direct input has no same-cut anchor-side authority.
fn connection_cuts(
    input: &SourceProjection,
    events: &[LogicalEvent],
    paragraph: &ParagraphData,
) -> Result<Vec<usize>> {
    use crate::ParagraphConnectionKind;
    let mut control_events = Vec::new();
    control_events
        .try_reserve_exact(paragraph.controls.len())
        .map_err(|_| capacity())?;
    control_events.resize(paragraph.controls.len(), 0);
    let atomic_count = paragraph
        .anchor_order
        .iter()
        .filter_map(|a| match a.item {
            ParagraphAnchorItem::Atomic(i) => Some(i),
            _ => None,
        })
        .max()
        .map_or(Ok(0), |i| i.checked_add(1).ok_or_else(capacity))?;
    let mut atomic_events = Vec::new();
    atomic_events
        .try_reserve_exact(atomic_count)
        .map_err(|_| capacity())?;
    atomic_events.resize(atomic_count, 0);
    let mut empty_events = Vec::new();
    empty_events
        .try_reserve_exact(paragraph.origins.len())
        .map_err(|_| capacity())?;
    empty_events.resize(paragraph.origins.len(), 0);
    let mut text_edges = Vec::new();
    text_edges
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    for (index, event) in events.iter().enumerate() {
        match event.kind {
            Event::Anchor(anchor) => match anchor.item {
                ParagraphAnchorItem::Control(i) => control_events[i] = index + 1,
                ParagraphAnchorItem::Atomic(i) => atomic_events[i] = index + 1,
                ParagraphAnchorItem::EmptyOrigin(i) => empty_events[i] = index + 1,
            },
            Event::Text(i) => text_edges.push((input.atoms[i].source.end, index + 1)),
            Event::Deleted(range, _) => text_edges.push((range.end, index + 1)),
        }
    }
    text_edges.sort_unstable();
    let mut max = 0;
    for (_, event) in &mut text_edges {
        max = max.max(*event);
        *event = max;
    }
    let mut anchor_cuts = Vec::new();
    anchor_cuts
        .try_reserve_exact(
            paragraph
                .anchor_order
                .len()
                .checked_add(1)
                .ok_or_else(capacity)?,
        )
        .map_err(|_| capacity())?;
    anchor_cuts.push(0);
    for anchor in &paragraph.anchor_order {
        let event = match anchor.item {
            ParagraphAnchorItem::Control(i) => control_events[i],
            ParagraphAnchorItem::Atomic(i) => atomic_events[i],
            ParagraphAnchorItem::EmptyOrigin(i) => empty_events[i],
        };
        anchor_cuts.push(anchor_cuts.last().copied().unwrap().max(event));
    }
    let mut cuts = Vec::new();
    cuts.try_reserve_exact(paragraph.connections.len())
        .map_err(|_| capacity())?;
    for (index, connection) in paragraph.connections.iter().enumerate() {
        if connection.kind != ParagraphConnectionKind::Disconnected {
            continue;
        }
        let before = paragraph
            .connection_ends
            .as_ref()
            .map_or(connection.at, |ends| ends[index].0);
        let text = text_edges.partition_point(|(end, _)| *end <= before);
        let text_cut = text.checked_sub(1).map_or(0, |i| text_edges[i].1);
        let anchor_cut = paragraph
            .connection_anchor_cuts
            .as_ref()
            .map_or(0, |ranks| anchor_cuts[ranks[index]]);
        cuts.push(text_cut.max(anchor_cut));
    }
    cuts.sort_unstable();
    cuts.dedup();
    Ok(cuts)
}

/// A real source-less wbr emission joins the same semantic input as original
/// rows; its component is not an authored scalar or a replacement definition.
pub(super) fn wbr(
    input: SourceProjection,
    map: &mut TransformMap,
    events: Vec<LogicalEvent>,
    styles: &[AnalyzedStyleRun],
    processed: &crate::ProcessedSource,
    paragraph: &ParagraphData,
    proposals: Vec<automatic::Proposal>,
) -> Result<(SourceProjection, Vec<LogicalEvent>)> {
    use super::relation::{Component, ComponentOwner};
    for (row, atom) in input.atoms.iter().enumerate() {
        map.components[row].code_point = atom.code_point;
        map.components[row].separator_space = map.rows[row].separator_space;
    }
    if proposals.is_empty()
        && !paragraph
            .controls
            .iter()
            .any(|control| matches!(control.kind, ParagraphControlKind::Wbr { .. }))
    {
        return Ok((input, events));
    }
    let events = crate::engine::analysis::wrapping::separator_events(processed, paragraph, &input);
    let mut adjacent = Vec::new();
    adjacent
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    adjacent.resize(events.len(), false);
    let connection_cuts = connection_cuts(&input, &events, paragraph)?;
    let mut neighbor = false;
    for (index, event) in events.iter().enumerate() {
        if connection_cuts.binary_search(&index).is_ok() {
            neighbor = false;
        }
        adjacent[index] |= neighbor;
        if let Some(value) = forced(event, &input, processed.source(), paragraph) {
            neighbor = value;
        }
    }
    neighbor = false;
    for (index, event) in events.iter().enumerate().rev() {
        if connection_cuts.binary_search(&(index + 1)).is_ok() {
            neighbor = false;
        }
        adjacent[index] |= neighbor;
        if let Some(value) = forced(event, &input, processed.source(), paragraph) {
            neighbor = value;
        }
    }
    let mut control_styles = Vec::new();
    control_styles
        .try_reserve_exact(paragraph.controls.len())
        .map_err(|_| capacity())?;
    control_styles.resize(paragraph.controls.len(), None);
    for (index, style) in styles.iter().enumerate() {
        if let crate::engine::resolve::AnalyzedStyleOwner::Control(control) = style.owner {
            control_styles[control] = Some(index);
        }
    }
    let original = std::mem::take(&mut map.components);
    let mut components = Vec::new();
    let mut atoms = Vec::new();
    let mut captured = Vec::new();
    let limit = input
        .atoms
        .len()
        .checked_add(paragraph.controls.len())
        .and_then(|n| n.checked_add(proposals.len()))
        .ok_or_else(capacity)?;
    components
        .try_reserve_exact(limit)
        .map_err(|_| capacity())?;
    atoms.try_reserve_exact(limit).map_err(|_| capacity())?;
    captured
        .try_reserve_exact(
            events
                .len()
                .checked_add(proposals.len())
                .ok_or_else(capacity)?,
        )
        .map_err(|_| capacity())?;
    let mut expanded = false;
    let mut semantic_connections = Vec::new();
    semantic_connections
        .try_reserve_exact(connection_cuts.len())
        .map_err(|_| capacity())?;
    let mut proposals = proposals.into_iter().peekable();
    for (index, event) in events.into_iter().enumerate() {
        if connection_cuts.binary_search(&index).is_ok() {
            semantic_connections.push(atoms.len());
        }
        while proposals
            .peek()
            .is_some_and(|proposal| proposal.cut == index)
        {
            let proposal = proposals.next().expect("observed proposal");
            let virtual_ = &map.virtuals[proposal.occurrence];
            let cp = char::from_u32(proposal.code_point).expect("separator scalar");
            let source = crate::Range::new(virtual_.source, virtual_.source);
            let processed = crate::Range::new(virtual_.processed, virtual_.processed);
            captured.push(LogicalEvent {
                atom: atoms.len(),
                source: source.start,
                kind: Event::Text(components.len()),
            });
            components.push(Component {
                owner: ComponentOwner::Virtual(proposal.occurrence),
                source,
                processed,
                code_point: proposal.code_point,
                separator_space: proposal.code_point == 0x20,
                style: proposal.style,
                outputs: 0..0,
            });
            atoms.push(crate::engine::OriginalAtom {
                source,
                processed,
                presentation: crate::Range::new(0, 0),
                glyph_bytes: 0..0,
                code_point: proposal.code_point,
                glyph: cp,
            });
            expanded = true;
        }
        let mut event = event;
        let mut component = None;
        match event.kind {
            Event::Text(row) => component = Some((original[row].clone(), input.atoms[row].clone())),
            Event::Anchor(anchor) => {
                if let ParagraphAnchorItem::Control(control) = anchor.item
                    && let ParagraphControlKind::Wbr { style } = &paragraph.controls[control].kind
                    && style.word_space_transform != WordSpaceTransform::None
                {
                    let at = paragraph.controls[control].at;
                    let same_component_forced = input.atoms.get(event.atom).is_some_and(|atom| {
                        atom.source.start < at
                            && at < atom.source.end
                            && crate::engine::analysis::is_hard_separator(atom.code_point)
                    });
                    if paragraph.connection_anchor_cuts.is_none() && !same_component_forced {
                        for (connection, value) in paragraph.connections.iter().enumerate() {
                            if value.kind == crate::ParagraphConnectionKind::Disconnected
                                && value.at == at
                            {
                                return Err(Error::new(
                                    ErrorCode::UnsupportedFeature,
                                    "wbr expansion lacks disconnected-boundary event order",
                                )
                                .with_detail(
                                    ErrorDetail::TextTransform {
                                        reason:
                                            TextTransformError::MissingSeparatorBoundaryContext {
                                                control,
                                                at,
                                                connection,
                                            },
                                    },
                                ));
                            }
                        }
                    }
                    if adjacent[index] {
                        captured.push(LogicalEvent {
                            atom: atoms.len(),
                            ..event
                        });
                        continue;
                    }

                    if let Some(atom) = input
                        .atoms
                        .get(event.atom)
                        .filter(|atom| atom.source.start < at && at < atom.source.end)
                    {
                        return Err(Error::new(
                            ErrorCode::UnsupportedFeature,
                            "wbr expansion has no exact post-processing scalar seam",
                        )
                        .with_detail(ErrorDetail::TextTransform {
                            reason: TextTransformError::UnrepresentableSeparatorSeam {
                                control,
                                at,
                                retained: atom.source,
                            },
                        }));
                    }
                    let style = control_styles[control].expect("validated captured wbr style");
                    let cp = styles[style]
                        .style
                        .authored()
                        .word_space_transform
                        .replacement()
                        .expect("expanding wbr");
                    let p = processed.project(at, crate::Affinity::After);
                    let source = crate::Range::new(at, at);
                    let processed = crate::Range::new(p, p);
                    component = Some((
                        Component {
                            owner: ComponentOwner::Wbr(control),
                            source,
                            processed,
                            code_point: cp,
                            separator_space: cp == 0x20,
                            style,
                            outputs: 0..0,
                        },
                        crate::engine::OriginalAtom {
                            source,
                            processed,
                            presentation: crate::Range::new(0, 0),
                            glyph_bytes: 0..0,
                            code_point: cp,
                            glyph: char::from_u32(cp).expect("separator literal"),
                        },
                    ));
                    expanded = true;
                }
            }
            Event::Deleted(..) => (),
        }
        event.atom = atoms.len();
        if let Some((component, atom)) = component {
            event.kind = Event::Text(components.len());
            components.push(component);
            atoms.push(atom);
        }
        captured.push(event);
    }
    if connection_cuts.binary_search(&captured.len()).is_ok() {
        semantic_connections.push(atoms.len());
    }
    map.components = components;
    if !expanded {
        // None/suppressed still retain genuine capture order. Their ordinary
        // geometry independently keeps the original typographic projection.
        let mut projected = Vec::new();
        projected
            .try_reserve_exact(paragraph.anchor_order.len())
            .map_err(|_| capacity())?;
        for order in 0..paragraph.anchor_order.len() {
            projected.push(Some(crate::engine::analysis::anchor_atom(
                order,
                paragraph,
                processed.source().boxes(),
                &input,
            )));
        }
        let mut input = input;
        input.anchor_atoms = projected;
        return Ok((input, captured));
    }
    let mut end = 0usize;
    for atom in &mut atoms {
        let len = if (0xd800..=0xdfff).contains(&atom.code_point) {
            1
        } else {
            atom.glyph.len_utf16()
        };
        let next = end
            .checked_add(len)
            .ok_or_else(crate::projection::length_overflow)?;
        atom.presentation = crate::Range::new(end, next);
        end = next;
    }
    // A source offset can contain several distinct anchored barriers. Their
    // semantic cuts come from actual mixed order, not the next original atom.
    let mut barrier_atoms: Vec<_> = captured
        .iter()
        .filter_map(|event| match event.kind {
            Event::Anchor(anchor) => match anchor.item {
                ParagraphAnchorItem::Atomic(i)
                    if processed.source().boxes()[i].kind() == InlineBoxKind::InFlow =>
                {
                    Some(event.atom)
                }
                ParagraphAnchorItem::Control(i)
                    if paragraph.controls[i].kind == ParagraphControlKind::ForcedBreak =>
                {
                    Some(event.atom)
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    barrier_atoms.extend(semantic_connections);
    barrier_atoms.sort_unstable();
    barrier_atoms.dedup();
    let mut projection =
        SourceProjection::from_atoms_at_barriers(atoms, end, input.barriers, barrier_atoms)?;
    projection.style_rows = map
        .components
        .iter()
        .map(|component| Some(component.style))
        .collect();
    // Capture/relation events retain their genuine order. Remaining anchors
    // separately project their authentic component cut to rebuilt typography;
    // the consumed generating control retains its actual emission start.
    projection
        .anchor_atoms
        .try_reserve_exact(paragraph.anchor_order.len())
        .map_err(|_| capacity())?;
    projection
        .anchor_atoms
        .resize(paragraph.anchor_order.len(), None);
    let mut order = 0;
    for event in &captured {
        let cut = match event.kind {
            Event::Anchor(anchor) => {
                debug_assert_eq!(paragraph.anchor_order[order], anchor);
                crate::engine::analysis::project_anchor_atom(
                    order,
                    paragraph,
                    processed.source().boxes(),
                    &projection,
                    event.atom,
                )
            }
            Event::Text(component) => match map.components[component].owner {
                ComponentOwner::Wbr(control) => {
                    debug_assert_eq!(
                        paragraph.anchor_order[order].item,
                        ParagraphAnchorItem::Control(control)
                    );
                    event.atom
                }
                ComponentOwner::Original(_) | ComponentOwner::Virtual(_) => continue,
            },
            Event::Deleted(..) => continue,
        };
        projection.anchor_atoms[order] = Some(cut);
        order += 1;
    }
    debug_assert_eq!(order, paragraph.anchor_order.len());
    Ok((projection, captured))
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn capture(
        text: &str,
        base: Style,
        own: Style,
        direction: ParagraphBaseDirection,
    ) -> ParagraphInput {
        let source = Source::new(text);
        let len = source.text().len();
        ParagraphInput {
            source: source.clone(),
            base_style: base,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: direction,
            origins: vec![ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![ParagraphControl {
                at: if len == 3 { 2 } else { 1 },
                kind: ParagraphControlKind::Wbr {
                    style: Box::new(own),
                },
            }],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: Affinity::After,
            }],
        }
    }

    #[test]
    fn generated_wbr_changes_casing_context_but_never_becomes_a_casing_input_row() {
        let base = Style {
            text_transform: TextTransform::Set(
                TextTransformSet::none().with_case(CaseTransform::Lowercase),
            ),
            ..Style::default()
        };
        for (operation, expected) in [
            (WordSpaceTransform::None, "οσα"),
            (WordSpaceTransform::Space, "ος α"),
            (WordSpaceTransform::IdeographicSpace, "ος\u{3000}α"),
        ] {
            let own = Style {
                word_space_transform: operation,
                ..base.clone()
            };
            let a = AnalyzedSource::from_paragraph(capture(
                "ΟΣΑ",
                base.clone(),
                own,
                ParagraphBaseDirection::LeftToRight,
            ))
            .unwrap();
            assert_eq!(a.inner.projection.text, expected);
            assert_eq!(a.inner.transform.rows.len(), 3);
            for (row, range) in [
                (0, Range::new(0, 1)),
                (1, Range::new(1, 2)),
                (2, Range::new(2, 3)),
            ] {
                assert_eq!(a.inner.transform.original_pieces(row)[0].aggregate, range);
            }
        }
    }

    #[test]
    fn generated_wbr_participates_once_in_real_global_bidi_with_its_actual_style() {
        let own = Style {
            size: 17.0,
            word_space_transform: WordSpaceTransform::Space,
            ..Style::default()
        };
        let a = AnalyzedSource::from_paragraph(capture(
            "אA",
            Style::default(),
            own,
            ParagraphBaseDirection::RightToLeft,
        ))
        .unwrap();
        assert_eq!(a.inner.projection.text, "א A");
        assert_eq!(a.inner.analysis.levels, [1, 1, 2]);
        let style = a.inner.analysis.infos[1].1;
        assert_eq!(a.inner.styles[style].style.authored().size, 17.0);
        assert!(matches!(
            a.inner.styles[style].owner,
            crate::engine::resolve::AnalyzedStyleOwner::Control(0)
        ));
        assert!(a.inner.transform.original_row_for_atom(1).is_none());
        assert_eq!(a.inner.transform.controls_for_atoms(0..3), [0]);
    }

    #[test]
    fn even_unexpanded_wbr_requires_a_valid_real_style() {
        let own = Style {
            size: f32::NAN,
            ..Style::default()
        };
        let error = AnalyzedSource::from_paragraph(capture(
            "AA",
            Style::default(),
            own,
            ParagraphBaseDirection::LeftToRight,
        ))
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidStyle);
    }
}

#[cfg(test)]
mod barrier_tests {
    use crate::*;

    fn atomic_input(before: bool) -> ParagraphInput {
        let mut source = Source::new("A\u{3099}");
        source.inline_box(InlineBox::new(
            Id::from_u64(6318),
            InlineBoxKind::InFlow,
            1,
            Size::new(0.0, 0.0),
        ));
        let range = Range::new(0, 2);
        let atomic = ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(0),
            affinity: Affinity::After,
        };
        let wbr = ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        };
        ParagraphInput {
            source: source.clone(),
            base_style: Style::default(),
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![ParagraphOrigin {
                source,
                text_node: None,
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![ParagraphControl {
                at: 1,
                kind: ParagraphControlKind::Wbr {
                    style: Box::new(Style {
                        word_space_transform: WordSpaceTransform::Space,
                        ..Style::default()
                    }),
                },
            }],
            replacements: vec![],
            boundary_policies: [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            })
            .collect(),
            anchor_order: if before {
                vec![atomic, wbr]
            } else {
                vec![wbr, atomic]
            },
        }
    }

    fn atomic_order(before: bool) -> AnalyzedSource {
        AnalyzedSource::from_paragraph(atomic_input(before)).unwrap()
    }

    #[test]
    fn atomic_wbr_barrier_before_separator_keeps_space_and_mark_together() {
        let a = atomic_order(true);
        assert_eq!(a.source().text().code_units(), [0x41, 0x3099]);
        assert_eq!(a.inner.projection.text, "A \u{3099}");
        assert_eq!(a.inner.projection.barrier_atoms, [1]);
        assert_eq!(a.inner.projection.graphemes, [0, 1, 3]);
        assert_eq!(a.inner.analysis.graphemes, [0..1, 1..3]);
    }

    #[test]
    fn atomic_wbr_barrier_after_separator_separates_space_and_mark() {
        let a = atomic_order(false);
        assert_eq!(a.source().text().code_units(), [0x41, 0x3099]);
        assert_eq!(a.inner.projection.text, "A \u{3099}");
        assert_eq!(a.inner.projection.barrier_atoms, [2]);
        assert_eq!(a.inner.projection.graphemes, [0, 1, 2, 3]);
        assert_eq!(a.inner.analysis.graphemes, [0..1, 1..2, 2..3]);
    }
    #[test]
    fn two_actual_atomics_keep_distinct_same_source_cuts_around_wbr() {
        let mut p = atomic_input(true);
        p.source.inline_box(InlineBox::new(
            Id::from_u64(6319),
            InlineBoxKind::InFlow,
            1,
            Size::new(0.0, 0.0),
        ));
        p.anchor_order.push(ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(1),
            affinity: Affinity::After,
        });
        for definition in [
            ParagraphBoundaryDefinition::AtomicBefore(1),
            ParagraphBoundaryDefinition::AtomicAfter(1),
        ] {
            p.boundary_policies.push(ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            });
        }
        let a = AnalyzedSource::from_paragraph(p).unwrap();
        assert_eq!(a.inner.projection.text, "A \u{3099}");
        assert_eq!(a.inner.projection.barriers, [1]);
        assert_eq!(a.inner.projection.barrier_atoms, [1, 2]);
        assert_eq!(a.inner.projection.graphemes, [0, 1, 2, 3]);
        assert_eq!(a.inner.analysis.graphemes, [0..1, 1..2, 2..3]);
    }
}
