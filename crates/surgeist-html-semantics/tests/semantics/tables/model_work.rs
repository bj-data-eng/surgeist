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
use crate::semantics::support::wide_builder_without_registries;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilderSubject, HtmlSemanticFact, HtmlSemanticLimits, HtmlSemanticValue,
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
fn grid(n: usize, headers: bool) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    let (mut b, d) = wide_builder_without_registries();
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
    b.establish_children(&table, &rows).unwrap();
    (b.finish().unwrap(), table)
}

/// Work the model query admitted, after checking that it formed the whole grid
/// and found the header associations the algorithm defines for it.
fn work(n: usize, headers: bool) -> usize {
    let (capture, table) = grid(n, headers);
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
fn assert_work_grows_about_four_times_per_four_times_the_cells(headers: bool) {
    let sizes = [5, 10, 20, 40];
    let measured: Vec<usize> = sizes.iter().map(|n| work(*n, headers)).collect();
    for (pair, sizes) in measured.windows(2).zip(sizes.windows(2)) {
        // Four times the cells, linear work, plus a small allowance for the
        // logarithmic slot lookups and the fixed per-table charges.
        assert!(
            pair[1] <= pair[0] * 5,
            "headers={headers}: work for {0}x{0} is {1} but {2} for {3}x{3} \
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
    assert_work_grows_about_four_times_per_four_times_the_cells(false);
}

#[test]
fn header_row_and_column_grid_work_grows_about_four_times_per_four_times_the_cells() {
    assert_work_grows_about_four_times_per_four_times_the_cells(true);
}
