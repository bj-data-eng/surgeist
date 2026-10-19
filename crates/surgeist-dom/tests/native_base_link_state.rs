//! Qualified external outcomes through the real native Base owner.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(records: usize, work: u64, units: usize) -> DomLimits {
    DomLimits::new(
        128,
        units,
        512,
        256,
        128,
        8,
        262144,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_base_processing(records, work)
    .unwrap()
}
fn apply(dom: &mut Dom, b: DomBatch) -> DomCommit {
    dom.apply(b, &cancel()).unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(limits(128, 1_000_000, 131072)).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    let base = b.create_element_ns(&d, Namespace::html(), "base");
    b.insert_before(&root, &d, None);
    let c = apply(&mut dom, b);
    (
        dom,
        c.document(&d).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&base).unwrap().unwrap(),
    )
}
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn record(
    r: &HtmlBaseFactsRevision,
    text: &str,
    scheme: HtmlBaseUrlScheme,
    blank: bool,
    fragment_delimiter_offset: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        r.clone(),
        text.into(),
        scheme,
        blank,
        fragment_delimiter_offset,
    )
    .unwrap()
}
fn facts(
    r: &HtmlBaseFactsRevision,
    raw: &str,
    outcome: HtmlBaseParseOutcome,
) -> (HtmlParseBaseFacts, HtmlBaseUrlRecord) {
    let fallback = record(r, "about:blank", HtmlBaseUrlScheme::Other, true, None);
    let encoding = HtmlBaseEncoding::fresh();
    let observation = HtmlBaseParseObservation::new(
        raw.into(),
        fallback.clone(),
        encoding.clone(),
        r.clone(),
        outcome,
    )
    .unwrap();
    let document = HtmlBaseDocumentFacts::new(
        r.clone(),
        Some(fallback.clone()),
        HtmlAboutBaseUrl::KnownNull,
        false,
        Some(encoding),
    )
    .unwrap();
    (
        HtmlParseBaseFacts::new(
            document,
            vec![observation],
            HtmlBaseFactsLimits::new(128, 65536, 1_000_000),
        )
        .unwrap(),
        fallback,
    )
}
fn bind(dom: &mut Dom, doc: DocumentId, facts: &HtmlParseBaseFacts) {
    let mut b = DomBatch::new();
    b.bind_base_facts(dom.snapshot().document(doc).unwrap().stamp(), facts);
    apply(dom, b);
}
fn freeze(dom: &mut Dom, base: NodeId, parent: NodeId, raw: &str) -> DomCommit {
    let mut b = DomBatch::new();
    b.set_attribute(base, "href", raw);
    b.insert_before(base, parent, None);
    apply(dom, b)
}

#[test]
fn actual_failure_forbidden_schemes_and_csp_block_select_fallback_without_response() {
    for branch in 0..4 {
        let (mut dom, doc, parent, base) = fixture();
        let r = revision();
        let outcome = match branch {
            0 => HtmlBaseParseOutcome::Failure,
            1 => HtmlBaseParseOutcome::Parsed {
                record: record(
                    &r,
                    "data:text/plain,x",
                    HtmlBaseUrlScheme::Data,
                    false,
                    None,
                ),
                base_allowed: HtmlBaseAllowed::Allowed,
            },
            2 => HtmlBaseParseOutcome::Parsed {
                record: record(
                    &r,
                    "javascript:void 0",
                    HtmlBaseUrlScheme::Javascript,
                    false,
                    None,
                ),
                base_allowed: HtmlBaseAllowed::Unavailable,
            },
            _ => HtmlBaseParseOutcome::Parsed {
                record: record(
                    &r,
                    "https://blocked.test/",
                    HtmlBaseUrlScheme::Other,
                    false,
                    None,
                ),
                base_allowed: HtmlBaseAllowed::Blocked,
            },
        };
        let (facts, fallback) = facts(&r, "target", outcome);
        bind(&mut dom, doc, &facts);
        let c = freeze(&mut dom, base, parent, "target");
        let snapshot = dom.snapshot();
        let frozen = snapshot
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap();
        assert_eq!(frozen.value(), &fallback);
        assert_eq!(frozen.response(), DomBaseResponse::Returned);
        assert_eq!(frozen.document(), doc);
        assert_eq!(frozen.revision(), &r);
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::DocumentBase(doc))
        );
    }
}

#[test]
fn positive_native_response_reads_actual_script_results_and_identical_refreeze_is_noop() {
    let (mut dom, doc, parent, base) = fixture();
    let r = revision();
    let selected = record(
        &r,
        "https://allowed.test/a",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (facts, _) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: selected.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    bind(&mut dom, doc, &facts);
    let mut b = DomBatch::new();
    let host = b.create_element_ns(doc, Namespace::html(), "div");
    let shadow = b.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let script = b.create_element_ns(doc, Namespace::html(), "script");
    let nested = b.create_element_ns(doc, Namespace::html(), "script");
    for n in [&script, &nested] {
        b.set_attribute(n, "type", "speculationrules");
    }
    b.insert_before(&nested, &shadow, None);
    b.insert_before(&script, parent, None);
    b.insert_before(&host, parent, None);
    let c = apply(&mut dom, b);
    for n in [script, nested] {
        assert_eq!(
            dom.snapshot()
                .node(c.created(&n).unwrap().unwrap())
                .unwrap()
                .as_html_script()
                .unwrap()
                .result(),
            surgeist_html::HtmlScriptResult::Uninitialized
        );
    }
    let c = freeze(&mut dom, base, parent, "target");
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::DocumentBase(doc))
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &selected
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .response(),
        DomBaseResponse::FactsPublished
    );
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(base, "href", "target");
    let c = apply(&mut dom, b);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(c.changes().changes().is_empty());
}

#[test]
fn missing_csp_and_missing_observation_refuse_after_staging_without_partial_freeze() {
    for available in [false, true] {
        let (mut dom, doc, parent, base) = fixture();
        let r = revision();
        let outcome = HtmlBaseParseOutcome::Parsed {
            record: record(
                &r,
                "https://unknown.test/",
                HtmlBaseUrlScheme::Other,
                false,
                None,
            ),
            base_allowed: HtmlBaseAllowed::Unavailable,
        };
        let (facts, _) = facts(&r, if available { "target" } else { "other" }, outcome);
        bind(&mut dom, doc, &facts);
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        b.set_attribute(base, "href", "target");
        b.insert_before(base, parent, None);
        assert_eq!(
            dom.apply(b, &cancel()).unwrap_err(),
            DomError::BaseFacts(DomBaseFactsError::Missing(if available {
                HtmlBaseMissing::Csp
            } else {
                HtmlBaseMissing::ParseObservation
            }))
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(
            dom.snapshot()
                .node(base)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .is_none()
        );
        assert_eq!(
            dom.snapshot().document(doc).unwrap().first_base().unwrap(),
            None
        );
    }
}

#[test]
fn environment_and_identity_qualification_refuse_mismatches_without_replacing_binding() {
    let (mut dom, doc, _, _) = fixture();
    let r = revision();
    let (original, _) = facts(&r, "x", HtmlBaseParseOutcome::Failure);
    bind(&mut dom, doc, &original);
    // Same full revision and spelling cannot substitute a different record or encoding.
    let (substitute, _) = facts(&r, "x", HtmlBaseParseOutcome::Failure);
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &substitute);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::BaseFacts(DomBaseFactsError::InputsMismatch)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let (foreign, _) = facts(&revision(), "x", HtmlBaseParseOutcome::Failure);
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &foreign);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::BaseFacts(DomBaseFactsError::ForeignProducer)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let wrong = HtmlBaseDocumentFacts::new(
        r.clone(),
        Some(record(
            &r,
            "https://wrong.test/",
            HtmlBaseUrlScheme::Other,
            false,
            None,
        )),
        HtmlAboutBaseUrl::KnownNull,
        false,
        Some(HtmlBaseEncoding::fresh()),
    )
    .unwrap();
    let wrong =
        HtmlParseBaseFacts::new(wrong, vec![], HtmlBaseFactsLimits::new(0, 1024, 1024)).unwrap();
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &wrong);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::BaseFacts(DomBaseFactsError::EnvironmentMismatch)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let env = original.document();
    let bad = HtmlBaseParseObservation::new(
        "x".into(),
        env.fallback().unwrap().clone(),
        HtmlBaseEncoding::fresh(),
        r.clone(),
        HtmlBaseParseOutcome::Failure,
    )
    .unwrap();
    let mismatched = HtmlParseBaseFacts::new(
        env.clone(),
        vec![bad],
        HtmlBaseFactsLimits::new(16, 4096, 4096),
    )
    .unwrap();
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &mismatched);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::BaseFacts(DomBaseFactsError::InputsMismatch)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        dom.snapshot()
            .document(doc)
            .unwrap()
            .base_facts()
            .unwrap()
            .same_owner(&original)
    );
}

#[test]
fn aggregate_actual_observation_capacity_and_retained_units_have_exact_boundaries() {
    let (mut dom, doc, parent, base) = fixture();
    let r = revision();
    let selected = record(
        &r,
        "https://allowed.test/a",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (facts, _) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: selected.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    let records = facts.formation_usage().record_capacity();
    assert!(records > 0);
    let before = dom.snapshot();
    dom.set_limits(limits(records - 1, 1_000_000, 131072))
        .unwrap();
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &facts);
    assert!(matches!(
        dom.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::BaseObservations,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(limits(records, 1_000_000, 131072)).unwrap();
    bind(&mut dom, doc, &facts);
    let expected =
        dom.usage().string_code_units() + 4 + "target".len() + selected.serialization().len();
    dom.set_limits(limits(records, 1_000_000, expected - 1))
        .unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(base, "href", "target");
    b.insert_before(base, parent, None);
    assert!(matches!(
        dom.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(limits(records, 1_000_000, expected))
        .unwrap();
    freeze(&mut dom, base, parent, "target");
    assert_eq!(dom.usage().string_code_units(), expected);
    dom.set_limits(limits(records, 1_000_000, 131072)).unwrap();
    let mut b = DomBatch::new();
    let other = b.create_document();
    let c = apply(&mut dom, b);
    let other = c.document(&other).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(other).unwrap().stamp(), &facts);
    assert!(matches!(
        dom.apply(b, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::BaseObservations,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn new_qualified_observations_preserve_frozen_history_until_an_actual_refreeze() {
    let (mut dom, doc, parent, base) = fixture();
    let r = revision();
    let first = record(
        &r,
        "https://first.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (old, _) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: first.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    bind(&mut dom, doc, &old);
    freeze(&mut dom, base, parent, "target");
    let before = dom.snapshot();
    let next = HtmlBaseFactsRevision::new(r.producer().clone(), 4, 5, 6);
    let second = record(
        &next,
        "https://second.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (new, _) = facts(
        &next,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: second.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    let mut b = DomBatch::new();
    b.bind_base_facts(before.document(doc).unwrap().stamp(), &new);
    let c = apply(&mut dom, b);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::DocumentBase(doc))
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &first
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .revision(),
        &r
    );
    let mut b = DomBatch::new();
    b.set_attribute(base, "href", "target");
    let c = apply(&mut dom, b);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::DocumentBase(doc))
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &second
    );
    assert_eq!(
        before
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &first
    );
}

#[test]
fn clone_starts_unfrozen_then_real_removal_reveals_it_using_current_facts_and_reuse_is_fresh() {
    let (mut dom, doc, parent, base) = fixture();
    let r = revision();
    let first = record(
        &r,
        "https://old.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (old, _) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: first.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    bind(&mut dom, doc, &old);
    freeze(&mut dom, base, parent, "target");
    let historical = dom.snapshot();
    let next = HtmlBaseFactsRevision::new(r.producer().clone(), 4, 5, 6);
    let current = record(
        &next,
        "https://current.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (new, _) = facts(
        &next,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: current.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    bind(&mut dom, doc, &new);
    let mut b = DomBatch::new();
    let clone = b.import_node(&historical, base, doc, true);
    b.insert_before(&clone, parent, None);
    let c = apply(&mut dom, b);
    let clone = c.created(&clone).unwrap().unwrap();
    assert!(
        dom.snapshot()
            .node(clone)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .is_none()
    );
    let mut b = DomBatch::new();
    b.detach(base);
    let c = apply(&mut dom, b);
    assert!(
        c.changes()
            .changes()
            .contains(&DomChange::DocumentBase(doc))
    );
    assert_eq!(
        dom.snapshot().document(doc).unwrap().first_base().unwrap(),
        Some(clone)
    );
    assert_eq!(
        dom.snapshot()
            .node(clone)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &current
    );
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &first
    );
    let mut b = DomBatch::new();
    b.destroy(base);
    let fresh = b.create_element_ns(doc, Namespace::html(), "base");
    let c = apply(&mut dom, b);
    let fresh = c.created(&fresh).unwrap().unwrap();
    assert_ne!(fresh, base);
    assert!(matches!(
        dom.snapshot().node(base),
        Err(DomError::StaleNode)
    ));
    assert!(
        dom.snapshot()
            .node(fresh)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .is_none()
    );
    assert!(historical.node(base).is_ok());
}

#[test]
fn native_immutable_installation_runs_fresh_base_and_link_hooks_atomically() {
    use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
    let (mut dom, doc, parent, _) = fixture();
    let r = revision();
    let uri = record(
        &r,
        "https://installed.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (facts, _) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: uri.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    );
    bind(&mut dom, doc, &facts);
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(8, 16, 8, 1024, 8)).unwrap();
    let root = builder.root();
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "base"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "href"),
                "target",
            )])
            .unwrap(),
        )
        .unwrap();
    builder
        .append_element(
            &root,
            QualifiedName::new(Namespace::html(), None, "link"),
            OrderedAttributes::new([Attribute::new(
                QualifiedName::new(Namespace::none(), None, "disabled"),
                "",
            )])
            .unwrap(),
        )
        .unwrap();
    let source = builder.finish();
    let source_ids: Vec<_> = source.root().children().collect();
    let report = dom
        .install_native_html_fragment(
            dom.snapshot().document(doc).unwrap().stamp(),
            parent,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(32, 100000, 131072),
            &cancel(),
        )
        .unwrap();
    let installed = report.node_for(&source_ids[0]).unwrap();
    let link = report.node_for(&source_ids[1]).unwrap();
    assert_eq!(
        report
            .snapshot()
            .node(installed)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &uri
    );
    assert!(
        !report
            .snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
    assert_eq!(
        report
            .snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .parser_document(),
        None
    );
    let mut b = DomBatch::new();
    b.remove_attribute(link, "disabled");
    apply(&mut dom, b);
    assert!(
        dom.snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
    assert!(
        !report
            .snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
}

#[test]
fn native_link_trigger_inputs_remain_qualified_without_advertising_resource_models() {
    let (mut dom, doc, parent, _) = fixture();
    let mut b = DomBatch::new();
    let link = b.create_element_ns(doc, Namespace::html(), "link");
    b.set_attribute(&link,"rel","stylesheet modulepreload preload prefetch preconnect dns-prefetch expect icon manifest alternate search pingback");
    b.set_attribute(&link, "href", "unobserved://raw");
    b.set_attribute(&link, "disabled", "");
    b.insert_before(&link, parent, None);
    let c = apply(&mut dom, b);
    let link = c.created(&link).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .document(doc)
            .unwrap()
            .metadata()
            .document_environment()
            .browsing_context(),
        surgeist_html::HtmlBrowsingContextPresence::Absent
    );
    let mut b = DomBatch::new();
    b.set_attribute(link, "href", "another raw value");
    b.set_attribute(link, "id", "indicated");
    b.remove_attribute(link, "disabled");
    b.set_attribute(link, "disabled", "");
    b.set_attribute(parent, "name", "indicated");
    apply(&mut dom, b);
    assert!(
        dom.snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
    assert_eq!(dom.pending_native_tasks(), 0);
    // Actual native guard inputs and historical flag state are asserted here;
    // no stylesheet/fetch/render-blocked/service delivery result is invented.
}

#[test]
fn capture_rejects_differing_actual_document_revisions_in_the_producer_single_revision_domain() {
    let (mut dom, doc, parent, _) = fixture();
    let r = revision();
    let (one, _) = facts(&r, "unused", HtmlBaseParseOutcome::Failure);
    bind(&mut dom, doc, &one);
    let mut b = DomBatch::new();
    let template = b.create_element_ns(doc, Namespace::html(), "template");
    b.insert_before(&template, parent, None);
    let other = b.create_document();
    let c = apply(&mut dom, b);
    let template = c.created(&template).unwrap().unwrap();
    let other = c.document(&other).unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let mut b = DomBatch::new();
    b.adopt(contents, other);
    apply(&mut dom, b);
    let (two, _) = facts(&revision(), "unused", HtmlBaseParseOutcome::Failure);
    bind(&mut dom, other, &two);
    let before = dom.snapshot();
    assert!(matches!(
        before.html_capture(
            doc,
            None,
            HtmlCaptureLimits::new(256, 1024, 256, 65536, 256, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(256, 1_000_000).with_base_facts(256, 65536, 1_000_000),
            &cancel()
        ),
        Err(DomHtmlCaptureError::BaseFacts(
            HtmlBaseFactsError::RevisionMismatch
        ))
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn csp_blocked_frozen_base_does_not_replace_the_independent_semantic_href_parse() {
    let (mut dom, doc, parent, base) = fixture();
    let r = revision();
    let parsed = record(
        &r,
        "https://blocked.test/",
        HtmlBaseUrlScheme::Other,
        false,
        None,
    );
    let (facts, fallback) = facts(
        &r,
        "target",
        HtmlBaseParseOutcome::Parsed {
            record: parsed.clone(),
            base_allowed: HtmlBaseAllowed::Blocked,
        },
    );
    bind(&mut dom, doc, &facts);
    freeze(&mut dom, base, parent, "target");
    assert_eq!(
        dom.snapshot()
            .node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &fallback
    );
    let capture = dom
        .snapshot()
        .html_capture(
            doc,
            None,
            HtmlCaptureLimits::new(128, 512, 128, 65536, 128, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000).with_base_facts(128, 65536, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(base).unwrap(),
                HtmlSemanticQuery::BaseHref,
            )],
            surgeist_html::HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(1_000_000, 8, 1024, 128, 1_000_000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    match report.results()[0].fact() {
        HtmlSemanticFact::BaseHref(HtmlSemanticValue::Known(value)) => {
            assert_eq!(value.parsed(), Some(&parsed))
        }
        other => panic!("actual href result: {other:?}"),
    }
}
