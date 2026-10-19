//! Fixed Range endpoints do not rediscover unrelated ordinary siblings.
use super::*;

const WORK: u64 = 256;

fn fixture(siblings: usize) -> (Dom, DomRangeId, NodeId, NodeId) {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(
        siblings + 8,
        65_536,
        2 * siblings + 32,
        0,
        0,
        16,
        64_000_000,
        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(1)
    .unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element(&document, "range-parent");
    batch.insert_before(&parent, &document, None);
    for _ in 0..siblings {
        let comment = batch.create_comment(&document, "unrelated");
        batch.insert_before(&comment, &parent, None);
    }
    let a = batch.create_text(&document, "a");
    let b = batch.create_text(&document, "b");
    batch.insert_before(&a, &parent, None);
    batch.insert_before(&b, &parent, None);
    let range = batch.create_range(&document);
    batch.set_range_start(&range, &a, 0);
    batch.set_range_end(&range, &b, 1);
    let commit = dom.apply(batch, &cancel).unwrap();
    (
        dom,
        commit.created_range(&range).unwrap().unwrap(),
        commit.created(&a).unwrap().unwrap(),
        commit.created(&b).unwrap().unwrap(),
    )
}

fn policy() -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(500_000_000, 10_000_000, 10_000_000, 8, 128, 128, 1)
}

fn source(dom: &Dom, cancel: &AtomicBool) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(dom, batch.owner, &batch.commands, policy(), cancel)
        .unwrap()
        .0
}

#[test]
fn unchanged_range_boundary_has_fixed_work_with_unrelated_sibling_prefix() {
    for siblings in [0, 8, 128, 512] {
        let cancel = AtomicBool::new(false);
        let (dom, id, a, b) = fixture(siblings);
        let snapshot = dom.snapshot();
        let mut draft = source(&dom, &cancel);
        let a = draft.resolve(a.into()).unwrap();
        let before = draft.range_record(id).unwrap();
        let spent = draft.source_work.load(Ordering::Relaxed);
        let limit = spent.checked_add(WORK).unwrap();
        draft.source_limits.as_mut().unwrap().work = limit;
        let result = draft.set_range_boundary(id, a, 0, true, &cancel);
        let used = draft.source_work.load(Ordering::Relaxed) - spent;
        eprintln!("Range no-op siblings={siblings} used={used} remaining={WORK} result={result:?}");
        assert_eq!(result, Ok(()), "siblings={siblings} used={used}");
        assert!(used <= WORK);
        assert_eq!(draft.range_record(id).unwrap(), before);
        assert_eq!(draft.ranges.touched().count(), 0);
        assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
        assert_eq!(
            snapshot.range(id).unwrap().start().container(),
            match a {
                Key::Existing(id) => id,
                Key::Created(_) => unreachable!("public setup creates canonical endpoints"),
            }
        );
        assert_eq!(snapshot.range(id).unwrap().end().container(), b);
    }
}

#[test]
fn changed_range_publication_has_fixed_work_with_unrelated_sibling_prefix() {
    for siblings in [0, 8, 128, 512] {
        let cancel = AtomicBool::new(false);
        let (dom, id, a, b) = fixture(siblings);
        let snapshot = dom.snapshot();
        let mut draft = source(&dom, &cancel);
        let a = draft.resolve(a.into()).unwrap();
        draft.set_range_boundary(id, a, 1, true, &cancel).unwrap();
        assert_eq!(draft.ranges.touched().count(), 1);
        let before = draft.range_record(id).unwrap();
        let spent = draft.source_work.load(Ordering::Relaxed);
        let limit = spent.checked_add(WORK).unwrap();
        draft.source_limits.as_mut().unwrap().work = limit;
        let result = draft.range_publication_ready(&cancel);
        let used = draft.source_work.load(Ordering::Relaxed) - spent;
        eprintln!(
            "Range publication siblings={siblings} used={used} remaining={WORK} result={result:?}"
        );
        assert_eq!(result, Ok(()), "siblings={siblings} used={used}");
        assert!(used <= WORK);
        assert_eq!(draft.range_record(id).unwrap(), before);
        draft.source_limits = Some(policy());
        assert_eq!(draft.full_range_publication_ready(&cancel), Ok(()));
        assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
        assert_eq!(snapshot.range(id).unwrap().start().offset(), 0);
        assert_eq!(snapshot.range(id).unwrap().end().container(), b);
        assert_eq!(before.start.offset, 1);
        assert_eq!(before.end.offset, 1);
    }
}
