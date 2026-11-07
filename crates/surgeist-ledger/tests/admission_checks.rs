use surgeist_ledger::{
    Change, Error, EventFilter, Limits, Operation, ProfileMode, RefusalReason, Resource, Tariff,
    Tariffs, Work,
};
const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<1> = Resource::new(0).unwrap();

#[test]
fn checking_limits_custody_and_success_changes_no_usage_profile_or_interval_peak() {
    let mut op = Operation::with_profile(
        "root",
        Limits::new([10], [20]),
        1,
        ProfileMode::Detailed {
            timing: false,
            event_limit: 10,
            filter: EventFilter::All,
        },
    )
    .unwrap()
    .with_checkpoint_limit(1)
    .unwrap();
    let root = op.root();
    op.apply(root, Change::Acquire(BYTES, 4)).unwrap();
    let checkpoint = op.begin_checkpoint().unwrap();
    let before = op.snapshot().unwrap();
    let usage = *op.usage(root).unwrap();
    op.check_batch(
        root,
        &[
            Change::Acquire(BYTES, 10),
            Change::Release(BYTES, 10),
            Change::Work(WORK, 10),
        ],
    )
    .unwrap();
    assert!(
        matches!(op.check_batch(root,&[Change::Acquire(BYTES,17)]),Err(Error::Refused(r)) if matches!(r.reason,RefusalReason::Limit{limit:20}))
    );
    assert!(
        matches!(op.check_batch(root,&[Change::Release(BYTES,5)]),Err(Error::Refused(r)) if r.reason==RefusalReason::Underflow)
    );
    assert_eq!(*op.usage(root).unwrap(), usage);
    assert_eq!(op.snapshot().unwrap(), before);
    let report = op.end_checkpoint(&checkpoint).unwrap();
    let u = report.scopes[0].usage;
    assert_eq!((u.work(WORK), u.live(BYTES), u.peak(BYTES)), (0, 4, 4));
    let p = report.profile.unwrap();
    assert_eq!(
        (
            p.scopes[0].accepted_batches,
            p.scopes[0].refused_batches,
            p.events.len()
        ),
        (0, 0, 0)
    );
}

#[test]
fn checking_uses_parent_tariffs_and_origin_custody_without_reserving() {
    let mut op = Operation::new("root", Limits::new([10], [20]), 3).unwrap();
    let root = op.root();
    let child = op
        .child(
            root,
            "child",
            Limits::unlimited(),
            Tariffs::new([Tariff::new(WORK, 2)], [Tariff::new(BYTES, 3)]),
        )
        .unwrap();
    let sibling = op
        .child(root, "sibling", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    op.apply(sibling, Change::Acquire(BYTES, 4)).unwrap();
    assert!(
        matches!(op.check_batch(child,&[Change::Work(WORK,6)]),Err(Error::Refused(r)) if r.scope==root&&r.current==0)
    );
    assert!(
        matches!(op.check_batch(root,&[Change::Release(BYTES,1)]),Err(Error::Refused(r)) if r.reason==RefusalReason::Underflow)
    );
    op.check_batch(child, &[Change::Acquire(BYTES, 5)]).unwrap();
    let shared = op.share();
    let participant = shared.clone();
    participant
        .apply(sibling, Change::Acquire(BYTES, 2))
        .unwrap();
    assert!(
        matches!(shared.apply(child,Change::Acquire(BYTES,5)),Err(Error::Refused(r)) if r.scope==root)
    );
    assert_eq!(shared.usage(child).unwrap().live(BYTES), 0);
}

#[test]
fn arithmetic_and_tariff_overflows_are_checked_without_observation() {
    let mut op = Operation::new("root", Limits::unlimited(), 2).unwrap();
    let root = op.root();
    let child = op
        .child(
            root,
            "child",
            Limits::unlimited(),
            Tariffs::new([Tariff::new(WORK, 2)], [Tariff::new(BYTES, u64::MAX)]),
        )
        .unwrap();
    op.apply(root, Change::Work(WORK, u64::MAX)).unwrap();
    assert!(
        matches!(op.check_batch(root,&[Change::Work(WORK,1)]),Err(Error::Refused(r)) if r.reason==RefusalReason::Overflow)
    );
    assert!(
        matches!(op.check_batch(child,&[Change::Acquire(BYTES,2)]),Err(Error::Refused(r)) if r.scope==child&&r.reason==RefusalReason::Overflow)
    );
    let other = Operation::<1, 1>::new("other", Limits::unlimited(), 1).unwrap();
    assert_eq!(op.check_batch(other.root(), &[]), Err(Error::ForeignScope));
    op.check_batch(root, &[]).unwrap();
    assert_eq!(op.usage(root).unwrap().work(WORK), u64::MAX);
}
