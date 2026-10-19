//! Independent literal oracles from DOM3071 §4.2.2/4.2.3/clone/adopt/attach-shadow
//! and #1074's native destruction, identity and publication contract.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomError, DomHistoryError,
    DomLimits, DomNodeKind, DomRegistryBinding, DomResource, DomShadowRootInit,
    HtmlShadowRootMode as Mode, HtmlShadowRootProperties, HtmlSlotAssignment as Assignment,
    Namespace, NodeId,
};
const HTML: &str = "http://www.w3.org/1999/xhtml";
fn dom() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            65536,
            512,
            0,
            0,
            32,
            65536,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn base(dom: &mut Dom, mode: Mode, clonable: bool) -> (DocumentId, NodeId, NodeId) {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), "div");
    let root = batch.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(mode, Assignment::Named, false, clonable, false),
    );
    batch.insert_before(&host, &document, None);
    let commit = apply(dom, batch);
    (
        commit.document(&document).unwrap(),
        commit.created(&host).unwrap().unwrap(),
        commit.created(&root).unwrap().unwrap(),
    )
}
#[test]
fn explicit_mode_defaults_and_complete_native_state_have_real_canonical_identity() {
    for mode in [Mode::Open, Mode::Closed] {
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), "p:div");
        let init = DomShadowRootInit::new(mode);
        assert_eq!(init.properties().slot_assignment(), Assignment::Named);
        batch.insert_before(&host, &document, None);
        let first = apply(&mut dom, batch);
        let document = first.document(&document).unwrap();
        let host = first.created(&host).unwrap().unwrap();
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        let root = batch.attach_shadow(host, init);
        let commit = apply(&mut dom, batch);
        let root = commit.created(&root).unwrap().unwrap();
        assert_eq!(before.node(host).unwrap().hosted_shadow_root(), None);
        assert_ne!(root, host);
        let snapshot = dom.snapshot();
        let capture = snapshot.shadow_root(root).unwrap();
        assert_eq!(capture.id(), root);
        assert_eq!(capture.host(), host);
        let state = capture.state();
        assert_eq!(state.mode(), mode);
        assert_eq!(state.slot_assignment(), Assignment::Named);
        assert!(!state.delegates_focus());
        assert!(!state.clonable());
        assert!(!state.serializable());
        assert!(!state.declarative());
        assert!(!state.available_to_element_internals());
        assert!(!state.keep_custom_element_registry_null());
        assert_eq!(state.custom_element_registry(), DomRegistryBinding::Null);
        let node = capture.node();
        assert!(matches!(node.kind(), DomNodeKind::ShadowRoot(_)));
        assert_eq!(node.node_name(), DocumentString::from("#document-fragment"));
        assert_eq!(node.parent(), None);
        assert_eq!(node.parent_element(), None);
        assert_eq!(node.owner_document(), Some(document));
        assert!(node.children().is_empty());
        assert_eq!(node.root(), root);
        assert_eq!(node.shadow_including_root(), document.node());
        assert!(node.is_connected());
        assert_eq!(node.tree_scope().root(), root);
        assert_eq!(node.tree_scope().host(), Some(host));
        let host_view = snapshot.node(host).unwrap();
        assert_eq!(host_view.children(), &[]);
        assert_eq!(host_view.hosted_shadow_root(), Some(root));
        assert_eq!(
            host_view.shadow_root(),
            (mode == Mode::Open).then_some(root)
        );
        assert_eq!(host_view.tree_scope().root(), document.node());
        assert_eq!(host_view.tree_scope().host(), None);
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::ShadowRoot(host))
        );
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::ShadowRelations)
        );
    }
}
#[test]
fn explicit_properties_are_retained_without_claiming_slot_or_focus_execution() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), "x-panel");
    let properties =
        HtmlShadowRootProperties::new(Mode::Closed, Assignment::Manual, true, true, true);
    let root = batch.attach_shadow(&host, properties);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let state = snapshot
        .shadow_root(commit.created(&root).unwrap().unwrap())
        .unwrap()
        .state();
    assert_eq!(state.properties(), properties);
    assert!(state.delegates_focus());
    assert!(state.serializable());
    assert!(state.clonable());
    assert_eq!(state.slot_assignment(), Assignment::Manual);
    assert!(!state.available_to_element_internals());
}
#[test]
fn shadow_preorder_and_scope_do_not_rewrite_ordinary_or_element_sibling_relations() {
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Closed, false);
    let mut batch = DomBatch::new();
    let inner = batch.create_element_ns(document, Namespace::from_uri(HTML), "span");
    let inner_root = batch.attach_shadow(&inner, DomShadowRootInit::new(Mode::Open));
    let deep = batch.create_text(document, "deep");
    let inner_light = batch.create_text(document, "hidden inner light");
    let light = batch.create_element(document, "light");
    let text = batch.create_text(document, " ");
    let comment = batch.create_comment(document, "comment");
    let sibling = batch.create_element(document, "sibling");
    batch.insert_before(&inner, root, None);
    batch.insert_before(&deep, &inner_root, None);
    batch.insert_before(&inner_light, &inner, None);
    batch.insert_before(&light, host, None);
    batch.insert_before(&text, host, None);
    batch.insert_before(&comment, host, None);
    batch.insert_before(&sibling, host, None);
    let commit = apply(&mut dom, batch);
    let id = |ticket| commit.created(ticket).unwrap().unwrap();
    let (inner, inner_root, deep, inner_light, light, text, comment, sibling) = (
        id(&inner),
        id(&inner_root),
        id(&deep),
        id(&inner_light),
        id(&light),
        id(&text),
        id(&comment),
        id(&sibling),
    );
    let snapshot = dom.snapshot();
    let host_view = snapshot.node(host).unwrap();
    assert_eq!(
        host_view.tree_order().collect::<Vec<_>>(),
        [host, light, text, comment, sibling]
    );
    assert_eq!(
        host_view.shadow_including_tree_order().collect::<Vec<_>>(),
        [
            host,
            root,
            inner,
            inner_root,
            deep,
            inner_light,
            light,
            text,
            comment,
            sibling
        ]
    );
    assert_eq!(
        snapshot
            .node(root)
            .unwrap()
            .tree_order()
            .collect::<Vec<_>>(),
        [root, inner, inner_light]
    );
    assert_eq!(
        snapshot
            .node(root)
            .unwrap()
            .shadow_including_tree_order()
            .collect::<Vec<_>>(),
        [root, inner, inner_root, deep, inner_light]
    );
    assert_eq!(snapshot.node(deep).unwrap().root(), inner_root);
    assert_eq!(
        snapshot.node(deep).unwrap().tree_scope().host(),
        Some(inner)
    );
    assert_eq!(
        snapshot.node(deep).unwrap().shadow_including_root(),
        document.node()
    );
    assert_eq!(snapshot.node(inner).unwrap().parent(), Some(root));
    assert_eq!(snapshot.node(inner).unwrap().parent_element(), None);
    assert_eq!(
        snapshot.node(light).unwrap().next_element_sibling(),
        Some(sibling)
    );
    assert_eq!(snapshot.node(sibling).unwrap().element_index(), Some(1));
    assert!(
        host_view
            .is_shadow_including_inclusive_ancestor_of(deep)
            .unwrap()
    );
    assert!(
        host_view
            .is_host_including_inclusive_ancestor_of(deep)
            .unwrap()
    );
    assert!(!host_view.tree_order().any(|id| id == deep));
}
#[test]
fn source_host_validation_rejects_namespace_spelling_and_duplicate_roots_atomically() {
    for (namespace, name) in [
        ("", "div"),
        ("urn:other", "div"),
        (HTML, "Div"),
        (HTML, "em"),
        (HTML, "X-panel"),
        (HTML, "annotation-xml"),
    ] {
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let host = batch.create_element_ns(&document, Namespace::from_uri(namespace), name);
        let commit = apply(&mut dom, batch);
        let host = commit.created(&host).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::NotSupported
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            dom.snapshot().node(host).unwrap().hosted_shadow_root(),
            None
        );
    }
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Open, false);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NotSupported
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().node(host).unwrap().hosted_shadow_root(),
        Some(root)
    );
    let mut batch = DomBatch::new();
    batch.attach_shadow(document, DomShadowRootInit::new(Mode::Open));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::WrongNodeKind
    );
    // A real native Slot is constructible, but its name is not a valid native
    // shadow host. Family admission and host eligibility are separate rules.
    let mut batch = DomBatch::new();
    let slot = batch.create_element_ns(document, Namespace::from_uri(HTML), "slot");
    let commit = apply(&mut dom, batch);
    let slot = commit.created(&slot).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.attach_shadow(slot, DomShadowRootInit::new(Mode::Open));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::NotSupported
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}
#[test]
fn host_including_cycles_and_direct_root_lifecycle_operations_reject_without_publication() {
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Open, false);
    let mut batch = DomBatch::new();
    let child = batch.create_element(document, "child");
    batch.insert_before(&child, root, None);
    let commit = apply(&mut dom, batch);
    let child = commit.created(&child).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut cases = Vec::new();
    let mut batch = DomBatch::new();
    batch.insert_before(host, root, None);
    cases.push((batch, DomError::HierarchyRequest));
    let mut batch = DomBatch::new();
    batch.insert_before(root, child, None);
    cases.push((batch, DomError::HierarchyRequest));
    let mut batch = DomBatch::new();
    batch.replace_child(host, root, child);
    cases.push((batch, DomError::HierarchyRequest));
    let mut batch = DomBatch::new();
    batch.adopt(root, document);
    cases.push((batch, DomError::HierarchyRequest));
    let mut batch = DomBatch::new();
    batch.import_node(&before, root, document, true);
    cases.push((batch, DomError::NotSupported));
    let mut batch = DomBatch::new();
    batch.destroy(root);
    cases.push((batch, DomError::InvalidDestruction));
    let mut batch = DomBatch::new();
    batch.destroy(host);
    cases.push((batch, DomError::InvalidDestruction));
    for (batch, error) in cases {
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
}
#[test]
fn adoption_preserves_nested_shadow_identities_and_detach_connectedness_with_old_snapshots() {
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Closed, false);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let inner = batch.create_element_ns(document, Namespace::from_uri(HTML), "span");
    let inner_root = batch.attach_shadow(&inner, DomShadowRootInit::new(Mode::Open));
    let text = batch.create_text(document, "text");
    batch.insert_before(&inner, root, None);
    batch.insert_before(&text, &inner_root, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let inner = commit.created(&inner).unwrap().unwrap();
    let inner_root = commit.created(&inner_root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(host, document);
    let commit = apply(&mut dom, batch);
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::ShadowRelations)
    );
    let detached = dom.snapshot();
    for id in [host, root, inner, inner_root, text] {
        assert!(!detached.node(id).unwrap().is_connected());
        assert_eq!(detached.node(id).unwrap().owner_document(), Some(document));
    }
    assert_eq!(detached.node(text).unwrap().shadow_including_root(), host);
    let mut batch = DomBatch::new();
    batch.adopt(host, other);
    batch.insert_before(host, other, None);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for id in [host, root, inner, inner_root, text] {
        assert_eq!(snapshot.node(id).unwrap().owner_document(), Some(other));
        assert!(snapshot.node(id).unwrap().is_connected());
        assert_eq!(old.node(id).unwrap().owner_document(), Some(document));
        assert!(old.node(id).unwrap().is_connected());
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::NodeDocument(id))
        );
    }
    assert_eq!(
        snapshot.node(host).unwrap().hosted_shadow_root(),
        Some(root)
    );
    let mut batch = DomBatch::new();
    batch.detach(host);
    apply(&mut dom, batch);
    assert!(!dom.snapshot().node(text).unwrap().is_connected());
    assert!(old.node(text).unwrap().is_connected());
}
#[test]
fn shadow_fragment_insertion_transfers_children_without_adopting_its_root() {
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Open, false);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let target = batch.create_element(&other, "target");
    let a = batch.create_element(document, "a");
    let b = batch.create_text(document, "b");
    batch.insert_before(&a, root, None);
    batch.insert_before(&b, root, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.insert_before(root, target, None);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(target).unwrap().children(), &[a, b]);
    assert_eq!(snapshot.node(root).unwrap().children(), &[]);
    assert_eq!(snapshot.node(root).unwrap().parent(), None);
    assert_eq!(snapshot.shadow_root(root).unwrap().host(), host);
    assert_eq!(
        snapshot.node(root).unwrap().owner_document(),
        Some(document)
    );
    for id in [a, b] {
        assert_eq!(snapshot.node(id).unwrap().owner_document(), Some(other));
    }
}
#[test]
fn shadow_fragment_replacement_internally_adopts_root_but_preserves_original_host_identity() {
    // DOM replace step 6 adopts its supplied fragment before moving children.
    // Direct adoptNode's ShadowRoot rejection is a distinct public guard.
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Open, false);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    let target = batch.create_element(&other, "target");
    let old = batch.create_comment(&other, "old");
    let child = batch.create_element(document, "child");
    batch.insert_before(&old, &target, None);
    batch.insert_before(&child, root, None);
    let commit = apply(&mut dom, batch);
    let other = commit.document(&other).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_child(root, target, old);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(target).unwrap().children(), &[child]);
    assert_eq!(snapshot.node(old).unwrap().parent(), None);
    assert!(snapshot.node(root).unwrap().children().is_empty());
    assert_eq!(snapshot.node(root).unwrap().owner_document(), Some(other));
    assert_eq!(
        snapshot.node(host).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(snapshot.shadow_root(root).unwrap().host(), host);
    assert_eq!(
        snapshot.node(host).unwrap().hosted_shadow_root(),
        Some(root)
    );
    assert_eq!(
        snapshot.node(root).unwrap().shadow_including_root(),
        document.node()
    );
    assert!(snapshot.node(root).unwrap().is_connected());
    assert_eq!(snapshot.node(child).unwrap().owner_document(), Some(other));
    assert_eq!(before.node(root).unwrap().owner_document(), Some(document));
    assert_eq!(before.node(root).unwrap().children(), &[child]);
    assert!(snapshot.document(document).unwrap().node(root).is_err());
    assert!(snapshot.document(other).unwrap().node(root).is_ok());
    assert!(
        commit
            .changes()
            .changes()
            .contains(&DomChange::NodeDocument(root))
    );
}
#[test]
fn shallow_and_deep_import_copy_clonable_shadow_descendants_after_ordinary_children() {
    let mut source = dom();
    let (document, host, root) = base(&mut source, Mode::Closed, true);
    let mut batch = DomBatch::new();
    let light = batch.create_element(document, "light");
    let inner = batch.create_element_ns(document, Namespace::from_uri(HTML), "span");
    let inner_root = batch.attach_shadow(
        &inner,
        HtmlShadowRootProperties::new(Mode::Open, Assignment::Manual, true, true, true),
    );
    let deep = batch.create_text(document, "deep");
    batch.set_attribute(&inner, "nonce", "public");
    batch.set_nonce(&inner, "private");
    batch.insert_before(&light, host, None);
    batch.insert_before(&inner, root, None);
    batch.insert_before(&deep, &inner_root, None);
    let commit = apply(&mut source, batch);
    let inner = commit.created(&inner).unwrap().unwrap();
    let inner_root = commit.created(&inner_root).unwrap().unwrap();
    let captured = source.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(host);
    batch.destroy(host);
    apply(&mut source, batch);
    drop(source);
    for subtree in [false, true] {
        let mut target = dom();
        let mut batch = DomBatch::new();
        let doc = batch.create_document();
        let copied = batch.import_node(&captured, host, &doc, subtree);
        let after = batch.create_comment(&doc, "ticket after private descendants");
        let commit = apply(&mut target, batch);
        let copied = commit.created(&copied).unwrap().unwrap();
        let doc = commit.document(&doc).unwrap();
        let snapshot = target.snapshot();
        let copy = snapshot.node(copied).unwrap();
        assert_ne!(copied, host);
        assert_eq!(copy.children().len(), usize::from(subtree));
        assert_eq!(
            snapshot
                .node(commit.created(&after).unwrap().unwrap())
                .unwrap()
                .data(),
            Some(&"ticket after private descendants".into())
        );
        let copy_root = copy.hosted_shadow_root().unwrap();
        assert_ne!(copy_root, root);
        assert_eq!(copy.shadow_root(), None);
        let copy_inner = snapshot.node(copy_root).unwrap().children()[0];
        assert_ne!(copy_inner, inner);
        let copy_inner_root = snapshot
            .node(copy_inner)
            .unwrap()
            .hosted_shadow_root()
            .unwrap();
        assert_ne!(copy_inner_root, inner_root);
        let copy_deep = snapshot.node(copy_inner_root).unwrap().children()[0];
        assert_eq!(
            snapshot.node(copy_deep).unwrap().data(),
            Some(&"deep".into())
        );
        assert_eq!(
            snapshot
                .shadow_root(copy_inner_root)
                .unwrap()
                .state()
                .properties(),
            captured
                .shadow_root(inner_root)
                .unwrap()
                .state()
                .properties()
        );
        let element = snapshot.node(copy_inner).unwrap().as_element().unwrap();
        assert_eq!(element.nonce(), Some(&"private".into()));
        assert_eq!(element.attribute(&"nonce".into()), Some(&"public".into()));
        for id in copy.shadow_including_tree_order() {
            assert_eq!(snapshot.node(id).unwrap().owner_document(), Some(doc));
            assert!(captured.node(id).is_err());
        }
        assert_eq!(target.usage().managed_nodes(), 7 + usize::from(subtree));
    }
}
#[test]
fn nonclonable_roots_are_omitted_in_both_import_modes() {
    let mut source = dom();
    let (document, host, root) = base(&mut source, Mode::Open, false);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "not copied");
    batch.insert_before(&text, root, None);
    apply(&mut source, batch);
    for subtree in [false, true] {
        let mut target = dom();
        let mut batch = DomBatch::new();
        let doc = batch.create_document();
        let copy = batch.import_node(&source.snapshot(), host, &doc, subtree);
        let commit = apply(&mut target, batch);
        assert_eq!(
            target
                .snapshot()
                .node(commit.created(&copy).unwrap().unwrap())
                .unwrap()
                .hosted_shadow_root(),
            None
        );
        assert_eq!(target.usage().managed_nodes(), 2);
    }
}
#[test]
fn detached_host_destruction_retires_all_owned_shadow_generations_and_old_snapshots_survive() {
    let mut dom = dom();
    let (document, host, root) = base(&mut dom, Mode::Closed, false);
    let mut batch = DomBatch::new();
    let inner = batch.create_element_ns(document, Namespace::from_uri(HTML), "span");
    let inner_root = batch.attach_shadow(&inner, DomShadowRootInit::new(Mode::Open));
    let text = batch.create_text(document, "nested");
    batch.insert_before(&inner, root, None);
    batch.insert_before(&text, &inner_root, None);
    let commit = apply(&mut dom, batch);
    let inner = commit.created(&inner).unwrap().unwrap();
    let inner_root = commit.created(&inner_root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.detach(host);
    batch.destroy(host);
    let commit = apply(&mut dom, batch);
    assert_eq!(dom.usage().managed_nodes(), 1);
    assert_eq!(dom.usage().string_code_units(), 31);
    for id in [host, root, inner, inner_root, text] {
        assert_eq!(dom.snapshot().node(id).unwrap_err(), DomError::StaleNode);
        assert!(old.node(id).unwrap().is_connected());
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::Destroyed(id))
        );
    }
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(document, Namespace::from_uri(HTML), "div");
    let new_root = batch.attach_shadow(&replacement, DomShadowRootInit::new(Mode::Open));
    let commit = apply(&mut dom, batch);
    let replacement = commit.created(&replacement).unwrap().unwrap();
    let new_root = commit.created(&new_root).unwrap().unwrap();
    assert_ne!(replacement, host);
    assert_ne!(new_root, root);
    assert_eq!(old.shadow_root(root).unwrap().host(), host);
    assert_eq!(
        dom.snapshot().shadow_root(new_root).unwrap().host(),
        replacement
    );
}
#[test]
fn exact_node_and_string_admission_and_late_failure_leave_no_root_or_identity_history() {
    for capacity in [2, 3] {
        // 31 source Document units + 28 HTML namespace + 3 div. Root adds no strings.
        let mut dom = Dom::new(
            DomLimits::new(
                capacity,
                62,
                16,
                0,
                0,
                8,
                8192,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let doc = batch.create_document();
        let host = batch.create_element_ns(&doc, Namespace::from_uri(HTML), "div");
        let commit = apply(&mut dom, batch);
        let doc = commit.document(&doc).unwrap();
        let host = commit.created(&host).unwrap().unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        let root = batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
        let result = dom.apply(batch, &AtomicBool::new(false));
        if capacity == 2 {
            assert_eq!(
                result.unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::ManagedNodes,
                    limit: 2
                }
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert_eq!(
                dom.snapshot().node(host).unwrap().hosted_shadow_root(),
                None
            );
        } else {
            let commit = result.unwrap();
            assert!(commit.created(&root).unwrap().is_some());
            assert_eq!(dom.usage().string_code_units(), 62);
        }
        let mut batch = DomBatch::new();
        batch.attach_shadow(host, DomShadowRootInit::new(Mode::Closed));
        batch.insert_before(doc, host, None);
        assert!(dom.apply(batch, &AtomicBool::new(false)).is_err());
    }
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let host = batch.create_element_ns(&doc, Namespace::from_uri(HTML), "div");
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&doc).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    let rejected = batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
    batch.insert_before(doc, host, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let mut batch = DomBatch::new();
    let root = batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
    let commit = apply(&mut dom, batch);
    assert_eq!(
        commit.created(&rejected).unwrap_err(),
        DomError::ForeignBatch
    );
    assert!(commit.created(&root).unwrap().is_some());
}
#[test]
fn no_op_root_detach_and_transient_owned_tree_do_not_advance_revision_or_history() {
    let mut dom = dom();
    let (document, _, root) = base(&mut dom, Mode::Open, false);
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.detach(root);
    let temporary = batch.create_element_ns(document, Namespace::from_uri(HTML), "span");
    let temp_root = batch.attach_shadow(&temporary, DomShadowRootInit::new(Mode::Open));
    batch.destroy(&temporary);
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.created(&temporary).unwrap(), None);
    assert_eq!(commit.created(&temp_root).unwrap(), None);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(commit.changes().changes().is_empty());
    let mut batch = DomBatch::new();
    batch.attach_shadow(root, DomShadowRootInit::new(Mode::Open));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    dom.set_limits(
        DomLimits::new(
            256,
            65536,
            512,
            0,
            0,
            0,
            0,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "new");
    batch.insert_before(&text, root, None);
    apply(&mut dom, batch);
    assert!(matches!(
        dom.changes_since(before.stamp()),
        Err(DomHistoryError::FullRecomputeRequired { .. })
    ));
}
#[test]
fn foreign_host_and_foreign_creation_tickets_cannot_attach_roots() {
    let mut a = dom();
    let (document, host, _) = base(&mut a, Mode::Open, false);
    let mut b = dom();
    let before = b.snapshot();
    let mut batch = DomBatch::new();
    batch.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
    assert_eq!(
        b.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(b.snapshot().stamp(), before.stamp());
    let mut first = DomBatch::new();
    let never_published = first.attach_shadow(host, DomShadowRootInit::new(Mode::Open));
    let mut second = DomBatch::new();
    second.insert_before(&never_published, document, None);
    assert_eq!(
        a.apply(second, &AtomicBool::new(false)).unwrap_err(),
        DomError::ForeignBatch
    );
}

#[test]
fn late_shadow_import_node_or_string_limit_failure_rolls_back_the_whole_private_graph() {
    let mut source = dom();
    let (doc, host, root) = base(&mut source, Mode::Open, true);
    let mut batch = DomBatch::new();
    let text = batch.create_text(doc, "x");
    batch.insert_before(&text, root, None);
    apply(&mut source, batch);
    for (nodes, units, error) in [
        (
            3,
            63,
            Some(DomError::ResourceLimit {
                resource: DomResource::ManagedNodes,
                limit: 3,
            }),
        ),
        (
            4,
            62,
            Some(DomError::ResourceLimit {
                resource: DomResource::StringCodeUnits,
                limit: 62,
            }),
        ),
        (4, 63, None),
    ] {
        let mut target = Dom::new(
            DomLimits::new(
                nodes,
                units,
                16,
                0,
                0,
                8,
                8192,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = apply(&mut target, batch);
        let document = commit.document(&document).unwrap();
        let before = target.snapshot();
        let usage = target.usage();
        let mut batch = DomBatch::new();
        let copy = batch.import_node(&source.snapshot(), host, document, false);
        match error {
            Some(error) => {
                assert_eq!(
                    target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                    error
                );
                assert_eq!(target.snapshot().stamp(), before.stamp());
                assert_eq!(target.usage(), usage);
                assert!(
                    target
                        .changes_since(before.stamp())
                        .unwrap()
                        .changes()
                        .is_empty()
                );
            }
            None => {
                let commit = apply(&mut target, batch);
                let copy = commit.created(&copy).unwrap().unwrap();
                assert_eq!(
                    target
                        .snapshot()
                        .node(copy)
                        .unwrap()
                        .shadow_including_tree_order()
                        .count(),
                    3
                );
                assert_eq!(target.usage().managed_nodes(), 4);
                assert_eq!(target.usage().string_code_units(), 63);
            }
        }
    }
}
#[test]
fn source_valid_builtin_hosts_and_actual_custom_name_grammar_are_admitted() {
    for name in [
        "article",
        "aside",
        "blockquote",
        "body",
        "div",
        "footer",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "main",
        "nav",
        "p",
        "section",
        "span",
        "x-panel",
        "x-é",
    ] {
        let mut dom = dom();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let host = batch.create_element_ns(&document, Namespace::from_uri(HTML), name);
        let root = batch.attach_shadow(&host, DomShadowRootInit::new(Mode::Open));
        let commit = apply(&mut dom, batch);
        assert!(
            dom.snapshot()
                .shadow_root(commit.created(&root).unwrap().unwrap())
                .is_ok()
        );
    }
}
