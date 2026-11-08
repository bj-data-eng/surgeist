use std::{
    cell::{Cell, RefCell},
    future::{Future, pending},
    panic::{AssertUnwindSafe, catch_unwind},
    task::{Context, Waker},
    time::Duration,
};

use surgeist_ledger::{
    Change, Error, Event, EventFilter, Limits, Operation, PhaseObservation, ProfileMode, Resource,
    Scope, Tariffs, Work,
};

const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<1> = Resource::new(0).unwrap();

thread_local! {
    static NOW: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static READS: Cell<usize> = const { Cell::new(0) };
}

fn clock() -> Duration {
    READS.set(READS.get() + 1);
    NOW.get()
}

fn reset() {
    NOW.set(Duration::ZERO);
    READS.set(0);
}

fn time(milliseconds: u64) {
    NOW.set(Duration::from_millis(milliseconds));
}

fn ms(milliseconds: u64) -> Option<Duration> {
    Some(Duration::from_millis(milliseconds))
}

fn operation(mode: ProfileMode) -> Operation<1, 1> {
    Operation::with_profile_clock("root", Limits::new([3], [8]), 3, mode, clock).unwrap()
}

// A consumer owns this completion policy; the observation itself has no drop hook.
struct EndOnDrop<'a> {
    owner: &'a RefCell<Operation<1, 1>>,
    observation: Option<PhaseObservation>,
}

impl<'a> EndOnDrop<'a> {
    fn begin(owner: &'a RefCell<Operation<1, 1>>, scope: Scope) -> Self {
        let observation = owner.borrow().begin_phase(scope).unwrap();
        Self {
            owner,
            observation: Some(observation),
        }
    }
}

impl Drop for EndOnDrop<'_> {
    fn drop(&mut self) {
        if let Some(observation) = self.observation.take() {
            self.owner.borrow_mut().end_phase(observation).unwrap();
        }
    }
}

#[test]
fn consumer_guard_ends_reentrant_phase_on_unwind_and_preserves_cleanup_custody() {
    reset();
    let owner = RefCell::new(operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    }));
    let root = owner.borrow().root();
    time(10);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _phase = EndOnDrop::begin(&owner, root);
        time(15);
        owner
            .borrow_mut()
            .apply_batch(root, &[Change::Work(WORK, 2), Change::Acquire(BYTES, 5)])
            .unwrap();
        time(25);
        panic!("consumer phase unwind");
    }))
    .unwrap_err();
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"consumer phase unwind"));
    assert_eq!(owner.borrow().usage(root).unwrap().live(BYTES), 5);
    time(30);
    owner
        .borrow_mut()
        .apply(root, Change::Release(BYTES, 5))
        .unwrap();
    time(40);
    let report = owner.into_inner().finish().unwrap();
    let scope = &report.scopes[0];
    assert_eq!((scope.phases, scope.phase_elapsed), (1, ms(15)));
    assert_eq!((scope.accepted_batches, scope.accepted_items), (2, 3));
    assert_eq!(
        (
            scope.usage.work(WORK),
            scope.usage.live(BYTES),
            scope.usage.peak(BYTES)
        ),
        (2, 0, 5)
    );
    assert_eq!(
        report.events[2],
        Event::Phase {
            scope: root,
            duration: ms(15),
            elapsed: ms(25)
        }
    );
}

#[test]
fn consumer_guard_ends_polled_future_on_drop_but_unpolled_future_has_no_observation() {
    reset();
    let owner = RefCell::new(operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    }));
    let root = owner.borrow().root();
    time(5);
    let reads = READS.get();
    let unpolled = async {
        let _phase = EndOnDrop::begin(&owner, root);
        owner
            .borrow_mut()
            .apply(root, Change::Work(WORK, 1))
            .unwrap();
        pending::<()>().await;
    };
    time(8);
    drop(unpolled);
    assert_eq!(READS.get(), reads);
    let before_poll = owner.borrow_mut().snapshot().unwrap().unwrap();
    assert_eq!(before_poll.scopes[0].phases, 0);
    assert_eq!(before_poll.scopes[0].usage.work(WORK), 0);
    assert!(before_poll.events.is_empty());

    let mut future = Box::pin(async {
        let _phase = EndOnDrop::begin(&owner, root);
        time(15);
        owner
            .borrow_mut()
            .apply_batch(root, &[Change::Work(WORK, 2), Change::Acquire(BYTES, 5)])
            .unwrap();
        pending::<()>().await;
    });
    time(10);
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    time(25);
    drop(future);
    assert_eq!(owner.borrow().usage(root).unwrap().live(BYTES), 5);
    time(30);
    owner
        .borrow_mut()
        .apply(root, Change::Release(BYTES, 5))
        .unwrap();
    time(40);
    let report = owner.into_inner().finish().unwrap();
    let scope = &report.scopes[0];
    assert_eq!((scope.phases, scope.phase_elapsed), (1, ms(15)));
    assert_eq!((scope.accepted_batches, scope.accepted_items), (2, 3));
    assert_eq!(
        (
            scope.usage.work(WORK),
            scope.usage.live(BYTES),
            scope.usage.peak(BYTES)
        ),
        (2, 0, 5)
    );
    assert_eq!(
        report.events[2],
        Event::Phase {
            scope: root,
            duration: ms(15),
            elapsed: ms(25)
        }
    );
}

#[test]
fn explicit_end_preserves_accepted_prefix_refusal_and_cleanup_in_every_mode() {
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 16,
            filter: EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 16,
            filter: EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 2,
            filter: EventFilter::All,
        },
    ] {
        reset();
        let timed = matches!(
            mode,
            ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
        );
        let mut owner = operation(mode);
        let root = owner.root();
        time(10);
        let before_begin = READS.get();
        let observation = owner.begin_phase(root).unwrap();
        assert_eq!(READS.get() - before_begin, usize::from(timed));
        time(12);
        owner
            .apply_batch(root, &[Change::Work(WORK, 2), Change::Acquire(BYTES, 5)])
            .unwrap();
        time(13);
        let Err(Error::Refused(refusal)) =
            owner.apply_batch(root, &[Change::Acquire(BYTES, 1), Change::Work(WORK, 2)])
        else {
            panic!("the second batch must exceed the work limit");
        };
        assert_eq!(refusal.item, 1);
        assert_eq!(owner.usage(root).unwrap().live(BYTES), 5);
        time(14);
        owner.apply(root, Change::Release(BYTES, 5)).unwrap();
        time(20);
        let before_end = READS.get();
        owner.end_phase(observation).unwrap();
        assert_eq!(READS.get() - before_end, usize::from(timed));
        let usage = owner.usage(root).unwrap();
        assert_eq!(
            (usage.work(WORK), usage.live(BYTES), usage.peak(BYTES)),
            (2, 0, 5)
        );
        time(25);
        let report = owner.finish();
        if matches!(mode, ProfileMode::Off) {
            assert!(report.is_none());
        } else {
            let report = report.unwrap();
            let scope = &report.scopes[0];
            assert_eq!(
                (
                    scope.accepted_batches,
                    scope.accepted_items,
                    scope.refused_batches,
                    scope.phases
                ),
                (2, 3, 1, 1)
            );
            assert_eq!(scope.first_refusal, Some(refusal));
            assert_eq!(
                scope.phase_elapsed,
                timed.then_some(Duration::from_millis(10))
            );
            assert_eq!(report.elapsed, timed.then_some(Duration::from_millis(25)));
            if let ProfileMode::Detailed { event_limit, .. } = mode {
                assert_eq!(report.events.len(), event_limit.min(5));
                assert_eq!(
                    report.dropped_events,
                    5_u64.saturating_sub(event_limit as u64)
                );
                if event_limit >= 5 {
                    assert!(
                        matches!(report.events.last(), Some(Event::Phase { scope, duration, elapsed }) if *scope == root && *duration == timed.then_some(Duration::from_millis(10)) && *elapsed == timed.then_some(Duration::from_millis(20)))
                    );
                }
            } else {
                assert!(report.events.is_empty());
                assert_eq!(report.dropped_events, 0);
            }
        }
        if !timed {
            assert_eq!(READS.get(), 0);
        }
    }
}

#[test]
fn foreign_plain_begin_and_end_validate_before_clock_and_retain_the_original_marker() {
    reset();
    let mut owner = operation(ProfileMode::Summary { timing: true });
    let mut foreign = Operation::<0, 0>::with_profile_clock(
        "foreign",
        Limits::unlimited(),
        1,
        ProfileMode::Summary { timing: true },
        clock,
    )
    .unwrap()
    .with_checkpoint_limit(1)
    .unwrap();
    let foreign_interval = foreign.begin_checkpoint().unwrap();
    let before = READS.get();
    assert!(matches!(
        owner.begin_phase(foreign.root()),
        Err(Error::ForeignScope)
    ));
    assert_eq!(READS.get(), before);
    time(10);
    let observation = owner.begin_phase(owner.root()).unwrap();
    time(20);
    let before = READS.get();
    let observation = foreign.end_phase(observation).unwrap_err();
    assert_eq!(READS.get(), before);
    assert_eq!(foreign.snapshot().unwrap().unwrap().scopes[0].phases, 0);
    let foreign_interval = foreign.end_checkpoint(&foreign_interval).unwrap();
    assert_eq!(foreign_interval.profile.unwrap().scopes[0].phases, 0);
    time(40);
    owner.end_phase(observation).unwrap();
    let report = owner.finish().unwrap();
    assert_eq!(report.scopes[0].phases, 1);
    assert_eq!(report.scopes[0].phase_elapsed, ms(30));
    assert_eq!(report.scopes[0].usage.work(WORK), 0);
}

#[test]
fn sharing_and_cloned_participants_preserve_foreign_retry_and_completion_access() {
    reset();
    let owner = operation(ProfileMode::Summary { timing: true });
    let root = owner.root();
    time(10);
    let before_share = owner.begin_phase(root).unwrap();
    let shared = owner.share();
    let participant = shared.clone();
    let foreign = Operation::<2, 0>::with_profile_clock(
        "foreign",
        Limits::unlimited(),
        1,
        ProfileMode::Summary { timing: true },
        clock,
    )
    .unwrap()
    .share();
    time(15);
    let before = READS.get();
    assert!(matches!(
        foreign.begin_phase(root),
        Err(Error::ForeignScope)
    ));
    let before_share = foreign.end_phase(before_share).unwrap_err();
    assert_eq!(READS.get(), before);
    assert_eq!(foreign.snapshot().unwrap().unwrap().scopes[0].phases, 0);
    participant.apply(root, Change::Work(WORK, 1)).unwrap();
    time(30);
    participant.end_phase(before_share).unwrap();
    time(40);
    let from_participant = participant.begin_phase(root).unwrap();
    let mut plain_foreign = operation(ProfileMode::Summary { timing: true });
    time(45);
    let before = READS.get();
    let from_participant = plain_foreign.end_phase(from_participant).unwrap_err();
    assert_eq!(READS.get(), before);
    assert_eq!(
        plain_foreign.snapshot().unwrap().unwrap().scopes[0].phases,
        0
    );
    time(50);
    shared.end_phase(from_participant).unwrap();
    let report = shared.snapshot().unwrap().unwrap();
    assert_eq!(report.scopes[0].phases, 2);
    assert_eq!(report.scopes[0].phase_elapsed, ms(30));
    assert_eq!(report.scopes[0].usage.work(WORK), 1);
}

#[test]
fn detached_and_automatic_phases_share_completion_and_checkpoint_observers() {
    reset();
    let mut owner = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    })
    .with_checkpoint_limit(1)
    .unwrap();
    let root = owner.root();
    time(5);
    let detached = owner.begin_phase(root).unwrap();
    time(10);
    let checkpoint = owner.begin_checkpoint().unwrap();
    time(15);
    owner
        .with_phase(root, |phase| {
            phase.apply(root, Change::Work(WORK, 1)).unwrap();
            time(20);
        })
        .unwrap();
    let shared = owner.share();
    let participant = shared.clone();
    time(25);
    shared
        .with_phase(root, || {
            participant.apply(root, Change::Work(WORK, 1)).unwrap();
            time(30);
        })
        .unwrap();
    time(35);
    participant.end_phase(detached).unwrap();
    time(40);
    let interval = shared.end_checkpoint(&checkpoint).unwrap();
    let profile = interval.profile.unwrap();
    assert_eq!(interval.scopes[0].usage.work(WORK), 2);
    assert_eq!(
        (profile.scopes[0].phases, profile.scopes[0].phase_elapsed),
        (3, ms(35))
    );
    let phases = profile
        .events
        .into_iter()
        .filter(|event| matches!(event, Event::Phase { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        phases,
        [
            Event::Phase {
                scope: root,
                duration: ms(5),
                elapsed: ms(10)
            },
            Event::Phase {
                scope: root,
                duration: ms(5),
                elapsed: ms(20)
            },
            Event::Phase {
                scope: root,
                duration: ms(25),
                elapsed: ms(25)
            },
        ]
    );
    let lifetime = shared.snapshot().unwrap().unwrap();
    assert_eq!(
        (lifetime.scopes[0].phases, lifetime.scopes[0].phase_elapsed),
        (3, ms(40))
    );
}

#[test]
fn overlapping_observations_complete_out_of_order_with_inclusive_durations() {
    reset();
    let mut owner = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    });
    let root = owner.root();
    let child = owner
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    time(10);
    let first = owner.begin_phase(root).unwrap();
    time(20);
    let second = owner.begin_phase(root).unwrap();
    time(30);
    let nested = owner.begin_phase(child).unwrap();
    owner.apply(child, Change::Work(WORK, 3)).unwrap();
    time(40);
    owner.end_phase(first).unwrap();
    time(45);
    owner.end_phase(nested).unwrap();
    time(60);
    owner.end_phase(second).unwrap();
    time(75);
    let report = owner.finish().unwrap();
    assert_eq!(report.elapsed, ms(75));
    assert_eq!(
        (report.scopes[0].phases, report.scopes[0].phase_elapsed),
        (2, ms(70))
    );
    assert_eq!(
        (report.scopes[1].phases, report.scopes[1].phase_elapsed),
        (1, ms(15))
    );
    assert_eq!(report.scopes[0].usage.work(WORK), 3);
    assert_eq!(
        report.events[1..],
        [
            Event::Phase {
                scope: root,
                duration: ms(30),
                elapsed: ms(40)
            },
            Event::Phase {
                scope: child,
                duration: ms(15),
                elapsed: ms(45)
            },
            Event::Phase {
                scope: root,
                duration: ms(40),
                elapsed: ms(60)
            },
        ]
    );
}

#[test]
fn discard_and_finishing_the_owner_never_manufacture_phase_completions() {
    reset();
    let mut owner = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    });
    let root = owner.root();
    let completed;
    let reads;
    {
        time(10);
        let _discarded = owner.begin_phase(root).unwrap();
        time(20);
        completed = owner.begin_phase(root).unwrap();
        time(30);
        reads = READS.get();
    }
    assert_eq!(READS.get(), reads);
    time(35);
    owner.end_phase(completed).unwrap();
    let report;
    let reads;
    {
        time(40);
        let _unfinished = owner.begin_phase(root).unwrap();
        time(50);
        report = owner.finish().unwrap();
        reads = READS.get();
    }
    assert_eq!(READS.get(), reads);
    assert_eq!(report.scopes[0].phases, 1);
    assert_eq!(report.scopes[0].phase_elapsed, ms(15));
    assert_eq!(
        report.events,
        [Event::Phase {
            scope: root,
            duration: ms(15),
            elapsed: ms(35)
        }]
    );

    let reads;
    {
        let abandoned_owner = operation(ProfileMode::Summary { timing: true });
        let _unfinished = abandoned_owner.begin_phase(abandoned_owner.root()).unwrap();
        reads = READS.get();
        drop(abandoned_owner);
    }
    assert_eq!(READS.get(), reads);
}

#[test]
fn checkpoint_profiles_clip_only_observations_explicitly_ended_inside_them() {
    reset();
    let mut owner = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 16,
        filter: EventFilter::All,
    })
    .with_checkpoint_limit(2)
    .unwrap();
    let root = owner.root();
    time(5);
    let early = owner.begin_phase(root).unwrap();
    time(10);
    let first = owner.begin_checkpoint().unwrap();
    time(12);
    owner.apply(root, Change::Work(WORK, 2)).unwrap();
    time(15);
    let later = owner.begin_phase(root).unwrap();
    time(20);
    let second = owner.begin_checkpoint().unwrap();
    time(22);
    owner.apply(root, Change::Work(WORK, 1)).unwrap();
    time(30);
    owner.end_phase(early).unwrap();
    time(35);
    let first_report = owner.end_checkpoint(&first).unwrap();
    let first_profile = first_report.profile.unwrap();
    assert_eq!(first_report.scopes[0].usage.work(WORK), 3);
    assert_eq!(
        (
            first_profile.scopes[0].phases,
            first_profile.scopes[0].phase_elapsed
        ),
        (1, ms(20))
    );
    assert!(
        matches!(first_profile.events.last(), Some(Event::Phase { scope, duration, elapsed }) if *scope == root && *duration == ms(20) && *elapsed == ms(20))
    );
    time(40);
    owner.end_phase(later).unwrap();
    time(45);
    let second_report = owner.end_checkpoint(&second).unwrap();
    let second_profile = second_report.profile.unwrap();
    assert_eq!(second_report.scopes[0].usage.work(WORK), 1);
    assert_eq!(
        (
            second_profile.scopes[0].phases,
            second_profile.scopes[0].phase_elapsed
        ),
        (2, ms(30))
    );
    assert_eq!(
        second_profile.events[1..],
        [
            Event::Phase {
                scope: root,
                duration: ms(10),
                elapsed: ms(10)
            },
            Event::Phase {
                scope: root,
                duration: ms(20),
                elapsed: ms(20)
            },
        ]
    );
    assert_eq!(first_profile.scopes[0].phases, 1);
    let lifetime = owner.finish().unwrap();
    assert_eq!(
        (lifetime.scopes[0].phases, lifetime.scopes[0].phase_elapsed),
        (2, ms(50))
    );
}

#[test]
fn static_scope_filter_and_detail_cap_preserve_all_phase_summaries() {
    reset();
    let mut owner = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 2,
        filter: EventFilter::ScopeName("selected"),
    });
    let root = owner.root();
    let selected = owner
        .child(root, "selected", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    let other = owner
        .child(root, "other", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    time(1);
    let hidden = owner.begin_phase(root).unwrap();
    time(2);
    owner.end_phase(hidden).unwrap();
    time(3);
    let hidden = owner.begin_phase(other).unwrap();
    time(4);
    owner.end_phase(hidden).unwrap();
    time(5);
    let retained = owner.begin_phase(selected).unwrap();
    time(8);
    owner.end_phase(retained).unwrap();
    time(9);
    owner.apply(selected, Change::Work(WORK, 1)).unwrap();
    time(10);
    let clipped = owner.begin_phase(selected).unwrap();
    time(12);
    owner.end_phase(clipped).unwrap();
    time(13);
    owner.apply(other, Change::Work(WORK, 2)).unwrap();
    let report = owner.finish().unwrap();
    assert_eq!(
        report.events,
        [
            Event::Phase {
                scope: selected,
                duration: ms(3),
                elapsed: ms(8)
            },
            Event::Accepted {
                scope: selected,
                change: Change::Work(WORK, 1),
                elapsed: ms(9)
            },
        ]
    );
    assert_eq!(report.dropped_events, 1);
    assert_eq!(
        report
            .scopes
            .iter()
            .map(|scope| scope.phases)
            .collect::<Vec<_>>(),
        [1, 2, 1]
    );
    assert_eq!(report.scopes[1].phase_elapsed, ms(5));
    assert_eq!(report.scopes[0].usage.work(WORK), 3);
}

#[test]
fn overlapping_maximum_intervals_saturate_observation_without_changing_usage() {
    reset();
    let mut owner = operation(ProfileMode::Summary { timing: true });
    let root = owner.root();
    let first = owner.begin_phase(root).unwrap();
    let second = owner.begin_phase(root).unwrap();
    NOW.set(Duration::MAX);
    owner.end_phase(first).unwrap();
    owner.end_phase(second).unwrap();
    let report = owner.finish().unwrap();
    assert_eq!(report.scopes[0].phases, 2);
    assert_eq!(report.scopes[0].phase_elapsed, Some(Duration::MAX));
    assert_eq!(report.elapsed, Some(Duration::MAX));
    assert!(report.saturated);
    assert_eq!(report.scopes[0].usage.work(WORK), 0);
    assert_eq!(report.scopes[0].usage.live(BYTES), 0);
}
