//! HTML877d ordinary first-Base/frozen-history and Link disabled-removal outcomes.
//! URL records below are externally supplied observations, never parsed by tests.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(256, 65536, 1024, 512, 256, 16, 1_000_000)
            .unwrap()
            .with_base_processing(256, 10_000_000)
            .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}

fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 7, 11, 13)
}

fn record(
    revision: &HtmlBaseFactsRevision,
    spelling: &str,
    fragment_delimiter_offset: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        spelling.into(),
        HtmlBaseUrlScheme::Other,
        spelling == "about:blank",
        fragment_delimiter_offset,
    )
    .unwrap()
}

fn facts(
    revision: &HtmlBaseFactsRevision,
    observations: Vec<(DocumentString, HtmlBaseUrlRecord)>,
) -> (HtmlParseBaseFacts, HtmlBaseUrlRecord) {
    let fallback = record(revision, "about:blank", None);
    let encoding = HtmlBaseEncoding::fresh();
    let observations = observations
        .into_iter()
        .map(|(raw, record)| {
            HtmlBaseParseObservation::new(
                raw,
                fallback.clone(),
                encoding.clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record,
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap()
        })
        .collect();
    (
        HtmlParseBaseFacts::new(
            HtmlBaseDocumentFacts::new(
                revision.clone(),
                Some(fallback.clone()),
                HtmlAboutBaseUrl::KnownNull,
                false,
                Some(encoding),
            )
            .unwrap(),
            observations,
            HtmlBaseFactsLimits::new(64, 32768, 1_000_000),
        )
        .unwrap(),
        fallback,
    )
}

fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(512, 8192, 512, 65536, 512, 0, 0),
            DomHtmlCaptureLimits::new(512, 1_000_000).with_base_facts(512, 65536, 1_000_000),
            &cancel(),
        )
        .unwrap()
}

fn document_base(capture: &DomHtmlCapture, document: DocumentId) -> HtmlDocumentBaseUrl {
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(document.node()).unwrap(),
                HtmlSemanticQuery::DocumentBaseUrl,
            )],
            HtmlValueInterpreter::new(65536),
            HtmlSemanticLimits::new(1_000_000, 8, 1024, 512, 4_000_000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    match report.results()[0].fact() {
        HtmlSemanticFact::DocumentBaseUrl(HtmlSemanticValue::Known(value)) => value.clone(),
        other => panic!("qualified native Document base must be known: {other:?}"),
    }
}

fn frozen(snapshot: &DomSnapshot, base: NodeId) -> DomFrozenBase<'_> {
    snapshot
        .node(base)
        .unwrap()
        .as_base()
        .unwrap()
        .frozen()
        .unwrap()
}

#[test]
fn ordinary_document_first_base_ignores_shadow_and_template_and_defers_nonfirst_changes() {
    let mut dom = owner();
    let revision = revision();
    let first_record = record(&revision, "https://supplied.test/first/", None);
    let second_record = record(&revision, "https://supplied.test/second/", None);
    let latest_record = record(&revision, "https://supplied.test/latest/", None);
    let (facts, _) = facts(
        &revision,
        vec![
            ("first".into(), first_record.clone()),
            ("second".into(), second_record),
            ("latest".into(), latest_record.clone()),
        ],
    );
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let host = batch.create_element_ns(&document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let template = batch.create_element_ns(&document, Namespace::html(), "template");
    let a = batch.create_element_ns(&document, Namespace::html(), "base");
    let b = batch.create_element_ns(&document, Namespace::html(), "base");
    for node in [&host, &template, &a, &b] {
        batch.insert_before(node, &root, None);
    }
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let template = commit.created(&template).unwrap().unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();

    let mut batch = DomBatch::new();
    batch.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
    let shadow_base = batch.create_element_ns(document, Namespace::html(), "base");
    let template_base = batch.create_element_ns(document, Namespace::html(), "base");
    // No observation for this spelling exists: these trees must not request one.
    for (node, parent) in [(&shadow_base, shadow), (&template_base, contents)] {
        batch.set_attribute(node, "href", "unobserved");
        batch.insert_before(node, parent, None);
    }
    batch.set_attribute(a, "href", "first");
    batch.set_attribute(b, "href", "second");
    batch.set_attribute(b, "href", "latest");
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.document(document).unwrap().first_base().unwrap(),
        Some(a)
    );
    assert_eq!(frozen(&snapshot, a).value(), &first_record);
    for node in [
        b,
        commit.created(&shadow_base).unwrap().unwrap(),
        commit.created(&template_base).unwrap().unwrap(),
    ] {
        assert!(
            snapshot
                .node(node)
                .unwrap()
                .as_base()
                .unwrap()
                .frozen()
                .is_none()
        );
    }
    assert_eq!(
        document_base(&capture(&snapshot, document), document).value(),
        &first_record
    );

    let mut batch = DomBatch::new();
    batch.remove_attribute(a, "href");
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(
        current.document(document).unwrap().first_base().unwrap(),
        Some(b)
    );
    assert_eq!(frozen(&current, b).value(), &latest_record);
    assert_eq!(frozen(&current, a).value(), &first_record);
    assert_eq!(
        document_base(&capture(&current, document), document).value(),
        &latest_record
    );
    assert_eq!(
        snapshot.document(document).unwrap().first_base().unwrap(),
        Some(a)
    );
    assert!(
        snapshot
            .node(b)
            .unwrap()
            .as_base()
            .unwrap()
            .frozen()
            .is_none()
    );
}

#[test]
fn detached_and_adopted_base_preserves_frozen_document_until_a_new_actual_freeze() {
    let mut dom = owner();
    let revision = revision();
    let from_first = record(&revision, "https://first.test/retained/", None);
    let from_second = record(&revision, "https://second.test/new/", None);
    let (first_facts, _) = facts(&revision, vec![("kept".into(), from_first.clone())]);
    let (second_facts, second_fallback) =
        facts(&revision, vec![("kept".into(), from_second.clone())]);
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let first_root = batch.create_element_ns(&first, Namespace::html(), "div");
    let second_root = batch.create_element_ns(&second, Namespace::html(), "div");
    let base = batch.create_element_ns(&first, Namespace::html(), "base");
    batch.insert_before(&base, &first_root, None);
    batch.insert_before(&first_root, &first, None);
    batch.insert_before(&second_root, &second, None);
    let commit = apply(&mut dom, batch);
    let first = commit.document(&first).unwrap();
    let second = commit.document(&second).unwrap();
    let second_root = commit.created(&second_root).unwrap().unwrap();
    let base = commit.created(&base).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.bind_base_facts(snapshot.document(first).unwrap().stamp(), &first_facts);
    batch.bind_base_facts(snapshot.document(second).unwrap().stamp(), &second_facts);
    batch.set_attribute(base, "href", "kept");
    apply(&mut dom, batch);
    let old = dom.snapshot();

    let mut batch = DomBatch::new();
    batch.detach(base);
    batch.remove_attribute(base, "href");
    batch.adopt(base, second);
    apply(&mut dom, batch);
    let adopted = dom.snapshot();
    assert_eq!(adopted.node(base).unwrap().owner_document(), Some(second));
    assert_eq!(frozen(&adopted, base).document(), first);
    assert_eq!(frozen(&adopted, base).value(), &from_first);
    assert_eq!(frozen(&adopted, base).revision(), &revision);
    // Producer validation requires the frozen row's CURRENT node Document.
    let captured = capture(&adopted, second);
    assert_eq!(document_base(&captured, second).value(), &second_fallback);

    let mut batch = DomBatch::new();
    batch.insert_before(base, second_root, None);
    batch.set_attribute(base, "href", "kept");
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(frozen(&current, base).document(), second);
    assert_eq!(frozen(&current, base).value(), &from_second);
    assert_eq!(
        document_base(&capture(&current, second), second).value(),
        &from_second
    );
    assert_eq!(frozen(&old, base).document(), first);
    assert_eq!(old.node(base).unwrap().owner_document(), Some(first));
    assert_eq!(frozen(&adopted, base).value(), &from_first);
}

#[test]
fn exact_raw_utf16_observations_and_late_error_preserve_the_prior_frozen_record() {
    let mut dom = owner();
    let revision = revision();
    let first_record = record(&revision, "https://literal.test/first/", None);
    let next_record = record(&revision, "https://literal.test/next/", None);
    let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    let next_raw = DocumentString::from_utf16(vec![0xd800, 9, 0xdc00]);
    let missing_raw = DocumentString::from_utf16(vec![0xd801, 0, 0xdc00]);
    let (facts, _) = facts(
        &revision,
        vec![
            (raw.clone(), first_record.clone()),
            (next_raw.clone(), next_record.clone()),
        ],
    );
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let base = batch.create_element_ns(&document, Namespace::html(), "base");
    batch.insert_before(&base, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let base = commit.created(&base).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), &facts);
    batch.set_attribute(base, "href", raw.clone());
    apply(&mut dom, batch);
    let before = dom.snapshot();
    assert_eq!(frozen(&before, base).value(), &first_record);

    let mut batch = DomBatch::new();
    batch.set_attribute(base, "href", missing_raw);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::BaseFacts(DomBaseFactsError::Missing(
            HtmlBaseMissing::ParseObservation
        ))
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().node(base).unwrap().attribute(&"href".into()),
        Some(&raw)
    );

    let mut batch = DomBatch::new();
    batch.set_attribute(base, "href", next_raw.clone());
    batch.set_attribute(base, "invalid name", "late failure");
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidCharacter
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(frozen(&dom.snapshot(), base).value(), &first_record);
    let mut batch = DomBatch::new();
    batch.set_attribute(base, "href", next_raw.clone());
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(frozen(&current, base).value(), &next_record);
    assert_eq!(
        current.node(base).unwrap().attribute(&"href".into()),
        Some(&next_raw)
    );
    assert_eq!(frozen(&before, base).value(), &first_record);
}

#[test]
fn link_disabled_removal_history_survives_adoption_but_clone_starts_fresh() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let link = batch.create_element_ns(&first, Namespace::html(), "link");
    let commit = apply(&mut dom, batch);
    let first = commit.document(&first).unwrap();
    let second = commit.document(&second).unwrap();
    let link = commit.created(&link).unwrap().unwrap();
    let fresh = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.remove_attribute(link, "disabled");
    batch.set_attribute_ns(link, Namespace::from_uri("urn:test"), "p:disabled", "");
    batch.remove_attribute_ns(link, Namespace::from_uri("urn:test"), "disabled");
    apply(&mut dom, batch);
    assert!(
        !dom.snapshot()
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(link, "disabled", "false");
    batch.remove_attribute(link, "disabled");
    batch.set_attribute(link, "disabled", "");
    apply(&mut dom, batch);
    let enabled = dom.snapshot();
    assert!(
        enabled
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );
    assert!(
        !fresh
            .node(link)
            .unwrap()
            .as_link()
            .unwrap()
            .flags()
            .explicitly_enabled()
    );

    let mut batch = DomBatch::new();
    batch.adopt(link, second);
    let copy = batch.import_node(&enabled, link, second, false);
    let commit = apply(&mut dom, batch);
    let copy = commit.created(&copy).unwrap().unwrap();
    let current = dom.snapshot();
    assert_eq!(current.node(link).unwrap().owner_document(), Some(second));
    assert_eq!(enabled.node(link).unwrap().owner_document(), Some(first));
    for (node, expected) in [(link, true), (copy, false)] {
        let view = current.node(node).unwrap().as_link().unwrap();
        assert_eq!(view.flags().explicitly_enabled(), expected);
        assert_eq!(view.parser_document(), None);
        assert!(!view.history().created_by_node_document_parser());
        assert_eq!(
            current.node(node).unwrap().attribute(&"disabled".into()),
            Some(&DocumentString::from(""))
        );
    }
    let capture = capture(&current, second);
    for (node, expected) in [(link, true), (copy, false)] {
        let subject = capture.subject(node).unwrap();
        let captured = capture.capture().node(&subject).unwrap();
        let HtmlCapturedNodeKind::Element(element) = captured.kind() else {
            panic!("Link capture must be Element");
        };
        let history = element.link_history().unwrap();
        assert_eq!(history.flags().explicitly_enabled(), expected);
        assert!(!history.created_by_node_document_parser());
    }
}
