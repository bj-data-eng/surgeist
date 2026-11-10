use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue,
    builtins::promise::PromiseState,
    error::{EngineError, EngineResourceError, PanicError},
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobRetirement, JobStatus,
    },
    object::builtins::{JsFuture, JsPromise},
};

enum Action {
    Queue,
    Reject(JsError),
    InlineFailure(JsError),
}
#[derive(Default)]
struct ScriptedExecutor {
    plan: RefCell<VecDeque<Action>>,
    queue: RefCell<VecDeque<Job>>,
    planned_observers: RefCell<Vec<JobObserver>>,
    inside_inline: Cell<bool>,
}
struct InlineScope<'a>(&'a Cell<bool>);
impl Drop for InlineScope<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
impl JobExecutor for ScriptedExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        // Nested publications belong to the actual accepted inline callback.
        // They are retained independently, rather than consuming the outer
        // authored source-reaction plan or being mislabeled as its next tail.
        let action = if self.inside_inline.get() {
            None
        } else {
            self.plan.borrow_mut().pop_front()
        };
        let Some(action) = action else {
            self.queue.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        };
        let Job::PromiseJob(reaction) = &job else {
            panic!("the plan applies to the three authored source reactions")
        };
        self.planned_observers
            .borrow_mut()
            .push(reaction.observer());
        match action {
            Action::Queue => {
                self.queue.borrow_mut().push_back(job);
                Ok(JobAcceptance::Accepted)
            }
            Action::Reject(error) => Err(JobPublicationError::new(
                job,
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            Action::InlineFailure(error) => {
                let Job::PromiseJob(reaction) = job else {
                    unreachable!()
                };
                self.inside_inline.set(true);
                let scope = InlineScope(&self.inside_inline);
                match reaction.call(access.reborrow()) {
                    JobCall::Complete(result) => {
                        result.unwrap();
                    }
                    JobCall::Deferred { .. } => panic!(
                        "the authored inline reaction must actually execute before the host hook fails"
                    ),
                }
                drop(scope);
                // This configured host publication hook fails only after its
                // real inline action consumed the wrapper and settled a target.
                Ok(JobAcceptance::AcceptedWithError(
                    JobPublicationCause::try_new(error, access).unwrap(),
                ))
            }
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the test dispatches the retained real promise wrappers explicitly")
    }
}
fn context(executor: Rc<ScriptedExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
#[derive(Default)]
struct WakeCount {
    count: Mutex<usize>,
    changed: Condvar,
}
impl WakeCount {
    fn record(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        drop(count);
        self.changed.notify_all();
    }
    fn current(&self) -> usize {
        *self.count.lock().unwrap()
    }
    fn after(&self, baseline: usize) {
        let (count, _) = self
            .changed
            .wait_timeout_while(
                self.count.lock().unwrap(),
                Duration::from_secs(5),
                |count| *count <= baseline,
            )
            .unwrap();
        assert!(
            *count > baseline,
            "the preexisting native target waiter did not receive a fresh publication/retirement notification: before={baseline}, observed={}",
            *count
        );
    }
}
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.record();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.record();
    }
}
fn poll(future: &mut JsFuture, waker: &Waker) -> Poll<JsResult<JsValue>> {
    Pin::new(future).poll(&mut TaskContext::from_waker(waker))
}
fn terminal(future: &mut JsFuture, expected: &JsValue, rejected: bool) {
    match poll(future, Waker::noop()) {
        Poll::Ready(Ok(value)) if !rejected => assert_eq!(&value, expected),
        Poll::Ready(Err(error)) if rejected => assert_eq!(error.as_opaque(), Some(expected)),
        other => panic!("the actual target must retain its authored guest settlement: {other:?}"),
    }
}
fn retired(future: &mut JsFuture) {
    let Poll::Ready(Err(error)) = poll(future, Waker::noop()) else {
        panic!("disposed publication must complete the original native wait")
    };
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::Dropped))
    );
}
fn fanout(reject_source: bool, inline_first: bool) {
    let executor = Rc::new(ScriptedExecutor::default());
    let mut owner = context(executor.clone());
    let (source, source_resolvers) = JsPromise::new_pending(owner.access()).unwrap();
    let value: JsValue = JsObject::with_null_proto(owner.runtime()).unwrap().into();
    let mut targets = Vec::new();
    let mut futures = Vec::new();
    let mut wakes = Vec::new();
    for _ in 0..3 {
        let (target, resolvers) = JsPromise::new_pending(owner.access()).unwrap();
        // Establish the target's actual native association before attaching
        // these real resolving functions as the source's ordered reactions.
        let mut future = target.clone().into_js_future(owner.access()).unwrap();
        let wake = Arc::new(WakeCount::default());
        assert!(poll(&mut future, &Waker::from(wake.clone())).is_pending());
        drop(
            source
                .then(
                    Some(resolvers.resolve),
                    Some(resolvers.reject),
                    owner.access(),
                )
                .unwrap(),
        );
        targets.push(target);
        futures.push(future);
        wakes.push(wake);
    }
    let mut baseline = Vec::new();
    for (future, wake) in futures.iter_mut().zip(&wakes) {
        assert!(poll(future, &Waker::from(wake.clone())).is_pending());
        baseline.push(wake.current());
    }
    let first: JsError = PanicError::new("first authored publication failure")
        .with_source(JsNativeError::range().with_message("first retained native cause"))
        .into();
    let later: JsError = EngineResourceError::Allocation.into();
    *executor.plan.borrow_mut() = VecDeque::from([
        if inline_first {
            Action::InlineFailure(first.clone())
        } else {
            Action::Reject(first.clone())
        },
        Action::Queue,
        Action::Reject(later),
    ]);
    let resolver = if reject_source {
        source_resolvers.reject
    } else {
        source_resolvers.resolve
    };
    let error = resolver
        .call(
            &JsValue::undefined(),
            std::slice::from_ref(&value),
            owner.access(),
        )
        .unwrap_err();
    assert_eq!(
        error, first,
        "a later publication failure cannot replace the first actual cause"
    );
    assert!(
        executor.plan.borrow().is_empty(),
        "all three authored prepared reactions must reach their publication policy"
    );
    assert_eq!(
        source.state().unwrap(),
        if reject_source {
            PromiseState::Rejected(value.clone())
        } else {
            PromiseState::Fulfilled(value.clone())
        }
    );
    let observers = executor.planned_observers.borrow().clone();
    assert_eq!(observers.len(), 3);
    assert_eq!(
        observers[0].status(),
        Ok(if inline_first {
            JobStatus::Completed
        } else {
            JobStatus::Retired(JobRetirement::Dropped)
        })
    );
    assert_eq!(observers[1].status(), Ok(JobStatus::Pending));
    assert_eq!(
        observers[2].status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(targets[2].state().unwrap(), PromiseState::Pending);
    wakes[2].after(baseline[2]);
    retired(&mut futures[2]);
    if inline_first {
        wakes[0].after(baseline[0]);
        terminal(&mut futures[0], &value, reject_source);
        assert_eq!(
            targets[0].state().unwrap(),
            if reject_source {
                PromiseState::Rejected(value.clone())
            } else {
                PromiseState::Fulfilled(value.clone())
            }
        );
    } else {
        assert_eq!(targets[0].state().unwrap(), PromiseState::Pending);
        wakes[0].after(baseline[0]);
        retired(&mut futures[0]);
    }
    assert_eq!(targets[1].state().unwrap(), PromiseState::Pending);
    assert!(poll(&mut futures[1], &Waker::from(wakes[1].clone()),).is_pending());
    let before_middle_completion = wakes[1].current();
    for _ in 0..16 {
        if targets[1].state().unwrap() != PromiseState::Pending {
            break;
        }
        let queued = executor
            .queue
            .borrow_mut()
            .pop_front()
            .expect("the middle accepted reaction retains actual dispatch custody");
        let Job::PromiseJob(job) = queued else {
            panic!("only actual source/delivery promise reactions are queued")
        };
        match job.call(owner.access()) {
            JobCall::Complete(result) => {
                result.unwrap();
            }
            JobCall::Deferred { .. } => panic!("fresh dispatch cannot lose the accepted tail"),
        }
    }
    wakes[1].after(before_middle_completion);
    terminal(&mut futures[1], &value, reject_source);
    assert_eq!(observers[1].status(), Ok(JobStatus::Completed));
    assert_eq!(targets[2].state().unwrap(), PromiseState::Pending);
    assert!(observers.iter().all(|observer| observer.fault().is_none()));
}

#[test]
fn rejected_prepared_reactions_do_not_cut_off_tails_notifications_or_committed_source_values() {
    for reject_source in [false, true] {
        fanout(reject_source, false);
    }
}

#[test]
fn an_accepted_inline_failure_cannot_unsettle_its_target_or_dispose_unrelated_prepared_tails() {
    for reject_source in [false, true] {
        fanout(reject_source, true);
    }
}
