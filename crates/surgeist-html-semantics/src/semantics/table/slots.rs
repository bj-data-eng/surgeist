//! Interval index over the slots of a formed table.
//!
//! HTML 877d §4.9.12.2 (assigning header cells) asks about slots: which cell
//! covers a slot, which headers lie before it in its row or column. A cell
//! covers a rectangle, so within one row (or column) it covers one interval of
//! slots. A line holds those intervals sorted by start, and every question is
//! a binary search plus a walk over the few intervals that can answer it,
//! instead of a scan of every cell. Overlapping cells (a recovered error) keep
//! their intervals, so a slot can be covered by several cells; the running
//! maximum of the interval ends (`reach`) answers coverage exactly in that
//! case too.
//!
//! The helpers are pure and allocate nothing. Each takes the `charge` of its
//! caller and pays for a step before taking it, so a refusal leaves the work
//! undone.

#[cfg(test)]
mod tests;

/// Pays for `steps` units of work, or refuses.
pub(super) trait Charge<E>: FnMut(usize) -> Result<(), E> {}
impl<E, F: FnMut(usize) -> Result<(), E>> Charge<E> for F {}

/// One cell's interval of slots in one row or column of a finished table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SlotEntry {
    /// First covered slot along the line.
    pub(super) start: usize,
    /// One past the last covered slot along the line.
    pub(super) end: usize,
    /// Index of the cell in the model.
    pub(super) cell: usize,
    /// Largest `end` of this and every earlier entry of the line.
    pub(super) reach: usize,
    /// Index in the axis' entry table of the nearest header cell at or before
    /// this entry in its line.
    pub(super) header: Option<usize>,
}

/// A finished line: its entries and what they are.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SlotLine {
    /// Index of the line's first entry in the axis' entry table.
    pub(super) first: usize,
    /// One past the index of its last entry.
    pub(super) end: usize,
    /// No two entries of the line share a slot, so the entries are the line's
    /// distinct occupants in order.
    pub(super) regular: bool,
    /// Some entry is a data cell.
    pub(super) data: bool,
}

/// A finished line and the entries it holds.
#[derive(Clone, Copy, Debug)]
pub(super) struct LineView<'a> {
    pub(super) line: SlotLine,
    /// The line's entries, which start at `line.first` in the entry table.
    pub(super) entries: &'a [SlotEntry],
}

/// Lines `first..last` of an axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LineSpan {
    pub(super) first: usize,
    pub(super) last: usize,
}

/// Every row (or every column) of a finished table. Consecutive rows that the
/// same cells cover alike are one line, so a cell that spans thousands of
/// rows adds one entry to each line it crosses, not one per row; `breaks`
/// holds the first row of every line.
#[derive(Debug, Default)]
pub(super) struct SlotAxis {
    pub(super) breaks: Vec<usize>,
    pub(super) lines: Vec<SlotLine>,
    pub(super) entries: Vec<SlotEntry>,
}

impl SlotAxis {
    pub(super) fn line(&self, index: usize) -> Option<LineView<'_>> {
        let line = *self.lines.get(index)?;
        let entries = self.entries.get(line.first..line.end)?;
        Some(LineView { line, entries })
    }

    /// The lines that hold the positions `start..end` (`end` past `start`).
    pub(super) fn lines_of<E>(
        &self,
        start: usize,
        end: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<LineSpan, E> {
        let first = self.line_holding(start, charge)?;
        let last = self.line_holding(end.saturating_sub(1), charge)?;
        Ok(LineSpan {
            first,
            last: last + 1,
        })
    }

    fn line_holding<E>(&self, position: usize, charge: &mut impl Charge<E>) -> Result<usize, E> {
        let begun = leading_by(
            self.breaks.len(),
            |index| self.breaks.get(index).is_some_and(|b| *b <= position),
            charge,
        )?;
        Ok(begun.saturating_sub(1))
    }
}

/// The cells covering one slot of a line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Cover {
    Empty,
    One(usize),
    Many,
}

/// How many leading positions of `0..len` satisfy `holds`, which must hold
/// for a prefix. Each probe is paid for before it is made.
pub(super) fn leading_by<E>(
    len: usize,
    holds: impl Fn(usize) -> bool,
    charge: &mut impl Charge<E>,
) -> Result<usize, E> {
    let (mut low, mut high) = (0, len);
    while low < high {
        charge(1)?;
        let middle = low + (high - low) / 2;
        if holds(middle) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low)
}

/// [`leading_by`] over the entries of a line.
pub(super) fn leading<E>(
    entries: &[SlotEntry],
    holds: impl Fn(&SlotEntry) -> bool,
    charge: &mut impl Charge<E>,
) -> Result<usize, E> {
    leading_by(
        entries.len(),
        |index| entries.get(index).is_some_and(&holds),
        charge,
    )
}

/// Where a group of rows or columns lies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Extent {
    pub(super) start: usize,
    pub(super) length: usize,
}

/// The one group of `groups` holding `slot`, if any. `extent` gives where a
/// group lies; groups are sorted and disjoint.
pub(super) fn containing<T, E>(
    groups: &[T],
    slot: usize,
    extent: impl Fn(&T) -> Extent,
    charge: &mut impl Charge<E>,
) -> Result<Option<usize>, E> {
    let begun = leading_by(
        groups.len(),
        |index| {
            groups
                .get(index)
                .is_some_and(|group| extent(group).start <= slot)
        },
        charge,
    )?;
    charge(1)?;
    Ok(begun.checked_sub(1).filter(|index| {
        groups.get(*index).is_some_and(|group| {
            let Extent { start, length } = extent(group);
            slot - start < length
        })
    }))
}

/// Which cells cover `slot` of a line.
pub(super) fn cover<E>(
    entries: &[SlotEntry],
    slot: usize,
    charge: &mut impl Charge<E>,
) -> Result<Cover, E> {
    let count = leading(entries, |e| e.start <= slot, charge)?;
    charge(1)?;
    let mut found = None;
    let mut index = count;
    // An entry before `index` can cover the slot only while the reach through
    // it does; once the reach falls to the slot, no earlier entry covers it.
    while index > 0 && entries.get(index - 1).is_some_and(|e| e.reach > slot) {
        charge(1)?;
        index -= 1;
        if let Some(entry) = entries.get(index).filter(|e| e.end > slot) {
            if found.is_some() {
                return Ok(Cover::Many);
            }
            found = Some(entry.cell);
        }
    }
    Ok(found.map_or(Cover::Empty, Cover::One))
}
