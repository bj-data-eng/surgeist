//! Runtime cannot bootstrap empty association state over retained original work.
use super::*;

#[test]
fn parsed_pending_associations_return_original_dom_before_session_owner_adoption() {
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let report = std::sync::Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><meta http-equiv=default-style content=chosen><p>X"),
            HtmlParseLimits::builder()
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
                .unwrap(),
        )
        .parse()
        .unwrap(),
    );
    let cancel = AtomicBool::new(false);
    let mut dom = dispatch_dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let created = dom.apply(batch, &cancel).unwrap();
    let origin = dom
        .snapshot()
        .document(created.document(&document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let environment = surgeist_dom::DomParsedDocumentEnvironment::headless(
        origin,
        HtmlBaseUrlRecord::observed(
            HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
            "https://example.test/parsed/".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap(),
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        surgeist_dom::DomNativeInputPlatform::headless(),
    );
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            surgeist_dom::DomParsedHtmlInstallLimits::new(
                1024,
                10_000_000,
                16_000_000,
                100_000_000,
            ),
            &cancel,
        )
        .unwrap();
    let pending = installed.pending_association().unwrap();
    let document = installed.document();
    drop(installed);
    let before = dom.snapshot().stamp();
    let usage = dom.usage();
    let rejected = DomSession::new(dom, session_limits(limits())).unwrap_err();
    assert!(
        matches!(rejected.error(),DomSessionCreateFailure::Style(DomStyleRuntimeError::PendingParsedAssociations(actual)) if *actual == pending)
    );
    let dom = rejected.into_dom();
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.pending_parsed_html_associations().next().unwrap().id(),
        pending
    );
    assert!(dom.snapshot().document(document).is_ok());
    assert!(!dom.has_element_task_source());
    assert!(!dom.has_source_event_executor());
}

mod consumption;
mod style_decision;
mod style_prefix;
