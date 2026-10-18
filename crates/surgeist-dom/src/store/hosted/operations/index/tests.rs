//! Private index transitions use genuine issuer identities; these tests make no
//! claim about Window lifecycle completion.
use super::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn fixture() -> (Dom, DomHostedSource, Arc<Work>, [DomHostedOperationId; 3]) {
    let mut dom = Dom::new(DomLimits::new(64, 4096, 64, 0, 0, 16, 4_000_000).unwrap()).unwrap();
    let mut source = dom
        .register_hosted_source(
            DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            &cancel(),
        )
        .unwrap();
    let work = source.work(&cancel()).unwrap();
    let ids = std::array::from_fn(|_| source.begin_operation(&work, 1, 0, &cancel()).unwrap());
    (dom, source, work, ids)
}
fn prepare(
    index: &Index,
    key: KeyWords,
    id: DomHostedOperationId,
    source: &DomHostedSource,
    work: &Work,
) -> Plan {
    index
        .prepare(
            key,
            id.slot,
            &mut |charge| {
                work.step(&cancel())?;
                source.retain(charge.bytes, &cancel()).map(Some)
            },
            None,
            work,
            &cancel(),
        )
        .unwrap()
}
#[test]
fn held_member_survives_sibling_retirement_and_complete_pending_path_survives_last_prefix_purge() {
    let (_dom, source, work, [a, b, c]) = fixture();
    let key = KeyWords::history(a);
    let before = source.retained_storage_bytes();
    let mut index = Index::default();
    for id in [a, b] {
        let plan = prepare(&index, key, id, &source, &work);
        index.install(plan, id);
    }
    assert_eq!(index.first(key), Some(a));
    index.remove(key, a);
    assert_eq!(index.first(key), Some(b));
    index.retire(key, b);
    assert_eq!(index.first(key), None);
    assert!(
        index.inspect_storage_bytes() > 0,
        "held member retains its exact path"
    );
    index.restore(key, a);
    assert_eq!(index.first(key), Some(a));
    assert_eq!(
        source.retained_storage_bytes() - before,
        index.inspect_storage_bytes()
    );
    let pending = prepare(&index, key, c, &source, &work);
    let pending_raw = pending.inspect_storage_bytes();
    index.remove(key, a);
    index.retire(key, a); // Permanently release an already narrowed/inactive path.
    assert_eq!(index.inspect_storage_bytes(), 0);
    assert_eq!(source.retained_storage_bytes() - before, pending_raw);
    let moved = index.install(pending, c);
    assert_eq!(index.first(key), Some(c));
    assert_eq!(moved.bytes, index.inspect_storage_bytes());
    assert_eq!(
        source.retained_storage_bytes() - before,
        index.inspect_storage_bytes()
    );
    index.retire(key, c);
    assert_eq!(index.inspect_storage_bytes(), 0);
    assert_eq!(source.retained_storage_bytes(), before);
}
#[test]
fn shared_prefix_install_releases_complete_unused_spares_without_changing_original_member() {
    let (_dom, source, work, [a, b, _]) = fixture();
    let key = KeyWords::history(a);
    let before = source.retained_storage_bytes();
    let mut index = Index::default();
    let plan = prepare(&index, key, a, &source, &work);
    index.install(plan, a);
    let old = index.inspect_storage_bytes();
    let plan = prepare(&index, key, b, &source, &work);
    let reserved = plan.component_bytes();
    let moved = index.install(plan, b);
    assert!(
        moved.bytes < reserved,
        "existing exact prefixes keep original pages"
    );
    assert_eq!(index.inspect_storage_bytes(), old + moved.bytes);
    assert_eq!(
        source.retained_storage_bytes() - before,
        index.inspect_storage_bytes()
    );
    assert_eq!(index.first(key), Some(a));
    index.retire(key, a);
    assert_eq!(index.first(key), Some(b));
    index.retire(key, b);
    assert_eq!(source.retained_storage_bytes(), before);
}

#[test]
fn reservation_work_refusal_drops_temporary_backing_and_preserves_original_lookup_for_retry() {
    let (_dom, mut source, work, [a, b, _]) = fixture();
    let key = KeyWords::history(a);
    let mut index = Index::default();
    let plan = prepare(&index, key, a, &source, &work);
    index.install(plan, a);
    let raw = index.inspect_storage_bytes();
    let retained = source.retained_storage_bytes();
    let policy = source.limits();
    let spent = work.spent.load(Ordering::Relaxed);
    source
        .set_limits(
            DomHostedLimits::new(
                policy.records(),
                policy.retained_bytes(),
                policy.scratch_bytes(),
                spent + 2,
            ),
            &cancel(),
        )
        .unwrap();
    let result = index.prepare(
        key,
        b.slot,
        &mut |charge| {
            work.step(&cancel())?;
            source.retain(charge.bytes, &cancel()).map(Some)
        },
        None,
        &work,
        &cancel(),
    );
    assert!(matches!(result, Err(DomHostedError::WorkLimit { .. })));
    assert_eq!(index.first(key), Some(a));
    assert_eq!(index.inspect_storage_bytes(), raw);
    assert_eq!(source.retained_storage_bytes(), retained);
    assert!(work.spent.load(Ordering::Relaxed) > spent);
    source.set_limits(policy, &cancel()).unwrap();
    let plan = prepare(&index, key, b, &source, &work);
    index.install(plan, b);
    assert_eq!(index.first(key), Some(a));
    index.retire(key, a);
    assert_eq!(index.first(key), Some(b));
}
