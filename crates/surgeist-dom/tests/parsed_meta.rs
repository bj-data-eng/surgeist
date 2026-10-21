//! Real parse/install/consume paths; no caller-authored acknowledgement proof.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_cssom::*;
use surgeist_dom::*;
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
fn envelope() -> DomMetaAtomicLimits {
    DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap()
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
fn exact_original_names_complete_once_with_real_unchanged_producer_and_no_tree_revision() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse(
            "<!doctype html><meta http-equiv=default-style content='B&#x20;C'><meta http-equiv=default-style content=A><meta http-equiv=default-style content='B&#x20;C'>",
        ),
        environment,
    );
    let root = owner(document);
    let caps = [&root];
    let stamp = dom.snapshot().stamp();
    let original_revision = root.snapshot().unwrap().revision().clone();
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    hold.validate_meta(&flag()).unwrap();
    let prepared = hold.prepare(&caps, &flag()).unwrap();
    let rows = prepared.occurrences();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.iter()
            .map(|r| r.content().code_units())
            .collect::<Vec<_>>(),
        vec![&[66, 32, 67][..], &[65][..], &[66, 32, 67][..]]
    );
    assert!(
        rows.iter()
            .all(|r| r.document() == document && r.contribution() == id)
    );
    assert_ne!(rows[0].id(), rows[2].id());
    assert_ne!(rows[0].subject(), rows[2].subject());
    assert!(
        prepared
            .root_snapshot(document.node(), &flag())
            .unwrap()
            .sheets()
            .is_empty()
    );
    let published = prepared.commit(&flag()).unwrap();
    drop(hold);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.pending_parsed_html_association(document).unwrap(), None);
    assert!(dom.pending_parsed_html_associations().next().is_none());
    assert!(matches!(root.snapshot(), Err(DomStyleCssomRootError::Busy)));
    let (completion, dead) = published.into_parts();
    assert_eq!(completion.contribution(), id);
    assert_eq!(completion.stamp(), stamp);
    assert_eq!(completion.roots().len(), 1);
    assert_eq!(completion.roots()[0].root(), document.node());
    assert!(
        matches!(completion.roots()[0].cssom().publication(), CssomPublication::Unchanged { revision } if revision == &original_revision)
    );
    drop(dead);
    assert_eq!(root.snapshot().unwrap().revision(), &original_revision);
    let error = match dom.begin_parsed_html_meta(id, envelope(), &flag()) {
        Err(error) => error,
        Ok(_) => panic!("completed destination cannot replay"),
    };
    assert!(
        matches!(error, DomParsedHtmlMetaAdmissionError::NotPending { contribution } if contribution == id)
    );
}

#[test]
fn whole_influencing_style_refuses_without_an_owner_and_remains_pending() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse(
            "<!doctype html><meta http-equiv=default-style content=B><style title=A>a { color:red }</style>",
        ),
        environment,
    );
    let root = owner(document);
    let stamp = dom.snapshot().stamp();
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    let error = hold.validate_meta(&flag()).unwrap_err();
    assert!(
        matches!(error, DomParsedHtmlMetaAdmissionError::Unsupported { contribution, .. } if contribution == id)
    );
    let error = match hold.prepare(&[], &flag()) {
        Err(error) => error,
        Ok(_) => panic!("whole influencing history must remain pending"),
    };
    assert!(
        matches!(error, DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Unsupported { contribution, .. }) if contribution == id)
    );
    drop(hold);
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
    assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
    assert_eq!(root.snapshot().unwrap().sheets().len(), 0);
}

#[test]
fn removing_first_installation_keeps_the_genuinely_shared_report_and_its_charge_until_last_consumption()
 {
    let (mut dom, environment) = setup();
    let report = parse("<!doctype html><meta http-equiv=default-style content=B>");
    let weak = Arc::downgrade(&report);
    let producer_bytes = usize::try_from(report.retained_construction_bytes()).unwrap();
    let (a, first) = install(&mut dom, report.clone(), environment.clone());
    let (b, second) = install(&mut dom, report.clone(), environment);
    drop(report);
    assert_ne!(first, second);
    for (document, id) in [(a, first), (b, second)] {
        let root = owner(document);
        let caps = [&root];
        let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
        let prepared = hold.prepare(&caps, &flag()).unwrap();
        let (_, dead) = prepared.commit(&flag()).unwrap().into_parts();
        drop(hold);
        drop(dead);
        if id == first {
            assert!(weak.upgrade().is_some());
            assert_eq!(
                dom.pending_parsed_html_association(b).unwrap(),
                Some(second)
            );
            assert!(dom.usage().parsed_association_bytes() >= producer_bytes);
        }
    }
    assert!(weak.upgrade().is_none());
    assert_eq!(dom.usage().parsed_association_bytes(), 0);
}

#[test]
fn normal_drop_retries_original_work_but_forgetting_preparation_keeps_it_sealed() {
    for forget in [false, true] {
        let (mut dom, environment) = setup();
        let (document, id) = install(
            &mut dom,
            parse("<!doctype html><meta http-equiv=default-style content=B>"),
            environment,
        );
        let root = owner(document);
        let caps = [&root];
        let stamp = dom.snapshot().stamp();
        let spent;
        {
            let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
            hold.validate_meta(&flag()).unwrap();
            spent = hold.spent_work();
            let prepared = hold.prepare(&caps, &flag()).unwrap();
            if forget {
                std::mem::forget(prepared);
            } else {
                drop(prepared);
            }
            assert!(
                matches!(hold.step(&flag()), Err(DomError::ParsedAssociationOperationPending { pending }) if pending == id)
            );
        }
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_parsed_html_association(document).unwrap(),
            Some(id)
        );
        if forget {
            assert!(matches!(root.snapshot(), Err(DomStyleCssomRootError::Busy)));
            let error = match dom.begin_parsed_html_meta(id, envelope(), &flag()) {
                Err(error) => error,
                Ok(_) => panic!("forgotten graph cannot be reacquired"),
            };
            assert!(
                matches!(error, DomParsedHtmlMetaAdmissionError::Dom(DomError::ParsedAssociationOperationPending { pending }) if pending == id)
            );
        } else {
            let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
            assert!(hold.spent_work() > spent);
            let prepared = hold.prepare(&caps, &flag()).unwrap();
            let (_, dead) = prepared.commit(&flag()).unwrap().into_parts();
            drop(dead);
        }
    }
}

#[test]
fn original_work_is_finite_persistent_and_only_raiseable_after_refusal() {
    let (mut dom, environment) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        environment,
    );
    let root = owner(document);
    let stamp = dom.snapshot().stamp();
    let short = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 2, 1024).unwrap();
    {
        let mut hold = dom.begin_parsed_html_meta(id, short, &flag()).unwrap();
        assert!(matches!(
            hold.validate_meta(&flag()),
            Err(DomParsedHtmlMetaAdmissionError::Dom(DomError::Meta(
                DomMetaError::WorkLimit { limit: 2 }
            )))
        ));
        assert_eq!(hold.spent_work(), 2);
    }
    let lower = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1, 1024).unwrap();
    let error = match dom.begin_parsed_html_meta(id, lower, &flag()) {
        Err(error) => error,
        Ok(_) => panic!("original work policy cannot decrease"),
    };
    assert!(
        matches!(error, DomParsedHtmlMetaAdmissionError::LimitReduction { contribution } if contribution == id)
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
    let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
    assert_eq!(hold.spent_work(), 3);
    let caps = [&root];
    let prepared = hold.prepare(&caps, &flag()).unwrap();
    let (_, dead) = prepared.commit(&flag()).unwrap().into_parts();
    drop(dead);
}

#[test]
fn cancellation_and_output_refusal_preserve_original_pending_and_owner() {
    for canceled in [true, false] {
        let (mut dom, environment) = setup();
        let (document, id) = install(
            &mut dom,
            parse("<!doctype html><meta http-equiv=default-style content=B>"),
            environment,
        );
        let root = owner(document);
        let caps = [&root];
        let stamp = dom.snapshot().stamp();
        let limits = if canceled {
            envelope()
        } else {
            DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap()
        };
        let mut hold = dom.begin_parsed_html_meta(id, limits, &flag()).unwrap();
        let prepared = hold.prepare(&caps, &flag()).unwrap();
        let error = match prepared.commit(&AtomicBool::new(canceled)) {
            Err(error) => error,
            Ok(_) => panic!("publication admission must refuse"),
        };
        if canceled {
            assert!(matches!(
                error,
                DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Dom(
                    DomError::Cancelled
                ))
            ));
        } else {
            assert!(matches!(
                error,
                DomParsedHtmlMetaError::Admission(DomParsedHtmlMetaAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::MetaAtomicOutputs,
                        limit: 0
                    }
                ))
            ));
        }
        drop(hold);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_parsed_html_association(document).unwrap(),
            Some(id)
        );
        assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
        assert!(root.snapshot().unwrap().sheets().is_empty());
        let mut hold = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
        let prepared = hold.prepare(&caps, &flag()).unwrap();
        let (_, dead) = prepared.commit(&flag()).unwrap().into_parts();
        drop(dead);
    }
}
