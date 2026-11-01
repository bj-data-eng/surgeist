// Copyright 2021 the Parley Authors
// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Selected-line metrics and alignment adapted from Parley 0.9
//! `layout/line_break.rs` (7793b3a6d1ef56f1c7551c4ec4595ece5a33e09cf92dddfa08d3be46a0e6f91b)
//! and `layout/alignment.rs`. Adjustments belong to this line, never preparation.

use std::sync::Arc;

use crate::{
    Alignment, BidiLevel, Error, ErrorCode, LineBoxDirection, Point, Rect, Result,
    SuppliedInlineParticipant, SuppliedLine, TextNumericError, TextNumericPhase,
};

use super::super::{
    FinalizedLine,
    shape::{self, ShapeContext},
};
use super::{add, data::*, finite};
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;
use crate::protocol::input::PresentationInput;
use crate::protocol::selection::{LineMetricPolicy, SelectedData};
use crate::protocol::{
    AtomicAllocation, ControlAllocation, ControlStrut, LineBoxAlignment, LineContext, LineEnd,
    ProtocolError, SpacingSlotDisposition, protocol_error,
};

mod phase_two;

/// Role-independent shaping and edge collapse shared with nonplaceable flow
/// observations. No hanging, spacing allocation, tabs or geometry receipt is produced here.
pub(super) fn shape_and_collapse(
    input: PresentationInput<'_>,
    scratch: &mut ShapeContext,
    allowance: Option<crate::ScopeAlignmentAllowance>,
    scope_input: super::super::scope_alignment::PlanInput<'_>,
) -> Result<(
    ShapedData,
    Arc<crate::protocol::prepared::SelectedMapping>,
    Vec<SpacingSlotDisposition>,
)> {
    let (mut shaped, mapping, mut scope_plan) = shape::finalize(
        input,
        scratch,
        crate::SpacingPolicy::None,
        allowance,
        scope_input,
    )?;
    validate_rows(input, &shaped)?;
    let dispositions = collapse_edges(input, &mut shaped)?;
    let ordered = crate::protocol::prepared::presentation::order_presentation(
        input.prepared,
        input.presentation,
        &shaped,
        &mut scope_plan.budget,
    )?;
    let levels = item_levels(input.presentation, &shaped);
    let participants = levels
        .into_iter()
        .map(|level| {
            BidiLevel::try_new(level)
                .map(|level| SuppliedInlineParticipant::Boundary { level })
                .map_err(|_| invalid("flow character level"))
        })
        .collect::<Result<Vec<_>>>()?;
    let order = SuppliedLine::new(&participants)
        .composed_order(&ordered)
        .ok_or_else(|| invalid("flow character order"))?;
    super::super::spacing::visual_authored(&mut shaped, &order, &dispositions)?;
    let character_input = super::super::spacing::character::Input::selected(
        input.prepared,
        input.presentation,
        &mut scope_plan.budget,
    )?;
    let graph = super::super::spacing::character::build(
        character_input,
        false,
        Some(&shaped.character_exclusions),
        &mut scope_plan.budget,
    )?;
    // Ordinary flow observation keeps its existing Native tracking owner;
    // an unrelated scope ledger does not adopt its character scratch.
    let mut native_character_budget = None;
    let application_budget = if shaped.cells.is_empty() {
        &mut native_character_budget
    } else {
        &mut scope_plan.budget
    };
    super::super::spacing::character::apply_selected_accounted(
        input,
        &graph,
        &mut shaped,
        &ordered,
        &dispositions,
        None,
        application_budget,
    )?;
    graph.drop_scratch(&mut scope_plan.budget)?;
    if shaped.cells.is_empty() {
        // Native ordering was deliberately not admitted by this scope ledger.
        drop(ordered);
    } else {
        account::drop_vec(&mut scope_plan.budget, ordered)?;
    }
    Ok((shaped, mapping, dispositions))
}

pub(super) fn finalize(selected: Arc<SelectedData>, amounts: &[f64]) -> Result<FinalizedLine> {
    let scopes = super::super::scope_alignment::allocation_frames(
        &selected,
        crate::ScopeAllocations::None,
        None,
        &crate::SpacingAllocations::None,
    )?;
    execute(selected, amounts, scopes)
}

pub(super) fn finalize_scoped(
    selected: Arc<SelectedData>,
    amounts: &[f64],
    scopes: super::super::scope_alignment::FrameAllocations,
) -> Result<FinalizedLine> {
    execute(selected, amounts, scopes)
}

pub(super) fn refinalize(
    line: &FinalizedLine,
    amounts: &[f64],
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<FinalizedLine> {
    execute(
        Arc::clone(&line.selected),
        amounts,
        super::super::scope_alignment::replay_frames(
            line,
            allowance,
            &crate::SpacingAllocations::None,
        )?,
    )
}

/// Row presence depends on actual glyph output and edge disposition, not on
/// allocated advances. Keep this predicate shared with the paint executor.
fn paints_cluster(cluster: &PreparedCluster, disposition: SpacingSlotDisposition) -> bool {
    disposition != SpacingSlotDisposition::DiscardedAtLineEdge
        && cluster.whitespace != Whitespace::Newline
        && !cluster.glyphs.is_empty()
}

fn metric_only_run<'a>(
    shaped: &'a ShapedData,
    participants: &[crate::PreparedParticipant],
    has_paint: bool,
) -> Option<&'a PreparedRun> {
    if has_paint
        || shaped
            .items
            .iter()
            .any(|item| matches!(item, ShapedItem::Cell(_)))
        || !((shaped.empty_run.is_some()
            && !participants
                .iter()
                .any(|participant| matches!(participant, crate::PreparedParticipant::Atomic(_))))
            || shaped
                .items
                .iter()
                .any(|item| matches!(item, ShapedItem::TextCluster(_))))
    {
        return None;
    }
    shaped.empty_run.as_ref().or_else(|| shaped.runs.first())
}

pub(super) fn metric_only_decoration_demand(
    run: &PreparedRun,
    shaped: &ShapedData,
    projection: &crate::projection::SourceProjection,
) -> bool {
    // A zero-atom font operation is genuine glyph-free demand. For a source
    // fallback, only complete exclusions in this actual presentation establish
    // bookkeeping; an eligible/Normal input keeps its demand at any advance.
    if run.atoms.is_empty() || projection.atoms.get(run.atoms.clone()).is_none() {
        return true;
    }
    run.atoms
        .clone()
        .any(|atom| !shaped.character_exclusions.contains_key(&atom))
}

pub(super) fn qualify_selected_paint(
    input: PresentationInput<'_>,
    shaped: &ShapedData,
    participants: &[crate::PreparedParticipant],
) -> Result<()> {
    if input.prepared.analyzed.typography != crate::AnalysisTypography::Mixed {
        return Ok(());
    }
    let dispositions = edge_dispositions(input, shaped)?;
    let has_paint = shaped.items.iter().any(|item| match *item {
        ShapedItem::TextCluster(index) => {
            paints_cluster(&shaped.clusters[index], dispositions[index])
        }
        _ => false,
    });
    if let Some(run) = metric_only_run(shaped, participants, has_paint)
        && run.metrics.horizontal().is_none()
        && metric_only_decoration_demand(run, shaped, &input.presentation.projection)
    {
        let owner = if input.prepared.default_style_index == Some(run.style) {
            crate::TextStyleOwner::Paragraph
        } else {
            let captured = input
                .prepared
                .analyzed
                .styles
                .get(run.style)
                .ok_or_else(|| invalid("metric-only paint has no captured style"))?;
            super::super::upright::style_owner(captured.owner)
        };
        let style = input
            .prepared
            .styles
            .get(run.style)
            .ok_or_else(|| invalid("metric-only paint has no resolved style"))?;
        super::super::upright::admit_resolved_decoration(&style.style, owner)?;
    }
    Ok(())
}

fn execute(
    selected: Arc<SelectedData>,
    amounts: &[f64],
    mut scopes: super::super::scope_alignment::FrameAllocations,
) -> Result<FinalizedLine> {
    let selection = &selected.request.context;
    let mut ledger = scopes.budget.take();
    account::header::<FinalizedLine>(&mut ledger)?;
    let atomics = atomic_allocations(&selected, &mut ledger)?;
    let controls = control_allocations(&selected, &mut ledger)?;
    let mut shaped = account::clone_shaped(&mut ledger, &selected.basis.shaped)?;
    validate_rows(selected.presentation_input(), &shaped)?;
    let mut dispositions =
        collapse_edges_accounted(selected.presentation_input(), &mut shaped, &mut ledger)?;
    let levels = item_levels_accounted(&selected.presentation, &shaped, &mut ledger)?;
    let base = selected.presentation.analysis.base_level;
    let boxed_execution = crate::protocol::inline_box::execution::build(
        &selected,
        &shaped,
        &selected.basis.order,
        &mut ledger,
    )?;
    let ordered = boxed_execution
        .as_ref()
        .map_or(selected.basis.order.as_slice(), |plan| {
            plan.order.as_slice()
        });
    let participants = account::try_collect(
        &mut ledger,
        levels.iter().map(|&level| {
            crate::BidiLevel::try_new(level)
                .map(|level| SuppliedInlineParticipant::Boundary { level })
                .map_err(|_| invalid("selected line has an invalid resolved level"))
        }),
    )?;
    let order = SuppliedLine::new(&participants)
        .composed_order_accounted(ordered, &mut ledger)?
        .ok_or_else(|| invalid("composed line order is not a complete permutation"))?;
    let mut word_halves = super::super::spacing::visual_authored_accounted(
        &mut shaped,
        &order,
        &dispositions,
        &mut ledger,
    )?;
    let character_order = account::collect(&mut ledger, order.iter().map(|(i, _)| i))?;
    super::super::spacing::character::apply_selected_accounted(
        selected.presentation_input(),
        &selected.basis.characters,
        &mut shaped,
        &character_order,
        &dispositions,
        None,
        &mut ledger,
    )?;
    resolve_tabs(&selected, &mut shaped, &atomics, &order, &mut ledger)?;
    let natural = phase_two::natural(
        &selected,
        &shaped,
        &atomics,
        &order,
        &dispositions,
        &mut ledger,
    )?;
    let responses =
        phase_two::spacing_responses(&selected, &shaped, &order, &natural, &mut ledger)?;
    let mut spacing_slots = apply_spacing(
        &selected,
        &mut shaped,
        amounts,
        &dispositions,
        &responses,
        &character_order,
        &mut ledger,
    )?;
    account::drop_vec(&mut ledger, responses)?;
    if matches!(
        selection.spacing,
        crate::SpacingPolicy::InterWord | crate::SpacingPolicy::InterCharacter
    ) {
        super::super::spacing::character::restore(&mut shaped);
        let next = super::super::spacing::visual_authored_accounted(
            &mut shaped,
            &order,
            &dispositions,
            &mut ledger,
        )?;
        account::drop_vec(&mut ledger, word_halves)?;
        word_halves = next;
        super::super::spacing::character::apply_selected_accounted(
            selected.presentation_input(),
            &selected.basis.characters,
            &mut shaped,
            &character_order,
            &dispositions,
            (selection.spacing == crate::SpacingPolicy::InterCharacter).then_some(amounts),
            &mut ledger,
        )?;
    }
    if amounts.iter().any(|amount| *amount != 0.0) {
        resolve_tabs(&selected, &mut shaped, &atomics, &order, &mut ledger)?;
    }
    let mut prefix_receipts = Vec::new();
    let mut metrics = line_metrics(
        &selected,
        &shaped,
        &atomics,
        &controls,
        &mut dispositions,
        &mut prefix_receipts,
        &mut ledger,
    )?;
    let prefix = crate::protocol::prefix::PrefixData::new(
        &selected,
        &shaped,
        &atomics,
        &controls,
        &dispositions,
        prefix_receipts,
        &mut ledger,
    )?;
    for row in &mut spacing_slots {
        if let Some(edge) = row.edge {
            let discarded = super::super::spacing::character::discarded(
                &selected.basis.characters,
                edge,
                &shaped,
                &dispositions,
                &mut ledger,
            )?;
            row.disposition = if discarded {
                SpacingSlotDisposition::DiscardedAtLineEdge
            } else {
                SpacingSlotDisposition::Retained { trailing: false }
            };
            row.advance = if discarded
                || matches!(
                    row.geometry,
                    crate::SpacingSlotGeometry::Boundary(
                        crate::SpacingBoundaryGeometry::ProtectedRequired
                    )
                ) {
                0.0
            } else {
                add(
                    "character slot gap",
                    TextNumericPhase::Finalization,
                    row.letter,
                    row.allocation,
                )?
            };
            continue;
        }
        row.disposition = dispositions[row.cluster];
        row.advance = row.members.iter().try_fold(0.0, |value, &index| {
            add(
                "spacing unit advance",
                TextNumericPhase::Finalization,
                value,
                shaped.clusters[index].advance,
            )
        })?;
    }
    if selected.metric_policy != LineMetricPolicy::Native {
        include_control_struts(
            &mut metrics,
            controls
                .values()
                .filter_map(|allocation| allocation.strut)
                .chain(
                    selected
                        .request
                        .line_box_struts
                        .iter()
                        .map(|allocation| ControlStrut {
                            baseline: allocation.metrics.baseline,
                            line_extent: allocation.metrics.line_extent,
                            alignment: allocation.alignment,
                        }),
                ),
            selected.request.line_strut,
            shaped.empty_run.is_some()
                || !shaped.runs.is_empty()
                || shaped
                    .items
                    .iter()
                    .any(|item| matches!(item, ShapedItem::InlineBox { .. })),
        )?;
    }
    // Captured domain context can retain feature accounting without selecting
    // any aligned occurrence. Only actual allocated frames demand reference
    // geometry (including an explicitly allocated zero contribution).
    let scope_cell_band = (!scopes.frames.is_empty()).then_some((
        metrics.baseline - metrics.block_min,
        metrics.block_max - metrics.baseline,
    ));
    if !scopes.frames.is_empty() {
        account::work(&mut ledger, selected.scopes.contributions.len())?;
        metrics = native_metrics_scoped(
            &selected,
            &shaped,
            &atomics,
            &controls,
            metrics,
            Some(&scopes),
        )?;
    }
    let measurements = phase_two::allocated(
        &selected,
        &shaped,
        &atomics,
        phase_two::MeasurementOrder {
            order: &order,
            box_edges: boxed_execution.as_ref(),
        },
        natural,
        &mut metrics,
        &mut ledger,
    )?;
    let width = selection
        .available_inline
        .unwrap_or_else(|| (measurements.alignment.advance() + selection.indent.max(0.0)).max(0.0));
    finite(
        "line available width",
        TextNumericPhase::Finalization,
        width,
    )?;
    metrics.inline_min = 0.0;
    metrics.inline_max = width;
    metrics.offset = {
        let mut measured = metrics;
        measured.advance = measurements.alignment.advance();
        measured.trailing_whitespace = 0.0;
        let offset = alignment_offset(selection, base, width, &measured)?;
        add(
            "native hanging alignment",
            TextNumericPhase::Finalization,
            offset,
            -metrics.hanging_start,
        )?
    };

    let mut data = FinalizedLineData {
        measurements: None,
        spacing_slots,
        spacing_amounts: account::collect(
            &mut ledger,
            amounts
                .iter()
                .map(|&amount| if amount == 0.0 { 0.0 } else { amount }),
        )?,
        prefix,
        clusters: account::try_collect(
            &mut ledger,
            shaped.clusters.iter().enumerate().map(
                |(index, cluster)| -> Result<FinalizedCluster> {
                    Ok(FinalizedCluster {
                        frame: FinalizedRunFrame::Line,
                        reference: scope_cell_band
                            .map(|_| {
                                scoped_reference(
                                    metrics.baseline,
                                    scopes.displacement(selected.scopes.cluster_owners[index]),
                                )
                            })
                            .transpose()?,
                        line_height: cluster.line_height,
                        shaped_advance: cluster.shaped_advance,
                        logical_adjustments: cluster.logical_adjustments,
                        provider_atoms: cluster.provider_group.clone(),
                        disposition: dispositions[index],
                        source: cluster.source,
                        presentation: {
                            let atoms =
                                &selected.presentation.projection.atoms[cluster.atoms.clone()];
                            crate::Range::new(
                                atoms[0].presentation.start,
                                atoms[atoms.len() - 1].presentation.end,
                            )
                        },
                        content: cluster.content,
                        #[cfg(feature = "text-accessibility")]
                        style: cluster.style,
                        x: 0.0,
                        advance: cluster.advance,
                        level: cluster.level,
                        boundary: cluster.boundary,
                        whitespace: cluster.whitespace,
                    })
                },
            ),
        )?,
        metrics,
        scope_cell_band,
        ..FinalizedLineData::default()
    };
    let mut pen = metrics.offset;
    let mut last_run = None;
    let groups = crate::protocol::boundary::groups_accounted(&selected, &shaped, &mut ledger)?;
    let mut atom_groups =
        account::filled(&mut ledger, selected.presentation.projection.atoms.len(), 0)?;
    for (index, group) in groups.iter().enumerate() {
        atom_groups[group.clone()].fill(index);
    }
    let endpoint_offsets = shared_endpoint_offsets(
        &selected,
        &shaped,
        &dispositions,
        &order,
        SharedEndpointGroups {
            ranges: &groups,
            atom_groups: &atom_groups,
        },
        boxed_execution
            .as_ref()
            .map(|plan| plan.boxed_units.as_slice()),
        &mut ledger,
    )?;
    let mut internal_edges = account::filled(&mut ledger, groups.len(), 0.0)?;
    let cell_gap_indices =
        super::super::spacing::character::cell_gap_indices(&shaped, &mut ledger)?;
    for (ordinal, _) in order.iter() {
        account::work(&mut ledger, 1)?;
        let level = levels[ordinal];
        match shaped.items[ordinal] {
            ShapedItem::Cell(cell) => {
                pen = append_cell(
                    &selected,
                    &shaped,
                    &scopes,
                    CellFlow {
                        index: cell,
                        gap: cell_gap_indices
                            .get(cell)
                            .copied()
                            .flatten()
                            .map(|index| &shaped.character_gaps[index]),
                    },
                    Point::new(pen, metrics.baseline),
                    &mut data,
                    &mut ledger,
                )?;
                last_run = None;
            }
            ShapedItem::Anchor { unit, .. } => {
                let boundary =
                    crate::protocol::boundary::facts(selected.presentation_input(), unit);
                let advance = boundary.advance();
                let executed = boundary.execute(pen, TextNumericPhase::Finalization)?;
                let inline_end = executed.flow.1;
                let component_edge = boundary
                    .allocation()
                    .is_some_and(|edge| edge.box_components().is_some());
                let endpoint_offset = endpoint_offsets
                    .as_ref()
                    .and_then(|geometry| geometry.offsets[unit]);
                let slot = if component_edge {
                    executed.strip.0
                } else if let Some(offset) = endpoint_offset {
                    add(
                        "shared unit endpoint origin",
                        TextNumericPhase::Finalization,
                        pen,
                        offset,
                    )?
                } else {
                    pen
                };
                let slot_end = if component_edge {
                    executed.strip.1
                } else if endpoint_offset.is_some() {
                    add(
                        "shared unit endpoint end",
                        TextNumericPhase::Finalization,
                        slot,
                        advance,
                    )?
                } else {
                    inline_end
                };
                let cut = selected.presentation.units[unit].atoms.start;
                let group = groups.partition_point(|group| group.end <= cut);
                if !boxed_execution
                    .as_ref()
                    .is_some_and(|plan| plan.boxed_units[unit])
                    && groups
                        .get(group)
                        .is_some_and(|group| group.start < cut && cut < group.end)
                {
                    internal_edges[group] = add(
                        "shared unit boundary advance",
                        TextNumericPhase::Finalization,
                        internal_edges[group],
                        advance,
                    )?;
                }
                account::push(
                    &mut ledger,
                    &mut data.visual_order,
                    FinalizedVisualItem::Anchor(data.anchors.len()),
                )?;
                let delta = unit_scope_displacement(&selected, &scopes, unit, &controls);
                let reference = scope_cell_band
                    .map(|_| scoped_reference(metrics.baseline, delta))
                    .transpose()?;
                let slot_y = match selected.presentation.units[unit].kind {
                    crate::protocol::prepared::PresentationUnitKind::Control { occurrence } => {
                        let allocation = controls.get(&occurrence).ok_or_else(|| {
                            protocol_error(ProtocolError::MissingControl { occurrence })
                        })?;
                        let mut local = metrics;
                        local.baseline = reference.unwrap_or(metrics.baseline);
                        control_baseline(&local, allocation.strut)?
                    }
                    _ => reference.unwrap_or(metrics.baseline),
                };
                account::push(
                    &mut ledger,
                    &mut data.anchors,
                    FinalizedAnchor {
                        reference: reference.map(|_| slot_y),
                        scope_displacement: crate::BaselineShift::try_new(delta)?,
                        unit,
                        slot: Point::new(slot, slot_y),
                        level,
                        geometry: if component_edge {
                            AnchorGeometry::BoxEdge {
                                flow_start: executed.flow.0,
                                flow_end: executed.flow.1,
                                strip_end: slot_end,
                            }
                        } else {
                            AnchorGeometry::Ordinary {
                                inline_end: slot_end,
                            }
                        },
                    },
                )?;
                pen = inline_end;
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let allocation = atomics
                    .get(&occurrence)
                    .copied()
                    .ok_or_else(|| invalid("selected atomic has no admitted allocation"))?;
                let (flow, geometry, alignment, fit) = inflow_allocation(allocation)?;
                let delta = atomic_scope_displacement(
                    &selected,
                    &scopes,
                    occurrence,
                    alignment,
                    &mut ledger,
                )?;
                let reference = scoped_reference(metrics.baseline, delta)?;
                let baseline = match alignment {
                    LineBoxAlignment::Baseline => reference,
                    LineBoxAlignment::Shift(shift) => shifted_baseline(reference, shift)?,
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
                let gap = shaped.atomic_gaps.get(&occurrence).copied().unwrap_or(0.0);
                if gap != 0.0 {
                    let before = add(
                        "atomic pre-gap endpoint",
                        TextNumericPhase::Finalization,
                        pen,
                        flow.advance,
                    )?;
                    let after = add(
                        "atomic cell endpoint",
                        TextNumericPhase::Finalization,
                        pen,
                        super::super::spacing::character::atomic_advance(
                            &shaped,
                            occurrence,
                            flow.advance,
                        )?,
                    )?;
                    if before == after {
                        return Err(super::super::text_numeric_error(
                            "atomic boundary displacement",
                            TextNumericPhase::Finalization,
                            TextNumericError::NonZeroLost,
                            gap,
                        ));
                    }
                    account::push(
                        &mut ledger,
                        &mut data.character_anchors,
                        SpacingDisplacement {
                            frame: FinalizedRunFrame::Line,
                            before: Point::new(before, baseline),
                            after: Point::new(after, baseline),
                            amount: gap,
                        },
                    )?;
                }
                let rect = translate_rect(geometry.border, pen, baseline)?;
                translate_rect(geometry.content, pen, baseline)?;
                if let LineBoxAlignment::Shift(shift) = alignment {
                    for local in [
                        geometry.border.origin.y,
                        geometry.border.origin.y + geometry.border.size.height,
                        geometry.content.origin.y,
                        geometry.content.origin.y + geometry.content.size.height,
                        geometry.baseline,
                    ] {
                        qualify_local_shift(reference, baseline, local, shift)?;
                    }
                }
                add(
                    "atomic actual baseline",
                    TextNumericPhase::Finalization,
                    baseline,
                    geometry.baseline,
                )?;
                account::push(
                    &mut ledger,
                    &mut data.visual_order,
                    FinalizedVisualItem::InlineBox(data.boxes.len()),
                )?;
                account::push(
                    &mut ledger,
                    &mut data.boxes,
                    FinalizedBox {
                        scope_displacement: crate::BaselineShift::try_new(delta)?,
                        occurrence,
                        atomic: allocation.occurrence.clone(),
                        slot: Point::new(pen, baseline),
                        following_gap: shaped.atomic_gaps.get(&occurrence).copied().unwrap_or(0.0),
                        geometry: FinalizedBoxGeometry::InFlow {
                            flow,
                            fit,
                            alignment,
                            box_geometry: geometry,
                            rect,
                        },
                    },
                )?;
                last_run = None;
                pen = add(
                    "atomic logical advance",
                    TextNumericPhase::Finalization,
                    pen,
                    super::super::spacing::character::atomic_advance(
                        &shaped,
                        occurrence,
                        flow.advance,
                    )?,
                )?;
            }
            ShapedItem::TextCluster(cluster_index) => {
                account::push(
                    &mut ledger,
                    &mut data.visual_order,
                    FinalizedVisualItem::Cluster(cluster_index),
                )?;
                let cluster = &shaped.clusters[cluster_index];
                let group = atom_groups[cluster.atoms.start];
                let mut cluster_x = add(
                    "shared unit glyph origin",
                    TextNumericPhase::Finalization,
                    pen,
                    -internal_edges[group],
                )?;
                if let Some(geometry) = &endpoint_offsets
                    && geometry.leading[group] != 0.0
                {
                    cluster_x = add(
                        "shared unit leading endpoint inset",
                        TextNumericPhase::Finalization,
                        cluster_x,
                        geometry.leading[group],
                    )?;
                }
                let discarded =
                    dispositions[cluster_index] == SpacingSlotDisposition::DiscardedAtLineEdge;
                let word_half = word_halves[cluster_index].half;
                if !discarded && word_halves[cluster_index].unit_start && word_half != 0.0 {
                    let after = add(
                        "word spacing unit half",
                        TextNumericPhase::Finalization,
                        cluster_x,
                        word_half,
                    )?;
                    account::push(
                        &mut ledger,
                        &mut data.word_anchors,
                        SpacingDisplacement {
                            frame: FinalizedRunFrame::Line,
                            before: Point::new(
                                cluster_x,
                                data.clusters[cluster_index]
                                    .reference
                                    .unwrap_or(metrics.baseline),
                            ),
                            after: Point::new(
                                after,
                                data.clusters[cluster_index]
                                    .reference
                                    .unwrap_or(metrics.baseline),
                            ),
                            amount: word_half,
                        },
                    )?;
                }
                // RTL ligature/component shares are visited before their logical
                // start. The one actual glyph belongs at the group's visual
                // origin, not after those already-positioned component shares.
                let component_shift = if level & 1 != 0 && cluster.ligature_start {
                    shaped.clusters[cluster_index + 1..]
                        .iter()
                        .take_while(|next| {
                            next.ligature_component && next.content == cluster.content
                        })
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
                if paints_cluster(cluster, dispositions[cluster_index]) {
                    for glyph_index in cluster.glyphs.clone() {
                        let glyph = &shaped.glyphs[glyph_index];
                        let reference = scope_cell_band
                            .map(|_| {
                                scoped_reference(
                                    metrics.baseline,
                                    scopes.displacement(selected.scopes.glyph_owners[glyph_index]),
                                )
                            })
                            .transpose()?;
                        let continue_run = last_run.filter(|&run_index| {
                            let run: &FinalizedRun = &data.runs[run_index];
                            cluster.content.font_run().is_some_and(|font| {
                                run.run == font
                                    || (selected.prepared.options.tabs()
                                        == crate::TabPreparation::CssStops
                                        && same_paint_font(
                                            &shaped.runs[run.run],
                                            &shaped.runs[font],
                                        ))
                            }) && run.style == glyph.style
                                && run.level == level
                                && run.reference == reference
                        });
                        let run_index = if let Some(index) = continue_run {
                            index
                        } else {
                            let index = data.runs.len();
                            account::push(
                                &mut ledger,
                                &mut data.runs,
                                FinalizedRun {
                                    frame: FinalizedRunFrame::Line,
                                    reference,
                                    run: cluster
                                        .content
                                        .font_run()
                                        .expect("provider glyph owns its font"),
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
                                    y: add(
                                        "paint font origin",
                                        TextNumericPhase::Finalization,
                                        reference.unwrap_or(metrics.baseline),
                                        shaped.runs[cluster
                                            .content
                                            .font_run()
                                            .expect("provider glyph owns its font")]
                                        .metrics
                                        .font_origin_offset(),
                                    )?,
                                    advance: 0.0,
                                },
                            )?;
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
                        account::push(
                            &mut ledger,
                            &mut data.glyphs,
                            FinalizedGlyph {
                                glyph: *glyph,
                                pen: add(
                                    "word spacing glyph half",
                                    TextNumericPhase::Finalization,
                                    actual_pen,
                                    word_half,
                                )?,
                                pen_base: actual_pen,
                                word_half,
                            },
                        )?;
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
                output.advance = placed_cluster_advance(cluster, dispositions[cluster_index]);
                if let Some(index) = last_run {
                    let run = &mut data.runs[index];
                    if cluster.content.font_run().is_some_and(|font| {
                        run.run == font
                            || (selected.prepared.options.tabs() == crate::TabPreparation::CssStops
                                && same_paint_font(&shaped.runs[run.run], &shaped.runs[font]))
                    }) && run.level == level
                    {
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
                if let Some(gap) = shaped.character_gaps.iter().find(|g| matches!(g.owner, super::super::spacing::character::Owner::Cluster(i) if i == cluster_index)) && gap.amount != 0.0 {
                    let before = add("character pre-gap endpoint", TextNumericPhase::Finalization, pen, gap.base)?;
                    let after = add("character cell endpoint", TextNumericPhase::Finalization, pen, output.advance)?;
                    if before == after { return Err(super::super::text_numeric_error("character boundary displacement", TextNumericPhase::Finalization, TextNumericError::NonZeroLost, gap.amount)); }
                    account::push(&mut ledger,&mut data.character_anchors,SpacingDisplacement { frame: FinalizedRunFrame::Line, before: Point::new(before, output.reference.unwrap_or(metrics.baseline)), after: Point::new(after, output.reference.unwrap_or(metrics.baseline)), amount: gap.amount })?;
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
    account::drop_vec(&mut ledger, cell_gap_indices)?;
    // Zero-glyph controls and actual empty lines still retain a real font/style
    // row, rather than publishing a fabricated source space or fallback range.
    if let Some(run) = metric_only_run(
        &shaped,
        &selected.request.participants,
        !data.runs.is_empty(),
    ) {
        let source = selected.presentation.source;
        let run_index = if let Some(index) = shaped.runs.iter().position(|r| std::ptr::eq(r, run)) {
            index
        } else {
            let run = account::clone_run(&mut ledger, run)?;
            account::push(&mut ledger, &mut shaped.runs, run)?;
            shaped.runs.len() - 1
        };
        account::push(
            &mut ledger,
            &mut data.runs,
            FinalizedRun {
                frame: FinalizedRunFrame::Line,
                reference: scope_cell_band.map(|_| metrics.baseline),
                run: run_index,
                style: shaped.runs[run_index].style,
                level: base,
                source,
                glyphs: 0..0,
                x: metrics.offset,
                y: add(
                    "empty paint font origin",
                    TextNumericPhase::Finalization,
                    metrics.baseline,
                    shaped.runs[run_index].metrics.font_origin_offset(),
                )?,
                advance: 0.0,
            },
        )?;
    }
    install_out_of_flow(&selected, &mut data, &scopes, &mut ledger)?;
    data.font_runs = std::mem::take(&mut shaped.runs);
    account::work(&mut ledger, selected.presentation.units.len())?;
    crate::protocol::prepared::visit_selected_origin_intersections(&selected, |association| {
        account::push(
            &mut ledger,
            &mut data.fragments,
            FinalizedFragmentData {
                association,
                bounds: None,
            },
        )
    })?;
    let mut source_fragments = account::collect(
        &mut ledger,
        data.fragments
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
                    .map(|source| (source.aggregate, index, 0))
            }),
    )?;
    source_fragments.sort_unstable_by_key(|(range, index, _)| (range.start, range.end, *index));
    let mut maximum_end = 0;
    for (range, _, end) in &mut source_fragments {
        maximum_end = maximum_end.max(range.end);
        *end = maximum_end;
    }
    let mut owned_fragments = account::collect(
        &mut ledger,
        data.fragments
            .iter()
            .enumerate()
            .map(|(index, row)| (fragment_key(row.association.owner), index)),
    )?;
    operations::order_by(&mut owned_fragments, ledger.as_mut(), |left, right| {
        left.cmp(right)
    })?;
    for cluster in &data.clusters {
        account::work(&mut ledger, 1)?;
        if matches!(cluster.frame, FinalizedRunFrame::Cell(_)) {
            continue;
        }
        let end = add(
            "fragment cluster end",
            TextNumericPhase::Finalization,
            cluster.x,
            cluster.advance,
        )?;
        let (top, height) = if selected.metric_policy == LineMetricPolicy::Native {
            if let Some(run) = cluster.content.font_run() {
                let font = RunMetrics {
                    line_height: cluster.line_height,
                    ..data.font_runs[run].metrics
                };
                let (before, _) = font_strut(font)?;
                (
                    finite(
                        "source fragment strut top",
                        TextNumericPhase::Finalization,
                        cluster.reference.unwrap_or(metrics.baseline) - before,
                    )?,
                    font.line_height,
                )
            } else {
                (cluster.reference.unwrap_or(metrics.baseline), 0.0)
            }
        } else {
            data.cell_band(cluster.reference)
        };
        let rect = Rect::new(cluster.x.min(end), top, (end - cluster.x).abs(), height);
        add_fragment_bounds(
            &source_fragments,
            &mut data.fragments,
            cluster.source,
            rect,
            &mut ledger,
        )?;
        let first = selected
            .presentation
            .units
            .partition_point(|unit| unit.presentation.end <= cluster.presentation.start);
        for unit in &selected.presentation.units[first..] {
            if unit.presentation.start >= cluster.presentation.end {
                break;
            }
            if let crate::protocol::prepared::PresentationUnitKind::GeneratedSeparator { owner } =
                unit.kind
            {
                let owner = match owner {
                    crate::transform::relation::GeneratedSeparatorOwner::Control(index) => {
                        crate::protocol::paragraph::FragmentOwner::Control(index)
                    }
                    crate::transform::relation::GeneratedSeparatorOwner::Virtual(index) => {
                        crate::protocol::paragraph::FragmentOwner::VirtualSeparator(index)
                    }
                };
                add_owned_fragment_bounds(
                    &owned_fragments,
                    &mut data.fragments,
                    owner,
                    rect,
                    &mut ledger,
                )?;
            }
            if let crate::protocol::prepared::PresentationUnitKind::Replacement {
                alternative,
                ..
            } = unit.kind
            {
                add_owned_fragment_bounds(
                    &owned_fragments,
                    &mut data.fragments,
                    crate::protocol::paragraph::FragmentOwner::Alternative(alternative),
                    rect,
                    &mut ledger,
                )?;
            }
        }
    }
    for cell in &data.cells {
        let output = selected
            .prepared
            .analyzed
            .combination
            .as_deref()
            .ok_or_else(|| invalid("finalized cell has no source operation"))?;
        let binding = &output.bindings[cell.binding];
        let group = &output.plan.groups[binding.group];
        let rect = Rect::new(
            cell.inline.start,
            cell.under.start,
            cell.inline.end - cell.inline.start,
            cell.under.end - cell.under.start,
        );
        for part in &output.plan.parts[group.parts.clone()] {
            account::work(&mut ledger, 1)?;
            for piece in &part.pieces {
                add_fragment_bounds(
                    &source_fragments,
                    &mut data.fragments,
                    piece.aggregate,
                    rect,
                    &mut ledger,
                )?;
            }
        }
        use crate::transform::relation::{ComponentOwner, RelationEvent};
        for event in &selected.prepared.analyzed.transform.events()[binding.events.clone()] {
            account::work(&mut ledger, 1)?;
            let component_owner = |owner| match owner {
                ComponentOwner::Wbr(control) => {
                    Some(crate::protocol::paragraph::FragmentOwner::Control(control))
                }
                ComponentOwner::Virtual(index) => Some(
                    crate::protocol::paragraph::FragmentOwner::VirtualSeparator(index),
                ),
                ComponentOwner::Original(_) => None,
            };
            let owner = match *event {
                RelationEvent::Text { atom } => component_owner(
                    selected
                        .prepared
                        .analyzed
                        .transform
                        .component_for_atom(atom)
                        .owner,
                ),
                RelationEvent::GeneratedSuppression { component, .. } => component_owner(
                    selected.prepared.analyzed.transform.components[component].owner,
                ),
                RelationEvent::Anchor { order } => {
                    match selected.prepared.analyzed.paragraph.anchor_order[order].item {
                        crate::protocol::paragraph::SemanticAnchorItem::Control(control) => {
                            Some(crate::protocol::paragraph::FragmentOwner::Control(control))
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            if let Some(owner) = owner {
                add_owned_fragment_bounds(
                    &owned_fragments,
                    &mut data.fragments,
                    owner,
                    rect,
                    &mut ledger,
                )?;
            }
        }
    }
    for anchor in &data.anchors {
        let unit = &selected.presentation.units[anchor.unit];
        let rect = data.cell_rect(
            anchor.slot.x,
            anchor.strip_interval().1 - anchor.strip_interval().0,
            anchor.reference,
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
                    &mut ledger,
                )?;
            }
            PresentationUnitKind::Deleted => add_fragment_bounds(
                &source_fragments,
                &mut data.fragments,
                unit.source,
                rect,
                &mut ledger,
            )?,
            PresentationUnitKind::Control { occurrence } => add_owned_fragment_bounds(
                &owned_fragments,
                &mut data.fragments,
                FragmentOwner::Control(occurrence),
                rect,
                &mut ledger,
            )?,
            PresentationUnitKind::Replacement { alternative, .. } => add_owned_fragment_bounds(
                &owned_fragments,
                &mut data.fragments,
                FragmentOwner::Alternative(alternative),
                rect,
                &mut ledger,
            )?,
            _ => {}
        }
    }
    let mut occupied_start = metrics.offset;
    let mut occupied_end = metrics.offset;
    for item in &data.visual_order {
        let (start, end) = match *item {
            FinalizedVisualItem::Cell(index) => {
                let cell = &data.cells[index];
                (cell.inline.start, cell.inline.end)
            }
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
                item.strip_interval()
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
    // Every consumer lends these same immutable local references. This separate
    // publication pass charges one work unit per translated-band validation;
    // output-row creation has already charged its bounded frame calculation.
    if let Some((before, after)) = data.scope_cell_band {
        for reference in data
            .clusters
            .iter()
            .filter_map(|row| row.reference)
            .chain(data.anchors.iter().filter_map(|row| row.reference))
        {
            account::work(&mut ledger, 1)?;
            add(
                "scope source-cell top",
                TextNumericPhase::Finalization,
                reference,
                -before,
            )?;
            add(
                "scope source-cell bottom",
                TextNumericPhase::Finalization,
                reference,
                after,
            )?;
            finite(
                "scope source-cell extent",
                TextNumericPhase::Finalization,
                before + after,
            )?;
        }
    }
    if !data.cells.is_empty() {
        for cluster in &data.clusters {
            account::work(&mut ledger, 1)?;
            cell_geometry::qualify_cluster_rect(
                &data,
                cluster,
                selected.request.context.orientation,
                None,
                TextNumericPhase::Finalization,
            )?;
        }
        for cell in &data.cells {
            account::work(&mut ledger, 1)?;
            cell_geometry::qualify_edge_rect(cell, &data.metrics, TextNumericPhase::Finalization)?;
            account::work(&mut ledger, cell.tabs.len())?;
            cell_geometry::qualify_tabs(
                cell,
                selected.request.context.orientation,
                None,
                TextNumericPhase::Finalization,
            )?;
        }
    }
    account::drop_vec(&mut ledger, source_fragments)?;
    account::drop_vec(&mut ledger, owned_fragments)?;
    account::drop_vec(&mut ledger, groups)?;
    account::drop_vec(&mut ledger, atom_groups)?;
    account::drop_vec(&mut ledger, internal_edges)?;
    if let Some(geometry) = endpoint_offsets {
        account::drop_vec(&mut ledger, geometry.offsets)?;
        account::drop_vec(&mut ledger, geometry.leading)?;
    }
    if let Some(plan) = boxed_execution {
        account::drop_vec(&mut ledger, plan.order)?;
        account::drop_vec(&mut ledger, plan.boxed_units)?;
    }
    account::drop_vec(&mut ledger, character_order)?;
    account::drop_vec(&mut ledger, word_halves)?;
    account::drop_vec(&mut ledger, dispositions)?;
    account::drop_vec(&mut ledger, levels)?;
    account::drop_vec(&mut ledger, participants)?;
    data.measurements = Some(measurements.seal(&data.cells, shaped.cells.len(), &mut ledger)?);
    account::drop_shaped(&mut ledger, shaped)?;
    account::drop_vec(&mut ledger, atomics.rows)?;
    account::drop_vec(&mut ledger, controls.rows)?;
    scopes.budget = ledger;
    super::super::scope_alignment::finish_frames(&selected, &mut data, scopes)?;
    Ok(FinalizedLine { selected, data })
}

/// A genuine selected Cell paired with its canonical exterior gap receipt.
struct CellFlow<'a> {
    index: usize,
    gap: Option<&'a super::super::spacing::character::Gap>,
}

/// The required half must survive the actual inner paint mapping, including
/// compression and centering. Glyph-free units use the same baseline frame.
fn qualify_cell_word_half(
    frame: crate::GlyphFrame,
    pen_base: f64,
    pen: f64,
    offset: Point,
    half: f64,
) -> Result<()> {
    if half == 0.0 {
        return Ok(());
    }
    let scaled = finite(
        "cell word spacing scaled half",
        TextNumericPhase::Finalization,
        frame.outline_x().x * half,
    )?;
    let before = frame.point(pen_base, offset.x, offset.y);
    let after = frame.point(pen, offset.x, offset.y);
    for point in [before, after] {
        finite(
            "cell word spacing x",
            TextNumericPhase::Finalization,
            point.x,
        )?;
        finite(
            "cell word spacing y",
            TextNumericPhase::Finalization,
            point.y,
        )?;
    }
    if scaled == 0.0 || before.x == after.x {
        return Err(super::super::text_numeric_error(
            "cell word spacing displacement",
            TextNumericPhase::Finalization,
            TextNumericError::NonZeroLost,
            half,
        ));
    }
    Ok(())
}

fn append_cell(
    selected: &SelectedData,
    shaped: &ShapedData,
    scopes: &super::super::scope_alignment::FrameAllocations,
    flow: CellFlow<'_>,
    origin: Point,
    data: &mut FinalizedLineData,
    ledger: &mut Ledger,
) -> Result<f64> {
    let cell_index = flow.index;
    let cell = &shaped.cells[cell_index];
    let mut unit = None;
    for (index, candidate) in selected.presentation.units.iter().enumerate() {
        account::work(ledger, 1)?;
        if matches!(candidate.kind, crate::protocol::prepared::PresentationUnitKind::Cell { cell: original }
            if selected.prepared.shaped.cells[original].binding == cell.binding)
        {
            unit = Some(index);
            break;
        }
    }
    let unit = unit.ok_or_else(|| invalid("cell has no actual selected participant"))?;
    let participant = selected.presentation.units[unit]
        .participant
        .and_then(|index| selected.request.participants.get(index));
    if !matches!(participant, Some(crate::PreparedParticipant::Cell(_))) {
        return Err(invalid(
            "cell selected unit has no genuine Cell participant",
        ));
    }
    let owner = selected
        .scopes
        .unit_owners
        .get(unit)
        .copied()
        .ok_or_else(|| invalid("cell has no admitted scope ownership"))?;
    let delta = scopes.displacement(owner);
    let shift = crate::BaselineShift::try_new(delta)?;
    let reference = scoped_reference(origin.y, delta)?;
    qualify_local_shift(origin.y, reference, -cell.before, shift)?;
    qualify_local_shift(origin.y, reference, cell.after, shift)?;
    let end = add(
        "cell inline end",
        TextNumericPhase::Finalization,
        origin.x,
        cell.extent,
    )?;
    let flow_end = add(
        "cell flow end",
        TextNumericPhase::Finalization,
        origin.x,
        cell.advance,
    )?;
    if let Some(gap) = flow.gap
        && gap.amount != 0.0
    {
        let before = add(
            "cell pre-gap endpoint",
            TextNumericPhase::Finalization,
            origin.x,
            gap.base,
        )?;
        if before == flow_end {
            return Err(super::super::text_numeric_error(
                "cell boundary displacement",
                TextNumericPhase::Finalization,
                TextNumericError::NonZeroLost,
                gap.amount,
            ));
        }
        account::push(
            ledger,
            &mut data.character_anchors,
            SpacingDisplacement {
                frame: FinalizedRunFrame::Line,
                before: Point::new(before, reference),
                after: Point::new(flow_end, reference),
                amount: gap.amount,
            },
        )?;
    }
    let top = add(
        "cell square over edge",
        TextNumericPhase::Finalization,
        reference,
        -cell.before,
    )?;
    let bottom = add(
        "cell square under edge",
        TextNumericPhase::Finalization,
        reference,
        cell.after,
    )?;
    let (frame, transform, tabs) = match &cell.payload {
        CellPayload::NoProviderInput => (None, None, Vec::new()),
        CellPayload::LaidOut { layout, provider } => (
            provider.as_ref().map(|provider| CellFrame {
                x_scale: layout.transform.x_scale,
                x_offset: layout.transform.x_offset,
                baseline: provider.baseline,
            }),
            Some(layout.transform),
            account::clone_vec(ledger, &layout.tabs)?,
        ),
    };
    let index = data.cells.len();
    account::push(
        ledger,
        &mut data.cells,
        FinalizedCell {
            cell: cell_index,
            unit,
            binding: cell.binding,
            reference,
            inline: origin.x..end,
            flow_end,
            advance: cell.advance,
            under: top..bottom,
            scope_displacement: shift,
            level: selected.presentation.units[unit].line_level,
            frame,
            transform,
            tabs,
        },
    )?;
    account::push(
        ledger,
        &mut data.visual_order,
        FinalizedVisualItem::Cell(index),
    )?;
    let CellPayload::LaidOut { layout, provider } = &cell.payload else {
        return Ok(flow_end);
    };
    for position in &layout.positions {
        account::work(ledger, 1)?;
        data.clusters[position.cluster].frame = FinalizedRunFrame::Cell(index);
        data.clusters[position.cluster].x = position.pen;
        data.clusters[position.cluster].reference = Some(reference);
    }
    let Some(provider) = provider else {
        return Ok(flow_end);
    };
    let inner = CellFrame {
        x_scale: layout.transform.x_scale,
        x_offset: layout.transform.x_offset,
        baseline: provider.baseline,
    };
    let unit_frame =
        crate::GlyphFrame::horizontal(Point::new(inner.x_offset, inner.baseline), inner.x_scale);
    let positions = &layout.positions;
    let mut last_run: Option<usize> = None;
    for position in positions {
        account::work(ledger, 1)?;
        let cluster = &shaped.clusters[position.cluster];
        data.clusters[position.cluster].frame = FinalizedRunFrame::Cell(index);
        data.clusters[position.cluster].x = position.pen;
        data.clusters[position.cluster].reference = Some(reference);
        let word_half = position.word.half;
        if position.word.unit_start && word_half != 0.0 {
            let after = add(
                "cell word spacing unit half",
                TextNumericPhase::Finalization,
                position.pen,
                word_half,
            )?;
            qualify_cell_word_half(
                unit_frame,
                position.pen,
                after,
                Point::new(0.0, 0.0),
                word_half,
            )?;
            account::push(
                ledger,
                &mut data.word_anchors,
                SpacingDisplacement {
                    frame: FinalizedRunFrame::Cell(index),
                    before: Point::new(position.pen, 0.0),
                    after: Point::new(after, 0.0),
                    amount: word_half,
                },
            )?;
        }
        let mut glyph_pen = 0.0;
        for glyph in &shaped.glyphs[cluster.glyphs.clone()] {
            account::work(ledger, 1)?;
            let font = cluster
                .content
                .font_run()
                .ok_or_else(|| invalid("cell provider glyph has no retained font"))?;
            let actual = add(
                "cell glyph horizontal pen",
                TextNumericPhase::Finalization,
                position.pen,
                glyph_pen,
            )?;
            let run = if let Some(run) = last_run.filter(|&run| {
                let run = &data.runs[run];
                run.run == font && run.style == glyph.style && run.level == shaped.runs[font].level
            }) {
                run
            } else {
                let run = data.runs.len();
                account::push(
                    ledger,
                    &mut data.runs,
                    FinalizedRun {
                        frame: FinalizedRunFrame::Cell(index),
                        reference: Some(reference),
                        run: font,
                        style: glyph.style,
                        level: shaped.runs[font].level,
                        source: glyph.source,
                        glyphs: data.glyphs.len()..data.glyphs.len(),
                        x: actual,
                        y: shaped.runs[font].metrics.font_origin_offset(),
                        advance: 0.0,
                    },
                )?;
                last_run = Some(run);
                run
            };
            let relative = add(
                "cell run-local horizontal pen",
                TextNumericPhase::Finalization,
                actual,
                -data.runs[run].x,
            )?;
            let pen = add(
                "cell word spacing glyph half",
                TextNumericPhase::Finalization,
                relative,
                word_half,
            )?;
            if word_half != 0.0 {
                // The run origin is PRE-half. Preserve that independent operand;
                // never recover it by subtracting a half from rounded paint data.
                let local_frame = crate::GlyphFrame::horizontal(
                    Point::new(
                        finite(
                            "cell local run x",
                            TextNumericPhase::Finalization,
                            inner.x_offset + inner.x_scale * data.runs[run].x,
                        )?,
                        finite(
                            "cell local run y",
                            TextNumericPhase::Finalization,
                            inner.baseline + data.runs[run].y,
                        )?,
                    ),
                    inner.x_scale,
                );
                qualify_cell_word_half(
                    local_frame,
                    relative,
                    pen,
                    Point::new(f64::from(glyph.x), f64::from(glyph.y)),
                    word_half,
                )?;
            }
            account::push(
                ledger,
                &mut data.glyphs,
                FinalizedGlyph {
                    glyph: *glyph,
                    pen,
                    pen_base: relative,
                    word_half,
                },
            )?;
            data.runs[run].glyphs.end = data.glyphs.len();
            data.runs[run].source.start = data.runs[run].source.start.min(glyph.source.start);
            data.runs[run].source.end = data.runs[run].source.end.max(glyph.source.end);
            glyph_pen = add(
                "cell provider glyph advance",
                TextNumericPhase::Finalization,
                glyph_pen,
                f64::from(glyph.advance),
            )?;
            data.runs[run].advance = add(
                "cell glyph run advance",
                TextNumericPhase::Finalization,
                add(
                    "cell glyph end",
                    TextNumericPhase::Finalization,
                    position.pen,
                    glyph_pen,
                )?,
                -data.runs[run].x,
            )?;
        }
        if let Some(run) = last_run
            && cluster.content.font_run() == Some(data.runs[run].run)
        {
            data.runs[run].advance = add(
                "cell cluster run advance",
                TextNumericPhase::Finalization,
                add(
                    "cell cluster end",
                    TextNumericPhase::Finalization,
                    position.pen,
                    cluster.advance,
                )?,
                -data.runs[run].x,
            )?;
        }
    }
    Ok(flow_end)
}

struct SharedEndpointGeometry {
    offsets: Vec<Option<f64>>,
    leading: Vec<f64>,
}

/// Closed units and the corresponding atom-to-unit index from the same pass.
struct SharedEndpointGroups<'a> {
    ranges: &'a [std::ops::Range<usize>],
    atom_groups: &'a [usize],
}

// Physical endpoints inside a shared unit retain their exact event order, but
// their numeric intervals surround its complete text contribution rather than
// its artificial component/caret shares. The flow pen consumes every item once.
fn shared_endpoint_offsets(
    selected: &SelectedData,
    shaped: &ShapedData,
    dispositions: &[SpacingSlotDisposition],
    order: &crate::SuppliedVisualOrder<'_, '_>,
    groups: SharedEndpointGroups<'_>,
    boxed_units: Option<&[bool]>,
    ledger: &mut Ledger,
) -> Result<Option<SharedEndpointGeometry>> {
    let SharedEndpointGroups {
        ranges: groups,
        atom_groups,
    } = groups;
    account::work(ledger, shaped.items.len())?;
    let mut endpoints = shaped.items.iter().filter_map(|item| {
        let ShapedItem::Anchor { unit } = *item else {
            return None;
        };
        if boxed_units.is_some_and(|units| units[unit]) {
            return None;
        }
        let crate::protocol::prepared::PresentationUnitKind::Control { occurrence } =
            selected.presentation.units[unit].kind
        else {
            return None;
        };
        if !crate::protocol::boundary::is_endpoint(
            &selected.prepared.analyzed.paragraph.controls[occurrence].kind,
        ) {
            return None;
        }
        let cut = selected.presentation.units[unit].atoms.start;
        let group = groups.partition_point(|group| group.end <= cut);
        groups
            .get(group)
            .is_some_and(|group| group.start < cut && cut < group.end)
            .then_some((
                unit,
                group,
                selected.presentation.units[unit].line_level & 1 != 0,
            ))
    });
    // Ordinary lines retain their existing geometry path without allocating
    // shared-endpoint state. Reuse this same iterator after the first demand.
    let Some(first) = endpoints.next() else {
        return Ok(None);
    };
    let mut endpoint_groups = account::filled(ledger, selected.presentation.units.len(), None)?;
    let mut remaining = account::filled(ledger, groups.len(), None)?;
    let mut consumed = account::filled(ledger, groups.len(), None)?;
    let mut pending = account::filled(ledger, groups.len(), [0; 2])?;
    for (unit, group, odd) in std::iter::once(first).chain(endpoints) {
        endpoint_groups[unit] = Some((group, odd));
        if odd {
            consumed[group] = Some(0.0);
        } else {
            remaining[group] = Some(0.0);
        }
        pending[group][usize::from(odd)] += 1;
    }
    let mut result = SharedEndpointGeometry {
        offsets: account::filled(ledger, selected.presentation.units.len(), None)?,
        leading: account::filled(ledger, groups.len(), 0.0)?,
    };
    // Odd endpoints occupy the physical leading side. Retain the complete
    // leading inset so even the component visited before those endpoints moves
    // with the same provider glyph packet, without moving a mark independently.
    for (ordinal, _) in order.iter() {
        match shaped.items[ordinal] {
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if let Some(value) = &mut consumed[atom_groups[cluster.atoms.start]] {
                    *value = add(
                        "shared unit consumed text advance",
                        TextNumericPhase::Finalization,
                        *value,
                        placed_cluster_advance(cluster, dispositions[index]),
                    )?;
                }
            }
            ShapedItem::Anchor { unit } => {
                if let Some((group, true)) = endpoint_groups[unit] {
                    result.offsets[unit] = consumed[group].map(|value| -value);
                    result.leading[group] = add(
                        "shared unit leading endpoint advance",
                        TextNumericPhase::Finalization,
                        result.leading[group],
                        crate::protocol::boundary::advance(selected.presentation_input(), unit),
                    )?;
                    pending[group][1] -= 1;
                    if pending[group][1] == 0 {
                        consumed[group] = None;
                    }
                }
            }
            ShapedItem::Cell(_) | ShapedItem::InlineBox { .. } => {}
        }
    }
    // Even endpoints occupy the physical trailing side. Only the suffix needed
    // by an actual endpoint demands arithmetic; use the post-L1/L2 visual order.
    let ordinals = account::collect(ledger, order.iter().map(|(ordinal, _)| ordinal))?;
    for &ordinal in ordinals.iter().rev() {
        match shaped.items[ordinal] {
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if let Some(value) = &mut remaining[atom_groups[cluster.atoms.start]] {
                    *value = add(
                        "shared unit remaining text advance",
                        TextNumericPhase::Finalization,
                        placed_cluster_advance(cluster, dispositions[index]),
                        *value,
                    )?;
                }
            }
            ShapedItem::Anchor { unit } => {
                if let Some((group, false)) = endpoint_groups[unit] {
                    result.offsets[unit] = remaining[group];
                    pending[group][0] -= 1;
                    if pending[group][0] == 0 {
                        remaining[group] = None;
                    }
                }
            }
            ShapedItem::Cell(_) | ShapedItem::InlineBox { .. } => {}
        }
    }
    account::drop_vec(ledger, ordinals)?;
    account::drop_vec(ledger, endpoint_groups)?;
    account::drop_vec(ledger, remaining)?;
    account::drop_vec(ledger, consumed)?;
    account::drop_vec(ledger, pending)?;
    Ok(Some(result))
}

fn placed_cluster_advance(cluster: &PreparedCluster, disposition: SpacingSlotDisposition) -> f64 {
    if disposition == SpacingSlotDisposition::DiscardedAtLineEdge
        || cluster.whitespace == Whitespace::Newline
    {
        0.0
    } else {
        cluster.advance
    }
}

fn fragment_key(owner: crate::protocol::paragraph::FragmentOwner) -> (u8, usize) {
    use crate::protocol::paragraph::FragmentOwner;
    match owner {
        FragmentOwner::Origin(index) => (0, index),
        FragmentOwner::Control(index) => (1, index),
        FragmentOwner::Alternative(index) => (2, index),
        FragmentOwner::VirtualSeparator(index) => (3, index),
    }
}

fn add_owned_fragment_bounds(
    index: &[((u8, usize), usize)],
    fragments: &mut [FinalizedFragmentData],
    owner: crate::protocol::paragraph::FragmentOwner,
    rect: Rect,
    ledger: &mut Ledger,
) -> Result<()> {
    let key = fragment_key(owner);
    let first = index.partition_point(|(old, _)| *old < key);
    for &(_, index) in index[first..].iter().take_while(|(old, _)| *old == key) {
        account::work(ledger, 1)?;
        let row = &mut fragments[index];
        row.bounds = Some(row.bounds.map_or(rect, |old| union(old, rect)));
    }
    Ok(())
}

fn add_fragment_bounds(
    index: &[(crate::Range, usize, usize)],
    fragments: &mut [FinalizedFragmentData],
    source: crate::Range,
    rect: Rect,
    ledger: &mut Ledger,
) -> Result<()> {
    if source.is_empty() {
        return Ok(());
    }
    let first = index.partition_point(|(_, _, maximum_end)| *maximum_end <= source.start);
    for &(range, index, _) in &index[first..] {
        account::work(ledger, 1)?;
        if range.start >= source.end {
            break;
        }
        if range.end > source.start {
            let row = &mut fragments[index];
            row.bounds = Some(row.bounds.map_or(rect, |old| union(old, rect)));
        }
    }
    Ok(())
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
    crate::AtomicFitAllocation,
)> {
    match allocation.allocation {
        crate::protocol::AtomicLineAllocation::InFlow {
            flow,
            box_geometry,
            alignment,
            fit,
        } => Ok((flow, box_geometry, alignment, fit)),
        crate::protocol::AtomicLineAllocation::OutOfFlow => {
            Err(protocol_error(ProtocolError::InvalidAtomicGeometry {
                occurrence: allocation.occurrence.index,
            }))
        }
    }
}

fn install_out_of_flow(
    selected: &SelectedData,
    data: &mut FinalizedLineData,
    scopes: &super::super::scope_alignment::FrameAllocations,
    ledger: &mut Ledger,
) -> Result<()> {
    use crate::protocol::{
        AtomicLineAllocation, PreparedParticipant, prepared::PresentationUnitKind,
    };
    let units = &selected.presentation.units;
    let mut slots = account::filled(ledger, units.len(), None)?;
    for (index, unit) in units.iter().enumerate() {
        if matches!(unit.kind, PresentationUnitKind::Cell { .. }) {
            continue;
        }
        let mut first_cluster = None;
        let mut last_cluster = None;
        for cluster in &data.clusters {
            if cluster.frame != FinalizedRunFrame::Line {
                continue;
            }
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
    // Inner clusters use intrinsic coordinates. An exterior static site instead
    // consumes the actual Cell's executed outer flow endpoints, including its
    // following tracking/allocation, in the selected line's resolved direction.
    for cell in &data.cells {
        account::work(ledger, 1)?;
        slots[cell.unit] = Some(if cell.level & 1 == 0 {
            (cell.inline.start, cell.flow_end)
        } else {
            (cell.flow_end, cell.inline.start)
        });
    }
    for anchor in &data.anchors {
        slots[anchor.unit] = Some(if anchor.level & 1 == 0 {
            anchor.flow_interval()
        } else {
            {
                let (start, end) = anchor.flow_interval();
                (end, start)
            }
        });
    }
    for item in &data.boxes {
        let unit = units.iter().position(|unit| matches!(unit.kind,PresentationUnitKind::Atomic {occurrence} if occurrence == item.occurrence)).ok_or_else(|| invalid("final atomic has no actual selected unit"))?;
        let level = units[unit].level;
        let end = add(
            "atomic logical endpoint",
            TextNumericPhase::Finalization,
            item.slot.x,
            item.cell_advance(),
        )?;
        slots[unit] = Some(if level & 1 == 0 {
            (item.slot.x, end)
        } else {
            (end, item.slot.x)
        });
    }
    let mut before = account::filled(ledger, units.len(), None)?;
    let mut prior = None;
    for (index, slot) in slots.iter().enumerate() {
        before[index] = prior;
        if let Some((_, end)) = slot {
            prior = Some(*end);
        }
    }
    let mut after = account::filled(ledger, units.len(), None)?;
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
        account::push(
            ledger,
            &mut data.boxes,
            FinalizedBox {
                occurrence,
                atomic: atomic.clone(),
                slot: Point::new(
                    x,
                    scoped_reference(
                        data.metrics.baseline,
                        scopes.displacement(
                            selected
                                .scopes
                                .unit_owners
                                .get(unit_index)
                                .copied()
                                .flatten(),
                        ),
                    )?,
                ),
                scope_displacement: crate::BaselineShift::try_new(
                    scopes.displacement(
                        selected
                            .scopes
                            .unit_owners
                            .get(unit_index)
                            .copied()
                            .flatten(),
                    ),
                )?,
                following_gap: 0.0,
                geometry: FinalizedBoxGeometry::OutOfFlow,
            },
        )?;
    }
    account::drop_vec(ledger, slots)?;
    account::drop_vec(ledger, before)?;
    account::drop_vec(ledger, after)?;
    Ok(())
}

fn scoped_reference(baseline: f64, displacement: f64) -> Result<f64> {
    add(
        "scope occurrence reference",
        TextNumericPhase::Finalization,
        baseline,
        -displacement,
    )
}

fn unit_scope_displacement(
    selected: &SelectedData,
    scopes: &super::super::scope_alignment::FrameAllocations,
    unit: usize,
    controls: &OrdinalMap<&ControlAllocation>,
) -> f64 {
    if let crate::protocol::prepared::PresentationUnitKind::Control { occurrence } =
        selected.presentation.units[unit].kind
        && controls
            .get(&occurrence)
            .and_then(|allocation| allocation.strut)
            .is_some_and(|strut| {
                matches!(
                    strut.alignment,
                    LineBoxAlignment::Top | LineBoxAlignment::Bottom
                )
            })
    {
        return 0.0;
    }
    scopes.displacement(selected.scopes.unit_owners.get(unit).copied().flatten())
}

fn atomic_scope_displacement(
    selected: &SelectedData,
    scopes: &super::super::scope_alignment::FrameAllocations,
    occurrence: usize,
    alignment: LineBoxAlignment,
    ledger: &mut Ledger,
) -> Result<f64> {
    if matches!(alignment, LineBoxAlignment::Top | LineBoxAlignment::Bottom) {
        return Ok(0.0);
    }
    for (index, unit) in selected.presentation.units.iter().enumerate() {
        account::work(ledger, 1)?;
        if matches!(unit.kind,crate::protocol::prepared::PresentationUnitKind::Atomic{occurrence:actual} if actual==occurrence)
        {
            return Ok(
                scopes.displacement(selected.scopes.unit_owners.get(index).copied().flatten())
            );
        }
    }
    Ok(0.0)
}

fn control_allocations<'a>(
    selected: &'a SelectedData,
    ledger: &mut Ledger,
) -> Result<OrdinalMap<&'a ControlAllocation>> {
    let mut result = OrdinalMap::default();
    for allocation in &selected.request.controls {
        let row = selected
            .prepared
            .tables
            .controls
            .get(allocation.occurrence.index)
            .ok_or_else(|| invalid("selected control has no prepared occurrence"))?;
        account::insert(ledger, &mut result, row.occurrence, allocation)?;
    }
    Ok(result)
}

/// The same metric-group algebra used by Layout's real box/control fitter.
/// A supplied strut is an allocation fact, never a manufactured font metric.
fn include_control_struts(
    metrics: &mut LineMetrics,
    struts: impl Iterator<Item = ControlStrut>,
    empty: Option<crate::protocol::LineStrut>,
    have_base: bool,
) -> Result<()> {
    let mut groups = MetricGroups::default();
    if have_base {
        groups.include(
            metrics.baseline - metrics.block_min,
            metrics.block_max - metrics.baseline,
            LineBoxAlignment::Baseline,
        )?;
    }
    if let Some(strut) = empty {
        groups.include(
            strut.baseline,
            strut.line_extent - strut.baseline,
            LineBoxAlignment::Baseline,
        )?;
    }
    let mut any = false;
    for strut in struts {
        any = true;
        groups.include(
            strut.baseline,
            strut.line_extent - strut.baseline,
            strut.alignment,
        )?;
    }
    if any || empty.is_some() {
        *metrics = groups.finish(*metrics)?;
    }
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
            LineBoxAlignment::Shift(shift) => return shifted_baseline(metrics.baseline, shift),
            LineBoxAlignment::Top => metrics.block_min + strut.baseline,
            LineBoxAlignment::Bottom => metrics.block_max - (strut.line_extent - strut.baseline),
        },
    )
}

fn atomic_allocations<'a>(
    selected: &'a SelectedData,
    ledger: &mut Ledger,
) -> Result<OrdinalMap<&'a AtomicAllocation>> {
    let mut result = OrdinalMap::default();
    for allocation in &selected.request.atomics {
        let row = selected
            .prepared
            .tables
            .atomics
            .get(allocation.occurrence.index)
            .ok_or_else(|| invalid("selected atomic has no prepared occurrence"))?;
        account::insert(ledger, &mut result, row.occurrence, allocation)?;
    }
    Ok(result)
}

fn validate_rows(selected: PresentationInput<'_>, shaped: &ShapedData) -> Result<()> {
    for cluster in &shaped.clusters {
        crate::range::validate(
            selected.prepared.analyzed.processed.source().text(),
            cluster.source,
        )?;
        if cluster
            .content
            .font_run()
            .is_some_and(|run| run >= shaped.runs.len())
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
            "cluster line height",
            TextNumericPhase::Finalization,
            cluster.line_height,
        )?;
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
            ShapedItem::Cell(index) if *index >= shaped.cells.len() => {
                return Err(invalid("selected cell item has no retained cell"));
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
    atomics: &OrdinalMap<&AtomicAllocation>,
    controls: &OrdinalMap<&ControlAllocation>,
    dispositions: &mut [SpacingSlotDisposition],
    prefix_receipts: &mut Vec<f64>,
    ledger: &mut Ledger,
) -> Result<LineMetrics> {
    account::work(ledger, shaped.items.len())?;
    let mut result = LineMetrics::default();
    let mut have_metrics = false;
    let mut have_cluster_height = false;
    account::push(ledger, prefix_receipts, 0.0)?;
    for item in &shaped.items {
        match *item {
            ShapedItem::Cell(cell) => {
                let cell = &shaped.cells[cell];
                result.advance = add(
                    "line cell advance",
                    TextNumericPhase::Finalization,
                    result.advance,
                    cell.advance,
                )?;
                result.line_height = result.line_height.max(cell.extent);
                have_cluster_height = true;
            }
            ShapedItem::Anchor { unit } => {
                result.advance = add(
                    "line boundary advance",
                    TextNumericPhase::Finalization,
                    result.advance,
                    crate::protocol::boundary::advance(selected.presentation_input(), unit),
                )?;
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if cluster.content.font_run().is_some() {
                    have_cluster_height = true;
                    result.line_height = result.line_height.max(cluster.line_height);
                }
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
                    super::super::spacing::character::atomic_advance(
                        shaped,
                        occurrence,
                        flow.advance,
                    )?,
                )?;
                result.line_height = result.line_height.max(finite(
                    "atomic line height",
                    TextNumericPhase::Finalization,
                    flow.before_baseline + flow.after_baseline,
                )?);
            }
        }
        account::push(ledger, prefix_receipts, result.advance)?;
    }
    result.trailing_whitespace = classify_trailing(selected, shaped, dispositions)?;
    if selected.metric_policy == LineMetricPolicy::Native {
        return native_metrics(selected, shaped, atomics, controls, result);
    }
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::Cell(cell) => {
                result.ascent = result.ascent.max(shaped.cells[cell].before);
                result.descent = result.descent.max(shaped.cells[cell].after);
                have_metrics = true;
            }
            ShapedItem::Anchor { .. } => {}
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if !have_metrics && cluster.is_whitespace {
                    continue;
                }
                let Some(run) = cluster.content.font_run() else {
                    continue;
                };
                let metrics = shaped.runs[run].metrics;
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
            // Nonempty whitespace already supplied its actual component heights.
            // A retained run's initial style may belong to unselected content.
            if !have_cluster_height {
                result.line_height = result.line_height.max(font.metrics.line_height);
            }
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
pub(super) fn font_strut(metrics: RunMetrics) -> Result<(f64, f64)> {
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

fn shifted_baseline(baseline: f64, shift: crate::BaselineShift) -> Result<f64> {
    add(
        "allocated baseline displacement",
        TextNumericPhase::Finalization,
        baseline,
        -shift.get(),
    )
}

fn qualify_local_shift(
    before: f64,
    after: f64,
    local: f64,
    shift: crate::BaselineShift,
) -> Result<()> {
    let before = finite(
        "unshifted allocated coordinate",
        TextNumericPhase::Finalization,
        before + local,
    )?;
    let after = finite(
        "shifted allocated coordinate",
        TextNumericPhase::Finalization,
        after + local,
    )?;
    if shift.get() != 0.0 && before == after {
        return Err(super::super::text_numeric_error(
            "allocated coordinate displacement",
            TextNumericPhase::Finalization,
            crate::TextNumericError::NonZeroLost,
            shift.get(),
        ));
    }
    Ok(())
}

/// Preserve local metrics; only their relative endpoints are displaced.
fn shifted_bounds(before: f64, after: f64, shift: crate::BaselineShift) -> Result<(f64, f64)> {
    let actual_before = add(
        "allocated shifted over bound",
        TextNumericPhase::Finalization,
        before,
        shift.get(),
    )?;
    let actual_after = add(
        "allocated shifted under bound",
        TextNumericPhase::Finalization,
        after,
        -shift.get(),
    )?;
    let extent = finite(
        "allocated local extent",
        TextNumericPhase::Finalization,
        before + after,
    )?;
    let actual_extent = finite(
        "allocated shifted extent",
        TextNumericPhase::Finalization,
        actual_before + actual_after,
    )?;
    if extent != 0.0 && actual_extent == 0.0 {
        return Err(super::super::text_numeric_error(
            "allocated shifted extent",
            TextNumericPhase::Finalization,
            crate::TextNumericError::NonZeroLost,
            extent,
        ));
    }
    Ok((actual_before, actual_after))
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
            LineBoxAlignment::Baseline | LineBoxAlignment::Shift(_) => {
                let (before, after) = match alignment {
                    LineBoxAlignment::Shift(shift) => shifted_bounds(before, after, shift)?,
                    _ => (before, after),
                };
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
    atomics: &OrdinalMap<&AtomicAllocation>,
    controls: &OrdinalMap<&ControlAllocation>,
    result: LineMetrics,
) -> Result<LineMetrics> {
    native_metrics_scoped(selected, shaped, atomics, controls, result, None)
}

fn native_metrics_scoped(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    controls: &OrdinalMap<&ControlAllocation>,
    result: LineMetrics,
    scopes: Option<&super::super::scope_alignment::FrameAllocations>,
) -> Result<LineMetrics> {
    let mut groups = MetricGroups::default();
    if let Some(scopes) = scopes {
        for row in &selected.scopes.contributions {
            let delta = scopes.displacement(row.owner);
            let alignment = match row.metrics.alignment {
                LineBoxAlignment::Top | LineBoxAlignment::Bottom => row.metrics.alignment,
                LineBoxAlignment::Baseline if delta == 0.0 => LineBoxAlignment::Baseline,
                LineBoxAlignment::Baseline => {
                    LineBoxAlignment::Shift(crate::BaselineShift::try_new(delta)?)
                }
                LineBoxAlignment::Shift(own) => {
                    LineBoxAlignment::Shift(crate::BaselineShift::try_new(add(
                        "scope leaf alignment",
                        TextNumericPhase::Finalization,
                        own.get(),
                        delta,
                    )?)?)
                }
            };
            groups.include(row.metrics.before, row.metrics.after, alignment)?;
        }
        return groups.finish(result);
    }
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
    for item in &shaped.items {
        if let ShapedItem::Cell(cell) = *item {
            let cell = &shaped.cells[cell];
            groups.include(cell.before, cell.after, LineBoxAlignment::Baseline)?;
        }
        if let ShapedItem::TextCluster(cluster) = *item
            && let Some(run) = shaped.clusters[cluster].content.font_run()
        {
            let cluster = &shaped.clusters[cluster];
            let metrics = RunMetrics {
                line_height: cluster.line_height,
                ..shaped.runs[run].metrics
            };
            let (before, after) = font_strut(metrics)?;
            groups.include(before, after, LineBoxAlignment::Baseline)?;
        }
    }
    for item in &shaped.items {
        if let ShapedItem::InlineBox { occurrence, .. } = *item {
            let (flow, _, alignment, _) = inflow_allocation(
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
    for allocation in &selected.request.line_box_struts {
        groups.include(
            allocation.metrics.baseline,
            allocation.metrics.line_extent - allocation.metrics.baseline,
            allocation.alignment,
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

/// Applicable collapsible-space line-edge suppression. Invisible occurrence/control
/// anchors and out-of-flow boxes are transparent; real in-flow atomics stop it.
fn collapse_edges(
    selected: PresentationInput<'_>,
    shaped: &mut ShapedData,
) -> Result<Vec<SpacingSlotDisposition>> {
    collapse_edges_accounted(selected, shaped, &mut None)
}
fn collapse_edges_accounted(
    selected: PresentationInput<'_>,
    shaped: &mut ShapedData,
    ledger: &mut Ledger,
) -> Result<Vec<SpacingSlotDisposition>> {
    let dispositions = edge_dispositions_accounted(selected, shaped, ledger)?;
    for (index, disposition) in dispositions.iter().enumerate() {
        if *disposition == SpacingSlotDisposition::DiscardedAtLineEdge {
            shaped.clusters[index].advance = 0.0;
        }
    }
    Ok(dispositions)
}

fn edge_dispositions(
    selected: PresentationInput<'_>,
    shaped: &ShapedData,
) -> Result<Vec<SpacingSlotDisposition>> {
    edge_dispositions_accounted(selected, shaped, &mut None)
}
fn edge_dispositions_accounted(
    selected: PresentationInput<'_>,
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<Vec<SpacingSlotDisposition>> {
    account::work(ledger, shaped.items.len())?;
    let mut projected = Vec::new();
    let mut clusters = Vec::new();
    for item in &shaped.items {
        match *item {
            ShapedItem::Cell(cell) => {
                account::push(
                    ledger,
                    &mut projected,
                    SuppliedInlineParticipant::Cell {
                        level: BidiLevel::try_new(shaped.cells[cell].level)
                            .map_err(|_| invalid("invalid cell edge level"))?,
                    },
                )?;
                account::push(ledger, &mut clusters, None)?;
            }
            ShapedItem::Anchor { .. } => {}
            ShapedItem::InlineBox { occurrence, level } => {
                if selected.prepared.analyzed.processed.source().boxes()[occurrence].kind()
                    == crate::InlineBoxKind::InFlow
                {
                    account::push(
                        ledger,
                        &mut projected,
                        SuppliedInlineParticipant::Atomic {
                            level: BidiLevel::try_new(level)
                                .map_err(|_| invalid("invalid atomic edge level"))?,
                        },
                    )?;
                    account::push(ledger, &mut clusters, None)?;
                }
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &shaped.clusters[index];
                if selected.metric_policy == LineMetricPolicy::Native
                    && cluster.whitespace == Whitespace::Newline
                {
                    continue;
                }
                let mode = selected
                    .prepared
                    .analyzed
                    .styles
                    .get(cluster.style)
                    .ok_or_else(|| invalid("cluster has no captured processing style"))?
                    .style
                    .authored()
                    .white_space;
                let edge = crate::engine::analysis::edge::whitespace_edge(
                    cluster.whitespace == Whitespace::Space,
                    selected.presentation.projection.atoms[cluster.atoms.clone()]
                        .iter()
                        .all(|atom| atom.code_point == 0x1680),
                    mode,
                    selected.metric_policy == LineMetricPolicy::Native,
                );
                account::push(
                    ledger,
                    &mut projected,
                    SuppliedInlineParticipant::Text {
                        level: BidiLevel::try_new(cluster.level)
                            .map_err(|_| invalid("invalid text edge level"))?,
                        edge,
                    },
                )?;
                account::push(ledger, &mut clusters, Some(index))?;
            }
        }
    }
    let edges = SuppliedLine::new(&projected).edges();
    let mut dispositions = account::filled(
        ledger,
        shaped.clusters.len(),
        SpacingSlotDisposition::Retained { trailing: false },
    )?;
    for (ordinal, &index) in clusters.iter().enumerate() {
        if let Some(index) = index
            && edges.is_discarded(ordinal) == Some(true)
        {
            dispositions[index] = SpacingSlotDisposition::DiscardedAtLineEdge;
        }
    }
    account::drop_vec(ledger, clusters)?;
    account::drop_vec(ledger, projected)?;
    Ok(dispositions)
}

/// One executed trailing classification supplies both aggregate metrics and
/// per-slot readback. Real selected anchors are transparent; in-flow items stop
/// the scan. Out-of-flow occurrences are absent from this shaped flow sequence.
fn classify_trailing(
    selected: &SelectedData,
    shaped: &ShapedData,
    dispositions: &mut [SpacingSlotDisposition],
) -> Result<f64> {
    let mut advance = 0.0;
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::Anchor { unit } => {
                if crate::protocol::boundary::facts(selected.presentation_input(), unit)
                    .allocation()
                    .is_some_and(crate::InlineBoundaryAllocation::blocks_hanging)
                {
                    break;
                }
                continue;
            }
            ShapedItem::TextCluster(index) if shaped.clusters[index].is_whitespace => {
                if let SpacingSlotDisposition::Retained { trailing } = &mut dispositions[index] {
                    *trailing = true;
                }
                if shaped.clusters[index].whitespace != Whitespace::Newline {
                    advance = add(
                        "line trailing advance",
                        TextNumericPhase::Finalization,
                        advance,
                        shaped.clusters[index].advance,
                    )?;
                }
            }
            _ => break,
        }
    }
    Ok(advance)
}

fn apply_spacing(
    selected: &SelectedData,
    shaped: &mut ShapedData,
    amounts: &[f64],
    dispositions: &[SpacingSlotDisposition],
    responses: &[(crate::SpacingStretchability, usize)],
    character_order: &[usize],
    ledger: &mut Ledger,
) -> Result<Vec<FinalizedSpacingSlot>> {
    if amounts.len() != selected.spacing_slots.len() {
        return Err(invalid("spacing amounts do not cover selected slots"));
    }
    if selected.request.context.spacing == crate::SpacingPolicy::InterCharacter {
        return character_slots(
            selected,
            shaped,
            amounts,
            dispositions,
            responses,
            character_order,
            ledger,
        );
    }
    let mut result = Vec::new();
    let mut cluster_at =
        account::filled(ledger, selected.presentation.projection.atoms.len(), None)?;
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        for atom in cluster.atoms.clone() {
            cluster_at[atom] = Some(index);
        }
    }
    let mut units = OrdinalMap::default();
    for (index, unit) in shaped.word_spacing.iter().enumerate() {
        account::insert(ledger, &mut units, unit.base, index)?;
    }
    for (slot, (row, &amount)) in selected.spacing_slots.iter().zip(amounts).enumerate() {
        let crate::protocol::selection::SpacingTargetData::Separator { atoms, base, .. } =
            &row.target
        else {
            return Err(invalid("separator target"));
        };
        let base = *base;
        finite(
            "selected spacing amount",
            TextNumericPhase::Finalization,
            amount,
        )?;
        let mut members = Vec::new();
        for atom in atoms.clone() {
            if let Some(index) = cluster_at[atom]
                && members.last() != Some(&index)
            {
                account::push(ledger, &mut members, index)?;
            }
        }
        let index = members
            .iter()
            .copied()
            .find(|&index| shaped.clusters[index].atoms.contains(&base))
            .ok_or_else(|| protocol_error(ProtocolError::IneligibleSpacingSlot { slot }))?;
        let unit = units.get(&base).copied();
        let (geometry, shared) =
            if let Some(unit) = unit.filter(|&index| shaped.word_spacing[index].atoms == *atoms) {
                let unit = &shaped.word_spacing[unit];
                (unit.geometry, unit.shared.clone())
            } else {
                super::super::spacing::geometry_accounted(shaped, &members, atoms.clone(), ledger)?
            };
        let authored = unit.map_or(0.0, |unit| shaped.word_spacing[unit].authored);
        if dispositions[index] == SpacingSlotDisposition::DiscardedAtLineEdge && amount != 0.0 {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        }
        match selected.request.context.spacing {
            crate::SpacingPolicy::Spaces => {
                let cluster = &mut shaped.clusters[index];
                if !cluster.whitespace.is_space_or_nbsp() {
                    return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                        slot,
                    }));
                }
                if let Some(gap) = shaped.character_gaps.iter_mut().find(|gap| matches!(gap.owner, super::super::spacing::character::Owner::Cluster(i) if i == index)) {
                    gap.base = add("native space cell basis", TextNumericPhase::Finalization, gap.base, amount)?;
                    cluster.advance = add("native space and tracking", TextNumericPhase::Finalization, gap.base, gap.amount)?;
                } else { cluster.advance = add("selected spacing amount", TextNumericPhase::Finalization, cluster.advance, amount)?; }
                cluster.logical_adjustments[2] = amount;
            }
            crate::SpacingPolicy::InterWord => {
                if amount < 0.0 {
                    return Err(super::super::text_numeric_error(
                        "selected spacing amount",
                        TextNumericPhase::Finalization,
                        TextNumericError::OutOfRange,
                        amount,
                    ));
                }
                if amount != 0.0 {
                    if let Some(provider_atoms) = shared {
                        return Err(super::super::spacing::unsupported(
                            crate::SpacingStage::SelectedShaping,
                            crate::SpacingOperation::InterWordAllocation,
                            row.pieces.iter().map(|piece| piece.owner).collect(),
                            provider_atoms,
                        ));
                    }
                    if matches!(
                        geometry,
                        crate::SpacingSlotGeometry::Independent {
                            intrinsic_advance: 0.0
                        }
                    ) {
                        return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                            slot,
                        }));
                    }
                }
                shaped.word_spacing[unit
                    .ok_or_else(|| invalid("separator candidate has no actual unit geometry"))?]
                .allocation = amount;
            }
            crate::SpacingPolicy::None | crate::SpacingPolicy::InterCharacter => {
                return Err(invalid("None has no selected spacing slots"));
            }
        }
        account::push(
            ledger,
            &mut result,
            FinalizedSpacingSlot {
                cluster: index,
                edge: None,
                members,
                disposition: dispositions[index],
                geometry,
                stretchability: responses[index].0,
                traversal_order: responses[index].1,
                authored,
                letter: 0.0,
                allocation: amount,
                advance: 0.0,
            },
        )?;
    }
    account::drop_vec(ledger, cluster_at)?;
    account::drop_vec(ledger, units.rows)?;
    Ok(result)
}

fn character_slots(
    selected: &SelectedData,
    shaped: &ShapedData,
    amounts: &[f64],
    dispositions: &[SpacingSlotDisposition],
    responses: &[(crate::SpacingStretchability, usize)],
    order: &[usize],
    ledger: &mut Ledger,
) -> Result<Vec<FinalizedSpacingSlot>> {
    use super::super::spacing::character::{self, Owner};
    use crate::{SpacingBoundaryGeometry as Geometry, SpacingSlotGeometry};
    let mut rows = Vec::new();
    let owners = selected.basis.characters.owners(shaped, order, ledger)?;
    for (slot, (target, &amount)) in selected.spacing_slots.iter().zip(amounts).enumerate() {
        let crate::protocol::selection::SpacingTargetData::Boundary { edge, .. } = target.target
        else {
            return Err(invalid("character boundary target"));
        };
        finite(
            "selected character amount",
            TextNumericPhase::Finalization,
            amount,
        )?;
        if amount < 0.0 {
            return Err(super::super::text_numeric_error(
                "selected character amount",
                TextNumericPhase::Finalization,
                TextNumericError::OutOfRange,
                amount,
            ));
        }
        let boundary = &selected.basis.characters.boundaries[edge];
        let geometry =
            selected
                .basis
                .characters
                .geometry(edge, &shaped.character_invocations, ledger)?;
        let geometry = if geometry == Geometry::Independent
            && !character::qualified(&selected.basis.characters, edge)
        {
            Geometry::Constrained
        } else {
            geometry
        };
        let discarded = character::discarded(
            &selected.basis.characters,
            edge,
            shaped,
            dispositions,
            ledger,
        )?;
        let protected = matches!(
            geometry,
            Geometry::ProtectedRequired | Geometry::ProtectedCursive
        );
        if amount != 0.0 && (discarded || protected) {
            return Err(protocol_error(ProtocolError::IneligibleSpacingSlot {
                slot,
            }));
        }
        let owner = owners[boundary.before];
        let (cluster, response, rank) = match owner {
            Some(Owner::Cluster(i)) => (i, responses[i].0, responses[i].1),
            Some(owner @ (Owner::Atomic(_) | Owner::Cell(_))) => {
                let mut traversal = account::clone_vec(ledger, order)?;
                let scans = traversal.len().checked_mul(3).ok_or_else(|| {
                    super::super::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
                        kind: crate::ScopeAlignmentLimitKind::StructuralWork,
                    })
                })?;
                account::work(ledger, scans)?;
                if selected.presentation.analysis.base_level & 1 != 0 {
                    traversal.reverse();
                }
                let rank = traversal
                    .iter()
                    .position(|&i| match (owner, shaped.items[i]) {
                        (Owner::Atomic(expected), ShapedItem::InlineBox { occurrence, .. }) => {
                            occurrence == expected
                        }
                        (Owner::Cell(expected), ShapedItem::Cell(cell)) => cell == expected,
                        _ => false,
                    })
                    .unwrap_or(0);
                let downstream_tab=traversal.iter().skip(rank+1).any(|&i| matches!(shaped.items[i],ShapedItem::TextCluster(c) if matches!(shaped.clusters[c].content,ClusterContent::CssTab(_)) && responses[c].0!=crate::SpacingStretchability::Excluded));
                account::drop_vec(ledger, traversal)?;
                (
                    usize::MAX,
                    if downstream_tab {
                        crate::SpacingStretchability::TabDependent
                    } else {
                        crate::SpacingStretchability::Direct
                    },
                    rank,
                )
            }
            None => (usize::MAX, crate::SpacingStretchability::Excluded, 0),
        };
        account::push(
            ledger,
            &mut rows,
            FinalizedSpacingSlot {
                cluster,
                edge: Some(edge),
                members: Vec::new(),
                geometry: SpacingSlotGeometry::Boundary(geometry),
                stretchability: if discarded || protected {
                    crate::SpacingStretchability::Excluded
                } else {
                    response
                },
                traversal_order: rank,
                authored: 0.0,
                letter: if geometry == Geometry::ProtectedRequired || discarded {
                    0.0
                } else {
                    boundary.authored
                },
                allocation: amount,
                advance: 0.0,
                disposition: if discarded {
                    SpacingSlotDisposition::DiscardedAtLineEdge
                } else {
                    SpacingSlotDisposition::Retained { trailing: false }
                },
            },
        )?;
    }
    account::drop_vec(ledger, owners)?;
    Ok(rows)
}

/// The standalone policy produces the same explicit logical allocations as a
/// composed consumer; the core finalizer never performs implicit justification.
pub(super) fn standalone_amounts(
    selected: &SelectedData,
    metrics: &LineMetrics,
    measurements: &crate::LineMeasurements,
) -> Result<Vec<f64>> {
    let measured = LineMetrics {
        advance: measurements.alignment().advance(),
        trailing_whitespace: 0.0,
        ..*metrics
    };
    let metrics = &measured;
    let selection = &selected.request.context;
    let mut amounts = vec![0.0; selected.spacing_slots.len()];
    if selection.alignment != Alignment::Justify || selection.end != LineEnd::Soft {
        return Ok(amounts);
    }
    let width = selection.available_inline.unwrap_or_else(|| {
        (metrics.advance + selection.indent.max(0.0) - metrics.trailing_whitespace).max(0.0)
    });
    let width = alignment_width(selection, width)?;
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
                slot.pieces
                    .first()
                    .is_none_or(|p| p.presentation.start != last.start)
                    || slot
                        .pieces
                        .last()
                        .is_none_or(|p| p.presentation.end != last.end)
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

/// The original width remains the outer metric and tab frame. Only ordinary
/// content alignment and its explicit spacing budget consume these insets.
fn alignment_width(selection: &LineContext, width: f64) -> Result<f64> {
    let insets = selection.alignment_insets;
    if insets == crate::LineAlignmentInsets::ZERO {
        return Ok(width);
    }
    let after_left = add(
        "line alignment content width",
        TextNumericPhase::Finalization,
        width,
        -insets.line_left(),
    )?;
    add(
        "line alignment content width",
        TextNumericPhase::Finalization,
        after_left,
        -insets.line_right(),
    )
}

fn alignment_offset(
    selection: &LineContext,
    body_base: u8,
    width: f64,
    metrics: &LineMetrics,
) -> Result<f64> {
    let width = alignment_width(selection, width)?;
    let rtl = selection.line_box_direction == LineBoxDirection::RightToLeft;
    let mut offset = if rtl { 0.0 } else { selection.indent };
    // Standalone's trailing SPACE still belongs to the physical edge selected
    // by body bidi. Native supplies its measured alignment advance here with
    // trailing=0, then subtracts its actual physical hanging_start separately.
    // The independently resolved line-box basis must not flip either owner.
    if body_base & 1 != 0 {
        offset = add(
            "body trailing alignment",
            TextNumericPhase::Finalization,
            offset,
            -metrics.trailing_whitespace,
        )?;
    }
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
    let left = selection.alignment_insets.line_left();
    if left != 0.0 {
        offset = add(
            "line alignment inset offset",
            TextNumericPhase::Finalization,
            offset,
            left,
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

fn item_levels(
    presentation: &crate::protocol::prepared::SelectedPresentation,
    shaped: &ShapedData,
) -> Vec<u8> {
    item_levels_accounted(presentation, shaped, &mut None)
        .expect("ordinary levels have no scope ledger")
}
fn item_levels_accounted(
    presentation: &crate::protocol::prepared::SelectedPresentation,
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<Vec<u8>> {
    account::collect(
        ledger,
        shaped.items.iter().map(|item| match *item {
            ShapedItem::Cell(cell) => shaped.cells[cell].level,
            ShapedItem::Anchor { unit } => presentation.units[unit].line_level,
            ShapedItem::InlineBox { level, .. } => level,
            ShapedItem::TextCluster(i) => presentation
                .units
                .iter()
                .find(|u| u.atoms.contains(&shaped.clusters[i].atoms.start))
                .map_or(shaped.clusters[i].level, |u| u.line_level),
        }),
    )
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorCode::LayoutFailed, message)
}

/// CSS positioning follows the actual post-L1/L2 sequence. Only advance deltas
/// escape this temporary signed, unaligned frame.
fn resolve_tabs(
    selected: &SelectedData,
    shaped: &mut ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    ledger: &mut Ledger,
) -> Result<()> {
    if selected.prepared.options.tabs() != crate::TabPreparation::CssStops {
        return Ok(());
    }
    let intrinsic = selected.request.context.tabs == crate::TabContext::IntrinsicCssStops;
    let origin = match selected.request.context.tabs {
        crate::TabContext::CssStops { origin } => origin,
        crate::TabContext::IntrinsicCssStops => Some(0.0),
        _ => return Ok(()),
    };
    // The signed grid frame is a consumed capability, not another general
    // line-geometry pass. Absent and zero tabs cannot require its arithmetic.
    let mut remaining_tabs = 0;
    for item in &shaped.items {
        let ShapedItem::TextCluster(index) = *item else {
            continue;
        };
        let cluster = &shaped.clusters[index];
        if matches!(cluster.content, ClusterContent::CssTab(_)) {
            let metric = selected.prepared.tab_metrics[cluster.style]
                .as_ref()
                .ok_or_else(|| invalid("CSS tab has no prepared metric"))?;
            remaining_tabs += usize::from(metric.period != 0.0);
        }
    }
    if remaining_tabs == 0 {
        return Ok(());
    }
    let sign = if selected.presentation.analysis.base_level & 1 == 0 {
        1.0
    } else {
        -1.0
    };
    let grid_sign = if intrinsic { 1.0 } else { sign };
    let grid = crate::tabs::Grid::new(TextNumericPhase::Finalization);
    let mut pen = grid_sign * selected.request.context.indent;
    let mut ordinals = account::collect(ledger, order.iter().map(|(ordinal, _)| ordinal))?;
    if sign < 0.0 {
        ordinals.reverse();
    }
    for &ordinal in &ordinals {
        let advance = match shaped.items[ordinal] {
            ShapedItem::Cell(cell) => shaped.cells[cell].advance,
            ShapedItem::Anchor { unit } => {
                crate::protocol::boundary::advance(selected.presentation_input(), unit)
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                super::super::spacing::character::atomic_advance(
                    shaped,
                    occurrence,
                    inflow_allocation(atomics[&occurrence])?.0.advance,
                )?
            }
            ShapedItem::TextCluster(index) => {
                let cluster = &mut shaped.clusters[index];
                if matches!(cluster.content, ClusterContent::CssTab(_)) {
                    let metric = selected.prepared.tab_metrics[cluster.style]
                        .as_ref()
                        .ok_or_else(|| invalid("CSS tab has no prepared metric"))?;
                    let advance = if metric.period == 0.0 {
                        0.0
                    } else {
                        let origin = origin.ok_or_else(|| {
                            protocol_error(ProtocolError::MissingPhaseTwoContext(
                                crate::PhaseTwoContext::TabOrigin,
                            ))
                        })?;
                        grid.stop(origin, pen, grid_sign, metric.period, metric.threshold)?
                    };
                    cluster.advance = advance;
                    if metric.period != 0.0 {
                        remaining_tabs -= 1;
                        if remaining_tabs == 0 {
                            // This tab's resolved advance is the last consumed
                            // phase result. Its destination and later items use
                            // ordinary finalized geometry, not this grid frame.
                            break;
                        }
                    }
                }
                if cluster.whitespace == Whitespace::Newline {
                    0.0
                } else {
                    cluster.advance
                }
            }
        };
        pen = grid.add("tab traversal pen", pen, grid_sign * advance)?;
    }
    account::drop_vec(ledger, ordinals)?;
    Ok(())
}

fn same_paint_font(a: &PreparedRun, b: &PreparedRun) -> bool {
    a.font == b.font
        && a.font_size == b.font_size
        && a.font_attrs == b.font_attrs
        && a.synthesis == b.synthesis
        && a.coords == b.coords
        && a.glyph_basis == b.glyph_basis
        && a.metrics.font_origin_offset() == b.metrics.font_origin_offset()
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
    use icu_properties::props::BidiClass;

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
        let analyzed = crate::AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: style,
                segment_breaks: crate::DocumentSegmentBreaks::Defined(Vec::new()),
                paragraph_strut: ParagraphStrut::DefaultFont,
                base_direction: ParagraphBaseDirection::LeftToRight,
                origins: vec![ParagraphOrigin {
                    text_node: None,
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
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system
            .register_fonts(include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec())
            .unwrap();
        let analyzed = crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word);
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
                            fit: crate::AtomicFitAllocation::NativeOccupied,
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
            .map(|view| {
                let occurrence = view.handle();
                ControlAllocation {
                    boundary: matches!(
                        occurrence.view().kind(),
                        crate::ParagraphControlKind::InlineBoundaryStart
                            | crate::ParagraphControlKind::InlineBoundaryEnd
                    )
                    .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                    occurrence,
                    strut: None,
                }
            })
            .collect();
        prepared
            .select(LineSelectionRequest {
                segment: crate::test_wrapping::only_source_segment(&prepared),
                boundary_context: Vec::new(),
                incoming: None,
                start: boundaries[0].clone(),
                end: boundaries.last().unwrap().clone(),
                participants: prepared.participants().collect(),
                termination: crate::PreparedTermination::Ordinary(termination),
                alternative: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    line_box_direction: LineBoxDirection::LeftToRight,
                    formatted_line: crate::FormattedLineContext::default(),
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,

                    spacing: SpacingPolicy::Spaces,
                },
                atomics,
                controls,
                line_box_struts: Vec::new(),
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

    #[test]
    fn compatible_shaping_spans_keep_their_own_line_heights() {
        use crate::Range;

        let mut short = ahem_strut(10.0, 10.0);
        short.white_space = WhiteSpace::Collapse;
        let mut tall = short.clone();
        tall.line_height = crate::LineHeight::Absolute(20.0);
        let mut observations = Vec::new();
        for tall_first in [false, true] {
            let mut source = Source::new("X X");
            source.span(
                if tall_first {
                    Range::new(0, 1)
                } else {
                    Range::new(2, 3)
                },
                tall.clone(),
            );
            let selected = selected_with_source(source, short.clone(), None, None);
            let whole = selected
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            // Real Ahem has one-em advances, ascent8/descent2 at10px. Only
            // the authored line-height differs; both original X glyphs keep
            // their same face, size and source custody.
            let mut glyphs = whole
                .run_views()
                .flat_map(|run| {
                    assert_eq!(
                        run.face().bytes(),
                        include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf")
                    );
                    assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                    run.glyphs()
                        .map(|g| (g.range(), g.id(), g.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            glyphs.sort_by_key(|g| g.0.start);
            assert_eq!(
                glyphs,
                [
                    (Range::new(0, 1), 58, 10.0),
                    (Range::new(1, 2), 3, 10.0),
                    (Range::new(2, 3), 58, 10.0)
                ]
            );
            assert_eq!(whole.feedback().advance, 30.0);

            let prepared = whole.prepared_text();
            let offer = prepared
                .opportunity_views()
                .find(|offer| {
                    offer.kind() == PreparedBreakKind::Allowed
                        && offer.boundary().view().source_before().get() == 2
                })
                .unwrap()
                .handle();
            let mut request = selected.request().clone();
            request.end = offer.view().boundary();
            request.participants = prepared
                .participants()
                .take(request.end.view().participant_offset())
                .collect();
            request.termination = PreparedTermination::Ordinary(offer);
            request.context.end = LineEnd::Soft;
            let prefix = prepared
                .select(request)
                .unwrap()
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
                .unwrap();
            assert_eq!(prefix.feedback().advance, 10.0);
            observations.push((
                whole.feedback().baseline,
                whole.feedback().line_height,
                prefix.feedback().baseline,
                prefix.feedback().line_height,
            ));
        }
        // Both spans contribute their own half-leading: tall has13 before
        // and7 after baseline. The ordinary prefix sees only its actual X.
        // Qualify both orders before the regression assertion; taking the
        // initial shaping style's metrics incorrectly makes the first10px.
        assert_eq!(
            observations,
            [(13.0, 20.0, 8.0, 10.0), (13.0, 20.0, 13.0, 20.0)]
        );
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
                ..
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
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
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
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
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
        let analyzed = crate::AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
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
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
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
                segment: crate::test_wrapping::only_source_segment(&prepared),
                boundary_context: Vec::new(),
                incoming: None,
                start: prepared.boundary_views().next().unwrap().handle(),
                end: prepared.boundary_views().last().unwrap().handle(),
                participants: prepared.participants().collect(),
                termination: crate::PreparedTermination::Ordinary(termination),
                alternative: None,
                atomics: vec![AtomicAllocation {
                    occurrence: occurrence.clone(),
                    allocation: AtomicLineAllocation::InFlow {
                        fit: crate::AtomicFitAllocation::NativeOccupied,
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
                line_box_struts: Vec::new(),
                line_strut: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    line_box_direction: LineBoxDirection::LeftToRight,
                    formatted_line: crate::FormattedLineContext::default(),
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: Alignment::Start,
                    tabs: TabContext::NoTabs,

                    spacing: SpacingPolicy::None,
                },
            })
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(line.feedback().line_height, 0.0);
        assert_eq!(line.feedback().baseline, -7.0);
        let slot = line.atomic_slots().next().unwrap();
        assert_eq!(slot.origin, Point::new(0.0, -7.0));
        let placed = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
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
            })],
        })
        .unwrap();
        assert_eq!(placed.lines()[0].bounds().size.height, 0.0);
        assert_eq!(placed.bounds(), Rect::new(0.0, -10.0, 4.0, 10.0));
    }

    #[test]
    fn composed_collapse_suppresses_actual_line_edges_before_feedback_and_spacing() {
        let selected = selected("  X  ", WhiteSpace::Collapse, None, None);
        let line = selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
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
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::Complete(
                    slots
                        .into_iter()
                        .enumerate()
                        .map(|(index, slot)| SpacingAllocation {
                            slot,
                            amount: if index == 0 { 1.0 } else { 0.0 },
                        })
                        .collect(),
                ),
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap_err();
        assert!(matches!(
            error.detail(),
            Some(crate::ErrorDetail::InvalidTextProtocol {
                reason: ProtocolError::IneligibleSpacingSlot { slot: 0 }
            })
        ));
    }

    #[test]
    fn new_whitespace_modes_issue_distinct_edge_and_intrinsic_measurements() {
        qualify_en_quad_fixture();
        for wrap in [crate::Wrap::Word, crate::Wrap::None] {
            for (value, mode, raw, contribution) in [
                (" X ", WhiteSpace::PreserveBreaks, 10.0, 10.0),
                ("\u{1680}X\u{1680}", WhiteSpace::PreserveBreaks, 20.0, 20.0),
                ("X\u{2000}", WhiteSpace::PreserveBreaks, 15.0, 10.0),
                (" X ", WhiteSpace::BreakSpaces, 30.0, 30.0),
                ("\u{1680}X\u{1680}", WhiteSpace::BreakSpaces, 30.0, 30.0),
                ("X\u{2000}", WhiteSpace::BreakSpaces, 15.0, 15.0),
            ] {
                let selected = selected_with_source(
                    Source::new(value),
                    Style {
                        font: Font::new().family("Ahem"),
                        size: 10.0,
                        white_space: mode,
                        wrap,
                        ..Style::default()
                    },
                    None,
                    None,
                );
                let line = selected
                    .finalize(crate::LineAllocations {
                        spacing: SpacingAllocations::None,
                        scopes: crate::ScopeAllocations::None,
                    })
                    .unwrap();
                assert_eq!(line.feedback().advance, raw, "{value:?}/{mode:?}/{wrap:?}");
                assert_eq!(line.measurements().fit().advance(), contribution);
                assert_eq!(line.measurements().alignment().advance(), contribution);
                assert_eq!(
                    line.measurements().min_content_contribution().advance(),
                    contribution
                );
                assert_eq!(
                    line.measurements().max_content_contribution().advance(),
                    contribution
                );
            }
        }
    }

    #[test]
    fn ordinary_break_spaces_separator_stops_a_preceding_hanging_group() {
        qualify_en_quad_fixture();
        let mut source = Source::new("X\u{2000}\u{2000}");
        source.span(
            crate::Range::new(2, 3),
            Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                white_space: WhiteSpace::BreakSpaces,
                ..Style::default()
            },
        );
        let line = selected_with_source(
            source,
            Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                white_space: WhiteSpace::PreserveBreaks,
                ..Style::default()
            },
            None,
            None,
        )
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(line.measurements().fit().advance(), 20.0);
        assert_eq!(
            line.measurements().min_content_contribution().advance(),
            20.0
        );
        assert_eq!(
            line.measurements().max_content_contribution().advance(),
            20.0
        );
    }

    fn qualify_en_quad_fixture() {
        use skrifa::MetadataProvider;
        let bytes = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let font = skrifa::FontRef::new(bytes).unwrap();
        assert!(font.charmap().map(0x2000_u32).is_none());
        assert_eq!(font.charmap().map(0x20_u32).unwrap().to_u32(), 3);
        assert_eq!(
            crate::test_fonts::u16_at(crate::test_fonts::table(bytes, *b"head"), 18),
            1000
        );
        assert_eq!(
            crate::test_fonts::u16_at(crate::test_fonts::table(bytes, *b"hmtx"), 3 * 4),
            1000
        );
        // U+2000 EN QUAD has no nominal glyph here. The real space fallback
        // uses its Unicode half-em width: 500 design units = 5px at size10.
    }

    #[test]
    fn preserved_spaces_and_nbsp_keep_real_advances_at_selected_edges() {
        for (text, mode) in [
            (" X ", WhiteSpace::Preserve),
            ("\tX\t", WhiteSpace::PreserveSpaces),
            ("\u{a0}X\u{a0}", WhiteSpace::Collapse),
        ] {
            let line = selected(text, mode, None, None)
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
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
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
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
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
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
            include_control_struts(&mut metrics, [strut].into_iter(), None, true).unwrap();
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
                .finalize(crate::LineAllocations {
                    spacing: SpacingAllocations::None,
                    scopes: crate::ScopeAllocations::None,
                })
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
            true,
        )
        .unwrap();
        assert_eq!((metrics.baseline, metrics.line_height), (14.0, 24.0));
    }

    #[test]
    fn rtl_trailing_pdi_keeps_original_class_through_l1_l2_and_exact_placement() {
        use crate::{
            Affinity, AnalyzedSource, Cursor, Id, Layout, ParagraphConnection,
            ParagraphConnectionKind, Range,
        };

        // Native characterization, not a browser Range-coordinate oracle.
        // UAX #9 rev51 L1 uses ORIGINAL isolate classes in the line's trailing
        // sequence; L2 then orders the resulting levels. PDI therefore moves
        // from pre-L1 level2 to paragraph level1 without losing source4..5.
        // https://www.unicode.org/reports/tr9/tr9-51.html#L1
        let original = [0x58, 0x2067, 0x58, 0x58, 0x2069, 0x20, 0x58];
        let source = Source::identified("X\u{2067}XX\u{2069} X", Id::from_u64(1450), 1);
        let origins = [("X", 0, 1), ("\u{2067}XX\u{2069}", 1, 5), (" X", 5, 7)]
            .into_iter()
            .enumerate()
            .map(|(index, (text, start, end))| ParagraphOrigin {
                text_node: None,
                source: Source::identified(text, Id::from_u64(1451 + index as u64), 1),
                local: Range::new(0, end - start),
                aggregate: Range::new(start, end),
                kind: ParagraphOriginKind::Authored,
            })
            .collect();
        let analyzed = AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    white_space: WhiteSpace::Collapse,
                    wrap: crate::Wrap::Word,
                    ..ahem_strut(16.0, 20.0)
                },
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::RightToLeft,
                paragraph_strut: ParagraphStrut::DefaultFont,
                origins,
                connections: [1, 5]
                    .map(|at| ParagraphConnection {
                        at,
                        kind: ParagraphConnectionKind::Continuous,
                    })
                    .to_vec(),
                controls: [
                    (1, ParagraphControlKind::InlineBoundaryStart),
                    (5, ParagraphControlKind::InlineBoundaryEnd),
                ]
                .map(|(at, kind)| ParagraphControl { at, kind })
                .to_vec(),
                boundary_policies: (0..2)
                    .map(|index| ParagraphBoundaryPolicy {
                        definition: ParagraphBoundaryDefinition::Connection(index),
                        preceding_wrap: crate::Wrap::Word,
                        following_wrap: crate::Wrap::Word,
                        ancestor_wrap: crate::Wrap::Word,
                    })
                    .collect(),
                replacements: vec![],
                anchor_order: (0..2)
                    .map(|index| ParagraphAnchor {
                        item: ParagraphAnchorItem::Control(index),
                        affinity: Affinity::Before,
                    })
                    .collect(),
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.source().text().code_units(), original);
        assert_eq!(analyzed.inner.analysis.base_level, 1);
        assert_eq!(
            analyzed.inner.analysis.infos[4].0.bidi_class,
            BidiClass::PopDirectionalIsolate
        );
        assert_eq!(analyzed.inner.analysis.levels[4], 2);

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
        let isolate_origin = prepared.origin_views().nth(1).unwrap().handle();
        let soft = prepared
            .opportunity_views()
            .find(|offer| {
                offer.kind() == PreparedBreakKind::Allowed
                    && offer.boundary().view().source_before().get() == 6
            })
            .unwrap()
            .handle();
        let end = prepared
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let participants: Vec<_> = prepared.participants().collect();
        let split = soft.view().boundary().view().participant_offset();
        let select = |start, termination: PreparedOpportunity, from, to, first| {
            let participants = participants[from..to].to_vec();
            let controls = participants
                .iter()
                .filter_map(|participant| match participant {
                    PreparedParticipant::Control(control) => Some({
                        let occurrence = control.clone();
                        ControlAllocation {
                            boundary: matches!(
                                occurrence.view().kind(),
                                crate::ParagraphControlKind::InlineBoundaryStart
                                    | crate::ParagraphControlKind::InlineBoundaryEnd
                            )
                            .then_some(crate::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                            occurrence,
                            strut: None,
                        }
                    }),
                    _ => None,
                })
                .collect();
            crate::test_wrapping::select_neutral(
                &prepared,
                LineSelectionRequest {
                    segment: crate::test_wrapping::only_source_segment(&prepared),
                    boundary_context: Vec::new(),
                    incoming: None,
                    start,
                    end: termination.view().boundary(),
                    participants,
                    termination: crate::PreparedTermination::Ordinary(termination),
                    alternative: None,
                    context: LineContext {
                        alignment_insets: crate::LineAlignmentInsets::ZERO,
                        base_direction: ParagraphBaseDirection::RightToLeft,
                        line_box_direction: LineBoxDirection::RightToLeft,
                        formatted_line: crate::FormattedLineContext::default(),
                        orientation: LineOrientation::Horizontal,
                        start: if first {
                            LineStart::Paragraph
                        } else {
                            LineStart::Continuation
                        },
                        end: if first {
                            LineEnd::Soft
                        } else {
                            LineEnd::Paragraph
                        },
                        available_inline: Some(72.0),
                        indent: 0.0,
                        alignment: Alignment::Start,
                        tabs: TabContext::NoTabs,

                        spacing: SpacingPolicy::None,
                    },
                    atomics: vec![],
                    controls,
                    line_box_struts: Vec::new(),
                    line_strut: None,
                },
            )
            .unwrap()
        };
        let first_selected = select(
            prepared.boundary_views().next().unwrap().handle(),
            soft.clone(),
            0,
            split,
            true,
        );
        let second_selected = select(
            soft.view().boundary(),
            end,
            split,
            participants.len(),
            false,
        );
        let pdi_unit = first_selected
            .inner
            .presentation
            .units
            .iter()
            .find(|unit| unit.source == Range::new(4, 5))
            .unwrap();
        assert_eq!(pdi_unit.code_point, Some(0x2069));
        assert_eq!(pdi_unit.bidi_class, Some(BidiClass::PopDirectionalIsolate));
        assert_eq!(pdi_unit.level, 2);
        let (shaped, _, _) = shape::finalize(
            first_selected.inner.presentation_input(),
            &mut ShapeContext::default(),
            first_selected.inner.request.context.spacing,
            None,
            crate::engine::scope_alignment::PlanInput::Selection(&first_selected.inner.request),
        )
        .unwrap();
        let pdi_item = shaped.items.iter().position(|item| matches!(item,
            ShapedItem::TextCluster(index) if shaped.clusters[*index].source == Range::new(4, 5)
        )).unwrap();
        let ShapedItem::TextCluster(pdi_cluster) = shaped.items[pdi_item] else {
            unreachable!()
        };
        assert_eq!(shaped.clusters[pdi_cluster].level, 2);
        assert_eq!(shaped.clusters[pdi_cluster].advance, 0.0);
        assert_eq!(
            item_levels(&first_selected.inner.presentation, &shaped)[pdi_item],
            1
        );

        let first = first_selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let second = second_selected
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        assert_eq!(first.selected_line(), first_selected);
        assert_eq!(second.selected_line(), second_selected);
        assert_eq!(first.inner.selected.presentation.source, Range::new(0, 6));
        assert_eq!(second.inner.selected.presentation.source, Range::new(6, 7));
        assert_eq!(first.feedback().advance, 48.0);
        near(first.feedback().baseline, 14.8);
        assert_eq!(first.feedback().line_height, 20.0);
        // L2 gives text-source order [SPACE, PDI, firstX, RLI, X, X].
        // Suppressing the trailing SPACE leaves 48 units, right-aligned in72:
        // PDI/firstX begin at24, RLI/isolatedXX at40, and XX ends at72.
        let visual_sources: Vec<_> = first
            .inner
            .data
            .visual_order
            .iter()
            .filter_map(|item| match *item {
                FinalizedVisualItem::Cluster(index) => {
                    Some(first.inner.data.clusters[index].source)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            visual_sources,
            [
                Range::new(5, 6),
                Range::new(4, 5),
                Range::new(0, 1),
                Range::new(1, 2),
                Range::new(2, 3),
                Range::new(3, 4)
            ]
        );
        for (source_start, x, advance, level) in [
            (0, 24.0, 16.0, 2),
            (1, 40.0, 0.0, 2),
            (2, 40.0, 16.0, 4),
            (3, 56.0, 16.0, 4),
            (4, 24.0, 0.0, 1),
            (5, 24.0, 0.0, 1),
        ] {
            let cluster = first
                .inner
                .data
                .clusters
                .iter()
                .find(|row| row.source == Range::new(source_start, source_start + 1))
                .unwrap();
            assert_eq!(
                (cluster.x, cluster.advance, cluster.level),
                (x, advance, level)
            );
        }
        let fragment = first
            .fragments()
            .find(|fragment| {
                let view = fragment.view();
                view.owner() == FinalizedFragmentOwner::Origin(isolate_origin.clone())
                    && view.aggregate_range() == Some(Range::new(4, 5))
                    && view.local_range() == Some(Range::new(3, 4))
            })
            .unwrap();
        assert_eq!(fragment.finalized_line(), first);
        assert_eq!(fragment.view().aggregate_range(), Some(Range::new(4, 5)));
        assert_eq!(fragment.view().local_range(), Some(Range::new(3, 4)));
        let pdi_bounds = fragment.view().bounds().unwrap();
        assert_eq!((pdi_bounds.origin.x, pdi_bounds.size.width), (24.0, 0.0));
        assert_eq!(isolate_origin.view().aggregate_range(), Range::new(1, 5));
        assert_eq!(isolate_origin.view().local_range(), Range::new(0, 4));
        assert_eq!(
            isolate_origin.view().source().text().code_units(),
            [0x2067, 0x58, 0x58, 0x2069]
        );

        let request = PlacementRequest {
            prepared: prepared.clone(),
            coverage: PlacementCoverage::WholeParagraph,
            entries: [first.clone(), second.clone()]
                .into_iter()
                .enumerate()
                .map(|(index, line)| {
                    crate::PlacementEntry::Formatted(LinePlacement {
                        anchor: LineAnchor {
                            inline_origin: 0.0,
                            baseline: index as f64 * 20.0 + line.feedback().baseline,
                        },
                        line,
                        orientation: LineOrientation::Horizontal,
                        atomics: vec![],
                    })
                })
                .collect(),
        };
        drop(system);
        let placed = Layout::from_finalized_lines(request.clone()).unwrap();
        assert_eq!(placed.source(), &source);
        assert_eq!(placed.analyzed_source().identity(), analyzed.identity());
        assert!(Arc::ptr_eq(&placed.inner.prepared, &prepared.inner));
        assert!(Arc::ptr_eq(&placed.inner.lines[0].finalized, &first.inner));
        assert!(Arc::ptr_eq(&placed.inner.lines[1].finalized, &second.inner));
        assert_eq!(
            placed
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            [Range::new(0, 6), Range::new(6, 7)]
        );
        let visible: Vec<_> = placed
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().copied())
            .filter(|glyph| glyph.advance() > 0.0)
            .map(|glyph| (glyph.range(), glyph.id(), glyph.advance()))
            .collect();
        assert_eq!(
            visible,
            [
                (Range::new(0, 1), 58, 16.0),
                (Range::new(2, 3), 58, 16.0),
                (Range::new(3, 4), 58, 16.0),
                (Range::new(6, 7), 58, 16.0)
            ]
        );
        let pdi_position = placed
            .positions()
            .find(|position| position.source_cursor() == Cursor::new(4, Affinity::After))
            .unwrap();
        assert_eq!(
            placed
                .position_geometry(&pdi_position)
                .unwrap()
                .rect()
                .origin
                .x,
            24.0
        );
        assert_eq!(
            placed.clone().position_geometry(&pdi_position).unwrap(),
            placed.position_geometry(&pdi_position).unwrap()
        );
        let other_placement = Layout::from_finalized_lines(request).unwrap();
        assert!(!Arc::ptr_eq(&placed.inner, &other_placement.inner));
        assert!(other_placement.position_geometry(&pdi_position).is_err());
        assert_eq!(fragment.finalized_line(), first);
        assert_eq!(analyzed.source().text().code_units(), original);
    }
}
