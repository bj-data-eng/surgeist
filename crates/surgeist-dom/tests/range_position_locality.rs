//! Fixed endpoints and ordinary depth exclude unrelated sibling discovery.
use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomChange, DomLimits, DomRangeId, DomSnapshot, DomSourceEdit, DomSourceLimits,
    DomSourceProgress, NodeId,
};

const SIBLINGS: usize = 4096;
const WORK: u64 = 4096;

fn bounds(snapshot: &DomSnapshot, id: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let range = snapshot.range(id).unwrap();
    (
        (range.start().container(), range.start().offset()),
        (range.end().container(), range.end().offset()),
    )
}

#[test]
fn source_range_boundary_changes_and_noops_fit_fixed_work_beside_unrelated_siblings() {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(8192, 1_000_000, 16_384, 0, 0, 16, 64_000_000)
        .unwrap()
        .with_ranges(1)
        .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let parent = setup.create_element(&document, "range-parent");
    setup.insert_before(&parent, &document, None);
    for _ in 0..SIBLINGS {
        let sibling = setup.create_comment(&document, "unrelated");
        setup.insert_before(&sibling, &parent, None);
    }
    let a = setup.create_text(&document, "a");
    let b = setup.create_text(&document, "b");
    setup.insert_before(&a, &parent, None);
    setup.insert_before(&b, &parent, None);
    let range = setup.create_range(&document);
    setup.set_range_start(&range, &a, 0);
    setup.set_range_end(&range, &b, 1);
    setup.set_active_range(&document, Some((&range).into()));
    let commit = dom.apply(setup, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let parent = commit.created(&parent).unwrap().unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    let original = dom.snapshot();
    let children = original.node(parent).unwrap().children().to_vec();
    assert_eq!(children.len(), SIBLINGS + 2);
    assert_eq!(&children[SIBLINGS..], &[a, b]);
    assert_eq!(bounds(&original, range), ((a, 0), (b, 1)));

    let mut noop = DomBatch::new();
    noop.set_range_start(range, a, 0);
    let commit = dom.apply(noop, &cancel).unwrap();
    assert!(commit.changes().changes().is_empty());
    assert_eq!(dom.snapshot().stamp(), original.stamp());
    let mut change = DomBatch::new();
    change.set_range_start(range, a, 1);
    let commit = dom.apply(change, &cancel).unwrap();
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::RangeBoundary(range)]
    );
    let changed = dom.snapshot();
    assert_eq!(bounds(&changed, range), ((a, 1), (b, 1)));
    assert_eq!(changed.node(parent).unwrap().children(), children);
    assert_eq!(bounds(&original, range), ((a, 0), (b, 1)));

    for (offset, expected_change) in [(1, false), (0, true)] {
        let before = dom.snapshot();
        let policy = DomSourceLimits::new(500_000_000, 10_000_000, WORK, 8, 128, 128, 1);
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::set_range_start(range, a, offset),
                policy,
                &cancel,
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("intrinsic Range boundary update needs no external service");
        };
        assert_eq!(done.output(), None);
        if expected_change {
            assert_eq!(
                done.commit().changes().changes(),
                &[DomChange::RangeBoundary(range)]
            );
        } else {
            assert!(done.commit().changes().changes().is_empty());
        }
        drop(lease);
        let after = dom.snapshot();
        assert_eq!(bounds(&after, range), ((a, offset), (b, 1)));
        assert_eq!(after.node(parent).unwrap().children(), children);
        assert_eq!(
            after.document(document).unwrap().active_range(),
            original.document(document).unwrap().active_range()
        );
        if expected_change {
            assert!(after.revision() > before.revision());
            assert_eq!(
                dom.changes_since(before.stamp()).unwrap().changes(),
                &[DomChange::RangeBoundary(range)]
            );
        } else {
            assert_eq!(after.stamp(), before.stamp());
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
        assert_eq!(bounds(&original, range), ((a, 0), (b, 1)));
        assert_eq!(bounds(&changed, range), ((a, 1), (b, 1)));
    }
    assert!(dom.pending_source_edit().is_none());
}
