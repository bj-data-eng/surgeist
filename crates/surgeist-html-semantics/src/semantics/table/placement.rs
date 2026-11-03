//! Placing cells row by row (HTML 877d §4.9.12.1, the algorithm for processing
//! rows): the next free column, the cells a new cell overlaps, and the growth
//! of cells that grow downward.
//!
//! Every question about the row being placed is answered in `O(log)` steps plus
//! its output, however many tall cells stand in the way, and a cell is touched
//! when it is placed and when it is settled, not once per row it crosses:
//!
//! - which columns are free is a descent of the [`Occupancy`] tree, which never
//!   removes a cell because a column is free once its latest end is reached;
//! - a new cell starts at a free column, so the cells it overlaps start inside
//!   it, to the right of that column: they are found through the cells anchored
//!   at each of those columns, newest first, where the cells whose end is
//!   reached are dropped the first time they are met;
//! - a cell that grows downward covers its columns until its group ends and is
//!   given its height then, once, from the last row the growth reached.

use super::occupancy::{Columns, Occupancy};
use super::*;

/// Until which row a placed cell covers its columns.
#[derive(Clone, Copy, Debug)]
pub(super) enum Reach {
    /// The cell covers rows up to, not including, this one.
    Until(usize),
    /// The cell grows downward and covers every row of its group so far.
    Growing,
}

/// Where a cell is placed: its first column and row, and one past its last
/// column.
#[derive(Clone, Copy, Debug)]
pub(super) struct Footprint {
    pub(super) x: usize,
    pub(super) y: usize,
    pub(super) right: usize,
}

/// Two cells that share a slot, by their place in the model.
#[derive(Clone, Copy, Debug)]
pub(super) struct Overlap {
    pub(super) cell: usize,
    pub(super) other: usize,
}

/// What formation keeps while it places cells row by row.
#[derive(Default)]
pub(super) struct Placement {
    /// The cells that grow downward in the current group, in model order.
    downward: Vec<usize>,
    /// The last row the growth of downward-growing cells reached.
    through: usize,
    occupancy: Occupancy,
    /// For each column, the newest cell anchored there that may still cover
    /// a row.
    anchors: Vec<Option<usize>>,
    /// For each cell, the next older cell anchored in the same column.
    links: Vec<Option<usize>>,
    reaches: Vec<Reach>,
    /// Earlier cells a new cell overlaps, before they are reported in order.
    hits: Vec<usize>,
    /// Every pair of cells that share a slot: the lines the header scans
    /// cannot take one occupant at a time are the lines these cross.
    pub(super) overlaps: Vec<Overlap>,
}

impl Placement {
    /// Every capacity admitted for this placement's scratch.
    pub(super) fn capacities(&self) -> [usize; 9] {
        let [low, floor, ceil] = self.occupancy.capacities();
        [
            self.downward.capacity(),
            self.anchors.capacity(),
            self.links.capacity(),
            self.reaches.capacity(),
            self.hits.capacity(),
            self.overlaps.capacity(),
            low,
            floor,
            ceil,
        ]
    }
}

impl<'a> HtmlSemanticSession<'a> {
    /// Makes the placement ready for `columns` columns: a head for every
    /// column and a tree wide enough. A widening at least doubles the tree, so
    /// the work of all widenings is that of the last.
    pub(super) fn table_columns_ready(
        &mut self,
        place: &mut Placement,
        columns: usize,
    ) -> TableResult<()> {
        if place.anchors.len() < columns {
            let more = columns - place.anchors.len();
            self.charge(more)?;
            self.reserve_scratch(&mut place.anchors, more)?;
            place.anchors.resize(columns, None);
        }
        let leaves = Occupancy::leaves_for(columns);
        if leaves > place.occupancy.leaves() {
            let nodes = Occupancy::nodes_for(leaves);
            self.charge(nodes.saturating_add(place.occupancy.leaves()))?;
            for array in place.occupancy.arrays() {
                let more = nodes - array.len();
                self.reserve_scratch(array, more)?;
            }
            place.occupancy.widen(leaves);
        }
        Ok(())
    }

    /// The first column at or after `from` that has no cell in row `y`, or
    /// `width`: HTML's loop that increases the current column while its slot
    /// "already has a cell assigned to it".
    pub(super) fn table_first_free(
        &mut self,
        place: &mut Placement,
        from: usize,
        y: usize,
        width: usize,
    ) -> TableResult<usize> {
        if from >= width {
            return Ok(from);
        }
        let found = place.occupancy.first_at_most(from, y, &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        Ok(found.map_or(width, |column| column.min(width)))
    }

    /// Records the recovered overlap errors of a cell about to be placed at
    /// `footprint`: one per earlier cell that shares a slot with it, in model
    /// order. The cell starts at a free column, so a cell it overlaps starts to
    /// the right of that column and covers row `y`: rows are placed in order
    /// and a cell reaches only downward from its own row.
    pub(super) fn table_overlaps(
        &mut self,
        model: &mut HtmlTableModel,
        cell: &HtmlCaptureSubjectId,
        footprint: Footprint,
        place: &mut Placement,
    ) -> TableResult<()> {
        let Footprint { x, y, right } = footprint;
        place.hits.clear();
        for column in x.saturating_add(1)..right {
            self.charge(1)?;
            let mut previous: Option<usize> = None;
            let mut cursor = place.anchors.get(column).copied().flatten();
            while let Some(candidate) = cursor {
                self.charge(1)?;
                let next = place.links.get(candidate).copied().flatten();
                let live = match place.reaches.get(candidate) {
                    Some(Reach::Until(end)) => *end > y,
                    Some(Reach::Growing) => true,
                    None => false,
                };
                if live {
                    self.table_scratch(&mut place.hits, candidate)?;
                    previous = Some(candidate);
                } else if let Some(older) = previous {
                    if let Some(link) = place.links.get_mut(older) {
                        *link = next;
                    }
                } else if let Some(head) = place.anchors.get_mut(column) {
                    *head = next;
                }
                cursor = next;
            }
        }
        // Sorting charges one unit per element for each bit of the count.
        let overlapping = place.hits.len();
        let bits = overlapping
            .checked_ilog2()
            .and_then(|bits| usize::try_from(bits).ok())
            .unwrap_or(0);
        self.charge(overlapping.saturating_mul(1 + bits))?;
        place.hits.sort_unstable();
        let new = model.cells.len();
        for position in 0..place.hits.len() {
            self.charge(1)?;
            let other = place.hits[position];
            self.table_scratch(&mut place.overlaps, Overlap { cell: new, other })?;
        }
        for &index in &place.hits {
            self.charge(1)?;
            let Some(other) = model.cells.get(index) else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            let error = HtmlTableModelError::Overlap {
                cell: cell.clone(),
                other: other.subject.clone(),
                x: x.max(other.x),
                y: y.max(other.y),
            };
            self.table_push(&mut model.errors, error)?;
        }
        Ok(())
    }

    /// Records the placed cell `index`, which starts the cells anchored at its
    /// column and covers its columns until `reach`. The cell is already the
    /// last of the model's cells.
    pub(super) fn table_occupy(
        &mut self,
        place: &mut Placement,
        index: usize,
        footprint: Footprint,
        reach: Reach,
    ) -> TableResult<()> {
        let older = place.anchors.get(footprint.x).copied().flatten();
        self.table_scratch(&mut place.links, older)?;
        self.table_scratch(&mut place.reaches, reach)?;
        if let Some(head) = place.anchors.get_mut(footprint.x) {
            *head = Some(index);
        }
        let end = match reach {
            Reach::Until(end) => end,
            Reach::Growing => usize::MAX,
        };
        let columns = Columns {
            first: footprint.x,
            end: footprint.right,
        };
        place.occupancy.raise(columns, end, &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        if matches!(reach, Reach::Growing) {
            self.table_scratch(&mut place.downward, index)?;
        }
        Ok(())
    }

    /// HTML's "algorithm for growing downward-growing cells" at row `y`: every
    /// such cell also covers row `y`. The cells are given their heights when
    /// the group ends, from the last row growth reached.
    pub(super) fn table_grow(&mut self, place: &mut Placement, y: usize) -> TableResult<()> {
        self.charge(1)?;
        place.through = y;
        Ok(())
    }

    /// Gives each downward-growing cell the height growth has reached, frees
    /// its columns for the next group and empties the list.
    pub(super) fn table_settle(
        &mut self,
        model: &mut HtmlTableModel,
        place: &mut Placement,
    ) -> TableResult<()> {
        for position in 0..place.downward.len() {
            self.charge(1)?;
            let Some(index) = place.downward.get(position).copied() else {
                break;
            };
            let Some(cell) = model.cells.get_mut(index) else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            let height = place.through.saturating_sub(cell.y).saturating_add(1);
            cell.height = height;
            let columns = Columns {
                first: cell.x,
                end: cell.x.saturating_add(cell.width),
            };
            let end = cell.y.saturating_add(height);
            if let Some(reach) = place.reaches.get_mut(index) {
                *reach = Reach::Until(end);
            }
            place.occupancy.clear(columns, &mut |work| {
                self.charge(work).map_err(TableStop::from)
            })?;
        }
        self.charge(1)?;
        place.downward.clear();
        Ok(())
    }
}
