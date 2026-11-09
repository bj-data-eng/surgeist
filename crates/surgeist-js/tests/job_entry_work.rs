#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
    time::Duration,
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsResult, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{
        EventFilter, ExecutionOptions, ExecutionReceipt, ExecutionReport, ExecutionTermination,
        ProfileMode,
    },
    job::{
        ContextTask, GenericJob, IntervalJob, Job, JobAcceptance, JobCall, JobExecutor,
        JobInvocation, JobPublicationCause, JobPublicationError, JobStep, NativeAsyncJob,
        NativeJob, NativeJobFn, PromiseJob, SimpleJobExecutor, TimeoutJob,
    },
};

fn vm_context() -> Context {
    let builder = Context::builder();
    // Keep the independent fuzz guard from preempting operation-limit stimuli.
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

thread_local! { static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) }; static CLOCK_READS: Cell<usize> = const { Cell::new(0) }; }
fn clock() -> Duration {
    CLOCK_READS.set(CLOCK_READS.get() + 1);
    TIME.get()
}
fn modes() -> [ProfileMode; 6] {
    [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::All,
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 64,
            filter: EventFilter::ScopeName("job-poll"),
        },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 1,
            filter: EventFilter::ScopeName("job-poll"),
        },
    ]
}
fn timed(mode: ProfileMode) -> bool {
    matches!(
        mode,
        ProfileMode::Summary { timing: true } | ProfileMode::Detailed { timing: true, .. }
    )
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn step(job: &mut NativeAsyncJob, host: &surgeist_js::HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
}
async fn yield_once() {
    let mut yielded = false;
    std::future::poll_fn(move |cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
fn complete<J>(call: JobCall<J>) -> JsResult<JsValue> {
    match call {
        JobCall::Complete(result) => result,
        JobCall::Deferred { .. } => panic!("expected an admitted synchronous invocation"),
    }
}
fn invoked(call: JobInvocation) -> JsResult<JsValue> {
    match call {
        JobInvocation::Complete(result) => result,
        JobInvocation::Deferred(_) => panic!("expected an admitted reusable invocation"),
    }
}
fn deferred<J>(call: JobCall<J>) -> (J, JsError) {
    match call {
        JobCall::Deferred { job, error } => (job, error),
        JobCall::Complete(_) => panic!("expected an uncalled owned job"),
    }
}
fn assert_counts(report: &ExecutionReport, prep: u64, invoke: u64, polls: u64) {
    assert_eq!(report.job_preparations().unwrap(), prep);
    assert_eq!(report.job_invocations().unwrap(), invoke);
    assert_eq!(report.job_polls().unwrap(), polls);
    assert_eq!(report.instructions().unwrap(), 0);
}
fn assert_phase(report: &ExecutionReport, name: &str, phases: u64, elapsed: Option<Duration>) {
    let phase = report
        .profile()
        .unwrap()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap();
    assert_eq!(phase.phases, phases, "{name}");
    assert_eq!(phase.phase_elapsed, elapsed, "{name}");
}
fn counted(counter: &Rc<Cell<u64>>) -> NativeJob {
    let counter = counter.clone();
    NativeJob::new(move |_| {
        counter.set(counter.get() + 1);
        Ok(JsValue::new(42))
    })
}
fn reusable_counted(counter: &Rc<Cell<u64>>) -> NativeJobFn {
    let counter = counter.clone();
    NativeJobFn::new(move |_| {
        counter.set(counter.get() + 1);
        Ok(JsValue::new(42))
    })
}
fn ready(value: JsValue) -> ContextTask<JsValue> {
    ContextTask::from_future(async move { Ok(value) })
}

#[test]
fn preparation_zero_exact_and_one_short_caps_count_only_created_inner_futures() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let called = Rc::new(Cell::new(0));
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_job_preparation_limit(limit),
                |access| {
                    let holder = access.into_host_access();
                    for index in 0..2 {
                        let calls = called.clone();
                        let mut job = NativeAsyncJob::new(move |_| {
                            calls.set(calls.get() + 1);
                            Ok(ready(JsValue::new(42)))
                        });
                        match step(&mut job, &holder) {
                            JobStep::Prepared => assert!(index < limit),
                            JobStep::Deferred(error) => {
                                assert!(index >= limit);
                                assert_eq!(
                                    error.as_engine(),
                                    Some(&EngineError::RuntimeLimit(
                                        RuntimeLimitError::JobPreparation { limit }
                                    ))
                                );
                            }
                            _ => panic!("preparation must perform no inner poll"),
                        }
                    }
                    Ok(())
                },
            )
            .unwrap();
        if limit < 2 {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobPreparation { limit }
                ))
            );
        } else {
            outcome.result.unwrap();
        }
        assert_counts(&outcome.report, limit.min(2), 0, 0);
        assert_eq!(called.get(), limit.min(2));
        assert_eq!(
            outcome.report.effects_started(),
            limit != 0,
            "admitted preparation now dispatches the host factory"
        );
    }
}
#[test]
fn invocation_zero_exact_and_one_short_caps_refuse_before_reusable_callbacks() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let calls = Rc::new(Cell::new(0));
        let job = reusable_counted(&calls);
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_job_invocation_limit(limit),
                |mut access| {
                    for index in 0..2 {
                        match job.call(access.reborrow()) {
                            JobInvocation::Complete(result) => {
                                assert!(index < limit);
                                assert_eq!(result?, JsValue::new(42));
                            }
                            JobInvocation::Deferred(error) => {
                                assert!(index >= limit);
                                assert_eq!(
                                    error.as_engine(),
                                    Some(&EngineError::RuntimeLimit(
                                        RuntimeLimitError::JobInvocation { limit }
                                    ))
                                );
                            }
                        }
                    }
                    Ok(())
                },
            )
            .unwrap();
        if limit < 2 {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobInvocation { limit }
                ))
            );
        } else {
            outcome.result.unwrap();
        }
        assert_counts(&outcome.report, 0, limit.min(2), 0);
        assert_eq!(calls.get(), limit.min(2));
        assert_eq!(outcome.report.effects_started(), limit != 0);
    }
}
#[test]
fn poll_zero_exact_and_one_short_caps_count_pending_and_ready_separately() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let entries = Rc::new(Cell::new(0));
        let calls = entries.clone();
        let mut first = true;
        let mut job = NativeAsyncJob::new(move |_| {
            Ok(ContextTask::from_poll(move |cx, _| {
                calls.set(calls.get() + 1);
                if first {
                    first = false;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    Poll::Ready(Ok(JsValue::new(42)))
                }
            }))
        });
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_job_poll_limit(limit),
                |access| {
                    let holder = access.into_host_access();
                    assert!(matches!(step(&mut job, &holder), JobStep::Prepared));
                    if limit != 0 {
                        assert!(matches!(step(&mut job, &holder), JobStep::Pending));
                    }
                    match step(&mut job, &holder) {
                        JobStep::Complete(result) => {
                            assert_eq!(limit, 2);
                            assert_eq!(result?, JsValue::new(42));
                        }
                        JobStep::Deferred(error) => {
                            assert!(limit < 2);
                            assert_eq!(
                                error.as_engine(),
                                Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
                                    limit
                                }))
                            );
                        }
                        _ => panic!("the next step must complete or defer at the poll cap"),
                    }
                    Ok(())
                },
            )
            .unwrap();
        if limit < 2 {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
                    limit
                }))
            );
        } else {
            outcome.result.unwrap();
        }
        assert_counts(&outcome.report, 1, 0, limit.min(2));
        assert_eq!(entries.get(), limit.min(2));
        assert!(
            outcome.report.effects_started(),
            "the factory was admitted even when no poll was admitted"
        );
    }
}
#[test]
fn every_wrapper_delegates_to_exactly_one_synchronous_invocation_owner() {
    let mut context = Context::default();
    let realm = context.realm().clone();
    let calls = Rc::new(Cell::new(0));
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            complete(counted(&calls).call(access.reborrow()))?;
            let reusable = reusable_counted(&calls);
            invoked(reusable.call(access.reborrow()))?;
            invoked(reusable.call(access.reborrow()))?;
            let p = calls.clone();
            complete(
                PromiseJob::new(move |_| {
                    p.set(p.get() + 1);
                    Ok(JsValue::new(42))
                })
                .call(access.reborrow()),
            )?;
            let g = calls.clone();
            complete(
                GenericJob::new(
                    move |_| {
                        g.set(g.get() + 1);
                        Ok(JsValue::new(42))
                    },
                    realm,
                )
                .call(access.reborrow()),
            )?;
            complete(TimeoutJob::new(counted(&calls), 0).call(access.reborrow()))?;
            let interval = IntervalJob::new(reusable_counted(&calls), 0);
            invoked(interval.call(access.reborrow()))?;
            invoked(interval.call(access.reborrow()))?;
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(calls.get(), 8);
    assert_counts(&outcome.report, 0, 8, 0);
    assert!(outcome.report.effects_started());
}
#[test]
fn callback_vm_instructions_retain_their_own_units_beside_job_invocation() {
    let control = vm_context()
        .eval_with_budget(Source::from_bytes("40 + 2"), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    assert!(control.report.instructions().unwrap() > 0);
    let mut context = vm_context();
    let outcome = context
        .with_execution(
            ExecutionOptions::new(64).with_job_invocation_limit(1),
            |access| {
                complete(
                    NativeJob::new(|mut access| access.eval(Source::from_bytes("40 + 2")))
                        .call(access),
                )
            },
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(outcome.report.job_preparations().unwrap(), 0);
    assert_eq!(outcome.report.job_invocations().unwrap(), 1);
    assert_eq!(outcome.report.job_polls().unwrap(), 0);
    assert_eq!(
        outcome.report.instructions().unwrap(),
        control.report.instructions().unwrap()
    );
}
#[test]
fn reentry_preserves_the_first_invocation_stop_over_success_and_later_job_dimensions() {
    let mut context = Context::default();
    let calls = Rc::new(Cell::new(0));
    let denied = calls.clone();
    let outer = NativeJob::new(move |mut access| {
        let (_uncalled, error) = deferred(counted(&denied).call(access.reborrow()));
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::JobInvocation { limit: 1 }
            ))
        );
        Ok(JsValue::new(42))
    });
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_invocation_limit(1)
                .with_job_preparation_limit(0)
                .with_job_poll_limit(0),
            |mut access| {
                let error = complete(outer.call(access.reborrow())).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobInvocation { limit: 1 }
                    ))
                );
                let host = access.into_host_access();
                let mut job = NativeAsyncJob::new(|_| Ok(ready(JsValue::new(42))));
                let JobStep::Deferred(error) = step(&mut job, &host) else {
                    panic!("the retained stop must defer preparation")
                };
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobInvocation { limit: 1 }
                    ))
                );
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 1 }
        ))
    );
    assert_counts(&outcome.report, 0, 1, 0);
    assert_eq!(calls.get(), 0);
}
#[test]
fn availability_realm_validation_and_preexisting_stops_do_not_charge_new_job_units() {
    let foreign = Context::default();
    let foreign_realm = foreign.realm().clone();
    let mut context = vm_context();
    let calls = Rc::new(Cell::new(0));
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(1)
                .with_job_invocation_limit(1)
                .with_job_poll_limit(1),
            |access| {
                let host = access.into_host_access();
                host.with(|_| {
                    let c = calls.clone();
                    let mut job = NativeAsyncJob::new(move |_| {
                        c.set(c.get() + 1);
                        Ok(ready(JsValue::new(42)))
                    });
                    let JobStep::Deferred(error) = step(&mut job, &host) else {
                        panic!("a competing lease must defer preparation")
                    };
                    assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
                    Ok(())
                })?;
                let c = calls.clone();
                let mut invalid = NativeAsyncJob::with_realm(
                    move |_| {
                        c.set(c.get() + 1);
                        Ok(ready(JsValue::new(42)))
                    },
                    foreign_realm.clone(),
                );
                let JobStep::Deferred(error) = step(&mut invalid, &host) else {
                    panic!("a foreign realm must defer preparation")
                };
                assert!(error.as_native().is_some_and(|e| e.is_type()));
                host.with(|access| {
                    let c = calls.clone();
                    let (_uncalled, error) = deferred(
                        NativeJob::with_realm(
                            move |_| {
                                c.set(c.get() + 1);
                                Ok(JsValue::new(42))
                            },
                            foreign_realm,
                        )
                        .call(access),
                    );
                    assert!(error.as_native().is_some_and(|e| e.is_type()));
                    Ok(())
                })?;
                let c = calls.clone();
                let mut valid = NativeAsyncJob::new(move |_| {
                    Ok(ContextTask::from_future(async move {
                        c.set(c.get() + 1);
                        Ok(JsValue::new(42))
                    }))
                });
                assert!(matches!(step(&mut valid, &host), JobStep::Prepared));
                let JobStep::Complete(result) = step(&mut valid, &host) else {
                    panic!("the valid task must complete")
                };
                assert_eq!(result?, JsValue::new(42));
                host.with(|access| complete(counted(&calls).call(access)))?;
                let first = host
                    .with(|mut access| access.eval(Source::from_bytes("0")))
                    .unwrap_err();
                assert_eq!(
                    first.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 0
                    }))
                );
                let mut stopped = NativeAsyncJob::new(|_| Ok(ready(JsValue::new(42))));
                let JobStep::Deferred(error) = step(&mut stopped, &host) else {
                    panic!("a retained stop must defer new actions")
                };
                assert_eq!(error.as_engine(), first.as_engine());
                let error = host
                    .with(|access| {
                        let (_uncalled, error) = deferred(counted(&calls).call(access));
                        Err::<(), _>(error)
                    })
                    .unwrap_err();
                assert_eq!(error.as_engine(), first.as_engine());
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_counts(&outcome.report, 1, 1, 1);
    assert_eq!(calls.get(), 2);
}
struct InlineExecutor {
    errors: RefCell<Vec<JsError>>,
    deferred: RefCell<Vec<Job>>,
}
impl JobExecutor for InlineExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Job::PromiseJob(job) = job else {
            panic!("only authored promise jobs are sent here")
        };
        match job.call(access.reborrow()) {
            JobCall::Complete(Ok(_)) => Ok(JobAcceptance::Accepted),
            JobCall::Complete(Err(error)) => {
                let cause = JobPublicationCause::try_new(error.clone(), access.reborrow());
                self.errors.borrow_mut().push(error);
                match cause {
                    Ok(cause) => Ok(JobAcceptance::AcceptedWithError(cause)),
                    // Preserve the fixture's recorded catchable callback-error policy.
                    Err(_catchable) => Ok(JobAcceptance::Accepted),
                }
            }
            JobCall::Deferred { job, error } => {
                self.deferred.borrow_mut().push(job.into());
                self.errors.borrow_mut().push(error);
                Ok(JobAcceptance::Accepted)
            }
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        Ok(())
    }
}
#[test]
fn inline_executor_dispatch_cannot_bypass_the_selected_job_invocation_cap() {
    let executor = Rc::new(InlineExecutor {
        errors: RefCell::new(Vec::new()),
        deferred: RefCell::new(Vec::new()),
    });
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(1),
            |mut access| {
                for attempt in 0..2 {
                    let c = calls.clone();
                    let accepted = access
                        .enqueue_job(
                            PromiseJob::new(move |_| {
                                c.set(c.get() + 1);
                                Ok(JsValue::new(42))
                            })
                            .into(),
                        )
                        .unwrap();
                    if attempt == 0 {
                        assert!(matches!(accepted, JobAcceptance::Accepted));
                    } else {
                        assert!(
                            matches!(accepted, JobAcceptance::Accepted),
                            "fallback storage accepts custody of the refused callback"
                        );
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 1 }
        ))
    );
    assert_eq!(calls.get(), 1);
    assert_counts(&outcome.report, 0, 1, 0);
    assert_eq!(executor.deferred.borrow().len(), 1);
    assert_eq!(executor.errors.borrow().len(), 1);
    assert_eq!(
        executor.errors.borrow()[0].as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 1 }
        ))
    );
}
#[test]
fn reentrant_jobs_share_owner_units_while_same_runtime_peers_keep_independent_caps() {
    let mut context = Context::default();
    let peer = Rc::new(RefCell::new(
        Context::builder()
            .runtime(context.runtime().clone())
            .build()
            .unwrap(),
    ));
    let report = Rc::new(RefCell::new(None));
    let calls = Rc::new(Cell::new(0));
    let p = peer.clone();
    let r = report.clone();
    let c = calls.clone();
    let outer = NativeJob::new(move |mut access| {
        c.set(c.get() + 1);
        complete(counted(&c).call(access.reborrow()))?;
        let outcome = p
            .borrow_mut()
            .with_execution(
                ExecutionOptions::new(0).with_job_invocation_limit(1),
                |access| complete(counted(&c).call(access)),
            )
            .unwrap();
        assert_eq!(outcome.result?, JsValue::new(42));
        *r.borrow_mut() = Some(outcome.report);
        Ok(JsValue::new(42))
    });
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(2),
            |access| complete(outer.call(access)),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_counts(&outcome.report, 0, 2, 0);
    assert_counts(report.borrow().as_ref().unwrap(), 0, 1, 0);
    assert_eq!(calls.get(), 3);
}
#[test]
fn queued_jobs_use_the_draining_operation_instead_of_capturing_enqueue_caps() {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let c = calls.clone();
    let queued = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(0)
                .with_job_poll_limit(0),
            |mut access| {
                access
                    .enqueue_job(
                        NativeAsyncJob::new(move |_| {
                            Ok(ContextTask::from_future(async move {
                                c.set(c.get() + 1);
                                Ok(JsValue::new(42))
                            }))
                        })
                        .into(),
                    )
                    .unwrap()
                    .into_result()
                    .unwrap();
                Ok(())
            },
        )
        .unwrap();
    queued.result.unwrap();
    assert_counts(&queued.report, 0, 0, 0);
    assert_eq!(calls.get(), 0);
    let drained = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(1)
                .with_job_poll_limit(1),
            |mut access| access.run_jobs(),
        )
        .unwrap();
    drained.result.unwrap();
    assert_counts(&drained.report, 1, 0, 1);
    assert_eq!(calls.get(), 1);
}
struct DropCount(Rc<Cell<usize>>);
impl Drop for DropCount {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[test]
fn unpolled_prepared_task_disposal_has_no_poll_units_and_retains_factory_effects() {
    let mut context = Context::default();
    let drops = Rc::new(Cell::new(0));
    let called = Rc::new(Cell::new(false));
    let c = called.clone();
    let probe = DropCount(drops.clone());
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_future(async move {
            let _probe = probe;
            c.set(true);
            Ok(JsValue::new(42))
        }))
    });
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let host = access.into_host_access();
            assert!(matches!(step(&mut job, &host), JobStep::Prepared));
            assert_eq!(drops.get(), 0);
            drop(job);
            assert_eq!(drops.get(), 1);
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    assert_counts(&outcome.report, 1, 0, 0);
    assert!(outcome.report.effects_started());
    assert!(!called.get());
}
#[test]
fn dropping_or_polling_pending_jobs_inside_a_real_lease_is_safe_in_every_profile_mode() {
    for mode in modes() {
        for selected in [false, true] {
            for busy in [false, true] {
                TIME.set(Duration::ZERO);
                CLOCK_READS.set(0);
                let mut context = Context::default();
                let realm = context.realm().clone();
                let drops = Rc::new(Cell::new(0));
                let probe = DropCount(drops.clone());
                let factory = move |_: ContextAccess<'_>| {
                    Ok(ContextTask::from_future(async move {
                        let _probe = probe;
                        yield_once().await;
                        Ok(JsValue::new(42))
                    }))
                };
                let mut job = if selected {
                    NativeAsyncJob::with_realm(factory, realm)
                } else {
                    NativeAsyncJob::new(factory)
                };
                let outcome = context
                    .with_execution(
                        ExecutionOptions::new(0)
                            .with_profile(mode)
                            .with_profile_clock(clock),
                        |access| {
                            let host = access.into_host_access();
                            assert!(matches!(step(&mut job, &host), JobStep::Prepared));
                            assert!(matches!(step(&mut job, &host), JobStep::Pending));
                            host.with(|_| {
                                if busy {
                                    let JobStep::Deferred(error) = step(&mut job, &host) else {
                                        panic!("a competing lease must retain the pending task")
                                    };
                                    assert_eq!(
                                        error.as_engine(),
                                        Some(&EngineError::ContextAccessBusy)
                                    );
                                    assert_eq!(
                                        drops.get(),
                                        0,
                                        "pre-action refusal keeps inner work owned"
                                    );
                                }
                                drop(job);
                                assert_eq!(drops.get(), 1);
                                Ok(())
                            })?;
                            host.with(|_| Ok(()))?;
                            Ok(42_u32)
                        },
                    )
                    .unwrap();
                assert_eq!(outcome.result.unwrap(), 42);
                assert_counts(&outcome.report, 1, 0, 1);
                assert!(outcome.report.effects_started());
                if !matches!(mode, ProfileMode::Off) {
                    assert_phase(
                        &outcome.report,
                        "job-prepare",
                        1,
                        timed(mode).then_some(Duration::ZERO),
                    );
                    assert_phase(
                        &outcome.report,
                        "job-poll",
                        1,
                        timed(mode).then_some(Duration::ZERO),
                    );
                } else {
                    assert!(outcome.report.profile().unwrap().is_none());
                }
                if !timed(mode) {
                    assert_eq!(CLOCK_READS.get(), 0);
                }
            }
        }
    }
}
#[test]
fn action_timing_excludes_host_gaps_while_the_execution_lifetime_includes_them() {
    for mode in modes() {
        TIME.set(Duration::ZERO);
        CLOCK_READS.set(0);
        let mut context = Context::default();
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                |mut access| {
                    complete(
                        NativeJob::new(|_| {
                            TIME.set(Duration::from_millis(2));
                            Ok(JsValue::new(42))
                        })
                        .call(access.reborrow()),
                    )?;
                    let host = access.into_host_access();
                    let mut job = NativeAsyncJob::new(|_| {
                        Ok(ContextTask::from_future(async {
                            TIME.set(Duration::from_millis(5));
                            yield_once().await;
                            TIME.set(Duration::from_millis(110));
                            Ok(JsValue::new(42))
                        }))
                    });
                    assert!(matches!(step(&mut job, &host), JobStep::Prepared));
                    assert!(matches!(step(&mut job, &host), JobStep::Pending));
                    TIME.set(Duration::from_millis(105));
                    let JobStep::Complete(result) = step(&mut job, &host) else {
                        panic!("the second poll must complete")
                    };
                    assert_eq!(result?, JsValue::new(42));
                    Ok(42_u32)
                },
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), 42);
        assert_counts(&outcome.report, 1, 1, 2);
        if !matches!(mode, ProfileMode::Off) {
            let profile = outcome.report.profile().unwrap().unwrap();
            assert_eq!(
                profile.elapsed,
                timed(mode).then_some(Duration::from_millis(110))
            );
            assert_phase(
                &outcome.report,
                "execution-lifetime",
                1,
                timed(mode).then_some(Duration::from_millis(110)),
            );
            assert_phase(
                &outcome.report,
                "job-prepare",
                1,
                timed(mode).then_some(Duration::ZERO),
            );
            assert_phase(
                &outcome.report,
                "job-invoke",
                1,
                timed(mode).then_some(Duration::from_millis(2)),
            );
            assert_phase(
                &outcome.report,
                "job-poll",
                2,
                timed(mode).then_some(Duration::from_millis(8)),
            );
            assert!(
                !profile
                    .scopes
                    .iter()
                    .any(|scope| scope.name == "job-waiting")
            );
            if let ProfileMode::Detailed { event_limit, .. } = mode {
                assert!(profile.events.len() <= event_limit);
                if event_limit == 1 {
                    assert_eq!(profile.events.len(), 1);
                    assert!(profile.dropped_events >= 1);
                    assert_eq!(outcome.report.job_polls().unwrap(), 2);
                }
            }
        } else {
            assert!(outcome.report.profile().unwrap().is_none());
        }
        if !timed(mode) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}
struct DisposalProbe {
    receipt: Rc<RefCell<Option<ExecutionReceipt>>>,
    disposed: Rc<Cell<bool>>,
    unpublished: Rc<Cell<bool>>,
}
impl Drop for DisposalProbe {
    fn drop(&mut self) {
        self.disposed.set(true);
        self.unpublished
            .set(self.receipt.borrow().as_ref().unwrap().report().is_none());
        TIME.set(Duration::from_millis(7));
    }
}
#[test]
fn a_stop_during_ready_or_pending_poll_disposes_inner_work_before_phase_close_and_receipt() {
    for mode in modes() {
        for returns_pending in [false, true] {
            TIME.set(Duration::ZERO);
            CLOCK_READS.set(0);
            let mut context = Context::default();
            let slot = Rc::new(RefCell::new(None));
            let disposed = Rc::new(Cell::new(false));
            let unpublished = Rc::new(Cell::new(false));
            let actual = Rc::new(RefCell::new(None));
            let record = actual.clone();
            let probe = DisposalProbe {
                receipt: slot.clone(),
                disposed: disposed.clone(),
                unpublished: unpublished.clone(),
            };
            let mut task = context
                .execution_task(
                    ExecutionOptions::new(0)
                        .with_job_invocation_limit(0)
                        .with_profile(mode)
                        .with_profile_clock(clock),
                    async move |access| {
                        let host = access.into_host_access();
                        let mut job = NativeAsyncJob::new(move |_| {
                            Ok(ContextTask::from_poll(move |_, host| {
                                let _keep = &probe;
                                TIME.set(Duration::from_millis(2));
                                let error = host
                                    .with(|access| {
                                        let (_uncalled, error) = deferred(
                                            NativeJob::new(|_| {
                                                panic!("a refused callback must not execute")
                                            })
                                            .call(access),
                                        );
                                        Err::<(), _>(error)
                                    })
                                    .unwrap_err();
                                assert_eq!(
                                    error.as_engine(),
                                    Some(&EngineError::RuntimeLimit(
                                        RuntimeLimitError::JobInvocation { limit: 0 }
                                    ))
                                );
                                if returns_pending {
                                    Poll::Pending
                                } else {
                                    Poll::Ready(Ok(JsValue::new(42)))
                                }
                            }))
                        });
                        assert!(matches!(step(&mut job, &host), JobStep::Prepared));
                        let JobStep::Complete(result) = step(&mut job, &host) else {
                            panic!("an admitted task stop must complete immediately")
                        };
                        *record.borrow_mut() = Some(result.clone());
                        result
                    },
                )
                .unwrap();
            let receipt = task.receipt();
            *slot.borrow_mut() = Some(receipt.clone());
            let Poll::Ready(outcome) = poll(&mut task) else {
                panic!("a retained stop must override Pending")
            };
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobInvocation { limit: 0 }
                ))
            );
            let actual = actual.borrow();
            let actual = actual
                .as_ref()
                .expect("the job returned an actual terminal result");
            assert_eq!(
                actual.as_ref().err().and_then(|e| e.as_engine()),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobInvocation { limit: 0 }
                ))
            );
            assert!(disposed.get());
            assert!(unpublished.get());
            assert!(receipt.report().is_some());
            assert_counts(&outcome.report, 1, 0, 1);
            assert!(outcome.report.effects_started());
            if !matches!(mode, ProfileMode::Off) {
                assert_phase(
                    &outcome.report,
                    "job-poll",
                    1,
                    timed(mode).then_some(Duration::from_millis(7)),
                );
            }
            if !timed(mode) {
                assert_eq!(CLOCK_READS.get(), 0);
            }
            drop(task);
        }
    }
}
#[test]
fn admitted_native_unwind_retains_its_unit_and_closes_its_action_before_abandonment() {
    for mode in modes() {
        TIME.set(Duration::ZERO);
        CLOCK_READS.set(0);
        let mut context = Context::default();
        let mut task = context
            .execution_task(
                ExecutionOptions::new(0)
                    .with_profile(mode)
                    .with_profile_clock(clock),
                async |access| {
                    complete(
                        NativeJob::new(|_| {
                            TIME.set(Duration::from_millis(3));
                            panic!("admitted native job unwind")
                        })
                        .call(access),
                    )
                },
            )
            .unwrap();
        let receipt = task.receipt();
        let panic = catch_unwind(AssertUnwindSafe(|| poll(&mut task))).unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"admitted native job unwind")
        );
        drop(task);
        let report = receipt.report().unwrap();
        assert_counts(&report, 0, 1, 0);
        assert!(report.effects_started());
        assert!(matches!(
            report.termination(),
            ExecutionTermination::Abandoned { first_stop: None }
        ));
        if !matches!(mode, ProfileMode::Off) {
            assert_phase(
                &report,
                "job-invoke",
                1,
                timed(mode).then_some(Duration::from_millis(3)),
            );
        }
        if !timed(mode) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}
#[test]
fn unscoped_job_calls_do_not_retain_a_previous_operations_profiling_clock() {
    TIME.set(Duration::ZERO);
    CLOCK_READS.set(0);
    let mut context = Context::default();
    context
        .with_execution(
            ExecutionOptions::new(0)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
            |_| Ok(()),
        )
        .unwrap()
        .result
        .unwrap();
    CLOCK_READS.set(0);
    let calls = Rc::new(Cell::new(0));
    assert_eq!(
        complete(counted(&calls).call(context.access())).unwrap(),
        JsValue::new(42)
    );
    let host = context.host_access();
    let mut job = NativeAsyncJob::new(|_| {
        Ok(ContextTask::from_future(async {
            yield_once().await;
            Ok(JsValue::new(42))
        }))
    });
    assert!(matches!(step(&mut job, &host), JobStep::Prepared));
    assert!(matches!(step(&mut job, &host), JobStep::Pending));
    let JobStep::Complete(result) = step(&mut job, &host) else {
        panic!("the second poll must complete")
    };
    assert_eq!(result.unwrap(), JsValue::new(42));
    assert_eq!(calls.get(), 1);
    assert_eq!(CLOCK_READS.get(), 0);
}
