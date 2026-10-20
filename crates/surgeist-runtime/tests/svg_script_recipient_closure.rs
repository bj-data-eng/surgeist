//! Public Runtime registration must invalidate closed SVG notification suppression
//! at the actual canonical processing boundary. These cases do not construct or
//! dispatch SVG events and do not infer closure from an observed callback count.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomError, DomEventRecipientLimits, DomLimits,
    DomNameLimits, DomResource, DomSnapshot, DomSvgScriptOrigin, DomUsage, HtmlShadowRootMode,
    HtmlShadowRootProperties, HtmlSlotAssignment, Namespace, NodeId,
};
use surgeist_runtime::{
    DomEventCallback, DomEventDispatchError, DomEventDispatcher, DomEventDom,
    DomEventListenerOptions, DomEventRegistrationId, DomEventRegistryLimits, DomEventResource,
    DomEventType,
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
        .unwrap()
        .with_event_recipients(
            DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
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
    register_callback(
        dom,
        recipients,
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
}

fn register_callback(
    dom: &mut Dom,
    recipients: &mut Recipients,
    target: NodeId,
    kind: DomEventType,
    callback: DomEventCallback<Vec<DomEventType>>,
    options: DomEventListenerOptions,
) -> DomEventRegistrationId {
    // The borrowed Runtime front validates the same canonical owner that will
    // later perform the ordinary edit.
    let mut front = DomEventDom::borrow(dom);
    recipients
        .add_listener(&mut front, target, kind, callback, options)
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

fn refuse_insertion(dom: &mut Dom, script: NodeId, parent: NodeId, prefix: NodeId) {
    let before = dom.snapshot();
    let usage = dom.usage();
    assert!(
        !before
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(prefix, "data-unpublished", "must-roll-back");
    batch.insert_before(script, parent, None);
    assert_eq!(
        dom.apply(batch, &cancelled()).unwrap_err(),
        DomError::SvgScriptProcessingUnavailable
    );
    unchanged(dom, &before, usage, prefix);
    assert!(
        !dom.snapshot()
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    assert!(
        !before
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
}

fn processed(snapshot: &DomSnapshot, script: NodeId, document: DocumentId, parent: NodeId) {
    let node = snapshot.node(script).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert_eq!(node.parent(), Some(parent));
    assert!(node.is_connected());
    let state = node.as_svg_script().unwrap();
    assert_eq!(state.origin(), DomSvgScriptOrigin::Native);
    assert!(state.flags().already_processed());
}

fn connect_without_recipient(dom: &mut Dom, script: NodeId, document: DocumentId, parent: NodeId) {
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(script, parent, None);
    dom.apply(batch, &cancelled()).unwrap();
    processed(&dom.snapshot(), script, document, parent);
    assert!(
        !before
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
}

#[test]
fn removing_one_registry_recipient_keeps_the_other_until_its_exact_removal() {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut first = recipients();
    let mut second = recipients();
    let first_id = register(&mut dom, &mut first, script, DomEventType::Load, false);
    let second_id = register(&mut dom, &mut second, script, DomEventType::Error, false);
    refuse_insertion(&mut dom, script, root, root);
    assert!(first.remove_listener(first_id));
    refuse_insertion(&mut dom, script, root, root);
    assert!(second.remove_listener(second_id));
    connect_without_recipient(&mut dom, script, document, root);
}

#[test]
fn dropping_registries_releases_only_their_own_ancestor_recipients() {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut first = recipients();
    let mut second = recipients();
    register(&mut dom, &mut first, root, DomEventType::Load, true);
    register(&mut dom, &mut second, root, DomEventType::Error, true);
    drop(first);
    refuse_insertion(&mut dom, script, root, root);
    drop(second);
    connect_without_recipient(&mut dom, script, document, root);
}

#[test]
fn duplicate_registration_has_one_lifetime_and_capture_remains_distinct() {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut recipients = recipients();
    let callback = DomEventCallback::new(|_, _, observed: &mut Vec<DomEventType>, event| {
        observed.push(event.event_type());
        Ok(())
    });
    let first = register_callback(
        &mut dom,
        &mut recipients,
        script,
        DomEventType::Load,
        callback.clone(),
        DomEventListenerOptions {
            capture: false,
            once: false,
        },
    );
    let duplicate = register_callback(
        &mut dom,
        &mut recipients,
        script,
        DomEventType::Load,
        callback.clone(),
        DomEventListenerOptions {
            capture: false,
            once: true,
        },
    );
    assert_eq!(duplicate, first);
    let capture = register_callback(
        &mut dom,
        &mut recipients,
        script,
        DomEventType::Load,
        callback,
        DomEventListenerOptions {
            capture: true,
            once: false,
        },
    );
    assert_ne!(capture, first);
    assert!(recipients.remove_listener(first));
    assert!(!recipients.remove_listener(duplicate));
    refuse_insertion(&mut dom, script, root, root);
    assert!(recipients.remove_listener(capture));
    connect_without_recipient(&mut dom, script, document, root);
}

#[test]
fn stale_registration_removal_cannot_release_its_replacement_generation() {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut recipients = recipients();
    let old = register(&mut dom, &mut recipients, script, DomEventType::Load, false);
    assert!(recipients.remove_listener(old));
    let replacement = register(&mut dom, &mut recipients, script, DomEventType::Load, false);
    assert_ne!(replacement, old);
    assert!(!recipients.remove_listener(old));
    refuse_insertion(&mut dom, script, root, root);
    assert!(recipients.remove_listener(replacement));
    connect_without_recipient(&mut dom, script, document, root);
}

#[test]
fn retired_node_recipient_does_not_qualify_the_replacement_node_generation() {
    let (mut dom, document, root) = native_owner();
    let old = detached_script(&mut dom, document, None);
    let mut recipients = recipients();
    let registration = register(&mut dom, &mut recipients, old, DomEventType::Load, false);
    let retained = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(old);
    dom.apply(batch, &cancelled()).unwrap();
    let replacement = detached_script(&mut dom, document, None);
    assert_ne!(replacement, old);
    assert_eq!(dom.snapshot().node(old).unwrap_err(), DomError::StaleNode);
    unprocessed(&retained, old, document, None);
    connect_without_recipient(&mut dom, replacement, document, root);
    assert!(recipients.remove_listener(registration));
    processed(&dom.snapshot(), replacement, document, root);
    unprocessed(&retained, old, document, None);
}

#[test]
fn ordinary_noncapture_ancestor_recipients_do_not_apply_to_nonbubbling_svg_events() {
    for kind in [DomEventType::Load, DomEventType::Error] {
        let (mut dom, document, root) = native_owner();
        let mut recipients = recipients();
        register(&mut dom, &mut recipients, root, kind, false);
        register(&mut dom, &mut recipients, document.node(), kind, false);
        let script = detached_script(&mut dom, document, None);
        connect_without_recipient(&mut dom, script, document, root);
    }
}

struct ShadowOwner {
    dom: Dom,
    document: DocumentId,
    connected_root: NodeId,
    host: NodeId,
    shadow: NodeId,
    slot: NodeId,
    inside: NodeId,
}

fn shadow_owner(mode: HtmlShadowRootMode, connected: bool) -> ShadowOwner {
    let (mut dom, document, connected_root) = native_owner();
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(mode, HtmlSlotAssignment::Manual, false, false, false),
    );
    let slot = batch.create_element_ns(document, Namespace::html(), "slot");
    let inside = batch.create_element_ns(document, Namespace::html(), "section");
    batch.insert_before(&slot, &shadow, None);
    batch.insert_before(&inside, &shadow, None);
    if connected {
        batch.insert_before(&host, connected_root, None);
    }
    let commit = dom.apply(batch, &cancelled()).unwrap();
    ShadowOwner {
        host: commit.created(&host).unwrap().unwrap(),
        shadow: commit.created(&shadow).unwrap().unwrap(),
        slot: commit.created(&slot).unwrap().unwrap(),
        inside: commit.created(&inside).unwrap().unwrap(),
        dom,
        document,
        connected_root,
    }
}

#[test]
fn noncomposed_shadow_origin_excludes_outside_capture_in_open_and_closed_roots() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        for kind in [DomEventType::Load, DomEventType::Error] {
            let mut owner = shadow_owner(mode, true);
            let mut recipients = recipients();
            register(&mut owner.dom, &mut recipients, owner.host, kind, true);
            register(
                &mut owner.dom,
                &mut recipients,
                owner.document.node(),
                kind,
                true,
            );
            let script = detached_script(&mut owner.dom, owner.document, None);
            connect_without_recipient(&mut owner.dom, script, owner.document, owner.inside);
        }
    }
}

#[test]
fn shadow_origin_capture_inside_its_root_refuses_in_open_and_closed_roots() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        for kind in [DomEventType::Load, DomEventType::Error] {
            for at_root in [false, true] {
                let mut owner = shadow_owner(mode, true);
                let mut recipients = recipients();
                let target = if at_root { owner.shadow } else { owner.inside };
                register(&mut owner.dom, &mut recipients, target, kind, true);
                let script = detached_script(&mut owner.dom, owner.document, None);
                refuse_insertion(&mut owner.dom, script, owner.inside, owner.host);
                unprocessed(&owner.dom.snapshot(), script, owner.document, None);
            }
        }
    }
}

#[test]
fn stored_slotted_light_origin_reaches_each_capture_segment_across_the_shadow_host() {
    #[derive(Clone, Copy)]
    enum CaptureSegment {
        Slot,
        ShadowRoot,
        Host,
        Document,
    }
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        for kind in [DomEventType::Load, DomEventType::Error] {
            for segment in [
                CaptureSegment::Slot,
                CaptureSegment::ShadowRoot,
                CaptureSegment::Host,
                CaptureSegment::Document,
            ] {
                let mut owner = shadow_owner(mode, false);
                let script = detached_script(&mut owner.dom, owner.document, Some(owner.host));
                let mut assignment = DomBatch::new();
                assignment.assign_slot(owner.slot, vec![script.into()]);
                owner.dom.apply(assignment, &cancelled()).unwrap();
                assert_eq!(
                    owner
                        .dom
                        .snapshot()
                        .node(script)
                        .unwrap()
                        .stored_assigned_slot()
                        .unwrap(),
                    Some(owner.slot)
                );
                let target = match segment {
                    CaptureSegment::Slot => owner.slot,
                    CaptureSegment::ShadowRoot => owner.shadow,
                    CaptureSegment::Host => owner.host,
                    CaptureSegment::Document => owner.document.node(),
                };
                let mut recipients = recipients();
                register(&mut owner.dom, &mut recipients, target, kind, true);
                let before = owner.dom.snapshot();
                let usage = owner.dom.usage();
                let mut insertion = DomBatch::new();
                insertion.set_attribute(owner.host, "data-unpublished", "must-roll-back");
                insertion.insert_before(owner.host, owner.connected_root, None);
                assert_eq!(
                    owner.dom.apply(insertion, &cancelled()).unwrap_err(),
                    DomError::SvgScriptProcessingUnavailable
                );
                unchanged(&owner.dom, &before, usage, owner.host);
                unprocessed(
                    &owner.dom.snapshot(),
                    script,
                    owner.document,
                    Some(owner.host),
                );
                assert_eq!(
                    owner
                        .dom
                        .snapshot()
                        .node(script)
                        .unwrap()
                        .stored_assigned_slot()
                        .unwrap(),
                    Some(owner.slot)
                );
            }
        }
    }
}

#[test]
fn registration_after_processing_does_not_replay_the_processed_svg_transition() {
    let (mut dom, document, root) = native_owner();
    let mut batch = DomBatch::new();
    let destination = batch.create_element_ns(document, Namespace::html(), "section");
    batch.insert_before(&destination, root, None);
    let commit = dom.apply(batch, &cancelled()).unwrap();
    let destination = commit.created(&destination).unwrap().unwrap();
    drop(commit);
    let script = detached_script(&mut dom, document, None);
    connect_without_recipient(&mut dom, script, document, root);
    let retained = dom.snapshot();
    let mut recipients = recipients();
    register(&mut dom, &mut recipients, script, DomEventType::Load, false);
    register(&mut dom, &mut recipients, root, DomEventType::Error, true);
    let mut batch = DomBatch::new();
    batch.detach(script);
    batch.insert_before(script, destination, None);
    dom.apply(batch, &cancelled()).unwrap();
    processed(&retained, script, document, root);
    processed(&dom.snapshot(), script, document, destination);
}

#[test]
fn canceled_mutation_keeps_recipient_and_false_history_for_the_uncanceled_retry() {
    let (mut dom, document, root) = native_owner();
    let script = detached_script(&mut dom, document, None);
    let mut recipients = recipients();
    let registration = register(
        &mut dom,
        &mut recipients,
        script,
        DomEventType::Error,
        false,
    );
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-unpublished", "must-roll-back");
    batch.insert_before(script, root, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    unchanged(&dom, &before, usage, root);
    unprocessed(&dom.snapshot(), script, document, None);
    refuse_insertion(&mut dom, script, root, root);
    assert!(recipients.remove_listener(registration));
    connect_without_recipient(&mut dom, script, document, root);
}

#[test]
fn one_dom_recipient_slot_is_shared_by_registries_and_duplicate_keeps_its_original_lease() {
    let (mut dom, document, root) = native_owner();
    dom.set_limits(dom.limits().with_event_recipients(
        DomEventRecipientLimits::new(1, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    ))
    .unwrap();
    let first_script = detached_script(&mut dom, document, None);
    let second_script = detached_script(&mut dom, document, None);
    let mut first = recipients();
    let mut second = recipients();
    let callback = DomEventCallback::new(|_, _, observed: &mut Vec<DomEventType>, event| {
        observed.push(event.event_type());
        Ok(())
    });
    let first_id = register_callback(
        &mut dom,
        &mut first,
        first_script,
        DomEventType::Load,
        callback.clone(),
        DomEventListenerOptions::default(),
    );
    let duplicate = register_callback(
        &mut dom,
        &mut first,
        first_script,
        DomEventType::Load,
        callback.clone(),
        DomEventListenerOptions {
            capture: false,
            once: true,
        },
    );
    assert_eq!(duplicate, first_id);
    let before = dom.snapshot();
    // Repeating the exact failed key must still fail: the first refusal must
    // not leave a Runtime registration that bypasses canonical admission.
    for _ in 0..2 {
        let error = second
            .add_listener(
                &mut DomEventDom::borrow(&mut dom),
                second_script,
                DomEventType::Error,
                callback.clone(),
                DomEventListenerOptions::default(),
            )
            .unwrap_err();
        assert_eq!(
            error,
            DomEventDispatchError::Dom(DomError::ResourceLimit {
                resource: DomResource::EventRecipients,
                limit: 1,
            })
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        unprocessed(&dom.snapshot(), second_script, document, None);
    }
    refuse_insertion(&mut dom, first_script, root, root);
    assert!(first.remove_listener(first_id));
    assert!(!first.remove_listener(duplicate));
    let second_id = register_callback(
        &mut dom,
        &mut second,
        second_script,
        DomEventType::Error,
        callback,
        DomEventListenerOptions::default(),
    );
    assert!(!second.remove_listener(first_id));
    assert!(!first.remove_listener(first_id));
    refuse_insertion(&mut dom, second_script, root, root);
    connect_without_recipient(&mut dom, first_script, document, root);
    assert!(second.remove_listener(second_id));
    connect_without_recipient(&mut dom, second_script, document, root);
    unprocessed(&before, first_script, document, None);
    unprocessed(&before, second_script, document, None);
}

#[test]
fn runtime_registration_refusal_does_not_consume_the_other_dom_recipient_slot() {
    let (mut dom, document, root) = native_owner();
    dom.set_limits(dom.limits().with_event_recipients(
        DomEventRecipientLimits::new(2, 16_000_000, 16_000_000, 4_000_000).unwrap(),
    ))
    .unwrap();
    let first_script = detached_script(&mut dom, document, None);
    let second_script = detached_script(&mut dom, document, None);
    let mut first: Recipients = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 8,
        nested_dispatches: 2,
    })
    .unwrap();
    let first_id = register(
        &mut dom,
        &mut first,
        first_script,
        DomEventType::Load,
        false,
    );
    let error = first
        .add_listener(
            &mut DomEventDom::borrow(&mut dom),
            second_script,
            DomEventType::Error,
            DomEventCallback::new(|_, _, observed: &mut Vec<DomEventType>, event| {
                observed.push(event.event_type());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap_err();
    assert_eq!(
        error,
        DomEventDispatchError::Admission(DomEventResource::Registrations)
    );
    // Both canonical slots are usable despite the failed Runtime attempt.
    let mut second = recipients();
    let second_id = register(
        &mut dom,
        &mut second,
        second_script,
        DomEventType::Error,
        false,
    );
    refuse_insertion(&mut dom, first_script, root, root);
    refuse_insertion(&mut dom, second_script, root, root);
    assert!(second.remove_listener(second_id));
    assert!(first.remove_listener(first_id));
    let replacement = register(
        &mut dom,
        &mut first,
        second_script,
        DomEventType::Error,
        false,
    );
    assert_ne!(replacement, first_id);
    assert!(!first.remove_listener(first_id));
    refuse_insertion(&mut dom, second_script, root, root);
    assert!(first.remove_listener(replacement));
    connect_without_recipient(&mut dom, first_script, document, root);
    connect_without_recipient(&mut dom, second_script, document, root);
}
