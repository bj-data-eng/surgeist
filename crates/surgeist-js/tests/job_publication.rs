use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue, Source,
    error::{
        DiagnosticKind, DiagnosticNativeKind, EngineError, EngineResourceError, PanicError,
        RuntimeLimitError,
    },
    execution::ExecutionOptions,
    job::{
        ContextTask, GenericJob, IdleJobExecutor, Job, JobAcceptance, JobCall, JobExecutor,
        JobPublicationCause, JobPublicationError, JobRetirement, JobStatus, JobStep,
        NativeAsyncJob, NativeJob, SimpleJobExecutor, TimeoutJob,
    },
};

fn context<Q: JobExecutor + 'static>(executor: Rc<Q>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn realm_witness(owner: &mut Context) -> surgeist_js::gc::heap::ObjectId {
    // A real Error authored in this Context records its actual constructor
    // realm. Existing public diagnostics expose that independent identity.
    let value = owner
        .eval(Source::from_bytes("new Error('authored realm witness')"))
        .unwrap();
    let native = JsError::from_opaque(value)
        .unwrap()
        .try_native(owner.access())
        .unwrap();
    let outer = PanicError::new("realm identity witness").with_source(native);
    let DiagnosticKind::Native(view) = outer.diagnostic().unwrap().kind() else {
        panic!("the authored Error remains a native diagnostic")
    };
    view.realm_identity().unwrap()
}
struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
struct Rejector(JsError);
impl JobExecutor for Rejector {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        let cause = JobPublicationCause::try_new(self.0.clone(), access).unwrap();
        Err(JobPublicationError::new(job, cause))
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        Ok(())
    }
}

#[derive(Default)]
struct ManualQueue(RefCell<VecDeque<Job>>);
impl JobExecutor for ManualQueue {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("dispatch uses the retained real wrapper explicitly")
    }
}

#[test]
fn rejected_generic_job_retries_with_its_original_realm_capture_and_observer() {
    let expected: JsError = PanicError::new("authored rejection")
        .with_source(JsNativeError::range().with_message("authored source"))
        .into();
    let mut owner = context(Rc::new(Rejector(expected.clone())));
    let selected = owner.realm().clone();
    let captured_realm = selected.clone();
    let value = JsObject::with_null_proto(owner.runtime()).unwrap();
    let supplied = value.clone();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let observed_value = Rc::new(RefCell::new(None));
    let delivered = observed_value.clone();
    let job = GenericJob::new(
        move |access| {
            let _keep = &probe;
            assert_eq!(access.realm(), &captured_realm);
            seen.set(seen.get() + 1);
            *delivered.borrow_mut() = Some(supplied);
            Ok(JsValue::new(42))
        },
        selected.clone(),
    );
    let observer = job.observer();
    let error = owner.enqueue_job(job.into()).unwrap_err();
    assert_eq!(error.cause().error(), &expected);
    let Job::GenericJob(original) = error.job() else {
        panic!("publication returns the original job kind")
    };
    assert_eq!(original.realm(), &selected);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let (job, cause) = error.into_parts();
    assert_eq!(cause.into_error(), expected);
    let accepted = Rc::new(SimpleJobExecutor::new());
    assert!(matches!(
        accepted.clone().enqueue_job(job, owner.access()).unwrap(),
        JobAcceptance::Accepted
    ));
    assert_eq!(calls.get(), 0);
    accepted.run_jobs(owner.access()).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(drops.get(), 1);
    assert_eq!(observed_value.borrow().as_ref(), Some(&value));
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
}

#[test]
fn rejection_does_not_bind_an_unstarted_async_factory_to_the_receiving_context() {
    let expected: JsError = EngineResourceError::Allocation.into();
    let mut original = context(Rc::new(Rejector(expected.clone())));
    let mut receiving = Context::builder()
        .runtime(original.runtime().clone())
        .build()
        .unwrap();
    assert_ne!(original.id(), receiving.id());
    let factories = Rc::new(Cell::new(0));
    let made = factories.clone();
    let actual_owner = Rc::new(RefCell::new(None));
    let selected = actual_owner.clone();
    let job = NativeAsyncJob::new(move |access| {
        made.set(made.get() + 1);
        *selected.borrow_mut() = Some(access.id());
        Ok(ContextTask::from_future(std::future::ready(Ok(
            JsValue::new(42),
        ))))
    });
    let observer = job.observer();
    let refused = original.enqueue_job(job.into()).unwrap_err();
    assert_eq!(refused.cause().error(), &expected);
    assert_eq!(factories.get(), 0);
    assert!(actual_owner.borrow().is_none());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let (job, _) = refused.into_parts();
    let Job::AsyncJob(mut job) = job else {
        panic!("the unstarted async wrapper is preserved")
    };
    let host = receiving.host_access();
    assert!(matches!(
        job.run_step(
            &mut std::task::Context::from_waker(std::task::Waker::noop()),
            &host
        ),
        JobStep::Prepared
    ));
    let JobStep::Complete(result) = job.run_step(
        &mut std::task::Context::from_waker(std::task::Waker::noop()),
        &host,
    ) else {
        panic!("the original ready task completes")
    };
    assert_eq!(result.unwrap(), JsValue::new(42));
    assert_eq!(factories.get(), 1);
    assert_eq!(
        *actual_owner.borrow(),
        Some(host.with(|access| Ok(access.id())).unwrap())
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
}

#[test]
fn deliberate_disposal_of_a_rejected_timer_preserves_then_releases_its_exact_cancellation_ownership()
 {
    let mut owner = context(Rc::new(Rejector(EngineResourceError::Allocation.into())));
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
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
    let timer = TimeoutJob::new(callback, 7);
    let token = timer.cancellation_token().clone();
    let cancelled = Rc::new(Cell::new(0));
    let notified = cancelled.clone();
    token
        .push_callback(move |_| notified.set(notified.get() + 1))
        .unwrap();
    let refused = owner.enqueue_job(timer.into()).unwrap_err();
    assert_eq!(drops.get(), 0);
    assert_eq!(calls.get(), 0);
    assert!(!token.revoked());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let Job::TimeoutJob(timer) = refused.job() else {
        panic!("original timer wrapper")
    };
    assert_eq!(timer.timeout().as_nanos(), 7_000_000);
    // The returned wrapper's token is the very cancellation owner registered
    // through the caller's alias, demonstrated by its actual callback.
    timer.cancellation_token().cancel(owner.access());
    assert!(token.revoked());
    assert_eq!(cancelled.get(), 1);
    let disposed_error = refused.into_error();
    assert_eq!(
        disposed_error.as_engine(),
        Some(&EngineError::Resource(EngineResourceError::Allocation))
    );
    assert_eq!(drops.get(), 1);
    assert_eq!(calls.get(), 0);
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
}

struct MaximumClock;
impl surgeist_js::context::time::Clock for MaximumClock {
    fn now(&self) -> surgeist_js::context::time::JsInstant {
        surgeist_js::context::time::JsInstant::new(u64::MAX, 999_999_999)
    }
    fn system_time_millis(&self) -> i64 {
        0
    }
}

#[test]
fn idle_policy_discards_even_a_foreign_timer_without_sampling_an_overflowing_deadline() {
    let original = Context::builder().build().unwrap();
    let mut owner = Context::builder()
        .job_executor(Rc::new(IdleJobExecutor))
        .clock(Rc::new(MaximumClock))
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
    let probe = DropProbe(drops.clone());
    let callback = NativeJob::with_realm(
        move |_| {
            let _keep = &probe;
            seen.set(seen.get() + 1);
            Ok(JsValue::undefined())
        },
        original.realm().clone(),
    );
    let observer = callback.observer();
    let timer = TimeoutJob::new(callback, 1);
    let token = timer.cancellation_token().clone();
    let discarded = owner.enqueue_job(timer.into()).unwrap();
    assert!(matches!(&discarded, JobAcceptance::Discarded));
    discarded.into_result().unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
    assert!(token.revoked());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
}

#[test]
fn actual_deadline_rejection_retains_exact_clock_delay_and_unaccepted_timer_custody() {
    let mut owner = Context::builder()
        .clock(Rc::new(MaximumClock))
        .build()
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let drops = Rc::new(Cell::new(0));
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
    let timer = TimeoutJob::new(callback, 1);
    let token = timer.cancellation_token().clone();
    let refused = owner.enqueue_job(timer.into()).unwrap_err();
    let Some(EngineError::JobDeadlineOverflow { now, delay }) = refused.cause().error().as_engine()
    else {
        panic!("the initial timer failure must retain its explicit deadline cause")
    };
    assert_eq!(
        *now,
        surgeist_js::context::time::JsInstant::new(u64::MAX, 999_999_999)
    );
    assert_eq!(delay.as_nanos(), 1_000_000);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 0);
    assert!(!token.revoked());
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let (job, cause) = refused.into_parts();
    let Job::TimeoutJob(job) = job else {
        panic!("the original unaccepted timer must be returned")
    };
    assert_eq!(job.timeout().as_nanos(), 1_000_000);
    assert!(matches!(
        cause.into_error().as_engine(),
        Some(EngineError::JobDeadlineOverflow { .. })
    ));
    drop(job);
    assert_eq!(calls.get(), 0);
    assert_eq!(drops.get(), 1);
    assert!(token.revoked());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(observer.fault(), None);
}

#[test]
fn checked_publication_causes_keep_authored_diagnostics_and_return_catchable_inputs() {
    let mut owner = Context::builder().build().unwrap();
    let value = JsObject::with_null_proto(owner.runtime()).unwrap();
    let ordinary = JsError::from_opaque(value.clone().into()).unwrap();
    let refused = JobPublicationCause::try_new(ordinary.clone(), owner.access()).unwrap_err();
    assert_eq!(refused, ordinary);
    assert_eq!(refused.as_opaque().unwrap().as_object().unwrap(), value);
    let authored: JsError = PanicError::new("authored publication panic")
        .with_source(JsNativeError::aggregate(vec![
            ordinary,
            JsNativeError::range()
                .with_message("second authored member")
                .into(),
        ]))
        .into();
    let cause = JobPublicationCause::try_new(authored.clone(), owner.access()).unwrap();
    assert_eq!(cause.error(), &authored);
    let Some(EngineError::Panic(panic)) = cause.error().as_engine() else {
        panic!("the actual admitted panic remains typed")
    };
    let DiagnosticKind::Native(aggregate) = panic.diagnostic().unwrap().kind() else {
        panic!("authored aggregate source")
    };
    assert_eq!(aggregate.kind(), DiagnosticNativeKind::Aggregate);
    assert_eq!(aggregate.members().len(), 2);
    assert_eq!(cause.into_error(), authored);
}

struct InlineExecutor;
impl JobExecutor for InlineExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Job::GenericJob(job) = job else {
            panic!("this host policy dispatches the authored generic wrapper only")
        };
        match job.call(access.reborrow()) {
            JobCall::Complete(Ok(_)) => Ok(JobAcceptance::Accepted),
            JobCall::Complete(Err(error)) => Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            JobCall::Deferred { job, error } => Err(JobPublicationError::new(
                job.into(),
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        Ok(())
    }
}

#[test]
fn an_inline_engine_failure_is_accepted_once_with_its_full_original_cause() {
    let mut owner = context(Rc::new(InlineExecutor));
    let authored: JsError = PanicError::new("actual inline failure")
        .with_source(JsNativeError::range().with_message("retained native cause"))
        .into();
    let supplied = authored.clone();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let job = GenericJob::new(
        move |_| {
            seen.set(seen.get() + 1);
            Err(supplied)
        },
        owner.realm().clone(),
    );
    let observer = job.observer();
    let acceptance = owner.enqueue_job(job.into()).unwrap();
    let JobAcceptance::AcceptedWithError(cause) = &acceptance else {
        panic!("an actually consumed callback cannot be rejected as unstarted")
    };
    assert_eq!(cause.error(), &authored);
    assert_eq!(calls.get(), 1);
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Interrupted))
    );
    assert_eq!(acceptance.into_result().unwrap_err(), authored);
}

#[test]
fn inline_before_action_refusal_returns_the_original_job_for_fresh_admission() {
    let mut owner = context(Rc::new(InlineExecutor));
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let mut original = None;
    let selected = owner.realm().clone();
    let job = GenericJob::new(
        move |_| {
            seen.set(seen.get() + 1);
            Ok(JsValue::new(42))
        },
        selected,
    );
    let observer = job.observer();
    let stopped = owner
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(0),
            |mut access| {
                let rejected = access.enqueue_job(job.into()).unwrap_err();
                assert_eq!(
                    rejected.cause().error().as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::JobInvocation { limit: 0 }
                    ))
                );
                original = Some(rejected.into_parts().0);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::JobInvocation { limit: 0 }
        ))
    );
    assert_eq!(stopped.report.job_invocations().unwrap(), 0);
    assert!(!stopped.report.effects_started());
    assert_eq!(calls.get(), 0);
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let fresh = owner
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(1),
            |mut access| {
                assert!(matches!(
                    access.enqueue_job(original.take().unwrap()).unwrap(),
                    JobAcceptance::Accepted
                ));
                Ok(())
            },
        )
        .unwrap();
    fresh.result.unwrap();
    assert_eq!(fresh.report.job_invocations().unwrap(), 1);
    assert_eq!(calls.get(), 1);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
}

#[test]
fn retained_stop_does_not_block_valid_queue_custody_and_remains_the_first_result() {
    let executor = Rc::new(ManualQueue::default());
    let mut owner = context(executor.clone());
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let job = GenericJob::new(
        move |_| {
            seen.set(seen.get() + 1);
            Ok(JsValue::new(42))
        },
        owner.realm().clone(),
    );
    let observer = job.observer();
    let old = owner
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let first = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_eq!(
                first.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: 0
                }))
            );
            assert!(matches!(
                access.enqueue_job(job.into()).unwrap(),
                JobAcceptance::Accepted
            ));
            let later = JobPublicationCause::try_new(
                EngineResourceError::Allocation.into(),
                access.reborrow(),
            )
            .unwrap();
            assert_eq!(later.error(), &first);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        old.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(old.report.job_invocations().unwrap(), 0);
    assert!(!old.report.effects_started());
    assert_eq!(calls.get(), 0);
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let queued = executor.0.borrow_mut().pop_front().unwrap();
    let Job::GenericJob(queued) = queued else {
        panic!("the accepted original generic wrapper survives")
    };
    let fresh = owner
        .with_execution(
            ExecutionOptions::new(0).with_job_invocation_limit(1),
            |access| match queued.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { .. } => {
                    panic!("the new operation admits the original retained callback")
                }
            },
        )
        .unwrap();
    assert_eq!(fresh.result.unwrap(), JsValue::new(42));
    assert_eq!(fresh.report.job_invocations().unwrap(), 1);
    assert_eq!(calls.get(), 1);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
}

#[test]
fn foreign_publication_retains_the_original_job_while_direct_entry_keeps_its_type_error() {
    let mut selected = context(Rc::new(SimpleJobExecutor::new()));
    let mut receiving = context(Rc::new(SimpleJobExecutor::new()));
    let selected_identity = realm_witness(&mut selected);
    let receiving_identity = realm_witness(&mut receiving);
    let selected_realm = selected.realm().clone();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let job = GenericJob::new(
        move |_| {
            seen.set(seen.get() + 1);
            Ok(JsValue::new(42))
        },
        selected_realm.clone(),
    );
    let observer = job.observer();
    let JobCall::Deferred { job, error: direct } = job.call(receiving.access()) else {
        panic!("foreign direct entry must preserve the original wrapper")
    };
    assert!(matches!(
        direct.as_native().unwrap().kind(),
        surgeist_js::JsNativeErrorKind::Type
    ));
    assert!(direct.as_engine().is_none());
    let refused = receiving.enqueue_job(job.into()).unwrap_err();
    let Some(EngineError::ForeignSelectedRealm(mismatch)) = refused.cause().error().as_engine()
    else {
        panic!("publication must use its explicit receiving failure")
    };
    assert_eq!(mismatch.selected_realm(), selected_identity);
    assert_eq!(mismatch.receiving_realm(), receiving_identity);
    assert!(
        mismatch
            .selected_realm()
            .same_heap(selected.global_object().unwrap().identity())
    );
    assert!(
        mismatch
            .receiving_realm()
            .same_heap(receiving.global_object().unwrap().identity())
    );
    let DiagnosticKind::Native(cause) = mismatch.diagnostic().kind() else {
        panic!("the owner-generated direct TypeError remains typed diagnostic data")
    };
    assert_eq!(cause.kind(), DiagnosticNativeKind::Type);
    assert_eq!(calls.get(), 0);
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    let (job, _) = refused.into_parts();
    let Job::GenericJob(job) = job else {
        panic!("the same foreign-selected wrapper is returned")
    };
    assert_eq!(job.realm(), &selected_realm);
    match job.call(selected.access()) {
        JobCall::Complete(result) => assert_eq!(result.unwrap(), JsValue::new(42)),
        JobCall::Deferred { .. } => panic!("the selected runtime admits its original wrapper"),
    }
    assert_eq!(calls.get(), 1);
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
}
