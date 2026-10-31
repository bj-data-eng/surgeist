//! Derived incidence into one immutable graph's actual units.
use super::*;
use crate::engine::scope_alignment::census::{AllocationVisitor, AllocationWitness as W};
use crate::processing::contextual::alignment::operations;

#[derive(Debug)]
struct TextRow {
    atoms: Range<usize>,
    unit: usize,
}

#[derive(Debug, Default)]
pub(super) struct Index {
    text: Vec<TextRow>,
    atomics: Vec<(usize, usize)>,
    cells: Vec<(usize, usize)>,
}

impl Index {
    pub(super) fn new(units: &[Unit], ledger: &mut Ledger) -> Result<Self> {
        let mut result = Self::default();
        for (unit, row) in units.iter().enumerate() {
            account::work(ledger, 1)?;
            for atoms in &row.atoms {
                account::push(
                    ledger,
                    &mut result.text,
                    TextRow {
                        atoms: atoms.clone(),
                        unit,
                    },
                )?;
            }
            for &occurrence in &row.atomics {
                account::push(ledger, &mut result.atomics, (occurrence, unit))?;
            }
            if let Subject::Cell(binding) = row.subject {
                account::push(ledger, &mut result.cells, (binding, unit))?;
            }
        }
        operations::order_by(&mut result.text, ledger.as_mut(), |a, b| {
            (a.atoms.start, a.atoms.end, a.unit).cmp(&(b.atoms.start, b.atoms.end, b.unit))
        })?;
        operations::order_by(&mut result.atomics, ledger.as_mut(), Ord::cmp)?;
        operations::order_by(&mut result.cells, ledger.as_mut(), Ord::cmp)?;
        Ok(result)
    }
    pub(super) fn atom(&self, atom: usize, ledger: &mut Ledger) -> Result<Option<usize>> {
        let end = operations::partition_point(&self.text, ledger.as_mut(), |row| {
            row.atoms.start <= atom
        })?;
        account::work(ledger, 1)?;
        Ok(end
            .checked_sub(1)
            .and_then(|i| self.text.get(i))
            .filter(|row| row.atoms.contains(&atom))
            .map(|row| row.unit))
    }
    fn sparse(rows: &[(usize, usize)], key: usize, ledger: &mut Ledger) -> Result<Option<usize>> {
        Ok(
            operations::binary_search_by(rows, ledger.as_mut(), |row| row.0.cmp(&key))?
                .ok()
                .map(|i| rows[i].1),
        )
    }
    fn item(
        &self,
        shaped: &ShapedData,
        item: ShapedItem,
        filtered: bool,
        ledger: &mut Ledger,
    ) -> Result<Option<(usize, Owner)>> {
        account::work(ledger, 1)?;
        Ok(match item {
            ShapedItem::TextCluster(i) => {
                let atom = shaped.clusters[i].atoms.start;
                if filtered
                    && operations::binary_search_by(
                        &shaped.character_exclusions.rows,
                        ledger.as_mut(),
                        |row| row.0.cmp(&atom),
                    )?
                    .is_ok()
                {
                    None
                } else {
                    self.atom(atom, ledger)?.map(|u| (u, Owner::Cluster(i)))
                }
            }
            ShapedItem::InlineBox { occurrence, .. } => {
                Self::sparse(&self.atomics, occurrence, ledger)?
                    .map(|u| (u, Owner::Atomic(occurrence)))
            }
            ShapedItem::Cell(i) => Self::sparse(&self.cells, shaped.cells[i].binding, ledger)?
                .map(|u| (u, Owner::Cell(i))),
            ShapedItem::Anchor { .. } => None,
        })
    }
    pub(super) fn original_order(
        &self,
        shaped: &ShapedData,
        units: &[Unit],
        ledger: &mut Ledger,
    ) -> Result<Vec<usize>> {
        let mut keys = Vec::new();
        for (ordinal, &item) in shaped.items.iter().enumerate() {
            if let Some((u, _)) = self.item(shaped, item, false, ledger)? {
                account::push(ledger, &mut keys, (u, ordinal))?;
            }
        }
        operations::order_by(&mut keys, ledger.as_mut(), |a, b| {
            let key = |(u, i): (usize, usize)| {
                (
                    u,
                    if units[u].level & 1 == 0 {
                        i
                    } else {
                        usize::MAX - i
                    },
                )
            };
            key(*a).cmp(&key(*b))
        })?;
        let order = account::collect(ledger, keys.iter().map(|row| row.1))?;
        account::drop_vec(ledger, keys)?;
        Ok(order)
    }
    pub(super) fn owners(
        &self,
        shaped: &ShapedData,
        order: &[usize],
        count: usize,
        ledger: &mut Ledger,
    ) -> Result<Vec<Option<Owner>>> {
        let mut owners = account::filled(ledger, count, None)?;
        for &i in order {
            if let Some((unit, owner)) = self.item(shaped, shaped.items[i], true, ledger)? {
                owners[unit] = Some(owner);
            }
        }
        Ok(owners)
    }
    pub(super) fn visit(&self, visit: &mut AllocationVisitor<'_>) -> Result<()> {
        visit(W::buffer(&self.text, self.text.len())?)?;
        visit(W::buffer(&self.atomics, self.atomics.len())?)?;
        visit(W::buffer(&self.cells, self.cells.len())?)
    }
}
