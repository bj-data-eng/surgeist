//! Public revision-history locality and unchanged chronological semantics.
//! The Source budget is independent of retained history and fixture outcomes.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomChanges, DomHistoryError, DomLimits,
    DomSnapshot, DomSourceEdit, DomSourceLimits, DomSourceProgress, NodeId,
};

const SOURCE_WORK: u64 = 20_000;
const LARGE_HISTORY: usize = 32_768;
const HISTORY_BYTES: usize = 64_000_000;

fn limits(entries: usize, bytes: usize) -> DomLimits {
    DomLimits::new(
        16,
        16_384,
        16,
        0,
        0,
        entries,
        bytes,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    root: NodeId,
    text: NodeId,
    comment: NodeId,
}
fn fixture(entries: usize, bytes: usize) -> Fixture {
    // Initial construction is outside retained revision history. The four
    // genuine live nodes and all string lengths remain fixed as history grows.
    let mut dom = Dom::new(limits(0, 0)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "root");
    let text = batch.create_text(&document, "base");
    let comment = batch.create_comment(&document, "kept");
    batch.set_attribute(&root, "data-local", "before");
    batch.set_attribute(&root, "data-history", "a");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&text, &root, None);
    batch.insert_before(&comment, &root, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    dom.set_limits(limits(entries, bytes)).unwrap();
    assert_eq!(dom.usage().history_entries(), 0);
    assert_eq!(dom.snapshot().nodes().count(), 4);
    Fixture {
        dom,
        document,
        root,
        text,
        comment,
    }
}

fn append_history(case: &mut Fixture, revisions: usize) {
    for index in 0..revisions {
        let mut batch = DomBatch::new();
        batch.set_attribute(
            case.root,
            "data-history",
            if index % 2 == 0 { "b" } else { "a" },
        );
        let commit = case.dom.apply(batch, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            commit.changes().changes(),
            &[DomChange::Attributes(case.root)]
        );
    }
    assert_eq!(case.dom.snapshot().nodes().count(), 4);
}

fn source_attribute(case: &mut Fixture, value: &str, extent: usize) -> DomChanges {
    let policy = DomSourceLimits::new(256_000_000, 1_000_000, SOURCE_WORK, 8, 128, 128, 1);
    let mut lease = case
        .dom
        .begin_source_edit(
            DomSourceEdit::set_attribute(case.root, "data-local", value),
            policy,
            &AtomicBool::new(false),
        )
        .unwrap_or_else(|error| {
            panic!("history extent {extent}, fixed work {SOURCE_WORK}, begin: {error:?}")
        });
    match lease.advance(&AtomicBool::new(false)) {
        Ok(DomSourceProgress::Complete(done)) => {
            assert!(!done.cancellation_requested());
            done.commit().changes().clone()
        }
        other => panic!("history extent {extent}, fixed work {SOURCE_WORK}, advance: {other:?}"),
    }
}

fn assert_neighborhood(case: &Fixture, old: &DomSnapshot) {
    let now = case.dom.snapshot();
    assert_eq!(now.nodes().count(), 4);
    assert_eq!(
        now.node(case.document.node()).unwrap().children(),
        &[case.root]
    );
    assert_eq!(
        now.node(case.root).unwrap().children(),
        &[case.text, case.comment]
    );
    assert_eq!(
        now.node(case.root).unwrap().owner_document(),
        Some(case.document)
    );
    assert_eq!(
        old.node(case.root).unwrap().attribute(&"data-local".into()),
        Some(&"before".into())
    );
    assert_eq!(
        now.node(case.root).unwrap().attribute(&"data-local".into()),
        Some(&"after".into())
    );
    for snapshot in [old, &now] {
        assert_eq!(
            snapshot
                .node(case.root)
                .unwrap()
                .attribute(&"data-history".into()),
            Some(&"a".into())
        );
        assert_eq!(
            snapshot.node(case.text).unwrap().data(),
            Some(&DocumentString::from("base"))
        );
        assert_eq!(
            snapshot.node(case.comment).unwrap().data(),
            Some(&DocumentString::from("kept"))
        );
    }
    assert!(case.dom.pending_source_edit().is_none());
}

#[test]
fn fixed_source_attribute_work_does_not_grow_with_unrelated_retained_revisions() {
    for retained in [0, 64, LARGE_HISTORY] {
        let mut case = fixture(retained + 8, HISTORY_BYTES);
        append_history(&mut case, retained);
        assert_eq!(
            case.dom.usage().history_entries(),
            retained,
            "no eviction during setup"
        );
        let old = case.dom.snapshot();
        let delta = source_attribute(&mut case, "after", retained);
        assert_eq!(delta.from(), old.stamp());
        assert_eq!(delta.to(), case.dom.snapshot().stamp());
        assert_eq!(delta.changes(), &[DomChange::Attributes(case.root)]);
        assert_eq!(case.dom.usage().history_entries(), retained + 1);
        assert_eq!(
            case.dom.changes_since(old.stamp()).unwrap().changes(),
            delta.changes()
        );
        assert_neighborhood(&case, &old);
        let stamp = case.dom.snapshot().stamp();
        let usage = case.dom.usage();
        assert!(
            source_attribute(&mut case, "after", retained)
                .changes()
                .is_empty()
        );
        assert_eq!(case.dom.snapshot().stamp(), stamp);
        assert_eq!(case.dom.usage(), usage);
    }
}

#[test]
fn fixed_source_attribute_work_keeps_a_full_history_window_incremental() {
    for retained in [0, 64, LARGE_HISTORY] {
        let mut case = fixture(retained, HISTORY_BYTES);
        append_history(&mut case, retained + 64);
        assert_eq!(case.dom.usage().history_entries(), retained);
        let old = case.dom.snapshot();
        let bytes = case.dom.usage().history_bytes();
        let delta = source_attribute(&mut case, "after", retained);
        assert_eq!(delta.changes(), &[DomChange::Attributes(case.root)]);
        assert_eq!(case.dom.usage().history_entries(), retained);
        // All retained entries have exactly one DomChange. At steady capacity
        // this append evicts exactly one equally sized original entry.
        assert_eq!(case.dom.usage().history_bytes(), bytes);
        if retained == 0 {
            assert!(
                matches!(case.dom.changes_since(old.stamp()), Err(DomHistoryError::FullRecomputeRequired { available_from }) if available_from == delta.to().revision())
            );
        } else {
            assert_eq!(
                case.dom.changes_since(old.stamp()).unwrap().changes(),
                delta.changes()
            );
        }
        assert_neighborhood(&case, &old);
    }
}

#[test]
fn retained_entry_eviction_keeps_literal_chronology_and_exact_noops() {
    let mut case = fixture(3, HISTORY_BYTES);
    let old = case.dom.snapshot();
    let mut first = DomBatch::new();
    first.set_attribute(case.root, "data-local", "after");
    let first = case.dom.apply(first, &AtomicBool::new(false)).unwrap();
    let mut second = DomBatch::new();
    second.replace_data(case.text, 0, 4, "next");
    case.dom.apply(second, &AtomicBool::new(false)).unwrap();
    let mut third = DomBatch::new();
    third.set_attribute(case.root, "data-local", "before");
    case.dom.apply(third, &AtomicBool::new(false)).unwrap();
    assert_eq!(
        case.dom.changes_since(old.stamp()).unwrap().changes(),
        &[
            DomChange::Attributes(case.root),
            DomChange::Data(case.text),
            DomChange::Attributes(case.root),
        ]
    );
    let stamp = case.dom.snapshot().stamp();
    let usage = case.dom.usage();
    let mut noop = DomBatch::new();
    noop.set_attribute(case.root, "temporary", "x");
    noop.remove_attribute(case.root, "temporary");
    assert!(
        case.dom
            .apply(noop, &AtomicBool::new(false))
            .unwrap()
            .changes()
            .changes()
            .is_empty()
    );
    assert_eq!(case.dom.snapshot().stamp(), stamp);
    assert_eq!(case.dom.usage(), usage);
    let mut fourth = DomBatch::new();
    fourth.replace_data(case.comment, 0, 4, "last");
    case.dom.apply(fourth, &AtomicBool::new(false)).unwrap();
    assert_eq!(case.dom.usage().history_entries(), 3);
    assert_eq!(
        case.dom.changes_since(old.stamp()).unwrap_err(),
        DomHistoryError::FullRecomputeRequired {
            available_from: first.changes().to().revision()
        }
    );
    assert_eq!(
        case.dom
            .changes_since(first.changes().to())
            .unwrap()
            .changes(),
        &[
            DomChange::Data(case.text),
            DomChange::Attributes(case.root),
            DomChange::Data(case.comment),
        ]
    );
    assert_eq!(
        old.node(case.text).unwrap().data(),
        Some(&DocumentString::from("base"))
    );
    assert_eq!(
        old.node(case.comment).unwrap().data(),
        Some(&DocumentString::from("kept"))
    );
}

#[test]
fn a_single_oversized_history_entry_requires_recompute_without_losing_the_commit() {
    let mut case = fixture(1, HISTORY_BYTES);
    append_history(&mut case, 2);
    // Select the bound from this original public owner. This is a history
    // eviction control, not an independent physical-layout/receipt oracle.
    let one_entry_bytes = case.dom.usage().history_bytes();
    assert!(one_entry_bytes > 0);
    case.dom.set_limits(limits(1, one_entry_bytes - 1)).unwrap();
    assert_eq!(case.dom.usage().history_entries(), 0);
    let old = case.dom.snapshot();
    let delta = source_attribute(&mut case, "after", 0);
    assert_eq!(delta.changes(), &[DomChange::Attributes(case.root)]);
    assert_eq!(case.dom.usage().history_entries(), 0);
    assert_eq!(case.dom.usage().history_bytes(), 0);
    assert_eq!(
        case.dom.changes_since(old.stamp()).unwrap_err(),
        DomHistoryError::FullRecomputeRequired {
            available_from: delta.to().revision()
        }
    );
    assert_neighborhood(&case, &old);
}
