//! Transaction-owned indexes over actual finalized fragment owners.
use super::*;
use crate::engine::scope_alignment::accounting::{self as account, Ledger};
use crate::processing::contextual::alignment::operations;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum OwnerKey {
    Origin(usize),
    Alternative(usize),
    Control(usize),
    Virtual(usize),
}
impl From<FragmentOwner> for OwnerKey {
    fn from(owner: FragmentOwner) -> Self {
        match owner {
            FragmentOwner::Origin(index) => Self::Origin(index),
            FragmentOwner::Alternative(index) => Self::Alternative(index),
            FragmentOwner::Control(index) => Self::Control(index),
            FragmentOwner::VirtualSeparator(index) => Self::Virtual(index),
        }
    }
}
#[derive(Clone, Copy)]
struct FragmentIndex {
    owner: OwnerKey,
    source: Option<Range>,
    fragment: usize,
}
pub(super) struct LineIndexes {
    fragments: Vec<FragmentIndex>,
    cell_clusters: Vec<(usize, usize)>,
}
impl LineIndexes {
    pub(super) fn new(line: &crate::engine::FinalizedLine, ledger: &mut Ledger) -> Result<Self> {
        let mut fragments = Vec::new();
        for (index, fragment) in line.data.fragments.iter().enumerate() {
            account::work(ledger, 1)?;
            let source = if matches!(fragment.association.owner, FragmentOwner::Origin(_)) {
                let Some(source) = fragment
                    .association
                    .source
                    .filter(|piece| !piece.aggregate.is_empty())
                else {
                    continue;
                };
                Some(source.aggregate)
            } else {
                None
            };
            account::push(
                ledger,
                &mut fragments,
                FragmentIndex {
                    owner: fragment.association.owner.into(),
                    source,
                    fragment: index,
                },
            )?;
        }
        operations::order_by(&mut fragments, ledger.as_mut(), |a, b| {
            (a.owner, a.source.map(|r| (r.start, r.end)), a.fragment).cmp(&(
                b.owner,
                b.source.map(|r| (r.start, r.end)),
                b.fragment,
            ))
        })?;
        let mut cell_clusters = Vec::new();
        for (index, cluster) in line.data.clusters.iter().enumerate() {
            account::work(ledger, 1)?;
            if let crate::engine::layout::data::FinalizedRunFrame::Cell(cell) = cluster.frame {
                account::push(ledger, &mut cell_clusters, (cell, index))?;
            }
        }
        operations::order_by(&mut cell_clusters, ledger.as_mut(), Ord::cmp)?;
        Ok(Self {
            fragments,
            cell_clusters,
        })
    }
    fn rows(&self, owner: FragmentOwner, ledger: &mut Ledger) -> Result<&[FragmentIndex]> {
        let key = OwnerKey::from(owner);
        let first =
            operations::partition_point(&self.fragments, ledger.as_mut(), |row| row.owner < key)?;
        let count =
            operations::partition_point(&self.fragments[first..], ledger.as_mut(), |row| {
                row.owner == key
            })?;
        Ok(&self.fragments[first..first + count])
    }
    pub(super) fn append_original(
        &self,
        origin: usize,
        source: Range,
        result: &mut Vec<usize>,
        ledger: &mut Ledger,
    ) -> Result<()> {
        let rows = self.rows(FragmentOwner::Origin(origin), ledger)?;
        let first = operations::partition_point(rows, ledger.as_mut(), |row| {
            row.source
                .expect("original fragment index retains source")
                .end
                <= source.start
        })?;
        for row in &rows[first..] {
            account::work(ledger, 1)?;
            if row
                .source
                .expect("original fragment index retains source")
                .start
                >= source.end
            {
                break;
            }
            account::push(ledger, result, row.fragment)?;
        }
        Ok(())
    }
    pub(super) fn append_owner(
        &self,
        owner: FragmentOwner,
        result: &mut Vec<usize>,
        ledger: &mut Ledger,
    ) -> Result<()> {
        for row in self.rows(owner, ledger)? {
            account::work(ledger, 1)?;
            account::push(ledger, result, row.fragment)?;
        }
        Ok(())
    }
    pub(super) fn cell_clusters(
        &self,
        cell: usize,
        ledger: &mut Ledger,
    ) -> Result<&[(usize, usize)]> {
        let first =
            operations::partition_point(&self.cell_clusters, ledger.as_mut(), |row| row.0 < cell)?;
        let count =
            operations::partition_point(&self.cell_clusters[first..], ledger.as_mut(), |row| {
                row.0 == cell
            })?;
        Ok(&self.cell_clusters[first..first + count])
    }
    pub(super) fn drop_scratch(self, ledger: &mut Ledger) -> Result<()> {
        account::drop_vec(ledger, self.fragments)?;
        account::drop_vec(ledger, self.cell_clusters)
    }
}
