//! One host operation holding a parse, a native construction, a capture with an import, a
//! semantic session, a fact constructor and a controlled autocomplete materialization.
//!
//! The expectations come from the ledger's documented behavior: a scope's usage is its own units
//! and a parent's usage is the sum of its children's under the tariffs the schema defines, so
//! the host's totals are the sums of the per-scope measurements. The typed usage of each stage
//! and the stage's own refusal are the oracles for its scope.

use super::support::limits;
use surgeist_html::{
    accounting::{
        HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlCustody, HtmlHostScope,
        HtmlLimits, HtmlResource, HtmlWork,
    },
    attributes::HtmlInputType,
    bounded::attributes::{
        HtmlAutocompleteMaterializationError, HtmlAutocompleteMaterializationLimits,
    },
    construction::HtmlConstructionResult,
    construction::{
        HtmlConstructionCause, HtmlConstructionLimits, HtmlConstructionResource, HtmlDocument,
        HtmlDocumentMode, HtmlNodeId, HtmlNodeKind,
    },
    values::{
        Attribute, DocumentString, HtmlValueInterpreter, Namespace, OrderedAttributes,
        QualifiedName,
    },
    vocabulary::{KnownHtmlAttribute, KnownHtmlElement},
};
use surgeist_html_parser::*;
use surgeist_html_semantics::{
    HtmlCaptureBuilder, HtmlCaptureError, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCapturedIsValue, HtmlConstructionCaptureAdapter, HtmlConstructionCaptureDocument,
    HtmlConstructionCaptureDocumentFacts, HtmlConstructionCaptureElementFacts,
    HtmlConstructionCaptureError, HtmlConstructionCaptureFacts, HtmlConstructionCaptureLimits,
    HtmlConstructionCaptureNodeFacts, HtmlConstructionCapturedResult, HtmlDocumentFlatTreeRelation,
    HtmlFlatTreeProducerId, HtmlFlatTreeRevision, HtmlHostFactsRevision, HtmlInertFactError,
    HtmlInertFactLimits, HtmlInertFacts, HtmlInertNodeFact, HtmlScriptingStatus,
    HtmlSemanticCancellation, HtmlSemanticContext, HtmlSemanticContextInput, HtmlSemanticFailure,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession,
};
use surgeist_ledger::{Dimension, Operation, ProfileMode, Report, ScopeReport, SharedOperation};

type Ledger = SharedOperation<HtmlWork, HtmlResource>;

const SOURCE: &str = "<!doctype html><p class=a>one<b>two</b></p><table><tr><td>x</table>";

fn capture_limits() -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(100, 1000, 100, 10_000, 100, 0, 0, 16_777_216)
}

fn import_limits() -> HtmlConstructionCaptureLimits {
    HtmlConstructionCaptureLimits::new(100, 1000, 1_000_000)
}

fn session_limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(1_000_000, 100, 1_000, 10, 1_000_000, 10_000)
}

fn autocomplete_limits() -> HtmlAutocompleteMaterializationLimits {
    HtmlAutocompleteMaterializationLimits::new(100_000, 100_000, 100_000)
}

/// A host operation with `scopes` scopes, profiled so that its report lists every scope's usage.
fn host(caps: HtmlLimits, scopes: usize) -> (Ledger, HtmlHostScope) {
    let operation =
        Operation::with_profile("host", caps, scopes, ProfileMode::Summary { timing: false })
            .unwrap();
    let root = operation.root();
    let ledger = operation.share();
    let scope = HtmlHostScope::new(ledger.clone(), root);
    (ledger, scope)
}

fn report(ledger: &Ledger) -> Report<HtmlWork, HtmlResource> {
    ledger.snapshot().unwrap().unwrap()
}

fn scope<'a>(
    report: &'a Report<HtmlWork, HtmlResource>,
    name: &str,
) -> &'a ScopeReport<HtmlWork, HtmlResource> {
    report
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap_or_else(|| panic!("no scope named {name}"))
}

fn html_name(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, local)
}

// ---------------------------------------------------------------------------------------------
// The stages. Each charges the host scope it is given and returns what it produced, so a test can
// keep the value alive or drop it.

fn parse_request() -> HtmlDocumentParseRequest {
    HtmlDocumentParseRequest::offline(HtmlSource::utf8(SOURCE), limits())
}

#[expect(
    clippy::result_large_err,
    reason = "a stopped parse hands back its complete original owners"
)]
fn parse(host: &HtmlHostScope) -> Result<HtmlParseReport, HtmlParseFailure> {
    parse_request()
        .with_accounting(HtmlParseAccounting::host(host.clone()))
        .parse()
}

/// A document of a `div` holding a text.
fn construct(
    host: &HtmlHostScope,
) -> Result<HtmlDocument, surgeist_html::construction::HtmlConstructionError> {
    let mut native = HtmlDocument::builder_with_accounting(
        HtmlConstructionLimits::new(100, 200, 100, 10_000, 100),
        HtmlDocumentMode::NoQuirks,
        host,
    )?;
    let root = native.root();
    let div = native.append_element(&root, html_name("div"), OrderedAttributes::default())?;
    native.append_text(&div, DocumentString::from("text"))?;
    native.finish()
}

fn facts_of(
    document: &HtmlDocument,
    revision: &HtmlHostFactsRevision,
) -> HtmlConstructionCaptureFacts {
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
    HtmlConstructionCaptureFacts::new(revision.clone(), rows)
}

/// A refusal of the capture stage: its builder could not open its scope, or its import stopped.
#[derive(Debug)]
enum CaptureStop {
    Builder(HtmlAccountingFault),
    Import(HtmlConstructionCaptureError),
}

fn capture(
    host: &HtmlHostScope,
    document: &HtmlDocument,
) -> Result<HtmlConstructionCapturedResult, CaptureStop> {
    let revision = HtmlHostFactsRevision::fresh();
    let builder = HtmlCaptureBuilder::with_accounting(capture_limits(), revision.clone(), host)
        .map_err(CaptureStop::Builder)?;
    let import = HtmlConstructionCaptureAdapter::document(
        builder,
        document.clone(),
        facts_of(document, &revision),
        import_limits(),
        HtmlSemanticCancellation::new(),
    )
    .map_err(|failure| CaptureStop::Import(failure.reason().clone()))?;
    import
        .finish()
        .map_err(|failure| CaptureStop::Import(failure.reason().clone()))
}

fn context_of(captured: &HtmlConstructionCapturedResult) -> HtmlSemanticContext {
    let capture = captured.capture();
    HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(Vec::new()),
        0,
    )
    .unwrap()
}

fn session(
    host: &HtmlHostScope,
    captured: &HtmlConstructionCapturedResult,
    document: &HtmlDocument,
) -> Result<HtmlSemanticReport, HtmlSemanticFailure> {
    let context = context_of(captured);
    let div = document.root().children().next().unwrap();
    let subject = captured
        .mapping()
        .subject(&div, captured.capture())
        .unwrap();
    HtmlSemanticSession::with_accounting(
        captured.capture(),
        &context,
        HtmlValueInterpreter::new(100),
        session_limits(),
        HtmlSemanticCancellation::new(),
        host,
    )?
    .query(&[HtmlSemanticRequest::new(subject, HtmlSemanticQuery::Hidden)])
}

fn facts(
    host: &HtmlHostScope,
    captured: &HtmlConstructionCapturedResult,
    document: &HtmlDocument,
) -> Result<HtmlInertFacts, HtmlInertFactError> {
    let div = document.root().children().next().unwrap();
    let subject = captured
        .mapping()
        .subject(&div, captured.capture())
        .unwrap();
    let nodes = vec![HtmlInertNodeFact::new(
        subject,
        None,
        HtmlDocumentFlatTreeRelation::Unavailable,
    )];
    let capacity = nodes.capacity();
    HtmlInertFacts::with_accounting(
        captured.capture(),
        captured.capture().host_facts_revision().clone(),
        HtmlFlatTreeRevision::new(HtmlFlatTreeProducerId::fresh(), 7, 12),
        nodes,
        Vec::new(),
        HtmlInertFactLimits::new(capacity, 1_000_000),
        host,
    )
}

fn autocomplete(
    host: &HtmlHostScope,
) -> Result<
    surgeist_html::bounded::attributes::HtmlAutocompleteMaterialized,
    surgeist_html::bounded::attributes::HtmlAutocompleteMaterializationFailure,
> {
    let raw = Attribute::new(
        KnownHtmlAttribute::Autocomplete.qualified_name(),
        "section-a shipping home tel webauthn",
    );
    HtmlValueInterpreter::new(100).control_autocomplete_controlled_with_accounting(
        &KnownHtmlElement::Input.qualified_name(),
        Some(&raw),
        Some(HtmlInputType::Text),
        autocomplete_limits(),
        host,
    )
}

/// Everything the seven stages produce.
struct Everything {
    parse: HtmlParseReport,
    document: HtmlDocument,
    captured: HtmlConstructionCapturedResult,
    session: HtmlSemanticReport,
    facts: HtmlInertFacts,
    autocomplete: surgeist_html::bounded::attributes::HtmlAutocompleteMaterialized,
}

fn everything(host: &HtmlHostScope) -> Everything {
    let parse = parse(host).unwrap();
    let document = construct(host).unwrap();
    let captured = capture(host, &document).unwrap();
    let session = session(host, &captured, &document).unwrap();
    let facts = facts(host, &captured, &document).unwrap();
    let autocomplete = autocomplete(host).unwrap();
    Everything {
        parse,
        document,
        captured,
        session,
        facts,
        autocomplete,
    }
}

/// The scopes a host operation holds after every stage: the host's, then each stage's.
const SCOPE_NAMES: [&str; 12] = [
    "host",
    "html-parse",
    "html-parse-charging",
    "html-construction",
    "html-capture",
    "html-capture-import",
    "html-query",
    "html-query-observed",
    "html-inert-facts",
    "html-fact-grown",
    "html-autocomplete",
    "html-autocomplete-observed",
];

#[test]
fn the_host_holds_the_scopes_of_every_stage_with_names_from_the_schema() {
    let (ledger, host) = host(HtmlLimits::unlimited(), 12);
    let _everything = everything(&host);
    let report = report(&ledger);
    let names: Vec<_> = report.scopes.iter().map(|scope| scope.name).collect();
    assert_eq!(names, SCOPE_NAMES);
    assert_eq!(names[1], HtmlCapScope::Parse.name());
    assert_eq!(names[3], HtmlCapScope::Construction.name());
    assert_eq!(names[4], HtmlCapScope::Capture.name());
    assert_eq!(names[5], HtmlCapScope::CaptureImport.name());
    assert_eq!(names[6], HtmlCapScope::Query.name());
    assert_eq!(names[7], HtmlAuxiliaryScope::QueryObserved.name());
    assert_eq!(names[8], HtmlCapScope::InertFacts.name());
    assert_eq!(names[9], HtmlAuxiliaryScope::FactGrown.name());
    assert_eq!(names[10], HtmlCapScope::Autocomplete.name());
    assert_eq!(names[11], HtmlAuxiliaryScope::AutocompleteObserved.name());
}

#[test]
fn the_host_usage_is_the_sum_of_the_stage_scopes_and_each_stage_matches_its_typed_usage() {
    let (ledger, host) = host(HtmlLimits::unlimited(), 12);
    let everything = everything(&host);
    let report = report(&ledger);
    let root = &report.scopes[0];
    // The scopes that project into the host: its direct children, except the observation scopes
    // that carry scale zero.
    let projecting: Vec<_> = report
        .scopes
        .iter()
        .filter(|scope| {
            scope.parent == Some(host.scope())
                && scope.name != HtmlAuxiliaryScope::QueryObserved.name()
                && scope.name != HtmlAuxiliaryScope::AutocompleteObserved.name()
        })
        .collect();
    assert_eq!(projecting.len(), 7);
    for &work in HtmlWork::ALL {
        let sum: u64 = projecting.iter().map(|s| s.usage.work(work)).sum();
        assert_eq!(root.usage.work(work), sum, "{work:?}");
    }
    for &resource in HtmlResource::ALL {
        let expected: u64 = if resource.custody() == HtmlCustody::Gauge {
            0
        } else {
            projecting.iter().map(|s| s.usage.live(resource)).sum()
        };
        assert_eq!(root.usage.live(resource), expected, "{resource:?}");
    }
    // Each stage's scope matches what the stage reports of itself.
    let credits = everything
        .parse
        .usage()
        .resource(HtmlParseResource::WorkCredits)
        .current();
    assert_eq!(
        scope(&report, "html-parse").usage.work(HtmlWork::Credits),
        credits
    );
    assert_eq!(
        scope(&report, "html-construction")
            .usage
            .live(HtmlResource::ManagedNodes),
        everything.document.usage().nodes() as u64
    );
    assert_eq!(
        scope(&report, "html-query")
            .usage
            .work(HtmlWork::SemanticStep),
        everything.session.usage().work() as u64
    );
    assert_eq!(
        scope(&report, "html-query")
            .usage
            .live(HtmlResource::OutputRecords),
        everything.session.usage().output_records() as u64
    );
    assert_eq!(
        scope(&report, "html-autocomplete")
            .usage
            .live(HtmlResource::AllocatedBytes),
        everything.autocomplete.usage().allocated_bytes() as u64
    );
    assert_eq!(
        scope(&report, "html-capture")
            .usage
            .live(HtmlResource::CapturedSubjects),
        3
    );
    assert_eq!(
        scope(&report, "html-capture-import")
            .usage
            .live(HtmlResource::MappingEntries),
        3
    );
    assert!(
        scope(&report, "html-inert-facts")
            .usage
            .work(HtmlWork::SemanticStep)
            > 0
    );
    let _ = &everything.facts;
    let _ = &everything.captured;
}

#[test]
fn the_host_keeps_the_permanent_charges_and_releases_the_balances_when_the_values_drop() {
    let (ledger, host) = host(HtmlLimits::unlimited(), 12);
    let everything = everything(&host);
    let before = report(&ledger);
    let parse_held: u64 = HtmlResource::ALL
        .iter()
        .filter(|resource| resource.custody() == HtmlCustody::Balance)
        .map(|resource| scope(&before, "html-parse-charging").usage.live(*resource))
        .sum();
    assert!(parse_held > 0, "a report keeps its custody while it lives");
    drop(everything);
    let after = report(&ledger);
    let root = &after.scopes[0];
    for &resource in HtmlResource::ALL {
        let held = |name: &str| scope(&after, name).usage.live(resource);
        // The parse returns every live resource with its report, permanent ones included.
        assert_eq!(held("html-parse"), 0, "{resource:?}");
        assert_eq!(held("html-parse-charging"), 0, "{resource:?}");
        if resource.custody() == HtmlCustody::Gauge {
            continue;
        }
        // The other stages keep what they admitted: the host's total is their sum.
        let kept: u64 = [
            "html-construction",
            "html-capture",
            "html-capture-import",
            "html-query",
            "html-inert-facts",
            "html-autocomplete",
        ]
        .iter()
        .map(|name| held(name))
        .sum();
        assert_eq!(root.usage.live(resource), kept, "{resource:?}");
        if resource.custody() == HtmlCustody::Balance {
            // Only the native construction keeps a balance: it charges storage that its immutable
            // result owns, and nothing returns it.
            assert_eq!(
                root.usage.live(resource),
                held("html-construction"),
                "{resource:?}"
            );
        }
    }
    for scope in &after.scopes {
        assert_eq!(
            scope.usage.live(HtmlResource::ScratchRecords),
            0,
            "{}",
            scope.name
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Caps of the host scope, mapped to each crate's typed stop. A cap is the total of the stages
// before the one under test plus what that stage needs, less one, so the stage refuses at the
// host's cap with the stages before it counted.

/// The usage of the host root after the stages that ran, read from a baseline run.
struct Baseline {
    parse_credits: u64,
    parse_nodes: u64,
    construction_nodes: u64,
    capture_subjects: u64,
    import_mappings: u64,
    import_work: u64,
    session_work: u64,
    fact_work: u64,
    session_bytes: u64,
    autocomplete_bytes: u64,
}

fn baseline() -> Baseline {
    let (ledger, host) = host(HtmlLimits::unlimited(), 12);
    let _everything = everything(&host);
    let report = report(&ledger);
    let usage = |name: &str| scope(&report, name).usage;
    Baseline {
        parse_credits: usage("html-parse").work(HtmlWork::Credits),
        parse_nodes: usage("html-parse").live(HtmlResource::ManagedNodes),
        construction_nodes: usage("html-construction").live(HtmlResource::ManagedNodes),
        capture_subjects: usage("html-capture").live(HtmlResource::CapturedSubjects),
        import_mappings: usage("html-capture-import").live(HtmlResource::MappingEntries),
        import_work: usage("html-capture-import").work(HtmlWork::SemanticStep),
        session_work: usage("html-query").work(HtmlWork::SemanticStep),
        fact_work: usage("html-inert-facts").work(HtmlWork::SemanticStep),
        session_bytes: usage("html-query").live(HtmlResource::AllocatedBytes),
        autocomplete_bytes: usage("html-autocomplete").live(HtmlResource::AllocatedBytes),
    }
}

fn work_cap(dimension: HtmlWork, cap: u64) -> HtmlLimits {
    HtmlLimits::from_fn(
        |work| if work == dimension { cap } else { u64::MAX },
        |_| u64::MAX,
    )
}

fn resource_cap(dimension: HtmlResource, cap: u64) -> HtmlLimits {
    HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| if resource == dimension { cap } else { u64::MAX },
    )
}

#[test]
fn a_host_credit_cap_stops_the_parse_with_the_parse_stop() {
    let base = baseline();
    let cap = base.parse_credits - 1;
    let (ledger, host) = host(work_cap(HtmlWork::Credits, cap), 12);
    let Err(failure) = parse(&host) else {
        panic!("the cap is below the work the parse needs");
    };
    let Some(HtmlParseStop::LimitExceeded {
        resource,
        limit,
        current,
        requested_delta,
    }) = failure.first_stop()
    else {
        panic!("a limit stop, not {:?}", failure.first_stop());
    };
    assert_eq!(resource, HtmlParseResource::WorkCredits);
    assert_eq!(limit, cap);
    assert!(current <= cap && current + requested_delta > cap);
    assert!(ledger.usage(host.scope()).unwrap().work(HtmlWork::Credits) <= cap);
}

#[test]
fn a_host_node_cap_counts_the_parse_before_it_and_stops_the_construction() {
    let base = baseline();
    let cap = base.parse_nodes + base.construction_nodes - 1;
    let (_, host) = host(resource_cap(HtmlResource::ManagedNodes, cap), 12);
    let _parse = parse(&host).unwrap();
    let error = construct(&host).unwrap_err();
    assert_eq!(
        error.cause(),
        HtmlConstructionCause::LimitExceeded {
            resource: HtmlConstructionResource::Nodes,
            limit: usize::try_from(cap).unwrap(),
            attempted: usize::try_from(base.parse_nodes + base.construction_nodes).unwrap(),
        }
    );
}

#[test]
fn a_host_subject_cap_stops_the_capture_with_the_capture_limit() {
    let base = baseline();
    let cap = base.capture_subjects - 1;
    let (_, host) = host(resource_cap(HtmlResource::CapturedSubjects, cap), 12);
    let _parse = parse(&host).unwrap();
    let document = construct(&host).unwrap();
    let Err(CaptureStop::Import(HtmlConstructionCaptureError::Capture(
        HtmlCaptureError::LimitExceeded {
            resource, limit, ..
        },
    ))) = capture(&host, &document)
    else {
        panic!("the capture stops at the host's subject cap");
    };
    assert_eq!(
        resource,
        surgeist_html_semantics::HtmlCaptureResource::Subjects
    );
    assert_eq!(limit, usize::try_from(cap).unwrap());
}

#[test]
fn a_host_mapping_cap_stops_the_import_with_the_import_limit() {
    let base = baseline();
    let cap = base.import_mappings - 1;
    let (_, host) = host(resource_cap(HtmlResource::MappingEntries, cap), 12);
    let _parse = parse(&host).unwrap();
    let document = construct(&host).unwrap();
    let Err(CaptureStop::Import(error)) = capture(&host, &document) else {
        panic!("the import stops at the host's mapping cap");
    };
    assert_eq!(error, HtmlConstructionCaptureError::LimitExceeded);
}

#[test]
fn a_host_step_cap_counts_the_import_before_it_and_stops_the_session() {
    let base = baseline();
    let cap = base.import_work + base.session_work - 1;
    let (_, host) = host(work_cap(HtmlWork::SemanticStep, cap), 12);
    let _parse = parse(&host).unwrap();
    let document = construct(&host).unwrap();
    let captured = capture(&host, &document).unwrap();
    let failure = session(&host, &captured, &document).unwrap_err();
    let HtmlSemanticFailureKind::LimitExceeded {
        resource,
        limit,
        required,
    } = failure.kind()
    else {
        panic!("a limit stop, not {:?}", failure.kind());
    };
    assert_eq!(*resource, HtmlSemanticResource::Work);
    assert_eq!(*limit, usize::try_from(cap).unwrap());
    assert!(*required > *limit);
}

#[test]
fn a_host_step_cap_counts_the_import_and_the_session_before_it_and_stops_the_facts() {
    let base = baseline();
    let cap = base.import_work + base.session_work + base.fact_work - 1;
    let (_, host) = host(work_cap(HtmlWork::SemanticStep, cap), 12);
    let _parse = parse(&host).unwrap();
    let document = construct(&host).unwrap();
    let captured = capture(&host, &document).unwrap();
    let _session = session(&host, &captured, &document).unwrap();
    let error = facts(&host, &captured, &document).unwrap_err();
    let HtmlInertFactError::ValidationWork { limit, required } = error else {
        panic!("a work stop, not {error:?}");
    };
    assert_eq!(limit, usize::try_from(cap).unwrap());
    assert!(required > limit);
}

#[test]
fn a_host_byte_cap_counts_the_session_before_it_and_stops_the_materialization() {
    let base = baseline();
    let cap = base.session_bytes + base.autocomplete_bytes - 1;
    let (_, host) = host(resource_cap(HtmlResource::AllocatedBytes, cap), 12);
    let _parse = parse(&host).unwrap();
    let document = construct(&host).unwrap();
    let captured = capture(&host, &document).unwrap();
    let _session = session(&host, &captured, &document).unwrap();
    let _facts = facts(&host, &captured, &document).unwrap();
    let failure = autocomplete(&host).unwrap_err();
    let HtmlAutocompleteMaterializationError::BytesLimit { limit, required } = failure.kind()
    else {
        panic!("a byte stop, not {:?}", failure.kind());
    };
    assert_eq!(limit, usize::try_from(cap).unwrap());
    assert!(required > limit);
}

// ---------------------------------------------------------------------------------------------
// The scope bound. The stages open 11 scopes below the host's own, and an operation whose bound is
// smaller stops the first stage that has no room, with the one accounting fault in its own stop
// type.

/// The first stage that stops, as its position among parse, construction, capture (its builder
/// and its import), session, facts and autocomplete, with the accounting fault it reports. A stage
/// that stops for another reason, or a bound that fits every stage, reports `None`.
fn first_stop(bound: usize) -> Option<(usize, HtmlAccountingFault)> {
    let (_, host) = host(HtmlLimits::unlimited(), bound);
    let parse = parse(&host);
    if let Err(failure) = &parse {
        return match failure.first_stop() {
            Some(HtmlParseStop::Accounting(fault)) => Some((0, fault)),
            other => panic!("the parse stopped for another reason: {other:?}"),
        };
    }
    let document = match construct(&host) {
        Ok(document) => document,
        Err(error) => return Some((1, error.accounting_fault().expect("a scope fault"))),
    };
    let captured = match capture(&host, &document) {
        Ok(captured) => captured,
        Err(CaptureStop::Builder(fault)) => return Some((2, fault)),
        Err(CaptureStop::Import(error)) => {
            return Some((2, error.accounting_fault().expect("a scope fault")));
        }
    };
    if let Err(failure) = session(&host, &captured, &document) {
        return Some((3, failure.kind().accounting_fault().expect("a scope fault")));
    }
    match facts(&host, &captured, &document) {
        Err(HtmlInertFactError::Accounting(fault)) => return Some((4, fault)),
        Err(other) => panic!("the facts stopped for another reason: {other:?}"),
        Ok(_) => {}
    }
    match autocomplete(&host) {
        Err(failure) => match failure.kind() {
            HtmlAutocompleteMaterializationError::Accounting(fault) => Some((5, fault)),
            other => panic!("the materialization stopped for another reason: {other:?}"),
        },
        Ok(_) => None,
    }
}

#[test]
fn a_scope_bound_below_the_eleven_stage_scopes_stops_the_first_stage_without_room() {
    // The scopes the host holds once each stage has opened its own: the host's scope, then two
    // for the parse, one for the construction, two for the capture (its builder and its import),
    // two for the session, two for the facts and two for the autocomplete.
    let held_after = [3, 4, 6, 8, 10, 12];
    for bound in 1..12 {
        let stage = held_after
            .iter()
            .position(|held| *held > bound)
            .expect("a bound below 12 stops a stage");
        assert_eq!(
            first_stop(bound),
            Some((stage, HtmlAccountingFault::ScopeLimit { limit: bound })),
            "bound {bound}"
        );
    }
    assert_eq!(first_stop(12), None);
}
