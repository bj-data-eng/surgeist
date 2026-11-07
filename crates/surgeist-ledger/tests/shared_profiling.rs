use std::{
    cell::Cell,
    future::{Future, poll_fn},
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};
use surgeist_ledger::{
    Change, Error, Limits, Operation, ProfileMode, Resource, SharedOperation, Work,
};
const WORK: Work<1> = Work::new(0).unwrap();
const BYTES: Resource<1> = Resource::new(0).unwrap();
thread_local! { static TIME:Cell<u64>=const {Cell::new(0)}; static READS:Cell<usize>=const{Cell::new(0)}; }
fn clock() -> Duration {
    READS.set(READS.get() + 1);
    Duration::from_millis(TIME.get())
}
fn poll<F: Future>(f: Pin<&mut F>) -> Poll<F::Output> {
    f.poll(&mut Context::from_waker(Waker::noop()))
}
fn operation(mode: ProfileMode) -> SharedOperation<1, 1> {
    TIME.set(0);
    READS.set(0);
    Operation::with_profile_clock("root", Limits::new([10], [20]), 2, mode, clock)
        .unwrap()
        .with_checkpoint_limit(2)
        .unwrap()
        .share()
}

#[test]
fn pending_phase_permits_other_participant_and_completion_measures_actual_await() {
    let op = operation(ProfileMode::Summary { timing: true });
    let participant = op.clone();
    let root = op.root();
    let entered = Cell::new(false);
    let ready = Cell::new(false);
    let checkpoint = op.begin_checkpoint().unwrap();
    let mut future = Box::pin(op.with_async_phase(root, async || {
        entered.set(true);
        participant.apply(root, Change::Work(WORK, 2)).unwrap();
        poll_fn(|_| {
            if ready.get() {
                Poll::Ready(7)
            } else {
                Poll::Pending
            }
        })
        .await
    }));
    assert!(!entered.get());
    TIME.set(10);
    assert!(poll(future.as_mut()).is_pending());
    participant.apply(root, Change::Acquire(BYTES, 4)).unwrap();
    assert_eq!(participant.usage(root).unwrap().live(BYTES), 4);
    TIME.set(35);
    ready.set(true);
    assert_eq!(poll(future.as_mut()), Poll::Ready(Ok(7)));
    drop(future);
    let r = op.end_checkpoint(&checkpoint).unwrap();
    let p = r.profile.unwrap();
    assert_eq!(p.scopes[0].phase_elapsed, Some(Duration::from_millis(25)));
    assert_eq!(p.scopes[0].phases, 1);
    assert_eq!(r.scopes[0].usage.work(WORK), 2);
}

#[test]
fn cancellation_retains_prefix_refusal_phase_and_cleanup_report() {
    let op = operation(ProfileMode::Summary { timing: true });
    let root = op.root();
    let checkpoint = op.begin_checkpoint().unwrap();
    let mut future = Box::pin(op.with_async_phase(root, async || {
        op.apply_batch(root, &[Change::Work(WORK, 3), Change::Acquire(BYTES, 8)])
            .unwrap();
        assert!(matches!(
            op.apply(root, Change::Work(WORK, 8)),
            Err(Error::Refused(_))
        ));
        std::future::pending::<()>().await;
    }));
    TIME.set(5);
    assert!(poll(future.as_mut()).is_pending());
    TIME.set(20);
    drop(future);
    op.apply(root, Change::Release(BYTES, 8)).unwrap();
    let r = op.end_checkpoint(&checkpoint).unwrap();
    let p = r.profile.unwrap();
    assert_eq!(
        (
            r.scopes[0].usage.work(WORK),
            r.scopes[0].usage.live(BYTES),
            r.scopes[0].usage.peak(BYTES)
        ),
        (3, 0, 8)
    );
    assert_eq!(p.scopes[0].phase_elapsed, Some(Duration::from_millis(15)));
    assert_eq!(p.scopes[0].refused_batches, 1);
}

#[test]
fn overlap_nested_calls_and_checkpoint_clipping_are_independent() {
    let op = operation(ProfileMode::Summary { timing: true });
    let root = op.root();
    let mut a = Box::pin(op.with_async_phase(root, async || std::future::pending::<()>().await));
    TIME.set(5);
    assert!(poll(a.as_mut()).is_pending());
    TIME.set(10);
    let checkpoint = op.begin_checkpoint().unwrap();
    let mut b = Box::pin(op.with_async_phase(root, async || std::future::pending::<()>().await));
    TIME.set(15);
    assert!(poll(b.as_mut()).is_pending());
    TIME.set(30);
    drop(a);
    let r = op.end_checkpoint(&checkpoint).unwrap();
    let p = r.profile.unwrap();
    assert_eq!(p.scopes[0].phases, 1);
    assert_eq!(p.scopes[0].phase_elapsed, Some(Duration::from_millis(20)));
    TIME.set(40);
    drop(b);
    TIME.set(45);
    op.with_phase(root, || {
        TIME.set(50);
        op.with_phase(root, || {
            TIME.set(55);
        })
        .unwrap();
        TIME.set(60);
    })
    .unwrap();
    let p = op.snapshot().unwrap().unwrap();
    assert_eq!(p.scopes[0].phases, 4);
    // A=25, B=25, outer=15, inner=5, inclusive sum=70.
    assert_eq!(p.scopes[0].phase_elapsed, Some(Duration::from_millis(70)));
}

#[test]
fn unwind_and_unpolled_drop_preserve_exactly_once_observation() {
    let op = operation(ProfileMode::Summary { timing: true });
    let root = op.root();
    let future = op.with_async_phase(root, async || panic!("unpolled"));
    drop(future);
    assert_eq!(op.snapshot().unwrap().unwrap().scopes[0].phases, 0);
    let mut future = Box::pin(op.with_async_phase(root, async || {
        op.apply(root, Change::Work(WORK, 1)).unwrap();
        TIME.set(9);
        panic!("phase unwind");
    }));
    TIME.set(4);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| poll(future.as_mut()))).is_err()
    );
    drop(future);
    assert_eq!(op.snapshot().unwrap().unwrap().scopes[0].phases, 1);
    assert_eq!(op.usage(root).unwrap().work(WORK), 1);
    op.with_phase(root, || op.apply(root, Change::Work(WORK, 1)))
        .unwrap()
        .unwrap();
}

#[test]
fn off_and_untimed_shared_phases_and_checkpoints_never_read_clock() {
    for mode in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let op = operation(mode);
        let root = op.root();
        let checkpoint = op.begin_checkpoint().unwrap();
        let mut f = Box::pin(op.with_async_phase(root, async || {
            op.with_phase(root, || op.apply(root, Change::Work(WORK, 2)))
                .unwrap()
                .unwrap();
        }));
        assert_eq!(poll(f.as_mut()), Poll::Ready(Ok(())));
        drop(f);
        assert_eq!(
            op.end_checkpoint(&checkpoint).unwrap().scopes[0]
                .usage
                .work(WORK),
            2
        );
        assert_eq!(READS.get(), 0);
    }
}

#[test]
fn foreign_scope_never_invokes_shared_callback() {
    let op = operation(ProfileMode::Off);
    let foreign = operation(ProfileMode::Off);
    let invoked = Cell::new(false);
    let mut f = Box::pin(op.with_async_phase(foreign.root(), async || {
        invoked.set(true);
    }));
    assert_eq!(poll(f.as_mut()), Poll::Ready(Err(Error::ForeignScope)));
    assert!(!invoked.get());
}
