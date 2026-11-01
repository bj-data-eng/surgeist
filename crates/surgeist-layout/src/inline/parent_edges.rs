use super::{
    FlowAxes, InlineLineInput, LayoutScalar, MixedInlineParticipantOf, SelectedInlineLineOf,
};
use crate::inline_wrapping::{NumericWrapping, NumericWrappingError, WrappingScopeError};
use std::ops::Range;
use surgeist_text::SuppliedLine;

#[derive(Clone, Copy)]
struct Edge {
    unit: usize,
    owner: usize,
    left: bool,
    gap: usize,
    rank: usize,
}

fn buffer<T>(count: usize) -> Result<Vec<T>, NumericWrappingError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count)
        .map_err(|_| NumericWrappingError::Allocation)?;
    Ok(rows)
}

fn invalid() -> NumericWrappingError {
    WrappingScopeError::UnsupportedIncidence.into()
}

/// Two borrowed views of the same actual numeric units. Candidate retention
/// comes from that slice's existing discard summary, never a future endpoint.
enum Line<'a, S: LayoutScalar> {
    Selected(&'a SelectedInlineLineOf<S>),
    Candidate {
        input: InlineLineInput<'a, S>,
        retained: Range<usize>,
    },
}
impl<S: LayoutScalar> Line<'_, S> {
    fn len(&self) -> usize {
        match self {
            Self::Selected(line) => line.units.len(),
            Self::Candidate { input, .. } => input.participants.len(),
        }
    }
    fn source_start(&self) -> usize {
        match self {
            Self::Selected(line) => line.source_start,
            Self::Candidate { input, .. } => input.source_start,
        }
    }
    fn participant(&self, unit: usize) -> MixedInlineParticipantOf<S> {
        match self {
            Self::Selected(line) => line.units[unit].participant,
            Self::Candidate { input, .. } => input.participants[unit],
        }
    }
    fn discarded(&self, unit: usize) -> bool {
        match self {
            Self::Selected(line) => line.units[unit].discarded,
            Self::Candidate { retained, .. } => !retained.contains(&unit),
        }
    }
    fn boxed(&self, unit: usize) -> bool {
        matches!(self.participant(unit),
            MixedInlineParticipantOf::Boundary(control) if control.parent_edge_axes().is_some())
    }
    fn has_box_edges(&self) -> bool {
        (0..self.len()).any(|unit| self.boxed(unit))
    }
}

/// Used flow in the same canonical packet/edge order as selected placement.
/// Ordinary scalar runs retain their existing accumulator and allocate nothing.
pub(super) fn canonical_flow<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    retained: Range<usize>,
    axes: FlowAxes,
    wrapping: Option<&NumericWrapping>,
) -> Result<Option<S>, NumericWrappingError> {
    let line = Line::Candidate { input, retained };
    if !line.has_box_edges() {
        return Ok(None);
    }
    let mut order = buffer(line.len())?;
    order.resize(line.len(), 0);
    SuppliedLine::new(input.supplied)
        .reorder_into(&mut order)
        .map_err(|_| invalid())?;
    let mut flow = S::ZERO;
    schedule(&line, axes, wrapping, &mut order, |unit| {
        if !line.discarded(unit) {
            // Preserve the actual S arithmetic; only its semantic traversal
            // changes. Positioned margin recipients retain their own checks.
            flow = flow + line.participant(unit).inline_advance(axes);
            if !flow.is_finite() {
                return Err(NumericWrappingError::Geometry {
                    value: flow.to_f64(),
                    reason: crate::LayoutTextNumericError::NonFinite,
                });
            }
        }
        Ok(())
    })?;
    Ok(Some(flow))
}

/// Numeric units are already the host's indivisible native packets. The
/// source cursor and admitted native pair interval, rather than bidi levels
/// or equal numeric coordinates, identify each packet's actual box incidence.
pub(super) fn reorder<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    axes: FlowAxes,
    wrapping: Option<&NumericWrapping>,
    order: &mut Vec<usize>,
) -> Result<(), NumericWrappingError> {
    let line = Line::Selected(line);
    if !line.has_box_edges() {
        return Ok(());
    }
    schedule(&line, axes, wrapping, order, |_| Ok(()))
}

fn schedule<S: LayoutScalar>(
    line: &Line<'_, S>,
    axes: FlowAxes,
    wrapping: Option<&NumericWrapping>,
    order: &mut Vec<usize>,
    mut emit: impl FnMut(usize) -> Result<(), NumericWrappingError>,
) -> Result<(), NumericWrappingError> {
    let boxed = |unit: usize| line.boxed(unit);
    let wrapping = wrapping.ok_or_else(invalid)?;
    let mut payload = buffer(order.len())?;
    for &unit in order.iter() {
        if !boxed(unit) {
            payload.push(unit);
        }
    }
    let mut edges = buffer(order.len())?;
    let positive_side = FlowAxes::new(axes.writing_mode(), crate::Direction::Ltr).inline_start();
    for (rank, &unit) in order.iter().enumerate() {
        let MixedInlineParticipantOf::Boundary(control) = line.participant(unit) else {
            continue;
        };
        let Some(parent) = control.parent_edge_axes() else {
            continue;
        };
        let source = line.source_start().checked_add(unit).ok_or_else(invalid)?;
        let endpoint = match control.kind() {
            crate::InlineBoundaryKind::Start => 0,
            crate::InlineBoundaryKind::End => 1,
        };
        let owner = wrapping
            .boxes()
            .iter()
            .position(|row| row.endpoints[endpoint] == Some(source))
            .ok_or_else(invalid)?;
        let row = &wrapping.boxes()[owner];
        let first = row.first + usize::from(row.endpoints[0].is_some());
        let after = row.after - usize::from(row.endpoints[1].is_some());
        let mut extent = None;
        for (position, &candidate) in payload.iter().enumerate() {
            let actual = line
                .source_start()
                .checked_add(candidate)
                .ok_or_else(invalid)?;
            if first <= actual && actual < after {
                extent = Some(extent.map_or((position, position), |(lo, _)| (lo, position)));
            }
        }
        let start_left = if parent.inline_start() == positive_side {
            true
        } else if parent.inline_end() == positive_side {
            false
        } else {
            return Err(invalid());
        };
        let left = start_left == (endpoint == 0);
        let gap = if let Some((first, last)) = extent {
            if left { first } else { last + 1 }
        } else {
            // A real empty native pair is anchored by its actual two control
            // slots. Both must name the same gap after BoxEdges are removed.
            let gap = order[..rank]
                .iter()
                .filter(|&&candidate| !boxed(candidate))
                .count();
            for other in row.endpoints.into_iter().flatten() {
                let Some(local) = other
                    .checked_sub(line.source_start())
                    .filter(|&at| at < line.len())
                else {
                    continue;
                };
                let at = order
                    .iter()
                    .position(|&candidate| candidate == local)
                    .ok_or_else(invalid)?;
                if order[..at]
                    .iter()
                    .filter(|&&candidate| !boxed(candidate))
                    .count()
                    != gap
                {
                    return Err(invalid());
                }
            }
            gap
        };
        edges.push(Edge {
            unit,
            owner,
            left,
            gap,
            rank,
        });
    }
    let mut emitted = buffer(edges.len())?;
    emitted.resize(edges.len(), false);
    let before = |a: Edge, b: Edge| {
        if a.gap != b.gap {
            return false;
        }
        let aa = wrapping.boxes()[a.owner].ancestry;
        let bb = wrapping.boxes()[b.owner].ancestry;
        let related = wrapping.box_ancestor(aa, bb) || wrapping.box_ancestor(bb, aa);
        if related {
            if a.left != b.left {
                a.left
            } else if a.owner == b.owner {
                a.rank < b.rank
            } else if a.left {
                wrapping.box_ancestor(aa, bb)
            } else {
                wrapping.box_ancestor(bb, aa)
            }
        } else {
            a.rank < b.rank
        }
    };
    order.clear();
    for gap in 0..=payload.len() {
        loop {
            let mut ready = None;
            let mut pending = false;
            for (index, &edge) in edges.iter().enumerate() {
                if emitted[index] || edge.gap != gap {
                    continue;
                }
                pending = true;
                if edges.iter().enumerate().any(|(other, &candidate)| {
                    !emitted[other] && other != index && before(candidate, edge)
                }) {
                    continue;
                }
                if ready.is_none_or(|previous: usize| edges[previous].rank > edge.rank) {
                    ready = Some(index);
                }
            }
            let Some(index) = ready else {
                if pending {
                    return Err(invalid());
                }
                break;
            };
            emitted[index] = true;
            order.push(edges[index].unit);
            emit(edges[index].unit)?;
        }
        if let Some(&unit) = payload.get(gap) {
            order.push(unit);
            emit(unit)?;
        }
    }
    if order.len() != line.len() {
        return Err(invalid());
    }
    Ok(())
}

/// Execute in the existing native logical S frame. Only the new nonzero
/// margin recipients add loss checks; ordinary BP rounding is preserved.
pub(super) fn execute<S: LayoutScalar>(
    control: super::InlineBoundaryControlOf<S>,
    incoming: S,
    axes: FlowAxes,
) -> Result<(super::ResolvedInlineSlot<S>, S), NumericWrappingError> {
    let edge = control.edge();
    let margin = edge.margin();
    let extent = edge.border_padding_advance();
    let flow = edge.flow_advance();
    let rtl = axes.direction() == crate::Direction::Rtl;
    let end = if rtl {
        incoming - flow
    } else {
        incoming + flow
    };
    if margin == S::ZERO {
        return Ok((
            super::ResolvedInlineSlot {
                inline_start: if rtl { end } else { incoming },
                margin_flow: None,
            },
            end,
        ));
    }
    let check = |value: S| {
        if value.is_finite() {
            Ok(value)
        } else {
            Err(NumericWrappingError::Geometry {
                value: value.to_f64(),
                reason: crate::LayoutTextNumericError::NonFinite,
            })
        }
    };
    check(end)?;
    if flow != S::ZERO && end == incoming {
        return Err(NumericWrappingError::Geometry {
            value: flow.to_f64(),
            reason: crate::LayoutTextNumericError::NonZeroLost,
        });
    }
    let positive = FlowAxes::new(axes.writing_mode(), crate::Direction::Ltr).inline_start();
    let parent = control
        .parent_edge_axes()
        .expect("explicit native box edge retains parent basis");
    let side = match control.kind() {
        crate::InlineBoundaryKind::Start => parent.inline_start(),
        crate::InlineBoundaryKind::End => parent.inline_end(),
    };
    let left = side == positive;
    let start = if rtl {
        let before_strip_end = if left {
            check(incoming - margin)?
        } else {
            incoming
        };
        check(before_strip_end - extent)?
    } else {
        let start = if left {
            check(incoming + margin)?
        } else {
            incoming
        };
        check(start + extent)?;
        start
    };
    let collapsed = if left {
        if rtl {
            incoming == check(start + extent)?
        } else {
            incoming == start
        }
    } else if rtl {
        start == end
    } else {
        check(start + extent)? == end
    };
    if collapsed {
        return Err(NumericWrappingError::Geometry {
            value: margin.to_f64(),
            reason: crate::LayoutTextNumericError::NonZeroLost,
        });
    }
    Ok((
        super::ResolvedInlineSlot {
            inline_start: start,
            margin_flow: Some((incoming, end)),
        },
        end,
    ))
}
