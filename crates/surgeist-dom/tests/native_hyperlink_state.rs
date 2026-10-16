//! Qualified A/Area set-URL through actual native attribute and tree owners.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn policy(records: usize, work: u64, units: usize) -> DomLimits {
    DomLimits::new(128, units, 512, 64, 64, 8, 262144)
        .unwrap()
        .with_base_processing(128, 1_000_000)
        .unwrap()
        .with_encoding_url_processing(records, work)
        .unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId, NodeId) {
    let mut dom = Dom::new(policy(128, 1_000_000, 131072)).unwrap();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let root = b.create_element_ns(&doc, Namespace::html(), "div");
    let a = b.create_element_ns(&doc, Namespace::html(), "a");
    let area = b.create_element_ns(&doc, Namespace::html(), "area");
    b.insert_before(&root, &doc, None);
    let c = dom.apply(b, &cancel()).unwrap();
    (
        dom,
        c.document(&doc).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&a).unwrap().unwrap(),
        c.created(&area).unwrap().unwrap(),
    )
}
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn record(r: &HtmlBaseFactsRevision, s: &str) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        r.clone(),
        s.into(),
        HtmlBaseUrlScheme::Other,
        s == "about:blank",
    )
    .unwrap()
}
fn environment(
    r: &HtmlBaseFactsRevision,
    encoding: &HtmlBaseEncoding,
    fallback: &HtmlBaseUrlRecord,
    observations: Vec<HtmlBaseParseObservation>,
) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            r.clone(),
            Some(fallback.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        observations,
        HtmlBaseFactsLimits::new(128, 65536, 1_000_000),
    )
    .unwrap()
}
fn bind_base(dom: &mut Dom, doc: DocumentId, facts: &HtmlParseBaseFacts) {
    let mut b = DomBatch::new();
    b.bind_base_facts(dom.snapshot().document(doc).unwrap().stamp(), facts);
    dom.apply(b, &cancel()).unwrap();
}
fn href_facts(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    r: &HtmlBaseFactsRevision,
    entries: &[(&str, Option<HtmlBaseUrlRecord>)],
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        r.clone(),
        entries
            .iter()
            .map(|(raw, out)| {
                HtmlEncodingUrlParseObservation::new(
                    (*raw).into(),
                    out.clone().map_or(
                        HtmlEncodingUrlParseOutcome::Failure,
                        HtmlEncodingUrlParseOutcome::Parsed,
                    ),
                )
            })
            .collect(),
        HtmlBaseFactsLimits::new(128, 65536, 1_000_000),
    )
    .unwrap()
}
fn bind(
    dom: &mut Dom,
    doc: DocumentId,
    facts: &HtmlEncodingUrlParseFacts,
) -> Result<DomCommit, DomError> {
    let mut b = DomBatch::new();
    b.bind_encoding_url_facts(dom.snapshot().document(doc).unwrap().stamp(), facts);
    dom.apply(b, &cancel())
}
fn set(dom: &mut Dom, node: NodeId, raw: &str) -> Result<DomCommit, DomError> {
    let mut b = DomBatch::new();
    b.set_attribute(node, "href", raw);
    dom.apply(b, &cancel())
}
#[test]
fn detached_success_failure_empty_and_removal_retain_actual_nullable_cache_without_csp_filtering() {
    let (mut dom, doc, _, a, area) = fixture();
    let s = dom.snapshot();
    assert!(s.node(a).unwrap().as_hyperlink().unwrap().url().is_none());
    assert!(
        s.node(area)
            .unwrap()
            .as_hyperlink()
            .unwrap()
            .parse_observation()
            .is_none()
    );
    let r = revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = record(&r, "about:blank");
    bind_base(
        &mut dom,
        doc,
        &environment(&r, &encoding, &fallback, vec![]),
    );
    let url = HtmlBaseUrlRecord::observed(
        r.clone(),
        "data:text/plain,hello".into(),
        HtmlBaseUrlScheme::Data,
        false,
    )
    .unwrap();
    let facts = href_facts(
        &fallback,
        &encoding,
        &r,
        &[
            ("success", Some(url.clone())),
            ("failure", None),
            ("", None),
        ],
    );
    bind(&mut dom, doc, &facts).unwrap();
    set(&mut dom, a, "success").unwrap();
    let old = dom.snapshot();
    let view = old.node(a).unwrap().as_hyperlink().unwrap();
    assert_eq!(view.url(), Some(&url));
    assert_eq!(view.parse_document(), Some(doc));
    assert_eq!(view.parse_observation().unwrap().base(), &fallback);
    set(&mut dom, a, "failure").unwrap();
    let s = dom.snapshot();
    let view = s.node(a).unwrap().as_hyperlink().unwrap();
    assert!(view.url().is_none());
    assert!(matches!(
        view.parse_observation().unwrap().outcome(),
        HtmlEncodingUrlParseOutcome::Failure
    ));
    set(&mut dom, area, "").unwrap();
    assert!(
        dom.snapshot()
            .node(area)
            .unwrap()
            .as_hyperlink()
            .unwrap()
            .parse_observation()
            .is_some()
    );
    dom.set_limits(policy(128, 0, 131072)).unwrap();
    let mut b = DomBatch::new();
    b.remove_attribute(a, "href");
    b.remove_attribute(area, "href");
    dom.apply(b, &cancel()).unwrap();
    let s = dom.snapshot();
    for id in [a, area] {
        let v = s.node(id).unwrap().as_hyperlink().unwrap();
        assert!(v.url().is_none());
        assert!(v.parse_observation().is_none());
        assert_eq!(v.parse_document(), None);
    }
    assert_eq!(
        old.node(a).unwrap().as_hyperlink().unwrap().url(),
        Some(&url)
    );
}
#[test]
fn current_document_base_selection_and_historical_frozen_revision_are_real_parse_inputs() {
    let (mut dom, doc, root, a, _) = fixture();
    let r = revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = record(&r, "about:blank");
    let base_url = record(&r, "https://base.test/");
    let obs = HtmlBaseParseObservation::new(
        "base".into(),
        fallback.clone(),
        encoding.clone(),
        r.clone(),
        HtmlBaseParseOutcome::Parsed {
            record: base_url.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    )
    .unwrap();
    bind_base(
        &mut dom,
        doc,
        &environment(&r, &encoding, &fallback, vec![obs]),
    );
    let mut b = DomBatch::new();
    let base = b.create_element_ns(doc, Namespace::html(), "base");
    b.set_attribute(&base, "href", "base");
    b.insert_before(&base, root, None);
    let c = dom.apply(b, &cancel()).unwrap();
    let base = c.created(&base).unwrap().unwrap();
    let url = record(&r, "https://base.test/result");
    let wrong = href_facts(&fallback, &encoding, &r, &[("target", Some(url.clone()))]);
    assert_eq!(
        bind(&mut dom, doc, &wrong).unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
            HtmlEncodingUrlFactsError::InputsMismatch
        ))
    );
    bind(
        &mut dom,
        doc,
        &href_facts(&base_url, &encoding, &r, &[("target", Some(url.clone()))]),
    )
    .unwrap();
    set(&mut dom, a, "target").unwrap();
    let r2 = HtmlBaseFactsRevision::new(r.producer().clone(), 1, 20, 30);
    let fallback2 = record(&r2, "about:blank");
    let base2 = record(&r2, "https://second.test/");
    let obs = HtmlBaseParseObservation::new(
        "other".into(),
        fallback2.clone(),
        encoding.clone(),
        r2.clone(),
        HtmlBaseParseOutcome::Parsed {
            record: base2.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    )
    .unwrap();
    bind_base(
        &mut dom,
        doc,
        &environment(&r2, &encoding, &fallback2, vec![obs]),
    );
    // Binding changed the current parse context, without refreezing Base or A.
    let old = dom.snapshot();
    assert_eq!(
        old.node(base)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .unwrap()
            .value(),
        &base_url
    );
    dom.set_limits(policy(128, 0, 131072)).unwrap();
    set(&mut dom, a, "target").unwrap();
    assert_eq!(
        dom.snapshot()
            .node(a)
            .unwrap()
            .as_hyperlink()
            .unwrap()
            .url(),
        Some(&url)
    ); // equal skip uses no reachedparse allowance.
    dom.set_limits(policy(128, 1_000_000, 131072)).unwrap();
    let url2 = record(&r2, "https://base.test/new");
    let facts2 = href_facts(&base_url, &encoding, &r2, &[("next", Some(url2.clone()))]);
    bind(&mut dom, doc, &facts2).unwrap();
    set(&mut dom, a, "next").unwrap();
    let s = dom.snapshot();
    let parsed = s
        .node(a)
        .unwrap()
        .as_hyperlink()
        .unwrap()
        .parse_observation()
        .unwrap();
    assert_eq!(parsed.base().revision(), &r);
    assert_eq!(parsed.revision(), &r2);
    assert_eq!(parsed.url(), Some(&url2));
    let mut b = DomBatch::new();
    b.set_attribute(base, "href", "other");
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    assert_eq!(
        set(&mut dom, a, "target").unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
            HtmlEncodingUrlFactsError::InputsMismatch
        ))
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        before.node(a).unwrap().as_hyperlink().unwrap().url(),
        Some(&url2)
    );
}
#[test]
fn binding_rejects_missing_stale_foreign_and_identity_mismatched_context_atomically() {
    let (mut dom, doc, _, a, _) = fixture();
    let before = dom.snapshot();
    let r = revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = record(&r, "about:blank");
    let facts = href_facts(&fallback, &encoding, &r, &[("x", None)]);
    assert_eq!(
        bind(&mut dom, doc, &facts).unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Base(
            HtmlBaseMissing::DocumentFacts
        ))
    );
    bind_base(
        &mut dom,
        doc,
        &environment(&r, &encoding, &fallback, vec![]),
    );
    let mut b = DomBatch::new();
    b.bind_encoding_url_facts(before.document(doc).unwrap().stamp(), &facts);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::BasisRevisionMismatch)
    );
    let inputs = [
        (
            href_facts(&fallback, &HtmlBaseEncoding::fresh(), &r, &[("x", None)]),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            href_facts(&record(&r, "about:blank"), &encoding, &r, &[("x", None)]),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            href_facts(&fallback, &encoding, &revision(), &[("x", None)]),
            HtmlEncodingUrlFactsError::ForeignProducer,
        ),
        (
            href_facts(
                &fallback,
                &encoding,
                &HtmlBaseFactsRevision::new(r.producer().clone(), 1, 99, 3),
                &[("x", None)],
            ),
            HtmlEncodingUrlFactsError::RevisionMismatch,
        ),
    ];
    for (facts, error) in inputs {
        let old = dom.snapshot();
        assert_eq!(
            bind(&mut dom, doc, &facts).unwrap_err(),
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(error))
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
    }
    bind(&mut dom, doc, &facts).unwrap();
    let old = dom.snapshot();
    assert_eq!(
        set(&mut dom, a, "unobserved").unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
            HtmlEncodingUrlFactsError::MissingObservation
        ))
    );
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    assert!(
        dom.snapshot()
            .node(a)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute_ns(&Namespace::none(), &"href".into())
            .is_none()
    );
}
#[test]
fn actual_observation_capacity_and_cache_extent_have_exact_atomic_admission() {
    let (mut dom, doc, _, a, _) = fixture();
    let r = revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = record(&r, "about:blank");
    bind_base(
        &mut dom,
        doc,
        &environment(&r, &encoding, &fallback, vec![]),
    );
    let url = record(&r, "https://value.test/");
    let mut raw_units = Vec::with_capacity(64);
    raw_units.extend("success".encode_utf16());
    let raw = DocumentString::from_utf16(raw_units);
    let facts = HtmlEncodingUrlParseFacts::new(
        fallback.clone(),
        encoding.clone(),
        r.clone(),
        vec![HtmlEncodingUrlParseObservation::new(
            raw.clone(),
            HtmlEncodingUrlParseOutcome::Parsed(url.clone()),
        )],
        HtmlBaseFactsLimits::new(128, 65536, 1_000_000),
    )
    .unwrap();
    let records = facts.formation_usage().record_capacity();
    assert!(records > 0);
    dom.set_limits(policy(records - 1, 1_000_000, 131072))
        .unwrap();
    let old = dom.snapshot();
    assert!(matches!(
        bind(&mut dom, doc, &facts),
        Err(DomError::ResourceLimit {
            resource: DomResource::EncodingUrlObservations,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    dom.set_limits(policy(records, 1_000_000, 131072)).unwrap();
    bind(&mut dom, doc, &facts).unwrap();
    let units = dom.usage().string_code_units();
    let additional = 4
        + 7
        + raw.shared_code_units().capacity()
        + fallback.serialization().shared_code_units().capacity()
        + url.serialization().shared_code_units().capacity(); // logical href storage plus actual retained cache backing capacities.
    dom.set_limits(policy(records, 1_000_000, units + additional - 1))
        .unwrap();
    let old = dom.snapshot();
    assert!(matches!(
        set(&mut dom, a, "success"),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    dom.set_limits(policy(records, 1_000_000, units + additional))
        .unwrap();
    set(&mut dom, a, "success").unwrap();
    assert_eq!(dom.usage().string_code_units(), units + additional);
    dom.set_limits(policy(records, 1_000_000, 131072)).unwrap();
    let mut b = DomBatch::new();
    let second = b.create_document();
    let c = dom.apply(b, &cancel()).unwrap();
    let second = c.document(&second).unwrap();
    bind_base(
        &mut dom,
        second,
        &environment(&r, &encoding, &fallback, vec![]),
    );
    let old = dom.snapshot();
    assert!(matches!(
        bind(&mut dom, second, &facts),
        Err(DomError::ResourceLimit {
            resource: DomResource::EncodingUrlObservations,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    dom.set_limits(policy(records * 2, 1_000_000, 131072))
        .unwrap();
    bind(&mut dom, second, &facts).unwrap();
    assert!(matches!(
        dom.set_limits(policy(records * 2 - 1, 1_000_000, 131072)),
        Err(DomError::ResourceLimit {
            resource: DomResource::EncodingUrlObservations,
            ..
        })
    ));
}
#[test]
fn native_authoring_installation_parses_real_copied_href_and_captures_nullable_history() {
    use surgeist_html::{HtmlConstructionLimits, HtmlFragment};
    let (mut dom, doc, root, a, _) = fixture();
    let r = revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = record(&r, "about:blank");
    let url = record(&r, "https://installed.test/");
    bind_base(
        &mut dom,
        doc,
        &environment(&r, &encoding, &fallback, vec![]),
    );
    bind(
        &mut dom,
        doc,
        &href_facts(&fallback, &encoding, &r, &[("target", Some(url.clone()))]),
    )
    .unwrap();
    let mut builder =
        HtmlFragment::builder(HtmlConstructionLimits::new(8, 16, 8, 4096, 8)).unwrap();
    let source_root = builder.root();
    for name in ["a", "area"] {
        builder
            .append_element(
                &source_root,
                QualifiedName::new(Namespace::html(), None, name),
                OrderedAttributes::new([Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "href"),
                    "target",
                )])
                .unwrap(),
            )
            .unwrap();
    }
    let source = builder.finish();
    let report = dom
        .install_native_html_fragment(
            dom.snapshot().document(doc).unwrap().stamp(),
            root,
            None,
            &source,
            DomNativeHtmlInstallLimits::new(16, 100000, 131072),
            &cancel(),
        )
        .unwrap();
    let installed: Vec<_> = source
        .root()
        .children()
        .map(|s| report.node_for(&s).unwrap())
        .collect();
    for id in &installed {
        let h = report.snapshot().node(*id).unwrap().as_hyperlink().unwrap();
        assert_eq!(h.url(), Some(&url));
        assert_eq!(h.parse_document(), Some(doc));
    }
    // A-name/id expect hooks are now actual admitted A operations; native
    // Documents retain the source-initial empty render-blocking set.
    let mut b = DomBatch::new();
    b.set_attribute(a, "name", "target");
    b.set_attribute(a, "id", "target");
    b.insert_before(a, root, None);
    b.detach(a);
    b.set_attribute(a, "rel", "preload");
    b.set_attribute(a, "referrerpolicy", "no-referrer");
    dom.apply(b, &cancel()).unwrap();
    let capture = dom
        .snapshot()
        .html_capture(
            doc,
            None,
            HtmlCaptureLimits::new(128, 512, 128, 65536, 128, 0, 0),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let usage = capture.capture().usage();
    let snapshot = dom.snapshot();
    let producer =
        |units, relationships| HtmlCaptureLimits::new(128, relationships, 128, units, 128, 0, 0);
    snapshot
        .html_capture(
            doc,
            None,
            producer(usage.utf16_units(), usage.relationships()),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    for (units, relationships, expected) in [
        (
            usage.utf16_units() - 1,
            usage.relationships(),
            HtmlCaptureResource::Utf16Units,
        ),
        (
            usage.utf16_units(),
            usage.relationships() - 1,
            HtmlCaptureResource::Relationships,
        ),
    ] {
        assert!(
            matches!(snapshot.html_capture(doc,None,producer(units,relationships),DomHtmlCaptureLimits::new(128,1_000_000),&cancel()),Err(DomHtmlCaptureError::Capture(HtmlCaptureError::LimitExceeded { resource, .. })) if resource == expected)
        );
    }
    assert!(matches!(
        snapshot.html_capture(
            doc,
            None,
            producer(65536, 512),
            DomHtmlCaptureLimits::new(128, 0),
            &cancel()
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            ..
        })
    ));
    assert!(matches!(
        snapshot.html_capture(
            doc,
            None,
            producer(65536, 512),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &AtomicBool::new(true)
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    let capture = capture
        .with_slot_semantics(
            DomSlotSemanticLimits::new(
                DomSlotQueryLimits::new(128, 512).unwrap(),
                DomHtmlCaptureLimits::new(128, 1_000_000),
                HtmlLanguageDirectionLimits::new(128, 128, 65536, 1_000_000),
                HtmlInertFactLimits::new(128, 1_000_000),
            ),
            &cancel(),
        )
        .unwrap();
    for id in installed {
        let subject = capture.subject(id).unwrap();
        let node = capture.capture().node(&subject).unwrap();
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            panic!("Element")
        };
        assert!(
            matches!(e.hyperlink_url(), Some(HtmlCapturedHyperlinkUrl::Observed { snapshot, .. }) if snapshot.url() == Some(&url))
        );
    }
    let subject = capture.subject(a).unwrap();
    let node = capture.capture().node(&subject).unwrap();
    let HtmlCapturedNodeKind::Element(e) = node.kind() else {
        panic!("Element")
    };
    assert_eq!(e.hyperlink_url(), Some(&HtmlCapturedHyperlinkUrl::Null));
}
