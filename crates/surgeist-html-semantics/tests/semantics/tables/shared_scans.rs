//! Header lists of cells that cross rows (or columns) holding cells of their
//! own among the tall cells.
//!
//! HTML 877d, internal algorithm for scanning and assigning header cells,
//! steps 4 and 9: a scan goes from the principal cell toward the start of its
//! row (or column), a header joins the block of headers it follows, a data
//! cell ends the block and makes its headers opaque, and a header is hidden by
//! an opaque one anchored at the same place with the same extent. The scans
//! of the rows a cell crosses run top to bottom and their results are merged
//! keeping the first of each header. Rows that cross the same tall cells
//! differ only by the cells of their own, so a table below puts those among
//! tall cells that share their rows and checks the list of its tall data cell
//! `P`, derived by hand, for the layout as drawn and for its transpose (whose
//! cells scan upward instead).
//!
//! Every cell but `P` carries `headers=""`, which keeps it from scanning, so
//! only `P` takes headers. The `th` cells have `scope="row"`.

use super::scope_work::{cell, element, limits, query};
use crate::semantics::support::wide_builder_without_registries;
use surgeist_html_semantics::{
    HtmlCaptureBuilderSubject, HtmlCaptureSubjectId, HtmlSemanticFact, HtmlSemanticValue,
};

/// A cell of the layout, by the slots it covers.
struct Spec {
    name: &'static str,
    header: bool,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

const fn th(name: &'static str, x: usize, y: usize, width: usize, height: usize) -> Spec {
    Spec {
        name,
        header: true,
        x,
        y,
        width,
        height,
    }
}

const fn td(name: &'static str, x: usize, y: usize, width: usize, height: usize) -> Spec {
    Spec {
        name,
        header: false,
        x,
        y,
        width,
        height,
    }
}

/// The names of the headers of `principal`, in order, for the layout (or its
/// transpose). The table is built in rows, each holding its cells in column
/// order; the layouts leave no gap before a cell, so each lands at its slot.
fn headers_of(layout: &[Spec], transpose: bool, principal: &str) -> Vec<&'static str> {
    let place = |spec: &Spec| {
        if transpose {
            (spec.y, spec.x, spec.height, spec.width)
        } else {
            (spec.x, spec.y, spec.width, spec.height)
        }
    };
    let (mut b, d) = wide_builder_without_registries();
    let table = element(&mut b, &d, "table", &[]);
    let rows = layout.iter().map(|s| place(s).1).max().unwrap() + 1;
    let mut subjects = Vec::new();
    let mut trs = Vec::new();
    for y in 0..rows {
        let mut in_row: Vec<_> = layout.iter().filter(|s| place(s).1 == y).collect();
        in_row.sort_by_key(|s| place(s).0);
        let mut cells = Vec::new();
        for spec in in_row {
            let (_, _, width, height) = place(spec);
            let (colspan, rowspan) = (width.to_string(), height.to_string());
            let mut attrs = Vec::new();
            if width > 1 {
                attrs.push(("colspan", colspan.as_str()));
            }
            if height > 1 {
                attrs.push(("rowspan", rowspan.as_str()));
            }
            if spec.header {
                attrs.push(("scope", if transpose { "col" } else { "row" }));
            }
            if spec.name != principal {
                attrs.push(("headers", ""));
            }
            let subject = cell(&mut b, &d, if spec.header { "th" } else { "td" }, &attrs);
            cells.push(subject.clone());
            subjects.push((spec.name, subject));
        }
        let row = element(&mut b, &d, "tr", &[]);
        b.establish_children(&row, &cells).unwrap();
        trs.push(row);
    }
    b.establish_children(&table, &trs).unwrap();
    let capture = b.finish().unwrap();
    let report = query(&capture, &table, limits()).0.unwrap();
    let HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(model)) = report.results()[0].fact()
    else {
        panic!("expected a complete table model");
    };
    // Every cell lands in the slots the layout draws.
    for spec in layout {
        let placed = model
            .cells()
            .iter()
            .find(|c| {
                &capture
                    .subject_id(&subject_of(&subjects, spec.name))
                    .unwrap()
                    == c.subject()
            })
            .unwrap();
        assert_eq!(
            (placed.x(), placed.y(), placed.width(), placed.height()),
            place(spec),
            "{} lands where the layout draws it",
            spec.name
        );
    }
    let name_of = |id: &HtmlCaptureSubjectId| {
        subjects
            .iter()
            .find(|(_, subject)| &capture.subject_id(subject).unwrap() == id)
            .map(|(name, _)| *name)
            .unwrap()
    };
    model
        .headers()
        .iter()
        .filter(|association| name_of(association.cell()) == principal)
        .map(|association| name_of(association.header()))
        .collect()
}

fn subject_of(
    subjects: &[(&'static str, HtmlCaptureBuilderSubject)],
    name: &str,
) -> HtmlCaptureBuilderSubject {
    subjects
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, subject)| subject.clone())
        .unwrap()
}

fn assert_headers(layout: &[Spec], principal: &str, expected: &[&str]) {
    for transpose in [false, true] {
        assert_eq!(
            headers_of(layout, transpose, principal),
            expected,
            "transpose={transpose}"
        );
    }
}

/// Row 0 holds data cells `x1`, `x3` and `x5` between tall headers of two
/// kinds of extent: `a0` and `a1` cross two rows, `b0` and `b1` three, and the
/// tall `td` `P` ends the row. Row 1 holds headers `h1` and `h3` where row 0
/// holds data cells, and a data cell `d5` where row 0 holds `x5`. Row 2, which
/// only `b0`, `b1` and `P` cross, holds `z` at column 0. `P` scans rows 0, 1
/// and 2:
///
/// - Row 0, nearest first: `b1` is taken; `x5` ends its block, so `b0`, of the
///   same rows, is hidden; `x3`; `a1` is taken, since only `b1` is opaque;
///   `x1` ends its block, so `a0` is hidden. Taken: `b1`, `a1`.
/// - Row 1: `b1` is taken; `d5` makes it opaque and hides `b0`; `h3`, `a1` and
///   `h1` and `a0` follow with no data cell between them, so all are taken.
///   New: `h3`, `h1`, `a0`.
/// - Row 2: `b1` and `b0` share a block, as the empty slot between them holds
///   no cell, and `z` follows. New: `b0`.
fn two_kinds_of_extent() -> Vec<Spec> {
    vec![
        th("a0", 0, 0, 1, 2),
        td("x1", 1, 0, 1, 1),
        th("a1", 2, 0, 1, 2),
        td("x3", 3, 0, 1, 1),
        th("b0", 4, 0, 1, 3),
        td("x5", 5, 0, 1, 1),
        th("b1", 6, 0, 1, 3),
        td("P", 7, 0, 1, 3),
        th("h1", 1, 1, 1, 1),
        th("h3", 3, 1, 1, 1),
        td("d5", 5, 1, 1, 1),
        td("z", 0, 2, 1, 1),
    ]
}

#[test]
fn a_row_lacking_a_data_cell_the_first_row_had_lists_the_headers_it_hid() {
    assert_headers(
        &two_kinds_of_extent(),
        "P",
        &["b1", "a1", "h3", "h1", "a0", "b0"],
    );
}

/// `S_a` and `S_b` are headers of one row, with a tall data cell `Dl` between
/// them, and `Tt` is a tall header right of `S_b`. `P` scans rows 0 and 1; in
/// each, `Tt` is taken, `S_b` joins its block and is taken, `Dl` ends the
/// block, which makes `S_b` opaque, and `S_a`, of the same row, is hidden.
/// Taken in order: `Tt`, `S_b0`, then `Tt` again and `S_b1`.
#[test]
fn a_tall_data_cell_between_headers_of_a_row_hides_the_earlier_header() {
    let layout = vec![
        th("S_a0", 0, 0, 1, 1),
        td("Dl", 1, 0, 1, 2),
        th("S_b0", 2, 0, 1, 1),
        th("Tt", 3, 0, 1, 2),
        td("P", 4, 0, 1, 2),
        th("S_a1", 0, 1, 1, 1),
        th("S_b1", 2, 1, 1, 1),
    ];
    assert_headers(&layout, "P", &["Tt", "S_b0", "S_b1"]);
}

/// Row 1 holds no cell of its own. `P` scans row 0: `T1` is taken, `x` ends
/// its block, and `T0`, of the same rows, is hidden. Row 1: `T1` and `T0` share
/// a block, so both are taken, `T0` new.
#[test]
fn a_row_with_no_cell_of_its_own_lists_what_an_earlier_row_hid() {
    let layout = vec![
        th("T0", 0, 0, 1, 2),
        td("x", 1, 0, 1, 1),
        th("T1", 2, 0, 1, 2),
        td("P", 3, 0, 1, 2),
    ];
    assert_headers(&layout, "P", &["T1", "T0"]);
}

/// Tall `th` `T0`, a `td` in each of three rows, three tall `th` of the same
/// rows, and the tall `td` `P`. In every row `T3`, `T2` and `T1` share a block
/// and are taken; the row's `td` ends it and hides `T0`.
#[test]
fn a_cell_between_tall_headers_of_the_same_rows_hides_those_before_it() {
    let layout = vec![
        th("T0", 0, 0, 1, 3),
        td("D0", 1, 0, 1, 1),
        th("T1", 2, 0, 1, 3),
        th("T2", 3, 0, 1, 3),
        th("T3", 4, 0, 1, 3),
        td("P", 5, 0, 1, 3),
        td("D1", 1, 1, 1, 1),
        td("D2", 1, 2, 1, 1),
    ];
    assert_headers(&layout, "P", &["T3", "T2", "T1"]);
}
