//! Real parse/install/consume paths; no caller-authored acknowledgement proof.
use super::*;
use crate::*;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_cssom::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn envelope() -> DomParsedHtmlAssociationLimits {
    DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap()
}
fn parse(text: &str) -> Arc<HtmlParseReport> {
    let limits = HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap();
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), limits)
            .parse()
            .unwrap(),
    )
}
fn setup() -> (Dom, DomParsedDocumentEnvironment) {
    let policy = DomLimits::new(
        1024,
        1_000_000,
        1024,
        4096,
        4096,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_meta_processing_work(1_000_000)
    .with_meta_theme_color_scratch_bytes(262144)
    .unwrap();
    let mut dom = Dom::new(policy).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &flag()).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        "https://example.test/parsed/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    (dom, environment)
}
fn install(
    dom: &mut Dom,
    report: Arc<HtmlParseReport>,
    environment: DomParsedDocumentEnvironment,
) -> (DocumentId, DomParsedHtmlAssociationId) {
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &flag(),
        )
        .unwrap();
    (
        installed.document(),
        installed.pending_association().unwrap(),
    )
}
fn owner(document: DocumentId) -> DomStyleCssomRoot {
    DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    )
}

#[test]
fn dropped_preparation_releases_exact_original_ledger_union_after_its_graph() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        environment,
    );
    let root = owner(document);
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let caps = [&root];
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    let holding = ledger.total();
    assert_eq!(
        holding.bytes() - before.bytes(),
        size_of::<DomParsedHtmlMetaRuntimeHold<'_>>()
    );
    let prepared = hold.prepare(&caps, &flag()).unwrap();
    assert!(ledger.total().bytes() > holding.bytes());
    drop(prepared);
    assert_eq!(ledger.total(), holding);
    drop(hold);
    assert_eq!(ledger.total(), before);
    assert!(root.snapshot().unwrap().sheets().is_empty());
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
}

#[test]
fn forgotten_preparation_keeps_original_graph_and_ledger_debit_after_owner_drop() {
    let (mut dom, environment) = setup();
    let report = parse("<!doctype html><meta http-equiv=default-style content=B>");
    let weak = Arc::downgrade(&report);
    let (document, id) = install(&mut dom, report, environment);
    let root = owner(document);
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    let holding = ledger.total();
    let caps = [&root];
    let prepared = hold.prepare(&caps, &flag()).unwrap();
    let prepared_total = ledger.total();
    assert!(prepared_total.bytes() > holding.bytes());
    std::mem::forget(prepared);
    drop(hold);
    assert_eq!(
        ledger.total().bytes(),
        prepared_total.bytes() - (holding.bytes() - before.bytes())
    );
    assert!(matches!(root.snapshot(), Err(DomStyleCssomRootError::Busy)));
    drop(dom);
    drop(root);
    assert!(weak.upgrade().is_some());
    // The forgotten graph owns both authentic report custody and its debit;
    // dropping the canonical owner cannot issue a refund for that live graph.
    assert!(ledger.total().bytes() > 0);
}

#[test]
fn runtime_whole_reports_charge_only_positive_storage_and_output_deltas() {
    let (mut dom, environment) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        environment,
    );
    let ledger = dom.custody_registry.ledger().clone();
    let baseline = ledger.total();
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 5).unwrap();
    let mut hold = dom.begin_parsed_html_meta(id, limits, &flag()).unwrap();
    let holding = ledger.total();
    let start_work = hold.spent_work();
    hold.admit_runtime(
        DomParsedHtmlAssociationRuntimeUsage::new(60, 3, 2).unwrap(),
        &flag(),
    )
    .unwrap();
    assert_eq!(
        ledger.total(),
        holding.checked_add(custody::Usage::new(60, 3)).unwrap()
    );
    hold.admit_runtime(
        DomParsedHtmlAssociationRuntimeUsage::new(60, 3, 2).unwrap(),
        &flag(),
    )
    .unwrap();
    hold.admit_runtime(
        DomParsedHtmlAssociationRuntimeUsage::new(20, 1, 1).unwrap(),
        &flag(),
    )
    .unwrap();
    assert_eq!(
        ledger.total(),
        holding.checked_add(custody::Usage::new(60, 3)).unwrap()
    );
    hold.admit_runtime(
        DomParsedHtmlAssociationRuntimeUsage::new(90, 7, 5).unwrap(),
        &flag(),
    )
    .unwrap();
    assert_eq!(
        ledger.total(),
        holding.checked_add(custody::Usage::new(90, 7)).unwrap()
    );
    assert_eq!(hold.spent_work(), start_work + 4);
    drop(hold);
    assert_eq!(ledger.total(), baseline);
    let mut retry = dom.begin_parsed_html_meta(id, limits, &flag()).unwrap();
    let before_refusal = ledger.total();
    assert!(matches!(
        retry.admit_runtime(
            DomParsedHtmlAssociationRuntimeUsage::new(0, 0, 1).unwrap(),
            &flag()
        ),
        Err(DomError::ResourceLimit {
            resource: DomResource::ParsedAssociationProcessingOutputs,
            limit: 5
        })
    ));
    assert_eq!(ledger.total(), before_refusal);
    drop(retry);
    assert_eq!(ledger.total(), baseline);
}

#[test]
fn conflicting_busy_and_poisoned_actual_capabilities_cannot_acknowledge_the_contribution() {
    for failure in 0..3 {
        let (mut dom, environment) = setup();
        let (document, id) = install(
            &mut dom,
            parse("<!doctype html><meta http-equiv=default-style content=B>"),
            environment,
        );
        let root = owner(document);
        let other = owner(document);
        let stamp = dom.snapshot().stamp();
        let held = if failure == 1 {
            Some(root.acquire().unwrap())
        } else {
            None
        };
        if failure == 2 {
            let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _guard = root.acquire().unwrap();
                panic!("actual external owner failure while locked");
            }));
            assert!(caught.is_err());
        }
        let caps = if failure == 0 {
            vec![&root, &other]
        } else {
            vec![&root]
        };
        let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
        let error = match hold.prepare(&caps, &flag()) {
            Err(error) => error,
            Ok(_) => panic!("qualified owner refusal required"),
        };
        assert!(match error {
            DomParsedHtmlMetaError::MismatchedOwner { root } if failure == 0 =>
                root == document.node(),
            DomParsedHtmlMetaError::OwnerBusy { root } if failure == 1 => root == document.node(),
            DomParsedHtmlMetaError::OwnerPoisoned { root } if failure == 2 =>
                root == document.node(),
            _ => false,
        });
        drop(hold);
        drop(held);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_parsed_html_association(document).unwrap(),
            Some(id)
        );
        assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
        assert!(other.snapshot().unwrap().sheets().is_empty());
        if failure != 2 {
            assert!(root.snapshot().unwrap().sheets().is_empty());
        }
    }
}

#[test]
fn a_preferred_name_change_without_its_original_name_is_a_typed_report_defect() {
    use surgeist_html_parser::{
        HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
    };
    let (mut dom, environment) = setup();
    let report = parse("<!doctype html><head><meta http-equiv=default-style content=B></head>");
    let (_, id) = install(&mut dom, report.clone(), environment);
    let mut cursor = report.pending_associations();
    let occurrence = loop {
        match cursor.advance() {
            Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => {
                // The sealed row carries the name the decoder needs.
                assert!(row.preferred_name().is_some());
                break row.id();
            }
            Step::Complete | Step::Fault(_) => panic!("no preferred-name change was sealed"),
            _ => {}
        }
    };
    assert!(matches!(
        require_preferred_name::<()>(None, id, occurrence.clone()),
        Err(DomParsedHtmlAssociationAdmissionError::MissingPreferredName {
            contribution,
            occurrence: found,
        }) if contribution == id && found == occurrence
    ));
    assert_eq!(require_preferred_name(Some(7), id, occurrence.clone()).unwrap(), 7);
}
