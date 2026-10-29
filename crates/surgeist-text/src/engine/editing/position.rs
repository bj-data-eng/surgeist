//! Placement-specific observations of the existing interaction rows.

use super::super::{
    PlacedLine,
    layout::data::{FinalizedVisualItem, PlacedLayout},
};
use crate::protocol::prepared::SelectedRelationEvent;
use crate::protocol::{ProtocolError, protocol_error};
use crate::transform::relation::{OriginalPiece, RelationEvent};
use crate::{
    Affinity, Cursor, CursorGeometry, Movement, Point, Range, Rect, Result, SelectionGeometry,
};
use std::sync::Arc;

/// References real finalized rows. No second geometry or source owner is stored.
#[derive(Clone, Copy)]
pub(super) enum Target {
    Cluster { cluster: usize, right: bool },
    Anchor { anchor: usize, right: bool },
    Strut,
}
#[derive(Clone, Copy)]
pub(crate) struct Position {
    pub(super) line: usize,
    pub(crate) source: Cursor,
    target: Target,
    /// Genuine source-less slot ownership, not a borrowed removal geometry.
    slot: bool,
    /// Cut in the actual selected mixed-event stream, independent of source projection.
    cut: usize,
}

fn cluster_atoms(line: &PlacedLine, cluster: usize) -> std::ops::Range<usize> {
    let presentation = &line.finalized.selected.presentation;
    let range = line.finalized.data.clusters[cluster].presentation;
    let atoms = &presentation.projection.atoms;
    atoms.partition_point(|atom| atom.presentation.end <= range.start)
        ..atoms.partition_point(|atom| atom.presentation.start < range.end)
}

fn legal_cluster_edge(line: &PlacedLine, cluster: usize, atom: usize, presentation: usize) -> bool {
    line.finalized
        .selected
        .presentation
        .projection
        .graphemes
        .binary_search(&presentation)
        .is_ok()
        && !(line.finalized.data.clusters[cluster].provider_atoms.start < atom
            && atom < line.finalized.data.clusters[cluster].provider_atoms.end)
}

pub(super) fn prepare(lines: &[PlacedLine], cuts: &[usize]) -> Vec<Position> {
    let mut positions = Vec::new();
    for (line_index, (line, &offset)) in lines.iter().zip(cuts).enumerate() {
        let presentation = &line.finalized.selected.presentation;
        let relation = &presentation.selected_relation;
        let mut local = Vec::new();
        for item in &line.finalized.data.visual_order {
            match *item {
                FinalizedVisualItem::Cluster(index) => {
                    let cluster = &line.finalized.data.clusters[index];
                    let atoms = cluster_atoms(line, index);
                    if atoms.is_empty() {
                        continue;
                    }
                    for right in [false, true] {
                        let start = right == (cluster.level & 1 != 0);
                        let (atom, at, source_index, affinity) = if start {
                            (
                                atoms.start,
                                cluster.presentation.start,
                                cluster.source.start,
                                Affinity::After,
                            )
                        } else {
                            (
                                atoms.end,
                                cluster.presentation.end,
                                cluster.source.end,
                                Affinity::Before,
                            )
                        };
                        if legal_cluster_edge(line, index, atom, at) {
                            let cut = if start {
                                relation.atom_events[atom]
                            } else {
                                relation.atom_events[atom - 1] + 1
                            };
                            local.push(Position {
                                line: line_index,
                                source: Cursor::new(source_index, affinity),
                                target: Target::Cluster {
                                    cluster: index,
                                    right,
                                },
                                slot: false,
                                cut: offset + cut,
                            });
                        }
                    }
                }
                FinalizedVisualItem::Anchor(anchor) => {
                    let row = &line.finalized.data.anchors[anchor];
                    let unit = &presentation.units[row.unit];
                    let extent = &relation.unit_events[row.unit];
                    let transform = &line.finalized.selected.prepared.analyzed.transform;
                    // A participant's canonical extent may also own adjacent
                    // transform removals. Its actual slot is the real event,
                    // while those removals receive their separate cuts below.
                    let mut owned = extent
                        .clone()
                        .filter(|&event| match relation.events[event] {
                            SelectedRelationEvent::Base { event } => {
                                !matches!(transform.events()[event], RelationEvent::Removal { .. })
                            }
                            SelectedRelationEvent::Indication { .. } => true,
                        });
                    let first = owned.next();
                    let events = first.map_or_else(
                        || extent.clone(),
                        |first| first..owned.next_back().unwrap_or(first) + 1,
                    );
                    for right in [false, true] {
                        let before = right == (row.level & 1 != 0);
                        if !before && unit.source.is_empty() && row.inline_end == row.slot.x {
                            continue;
                        }
                        local.push(Position {
                            line: line_index,
                            source: if before { Cursor::new(unit.source.start, Affinity::After) } else { Cursor::new(unit.source.end, Affinity::Before) },
                            target: Target::Anchor { anchor, right },
                            slot: matches!(unit.kind, crate::protocol::prepared::PresentationUnitKind::Control { .. } | crate::protocol::prepared::PresentationUnitKind::EmptyOrigin { .. }),
                            cut: offset + if before { events.start } else { events.end },
                        });
                    }
                }
                FinalizedVisualItem::InlineBox(_) => {}
            }
        }
        if local.is_empty() {
            local.push(Position {
                line: line_index,
                source: Cursor::new(presentation.source.start, Affinity::After),
                target: Target::Strut,
                slot: false,
                cut: offset,
            });
        }
        // Removal cuts borrow an actual interaction edge; they do not create
        // anchors, glyphs, typographic participants or independent rectangles.
        let mut logical: Vec<_> = local
            .iter()
            .enumerate()
            .map(|(index, position)| (position.cut, index))
            .collect();
        logical.sort_unstable();
        let mut ordered: Vec<_> = local.iter().copied().enumerate().collect();
        let transform = &line.finalized.selected.prepared.analyzed.transform;
        for (event, entry) in relation.events.iter().enumerate() {
            let SelectedRelationEvent::Base { event: base } = *entry else {
                continue;
            };
            let RelationEvent::Removal { row } = transform.events()[base] else {
                continue;
            };
            let original = &transform.rows[row];
            for (cut, source, before) in [
                (
                    offset + event,
                    Cursor::new(original.source.start, Affinity::After),
                    true,
                ),
                (
                    offset + event + 1,
                    Cursor::new(original.source.end, Affinity::Before),
                    false,
                ),
            ] {
                let split = logical.partition_point(|&(position, _)| position < cut);
                let at = if before {
                    logical
                        .partition_point(|&(position, _)| position <= cut)
                        .saturating_sub(1)
                } else {
                    split.min(logical.len() - 1)
                };
                let target = logical[at].1;
                ordered.push((
                    target,
                    Position {
                        line: line_index,
                        source,
                        target: local[target].target,
                        slot: false,
                        cut,
                    },
                ));
            }
        }
        ordered.sort_by(|(a, left), (b, right)| {
            a.cmp(b).then_with(|| {
                let level = match left.target {
                    Target::Cluster { cluster, .. } => line.finalized.data.clusters[cluster].level,
                    Target::Anchor { anchor, .. } => line.finalized.data.anchors[anchor].level,
                    Target::Strut => presentation.analysis.base_level,
                };
                if level & 1 == 0 {
                    left.cut.cmp(&right.cut)
                } else {
                    right.cut.cmp(&left.cut)
                }
            })
        });
        positions.extend(ordered.into_iter().map(|(_, row)| row));
    }
    positions
}

fn row(layout: &PlacedLayout, index: usize) -> Result<&Position> {
    layout
        .interaction
        .positions
        .get(index)
        .ok_or_else(|| protocol_error(ProtocolError::InvalidPlacedPosition { position: index }))
}
fn local_x(line: &PlacedLine, target: Target) -> f64 {
    match target {
        Target::Cluster { cluster, right } => {
            let cluster = &line.finalized.data.clusters[cluster];
            cluster.x + if right { cluster.advance } else { 0.0 }
        }
        Target::Anchor { anchor, right } => {
            let anchor = &line.finalized.data.anchors[anchor];
            if right {
                anchor.inline_end
            } else {
                anchor.slot.x
            }
        }
        Target::Strut => line.finalized.data.metrics.offset,
    }
}
pub(super) fn inline(layout: &PlacedLayout, index: usize) -> Result<f64> {
    let position = row(layout, index)?;
    Ok(local_x(&layout.lines[position.line], position.target))
}
pub(crate) fn geometry(layout: &PlacedLayout, index: usize) -> Result<CursorGeometry> {
    let position = row(layout, index)?;
    let line = &layout.lines[position.line];
    let metrics = line.finalized.data.metrics;
    Ok(CursorGeometry::new(crate::layout::line_rect(
        line,
        Rect::new(
            local_x(line, position.target),
            metrics.block_min,
            1.0,
            metrics.block_max - metrics.block_min,
        ),
    )))
}

/// An executed deletion owns its original interval even though it paints no
/// cluster. Lend the existing occurrence row instead of reconstructing an edge
/// from a numerically adjacent source offset.
pub(super) fn deleted_position(layout: &PlacedLayout, value: Cursor) -> Option<usize> {
    layout
        .interaction
        .positions
        .iter()
        .enumerate()
        .find_map(|(index, position)| {
            let Target::Anchor { anchor, .. } = position.target else {
                return None;
            };
            let line = &layout.lines[position.line];
            let unit = &line.finalized.selected.presentation.units
                [line.finalized.data.anchors[anchor].unit];
            let eligible = super::owns_source_side(unit.source, value);
            (matches!(
                unit.kind,
                crate::protocol::prepared::PresentationUnitKind::Deleted
            ) && eligible)
                .then_some(index)
        })
}
pub(crate) fn hit(layout: &PlacedLayout, point: Point) -> Option<usize> {
    let source = super::cursor::from_point(layout, point);
    let vertical_axis_line =
        if layout.prepared.options.orientation() != crate::LineOrientation::Horizontal {
            super::cursor::line_at_point(layout, point)
        } else {
            None
        };
    let matching =
        layout.interaction.positions.iter().any(|row| {
            row.source == source && vertical_axis_line.is_none_or(|line| row.line == line)
        });
    layout
        .interaction
        .positions
        .iter()
        .enumerate()
        .filter(|(_, row)| {
            vertical_axis_line.is_none_or(|line| row.line == line)
                && (!matching || row.source == source)
        })
        .min_by(|(a, _), (b, _)| {
            position_distance(layout, *a, point).total_cmp(&position_distance(layout, *b, point))
        })
        .map(|(index, _)| index)
}
fn position_distance(layout: &PlacedLayout, index: usize, point: Point) -> f64 {
    let rect = geometry(layout, index)
        .expect("owned interaction row")
        .rect();
    let orientation = layout.prepared.options.orientation();
    if orientation == crate::LineOrientation::SidewaysCounterclockwise {
        let position = &layout.interaction.positions[index];
        let line = &layout.lines[position.line];
        // The top edge of a CCW caret hull is not its actual source edge.
        let caret_y = line.anchor.y - local_x(line, position.target);
        let dx = if point.x < rect.origin.x {
            rect.origin.x - point.x
        } else if point.x > rect.origin.x + rect.size.width {
            point.x - (rect.origin.x + rect.size.width)
        } else {
            0.0
        };
        (point.y - caret_y).hypot(dx)
    } else {
        distance(rect, point, orientation)
    }
}
fn distance(rect: Rect, point: Point, orientation: crate::LineOrientation) -> f64 {
    if orientation != crate::LineOrientation::Horizontal {
        let dx = if point.x < rect.origin.x {
            rect.origin.x - point.x
        } else if point.x > rect.origin.x + rect.size.width {
            point.x - (rect.origin.x + rect.size.width)
        } else {
            0.0
        };
        return (point.y - rect.origin.y).hypot(dx);
    }
    let dy = if point.y < rect.origin.y {
        rect.origin.y - point.y
    } else if point.y > rect.origin.y + rect.size.height {
        point.y - (rect.origin.y + rect.size.height)
    } else {
        0.0
    };
    // Scale the comparison without squared distances which can overflow for
    // otherwise admitted finite placement coordinates.
    (point.x - rect.origin.x).hypot(dy)
}
/// Selects an actual eligible interaction row on the requested formatted line.
/// Anchors and genuine struts participate alongside cluster edges; this does
/// not reconstruct a caret from a source offset or a guessed rectangle.
/// Horizontal proximity uses physical x; vertical axes use local inline distance.
pub(super) fn nearest_on_line(layout: &PlacedLayout, line: usize, inline: f64) -> Option<usize> {
    let coordinate = |index: usize| {
        if layout.lines[line].orientation != crate::LineOrientation::Horizontal {
            local_x(
                &layout.lines[line],
                layout.interaction.positions[index].target,
            )
        } else {
            geometry(layout, index).expect("owned row").rect().origin.x
        }
    };
    layout
        .interaction
        .positions
        .iter()
        .enumerate()
        .filter(|(_, row)| row.line == line)
        .min_by(|(a, _), (b, _)| {
            (coordinate(*a) - inline)
                .abs()
                .total_cmp(&(coordinate(*b) - inline).abs())
        })
        .map(|(index, _)| index)
}

pub(crate) fn movement(layout: &PlacedLayout, index: usize, movement: Movement) -> Result<usize> {
    let current = row(layout, index)?;
    let rows = &layout.interaction.positions;
    let last = rows.len().saturating_sub(1);
    Ok(match movement {
        Movement::DocumentStart => 0,
        Movement::DocumentEnd => last,
        Movement::PreviousCluster => index.saturating_sub(1),
        Movement::NextCluster => (index + 1).min(last),
        Movement::LineStart => rows
            .iter()
            .position(|row| row.line == current.line)
            .unwrap_or(index),
        Movement::LineEnd => rows
            .iter()
            .rposition(|row| row.line == current.line)
            .unwrap_or(index),
        Movement::PreviousLine | Movement::NextLine => {
            let target = if movement == Movement::PreviousLine {
                current.line.checked_sub(1)
            } else {
                current
                    .line
                    .checked_add(1)
                    .filter(|&line| line < layout.lines.len())
            };
            match target {
                None => index,
                Some(target) => {
                    let vertical_axis =
                        layout.prepared.options.orientation() != crate::LineOrientation::Horizontal;
                    let inline = if vertical_axis {
                        local_x(&layout.lines[current.line], current.target)
                    } else {
                        geometry(layout, index)?.rect().origin.x
                    };
                    nearest_on_line(layout, target, inline).unwrap_or(index)
                }
            }
        }
        Movement::PreviousWord | Movement::NextWord => {
            let forward = movement == Movement::NextWord;
            let target = match current.target {
                Target::Cluster { cluster, right } => {
                    let level = layout.lines[current.line].finalized.data.clusters[cluster].level;
                    super::cursor::logical_word_from_cluster(
                        layout,
                        current.source,
                        forward,
                        (current.line, cluster),
                        right != (level & 1 != 0),
                    )
                }
                Target::Anchor { anchor, .. } => super::cursor::logical_word_from_anchor(
                    layout,
                    current.source,
                    forward,
                    current.line,
                    anchor,
                ),
                Target::Strut => return Ok(index),
            };
            match target.row {
                None => index,
                Some(((line, cluster), end)) => {
                    let level = layout.lines[line].finalized.data.clusters[cluster].level;
                    let right = end != (level & 1 != 0);
                    rows.iter()
                        .position(|position| {
                            position.line == line
                                && matches!(position.target, Target::Cluster {cluster: actual, right: actual_right}
                                    if actual == cluster && actual_right == right)
                        })
                        .ok_or_else(|| protocol_error(ProtocolError::InvalidPlacedPosition { position: index }))?
                }
            }
        }
    })
}
/// Exact selected-event coverage, never a hull of projected source cursors.
pub(crate) fn selection(
    layout: &Arc<PlacedLayout>,
    anchor: usize,
    focus: usize,
) -> Result<crate::PlacedSelection> {
    let a = row(layout, anchor)?;
    let b = row(layout, focus)?;
    let (first, last) = if a.cut <= b.cut { (a, b) } else { (b, a) };
    let (start, end) = (first.cut, last.cut);
    let mut pieces = Vec::new();
    let mut selected_slot = false;
    let offsets = &layout.line_cuts;
    let mut offset = 0;
    for receipt in &layout.receipts {
        let selected = &receipt.selected;
        let relation = &selected.presentation.selected_relation;
        let transform = &selected.prepared.analyzed.transform;
        for (index, event) in relation.events.iter().enumerate() {
            if !(start..end).contains(&(offset + index)) {
                continue;
            }
            match *event {
                SelectedRelationEvent::Base { event } => match transform.events()[event] {
                    RelationEvent::Text { atom } => {
                        if let Some(row) = transform.original_row_for_atom(atom) {
                            pieces.extend_from_slice(transform.original_pieces(row));
                        }
                    }
                    RelationEvent::Removal { row } => {
                        pieces.extend_from_slice(transform.original_pieces(row));
                    }
                    RelationEvent::PhaseIDeleted { source, .. } => {
                        append_source_pieces(layout, source, &mut pieces);
                    }
                    RelationEvent::Anchor { .. } => selected_slot = true,
                },
                SelectedRelationEvent::Indication { unit, atom } => {
                    if !selected.presentation.units[unit].atoms.contains(&atom)
                        || selected.presentation.selected_relation.atom_events[atom] != index
                    {
                        return Err(protocol_error(ProtocolError::InvalidPlacedPosition {
                            position: anchor,
                        }));
                    }
                    let crate::protocol::prepared::PresentationUnitKind::Replacement {
                        alternative,
                        ..
                    } = selected.presentation.units[unit].kind
                    else {
                        return Err(protocol_error(ProtocolError::InvalidPlacedPosition {
                            position: anchor,
                        }));
                    };
                    if let crate::protocol::prepared::AlternativeDefinition::Source {
                        source, ..
                    } = selected.prepared.tables.alternatives[alternative].definition
                    {
                        append_source_pieces(layout, source, &mut pieces);
                    }
                }
            }
        }
        offset += relation.events.len();
    }
    if end > offset {
        return Err(protocol_error(ProtocolError::InvalidPlacedPosition {
            position: focus,
        }));
    }
    let pieces = normalize_pieces(layout, pieces)?;
    let mut original = Vec::new();
    let mut coverage: Vec<Range> = Vec::new();
    for piece in &pieces {
        original.extend_from_slice(piece.code_units());
        if let Some(previous) = coverage.last_mut()
            && piece.aggregate.start <= previous.end
        {
            previous.end = previous.end.max(piece.aggregate.end);
        } else {
            coverage.push(piece.aggregate);
        }
    }
    let plaintext = layout
        .prepared
        .analyzed
        .processed
        .plaintext_for_coverage(&coverage)?;
    let geometry = if start == end {
        SelectionGeometry::default()
    } else {
        super::selection::with_membership(
            layout,
            first.line,
            last.line,
            |line, cluster| {
                let selected = &layout.lines[line].finalized.selected.presentation;
                cluster_atoms(&layout.lines[line], cluster).any(|atom| {
                    (start..end)
                        .contains(&(offsets[line] + selected.selected_relation.atom_events[atom]))
                })
            },
            |line| {
                let selected = &layout.lines[line].finalized.selected.presentation;
                selected
                    .projection
                    .atoms
                    .iter()
                    .enumerate()
                    .any(|(atom, value)| {
                        matches!(value.code_point, 0x0a | 0x0d)
                            && (start..end).contains(
                                &(offsets[line] + selected.selected_relation.atom_events[atom]),
                            )
                    })
            },
            |index, anchor| {
                let line = &layout.lines[index];
                let anchor = &line.finalized.data.anchors[anchor];
                let presentation = &line.finalized.selected.presentation;
                anchor.inline_end != anchor.slot.x
                    && presentation.selected_relation.unit_events[anchor.unit]
                        .clone()
                        .any(|event| {
                            (start..end).contains(&(offsets[index] + event))
                                && matches!(presentation.selected_relation.events[event], SelectedRelationEvent::Base { event }
                                    if matches!(line.finalized.selected.prepared.analyzed.transform.events()[event], RelationEvent::Anchor { .. }))
                        })
            },
            |index| {
                let line = &layout.lines[index];
                let presentation = &line.finalized.selected.presentation;
                let has_slot = presentation.units.iter().enumerate().any(|(unit, value)| {
                    matches!(
                        value.kind,
                        crate::protocol::prepared::PresentationUnitKind::Control { .. }
                            | crate::protocol::prepared::PresentationUnitKind::EmptyOrigin { .. }
                    ) && presentation.selected_relation.unit_events[unit]
                        .clone()
                        .any(|event| {
                            (start..end).contains(&(offsets[index] + event))
                            && matches!(presentation.selected_relation.events[event], SelectedRelationEvent::Base { event }
                                if matches!(line.finalized.selected.prepared.analyzed.transform.events()[event], RelationEvent::Anchor { .. }))
                        })
                });
                // Retain an endpoint's genuine source-less occurrence slot,
                // including zero width. A borrowed removal edge is not a slot.
                let endpoint_slot = [a, b]
                    .iter()
                    .any(|position| position.line == index && position.slot);
                if !selected_slot || (!has_slot && !endpoint_slot) {
                    return None;
                }
                let metrics = line.finalized.data.metrics;
                Some((
                    if index == first.line {
                        local_x(line, first.target)
                    } else {
                        metrics.inline_min
                    },
                    if index == last.line {
                        local_x(line, last.target)
                    } else {
                        metrics.inline_max
                    },
                ))
            },
        )?
    };
    Ok(crate::PlacedSelection {
        _placement: Arc::clone(layout),
        geometry,
        pieces,
        source_text: crate::Text::from_code_units(original),
        plaintext,
        collapsed: start == end,
    })
}

fn append_source_pieces(layout: &PlacedLayout, source: Range, pieces: &mut Vec<OriginalPiece>) {
    pieces.extend(
        layout
            .prepared
            .analyzed
            .paragraph
            .origin_intersections(&layout.prepared.analyzed.processed, source)
            .filter(|piece| !piece.aggregate.is_empty())
            .map(|piece| OriginalPiece {
                origin: piece.origin,
                aggregate: piece.aggregate,
                local: piece.local,
            }),
    );
}

fn normalize_pieces(
    layout: &PlacedLayout,
    mut pieces: Vec<OriginalPiece>,
) -> Result<Vec<crate::PlacedSelectionPiece>> {
    // Normalize original occurrences only after event coverage has been chosen.
    // Equal text in another occurrence remains distinct; holes stay holes.
    pieces.sort_unstable_by_key(|piece| {
        (
            piece.aggregate.start,
            piece.origin,
            piece.local.start,
            piece.local.end,
        )
    });
    let mut normalized: Vec<crate::PlacedSelectionPiece> = Vec::new();
    for piece in pieces {
        let origin = layout
            .prepared
            .analyzed
            .paragraph
            .origins
            .get(piece.origin)
            .ok_or_else(|| {
                protocol_error(ProtocolError::InvalidOriginRange {
                    origin: piece.origin,
                    local: piece.local,
                    aggregate: piece.aggregate,
                })
            })?;
        if piece.local.start < origin.local.start
            || piece.local.end > origin.local.end
            || piece.local.start > piece.local.end
            || piece.aggregate.start < origin.aggregate.start
            || piece.aggregate.end > origin.aggregate.end
            || piece.aggregate.start > piece.aggregate.end
            || piece.local.start - origin.local.start
                != piece.aggregate.start - origin.aggregate.start
            || piece.local.len() != piece.aggregate.len()
        {
            return Err(protocol_error(ProtocolError::InvalidOriginRange {
                origin: piece.origin,
                local: piece.local,
                aggregate: piece.aggregate,
            }));
        }
        if piece.local.is_empty() {
            continue;
        }
        if let Some(previous) = normalized.last_mut()
            && previous.origin.index == piece.origin
            && piece.local.start <= previous.local.end
            && piece.aggregate.start <= previous.aggregate.end
        {
            previous.local.end = previous.local.end.max(piece.local.end);
            previous.aggregate.end = previous.aggregate.end.max(piece.aggregate.end);
        } else {
            normalized.push(crate::PlacedSelectionPiece {
                origin: crate::PreparedOrigin {
                    prepared: Arc::clone(&layout.prepared),
                    index: piece.origin,
                },
                local: piece.local,
                aggregate: piece.aggregate,
            });
        }
    }
    Ok(normalized)
}

pub(crate) fn validate_source(layout: &PlacedLayout, value: Cursor) -> Result<()> {
    layout.prepared.analyzed.paragraph.admit(
        layout.prepared.analyzed.processed.source(),
        Range::new(value.index(), value.index()),
    )?;
    let coverage = layout.coverage.source;
    let paragraph = layout
        .prepared
        .analyzed
        .paragraph
        .extent(&layout.prepared.analyzed.processed);
    let outside_facing_start = coverage.start != paragraph.start
        && value.index() == coverage.start
        && value.affinity() == Affinity::Before;
    let outside_facing_end = coverage.end != paragraph.end
        && value.index() == coverage.end
        && value.affinity() == Affinity::After;
    if value.index() < coverage.start
        || value.index() > coverage.end
        || outside_facing_start
        || outside_facing_end
    {
        return Err(protocol_error(ProtocolError::OutsidePlacement {
            index: value.index(),
            coverage,
        }));
    }
    if layout.lines.is_empty() {
        return Err(protocol_error(ProtocolError::NoFormattedLineGeometry));
    }
    // Only distinct actual source-less occurrence rows create this ambiguity;
    // ordinary source edges continue to use the existing affinity policy.
    let mut first = None;
    for position in &layout.interaction.positions {
        if position.source.index() != value.index() {
            continue;
        }
        let identity = match position.target {
            Target::Anchor { anchor, .. } => {
                let line = &layout.lines[position.line];
                let unit = &line.finalized.selected.presentation.units
                    [line.finalized.data.anchors[anchor].unit];
                if !unit.source.is_empty() {
                    continue;
                }
                use crate::protocol::prepared::PresentationUnitKind;
                match unit.kind {
                    PresentationUnitKind::Replacement { alternative, .. } => {
                        (position.line, 1, alternative)
                    }
                    PresentationUnitKind::EmptyOrigin { origin } => (position.line, 2, origin),
                    _ => (position.line, 0, anchor),
                }
            }
            Target::Strut => (position.line, 3, 0),
            Target::Cluster { cluster, .. } => {
                let line = &layout.lines[position.line];
                let cluster = &line.finalized.data.clusters[cluster];
                let alternative =
                    line.finalized
                        .selected
                        .presentation
                        .units
                        .iter()
                        .find_map(|unit| {
                            if unit.presentation.start < cluster.presentation.end
                            && cluster.presentation.start < unit.presentation.end
                            && let crate::protocol::prepared::PresentationUnitKind::Replacement {
                                alternative,
                                ..
                            } = unit.kind
                        {
                            return Some(alternative);
                        }
                            None
                        });
                let Some(alternative) = alternative else {
                    continue;
                };
                (position.line, 1, alternative)
            }
        };
        if first.is_some_and(|old| old != identity) {
            return Err(protocol_error(ProtocolError::AmbiguousSourcePosition {
                index: value.index(),
                affinity: value.affinity(),
            }));
        }
        first = Some(identity);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::paragraph::{
        ParagraphAnchor, ParagraphAnchorItem, ParagraphControl, ParagraphControlKind,
    };
    use crate::protocol::*;
    use crate::{
        Alignment, AnalyzedSource, DocumentSegmentBreaks, Font, FontCatalogueMode, Layout,
        LineHeight, Source, Style, System, SystemOptions,
    };

    fn transformed(text: &str, case: crate::CaseTransform, locale: Option<&str>) -> Layout {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(crate::shaping_test_fonts::casing_ahem())
            .unwrap();
        system
            .layout(
                Source::new(text),
                Style {
                    font: Font::new().family("Ahem"),
                    size: 10.0,
                    line_height: LineHeight::Absolute(10.0),
                    white_space: crate::WhiteSpace::Preserve,
                    text_transform: crate::TextTransform::Set(
                        crate::TextTransformSet::none().with_case(case),
                    ),
                    locale: locale.map(str::to_owned),
                    ..Style::default()
                },
                crate::Options {
                    alignment: Alignment::Left,
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap()
    }

    fn at_cut(layout: &Layout, cut: usize) -> crate::PlacedPosition {
        layout
            .positions()
            .find(|position| layout.inner.interaction.positions[position.index].cut == cut)
            .expect("actual selected relation cut")
    }

    #[test]
    fn expansion_emission_selection_extracts_one_original_occurrence() {
        // Original Ahem owns the independently qualified ß134/S53 cmap;
        // the finite Greek/mark derivative deliberately does not map ASCII S.
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let layout = system
            .layout(
                Source::new("ß"),
                Style {
                    text_transform: crate::TextTransform::Set(
                        crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
                    ),
                    ..native_style()
                },
                crate::Options {
                    alignment: Alignment::Left,
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let glyphs: Vec<_> = layout
            .run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect();
        assert_eq!(glyphs, [53, 53]);
        assert_eq!(layout.lines()[0].bounds().size.width, 20.0);
        let (start, middle, end) = (at_cut(&layout, 0), at_cut(&layout, 1), at_cut(&layout, 2));
        let second = layout.position_selection(&middle, &end).unwrap();
        assert!(!second.is_collapsed());
        assert_eq!(second.source_text().code_units(), &[0xdf]);
        assert_eq!(second.plaintext().code_units(), &[0xdf]);
        assert_eq!(second.source_pieces().len(), 1);
        assert_eq!(second.source_pieces()[0].local_range(), Range::new(0, 1));
        assert_eq!(second.geometry().rects().len(), 1);
        assert_eq!(
            second.geometry().rects()[0].rect(),
            Rect::new(10.0, 0.0, 10.0, 10.0)
        );
        let whole = layout.position_selection(&start, &end).unwrap();
        assert_eq!(whole.source_text().code_units(), &[0xdf]);
        assert_eq!(whole.geometry().rects()[0].rect().size.width, 20.0);
        let reverse = layout.position_selection(&end, &middle).unwrap();
        assert_eq!(reverse.source_text(), second.source_text());
        assert_eq!(reverse.geometry(), second.geometry());
        let aliases: Vec<_> = layout
            .positions()
            .filter(|position| layout.inner.interaction.positions[position.index].cut == 1)
            .collect();
        assert_eq!(aliases.len(), 2);
        let collapsed = layout.position_selection(&aliases[0], &aliases[1]).unwrap();
        assert!(collapsed.is_collapsed());
        assert!(collapsed.source_text().is_empty());
        assert!(collapsed.geometry().rects().is_empty());
    }

    #[test]
    fn removed_dot_has_distinct_cuts_without_a_rectangle_or_emission_coverage() {
        let layout = transformed("I\u{307}B", crate::CaseTransform::Lowercase, Some("tr"));
        assert_eq!(layout.source().text().code_units(), &[0x49, 0x307, 0x42]);
        assert_eq!(layout.lines()[0].bounds().size.width, 20.0);
        let (start, before, after, end) = (
            at_cut(&layout, 0),
            at_cut(&layout, 1),
            at_cut(&layout, 2),
            at_cut(&layout, 3),
        );
        assert_eq!(
            layout.position_geometry(&before).unwrap(),
            layout.position_geometry(&after).unwrap()
        );
        let removed = layout.position_selection(&before, &after).unwrap();
        assert!(!removed.is_collapsed());
        assert_eq!(removed.source_text().code_units(), &[0x307]);
        assert_eq!(removed.plaintext().code_units(), &[0x307]);
        assert!(removed.geometry().rects().is_empty());
        let first = layout.position_selection(&start, &before).unwrap();
        assert_eq!(first.source_text().code_units(), &[0x49]);
        assert_eq!(first.geometry().rects()[0].rect().size.width, 10.0);
        let whole = layout.position_selection(&start, &end).unwrap();
        assert_eq!(whole.source_text().code_units(), &[0x49, 0x307, 0x42]);
        assert_eq!(whole.source_pieces().len(), 1);
        let reverse = layout.position_selection(&after, &before).unwrap();
        assert_eq!(reverse.source_text(), removed.source_text());
        assert_eq!(reverse.geometry(), removed.geometry());
    }

    #[test]
    fn original_utf16_and_plaintext_are_separate_selected_observations() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let layout = system
            .layout(
                Source::new(crate::Text::from_code_units(vec![0x41, 0xd800, 0x42])),
                Style {
                    font: Font::new().family("Ahem"),
                    size: 10.0,
                    ..Style::default()
                },
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let positions: Vec<_> = layout.positions().collect();
        let selection = layout
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(selection.source_text().code_units(), &[0x41, 0xd800, 0x42]);
        assert_eq!(
            selection.source_pieces()[0].code_units(),
            &[0x41, 0xd800, 0x42]
        );
        assert_eq!(selection.plaintext().code_units(), &[0x41, 0xd800, 0x42]);
        let clone = layout.clone();
        assert_eq!(
            clone
                .position_selection(positions.first().unwrap(), positions.last().unwrap())
                .unwrap()
                .source_text(),
            selection.source_text()
        );
    }

    fn native_selection_layout(
        source: Source,
        origins: Vec<crate::ParagraphOrigin>,
        style: Style,
        controls: Vec<ParagraphControl>,
        base_direction: ParagraphBaseDirection,
        font: Vec<u8>,
    ) -> Layout {
        // This fixture captures adjacent contributions in one continuous inline
        // context, with the same real governing wrapping policy on both sides.
        let connections: Vec<_> = origins
            .windows(2)
            .map(|pair| crate::ParagraphConnection {
                at: pair[0].aggregate.end,
                kind: crate::ParagraphConnectionKind::Continuous,
            })
            .collect();
        let boundary_policies = (0..connections.len())
            .map(|index| crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(index),
                preceding_wrap: style.wrap,
                following_wrap: style.wrap,
                ancestor_wrap: style.wrap,
            })
            .collect();
        let anchor_order = (0..controls.len())
            .map(|index| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(index),
                affinity: Affinity::After,
            })
            .collect();
        let analyzed = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source,
                base_style: style,
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction,
                paragraph_strut: crate::ParagraphStrut::DefaultFont,
                origins,
                connections,
                controls,
                boundary_policies,
                replacements: vec![],
                anchor_order,
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(font).unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let start = prepared.boundary_views().next().unwrap().handle();
        let termination = prepared
            .opportunity_views()
            .find(|opportunity| opportunity.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let end = termination.view().boundary();
        let line = prepared
            .select(LineSelectionRequest {
                boundary_context: Vec::new(),
                incoming: None,
                start,
                end,
                termination: crate::PreparedTermination::Ordinary(termination),
                participants: prepared.participants().collect(),
                alternative: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    formatted_line: crate::FormattedLineContext::default(),
                    base_direction,
                    line_box_direction: match base_direction {
                        crate::ParagraphBaseDirection::LeftToRight => {
                            crate::LineBoxDirection::LeftToRight
                        }
                        crate::ParagraphBaseDirection::RightToLeft => {
                            crate::LineBoxDirection::RightToLeft
                        }
                    },
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Left,
                    tabs: TabContext::NoTabs,
                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls: prepared
                    .participants()
                    .filter_map(|participant| match participant {
                        PreparedParticipant::Control(occurrence) => Some({
                            ControlAllocation {
                                boundary: matches!(
                                    occurrence.view().kind(),
                                    crate::ParagraphControlKind::InlineBoundaryStart
                                        | crate::ParagraphControlKind::InlineBoundaryEnd
                                )
                                .then_some(crate::InlineBoundaryAllocation::ZERO),
                                occurrence,
                                strut: None,
                            }
                        }),
                        _ => None,
                    })
                    .collect(),
                line_box_struts: Vec::new(),
                line_strut: None,
            })
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 8.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap()
    }

    fn native_style() -> Style {
        Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            ..Style::default()
        }
    }

    #[test]
    fn equal_text_from_distinct_original_occurrences_is_not_deduplicated() {
        let source = Source::new("AA");
        // The exact same immutable resource occurs twice. Occurrence identity,
        // not text, ID, revision or resource identity, controls deduplication.
        let original = Source::identified("A", crate::Id::from_u64(1), 3);
        let originals = [original.clone(), original];
        let origins = originals
            .iter()
            .enumerate()
            .map(|(index, source)| crate::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(0, 1),
                aggregate: Range::new(index, index + 1),
                kind: crate::ParagraphOriginKind::Authored,
            })
            .collect();
        let layout = native_selection_layout(
            source,
            origins,
            native_style(),
            vec![],
            ParagraphBaseDirection::LeftToRight,
            crate::shaping_test_fonts::casing_ahem(),
        );
        let selection = layout
            .position_selection(&at_cut(&layout, 0), &at_cut(&layout, 2))
            .unwrap();
        assert_eq!(selection.source_text().code_units(), &[0x41, 0x41]);
        assert_eq!(selection.source_pieces().len(), 2);
        for (piece, source) in selection.source_pieces().iter().zip(&originals) {
            assert_eq!(piece.origin().view().source(), source);
            assert_eq!(piece.local_range(), Range::new(0, 1));
            assert_eq!(piece.code_units(), &[0x41]);
        }
        assert_ne!(
            selection.source_pieces()[0].origin(),
            selection.source_pieces()[1].origin()
        );
    }

    #[test]
    fn native_plaintext_uses_executed_collapse_while_original_extraction_retains_spaces() {
        let source = Source::new("A  B");
        let origins = vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, 4),
            aggregate: Range::new(0, 4),
            kind: crate::ParagraphOriginKind::Authored,
        }];
        let layout = native_selection_layout(
            source,
            origins,
            Style {
                white_space: crate::WhiteSpace::Collapse,
                ..native_style()
            },
            vec![],
            ParagraphBaseDirection::LeftToRight,
            crate::shaping_test_fonts::casing_ahem(),
        );
        let positions: Vec<_> = layout.positions().collect();
        let selection = layout
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(
            selection.source_text().code_units(),
            &[0x41, 0x20, 0x20, 0x42]
        );
        assert_eq!(selection.plaintext().code_units(), &[0x41, 0x20, 0x42]);
        assert_eq!(selection.source_pieces().len(), 1);
    }

    #[test]
    fn actual_shared_ligature_and_mark_keep_only_authentic_outer_edges() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(crate::shaping_test_fonts::ligature_ahem())
            .unwrap();
        let layout = system
            .layout(
                Source::new("AA"),
                native_style(),
                crate::Options {
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let glyphs: Vec<_> = layout
            .run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect();
        assert_eq!(glyphs, [58]);
        let positions: Vec<_> = layout.positions().collect();
        assert_eq!(positions.len(), 2);
        let selection = layout
            .position_selection(&positions[0], &positions[1])
            .unwrap();
        assert_eq!(selection.source_text().code_units(), &[0x41, 0x41]);
        assert_eq!(selection.geometry().rects()[0].rect().size.width, 10.0);
        let marked = transformed("i\u{307}", crate::CaseTransform::None, None);
        assert_eq!(marked.positions().count(), 2);
        let positions: Vec<_> = marked.positions().collect();
        let selection = marked
            .position_selection(&positions[0], &positions[1])
            .unwrap();
        assert_eq!(selection.source_text().code_units(), &[0x69, 0x307]);
    }

    #[test]
    fn reordered_greek_emission_selection_uses_event_order_then_original_piece_order() {
        // Root uppercase decomposes the source's subscript iota to a delayed
        // capital Iota AFTER the following original acute: 1F08,0301,0399.
        // Unicode canonical composition shapes the first two as 1F0C, glyph189.
        let layout = transformed("\u{1f80}\u{301}", crate::CaseTransform::Uppercase, None);
        let glyphs: Vec<_> = layout
            .run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect();
        assert_eq!(glyphs, [189, 43]);
        assert_eq!(layout.source().text().code_units(), &[0x1f80, 0x301]);
        let (start, middle, end) = (at_cut(&layout, 0), at_cut(&layout, 2), at_cut(&layout, 3));
        let delayed = layout.position_selection(&middle, &end).unwrap();
        assert_eq!(delayed.source_text().code_units(), &[0x1f80]);
        assert_eq!(delayed.source_pieces().len(), 1);
        assert_eq!(delayed.source_pieces()[0].local_range(), Range::new(0, 1));
        assert_eq!(
            delayed.geometry().rects()[0].rect(),
            Rect::new(10.0, 0.0, 10.0, 10.0)
        );
        let first = layout.position_selection(&start, &middle).unwrap();
        assert_eq!(first.source_text().code_units(), &[0x1f80, 0x301]);
        let all = layout.position_selection(&start, &end).unwrap();
        assert_eq!(all.source_text().code_units(), &[0x1f80, 0x301]);
        assert_eq!(all.source_pieces().len(), 1);
        assert_eq!(all.geometry().rects()[0].rect().size.width, 20.0);
    }

    #[test]
    fn trailing_removal_belongs_to_the_whole_line_without_extending_ink_geometry() {
        let layout = transformed("I\u{307}", crate::CaseTransform::Lowercase, Some("tr"));
        let positions: Vec<_> = layout.positions().collect();
        assert_eq!(
            layout.inner.interaction.positions[positions.first().unwrap().index].cut,
            0
        );
        assert_eq!(
            layout.inner.interaction.positions[positions.last().unwrap().index].cut,
            2
        );
        let whole = layout
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(whole.source_text().code_units(), &[0x49, 0x307]);
        assert_eq!(whole.geometry().rects()[0].rect().size.width, 10.0);
        let removed = layout
            .position_selection(&at_cut(&layout, 1), &at_cut(&layout, 2))
            .unwrap();
        assert!(!removed.is_collapsed());
        assert_eq!(removed.source_text().code_units(), &[0x307]);
        assert!(removed.geometry().rects().is_empty());
    }

    #[test]
    fn removal_adjoining_a_real_zero_width_control_does_not_select_its_slot() {
        let source = Source::new("I\u{307}B");
        let origins = vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, 3),
            aggregate: Range::new(0, 3),
            kind: crate::ParagraphOriginKind::Authored,
        }];
        let layout = native_selection_layout(
            source,
            origins,
            Style {
                locale: Some("tr".to_owned()),
                text_transform: crate::TextTransform::Set(
                    crate::TextTransformSet::none().with_case(crate::CaseTransform::Lowercase),
                ),
                ..native_style()
            },
            vec![ParagraphControl {
                at: 2,
                kind: ParagraphControlKind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                },
            }],
            ParagraphBaseDirection::LeftToRight,
            crate::shaping_test_fonts::casing_ahem(),
        );
        let removed = layout
            .position_selection(&at_cut(&layout, 1), &at_cut(&layout, 2))
            .unwrap();
        assert!(!removed.is_collapsed());
        assert_eq!(removed.source_text().code_units(), &[0x307]);
        assert!(removed.geometry().rects().is_empty());
        let control = layout
            .position_selection(&at_cut(&layout, 2), &at_cut(&layout, 3))
            .unwrap();
        assert!(!control.is_collapsed());
        assert!(control.source_text().is_empty());
        assert_eq!(control.geometry().rects().len(), 1);
        assert_eq!(control.geometry().rects()[0].rect().size.width, 0.0);
    }

    #[test]
    fn kana_contraction_keeps_the_supplementary_original_pair_and_actual_five_pixel_glyph() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(crate::shaping_test_fonts::kana_ahem())
            .unwrap();
        let layout = system
            .layout(
                Source::new("A\u{1b132}C"),
                Style {
                    text_transform: crate::TextTransform::Set(
                        crate::TextTransformSet::none().with_full_size_kana(true),
                    ),
                    ..native_style()
                },
                crate::Options {
                    alignment: Alignment::Left,
                    quantize: false,
                    ..crate::Options::default()
                },
            )
            .unwrap();
        let glyphs: Vec<_> = layout
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(glyphs, [(35, 10.0), (36, 5.0), (37, 10.0)]);
        let selected = layout
            .position_selection(&at_cut(&layout, 1), &at_cut(&layout, 2))
            .unwrap();
        assert_eq!(selected.source_text().code_units(), &[0xd82c, 0xdd32]);
        assert_eq!(selected.source_pieces()[0].local_range(), Range::new(1, 3));
        assert_eq!(
            selected.source_pieces()[0].aggregate_range(),
            Range::new(1, 3)
        );
        assert_eq!(selected.plaintext().code_units(), &[0xd82c, 0xdd32]);
        assert_eq!(
            selected.geometry().rects()[0].rect(),
            Rect::new(10.0, 0.0, 5.0, 10.0)
        );
    }

    #[test]
    fn real_rtl_override_expansion_keeps_both_interior_affinities_and_exact_coverage() {
        let source = Source::new("ß");
        let origins = vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1),
            kind: crate::ParagraphOriginKind::Authored,
        }];
        let layout = native_selection_layout(
            source,
            origins,
            Style {
                text_transform: crate::TextTransform::Set(
                    crate::TextTransformSet::none().with_case(crate::CaseTransform::Uppercase),
                ),
                ..native_style()
            },
            vec![
                ParagraphControl {
                    at: 0,
                    kind: ParagraphControlKind::RightToLeftOverride,
                },
                ParagraphControl {
                    at: 1,
                    kind: ParagraphControlKind::PopDirectionalFormat,
                },
            ],
            ParagraphBaseDirection::RightToLeft,
            include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        );
        let line = &layout.inner.lines[0];
        assert_eq!(line.finalized.data.clusters.len(), 2);
        assert!(
            line.finalized
                .data
                .clusters
                .iter()
                .all(|cluster| cluster.level & 1 == 1)
        );
        let visual_atoms: Vec<_> = line
            .finalized
            .data
            .visual_order
            .iter()
            .filter_map(|item| {
                if let FinalizedVisualItem::Cluster(cluster) = *item {
                    Some(cluster_atoms(line, cluster).start)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            visual_atoms,
            [1, 0],
            "actual odd-level S emissions reverse physically"
        );
        let glyphs: Vec<_> = layout
            .run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.id()).collect::<Vec<_>>())
            .collect();
        assert_eq!(glyphs, [53, 53]);
        let aliases: Vec<_> = layout
            .positions()
            .filter(|position| layout.inner.interaction.positions[position.index].cut == 2)
            .collect();
        assert_eq!(aliases.len(), 2);
        assert!(
            aliases
                .iter()
                .any(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
        );
        assert!(
            aliases
                .iter()
                .any(|position| position.source_cursor() == Cursor::new(1, Affinity::Before))
        );
        for alias in &aliases {
            assert_eq!(
                layout.position_geometry(alias).unwrap().rect(),
                Rect::new(10.0, 0.0, 1.0, 10.0)
            );
        }
        let collapsed = layout.position_selection(&aliases[0], &aliases[1]).unwrap();
        assert!(collapsed.is_collapsed());
        assert!(collapsed.source_text().is_empty());
        assert!(collapsed.geometry().rects().is_empty());
        let first = layout
            .position_selection(&at_cut(&layout, 1), &aliases[0])
            .unwrap();
        let second = layout
            .position_selection(&aliases[1], &at_cut(&layout, 3))
            .unwrap();
        assert_eq!(
            first.geometry().rects()[0].rect(),
            Rect::new(10.0, 0.0, 10.0, 10.0)
        );
        assert_eq!(
            second.geometry().rects()[0].rect(),
            Rect::new(0.0, 0.0, 10.0, 10.0)
        );
        assert_eq!(first.source_text().code_units(), &[0xdf]);
        assert_eq!(second.source_text().code_units(), &[0xdf]);
        let reverse = layout
            .position_selection(&at_cut(&layout, 3), &aliases[1])
            .unwrap();
        assert_eq!(reverse.source_text(), second.source_text());
        assert_eq!(reverse.geometry(), second.geometry());
        let whole = layout
            .position_selection(&at_cut(&layout, 1), &at_cut(&layout, 3))
            .unwrap();
        assert_eq!(whole.source_text().code_units(), &[0xdf]);
        assert_eq!(whole.source_pieces().len(), 1);
        assert_eq!(whole.geometry().rects()[0].rect().size.width, 20.0);
    }

    fn four_source_less_lines() -> Layout {
        native_four_lines(false)
    }

    fn native_four_lines(with_text: bool) -> Layout {
        native_break_lines(
            if with_text { "A" } else { "" },
            usize::from(with_text),
            3,
            false,
        )
    }

    fn native_break_lines(text: &str, at: usize, breaks: usize, disconnected: bool) -> Layout {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new(text);
        let source_end = source.text().len();
        let analyzed = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    font: Font::new().family("Ahem"),
                    size: 20.0,
                    line_height: LineHeight::FontSizeRelative(1.0),
                    ..Style::default()
                },
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: crate::protocol::paragraph::ParagraphStrut::DefaultFont,
                origins: if disconnected {
                    assert_eq!(text, "AB");
                    vec![
                        crate::ParagraphOrigin {
                            text_node: None,
                            source: Source::new("A"),
                            local: Range::new(0, 1),
                            aggregate: Range::new(0, 1),
                            kind: crate::ParagraphOriginKind::Authored,
                        },
                        crate::ParagraphOrigin {
                            text_node: None,
                            source: Source::new("B"),
                            local: Range::new(0, 1),
                            aggregate: Range::new(1, 2),
                            kind: crate::ParagraphOriginKind::Authored,
                        },
                    ]
                } else if source_end != 0 {
                    vec![crate::ParagraphOrigin {
                        text_node: None,
                        source,
                        local: Range::new(0, source_end),
                        aggregate: Range::new(0, source_end),
                        kind: crate::ParagraphOriginKind::Authored,
                    }]
                } else {
                    Vec::new()
                },
                connections: if disconnected {
                    vec![crate::ParagraphConnection {
                        at: 1,
                        kind: crate::ParagraphConnectionKind::Disconnected,
                    }]
                } else {
                    Vec::new()
                },
                controls: vec![
                    ParagraphControl {
                        at,
                        kind: ParagraphControlKind::ForcedBreak
                    };
                    breaks
                ],
                boundary_policies: Vec::new(),
                replacements: Vec::new(),
                anchor_order: (if breaks == 3 {
                    vec![2, 0, 1]
                } else {
                    (0..breaks).collect()
                })
                .into_iter()
                .map(|index| ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(index),
                    affinity: Affinity::After,
                })
                .collect(),
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let participants: Vec<_> = prepared.participants().collect();
        let boundaries: Vec<_> = prepared
            .boundary_views()
            .map(|view| view.handle())
            .collect();
        let opportunities: Vec<_> = prepared
            .opportunity_views()
            .filter(|view| {
                matches!(
                    view.kind(),
                    PreparedBreakKind::Forced | PreparedBreakKind::ParagraphEnd
                )
            })
            .map(|view| view.handle())
            .collect();
        assert_eq!(participants.len(), source_end + breaks);
        assert_eq!(opportunities.len(), breaks + 1);
        let mut start = boundaries[0].clone();
        let mut from = 0;
        let mut lines = Vec::new();
        for (index, opportunity) in opportunities.into_iter().enumerate() {
            let end = opportunity.view().boundary();
            let to = end.view().participant_offset();
            let selected = prepared
                .select(LineSelectionRequest {
                    boundary_context: Vec::new(),
                    incoming: None,
                    start: start.clone(),
                    end: end.clone(),
                    participants: participants[from..to].to_vec(),
                    termination: crate::PreparedTermination::Ordinary(opportunity),
                    alternative: None,
                    context: LineContext {
                        alignment_insets: crate::LineAlignmentInsets::ZERO,
                        formatted_line: crate::FormattedLineContext::default(),
                        base_direction: ParagraphBaseDirection::LeftToRight,
                        line_box_direction: crate::LineBoxDirection::LeftToRight,
                        orientation: LineOrientation::Horizontal,
                        start: if index == 0 {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if index == breaks {
                            LineEnd::Paragraph
                        } else {
                            LineEnd::Forced
                        },
                        available_inline: None,
                        indent: 0.0,
                        alignment: Alignment::Start,
                        tabs: TabContext::NoTabs,

                        spacing: SpacingPolicy::None,
                    },
                    atomics: Vec::new(),
                    line_box_struts: Vec::new(),
                    line_strut: None,
                    controls: participants[from..to]
                        .iter()
                        .filter_map(|participant| {
                            if let PreparedParticipant::Control(control) = participant {
                                Some({
                                    let occurrence = control.clone();
                                    ControlAllocation {
                                        boundary: matches!(
                                            occurrence.view().kind(),
                                            crate::ParagraphControlKind::InlineBoundaryStart
                                                | crate::ParagraphControlKind::InlineBoundaryEnd
                                        )
                                        .then_some(crate::InlineBoundaryAllocation::ZERO),
                                        occurrence,
                                        strut: None,
                                    }
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                })
                .unwrap();
            let expected_control = selected
                .request()
                .participants
                .iter()
                .find_map(|participant| {
                    if let PreparedParticipant::Control(control) = participant {
                        Some(control.clone())
                    } else {
                        None
                    }
                });
            let line = selected.finalize(SpacingAllocations::None).unwrap();
            if let Some(expected) = expected_control {
                assert_eq!(line.control_slots().next().unwrap().control(), expected);
                assert!(line.fragment_views().any(|fragment| matches!(fragment.owner(),
                    crate::protocol::readback::FinalizedFragmentOwner::Control(actual) if actual == expected)));
            }
            lines.push(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + index as f64 * 20.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: Vec::new(),
            });
            start = end;
            from = to;
        }
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: lines
                .into_iter()
                .map(crate::PlacementEntry::Formatted)
                .collect(),
        })
        .unwrap()
    }

    #[test]
    fn genuine_same_source_controls_retain_middle_positions_and_foreign_placement() {
        let layout = four_source_less_lines();
        let positions: Vec<_> = layout.positions().collect();
        assert_eq!(positions.len(), 4);
        let metric_runs = layout.glyph_runs();
        assert_eq!(metric_runs.len(), 4);
        assert!(metric_runs.iter().all(|run| run.glyphs().is_empty()));
        assert_eq!(
            metric_runs
                .iter()
                .map(|run| run.style().size)
                .collect::<Vec<_>>(),
            [20.0; 4]
        );
        assert!(
            metric_runs
                .iter()
                .all(|run| run.style() == layout.default_style())
        );
        assert!(
            positions
                .iter()
                .all(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
        );
        let a = &positions[1];
        let b = &positions[2];
        assert_ne!(a, b);
        assert_eq!(
            layout.position_geometry(a).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_eq!(
            layout.position_geometry(b).unwrap().rect(),
            Rect::new(0.0, 40.0, 1.0, 20.0)
        );
        assert_eq!(layout.move_position(a, Movement::NextLine).unwrap(), *b);
        assert_eq!(
            layout.move_position(b, Movement::PreviousCluster).unwrap(),
            *a
        );
        let selection = layout.position_selection(a, b).unwrap();
        assert_eq!(selection.geometry().rects().len(), 2);
        assert_eq!(selection.geometry().rects()[0].line(), 1);
        assert_eq!(selection.geometry().rects()[1].line(), 2);
        let clone = layout.clone();
        assert_eq!(
            clone.position_geometry(a).unwrap(),
            layout.position_geometry(a).unwrap()
        );
        let foreign = four_source_less_lines();
        assert!(foreign.position_geometry(a).is_err());
        assert!(
            layout
                .position_selection(a, &foreign.positions().nth(2).unwrap())
                .is_err()
        );
        assert!(layout.cursor(Cursor::new(0, Affinity::After)).is_err());
        assert_eq!(layout.source().text().code_units(), &[]);
    }

    fn partial_word_line() -> Layout {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new("abcdef");
        let analysis = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    font: Font::new().family("Ahem"),
                    size: 20.0,
                    line_height: LineHeight::FontSizeRelative(1.0),
                    ..Style::default()
                },
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: crate::ParagraphStrut::DefaultFont,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, 6),
                    aggregate: Range::new(0, 6),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![ParagraphControl {
                    at: 4,
                    kind: ParagraphControlKind::OptionalBreak {
                        wrap: crate::Wrap::Word,
                    },
                }],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(0),
                    affinity: Affinity::After,
                }],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let prepared = system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap();
        let start = prepared
            .boundary_views()
            .find(|view| view.source_before().get() == 2)
            .unwrap()
            .handle();
        let termination = prepared
            .opportunity_views()
            .find(|view| {
                view.kind() == PreparedBreakKind::Allowed
                    && view.boundary().view().source_before().get() == 4
            })
            .unwrap()
            .handle();
        let end = termination.view().boundary();
        let participants: Vec<_> = prepared
            .participants()
            .skip(start.view().participant_offset())
            .take(end.view().participant_offset() - start.view().participant_offset())
            .collect();
        let controls = participants
            .iter()
            .filter_map(|participant| {
                if let PreparedParticipant::Control(control) = participant {
                    Some({
                        let occurrence = control.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::ZERO),
                            occurrence,
                            strut: None,
                        }
                    })
                } else {
                    None
                }
            })
            .collect();
        let selected = prepared
            .select(LineSelectionRequest {
                boundary_context: Vec::new(),
                incoming: None,
                start: start.clone(),
                end: end.clone(),
                participants,
                termination: crate::PreparedTermination::Ordinary(termination),
                alternative: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    formatted_line: crate::FormattedLineContext::default(),
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    line_box_direction: crate::LineBoxDirection::LeftToRight,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Continuation,
                    end: LineEnd::Soft,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,

                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls,
                line_box_struts: Vec::new(),
                line_strut: None,
            })
            .unwrap();
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::Range { start, end },
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap()
    }

    fn refuses_global_target(movement: Movement, index: usize) {
        let layout = partial_word_line();
        assert_eq!(layout.lines()[0].range(), Range::new(2, 4));
        let error=layout.move_cursor(Cursor::new(3,Affinity::After),movement).expect_err("a source-global target outside this actual placement cannot acquire its edge geometry");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::OutsidePlacement {
                    index,
                    coverage: Range::new(2, 4)
                }
            })
        );
    }
    #[test]
    fn partial_source_document_start_refuses_the_uncovered_original_start() {
        refuses_global_target(Movement::DocumentStart, 0);
    }
    #[test]
    fn partial_source_document_end_refuses_the_uncovered_original_end() {
        refuses_global_target(Movement::DocumentEnd, 6);
    }
    #[test]
    fn partial_source_previous_word_refuses_the_uncovered_original_word_start() {
        refuses_global_target(Movement::PreviousWord, 0);
    }
    #[test]
    fn partial_source_next_word_refuses_the_uncovered_original_word_end() {
        refuses_global_target(Movement::NextWord, 6);
    }
    #[test]
    fn partial_source_exact_positions_keep_their_real_placement_relative_edges() {
        let layout = partial_word_line();
        let positions: Vec<_> = layout.positions().collect();
        let middle = positions
            .iter()
            .find(|position| position.source_cursor().index() == 3)
            .unwrap();
        assert_eq!(
            layout
                .move_position(middle, Movement::DocumentStart)
                .unwrap(),
            positions[0]
        );
        assert_eq!(
            layout.move_position(middle, Movement::DocumentEnd).unwrap(),
            *positions.last().unwrap()
        );
        assert_eq!(
            layout
                .position_geometry(&positions[0])
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .position_geometry(positions.last().unwrap())
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        let previous_word = layout
            .move_position(middle, Movement::PreviousWord)
            .unwrap();
        let next_word = layout.move_position(middle, Movement::NextWord).unwrap();
        assert_eq!(previous_word.source_cursor().index(), 2);
        assert_eq!(next_word.source_cursor().index(), 4);
        assert_eq!(
            layout
                .position_geometry(&previous_word)
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .position_geometry(&next_word)
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
    }

    #[test]
    fn partial_source_raw_endpoint_affinity_requires_the_selected_side() {
        let layout = partial_word_line();
        for (cursor, x) in [
            (Cursor::new(2, Affinity::After), 0.0),
            (Cursor::new(4, Affinity::Before), 40.0),
        ] {
            assert_eq!(layout.cursor(cursor).unwrap().rect().origin.x, x);
        }
        for cursor in [
            Cursor::new(2, Affinity::Before),
            Cursor::new(4, Affinity::After),
        ] {
            assert_eq!(
                layout.cursor(cursor).unwrap_err().detail(),
                Some(&crate::ErrorDetail::InvalidTextProtocol {
                    reason: ProtocolError::OutsidePlacement {
                        index: cursor.index(),
                        coverage: Range::new(2, 4)
                    }
                })
            );
        }
        // This exact native-control position is covered even though its source
        // projection's downstream authored side lies outside the placement.
        let control = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(4, Affinity::After))
            .unwrap();
        assert_eq!(
            layout.position_geometry(&control).unwrap().rect().origin.x,
            40.0
        );
    }

    fn native_whitespace_line(text: &str, white_space: crate::WhiteSpace) -> Layout {
        native_whitespace_line_with_direction(
            text,
            white_space,
            ParagraphBaseDirection::LeftToRight,
        )
    }

    fn native_whitespace_prepared(
        text: &str,
        white_space: crate::WhiteSpace,
        base_direction: ParagraphBaseDirection,
    ) -> PreparedText {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let source = Source::new(text);
        let end = text.encode_utf16().count();
        let analysis = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    font: Font::new().family("Ahem"),
                    size: 20.0,
                    white_space,
                    line_height: LineHeight::FontSizeRelative(1.0),
                    ..Style::default()
                },
                segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
                base_direction,
                paragraph_strut: crate::ParagraphStrut::DefaultFont,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source,
                    local: Range::new(0, end),
                    aggregate: Range::new(0, end),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let analysis = crate::test_wrapping::bind_missing(&analysis, crate::Wrap::Word);
        system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap()
    }

    fn native_whitespace_line_with_direction(
        text: &str,
        white_space: crate::WhiteSpace,
        base_direction: ParagraphBaseDirection,
    ) -> Layout {
        let prepared = native_whitespace_prepared(text, white_space, base_direction);
        let termination = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let selected = prepared
            .select(LineSelectionRequest {
                boundary_context: Vec::new(),
                incoming: None,
                start: prepared.boundary_views().next().unwrap().handle(),
                end: termination.view().boundary(),
                participants: prepared.participants().collect(),
                termination: crate::PreparedTermination::Ordinary(termination),
                alternative: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    formatted_line: crate::FormattedLineContext::default(),
                    base_direction,
                    line_box_direction: match base_direction {
                        crate::ParagraphBaseDirection::LeftToRight => {
                            crate::LineBoxDirection::LeftToRight
                        }
                        crate::ParagraphBaseDirection::RightToLeft => {
                            crate::LineBoxDirection::RightToLeft
                        }
                    },
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,

                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls: vec![],
                line_box_struts: Vec::new(),
                line_strut: None,
            })
            .unwrap();
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap()
    }

    #[test]
    fn native_collapsed_gap_next_word_uses_the_next_actual_source_cluster() {
        let preserved = native_whitespace_line("A B", crate::WhiteSpace::Preserve);
        assert_eq!(
            preserved
                .move_cursor(Cursor::new(1, Affinity::After), Movement::NextWord)
                .unwrap(),
            Cursor::new(2, Affinity::Before)
        );
        assert_eq!(preserved.source().text().code_units(), [0x41, 0x20, 0x42]);
        let collapsed = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        assert_eq!(
            collapsed
                .analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [0x41, 0x20, 0x42]
        );
        assert_eq!(
            collapsed.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            collapsed
                .clusters()
                .iter()
                .map(|cluster| cluster.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2), Range::new(3, 4)]
        );
        let moved = collapsed
            .move_cursor(Cursor::new(1, Affinity::After), Movement::NextWord)
            .unwrap();
        assert_eq!(moved, Cursor::new(3, Affinity::Before));
        let b_start = collapsed
            .clusters()
            .into_iter()
            .find(|cluster| cluster.range() == Range::new(3, 4))
            .unwrap()
            .range()
            .start;
        assert_eq!(moved.index(), b_start);
        assert_eq!(collapsed.cursor(moved).unwrap().rect().origin.x, 40.0);
        assert_eq!(
            collapsed.move_cursor(moved, Movement::NextWord).unwrap(),
            Cursor::new(4, Affinity::Before)
        );
    }

    #[test]
    fn native_collapsed_gap_previous_word_uses_the_previous_actual_source_cluster() {
        let preserved = native_whitespace_line("A B", crate::WhiteSpace::Preserve);
        assert_eq!(
            preserved
                .move_cursor(Cursor::new(2, Affinity::After), Movement::PreviousWord)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        let collapsed = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        assert_eq!(
            collapsed.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            collapsed
                .move_cursor(Cursor::new(3, Affinity::After), Movement::PreviousWord)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
    }

    #[test]
    fn native_mixed_occurrence_next_cluster_refuses_an_ambiguous_source_projection() {
        let layout = native_four_lines(true);
        assert_eq!(layout.lines().len(), 4);
        let error = layout
            .move_cursor(Cursor::new(0, Affinity::After), Movement::NextCluster)
            .expect_err("the target projects several real control/line occurrences to source1");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AmbiguousSourcePosition {
                    index: 1,
                    affinity: Affinity::Before
                }
            })
        );
    }
    #[test]
    fn native_mixed_occurrence_next_line_refuses_an_ambiguous_source_projection() {
        let layout = native_four_lines(true);
        assert_eq!(layout.lines().len(), 4);
        let error = layout
            .move_cursor(Cursor::new(0, Affinity::After), Movement::NextLine)
            .expect_err("the target projects several real control/line occurrences to source1");
        assert_eq!(
            error.detail(),
            Some(&crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::AmbiguousSourcePosition {
                    index: 1,
                    affinity: Affinity::After
                }
            })
        );
    }
    #[test]
    fn native_mixed_occurrence_exact_handles_traverse_the_real_cluster_and_line_targets() {
        let layout = native_four_lines(true);
        let start = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
            .unwrap();
        let cluster = layout.move_position(&start, Movement::NextCluster).unwrap();
        let line = layout.move_position(&start, Movement::NextLine).unwrap();
        assert_eq!(cluster.source_cursor(), Cursor::new(1, Affinity::Before));
        assert_eq!(line.source_cursor(), Cursor::new(1, Affinity::After));
        assert_eq!(
            layout.position_geometry(&cluster).unwrap().rect(),
            Rect::new(20.0, 0.0, 1.0, 20.0)
        );
        assert_eq!(
            layout.position_geometry(&line).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_ne!(cluster, line);
    }

    #[test]
    fn native_collapsed_gap_affinity_probe_keeps_the_actual_b_start_geometry() {
        let layout = native_whitespace_line("A  B", crate::WhiteSpace::Collapse);
        let line = &layout.inner.lines[0].finalized;
        let b = line
            .data
            .clusters
            .iter()
            .find(|cluster| cluster.source == Range::new(3, 4))
            .unwrap();
        assert_eq!(
            (b.source, b.presentation, b.x, b.advance),
            (Range::new(3, 4), Range::new(2, 3), 40.0, 20.0)
        );
        let anchor = line
            .data
            .anchors
            .iter()
            .find(|anchor| {
                matches!(
                    line.selected.presentation.units[anchor.unit].kind,
                    crate::protocol::prepared::PresentationUnitKind::Deleted
                )
            })
            .unwrap();
        let deleted = &line.selected.presentation.units[anchor.unit];
        assert_eq!(
            (
                deleted.source,
                deleted.processed,
                deleted.presentation,
                anchor.slot
            ),
            (
                Range::new(2, 3),
                Range::new(2, 2),
                Range::new(2, 2),
                Point::new(40.0, 16.0)
            )
        );
        // Only original-source affinity differs. The actual source/presentation
        // rows and finite physical anchor are the same in these two queries.
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
    }

    #[test]
    fn native_exact_word_target_keeps_the_b_row_across_a_forced_control() {
        let layout = native_break_lines("AB", 1, 1, true);
        assert_eq!(layout.lines().len(), 2);
        let b = layout
            .clusters()
            .into_iter()
            .find(|cluster| cluster.range() == Range::new(1, 2))
            .unwrap();
        assert_eq!(b.bounds(), Rect::new(0.0, 20.0, 20.0, 20.0));
        assert_eq!(
            layout.inner.lines[0].finalized.selected.request.context.end,
            crate::LineEnd::Forced
        );
        let start = layout
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
            .unwrap();
        let moved = layout.move_position(&start, Movement::NextWord).unwrap();
        assert_eq!(
            layout.position_geometry(&moved).unwrap().rect(),
            Rect::new(0.0, 20.0, 1.0, 20.0)
        );
        assert_eq!(moved.source_cursor(), Cursor::new(1, Affinity::After));
    }

    #[test]
    fn native_preserved_long_space_interior_word_uses_the_original_segment() {
        let layout = native_whitespace_line("A    B", crate::WhiteSpace::Preserve);
        assert_eq!(
            layout.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x20, 0x20, 0x42]
        );
        let cursor = Cursor::new(3, Affinity::After);
        let next = layout.move_cursor(cursor, Movement::NextWord).unwrap();
        let previous = layout.move_cursor(cursor, Movement::PreviousWord).unwrap();
        assert_eq!(next, Cursor::new(5, Affinity::Before));
        assert_eq!(previous, Cursor::new(1, Affinity::Before));
    }

    #[test]
    fn native_collapsed_gap_rtl_affinities_keep_the_actual_deleted_slot() {
        let layout = native_whitespace_line_with_direction(
            "\u{05d0}  \u{05d1}",
            crate::WhiteSpace::Collapse,
            ParagraphBaseDirection::RightToLeft,
        );
        let line = &layout.inner.lines[0].finalized;
        let bet = line
            .data
            .clusters
            .iter()
            .find(|cluster| cluster.source == Range::new(3, 4))
            .unwrap();
        assert_eq!((bet.level, bet.x, bet.advance), (1, 0.0, 20.0));
        let anchor = line
            .data
            .anchors
            .iter()
            .find(|anchor| {
                matches!(
                    line.selected.presentation.units[anchor.unit].kind,
                    crate::protocol::prepared::PresentationUnitKind::Deleted
                )
            })
            .unwrap();
        assert_eq!(anchor.slot, Point::new(20.0, 16.0));
        for affinity in [Affinity::Before, Affinity::After] {
            assert_eq!(
                layout
                    .cursor(Cursor::new(3, affinity))
                    .unwrap()
                    .rect()
                    .origin
                    .x,
                anchor.slot.x
            );
        }
    }

    #[test]
    fn native_collapsed_gap_interior_word_queries_use_real_neighbor_rows() {
        let layout = native_whitespace_line("A    B", crate::WhiteSpace::Collapse);
        assert_eq!(
            layout.source().text().code_units(),
            [0x41, 0x20, 0x20, 0x20, 0x20, 0x42]
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            40.0
        );
        let next = layout
            .move_cursor(Cursor::new(3, Affinity::After), Movement::NextWord)
            .unwrap();
        let previous = layout
            .move_cursor(Cursor::new(3, Affinity::After), Movement::PreviousWord)
            .unwrap();
        assert_eq!(
            [next, previous],
            [
                Cursor::new(5, Affinity::Before),
                Cursor::new(1, Affinity::Before)
            ]
        );
    }

    fn native_whitespace_soft_lines(
        text: &str,
        mode: crate::WhiteSpace,
        split_at: usize,
    ) -> Layout {
        let prepared = native_whitespace_prepared(text, mode, ParagraphBaseDirection::LeftToRight);
        let split = prepared
            .opportunity_views()
            .find(|view| {
                view.kind() == PreparedBreakKind::Allowed
                    && view.boundary().view().source_before().get() == split_at
            })
            .expect("the actual lexical boundary after the whitespace is Allowed")
            .handle();
        let end = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let mut start = prepared.boundary_views().next().unwrap().handle();
        let mut lines = Vec::new();
        for (index, termination) in [split, end].into_iter().enumerate() {
            let end = termination.view().boundary();
            let participants = prepared
                .participants()
                .skip(start.view().participant_offset())
                .take(end.view().participant_offset() - start.view().participant_offset())
                .collect();
            let selected = prepared
                .select(LineSelectionRequest {
                    boundary_context: Vec::new(),
                    incoming: None,
                    start: start.clone(),
                    end: end.clone(),
                    participants,
                    termination: crate::PreparedTermination::Ordinary(termination),
                    alternative: None,
                    context: LineContext {
                        alignment_insets: crate::LineAlignmentInsets::ZERO,
                        formatted_line: crate::FormattedLineContext::default(),
                        base_direction: ParagraphBaseDirection::LeftToRight,
                        line_box_direction: crate::LineBoxDirection::LeftToRight,
                        orientation: LineOrientation::Horizontal,
                        start: if index == 0 {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if index == 0 {
                            LineEnd::Soft
                        } else {
                            LineEnd::Paragraph
                        },
                        available_inline: None,
                        indent: 0.0,
                        alignment: Alignment::Start,
                        tabs: TabContext::NoTabs,

                        spacing: SpacingPolicy::None,
                    },
                    atomics: vec![],
                    controls: vec![],
                    line_box_struts: Vec::new(),
                    line_strut: None,
                })
                .unwrap();
            lines.push(LinePlacement {
                line: selected.finalize(SpacingAllocations::None).unwrap(),
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: 16.0 + index as f64 * 40.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            });
            start = end;
        }
        Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: lines
                .into_iter()
                .map(crate::PlacementEntry::Formatted)
                .collect(),
        })
        .unwrap()
    }

    #[test]
    fn native_soft_line_deleted_end_keeps_original_affinity_side_eligibility() {
        for (text, mode, split, space_advance, upstream_x) in [
            ("A B", crate::WhiteSpace::Preserve, 2, 20.0, 40.0),
            ("A  B", crate::WhiteSpace::Collapse, 3, 0.0, 20.0),
        ] {
            let layout = native_whitespace_soft_lines(text, mode, split);
            assert_eq!(
                layout
                    .lines()
                    .iter()
                    .map(|line| line.range())
                    .collect::<Vec<_>>(),
                [Range::new(0, split), Range::new(split, text.len())]
            );
            // The first Soft line applies the existing Collapse edge policy
            // before placing its actual Deleted anchor; Preserve keeps space.
            let first = &layout.inner.lines[0].finalized;
            let space = first
                .data
                .clusters
                .iter()
                .find(|cluster| cluster.source == Range::new(1, 2))
                .unwrap();
            assert_eq!((space.x, space.advance), (20.0, space_advance));
            if mode == crate::WhiteSpace::Collapse {
                let deleted = first
                    .data
                    .anchors
                    .iter()
                    .find(|anchor| {
                        matches!(
                            first.selected.presentation.units[anchor.unit].kind,
                            crate::protocol::prepared::PresentationUnitKind::Deleted
                        )
                    })
                    .unwrap();
                assert_eq!(deleted.slot, Point::new(20.0, 16.0));
            }
            assert_eq!(
                layout
                    .cursor(Cursor::new(split, Affinity::Before))
                    .unwrap()
                    .rect(),
                Rect::new(upstream_x, 0.0, 1.0, 20.0)
            );
            assert_eq!(
                layout
                    .cursor(Cursor::new(split, Affinity::After))
                    .unwrap()
                    .rect(),
                Rect::new(0.0, 40.0, 1.0, 20.0)
            );
        }
    }

    #[test]
    fn native_deleted_original_endpoints_keep_canonical_raw_affinities() {
        let layout = native_whitespace_line(" A ", crate::WhiteSpace::Discard);
        assert!(
            layout
                .inner
                .prepared
                .tables
                .units
                .iter()
                .any(|unit| unit.deleted && unit.source == Range::new(0, 1))
        );
        assert!(
            layout
                .inner
                .prepared
                .tables
                .units
                .iter()
                .any(|unit| unit.deleted && unit.source == Range::new(2, 3))
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(0, Affinity::Before), Movement::DocumentStart)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        assert_eq!(
            layout
                .move_cursor(Cursor::new(3, Affinity::After), Movement::DocumentEnd)
                .unwrap(),
            Cursor::new(3, Affinity::Before)
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(0, Affinity::Before))
                .unwrap()
                .rect()
                .origin
                .x,
            0.0
        );
        assert_eq!(
            layout
                .cursor(Cursor::new(3, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            20.0
        );
    }

    #[test]
    fn native_deleted_start_next_word_keeps_the_original_whitespace_segment() {
        for mode in [crate::WhiteSpace::Preserve, crate::WhiteSpace::Collapse] {
            let layout = native_whitespace_line("A  B", mode);
            assert_eq!(
                layout.source().text().code_units(),
                [0x41, 0x20, 0x20, 0x42]
            );
            if mode == crate::WhiteSpace::Collapse {
                assert!(
                    layout
                        .inner
                        .prepared
                        .tables
                        .units
                        .iter()
                        .any(|unit| unit.deleted && unit.source == Range::new(2, 3))
                );
                let b_start = layout
                    .clusters()
                    .into_iter()
                    .find(|cluster| cluster.range() == Range::new(3, 4))
                    .unwrap()
                    .range()
                    .start;
                assert!(2 < b_start);
            }
            for affinity in [Affinity::After, Affinity::Before] {
                assert_eq!(
                    layout
                        .move_cursor(Cursor::new(2, affinity), Movement::NextWord)
                        .unwrap(),
                    Cursor::new(3, Affinity::Before)
                );
            }
        }
    }

    #[test]
    fn native_deleted_leading_previous_word_cannot_move_forward() {
        for mode in [crate::WhiteSpace::Preserve, crate::WhiteSpace::Discard] {
            let layout = native_whitespace_line("  B", mode);
            assert_eq!(layout.source().text().code_units(), [0x20, 0x20, 0x42]);
            if mode == crate::WhiteSpace::Discard {
                assert!(
                    layout
                        .inner
                        .prepared
                        .tables
                        .units
                        .iter()
                        .any(|unit| unit.deleted && unit.source == Range::new(0, 2))
                );
                assert_eq!(layout.clusters()[0].range(), Range::new(2, 3));
            }
            assert_eq!(
                layout
                    .move_cursor(Cursor::new(0, Affinity::After), Movement::PreviousWord)
                    .unwrap(),
                Cursor::new(0, Affinity::After)
            );
        }
    }
}
