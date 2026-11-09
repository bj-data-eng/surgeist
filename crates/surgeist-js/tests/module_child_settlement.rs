use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsNativeError, JsObject, JsResult, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{
        DiagnosticEngine, DiagnosticKind, DiagnosticNativeKind, EngineError, ModuleInterruption,
        PanicError,
    },
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobStatus, JobStep,
    },
    js_string,
    module::MapModuleLoader,
    object::builtins::{JsFunction, JsFuture, JsPromise},
    property::Attribute,
};
const FINITE: u64 = 100_000;
#[derive(Default)]
struct SettlementExecutor {
    queue: RefCell<VecDeque<Job>>,
    child: RefCell<Option<JsPromise>>,
    hook: RefCell<Option<JsError>>,
    saved: RefCell<Option<Job>>,
    observer: RefCell<Option<JobObserver>>,
}
impl JobExecutor for SettlementExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        // The child's actual PUBLIC top-level capability distinguishes this
        // canonical settlement from its earlier private body-completion job.
        let child_published = self
            .child
            .borrow()
            .as_ref()
            .is_some_and(|child| child.state().unwrap() != PromiseState::Pending);
        let hook = if child_published {
            self.hook.borrow_mut().take()
        } else {
            None
        };
        if let Some(error) = hook {
            let Job::PromiseJob(reaction) = &job else {
                panic!("the child's authored observation is a real PromiseJob")
            };
            assert!(self.observer.replace(Some(reaction.observer())).is_none());
            assert!(self.saved.replace(Some(job)).is_none());
            return Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error, access).unwrap(),
            ));
        }
        self.queue.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("actual loader and continuation wrappers are dispatched explicitly")
    }
}
impl SettlementExecutor {
    fn invoke(job: Job, access: ContextAccess<'_>) -> JsResult<JsValue> {
        match job {
            Job::PromiseJob(job) => match job.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!(
                        "the fresh actual callback must enter, never masquerade as accepted Deferred: {error:?}"
                    )
                }
            },
            Job::AsyncJob(mut loader) => {
                let host = access.into_host_access();
                let mut task = TaskContext::from_waker(Waker::noop());
                for _ in 0..8 {
                    match loader.run_step(&mut task, &host) {
                        JobStep::Prepared | JobStep::Pending => {}
                        JobStep::Complete(result) => return result,
                        JobStep::Deferred(error) => {
                            panic!("actual ready MapModuleLoader must enter: {error:?}")
                        }
                        JobStep::Finished(status) => {
                            panic!("actual loader must deliver its one result: {status:?}")
                        }
                    }
                }
                panic!("the finite ready loader exceeded its bounded dispatch allowance")
            }
            other => panic!("the graph owns its real loader/Promise wrappers: {other:?}"),
        }
    }
    fn next(&self) -> Job {
        self.queue
            .borrow_mut()
            .pop_front()
            .expect("actual staged or completion work remains queued")
    }
    fn step(&self, job: Job, context: &mut Context) -> (JsResult<JsValue>, JsResult<JsValue>) {
        let mut direct = None;
        let outcome = context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                let result = Self::invoke(job, access);
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
fn native(promise: &JsPromise, context: &mut Context) -> JsFuture {
    let mut wait = promise.clone().into_js_future(context.access()).unwrap();
    assert!(poll(&mut wait).is_pending());
    wait
}
fn count(name: &str, context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(surgeist_js::JsString::from(name), context.access())
        .unwrap()
}
fn evaluate(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}
fn interrupted(error: &JsError, hook_leaf: &JsValue) -> ModuleInterruption {
    let Some(EngineError::ModuleInterrupted(value)) = error.as_engine() else {
        panic!("the parent must receive a typed interruption after its required action was refused")
    };
    let DiagnosticKind::Engine(DiagnosticEngine::Panic {
        message,
        diagnostic: Some(source),
    }) = value.cause().kind()
    else {
        panic!("the exact first publication Panic must remain the parent cause")
    };
    assert_eq!(message, "authored child settlement publication failure");
    let DiagnosticKind::Native(native) = source.kind() else {
        panic!("the authored native source survives")
    };
    assert_eq!(native.kind(), DiagnosticNativeKind::Range);
    assert_eq!(native.message(), "authored settlement cause");
    let DiagnosticKind::Opaque(leaf) = native.cause().unwrap().kind() else {
        panic!("the original local diagnostic identity survives")
    };
    assert_eq!(leaf.value().unwrap(), *hook_leaf);
    value.clone()
}
fn child_settlement(rejected: bool) {
    let executor = Rc::new(SettlementExecutor::default());
    let loader = Rc::new(MapModuleLoader::new());
    let builder = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    for name in [
        "settlementChildEffects",
        "settlementParentEffects",
        "settlementParentValue",
        "settlementObserved",
    ] {
        context
            .register_global_property(surgeist_js::JsString::from(name), 0, Attribute::all())
            .unwrap();
    }
    let guest_reason: JsValue = JsObject::with_null_proto(context.runtime()).into();
    context
        .register_global_property(
            js_string!("settlementGuestReason"),
            guest_reason.clone(),
            Attribute::all(),
        )
        .unwrap();
    let (gate, resolvers) = JsPromise::new_pending(context.access());
    context
        .register_global_property(js_string!("settlementGate"), gate, Attribute::all())
        .unwrap();
    let child_source = if rejected {
        "await settlementGate; globalThis.settlementChildEffects += 1; throw settlementGuestReason; export const value = 40;"
    } else {
        "await settlementGate; globalThis.settlementChildEffects += 1; export const value = 40;"
    };
    let child = Module::parse(Source::from_bytes(child_source), None, context.access()).unwrap();
    assert!(loader.insert("settlement-child", child.clone()).is_none());
    let parent = Module::parse(Source::from_bytes(
        "import {value} from 'settlement-child'; globalThis.settlementParentEffects += 1; globalThis.settlementParentValue = value + 2;"
    ), None, context.access()).unwrap();
    let loaded = parent.load(context.access()).unwrap();
    for _ in 0..16 {
        if loaded.state().unwrap() != PromiseState::Pending {
            break;
        }
        let (direct, effective) = executor.step(executor.next(), &mut context);
        direct.unwrap();
        effective.unwrap();
    }
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    parent.link(context.access()).unwrap();
    let child_result = evaluate(&child, &mut context);
    let child_identity = child_result.identity();
    let mut child_wait = native(&child_result, &mut context);
    let parent_result = evaluate(&parent, &mut context);
    let parent_identity = parent_result.identity();
    let mut parent_wait = native(&parent_result, &mut context);
    assert_eq!(child_result.state().unwrap(), PromiseState::Pending);
    assert_eq!(parent_result.state().unwrap(), PromiseState::Pending);
    let observer_fn = JsFunction::from_object(
        context
            .eval_with_budget(
                Source::from_bytes(
                    "(value) => { globalThis.settlementObserved += 1; return value; }",
                ),
                ExecutionOptions::new(FINITE),
            )
            .unwrap()
            .result
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let observation = if rejected {
        child_result
            .then(None, Some(observer_fn), context.access())
            .unwrap()
    } else {
        child_result
            .then(Some(observer_fn), None, context.access())
            .unwrap()
    };
    let mut observation_wait = native(&observation, &mut context);
    let hook_leaf: JsValue = JsObject::with_null_proto(context.runtime()).into();
    let original: JsError = PanicError::new("authored child settlement publication failure")
        .with_source(
            JsNativeError::range()
                .with_message("authored settlement cause")
                .with_cause(JsError::from_opaque(hook_leaf.clone())),
        )
        .into();
    *executor.child.borrow_mut() = Some(child_result.clone());
    *executor.hook.borrow_mut() = Some(original.clone());
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[JsValue::new(7)], context.access())
        .unwrap();
    let mut failure = None;
    for _ in 0..16 {
        let (direct, effective) = executor.step(executor.next(), &mut context);
        if executor.saved.borrow().is_some() {
            failure = Some((direct, effective));
            break;
        }
        direct.unwrap();
        effective.unwrap();
    }
    let (direct, effective) =
        failure.expect("canonical child settlement must publish the authored observer");
    assert_eq!(
        direct.unwrap_err(),
        original,
        "the actual completion callback must not wrap the original publication cause"
    );
    assert_eq!(effective.unwrap_err(), original);
    let expected_child = if rejected {
        PromiseState::Rejected(guest_reason.clone())
    } else {
        PromiseState::Fulfilled(JsValue::undefined())
    };
    assert_eq!(child_result.state().unwrap(), expected_child);
    assert_eq!(child_result.identity(), child_identity);
    if rejected {
        let Poll::Ready(Err(reason)) = poll(&mut child_wait) else {
            panic!("the original child native wait retains its published guest rejection")
        };
        assert_eq!(reason.as_opaque(), Some(&guest_reason));
        assert!(reason.as_engine().is_none());
    } else {
        assert_eq!(poll(&mut child_wait), Poll::Ready(Ok(JsValue::undefined())));
    }
    let cached_child = evaluate(&child, &mut context);
    assert_eq!(cached_child.identity(), child_identity);
    assert_eq!(cached_child.state().unwrap(), expected_child);
    assert_eq!(
        count("settlementChildEffects", &mut context),
        JsValue::new(1)
    );
    assert_eq!(parent_result.identity(), parent_identity);
    assert_eq!(parent_result.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        count("settlementParentEffects", &mut context),
        JsValue::new(0)
    );
    let Poll::Ready(Err(parent_error)) = poll(&mut parent_wait) else {
        panic!(
            "mandatory parent dependency/rejection bookkeeping must not strand the original native importer Pending"
        )
    };
    let parent_cause = interrupted(&parent_error, &hook_leaf);
    let later = context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            parent.evaluate(access)
        })
        .unwrap();
    assert_eq!(later.report.instructions().unwrap(), 0);
    assert_eq!(
        interrupted(&later.result.unwrap_err(), &hook_leaf).identity(),
        parent_cause.identity()
    );
    assert_eq!(count("settlementObserved", &mut context), JsValue::new(0));
    let observer = executor.observer.borrow().as_ref().unwrap().clone();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    assert_eq!(observation.state().unwrap(), PromiseState::Pending);
    assert!(poll(&mut observation_wait).is_pending());
    let saved = executor.saved.borrow_mut().take().unwrap();
    let (direct, effective) = executor.step(saved, &mut context);
    direct.unwrap();
    effective.unwrap();
    let observation_value = if rejected {
        guest_reason
    } else {
        JsValue::undefined()
    };
    assert_eq!(
        observation.state().unwrap(),
        PromiseState::Fulfilled(observation_value.clone())
    );
    assert_eq!(
        poll(&mut observation_wait),
        Poll::Ready(Ok(observation_value))
    );
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(count("settlementObserved", &mut context), JsValue::new(1));
    assert_eq!(
        count("settlementParentEffects", &mut context),
        JsValue::new(0)
    );
    assert_eq!(evaluate(&child, &mut context).identity(), child_identity);
    let repeated = context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            parent.evaluate(access)
        })
        .unwrap();
    assert_eq!(repeated.report.instructions().unwrap(), 0);
    assert_eq!(
        interrupted(&repeated.result.unwrap_err(), &hook_leaf).identity(),
        parent_cause.identity()
    );
}
#[test]
fn fulfilled_child_observer_publication_failure_preserves_its_terminal_result_and_finishes_parent_refusal()
 {
    child_settlement(false);
}
#[test]
fn rejected_child_observer_publication_failure_preserves_its_guest_reason_and_finishes_parent_refusal()
 {
    child_settlement(true);
}
