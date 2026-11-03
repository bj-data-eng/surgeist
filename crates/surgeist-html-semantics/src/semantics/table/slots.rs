//! Interval index over table slots.
//!
//! HTML 877d §4.9.12.1 (forming a table) and §4.9.12.2 (assigning header cells)
//! ask about slots: whether a slot is covered, which cell covers it, which
//! headers lie before it in its row or column. A cell covers a rectangle, so
//! within one row (or column) it covers one interval of slots. A line holds
//! those intervals sorted by start, and every question is a binary search plus
//! a walk over the few intervals that can answer it, instead of a scan of
//! every cell. Overlapping cells (a recovered error) keep their intervals, so
//! a slot can be covered by several cells; the running maximum of the
//! interval ends (`reach`) answers coverage exactly in that case too.
//!
//! These helpers are pure: they report how many steps they took and never
//! charge or allocate. Their callers admit the storage and charge the steps.

/// No header at or before an entry.
pub(super) const NO_HEADER: usize = usize::MAX;

/// One cell's interval of slots in one row or column.
#[derive(Clone, Copy, Debug, Default)]
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
    /// this entry in its line, or [`NO_HEADER`]. Set when a line is finished.
    pub(super) header: usize,
}

/// A finished line's place in the entry table and what its entries are.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SlotLine {
    /// Index of the line's first entry; the next line's `offset` ends it.
    pub(super) offset: usize,
    /// No two entries of the line share a slot, so the entries are the line's
    /// distinct occupants in order.
    pub(super) regular: bool,
    /// Some entry is a data cell.
    pub(super) data: bool,
}

/// Every row (or every column) of a finished table. Consecutive rows that the
/// same cells cover alike are one line, so a cell that spans thousands of
/// rows adds one entry to each line it crosses, not one per row; `breaks`
/// holds the first row of every line. The lines end with one closing line that
/// only carries the end of the last.
#[derive(Debug, Default)]
pub(super) struct SlotAxis {
    pub(super) breaks: Vec<usize>,
    pub(super) lines: Vec<SlotLine>,
    pub(super) entries: Vec<SlotEntry>,
}

impl SlotAxis {
    /// The entries of line `index` with the line's facts and the entry-table
    /// index of its first entry.
    pub(super) fn line(&self, index: usize) -> Option<(&[SlotEntry], SlotLine, usize)> {
        let line = *self.lines.get(index)?;
        let end = self.lines.get(index + 1)?.offset;
        Some((self.entries.get(line.offset..end)?, line, line.offset))
    }

    /// The lines `first..last` that hold the positions `start..end` (`end`
    /// past `start`), and the probes made to find them.
    pub(super) fn lines_of(&self, start: usize, end: usize) -> (usize, usize, usize) {
        let holding = |position: usize| {
            let (count, probes) = leading_by(self.breaks.len(), |index| {
                self.breaks
                    .get(index)
                    .is_some_and(|first| *first <= position)
            });
            (count.saturating_sub(1), probes)
        };
        let (first, a) = holding(start);
        let (last, b) = holding(end.saturating_sub(1));
        (first, last + 1, a + b)
    }
}

/// The slots of one line covered by exactly one cell, none, or several.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Cover {
    Empty,
    One(usize),
    Many,
}

/// How many leading positions of `0..len` satisfy `holds`, which must hold for
/// a prefix. Also the number of probes made.
pub(super) fn leading_by(len: usize, holds: impl Fn(usize) -> bool) -> (usize, usize) {
    let (mut low, mut high, mut probes) = (0, len, 0);
    while low < high {
        probes += 1;
        let middle = low + (high - low) / 2;
        if holds(middle) {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    (low, probes)
}

/// [`leading_by`] over the entries of a line.
pub(super) fn leading(entries: &[SlotEntry], holds: impl Fn(&SlotEntry) -> bool) -> (usize, usize) {
    leading_by(entries.len(), |index| {
        entries.get(index).is_some_and(&holds)
    })
}

/// The one group of `groups` holding `slot`, if any, and the probes made.
/// `extent` gives a group's first slot and its length; groups are sorted and
/// disjoint.
pub(super) fn containing<T>(
    groups: &[T],
    slot: usize,
    extent: impl Fn(&T) -> (usize, usize),
) -> (Option<usize>, usize) {
    let (count, probes) = leading_by(groups.len(), |index| {
        groups
            .get(index)
            .is_some_and(|group| extent(group).0 <= slot)
    });
    let found = count.checked_sub(1).filter(|index| {
        groups.get(*index).is_some_and(|group| {
            let (start, length) = extent(group);
            slot - start < length
        })
    });
    (found, probes + 1)
}

/// Which cells cover `slot`, and the steps taken.
pub(super) fn cover(entries: &[SlotEntry], slot: usize) -> (Cover, usize) {
    let (count, probes) = leading(entries, |e| e.start <= slot);
    let mut work = probes + 1;
    let mut found = None;
    let mut index = count;
    // An entry before `index` can cover the slot only while the reach through
    // it does; once the reach falls to the slot, no earlier entry covers it.
    while index > 0 && entries.get(index - 1).is_some_and(|e| e.reach > slot) {
        index -= 1;
        work += 1;
        if let Some(entry) = entries.get(index).filter(|e| e.end > slot) {
            if found.is_some() {
                return (Cover::Many, work);
            }
            found = Some(entry.cell);
        }
    }
    (found.map_or(Cover::Empty, Cover::One), work)
}

/// The first slot at or after `from`, below `limit`, that no entry covers (or
/// `limit`), and the steps taken.
pub(super) fn first_free(entries: &[SlotEntry], mut from: usize, limit: usize) -> (usize, usize) {
    let mut work = 0;
    while from < limit {
        let (count, probes) = leading(entries, |e| e.start <= from);
        work += probes + 1;
        // The entry with the greatest end among those starting at or before
        // `from` covers every slot from `from` up to that end.
        match count.checked_sub(1).and_then(|last| entries.get(last)) {
            Some(entry) if entry.reach > from => from = entry.reach,
            _ => break,
        }
    }
    (from, work)
}

/// The entries that may overlap `[start, end)`: indices `first..last` hold
/// every one, among others whose own end is not past `start`. Also the probes.
pub(super) fn overlap_window(
    entries: &[SlotEntry],
    start: usize,
    end: usize,
) -> (usize, usize, usize) {
    let (last, a) = leading(entries, |e| e.start < end);
    let (first, b) = entries
        .get(..last)
        .map_or((0, 0), |prefix| leading(prefix, |e| e.reach <= start));
    (first, last, a + b + 1)
}

/// Inserts `entry` after the entries that start at or before it and repairs
/// the running reach. The vector must have room: the caller admits it.
/// Returns the steps taken.
pub(super) fn insert(entries: &mut Vec<SlotEntry>, mut entry: SlotEntry) -> usize {
    let (position, probes) = match entries.last() {
        Some(last) if last.start > entry.start => leading(entries, |e| e.start <= entry.start),
        _ => (entries.len(), 1),
    };
    let before = position
        .checked_sub(1)
        .and_then(|index| entries.get(index))
        .map_or(0, |e| e.reach);
    entry.reach = before.max(entry.end);
    let mut reach = entry.reach;
    entries.insert(position, entry);
    let mut work = probes + (entries.len() - position);
    for later in entries.iter_mut().skip(position + 1) {
        work += 1;
        let repaired = reach.max(later.end);
        if repaired == later.reach {
            break;
        }
        later.reach = repaired;
        reach = repaired;
    }
    work
}
