use surgeist_html::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseFailureReason, HtmlParseLimits,
    HtmlParseMetadataStep, HtmlParseRequestError, HtmlParseResource, HtmlParseStop, HtmlSource,
};
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsError,
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseParseObservation,
    HtmlBaseParseOutcome, HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
    HtmlCaptureBuilder, HtmlCaptureLimits, HtmlCaptureRegistryInput, HtmlCaptureTemplateCacheInput,
    HtmlCapturedBaseFacts, HtmlCapturedIsValue, HtmlDocumentClass, HtmlFragmentContext,
    HtmlParseBaseFacts, HtmlScriptingStatus, HtmlTemplateDocumentRole,
};

fn limits() -> HtmlParseLimits {
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
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn url(
    revision: &HtmlBaseFactsRevision,
    text: &str,
    scheme: HtmlBaseUrlScheme,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(revision.clone(), text.into(), scheme, false).unwrap()
}
fn facts(
    raw: &str,
    outcome: impl FnOnce(&HtmlBaseFactsRevision) -> HtmlBaseParseOutcome,
) -> HtmlParseBaseFacts {
    let revision = revision();
    let fallback = url(
        &revision,
        "https://example.test/original/",
        HtmlBaseUrlScheme::Other,
    );
    let encoding = HtmlBaseEncoding::fresh();
    let observation = HtmlBaseParseObservation::new(
        raw.into(),
        fallback.clone(),
        encoding.clone(),
        revision.clone(),
        outcome(&revision),
    )
    .unwrap();
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision,
            Some(fallback),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        vec![observation],
        HtmlBaseFactsLimits::new(10, 1000, 10000),
    )
    .unwrap()
}
fn source(raw: &str, utf16: bool) -> HtmlSource {
    if utf16 {
        HtmlSource::utf16(raw.into())
    } else {
        HtmlSource::utf8(raw)
    }
}

#[test]
fn ordinary_request_rejects_srcdoc_facts_and_failure_keeps_original_facts() {
    let revision = revision();
    let about = url(&revision, "https://creator.test/", HtmlBaseUrlScheme::Other);
    let srcdoc = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision,
            None,
            HtmlAboutBaseUrl::KnownRecord(about),
            true,
            None,
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(0, 100, 100),
    )
    .unwrap();
    assert_eq!(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits())
            .with_base_facts(srcdoc)
            .unwrap_err(),
        HtmlParseRequestError::Base(HtmlBaseFactsError::ContradictoryObservation)
    );
    let facts = facts("X", |_| HtmlBaseParseOutcome::Failure);
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<base href=X>"), limits())
        .with_base_facts(facts.clone())
        .unwrap();
    request.cancellation().cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(
        failure.document_base_facts().unwrap().document().revision(),
        facts.document().revision()
    );
    assert!(failure.captured_base_facts().is_none());
    assert_eq!(
        failure
            .document_base_facts()
            .unwrap()
            .document()
            .document_url()
            .unwrap(),
        facts.document().document_url().unwrap()
    );
}

#[test]
fn fragment_base_is_not_an_attachment_to_the_captured_document_and_keeps_actual_facts() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 1000, 100, 1000),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture = builder.finish().unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
    let revision = revision();
    let captured = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![],
        vec![],
        HtmlBaseFactsLimits::new(0, 0, 0),
    )
    .unwrap();
    let request = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<base href=unknown><p>X"),
        context.clone(),
        limits(),
    )
    .unwrap()
    .with_base_facts(captured)
    .unwrap();
    let cancellation = request.cancellation().clone();
    let report = request.parse().unwrap();
    let usage = report.usage().clone();
    cancellation.cancel();
    assert_eq!(report.captured_base_facts().unwrap().revision(), &revision);
    assert_eq!(
        report.captured_base_facts().unwrap().capture(),
        capture.identity()
    );
    assert!(report.document_base_facts().is_none());
    let fragment = report.fragment().unwrap();
    let children: Vec<_> = fragment.root().children().collect();
    assert_eq!(children.len(), 2);
    assert!(
        matches!(fragment.node(&children[0]).unwrap().kind(), HtmlNodeKind::Element(e) if e.name().local_name().code_units() == [98,97,115,101])
    );
    assert_eq!(report.usage(), &usage);
    let captured = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision,
        vec![],
        vec![],
        HtmlBaseFactsLimits::new(0, 0, 0),
    )
    .unwrap();
    let request =
        HtmlFragmentParseRequest::new(HtmlSource::utf8("<base href=unknown>"), context, limits())
            .unwrap()
            .with_base_facts(captured)
            .unwrap();
    request.cancellation().cancel();
    let failure = request.parse().unwrap_err();
    assert_eq!(
        failure.captured_base_facts().unwrap().capture(),
        capture.identity()
    );
    assert!(failure.document_base_facts().is_none());
}

#[test]
fn first_href_binds_decoded_value_and_actual_node_before_later_bases() {
    // First target and first href are different actual elements. Entity
    // decoding, not a prospective node/source ordinal, selects the URL input.
    let raw = "<!doctype html><base target=T><base href='A&amp;B'><base href=Z target=U><p>X";
    for utf16 in [false, true] {
        let facts = facts("A&B", |revision| HtmlBaseParseOutcome::Parsed {
            record: url(
                revision,
                "https://example.test/original/A&B",
                HtmlBaseUrlScheme::Other,
            ),
            base_allowed: HtmlBaseAllowed::Allowed,
        });
        let request = HtmlDocumentParseRequest::offline(source(raw, utf16), limits())
            .with_base_facts(facts)
            .unwrap();
        let original = request.source().identity().clone();
        let cancel = request.cancellation().clone();
        let report = request.parse().unwrap();
        let usage = report.usage().clone();
        cancel.cancel();
        assert_eq!(report.source().identity(), &original);
        assert!(report.document_base_facts().is_some());
        let document = report.document().unwrap();
        let html = document
            .node(&document.root().children().nth(1).unwrap())
            .unwrap();
        let head = document.node(&html.children().next().unwrap()).unwrap();
        let bases: Vec<_> = head.children().collect();
        assert_eq!(bases.len(), 3);
        assert_ne!(bases[0], bases[1]);
        assert_ne!(bases[1], bases[2]);
        let HtmlNodeKind::Element(element) = document.node(&bases[1]).unwrap().kind() else {
            panic!("base")
        };
        assert_eq!(element.name().local_name(), &DocumentString::from("base"));
        assert_eq!(
            element.attributes().as_slice()[0].value().code_units(),
            &[65, 38, 66]
        );
        let mut seen = [false; 3];
        let mut document_binding = false;
        let mut metadata = report.metadata();
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(first) = record.document_first_base() {
                        assert_eq!(first.unwrap().id(), bases[1]);
                        document_binding = true;
                    }
                    if let Some(frozen) = record.base_frozen() {
                        let position = bases
                            .iter()
                            .position(|id| *id == record.node().id())
                            .unwrap();
                        seen[position] = true;
                        if position == 1 {
                            let frozen = frozen.unwrap();
                            assert!(frozen.responded());
                            assert_eq!(frozen.document().id(), document.root().id());
                            assert_eq!(
                                frozen.value().serialization(),
                                &DocumentString::from("https://example.test/original/A&B")
                            );
                        } else {
                            assert!(frozen.is_none());
                        }
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(seen, [true; 3]);
        assert!(document_binding);
        assert_eq!(report.usage(), &usage);
    }
}

#[test]
fn failure_blocked_and_forbidden_schemes_freeze_fallback_without_response() {
    for (scheme, allowed, failure) in [
        (HtmlBaseUrlScheme::Other, HtmlBaseAllowed::Allowed, true),
        (HtmlBaseUrlScheme::Other, HtmlBaseAllowed::Blocked, false),
        (HtmlBaseUrlScheme::Data, HtmlBaseAllowed::Unavailable, false),
        (
            HtmlBaseUrlScheme::Javascript,
            HtmlBaseAllowed::Unavailable,
            false,
        ),
    ] {
        let facts = facts("bad", |revision| {
            if failure {
                HtmlBaseParseOutcome::Failure
            } else {
                HtmlBaseParseOutcome::Parsed {
                    record: url(revision, "externally-observed-record", scheme),
                    base_allowed: allowed,
                }
            }
        });
        let report = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><base href=bad><base href=later>"),
            limits(),
        )
        .with_base_facts(facts)
        .unwrap()
        .parse()
        .unwrap();
        let mut count = 0;
        let mut metadata = report.metadata();
        loop {
            match metadata.advance() {
                HtmlParseMetadataStep::Record(record) => {
                    if let Some(Some(frozen)) = record.base_frozen() {
                        count += 1;
                        assert!(!frozen.responded());
                        assert_eq!(
                            frozen.value().serialization(),
                            &DocumentString::from("https://example.test/original/")
                        );
                    }
                }
                HtmlParseMetadataStep::Skipped => {}
                HtmlParseMetadataStep::Complete => break,
            }
        }
        assert_eq!(count, 1);
    }
}

#[test]
fn first_href_missing_facts_is_conditional_and_keeps_original_stop_after_cleanup() {
    for utf16 in [false, true] {
        for raw in [
            "<!doctype html><base href=''><base href=later>",
            "<!doctype html><base href=first><base href=later>",
        ] {
            let request = HtmlDocumentParseRequest::offline(source(raw, utf16), limits());
            let original = request.source().identity().clone();
            let cancel = request.cancellation().clone();
            let failure = request.parse().unwrap_err();
            assert_eq!(
                failure.reason(),
                HtmlParseFailureReason::Stop(HtmlParseStop::BaseFacts(
                    HtmlBaseMissing::DocumentFacts
                ))
            );
            assert_eq!(
                failure.first_stop(),
                Some(HtmlParseStop::BaseFacts(HtmlBaseMissing::DocumentFacts))
            );
            assert_eq!(failure.source().identity(), &original);
            assert_eq!(failure.cleanup_error(), None);
            let usage = failure.after_cleanup().unwrap();
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::OpenElements,
            ] {
                assert_eq!(usage.resource(resource).current(), 0, "{resource:?}");
            }
            cancel.cancel();
            assert_eq!(
                failure.first_stop(),
                Some(HtmlParseStop::BaseFacts(HtmlBaseMissing::DocumentFacts))
            );
        }
        for raw in [
            "<!doctype html><p>X",
            "<!doctype html><base/><p>X",
            "<!doctype html><base target=''><p>X",
            "<!doctype html><template><base href=unused></template><p>X",
        ] {
            let report = HtmlDocumentParseRequest::offline(source(raw, utf16), limits())
                .parse()
                .unwrap();
            assert_eq!(
                report.document().unwrap().mode(),
                HtmlDocumentMode::NoQuirks
            );
            assert!(report.document_base_facts().is_none());
        }
    }
}

#[test]
fn each_influencing_environment_fact_refuses_at_the_first_href_and_preserves_originals() {
    // The first actual href is X; the later Y cannot supply any missing fact.
    // These are source-selected prerequisites, not observed work thresholds.
    for utf16 in [false, true] {
        for missing in [
            HtmlBaseMissing::DocumentUrl,
            HtmlBaseMissing::AboutBaseUrl,
            HtmlBaseMissing::CharacterEncoding,
            HtmlBaseMissing::ParseObservation,
            HtmlBaseMissing::Csp,
        ] {
            let revision = revision();
            let fallback = HtmlBaseUrlRecord::observed(
                revision.clone(),
                if missing == HtmlBaseMissing::AboutBaseUrl {
                    "about:blank?x#y"
                } else {
                    "https://example.test/fallback/"
                }
                .into(),
                HtmlBaseUrlScheme::Other,
                missing == HtmlBaseMissing::AboutBaseUrl,
            )
            .unwrap();
            let encoding = HtmlBaseEncoding::fresh();
            let observations = if missing == HtmlBaseMissing::Csp {
                vec![
                    HtmlBaseParseObservation::new(
                        "X".into(),
                        fallback.clone(),
                        encoding.clone(),
                        revision.clone(),
                        HtmlBaseParseOutcome::Parsed {
                            record: url(
                                &revision,
                                "externally-observed-parsed",
                                HtmlBaseUrlScheme::Other,
                            ),
                            base_allowed: HtmlBaseAllowed::Unavailable,
                        },
                    )
                    .unwrap(),
                ]
            } else {
                vec![]
            };
            let facts = HtmlParseBaseFacts::new(
                HtmlBaseDocumentFacts::new(
                    revision.clone(),
                    (missing != HtmlBaseMissing::DocumentUrl).then_some(fallback),
                    HtmlAboutBaseUrl::Unavailable,
                    false,
                    (missing != HtmlBaseMissing::CharacterEncoding).then_some(encoding),
                )
                .unwrap(),
                observations,
                HtmlBaseFactsLimits::new(10, 1000, 10000),
            )
            .unwrap();
            let request = HtmlDocumentParseRequest::offline(
                source("<!doctype html><base href=X><base href=Y>", utf16),
                limits(),
            )
            .with_base_facts(facts)
            .unwrap();
            let original = request.source().identity().clone();
            let cancel = request.cancellation().clone();
            let failure = request.parse().unwrap_err();
            assert_eq!(
                failure.reason(),
                HtmlParseFailureReason::Stop(HtmlParseStop::BaseFacts(missing))
            );
            assert_eq!(
                failure.first_stop(),
                Some(HtmlParseStop::BaseFacts(missing))
            );
            assert_eq!(failure.source().identity(), &original);
            assert_eq!(
                failure.document_base_facts().unwrap().document().revision(),
                &revision
            );
            assert_eq!(failure.cleanup_error(), None);
            let usage = failure.after_cleanup().unwrap().clone();
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::OpenElements,
            ] {
                assert_eq!(
                    usage.resource(resource).current(),
                    0,
                    "{missing:?}/{resource:?}"
                );
            }
            cancel.cancel();
            assert_eq!(
                failure.first_stop(),
                Some(HtmlParseStop::BaseFacts(missing))
            );
            assert_eq!(failure.after_cleanup(), Some(&usage));
        }
    }
}
