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
    Context, JsError, JsResult, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ProfileMode},
    job::{
        ContextTask, GenericJob, JobExecutor, JobRetirement, JobStatus, NativeAsyncJob,
        SimpleJobExecutor,
    },
};

fn context(executor: Rc<SimpleJobExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn options(limit: u64) -> ExecutionOptions {
    ExecutionOptions::new(0).with_job_claim_work_limit(limit)
}
fn claim_error(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobClaimWork { limit }
        ))
    );
}
fn counted(calls: &Rc<Cell<usize>>, realm: surgeist_js::realm::Realm) -> GenericJob {
    let calls = calls.clone();
    GenericJob::new(
        move |_| {
            calls.set(calls.get() + 1);
            Ok(JsValue::undefined())
        },
        realm,
    )
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}

#[test]
fn an_empty_drain_charges_one_async_bucket_head_and_no_job_action() {
    for limit in [0, 1] {
        let executor = Rc::new(SimpleJobExecutor::new());
        let mut owner = context(executor);
        let mut direct = None;
        let outcome = owner
            .with_execution(options(limit), |mut access| {
                let result = access.run_jobs();
                direct = result.as_ref().err().cloned();
                result
            })
            .unwrap();
        if limit == 0 {
            claim_error(direct.as_ref().unwrap(), 0);
            assert_eq!(outcome.result.unwrap_err(), direct.unwrap());
        } else {
            outcome.result.unwrap();
            assert!(direct.is_none());
        }
        // No clock bucket exists, and the established idle boundary exits
        // before the promise/generic snapshots on an empty runtime.
        assert_eq!(outcome.report.job_claim_work().unwrap(), limit);
        assert_eq!(outcome.report.job_preparations().unwrap(), 0);
        assert_eq!(outcome.report.job_invocations().unwrap(), 0);
        assert_eq!(outcome.report.job_polls().unwrap(), 0);
        assert!(!outcome.report.effects_started());
    }
}

#[test]
fn generic_bucket_prefix_refusals_keep_the_original_callback_for_a_fresh_operation() {
    for limit in 0..=4 {
        let executor = Rc::new(SimpleJobExecutor::new());
        let mut owner = context(executor);
        let calls = Rc::new(Cell::new(0));
        let job = counted(&calls, owner.realm().clone());
        let observer = job.observer();
        owner
            .enqueue_job(job.into())
            .unwrap()
            .into_result()
            .unwrap();
        let outcome = owner
            .with_execution(options(limit), |mut access| access.run_jobs())
            .unwrap();
        claim_error(&outcome.result.unwrap_err(), limit);
        // Async empty head, promise empty head, generic head, generic node.
        // Candidate capacity and copy have not been accepted at these limits.
        assert_eq!(outcome.report.job_claim_work().unwrap(), limit);
        assert_eq!(outcome.report.job_invocations().unwrap(), 0);
        assert!(!outcome.report.effects_started());
        assert_eq!(calls.get(), 0);
        assert_eq!(observer.status().unwrap(), JobStatus::Pending);
        let retried = owner
            .with_execution(options(100_000), |mut access| access.run_jobs())
            .unwrap();
        retried.result.unwrap();
        assert_eq!(retried.report.job_invocations().unwrap(), 1);
        assert_eq!(calls.get(), 1);
        assert_eq!(observer.status().unwrap(), JobStatus::Completed);
        assert_eq!(
            outcome.report.job_claim_work().unwrap(),
            limit,
            "a later operation cannot mutate an old receipt"
        );
    }
}

#[test]
fn a_claimed_callback_refused_by_its_action_budget_is_returned_for_later_execution() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = context(executor);
    let calls = Rc::new(Cell::new(0));
    let job = counted(&calls, owner.realm().clone());
    let observer = job.observer();
    owner
        .enqueue_job(job.into())
        .unwrap()
        .into_result()
        .unwrap();
    let first = owner
        .with_execution(
            options(100_000).with_job_invocation_limit(0),
            |mut access| access.run_jobs(),
        )
        .unwrap();
    assert_eq!(
        first.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 0 }
        ))
    );
    assert!(first.report.job_claim_work().unwrap() > 0);
    assert_eq!(first.report.job_invocations().unwrap(), 0);
    assert!(!first.report.effects_started());
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    assert_eq!(calls.get(), 0);
    let later = owner
        .with_execution(
            options(100_000).with_job_invocation_limit(1),
            |mut access| access.run_jobs(),
        )
        .unwrap();
    later.result.unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
}

#[test]
fn an_existing_instruction_stop_precedes_claim_admission_and_busy_access_does_not_replace_it() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = context(executor.clone());
    let calls = Rc::new(Cell::new(0));
    owner
        .enqueue_job(counted(&calls, owner.realm().clone()).into())
        .unwrap()
        .into_result()
        .unwrap();
    let first = owner
        .with_execution(options(0), |access| {
            let host = access.into_host_access();
            let original = host
                .with(|mut access| access.eval(Source::from_bytes("0")))
                .unwrap_err();
            assert_eq!(
                original.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            host.with(|_| {
                let mut nested = Box::pin(executor.clone().run_jobs_async(&host));
                let Poll::Ready(Err(error)) = poll(&mut nested) else {
                    panic!("an unavailable holder must refuse synchronously")
                };
                assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
                Ok(())
            })?;
            let mut drain = Box::pin(executor.clone().run_jobs_async(&host));
            let Poll::Ready(Err(error)) = poll(&mut drain) else {
                panic!("the retained stop must precede any queue snapshot")
            };
            assert_eq!(error, original);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        first.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(first.report.job_claim_work().unwrap(), 0);
    assert!(!first.report.effects_started());
    assert_eq!(calls.get(), 0);
    owner
        .with_execution(options(100_000), |mut access| access.run_jobs())
        .unwrap()
        .result
        .unwrap();
    assert_eq!(calls.get(), 1);
}

#[derive(Default)]
struct Gate {
    ready: Cell<bool>,
    factories: Cell<usize>,
    polls: Cell<usize>,
    drops: Cell<usize>,
    waker: RefCell<Option<Waker>>,
}
struct GateFuture(Rc<Gate>);
impl Future for GateFuture {
    type Output = JsResult<JsValue>;
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        self.0.polls.set(self.0.polls.get() + 1);
        TIME.set(TIME.get() + Duration::from_millis(11));
        if self.0.ready.get() {
            Poll::Ready(Ok(JsValue::new(42)))
        } else {
            *self.0.waker.borrow_mut() = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl Drop for GateFuture {
    fn drop(&mut self) {
        self.0.drops.set(self.0.drops.get() + 1);
    }
}
fn pending(gate: &Rc<Gate>) -> NativeAsyncJob {
    let gate = gate.clone();
    NativeAsyncJob::new(move |_| {
        gate.factories.set(gate.factories.get() + 1);
        TIME.set(TIME.get() + Duration::from_millis(7));
        Ok(ContextTask::from_future(GateFuture(gate)))
    })
}
fn release(gate: &Gate) {
    gate.ready.set(true);
    let wake = gate.waker.borrow_mut().take();
    wake.expect("the real pending task subscribed before release")
        .wake();
}

#[test]
fn repeated_nonready_snapshots_spend_claim_work_without_fabricating_task_polls() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = context(executor.clone());
    let gate = Rc::new(Gate::default());
    let job = pending(&gate);
    let observer = job.observer();
    owner
        .enqueue_job(job.into())
        .unwrap()
        .into_result()
        .unwrap();
    let first = owner
        .with_execution(options(1_000), |access| {
            let host = access.into_host_access();
            let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
            for _ in 0..256 {
                if let Poll::Ready(result) = poll(&mut driver) {
                    return result;
                }
            }
            panic!(
                "finite positive snapshot credits must exhaust without a wake of the resident task"
            )
        })
        .unwrap();
    claim_error(&first.result.unwrap_err(), 1_000);
    assert!(first.report.job_claim_work().unwrap() > 1);
    assert!(first.report.job_claim_work().unwrap() <= 1_000);
    assert_eq!(first.report.job_preparations().unwrap(), 1);
    assert_eq!(first.report.job_polls().unwrap(), 1);
    assert_eq!(gate.factories.get(), 1);
    assert_eq!(gate.polls.get(), 1);
    assert_eq!(gate.drops.get(), 0);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    release(&gate);
    let second = owner
        .with_execution(options(100_000), |mut access| access.run_jobs())
        .unwrap();
    second.result.unwrap();
    assert_eq!(second.report.job_preparations().unwrap(), 0);
    assert_eq!(second.report.job_polls().unwrap(), 1);
    assert_eq!(gate.polls.get(), 2);
    assert_eq!(gate.drops.get(), 1);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(first.report.job_polls().unwrap(), 1);
}

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static READS: Cell<usize> = const { Cell::new(0) };
}
fn clock() -> Duration {
    READS.set(READS.get() + 1);
    TIME.get()
}
fn modes() -> [ProfileMode; 5] {
    [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::ScopeName("job-claim"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::ScopeName("job-claim"),
        },
    ]
}
fn timed(mode: ProfileMode) -> bool {
    matches!(
        mode,
        ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
    )
}

#[test]
fn profiling_preserves_empty_snapshot_limits_and_untimed_modes_read_no_clock() {
    for mode in modes() {
        for limit in [0, 1] {
            let executor = Rc::new(SimpleJobExecutor::new());
            let mut owner = context(executor);
            TIME.set(Duration::ZERO);
            READS.set(0);
            let outcome = owner
                .with_execution(
                    options(limit).with_profile(mode).with_profile_clock(clock),
                    |mut access| access.run_jobs(),
                )
                .unwrap();
            if limit == 0 {
                claim_error(&outcome.result.unwrap_err(), 0);
            } else {
                outcome.result.unwrap();
            }
            assert_eq!(outcome.report.job_claim_work().unwrap(), limit);
            assert!(!outcome.report.effects_started());
            if mode == ProfileMode::Off {
                assert!(outcome.report.profile().unwrap().is_none());
            } else {
                let profile = outcome.report.profile().unwrap().unwrap();
                let claim = profile
                    .scopes
                    .iter()
                    .find(|scope| scope.name == "job-claim")
                    .unwrap();
                // A successful empty async snapshot is followed by the empty
                // clock-range action before the idle exit. Refusal stops at
                // the first action. An absent clock bucket costs zero work.
                assert_eq!(claim.phases, if limit == 0 { 1 } else { 2 });
                assert_eq!(claim.phase_elapsed, timed(mode).then_some(Duration::ZERO));
                assert!(profile.events.len() <= 8);
            }
            if !timed(mode) {
                assert_eq!(READS.get(), 0);
            }
        }
    }
}

#[test]
fn claim_phases_exclude_factory_poll_pending_wait_and_later_host_gap() {
    for mode in modes() {
        let executor = Rc::new(SimpleJobExecutor::new());
        let mut owner = context(executor.clone());
        let gate = Rc::new(Gate::default());
        owner
            .enqueue_job(pending(&gate).into())
            .unwrap()
            .into_result()
            .unwrap();
        TIME.set(Duration::ZERO);
        READS.set(0);
        let outcome = owner
            .with_execution(
                options(100_000)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                |access| {
                    let host = access.into_host_access();
                    let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
                    for _ in 0..16 {
                        assert!(poll(&mut driver).is_pending());
                        if gate.polls.get() == 1 {
                            break;
                        }
                    }
                    assert_eq!(gate.polls.get(), 1);
                    drop(driver);
                    TIME.set(TIME.get() + Duration::from_millis(100));
                    Ok(())
                },
            )
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(gate.factories.get(), 1);
        assert_eq!(gate.polls.get(), 1);
        assert_eq!(gate.drops.get(), 0);
        if mode != ProfileMode::Off {
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(
                profile.elapsed,
                timed(mode).then_some(Duration::from_millis(118))
            );
            let claim = profile
                .scopes
                .iter()
                .find(|scope| scope.name == "job-claim")
                .unwrap();
            assert_eq!(claim.phase_elapsed, timed(mode).then_some(Duration::ZERO));
            assert!(claim.phases >= 1);
        }
        if !timed(mode) {
            assert_eq!(READS.get(), 0);
        }
        let receipt = outcome.report.profile().unwrap().cloned();
        release(&gate);
        owner
            .with_execution(options(100_000), |mut access| access.run_jobs())
            .unwrap()
            .result
            .unwrap();
        assert_eq!(gate.polls.get(), 2);
        assert_eq!(gate.drops.get(), 1);
        assert_eq!(outcome.report.profile().unwrap(), receipt.as_ref());
    }
}

#[test]
fn a_synchronous_callback_and_postdrain_gap_are_outside_every_claim_phase() {
    for mode in modes() {
        let executor = Rc::new(SimpleJobExecutor::new());
        let mut owner = context(executor);
        let called = Rc::new(Cell::new(0));
        let counter = called.clone();
        owner
            .enqueue_job(
                GenericJob::new(
                    move |_| {
                        counter.set(counter.get() + 1);
                        TIME.set(TIME.get() + Duration::from_millis(7));
                        Ok(JsValue::undefined())
                    },
                    owner.realm().clone(),
                )
                .into(),
            )
            .unwrap()
            .into_result()
            .unwrap();
        TIME.set(Duration::ZERO);
        READS.set(0);
        let outcome = owner
            .with_execution(
                options(100_000)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                |mut access| {
                    access.run_jobs()?;
                    TIME.set(TIME.get() + Duration::from_millis(100));
                    Ok(())
                },
            )
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(called.get(), 1);
        assert_eq!(outcome.report.job_invocations().unwrap(), 1);
        assert!(outcome.report.effects_started());
        if mode != ProfileMode::Off {
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(
                profile.elapsed,
                timed(mode).then_some(Duration::from_millis(107))
            );
            let claim = profile
                .scopes
                .iter()
                .find(|scope| scope.name == "job-claim")
                .unwrap();
            assert_eq!(claim.phase_elapsed, timed(mode).then_some(Duration::ZERO));
        }
        if !timed(mode) {
            assert_eq!(READS.get(), 0);
        }
    }
}

#[test]
fn a_completed_callback_and_its_newly_enqueued_job_survive_refusal_of_the_old_tail() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = context(executor);
    let log = Rc::new(RefCell::new(Vec::new()));
    let first_log = log.clone();
    owner
        .enqueue_job(
            GenericJob::new(
                move |mut access| {
                    first_log.borrow_mut().push(1);
                    let independent = first_log.clone();
                    access
                        .enqueue_job(
                            GenericJob::new(
                                move |_| {
                                    independent.borrow_mut().push(3);
                                    Ok(JsValue::undefined())
                                },
                                access.realm().clone(),
                            )
                            .into(),
                        )
                        .unwrap()
                        .into_result()
                        .unwrap();
                    Ok(JsValue::undefined())
                },
                owner.realm().clone(),
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let tail_log = log.clone();
    let tail = GenericJob::new(
        move |_| {
            tail_log.borrow_mut().push(2);
            Ok(JsValue::undefined())
        },
        owner.realm().clone(),
    );
    let observer = tail.observer();
    owner
        .enqueue_job(tail.into())
        .unwrap()
        .into_result()
        .unwrap();
    let first = owner
        .with_execution(
            options(100_000).with_job_invocation_limit(1),
            |mut access| access.run_jobs(),
        )
        .unwrap();
    assert_eq!(
        first.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 1 }
        ))
    );
    assert_eq!(*log.borrow(), [1]);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    assert_eq!(first.report.job_invocations().unwrap(), 1);
    owner
        .with_execution(
            options(100_000).with_job_invocation_limit(2),
            |mut access| access.run_jobs(),
        )
        .unwrap()
        .result
        .unwrap();
    assert_eq!(*log.borrow(), [1, 2, 3]);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
}

#[test]
fn explicit_executor_cancellation_at_the_top_loop_remains_separate_from_claim_refusal() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut owner = context(executor.clone());
    let calls = Rc::new(Cell::new(0));
    let job = counted(&calls, owner.realm().clone());
    let observer = job.observer();
    owner
        .enqueue_job(job.into())
        .unwrap()
        .into_result()
        .unwrap();
    executor
        .get_cancellation_token()
        .store(true, Ordering::Relaxed);
    let outcome = owner
        .with_execution(options(0), |mut access| access.run_jobs())
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(outcome.report.job_claim_work().unwrap(), 0);
    assert!(!outcome.report.effects_started());
    assert_eq!(calls.get(), 0);
    assert_eq!(
        observer.status().unwrap(),
        JobStatus::Retired(JobRetirement::Cancelled)
    );
    assert!(!executor.get_cancellation_token().load(Ordering::Relaxed));
}
