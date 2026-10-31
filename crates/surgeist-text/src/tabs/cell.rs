//! Fixed genuine container references and original-atom inner TAB operations.
use crate::Result;
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;

pub(crate) struct CellTabMetric {
    pub(crate) advance: f64,
    pub(crate) font: Option<crate::engine::layout::data::PreparedRun>,
}
pub(crate) struct CellTabReference {
    pub(crate) scope: crate::CapturedScope,
    pub(crate) direction: crate::ParagraphBaseDirection,
    pub(crate) space: Option<CellTabMetric>,
    pub(crate) ch: Option<CellTabMetric>,
}
#[derive(Clone, Copy)]
pub(crate) struct CellTabOperation {
    pub(crate) atom: usize,
    pub(crate) reference: usize,
    pub(crate) size: crate::TabSize,
    pub(crate) period: f64,
    pub(crate) threshold: f64,
}
#[derive(Default)]
pub(crate) struct CellTabPlan {
    pub(crate) operations: Vec<CellTabOperation>,
}
impl CellTabPlan {
    pub(crate) fn operation(
        &self,
        atom: usize,
        ledger: &mut Ledger,
    ) -> Result<Option<&CellTabOperation>> {
        Ok(
            operations::binary_search_by(&self.operations, ledger.as_mut(), |row| {
                row.atom.cmp(&atom)
            })?
            .ok()
            .map(|index| &self.operations[index]),
        )
    }
    pub(crate) fn contains(&self, atom: usize, ledger: &mut Ledger) -> Result<bool> {
        if self.operations.is_empty() {
            return Ok(false);
        }
        self.operation(atom, ledger).map(|row| row.is_some())
    }
    pub(crate) fn for_range(
        &self,
        range: std::ops::Range<usize>,
        ledger: &mut Ledger,
    ) -> Result<&[CellTabOperation]> {
        let start = operations::partition_point(&self.operations, ledger.as_mut(), |row| {
            row.atom < range.start
        })?;
        let end = operations::partition_point(&self.operations, ledger.as_mut(), |row| {
            row.atom < range.end
        })?;
        Ok(&self.operations[start..end])
    }
    pub(crate) fn drop(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.operations)
    }
}

/// Genuine original provider-projection custody, independent of style identity.
pub(crate) fn owns_atom(
    analyzed: &crate::engine::AnalyzedData,
    atom: usize,
    ledger: &mut Ledger,
) -> Result<bool> {
    let Some(output) = &analyzed.combination else {
        return Ok(false);
    };
    if output.bindings.is_empty() {
        return Ok(false);
    }
    let index = operations::partition_point(&output.bindings, ledger.as_mut(), |binding| {
        binding.atoms.end <= atom
    })?;
    Ok(output
        .bindings
        .get(index)
        .is_some_and(|binding| binding.atoms.contains(&atom)))
}
