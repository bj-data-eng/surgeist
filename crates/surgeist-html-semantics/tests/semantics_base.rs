use surgeist_html::*;
use surgeist_html_semantics::{
    HtmlAboutBaseUrl, HtmlBaseAllowed, HtmlBaseDocumentFacts, HtmlBaseEncoding, HtmlBaseFactsError,
    HtmlBaseFactsLimits, HtmlBaseFactsRevision, HtmlBaseMissing, HtmlBaseParseObservation,
    HtmlBaseParseOutcome, HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme, HtmlCapture,
    HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureLimits, HtmlCaptureRegistryInput,
    HtmlCapturedBaseDocument, HtmlCapturedBaseFacts, HtmlCapturedFrozenBase, HtmlCapturedIsValue,
    HtmlControlFacts, HtmlControlFactsLimits, HtmlFormAssociation, HtmlFormAssociationState,
    HtmlParseBaseFacts, HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticContext,
    HtmlSemanticContextError, HtmlSemanticContextInput, HtmlSemanticFact, HtmlSemanticFailureKind,
    HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery, HtmlSemanticReport,
    HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticSession, HtmlSemanticValue,
    HtmlTargetOrigin,
};
use surgeist_html_semantics::{HtmlHostFactsRevision, select_frozen_base};

#[test]
fn base_observation_access_preserves_exact_inputs_and_owner_qualification() {
    let producer = HtmlBaseProducerId::fresh();
    let revision = revision(&producer, 1);
    let fallback = url(&revision, "https://example.test/");
    let encoding = HtmlBaseEncoding::fresh();
    let raw = DocumentString::from_utf16([0xd800, 0, 0xdc00]);
    let observation = HtmlBaseParseObservation::new(
        raw.clone(),
        fallback.clone(),
        encoding.clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Failure,
    )
    .unwrap();
    let document = HtmlBaseDocumentFacts::new(
        revision.clone(),
        Some(fallback.clone()),
        HtmlAboutBaseUrl::KnownNull,
        false,
        Some(encoding.clone()),
    )
    .unwrap();
    let make_limits = || HtmlBaseFactsLimits::new(8, 1024, 1024);
    let separate = HtmlParseBaseFacts::new(document.clone(), vec![], make_limits()).unwrap();
    let facts = HtmlParseBaseFacts::new(document, vec![observation], make_limits()).unwrap();
    let clone = facts.clone();
    assert!(facts.same_owner(&clone));
    assert!(!facts.same_owner(&separate));
    let observation = &facts.observations()[0];
    assert_eq!(observation.raw().code_units(), [0xd800, 0, 0xdc00]);
    assert!(observation.inputs_match(&fallback, &encoding, &revision));
    let equal_text = url(&revision, "https://example.test/");
    assert!(!observation.inputs_match(&equal_text, &encoding, &revision));
    assert!(!observation.inputs_match(&fallback, &HtmlBaseEncoding::fresh(), &revision));
    assert!(matches!(
        observation.outcome(),
        HtmlBaseParseOutcome::Failure
    ));
    assert!(std::ptr::eq(
        facts.document().fallback().unwrap(),
        facts.document().document_url().unwrap()
    ));
}

#[test]
fn frozen_base_selection_keeps_scheme_csp_order_and_borrowed_record() {
    let revision = revision(&HtmlBaseProducerId::fresh(), 1);
    let fallback = url(&revision, "https://example.test/fallback/");
    for (scheme, allowed, response, missing) in [
        (
            HtmlBaseUrlScheme::Data,
            HtmlBaseAllowed::Unavailable,
            false,
            false,
        ),
        (
            HtmlBaseUrlScheme::Javascript,
            HtmlBaseAllowed::Unavailable,
            false,
            false,
        ),
        (
            HtmlBaseUrlScheme::Other,
            HtmlBaseAllowed::Blocked,
            false,
            false,
        ),
        (
            HtmlBaseUrlScheme::Other,
            HtmlBaseAllowed::Allowed,
            true,
            false,
        ),
        (
            HtmlBaseUrlScheme::Other,
            HtmlBaseAllowed::Unavailable,
            false,
            true,
        ),
    ] {
        let serialization = match scheme {
            HtmlBaseUrlScheme::Data => "data:text/plain,x",
            HtmlBaseUrlScheme::Javascript => "javascript:void(0)",
            HtmlBaseUrlScheme::Other => "https://example.test/parsed/",
        };
        let record =
            HtmlBaseUrlRecord::observed(revision.clone(), serialization.into(), scheme, false)
                .unwrap();
        let outcome = HtmlBaseParseOutcome::Parsed {
            record,
            base_allowed: allowed,
        };
        let selected = select_frozen_base(&fallback, &outcome);
        if missing {
            assert_eq!(selected.unwrap_err(), HtmlBaseMissing::Csp);
        } else {
            let (selected, actual_response) = selected.unwrap();
            assert_eq!(actual_response, response);
            let expected = if response {
                let HtmlBaseParseOutcome::Parsed { record, .. } = &outcome else {
                    unreachable!()
                };
                record
            } else {
                &fallback
            };
            assert!(std::ptr::eq(selected, expected));
        }
    }
    let failed = HtmlBaseParseOutcome::Failure;
    let (selected, response) = select_frozen_base(&fallback, &failed).unwrap();
    assert!(std::ptr::eq(selected, &fallback));
    assert!(!response);
}

#[test]
fn document_fallback_selects_srcdoc_and_about_blank_facts_lazily() {
    let revision = revision(&HtmlBaseProducerId::fresh(), 1);
    let about = url(&revision, "https://example.test/about/");
    let srcdoc = HtmlBaseDocumentFacts::new(
        revision.clone(),
        None,
        HtmlAboutBaseUrl::KnownRecord(about.clone()),
        true,
        None,
    )
    .unwrap();
    assert_eq!(srcdoc.fallback().unwrap(), &about);
    let ordinary = url(&revision, "https://example.test/ordinary/");
    let document = HtmlBaseDocumentFacts::new(
        revision.clone(),
        Some(ordinary.clone()),
        HtmlAboutBaseUrl::Unavailable,
        false,
        None,
    )
    .unwrap();
    assert_eq!(document.fallback().unwrap(), &ordinary);
    let blank = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    let document = HtmlBaseDocumentFacts::new(
        revision,
        Some(blank),
        HtmlAboutBaseUrl::Unavailable,
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        document.fallback().unwrap_err(),
        HtmlBaseMissing::AboutBaseUrl
    );
}

fn attr(name: &str, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, name), value)
}
fn builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (builder, document)
}
fn element(
    builder: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    name: &str,
    attrs: Vec<Attribute>,
) -> HtmlCaptureBuilderSubject {
    builder
        .add_element(
            document,
            QualifiedName::new(Namespace::html(), None, name),
            OrderedAttributes::new(attrs).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}
fn context(capture: &HtmlCapture, facts: Option<HtmlCapturedBaseFacts>) -> HtmlSemanticContext {
    let mut input = HtmlSemanticContextInput::new(vec![]);
    if let Some(facts) = facts {
        input = input.with_base_facts(facts.revision().clone(), Some(facts));
    }
    HtmlSemanticContext::new(capture, capture.host_facts_revision().clone(), input, 0).unwrap()
}
fn query(
    capture: &HtmlCapture,
    context: &HtmlSemanticContext,
    requests: &[HtmlSemanticRequest],
) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        capture,
        context,
        HtmlValueInterpreter::new(100000),
        HtmlSemanticLimits::new(1000000, 100, 1000, 1000, 1000000, 10000),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(requests)
    .unwrap()
}
fn revision(producer: &HtmlBaseProducerId, n: u64) -> HtmlBaseFactsRevision {
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

#[test]
fn captured_facts_keep_capture_host_and_configuration_qualifications_distinct() {
    let (builder_a, _) = builder();
    let capture_a = builder_a.finish().unwrap();
    let (builder_b, _) = builder();
    let capture_b = builder_b.finish().unwrap();
    let producer = HtmlBaseProducerId::fresh();
    let current = revision(&producer, 1);
    assert_eq!(
        HtmlCapturedBaseFacts::new(
            &capture_a,
            capture_b.host_facts_revision().clone(),
            current.clone(),
            vec![],
            vec![],
            HtmlBaseFactsLimits::new(0, 0, 0),
        )
        .unwrap_err(),
        HtmlBaseFactsError::HostRevisionMismatch
    );
    let facts = HtmlCapturedBaseFacts::new(
        &capture_a,
        capture_a.host_facts_revision().clone(),
        current.clone(),
        vec![],
        vec![],
        HtmlBaseFactsLimits::new(0, 0, 0),
    )
    .unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &capture_b,
            capture_b.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_base_facts(current.clone(), Some(facts)),
            0,
        )
        .unwrap_err(),
        HtmlSemanticContextError::BaseCaptureMismatch
    );
    let facts = HtmlCapturedBaseFacts::new(
        &capture_a,
        capture_a.host_facts_revision().clone(),
        current,
        vec![],
        vec![],
        HtmlBaseFactsLimits::new(0, 0, 0),
    )
    .unwrap();
    assert_eq!(
        HtmlSemanticContext::new(
            &capture_a,
            capture_a.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![])
                .with_base_facts(revision(&producer, 2), Some(facts)),
            0,
        )
        .unwrap_err(),
        HtmlSemanticContextError::BaseRevisionMismatch
    );
}

#[test]
fn no_href_selects_document_fallback_but_the_getter_parses_its_empty_input() {
    let (mut builder, document) = builder();
    let base = element(&mut builder, &document, "base", vec![attr("target", "T")]);
    builder.establish_children(&base, &[]).unwrap();
    builder
        .establish_children(&document, std::slice::from_ref(&base))
        .unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let base = capture.subject_id(&base).unwrap();
    let producer = HtmlBaseProducerId::fresh();
    let revision = revision(&producer, 1);
    let blank = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "about:blank?x#y".into(),
        HtmlBaseUrlScheme::Other,
        true,
    )
    .unwrap();
    let fallback = url(&revision, "https://creator.test/fallback/");
    let parsed = url(&revision, "https://creator.test/empty-getter");
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(blank),
            HtmlAboutBaseUrl::KnownRecord(fallback.clone()),
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                DocumentString::default(),
                fallback.clone(),
                encoding,
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: parsed,
                    base_allowed: HtmlBaseAllowed::Unavailable,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(10, 1000, 10000),
    )
    .unwrap();
    let facts = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![HtmlCapturedBaseDocument::new(document.clone(), facts)],
        vec![],
        HtmlBaseFactsLimits::new(10, 0, 10),
    )
    .unwrap();
    let context = context(&capture, Some(facts));
    let report = query(
        &capture,
        &context,
        &[
            HtmlSemanticRequest::new(
                document.subject().clone(),
                HtmlSemanticQuery::DocumentBaseUrl,
            ),
            HtmlSemanticRequest::new(base, HtmlSemanticQuery::BaseHref),
        ],
    );
    match report.results()[0].fact() {
        HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Known(value)) => {
            assert_eq!(value.value(), &fallback);
            assert_eq!(value.base_element(), None);
            assert_eq!(value.frozen_revision(), None);
        }
        other => panic!("{other:?}"),
    }
    match report.results()[1].fact() {
        HtmlSemanticFact::BaseHref(HtmlSemanticValue::Known(value)) => {
            assert!(value.raw().is_empty());
            assert_eq!(
                value.value(),
                &DocumentString::from("https://creator.test/empty-getter")
            );
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(report.qualification().base_revision(), Some(&revision));
}

#[test]
fn captured_first_href_is_historical_while_getter_uses_current_parse_and_target_is_independent() {
    let (mut builder, document) = builder();
    let target = element(&mut builder, &document, "base", vec![attr("target", "T")]);
    let first = element(&mut builder, &document, "base", vec![attr("href", "raw")]);
    let later = element(
        &mut builder,
        &document,
        "base",
        vec![attr("href", "later"), attr("target", "U")],
    );
    let anchor = element(&mut builder, &document, "a", vec![]);
    for leaf in [&target, &first, &later, &anchor] {
        builder.establish_children(leaf, &[]).unwrap();
    }
    builder
        .establish_children(
            &document,
            &[target.clone(), first.clone(), later, anchor.clone()],
        )
        .unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let first = capture.subject_id(&first).unwrap();
    let producer = HtmlBaseProducerId::fresh();
    let old = revision(&producer, 1);
    let current = revision(&producer, 2);
    let old_url = url(&old, "https://old.test/frozen");
    let fallback = url(&current, "https://new.test/fallback/");
    let parsed = url(&current, "https://new.test/current-getter");
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            current.clone(),
            Some(fallback.clone()),
            HtmlAboutBaseUrl::Unavailable,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "raw".into(),
                fallback,
                encoding,
                current.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: parsed,
                    base_allowed: HtmlBaseAllowed::Blocked,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(10, 1000, 10000),
    )
    .unwrap();
    let facts = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        current,
        vec![HtmlCapturedBaseDocument::new(document.clone(), facts)],
        vec![HtmlCapturedFrozenBase::new(
            first.clone(),
            document.clone(),
            "raw".into(),
            Some(old_url),
            old.clone(),
        )],
        HtmlBaseFactsLimits::new(10, 1000, 10000),
    )
    .unwrap();
    let context = context(&capture, Some(facts));
    let report = query(
        &capture,
        &context,
        &[
            HtmlSemanticRequest::new(
                document.subject().clone(),
                HtmlSemanticQuery::DocumentBaseUrl,
            ),
            HtmlSemanticRequest::new(first.clone(), HtmlSemanticQuery::BaseHref),
            HtmlSemanticRequest::new(
                capture.subject_id(&anchor).unwrap(),
                HtmlSemanticQuery::ElementTarget,
            ),
        ],
    );
    match report.results()[0].fact() {
        HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Known(value)) => {
            assert_eq!(
                value.value().serialization().code_units(),
                "https://old.test/frozen".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(value.base_element(), Some(&first));
            assert_eq!(value.frozen_revision(), Some(&old));
        }
        other => panic!("{other:?}"),
    }
    match report.results()[1].fact() {
        HtmlSemanticFact::BaseHref(HtmlSemanticValue::Known(value)) => {
            assert_eq!(value.raw().code_units(), &[114, 97, 119]);
            assert_eq!(
                value.value().code_units(),
                "https://new.test/current-getter"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
        }
        other => panic!("{other:?}"),
    }
    match report.results()[2].fact() {
        HtmlSemanticFact::ElementTarget(HtmlSemanticValue::Known(value)) => {
            assert_eq!(value.value().unwrap().code_units(), &[84]);
            assert_eq!(
                value.origin(),
                &HtmlTargetOrigin::DocumentBase(capture.subject_id(&target).unwrap())
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn explicit_empty_own_empty_and_dangerous_target_recovery_do_not_require_url_or_tree_facts() {
    for (raw, expected) in [
        (vec![], vec![]),
        (vec![9, 60], vec![95, 98, 108, 97, 110, 107]),
        (vec![10, 60], vec![95, 98, 108, 97, 110, 107]),
        (vec![13, 60], vec![95, 98, 108, 97, 110, 107]),
        (vec![12, 60], vec![12, 60]),
        (vec![0xd800, 60], vec![0xd800, 60]),
    ] {
        let (mut builder, document) = builder();
        let anchor = element(
            &mut builder,
            &document,
            "a",
            vec![attr("target", DocumentString::from_utf16(raw.clone()))],
        );
        let capture = builder.finish().unwrap();
        let id = capture.subject_id(&anchor).unwrap();
        let context = context(&capture, None);
        let report = query(
            &capture,
            &context,
            &[
                HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::ElementTarget),
                HtmlSemanticRequest::element_target(id, "".into()),
            ],
        );
        for (ordinal, (expected, origin)) in [
            (expected.as_slice(), HtmlTargetOrigin::OwnAttribute),
            (&[][..], HtmlTargetOrigin::ExplicitOverride),
        ]
        .into_iter()
        .enumerate()
        {
            match report.results()[ordinal].fact() {
                HtmlSemanticFact::ElementTarget(HtmlSemanticValue::Known(value)) => {
                    assert_eq!(value.value().unwrap().code_units(), expected);
                    assert_eq!(value.origin(), &origin);
                }
                other => panic!("{other:?}"),
            }
        }
    }
}

#[test]
fn unresolved_first_href_cannot_skip_to_a_later_qualified_frozen_base() {
    let (mut builder, document) = builder();
    let first = element(&mut builder, &document, "base", vec![attr("href", "")]);
    let later = element(&mut builder, &document, "base", vec![attr("href", "later")]);
    builder.establish_children(&first, &[]).unwrap();
    builder.establish_children(&later, &[]).unwrap();
    builder
        .establish_children(&document, &[first.clone(), later.clone()])
        .unwrap();
    let capture = builder.finish().unwrap();
    let document = capture
        .document_id(&capture.subject_id(&document).unwrap())
        .unwrap();
    let revision = revision(&HtmlBaseProducerId::fresh(), 1);
    let facts = HtmlCapturedBaseFacts::new(
        &capture,
        capture.host_facts_revision().clone(),
        revision.clone(),
        vec![],
        vec![HtmlCapturedFrozenBase::new(
            capture.subject_id(&later).unwrap(),
            document.clone(),
            "later".into(),
            Some(url(&revision, "https://later.test/")),
            revision,
        )],
        HtmlBaseFactsLimits::new(10, 1000, 10000),
    )
    .unwrap();
    let context = context(&capture, Some(facts));
    let report = query(
        &capture,
        &context,
        &[HtmlSemanticRequest::new(
            document.subject().clone(),
            HtmlSemanticQuery::DocumentBaseUrl,
        )],
    );
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Missing(HtmlSemanticMissing::Base {
            subject: capture.subject_id(&first).unwrap(),
            fact: HtmlBaseMissing::FrozenBase
        }))
    );
}

#[test]
fn submission_target_uses_actual_form_owner_and_only_submitter_override() {
    for (kind, override_value, has_owner, expected, origin) in [
        (
            "submit",
            Some(""),
            true,
            Some(""),
            HtmlTargetOrigin::ExplicitOverride,
        ),
        (
            "reset",
            Some("Z"),
            true,
            Some("F"),
            HtmlTargetOrigin::OwnAttribute,
        ),
        (
            "submit",
            None,
            true,
            Some("F"),
            HtmlTargetOrigin::OwnAttribute,
        ),
        (
            "submit",
            Some("Z"),
            false,
            None,
            HtmlTargetOrigin::NoFormOwner,
        ),
    ] {
        let (mut builder, document) = builder();
        let form = element(&mut builder, &document, "form", vec![attr("target", "F")]);
        let mut attributes = vec![attr("type", kind)];
        if let Some(value) = override_value {
            attributes.push(attr("formtarget", value));
        }
        let button = element(&mut builder, &document, "button", attributes);
        let capture = builder.finish().unwrap();
        let button = capture.subject_id(&button).unwrap();
        let owner = has_owner.then(|| capture.subject_id(&form).unwrap());
        let controls = HtmlControlFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            vec![HtmlFormAssociation::new(
                button.clone(),
                HtmlFormAssociationState::EstablishedCurrent(owner),
            )],
            vec![],
            vec![],
            vec![],
            HtmlControlFactsLimits::new(10, 100000),
        )
        .unwrap();
        let context = HtmlSemanticContext::new(
            &capture,
            capture.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_controls(controls),
            0,
        )
        .unwrap();
        let report = query(
            &capture,
            &context,
            &[HtmlSemanticRequest::new(
                button,
                HtmlSemanticQuery::FormSubmissionTarget,
            )],
        );
        match report.results()[0].fact() {
            HtmlSemanticFact::FormSubmissionTarget(HtmlSemanticValue::Known(value)) => {
                assert_eq!(
                    value.value().map(DocumentString::code_units),
                    expected
                        .map(|s| s.encode_utf16().collect::<Vec<_>>())
                        .as_deref()
                );
                assert_eq!(value.origin(), &origin);
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn ordered_topology_before_first_base_is_required_but_later_or_first_children_are_not() {
    for earlier in [true, false] {
        let (mut builder, document) = builder();
        let unknown = element(&mut builder, &document, "div", vec![]);
        let base = element(&mut builder, &document, "base", vec![attr("href", "")]);
        let children = if earlier {
            [unknown.clone(), base.clone()]
        } else {
            [base.clone(), unknown.clone()]
        };
        builder.establish_children(&document, &children).unwrap();
        let capture = builder.finish().unwrap();
        let document = capture
            .document_id(&capture.subject_id(&document).unwrap())
            .unwrap();
        let revision = revision(&HtmlBaseProducerId::fresh(), 1);
        let facts = HtmlCapturedBaseFacts::new(
            &capture,
            capture.host_facts_revision().clone(),
            revision.clone(),
            vec![],
            vec![HtmlCapturedFrozenBase::new(
                capture.subject_id(&base).unwrap(),
                document.clone(),
                "".into(),
                Some(url(&revision, "https://first.test/")),
                revision,
            )],
            HtmlBaseFactsLimits::new(10, 1000, 10000),
        )
        .unwrap();
        let context = context(&capture, Some(facts));
        let report = query(
            &capture,
            &context,
            &[HtmlSemanticRequest::new(
                document.subject().clone(),
                HtmlSemanticQuery::DocumentBaseUrl,
            )],
        );
        if earlier {
            assert_eq!(
                report.results()[0].fact(),
                &HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Missing(
                    HtmlSemanticMissing::OrderedChildren(capture.subject_id(&unknown).unwrap())
                ))
            );
        } else {
            assert!(
                matches!(report.results()[0].fact(), HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Known(value)) if value.base_element() == Some(&capture.subject_id(&base).unwrap()) && value.value().serialization().code_units() == "https://first.test/".encode_utf16().collect::<Vec<_>>())
            );
        }
    }
}

#[test]
fn recovered_target_has_six_real_output_units_with_exact_limit_and_original_cancellation() {
    let (mut builder, document) = builder();
    let anchor = element(&mut builder, &document, "a", vec![attr("target", "\t<")]);
    let capture = builder.finish().unwrap();
    let context = context(&capture, None);
    let request = HtmlSemanticRequest::new(
        capture.subject_id(&anchor).unwrap(),
        HtmlSemanticQuery::ElementTarget,
    );
    for cap in [5, 6] {
        let cancellation = HtmlSemanticCancellation::new();
        let mut session = HtmlSemanticSession::new(
            &capture,
            &context,
            HtmlValueInterpreter::new(1000),
            HtmlSemanticLimits::new(100000, 1, 0, 0, 100000, cap),
            cancellation.clone(),
        )
        .unwrap();
        let result = session.query(std::slice::from_ref(&request));
        if cap == 5 {
            let failure = result.unwrap_err();
            assert_eq!(
                failure.kind(),
                &HtmlSemanticFailureKind::LimitExceeded {
                    resource: HtmlSemanticResource::NestedOutputRecords,
                    limit: 5,
                    required: 6
                }
            );
            assert_eq!(failure.usage().nested_output_records(), 0);
            cancellation.cancel();
            assert_eq!(
                session.query(std::slice::from_ref(&request)).unwrap_err(),
                failure
            );
        } else {
            let report = result.unwrap();
            let usage = report.usage();
            cancellation.cancel();
            assert_eq!(report.usage(), usage);
            assert_eq!(report.usage().nested_output_records(), 6);
            assert!(
                matches!(report.results()[0].fact(), HtmlSemanticFact::ElementTarget(HtmlSemanticValue::Known(value)) if value.raw().unwrap().code_units() == [9,60] && value.value().unwrap().code_units() == [95,98,108,97,110,107])
            );
            assert_eq!(
                session
                    .query(std::slice::from_ref(&request))
                    .unwrap_err()
                    .kind(),
                &HtmlSemanticFailureKind::Cancelled
            );
        }
    }
}
