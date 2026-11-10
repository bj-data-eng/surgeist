use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsResult, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobStatus,
    },
    js_string,
    object::builtins::JsPromise,
    property::Attribute,
};

const STOP: u64 = 256;
const FRESH: u64 = 100_000;
#[derive(Default)]
struct TransferExecutor {
    queue: RefCell<VecDeque<Job>>,
    inline_next: Cell<bool>,
    first_observer: RefCell<Option<JobObserver>>,
    first_error: RefCell<Option<JsError>>,
}
impl JobExecutor for TransferExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        if !self.inline_next.replace(false) {
            self.queue.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        }
        let Job::PromiseJob(reaction) = job else {
            panic!("the initial await owns the authored first inline reaction")
        };
        *self.first_observer.borrow_mut() = Some(reaction.observer());
        match reaction.call(access.reborrow()) {
            JobCall::Complete(result) => {
                result.unwrap();
            }
            JobCall::Deferred { .. } => panic!(
                "the inline action must really transfer its next await before the publication hook fails"
            ),
        }
        let error = access
            .eval(Source::from_bytes(
                "for (let publicationStep = 0; publicationStep < 10000; publicationStep++) {}",
            ))
            .expect_err(
                "the finite publication hook exhausts the authored finite instruction allowance",
            );
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                limit: STOP
            }))
        );
        assert!(self.first_error.replace(Some(error.clone())).is_none());
        Ok(JobAcceptance::AcceptedWithError(
            JobPublicationCause::try_new(error, access).unwrap(),
        ))
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the test dispatches its accepted real wrappers explicitly")
    }
}
fn effects(owner: &mut Context) -> JsValue {
    owner
        .global_object()
        .unwrap()
        .get(js_string!("publicationEffects"), owner.access())
        .unwrap()
}
fn evaluate(module: &Module, owner: &mut Context) -> JsPromise {
    owner
        .with_execution(ExecutionOptions::new(FRESH), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}

#[test]
fn accepted_inline_publication_failure_preserves_a_transferred_module_await() {
    let executor = Rc::new(TransferExecutor::default());
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut owner = builder.build().unwrap();
    owner
        .register_global_property(js_string!("publicationEffects"), 0, Attribute::all())
        .unwrap();
    let (gate, resolvers) = JsPromise::new_pending(owner.access()).unwrap();
    owner
        .register_global_property(
            js_string!("publicationGate"),
            gate.clone(),
            Attribute::all(),
        )
        .unwrap();
    let module = Module::parse(Source::from_bytes(
        "await 0; globalThis.publicationEffects += 11; await publicationGate; globalThis.publicationEffects += 22;"
    ), None, owner.access()).unwrap();
    let loaded = module.load(owner.access()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(owner.access()).unwrap();
    assert!(executor.queue.borrow().is_empty());
    executor.inline_next.set(true);
    let stopped = owner
        .with_execution(ExecutionOptions::new(STOP), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let error = stopped.result.unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: STOP
        }))
    );
    assert_eq!(executor.first_error.borrow().as_ref(), Some(&error));
    assert_eq!(stopped.report.instructions().unwrap(), STOP);
    assert_eq!(
        executor.first_observer.borrow().as_ref().unwrap().status(),
        Ok(JobStatus::Completed)
    );
    assert_eq!(effects(&mut owner), JsValue::new(11));
    assert_eq!(gate.state().unwrap(), PromiseState::Pending);

    // The actual inline callback committed a different suspension. Receiving an
    // accepted-error report is no authority to revoke that newer continuation.
    let evaluation = evaluate(&module, &mut owner);
    assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    let mut native = evaluation.clone().into_js_future(owner.access()).unwrap();
    assert!(
        Pin::new(&mut native)
            .poll(&mut TaskContext::from_waker(Waker::noop()))
            .is_pending()
    );
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], owner.access())
        .unwrap();
    for _ in 0..16 {
        if evaluation.state().unwrap() != PromiseState::Pending {
            break;
        }
        let queued = executor
            .queue
            .borrow_mut()
            .pop_front()
            .expect("the transferred await retains its actual accepted reaction");
        let Job::PromiseJob(reaction) = queued else {
            panic!("the fixture dispatches its actual accepted promise reactions")
        };
        let resumed = owner
            .with_execution(ExecutionOptions::new(FRESH), |access| {
                match reaction.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { .. } => {
                        panic!("the fresh budget must admit the accepted continuation")
                    }
                }
            })
            .unwrap();
        resumed.result.unwrap();
    }
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        Pin::new(&mut native).poll(&mut TaskContext::from_waker(Waker::noop())),
        Poll::Ready(Ok(JsValue::undefined()))
    );
    assert_eq!(effects(&mut owner), JsValue::new(33));
    let cached = evaluate(&module, &mut owner);
    assert_eq!(cached.identity(), evaluation.identity());
    assert_eq!(effects(&mut owner), JsValue::new(33));
}
