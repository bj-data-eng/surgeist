use surgeist_ledger::{
    Change, Charge, Error, Limits, Operation, RefusalReason, Scope, Tariff, Tariffs,
};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Steps, Credits }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { Bytes, Depth }
}

type Ledger = Operation<Work, Resource>;

/// A root capped at `credits` with a child that converts its steps to credits.
fn scopes(credits: u64, depth: u64) -> (Ledger, Scope, Scope) {
    let mut operation = Operation::new(
        "root",
        Limits::new(
            WorkTable {
                Steps: u64::MAX,
                Credits: credits,
            },
            ResourceTable {
                Bytes: 64,
                Depth: depth,
            },
        ),
        2,
    )
    .unwrap();
    let root = operation.root();
    let child = operation
        .child(
            root,
            "child",
            Limits::unlimited(),
            Tariffs::from_fn(
                |work| match work {
                    Work::Steps | Work::Credits => Tariff::new(Work::Credits, 2),
                },
                |resource| Tariff::new(resource, 1),
            ),
        )
        .unwrap();
    (operation, root, child)
}

/// A bounded operation that only knows the trait.
fn run<C: Charge<Work, Resource>>(charge: &mut C, steps: u64) -> Result<(), C::Stop> {
    charge.charge(&[
        Change::Work(Work::Steps, steps),
        Change::Acquire(Resource::Bytes, 8),
    ])?;
    charge.record_largest(Resource::Depth, steps)
}

#[test]
fn an_owned_operation_charges_a_scope_through_the_trait() {
    let (mut operation, root, child) = scopes(100, 100);
    run(&mut operation.charge_at(child), 3).unwrap();
    let child_usage = *operation.usage(child).unwrap();
    assert_eq!(child_usage.work(Work::Steps), 3);
    assert_eq!(child_usage.live(Resource::Bytes), 8);
    assert_eq!(child_usage.live(Resource::Depth), 3);
    // The child's tariff converts steps into the root's credits at rate two.
    assert_eq!(operation.usage(root).unwrap().work(Work::Credits), 6);
}

#[test]
fn a_shared_operation_charges_a_scope_through_the_trait() {
    let (operation, root, child) = scopes(100, 100);
    let shared = operation.share();
    run(&mut shared.charge_at(child), 4).unwrap();
    assert_eq!(shared.usage(child).unwrap().work(Work::Steps), 4);
    assert_eq!(shared.usage(root).unwrap().work(Work::Credits), 8);
}

#[test]
fn a_refused_batch_changes_nothing_and_reports_the_ledger_error() {
    let (mut operation, root, child) = scopes(5, 100);
    let error = run(&mut operation.charge_at(child), 3).unwrap_err();
    let Error::Refused(refusal) = error else {
        panic!("expected a refusal, got {error:?}");
    };
    assert_eq!(refusal.scope, root);
    assert_eq!(refusal.item, 0);
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 5 });
    let usage = *operation.usage(child).unwrap();
    assert_eq!(usage.work(Work::Steps), 0);
    assert_eq!(usage.live(Resource::Bytes), 0);
}

#[test]
fn record_largest_keeps_the_peak_and_checks_the_new_value_against_the_cap() {
    let (mut operation, root, child) = scopes(100, 10);
    let mut charge = operation.charge_at(child);
    charge.record_largest(Resource::Depth, 7).unwrap();
    // A smaller value replaces the held value; the peak stays at the largest.
    charge.record_largest(Resource::Depth, 2).unwrap();
    let error = charge.record_largest(Resource::Depth, 11).unwrap_err();
    let Error::Refused(refusal) = error else {
        panic!("expected a refusal, got {error:?}");
    };
    // The cap is checked against the new value, in the batch's acquire.
    assert_eq!(refusal.item, 1);
    assert_eq!(refusal.change, Change::Acquire(Resource::Depth, 11));
    assert_eq!(refusal.reason, RefusalReason::Limit { limit: 10 });
    let usage = *operation.usage(child).unwrap();
    // The refused record left the held value and the peak as they were.
    assert_eq!(usage.live(Resource::Depth), 2);
    assert_eq!(usage.peak(Resource::Depth), 7);
    assert_eq!(operation.usage(root).unwrap().live(Resource::Depth), 2);
}

#[test]
fn a_scope_of_another_operation_is_a_foreign_scope_error() {
    let (_, _, foreign) = scopes(100, 100);
    let (mut operation, _, _) = scopes(100, 100);
    let mut charge = operation.charge_at(foreign);
    assert_eq!(
        charge.charge(&[Change::Work(Work::Steps, 1)]),
        Err(Error::ForeignScope)
    );
    assert_eq!(
        charge.record_largest(Resource::Depth, 1),
        Err(Error::ForeignScope)
    );
}

/// Recording one gauge from several chargers of one shared scope never refuses a record whose
/// value fits the cap and leaves the live value equal to a value that was recorded.
#[test]
fn concurrent_gauge_records_on_one_shared_scope_leave_a_recorded_value() {
    let (operation, _, child) = scopes(u64::MAX, 100);
    let shared = operation.share();
    let threads: Vec<_> = (1..=4_u64)
        .map(|id| {
            let shared = shared.clone();
            std::thread::spawn(move || {
                for round in 0..20_000_u64 {
                    // Every value is within the cap of 100.
                    let value = 1 + (id * 7 + round) % 50;
                    shared
                        .charge_at(child)
                        .record_largest(Resource::Depth, value)
                        .unwrap_or_else(|error| panic!("record of {value} refused: {error:?}"));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
    let live = shared.usage(child).unwrap().live(Resource::Depth);
    assert!(
        (1..=50).contains(&live),
        "live {live} is not a recorded value"
    );
}
