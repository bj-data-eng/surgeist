//! Slot questions answered through the formation index agree with the HTML
//! 877d "forming a table" and "assigning header cells" algorithms.
//!
//! Each expectation is derived by hand from the algorithm text for the small
//! table drawn in the test: cells covering a slot, the scan that stops when the
//! slot has none or several, the opaque-header blocks and the order in which
//! headers are added to the header list.

use super::scope_work::{cell, element, limits, query};
use crate::semantics::support::wide_builder_without_registries;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilderSubject, HtmlCaptureSubjectId, HtmlSemanticFact,
    HtmlSemanticValue, HtmlTableModel, HtmlTableModelError,
};

type Subject = HtmlCaptureBuilderSubject;
/// A cell's tag and attributes.
type Spec<'a> = (&'a str, &'a [(&'a str, &'a str)]);

/// A table of direct rows; each row lists (tag, attributes) cells.
fn table(rows: &[&[Spec<'_>]]) -> (HtmlCapture, Subject, Vec<Vec<Subject>>) {
    let (mut b, d) = wide_builder_without_registries();
    let table = element(&mut b, &d, "table", &[]);
    let mut row_elements = Vec::new();
    let mut cells = Vec::new();
    for row in rows {
        let mut in_row = Vec::new();
        for (tag, attrs) in *row {
            in_row.push(cell(&mut b, &d, tag, attrs));
        }
        let tr = element(&mut b, &d, "tr", &[]);
        b.establish_children(&tr, &in_row).unwrap();
        row_elements.push(tr);
        cells.push(in_row);
    }
    b.establish_children(&table, &row_elements).unwrap();
    (b.finish().unwrap(), table, cells)
}

fn model_of(capture: &HtmlCapture, table: &Subject) -> HtmlTableModel {
    let report = query(capture, table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    model.clone()
}

/// The header associations as (cell, header) pairs of positions in `cells`
/// read row by row.
fn associations(
    capture: &HtmlCapture,
    model: &HtmlTableModel,
    cells: &[Vec<Subject>],
) -> Vec<(usize, usize)> {
    let ids: Vec<HtmlCaptureSubjectId> = cells
        .iter()
        .flatten()
        .map(|c| capture.subject_id(c).unwrap())
        .collect();
    let position = |id: &HtmlCaptureSubjectId| ids.iter().position(|other| other == id).unwrap();
    model
        .headers()
        .iter()
        .map(|edge| (position(edge.cell()), position(edge.header())))
        .collect()
}

#[test]
fn a_slot_covered_by_two_cells_ends_no_scan_and_belongs_to_neither() {
    // Row 0:  A(rowspan 2)  H(th scope=row, rowspan 2)  D  E(rowspan 2)
    // Row 1:  Q(colspan 2) is placed at x=2, the first slot no rowspan holds.
    // Q covers (2,1) and (3,1); E already covers (3,1): a table model error.
    // The slot (3,1) has two cells, so a scan that reaches it skips it.
    // D, E and Q each scan left to H, a row header, and stop after the data
    // cell A: their only header is H. E scans rows 0 and 1 and keeps H once.
    let (capture, table, cells) = table(&[
        &[
            ("td", &[("rowspan", "2")]),
            ("th", &[("scope", "row"), ("rowspan", "2")]),
            ("td", &[]),
            ("td", &[("rowspan", "2")]),
        ],
        &[("td", &[("colspan", "2")])],
    ]);
    let model = model_of(&capture, &table);
    assert_eq!((model.width(), model.height()), (4, 2));
    let [h, d, e, q] = [1, 2, 3, 4];
    assert_eq!(
        model.errors(),
        [HtmlTableModelError::Overlap {
            cell: capture.subject_id(&cells[1][0]).unwrap(),
            other: capture.subject_id(&cells[0][3]).unwrap(),
            x: 3,
            y: 1,
        }]
    );
    assert_eq!(
        associations(&capture, &model, &cells),
        [(d, h), (e, h), (q, h)]
    );
}

#[test]
fn overlap_errors_follow_model_order_of_the_cells_overlapped() {
    // Row 0:  A  B(rowspan 2)  C(rowspan 2)
    // Row 1:  F(colspan 3) starts at (0,1), which is free, and covers (1,1)
    //         and (2,1), held by B and then C.
    let (capture, table, cells) = table(&[
        &[
            ("td", &[]),
            ("td", &[("rowspan", "2")]),
            ("td", &[("rowspan", "2")]),
        ],
        &[("td", &[("colspan", "3")])],
    ]);
    let model = model_of(&capture, &table);
    let id = |row: usize, column: usize| capture.subject_id(&cells[row][column]).unwrap();
    assert_eq!(
        model.errors(),
        [
            HtmlTableModelError::Overlap {
                cell: id(1, 0),
                other: id(0, 1),
                x: 1,
                y: 1,
            },
            HtmlTableModelError::Overlap {
                cell: id(1, 0),
                other: id(0, 2),
                x: 2,
                y: 1,
            },
        ]
    );
}

#[test]
fn a_row_scan_adds_the_nearest_header_block_and_later_equal_extent_headers_are_opaque() {
    // One row:  H1  a  b  H2  c  d  P   (th are scope=row)
    // From P the scan meets d and c (data, no block open), then H2, a row
    // header, which is added. b is data and closes the block: H2 becomes
    // opaque. a is data. H1 has H2's row and height, so H2 hides it.
    // c and d see the same. a and b see H1 alone, nothing hiding it. A header
    // principal sees the headers to its left the same way: H2 scans through
    // data to H1 and is hidden by itself.
    let row: &[Spec<'_>] = &[
        ("th", &[("scope", "row")]),
        ("td", &[]),
        ("td", &[]),
        ("th", &[("scope", "row")]),
        ("td", &[]),
        ("td", &[]),
        ("td", &[]),
    ];
    let (capture, table, cells) = table(&[row]);
    let model = model_of(&capture, &table);
    let (h1, a, b, h2, c, d, p) = (0, 1, 2, 3, 4, 5, 6);
    assert_eq!(
        associations(&capture, &model, &cells),
        [(a, h1), (b, h1), (c, h2), (d, h2), (p, h2)]
    );
}

#[test]
fn a_column_scan_adds_the_nearest_header_block_and_later_equal_extent_headers_are_opaque() {
    // The same cells in one column, th with scope=col, read upward.
    let column: Vec<[Spec<'_>; 1]> = vec![
        [("th", &[("scope", "col")])],
        [("td", &[])],
        [("td", &[])],
        [("th", &[("scope", "col")])],
        [("td", &[])],
        [("td", &[])],
        [("td", &[])],
    ];
    let rows: Vec<&[Spec<'_>]> = column.iter().map(|row| &row[..]).collect();
    let (capture, table, cells) = table(&rows);
    let model = model_of(&capture, &table);
    let (h1, a, b, h2, c, d, p) = (0, 1, 2, 3, 4, 5, 6);
    assert_eq!(
        associations(&capture, &model, &cells),
        [(a, h1), (b, h1), (c, h2), (d, h2), (p, h2)]
    );
}

#[test]
fn spanning_headers_reach_the_cells_below_and_beside_them_in_scan_order() {
    // Row 0:  C0  C1  C2          th scope=col
    // Row 1:  R(rowspan 2)  a  b  R is th scope=row
    // Row 2:  c  d                placed right of R
    // A cell takes its row scans (rows top to bottom, each leftward) before
    // its column scans, so a, b, c and d list R before their column header.
    // R, a header, scans up its column and finds C0, a column header.
    let (capture, table, cells) = table(&[
        &[
            ("th", &[("scope", "col")]),
            ("th", &[("scope", "col")]),
            ("th", &[("scope", "col")]),
        ],
        &[
            ("th", &[("scope", "row"), ("rowspan", "2")]),
            ("td", &[]),
            ("td", &[]),
        ],
        &[("td", &[]), ("td", &[])],
    ]);
    let model = model_of(&capture, &table);
    let (c0, c1, c2, r, a, b, c, d) = (0, 1, 2, 3, 4, 5, 6, 7);
    assert_eq!(
        associations(&capture, &model, &cells),
        [
            (r, c0),
            (a, r),
            (a, c1),
            (b, r),
            (b, c2),
            (c, r),
            (c, c1),
            (d, r),
            (d, c2)
        ]
    );
}

#[test]
fn rows_without_a_cell_of_their_own_are_listed_where_the_first_such_row_stands() {
    // Q (th, scope row) and P (td) span rows 0 to 2. Row 0 has S0 (th, scope
    // row) before P, row 1 is empty, row 2 has S2 (th, scope row) before P.
    // P scans row 0 leftward (S0 then Q, both row headers), row 1 (Q again) and
    // row 2 (S2 then Q), and adds each header when it first meets it: S0, Q, S2.
    // Row 1 has no cell of its own, so it shares its scan with no other row,
    // and its place in the order is between rows 0 and 2.
    let row_th = &[("scope", "row")];
    let (capture, table, cells) = table(&[
        &[
            ("th", &[("scope", "row"), ("rowspan", "3")]),
            ("th", row_th),
            ("td", &[("rowspan", "3")]),
        ],
        &[],
        &[("th", row_th)],
    ]);
    let model = model_of(&capture, &table);
    let [q, s0, p, s2] = [0, 1, 2, 3];
    let of_p: Vec<usize> = associations(&capture, &model, &cells)
        .into_iter()
        .filter(|(cell, _)| *cell == p)
        .map(|(_, header)| header)
        .collect();
    assert_eq!(of_p, [s0, q, s2]);
}

#[test]
fn the_shared_scan_of_a_run_stands_where_its_first_line_with_no_cell_of_its_own_stands() {
    // X, Y (th, scope row) and P (td) span rows 0 to 2. s0 and s0' are td of
    // row 0 between them, row 1 is empty, and row 2 has d2 (td) before and Z
    // (th, scope row) after Y.
    //   row 0 from P: s0', Y (a row header: taken), s0 (data: closes the
    //     block, so X, crossing Y's rows, is hidden): [Y].
    //   row 1 from P: Y, X (no data between them: both taken): [Y, X].
    //   row 2 from P: Z, Y (taken), d2 (closes the block), X (hidden): [Z, Y].
    // P adds each header when it first meets it, row by row: Y, X, Z.
    let row_th = &[("scope", "row"), ("rowspan", "3")];
    let (capture, table, cells) = table(&[
        &[
            ("th", row_th),
            ("td", &[]),
            ("th", row_th),
            ("td", &[]),
            ("td", &[("rowspan", "3")]),
        ],
        &[],
        &[("td", &[]), ("th", &[("scope", "row")])],
    ]);
    let model = model_of(&capture, &table);
    let [x, _s0, y, _s1, p, _d2, z] = [0, 1, 2, 3, 4, 5, 6];
    let of_p: Vec<usize> = associations(&capture, &model, &cells)
        .into_iter()
        .filter(|(cell, _)| *cell == p)
        .map(|(_, header)| header)
        .collect();
    assert_eq!(of_p, [y, x, z]);
}

#[test]
fn a_scope_too_long_to_read_refuses_a_table_with_a_row_group_only() {
    // The last `th` of the first row is reached by no scan: no cell lies right
    // of it or below it. Its scope is longer than the limit on attribute input.
    // A table without groups never reads it; a table with a row group reads the
    // scope of every header cell once, to know which cells name the group, and
    // so is refused.
    let long = "x".repeat(20_000);
    let long = long.as_str();
    let build = |grouped: bool| {
        let (mut b, d) = wide_builder_without_registries();
        let first = [
            cell(&mut b, &d, "td", &[]),
            cell(&mut b, &d, "th", &[("scope", long)]),
        ];
        let row = element(&mut b, &d, "tr", &[]);
        b.establish_children(&row, &first).unwrap();
        let rows = if grouped {
            let body = element(&mut b, &d, "tbody", &[]);
            b.establish_children(&body, &[row]).unwrap();
            body
        } else {
            row
        };
        let table = element(&mut b, &d, "table", &[]);
        b.establish_children(&table, &[rows]).unwrap();
        (b.finish().unwrap(), table)
    };
    let (capture, table) = build(false);
    assert!(query(&capture, &table, limits()).0.is_ok());
    let (capture, table) = build(true);
    assert!(query(&capture, &table, limits()).0.is_err());
}
