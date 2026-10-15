use std::sync::atomic::AtomicBool;

use surgeist_dom::{DocumentString, Dom, DomBatch, DomError, DomLimits};

// Selected DOM insert adopts each inserted node into the parent's document.
// Adoption changes node-document, not canonical store identity. A fragment
// supplies its children and retains its own node-document; empty insert is a no-op.
fn dom() -> Dom {
    Dom::new(DomLimits::new(32, 1024, 64, 0, 0, 8, 8192).unwrap()).unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> surgeist_dom::DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

#[test]
fn detached_text_adopts_without_new_capacity_and_cancelled_attempt_is_invisible() {
    // Two default Documents retain 62 units; this exact text retains 3 more.
    let mut dom = Dom::new(DomLimits::new(4, 65, 8, 0, 0, 0, 0).unwrap()).unwrap();
    let text_value = DocumentString::from_utf16(vec![0xd800, 0x41, 0xdc00]);
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&target);
    let text = batch.create_text(&source, text_value.clone());
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut cancelled = DomBatch::new();
    cancelled.insert_before(text, fragment, None);
    assert_eq!(
        dom.apply(cancelled, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot().node(text).unwrap().owner_document(),
        Some(source)
    );
    assert_eq!(dom.usage(), usage);

    let mut batch = DomBatch::new();
    batch.insert_before(text, fragment, None);
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.revision().get(), before.revision().get() + 1);
    let actual = after.node(text).unwrap();
    assert_eq!(actual.id(), text);
    assert_eq!(actual.owner_document(), Some(target));
    assert_eq!(actual.parent(), Some(fragment));
    assert_eq!(actual.data(), Some(&text_value));
    assert_eq!(dom.usage(), usage);
    assert_eq!(before.node(text).unwrap().owner_document(), Some(source));
    assert_eq!(before.node(text).unwrap().parent(), None);
    assert_eq!(
        after.document(source).unwrap().node(text).unwrap_err(),
        DomError::WrongDocument
    );
    assert_eq!(
        after.document(target).unwrap().node(text).unwrap().id(),
        text
    );
}

#[test]
fn attached_comments_and_pi_move_between_documents_in_requested_order() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let comment = batch.create_comment(&source, "moved");
    let pi = batch.create_processing_instruction(&source, "target", "a='1'");
    let left = batch.create_comment(&target, "left");
    let right = batch.create_comment(&target, "right");
    batch.insert_before(&comment, &source, None);
    batch.insert_before(&pi, &source, None);
    batch.insert_before(&left, &target, None);
    batch.insert_before(&right, &target, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let pi = commit.created(&pi).unwrap().unwrap();
    let left = commit.created(&left).unwrap().unwrap();
    let right = commit.created(&right).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(comment, target, Some(right.into()));
    batch.insert_before(pi, target, Some(right.into()));
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.revision().get(), before.revision().get() + 1);
    assert!(after.node(source.node()).unwrap().children().is_empty());
    assert_eq!(
        after.node(target.node()).unwrap().children(),
        [left, comment, pi, right]
    );
    for id in [comment, pi] {
        let node = after.node(id).unwrap();
        assert_eq!(node.id(), id);
        assert_eq!(node.owner_document(), Some(target));
        assert_eq!(node.parent(), Some(target.node()));
        assert!(node.is_connected());
        assert_eq!(node.kind(), before.node(id).unwrap().kind());
    }
    assert_eq!(
        before.node(source.node()).unwrap().children(),
        [comment, pi]
    );
    assert_eq!(
        before.node(target.node()).unwrap().children(),
        [left, right]
    );
}

#[test]
fn transferred_fragment_children_adopt_while_fragment_retains_its_source_document() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let source_fragment = batch.create_fragment(&source);
    let target_fragment = batch.create_fragment(&target);
    let text = batch.create_text(&source, "text");
    let comment = batch.create_comment(&source, "comment");
    let pi = batch.create_processing_instruction(&source, "pi", "data");
    let marker = batch.create_comment(&target, "marker");
    for child in [&text, &comment, &pi] {
        batch.insert_before(child, &source_fragment, None);
    }
    batch.insert_before(&marker, &target_fragment, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let source_fragment = commit.created(&source_fragment).unwrap().unwrap();
    let target_fragment = commit.created(&target_fragment).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let pi = commit.created(&pi).unwrap().unwrap();
    let marker = commit.created(&marker).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(source_fragment, target_fragment, Some(marker.into()));
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert!(after.node(source_fragment).unwrap().children().is_empty());
    assert_eq!(after.node(source_fragment).unwrap().parent(), None);
    assert_eq!(
        after.node(target_fragment).unwrap().children(),
        [text, comment, pi, marker]
    );
    for id in [text, comment, pi] {
        assert_eq!(after.node(id).unwrap().owner_document(), Some(target));
        assert_eq!(before.node(id).unwrap().owner_document(), Some(source));
        assert_eq!(after.node(id).unwrap().id(), id);
    }
    assert_eq!(
        after.node(source_fragment).unwrap().owner_document(),
        Some(source)
    );
    assert_eq!(
        before.node(source_fragment).unwrap().children(),
        [text, comment, pi]
    );
    assert_eq!(before.node(target_fragment).unwrap().children(), [marker]);
}

#[test]
fn empty_foreign_document_fragment_insertion_is_a_semantic_noop() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&source);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, target, None);
    let report = apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert!(report.changes().changes().is_empty());
    assert_eq!(after.node(fragment).unwrap().owner_document(), Some(source));
    assert_eq!(after.node(fragment).unwrap().parent(), None);
    assert!(after.node(target.node()).unwrap().children().is_empty());
    assert_eq!(
        dom.changes_since(before.stamp()).unwrap().changes(),
        report.changes().changes()
    );
    let mut noop = DomBatch::new();
    noop.insert_before(fragment, target, None);
    assert!(apply(&mut dom, noop).changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), after.stamp());
}

#[test]
fn failure_after_staged_adoption_rolls_back_document_relations_identity_and_history() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let comment = batch.create_comment(&source, "kept");
    batch.insert_before(&comment, &source, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.create_comment(target, "not published");
    batch.insert_before(comment, target, None);
    batch.replace_data(comment, 99, 0, "invalid offset");
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::IndexSize
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        after.nodes().collect::<Vec<_>>(),
        before.nodes().collect::<Vec<_>>()
    );
    assert_eq!(after.node(comment).unwrap().owner_document(), Some(source));
    assert_eq!(after.node(comment).unwrap().parent(), Some(source.node()));
    assert_eq!(after.node(source.node()).unwrap().children(), [comment]);
    assert!(after.node(target.node()).unwrap().children().is_empty());
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
}

#[test]
fn cross_document_fragment_preinsert_rejects_text_for_document_before_adoption() {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let target = batch.create_document();
    let fragment = batch.create_fragment(&source);
    let text = batch.create_text(&source, "forbidden");
    batch.insert_before(&text, &fragment, None);
    let commit = apply(&mut dom, batch);
    let source = commit.document(&source).unwrap();
    let target = commit.document(&target).unwrap();
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.insert_before(fragment, target, None);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    for id in [fragment, text] {
        assert_eq!(
            dom.snapshot().node(id).unwrap().owner_document(),
            Some(source)
        );
    }
    assert_eq!(dom.snapshot().node(fragment).unwrap().children(), [text]);
    assert_eq!(dom.snapshot().node(text).unwrap().parent(), Some(fragment));
}
