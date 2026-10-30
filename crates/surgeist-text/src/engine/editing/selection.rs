// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Selection rectangles adapted from Parley 0.9 `editing/selection.rs`
//! (e96ab53b7c5c2f0dfeeec4f1f1c0ee568a66163825bf73460a67d0eab3a75faa).

use super::super::layout::data::{FinalizedVisualItem, PlacedLayout};
use super::cursor::{admit, geometry};
use crate::protocol::LineEnd;
use crate::{Affinity, Cursor, Rect, Result, Selection, SelectionGeometry, SelectionRect};

pub(crate) fn selection(layout: &PlacedLayout, value: Selection) -> Result<SelectionGeometry> {
    crate::range::validate(
        layout.prepared.analyzed.processed.source().text(),
        value.range(),
    )?;
    layout
        .prepared
        .analyzed
        .paragraph
        .admit(layout.prepared.analyzed.processed.source(), value.range())?;
    super::position::validate_source(layout, value.anchor())?;
    super::position::validate_source(layout, value.focus())?;
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
    between(layout, range, start_line, end_line)
}

pub(super) fn between(
    layout: &PlacedLayout,
    range: crate::Range,
    start_line: usize,
    end_line: usize,
) -> Result<SelectionGeometry> {
    with_membership(
        layout,
        start_line,
        end_line,
        |line_index, item| match item {
            FinalizedVisualItem::Cluster(cluster_index) => {
                let line = &layout.lines[line_index];
                let cluster = &line.finalized.data.clusters[cluster_index];
                overlaps(cluster.source, range)
                    || (cluster.source.is_empty()
                        && line
                            .finalized
                            .selected
                            .basis
                            .mapping
                            .pieces
                            .iter()
                            .any(|piece| {
                                let crate::protocol::prepared::SelectedMappingKind::Replacement {
                                    alternative,
                                    anchor,
                                } = piece.kind
                                else {
                                    return false;
                                };
                                let defining = match line
                                .finalized
                                .selected
                                .prepared
                                .tables
                                .alternatives[alternative]
                                .definition
                            {
                                crate::protocol::prepared::AlternativeDefinition::Source {
                                    source,
                                    ..
                                } => source,
                                crate::protocol::prepared::AlternativeDefinition::Control {
                                    ..
                                }
                                | crate::protocol::prepared::AlternativeDefinition::Automatic {
                                    ..
                                } => crate::Range::new(anchor, anchor),
                            };
                                overlaps(cluster.presentation, piece.presentation)
                                    && (overlaps(defining, range) || range.contains(anchor))
                            }))
            }
            FinalizedVisualItem::Cell(cell_index) => {
                let line = &layout.lines[line_index];
                let cell = &line.finalized.data.cells[cell_index];
                let output = line
                    .finalized
                    .selected
                    .prepared
                    .analyzed
                    .combination
                    .as_deref()
                    .expect("a finalized cell retains its authentic source plan");
                let group = &output.plan.groups[output.bindings[cell.binding].group];
                output.plan.parts[group.parts.clone()]
                    .iter()
                    .flat_map(|part| &part.pieces)
                    .any(|piece| overlaps(piece.aggregate, range))
            }
            FinalizedVisualItem::InlineBox(_) | FinalizedVisualItem::Anchor(_) => false,
        },
        |line_index| {
            layout.lines[line_index]
                .finalized
                .selected
                .presentation
                .projection
                .atoms
                .iter()
                .any(|atom| matches!(atom.code_point, 0x0a | 0x0d) && overlaps(atom.source, range))
        },
        |_, _| false,
        |_| None,
    )
}

/// The shared visual rectangle pass. Membership comes from either admitted
/// original-source selection or exact selected-presentation relation coverage.
/// Slot fallback retains real source-less controls, never transform removals.
pub(super) fn with_membership(
    layout: &PlacedLayout,
    start_line: usize,
    end_line: usize,
    mut selected: impl FnMut(usize, FinalizedVisualItem) -> bool,
    mut source_break_selected: impl FnMut(usize) -> bool,
    mut selected_anchor: impl FnMut(usize, usize) -> bool,
    mut slot_fallback: impl FnMut(usize) -> Option<(f64, f64)>,
) -> Result<SelectionGeometry> {
    let mut output = Vec::new();
    for line_index in start_line.min(end_line)..=start_line.max(end_line) {
        let line = &layout.lines[line_index];
        let data = &line.finalized.data;
        let metrics = data.metrics;
        let height = metrics.block_max - metrics.block_min;
        let newline = if line.finalized.selected.request.context.end == LineEnd::Forced
            && source_break_selected(line_index)
        {
            (metrics.ascent + metrics.descent) * 0.25
        } else {
            0.0
        };
        let mut group: Option<(f64, f64, f64, f64)> = None;
        let mut selected_clusters = 0;
        let mut selected_contributions = 0;
        for item in &data.visual_order {
            let (selected, left, right, reference, band) = match *item {
                FinalizedVisualItem::Cluster(index) => {
                    let cluster = &data.clusters[index];
                    let end = cluster.x + cluster.advance;
                    let selected = selected(line_index, *item);
                    selected_clusters += usize::from(selected);
                    (
                        selected,
                        cluster.x.min(end),
                        cluster.x.max(end),
                        cluster.reference,
                        None,
                    )
                }
                FinalizedVisualItem::Cell(index) => {
                    let cell = &data.cells[index];
                    (
                        selected(line_index, *item),
                        cell.inline.start,
                        cell.inline.end,
                        Some(cell.reference),
                        Some((cell.under.start, cell.under.end - cell.under.start)),
                    )
                }
                FinalizedVisualItem::Anchor(index) => {
                    let anchor = &data.anchors[index];
                    if anchor.inline_end == anchor.slot.x {
                        continue;
                    }
                    (
                        selected_anchor(line_index, index),
                        anchor.slot.x,
                        anchor.inline_end,
                        anchor.reference,
                        None,
                    )
                }
                FinalizedVisualItem::InlineBox(_) => continue,
            };
            if selected {
                selected_contributions += 1;
                let (top, cell_height) = band.unwrap_or_else(|| data.cell_band(reference));
                if let Some((start, end, old_top, old_height)) = group.take() {
                    if old_top == top && old_height == cell_height {
                        group = Some((start.min(left), end.max(right), top, cell_height));
                    } else {
                        push_local(
                            &mut output,
                            line_index,
                            line,
                            start..end,
                            old_top,
                            old_height,
                            false,
                        )?;
                        group = Some((left, right, top, cell_height));
                    }
                } else {
                    group = Some((left, right, top, cell_height));
                }
            } else if let Some((start, end, top, cell_height)) = group.take() {
                push_local(
                    &mut output,
                    line_index,
                    line,
                    start..end,
                    top,
                    cell_height,
                    false,
                )?;
            }
        }
        let extra = if line_index != end_line || (selected_clusters != 0 && metrics.advance == 0.0)
        {
            newline
        } else {
            0.0
        };
        if let Some((start, end, top, cell_height)) = group {
            push_local(
                &mut output,
                line_index,
                line,
                start..end + extra,
                top,
                cell_height,
                false,
            )?;
        } else if extra != 0.0 {
            push_local(
                &mut output,
                line_index,
                line,
                metrics.offset..metrics.offset + extra,
                metrics.block_min,
                height,
                false,
            )?;
        }
        if selected_contributions == 0
            && extra == 0.0
            && let Some((start, end)) = slot_fallback(line_index)
        {
            push_local(
                &mut output,
                line_index,
                line,
                start.min(end)..start.max(end),
                metrics.block_min,
                height,
                true,
            )?;
        }
    }
    Ok(SelectionGeometry::new(output))
}

fn overlaps(a: crate::Range, b: crate::Range) -> bool {
    a.start < b.end && b.start < a.end
}

fn push_local(
    output: &mut Vec<SelectionRect>,
    line_index: usize,
    line: &crate::engine::PlacedLine,
    inline: std::ops::Range<f64>,
    top: f64,
    height: f64,
    include_zero: bool,
) -> Result<()> {
    let (start, end) = (inline.start, inline.end);
    if !start.is_finite() || !end.is_finite() {
        return Err(crate::error::text_numeric_error(
            "selected source rectangle",
            crate::TextNumericPhase::Placement,
            crate::TextNumericError::NonFinite,
            end,
        ));
    }
    if end > start || (include_zero && end == start) {
        let rect = crate::layout::line_rect(line, Rect::new(start, top, end - start, height));
        for (field, value) in [
            ("selection x", rect.origin.x),
            ("selection y", rect.origin.y),
            ("selection width", rect.size.width),
            ("selection height", rect.size.height),
            ("selection right", rect.origin.x + rect.size.width),
            ("selection bottom", rect.origin.y + rect.size.height),
        ] {
            if !value.is_finite() {
                return Err(crate::error::text_numeric_error(
                    field,
                    crate::TextNumericPhase::Placement,
                    crate::TextNumericError::NonFinite,
                    value,
                ));
            }
        }
        output.push(SelectionRect::new(rect, line_index));
    }
    Ok(())
}
