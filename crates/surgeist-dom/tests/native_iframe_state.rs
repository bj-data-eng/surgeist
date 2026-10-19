//! Actual native never-BC Iframe state and canonical DOM lifecycle.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlIframeInitialState;
use surgeist_html_semantics::HtmlCaptureLimits;

fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            128,
            16384,
            256,
            64,
            16,
            16,
            131072,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn state(snapshot: &DomSnapshot, node: NodeId) -> HtmlIframeInitialState {
    let state = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .iframe_initial_state()
        .unwrap();
    let content = snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable_presence()
        .unwrap();
    assert_eq!(content, surgeist_html::HtmlChildNavigablePresence::Null);
    assert_eq!(
        content.content_access(),
        surgeist_html::HtmlChildNavigableAccessDecision::ReturnNull
    );
    assert_eq!(
        content.content_access(),
        surgeist_html::HtmlChildNavigableAccessDecision::ReturnNull
    );
    assert!(!state.current_navigation_was_lazy_loaded());
    assert!(state.pending_resource_timing_start_time_is_null());
    assert!(state.pending_resource_timing_url_is_null());
    assert!(state.lazy_load_resumption_is_null());
    assert!(state.potentially_delays_load());
    assert_eq!(
        content.load_delay(),
        surgeist_html::HtmlChildNavigableLoadDelayDecision::DoesNotDelay
    );
    state
}
fn setup(dom: &mut Dom) -> (DocumentId, NodeId) {
    let mut b = DomBatch::new();
    let d = b.create_document();
    let iframe = b.create_element_ns(&d, Namespace::html(), "iframe");
    let c = apply(dom, b);
    (
        c.document(&d).unwrap(),
        c.created(&iframe).unwrap().unwrap(),
    )
}

#[test]
fn exact_family_dispatch_and_actual_html_document_case_rules() {
    let mut dom = dom();
    let (d, iframe) = setup(&mut dom);
    state(&dom.snapshot(), iframe);
    let mut b = DomBatch::new();
    let prefixed = b.create_element_ns(d, Namespace::html(), "p:iframe");
    let upper = b.create_element_ns(d, Namespace::html(), "Iframe");
    let other = b.create_element_ns(d, Namespace::none(), "iframe");
    let html = b.create_html_document(d, None);
    let folded = b.create_element(&html, "IFRAME");
    let c = apply(&mut dom, b);
    let s = dom.snapshot();
    let prefixed = c.created(&prefixed).unwrap().unwrap();
    let folded = c.created(&folded).unwrap().unwrap();
    assert_eq!(s.node(prefixed).unwrap().node_name(), "p:iframe".into());
    assert_eq!(s.node(folded).unwrap().node_name(), "IFRAME".into());
    state(&s, prefixed);
    state(&s, folded);
    for ticket in [upper, other] {
        assert!(
            s.node(c.created(&ticket).unwrap().unwrap())
                .unwrap()
                .as_element()
                .unwrap()
                .iframe_initial_state()
                .is_none()
        );
    }
}

#[test]
fn raw_attribute_changes_and_loading_transitions_preserve_unprocessed_state() {
    let mut dom = dom();
    let (d, iframe) = setup(&mut dom);
    let initial = dom.snapshot();
    let raw = DocumentString::from_utf16(vec![0xd800, 0x3a, 0xdc00]);
    let mut b = DomBatch::new();
    b.insert_before(iframe, d, None);
    b.set_attribute(iframe, "sandbox", "");
    b.set_attribute(iframe, "srcdoc", "");
    b.set_attribute(iframe, "src", raw.clone());
    b.set_attribute(iframe, "loading", "LaZy");
    let c = apply(&mut dom, b);
    assert!(
        !c.changes()
            .changes()
            .contains(&DomChange::ElementState(iframe))
    );
    let lazy = dom.snapshot();
    assert!(lazy.node(iframe).unwrap().is_connected());
    assert_eq!(state(&lazy, iframe), state(&initial, iframe));
    assert_eq!(
        lazy.node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"srcdoc".into()),
        Some(&"".into())
    );
    for loading in [
        DocumentString::from("eager"),
        raw.clone(),
        DocumentString::from("lazy"),
    ] {
        let mut b = DomBatch::new();
        b.set_attribute(iframe, "loading", loading);
        b.set_attribute(iframe, "sandbox", raw.clone());
        apply(&mut dom, b);
        state(&dom.snapshot(), iframe);
    }
    // Lazy -> missing is Eager according to the shared language rule.
    let mut b = DomBatch::new();
    b.remove_attribute(iframe, "loading");
    b.remove_attribute(iframe, "sandbox");
    b.remove_attribute(iframe, "srcdoc");
    b.remove_attribute(iframe, "src");
    apply(&mut dom, b);
    let s = dom.snapshot();
    assert!(
        s.node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    state(&s, iframe);
    // Namespace and case are exact in this XML node-document.
    let mut b = DomBatch::new();
    b.set_attribute_ns(
        iframe,
        Namespace::from_uri("urn:other"),
        "p:loading",
        "lazy",
    );
    b.set_attribute(iframe, "LOADING", "lazy");
    b.set_attribute(iframe, "src", raw.clone());
    apply(&mut dom, b);
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(iframe, "src", raw.clone());
    assert!(apply(&mut dom, b).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        lazy.node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"src".into()),
        Some(&raw)
    );
}

#[test]
fn cloning_is_fresh_creation_adoption_preserves_and_import_reads_retained_source() {
    let mut source = dom();
    let (d, iframe) = setup(&mut source);
    let mut b = DomBatch::new();
    let destination = b.create_document();
    let text = b.create_text(d, "old child");
    b.insert_before(&text, iframe, None);
    for (name, value) in [
        ("src", "old src"),
        ("srcdoc", ""),
        ("sandbox", "allow-scripts"),
        ("loading", "lazy"),
    ] {
        b.set_attribute(iframe, name, value);
    }
    b.set_attribute(iframe, "nonce", "content");
    b.set_nonce(iframe, "slot");
    let c = apply(&mut source, b);
    let destination = c.document(&destination).unwrap();
    let old = source.snapshot();
    let mut b = DomBatch::new();
    let clone = b.import_node(&old, iframe, d, false);
    b.adopt(iframe, destination);
    let c = apply(&mut source, b);
    let clone = c.created(&clone).unwrap().unwrap();
    let s = source.snapshot();
    assert_ne!(clone, iframe);
    assert!(s.node(clone).unwrap().children().is_empty());
    assert_eq!(s.node(iframe).unwrap().owner_document(), Some(destination));
    assert_eq!(
        s.node(iframe).unwrap().as_element().unwrap().nonce(),
        Some(&"slot".into())
    );
    state(&s, clone);
    state(&s, iframe);
    let captured = old
        .html_capture(
            d,
            None,
            HtmlCaptureLimits::new(64, 1024, 128, 16384, 64, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(64, 10000),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(captured.subject(iframe).is_ok());
    state(captured.source(), iframe);
    let mut b = DomBatch::new();
    b.set_attribute(iframe, "sandbox", "new live");
    b.destroy(iframe);
    apply(&mut source, b);
    drop(source);
    let mut target = dom();
    let mut b = DomBatch::new();
    let destination = b.create_document();
    let imported = b.import_node(&old, iframe, &destination, true);
    let c = apply(&mut target, b);
    let imported = c.created(&imported).unwrap().unwrap();
    let s = target.snapshot();
    let node = s.node(imported).unwrap();
    assert_eq!(
        node.owner_document(),
        Some(c.document(&destination).unwrap())
    );
    assert_eq!(node.children().len(), 1);
    assert_eq!(
        s.node(node.children()[0]).unwrap().data(),
        Some(&"old child".into())
    );
    let e = node.as_element().unwrap();
    assert_eq!(
        e.attribute(&"sandbox".into()),
        Some(&"allow-scripts".into())
    );
    assert_eq!(e.attribute(&"src".into()), Some(&"old src".into()));
    assert_eq!(e.attribute(&"srcdoc".into()), Some(&"".into()));
    assert_eq!(e.nonce(), Some(&"slot".into()));
    state(&s, imported);
    state(captured.source(), iframe);
}

#[test]
fn mixed_navigable_containers_share_slot_clone_and_removal_lifecycle() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let target = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    let root = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            true,
            false,
        ),
    );
    let slot = b.create_element_ns(&d, Namespace::html(), "slot");
    let frame = b.create_element_ns(&d, Namespace::html(), "frame");
    let iframe = b.create_element_ns(&d, Namespace::html(), "iframe");
    b.insert_before(&slot, &root, None);
    b.insert_before(&frame, &host, None);
    b.insert_before(&iframe, &host, None);
    b.insert_before(&host, &d, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let target = c.document(&target).unwrap();
    let host = c.created(&host).unwrap().unwrap();
    let slot = c.created(&slot).unwrap().unwrap();
    let frame = c.created(&frame).unwrap().unwrap();
    let iframe = c.created(&iframe).unwrap().unwrap();
    let old = dom.snapshot();
    assert_eq!(old.slot(slot).unwrap().assigned_nodes(), &[frame, iframe]);
    let mut b = DomBatch::new();
    let cloned = b.import_node(&old, host, d, true);
    b.adopt(iframe, target);
    let c = apply(&mut dom, b);
    let cloned = c.created(&cloned).unwrap().unwrap();
    let s = dom.snapshot();
    let children = s.node(cloned).unwrap().children();
    assert_eq!(children.len(), 2);
    assert!(
        s.node(children[0])
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .is_none()
    );
    state(&s, children[1]);
    let cloned_root = s.node(cloned).unwrap().hosted_shadow_root().unwrap();
    let cloned_slot = s.node(cloned_root).unwrap().children()[0];
    assert_eq!(s.slot(cloned_slot).unwrap().assigned_nodes(), children);
    assert_eq!(s.slot(slot).unwrap().assigned_nodes(), &[frame]);
    assert_eq!(s.node(iframe).unwrap().owner_document(), Some(target));
    assert_eq!(
        s.node(iframe)
            .unwrap()
            .assigned_slot(&AtomicBool::new(false))
            .unwrap(),
        None
    );
    state(&s, iframe);
    let mut b = DomBatch::new();
    b.destroy(cloned);
    apply(&mut dom, b);
    assert_eq!(state(&old, iframe), state(&s, children[1]));
    assert_eq!(
        dom.snapshot().node(children[1]).unwrap_err(),
        DomError::StaleNode
    );
}

#[test]
fn connected_shadow_iframe_uses_actual_document_independently_of_host() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let other = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    let root = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            HtmlSlotAssignment::Named,
            false,
            false,
            false,
        ),
    );
    let target = b.create_fragment(&other);
    let old = b.create_comment(&other, "old");
    b.insert_before(&old, &target, None);
    b.insert_before(&host, &d, None);
    let c = apply(&mut dom, b);
    let d = c.document(&d).unwrap();
    let other = c.document(&other).unwrap();
    let host = c.created(&host).unwrap().unwrap();
    let root = c.created(&root).unwrap().unwrap();
    let target = c.created(&target).unwrap().unwrap();
    let old = c.created(&old).unwrap().unwrap();
    let mut b = DomBatch::new();
    b.replace_child(root, target, old);
    let iframe = b.create_element_ns(other, Namespace::html(), "iframe");
    b.insert_before(&iframe, root, None);
    let c = apply(&mut dom, b);
    let iframe = c.created(&iframe).unwrap().unwrap();
    let s = dom.snapshot();
    assert_eq!(s.node(host).unwrap().owner_document(), Some(d));
    assert_eq!(s.node(root).unwrap().owner_document(), Some(other));
    assert_eq!(s.node(iframe).unwrap().owner_document(), Some(other));
    assert!(s.node(iframe).unwrap().is_connected());
    state(&s, iframe);
    let mut b = DomBatch::new();
    b.detach(host);
    apply(&mut dom, b);
    assert!(!dom.snapshot().node(iframe).unwrap().is_connected());
    state(&dom.snapshot(), iframe);
}

#[test]
fn exact_native_iframe_admission_and_failed_attribute_edit_are_atomic() {
    // XML Document metadata31 + HTML namespace28 + local iframe6 =65.
    // The actual closed family owns no strings, child nodes or external handles.
    for units in [64, 65] {
        let mut dom = Dom::new(
            DomLimits::new(
                2,
                units,
                8,
                0,
                0,
                4,
                8192,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let iframe = b.create_element_ns(&d, Namespace::html(), "iframe");
        let result = dom.apply(b, &AtomicBool::new(false));
        if units == 64 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 64
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage().managed_nodes(), 0);
        } else {
            let c = result.unwrap();
            let iframe = c.created(&iframe).unwrap().unwrap();
            assert_eq!(dom.usage().managed_nodes(), 2);
            assert_eq!(dom.usage().string_code_units(), 65);
            state(&dom.snapshot(), iframe);
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            b.set_attribute(iframe, "sandbox", "");
            assert_eq!(
                dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit: 65
                }
            );
            assert_eq!(dom.usage(), usage);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert!(
                dom.snapshot()
                    .node(iframe)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attributes()
                    .is_empty()
            );
            let mut b = DomBatch::new();
            b.destroy(iframe);
            apply(&mut dom, b);
            assert_eq!(dom.usage().string_code_units(), 31);
        }
    }
}

#[test]
fn failed_and_cancelled_batches_preserve_history_and_native_state() {
    for history in [0, 8] {
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                2048,
                32,
                0,
                0,
                history,
                65536,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let (d, iframe) = setup(&mut dom);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.set_attribute(iframe, "loading", "lazy");
        b.insert_before(iframe, d, None);
        b.insert_before(d, iframe, None);
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::HierarchyRequest
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        let mut b = DomBatch::new();
        b.set_attribute(iframe, "sandbox", "unpublished");
        b.insert_before(iframe, d, None);
        assert_eq!(
            dom.apply(b, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        let mut b = DomBatch::new();
        b.set_attribute(iframe, "loading", "lazy");
        b.insert_before(iframe, d, None);
        let c = apply(&mut dom, b);
        if history == 0 {
            assert!(matches!(
                dom.changes_since(before.stamp()),
                Err(DomHistoryError::FullRecomputeRequired { .. })
            ));
        } else {
            assert_eq!(
                dom.changes_since(before.stamp()).unwrap().changes(),
                c.changes().changes()
            );
        }
        state(&dom.snapshot(), iframe);
        assert!(
            before
                .node(iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .attributes()
                .is_empty()
        );
    }
}
