//! Actual source triggers and Document history; no CSS, policy or UI stand-ins.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlReferrerPolicy;
use surgeist_html_semantics::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(work: u64) -> DomLimits {
    DomLimits::new(128, 32768, 512, 128, 64, 8, 262144)
        .unwrap()
        .with_meta_processing_work(work)
        .with_meta_theme_color_scratch_bytes(262144)
        .unwrap()
        .with_meta_color_scheme_limits(surgeist_css::CssUtf16PropertyValueLimits {
            input: surgeist_css::CssUtf16InputLimits::default(),
            syntax: surgeist_css::CssComponentValueLimits::default(),
            max_work: 1_000_000,
            max_live_bytes: 1_000_000,
            max_retained_bytes: 1_000_000,
        })
        .with_meta_color_scheme_retained_bytes(1_000_000)
        .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(limits(1_000_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "head");
    let meta = batch.create_element_ns(&document, Namespace::html(), "meta");
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&root).unwrap().unwrap(),
        commit.created(&meta).unwrap().unwrap(),
    )
}
fn language(snapshot: &DomSnapshot, document: DocumentId) -> Option<&DocumentString> {
    snapshot
        .document(document)
        .unwrap()
        .metadata()
        .meta_state()
        .unwrap()
        .pragma_language()
}

#[test]
fn language_is_exact_insertion_history_and_snapshots_preserve_it() {
    let (mut dom, document, root, meta) = fixture();
    let initial = dom.snapshot();
    assert_eq!(language(&initial, document), None);
    assert_eq!(
        initial
            .document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::StrictOriginWhenCrossOrigin)
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "CoNtEnT-LaNgUaGe");
    batch.set_attribute(meta, "content", " \tEN-private more");
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    let first = dom.snapshot();
    assert_eq!(language(&first, document), Some(&"EN-private".into()));
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "fr");
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.detach(meta);
    apply(&mut dom, batch);
    assert_eq!(
        language(&dom.snapshot(), document),
        Some(&"EN-private".into())
    );
    let mut batch = DomBatch::new();
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    assert_eq!(language(&dom.snapshot(), document), Some(&"fr".into()));
    assert_eq!(language(&first, document), Some(&"EN-private".into()));
    assert_eq!(language(&initial, document), None);
    // A comma anywhere rejects the entire candidate; invalid tags are not filtered.
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.set_attribute(meta, "content", "de extra,tail");
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    assert_eq!(language(&dom.snapshot(), document), Some(&"fr".into()));
}

#[test]
fn referrer_uses_source_aliases_without_trimming_and_equal_setters_reapply() {
    let (mut dom, document, root, meta) = fixture();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "name", "ReFeRrEr");
    batch.set_attribute(meta, "content", "ALWAYS");
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    let first = dom.snapshot();
    assert_eq!(
        first
            .document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::UnsafeUrl)
    );
    for (raw, expected) in [
        ("never", HtmlReferrerPolicy::NoReferrer),
        ("default", HtmlReferrerPolicy::StrictOriginWhenCrossOrigin),
        (
            "origin-when-crossorigin",
            HtmlReferrerPolicy::OriginWhenCrossOrigin,
        ),
        (" no-referrer ", HtmlReferrerPolicy::OriginWhenCrossOrigin),
        ("", HtmlReferrerPolicy::OriginWhenCrossOrigin),
    ] {
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "content", raw);
        apply(&mut dom, batch);
        assert_eq!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .policy_container()
                .referrer_policy(),
            Some(expected)
        );
    }
    // Another Meta changes the policy; a same-value setter on the first must rerun.
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "never");
    let other = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&other, "name", "referrer");
    batch.set_attribute(&other, "content", "always");
    batch.insert_before(&other, root, None);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "never");
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot()
            .document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::NoReferrer)
    );
    assert_eq!(
        first
            .document(document)
            .unwrap()
            .metadata()
            .policy_container()
            .referrer_policy(),
        Some(HtmlReferrerPolicy::UnsafeUrl)
    );
}

#[test]
fn refresh_current_url_sets_real_flag_and_later_insertions_return_before_url_work() {
    for input in ["0", ".", "123456789012345678901234567890..9; \t", " 4 , "] {
        let (mut dom, document, root, meta) = fixture();
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "http-equiv", "refresh");
        batch.set_attribute(meta, "content", input);
        batch.insert_before(meta, root, None);
        apply(&mut dom, batch);
        let before = dom.snapshot();
        assert!(
            before
                .document(document)
                .unwrap()
                .metadata()
                .meta_state()
                .unwrap()
                .will_declaratively_refresh()
        );
        let mut batch = DomBatch::new();
        batch.detach(meta);
        batch.set_attribute(meta, "content", "0; URL='unsupported'");
        batch.insert_before(meta, root, None);
        apply(&mut dom, batch);
        assert!(
            dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .meta_state()
                .unwrap()
                .will_declaratively_refresh()
        );
    }
}

#[test]
fn refresh_rejections_and_unavailable_url_branch_are_distinct_and_atomic() {
    let (mut dom, document, root, meta) = fixture();
    dom.set_limits(url_limits(1_000_000)).unwrap();
    for raw in ["", "x", "-1", "2x"] {
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "http-equiv", "refresh");
        batch.set_attribute(meta, "content", raw);
        batch.insert_before(meta, root, None);
        apply(&mut dom, batch);
        assert!(
            !dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .meta_state()
                .unwrap()
                .will_declaratively_refresh()
        );
    }
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.set_attribute(meta, "content", "0; URL='next'");
    batch.insert_before(meta, root, None);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Base(
            surgeist_html_semantics::HtmlBaseMissing::DocumentFacts,
        ))
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
}

#[test]
fn reached_external_operations_refuse_whole_batch_without_history_or_identity() {
    for (attribute, value, content, error) in [
        (
            "http-equiv",
            "default-style",
            "named",
            DomMetaError::DefaultStyleUnsupported,
        ),
        (
            "http-equiv",
            "content-security-policy",
            "default-src 'none'",
            DomMetaError::ContentSecurityPolicyUnsupported,
        ),
    ] {
        let (mut dom, _, root, meta) = fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, attribute, value);
        batch.set_attribute(meta, "content", content);
        batch.insert_before(meta, root, None);
        assert_eq!(
            dom.apply(batch, &cancel()).unwrap_err(),
            DomError::Meta(error)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}

#[test]
fn empty_css_meta_candidates_complete_with_no_selected_value() {
    // The original empty-content cases reach CSS syntax failure, rather than
    // requiring an external service or producing a fallback selection.
    for name in ["theme-color", "color-scheme"] {
        let (mut dom, document, root, meta) = fixture();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "name", name);
        batch.set_attribute(meta, "content", "");
        batch.insert_before(meta, root, None);
        apply(&mut dom, batch);
        let snapshot = dom.snapshot();
        let metadata = snapshot.document(document).unwrap().metadata();
        assert_eq!(
            metadata.meta_theme_color(),
            &DomMetaThemeColorState::Known(None)
        );
        assert!(matches!(
            metadata.meta_color_scheme(),
            DomMetaColorSchemeState::Known(None)
        ));
        assert_eq!(snapshot.node(meta).unwrap().parent(), Some(root));
        assert_eq!(before.node(meta).unwrap().parent(), None);
        assert!(snapshot.incomplete_source_operation().is_none());
    }
}

#[test]
fn adoption_preserves_old_document_history_and_insertion_uses_destination_history() {
    let (mut dom, first_document, first_root, meta) = fixture();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "first");
    batch.insert_before(meta, first_root, None);
    let second = batch.create_document();
    let root = batch.create_element_ns(&second, Namespace::html(), "head");
    batch.insert_before(&root, &second, None);
    let commit = apply(&mut dom, batch);
    let second_document = commit.document(&second).unwrap();
    let second_root = commit.created(&root).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(meta, second_document);
    batch.set_attribute(meta, "content", "second");
    apply(&mut dom, batch);
    assert_eq!(
        language(&dom.snapshot(), first_document),
        Some(&"first".into())
    );
    assert_eq!(language(&dom.snapshot(), second_document), None);
    let mut batch = DomBatch::new();
    batch.insert_before(meta, second_root, None);
    apply(&mut dom, batch);
    assert_eq!(
        language(&dom.snapshot(), first_document),
        Some(&"first".into())
    );
    assert_eq!(
        language(&dom.snapshot(), second_document),
        Some(&"second".into())
    );
    assert_eq!(language(&before, second_document), None);
}

#[test]
fn work_and_cancellation_refuse_without_publication_even_for_equal_final_history() {
    let (mut dom, document, root, meta) = fixture();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "same");
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let usage = dom.usage();
    dom.set_limits(limits(0)).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.insert_before(meta, root, None);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::Meta(DomMetaError::WorkLimit { limit: 0 })
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(limits(1_000_000)).unwrap();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.insert_before(meta, root, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(language(&before, document), Some(&"same".into()));
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.insert_before(meta, root, None);
    let commit = apply(&mut dom, batch);
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(std::sync::Arc::ptr_eq(
        language(&before, document).unwrap().shared_code_units(),
        language(&dom.snapshot(), document)
            .unwrap()
            .shared_code_units(),
    ));
}

#[test]
fn ordinary_and_slot_semantics_consume_retained_language_after_meta_removal() {
    use surgeist_html_semantics::*;
    let (mut dom, document, root, meta) = fixture();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "MiXeD-private extra");
    batch.insert_before(meta, root, None);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.destroy(meta);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for slot in [false, true] {
        let mut capture = snapshot
            .html_capture(
                document,
                None,
                HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 0, 0, 16_777_216),
                DomHtmlCaptureLimits::new(128, 1_000_000),
                &cancel(),
            )
            .unwrap();
        if slot {
            capture = capture
                .with_slot_semantics(
                    DomSlotSemanticLimits::new(
                        DomSlotQueryLimits::new(128, 256).unwrap(),
                        DomHtmlCaptureLimits::new(128, 1_000_000),
                        HtmlLanguageDirectionLimits::new(256, 256, 32768, 1_000_000),
                        HtmlInertFactLimits::new(256, 1_000_000),
                    ),
                    &cancel(),
                )
                .unwrap();
        }
        let report = capture
            .query(
                &[HtmlSemanticRequest::new(
                    capture.subject(root).unwrap(),
                    HtmlSemanticQuery::Language,
                )],
                surgeist_html::HtmlValueInterpreter::new(32768),
                HtmlSemanticLimits::new(1_000_000, 16, 1024, 256, 1_000_000, 1024),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
        let HtmlSemanticFact::Language(HtmlSemanticValue::Known(value)) =
            report.results()[0].fact()
        else {
            panic!("actual established language must remain known");
        };
        assert_eq!(value.tag(), &DocumentString::from("MiXeD-private"));
        assert_eq!(
            value.origin(),
            &HtmlLanguageOrigin::Pragma(capture.subject(document.node()).unwrap())
        );
    }
}

#[test]
fn clone_and_native_install_process_copied_attributes_in_actual_destination() {
    let (mut dom, first, first_root, meta) = fixture();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "historical");
    batch.insert_before(meta, first_root, None);
    let second = batch.create_document();
    let root = batch.create_element_ns(&second, Namespace::html(), "head");
    batch.insert_before(&root, &second, None);
    let commit = apply(&mut dom, batch);
    let second = commit.document(&second).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "content", "copied");
    apply(&mut dom, batch);
    let captured = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.destroy(meta);
    let clone = batch.import_node(&captured, meta, second, false);
    let commit = apply(&mut dom, batch);
    let clone = commit.created(&clone).unwrap().unwrap();
    assert_ne!(clone, meta);
    assert_eq!(language(&dom.snapshot(), first), Some(&"historical".into()));
    assert_eq!(language(&dom.snapshot(), second), None);
    let mut batch = DomBatch::new();
    batch.insert_before(clone, root, None);
    apply(&mut dom, batch);
    assert_eq!(language(&dom.snapshot(), second), Some(&"copied".into()));

    let mut builder = surgeist_html::HtmlFragment::builder(
        surgeist_html::HtmlConstructionLimits::new(16, 16, 16, 1024, 8),
    )
    .unwrap();
    builder
        .append_element(
            &builder.root(),
            QualifiedName::new(Namespace::html(), None, "meta"),
            OrderedAttributes::new([
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "http-equiv"),
                    "content-language",
                ),
                Attribute::new(
                    QualifiedName::new(Namespace::none(), None, "content"),
                    "installed",
                ),
            ])
            .unwrap(),
        )
        .unwrap();
    let fragment = builder.finish();
    let source = fragment.root().children().next().unwrap();
    let stamp = dom.snapshot().document(second).unwrap().stamp();
    let report = dom
        .install_native_html_fragment(
            stamp,
            root,
            None,
            &fragment,
            DomNativeHtmlInstallLimits::new(16, 100_000, 131072),
            &cancel(),
        )
        .unwrap();
    assert!(report.node_for(&source).is_some());
    assert_eq!(
        language(report.snapshot(), second),
        Some(&"installed".into())
    );
    assert_eq!(language(&captured, first), Some(&"historical".into()));
    assert_eq!(language(&captured, second), None);
}

#[test]
fn language_history_retention_is_independent_from_attribute_units() {
    let (mut dom, document, root, meta) = fixture();
    let raw = DocumentString::from_utf16(vec![32, 0xd800, 65, 32, 66]);
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", raw);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let base_units = dom.usage().string_code_units();
    let bounded = |units| {
        DomLimits::new(128, units, 512, 128, 64, 8, 262144)
            .unwrap()
            .with_meta_processing_work(1_000_000)
    };
    dom.set_limits(bounded(base_units + 1)).unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(meta, root, None);
    assert!(
        matches!(dom.apply(batch, &cancel()), Err(DomError::ResourceLimit { resource: DomResource::StringCodeUnits, limit }) if limit == base_units + 1)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(bounded(base_units + 2)).unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(meta, root, None);
    let commit = apply(&mut dom, batch);
    assert_eq!(
        language(&dom.snapshot(), document).unwrap().code_units(),
        [0xd800, 65]
    );
    assert_eq!(dom.usage().string_code_units(), base_units + 2);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::DocumentMeta(document))
    );
    assert_eq!(language(&before, document), None);
}

#[test]
fn every_work_cut_before_actual_completion_rolls_back_history_and_tree() {
    let mut exact = None;
    for allowance in 0..4096 {
        let (mut dom, document, root, meta) = fixture();
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "http-equiv", "content-language");
        batch.set_attribute(meta, "content", "admitted");
        apply(&mut dom, batch);
        let before = dom.snapshot();
        let usage = dom.usage();
        dom.set_limits(limits(allowance)).unwrap();
        let mut batch = DomBatch::new();
        batch.insert_before(meta, root, None);
        match dom.apply(batch, &cancel()) {
            Ok(_) => {
                assert_eq!(
                    language(&dom.snapshot(), document),
                    Some(&"admitted".into())
                );
                assert_eq!(dom.snapshot().node(meta).unwrap().parent(), Some(root));
                exact = Some(allowance);
                break;
            }
            Err(DomError::Meta(DomMetaError::WorkLimit { limit })) => {
                assert_eq!(limit, allowance);
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(language(&dom.snapshot(), document), None);
                assert_eq!(dom.snapshot().node(meta).unwrap().parent(), None);
            }
            Err(error) => panic!("unexpected refusal: {error:?}"),
        }
    }
    assert!(
        exact.is_some_and(|work| work > 0),
        "finite real operation must complete"
    );
}

#[test]
fn ordinary_document_and_actual_head_guards_survive_shadow_connection_and_moves() {
    let (mut dom, document, root, meta) = fixture();
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(&host, root, None);
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            false,
            false,
        ),
    );
    batch.set_attribute(meta, "http-equiv", "content-language");
    batch.set_attribute(meta, "content", "shadow-only");
    batch.insert_before(meta, &shadow, None);
    let commit = apply(&mut dom, batch);
    let host = commit.created(&host).unwrap().unwrap();
    assert!(dom.snapshot().node(meta).unwrap().is_connected());
    assert_eq!(language(&dom.snapshot(), document), None);

    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "http-equiv", "content-security-policy");
    batch.set_attribute(meta, "content", "default-src 'none'");
    batch.insert_before(meta, host, None);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(meta, root, None);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::Meta(DomMetaError::ContentSecurityPolicyUnsupported)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().node(meta).unwrap().parent(), Some(host));
}

#[test]
fn subtree_insertion_runs_each_real_pragma_step_and_removal_does_not_recompute() {
    let (mut dom, document, root, outer) = fixture();
    let mut batch = DomBatch::new();
    let inner = batch.create_element_ns(document, Namespace::html(), "meta");
    for (node, value) in [
        (NodeOperand::from(outer), "outer"),
        (NodeOperand::from(&inner), "inner"),
    ] {
        batch.set_attribute(node.clone(), "http-equiv", "content-language");
        batch.set_attribute(node, "content", value);
    }
    // DOM does not enforce HTML authoring content models for this real subtree.
    batch.insert_before(&inner, outer, None);
    batch.insert_before(outer, root, None);
    let commit = apply(&mut dom, batch);
    let inner = commit.created(&inner).unwrap().unwrap();
    let first = dom.snapshot();
    assert_eq!(language(&first, document), Some(&"inner".into()));
    let mut batch = DomBatch::new();
    batch.detach(inner);
    batch.destroy(inner);
    apply(&mut dom, batch);
    assert_eq!(language(&dom.snapshot(), document), Some(&"inner".into()));
    let mut batch = DomBatch::new();
    batch.detach(outer);
    batch.insert_before(outer, root, None);
    apply(&mut dom, batch);
    assert_eq!(language(&dom.snapshot(), document), Some(&"outer".into()));
    assert_eq!(language(&first, document), Some(&"inner".into()));
}

fn url_limits(work: u64) -> DomLimits {
    limits(1_000_000)
        .with_base_processing(128, 1_000_000)
        .unwrap()
        .with_encoding_url_processing(128, work)
        .unwrap()
}
fn url_revision() -> HtmlBaseFactsRevision {
    HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3)
}
fn url_record(
    revision: &HtmlBaseFactsRevision,
    raw: &str,
    fragment_delimiter_offset: Option<usize>,
) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        raw.into(),
        HtmlBaseUrlScheme::Other,
        raw == "about:blank",
        fragment_delimiter_offset,
    )
    .unwrap()
}
fn url_environment(
    revision: &HtmlBaseFactsRevision,
    encoding: &HtmlBaseEncoding,
    fallback: &HtmlBaseUrlRecord,
    observations: Vec<HtmlBaseParseObservation>,
) -> HtmlParseBaseFacts {
    HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision.clone(),
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
fn bind_url_environment(dom: &mut Dom, document: DocumentId, facts: &HtmlParseBaseFacts) {
    let mut batch = DomBatch::new();
    batch.bind_base_facts(dom.snapshot().document(document).unwrap().stamp(), facts);
    apply(dom, batch);
}
fn url_facts(
    base: &HtmlBaseUrlRecord,
    encoding: &HtmlBaseEncoding,
    revision: &HtmlBaseFactsRevision,
    observations: Vec<HtmlEncodingUrlParseObservation>,
) -> HtmlEncodingUrlParseFacts {
    HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        observations,
        HtmlBaseFactsLimits::new(128, 65536, 1_000_000),
    )
    .unwrap()
}
fn bind_urls(
    dom: &mut Dom,
    document: DocumentId,
    facts: &HtmlEncodingUrlParseFacts,
) -> Result<DomCommit, DomError> {
    let mut batch = DomBatch::new();
    batch.bind_encoding_url_facts(dom.snapshot().document(document).unwrap().stamp(), facts);
    dom.apply(batch, &cancel())
}
fn refresh_batch(meta: NodeId, root: NodeId, raw: DocumentString) -> DomBatch {
    let mut batch = DomBatch::new();
    batch.detach(meta);
    batch.set_attribute(meta, "http-equiv", "refresh");
    batch.set_attribute(meta, "content", raw);
    batch.insert_before(meta, root, None);
    batch
}
fn refreshing(dom: &Dom, document: DocumentId) -> bool {
    dom.snapshot()
        .document(document)
        .unwrap()
        .metadata()
        .meta_state()
        .unwrap()
        .will_declaratively_refresh()
}

#[test]
fn refresh_url_substring_consumes_actual_failed_javascript_and_successful_observations() {
    let (mut dom, document, root, meta) = fixture();
    dom.set_limits(url_limits(1_000_000)).unwrap();
    let revision = url_revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = url_record(&revision, "about:blank", None);
    bind_url_environment(
        &mut dom,
        document,
        &url_environment(&revision, &encoding, &fallback, vec![]),
    );
    let script = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "javascript:0".into(),
        HtmlBaseUrlScheme::Javascript,
        false,
        None,
    )
    .unwrap();
    let target = url_record(&revision, "https://refresh.test/%EF%BF%BDx%EF%BF%BD", None);
    let mut raw_units = "https://refresh.test/".encode_utf16().collect::<Vec<_>>();
    raw_units.extend_from_slice(&[0xd800, 120, 0xdc00]);
    let raw = DocumentString::from_utf16(raw_units);
    let facts = url_facts(
        &fallback,
        &encoding,
        &revision,
        vec![
            HtmlEncodingUrlParseObservation::new(
                "https://[".into(),
                HtmlEncodingUrlParseOutcome::Failure,
            ),
            HtmlEncodingUrlParseObservation::new(
                "javascript:0".into(),
                HtmlEncodingUrlParseOutcome::Parsed(script),
            ),
            HtmlEncodingUrlParseObservation::new(
                raw.clone(),
                HtmlEncodingUrlParseOutcome::Parsed(target),
            ),
        ],
    );
    bind_urls(&mut dom, document, &facts).unwrap();
    let before = dom.snapshot();
    for content in ["0; URL='https://['", "2.4;javascript:0"] {
        apply(&mut dom, refresh_batch(meta, root, content.into()));
        assert!(!refreshing(&dom, document));
    }
    let mut content = "9; URL='".encode_utf16().collect::<Vec<_>>();
    content.extend_from_slice(raw.code_units());
    content.extend("'ignored".encode_utf16());
    let commit = apply(
        &mut dom,
        refresh_batch(meta, root, DocumentString::from_utf16(content)),
    );
    assert!(refreshing(&dom, document));
    assert!(
        commit
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::DocumentMeta(id) if *id == document))
    );
    assert!(
        !before
            .document(document)
            .unwrap()
            .metadata()
            .meta_state()
            .unwrap()
            .will_declaratively_refresh()
    );
    // The source's already-true flag precedes even a missing lookup and its allowance.
    dom.set_limits(url_limits(0)).unwrap();
    apply(&mut dom, refresh_batch(meta, root, "0; unobserved".into()));
    assert!(refreshing(&dom, document));
}

#[test]
fn refresh_lookup_missing_and_rebound_full_revision_refuse_atomically() {
    let (mut dom, document, root, meta) = fixture();
    dom.set_limits(url_limits(1_000_000)).unwrap();
    let revision = url_revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = url_record(&revision, "about:blank", None);
    bind_url_environment(
        &mut dom,
        document,
        &url_environment(&revision, &encoding, &fallback, vec![]),
    );
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        dom.apply(
            refresh_batch(meta, root, "0; https://refresh.test/".into()),
            &cancel()
        )
        .unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::MissingFacts)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let facts = url_facts(
        &fallback,
        &encoding,
        &revision,
        vec![HtmlEncodingUrlParseObservation::new(
            "https://refresh.test/".into(),
            HtmlEncodingUrlParseOutcome::Parsed(url_record(
                &revision,
                "https://refresh.test/",
                None,
            )),
        )],
    );
    bind_urls(&mut dom, document, &facts).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        dom.apply(
            refresh_batch(meta, root, "0; https://unobserved.test/".into()),
            &cancel()
        )
        .unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
            HtmlEncodingUrlFactsError::MissingObservation
        ))
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    let newer = HtmlBaseFactsRevision::new(revision.producer().clone(), 1, 2, 4);
    let newer_fallback = url_record(&newer, "about:blank", None);
    bind_url_environment(
        &mut dom,
        document,
        &url_environment(&newer, &encoding, &newer_fallback, vec![]),
    );
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(
        dom.apply(
            refresh_batch(meta, root, "0; https://refresh.test/".into()),
            &cancel()
        )
        .unwrap_err(),
        DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(
            HtmlEncodingUrlFactsError::RevisionMismatch
        ))
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(!refreshing(&dom, document));
}

#[test]
fn refresh_uses_current_first_base_and_refuses_foreign_encoding_and_equal_spelling_substitutes() {
    let (mut dom, document, root, meta) = fixture();
    dom.set_limits(url_limits(1_000_000)).unwrap();
    let revision = url_revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = url_record(&revision, "about:blank", None);
    let selected = url_record(&revision, "https://base.test/", None);
    let observation = HtmlBaseParseObservation::new(
        "base".into(),
        fallback.clone(),
        encoding.clone(),
        revision.clone(),
        HtmlBaseParseOutcome::Parsed {
            record: selected.clone(),
            base_allowed: HtmlBaseAllowed::Allowed,
        },
    )
    .unwrap();
    bind_url_environment(
        &mut dom,
        document,
        &url_environment(&revision, &encoding, &fallback, vec![observation]),
    );
    let mut batch = DomBatch::new();
    let base = batch.create_element_ns(document, Namespace::html(), "base");
    batch.set_attribute(&base, "href", "base");
    batch.insert_before(&base, root, None);
    apply(&mut dom, batch);
    for (base, encoding, revision, expected) in [
        (
            fallback.clone(),
            encoding.clone(),
            revision.clone(),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            url_record(&revision, "https://base.test/", None),
            encoding.clone(),
            revision.clone(),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            selected.clone(),
            HtmlBaseEncoding::fresh(),
            revision.clone(),
            HtmlEncodingUrlFactsError::InputsMismatch,
        ),
        (
            selected.clone(),
            encoding.clone(),
            url_revision(),
            HtmlEncodingUrlFactsError::ForeignProducer,
        ),
        (
            selected.clone(),
            encoding.clone(),
            HtmlBaseFactsRevision::new(revision.producer().clone(), 1, 99, 3),
            HtmlEncodingUrlFactsError::RevisionMismatch,
        ),
    ] {
        let facts = url_facts(&base, &encoding, &revision, vec![]);
        let before = dom.snapshot();
        let usage = dom.usage();
        assert_eq!(
            bind_urls(&mut dom, document, &facts).unwrap_err(),
            DomError::EncodingUrlFacts(DomEncodingUrlFactsError::Producer(expected))
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    let facts = url_facts(
        &selected,
        &encoding,
        &revision,
        vec![HtmlEncodingUrlParseObservation::new(
            "target".into(),
            HtmlEncodingUrlParseOutcome::Parsed(url_record(
                &revision,
                "https://base.test/target",
                None,
            )),
        )],
    );
    bind_urls(&mut dom, document, &facts).unwrap();
    apply(&mut dom, refresh_batch(meta, root, "0; target".into()));
    assert!(refreshing(&dom, document));
}

#[test]
fn refresh_and_hyperlink_spend_one_cumulative_bank_allowance_and_rollback_at_every_cut() {
    let mut minima = Vec::new();
    for mode in 0..3 {
        let (mut dom, document, root, meta) = fixture();
        dom.set_limits(url_limits(1_000_000)).unwrap();
        let revision = url_revision();
        let encoding = HtmlBaseEncoding::fresh();
        let fallback = url_record(&revision, "about:blank", None);
        bind_url_environment(
            &mut dom,
            document,
            &url_environment(&revision, &encoding, &fallback, vec![]),
        );
        let facts = url_facts(
            &fallback,
            &encoding,
            &revision,
            vec![HtmlEncodingUrlParseObservation::new(
                "https://refresh.test/".into(),
                HtmlEncodingUrlParseOutcome::Parsed(url_record(
                    &revision,
                    "https://refresh.test/",
                    None,
                )),
            )],
        );
        bind_urls(&mut dom, document, &facts).unwrap();
        let mut batch = DomBatch::new();
        let anchor = batch.create_element_ns(document, Namespace::html(), "a");
        let commit = apply(&mut dom, batch);
        let anchor = commit.created(&anchor).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let operation = || {
            let mut batch = if mode == 0 {
                DomBatch::new()
            } else {
                refresh_batch(meta, root, "0; https://refresh.test/".into())
            };
            if mode != 1 {
                batch.set_attribute(anchor, "href", "https://refresh.test/");
            }
            batch
        };
        assert_eq!(
            dom.apply(operation(), &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        let mut completed = None;
        for work in 0..512 {
            dom.set_limits(url_limits(work)).unwrap();
            match dom.apply(operation(), &cancel()) {
                Ok(_) => {
                    completed = Some(work);
                    break;
                }
                Err(error) => {
                    assert_eq!(error, DomError::EncodingUrlWorkLimit { limit: work });
                    assert_eq!(dom.snapshot().stamp(), before.stamp());
                    assert_eq!(dom.usage(), usage);
                    assert!(!refreshing(&dom, document));
                    assert!(
                        dom.snapshot()
                            .node(anchor)
                            .unwrap()
                            .as_hyperlink()
                            .unwrap()
                            .url()
                            .is_none()
                    );
                }
            }
        }
        minima.push(
            completed.expect("the finite source operation must complete within 512 admitted steps"),
        );
        assert_eq!(refreshing(&dom, document), mode != 0);
        assert_eq!(
            dom.snapshot()
                .node(anchor)
                .unwrap()
                .as_hyperlink()
                .unwrap()
                .url()
                .is_some(),
            mode != 1
        );
    }
    assert!(minima[2] > minima[0]);
    assert!(minima[2] > minima[1]);
}

#[test]
fn successful_url_refresh_is_private_until_all_later_effects_complete() {
    let (mut dom, document, root, meta) = fixture();
    dom.set_limits(url_limits(1_000_000)).unwrap();
    let revision = url_revision();
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = url_record(&revision, "about:blank", None);
    bind_url_environment(
        &mut dom,
        document,
        &url_environment(&revision, &encoding, &fallback, vec![]),
    );
    let facts = url_facts(
        &fallback,
        &encoding,
        &revision,
        vec![HtmlEncodingUrlParseObservation::new(
            "https://refresh.test/".into(),
            HtmlEncodingUrlParseOutcome::Parsed(url_record(
                &revision,
                "https://refresh.test/",
                None,
            )),
        )],
    );
    bind_urls(&mut dom, document, &facts).unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = refresh_batch(meta, root, "0; https://refresh.test/".into());
    let csp = batch.create_element_ns(document, Namespace::html(), "meta");
    batch.set_attribute(&csp, "http-equiv", "content-security-policy");
    batch.set_attribute(&csp, "content", "default-src 'none'");
    batch.insert_before(&csp, root, None);
    assert_eq!(
        dom.apply(batch, &cancel()).unwrap_err(),
        DomError::Meta(DomMetaError::ContentSecurityPolicyUnsupported)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(!refreshing(&dom, document));
    assert!(dom.snapshot().node(meta).unwrap().parent().is_none());
    apply(
        &mut dom,
        refresh_batch(meta, root, "0; https://refresh.test/".into()),
    );
    assert!(refreshing(&dom, document));
}

#[test]
fn source_phase_admits_language_token_units_and_keeps_history_atomic_at_the_actual_boundary() {
    fn attempt(allowance: usize) -> bool {
        let (mut dom, document, root, meta) = fixture();
        let old = DocumentString::from("a".repeat(64));
        let new = DocumentString::from("b".repeat(128));
        let mut batch = DomBatch::new();
        batch.set_attribute(meta, "http-equiv", "content-language");
        batch.set_attribute(meta, "content", old.clone());
        batch.insert_before(meta, root, None);
        apply(&mut dom, batch);
        let mut batch = DomBatch::new();
        batch.detach(meta);
        batch.set_attribute(meta, "content", new.clone());
        apply(&mut dom, batch);
        let before = dom.snapshot();
        let usage = dom.usage();
        let edit = DomSourceEdit::insert_before(meta, root, None);
        let result = {
            match dom.begin_source_edit(
                edit,
                DomSourceLimits::new(16_000_000, allowance, 10_000_000, 64, 4096, 4096, 1),
                &cancel(),
            ) {
                Err(error) => Err(error),
                Ok(mut lease) => match lease.advance(&cancel()) {
                    Ok(DomSourceProgress::Complete(_)) => Ok(()),
                    Ok(other) => panic!("Meta requires no external Style request: {other:?}"),
                    Err(error) => Err(error),
                },
            }
        };
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(language(&before, document), Some(&old));
        match result {
            Ok(()) => {
                assert_eq!(language(&dom.snapshot(), document), Some(&new));
                assert_eq!(dom.snapshot().node(meta).unwrap().parent(), Some(root));
                true
            }
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        DomError::ResourceLimit {
                            resource: DomResource::SourceRetainedUnits,
                            ..
                        }
                    ),
                    "unexpected failure: {error:?}"
                );
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(language(&dom.snapshot(), document), Some(&old));
                assert_eq!(dom.snapshot().node(meta).unwrap().parent(), None);
                false
            }
        }
    }
    let mut low = 0;
    let mut high = 131_072;
    assert!(attempt(high));
    while low < high {
        let allowance = low + (high - low) / 2;
        if attempt(allowance) {
            high = allowance;
        } else {
            low = allowance + 1;
        }
    }
    assert!(low > 0);
    assert!(!attempt(low - 1));
    assert!(attempt(low));
}

#[test]
fn local_css_theme_color_finite_literal_without_media_completes_native_insertion() {
    let (mut dom, document, root, meta) = fixture();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "name", "theme-color");
    batch.set_attribute(meta, "content", "  #123abc  ");
    batch.insert_before(meta, root, None);
    // HTML877d raw11466–11500 requires actual local color parsing even with
    // the native constructor's sealed empty set of affected Meta UI consumers.
    // No media attribute introduces an environment-matching dependency here.
    let result = dom.apply(batch, &cancel());
    assert!(
        result.is_ok(),
        "available finite local CSS color must complete: {result:?}"
    );
    let current = dom.snapshot();
    assert_ne!(current.stamp(), before.stamp());
    assert_ne!(
        current.document(document).unwrap().stamp(),
        before.document(document).unwrap().stamp()
    );
    assert_eq!(current.node(meta).unwrap().parent(), Some(root));
    assert_eq!(current.node(meta).unwrap().owner_document(), Some(document));
    assert_eq!(current.node(root).unwrap().children(), &[meta]);
    assert_eq!(
        current.node(meta).unwrap().attribute(&"name".into()),
        Some(&"theme-color".into())
    );
    assert_eq!(
        current.node(meta).unwrap().attribute(&"content".into()),
        Some(&"  #123abc  ".into())
    );
    assert_eq!(current.node(meta).unwrap().attribute(&"media".into()), None);
    assert_eq!(current.incomplete_source_operation(), None);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(before.node(meta).unwrap().parent(), None);
    assert_eq!(
        before.node(meta).unwrap().attribute(&"content".into()),
        None
    );
}

#[test]
fn local_css_color_scheme_keyword_list_completes_native_insertion() {
    let (mut dom, document, root, meta) = fixture();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(meta, "name", "color-scheme");
    batch.set_attribute(meta, "content", "light dark");
    batch.insert_before(meta, root, None);
    // HTML877d raw11514–11539 requires the CSS-owned component-value/property
    // grammar. This finite keyword list needs no host or rendering operation.
    let result = dom.apply(batch, &cancel());
    assert!(
        result.is_ok(),
        "available local CSS keyword list must complete: {result:?}"
    );
    let current = dom.snapshot();
    assert_ne!(current.stamp(), before.stamp());
    assert_ne!(
        current.document(document).unwrap().stamp(),
        before.document(document).unwrap().stamp()
    );
    assert_eq!(current.node(meta).unwrap().parent(), Some(root));
    assert_eq!(current.node(meta).unwrap().owner_document(), Some(document));
    assert_eq!(current.node(root).unwrap().children(), &[meta]);
    assert_eq!(
        current.node(meta).unwrap().attribute(&"name".into()),
        Some(&"color-scheme".into())
    );
    assert_eq!(
        current.node(meta).unwrap().attribute(&"content".into()),
        Some(&"light dark".into())
    );
    assert_eq!(current.incomplete_source_operation(), None);
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(before.node(meta).unwrap().parent(), None);
    assert_eq!(
        before.node(meta).unwrap().attribute(&"content".into()),
        None
    );
}
