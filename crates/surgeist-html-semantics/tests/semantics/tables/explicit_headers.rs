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
    HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticUsage, HtmlSemanticValue,
    HtmlTableModel,
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

/// The failure is the interpreter refusing the `id` of the element that owns
/// `REFUSED`: 12 units against the cap of 8. A refused `headers` value or any
/// other attribute would report a different length.
fn assert_refused_id(failure: &HtmlSemanticFailure) {
    let HtmlSemanticFailureKind::Attribute(error) = failure.kind() else {
        panic!("expected a refused attribute: {failure:?}");
    };
    assert_eq!(
        (error.required(), error.capacity()),
        (REFUSED.len(), 8),
        "{failure:?}"
    );
}

#[test]
fn an_id_the_interpreter_refuses_stops_a_token_that_the_scan_reaches_after_it() {
    let (c, t) = refused_id_before("h", &[REFUSED]);
    let failure = query(&c, &t, 8, small_limits()).unwrap_err();
    assert_refused_id(&failure);
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
    assert_refused_id(&failure);
}

/// A table whose `th#h` is followed, after the table, by a stop: an element
/// whose ID the interpreter (cap 8) refuses, or one whose children are
/// unknown. The data cell names `tokens`. Returns the capture, the table, the
/// data cell, the `th` and the element that stops the search.
fn stop_after_table(
    tokens: &str,
    refused: bool,
) -> (HtmlCapture, Subject, Subject, Subject, Subject) {
    let (mut b, d) = builder();
    let h = cell(&mut b, &d, "th", &[("id", "h"), ("headers", "")]);
    let data = cell(&mut b, &d, "td", &[("headers", tokens)]);
    let t = table(&mut b, &d, &[h.clone(), data.clone()]);
    let stop = if refused {
        leaf(&mut b, &d, REFUSED)
    } else {
        // Its children are not established: unknown.
        element(&mut b, &d, "section", &[])
    };
    b.establish_children(&d, &[t.clone(), stop.clone()])
        .unwrap();
    (b.finish().unwrap(), t, data, h, stop)
}

#[test]
fn a_token_no_indexed_element_owns_meets_the_refused_id_that_follows_the_table() {
    // The index holds `h` when the search for `nothing` meets the stop.
    let (c, t, _, _, _) = stop_after_table("nothing", true);
    let failure = query(&c, &t, 8, small_limits()).unwrap_err();
    assert_refused_id(&failure);
}

#[test]
fn a_token_no_indexed_element_owns_meets_the_unknown_branch_that_follows_the_table() {
    let (c, t, _, _, stop) = stop_after_table("nothing", false);
    let report = query(&c, &t, 10000, small_limits()).unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::OrderedChildren(id(&c, &stop))
        ))
    );
}

#[test]
fn within_one_cell_only_the_tokens_with_no_earlier_owner_meet_the_pending_stop() {
    for refused in [true, false] {
        let limit = if refused { 8 } else { 10000 };
        // Both tokens are owned before the stop.
        let (c, t, data, h, _) = stop_after_table("h h", refused);
        let report = query(&c, &t, limit, small_limits()).unwrap();
        let found: Vec<_> = model(&report)
            .headers()
            .iter()
            .map(|e| (e.cell().clone(), e.header().clone()))
            .collect();
        assert_eq!(
            found,
            vec![(id(&c, &data), id(&c, &h))],
            "refused={refused}"
        );
        // The second token has none, in either order.
        for tokens in ["h none", "none h"] {
            let (c, t, _, _, stop) = stop_after_table(tokens, refused);
            match query(&c, &t, limit, small_limits()) {
                Err(failure) => {
                    assert!(refused, "{tokens}");
                    assert_refused_id(&failure);
                }
                Ok(report) => {
                    assert!(!refused, "{tokens}");
                    assert_eq!(
                        report.results()[0].fact(),
                        &HtmlSemanticFact::TableModel(HtmlSemanticValue::Missing(
                            HtmlSemanticMissing::OrderedChildren(id(&c, &stop))
                        )),
                        "{tokens}"
                    );
                }
            }
        }
    }
}

/// A `th` with an ID of exactly these UTF-16 units.
fn header_with_id_units(b: &mut HtmlCaptureBuilder, d: &Subject, units: &[u16]) -> Subject {
    let attrs = OrderedAttributes::new([
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, "id"),
            DocumentString::from_utf16(units.to_vec()),
        ),
        Attribute::new(QualifiedName::new(Namespace::none(), None, "headers"), ""),
    ])
    .unwrap();
    let e = b
        .add_element(
            d,
            QualifiedName::new(Namespace::html(), None, "th"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let t = b.add_text(d, DocumentString::from("x")).unwrap();
    b.establish_children(&e, &[t]).unwrap();
    e
}

/// A `td` whose `headers` value is exactly these UTF-16 units.
fn data_naming_units(b: &mut HtmlCaptureBuilder, d: &Subject, units: &[u16]) -> Subject {
    let attrs = OrderedAttributes::new([Attribute::new(
        QualifiedName::new(Namespace::none(), None, "headers"),
        DocumentString::from_utf16(units.to_vec()),
    )])
    .unwrap();
    let e = b
        .add_element(
            d,
            QualifiedName::new(Namespace::html(), None, "td"),
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let t = b.add_text(d, DocumentString::from("x")).unwrap();
    b.establish_children(&e, &[t]).unwrap();
    e
}

/// For each (token units, expected owner) a data cell naming the token is
/// associated with the header of the expected ID alone, or with none: IDs match
/// by exact UTF-16 code units.
fn assert_tokens_select(ids: &[&[u16]], tokens: &[(&[u16], Option<usize>)]) {
    let (mut b, d) = builder();
    let headers: Vec<_> = ids
        .iter()
        .map(|units| header_with_id_units(&mut b, &d, units))
        .collect();
    let data: Vec<_> = tokens
        .iter()
        .map(|(units, _)| data_naming_units(&mut b, &d, units))
        .collect();
    let all: Vec<_> = headers.iter().chain(&data).cloned().collect();
    let t = table(&mut b, &d, &all);
    b.establish_children(&d, std::slice::from_ref(&t)).unwrap();
    let c = b.finish().unwrap();
    let expected: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter_map(|(at, (_, owner))| Some((id(&c, &data[at]), id(&c, &headers[(*owner)?]))))
        .collect();
    assert_eq!(edges(&c, &t), expected);
}

const LOWER: u16 = 0x68;
const UPPER: u16 = 0x48;
const ONE: u16 = 0x31;

#[test]
fn ids_differing_only_in_case_are_different_ids() {
    assert_tokens_select(
        &[&[LOWER], &[UPPER]],
        &[
            (&[UPPER], Some(1)),
            (&[LOWER], Some(0)),
            (&[UPPER], Some(1)),
        ],
    );
}

#[test]
fn an_id_that_is_a_prefix_of_another_does_not_stand_for_it() {
    assert_tokens_select(
        &[&[LOWER, ONE], &[LOWER]],
        &[(&[LOWER], Some(1)), (&[LOWER, ONE], Some(0))],
    );
    // A token that only begins an ID, or extends one, names nothing.
    assert_tokens_select(
        &[&[LOWER, ONE]],
        &[(&[LOWER], None), (&[LOWER, ONE, ONE], None)],
    );
}

#[test]
fn ids_match_by_exact_utf16_units_including_surrogates() {
    // U+1F600 is the pair d83d de00; a lone d83d and a lone de00 are other IDs.
    assert_tokens_select(
        &[&[0xd83d, 0xde00], &[0xd83d], &[0xde00], &[0xd83d, 0xde01]],
        &[
            (&[0xde00], Some(2)),
            (&[0xd83d, 0xde00], Some(0)),
            (&[0xd83d], Some(1)),
            (&[0xd83d, 0xde01], Some(3)),
        ],
    );
}

// ---- Cleanup ------------------------------------------------------------

/// Runs the query of a 3 by 3 grid under the given work and scratch caps,
/// returning its result and the session's usage afterwards.
fn capped(
    c: &HtmlCapture,
    t: &Subject,
    work: usize,
    scratch: usize,
) -> (
    Result<HtmlSemanticReport, HtmlSemanticFailure>,
    HtmlSemanticUsage,
) {
    let cx = context(c);
    let mut session = HtmlSemanticSession::new(
        c,
        &cx,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(work, 100, scratch, 10000, usize::MAX / 4, 10_000_000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let result = session.query(&[HtmlSemanticRequest::new(
        c.subject_id(t).unwrap(),
        HtmlSemanticQuery::TableModel,
    )]);
    (result, session.usage())
}

fn assert_stops_on(
    result: Result<HtmlSemanticReport, HtmlSemanticFailure>,
    expected: HtmlSemanticResource,
    cap: usize,
) {
    assert!(
        matches!(
            result.unwrap_err().kind(),
            HtmlSemanticFailureKind::LimitExceeded { resource, .. } if *resource == expected
        ),
        "{expected:?} cap {cap}"
    );
}

#[test]
fn a_scratch_cap_stopping_the_index_build_releases_its_scratch_records() {
    let (c, t, _) = grid(3);
    let baseline = query(&c, &t, 10000, work_limits()).unwrap();
    let peak = baseline.usage().peak_scratch_records();
    assert!(peak > 0);
    // Every scratch cap below the peak stops the query on scratch records: in
    // the axes, the walk, the entries or the cells. Each stop, including those
    // inside the index build, releases all of it.
    for cap in 0..peak {
        let (result, usage) = capped(&c, &t, usize::MAX / 4, cap);
        assert_stops_on(result, HtmlSemanticResource::ScratchRecords, cap);
        assert_eq!(usage.scratch_records(), 0, "scratch cap {cap}");
    }
    let (result, usage) = capped(&c, &t, usize::MAX / 4, peak);
    assert_eq!(result.unwrap().results(), baseline.results());
    assert_eq!(usage.scratch_records(), 0);
}

#[test]
fn a_work_cap_stopping_the_index_build_releases_its_scratch_records() {
    let (c, t, _) = grid(3);
    let baseline = query(&c, &t, 10000, work_limits()).unwrap();
    let work = baseline.usage().work();
    // Every work cap below the total stops the query on work, at any point of
    // the build, among them the charge for the sort after the walk, and
    // leaves no scratch records behind.
    for cap in 0..work {
        let (result, usage) = capped(&c, &t, cap, usize::MAX / 4);
        assert_stops_on(result, HtmlSemanticResource::Work, cap);
        assert_eq!(usage.scratch_records(), 0, "work cap {cap}");
    }
    let (result, usage) = capped(&c, &t, work, usize::MAX / 4);
    assert_eq!(result.unwrap().results(), baseline.results());
    assert_eq!(usage.scratch_records(), 0);
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
