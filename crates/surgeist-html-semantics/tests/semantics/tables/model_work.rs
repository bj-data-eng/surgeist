//! Work a table model query admits grows with the number of cells.
//!
//! HTML 877d "forming a table" (Processing model, the table model) and the
//! "algorithm for assigning header cells" answer questions about slots: is a
//! slot covered, which cell covers it, which headers lie in the row to its left
//! and the column above it. A table of `c` cells therefore needs work that is
//! proportional to `c`, not to `c` times the cells the query can reach, because
//! every slot question concerns one row or one column. The expectations are
//! derived from the table shape alone: an `n` by `n` grid has `n * n` cells,
//! and quadrupling the cells (doubling `n`) may grow the admitted work by about
//! four, never by the 23 to 30 a per-slot scan of every cell costs.

use super::scope_work::{cell, element, query};
use crate::semantics::support::{builder_with, wide_builder_without_registries};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilderSubject, HtmlCaptureLimits, HtmlSemanticFact,
    HtmlSemanticLimits, HtmlSemanticValue,
};

/// Ample for every size below, so the work measured is the work needed, not a
/// cap: the table model is retained as nested records and scanned in scratch.
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        usize::MAX / 4,
        100,
        10_000_000,
        10000,
        usize::MAX / 4,
        10_000_000,
    )
}

/// An `n` by `n` table. With `headers`, the first row and the first column are
/// th cells without a `scope`; every other cell is td. Each cell holds one
/// unit of text. The rows are direct children of the table.
fn grid(n: usize, headers: bool, grouped: bool) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    // Large grids hold more subjects than the shared table fixture admits.
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        1_000_000,
        10_000_000,
        1_000_000,
        100_000_000,
        1000,
        0,
        0,
        16_777_216,
    ));

    let table = element(&mut b, &d, "table", &[]);
    let mut rows = Vec::new();
    for y in 0..n {
        let mut cells = Vec::new();
        for x in 0..n {
            let kind = if headers && (x == 0 || y == 0) {
                "th"
            } else {
                "td"
            };
            cells.push(cell(&mut b, &d, kind, &[]));
        }
        let row = element(&mut b, &d, "tr", &[]);
        b.establish_children(&row, &cells).unwrap();
        rows.push(row);
    }
    if grouped {
        let body = element(&mut b, &d, "tbody", &[]);
        b.establish_children(&body, &rows).unwrap();
        b.establish_children(&table, &[body]).unwrap();
    } else {
        b.establish_children(&table, &rows).unwrap();
    }
    (b.finish().unwrap(), table)
}

/// Work the model query admitted, after checking that it formed the whole grid
/// and found the header associations the algorithm defines for it.
fn work(n: usize, headers: bool, grouped: bool) -> usize {
    let (capture, table) = grid(n, headers, grouped);
    let report = query(&capture, &table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    assert_eq!(
        (model.width(), model.height(), model.cells().len()),
        (n, n, n * n)
    );
    // Header assignment for the grid, from the algorithm's rules. A td off the
    // first row and column has two headers: the th that starts its column (the
    // first row holds no td, so those th are column headers) and the th that
    // starts its row (the first column holds only th, so below the first row
    // its th are row headers). A th in the first column below the corner scans
    // up a block of th and keeps only the corner, the one column header among
    // them. Nothing else has a header: a th in the first row has only th of
    // the column-header kind to its left, which a row scan rejects. Without th
    // there is nothing to associate.
    let expected = if headers {
        2 * (n - 1) * (n - 1) + (n - 1)
    } else {
        0
    };
    assert_eq!(model.headers().len(), expected, "n={n} headers={headers}");
    report.usage().work()
}

/// Work for 5x5, 10x10, 20x20 and 40x40 grids: three quadruplings of the cells.
fn assert_work_grows_about_four_times_per_four_times_the_cells(headers: bool, grouped: bool) {
    let sizes: &[usize] = if grouped {
        &[25, 50, 100, 200]
    } else {
        &[5, 10, 20, 40]
    };
    let measured: Vec<usize> = sizes.iter().map(|n| work(*n, headers, grouped)).collect();
    for (pair, sizes) in measured.windows(2).zip(sizes.windows(2)) {
        // Four times the cells, linear work, plus a small allowance for the
        // logarithmic slot lookups and the fixed per-table charges.
        assert!(
            pair[1] <= pair[0] * 5,
            "headers={headers} grouped={grouped}: work for {0}x{0} is {1} but {2} for {3}x{3} \
             (cells x4 may cost at most x5)",
            sizes[0],
            pair[0],
            pair[1],
            sizes[1],
        );
    }
}

#[test]
fn data_grid_work_grows_about_four_times_per_four_times_the_cells() {
    assert_work_grows_about_four_times_per_four_times_the_cells(false, false);
}

#[test]
fn header_row_and_column_grid_work_grows_about_four_times_per_four_times_the_cells() {
    assert_work_grows_about_four_times_per_four_times_the_cells(true, false);
}

#[test]
fn header_row_and_column_grid_in_a_tbody_grows_about_four_times_per_four_times_the_cells() {
    // A parser puts every row in a tbody; no header cell scopes a group.
    assert_work_grows_about_four_times_per_four_times_the_cells(true, true);
}

/// The live tall cells of a table are not revisited once per row, and a cell
/// placed beside them does not move them.
///
/// `tall` cells of `rows + 1` rows start the table in row 0. `rows` rows
/// follow, each with one `td`. Placing the cells of a row needs only the live cells that sit at or
/// after the new cell's column, and each tall cell is live for the same rows,
/// so the work of the query is that of its input, `tall + rows` elements, not
/// `tall * rows`.
#[derive(Clone, Copy)]
enum Later {
    /// Every later row holds one `td`, which lands right of the tall cells.
    Right,
    /// Row 0 starts with one short `td`, every later row holds one `td` that
    /// lands in column 0, left of every tall cell.
    Left,
    /// As `Right`, but the tall cells have `rowspan="0"`: they grow
    /// downward one row at a time to the end of the rows.
    GrowingRight,
}

fn tall_and_later_rows(
    tall: usize,
    rows: usize,
    later: Later,
) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    let (mut b, d) = wide_builder_without_registries();
    let table = element(&mut b, &d, "table", &[]);
    let span = if matches!(later, Later::GrowingRight) {
        "0".to_string()
    } else {
        (rows + 1).to_string()
    };
    let mut first = Vec::new();
    if matches!(later, Later::Left) {
        first.push(cell(&mut b, &d, "td", &[]));
    }
    for _ in 0..tall {
        first.push(cell(&mut b, &d, "td", &[("rowspan", &span)]));
    }
    let mut trs = Vec::new();
    let row = element(&mut b, &d, "tr", &[]);
    b.establish_children(&row, &first).unwrap();
    trs.push(row);
    for _ in 0..rows {
        let row = element(&mut b, &d, "tr", &[]);
        let td = cell(&mut b, &d, "td", &[]);
        b.establish_children(&row, &[td]).unwrap();
        trs.push(row);
    }
    b.establish_children(&table, &trs).unwrap();
    (b.finish().unwrap(), table)
}

/// Work of the query after checking the model's size and that it has no
/// recovered errors: no cell overlaps and every row holds a cell of its own.
fn tall_work(tall: usize, rows: usize, later: Later) -> usize {
    let (capture, table) = tall_and_later_rows(tall, rows, later);
    let report = query(&capture, &table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    let (width, cells) = match later {
        Later::Right | Later::GrowingRight => (tall + 1, tall + rows),
        Later::Left => (tall + 1, 1 + tall + rows),
    };
    assert_eq!(
        (model.width(), model.height(), model.cells().len()),
        (width, rows + 1, cells)
    );
    assert!(model.errors().is_empty());
    assert!(model.headers().is_empty());
    // The tall cells reach the last row, either by their rowspan or by growing.
    let tall_cells = if matches!(later, Later::Left) {
        &model.cells()[1..=tall]
    } else {
        &model.cells()[..tall]
    };
    assert!(tall_cells.iter().all(|cell| cell.height() == rows + 1));
    report.usage().work()
}

fn assert_input_linear_work(later: Later) {
    // Doubling `tall` and `rows` doubles the input: allow a little over two.
    let sizes = [50, 100, 200, 400];
    let measured: Vec<usize> = sizes.iter().map(|n| tall_work(*n, *n, later)).collect();
    for (pair, sizes) in measured.windows(2).zip(sizes.windows(2)) {
        assert!(
            pair[1] * 10 <= pair[0] * 26,
            "{0} tall cells and {0} rows cost {1} but {2} for {3} of each \
             (input x2 may cost at most x2.6)",
            sizes[0],
            pair[0],
            pair[1],
            sizes[1],
        );
    }
}

#[test]
fn tall_cells_beside_rows_placed_to_their_right_cost_the_size_of_the_input() {
    assert_input_linear_work(Later::Right);
}

#[test]
fn tall_cells_beside_rows_placed_to_their_left_cost_the_size_of_the_input() {
    assert_input_linear_work(Later::Left);
}

#[test]
fn growing_cells_beside_rows_cost_the_size_of_the_input() {
    assert_input_linear_work(Later::GrowingRight);
}

/// Tall header cells with data cells between them are not scanned once per row
/// they cross, and a row's own cell at their right does not copy them.
///
/// `tall` cells of `rows + 1` rows start the table in row 0, alternately `th`
/// and `td`, and each of the `rows` rows after holds one `td`, which lands right
/// of them. A cell scans the rows it crosses leftward for headers (HTML 877d
/// internal algorithm for scanning and assigning header cells), but every row
/// it crosses holds the same cells to its left, so its scans find the same
/// headers: the work is that of the input, not `tall * rows`.
///
/// By the algorithm: a tall `td` (odd position `k`) meets the `th` before it,
/// a row header since no data cell shares its column, and then a `td` that
/// closes the block, so it takes that `th` alone; a tall `th` meets a `td`
/// first, which closes its own block, and the `th` beyond is hidden by it. The
/// `td` of a later row meets the last two cells: a `td`, then the `th` before
/// it, so it takes the `th` at position `tall - 2`.
fn tall_headers_work(tall: usize, rows: usize) -> usize {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        1_000_000,
        10_000_000,
        1_000_000,
        100_000_000,
        1000,
        0,
        0,
        16_777_216,
    ));
    let table = element(&mut b, &d, "table", &[]);
    let span = (rows + 1).to_string();
    let first: Vec<_> = (0..tall)
        .map(|k| {
            let kind = if k % 2 == 0 { "th" } else { "td" };
            cell(&mut b, &d, kind, &[("rowspan", &span)])
        })
        .collect();
    let mut trs = Vec::new();
    let row = element(&mut b, &d, "tr", &[]);
    b.establish_children(&row, &first).unwrap();
    trs.push(row);
    for _ in 0..rows {
        let row = element(&mut b, &d, "tr", &[]);
        let td = cell(&mut b, &d, "td", &[]);
        b.establish_children(&row, &[td]).unwrap();
        trs.push(row);
    }
    b.establish_children(&table, &trs).unwrap();
    let capture = b.finish().unwrap();
    let report = query(&capture, &table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    assert_eq!(
        (model.width(), model.height(), model.cells().len()),
        (tall + 1, rows + 1, tall + rows)
    );
    assert!(model.errors().is_empty());
    assert_eq!(model.headers().len(), tall / 2 + rows);
    report.usage().work()
}

#[test]
fn tall_header_cells_beside_rows_cost_the_size_of_the_input() {
    let sizes = [50, 100, 200, 400];
    let measured: Vec<usize> = sizes.iter().map(|n| tall_headers_work(*n, *n)).collect();
    for (pair, sizes) in measured.windows(2).zip(sizes.windows(2)) {
        assert!(
            pair[1] * 10 <= pair[0] * 26,
            "{0} tall cells and {0} rows cost {1} but {2} for {3} of each \
             (input x2 may cost at most x2.6)",
            sizes[0],
            pair[0],
            pair[1],
            sizes[1],
        );
    }
}

// ---- Work follows the input and the associations it must produce -----------

/// A model query over a table the closure builds, with the model's checks.
fn associated_work(
    build: impl FnOnce(
        &mut surgeist_html_semantics::HtmlCaptureBuilder,
        &HtmlCaptureBuilderSubject,
    ) -> HtmlCaptureBuilderSubject,
    size: (usize, usize),
    cells: usize,
    associations: usize,
) -> usize {
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        1_000_000,
        10_000_000,
        1_000_000,
        100_000_000,
        1000,
        0,
        0,
        16_777_216,
    ));
    let table = build(&mut b, &d);
    let capture = b.finish().unwrap();
    let report = query(&capture, &table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    assert_eq!(
        (model.width(), model.height(), model.cells().len()),
        (size.0, size.1, cells)
    );
    assert!(model.errors().is_empty());
    assert_eq!(model.headers().len(), associations);
    report.usage().work()
}

/// Work divided by the input and the associations may grow only a little when
/// the input doubles: the work is `O((input + associations) log input)`, not
/// the product of the headers a scan meets and the headers it has already seen.
/// `measured` holds (input, associations, work) for each size.
fn assert_work_follows_input_and_associations(label: &str, measured: &[(usize, usize, usize)]) {
    for pair in measured.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        let (before_units, after_units) = (before.0 + before.1, after.0 + after.1);
        // work / units grows by at most 1.25 per doubling.
        assert!(
            u128::try_from(after.2).unwrap() * u128::try_from(before_units).unwrap() * 100
                <= u128::try_from(before.2).unwrap() * u128::try_from(after_units).unwrap() * 125,
            "{label}: {} units cost {} but {} units cost {} \
             (work per unit may grow at most x1.25 per doubling)",
            before_units,
            before.2,
            after_units,
            after.2,
        );
    }
}

/// `n` tall cells (n even), alternately `th scope=row` and `td`, the cell at
/// position `k` spanning `n + 1 - k` rows, so that every row end differs; `n`
/// rows after, each with one `td`. Row `r` has the tall cells `k < n + 1 - r`
/// still under it, so its `td` takes column `n + 1 - r`.
///
/// By the scanning algorithm every cell takes the row-header `th` cells before
/// it, since no two of them cross the same rows and so none hides another.
/// Tall cell `k` takes the `th` at the even positions below `k`: `(k + 1) / 2`
/// for a `td`, `k / 2` for a `th`, `n * n / 4` in all. The `td` of a later row
/// at column `x` takes the `th` at the even positions below `x`, `(x + 1) / 2`
/// each, for `x` from 1 to `n`: `n / 2 * (n / 2 + 1)`.
fn staggered_work(n: usize) -> (usize, usize, usize) {
    let associations = n * n / 4 + n / 2 * (n / 2 + 1);
    let work = associated_work(
        |b, d| {
            let table = element(b, d, "table", &[]);
            let first: Vec<_> = (0..n)
                .map(|k| {
                    let span = (n + 1 - k).to_string();
                    if k % 2 == 0 {
                        cell(b, d, "th", &[("scope", "row"), ("rowspan", &span)])
                    } else {
                        cell(b, d, "td", &[("rowspan", &span)])
                    }
                })
                .collect();
            let mut trs = Vec::new();
            let row = element(b, d, "tr", &[]);
            b.establish_children(&row, &first).unwrap();
            trs.push(row);
            for _ in 0..n {
                let row = element(b, d, "tr", &[]);
                let td = cell(b, d, "td", &[]);
                b.establish_children(&row, &[td]).unwrap();
                trs.push(row);
            }
            b.establish_children(&table, &trs).unwrap();
            table
        },
        (n + 1, n + 1),
        2 * n,
        associations,
    );
    (2 * n, associations, work)
}

#[test]
fn staggered_tall_headers_cost_the_input_and_the_associations() {
    let measured: Vec<_> = [50, 100, 200, 400]
        .into_iter()
        .map(staggered_work)
        .collect();
    assert_work_follows_input_and_associations("staggered", &measured);
}

/// `n` rows of one `th` each. A `th` in the column takes every `th` above it:
/// no data cell shares a row with one, so each is a column header, and no data
/// cell closes a block, so none is hidden: `n * (n - 1) / 2` associations.
fn stacked_column_work(n: usize) -> (usize, usize, usize) {
    let associations = n * (n - 1) / 2;
    let work = associated_work(
        |b, d| {
            let table = element(b, d, "table", &[]);
            let trs: Vec<_> = (0..n)
                .map(|_| {
                    let row = element(b, d, "tr", &[]);
                    let th = cell(b, d, "th", &[]);
                    b.establish_children(&row, &[th]).unwrap();
                    row
                })
                .collect();
            b.establish_children(&table, &trs).unwrap();
            table
        },
        (1, n),
        n,
        associations,
    );
    (n, associations, work)
}

#[test]
fn a_stacked_header_column_costs_the_input_and_the_associations() {
    let measured: Vec<_> = [50, 100, 200, 400]
        .into_iter()
        .map(stacked_column_work)
        .collect();
    assert_work_follows_input_and_associations("stacked column", &measured);
}

/// A cell of one row among the tall header cells that cross all the rows does
/// not make the rows be scanned one by one.
///
/// HTML 877d, internal algorithm for scanning and assigning header cells,
/// steps 4 and 9. Row 0 holds a tall `th` (`rowspan="n"`, `scope="row"`), a
/// `td`, then `n` more tall `th` of the same rows, then a tall `td`; each of
/// the `n - 1` rows after holds one `td`, which lands in column 1 beside the
/// first `th`. So every row has a cell of its own between the tall cells, and
/// all the `th` cross the same rows. Every cell but the last carries
/// `headers=""`, which assigns it no header and keeps it from scanning.
///
/// The tall `td` is the one principal cell that scans. Each of its `n` rows is
/// read leftward: the `n` later `th` meet no data cell, so they share one
/// header block, none is opaque, and each is a row header by its `scope`: all
/// are taken. Then the row's `td` ends that block, which makes those `th`
/// opaque, and the first `th`, of the same rows, is hidden by them. A row's
/// own cell therefore decides one header, and the same one in every row. The
/// `n` rows give the same headers, so the tall `td` takes the `n` later `th`
/// once: `n` associations in all, and the first `th` is not among them.
fn among_work(n: usize) -> (usize, usize, usize) {
    let span = n.to_string();
    let work = associated_work(
        |b, d| {
            let table = element(b, d, "table", &[]);
            let tall = |b: &mut surgeist_html_semantics::HtmlCaptureBuilder| {
                cell(
                    b,
                    d,
                    "th",
                    &[("scope", "row"), ("rowspan", &span), ("headers", "")],
                )
            };
            let mut first = vec![tall(b), cell(b, d, "td", &[("headers", "")])];
            for _ in 0..n {
                first.push(tall(b));
            }
            first.push(cell(b, d, "td", &[("rowspan", &span)]));
            let mut trs = Vec::new();
            let row = element(b, d, "tr", &[]);
            b.establish_children(&row, &first).unwrap();
            trs.push(row);
            for _ in 1..n {
                let row = element(b, d, "tr", &[]);
                let td = cell(b, d, "td", &[("headers", "")]);
                b.establish_children(&row, &[td]).unwrap();
                trs.push(row);
            }
            b.establish_children(&table, &trs).unwrap();
            table
        },
        (n + 3, n),
        2 * n + 2,
        n,
    );
    (2 * n + 2, n, work)
}

#[test]
fn cells_among_tall_header_cells_cost_the_input_and_the_associations() {
    let measured: Vec<_> = [100, 200, 400, 800].into_iter().map(among_work).collect();
    // The input and the associations double with `n`: allow a little over
    // two for the logarithms of the lookups, as the tests above do.
    for pair in measured.windows(2) {
        assert!(
            pair[1].2 * 10 <= pair[0].2 * 26,
            "{} cells and {} associations cost {} but {} and {} cost {} \
             (input and associations x2 may cost at most x2.6)",
            pair[0].0,
            pair[0].1,
            pair[0].2,
            pair[1].0,
            pair[1].1,
            pair[1].2,
        );
    }
}
