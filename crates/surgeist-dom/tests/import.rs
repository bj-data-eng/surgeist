use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomError, DomLimits, DomNodeKind,
    DomResource, DomSnapshot, Namespace, NodeId,
};

fn limits() -> DomLimits {
    DomLimits::new(
        64,
        4096,
        128,
        0,
        0,
        16,
        65536,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}
fn dom() -> Dom {
    Dom::new(limits()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn new_document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    apply(dom, batch).document(&document).unwrap()
}

struct Source {
    dom: Dom,
    document: DocumentId,
    fragment: NodeId,
    root: NodeId,
    text: NodeId,
    comment: NodeId,
    pi: NodeId,
    doctype: NodeId,
}
fn source() -> Source {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let root = batch.create_element(&document, "MiXeD:Root");
    let text = batch.create_text(
        &document,
        DocumentString::from_utf16(vec![0xd800, 0x41, 0xdc00]),
    );
    let comment = batch.create_comment(&document, "comment");
    let pi = batch.create_processing_instruction(&document, "pi", "a='1'");
    let child = batch.create_element(&document, "Child");
    let leaf = batch.create_text(&document, "leaf");
    let doctype = batch.create_document_type(
        &document,
        "",
        DocumentString::from_utf16(vec![0xd800]),
        "system",
    );
    batch.set_attribute(&root, "xml:lang", "literal");
    batch.set_attribute_ns(
        &root,
        Namespace::from_uri("urn:x"),
        "p:key",
        DocumentString::from_utf16(vec![0xdc00, 0xd800]),
    );
    batch.set_attribute(&root, "is", "x-test");
    for node in [&text, &comment, &pi, &child] {
        batch.insert_before(node, &root, None);
    }
    batch.insert_before(&leaf, &child, None);
    batch.insert_before(&root, &fragment, None);
    let commit = apply(&mut dom, batch);
    Source {
        document: commit.document(&document).unwrap(),
        fragment: commit.created(&fragment).unwrap().unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        text: commit.created(&text).unwrap().unwrap(),
        comment: commit.created(&comment).unwrap().unwrap(),
        pi: commit.created(&pi).unwrap().unwrap(),
        doctype: commit.created(&doctype).unwrap().unwrap(),
        dom,
    }
}

#[test]
fn shallow_element_import_copies_all_exact_attributes_and_has_fresh_detached_identity() {
    let source = source();
    let snapshot = source.dom.snapshot();
    let source_stamp = snapshot.stamp();
    let mut target = dom();
    let target_document = new_document(&mut target);
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.root, target_document, false);
    let report = apply(&mut target, batch);
    let imported = report.created(&imported).unwrap().unwrap();
    let capture = target.snapshot();
    let node = capture.node(imported).unwrap();
    let element = node.as_element().unwrap();
    assert_ne!(imported, source.root);
    assert_eq!(imported.owner(), target.owner());
    assert_eq!(node.owner_document(), Some(target_document));
    assert_eq!(node.parent(), None);
    assert!(node.children().is_empty());
    assert_eq!(node.node_name(), DocumentString::from("MiXeD:Root"));
    assert_eq!(element.attributes().len(), 3);
    assert_eq!(
        element.attributes(),
        snapshot
            .node(source.root)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
    );
    assert_eq!(
        element.attribute_ns(&Namespace::none(), &"xml:lang".into()),
        Some(&DocumentString::from("literal"))
    );
    assert_eq!(
        element.attributes()[1].name().prefix(),
        Some(&DocumentString::from("p"))
    );
    assert_eq!(element.saved_is(), None);
    assert_eq!(report.changes().changes(), [DomChange::Created(imported)]);
    assert_eq!(source.dom.snapshot().stamp(), source_stamp);
    assert_eq!(
        snapshot.node(source.root).unwrap().parent(),
        Some(source.fragment)
    );
    assert_eq!(snapshot.node(source.root).unwrap().children().len(), 4);
}

#[test]
fn deep_fragment_import_preserves_mixed_order_and_every_copy_uses_target_document() {
    let source = source();
    let snapshot = source.dom.snapshot();
    let source_ids = snapshot.nodes().collect::<Vec<_>>();
    let mut target = dom();
    let document = new_document(&mut target);
    let before = target.snapshot();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.fragment, document, true);
    let report = apply(&mut target, batch);
    let imported = report.created(&imported).unwrap().unwrap();
    let capture = target.snapshot();
    let fragment = capture.node(imported).unwrap();
    assert!(matches!(fragment.kind(), DomNodeKind::DocumentFragment));
    assert_eq!(fragment.children().len(), 1);
    let root = capture.node(fragment.children()[0]).unwrap();
    assert_eq!(root.node_name(), DocumentString::from("MiXeD:Root"));
    assert_eq!(root.children().len(), 4);
    let children = root.children();
    assert_eq!(
        capture
            .node(children[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [0xd800, 0x41, 0xdc00]
    );
    assert_eq!(
        capture.node(children[1]).unwrap().data(),
        Some(&DocumentString::from("comment"))
    );
    assert_eq!(
        capture.node(children[2]).unwrap().node_name(),
        DocumentString::from("pi")
    );
    assert_eq!(
        capture.node(children[3]).unwrap().node_name(),
        DocumentString::from("Child")
    );
    let leaf = capture
        .node(capture.node(children[3]).unwrap().children()[0])
        .unwrap();
    assert_eq!(leaf.data(), Some(&DocumentString::from("leaf")));
    assert_eq!(report.changes().changes().len(), 7);
    for change in report.changes().changes() {
        let DomChange::Created(id) = *change else {
            panic!("import creates fresh state only")
        };
        assert!(!source_ids.contains(&id));
        assert_eq!(id.owner(), target.owner());
        assert_eq!(capture.node(id).unwrap().owner_document(), Some(document));
    }
    assert_eq!(capture.revision().get(), before.revision().get() + 1);
    assert_eq!(
        snapshot.node(source.fragment).unwrap().owner_document(),
        Some(source.document)
    );
    assert_eq!(
        snapshot.node(source.fragment).unwrap().children(),
        [source.root]
    );
}

#[test]
fn character_data_and_doctype_import_copy_actual_values_including_edited_pi_terminator() {
    let mut source = source();
    let mut edit = DomBatch::new();
    edit.replace_data(source.pi, 0, usize::MAX, "a='?>'");
    apply(&mut source.dom, edit);
    let snapshot = source.dom.snapshot();
    let mut target = dom();
    let document = new_document(&mut target);
    let mut batch = DomBatch::new();
    let text = batch.import_node(&snapshot, source.text, document, false);
    let comment = batch.import_node(&snapshot, source.comment, document, true);
    let pi = batch.import_node(&snapshot, source.pi, document, true);
    let doctype = batch.import_node(&snapshot, source.doctype, document, true);
    let report = apply(&mut target, batch);
    let capture = target.snapshot();
    for (ticket, original) in [
        (&text, source.text),
        (&comment, source.comment),
        (&pi, source.pi),
        (&doctype, source.doctype),
    ] {
        let id = report.created(ticket).unwrap().unwrap();
        assert_ne!(id, original);
        assert_eq!(
            capture.node(id).unwrap().kind(),
            snapshot.node(original).unwrap().kind()
        );
        assert!(capture.node(id).unwrap().children().is_empty());
    }
    let DomNodeKind::ProcessingInstruction(value) = capture
        .node(report.created(&pi).unwrap().unwrap())
        .unwrap()
        .kind()
    else {
        panic!("expected PI")
    };
    assert_eq!(value.data(), &DocumentString::from("a='?>'"));
    assert_eq!(
        value.attribute(&"a".into()),
        Some(&DocumentString::from("?>"))
    );
    let DomNodeKind::DocumentType(value) = capture
        .node(report.created(&doctype).unwrap().unwrap())
        .unwrap()
        .kind()
    else {
        panic!("expected doctype")
    };
    assert_eq!(value.name(), &DocumentString::from(""));
    assert_eq!(value.public_id().code_units(), [0xd800]);
    assert_eq!(value.system_id(), &DocumentString::from("system"));
}

#[test]
fn native_tickets_before_and_after_private_descendants_keep_their_own_nodes() {
    let source = source();
    let snapshot = source.dom.snapshot();
    let mut target = dom();
    let mut batch = DomBatch::new();
    let first_document = batch.create_document();
    let before = batch.create_comment(&first_document, "before");
    let imported = batch.import_node(&snapshot, source.root, &first_document, true);
    let after = batch.create_text(&first_document, "after");
    let second_document = batch.create_document();
    let shallow = batch.import_node(&snapshot, source.root, &second_document, false);
    let last = batch.create_comment(&second_document, "last");
    batch.set_attribute(&imported, "changed", "later command");
    batch.insert_before(&before, &first_document, None);
    batch.insert_before(&imported, &first_document, None);
    let report = apply(&mut target, batch);
    let first_document = report.document(&first_document).unwrap();
    let second_document = report.document(&second_document).unwrap();
    let capture = target.snapshot();
    let before = report.created(&before).unwrap().unwrap();
    let imported = report.created(&imported).unwrap().unwrap();
    let after = report.created(&after).unwrap().unwrap();
    let shallow = report.created(&shallow).unwrap().unwrap();
    let last = report.created(&last).unwrap().unwrap();
    assert_eq!(
        capture.node(first_document.node()).unwrap().children(),
        [before, imported]
    );
    assert_eq!(
        capture.node(after).unwrap().data(),
        Some(&DocumentString::from("after"))
    );
    assert_eq!(
        capture
            .node(imported)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"changed".into()),
        Some(&DocumentString::from("later command"))
    );
    assert_eq!(
        capture.node(shallow).unwrap().owner_document(),
        Some(second_document)
    );
    assert!(capture.node(shallow).unwrap().children().is_empty());
    assert_eq!(
        capture.node(last).unwrap().data(),
        Some(&DocumentString::from("last"))
    );
    assert_eq!(
        capture.node(last).unwrap().owner_document(),
        Some(second_document)
    );
}

#[test]
fn historical_source_capture_imports_after_source_destruction_in_the_same_store() {
    let mut source = source();
    let old = source.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(source.fragment);
    let target_document = batch.create_document();
    let report = apply(&mut source.dom, batch);
    let document = report.document(&target_document).unwrap();
    assert_eq!(
        source.dom.snapshot().node(source.root).unwrap_err(),
        DomError::StaleNode
    );
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&old, source.root, document, true);
    let imported = apply(&mut source.dom, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    assert_ne!(imported, source.root);
    let capture = source.dom.snapshot();
    assert_eq!(
        capture.node(imported).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(capture.node(imported).unwrap().children().len(), 4);
    assert_eq!(capture.node(source.root).unwrap_err(), DomError::StaleNode);
    assert_eq!(
        old.node(source.root).unwrap().owner_document(),
        Some(source.document)
    );
    assert_eq!(
        old.node(source.root).unwrap().parent(),
        Some(source.fragment)
    );
}

#[test]
fn same_store_live_source_still_clones_and_capture_can_outlive_its_store() {
    let mut source = source();
    let snapshot = source.dom.snapshot();
    let document = source.document;
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.root, document, true);
    let imported = apply(&mut source.dom, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    assert_ne!(imported, source.root);
    assert_eq!(
        source.dom.snapshot().node(source.root).unwrap().parent(),
        Some(source.fragment)
    );
    let root = source.root;
    drop(source.dom);
    let mut target = dom();
    let document = new_document(&mut target);
    let mut batch = DomBatch::new();
    let copied = batch.import_node(&snapshot, root, document, true);
    let copied = apply(&mut target, batch).created(&copied).unwrap().unwrap();
    assert_eq!(target.snapshot().node(copied).unwrap().children().len(), 4);
    assert_eq!(snapshot.node(root).unwrap().children().len(), 4);
}

#[test]
fn destroyed_import_reclaims_private_descendants_and_does_not_publish_identities() {
    let source = source();
    let snapshot = source.dom.snapshot();
    let mut target = dom();
    let document = new_document(&mut target);
    let before = target.snapshot();
    let usage = target.usage();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.root, document, true);
    batch.destroy(&imported);
    let transient = batch.create_comment(document, "also unpublished");
    batch.destroy(&transient);
    let report = apply(&mut target, batch);
    assert_eq!(report.created(&imported).unwrap(), None);
    assert_eq!(report.created(&transient).unwrap(), None);
    assert!(report.changes().changes().is_empty());
    assert_eq!(target.snapshot().stamp(), before.stamp());
    assert_eq!(target.usage(), usage);
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.root, document, true);
    batch.destroy(&imported);
    batch.detach(&imported);
    assert_eq!(
        target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::UnavailableCreation
    );
    assert_eq!(target.snapshot().stamp(), before.stamp());
}

#[test]
fn import_rejects_document_foreign_source_stale_source_and_invalid_target_atomically() {
    let mut source = source();
    let old = source.dom.snapshot();
    let mut batch = DomBatch::new();
    batch.destroy(source.fragment);
    apply(&mut source.dom, batch);
    let current = source.dom.snapshot();
    let mut target = dom();
    let document = new_document(&mut target);
    let before = target.snapshot();
    for (snapshot, node, error) in [
        (&old, source.document.node(), DomError::NotSupported),
        (&old, document.node(), DomError::ForeignOwner),
        (&current, source.root, DomError::StaleNode),
    ] {
        let mut batch = DomBatch::new();
        batch.import_node(snapshot, node, document, true);
        assert_eq!(
            target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(target.snapshot().stamp(), before.stamp());
    }
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "wrong target");
    batch.import_node(&old, source.root, &text, false);
    assert_eq!(
        target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::WrongNodeKind
    );
    assert_eq!(target.snapshot().stamp(), before.stamp());
    assert_eq!(target.snapshot().nodes().count(), 1);
}

#[test]
fn resource_failure_after_partial_subtree_copy_and_late_command_leave_no_state() {
    let source = source();
    let snapshot = source.dom.snapshot();
    let mut target = Dom::new(
        DomLimits::new(
            4,
            4096,
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
    let document = new_document(&mut target);
    let before = target.snapshot();
    let usage = target.usage();
    let mut batch = DomBatch::new();
    let failed = batch.import_node(&snapshot, source.root, document, true);
    assert_eq!(
        target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::ManagedNodes,
            limit: 4
        }
    );
    assert_eq!(target.snapshot().stamp(), before.stamp());
    assert_eq!(target.usage(), usage);
    target.set_limits(limits()).unwrap();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, source.root, document, true);
    batch.replace_data(&imported, 0, 0, "wrong kind");
    assert_eq!(
        target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::WrongNodeKind
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
    let mut batch = DomBatch::new();
    let next = batch.create_comment(document, "next");
    let report = apply(&mut target, batch);
    assert_eq!(report.created(&failed), Err(DomError::ForeignBatch));
    assert_eq!(
        target
            .snapshot()
            .node(report.created(&next).unwrap().unwrap())
            .unwrap()
            .data(),
        Some(&DocumentString::from("next"))
    );
}

fn attribute_source() -> (DomSnapshot, NodeId) {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "R");
    batch.set_attribute_ns(&root, Namespace::from_uri("urn:a"), "p:x", "😀");
    let root = apply(&mut dom, batch).created(&root).unwrap().unwrap();
    (dom.snapshot(), root)
}

#[test]
fn import_counts_exact_logical_strings_even_when_immutable_backing_is_shared() {
    let (snapshot, root) = attribute_source();
    // 31 target Document + 1 Element + 5 URI + 1 prefix + 1 local + 2 value.
    for (limit, succeeds) in [(40, false), (41, true)] {
        let mut target = Dom::new(
            DomLimits::new(
                2,
                limit,
                4,
                0,
                0,
                8,
                8192,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let document = new_document(&mut target);
        let before = target.snapshot();
        let mut batch = DomBatch::new();
        batch.import_node(&snapshot, root, document, false);
        if succeeds {
            apply(&mut target, batch);
            assert_eq!(target.usage().string_code_units(), 41);
            assert_eq!(target.usage().managed_nodes(), 2);
        } else {
            assert_eq!(
                target.apply(batch, &AtomicBool::new(false)).unwrap_err(),
                DomError::ResourceLimit {
                    resource: DomResource::StringCodeUnits,
                    limit
                }
            );
            assert_eq!(target.snapshot().stamp(), before.stamp());
            assert_eq!(target.usage().string_code_units(), 31);
        }
    }
}

#[test]
fn deep_import_walks_bounded_tree_without_recursive_native_calls() {
    let limits = DomLimits::new(
        1100,
        4096,
        3000,
        0,
        0,
        0,
        0,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap();
    let mut source = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "n");
    let mut parent = root.clone();
    for _ in 1..1024 {
        let child = batch.create_element(&document, "n");
        batch.insert_before(&child, &parent, None);
        parent = child;
    }
    let root = apply(&mut source, batch).created(&root).unwrap().unwrap();
    let snapshot = source.snapshot();
    let mut target = Dom::new(limits).unwrap();
    let document = new_document(&mut target);
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&snapshot, root, document, true);
    let imported = apply(&mut target, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    let capture = target.snapshot();
    let mut current = imported;
    for depth in 0..1024 {
        let node = capture.node(current).unwrap();
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.node_name(), DocumentString::from("n"));
        if depth == 1023 {
            assert!(node.children().is_empty());
        } else {
            assert_eq!(node.children().len(), 1);
            current = node.children()[0];
        }
    }
    assert_eq!(target.usage().managed_nodes(), 1025);
}
