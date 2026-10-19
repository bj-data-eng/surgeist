#![forbid(unsafe_code)]
use surgeist_html::*;
use surgeist_html_semantics::*;

fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(100_000, 20, 100, 0, 100_000, 0)
}
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn url(revision: &HtmlBaseFactsRevision, text: &str, fragment: Option<usize>) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        text.into(),
        HtmlBaseUrlScheme::Other,
        false,
        fragment,
    )
    .unwrap()
}
fn fixture(
    children: bool,
    frozen: bool,
    encoding: Option<HtmlBaseEncoding>,
) -> (
    HtmlCapture,
    HtmlCaptureDocumentId,
    HtmlSemanticContext,
    HtmlBaseUrlRecord,
) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100, 100, 100_000, 100, 0, 0, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let base = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "base"),
            OrderedAttributes::new(vec![Attribute::new(
                QualifiedName::new(Namespace::none(), None, "href"),
                "x",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let later = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    if children {
        builder
            .establish_children(&document, &[base.clone(), later])
            .unwrap();
    }
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let current = revision();
    let historical = revision();
    let fallback = url(&current, "https://example.test/fallback/", None);
    let selected = url(&historical, "https://example.test/historical/", None);
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            current.clone(),
            Some(fallback),
            HtmlAboutBaseUrl::KnownNull,
            false,
            encoding,
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(20, 1000, 10000),
    )
    .unwrap();
    let bank = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        current.clone(),
        vec![HtmlCapturedBaseDocument::new(document.clone(), facts)],
        if children {
            vec![HtmlCapturedFrozenBase::new(
                capture.subject_id(&base).unwrap(),
                document.clone(),
                "x".into(),
                frozen.then(|| selected.clone()),
                historical,
            )]
        } else {
            vec![]
        },
        HtmlBaseFactsLimits::new(20, 1000, 10000),
    )
    .unwrap();
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_base_facts(current, Some(bank)),
        0,
    )
    .unwrap();
    (capture, document, context, selected)
}
fn query(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    document: &HtmlCaptureDocumentId,
    limit: HtmlSemanticLimits,
    cancel: HtmlSemanticCancellation,
) -> Result<HtmlSemanticReport, HtmlSemanticFailure> {
    HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(10000),
        limit,
        cancel,
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        document.subject().clone(),
        HtmlSemanticQuery::DocumentEncodingUrlContext,
    )])
}

#[test]
fn selected_context_reuses_first_base_and_keeps_opaque_encoding_and_current_revision() {
    // HTML877d #document-base-url uses the first frozen Base even when its
    // producing revision is historical; encoding is the current Document's.
    let encoding = HtmlBaseEncoding::fresh();
    let (capture, document, context, expected) = fixture(true, true, Some(encoding.clone()));
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[
        HtmlSemanticRequest::new(
            document.subject().clone(),
            HtmlSemanticQuery::DocumentBaseUrl,
        ),
        HtmlSemanticRequest::new(
            document.subject().clone(),
            HtmlSemanticQuery::DocumentEncodingUrlContext,
        ),
    ])
    .unwrap();
    let HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Known(base)) =
        report.results()[0].fact()
    else {
        panic!("actual first Base")
    };
    let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Known(value)) =
        report.results()[1].fact()
    else {
        panic!("qualified context")
    };
    assert_eq!(value.base_url(), base);
    assert_eq!(value.base_url().value(), &expected);
    assert_eq!(value.document(), &document);
    assert_eq!(value.host_revision(), capture.host_facts_revision());
    assert_eq!(value.encoding(), &encoding);
    assert_eq!(Some(value.revision()), context.base_revision());
    assert_ne!(value.base_url().frozen_revision(), Some(value.revision()));
    let retained = value.clone();
    drop(report);
    drop(context);
    assert_eq!(retained.base_url().value(), &expected);
    assert_eq!(retained.encoding(), &encoding);
    assert_eq!(
        retained.retained_storage_usage().retained_bytes(),
        size_of::<HtmlDocumentEncodingUrlContext>()
            + expected.retained_storage_usage().retained_bytes()
    );
    assert_eq!(
        retained.retained_storage_usage().utf16_extent(),
        expected.retained_storage_usage().utf16_extent()
    );
}

#[test]
fn unavailable_topology_frozen_state_and_encoding_never_become_empty_context() {
    for (children, frozen, encoding, missing) in [
        (false, false, Some(HtmlBaseEncoding::fresh()), None),
        (
            true,
            false,
            Some(HtmlBaseEncoding::fresh()),
            Some(HtmlBaseMissing::FrozenBase),
        ),
        (true, true, None, Some(HtmlBaseMissing::CharacterEncoding)),
    ] {
        let (capture, document, context, _) = fixture(children, frozen, encoding);
        let report = query(
            &capture,
            &context,
            &document,
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Missing(actual)) =
            report.results()[0].fact()
        else {
            panic!("explicit missing")
        };
        match missing {
            Some(expected) => assert!(
                matches!(actual, HtmlSemanticMissing::Base { fact, .. } if *fact == expected)
            ),
            None => assert!(
                matches!(actual, HtmlSemanticMissing::OrderedChildren(id) if id == document.subject())
            ),
        }
    }
}

#[test]
fn selected_query_work_and_scratch_are_finite_and_first_cancellation_is_sticky() {
    let (capture, document, context, _) = fixture(true, true, Some(HtmlBaseEncoding::fresh()));
    let complete = query(
        &capture,
        &context,
        &document,
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let usage = complete.usage();
    for work in 0..usage.work() {
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(10000),
            HtmlSemanticLimits::new(work, 20, 100, 0, 100_000, 0),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let failure = session
            .query(&[HtmlSemanticRequest::new(
                document.subject().clone(),
                HtmlSemanticQuery::DocumentEncodingUrlContext,
            )])
            .unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                ..
            }
        ));
        assert_eq!(session.usage().scratch_records(), 0);
        assert_eq!(session.failure(), Some(&failure));
    }
    query(
        &capture,
        &context,
        &document,
        HtmlSemanticLimits::new(usage.work(), 20, 100, 0, 100_000, 0),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let failure = query(&capture, &context, &document, limits(), cancel).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert!(failure.qualification().is_some());
}

#[test]
fn result_and_traversal_capacities_have_exact_and_one_short_boundaries() {
    let (capture, document, context, _) = fixture(true, true, Some(HtmlBaseEncoding::fresh()));
    let report = query(
        &capture,
        &context,
        &document,
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let usage = report.usage();
    // Independent domains: one returned result and the actual first-Base DFS
    // scratch/allocator capacities measured by the owning session (not URL-bank
    // formation or selected-value retained bytes).
    for resource in [
        HtmlSemanticResource::OutputRecords,
        HtmlSemanticResource::ScratchRecords,
        HtmlSemanticResource::AllocatedBytes,
    ] {
        let (outputs, scratch, bytes) = (
            usage.output_records(),
            usage.peak_scratch_records(),
            usage.allocated_bytes(),
        );
        let exact = HtmlSemanticLimits::new(100000, outputs, scratch, 0, bytes, 0);
        query(
            &capture,
            &context,
            &document,
            exact,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let one_short = match resource {
            HtmlSemanticResource::OutputRecords => {
                HtmlSemanticLimits::new(100000, outputs - 1, scratch, 0, bytes, 0)
            }
            HtmlSemanticResource::ScratchRecords => {
                HtmlSemanticLimits::new(100000, outputs, scratch - 1, 0, bytes, 0)
            }
            _ => HtmlSemanticLimits::new(100000, outputs, scratch, 0, bytes - 1, 0),
        };
        let failure = query(
            &capture,
            &context,
            &document,
            one_short,
            HtmlSemanticCancellation::new(),
        )
        .unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
        );
    }
}

#[test]
fn absent_bank_preserves_first_frozen_base_missing_and_non_document_is_inapplicable() {
    let (capture, document, _, _) = fixture(true, true, Some(HtmlBaseEncoding::fresh()));
    let context = HtmlSemanticContext::new(
        &capture,
        capture.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        0,
    )
    .unwrap();
    let report = query(
        &capture,
        &context,
        &document,
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::Base {
                fact: HtmlBaseMissing::FrozenBase,
                ..
            }
        ))
    ));
    let HtmlCapturedChildren::Complete(mut children) =
        capture.node(document.subject()).unwrap().children()
    else {
        panic!("complete fixture")
    };
    let element = children.next().unwrap();
    let report = HtmlSemanticSession::new(
        &capture,
        &context,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[HtmlSemanticRequest::new(
        element,
        HtmlSemanticQuery::DocumentEncodingUrlContext,
    )])
    .unwrap();
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::DocumentEncodingUrlContext(HtmlSemanticValue::Inapplicable)
    ));
}
