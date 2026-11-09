#![allow(unused_crate_dependencies)]
use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{Context, HostContext, JsError, JsResult, job::ContextTask};

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn poll<T: 'static>(task: &mut ContextTask<T>, host: &HostContext<'_>) -> Poll<JsResult<T>> {
    task.poll(&mut TaskContext::from_waker(Waker::noop()), host)
}
fn terminal_error<T>(result: Poll<JsResult<T>>) -> JsError {
    let Poll::Ready(Err(error)) = result else {
        panic!("a completed task must return a host invalid-state error")
    };
    // EngineError is the existing public family for errors that guest code
    // cannot catch. The precise new invalid-state cause is qualified later.
    assert!(error.as_engine().is_some());
    error
}

#[test]
fn first_future_results_preserve_success_conversion_and_an_ordinary_guest_error() {
    for guest_error in [false, true] {
        let completions = Rc::new(Cell::new(0));
        let calls = completions.clone();
        let original = JsError::from_opaque(7.into());
        let supplied = if guest_error {
            Err(original.clone())
        } else {
            Ok(40)
        };
        let mut task =
            ContextTask::from_future(std::future::ready(supplied)).then(move |value, _| {
                calls.set(calls.get() + 1);
                Ok(value + 2)
            });
        let mut owner = Context::default();
        let host = owner.host_access();
        assert_eq!(
            poll(&mut task, &host),
            if guest_error {
                Poll::Ready(Err(original))
            } else {
                Poll::Ready(Ok(42))
            }
        );
        assert_eq!(completions.get(), usize::from(!guest_error));
    }
}

#[test]
fn completed_poll_tasks_reject_further_polls_without_entering_the_owned_callback() {
    let polls = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let seen = polls.clone();
    let mut task = ContextTask::from_poll(move |_, _| {
        let _keep = &probe;
        seen.set(seen.get() + 1);
        Poll::Ready(Ok(42))
    });
    let mut owner = Context::default();
    let host = owner.host_access();
    assert_eq!(poll(&mut task, &host), Poll::Ready(Ok(42)));
    let second = catch_unwind(AssertUnwindSafe(|| poll(&mut task, &host)));
    assert_eq!(
        polls.get(),
        1,
        "terminal state must stop the actual owned poll callback"
    );
    terminal_error(second.expect("duplicate poll must return its invalid-state outcome"));
    terminal_error(poll(&mut task, &host));
    assert_eq!(polls.get(), 1);
    drop(task);
    assert_eq!(drops.get(), 1);
}

#[test]
fn completed_then_tasks_do_not_repoll_the_source_or_consume_completion_twice() {
    let polls = Rc::new(Cell::new(0));
    let completions = Rc::new(Cell::new(0));
    let source_drops = Rc::new(Cell::new(0));
    let completion_drops = Rc::new(Cell::new(0));
    let source_probe = DropProbe(source_drops.clone());
    let completion_probe = DropProbe(completion_drops.clone());
    let source_calls = polls.clone();
    let complete_calls = completions.clone();
    let mut task = ContextTask::from_poll(move |_, _| {
        let _keep = &source_probe;
        source_calls.set(source_calls.get() + 1);
        Poll::Ready(Ok(40))
    })
    .then(move |value, _| {
        let _keep = &completion_probe;
        complete_calls.set(complete_calls.get() + 1);
        Ok(value + 2)
    });
    let mut owner = Context::default();
    let host = owner.host_access();
    assert_eq!(poll(&mut task, &host), Poll::Ready(Ok(42)));
    assert_eq!(completion_drops.get(), 1);
    let second = catch_unwind(AssertUnwindSafe(|| poll(&mut task, &host)));
    assert_eq!(
        polls.get(),
        1,
        "a consumed completion must be guarded before source reentry"
    );
    assert_eq!(completions.get(), 1);
    terminal_error(second.expect("the consumed completion must not be reached"));
    terminal_error(poll(&mut task, &host));
    assert_eq!(polls.get(), 1);
    assert_eq!(completions.get(), 1);
    drop(task);
    assert_eq!(source_drops.get(), 1);
    assert_eq!(completion_drops.get(), 1);
}

#[test]
fn an_ordinary_error_completes_the_task_once_without_running_its_success_conversion() {
    let polls = Rc::new(Cell::new(0));
    let completions = Rc::new(Cell::new(0));
    let source_calls = polls.clone();
    let complete_calls = completions.clone();
    let original = JsError::from_opaque(7.into());
    let supplied = original.clone();
    let mut task: ContextTask<i32> = ContextTask::from_poll(move |_, _| {
        source_calls.set(source_calls.get() + 1);
        Poll::Ready(Err::<i32, _>(supplied.clone()))
    })
    .then(move |value, _| {
        complete_calls.set(complete_calls.get() + 1);
        Ok(value)
    });
    let mut owner = Context::default();
    let host = owner.host_access();
    assert_eq!(poll(&mut task, &host), Poll::Ready(Err(original)));
    let second = catch_unwind(AssertUnwindSafe(|| poll(&mut task, &host)));
    assert_eq!(
        polls.get(),
        1,
        "an ordinary Err is also an actual terminal result"
    );
    assert_eq!(completions.get(), 0);
    terminal_error(second.expect("completed-error observation must return normally"));
}

#[test]
fn a_pending_stage_remains_viable_until_one_result_and_one_completion_are_delivered() {
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
    let mut owner = Context::default();
    let host = owner.host_access();
    assert_eq!(poll(&mut task, &host), Poll::Pending);
    assert_eq!(polls.get(), 1);
    assert_eq!(completions.get(), 0);
    assert_eq!(poll(&mut task, &host), Poll::Ready(Ok(42)));
    assert_eq!(polls.get(), 2);
    assert_eq!(completions.get(), 1);
    let second = catch_unwind(AssertUnwindSafe(|| poll(&mut task, &host)));
    assert_eq!(polls.get(), 2);
    assert_eq!(completions.get(), 1);
    terminal_error(second.expect("the delivered result seals the staged task"));
}
