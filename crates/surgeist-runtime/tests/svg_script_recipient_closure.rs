//! Public Runtime registration must invalidate closed SVG notification suppression
//! at the actual canonical processing boundary. These cases do not construct or
//! dispatch SVG events and do not infer closure from an observed callback count.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomError, DomLimits, DomNameLimits, DomSnapshot,
    DomSvgScriptOrigin, DomUsage, Namespace, NodeId,
};
use surgeist_runtime::{
    DomEventCallback, DomEventDispatcher, DomEventDom, DomEventListenerOptions,
    DomEventRegistrationId, DomEventRegistryLimits, DomEventType,
};

type Recipients = DomEventDispatcher<Vec<DomEventType>>;

fn cancelled() -> AtomicBool {
    AtomicBool::new(false)
}

fn native_owner() -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            64,
            65_536,
            256,
            256,
            256,
            32,
            1_048_576,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancelled()).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    drop(commit);
    assert!(dom.snapshot().node(root).unwrap().is_connected());
    (dom, document, root)
}

fn recipients() -> Recipients {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 8,
        nested_dispatches: 2,
    })
    .unwrap()
}

fn register(
    dom: &mut Dom,
    recipients: &mut Recipients,
    target: NodeId,
    kind: DomEventType,
    capture: bool,
) -> DomEventRegistrationId {
    // The borrowed Runtime front validates the same canonical owner that will
    // later perform the ordinary edit.
    let front = DomEventDom::borrow(dom);
    recipients
        .add_listener(
            &front,
            target,
            kind,
            DomEventCallback::new(|_, _, observed: &mut Vec<DomEventType>, event| {
                observed.push(event.event_type());
                Ok(())
            }),
            DomEventListenerOptions {
                capture,
                once: false,
            },
        )
        .unwrap()
}

fn detached_script(dom: &mut Dom, document: DocumentId, parent: Option<NodeId>) -> NodeId {
    let mut batch = DomBatch::new();
    let script = batch.create_element_ns(document, Namespace::svg(), "script");
    if let Some(parent) = parent {
        batch.insert_before(&script, parent, None);
    }
    let commit = dom.apply(batch, &cancelled()).unwrap();
    let script = commit.created(&script).unwrap().unwrap();
    drop(commit);
    unprocessed(&dom.snapshot(), script, document, parent);
    script
}

fn unprocessed(
    snapshot: &DomSnapshot,
    script: NodeId,
    document: DocumentId,
    parent: Option<NodeId>,
) {
    let node = snapshot.node(script).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert_eq!(node.parent(), parent);
    assert!(!node.is_connected());
    let state = node.as_svg_script().unwrap();
    assert_eq!(state.origin(), DomSvgScriptOrigin::Native);
    assert!(!state.flags().already_processed());
    assert_eq!(
        node.as_element().unwrap().name().namespace_uri(),
        Namespace::svg().uri()
    );
    assert_eq!(
        node.as_element().unwrap().name().local_name(),
        &DocumentString::from("script")
    );
}

fn unchanged(dom: &Dom, before: &DomSnapshot, usage: DomUsage, prefix_target: NodeId) {
    let current = dom.snapshot();
    assert_eq!(current.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    assert!(current.nodes().eq(before.nodes()));
    for id in before.nodes() {
        let original = before.node(id).unwrap();
        let retained = current.node(id).unwrap();
        assert_eq!(retained.parent(), original.parent());
        assert_eq!(retained.children(), original.children());
        assert_eq!(retained.owner_document(), original.owner_document());
    }
    for snapshot in [&current, before] {
        assert!(
            snapshot
                .node(prefix_target)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&DocumentString::from("data-unpublished"))
                .is_none()
        );
    }
}

fn direct_recipient(kind: DomEventType) {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut recipients = recipients();
    let registration = register(&mut dom, &mut recipients, script, kind, false);
    let before = dom.snapshot();
    let usage = dom.usage();
    unprocessed(&before, script, document, None);

    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-unpublished", "must-roll-back");
    batch.insert_before(script, root, None);
    // Ordinary DOM edits after releasing DomEventDom must still see the actual
    // Runtime registration. No convenience Source/event driver is involved.
    assert_eq!(
        dom.apply(batch, &cancelled()).unwrap_err(),
        DomError::SvgScriptProcessingUnavailable
    );
    unchanged(&dom, &before, usage, root);
    unprocessed(&before, script, document, None);
    unprocessed(&dom.snapshot(), script, document, None);
    assert!(recipients.remove_listener(registration));
}

#[derive(Clone, Copy)]
enum CaptureTarget {
    Ancestor,
    Document,
}

fn capture_before_creation(kind: DomEventType, target: CaptureTarget) {
    let (mut dom, document, root) = native_owner();
    let mut recipients = recipients();
    let target = match target {
        CaptureTarget::Ancestor => root,
        CaptureTarget::Document => document.node(),
    };
    // Capture is applicable for these nonbubbling events. In this ordinary
    // never-hosted tree both paths include Document; Load stops there.
    let registration = register(&mut dom, &mut recipients, target, kind, true);
    let script = detached_script(&mut dom, document, None);
    let before = dom.snapshot();
    let usage = dom.usage();

    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-unpublished", "must-roll-back");
    batch.insert_before(script, root, None);
    assert_eq!(
        dom.apply(batch, &cancelled()).unwrap_err(),
        DomError::SvgScriptProcessingUnavailable
    );
    unchanged(&dom, &before, usage, root);
    unprocessed(&before, script, document, None);
    unprocessed(&dom.snapshot(), script, document, None);
    assert!(recipients.remove_listener(registration));
}

fn move_under_registered_ancestor(kind: DomEventType) {
    let (mut dom, document, root) = native_owner();
    let mut batch = DomBatch::new();
    let ancestor = batch.create_element_ns(document, Namespace::html(), "section");
    let staging = batch.create_element_ns(document, Namespace::html(), "div");
    batch.insert_before(&ancestor, root, None);
    let commit = dom.apply(batch, &cancelled()).unwrap();
    let ancestor = commit.created(&ancestor).unwrap().unwrap();
    let staging = commit.created(&staging).unwrap().unwrap();
    drop(commit);
    let mut recipients = recipients();
    let registration = register(&mut dom, &mut recipients, ancestor, kind, true);
    // The staging parent is detached. A connected initial location would
    // already process the Script and would miss this false-bit transition.
    let script = detached_script(&mut dom, document, Some(staging));
    let before = dom.snapshot();
    let usage = dom.usage();
    assert_eq!(before.node(staging).unwrap().children(), &[script]);
    assert!(before.node(ancestor).unwrap().children().is_empty());
    assert!(before.node(ancestor).unwrap().is_connected());

    let mut batch = DomBatch::new();
    batch.set_attribute(staging, "data-unpublished", "must-roll-back");
    batch.insert_before(script, ancestor, None);
    assert_eq!(
        dom.apply(batch, &cancelled()).unwrap_err(),
        DomError::SvgScriptProcessingUnavailable
    );
    unchanged(&dom, &before, usage, staging);
    unprocessed(&before, script, document, Some(staging));
    unprocessed(&dom.snapshot(), script, document, Some(staging));
    assert_eq!(dom.snapshot().node(staging).unwrap().children(), &[script]);
    assert!(dom.snapshot().node(ancestor).unwrap().children().is_empty());
    assert!(recipients.remove_listener(registration));
}

#[test]
fn direct_load_recipient_refuses_connected_svg_processing_atomically() {
    direct_recipient(DomEventType::Load);
}

#[test]
fn direct_error_recipient_refuses_connected_svg_processing_atomically() {
    direct_recipient(DomEventType::Error);
}

#[test]
fn prior_ancestor_load_capture_refuses_later_svg_processing_atomically() {
    capture_before_creation(DomEventType::Load, CaptureTarget::Ancestor);
}

#[test]
fn prior_ancestor_error_capture_refuses_later_svg_processing_atomically() {
    capture_before_creation(DomEventType::Error, CaptureTarget::Ancestor);
}

#[test]
fn prior_document_load_capture_refuses_later_svg_processing_atomically() {
    capture_before_creation(DomEventType::Load, CaptureTarget::Document);
}

#[test]
fn prior_document_error_capture_refuses_later_svg_processing_atomically() {
    capture_before_creation(DomEventType::Error, CaptureTarget::Document);
}

#[test]
fn moving_unprocessed_svg_under_load_capture_refuses_atomically() {
    move_under_registered_ancestor(DomEventType::Load);
}

#[test]
fn moving_unprocessed_svg_under_error_capture_refuses_atomically() {
    move_under_registered_ancestor(DomEventType::Error);
}
