use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context as TaskContext, Poll, Wake, Waker},
};
use surgeist_js::{
    Context, ContextAccess, JsError, JsResult, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{DiagnosticEngine, DiagnosticKind, EngineError, ModuleInterruption, RuntimeLimitError},
    execution::ExecutionOptions,
    job::{Job, JobAcceptance, JobCall, JobExecutor, JobPublicationError, JobStep},
    js_string,
    module::MapModuleLoader,
    object::builtins::{JsFuture, JsPromise},
};

const STOP_LIMIT: u64 = 64;
const FRESH_LIMIT: u64 = 100_000;

#[derive(Default)]
struct ManualExecutor {
    jobs: RefCell<VecDeque<Job>>,
}
impl JobExecutor for ManualExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.jobs.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("this fixture explicitly drives its real loader and reaction jobs")
    }
}

struct Graph {
    context: Context,
    executor: Rc<ManualExecutor>,
    loader: Rc<MapModuleLoader>,
}
impl Graph {
    fn new() -> Self {
        let executor = Rc::new(ManualExecutor::default());
        let loader = Rc::new(MapModuleLoader::new());
        let builder = Context::builder()
            .job_executor(executor.clone())
            .module_loader(loader.clone());
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        for name in [
            "childEffects",
            "parentEffects",
            "grandEffects",
            "topEffects",
        ] {
            context
                .global_object()
                .set(js_string!(name), 0, true, context.access())
                .unwrap();
        }
        Self {
            context,
            executor,
            loader,
        }
    }
    fn insert(&mut self, name: &str, source: &str) -> Module {
        let module = Module::parse(
            Source::from_bytes(source).with_path(std::path::Path::new(name)),
            None,
            self.context.access(),
        )
        .unwrap();
        assert!(self.loader.insert(name, module.clone()).is_none());
        module
    }
    fn load_and_link(&mut self, module: &Module) {
        let loaded = module.load(self.context.access()).unwrap();
        for _ in 0..128 {
            if loaded.state().unwrap() != PromiseState::Pending {
                break;
            }
            let queued = self.executor.jobs.borrow_mut().pop_front();
            let Some(Job::AsyncJob(mut job)) = queued else {
                panic!("the map loader must supply its actual native task: {queued:?}")
            };
            let host = self.context.host_access();
            let mut task = TaskContext::from_waker(Waker::noop());
            assert!(matches!(job.run_step(&mut task, &host), JobStep::Prepared));
            let JobStep::Complete(result) = job.run_step(&mut task, &host) else {
                panic!("the actual in-memory map loader is immediately ready")
            };
            result.unwrap();
        }
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(JsValue::undefined())
        );
        module.link(self.context.access()).unwrap();
    }
    fn evaluate(&mut self, module: &Module) -> JsPromise {
        self.context
            .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
                module.evaluate(access)
            })
            .unwrap()
            .result
            .unwrap()
    }
    fn count(&mut self, name: &str) -> JsValue {
        self.context
            .global_object()
            .get(js_string!(name), self.context.access())
            .unwrap()
    }
    fn assert_no_replay(&mut self) {
        assert_eq!(self.count("childEffects"), 1.into());
        for name in ["parentEffects", "grandEffects", "topEffects"] {
            assert_eq!(self.count(name), 0.into());
        }
    }
}

#[derive(Debug)]
struct FirstNoticePanic;
#[derive(Debug)]
struct SecondaryDisposalPanic;

struct SecondaryNoticePanic {
    drops: Arc<AtomicUsize>,
    disposal_panic: Arc<SecondaryDisposalPanic>,
}
impl Drop for SecondaryNoticePanic {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
        panic_any(self.disposal_panic.clone());
    }
}

struct NoticePlan {
    armed: AtomicBool,
    unique_notices: AtomicUsize,
    calls: [AtomicUsize; 3],
    passive_notices: AtomicUsize,
    secondary_drops: Arc<AtomicUsize>,
    first: Arc<FirstNoticePanic>,
    secondary_disposal: Arc<SecondaryDisposalPanic>,
}
impl NoticePlan {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            armed: AtomicBool::new(false),
            unique_notices: AtomicUsize::new(0),
            calls: std::array::from_fn(|_| AtomicUsize::new(0)),
            passive_notices: AtomicUsize::new(0),
            secondary_drops: Arc::new(AtomicUsize::new(0)),
            first: Arc::new(FirstNoticePanic),
            secondary_disposal: Arc::new(SecondaryDisposalPanic),
        })
    }
    fn notify(&self, waiter: usize) {
        if !self.armed.load(Ordering::Acquire) {
            return;
        }
        // Choose behavior by the first actual notification of each independent
        // parked waiter. No module-name or traversal-order assumption is used;
        // duplicate/spurious wakes do not consume another waiter's role.
        if self.calls[waiter].fetch_add(1, Ordering::Relaxed) != 0 {
            return;
        }
        match self.unique_notices.fetch_add(1, Ordering::AcqRel) {
            0 => panic_any(self.first.clone()),
            1 => panic_any(SecondaryNoticePanic {
                drops: self.secondary_drops.clone(),
                disposal_panic: self.secondary_disposal.clone(),
            }),
            _ => {
                self.passive_notices.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
struct NumberedWake {
    waiter: usize,
    plan: Arc<NoticePlan>,
}
impl Wake for NumberedWake {
    fn wake(self: Arc<Self>) {
        self.plan.notify(self.waiter);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.plan.notify(self.waiter);
    }
}
struct NativeWait {
    future: Pin<Box<JsFuture>>,
    wake: Arc<NumberedWake>,
}
impl NativeWait {
    fn new(
        promise: &JsPromise,
        waiter: usize,
        plan: Arc<NoticePlan>,
        context: &mut Context,
    ) -> Self {
        let mut wait = Self {
            future: Box::pin(promise.clone().into_js_future(context.access()).unwrap()),
            wake: Arc::new(NumberedWake { waiter, plan }),
        };
        assert!(wait.poll().is_pending());
        wait
    }
    fn poll(&mut self) -> Poll<JsResult<JsValue>> {
        self.future
            .as_mut()
            .poll(&mut TaskContext::from_waker(&Waker::from(
                self.wake.clone(),
            )))
    }
}

fn interruption(error: &JsError) -> &ModuleInterruption {
    let Some(EngineError::ModuleInterrupted(interruption)) = error.as_engine() else {
        panic!("native observation must retain the actual module interruption: {error:?}")
    };
    assert_eq!(
        interruption.cause().kind(),
        DiagnosticKind::Engine(DiagnosticEngine::RuntimeLimit(
            RuntimeLimitError::Instruction { limit: STOP_LIMIT }
        )),
    );
    interruption
}

#[test]
fn transitive_module_notices_finish_after_secondary_payload_drop_panics_and_keep_first_unwind() {
    let mut graph = Graph::new();
    let (gate, resolvers) = JsPromise::new_pending(graph.context.access());
    graph
        .context
        .global_object()
        .set(
            js_string!("childGate"),
            gate.clone(),
            true,
            graph.context.access(),
        )
        .unwrap();
    // Preserve the established unrequested-child fixture: the child is entered
    // through imports. Three ancestors are explicitly evaluated, each issuing
    // its own canonical evaluation promise/native signal before interruption.
    let child = graph.insert(
        "child",
        "await childGate; globalThis.childEffects += 1; for (let step = 0; step < 1000; step++) {}",
    );
    let parent = graph.insert("parent", "import 'child'; globalThis.parentEffects += 1;");
    let grandparent = graph.insert(
        "grandparent",
        "import 'parent'; globalThis.grandEffects += 1;",
    );
    let top = graph.insert("top", "import 'grandparent'; globalThis.topEffects += 1;");
    graph.load_and_link(&top);
    let evaluations = [
        graph.evaluate(&parent),
        graph.evaluate(&grandparent),
        graph.evaluate(&top),
    ];
    for (index, evaluation) in evaluations.iter().enumerate() {
        assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
        for previous in &evaluations[..index] {
            assert_ne!(
                evaluation.identity(),
                previous.identity(),
                "independent canonical module promises"
            );
        }
    }
    let plan = NoticePlan::new();
    let mut waits: Vec<_> = evaluations
        .iter()
        .enumerate()
        .map(|(index, evaluation)| {
            NativeWait::new(evaluation, index, plan.clone(), &mut graph.context)
        })
        .collect();
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[1.into()], graph.context.access())
        .unwrap();
    assert_eq!(gate.state().unwrap(), PromiseState::Fulfilled(1.into()));
    for wait in &mut waits {
        assert!(wait.poll().is_pending());
    }
    assert_eq!(plan.unique_notices.load(Ordering::Acquire), 0);
    assert_eq!(graph.count("childEffects"), 0.into());
    let queued = graph.executor.jobs.borrow_mut().pop_front();
    let Some(Job::PromiseJob(job)) = queued else {
        panic!("gate settlement must supply its actual Await reaction: {queued:?}")
    };
    plan.armed.store(true, Ordering::Release);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _outcome = graph
            .context
            .with_execution(ExecutionOptions::new(STOP_LIMIT), |access| {
                match job.call(access) {
                    JobCall::Complete(result) => result,
                    JobCall::Deferred { job, error } => {
                        graph
                            .executor
                            .jobs
                            .borrow_mut()
                            .push_front(Job::PromiseJob(job));
                        Err(error)
                    }
                }
            })
            .unwrap();
    }));
    let Err(payload) = unwind else {
        panic!("the authored first notification panic must escape")
    };
    let actual = payload
        .downcast_ref::<Arc<FirstNoticePanic>>()
        .expect("the initiating wake panic must survive secondary-payload disposal");
    assert!(Arc::ptr_eq(actual, &plan.first));
    assert_eq!(
        plan.secondary_drops.load(Ordering::Relaxed),
        1,
        "dispose the actual later panic payload once"
    );
    assert_eq!(
        plan.unique_notices.load(Ordering::Acquire),
        3,
        "finish all three independently parked module notices"
    );
    assert_eq!(
        plan.passive_notices.load(Ordering::Relaxed),
        1,
        "the notice after the secondary disposal must still run"
    );
    for calls in &plan.calls {
        assert!(calls.load(Ordering::Relaxed) > 0);
    }

    let mut errors = Vec::new();
    for wait in &mut waits {
        let Poll::Ready(Err(error)) = wait.poll() else {
            panic!("every originally parked native future must observe interruption")
        };
        interruption(&error);
        errors.push(error);
    }
    let identity = interruption(&errors[0]).identity();
    for error in &errors {
        assert_eq!(interruption(error).identity(), identity);
    }
    for evaluation in &evaluations {
        assert_eq!(evaluation.state().unwrap(), PromiseState::Pending);
    }
    graph.assert_no_replay();
    for module in [&child, &parent, &grandparent, &top] {
        let later = graph
            .context
            .with_execution(ExecutionOptions::new(FRESH_LIMIT), |access| {
                module.evaluate(access)
            })
            .unwrap();
        assert_eq!(later.report.instructions().unwrap(), 0);
        let error = later.result.unwrap_err();
        assert_eq!(interruption(&error).identity(), identity);
    }
    let fresh = graph
        .context
        .eval_with_budget(
            Source::from_bytes("20 + 22"),
            ExecutionOptions::new(FRESH_LIMIT),
        )
        .unwrap();
    assert_eq!(
        fresh.result.unwrap(),
        42.into(),
        "the selected Context and VM are restored after host unwind"
    );
    graph.assert_no_replay();
    assert_eq!(plan.secondary_drops.load(Ordering::Relaxed), 1);
}
