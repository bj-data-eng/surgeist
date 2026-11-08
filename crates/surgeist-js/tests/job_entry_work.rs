#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    future::{Future, pending},
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
        GenericJob, IntervalJob, Job, JobExecutor, NativeAsyncJob, NativeJob, NativeJobFn,
        PromiseJob, SimpleJobExecutor, TimeoutJob,
    },
};

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static CLOCK_READS: Cell<usize> = const { Cell::new(0) };
}

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

fn assert_counts(report: &ExecutionReport, preparations: u64, invocations: u64, polls: u64) {
    assert_eq!(report.job_preparations(), preparations);
    assert_eq!(report.job_invocations(), invocations);
    assert_eq!(report.job_polls(), polls);
    assert_eq!(
        report.instructions(),
        0,
        "pure Rust job actions consume no VM instructions"
    );
}

fn assert_phase(report: &ExecutionReport, name: &str, phases: u64, elapsed: Option<Duration>) {
    let phase = report
        .profile()
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

#[test]
fn preparation_zero_exact_and_one_short_caps_count_only_created_inner_futures() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let called = Rc::new(Cell::new(0));
        let outcome = context.with_execution(
            ExecutionOptions::new(0).with_job_preparation_limit(limit),
            |access| {
                let holder = access.into_host_access();
                for index in 0..2 {
                    let called = called.clone();
                    let mut future = NativeAsyncJob::new(async move |_| {
                        called.set(called.get() + 1);
                        Ok(JsValue::new(42))
                    }).call(&holder);
                    if index >= limit {
                        let Poll::Ready(Err(error)) = poll(&mut future) else {
                            panic!("refused preparation must expose its existing error without polling inner work")
                        };
                        assert_eq!(error.as_engine(), Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPreparation { limit })));
                    }
                    drop(future);
                }
                Ok(())
            },
        ).unwrap();
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
        assert_eq!(called.get(), 0);
        assert!(
            !outcome.report.effects_started(),
            "preparation without polling is not a program callback"
        );
    }
}

#[test]
fn invocation_zero_exact_and_one_short_caps_refuse_before_reusable_callbacks() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let called = Rc::new(Cell::new(0));
        let job = reusable_counted(&called);
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_job_invocation_limit(limit),
                |mut access| {
                    for index in 0..2 {
                        let result = job.call(access.reborrow());
                        if index >= limit {
                            assert_eq!(
                                result.unwrap_err().as_engine(),
                                Some(&EngineError::RuntimeLimit(
                                    RuntimeLimitError::JobInvocation { limit }
                                ))
                            );
                        } else {
                            assert_eq!(result?, JsValue::new(42));
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
        assert_eq!(called.get(), limit.min(2));
        assert_eq!(outcome.report.effects_started(), limit != 0);
    }
}

#[test]
fn poll_zero_exact_and_one_short_caps_count_pending_and_ready_separately() {
    for limit in [0, 1, 2] {
        let mut context = Context::default();
        let entries = Rc::new(Cell::new(0));
        let callback_entries = entries.clone();
        let job = NativeAsyncJob::new(async move |_| {
            callback_entries.set(callback_entries.get() + 1);
            yield_once().await;
            callback_entries.set(callback_entries.get() + 1);
            Ok(JsValue::new(42))
        });
        let outcome = context
            .with_execution(
                ExecutionOptions::new(0).with_job_poll_limit(limit),
                |access| {
                    let holder = access.into_host_access();
                    let mut future = job.call(&holder);
                    if limit != 0 {
                        assert!(poll(&mut future).is_pending());
                    }
                    let result = poll(&mut future);
                    if limit < 2 {
                        let Poll::Ready(Err(error)) = result else {
                            panic!("the refused poll must return its typed stop")
                        };
                        assert_eq!(
                            error.as_engine(),
                            Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
                                limit
                            }))
                        );
                    } else {
                        assert_eq!(result, Poll::Ready(Ok(JsValue::new(42))));
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
        assert_eq!(outcome.report.effects_started(), limit != 0);
    }
}

#[test]
fn every_wrapper_delegates_to_exactly_one_synchronous_invocation_owner() {
    let mut context = Context::default();
    let realm = context.realm().clone();
    let calls = Rc::new(Cell::new(0));
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            counted(&calls).call(access.reborrow())?;
            let reusable = reusable_counted(&calls);
            reusable.call(access.reborrow())?;
            reusable.call(access.reborrow())?;
            let promise_calls = calls.clone();
            PromiseJob::new(move |_| {
                promise_calls.set(promise_calls.get() + 1);
                Ok(JsValue::new(42))
            })
            .call(access.reborrow())?;
            let generic_calls = calls.clone();
            GenericJob::new(
                move |_| {
                    generic_calls.set(generic_calls.get() + 1);
                    Ok(JsValue::new(42))
                },
                realm,
            )
            .call(access.reborrow())?;
            TimeoutJob::new(counted(&calls), 0).call(access.reborrow())?;
            let interval = IntervalJob::new(reusable_counted(&calls), 0);
            interval.call(access.reborrow())?;
            interval.call(access.reborrow())?;
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
    let source = "40 + 2";
    let control = Context::default()
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    assert!(control.report.instructions() > 0);
    let mut context = Context::default();
    let outcome = context
        .with_execution(
            ExecutionOptions::new(64).with_job_invocation_limit(1),
            |access| {
                NativeJob::new(|mut access| access.eval(Source::from_bytes("40 + 2"))).call(access)
            },
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(outcome.report.job_preparations(), 0);
    assert_eq!(outcome.report.job_invocations(), 1);
    assert_eq!(outcome.report.job_polls(), 0);
    assert_eq!(outcome.report.instructions(), control.report.instructions());
}

#[test]
fn reentry_preserves_the_first_invocation_stop_over_success_and_later_job_dimensions() {
    let mut context = Context::default();
    let nested_calls = Rc::new(Cell::new(0));
    let denied = nested_calls.clone();
    let outer = NativeJob::new(move |mut access| {
        let error = counted(&denied).call(access.reborrow()).unwrap_err();
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
                let error = outer.call(access.reborrow()).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobInvocation { limit: 1 }
                    ))
                );
                let holder = access.into_host_access();
                let mut future = NativeAsyncJob::new(async |_| Ok(JsValue::new(42))).call(&holder);
                let Poll::Ready(Err(error)) = poll(&mut future) else {
                    panic!("the retained invocation stop must prevent later preparation")
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
    assert_eq!(nested_calls.get(), 0);
}

#[test]
fn availability_realm_validation_and_preexisting_stops_do_not_charge_new_job_units() {
    let foreign = Context::default();
    let foreign_realm = foreign.realm().clone();
    let mut context = Context::default();
    let calls = Rc::new(Cell::new(0));
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(1)
                .with_job_invocation_limit(1)
                .with_job_poll_limit(1),
            |access| {
                let holder = access.into_host_access();
                holder.with(|_| {
                    let called = calls.clone();
                    let mut future = NativeAsyncJob::new(async move |_| {
                        called.set(called.get() + 1);
                        Ok(JsValue::new(42))
                    })
                    .call(&holder);
                    let Poll::Ready(Err(error)) = poll(&mut future) else {
                        panic!("a real competing lease must return Busy")
                    };
                    assert_eq!(error.as_engine(), Some(&EngineError::ContextAccessBusy));
                    Ok(())
                })?;
                let called = calls.clone();
                let mut invalid = NativeAsyncJob::with_realm(
                    async move |_| {
                        called.set(called.get() + 1);
                        Ok(JsValue::new(42))
                    },
                    foreign_realm.clone(),
                )
                .call(&holder);
                let Poll::Ready(Err(error)) = poll(&mut invalid) else {
                    panic!("a foreign realm must refuse async entry")
                };
                assert!(error.as_native().is_some_and(|error| error.is_type()));
                drop(invalid);
                holder.with(|access| {
                    let called = calls.clone();
                    let error = NativeJob::with_realm(
                        move |_| {
                            called.set(called.get() + 1);
                            Ok(JsValue::new(42))
                        },
                        foreign_realm,
                    )
                    .call(access)
                    .unwrap_err();
                    assert!(error.as_native().is_some_and(|error| error.is_type()));
                    Ok(())
                })?;
                let called = calls.clone();
                let mut valid = NativeAsyncJob::new(async move |_| {
                    called.set(called.get() + 1);
                    Ok(JsValue::new(42))
                })
                .call(&holder);
                assert_eq!(poll(&mut valid), Poll::Ready(Ok(JsValue::new(42))));
                drop(valid);
                holder.with(|access| counted(&calls).call(access))?;
                let first = holder
                    .with(|mut access| access.eval(Source::from_bytes("0")))
                    .unwrap_err();
                assert_eq!(
                    first.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 0
                    }))
                );
                let mut stopped = NativeAsyncJob::new(async |_| Ok(JsValue::new(42))).call(&holder);
                let Poll::Ready(Err(error)) = poll(&mut stopped) else {
                    panic!("retained stop must precede exhausted job caps")
                };
                assert_eq!(error.as_engine(), first.as_engine());
                let error = holder
                    .with(|access| counted(&calls).call(access))
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
}

impl JobExecutor for InlineExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, access: ContextAccess<'_>) {
        let Job::PromiseJob(job) = job else {
            panic!("this fixture accepts only its authored promise jobs")
        };
        if let Err(error) = job.call(access) {
            self.errors.borrow_mut().push(error);
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
                for _ in 0..2 {
                    let called = calls.clone();
                    access.enqueue_job(
                        PromiseJob::new(move |_| {
                            called.set(called.get() + 1);
                            Ok(JsValue::new(42))
                        })
                        .into(),
                    );
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
    let errors = executor.errors.borrow();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].as_engine(),
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
    let peer_report = Rc::new(RefCell::new(None));
    let calls = Rc::new(Cell::new(0));
    let callback_peer = peer.clone();
    let callback_report = peer_report.clone();
    let callback_calls = calls.clone();
    let outer = NativeJob::new(move |mut access| {
        callback_calls.set(callback_calls.get() + 1);
        counted(&callback_calls).call(access.reborrow())?;
        let peer_outcome = callback_peer
            .borrow_mut()
            .with_execution(
                ExecutionOptions::new(0).with_job_invocation_limit(1),
                |access| counted(&callback_calls).call(access),
            )
            .unwrap();
        assert_eq!(peer_outcome.result?, JsValue::new(42));
        *callback_report.borrow_mut() = Some(peer_outcome.report);
        Ok(JsValue::new(42))
    });
    let outcome = context
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(2),
            |access| outer.call(access),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_counts(&outcome.report, 0, 2, 0);
    assert_counts(peer_report.borrow().as_ref().unwrap(), 0, 1, 0);
    assert_eq!(calls.get(), 3);
}

#[test]
fn queued_jobs_use_the_draining_operation_instead_of_capturing_enqueue_caps() {
    let mut context = Context::builder()
        .job_executor(Rc::new(SimpleJobExecutor::new()))
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let callback_calls = calls.clone();
    let queued = context
        .with_execution(
            ExecutionOptions::new(0)
                .with_job_preparation_limit(0)
                .with_job_poll_limit(0),
            |mut access| {
                access.enqueue_job(
                    NativeAsyncJob::new(async move |_| {
                        callback_calls.set(callback_calls.get() + 1);
                        Ok(JsValue::new(42))
                    })
                    .into(),
                );
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
fn unpolled_prepared_future_disposal_has_no_poll_units_or_possible_program_effects() {
    let mut context = Context::default();
    let drops = Rc::new(Cell::new(0));
    let called = Rc::new(Cell::new(false));
    let callback_called = called.clone();
    let probe = DropCount(drops.clone());
    let job = NativeAsyncJob::new(async move |_| {
        let _probe = probe;
        callback_called.set(true);
        Ok(JsValue::new(42))
    });
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            let future = job.call(&holder);
            assert_eq!(drops.get(), 0);
            drop(future);
            assert_eq!(drops.get(), 1);
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    assert_counts(&outcome.report, 1, 0, 0);
    assert!(!outcome.report.effects_started());
    assert!(!called.get());
}

#[test]
fn dropping_or_polling_pending_jobs_inside_a_real_lease_is_safe_in_every_profile_mode() {
    for mode in modes() {
        for selected_realm in [false, true] {
            for poll_while_busy in [false, true] {
                TIME.set(Duration::ZERO);
                CLOCK_READS.set(0);
                let mut context = Context::default();
                let realm = context.realm().clone();
                let drops = Rc::new(Cell::new(0));
                let probe = DropCount(drops.clone());
                let job = if selected_realm {
                    NativeAsyncJob::with_realm(
                        async move |_| {
                            let _probe = probe;
                            yield_once().await;
                            Ok(JsValue::new(42))
                        },
                        realm,
                    )
                } else {
                    NativeAsyncJob::new(async move |_| {
                        let _probe = probe;
                        yield_once().await;
                        Ok(JsValue::new(42))
                    })
                };
                let outcome = context
                    .with_execution(
                        ExecutionOptions::new(0)
                            .with_profile(mode)
                            .with_profile_clock(clock),
                        |access| {
                            let holder = access.into_host_access();
                            let mut future = job.call(&holder);
                            assert!(poll(&mut future).is_pending());
                            holder.with(|_| {
                                if poll_while_busy {
                                    let Poll::Ready(Err(error)) = poll(&mut future) else {
                                        panic!(
                                            "a genuine competing lease must reject the next poll"
                                        )
                                    };
                                    assert_eq!(
                                        error.as_engine(),
                                        Some(&EngineError::ContextAccessBusy)
                                    );
                                    assert_eq!(
                                        drops.get(),
                                        1,
                                        "terminal Busy must dispose inner work before returning"
                                    );
                                }
                                drop(future);
                                assert_eq!(drops.get(), 1);
                                Ok(())
                            })?;
                            // Availability refusal does not become a retained work stop.
                            holder.with(|_| Ok(()))?;
                            Ok(42_u32)
                        },
                    )
                    .unwrap();
                assert_eq!(outcome.result.unwrap(), 42);
                assert_counts(&outcome.report, 1, 0, 1);
                assert!(outcome.report.effects_started());
                if matches!(mode, ProfileMode::Off) {
                    assert!(outcome.report.profile().is_none());
                } else {
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
                    NativeJob::new(|_| {
                        TIME.set(Duration::from_millis(2));
                        Ok(JsValue::new(42))
                    })
                    .call(access.reborrow())?;
                    let holder = access.into_host_access();
                    let mut future = NativeAsyncJob::new(async |_| {
                        TIME.set(Duration::from_millis(5));
                        yield_once().await;
                        TIME.set(Duration::from_millis(110));
                        Ok(JsValue::new(42))
                    })
                    .call(&holder);
                    assert!(poll(&mut future).is_pending());
                    TIME.set(Duration::from_millis(105));
                    assert_eq!(poll(&mut future), Poll::Ready(Ok(JsValue::new(42))));
                    Ok(42_u32)
                },
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), 42);
        assert_counts(&outcome.report, 1, 1, 2);
        if matches!(mode, ProfileMode::Off) {
            assert!(outcome.report.profile().is_none());
        } else {
            let profile = outcome.report.profile().unwrap();
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
                    // Detail loss cannot reduce either accepted poll units or the
                    // two authoritative action observations above.
                    assert_eq!(outcome.report.job_polls(), 2);
                }
            }
        }
        if !timed(mode) {
            assert_eq!(CLOCK_READS.get(), 0);
        }
    }
}

struct DisposalProbe {
    receipt: Rc<RefCell<Option<ExecutionReceipt>>>,
    disposed: Rc<Cell<bool>>,
    unpublished_at_drop: Rc<Cell<bool>>,
}

impl Drop for DisposalProbe {
    fn drop(&mut self) {
        self.disposed.set(true);
        self.unpublished_at_drop
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
            let receipt_slot = Rc::new(RefCell::new(None));
            let disposed = Rc::new(Cell::new(false));
            let unpublished_at_drop = Rc::new(Cell::new(false));
            let actual_job_result = Rc::new(RefCell::new(None));
            let callback_result = actual_job_result.clone();
            let probe = DisposalProbe {
                receipt: receipt_slot.clone(),
                disposed: disposed.clone(),
                unpublished_at_drop: unpublished_at_drop.clone(),
            };
            let mut task = context
                .execution_task(
                    ExecutionOptions::new(0)
                        .with_job_invocation_limit(0)
                        .with_profile(mode)
                        .with_profile_clock(clock),
                    async move |access| {
                        let holder = access.into_host_access();
                        let job = NativeAsyncJob::new(async move |host| {
                            let _probe = probe;
                            TIME.set(Duration::from_millis(2));
                            let error = host
                                .with(|access| {
                                    NativeJob::new(|_| panic!("refused invocation must not run"))
                                        .call(access)
                                })
                                .unwrap_err();
                            assert_eq!(
                                error.as_engine(),
                                Some(&EngineError::RuntimeLimit(
                                    RuntimeLimitError::JobInvocation { limit: 0 }
                                ))
                            );
                            if returns_pending {
                                pending::<()>().await;
                            }
                            Ok(JsValue::new(42))
                        });
                        let result = job.call(&holder).await;
                        *callback_result.borrow_mut() = Some(result.clone());
                        result
                    },
                )
                .unwrap();
            let receipt = task.receipt();
            *receipt_slot.borrow_mut() = Some(receipt.clone());
            let Poll::Ready(outcome) = poll(&mut task) else {
                panic!("a newly retained stop must override even Pending immediately")
            };
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobInvocation { limit: 0 }
                ))
            );
            let actual = actual_job_result.borrow();
            let actual = actual
                .as_ref()
                .expect("the job must return a terminal result");
            assert_eq!(
                actual.as_ref().err().and_then(|error| error.as_engine()),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::JobInvocation { limit: 0 }
                )),
                "the job itself must preserve the first stop before outer finalization: {actual:?}"
            );
            assert!(disposed.get());
            assert!(unpublished_at_drop.get());
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
                    NativeJob::new(|_| {
                        TIME.set(Duration::from_millis(3));
                        panic!("admitted native job unwind")
                    })
                    .call(access)
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
        counted(&calls).call(context.access()).unwrap(),
        JsValue::new(42)
    );
    let holder = context.host_access();
    let mut future = NativeAsyncJob::new(async |_| {
        yield_once().await;
        Ok(JsValue::new(42))
    })
    .call(&holder);
    assert!(poll(&mut future).is_pending());
    assert_eq!(poll(&mut future), Poll::Ready(Ok(JsValue::new(42))));
    assert_eq!(calls.get(), 1);
    assert_eq!(CLOCK_READS.get(), 0);
}
