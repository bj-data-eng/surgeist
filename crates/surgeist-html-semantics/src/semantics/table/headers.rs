//! Assigning header cells (HTML 877d §4.9.12.2): the explicit `headers`
//! attribute, and the scans left and up from each cell through the row and
//! column indexes.
//!
//! A scan over a line of disjoint intervals visits each cell once and jumps
//! over a run of data cells, which only ends the current header block however
//! long it is. A line where cells overlap is scanned one slot at a time, the
//! slots with no occupant or several being skipped as the algorithm says.
//!
//! # Sharing the scans of the lines a cell crosses
//!
//! A cell scans each line it crosses (HTML 877d §4.9.12.2, assigning header
//! cells, steps 3.3 and 3.4) and keeps the first of each header in the order
//! the lines are scanned (step 5). The lines of a run cross the same *long*
//! cells (cells that cross several lines) and each has its own *short* cells
//! (cells of that line only). In a line where no two cells share a slot, the
//! internal algorithm for scanning and assigning header cells reads the
//! line's cells from the scanning cell toward the start of the line as one
//! stream: a header joins the current block and is taken unless it is not a
//! header of the scan's kind or is hidden (step 9, substeps 1 to 5); a data
//! cell ends the block, which makes its headers *opaque* (step 9, the data
//! cell case); and a header is hidden when an opaque header has the same
//! extent across the other axis (substep 4). The scanning cell, when it is a
//! header, is in the first block (step 4). So, for a scanning cell that
//! crosses several lines:
//!
//! 1. A long header is hidden exactly when a header of the same extent, long or
//!    the scanning cell, lies nearer the scanning cell with a data cell, long
//!    or short, between the two. A short header is hidden exactly when a
//!    short header of its line lies nearer with a data cell between the two,
//!    since no long header and not the scanning cell has the extent of one
//!    line.
//! 2. Hiding is monotone: more data cells in the stream hide at least the
//!    headers fewer hid.
//! 3. A short header decides nothing about a long header: it joins blocks, and
//!    no long header has its extent.
//!
//! The headers a line takes are therefore the long headers that no data cell
//! of the long cells and no short data cell of the line hides, and the short
//! headers that no data cell of the long cells and no short header of the line
//! hides, in the order of the stream.
//!
//! For one extent (a *key*) let `c_1 > c_2 > ...` be the starts of its long
//! headers below the scanning cell, `c_1` the first (the scanning cell itself,
//! if it is a header of that extent), and let a header be a *candidate* when no
//! data cell of the long cells lies between it and `c_1`. By (1), a regular
//! line takes a candidate `c_j` (that is a header of the scan's kind) exactly
//! when it has no short data cell starting between `c_j` and `c_1`; call it
//! *reaching* `c_j`. So a line takes a prefix of the candidates of an extent:
//! those nearer than its nearest short data cell below `c_1`. A candidate that
//! the line does not take is *hidden* by that data cell.
//!
//! Scanning the lines of a segment in order, then:
//!
//! - The *shared scan*, of the long cells alone, stands in for the first line
//!   with no short cell before the scanning cell. By (2) it takes every
//!   candidate any line could, and later lines add no long header to it.
//! - A regular line with cells of its own is scanned in full unless it meets
//!   the *requirements*. A scan in full notes, for each extent, the start `N`
//!   of the first candidate it reaches hidden (the nearest the line did not
//!   take) and `C`, the start of the first header of the extent that it
//!   reaches, when the data cell that last made the extent opaque was a short
//!   one. If the first hidden candidate was hidden by a data cell of the long
//!   cells, no candidate beyond it is a candidate, and nothing is noted. The
//!   scan reaches the first header of every extent, and a header it skips
//!   after a hidden one is hidden by the same data cell, so `N` is at or
//!   after every candidate of the extent the line did not take.
//! - A requirement `(N, C)` is met by a line that has a short data cell
//!   starting between `N` and `C`. *Lemma A*: if a line meets the requirement
//!   of every extent that has one, the line takes no candidate that is not
//!   taken. For an untaken candidate `c` of an extent lies at or beyond `N` (by
//!   the invariant below), so the data cell between `N` and `C` is between `c`
//!   and `C`, and by (1) it hides `c`. Only its own headers are then new, and
//!   by (1) and (3) they are the short header nearest the scanning cell and
//!   the short headers next to it in the line, up to the first data cell
//!   between two, long or short, each taken when it is a header of the scan's
//!   kind (`table_scan_blocks`).
//! - *Invariant*: for each extent that has an untaken candidate there is a
//!   requirement whose `N` is at or after every untaken candidate. It holds
//!   after the first line scanned in full, by what the scan notes. *Lemma B*:
//!   it holds after any line scanned in full, if each requirement is replaced
//!   by `(min(N, N'), C)` when the line noted `(N', C)` for the extent, and
//!   dropped when the line noted none. A candidate is untaken after the line
//!   only if it was untaken before and the line hid it; the first is at or
//!   after `N` and the second at or after `N'`, so it lies at or beyond
//!   `min(N, N')`. If the line noted nothing for the extent, it took every
//!   candidate (or none is one), so none is untaken. An extent with no
//!   requirement has no untaken candidate, and none is added for it.
//! - With no requirement left the shared scan has been done by the lines
//!   scanned in full, and every regular line adds only its own headers.
//! - Any line where cells share a slot is scanned in full and changes nothing.
//!
//! The order is the specification's: the lists are merged keeping the first of
//! each header, a line adds to it the headers it takes that no line before took
//! (the long ones when it was scanned in full, in the stream order) and its own
//! headers in the stream order; a line that meets the requirements takes no
//! long header that is new.

use super::slots::{self, Cover, Extent, LeastTree, LineSpan, List, SlotAxis, SlotEntry};
use super::*;

/// What a scan that notes the data cells hiding tall headers remembers about
/// one header extent (a key). Each field is valid for the keys stamped with
/// the scan's epoch, as the field says.
#[derive(Clone, Copy)]
pub(super) struct KeyNote {
    /// The start of the short data cell that last made headers of the key
    /// opaque, or `usize::MAX` when a data cell of the long cells did.
    closed: usize,
    /// The epoch of the scan that reached a header of the key, and the start of
    /// the first (the nearest to the scanning cell).
    seen: usize,
    first: usize,
    /// The epoch of the scan that reached a header of the key hidden, which
    /// noted it.
    noted: usize,
}

impl KeyNote {
    const NONE: Self = Self {
        closed: usize::MAX,
        seen: 0,
        first: 0,
        noted: 0,
    };
}

/// A header a scan reaches: its cell, its key, where it starts, and whether
/// it is one of the long cells.
#[derive(Clone, Copy)]
struct Reached {
    cell: usize,
    key: usize,
    start: usize,
    long: bool,
}

/// A requirement for a line to share the scan of the long cells: the nearest
/// header of the key that is still unlisted (`least`) is hidden for a line
/// that has a short data cell starting between it and the first header of the
/// key (`right`).
#[derive(Clone, Copy)]
pub(super) struct Need {
    key: usize,
    least: usize,
    right: usize,
}

/// What the header scans remember about one cell. A cell's attributes do not
/// change during a query, so each is resolved the first time a header
/// assignment consults it and reused by every assignment that reaches the cell.
#[derive(Clone, Copy, Default)]
pub(super) struct HeaderMemo {
    /// The th scope state.
    scope: Option<HtmlTableHeaderScopeState>,
    /// Whether no data cell shares a row with the header (scope auto).
    column: Option<bool>,
    /// Whether no data cell shares a column with the header (scope auto).
    row: Option<bool>,
    /// Whether the cell is empty: it holds no element and only ASCII
    /// whitespace as text.
    empty: Option<bool>,
}

/// An element with a non-empty `id`: its value, the capture ordinal of its
/// subject, and its place among the nodes of a walk in tree order.
struct IdEntry<'a> {
    value: &'a [u16],
    ordinal: usize,
    order: usize,
}

/// What ends a search for an ID before any element has it.
enum IdStop {
    /// A node whose ordered children are unknown.
    Missing(HtmlSemanticMissing),
    /// An element whose ID the interpreter refuses.
    Refused(HtmlValueAdmissionError),
}

/// A cell of the model: the capture ordinal of its subject and its index.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct CellSlot {
    ordinal: usize,
    cell: usize,
}

/// What resolves a `headers` token: the IDs of a document, sorted by value
/// with ties in tree order, and what ends a search for an absent ID; and the
/// cells of the model by capture ordinal. Built whole on the first token.
#[derive(Default)]
pub(super) struct IdTargets<'a> {
    entries: Vec<IdEntry<'a>>,
    stop: Option<IdStop>,
    cells: Vec<CellSlot>,
}

impl IdTargets<'_> {
    /// The backing admitted for the entries and the cells.
    pub(super) fn backings(&self) -> [ScratchBacking; 2] {
        [
            ScratchBacking::of(&self.entries),
            ScratchBacking::of(&self.cells),
        ]
    }
}

/// The bit length of `n`: the depth of a search or sort over `n` records.
fn table_log2(n: usize) -> usize {
    (usize::BITS - n.leading_zeros()) as usize
}

/// The finished row and column lines.
#[derive(Clone, Copy)]
struct Axes<'s> {
    rows: &'s SlotAxis,
    columns: &'s SlotAxis,
}

/// The scratch a header scan works in: the headers found for the principal
/// cell, the block of the scan in progress and the extents hidden so far, what
/// is known of each header, and the finished lines.
struct Scan<'s> {
    headers: &'s mut Vec<usize>,
    /// A cell marked with `mark` is already in `headers`.
    listed: &'s mut Vec<usize>,
    mark: usize,
    /// A header whose key is stamped with `epoch` is hidden: an opaque header
    /// of this scan crosses the same lines.
    stamps: &'s mut Vec<usize>,
    epoch: usize,
    block: &'s mut Vec<usize>,
    memo: &'s mut Vec<HeaderMemo>,
    /// The lines of a scan that must be scanned one at a time.
    specials: &'s mut Vec<usize>,
    /// Per header key, what a scan that is noting remembers.
    closed: &'s mut Vec<KeyNote>,
    /// The requirements a line must meet to share the scan of the long cells,
    /// at most one for each key, in ascending order of key.
    window: &'s mut Vec<Need>,
    /// The requirements the scan in progress notes.
    notes: &'s mut Vec<Need>,
    /// Whether the scan in progress notes requirements.
    noting: bool,
    axes: Axes<'s>,
    in_block: bool,
    /// Some cell of the block crosses lines that no opaque header does.
    live: bool,
}

/// The first row and column beyond the principal cell.
#[derive(Clone, Copy)]
struct Far {
    right: usize,
    bottom: usize,
}

/// What a scan of the lines of one axis has in common: the axis, the key of the
/// principal cell if it is a header, where the scan starts and its direction.
#[derive(Clone, Copy)]
struct Target<'a> {
    axis: &'a SlotAxis,
    principal_key: Option<usize>,
    from: usize,
    vertical: bool,
}

/// The cells of one line: the long cells of its run and its short cells.
#[derive(Clone, Copy)]
struct Occupants<'a> {
    long: List<'a>,
    short: List<'a>,
}

/// The header cells whose `scope` names a group, by kind.
#[derive(Clone, Copy)]
struct GroupHeaders<'s> {
    row_group: &'s [usize],
    row_group_least: &'s LeastTree,
    column_group: &'s [usize],
    column_group_least: &'s LeastTree,
}

impl<'a> HtmlSemanticSession<'a> {
    /// Admits the memo table: one entry per cell, on first use.
    fn table_memo_ready(
        &mut self,
        model: &HtmlTableModel,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<()> {
        if memo.len() != model.cells.len() {
            self.charge(model.cells.len())?;
            self.reserve_scratch(memo, model.cells.len())?;
            memo.resize(model.cells.len(), HeaderMemo::default());
        }
        Ok(())
    }
    /// Whether cell `index` is empty, read once per cell.
    fn table_empty_once(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<bool> {
        self.table_memo_ready(model, memo)?;
        self.charge(1)?;
        if let Some(known) = memo.get(index).and_then(|m| m.empty) {
            return Ok(known);
        }
        let empty = self.table_empty(model, index)?;
        if let Some(entry) = memo.get_mut(index) {
            entry.empty = Some(empty);
        }
        Ok(empty)
    }
    /// The th scope state of cell `index`, resolved through the HTML-owned rule
    /// the first time this query consults it and reused afterwards. The first
    /// consultation admits the memo table (one entry per cell) and the rule's
    /// published bound for the raw value; later ones charge a single lookup.
    fn table_scope(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<HtmlTableHeaderScopeState> {
        self.table_memo_ready(model, memo)?;
        self.charge(1)?;
        if let Some(state) = memo.get(index).and_then(|m| m.scope) {
            return Ok(state);
        }
        let element = self.table_element(&model.cells[index].subject)?;
        let raw = self.table_raw(element, "scope")?;
        let value = raw.map(Attribute::value);
        self.table_charge_bound(th_scope_work_bound(value.map_or(0, DocumentString::len)))?;
        let state = th_scope_state(value);
        self.poll()?;
        if let Some(entry) = memo.get_mut(index) {
            entry.scope = Some(state);
        }
        Ok(state)
    }
    /// Whether header `index` is a column header: its `scope` says column, or
    /// it is auto and no data cell shares a row with it. The auto answer is
    /// read from the row lines once per header.
    fn table_column_header(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        rows: &SlotAxis,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<bool> {
        match self.table_scope(model, index, memo)? {
            HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Column) => Ok(true),
            HtmlTableHeaderScopeState::Auto => {
                self.charge(1)?;
                if let Some(known) = memo.get(index).and_then(|m| m.column) {
                    return Ok(known);
                }
                let header = &model.cells[index];
                let lines = Extent {
                    start: header.y,
                    length: header.height,
                };
                let shared = rows.data.crosses(lines, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                if let Some(entry) = memo.get_mut(index) {
                    entry.column = Some(!shared);
                }
                Ok(!shared)
            }
            _ => Ok(false),
        }
    }
    /// Whether header `index` is a row header: its `scope` says row, or it is
    /// auto, not a column header, and no data cell shares a column with it.
    fn table_row_header(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        axes: Axes<'_>,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<bool> {
        match self.table_scope(model, index, memo)? {
            HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Row) => Ok(true),
            HtmlTableHeaderScopeState::Auto
                if !self.table_column_header(model, index, axes.rows, memo)? =>
            {
                self.charge(1)?;
                if let Some(known) = memo.get(index).and_then(|m| m.row) {
                    return Ok(known);
                }
                let header = &model.cells[index];
                let lines = Extent {
                    start: header.x,
                    length: header.width,
                };
                let shared = axes.columns.data.crosses(lines, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                if let Some(entry) = memo.get_mut(index) {
                    entry.row = Some(!shared);
                }
                Ok(!shared)
            }
            _ => Ok(false),
        }
    }
    /// Adds `index` to the principal cell's header list unless it is there.
    fn table_list(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        index: usize,
    ) -> TableResult<()> {
        if scan.listed.len() != model.cells.len() {
            self.charge(model.cells.len())?;
            self.reserve_scratch(scan.listed, model.cells.len())?;
            scan.listed.resize(model.cells.len(), 0);
        }
        self.charge(1)?;
        if scan.listed.get(index) == Some(&scan.mark) {
            return Ok(());
        }
        if let Some(mark) = scan.listed.get_mut(index) {
            *mark = scan.mark;
        }
        self.table_scratch(scan.headers, index)
    }
    /// Collects into `out` the positions of `positions` whose start in the tree
    /// is below `bound`, in ascending order.
    fn table_collect_below(
        &mut self,
        tree: &LeastTree,
        positions: LineSpan,
        bound: usize,
        out: &mut Vec<usize>,
    ) -> TableResult<()> {
        out.clear();
        let mut matches = 0usize;
        tree.visit_below(
            positions,
            bound,
            &mut |work| self.charge(work).map_err(TableStop::from),
            &mut |_| {
                matches += 1;
                Ok(())
            },
        )?;
        self.reserve_scratch(out, matches)?;
        tree.visit_below(
            positions,
            bound,
            &mut |work| self.charge(work).map_err(TableStop::from),
            &mut |position| {
                out.push(position);
                Ok(())
            },
        )
    }
    /// Builds the tree over a group's header cells from a leaf for each.
    fn table_group_tree(
        &mut self,
        model: &HtmlTableModel,
        cells: &[usize],
        tree: &mut LeastTree,
        leaf: impl Fn(&HtmlTableCell) -> usize,
    ) -> TableResult<()> {
        let size = LeastTree::leaves_for(cells.len());
        let nodes = self.table_add(size, size)?;
        self.charge(nodes)?;
        tree.nodes.clear();
        self.reserve_scratch(&mut tree.nodes, nodes)?;
        tree.nodes.resize(nodes, usize::MAX);
        for (position, index) in cells.iter().enumerate() {
            self.charge(1)?;
            if let (Some(cell), Some(slot)) =
                (model.cells.get(*index), tree.nodes.get_mut(size + position))
            {
                *slot = leaf(cell);
            }
        }
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
    /// A data cell reached by a scan ends the header block it follows: the
    /// block's extents become opaque to earlier headers that cross the same
    /// lines. A scan that notes the window also notes, for each of them, the
    /// short data cell `short` that made it opaque.
    fn table_end_block(&mut self, scan: &mut Scan<'_>, short: Option<usize>) -> TableResult<()> {
        if !scan.in_block {
            return Ok(());
        }
        scan.in_block = false;
        scan.live = false;
        for position in 0..scan.block.len() {
            self.charge(1)?;
            if let Some(key) = scan.block.get(position)
                && let Some(stamp) = scan.stamps.get_mut(*key)
            {
                *stamp = scan.epoch;
                if scan.noting {
                    let Some(note) = scan.closed.get_mut(*key) else {
                        return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated).into());
                    };
                    note.closed = short.unwrap_or(usize::MAX);
                }
            }
        }
        self.charge(1)?;
        scan.block.clear();
        Ok(())
    }
    /// A header cell reached by a scan joins the current block, and is a
    /// header of the principal cell unless an opaque header that crosses the
    /// same lines hides it or it is not a header of the scan's kind. A scan
    /// that notes requirements notes, for the first header of the long cells
    /// of an extent that it reaches hidden, the short data cell that made the
    /// extent opaque, if one did, with `start` the start of the header.
    fn table_reach_header(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        reached: Reached,
        vertical: bool,
    ) -> TableResult<()> {
        let Reached {
            cell: index,
            key,
            start,
            long,
        } = reached;
        scan.in_block = true;
        self.table_scratch(scan.block, key)?;
        self.charge(1)?;
        let mut blocked = scan.stamps.get(key) == Some(&scan.epoch);
        if scan.noting && long {
            let Some(note) = scan.closed.get_mut(key) else {
                return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated).into());
            };
            if note.seen != scan.epoch {
                note.seen = scan.epoch;
                note.first = start;
            }
            if blocked && note.noted != scan.epoch {
                note.noted = scan.epoch;
                let need = Need {
                    key,
                    least: start,
                    right: note.first,
                };
                if note.closed != usize::MAX {
                    self.table_scratch(scan.notes, need)?;
                }
            }
        }
        if !blocked {
            scan.live = true;
            blocked = if vertical {
                !self.table_column_header(model, index, scan.axes.rows, scan.memo)?
            } else {
                !self.table_row_header(model, index, scan.axes, scan.memo)?
            };
        }
        if !blocked {
            self.table_list(model, scan, index)?;
        }
        Ok(())
    }
    /// Scans from the principal cell toward the start of each row it crosses
    /// (leftward) or of each column it crosses (upward), taking the occupant of
    /// each slot in turn.
    ///
    /// The lines of one run hold the same long cells, so the scans of those
    /// lines differ only by their short cells and by overlaps. The lines with
    /// no short cell before the principal cell, however many, are scanned once;
    /// a line where two cells share a slot is scanned on its own; and a line
    /// with short cells shares the scans of the long cells of the lines
    /// scanned in full before it when it meets their requirements (see the
    /// module documentation), and is otherwise scanned on its own.
    fn table_scan_axis(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        principal: usize,
        vertical: bool,
    ) -> TableResult<()> {
        let cell = &model.cells[principal];
        let (axis, lines, from) = if vertical {
            (
                scan.axes.columns,
                Extent {
                    start: cell.x,
                    length: cell.width,
                },
                cell.y,
            )
        } else {
            (
                scan.axes.rows,
                Extent {
                    start: cell.y,
                    length: cell.height,
                },
                cell.x,
            )
        };
        let principal_key = self.table_key(axis, cell, vertical)?;
        let target = Target {
            axis,
            principal_key,
            from,
            vertical,
        };
        let runs = axis.runs_of(lines, &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        // The runs of a segment differ from each other only by cells that
        // start at or after the scanning cell, which a scan does not reach,
        // so the first run's long cells stand for all of them.
        let mut run = runs.first;
        while run < runs.last {
            self.charge(1)?;
            let next = axis.run_least.first_below(
                LineSpan {
                    first: run + 1,
                    last: runs.last,
                },
                from,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let following = next.unwrap_or(runs.last);
            let Some(long) = axis.run(run) else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            let start = axis.run_lines(run).start.max(lines.start);
            let end = if next.is_some() {
                axis.run_lines(following).start.min(lines.end())
            } else {
                lines.end()
            };
            run = following;
            let here = Extent {
                start,
                length: end.saturating_sub(start),
            };
            if here.length == 1 {
                let short = axis.short_of(start, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let irregular = axis.irregular_at(start, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let cells = Occupants { long, short };
                self.table_scan_line(model, scan, principal, target, cells, irregular)?;
                continue;
            }
            // The lines that cannot share the scan of the run's long cells.
            scan.specials.clear();
            let positions =
                axis.short_positions(here, &mut |work| self.charge(work).map_err(TableStop::from))?;
            let mut matches = 0usize;
            axis.visit_left(
                positions,
                from,
                &mut |work| self.charge(work).map_err(TableStop::from),
                &mut |_| {
                    matches += 1;
                    Ok(())
                },
            )?;
            self.reserve_scratch(scan.specials, matches)?;
            let specials = &mut *scan.specials;
            axis.visit_left(
                positions,
                from,
                &mut |work| self.charge(work).map_err(TableStop::from),
                &mut |position| {
                    if let Some(line) = axis.short_lines.get(position) {
                        specials.push(line.line);
                    }
                    Ok(())
                },
            )?;
            let begun = slots::leading_by(
                axis.irregular.len(),
                |index| {
                    axis.irregular
                        .get(index)
                        .is_some_and(|extent| extent.end() <= here.start)
                },
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            for extent in axis.irregular.get(begun..).unwrap_or_default() {
                self.charge(1)?;
                if extent.start >= here.end() {
                    break;
                }
                for line in extent.start.max(here.start)..extent.end().min(here.end()) {
                    self.table_scratch(scan.specials, line)?;
                }
            }
            let count = scan.specials.len();
            let bits = count
                .checked_ilog2()
                .and_then(|bits| usize::try_from(bits).ok())
                .unwrap_or(0);
            self.charge(count.saturating_mul(1 + bits))?;
            scan.specials.sort_unstable();
            scan.specials.dedup();
            // The lines are taken top to bottom (or left to right), so that
            // headers are found in the order the specification lists them.
            // `base` says that every header of the long cells that any line
            // lists has been listed: by the shared scan, which stands in for
            // the first line that has no cell of its own before the scanning
            // cell, or by regular lines scanned in full that left no
            // requirement. Then a regular line adds only the headers of its
            // own cells (`table_scan_blocks`). Otherwise `window` holds the
            // requirements of the long headers not yet listed, and a regular
            // line that meets them all lists no header of the long cells that
            // is not listed (see the module documentation); any other regular
            // line is scanned in full and its requirements refresh them.
            let mut shared = Some(here.start);
            let mut base = false;
            let mut kept = false;
            for position in 0..scan.specials.len() {
                self.charge(1)?;
                let line = scan.specials[position];
                if !base
                    && let Some(first) = shared
                    && first < line
                {
                    self.table_scan_shared(model, scan, principal, target, long)?;
                    base = true;
                }
                if shared == Some(line) {
                    shared = Some(line + 1);
                }
                let short =
                    axis.short_of(line, &mut |work| self.charge(work).map_err(TableStop::from))?;
                let irregular = axis
                    .irregular_at(line, &mut |work| self.charge(work).map_err(TableStop::from))?;
                let cells = Occupants { long, short };
                if irregular {
                    self.table_scan_line(model, scan, principal, target, cells, true)?;
                    continue;
                }
                if base || (kept && self.table_meets_needs(short, scan.window)?) {
                    self.table_scan_blocks(model, scan, target, cells)?;
                    continue;
                }
                // A line to be scanned in full: note which short data cells of
                // it made a header of the long cells hide another.
                self.table_closed_ready(scan)?;
                scan.notes.clear();
                scan.noting = true;
                let scanned = self.table_scan_line(model, scan, principal, target, cells, false);
                scan.noting = false;
                scanned?;
                self.table_refresh_needs(scan, kept)?;
                if scan.window.is_empty() {
                    base = true;
                } else {
                    kept = true;
                }
            }
            if !base
                && let Some(first) = shared
                && first < here.end()
            {
                self.table_scan_shared(model, scan, principal, target, long)?;
            }
        }
        Ok(())
    }

    /// The scan the lines without a cell of their own before the scanning cell
    /// share: the run's long cells alone.
    fn table_scan_shared(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        principal: usize,
        target: Target<'_>,
        long: List<'_>,
    ) -> TableResult<()> {
        let cells = Occupants {
            long,
            short: List::EMPTY,
        };
        self.table_scan_line(model, scan, principal, target, cells, false)
    }

    /// Admits the per-key notes of a scan that notes, on first use.
    fn table_closed_ready(&mut self, scan: &mut Scan<'_>) -> TableResult<()> {
        if scan.closed.len() != scan.stamps.len() {
            self.charge(scan.stamps.len())?;
            self.reserve_scratch(scan.closed, scan.stamps.len())?;
            scan.closed.resize(scan.stamps.len(), KeyNote::NONE);
        }
        Ok(())
    }

    /// Replaces the requirements of the lines that share a scan, once a line
    /// has been scanned in full and noted its own. `refresh` says that there
    /// are requirements already, which a key keeps only if the line noted it
    /// too, with the nearer of the two unlisted headers: a key the line noted
    /// no requirement for has all its headers listed by the line, and one the
    /// requirements did not have is already listed.
    fn table_refresh_needs(&mut self, scan: &mut Scan<'_>, refresh: bool) -> TableResult<()> {
        let count = scan.notes.len();
        let bits = count
            .checked_ilog2()
            .and_then(|bits| usize::try_from(bits).ok())
            .unwrap_or(0);
        self.charge(count.saturating_mul(1 + bits))?;
        scan.notes.sort_unstable_by_key(|need| need.key);
        if !refresh {
            core::mem::swap(scan.window, scan.notes);
            return Ok(());
        }
        let mut kept = 0;
        let mut noted = 0;
        for position in 0..scan.window.len() {
            self.charge(1)?;
            let old = scan.window[position];
            while scan.notes.get(noted).is_some_and(|need| need.key < old.key) {
                self.charge(1)?;
                noted += 1;
            }
            let Some(new) = scan.notes.get(noted).filter(|need| need.key == old.key) else {
                continue;
            };
            let merged = Need {
                least: old.least.min(new.least),
                ..old
            };
            scan.window[kept] = merged;
            kept += 1;
        }
        scan.window.truncate(kept);
        Ok(())
    }

    /// Whether `short` has, for each requirement, a data cell starting between
    /// the header and the first header of its key.
    fn table_meets_needs(&mut self, short: List<'_>, needs: &[Need]) -> TableResult<bool> {
        for need in needs {
            self.charge(1)?;
            let before = slots::leading(
                short.entries,
                |entry| entry.start < need.right,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let held = before
                .checked_sub(1)
                .and_then(|last| short.data_at(last))
                .and_then(|data| short.entries.get(data))
                .is_some_and(|entry| entry.start > need.least);
            if !held {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Whether `long` has a data cell starting between `left` and `right`.
    fn table_long_data_between(
        &mut self,
        long: List<'_>,
        left: usize,
        right: usize,
    ) -> TableResult<bool> {
        let before = slots::leading(long.entries, |entry| entry.start < right, &mut |work| {
            self.charge(work).map_err(TableStop::from)
        })?;
        self.charge(1)?;
        Ok(before
            .checked_sub(1)
            .and_then(|last| long.data_at(last))
            .and_then(|data| long.entries.get(data))
            .is_some_and(|entry| entry.start > left))
    }

    /// The headers of a line's own cells, once the headers of the long cells
    /// it crosses have been listed by an earlier scan (or are not listed by
    /// this line).
    ///
    /// A header of a short cell crosses one line, so only a header of that
    /// line's short cells can hide it, and none is hidden by a long header or
    /// the principal cell. The nearest header of the line is therefore taken,
    /// and so is each header after it until a data cell, short or long, lies
    /// between two: that data cell makes the headers before it opaque, and
    /// every header beyond has the same line as one of them.
    fn table_scan_blocks(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        target: Target<'_>,
        cells: Occupants<'_>,
    ) -> TableResult<()> {
        self.charge(1)?;
        let Occupants { long, short } = cells;
        let before = slots::leading(
            short.entries,
            |entry| entry.start < target.from,
            &mut |work| self.charge(work).map_err(TableStop::from),
        )?;
        let Some(mut at) = before.checked_sub(1).and_then(|last| short.header_at(last)) else {
            return Ok(());
        };
        loop {
            self.charge(1)?;
            let Some(header) = short.entries.get(at) else {
                return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated).into());
            };
            let wanted = if target.vertical {
                self.table_column_header(model, header.cell, scan.axes.rows, scan.memo)?
            } else {
                self.table_row_header(model, header.cell, scan.axes, scan.memo)?
            };
            if wanted {
                self.table_list(model, scan, header.cell)?;
            }
            let Some(previous) = at.checked_sub(1) else {
                return Ok(());
            };
            // The next header must follow directly in the short cells ...
            let Some(next) = short.header_at(previous).filter(|next| *next == previous) else {
                return Ok(());
            };
            // ... and no long data cell may lie between the two.
            let Some(left) = short.entries.get(next) else {
                return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated).into());
            };
            if self.table_long_data_between(long, left.start, header.start)? {
                return Ok(());
            }
            at = next;
        }
    }

    /// One scan of one line, whose cells are the two lists. Visiting a cell
    /// once instead of once per slot it covers changes nothing, because a
    /// repeated visit re-adds a header already found (duplicates are dropped
    /// when associations are made), a block member already present, and a data
    /// cell with no block left to end. When cells share slots the line is
    /// taken a slot at a time, a slot with no occupant or several being
    /// skipped; otherwise the line is its two lists merged, the data cells
    /// between two headers being jumped.
    fn table_scan_line(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        principal: usize,
        target: Target<'_>,
        cells: Occupants<'_>,
        irregular: bool,
    ) -> TableResult<()> {
        let Target {
            axis,
            principal_key,
            from,
            vertical,
        } = target;
        self.charge(1)?;
        scan.epoch += 1;
        scan.block.clear();
        scan.in_block = model.cells[principal].kind == HtmlTableCellKind::Header;
        scan.live = scan.in_block;
        if scan.in_block {
            let Some(key) = principal_key else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            self.table_scratch(scan.block, key)?;
            if scan.noting {
                let Some(note) = scan.closed.get_mut(key) else {
                    return Err(self.stop(HtmlSemanticFailureKind::ProtocolViolated).into());
                };
                note.seen = scan.epoch;
                note.first = from;
            }
        }
        if irregular {
            let mut slot = from;
            loop {
                self.charge(1)?;
                if slot == 0 {
                    return Ok(());
                }
                slot -= 1;
                let long = slots::cover(cells.long.entries, slot, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let short = slots::cover(cells.short.entries, slot, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let index = match (long, short) {
                    (Cover::One(index), Cover::Empty) | (Cover::Empty, Cover::One(index)) => index,
                    _ => continue,
                };
                let other = &model.cells[index];
                if other.kind == HtmlTableCellKind::Header {
                    let key = self.table_key(axis, other, vertical)?;
                    let Some(key) = key else {
                        return Err(self
                            .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                            .into());
                    };
                    self.table_reach_header(
                        model,
                        scan,
                        Reached {
                            cell: index,
                            key,
                            start: slot,
                            long: false,
                        },
                        vertical,
                    )?;
                } else {
                    self.table_end_block(scan, None)?;
                }
            }
        }
        let mut bound = from;
        loop {
            self.charge(1)?;
            let before_long = slots::leading(
                cells.long.entries,
                |entry| entry.start < bound,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let before_short = slots::leading(
                cells.short.entries,
                |entry| entry.start < bound,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let last_long = before_long.checked_sub(1);
            let last_short = before_short.checked_sub(1);
            // The nearest entry before the bound, and the nearest header.
            let (nearest, long) = match (
                last_long.and_then(|p| cells.long.entries.get(p)),
                last_short.and_then(|p| cells.short.entries.get(p)),
            ) {
                (None, None) => return Ok(()),
                (Some(entry), None) => (entry, true),
                (None, Some(entry)) => (entry, false),
                (Some(a), Some(b)) => {
                    if a.start > b.start {
                        (a, true)
                    } else {
                        (b, false)
                    }
                }
            };
            let header_start = |list: List<'_>, last: Option<usize>| {
                last.and_then(|position| list.header_at(position))
                    .and_then(|header| list.entries.get(header))
                    .map(|entry: &SlotEntry| entry.start)
            };
            let nearest_header = header_start(cells.long, last_long)
                .into_iter()
                .chain(header_start(cells.short, last_short))
                .max();
            if nearest_header == Some(nearest.start) {
                let Some(key) = nearest.key else {
                    return Err(self
                        .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                        .into());
                };
                self.table_reach_header(
                    model,
                    scan,
                    Reached {
                        cell: nearest.cell,
                        key,
                        start: nearest.start,
                        long,
                    },
                    vertical,
                )?;
                bound = nearest.start;
                if scan.live {
                    continue;
                }
                // Every cell of the block is hidden behind an opaque header
                // that crosses the same lines, so the headers that cross those
                // lines, and the data cells among them, change nothing: the
                // scan resumes at the nearest header that crosses others.
                let before_long = slots::leading(
                    cells.long.entries,
                    |entry| entry.start < bound,
                    &mut |work| self.charge(work).map_err(TableStop::from),
                )?;
                let before_short = slots::leading(
                    cells.short.entries,
                    |entry| entry.start < bound,
                    &mut |work| self.charge(work).map_err(TableStop::from),
                )?;
                let other = cells
                    .long
                    .header_other_than(before_long.checked_sub(1), key)
                    .into_iter()
                    .chain(
                        cells
                            .short
                            .header_other_than(before_short.checked_sub(1), key),
                    )
                    .map(|entry| entry.start)
                    .max();
                let Some(start) = other else {
                    return Ok(());
                };
                bound = start + 1;
                continue;
            }
            // The nearest entry is a data cell: it ends the block, and every
            // entry down to the nearest header is data.
            self.table_end_block(scan, (!long).then_some(nearest.start))?;
            let Some(start) = nearest_header else {
                return Ok(());
            };
            bound = start + 1;
        }
    }
    /// Admits and charges a product of factors as one operation.
    fn table_charge_product(&mut self, factors: &[usize]) -> TableResult<()> {
        let mut work = 1usize;
        for &factor in factors {
            work = work.checked_mul(factor).ok_or_else(|| {
                TableStop::from(self.stop(HtmlSemanticFailureKind::RepresentationExhausted))
            })?;
        }
        self.charge(work)?;
        Ok(())
    }
    /// Indexes the elements of the document that have a non-empty `id`, once,
    /// walking them in tree order as the search for one ID would (HTML 877d
    /// §4.9.12.2, assigning header cells, step 3.2: "the first element in the
    /// Document with an ID equal to the token"). The walk ends at the first
    /// thing a search from the root would stop at, which is recorded and
    /// answers every token that no earlier element owns: an element whose ID
    /// the interpreter refuses, or a node whose ordered children are unknown.
    /// Entries are admitted one at a time as they are pushed and the sort is
    /// charged before it runs.
    fn table_index_ids(
        &mut self,
        document: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
        targets: &mut IdTargets<'a>,
    ) -> TableResult<()> {
        self.charge(1)?;
        frames.clear();
        match self.table_children(document) {
            Ok(children) => self.table_scratch(frames, children)?,
            Err(TableStop::Missing(missing)) => {
                targets.stop = Some(IdStop::Missing(missing));
                return Ok(());
            }
            Err(failure) => return Err(failure),
        }
        let mut order = 0usize;
        let mut longest = 0usize;
        while !frames.is_empty() {
            self.charge(1)?;
            let Some(id) = frames.last_mut().and_then(Iterator::next) else {
                frames.pop();
                continue;
            };
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                let raw = self.attribute(element, "id")?;
                if let Err(error) = self
                    .interpreter
                    .admit_attribute_raw(raw.map(Attribute::value))
                {
                    targets.stop = Some(IdStop::Refused(error));
                    break;
                }
                if let Some(raw) = raw
                    && !raw.value().is_empty()
                {
                    let value = raw.value().code_units();
                    longest = longest.max(value.len());
                    self.table_scratch(
                        &mut targets.entries,
                        IdEntry {
                            value,
                            ordinal: id.ordinal(),
                            order,
                        },
                    )?;
                }
            }
            order += 1;
            match self.table_children(&id) {
                Ok(children) => {
                    if children.len() != 0 {
                        self.table_scratch(frames, children)?;
                    }
                }
                Err(TableStop::Missing(missing)) => {
                    targets.stop = Some(IdStop::Missing(missing));
                    break;
                }
                Err(failure) => return Err(failure),
            }
        }
        frames.clear();
        // Tree order breaks ties, so the first equal entry is the first
        // element; each comparison reads up to the longest ID.
        let count = targets.entries.len();
        self.table_charge_product(&[count, table_log2(count), longest + 1])?;
        targets
            .entries
            .sort_unstable_by(|a, b| a.value.cmp(b.value).then(a.order.cmp(&b.order)));
        Ok(())
    }
    /// The cells of the model by capture ordinal, so a cell is found from its
    /// subject without reading the cells one by one.
    fn table_index_cells(
        &mut self,
        model: &HtmlTableModel,
        cells: &mut Vec<CellSlot>,
    ) -> TableResult<()> {
        for (cell, model_cell) in model.cells.iter().enumerate() {
            let ordinal = model_cell.subject.ordinal();
            self.table_scratch(cells, CellSlot { ordinal, cell })?;
        }
        self.table_charge_product(&[cells.len(), table_log2(cells.len())])?;
        cells.sort_unstable();
        Ok(())
    }
    /// Builds the index of a `headers` token's targets whole. A failure
    /// releases what was admitted for it, so a half-built index is never kept.
    fn table_index_targets(
        &mut self,
        model: &HtmlTableModel,
        document: &HtmlCaptureSubjectId,
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> TableResult<IdTargets<'a>> {
        let mut targets = IdTargets::default();
        let result = self
            .table_index_ids(document, frames, &mut targets)
            .and_then(|()| self.table_index_cells(model, &mut targets.cells));
        match result {
            Ok(()) => Ok(targets),
            Err(stop) => {
                for backing in targets.backings() {
                    self.release_scratch_backing(backing);
                }
                Err(stop)
            }
        }
    }
    /// The cell of the model that the first element with this ID is, if that
    /// element is a cell of it. What a search from the root would reach before
    /// a match decides the token if there is none.
    fn table_find_cell(
        &mut self,
        targets: &IdTargets<'a>,
        token: &[u16],
    ) -> TableResult<Option<usize>> {
        let entries = &targets.entries;
        self.table_charge_product(&[table_log2(entries.len()) + 1, token.len() + 1])?;
        let at = entries.partition_point(|entry| entry.value < token);
        let Some(entry) = entries.get(at).filter(|entry| entry.value == token) else {
            return match &targets.stop {
                None => Ok(None),
                Some(IdStop::Missing(missing)) => Err(TableStop::Missing(missing.clone())),
                Some(IdStop::Refused(error)) => {
                    Err(self.stop(HtmlSemanticFailureKind::Attribute(*error)).into())
                }
            };
        };
        let cells = &targets.cells;
        self.charge(table_log2(cells.len()) + 1)?;
        Ok(cells
            .binary_search_by_key(&entry.ordinal, |slot| slot.ordinal)
            .ok()
            .map(|at| cells[at].cell))
    }
    fn table_empty(&mut self, model: &HtmlTableModel, index: usize) -> TableResult<bool> {
        let mut children = self.table_children(&model.cells[index].subject)?;
        loop {
            self.charge(1)?;
            let Some(id) = children.next() else {
                return Ok(true);
            };
            match self.node(&id)?.kind() {
                HtmlCapturedNodeKind::Element(_) => return Ok(false),
                HtmlCapturedNodeKind::Text(text) => {
                    for unit in text.code_units() {
                        self.charge(1)?;
                        if !is_ascii_whitespace(*unit) {
                            return Ok(false);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    pub(super) fn table_headers(
        &mut self,
        model: &mut HtmlTableModel,
        document: &HtmlCaptureSubjectId,
        scratch: &mut Scratch<'a>,
    ) -> TableResult<()> {
        if model.cells.is_empty() {
            return Ok(());
        }
        // Without a header cell no scan or group finds one, so the indexes
        // that answer the scans are not built; only a `headers` attribute,
        // which may name any cell, can still assign a header.
        let mut any_header = false;
        for cell in &model.cells {
            self.charge(1)?;
            if cell.kind == HtmlTableCellKind::Header {
                any_header = true;
                break;
            }
        }
        if any_header {
            self.table_axes(model, scratch)?;
        }
        let Scratch {
            document: frames,
            targets,
            headers,
            stamps,
            listed,
            block,
            memo,
            specials,
            closed,
            window,
            notes,
            row_group_headers,
            column_group_headers,
            row_group_least,
            column_group_least,
            columns,
            rows,
            ..
        } = scratch;
        // A cell consults only the header cells whose `scope` names its group,
        // so those are found once, reading each header's `scope` in model order.
        let grouped =
            any_header && (!model.row_groups.is_empty() || !model.column_groups.is_empty());
        for index in 0..model.cells.len() {
            if !grouped {
                break;
            }
            if model.cells[index].kind != HtmlTableCellKind::Header {
                continue;
            }
            match self.table_scope(model, index, memo)? {
                HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::RowGroup)
                    if !model.row_groups.is_empty() =>
                {
                    self.table_scratch(row_group_headers, index)?;
                }
                HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::ColumnGroup)
                    if !model.column_groups.is_empty() =>
                {
                    self.table_scratch(column_group_headers, index)?;
                }
                _ => {}
            }
        }
        if !row_group_headers.is_empty() {
            self.table_group_tree(model, row_group_headers, row_group_least, |cell| cell.x)?;
        }
        if !column_group_headers.is_empty() {
            let count = column_group_headers.len();
            let bits = count
                .checked_ilog2()
                .and_then(|bits| usize::try_from(bits).ok())
                .unwrap_or(0);
            self.charge(count.saturating_mul(1 + bits))?;
            column_group_headers.sort_unstable_by_key(|index| (model.cells[*index].x, *index));
            self.table_group_tree(model, column_group_headers, column_group_least, |cell| {
                cell.y
            })?;
        }
        let group_headers = GroupHeaders {
            row_group: row_group_headers,
            row_group_least,
            column_group: column_group_headers,
            column_group_least,
        };
        let keys = rows.keys.len().max(columns.keys.len());
        self.charge(keys)?;
        self.reserve_scratch(stamps, keys)?;
        stamps.resize(keys, 0);
        let mut scan = Scan {
            headers,
            listed,
            mark: 0,
            stamps,
            epoch: 0,
            block,
            memo,
            specials,
            closed,
            window,
            notes,
            noting: false,
            axes: Axes { rows, columns },
            in_block: false,
            live: false,
        };
        for principal in 0..model.cells.len() {
            self.charge(1)?;
            scan.headers.clear();
            scan.mark = principal + 1;
            let cell = &model.cells[principal];
            let (cell_x, cell_y) = (cell.x, cell.y);
            let element = self.table_element(&cell.subject)?;
            if let Some(raw) = self.table_raw(element, "headers")? {
                let units = raw.value().code_units();
                let mut position = 0;
                while position < units.len() {
                    self.charge(1)?;
                    if is_ascii_whitespace(units[position]) {
                        position += 1;
                        continue;
                    }
                    let start = position;
                    while position < units.len() {
                        self.charge(1)?;
                        if is_ascii_whitespace(units[position]) {
                            break;
                        }
                        position += 1;
                    }
                    let index = match targets {
                        Some(index) => index,
                        None => targets.insert(self.table_index_targets(model, document, frames)?),
                    };
                    if let Some(found) = self.table_find_cell(index, &units[start..position])?
                        && found != principal
                    {
                        self.table_list(model, &mut scan, found)?;
                    }
                }
            } else if any_header {
                let right = self.table_add(cell_x, cell.width)?;
                let bottom = self.table_add(cell_y, cell.height)?;
                self.table_scan_axis(model, &mut scan, principal, false)?;
                self.table_scan_axis(model, &mut scan, principal, true)?;
                self.table_group_scopes(
                    model,
                    &mut scan,
                    group_headers,
                    principal,
                    Far { right, bottom },
                )?;
            }
            for position in 0..scan.headers.len() {
                self.charge(1)?;
                let header = scan.headers[position];
                if header == principal || self.table_empty_once(model, header, scan.memo)? {
                    continue;
                }
                self.table_push(
                    &mut model.headers,
                    HtmlTableHeaderAssociation {
                        cell: model.cells[principal].subject.clone(),
                        header: model.cells[header].subject.clone(),
                    },
                )?;
            }
        }
        Ok(())
    }
    /// The headers a row group or column group gives its cells: a th whose
    /// `scope` is `rowgroup` (or `colgroup`) in the principal cell's group and
    /// not below or right of the principal cell's far edges. Row groups and
    /// column groups are disjoint and sorted, so one group at most holds the
    /// principal cell, and the header cells sorted by row bound the candidates.
    fn table_group_scopes(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        group_headers: GroupHeaders<'_>,
        principal: usize,
        far: Far,
    ) -> TableResult<()> {
        let Some(cell) = model.cells.get(principal) else {
            return Ok(());
        };
        let (cell_x, cell_y) = (cell.x, cell.y);
        let group = slots::containing(
            &model.row_groups,
            cell_y,
            |g| Extent {
                start: g.y,
                length: g.height,
            },
            &mut |work| self.charge(work).map_err(TableStop::from),
        )?;
        if let Some(group) = group.and_then(|index| model.row_groups.get(index)) {
            let candidates = group_headers.row_group;
            let limit = self.table_add(group.y, group.height)?.min(far.bottom);
            let first = slots::leading_by(
                candidates.len(),
                |i| model.cells[candidates[i]].y < group.y,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let last = slots::leading_by(
                candidates.len(),
                |i| model.cells[candidates[i]].y < limit,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            // The candidates in the group's rows and left of the far edge.
            self.table_collect_below(
                group_headers.row_group_least,
                LineSpan { first, last },
                far.right,
                scan.specials,
            )?;
            for position in 0..scan.specials.len() {
                self.charge(1)?;
                if let Some(index) = candidates.get(scan.specials[position]) {
                    self.table_list(model, scan, *index)?;
                }
            }
        }
        let group = slots::containing(
            &model.column_groups,
            cell_x,
            |g| Extent {
                start: g.x,
                length: g.width,
            },
            &mut |work| self.charge(work).map_err(TableStop::from),
        )?;
        if let Some(group) = group.and_then(|index| model.column_groups.get(index)) {
            let candidates = group_headers.column_group;
            let columns = group.x..group.x.saturating_add(group.width).min(far.right);
            let first = slots::leading_by(
                candidates.len(),
                |i| model.cells[candidates[i]].x < columns.start,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            let last = slots::leading_by(
                candidates.len(),
                |i| model.cells[candidates[i]].x < columns.end,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            // The candidates in the group's columns and above the far edge,
            // listed in model order.
            self.table_collect_below(
                group_headers.column_group_least,
                LineSpan { first, last },
                far.bottom,
                scan.specials,
            )?;
            for position in 0..scan.specials.len() {
                self.charge(1)?;
                let at = scan.specials[position];
                if let Some(index) = candidates.get(at) {
                    scan.specials[position] = *index;
                }
            }
            let count = scan.specials.len();
            let bits = count
                .checked_ilog2()
                .and_then(|bits| usize::try_from(bits).ok())
                .unwrap_or(0);
            self.charge(count.saturating_mul(1 + bits))?;
            scan.specials.sort_unstable();
            for position in 0..scan.specials.len() {
                self.charge(1)?;
                let index = scan.specials[position];
                self.table_list(model, scan, index)?;
            }
        }
        Ok(())
    }
}
