use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::atomic::Ordering,
    task::{Context as TaskContext, Poll, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, JsValue,
    context::time::FixedClock,
    execution::ExecutionOptions,
    job::{IntervalJob, JobAcceptance, JobExecutor, JobStatus, SimpleJobExecutor, TimeoutJob},
};

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}

#[test]
fn an_old_interval_reservation_cannot_modify_a_same_deadline_timer_published_after_reentrant_clear()
{
    let executor = Rc::new(SimpleJobExecutor::new());
    let clock = Rc::new(FixedClock::from_millis(100));
    let mut owner = Context::builder()
        .job_executor(executor.clone())
        .clock(clock.clone())
        .build()
        .unwrap();
    let old_calls = Rc::new(Cell::new(0));
    let new_calls = Rc::new(Cell::new(0));
    let old_drops = Rc::new(Cell::new(0));
    let new_drops = Rc::new(Cell::new(0));
    let new_observer = Rc::new(RefCell::new(None));
    let new_token = Rc::new(RefCell::new(None));
    let old_seen = old_calls.clone();
    let new_seen = new_calls.clone();
    let old_probe = DropProbe(old_drops.clone());
    let new_disposal = new_drops.clone();
    let observed = new_observer.clone();
    let cancellation = new_token.clone();
    let nested_executor = executor.clone();
    let interval = IntervalJob::with_realm(
        move |mut access| {
            let _keep = &old_probe;
            old_seen.set(old_seen.get() + 1);
            // The outer interval has already reserved now+10ms. This legitimate
            // nested drain observes the executor's existing cancellation token.
            nested_executor
                .get_cancellation_token()
                .store(true, Ordering::Relaxed);
            access.run_jobs()?;
            let seen = new_seen.clone();
            let probe = DropProbe(new_disposal.clone());
            let timer = TimeoutJob::with_realm(
                move |_| {
                    let _keep = &probe;
                    seen.set(seen.get() + 1);
                    Ok(JsValue::new(42))
                },
                access.realm().clone(),
                Duration::from_millis(10),
            );
            *observed.borrow_mut() = Some(timer.observer());
            *cancellation.borrow_mut() = Some(timer.cancellation_token().clone());
            assert!(matches!(
                access.enqueue_job(timer.into()).unwrap(),
                JobAcceptance::Accepted
            ));
            Ok(JsValue::undefined())
        },
        owner.realm().clone(),
        Duration::from_millis(10),
    );
    let old_token = interval.cancellation_token().clone();
    assert!(matches!(
        owner.enqueue_job(interval.into()).unwrap(),
        JobAcceptance::Accepted
    ));
    // Original deadline=110ms. At now=111ms, both the old rescheduling
    // reservation and the callback's new timeout deadline are exactly121ms.
    clock.forward(11);
    {
        let host = owner.host_access();
        let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
        for _ in 0..16 {
            match poll(&mut driver) {
                Poll::Pending => {}
                Poll::Ready(result) => {
                    panic!("the fresh future timer must still be pending at111ms: {result:?}")
                }
            }
            if old_calls.get() == 1 {
                break;
            }
        }
        assert_eq!(old_calls.get(), 1);
        assert_eq!(new_calls.get(), 0);
        drop(driver);
    }
    assert!(old_token.revoked());
    assert_eq!(old_drops.get(), 1);
    assert_eq!(new_drops.get(), 0);
    let observer = new_observer.borrow().as_ref().unwrap().clone();
    let token = new_token.borrow().as_ref().unwrap().clone();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert!(!token.revoked());
    // Advance outside all engine calls and clock reads: the replacement is now
    // strictly due. A finite snapshot allowance also bounds an accidental idle loop.
    clock.forward(11);
    let completed = owner
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_invocation_limit(1)
                .with_job_claim_work_limit(10_000),
            |mut access| access.run_jobs(),
        )
        .unwrap();
    completed.result.unwrap();
    assert_eq!(completed.report.job_invocations().unwrap(), 1);
    assert_eq!(new_calls.get(), 1);
    assert_eq!(new_drops.get(), 1);
    assert!(token.revoked());
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(old_calls.get(), 1);
}
