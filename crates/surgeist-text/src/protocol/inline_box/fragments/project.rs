//! One selected Source/unit/packet projection shared by execution and readback.
use super::*;
use crate::protocol::inline_box::incidence::visit_unit_events;
use crate::protocol::{
    prepared::{PresentationUnitKind as K, SelectedRelationEvent},
    selection::SelectedData,
};
use crate::transform::relation::RelationEvent;

pub(crate) struct Piece {
    pub(crate) fragment: usize,
    pub(crate) members: Range<usize>,
    pub(crate) packets: Range<usize>,
    pub(crate) start: Option<usize>,
    pub(crate) end: Option<usize>,
}
pub(crate) struct Member {
    pub(crate) unit: usize,
}
/// Ancillary Source aliases are not independently invented body/TCU rows.
pub(crate) struct Alias {
    pub(crate) unit: usize,
    pub(crate) owner: Owner,
}
pub(crate) struct SelectedBoxPieces {
    pub(crate) pieces: Vec<Piece>,
    pub(crate) members: Vec<Member>,
    pub(crate) aliases: Vec<Alias>,
    pub(crate) execution: Option<super::super::execution::ExecutionPlan>,
}
impl SelectedBoxPieces {
    pub(crate) fn visit_allocations(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        visit(AllocationWitness::object(self, 0))?;
        visit(AllocationWitness::buffer(&self.pieces, self.pieces.len())?)?;
        visit(AllocationWitness::buffer(
            &self.members,
            self.members.len(),
        )?)?;
        visit(AllocationWitness::buffer(
            &self.aliases,
            self.aliases.len(),
        )?)?;
        if let Some(plan) = &self.execution {
            visit(AllocationWitness::buffer(&plan.order, plan.order.len())?)?;
            visit(AllocationWitness::buffer(
                &plan.boxed_units,
                plan.boxed_units.len(),
            )?)?;
        }
        Ok(())
    }
}
fn row(data: &FamilyData, key: BidiOwner, ledger: &mut Ledger) -> Result<Option<usize>> {
    let (mut a, mut b) = (0, data.row_lookup.len());
    while a < b {
        account::work(ledger, 1)?;
        let m = a + (b - a) / 2;
        if data.row_lookup[m].0 < key {
            a = m + 1
        } else {
            b = m
        }
    }
    Ok(data.row_lookup.get(a).filter(|v| v.0 == key).map(|v| v.1))
}
fn empty(
    data: &FamilyData,
    body: usize,
    owner: Owner,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    let key = (body, owner);
    let (mut a, mut b) = (0, data.empty_lookup.len());
    while a < b {
        account::work(ledger, 1)?;
        let m = a + (b - a) / 2;
        if data.empty_lookup[m].0 < key {
            a = m + 1
        } else {
            b = m
        }
    }
    Ok(data.empty_lookup.get(a).filter(|v| v.0 == key).map(|v| v.1))
}
fn associations(
    data: &FamilyData,
    row: usize,
    unit: usize,
    out: &mut Vec<(usize, usize)>,
    ledger: &mut Ledger,
) -> Result<()> {
    let (mut a, mut b) = (0, data.row_fragments.len());
    while a < b {
        account::work(ledger, 1)?;
        let m = a + (b - a) / 2;
        if data.row_fragments[m].0 < row {
            a = m + 1
        } else {
            b = m
        }
    }
    while let Some(&(r, fragment)) = data.row_fragments.get(a) {
        account::work(ledger, 1)?;
        if r != row {
            break;
        }
        account::push_fallible(ledger, out, (fragment, unit))?;
        a += 1;
    }
    Ok(())
}
fn endpoint_fragment(
    data: &FamilyData,
    control: usize,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    let index = crate::processing::contextual::alignment::operations::binary_search_by(
        &data.endpoint_fragments,
        ledger.as_mut(),
        |entry| entry.0.cmp(&control),
    )?;
    Ok(index.ok().map(|index| data.endpoint_fragments[index].1))
}

/// Exact aliases for this issued unit are contiguous. Different Source/Native
/// owners remain distinct; repeated proof of the same unit/owner adds no custody.
fn retain_alias(
    aliases: &mut Vec<Alias>,
    first: usize,
    unit: usize,
    owner: Owner,
    ledger: &mut Ledger,
) -> Result<()> {
    for alias in &aliases[first..] {
        account::work(ledger, 1)?;
        if alias.unit == unit && alias.owner == owner {
            return Ok(());
        }
    }
    account::push_fallible(ledger, aliases, Alias { unit, owner })
}

pub(crate) fn project(selected: &mut SelectedData, ledger: &mut Ledger) -> Result<()> {
    let Some(family) = selected.request.inline_boxes.clone() else {
        return Ok(());
    };
    family.require_prepared(&selected.prepared)?;
    let data = &family.inner;
    let mut found = Vec::new();
    let mut aliases = Vec::new();
    let mut body_units = Vec::new();
    let mut indications = selected.presentation.domains.iter().peekable();
    for (unit, value) in selected.presentation.units.iter().enumerate() {
        account::work(ledger, 1)?;
        let source_body = super::bodies::selected_body(selected, unit, ledger)?;
        let first_alias = aliases.len();
        let mut represented = false;
        let mut body = false;
        let key = match value.kind {
            K::Cell { cell } => Some(BidiOwner::CombinedCell(
                selected.prepared.shaped.cells[cell].binding,
            )),
            K::Atomic { occurrence } => Some(BidiOwner::Atomic(occurrence)),
            K::Control { occurrence } => Some(BidiOwner::Control(occurrence)),
            K::ComputedLineBoundary { boundary } => Some(BidiOwner::ComputedLineBoundary(boundary)),
            _ => None,
        };
        if let Some(key) = key
            && let Some(row) = row(data, key, ledger)?
        {
            associations(data, row, unit, &mut found, ledger)?;
            represented = true;
            body |= data.rows[row].content;
        }
        if !matches!(value.kind, K::Cell { .. }) {
            for event in &selected.presentation.selected_relation.events
                [selected.presentation.selected_relation.unit_events[unit].clone()]
            {
                account::work(ledger, 1)?;
                if let SelectedRelationEvent::Base { event } = *event
                    && let RelationEvent::Text { atom } =
                        selected.prepared.analyzed.transform.events[event]
                    && let Some(row) = row(data, BidiOwner::Atom(atom), ledger)?
                {
                    associations(data, row, unit, &mut found, ledger)?;
                    represented = true;
                    body |= data.rows[row].content;
                }
            }
        }
        // The local indication domain already retains the exact authenticated
        // Before/After occurrence. It supplies attachment, never original text.
        while indications
            .peek()
            .is_some_and(|domain| domain.units.end <= unit)
        {
            account::work(ledger, 1)?;
            indications.next();
        }
        if let Some(domain) = indications
            .peek()
            .filter(|domain| domain.units.contains(&unit))
        {
            account::work(ledger, 1)?;
            let owner = domain
                .attachment
                .original_owner(&selected.prepared.analyzed, &selected.prepared.tables)?;
            let attached = row(data, owner, ledger)?
                .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
            associations(data, attached, unit, &mut found, ledger)?;
            represented = true;
            body = true;
        }
        if !represented {
            // Actual nonempty selected operations remain body content even when
            // their indication is issued at a source cut, not an original row.
            body = !value.atoms.is_empty()
                || matches!(
                    value.kind,
                    K::Cell { .. } | K::Atomic { .. } | K::ComputedLineBoundary { .. }
                );
        }
        if !represented
            || matches!(
                value.kind,
                K::Deleted | K::EmptyOrigin { .. } | K::Control { .. }
            )
            || value.code_point.is_some_and(|cp| {
                icu_properties::CodePointSetData::new::<icu_properties::props::BidiControl>()
                    .contains32(cp)
            })
        {
            // Preserve exact ancillary Source custody separately, without claiming
            // a marker has an exclusive global fragment merely by enclosure.
            visit_unit_events(
                &selected.prepared,
                &selected.presentation,
                unit,
                ledger,
                |site, ledger| {
                    let owner = super::build::owner_at(data, site, ledger)?;
                    retain_alias(&mut aliases, first_alias, unit, owner, ledger)?;
                    let mut at = Some(owner);
                    while let Some(owner) = at {
                        account::work(ledger, 1)?;
                        if let Some(body_index) = source_body
                            && let Some(row) = empty(data, body_index, owner, ledger)?
                        {
                            associations(data, row, unit, &mut found, ledger)?;
                            body = true;
                            break;
                        }
                        at = data.parent(owner, ledger)?;
                    }
                    Ok(())
                },
            )?;
        }
        if let K::Control { occurrence } = value.kind
            && let Some(fragment) = endpoint_fragment(data, occurrence, ledger)?
        {
            let frontier = &data.fragments[fragment];
            if source_body != Some(frontier.body) {
                return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
            }
            retain_alias(&mut aliases, first_alias, unit, frontier.owner, ledger)?;
            let mut at = Some(fragment);
            while let Some(fragment) = at {
                account::work(ledger, 1)?;
                account::push_fallible(ledger, &mut found, (fragment, unit))?;
                at = data.fragments[fragment].parent;
            }
            // This selects only the actual endpoint unit. Its frontier's text,
            // provider packet and content classification are never imported.
            // BoxEdge execution authority remains the separate allocation below.
        }
        account::push_fallible(ledger, &mut body_units, body)?;
    }
    // A real nominated endpoint can carry a genuine empty box's incidence;
    // the Source control itself remains the original operation and level.
    for allocation in &selected.request.controls {
        let Some(context) = allocation
            .boundary
            .as_ref()
            .and_then(crate::InlineBoundaryAllocation::box_context)
        else {
            continue;
        };
        let owner = data.owner(&context.owner, ledger)?;
        let source_body = super::bodies::control_body(
            &selected.prepared,
            &selected.request,
            allocation.occurrence.index,
            ledger,
        )?;
        let Some(row) = empty(data, source_body, owner, ledger)? else {
            continue;
        };
        let occurrence = selected.prepared.tables.controls[allocation.occurrence.index].occurrence;
        for (unit, value) in selected.presentation.units.iter().enumerate() {
            account::work(ledger, 1)?;
            if value.kind == (K::Control { occurrence }) {
                associations(data, row, unit, &mut found, ledger)?;
                body_units[unit] = true;
                break;
            }
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut found,
        ledger.as_mut(),
        |a, b| a.cmp(b),
    )?;
    let mut pieces = Vec::new();
    let mut members = Vec::new();
    let mut cursor = 0;
    while cursor < found.len() {
        let fragment = found[cursor].0;
        let start = members.len();
        let mut previous = None;
        while cursor < found.len() && found[cursor].0 == fragment {
            account::work(ledger, 1)?;
            let unit = found[cursor].1;
            if previous != Some(unit) {
                account::push_fallible(ledger, &mut members, Member { unit })?;
                previous = Some(unit);
            }
            cursor += 1;
        }
        let end = members.len();
        account::push_fallible(
            ledger,
            &mut pieces,
            Piece {
                fragment,
                members: start..end,
                packets: 0..0,
                start: None,
                end: None,
            },
        )?;
    }
    account::drop_vec(ledger, found)?;
    account::header::<SelectedBoxPieces>(ledger)?;
    selected.basis.inline_boxes = Some(SelectedBoxPieces {
        pieces,
        members,
        aliases,
        execution: None,
    });
    let mut execution = super::super::execution::build(
        selected,
        &selected.basis.shaped,
        &selected.basis.order,
        ledger,
    )?;
    if let Some(plan) = &mut execution {
        let bounds = plan
            .projection
            .take()
            .expect("family projection returns its actual packet incidence");
        let projected = selected
            .basis
            .inline_boxes
            .as_mut()
            .expect("constructed projection");
        seal_pieces(projected, &bounds, &body_units, ledger)?;
        account::drop_vec(ledger, bounds)?;
        // Only actual allocated edges associate; context-only/future edges do not.
        for allocation in &selected.request.controls {
            let Some(context) = allocation
                .boundary
                .as_ref()
                .and_then(crate::InlineBoundaryAllocation::box_context)
            else {
                continue;
            };
            let owner = data.owner(&context.owner, ledger)?;
            let source_body = super::bodies::control_body(
                &selected.prepared,
                &selected.request,
                allocation.occurrence.index,
                ledger,
            )?;
            let actual = allocation
                .occurrence
                .captured_control()
                .expect("admitted BoxEdge issuer");
            let (start, end) = context.owner.endpoints().expect("admitted box endpoints");
            let source_start = actual == start;
            if !source_start && actual != end {
                return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
            }
            let physical_start = if source_start {
                context.start_side == crate::LineEdgeSide::Start
            } else {
                context.start_side == crate::LineEdgeSide::End
            };
            let mut target = None;
            for (index, piece) in projected.pieces.iter().enumerate() {
                account::work(ledger, 1)?;
                if data.fragments[piece.fragment].owner != owner
                    || data.fragments[piece.fragment].body != source_body
                {
                    continue;
                }
                if target.is_none_or(|other: usize| {
                    if physical_start {
                        piece.packets.start < projected.pieces[other].packets.start
                    } else {
                        projected.pieces[other].packets.end < piece.packets.end
                    }
                }) {
                    target = Some(index);
                }
            }
            if let Some(index) = target {
                if source_start {
                    projected.pieces[index].start = Some(allocation.occurrence.index)
                } else {
                    projected.pieces[index].end = Some(allocation.occurrence.index)
                }
            }
        }
        projected.execution = execution;
    }
    account::drop_vec(ledger, body_units)?;
    Ok(())
}

fn seal_pieces(
    projected: &mut SelectedBoxPieces,
    bounds: &[Option<(usize, usize)>],
    body_units: &[bool],
    ledger: &mut Ledger,
) -> Result<()> {
    let material = material_packets(bounds, body_units, ledger)?;
    let old_pieces = std::mem::take(&mut projected.pieces);
    let old_members = std::mem::take(&mut projected.members);
    for piece in &old_pieces {
        let mut ordered = Vec::new();
        for member in &old_members[piece.members.clone()] {
            account::work(ledger, 1)?;
            if let Some((a, b)) = bounds[member.unit] {
                account::push_fallible(ledger, &mut ordered, (a, b, member.unit))?;
            }
        }
        crate::processing::contextual::alignment::operations::order_by(
            &mut ordered,
            ledger.as_mut(),
            |a, b| a.cmp(b),
        )?;
        if ordered.is_empty() {
            let start = projected.members.len();
            for member in &old_members[piece.members.clone()] {
                account::push_fallible(
                    ledger,
                    &mut projected.members,
                    Member { unit: member.unit },
                )?;
            }
            let end = projected.members.len();
            account::push_fallible(
                ledger,
                &mut projected.pieces,
                Piece {
                    fragment: piece.fragment,
                    members: start..end,
                    packets: 0..0,
                    start: None,
                    end: None,
                },
            )?;
        } else {
            let mut cursor = 0;
            while cursor < ordered.len() {
                let begin = ordered[cursor].0;
                let mut after = ordered[cursor].1;
                let start = projected.members.len();
                loop {
                    account::work(ledger, 1)?;
                    let (_, b, unit) = ordered[cursor];
                    after = after.max(b);
                    account::push_fallible(ledger, &mut projected.members, Member { unit })?;
                    cursor += 1;
                    if cursor == ordered.len()
                        || (after < ordered[cursor].0
                            && material_between(&material, after, ordered[cursor].0, ledger)?)
                    {
                        break;
                    }
                }
                let end = projected.members.len();
                account::push_fallible(
                    ledger,
                    &mut projected.pieces,
                    Piece {
                        fragment: piece.fragment,
                        members: start..end,
                        packets: begin..after,
                        start: None,
                        end: None,
                    },
                )?;
            }
        }
        account::drop_vec(ledger, ordered)?;
    }
    account::drop_vec(ledger, old_members)?;
    account::drop_vec(ledger, old_pieces)?;
    account::drop_vec(ledger, material)
}

// Only actual body/empty-box rows or nonempty selected operations can make a
// foreign packet gap. Ancillary Source anchors remain in the payload order.
fn material_packets(
    bounds: &[Option<(usize, usize)>],
    body: &[bool],
    ledger: &mut Ledger,
) -> Result<Vec<(usize, usize)>> {
    let mut intervals = Vec::new();
    for (unit, value) in bounds.iter().enumerate() {
        account::work(ledger, 1)?;
        if body[unit]
            && let Some(range) = *value
        {
            account::push_fallible(ledger, &mut intervals, range)?;
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut intervals,
        ledger.as_mut(),
        |a, b| a.cmp(b),
    )?;
    let mut retained = 0;
    for index in 0..intervals.len() {
        account::work(ledger, 1)?;
        let next = intervals[index];
        if retained > 0 && next.0 <= intervals[retained - 1].1 {
            intervals[retained - 1].1 = intervals[retained - 1].1.max(next.1);
        } else {
            intervals[retained] = next;
            retained += 1;
        }
    }
    intervals.truncate(retained);
    Ok(intervals)
}
fn material_between(
    intervals: &[(usize, usize)],
    before: usize,
    after: usize,
    ledger: &mut Ledger,
) -> Result<bool> {
    let (mut a, mut b) = (0, intervals.len());
    while a < b {
        account::work(ledger, 1)?;
        let m = a + (b - a) / 2;
        if intervals[m].1 <= before {
            a = m + 1
        } else {
            b = m
        }
    }
    account::work(ledger, 1)?;
    Ok(intervals.get(a).is_some_and(|&(start, _)| start < after))
}
