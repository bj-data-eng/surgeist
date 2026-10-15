use std::sync::atomic::AtomicBool;
use surgeist_dom::{Dom, DomBatch, DomChange, DomError, DomLimits, Namespace};

fn dom() -> Dom {
    Dom::new(DomLimits::new(64, 4096, 128, 16, 65536).unwrap()).unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn documents_enforce_single_element_doctype_order_and_invalid_reference_before_mutation() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "Root");
    let doctype = batch.create_document_type(&document, "", "", "");
    let comment = batch.create_comment(&document, "middle");
    let second = batch.create_element(&document, "Other");
    batch.insert_before(&doctype, &document, None);
    batch.insert_before(&comment, &document, None);
    batch.insert_before(&root, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let before = dom.snapshot();
    for (node, reference, error) in [
        (second, None, DomError::HierarchyRequest),
        (root, Some(root), DomError::HierarchyRequest),
        (second, Some(second), DomError::NotFound),
    ] {
        let mut batch = DomBatch::new();
        batch.insert_before(node, document, reference.map(Into::into));
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    let mut batch = DomBatch::new();
    batch.detach(root);
    batch.insert_before(root, document, Some(doctype.into()));
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut batch = DomBatch::new();
    batch.detach(doctype);
    batch.insert_before(doctype, document, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(
        dom.snapshot().node(document.node()).unwrap().children(),
        [doctype, comment, root]
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn replacement_excludes_only_the_replaced_child_and_retains_detached_identity() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let old = batch.create_element(&document, "old");
    let new = batch.create_element(&document, "new");
    let doctype = batch.create_document_type(&document, "old", "", "");
    let fresh_doctype = batch.create_document_type(&document, "new", "", "");
    batch.insert_before(&doctype, &document, None);
    batch.insert_before(&old, &document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let new = commit.created(&new).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let fresh_doctype = commit.created(&fresh_doctype).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_child(new, document, old);
    batch.replace_child(fresh_doctype, document, doctype);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(
        after.node(document.node()).unwrap().children(),
        [fresh_doctype, new]
    );
    assert_eq!(after.node(old).unwrap().parent(), None);
    assert_eq!(after.node(doctype).unwrap().parent(), None);
    assert_eq!(
        before.node(document.node()).unwrap().children(),
        [doctype, old]
    );
    let stamp = after.stamp();
    let mut batch = DomBatch::new();
    batch.replace_child(new, document, new);
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), stamp);
    let mut batch = DomBatch::new();
    batch.replace_child(new, document, fresh_doctype);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn fragment_document_validation_counts_only_direct_elements_and_rejects_text_atomically() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let first = batch.create_element(&document, "first");
    let second = batch.create_element(&document, "second");
    let nested = batch.create_element(&document, "nested");
    batch.insert_before(&nested, &first, None);
    batch.insert_before(&first, &fragment, None);
    batch.insert_before(&second, &fragment, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let first = commit.created(&first).unwrap().unwrap();
    let second = commit.created(&second).unwrap().unwrap();
    let nested = commit.created(&nested).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, document, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut batch = DomBatch::new();
    batch.detach(second);
    batch.insert_before(fragment, document, None);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot().node(document.node()).unwrap().children(),
        [first]
    );
    assert_eq!(dom.snapshot().node(first).unwrap().children(), [nested]);
    let mut batch = DomBatch::new();
    let text = batch.create_text(document, "text");
    batch.insert_before(&text, fragment, None);
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_child(fragment, document, first);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().node(fragment).unwrap().children(), [text]);
}

#[test]
fn elements_accept_text_and_pi_but_reject_doctypes_documents_and_cycles() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let child = batch.create_element(&document, "child");
    let text = batch.create_text(&document, "text");
    let pi = batch.create_processing_instruction(&document, "pi", "");
    let doctype = batch.create_document_type(&document, "", "", "");
    batch.insert_before(&child, &root, None);
    batch.insert_before(&text, &child, None);
    batch.insert_before(&pi, &child, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let pi = commit.created(&pi).unwrap().unwrap();
    let doctype = commit.created(&doctype).unwrap().unwrap();
    let before = dom.snapshot();
    for (node, parent) in [
        (doctype, root),
        (document.node(), root),
        (root, child),
        (root, text),
        (root, pi),
    ] {
        let mut batch = DomBatch::new();
        batch.insert_before(node, parent, None);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::HierarchyRequest
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
    let mut batch = DomBatch::new();
    batch.insert_before(child, root, Some(child.into()));
    assert!(apply(&mut dom, batch).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn explicit_adoption_preserves_subtree_ids_and_reports_document_only_changes_for_detached_root() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let root = batch.create_element(&source, "root");
    let child = batch.create_element(&source, "child");
    let text = batch.create_text(&source, "exact");
    batch.set_attribute_ns(&child, Namespace::from_uri("urn:x"), "p:name", "value");
    batch.insert_before(&child, &root, None);
    batch.insert_before(&text, &child, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(root, target);
    let report = apply(&mut dom, batch);
    assert_eq!(
        report.changes().changes(),
        [
            DomChange::NodeDocument(root),
            DomChange::NodeDocument(child),
            DomChange::NodeDocument(text)
        ]
    );
    let after = dom.snapshot();
    assert_eq!(after.node(root).unwrap().parent(), None);
    assert_eq!(after.node(root).unwrap().children(), [child]);
    for id in [root, child, text] {
        assert_eq!(after.node(id).unwrap().id(), id);
        assert_eq!(after.node(id).unwrap().owner_document(), Some(target));
        assert_eq!(before.node(id).unwrap().owner_document(), Some(source));
        assert_eq!(
            after.node(id).unwrap().kind(),
            before.node(id).unwrap().kind()
        );
    }
    assert_eq!(
        after.document(source).unwrap().node(root).unwrap_err(),
        DomError::WrongDocument
    );
    assert_eq!(
        dom.changes_since(before.stamp()).unwrap().changes(),
        report.changes().changes()
    );
    let mut noop = DomBatch::new();
    noop.adopt(root, target);
    assert!(apply(&mut dom, noop).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), after.stamp());
}

#[test]
fn explicit_fragment_adoption_includes_root_and_same_document_adoption_still_removes() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&source);
    let root = batch.create_element(&source, "root");
    let text = batch.create_text(&source, "text");
    batch.insert_before(&root, &fragment, None);
    batch.insert_before(&text, &root, None);
    let commit = apply(&mut dom, batch);
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(fragment, target);
    apply(&mut dom, batch);
    for id in [fragment, root, text] {
        assert_eq!(
            dom.snapshot().node(id).unwrap().owner_document(),
            Some(target)
        );
    }
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.adopt(root, target);
    let report = apply(&mut dom, batch);
    assert!(
        !report
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::NodeDocument(_)))
    );
    assert_eq!(dom.snapshot().node(root).unwrap().parent(), None);
    assert!(dom.snapshot().node(fragment).unwrap().children().is_empty());
    assert_eq!(before.node(root).unwrap().parent(), Some(fragment));
}

#[test]
fn replacement_adopts_fragment_root_unlike_insertion_and_preserves_source_snapshot() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&source);
    let replacement = batch.create_element(&source, "replacement");
    let old = batch.create_element(&target, "old");
    batch.insert_before(&replacement, &fragment, None);
    batch.insert_before(&old, &target, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let replacement = commit.created(&replacement).unwrap().unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_child(fragment, target, old);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.node(fragment).unwrap().owner_document(), Some(target));
    assert!(after.node(fragment).unwrap().children().is_empty());
    assert_eq!(
        after.node(replacement).unwrap().owner_document(),
        Some(target)
    );
    assert_eq!(after.node(target.node()).unwrap().children(), [replacement]);
    assert_eq!(after.node(old).unwrap().parent(), None);
    assert_eq!(
        before.node(fragment).unwrap().owner_document(),
        Some(source)
    );
    assert_eq!(before.node(fragment).unwrap().children(), [replacement]);
}

#[test]
fn adopt_rejects_documents_foreign_owners_wrong_targets_and_stale_nodes_atomically() {
    let mut other = dom();
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let stale = batch.create_comment(&document, "stale");
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let stale = commit.created(&stale).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(stale);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    let foreign = batch.create_document();
    let foreign = apply(&mut other, batch).document(&foreign).unwrap();
    let before = dom.snapshot();
    for (node, target, error) in [
        (document.node(), document.node(), DomError::NotSupported),
        (root, foreign.node(), DomError::ForeignOwner),
        (root, root, DomError::WrongNodeKind),
        (stale, document.node(), DomError::StaleNode),
    ] {
        let mut batch = DomBatch::new();
        batch.adopt(node, target);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
    }
}

#[test]
fn late_attribute_failure_rolls_back_creation_adoption_relations_and_metadata() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let root = batch.create_element(&source, "root");
    batch.set_attribute(&root, "kept", "old");
    batch.insert_before(&root, &source, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.create_element(target, "unpublished");
    batch.set_attribute(root, "kept", "unpublished");
    batch.adopt(root, target);
    batch.set_attribute(root, "bad name", "invalid");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::InvalidCharacter
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(after.nodes().count(), before.nodes().count());
    assert_eq!(after.node(root).unwrap().parent(), Some(source.node()));
    assert_eq!(after.node(root).unwrap().owner_document(), Some(source));
    assert_eq!(
        after.node(root).unwrap().kind(),
        before.node(root).unwrap().kind()
    );
    assert_eq!(
        after.document(source).unwrap().metadata(),
        before.document(source).unwrap().metadata()
    );
    assert_eq!(
        after.document(target).unwrap().metadata(),
        before.document(target).unwrap().metadata()
    );
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}
