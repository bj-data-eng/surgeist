//! A session that charges a host's operation: the scopes it creates, the usage the host reads,
//! the refusals a host cap reports and the cleanup of every path.

use crate::semantics::support::{builder, context, semantic_limits};
use surgeist_html::{
    accounting::{
        HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlHostScope, HtmlLimits,
        HtmlResource, HtmlWork, aggregate_tariffs,
    },
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCapturedIsValue, HtmlSemanticCancellation, HtmlSemanticFailureKind, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource,
    HtmlSemanticSession,
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
    accounting: &HtmlHostScope,
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
    let accounting = HtmlHostScope::new(ledger.clone(), root);
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
    assert_eq!(
        names,
        [
            "host",
            HtmlCapScope::Query.name(),
            HtmlAuxiliaryScope::QueryObserved.name()
        ]
    );
    assert_eq!(names[1..], ["html-query", "html-query-observed"]);
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
        let accounting = HtmlHostScope::new(ledger.clone(), root);
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
    let accounting = HtmlHostScope::new(ledger.clone(), root);
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
    let accounting = HtmlHostScope::new(ledger, foreign);
    let failure = open(&c, &cx, semantic_limits(), &accounting)
        .err()
        .expect("a foreign scope is refused");
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Accounting(HtmlAccountingFault::ForeignScope)
    );
    assert!(failure.qualification().is_none());
}

#[test]
fn a_scope_bound_without_room_for_the_session_is_an_accounting_fault() {
    let (c, _) = table();
    let cx = context(&c);
    let (ledger, root) = host(HtmlLimits::unlimited(), 2);
    let accounting = HtmlHostScope::new(ledger, root);
    let failure = open(&c, &cx, semantic_limits(), &accounting)
        .err()
        .expect("the scope bound is exhausted");
    assert_eq!(
        failure.kind(),
        &HtmlSemanticFailureKind::Accounting(HtmlAccountingFault::ScopeLimit { limit: 2 })
    );
}

mod capture {
    use super::*;
    use surgeist_html::{
        bounded::construction::HtmlConstructionResult,
        construction::{
            HtmlConstructionLimits, HtmlDocument, HtmlDocumentMode, HtmlNodeId, HtmlNodeKind,
        },
        values::DocumentString,
    };
    use surgeist_html_semantics::{
        HtmlCaptureError, HtmlCaptureLimits, HtmlCaptureResource, HtmlCapturedIsValue,
        HtmlConstructionCaptureAdapter, HtmlConstructionCaptureDocument,
        HtmlConstructionCaptureDocumentFacts, HtmlConstructionCaptureElementFacts,
        HtmlConstructionCaptureError, HtmlConstructionCaptureFacts, HtmlConstructionCaptureLimits,
        HtmlConstructionCaptureNodeFacts, HtmlHostFactsRevision, HtmlScriptingStatus,
    };

    fn limits() -> HtmlCaptureLimits {
        HtmlCaptureLimits::new(200, 2000, 200, 100_000, 200, 10, 10, 16_777_216)
    }

    fn hosted(
        capture_limits: HtmlCaptureLimits,
        scopes: usize,
    ) -> (HtmlCaptureBuilder, Ledger, Scope) {
        hosted_at(capture_limits, scopes, HtmlHostFactsRevision::fresh())
    }

    fn hosted_at(
        capture_limits: HtmlCaptureLimits,
        scopes: usize,
        revision: HtmlHostFactsRevision,
    ) -> (HtmlCaptureBuilder, Ledger, Scope) {
        let (ledger, root) = host(HtmlLimits::unlimited(), scopes);
        let accounting = HtmlHostScope::new(ledger.clone(), root);
        let builder =
            HtmlCaptureBuilder::with_accounting(capture_limits, revision, &accounting).unwrap();
        (builder, ledger, root)
    }

    fn add_document(builder: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
        builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                surgeist_html_semantics::HtmlScriptingStatus::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap()
    }

    /// A document with a `div` holding a text, and the facts of its nodes.
    fn source(revision: &HtmlHostFactsRevision) -> (HtmlDocument, HtmlConstructionCaptureFacts) {
        let mut native = HtmlDocument::builder(
            HtmlConstructionLimits::new(100, 200, 100, 10_000, 100),
            HtmlDocumentMode::NoQuirks,
        )
        .unwrap();
        let root = native.root();
        let div = native
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
            )
            .unwrap();
        native
            .append_text(&div, DocumentString::from("text"))
            .unwrap();
        let document = native.finish().unwrap();
        let result = HtmlConstructionResult::Document(document.clone());
        let mut ids: Vec<HtmlNodeId> = vec![document.root().id()];
        let mut pending = ids.clone();
        while let Some(id) = pending.pop() {
            for child in result.node(&id).unwrap().children() {
                ids.push(child);
                pending.push(child);
            }
        }
        let rows = ids
            .into_iter()
            .map(|id| {
                let kind = result.node(&id).unwrap();
                let document_facts = matches!(kind.kind(), HtmlNodeKind::Document(_)).then(|| {
                    HtmlConstructionCaptureDocumentFacts::new(
                        HtmlScriptingStatus::Disabled,
                        HtmlCaptureRegistryInput::Unavailable,
                    )
                });
                let element_facts = matches!(kind.kind(), HtmlNodeKind::Element(_)).then(|| {
                    HtmlConstructionCaptureElementFacts::new(
                        HtmlCapturedIsValue::Unavailable,
                        HtmlCaptureRegistryInput::Unavailable,
                    )
                });
                HtmlConstructionCaptureNodeFacts::new(
                    id,
                    HtmlConstructionCaptureDocument::ResultDocument,
                    document_facts,
                    element_facts,
                    None,
                )
            })
            .collect();
        (
            document,
            HtmlConstructionCaptureFacts::new(revision.clone(), rows),
        )
    }

    #[test]
    fn a_hosted_builder_charges_the_capture_scope_and_the_host_sees_the_counts() {
        let (mut builder, ledger, root) = hosted(limits(), 4);
        let document = add_document(&mut builder);
        let element = html(&mut builder, &document, "div");
        builder.establish_children(&document, &[element]).unwrap();
        let usage = builder.usage();
        assert_eq!(
            (usage.subjects(), usage.relationships(), usage.depth()),
            (2, 4, 1)
        );
        let report = report(&ledger);
        assert_eq!(report.scopes.len(), 2);
        let capture = scope(&report, "html-capture");
        assert_eq!(capture.parent, Some(root));
        for (resource, value) in [
            (HtmlResource::CapturedSubjects, usage.subjects()),
            (HtmlResource::CapturedRelationships, usage.relationships()),
            (HtmlResource::CapturedUtf16Units, usage.utf16_units()),
            (HtmlResource::ElementValueBytes, usage.element_value_bytes()),
        ] {
            assert_eq!(capture.usage.live(resource), value as u64, "{resource:?}");
            assert_eq!(
                report.scopes[0].usage.live(resource),
                value as u64,
                "{resource:?}"
            );
        }
        // A gauge stays in the scope that observed it.
        assert_eq!(
            capture.usage.live(HtmlResource::CapturedDepth),
            usage.depth() as u64
        );
        assert_eq!(report.scopes[0].usage.live(HtmlResource::CapturedDepth), 0);
    }

    #[test]
    fn a_host_cap_refuses_a_capture_with_the_host_limit_and_the_value_it_would_reach() {
        let caps = HtmlLimits::from_fn(
            |_| u64::MAX,
            |resource| match resource {
                HtmlResource::CapturedSubjects => 1,
                _ => u64::MAX,
            },
        );
        let (ledger, root) = host(caps, 4);
        let accounting = HtmlHostScope::new(ledger, root);
        let mut builder = HtmlCaptureBuilder::with_accounting(
            limits(),
            HtmlHostFactsRevision::fresh(),
            &accounting,
        )
        .unwrap();
        let document = add_document(&mut builder);
        let before = builder.usage();
        let refused = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::new([]).unwrap(),
                HtmlCapturedIsValue::Unavailable,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap_err();
        assert_eq!(
            refused,
            HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::Subjects,
                limit: 1,
                required: 2,
            }
        );
        assert_eq!(builder.usage(), before);
    }

    #[test]
    fn the_builder_depth_cap_is_a_gauge_that_refuses_at_its_value() {
        let tight = HtmlCaptureLimits::new(200, 2000, 200, 100_000, 1, 10, 10, 16_777_216);
        let (mut builder, _, _) = hosted(tight, 4);
        let document = add_document(&mut builder);
        let a = html(&mut builder, &document, "a");
        let b = html(&mut builder, &document, "b");
        builder
            .establish_children(&document, std::slice::from_ref(&a))
            .unwrap();
        let before = builder.usage();
        assert_eq!(before.depth(), 1);
        let refused = builder
            .establish_children(&a, std::slice::from_ref(&b))
            .unwrap_err();
        assert_eq!(
            refused,
            HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::Depth,
                limit: 1,
                required: 2,
            }
        );
        assert_eq!(builder.usage(), before);
    }

    #[test]
    fn a_scope_of_another_operation_is_an_accounting_fault_for_a_builder() {
        let (ledger, _) = host(HtmlLimits::unlimited(), 4);
        let (_, foreign) = host(HtmlLimits::unlimited(), 4);
        let accounting = HtmlHostScope::new(ledger, foreign);
        let error = HtmlCaptureBuilder::with_accounting(
            limits(),
            HtmlHostFactsRevision::fresh(),
            &accounting,
        )
        .expect_err("a foreign scope is refused");
        assert_eq!(error, HtmlAccountingFault::ForeignScope);
    }

    #[test]
    fn a_hosted_import_charges_its_own_scope_and_a_refused_import_gives_back_the_capture() {
        let revision = HtmlHostFactsRevision::fresh();
        let (mut builder, ledger, root) = hosted_at(limits(), 4, revision.clone());
        add_document(&mut builder);
        let before = builder.usage();
        let (document, facts) = source(&revision);
        // The import's own mapping cap refuses the third node.
        let refused = HtmlConstructionCaptureAdapter::document(
            builder,
            document.clone(),
            facts,
            HtmlConstructionCaptureLimits::new(2, 1000, 1_000_000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
        assert_eq!(
            refused.reason(),
            &HtmlConstructionCaptureError::LimitExceeded
        );
        assert_eq!(refused.usage().mapping_entries(), 2);
        assert_eq!(refused.builder().usage(), before);
        let report = report(&ledger);
        let import = scope(&report, "html-capture-import");
        assert_eq!(import.parent, Some(root));
        assert_eq!(import.usage.live(HtmlResource::MappingEntries), 2);
        assert_eq!(import.usage.live(HtmlResource::ScratchRecords), 0);
        // The capture's subjects are given back with the refused import.
        assert_eq!(
            report.scopes[0].usage.live(HtmlResource::CapturedSubjects),
            before.subjects() as u64
        );
    }

    #[test]
    fn a_finished_import_leaves_no_scratch_and_reports_the_work_and_peak_it_charged() {
        let (ledger, root) = host(HtmlLimits::unlimited(), 4);
        let accounting = HtmlHostScope::new(ledger.clone(), root);
        let revision = HtmlHostFactsRevision::fresh();
        let builder =
            HtmlCaptureBuilder::with_accounting(limits(), revision.clone(), &accounting).unwrap();
        let (document, facts) = source(&revision);
        let import = HtmlConstructionCaptureAdapter::document(
            builder,
            document,
            facts,
            HtmlConstructionCaptureLimits::new(100, 1000, 1_000_000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let usage = import.usage();
        assert_eq!(usage.mapping_entries(), 3);
        assert!(usage.work() > 0 && usage.scratch_entries() > 0);
        let report = report(&ledger);
        let import = scope(&report, "html-capture-import");
        assert_eq!(
            import.usage.work(HtmlWork::SemanticStep),
            usage.work() as u64
        );
        assert_eq!(
            import.usage.peak(HtmlResource::ScratchRecords),
            usage.scratch_entries() as u64
        );
        assert_eq!(import.usage.live(HtmlResource::ScratchRecords), 0);
    }
}

/// A builder opens its capped scope in a host operation whose scope bound it spends: the host's
/// bound is the one accounting fault, as it is for a parse and a session.
#[test]
fn a_scope_bound_too_small_for_a_builder_is_an_accounting_fault() {
    let (ledger, root) = host(HtmlLimits::unlimited(), 1);
    let accounting = HtmlHostScope::new(ledger, root);
    let error = HtmlCaptureBuilder::with_accounting(
        surgeist_html_semantics::HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0, 16_777_216),
        surgeist_html_semantics::HtmlHostFactsRevision::fresh(),
        &accounting,
    )
    .expect_err("the host has no room for a scope");
    assert_eq!(error, HtmlAccountingFault::ScopeLimit { limit: 1 });
}

/// The scopes a builder creates for a capture take their names from the schema.
#[test]
fn a_hosted_builder_names_its_scope_by_the_schema() {
    let (ledger, root) = host(HtmlLimits::unlimited(), 4);
    let accounting = HtmlHostScope::new(ledger.clone(), root);
    let _builder = HtmlCaptureBuilder::with_accounting(
        surgeist_html_semantics::HtmlCaptureLimits::new(10, 100, 10, 1000, 10, 0, 0, 16_777_216),
        surgeist_html_semantics::HtmlHostFactsRevision::fresh(),
        &accounting,
    )
    .unwrap();
    let names: Vec<_> = report(&ledger).scopes.iter().map(|s| s.name).collect();
    assert_eq!(names, ["host", HtmlCapScope::Capture.name()]);
}

/// A `time` element whose `datetime` attribute is parsed by the numeric owner, and the request
/// that reads it.
fn time() -> (HtmlCapture, HtmlSemanticRequest) {
    let (mut b, d) = builder();
    let time = b
        .add_element(
            &d,
            QualifiedName::new(Namespace::html(), None, "time"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "datetime"),
                "2011-11-18",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&time, &[]).unwrap();
    b.establish_root(&time).unwrap();
    let capture = b.finish().unwrap();
    let request = HtmlSemanticRequest::new(
        capture.subject_id(&time).unwrap(),
        HtmlSemanticQuery::TimeElement,
    );
    (capture, request)
}

fn time_limits(bytes: usize) -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(100_000_000, 100, 1_000, 10_000, bytes, 1_000_000)
}

/// A host cap on the allocated bytes tighter than the session's own limit bounds the numeric
/// materialization as a session limit of that size does: the same stop, with the same usage,
/// because the numeric owner refuses before it allocates beyond the cap. The cap sits on the
/// supplied scope, or on its parent so that the supplied scope reaches it through a tariff.
#[test]
fn a_host_byte_cap_bounds_a_numeric_materialization_as_a_session_limit_of_that_size_does() {
    let (c, request) = time();
    let cx = context(&c);
    let requests = [request];
    let baseline = private(&c, &cx, time_limits(10_000_000))
        .query(&requests)
        .unwrap()
        .usage()
        .allocated_bytes();
    assert!(baseline > 0);
    let capped = |cap: usize| {
        HtmlLimits::from_fn(
            |_| u64::MAX,
            |resource| {
                if resource == HtmlResource::AllocatedBytes {
                    cap as u64
                } else {
                    u64::MAX
                }
            },
        )
    };
    for cap in 0..baseline {
        let expected = private(&c, &cx, time_limits(cap))
            .query(&requests)
            .unwrap_err();
        for below_root in [false, true] {
            let (ledger, root) = host(capped(cap), 8);
            let scope = if below_root {
                ledger
                    .child(root, "tenant", HtmlLimits::unlimited(), aggregate_tariffs())
                    .unwrap()
            } else {
                root
            };
            let accounting = HtmlHostScope::new(ledger, scope);
            let mut session = open(&c, &cx, time_limits(10_000_000), &accounting).unwrap();
            let failure = session.query(&requests).unwrap_err();
            assert_eq!(
                failure.kind(),
                expected.kind(),
                "host cap {cap} below root {below_root}"
            );
            assert_eq!(
                failure.usage(),
                expected.usage(),
                "host cap {cap} below root {below_root}"
            );
        }
    }
}
