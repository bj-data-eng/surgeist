//! The row and column indexes of a formed model, and the pass that finds the
//! rows and columns that no cell is anchored in (HTML 877d §4.9.12.1, the last
//! step of forming a table).
//!
//! An axis is built from the finished cells: the lines where the covering
//! cells change, how many cells cross each line, the cells ordered line by
//! line, and then the finished entries, each with its running reach and the
//! nearest header before it. Every record is admitted from the cells' spans
//! before anything is allocated, and every step is paid for before it is taken.

use super::slots::{SlotEntry, SlotLine};
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

/// The two tables an axis is placed into: where each line's cells begin, then
/// the cells in line order.
struct Tally<'t> {
    line_counts: &'t mut [usize],
    placed: &'t mut [usize],
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

    /// Builds the row and column lines of the finished model: the cells that
    /// cover a run of identical rows as intervals of columns sorted by start,
    /// and for each run of identical columns the intervals of rows.
    pub(super) fn table_axes(
        &mut self,
        model: &HtmlTableModel,
        scratch: &mut Scratch<'_>,
    ) -> TableResult<()> {
        self.table_axis(model, true, scratch)?;
        self.table_axis(model, false, scratch)
    }

    /// One axis. The lines list their cells in ascending order of start:
    /// columns take the cells in model order, which is ascending in row; rows
    /// take them ordered by column.
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
            column_counts,
            columns,
            rows,
            ..
        } = scratch;
        let axis = if vertical { columns } else { rows };
        let extent = if vertical {
            model.width()
        } else {
            model.height()
        };
        // A new line begins wherever a cell starts or ends.
        self.table_flags(flags, extent)?;
        if let Some(first) = flags.first_mut() {
            *first = true;
        }
        for cell in &model.cells {
            self.charge(1)?;
            let crossing = Crossing::of(cell, vertical);
            for position in [crossing.first, crossing.after()] {
                if let Some(mark) = flags.get_mut(position) {
                    *mark = true;
                }
            }
        }
        let mut runs = 0;
        for position in 0..extent {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                runs += 1;
            }
        }
        self.reserve_scratch(&mut axis.breaks, runs)?;
        for position in 0..extent {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                axis.breaks.push(position);
            }
        }
        // How many cells cross each line.
        self.charge(runs)?;
        line_counts.clear();
        self.reserve_scratch(line_counts, runs)?;
        line_counts.resize(runs, 0);
        let mut total = 0usize;
        for cell in &model.cells {
            self.charge(1)?;
            let crossing = Crossing::of(cell, vertical);
            let span = axis.lines_of(crossing.first, crossing.after(), &mut |work| {
                self.charge(work).map_err(TableStop::from)
            })?;
            total = self.table_add(total, span.last - span.first)?;
            for line in span.first..span.last {
                self.charge(1)?;
                if let Some(count) = line_counts.get_mut(line) {
                    *count += 1;
                }
            }
        }
        placed.clear();
        self.reserve_scratch(placed, total)?;
        self.reserve_scratch(&mut axis.entries, total)?;
        self.reserve_scratch(&mut axis.lines, runs)?;
        self.charge(total)?;
        placed.resize(total, 0);
        let mut running = 0;
        for count in line_counts.iter_mut() {
            self.charge(1)?;
            running += *count;
            *count = running;
        }
        // Place every cell at the back of each line it crosses, last cell
        // first, so that each line ends up in ascending order.
        if vertical {
            for index in (0..model.cells.len()).rev() {
                self.table_place(
                    model,
                    index,
                    vertical,
                    axis,
                    &mut Tally {
                        line_counts,
                        placed,
                    },
                )?;
            }
        } else {
            let width = model.width();
            let counted = self.table_add(width, 1)?;
            self.charge(counted)?;
            column_counts.clear();
            self.reserve_scratch(column_counts, counted)?;
            column_counts.resize(counted, 0);
            self.charge(model.cells.len())?;
            order.clear();
            self.reserve_scratch(order, model.cells.len())?;
            order.resize(model.cells.len(), 0);
            for cell in &model.cells {
                self.charge(1)?;
                if let Some(count) = column_counts.get_mut(cell.x + 1) {
                    *count += 1;
                }
            }
            let mut running = 0;
            for count in column_counts.iter_mut() {
                self.charge(1)?;
                running += *count;
                *count = running;
            }
            // `column_counts[x]` is now the number of cells left of column `x`.
            for (index, cell) in model.cells.iter().enumerate() {
                self.charge(1)?;
                if let Some(slot) = column_counts.get_mut(cell.x)
                    && let Some(place) = order.get_mut(*slot)
                {
                    *place = index;
                    *slot += 1;
                }
            }
            for position in (0..order.len()).rev() {
                self.charge(1)?;
                let index = order.get(position).copied().unwrap_or(0);
                self.table_place(
                    model,
                    index,
                    vertical,
                    axis,
                    &mut Tally {
                        line_counts,
                        placed,
                    },
                )?;
            }
        }
        // Finish the lines: the running reach, whether the line's entries are
        // disjoint, the nearest header at or before each entry, and whether
        // any entry is a data cell.
        for line in 0..runs {
            self.charge(1)?;
            let first = line_counts.get(line).copied().unwrap_or(0);
            let end = line_counts.get(line + 1).copied().unwrap_or(total);
            let (mut reach, mut header, mut regular, mut data) = (0, None, true, false);
            for position in first..end {
                self.charge(1)?;
                let Some(cell) = placed.get(position).and_then(|&c| model.cells.get(c)) else {
                    return Err(self
                        .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        .into());
                };
                let crossing = Crossing::of(cell, vertical);
                if position > first && crossing.start < reach {
                    regular = false;
                }
                reach = reach.max(crossing.end);
                if cell.kind == HtmlTableCellKind::Header {
                    header = Some(position);
                } else {
                    data = true;
                }
                axis.entries.push(SlotEntry {
                    start: crossing.start,
                    end: crossing.end,
                    cell: placed[position],
                    reach,
                    header,
                });
            }
            axis.lines.push(SlotLine {
                first,
                end,
                regular,
                data,
            });
        }
        Ok(())
    }

    /// Places cell `index` at the back of each line it crosses.
    fn table_place(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        vertical: bool,
        axis: &SlotAxis,
        tally: &mut Tally<'_>,
    ) -> TableResult<()> {
        let Some(cell) = model.cells.get(index) else {
            return Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into());
        };
        let crossing = Crossing::of(cell, vertical);
        let span = axis.lines_of(crossing.first, crossing.after(), &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        for line in span.first..span.last {
            self.charge(1)?;
            if let Some(count) = tally.line_counts.get_mut(line)
                && let Some(back) = count.checked_sub(1)
                && let Some(slot) = tally.placed.get_mut(back)
            {
                *count = back;
                *slot = index;
            }
        }
        Ok(())
    }
}
