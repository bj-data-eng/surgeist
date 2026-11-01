use super::{
    FlowAxes, InlineLineInput, LayoutScalar, MixedInlineParticipantOf, SelectedInlineLineOf,
};
use crate::inline_wrapping::{
    NativeOrderingProjection, NativeOrderingSelection, NumericWrapping, NumericWrappingError,
    WrappingScopeError,
};
use std::{ops::Range, sync::Arc};
use surgeist_text::SuppliedLine;

#[derive(Clone, Copy)]
struct Edge {
    unit: usize,
    owner: usize,
    ancestry: usize,
    alias: bool,
    left: bool,
    gap: usize,
    rank: usize,
    empty: bool,
}
/// Successful ordering is retained with the exact native run and selected cut.
/// Geometry consumes this receipt; it does not repeat a fenced L2 pass.
#[derive(Debug, PartialEq)]
pub(super) struct OrderReceipt {
    context: Arc<NativeOrderingProjection>,
    source_start: usize,
    descriptor_start: usize,
    order: Vec<usize>,
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
        matches!(self.participant(unit), MixedInlineParticipantOf::Boundary(control) if control.parent_edge_axes().is_some())
    }
    fn has_box_edges(&self) -> bool {
        (0..self.len()).any(|unit| self.boxed(unit))
    }
}

pub(super) fn canonical_flow<S: LayoutScalar>(
    input: InlineLineInput<'_, S>,
    retained: Range<usize>,
    axes: FlowAxes,
    wrapping: Option<&NumericWrapping>,
) -> Result<Option<(S, Arc<OrderReceipt>)>, NumericWrappingError> {
    let line = Line::Candidate { input, retained };
    if !line.has_box_edges()
        && !wrapping
            .and_then(NumericWrapping::ordering)
            .is_some_and(|c| c.profile.has_family())
    {
        return Ok(None);
    }
    let wrapping = wrapping.ok_or_else(invalid)?;
    let context = wrapping.ordering().ok_or_else(invalid)?;
    let after = line
        .source_start()
        .checked_add(line.len())
        .ok_or_else(invalid)?;
    let selection = context.selection(line.source_start()..after)?;
    let ranks = selection.ranks;
    let mut descriptors = buffer(line.len())?;
    let mut logical = buffer(line.len())?;
    let mut body = None;
    for (unit, &rank) in ranks.iter().enumerate() {
        let descriptor = input.supplied.get(unit).ok_or_else(invalid)?;
        let level = match *descriptor {
            surgeist_text::SuppliedInlineParticipant::Text { level, .. }
            | surgeist_text::SuppliedInlineParticipant::Atomic { level }
            | surgeist_text::SuppliedInlineParticipant::Cell { level }
            | surgeist_text::SuppliedInlineParticipant::Boundary { level } => level,
        };
        let actual =
            context
                .profile
                .validate_slot(rank, context.profile.structural(rank), level)?;
        if body.is_some_and(|body| body != actual) {
            return Err(invalid());
        }
        body = Some(actual);
        if !context.profile.structural(rank) {
            descriptors.push(*descriptor);
            logical.push(unit);
        }
    }
    let mut permutation = buffer(descriptors.len())?;
    permutation.resize(descriptors.len(), 0);
    SuppliedLine::new(&descriptors)
        .reorder_into(&mut permutation)
        .map_err(|_| invalid())?;
    let mut payload = buffer(permutation.len())?;
    for local in permutation {
        payload.push(logical[local]);
    }
    let order = schedule(
        &line,
        axes,
        wrapping,
        &selection,
        body.map_or_else(|| context.profile.selected_body(&selection), Ok)?,
        &payload,
    )?;
    let mut flow = S::ZERO;
    for &unit in &order {
        if !line.discarded(unit) {
            flow = flow + line.participant(unit).inline_advance(axes);
            if !flow.is_finite() {
                return Err(NumericWrappingError::Geometry {
                    value: flow.to_f64(),
                    reason: crate::LayoutTextNumericError::NonFinite,
                });
            }
        }
    }
    Ok(Some((
        flow,
        Arc::new(OrderReceipt {
            context: Arc::clone(context),
            source_start: input.source_start,
            descriptor_start: input.descriptor_start,
            order,
        }),
    )))
}

pub(super) fn reorder<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    wrapping: Option<&NumericWrapping>,
    order: &mut Vec<usize>,
) -> Result<(), NumericWrappingError> {
    if !Line::Selected(line).has_box_edges()
        && !wrapping
            .and_then(NumericWrapping::ordering)
            .is_some_and(|c| c.profile.has_family())
    {
        if line.native_order.is_some() {
            return Err(invalid());
        }
        return Ok(());
    }
    let receipt = line.native_order.as_ref().ok_or_else(invalid)?;
    let context = wrapping
        .and_then(NumericWrapping::ordering)
        .ok_or_else(invalid)?;
    if receipt.context != *context
        || receipt.source_start != line.source_start
        || receipt.descriptor_start != line.descriptor_range.start
        || receipt.order.len() != line.units.len()
    {
        return Err(WrappingScopeError::PlanMismatch.into());
    }
    order.clear();
    order
        .try_reserve(receipt.order.len())
        .map_err(|_| NumericWrappingError::Allocation)?;
    order.extend_from_slice(&receipt.order);
    Ok(())
}

fn schedule<S: LayoutScalar>(
    line: &Line<'_, S>,
    axes: FlowAxes,
    wrapping: &NumericWrapping,
    selection: &NativeOrderingSelection<'_>,
    body: usize,
    payload: &[usize],
) -> Result<Vec<usize>, NumericWrappingError> {
    let context = wrapping.ordering().ok_or_else(invalid)?;
    let ranks = selection.ranks;
    let mut edges = buffer(line.len())?;
    let positive_side = FlowAxes::new(axes.writing_mode(), crate::Direction::Ltr).inline_start();
    for unit in 0..line.len() {
        let MixedInlineParticipantOf::Boundary(control) = line.participant(unit) else {
            continue;
        };
        let alias = context.profile.alias(ranks[unit]);
        let parent = control.parent_edge_axes();
        if parent.is_none() && !alias {
            continue;
        }
        let source = line.source_start().checked_add(unit).ok_or_else(invalid)?;
        let endpoint = match control.kind() {
            crate::InlineBoundaryKind::Start => 0,
            crate::InlineBoundaryKind::End => 1,
        };
        let owner = if alias {
            usize::MAX
        } else {
            wrapping
                .boxes()
                .iter()
                .position(|row| row.endpoints[endpoint] == Some(source))
                .ok_or_else(invalid)?
        };
        let ancestry = if alias {
            context.profile.endpoint(ranks[unit]).ok_or_else(invalid)?.0
        } else {
            wrapping.boxes()[owner].ancestry
        };
        let mut extent = None;
        for (position, &candidate) in payload.iter().enumerate() {
            if context.profile.owns(ancestry, ranks[candidate])? {
                extent = Some(extent.map_or((position, position), |(first, _)| (first, position)));
            }
        }
        let start_left = if alias || parent.ok_or_else(invalid)?.inline_start() == positive_side {
            true
        } else if parent.ok_or_else(invalid)?.inline_end() == positive_side {
            false
        } else {
            return Err(invalid());
        };
        let left = start_left == (endpoint == 0);
        let gap = if alias {
            context
                .profile
                .alias_gap(ancestry, endpoint == 0, body, selection, payload)?
        } else if let Some((first, last)) = extent {
            if left { first } else { last + 1 }
        } else if let Some((first, last)) = context
            .profile
            .empty_extents(ancestry, body, selection, payload)?
        {
            if left { first } else { last }
        } else {
            context.profile.gap(ancestry, body, selection, payload)?
        };
        edges.push(Edge {
            unit,
            owner,
            ancestry,
            alias,
            left,
            gap,
            rank: unit,
            empty: extent.is_none(),
        });
    }
    let before = |a: Edge, b: Edge| -> Result<bool, NumericWrappingError> {
        if a.gap != b.gap {
            return Ok(false);
        }
        let aa = a.ancestry;
        let bb = b.ancestry;
        let related = wrapping.box_ancestor(aa, bb) || wrapping.box_ancestor(bb, aa);
        if related {
            Ok(if a.alias && !b.alias {
                !b.left
            } else if !a.alias && b.alias {
                a.left
            } else if a.alias && b.alias {
                a.rank < b.rank
            } else if a.left != b.left {
                a.left
            } else if a.owner == b.owner {
                a.rank < b.rank
            } else if a.left {
                wrapping.box_ancestor(aa, bb)
            } else {
                wrapping.box_ancestor(bb, aa)
            })
        } else if let Some(before) = context
            .profile
            .tie_before(aa, bb, body, selection, payload, a.gap)?
        {
            Ok(before)
        } else if !a.empty && !b.empty && a.left != b.left {
            // Adjacent real content extents close before the following one opens.
            Ok(!a.left)
        } else {
            Err(WrappingScopeError::AmbiguousGap.into())
        }
    };
    let mut emitted = buffer(edges.len())?;
    emitted.resize(edges.len(), false);
    let mut order = buffer(line.len())?;
    for gap in 0..=payload.len() {
        loop {
            let mut ready = None;
            let mut pending = false;
            for (index, &edge) in edges.iter().enumerate() {
                if emitted[index] || edge.gap != gap {
                    continue;
                }
                pending = true;
                let mut blocked = false;
                for (other, &candidate) in edges.iter().enumerate() {
                    if !emitted[other] && other != index && before(candidate, edge)? {
                        blocked = true;
                        break;
                    }
                }
                if !blocked && ready.is_none_or(|previous: usize| edges[previous].rank > edge.rank)
                {
                    ready = Some(index);
                }
            }
            let Some(index) = ready else {
                if pending {
                    return Err(WrappingScopeError::CyclicGap.into());
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
    if order.len() != line.len() {
        return Err(invalid());
    }
    Ok(order)
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
                executed_flow: None,
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
            executed_flow: None,
        },
        end,
    ))
}

/// Source aliases borrow only qualified actual executed Source ports. They are
/// still zero Contribution operations, never physical BoxEdge strips.
pub(super) fn resolve_aliases<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    wrapping: Option<&NumericWrapping>,
    slots: &mut [super::ResolvedInlineSlot<S>],
) -> Result<(), NumericWrappingError> {
    let Some(context) = wrapping
        .and_then(NumericWrapping::ordering)
        .filter(|p| p.profile.has_family())
    else {
        return Ok(());
    };
    let after = line
        .source_start
        .checked_add(line.units.len())
        .ok_or_else(invalid)?;
    let selection = context.selection(line.source_start..after)?;
    let mut payload = buffer(line.units.len())?;
    let order = &line.native_order.as_ref().ok_or_else(invalid)?.order;
    for &unit in order {
        if !context.profile.structural(selection.ranks[unit]) {
            payload.push(unit);
        }
    }
    for unit in 0..line.units.len() {
        let rank = selection.ranks[unit];
        if !context.profile.alias(rank) {
            continue;
        }
        let (owner, start) = context.profile.endpoint(rank).ok_or_else(invalid)?;
        let body = context.profile.body(rank)?;
        let ports = context
            .profile
            .alias_ports(owner, start, body, &selection, &payload)?;
        let mut coordinate = None;
        for port in ports.into_iter().flatten() {
            let point = match port {
                crate::inline_wrapping::ordering::AliasPort::Neighbor { rank, affinity } => {
                    let target = selection
                        .ranks
                        .iter()
                        .position(|&r| r == rank)
                        .ok_or_else(invalid)?;
                    let flow = slots[target].executed_flow.ok_or_else(invalid)?;
                    if affinity == crate::InlineGapAffinity::Before {
                        flow.0
                    } else {
                        flow.1
                    }
                }
                crate::inline_wrapping::ordering::AliasPort::Body { start } => {
                    let left = start == context.profile.source_start_left();
                    let target =
                        if left { order.first() } else { order.last() }.ok_or_else(invalid)?;
                    let flow = slots[*target].executed_flow.ok_or_else(invalid)?;
                    if left { flow.0 } else { flow.1 }
                }
            };
            if coordinate.is_some_and(|old| old != point) {
                return Err(WrappingScopeError::ConflictingGap.into());
            }
            coordinate = Some(point);
        }
        // No neighboring operand means an already-authenticated empty gap; its
        // zero operation's executed position is the actual cut-local witness.
        let point = coordinate.unwrap_or(slots[unit].inline_start);
        slots[unit].inline_start = point;
        slots[unit].executed_flow = Some((point, point));
    }
    Ok(())
}

pub(super) fn accepted<S: LayoutScalar>(
    line: &SelectedInlineLineOf<S>,
    wrapping: Option<&NumericWrapping>,
    slots: &[super::ResolvedInlineSlot<S>],
    axes: FlowAxes,
    line_index: usize,
) -> Result<Option<crate::inline_wrapping::family::NativeLineReceipt<S>>, NumericWrappingError> {
    use crate::inline_wrapping::family::{NativeExecutedEdge, NativeLineReceipt};
    let Some(context) = wrapping
        .and_then(NumericWrapping::ordering)
        .filter(|p| p.profile.has_family())
    else {
        return Ok(None);
    };
    let end = line
        .source_start
        .checked_add(line.units.len())
        .ok_or_else(invalid)?;
    let selection = context.selection(line.source_start..end)?;
    let mut payload = buffer(line.units.len())?;
    let order = &line.native_order.as_ref().ok_or_else(invalid)?.order;
    for &unit in order {
        if !context.profile.structural(selection.ranks[unit]) {
            payload.push(unit);
        }
    }
    let body = context.profile.selected_body(&selection)?;
    let rows = context
        .profile
        .selected_family_rows(&selection, &payload, body)?;
    let mut ranks = buffer(selection.ranks.len())?;
    ranks.extend_from_slice(selection.ranks);
    let mut edges = Vec::new();
    for (unit, &rank) in selection.ranks.iter().enumerate() {
        let Some((owner, start)) = context.profile.endpoint(rank) else {
            continue;
        };
        let Some(key) = context.profile.family_key(owner) else {
            continue;
        };
        let alias = context.profile.alias(rank);
        let MixedInlineParticipantOf::Boundary(control) = line.units[unit].participant else {
            return Err(invalid());
        };
        if !alias && control.parent_edge_axes().is_none() {
            continue;
        }
        let recipient = if alias {
            context.profile.alias_component(owner, start, body)?
        } else {
            let side = if start {
                control
                    .parent_edge_axes()
                    .ok_or_else(invalid)?
                    .inline_start()
            } else {
                control.parent_edge_axes().ok_or_else(invalid)?.inline_end()
            };
            let left =
                side == FlowAxes::new(axes.writing_mode(), crate::Direction::Ltr).inline_start();
            let mut found = None;
            for &candidate in &payload {
                if context.profile.owns(owner, selection.ranks[candidate])?
                    && (left && found.is_none() || !left)
                {
                    found = Some(selection.ranks[candidate]);
                }
            }
            if let Some(found) = found {
                context.profile.component_for(owner, Some(found), body)?
            } else {
                context
                    .profile
                    .empty_component(owner, body, &selection, &payload, left)?
            }
        };
        let slot = slots[unit];
        let flow = slot.executed_flow.ok_or_else(invalid)?;
        let extent = control.edge().border_padding_advance();
        edges
            .try_reserve(1)
            .map_err(|_| NumericWrappingError::Allocation)?;
        edges.push(NativeExecutedEdge {
            rank,
            owner: key,
            component: recipient,
            kind: control.kind(),
            alias,
            flow,
            strip: (slot.inline_start, slot.inline_start + extent),
        });
    }
    Ok(Some(NativeLineReceipt {
        profile: Arc::clone(&context.profile),
        receipt: Arc::clone(context),
        source: selection.raw_fences.clone(),
        body,
        line: line_index,
        axes,
        ranks,
        rows,
        edges,
    }))
}
