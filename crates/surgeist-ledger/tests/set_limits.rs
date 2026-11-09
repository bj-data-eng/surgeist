use surgeist_ledger::{Change, Error, Limits, Operation, RefusalReason, Tariff, Tariffs};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

// Four resources and one work dimension make nine words of usage, which the in-place store
// prefers for a small batch.
surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { A, B, C, D }
}

fn caps(visits: u64, a: u64) -> Limits<Work, Resource> {
    Limits::new(
        WorkTable { Visit: visits },
        ResourceTable {
            A: a,
            B: u64::MAX,
            C: u64::MAX,
            D: u64::MAX,
        },
    )
}

fn operation(visits: u64, a: u64) -> Operation<Work, Resource> {
    Operation::new("root", caps(visits, a), 3).unwrap()
}

fn refusal_limit<T>(result: Result<T, Error<Work, Resource>>) -> (u64, u64) {
    let Err(Error::Refused(refusal)) = result else {
        panic!("expected a refusal");
    };
    let RefusalReason::Limit { limit } = refusal.reason else {
        panic!("expected a cap refusal");
    };
    (limit, refusal.current)
}

#[test]
fn new_caps_apply_to_later_batches_and_a_refused_batch_passes_under_a_raised_cap() {
    let mut operation = operation(5, 5);
    let root = operation.root();
    let batch = [
        Change::Acquire(Resource::A, 6),
        Change::Work(Work::Visit, 6),
    ];
    assert_eq!(refusal_limit(operation.apply_batch(root, &batch)), (5, 0));
    operation.set_limits(root, caps(10, 10)).unwrap();
    operation.apply_batch(root, &batch).unwrap();
    let usage = operation.usage(root).unwrap();
    assert_eq!((usage.live(Resource::A), usage.work(Work::Visit)), (6, 6));
    assert_eq!(*operation.limits(root).unwrap(), caps(10, 10));
}

#[test]
fn replacing_caps_changes_no_usage_peak_or_custody() {
    let mut operation = operation(10, 10);
    let root = operation.root();
    operation
        .apply_batch(
            root,
            &[
                Change::Acquire(Resource::A, 8),
                Change::Release(Resource::A, 3),
                Change::Work(Work::Visit, 4),
            ],
        )
        .unwrap();
    let before = *operation.usage(root).unwrap();
    operation.set_limits(root, caps(1, 1)).unwrap();
    assert_eq!(*operation.usage(root).unwrap(), before);
    assert_eq!(before.peak(Resource::A), 8);
    // The direct custody is intact: the 5 held can still be released.
    operation
        .apply(root, Change::Release(Resource::A, 5))
        .unwrap();
}

#[test]
fn a_scope_above_its_new_cap_refuses_the_next_acquisition_even_of_zero_and_accepts_releases() {
    let mut operation = operation(10, 10);
    let root = operation.root();
    operation
        .apply(root, Change::Acquire(Resource::A, 8))
        .unwrap();
    operation.set_limits(root, caps(10, 5)).unwrap();
    assert_eq!(
        refusal_limit(operation.apply(root, Change::Acquire(Resource::A, 0))),
        (5, 8)
    );
    assert_eq!(operation.usage(root).unwrap().live(Resource::A), 8);
    operation
        .apply(root, Change::Release(Resource::A, 4))
        .unwrap();
    operation
        .apply(root, Change::Acquire(Resource::A, 1))
        .unwrap();
    assert_eq!(operation.usage(root).unwrap().live(Resource::A), 5);
}

#[test]
fn a_work_cap_set_to_the_work_so_far_plus_a_call_allowance_refuses_beyond_it() {
    let mut operation = operation(100, 10);
    let root = operation.root();
    operation
        .apply(root, Change::Work(Work::Visit, 30))
        .unwrap();
    let spent = operation.usage(root).unwrap().work(Work::Visit);
    operation.set_limits(root, caps(spent + 4, 10)).unwrap();
    operation.apply(root, Change::Work(Work::Visit, 4)).unwrap();
    assert_eq!(
        refusal_limit(operation.apply(root, Change::Work(Work::Visit, 1))),
        (34, 34)
    );
}

#[test]
fn a_cap_replaced_above_a_child_refuses_the_childs_batch_from_the_changed_scope() {
    let mut operation = operation(10, 10);
    let root = operation.root();
    let child = operation
        .child(root, "child", caps(10, 10), Tariffs::identity())
        .unwrap();
    operation
        .apply(child, Change::Acquire(Resource::A, 4))
        .unwrap();
    operation.set_limits(root, caps(10, 6)).unwrap();
    let refused = operation.apply(child, Change::Acquire(Resource::A, 3));
    let Err(Error::Refused(refusal)) = refused else {
        panic!("expected a refusal");
    };
    assert_eq!(refusal.scope, root);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 6 });
    assert_eq!(refusal.current, 4);
    operation
        .apply(child, Change::Acquire(Resource::A, 2))
        .unwrap();
}

#[test]
fn the_in_place_store_rolls_back_a_batch_refused_under_a_changed_cap() {
    // A wide pair and a small batch go through the in-place store.
    let mut operation = operation(10, 100);
    let root = operation.root();
    operation
        .apply_batch(
            root,
            &[
                Change::Acquire(Resource::A, 10),
                Change::Work(Work::Visit, 2),
            ],
        )
        .unwrap();
    operation.set_limits(root, caps(3, 12)).unwrap();
    let before = *operation.usage(root).unwrap();
    // The acquisition fits the new cap and the visits do not: the whole batch is undone.
    let refused = operation.apply_batch(
        root,
        &[
            Change::Acquire(Resource::A, 2),
            Change::Work(Work::Visit, 2),
        ],
    );
    assert_eq!(refusal_limit(refused), (3, 2));
    assert_eq!(*operation.usage(root).unwrap(), before);
    operation
        .apply_batch(
            root,
            &[
                Change::Acquire(Resource::A, 2),
                Change::Work(Work::Visit, 1),
            ],
        )
        .unwrap();
    assert_eq!(operation.usage(root).unwrap().live(Resource::A), 12);
}

#[test]
fn a_cap_replaced_while_an_interval_is_open_applies_to_its_later_batches() {
    let mut operation = Operation::<Work, Resource>::new("root", caps(10, 10), 1)
        .unwrap()
        .with_checkpoint_limit(1)
        .unwrap();
    let root = operation.root();
    operation
        .apply(root, Change::Acquire(Resource::A, 4))
        .unwrap();
    let interval = operation.begin_checkpoint().unwrap();
    assert_eq!(
        refusal_limit(operation.apply(root, Change::Acquire(Resource::A, 7))),
        (10, 4)
    );
    operation.set_limits(root, caps(10, 20)).unwrap();
    operation
        .apply_batch(
            root,
            &[
                Change::Acquire(Resource::A, 7),
                Change::Release(Resource::A, 2),
                Change::Work(Work::Visit, 3),
            ],
        )
        .unwrap();
    let report = operation.end_checkpoint(&interval).unwrap();
    let usage = &report.scopes[0].usage;
    assert_eq!(usage.work(Work::Visit), 3);
    assert_eq!(usage.opening_live(Resource::A), 4);
    // The interval's peak holds the admitted batch, not the refused one.
    assert_eq!(usage.peak(Resource::A), 11);
    assert_eq!(operation.usage(root).unwrap().live(Resource::A), 9);
}

#[test]
fn a_scope_of_another_operation_is_a_foreign_scope() {
    let mut operation = operation(1, 1);
    let other = Operation::<Work, Resource>::new("other", caps(1, 1), 1).unwrap();
    assert_eq!(
        operation.set_limits(other.root(), caps(2, 2)),
        Err(Error::ForeignScope)
    );
    assert_eq!(*operation.limits(operation.root()).unwrap(), caps(1, 1));
}

#[test]
fn the_shared_variant_replaces_caps_for_every_clone() {
    let shared = operation(5, 5).share();
    let clone = shared.clone();
    let root = shared.root();
    assert_eq!(
        refusal_limit(clone.apply(root, Change::Acquire(Resource::A, 6))),
        (5, 0)
    );
    shared.set_limits(root, caps(5, 9)).unwrap();
    clone.apply(root, Change::Acquire(Resource::A, 6)).unwrap();
    assert_eq!(shared.limits(root).unwrap(), caps(5, 9));
    assert_eq!(shared.usage(root).unwrap().live(Resource::A), 6);
}

#[test]
fn a_tariff_into_a_changed_parent_still_converts_by_its_scale() {
    let mut operation = Operation::<Work, Resource>::new("root", caps(10, 10), 2).unwrap();
    let root = operation.root();
    let child = operation
        .child(
            root,
            "child",
            Limits::unlimited(),
            Tariffs::from_fn(
                |work| Tariff::new(work, 1),
                |resource| Tariff::new(resource, 2),
            ),
        )
        .unwrap();
    assert_eq!(
        refusal_limit(operation.apply(child, Change::Acquire(Resource::A, 6))),
        (10, 0)
    );
    operation.set_limits(root, caps(10, 12)).unwrap();
    operation
        .apply(child, Change::Acquire(Resource::A, 6))
        .unwrap();
    assert_eq!(operation.usage(root).unwrap().live(Resource::A), 12);
}
