use super::{FlowAxes, LayoutScalar, MixedInlineParticipantOf, SelectedInlineLineOf};
use crate::inline_wrapping::{NumericWrapping, NumericWrappingError, WrappingScopeError};

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

/// Numeric units are already the host's indivisible native packets. The
/// source cursor and admitted native pair interval, rather than bidi levels
/// or equal numeric coordinates, identify each packet's actual box incidence.
pub(super) fn reorder<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    axes: FlowAxes,
    wrapping: Option<&NumericWrapping>,
    order: &mut Vec<usize>,
) -> Result<(), NumericWrappingError> {
    let boxed = |unit: usize| {
        matches!(line.units[unit].participant,
        MixedInlineParticipantOf::Boundary(control) if control.parent_edge_axes().is_some())
    };
    if !line.units.iter().enumerate().any(|(index, _)| boxed(index)) {
        return Ok(());
    }
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
        let MixedInlineParticipantOf::Boundary(control) = line.units[unit].participant else {
            continue;
        };
        let Some(parent) = control.parent_edge_axes() else {
            continue;
        };
        let source = line.source_start.checked_add(unit).ok_or_else(invalid)?;
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
                .source_start
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
                    .checked_sub(line.source_start)
                    .filter(|&at| at < line.units.len())
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
        }
        if let Some(&unit) = payload.get(gap) {
            order.push(unit);
        }
    }
    if order.len() != line.units.len() {
        return Err(invalid());
    }
    Ok(())
}
