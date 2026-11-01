//! Canonical body borrowing and authenticated body-local box appearances.
use super::super::incidence::{SourceIncidence, visit_logical_event};
use super::*;
use crate::engine::analysis::wrapping::Event;
use crate::protocol::{
    paragraph::{ParagraphControlKind, SemanticAnchorItem},
    selection::SelectedData,
};

#[derive(Clone, Copy)]
pub(super) struct Presence {
    pub(super) body: usize,
    pub(super) owner: Owner,
    pub(super) opening: bool,
    closing: bool,
}

/// A closed family's real endpoint occurrence; frontier is filled only from
/// actual participating logical rows in that same canonical body.
pub(super) struct Endpoint {
    pub(super) control: usize,
    pub(super) owner: Owner,
    pub(super) body: usize,
    pub(super) start: bool,
    pub(super) frontier: Option<(usize, usize)>,
}

pub(super) struct PreparedBodies {
    pub(super) presence: Vec<Presence>,
    pub(super) endpoints: Vec<Endpoint>,
}

fn retain(
    data: &FamilyData,
    rows: &mut Vec<Presence>,
    body: usize,
    leaf: Owner,
    opening: bool,
    closing: bool,
    ledger: &mut Ledger,
) -> Result<()> {
    let mut at = Some(leaf);
    while let Some(owner) = at {
        account::work(ledger, 1)?;
        account::push_fallible(
            ledger,
            rows,
            Presence {
                body,
                owner,
                opening: opening && owner == leaf,
                closing: closing && owner == leaf,
            },
        )?;
        at = data.parent(owner, ledger)?;
    }
    Ok(())
}

pub(super) fn prepare(
    data: &mut FamilyData,
    owners: &[Owner],
    first: &[Option<usize>],
    ledger: &mut Ledger,
) -> Result<PreparedBodies> {
    let prepared = data.prepared.inner.clone();
    let a = &prepared.analyzed;
    let mut next = 0;
    // Body ordinals are exactly the retained canonical context ordinals. A real
    // context survives even when its entire raw range emits no bidi row.
    for section in 0..a.sections.len() {
        account::work(ledger, 1)?;
        let bodies = a
            .direction
            .body_indices(section)
            .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
        for body in bodies {
            account::work(ledger, 1)?;
            if body != data.bodies.len() {
                return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
            }
            account::push_fallible(ledger, &mut data.bodies, Body { visual: 0..0 })?;
            while next < data.rows.len() {
                account::work(ledger, 1)?;
                let row = &mut data.rows[next];
                if row.section != section {
                    break;
                }
                row.body = body;
                next += 1;
                if row.ends_body {
                    break;
                }
            }
        }
    }
    if next != data.rows.len() {
        return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
    }
    let mut presence = Vec::new();
    let mut endpoints = Vec::new();
    for section in 0..a.sections.len() {
        account::work(ledger, 1)?;
        for body in a
            .direction
            .body_indices(section)
            .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?
        {
            account::work(ledger, 1)?;
            let range = a
                .direction
                .body_range(body)
                .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
            for participant in range {
                account::work(ledger, 1)?;
                let event = a.logical.events[participant];
                visit_logical_event(&prepared, event.kind, ledger, &mut |site, ledger| {
                    let leaf = super::build::owner_at(data, site, ledger)?;
                    let endpoints = data.source(leaf).endpoints();
                    let actual_event = matches!(site, SourceIncidence::Event(_));
                    let opening = actual_event
                        && endpoints.as_ref().is_some_and(|(start, _)| {
                            start.event_position().index == site.position()
                        });
                    let closing = actual_event
                        && endpoints
                            .as_ref()
                            .is_some_and(|(_, end)| end.event_position().index == site.position());
                    retain(data, &mut presence, body, leaf, opening, closing, ledger)
                })?;
                // Native content containment is deliberately half-open. The
                // issuer's exact nominated endpoint nevertheless owns its box.
                if let Event::Anchor(anchor) = event.kind
                    && let SemanticAnchorItem::Control(control) = anchor.item
                {
                    let kind = &a.paragraph.controls[control].kind;
                    let start = *kind == ParagraphControlKind::InlineBoundaryStart;
                    if start || *kind == ParagraphControlKind::InlineBoundaryEnd {
                        let position = a
                            .paragraph
                            .scope_topology
                            .as_ref()
                            .expect("admitted family")
                            .control_orders[control];
                        let leaf =
                            super::build::owner_at(data, SourceIncidence::Event(position), ledger)?;
                        let source = data.source(leaf);
                        if source.is_inline()
                            && source.endpoints().is_some_and(|(entry, exit)| {
                                let endpoint = if start { entry } else { exit };
                                endpoint.event_position().index == position
                            })
                        {
                            account::push_fallible(
                                ledger,
                                &mut endpoints,
                                Endpoint {
                                    control,
                                    owner: leaf,
                                    body,
                                    start,
                                    frontier: None,
                                },
                            )?;
                        }
                        if let Some(native) = data.native_endpoint(position, start, ledger)? {
                            account::push_fallible(
                                ledger,
                                &mut endpoints,
                                Endpoint {
                                    control,
                                    owner: Owner::Native(native),
                                    body,
                                    start,
                                    frontier: None,
                                },
                            )?;
                            retain(
                                data,
                                &mut presence,
                                body,
                                Owner::Native(native),
                                start,
                                !start,
                                ledger,
                            )?;
                        }
                    }
                }
            }
        }
    }
    // Only actual closing endpoint incidences enter this temporary lookup.
    // It avoids rescanning the whole body inventory for each continuation.
    let mut closings = Vec::new();
    for occurrence in &presence {
        account::work(ledger, 1)?;
        if occurrence.closing {
            account::push_fallible(ledger, &mut closings, (occurrence.owner, occurrence.body))?;
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut closings,
        ledger.as_mut(),
        |a, b| a.cmp(b),
    )?;
    // Opaque Cells can have no raw participants. Their actual retained Source
    // incidence is already in the emitted row, and cannot be reconstructed by
    // expanding the Cell's raw cardinality.
    for row in &data.rows {
        account::work(ledger, 1)?;
        for &leaf in &data.row_owners[row.owners.clone()] {
            account::work(ledger, 1)?;
            retain(data, &mut presence, row.body, leaf, false, false, ledger)?;
            if row.ends_body {
                let bodies = a
                    .direction
                    .body_indices(row.section)
                    .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
                let next = row.body + 1;
                if bodies.contains(&next) {
                    account::work(ledger, 1)?;
                    let current = a
                        .direction
                        .body_range(row.body)
                        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
                    let following = a
                        .direction
                        .body_range(next)
                        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
                    if current.end != following.start {
                        return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
                    }
                    // This is continuation at the authentic post-delimiter cut:
                    // actual delimiter Source custody plus its real ancestry,
                    // not a global box-interval/body overlap.
                    let mut at = Some(leaf);
                    while let Some(owner) = at {
                        account::work(ledger, 1)?;
                        let first =
                            crate::processing::contextual::alignment::operations::partition_point(
                                &closings,
                                ledger.as_mut(),
                                |entry| entry.0 < owner,
                            )?;
                        let closed = closings.get(first).is_some_and(|&(closed_owner, body)| {
                            closed_owner == owner && body <= row.body
                        });
                        if !closed {
                            account::push_fallible(
                                ledger,
                                &mut presence,
                                Presence {
                                    body: next,
                                    owner,
                                    opening: false,
                                    closing: false,
                                },
                            )?;
                        }
                        at = data.parent(owner, ledger)?;
                    }
                }
            }
        }
    }
    account::drop_vec(ledger, closings)?;
    // Real OPEN/entry custody is independent of text, markers or pixel size.
    // Preserve the established opening-gap ordering for globally empty boxes.
    for &owner in owners {
        account::work(ledger, 1)?;
        if !data.source(owner).is_inline() {
            continue;
        }
        let mut known = false;
        for row in &presence {
            account::work(ledger, 1)?;
            if row.owner == owner && row.opening {
                known = true;
                break;
            }
        }
        if known {
            continue;
        }
        let (begin, _) = data.interval(owner);
        let Some(section) = crate::engine::analysis::structural::section_at_order(
            begin,
            &a.sections,
            ledger.as_mut(),
        )?
        else {
            // An actual ancestor can enter before this complete owned section.
            // Its body appearances must come from real local Source custody.
            continue;
        };
        let mut body = a
            .direction
            .body_indices(section)
            .and_then(|range| range.end.checked_sub(1))
            .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
        for (index, row) in data.rows.iter().enumerate() {
            account::work(ledger, 1)?;
            if row.section == section && first[index].is_some_and(|at| begin <= at) {
                body = row.body;
                break;
            }
        }
        retain(data, &mut presence, body, owner, true, false, ledger)?;
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut presence,
        ledger.as_mut(),
        |a, b| (a.body, a.owner).cmp(&(b.body, b.owner)),
    )?;
    let mut retained = 0;
    for index in 0..presence.len() {
        account::work(ledger, 1)?;
        let row = presence[index];
        if retained != 0
            && presence[retained - 1].body == row.body
            && presence[retained - 1].owner == row.owner
        {
            presence[retained - 1].opening |= row.opening;
            presence[retained - 1].closing |= row.closing;
        } else {
            presence[retained] = row;
            retained += 1;
        }
    }
    presence.truncate(retained);
    // Reversed genuine entry order lets continuation insertions at the same
    // body-start cut preserve Source box order, independently of numeric IDs.
    crate::processing::contextual::alignment::operations::order_by(
        &mut presence,
        ledger.as_mut(),
        |a, b| {
            a.body
                .cmp(&b.body)
                .then_with(|| data.interval(b.owner).0.cmp(&data.interval(a.owner).0))
        },
    )?;
    crate::processing::contextual::alignment::operations::order_by(
        &mut endpoints,
        ledger.as_mut(),
        |a, b| (a.body, a.owner, a.start).cmp(&(b.body, b.owner, b.start)),
    )?;
    Ok(PreparedBodies {
        presence,
        endpoints,
    })
}

pub(super) fn selected_body(
    selected: &SelectedData,
    unit: usize,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    let Some(local) = selected.presentation.units[unit].participant else {
        return Ok(None);
    };
    let tables = &selected.prepared.tables;
    let start = tables.boundaries[selected.request.start.index].participant;
    let outer = start
        .checked_add(local)
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    body_for_outer(&selected.prepared, &selected.request, outer, ledger)
}

pub(super) fn control_body(
    prepared: &PreparedData,
    request: &crate::LineSelectionRequest,
    control: usize,
    ledger: &mut Ledger,
) -> Result<usize> {
    let outer = prepared.tables.controls[control]
        .participant
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    body_for_outer(prepared, request, outer, ledger)?
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))
}

fn body_for_outer(
    prepared: &PreparedData,
    request: &crate::LineSelectionRequest,
    outer: usize,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    let tables = &prepared.tables;
    let after = outer
        .checked_add(1)
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    let raw = tables
        .source_prefix(outer)
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    let end = tables
        .source_prefix(after)
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    prepared
        .analyzed
        .direction
        .body_for_range(request.segment.index, raw..end, ledger.as_mut())
}
