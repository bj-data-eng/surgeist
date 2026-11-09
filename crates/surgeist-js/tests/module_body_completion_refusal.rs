#![allow(unused_crate_dependencies)]
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue, Module,
    NativeFunction, Source,
    builtins::promise::PromiseState,
    error::{
        DiagnosticEngine, DiagnosticKind, DiagnosticNativeKind, EngineError, ModuleInterruption,
        PanicError,
    },
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobRetirement, JobStatus,
    },
    js_string,
    object::builtins::{JsFuture, JsPromise},
    property::Attribute,
};
const FINITE: u64 = 100_000;
thread_local! { static MARKER: RefCell<Option<Rc<Cell<usize>>>> = const { RefCell::new(None) }; }
struct MarkerInput;
impl Drop for MarkerInput {
    fn drop(&mut self) {
        MARKER.with(|slot| drop(slot.borrow_mut().take()));
    }
}
struct CompletionExecutor {
    queue: RefCell<VecDeque<Job>>,
    target: RefCell<Option<JsPromise>>,
    marks: Rc<Cell<usize>>,
    refusal: RefCell<Option<JsError>>,
    returned: RefCell<Option<JobObserver>>,
}
impl JobExecutor for CompletionExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let result_pending = self
            .target
            .borrow()
            .as_ref()
            .is_some_and(|promise| promise.state().unwrap() == PromiseState::Pending);
        let refusal = if self.marks.get() > 0 && result_pending {
            self.refusal.borrow_mut().take()
        } else {
            None
        };
        if let Some(error) = refusal {
            let Job::PromiseJob(reaction) = &job else {
                panic!(
                    "the first actual publication after the body marker is its completion reaction"
                )
            };
            let observer = reaction.observer();
            assert_eq!(observer.status(), Ok(JobStatus::Pending));
            assert_eq!(observer.fault(), None);
            assert!(self.returned.replace(Some(observer)).is_none());
            // Move the exact uncalled input back to its actual producer.
            // No job is copied, stored by this executor, or invoked here.
            let returned =
                JobPublicationError::new(job, JobPublicationCause::try_new(error, access).unwrap());
            let Job::PromiseJob(original) = returned.job() else {
                unreachable!()
            };
            assert_eq!(original.observer().status(), Ok(JobStatus::Pending));
            return Err(returned);
        }
        self.queue.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches actual staged/resume wrappers explicitly")
    }
}
impl CompletionExecutor {
    fn step(&self, context: &mut Context) -> (JsResult<JsValue>, JsResult<JsValue>) {
        let queued = self
            .queue
            .borrow_mut()
            .pop_front()
            .expect("actual staged or Await resume work remains queued");
        let Job::PromiseJob(reaction) = queued else {
            panic!("the import-free source has real Promise wrappers")
        };
        let mut direct = None;
        let outcome = context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                let result = match reaction.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { job, error } => {
                        drop(job);
                        panic!("a fresh staged/resume action must actually enter: {error:?}")
                    }
                };
                direct = Some(result.clone());
                result
            })
            .unwrap();
        (direct.unwrap(), outcome.result)
    }
}
fn poll(wait: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn cause(error: &JsError, expected_leaf: &JsValue) -> ModuleInterruption {
    let Some(EngineError::ModuleInterrupted(value)) = error.as_engine() else {
        panic!("the actual refused body completion must interrupt its incomplete module origin")
    };
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(source),
    }) = value.cause().kind()
    else {
        panic!("the authored publication failure must remain the first cause")
    };
    assert_eq!(message, "authored body-completion refusal");
    let DiagnosticKind::Native(native) = source.kind() else {
        panic!("the original typed source must remain visible")
    };
    assert_eq!(native.kind(), DiagnosticNativeKind::Range);
    assert_eq!(native.message(), "authored completion cause");
    let DiagnosticKind::Opaque(leaf) = native.cause().unwrap().kind() else {
        panic!("the authored local diagnostic identity must remain visible")
    };
    assert_eq!(leaf.value().unwrap(), *expected_leaf);
    value.clone()
}
fn completion_refusal(resumed: bool, guest_throw: bool) {
    let marks = Rc::new(Cell::new(0));
    MARKER.with(|slot| assert!(slot.replace(Some(marks.clone())).is_none()));
    let _marker = MarkerInput;
    let executor = Rc::new(CompletionExecutor {
        queue: RefCell::new(VecDeque::new()),
        target: RefCell::new(None),
        marks: marks.clone(),
        refusal: RefCell::new(None),
        returned: RefCell::new(None),
    });
    let builder = Context::builder().job_executor(executor.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("markBodyDone"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                MARKER.with(|slot| {
                    let slot = slot.borrow();
                    let marks = slot.as_ref().unwrap();
                    marks.set(marks.get() + 1);
                });
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    let (gate, resolvers) = JsPromise::new_pending(context.access());
    context
        .register_global_property(js_string!("bodyCompletionGate"), gate, Attribute::all())
        .unwrap();
    let guest_reason: JsValue = JsObject::with_null_proto(context.runtime()).into();
    context
        .register_global_property(
            js_string!("bodyCompletionGuestReason"),
            guest_reason,
            Attribute::all(),
        )
        .unwrap();
    let hook_leaf: JsValue = JsObject::with_null_proto(context.runtime()).into();
    let original: JsError = PanicError::new("authored body-completion refusal")
        .with_source(
            JsNativeError::range()
                .with_message("authored completion cause")
                .with_cause(JsError::from_opaque(hook_leaf.clone())),
        )
        .into();
    let source = match (resumed, guest_throw) {
        (true, false) => "await bodyCompletionGate; markBodyDone();",
        (true, true) => {
            "await bodyCompletionGate; markBodyDone(); throw bodyCompletionGuestReason;"
        }
        (false, false) => "if (false) await 0; markBodyDone();",
        (false, true) => "if (false) await 0; markBodyDone(); throw bodyCompletionGuestReason;",
    };
    let module = Module::parse(Source::from_bytes(source), None, context.access()).unwrap();
    assert_eq!(
        module.load(context.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    let result = if resumed {
        // Here the real source canonical capability is publicly returned
        // before its genuine suspended body resumes.
        context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                module.evaluate(access)
            })
            .unwrap()
            .result
            .unwrap()
    } else {
        // Initial synchronous refusal prevents evaluate from returning its
        // source capability. This is the supported public native bridge target,
        // not an alleged leaked/external canonical source Promise.
        module.load_link_evaluate(context.access()).unwrap()
    };
    let identity = result.identity();
    assert_eq!(result.state().unwrap(), PromiseState::Pending);
    let mut wait = result.clone().into_js_future(context.access()).unwrap();
    assert!(poll(&mut wait).is_pending());
    assert_eq!(marks.get(), 0);
    *executor.target.borrow_mut() = Some(result.clone());
    *executor.refusal.borrow_mut() = Some(original.clone());
    if resumed {
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
            .unwrap();
    }
    let mut error = None;
    for _ in 0..12 {
        let (direct, effective) = executor.step(&mut context);
        if executor.returned.borrow().is_some() {
            error = Some((direct, effective));
            break;
        }
        direct.unwrap();
        effective.unwrap();
    }
    let (direct, effective) = error.expect(
        "the actual body completion must publish after the marker while its result remains Pending",
    );
    assert_eq!(direct.unwrap_err(), original);
    assert_eq!(effective.unwrap_err(), original);
    assert!(executor.refusal.borrow().is_none());
    assert_eq!(marks.get(), 1);
    let refused = executor.returned.borrow().as_ref().unwrap().clone();
    assert_eq!(
        refused.status(),
        Ok(JobStatus::Retired(JobRetirement::Dropped))
    );
    assert_eq!(refused.fault(), None);
    assert_eq!(result.identity(), identity);
    assert_eq!(result.state().unwrap(), PromiseState::Pending);
    let Poll::Ready(Err(observed)) = poll(&mut wait) else {
        panic!(
            "the original supported native result must not remain stranded on an unaccepted completion-only reaction"
        )
    };
    let first = cause(&observed, &hook_leaf);
    for _ in 0..2 {
        let later = context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                module.evaluate(access)
            })
            .unwrap();
        assert_eq!(later.report.instructions().unwrap(), 0);
        assert_eq!(
            cause(&later.result.unwrap_err(), &hook_leaf).identity(),
            first.identity()
        );
    }
    assert_eq!(marks.get(), 1);
    assert_eq!(result.state().unwrap(), PromiseState::Pending);
    // The producer deliberately disposed the returned original input. There is
    // no executor-held job/ghost continuation to retry or silently execute.
    assert!(executor.queue.borrow().is_empty());
}
#[test]
fn resumed_await_body_completion_refusal_preserves_the_original_error_and_canonical_native_wait() {
    for guest_throw in [false, true] {
        completion_refusal(true, guest_throw);
    }
}
#[test]
fn no_executed_await_body_completion_refusal_preserves_the_original_error_and_public_bridge_wait() {
    for guest_throw in [false, true] {
        completion_refusal(false, guest_throw);
    }
}
