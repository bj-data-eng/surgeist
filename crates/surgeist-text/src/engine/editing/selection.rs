// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Selection rectangles adapted from Parley 0.9 `editing/selection.rs`
//! (e96ab53b7c5c2f0dfeeec4f1f1c0ee568a66163825bf73460a67d0eab3a75faa).

use super::super::{
    BreakKind,
    layout::data::{FinalizedVisualItem, PlacedLayout},
};
use super::cursor::{admit, geometry};
use crate::{
    Affinity, Cursor, InlineBoxKind, Rect, Result, Selection, SelectionGeometry, SelectionRect,
};

pub(crate) fn selection(layout: &PlacedLayout, value: Selection) -> Result<SelectionGeometry> {
    crate::range::validate(layout.prepared.analyzed.source.text(), value.range())?;
    if value.is_collapsed() {
        return Ok(SelectionGeometry::default());
    }
    let range = layout.interaction.selection_range(value.range());
    let start_line = geometry(
        layout,
        admit(layout, Cursor::new(range.start, Affinity::After))?,
    )
    .0;
    let end_line = geometry(
        layout,
        admit(layout, Cursor::new(range.end, Affinity::Before))?,
    )
    .0;
    let mut output = Vec::new();
    for line_index in start_line..=end_line {
        let line = &layout.lines[line_index];
        let data = &line.finalized.data;
        let metrics = data.metrics;
        let y = line.origin.y + metrics.block_min;
        let height = metrics.block_max - metrics.block_min;
        let newline = if line.finalized.selection.break_kind == BreakKind::Explicit {
            (metrics.ascent + metrics.descent) * 0.25
        } else {
            0.0
        };
        if line_index != start_line && line_index != end_line {
            push(
                &mut output,
                line_index,
                line.origin.x + metrics.offset,
                y,
                metrics.advance + newline,
                height,
            );
            continue;
        }
        let mut start = metrics.offset;
        let mut current = start;
        let mut cluster_count = 0;
        let mut box_advance = 0.0;
        let mut seen_text = false;
        for item in &data.visual_order {
            match *item {
                FinalizedVisualItem::Cluster(index) => {
                    seen_text = true;
                    let cluster = &data.clusters[index];
                    let advance = cluster.advance + box_advance;
                    box_advance = 0.0;
                    if range.contains(cluster.source.start) {
                        cluster_count += 1;
                        current += advance;
                    } else {
                        push(
                            &mut output,
                            line_index,
                            line.origin.x + start,
                            y,
                            current - start,
                            height,
                        );
                        current += advance;
                        start = current;
                    }
                }
                FinalizedVisualItem::InlineBox(index) => {
                    let item = &data.boxes[index];
                    if layout.prepared.analyzed.source.boxes()[item.occurrence].kind()
                        == InlineBoxKind::InFlow
                    {
                        box_advance += item.rect.size.width;
                    }
                    // Leading atomic occurrences are not text selection geometry.
                    if !seen_text {
                        current += box_advance;
                        box_advance = 0.0;
                        start = current;
                    }
                }
            }
        }
        let extra = if line_index != end_line || (cluster_count != 0 && metrics.advance == 0.0) {
            newline
        } else {
            0.0
        };
        push(
            &mut output,
            line_index,
            line.origin.x + start,
            y,
            current + extra - start,
            height,
        );
    }
    Ok(SelectionGeometry::new(output))
}

fn push(output: &mut Vec<SelectionRect>, line: usize, x: f64, y: f64, width: f64, height: f64) {
    if width != 0.0 {
        output.push(SelectionRect::new(Rect::new(x, y, width, height), line));
    }
}
