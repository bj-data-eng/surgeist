//! Which columns of the row being placed are free.
//!
//! HTML 877d §4.9.12.1 (processing rows) moves the current column past every
//! slot that "already has a cell assigned to it". A slot of row `y` has a cell
//! exactly when some placed cell covers it and ends below `y`, so the question
//! is the first column at or after a given one whose latest covering end is at
//! most `y`. The tree keeps, for every column, the greatest end below which a
//! placed cell covers it; placing a cell raises a range of columns to its end,
//! a cell that grows downward raises its columns past any end until its group
//! ends and clears them then, and the next free column is a descent. Every
//! operation takes `O(log columns)` steps, each paid for before it is taken,
//! however many tall cells stand in the way.
//!
//! The tree is a heap-shaped array over a power-of-two number of columns. A
//! node holds the least value below it and a pending [`Clamp`] for its
//! children, which raising (a floor, no ceiling) and clearing (ceiling zero)
//! compose into.

use super::slots::Charge;

#[cfg(test)]
mod tests;

/// Columns whose end is greater than `y` hold a cell at row `y`.
#[derive(Debug, Default)]
pub(super) struct Occupancy {
    size: usize,
    low: Vec<usize>,
    floor: Vec<usize>,
    ceil: Vec<usize>,
}

/// A range of columns: `first` up to, not including, `end`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Columns {
    pub(super) first: usize,
    pub(super) end: usize,
}

/// A value change: `max(floor, min(ceil, v))`.
#[derive(Clone, Copy, Debug)]
struct Clamp {
    floor: usize,
    ceil: usize,
}

impl Clamp {
    const KEEP: Self = Self {
        floor: 0,
        ceil: usize::MAX,
    };

    fn apply(self, value: usize) -> usize {
        value.max(self.floor).min(self.ceil)
    }
}

impl Occupancy {
    /// The number of leaves a tree over `columns` columns has.
    pub(super) fn leaves_for(columns: usize) -> usize {
        columns.next_power_of_two()
    }

    pub(super) fn leaves(&self) -> usize {
        self.size
    }

    /// The length of each of the three node arrays for `leaves` leaves.
    pub(super) fn nodes_for(leaves: usize) -> usize {
        leaves.saturating_mul(2)
    }

    /// The capacities of the three node arrays, as admitted.
    pub(super) fn capacities(&self) -> [usize; 3] {
        [
            self.low.capacity(),
            self.floor.capacity(),
            self.ceil.capacity(),
        ]
    }

    /// The three node arrays, so their room can be admitted before a widening.
    pub(super) fn arrays(&mut self) -> [&mut Vec<usize>; 3] {
        [&mut self.low, &mut self.floor, &mut self.ceil]
    }

    /// Widens the tree to `leaves` leaves, keeping every column's value; new
    /// columns are free. The node arrays must already have room for
    /// [`Self::nodes_for`]`(leaves)` entries, and `leaves` must be a power of
    /// two that is at least twice the current number (or any, from none).
    pub(super) fn widen(&mut self, leaves: usize) {
        for node in 1..self.size {
            self.push_down(node);
        }
        let old = self.size;
        let nodes = Self::nodes_for(leaves);
        self.low.resize(nodes, 0);
        self.floor.resize(nodes, 0);
        self.ceil.resize(nodes, usize::MAX);
        if old > 0 {
            self.low.copy_within(old..2 * old, leaves);
        }
        self.floor.fill(0);
        self.ceil.fill(usize::MAX);
        for node in (1..leaves).rev() {
            self.low[node] = self.low[2 * node].min(self.low[2 * node + 1]);
        }
        self.size = leaves;
    }

    fn apply(&mut self, node: usize, change: Clamp) {
        self.low[node] = change.apply(self.low[node]);
        if node < self.size {
            self.floor[node] = change.apply(self.floor[node]);
            self.ceil[node] = change.apply(self.ceil[node]);
        }
    }

    fn push_down(&mut self, node: usize) {
        let pending = Clamp {
            floor: self.floor[node],
            ceil: self.ceil[node],
        };
        if pending.floor != Clamp::KEEP.floor || pending.ceil != Clamp::KEEP.ceil {
            self.apply(2 * node, pending);
            self.apply(2 * node + 1, pending);
            self.floor[node] = Clamp::KEEP.floor;
            self.ceil[node] = Clamp::KEEP.ceil;
        }
    }

    /// Raises every column of `columns` to at least `end`.
    pub(super) fn raise<E>(
        &mut self,
        columns: Columns,
        end: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<(), E> {
        let change = Clamp {
            floor: end,
            ceil: usize::MAX,
        };
        self.clamp(1, 0, self.size, columns, change, charge)
    }

    /// Sets every column of `columns` to zero.
    pub(super) fn clear<E>(
        &mut self,
        columns: Columns,
        charge: &mut impl Charge<E>,
    ) -> Result<(), E> {
        let change = Clamp { floor: 0, ceil: 0 };
        self.clamp(1, 0, self.size, columns, change, charge)
    }

    fn clamp<E>(
        &mut self,
        node: usize,
        low: usize,
        high: usize,
        columns: Columns,
        change: Clamp,
        charge: &mut impl Charge<E>,
    ) -> Result<(), E> {
        charge(1)?;
        if high <= columns.first || columns.end <= low || low >= high {
            return Ok(());
        }
        if columns.first <= low && high <= columns.end {
            self.apply(node, change);
            return Ok(());
        }
        self.push_down(node);
        let middle = low + (high - low) / 2;
        self.clamp(2 * node, low, middle, columns, change, charge)?;
        self.clamp(2 * node + 1, middle, high, columns, change, charge)?;
        self.low[node] = self.low[2 * node].min(self.low[2 * node + 1]);
        Ok(())
    }

    /// The first column at or after `from` whose value is at most `most`.
    pub(super) fn first_at_most<E>(
        &mut self,
        from: usize,
        most: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<Option<usize>, E> {
        self.find(1, 0, self.size, from, most, charge)
    }

    fn find<E>(
        &mut self,
        node: usize,
        low: usize,
        high: usize,
        from: usize,
        most: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<Option<usize>, E> {
        charge(1)?;
        if high <= from || low >= high || self.low[node] > most {
            return Ok(None);
        }
        if high - low == 1 {
            return Ok(Some(low));
        }
        self.push_down(node);
        let middle = low + (high - low) / 2;
        if let Some(found) = self.find(2 * node, low, middle, from, most, charge)? {
            return Ok(Some(found));
        }
        self.find(2 * node + 1, middle, high, from, most, charge)
    }
}
