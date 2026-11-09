//! A session that charges a host's operation: the scopes it creates, the usage the host reads,
//! the refusals a host cap reports and the cleanup of every path.

use crate::semantics::support::{builder, context, semantic_limits};
use surgeist_html::{
    accounting::{HtmlLimits, HtmlResource, HtmlWork},
    values::{Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCapturedIsValue, HtmlSemanticAccounting, HtmlSemanticAccountingFault,
    HtmlSemanticCancellation, HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
};
use surgeist_ledger::{Operation, ProfileMode, Report, Scope, SharedOperation};

type Ledger = SharedOperation<HtmlWork, HtmlResource>;

fn html(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
) -> HtmlCaptureBuilderSubject {
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        OrderedAttributes::new([]).unwrap(),
        HtmlCapturedIsValue::Unavailable,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}

/// A table of two bodies with two rows of two cells each, and the requests that read it.
fn table() -> (HtmlCapture, Vec<HtmlSemanticRequest>) {
    let (mut b, d) = builder();
    let table = html(&mut b, &d, "table");
    let mut bodies = Vec::new();
    let mut cells = Vec::new();
    let mut rows = Vec::new();
    for _ in 0..2 {
        let body = html(&mut b, &d, "tbody");
        let mut body_rows = Vec::new();
        for _ in 0..2 {
            let row = html(&mut b, &d, "tr");
            let row_cells = [html(&mut b, &d, "td"), html(&mut b, &d, "td")];
            b.establish_children(&row, &row_cells).unwrap();
            cells.extend(row_cells);
            rows.push(row.clone());
            body_rows.push(row);
        }
        b.establish_children(&body, &body_rows).unwrap();
        bodies.push(body);
    }
    b.establish_children(&table, &bodies).unwrap();
    b.establish_root(&table).unwrap();
    let capture = b.finish().unwrap();
    let id = |h: &HtmlCaptureBuilderSubject| capture.subject_id(h).unwrap();
    let mut requests = vec![
        HtmlSemanticRequest::new(id(&table), HtmlSemanticQuery::TableModel),
        HtmlSemanticRequest::new(id(&table), HtmlSemanticQuery::TableRows),
    ];
    requests.extend(
        rows.iter()
            .map(|row| HtmlSemanticRequest::new(id(row), HtmlSemanticQuery::TableRowCells)),
    );
    (capture, requests)
}

/// A host operation that profiles in summary mode, so its report lists every scope's usage.
fn host(caps: HtmlLimits, scopes: usize) -> (Ledger, Scope) {
    let operation =
        Operation::with_profile("host", caps, scopes, ProfileMode::Summary { timing: false })
            .unwrap();
    let root = operation.root();
    (operation.share(), root)
}

fn open<'a>(
    c: &'a HtmlCapture,
    cx: &'a surgeist_html_semantics::HtmlSemanticContext,
    limits: HtmlSemanticLimits,
    accounting: &HtmlSemanticAccounting,
) -> Result<HtmlSemanticSession<'a>, surgeist_html_semantics::HtmlSemanticFailure> {
    HtmlSemanticSession::with_accounting(
        c,
        cx,
        surgeist_html::values::HtmlValueInterpreter::new(100),
        limits,
        HtmlSemanticCancellation::new(),
        accounting,
    )
}

fn private<'a>(
    c: &'a HtmlCapture,
    cx: &'a surgeist_html_semantics::HtmlSemanticContext,
    limits: HtmlSemanticLimits,
) -> HtmlSemanticSession<'a> {
    HtmlSemanticSession::new(
        c,
        cx,
        surgeist_html::values::HtmlValueInterpreter::new(100),
        limits,
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
}

fn report(ledger: &Ledger) -> Report<HtmlWork, HtmlResource> {
    ledger.snapshot().unwrap().unwrap()
}

fn scope<'a>(
    report: &'a Report<HtmlWork, HtmlResource>,
    name: &str,
) -> &'a surgeist_ledger::ScopeReport<HtmlWork, HtmlResource> {
    report
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap_or_else(|| panic!("no scope {name}"))
}

fn queried(
    c: &HtmlCapture,
    requests: &[HtmlSemanticRequest],
    limits: HtmlSemanticLimits,
) -> (HtmlSemanticReport, Ledger, Scope) {
    let cx = context(c);
    let (ledger, root) = host(HtmlLimits::unlimited(), 8);
    let accounting = HtmlSemanticAccounting::host(ledger.clone(), root);
    let report = open(c, &cx, limits, &accounting)
        .unwrap()
        .query(requests)
        .unwrap();
    (report, ledger, root)
}

#[test]
fn a_host_session_reports_what_a_private_session_reports() {
    let (c, requests) = table();
    let cx = context(&c);
    let limits = semantic_limits();
    let expected = private(&c, &cx, limits).query(&requests).unwrap();
    let (hosted, _, _) = queried(&c, &requests, limits);
    assert_eq!(hosted.results(), expected.results());
    assert_eq!(hosted.usage(), expected.usage());
    assert!(hosted.usage().peak_scratch_records() > 0);
}

#[test]
fn a_session_creates_the_capped_and_the_observed_scope_below_the_host_scope() {
    let (c, requests) = table();
    let (_, ledger, root) = queried(&c, &requests, semantic_limits());
    let report = report(&ledger);
    let names: Vec<_> = report.scopes.iter().map(|scope| scope.name).collect();
    assert_eq!(names, ["host", "html-query", "html-query-observed"]);
    assert_eq!(report.scopes[1].parent, Some(root));
    assert_eq!(report.scopes[2].parent, Some(root));
}

#[test]
fn the_host_sees_the_admitted_amounts_once_and_the_observed_scope_the_actual_ones() {
    let (c, requests) = table();
    let (hosted, ledger, _) = queried(&c, &requests, semantic_limits());
    let usage = hosted.usage();
    let report = report(&ledger);
    let host = &report.scopes[0];
    let query = scope(&report, "html-query");
    let observed = scope(&report, "html-query-observed");
    assert_eq!(host.usage.work(HtmlWork::SemanticStep), usage.work() as u64);
    assert_eq!(
        query.usage.work(HtmlWork::SemanticStep),
        usage.work() as u64
    );
    assert_eq!(
        host.usage.live(HtmlResource::OutputRecords),
        usage.output_records() as u64
    );
    assert_eq!(
        host.usage.live(HtmlResource::AllocatedBytes),
        usage.allocated_bytes() as u64
    );
    assert_eq!(
        host.usage.peak(HtmlResource::ScratchRecords),
        usage.peak_scratch_records() as u64
    );
    // The observed scope projects nothing into the host.
    assert_eq!(observed.usage.work(HtmlWork::SemanticStep), 0);
    assert_eq!(
        observed.usage.live(HtmlResource::OutputRecords),
        usage.output_records() as u64
    );
    assert_eq!(
        observed.usage.peak(HtmlResource::ScratchRecords),
        usage.peak_scratch_records() as u64
    );
}

#[test]
fn a_finished_session_leaves_no_scratch_in_any_scope() {
    let (c, requests) = table();
    let (_, ledger, _) = queried(&c, &requests, semantic_limits());
    for scope in report(&ledger).scopes {
        assert_eq!(
            scope.usage.live(HtmlResource::ScratchRecords),
            0,
            "{}",
            scope.name
        );
    }
}

/// The limits that stop a query after the work or resource `cap`.
fn capped(resource: HtmlSemanticResource, cap: usize) -> HtmlSemanticLimits {
    let base = semantic_limits();
    let (work, output, scratch, bytes, nested) = (
        base.work(),
        base.output_records(),
        base.scratch_records(),
        base.allocated_bytes(),
        base.nested_output_records(),
    );
    let pick = |this, value| if resource == this { cap } else { value };
    HtmlSemanticLimits::new(
        pick(HtmlSemanticResource::Work, work),
        pick(HtmlSemanticResource::OutputRecords, output),
        pick(HtmlSemanticResource::ScratchRecords, scratch),
        base.integer_digits(),
        pick(HtmlSemanticResource::AllocatedBytes, bytes),
        pick(HtmlSemanticResource::NestedOutputRecords, nested),
    )
}

#[test]
fn every_stop_leaves_no_scratch_and_the_same_failure_in_a_host_and_a_private_session() {
    let (c, requests) = table();
    let cx = context(&c);
    let baseline = private(&c, &cx, semantic_limits())
        .query(&requests)
        .unwrap();
    let usage = baseline.usage();
    let mut stops = 0;
    for (resource, exact) in [
        (HtmlSemanticResource::Work, usage.work()),
        (HtmlSemanticResource::OutputRecords, usage.output_records()),
        (
            HtmlSemanticResource::ScratchRecords,
            usage.peak_scratch_records(),
        ),
        (
            HtmlSemanticResource::AllocatedBytes,
            usage.allocated_bytes(),
        ),
        (
            HtmlSemanticResource::NestedOutputRecords,
            usage.nested_output_records(),
        ),
    ] {
        if exact == 0 {
            continue;
        }
        let limits = capped(resource, exact - 1);
        let expected = private(&c, &cx, limits).query(&requests).unwrap_err();
        let (ledger, root) = host(HtmlLimits::unlimited(), 8);
        let accounting = HtmlSemanticAccounting::host(ledger.clone(), root);
        let mut session = open(&c, &cx, limits, &accounting).unwrap();
        let failure = session.query(&requests).unwrap_err();
        assert_eq!(failure.kind(), expected.kind(), "{resource:?}");
        assert_eq!(failure.usage(), expected.usage(), "{resource:?}");
        assert_eq!(session.usage().scratch_records(), 0, "{resource:?}");
        for scope in report(&ledger).scopes {
            assert_eq!(
                scope.usage.live(HtmlResource::ScratchRecords),
                0,
                "{resource:?} {}",
                scope.name
            );
        }
        stops += 1;
    }
    assert_eq!(stops, 5);
}

#[test]
fn a_host_cap_refuses_with_the_host_limit_and_the_value_it_would_have_reached() {
    let (c, requests) = table();
    let cx = context(&c);
    let baseline = private(&c, &cx, semantic_limits())
        .query(&requests)
        .unwrap();
    let cap = baseline.usage().work() as u64 - 1;
    let caps = HtmlLimits::from_fn(
        |work| match work {
            HtmlWork::SemanticStep => cap,
            _ => u64::MAX,
        },
        |_| u64::MAX,
    );
    let (ledger, root) = host(caps, 8);
    let accounting = HtmlSemanticAccounting::host(ledger.clone(), root);
    let failure = open(&c, &cx, semantic_limits(), &accounting)
        .unwrap()
        .query(&requests)
        .unwrap_err();
    let HtmlSemanticFailureKind::LimitExceeded {
        resource,
        limit,
        required,
    } = failure.kind().clone()
    else {
        panic!("{:?}", failure.kind());
    };
    assert_eq!(resource, HtmlSemanticResource::Work);
    assert_eq!(limit as u64, cap);
    assert!(required as u64 > cap);
    // The refusal names the host's value before the refused batch.
    assert_eq!(
        failure.usage().work() as u64,
        report(&ledger).scopes[0].usage.work(HtmlWork::SemanticStep)
    );
    // The same cap on a private session stops the same way.
    let private = private(&c, &cx, capped(HtmlSemanticResource::Work, cap as usize))
        .query(&requests)
        .unwrap_err();
    assert_eq!(failure.kind(), private.kind());
    assert_eq!(failure.usage(), private.usage());
}

#[test]
fn a_scope_of_another_operation_is_an_accounting_fault() {
    let (c, _) = table();
    let cx = context(&c);
    let (ledger, _) = host(HtmlLimits::unlimited(), 8);
    let (_, foreign) = host(HtmlLimits::unlimited(), 8);
    let accounting = HtmlSemanticAccounting::host(ledger, foreign);
    let failure = open(&c, &cx, semantic_limits(), &accounting)
        .err()
        .expect("a foreign scope is refused");
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Accounting(HtmlSemanticAccountingFault::ForeignScope)
    );
    assert!(failure.qualification().is_none());
}

#[test]
fn a_scope_bound_without_room_for_the_session_is_an_accounting_fault() {
    let (c, _) = table();
    let cx = context(&c);
    let (ledger, root) = host(HtmlLimits::unlimited(), 2);
    let accounting = HtmlSemanticAccounting::host(ledger, root);
    let failure = open(&c, &cx, semantic_limits(), &accounting)
        .err()
        .expect("the scope bound is exhausted");
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Accounting(HtmlSemanticAccountingFault::ScopeLimit { limit: 2 })
    );
}
