//! Functional native Dialog state and ordinary closed-hook contracts.
//! HTML877d Dialog fresh/returnValue and ordinary inactive hooks; not hosted UI.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(128, 8192, 128, 0, 0, 32, 1_000_000).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn setup(limits: DomLimits) -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(limits).unwrap();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let dialog = b.create_element_ns(&d, Namespace::html(), "dialog");
    let c = apply(&mut dom, b);
    (
        dom,
        c.document(&d).unwrap(),
        c.created(&dialog).unwrap().unwrap(),
    )
}
fn dialog(snapshot: &DomSnapshot, node: NodeId) -> DomDialog<'_> {
    snapshot.node(node).unwrap().as_dialog().unwrap()
}
fn closed_services(view: DomDialog<'_>) {
    assert!(!view.is_modal());
    assert!(!view.has_toggle_task_tracker());
    assert!(!view.has_close_watcher());
    assert_eq!(view.request_close_return_value(), None);
    assert_eq!(view.request_close_source(), None);
    assert!(!view.force_close_watcher_enabled());
}

#[test]
fn actual_fresh_tuple_and_expanded_name_view_are_independent_of_open_attribute() {
    let (mut dom, document, node) = setup(limits());
    let snapshot = dom.snapshot();
    let view = dialog(&snapshot, node);
    assert_eq!(view.id(), node);
    assert_eq!(view.return_value().code_units(), []);
    closed_services(view);
    assert!(
        !snapshot
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .has_previously_focused_element()
    );
    let mut b = DomBatch::new();
    b.set_attribute(node, "open", "");
    let svg = b.create_element_ns(document, Namespace::svg(), "dialog");
    let no_namespace = b.create_element_ns(document, Namespace::none(), "dialog");
    let upper = b.create_element_ns(document, Namespace::html(), "DIALOG");
    let commit = apply(&mut dom, b);
    let current = dom.snapshot();
    closed_services(dialog(&current, node));
    for ticket in [&svg, &no_namespace, &upper] {
        assert!(
            current
                .node(commit.created(ticket).unwrap().unwrap())
                .unwrap()
                .as_dialog()
                .is_none()
        );
    }
}

#[test]
fn return_value_preserves_exact_utf16_and_old_snapshot_then_equal_value_is_noop() {
    let (mut dom, _, node) = setup(limits());
    let old = dom.snapshot();
    let value = DocumentString::from_utf16(vec![0xd800, 0, 0x0d, 0x0a, 0xdc00]);
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, value.clone());
    let commit = apply(&mut dom, b);
    assert_eq!(commit.changes().changes(), &[DomChange::ElementState(node)]);
    let current = dom.snapshot();
    assert_eq!(
        dialog(&current, node).return_value().code_units(),
        [0xd800, 0, 0x0d, 0x0a, 0xdc00]
    );
    assert_eq!(dialog(&old, node).return_value().code_units(), []);
    closed_services(dialog(&current, node));
    let usage = dom.usage();
    let mut spare = Vec::with_capacity(4096);
    spare.extend_from_slice(value.code_units());
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, DocumentString::from_utf16(spare));
    let c = apply(&mut dom, b);
    assert!(c.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), current.stamp());
    assert_eq!(dom.usage(), usage);
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, "");
    apply(&mut dom, b);
    assert_eq!(
        dialog(&dom.snapshot(), node).return_value().code_units(),
        []
    );
    assert_eq!(dialog(&current, node).return_value(), &value);
}

#[test]
fn ordinary_inactive_open_hooks_insert_remove_equal_and_namespace_lookalikes_stay_closed() {
    let (mut dom, document, node) = setup(limits());
    let mut b = DomBatch::new();
    b.set_attribute(node, "open", "source");
    b.insert_before(node, document, None);
    b.set_attribute(node, "open", "source");
    b.set_attribute_ns(node, Namespace::from_uri("urn:test"), "x:open", "");
    b.set_dialog_return_value(node, "kept");
    apply(&mut dom, b);
    let open = dom.snapshot();
    closed_services(dialog(&open, node));
    assert!(open.node(node).unwrap().is_connected());
    let mut b = DomBatch::new();
    b.remove_attribute(node, "open");
    b.detach(node);
    b.insert_before(node, document, None);
    b.detach(node);
    apply(&mut dom, b);
    let closed = dom.snapshot();
    assert!(!closed.node(node).unwrap().is_connected());
    assert_eq!(
        dialog(&closed, node).return_value(),
        &DocumentString::from("kept")
    );
    closed_services(dialog(&closed, node));
    assert_eq!(dom.pending_details_toggle_intentions(), 0);
}

#[test]
fn actual_clone_is_fresh_while_adoption_and_historical_snapshots_keep_return_value() {
    let (mut dom, original_doc, node) = setup(limits());
    let mut b = DomBatch::new();
    b.set_attribute(node, "open", "");
    b.set_dialog_return_value(node, "original");
    b.insert_before(node, original_doc, None);
    let destination = b.create_document();
    let c = apply(&mut dom, b);
    let destination = c.document(&destination).unwrap();
    let old = dom.snapshot();
    let mut b = DomBatch::new();
    let cloned = b.import_node(&old, node, destination, true);
    b.adopt(node, destination);
    let c = apply(&mut dom, b);
    let cloned = c.created(&cloned).unwrap().unwrap();
    let current = dom.snapshot();
    assert_eq!(dialog(&current, cloned).return_value().code_units(), []);
    assert_eq!(
        dialog(&current, node).return_value(),
        &DocumentString::from("original")
    );
    assert_eq!(
        current.node(node).unwrap().owner_document(),
        Some(destination)
    );
    assert_eq!(old.node(node).unwrap().owner_document(), Some(original_doc));
    closed_services(dialog(&current, node));
    closed_services(dialog(&current, cloned));
}

#[test]
fn logical_string_exact_boundary_succeeds_and_one_more_rolls_back_every_edit() {
    let cap = 128;
    let (mut dom, _, node) = setup(DomLimits::new(16, cap, 32, 0, 0, 8, 65536).unwrap());
    let available = cap - dom.usage().string_code_units();
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, DocumentString::from_utf16(vec![65; available]));
    apply(&mut dom, b);
    assert_eq!(dom.usage().string_code_units(), cap);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, "short");
    b.set_dialog_return_value(node, DocumentString::from_utf16(vec![65; available + 1]));
    assert!(matches!(
        dom.apply(b, &AtomicBool::new(false)),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dialog(&dom.snapshot(), node).return_value().len(),
        available
    );
}

#[test]
fn wrong_kind_foreign_stale_and_cancellation_never_publish_a_partial_return_value() {
    let (mut dom, document, node) = setup(limits());
    let mut b = DomBatch::new();
    let text = b.create_text(document, "text");
    let c = apply(&mut dom, b);
    let text = c.created(&text).unwrap().unwrap();
    let (foreign, _, other) = setup(limits());
    drop(foreign);
    for (target, expected) in [
        (text, DomError::DialogInapplicable),
        (other, DomError::ForeignOwner),
    ] {
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_dialog_return_value(node, "partial");
        b.set_dialog_return_value(target, "invalid");
        assert_eq!(dom.apply(b, &AtomicBool::new(false)).unwrap_err(), expected);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, "canceled");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut b = DomBatch::new();
    b.destroy(node);
    apply(&mut dom, b);
    let after = dom.snapshot();
    let mut b = DomBatch::new();
    let _fresh = b.create_element_ns(document, Namespace::html(), "dialog");
    b.set_dialog_return_value(node, "stale");
    assert_eq!(
        dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
        DomError::StaleNode
    );
    assert_eq!(dom.snapshot().stamp(), after.stamp());
    assert_eq!(dialog(&before, node).return_value().code_units(), []);
}

#[test]
fn source_return_value_retains_spare_backing_and_cancellation_never_publishes() {
    let (mut dom, _, node) = setup(limits());
    let cancel = AtomicBool::new(false);
    let mut backing = Vec::with_capacity(4096);
    backing.push(0xd800);
    let value = DocumentString::from_utf16(backing);
    let before = dom.snapshot();
    let usage = dom.usage();
    let edit = DomSourceEdit::set_dialog_return_value(node, value.clone());
    let bounded = DomSourceLimits::new(16_000_000, 4095, 10_000_000, 16, 4096, 4096);
    assert!(matches!(
        dom.begin_source_edit(edit, bounded, &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_source_edit().is_none());

    let generous = DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 16, 4096, 4096);
    let edit = DomSourceEdit::set_dialog_return_value(node, value.clone());
    let mut lease = dom.begin_source_edit(edit, generous, &cancel).unwrap();
    assert_eq!(
        lease.advance(&AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    drop(lease);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(dom.pending_source_edit().is_none());

    let edit = DomSourceEdit::set_dialog_return_value(node, value);
    let mut lease = dom.begin_source_edit(edit, generous, &cancel).unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("closed returnValue setter needs no external service");
    };
    assert_eq!(
        done.commit().changes().changes(),
        &[DomChange::ElementState(node)]
    );
    drop(lease);
    let current = dom.snapshot();
    assert_eq!(dialog(&current, node).return_value().code_units(), [0xd800]);
    assert!(
        dialog(&current, node)
            .return_value()
            .shared_code_units()
            .capacity()
            >= 4096
    );
    assert_eq!(dialog(&before, node).return_value().code_units(), []);
    closed_services(dialog(&current, node));
}

#[test]
fn actual_parsed_document_history_supports_native_dialog_cleanup_without_resetting_old_snapshots() {
    use std::sync::Arc;
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlParseLimits, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme,
    };
    let (mut dom, original, node) = setup(limits());
    let snapshot = dom.snapshot();
    let origin = snapshot
        .document(original)
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let parse_limits = HtmlParseLimits::builder()
        .source_units(100)
        .buffered_utf16_units(100)
        .token_attributes(100)
        .auxiliary_entries(100_000)
        .managed_nodes(100)
        .managed_edges(100)
        .managed_attributes(100)
        .managed_utf16_units(1000)
        .source_map_entries(1000)
        .source_map_payload_units(1000)
        .diagnostics(100)
        .diagnostic_payload_units(1000)
        .context_effects(100)
        .open_elements(100)
        .tree_depth(100)
        .managed_capacity_bytes(1_000_000)
        .peak_transient_bytes(1_000_000)
        .atomic_layout_bytes(1_000_000)
        .work_credits(10_000_000)
        .finish()
        .unwrap();
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>source</p>"),
            parse_limits,
        )
        .parse()
        .unwrap(),
    );
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1);
    let url = HtmlBaseUrlRecord::observed(
        revision,
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let installed = dom
        .install_parsed_html_document(
            snapshot.stamp(),
            report,
            DomParsedDocumentEnvironment::headless(
                origin,
                url,
                HtmlBaseEncoding::fresh(),
                "UTF-8".into(),
                DomNativeInputPlatform::headless(),
            ),
            DomParsedHtmlInstallLimits::new(128, 10_000_000, 10_000_000, 10_000_000),
            &AtomicBool::new(false),
        )
        .unwrap();
    let parsed_document = installed.document();
    let mut b = DomBatch::new();
    b.adopt(node, parsed_document);
    // The real inactive guard precedes setup/list reads on open addition.
    b.set_attribute(node, "open", "");
    b.set_dialog_return_value(node, "retained");
    apply(&mut dom, b);
    let before = dom.snapshot();
    assert_eq!(
        before.node(node).unwrap().owner_document(),
        Some(parsed_document)
    );
    assert_eq!(
        dialog(&before, node).return_value(),
        &DocumentString::from("retained")
    );
    let mut b = DomBatch::new();
    b.set_dialog_return_value(node, "partial");
    // Removal must clean the actual list BEFORE any inactive return.
    b.remove_attribute(node, "open");
    apply(&mut dom, b);
    let current = dom.snapshot();
    assert_ne!(current.stamp(), before.stamp());
    assert_eq!(
        current.node(node).unwrap().owner_document(),
        Some(parsed_document)
    );
    assert_eq!(
        dialog(&current, node).return_value(),
        &DocumentString::from("partial")
    );
    assert_eq!(
        current
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("open")),
        None
    );
    closed_services(dialog(&current, node));
    assert_eq!(
        dialog(&before, node).return_value(),
        &DocumentString::from("retained")
    );
    assert!(
        before
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&DocumentString::from("open"))
            .is_some()
    );
}
