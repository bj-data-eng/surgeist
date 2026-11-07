use std::{
    cell::Cell,
    future::{Future, poll_fn},
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use surgeist_ledger::{
    Change, Error, Event, EventFilter, Limits, Operation, ProfileMode, Resource, Tariffs, Work,
};

const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<1> = Resource::new(0).unwrap();
thread_local! {
    static TIME: Cell<u64> = const { Cell::new(0) };
    static CLOCK_READS: Cell<usize> = const { Cell::new(0) };
}

fn clock() -> Duration {
    CLOCK_READS.set(CLOCK_READS.get() + 1);
    Duration::from_millis(TIME.get())
}

fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}

fn operation(mode: ProfileMode) -> Operation<1, 1> {
    Operation::with_profile_clock("operation", Limits::new([10], [10]), 2, mode, clock).unwrap()
}

#[test]
fn awaited_phase_starts_at_first_poll_and_includes_pending_time() {
    TIME.set(100);
    let mut operation = operation(ProfileMode::Detailed {
        timing: true,
        event_limit: 8,
        filter: EventFilter::All,
    });
    let root = operation.root();
    let ready = Cell::new(false);
    let entered = Cell::new(false);
    TIME.set(110);
    let mut future = Box::pin(operation.with_async_phase(root, async |phase| {
        entered.set(true);
        phase
            .apply_batch(root, &[Change::Work(WORK, 1), Change::Acquire(BYTES, 4)])
            .unwrap();
        poll_fn(|_| {
            if ready.get() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        phase
            .apply_batch(root, &[Change::Work(WORK, 2), Change::Release(BYTES, 4)])
            .unwrap();
        7
    }));
    assert!(!entered.get());
    TIME.set(120);
    assert!(poll(future.as_mut()).is_pending());
    TIME.set(150);
    ready.set(true);
    assert_eq!(poll(future.as_mut()), Poll::Ready(Ok(7)));
    drop(future);
    let report = operation.snapshot().unwrap().unwrap();
    assert_eq!(report.elapsed, Some(Duration::from_millis(50)));
    assert_eq!(
        report.scopes[0].phase_elapsed,
        Some(Duration::from_millis(30))
    );
    assert_eq!(report.scopes[0].phases, 1);
    assert_eq!(report.scopes[0].usage.work(WORK), 3);
    assert_eq!(report.scopes[0].usage.live(BYTES), 0);
    assert_eq!(report.scopes[0].usage.peak(BYTES), 4);
    assert!(
        matches!(report.events.last(), Some(Event::Phase { duration: Some(value), .. }) if *value == Duration::from_millis(30))
    );
}

#[test]
fn canceling_pending_phase_retains_admitted_usage_and_allows_cleanup() {
    TIME.set(0);
    let mut operation = operation(ProfileMode::Summary { timing: true });
    let root = operation.root();
    let mut future = Box::pin(operation.with_async_phase(root, async |phase| {
        phase
            .apply_batch(root, &[Change::Work(WORK, 1), Change::Acquire(BYTES, 4)])
            .unwrap();
        std::future::pending::<()>().await;
        phase.apply(root, Change::Work(WORK, 9)).unwrap();
    }));
    TIME.set(10);
    assert!(poll(future.as_mut()).is_pending());
    TIME.set(35);
    drop(future);
    let partial = operation.snapshot().unwrap().unwrap();
    assert_eq!(
        partial.scopes[0].phase_elapsed,
        Some(Duration::from_millis(25))
    );
    assert_eq!(partial.scopes[0].phases, 1);
    assert_eq!(partial.scopes[0].usage.work(WORK), 1);
    assert_eq!(partial.scopes[0].usage.live(BYTES), 4);
    operation.apply(root, Change::Release(BYTES, 4)).unwrap();
    let complete = operation.finish().unwrap();
    assert_eq!(complete.scopes[0].usage.live(BYTES), 0);
    assert_eq!(complete.scopes[0].usage.peak(BYTES), 4);
}

#[test]
fn nested_async_phases_attribute_real_inclusive_intervals() {
    TIME.set(0);
    let mut operation = operation(ProfileMode::Summary { timing: true });
    let root = operation.root();
    let child = operation
        .child(root, "child", Limits::unlimited(), Tariffs::identity())
        .unwrap();
    let ready = Cell::new(false);
    let mut future = Box::pin(operation.with_async_phase(root, async |outer| {
        TIME.set(10);
        outer
            .with_async_phase(child, async |inner| {
                inner.apply(child, Change::Work(WORK, 2)).unwrap();
                poll_fn(|_| {
                    if ready.get() {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await;
            })
            .await
            .unwrap();
        TIME.set(40);
    }));
    TIME.set(5);
    assert!(poll(future.as_mut()).is_pending());
    TIME.set(30);
    ready.set(true);
    assert!(poll(future.as_mut()).is_ready());
    drop(future);
    let report = operation.finish().unwrap();
    assert_eq!(
        report.scopes[0].phase_elapsed,
        Some(Duration::from_millis(35))
    );
    assert_eq!(
        report.scopes[1].phase_elapsed,
        Some(Duration::from_millis(20))
    );
    assert_eq!(report.scopes[0].usage.work(WORK), 2);
    assert_eq!(report.scopes[1].usage.work(WORK), 2);
}

#[test]
fn snapshots_leave_persistent_balances_peaks_and_detail_loss_unchanged() {
    let mut operation = operation(ProfileMode::Detailed {
        timing: false,
        event_limit: 1,
        filter: EventFilter::All,
    });
    let root = operation.root();
    operation
        .apply_batch(
            root,
            &[Change::Acquire(BYTES, 8), Change::Release(BYTES, 5)],
        )
        .unwrap();
    let first = operation.snapshot().unwrap().unwrap();
    assert_eq!(first.scopes[0].usage.live(BYTES), 3);
    assert_eq!(first.scopes[0].usage.peak(BYTES), 8);
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.dropped_events, 1);
    operation
        .apply_batch(
            root,
            &[
                Change::Acquire(BYTES, 2),
                Change::Release(BYTES, 1),
                Change::Work(WORK, 1),
            ],
        )
        .unwrap();
    let second = operation.snapshot().unwrap().unwrap();
    assert_eq!(second.scopes[0].usage.live(BYTES), 4);
    assert_eq!(second.scopes[0].usage.peak(BYTES), 8);
    assert_eq!(second.dropped_events, 4);
    assert_eq!(first.scopes[0].usage.live(BYTES), 3);
    assert_eq!(first.dropped_events, 1);
    assert_eq!(second, operation.finish().unwrap());
}

#[test]
fn foreign_scope_is_rejected_without_polling_callback() {
    let original = operation(ProfileMode::Off);
    let mut other = operation(ProfileMode::Off);
    let entered = Cell::new(false);
    let mut future = Box::pin(other.with_async_phase(original.root(), async |_| {
        entered.set(true);
    }));
    assert_eq!(poll(future.as_mut()), Poll::Ready(Err(Error::ForeignScope)));
    assert!(!entered.get());
}

#[test]
fn profiling_off_performs_admissions_without_clock_reads_or_reports() {
    CLOCK_READS.set(0);
    let mut operation = operation(ProfileMode::Off);
    let root = operation.root();
    let mut future = Box::pin(operation.with_async_phase(root, async |phase| {
        phase.apply(root, Change::Work(WORK, 1)).unwrap();
        std::future::ready(()).await;
    }));
    assert_eq!(poll(future.as_mut()), Poll::Ready(Ok(())));
    drop(future);
    assert_eq!(operation.usage(root).unwrap().work(WORK), 1);
    assert!(operation.snapshot().unwrap().is_none());
    assert!(operation.finish().is_none());
    assert_eq!(CLOCK_READS.get(), 0);
}

#[test]
fn dropping_an_unpolled_future_does_not_start_a_phase() {
    let mut operation = operation(ProfileMode::Summary { timing: true });
    let root = operation.root();
    let entered = Cell::new(false);
    let future = operation.with_async_phase(root, async |_| {
        entered.set(true);
    });
    drop(future);
    assert!(!entered.get());
    assert_eq!(operation.snapshot().unwrap().unwrap().scopes[0].phases, 0);
}
