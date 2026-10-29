// Copyright 2025 the Parley Authors
// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cursor geometry and movement adapted from Parley 0.9 `editing/cursor.rs`
//! (cd7237d6c75c495f6aa0941e3ecf9182dc62d33b6f0d4599a6a117765265e156)
//! and `layout/cluster.rs`
//! (43795e0d29905679d983080f99ea070d665043601c192cf822550228d10c6899).
//! Queries traverse the actual committed L2 order, including zero-advance text.

use super::super::{
    PlacedLine,
    analysis::Boundary,
    layout::data::{FinalizedCluster, FinalizedVisualItem, PlacedLayout, Whitespace},
};
use crate::protocol::LineEnd;
use crate::{Affinity, Cursor, CursorGeometry, Hit, Movement, Point, Rect, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClusterPath {
    line: usize,
    cluster: usize,
    visual: usize,
}

impl ClusterPath {
    fn get(self, layout: &PlacedLayout) -> &FinalizedCluster {
        &layout.lines[self.line].finalized.data.clusters[self.cluster]
    }

    fn rtl(self, layout: &PlacedLayout) -> bool {
        self.get(layout).level & 1 != 0
    }

    fn end_of_line(self, layout: &PlacedLayout) -> bool {
        layout.lines[self.line].finalized.data.visual_order[self.visual + 1..]
            .iter()
            .all(|item| matches!(item, FinalizedVisualItem::Anchor(_)))
    }

    fn soft(self, layout: &PlacedLayout) -> bool {
        self.end_of_line(layout)
            && layout.lines[self.line]
                .finalized
                .selected
                .request
                .context
                .end
                == LineEnd::Soft
    }

    fn adjacent(self, layout: &PlacedLayout, forward: bool) -> Option<Self> {
        if forward {
            for line in self.line..layout.lines.len() {
                let start = if line == self.line {
                    self.visual + 1
                } else {
                    0
                };
                for (visual, item) in layout.lines[line]
                    .finalized
                    .data
                    .visual_order
                    .iter()
                    .enumerate()
                    .skip(start)
                {
                    if let FinalizedVisualItem::Cluster(cluster) = *item {
                        return Some(Self {
                            line,
                            cluster,
                            visual,
                        });
                    }
                }
            }
        } else {
            for line in (0..=self.line).rev() {
                let order = &layout.lines[line].finalized.data.visual_order;
                let end = if line == self.line {
                    self.visual
                } else {
                    order.len()
                };
                for (visual, item) in order[..end].iter().enumerate().rev() {
                    if let FinalizedVisualItem::Cluster(cluster) = *item {
                        return Some(Self {
                            line,
                            cluster,
                            visual,
                        });
                    }
                }
            }
        }
        None
    }
}

fn at_source(layout: &PlacedLayout, index: usize) -> Option<ClusterPath> {
    for (line, row) in layout.lines.iter().enumerate() {
        if let Some(cluster) = row
            .finalized
            .data
            .clusters
            .iter()
            .position(|c| c.source.contains(index))
        {
            let visual = row
                .finalized
                .data
                .visual_order
                .iter()
                .position(|item| *item == FinalizedVisualItem::Cluster(cluster))?;
            return Some(ClusterPath {
                line,
                cluster,
                visual,
            });
        }
    }
    None
}

fn normalize(layout: &PlacedLayout, index: usize, affinity: Affinity) -> Cursor {
    // The original empty source has one raw endpoint. Retaining a genuine
    // empty-origin slot does not add authored cursor affinity states. Native
    // occurrence handles remain separate and never pass through this projection.
    if layout.coverage.source.is_empty()
        && layout
            .lines
            .iter()
            .all(|line| line.finalized.data.clusters.is_empty())
    {
        return Cursor::new(layout.coverage.source.start, Affinity::Before);
    }
    let extent = layout
        .prepared
        .analyzed
        .paragraph
        .extent(&layout.prepared.analyzed.processed);
    let original_end = extent.end;
    let original_start = extent.start;
    if original_end != original_start {
        if index == original_start {
            return Cursor::new(original_start, Affinity::After);
        }
        if index == original_end {
            return Cursor::new(original_end, Affinity::Before);
        }
    }
    if let Some(cluster) = at_source(layout, index) {
        let index = cluster.get(layout).source.start;
        Cursor::new(
            index,
            if index == original_start {
                Affinity::After
            } else {
                affinity
            },
        )
    } else if anchor_geometry(layout, index, affinity).is_some() {
        Cursor::new(index, affinity)
    } else {
        Cursor::new(layout.coverage.source.end, Affinity::Before)
    }
}

pub(crate) fn hit(layout: &std::sync::Arc<PlacedLayout>, point: Point) -> Hit {
    if !point.x.is_finite() || !point.y.is_finite() {
        return Hit::None;
    }
    for line in &layout.lines {
        for item in &line.atomics {
            if item.border.contains(point) {
                return Hit::InlineBox(item.occurrence.view().original_box().id());
            }
        }
    }
    if !layout.bounds.contains(point) {
        return Hit::None;
    }
    super::position::hit(layout, point).map_or(Hit::None, |index| {
        Hit::Text(crate::layout::PlacedPosition {
            placement: std::sync::Arc::clone(layout),
            index,
        })
    })
}

pub(crate) fn cursor(layout: &PlacedLayout, value: Cursor) -> Result<CursorGeometry> {
    Ok(CursorGeometry::new(
        geometry(layout, admit(layout, value)?).1,
    ))
}

pub(super) fn admit(layout: &PlacedLayout, value: Cursor) -> Result<Cursor> {
    crate::range::validate_index(
        layout.prepared.analyzed.processed.source().text(),
        value.index(),
        "cursor",
    )?;
    super::position::validate_source(layout, value)?;
    Ok(normalize(
        layout,
        layout.interaction.caret(value.index(), value.affinity()),
        value.affinity(),
    ))
}

fn logical_pair(layout: &PlacedLayout, value: Cursor) -> [Option<ClusterPath>; 2] {
    let mut pair = [
        value
            .index()
            .checked_sub(1)
            .and_then(|index| at_source(layout, index)),
        at_source(layout, value.index()),
    ];
    // Original source intervals need not be adjacent after an actual deletion.
    // The rows remain in original logical order; retain their identity while
    // finding the closest predecessor and successor of the original position.
    if pair.iter().any(Option::is_none) {
        for (line, placed) in layout.lines.iter().enumerate() {
            for (cluster, row) in placed.finalized.data.clusters.iter().enumerate() {
                if row.source.is_empty() {
                    continue;
                }
                let side = if row.source.end <= value.index() {
                    0
                } else if value.index() <= row.source.start {
                    1
                } else {
                    continue;
                };
                let closer = pair[side].is_none_or(|old| {
                    let old = old.get(layout);
                    if side == 0 {
                        old.source.end < row.source.end
                    } else {
                        row.source.start < old.source.start
                    }
                });
                if closer {
                    let visual = placed
                        .finalized
                        .data
                        .visual_order
                        .iter()
                        .position(|item| *item == FinalizedVisualItem::Cluster(cluster))
                        .expect("finalized cluster is in actual visual order");
                    pair[side] = Some(ClusterPath {
                        line,
                        cluster,
                        visual,
                    });
                }
            }
        }
    }
    pair
}

fn visual_pair(layout: &PlacedLayout, value: Cursor) -> [Option<ClusterPath>; 2] {
    let [upstream, downstream] = logical_pair(layout, value);
    if value.affinity() == Affinity::Before {
        if let Some(cluster) = upstream {
            if cluster.rtl(layout) {
                [cluster.adjacent(layout, false), Some(cluster)]
            } else {
                [Some(cluster), cluster.adjacent(layout, true)]
            }
        } else if let Some(cluster) = downstream {
            if cluster.rtl(layout) {
                [None, Some(cluster)]
            } else {
                [Some(cluster), None]
            }
        } else {
            [None, None]
        }
    } else if let Some(cluster) = downstream {
        if cluster.rtl(layout) {
            [Some(cluster), cluster.adjacent(layout, true)]
        } else {
            [cluster.adjacent(layout, false), Some(cluster)]
        }
    } else if let Some(cluster) = upstream {
        if cluster.rtl(layout) {
            [None, Some(cluster)]
        } else {
            [Some(cluster), None]
        }
    } else {
        [None, None]
    }
}

pub(super) fn geometry(layout: &PlacedLayout, value: Cursor) -> (usize, Rect) {
    // Deleted anchors are genuine source-associated occurrences. This does not
    // change the historical raw-source visual-neighbor policy around atomics:
    // a physical atomic gap and an executed source deletion have different owners.
    if let Some(index) = super::position::deleted_position(layout, value) {
        let line = layout.interaction.positions[index].line;
        return (
            line,
            super::position::geometry(layout, index)
                .expect("owned deleted row")
                .rect(),
        );
    }
    let no_text_neighbor = at_source(layout, value.index()).is_none()
        && value
            .index()
            .checked_sub(1)
            .and_then(|index| at_source(layout, index))
            .is_none();
    if no_text_neighbor
        && let Some((line, x)) = anchor_geometry(layout, value.index(), value.affinity())
    {
        return (line, line_rect(&layout.lines[line], x));
    }
    let chosen = match visual_pair(layout, value) {
        [Some(left), Some(right)] => {
            if left.end_of_line(layout) {
                if left.soft(layout)
                    && ((left.rtl(layout) && value.affinity() == Affinity::After)
                        || (!left.rtl(layout) && value.affinity() == Affinity::Before))
                {
                    Some((left, true))
                } else {
                    Some((right, false))
                }
            } else {
                Some((left, true))
            }
        }
        [Some(left), None] if left.get(layout).whitespace == Whitespace::Newline => None,
        [Some(left), _] => Some((left, true)),
        [_, Some(right)] => Some((right, false)),
        _ => None,
    };
    if let Some((path, at_end)) = chosen {
        let cluster = path.get(layout);
        (
            path.line,
            line_rect(
                &layout.lines[path.line],
                cluster.x + if at_end { cluster.advance } else { 0.0 },
            ),
        )
    } else {
        let index = layout.lines.len() - 1;
        let line = &layout.lines[index];
        (index, line_rect(line, line.finalized.data.metrics.offset))
    }
}

/// Actual selected anchor slots and replacement clusters supply geometry for
/// zero-glyph/deleted source. Numeric source equality does not create a slot.
fn anchor_geometry(
    layout: &PlacedLayout,
    index: usize,
    affinity: Affinity,
) -> Option<(usize, f64)> {
    let mut chosen: Option<((usize, usize), f64)> = None;
    let mut consider = |key: (usize, usize), x: f64| {
        if chosen.is_none_or(|(old, _)| match affinity {
            Affinity::Before => key < old,
            Affinity::After => key > old,
        }) {
            chosen = Some((key, x));
        }
    };
    for (line_index, line) in layout.lines.iter().enumerate() {
        for anchor in &line.finalized.data.anchors {
            let unit = &line.finalized.selected.presentation.units[anchor.unit];
            if unit.source.start <= index && index <= unit.source.end {
                consider((line_index, anchor.unit), anchor.slot.x);
            }
        }
        for cluster in &line.finalized.data.clusters {
            let replacement_incident = cluster.source.is_empty()
                && line.finalized.shape.mapping.pieces.iter().any(|piece| {
                    matches!(
                        piece.kind,
                        crate::protocol::prepared::SelectedMappingKind::Replacement { .. }
                    ) && piece.source.start <= index
                        && index <= piece.source.end
                        && piece.presentation.start < cluster.presentation.end
                        && cluster.presentation.start < piece.presentation.end
                });
            if (cluster.source.is_empty() && cluster.source.start == index) || replacement_incident
            {
                let after = affinity == Affinity::After;
                let x = if after != (cluster.level & 1 != 0) {
                    cluster.x + cluster.advance
                } else {
                    cluster.x
                };
                let unit = line
                    .finalized
                    .selected
                    .presentation
                    .units
                    .iter()
                    .position(|unit| {
                        unit.presentation.start <= cluster.presentation.start
                            && cluster.presentation.start < unit.presentation.end
                    })
                    .expect("an actual selected cluster retains its presentation unit");
                consider((line_index, unit), x);
            }
        }
    }
    chosen.map(|((line, _), x)| (line, x))
}

fn line_rect(line: &PlacedLine, x: f64) -> Rect {
    let metrics = line.finalized.data.metrics;
    crate::layout::line_rect(
        line,
        Rect::new(
            x,
            metrics.block_min,
            1.0,
            metrics.block_max - metrics.block_min,
        ),
    )
}

// Vertical inline axes use actual physical metric rectangles without assuming
// a left/right paragraph stack. Source ordinal resolves exact ties.
pub(super) fn line_at_point(layout: &PlacedLayout, point: Point) -> Option<usize> {
    if layout.prepared.options.orientation() != crate::LineOrientation::Horizontal {
        return layout
            .lines
            .iter()
            .enumerate()
            .min_by(|(a, left), (b, right)| {
                let distance = |line: &PlacedLine| {
                    let metrics = line.finalized.data.metrics;
                    let rect = crate::layout::line_rect(
                        line,
                        Rect::new(
                            metrics.inline_min,
                            metrics.block_min,
                            metrics.inline_max - metrics.inline_min,
                            metrics.block_max - metrics.block_min,
                        ),
                    );
                    let dx = (rect.origin.x - point.x)
                        .max(0.0)
                        .max(point.x - (rect.origin.x + rect.size.width));
                    let dy = (rect.origin.y - point.y)
                        .max(0.0)
                        .max(point.y - (rect.origin.y + rect.size.height));
                    dx.hypot(dy)
                };
                distance(left)
                    .total_cmp(&distance(right))
                    .then_with(|| a.cmp(b))
            })
            .map(|(index, _)| index);
    }
    layout
        .lines
        .iter()
        .position(|line| {
            let metrics = line.finalized.data.metrics;
            crate::layout::line_rect(
                line,
                Rect::new(
                    metrics.inline_min,
                    metrics.block_min,
                    metrics.inline_max - metrics.inline_min,
                    metrics.block_max - metrics.block_min,
                ),
            )
            .contains(point)
        })
        .or_else(|| {
            layout.lines.iter().position(|line| {
                point.y
                    < crate::layout::line_point(
                        line,
                        Point::new(0.0, line.finalized.data.metrics.block_max),
                    )
                    .y
            })
        })
}

fn cursor_at_inline(layout: &PlacedLayout, line_index: usize, x: f64) -> Option<Cursor> {
    let line = &layout.lines[line_index];
    let data = &line.finalized.data;
    let mut visual = 0;
    while visual < data.visual_order.len() {
        let FinalizedVisualItem::Cluster(first) = data.visual_order[visual] else {
            visual += 1;
            continue;
        };
        let first_cluster = &data.clusters[first];
        let mut end = visual + 1;
        while let Some(FinalizedVisualItem::Cluster(next)) = data.visual_order.get(end) {
            let next = &data.clusters[*next];
            if next.content != first_cluster.content || next.level != first_cluster.level {
                break;
            }
            end += 1;
        }
        let last = match data.visual_order[end - 1] {
            FinalizedVisualItem::Cluster(index) => &data.clusters[index],
            FinalizedVisualItem::InlineBox(_) | FinalizedVisualItem::Anchor(_) => {
                unreachable!()
            }
        };
        if x > last.x + last.advance && end != data.visual_order.len() {
            visual = end;
            continue;
        }
        while visual < end {
            let FinalizedVisualItem::Cluster(cluster) = data.visual_order[visual] else {
                unreachable!()
            };
            let path = ClusterPath {
                line: line_index,
                cluster,
                visual,
            };
            let cluster = path.get(layout);
            if x <= cluster.x + cluster.advance || visual + 1 == data.visual_order.len() {
                return Some(point_edge(
                    layout,
                    path,
                    x > cluster.x + cluster.advance * 0.5,
                ));
            }
            visual += 1;
        }
    }
    None
}

pub(super) fn from_point(layout: &PlacedLayout, point: Point) -> Cursor {
    if let Some(line_index) = line_at_point(layout, point) {
        let x = crate::layout::local_point(&layout.lines[line_index], point).x;
        if let Some(cursor) = cursor_at_inline(layout, line_index, x) {
            return cursor;
        }
    }
    // The provider's non-exact query above the layout clamps to its first cluster.
    if layout.prepared.options.orientation() == crate::LineOrientation::Horizontal
        && point.y <= layout.bounds.origin.y
        && let Some(first) = layout.lines.first()
        && let Some((visual, cluster)) = first
            .finalized
            .data
            .visual_order
            .iter()
            .enumerate()
            .find_map(|(visual, item)| match item {
                FinalizedVisualItem::Cluster(cluster) => Some((visual, *cluster)),
                FinalizedVisualItem::InlineBox(_) | FinalizedVisualItem::Anchor(_) => None,
            })
    {
        return point_edge(
            layout,
            ClusterPath {
                line: 0,
                cluster,
                visual,
            },
            false,
        );
    }
    Cursor::new(layout.coverage.source.end, Affinity::After)
}

fn point_edge(layout: &PlacedLayout, path: ClusterPath, right: bool) -> Cursor {
    let cluster = path.get(layout);
    let value = if path.rtl(layout) {
        if right {
            Cursor::new(cluster.source.start, Affinity::After)
        } else {
            Cursor::new(cluster.source.end, Affinity::Before)
        }
    } else if !right
        || (path.end_of_line(layout)
            && layout.lines[path.line]
                .finalized
                .selected
                .request
                .context
                .end
                == LineEnd::Forced)
    {
        Cursor::new(cluster.source.start, Affinity::After)
    } else {
        Cursor::new(cluster.source.end, Affinity::Before)
    };
    Cursor::new(
        layout.interaction.caret(value.index(), value.affinity()),
        value.affinity(),
    )
}

pub(crate) fn move_cursor(
    layout: &PlacedLayout,
    value: Cursor,
    movement: Movement,
) -> Result<Cursor> {
    let value = admit(layout, value)?;
    let result = match movement {
        Movement::DocumentStart => admit(
            layout,
            Cursor::new(
                layout
                    .prepared
                    .analyzed
                    .paragraph
                    .extent(&layout.prepared.analyzed.processed)
                    .start,
                Affinity::After,
            ),
        )?,
        Movement::DocumentEnd => admit(
            layout,
            Cursor::new(
                layout
                    .prepared
                    .analyzed
                    .paragraph
                    .extent(&layout.prepared.analyzed.processed)
                    .end,
                Affinity::Before,
            ),
        )?,
        Movement::LineStart | Movement::LineEnd => {
            source_line(layout, value.index()).map_or(value, |index| {
                let line = &layout.lines[index];
                if movement == Movement::LineStart {
                    normalize(
                        layout,
                        line.finalized.selected.presentation.source.start,
                        Affinity::After,
                    )
                } else {
                    normalize(layout, visible_end(layout, line), Affinity::Before)
                }
            })
        }
        Movement::PreviousLine | Movement::NextLine => {
            let target = source_line(layout, value.index()).and_then(|index| {
                if movement == Movement::PreviousLine {
                    index.checked_sub(1)
                } else {
                    (index + 1 < layout.lines.len()).then_some(index + 1)
                }
            });
            target.map_or(value, |index| {
                let line = &layout.lines[index];
                let (current, rect) = geometry(layout, value);
                let inline = if line.orientation != crate::LineOrientation::Horizontal {
                    crate::layout::local_point(&layout.lines[current], rect.origin).x
                } else {
                    rect.origin.x
                };
                // Exact and raw movement select the same genuine target rows.
                // Final source validation owns any lossy projection ambiguity.
                super::position::nearest_on_line(layout, index, inline)
                    .map_or(value, |target| layout.interaction.positions[target].source)
            })
        }
        Movement::PreviousWord | Movement::NextWord => {
            let forward = movement == Movement::NextWord;
            let complete = layout
                .prepared
                .analyzed
                .paragraph
                .extent(&layout.prepared.analyzed.processed);
            if layout.coverage.source == complete {
                logical_word(layout, value, forward)
            } else {
                // Source-global movement resolves its target on the retained
                // complete preparation before asking this placement for geometry.
                let clusters = &layout.prepared.shaped.clusters;
                let target = word_target(
                    value,
                    forward,
                    complete,
                    |index| {
                        // Base preparation emits nonoverlapping original atoms in
                        // logical order; selected replacements are not this owner.
                        let at = clusters.partition_point(|cluster| cluster.source.end <= index);
                        let downstream = clusters.get(at).map(|_| at);
                        let upstream = if clusters
                            .get(at)
                            .is_some_and(|cluster| cluster.source.start < index)
                        {
                            Some(at)
                        } else {
                            at.checked_sub(1)
                        };
                        [upstream, downstream]
                    },
                    |index| {
                        let cluster = &clusters[index];
                        (cluster.source, cluster.boundary, cluster.level)
                    },
                    |index, forward| {
                        if forward {
                            index.checked_add(1).filter(|&next| next < clusters.len())
                        } else {
                            index.checked_sub(1)
                        }
                    },
                );
                admit(layout, target.cursor)?
            }
        }
        Movement::PreviousCluster | Movement::NextCluster => {
            let right = movement == Movement::NextCluster;
            let mut current = visual_move(layout, value, right);
            // Each step consumes a genuine visual cluster or the two sides of a
            // soft boundary. Skip only illegal original grapheme/ligature stops.
            while !layout.interaction.is_stop(current.index()) {
                let next = visual_move(layout, current, right);
                if next == current {
                    break;
                }
                current = next;
            }
            current
        }
    };
    // The target already lends a real original-source row. Only enforce legal
    // original stops here; a second deletion-map round trip can move its edge.
    let result = Cursor::new(
        layout.interaction.caret(result.index(), result.affinity()),
        result.affinity(),
    );
    super::position::validate_source(layout, result)?;
    Ok(result)
}

fn source_line(layout: &PlacedLayout, index: usize) -> Option<usize> {
    layout
        .lines
        .iter()
        .position(|line| line.finalized.selected.presentation.source.contains(index))
        .or_else(|| {
            layout.lines.iter().rposition(|line| {
                let range = line.finalized.selected.presentation.source;
                range.end == index && !range.is_empty()
            })
        })
        .or_else(|| (index >= layout.coverage.source.end).then_some(layout.lines.len() - 1))
}

fn visible_end(layout: &PlacedLayout, line: &PlacedLine) -> usize {
    let range = line.finalized.selected.presentation.source;
    let units = layout
        .prepared
        .analyzed
        .processed
        .source()
        .text()
        .code_units();
    let mut end = range.end;
    // Preserve the existing standalone wrapper's exact CR/LF policy.
    while end > range.start
        && layout.prepared.analyzed.paragraph.contains(
            layout.prepared.analyzed.processed.source(),
            crate::Range::new(end - 1, end),
        )
        && matches!(units[end - 1], 0x0a | 0x0d)
    {
        end -= 1;
    }
    end
}

fn directional_affinity(rtl: bool, right: bool) -> Affinity {
    if rtl == right {
        Affinity::After
    } else {
        Affinity::Before
    }
}

pub(super) fn logical_word(layout: &PlacedLayout, value: Cursor, forward: bool) -> Cursor {
    let target = logical_word_target(layout, value, forward);
    normalize(layout, target.cursor.index(), target.cursor.affinity())
}

pub(super) struct WordTarget<K> {
    pub(super) cursor: Cursor,
    /// Actual logical row and trailing-edge flag. Exact occurrence movement
    /// consumes this directly; its source projection is not an inverse key.
    pub(super) row: Option<(K, bool)>,
}

pub(super) fn logical_word_target(
    layout: &PlacedLayout,
    value: Cursor,
    forward: bool,
) -> WordTarget<(usize, usize)> {
    word_target(
        value,
        forward,
        layout.coverage.source,
        |index| {
            logical_pair(layout, Cursor::new(index, value.affinity()))
                .map(|path| path.map(|path| (path.line, path.cluster)))
        },
        |(line, cluster)| {
            let cluster = &layout.lines[line].finalized.data.clusters[cluster];
            (cluster.source, cluster.boundary, cluster.level)
        },
        |path, forward| adjacent_word_row(layout, path, forward),
    )
}

pub(super) fn logical_word_from_cluster(
    layout: &PlacedLayout,
    value: Cursor,
    forward: bool,
    row: (usize, usize),
    trailing: bool,
) -> WordTarget<(usize, usize)> {
    word_target(
        value,
        forward,
        layout.coverage.source,
        |_| {
            if trailing {
                [Some(row), adjacent_word_row(layout, row, true)]
            } else {
                [adjacent_word_row(layout, row, false), Some(row)]
            }
        },
        |(line, cluster)| {
            let cluster = &layout.lines[line].finalized.data.clusters[cluster];
            (cluster.source, cluster.boundary, cluster.level)
        },
        |row, forward| adjacent_word_row(layout, row, forward),
    )
}

pub(super) fn logical_word_from_anchor(
    layout: &PlacedLayout,
    value: Cursor,
    forward: bool,
    line: usize,
    anchor: usize,
) -> WordTarget<(usize, usize)> {
    let finalized = &layout.lines[line].finalized;
    let unit = &finalized.selected.presentation.units[finalized.data.anchors[anchor].unit];
    let mut neighbors = [None, None];
    for (row_line, placed) in layout.lines.iter().enumerate() {
        for (cluster, row) in placed.finalized.data.clusters.iter().enumerate() {
            if row_line < line
                || (row_line == line && row.presentation.end <= unit.presentation.start)
            {
                neighbors[0] = Some((row_line, cluster));
            }
            if neighbors[1].is_none()
                && (row_line > line
                    || (row_line == line && unit.presentation.end <= row.presentation.start))
            {
                neighbors[1] = Some((row_line, cluster));
            }
        }
    }
    word_target(
        value,
        forward,
        layout.coverage.source,
        |_| neighbors,
        |(line, cluster)| {
            let cluster = &layout.lines[line].finalized.data.clusters[cluster];
            (cluster.source, cluster.boundary, cluster.level)
        },
        |row, forward| adjacent_word_row(layout, row, forward),
    )
}

fn adjacent_word_row(
    layout: &PlacedLayout,
    (line, cluster): (usize, usize),
    forward: bool,
) -> Option<(usize, usize)> {
    if forward {
        let next = cluster + 1;
        if next < layout.lines[line].finalized.data.clusters.len() {
            return Some((line, next));
        }
        (line + 1..layout.lines.len()).find_map(|line| {
            (!layout.lines[line].finalized.data.clusters.is_empty()).then_some((line, 0))
        })
    } else {
        cluster
            .checked_sub(1)
            .map(|cluster| (line, cluster))
            .or_else(|| {
                (0..line).rev().find_map(|line| {
                    layout.lines[line]
                        .finalized
                        .data
                        .clusters
                        .len()
                        .checked_sub(1)
                        .map(|cluster| (line, cluster))
                })
            })
    }
}

/// One traversal over actual neighboring logical rows. Source lookup locates
/// the input only; an executed deletion gap cannot become a fabricated end.
fn word_target<K: Copy>(
    value: Cursor,
    forward: bool,
    extent: crate::Range,
    neighbors: impl Fn(usize) -> [Option<K>; 2],
    facts: impl Fn(K) -> (crate::Range, Boundary, u8),
    adjacent: impl Fn(K, bool) -> Option<K>,
) -> WordTarget<K> {
    let [upstream, downstream] = neighbors(value.index());
    let leading = |row| {
        let (source, _, level) = facts(row);
        WordTarget {
            cursor: Cursor::new(source.start, directional_affinity(level & 1 != 0, true)),
            row: Some((row, false)),
        }
    };
    if !forward && upstream.is_none() {
        // No original row precedes this source side. A deleted prefix is not
        // permission to move forward into the first surviving word.
        return WordTarget {
            cursor: Cursor::new(extent.start, Affinity::After),
            row: None,
        };
    }
    let Some(start) = (if forward {
        downstream.or(upstream)
    } else {
        upstream.or(downstream)
    }) else {
        return WordTarget {
            cursor: value,
            row: None,
        };
    };
    // Original positions between surviving rows have not reached the next
    // row's word boundary. At a real surviving boundary, ordinary traversal
    // skips that initial stop regardless of its raw geometric affinity.
    let (source, boundary, _) = facts(start);
    let initial_boundary_ahead = if forward {
        value.index() < source.start
    } else {
        source.end < value.index()
            && downstream.is_none_or(|row| value.index() < facts(row).0.start)
    };
    if initial_boundary_ahead && boundary != Boundary::None {
        return leading(start);
    }
    let mut current = start;
    loop {
        let Some(next) = adjacent(current, forward) else {
            return if forward {
                WordTarget {
                    cursor: Cursor::new(extent.end, Affinity::Before),
                    row: Some((current, true)),
                }
            } else {
                leading(current)
            };
        };
        let (_, boundary, _) = facts(next);
        if boundary != Boundary::None {
            return leading(next);
        }
        current = next;
    }
}

fn visual_move(layout: &PlacedLayout, value: Cursor, rightward: bool) -> Cursor {
    let [left, right] = visual_pair(layout, value);
    if let (Some(left), Some(right)) = (left, right)
        && left.soft(layout)
    {
        let left_rtl = left.rtl(layout);
        if rightward
            && ((left_rtl && value.affinity() == Affinity::After)
                || (!left_rtl && value.affinity() == Affinity::Before))
        {
            let range = right.get(layout).source;
            return normalize(
                layout,
                if right.rtl(layout) {
                    range.end
                } else {
                    range.start
                },
                if left_rtl {
                    Affinity::Before
                } else {
                    Affinity::After
                },
            );
        }
        if !rightward {
            if left_rtl && value.affinity() == Affinity::Before {
                let range = left.get(layout).source;
                return normalize(
                    layout,
                    if right.rtl(layout) {
                        range.start
                    } else {
                        range.end
                    },
                    Affinity::After,
                );
            }
            if !left_rtl && value.affinity() == Affinity::After {
                let range = right.get(layout).source;
                return normalize(
                    layout,
                    if right.rtl(layout) {
                        range.end
                    } else {
                        range.start
                    },
                    Affinity::Before,
                );
            }
        }
    }
    let Some(path) = (if rightward { right } else { left }) else {
        return value;
    };
    let range = path.get(layout).source;
    let index = if path.rtl(layout) == rightward {
        range.start
    } else {
        range.end
    };
    // Moving left enters the cluster's logical start for LTR, end for RTL.
    let index = if rightward {
        index
    } else if path.rtl(layout) {
        range.end
    } else {
        range.start
    };
    normalize(
        layout,
        index,
        directional_affinity(path.rtl(layout), rightward),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Font, FontCatalogueMode, Options, Source, Style, System, SystemOptions};

    fn system() -> (System, Style) {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let style = Style {
            font: Font::new().family("Ahem"),
            size: 20.0,
            ..Style::default()
        };
        (system, style)
    }

    #[test]
    fn logical_words_keep_upstream_search_and_directional_affinity() {
        let (mut system, style) = system();
        let output = system
            .layout(
                Source::new("one two"),
                style.clone(),
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        // A word move starts beyond the adjacent cluster, not at that cluster.
        for affinity in [Affinity::Before, Affinity::After] {
            for (index, expected) in [
                (4, Cursor::new(0, Affinity::After)),
                (5, Cursor::new(3, Affinity::Before)),
                (6, Cursor::new(4, Affinity::Before)),
                (7, Cursor::new(4, Affinity::Before)),
            ] {
                assert_eq!(
                    output
                        .move_cursor(Cursor::new(index, affinity), Movement::PreviousWord)
                        .unwrap(),
                    expected
                );
            }
            assert_eq!(
                output
                    .move_cursor(Cursor::new(0, affinity), Movement::NextWord)
                    .unwrap(),
                Cursor::new(3, Affinity::Before)
            );
        }
        let rtl = system
            .layout(
                Source::new("אב גד"),
                style,
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            rtl.move_cursor(Cursor::new(0, Affinity::After), Movement::NextWord)
                .unwrap(),
            Cursor::new(2, Affinity::After)
        );
        assert_eq!(
            rtl.move_cursor(Cursor::new(4, Affinity::After), Movement::PreviousWord)
                .unwrap(),
            Cursor::new(2, Affinity::After)
        );
    }

    #[test]
    fn visual_steps_keep_rtl_zero_advance_and_soft_boundary_stops() {
        let (mut system, style) = system();
        let rtl = system
            .layout(
                Source::new("אב"),
                style.clone(),
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        for (from, movement, expected) in [
            (
                Cursor::new(0, Affinity::After),
                Movement::PreviousCluster,
                Cursor::new(1, Affinity::Before),
            ),
            (
                Cursor::new(1, Affinity::Before),
                Movement::PreviousCluster,
                Cursor::new(2, Affinity::Before),
            ),
            (
                Cursor::new(1, Affinity::Before),
                Movement::NextCluster,
                Cursor::new(0, Affinity::After),
            ),
            (
                Cursor::new(2, Affinity::Before),
                Movement::NextCluster,
                Cursor::new(1, Affinity::After),
            ),
        ] {
            assert_eq!(rtl.move_cursor(from, movement).unwrap(), expected);
        }
        for text in ["A\u{200e}B", "A\nB"] {
            let output = system
                .layout(
                    Source::new(text),
                    style.clone(),
                    Options {
                        quantize: false,
                        ..Options::default()
                    },
                )
                .unwrap();
            for affinity in [Affinity::Before, Affinity::After] {
                assert_eq!(
                    output
                        .move_cursor(Cursor::new(1, affinity), Movement::NextCluster)
                        .unwrap(),
                    Cursor::new(2, Affinity::Before)
                );
                assert_eq!(
                    output
                        .move_cursor(Cursor::new(2, affinity), Movement::PreviousCluster)
                        .unwrap(),
                    Cursor::new(1, Affinity::After)
                );
            }
        }
        let soft = system
            .layout(
                Source::new("A B"),
                style,
                Options {
                    width: Some(40.0),
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            soft.move_cursor(Cursor::new(2, Affinity::Before), Movement::NextCluster)
                .unwrap(),
            Cursor::new(2, Affinity::After)
        );
        assert_eq!(
            soft.move_cursor(Cursor::new(2, Affinity::After), Movement::PreviousCluster)
                .unwrap(),
            Cursor::new(2, Affinity::Before)
        );
        assert_eq!(
            soft.cursor(Cursor::new(2, Affinity::Before))
                .unwrap()
                .rect()
                .origin,
            Point::new(40.0, 0.0)
        );
        assert_eq!(
            soft.cursor(Cursor::new(2, Affinity::After))
                .unwrap()
                .rect()
                .origin,
            Point::new(0.0, 20.0)
        );
    }

    #[test]
    fn line_movements_use_original_ranges_and_preserve_cr_lf_only_visible_end() {
        let (mut system, style) = system();
        let soft = system
            .layout(
                Source::new("A B"),
                style.clone(),
                Options {
                    width: Some(40.0),
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        for affinity in [Affinity::Before, Affinity::After] {
            assert_eq!(
                soft.move_cursor(Cursor::new(2, affinity), Movement::LineStart)
                    .unwrap(),
                Cursor::new(2, Affinity::After)
            );
            assert_eq!(
                soft.move_cursor(Cursor::new(2, affinity), Movement::LineEnd)
                    .unwrap(),
                Cursor::new(3, Affinity::Before)
            );
        }
        assert_eq!(
            soft.move_cursor(Cursor::new(2, Affinity::Before), Movement::NextLine)
                .unwrap(),
            Cursor::new(2, Affinity::Before)
        );
        let hard = system
            .layout(
                Source::new("A\nB"),
                style.clone(),
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            hard.move_cursor(Cursor::new(0, Affinity::After), Movement::NextLine)
                .unwrap(),
            Cursor::new(2, Affinity::After)
        );
        assert_eq!(
            hard.move_cursor(Cursor::new(2, Affinity::Before), Movement::PreviousLine)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        for (text, expected) in [
            ("A\nB", 1),
            ("A\rB", 1),
            ("A\u{2028}B", 2),
            ("A\u{2029}B", 2),
        ] {
            let output = system
                .layout(
                    Source::new(text),
                    style.clone(),
                    Options {
                        quantize: false,
                        ..Options::default()
                    },
                )
                .unwrap();
            assert_eq!(
                output
                    .move_cursor(Cursor::new(0, Affinity::After), Movement::LineEnd)
                    .unwrap(),
                Cursor::new(expected, Affinity::Before)
            );
        }
        let terminal = system
            .layout(
                Source::new("A\n"),
                style,
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            terminal
                .move_cursor(Cursor::new(2, Affinity::Before), Movement::LineStart)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        assert_eq!(
            terminal
                .move_cursor(Cursor::new(2, Affinity::Before), Movement::LineEnd)
                .unwrap(),
            Cursor::new(1, Affinity::Before)
        );
    }

    #[test]
    fn visual_queries_keep_atomic_gaps_and_left_midpoint_ties() {
        let (mut system, style) = system();
        let mut source = Source::new("AB");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(1),
            crate::InlineBoxKind::InFlow,
            1,
            crate::Size::new(20.0, 20.0),
        ));
        let output = system
            .layout(
                source,
                style,
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            output
                .cursor(Cursor::new(1, Affinity::After))
                .unwrap()
                .rect()
                .origin
                .x,
            20.0
        );
        let Hit::Text(left) = output.hit(Point::new(10.0, 10.0)) else {
            panic!("actual text hit")
        };
        assert_eq!(left.source_cursor(), Cursor::new(0, Affinity::After));
        let Hit::Text(right) = output.hit(Point::new(50.0, 10.0)) else {
            panic!("actual text hit")
        };
        assert_eq!(right.source_cursor(), Cursor::new(1, Affinity::After));
        assert_eq!(
            output.position_geometry(&right).unwrap().rect().origin.x,
            40.0
        );
        let selected = output
            .selection(crate::Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(2, Affinity::Before),
            ))
            .unwrap();
        assert_eq!(selected.rects().len(), 1);
        // The 20-high box raises ascent to 20 while descent remains 4. With
        // line height 20, leading is -4 and baseline 18: block bounds are -2..22.
        assert_eq!(selected.rects()[0].rect(), Rect::new(0.0, -2.0, 60.0, 24.0));
    }

    #[test]
    fn document_and_cluster_endpoints_keep_original_pair_and_empty_affinities() {
        let (mut system, style) = system();
        let output = system
            .layout(
                Source::new("a😀 b"),
                style.clone(),
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        assert_eq!(
            output
                .move_cursor(Cursor::new(1, Affinity::After), Movement::NextCluster)
                .unwrap(),
            Cursor::new(3, Affinity::Before)
        );
        assert_eq!(
            output
                .move_cursor(Cursor::new(3, Affinity::Before), Movement::PreviousCluster)
                .unwrap(),
            Cursor::new(1, Affinity::After)
        );
        assert_eq!(
            output
                .move_cursor(Cursor::new(0, Affinity::Before), Movement::PreviousCluster)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        assert_eq!(
            output
                .move_cursor(Cursor::new(5, Affinity::After), Movement::NextCluster)
                .unwrap(),
            Cursor::new(5, Affinity::Before)
        );
        assert_eq!(
            output
                .move_cursor(Cursor::new(3, Affinity::After), Movement::DocumentStart)
                .unwrap(),
            Cursor::new(0, Affinity::After)
        );
        assert_eq!(
            output
                .move_cursor(Cursor::new(1, Affinity::Before), Movement::DocumentEnd)
                .unwrap(),
            Cursor::new(5, Affinity::Before)
        );
        let empty = system
            .layout(
                Source::new(""),
                style,
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        for movement in [
            Movement::PreviousCluster,
            Movement::NextCluster,
            Movement::PreviousWord,
            Movement::NextWord,
        ] {
            for affinity in [Affinity::Before, Affinity::After] {
                assert_eq!(
                    empty
                        .move_cursor(Cursor::new(0, affinity), movement)
                        .unwrap(),
                    Cursor::new(0, Affinity::Before)
                );
            }
        }
    }
}
