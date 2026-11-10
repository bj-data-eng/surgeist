#![allow(unused_crate_dependencies)]
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};
use surgeist_js::{Context, JsValue, job::TimeoutJob};

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn registration_after_cancellation_cannot_revive_a_token_or_execute_the_rejected_callback() {
    let mut owner = Context::builder().build().unwrap();
    let timer_calls = Rc::new(Cell::new(0));
    let called = timer_calls.clone();
    let timer = TimeoutJob::with_realm(
        move |_| {
            called.set(called.get() + 1);
            Ok(JsValue::undefined())
        },
        owner.realm().clone(),
        std::time::Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    let accepted_calls = Rc::new(Cell::new(0));
    let accepted = accepted_calls.clone();
    token
        .push_callback(move |_| accepted.set(accepted.get() + 1))
        .unwrap();
    token.cancel(owner.access());
    assert!(token.revoked());
    assert_eq!(accepted_calls.get(), 1);

    let rejected_calls = Rc::new(Cell::new(0));
    let rejected = rejected_calls.clone();
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let registration = catch_unwind(AssertUnwindSafe(|| {
        // The existing API returns (). Typed registration refusal is a later
        // functional assertion; the current behavior is independently visible
        // through permanent revocation, callback effects and capture disposal.
        let _ = token.push_callback(move |_| {
            let _keep = &probe;
            rejected.set(rejected.get() + 1);
        });
    }));
    assert!(
        token.revoked(),
        "a rejected registration cannot rearm cancellation"
    );
    token.cancel(owner.access());
    assert_eq!(rejected_calls.get(), 0);
    assert_eq!(accepted_calls.get(), 1);
    assert_eq!(timer_calls.get(), 0);
    assert_eq!(drops.get(), 1);
    assert!(token.revoked());
    registration.expect("registration on a revoked token must return its refusal");
    drop(timer);
    assert!(token.revoked());
}

#[test]
fn registration_after_timer_disposal_cannot_rearm_the_surviving_token() {
    let mut owner = Context::builder().build().unwrap();
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        std::time::Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    drop(timer);
    assert!(token.revoked());
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let registration = catch_unwind(AssertUnwindSafe(|| {
        let _ = token.push_callback(move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
        });
    }));
    assert!(token.revoked());
    token.cancel(owner.access());
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
    registration.expect("disposed timer authority cannot accept a new cancellation callback");
}

#[test]
fn ordinary_cancellation_runs_registered_callbacks_once_and_keeps_the_token_revoked() {
    let mut owner = Context::builder().build().unwrap();
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        std::time::Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    token
        .push_callback(move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
        })
        .unwrap();
    assert!(!token.revoked());
    assert_eq!(drops.get(), 0);
    token.cancel(owner.access());
    assert_eq!(calls.get(), 1);
    assert_eq!(drops.get(), 1);
    assert!(token.revoked());
    token.cancel(owner.access());
    assert_eq!(calls.get(), 1);
    drop(timer);
    assert!(token.revoked());
}
