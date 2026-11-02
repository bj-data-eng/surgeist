use crate::parser::support::revision;
use surgeist_html::*;
use surgeist_html_parser::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlParseLimits {
    HtmlParseLimits::builder()
        .source_units(100000)
        .buffered_utf16_units(100000)
        .token_attributes(10000)
        .auxiliary_entries(1000000)
        .managed_nodes(10000)
        .managed_edges(10000)
        .managed_attributes(10000)
        .managed_utf16_units(100000)
        .source_map_entries(100000)
        .source_map_payload_units(100000)
        .diagnostics(10000)
        .diagnostic_payload_units(100000)
        .context_effects(10000)
        .open_elements(100)
        .tree_depth(10000)
        .managed_capacity_bytes(100000000)
        .peak_transient_bytes(100000000)
        .atomic_layout_bytes(100000000)
        .work_credits(100000000)
        .finish()
        .unwrap()
}
fn record(
    r: &HtmlBaseFactsRevision,
    text: &str,
    blank: bool,
    fragment: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        r.clone(),
        text.into(),
        HtmlBaseUrlScheme::Other,
        blank,
        fragment,
    )
    .unwrap()
}
fn environment(base: &HtmlBaseUrlRecord, encoding: &HtmlBaseEncoding) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            base.revision().clone(),
            Some(base.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(100, 10000, 100000),
    )
    .unwrap()
}
fn fixture(
    cached: bool,
) -> (
    HtmlFragmentContext,
    HtmlCaptureDocumentId,
    HtmlCaptureDocumentId,
) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 100, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let doc = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let inert = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_document_template_facts(
        &inert,
        HtmlDocumentClass::Html,
        HtmlTemplateDocumentRole::InertTemplate,
        HtmlCaptureTemplateCacheInput::Absent,
    )
    .unwrap();
    b.establish_document_template_facts(
        &doc,
        HtmlDocumentClass::Html,
        HtmlTemplateDocumentRole::Ordinary,
        if cached {
            HtmlCaptureTemplateCacheInput::Document(inert.clone())
        } else {
            HtmlCaptureTemplateCacheInput::Absent
        },
    )
    .unwrap();
    let div = b
        .add_element(
            &doc,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_no_shadow_root(&div).unwrap();
    b.establish_children(&doc, std::slice::from_ref(&div))
        .unwrap();
    b.establish_children(&div, &[]).unwrap();
    b.establish_children(&inert, &[]).unwrap();
    let capture = b.finish().unwrap();
    let doc = capture
        .document_id(&capture.subject_id(&doc).unwrap())
        .unwrap();
    let inert = capture
        .document_id(&capture.subject_id(&inert).unwrap())
        .unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
    (context, doc, inert)
}
fn selected(
    context: &HtmlFragmentContext,
    doc: &HtmlCaptureDocumentId,
    facts: HtmlParseBaseFacts,
) -> HtmlDocumentEncodingUrlContext {
    let capture = context.capture();
    let revision = facts.document().revision().clone();
    let bank = HtmlCapturedBaseFacts::new(
        capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![HtmlCapturedBaseDocument::new(doc.clone(), facts)],
        vec![],
        HtmlBaseFactsLimits::new(100, 10000, 100000),
    )
    .unwrap();
    let semantic = HtmlSemanticContext::new(
        capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_base_facts(revision, Some(bank)),
        0,
    )
    .unwrap();
    let report = HtmlSemanticSession::new(
        capture,
        &semantic,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(100000, 10, 100, 0, 100000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        doc.subject().clone(),
        HtmlSemanticQuery::DocumentEncodingUrlContext,
    )])
    .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(value)) =
        report.results()[0].fact()
    else {
        panic!("actual selected context")
    };
    value.clone()
}
fn bank(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    raw: DocumentString,
    outcome: HtmlEncodingUrlParseOutcome,
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        base.revision().clone(),
        vec![HtmlEncodingUrlParseObservation::new(raw, outcome)],
        HtmlBaseFactsLimits::new(100, 10000, 100000),
    )
    .unwrap()
}
fn links(report: &HtmlParseReport) -> Vec<HtmlParseHyperlink<'_>> {
    let mut result = Vec::new();
    let mut cursor = report.metadata();
    loop {
        match cursor.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(link) = row.hyperlink() {
                    result.push(link)
                }
            }
            HtmlParseMetadataStep::Skipped => (),
            HtmlParseMetadataStep::Complete => break,
            HtmlParseMetadataStep::Fault(fault) => panic!("{fault}"),
        }
    }
    result
}

#[test]
fn root_nested_and_owned_template_href_keep_actual_source_document_and_outcome() {
    // HTML877d95977/99276–85: root attributes run before adoption; nested div
    // children use its current captured Document. Template contents have their
    // independent actual owner. Adoption never calls A/Area set-URL again.
    let (context, target, _) = fixture(false);
    let aux = record(&revision(), "about:blank", true, None);
    let inert = record(&revision(), "about:blank", true, None);
    let target_base = record(&revision(), "https://example.test/target/", false, None);
    let target_x = record(
        target_base.revision(),
        "https://example.test/target/x",
        false,
        None,
    );
    let a = HtmlBaseEncoding::fresh();
    let i = HtmlBaseEncoding::fresh();
    let t = HtmlBaseEncoding::fresh();
    let prepared = selected(&context, &target, environment(&target_base, &t));
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf8(
            "<a href=x></a><div><a href=x></a><template><area href=x></template></div>",
        ),
        context,
        limits(),
    )
    .unwrap()
    .with_parser_document_base_facts(environment(&aux, &a))
    .unwrap()
    .with_template_base_facts(environment(&inert, &i))
    .unwrap()
    .with_encoding_url_contexts(vec![prepared])
    .with_encoding_url_facts(vec![
        bank(&aux, &a, "x".into(), HtmlEncodingUrlParseOutcome::Failure),
        bank(
            &target_base,
            &t,
            "x".into(),
            HtmlEncodingUrlParseOutcome::Parsed(target_x.clone()),
        ),
        bank(&inert, &i, "x".into(), HtmlEncodingUrlParseOutcome::Failure),
    ])
    .parse()
    .unwrap();
    assert!(
        report.captured_base_facts().is_none(),
        "selected context does not retain whole bank"
    );
    let links = links(&report);
    assert_eq!(links.len(), 3);
    let mut roles = Vec::new();
    for link in links {
        let HtmlParseHyperlinkState::Observed(value) = link.state() else {
            panic!("actual observed success or failure")
        };
        assert_eq!(value.raw(), &DocumentString::from("x"));
        assert!(link.cause().is_some());
        if value.base() == &aux {
            let HtmlParseMetadataDocument::Owned(doc) = link.document() else {
                panic!("auxiliary parse Document")
            };
            assert_eq!(
                doc.id(),
                report
                    .fragment_parser_document()
                    .unwrap()
                    .unwrap()
                    .document()
                    .id()
            );
            assert_eq!(value.encoding(), &a);
            assert!(value.url().is_none());
            roles.push("aux");
        } else if value.base() == &target_base {
            let HtmlParseMetadataDocument::Captured(doc) = link.document() else {
                panic!("captured target")
            };
            assert_eq!(doc.id(), *target.subject());
            assert_eq!(value.encoding(), &t);
            assert_eq!(value.url(), Some(&target_x));
            roles.push("target");
        } else {
            assert_eq!(value.base(), &inert);
            let HtmlParseMetadataDocument::Owned(doc) = link.document() else {
                panic!("owned inert Document")
            };
            assert_ne!(
                doc.id(),
                report
                    .fragment_parser_document()
                    .unwrap()
                    .unwrap()
                    .document()
                    .id()
            );
            assert_eq!(value.encoding(), &i);
            assert!(value.url().is_none());
            roles.push("inert");
        }
    }
    roles.sort_unstable();
    assert_eq!(roles, ["aux", "inert", "target"]);
}

#[test]
fn existing_captured_inert_owner_uses_its_own_revision_and_encoding() {
    let (context, target, inert_doc) = fixture(true);
    let target_base = record(&revision(), "https://example.test/", false, None);
    let inert_base = record(&revision(), "about:blank", true, None);
    let t = HtmlBaseEncoding::fresh();
    let i = HtmlBaseEncoding::fresh();
    let target_context = selected(&context, &target, environment(&target_base, &t));
    let inert_context = selected(&context, &inert_doc, environment(&inert_base, &i));
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<template><area href=x></template>"),
        context,
        limits(),
    )
    .unwrap()
    .with_encoding_url_contexts(vec![target_context, inert_context])
    .with_encoding_url_facts(vec![bank(
        &inert_base,
        &i,
        "x".into(),
        HtmlEncodingUrlParseOutcome::Failure,
    )])
    .parse()
    .unwrap();
    let links = links(&report);
    assert_eq!(links.len(), 1);
    let HtmlParseMetadataDocument::Captured(doc) = links[0].document() else {
        panic!("actual captured inert")
    };
    assert_eq!(doc.id(), *inert_doc.subject());
    let HtmlParseHyperlinkState::Observed(value) = links[0].state() else {
        panic!("observed failure")
    };
    assert_eq!(value.encoding(), &i);
    assert_eq!(value.revision(), inert_base.revision());
}

#[test]
fn foreign_and_duplicate_selected_contexts_preserve_original_execution_inputs() {
    let (context, target, _) = fixture(false);
    let (foreign, foreign_doc, _) = fixture(false);
    let base = record(&revision(), "https://example.test/", false, None);
    let enc = HtmlBaseEncoding::fresh();
    let own = selected(&context, &target, environment(&base, &enc));
    let other = selected(&foreign, &foreign_doc, environment(&base, &enc));
    for values in [vec![other], vec![own.clone(), own.clone()]] {
        let mut supplied = Vec::with_capacity(13);
        supplied.extend(values);
        let capacity = supplied.capacity();
        let expected = supplied.clone();
        let observation = bank(
            &base,
            &enc,
            "http://[".into(),
            HtmlEncodingUrlParseOutcome::Failure,
        );
        let failure = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><a href='http://['>"),
            context.clone(),
            limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(supplied)
        .with_encoding_url_facts(vec![observation.clone()])
        .parse()
        .unwrap_err();
        assert_eq!(
            failure.first_stop(),
            Some(HtmlParseStop::InvalidAssociatedFacts)
        );
        assert_eq!(failure.encoding_url_contexts(), Some(expected.as_slice()));
        assert_eq!(failure.encoding_url_context_capacity(), Some(capacity));
        assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&observation));
        assert_eq!(failure.cleanup_error(), None);
    }
}

#[test]
fn missing_stale_foreign_and_mismatched_observations_are_unavailable_not_url_failure() {
    let (context, target, _) = fixture(false);
    let r = revision();
    let base = record(&r, "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let prepared = selected(&context, &target, environment(&base, &encoding));
    let older = HtmlBaseFactsRevision::new(r.producer().clone(), 0, 2, 3);
    let cases = [
        (vec![], HtmlEncodingUrlFactsError::MissingObservation),
        (
            vec![bank(
                &base,
                &encoding,
                "https://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlEncodingUrlFactsError::MissingObservation,
        ),
        (
            vec![bank(
                &record(&revision(), "https://foreign.test/", false, None),
                &encoding,
                "http://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlEncodingUrlFactsError::ForeignProducer,
        ),
        (
            vec![
                HtmlEncodingUrlParseFacts::new(
                    base.clone(),
                    encoding.clone(),
                    older,
                    vec![],
                    HtmlBaseFactsLimits::new(100, 10000, 100000),
                )
                .unwrap(),
            ],
            HtmlEncodingUrlFactsError::RevisionMismatch,
        ),
        (
            vec![bank(
                &base,
                &HtmlBaseEncoding::fresh(),
                "http://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            vec![bank(
                &record(&r, "https://other.test/", false, None),
                &encoding,
                "http://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
    ];
    for (facts, expected) in cases {
        let report = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><a href='http://['><area>"),
            context.clone(),
            limits(),
        )
        .unwrap()
        .with_encoding_url_contexts(vec![prepared.clone()])
        .with_encoding_url_facts(facts)
        .parse()
        .unwrap();
        let links = links(&report);
        assert_eq!(links.len(), 2);
        assert!(
            matches!(links[0].state(), HtmlParseHyperlinkState::Unavailable(HtmlParseHyperlinkUnavailable::Observation(actual)) if *actual == expected)
        );
        assert!(matches!(
            links[1].state(),
            HtmlParseHyperlinkState::AbsentHref
        ));
    }
    let missing = HtmlFragmentParseRequest::new(
        HtmlSource::utf8("<div><a href='http://['>"),
        context,
        limits(),
    )
    .unwrap()
    .parse()
    .unwrap();
    assert!(matches!(
        links(&missing)[0].state(),
        HtmlParseHyperlinkState::Unavailable(HtmlParseHyperlinkUnavailable::DocumentEnvironment)
    ));
}

#[test]
fn raw_isolated_surrogates_and_detached_base_preserve_exact_selected_input() {
    let (context, target, _) = fixture(false);
    let base = record(&revision(), "https://example.test/old/", false, None);
    // Same literal UTF-8 replacement result as the retained shared EncodingUrl
    // surrogate fixture; raw DOMString code units themselves stay unchanged.
    let parsed = record(
        base.revision(),
        "https://example.test/old/%EF%BF%BDx%EF%BF%BD",
        false,
        None,
    );
    let encoding = HtmlBaseEncoding::fresh();
    let prepared = selected(&context, &target, environment(&base, &encoding));
    let raw = DocumentString::from_utf16(vec![0xd800, 120, 0xdc00]);
    let mut source = "<div><base href='https://other.test/'><a href='"
        .encode_utf16()
        .collect::<Vec<_>>();
    source.extend_from_slice(raw.code_units());
    source.extend("'>".encode_utf16());
    let report = HtmlFragmentParseRequest::new(
        HtmlSource::utf16(DocumentString::from_utf16(source)),
        context,
        limits(),
    )
    .unwrap()
    .with_encoding_url_contexts(vec![prepared])
    .with_encoding_url_facts(vec![bank(
        &base,
        &encoding,
        raw.clone(),
        HtmlEncodingUrlParseOutcome::Parsed(parsed.clone()),
    )])
    .parse()
    .unwrap();
    let links = links(&report);
    let HtmlParseHyperlinkState::Observed(value) = links[0].state() else {
        panic!("exact surrogate observation")
    };
    assert_eq!(value.raw(), &raw);
    assert_eq!(
        value.base(),
        &base,
        "detached fragment Base cannot enter captured Document first-Base order"
    );
    assert_eq!(value.encoding(), &encoding);
    assert_eq!(value.url(), Some(&parsed));
}

#[test]
fn constructor_roles_reject_contradictory_facts_without_issuing_a_request() {
    let (context, _, _) = fixture(false);
    let not_blank = record(&revision(), "https://example.test/", false, None);
    let facts = environment(&not_blank, &HtmlBaseEncoding::fresh());
    assert!(
        HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context.clone(), limits())
            .unwrap()
            .with_parser_document_base_facts(facts.clone())
            .is_err()
    );
    assert!(
        HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context, limits())
            .unwrap()
            .with_template_base_facts(facts)
            .is_err()
    );
}

#[test]
fn selected_execution_spare_capacity_is_released_and_failure_returns_originals() {
    let (context, target, _) = fixture(false);
    let base = record(&revision(), "https://example.test/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let prepared = selected(&context, &target, environment(&base, &encoding));
    let observation = bank(
        &base,
        &encoding,
        "http://[".into(),
        HtmlEncodingUrlParseOutcome::Failure,
    );
    let request = |capacity, limits| {
        let mut contexts = Vec::with_capacity(capacity);
        contexts.push(prepared.clone());
        let mut facts = Vec::with_capacity(capacity);
        facts.push(observation.clone());
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<div><a href='http://['>"),
            context.clone(),
            limits,
        )
        .unwrap()
        .with_encoding_url_contexts(contexts)
        .with_encoding_url_facts(facts)
    };
    let sparse = request(1, limits()).parse().unwrap();
    let spare = request(128, limits()).parse().unwrap();
    assert_eq!(
        sparse.retained_construction_bytes(),
        spare.retained_construction_bytes()
    );
    assert_eq!(
        spare.retained_construction_bytes(),
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .current()
    );
    assert!(
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water()
            > sparse
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water()
    );
    // Only the actual execution Vec backing differs: each occurrence owns one
    // public selected-context header and one opaque URL-bank handle. Graphs,
    // payload slots and source are identical; no allocator/header guess is used.
    assert_eq!(
        spare
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water()
            - sparse
                .usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .high_water(),
        (127 * (size_of::<HtmlDocumentEncodingUrlContext>()
            + size_of::<HtmlEncodingUrlParseFacts>())) as u64
    );
    let cancelled = request(128, limits());
    cancelled.cancellation().cancel();
    let failure = cancelled.parse().unwrap_err();
    assert_eq!(failure.first_stop(), Some(HtmlParseStop::Cancelled));
    assert_eq!(
        failure.encoding_url_contexts(),
        Some([prepared.clone()].as_slice())
    );
    assert_eq!(failure.encoding_url_context_capacity(), Some(128));
    assert_eq!(failure.encoding_url_fact_capacity(), Some(128));
    assert!(failure.encoding_url_facts().unwrap()[0].same_owner(&observation));
    assert_eq!(failure.cleanup_error(), None);
    for resource in [
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::WorkCredits,
    ] {
        let peak = sparse.usage().resource(resource).high_water();
        let exact = match resource {
            HtmlParseResource::ManagedCapacityBytes => limits()
                .into_builder()
                .managed_capacity_bytes(peak)
                .finish()
                .unwrap(),
            _ => limits().into_builder().work_credits(peak).finish().unwrap(),
        };
        request(1, exact).parse().unwrap();
        let one_short = match resource {
            HtmlParseResource::ManagedCapacityBytes => limits()
                .into_builder()
                .managed_capacity_bytes(peak - 1)
                .finish()
                .unwrap(),
            _ => limits()
                .into_builder()
                .work_credits(peak - 1)
                .finish()
                .unwrap(),
        };
        let failure = request(1, one_short).parse().unwrap_err();
        assert!(
            matches!(failure.first_stop(), Some(HtmlParseStop::LimitExceeded { resource: actual, .. }) if actual == resource)
        );
        assert_eq!(failure.encoding_url_context_capacity(), Some(1));
        assert_eq!(failure.cleanup_error(), None);
    }
}

#[test]
fn historical_first_base_uses_current_independent_revision_after_detached_meta() {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100000, 100, 0, 0, 16_777_216),
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
    let first_base = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "base"),
            OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, "href"),
                "old",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
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
    builder.establish_no_shadow_root(&div).unwrap();
    builder
        .establish_children(&document, &[first_base.clone(), div.clone()])
        .unwrap();
    builder.establish_children(&div, &[]).unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let context =
        HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap();
    let current = revision();
    let historical = revision();
    let parsed = record(&current, "https://example.test/old/x", false, None);
    let fallback = record(&current, "https://example.test/current/", false, None);
    let frozen = record(&historical, "https://example.test/old/", false, None);
    let encoding = HtmlBaseEncoding::fresh();
    let bank = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        current.clone(),
        vec![HtmlCapturedBaseDocument::new(
            document.clone(),
            environment(&fallback, &encoding),
        )],
        vec![HtmlCapturedFrozenBase::new(
            capture.subject_id(&first_base).unwrap(),
            document.clone(),
            "old".into(),
            Some(frozen.clone()),
            historical.clone(),
        )],
        HtmlBaseFactsLimits::new(100, 10000, 100000),
    )
    .unwrap();
    let semantic = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_base_facts(current.clone(), Some(bank)),
        0,
    )
    .unwrap();
    let queried = HtmlSemanticSession::new(
        &capture,
        &semantic,
        HtmlValueInterpreter::new(10000),
        HtmlSemanticLimits::new(100000, 10, 100, 0, 100000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        document.subject().clone(),
        HtmlSemanticQuery::DocumentEncodingUrlContext,
    )])
    .unwrap();
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(prepared)) =
        queried.results()[0].fact()
    else {
        panic!("first frozen Base")
    };
    assert_eq!(prepared.base_url().frozen_revision(), Some(&historical));
    assert_eq!(prepared.revision(), &current);
    let observation = HtmlEncodingUrlParseFacts::new(
        frozen.clone(),
        encoding.clone(),
        current.clone(),
        vec![HtmlEncodingUrlParseObservation::new(
            "x".into(),
            HtmlEncodingUrlParseOutcome::Parsed(parsed.clone()),
        )],
        HtmlBaseFactsLimits::new(100, 10000, 100000),
    )
    .unwrap();
    // A Meta in detached fragment content never reaches the captured Document's
    // direct-head CSP operation. The immutable selection cannot bypass a live
    // policy transition; there is none in this closed fragment effect domain.
    let report = HtmlFragmentParseRequest::new(HtmlSource::utf8("<div><meta http-equiv=Content-Security-Policy content=\"default-src 'none'\"><a href=x>"), context, limits()).unwrap().with_encoding_url_contexts(vec![prepared.clone()]).with_encoding_url_facts(vec![observation]).parse().unwrap();
    let links = links(&report);
    let HtmlParseHyperlinkState::Observed(value) = links[0].state() else {
        panic!("historical Base/current URL revision")
    };
    assert_eq!(value.base(), &frozen);
    assert_eq!(value.revision(), &current);
    assert_eq!(value.encoding(), &encoding);
    assert_eq!(value.url(), Some(&parsed));
    let HtmlParseMetadataDocument::Captured(actual) = links[0].document() else {
        panic!("actual target")
    };
    assert_eq!(actual.id(), *document.subject());
}
