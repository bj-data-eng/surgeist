//! Boundary values for the table span caps, zero handling and th scope states.
//!
//! Expected values come from the pinned HTML edition's table sections (forming
//! a table, forming relationships between data cells and header cells, and the
//! `colspan`, `rowspan`, `span` and `scope` attribute definitions), never from
//! the semantics implementation's own output.

use crate::semantics::support::{builder_with, context};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlSemanticCancellation, HtmlSemanticFact,
    HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlSemanticValue, HtmlTableModel,
};

fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    builder_with(HtmlCaptureLimits::new(
        10000, 100000, 10000, 1000000, 1000, 0, 0, 16_777_216,
    ))
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    let attrs = OrderedAttributes::new(attrs.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap();
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        attrs,
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn cell(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    let e = element(b, d, local, attrs);
    let t = b.add_text(d, DocumentString::from("x")).unwrap();
    b.establish_children(&e, &[t]).unwrap();
    e
}
fn run(c: &HtmlCapture, table: &HtmlCaptureBuilderSubject) -> HtmlSemanticReport {
    let cx = context(c);
    HtmlSemanticSession::new(
        c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(1_000_000_000, 100, 100000, 10000, 100000000, 200000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        c.subject_id(table).unwrap(),
        HtmlSemanticQuery::TableModel,
    )])
    .unwrap()
}
fn model(report: &HtmlSemanticReport) -> &HtmlTableModel {
    match report.results()[0].fact() {
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected complete table: {other:?}"),
    }
}

/// Raw colspan/span values with the width HTML's forming-a-table algorithm
/// selects: failure and zero give 1, and anything above 1000 is 1000.
const WIDTHS: &[(&str, usize)] = &[
    ("0", 1),
    ("1", 1),
    ("2", 2),
    ("999", 999),
    ("1000", 1000),
    ("1001", 1000),
    ("0001001", 1000),
    ("+1001", 1000),
    ("1000tail", 1000),
    ("", 1),
    ("many", 1),
    ("-1", 1),
    ("-0", 1),
    ("999999999999999999999999999999999999", 1000),
];

#[test]
fn colspan_selects_one_for_zero_or_failure_and_caps_at_one_thousand() {
    for local in ["td", "th"] {
        for (raw, width) in WIDTHS {
            let (mut b, d) = builder();
            let table = element(&mut b, &d, "table", &[]);
            let row = element(&mut b, &d, "tr", &[]);
            let c = cell(&mut b, &d, local, &[("colspan", raw)]);
            b.establish_children(&row, &[c]).unwrap();
            b.establish_children(&table, &[row]).unwrap();
            let capture = b.finish().unwrap();
            let report = run(&capture, &table);
            let m = model(&report);
            assert_eq!(
                (m.width(), m.cells()[0].width(), m.cells()[0].height()),
                (*width, *width, 1),
                "{local} colspan={raw:?}"
            );
        }
    }
}

#[test]
fn absent_colspan_selects_one() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let row = element(&mut b, &d, "tr", &[]);
    let c = cell(&mut b, &d, "td", &[]);
    b.establish_children(&row, &[c]).unwrap();
    b.establish_children(&table, &[row]).unwrap();
    let capture = b.finish().unwrap();
    let report = run(&capture, &table);
    assert_eq!(model(&report).cells()[0].width(), 1);
}

#[test]
fn col_span_selects_one_for_zero_or_failure_and_caps_at_one_thousand() {
    for (raw, width) in WIDTHS {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        // A colgroup with col children ignores its own span.
        let group = element(&mut b, &d, "colgroup", &[("span", "7")]);
        let col = element(&mut b, &d, "col", &[("span", raw)]);
        b.establish_children(&group, &[col]).unwrap();
        b.establish_children(&table, &[group]).unwrap();
        let capture = b.finish().unwrap();
        let report = run(&capture, &table);
        let m = model(&report);
        assert_eq!(m.width(), *width, "col span={raw:?}");
        assert_eq!(m.column_groups()[0].width(), *width, "col span={raw:?}");
    }
}

#[test]
fn colgroup_span_without_col_children_selects_one_for_zero_or_failure_and_caps() {
    for (raw, width) in WIDTHS {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        let group = element(&mut b, &d, "colgroup", &[("span", raw)]);
        b.establish_children(&group, &[]).unwrap();
        b.establish_children(&table, &[group]).unwrap();
        let capture = b.finish().unwrap();
        let report = run(&capture, &table);
        let m = model(&report);
        assert_eq!(m.width(), *width, "colgroup span={raw:?}");
        assert_eq!(
            m.column_groups()[0].width(),
            *width,
            "colgroup span={raw:?}"
        );
    }
}

#[test]
fn absent_col_and_colgroup_span_select_one() {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let with_col = element(&mut b, &d, "colgroup", &[]);
    let col = element(&mut b, &d, "col", &[]);
    b.establish_children(&with_col, &[col]).unwrap();
    let bare = element(&mut b, &d, "colgroup", &[]);
    b.establish_children(&bare, &[]).unwrap();
    b.establish_children(&table, &[with_col, bare]).unwrap();
    let capture = b.finish().unwrap();
    let report = run(&capture, &table);
    let m = model(&report);
    assert_eq!(m.width(), 2);
    assert_eq!(
        m.column_groups()
            .iter()
            .map(|g| (g.x(), g.width()))
            .collect::<Vec<_>>(),
        vec![(0, 1), (1, 1)]
    );
}

#[test]
fn rowspan_zero_grows_to_the_end_of_its_row_group_and_one_stays_one() {
    // Row group one has three rows; rowspan zero ends with that group, not the
    // table, so the following row group's row is not covered.
    for (raw, grower_height) in [
        ("0", 3),
        ("1", 1),
        ("2", 2),
        ("", 1),
        ("many", 1),
        ("-1", 1),
    ] {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        let first = element(&mut b, &d, "tbody", &[]);
        let second = element(&mut b, &d, "tbody", &[]);
        let r1 = element(&mut b, &d, "tr", &[]);
        let r2 = element(&mut b, &d, "tr", &[]);
        let r3 = element(&mut b, &d, "tr", &[]);
        let r4 = element(&mut b, &d, "tr", &[]);
        let grower = cell(&mut b, &d, "td", &[("rowspan", raw)]);
        let a = cell(&mut b, &d, "td", &[]);
        let b2 = cell(&mut b, &d, "td", &[]);
        let c3 = cell(&mut b, &d, "td", &[]);
        let d4 = cell(&mut b, &d, "td", &[]);
        b.establish_children(&r1, &[grower.clone()]).unwrap();
        b.establish_children(&r2, &[a]).unwrap();
        b.establish_children(&r3, &[b2]).unwrap();
        b.establish_children(&r4, &[c3, d4]).unwrap();
        b.establish_children(&first, &[r1, r2, r3]).unwrap();
        b.establish_children(&second, &[r4]).unwrap();
        b.establish_children(&table, &[first, second]).unwrap();
        let capture = b.finish().unwrap();
        let report = run(&capture, &table);
        let m = model(&report);
        assert_eq!(m.height(), 4, "rowspan={raw:?}");
        assert_eq!(
            m.cells()[0].subject(),
            &capture.subject_id(&grower).unwrap()
        );
        assert_eq!(m.cells()[0].height(), grower_height, "rowspan={raw:?}");
    }
}

#[test]
fn rowspan_is_capped_at_sixty_five_thousand_five_hundred_thirty_four() {
    for (raw, height) in [("65534", 65534), ("65535", 65534), ("65536", 65534)] {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        let row = element(&mut b, &d, "tr", &[]);
        let c = cell(&mut b, &d, "td", &[("rowspan", raw)]);
        b.establish_children(&row, &[c]).unwrap();
        b.establish_children(&table, &[row]).unwrap();
        let capture = b.finish().unwrap();
        let report = run(&capture, &table);
        let m = model(&report);
        assert_eq!(
            (m.height(), m.cells()[0].height()),
            (height, height),
            "rowspan={raw:?}"
        );
    }
}

/// The three cells that can receive the scoped header: `a` beside it, `b`
/// below it and `c` diagonally below.
struct ScopeFixture {
    capture: HtmlCapture,
    header: HtmlCaptureBuilderSubject,
    cells: [HtmlCaptureBuilderSubject; 3],
    table: HtmlCaptureBuilderSubject,
}

/// Header H at (0,0), data a at (1,0), b at (0,1) and c at (1,1). With a
/// tbody the cells are in one row group; with `row_grouped` false the rows are
/// direct table children (no row group) and a two-wide colgroup precedes them.
fn scope_fixture(scope: Option<&str>, row_grouped: bool) -> ScopeFixture {
    let (mut b, d) = builder();
    let table = element(&mut b, &d, "table", &[]);
    let attrs: Vec<(&str, &str)> = scope.map(|s| ("scope", s)).into_iter().collect();
    let header = cell(&mut b, &d, "th", &attrs);
    let a = cell(&mut b, &d, "td", &[]);
    let below = cell(&mut b, &d, "td", &[]);
    let c = cell(&mut b, &d, "td", &[]);
    let r1 = element(&mut b, &d, "tr", &[]);
    let r2 = element(&mut b, &d, "tr", &[]);
    b.establish_children(&r1, &[header.clone(), a.clone()])
        .unwrap();
    b.establish_children(&r2, &[below.clone(), c.clone()])
        .unwrap();
    if row_grouped {
        let body = element(&mut b, &d, "tbody", &[]);
        b.establish_children(&body, &[r1, r2]).unwrap();
        b.establish_children(&table, &[body]).unwrap();
    } else {
        let group = element(&mut b, &d, "colgroup", &[("span", "2")]);
        b.establish_children(&group, &[]).unwrap();
        b.establish_children(&table, &[group, r1, r2]).unwrap();
    }
    ScopeFixture {
        capture: b.finish().unwrap(),
        header,
        cells: [a, below, c],
        table,
    }
}

/// Whether each of `a`, `b` and `c` is assigned the scoped header.
fn assigned(fixture: &ScopeFixture) -> [bool; 3] {
    let report = run(&fixture.capture, &fixture.table);
    let m = model(&report);
    let header = fixture.capture.subject_id(&fixture.header).unwrap();
    fixture.cells.clone().map(|cell| {
        let cell = fixture.capture.subject_id(&cell).unwrap();
        m.headers()
            .iter()
            .any(|edge| edge.cell() == &cell && edge.header() == &header)
    })
}

#[test]
fn scope_keywords_select_their_states_ascii_case_insensitively() {
    const T: bool = true;
    const F: bool = false;
    // (value, assigned in a row-grouped table, assigned in a column-grouped one)
    //
    // Row: a row header, assigned to the cell beside it. Col: a column header,
    // assigned to the cell below it. Row Group and Column Group: assigned to
    // every cell at or after the anchor within their own kind of group, so only
    // where the table actually has that group.
    for (value, grouped, column_grouped) in [
        ("row", [T, F, F], [T, F, F]),
        ("ROW", [T, F, F], [T, F, F]),
        ("rOw", [T, F, F], [T, F, F]),
        ("col", [F, T, F], [F, T, F]),
        ("COL", [F, T, F], [F, T, F]),
        ("CoL", [F, T, F], [F, T, F]),
        ("rowgroup", [T, T, T], [F, F, F]),
        ("ROWGROUP", [T, T, T], [F, F, F]),
        ("RowGroup", [T, T, T], [F, F, F]),
        ("colgroup", [F, F, F], [T, T, T]),
        ("COLGROUP", [F, F, F], [T, T, T]),
        ("ColGroup", [F, F, F], [T, T, T]),
    ] {
        assert_eq!(
            assigned(&scope_fixture(Some(value), true)),
            grouped,
            "scope={value:?} in a row group"
        );
        assert_eq!(
            assigned(&scope_fixture(Some(value), false)),
            column_grouped,
            "scope={value:?} in a column group"
        );
    }
}

#[test]
fn missing_empty_and_invalid_scope_select_auto_which_here_matches_no_cell() {
    // Auto: the header is a column header only if no data cell shares its row
    // (a does), and a row header only if it is not a column header and no data
    // cell shares its column (b does). So neither scan assigns it, and Auto is
    // not a group state.
    let invalid = [
        "",
        "auto",
        "AUTO",
        " row",
        "row ",
        "\trow",
        "rows",
        "ro",
        "r",
        "colgrou",
        "colgroups",
        "rowgroup2",
        "ro w",
        "r\u{ff4f}w",
        "\u{212a}",
    ];
    assert_eq!(assigned(&scope_fixture(None, true)), [false; 3]);
    assert_eq!(assigned(&scope_fixture(None, false)), [false; 3]);
    for value in invalid {
        assert_eq!(
            assigned(&scope_fixture(Some(value), true)),
            [false; 3],
            "scope={value:?} in a row group"
        );
        assert_eq!(
            assigned(&scope_fixture(Some(value), false)),
            [false; 3],
            "scope={value:?} in a column group"
        );
    }
}

#[test]
fn auto_scope_is_a_row_header_when_no_data_cell_shares_its_column() {
    // Auto: H at (0,0) with data a at (1,0) and no cell under H. It is not a
    // column header (a shares its row) but is a row header (nothing below it),
    // so a receives it. An invalid keyword must behave identically to Auto.
    for scope in [None, Some(""), Some("auto"), Some("rows")] {
        let (mut b, d) = builder();
        let table = element(&mut b, &d, "table", &[]);
        let attrs: Vec<(&str, &str)> = scope.map(|s| ("scope", s)).into_iter().collect();
        let header = cell(&mut b, &d, "th", &attrs);
        let a = cell(&mut b, &d, "td", &[]);
        let r1 = element(&mut b, &d, "tr", &[]);
        b.establish_children(&r1, &[header.clone(), a.clone()])
            .unwrap();
        b.establish_children(&table, &[r1]).unwrap();
        let capture = b.finish().unwrap();
        let report = run(&capture, &table);
        let edges: Vec<_> = model(&report)
            .headers()
            .iter()
            .map(|edge| (edge.cell().clone(), edge.header().clone()))
            .collect();
        assert_eq!(
            edges,
            vec![(
                capture.subject_id(&a).unwrap(),
                capture.subject_id(&header).unwrap()
            )],
            "scope={scope:?}"
        );
    }
}
