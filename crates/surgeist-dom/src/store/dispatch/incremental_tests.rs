use super::*;

fn limits(work: u64) -> DomLimits {
    DomLimits::new(128, 4096, 1024, 0, 0, 0, 0)
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 128, 4_000_000, 4_000_000, work).unwrap(),
        )
}

#[test]
fn unchanged_custody_extent_admits_a_fixed_budget_with_many_live_scopes() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "target");
    let commit = dom.apply(batch, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scopes: Vec<_> = (0..64)
        .map(|_| dom.register_dispatch_custody(&[target], &cancel).unwrap())
        .collect();
    let retained = dom.dispatch_custody_retained_bytes(&cancel).unwrap();

    // No scope or row changed: an aggregate read and an unchanged limit check
    // have fixed work, independent of these sixty-four live registrations.
    assert_eq!(dom.set_limits(limits(4)), Ok(()));
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(retained));
    assert_eq!(scopes.len(), 64);
}

#[test]
fn scope_drop_debits_rows_before_any_prune_but_keeps_indexed_headers() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "target");
    let commit = dom.apply(batch, &cancel).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scope = dom
        .register_dispatch_custody(&[target, target, target], &cancel)
        .unwrap();
    let before = dom.dispatch_custody_retained_bytes(&cancel).unwrap();
    drop(scope);

    // Destruction releases the exact boxed rows. Reading the aggregate does
    // not silently prune the retained Weak/header or its cleanup index.
    assert_eq!(
        dom.dispatch_custody_retained_bytes(&cancel),
        Ok(before - 3 * size_of::<Row>())
    );
}
