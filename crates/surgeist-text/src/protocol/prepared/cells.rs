//! Genuine source cells lowered into the outer prepared occurrence stream.

use super::{BoundaryData, ParticipantData, PreparedTables};
use crate::engine::layout::data::{ShapedData, ShapedItem};
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::engine::scope_alignment::census::{AllocationVisitor, AllocationWitness};
use crate::{Range, Result};

#[derive(Debug)]
pub(crate) struct CellData {
    /// The real retained cell, not a font run or source-group ordinal.
    pub(crate) cell: usize,
    pub(crate) binding: usize,
    pub(crate) participant: usize,
    /// Enclosing aggregate correspondence; pieces remain with the source plan.
    pub(crate) source: Range,
    pub(crate) processed: Range,
}

fn invalid() -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::LayoutFailed,
        "prepared cell source incidence disagrees with its completed source or shaped inventory",
    )
}

/// Convert an authenticated source cut to its outer occurrence prefix. A cut
/// strictly inside a real cell is unavailable, even if its raw prefix is shared
/// with one of the cell's edges.
pub(crate) fn outer_cut(
    tables: &PreparedTables,
    analyzed: &crate::engine::AnalyzedData,
    source_participant: usize,
    relation_cut: usize,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    if tables.cells.is_empty() {
        return Ok((source_participant <= tables.participants.len()).then_some(source_participant));
    }
    let output = analyzed.combination.as_deref().ok_or_else(invalid)?;
    for (outer, (participant, source)) in tables
        .participants
        .iter()
        .zip(&tables.source_participants)
        .enumerate()
    {
        account::work(ledger, 1)?;
        if let ParticipantData::Cell(cell) = *participant {
            let binding = &output.bindings[tables.cells[cell].binding];
            if relation_cut <= binding.events.start && source_participant <= source.start {
                return Ok(Some(outer));
            }
            if relation_cut >= binding.events.end && source_participant >= source.end {
                continue;
            }
            return Ok(None);
        }
        if source_participant <= source.start {
            return Ok(Some(outer));
        }
        if source_participant < source.end {
            return Ok(None);
        }
    }
    Ok((source_participant == analyzed.logical.events.len()).then_some(tables.participants.len()))
}

pub(crate) fn lower(
    tables: &mut PreparedTables,
    analyzed: &crate::engine::AnalyzedData,
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<()> {
    if shaped.cells.is_empty() {
        return Ok(());
    }
    let output = analyzed.combination.as_deref().ok_or_else(invalid)?;
    account::header::<Vec<CellData>>(ledger)?;
    account::header::<Vec<std::ops::Range<usize>>>(ledger)?;
    account::header::<Vec<ParticipantData>>(ledger)?;
    account::header::<Vec<usize>>(ledger)?;
    account::header::<Vec<BoundaryData>>(ledger)?;
    let mut participants = Vec::new();
    let mut source_participants = Vec::new();
    let mut raw = 0;
    let mut cell = 0;
    while raw < tables.participants.len() || cell < shaped.cells.len() {
        account::work(ledger, 1)?;
        let next = shaped
            .cells
            .get(cell)
            .map(|row| (row, &output.bindings[row.binding]));
        if let Some((row, binding)) = next
            && binding.participants.start == raw
        {
            if binding.participants.end > tables.participants.len()
                || binding.events.start >= binding.events.end
            {
                return Err(invalid());
            }
            let group = &output.plan.groups[binding.group];
            let mut parts = output.plan.parts[group.parts.clone()].iter();
            let first = parts.next().ok_or_else(invalid)?;
            let mut source = first.source;
            let mut processed = first.processed;
            for part in parts {
                account::work(ledger, 1)?;
                source.start = source.start.min(part.source.start);
                source.end = source.end.max(part.source.end);
                processed.start = processed.start.min(part.processed.start);
                processed.end = processed.end.max(part.processed.end);
            }
            let index = tables.cells.len();
            account::push(
                ledger,
                &mut tables.cells,
                CellData {
                    cell,
                    binding: row.binding,
                    participant: participants.len(),
                    source,
                    processed,
                },
            )?;
            account::push(ledger, &mut participants, ParticipantData::Cell(index))?;
            account::push(
                ledger,
                &mut source_participants,
                binding.participants.clone(),
            )?;
            raw = binding.participants.end;
            cell += 1;
        } else {
            if next.is_some_and(|(_, binding)| binding.participants.start < raw)
                || raw >= tables.participants.len()
            {
                return Err(invalid());
            }
            account::push(ledger, &mut participants, tables.participants[raw])?;
            account::push(ledger, &mut source_participants, raw..raw + 1)?;
            raw += 1;
        }
    }
    // The original raw tables were native preparation work. These replacement
    // buffers are actual cell-owned output and remain in the feature census.
    tables.participants = participants;
    tables.source_participants = source_participants;
    // Inner source members remain in the immutable completed Source operation;
    // they are not separately issued as outer Text capabilities.
    let mut unit = 0;
    for participant in &mut tables.participants {
        account::work(ledger, 1)?;
        if let ParticipantData::Text(original) = participant {
            if unit != *original {
                account::work(ledger, 2)?;
                tables.units.swap(unit, *original);
            }
            *original = unit;
            unit += 1;
        }
    }
    tables.units.truncate(unit);
    tables.auto_phrase_prefix = account::filled(ledger, 1, 0)?;
    for row in &mut tables.atomics {
        row.participant = outer_cut_parts(
            &tables.participants,
            &tables.source_participants,
            row.participant,
            ledger,
        )?
        .ok_or_else(invalid)?;
    }
    for row in &mut tables.controls {
        if let Some(raw) = row.participant {
            row.participant = outer_cut_parts(
                &tables.participants,
                &tables.source_participants,
                raw,
                ledger,
            )?;
        }
    }
    let native_boundaries = std::mem::take(&mut tables.boundaries);
    for boundary in &native_boundaries {
        account::work(ledger, 1)?;
        let mut boundary = *boundary;
        if let Some(outer) = outer_cut(
            tables,
            analyzed,
            boundary.participant,
            boundary.relation_cut,
            ledger,
        )? {
            boundary.participant = outer;
            account::push(ledger, &mut tables.boundaries, boundary)?;
        }
    }
    for index in 0..tables.cells.len() {
        account::work(ledger, 1)?;
        let cell = &tables.cells[index];
        let binding = &output.bindings[cell.binding];
        let mut item = None;
        for (ordinal, candidate) in shaped.items.iter().enumerate() {
            account::work(ledger, 1)?;
            if matches!(*candidate, ShapedItem::Cell(i) if i == cell.cell) {
                item = Some(ordinal);
                break;
            }
        }
        let item = item.ok_or_else(invalid)?;
        for (outer, raw, atom, relation_cut, preliminary) in [
            (
                cell.participant,
                binding.participants.start,
                binding.atoms.start,
                binding.events.start,
                item,
            ),
            (
                cell.participant + 1,
                binding.participants.end,
                binding.atoms.end,
                binding.events.end,
                item + 1,
            ),
        ] {
            let mut found = None;
            for (index, row) in tables.boundaries.iter().enumerate() {
                account::work(ledger, 1)?;
                if row.participant == outer && row.atom == atom && row.relation_cut == relation_cut
                {
                    found = Some(index);
                    break;
                }
            }
            if let Some(index) = found {
                let row = &mut tables.boundaries[index];
                row.preliminary_item = Some(preliminary);
                continue;
            }
            let mut row = None;
            for native in &native_boundaries {
                account::work(ledger, 1)?;
                if native.participant == raw && native.atom == atom {
                    row = Some(*native);
                    break;
                }
            }
            let mut row = row.ok_or_else(invalid)?;
            // Cell edges are genuine source-unit edges, including a unit with
            // no logical/provider scalar. Preserve the native incident extrema
            // and right-atom processed coordinate; no source bounds are invented.
            row.participant = outer;
            row.source_participant = raw;
            row.relation_cut = relation_cut;
            row.preliminary_item = Some(preliminary);
            let key = (outer, relation_cut, atom);
            let mut at = 0;
            while let Some(existing) = tables.boundaries.get(at) {
                account::work(ledger, 1)?;
                if (existing.participant, existing.relation_cut, existing.atom) > key {
                    break;
                }
                at += 1;
            }
            account::push(ledger, &mut tables.boundaries, row)?;
            // The search proves at <= len. Admit every row in the rotated
            // suffix before this variable-sized movement, not only its push.
            account::work(ledger, tables.boundaries.len() - at)?;
            tables.boundaries[at..].rotate_right(1);
        }
    }
    Ok(())
}

fn outer_cut_parts(
    participants: &[ParticipantData],
    source: &[std::ops::Range<usize>],
    raw: usize,
    ledger: &mut Ledger,
) -> Result<Option<usize>> {
    for (outer, range) in source.iter().enumerate() {
        account::work(ledger, 1)?;
        if range.contains(&raw) {
            return Ok((!matches!(participants[outer], ParticipantData::Cell(_))).then_some(outer));
        }
    }
    Ok(None)
}

pub(crate) fn visit_allocations(
    tables: &PreparedTables,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    if tables.cells.is_empty() {
        return Ok(());
    }
    visit(AllocationWitness::object(&tables.cells, 0))?;
    visit(AllocationWitness::buffer(
        &tables.cells,
        tables.cells.len(),
    )?)?;
    visit(AllocationWitness::object(&tables.source_participants, 0))?;
    visit(AllocationWitness::buffer(
        &tables.source_participants,
        tables.source_participants.len(),
    )?)?;
    visit(AllocationWitness::object(&tables.participants, 0))?;
    visit(AllocationWitness::buffer(
        &tables.participants,
        tables.participants.len(),
    )?)?;
    visit(AllocationWitness::object(&tables.auto_phrase_prefix, 0))?;
    visit(AllocationWitness::buffer(
        &tables.auto_phrase_prefix,
        tables.auto_phrase_prefix.len(),
    )?)?;
    visit(AllocationWitness::object(&tables.boundaries, 0))?;
    visit(AllocationWitness::buffer(
        &tables.boundaries,
        tables.boundaries.len(),
    )?)?;
    Ok(())
}
