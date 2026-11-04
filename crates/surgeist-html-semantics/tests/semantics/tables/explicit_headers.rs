//! Explicit `headers` tokens: the cells they name and the work they cost.
//!
//! HTML 877d "algorithm for assigning header cells" (§4.9.12.2), step 3: for
//! each token of the principal cell's `headers` value, if the first element in
//! the Document whose ID equals the token is a cell in the same table, and is
//! not the principal cell, the cell joins the header list. Whatever element
//! owns an ID first in tree order decides the token, cell or not; a token that
//! names no element names nothing. The expectations below follow from that
//! text and from the fixtures' tree order, not from the implementation.
//!
//! The work of resolving all tokens must follow the size of the document and
//! the number of tokens, not their product: quadrupling the cells of a grid in
//! which every cell names a header by ID may grow the admitted work by about
//! four.

use crate::semantics::support::{context, wide_builder_without_registries as builder};
use surgeist_html::values::{
    Attribute, DocumentString, HtmlValueInterpreter, Namespace, OrderedAttributes, QualifiedName,
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits,
    HtmlCaptureRegistryInput, HtmlCaptureSubjectId, HtmlCapturedIsValue, HtmlSemanticCancellation,
    HtmlSemanticFact, HtmlSemanticFailure, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest,
    HtmlSemanticSession, HtmlSemanticValue, HtmlTableModel,
};

type Subject = HtmlCaptureBuilderSubject;

fn element(
    b: &mut HtmlCaptureBuilder,
    d: &Subject,
    local: &str,
    attrs: &[(&str, &str)],
) -> Subject {
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

fn cell(b: &mut HtmlCaptureBuilder, d: &Subject, local: &str, attrs: &[(&str, &str)]) -> Subject {
    let e = element(b, d, local, attrs);
    let t = b.add_text(d, DocumentString::from("x")).unwrap();
    b.establish_children(&e, &[t]).unwrap();
    e
}

/// A `tr` holding `cells`.
fn row(b: &mut HtmlCaptureBuilder, d: &Subject, cells: &[Subject]) -> Subject {
    let tr = element(b, d, "tr", &[]);
    b.establish_children(&tr, cells).unwrap();
    tr
}

/// A table of one row of `cells`.
fn table(b: &mut HtmlCaptureBuilder, d: &Subject, cells: &[Subject]) -> Subject {
    let tr = row(b, d, cells);
    let table = element(b, d, "table", &[]);
    b.establish_children(&table, &[tr]).unwrap();
    table
}

/// An element with no children.
fn leaf(b: &mut HtmlCaptureBuilder, d: &Subject, id: &str) -> Subject {
    let e = element(b, d, "div", &[("id", id)]);
    b.establish_children(&e, &[]).unwrap();
    e
}

fn query(
    c: &HtmlCapture,
    table: &Subject,
    id_units: usize,
    limits: HtmlSemanticLimits,
) -> Result<HtmlSemanticReport, HtmlSemanticFailure> {
    let cx = context(c);
    HtmlSemanticSession::new(
        c,
        &cx,
        HtmlValueInterpreter::new(id_units),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        c.subject_id(table).unwrap(),
        HtmlSemanticQuery::TableModel,
    )])
}

fn small_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(100000000, 100, 100000, 10000, 100000000, 100000)
}

fn model(report: &HtmlSemanticReport) -> &HtmlTableModel {
    match report.results()[0].fact() {
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected complete table: {other:?}"),
    }
}

/// The header associations of the table, as (cell, header) pairs.
fn edges(c: &HtmlCapture, table: &Subject) -> Vec<(HtmlCaptureSubjectId, HtmlCaptureSubjectId)> {
    let report = query(c, table, 10000, small_limits()).unwrap();
    model(&report)
        .headers()
        .iter()
        .map(|edge| (edge.cell().clone(), edge.header().clone()))
        .collect()
}

fn id(c: &HtmlCapture, subject: &Subject) -> HtmlCaptureSubjectId {
    c.subject_id(subject).unwrap()
}

#[test]
fn duplicate_ids_in_the_table_name_the_first_cell_in_tree_order() {
    let (mut b, d) = builder();
    let first = cell(&mut b, &d, "th", &[("id", "dup"), ("headers", "")]);
    let second = cell(&mut b, &d, "th", &[("id", "dup"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "dup")]);
    let t = table(&mut b, &d, &[first.clone(), second, data.clone()]);
    b.establish_children(&d, std::slice::from_ref(&t)).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(edges(&c, &t), vec![(id(&c, &data), id(&c, &first))]);
}

#[test]
fn a_token_with_no_element_is_ignored_and_the_other_tokens_still_resolve() {
    let (mut b, d) = builder();
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "none h also-none H")]);
    let t = table(&mut b, &d, &[h.clone(), data.clone()]);
    b.establish_children(&d, std::slice::from_ref(&t)).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(edges(&c, &t), vec![(id(&c, &data), id(&c, &h))]);
}

#[test]
fn a_cell_naming_only_itself_has_no_headers_and_names_others_in_token_order() {
    let (mut b, d) = builder();
    let a = cell(&mut b, &d, "th", &[("id", "a"), ("headers", "")]);
    let z = cell(&mut b, &d, "th", &[("id", "z"), ("headers", "")]);
    let lone = cell(&mut b, &d, "td", &[("id", "me"), ("headers", "me")]);
    let both = cell(&mut b, &d, "td", &[("id", "us"), ("headers", "z us a")]);
    let t = table(
        &mut b,
        &d,
        &[a.clone(), z.clone(), lone.clone(), both.clone()],
    );
    b.establish_children(&d, std::slice::from_ref(&t)).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(
        edges(&c, &t),
        vec![(id(&c, &both), id(&c, &z)), (id(&c, &both), id(&c, &a)),]
    );
}

#[test]
fn an_earlier_element_outside_the_table_owns_the_id_even_when_a_cell_has_it_too() {
    let (mut b, d) = builder();
    let outsider = leaf(&mut b, &d, "h");
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h, data]);
    b.establish_children(&d, &[outsider, t.clone()]).unwrap();
    let c = b.finish().unwrap();
    assert!(edges(&c, &t).is_empty());
}

#[test]
fn a_later_element_outside_the_table_does_not_take_the_id_from_a_cell() {
    let (mut b, d) = builder();
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h.clone(), data.clone()]);
    let outsider = leaf(&mut b, &d, "h");
    b.establish_children(&d, &[t.clone(), outsider]).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(edges(&c, &t), vec![(id(&c, &data), id(&c, &h))]);
}

#[test]
fn a_cell_of_another_table_with_the_id_is_not_a_cell_in_the_same_table() {
    let (mut b, d) = builder();
    let elsewhere = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let other = table(&mut b, &d, &[elsewhere]);
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h, data]);
    b.establish_children(&d, &[other, t.clone()]).unwrap();
    let c = b.finish().unwrap();
    assert!(edges(&c, &t).is_empty());
}

#[test]
fn a_header_found_before_an_unknown_branch_resolves_without_that_branch() {
    let (mut b, d) = builder();
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h.clone(), data.clone()]);
    // Its children are not established: unknown.
    let unknown = element(&mut b, &d, "section", &[]);
    b.establish_children(&d, &[t.clone(), unknown]).unwrap();
    let c = b.finish().unwrap();
    assert_eq!(edges(&c, &t), vec![(id(&c, &data), id(&c, &h))]);
}

#[test]
fn a_token_beyond_an_unknown_branch_leaves_the_model_missing_that_branch() {
    let (mut b, d) = builder();
    let unknown = element(&mut b, &d, "section", &[]);
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h, data]);
    b.establish_children(&d, &[unknown.clone(), t.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let report = query(&c, &t, 10000, small_limits()).unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(id(&c, &unknown))
        ))
    );
}

/// A document with `ids` ahead of the table, whose lone data cell names `token`
/// and whose header is `h`; an element whose ID exceeds the interpreter's cap
/// of 8 units is refused when the scan reaches it.
fn refused_id_before(token: &str, ahead: &[&str]) -> (HtmlCapture, Subject) {
    let (mut b, d) = builder();
    let mut children: Vec<Subject> = Vec::new();
    for id in ahead {
        children.push(if *id == "unknown" {
            element(&mut b, &d, "section", &[])
        } else {
            leaf(&mut b, &d, id)
        });
    }
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", token)]);
    let t = table(&mut b, &d, &[h, data]);
    children.push(t.clone());
    b.establish_children(&d, &children).unwrap();
    (b.finish().unwrap(), t)
}

const REFUSED: &str = "far-too-long";

#[test]
fn an_id_the_interpreter_refuses_stops_a_token_that_the_scan_reaches_after_it() {
    let (c, t) = refused_id_before("h", &[REFUSED]);
    let failure = query(&c, &t, 8, small_limits()).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}

#[test]
fn an_id_the_interpreter_refuses_is_not_reached_by_a_token_matched_before_it() {
    let (mut b, d) = builder();
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", "h")]);
    let t = table(&mut b, &d, &[h.clone(), data.clone()]);
    let refused = leaf(&mut b, &d, REFUSED);
    b.establish_children(&d, &[t.clone(), refused]).unwrap();
    let c = b.finish().unwrap();
    let report = query(&c, &t, 8, small_limits()).unwrap();
    let found: Vec<_> = model(&report)
        .headers()
        .iter()
        .map(|e| (e.cell().clone(), e.header().clone()))
        .collect();
    assert_eq!(found, vec![(id(&c, &data), id(&c, &h))]);
}

#[test]
fn the_first_of_a_refused_id_and_an_unknown_branch_in_tree_order_ends_the_search() {
    // An unknown branch, then a refused ID: the branch is reached first.
    let (c, t) = refused_id_before("nothing", &["unknown", REFUSED]);
    let report = query(&c, &t, 8, small_limits()).unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(_)
        ))
    ));
    // A refused ID, then an unknown branch: the ID is reached first.
    let (c, t) = refused_id_before("nothing", &[REFUSED, "unknown"]);
    let failure = query(&c, &t, 8, small_limits()).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
}

// ---- Work ---------------------------------------------------------------

fn work_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(
        usize::MAX / 4,
        100,
        10_000_000,
        10000,
        usize::MAX / 4,
        10_000_000,
    )
}

/// An `n` by `n` table. The last row holds `n` th cells, `h0` to `h{n-1}`,
/// that name nothing; every cell above is a td naming the th of its column.
/// The th are last in tree order and last among the cells, so a token cannot
/// be answered by a short search.
fn grid(n: usize) -> (HtmlCapture, Subject, Vec<Subject>) {
    let (mut b, d) = crate::semantics::support::builder_with(HtmlCaptureLimits::new(
        1_000_000,
        10_000_000,
        1_000_000,
        100_000_000,
        1000,
        0,
        0,
        16_777_216,
    ));
    let names: Vec<String> = (0..n).map(|x| format!("h{x}")).collect();
    let mut rows = Vec::new();
    for _ in 0..n - 1 {
        let cells: Vec<_> = names
            .iter()
            .map(|name| cell(&mut b, &d, "td", &[("headers", name)]))
            .collect();
        rows.push(row(&mut b, &d, &cells));
    }
    let heads: Vec<_> = names
        .iter()
        .map(|name| cell(&mut b, &d, "th", &[("id", name), ("headers", "")]))
        .collect();
    rows.push(row(&mut b, &d, &heads));
    let table = element(&mut b, &d, "table", &[]);
    b.establish_children(&table, &rows).unwrap();
    b.establish_children(&d, std::slice::from_ref(&table))
        .unwrap();
    (b.finish().unwrap(), table, heads)
}

/// The work of the query, after checking that every td has exactly the th of
/// its column as its header.
fn work(n: usize) -> usize {
    let (c, t, heads) = grid(n);
    let report = query(&c, &t, 10000, work_limits()).unwrap();
    let m = model(&report);
    assert_eq!(m.cells().len(), n * n);
    assert_eq!(m.headers().len(), n * (n - 1), "n={n}");
    let columns: Vec<_> = heads.iter().map(|h| id(&c, h)).collect();
    for edge in m.headers() {
        let cell = m.cells().iter().find(|cell| cell.subject() == edge.cell());
        let x = cell.map(|cell| cell.x()).unwrap();
        assert_eq!(edge.header(), &columns[x], "n={n}");
    }
    report.usage().work()
}

#[test]
fn explicit_headers_work_grows_about_four_times_per_four_times_the_cells() {
    let sizes = [5, 10, 20, 40];
    let measured: Vec<usize> = sizes.iter().map(|n| work(*n)).collect();
    for (pair, sizes) in measured.windows(2).zip(sizes.windows(2)) {
        // Four times the cells and tokens, linear work, plus a small allowance
        // for the logarithmic lookups and the fixed per-table charges.
        assert!(
            pair[1] <= pair[0] * 5,
            "work for {0}x{0} is {1} but {2} for {3}x{3} (cells x4 may cost at most x5)",
            sizes[0],
            pair[0],
            pair[1],
            sizes[1],
        );
    }
}
