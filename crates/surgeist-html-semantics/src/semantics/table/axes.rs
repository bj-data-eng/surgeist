//! The row and column indexes of a formed model, and the pass that finds the
//! rows and columns that no cell is anchored in (HTML 877d §4.9.12.1, the last
//! step of forming a table).
//!
//! An axis is built from the finished cells in two parts. Cells that cross
//! more than one line are listed once for each run of lines they cross alike,
//! and cells that cross one line only are listed with their line; each list is
//! sorted by start, with its running reach and the nearest header before each
//! entry. A tree over the short lines finds those with a cell before a given
//! slot, the data cells are kept as sorted extents, and the lines where cells
//! share a slot are merged from the overlaps formation recorded. Every record
//! is admitted from the cells' spans before anything is allocated, and every
//! step is paid for before it is taken.

use super::slots::{Extent, LeastTree, Run, ShortLine, SlotAxis, SlotEntry};
use super::*;

/// How a cell crosses one axis: it spans `count` lines from line `first`, and
/// on each line it covers the positions `start..end`.
struct Crossing {
    first: usize,
    count: usize,
    start: usize,
    end: usize,
}

impl Crossing {
    /// The columns axis has a line for each column and positions are rows; the
    /// rows axis has a line for each row and positions are columns.
    fn of(cell: &HtmlTableCell, vertical: bool) -> Self {
        if vertical {
            Self {
                first: cell.x,
                count: cell.width,
                start: cell.y,
                end: cell.y.saturating_add(cell.height),
            }
        } else {
            Self {
                first: cell.y,
                count: cell.height,
                start: cell.x,
                end: cell.x.saturating_add(cell.width),
            }
        }
    }

    fn after(&self) -> usize {
        self.first.saturating_add(self.count)
    }
}

impl<'a> HtmlSemanticSession<'a> {
    /// Reports the rows, then the columns, in which no cell has its top-left
    /// slot. One pass over the cells marks the anchored lines.
    pub(super) fn table_unanchored(
        &mut self,
        model: &mut HtmlTableModel,
        flags: &mut Vec<bool>,
    ) -> TableResult<()> {
        for vertical in [false, true] {
            let lines = if vertical {
                model.width()
            } else {
                model.height()
            };
            self.table_flags(flags, lines)?;
            for cell in &model.cells {
                self.charge(1)?;
                let line = if vertical { cell.x } else { cell.y };
                if let Some(mark) = flags.get_mut(line) {
                    *mark = true;
                }
            }
            for line in 0..lines {
                self.charge(1)?;
                if !flags.get(line).copied().unwrap_or(true) {
                    let error = if vertical {
                        HtmlTableModelError::UnanchoredColumn { x: line }
                    } else {
                        HtmlTableModelError::UnanchoredRow { y: line }
                    };
                    self.table_push(&mut model.errors, error)?;
                }
            }
        }
        Ok(())
    }

    /// Admits and clears `lines + 1` flags: one per row or column and one past
    /// the last.
    fn table_flags(&mut self, flags: &mut Vec<bool>, lines: usize) -> TableResult<()> {
        let count = self.table_add(lines, 1)?;
        self.charge(count)?;
        flags.clear();
        self.reserve_scratch(flags, count)?;
        flags.resize(count, false);
        Ok(())
    }

    /// Builds the row and column indexes of the finished model.
    pub(super) fn table_axes(
        &mut self,
        model: &HtmlTableModel,
        scratch: &mut Scratch<'_>,
    ) -> TableResult<()> {
        // The cells ordered by column, ties in model order.
        let width = model.width();
        let counted = self.table_add(width, 1)?;
        self.charge(counted)?;
        scratch.column_counts.clear();
        self.reserve_scratch(&mut scratch.column_counts, counted)?;
        scratch.column_counts.resize(counted, 0);
        self.charge(model.cells.len())?;
        scratch.order.clear();
        self.reserve_scratch(&mut scratch.order, model.cells.len())?;
        scratch.order.resize(model.cells.len(), 0);
        for cell in &model.cells {
            self.charge(1)?;
            if let Some(count) = scratch.column_counts.get_mut(cell.x + 1) {
                *count += 1;
            }
        }
        let mut running = 0;
        for count in scratch.column_counts.iter_mut() {
            self.charge(1)?;
            running += *count;
            *count = running;
        }
        // `column_counts[x]` is now the number of cells left of column `x`.
        for (index, cell) in model.cells.iter().enumerate() {
            self.charge(1)?;
            if let Some(slot) = scratch.column_counts.get_mut(cell.x)
                && let Some(place) = scratch.order.get_mut(*slot)
            {
                *place = index;
                *slot += 1;
            }
        }
        self.table_axis(model, true, scratch)?;
        self.table_axis(model, false, scratch)
    }

    /// One axis. Its lists are in ascending order of start: along the columns
    /// the cells of a column come in model order, which is ascending in row;
    /// along the rows the cells of a row come in model order, which is
    /// ascending in column.
    fn table_axis(
        &mut self,
        model: &HtmlTableModel,
        vertical: bool,
        scratch: &mut Scratch<'_>,
    ) -> TableResult<()> {
        let Scratch {
            flags,
            line_counts,
            placed,
            order,
            placement,
            columns,
            rows,
            ..
        } = scratch;
        let overlaps = &placement.overlaps;
        let axis = if vertical { columns } else { rows };
        axis.lines = if vertical {
            model.width()
        } else {
            model.height()
        };
        let cells = model.cells.len();
        // The distinct extents of the header cells.
        let mut headers = 0usize;
        for cell in &model.cells {
            self.charge(1)?;
            if cell.kind == HtmlTableCellKind::Header {
                headers += 1;
            }
        }
        self.reserve_scratch(&mut axis.keys, headers)?;
        for cell in &model.cells {
            self.charge(1)?;
            if cell.kind == HtmlTableCellKind::Header {
                let crossing = Crossing::of(cell, vertical);
                axis.keys.push(Extent {
                    start: crossing.first,
                    length: crossing.count,
                });
            }
        }
        let bits = headers
            .checked_ilog2()
            .and_then(|bits| usize::try_from(bits).ok())
            .unwrap_or(0);
        self.charge(headers.saturating_mul(1 + bits))?;
        axis.keys.sort_unstable();
        axis.keys.dedup();
        // The cells in ascending order of start along their line, and in
        // ascending order of the line they begin at.
        let by_start = |position: usize| {
            if vertical {
                position
            } else {
                order.get(position).copied().unwrap_or(0)
            }
        };
        let by_line = |position: usize| {
            if vertical {
                order.get(position).copied().unwrap_or(0)
            } else {
                position
            }
        };
        // A new run begins wherever a cell that crosses several lines starts
        // or ends.
        self.table_flags(flags, axis.lines)?;
        if let Some(first) = flags.first_mut() {
            *first = true;
        }
        for cell in &model.cells {
            self.charge(1)?;
            let crossing = Crossing::of(cell, vertical);
            if crossing.count < 2 {
                continue;
            }
            for position in [crossing.first, crossing.after()] {
                if let Some(mark) = flags.get_mut(position) {
                    *mark = true;
                }
            }
        }
        let mut runs = 0;
        for position in 0..axis.lines {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                runs += 1;
            }
        }
        self.reserve_scratch(&mut axis.breaks, runs)?;
        for position in 0..axis.lines {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                axis.breaks.push(position);
            }
        }
        // How many long cells cross each run.
        self.charge(runs)?;
        line_counts.clear();
        self.reserve_scratch(line_counts, runs)?;
        line_counts.resize(runs, 0);
        let mut total = 0usize;
        for cell in &model.cells {
            self.charge(1)?;
            let crossing = Crossing::of(cell, vertical);
            if crossing.count < 2 {
                continue;
            }
            let span = axis.runs_of(
                Extent {
                    start: crossing.first,
                    length: crossing.count,
                },
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            total = self.table_add(total, span.last - span.first)?;
            for run in span.first..span.last {
                self.charge(1)?;
                if let Some(count) = line_counts.get_mut(run) {
                    *count += 1;
                }
            }
        }
        placed.clear();
        self.reserve_scratch(placed, total)?;
        self.reserve_scratch(&mut axis.long, total)?;
        self.reserve_scratch(&mut axis.runs, runs)?;
        self.charge(total)?;
        placed.resize(total, 0);
        let mut running = 0;
        for count in line_counts.iter_mut() {
            self.charge(1)?;
            running += *count;
            *count = running;
        }
        // Place every long cell at the back of each run it crosses, last cell
        // first, so that each run ends up in ascending order of start.
        for position in (0..cells).rev() {
            self.charge(1)?;
            let index = by_start(position);
            let Some(cell) = model.cells.get(index) else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            let crossing = Crossing::of(cell, vertical);
            if crossing.count < 2 {
                continue;
            }
            let span = axis.runs_of(
                Extent {
                    start: crossing.first,
                    length: crossing.count,
                },
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            for run in span.first..span.last {
                self.charge(1)?;
                if let Some(count) = line_counts.get_mut(run)
                    && let Some(back) = count.checked_sub(1)
                    && let Some(slot) = placed.get_mut(back)
                {
                    *count = back;
                    *slot = index;
                }
            }
        }
        // Finish the runs: each entry gets the running reach and the nearest
        // header at or before it.
        for run in 0..runs {
            self.charge(1)?;
            let first = line_counts.get(run).copied().unwrap_or(0);
            let end = line_counts.get(run + 1).copied().unwrap_or(total);
            let mut tail = Tail::default();
            for position in first..end {
                self.charge(1)?;
                let Some(cell) = placed.get(position).and_then(|&c| model.cells.get(c)) else {
                    return Err(self
                        .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        .into());
                };
                let key = self.table_key(axis, cell, vertical)?;
                let entry = tail.entry(cell, placed[position], position, key, vertical);
                axis.long.push(entry);
            }
            axis.runs.push(Run { first, end });
        }
        // The cells that cross one line only, each line's in ascending order
        // of start.
        let (mut entries, mut lines, mut previous) = (0usize, 0usize, None);
        for position in 0..cells {
            self.charge(1)?;
            let Some(cell) = model.cells.get(by_line(position)) else {
                continue;
            };
            let crossing = Crossing::of(cell, vertical);
            if crossing.count != 1 {
                continue;
            }
            entries += 1;
            if previous != Some(crossing.first) {
                lines += 1;
                previous = Some(crossing.first);
            }
        }
        self.reserve_scratch(&mut axis.short, entries)?;
        self.reserve_scratch(&mut axis.short_lines, lines)?;
        let mut open: Option<(usize, usize)> = None;
        let mut tail = Tail::default();
        for position in 0..cells {
            self.charge(1)?;
            let index = by_line(position);
            let Some(cell) = model.cells.get(index) else {
                continue;
            };
            let crossing = Crossing::of(cell, vertical);
            if crossing.count != 1 {
                continue;
            }
            if open.is_none_or(|(line, _)| line != crossing.first) {
                if let Some((line, first)) = open {
                    axis.short_lines.push(ShortLine {
                        line,
                        first,
                        end: axis.short.len(),
                    });
                }
                open = Some((crossing.first, axis.short.len()));
                tail = Tail::default();
            }
            let at = axis.short.len();
            let key = self.table_key(axis, cell, vertical)?;
            let entry = tail.entry(cell, index, at, key, vertical);
            axis.short.push(entry);
        }
        if let Some((line, first)) = open {
            axis.short_lines.push(ShortLine {
                line,
                first,
                end: axis.short.len(),
            });
        }
        // The trees of least starts over the short lines, and over the runs.
        let size = LeastTree::leaves_for(axis.short_lines.len());
        let nodes = self.table_add(size, size)?;
        self.charge(nodes)?;
        axis.short_least.nodes.clear();
        self.reserve_scratch(&mut axis.short_least.nodes, nodes)?;
        axis.short_least.nodes.resize(nodes, usize::MAX);
        for (position, line) in axis.short_lines.iter().enumerate() {
            self.charge(1)?;
            let least = axis.short.get(line.first).map_or(usize::MAX, |e| e.start);
            if let Some(leaf) = axis.short_least.nodes.get_mut(size + position) {
                *leaf = least;
            }
        }
        self.table_least_internal(&mut axis.short_least, size)?;
        let size = LeastTree::leaves_for(axis.runs.len());
        let nodes = self.table_add(size, size)?;
        self.charge(nodes)?;
        axis.run_least.nodes.clear();
        self.reserve_scratch(&mut axis.run_least.nodes, nodes)?;
        axis.run_least.nodes.resize(nodes, usize::MAX);
        for cell in &model.cells {
            self.charge(1)?;
            let crossing = Crossing::of(cell, vertical);
            if crossing.count < 2 {
                continue;
            }
            for line in [crossing.first, crossing.after()] {
                if line >= axis.lines {
                    continue;
                }
                let span = axis.runs_of(
                    Extent {
                        start: line,
                        length: 1,
                    },
                    &mut |work| self.charge(work).map_err(TableStop::from),
                )?;
                if let Some(leaf) = axis.run_least.nodes.get_mut(size + span.first) {
                    *leaf = (*leaf).min(crossing.start);
                }
            }
        }
        self.table_least_internal(&mut axis.run_least, size)?;
        // The data cells' extents in ascending order of the line they begin
        // at, with the greatest end so far.
        let mut data = 0usize;
        for cell in &model.cells {
            self.charge(1)?;
            if cell.kind == HtmlTableCellKind::Data {
                data += 1;
            }
        }
        self.reserve_scratch(&mut axis.data.starts, data)?;
        self.reserve_scratch(&mut axis.data.reach, data)?;
        let mut reach = 0;
        for position in 0..cells {
            self.charge(1)?;
            let Some(cell) = model.cells.get(by_line(position)) else {
                continue;
            };
            if cell.kind != HtmlTableCellKind::Data {
                continue;
            }
            let crossing = Crossing::of(cell, vertical);
            reach = reach.max(crossing.after());
            axis.data.starts.push(crossing.first);
            axis.data.reach.push(reach);
        }
        // The lines where two cells share a slot: where the pair's rectangles
        // meet, merged.
        for pair in overlaps {
            self.charge(1)?;
            let (Some(cell), Some(other)) =
                (model.cells.get(pair.cell), model.cells.get(pair.other))
            else {
                continue;
            };
            let (a, b) = (Crossing::of(cell, vertical), Crossing::of(other, vertical));
            let start = a.first.max(b.first);
            let end = a.after().min(b.after());
            if start < end {
                let extent = Extent {
                    start,
                    length: end - start,
                };
                self.table_scratch(&mut axis.irregular, extent)?;
            }
        }
        let count = axis.irregular.len();
        let bits = count
            .checked_ilog2()
            .and_then(|bits| usize::try_from(bits).ok())
            .unwrap_or(0);
        self.charge(count.saturating_mul(1 + bits))?;
        axis.irregular.sort_unstable_by_key(|e| e.start);
        let mut kept = 0;
        for position in 0..count {
            self.charge(1)?;
            let next = axis.irregular[position];
            if kept > 0 && next.start <= axis.irregular[kept - 1].end() {
                let last = &mut axis.irregular[kept - 1];
                last.length = last.end().max(next.end()) - last.start;
            } else {
                axis.irregular[kept] = next;
                kept += 1;
            }
        }
        axis.irregular.truncate(kept);
        Ok(())
    }
}

impl<'a> HtmlSemanticSession<'a> {
    /// Fills the inner nodes of a tree of `size` leaves from its leaves.
    fn table_least_internal(&mut self, tree: &mut LeastTree, size: usize) -> TableResult<()> {
        for node in (1..size).rev() {
            self.charge(1)?;
            let least = tree
                .nodes
                .get(2 * node)
                .zip(tree.nodes.get(2 * node + 1))
                .map_or(usize::MAX, |(a, b)| *a.min(b));
            if let Some(slot) = tree.nodes.get_mut(node) {
                *slot = least;
            }
        }
        Ok(())
    }

    /// The key of `cell` on an axis whose keys are built: its extent's place
    /// among the header extents, for a header.
    pub(super) fn table_key(
        &mut self,
        axis: &SlotAxis,
        cell: &HtmlTableCell,
        vertical: bool,
    ) -> TableResult<Option<usize>> {
        if cell.kind != HtmlTableCellKind::Header {
            return Ok(None);
        }
        let crossing = Crossing::of(cell, vertical);
        let lines = Extent {
            start: crossing.first,
            length: crossing.count,
        };
        let key = axis.key_of(lines, &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        match key {
            Some(key) => Ok(Some(key)),
            None => Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into()),
        }
    }
}

/// What a line's entries carry from the entries before them.
#[derive(Default)]
struct Tail {
    reach: usize,
    header: Option<LastHeader>,
}

/// The nearest header so far.
#[derive(Clone, Copy)]
struct LastHeader {
    at: usize,
    key: usize,
    differs: Option<usize>,
}

impl Tail {
    /// The entry of `cell` at table index `at`.
    fn entry(
        &mut self,
        cell: &HtmlTableCell,
        index: usize,
        at: usize,
        key: Option<usize>,
        vertical: bool,
    ) -> SlotEntry {
        let crossing = Crossing::of(cell, vertical);
        self.reach = self.reach.max(crossing.end);
        let mut differs = None;
        if let Some(key) = key {
            differs = match self.header {
                Some(last) if last.key != key => Some(last.at),
                Some(last) => last.differs,
                None => None,
            };
            self.header = Some(LastHeader { at, key, differs });
        }
        SlotEntry {
            start: crossing.start,
            end: crossing.end,
            cell: index,
            reach: self.reach,
            header: self.header.map(|last| last.at),
            key,
            differs,
        }
    }
}
