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
    RightOfTall,
    /// Row 0 starts with one short `td`, every later row holds one `td` that
    /// lands in column 0, left of every tall cell.
    LeftOfTall,
    /// As `RightOfTall`, but the tall cells have `rowspan="0"`: they grow
    /// downward one row at a time to the end of the rows.
    GrowingRightOfTall,
}

fn tall_and_later_rows(
    tall: usize,
    rows: usize,
    later: Later,
) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    let (mut b, d) = wide_builder_without_registries();
    let table = element(&mut b, &d, "table", &[]);
    let span = if matches!(later, Later::GrowingRightOfTall) {
        "0".to_string()
    } else {
        (rows + 1).to_string()
    };
    let mut first = Vec::new();
    if matches!(later, Later::LeftOfTall) {
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
        Later::RightOfTall | Later::GrowingRightOfTall => (tall + 1, tall + rows),
        Later::LeftOfTall => (tall + 1, 1 + tall + rows),
    };
    assert_eq!(
        (model.width(), model.height(), model.cells().len()),
        (width, rows + 1, cells)
    );
    assert!(model.errors().is_empty());
    assert!(model.headers().is_empty());
    // The tall cells reach the last row, either by their rowspan or by growing.
    let tall_cells = if matches!(later, Later::LeftOfTall) {
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
    assert_input_linear_work(Later::RightOfTall);
}

#[test]
fn tall_cells_beside_rows_placed_to_their_left_cost_the_size_of_the_input() {
    assert_input_linear_work(Later::LeftOfTall);
}

#[test]
fn growing_cells_beside_rows_cost_the_size_of_the_input() {
    assert_input_linear_work(Later::GrowingRightOfTall);
}
