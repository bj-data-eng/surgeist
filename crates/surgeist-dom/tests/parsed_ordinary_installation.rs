//! Literal completed-report transfer through the actual public owners.
#[path = "parsed_ordinary_installation/canonical_binding_changes.rs"]
mod canonical_binding_changes;
#[path = "parsed_ordinary_installation/contextual_controls.rs"]
mod contextual_controls;
#[path = "parsed_ordinary_installation/noncontrol_state.rs"]
mod noncontrol_state;

use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;
use surgeist_html::{HtmlCustomElementState, HtmlDocumentMode};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn parse_limits() -> HtmlParseLimits {
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
        .unwrap()
}
fn parse(source: HtmlSource) -> Arc<HtmlParseReport> {
    Arc::new(
        HtmlDocumentParseRequest::offline(source, parse_limits())
            .parse()
            .unwrap(),
    )
}
fn setup() -> (Dom, DomParsedDocumentEnvironment) {
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut b = DomBatch::new();
    let origin_document = b.create_document();
    let commit = dom.apply(b, &AtomicBool::new(false)).unwrap();
    let snapshot = dom.snapshot();
    let origin = snapshot
        .document(commit.document(&origin_document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let url = HtmlBaseUrlRecord::observed(
        revision,
        "https://example.test/ordinary/".into(),
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
fn limits() -> DomParsedHtmlInstallLimits {
    DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000)
}

fn selected_encoding_context(
    capture: &DomHtmlCapture,
    document: DocumentId,
    facts: surgeist_html_semantics::HtmlParseBaseFacts,
    frozen: Vec<surgeist_html_semantics::HtmlCapturedFrozenBase>,
) -> surgeist_html_semantics::HtmlDocumentEncodingUrlContext {
    use surgeist_html_semantics::*;
    let doc = capture
        .capture()
        .document_id(&capture.subject(document.node()).unwrap())
        .unwrap();
    let revision = facts.document().revision().clone();
    let bank = HtmlCapturedBaseFacts::new(
        capture.capture(),
        capture.capture().host_facts_revision().clone(),
        revision.clone(),
        vec![HtmlCapturedBaseDocument::new(doc.clone(), facts)],
        frozen,
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let semantic = HtmlSemanticContext::new(
        capture.capture(),
        capture.capture().host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_base_facts(revision, Some(bank)),
        0,
    )
    .unwrap();
    let selected = HtmlSemanticSession::new(
        capture.capture(),
        &semantic,
        surgeist_html::HtmlValueInterpreter::new(65536),
        HtmlSemanticLimits::new(10_000_000, 16, 4096, 0, 10_000_000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        doc.subject().clone(),
        HtmlSemanticQuery::DocumentEncodingUrlContext,
    )])
    .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(selected)) =
        selected.results()[0].fact()
    else {
        panic!("actual semantic selection")
    };
    selected.clone()
}

#[test]
fn contextual_href_observation_requires_the_actual_canonical_bound_inputs() {
    use surgeist_html_parser::{HtmlFragmentParseRequest, HtmlParseHyperlinkState};
    use surgeist_html_semantics::*;
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(256, 10_000_000)
        .unwrap()
        .with_encoding_url_processing(256, 10_000_000)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let make_facts = || {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let url = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap();
        let encoding = HtmlBaseEncoding::fresh();
        let absolute = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "https://example.test/exact".into(),
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )
        .unwrap();
        let base = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(url.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            vec![],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        // Relative x against about:blank is an actual observed failure, never
        // an invented successful relative URL or unavailable observation.
        let parsing = HtmlEncodingUrlParseFacts::new(
            url,
            encoding,
            revision,
            vec![
                HtmlEncodingUrlParseObservation::new(
                    "x".into(),
                    HtmlEncodingUrlParseOutcome::Failure,
                ),
                HtmlEncodingUrlParseObservation::new(
                    "https://example.test/exact".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(absolute.clone()),
                ),
                HtmlEncodingUrlParseObservation::new(
                    DocumentString::from_utf16(vec![0xd800, 120, 0xdc00]),
                    HtmlEncodingUrlParseOutcome::Failure,
                ),
            ],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        (base, parsing, absolute)
    };
    let (bound_base, bound_parsing, absolute) = make_facts();
    let mut batch = DomBatch::new();
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    batch.bind_base_facts(stamp, &bound_base);
    batch.bind_encoding_url_facts(stamp, &bound_parsing);
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(4096, 8192, 4096, 1_000_000, 4096, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(4096, 10_000_000).with_base_facts(4096, 65536, 10_000_000),
            &cancel,
        )
        .unwrap();
    let doc = capture
        .capture()
        .document_id(&capture.subject(document.node()).unwrap())
        .unwrap();
    let (unbound_base, unbound_parsing, _) = make_facts();
    assert_ne!(
        unbound_base.document().revision(),
        bound_base.document().revision()
    );
    assert_ne!(
        unbound_base.document().encoding(),
        bound_base.document().encoding()
    );
    // The same real DomHtmlCapture can be interpreted with independently supplied
    // external facts. Its identity does not promote those facts to canonical.
    let selected = selected_encoding_context(&capture, document, unbound_base.clone(), vec![]);
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><a href=x></a></div>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(vec![selected.clone()])
        .with_encoding_url_facts(vec![unbound_parsing])
        .parse()
        .unwrap(),
    );
    assert!(report.captured_base_facts().is_none());
    let mut cursor = report.metadata();
    let mut observed = false;
    loop {
        match cursor.advance() {
            surgeist_html_parser::HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    let surgeist_html_parser::HtmlParseMetadataDocument::Captured(actual) =
                        link.document()
                    else {
                        panic!("nested href uses captured Document")
                    };
                    assert_eq!(actual.id(), *doc.subject());
                    let HtmlParseHyperlinkState::Observed(snapshot) = link.state() else {
                        panic!("real observed failure")
                    };
                    assert_eq!(snapshot.outcome(), &HtmlEncodingUrlParseOutcome::Failure);
                    assert_eq!(snapshot.revision(), unbound_base.document().revision());
                    observed = true;
                }
            }
            surgeist_html_parser::HtmlParseMetadataStep::Skipped => {}
            surgeist_html_parser::HtmlParseMetadataStep::Complete => break,
        }
    }
    assert!(observed);
    let error = dom
        .install_parsed_html_fragment(report, &capture, limits(), &cancel)
        .err();
    assert!(
        matches!(
            &error,
            Some(DomParsedHtmlInstallError::CurrentPolicyUnavailable)
        ),
        "expected current-authority refusal, actual error: {error:?}"
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    for which in 0..4 {
        let revision = bound_base.document().revision().clone();
        let base = if which == 1 {
            HtmlBaseUrlRecord::observed(
                revision.clone(),
                "about:blank".into(),
                HtmlBaseUrlScheme::Other,
                true,
                None,
            )
            .unwrap()
        } else {
            bound_base.document().document_url().unwrap().clone()
        };
        let encoding = if which == 0 {
            HtmlBaseEncoding::fresh()
        } else {
            bound_base.document().encoding().unwrap().clone()
        };
        let raw = match which {
            2 => "X",
            3 => "https://example.test/exact",
            _ => "x",
        };
        let outcome = if which == 3 {
            // Same correct URL spelling, but a different opaque observation.
            let other = HtmlBaseUrlRecord::observed(
                revision.clone(),
                "https://example.test/exact".into(),
                HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap();
            assert_ne!(other, absolute);
            HtmlEncodingUrlParseOutcome::Parsed(other)
        } else {
            HtmlEncodingUrlParseOutcome::Failure
        };
        let facts = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(base.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            vec![],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        let parsing = HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            vec![HtmlEncodingUrlParseObservation::new(raw.into(), outcome)],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        let selected = selected_encoding_context(&capture, document, facts, vec![]);
        let context =
            HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap())
                .unwrap();
        let report = Arc::new(
            HtmlFragmentParseRequest::new(
                HtmlSource::utf8(format!("<div><a href='{raw}'></a></div>")),
                context,
                parse_limits(),
            )
            .unwrap()
            .with_encoding_url_contexts(vec![selected])
            .with_encoding_url_facts(vec![parsing])
            .parse()
            .unwrap(),
        );
        let error = dom
            .install_parsed_html_fragment(report, &capture, limits(), &cancel)
            .err();
        assert!(
            matches!(
                &error,
                Some(DomParsedHtmlInstallError::CurrentPolicyUnavailable)
            ),
            "variant {which}: {error:?}"
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
    // Positive source selection comes from the actual canonical capture's own
    // semantic context, independently of the foreign candidate above.
    let selected = capture
        .query(
            &[HtmlSemanticRequest::new(
                doc.subject().clone(),
                HtmlSemanticQuery::DocumentEncodingUrlContext,
            )],
            surgeist_html::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 0, 10_000_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(selected)) =
        selected.results()[0].fact()
    else {
        panic!("canonical context")
    };
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let unused = HtmlCapturedBaseFacts::new(
        capture.capture(),
        capture.capture().host_facts_revision().clone(),
        unbound_base.document().revision().clone(),
        vec![HtmlCapturedBaseDocument::new(
            doc.clone(),
            unbound_base.clone(),
        )],
        vec![],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let mut source_units: Vec<u16> = "<div><a href=x></a><a href='".encode_utf16().collect();
    source_units.extend([0xd800, 120, 0xdc00]);
    source_units.extend("'></a><area href='https://example.test/exact'></div>".encode_utf16());
    let report = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf16(DocumentString::from_utf16(source_units)),
            context,
            parse_limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(vec![selected.clone()])
        .with_encoding_url_facts(vec![bound_parsing])
        .with_base_facts(unused)
        .unwrap()
        .parse()
        .unwrap(),
    );
    let mut cursor = report.metadata();
    let source = loop {
        match cursor.advance() {
            surgeist_html_parser::HtmlParseMetadataStep::Record(row)
                if row.hyperlink().is_some() =>
            {
                break row.node().id();
            }
            surgeist_html_parser::HtmlParseMetadataStep::Complete => panic!("actual source A"),
            _ => {}
        }
    };
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &cancel)
        .unwrap();
    let link = installed.node_for(&source).unwrap();
    let view = installed
        .snapshot()
        .node(link)
        .unwrap()
        .as_hyperlink()
        .unwrap();
    assert_eq!(view.parse_document(), Some(document));
    assert_eq!(view.url(), None);
    let original = view.parse_observation().unwrap();
    assert_eq!(
        original.base(),
        bound_base.document().document_url().unwrap()
    );
    assert_eq!(
        original.encoding(),
        bound_base.document().encoding().unwrap()
    );
    assert_eq!(original.revision(), bound_base.document().revision());
    assert_eq!(original.outcome(), &HtmlEncodingUrlParseOutcome::Failure);
    assert_eq!(
        installed.captured_node_for(doc.subject()),
        Some(document.node())
    );
    assert!(
        installed
            .snapshot()
            .node(installed.root())
            .unwrap()
            .parent()
            .is_none()
    );
    assert!(installed.bindings().iter().any(|binding| {
        installed
            .snapshot()
            .node(binding.node())
            .unwrap()
            .as_hyperlink()
            .is_some_and(|link| link.url() == Some(&absolute))
    }));
    assert!(installed.bindings().iter().any(|binding| {
        installed
            .snapshot()
            .node(binding.node())
            .unwrap()
            .as_hyperlink()
            .and_then(|link| link.parse_observation())
            .is_some_and(|observation| {
                observation.raw().code_units() == [0xd800, 120, 0xdc00]
                    && observation.outcome() == &HtmlEncodingUrlParseOutcome::Failure
            })
    }));
    let current = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(link, "href", "x");
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(dom.snapshot().stamp(), current.stamp());
    assert_eq!(installed.node_for(&source), Some(link));
    assert_eq!(view.parse_observation().unwrap(), original);
}
#[test]
fn contextual_current_revision_is_independent_of_the_older_frozen_base_record() {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::*;
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            1024,
            1_000_000,
            1024,
            4096,
            4096,
            32,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(256, 10_000_000)
        .unwrap()
        .with_encoding_url_processing(256, 10_000_000)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element_ns(&doc, Namespace::html(), "div");
    let base = batch.create_element_ns(&doc, Namespace::html(), "base");
    let host = batch.create_element_ns(&doc, Namespace::html(), "div");
    batch.insert_before(&root, &doc, None);
    batch.insert_before(&base, &root, None);
    batch.insert_before(&host, &root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&doc).unwrap();
    let base = commit.created(&base).unwrap().unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let first = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 5, 7, 11);
    let encoding = HtmlBaseEncoding::fresh();
    let record = |revision: &HtmlBaseFactsRevision,
                  text: &str,
                  blank,
                  fragment_delimiter_offset: Option<usize>| {
        HtmlBaseUrlRecord::observed(
            revision.clone(),
            text.into(),
            HtmlBaseUrlScheme::Other,
            blank,
            fragment_delimiter_offset,
        )
        .unwrap()
    };
    let old_blank = record(&first, "about:blank", true, None);
    let old_base = record(&first, "https://old.test/base/", false, None);
    let original = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            first.clone(),
            Some(old_blank.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "https://old.test/base/".into(),
                old_blank,
                encoding.clone(),
                first.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: old_base.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.bind_base_facts(
        dom.snapshot().document(document).unwrap().stamp(),
        &original,
    );
    batch.set_attribute(base, "href", "https://old.test/base/");
    dom.apply(batch, &cancel).unwrap();
    let current = HtmlBaseFactsRevision::new(first.producer().clone(), 5, 8, 11);
    let current_facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            current.clone(),
            Some(record(&current, "about:blank", true, None)),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let parsing = HtmlEncodingUrlParseFacts::new(
        old_base.clone(),
        encoding.clone(),
        current.clone(),
        vec![HtmlEncodingUrlParseObservation::new(
            "http://[".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        )],
        HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let stamp = dom.snapshot().document(document).unwrap().stamp();
    batch.bind_base_facts(stamp, &current_facts);
    batch.bind_encoding_url_facts(stamp, &parsing);
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        before
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &old_base
    );
    let capture = before
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(4096, 8192, 4096, 1_000_000, 4096, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(4096, 10_000_000).with_base_facts(4096, 65536, 10_000_000),
            &cancel,
        )
        .unwrap();
    let captured_doc = capture
        .capture()
        .document_id(&capture.subject(document.node()).unwrap())
        .unwrap();
    for (configuration, urls, csp) in [(6, 8, 11), (5, 9, 11), (5, 8, 12)] {
        let revision =
            HtmlBaseFactsRevision::new(first.producer().clone(), configuration, urls, csp);
        let facts = HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(record(&revision, "about:blank", true, None)),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding.clone()),
            )
            .unwrap(),
            vec![],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        let selected = selected_encoding_context(
            &capture,
            document,
            facts,
            vec![HtmlCapturedFrozenBase::new(
                capture.subject(base).unwrap(),
                captured_doc.clone(),
                "https://old.test/base/".into(),
                Some(old_base.clone()),
                first.clone(),
            )],
        );
        assert_eq!(selected.base_url().value(), &old_base);
        assert_eq!(selected.encoding(), &encoding);
        assert_eq!(selected.revision(), &revision);
        let bank = HtmlEncodingUrlParseFacts::new(
            old_base.clone(),
            encoding.clone(),
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "http://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlBaseFactsLimits::new(256, 65536, 1_000_000),
        )
        .unwrap();
        let context =
            HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap())
                .unwrap();
        let report = Arc::new(
            HtmlFragmentParseRequest::new(
                HtmlSource::utf8("<div><a href='http://['></a></div>"),
                context,
                parse_limits(),
            )
            .unwrap()
            .with_encoding_url_contexts(vec![selected])
            .with_encoding_url_facts(vec![bank])
            .parse()
            .unwrap(),
        );
        assert!(matches!(
            dom.install_parsed_html_fragment(report, &capture, limits(), &cancel),
            Err(DomParsedHtmlInstallError::CurrentPolicyUnavailable)
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    let selected = capture
        .query(
            &[HtmlSemanticRequest::new(
                captured_doc.subject().clone(),
                HtmlSemanticQuery::DocumentEncodingUrlContext,
            )],
            surgeist_html::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(10_000_000, 16, 4096, 0, 10_000_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(selected)) =
        selected.results()[0].fact()
    else {
        panic!("current canonical selection")
    };
    assert_eq!(selected.base_url().value(), &old_base);
    assert_eq!(selected.base_url().frozen_revision(), Some(&first));
    assert_eq!(selected.revision(), &current);
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><area href='http://['></div>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(vec![selected.clone()])
        .with_encoding_url_facts(vec![parsing])
        .parse()
        .unwrap(),
    );
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &cancel)
        .unwrap();
    let link = installed
        .bindings()
        .iter()
        .find_map(|binding| {
            installed
                .snapshot()
                .node(binding.node())
                .unwrap()
                .as_hyperlink()
        })
        .unwrap();
    let observed = link.parse_observation().unwrap();
    assert_eq!(observed.base(), &old_base);
    assert_eq!(observed.base().revision(), &first);
    assert_eq!(observed.revision(), &current);
    assert_eq!(observed.encoding(), &encoding);
    assert_eq!(observed.raw(), &DocumentString::from("http://["));
    assert_eq!(observed.outcome(), &HtmlEncodingUrlParseOutcome::Failure);
    assert_eq!(link.parse_document(), Some(document));
}

#[test]
fn unused_captured_base_graph_is_retained_and_charged_once_with_actual_spare_backing() {
    use surgeist_html_parser::{HtmlFragmentParseRequest, HtmlParseMetadataStep};
    use surgeist_html_semantics::*;
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let cancel = AtomicBool::new(false);
    let before = dom.snapshot();
    let usage = dom.usage();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 13, 17, 19);
    let encoding = HtmlBaseEncoding::fresh();
    let mut blank = Vec::with_capacity(65536);
    blank.extend("about:blank".encode_utf16());
    let blank = HtmlBaseUrlRecord::observed(
        revision.clone(),
        DocumentString::from_utf16(blank),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let mut raw = Vec::with_capacity(32768);
    raw.extend("never-used".encode_utf16());
    let mut observations = Vec::with_capacity(64);
    observations.push(
        HtmlBaseParseObservation::new(
            DocumentString::from_utf16(raw),
            blank.clone(),
            encoding.clone(),
            revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap(),
    );
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(blank),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        observations,
        HtmlBaseFactsLimits::new(512, 65536, 1_000_000),
    )
    .unwrap();
    let mut documents = Vec::with_capacity(128);
    documents.push(HtmlCapturedBaseDocument::new(
        capture
            .capture()
            .document_id(&capture.subject(document.node()).unwrap())
            .unwrap(),
        facts,
    ));
    let bank = HtmlCapturedBaseFacts::new(
        capture.capture(),
        capture.capture().host_facts_revision().clone(),
        revision,
        documents,
        vec![],
        HtmlBaseFactsLimits::new(512, 65536, 1_000_000),
    )
    .unwrap();
    let bank_bytes = bank.retained_storage_usage().retained_bytes();
    assert!(bank_bytes > 100_000);
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(HtmlSource::utf8("<p>x</p>"), context, parse_limits())
            .unwrap()
            .with_base_facts(bank)
            .unwrap()
            .parse()
            .unwrap(),
    );
    assert!(report.document_base_facts().is_none());
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => assert!(row.document_base_facts().is_none()),
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let captured_bytes = capture
        .capture()
        .retained_storage_usage_with(|| Ok::<_, ()>(()))
        .unwrap()
        .retained_bytes();
    // Each owner supplies its complete payload receipt; the public report header
    // embeds only the Arc handle, not the out-of-line optional bank payload.
    let exact = report.retained_construction_bytes()
        + u64::try_from(
            std::mem::size_of::<HtmlParseReport>()
                + report.source().len()
                + captured_bytes
                + bank_bytes,
        )
        .unwrap();
    let original_report = Arc::as_ptr(&report);
    let original_bank = report.captured_base_facts().unwrap() as *const HtmlCapturedBaseFacts;
    assert!(matches!(
        dom.install_parsed_html_fragment(
            report.clone(),
            &capture,
            limits(),
            &AtomicBool::new(true)
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    let result = dom.install_parsed_html_fragment(
        report.clone(),
        &capture,
        DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, exact - 1),
        &cancel,
    );
    assert!(
        matches!(result,Err(DomParsedHtmlInstallError::ResourceLimit{resource:DomParsedHtmlInstallResource::HistoricalBytes,limit}) if limit==exact-1)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let installed = dom
        .install_parsed_html_fragment(
            report.clone(),
            &capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, exact),
            &cancel,
        )
        .unwrap();
    drop(report);
    assert!(std::ptr::eq(original_report, installed.report()));
    assert!(std::ptr::eq(
        original_bank,
        installed.report().captured_base_facts().unwrap()
    ));
    assert_eq!(
        installed
            .report()
            .captured_base_facts()
            .unwrap()
            .retained_storage_usage()
            .retained_bytes(),
        bank_bytes
    );
    assert_eq!(installed.document(), document);
    assert!(
        installed
            .snapshot()
            .node(installed.root())
            .unwrap()
            .parent()
            .is_none()
    );
    assert_eq!(
        installed.snapshot().node(host).unwrap().children(),
        before.node(host).unwrap().children()
    );
    assert!(
        installed
            .snapshot()
            .document(document)
            .unwrap()
            .base_facts()
            .is_none(),
        "unused graph is history, not a new canonical binding"
    );
}

fn install(
    dom: &mut Dom,
    report: Arc<HtmlParseReport>,
    environment: DomParsedDocumentEnvironment,
) -> DomParsedHtmlInstallation {
    dom.install_parsed_html_document(
        dom.snapshot().stamp(),
        report,
        environment,
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn fragment_fixture(local: &str) -> (Dom, DomHtmlCapture, DocumentId, NodeId) {
    use surgeist_html_semantics::HtmlCaptureLimits;
    let (mut dom, _) = setup();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, Namespace::html(), local);
    batch.insert_before(&host, &doc, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let capture = dom
        .snapshot()
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    (dom, capture, document, host)
}
fn fragment_report(
    capture: &DomHtmlCapture,
    target: NodeId,
    source: HtmlSource,
    dsd: bool,
) -> Arc<HtmlParseReport> {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    let context = surgeist_html_semantics::HtmlFragmentContext::element(
        capture.capture(),
        &capture.subject(target).unwrap(),
    )
    .unwrap();
    Arc::new(
        HtmlFragmentParseRequest::with_options(
            source,
            context,
            parse_limits(),
            surgeist_html::HtmlParserScriptingMode::Inert,
            dsd,
        )
        .unwrap()
        .parse()
        .unwrap(),
    )
}

#[test]
fn contextual_utf16_fragment_is_detached_and_later_explicit_insertion_keeps_historical_joins() {
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let raw = vec![
        60, 112, 32, 100, 97, 116, 97, 45, 120, 61, 39, 0xd800, 39, 62, 65, 13, 10, 66, 60, 47,
        112, 62, 60, 33, 45, 45, 67, 45, 45, 62,
    ];
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf16(DocumentString::from_utf16(raw)),
        false,
    );
    let source_root = report.fragment().unwrap().root().id();
    let basis = capture.basis();
    let before = dom.snapshot();
    let installed = dom
        .install_parsed_html_fragment(report.clone(), &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(installed.document(), document);
    assert_eq!(installed.context_basis(), Some(basis));
    assert_eq!(installed.node_for(&source_root), Some(installed.root()));
    assert_eq!(
        installed.captured_node_for(&capture.subject(host).unwrap()),
        Some(host)
    );
    let root = installed.snapshot().node(installed.root()).unwrap();
    assert!(matches!(root.kind(), DomNodeKind::DocumentFragment));
    assert_eq!(root.owner_document(), Some(document));
    assert_eq!(root.parent(), None);
    assert!(
        installed
            .snapshot()
            .node(host)
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(
        installed.snapshot().revision().get(),
        before.revision().get() + 1
    );
    // Unreferenced parser bootstrap Document/html are historical only.
    assert!(!installed.bindings().iter().any(|binding| matches!(
        installed.snapshot().node(binding.node()).unwrap().kind(),
        DomNodeKind::Document(_)
    )));
    let p = element(&installed, "p");
    assert_eq!(
        installed
            .snapshot()
            .node(p)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("data-x")),
        Some(&DocumentString::from_utf16(vec![0xd800]))
    );
    let units: Vec<_> = installed
        .snapshot()
        .node(p)
        .unwrap()
        .children()
        .iter()
        .flat_map(|id| {
            installed
                .snapshot()
                .node(*id)
                .unwrap()
                .data()
                .unwrap()
                .code_units()
                .iter()
                .copied()
        })
        .collect();
    assert_eq!(units, vec![65, 10, 66]);
    let mut batch = DomBatch::new();
    batch.insert_before(installed.root(), host, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.snapshot().node(p).unwrap().parent(), Some(host));
    assert_eq!(
        installed.snapshot().node(p).unwrap().parent(),
        Some(installed.root())
    );
    assert_eq!(installed.node_for(&source_root), Some(installed.root()));
}

#[test]
fn contextual_disabled_script_prefix_and_style_keep_distinct_actual_parser_history() {
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf8("<script>inert</script><style>a{}</style>"),
        true,
    );
    let parser_source = report.fragment_parser_document().unwrap().document().id();
    let original = dom
        .snapshot()
        .document(document)
        .unwrap()
        .metadata()
        .clone();
    let install = |dom: &mut Dom, capture: &DomHtmlCapture| {
        dom.install_parsed_html_fragment(report.clone(), capture, limits(), &AtomicBool::new(false))
            .unwrap()
    };
    let installed = install(&mut dom, &capture);
    let auxiliary = installed
        .snapshot()
        .node(installed.node_for(&parser_source).unwrap())
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    assert_ne!(auxiliary, document);
    let metadata = installed.snapshot().document(auxiliary).unwrap().metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Html);
    assert_eq!(
        metadata.content_type(),
        &DocumentString::from("application/xml")
    );
    assert_eq!(metadata.url(), &DocumentString::from("about:blank"));
    assert_eq!(metadata.encoding(), &DocumentString::from("UTF-8"));
    assert!(!metadata.allow_declarative_shadow_roots());
    assert_eq!(
        metadata.creation_cause(),
        &DomDocumentCreationCause::ParsedFragmentParser
    );
    assert_eq!(
        installed
            .snapshot()
            .node(auxiliary.node())
            .unwrap()
            .children()
            .len(),
        1
    );
    let script = element(&installed, "script");
    let style = element(&installed, "style");
    // HTML877d47590–47604 clears this slot and sets force-async before
    // returning for the detached fragment, before the later restore step.
    let script = installed
        .snapshot()
        .node(script)
        .unwrap()
        .as_html_script()
        .unwrap();
    assert_eq!(script.parser_document(), None);
    assert_eq!(script.preparation_document(), None);
    assert!(script.flags().force_async());
    assert!(!script.flags().already_started());
    assert_eq!(script.script_type(), None);
    assert_eq!(
        script.result(),
        surgeist_html::HtmlScriptResult::Uninitialized
    );
    assert_eq!(
        installed
            .snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .parser_document(),
        Some(auxiliary)
    );
    assert_eq!(
        installed.snapshot().document(document).unwrap().metadata(),
        &original
    );
    assert_ne!(metadata.origin(), original.origin());
}

#[test]
fn contextual_dsd_applies_captured_and_owned_effects_and_preserves_fallback_template() {
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf8(
            "<template shadowrootmode=open shadowrootdelegatesfocus shadowrootclonable shadowrootserializable><p>S</p></template><template shadowrootmode=closed><b>F</b></template><section><template shadowrootmode=closed><i>I</i></template></section>tail",
        ),
        true,
    );
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let shadow = installed
        .snapshot()
        .node(host)
        .unwrap()
        .hosted_shadow_root()
        .unwrap();
    let root = installed.snapshot().shadow_root(shadow).unwrap();
    assert_eq!(root.host(), host);
    assert_eq!(root.node().owner_document(), Some(document));
    assert_eq!(root.state().mode(), HtmlShadowRootMode::Open);
    assert!(root.state().declarative());
    assert!(root.state().available_to_element_internals());
    assert!(root.state().delegates_focus());
    assert!(root.state().clonable());
    assert!(root.state().serializable());
    assert!(
        installed
            .snapshot()
            .node(host)
            .unwrap()
            .children()
            .is_empty()
    );
    let section = element(&installed, "section");
    let owned_root = installed
        .snapshot()
        .node(section)
        .unwrap()
        .hosted_shadow_root()
        .unwrap();
    assert_eq!(
        installed
            .snapshot()
            .shadow_root(owned_root)
            .unwrap()
            .state()
            .mode(),
        HtmlShadowRootMode::Closed
    );
    assert_eq!(
        installed
            .snapshot()
            .node(element(&installed, "i"))
            .unwrap()
            .parent(),
        Some(owned_root)
    );
    let b = element(&installed, "b");
    assert!(matches!(
        installed
            .snapshot()
            .node(installed.snapshot().node(b).unwrap().parent().unwrap())
            .unwrap()
            .kind(),
        DomNodeKind::DocumentFragment
    ));
    assert!(installed.bindings().iter().any(|binding| {
        installed
            .snapshot()
            .node(binding.node())
            .unwrap()
            .as_template()
            .is_some_and(|template| template.contents().unwrap() == shadow)
    }));
}

#[test]
fn contextual_foreign_stale_cancelled_and_missing_intrinsic_inputs_are_atomic() {
    for which in 0..4 {
        let (mut dom, capture, _, host) = fragment_fixture("div");
        let report = fragment_report(
            &capture,
            host,
            HtmlSource::utf8(match which {
                3 => "<a href=x>x</a>",
                _ => "<p>x</p>",
            }),
            false,
        );
        let other_capture = if which == 0 {
            Some(
                dom.snapshot()
                    .html_capture(
                        capture.basis().document(),
                        None,
                        surgeist_html_semantics::HtmlCaptureLimits::new(
                            1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216,
                        ),
                        DomHtmlCaptureLimits::new(1024, 1_000_000),
                        &AtomicBool::new(false),
                    )
                    .unwrap(),
            )
        } else {
            None
        };
        if which == 1 {
            let mut b = DomBatch::new();
            b.set_attribute(host, "data-changed", "yes");
            dom.apply(b, &AtomicBool::new(false)).unwrap();
        }
        let before = dom.snapshot();
        let usage = dom.usage();
        let changes = dom
            .changes_since(before.stamp())
            .unwrap()
            .changes()
            .to_vec();
        let error = dom
            .install_parsed_html_fragment(
                report,
                other_capture.as_ref().unwrap_or(&capture),
                limits(),
                &AtomicBool::new(which == 2),
            )
            .unwrap_err();
        match which {
            0 => assert!(matches!(error, DomParsedHtmlInstallError::ForeignContext)),
            1 => assert!(matches!(
                error,
                DomParsedHtmlInstallError::StaleDestination { .. }
            )),
            2 => assert!(matches!(error, DomParsedHtmlInstallError::Cancelled)),
            3 => assert!(matches!(
                error,
                DomParsedHtmlInstallError::HyperlinkUnavailable { .. }
            )),
            _ => unreachable!(),
        }
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            dom.changes_since(before.stamp()).unwrap().changes(),
            changes.as_slice()
        );
    }
}

#[test]
fn contextual_inactive_img_retains_original_wait_then_native_update_advances_its_identity() {
    use surgeist_html::{HtmlBrowsingContextPresence, HtmlImgFreshRequestState};
    use surgeist_html_parser::{
        HtmlParseImageUpdate, HtmlParseMetadataDocument, HtmlParseMetadataStep,
    };
    use surgeist_html_semantics::{HtmlCapturedNodeKind, HtmlImageSupport};
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let document_subject = capture.subject(document.node()).unwrap();
    let captured_document = capture.capture().node(&document_subject).unwrap();
    let HtmlCapturedNodeKind::Document(facts) = captured_document.kind() else {
        panic!("actual captured Document");
    };
    assert_eq!(
        facts.nonce_environment().browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    // Support is unavailable, not disabled: the actual first inactive guard wins.
    assert_eq!(facts.image_support(), HtmlImageSupport::Unavailable);
    assert_eq!(
        facts.parser_creation_image_support(),
        HtmlImageSupport::Unavailable
    );
    let report = fragment_report(&capture, host, HtmlSource::utf8("<img src=x>"), false);
    let auxiliary = report.fragment_parser_document().unwrap().document().id();
    let mut source_img = None;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(image) = row.image() {
                    assert!(source_img.is_none());
                    let id = image.subject().id();
                    assert_eq!(image.dimension_attribute_source().id(), id);
                    // Creation + src assignment + auxiliary-to-captured adoption.
                    assert_eq!(image.latest_started(), 3);
                    let HtmlParseImageUpdate::Waiting { generation, flags } = image.update() else {
                        panic!("actual inactive wait before support guard");
                    };
                    assert_eq!(generation, 3);
                    assert!(!flags.restart_animation());
                    assert!(!flags.maybe_omit_events());
                    assert_eq!(
                        image.state().current_request().state(),
                        HtmlImgFreshRequestState::Unavailable
                    );
                    source_img = Some(id);
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    let source_img = source_img.unwrap();
    let mut current_document_seen = false;
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) if row.node().id() == source_img => {
                if let Some(current) = row.node_document() {
                    let HtmlParseMetadataDocument::Captured(current) = current else {
                        panic!("final image nodeDocument is the actual captured target");
                    };
                    assert_eq!(current.id(), document_subject);
                    current_document_seen = true;
                }
            }
            HtmlParseMetadataStep::Complete => break,
            _ => {}
        }
    }
    assert!(current_document_seen);
    let before = dom.snapshot();
    let host_children = before.node(host).unwrap().children().to_vec();
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let image = installed.node_for(&source_img).unwrap();
    let original = installed.snapshot().node(image).unwrap().as_img().unwrap();
    assert_eq!(installed.document(), document);
    assert_eq!(
        installed.captured_node_for(&document_subject),
        Some(document.node())
    );
    assert_eq!(
        installed.snapshot().node(image).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(
        installed.snapshot().node(image).unwrap().parent(),
        Some(installed.root())
    );
    assert!(
        installed
            .snapshot()
            .node(installed.root())
            .unwrap()
            .parent()
            .is_none()
    );
    assert_eq!(
        installed.snapshot().node(host).unwrap().children(),
        host_children
    );
    // Bootstrap-only parser Document remains historical, not a fabricated live ID.
    assert_eq!(installed.node_for(&auxiliary), None);
    assert_eq!(original.dimension_attribute_source(), image);
    assert!(!original.parser_inserted());
    assert_eq!(original.form_owner().unwrap(), None);
    let state = original.state();
    assert_eq!(
        state.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(state.current_request().current_url().is_empty());
    assert_eq!(state.current_request().current_pixel_density(), 1);
    assert!(state.current_request().preferred_dimensions_are_null());
    assert!(!state.current_request().image_data_is_available());
    assert!(state.pending_request_is_null());
    assert!(state.last_selected_source_is_null());
    assert!(state.last_auto_sizes_width_is_null());
    assert!(state.lazy_load_resumption_is_null());
    let original_wait = original.suspended_update().unwrap().unwrap();
    assert_eq!(original_wait.id().target(), image);
    assert_eq!(original_wait.document(), document);
    assert!(!original_wait.flags().restart_animation());
    assert!(!original_wait.flags().maybe_omit_events());
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", "x");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let updated = dom.snapshot();
    let next = updated
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    assert_ne!(next.id(), original_wait.id());
    assert_eq!(next.id().target(), image);
    assert_eq!(next.document(), document);
    assert!(next.flags().restart_animation());
    assert!(!next.flags().maybe_omit_events());
    assert_eq!(
        original.suspended_update().unwrap().unwrap().id(),
        original_wait.id()
    );
    assert_eq!(installed.node_for(&source_img), Some(image));
    assert_eq!(
        installed.snapshot().node(host).unwrap().children(),
        host_children
    );
}

#[test]
fn contextual_exact_combined_owned_and_captured_admission_and_empty_result_are_real() {
    for (markup, minimum) in [("", 3), ("<p>x</p>", 5)] {
        for allowance in [minimum - 1, minimum] {
            let (mut dom, capture, document, host) = fragment_fixture("div");
            let report = fragment_report(&capture, host, HtmlSource::utf8(markup), false);
            let before = dom.snapshot();
            let usage = dom.usage();
            let outcome = dom.install_parsed_html_fragment(
                report,
                &capture,
                DomParsedHtmlInstallLimits::new(allowance, 10_000_000, 16_000_000, 100_000_000),
                &AtomicBool::new(false),
            );
            if allowance == minimum {
                let installed = outcome.unwrap();
                assert_eq!(
                    installed.bindings().len() + installed.captured_bindings().len(),
                    minimum
                );
                assert_eq!(installed.document(), document);
                assert_eq!(
                    installed
                        .snapshot()
                        .node(installed.root())
                        .unwrap()
                        .children()
                        .is_empty(),
                    markup.is_empty()
                );
            } else {
                assert!(matches!(
                    outcome,
                    Err(DomParsedHtmlInstallError::ResourceLimit {
                        resource: DomParsedHtmlInstallResource::MappedNodes,
                        ..
                    })
                ));
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
            }
        }
    }
}

#[test]
fn contextual_all_modes_and_current_target_platform_remain_distinct_from_new_auxiliary_selection() {
    use surgeist_html::{
        HtmlInputEmptySelectionSupport, HtmlInputNoneDirectionSupport, HtmlInputSelectionDirection,
    };
    use surgeist_html_semantics::HtmlCaptureLimits;
    for (doctype, mode) in [
        ("<!doctype html>", HtmlDocumentMode::NoQuirks),
        ("", HtmlDocumentMode::Quirks),
        (
            "<!doctype html PUBLIC '-//W3C//DTD XHTML 1.0 Transitional//EN' 'http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd'>",
            HtmlDocumentMode::LimitedQuirks,
        ),
    ] {
        let (mut dom, environment) = setup();
        let target_environment = DomParsedDocumentEnvironment::headless(
            environment.origin().clone(),
            environment.url().clone(),
            environment.encoding().clone(),
            environment.encoding_label().clone(),
            DomNativeInputPlatform::new(
                HtmlInputEmptySelectionSupport::Unsupported,
                HtmlInputNoneDirectionSupport::Unsupported,
            ),
        );
        let target = install(
            &mut dom,
            parse(HtmlSource::utf8(format!("{doctype}<div></div>"))),
            target_environment,
        );
        let document = target.document();
        let host = element(&target, "div");
        let original = target
            .snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .clone();
        let capture = dom
            .snapshot()
            .html_capture(
                document,
                None,
                HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
                DomHtmlCaptureLimits::new(1024, 1_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        let report = fragment_report(
            &capture,
            host,
            HtmlSource::utf8("<input><style>a{}</style><style>b{}</style>"),
            true,
        );
        let parser = report.fragment_parser_document().unwrap().document().id();
        let result = dom
            .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
            .unwrap();
        let auxiliary = result
            .snapshot()
            .node(result.node_for(&parser).unwrap())
            .unwrap()
            .as_document()
            .unwrap();
        assert_eq!(auxiliary.metadata().mode(), mode);
        assert_eq!(
            auxiliary.metadata().content_type(),
            &DocumentString::from("application/xml")
        );
        assert_eq!(
            auxiliary
                .metadata()
                .document_environment()
                .input_platform()
                .none_direction(),
            HtmlInputNoneDirectionSupport::Supported
        );
        assert_eq!(
            result.snapshot().document(document).unwrap().metadata(),
            &original
        );
        let input = result
            .snapshot()
            .node(element(&result, "input"))
            .unwrap()
            .as_input()
            .unwrap();
        assert_eq!(
            input.state().selection().unwrap().direction(),
            HtmlInputSelectionDirection::Forward
        );
        let styles: Vec<_> = result
            .bindings()
            .iter()
            .filter_map(|binding| result.snapshot().node(binding.node()).unwrap().as_style())
            .collect();
        assert_eq!(styles.len(), 2);
        assert!(
            styles
                .iter()
                .all(|style| style.parser_document() == Some(auxiliary.id()))
        );
    }
}

#[test]
fn contextual_table_and_rcdata_follow_the_original_context_without_reparsing_or_replacement() {
    let (mut dom, capture, _, host) = fragment_fixture("table");
    let report = fragment_report(&capture, host, HtmlSource::utf8("<tr><td>X"), false);
    let result = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let tbody = element(&result, "tbody");
    let tr = element(&result, "tr");
    let td = element(&result, "td");
    assert_eq!(
        result.snapshot().node(tbody).unwrap().parent(),
        Some(result.root())
    );
    assert_eq!(result.snapshot().node(tr).unwrap().parent(), Some(tbody));
    assert_eq!(result.snapshot().node(td).unwrap().parent(), Some(tr));
    assert!(result.snapshot().node(host).unwrap().children().is_empty());

    let (mut dom, capture, _, host) = fragment_fixture("textarea");
    let report = fragment_report(&capture, host, HtmlSource::utf8("<b>&amp;"), false);
    let result = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let units: Vec<_> = result
        .snapshot()
        .node(result.root())
        .unwrap()
        .children()
        .iter()
        .flat_map(|id| {
            result
                .snapshot()
                .node(*id)
                .unwrap()
                .data()
                .unwrap()
                .code_units()
                .iter()
                .copied()
        })
        .collect();
    assert_eq!(units, vec![60, 98, 62, 38]);
    assert_eq!(
        result
            .snapshot()
            .node(host)
            .unwrap()
            .as_textarea()
            .unwrap()
            .state()
            .raw_value(),
        &DocumentString::default()
    );
}

#[test]
fn contextual_unused_external_bank_is_history_and_never_establishes_current_owner_facts() {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::{
        HtmlBaseFactsLimits, HtmlCapturedBaseFacts, HtmlFragmentContext,
    };
    let (mut dom, capture, document, host) = fragment_fixture("div");
    let context =
        HtmlFragmentContext::element(capture.capture(), &capture.subject(host).unwrap()).unwrap();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let facts = HtmlCapturedBaseFacts::new(
        capture.capture(),
        capture.capture().host_facts_revision().clone(),
        revision,
        Vec::new(),
        Vec::new(),
        HtmlBaseFactsLimits::new(0, 0, 0),
    )
    .unwrap();
    let report = Arc::new(
        HtmlFragmentParseRequest::new(HtmlSource::utf8("<p>x</p>"), context, parse_limits())
            .unwrap()
            .with_base_facts(facts)
            .unwrap()
            .parse()
            .unwrap(),
    );
    let original_report = Arc::as_ptr(&report);
    let original_bank = report.captured_base_facts().unwrap() as *const HtmlCapturedBaseFacts;
    let bank_bytes = report
        .captured_base_facts()
        .unwrap()
        .retained_storage_usage()
        .retained_bytes();
    assert!(bank_bytes > 0);
    let before = dom.snapshot();
    assert_eq!(capture.basis().store(), before.stamp());
    assert!(before.document(document).unwrap().base_facts().is_none());
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    assert!(std::ptr::eq(original_report, installed.report()));
    assert!(std::ptr::eq(
        original_bank,
        installed.report().captured_base_facts().unwrap()
    ));
    assert_eq!(
        installed
            .report()
            .captured_base_facts()
            .unwrap()
            .retained_storage_usage()
            .retained_bytes(),
        bank_bytes
    );
    assert!(
        installed
            .snapshot()
            .document(document)
            .unwrap()
            .base_facts()
            .is_none()
    );
    assert_eq!(
        installed.snapshot().node(host).unwrap().children(),
        before.node(host).unwrap().children()
    );
    assert_eq!(
        installed
            .snapshot()
            .node(installed.root())
            .unwrap()
            .parent(),
        None
    );
}

#[test]
fn contextual_hosted_template_target_reuses_actual_inert_owner_without_overwriting_context_document()
 {
    use surgeist_html_parser::HtmlFragmentParseRequest;
    use surgeist_html_semantics::HtmlFragmentContext;
    let (mut dom, capture, outer, template) = fragment_fixture("template");
    let original = dom.snapshot();
    let contents = original
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = original.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, outer);
    let context = HtmlFragmentContext::hosted_fragment(
        capture.capture(),
        &capture.subject(contents).unwrap(),
    )
    .unwrap();
    assert_ne!(context.target_document(), context.context_document());
    let report = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<p>X</p><template><b>T</b></template>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    assert!(report.effects().unwrap().is_empty()); // actual existing inert owner reuse
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(installed.document(), inert);
    assert_eq!(installed.context_basis().unwrap().document(), outer);
    assert_eq!(
        installed.snapshot().document(outer).unwrap().metadata(),
        original.document(outer).unwrap().metadata()
    );
    assert_eq!(
        installed.snapshot().document(inert).unwrap().metadata(),
        original.document(inert).unwrap().metadata()
    );
    let nested = installed
        .snapshot()
        .node(element(&installed, "template"))
        .unwrap()
        .as_template()
        .unwrap();
    assert_eq!(
        installed
            .snapshot()
            .node(nested.contents().unwrap())
            .unwrap()
            .owner_document(),
        Some(inert)
    );
    assert!(
        installed
            .snapshot()
            .node(contents)
            .unwrap()
            .children()
            .is_empty()
    );
    let p = element(&installed, "p");
    let mut batch = DomBatch::new();
    batch.insert_before(installed.root(), contents, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(dom.snapshot().node(p).unwrap().parent(), Some(contents));
    assert_eq!(
        dom.snapshot().node(p).unwrap().owner_document(),
        Some(inert)
    );
}

#[test]
fn contextual_complete_capture_backing_has_exact_and_one_short_historical_admission() {
    use surgeist_html_semantics::HtmlCaptureLimits;
    for succeeds in [false, true] {
        let (mut dom, _, document, host) = fragment_fixture("div");
        let mut padded = Vec::with_capacity(16_384);
        padded.push(0xd800);
        let mut batch = DomBatch::new();
        batch.set_attribute(host, "data-padded", DocumentString::from_utf16(padded));
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let capture = dom
            .snapshot()
            .html_capture(
                document,
                None,
                HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
                DomHtmlCaptureLimits::new(1024, 1_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        let report = fragment_report(&capture, host, HtmlSource::utf8("<p>X</p>"), false);
        let retained = capture
            .capture()
            .retained_storage_usage_with(|| Ok::<(), std::convert::Infallible>(()))
            .unwrap()
            .retained_bytes();
        assert!(retained >= 32_768); // spare raw backing is retained by whole capture
        let exact = (std::mem::size_of::<HtmlParseReport>() as u64)
            + report.retained_construction_bytes()
            + (report.source().len() as u64)
            + (retained as u64);
        let before = dom.snapshot();
        let usage = dom.usage();
        let result = dom.install_parsed_html_fragment(
            report,
            &capture,
            DomParsedHtmlInstallLimits::new(
                1024,
                10_000_000,
                16_000_000,
                exact - u64::from(!succeeds),
            ),
            &AtomicBool::new(false),
        );
        if succeeds {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(DomParsedHtmlInstallError::ResourceLimit {
                    resource: DomParsedHtmlInstallResource::HistoricalBytes,
                    ..
                })
            ));
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
    }
}

#[test]
fn declarative_template_reference_does_not_own_adopted_destroyed_or_cloned_contents() {
    for destroy_template in [false, true] {
        let (mut dom, capture, document, host) = fragment_fixture("div");
        let report = fragment_report(
            &capture,
            host,
            HtmlSource::utf8("<template shadowrootmode=open><b>S</b></template>"),
            true,
        );
        let result = dom
            .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
            .unwrap();
        let template = element(&result, "template");
        let shadow = result
            .snapshot()
            .node(host)
            .unwrap()
            .hosted_shadow_root()
            .unwrap();
        assert_eq!(
            result
                .snapshot()
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            shadow
        );
        // Make the retained Template reachable by an ordinary capture; its
        // reference must not replace the existing root's actual host edge.
        let mut expose = DomBatch::new();
        expose.insert_before(template, host, None);
        dom.apply(expose, &AtomicBool::new(false)).unwrap();
        let original = dom.snapshot();
        let recapture = original
            .html_capture(
                document,
                None,
                surgeist_html_semantics::HtmlCaptureLimits::new(
                    1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216,
                ),
                DomHtmlCaptureLimits::new(1024, 1_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        let captured_template = recapture
            .capture()
            .node(&recapture.subject(template).unwrap())
            .unwrap();
        assert_eq!(
            captured_template.template_contents_kind(),
            Some(surgeist_html::HtmlTemplateContentsKind::ShadowReference)
        );
        let Some(surgeist_html_semantics::HtmlCapturedTemplateContents::Present(root)) =
            captured_template.template_contents()
        else {
            panic!("actual shadow reference required")
        };
        assert_eq!(recapture.node(root.subject()).unwrap(), shadow);
        let root = recapture.capture().node(root.subject()).unwrap();
        let Some(surgeist_html_semantics::HtmlCapturedHost::Element(captured_host)) = root.host()
        else {
            panic!("actual shadow host required")
        };
        assert_eq!(recapture.node(captured_host.subject()).unwrap(), host);
        let mut batch = DomBatch::new();
        let other = batch.create_document();
        batch.adopt(template, &other);
        let clone = batch.import_node(&original, template, &other, true);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let other = commit.document(&other).unwrap();
        let clone = commit.created(&clone).unwrap().unwrap();
        let snapshot = dom.snapshot();
        assert_eq!(
            snapshot.node(template).unwrap().owner_document(),
            Some(other)
        );
        assert_eq!(
            snapshot
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            shadow
        );
        assert_eq!(
            snapshot.node(shadow).unwrap().owner_document(),
            Some(document)
        );
        let cloned_contents = snapshot
            .node(clone)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        assert_ne!(cloned_contents, shadow);
        assert!(matches!(
            snapshot.node(cloned_contents).unwrap().kind(),
            DomNodeKind::DocumentFragment
        ));
        assert_eq!(
            snapshot.node(cloned_contents).unwrap().fragment_host(),
            Some(clone)
        );
        assert_eq!(snapshot.node(cloned_contents).unwrap().children().len(), 1);
        // An actual late invalid command must roll back retirement and every
        // incoming weak reference, even after all private destruction work.
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut invalid = DomBatch::new();
        invalid.detach(host);
        invalid.destroy(host);
        invalid.set_attribute(document.node(), "bad", "x");
        assert_eq!(
            dom.apply(invalid, &AtomicBool::new(false)).unwrap_err(),
            DomError::WrongNodeKind
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            dom.snapshot()
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            shadow
        );
        let mut destroy = DomBatch::new();
        if !destroy_template {
            destroy.detach(host);
        }
        destroy.destroy(if destroy_template { template } else { host });
        dom.apply(destroy, &AtomicBool::new(false)).unwrap();
        if destroy_template {
            assert_eq!(
                dom.snapshot().node(host).unwrap().hosted_shadow_root(),
                Some(shadow)
            );
            assert_eq!(dom.snapshot().shadow_root(shadow).unwrap().host(), host);
        } else {
            assert_eq!(
                dom.snapshot()
                    .node(template)
                    .unwrap()
                    .as_template()
                    .unwrap()
                    .contents(),
                Err(DomError::TemplateContentsUnavailable)
            );
            let mut reuse = DomBatch::new();
            for _ in 0..16 {
                reuse.create_element_ns(other, Namespace::html(), "span");
            }
            dom.apply(reuse, &AtomicBool::new(false)).unwrap();
            assert_eq!(
                dom.snapshot()
                    .node(template)
                    .unwrap()
                    .as_template()
                    .unwrap()
                    .contents(),
                Err(DomError::TemplateContentsUnavailable)
            );
        }
        assert_eq!(
            original
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            shadow
        );
        assert_eq!(original.node(shadow).unwrap().children().len(), 1);
        assert_eq!(
            dom.snapshot()
                .node(clone)
                .unwrap()
                .as_template()
                .unwrap()
                .contents()
                .unwrap(),
            cloned_contents
        );
    }
}
fn element(result: &DomParsedHtmlInstallation, local: &str) -> NodeId {
    result
        .bindings()
        .iter()
        .find_map(|binding| {
            result
                .snapshot()
                .node(binding.node())
                .unwrap()
                .as_element()
                .filter(|element| {
                    element.name().namespace_uri() == Namespace::html().uri()
                        && element
                            .name()
                            .local_name()
                            .code_units()
                            .iter()
                            .copied()
                            .eq(local.encode_utf16())
                })
                .map(|_| binding.node())
        })
        .unwrap()
}

#[test]
fn completed_utf16_mode_and_text_survive_one_publication_and_later_edit() {
    // HTML preprocessing preserves FEFF and the isolated surrogate, and maps
    // CRLF to one LF. Missing doctype selects Quirks, without a native skeleton.
    let raw = vec![0xfeff, 60, 112, 62, 65, 0xd800, 13, 10, 66, 60, 47, 112, 62];
    let report = parse(HtmlSource::utf16(DocumentString::from_utf16(raw.clone())));
    let identity = report.source().identity().clone();
    let root = report.document().unwrap().root().id();
    let (mut dom, environment) = setup();
    let before = dom.snapshot();
    let result = install(&mut dom, report.clone(), environment);
    assert_eq!(result.source(), &identity);
    assert_eq!(result.source_root(), &root);
    assert_eq!(result.node_for(&root), Some(result.document().node()));
    assert_eq!(result.bindings().len(), 7); // Document/html/head/body/p/two Text.
    assert_eq!(
        result.snapshot().nodes().count(),
        before.nodes().count() + 7
    );
    assert_eq!(
        result.snapshot().revision().get(),
        before.revision().get() + 1
    );
    assert_eq!(
        result
            .snapshot()
            .document(result.document())
            .unwrap()
            .metadata()
            .mode(),
        HtmlDocumentMode::Quirks
    );
    let p = element(&result, "p");
    let children = result.snapshot().node(p).unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(
        result
            .snapshot()
            .node(children[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [65, 0xd800, 10, 66]
    );
    assert_eq!(
        result.report().source().as_utf16().unwrap().code_units(),
        raw
    );
    assert_eq!(
        result.association_readiness(),
        DomParsedHtmlAssociationReadiness::Pending
    );
    let mut b = DomBatch::new();
    b.set_attribute(p, "class", "later");
    dom.apply(b, &AtomicBool::new(false)).unwrap();
    drop(report);
    assert!(
        result
            .snapshot()
            .node(p)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"class".into())
            .is_none()
    );
    assert_eq!(
        dom.snapshot()
            .node(p)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"class".into()),
        Some(&DocumentString::from("later"))
    );
    assert_eq!(
        result.report().source().as_utf16().unwrap().code_units(),
        raw
    );
}

#[test]
fn saved_creation_state_is_independent_of_recovered_attribute_merge_and_foreign_results() {
    let source = HtmlSource::utf8(
        "<!doctype html><body><body is=x-control nonce=late><x-card></x-card><div a:b=first a:b=second></div>",
    );
    let report = parse(source.clone());
    let another = parse(source);
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let body = result
        .snapshot()
        .node(element(&result, "body"))
        .unwrap()
        .as_element()
        .unwrap();
    assert_eq!(
        body.custom_element_state(),
        HtmlCustomElementState::Uncustomized
    );
    assert_eq!(body.saved_is(), None);
    assert_eq!(body.nonce(), Some(&DocumentString::from("late")));
    assert_eq!(
        body.attribute(&"is".into()),
        Some(&DocumentString::from("x-control"))
    );
    assert_eq!(
        result
            .snapshot()
            .node(element(&result, "x-card"))
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_state(),
        HtmlCustomElementState::Undefined
    );
    let div = result
        .snapshot()
        .node(element(&result, "div"))
        .unwrap()
        .as_element()
        .unwrap();
    assert_eq!(div.attributes().len(), 1);
    let first = &div.attributes()[0];
    assert_eq!(first.name().local_name(), &DocumentString::from("a:b"));
    assert_eq!(first.value(), &DocumentString::from("first"));
    assert!(first.name().prefix().is_none());
    assert!(
        result
            .node_for(&another.document().unwrap().root().id())
            .is_none()
    );
}

#[test]
fn input_current_value_and_parser_owner_survive_then_native_setter_edits_canonical_state() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><form id=f><input type=text value='initial'></form>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let input = element(&result, "input");
    let form = element(&result, "form");
    let original = result.snapshot().node(input).unwrap().as_input().unwrap();
    assert_eq!(
        original.value(),
        "initial".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(original.form_owner().unwrap(), Some(form));
    assert!(original.state().parser_inserted());
    assert!(!original.state().flags().dirty_value());
    let mut b = DomBatch::new();
    b.set_input_value(input, "current");
    dom.apply(b, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let input = current.node(input).unwrap().as_input().unwrap();
    assert_eq!(input.value(), "current".encode_utf16().collect::<Vec<_>>());
    assert!(input.state().flags().dirty_value());
    assert_eq!(
        original.value(),
        "initial".encode_utf16().collect::<Vec<_>>()
    );
}

#[test]
fn disabled_script_keeps_all_flags_preparation_document_nonce_and_real_parser_cause() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><script nonce=secret src=external.js></script>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let id = element(&result, "script");
    let node = result.snapshot().node(id).unwrap();
    let script = node.as_html_script().unwrap();
    let flags = script.flags();
    assert!(!flags.force_async());
    assert!(flags.already_started());
    assert!(!flags.external_file());
    assert!(!flags.ready_to_be_parser_executed());
    assert!(!flags.delaying_load());
    assert_eq!(script.parser_document(), Some(result.document()));
    assert_eq!(script.preparation_document(), Some(result.document()));
    assert_eq!(
        script.script_type(),
        Some(surgeist_html::HtmlScriptKind::Classic)
    );
    assert_eq!(
        script.result(),
        surgeist_html::HtmlScriptResult::Uninitialized
    );
    assert_eq!(script.steps_when_result_ready(), None);
    assert_eq!(
        node.as_element().unwrap().nonce(),
        Some(&DocumentString::from("secret"))
    );
}

#[test]
fn table_parser_form_pointer_preserves_nonancestor_form_owner() {
    // In-table form insertion pops that Form from the stack while retaining
    // the parser form pointer. The later Input inside the cell belongs to it.
    let report = parse(HtmlSource::utf8(
        "<!doctype html><table><form id=f><tr><td><input value=X></table>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let input = element(&result, "input");
    let form = element(&result, "form");
    let node = result.snapshot().node(input).unwrap();
    assert_ne!(node.parent(), Some(form));
    assert_eq!(node.as_input().unwrap().form_owner().unwrap(), Some(form));
    assert!(node.as_input().unwrap().state().parser_inserted());
}

#[test]
fn eof_template_patch_retains_nonchild_subjects_and_each_weak_marker_slot_retires_exactly() {
    let report = parse(HtmlSource::utf8(
        "<body><div><?marker name='x'?></div><template for=x><p>N",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let template = element(&result, "template");
    let target = element(&result, "div");
    let marker = result
        .bindings()
        .iter()
        .find_map(|row| {
            matches!(
                result.snapshot().node(row.node()).unwrap().kind(),
                DomNodeKind::ProcessingInstruction(_)
            )
            .then_some(row.node())
        })
        .unwrap();
    let DomNodeKind::ProcessingInstruction(pi) = result.snapshot().node(marker).unwrap().kind()
    else {
        panic!("retained marker PI")
    };
    assert_eq!(pi.target(), &DocumentString::from("marker"));
    assert_eq!(
        pi.attribute(&DocumentString::from("name")),
        Some(&DocumentString::from("x"))
    );
    assert_eq!(pi.attributes().len(), 1);
    let template_view = result
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap();
    let original = template_view.insertion_state();
    assert_eq!(original.target(), Some(target));
    assert_eq!(original.start(), Some(marker));
    assert_eq!(original.end(), Some(marker));
    assert!(result.snapshot().node(template).unwrap().parent().is_none());
    let contents = template_view.contents().unwrap();
    assert_eq!(
        result.snapshot().node(contents).unwrap().fragment_host(),
        Some(template)
    );
    let inert = result
        .snapshot()
        .node(contents)
        .unwrap()
        .owner_document()
        .unwrap();
    assert_ne!(inert, result.document());
    assert_eq!(
        result
            .snapshot()
            .document(result.document())
            .unwrap()
            .associated_inert_template_document(),
        Some(inert)
    );
    let mut detach = DomBatch::new();
    detach.adopt(marker, result.document());
    dom.apply(detach, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .insertion_state(),
        original
    );
    let mut destroy = DomBatch::new();
    destroy.destroy(marker);
    let commit = dom.apply(destroy, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let now = current
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .insertion_state();
    assert_eq!(now.target(), Some(target));
    assert_eq!(now.start(), None);
    assert_eq!(now.end(), None);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::TemplateRelations)
    );
    assert_eq!(
        result
            .snapshot()
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .insertion_state(),
        original
    );
    assert!(result.snapshot().node(marker).is_ok());
    assert!(current.node(marker).is_err());
}

#[test]
fn nested_template_owners_share_actual_inert_document_and_svg_parser_cause_survives_later_insertion()
 {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><template><template><svg><script/></svg></template></template><template></template>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let templates: Vec<_> = result
        .bindings()
        .iter()
        .filter_map(|row| result.snapshot().node(row.node()).unwrap().as_template())
        .collect();
    assert_eq!(templates.len(), 3);
    let inert = result
        .snapshot()
        .node(templates[0].contents().unwrap())
        .unwrap()
        .owner_document()
        .unwrap();
    assert_ne!(inert, result.document());
    assert_eq!(
        result.snapshot().document(inert).unwrap().template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    let inert_metadata = result.snapshot().document(inert).unwrap().metadata();
    assert_eq!(inert_metadata.url(), &DocumentString::from("about:blank"));
    assert_eq!(inert_metadata.encoding(), &DocumentString::from("UTF-8"));
    assert!(matches!(
        inert_metadata.creation_cause(),
        DomDocumentCreationCause::ParsedTemplateOwner
    ));
    assert_eq!(
        result
            .snapshot()
            .document(result.document())
            .unwrap()
            .associated_inert_template_document(),
        Some(inert)
    );
    for template in templates {
        let contents = result
            .snapshot()
            .node(template.contents().unwrap())
            .unwrap();
        assert_eq!(contents.owner_document().unwrap(), inert);
        assert_eq!(contents.fragment_host(), Some(template.id()));
    }
    let script_id = result
        .bindings()
        .iter()
        .find_map(|row| {
            result
                .snapshot()
                .node(row.node())
                .unwrap()
                .as_svg_script()
                .map(|_| row.node())
        })
        .unwrap();
    let original = result
        .snapshot()
        .node(script_id)
        .unwrap()
        .as_svg_script()
        .unwrap();
    assert_eq!(original.origin(), DomSvgScriptOrigin::Parser);
    assert!(!original.flags().already_processed());
    let body = element(&result, "body");
    let mut b = DomBatch::new();
    b.insert_before(script_id, body, None);
    dom.apply(b, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let moved = current.node(script_id).unwrap().as_svg_script().unwrap();
    assert_eq!(moved.origin(), DomSvgScriptOrigin::Parser);
    assert!(moved.flags().already_processed());
    assert!(!original.flags().already_processed());
}

#[test]
fn qualified_base_freezing_and_link_parser_history_transfer_without_native_response_relabeling() {
    use surgeist_html_semantics::{
        HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseFactsLimits,
        HtmlBaseParseObservation, HtmlBaseParseOutcome, HtmlParseBaseFacts,
    };
    let (mut dom, environment) = setup();
    dom.set_limits(dom.limits().with_base_processing(8, 100_000).unwrap())
        .unwrap();
    let revision = environment.url().revision().clone();
    let parsed = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/ordinary/sub/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let observation = HtmlBaseParseObservation::new(
        "sub/".into(),
        environment.url().clone(),
        environment.encoding().clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Parsed {
            record: parsed.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    )
    .unwrap();
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(environment.url().clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(environment.encoding().clone()),
        )
        .unwrap(),
        vec![observation],
        HtmlBaseFactsLimits::new(8, 4096, 100_000),
    )
    .unwrap();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><base href='sub/'><link rel=stylesheet href=sheet.css disabled>",
            ),
            parse_limits(),
        )
        .with_base_facts(facts.clone())
        .unwrap()
        .parse()
        .unwrap(),
    );
    let before = dom.snapshot();
    for (url, encoding) in [
        (environment.url().clone(), HtmlBaseEncoding::fresh()),
        (
            HtmlBaseUrlRecord::observed(
                revision.clone(),
                "https://example.test/ordinary/".into(),
                HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap(),
            environment.encoding().clone(),
        ),
    ] {
        let mismatch = DomParsedDocumentEnvironment::headless(
            environment.origin().clone(),
            url,
            encoding,
            environment.encoding_label().clone(),
            environment.input_platform(),
        );
        assert!(matches!(
            dom.install_parsed_html_document(
                before.stamp(),
                report.clone(),
                mismatch,
                limits(),
                &AtomicBool::new(false),
            ),
            Err(DomParsedHtmlInstallError::EnvironmentMismatch)
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
    let result = install(&mut dom, report, environment.clone());
    let document = result.snapshot().document(result.document()).unwrap();
    assert_eq!(document.metadata().url(), environment.url().serialization());
    assert!(
        matches!(document.metadata().creation_cause(), DomDocumentCreationCause::ParsedOffline { url, encoding } if url == environment.url() && encoding == environment.encoding())
    );
    assert!(document.base_facts().unwrap().same_owner(&facts));
    let base = element(&result, "base");
    assert_eq!(document.first_base().unwrap(), Some(base));
    let frozen = result
        .snapshot()
        .node(base)
        .unwrap()
        .as_base()
        .unwrap()
        .frozen()
        .unwrap();
    assert_eq!(frozen.value(), &parsed);
    assert_eq!(frozen.document(), result.document());
    assert_eq!(frozen.revision(), &revision);
    assert_eq!(frozen.response(), DomBaseResponse::ParsedCompleted);
    let link = result
        .snapshot()
        .node(element(&result, "link"))
        .unwrap()
        .as_link()
        .unwrap();
    assert_eq!(link.parser_document(), Some(result.document()));
    assert!(link.history().created_by_node_document_parser());
    assert!(!link.flags().explicitly_enabled());
}

#[test]
fn full_original_and_environment_backing_admit_exact_budget_and_refuse_one_byte_short() {
    use std::mem::size_of;
    let mut raw = Vec::with_capacity(100_000);
    raw.extend("<p>X".encode_utf16());
    let report = parse(HtmlSource::utf16(DocumentString::from_utf16(raw)));
    let (mut dom, ordinary) = setup();
    let mut padded_url = Vec::with_capacity(10_000);
    padded_url.extend("https://example.test/ordinary/".encode_utf16());
    let url = HtmlBaseUrlRecord::observed(
        ordinary.url().revision().clone(),
        DocumentString::from_utf16(padded_url),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let mut label = Vec::with_capacity(1_000);
    label.extend("UTF-8".encode_utf16());
    let label = DocumentString::from_utf16(label);
    let environment = DomParsedDocumentEnvironment::headless(
        ordinary.origin().clone(),
        url,
        ordinary.encoding().clone(),
        label,
        ordinary.input_platform(),
    );
    let original = report.source().as_utf16().unwrap();
    assert!(original.shared_code_units().capacity() > original.len());
    // Public owner-issued footprints plus public supplied Vec backing. No URL
    // payload layout or parser-private source inventory is reimplemented here.
    let bytes = report.retained_construction_bytes()
        + size_of::<HtmlParseReport>() as u64
        + (size_of::<Vec<u16>>() + original.shared_code_units().capacity() * size_of::<u16>())
            as u64
        + environment.url().retained_storage_usage().retained_bytes() as u64
        + (size_of::<Vec<u16>>()
            + environment.encoding_label().shared_code_units().capacity() * size_of::<u16>())
            as u64;
    let before = dom.snapshot();
    let usage = dom.usage();
    let short = DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, bytes - 1);
    assert!(
        matches!(dom.install_parsed_html_document(before.stamp(), report.clone(), environment.clone(), short, &AtomicBool::new(false)), Err(DomParsedHtmlInstallError::ResourceLimit { resource: DomParsedHtmlInstallResource::HistoricalBytes, limit }) if limit == bytes - 1)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let exact = DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, bytes);
    let result = dom
        .install_parsed_html_document(
            before.stamp(),
            report,
            environment,
            exact,
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(
        result.report().source().as_utf16().unwrap().code_units(),
        "<p>X".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        result
            .snapshot()
            .node(element(&result, "p"))
            .unwrap()
            .children()
            .len(),
        1
    );
}

#[test]
fn closed_canvas_none_state_uses_actual_parsed_dimensions_and_later_owner_attribute_rules() {
    use surgeist_html::HtmlCanvasBitmapDimension;
    let report = parse(HtmlSource::utf8(
        "<!doctype html><canvas width=' +0042trailing' height='-1'>fallback</canvas>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let id = element(&result, "canvas");
    let original = result
        .snapshot()
        .node(id)
        .unwrap()
        .as_element()
        .unwrap()
        .canvas_none_state()
        .unwrap();
    assert!(original.context_is_none());
    assert!(original.origin_clean());
    assert!(original.bitmap_is_transparent_black());
    assert!(
        matches!(original.width(), HtmlCanvasBitmapDimension::Parsed(value) if value.to_u128() == Ok(42))
    );
    assert_eq!(original.height(), HtmlCanvasBitmapDimension::Default(150));
    let mut b = DomBatch::new();
    b.set_attribute(id, "width", "7");
    dom.apply(b, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let state = current
        .node(id)
        .unwrap()
        .as_element()
        .unwrap()
        .canvas_none_state()
        .unwrap();
    assert!(
        matches!(state.width(), HtmlCanvasBitmapDimension::Parsed(value) if value.to_u128() == Ok(7))
    );
    assert!(
        matches!(original.width(), HtmlCanvasBitmapDimension::Parsed(value) if value.to_u128() == Ok(42))
    );
}

#[test]
fn parsed_select_cache_textarea_raw_and_output_null_continue_through_canonical_setters() {
    use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};
    let report = parse(HtmlSource::utf8(
        "<!doctype html><form><select><option selected>A<option selected>B</select><textarea>\nA\r\nB</textarea><output>D</output></form>",
    ));
    let (mut dom, environment) = setup();
    dom.set_limits(
        dom.limits()
            .with_select_processing(128, 4_000_000, 10_000_000)
            .unwrap()
            .with_select_value_queries(
                HtmlCaptureLimits::new(1024, 4096, 1024, 1_000_000, 1024, 0, 0, 16_777_216),
                HtmlSemanticLimits::new(1_000_000, 32, 4096, 1024, 4_000_000, 4096),
            ),
    )
    .unwrap();
    let result = install(&mut dom, report, environment);
    let form = element(&result, "form");
    let select = element(&result, "select");
    let textarea = element(&result, "textarea");
    let output = element(&result, "output");
    let old = result.snapshot();
    let select_view = old.node(select).unwrap().as_select().unwrap();
    assert_eq!(select_view.form_owner().unwrap(), Some(form));
    assert!(!select_view.state().flags().user_validity());
    let options = old.node(select).unwrap().children();
    assert_eq!(options.len(), 2);
    for (id, selected) in options.iter().zip([false, true]) {
        let option = old.node(*id).unwrap().as_option().unwrap();
        assert_eq!(option.cached_select().unwrap(), Some(select));
        assert_eq!(option.state().flags().selectedness(), selected);
        assert!(!option.state().flags().dirtiness());
    }
    let text_view = old.node(textarea).unwrap().as_textarea().unwrap();
    assert_eq!(text_view.form_owner().unwrap(), Some(form));
    assert_eq!(text_view.state().raw_value(), &DocumentString::from("A\nB"));
    assert!(!text_view.state().flags().dirty_value());
    assert!(!text_view.state().flags().user_validity());
    assert_eq!(text_view.state().selection().cursor_position(), Some(0));
    let output_view = old.node(output).unwrap().as_output().unwrap();
    assert_eq!(output_view.form_owner().unwrap(), Some(form));
    assert_eq!(output_view.state().default_value_override(), None);
    let mut batch = DomBatch::new();
    batch.set_select_value(select, "A");
    batch.set_textarea_value(textarea, "new");
    batch.set_output_value(output, "new");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    for (id, selected) in options.iter().zip([true, false]) {
        let option = current.node(*id).unwrap().as_option().unwrap();
        assert_eq!(option.cached_select().unwrap(), Some(select));
        assert_eq!(option.state().flags().selectedness(), selected);
    }
    let text = current.node(textarea).unwrap().as_textarea().unwrap();
    assert_eq!(text.state().raw_value(), &DocumentString::from("new"));
    assert!(text.state().flags().dirty_value());
    assert_eq!(text_view.state().raw_value(), &DocumentString::from("A\nB"));
    assert_eq!(
        current
            .node(output)
            .unwrap()
            .as_output()
            .unwrap()
            .state()
            .default_value_override(),
        Some(&DocumentString::from("D"))
    );
    assert_eq!(output_view.state().default_value_override(), None);
}

#[test]
fn hyperlink_source_time_base_failed_parse_and_adoption_keep_the_actual_cache() {
    use surgeist_html_semantics::{
        HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseFactsLimits,
        HtmlBaseParseObservation, HtmlBaseParseOutcome, HtmlEncodingUrlParseFacts,
        HtmlEncodingUrlParseObservation, HtmlEncodingUrlParseOutcome, HtmlParseBaseFacts,
    };
    let (mut dom, environment) = setup();
    dom.set_limits(dom.limits().with_base_processing(8, 100_000).unwrap())
        .unwrap();
    let revision = environment.url().revision().clone();
    let sub = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/ordinary/sub/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let early = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/ordinary/early".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let limits = || HtmlBaseFactsLimits::new(8, 10_000, 100_000);
    let base = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(environment.url().clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(environment.encoding().clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "sub/".into(),
                environment.url().clone(),
                environment.encoding().clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: sub.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        limits(),
    )
    .unwrap();
    let observations = vec![
        HtmlEncodingUrlParseFacts::new(
            environment.url().clone(),
            environment.encoding().clone(),
            revision.clone(),
            vec![HtmlEncodingUrlParseObservation::new(
                "early".into(),
                HtmlEncodingUrlParseOutcome::Parsed(early.clone()),
            )],
            limits(),
        )
        .unwrap(),
        HtmlEncodingUrlParseFacts::new(
            sub.clone(),
            environment.encoding().clone(),
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "broken".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            limits(),
        )
        .unwrap(),
    ];
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><a href=early>E</a><base href=sub/><area href=broken>",
            ),
            parse_limits(),
        )
        .with_base_facts(base)
        .unwrap()
        .with_encoding_url_facts(observations)
        .parse()
        .unwrap(),
    );
    let result = install(&mut dom, report, environment.clone());
    let a = element(&result, "a");
    let area = element(&result, "area");
    let original = result.snapshot().node(a).unwrap().as_hyperlink().unwrap();
    assert_eq!(original.url(), Some(&early));
    assert_eq!(
        original.parse_observation().unwrap().base(),
        environment.url()
    );
    assert_eq!(original.parse_document(), Some(result.document()));
    let failed = result
        .snapshot()
        .node(area)
        .unwrap()
        .as_hyperlink()
        .unwrap();
    assert_eq!(failed.url(), None);
    assert_eq!(failed.parse_observation().unwrap().base(), &sub);
    assert!(matches!(
        failed.parse_observation().unwrap().outcome(),
        HtmlEncodingUrlParseOutcome::Failure
    ));
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    batch.adopt(a, &destination);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(
        current.node(a).unwrap().owner_document().unwrap(),
        commit.document(&destination).unwrap()
    );
    let adopted = current.node(a).unwrap().as_hyperlink().unwrap();
    assert_eq!(adopted.url(), Some(&early));
    assert_eq!(adopted.parse_document(), Some(result.document()));
    assert_eq!(original.url(), Some(&early));
}

#[test]
fn absent_hyperlink_is_actual_null_but_missing_present_href_observation_refuses_atomically() {
    let (mut dom, environment) = setup();
    let result = install(
        &mut dom,
        parse(HtmlSource::utf8("<!doctype html><a></a><area>")),
        environment.clone(),
    );
    for local in ["a", "area"] {
        let hyperlink = result
            .snapshot()
            .node(element(&result, local))
            .unwrap()
            .as_hyperlink()
            .unwrap();
        assert_eq!(hyperlink.url(), None);
        assert_eq!(hyperlink.parse_observation(), None);
    }
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            parse(HtmlSource::utf8("<!doctype html><a href=unobserved>X</a>")),
            environment,
            limits(),
            &AtomicBool::new(false),
        ),
        Err(DomParsedHtmlInstallError::HyperlinkUnavailable { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn parsed_img_wait_is_real_owned_work_then_later_update_and_exact_retirement_use_canonical_ids() {
    use surgeist_html::HtmlImgFreshRequestState;
    let (mut dom, environment) = setup();
    let result = install(
        &mut dom,
        parse(HtmlSource::utf8("<!doctype html><form><img src=x></form>")),
        environment,
    );
    let image = element(&result, "img");
    let form = element(&result, "form");
    let original = result.snapshot().node(image).unwrap().as_img().unwrap();
    assert_eq!(original.dimension_attribute_source(), image);
    assert_eq!(original.form_owner().unwrap(), Some(form));
    assert!(original.parser_inserted());
    let state = original.state();
    assert_eq!(
        state.current_request().state(),
        HtmlImgFreshRequestState::Unavailable
    );
    assert!(state.current_request().current_url().is_empty());
    assert_eq!(state.current_request().current_pixel_density(), 1);
    assert!(state.current_request().preferred_dimensions_are_null());
    assert!(!state.current_request().image_data_is_available());
    assert!(state.pending_request_is_null());
    assert!(state.last_selected_source_is_null());
    assert!(state.last_auto_sizes_width_is_null());
    assert!(state.lazy_load_resumption_is_null());
    let first = original.suspended_update().unwrap().unwrap();
    assert_eq!(first.id().target(), image);
    assert_eq!(first.document(), result.document());
    assert!(!first.flags().restart_animation());
    assert!(!first.flags().maybe_omit_events());
    let mut batch = DomBatch::new();
    batch.set_attribute(image, "src", "x");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let next = current
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    assert_ne!(next.id(), first.id());
    assert!(next.flags().restart_animation());
    assert!(!next.flags().maybe_omit_events());
    assert_eq!(
        original.suspended_update().unwrap().unwrap().id(),
        first.id()
    );
    let mut batch = DomBatch::new();
    batch.detach(image);
    let removal = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(
        !removal
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::ImageUpdateCancelled { .. }))
    );
    let detached = dom.snapshot();
    let detached_wait = detached
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    // Removal from Form does not satisfy the old-parent Picture update guard.
    assert_eq!(detached_wait.id(), next.id());
    assert_eq!(detached_wait.flags(), next.flags());
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    batch.adopt(image, &destination);
    let adoption = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(
        !adoption
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::ImageUpdateCancelled { .. }))
    );
    let adopted = dom.snapshot();
    let retiring = adopted
        .node(image)
        .unwrap()
        .as_img()
        .unwrap()
        .suspended_update()
        .unwrap()
        .unwrap();
    assert_ne!(retiring.id(), next.id());
    assert_eq!(
        retiring.document(),
        adoption.document(&destination).unwrap()
    );
    assert!(!retiring.flags().restart_animation());
    let mut batch = DomBatch::new();
    batch.destroy(image);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert!(commit.changes().changes().iter().any(|change| matches!(change,
        DomChange::ImageUpdateCancelled { update, flags, reason: DomImageUpdateCancellationReason::TargetDestroyed }
        if *update == retiring.id() && *flags == retiring.flags()
    )));
    assert!(dom.snapshot().node(image).is_err());
    assert_eq!(
        original.suspended_update().unwrap().unwrap().id(),
        first.id()
    );
}

#[test]
fn parsed_style_retains_parser_and_general_clone_causes_with_real_pending_associations() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><style>a{}</style><template><style>b{}</style></template><select><button><selectedcontent></selectedcontent></button><option selected><style>c{}</style></option></select>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let mut roles = Vec::new();
    let mut head = None;
    for binding in result.bindings() {
        let node = result.snapshot().node(binding.node()).unwrap();
        let Some(style) = node.as_style() else {
            continue;
        };
        assert_eq!(
            style.state().association_availability(),
            DomStyleSheetAvailability::PendingParsedProcessing
        );
        assert!(matches!(
            style.sheet(),
            Err(DomError::StyleProcessingUnavailable)
        ));
        assert!(!style.state().creation_state().on_parser_stack());
        let parent = result.snapshot().node(node.parent().unwrap()).unwrap();
        let role = match parent.kind() {
            DomNodeKind::DocumentFragment => {
                assert_ne!(node.owner_document().unwrap(), result.document());
                "template"
            }
            DomNodeKind::Element(element)
                if element.name().local_name() == &DocumentString::from("head") =>
            {
                head = Some(node.id());
                "head"
            }
            DomNodeKind::Element(element)
                if element.name().local_name() == &DocumentString::from("option") =>
            {
                "option"
            }
            DomNodeKind::Element(element)
                if element.name().local_name() == &DocumentString::from("selectedcontent") =>
            {
                "clone"
            }
            other => panic!("unexpected literal Style parent {other:?}"),
        };
        let parser_created = role != "clone";
        assert_eq!(
            style.state().creation_state().created_by_document_parser(),
            parser_created
        );
        assert_eq!(
            style.parser_document(),
            parser_created.then_some(result.document())
        );
        roles.push(role);
    }
    roles.sort_unstable();
    assert_eq!(roles, ["clone", "head", "option", "template"]);
    let head = head.unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(head, "class", "canonical-edit");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    let style = current.node(head).unwrap().as_style().unwrap();
    assert_eq!(style.parser_document(), Some(result.document()));
    assert_eq!(
        style.state().association_availability(),
        DomStyleSheetAvailability::PendingParsedProcessing
    );
    assert!(matches!(
        style.sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    assert!(
        result
            .snapshot()
            .node(head)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("class"))
            .is_none()
    );
}

#[test]
fn template_hyperlink_uses_its_actual_inert_document_facts_and_cannot_inherit_outer_facts() {
    use surgeist_html_semantics::{
        HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseFactsLimits, HtmlEncodingUrlParseFacts,
        HtmlEncodingUrlParseObservation, HtmlEncodingUrlParseOutcome, HtmlParseBaseFacts,
    };
    let (mut dom, environment) = setup();
    let limits = || HtmlBaseFactsLimits::new(8, 10_000, 100_000);
    let outer = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            environment.url().revision().clone(),
            Some(environment.url().clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(environment.encoding().clone()),
        )
        .unwrap(),
        vec![],
        limits(),
    )
    .unwrap();
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 4, 5, 6);
    let blank = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let encoding = HtmlBaseEncoding::fresh();
    let inert = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(blank.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        limits(),
    )
    .unwrap();
    let target = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://inert.example/x".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let observations = HtmlEncodingUrlParseFacts::new(
        blank.clone(),
        encoding.clone(),
        revision,
        vec![HtmlEncodingUrlParseObservation::new(
            "https://inert.example/x".into(),
            HtmlEncodingUrlParseOutcome::Parsed(target.clone()),
        )],
        limits(),
    )
    .unwrap();
    let source = HtmlSource::utf8(
        "<!doctype html><template><a href='https://inert.example/x'>X</a></template>",
    );
    let missing = Arc::new(
        HtmlDocumentParseRequest::offline(source.clone(), parse_limits())
            .with_base_facts(outer.clone())
            .unwrap()
            .with_encoding_url_facts(vec![observations.clone()])
            .parse()
            .unwrap(),
    );
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            missing,
            environment.clone(),
            self::limits(),
            &AtomicBool::new(false)
        ),
        Err(DomParsedHtmlInstallError::HyperlinkUnavailable { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(source, parse_limits())
            .with_base_facts(outer)
            .unwrap()
            .with_template_base_facts(inert.clone())
            .unwrap()
            .with_encoding_url_facts(vec![observations])
            .parse()
            .unwrap(),
    );
    // Complete construction admission includes selected associated graphs;
    // separately lent root/inert BaseFacts remain additional real owners.
    let bytes = report.retained_construction_bytes()
        + std::mem::size_of::<HtmlParseReport>() as u64
        + report.source().len() as u64
        + report
            .document_base_facts()
            .unwrap()
            .retained_storage_usage()
            .retained_bytes() as u64
        + inert.retained_storage_usage().retained_bytes() as u64
        + environment.url().retained_storage_usage().retained_bytes() as u64
        + (std::mem::size_of::<Vec<u16>>()
            + environment.encoding_label().shared_code_units().capacity()
                * std::mem::size_of::<u16>()) as u64;
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, bytes - 1),
            &AtomicBool::new(false),
        ),
        Err(DomParsedHtmlInstallError::ResourceLimit {
            resource: DomParsedHtmlInstallResource::HistoricalBytes,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let result = dom
        .install_parsed_html_document(
            before.stamp(),
            report,
            environment.clone(),
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, bytes),
            &AtomicBool::new(false),
        )
        .unwrap();
    let node = result.snapshot().node(element(&result, "a")).unwrap();
    let owner = node.owner_document().unwrap();
    assert_ne!(owner, result.document());
    let hyperlink = node.as_hyperlink().unwrap();
    assert_eq!(hyperlink.parse_document(), Some(owner));
    assert_eq!(hyperlink.url(), Some(&target));
    assert_eq!(hyperlink.parse_observation().unwrap().base(), &blank);
    assert_eq!(hyperlink.parse_observation().unwrap().encoding(), &encoding);
    assert_ne!(
        hyperlink.parse_observation().unwrap().base(),
        environment.url()
    );
    assert_ne!(
        hyperlink.parse_observation().unwrap().encoding(),
        environment.encoding()
    );
    assert!(
        result
            .snapshot()
            .document(owner)
            .unwrap()
            .base_facts()
            .unwrap()
            .same_owner(&inert)
    );
}

#[test]
fn actual_general_clone_uses_its_own_href_observation_and_image_subject_identity() {
    use surgeist_html_semantics::{
        HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseFactsLimits,
        HtmlBaseParseObservation, HtmlBaseParseOutcome, HtmlEncodingUrlParseFacts,
        HtmlEncodingUrlParseObservation, HtmlEncodingUrlParseOutcome, HtmlParseBaseFacts,
    };
    let (mut dom, seed) = setup();
    dom.set_limits(dom.limits().with_base_processing(8, 100_000).unwrap())
        .unwrap();
    let revision = seed.url().revision().clone();
    let url = |value: &str, fragment_delimiter_offset: Option<usize>| {
        HtmlBaseUrlRecord::observed(
            revision.clone(),
            value.into(),
            HtmlBaseUrlScheme::Other,
            false,
            fragment_delimiter_offset,
        )
        .unwrap()
    };
    let old = url("https://example.test/old/", None);
    let new = url("https://example.test/new/", None);
    let old_x = url("https://example.test/old/x", None);
    let new_x = url("https://example.test/new/x", None);
    let environment = DomParsedDocumentEnvironment::headless(
        seed.origin().clone(),
        old.clone(),
        seed.encoding().clone(),
        seed.encoding_label().clone(),
        seed.input_platform(),
    );
    let fact_limits = || HtmlBaseFactsLimits::new(8, 10_000, 100_000);
    let base = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(old.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(environment.encoding().clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "https://example.test/new/".into(),
                old.clone(),
                environment.encoding().clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: new.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        fact_limits(),
    )
    .unwrap();
    let observations = [(old.clone(), old_x.clone()), (new.clone(), new_x.clone())]
        .into_iter()
        .map(|(base, target)| {
            HtmlEncodingUrlParseFacts::new(
                base,
                environment.encoding().clone(),
                revision.clone(),
                vec![HtmlEncodingUrlParseObservation::new(
                    "x".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(target),
                )],
                fact_limits(),
            )
            .unwrap()
        })
        .collect();
    let report = Arc::new(HtmlDocumentParseRequest::offline(HtmlSource::utf8(
        "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><a href=x><img src=x></a><base href='https://example.test/new/'></option></select>",
    ), parse_limits()).with_base_facts(base).unwrap().with_encoding_url_facts(observations).parse().unwrap());
    let result = install(&mut dom, report, environment);
    let mut subjects = Vec::new();
    let mut waits = Vec::new();
    let mut roles = Vec::new();
    for binding in result.bindings() {
        let node = result.snapshot().node(binding.node()).unwrap();
        let Some(hyperlink) = node.as_hyperlink() else {
            continue;
        };
        let parent = result
            .snapshot()
            .node(node.parent().unwrap())
            .unwrap()
            .as_element()
            .unwrap();
        let is_clone = parent.name().local_name() == &DocumentString::from("selectedcontent");
        assert!(is_clone || parent.name().local_name() == &DocumentString::from("option"));
        assert_eq!(
            hyperlink.url(),
            Some(if is_clone { &new_x } else { &old_x })
        );
        assert_eq!(
            hyperlink.parse_observation().unwrap().base(),
            if is_clone { &new } else { &old }
        );
        assert_eq!(hyperlink.parse_document(), Some(result.document()));
        let image = result
            .snapshot()
            .node(node.children()[0])
            .unwrap()
            .as_img()
            .unwrap();
        assert_eq!(image.dimension_attribute_source(), image.id());
        let waiting = image.suspended_update().unwrap().unwrap();
        assert_eq!(waiting.id().target(), image.id());
        assert_eq!(waiting.document(), result.document());
        assert!(!waiting.flags().restart_animation());
        assert!(!waiting.flags().maybe_omit_events());
        subjects.push(image.id());
        waits.push(waiting.id());
        roles.push(is_clone);
    }
    roles.sort_unstable();
    assert_eq!(roles, [false, true]);
    assert_ne!(subjects[0], subjects[1]);
    assert_ne!(waits[0], waits[1]);
}

fn meta_language(snapshot: &DomSnapshot, document: DocumentId) -> Option<&DocumentString> {
    snapshot
        .document(document)
        .unwrap()
        .metadata()
        .meta_state()
        .unwrap()
        .pragma_language()
}

#[test]
fn parsed_meta_history_and_untouched_inert_owner_transfer_without_native_work() {
    use surgeist_html::HtmlReferrerPolicy;
    use surgeist_html_semantics::{
        HtmlAboutBaseUrl, HtmlBaseDocumentFacts, HtmlBaseFactsLimits, HtmlParseBaseFacts,
    };
    let (mut dom, environment) = setup(); // Native Meta work is zero.
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            environment.url().revision().clone(),
            Some(environment.url().clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(environment.encoding().clone()),
        )
        .unwrap(),
        Vec::new(),
        HtmlBaseFactsLimits::new(8, 4096, 100_000),
    )
    .unwrap();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><meta http-equiv=content-language content='EN-private more'><meta http-equiv=content-language content='en,fr'><meta name=referrer content=origin><meta name=referrer content=invalid><meta http-equiv=refresh content=0><meta http-equiv=refresh content='1;url=unobserved'><template><meta http-equiv=content-language content=fr></template><p>Z"),
            parse_limits(),
        )
        .with_base_facts(facts)
        .unwrap()
        .parse()
        .unwrap(),
    );
    let origin = environment.origin().clone();
    let result = install(&mut dom, report, environment);
    let metadata = result
        .snapshot()
        .document(result.document())
        .unwrap()
        .metadata();
    assert_eq!(
        meta_language(result.snapshot(), result.document()),
        Some(&"EN-private".into())
    );
    assert_eq!(
        metadata.policy_container().referrer_policy(),
        Some(HtmlReferrerPolicy::Origin)
    );
    assert!(metadata.meta_state().unwrap().will_declaratively_refresh());
    assert_eq!(metadata.origin(), &origin);
    let template = element(&result, "template");
    let contents = result
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = result
        .snapshot()
        .node(contents)
        .unwrap()
        .owner_document()
        .unwrap();
    assert_ne!(inert, result.document());
    let inert_metadata = result.snapshot().document(inert).unwrap().metadata();
    assert_eq!(meta_language(result.snapshot(), inert), None);
    assert_eq!(
        inert_metadata.policy_container().referrer_policy(),
        Some(HtmlReferrerPolicy::StrictOriginWhenCrossOrigin)
    );
    assert!(
        !inert_metadata
            .meta_state()
            .unwrap()
            .will_declaratively_refresh()
    );
    assert_ne!(inert_metadata.origin(), &origin);
}

#[test]
fn parsed_meta_history_survives_report_release_and_later_native_insertion_updates_only_current() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><meta http-equiv=content-language content=en><p>X",
    ));
    let weak = Arc::downgrade(&report);
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let document = result.document();
    let meta = element(&result, "meta");
    let head = element(&result, "head");
    let old = result.snapshot().clone();
    assert_eq!(meta_language(&old, document), Some(&"en".into()));
    drop(result);
    assert!(weak.upgrade().is_none());
    assert_eq!(meta_language(&dom.snapshot(), document), Some(&"en".into()));
    dom.set_limits(dom.limits().with_meta_processing_work(100_000))
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "fr");
    batch.detach(meta);
    batch.insert_before(meta, head, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(meta_language(&dom.snapshot(), document), Some(&"fr".into()));
    assert_eq!(meta_language(&old, document), Some(&"en".into()));
}

#[test]
fn every_unavailable_meta_domain_refuses_after_independent_history_without_publishing() {
    use surgeist_html_parser::HtmlParseMetaMissingKind as Missing;
    for (literal, expected) in [
        (
            "<meta http-equiv=refresh content='0;url=unobserved'>",
            Missing::RefreshUrl(surgeist_html_parser::HtmlParseHyperlinkUnavailable::Base(
                surgeist_html_semantics::HtmlBaseMissing::DocumentFacts,
            )),
        ),
        (
            "<meta http-equiv=default-style content=chosen>",
            Missing::PreferredStyle,
        ),
        (
            "<meta http-equiv=Content-Security-Policy content=default-src-none>",
            Missing::Csp,
        ),
        ("<meta name=theme-color content=red>", Missing::ThemeColor),
        (
            "<meta name=color-scheme content=dark>",
            Missing::ColorScheme,
        ),
    ] {
        let input = format!(
            "<!doctype html><meta http-equiv=content-language content=en>{literal}<meta http-equiv=content-language content=fr><p>Z"
        );
        let report = parse(HtmlSource::utf8(input));
        let source_document = report.document().unwrap().root().id();
        let (mut dom, environment) = setup();
        let before = dom.snapshot();
        let usage = dom.usage();
        let error = dom
            .install_parsed_html_document(
                before.stamp(),
                report.clone(),
                environment,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        let DomParsedHtmlInstallError::MetaStateUnavailable {
            document,
            subject,
            reason,
        } = error
        else {
            panic!("expected actual missing Meta domain: {error:?}");
        };
        assert_eq!(document, source_document);
        assert_eq!(reason, expected);
        assert!(
            matches!(report.document().unwrap().node(&subject).unwrap().kind(), surgeist_html::HtmlNodeKind::Element(element) if element.name().local_name() == &DocumentString::from("meta"))
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

#[test]
fn parsed_meta_utf16_language_uses_canonical_exact_units_and_one_short_rolls_back() {
    let mut raw: Vec<u16> = "<!doctype html><meta http-equiv=content-language content='"
        .encode_utf16()
        .collect();
    raw.extend([0xd800, 97]);
    raw.extend("'><p>X".encode_utf16());
    let report = parse(HtmlSource::utf16(DocumentString::from_utf16(raw)));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report.clone(), environment.clone());
    assert_eq!(
        meta_language(result.snapshot(), result.document())
            .unwrap()
            .code_units(),
        &[0xd800, 97]
    );
    let exact = dom.usage().string_code_units();
    for allowance in [exact - 1, exact] {
        let (mut target, _) = setup();
        target
            .set_limits(
                DomLimits::new(
                    1024,
                    allowance,
                    1024,
                    4096,
                    4096,
                    32,
                    1_000_000,
                    surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let before = target.snapshot();
        let usage = target.usage();
        let actual = target.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(false),
        );
        if allowance == exact {
            let installed = actual.unwrap();
            assert_eq!(
                meta_language(installed.snapshot(), installed.document())
                    .unwrap()
                    .code_units(),
                &[0xd800, 97]
            );
        } else {
            assert!(matches!(
                actual,
                Err(DomParsedHtmlInstallError::Target(DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    ..
                }))
            ));
            assert_eq!(target.snapshot().stamp(), before.stamp());
            assert_eq!(target.usage(), usage);
            assert!(
                target
                    .changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}

#[test]
fn stale_foreign_cancelled_and_finite_refusals_publish_no_document_or_partial_map() {
    let report = parse(HtmlSource::utf8("<!doctype html><p>finite</p>"));
    let (mut dom, environment) = setup();
    let before = dom.snapshot();
    let usage = dom.usage();
    for chosen in [
        DomParsedHtmlInstallLimits::new(0, 10_000_000, 16_000_000, 100_000_000),
        DomParsedHtmlInstallLimits::new(1024, 0, 16_000_000, 100_000_000),
        DomParsedHtmlInstallLimits::new(1024, 10_000_000, 0, 100_000_000),
        DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 0),
    ] {
        assert!(matches!(
            dom.install_parsed_html_document(
                before.stamp(),
                report.clone(),
                environment.clone(),
                chosen,
                &AtomicBool::new(false)
            ),
            Err(DomParsedHtmlInstallError::ResourceLimit { .. })
        ));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(true)
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    let (other, _) = setup();
    assert!(matches!(
        dom.install_parsed_html_document(
            other.snapshot().stamp(),
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomParsedHtmlInstallError::Target(DomError::ForeignOwner))
    ));
    let mut b = DomBatch::new();
    b.create_document();
    dom.apply(b, &AtomicBool::new(false)).unwrap();
    let newer = dom.snapshot();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report,
            environment,
            limits(),
            &AtomicBool::new(false)
        ),
        Err(DomParsedHtmlInstallError::StaleDestination { .. })
    ));
    assert_eq!(dom.snapshot().stamp(), newer.stamp());
}

fn assert_parsed_dialog_closed_services(view: DomDialog<'_>) {
    assert!(!view.is_modal());
    assert!(!view.has_toggle_task_tracker());
    assert!(!view.has_close_watcher());
    assert_eq!(view.request_close_return_value(), None);
    assert_eq!(view.request_close_source(), None);
    assert!(!view.force_close_watcher_enabled());
}

#[test]
fn parsed_dialog_report_transfers_actual_return_value_independently_of_attribute() {
    // HTML877d interactive chapter1026: returnValue is initialized to the
    // empty string; an unrelated content attribute is not an IDL assignment.
    // Ordinary open addition/insertion (1054-1094) cannot invoke show/close.
    // The actual offline Document is inactive, so setup is not reached.
    let report = parse(HtmlSource::utf8(
        "<!doctype html><dialog open returnvalue=attribute-only></dialog>",
    ));
    let source_root = report.document().unwrap().root().id();
    let (mut dom, environment) = setup();
    let before = dom.snapshot();
    let result = install(&mut dom, report, environment);
    assert_eq!(
        result.node_for(&source_root),
        Some(result.document().node())
    );
    assert_ne!(result.snapshot().stamp(), before.stamp());
    let id = element(&result, "dialog");
    let node = result.snapshot().node(id).unwrap();
    assert_eq!(node.owner_document(), Some(result.document()));
    assert!(node.is_connected());
    let element = node.as_element().unwrap();
    assert_eq!(
        element.attribute(&"open".into()),
        Some(&DocumentString::from(""))
    );
    assert_eq!(
        element.attribute(&"returnvalue".into()),
        Some(&DocumentString::from("attribute-only")),
    );
    let dialog = node.as_dialog().expect("actual parsed Dialog state");
    assert_eq!(dialog.return_value().code_units(), []);
    assert_parsed_dialog_closed_services(dialog);
    assert!(
        !element
            .common_state()
            .html()
            .unwrap()
            .has_previously_focused_element()
    );
}

#[test]
fn parsed_dialog_free_document_retains_history_for_later_native_open_cleanup() {
    // Foundation4879: each actual new Document starts with an empty open
    // dialogs list. In this closed original parse no operation adds a member.
    // Interactive1054-1094 and1294-1326: insertion stops at inactive, whereas
    // open removal/removal must perform cleanup before any inactivity guard.
    // The final absence of Dialog is not the reason history is known.
    let report = parse(HtmlSource::utf8("<!doctype html><p>source-only</p>"));
    let (mut dom, environment) = setup();
    let installed = install(&mut dom, report, environment);
    let document = installed.document();
    let body = element(&installed, "body");
    let historical = installed.snapshot().clone();

    let mut batch = DomBatch::new();
    let ticket = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.set_attribute(&ticket, "open", "");
    batch.insert_before(&ticket, body, None);
    let created = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let id = created.created(&ticket).unwrap().unwrap();
    let opened = dom.snapshot();
    let node = opened.node(id).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert!(node.is_connected());
    assert_eq!(node.as_dialog().unwrap().return_value().code_units(), []);
    assert_parsed_dialog_closed_services(node.as_dialog().unwrap());
    assert!(historical.node(id).is_err());

    // Cleanup needs this actual Document's retained interaction history,
    // independently of the later native Dialog's own creation state.
    let mut batch = DomBatch::new();
    batch.remove_attribute(id, "open");
    dom.apply(batch, &AtomicBool::new(false))
        .expect("source-established Document list history supports open cleanup");
    let cleaned = dom.snapshot();
    let node = cleaned.node(id).unwrap();
    assert_eq!(node.as_element().unwrap().attribute(&"open".into()), None);
    assert_parsed_dialog_closed_services(node.as_dialog().unwrap());
    assert!(
        opened
            .node(id)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );

    // Exercise removed-open cleanup as a separate real ordinary operation.
    let mut batch = DomBatch::new();
    batch.set_attribute(id, "open", "");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(id);
    dom.apply(batch, &AtomicBool::new(false))
        .expect("source-established Document list history supports removed-open cleanup");
    let detached = dom.snapshot();
    let node = detached.node(id).unwrap();
    assert!(!node.is_connected());
    assert_eq!(node.owner_document(), Some(document));
    assert!(
        node.as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    assert_eq!(node.as_dialog().unwrap().return_value().code_units(), []);
    assert_parsed_dialog_closed_services(node.as_dialog().unwrap());
}

#[test]
fn parsed_template_dialog_adoption_keeps_current_value_and_original_inert_history() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><dialog id=outer open></dialog><template><dialog id=inner open></dialog></template>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report, environment);
    let initial = result.snapshot().clone();
    let template = element(&result, "template");
    let contents = initial
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inner = initial.node(contents).unwrap().children()[0];
    let inert = initial.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, result.document());
    assert_eq!(initial.node(inner).unwrap().owner_document(), Some(inert));
    assert_eq!(initial.node(inner).unwrap().parent(), Some(contents));
    assert_eq!(
        initial
            .node(inner)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"id".into()),
        Some(&"inner".into())
    );
    assert_eq!(
        initial
            .node(inner)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value()
            .code_units(),
        []
    );
    assert_parsed_dialog_closed_services(initial.node(inner).unwrap().as_dialog().unwrap());
    let source = result
        .bindings()
        .iter()
        .find(|b| b.node() == inner)
        .unwrap()
        .source()
        .clone();
    let exact = DocumentString::from_utf16(vec![0xd800, 13, 10, 0xdc00]);
    let mut batch = DomBatch::new();
    batch.set_dialog_return_value(inner, exact.clone());
    // Adoption removes an open Dialog from the actual inert owner first.
    batch.adopt(inner, result.document());
    batch.insert_before(inner, element(&result, "body"), None);
    batch.remove_attribute(inner, "open");
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert_eq!(
        current.node(inner).unwrap().owner_document(),
        Some(result.document())
    );
    assert_eq!(
        current
            .node(inner)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &exact
    );
    assert_eq!(
        current
            .node(inner)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    assert!(current.node(contents).unwrap().children().is_empty());
    assert_parsed_dialog_closed_services(current.node(inner).unwrap().as_dialog().unwrap());
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::NodeDocument(inner))
    );
    assert_eq!(result.node_for(&source), Some(inner));
    assert_eq!(initial.node(inner).unwrap().owner_document(), Some(inert));
    assert_eq!(
        initial
            .node(inner)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value()
            .code_units(),
        []
    );
    assert!(
        initial
            .node(inner)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    assert_eq!(initial.node(contents).unwrap().children(), &[inner]);
}

#[test]
fn captured_fragment_dialogs_keep_target_state_and_auxiliary_identity_through_later_hooks() {
    let (mut dom, _, document, host) = fragment_fixture("div");
    let mut batch = DomBatch::new();
    let existing = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.set_dialog_return_value(&existing, "pre-existing");
    batch.set_attribute(&existing, "open", "");
    batch.insert_before(&existing, host, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let existing = commit.created(&existing).unwrap().unwrap();
    let before = dom.snapshot();
    let capture = before
        .html_capture(
            document,
            None,
            surgeist_html_semantics::HtmlCaptureLimits::new(
                1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216,
            ),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    // Style supplies a real retained parser-Document association. It stays
    // detached: later ordinary operations below move only the Dialog subtree.
    let report = fragment_report(
        &capture,
        host,
        HtmlSource::utf8(
            "<dialog open id=root><dialog open id=nested></dialog></dialog><style>a{}</style>",
        ),
        false,
    );
    let parser_document = report.fragment_parser_document().unwrap().document().id();
    let source_root = report.fragment().unwrap().root().id();
    let installed = dom
        .install_parsed_html_fragment(report, &capture, limits(), &AtomicBool::new(false))
        .unwrap();
    let initial = installed.snapshot().clone();
    assert_eq!(installed.document(), document);
    assert_eq!(installed.context_basis(), Some(capture.basis()));
    assert_eq!(installed.node_for(&source_root), Some(installed.root()));
    assert_eq!(
        installed.captured_node_for(&capture.subject(host).unwrap()),
        Some(host)
    );
    assert_eq!(
        installed.captured_node_for(&capture.subject(document.node()).unwrap()),
        Some(document.node())
    );
    assert_eq!(initial.node(installed.root()).unwrap().parent(), None);
    assert_eq!(initial.node(host).unwrap().children(), &[existing]);
    let root = initial.node(installed.root()).unwrap().children()[0];
    let nested = initial.node(root).unwrap().children()[0];
    let auxiliary = installed
        .node_for(&parser_document)
        .expect("actual retained Style parser Document");
    assert_ne!(auxiliary, document.node());
    assert!(initial.node(auxiliary).unwrap().as_document().is_some());
    for node in [root, nested] {
        assert_eq!(initial.node(node).unwrap().owner_document(), Some(document));
        assert_eq!(
            initial
                .node(node)
                .unwrap()
                .as_dialog()
                .unwrap()
                .return_value()
                .code_units(),
            []
        );
        assert_parsed_dialog_closed_services(initial.node(node).unwrap().as_dialog().unwrap());
    }
    let mut batch = DomBatch::new();
    batch.insert_before(root, host, None);
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let connected = dom.snapshot();
    assert!(connected.node(root).unwrap().is_connected());
    assert!(connected.node(nested).unwrap().is_connected());
    let mut batch = DomBatch::new();
    batch.detach(root);
    batch.remove_attribute(nested, "open");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let current = dom.snapshot();
    assert!(!current.node(root).unwrap().is_connected());
    assert!(!current.node(nested).unwrap().is_connected());
    assert_eq!(current.node(root).unwrap().owner_document(), Some(document));
    assert_eq!(
        current
            .node(nested)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    assert_eq!(current.node(host).unwrap().children(), &[existing]);
    assert_eq!(
        current
            .node(existing)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &"pre-existing".into()
    );
    assert!(
        current
            .node(existing)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    assert_eq!(initial.node(root).unwrap().parent(), Some(installed.root()));
    assert_eq!(initial.node(nested).unwrap().parent(), Some(root));
    assert!(
        initial
            .node(nested)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
    assert_eq!(
        before
            .node(existing)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &"pre-existing".into()
    );
}

#[test]
fn actual_selectedcontent_dialog_clone_has_distinct_original_metadata_and_current_identity() {
    use surgeist_html_parser::{HtmlParseDialogDisposition, HtmlParseMetadataStep};
    let report = parse(HtmlSource::utf8(
        "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected><dialog open data-copy=literal></dialog></option></select>",
    ));
    let (mut dom, environment) = setup();
    let result = install(&mut dom, report.clone(), environment);
    let mut pair = Vec::new();
    for binding in result.bindings() {
        let node = result.snapshot().node(binding.node()).unwrap();
        let Some(dialog) = node.as_dialog() else {
            continue;
        };
        let Some(parent) = node
            .parent()
            .and_then(|p| result.snapshot().node(p).unwrap().as_element())
        else {
            continue;
        };
        let local = parent.name().local_name();
        if local != &DocumentString::from("option")
            && local != &DocumentString::from("selectedcontent")
        {
            continue;
        }
        assert_eq!(dialog.return_value().code_units(), []);
        assert_parsed_dialog_closed_services(dialog);
        assert_eq!(
            node.as_element().unwrap().attribute(&"data-copy".into()),
            Some(&"literal".into())
        );
        assert!(
            node.as_element()
                .unwrap()
                .attribute(&"open".into())
                .is_some()
        );
        pair.push((local.clone(), binding.source().clone(), node.id()));
    }
    assert_eq!(
        pair.len(),
        2,
        "actual original and selectedcontent clone must both exist"
    );
    let original = pair
        .iter()
        .find(|p| p.0 == DocumentString::from("option"))
        .unwrap();
    let copy = pair
        .iter()
        .find(|p| p.0 == DocumentString::from("selectedcontent"))
        .unwrap();
    assert_ne!(original.1, copy.1);
    assert_ne!(original.2, copy.2);
    for (_, source, canonical) in &pair {
        assert_eq!(result.node_for(source), Some(*canonical));
        let mut metadata = report.metadata();
        let mut observed = false;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(row) if row.node().id() == *source => {
                    if let Some(state) = row.dialog() {
                        assert!(matches!(
                            state.disposition(),
                            HtmlParseDialogDisposition::Closed
                        ));
                        assert_eq!(state.return_value().code_units(), []);
                        observed = true;
                    }
                }
                HtmlParseMetadataStep::Record(_) | HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert!(
            observed,
            "each actual clone identity needs its own current Dialog row"
        );
    }
    let mut batch = DomBatch::new();
    batch.set_dialog_return_value(original.2, "later original only");
    dom.apply(batch, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(original.2)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value(),
        &"later original only".into()
    );
    assert_eq!(
        dom.snapshot()
            .node(copy.2)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value()
            .code_units(),
        []
    );
    assert_eq!(
        result
            .snapshot()
            .node(original.2)
            .unwrap()
            .as_dialog()
            .unwrap()
            .return_value()
            .code_units(),
        []
    );
}

#[test]
fn dialog_bearing_report_refusal_and_cancellation_leave_maps_unpublished_then_recover() {
    let report = parse(HtmlSource::utf8(
        "<!doctype html><dialog open><span>bounded</span></dialog>",
    ));
    let (mut dom, environment) = setup();
    let control = install(&mut dom, report.clone(), environment.clone());
    // The high-budget real flow establishes the actual mapped-node admission
    // requirement; this is setup, not an independently predicted parser tariff.
    let required = control.bindings().len();
    assert!(required > 1);
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            limits(),
            &AtomicBool::new(true),
        ),
        Err(DomParsedHtmlInstallError::Cancelled)
    ));
    assert!(matches!(
        dom.install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment.clone(),
            DomParsedHtmlInstallLimits::new(required - 1, 10_000_000, 16_000_000, 100_000_000),
            &AtomicBool::new(false),
        ),
        Err(DomParsedHtmlInstallError::ResourceLimit {
            resource: DomParsedHtmlInstallResource::MappedNodes,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let recovered = dom
        .install_parsed_html_document(
            before.stamp(),
            report,
            environment,
            DomParsedHtmlInstallLimits::new(required, 10_000_000, 16_000_000, 100_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(recovered.bindings().len(), required);
    assert_ne!(recovered.root(), control.root());
    assert_eq!(recovered.source_root(), control.source_root());
    assert_eq!(
        control.node_for(control.source_root()),
        Some(control.root())
    );
    assert_eq!(
        recovered.node_for(recovered.source_root()),
        Some(recovered.root())
    );
    assert_parsed_dialog_closed_services(
        recovered
            .snapshot()
            .node(element(&recovered, "dialog"))
            .unwrap()
            .as_dialog()
            .unwrap(),
    );
}

#[test]
fn same_store_input_adoption_preserves_selection_then_uses_destination_platform_for_effects() {
    use surgeist_html::{
        HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as NoneDirection,
        HtmlInputSelectionDirection as Direction,
    };
    let cancel = AtomicBool::new(false);
    let (mut dom, seed) = setup();
    let destination_platform =
        DomNativeInputPlatform::new(Empty::Unsupported, NoneDirection::Unsupported);
    let destination_environment = DomParsedDocumentEnvironment::headless(
        seed.origin().clone(),
        seed.url().clone(),
        seed.encoding().clone(),
        seed.encoding_label().clone(),
        destination_platform,
    );
    let destination = install(
        &mut dom,
        parse(HtmlSource::utf8("<!doctype html><div></div>")),
        destination_environment,
    )
    .document();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let input = batch.create_element_ns(&source, Namespace::html(), "input");
    batch.set_attribute(&input, "value", "xy");
    batch.set_input_value(&input, "abcde");
    let commit = dom.apply(batch, &cancel).unwrap();
    let source = commit.document(&source).unwrap();
    let input = commit.created(&input).unwrap().unwrap();
    let before = dom.snapshot();
    assert_eq!(
        before
            .document(source)
            .unwrap()
            .metadata()
            .document_environment()
            .input_platform(),
        DomNativeInputPlatform::headless()
    );
    assert_eq!(
        before
            .document(destination)
            .unwrap()
            .metadata()
            .document_environment()
            .input_platform(),
        destination_platform
    );
    let selection = before
        .node(input)
        .unwrap()
        .as_input()
        .unwrap()
        .state()
        .selection()
        .unwrap();
    assert_eq!(selection.cursor_position(), Some(5));
    assert_eq!(selection.direction(), Direction::None);

    let mut batch = DomBatch::new();
    batch.adopt(input, destination);
    dom.apply(batch, &cancel).unwrap();
    let adopted = dom.snapshot();
    assert_eq!(
        adopted.node(input).unwrap().owner_document(),
        Some(destination)
    );
    assert_eq!(
        adopted
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection(),
        Some(selection)
    );
    assert_eq!(before.node(input).unwrap().owner_document(), Some(source));
    assert_eq!(
        before
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection(),
        Some(selection)
    );

    // A genuine value setter changes selection using the then-current target
    // Document's independently established none-direction policy.
    let mut batch = DomBatch::new();
    batch.set_input_value(input, "z\r\nuvw");
    dom.apply(batch, &cancel).unwrap();
    let changed = dom.snapshot();
    let view = changed.node(input).unwrap().as_input().unwrap();
    assert_eq!(view.value(), "zuvw".encode_utf16().collect::<Vec<_>>());
    let current = view.state().selection().unwrap();
    assert_eq!(current.cursor_position(), Some(4));
    assert_eq!(current.direction(), Direction::Forward);
    let mut batch = DomBatch::new();
    batch.reset_input(input);
    dom.apply(batch, &cancel).unwrap();
    let reset = dom.snapshot();
    let view = reset.node(input).unwrap().as_input().unwrap();
    assert_eq!(view.value(), "xy".encode_utf16().collect::<Vec<_>>());
    assert!(!view.state().flags().dirty_value());
    assert_eq!(view.state().selection().unwrap().cursor_position(), Some(2));
    assert_eq!(
        view.state().selection().unwrap().direction(),
        Direction::Forward
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "checkbox");
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        dom.snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection(),
        None
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(input, "type", "text");
    dom.apply(batch, &cancel).unwrap();
    let final_snapshot = dom.snapshot();
    let final_selection = final_snapshot
        .node(input)
        .unwrap()
        .as_input()
        .unwrap()
        .state()
        .selection()
        .unwrap();
    assert_eq!(final_selection.cursor_position(), Some(0));
    assert_eq!(final_selection.direction(), Direction::Forward);
    assert_eq!(
        adopted
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection(),
        Some(selection)
    );
    assert_eq!(
        changed
            .node(input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection(),
        Some(current)
    );
}

#[test]
fn parsed_input_image_history_transfers_never_entered_absent_empty_and_nonempty_sources() {
    use surgeist_html::{HtmlInputImagesDisabled, HtmlInputType};
    use surgeist_html_parser::{HtmlInputImageSource, HtmlParseMetadataStep};
    for (markup, expected_type, expected_source) in [
        ("<input src='unobserved'>", HtmlInputType::Text, None),
        ("<input type=image>", HtmlInputType::Image, Some(None)),
        (
            "<input type=image src=''>",
            HtmlInputType::Image,
            Some(Some("")),
        ),
        (
            "<input type=image src='A&amp;B'>",
            HtmlInputType::Image,
            Some(Some("A&B")),
        ),
    ] {
        let report = parse(HtmlSource::utf8(markup));
        let mut metadata = report.metadata();
        let mut produced = None;
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(row) => {
                    if let Some(flags) = row.input_flags() {
                        assert!(produced.is_none(), "one actual Input source row");
                        assert_eq!(flags.input_type(), expected_type);
                        let observed = match row.input_image_source() {
                            None => None,
                            Some(HtmlInputImageSource::Absent) => Some(None),
                            Some(HtmlInputImageSource::Present(raw)) => Some(Some(raw.clone())),
                        };
                        let literal =
                            expected_source.map(|source| source.map(DocumentString::from));
                        assert_eq!(observed, literal);
                        let coordinate = row.input_image_coordinate();
                        assert_eq!(coordinate.is_some(), expected_type == HtmlInputType::Image);
                        if let Some(coordinate) = coordinate {
                            assert_eq!((coordinate.x(), coordinate.y()), (0, 0));
                        }
                        produced = Some((row.node().id(), observed, coordinate, *flags));
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        let (source, observed, coordinate, flags) = produced.unwrap();
        let (mut dom, environment) = setup();
        let installed = install(&mut dom, report.clone(), environment);
        let input = installed.node_for(&source).unwrap();
        let original = installed
            .snapshot()
            .node(input)
            .unwrap()
            .as_input()
            .unwrap();
        assert_eq!(
            installed
                .snapshot()
                .document(installed.document())
                .unwrap()
                .metadata()
                .document_environment()
                .input_images(),
            HtmlInputImagesDisabled::ExplicitlyDisabled
        );
        assert_eq!(original.state().flags(), flags);
        assert_eq!(
            original
                .state()
                .image_history()
                .observed_source()
                .map(|source| source.cloned()),
            observed
        );
        let actual_coordinate = original.state().image_history().coordinate();
        assert_eq!((actual_coordinate.x(), actual_coordinate.y()), (0, 0));
        if let Some(coordinate) = coordinate {
            assert_eq!(actual_coordinate, coordinate);
        }
        // A later native transition cannot rewrite the transferred immutable
        // source history or the original installed snapshot. No image resource,
        // loading state or event is supplied by these closed source guards.
        let mut batch = DomBatch::new();
        batch.set_attribute(input, "type", "image");
        batch.set_attribute(input, "src", "later-native");
        dom.apply(batch, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            original
                .state()
                .image_history()
                .observed_source()
                .map(|source| source.cloned()),
            observed
        );
        let current = dom.snapshot();
        assert_eq!(
            current
                .node(input)
                .unwrap()
                .as_input()
                .unwrap()
                .state()
                .image_history()
                .observed_source(),
            Some(Some(&DocumentString::from("later-native")))
        );
        assert_eq!(installed.node_for(&source), Some(input));
    }
}

#[test]
fn parsed_values_reuse_native_backing_without_rewriting_report_or_parser_state() {
    let cancel = AtomicBool::new(false);
    let (mut dom, environment) = setup();
    let mut seed = DomBatch::new();
    let native_document = seed.create_document();
    let native_input = seed.create_element_ns(&native_document, Namespace::html(), "input");
    seed.set_attribute(&native_input, "type", "text");
    seed.set_attribute(&native_input, "value", "initial");
    seed.set_attribute(&native_input, "class", "cell");
    let mut text = Vec::with_capacity(64);
    text.extend("repeat".encode_utf16());
    let native_text = seed.create_text(&native_document, DocumentString::from_utf16(text));
    let seeded = dom.apply(seed, &cancel).unwrap();
    let native_input = seeded.created(&native_input).unwrap().unwrap();
    let native_text = seeded.created(&native_text).unwrap().unwrap();
    let native = dom.snapshot();
    let native_bank = native
        .node(native_input)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    assert_eq!(native_bank.len(), 3);
    let native_data = native.node(native_text).unwrap().data().unwrap();
    assert_eq!(native_data.code_units(), [114, 101, 112, 101, 97, 116]);
    assert_eq!(native_data.shared_code_units().capacity(), 64);
    assert!(
        !native
            .node(native_input)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .parser_inserted()
    );

    let markup = concat!(
        "<!doctype html><form>",
        "<input type=text value=initial class=cell>",
        "<input type=text value=initial class=cell>",
        "<p class=cell>repeat</p><p class=cell>repeat</p></form>"
    );
    let mut source = Vec::with_capacity(8192);
    source.extend(markup.encode_utf16());
    let report = parse(HtmlSource::utf16(DocumentString::from_utf16(source)));
    let source_identity = report.source().identity().clone();
    let source_backing = Arc::as_ptr(report.source().as_utf16().unwrap().shared_code_units());
    let source_capacity = report
        .source()
        .as_utf16()
        .unwrap()
        .shared_code_units()
        .capacity();
    assert_eq!(source_capacity, 8192);
    let receipt = report.retained_construction_bytes();
    assert!(receipt > 0);
    let mut pending = vec![report.document().unwrap().root()];
    let mut source_inputs = Vec::new();
    let mut source_texts = Vec::new();
    while let Some(node) = pending.pop() {
        match node.kind() {
            surgeist_html::HtmlNodeKind::Element(element)
                if element.name().local_name() == &DocumentString::from("input") =>
            {
                let bank = element.attributes().shared_attributes();
                assert_eq!(bank.len(), 3);
                let fields: Vec<_> = bank
                    .iter()
                    .map(|attribute| {
                        (
                            Arc::as_ptr(attribute.value().shared_code_units()),
                            attribute.value().shared_code_units().capacity(),
                        )
                    })
                    .collect();
                source_inputs.push((
                    node.id(),
                    Arc::as_ptr(bank),
                    bank.capacity(),
                    bank.as_slice().as_ptr() as usize,
                    fields,
                ));
            }
            surgeist_html::HtmlNodeKind::Text(text) => {
                assert_eq!(text.code_units(), [114, 101, 112, 101, 97, 116]);
                source_texts.push((
                    node.id(),
                    Arc::as_ptr(text.shared_code_units()),
                    text.shared_code_units().capacity(),
                ));
            }
            _ => {}
        }
        pending.extend(
            node.children()
                .rev()
                .map(|id| report.document().unwrap().node(&id).unwrap()),
        );
    }
    assert_eq!(source_inputs.len(), 2);
    assert_eq!(source_texts.len(), 2);
    let result = install(&mut dom, report.clone(), environment);
    let inputs: Vec<_> = source_inputs
        .iter()
        .map(|(id, ..)| result.node_for(id).unwrap())
        .collect();
    let texts: Vec<_> = source_texts
        .iter()
        .map(|(id, ..)| result.node_for(id).unwrap())
        .collect();
    let form = element(&result, "form");
    assert_ne!(inputs[0], inputs[1]);
    assert_ne!(texts[0], texts[1]);
    for (index, input) in inputs.iter().enumerate() {
        let node = result.snapshot().node(*input).unwrap();
        let attributes = node.as_element().unwrap().attributes();
        assert_eq!(
            attributes.as_ptr(),
            native_bank.as_ptr(),
            "canonical bank reuse is independent of the report's HTML bank"
        );
        assert_ne!(
            attributes.as_ptr() as usize,
            source_inputs[index].3,
            "the retained HTML and canonical DOM arrays are distinct live allocations"
        );
        for (attribute, (name, value)) in
            attributes
                .iter()
                .zip([("type", "text"), ("value", "initial"), ("class", "cell")])
        {
            assert!(attribute.name().namespace_uri().is_none());
            assert!(attribute.name().prefix().is_none());
            assert!(
                attribute
                    .name()
                    .local_name()
                    .code_units()
                    .iter()
                    .copied()
                    .eq(name.encode_utf16())
            );
            assert!(
                attribute
                    .value()
                    .code_units()
                    .iter()
                    .copied()
                    .eq(value.encode_utf16())
            );
        }
        let input = node.as_input().unwrap();
        assert!(input.state().parser_inserted());
        assert!(!input.state().flags().dirty_value());
        assert_eq!(input.form_owner().unwrap(), Some(form));
        assert_eq!(input.value(), "initial".encode_utf16().collect::<Vec<_>>());
    }
    for text in &texts {
        let data = result.snapshot().node(*text).unwrap().data().unwrap();
        assert!(Arc::ptr_eq(
            data.shared_code_units(),
            native_data.shared_code_units()
        ));
        assert_eq!(data.code_units(), [114, 101, 112, 101, 97, 116]);
        assert_eq!(data.shared_code_units().capacity(), 64);
    }
    let mut edit = DomBatch::new();
    edit.set_attribute(inputs[0], "class", "edited");
    edit.set_input_value(inputs[0], "current");
    edit.replace_data(texts[0], 0, 6, "changed");
    dom.apply(edit, &cancel).unwrap();
    let current = dom.snapshot();
    assert_eq!(
        current.node(inputs[0]).unwrap().as_input().unwrap().value(),
        "current".encode_utf16().collect::<Vec<_>>()
    );
    assert!(
        current
            .node(inputs[0])
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    assert_eq!(
        current.node(texts[0]).unwrap().data().unwrap(),
        &DocumentString::from("changed")
    );
    for snapshot in [&native, result.snapshot(), &current] {
        assert_eq!(
            snapshot
                .node(native_input)
                .unwrap()
                .as_input()
                .unwrap()
                .value(),
            "initial".encode_utf16().collect::<Vec<_>>()
        );
        assert!(
            !snapshot
                .node(native_input)
                .unwrap()
                .as_input()
                .unwrap()
                .state()
                .parser_inserted()
        );
        assert_eq!(
            snapshot.node(native_text).unwrap().data().unwrap(),
            &DocumentString::from("repeat")
        );
    }
    for (snapshot, input, text) in [
        (result.snapshot(), inputs[0], texts[0]),
        (result.snapshot(), inputs[1], texts[1]),
        (&current, inputs[1], texts[1]),
    ] {
        let element = snapshot.node(input).unwrap().as_element().unwrap();
        assert_eq!(
            element.attribute(&"class".into()),
            Some(&DocumentString::from("cell"))
        );
        assert_eq!(
            snapshot.node(input).unwrap().as_input().unwrap().value(),
            "initial".encode_utf16().collect::<Vec<_>>()
        );
        let data = snapshot.node(text).unwrap().data().unwrap();
        assert!(Arc::ptr_eq(
            data.shared_code_units(),
            native_data.shared_code_units()
        ));
        assert_eq!(data.code_units(), [114, 101, 112, 101, 97, 116]);
    }
    assert!(std::ptr::eq(result.report(), Arc::as_ref(&report)));
    assert_eq!(report.source().identity(), &source_identity);
    assert_eq!(report.retained_construction_bytes(), receipt);
    assert_eq!(
        Arc::as_ptr(report.source().as_utf16().unwrap().shared_code_units()),
        source_backing
    );
    assert_eq!(
        report
            .source()
            .as_utf16()
            .unwrap()
            .shared_code_units()
            .capacity(),
        source_capacity
    );
    assert!(
        report
            .source()
            .as_utf16()
            .unwrap()
            .code_units()
            .iter()
            .copied()
            .eq(markup.encode_utf16())
    );
    for (id, pointer, capacity, buffer, fields) in source_inputs {
        let surgeist_html::HtmlNodeKind::Element(element) =
            report.document().unwrap().node(&id).unwrap().kind()
        else {
            panic!("same report Input");
        };
        let bank = element.attributes().shared_attributes();
        assert_eq!(Arc::as_ptr(bank), pointer);
        assert_eq!(bank.capacity(), capacity);
        assert_eq!(bank.as_slice().as_ptr() as usize, buffer);
        for (attribute, (pointer, capacity)) in bank.iter().zip(fields) {
            assert_eq!(Arc::as_ptr(attribute.value().shared_code_units()), pointer);
            assert_eq!(attribute.value().shared_code_units().capacity(), capacity);
        }
    }
    for (id, pointer, capacity) in source_texts {
        let surgeist_html::HtmlNodeKind::Text(text) =
            report.document().unwrap().node(&id).unwrap().kind()
        else {
            panic!("same report Text");
        };
        assert_eq!(text.code_units(), [114, 101, 112, 101, 97, 116]);
        assert_eq!(Arc::as_ptr(text.shared_code_units()), pointer);
        assert_eq!(text.shared_code_units().capacity(), capacity);
    }
}
