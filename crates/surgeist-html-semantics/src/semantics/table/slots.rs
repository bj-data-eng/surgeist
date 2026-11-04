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

use super::super::ScratchBacking;

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
    /// For a header, which lines it crosses, as the position of that extent
    /// among the axis' header extents. A scan hides a header behind an earlier
    /// header that crosses the same lines, so this is what is compared.
    pub(super) key: Option<usize>,
    /// For a header, the table index of the nearest earlier header of its line
    /// that crosses other lines.
    pub(super) differs: Option<usize>,
}

/// Where a group of lines, or of cells, lies.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Extent {
    pub(super) start: usize,
    pub(super) length: usize,
}

impl Extent {
    pub(super) fn end(&self) -> usize {
        self.start.saturating_add(self.length)
    }
}

/// A sorted list of entries inside an entry table, with the table index of
/// its first entry (the entries' header pointers are table indexes).
#[derive(Clone, Copy, Debug)]
pub(super) struct List<'a> {
    pub(super) entries: &'a [SlotEntry],
    pub(super) first: usize,
}

impl<'a> List<'a> {
    pub(super) const EMPTY: List<'static> = List {
        entries: &[],
        first: 0,
    };

    /// The nearest header at or before `position` that crosses lines other
    /// than `key`.
    pub(super) fn header_other_than(
        &self,
        position: Option<usize>,
        key: usize,
    ) -> Option<&'a SlotEntry> {
        let header = self.entries.get(self.header_at(position?)?)?;
        if header.key != Some(key) {
            return Some(header);
        }
        let earlier = header.differs?.checked_sub(self.first)?;
        self.entries.get(earlier)
    }

    /// The position in the list of the nearest header at or before `position`.
    pub(super) fn header_at(&self, position: usize) -> Option<usize> {
        self.entries
            .get(position)?
            .header?
            .checked_sub(self.first)
            .filter(|header| *header <= position)
    }
}

/// A run of consecutive lines that the same long cells cross, and where its
/// entries lie in the long-entry table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Run {
    pub(super) first: usize,
    pub(super) end: usize,
}

/// One line's cells that cross no other line, and where they lie in the
/// short-entry table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ShortLine {
    pub(super) line: usize,
    pub(super) first: usize,
    pub(super) end: usize,
}

/// Lines `first..last` of an axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LineSpan {
    pub(super) first: usize,
    pub(super) last: usize,
}

/// The extents of the data cells of an axis, sorted by start, with the
/// greatest end so far: whether any data cell crosses a range of lines is a
/// binary search.
#[derive(Debug, Default)]
pub(super) struct DataSpans {
    pub(super) starts: Vec<usize>,
    pub(super) reach: Vec<usize>,
}

impl DataSpans {
    /// Whether some data cell crosses a line of `lines`.
    pub(super) fn crosses<E>(&self, lines: Extent, charge: &mut impl Charge<E>) -> Result<bool, E> {
        let before = leading_by(
            self.starts.len(),
            |index| self.starts.get(index).is_some_and(|s| *s < lines.end()),
            charge,
        )?;
        charge(1)?;
        Ok(before
            .checked_sub(1)
            .and_then(|last| self.reach.get(last))
            .is_some_and(|reach| *reach > lines.start))
    }
}

/// Every row (or every column) of a finished table, for the scans of HTML's
/// assigning of header cells.
///
/// Cells that cross more than one line (long cells) are listed once per run
/// of lines they cross alike, where runs begin wherever a long cell starts or
/// ends, so a tall cell is not recorded once per row. Cells that cross one
/// line only (short cells) are listed with their line, and a line with none
/// has no record. A line's cells are its run's long cells and its short ones.
#[derive(Debug, Default)]
pub(super) struct SlotAxis {
    /// How many lines there are.
    pub(super) lines: usize,
    /// The first line of every run.
    pub(super) breaks: Vec<usize>,
    pub(super) runs: Vec<Run>,
    pub(super) long: Vec<SlotEntry>,
    pub(super) short_lines: Vec<ShortLine>,
    pub(super) short: Vec<SlotEntry>,
    /// Over the short lines, the least start of each one's entries.
    pub(super) short_least: LeastTree,
    /// Over the runs, the least start of the long cells that start or end where
    /// the run begins: a run whose value is below a slot differs, left of that
    /// slot, from the run before it.
    pub(super) run_least: LeastTree,
    pub(super) data: DataSpans,
    /// Lines where two cells share a slot, merged and sorted.
    pub(super) irregular: Vec<Extent>,
    /// The distinct extents of the header cells, sorted: a header's key is its
    /// extent's position here.
    pub(super) keys: Vec<Extent>,
}

impl SlotAxis {
    /// Every backing admitted for this axis' records.
    pub(super) fn backings(&self) -> impl Iterator<Item = ScratchBacking> {
        [
            ScratchBacking::of(&self.breaks),
            ScratchBacking::of(&self.runs),
            ScratchBacking::of(&self.long),
            ScratchBacking::of(&self.short_lines),
            ScratchBacking::of(&self.short),
            ScratchBacking::of(&self.short_least.nodes),
            ScratchBacking::of(&self.run_least.nodes),
            ScratchBacking::of(&self.data.starts),
            ScratchBacking::of(&self.data.reach),
            ScratchBacking::of(&self.irregular),
            ScratchBacking::of(&self.keys),
        ]
        .into_iter()
    }

    /// The key of a header that crosses `lines`.
    pub(super) fn key_of<E>(
        &self,
        lines: Extent,
        charge: &mut impl Charge<E>,
    ) -> Result<Option<usize>, E> {
        let before = leading_by(
            self.keys.len(),
            |index| self.keys.get(index).is_some_and(|key| *key < lines),
            charge,
        )?;
        charge(1)?;
        Ok(self
            .keys
            .get(before)
            .filter(|key| **key == lines)
            .map(|_| before))
    }

    pub(super) fn run(&self, index: usize) -> Option<List<'_>> {
        let run = *self.runs.get(index)?;
        Some(List {
            entries: self.long.get(run.first..run.end)?,
            first: run.first,
        })
    }

    /// The lines a run covers, within `lines`.
    pub(super) fn run_lines(&self, index: usize) -> Extent {
        let start = self.breaks.get(index).copied().unwrap_or(self.lines);
        let end = self.breaks.get(index + 1).copied().unwrap_or(self.lines);
        Extent {
            start,
            length: end.saturating_sub(start),
        }
    }

    /// The runs that hold the lines `lines` (not empty).
    pub(super) fn runs_of<E>(
        &self,
        lines: Extent,
        charge: &mut impl Charge<E>,
    ) -> Result<LineSpan, E> {
        let first = self.run_holding(lines.start, charge)?;
        let last = self.run_holding(lines.end().saturating_sub(1), charge)?;
        Ok(LineSpan {
            first,
            last: last + 1,
        })
    }

    fn run_holding<E>(&self, line: usize, charge: &mut impl Charge<E>) -> Result<usize, E> {
        let begun = leading_by(
            self.breaks.len(),
            |index| self.breaks.get(index).is_some_and(|b| *b <= line),
            charge,
        )?;
        Ok(begun.saturating_sub(1))
    }

    /// Where the short lines of `lines` lie in the short-line table.
    pub(super) fn short_positions<E>(
        &self,
        lines: Extent,
        charge: &mut impl Charge<E>,
    ) -> Result<LineSpan, E> {
        let first = leading_by(
            self.short_lines.len(),
            |index| {
                self.short_lines
                    .get(index)
                    .is_some_and(|l| l.line < lines.start)
            },
            charge,
        )?;
        let last = leading_by(
            self.short_lines.len(),
            |index| {
                self.short_lines
                    .get(index)
                    .is_some_and(|l| l.line < lines.end())
            },
            charge,
        )?;
        Ok(LineSpan { first, last })
    }

    pub(super) fn short_list(&self, position: usize) -> Option<List<'_>> {
        let line = *self.short_lines.get(position)?;
        Some(List {
            entries: self.short.get(line.first..line.end)?,
            first: line.first,
        })
    }

    /// The short list of `line`, if it has one.
    pub(super) fn short_of<E>(
        &self,
        line: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<List<'_>, E> {
        let before = leading_by(
            self.short_lines.len(),
            |index| self.short_lines.get(index).is_some_and(|l| l.line < line),
            charge,
        )?;
        charge(1)?;
        Ok(self
            .short_lines
            .get(before)
            .filter(|l| l.line == line)
            .and_then(|_| self.short_list(before))
            .unwrap_or(List::EMPTY))
    }

    /// Visits, in ascending order, each short line at a position of
    /// `positions` whose least start is below `bound`.
    pub(super) fn visit_left<E>(
        &self,
        positions: LineSpan,
        bound: usize,
        charge: &mut impl Charge<E>,
        visit: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        self.short_least
            .visit_below(positions, bound, charge, visit)
    }

    /// Whether two cells share a slot of `line`.
    pub(super) fn irregular_at<E>(
        &self,
        line: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<bool, E> {
        if self.irregular.is_empty() {
            return Ok(false);
        }
        let begun = leading_by(
            self.irregular.len(),
            |index| self.irregular.get(index).is_some_and(|e| e.start <= line),
            charge,
        )?;
        charge(1)?;
        Ok(begun
            .checked_sub(1)
            .and_then(|last| self.irregular.get(last))
            .is_some_and(|e| line < e.end()))
    }
}

/// A tree over an array of starts that finds the positions in a range whose
/// start is below a bound, padded with the greatest value.
#[derive(Debug, Default)]
pub(super) struct LeastTree {
    pub(super) nodes: Vec<usize>,
}

impl LeastTree {
    /// The number of leaves a tree over `count` positions has.
    pub(super) fn leaves_for(count: usize) -> usize {
        count.max(1).next_power_of_two()
    }

    /// Visits, in ascending order, each position of `positions` whose start
    /// is below `bound`.
    pub(super) fn visit_below<E>(
        &self,
        positions: LineSpan,
        bound: usize,
        charge: &mut impl Charge<E>,
        visit: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        let size = self.nodes.len() / 2;
        self.descend(1, 0, size, positions, bound, charge, &mut |position| {
            visit(position).map(|()| false)
        })
        .map(|_| ())
    }

    /// The first position of `positions` whose start is below `bound`.
    pub(super) fn first_below<E>(
        &self,
        positions: LineSpan,
        bound: usize,
        charge: &mut impl Charge<E>,
    ) -> Result<Option<usize>, E> {
        let size = self.nodes.len() / 2;
        let mut found = None;
        self.descend(1, 0, size, positions, bound, charge, &mut |position| {
            found = Some(position);
            Ok(true)
        })?;
        Ok(found)
    }

    /// Visits matches in order until `visit` says to stop; whether it stopped.
    #[allow(clippy::too_many_arguments)]
    fn descend<E>(
        &self,
        node: usize,
        low: usize,
        high: usize,
        positions: LineSpan,
        bound: usize,
        charge: &mut impl Charge<E>,
        visit: &mut impl FnMut(usize) -> Result<bool, E>,
    ) -> Result<bool, E> {
        charge(1)?;
        if high <= positions.first
            || positions.last <= low
            || low >= high
            || self.nodes.get(node).is_none_or(|least| *least >= bound)
        {
            return Ok(false);
        }
        if high - low == 1 {
            return visit(low);
        }
        let middle = low + (high - low) / 2;
        if self.descend(2 * node, low, middle, positions, bound, charge, visit)? {
            return Ok(true);
        }
        self.descend(2 * node + 1, middle, high, positions, bound, charge, visit)
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
