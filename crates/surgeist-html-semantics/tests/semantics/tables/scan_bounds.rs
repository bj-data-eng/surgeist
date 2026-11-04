//! Work of the header scans on the shapes that are costly for them.
//!
//! A cell scans the rows it crosses leftward and the columns it crosses upward
//! (HTML 877d internal algorithm for scanning and assigning header cells), so a
//! table of `C` cells with `W` columns and `H` rows needs at most work of order
//! `C * (W + H)`, up to logarithmic factors, however the cells span. Each shape
//! below is built to cost more than that, or to cost it all, when a scan is
//! repeated for rows that need not be scanned again, when a group test looks at
//! cells that cannot belong to it, or when overlaps are searched linearly. The
//! counts are derived by hand from the algorithm for the shape.

use super::scope_work::{cell, element, query};
use crate::semantics::support::builder_with;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits, HtmlSemanticFact,
    HtmlSemanticLimits, HtmlSemanticValue,
};

type Subject = HtmlCaptureBuilderSubject;

/// What a model query found and what it cost.
struct Measured {
    cells: usize,
    width: usize,
    height: usize,
    errors: usize,
    headers: usize,
    work: usize,
}

impl Measured {
    /// `C * (W + H)`, the bound of a header scan's slot-by-slot cost.
    fn bound(&self) -> usize {
        self.cells * (self.width + self.height)
    }
}

struct Builder {
    b: HtmlCaptureBuilder,
    d: Subject,
}

impl Builder {
    fn new() -> Self {
        let (b, d) = builder_with(HtmlCaptureLimits::new(
            1_000_000,
            10_000_000,
            1_000_000,
            100_000_000,
            1000,
            0,
            0,
            16_777_216,
        ));
        Self { b, d }
    }

    fn cell(&mut self, tag: &str, attrs: &[(&str, &str)]) -> Subject {
        cell(&mut self.b, &self.d, tag, attrs)
    }

    fn element(&mut self, tag: &str, kids: &[Subject]) -> Subject {
        let e = element(&mut self.b, &self.d, tag, &[]);
        self.b.establish_children(&e, kids).unwrap();
        e
    }

    fn row(&mut self, cells: &[Subject]) -> Subject {
        self.element("tr", cells)
    }

    fn table(&mut self, kids: &[Subject]) -> Subject {
        self.element("table", kids)
    }
}

fn measure(build: impl FnOnce(&mut Builder) -> Subject) -> Measured {
    let mut b = Builder::new();
    let table = build(&mut b);
    let capture = b.b.finish().unwrap();
    let limits =
        HtmlSemanticLimits::new(usize::MAX / 4, 100, 1 << 40, 10000, usize::MAX / 4, 1 << 40);
    let (result, _) = query(&capture, &table, limits);
    let report = result.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    Measured {
        cells: model.cells().len(),
        width: model.width(),
        height: model.height(),
        errors: model.errors().len(),
        headers: model.headers().len(),
        work: report.usage().work(),
    }
}

/// The work of each size must grow by at most `numerator / 100` when the
/// size doubles. Sizes are measured in turn and the first pair that fails stops
/// the test, so a shape that is far too costly is not measured at its largest.
fn assert_work_doubles_by_at_most(
    label: &str,
    numerator: usize,
    sizes: [usize; 4],
    work: impl Fn(usize) -> usize,
) {
    let mut before: Option<(usize, usize)> = None;
    for n in sizes {
        let after = (n, work(n));
        if let Some(before) = before {
            assert!(
                after.1 * 100 <= before.1 * numerator,
                "{label}: n = {} costs {} but n = {} costs {} (at most x{:.2} per doubling)",
                before.0,
                before.1,
                after.0,
                after.1,
                numerator as f64 / 100.0
            );
        }
        before = Some(after);
    }
}

/// The work per `C * (W + H)` must grow by at most `numerator / 100` when the
/// size doubles: the shape costs no more than the bound, up to that factor.
fn assert_work_per_bound_grows_by_at_most(
    label: &str,
    numerator: usize,
    shape: impl Fn(usize) -> Measured,
) {
    let mut before: Option<Measured> = None;
    for n in SIZES {
        let after = shape(n);
        if let Some(before) = &before {
            assert!(
                u128::try_from(after.work).unwrap() * u128::try_from(before.bound()).unwrap() * 100
                    <= u128::try_from(before.work).unwrap()
                        * u128::try_from(after.bound()).unwrap()
                        * u128::try_from(numerator).unwrap(),
                "{label}: work {} for bound {} then {} for {} \
                 (per bound at most x{:.2} per doubling)",
                before.work,
                before.bound(),
                after.work,
                after.bound(),
                numerator as f64 / 100.0
            );
        }
        before = Some(after);
    }
}

const SIZES: [usize; 4] = [100, 200, 400, 800];
/// For shapes whose cost over the bound is a few units for each pair of cells,
/// which a table of a few hundred cells hides behind the cost of its cells.
const LARGE: [usize; 4] = [400, 800, 1600, 3200];

/// B1. Row 0 is a `td` and `n` tall `th` (scope row, `n + 1` rows); `n` rows
/// follow with one `td` at column 0, left of every tall cell. Every row crossed
/// by a tall cell has a cell of its own before it, but they are all data cells
/// at the same slot, so each of those rows scans the same way. A tall `th` at
/// column `k` meets the `k - 1` tall `th` before it, no data cell between them
/// to hide any, so it takes all of them: `n * (n - 1) / 2` in all.
fn per_line(n: usize) -> Measured {
    let m = measure(|b| {
        let span = (n + 1).to_string();
        let mut first = vec![b.cell("td", &[])];
        for _ in 0..n {
            first.push(b.cell("th", &[("scope", "row"), ("rowspan", &span)]));
        }
        let mut rows = vec![b.row(&first)];
        for _ in 0..n {
            let td = b.cell("td", &[]);
            rows.push(b.row(&[td]));
        }
        b.table(&rows)
    });
    assert_eq!(
        (m.cells, m.width, m.height, m.errors),
        (2 * n + 1, n + 1, n + 1, 0)
    );
    assert_eq!(m.headers, n * (n - 1) / 2);
    m
}

#[test]
fn rows_with_equal_cells_before_a_tall_cell_share_one_scan() {
    // Quadratic in the size, as the associations are: x4 per doubling, with
    // the logarithms of the sorts allowed for.
    assert_work_doubles_by_at_most("per line", 480, SIZES, |n| per_line(n).work);
}

/// B3. `n` times: a row of a cell and a `td` of two rows, a row with a `td` of
/// two columns that overlaps it, and a row of one `td`. The first cell is a
/// `th`, which no cell takes (the rows of the table all hold data cells, so it
/// is no column header, and the column holds data cells, so it is no row
/// header). The `n` overlaps are separate, and every `td` of two rows crosses
/// one of them.
fn irregular_extents(n: usize) -> Measured {
    let m = measure(|b| {
        let mut rows = Vec::new();
        for i in 0..n {
            let a = b.cell(if i == 0 { "th" } else { "td" }, &[]);
            let tall = b.cell("td", &[("rowspan", "2")]);
            rows.push(b.row(&[a, tall]));
            let wide = b.cell("td", &[("colspan", "2")]);
            rows.push(b.row(&[wide]));
            let last = b.cell("td", &[]);
            rows.push(b.row(&[last]));
        }
        b.table(&rows)
    });
    assert_eq!((m.cells, m.width, m.height), (4 * n, 2, 3 * n));
    assert_eq!((m.errors, m.headers), (n, 0));
    m
}

#[test]
fn separate_overlaps_are_found_without_visiting_the_others() {
    assert_work_doubles_by_at_most("irregular extents", 260, SIZES, |n| {
        irregular_extents(n).work
    });
}

/// B5. A `td` and `n` `th` of scope `rowgroup`, which take no header of their
/// own (`headers=""`), in the first row of a `tbody`, then `n` rows with one
/// `td` each. A `td` is in the row group, but every `th` lies right of its
/// first column, so none is a row group header it takes.
fn rowgroup_wide(n: usize) -> Measured {
    let m = measure(|b| {
        let mut first = vec![b.cell("td", &[])];
        for _ in 0..n {
            first.push(b.cell("th", &[("scope", "rowgroup"), ("headers", "")]));
        }
        let mut rows = vec![b.row(&first)];
        for _ in 0..n {
            let td = b.cell("td", &[]);
            rows.push(b.row(&[td]));
        }
        let body = b.element("tbody", &rows);
        b.table(&[body])
    });
    assert_eq!(
        (m.cells, m.width, m.height, m.errors, m.headers),
        (2 * n + 1, n + 1, n + 1, 0, 0)
    );
    m
}

#[test]
fn row_group_headers_right_of_a_cell_are_not_tested_by_it() {
    assert_work_doubles_by_at_most("row group", 240, LARGE, |n| rowgroup_wide(n).work);
}

/// B5. `n` column groups of one column each, a first row of `n` `th` of scope
/// `colgroup` that take no header of their own, then `n` rows with one `td` at
/// column 0. A `td` takes the one `th` in its column group: `n` in all.
fn colgroups_wide(n: usize) -> Measured {
    let m = measure(|b| {
        let mut kids = Vec::new();
        for _ in 0..n {
            let col = b.element("col", &[]);
            kids.push(b.element("colgroup", &[col]));
        }
        let first: Vec<_> = (0..n)
            .map(|_| b.cell("th", &[("scope", "colgroup"), ("headers", "")]))
            .collect();
        kids.push(b.row(&first));
        for _ in 0..n {
            let td = b.cell("td", &[]);
            kids.push(b.row(&[td]));
        }
        b.table(&kids)
    });
    assert_eq!(
        (m.cells, m.width, m.height, m.errors, m.headers),
        (2 * n, n, n + 1, 0, n)
    );
    m
}

#[test]
fn column_group_headers_outside_a_cells_group_are_not_tested_by_it() {
    assert_work_doubles_by_at_most("column group", 240, LARGE, |n| colgroups_wide(n).work);
}

/// B2. `n` `th` of scope row, alternately of one row and of two, none with a
/// header of its own (`headers=""`), a `td`, a `th` of two rows, a `td` and a
/// `th`, then `n` `td` in the same row. Each of the last `n` cells meets the
/// headers one by one: the ones that cross different rows are not hidden by
/// each other, so each is visited.
fn alternating(n: usize) -> Measured {
    measure(|b| {
        let mut cells = Vec::new();
        for i in 0..n {
            cells.push(if i % 2 == 0 {
                b.cell("th", &[("scope", "row"), ("headers", "")])
            } else {
                b.cell("th", &[("scope", "row"), ("rowspan", "2"), ("headers", "")])
            });
        }
        cells.push(b.cell("td", &[("headers", "")]));
        cells.push(b.cell("th", &[("scope", "row"), ("rowspan", "2"), ("headers", "")]));
        cells.push(b.cell("td", &[("headers", "")]));
        cells.push(b.cell("th", &[("scope", "row"), ("headers", "")]));
        for _ in 0..n {
            cells.push(b.cell("td", &[]));
        }
        let row = b.row(&cells);
        b.table(&[row])
    })
}

#[test]
fn hidden_headers_of_alternating_extents_cost_no_more_than_the_bound() {
    assert_work_per_bound_grows_by_at_most("alternating", 125, alternating);
}

/// B4. A `th`, `n` `td` and a `td` of `n + 1` rows in the first row; then `n`
/// rows each with one `td` of `n + 2` columns that overlaps the tall cell. The
/// tall cell scans each of those rows slot by slot.
fn irregular_walk(n: usize) -> Measured {
    let m = measure(|b| {
        let span = (n + 1).to_string();
        let mut first = vec![b.cell("th", &[])];
        for _ in 0..n {
            first.push(b.cell("td", &[]));
        }
        first.push(b.cell("td", &[("rowspan", &span)]));
        let mut rows = vec![b.row(&first)];
        let wide = (n + 2).to_string();
        for _ in 0..n {
            let td = b.cell("td", &[("colspan", &wide)]);
            rows.push(b.row(&[td]));
        }
        b.table(&rows)
    });
    assert_eq!(
        (m.cells, m.width, m.height, m.errors),
        (2 * n + 2, n + 2, n + 1, n)
    );
    m
}

#[test]
fn rows_scanned_slot_by_slot_cost_no_more_than_the_bound() {
    assert_work_per_bound_grows_by_at_most("irregular walk", 125, irregular_walk);
}
