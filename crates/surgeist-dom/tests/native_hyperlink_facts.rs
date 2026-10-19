//! Literal HTML877d #concept-hyperlink-url-set and A/Area attribute outcomes.
//! Supplied URL observations are opaque external facts, not a test URL parser.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            65536,
            1024,
            1024,
            512,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_base_processing(256, 10_000_000)
        .unwrap()
        .with_encoding_url_processing(256, 10_000_000)
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 5, 7, 11)
}
fn record(
    revision: &HtmlBaseFactsRevision,
    spelling: &str,
    blank: bool,
    fragment_delimiter_offset: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        spelling.into(),
        HtmlBaseUrlScheme::Other,
        blank,
        fragment_delimiter_offset,
    )
    .unwrap()
}
struct Environment {
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    base_facts: HtmlParseBaseFacts,
    encoding_url_facts: HtmlEncodingUrlParseFacts,
}
fn environment(
    revision: &HtmlBaseFactsRevision,
    base_spelling: &str,
    outcomes: Vec<(DocumentString, HtmlEncodingUrlParseOutcome)>,
    fragment_delimiter_offset: Option<usize>,
) -> Environment {
    let fallback = record(revision, "about:blank", true, None);
    let base = record(revision, base_spelling, false, fragment_delimiter_offset);
    let encoding = HtmlBaseEncoding::fresh();
    let base_facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
            Some(fallback.clone()),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding.clone()),
        )
        .unwrap(),
        vec![
            HtmlBaseParseObservation::new(
                "base".into(),
                fallback,
                encoding.clone(),
                revision.clone(),
                HtmlBaseParseOutcome::Parsed {
                    record: base.clone(),
                    base_allowed: HtmlBaseAllowed::Allowed,
                },
            )
            .unwrap(),
        ],
        HtmlBaseFactsLimits::new(64, 32768, 1_000_000),
    )
    .unwrap();
    let encoding_url_facts = HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        outcomes
            .into_iter()
            .map(|(raw, outcome)| HtmlEncodingUrlParseObservation::new(raw, outcome))
            .collect(),
        HtmlBaseFactsLimits::new(64, 32768, 1_000_000),
    )
    .unwrap();
    Environment {
        base,
        encoding,
        base_facts,
        encoding_url_facts,
    }
}
fn bind_base(dom: &mut Dom, document: DocumentId, base: NodeId, environment: &Environment) {
    let mut batch = DomBatch::new();
    batch.bind_base_facts(
        dom.snapshot().document(document).unwrap().stamp(),
        &environment.base_facts,
    );
    batch.set_attribute(base, "href", "base");
    apply(dom, batch);
}
fn bind_hyperlinks(dom: &mut Dom, document: DocumentId, environment: &Environment) {
    let mut batch = DomBatch::new();
    batch.bind_encoding_url_facts(
        dom.snapshot().document(document).unwrap().stamp(),
        &environment.encoding_url_facts,
    );
    apply(dom, batch);
}
fn link(snapshot: &DomSnapshot, node: NodeId) -> DomHyperlink<'_> {
    snapshot.node(node).unwrap().as_hyperlink().unwrap()
}
fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(512, 8192, 512, 65536, 512, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(512, 1_000_000).with_base_facts(512, 65536, 1_000_000),
            &cancel(),
        )
        .unwrap()
}
struct Pair {
    dom: Dom,
    first: DocumentId,
    second: DocumentId,
    first_base: NodeId,
    second_base: NodeId,
    hyperlink: NodeId,
}
fn pair(tag: &str) -> Pair {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let first = batch.create_document();
    let second = batch.create_document();
    let first_base = batch.create_element_ns(&first, Namespace::html(), "base");
    let second_base = batch.create_element_ns(&second, Namespace::html(), "base");
    let hyperlink = batch.create_element_ns(&first, Namespace::html(), tag);
    batch.insert_before(&first_base, &first, None);
    batch.insert_before(&second_base, &second, None);
    let commit = apply(&mut dom, batch);
    Pair {
        first: commit.document(&first).unwrap(),
        second: commit.document(&second).unwrap(),
        first_base: commit.created(&first_base).unwrap().unwrap(),
        second_base: commit.created(&second_base).unwrap().unwrap(),
        hyperlink: commit.created(&hyperlink).unwrap().unwrap(),
        dom,
    }
}

#[test]
fn equal_href_after_adoption_preserves_history_without_requiring_the_new_context() {
    for tag in ["a", "area"] {
        let mut f = pair(tag);
        let revision = revision();
        let old_url = record(&revision, "https://old.test/opaque-result", false, None);
        let new_url = record(&revision, "https://new.test/opaque-result", false, None);
        let first = environment(
            &revision,
            "https://first.test/base/",
            vec![(
                "same".into(),
                HtmlEncodingUrlParseOutcome::Parsed(old_url.clone()),
            )],
            None,
        );
        let second = environment(
            &revision,
            "https://second.test/base/",
            vec![(
                "changed".into(),
                HtmlEncodingUrlParseOutcome::Parsed(new_url.clone()),
            )],
            None,
        );
        bind_base(&mut f.dom, f.first, f.first_base, &first);
        bind_base(&mut f.dom, f.second, f.second_base, &second);
        bind_hyperlinks(&mut f.dom, f.first, &first);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", "same");
        apply(&mut f.dom, batch);
        let original = f.dom.snapshot();
        assert_eq!(link(&original, f.hyperlink).url(), Some(&old_url));
        assert_eq!(
            link(&original, f.hyperlink)
                .parse_observation()
                .unwrap()
                .base(),
            &first.base
        );
        let mut batch = DomBatch::new();
        batch.adopt(f.hyperlink, f.second);
        apply(&mut f.dom, batch);
        let adopted = f.dom.snapshot();
        assert_eq!(
            adopted.node(f.hyperlink).unwrap().owner_document(),
            Some(f.second)
        );
        assert_eq!(link(&adopted, f.hyperlink).parse_document(), Some(f.first));
        assert_eq!(link(&adopted, f.hyperlink).url(), Some(&old_url));
        assert!(
            adopted
                .document(f.second)
                .unwrap()
                .encoding_url_facts()
                .is_none()
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", "same");
        apply(&mut f.dom, batch);
        assert_eq!(f.dom.snapshot().stamp(), adopted.stamp());
        let mut batch = DomBatch::new();
        batch.set_attribute_ns(
            f.hyperlink,
            Namespace::from_uri("urn:test"),
            "p:href",
            "changed",
        );
        apply(&mut f.dom, batch);
        let namespaced = f.dom.snapshot();
        assert_eq!(link(&namespaced, f.hyperlink).url(), Some(&old_url));
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", "changed");
        assert_eq!(
            f.dom.apply(batch, &cancel()).unwrap_err(),
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::MissingFacts)
        );
        assert_eq!(f.dom.snapshot().stamp(), namespaced.stamp());
        assert_eq!(
            f.dom
                .snapshot()
                .node(f.hyperlink)
                .unwrap()
                .attribute(&"href".into()),
            Some(&DocumentString::from("same"))
        );

        // The capture's historical parse Document is independent of nodeDocument.
        let captured = capture(&namespaced, f.second);
        let subject = captured.subject(f.hyperlink).unwrap();
        let node = captured.capture().node(&subject).unwrap();
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            panic!("hyperlink must capture as Element");
        };
        let Some(HtmlCapturedHyperlinkUrl::Observed { document, snapshot }) =
            element.hyperlink_url()
        else {
            panic!("actual old parse history must remain captured");
        };
        assert_eq!(captured.node(document.subject()).unwrap(), f.first.node());
        assert_eq!(snapshot.url(), Some(&old_url));

        bind_hyperlinks(&mut f.dom, f.second, &second);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", "changed");
        apply(&mut f.dom, batch);
        let current = f.dom.snapshot();
        let view = link(&current, f.hyperlink);
        assert_eq!(view.url(), Some(&new_url));
        assert_eq!(view.parse_document(), Some(f.second));
        assert_eq!(view.parse_observation().unwrap().base(), &second.base);
        assert_eq!(
            view.parse_observation().unwrap().encoding(),
            &second.encoding
        );
        assert_eq!(link(&original, f.hyperlink).url(), Some(&old_url));
        assert_eq!(
            original.node(f.hyperlink).unwrap().owner_document(),
            Some(f.first)
        );
    }
}

#[test]
fn clone_parses_copied_href_in_its_destination_instead_of_copying_source_record_identity() {
    for tag in ["a", "area"] {
        let mut f = pair(tag);
        let revision = revision();
        let source_url = record(&revision, "https://equal-spelling.test/result", false, None);
        let target_url = record(&revision, "https://equal-spelling.test/result", false, None);
        assert_ne!(source_url, target_url);
        let raw = DocumentString::from_utf16(vec![9, 0xd800, 0, 0xdc00, 32]);
        let first = environment(
            &revision,
            "https://source.test/base/",
            vec![(
                raw.clone(),
                HtmlEncodingUrlParseOutcome::Parsed(source_url.clone()),
            )],
            None,
        );
        let second = environment(
            &revision,
            "https://target.test/base/",
            vec![(
                raw.clone(),
                HtmlEncodingUrlParseOutcome::Parsed(target_url.clone()),
            )],
            None,
        );
        bind_base(&mut f.dom, f.first, f.first_base, &first);
        bind_base(&mut f.dom, f.second, f.second_base, &second);
        bind_hyperlinks(&mut f.dom, f.first, &first);
        bind_hyperlinks(&mut f.dom, f.second, &second);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", raw.clone());
        apply(&mut f.dom, batch);
        let before = f.dom.snapshot();
        let mut batch = DomBatch::new();
        let copy = batch.import_node(&before, f.hyperlink, f.second, false);
        let commit = apply(&mut f.dom, batch);
        let copy = commit.created(&copy).unwrap().unwrap();
        let current = f.dom.snapshot();
        assert_ne!(copy, f.hyperlink);
        let copied = link(&current, copy);
        assert_eq!(copied.url(), Some(&target_url));
        assert_eq!(copied.parse_document(), Some(f.second));
        assert_eq!(copied.parse_observation().unwrap().raw(), &raw);
        assert_eq!(copied.parse_observation().unwrap().base(), &second.base);
        assert_eq!(
            copied.parse_observation().unwrap().encoding(),
            &second.encoding
        );
        assert_eq!(link(&current, f.hyperlink).url(), Some(&source_url));
        assert_eq!(link(&before, f.hyperlink).url(), Some(&source_url));
    }
}

#[test]
fn exact_utf16_success_failure_empty_and_absent_href_have_distinct_history_and_atomic_refusals() {
    for tag in ["a", "area"] {
        let mut f = pair(tag);
        let revision = revision();
        let parsed_url = record(&revision, "https://literal.test/success", false, None);
        let empty_url = record(&revision, "https://literal.test/present-empty", false, None);
        let raw = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
        let failure_raw = DocumentString::from_utf16(vec![0xd800, 9, 0xdc00]);
        let missing_raw = DocumentString::from_utf16(vec![0xd801, 0, 0xdc00]);
        let environment = environment(
            &revision,
            "https://literal.test/base/",
            vec![
                (
                    raw.clone(),
                    HtmlEncodingUrlParseOutcome::Parsed(parsed_url.clone()),
                ),
                (failure_raw.clone(), HtmlEncodingUrlParseOutcome::Failure),
                (
                    "".into(),
                    HtmlEncodingUrlParseOutcome::Parsed(empty_url.clone()),
                ),
            ],
            None,
        );
        let fresh = f.dom.snapshot();
        assert!(link(&fresh, f.hyperlink).url().is_none());
        assert!(link(&fresh, f.hyperlink).parse_observation().is_none());
        bind_base(&mut f.dom, f.first, f.first_base, &environment);
        bind_hyperlinks(&mut f.dom, f.first, &environment);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", raw.clone());
        apply(&mut f.dom, batch);
        let parsed = f.dom.snapshot();
        assert_eq!(link(&parsed, f.hyperlink).url(), Some(&parsed_url));
        assert_eq!(
            link(&parsed, f.hyperlink)
                .parse_observation()
                .unwrap()
                .raw(),
            &raw
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", failure_raw.clone());
        apply(&mut f.dom, batch);
        let failed = f.dom.snapshot();
        let view = link(&failed, f.hyperlink);
        assert!(view.url().is_none());
        assert_eq!(view.parse_document(), Some(f.first));
        assert_eq!(view.parse_observation().unwrap().raw(), &failure_raw);
        assert_eq!(
            view.parse_observation().unwrap().outcome(),
            &HtmlEncodingUrlParseOutcome::Failure
        );
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", failure_raw);
        apply(&mut f.dom, batch);
        assert_eq!(f.dom.snapshot().stamp(), failed.stamp());
        let mut batch = DomBatch::new();
        batch.remove_attribute(f.hyperlink, "href");
        apply(&mut f.dom, batch);
        let absent = f.dom.snapshot();
        assert!(link(&absent, f.hyperlink).url().is_none());
        assert!(link(&absent, f.hyperlink).parse_observation().is_none());
        assert_eq!(link(&absent, f.hyperlink).parse_document(), None);
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", "");
        apply(&mut f.dom, batch);
        let empty = f.dom.snapshot();
        assert_eq!(link(&empty, f.hyperlink).url(), Some(&empty_url));
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", missing_raw);
        assert_eq!(
            f.dom.apply(batch, &cancel()).unwrap_err(),
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
                HtmlEncodingUrlFactsError::MissingObservation
            ))
        );
        assert_eq!(f.dom.snapshot().stamp(), empty.stamp());
        let mut batch = DomBatch::new();
        batch.set_attribute(f.hyperlink, "href", raw);
        batch.set_attribute(f.hyperlink, "invalid name", "late");
        assert_eq!(
            f.dom.apply(batch, &cancel()).unwrap_err(),
            DomError::InvalidCharacter
        );
        assert_eq!(f.dom.snapshot().stamp(), empty.stamp());
        assert_eq!(link(&f.dom.snapshot(), f.hyperlink).url(), Some(&empty_url));
        assert_eq!(link(&parsed, f.hyperlink).url(), Some(&parsed_url));
    }
}
