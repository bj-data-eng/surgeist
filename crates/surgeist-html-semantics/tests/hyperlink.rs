use surgeist_html::*;
use surgeist_html_semantics::*;

fn rev(producer: &HtmlBaseProducerId, n: u64) -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(producer.clone(), n, n, n)
}
fn url(revision: &HtmlBaseFactsRevision, text: &str) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        text.into(),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap()
}
fn limits() -> HtmlBaseFactsLimits {
    HtmlBaseFactsLimits::new(100, 10000, 10000)
}
fn facts(raw: DocumentString, outcome: HtmlEncodingUrlParseOutcome) -> HtmlEncodingUrlParseFacts {
    let revision = match &outcome {
        HtmlEncodingUrlParseOutcome::Parsed(record) => record.revision().clone(),
        _ => rev(&HtmlBaseProducerId::fresh(), 1),
    };
    HtmlEncodingUrlParseFacts::new(
        url(&revision, "https://base/"),
        HtmlBaseEncoding::fresh(),
        revision,
        vec![HtmlEncodingUrlParseObservation::new(raw, outcome)],
        limits(),
    )
    .unwrap()
}
fn resolve(facts: &HtmlEncodingUrlParseFacts, raw: &DocumentString) -> HtmlHyperlinkUrlSnapshot {
    facts
        .resolve_hyperlink_with(
            raw,
            facts.base(),
            facts.encoding(),
            facts.revision(),
            || Ok::<_, ()>(()),
        )
        .unwrap()
}

#[test]
fn document_base_selection_prefers_actual_historical_frozen_record() {
    let producer = HtmlBaseProducerId::fresh();
    let current = rev(&producer, 2);
    let fallback = url(&current, "https://fallback/");
    let historical = url(&rev(&producer, 1), "https://frozen/");
    let document = HtmlBaseDocumentFacts::new(
        current,
        Some(fallback.clone()),
        HtmlAboutBaseUrl::KnownNull,
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        select_document_base(Some(&document), HtmlDocumentFirstBase::Absent),
        Ok(&fallback)
    );
    assert_eq!(
        select_document_base(None, HtmlDocumentFirstBase::Present(Some(&historical))),
        Ok(&historical)
    );
    assert_eq!(
        select_document_base(Some(&document), HtmlDocumentFirstBase::Present(None)),
        Err(HtmlBaseMissing::FrozenBase)
    );
    assert_eq!(
        select_document_base(None, HtmlDocumentFirstBase::Absent),
        Err(HtmlBaseMissing::DocumentFacts)
    );
}
#[test]
fn positive_and_failed_snapshots_keep_exact_qualified_history() {
    let producer = HtmlBaseProducerId::fresh();
    let revision = rev(&producer, 3);
    let base = url(&rev(&producer, 1), "https://frozen/");
    let parsed = url(&revision, "https://frozen/result");
    let encoding = HtmlBaseEncoding::fresh();
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let facts = HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        vec![
            HtmlEncodingUrlParseObservation::new(
                raw.clone(),
                HtmlEncodingUrlParseOutcome::Parsed(parsed.clone()),
            ),
            HtmlEncodingUrlParseObservation::new("".into(), HtmlEncodingUrlParseOutcome::Failure),
        ],
        limits(),
    )
    .unwrap();
    assert!(facts.same_owner(&facts.clone()));
    let snapshot = resolve(&facts, &raw);
    assert_eq!(snapshot.url(), Some(&parsed));
    assert_eq!(snapshot.raw().code_units(), &[0xd800, 0, 0xdc00]);
    assert_eq!(snapshot.base(), &base);
    assert_eq!(snapshot.encoding(), &encoding);
    assert_eq!(snapshot.revision(), &revision);
    assert_eq!(snapshot, resolve(&facts, &raw));
    let failed = resolve(&facts, &"".into());
    assert_eq!(failed.url(), None);
    assert!(matches!(
        failed.outcome(),
        HtmlEncodingUrlParseOutcome::Failure
    ));
    assert_eq!(
        facts.resolve_hyperlink_with(
            &"absent".into(),
            &base,
            &encoding,
            &revision,
            || Ok::<_, ()>(())
        ),
        Err(HtmlEncodingUrlLookupError::Facts(
            HtmlEncodingUrlFactsError::MissingObservation
        ))
    );
    drop(facts);
    assert_eq!(snapshot.url(), Some(&parsed));
}
#[test]
fn qualification_refuses_foreign_stale_and_equal_spelling_different_identity() {
    let facts = facts("A".into(), HtmlEncodingUrlParseOutcome::Failure);
    let another = url(facts.revision(), "https://base/");
    assert_eq!(
        facts.validate_inputs(&another, facts.encoding(), facts.revision()),
        Err(HtmlEncodingUrlFactsError::InputsMismatch)
    );
    assert_eq!(
        facts.validate_inputs(facts.base(), &HtmlBaseEncoding::fresh(), facts.revision()),
        Err(HtmlEncodingUrlFactsError::InputsMismatch)
    );
    for changed in [
        HtmlBaseFactsRevision::new(facts.revision().producer().clone(), 2, 1, 1),
        HtmlBaseFactsRevision::new(facts.revision().producer().clone(), 1, 2, 1),
        HtmlBaseFactsRevision::new(facts.revision().producer().clone(), 1, 1, 2),
    ] {
        assert_eq!(
            facts.validate_inputs(facts.base(), facts.encoding(), &changed),
            Err(HtmlEncodingUrlFactsError::RevisionMismatch)
        );
    }
    assert_eq!(
        facts.validate_inputs(
            facts.base(),
            facts.encoding(),
            &rev(&HtmlBaseProducerId::fresh(), 1)
        ),
        Err(HtmlEncodingUrlFactsError::ForeignProducer)
    );
    assert!(matches!(
        facts.resolve_hyperlink_with(
            &"a".into(),
            facts.base(),
            facts.encoding(),
            facts.revision(),
            || Ok::<_, ()>(())
        ),
        Err(HtmlEncodingUrlLookupError::Facts(
            HtmlEncodingUrlFactsError::MissingObservation
        ))
    ));
}
#[test]
fn formation_charges_capacity_utf16_backing_and_exact_work() {
    let revision = rev(&HtmlBaseProducerId::fresh(), 1);
    let base = url(&revision, "b");
    let mut raw = Vec::with_capacity(40);
    raw.push(65);
    let raw = DocumentString::from_utf16(raw);
    let raw_extent = raw.shared_code_units().capacity();
    let mut observations = Vec::with_capacity(8);
    observations.push(HtmlEncodingUrlParseObservation::new(
        raw.clone(),
        HtmlEncodingUrlParseOutcome::Failure,
    ));
    let capacity = observations.capacity();
    let encoding = HtmlBaseEncoding::fresh();
    let make = |limits| {
        let mut bank = Vec::with_capacity(capacity);
        bank.push(HtmlEncodingUrlParseObservation::new(
            raw.clone(),
            HtmlEncodingUrlParseOutcome::Failure,
        ));
        HtmlEncodingUrlParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            bank,
            limits,
        )
    };
    // Clone of Vec can trim spare capacity, so form the original bank to observe it.
    let formed = HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        observations,
        limits(),
    )
    .unwrap();
    assert_eq!(formed.formation_usage().record_capacity(), capacity);
    assert_eq!(
        formed.formation_usage().utf16_extent(),
        base.serialization().shared_code_units().capacity() + raw_extent
    );
    assert_eq!(formed.formation_usage().validation_work(), 3);
    assert_eq!(
        resolve(&formed, &raw).retained_utf16_extent(),
        formed.formation_usage().utf16_extent()
    );
    assert!(matches!(
        make(HtmlBaseFactsLimits::new(capacity - 1, 1000, 100)),
        Err(HtmlEncodingUrlFactsError::Formation(
            HtmlBaseFactsError::RecordLimit
        ))
    ));
    assert!(matches!(
        make(HtmlBaseFactsLimits::new(8, raw_extent, 100)),
        Err(HtmlEncodingUrlFactsError::Formation(
            HtmlBaseFactsError::Utf16Limit
        ))
    ));
    assert!(matches!(
        make(HtmlBaseFactsLimits::new(8, 1000, 2)),
        Err(HtmlEncodingUrlFactsError::Formation(
            HtmlBaseFactsError::WorkLimit
        ))
    ));
    assert!(
        make(HtmlBaseFactsLimits::new(
            8,
            base.serialization().shared_code_units().capacity() + raw_extent,
            3
        ))
        .is_ok()
    );
}
#[test]
fn duplicate_and_wrong_result_revision_are_refused() {
    let revision = rev(&HtmlBaseProducerId::fresh(), 1);
    let base = url(&revision, "b");
    let encoding = HtmlBaseEncoding::fresh();
    let observation =
        HtmlEncodingUrlParseObservation::new("x".into(), HtmlEncodingUrlParseOutcome::Failure);
    assert!(matches!(
        HtmlEncodingUrlParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            vec![observation.clone(), observation],
            limits()
        ),
        Err(HtmlEncodingUrlFactsError::Formation(
            HtmlBaseFactsError::DuplicateObservation
        ))
    ));
    let wrong = url(&rev(revision.producer(), 2), "p");
    assert!(matches!(
        HtmlEncodingUrlParseFacts::new(
            base,
            encoding,
            revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "x".into(),
                HtmlEncodingUrlParseOutcome::Parsed(wrong)
            )],
            limits()
        ),
        Err(HtmlEncodingUrlFactsError::RevisionMismatch)
    ));
}
#[test]
fn each_lookup_charge_is_interruptible_and_first_error_is_preserved() {
    let facts = facts("abcd".into(), HtmlEncodingUrlParseOutcome::Failure);
    // Context + observation + four compared units + issuance = seven steps.
    for failure in 1..=7 {
        let mut calls = 0;
        let result = facts.resolve_hyperlink_with(
            &"abcd".into(),
            facts.base(),
            facts.encoding(),
            facts.revision(),
            || {
                calls += 1;
                if calls == failure {
                    Err((failure, "actual consumer error"))
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(
            result,
            Err(HtmlEncodingUrlLookupError::Interrupted((
                failure,
                "actual consumer error"
            )))
        );
        assert_eq!(calls, failure);
    }
    let mut calls = 0;
    facts
        .resolve_hyperlink_with(
            &"abcd".into(),
            facts.base(),
            facts.encoding(),
            facts.revision(),
            || {
                calls += 1;
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    assert_eq!(calls, 7);
}

fn capture_limits(units: usize, relationships: usize) -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(10, relationships, 10, units, 10, 0, 0)
}
fn document(b: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
    b.add_document(
        HtmlDocumentMode::NoQuirks,
        HtmlScriptingStatus::Disabled,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
fn element(
    b: &mut HtmlCaptureBuilder,
    d: &HtmlCaptureBuilderSubject,
    local: &str,
    href: Option<&str>,
) -> HtmlCaptureBuilderSubject {
    let attrs =
        href.map(|v| Attribute::new(QualifiedName::new(Namespace::none(), None, "href"), v));
    b.add_element(
        d,
        QualifiedName::new(Namespace::html(), None, local),
        OrderedAttributes::new(attrs).unwrap(),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Null,
    )
    .unwrap()
}
#[test]
fn capture_preserves_historical_document_separately_from_current_document() {
    let facts = facts("x".into(), HtmlEncodingUrlParseOutcome::Failure);
    let snapshot = resolve(&facts, &"x".into());
    let mut builder =
        HtmlCaptureBuilder::new(capture_limits(10000, 100), HtmlHostFactsRevision::fresh());
    let historical = document(&mut builder);
    let current = document(&mut builder);
    let a = element(&mut builder, &current, "a", Some("x"));
    let fresh = element(&mut builder, &current, "area", None);
    let unknown = element(&mut builder, &current, "a", None);
    let before = builder.usage();
    builder
        .establish_hyperlink_url_with(
            &a,
            Some((&historical, snapshot.clone())),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(builder.usage().relationships(), before.relationships() + 1);
    assert_eq!(
        builder.usage().utf16_units(),
        before.utf16_units() + snapshot.retained_utf16_extent()
    );
    builder
        .establish_hyperlink_url_with(&fresh, None, || Ok::<_, ()>(()))
        .unwrap();
    let capture = builder.finish().unwrap();
    let node = capture.node(&capture.subject_id(&a).unwrap()).unwrap();
    assert_eq!(
        node.node_document(),
        capture
            .document_id(&capture.subject_id(&current).unwrap())
            .unwrap()
    );
    let HtmlCapturedNodeKind::Element(value) = node.kind() else {
        panic!()
    };
    assert_eq!(
        value.hyperlink_url(),
        Some(&HtmlCapturedHyperlinkUrl::Observed {
            document: capture
                .document_id(&capture.subject_id(&historical).unwrap())
                .unwrap(),
            snapshot
        })
    );
    for (handle, expected) in [
        (fresh, Some(HtmlCapturedHyperlinkUrl::Null)),
        (unknown, None),
    ] {
        let HtmlCapturedNodeKind::Element(value) = capture
            .node(&capture.subject_id(&handle).unwrap())
            .unwrap()
            .kind()
        else {
            panic!()
        };
        assert_eq!(value.hyperlink_url(), expected.as_ref());
    }
}
#[test]
fn capture_refusals_and_every_interruption_leave_history_and_usage_untouched() {
    let facts = facts("x".into(), HtmlEncodingUrlParseOutcome::Failure);
    let snapshot = resolve(&facts, &"x".into());
    let mut builder =
        HtmlCaptureBuilder::new(capture_limits(10000, 100), HtmlHostFactsRevision::fresh());
    let doc = document(&mut builder);
    let a = element(&mut builder, &doc, "a", Some("x"));
    let div = element(&mut builder, &doc, "div", None);
    let wrong = element(&mut builder, &doc, "area", Some("y"));
    let mut foreign =
        HtmlCaptureBuilder::new(capture_limits(10000, 100), HtmlHostFactsRevision::fresh());
    let foreign_doc = document(&mut foreign);
    let before = builder.usage();
    for (subject, historical, expected) in [
        (&div, &doc, HtmlCaptureError::NotHtmlHyperlink),
        (&a, &foreign_doc, HtmlCaptureError::ForeignBuilder),
        (&wrong, &doc, HtmlCaptureError::HyperlinkHrefMismatch),
    ] {
        assert_eq!(
            builder.establish_hyperlink_url_with(
                subject,
                Some((historical, snapshot.clone())),
                || Ok::<_, ()>(())
            ),
            Err(HtmlHyperlinkCaptureError::Capture(expected))
        );
        assert_eq!(builder.usage(), before);
    }
    for failure in 1..=4 {
        let mut calls = 0;
        assert_eq!(
            builder.establish_hyperlink_url_with(&a, Some((&doc, snapshot.clone())), || {
                calls += 1;
                if calls == failure {
                    Err(failure)
                } else {
                    Ok(())
                }
            }),
            Err(HtmlHyperlinkCaptureError::Interrupted(failure))
        );
        assert_eq!(calls, failure);
        assert_eq!(builder.usage(), before);
    }
    builder
        .establish_hyperlink_url_with(&a, Some((&doc, snapshot.clone())), || Ok::<_, ()>(()))
        .unwrap();
    let after = builder.usage();
    assert_eq!(
        builder.establish_hyperlink_url_with(&a, Some((&doc, snapshot)), || Ok::<_, ()>(())),
        Err(HtmlHyperlinkCaptureError::Capture(
            HtmlCaptureError::HyperlinkUrlAlreadyEstablished
        ))
    );
    assert_eq!(builder.usage(), after);
}
#[test]
fn capture_admission_counts_actual_snapshot_extent_and_historical_relation() {
    let facts = facts("x".into(), HtmlEncodingUrlParseOutcome::Failure);
    let snapshot = resolve(&facts, &"x".into());
    for relation_short in [false, true] {
        let mut probe =
            HtmlCaptureBuilder::new(capture_limits(10000, 100), HtmlHostFactsRevision::fresh());
        let doc = document(&mut probe);
        element(&mut probe, &doc, "a", Some("x"));
        let usage = probe.usage();
        let mut builder = HtmlCaptureBuilder::new(
            capture_limits(
                usage.utf16_units() + snapshot.retained_utf16_extent()
                    - usize::from(!relation_short),
                usage.relationships() + usize::from(!relation_short),
            ),
            HtmlHostFactsRevision::fresh(),
        );
        let doc = document(&mut builder);
        let a = element(&mut builder, &doc, "a", Some("x"));
        let before = builder.usage();
        assert!(matches!(
            builder.establish_hyperlink_url_with(
                &a,
                Some((&doc, snapshot.clone())),
                || Ok::<_, ()>(())
            ),
            Err(HtmlHyperlinkCaptureError::Capture(
                HtmlCaptureError::LimitExceeded { .. }
            ))
        ));
        assert_eq!(builder.usage(), before);
    }
}

#[test]
fn capture_null_requires_absent_null_namespace_href_and_validates_document_kind() {
    let facts = facts("x".into(), HtmlEncodingUrlParseOutcome::Failure);
    let snapshot = resolve(&facts, &"x".into());
    let mut builder =
        HtmlCaptureBuilder::new(capture_limits(10000, 100), HtmlHostFactsRevision::fresh());
    let doc = document(&mut builder);
    let a = element(&mut builder, &doc, "a", Some("x"));
    let fresh = element(&mut builder, &doc, "area", None);
    let namespaced = builder
        .add_element(
            &doc,
            QualifiedName::new(Namespace::html(), None, "a"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::html(), None, "href"),
                "x",
            )])
            .unwrap(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let before = builder.usage();
    assert_eq!(
        builder.establish_hyperlink_url_with(&a, None, || Ok::<_, ()>(())),
        Err(HtmlHyperlinkCaptureError::Capture(
            HtmlCaptureError::HyperlinkHrefMismatch
        ))
    );
    assert_eq!(
        builder.establish_hyperlink_url_with(
            &fresh,
            Some((&doc, snapshot.clone())),
            || Ok::<_, ()>(())
        ),
        Err(HtmlHyperlinkCaptureError::Capture(
            HtmlCaptureError::HyperlinkHrefMismatch
        ))
    );
    assert!(matches!(
        builder.establish_hyperlink_url_with(&a, Some((&a, snapshot)), || Ok::<_, ()>(())),
        Err(HtmlHyperlinkCaptureError::Capture(
            HtmlCaptureError::WrongKind {
                expected: HtmlCaptureSubjectKind::Document,
                actual: HtmlCaptureSubjectKind::Element
            }
        ))
    ));
    assert_eq!(builder.usage(), before);
    builder
        .establish_hyperlink_url_with(&namespaced, None, || Ok::<_, ()>(()))
        .unwrap();
    builder
        .establish_hyperlink_url_with(&fresh, None, || Ok::<_, ()>(()))
        .unwrap();
}

#[test]
fn chosen_snapshot_retains_only_its_row_and_accepts_present_empty_success() {
    let revision = rev(&HtmlBaseProducerId::fresh(), 1);
    let base = url(&revision, "base");
    let mut result_units = Vec::with_capacity(80);
    result_units.extend("result".encode_utf16());
    let result = HtmlBaseUrlRecord::observed(
        revision.clone(),
        DocumentString::from_utf16(result_units),
        HtmlBaseUrlScheme::Other,
        false,
    )
    .unwrap();
    let result_extent = result.serialization().shared_code_units().capacity();
    let mut observations = Vec::with_capacity(32);
    observations.push(HtmlEncodingUrlParseObservation::new(
        "".into(),
        HtmlEncodingUrlParseOutcome::Parsed(result.clone()),
    ));
    observations.push(HtmlEncodingUrlParseObservation::new(
        "other input".into(),
        HtmlEncodingUrlParseOutcome::Failure,
    ));
    let facts = HtmlEncodingUrlParseFacts::new(
        base.clone(),
        HtmlBaseEncoding::fresh(),
        revision,
        observations,
        limits(),
    )
    .unwrap();
    let snapshot = resolve(&facts, &"".into());
    assert_eq!(snapshot.url(), Some(&result));
    assert_eq!(
        snapshot.retained_utf16_extent(),
        base.serialization().shared_code_units().capacity() + result_extent
    );
    assert!(snapshot.retained_utf16_extent() < facts.formation_usage().utf16_extent());
    assert_eq!(facts.formation_usage().record_capacity(), 32);
}

#[test]
fn snapshot_equality_uses_parse_content_and_identities_not_spare_backing_capacity() {
    let revision = rev(&HtmlBaseProducerId::fresh(), 1);
    let base = url(&revision, "base");
    let encoding = HtmlBaseEncoding::fresh();
    let make = |capacity| {
        let mut raw = Vec::with_capacity(capacity);
        raw.push(120);
        let facts = HtmlEncodingUrlParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            vec![HtmlEncodingUrlParseObservation::new(
                DocumentString::from_utf16(raw),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            limits(),
        )
        .unwrap();
        resolve(&facts, &"x".into())
    };
    let small = make(1);
    let spare = make(80);
    assert!(small.retained_utf16_extent() < spare.retained_utf16_extent());
    assert_eq!(small, spare);
}
