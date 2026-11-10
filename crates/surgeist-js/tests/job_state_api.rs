#![allow(unused_crate_dependencies)]
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, HostContext, JsError, JsObject, JsResult, JsValue, NativeFunction,
    error::{
        DiagnosticEngine, DiagnosticKind, EngineError, ErasedEngineError, JobError, PanicError,
    },
    execution::{ExecutionOptions, ExecutionReport},
    job::{CancellationToken, ContextTask, JobCall, TimeoutJob},
};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    builder.build().unwrap()
}
fn poll<T: 'static>(task: &mut ContextTask<T>, host: &HostContext<'_>) -> Poll<JsResult<T>> {
    task.poll(&mut TaskContext::from_waker(Waker::noop()), host)
}
fn error_is(error: &JsError, expected: JobError) {
    assert_eq!(error.as_engine(), Some(&EngineError::Job(expected)));
}
fn completed<T>(result: Poll<JsResult<T>>) -> JsError {
    let Poll::Ready(Err(error)) = result else {
        panic!("terminal observation must return a completed-task error")
    };
    error_is(&error, JobError::TaskCompleted);
    error
}
fn no_job_entry(report: &ExecutionReport) {
    assert_eq!(report.instructions().unwrap(), 0);
    assert_eq!(report.job_preparations().unwrap(), 0);
    assert_eq!(report.job_invocations().unwrap(), 0);
    assert_eq!(report.job_polls().unwrap(), 0);
    assert!(!report.effects_started());
}
fn observe_terminal<T: 'static>(task: &mut ContextTask<T>, owner: &mut Context) -> JsError {
    let outcome = owner
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(0)
                .with_job_invocation_limit(0)
                .with_job_poll_limit(0),
            |access| {
                let host = access.into_host_access();
                let first = completed(poll(task, &host));
                let second = completed(poll(task, &host));
                assert_eq!(second, first);
                Err::<(), _>(first)
            },
        )
        .unwrap();
    no_job_entry(&outcome.report);
    let error = outcome.result.unwrap_err();
    error_is(&error, JobError::TaskCompleted);
    error
}
struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn exact_completed_errors_spend_no_job_admission_and_preserve_first_guest_error_identity() {
    for guest_error in [false, true] {
        let mut owner = context();
        let object = JsObject::with_null_proto(owner.runtime()).unwrap();
        let original = JsError::from_opaque(object.clone().into()).unwrap();
        let supplied = original.clone();
        let calls = Rc::new(Cell::new(0));
        let seen = calls.clone();
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let mut task = ContextTask::from_poll(move |_, _| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Poll::Ready(if guest_error {
                Err(supplied.clone())
            } else {
                Ok(42)
            })
        });
        let first = poll(&mut task, &owner.host_access());
        if guest_error {
            let Poll::Ready(Err(error)) = first else {
                panic!("the original guest error must remain ordinary")
            };
            assert_eq!(error, original);
            assert_eq!(error.as_opaque().unwrap().as_object().unwrap(), object);
        } else {
            assert_eq!(first, Poll::Ready(Ok(42)));
        }
        observe_terminal(&mut task, &mut owner);
        assert_eq!(calls.get(), 1);
        drop(task);
        assert_eq!(drops.get(), 1);
    }
}

#[test]
fn then_keeps_pending_work_and_delivers_one_completion_before_exact_terminal_rejection() {
    let mut owner = context();
    let polls = Rc::new(Cell::new(0));
    let completions = Rc::new(Cell::new(0));
    let seen = polls.clone();
    let converted = completions.clone();
    let mut task = ContextTask::from_poll(move |_, _| {
        seen.set(seen.get() + 1);
        if seen.get() == 1 {
            Poll::Pending
        } else {
            Poll::Ready(Ok(21))
        }
    })
    .then(move |value, _| {
        converted.set(converted.get() + 1);
        Ok(value * 2)
    });
    assert_eq!(poll(&mut task, &owner.host_access()), Poll::Pending);
    assert_eq!(polls.get(), 1);
    assert_eq!(completions.get(), 0);
    assert_eq!(poll(&mut task, &owner.host_access()), Poll::Ready(Ok(42)));
    observe_terminal(&mut task, &mut owner);
    assert_eq!(polls.get(), 2);
    assert_eq!(completions.get(), 1);
}

#[test]
fn then_preserves_an_opaque_guest_error_and_never_enters_its_success_callback() {
    let mut owner = context();
    let object = JsObject::with_null_proto(owner.runtime()).unwrap();
    let original = JsError::from_opaque(object.clone().into()).unwrap();
    let supplied = original.clone();
    let converted = Rc::new(Cell::new(0));
    let seen = converted.clone();
    let mut task = ContextTask::from_future(std::future::ready(Err::<i32, _>(supplied))).then(
        move |value, _| {
            seen.set(seen.get() + 1);
            Ok(value)
        },
    );
    let Poll::Ready(Err(first)) = poll(&mut task, &owner.host_access()) else {
        panic!("authored guest failure")
    };
    assert_eq!(first, original);
    assert_eq!(first.as_opaque().unwrap().as_object().unwrap(), object);
    observe_terminal(&mut task, &mut owner);
    assert_eq!(converted.get(), 0);
}

#[test]
fn source_and_completion_unwind_leave_the_task_terminal_without_replaying_host_work() {
    for panic_in_completion in [false, true] {
        let mut owner = context();
        let calls = Rc::new(Cell::new(0));
        let completions = Rc::new(Cell::new(0));
        let source_calls = calls.clone();
        let completed_calls = completions.clone();
        let source_drops = Rc::new(Cell::new(0));
        let completion_drops = Rc::new(Cell::new(0));
        let source_probe = DropProbe(source_drops.clone());
        let completion_probe = DropProbe(completion_drops.clone());
        let mut task = ContextTask::from_poll(move |_, _| {
            let _keep = &source_probe;
            source_calls.set(source_calls.get() + 1);
            if !panic_in_completion {
                panic!("authored task source unwind");
            }
            Poll::Ready(Ok(42))
        })
        .then(move |_: i32, _| -> JsResult<i32> {
            let _keep = &completion_probe;
            completed_calls.set(completed_calls.get() + 1);
            panic!("authored task completion unwind")
        });
        let host = owner.host_access();
        let unwind = catch_unwind(AssertUnwindSafe(|| poll(&mut task, &host))).unwrap_err();
        let expected = if panic_in_completion {
            "authored task completion unwind"
        } else {
            "authored task source unwind"
        };
        assert_eq!(unwind.downcast_ref::<&str>().copied(), Some(expected));
        drop(host);
        observe_terminal(&mut task, &mut owner);
        assert_eq!(calls.get(), 1);
        assert_eq!(completions.get(), usize::from(panic_in_completion));
        drop(task);
        assert_eq!(source_drops.get(), 1);
        assert_eq!(completion_drops.get(), 1);
    }
}

#[test]
fn every_revocation_route_returns_the_exact_error_and_disposes_rejected_captures_once() {
    for route in 0..3 {
        let mut owner = context();
        let calls = Rc::new(Cell::new(0));
        let seen = calls.clone();
        let timer = TimeoutJob::with_realm(
            move |_| {
                seen.set(seen.get() + 1);
                Ok(JsValue::undefined())
            },
            owner.realm().clone(),
            Duration::ZERO,
        );
        let token = timer.cancellation_token().clone();
        match route {
            0 => {
                token.cancel(owner.access());
                drop(timer);
            }
            1 => drop(timer),
            2 => match timer.call(owner.access()) {
                JobCall::Complete(result) => {
                    result.unwrap();
                }
                JobCall::Deferred { .. } => {
                    panic!("the ordinary direct timer has no selected refusal")
                }
            },
            _ => unreachable!(),
        }
        assert!(token.revoked());
        let rejected_calls = Rc::new(Cell::new(0));
        let rejected = rejected_calls.clone();
        let drops = Rc::new(Cell::new(0));
        let probe = DropProbe(drops.clone());
        let refused = token
            .push_callback(move |_| {
                let _keep = &probe;
                rejected.set(rejected.get() + 1);
            })
            .unwrap_err();
        error_is(&refused, JobError::CancellationRevoked);
        assert_eq!(drops.get(), 1);
        token.cancel(owner.access());
        token.cancel(owner.access());
        assert!(token.revoked());
        assert_eq!(rejected_calls.get(), 0);
        assert_eq!(calls.get(), usize::from(route == 2));
        assert_eq!(drops.get(), 1);
    }
}

#[test]
fn cancellation_publishes_revocation_before_reentrant_registration_and_recursive_cancel() {
    let mut owner = context();
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    let log = Rc::new(RefCell::new(Vec::new()));
    let rejected_calls = Rc::new(Cell::new(0));
    let rejected_drops = Rc::new(Cell::new(0));
    let original_token = token.clone();
    let first_log = log.clone();
    let never = rejected_calls.clone();
    let drops = rejected_drops.clone();
    token
        .push_callback(move |access| {
            first_log.borrow_mut().push(1);
            assert!(original_token.revoked());
            let probe = DropProbe(drops.clone());
            let rejected = original_token
                .push_callback(move |_| {
                    let _keep = &probe;
                    never.set(never.get() + 1);
                })
                .unwrap_err();
            error_is(&rejected, JobError::CancellationRevoked);
            assert_eq!(drops.get(), 1);
            original_token.cancel(access);
        })
        .unwrap();
    let second_log = log.clone();
    token
        .push_callback(move |_| second_log.borrow_mut().push(2))
        .unwrap();
    token.cancel(owner.access());
    assert_eq!(*log.borrow(), [1, 2]);
    assert_eq!(rejected_calls.get(), 0);
    assert_eq!(rejected_drops.get(), 1);
    assert!(token.revoked());
    token.cancel(owner.access());
    assert_eq!(*log.borrow(), [1, 2]);
}

struct ReentrantDrop {
    token: CancellationToken,
    drops: Rc<Cell<usize>>,
    nested_error: Rc<RefCell<Option<JsError>>>,
}
impl Drop for ReentrantDrop {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        assert!(self.token.revoked());
        let result = self
            .token
            .push_callback(|_| panic!("a nested rejected callback cannot execute"));
        *self.nested_error.borrow_mut() = Some(result.unwrap_err());
    }
}

#[test]
fn rejected_capture_disposal_can_reenter_the_same_revoked_token() {
    let mut owner = context();
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    token.cancel(owner.access());
    let drops = Rc::new(Cell::new(0));
    let nested_error = Rc::new(RefCell::new(None));
    let probe = ReentrantDrop {
        token: token.clone(),
        drops: drops.clone(),
        nested_error: nested_error.clone(),
    };
    let error = token
        .push_callback(move |_| {
            let _keep = &probe;
            panic!("a rejected callback cannot execute");
        })
        .unwrap_err();
    error_is(&error, JobError::CancellationRevoked);
    assert_eq!(drops.get(), 1);
    error_is(
        nested_error.borrow().as_ref().unwrap(),
        JobError::CancellationRevoked,
    );
    assert!(token.revoked());
}

#[test]
fn cancellation_callback_unwind_preserves_revocation_and_never_replays_the_taken_callbacks() {
    let mut owner = context();
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        Duration::ZERO,
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
            panic!("authored cancellation unwind");
        })
        .unwrap();
    let unwind = catch_unwind(AssertUnwindSafe(|| token.cancel(owner.access()))).unwrap_err();
    assert_eq!(
        unwind.downcast_ref::<&str>().copied(),
        Some("authored cancellation unwind")
    );
    assert!(token.revoked());
    assert_eq!(calls.get(), 1);
    assert_eq!(drops.get(), 1);
    token.cancel(owner.access());
    assert_eq!(calls.get(), 1);
    let error = token
        .push_callback(|_| panic!("post-unwind rejection cannot execute"))
        .unwrap_err();
    error_is(&error, JobError::CancellationRevoked);
}

thread_local! { static ADMITTED: RefCell<Option<JsError>> = const { RefCell::new(None) }; }
fn admitted(error: JsError, owner: &mut Context) -> JsError {
    ADMITTED.with(|slot| assert!(slot.replace(Some(error)).is_none()));
    NativeFunction::from_fn_ptr(|_, _, _| {
        Err(ADMITTED.with(|slot| slot.borrow_mut().take().unwrap()))
    })
    .call(&JsValue::undefined(), &[], owner.access())
    .unwrap_err()
}

#[test]
fn actual_dynamic_errors_remain_typed_through_admission_diagnostics_and_erasure() {
    let mut owner = context();
    let mut task = ContextTask::from_future(std::future::ready(Ok(42)));
    assert_eq!(poll(&mut task, &owner.host_access()), Poll::Ready(Ok(42)));
    let completed_error = completed(poll(&mut task, &owner.host_access()));
    let timer = TimeoutJob::with_realm(
        |_| Ok(JsValue::undefined()),
        owner.realm().clone(),
        Duration::ZERO,
    );
    let token = timer.cancellation_token().clone();
    drop(timer);
    let revoked_error = token
        .push_callback(|_| panic!("rejected registration cannot execute"))
        .unwrap_err();
    for (error, expected) in [
        (completed_error, JobError::TaskCompleted),
        (revoked_error, JobError::CancellationRevoked),
    ] {
        let error = admitted(error, &mut owner);
        error_is(&error, expected);
        assert_eq!(
            error.clone().into_opaque(owner.access()).unwrap_err(),
            error
        );
        let outer = PanicError::new("authored job-state context").with_source(error.clone());
        let DiagnosticKind::Engine(DiagnosticEngine::Job(actual)) =
            outer.diagnostic().unwrap().kind()
        else {
            panic!("the nested diagnostic must preserve the exact job cause")
        };
        assert_eq!(actual, expected);
        let erased = error.into_erased(owner.access());
        assert_eq!(erased.as_engine(), Some(&ErasedEngineError::Job(expected)));
    }
}
