//! Shared natural/allocated hanging over actual signed contributions.
use super::*;
use crate::protocol::measurements::EdgeRow;
use crate::{
    FormattedLineUsage, HangingPunctuationEnd, InlineMeasure, LineEdgeDisposition as Edge,
    LineEdgeSide as Side, LineMeasurements,
};

mod punctuation;
const FIT: usize = 0;
const ALIGN: usize = 1;
const MIN: usize = 2;
const MAX: usize = 3;

pub(super) struct Natural {
    causes: Vec<Cause>,
    removed: Vec<bool>,
    raw_trim: Vec<bool>,
    deferred: Vec<Option<(crate::SpacingDeferredCause, Side)>>,
    usage: FormattedLineUsage,
    capacity: Option<Option<f64>>,
    positions: Vec<(f64, f64)>,
    total: f64,
}
struct Cause {
    rows: Vec<usize>,
    side: Side,
    effects: [bool; 4],
    conditional: bool,
    eligible: bool,
    punctuation: Option<crate::PresentationRange>,
    first: bool,
}

fn separator(selected: &SelectedData, cluster: &PreparedCluster) -> bool {
    use icu_properties::{CodePointMapData, props::GeneralCategory};
    selected.presentation.projection.atoms[cluster.atoms.clone()]
        .iter()
        .all(|atom| {
            matches!(atom.code_point, 0x20 | 0x09)
                || atom.code_point != 0xa0
                    && CodePointMapData::<GeneralCategory>::new().get32(atom.code_point)
                        == GeneralCategory::SpaceSeparator
        })
}
fn transparent(selected: &SelectedData, cluster: &PreparedCluster) -> bool {
    cluster.whitespace == Whitespace::Newline
        || selected.presentation.analysis.infos[cluster.atoms.clone()]
            .iter()
            .all(|(info, _)| !info.contributes_to_shaping())
}
fn positions(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    ledger: &mut Ledger,
) -> Result<(Vec<(f64, f64)>, f64)> {
    account::work(ledger, shaped.items.len())?;
    let mut rows = account::filled(ledger, shaped.clusters.len(), (0.0, 0.0))?;
    let mut pen = 0.0;
    for (ordinal, _) in order.iter() {
        let advance = match shaped.items[ordinal] {
            ShapedItem::Cell(cell) => shaped.cells[cell].extent,
            ShapedItem::TextCluster(i) => {
                let advance = if shaped.clusters[i].whitespace == Whitespace::Newline {
                    0.0
                } else {
                    shaped.clusters[i].advance
                };
                let end = add(
                    "edge text end",
                    TextNumericPhase::Finalization,
                    pen,
                    advance,
                )?;
                rows[i] = (pen, end);
                pen = end;
                continue;
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                super::super::super::spacing::character::atomic_advance(
                    shaped,
                    occurrence,
                    inflow_allocation(atomics[&occurrence])?.0.advance,
                )?
            }
            ShapedItem::Anchor { unit } => {
                crate::protocol::boundary::advance(selected.presentation_input(), unit)
            }
        };
        pen = add(
            "edge mixed end",
            TextNumericPhase::Finalization,
            pen,
            advance,
        )?;
    }
    Ok((rows, pen))
}
fn capacity(selected: &SelectedData) -> Result<Option<f64>> {
    selected
        .request
        .context
        .available_inline
        .map(|width| {
            add(
                "Phase II available indentation",
                TextNumericPhase::Finalization,
                width,
                -selected.request.context.indent,
            )
        })
        .transpose()
}
fn natural_capacity(
    cached: &mut Option<Option<f64>>,
    selected: &SelectedData,
) -> Result<Option<f64>> {
    if let Some(value) = *cached {
        return Ok(value);
    }
    let value = capacity(selected)?;
    *cached = Some(value);
    Ok(value)
}
fn physical_edge(side: Side, capacity: f64, total: f64) -> Result<f64> {
    if side == Side::Start {
        add(
            "physical edge",
            TextNumericPhase::Finalization,
            total,
            -capacity,
        )
    } else {
        Ok(capacity)
    }
}
fn overflows(side: Side, rows: &[usize], positions: &[(f64, f64)], edge: Option<f64>) -> bool {
    edge.is_some_and(|edge| {
        rows.iter().any(|&i| {
            let (a, b) = positions[i];
            a != b
                && if side == Side::Start {
                    a.min(b) < edge
                } else {
                    a.max(b) > edge
                }
        })
    })
}
fn raw_trailing(
    selected: &SelectedData,
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<Vec<bool>> {
    let mut rows = account::filled(ledger, shaped.clusters.len(), false)?;
    for item in shaped.items.iter().rev() {
        match *item {
            ShapedItem::Anchor { unit } => {
                if crate::protocol::boundary::advance(selected.presentation_input(), unit) != 0.0 {
                    break;
                }
            }
            ShapedItem::TextCluster(i) if shaped.clusters[i].is_whitespace => rows[i] = true,
            _ => break,
        }
    }
    Ok(rows)
}
fn role(selected: &SelectedData, first: bool, usage: &mut FormattedLineUsage) -> Result<bool> {
    let context = selected.request.context.formatted_line;
    if first {
        usage.first = true
    } else {
        usage.last = true
    }
    (if first {
        context.first()
    } else {
        context.last()
    })
    .ok_or_else(|| {
        protocol_error(ProtocolError::MissingPhaseTwoContext(if first {
            crate::PhaseTwoContext::FirstFormattedLine
        } else {
            crate::PhaseTwoContext::LastFormattedLine
        }))
    })
}

pub(super) fn natural(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    dispositions: &[SpacingSlotDisposition],
    ledger: &mut Ledger,
) -> Result<Natural> {
    let (positions, total) = positions(selected, shaped, atomics, order, ledger)?;
    let native = selected.metric_policy == LineMetricPolicy::Native;
    let body_side = if selected.presentation.analysis.base_level & 1 != 0 {
        Side::Start
    } else {
        Side::End
    };
    let start_side = if selected.request.context.line_box_direction == LineBoxDirection::LeftToRight
    {
        Side::Start
    } else {
        Side::End
    };
    let end_side = if start_side == Side::Start {
        Side::End
    } else {
        Side::Start
    };
    let units = punctuation::units(selected, ledger)?;
    let mut row_unit = account::filled(ledger, shaped.clusters.len(), None)?;
    let mut unit_rows = account::filled(ledger, units.len(), Vec::new())?;
    for (i, cluster) in shaped.clusters.iter().enumerate() {
        let unit = units.partition_point(|unit| unit.atoms.end <= cluster.atoms.start);
        if units.get(unit).is_some_and(|unit| {
            unit.atoms.start <= cluster.atoms.start && cluster.atoms.end <= unit.atoms.end
        }) {
            row_unit[i] = Some(unit);
            account::push(ledger, &mut unit_rows[unit], i)?;
        }
    }
    let mut result = Natural {
        causes: Vec::new(),
        removed: account::collect(
            ledger,
            dispositions
                .iter()
                .map(|d| *d == SpacingSlotDisposition::DiscardedAtLineEdge),
        )?,
        raw_trim: if native {
            account::filled(ledger, shaped.clusters.len(), false)?
        } else {
            raw_trailing(selected, shaped, ledger)?
        },
        deferred: account::filled(ledger, shaped.clusters.len(), None)?,
        usage: FormattedLineUsage {
            first: false,
            last: false,
        },
        capacity: None,
        positions: Vec::new(),
        total,
    };
    let mut covered = account::filled(ledger, units.len(), false)?;
    for side in [start_side, end_side] {
        let mut visual = account::collect(ledger, order.iter().map(|(ordinal, _)| ordinal))?;
        if side == Side::End {
            visual.reverse()
        }
        let mut terminal = [true; 4];
        let mut deferred: Option<crate::SpacingDeferredCause> = None;
        let mut quota = false;
        let mut handled = account::filled(ledger, units.len(), false)?;
        for &ordinal in &visual {
            let row = match shaped.items[ordinal] {
                ShapedItem::Anchor { unit } => {
                    if crate::protocol::boundary::advance(selected.presentation_input(), unit) > 0.0
                    {
                        break;
                    }
                    continue;
                }
                ShapedItem::Cell(_) | ShapedItem::InlineBox { .. } => break,
                ShapedItem::TextCluster(row) => row,
            };
            if result.removed[row] {
                continue;
            }
            if let Some(unit) = row_unit[row] {
                if handled[unit] {
                    continue;
                }
                let tcu = &units[unit];
                let declaration = selected.prepared.analyzed.styles[tcu.style]
                    .style
                    .authored()
                    .hanging_punctuation;
                let mut permission = None;
                let mut first = false;
                if !quota {
                    if side == start_side && tcu.first && declaration.first() {
                        if role(selected, true, &mut result.usage)? {
                            permission = Some(false);
                            first = true
                        }
                    } else if side == end_side {
                        if covered[unit] {
                            handled[unit] = true;
                            quota = true;
                            continue;
                        }
                        let stop = tcu.stop.then(|| declaration.end()).flatten();
                        if stop == Some(HangingPunctuationEnd::ForceEnd)
                            || tcu.last
                                && declaration.last()
                                && role(selected, false, &mut result.usage)?
                        {
                            permission = Some(false)
                        } else if stop == Some(HangingPunctuationEnd::AllowEnd) {
                            permission = Some(true)
                        }
                    }
                }
                if let Some(conditional) = permission {
                    let rows = account::collect(
                        ledger,
                        unit_rows[unit]
                            .iter()
                            .copied()
                            .filter(|&i| !result.removed[i]),
                    )?;
                    let edge =
                        if conditional && rows.iter().any(|&i| shaped.clusters[i].advance != 0.0) {
                            natural_capacity(&mut result.capacity, selected)?
                        } else {
                            None
                        }
                        .map(|b| physical_edge(side, b, total))
                        .transpose()?;
                    let eligible = conditional && overflows(side, &rows, &positions, edge);
                    let effects = if conditional {
                        [true, eligible, true, false]
                    } else {
                        terminal
                    };
                    if let Some(cause) = deferred {
                        for &i in &rows {
                            if shaped.clusters[i].advance != 0.0 {
                                return Err(cause.error());
                            }
                            result.deferred[i] = Some((cause, side));
                        }
                        account::drop_vec(ledger, rows)?;
                    } else {
                        if first && effects.into_iter().all(|value| value) {
                            covered[unit] = true
                        }
                        account::push(
                            ledger,
                            &mut result.causes,
                            Cause {
                                rows,
                                side,
                                effects,
                                conditional,
                                eligible,
                                punctuation: Some(tcu.presentation),
                                first,
                            },
                        )?;
                    }
                    handled[unit] = true;
                    quota = true;
                    if conditional {
                        terminal[FIT] &= eligible;
                        terminal[ALIGN] &= eligible;
                        terminal[MAX] = false
                    }
                    continue;
                }
            }
            if result.raw_trim[row] {
                continue;
            }
            let cluster = &shaped.clusters[row];
            if side != body_side || !native || !separator(selected, cluster) {
                if transparent(selected, cluster) {
                    continue;
                }
                break;
            }
            let style = selected.prepared.analyzed.styles[cluster.style]
                .style
                .authored();
            if matches!(
                style.white_space,
                crate::WhiteSpace::Discard | crate::WhiteSpace::BreakSpaces
            ) {
                break;
            }
            let conditional = if style.white_space.collapses_spaces() {
                false
            } else if style.wrap == crate::Wrap::None {
                break;
            } else {
                match selected.request.context.end {
                    LineEnd::Soft => false,
                    LineEnd::Forced => true,
                    LineEnd::Paragraph => {
                        match selected.prepared.analyzed.paragraph.terminal_edge {
                            crate::ParagraphTerminalEdge::BlockEnd => true,
                            crate::ParagraphTerminalEdge::Unknown => {
                                deferred =
                                    Some(crate::SpacingDeferredCause::MissingTerminalBlockEdge);
                                false
                            }
                            crate::ParagraphTerminalEdge::NonBlockBoundary => {
                                deferred =
                                    Some(crate::SpacingDeferredCause::NonBlockParagraphBoundary);
                                false
                            }
                        }
                    }
                }
            };
            if let Some(cause) = deferred {
                if cluster.advance != 0.0 {
                    return Err(cause.error());
                }
                result.deferred[row] = Some((cause, side));
                continue;
            }
            let edge = if conditional && cluster.advance != 0.0 {
                natural_capacity(&mut result.capacity, selected)?
            } else {
                None
            }
            .map(|b| physical_edge(side, b, total))
            .transpose()?;
            let eligible = conditional && overflows(side, &[row], &positions, edge);
            let cause_rows = account::filled(ledger, 1, row)?;
            account::push(
                ledger,
                &mut result.causes,
                Cause {
                    rows: cause_rows,
                    side,
                    effects: if conditional {
                        [true, eligible, true, false]
                    } else {
                        terminal
                    },
                    conditional,
                    eligible,
                    punctuation: None,
                    first: false,
                },
            )?;
            if conditional {
                terminal[FIT] &= eligible;
                terminal[ALIGN] &= eligible;
                terminal[MAX] = false
            }
        }
        account::drop_vec(ledger, visual)?;
        account::drop_vec(ledger, handled)?;
    }
    for rows in &mut unit_rows {
        account::drop_vec(ledger, std::mem::take(rows))?;
    }
    account::drop_vec(ledger, unit_rows)?;
    account::drop_vec(ledger, row_unit)?;
    account::drop_vec(ledger, covered)?;
    account::drop_vec(ledger, units)?;
    result.positions = positions;
    Ok(result)
}

#[derive(Clone, Copy, Default)]
struct Exclusion {
    flow: bool,
    occupied: bool,
    left: Option<f64>,
    right: Option<f64>,
    punctuation_start: bool,
    raw_trim: bool,
}
impl Exclusion {
    fn clip(self, pair: (f64, f64)) -> Option<(f64, f64)> {
        if self.occupied {
            return None;
        }
        let start = self
            .left
            .map_or(pair.0.min(pair.1), |edge| pair.0.min(pair.1).max(edge));
        let end = self
            .right
            .map_or(pair.0.max(pair.1), |edge| pair.0.max(pair.1).min(edge));
        (start < end).then_some((start, end))
    }
    fn excluded(self, pair: (f64, f64), advance: f64) -> Result<f64> {
        if self.flow {
            return Ok(advance);
        }
        if self.left.is_none() && self.right.is_none() {
            return Ok(0.0);
        }
        if self.clip(pair).is_none() {
            return Ok(advance);
        }
        let project = |value: f64| {
            self.right
                .map_or(self.left.map_or(value, |edge| value.max(edge)), |edge| {
                    self.left.map_or(value, |left| value.max(left)).min(edge)
                })
        };
        let before = add(
            "edge signed interval",
            TextNumericPhase::Finalization,
            pair.1,
            -pair.0,
        )?;
        let retained = add(
            "edge retained interval",
            TextNumericPhase::Finalization,
            project(pair.1),
            -project(pair.0),
        )?;
        add(
            "edge signed exclusion",
            TextNumericPhase::Finalization,
            before,
            -retained,
        )
    }
    fn partial(&mut self, side: Side, edge: f64) {
        match side {
            Side::Start => self.left = Some(self.left.map_or(edge, |old| old.max(edge))),
            Side::End => self.right = Some(self.right.map_or(edge, |old| old.min(edge))),
        }
    }
}
fn include(bounds: &mut Option<(f64, f64)>, start: f64, end: f64) {
    let (start, end) = (start.min(end), start.max(end));
    *bounds = Some(bounds.map_or((start, end), |(a, b)| (a.min(start), b.max(end))));
}
#[derive(Clone, Copy)]
struct Baseline {
    /// The existing raw profile folds total and trailing whitespace separately.
    /// Preserve that exact arithmetic, then apply additional union exclusions.
    raw_advance: Option<f64>,
    fit_anchor: bool,
}
fn measure(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    positions: &[(f64, f64)],
    exclusions: &[Exclusion],
    baseline: Baseline,
    ledger: &mut Ledger,
) -> Result<InlineMeasure> {
    account::work(ledger, shaped.items.len())?;
    let mut advance = baseline.raw_advance.unwrap_or(0.0);
    let mut bounds = None;
    let mut anchor = 0.0;
    for item in &shaped.items {
        let contribution = match *item {
            ShapedItem::Cell(cell) => {
                if baseline.raw_advance.is_some() {
                    0.0
                } else {
                    shaped.cells[cell].extent
                }
            }
            ShapedItem::TextCluster(i) if shaped.clusters[i].whitespace != Whitespace::Newline => {
                let excluded = exclusions[i].excluded(positions[i], shaped.clusters[i].advance)?;
                if baseline.fit_anchor && exclusions[i].punctuation_start {
                    anchor = add(
                        "raw punctuation origin",
                        TextNumericPhase::Finalization,
                        anchor,
                        excluded,
                    )?
                }
                if baseline.raw_advance.is_some() {
                    let prior = if exclusions[i].raw_trim {
                        shaped.clusters[i].advance
                    } else {
                        0.0
                    };
                    let extra = add(
                        "raw additional exclusion",
                        TextNumericPhase::Finalization,
                        excluded,
                        -prior,
                    )?;
                    -extra
                } else {
                    add(
                        "retained attributed advance",
                        TextNumericPhase::Finalization,
                        shaped.clusters[i].advance,
                        -excluded,
                    )?
                }
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                if baseline.raw_advance.is_some() {
                    0.0
                } else {
                    super::super::super::spacing::character::atomic_advance(
                        shaped,
                        occurrence,
                        inflow_allocation(atomics[&occurrence])?.0.advance,
                    )?
                }
            }
            ShapedItem::Anchor { unit } => {
                if baseline.raw_advance.is_some() {
                    0.0
                } else {
                    crate::protocol::boundary::advance(selected.presentation_input(), unit)
                }
            }
            _ => 0.0,
        };
        advance = add(
            "edge signed flow",
            TextNumericPhase::Finalization,
            advance,
            contribution,
        )?;
    }
    if baseline.fit_anchor {
        include(&mut bounds, anchor, anchor)
    }
    let mut pen = 0.0;
    // Finalization has already admitted this exact selected permutation. A
    // measurement view changes exclusions, never the retained traversal.
    for &ordinal in &selected.basis.order {
        let contribution = match shaped.items[ordinal] {
            ShapedItem::Cell(cell) => {
                let extent = shaped.cells[cell].extent;
                include(
                    &mut bounds,
                    pen,
                    add(
                        "occupied cell end",
                        TextNumericPhase::Finalization,
                        pen,
                        extent,
                    )?,
                );
                extent
            }
            ShapedItem::TextCluster(i) => {
                if shaped.clusters[i].whitespace != Whitespace::Newline
                    && let Some((a, b)) = exclusions[i].clip(positions[i])
                {
                    include(&mut bounds, a, b)
                }
                if shaped.clusters[i].whitespace == Whitespace::Newline {
                    0.0
                } else {
                    shaped.clusters[i].advance
                }
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let flow = inflow_allocation(atomics[&occurrence])?.0;
                include(
                    &mut bounds,
                    add(
                        "occupied atomic start",
                        TextNumericPhase::Finalization,
                        pen,
                        flow.occupied_start,
                    )?,
                    add(
                        "occupied atomic end",
                        TextNumericPhase::Finalization,
                        pen,
                        flow.occupied_end,
                    )?,
                );
                super::super::super::spacing::character::atomic_advance(
                    shaped,
                    occurrence,
                    flow.advance,
                )?
            }
            ShapedItem::Anchor { unit } => {
                let n = crate::protocol::boundary::advance(selected.presentation_input(), unit);
                if n != 0.0 {
                    include(
                        &mut bounds,
                        pen,
                        add("occupied boundary", TextNumericPhase::Finalization, pen, n)?,
                    )
                }
                n
            }
        };
        pen = add(
            "occupied edge flow",
            TextNumericPhase::Finalization,
            pen,
            contribution,
        )?;
    }
    let (occupied_start, occupied_end) = bounds.unwrap_or((0.0, 0.0));
    Ok(InlineMeasure {
        advance,
        occupied_start,
        occupied_end,
    })
}

/// Response attribution uses the same actual natural causes/exclusion algebra
/// as the allocated views. No font metric or source-class guess enters here.
pub(super) fn spacing_responses(
    selected: &SelectedData,
    shaped: &ShapedData,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    natural: &Natural,
    ledger: &mut Ledger,
) -> Result<Vec<(crate::SpacingStretchability, usize)>> {
    use crate::SpacingStretchability as Response;
    let mut exclusions = account::filled(ledger, shaped.clusters.len(), Exclusion::default())?;
    for (i, exclusion) in exclusions.iter_mut().enumerate() {
        exclusion.flow = natural.removed[i] || natural.raw_trim[i];
    }
    for cause in &natural.causes {
        if !cause.effects[ALIGN] {
            continue;
        }
        for &i in &cause.rows {
            if cause.conditional {
                if let Some(capacity) = natural.capacity.flatten() {
                    exclusions[i].partial(
                        cause.side,
                        physical_edge(cause.side, capacity, natural.total)?,
                    );
                }
            } else {
                exclusions[i].flow = true;
            }
        }
    }
    let mut result = account::collect(
        ledger,
        exclusions.iter().enumerate().map(|(i, exclusion)| {
            let response = if let Some((cause, _)) = natural.deferred[i] {
                Response::Deferred(cause)
            } else if exclusion.flow
                || exclusion
                    .left
                    .is_some_and(|edge| natural.positions[i].0 <= edge)
                || exclusion
                    .right
                    .is_some_and(|edge| natural.positions[i].1 >= edge)
            {
                Response::Excluded
            } else {
                Response::Direct
            };
            (response, 0)
        }),
    )?;
    let mut traversal = account::collect(ledger, order.iter().map(|(ordinal, _)| ordinal))?;
    if selected.presentation.analysis.base_level & 1 != 0 {
        traversal.reverse();
    }
    let mut intercepted = false;
    for (rank, &ordinal) in traversal.iter().enumerate().rev() {
        if let ShapedItem::TextCluster(index) = shaped.items[ordinal] {
            result[index].1 = rank;
            if matches!(shaped.clusters[index].content, ClusterContent::CssTab(_))
                && selected.prepared.tab_metrics[shaped.clusters[index].style]
                    .as_ref()
                    .is_some_and(|metric| metric.period != 0.0)
                && result[index].0 != Response::Excluded
            {
                intercepted = true;
            } else if intercepted && result[index].0 == Response::Direct {
                result[index].0 = Response::TabDependent;
            }
        }
    }
    account::drop_vec(ledger, traversal)?;
    account::drop_vec(ledger, exclusions)?;
    Ok(result)
}

pub(super) fn allocated(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &OrdinalMap<&AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    natural: Natural,
    metrics: &mut LineMetrics,
    ledger: &mut Ledger,
) -> Result<LineMeasurements> {
    for (i, cause) in natural.deferred.iter().enumerate() {
        if let Some((cause, _)) = cause
            && shaped.clusters[i].advance != 0.0
        {
            return Err(cause.error());
        }
    }
    let native = selected.metric_policy == LineMetricPolicy::Native;
    let body_side = if selected.presentation.analysis.base_level & 1 != 0 {
        Side::Start
    } else {
        Side::End
    };
    let (positions, total) = positions(selected, shaped, atomics, order, ledger)?;
    let needs_edge = natural.causes.iter().any(|cause| {
        cause
            .rows
            .iter()
            .any(|&i| shaped.clusters[i].advance != 0.0)
    });
    let capacity = if needs_edge {
        match natural.capacity {
            Some(value) => value,
            None => capacity(selected)?,
        }
    } else {
        None
    };
    let mut plans = [
        account::filled(ledger, shaped.clusters.len(), Exclusion::default())?,
        account::filled(ledger, shaped.clusters.len(), Exclusion::default())?,
        account::filled(ledger, shaped.clusters.len(), Exclusion::default())?,
        account::filled(ledger, shaped.clusters.len(), Exclusion::default())?,
    ];
    let mut rows = account::collect(
        ledger,
        shaped
            .clusters
            .iter()
            .enumerate()
            .map(|(i, cluster)| EdgeRow {
                disposition: if natural.removed[i] {
                    Edge::Removed
                } else if let Some((_, side)) = natural.deferred[i] {
                    Edge::UnconsumedTerminal { side }
                } else if natural.raw_trim[i] {
                    Edge::Unconditional {
                        side: body_side,
                        terminal: true,
                    }
                } else {
                    Edge::Ordinary
                },
                atoms: cluster.atoms.clone(),
                excluded: 0.0,
                overlap: 0.0,
                punctuation_unit: None,
            }),
    )?;
    let mut priority = account::filled(ledger, rows.len(), 0)?;
    let mut placement = account::filled(ledger, rows.len(), body_side)?;
    for i in 0..rows.len() {
        for (view, plan) in plans.iter_mut().enumerate() {
            if natural.removed[i] {
                plan[i].flow = true;
                plan[i].occupied = true
            }
            if natural.raw_trim[i] {
                plan[i].flow = true;
                plan[i].occupied |= view == FIT;
                plan[i].raw_trim = true;
            }
        }
    }
    for cause in &natural.causes {
        let edge = capacity
            .map(|b| physical_edge(cause.side, b, total))
            .transpose()?;
        if cause.conditional {
            let edge = natural
                .capacity
                .flatten()
                .map(|b| physical_edge(cause.side, b, natural.total))
                .transpose()?;
            debug_assert_eq!(
                cause.eligible,
                overflows(cause.side, &cause.rows, &natural.positions, edge)
            );
        }
        let rank = (if cause.effects[ALIGN] { 4 } else { 0 })
            + (if cause.first {
                3
            } else if cause.punctuation.is_some() {
                2
            } else {
                1
            });
        for &i in &cause.rows {
            for (view, plan) in plans.iter_mut().enumerate() {
                if cause.effects[view] {
                    if cause.conditional && view == ALIGN && shaped.clusters[i].advance != 0.0 {
                        plan[i].partial(
                            cause.side,
                            edge.ok_or_else(|| invalid("natural conditional frame was lost"))?,
                        )
                    } else {
                        plan[i].flow = true;
                        plan[i].occupied = true
                    }
                    if cause.punctuation.is_some() && cause.side == Side::Start {
                        plan[i].punctuation_start = true
                    }
                }
            }
            if rank > priority[i] {
                priority[i] = rank;
                rows[i].disposition = if cause.conditional {
                    Edge::Conditional {
                        side: cause.side,
                        eligible: cause.eligible,
                    }
                } else {
                    Edge::Unconditional {
                        side: cause.side,
                        terminal: cause.effects[ALIGN],
                    }
                };
                rows[i].punctuation_unit = cause.punctuation;
                if cause.effects[ALIGN] {
                    placement[i] = cause.side
                }
            }
        }
    }
    if !native {
        if body_side == Side::Start {
            metrics.hanging_start = metrics.trailing_whitespace;
        } else {
            metrics.hanging_end = metrics.trailing_whitespace;
        }
    }
    for (i, row) in rows.iter_mut().enumerate() {
        row.excluded = plans[ALIGN][i].excluded(positions[i], shaped.clusters[i].advance)?;
        let baseline = if natural.raw_trim[i] {
            shaped.clusters[i].advance
        } else {
            0.0
        };
        if !native && placement[i] != body_side && baseline != 0.0 {
            let old = if body_side == Side::Start {
                &mut metrics.hanging_start
            } else {
                &mut metrics.hanging_end
            };
            *old = add(
                "raw baseline edge transfer",
                TextNumericPhase::Finalization,
                *old,
                -baseline,
            )?;
        }
        let delta = if !native && placement[i] == body_side {
            add(
                "raw allocated edge delta",
                TextNumericPhase::Finalization,
                row.excluded,
                -baseline,
            )?
        } else {
            row.excluded
        };
        let hanging = if placement[i] == Side::Start {
            &mut metrics.hanging_start
        } else {
            &mut metrics.hanging_end
        };
        *hanging = add(
            "actual signed hanging",
            TextNumericPhase::Finalization,
            *hanging,
            delta,
        )?;
    }
    let baseline = Baseline {
        raw_advance: if native {
            None
        } else {
            Some(add(
                "standalone alignment measure",
                TextNumericPhase::Finalization,
                metrics.advance,
                -metrics.trailing_whitespace,
            )?)
        },
        fit_anchor: false,
    };
    let fit = measure(
        selected,
        shaped,
        atomics,
        &positions,
        &plans[FIT],
        Baseline {
            fit_anchor: !native,
            ..baseline
        },
        ledger,
    )?;
    let minimum = measure(
        selected,
        shaped,
        atomics,
        &positions,
        &plans[MIN],
        baseline,
        ledger,
    )?;
    let maximum = measure(
        selected,
        shaped,
        atomics,
        &positions,
        &plans[MAX],
        baseline,
        ledger,
    )?;
    let alignment = measure(
        selected,
        shaped,
        atomics,
        &positions,
        &plans[ALIGN],
        baseline,
        ledger,
    )?;
    let physical_capacity = if needs_edge {
        Some(match capacity {
            Some(capacity) => capacity,
            None => {
                let width = add(
                    "Phase II metric indentation",
                    TextNumericPhase::Finalization,
                    alignment.advance(),
                    selected.request.context.indent.max(0.0),
                )?
                .max(0.0);
                add(
                    "Phase II metric edge",
                    TextNumericPhase::Finalization,
                    width,
                    -selected.request.context.indent,
                )?
            }
        })
    } else {
        None
    };
    for (i, row) in rows.iter_mut().enumerate() {
        if matches!(
            row.disposition,
            Edge::Conditional { .. } | Edge::Unconditional { .. }
        ) && shaped.clusters[i].advance != 0.0
        {
            let side = match row.disposition {
                Edge::Conditional { side, .. } | Edge::Unconditional { side, .. } => side,
                _ => unreachable!(),
            };
            let Some(capacity) = physical_capacity else {
                continue;
            };
            let edge = physical_edge(side, capacity, total)?;
            let (a, b) = positions[i];
            let (start, end) = match side {
                Side::Start => (a.min(b).min(edge), a.max(b).min(edge)),
                Side::End => (a.min(b).max(edge), a.max(b).max(edge)),
            };
            row.overlap = add(
                "outward overlap",
                TextNumericPhase::Finalization,
                end,
                -start,
            )?
            .max(0.0);
        }
    }
    let formatted_usage = natural.usage;
    for plan in plans {
        account::drop_vec(ledger, plan)?;
    }
    account::drop_vec(ledger, positions)?;
    account::drop_vec(ledger, priority)?;
    account::drop_vec(ledger, placement)?;
    drop_natural(ledger, natural)?;
    Ok(LineMeasurements {
        fit,
        alignment,
        minimum,
        maximum,
        edges: rows,
        formatted_usage,
    })
}

fn drop_natural(ledger: &mut Ledger, mut natural: Natural) -> Result<()> {
    for cause in &mut natural.causes {
        account::drop_vec(ledger, std::mem::take(&mut cause.rows))?;
    }
    account::drop_vec(ledger, natural.causes)?;
    account::drop_vec(ledger, natural.removed)?;
    account::drop_vec(ledger, natural.raw_trim)?;
    account::drop_vec(ledger, natural.deferred)?;
    account::drop_vec(ledger, natural.positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_physical_exclusions_union_once_and_preserve_signed_attribution() {
        let mut exclusion = Exclusion::default();
        exclusion.partial(Side::Start, 8.0);
        exclusion.partial(Side::End, 12.0);
        assert_eq!(exclusion.clip((5.0, 15.0)), Some((8.0, 12.0)));
        assert_eq!(exclusion.excluded((5.0, 15.0), 10.0).unwrap(), 6.0);
        assert_eq!(exclusion.clip((15.0, 5.0)), Some((8.0, 12.0)));
        assert_eq!(exclusion.excluded((15.0, 5.0), -10.0).unwrap(), -6.0);
        exclusion.partial(Side::Start, 13.0);
        assert_eq!(exclusion.clip((5.0, 15.0)), None);
        assert_eq!(exclusion.excluded((5.0, 15.0), 10.0).unwrap(), 10.0);
        assert_eq!(exclusion.excluded((15.0, 5.0), -10.0).unwrap(), -10.0);
    }

    #[test]
    fn raw_baseline_flow_trimming_keeps_its_separate_occupied_observation() {
        let flow_only = Exclusion {
            flow: true,
            ..Exclusion::default()
        };
        assert_eq!(flow_only.excluded((0.0, 10.0), 10.0).unwrap(), 10.0);
        assert_eq!(flow_only.clip((0.0, 10.0)), Some((0.0, 10.0)));
        let removed = Exclusion {
            occupied: true,
            ..flow_only
        };
        assert_eq!(removed.clip((0.0, 10.0)), None);
    }
}
