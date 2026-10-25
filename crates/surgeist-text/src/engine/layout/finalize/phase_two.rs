//! Native CSS Text Phase II (Trimming and Positioning) over the actual line.
use super::*;
use crate::protocol::measurements::EdgeRow;
use crate::{InlineMeasure, LineEdgeDisposition as Edge, LineEdgeSide as Side, LineMeasurements};

pub(super) struct Natural {
    rows: Vec<NaturalRow>,
}

/// A neutral row can defer structural authority without claiming an ordinary
/// classification. Each dependent row keeps its own obligation across spacing.
#[derive(Clone, Copy)]
enum NaturalRow {
    Known {
        edge: Edge,
        minimum: bool,
        maximum: bool,
    },
    DeferredTerminal {
        cause: TerminalCause,
        side: Side,
    },
}

#[derive(Clone, Copy)]
enum TerminalCause {
    Missing,
    NonBlock,
}
impl TerminalCause {
    fn error(self) -> crate::Error {
        protocol_error(match self {
            Self::Missing => {
                ProtocolError::MissingPhaseTwoContext(crate::PhaseTwoContext::TerminalBlockEdge)
            }
            Self::NonBlock => ProtocolError::UnsupportedContext {
                field: crate::ProtocolContextField::CssParagraphBoundary,
            },
        })
    }
}
impl NaturalRow {
    fn known(edge: Edge, minimum: bool, maximum: bool) -> Self {
        Self::Known {
            edge,
            minimum,
            maximum,
        }
    }
    fn edge(self) -> Edge {
        match self {
            Self::Known { edge, .. } => edge,
            Self::DeferredTerminal { side, .. } => Edge::UnconsumedTerminal { side },
        }
    }
    fn minimum(self) -> bool {
        matches!(self, Self::Known { minimum: true, .. })
    }
    fn maximum(self) -> bool {
        matches!(self, Self::Known { maximum: true, .. })
    }
}

fn separator(selected: &SelectedData, cluster: &PreparedCluster) -> bool {
    use icu_properties::{CodePointMapData, props::GeneralCategory};
    selected.presentation.projection.atoms[cluster.atoms.clone()]
        .iter()
        .all(|atom| {
            matches!(atom.code_point, 0x20 | 0x09)
                || (atom.code_point != 0xa0
                    && CodePointMapData::<GeneralCategory>::new().get32(atom.code_point)
                        == GeneralCategory::SpaceSeparator)
        })
}

fn transparent(selected: &SelectedData, cluster: &PreparedCluster) -> bool {
    cluster.whitespace == Whitespace::Newline
        || selected.presentation.analysis.infos[cluster.atoms.clone()]
            .iter()
            .all(|(info, _)| !info.contributes_to_shaping())
}

fn positions(
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
) -> Result<(Vec<(f64, f64)>, f64)> {
    let mut result = vec![(0.0, 0.0); shaped.clusters.len()];
    let mut pen = 0.0;
    for (ordinal, _) in order.iter() {
        match shaped.items[ordinal] {
            ShapedItem::TextCluster(index) => {
                let advance = if shaped.clusters[index].whitespace == Whitespace::Newline {
                    0.0
                } else {
                    shaped.clusters[index].advance
                };
                let end = add(
                    "Phase II text end",
                    TextNumericPhase::Finalization,
                    pen,
                    advance,
                )?;
                result[index] = (pen, end);
                pen = end;
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                pen = add(
                    "Phase II atomic end",
                    TextNumericPhase::Finalization,
                    pen,
                    inflow_allocation(atomics[&occurrence])?.0.advance,
                )?;
            }
            ShapedItem::Anchor { .. } => {}
        }
    }
    Ok((result, pen))
}

/// Coordinates increase towards the paragraph's end. The same physical start
/// frame is used for natural and allocated passes; indentation consumes capacity
/// once and alignment is a later translation.
fn towards(pair: (f64, f64), total: f64, rtl: bool) -> Result<(f64, f64)> {
    if rtl {
        Ok((
            add(
                "Phase II RTL start",
                TextNumericPhase::Finalization,
                total,
                -pair.1,
            )?,
            add(
                "Phase II RTL end",
                TextNumericPhase::Finalization,
                total,
                -pair.0,
            )?,
        ))
    } else {
        Ok(pair)
    }
}

fn available_edge(selected: &SelectedData) -> Result<Option<f64>> {
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

pub(super) fn natural(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    dispositions: &[SpacingSlotDisposition],
) -> Result<Natural> {
    let (positions, total) = positions(shaped, atomics, order)?;
    let rtl = selected.presentation.analysis.base_level & 1 != 0;
    let side = if rtl { Side::Start } else { Side::End };
    let mut result = Natural {
        rows: vec![NaturalRow::known(Edge::Ordinary, false, false); shaped.clusters.len()],
    };
    for (index, disposition) in dispositions.iter().enumerate() {
        if *disposition == SpacingSlotDisposition::DiscardedAtLineEdge {
            result.rows[index] = NaturalRow::known(Edge::Removed, false, false);
        }
    }
    let mut visual: Vec<_> = order.iter().map(|(ordinal, _)| ordinal).collect();
    if !rtl {
        visual.reverse();
    }
    let mut terminal = true;
    let mut maximum_terminal = true;
    let mut deferred: Option<TerminalCause> = None;
    let terminal_cause = match selected.prepared.analyzed.paragraph.terminal_edge {
        crate::ParagraphTerminalEdge::Unknown => Some(TerminalCause::Missing),
        crate::ParagraphTerminalEdge::NonBlockBoundary => Some(TerminalCause::NonBlock),
        crate::ParagraphTerminalEdge::BlockEnd => None,
    };
    for ordinal in visual {
        let index = match shaped.items[ordinal] {
            ShapedItem::Anchor { .. } => continue,
            ShapedItem::InlineBox { .. } => break,
            ShapedItem::TextCluster(index) => index,
        };
        let cluster = &shaped.clusters[index];
        if result.rows[index].edge() == Edge::Removed {
            continue;
        }
        if !separator(selected, cluster) {
            if transparent(selected, cluster) {
                continue;
            }
            break;
        }
        let style = selected.prepared.analyzed.styles[cluster.style]
            .style
            .authored();
        // Discard's surviving characters and BreakSpaces separators are
        // ordinary flow. Preceding modes cannot hang through this retained
        // ordinary edge, including when it has zero advance.
        if matches!(
            style.white_space,
            crate::WhiteSpace::Discard | crate::WhiteSpace::BreakSpaces
        ) {
            break;
        }
        if style.white_space.collapses_spaces() {
            if let Some(cause) = deferred {
                if cluster.advance != 0.0 {
                    return Err(cause.error());
                }
                result.rows[index] = NaturalRow::DeferredTerminal { cause, side };
                continue;
            }
            result.rows[index] = NaturalRow::known(
                Edge::Unconditional { side, terminal },
                true,
                maximum_terminal,
            );
        } else if style.wrap != crate::Wrap::None {
            let conditional = match selected.request.context.end {
                LineEnd::Soft => false,
                LineEnd::Forced => true,
                LineEnd::Paragraph => match selected.prepared.analyzed.paragraph.terminal_edge {
                    crate::ParagraphTerminalEdge::BlockEnd => true,
                    _ => {
                        let cause = terminal_cause.expect("non-block terminal has a cause");
                        if cluster.advance != 0.0 {
                            return Err(cause.error());
                        }
                        deferred = Some(cause);
                        result.rows[index] = NaturalRow::DeferredTerminal { cause, side };
                        continue;
                    }
                },
            };
            if conditional {
                let (u, v) = towards(positions[index], total, rtl)?;
                let eligible =
                    u != v && available_edge(selected)?.is_some_and(|edge| u.max(v) > edge);
                result.rows[index] =
                    NaturalRow::known(Edge::Conditional { side, eligible }, true, false);
                terminal &= eligible;
                maximum_terminal = false;
            } else {
                result.rows[index] = NaturalRow::known(
                    Edge::Unconditional { side, terminal },
                    true,
                    maximum_terminal,
                );
            }
        } else {
            break;
        }
    }
    Ok(result)
}

fn include(bounds: &mut Option<(f64, f64)>, start: f64, end: f64) {
    let (start, end) = (start.min(end), start.max(end));
    *bounds = Some(bounds.map_or((start, end), |(old_start, old_end)| {
        (old_start.min(start), old_end.max(end))
    }));
}

/// Omitted rows retain actual paint/source geometry. Only measurement
/// contributions and occupied intervals are folded here; no glyph is rewritten.
fn measure(
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    omit: &[bool],
    excluded: Option<&[f64]>,
    clip: Option<(bool, f64)>,
) -> Result<InlineMeasure> {
    let mut advance = 0.0;
    for item in &shaped.items {
        let contribution = match *item {
            ShapedItem::TextCluster(i)
                if !omit[i] && shaped.clusters[i].whitespace != Whitespace::Newline =>
            {
                add(
                    "Phase II retained contribution",
                    TextNumericPhase::Finalization,
                    shaped.clusters[i].advance,
                    -excluded.map_or(0.0, |rows| rows[i]),
                )?
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                inflow_allocation(atomics[&occurrence])?.0.advance
            }
            _ => 0.0,
        };
        advance = add(
            "Phase II signed measure",
            TextNumericPhase::Finalization,
            advance,
            contribution,
        )?;
    }
    let mut bounds = None;
    let mut pen = 0.0;
    for (ordinal, _) in order.iter() {
        match shaped.items[ordinal] {
            ShapedItem::TextCluster(i) => {
                let cluster = &shaped.clusters[i];
                let end = add(
                    "Phase II occupied text",
                    TextNumericPhase::Finalization,
                    pen,
                    cluster.advance,
                )?;
                if !omit[i] && cluster.whitespace != Whitespace::Newline && pen != end {
                    let (mut start, mut finish) = (pen.min(end), pen.max(end));
                    if excluded.is_some_and(|rows| rows[i] != 0.0)
                        && let Some((rtl, edge)) = clip
                    {
                        if rtl {
                            start = start.max(edge);
                        } else {
                            finish = finish.min(edge);
                        }
                    }
                    if start < finish {
                        include(&mut bounds, start, finish);
                    }
                }
                pen = end;
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                let flow = inflow_allocation(atomics[&occurrence])?.0;
                include(
                    &mut bounds,
                    add(
                        "Phase II occupied atomic",
                        TextNumericPhase::Finalization,
                        pen,
                        flow.occupied_start,
                    )?,
                    add(
                        "Phase II occupied atomic",
                        TextNumericPhase::Finalization,
                        pen,
                        flow.occupied_end,
                    )?,
                );
                pen = add(
                    "Phase II atomic flow",
                    TextNumericPhase::Finalization,
                    pen,
                    flow.advance,
                )?;
            }
            ShapedItem::Anchor { .. } => {}
        }
    }
    let (occupied_start, occupied_end) = bounds.unwrap_or((0.0, 0.0));
    Ok(InlineMeasure {
        advance,
        occupied_start,
        occupied_end,
    })
}

pub(super) fn allocated(
    selected: &SelectedData,
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
    natural: Natural,
    metrics: &mut LineMetrics,
) -> Result<LineMeasurements> {
    // Validate the exact deferred dependencies after all actual allocations
    // and tab recomputation. Cancellation between rows cannot erase custody.
    for (row, cluster) in natural.rows.iter().zip(&shaped.clusters) {
        if let NaturalRow::DeferredTerminal { cause, .. } = row
            && cluster.advance != 0.0
        {
            return Err(cause.error());
        }
    }
    let (positions, total) = positions(shaped, atomics, order)?;
    let rtl = selected.presentation.analysis.base_level & 1 != 0;
    // Ordinary and zero-only rows consume no hanging frame. Preserve checked
    // precision when a real conditional or outward-overlap observation needs it.
    let needs_edge = shaped.clusters.iter().enumerate().any(|(index, cluster)| {
        cluster.advance != 0.0
            && matches!(
                natural.rows[index].edge(),
                Edge::Unconditional { .. } | Edge::Conditional { .. }
            )
    });
    let edge = if needs_edge {
        available_edge(selected)?
    } else {
        None
    };
    let mut omit = vec![false; shaped.clusters.len()];
    let mut exclusions = vec![0.0; shaped.clusters.len()];
    let mut rows = Vec::with_capacity(shaped.clusters.len());
    for (index, cluster) in shaped.clusters.iter().enumerate() {
        let (excluded, overlap) = match natural.rows[index].edge() {
            Edge::Removed => {
                omit[index] = true;
                (0.0, 0.0)
            }
            Edge::Unconditional { terminal: true, .. } => {
                omit[index] = true;
                (cluster.advance, 0.0)
            }
            Edge::Conditional { eligible, .. } => {
                omit[index] = true;
                if eligible && cluster.advance != 0.0 {
                    let b = edge.ok_or_else(|| invalid("natural conditional frame was lost"))?;
                    let (u, v) = towards(positions[index], total, rtl)?;
                    let u = add(
                        "Phase II conditional start",
                        TextNumericPhase::Finalization,
                        u,
                        -b,
                    )?
                    .max(0.0);
                    let v = add(
                        "Phase II conditional end",
                        TextNumericPhase::Finalization,
                        v,
                        -b,
                    )?
                    .max(0.0);
                    let signed = add(
                        "Phase II signed exclusion",
                        TextNumericPhase::Finalization,
                        v,
                        -u,
                    )?;
                    (signed, signed.abs())
                } else {
                    (0.0, 0.0)
                }
            }
            _ => (0.0, 0.0),
        };
        exclusions[index] = excluded;
        let hanging = if rtl {
            &mut metrics.hanging_start
        } else {
            &mut metrics.hanging_end
        };
        *hanging = add(
            "Phase II actual hanging",
            TextNumericPhase::Finalization,
            *hanging,
            excluded,
        )?;
        rows.push(EdgeRow {
            disposition: natural.rows[index].edge(),
            atoms: cluster.atoms.clone(),
            excluded,
            overlap,
        });
    }
    let fit = measure(shaped, atomics, order, &omit, None, None)?;
    let minimum = measure(
        shaped,
        atomics,
        order,
        &natural
            .rows
            .iter()
            .map(|row| row.minimum())
            .collect::<Vec<_>>(),
        None,
        None,
    )?;
    let maximum = measure(
        shaped,
        atomics,
        order,
        &natural
            .rows
            .iter()
            .map(|row| row.maximum())
            .collect::<Vec<_>>(),
        None,
        None,
    )?;
    let removed: Vec<_> = natural
        .rows
        .iter()
        .map(|row| {
            matches!(
                row.edge(),
                Edge::Removed | Edge::Unconditional { terminal: true, .. }
            )
        })
        .collect();
    let clip = edge.map(|edge| (rtl, if rtl { total - edge } else { edge }));
    let alignment = measure(shaped, atomics, order, &removed, Some(&exclusions), clip)?;
    // Physical overlap is an intersection, not the omitted signed advance.
    // With no supplied span the actual Text metric frame is purpose-neutral
    // and follows the alignment measure, as finalization will place it.
    let physical_edge = match (needs_edge, edge) {
        (false, _) => None,
        (true, Some(edge)) => Some(edge),
        (true, None) => {
            let width = add(
                "Phase II metric indentation",
                TextNumericPhase::Finalization,
                alignment.advance(),
                selected.request.context.indent.max(0.0),
            )?
            .max(0.0);
            Some(add(
                "Phase II metric edge",
                TextNumericPhase::Finalization,
                width,
                -selected.request.context.indent,
            )?)
        }
    };
    for (index, row) in rows.iter_mut().enumerate() {
        if let Some(physical_edge) = physical_edge
            && shaped.clusters[index].advance != 0.0
            && matches!(
                row.disposition,
                Edge::Conditional { .. } | Edge::Unconditional { .. }
            )
        {
            let (u, v) = towards(positions[index], total, rtl)?;
            let start = add(
                "Phase II overlap start",
                TextNumericPhase::Finalization,
                u,
                -physical_edge,
            )?
            .max(0.0);
            let end = add(
                "Phase II overlap end",
                TextNumericPhase::Finalization,
                v,
                -physical_edge,
            )?
            .max(0.0);
            row.overlap = add(
                "Phase II physical overlap",
                TextNumericPhase::Finalization,
                end,
                -start,
            )?
            .abs();
        }
    }
    Ok(LineMeasurements {
        fit,
        alignment,
        minimum,
        maximum,
        edges: rows,
    })
}

pub(super) fn standalone(
    metrics: &LineMetrics,
    shaped: &ShapedData,
    atomics: &BTreeMap<usize, &AtomicAllocation>,
    order: &crate::SuppliedVisualOrder<'_, '_>,
) -> Result<LineMeasurements> {
    let raw = measure(
        shaped,
        atomics,
        order,
        &vec![false; shaped.clusters.len()],
        None,
        None,
    )?;
    let alignment = InlineMeasure {
        advance: add(
            "standalone alignment measure",
            TextNumericPhase::Finalization,
            metrics.advance,
            -metrics.trailing_whitespace,
        )?,
        ..raw
    };
    Ok(LineMeasurements {
        fit: raw,
        alignment,
        minimum: alignment,
        maximum: alignment,
        edges: Vec::new(),
    })
}
