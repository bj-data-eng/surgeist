use surgeist_ledger::{
    Change, Error, Limits, Operation, RefusalReason, Scope, ScopeLedger, Tariffs,
};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes }
}

fn caps(bytes: u64) -> Limits<Work, Resource> {
    Limits::new(
        WorkTable { Visit: u64::MAX },
        ResourceTable { Bytes: bytes },
    )
}

/// Opens a child below `parent`, checks a batch against it, admits it, raises the cap and admits
/// what the old cap refused, then returns the child's live bytes. Runs the same on every ledger.
fn exercise(mut ledger: impl ScopeLedger<Work, Resource>, parent: Scope) -> u64 {
    let child = ledger
        .child(parent, "child", caps(10), Tariffs::identity())
        .unwrap();
    let batch = [Change::Acquire(Resource::Bytes, 8)];
    ledger.check_batch(child, &batch).unwrap();
    assert_eq!(ledger.usage(child).unwrap().live(Resource::Bytes), 0);
    ledger.apply_batch(child, &batch).unwrap();
    let refused = ledger.apply_batch(child, &batch).unwrap_err();
    assert!(matches!(
        refused,
        Error::Refused(refusal) if refusal.reason == RefusalReason::Limit { limit: 10 }
    ));
    ledger.set_limits(child, caps(20)).unwrap();
    ledger.apply_batch(child, &batch).unwrap();
    ledger.usage(child).unwrap().live(Resource::Bytes)
}

#[test]
fn an_owned_operation_a_shared_one_and_references_to_both_behave_alike() {
    let mut owned = Operation::<Work, Resource>::new("root", caps(100), 2).unwrap();
    let root = owned.root();
    assert_eq!(exercise(&mut owned, root), 16);
    assert_eq!(owned.usage(root).unwrap().live(Resource::Bytes), 16);

    let owned = Operation::<Work, Resource>::new("root", caps(100), 2).unwrap();
    let root = owned.root();
    assert_eq!(exercise(owned, root), 16);

    let shared = Operation::<Work, Resource>::new("root", caps(100), 2)
        .unwrap()
        .share();
    let root = shared.root();
    assert_eq!(exercise(shared.clone(), root), 16);
    assert_eq!(shared.usage(root).unwrap().live(Resource::Bytes), 16);
}

#[test]
fn a_scope_of_another_operation_is_a_foreign_scope_through_the_trait() {
    let mut ledger = Operation::<Work, Resource>::new("root", caps(1), 2).unwrap();
    let other = Operation::<Work, Resource>::new("other", caps(1), 1).unwrap();
    assert_eq!(
        ScopeLedger::child(&mut ledger, other.root(), "x", caps(1), Tariffs::identity()),
        Err(Error::ForeignScope)
    );
    assert_eq!(
        ScopeLedger::usage(&ledger, other.root()),
        Err(Error::ForeignScope)
    );
}
