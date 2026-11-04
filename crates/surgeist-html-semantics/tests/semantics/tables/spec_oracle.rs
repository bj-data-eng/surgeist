//! The table model against a literal reading of the pinned specification.
//!
//! `Oracle` below implements HTML 877d §4.9.12.1 (forming a table), the
//! algorithm for assigning header cells and its internal scanning algorithm
//! (§4.9.12.2) step by step on a dense grid in which every slot lists the
//! cells covering it. It shares nothing with the crate's implementation: no
//! interval index, no lines, no deferred growth. Seeded random tables, from
//! plain to dense with overlapping spans, are described once, built into a
//! capture and formed by both; every part of the model that the specification
//! defines must agree.
//!
//! What the specification leaves unordered is compared as a set, and nothing
//! else is: the table model errors (it names one condition, not a list), the
//! cells of the model, which cells have header cells (the algorithm runs for
//! each cell on its own), and, within one cell's header list, the cells that
//! the row group and column group steps add after the scans, which the steps
//! name as "all header cells that ..." without an order. The header list
//! itself is ordered: the `headers` attribute's ids, or the rows' scans from
//! top to bottom and then the columns' from left to right, each scan taking
//! the headers it meets nearest first; empty cells, duplicates and the
//! principal cell are removed from it keeping the first of each. The row
//! groups, column groups and the row and column sources are ordered too.
//!
//! Spans are written 0 to 9 and as garbage, scopes in every spelling, and the
//! `headers` attribute names cells, a caption that shares an id, elements
//! outside the table that own an id before or after its cells, cells that
//! repeat another's id, and unknown ids; cells are empty, blank, text or hold
//! an element. Every element of a kind carries its position in tree order as
//! its ordinal.

use super::scope_work::{element, limits, query};
use crate::semantics::support::wide_builder_without_registries;
use std::collections::BTreeMap;
use surgeist_html::values::DocumentString;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureSubjectId,
    HtmlSemanticFact, HtmlSemanticValue, HtmlTableCellKind, HtmlTableModel, HtmlTableModelError,
};

struct Rng(u64);
impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        usize::try_from((self.0 >> 33) % u64::try_from(n).unwrap()).unwrap()
    }
    fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }
}

// ---- A table as plain data ----------------------------------------------

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Content {
    Empty,
    Blank,
    Text,
    Element,
}

#[derive(Clone, Debug)]
struct Cell {
    ordinal: usize,
    /// The cell takes the id of the cell with this ordinal (any, in tree
    /// order, before or after its own) instead of its own: ids repeat, and the
    /// first element in the document with an id decides it.
    alias: Option<usize>,
    th: bool,
    colspan: Option<String>,
    rowspan: Option<String>,
    scope: Option<String>,
    headers: Option<String>,
    content: Content,
}

impl Cell {
    fn id(&self) -> String {
        format!("c{}", self.alias.unwrap_or(self.ordinal))
    }
}

#[derive(Clone, Debug)]
struct Row {
    ordinal: usize,
    cells: Vec<Cell>,
}

#[derive(Clone, Debug)]
struct Col {
    ordinal: usize,
    span: Option<String>,
}

#[derive(Clone, Debug)]
enum Child {
    Caption {
        ordinal: usize,
        id: Option<String>,
    },
    Colgroup {
        ordinal: usize,
        span: Option<String>,
        cols: Vec<Col>,
    },
    Row(Row),
    Section {
        ordinal: usize,
        kind: &'static str,
        rows: Vec<Row>,
    },
    Other,
    /// An element outside the table, in the document before it or after it,
    /// that owns an id a cell may name.
    Outside {
        id: String,
        after: bool,
    },
}

/// Counters for the ordinals of each element kind.
#[derive(Default)]
struct Ordinals {
    captions: usize,
    colgroups: usize,
    cols: usize,
    rows: usize,
    sections: usize,
    cells: usize,
}

fn span(rng: &mut Rng, dense: bool) -> Option<String> {
    match rng.below(if dense { 5 } else { 12 }) {
        0 => Some("0".into()),
        1 | 2 => Some(rng.below(10).to_string()),
        3 if !dense => Some(["x", "bogus", ""][rng.below(3)].into()),
        _ => None,
    }
}

fn cell(rng: &mut Rng, dense: bool, ordinals: &mut Ordinals) -> Cell {
    let th = rng.chance(2);
    let scope = th
        .then(|| match rng.below(10) {
            0 => Some("row"),
            1 => Some("COL"),
            2 => Some("rowgroup"),
            3 => Some("ColGroup"),
            4 => Some("auto"),
            5 => Some("bogus"),
            _ => None,
        })
        .flatten()
        .map(String::from);
    let headers = rng.chance(8).then(|| {
        let tokens: Vec<String> = (0..rng.below(4))
            .map(|_| {
                if rng.chance(5) {
                    "missing".to_string()
                } else {
                    format!("c{}", rng.below(30))
                }
            })
            .collect();
        tokens.join(["  ", " ", "\t"][rng.below(3)])
    });
    let ordinal = ordinals.cells;
    ordinals.cells += 1;
    Cell {
        ordinal,
        alias: rng.chance(10).then(|| rng.below(30)),
        th,
        colspan: span(rng, dense),
        rowspan: span(rng, dense),
        scope,
        headers,
        content: [
            Content::Empty,
            Content::Blank,
            Content::Text,
            Content::Text,
            Content::Element,
        ][rng.below(5)],
    }
}

fn row(rng: &mut Rng, dense: bool, ordinals: &mut Ordinals) -> Row {
    let ordinal = ordinals.rows;
    ordinals.rows += 1;
    let max_cells = if dense { 5 } else { 4 };
    let cells = (0..rng.below(max_cells + 1))
        .map(|_| cell(rng, dense, ordinals))
        .collect();
    Row { ordinal, cells }
}

fn table(rng: &mut Rng, dense: bool) -> Vec<Child> {
    let mut children = Vec::new();
    let mut ordinals = Ordinals::default();
    if rng.chance(6) {
        // The first element in the document with an id may lie before the
        // table, and then no cell of the table has it.
        children.push(Child::Outside {
            id: format!("c{}", rng.below(30)),
            after: false,
        });
    }
    if rng.chance(3) {
        // A caption may take an id a cell later claims: the first element in
        // the document with the id is then not a cell.
        children.push(Child::Caption {
            ordinal: 0,
            id: rng.chance(2).then(|| "c3".to_string()),
        });
        ordinals.captions = 1;
    }
    for _ in 0..rng.below(3) {
        let ordinal = ordinals.colgroups;
        ordinals.colgroups += 1;
        let cols = (0..rng.below(3))
            .map(|_| {
                ordinals.cols += 1;
                Col {
                    ordinal: ordinals.cols - 1,
                    span: span(rng, false),
                }
            })
            .collect();
        children.push(Child::Colgroup {
            ordinal,
            span: span(rng, false),
            cols,
        });
        if rng.chance(4) {
            children.push(Child::Other);
        }
    }
    let max_rows = if dense { 6 } else { 4 };
    for _ in 0..1 + rng.below(4) {
        let kind = rng.below(6);
        if kind < 2 {
            children.push(Child::Row(row(rng, dense, &mut ordinals)));
        } else {
            let ordinal = ordinals.sections;
            ordinals.sections += 1;
            let rows = (0..1 + rng.below(max_rows))
                .map(|_| row(rng, dense, &mut ordinals))
                .collect();
            children.push(Child::Section {
                ordinal,
                kind: ["thead", "tbody", "tbody", "tfoot"][kind - 2],
                rows,
            });
        }
        if rng.chance(6) {
            // A colgroup after rows takes no part in the table.
            let ordinal = ordinals.colgroups;
            ordinals.colgroups += 1;
            children.push(Child::Colgroup {
                ordinal,
                span: Some("2".into()),
                cols: Vec::new(),
            });
        }
    }
    if rng.chance(6) {
        // After the table, an element with an id never takes it from a cell.
        children.push(Child::Outside {
            id: format!("c{}", rng.below(30)),
            after: true,
        });
    }
    children
}

// ---- The specification, literally ---------------------------------------

#[derive(Clone, Debug)]
struct Placed {
    header: bool,
    ordinal: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

type CellFacts = (usize, bool, usize, usize, usize, usize);

#[derive(Debug, Default, PartialEq)]
struct Expected {
    width: usize,
    height: usize,
    caption: Option<usize>,
    cells: Vec<CellFacts>,
    row_sources: Vec<Option<usize>>,
    column_sources: Vec<Option<usize>>,
    row_groups: Vec<(usize, usize, usize)>,
    column_groups: Vec<(usize, usize, usize)>,
    errors: Vec<String>,
}

/// One cell's header list: the cells found by the `headers` attribute or the
/// scans, in order, then the cells the groups add, which have no order.
#[derive(Debug, PartialEq)]
struct Associated {
    ordered: Vec<usize>,
    grouped: Vec<usize>,
}

/// The rules for parsing non-negative integers, for the spellings written.
fn parse(value: &Option<String>) -> Option<usize> {
    let text = value
        .as_deref()?
        .trim_start_matches([' ', '\t', '\n', '\u{c}', '\r']);
    let text = text.strip_prefix('+').unwrap_or(text);
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn column_span(value: &Option<String>) -> usize {
    match parse(value) {
        None | Some(0) => 1,
        Some(n) => n.min(1000),
    }
}

/// Everything the specification's algorithms keep, as plain variables.
#[derive(Default)]
struct Oracle {
    xwidth: usize,
    yheight: usize,
    ycurrent: usize,
    cells: Vec<Placed>,
    /// The cells covering each slot, by row then column.
    grid: Vec<Vec<Vec<usize>>>,
    /// {cell, cellx, width} tuples of the downward-growing cells.
    downward: Vec<(usize, usize, usize)>,
    out: Expected,
    errors: BTreeMap<String, ()>,
    /// Per cell ordinal: content, scope attribute, headers attribute.
    facts: Vec<(Content, Option<String>, Option<String>)>,
    /// Ids in document order: the caption's, then each cell's, with the cell
    /// ordinal for a cell.
    ids: Vec<(String, Option<usize>)>,
}

impl Oracle {
    fn resize_grid(&mut self) {
        self.grid.resize(self.yheight, Vec::new());
        for row in &mut self.grid {
            row.resize(self.xwidth, Vec::new());
        }
    }

    /// The algorithm for growing downward-growing cells.
    fn grow_downward(&mut self) {
        for (index, cellx, width) in self.downward.clone() {
            for x in cellx..cellx + width {
                self.grid[self.ycurrent][x].push(index);
            }
            self.cells[index].height = self.ycurrent - self.cells[index].y + 1;
        }
    }

    /// The algorithm for ending a row group.
    fn end_row_group(&mut self) {
        while self.ycurrent < self.yheight {
            self.grow_downward();
            self.ycurrent += 1;
        }
        self.downward.clear();
    }

    /// The algorithm for processing rows.
    fn process_row(&mut self, row: &Row) {
        if self.yheight == self.ycurrent {
            self.yheight += 1;
            self.resize_grid();
        }
        self.out.row_sources.resize(self.yheight, None);
        self.out.row_sources[self.ycurrent] = Some(row.ordinal);
        let mut xcurrent = 0;
        self.grow_downward();
        if row.cells.is_empty() {
            self.ycurrent += 1;
            return;
        }
        for element in &row.cells {
            while xcurrent < self.xwidth && !self.grid[self.ycurrent][xcurrent].is_empty() {
                xcurrent += 1;
            }
            if xcurrent == self.xwidth {
                self.xwidth += 1;
                self.resize_grid();
            }
            let colspan = column_span(&element.colspan);
            let mut rowspan = parse(&element.rowspan).unwrap_or(1).min(65534);
            let grows = rowspan == 0;
            if grows {
                rowspan = 1;
            }
            self.xwidth = self.xwidth.max(xcurrent + colspan);
            self.yheight = self.yheight.max(self.ycurrent + rowspan);
            self.resize_grid();
            let index = self.cells.len();
            // Slots that already had a cell covering them are a table model
            // error: one per cell covering any of them, at its first slot.
            let mut shared: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
            for y in self.ycurrent..self.ycurrent + rowspan {
                for x in xcurrent..xcurrent + colspan {
                    for &other in &self.grid[y][x] {
                        let at = shared.entry(other).or_insert((x, y));
                        *at = (at.0.min(x), at.1.min(y));
                    }
                }
            }
            for (other, (x, y)) in shared {
                self.errors.insert(
                    format!(
                        "overlap {} {} {x},{y}",
                        element.ordinal, self.cells[other].ordinal
                    ),
                    (),
                );
            }
            self.cells.push(Placed {
                header: element.th,
                ordinal: element.ordinal,
                x: xcurrent,
                y: self.ycurrent,
                width: colspan,
                height: rowspan,
            });
            for y in self.ycurrent..self.ycurrent + rowspan {
                for x in xcurrent..xcurrent + colspan {
                    self.grid[y][x].push(index);
                }
            }
            if grows {
                self.downward.push((index, xcurrent, colspan));
            }
            xcurrent += colspan;
        }
        self.ycurrent += 1;
    }

    /// The algorithm for processing row groups.
    fn process_row_group(&mut self, ordinal: usize, rows: &[Row]) {
        let ystart = self.yheight;
        for row in rows {
            self.process_row(row);
        }
        if self.yheight > ystart {
            self.out
                .row_groups
                .push((ordinal, ystart, self.yheight - ystart));
        }
        self.end_row_group();
    }

    /// Forming a table.
    fn form(&mut self, table: &[Child]) {
        let mut pending: Vec<(usize, &[Row])> = Vec::new();
        let mut columns_phase = true;
        for child in table {
            match child {
                Child::Caption { ordinal, .. } => {
                    self.out.caption.get_or_insert(*ordinal);
                }
                Child::Colgroup {
                    ordinal,
                    span,
                    cols,
                } if columns_phase => {
                    let xstart = self.xwidth;
                    if cols.is_empty() {
                        self.xwidth += column_span(span);
                        self.out.column_sources.resize(self.xwidth, None);
                    }
                    for col in cols {
                        let n = column_span(&col.span);
                        self.xwidth += n;
                        self.out.column_sources.resize(self.xwidth, None);
                        for x in self.xwidth - n..self.xwidth {
                            self.out.column_sources[x] = Some(col.ordinal);
                        }
                    }
                    self.out
                        .column_groups
                        .push((*ordinal, xstart, self.xwidth - xstart));
                }
                Child::Row(row) => {
                    columns_phase = false;
                    self.process_row(row);
                }
                Child::Section {
                    ordinal,
                    kind,
                    rows,
                } => {
                    columns_phase = false;
                    self.end_row_group();
                    if *kind == "tfoot" {
                        pending.push((*ordinal, rows));
                    } else {
                        self.process_row_group(*ordinal, rows);
                    }
                }
                Child::Colgroup { .. } | Child::Other | Child::Outside { .. } => {}
            }
        }
        for (ordinal, rows) in pending {
            self.process_row_group(ordinal, rows);
        }
        // A row or column with no cell anchored to any slot is an error.
        for y in 0..self.yheight {
            if !self.cells.iter().any(|c| c.y == y) {
                self.errors.insert(format!("row {y}"), ());
            }
        }
        for x in 0..self.xwidth {
            if !self.cells.iter().any(|c| c.x == x) {
                self.errors.insert(format!("column {x}"), ());
            }
        }
        self.resize_grid();
        self.out.row_sources.resize(self.yheight, None);
        self.out.column_sources.resize(self.xwidth, None);
    }

    fn is_empty_cell(&self, index: usize) -> bool {
        matches!(
            self.facts[self.cells[index].ordinal].0,
            Content::Empty | Content::Blank
        )
    }

    /// The scope attribute: row, col, rowgroup, colgroup or auto, ASCII
    /// case-insensitively; the missing and invalid value default is auto.
    fn scope(&self, index: usize) -> &'static str {
        match self.facts[self.cells[index].ordinal]
            .1
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("row") => "row",
            Some("col") => "col",
            Some("rowgroup") => "rowgroup",
            Some("colgroup") => "colgroup",
            _ => "auto",
        }
    }

    fn covered_by_data(&self, x: usize, y: usize) -> bool {
        self.grid[y][x].iter().any(|&o| !self.cells[o].header)
    }

    fn is_column_header(&self, index: usize) -> bool {
        let c = &self.cells[index];
        match self.scope(index) {
            "col" => true,
            "auto" => {
                !(c.y..c.y + c.height).any(|y| (0..self.xwidth).any(|x| self.covered_by_data(x, y)))
            }
            _ => false,
        }
    }

    fn is_row_header(&self, index: usize) -> bool {
        let c = &self.cells[index];
        match self.scope(index) {
            "row" => true,
            "auto" => {
                !self.is_column_header(index)
                    && !(c.x..c.x + c.width)
                        .any(|x| (0..self.yheight).any(|y| self.covered_by_data(x, y)))
            }
            _ => false,
        }
    }

    /// The internal algorithm for scanning and assigning header cells.
    fn scan(
        &self,
        principal: usize,
        list: &mut Vec<usize>,
        initial: (usize, usize),
        delta: (isize, isize),
    ) {
        let (mut x, mut y) = (
            isize::try_from(initial.0).unwrap(),
            isize::try_from(initial.1).unwrap(),
        );
        let mut opaque: Vec<usize> = Vec::new();
        let (mut in_block, mut block) = if self.cells[principal].header {
            (true, vec![principal])
        } else {
            (false, Vec::new())
        };
        loop {
            x += delta.0;
            y += delta.1;
            if x < 0 || y < 0 {
                return;
            }
            let [current] =
                self.grid[usize::try_from(y).unwrap()][usize::try_from(x).unwrap()].as_slice()
            else {
                continue;
            };
            let current = *current;
            let cur = &self.cells[current];
            if cur.header {
                in_block = true;
                block.push(current);
                let mut blocked = false;
                if delta.0 == 0 {
                    blocked |= opaque
                        .iter()
                        .any(|&o| self.cells[o].x == cur.x && self.cells[o].width == cur.width);
                    blocked |= !self.is_column_header(current);
                }
                if delta.1 == 0 {
                    blocked |= opaque
                        .iter()
                        .any(|&o| self.cells[o].y == cur.y && self.cells[o].height == cur.height);
                    blocked |= !self.is_row_header(current);
                }
                if !blocked {
                    list.push(current);
                }
            } else if in_block {
                in_block = false;
                opaque.append(&mut block);
            }
        }
    }

    /// The algorithm for assigning header cells to `principal`: the header
    /// list in order, and the cells its group steps add after that.
    fn assign_headers(&self, principal: usize) -> (Vec<usize>, Vec<usize>) {
        let mut list = Vec::new();
        let mut grouped = Vec::new();
        let p = &self.cells[principal];
        if let Some(value) = &self.facts[p.ordinal].2 {
            for token in value
                .split([' ', '\t', '\n', '\u{c}', '\r'])
                .filter(|t| !t.is_empty())
            {
                // The first element in the document with the id, if a cell.
                if let Some((_, Some(ordinal))) = self.ids.iter().find(|(id, _)| id == token)
                    && let Some(found) = self.cells.iter().position(|c| c.ordinal == *ordinal)
                    && found != principal
                {
                    list.push(found);
                }
            }
        } else {
            for y in p.y..p.y + p.height {
                self.scan(principal, &mut list, (p.x, y), (-1, 0));
            }
            for x in p.x..p.x + p.width {
                self.scan(principal, &mut list, (x, p.y), (0, -1));
            }
            let (right, bottom) = (p.x + p.width - 1, p.y + p.height - 1);
            for &(_, ystart, height) in &self.out.row_groups {
                if (ystart..ystart + height).contains(&p.y) {
                    for (index, c) in self.cells.iter().enumerate() {
                        if c.header
                            && self.scope(index) == "rowgroup"
                            && (ystart..ystart + height).contains(&c.y)
                            && c.x <= right
                            && c.y <= bottom
                        {
                            grouped.push(index);
                        }
                    }
                }
            }
            for &(_, xstart, width) in &self.out.column_groups {
                if (xstart..xstart + width).contains(&p.x) {
                    for (index, c) in self.cells.iter().enumerate() {
                        if c.header
                            && self.scope(index) == "colgroup"
                            && (xstart..xstart + width).contains(&c.x)
                            && c.x <= right
                            && c.y <= bottom
                        {
                            grouped.push(index);
                        }
                    }
                }
            }
        }
        list.retain(|&c| !self.is_empty_cell(c));
        grouped.retain(|&c| !self.is_empty_cell(c));
        let mut seen = Vec::new();
        list.retain(|c| {
            let new = !seen.contains(c);
            seen.push(*c);
            new
        });
        grouped.retain(|c| {
            let new = !seen.contains(c);
            seen.push(*c);
            new
        });
        list.retain(|&c| c != principal);
        grouped.retain(|&c| c != principal);
        (list, grouped)
    }
}

/// Forms the table and assigns headers by the specification's algorithms.
fn oracle(table: &[Child]) -> (Expected, BTreeMap<usize, Associated>) {
    let mut o = Oracle::default();
    for child in table {
        let rows: Vec<&Row> = match child {
            Child::Caption { id: Some(id), .. } | Child::Outside { id, .. } => {
                o.ids.push((id.clone(), None));
                Vec::new()
            }
            Child::Row(row) => vec![row],
            Child::Section { rows, .. } => rows.iter().collect(),
            _ => Vec::new(),
        };
        for row in rows {
            for c in &row.cells {
                o.ids.push((c.id(), Some(c.ordinal)));
                o.facts.resize(
                    o.facts.len().max(c.ordinal + 1),
                    (Content::Empty, None, None),
                );
                o.facts[c.ordinal] = (c.content, c.scope.clone(), c.headers.clone());
            }
        }
    }
    o.form(table);
    o.out.width = o.xwidth;
    o.out.height = o.yheight;
    o.out.errors = o.errors.keys().cloned().collect();
    o.out.cells = o
        .cells
        .iter()
        .map(|c| (c.ordinal, c.header, c.x, c.y, c.width, c.height))
        .collect();
    o.out.cells.sort_unstable();
    let mut headers = BTreeMap::new();
    for index in 0..o.cells.len() {
        let (ordered, mut grouped) = o.assign_headers(index);
        let ordinal = |cell: usize| o.cells[cell].ordinal;
        grouped.sort_by_key(|cell| ordinal(*cell));
        headers.insert(
            ordinal(index),
            Associated {
                ordered: ordered.iter().map(|c| ordinal(*c)).collect(),
                grouped: grouped.iter().map(|c| ordinal(*c)).collect(),
            },
        );
    }
    (o.out, headers)
}

// ---- Building the capture and reading the model back --------------------

type Subject = HtmlCaptureBuilderSubject;

#[derive(Default)]
struct Subjects {
    captions: Vec<Subject>,
    cells: Vec<Subject>,
    rows: Vec<Subject>,
    sections: Vec<Subject>,
    cols: Vec<Subject>,
    colgroups: Vec<Subject>,
}

fn attributes<'a>(pairs: &[(&'a str, &'a Option<String>)]) -> Vec<(&'a str, &'a str)> {
    pairs
        .iter()
        .filter_map(|(name, value)| value.as_deref().map(|v| (*name, v)))
        .collect()
}

fn build_row(
    b: &mut HtmlCaptureBuilder,
    d: &Subject,
    row: &Row,
    subjects: &mut Subjects,
) -> Subject {
    let mut in_row = Vec::new();
    for c in &row.cells {
        let id = Some(c.id());
        let mut attrs = attributes(&[
            ("colspan", &c.colspan),
            ("rowspan", &c.rowspan),
            ("scope", &c.scope),
            ("headers", &c.headers),
        ]);
        attrs.push(("id", id.as_deref().unwrap_or_default()));
        let e = element(b, d, if c.th { "th" } else { "td" }, &attrs);
        let kids = match c.content {
            Content::Empty => Vec::new(),
            Content::Blank => vec![b.add_text(d, DocumentString::from(" \n\t ")).unwrap()],
            Content::Text => vec![b.add_text(d, DocumentString::from("x")).unwrap()],
            Content::Element => {
                let span = element(b, d, "span", &[]);
                b.establish_children(&span, &[]).unwrap();
                vec![span]
            }
        };
        b.establish_children(&e, &kids).unwrap();
        subjects.cells.push(e.clone());
        in_row.push(e);
    }
    let tr = element(b, d, "tr", &[]);
    b.establish_children(&tr, &in_row).unwrap();
    subjects.rows.push(tr.clone());
    tr
}

fn build(table: &[Child]) -> (HtmlCapture, Subject, Subjects) {
    let (mut b, d) = wide_builder_without_registries();
    let root = element(&mut b, &d, "table", &[]);
    let mut subjects = Subjects::default();
    let mut children = Vec::new();
    let (mut before, mut after) = (Vec::new(), Vec::new());
    for child in table {
        match child {
            Child::Outside { id, after: later } => {
                let e = element(&mut b, &d, "div", &[("id", id)]);
                b.establish_children(&e, &[]).unwrap();
                if *later {
                    after.push(e)
                } else {
                    before.push(e)
                }
            }
            Child::Caption { id, .. } => {
                let attrs: Vec<(&str, &str)> =
                    id.as_deref().map(|i| ("id", i)).into_iter().collect();
                let e = element(&mut b, &d, "caption", &attrs);
                b.establish_children(&e, &[]).unwrap();
                subjects.captions.push(e.clone());
                children.push(e);
            }
            Child::Colgroup { span, cols, .. } => {
                let g = element(&mut b, &d, "colgroup", &attributes(&[("span", span)]));
                let mut kids = Vec::new();
                for col in cols {
                    let e = element(&mut b, &d, "col", &attributes(&[("span", &col.span)]));
                    b.establish_children(&e, &[]).unwrap();
                    subjects.cols.push(e.clone());
                    kids.push(e);
                }
                b.establish_children(&g, &kids).unwrap();
                subjects.colgroups.push(g.clone());
                children.push(g);
            }
            Child::Row(row) => children.push(build_row(&mut b, &d, row, &mut subjects)),
            Child::Section { kind, rows, .. } => {
                let g = element(&mut b, &d, kind, &[]);
                let trs: Vec<_> = rows
                    .iter()
                    .map(|row| build_row(&mut b, &d, row, &mut subjects))
                    .collect();
                b.establish_children(&g, &trs).unwrap();
                subjects.sections.push(g.clone());
                children.push(g);
            }
            Child::Other => {
                let e = element(&mut b, &d, "div", &[]);
                b.establish_children(&e, &[]).unwrap();
                children.push(e);
            }
        }
    }
    b.establish_children(&root, &children).unwrap();
    // The document lists the table, so an id search can complete.
    before.push(root.clone());
    before.extend(after);
    b.establish_children(&d, &before).unwrap();
    (b.finish().unwrap(), root, subjects)
}

fn read(capture: &HtmlCapture, subjects: &Subjects, model: &HtmlTableModel) -> Expected {
    let position = |list: &[Subject], subject: &HtmlCaptureSubjectId| {
        list.iter()
            .position(|s| &capture.subject_id(s).unwrap() == subject)
            .unwrap()
    };
    let cell = |s: &HtmlCaptureSubjectId| position(&subjects.cells, s);
    let mut errors: Vec<String> = model
        .errors()
        .iter()
        .map(|e| match e {
            HtmlTableModelError::Overlap {
                cell: c,
                other,
                x,
                y,
            } => {
                format!("overlap {} {} {x},{y}", cell(c), cell(other))
            }
            HtmlTableModelError::UnanchoredRow { y } => format!("row {y}"),
            HtmlTableModelError::UnanchoredColumn { x } => format!("column {x}"),
        })
        .collect();
    errors.sort();
    let mut cells: Vec<CellFacts> = model
        .cells()
        .iter()
        .map(|c| {
            let header = c.kind() == HtmlTableCellKind::Header;
            (
                cell(c.subject()),
                header,
                c.x(),
                c.y(),
                c.width(),
                c.height(),
            )
        })
        .collect();
    cells.sort_unstable();
    Expected {
        width: model.width(),
        height: model.height(),
        caption: model.caption().map(|c| position(&subjects.captions, c)),
        cells,
        row_sources: model
            .rows()
            .iter()
            .map(|r| r.source().map(|s| position(&subjects.rows, s)))
            .collect(),
        column_sources: model
            .columns()
            .iter()
            .map(|c| c.source().map(|s| position(&subjects.cols, s)))
            .collect(),
        row_groups: model
            .row_groups()
            .iter()
            .map(|g| (position(&subjects.sections, g.subject()), g.y(), g.height()))
            .collect(),
        column_groups: model
            .column_groups()
            .iter()
            .map(|g| (position(&subjects.colgroups, g.subject()), g.x(), g.width()))
            .collect(),
        errors,
    }
}

/// The model's header lists by cell, each in the order the model gives it.
fn read_headers(
    subjects: &Subjects,
    capture: &HtmlCapture,
    model: &HtmlTableModel,
) -> BTreeMap<usize, Vec<usize>> {
    let position = |subject: &HtmlCaptureSubjectId| {
        subjects
            .cells
            .iter()
            .position(|s| &capture.subject_id(s).unwrap() == subject)
            .unwrap()
    };
    let mut lists: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for header in model.headers() {
        lists
            .entry(position(header.cell()))
            .or_default()
            .push(position(header.header()));
    }
    lists
}

/// Whether a cell's list is the specification's: the ordered part first, in
/// order, then the group-added cells in any order.
fn agrees(actual: &[usize], expected: &Associated) -> bool {
    let split = expected.ordered.len();
    let mut grouped = actual.get(split..).unwrap_or_default().to_vec();
    grouped.sort_unstable();
    actual.get(..split) == Some(expected.ordered.as_slice()) && grouped == expected.grouped
}

#[test]
fn seeded_random_tables_form_and_assign_headers_as_the_specification_says() {
    let mut rng = Rng(0x1733_0409);
    for case in 0..400 {
        let description = table(&mut rng, case % 3 == 0);
        let (capture, root, subjects) = build(&description);
        let report = query(&capture, &root, limits()).0.unwrap();
        let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) =
            report.results()[0].fact()
        else {
            panic!(
                "case {case}: expected a complete model, got {:?}: {description:#?}",
                report.results()[0].fact()
            );
        };
        let (expected, associated) = oracle(&description);
        assert_eq!(
            read(&capture, &subjects, model),
            expected,
            "case {case}: {description:#?}"
        );
        let actual = read_headers(&subjects, &capture, model);
        let none = Vec::new();
        for (cell, expected) in &associated {
            let found = actual.get(cell).unwrap_or(&none);
            assert!(
                agrees(found, expected),
                "case {case}: cell {cell} has headers {found:?}, expected {expected:?}: {description:#?}"
            );
        }
        assert!(
            actual.keys().all(|cell| associated.contains_key(cell)),
            "case {case}: headers for a cell that is not in the table"
        );
    }
}
