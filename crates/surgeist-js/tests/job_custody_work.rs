#![allow(unused_crate_dependencies)]
use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
};
use surgeist_js::{
    Context, HostContext, JsError, JsValue, Source,
    context::time::FixedClock,
    error::{EngineError, ErasedEngineError, JsErasedError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{
        ContextId, ContextTask, IntervalJob, JobCall, JobExecutor, JobObserver, JobRetirement,
        JobStatus, JobStep, NativeAsyncJob, NativeJob, NativeJobFn, PromiseJob, SimpleJobExecutor,
        TimeoutJob,
    },
};
fn step(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> JobStep {
    job.run_step(&mut TaskContext::from_waker(Waker::noop()), host)
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn prepared(job: &mut NativeAsyncJob, host: &HostContext<'_>) {
    assert!(matches!(step(job, host), JobStep::Prepared));
}
fn completion(job: &mut NativeAsyncJob, host: &HostContext<'_>) -> surgeist_js::JsResult<JsValue> {
    match step(job, host) {
        JobStep::Complete(result) => result,
        _ => panic!("an authored terminal step must deliver Complete"),
    }
}
fn ready() -> ContextTask<JsValue> {
    ContextTask::from_future(async { Ok(JsValue::new(42)) })
}
struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn pending_job(drops: Rc<Cell<usize>>, polls: Rc<Cell<usize>>) -> NativeAsyncJob {
    let probe = DropProbe(drops);
    NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, _| {
            let _keep = &probe;
            polls.set(polls.get() + 1);
            Poll::Pending
        }))
    })
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
fn pending_wait(observer: &JobObserver) -> (Pin<Box<surgeist_js::job::JobWait>>, Arc<WakeCount>) {
    let mut wait = Box::pin(observer.wait().unwrap());
    let wakes = Arc::new(WakeCount::default());
    let waker = Waker::from(wakes.clone());
    assert!(
        wait.as_mut()
            .poll(&mut TaskContext::from_waker(&waker))
            .is_pending()
    );
    (wait, wakes)
}

#[test]
fn deferred_preparation_is_unbound_and_only_a_later_admitted_factory_selects_the_owner() {
    let factories = Rc::new(Cell::new(0));
    let f = factories.clone();
    let mut job = NativeAsyncJob::new(move |_| {
        f.set(f.get() + 1);
        Ok(ready())
    });
    let observer = job.observer();
    let mut a = Context::default();
    let refused = a
        .with_execution(
            ExecutionOptions::new(0).with_job_preparation_limit(0),
            |access| {
                let host = access.into_host_access();
                let JobStep::Deferred(error) = step(&mut job, &host) else {
                    panic!("a zero preparation cap must preserve the unbound request")
                };
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobPreparation { limit: 0 }
                    ))
                );
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(refused.report.job_preparations().unwrap(), 0);
    assert_eq!(factories.get(), 0);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    let mut b = Context::builder()
        .runtime(a.runtime().clone())
        .build()
        .unwrap();
    let required = b.id();
    let admitted = b
        .with_execution(ExecutionOptions::new(0), |access| {
            prepared(&mut job, &access.into_host_access());
            Ok(())
        })
        .unwrap();
    admitted.result.unwrap();
    assert_eq!(admitted.report.job_preparations().unwrap(), 1);
    assert!(admitted.report.effects_started());
    assert_eq!(factories.get(), 1);
    let wrong = a
        .with_execution(ExecutionOptions::new(0), |access| {
            let host = access.into_host_access();
            let JobStep::Deferred(error) = step(&mut job, &host) else {
                panic!("the first admitted owner must remain required")
            };
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::JobContextRequired {
                    required: required.clone()
                })
            );
            Ok(())
        })
        .unwrap();
    wrong.result.unwrap();
    assert_eq!(wrong.report.job_polls().unwrap(), 0);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    {
        let host = b.host_access();
        assert_eq!(completion(&mut job, &host).unwrap(), JsValue::new(42));
        assert!(matches!(
            step(&mut job, &host),
            JobStep::Finished(Ok(JobStatus::Completed))
        ));
    }
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(factories.get(), 1);
}
#[test]
fn a_deferred_poll_keeps_the_exact_task_and_charges_only_its_later_operation() {
    let drops = Rc::new(Cell::new(0));
    let polls = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let p = polls.clone();
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, _| {
            let _keep = &probe;
            p.set(p.get() + 1);
            Poll::Ready(Ok(JsValue::new(42)))
        }))
    });
    let observer = job.observer();
    let mut owner = Context::default();
    let first = owner
        .with_execution(ExecutionOptions::new(0).with_job_poll_limit(0), |access| {
            let host = access.into_host_access();
            prepared(&mut job, &host);
            let JobStep::Deferred(error) = step(&mut job, &host) else {
                panic!("a zero poll cap must retain prepared work")
            };
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
                    limit: 0
                }))
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(
        first.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::JobPoll {
            limit: 0
        }))
    );
    assert_eq!(first.report.job_preparations().unwrap(), 1);
    assert_eq!(first.report.job_polls().unwrap(), 0);
    assert_eq!(polls.get(), 0);
    assert_eq!(drops.get(), 0);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    let second = owner
        .with_execution(ExecutionOptions::new(0).with_job_poll_limit(1), |access| {
            completion(&mut job, &access.into_host_access())
        })
        .unwrap();
    assert_eq!(second.result.unwrap(), JsValue::new(42));
    assert_eq!(second.report.job_preparations().unwrap(), 0);
    assert_eq!(second.report.job_polls().unwrap(), 1);
    assert_eq!(first.report.job_polls().unwrap(), 0);
    assert_eq!(polls.get(), 1);
    assert_eq!(drops.get(), 1);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
}
#[test]
fn admitted_interruption_is_terminal_and_finished_steps_do_no_new_work() {
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, host| {
            let _keep = &probe;
            let error = host
                .with(|mut access| access.eval(Source::from_bytes("0")))
                .unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            Poll::Pending
        }))
    });
    let observer = job.observer();
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut owner = builder.build().unwrap();
    let first = owner
        .with_execution(ExecutionOptions::new(0), |access| {
            let host = access.into_host_access();
            prepared(&mut job, &host);
            let error = completion(&mut job, &host).unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(first.report.job_preparations().unwrap(), 1);
    assert_eq!(first.report.job_polls().unwrap(), 1);
    assert_eq!(drops.get(), 1);
    assert_eq!(
        observer.status().unwrap(),
        JobStatus::Retired(JobRetirement::Interrupted)
    );
    let next = owner
        .with_execution(ExecutionOptions::new(0), |access| {
            assert!(matches!(
                step(&mut job, &access.into_host_access()),
                JobStep::Finished(Ok(JobStatus::Retired(JobRetirement::Interrupted)))
            ));
            Ok(())
        })
        .unwrap();
    next.result.unwrap();
    assert_eq!(next.report.job_preparations().unwrap(), 0);
    assert_eq!(next.report.job_polls().unwrap(), 0);
    assert!(!next.report.effects_started());
    assert_eq!(drops.get(), 1);
}
#[test]
fn idle_moves_and_new_holders_keep_affinity_while_a_replacement_owner_does_not() {
    let mut owner = Context::default();
    let required = owner.id();
    let expected = required.clone();
    let polls = Rc::new(Cell::new(0));
    let p = polls.clone();
    let mut job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, host| {
            host.with(|access| {
                assert_eq!(access.id(), expected);
                Ok(())
            })?;
            p.set(p.get() + 1);
            if p.get() == 1 {
                Poll::Pending
            } else {
                Poll::Ready(Ok(JsValue::new(42)))
            }
        }))
    });
    {
        let host = owner.host_access();
        prepared(&mut job, &host);
        assert!(matches!(step(&mut job, &host), JobStep::Pending));
    }
    let mut moved = Box::new(owner);
    assert_eq!(moved.id(), required);
    let mut original = std::mem::take(&mut *moved);
    assert_ne!(moved.id(), required);
    {
        let host = moved.host_access();
        let JobStep::Deferred(error) = step(&mut job, &host) else {
            panic!("replacement construction must not acquire old task affinity")
        };
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::JobContextRequired {
                required: required.clone()
            })
        );
    }
    assert_eq!(polls.get(), 1);
    {
        let host = original.host_access();
        assert_eq!(completion(&mut job, &host).unwrap(), JsValue::new(42));
    }
    assert_eq!(polls.get(), 2);
}
#[test]
fn owner_drop_retires_prepared_and_pending_tasks_and_wakes_native_waiters() {
    for started in [false, true] {
        let drops = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let mut job = pending_job(drops.clone(), polls.clone());
        let observer = job.observer();
        let (mut wait, wakes) = pending_wait(&observer);
        let mut owner = Context::default();
        {
            let host = owner.host_access();
            prepared(&mut job, &host);
            if started {
                assert!(matches!(step(&mut job, &host), JobStep::Pending));
            }
        }
        drop(owner);
        assert!(wakes.0.load(Ordering::Relaxed) > 0);
        assert_eq!(
            observer.status().unwrap(),
            JobStatus::Retired(JobRetirement::OwnerDropped)
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(
            poll(&mut wait).map(Result::unwrap),
            Poll::Ready(JobStatus::Retired(JobRetirement::OwnerDropped))
        );
        let mut other = Context::default();
        let host = other.host_access();
        let error = completion(&mut job, &host).unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::JobRetired(JobRetirement::OwnerDropped))
        );
        assert!(matches!(
            step(&mut job, &host),
            JobStep::Finished(Ok(JobStatus::Retired(JobRetirement::OwnerDropped)))
        ));
        assert_eq!(polls.get(), usize::from(started));
    }
}
#[test]
fn final_direct_drop_retires_unbound_prepared_and_pending_controls_once() {
    for stage in 0..3 {
        let drops = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let mut job = pending_job(drops.clone(), polls.clone());
        let observer = job.observer();
        let mut wait = Box::pin(observer.wait().unwrap());
        let mut owner = Context::default();
        if stage > 0 {
            let host = owner.host_access();
            prepared(&mut job, &host);
            if stage == 2 {
                assert!(matches!(step(&mut job, &host), JobStep::Pending));
            }
        }
        drop(job);
        assert_eq!(
            observer.status().unwrap(),
            JobStatus::Retired(JobRetirement::Dropped)
        );
        assert_eq!(
            poll(&mut wait).map(Result::unwrap),
            Poll::Ready(JobStatus::Retired(JobRetirement::Dropped))
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(polls.get(), usize::from(stage == 2));
        drop(owner);
        assert_eq!(
            observer.status().unwrap(),
            JobStatus::Retired(JobRetirement::Dropped)
        );
    }
}
#[test]
fn required_owner_errors_preserve_the_inert_identity_through_erasure() {
    fn send_sync_clone<T: Send + Sync + Clone>() {}
    send_sync_clone::<ContextId>();
    send_sync_clone::<JobObserver>();
    send_sync_clone::<JsErasedError>();
    let mut owner = Context::default();
    let required = owner.id();
    let mut other = Context::default();
    let mut job = NativeAsyncJob::new(|_| Ok(ready()));
    prepared(&mut job, &owner.host_access());
    let JobStep::Deferred(error) = step(&mut job, &other.host_access()) else {
        panic!("a different live context must report the required owner")
    };
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobContextRequired {
            required: required.clone()
        })
    );
    let materialization = error.clone().into_opaque(other.access()).unwrap_err();
    assert_eq!(materialization.as_engine(), error.as_engine());
    let erased = error.into_erased(other.access());
    assert_eq!(
        erased.as_engine(),
        Some(&ErasedEngineError::JobContextRequired { required })
    );
}
#[derive(Default)]
struct Gate {
    ready: Cell<bool>,
    polls: Cell<usize>,
    drops: Cell<usize>,
    waker: RefCell<Option<Waker>>,
}
struct GateFuture(Rc<Gate>);
impl Future for GateFuture {
    type Output = surgeist_js::JsResult<JsValue>;
    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        self.0.polls.set(self.0.polls.get() + 1);
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
fn signal(gate: &Gate) {
    gate.ready.set(true);
    let waker = gate.waker.borrow_mut().take();
    if let Some(waker) = waker {
        waker.wake();
    }
}
fn reach_pending<F: Future<Output = surgeist_js::JsResult<()>> + Unpin>(
    driver: &mut F,
    gate: &Gate,
) {
    for _ in 0..16 {
        assert!(poll(driver).is_pending());
        if gate.polls.get() == 1 {
            return;
        }
    }
    panic!("the accepted task must reach Pending within sixteen driver turns")
}
#[test]
fn a_nonready_affinity_bound_task_refuses_a_peer_drain_then_resumes_on_its_owner() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut a = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut b = Context::builder()
        .runtime(a.runtime().clone())
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let required = a.id();
    let gate = Rc::new(Gate::default());
    let g = gate.clone();
    let job = NativeAsyncJob::new(move |_| Ok(ContextTask::from_future(GateFuture(g))));
    let observer = job.observer();
    a.enqueue_job(job.into()).unwrap().into_result().unwrap();
    {
        let host = a.host_access();
        let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
        reach_pending(&mut driver, &gate);
    }
    assert_eq!(gate.drops.get(), 0);
    let actual = RefCell::new(None);
    let peer = b
        .with_execution(ExecutionOptions::new(0), |access| {
            let host = access.into_host_access();
            let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
            for _ in 0..16 {
                if let Poll::Ready(result) = poll(&mut driver) {
                    *actual.borrow_mut() = Some(result);
                    return Ok(());
                }
            }
            panic!("a nonready foreign-affinity task must promptly identify its owner")
        })
        .unwrap();
    peer.result.unwrap();
    assert_eq!(peer.report.job_polls().unwrap(), 0);
    let error = actual.into_inner().unwrap().unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::JobContextRequired { required })
    );
    assert_eq!(gate.polls.get(), 1);
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    signal(&gate);
    let drained = a
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    drained.result.unwrap();
    assert_eq!(drained.report.job_preparations().unwrap(), 0);
    assert_eq!(drained.report.job_polls().unwrap(), 1);
    assert_eq!(gate.polls.get(), 2);
    assert_eq!(gate.drops.get(), 1);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
}
#[test]
fn cancellation_retires_only_the_observing_runtime_and_wakes_started_job_waiters() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut a = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut b = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let gate = Rc::new(Gate::default());
    let g = gate.clone();
    let started = NativeAsyncJob::new(move |_| Ok(ContextTask::from_future(GateFuture(g))));
    let started_observer = started.observer();
    let (mut wait, wakes) = pending_wait(&started_observer);
    a.enqueue_job(started.into())
        .unwrap()
        .into_result()
        .unwrap();
    let other = NativeAsyncJob::new(|_| Ok(ready()));
    let other_observer = other.observer();
    b.enqueue_job(other.into()).unwrap().into_result().unwrap();
    {
        let host = a.host_access();
        let mut driver = Box::pin(executor.clone().run_jobs_async(&host));
        reach_pending(&mut driver, &gate);
    }
    assert_eq!(started_observer.status().unwrap(), JobStatus::Pending);
    let queued_factories = Rc::new(Cell::new(0));
    let factories = queued_factories.clone();
    let queued = NativeAsyncJob::new(move |_| {
        factories.set(factories.get() + 1);
        Ok(ready())
    });
    let queued_observer = queued.observer();
    a.enqueue_job(queued.into()).unwrap().into_result().unwrap();
    assert_eq!(queued_observer.status().unwrap(), JobStatus::Pending);
    executor
        .get_cancellation_token()
        .store(true, Ordering::Release);
    a.run_jobs().unwrap();
    assert_eq!(
        started_observer.status().unwrap(),
        JobStatus::Retired(JobRetirement::Cancelled)
    );
    assert_eq!(
        poll(&mut wait).map(Result::unwrap),
        Poll::Ready(JobStatus::Retired(JobRetirement::Cancelled))
    );
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    assert_eq!(gate.drops.get(), 1);
    assert_eq!(
        queued_observer.status().unwrap(),
        JobStatus::Retired(JobRetirement::Cancelled)
    );
    assert_eq!(queued_factories.get(), 0);
    assert_eq!(other_observer.status().unwrap(), JobStatus::Pending);
    b.run_jobs().unwrap();
    assert_eq!(other_observer.status().unwrap(), JobStatus::Completed);
}
#[test]
fn reentrant_draining_never_polls_the_currently_claimed_task_twice() {
    let mut owner = Context::default();
    let order = Rc::new(RefCell::new(Vec::new()));
    let polls = Rc::new(Cell::new(0));
    let p = polls.clone();
    let o = order.clone();
    let job = NativeAsyncJob::new(move |_| {
        Ok(ContextTask::from_poll(move |_, host| {
            p.set(p.get() + 1);
            o.borrow_mut().push("outer");
            let inner_order = o.clone();
            host.with(|mut access| {
                access
                    .enqueue_job(
                        PromiseJob::new(move |_| {
                            inner_order.borrow_mut().push("new");
                            Ok(JsValue::new(7))
                        })
                        .into(),
                    )
                    .unwrap()
                    .into_result()
                    .unwrap();
                access.run_jobs()
            })?;
            o.borrow_mut().push("return");
            Poll::Ready(Ok(JsValue::new(42)))
        }))
    });
    let observer = job.observer();
    owner
        .enqueue_job(job.into())
        .unwrap()
        .into_result()
        .unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(polls.get(), 1);
    assert_eq!(*order.borrow(), ["outer", "new", "return"]);
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
}
#[test]
fn an_untouched_due_interval_survives_a_stop_and_can_cancel_itself_after_resumption() {
    let clock = Rc::new(FixedClock::from_millis(100));
    let builder = Context::builder().clock(clock.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut owner = builder.build().unwrap();
    owner
        .enqueue_job(
            TimeoutJob::new(
                NativeJob::new(|mut access| access.eval(Source::from_bytes("0"))),
                0,
            )
            .into(),
        )
        .unwrap()
        .into_result()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let c = calls.clone();
    let token: Rc<RefCell<Option<surgeist_js::job::CancellationToken>>> =
        Rc::new(RefCell::new(None));
    let t = token.clone();
    let interval = IntervalJob::new(
        NativeJobFn::new(move |access| {
            c.set(c.get() + 1);
            let cancellation = t
                .borrow()
                .as_ref()
                .expect("interval token staged before dispatch")
                .clone();
            cancellation.cancel(access);
            Ok(JsValue::new(42))
        }),
        0,
    );
    *token.borrow_mut() = Some(interval.cancellation_token().clone());
    owner
        .enqueue_job(interval.into())
        .unwrap()
        .into_result()
        .unwrap();
    clock.forward(1);
    let refused = owner
        .with_execution(ExecutionOptions::new(0), |mut access| access.run_jobs())
        .unwrap();
    assert_eq!(
        refused.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(calls.get(), 0);
    owner.run_jobs().unwrap();
    assert_eq!(calls.get(), 1);
    owner.run_jobs().unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn an_owned_synchronous_deferred_callback_can_run_once_in_a_fresh_operation() {
    let calls = Rc::new(Cell::new(0));
    let c = calls.clone();
    let capture = Rc::new(RefCell::new(None));
    let seen = capture.clone();
    let payload = String::from("retained synchronous payload");
    let slot = RefCell::new(Some(NativeJob::new(move |_| {
        c.set(c.get() + 1);
        *seen.borrow_mut() = Some((payload, 42));
        Ok(JsValue::new(42))
    })));
    let mut owner = Context::default();
    let first = owner
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(0),
            |access| {
                let job = slot.borrow_mut().take().unwrap();
                let JobCall::Deferred { job, error } = job.call(access) else {
                    panic!("the refusal must return the original owned callback")
                };
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobInvocation { limit: 0 }
                    ))
                );
                *slot.borrow_mut() = Some(job);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        first.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 0 }
        ))
    );
    assert_eq!(first.report.job_invocations().unwrap(), 0);
    assert_eq!(calls.get(), 0);
    let second = owner
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(1),
            |access| {
                let job = slot.borrow_mut().take().unwrap();
                match job.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { .. } => {
                        panic!("the fresh operation must admit the retained callback")
                    }
                }
            },
        )
        .unwrap();
    assert_eq!(second.result.unwrap(), JsValue::new(42));
    assert_eq!(second.report.job_invocations().unwrap(), 1);
    assert_eq!(calls.get(), 1);
    assert_eq!(
        capture.borrow().as_ref(),
        Some(&(String::from("retained synchronous payload"), 42))
    );
}
#[test]
fn ordinary_factory_and_poll_errors_complete_once_without_retirement() {
    for factory_error in [false, true] {
        let mut owner = Context::default();
        let mut job = NativeAsyncJob::new(move |_| {
            let error = JsError::from_opaque(JsValue::new(7));
            if factory_error {
                Err(error)
            } else {
                Ok(ContextTask::from_future(async move { Err(error) }))
            }
        });
        let observer = job.observer();
        let outcome = owner
            .with_execution(ExecutionOptions::new(0), |access| {
                let host = access.into_host_access();
                let first = step(&mut job, &host);
                let result = match first {
                    JobStep::Complete(result) => {
                        assert!(factory_error);
                        result
                    }
                    JobStep::Prepared => {
                        assert!(!factory_error);
                        completion(&mut job, &host)
                    }
                    _ => panic!("the factory must prepare or deliver its own ordinary error"),
                };
                assert_eq!(result.unwrap_err().as_opaque(), Some(&JsValue::new(7)));
                assert!(matches!(
                    step(&mut job, &host),
                    JobStep::Finished(Ok(JobStatus::Completed))
                ));
                Ok(())
            })
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(outcome.report.job_preparations().unwrap(), 1);
        assert_eq!(
            outcome.report.job_polls().unwrap(),
            u64::from(!factory_error)
        );
        assert_eq!(observer.status().unwrap(), JobStatus::Completed);
        drop(owner);
        assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    }
}
