use surgeist_ledger::{Change, Error, Limits, Operation, RefusalReason, Scope, Tariff, Tariffs};

surgeist_ledger::dimensions! {
    enum Work => WorkTable { Visit }
}

surgeist_ledger::dimensions! {
    enum Resource => ResourceTable { A, B }
}

fn caps(a: u64, b: u64) -> Limits<Work, Resource> {
    Limits::new(WorkTable { Visit: u64::MAX }, ResourceTable { A: a, B: b })
}

/// Every resource converts to `target` at `scale`.
fn into(target: Resource, scale: u64) -> Tariffs<Work, Resource> {
    Tariffs::from_fn(|work| Tariff::new(work, 1), |_| Tariff::new(target, scale))
}

fn is_refused(result: Result<(), Error<Work, Resource>>) -> bool {
    matches!(result, Err(Error::Refused(_)))
}

/// A leaf of `A` charges a middle scope of `B` at `first`, which charges the root of `B` at
/// `second`.
fn chain(
    root: u64,
    middle: u64,
    leaf: u64,
    first: u64,
    second: u64,
) -> (Operation<Work, Resource>, [Scope; 3]) {
    let mut operation = Operation::new("root", caps(u64::MAX, root), 3).unwrap();
    let top = operation.root();
    let mid = operation
        .child(
            top,
            "middle",
            caps(u64::MAX, middle),
            into(Resource::B, second),
        )
        .unwrap();
    let low = operation
        .child(mid, "leaf", caps(leaf, u64::MAX), into(Resource::B, first))
        .unwrap();
    (operation, [top, mid, low])
}

#[test]
fn headroom_of_a_three_level_chain_is_the_tightest_cap_in_the_units_of_the_origin() {
    // The middle scope holds 10 of B, which is 30 of the root's B at scale 3.
    let (mut operation, [top, mid, low]) = chain(100, 50, 40, 2, 3);
    operation
        .apply(mid, Change::Acquire(Resource::B, 10))
        .unwrap();
    assert_eq!(operation.usage(top).unwrap().live(Resource::B), 30);
    // Leaf: 40 of A. Middle: (50 - 10) / 2 = 20. Root: (100 - 30) / (2 * 3) = 11.
    assert_eq!(operation.headroom(low, Resource::A), Ok(11));
    // 11 of A reaches 22 in the middle and 66 in the root.
    operation
        .check_batch(low, &[Change::Acquire(Resource::A, 11)])
        .unwrap();
    assert!(is_refused(
        operation.check_batch(low, &[Change::Acquire(Resource::A, 12)])
    ));
    // The middle scope reads its own cap and the root's: (50 - 10) and (100 - 30) / 3.
    assert_eq!(operation.headroom(mid, Resource::B), Ok(23));
    assert_eq!(operation.headroom(top, Resource::B), Ok(70));
}

#[test]
fn a_tariff_of_scale_zero_ends_the_walk() {
    // The root holds nothing the leaf can see: the leaf's charges reach the middle scope only.
    let (operation, [_, _, low]) = chain(0, 50, 40, 2, 0);
    assert_eq!(operation.headroom(low, Resource::A), Ok(25));
    let (mut operation, [_, _, low]) = chain(0, 50, 40, 0, 2);
    assert_eq!(operation.headroom(low, Resource::A), Ok(40));
    // Nothing reaches the capped root, so the leaf's whole cap is admitted.
    operation
        .check_batch(low, &[Change::Acquire(Resource::A, 40)])
        .unwrap();
}

#[test]
fn headroom_is_unbounded_where_no_cap_applies_and_rounds_down_through_a_scale() {
    let (operation, [top, _, low]) = chain(u64::MAX, u64::MAX, u64::MAX, 1, 1);
    assert_eq!(operation.headroom(low, Resource::A), Ok(u64::MAX));
    assert_eq!(operation.headroom(top, Resource::B), Ok(u64::MAX));
    let (operation, [_, _, low]) = chain(u64::MAX, u64::MAX, u64::MAX, 2, 3);
    assert_eq!(operation.headroom(low, Resource::A), Ok(u64::MAX / 6));
    // A product of scales beyond `u64::MAX` leaves no room.
    let (operation, [_, _, low]) = chain(u64::MAX, u64::MAX, u64::MAX, u64::MAX, u64::MAX);
    assert_eq!(operation.headroom(low, Resource::A), Ok(0));
}

#[test]
fn a_scope_already_above_its_cap_has_no_headroom() {
    let mut operation = Operation::new("root", caps(10, 10), 1).unwrap();
    let root = operation.root();
    operation
        .apply(root, Change::Acquire(Resource::A, 8))
        .unwrap();
    operation.set_limits(root, caps(5, 10)).unwrap();
    assert_eq!(operation.headroom(root, Resource::A), Ok(0));
    assert_eq!(operation.headroom(root, Resource::B), Ok(10));
}

#[test]
fn a_scope_of_another_operation_is_foreign() {
    let operation = Operation::<Work, Resource>::new("a", Limits::unlimited(), 1).unwrap();
    let other = Operation::<Work, Resource>::new("b", Limits::unlimited(), 1).unwrap();
    assert_eq!(
        operation.headroom(other.root(), Resource::A),
        Err(Error::ForeignScope)
    );
}

#[test]
fn the_shared_operation_reads_the_same_headroom() {
    let (mut operation, [_, mid, low]) = chain(100, 50, 40, 2, 3);
    operation
        .apply(mid, Change::Acquire(Resource::B, 10))
        .unwrap();
    let expected = operation.headroom(low, Resource::A);
    let shared = operation.share();
    assert_eq!(shared.headroom(low, Resource::A), expected);
    assert_eq!(expected, Ok(11));
    let other = Operation::<Work, Resource>::new("other", Limits::unlimited(), 1)
        .unwrap()
        .share();
    assert_eq!(other.headroom(low, Resource::A), Err(Error::ForeignScope));
}

/// The ledger's own admission is the oracle: an acquisition of the headroom is admitted, and one
/// more is refused for a cap or an overflow unless the headroom is the largest amount a `u64`
/// holds.
#[test]
fn the_headroom_is_the_largest_acquisition_the_ledger_admits_over_every_small_chain() {
    let values = [0_u64, 1, 3, 7, 20, u64::MAX];
    let scales = [0_u64, 1, 2, 3, 5];
    let mut checked = 0_u32;
    for root in values {
        for middle in values {
            for leaf in values {
                for first in scales {
                    for second in scales {
                        for held_at_middle in [0_u64, 1, 3] {
                            for held_at_leaf in [0_u64, 1, 2] {
                                let (mut operation, [_, mid, low]) =
                                    chain(root, middle, leaf, first, second);
                                if operation
                                    .apply(mid, Change::Acquire(Resource::B, held_at_middle))
                                    .is_err()
                                    || operation
                                        .apply(low, Change::Acquire(Resource::A, held_at_leaf))
                                        .is_err()
                                {
                                    continue;
                                }
                                for (scope, resource) in [(low, Resource::A), (mid, Resource::B)] {
                                    let headroom = operation.headroom(scope, resource).unwrap();
                                    assert_eq!(
                                        operation.check_batch(
                                            scope,
                                            &[Change::Acquire(resource, headroom)]
                                        ),
                                        Ok(()),
                                        "caps {root} {middle} {leaf} scales {first} {second} \
                                         held {held_at_middle} {held_at_leaf}"
                                    );
                                    if let Some(more) = headroom.checked_add(1) {
                                        let Err(Error::Refused(refusal)) = operation
                                            .check_batch(scope, &[Change::Acquire(resource, more)])
                                        else {
                                            panic!(
                                                "headroom {headroom} is not the largest: caps \
                                                 {root} {middle} {leaf} scales {first} {second} \
                                                 held {held_at_middle} {held_at_leaf}"
                                            );
                                        };
                                        assert!(matches!(
                                            refusal.reason,
                                            RefusalReason::Limit { .. } | RefusalReason::Overflow
                                        ));
                                    }
                                    checked += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(checked > 10_000);
}
