use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
use surgeist_js::{
    Context, JsValue,
    context::time::{Clock, JsInstant},
    job::{JobRetirement, JobStatus, NativeJob, TimeoutJob},
};

struct MaximumClock;
impl Clock for MaximumClock {
    fn now(&self) -> JsInstant {
        JsInstant::new(u64::MAX, 999_999_999)
    }
    fn system_time_millis(&self) -> i64 {
        0
    }
}
struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn initial_deadline_overflow_keeps_the_unaccepted_timer_until_the_caller_disposes_its_return() {
    let mut owner = Context::builder()
        .clock(Rc::new(MaximumClock))
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let probe = DropProbe(drops.clone());
    let callback = NativeJob::with_realm(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        owner.realm().clone(),
    );
    let observer = callback.observer();
    // This ordinary millisecond constructor does not exercise Duration narrowing.
    let timer = TimeoutJob::new(callback, 1);
    let token = timer.cancellation_token().clone();
    {
        // Keep the publication result alive until the caller explicitly disposes it.
        let attempted = catch_unwind(AssertUnwindSafe(|| owner.enqueue_job(timer.into())));
        assert_eq!(calls.get(), 0);
        assert_eq!(
            drops.get(),
            0,
            "an unaccepted timer cannot be disposed by failing deadline arithmetic"
        );
        assert!(
            !token.revoked(),
            "the unaccepted timer's cancellation ownership remains live"
        );
        assert_eq!(observer.status(), Ok(JobStatus::Pending));
        assert_eq!(observer.fault(), None);
        let _returned = attempted.expect(
            "deadline overflow must return through publication rather than a host arithmetic panic",
        );
        assert_eq!(drops.get(), 0);
    }
    // Leaving the scope is the caller's explicit disposal of whatever owning
    // publication return was supplied; no timer dispatch or wall-clock wait runs.
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
    assert!(token.revoked());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
}
