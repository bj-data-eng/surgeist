//! Actual storage ownership of character plans, graphs and mutable gap arenas.
use super::*;
use crate::engine::scope_alignment::census::{AllocationVisitor, AllocationWitness as W};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum GapArenaOwnership {
    #[default]
    Native,
    Funded,
}

pub(super) fn require_ledger(ledger: &Ledger) -> Result<()> {
    if ledger.is_none() {
        return Err(crate::engine::scope_alignment::refusal(
            crate::ScopeAlignmentError::LimitsRequired,
        ));
    }
    Ok(())
}

impl Graph {
    pub(crate) fn retain(&mut self, ledger: &mut Ledger) -> Result<()> {
        if self.ownership == GraphOwnership::CellScratch {
            require_ledger(ledger)?;
            account::header::<Self>(ledger)?;
            self.ownership = GraphOwnership::CellRetained;
        }
        Ok(())
    }

    fn visit_buffers(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        visit(W::buffer(&self.units, self.units.len())?)?;
        for unit in &self.units {
            visit(W::buffer(&unit.atoms, unit.atoms.len())?)?;
            visit(W::buffer(&unit.atomics, unit.atomics.len())?)?;
            visit(W::buffer(
                &unit.resolved_scripts,
                unit.resolved_scripts.len(),
            )?)?;
            visit(W::buffer(&unit.scripts, unit.scripts.len())?)?;
            visit(W::buffer(
                &unit.context_scripts,
                unit.context_scripts.len(),
            )?)?;
        }
        visit(W::buffer(&self.boundaries, self.boundaries.len())?)?;
        visit(W::buffer(&self.regions, self.regions.len())?)?;
        for region in &self.regions {
            visit(W::buffer(&region.units, region.units.len())?)?;
            visit(W::buffer(&region.members, region.members.len())?)?;
        }
        self.index.visit(visit)?;
        Ok(())
    }

    pub(crate) fn visit_allocations(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        if self.ownership == GraphOwnership::CellRetained {
            visit(W::object(self, 0))?;
            self.visit_buffers(visit)?;
        }
        Ok(())
    }

    pub(crate) fn drop_scratch(self, ledger: &mut Ledger) -> Result<()> {
        if self.ownership == GraphOwnership::Native {
            drop(self);
            return Ok(());
        }
        require_ledger(ledger)?;
        let mut released = 0usize;
        self.visit_buffers(&mut |w| {
            account::work(ledger, w.records().checked_add(1).ok_or_else(overflow)?)?;
            released = released
                .checked_add(w.payload_bytes())
                .ok_or_else(overflow)?;
            Ok(())
        })?;
        if self.ownership == GraphOwnership::CellRetained {
            released = released
                .checked_add(std::mem::size_of::<Self>())
                .ok_or_else(overflow)?;
        }
        drop(self);
        ledger
            .as_mut()
            .expect("cell graph ledger")
            .release_bytes(released)
    }
}

fn overflow() -> crate::Error {
    crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
        kind: crate::ScopeAlignmentLimitKind::PayloadBytes,
    })
}

pub(crate) fn visit_gap_allocations(
    shaped: &ShapedData,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    if shaped.character_gap_ownership == GapArenaOwnership::Funded {
        visit(W::object(&shaped.character_gaps, 0))?;
        visit(W::buffer(
            &shaped.character_gaps,
            shaped.character_gaps.len(),
        )?)?;
        visit(W::object(&shaped.atomic_gaps, 0))?;
        visit(W::buffer(
            &shaped.atomic_gaps.rows,
            shaped.atomic_gaps.rows.len(),
        )?)?;
    }
    Ok(())
}

pub(crate) fn visit_shaped_allocations(
    shaped: &ShapedData,
    visit: &mut AllocationVisitor<'_>,
) -> Result<()> {
    if let Some(requirements) = &shaped.character_requirements {
        requirements.visit_allocations(visit)?;
    }
    visit_gap_allocations(shaped, visit)
}

pub(super) fn fund_gaps(shaped: &mut ShapedData, ledger: &mut Ledger) -> Result<()> {
    if shaped.character_gap_ownership == GapArenaOwnership::Funded {
        return Ok(());
    }
    require_ledger(ledger)?;
    for (records, bytes) in [
        (
            shaped.character_gaps.len(),
            account::bytes(&shaped.character_gaps)?,
        ),
        (
            shaped.atomic_gaps.rows.len(),
            account::bytes(&shaped.atomic_gaps.rows)?,
        ),
    ] {
        let budget = ledger.as_mut().expect("cell gap ledger");
        budget.work(records.checked_add(1).ok_or_else(overflow)?)?;
        budget.add_records(records)?;
        budget.retain_bytes(bytes)?;
    }
    account::header::<Vec<Gap>>(ledger)?;
    account::header::<crate::engine::layout::data::OrdinalMap<f64>>(ledger)?;
    shaped.character_gap_ownership = GapArenaOwnership::Funded;
    Ok(())
}

pub(crate) fn cell_gap_indices(
    shaped: &ShapedData,
    ledger: &mut Ledger,
) -> Result<Vec<Option<usize>>> {
    if shaped.cells.is_empty() || shaped.character_gaps.is_empty() {
        return Ok(Vec::new());
    }
    let mut rows = account::filled(ledger, shaped.cells.len(), None)?;
    for (index, gap) in shaped.character_gaps.iter().enumerate() {
        account::work(ledger, 1)?;
        if let Owner::Cell(cell) = gap.owner {
            let row = rows.get_mut(cell).ok_or_else(|| {
                crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "character gap has no actual cell",
                )
            })?;
            if row.replace(index).is_some() {
                return Err(crate::Error::new(
                    crate::ErrorCode::LayoutFailed,
                    "duplicate actual cell gap",
                ));
            }
        }
    }
    Ok(rows)
}

pub(super) fn drop_unit(unit: Unit, ledger: &mut Ledger) -> Result<()> {
    account::drop_vec(ledger, unit.atoms)?;
    account::drop_vec(ledger, unit.atomics)?;
    account::drop_vec(ledger, unit.scripts)?;
    account::drop_vec(ledger, unit.resolved_scripts)?;
    account::drop_vec(ledger, unit.context_scripts)
}
