// Copyright 2021 the Parley Authors
// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Selected-line metrics and alignment adapted from Parley 0.9
//! `layout/line_break.rs` (7793b3a6d1ef56f1c7551c4ec4595ece5a33e09cf92dddfa08d3be46a0e6f91b)
//! and `layout/alignment.rs`. Adjustments belong to this line, never preparation.

use std::{collections::BTreeMap, sync::Arc};

use icu_properties::props::BidiClass;

use crate::{
    Alignment, BidiLevel, Error, ErrorCode, InlineWhitespaceEdge, Point, Rect, Result,
    SuppliedInlineParticipant, SuppliedLine, TextNumericError, TextNumericPhase,
};

use super::super::{
    FinalizedLine,
    shape::{self, ShapeContext},
};
use super::{add, data::*, finite};
use crate::protocol::selection::{LineMetricPolicy, SelectedData};
use crate::protocol::{
    AtomicAllocation, ControlAllocation, ControlStrut, LineBoxAlignment, LineContext, LineEnd,
    ProtocolError, protocol_error,
};

pub(super) fn finalize(
    selected: Arc<SelectedData>,
    amounts: &[f64],
    scratch: &mut ShapeContext,
) -> Result<FinalizedLine> {
    let selection = &selected.request.context;
    let atomics = atomic_allocations(&selected)?;
    let controls = control_allocations(&selected)?;
    let (mut shaped, mapping) = shape::finalize(&selected, scratch)?;
    validate_rows(&selected, &shaped)?;
    let discarded = collapse_edges(&selected, &mut shaped)?;
    apply_spacing(&selected, &mut shaped, amounts, &discarded)?;
    let levels = line_levels(&selected, &shaped)?;
    let base = selected.presentation.analysis.base_level;
    let mut metrics = line_metrics(&selected, &shaped, &atomics, &controls)?;
    if selected.metric_policy != LineMetricPolicy::Native {
        include_control_struts(
            &mut metrics,
            controls.values().filter_map(|allocation| allocation.strut),
            selected.request.line_strut,
        )?;
    }
    let width = selection.available_inline.unwrap_or_else(|| {
        (metrics.advance + selection.indent.max(0.0) - metrics.trailing_whitespace).max(0.0)
    });
    finite(
        "line available width",
        TextNumericPhase::Finalization,
        width,
    )?;
    metrics.inline_min = 0.0;
    metrics.inline_max = width;
    metrics.offset = alignment_offset(selection, base, width, &metrics)?;

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
                ShapedItem::Anchor { .. } => SuppliedInlineParticipant::Boundary { level },
            })
        })
        .collect::<Result<_>>()?;
    let supplied = SuppliedLine::new(&participants);
    // Edge suppression was already applied through the canonical Text edge
    // owner. The full mixed sequence now receives its one canonical L2 order.
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
                presentation: {
                    let atoms = &selected.presentation.projection.atoms[cluster.atoms.clone()];
                    crate::Range::new(
                        atoms[0].presentation.start,
                        atoms[atoms.len() - 1].presentation.end,
                    )
                },
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
            ShapedItem::Anchor { unit, .. } => {
                data.visual_order
                    .push(FinalizedVisualItem::Anchor(data.anchors.len()));
                data.anchors.push(FinalizedAnchor {
                    unit,
                    slot: Point::new(
                        pen,
                        match selected.presentation.units[unit].kind {
                            crate::protocol::prepared::PresentationUnitKind::Control {
                                occurrence,
                            } => {
                                let allocation = controls.get(&occurrence).ok_or_else(|| {
                                    protocol_error(ProtocolError::MissingControl { occurrence })
                                })?;
                                control_baseline(&metrics, allocation.strut)?
                            }
                            _ => metrics.baseline,
                        },
                    ),
                    level,
                });
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let allocation = atomics
                    .get(&occurrence)
                    .copied()
                    .ok_or_else(|| invalid("selected atomic has no admitted allocation"))?;
                let (flow, geometry, alignment) = inflow_allocation(allocation)?;
                let baseline = match alignment {
                    LineBoxAlignment::Baseline => metrics.baseline,
                    LineBoxAlignment::Top => finite(
                        "atomic top baseline",
                        TextNumericPhase::Finalization,
                        metrics.block_min + flow.before_baseline,
                    )?,
                    LineBoxAlignment::Bottom => finite(
                        "atomic bottom baseline",
                        TextNumericPhase::Finalization,
                        metrics.block_max - flow.after_baseline,
                    )?,
                };
                let rect = translate_rect(geometry.border, pen, baseline)?;
                translate_rect(geometry.content, pen, baseline)?;
                add(
                    "atomic actual baseline",
                    TextNumericPhase::Finalization,
                    baseline,
                    geometry.baseline,
                )?;
                data.visual_order
                    .push(FinalizedVisualItem::InlineBox(data.boxes.len()));
                data.boxes.push(FinalizedBox {
                    occurrence,
                    atomic: allocation.occurrence.clone(),
                    slot: Point::new(pen, baseline),
                    geometry: FinalizedBoxGeometry::InFlow {
                        flow,
                        alignment,
                        box_geometry: geometry,
                        rect,
                    },
                });
                last_run = None;
                pen = add(
                    "atomic logical advance",
                    TextNumericPhase::Finalization,
                    pen,
                    flow.advance,
                )?;
            }
            ShapedItem::TextCluster(cluster_index) => {
                data.visual_order
                    .push(FinalizedVisualItem::Cluster(cluster_index));
                let cluster = &shaped.clusters[cluster_index];
                let cluster_x = pen;
                let discarded = discarded[cluster_index];
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
        && ((shaped.empty_run.is_some()
            && !selected.request.participants.iter().any(|participant| {
                matches!(participant, crate::protocol::PreparedParticipant::Atomic(_))
            }))
            || shaped
                .items
                .iter()
                .any(|item| matches!(item, ShapedItem::TextCluster(_))))
    {
        let run = shaped.empty_run.as_ref().or_else(|| shaped.runs.first());
        if let Some(run) = run {
            let source = selected.presentation.source;
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
    install_out_of_flow(&selected, &mut data)?;
    data.font_runs = shaped.runs;
    data.fragments = crate::protocol::prepared::selected_origin_intersections(&selected)
        .into_iter()
        .map(|association| FinalizedFragmentData {
            association,
            bounds: None,
        })
        .collect();
    let source_fragments: Vec<_> = data
        .fragments
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            if !matches!(
                row.association.owner,
                crate::protocol::paragraph::FragmentOwner::Origin(_)
            ) {
                return None;
            }
            row.association
                .source
                .filter(|source| !source.aggregate.is_empty())
                .map(|source| (source.aggregate, index))
        })
        .collect();
    let mut source_fragments = source_fragments;
    source_fragments.sort_unstable_by_key(|(range, index)| (range.start, range.end, *index));
    let mut maximum_end = 0;
    let source_fragments: Vec<_> = source_fragments
        .into_iter()
        .map(|(range, index)| {
            maximum_end = maximum_end.max(range.end);
            (range, index, maximum_end)
        })
        .collect();
    let mut owned_fragments: BTreeMap<_, Vec<usize>> = BTreeMap::new();
    for (index, row) in data.fragments.iter().enumerate() {
        owned_fragments
            .entry(fragment_key(row.association.owner))
            .or_default()
            .push(index);
    }
    for cluster in &data.clusters {
        let end = add(
            "fragment cluster end",
            TextNumericPhase::Finalization,
            cluster.x,
            cluster.advance,
        )?;
        let (top, height) = if selected.metric_policy == LineMetricPolicy::Native {
            let font = data.font_runs[cluster.run].metrics;
            let (before, _) = font_strut(font)?;
            (
                finite(
                    "source fragment strut top",
                    TextNumericPhase::Finalization,
                    metrics.baseline - before,
                )?,
                font.line_height,
            )
        } else {
            (metrics.block_min, metrics.block_max - metrics.block_min)
        };
        let rect = Rect::new(cluster.x.min(end), top, (end - cluster.x).abs(), height);
        add_fragment_bounds(&source_fragments, &mut data.fragments, cluster.source, rect);
        let first = selected
            .presentation
            .units
            .partition_point(|unit| unit.presentation.end <= cluster.presentation.start);
        for unit in &selected.presentation.units[first..] {
            if unit.presentation.start >= cluster.presentation.end {
                break;
            }
            if let crate::protocol::prepared::PresentationUnitKind::Replacement { alternative } =
                unit.kind
            {
                add_owned_fragment_bounds(
                    &owned_fragments,
                    &mut data.fragments,
                    crate::protocol::paragraph::FragmentOwner::Alternative(alternative),
                    rect,
                );
            }
        }
    }
    for anchor in &data.anchors {
        let unit = &selected.presentation.units[anchor.unit];
        let rect = Rect::new(
            anchor.slot.x,
            metrics.block_min,
            0.0,
            metrics.block_max - metrics.block_min,
        );
        use crate::protocol::{paragraph::FragmentOwner, prepared::PresentationUnitKind};
        match unit.kind {
            PresentationUnitKind::EmptyOrigin { origin } => {
                // An original empty occurrence owns an anchor, not a font strut.
                // Caret/AX geometry still comes from the actual line metrics.
                let bounds = if selected.metric_policy == LineMetricPolicy::Native {
                    Rect::new(anchor.slot.x, anchor.slot.y, 0.0, 0.0)
                } else {
                    rect
                };
                add_owned_fragment_bounds(
                    &owned_fragments,
                    &mut data.fragments,
                    FragmentOwner::Origin(origin),
                    bounds,
                );
            }
            PresentationUnitKind::Deleted => {
                add_fragment_bounds(&source_fragments, &mut data.fragments, unit.source, rect)
            }
            PresentationUnitKind::Control { occurrence } => add_owned_fragment_bounds(
                &owned_fragments,
                &mut data.fragments,
                FragmentOwner::Control(occurrence),
                rect,
            ),
            PresentationUnitKind::Replacement { alternative } => add_owned_fragment_bounds(
                &owned_fragments,
                &mut data.fragments,
                FragmentOwner::Alternative(alternative),
                rect,
            ),
            _ => {}
        }
    }
    let mut occupied_start = metrics.offset;
    let mut occupied_end = metrics.offset;
    for item in &data.visual_order {
        let (start, end) = match *item {
            FinalizedVisualItem::Cluster(index) => {
                let cluster = &data.clusters[index];
                (
                    cluster.x,
                    add(
                        "occupied cluster end",
                        TextNumericPhase::Finalization,
                        cluster.x,
                        cluster.advance,
                    )?,
                )
            }
            FinalizedVisualItem::Anchor(index) => {
                let item = &data.anchors[index];
                (item.slot.x, item.slot.x)
            }
            FinalizedVisualItem::InlineBox(index) => {
                let item = &data.boxes[index];
                let FinalizedBoxGeometry::InFlow { flow, .. } = item.geometry else {
                    return Err(invalid("out-of-flow slot entered L2 order"));
                };
                (
                    add(
                        "atomic occupied start",
                        TextNumericPhase::Finalization,
                        item.slot.x,
                        flow.occupied_start,
                    )?,
                    add(
                        "atomic occupied end",
                        TextNumericPhase::Finalization,
                        item.slot.x,
                        flow.occupied_end,
                    )?,
                )
            }
        };
        occupied_start = occupied_start.min(start.min(end));
        occupied_end = occupied_end.max(start.max(end));
    }
    data.metrics.occupied_start = occupied_start;
    data.metrics.occupied_end = occupied_end;
    Ok(FinalizedLine {
        selected,
        mapping,
        data,
    })
}

fn fragment_key(owner: crate::protocol::paragraph::FragmentOwner) -> (u8, usize) {
    use crate::protocol::paragraph::FragmentOwner;
    match owner {
        FragmentOwner::Origin(index) => (0, index),
        FragmentOwner::Control(index) => (1, index),
        FragmentOwner::Alternative(index) => (2, index),
    }
}

fn add_owned_fragment_bounds(
    index: &BTreeMap<(u8, usize), Vec<usize>>,
    fragments: &mut [FinalizedFragmentData],
    owner: crate::protocol::paragraph::FragmentOwner,
    rect: Rect,
) {
    if let Some(indices) = index.get(&fragment_key(owner)) {
        for &index in indices {
            let row = &mut fragments[index];
            row.bounds = Some(row.bounds.map_or(rect, |old| union(old, rect)));
        }
    }
}

fn add_fragment_bounds(
    index: &[(crate::Range, usize, usize)],
    fragments: &mut [FinalizedFragmentData],
    source: crate::Range,
    rect: Rect,
) {
    if source.is_empty() {
        return;
    }
    let first = index.partition_point(|(_, _, maximum_end)| *maximum_end <= source.start);
    for &(range, index, _) in &index[first..] {
        if range.start >= source.end {
            break;
        }
        if range.end > source.start {
            let row = &mut fragments[index];
            row.bounds = Some(row.bounds.map_or(rect, |old| union(old, rect)));
        }
    }
}

fn union(left: Rect, right: Rect) -> Rect {
    let x = left.origin.x.min(right.origin.x);
    let y = left.origin.y.min(right.origin.y);
    let end_x = (left.origin.x + left.size.width).max(right.origin.x + right.size.width);
    let end_y = (left.origin.y + left.size.height).max(right.origin.y + right.size.height);
    Rect::new(x, y, end_x - x, end_y - y)
}

fn translate_rect(rect: Rect, x: f64, y: f64) -> Result<Rect> {
    let x = add(
        "atomic inline origin",
        TextNumericPhase::Finalization,
        x,
        rect.origin.x,
    )?;
    let y = add(
        "atomic block origin",
        TextNumericPhase::Finalization,
        y,
        rect.origin.y,
    )?;
    add(
        "atomic inline extent",
        TextNumericPhase::Finalization,
        x,
        rect.size.width,
    )?;
    add(
        "atomic block extent",
        TextNumericPhase::Finalization,
        y,
        rect.size.height,
    )?;
    Ok(Rect::new(x, y, rect.size.width, rect.size.height))
}

fn inflow_allocation(
    allocation: &AtomicAllocation,
) -> Result<(
    crate::protocol::AtomicFlowAllocation,
    crate::protocol::AtomicBoxAllocation,
    LineBoxAlignment,
)> {
    match allocation.allocation {
        crate::protocol::AtomicLineAllocation::InFlow {
            flow,
            box_geometry,
            alignment,
        } => Ok((flow, box_geometry, alignment)),
        crate::protocol::AtomicLineAllocation::OutOfFlow => {
            Err(protocol_error(ProtocolError::InvalidAtomicGeometry {
                occurrence: allocation.occurrence.index,
            }))
        }
    }
}

fn install_out_of_flow(selected: &SelectedData, data: &mut FinalizedLineData) -> Result<()> {
    use crate::protocol::{
        AtomicLineAllocation, PreparedParticipant, prepared::PresentationUnitKind,
    };
    let units = &selected.presentation.units;
    let mut slots = vec![None; units.len()];
    for (index, unit) in units.iter().enumerate() {
        let first = 0;
        let mut first_cluster = None;
        let mut last_cluster = None;
        for cluster in &data.clusters[first..] {
            if cluster.presentation.end > unit.presentation.start
                && cluster.presentation.start < unit.presentation.end
                && !cluster.presentation.is_empty()
            {
                if first_cluster.is_none_or(|prior: &FinalizedCluster| {
                    cluster.presentation.start < prior.presentation.start
                }) {
                    first_cluster = Some(cluster);
                }
                if last_cluster.is_none_or(|prior: &FinalizedCluster| {
                    cluster.presentation.end > prior.presentation.end
                }) {
                    last_cluster = Some(cluster);
                }
            }
        }
        if let (Some(first), Some(last)) = (first_cluster, last_cluster) {
            slots[index] = Some((
                first.x
                    + if first.level & 1 != 0 {
                        first.advance
                    } else {
                        0.0
                    },
                last.x
                    + if last.level & 1 == 0 {
                        last.advance
                    } else {
                        0.0
                    },
            ));
        }
    }
    for anchor in &data.anchors {
        slots[anchor.unit] = Some((anchor.slot.x, anchor.slot.x));
    }
    for item in &data.boxes {
        let unit = units.iter().position(|unit| matches!(unit.kind,PresentationUnitKind::Atomic {occurrence} if occurrence == item.occurrence)).ok_or_else(|| invalid("final atomic has no actual selected unit"))?;
        let level = units[unit].level;
        let end = add(
            "atomic logical endpoint",
            TextNumericPhase::Finalization,
            item.slot.x,
            item.flow_advance(),
        )?;
        slots[unit] = Some(if level & 1 == 0 {
            (item.slot.x, end)
        } else {
            (end, item.slot.x)
        });
    }
    let mut before = vec![None; units.len()];
    let mut prior = None;
    for (index, slot) in slots.iter().enumerate() {
        before[index] = prior;
        if let Some((_, end)) = slot {
            prior = Some(*end);
        }
    }
    let mut after = vec![None; units.len()];
    let mut next = None;
    for (index, slot) in slots.iter().enumerate().rev() {
        after[index] = next;
        if let Some((start, _)) = slot {
            next = Some(*start);
        }
    }
    for (unit_index, unit) in units.iter().enumerate() {
        let PresentationUnitKind::Atomic { occurrence } = unit.kind else {
            continue;
        };
        let participant = unit
            .participant
            .and_then(|index| selected.request.participants.get(index))
            .ok_or_else(|| invalid("atomic unit has no selected participant"))?;
        let PreparedParticipant::Atomic(atomic) = participant else {
            return Err(invalid("atomic unit has wrong selected role"));
        };
        let allocation = selected
            .request
            .atomics
            .iter()
            .find(|allocation| allocation.occurrence == *atomic)
            .ok_or_else(|| {
                protocol_error(ProtocolError::MissingAtomic {
                    occurrence: atomic.index,
                })
            })?;
        if !matches!(allocation.allocation, AtomicLineAllocation::OutOfFlow) {
            continue;
        }
        let affinity = selected.prepared.tables.atomics[atomic.index].affinity;
        let x = match affinity {
            crate::Affinity::Before => before[unit_index].or(after[unit_index]),
            crate::Affinity::After => after[unit_index].or(before[unit_index]),
        }
        .unwrap_or(data.metrics.offset);
        data.boxes.push(FinalizedBox {
            occurrence,
            atomic: atomic.clone(),
            slot: Point::new(x, data.metrics.baseline),
            geometry: FinalizedBoxGeometry::OutOfFlow,
        });
    }
    Ok(())
}

fn control_allocations(selected: &SelectedData) -> Result<BTreeMap<usize, &ControlAllocation>> {
    selected
        .request
        .controls
        .iter()
        .map(|allocation| {
            let row = selected
                .prepared
                .tables
                .controls
                .get(allocation.occurrence.index)
                .ok_or_else(|| invalid("selected control has no prepared occurrence"))?;
            Ok((row.occurrence, allocation))
        })
        .collect()
}

/// The same metric-group algebra used by Layout's real box/control fitter.
/// A supplied strut is an allocation fact, never a manufactured font metric.
fn include_control_struts(
    metrics: &mut LineMetrics,
    struts: impl Iterator<Item = ControlStrut>,
    empty: Option<crate::protocol::LineStrut>,
) -> Result<()> {
    let mut baseline = metrics.baseline - metrics.block_min;
    let mut after = metrics.block_max - metrics.baseline;
    let mut top = 0.0_f64;
    let mut bottom = 0.0_f64;
    let mut any = false;
    if let Some(strut) = empty {
        any = true;
        baseline = baseline.max(strut.baseline);
        after = after.max(strut.line_extent - strut.baseline);
    }
    for strut in struts {
        any = true;
        match strut.alignment {
            LineBoxAlignment::Baseline => {
                baseline = baseline.max(strut.baseline);
                after = after.max(strut.line_extent - strut.baseline);
            }
            LineBoxAlignment::Top => top = top.max(strut.line_extent),
            LineBoxAlignment::Bottom => bottom = bottom.max(strut.line_extent),
        }
    }
    if !any {
        return Ok(());
    }
    let under = finite(
        "control line under extent",
        TextNumericPhase::Finalization,
        baseline + after,
    )?
    .max(top);
    let over = finite(
        "control line over extent",
        TextNumericPhase::Finalization,
        under - bottom,
    )?
    .min(0.0);
    let actual_baseline = finite(
        "control line baseline",
        TextNumericPhase::Finalization,
        baseline - over,
    )?;
    let actual_after = finite(
        "control line after baseline",
        TextNumericPhase::Finalization,
        under - baseline,
    )?;
    let extent = finite(
        "control line extent",
        TextNumericPhase::Finalization,
        actual_baseline + actual_after,
    )?;
    metrics.baseline = actual_baseline;
    metrics.ascent = actual_baseline;
    metrics.descent = actual_after;
    metrics.leading = 0.0;
    metrics.line_height = extent;
    metrics.block_min = 0.0;
    metrics.block_max = extent;
    Ok(())
}

fn control_baseline(metrics: &LineMetrics, strut: Option<ControlStrut>) -> Result<f64> {
    let Some(strut) = strut else {
        return Ok(metrics.baseline);
    };
    finite(
        "allocated control baseline",
        TextNumericPhase::Finalization,
        match strut.alignment {
            LineBoxAlignment::Baseline => metrics.baseline,
            LineBoxAlignment::Top => metrics.block_min + strut.baseline,
            LineBoxAlignment::Bottom => metrics.block_max - (strut.line_extent - strut.baseline),
        },
    )
}

fn atomic_allocations(selected: &SelectedData) -> Result<BTreeMap<usize, &AtomicAllocation>> {
    selected
        .request
        .atomics
        .iter()
        .map(|allocation| {
            let row = selected
                .prepared
                .tables
                .atomics
                .get(allocation.occurrence.index)
                .ok_or_else(|| invalid("selected atomic has no prepared occurrence"))?;
            Ok((row.occurrence, allocation))
        })
        .collect()
}

fn validate_rows(selected: &SelectedData, shaped: &ShapedData) -> Result<()> {
    for cluster in &shaped.clusters {
        crate::range::validate(
            selected.prepared.analyzed.processed.source().text(),
            cluster.source,
        )?;
        if cluster.run >= shaped.runs.len()
            || cluster.glyphs.start > cluster.glyphs.end
            || cluster.glyphs.end > shaped.glyphs.len()
            || cluster.atoms.start >= cluster.atoms.end
            || cluster.atoms.end > selected.presentation.analysis.infos.len()
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
        crate::range::validate(
            selected.prepared.analyzed.processed.source().text(),
            glyph.source,
        )?;
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
            ShapedItem::Anchor { unit, .. } if *unit >= selected.presentation.units.len() => {
                return Err(invalid("selected anchor has no actual presentation unit"));
            }
            ShapedItem::TextCluster(index) if *index >= shaped.clusters.len() => {
                return Err(invalid("selected text item has no cluster"));
            }
            ShapedItem::InlineBox { occurrence, .. }
                if *occurrence >= selected.prepared.analyzed.processed.source().boxes().len() =>
            {
                return Err(invalid("selected atomic item has no original occurrence"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn line_metrics(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    controls: &BTreeMap<usize, &ControlAllocation>,
) -> Result<LineMetrics> {
    let mut result = LineMetrics::default();
    let mut have_metrics = false;
    for item in &shaped.items {
        match *item {
            ShapedItem::Anchor { .. } => {}
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
                let flow = inflow_allocation(
                    atomics
                        .get(&occurrence)
                        .copied()
                        .ok_or_else(|| invalid("atomic metrics have no admitted allocation"))?,
                )?
                .0;
                result.advance = add(
                    "line atomic advance",
                    TextNumericPhase::Finalization,
                    result.advance,
                    flow.advance,
                )?;
                result.line_height = result.line_height.max(finite(
                    "atomic line height",
                    TextNumericPhase::Finalization,
                    flow.before_baseline + flow.after_baseline,
                )?);
            }
        }
    }
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::Anchor { .. } => continue,
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
    if selected.metric_policy == LineMetricPolicy::Native {
        return native_metrics(selected, shaped, atomics, controls, result);
    }
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::Anchor { .. } => {}
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
                let flow = inflow_allocation(
                    atomics
                        .get(&occurrence)
                        .copied()
                        .ok_or_else(|| invalid("atomic metrics have no admitted allocation"))?,
                )?
                .0;
                result.ascent = result.ascent.max(flow.before_baseline);
                result.descent = result.descent.max(flow.after_baseline);
                if selected.prepared.analyzed.processed.source().boxes()[occurrence].kind()
                    == crate::InlineBoxKind::InFlow
                {
                    have_metrics = true;
                }
            }
        }
    }
    let only_boxes =
        selected.request.participants.iter().any(|participant| {
            matches!(participant, crate::protocol::PreparedParticipant::Atomic(_))
        }) && !shaped
            .items
            .iter()
            .any(|item| matches!(item, ShapedItem::TextCluster(_)));
    if !have_metrics && !only_boxes {
        if let Some(font) = shaped.runs.first().or(shaped.empty_run.as_ref()) {
            result.ascent = font.metrics.ascent;
            result.descent = font.metrics.descent;
            result.line_height = result.line_height.max(font.metrics.line_height);
        } else if selected.request.line_strut.is_none()
            && !selected
                .request
                .controls
                .iter()
                .any(|allocation| allocation.strut.is_some())
        {
            return Err(protocol_error(ProtocolError::MissingLineMetrics));
        }
    }
    result.leading = result.line_height - (result.ascent + result.descent);
    let (ascent, descent) = if selected.metric_policy == LineMetricPolicy::StandaloneQuantized {
        (result.ascent.round(), result.descent.round())
    } else {
        (result.ascent, result.descent)
    };
    let leading = result.line_height - (ascent + descent);
    let (above, below) = if selected.metric_policy == LineMetricPolicy::StandaloneQuantized {
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

/// A native inline box contributes its own signed half-leading before it is
/// combined with any other font or allocated box. This is not paint ink.
fn font_strut(metrics: RunMetrics) -> Result<(f64, f64)> {
    let half = finite(
        "font half-leading",
        TextNumericPhase::Finalization,
        (metrics.line_height - (metrics.ascent + metrics.descent)) * 0.5,
    )?;
    Ok((
        finite(
            "font strut before",
            TextNumericPhase::Finalization,
            metrics.ascent + half,
        )?,
        finite(
            "font strut after",
            TextNumericPhase::Finalization,
            metrics.descent + half,
        )?,
    ))
}

#[derive(Default)]
struct MetricGroups {
    baseline: Option<(f64, f64)>,
    top: Option<f64>,
    bottom: Option<f64>,
}

impl MetricGroups {
    fn include(&mut self, before: f64, after: f64, alignment: LineBoxAlignment) -> Result<()> {
        finite(
            "metric group before",
            TextNumericPhase::Finalization,
            before,
        )?;
        finite("metric group after", TextNumericPhase::Finalization, after)?;
        match alignment {
            LineBoxAlignment::Baseline => {
                self.baseline = Some(self.baseline.map_or((before, after), |(above, below)| {
                    (above.max(before), below.max(after))
                }));
            }
            LineBoxAlignment::Top | LineBoxAlignment::Bottom => {
                let extent = finite(
                    "aligned metric group extent",
                    TextNumericPhase::Finalization,
                    before + after,
                )?;
                let target = if alignment == LineBoxAlignment::Top {
                    &mut self.top
                } else {
                    &mut self.bottom
                };
                *target = Some(target.map_or(extent, |old| old.max(extent)));
            }
        }
        Ok(())
    }

    fn finish(self, mut result: LineMetrics) -> Result<LineMetrics> {
        if self.baseline.is_none() && self.top.is_none() && self.bottom.is_none() {
            return Err(protocol_error(ProtocolError::MissingLineMetrics));
        }
        // With only top/bottom allocations, zero is the metric coordinate
        // origin, not a fabricated font or a new baseline contribution.
        let (before, after) = self.baseline.unwrap_or((0.0, 0.0));
        let under = finite(
            "native baseline group extent",
            TextNumericPhase::Finalization,
            before + after,
        )?;
        let under = self.top.map_or(under, |top| under.max(top));
        let over = match self.bottom {
            Some(bottom) => finite(
                "native bottom group offset",
                TextNumericPhase::Finalization,
                under - bottom,
            )?
            .min(0.0),
            None => 0.0,
        };
        let before = finite(
            "native line baseline",
            TextNumericPhase::Finalization,
            before - over,
        )?;
        let after = finite(
            "native line after",
            TextNumericPhase::Finalization,
            under - self.baseline.map_or(0.0, |group| group.0),
        )?;
        result.ascent = before;
        result.descent = after;
        result.leading = 0.0;
        result.baseline = before;
        result.line_height = finite(
            "native line height",
            TextNumericPhase::Finalization,
            before + after,
        )?
        .max(0.0);
        result.block_min = 0.0;
        result.block_max = result.line_height;
        Ok(result)
    }
}

fn native_metrics(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    controls: &BTreeMap<usize, &ControlAllocation>,
    result: LineMetrics,
) -> Result<LineMetrics> {
    let mut groups = MetricGroups::default();
    if selected.prepared.analyzed.paragraph.paragraph_strut == crate::ParagraphStrut::DefaultFont {
        let font = selected
            .prepared
            .shaped
            .empty_run
            .as_ref()
            .ok_or_else(|| protocol_error(ProtocolError::MissingLineMetrics))?;
        let (before, after) = font_strut(font.metrics)?;
        groups.include(before, after, LineBoxAlignment::Baseline)?;
    }
    for font in &shaped.runs {
        let (before, after) = font_strut(font.metrics)?;
        groups.include(before, after, LineBoxAlignment::Baseline)?;
    }
    for item in &shaped.items {
        if let ShapedItem::InlineBox { occurrence, .. } = *item {
            let (flow, _, alignment) = inflow_allocation(
                atomics
                    .get(&occurrence)
                    .copied()
                    .ok_or_else(|| invalid("native atomic has no admitted allocation"))?,
            )?;
            groups.include(flow.before_baseline, flow.after_baseline, alignment)?;
        }
    }
    for strut in controls.values().filter_map(|allocation| allocation.strut) {
        groups.include(
            strut.baseline,
            strut.line_extent - strut.baseline,
            strut.alignment,
        )?;
    }
    if let Some(strut) = selected.request.line_strut {
        groups.include(
            strut.baseline,
            strut.line_extent - strut.baseline,
            LineBoxAlignment::Baseline,
        )?;
    }
    groups.finish(result)
}

/// Applicable Collapse line-edge suppression. Invisible occurrence/control
/// anchors and out-of-flow boxes are transparent; real in-flow atomics stop it.
fn collapse_edges(selected: &SelectedData, shaped: &mut ShapedData) -> Result<Vec<bool>> {
    let mut projected = Vec::new();
    let mut clusters = Vec::new();
    for item in &shaped.items {
        match *item {
            ShapedItem::Anchor { .. } => {}
            ShapedItem::InlineBox { occurrence, level } => {
                if selected.prepared.analyzed.processed.source().boxes()[occurrence].kind()
                    == crate::InlineBoxKind::InFlow
                {
                    projected.push(SuppliedInlineParticipant::Atomic {
                        level: BidiLevel::try_new(level)
                            .map_err(|_| invalid("invalid atomic edge level"))?,
                    });
                    clusters.push(None);
                }
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                let mode = selected
                    .prepared
                    .analyzed
                    .styles
                    .get(cluster.style)
                    .ok_or_else(|| invalid("cluster has no captured processing style"))?
                    .style
                    .authored()
                    .white_space;
                let edge = if cluster.whitespace == Whitespace::Space
                    && mode == crate::WhiteSpace::Collapse
                {
                    InlineWhitespaceEdge::DiscardAtBoth
                } else {
                    InlineWhitespaceEdge::Preserve
                };
                projected.push(SuppliedInlineParticipant::Text {
                    level: BidiLevel::try_new(cluster.level)
                        .map_err(|_| invalid("invalid text edge level"))?,
                    edge,
                });
                clusters.push(Some(index));
            }
        }
    }
    let edges = SuppliedLine::new(&projected).edges();
    let mut discarded = vec![false; shaped.clusters.len()];
    for (ordinal, index) in clusters.into_iter().enumerate() {
        if let Some(index) = index
            && edges.is_discarded(ordinal) == Some(true)
        {
            discarded[index] = true;
            shaped.clusters[index].advance = 0.0;
        }
    }
    Ok(discarded)
}

fn apply_spacing(
    selected: &SelectedData,
    shaped: &mut ShapedData,
    amounts: &[f64],
    discarded: &[bool],
) -> Result<()> {
    if amounts.len() != selected.spacing_slots.len() {
        return Err(invalid("spacing amounts do not cover selected slots"));
    }
    let atoms = &selected.presentation.projection.atoms;
    let mut targets = BTreeMap::new();
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        let range = atoms
            .get(cluster.atoms.clone())
            .ok_or_else(|| invalid("spacing cluster has no selected atoms"))?;
        if let (Some(first), Some(last)) = (range.first(), range.last())
            && targets
                .insert((first.presentation.start, last.presentation.end), index)
                .is_some()
        {
            return Err(invalid(
                "multiple shaped clusters claim one presentation target",
            ));
        }
    }
    for (slot, (row, &amount)) in selected.spacing_slots.iter().zip(amounts).enumerate() {
        finite(
            "selected spacing amount",
            TextNumericPhase::Finalization,
            amount,
        )?;
        let Some(&index) = targets.get(&(row.presentation.start, row.presentation.end)) else {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        };
        if discarded[index] && amount != 0.0 {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        }
        let cluster = &mut shaped.clusters[index];
        if !cluster.whitespace.is_space_or_nbsp() {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        }
        cluster.advance = add(
            "selected spacing amount",
            TextNumericPhase::Finalization,
            cluster.advance,
            amount,
        )?;
    }
    Ok(())
}

/// The standalone policy produces the same explicit logical allocations as a
/// composed consumer; the core finalizer never performs implicit justification.
pub(super) fn standalone_amounts(
    selected: &SelectedData,
    metrics: &LineMetrics,
) -> Result<Vec<f64>> {
    let selection = &selected.request.context;
    let mut amounts = vec![0.0; selected.spacing_slots.len()];
    if selection.alignment != Alignment::Justify || selection.end != LineEnd::Soft {
        return Ok(amounts);
    }
    let width = selection.available_inline.unwrap_or_else(|| {
        (metrics.advance + selection.indent.max(0.0) - metrics.trailing_whitespace).max(0.0)
    });
    let free = width - selection.indent - metrics.advance + metrics.trailing_whitespace;
    if free <= 0.0 {
        return Ok(amounts);
    }
    let last = selected
        .presentation
        .units
        .last()
        .map(|unit| unit.presentation);
    let eligible: Vec<_> = selected
        .spacing_slots
        .iter()
        .enumerate()
        .filter(|(_, slot)| {
            last.is_none_or(|last| {
                slot.presentation.start != last.start || slot.presentation.end != last.end
            })
        })
        .map(|(index, _)| index)
        .collect();
    if eligible.is_empty() {
        return Ok(amounts);
    }
    let adjustment = free_space(selection, width, metrics)? / eligible.len() as f64;
    if adjustment == 0.0 {
        return Err(super::super::text_numeric_error(
            "justification adjustment",
            TextNumericPhase::Finalization,
            TextNumericError::PositiveLost,
            adjustment,
        ));
    }
    for index in eligible {
        amounts[index] = adjustment;
    }
    Ok(amounts)
}

fn alignment_offset(
    selection: &LineContext,
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
            (Alignment::Justify, true) if selection.end != LineEnd::Soft => 1.0,
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

fn free_space(selection: &LineContext, width: f64, metrics: &LineMetrics) -> Result<f64> {
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

fn line_levels(selected: &SelectedData, shaped: &ShapedData) -> Result<Vec<u8>> {
    let base = selected.presentation.analysis.base_level;
    let mut levels = Vec::with_capacity(shaped.items.len());
    let mut roles = Vec::with_capacity(shaped.items.len());
    for item in &shaped.items {
        match *item {
            ShapedItem::Anchor { unit, level } => {
                levels.push(level);
                roles.push(anchor_reset(selected, unit)?);
            }
            ShapedItem::InlineBox { level, .. } => {
                levels.push(level);
                roles.push(Reset::Other);
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                levels.push(cluster.level);
                let infos = selected
                    .presentation
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

fn anchor_reset(selected: &SelectedData, unit: usize) -> Result<Reset> {
    let unit = selected
        .presentation
        .units
        .get(unit)
        .ok_or_else(|| invalid("anchor has no selected semantic unit"))?;
    Ok(match unit.bidi_class {
        Some(BidiClass::ParagraphSeparator | BidiClass::SegmentSeparator) => Reset::Separator,
        Some(
            BidiClass::WhiteSpace
            | BidiClass::LeftToRightIsolate
            | BidiClass::RightToLeftIsolate
            | BidiClass::FirstStrongIsolate
            | BidiClass::PopDirectionalIsolate,
        ) => Reset::White,
        Some(
            BidiClass::LeftToRightEmbedding
            | BidiClass::RightToLeftEmbedding
            | BidiClass::LeftToRightOverride
            | BidiClass::RightToLeftOverride
            | BidiClass::PopDirectionalFormat
            | BidiClass::BoundaryNeutral,
        )
        | None => Reset::Removed,
        Some(_) => Reset::Other,
    })
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorCode::LayoutFailed, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::*;
    use crate::{
        FinalizedFragmentOwner, Font, FontCatalogueMode, Source, Style, System, SystemOptions,
        WhiteSpace,
    };
    use crate::{
        ParagraphAnchor, ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
        ParagraphControl, ParagraphControlKind, ParagraphOrigin, ParagraphOriginKind,
        ParagraphStrut,
    };

    fn selected(
        text: &str,
        mode: WhiteSpace,
        atomic: Option<crate::InlineBoxKind>,
        control: Option<ParagraphControlKind>,
    ) -> SelectedLine {
        selected_with_source(
            Source::new(text),
            Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                white_space: mode,
                ..Style::default()
            },
            atomic,
            control,
        )
    }

    fn selected_with_source(
        mut source: Source,
        style: Style,
        atomic: Option<crate::InlineBoxKind>,
        control: Option<ParagraphControlKind>,
    ) -> SelectedLine {
        let source_len = source.text().len();
        if let Some(kind) = atomic {
            source.inline_box(crate::InlineBox::new(
                crate::Id::from_u64(1),
                kind,
                0,
                crate::Size::new(5.0, 10.0),
            ));
        }
        let mut anchor_order = Vec::new();
        if source_len == 0 {
            anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::Before,
            });
        }
        if control.is_some() {
            anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Control(0),
                affinity: crate::Affinity::Before,
            });
        }
        if atomic.is_some() {
            anchor_order.push(ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: crate::Affinity::Before,
            });
        }
        let analyzed = crate::AnalyzedSource::from_paragraph(ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: crate::DocumentSegmentBreaks::Defined(Vec::new()),
            paragraph_strut: ParagraphStrut::DefaultFont,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![ParagraphOrigin {
                source,
                local: crate::Range::new(0, source_len),
                aggregate: crate::Range::new(0, source_len),
                kind: ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls: control
                .map(|kind| vec![ParagraphControl { at: 0, kind }])
                .unwrap_or_default(),
            boundary_policies: if atomic == Some(crate::InlineBoxKind::InFlow) {
                [
                    ParagraphBoundaryDefinition::AtomicBefore(0),
                    ParagraphBoundaryDefinition::AtomicAfter(0),
                ]
                .into_iter()
                .map(|definition| ParagraphBoundaryPolicy {
                    definition,
                    preceding_wrap: crate::Wrap::Word,
                    following_wrap: crate::Wrap::Word,
                    ancestor_wrap: crate::Wrap::Word,
                })
                .collect()
            } else {
                Vec::new()
            },
            replacements: Vec::new(),
            anchor_order,
        })
        .unwrap();
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let boundaries: Vec<_> = prepared
            .boundary_views()
            .map(|view| view.handle())
            .collect();
        let termination = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let atomics = prepared
            .atomic_views()
            .map(|view| {
                let inflow = atomic == Some(crate::InlineBoxKind::InFlow);
                AtomicAllocation {
                    occurrence: view.handle(),
                    allocation: if inflow {
                        AtomicLineAllocation::InFlow {
                            alignment: LineBoxAlignment::Baseline,
                            flow: AtomicFlowAllocation {
                                advance: 5.0,
                                before_baseline: 8.0,
                                after_baseline: 2.0,
                                occupied_start: 0.0,
                                occupied_end: 5.0,
                            },
                            box_geometry: AtomicBoxAllocation {
                                border: Rect::new(0.0, -8.0, 5.0, 10.0),
                                content: Rect::new(0.0, -8.0, 5.0, 10.0),
                                baseline: 0.0,
                            },
                        }
                    } else {
                        AtomicLineAllocation::OutOfFlow
                    },
                }
            })
            .collect();
        let controls = prepared
            .control_views()
            .map(|view| ControlAllocation {
                occurrence: view.handle(),
                strut: None,
            })
            .collect();
        prepared
            .select(LineSelectionRequest {
                start: boundaries[0].clone(),
                end: boundaries.last().unwrap().clone(),
                participants: prepared.participants().collect(),
                termination,
                alternative: None,
                context: LineContext {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,
                    hanging: HangingContext::None,
                    spacing: SpacingPolicy::Spaces,
                },
                atomics,
                controls,
                line_strut: None,
            })
            .unwrap()
    }

    fn ahem_strut(size: f32, line_height: f64) -> Style {
        Style {
            font: Font::new().family("Ahem"),
            size,
            line_height: crate::LineHeight::Absolute(line_height),
            ..Style::default()
        }
    }

    fn native_strut_line(
        line_height: f64,
        before: Option<f64>,
        control: Option<ControlStrut>,
    ) -> crate::protocol::FinalizedLine {
        native_aligned_strut_line(line_height, before, control, LineBoxAlignment::Baseline)
    }

    fn native_aligned_strut_line(
        line_height: f64,
        before: Option<f64>,
        control: Option<ControlStrut>,
        alignment: LineBoxAlignment,
    ) -> crate::protocol::FinalizedLine {
        let initial = selected_with_source(
            Source::new("X"),
            ahem_strut(16.0, line_height),
            before.map(|_| crate::InlineBoxKind::InFlow),
            control.map(|_| ParagraphControlKind::InlineBoundaryStart),
        );
        let mut request = initial.inner.request.clone();
        if let Some(before) = before {
            let AtomicLineAllocation::InFlow {
                flow,
                box_geometry,
                alignment: actual,
            } = &mut request.atomics[0].allocation
            else {
                panic!("the actual captured box is in-flow")
            };
            *actual = alignment;
            flow.before_baseline = before;
            flow.after_baseline = 0.0;
            box_geometry.border = Rect::new(0.0, -before, 5.0, before);
            box_geometry.content = box_geometry.border;
        }
        if let Some(strut) = control {
            request.controls[0].strut = Some(strut);
        }
        PreparedText {
            inner: Arc::clone(&initial.inner.prepared),
        }
        .select(request)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
    }

    fn near(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-10,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn native_font_strut_half_leading_survives_larger_atomic_and_fragment_bounds() {
        // Ahem16 has A=12.8,D=3.2. Authored line-height20 adds2 to each
        // side before combining with the independently allocated atomic.
        for (before, height, top) in [(16.0, 21.2, 1.2), (40.0, 45.2, 25.2)] {
            let line = native_strut_line(20.0, Some(before), None);
            near(line.feedback().baseline, before);
            near(line.feedback().line_height, height);
            near(line.feedback().advance, 21.0);
            let fragment = line
                .fragment_views()
                .find(|view| view.origin().is_some())
                .unwrap();
            let bounds = fragment
                .bounds()
                .expect("a real X has an allocated source slot");
            near(bounds.origin.x, 5.0);
            near(bounds.origin.y, top);
            near(bounds.size.width, 16.0);
            near(bounds.size.height, 20.0);
            near(line.run_views().next().unwrap().origin().y, before);
        }
    }

    #[test]
    fn native_font_strut_composes_with_actual_allocated_control_and_atomic() {
        let line = native_strut_line(
            20.0,
            Some(16.0),
            Some(ControlStrut {
                baseline: 18.0,
                line_extent: 21.0,
                alignment: LineBoxAlignment::Baseline,
            }),
        );
        // max(14.8,16,18) + max(5.2,0,3).
        near(line.feedback().baseline, 18.0);
        near(line.feedback().line_height, 23.2);
        let source = line
            .fragment_views()
            .find(|view| view.origin().is_some())
            .unwrap()
            .bounds()
            .unwrap();
        near(source.origin.y, 3.2);
        near(source.size.height, 20.0);
        assert!(
            line.fragment_views()
                .any(|view| matches!(view.owner(), FinalizedFragmentOwner::Control(_)))
        );
    }

    #[test]
    fn native_font_strut_combines_top_bottom_atomic_and_control_groups_once() {
        for (alignment, baseline, height, slot) in [
            (LineBoxAlignment::Top, 14.8, 20.0, 16.0),
            (LineBoxAlignment::Bottom, 14.8, 20.0, 20.0),
        ] {
            let line = native_aligned_strut_line(20.0, Some(16.0), None, alignment);
            near(line.feedback().baseline, baseline);
            near(line.feedback().line_height, height);
            near(line.atomic_slots().next().unwrap().origin.y, slot);
        }
        let combined = native_aligned_strut_line(
            20.0,
            Some(40.0),
            Some(ControlStrut {
                baseline: 30.0,
                line_extent: 50.0,
                alignment: LineBoxAlignment::Bottom,
            }),
            LineBoxAlignment::Top,
        );
        // max(font20,top40), followed by the independent bottom50 group.
        near(combined.feedback().baseline, 24.8);
        near(combined.feedback().line_height, 50.0);
        near(combined.atomic_slots().next().unwrap().origin.y, 40.0);
        let source = combined
            .fragment_views()
            .find(|view| view.origin().is_some())
            .unwrap()
            .bounds()
            .unwrap();
        near(source.origin.y, 10.0);
        near(source.size.height, 20.0);
    }

    #[test]
    fn native_font_strut_preserves_each_run_half_leading_and_signed_descent() {
        for range in [crate::Range::new(1, 2), crate::Range::new(0, 2)] {
            let mut source = Source::new("XX");
            source.span(range, ahem_strut(8.0, 20.0));
            let line = selected_with_source(source, ahem_strut(16.0, 20.0), None, None)
                .finalize(SpacingAllocations::None)
                .unwrap();
            // Ahem8 contributes12.4/7.6, while the actual default16px strut
            // still contributes14.8/5.2 even when every body span is8px.
            near(line.feedback().baseline, 14.8);
            near(line.feedback().line_height, 22.4);
        }
        let tight = native_strut_line(4.0, None, None);
        // Signed half-leading=-6 gives6.8/-2.8, not a zero-clamped descent.
        near(tight.feedback().baseline, 6.8);
        near(tight.feedback().line_height, 4.0);
        let bounds = tight
            .fragment_views()
            .find(|view| view.origin().is_some())
            .unwrap()
            .bounds()
            .unwrap();
        near(bounds.origin.y, 0.0);
        near(bounds.size.height, 4.0);
    }

    #[test]
    fn raw_standalone_keeps_characterized_global_leading_policy() {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        for (quantize, baseline) in [(false, 38.4), (true, 38.0)] {
            let mut source = Source::new("X");
            source.inline_box(crate::InlineBox::new(
                crate::Id::from_u64(9),
                crate::InlineBoxKind::InFlow,
                0,
                crate::Size::new(5.0, 40.0),
            ));
            let layout = system
                .layout(
                    source,
                    ahem_strut(16.0, 20.0),
                    crate::Options {
                        quantize,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            near(layout.run_views().next().unwrap().origin().y, baseline);
        }
    }

    #[test]
    fn native_allocated_negative_outer_box_keeps_zero_physical_line_and_real_overflow() {
        let mut source = Source::new("");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(41),
            crate::InlineBoxKind::InFlow,
            0,
            crate::Size::new(4.0, 4.0),
        ));
        let analyzed = crate::AnalyzedSource::from_paragraph(ParagraphInput {
            source,
            base_style: Style::default(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            paragraph_strut: crate::ParagraphStrut::AllocatedContributions,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: [
                ParagraphBoundaryDefinition::AtomicBefore(0),
                ParagraphBoundaryDefinition::AtomicAfter(0),
            ]
            .into_iter()
            .map(|definition| ParagraphBoundaryPolicy {
                definition,
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
            anchor_order: vec![ParagraphAnchor {
                item: ParagraphAnchorItem::Atomic(0),
                affinity: crate::Affinity::Before,
            }],
        })
        .unwrap();
        // No font is registered or requested: the sole actual allocation owns
        // all metrics, including its used negative outer margins.
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        let prepared = system
            .prepare(&analyzed, PreparationOptions::default())
            .unwrap();
        let occurrence = prepared.atomic_views().next().unwrap().handle();
        let termination = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let line = prepared
            .select(LineSelectionRequest {
                start: prepared.boundary_views().next().unwrap().handle(),
                end: prepared.boundary_views().last().unwrap().handle(),
                participants: prepared.participants().collect(),
                termination,
                alternative: None,
                atomics: vec![AtomicAllocation {
                    occurrence: occurrence.clone(),
                    allocation: AtomicLineAllocation::InFlow {
                        // Four-unit border box, internal baseline3, margins-10/-10:
                        // before=-10+3=-7; after=(4-3)-10=-9.
                        flow: AtomicFlowAllocation {
                            advance: 4.0,
                            before_baseline: -7.0,
                            after_baseline: -9.0,
                            occupied_start: 0.0,
                            occupied_end: 4.0,
                        },
                        box_geometry: AtomicBoxAllocation {
                            border: Rect::new(0.0, -3.0, 4.0, 4.0),
                            content: Rect::new(0.0, -3.0, 4.0, 4.0),
                            baseline: 0.0,
                        },
                        alignment: LineBoxAlignment::Baseline,
                    },
                }],
                controls: vec![],
                line_strut: None,
                context: LineContext {
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,
                    hanging: HangingContext::None,
                    spacing: SpacingPolicy::None,
                },
            })
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(line.feedback().line_height, 0.0);
        assert_eq!(line.feedback().baseline, -7.0);
        let slot = line.atomic_slots().next().unwrap();
        assert_eq!(slot.origin, Point::new(0.0, -7.0));
        let placed = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            lines: vec![LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 0.0,
                    baseline: -7.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![AtomicPlacement {
                    occurrence,
                    border: Rect::new(0.0, -10.0, 4.0, 4.0),
                    content: Rect::new(0.0, -10.0, 4.0, 4.0),
                    baseline: -7.0,
                }],
            }],
        })
        .unwrap();
        assert_eq!(placed.lines()[0].bounds().size.height, 0.0);
        assert_eq!(placed.bounds(), Rect::new(0.0, -10.0, 4.0, 10.0));
    }

    #[test]
    fn composed_collapse_suppresses_actual_line_edges_before_feedback_and_spacing() {
        let selected = selected("  X  ", WhiteSpace::Collapse, None, None);
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.feedback().trailing_whitespace, 0.0);
        assert_eq!(line.inner.data.glyphs.len(), 1);
        assert_eq!(
            line.inner.data.glyphs[0].glyph.source,
            crate::Range::new(2, 3)
        );
        assert_eq!(
            selected
                .inner
                .prepared
                .analyzed
                .processed
                .source()
                .text()
                .code_units(),
            "  X  ".encode_utf16().collect::<Vec<_>>()
        );
        let slots: Vec<_> = selected.spacing_slots().collect();
        assert_eq!(slots.len(), 2);
        let error = selected
            .finalize(SpacingAllocations::Complete(
                slots
                    .into_iter()
                    .enumerate()
                    .map(|(index, slot)| SpacingAllocation {
                        slot,
                        amount: if index == 0 { 1.0 } else { 0.0 },
                    })
                    .collect(),
            ))
            .unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::IneligibleSpacingSlot { slot: 0 }
            })
        ));
    }

    #[test]
    fn preserved_spaces_and_nbsp_keep_real_advances_at_selected_edges() {
        for (text, mode) in [
            (" X ", WhiteSpace::Preserve),
            ("\tX\t", WhiteSpace::PreserveSpaces),
            ("\u{a0}X\u{a0}", WhiteSpace::Collapse),
        ] {
            let line = selected(text, mode, None, None)
                .finalize(SpacingAllocations::None)
                .unwrap();
            assert_eq!(line.feedback().advance, 30.0);
            assert_eq!(line.inner.data.glyphs.len(), 3);
        }
    }

    #[test]
    fn neutral_controls_and_out_of_flow_slots_are_transparent_to_collapse_edges() {
        for atomic in [None, Some(crate::InlineBoxKind::OutOfFlow)] {
            let line = selected(
                " X ",
                WhiteSpace::Collapse,
                atomic,
                Some(ParagraphControlKind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                }),
            )
            .finalize(SpacingAllocations::None)
            .unwrap();
            assert_eq!(line.feedback().advance, 10.0);
            assert_eq!(line.inner.data.glyphs.len(), 1);
            assert_eq!(line.inner.data.runs[0].x, 0.0);
            assert_eq!(line.inner.data.anchors.len(), 1);
        }
        let line = selected(
            " X ",
            WhiteSpace::Collapse,
            Some(crate::InlineBoxKind::InFlow),
            Some(ParagraphControlKind::OptionalBreak {
                wrap: crate::Wrap::Word,
            }),
        )
        .finalize(SpacingAllocations::None)
        .unwrap();
        assert_eq!(line.feedback().advance, 25.0);
        assert_eq!(line.inner.data.glyphs.len(), 2);
        assert!(matches!(
            line.atomic_slots().next().unwrap().allocation,
            AtomicLineAllocation::InFlow {
                flow: AtomicFlowAllocation { advance: 5.0, .. },
                ..
            }
        ));
    }

    #[test]
    fn allocated_control_struts_use_real_baseline_top_and_bottom_metric_groups() {
        for (alignment, expected_baseline, expected_slot) in [
            (LineBoxAlignment::Baseline, 13.0, 13.0),
            (LineBoxAlignment::Top, 8.0, 13.0),
            (LineBoxAlignment::Bottom, 16.0, 13.0),
        ] {
            let mut metrics = LineMetrics {
                ascent: 8.0,
                descent: 2.0,
                line_height: 10.0,
                baseline: 8.0,
                block_min: 0.0,
                block_max: 10.0,
                ..LineMetrics::default()
            };
            let strut = ControlStrut {
                baseline: 13.0,
                line_extent: 18.0,
                alignment,
            };
            include_control_struts(&mut metrics, [strut].into_iter(), None).unwrap();
            assert_eq!(metrics.line_height, 18.0);
            assert_eq!(metrics.baseline, expected_baseline);
            assert_eq!(
                control_baseline(&metrics, Some(strut)).unwrap(),
                expected_slot
            );
            let initial = selected(
                "X",
                WhiteSpace::Preserve,
                None,
                Some(ParagraphControlKind::InlineBoundaryStart),
            );
            let mut request = initial.inner.request.clone();
            request.controls[0].strut = Some(strut);
            let prepared = PreparedText {
                inner: Arc::clone(&initial.inner.prepared),
            };
            let line = prepared
                .select(request)
                .unwrap()
                .finalize(SpacingAllocations::None)
                .unwrap();
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (10.0, expected_baseline, 18.0)
            );
            assert_eq!(line.inner.data.anchors[0].slot.y, expected_slot);
        }
        let mut metrics = LineMetrics {
            ascent: 8.0,
            descent: 2.0,
            line_height: 10.0,
            baseline: 8.0,
            block_min: 0.0,
            block_max: 10.0,
            ..LineMetrics::default()
        };
        include_control_struts(
            &mut metrics,
            [
                ControlStrut {
                    baseline: 13.0,
                    line_extent: 18.0,
                    alignment: LineBoxAlignment::Top,
                },
                ControlStrut {
                    baseline: 7.0,
                    line_extent: 24.0,
                    alignment: LineBoxAlignment::Bottom,
                },
            ]
            .into_iter(),
            None,
        )
        .unwrap();
        assert_eq!((metrics.baseline, metrics.line_height), (14.0, 24.0));
    }
}
