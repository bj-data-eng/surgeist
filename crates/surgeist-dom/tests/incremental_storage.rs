//! Public behavior preserved by incremental canonical storage. Exact vacancy
//! indices and exhausted generations are private arena contracts: NodeId does
//! not expose either, and snapshot enumeration promises no arena order.
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomError, DomLimits,
    DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress, DomSourceRequest,
    DomStyleInlineCspObservation, Namespace, NodeId,
};

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            8192,
            262_144,
            16_384,
            0,
            0,
            64,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_ranges(32)
        .unwrap(),
    )
    .unwrap()
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_document();
    apply(dom, batch).document(&ticket).unwrap()
}

fn comment(dom: &mut Dom, document: DocumentId, data: &str) -> NodeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_comment(document, data);
    apply(dom, batch).created(&ticket).unwrap().unwrap()
}

fn source_limits(work: u64) -> DomSourceLimits {
    DomSourceLimits::new(256_000_000, 2_000_000, work, 64, 4096, 4096, 1)
}

#[test]
fn retired_holes_and_new_creations_preserve_survivors_and_historical_snapshots() {
    let mut dom = owner();
    let first_document = document(&mut dom);
    let second_document = document(&mut dom);
    let a = comment(&mut dom, first_document, "a");
    let b = comment(&mut dom, first_document, "b");
    let c = comment(&mut dom, second_document, "c");
    let d = comment(&mut dom, second_document, "d");
    let old = dom.snapshot();
    let mut retire = DomBatch::new();
    // Reverse retirement order must not affect surviving identities.
    retire.destroy(d);
    retire.destroy(b);
    apply(&mut dom, retire);
    let mut batch = DomBatch::new();
    let x = batch.create_comment(second_document, "x");
    let transient = batch.create_comment(first_document, "unpublished");
    batch.destroy(&transient);
    let y = batch.create_comment(first_document, "y");
    let commit = apply(&mut dom, batch);
    let x = commit.created(&x).unwrap().unwrap();
    let y = commit.created(&y).unwrap().unwrap();
    assert_eq!(commit.created(&transient).unwrap(), None);
    let now = dom.snapshot();
    let expected = [first_document.node(), second_document.node(), a, c, x, y];
    assert_eq!(now.nodes().count(), expected.len());
    for id in expected {
        assert!(now.nodes().any(|actual| actual == id));
        assert_eq!(now.node(id).unwrap().id(), id);
    }
    for retired in [b, d] {
        assert_eq!(now.node(retired).unwrap_err(), DomError::StaleNode);
        assert_ne!(retired, x);
        assert_ne!(retired, y);
        assert_eq!(old.node(retired).unwrap().id(), retired);
    }
    assert_eq!(now.node(x).unwrap().owner_document(), Some(second_document));
    assert_eq!(now.node(y).unwrap().owner_document(), Some(first_document));
    assert_eq!(old.node(x).unwrap_err(), DomError::StaleNode);
    assert_eq!(old.node(y).unwrap_err(), DomError::StaleNode);
    drop(dom);
    assert_eq!(
        old.node(b).unwrap().data(),
        Some(&DocumentString::from("b"))
    );
    assert_eq!(
        old.node(d).unwrap().data(),
        Some(&DocumentString::from("d"))
    );
}

#[test]
fn private_failed_and_cancelled_creations_leave_no_public_nodes_or_ticket_aliases() {
    let mut dom = owner();
    let document = document(&mut dom);
    let original = comment(&mut dom, document, "original");
    let old = dom.snapshot();
    let usage = dom.usage();
    let mut failed = DomBatch::new();
    let failed_ticket = failed.create_comment(document, "failed");
    failed.insert_before(&failed_ticket, original, None);
    assert_eq!(
        dom.apply(failed, &AtomicBool::new(false)).unwrap_err(),
        DomError::HierarchyRequest
    );
    let mut cancelled = DomBatch::new();
    let cancelled_ticket = cancelled.create_comment(document, "cancelled");
    assert_eq!(
        dom.apply(cancelled, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    let mut private = DomBatch::new();
    let ephemeral = private.create_comment(document, "ephemeral");
    private.destroy(&ephemeral);
    let private = apply(&mut dom, private);
    assert_eq!(private.created(&ephemeral).unwrap(), None);
    assert!(private.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), old.stamp());
    assert_eq!(dom.usage(), usage);
    let mut next = DomBatch::new();
    let next_ticket = next.create_comment(document, "next");
    let next = apply(&mut dom, next);
    let next_id = next.created(&next_ticket).unwrap().unwrap();
    for ticket in [&failed_ticket, &cancelled_ticket, &ephemeral] {
        assert_eq!(next.created(ticket).unwrap_err(), DomError::ForeignBatch);
    }
    assert_eq!(dom.snapshot().nodes().count(), 3);
    assert_eq!(old.node(next_id).unwrap_err(), DomError::StaleNode);
    assert_eq!(old.node(original).unwrap().id(), original);
}

#[test]
fn retired_and_foreign_node_operands_cannot_mutate_a_replacement() {
    let mut dom = owner();
    let document = document(&mut dom);
    let retired = comment(&mut dom, document, "old");
    let mut batch = DomBatch::new();
    batch.destroy(retired);
    apply(&mut dom, batch);
    let replacement = comment(&mut dom, document, "replacement");
    let mut other = owner();
    let foreign = self::document(&mut other).node();
    let old = dom.snapshot();
    for (operand, expected) in [
        (retired, DomError::StaleNode),
        (foreign, DomError::ForeignOwner),
    ] {
        let mut batch = DomBatch::new();
        batch.replace_data(replacement, 0, usize::MAX, "must roll back");
        batch.destroy(operand);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            expected
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        assert_eq!(
            dom.snapshot().node(replacement).unwrap().data(),
            Some(&DocumentString::from("replacement"))
        );
    }
}

#[test]
fn range_retirement_and_recreation_preserve_old_boundaries_and_reject_old_ids() {
    let mut dom = owner();
    let document = document(&mut dom);
    let text = {
        let mut batch = DomBatch::new();
        let text = batch.create_text(document, "abcdef");
        apply(&mut dom, batch).created(&text).unwrap().unwrap()
    };
    let mut batch = DomBatch::new();
    let range = batch.create_range(document);
    batch.set_range_start(&range, text, 1);
    batch.set_range_end(&range, text, 4);
    let retired = apply(&mut dom, batch)
        .created_range(&range)
        .unwrap()
        .unwrap();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.retire_range(retired);
    let replacement = batch.create_range(document);
    let private = batch.create_range(document);
    batch.retire_range(&private);
    let commit = apply(&mut dom, batch);
    let replacement = commit.created_range(&replacement).unwrap().unwrap();
    assert_eq!(commit.created_range(&private).unwrap(), None);
    assert_ne!(replacement, retired);
    assert_eq!(
        dom.snapshot().range(retired).unwrap_err(),
        DomError::StaleRange
    );
    assert_eq!(old.range(retired).unwrap().start().container(), text);
    assert_eq!(old.range(retired).unwrap().start().offset(), 1);
    assert_eq!(old.range(retired).unwrap().end().offset(), 4);
    let now = dom.snapshot();
    let range = now.range(replacement).unwrap();
    assert_eq!(range.start().container(), document.node());
    assert_eq!(range.start().offset(), 0);
    assert!(range.collapsed());
    let mut other = owner();
    let other_document = self::document(&mut other);
    let mut batch = DomBatch::new();
    let foreign = batch.create_range(other_document);
    let foreign = apply(&mut other, batch)
        .created_range(&foreign)
        .unwrap()
        .unwrap();
    for (id, error) in [
        (retired, DomError::StaleRange),
        (foreign, DomError::ForeignOwner),
    ] {
        let mut batch = DomBatch::new();
        batch.set_range_end(id, text, 2);
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            error
        );
        assert_eq!(dom.snapshot().stamp(), now.stamp());
    }
}

#[test]
fn captured_subtree_import_reuses_capacity_without_reusing_source_or_retired_identities() {
    let mut source = owner();
    let mut batch = DomBatch::new();
    let source_document = batch.create_document();
    let root = batch.create_element(&source_document, "root");
    let child = batch.create_text(&source_document, "literal child");
    batch.set_attribute(&root, "key", "literal attribute");
    batch.insert_before(&child, &root, None);
    let commit = apply(&mut source, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let captured = source.snapshot();
    let mut target = owner();
    let document = document(&mut target);
    let retired = comment(&mut target, document, "vacancy");
    let mut batch = DomBatch::new();
    batch.destroy(retired);
    apply(&mut target, batch);
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&captured, root, document, true);
    let imported = apply(&mut target, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    let now = target.snapshot();
    let imported_child = now.node(imported).unwrap().children()[0];
    assert_ne!(imported, root);
    assert_ne!(imported, retired);
    assert_ne!(imported_child, child);
    assert_eq!(now.node(imported).unwrap().parent(), None);
    assert_eq!(
        now.node(imported_child).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(now.node(imported_child).unwrap().parent(), Some(imported));
    assert_eq!(
        now.node(imported_child).unwrap().data(),
        Some(&DocumentString::from("literal child"))
    );
    assert_eq!(
        now.node(imported)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"key".into()),
        Some(&DocumentString::from("literal attribute"))
    );
    assert_eq!(now.node(retired).unwrap_err(), DomError::StaleNode);
    assert_eq!(captured.node(root).unwrap().children(), &[child]);
}

#[test]
fn attribute_edit_preserves_a_large_unchanged_child_sequence_and_exact_change_coverage() {
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let children: Vec<_> = (0..1024)
        .map(|_| batch.create_comment(&document, "child"))
        .collect();
    for child in &children {
        batch.insert_before(child, &root, None);
    }
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let children: Vec<_> = children
        .iter()
        .map(|ticket| commit.created(ticket).unwrap().unwrap())
        .collect();
    let old = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-local", "changed");
    let commit = apply(&mut dom, batch);
    assert_eq!(commit.changes().changes(), &[DomChange::Attributes(root)]);
    let now = dom.snapshot();
    assert_eq!(old.node(root).unwrap().children(), children);
    assert_eq!(now.node(root).unwrap().children(), children);
    assert_eq!(
        old.node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"data-local".into()),
        None
    );
    assert_eq!(
        now.node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"data-local".into()),
        Some(&DocumentString::from("changed"))
    );
    for child in children {
        assert_eq!(now.node(child).unwrap().parent(), Some(root));
        assert_eq!(old.node(child).unwrap().parent(), Some(root));
    }
}

#[test]
fn source_created_identities_survive_published_policy_phase_and_final_construction() {
    let mut dom = owner();
    let associated = document(&mut dom);
    let retired = comment(&mut dom, associated, "vacancy");
    let mut batch = DomBatch::new();
    batch.destroy(retired);
    apply(&mut dom, batch);
    let old = dom.snapshot();
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::create_xml_document(associated.node(), Namespace::html(), "style", None),
            source_limits(10_000_000),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("the inserted Style root requires its actual policy observation");
    };
    let document = request.source().document();
    let style = request.source().element();
    let pending = lease.snapshot();
    assert_eq!(pending.node(document.node()).unwrap().children(), &[style]);
    assert_eq!(
        pending.node(style).unwrap().owner_document(),
        Some(document)
    );
    assert_ne!(document.node(), retired);
    assert_ne!(style, retired);
    let observation = DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        1,
        false,
    );
    lease
        .accept_inline_csp(&request, &observation, &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("a prohibited inline sheet completes without a CSSOM creation");
    };
    assert_eq!(done.output(), Some(DomSourceOutput::Document(document)));
    assert_eq!(done.published_node(), Some(document.node()));
    let final_snapshot = lease.snapshot();
    assert_eq!(
        final_snapshot.node(document.node()).unwrap().children(),
        &[style]
    );
    assert_eq!(
        final_snapshot.node(style).unwrap().parent(),
        Some(document.node())
    );
    assert_eq!(pending.node(style).unwrap().id(), style);
    assert_eq!(old.node(document.node()).unwrap_err(), DomError::StaleNode);
    assert_eq!(old.node(style).unwrap_err(), DomError::StaleNode);
    assert_eq!(
        final_snapshot.node(retired).unwrap_err(),
        DomError::StaleNode
    );
}

#[test]
fn local_source_attribute_edit_completes_with_fixed_work_despite_unrelated_records() {
    // #1258 requires administrative work to depend on the affected records and
    // index depth. This generous finite allowance covers one attribute, one
    // output and a handful of index paths at both extents. No subtree, Range,
    // HTML hook or external service is a semantic participant. The allowance
    // is selected independently of current spent-work output and wall time.
    const WORK: u64 = 20_000;
    for unrelated in [0, 4096] {
        let mut dom = owner();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element(&document, "root");
        let other = batch.create_document();
        for _ in 0..unrelated {
            batch.create_comment(&other, "unrelated");
        }
        let commit = apply(&mut dom, batch);
        let root = commit.created(&root).unwrap().unwrap();
        let old = dom.snapshot();
        let cancel = AtomicBool::new(false);
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_attribute(root, "data-local", "changed"),
                source_limits(WORK),
                &cancel,
            )
            .unwrap_or_else(|error| {
                panic!("{unrelated} unrelated records: local Source admission failed: {error:?}")
            });
        let progress = lease.advance(&cancel).unwrap_or_else(|error| {
            panic!("{unrelated} unrelated records: local Source execution failed: {error:?}")
        });
        let DomSourceProgress::Complete(done) = progress else {
            panic!("plain XML attribute edit has no external service");
        };
        assert_eq!(
            done.commit().changes().changes(),
            &[DomChange::Attributes(root)]
        );
        let now = lease.snapshot();
        assert_eq!(
            now.node(root)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into()),
            Some(&DocumentString::from("changed"))
        );
        assert_eq!(
            old.node(root)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into()),
            None
        );
        assert_eq!(now.nodes().count(), unrelated + 3);
    }
}
