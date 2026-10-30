//! Original U+200B expansion over the actual post-Phase-I mixed stream.
use super::context::{SourceContext, StyleRow};
use super::relation::TransformMap;
pub(super) mod automatic;
use crate::engine::analysis::wrapping::{Event, LogicalEvent};
use crate::projection::SourceProjection;
use crate::protocol::paragraph::{ParagraphControlKind, SemanticAnchorItem};
use crate::{Error, ErrorCode, ErrorDetail, InlineBoxKind, Result, TextTransformError};

/// None is genuinely transparent; Some records the next actual item's forced role.
fn forced(
    event: &LogicalEvent,
    input: &SourceProjection,
    source: &crate::Source,
    paragraph: &SourceContext,
) -> Option<bool> {
    match event.kind {
        Event::Text(atom) => Some(crate::engine::analysis::is_hard_separator(
            input.atoms[atom].code_point,
        )),
        Event::Deleted(..) => None,
        Event::Anchor(anchor) => match anchor.item {
            SemanticAnchorItem::EmptyOrigin(_) => None,
            SemanticAnchorItem::ComputedLineBoundary(_) => Some(true),
            SemanticAnchorItem::Atomic(i) => {
                (source.boxes()[i].kind() == InlineBoxKind::InFlow).then_some(false)
            }
            SemanticAnchorItem::Control(i) => match paragraph.controls[i].kind {
                ParagraphControlKind::InlineBoundaryStart
                | ParagraphControlKind::InlineBoundaryEnd => None,
                ParagraphControlKind::ForcedBreak => Some(true),
                _ => Some(false),
            },
        },
    }
}

#[derive(Clone, Copy)]
pub(super) struct SeparatorValue {
    pub(super) code_point: u32,
    pub(super) separator_space: bool,
}

pub(super) fn original_value(
    input: &SourceProjection,
    map: &TransformMap,
    styles: &[StyleRow<'_>],
    paragraph: &SourceContext<'_>,
    row: usize,
    proof: Adjacency,
) -> Result<SeparatorValue> {
    let code_point = input.atoms[row].code_point;
    let unchanged = SeparatorValue {
        code_point,
        separator_space: false,
    };
    if code_point != 0x200b || proof.certain {
        return Ok(unchanged);
    }
    let Some(cp) = styles[map.rows[row].style]
        .style
        .authored()
        .word_space_transform
        .replacement()
    else {
        return Ok(unchanged);
    };
    if let Some(edge) = proof.missing {
        return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
    }
    if let Some(connection) = proof.uncertain {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "original separator lacks boundary event order",
        )
        .with_detail(ErrorDetail::TextTransform {
            reason: TextTransformError::MissingOriginalSeparatorBoundaryContext {
                source: input.atoms[row].source,
                at: paragraph.connections[connection].at,
                connection: paragraph.connections[connection].diagnostic,
            },
        }));
    }
    Ok(SeparatorValue {
        code_point: cp,
        separator_space: cp == 0x20,
    })
}

pub(super) fn wbr_value(
    input: &SourceProjection,
    paragraph: &SourceContext<'_>,
    control: usize,
    atom: usize,
    proof: Adjacency,
) -> Result<Option<SeparatorValue>> {
    let ParagraphControlKind::Wbr { style } = &paragraph.controls[control].kind else {
        return Ok(None);
    };
    let Some(cp) = style.word_space_transform.replacement() else {
        return Ok(None);
    };
    let at = paragraph.controls[control].at;
    let same_component_forced = input.atoms.get(atom).is_some_and(|atom| {
        atom.source.start < at
            && at < atom.source.end
            && crate::engine::analysis::is_hard_separator(atom.code_point)
    });
    if paragraph.connection_anchor_cuts.is_none() && !same_component_forced {
        for value in &paragraph.connections {
            if value.kind == crate::ParagraphConnectionKind::Disconnected && value.at == at {
                return Err(Error::new(
                    ErrorCode::UnsupportedFeature,
                    "wbr lacks boundary event order",
                )
                .with_detail(ErrorDetail::TextTransform {
                    reason: TextTransformError::MissingSeparatorBoundaryContext {
                        control,
                        at,
                        connection: value.diagnostic,
                    },
                }));
            }
        }
    }
    if proof.certain {
        return Ok(None);
    }
    if let Some(edge) = proof.missing {
        return Err(edge.unavailable(crate::SourceContinuityOperation::Transformation));
    }
    if let Some(connection) = proof.uncertain {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "wbr lacks boundary event order",
        )
        .with_detail(ErrorDetail::TextTransform {
            reason: TextTransformError::MissingSeparatorBoundaryContext {
                control,
                at,
                connection: paragraph.connections[connection].diagnostic,
            },
        }));
    }
    if let Some(atom) = input
        .atoms
        .get(atom)
        .filter(|atom| atom.source.start < at && at < atom.source.end)
    {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "wbr has no exact scalar seam",
        )
        .with_detail(ErrorDetail::TextTransform {
            reason: TextTransformError::UnrepresentableSeparatorSeam {
                control,
                at,
                retained: atom.source,
            },
        }));
    }
    Ok(Some(SeparatorValue {
        code_point: cp,
        separator_space: cp == 0x20,
    }))
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

/// A receipt is exact, or it retains every possible event seam between the two
/// genuine origin occurrences. A flat capture never chooses a control's side.
pub(crate) struct ConnectionSeam {
    connection: usize,
    pub(crate) lower: usize,
    pub(crate) upper: usize,
    pub(crate) atom: usize,
}

pub(crate) fn connection_seams(
    input: &SourceProjection,
    events: &[LogicalEvent],
    paragraph: &SourceContext,
) -> Result<Vec<ConnectionSeam>> {
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
            SemanticAnchorItem::Atomic(i) => Some(i),
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
    let mut computed_events = vec![0; paragraph.computed_boundaries.len()];
    let mut text_edges = Vec::new();
    text_edges
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    for (index, event) in events.iter().enumerate() {
        match event.kind {
            Event::Anchor(anchor) => match anchor.item {
                SemanticAnchorItem::Control(i) => control_events[i] = index + 1,
                SemanticAnchorItem::Atomic(i) => atomic_events[i] = index + 1,
                SemanticAnchorItem::EmptyOrigin(i) => empty_events[i] = index + 1,
                SemanticAnchorItem::ComputedLineBoundary(i) => computed_events[i] = index + 1,
            },
            Event::Text(i) => text_edges.push((input.atoms[i].source.end, index + 1, index)),
            Event::Deleted(range, _, _) => text_edges.push((range.end, index + 1, index)),
        }
    }
    text_edges.sort_unstable();
    let mut max = 0;
    for (_, event, _) in &mut text_edges {
        max = max.max(*event);
        *event = max;
    }
    let mut min = events.len();
    for (_, _, event) in text_edges.iter_mut().rev() {
        min = min.min(*event);
        *event = min;
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
            SemanticAnchorItem::Control(i) => control_events[i],
            SemanticAnchorItem::Atomic(i) => atomic_events[i],
            SemanticAnchorItem::EmptyOrigin(i) => empty_events[i],
            SemanticAnchorItem::ComputedLineBoundary(i) => computed_events[i],
        };
        anchor_cuts.push(anchor_cuts.last().copied().unwrap().max(event));
    }
    let mut seams = Vec::new();
    seams
        .try_reserve_exact(paragraph.connections.len())
        .map_err(|_| capacity())?;
    for (index, connection) in paragraph.connections.iter().enumerate() {
        if connection.kind != ParagraphConnectionKind::Disconnected {
            continue;
        }
        let before = paragraph
            .connection_ends
            .as_ref()
            .map_or(connection.at, |ends| ends[index].0);
        let text = text_edges.partition_point(|(end, _, _)| *end <= before);
        let text_cut = text.checked_sub(1).map_or(0, |i| text_edges[i].1);
        let (lower, upper) = if let Some(ranks) = &paragraph.connection_anchor_cuts {
            let cut = text_cut.max(anchor_cuts[ranks[index]]);
            (cut, cut)
        } else {
            let before_origin = &paragraph.origins[connection.before];
            let after_origin = &paragraph.origins[connection.after];
            let lower = if before_origin.aggregate.is_empty() {
                empty_events[connection.before]
            } else {
                text_cut
            };
            let upper = if after_origin.aggregate.is_empty() {
                empty_events[connection.after] - 1
            } else {
                let following =
                    text_edges.partition_point(|(end, _, _)| *end <= after_origin.aggregate.start);
                text_edges
                    .get(following)
                    .map_or(events.len(), |edge| edge.2)
            };
            if lower > upper {
                return Err(crate::protocol::protocol_error(
                    crate::ProtocolError::InvalidConnection {
                        connection: index,
                        at: connection.at,
                    },
                ));
            }
            (lower, upper)
        };
        seams.push(ConnectionSeam {
            connection: index,
            lower,
            upper,
            atom: events
                .get(lower)
                .map_or(input.atoms.len(), |event| event.atom),
        });
    }
    Ok(seams)
}

#[derive(Clone, Copy, Default)]
pub(super) struct Adjacency {
    pub(super) certain: bool,
    pub(super) uncertain: Option<usize>,
    pub(super) missing: Option<super::context::MissingEdge>,
}

struct ForcedNeighbors {
    before: Vec<Option<usize>>,
    after: Vec<Option<usize>>,
}

fn forced_neighbors(
    input: &SourceProjection,
    events: &[LogicalEvent],
    source: &crate::Source,
    paragraph: &SourceContext,
    cuts: &[usize],
) -> Result<ForcedNeighbors> {
    let mut before = Vec::new();
    let mut after = Vec::new();
    before
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    after
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    before.resize(events.len(), None);
    after.resize(events.len(), None);
    let mut neighbor = None;
    for (index, event) in events.iter().enumerate() {
        if cuts.binary_search(&index).is_ok() {
            neighbor = None;
        }
        before[index] = neighbor;
        if let Some(value) = forced(event, input, source, paragraph) {
            neighbor = value.then_some(index);
        }
    }
    neighbor = None;
    for (index, event) in events.iter().enumerate().rev() {
        if cuts.binary_search(&(index + 1)).is_ok() {
            neighbor = None;
        }
        after[index] = neighbor;
        if let Some(value) = forced(event, input, source, paragraph) {
            neighbor = value.then_some(index);
        }
    }
    Ok(ForcedNeighbors { before, after })
}

/// A flat interval contains no retained Text event: atoms ending at/before the
/// join precede lower, and upper precedes every atom ending after the join.
/// Empty-origin anchors occupy that same raw scalar cut; disconnected source
/// barriers already split crossing scalars. Contradictory occurrence bounds reject.
/// Thus moving the cut through anchors/deleted events can only remove a nearest
/// neighbor of an original Text target outside the interval. Keep directions
/// separate: uncertain neighbors on both sides do not prove either connected.
pub(super) fn forced_adjacency(
    input: &SourceProjection,
    events: &[LogicalEvent],
    source: &crate::Source,
    paragraph: &SourceContext,
    seams: &[ConnectionSeam],
) -> Result<Vec<Adjacency>> {
    let mut lower: Vec<_> = seams.iter().map(|seam| seam.lower).collect();
    let mut upper: Vec<_> = seams.iter().map(|seam| seam.upper).collect();
    lower.sort_unstable();
    lower.dedup();
    upper.sort_unstable();
    upper.dedup();
    let lower = forced_neighbors(input, events, source, paragraph, &lower)?;
    let upper = forced_neighbors(input, events, source, paragraph, &upper)?;
    // Resolve interval membership once in event order rather than scanning all
    // connections for every event. The heap retains the first actual connection
    // whose unresolved interval contains the candidate forced neighbor.
    let mut intervals = Vec::new();
    intervals
        .try_reserve_exact(seams.len())
        .map_err(|_| capacity())?;
    intervals.extend(
        seams
            .iter()
            .filter(|seam| seam.lower < seam.upper)
            .map(|seam| (seam.lower, seam.upper, seam.connection)),
    );
    intervals.sort_unstable();
    let mut intervals = intervals.into_iter().peekable();
    let mut active = std::collections::BinaryHeap::new();
    active.try_reserve(seams.len()).map_err(|_| capacity())?;
    let mut uncertain_connections = Vec::new();
    uncertain_connections
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    for index in 0..events.len() {
        while intervals.peek().is_some_and(|interval| interval.0 <= index) {
            let (_, end, connection) = intervals.next().expect("observed interval");
            active.push(std::cmp::Reverse((connection, end)));
        }
        while active.peek().is_some_and(|entry| entry.0.1 <= index) {
            active.pop();
        }
        uncertain_connections.push(active.peek().map(|entry| entry.0.0));
    }
    let mut adjacency = Vec::new();
    adjacency
        .try_reserve_exact(events.len())
        .map_err(|_| capacity())?;
    for index in 0..events.len() {
        let known = |neighbor: Option<usize>| {
            neighbor.is_some_and(|neighbor| {
                paragraph
                    .missing_between(&events[neighbor], &events[index], input)
                    .is_none()
            })
        };
        let certain = known(lower.before[index]) && known(upper.before[index])
            || known(lower.after[index]) && known(upper.after[index]);
        let possible = [
            lower.before[index],
            upper.before[index],
            lower.after[index],
            upper.after[index],
        ];
        let uncertain = if certain {
            None
        } else {
            possible
                .iter()
                .flatten()
                .filter_map(|&neighbor| uncertain_connections[neighbor])
                .min()
        };
        let missing = if certain {
            None
        } else {
            possible.iter().flatten().find_map(|&neighbor| {
                paragraph.missing_between(&events[neighbor], &events[index], input)
            })
        };
        adjacency.push(Adjacency {
            certain,
            uncertain,
            missing,
        });
    }
    Ok(adjacency)
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
            let a = AnalyzedSource::from_paragraph(
                capture(
                    "ΟΣΑ",
                    base.clone(),
                    own,
                    ParagraphBaseDirection::LeftToRight,
                ),
                crate::AnalysisTypography::HorizontalComposition,
            )
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
        let a = AnalyzedSource::from_paragraph(
            capture(
                "אA",
                Style::default(),
                own,
                ParagraphBaseDirection::RightToLeft,
            ),
            crate::AnalysisTypography::HorizontalComposition,
        )
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
        let error = AnalyzedSource::from_paragraph(
            capture(
                "AA",
                Style::default(),
                own,
                ParagraphBaseDirection::LeftToRight,
            ),
            crate::AnalysisTypography::HorizontalComposition,
        )
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
        AnalyzedSource::from_paragraph(
            atomic_input(before),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap()
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
        let a = AnalyzedSource::from_paragraph(p, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        assert_eq!(a.inner.projection.text, "A \u{3099}");
        assert_eq!(a.inner.projection.barriers, [1]);
        assert_eq!(a.inner.projection.barrier_atoms, [1, 2]);
        assert_eq!(a.inner.projection.graphemes, [0, 1, 2, 3]);
        assert_eq!(a.inner.analysis.graphemes, [0..1, 1..2, 2..3]);
    }
}
