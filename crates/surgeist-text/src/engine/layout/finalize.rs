// Copyright 2021 the Parley Authors
// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Selected-line metrics and alignment adapted from Parley 0.9
//! `layout/line_break.rs` (7793b3a6d1ef56f1c7551c4ec4595ece5a33e09cf92dddfa08d3be46a0e6f91b)
//! and `layout/alignment.rs`. Adjustments belong to this line, never preparation.

use std::sync::Arc;

use icu_properties::props::BidiClass;

use crate::{
    Alignment, BidiLevel, Error, ErrorCode, InlineBoxKind, InlineWhitespaceEdge, Rect, Result,
    SuppliedInlineParticipant, SuppliedLine, TextNumericError, TextNumericPhase,
};

use super::super::{
    BreakKind, FinalizedLine, LineSelection, PreparedData,
    shape::{self, ShapeContext},
};
use super::{add, data::*, finite};

pub(super) fn finalize(
    prepared: Arc<PreparedData>,
    selection: LineSelection,
    scratch: &mut ShapeContext,
) -> Result<FinalizedLine> {
    crate::range::validate(prepared.analyzed.source.text(), selection.source)?;
    if selection.items.start > selection.items.end
        || selection.items.end > prepared.shaped.items.len()
    {
        return Err(invalid("selected line has invalid prepared item extents"));
    }
    let mut shaped = shape::finalize(&prepared, &selection, scratch)?;
    validate_rows(&prepared, &shaped)?;
    let levels = line_levels(&prepared, &shaped)?;
    let base = prepared.analyzed.analysis.base_level;
    let before = line_metrics(&prepared, &shaped)?;
    let width = selection.available.unwrap_or_else(|| {
        (before.advance + selection.indent.max(0.0) - before.trailing_whitespace).max(0.0)
    });
    finite(
        "line available width",
        TextNumericPhase::Finalization,
        width,
    )?;
    justify(&mut shaped, &selection, width, &before)?;
    let mut metrics = line_metrics(&prepared, &shaped)?;
    metrics.inline_min = 0.0;
    metrics.inline_max = width;
    metrics.offset = alignment_offset(&selection, base, width, &metrics)?;

    let participants: Vec<_> = shaped
        .items
        .iter()
        .zip(&levels)
        .map(|(item, &level)| {
            let level = BidiLevel::try_new(level)
                .map_err(|_| invalid("selected line has an invalid resolved level"))?;
            Ok(match item {
                ShapedItem::TextCluster(_) => SuppliedInlineParticipant::Text {
                    level,
                    edge: InlineWhitespaceEdge::Preserve,
                },
                ShapedItem::InlineBox { .. } => SuppliedInlineParticipant::Atomic { level },
            })
        })
        .collect::<Result<_>>()?;
    let supplied = SuppliedLine::new(&participants);
    // Standalone Preserve retains every text slot. Edge processing and L2 still
    // use the same canonical supplied-line owner as the composed formatter.
    let edges = supplied.edges();
    let mut order = vec![0; participants.len()];
    let order = supplied
        .reorder_into(&mut order)
        .map_err(|_| invalid("line order scratch does not match its actual participants"))?;
    let mut data = FinalizedLineData {
        clusters: shaped
            .clusters
            .iter()
            .map(|cluster| FinalizedCluster {
                source: cluster.source,
                run: cluster.run,
                x: 0.0,
                advance: cluster.advance,
                level: cluster.level,
                boundary: cluster.boundary,
                whitespace: cluster.whitespace,
            })
            .collect(),
        metrics,
        ..FinalizedLineData::default()
    };
    let mut pen = metrics.offset;
    let mut last_run = None;
    for (ordinal, _) in order.iter() {
        let level = levels[ordinal];
        match shaped.items[ordinal] {
            ShapedItem::InlineBox { occurrence, .. } => {
                let item = prepared
                    .analyzed
                    .source
                    .boxes()
                    .get(occurrence)
                    .ok_or_else(|| invalid("line box has no original occurrence"))?;
                let size = item.size();
                data.visual_order
                    .push(FinalizedVisualItem::InlineBox(data.boxes.len()));
                data.boxes.push(FinalizedBox {
                    occurrence,
                    rect: Rect::new(
                        pen,
                        add(
                            "atomic block origin",
                            TextNumericPhase::Finalization,
                            metrics.baseline,
                            -size.height,
                        )?,
                        size.width,
                        size.height,
                    ),
                });
                last_run = None;
                if item.kind() == InlineBoxKind::InFlow {
                    pen = add(
                        "atomic logical advance",
                        TextNumericPhase::Finalization,
                        pen,
                        size.width,
                    )?;
                }
            }
            ShapedItem::TextCluster(cluster_index) => {
                data.visual_order
                    .push(FinalizedVisualItem::Cluster(cluster_index));
                let cluster = &shaped.clusters[cluster_index];
                let cluster_x = pen;
                let discarded = edges.is_discarded(ordinal) == Some(true);
                // RTL ligature/component shares are visited before their logical
                // start. The one actual glyph belongs at the group's visual
                // origin, not after those already-positioned component shares.
                let component_shift = if level & 1 != 0 && cluster.ligature_start {
                    shaped.clusters[cluster_index + 1..]
                        .iter()
                        .take_while(|next| next.ligature_component && next.run == cluster.run)
                        .try_fold(0.0, |total, next| {
                            add(
                                "ligature component advance",
                                TextNumericPhase::Finalization,
                                total,
                                next.advance,
                            )
                        })?
                } else {
                    0.0
                };
                let glyph_x = add(
                    "ligature glyph origin",
                    TextNumericPhase::Finalization,
                    cluster_x,
                    -component_shift,
                )?;
                let mut glyph_pen = 0.0;
                if !discarded && cluster.whitespace != Whitespace::Newline {
                    for glyph in &shaped.glyphs[cluster.glyphs.clone()] {
                        let continue_run = last_run.filter(|&run_index| {
                            let run: &FinalizedRun = &data.runs[run_index];
                            run.run == cluster.run && run.style == glyph.style && run.level == level
                        });
                        let run_index = if let Some(index) = continue_run {
                            index
                        } else {
                            let index = data.runs.len();
                            data.runs.push(FinalizedRun {
                                run: cluster.run,
                                style: glyph.style,
                                level,
                                source: glyph.source,
                                glyphs: data.glyphs.len()..data.glyphs.len(),
                                x: add(
                                    "paint run origin",
                                    TextNumericPhase::Finalization,
                                    glyph_x,
                                    glyph_pen,
                                )?,
                                advance: 0.0,
                            });
                            last_run = Some(index);
                            index
                        };
                        let run = &mut data.runs[run_index];
                        let actual_pen = add(
                            "glyph logical pen",
                            TextNumericPhase::Finalization,
                            add(
                                "glyph line origin",
                                TextNumericPhase::Finalization,
                                glyph_x,
                                glyph_pen,
                            )?,
                            -run.x,
                        )?;
                        data.glyphs.push(FinalizedGlyph {
                            glyph: *glyph,
                            pen: actual_pen,
                        });
                        run.glyphs.end = data.glyphs.len();
                        run.source.start = run.source.start.min(glyph.source.start);
                        run.source.end = run.source.end.max(glyph.source.end);
                        glyph_pen = add(
                            "glyph logical advance",
                            TextNumericPhase::Finalization,
                            glyph_pen,
                            f64::from(glyph.advance),
                        )?;
                        run.advance = add(
                            "paint run advance",
                            TextNumericPhase::Finalization,
                            add(
                                "paint glyph end",
                                TextNumericPhase::Finalization,
                                glyph_x,
                                glyph_pen,
                            )?,
                            -run.x,
                        )?;
                    }
                }
                let output = &mut data.clusters[cluster_index];
                output.x = cluster_x;
                output.level = level;
                output.advance = if discarded || cluster.whitespace == Whitespace::Newline {
                    0.0
                } else {
                    cluster.advance
                };
                if let Some(index) = last_run {
                    let run = &mut data.runs[index];
                    if run.run == cluster.run && run.level == level {
                        run.advance = add(
                            "paint run logical advance",
                            TextNumericPhase::Finalization,
                            add(
                                "paint cluster end",
                                TextNumericPhase::Finalization,
                                cluster_x,
                                output.advance,
                            )?,
                            -run.x,
                        )?;
                    }
                }
                pen = add(
                    "line logical advance",
                    TextNumericPhase::Finalization,
                    pen,
                    output.advance,
                )?;
            }
        }
    }
    // Zero-glyph controls and actual empty lines still retain a real font/style
    // row, rather than publishing a fabricated source space or fallback range.
    if data.runs.is_empty()
        && (shaped.items.is_empty()
            || shaped
                .items
                .iter()
                .any(|item| matches!(item, ShapedItem::TextCluster(_))))
    {
        let run = shaped.empty_run.as_ref().or_else(|| shaped.runs.first());
        if let Some(run) = run {
            let source = selection.source;
            let run_index =
                if let Some(index) = shaped.runs.iter().position(|r| std::ptr::eq(r, run)) {
                    index
                } else {
                    let run = run.clone();
                    shaped.runs.push(run);
                    shaped.runs.len() - 1
                };
            data.runs.push(FinalizedRun {
                run: run_index,
                style: shaped.runs[run_index].style,
                level: base,
                source,
                glyphs: 0..0,
                x: metrics.offset,
                advance: 0.0,
            });
        }
    }
    data.font_runs = shaped.runs;
    Ok(FinalizedLine {
        prepared,
        selection,
        data,
    })
}

fn validate_rows(prepared: &PreparedData, shaped: &ShapedData) -> Result<()> {
    for cluster in &shaped.clusters {
        crate::range::validate(prepared.analyzed.source.text(), cluster.source)?;
        if cluster.run >= shaped.runs.len()
            || cluster.glyphs.start > cluster.glyphs.end
            || cluster.glyphs.end > shaped.glyphs.len()
            || cluster.atoms.start > cluster.atoms.end
            || cluster.atoms.end > prepared.analyzed.analysis.infos.len()
        {
            return Err(invalid(
                "shaping returned an invalid selected cluster association",
            ));
        }
        finite(
            "cluster advance",
            TextNumericPhase::Finalization,
            cluster.advance,
        )?;
    }
    for glyph in &shaped.glyphs {
        crate::range::validate(prepared.analyzed.source.text(), glyph.source)?;
        for (field, value) in [
            ("glyph x offset", glyph.x),
            ("glyph y offset", glyph.y),
            ("glyph advance", glyph.advance),
        ] {
            finite(field, TextNumericPhase::Finalization, f64::from(value))?;
        }
    }
    for item in &shaped.items {
        match item {
            ShapedItem::TextCluster(index) if *index >= shaped.clusters.len() => {
                return Err(invalid("selected text item has no cluster"));
            }
            ShapedItem::InlineBox { occurrence, .. }
                if *occurrence >= prepared.analyzed.source.boxes().len() =>
            {
                return Err(invalid("selected atomic item has no original occurrence"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn line_metrics(prepared: &PreparedData, shaped: &ShapedData) -> Result<LineMetrics> {
    let mut result = LineMetrics::default();
    let mut have_metrics = false;
    for item in &shaped.items {
        match *item {
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                let metrics = shaped.runs[cluster.run].metrics;
                result.line_height = result.line_height.max(metrics.line_height);
                if cluster.whitespace != Whitespace::Newline {
                    result.advance = add(
                        "line advance",
                        TextNumericPhase::Finalization,
                        result.advance,
                        cluster.advance,
                    )?;
                }
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let item = &prepared.analyzed.source.boxes()[occurrence];
                if item.kind() == InlineBoxKind::InFlow {
                    result.advance = add(
                        "line atomic advance",
                        TextNumericPhase::Finalization,
                        result.advance,
                        item.size().width,
                    )?;
                    result.line_height = result.line_height.max(item.size().height);
                }
            }
        }
    }
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if !have_metrics && cluster.is_whitespace {
                    continue;
                }
                let metrics = shaped.runs[cluster.run].metrics;
                result.ascent = result.ascent.max(metrics.ascent);
                result.descent = result.descent.max(metrics.descent);
                have_metrics = true;
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let item = &prepared.analyzed.source.boxes()[occurrence];
                if item.kind() == InlineBoxKind::InFlow {
                    result.ascent = result.ascent.max(item.size().height);
                    have_metrics = true;
                }
            }
        }
    }
    let only_boxes = !shaped.items.is_empty()
        && shaped
            .items
            .iter()
            .all(|item| matches!(item, ShapedItem::InlineBox { .. }));
    if !have_metrics && !only_boxes {
        let font = shaped
            .runs
            .first()
            .or(shaped.empty_run.as_ref())
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::FontLoadFailed,
                    "empty line has no admitted font metrics",
                )
            })?;
        result.ascent = font.metrics.ascent;
        result.descent = font.metrics.descent;
        result.line_height = result.line_height.max(font.metrics.line_height);
    }
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::TextCluster(index) if shaped.clusters[index].is_whitespace => {
                if shaped.clusters[index].whitespace != Whitespace::Newline {
                    result.trailing_whitespace = add(
                        "line trailing advance",
                        TextNumericPhase::Finalization,
                        result.trailing_whitespace,
                        shaped.clusters[index].advance,
                    )?;
                }
            }
            _ => break,
        }
    }
    result.leading = result.line_height - (result.ascent + result.descent);
    let (ascent, descent) = if prepared.quantize {
        (result.ascent.round(), result.descent.round())
    } else {
        (result.ascent, result.descent)
    };
    let leading = result.line_height - (ascent + descent);
    let (above, below) = if prepared.quantize {
        let above = (leading * 0.5).floor();
        (above, leading.round() - above)
    } else {
        (leading * 0.5, leading * 0.5)
    };
    // These are derived metric combinations. Ordinary floating-point rounding
    // of a computed leading is not a newly requested authored displacement.
    result.baseline = ascent + above;
    result.block_min = result.baseline - ascent - above.max(0.0);
    result.block_max = result.baseline + descent + below.max(0.0);
    for value in [
        result.advance,
        result.trailing_whitespace,
        result.line_height,
        result.baseline,
        result.block_min,
        result.block_max,
        result.ascent,
        result.descent,
    ] {
        finite(
            "finalized line metrics",
            TextNumericPhase::Finalization,
            value,
        )?;
    }
    Ok(result)
}

fn justify(
    shaped: &mut ShapedData,
    selection: &LineSelection,
    width: f64,
    metrics: &LineMetrics,
) -> Result<()> {
    if selection.alignment != Alignment::Justify || selection.break_kind != BreakKind::Soft {
        return Ok(());
    }
    let mut eligible: Vec<_> = shaped
        .items
        .iter()
        .filter_map(|item| match *item {
            ShapedItem::TextCluster(index)
                if shaped.clusters[index].whitespace.is_space_or_nbsp() =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect();
    if let Some(ShapedItem::TextCluster(index)) = shaped.items.last()
        && eligible.last() == Some(index)
    {
        eligible.pop();
    }
    let preliminary = width - selection.indent - metrics.advance + metrics.trailing_whitespace;
    if preliminary <= 0.0 || eligible.is_empty() {
        return Ok(());
    }
    let free = free_space(selection, width, metrics)?;
    let count = eligible.len() as f64;
    let adjustment = free / count;
    if adjustment == 0.0 {
        return Err(super::super::text_numeric_error(
            "justification adjustment",
            TextNumericPhase::Finalization,
            TextNumericError::PositiveLost,
            adjustment,
        ));
    }
    for index in eligible {
        let cluster = &mut shaped.clusters[index];
        let actual = add(
            "justification adjustment",
            TextNumericPhase::Finalization,
            cluster.advance,
            adjustment,
        )?;
        // No shaping provider consumes this logical adjustment. Preserve raw
        // canonical glyph offsets/advances and carry spacing in the f64 pen.
        cluster.advance = actual;
    }
    Ok(())
}

fn alignment_offset(
    selection: &LineSelection,
    base: u8,
    width: f64,
    metrics: &LineMetrics,
) -> Result<f64> {
    let rtl = base & 1 != 0;
    let mut offset = if rtl {
        -metrics.trailing_whitespace
    } else {
        selection.indent
    };
    let preliminary = width - selection.indent - metrics.advance + metrics.trailing_whitespace;
    let factor = if preliminary <= 0.0 {
        if rtl { 1.0 } else { 0.0 }
    } else {
        match (selection.alignment, rtl) {
            (Alignment::Right, _) | (Alignment::Start, true) | (Alignment::End, false) => 1.0,
            (Alignment::Center, _) => 0.5,
            (Alignment::Justify, true) if selection.break_kind != BreakKind::Soft => 1.0,
            _ => 0.0,
        }
    };
    if factor != 0.0 {
        let free = free_space(selection, width, metrics)?;
        let adjustment = free * factor;
        if free != 0.0 && adjustment == 0.0 {
            return Err(super::super::text_numeric_error(
                "line alignment adjustment",
                TextNumericPhase::Finalization,
                TextNumericError::NonZeroLost,
                free,
            ));
        }
        offset = add(
            "line alignment offset",
            TextNumericPhase::Finalization,
            offset,
            adjustment,
        )?;
    }
    finite(
        "line alignment offset",
        TextNumericPhase::Finalization,
        offset,
    )
}

fn free_space(selection: &LineSelection, width: f64, metrics: &LineMetrics) -> Result<f64> {
    let after_indent = add(
        "line available indentation",
        TextNumericPhase::Finalization,
        width,
        -selection.indent,
    )?;
    let after_text = add(
        "line available text advance",
        TextNumericPhase::Finalization,
        after_indent,
        -metrics.advance,
    )?;
    add(
        "line available trailing advance",
        TextNumericPhase::Finalization,
        after_text,
        metrics.trailing_whitespace,
    )
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Reset {
    Other,
    White,
    Removed,
    Separator,
}

fn line_levels(prepared: &PreparedData, shaped: &ShapedData) -> Result<Vec<u8>> {
    let base = prepared.analyzed.analysis.base_level;
    let mut levels = Vec::with_capacity(shaped.items.len());
    let mut roles = Vec::with_capacity(shaped.items.len());
    for item in &shaped.items {
        match *item {
            ShapedItem::InlineBox { level, .. } => {
                levels.push(level);
                roles.push(Reset::Other);
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                levels.push(cluster.level);
                let infos = prepared
                    .analyzed
                    .analysis
                    .infos
                    .get(cluster.atoms.clone())
                    .ok_or_else(|| invalid("cluster has no original bidi properties"))?;
                let mut role = Reset::Removed;
                for (info, _) in infos {
                    let next = match info.bidi_class {
                        BidiClass::ParagraphSeparator | BidiClass::SegmentSeparator => {
                            Reset::Separator
                        }
                        BidiClass::WhiteSpace
                        | BidiClass::LeftToRightIsolate
                        | BidiClass::RightToLeftIsolate
                        | BidiClass::FirstStrongIsolate
                        | BidiClass::PopDirectionalIsolate => Reset::White,
                        BidiClass::LeftToRightEmbedding
                        | BidiClass::RightToLeftEmbedding
                        | BidiClass::LeftToRightOverride
                        | BidiClass::RightToLeftOverride
                        | BidiClass::PopDirectionalFormat
                        | BidiClass::BoundaryNeutral => Reset::Removed,
                        _ => Reset::Other,
                    };
                    if next == Reset::Separator {
                        role = next;
                        break;
                    }
                    if next == Reset::Other {
                        role = next;
                        break;
                    }
                    if next == Reset::White {
                        role = next;
                    }
                }
                roles.push(role);
            }
        }
    }
    for index in 0..levels.len() {
        if roles[index] != Reset::Separator {
            continue;
        }
        levels[index] = base;
        for prior in (0..index).rev() {
            match roles[prior] {
                Reset::Removed => continue,
                Reset::White => levels[prior] = base,
                _ => break,
            }
        }
    }
    for index in (0..levels.len()).rev() {
        match roles[index] {
            Reset::Removed => continue,
            Reset::White => levels[index] = base,
            _ => break,
        }
    }
    Ok(levels)
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorCode::LayoutFailed, message)
}
