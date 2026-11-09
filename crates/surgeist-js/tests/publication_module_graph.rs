use std::{
    cell::RefCell,
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
        JobPublicationError, JobRetirement, JobStatus, JobStep,
    },
    js_string,
    module::MapModuleLoader,
    object::builtins::{JsFuture, JsPromise},
    property::Attribute,
};
const FRESH: u64 = 100_000;
#[derive(Default)]
struct GraphExecutor {
    queue: RefCell<VecDeque<Job>>,
    hook: RefCell<Option<JsError>>,
    saved: RefCell<Option<Job>>,
    observer: RefCell<Option<JobObserver>>,
}
impl JobExecutor for GraphExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Some(error) = self.hook.borrow_mut().take() else {
            self.queue.borrow_mut().push_back(job);
            return Ok(JobAcceptance::Accepted);
        };
        let Job::PromiseJob(reaction) = &job else {
            panic!("the child body arms its actual initial Await source reaction")
        };
        assert!(self.observer.replace(Some(reaction.observer())).is_none());
        // This actual host-owned slot accepts the exact command BEFORE the
        // independent hook error. No inline call or entry refusal occurs.
        assert!(self.saved.replace(Some(job)).is_none());
        Ok(JobAcceptance::AcceptedWithError(
            JobPublicationCause::try_new(error, access).unwrap(),
        ))
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches its retained actual wrappers explicitly")
    }
}
impl GraphExecutor {
    fn observer(&self) -> JobObserver {
        self.observer.borrow().as_ref().unwrap().clone()
    }
    fn invoke(job: Job, access: ContextAccess<'_>) -> JsResult<JsValue> {
        match job {
            Job::PromiseJob(reaction) => match reaction.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!(
                        "an actual selected action must not masquerade as pre-entry Deferred: {error:?}"
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
                            panic!("the fresh actual in-memory loader must enter: {error:?}")
                        }
                        JobStep::Finished(status) => {
                            panic!("the actual loader must return its one result: {status:?}")
                        }
                    }
                }
                panic!("the finite ready MapModuleLoader exceeded its bounded dispatch allowance")
            }
            other => panic!("the graph owns its actual loader and Promise wrappers: {other:?}"),
        }
    }
    fn queue_job(&self) -> Job {
        self.queue
            .borrow_mut()
            .pop_front()
            .expect("actual graph staging/completion work remains queued")
    }
    fn saved_job(&self) -> Job {
        self.saved
            .borrow_mut()
            .take()
            .expect("the actual accepted child continuation remains owned")
    }
    fn fresh(&self, job: Job, context: &mut Context) -> JsResult<JsValue> {
        let mut direct = None;
        let outcome = context
            .with_execution(ExecutionOptions::new(FRESH), |access| {
                let result = Self::invoke(job, access);
                direct = Some(result.clone());
                result
            })
            .unwrap();
        assert_eq!(outcome.result, direct.unwrap());
        outcome.result
    }
    fn finish(&self, promise: &JsPromise, context: &mut Context) {
        for _ in 0..24 {
            if promise.state().unwrap() != PromiseState::Pending {
                return;
            }
            self.fresh(self.queue_job(), context).unwrap();
        }
        assert_ne!(
            promise.state().unwrap(),
            PromiseState::Pending,
            "the actual finite graph completion chain must finish"
        );
    }
}
thread_local! {
    static GRAPH_HOOK: RefCell<Option<(Rc<GraphExecutor>, JsError)>> = const { RefCell::new(None) };
}
struct GraphHookInput;
impl Drop for GraphHookInput {
    fn drop(&mut self) {
        GRAPH_HOOK.with(|slot| drop(slot.borrow_mut().take()));
    }
}
struct Graph {
    context: Context,
    executor: Rc<GraphExecutor>,
    loader: Rc<MapModuleLoader>,
    original: JsError,
    opaque: JsValue,
    _input: GraphHookInput,
}
impl Graph {
    fn new() -> Self {
        let executor = Rc::new(GraphExecutor::default());
        let loader = Rc::new(MapModuleLoader::new());
        let builder = Context::builder()
            .job_executor(executor.clone())
            .module_loader(loader.clone());
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        for name in [
            "graphChild",
            "graphParent",
            "graphSibling",
            "graphFinished",
            "graphResult",
        ] {
            context
                .register_global_property(surgeist_js::JsString::from(name), 0, Attribute::all())
                .unwrap();
        }
        context
            .register_global_builtin_callable(
                js_string!("armGraphPostStoreHook"),
                0,
                NativeFunction::from_fn_ptr(|_, _, _| {
                    GRAPH_HOOK.with(|slot| {
                        let (executor, error) =
                            slot.borrow_mut().take().expect("one actual child-body arm");
                        assert!(executor.hook.replace(Some(error)).is_none());
                    });
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        let opaque: JsValue = JsObject::with_null_proto(context.runtime()).into();
        let original: JsError = PanicError::new("authored graph post-storage failure")
            .with_source(
                JsNativeError::range()
                    .with_message("graph retained cause")
                    .with_cause(JsError::from_opaque(opaque.clone())),
            )
            .into();
        GRAPH_HOOK.with(|slot| {
            assert!(
                slot.replace(Some((executor.clone(), original.clone())))
                    .is_none()
            )
        });
        Self {
            context,
            executor,
            loader,
            original,
            opaque,
            _input: GraphHookInput,
        }
    }
    fn insert(&mut self, name: &str, source: &str) -> Module {
        let module =
            Module::parse(Source::from_bytes(source), None, self.context.access()).unwrap();
        assert!(self.loader.insert(name, module.clone()).is_none());
        module
    }
    fn ready(&mut self, root: &Module) {
        let loaded = root.load(self.context.access()).unwrap();
        for _ in 0..16 {
            if loaded.state().unwrap() != PromiseState::Pending {
                break;
            }
            self.executor
                .fresh(self.executor.queue_job(), &mut self.context)
                .unwrap();
        }
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        root.link(self.context.access()).unwrap();
        assert!(self.executor.queue.borrow().is_empty());
        assert!(self.executor.observer.borrow().is_none());
    }
    fn begin_unscoped(&mut self, root: &Module) -> (JsPromise, JsFuture) {
        let adapter = root.load_link_evaluate(self.context.access()).unwrap();
        let mut wait = adapter
            .clone()
            .into_js_future(self.context.access())
            .unwrap();
        assert!(poll(&mut wait).is_pending());
        let mut error = None;
        // The finite load/link/evaluate wrappers are real. In particular, this
        // initiating evaluation callback has NO selected Execution owner.
        for _ in 0..16 {
            match GraphExecutor::invoke(self.executor.queue_job(), self.context.access()) {
                Ok(_) => assert!(self.executor.saved.borrow().is_none()),
                Err(actual) => {
                    error = Some(actual);
                    break;
                }
            }
        }
        assert_eq!(error, Some(self.original.clone()));
        assert!(GRAPH_HOOK.with(|slot| slot.borrow().is_none()));
        assert!(self.executor.hook.borrow().is_none());
        assert!(self.executor.saved.borrow().is_some());
        (adapter, wait)
    }
    fn count(&mut self, name: &str) -> JsValue {
        self.context
            .global_object()
            .get(surgeist_js::JsString::from(name), self.context.access())
            .unwrap()
    }
    fn evaluation(&mut self, module: &Module) -> JsPromise {
        self.context
            .with_execution(ExecutionOptions::new(FRESH), |access| {
                module.evaluate(access)
            })
            .unwrap()
            .result
            .unwrap()
    }
    fn interrupted(&mut self, module: &Module) -> ModuleInterruption {
        let outcome = self
            .context
            .with_execution(ExecutionOptions::new(FRESH), |access| {
                module.evaluate(access)
            })
            .unwrap();
        assert_eq!(outcome.report.instructions().unwrap(), 0);
        self.cause(&outcome.result.unwrap_err())
    }
    fn cause(&self, error: &JsError) -> ModuleInterruption {
        let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() else {
            panic!("actual graph interruption must retain its typed module cause")
        };
        let DiagnosticKind::Engine(DiagnosticEngine::Panic {
            message,
            diagnostic: Some(source),
        }) = interruption.cause().kind()
        else {
            panic!("the first authored Panic remains the graph cause")
        };
        assert_eq!(message, "authored graph post-storage failure");
        let DiagnosticKind::Native(native) = source.kind() else {
            panic!("the exact authored native source remains visible")
        };
        assert_eq!(native.kind(), DiagnosticNativeKind::Range);
        assert_eq!(native.message(), "graph retained cause");
        let DiagnosticKind::Opaque(leaf) = native.cause().unwrap().kind() else {
            panic!("the original local opaque leaf remains visible")
        };
        assert_eq!(leaf.value().unwrap(), self.opaque);
        interruption.clone()
    }
    fn finished(&mut self, module: &Module) -> JsPromise {
        let result = self.evaluation(module);
        assert_eq!(
            result.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        assert_eq!(self.count("graphFinished"), JsValue::new(1));
        result
    }
}
fn poll(wait: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn graph_modules(
    graph: &mut Graph,
    cycle: bool,
    sibling: bool,
) -> (Module, Module, Module, Module) {
    let finished = graph.insert(
        "finished",
        "globalThis.graphFinished += 1; export const fixed = 2;",
    );
    let child = graph.insert(
        "child",
        if cycle {
            "import 'root'; armGraphPostStoreHook(); globalThis.graphChild += 1;
         await 0; globalThis.graphChild += 10; export const value = 40;"
        } else {
            "armGraphPostStoreHook(); globalThis.graphChild += 1;
         await 0; globalThis.graphChild += 10; export const value = 40;"
        },
    );
    let sibling_module = graph.insert("sibling", "globalThis.graphSibling += 1;");
    let source = if sibling {
        "import {fixed} from 'finished'; import {value} from 'child'; import 'sibling';
         globalThis.graphParent += 1; globalThis.graphResult = value + fixed;"
    } else {
        "import {fixed} from 'finished'; import {value} from 'child';
         globalThis.graphParent += 1; globalThis.graphResult = value + fixed;"
    };
    let root = graph.insert("root", source);
    graph.ready(&root);
    (root, child, sibling_module, finished)
}

#[test]
fn an_unscoped_child_post_store_error_still_publishes_the_parent_dependency_before_returning() {
    let mut graph = Graph::new();
    let (root, child, _, finished) = graph_modules(&mut graph, false, false);
    let (adapter, mut wait) = graph.begin_unscoped(&root);
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    assert!(poll(&mut wait).is_pending());
    assert_eq!(graph.count("graphChild"), JsValue::new(1));
    assert_eq!(graph.count("graphParent"), JsValue::new(0));
    let parent_evaluation = graph.evaluation(&root);
    let child_evaluation = graph.evaluation(&child);
    assert_eq!(parent_evaluation.state().unwrap(), PromiseState::Pending);
    assert_eq!(child_evaluation.state().unwrap(), PromiseState::Pending);
    let finished_before = graph.finished(&finished);
    let observer = graph.executor.observer();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    graph
        .executor
        .fresh(graph.executor.saved_job(), &mut graph.context)
        .unwrap();
    graph.executor.finish(&adapter, &mut graph.context);
    assert_eq!(poll(&mut wait), Poll::Ready(Ok(JsValue::undefined())));
    assert_eq!(
        adapter.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        parent_evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        child_evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(graph.count("graphChild"), JsValue::new(11));
    assert_eq!(graph.count("graphParent"), JsValue::new(1));
    assert_eq!(graph.count("graphResult"), JsValue::new(42));
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(
        graph.evaluation(&root).identity(),
        parent_evaluation.identity()
    );
    assert_eq!(
        graph.evaluation(&child).identity(),
        child_evaluation.identity()
    );
    assert_eq!(
        graph.finished(&finished).identity(),
        finished_before.identity()
    );
}

#[test]
fn a_closed_child_survives_its_importers_later_sibling_stop_and_the_sibling_stays_unentered() {
    let mut graph = Graph::new();
    let (root, child, sibling, finished) = graph_modules(&mut graph, false, true);
    let (adapter, mut wait) = graph.begin_unscoped(&root);
    assert_eq!(graph.count("graphChild"), JsValue::new(1));
    assert_eq!(graph.count("graphParent"), JsValue::new(0));
    assert_eq!(graph.count("graphSibling"), JsValue::new(0));
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    let Poll::Ready(Err(observed)) = poll(&mut wait) else {
        panic!("the original importer native wait must observe its actual later dependency refusal")
    };
    let original = graph.cause(&observed);
    assert_eq!(graph.interrupted(&root).identity(), original.identity());
    let child_evaluation = graph.evaluation(&child);
    assert_eq!(child_evaluation.state().unwrap(), PromiseState::Pending);
    let mut child_wait = child_evaluation
        .clone()
        .into_js_future(graph.context.access())
        .unwrap();
    assert!(poll(&mut child_wait).is_pending());
    let finished_before = graph.finished(&finished);
    let observer = graph.executor.observer();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    graph
        .executor
        .fresh(graph.executor.saved_job(), &mut graph.context)
        .unwrap();
    graph.executor.finish(&child_evaluation, &mut graph.context);
    assert_eq!(poll(&mut child_wait), Poll::Ready(Ok(JsValue::undefined())));
    assert_eq!(graph.count("graphChild"), JsValue::new(11));
    assert_eq!(observer.status(), Ok(JobStatus::Completed));
    assert_eq!(observer.fault(), None);
    assert_eq!(graph.count("graphParent"), JsValue::new(0));
    assert_eq!(graph.count("graphSibling"), JsValue::new(0));
    assert_eq!(graph.interrupted(&root).identity(), original.identity());
    assert_eq!(
        graph.evaluation(&child).identity(),
        child_evaluation.identity()
    );
    assert_eq!(
        graph.finished(&finished).identity(),
        finished_before.identity()
    );
    // The next child was refused before entry, rather than terminally poisoned.
    let loaded = sibling.load(graph.context.access()).unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    sibling.link(graph.context.access()).unwrap();
    let sibling_evaluation = graph.evaluation(&sibling);
    assert_eq!(
        sibling_evaluation.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(graph.count("graphSibling"), JsValue::new(1));
    assert_eq!(
        graph.evaluation(&sibling).identity(),
        sibling_evaluation.identity()
    );
    assert_eq!(graph.count("graphSibling"), JsValue::new(1));
    assert_eq!(graph.interrupted(&root).identity(), original.identity());
}

#[test]
fn an_open_cycle_is_invalidated_by_the_later_required_sibling_refusal_despite_job_acceptance() {
    let mut graph = Graph::new();
    let (root, child, sibling, finished) = graph_modules(&mut graph, true, true);
    let (adapter, mut wait) = graph.begin_unscoped(&root);
    assert_eq!(adapter.state().unwrap(), PromiseState::Pending);
    let Poll::Ready(Err(observed)) = poll(&mut wait) else {
        panic!("the original open-component native wait must receive its actual interruption")
    };
    let cause = graph.cause(&observed);
    assert_eq!(graph.interrupted(&root).identity(), cause.identity());
    assert_eq!(graph.interrupted(&child).identity(), cause.identity());
    let finished_before = graph.finished(&finished);
    assert_eq!(graph.count("graphChild"), JsValue::new(1));
    assert_eq!(graph.count("graphParent"), JsValue::new(0));
    assert_eq!(graph.count("graphSibling"), JsValue::new(0));
    let observer = graph.executor.observer();
    let stopped = graph
        .executor
        .fresh(graph.executor.saved_job(), &mut graph.context)
        .unwrap_err();
    assert_eq!(graph.cause(&stopped).identity(), cause.identity());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Interrupted))
    );
    assert_eq!(observer.fault(), None);
    assert_eq!(graph.count("graphChild"), JsValue::new(1));
    assert_eq!(graph.count("graphParent"), JsValue::new(0));
    assert_eq!(graph.interrupted(&root).identity(), cause.identity());
    assert_eq!(graph.interrupted(&child).identity(), cause.identity());
    assert_eq!(
        graph.finished(&finished).identity(),
        finished_before.identity()
    );
    // The unvisited sibling is still independently usable after the owner exits.
    assert_eq!(
        sibling
            .load(graph.context.access())
            .unwrap()
            .state()
            .unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    sibling.link(graph.context.access()).unwrap();
    assert_eq!(
        graph.evaluation(&sibling).state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(graph.count("graphSibling"), JsValue::new(1));
    assert_eq!(graph.interrupted(&root).identity(), cause.identity());
}
