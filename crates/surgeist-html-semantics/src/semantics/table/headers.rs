//! Assigning header cells (HTML 877d §4.9.12.2): the explicit `headers`
//! attribute, and the scans left and up from each cell through the row and
//! column indexes.
//!
//! A scan over a line of disjoint intervals visits each cell once and jumps
//! over a run of data cells, which only ends the current header block however
//! long it is. A line where cells overlap is scanned one slot at a time, the
//! slots with no occupant or several being skipped as the algorithm says.

use super::slots::{self, Cover, Extent, LineView, SlotAxis};
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
}

/// The finished row and column lines.
#[derive(Clone, Copy)]
struct Axes<'s> {
    rows: &'s SlotAxis,
    columns: &'s SlotAxis,
}

/// The scratch a header scan works in: the headers found for the principal
/// cell, the block and opaque cells of the scan in progress, what is known of
/// each header, and the finished lines.
struct Scan<'s> {
    headers: &'s mut Vec<usize>,
    opaque: &'s mut Vec<usize>,
    block: &'s mut Vec<usize>,
    memo: &'s mut Vec<HeaderMemo>,
    axes: Axes<'s>,
    in_block: bool,
}

/// Where a scan begins: the row or column to scan, and the slot in it that
/// the principal cell is anchored at (the scan starts one before it).
#[derive(Clone, Copy)]
struct ScanStart {
    line: usize,
    from: usize,
}

/// The first row and column beyond the principal cell.
#[derive(Clone, Copy)]
struct Far {
    right: usize,
    bottom: usize,
}

/// The header cells whose `scope` names a group, by kind.
#[derive(Clone, Copy)]
struct GroupHeaders<'s> {
    row_group: &'s [usize],
    column_group: &'s [usize],
}

impl<'a> HtmlSemanticSession<'a> {
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
        if memo.len() != model.cells.len() {
            self.charge(model.cells.len())?;
            self.reserve_scratch(memo, model.cells.len())?;
            memo.resize(model.cells.len(), HeaderMemo::default());
        }
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
                let bottom = self.table_add(header.y, header.height)?;
                let span = rows.lines_of(header.y, bottom, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let mut shared = false;
                for row in span.first..span.last {
                    self.charge(1)?;
                    if rows.lines.get(row).is_some_and(|line| line.data) {
                        shared = true;
                        break;
                    }
                }
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
                let right = self.table_add(header.x, header.width)?;
                let span = axes.columns.lines_of(header.x, right, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                let mut shared = false;
                for column in span.first..span.last {
                    self.charge(1)?;
                    if axes.columns.lines.get(column).is_some_and(|line| line.data) {
                        shared = true;
                        break;
                    }
                }
                if let Some(entry) = memo.get_mut(index) {
                    entry.row = Some(!shared);
                }
                Ok(!shared)
            }
            _ => Ok(false),
        }
    }
    /// A data cell reached by a scan ends the header block it follows: the
    /// block's cells become opaque to earlier headers of the same extent.
    fn table_end_block(&mut self, scan: &mut Scan<'_>) -> TableResult<()> {
        if !scan.in_block {
            return Ok(());
        }
        scan.in_block = false;
        for index in scan.block.iter() {
            self.charge(1)?;
            self.table_scratch(scan.opaque, *index)?;
        }
        self.charge(1)?;
        scan.block.clear();
        Ok(())
    }
    /// A header cell reached by a scan joins the current block, and is a
    /// header of the principal cell unless an opaque header of the same extent
    /// hides it or it is not a header of the scan's kind.
    fn table_reach_header(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        index: usize,
        vertical: bool,
    ) -> TableResult<()> {
        scan.in_block = true;
        self.table_scratch(scan.block, index)?;
        let cell = &model.cells[index];
        let mut blocked = false;
        for opaque in scan.opaque.iter() {
            self.charge(1)?;
            let old = &model.cells[*opaque];
            if (vertical && old.x == cell.x && old.width == cell.width)
                || (!vertical && old.y == cell.y && old.height == cell.height)
            {
                blocked = true;
                break;
            }
        }
        if !blocked {
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
    /// Scans from the principal cell toward the start of one row (leftward) or
    /// one column (upward), taking the occupant of each slot in turn. Visiting
    /// a cell once instead of once per slot it covers changes nothing, because
    /// a repeated visit re-adds a header already found (duplicates are dropped
    /// when associations are made), a block member already present, and a data
    /// cell with no block left to end.
    fn table_scan(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        principal: usize,
        start: ScanStart,
        vertical: bool,
    ) -> TableResult<()> {
        self.charge(1)?;
        scan.opaque.clear();
        scan.block.clear();
        scan.in_block = model.cells[principal].kind == HtmlTableCellKind::Header;
        if scan.in_block {
            self.table_scratch(scan.block, principal)?;
        }
        let axis = if vertical {
            scan.axes.columns
        } else {
            scan.axes.rows
        };
        let Some(LineView { line, entries }) = axis.line(start.line) else {
            return Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into());
        };
        if line.regular {
            let mut before =
                slots::leading(entries, |entry| entry.start < start.from, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
            while before > 0 {
                self.charge(1)?;
                let last = before - 1;
                let Some(position) = entries
                    .get(last)
                    .and_then(|entry| entry.header)
                    .and_then(|header| header.checked_sub(line.first))
                else {
                    // Every remaining entry is a data cell.
                    return self.table_end_block(scan);
                };
                if position < last {
                    self.table_end_block(scan)?;
                }
                let cell = entries.get(position).map_or(principal, |entry| entry.cell);
                self.table_reach_header(model, scan, cell, vertical)?;
                before = position;
            }
            return Ok(());
        }
        let mut slot = start.from;
        loop {
            self.charge(1)?;
            if slot == 0 {
                return Ok(());
            }
            slot -= 1;
            let cover = slots::cover(entries, slot, &mut |work| {
                self.charge(work).map_err(TableStop::from)
            })?;
            let Cover::One(index) = cover else {
                continue;
            };
            if model.cells[index].kind == HtmlTableCellKind::Header {
                self.table_reach_header(model, scan, index, vertical)?;
            } else {
                self.table_end_block(scan)?;
            }
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
            opaque,
            block,
            memo,
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
        let mut scan = Scan {
            headers,
            opaque,
            block,
            memo,
            axes: Axes { rows, columns },
            in_block: false,
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
                // Rows (or columns) of one line are covered by the same cells
                // in the same way, so one scan of the line serves all of them.
                let span = scan.axes.rows.lines_of(cell_y, bottom, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                for line in span.first..span.last {
                    let start = ScanStart { line, from: cell_x };
                    self.table_scan(model, &mut scan, principal, start, false)?;
                }
                let span = scan.axes.columns.lines_of(cell_x, right, &mut |work| {
                    self.charge(work).map_err(TableStop::from)
                })?;
                for line in span.first..span.last {
                    let start = ScanStart { line, from: cell_y };
                    self.table_scan(model, &mut scan, principal, start, true)?;
                }
                self.table_group_scopes(
                    model,
                    &mut scan,
                    group_headers,
                    principal,
                    Far { right, bottom },
                )?;
            }
            for (position, header) in scan.headers.iter().enumerate() {
                self.charge(1)?;
                let header = *header;
                if self.table_empty(model, header)? {
                    continue;
                }
                let mut duplicate = false;
                for earlier in &scan.headers[..position] {
                    self.charge(1)?;
                    if *earlier == header {
                        duplicate = true;
                        break;
                    }
                }
                if duplicate || header == principal {
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
