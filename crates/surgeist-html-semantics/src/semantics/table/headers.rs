//! Assigning header cells (HTML 877d §4.9.12.2): the explicit `headers`
//! attribute, and the scans left and up from each cell through the row and
//! column indexes.
//!
//! A scan over a line of disjoint intervals visits each cell once and jumps
//! over a run of data cells, which only ends the current header block however
//! long it is. A line where cells overlap is scanned one slot at a time, the
//! slots with no occupant or several being skipped as the algorithm says.

use super::slots::{self, Cover, Extent, LineSpan, List, SlotAxis, SlotEntry};
use super::*;

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
    /// A header whose key is stamped with `epoch` is hidden: an opaque header
    /// of this scan crosses the same lines.
    stamps: &'s mut Vec<usize>,
    epoch: usize,
    block: &'s mut Vec<usize>,
    memo: &'s mut Vec<HeaderMemo>,
    /// The lines of a scan that must be scanned one at a time.
    specials: &'s mut Vec<usize>,
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
    column_group: &'s [usize],
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
    /// A data cell reached by a scan ends the header block it follows: the
    /// block's extents become opaque to earlier headers that cross the same
    /// lines.
    fn table_end_block(&mut self, scan: &mut Scan<'_>) -> TableResult<()> {
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
            }
        }
        self.charge(1)?;
        scan.block.clear();
        Ok(())
    }
    /// A header cell reached by a scan joins the current block, and is a
    /// header of the principal cell unless an opaque header that crosses the
    /// same lines hides it or it is not a header of the scan's kind.
    fn table_reach_header(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        index: usize,
        key: usize,
        vertical: bool,
    ) -> TableResult<()> {
        scan.in_block = true;
        self.table_scratch(scan.block, key)?;
        self.charge(1)?;
        let mut blocked = scan.stamps.get(key) == Some(&scan.epoch);
        if !blocked {
            scan.live = true;
            blocked = if vertical {
                !self.table_column_header(model, index, scan.axes.rows, scan.memo)?
            } else {
                !self.table_row_header(model, index, scan.axes, scan.memo)?
            };
        }
        if !blocked {
            self.table_scratch(scan.headers, index)?;
        }
        Ok(())
    }
    /// Scans from the principal cell toward the start of each row it crosses
    /// (leftward) or of each column it crosses (upward), taking the occupant of
    /// each slot in turn.
    ///
    /// The lines of one run hold the same long cells, so the scans of those
    /// lines differ only by their short cells and by overlaps. A line with a
    /// short cell before the principal cell, or where two cells share a slot,
    /// is scanned on its own; all the others, however many, are scanned once.
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
            for extent in &axis.irregular {
                self.charge(1)?;
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
            // headers are found in the order the specification lists them:
            // the shared scan stands in for the first line that has no cell of
            // its own before the scanning cell.
            let mut shared = Some(here.start);
            for position in 0..scan.specials.len() {
                self.charge(1)?;
                let line = scan.specials[position];
                if let Some(first) = shared
                    && first < line
                {
                    self.table_scan_shared(model, scan, principal, target, long)?;
                    shared = None;
                }
                if shared == Some(line) {
                    shared = Some(line + 1);
                }
                let short =
                    axis.short_of(line, &mut |work| self.charge(work).map_err(TableStop::from))?;
                let irregular = axis
                    .irregular_at(line, &mut |work| self.charge(work).map_err(TableStop::from))?;
                let cells = Occupants { long, short };
                self.table_scan_line(model, scan, principal, target, cells, irregular)?;
            }
            if let Some(first) = shared
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
                    self.table_reach_header(model, scan, index, key, vertical)?;
                } else {
                    self.table_end_block(scan)?;
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
            let nearest = match (
                last_long.and_then(|p| cells.long.entries.get(p)),
                last_short.and_then(|p| cells.short.entries.get(p)),
            ) {
                (None, None) => return Ok(()),
                (Some(entry), None) | (None, Some(entry)) => entry,
                (Some(a), Some(b)) => {
                    if a.start > b.start {
                        a
                    } else {
                        b
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
                self.table_reach_header(model, scan, nearest.cell, key, vertical)?;
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
            self.table_end_block(scan)?;
            let Some(start) = nearest_header else {
                return Ok(());
            };
            bound = start + 1;
        }
    }
    fn table_id_match(&mut self, left: &[u16], right: &[u16]) -> TableResult<bool> {
        self.charge(1)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.charge(2)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn table_find_id(
        &mut self,
        document: &HtmlCaptureSubjectId,
        token: &[u16],
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> TableResult<Option<HtmlCaptureSubjectId>> {
        self.charge(1)?;
        frames.clear();
        let children = self.table_children(document)?;
        self.table_scratch(frames, children)?;
        while !frames.is_empty() {
            self.charge(1)?;
            let Some(id) = frames.last_mut().and_then(Iterator::next) else {
                frames.pop();
                continue;
            };
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                let raw = self.table_raw(element, "id")?;
                if let Some(raw) = raw
                    && !raw.value().is_empty()
                    && self.table_id_match(raw.value().code_units(), token)?
                {
                    return Ok(Some(id));
                }
            }
            let children = self.table_children(&id)?;
            if children.len() != 0 {
                self.table_scratch(frames, children)?;
            }
        }
        Ok(None)
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
            headers,
            stamps,
            seen,
            block,
            memo,
            specials,
            row_group_headers,
            column_group_headers,
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
        let group_headers = GroupHeaders {
            row_group: row_group_headers,
            column_group: column_group_headers,
        };
        let keys = rows.keys.len().max(columns.keys.len());
        self.charge(keys)?;
        self.reserve_scratch(stamps, keys)?;
        stamps.resize(keys, 0);
        let mut scan = Scan {
            headers,
            stamps,
            epoch: 0,
            block,
            memo,
            specials,
            axes: Axes { rows, columns },
            in_block: false,
            live: false,
        };
        for principal in 0..model.cells.len() {
            self.charge(1)?;
            scan.headers.clear();
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
                    if let Some(id) =
                        self.table_find_id(document, &units[start..position], frames)?
                    {
                        for (index, cell) in model.cells.iter().enumerate() {
                            self.charge(1)?;
                            if cell.subject == id && index != principal {
                                self.table_scratch(scan.headers, index)?;
                                break;
                            }
                        }
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
            if !scan.headers.is_empty() && seen.len() != model.cells.len() {
                self.charge(model.cells.len())?;
                self.reserve_scratch(seen, model.cells.len())?;
                seen.resize(model.cells.len(), 0);
            }
            for position in 0..scan.headers.len() {
                self.charge(1)?;
                let header = scan.headers[position];
                if self.table_empty_once(model, header, scan.memo)? {
                    continue;
                }
                // Each cell is listed once for a principal cell: the first
                // time it is found, and never the principal cell itself.
                let first = seen.get(header).is_some_and(|mark| *mark != principal + 1);
                if let Some(mark) = seen.get_mut(header) {
                    *mark = principal + 1;
                }
                if !first || header == principal {
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
            for &index in candidates.get(first..last).unwrap_or_default() {
                self.charge(1)?;
                if model.cells[index].x < far.right {
                    self.table_scratch(scan.headers, index)?;
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
            let last = slots::leading_by(
                candidates.len(),
                |i| model.cells[candidates[i]].y < far.bottom,
                &mut |work| self.charge(work).map_err(TableStop::from),
            )?;
            for &index in candidates.get(..last).unwrap_or_default() {
                self.charge(1)?;
                let header = &model.cells[index];
                if header.x >= group.x && header.x - group.x < group.width && header.x < far.right {
                    self.table_scratch(scan.headers, index)?;
                }
            }
        }
        Ok(())
    }
}
