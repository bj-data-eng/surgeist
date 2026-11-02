//! Work admitted for th scope reads while a table query assigns headers.
//!
//! HTML 877d "algorithm for assigning header cells" consults a header cell's
//! `scope` state for every data or header cell whose scan reaches it. The
//! scope attribute of one cell never changes during a query, so the work
//! charged for reading it must grow with the number of header cells, not with
//! the number of header and principal-cell pairs. The expectations below come
//! from counting header cells in the fixture, not from the implementation.

use crate::semantics::support::{builder_with, context};
use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCapturedIsValue, HtmlSemanticCancellation, HtmlSemanticFailure,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUsage,
};

pub(super) fn element(
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

pub(super) fn cell(
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

/// An `n` by `n` table whose first row and first column are th cells carrying
/// a `scope` of `scope_units` letters that match no keyword, and whose other
/// cells are td. The `2n - 1` header cells are each reached by the scan of
/// some other cell, and every cell is a principal cell.
pub(super) fn fixture(n: usize, scope_units: usize) -> (HtmlCapture, HtmlCaptureBuilderSubject) {
    let scope = "x".repeat(scope_units);
    let (mut b, d) = builder_with(HtmlCaptureLimits::new(
        10000, 100000, 10000, 1_000_000, 1000, 0, 0, 16_777_216,
    ));
    let table = element(&mut b, &d, "table", &[]);
    let mut rows = Vec::new();
    for y in 0..n {
        let mut cells = Vec::new();
        for x in 0..n {
            cells.push(if x == 0 || y == 0 {
                cell(&mut b, &d, "th", &[("scope", &scope)])
            } else {
                cell(&mut b, &d, "td", &[])
            });
        }
        let row = element(&mut b, &d, "tr", &[]);
        b.establish_children(&row, &cells).unwrap();
        rows.push(row);
    }
    b.establish_children(&table, &rows).unwrap();
    (b.finish().unwrap(), table)
}

pub(super) fn query(
    capture: &HtmlCapture,
    table: &HtmlCaptureBuilderSubject,
    limits: HtmlSemanticLimits,
) -> (
    Result<HtmlSemanticReport, HtmlSemanticFailure>,
    HtmlSemanticUsage,
) {
    let cx = context(capture);
    let mut session = HtmlSemanticSession::new(
        capture,
        &cx,
        HtmlValueInterpreter::new(10000),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let result = session.query(&[HtmlSemanticRequest::new(
        capture.subject_id(table).unwrap(),
        HtmlSemanticQuery::TableModel,
    )]);
    (result, session.usage())
}

pub(super) fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1_000_000_000, 100, 100000, 10000, 100000000, 200000)
}

pub(super) fn usage(n: usize, scope_units: usize) -> HtmlSemanticUsage {
    let (capture, table) = fixture(n, scope_units);
    query(&capture, &table, limits()).0.unwrap().usage()
}

#[test]
fn scope_work_grows_with_header_cells_not_header_and_cell_pairs() {
    const SHORT: usize = 10;
    const LONG: usize = 210;
    // Only the scope reads depend on the scope length, so the difference
    // isolates what the query charged for reading scope states.
    let scope_work = |n: usize| usage(n, LONG).work() - usage(n, SHORT).work();
    let headers = |n: usize| 2 * n - 1;

    let base = scope_work(2);
    assert!(base > 0, "scope reads must be charged at all");
    for n in [3, 5, 8] {
        // Equal work per header cell at every size: n=2 has 3 headers, but the
        // 8 by 8 table has 15 headers against 49 cells that scan them.
        assert_eq!(
            scope_work(n) * headers(2),
            base * headers(n),
            "scope work at n={n} is not proportional to its {} header cells",
            headers(n)
        );
    }
}

#[test]
fn cached_scope_states_keep_the_first_reached_limit_failure_and_balanced_cleanup() {
    let (capture, table) = fixture(4, 5);
    let (baseline, usage) = query(&capture, &table, limits());
    let baseline = baseline.unwrap();
    let work = usage.work();
    let peak = usage.peak_scratch_records();
    assert!(peak > 0);

    // One unit short of the total work stops in the query, not before it.
    let (short, after) = query(
        &capture,
        &table,
        HtmlSemanticLimits::new(work - 1, 100, 100000, 10000, 100000000, 200000),
    );
    assert!(matches!(
        short.unwrap_err().kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::Work,
            ..
        }
    ));
    assert_eq!(after.scratch_records(), 0);

    // The exact work budget succeeds with the same result.
    let (exact, _) = query(
        &capture,
        &table,
        HtmlSemanticLimits::new(work, 100, 100000, 10000, 100000000, 200000),
    );
    assert_eq!(exact.unwrap().results(), baseline.results());

    // Scratch records, including the cached scope states, are all released.
    let (short, after) = query(
        &capture,
        &table,
        HtmlSemanticLimits::new(1_000_000_000, 100, peak - 1, 10000, 100000000, 200000),
    );
    assert!(matches!(
        short.unwrap_err().kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            ..
        }
    ));
    assert_eq!(after.scratch_records(), 0);
    let (exact, after) = query(
        &capture,
        &table,
        HtmlSemanticLimits::new(1_000_000_000, 100, peak, 10000, 100000000, 200000),
    );
    assert_eq!(exact.unwrap().results(), baseline.results());
    assert_eq!(after.scratch_records(), 0);
}
