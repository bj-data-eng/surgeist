// Copyright 2025 the Parley Authors
// Copyright 2021 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Cursor geometry and movement adapted from Parley 0.9 `editing/cursor.rs`
//! (cd7237d6c75c495f6aa0941e3ecf9182dc62d33b6f0d4599a6a117765265e156)
//! and `layout/cluster.rs`
//! (43795e0d29905679d983080f99ea070d665043601c192cf822550228d10c6899).
//! Queries traverse the actual committed L2 order, including zero-advance text.

use super::super::{
    BreakKind, PlacedLine,
    analysis::Boundary,
    layout::data::{FinalizedCluster, FinalizedVisualItem, PlacedLayout, Whitespace},
};
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
        self.visual + 1 == layout.lines[self.line].finalized.data.visual_order.len()
    }

    fn soft(self, layout: &PlacedLayout) -> bool {
        self.end_of_line(layout)
            && layout.lines[self.line].finalized.selection.break_kind == BreakKind::Soft
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
    if let Some(cluster) = at_source(layout, index) {
        let index = cluster.get(layout).source.start;
        Cursor::new(
            index,
            if index == 0 {
                Affinity::After
            } else {
                affinity
            },
        )
    } else {
        let end = layout.prepared.analyzed.source.text().len();
        Cursor::new(end, Affinity::Before)
    }
}

pub(crate) fn hit(layout: &PlacedLayout, point: Point) -> Hit {
    if !point.x.is_finite() || !point.y.is_finite() {
        return Hit::None;
    }
    for line in &layout.lines {
        for item in &line.finalized.data.boxes {
            let rect = item.rect;
            if Rect::new(
                line.origin.x + rect.origin.x,
                line.origin.y + rect.origin.y,
                rect.size.width,
                rect.size.height,
            )
            .contains(point)
            {
                return Hit::InlineBox(
                    layout.prepared.analyzed.source.boxes()[item.occurrence].id(),
                );
            }
        }
    }
    if !Rect::new(0.0, 0.0, layout.width, layout.height).contains(point) {
        return Hit::None;
    }
    Hit::Text(from_point(layout, point))
}

pub(crate) fn cursor(layout: &PlacedLayout, value: Cursor) -> Result<CursorGeometry> {
    Ok(CursorGeometry::new(
        geometry(layout, admit(layout, value)?).1,
    ))
}

pub(super) fn admit(layout: &PlacedLayout, value: Cursor) -> Result<Cursor> {
    crate::range::validate_index(
        layout.prepared.analyzed.source.text(),
        value.index(),
        "cursor",
    )?;
    Ok(normalize(
        layout,
        layout.interaction.caret(value.index(), value.affinity()),
        value.affinity(),
    ))
}

fn logical_pair(layout: &PlacedLayout, value: Cursor) -> [Option<ClusterPath>; 2] {
    [
        value
            .index()
            .checked_sub(1)
            .and_then(|index| at_source(layout, index)),
        at_source(layout, value.index()),
    ]
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

fn line_rect(line: &PlacedLine, x: f64) -> Rect {
    let metrics = line.finalized.data.metrics;
    Rect::new(
        line.origin.x + x,
        line.origin.y + metrics.block_min,
        1.0,
        metrics.block_max - metrics.block_min,
    )
}

pub(super) fn from_point(layout: &PlacedLayout, point: Point) -> Cursor {
    let line_index = layout
        .lines
        .iter()
        .position(|line| point.y < line.origin.y + line.finalized.data.metrics.block_max);
    if let Some(line_index) = line_index {
        let line = &layout.lines[line_index];
        let data = &line.finalized.data;
        let x = point.x - line.origin.x;
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
                if next.run != first_cluster.run || next.level != first_cluster.level {
                    break;
                }
                end += 1;
            }
            let last = match data.visual_order[end - 1] {
                FinalizedVisualItem::Cluster(index) => &data.clusters[index],
                FinalizedVisualItem::InlineBox(_) => unreachable!(),
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
                    return point_edge(layout, path, x > cluster.x + cluster.advance * 0.5);
                }
                visual += 1;
            }
        }
    }
    // The provider's non-exact query above the layout clamps to its first cluster.
    if point.y <= 0.0
        && let Some(first) = layout.lines.first()
        && let Some((visual, cluster)) = first
            .finalized
            .data
            .visual_order
            .iter()
            .enumerate()
            .find_map(|(visual, item)| match item {
                FinalizedVisualItem::Cluster(cluster) => Some((visual, *cluster)),
                FinalizedVisualItem::InlineBox(_) => None,
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
    Cursor::new(
        layout.prepared.analyzed.source.text().len(),
        Affinity::After,
    )
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
            && layout.lines[path.line].finalized.selection.break_kind == BreakKind::Explicit)
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
        Movement::DocumentStart => normalize(layout, 0, Affinity::After),
        Movement::DocumentEnd => normalize(
            layout,
            layout.prepared.analyzed.source.text().len(),
            Affinity::Before,
        ),
        Movement::LineStart | Movement::LineEnd => {
            source_line(layout, value.index()).map_or(value, |index| {
                let line = &layout.lines[index];
                if movement == Movement::LineStart {
                    normalize(
                        layout,
                        line.finalized.selection.source.start,
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
                let metrics = line.finalized.data.metrics;
                from_point(
                    layout,
                    Point::new(
                        geometry(layout, value).1.origin.x,
                        line.origin.y + (metrics.block_min + metrics.block_max) * 0.5,
                    ),
                )
            })
        }
        Movement::PreviousWord | Movement::NextWord => {
            logical_word(layout, value, movement == Movement::NextWord)
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
    Ok(Cursor::new(
        layout.interaction.caret(result.index(), result.affinity()),
        result.affinity(),
    ))
}

fn source_line(layout: &PlacedLayout, index: usize) -> Option<usize> {
    layout
        .lines
        .iter()
        .position(|line| line.finalized.selection.source.contains(index))
        .or_else(|| {
            layout.lines.iter().rposition(|line| {
                let range = line.finalized.selection.source;
                range.end == index && !range.is_empty()
            })
        })
        .or_else(|| {
            (index >= layout.prepared.analyzed.source.text().len())
                .then_some(layout.lines.len() - 1)
        })
}

fn visible_end(layout: &PlacedLayout, line: &PlacedLine) -> usize {
    let range = line.finalized.selection.source;
    let units = layout.prepared.analyzed.source.text().code_units();
    let mut end = range.end;
    // Preserve the existing standalone wrapper's exact CR/LF policy.
    while end > range.start && matches!(units[end - 1], 0x0a | 0x0d) {
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

fn logical_word(layout: &PlacedLayout, value: Cursor, forward: bool) -> Cursor {
    let [upstream, downstream] = logical_pair(layout, value);
    let Some(start) = (if forward {
        downstream.or(upstream)
    } else {
        upstream.or(downstream)
    }) else {
        return value;
    };
    let mut current = start;
    loop {
        let range = current.get(layout).source;
        let next = if forward {
            at_source(layout, range.end)
        } else {
            range
                .start
                .checked_sub(1)
                .and_then(|index| at_source(layout, index))
        };
        let Some(next) = next else {
            return if forward {
                normalize(layout, usize::MAX, Affinity::After)
            } else {
                normalize(
                    layout,
                    start.get(layout).source.start,
                    directional_affinity(start.rtl(layout), true),
                )
            };
        };
        if next.get(layout).boundary != Boundary::None {
            return normalize(
                layout,
                next.get(layout).source.start,
                directional_affinity(next.rtl(layout), true),
            );
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
        assert_eq!(
            output.hit(Point::new(10.0, 10.0)),
            Hit::Text(Cursor::new(0, Affinity::After))
        );
        assert_eq!(
            output.hit(Point::new(50.0, 10.0)),
            Hit::Text(Cursor::new(1, Affinity::After))
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
