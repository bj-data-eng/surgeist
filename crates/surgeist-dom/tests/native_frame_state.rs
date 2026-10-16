//! HTML877d Frame null child-content, source guards and canonical lifecycle.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomError, DomHistoryError, DomLimits,
    DomResource, DomSnapshot, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    Namespace, NodeId,
};
use surgeist_html::{HtmlBrowsingContextPresence, HtmlFrameNullContent};

const HTML: &str = "http://www.w3.org/1999/xhtml";

fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 16384, 256, 0, 0, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn content(snapshot: &DomSnapshot, node: NodeId) -> HtmlFrameNullContent {
    let value = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .frame_null_content()
        .unwrap();
    assert!(value.content_navigable_is_null());
    assert!(value.content_document_is_null());
    assert!(value.content_window_is_null());
    assert!(value.potentially_delays_load());
    assert!(!value.delays_load());
    value
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let frame = batch.create_element_ns(&document, Namespace::from_uri(HTML), "frame");
    let commit = apply(dom, batch);
    (
        commit.document(&document).unwrap(),
        commit.created(&frame).unwrap().unwrap(),
    )
}

#[test]
fn exact_frame_family_names_and_prefixes_preserve_native_null_content() {
    let mut dom = dom();
    let (document, frame) = setup(&mut dom);
    content(&dom.snapshot(), frame);
    let mut batch = DomBatch::new();
    let prefixed = batch.create_element_ns(document, Namespace::from_uri(HTML), "p:frame");
    let ordinary = [
        (Namespace::none(), "frame"),
        (Namespace::from_uri(HTML), "Frame"),
        (Namespace::from_uri("http://www.w3.org/2000/svg"), "frame"),
    ];
    let ordinary: Vec<_> = ordinary
        .into_iter()
        .map(|(namespace, name)| batch.create_element_ns(document, namespace, name))
        .collect();
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let prefixed = commit.created(&prefixed).unwrap().unwrap();
    content(&snapshot, prefixed);
    assert_eq!(
        snapshot.node(prefixed).unwrap().node_name(),
        "p:frame".into()
    );
    for ticket in ordinary {
        let node = snapshot
            .node(commit.created(&ticket).unwrap().unwrap())
            .unwrap();
        assert!(node.as_element().unwrap().frame_null_content().is_none());
    }
}

#[test]
fn connected_frame_src_updates_and_removal_keep_actual_native_null_content() {
    let mut dom = dom();
    let (document, frame) = setup(&mut dom);
    let detached = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(frame, document, None);
    apply(&mut dom, batch);
    let connected = dom.snapshot();
    assert!(connected.node(frame).unwrap().is_connected());
    let environment = connected
        .document(document)
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(
        environment.browsing_context(),
        HtmlBrowsingContextPresence::Absent
    );
    assert!(!environment.has_ever_had_navigable());
    // No URL/UTF-8 interpretation belongs to Frame's actual null-content src
    // branch. Invalid surrogate units remain the exact stored content attribute.
    let raw = DocumentString::from_utf16(vec![0xd800, 0x3a, 0xdc00]);
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(frame, Namespace::none(), "src", raw.clone());
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.changes().changes(), &[DomChange::Attributes(frame)]);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot
            .node(frame)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"src".into()),
        Some(&raw)
    );
    content(&snapshot, frame);
    let mut batch = DomBatch::new();
    batch.set_attribute_ns(frame, Namespace::none(), "src", raw);
    let before_repeat = dom.snapshot().stamp();
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before_repeat);
    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(frame, Namespace::none(), "src");
    batch.detach(frame);
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(current.node(frame).unwrap().parent(), None);
    assert!(!current.node(frame).unwrap().is_connected());
    assert_eq!(content(&current, frame), content(&connected, frame));
    assert!(!detached.node(frame).unwrap().is_connected());
    assert!(connected.node(frame).unwrap().is_connected());
    assert!(
        connected
            .node(frame)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
}

#[test]
fn actual_attribute_expanded_identity_and_current_document_casing_remain_exact() {
    let mut dom = dom();
    let (xml, frame) = setup(&mut dom);
    let mut batch = DomBatch::new();
    let html = batch.create_html_document(xml, None);
    batch.set_attribute_ns(frame, Namespace::from_uri("urn:x"), "src", "foreign");
    batch.set_attribute(frame, "src", "foreign-updated");
    batch.set_attribute_ns(frame, Namespace::none(), "src", "native");
    batch.set_attribute_ns(frame, Namespace::none(), "SRC", "exact-uppercase");
    batch.set_attribute(frame, "p:src", "whole-local-name");
    let commit = apply(&mut dom, batch);
    let html = commit.document(&html).unwrap();
    let old = dom.snapshot();
    let old_element = old.node(frame).unwrap().as_element().unwrap();
    assert_eq!(
        old_element.attribute_ns(&Namespace::from_uri("urn:x"), &"src".into()),
        Some(&"foreign-updated".into())
    );
    assert_eq!(
        old_element.attribute_ns(&Namespace::none(), &"src".into()),
        Some(&"native".into())
    );
    assert_eq!(
        old_element.attribute_ns(&Namespace::none(), &"SRC".into()),
        Some(&"exact-uppercase".into())
    );
    let mut batch = DomBatch::new();
    batch.adopt(frame, html);
    batch.set_attribute(frame, "SRC", "html-folded-foreign-first");
    batch.remove_attribute(frame, "SRC"); // Removes the first qualified spelling.
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let element = snapshot.node(frame).unwrap().as_element().unwrap();
    assert!(
        element
            .attribute_ns(&Namespace::from_uri("urn:x"), &"src".into())
            .is_none()
    );
    assert_eq!(
        element.attribute_ns(&Namespace::none(), &"src".into()),
        Some(&"native".into())
    );
    assert_eq!(
        element.attribute_ns(&Namespace::none(), &"SRC".into()),
        Some(&"exact-uppercase".into())
    );
    assert_eq!(snapshot.node(frame).unwrap().owner_document(), Some(html));
    assert_eq!(content(&snapshot, frame), content(&old, frame));
    assert_eq!(old.node(frame).unwrap().owner_document(), Some(xml));
}

#[test]
fn fragment_transfer_replacement_and_adoption_preserve_frame_identity_and_state() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&source);
    let old_frame = batch.create_element_ns(&target, Namespace::from_uri(HTML), "frame");
    let moved = batch.create_element_ns(&source, Namespace::from_uri(HTML), "p:frame");
    let text = batch.create_text(&source, "child");
    batch.set_attribute(&moved, "src", "literal");
    batch.insert_before(&text, &moved, None);
    batch.insert_before(&moved, &fragment, None);
    batch.insert_before(&old_frame, &target, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let old_frame = commit.created(&old_frame).unwrap().unwrap();
    let moved = commit.created(&moved).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_child(fragment, target, old_frame);
    let change = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot.node(fragment).unwrap().owner_document(),
        Some(target)
    );
    assert!(snapshot.node(fragment).unwrap().children().is_empty());
    assert_eq!(snapshot.node(moved).unwrap().owner_document(), Some(target));
    assert_eq!(snapshot.node(text).unwrap().owner_document(), Some(target));
    assert_eq!(snapshot.node(old_frame).unwrap().parent(), None);
    assert_eq!(snapshot.node(moved).unwrap().parent(), Some(target.node()));
    assert!(
        change
            .changes()
            .changes()
            .contains(&DomChange::NodeDocument(moved))
    );
    assert!(
        !change
            .changes()
            .changes()
            .contains(&DomChange::ElementState(moved))
    );
    content(&snapshot, old_frame);
    assert_eq!(content(&snapshot, moved), content(&old, moved));
    assert_eq!(old.node(moved).unwrap().owner_document(), Some(source));
    let mut batch = DomBatch::new();
    batch.adopt(moved, source);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(moved).unwrap().parent(), None);
    assert_eq!(snapshot.node(moved).unwrap().owner_document(), Some(source));
    assert_eq!(snapshot.node(text).unwrap().owner_document(), Some(source));
    content(&snapshot, moved);
}

#[test]
fn same_store_clone_and_cross_store_import_use_the_actual_capture_and_fresh_null_content() {
    let mut source = dom();
    let (document, frame) = setup(&mut source);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "retained-child");
    batch.insert_before(&text, frame, None);
    batch.set_attribute(frame, "src", "old-src");
    batch.set_attribute(frame, "nonce", "content");
    batch.set_nonce(frame, "saved-slot");
    apply(&mut source, batch);
    let old = source.snapshot();
    let mut batch = DomBatch::new();
    let clone = batch.import_node(&old, frame, document, false);
    let commit = apply(&mut source, batch);
    let clone = commit.created(&clone).unwrap().unwrap();
    assert_ne!(clone, frame);
    assert!(source.snapshot().node(clone).unwrap().children().is_empty());
    content(&source.snapshot(), clone);
    let mut batch = DomBatch::new();
    batch.set_attribute(frame, "src", "new-live-src");
    batch.destroy(frame);
    apply(&mut source, batch);
    drop(source);
    let mut target = dom();
    let mut batch = DomBatch::new();
    let destination = batch.create_document();
    let imported = batch.import_node(&old, frame, &destination, true);
    let commit = apply(&mut target, batch);
    let imported = commit.created(&imported).unwrap().unwrap();
    let destination = commit.document(&destination).unwrap();
    let snapshot = target.snapshot();
    let node = snapshot.node(imported).unwrap();
    let element = node.as_element().unwrap();
    assert_eq!(element.attribute(&"src".into()), Some(&"old-src".into()));
    assert_eq!(element.nonce(), Some(&"saved-slot".into()));
    assert_eq!(node.owner_document(), Some(destination));
    assert_eq!(node.children().len(), 1);
    assert_eq!(
        snapshot.node(node.children()[0]).unwrap().data(),
        Some(&"retained-child".into())
    );
    content(&snapshot, imported);
    content(&old, frame);
}

#[test]
fn shadow_including_frame_hooks_follow_real_hosts_and_adoption_documents() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let destination = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let frame = batch.create_element_ns(&document, Namespace::from_uri(HTML), "frame");
    batch.set_attribute(&frame, "src", "never-navigated");
    batch.insert_before(&frame, &shadow, None);
    batch.insert_before(&host, &document, None);
    let commit = apply(&mut dom, batch);
    let destination = commit.document(&destination).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let frame = commit.created(&frame).unwrap().unwrap();
    let old = dom.snapshot();
    assert!(old.node(frame).unwrap().is_connected());
    let mut batch = DomBatch::new();
    batch.adopt(host, destination);
    batch.insert_before(host, destination, None);
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert!(current.node(frame).unwrap().is_connected());
    assert_eq!(
        current.node(frame).unwrap().owner_document(),
        Some(destination)
    );
    content(&current, frame);
    let mut batch = DomBatch::new();
    batch.detach(host);
    batch.destroy(host);
    apply(&mut dom, batch);
    assert!(matches!(
        dom.snapshot().node(frame),
        Err(DomError::StaleNode)
    ));
    content(&old, frame);
}

#[test]
fn exact_frame_storage_limits_and_failed_edits_publish_no_partial_family_or_identity() {
    // XML metadata31 + HTML namespace28 + local frame5; null family owns no strings.
    for units in [63, 64] {
        let mut dom = Dom::new(DomLimits::new(2, units, 8, 0, 0, 4, 8192).unwrap()).unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let frame = batch.create_element_ns(&document, Namespace::from_uri(HTML), "frame");
        let result = dom.apply(batch, &AtomicBool::new(false));
        if units == 63 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 63,
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let commit = result.unwrap();
            let frame = commit.created(&frame).unwrap().unwrap();
            assert_eq!(dom.usage().managed_nodes(), 2);
            assert_eq!(dom.usage().string_code_units(), 64);
            content(&dom.snapshot(), frame);
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut batch = DomBatch::new();
            batch.set_attribute(frame, "src", "x");
            assert_eq!(
                dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 64,
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert_eq!(content(&dom.snapshot(), frame), content(&before, frame));
            let mut batch = DomBatch::new();
            batch.destroy(frame);
            apply(&mut dom, batch);
            assert_eq!(dom.usage().string_code_units(), 31);
        }
    }
}

#[test]
fn late_hierarchy_errors_cancellation_and_history_preserve_published_frame_state() {
    for history in [0, 8] {
        let mut dom =
            Dom::new(DomLimits::new(16, 2048, 32, 0, 0, history, 65536).unwrap()).unwrap();
        let (document, frame) = setup(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.set_attribute(frame, "src", "unpublished");
        batch.insert_before(frame, document, None);
        batch.insert_before(document, frame, None);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::HierarchyRequest
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        let mut batch = DomBatch::new();
        batch.set_attribute(frame, "src", "cancelled");
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        let mut batch = DomBatch::new();
        batch.set_attribute(frame, "src", "published");
        batch.insert_before(frame, document, None);
        let commit = apply(&mut dom, batch);
        assert!(
            !commit
                .changes()
                .changes()
                .contains(&DomChange::ElementState(frame))
        );
        if history == 0 {
            assert!(matches!(
                dom.changes_since(before.stamp()),
                Err(DomHistoryError::FullRecomputeRequired { .. })
            ));
        } else {
            assert_eq!(
                dom.changes_since(before.stamp()).unwrap().changes(),
                commit.changes().changes()
            );
        }
        content(&dom.snapshot(), frame);
    }
}
