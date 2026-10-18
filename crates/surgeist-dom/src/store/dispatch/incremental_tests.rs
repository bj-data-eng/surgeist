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

#[test]
fn foreign_target_validation_precedes_registration_backing_admission() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut foreign = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = foreign.apply(batch, &cancel).unwrap();
    let target = commit.document(&document).unwrap().node();
    dom.set_limits(
        limits(2).with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 128, 0, 4_000_000, 2).unwrap(),
        ),
    )
    .unwrap();
    assert_eq!(
        dom.register_dispatch_custody(&[target], &cancel)
            .unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(dom.dispatch_custody_retained_bytes(&cancel), Ok(0));
}

#[test]
fn current_generation_retirement_fits_budget_despite_retained_historical_generations() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let mut historical = Vec::new();
    let mut previous: Option<NodeId> = None;
    for _ in 0..32 {
        let mut batch = DomBatch::new();
        let target = batch.create_comment(document.node(), "retained generation");
        let commit = dom.apply(batch, &cancel).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        if let Some(previous) = previous {
            assert_eq!(target.slot, previous.slot, "exercise the same reused slot");
        }
        previous = Some(target);
        let scope = dom.register_dispatch_custody(&[target], &cancel).unwrap();
        let mut batch = DomBatch::new();
        batch.destroy(target);
        dom.apply(batch, &cancel).unwrap();
        let association = dom.dispatch_custody_association(&scope, 0, target).unwrap();
        assert!(matches!(association, DomDispatchAssociation::Retired(_)));
        historical.push((target, scope, association));
    }
    let mut batch = DomBatch::new();
    let current = batch.create_comment(document.node(), "current generation");
    let commit = dom.apply(batch, &cancel).unwrap();
    let current = commit.created(&current).unwrap().unwrap();
    assert_eq!(current.slot, previous.unwrap().slot);
    let scope = dom.register_dispatch_custody(&[current], &cancel).unwrap();

    // Even the original aggregate limit check fits this budget. Retirement
    // itself has one participating row; old generations keep their own cells.
    dom.set_limits(limits(66)).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(current);
    dom.apply(batch, &cancel).unwrap();
    let association = dom
        .dispatch_custody_association(&scope, 0, current)
        .unwrap();
    assert!(matches!(association, DomDispatchAssociation::Retired(_)));
    assert_eq!(association.document(), document);
    for (target, scope, original) in historical {
        assert_eq!(
            dom.dispatch_custody_association(&scope, 0, target),
            Ok(original)
        );
    }
}

#[test]
fn many_distinct_rows_for_one_retirement_fit_nonquadratic_work_budget() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits(100_000)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_comment(&document, "many invocation rows");
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let scope = dom
        .register_dispatch_custody(&[target; 64], &cancel)
        .unwrap();
    dom.set_limits(limits(512)).unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(target);
    dom.apply(batch, &cancel).unwrap();
    let first = dom.dispatch_custody_association(&scope, 0, target).unwrap();
    assert!(matches!(first, DomDispatchAssociation::Retired(_)));
    assert_eq!(first.document(), document);
    for row in 1..64 {
        assert_eq!(
            dom.dispatch_custody_association(&scope, row, target),
            Ok(first)
        );
    }
}
