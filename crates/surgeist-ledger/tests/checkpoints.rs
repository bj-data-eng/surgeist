use surgeist_ledger::{
    Change, Error, EventFilter, Limits, Operation, ProfileMode, Resource, Tariff, Tariffs, Work,
};
const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<2> = Resource::new(0).unwrap();
const OTHER: Resource<2> = Resource::new(1).unwrap();
fn operation() -> Operation<1, 2> {
    Operation::with_profile(
        "root",
        Limits::new([10], [100, 100]),
        4,
        ProfileMode::Detailed {
            timing: false,
            event_limit: 2,
            filter: EventFilter::All,
        },
    )
    .unwrap()
    .with_checkpoint_limit(2)
    .unwrap()
}

#[test]
fn interval_peak_includes_temporary_custody_but_excludes_earlier_lifetime_peak() {
    let mut op = operation();
    let root = op.root();
    op.apply_batch(
        root,
        &[Change::Acquire(BYTES, 80), Change::Release(BYTES, 40)],
    )
    .unwrap();
    let first = op.begin_checkpoint().unwrap();
    op.apply_batch(
        root,
        &[
            Change::Acquire(BYTES, 9),
            Change::Release(BYTES, 9),
            Change::Work(WORK, 3),
        ],
    )
    .unwrap();
    let second = op.begin_checkpoint().unwrap();
    op.apply_batch(
        root,
        &[Change::Release(BYTES, 10), Change::Acquire(BYTES, 4)],
    )
    .unwrap();
    let first_report = op.end_checkpoint(&first).unwrap();
    let u = first_report.scopes[0].usage;
    assert_eq!(
        (
            u.opening_live(BYTES),
            u.live(BYTES),
            u.peak(BYTES),
            u.work(WORK)
        ),
        (40, 34, 49, 3)
    );
    let profile = first_report.profile.unwrap();
    assert_eq!(
        (
            profile.scopes[0].usage.work(WORK),
            profile.scopes[0].usage.live(BYTES),
            profile.scopes[0].usage.peak(BYTES)
        ),
        (3, 34, 49)
    );
    assert_eq!((profile.events.len(), profile.dropped_events), (2, 3));
    let second_report = op.end_checkpoint(&second).unwrap();
    let u = second_report.scopes[0].usage;
    assert_eq!(
        (
            u.opening_live(BYTES),
            u.live(BYTES),
            u.peak(BYTES),
            u.work(WORK)
        ),
        (40, 34, 40, 0)
    );
    assert_eq!(op.usage(root).unwrap().peak(BYTES), 80);
}

#[test]
fn refused_prefix_cannot_raise_any_interval_peak_and_cleanup_remains_accepted() {
    let mut op = operation();
    let root = op.root();
    op.apply(root, Change::Acquire(BYTES, 40)).unwrap();
    let a = op.begin_checkpoint().unwrap();
    let b = op.begin_checkpoint().unwrap();
    assert!(matches!(
        op.apply_batch(
            root,
            &[
                Change::Acquire(BYTES, 9),
                Change::Release(BYTES, 9),
                Change::Work(WORK, 11)
            ]
        ),
        Err(Error::Refused(_))
    ));
    op.apply(root, Change::Release(BYTES, 40)).unwrap();
    for checkpoint in [&a, &b] {
        let report = op.end_checkpoint(checkpoint).unwrap();
        let u = report.scopes[0].usage;
        assert_eq!(
            (
                u.opening_live(BYTES),
                u.live(BYTES),
                u.peak(BYTES),
                u.work(WORK)
            ),
            (40, 0, 40, 0)
        );
        let p = report.profile.unwrap();
        assert_eq!(p.scopes[0].refused_batches, 1);
        assert_eq!(p.scopes[0].first_refusal.unwrap().item, 2);
        assert_eq!(p.scopes[0].accepted_batches, 1);
    }
}

#[test]
fn child_creation_remapping_and_sibling_balances_define_ancestor_peak() {
    let mut op = operation();
    let root = op.root();
    let sibling = op
        .child(root, "sibling", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    op.apply(sibling, Change::Acquire(OTHER, 30)).unwrap();
    let checkpoint = op.begin_checkpoint().unwrap();
    let child = op
        .child(
            root,
            "child",
            Limits::unlimited(),
            Tariffs::new(
                [Tariff::new(WORK, 2)],
                [Tariff::new(OTHER, 2), Tariff::new(BYTES, 0)],
            ),
        )
        .unwrap();
    op.apply_batch(
        child,
        &[
            Change::Acquire(BYTES, 9),
            Change::Release(BYTES, 9),
            Change::Acquire(OTHER, 20),
            Change::Work(WORK, 3),
        ],
    )
    .unwrap();
    let r = op.end_checkpoint(&checkpoint).unwrap();
    assert_eq!(
        (
            r.scopes[0].usage.opening_live(OTHER),
            r.scopes[0].usage.live(OTHER),
            r.scopes[0].usage.peak(OTHER)
        ),
        (30, 30, 48)
    );
    assert_eq!(
        (
            r.scopes[0].usage.live(BYTES),
            r.scopes[0].usage.peak(BYTES),
            r.scopes[0].usage.work(WORK)
        ),
        (0, 0, 6)
    );
    assert_eq!(
        (
            r.scopes[2].usage.opening_live(BYTES),
            r.scopes[2].usage.peak(BYTES),
            r.scopes[2].usage.live(OTHER)
        ),
        (0, 9, 20)
    );
    assert_eq!(r.scopes[2].parent, Some(root));
}

#[test]
fn tariff_overflow_rolls_back_interval_maximum_and_direct_custody() {
    let mut op = operation();
    let root = op.root();
    let child = op
        .child(
            root,
            "overflow",
            Limits::unlimited(),
            Tariffs::new(
                [Tariff::new(WORK, 1)],
                [Tariff::new(BYTES, u64::MAX), Tariff::new(OTHER, 1)],
            ),
        )
        .unwrap();
    let checkpoint = op.begin_checkpoint().unwrap();
    assert!(matches!(
        op.apply(child, Change::Acquire(BYTES, 2)),
        Err(Error::Refused(_))
    ));
    let r = op.end_checkpoint(&checkpoint).unwrap();
    assert!(r.scopes.iter().all(|s| s.usage.peak(BYTES) == 0));
    assert!(matches!(
        op.apply(child, Change::Release(BYTES, 1)),
        Err(Error::Refused(_))
    ));
}

#[test]
fn identities_bounds_and_reused_slots_preserve_other_live_intervals() {
    let mut op = operation();
    let mut foreign = operation();
    let a = op.begin_checkpoint().unwrap();
    let b = op.begin_checkpoint().unwrap();
    assert_eq!(
        op.begin_checkpoint(),
        Err(Error::CheckpointLimit { limit: 2 })
    );
    assert_eq!(foreign.end_checkpoint(&a), Err(Error::ForeignCheckpoint));
    op.end_checkpoint(&a).unwrap();
    let c = op.begin_checkpoint().unwrap();
    assert_eq!(op.end_checkpoint(&a), Err(Error::StaleCheckpoint));
    assert_eq!(
        op.end_checkpoint(&b).unwrap().scopes[0].usage.peak(BYTES),
        0
    );
    op.end_checkpoint(&c).unwrap();
    let mut disabled = Operation::<0, 0>::new("zero", Limits::unlimited(), 1)
        .unwrap()
        .with_checkpoint_limit(0)
        .unwrap();
    assert_eq!(
        disabled.begin_checkpoint(),
        Err(Error::CheckpointLimit { limit: 0 })
    );
    assert!(matches!(
        disabled.with_checkpoint_limit(1),
        Err(Error::CheckpointsConfigured)
    ));
}

#[test]
fn each_interval_retains_its_own_filtered_detail_after_lifetime_buffer_fills() {
    let mut op = Operation::<1, 2>::with_profile(
        "root",
        Limits::unlimited(),
        2,
        ProfileMode::Detailed {
            timing: false,
            event_limit: 1,
            filter: EventFilter::ScopeName("child"),
        },
    )
    .unwrap()
    .with_checkpoint_limit(1)
    .unwrap();
    let root = op.root();
    let child = op
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    op.apply(child, Change::Work(WORK, 1)).unwrap();
    let checkpoint = op.begin_checkpoint().unwrap();
    op.apply(root, Change::Work(WORK, 1)).unwrap();
    op.apply_batch(child, &[Change::Work(WORK, 1), Change::Work(WORK, 1)])
        .unwrap();
    let r = op.end_checkpoint(&checkpoint).unwrap();
    let p = r.profile.unwrap();
    assert_eq!((p.events.len(), p.dropped_events), (1, 1));
    assert_eq!(
        (p.scopes[0].usage.work(WORK), p.scopes[1].usage.work(WORK)),
        (3, 2)
    );
    assert_eq!(op.snapshot().unwrap().unwrap().dropped_events, 2);
}

#[test]
fn profile_off_intervals_still_expose_exact_balances() {
    let mut op = Operation::<1, 2>::new("off", Limits::unlimited(), 1)
        .unwrap()
        .with_checkpoint_limit(1)
        .unwrap();
    let root = op.root();
    op.apply(root, Change::Acquire(BYTES, 8)).unwrap();
    let checkpoint = op.begin_checkpoint().unwrap();
    op.apply_batch(
        root,
        &[Change::Acquire(BYTES, 3), Change::Release(BYTES, 8)],
    )
    .unwrap();
    let r = op.end_checkpoint(&checkpoint).unwrap();
    assert!(r.profile.is_none());
    assert_eq!(
        (
            r.scopes[0].usage.opening_live(BYTES),
            r.scopes[0].usage.live(BYTES),
            r.scopes[0].usage.peak(BYTES)
        ),
        (8, 3, 11)
    );
}
