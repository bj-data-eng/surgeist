//! Later native edits consume the actual installed document's opaque environment.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
use surgeist_html_semantics::*;
fn parse_limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(10_000)
        .buffered_utf16_units(10_000)
        .token_attributes(1000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(1000)
        .managed_edges(1000)
        .managed_attributes(1000)
        .managed_utf16_units(10_000)
        .source_map_entries(10_000)
        .source_map_payload_units(10_000)
        .diagnostics(1000)
        .diagnostic_payload_units(10_000)
        .context_effects(1000)
        .open_elements(100)
        .tree_depth(1000)
        .managed_capacity_bytes(10_000_000)
        .peak_transient_bytes(10_000_000)
        .atomic_layout_bytes(10_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap()
}
fn base_facts(
    url: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    revision: &HtmlBaseFactsRevision,
) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(url.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        Vec::new(),
        HtmlBaseFactsLimits::new(32, 100_000, 1_000_000),
    )
    .unwrap()
}
#[test]
fn installed_environment_accepts_exact_inputs_and_rejects_same_spelling_foreign_identities() {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(128, 100_000, 256, 256, 256, 16, 1_000_000)
        .unwrap()
        .with_base_processing(128, 1_000_000)
        .unwrap()
        .with_encoding_url_processing(128, 1_000_000)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let origin_doc = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&origin_doc).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 3, 5, 7);
    let url = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/current/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><a></a>"),
            parse_limits(),
        )
        .parse()
        .unwrap(),
    );
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            DomParsedDocumentEnvironment::headless(
                origin,
                url.clone(),
                encoding.clone(),
                "UTF-8".into(),
                DomNativeInputPlatform::headless(),
            ),
            DomParsedHtmlInstallLimits::new(128, 10_000_000, 10_000_000, 10_000_000),
            &cancel,
        )
        .unwrap();
    let document = installed.document();
    let a = installed
        .bindings()
        .iter()
        .map(|binding| binding.node())
        .find(|id| dom.snapshot().node(*id).unwrap().as_hyperlink().is_some())
        .unwrap();
    let initial = dom.snapshot();
    let exact = base_facts(&url, &encoding, &revision);
    let mut batch = DomBatch::new();
    batch.bind_base_facts(initial.document(document).unwrap().stamp(), &exact);
    dom.apply(batch, &cancel).unwrap();
    let parsed = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/current/next".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let facts = HtmlEncodingUrlParseFacts::new(
        url.clone(),
        encoding.clone(),
        revision.clone(),
        vec![HtmlEncodingUrlParseObservation::new(
            "next".into(),
            HtmlEncodingUrlParseOutcome::Parsed(parsed.clone()),
        )],
        HtmlBaseFactsLimits::new(32, 100_000, 1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.bind_encoding_url_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
    batch.set_attribute(a, "href", "next");
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    assert_eq!(
        before.node(a).unwrap().as_hyperlink().unwrap().url(),
        Some(&parsed)
    );
    assert_eq!(initial.node(a).unwrap().as_hyperlink().unwrap().url(), None);
    let same_spelling = HtmlBaseUrlRecord::observed(
        revision.clone(),
        url.serialization().clone(),
        HtmlBaseUrlScheme::Other,
        false,
        url.fragment_delimiter_offset(),
    )
    .unwrap();
    for invalid in [
        base_facts(&same_spelling, &encoding, &revision),
        base_facts(&url, &HtmlBaseEncoding::fresh(), &revision),
    ] {
        let mut batch = DomBatch::new();
        batch.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), &invalid);
        assert_eq!(
            dom.apply(batch, &cancel).unwrap_err(),
            DomError::BaseFacts(DomBaseFactsError::EnvironmentMismatch)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot()
                .node(a)
                .unwrap()
                .as_hyperlink()
                .unwrap()
                .url(),
            Some(&parsed)
        );
    }
}
