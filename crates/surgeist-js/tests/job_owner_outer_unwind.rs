use std::{
    cell::Cell,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
};
use surgeist_js::{
    Context, JsValue,
    job::{ContextTask, JobRetirement, JobStatus, JobStep, NativeAsyncJob},
};

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[derive(Default)]
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
#[derive(Debug, Eq, PartialEq)]
struct ExternalUnwind {
    token: u64,
}

#[test]
fn exact_context_destruction_during_an_external_unwind_preserves_normal_owner_retirement() {
    let mut original = Some(Context::builder().build().unwrap());
    let runtime = original.as_ref().unwrap().runtime().clone();
    let peer = Context::builder().runtime(runtime).build().unwrap();
    let required = original.as_ref().unwrap().id();
    assert_ne!(peer.id(), required);
    let factories = Rc::new(Cell::new(0));
    let polls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let made = factories.clone();
    let polled = polls.clone();
    let probe = DropProbe(drops.clone());
    let mut job = NativeAsyncJob::new(move |_| {
        made.set(made.get() + 1);
        Ok(ContextTask::from_poll(move |_, _| {
            let _keep = &probe;
            polled.set(polled.get() + 1);
            Poll::Pending::<surgeist_js::JsResult<JsValue>>
        }))
    });
    let observer = job.observer();
    let alias = observer.clone();
    let mut wait = observer.wait().unwrap();
    let wakes = Arc::new(WakeCount::default());
    let waker = Waker::from(wakes.clone());
    assert!(
        Pin::new(&mut wait)
            .poll(&mut TaskContext::from_waker(&waker))
            .is_pending()
    );

    let external = catch_unwind(AssertUnwindSafe(|| {
        // This exact owner is local to the unwinding frame. The job, its inert
        // observation handles and a same-runtime peer deliberately outlive it.
        let mut owner = original.take().unwrap();
        let host = owner.host_access();
        assert!(matches!(
            job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host),
            JobStep::Prepared
        ));
        assert!(matches!(
            job.run_step(&mut TaskContext::from_waker(Waker::noop()), &host),
            JobStep::Pending
        ));
        assert_eq!(observer.status(), Ok(JobStatus::Pending));
        assert_eq!(observer.fault(), None);
        assert_eq!(factories.get(), 1);
        assert_eq!(polls.get(), 1);
        assert_eq!(drops.get(), 0);
        panic_any(ExternalUnwind { token: 0x2144 });
    }))
    .unwrap_err();

    assert_eq!(
        external.downcast_ref::<ExternalUnwind>(),
        Some(&ExternalUnwind { token: 0x2144 })
    );
    assert!(original.is_none());
    let terminal = JobStatus::Retired(JobRetirement::OwnerDropped);
    assert_eq!(observer.status(), Ok(terminal));
    assert_eq!(
        observer.fault(),
        None,
        "ordinary cleanup entered during someone else's unwind is not a job protocol defect"
    );
    assert_eq!(alias.status(), Ok(terminal));
    assert_eq!(alias.fault(), None);
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    assert_eq!(
        Pin::new(&mut wait).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Ok(terminal))
    );
    assert_eq!(factories.get(), 1);
    assert_eq!(polls.get(), 1);
    assert_eq!(drops.get(), 1);
    assert_ne!(peer.id(), required);
    drop(job);
    drop(peer);
    assert_eq!(observer.status(), Ok(terminal));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        Pin::new(&mut wait).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Ok(terminal))
    );
    assert_eq!(drops.get(), 1);
}
