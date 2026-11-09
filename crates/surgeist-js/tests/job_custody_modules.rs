#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex, atomic::Ordering},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::{Duration, Instant},
};

use surgeist_js::{
    Context, ContextAccess, JsResult, JsString, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::EngineError,
    job::{
        ContextTask, Job, JobAcceptance, JobCall, JobExecutor, JobObserver, JobPublicationCause,
        JobPublicationError, JobRetirement, JobStatus, JobStep, NativeAsyncJob, PromiseJob,
        SimpleJobExecutor,
    },
    js_string,
    module::{ModuleLoader, ModuleRequest, Referrer},
    object::builtins::{JsFuture, JsPromise},
};

// This executor deliberately leaves scheduling to the test. It retains the real
// public job wrappers, so selected disposal and dispatch use their actual owners.
// No queue borrow spans a job call, task step, capture drop or native wake.
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
        panic!("this fixture dispatches captured jobs explicitly")
    }
}

impl ManualExecutor {
    fn take_async(&self) -> NativeAsyncJob {
        let job = self.jobs.borrow_mut().pop_front();
        match job {
            Some(Job::AsyncJob(job)) => job,
            other => panic!("expected the next authored native producer, got {other:?}"),
        }
    }

    fn take_promise(&self) -> PromiseJob {
        let job = self.jobs.borrow_mut().pop_front();
        match job {
            Some(Job::PromiseJob(job)) => job,
            other => panic!("expected the next selected promise job, got {other:?}"),
        }
    }
}

fn context_with_manual_executor() -> (Context, Rc<ManualExecutor>) {
    let executor = Rc::new(ManualExecutor::default());
    let context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    (context, executor)
}

fn ready_promise(value: JsValue, context: &mut Context) -> JsPromise {
    JsPromise::from_task(
        move |_| Ok(ContextTask::from_future(std::future::ready(Ok(value)))),
        context.access(),
    )
    .unwrap()
}

fn step(job: &mut NativeAsyncJob, context: &mut Context) -> JobStep {
    job.run_step(
        &mut TaskContext::from_waker(Waker::noop()),
        &context.host_access(),
    )
}

fn prepare(job: &mut NativeAsyncJob, context: &mut Context) {
    assert!(matches!(step(job, context), JobStep::Prepared));
}

fn complete_ready(job: &mut NativeAsyncJob, context: &mut Context) -> JobObserver {
    let observer = job.observer();
    prepare(job, context);
    let JobStep::Complete(result) = step(job, context) else {
        panic!("the authored ready task must deliver its actual terminal result")
    };
    result.unwrap();
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    observer
}

fn invoke(job: PromiseJob, context: &mut Context) -> JobObserver {
    let observer = job.observer();
    match job.call(context.access()) {
        JobCall::Complete(result) => {
            result.unwrap();
        }
        JobCall::Deferred { job, error } => {
            drop(job);
            panic!("unbounded explicit promise dispatch was deferred: {error:?}")
        }
    }
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    observer
}

#[derive(Default)]
struct WakeCount {
    count: Mutex<usize>,
    changed: Condvar,
}

impl WakeCount {
    fn record(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        drop(count);
        self.changed.notify_all();
    }

    fn reset(&self) {
        *self.count.lock().unwrap() = 0;
    }

    fn await_fresh_notification(&self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut count = self.count.lock().unwrap();
        while *count == 0 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "the original native waiter received no fresh wake within 5 seconds; last count={}",
                *count
            );
            (count, _) = self.changed.wait_timeout(count, remaining).unwrap();
        }
        assert!(*count > 0, "the original native waiter must be notified");
    }
}

impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.record();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.record();
    }
}

struct NativeWait {
    future: Pin<Box<JsFuture>>,
    wakes: Arc<WakeCount>,
}

impl NativeWait {
    fn subscribe_before_dispatch(promise: &JsPromise, context: &mut Context) -> Self {
        let mut wait = Self {
            future: Box::pin(promise.clone().into_js_future(context.access()).unwrap()),
            wakes: Arc::new(WakeCount::default()),
        };
        wait.assert_pending();
        wait
    }

    fn poll(&mut self) -> Poll<JsResult<JsValue>> {
        let waker = Waker::from(self.wakes.clone());
        self.future
            .as_mut()
            .poll(&mut TaskContext::from_waker(&waker))
    }

    fn assert_pending(&mut self) {
        assert!(
            self.poll().is_pending(),
            "required settlement is still live"
        );
        // A preceding handoff wake cannot substitute for notification after
        // this poll has rearmed the original waiter for its next transition.
        self.wakes.reset();
    }

    fn assert_retired(&mut self, reason: JobRetirement) {
        self.wakes.await_fresh_notification();
        let Poll::Ready(Err(error)) = self.poll() else {
            panic!("retirement must terminate the original native wait")
        };
        assert_eq!(error.as_engine(), Some(&EngineError::JobRetired(reason)));
    }

    fn assert_fulfilled(&mut self, value: JsValue) {
        let Poll::Ready(result) = self.poll() else {
            panic!("actual guest settlement must complete the native wait")
        };
        assert_eq!(result.unwrap(), value);
    }
}

fn watch_guest_rejection(promise: &JsPromise, context: &mut Context) {
    context
        .global_object()
        .set(
            js_string!("custodySubject"),
            promise.clone(),
            true,
            context.access(),
        )
        .unwrap();
    context
        .eval(Source::from_bytes(
            "globalThis.custodyRejected = false;
             custodySubject.catch(() => { custodyRejected = true; });",
        ))
        .unwrap();
}

fn assert_guest_pending(promise: &JsPromise, context: &mut Context) {
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        context.eval(Source::from_bytes("custodyRejected")).unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn original_native_waiter_observes_drop_of_unstarted_assimilation_after_producer_completed() {
    let (mut context, executor) = context_with_manual_executor();
    let (source, _resolvers) = JsPromise::new_pending(context.access());
    let target = ready_promise(source.into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);

    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation = executor.take_promise();
    let assimilation_status = assimilation.observer();
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Pending);
    assert_guest_pending(&target, &mut context);
    wait.assert_pending();

    drop(assimilation);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        assimilation_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_guest_pending(&target, &mut context);
}

#[test]
fn former_producer_owner_loss_does_not_retire_its_unstarted_assimilation_successor() {
    let (mut context, executor) = context_with_manual_executor();
    let mut peer = Context::builder()
        .runtime(context.runtime().clone())
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let (source, _resolvers) = JsPromise::new_pending(context.access());
    let target = ready_promise(source.into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation = executor.take_promise();

    drop(context);
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(target.state().unwrap(), PromiseState::Pending);
    wait.assert_pending();
    // The successor was never prepared on the former owner. It remains real
    // unbound queue work and can register its guest-only source on a peer.
    let assimilation_status = invoke(assimilation, &mut peer);
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(target.state().unwrap(), PromiseState::Pending);
    wait.assert_pending();
}

#[test]
fn required_native_source_retirement_reaches_waiter_after_both_predecessors_completed() {
    let (mut context, executor) = context_with_manual_executor();
    let source = JsPromise::from_task(
        |_| Ok(ContextTask::from_future(std::future::pending())),
        context.access(),
    )
    .unwrap();
    let source_job = executor.take_async();
    let source_status = source_job.observer();
    let target = ready_promise(source.clone().into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation_status = invoke(executor.take_promise(), &mut context);
    wait.assert_pending();

    drop(source_job);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        source_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_eq!(source.state().unwrap(), PromiseState::Pending);
    assert_guest_pending(&target, &mut context);
}

#[test]
fn drop_of_selected_unstarted_reaction_retires_target_without_rejecting_settled_source() {
    let (mut context, executor) = context_with_manual_executor();
    let (source, resolvers) = JsPromise::new_pending(context.access());
    let target = ready_promise(source.clone().into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation_status = invoke(executor.take_promise(), &mut context);
    wait.assert_pending();
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[42.into()], context.access())
        .unwrap();
    assert_eq!(source.state().unwrap(), PromiseState::Fulfilled(42.into()));
    let reaction = executor.take_promise();
    let reaction_status = reaction.observer();
    assert_eq!(reaction_status.status().unwrap(), JobStatus::Pending);

    drop(reaction);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(source.state().unwrap(), PromiseState::Fulfilled(42.into()));
    assert_guest_pending(&target, &mut context);
}

#[test]
fn completing_native_source_and_selected_reaction_fulfills_the_original_native_waiter() {
    let (mut context, executor) = context_with_manual_executor();
    let source = ready_promise(42.into(), &mut context);
    let mut source_job = executor.take_async();
    let target = ready_promise(source.clone().into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation_status = invoke(executor.take_promise(), &mut context);
    wait.assert_pending();

    let source_status = complete_ready(&mut source_job, &mut context);
    assert_eq!(source.state().unwrap(), PromiseState::Fulfilled(42.into()));
    assert_eq!(target.state().unwrap(), PromiseState::Pending);
    let reaction_status = invoke(executor.take_promise(), &mut context);
    wait.assert_fulfilled(42.into());
    for observer in [
        producer_status,
        assimilation_status,
        source_status,
        reaction_status,
    ] {
        assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    }
    assert_eq!(target.state().unwrap(), PromiseState::Fulfilled(42.into()));
}

#[test]
fn already_settled_target_remains_native_ready_when_a_later_delivery_job_is_dropped() {
    let (mut context, executor) = context_with_manual_executor();
    let target = ready_promise(42.into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    assert_eq!(target.state().unwrap(), PromiseState::Fulfilled(42.into()));

    let later_delivery = executor.take_promise();
    let delivery_status = later_delivery.observer();
    drop(later_delivery);
    assert_eq!(
        delivery_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    wait.assert_fulfilled(42.into());
    assert_eq!(target.state().unwrap(), PromiseState::Fulfilled(42.into()));
}

#[test]
fn normally_completed_unresolved_thenable_retains_a_live_external_resolution_obligation() {
    let (mut context, executor) = context_with_manual_executor();
    let thenable = context
        .eval(Source::from_bytes(
            "globalThis.thenableEntered = false;
             ({ then(resolve, reject) { thenableEntered = true; } })",
        ))
        .unwrap();
    let target = ready_promise(thenable, &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let assimilation_status = invoke(executor.take_promise(), &mut context);
    assert_eq!(
        context.eval(Source::from_bytes("thenableEntered")).unwrap(),
        JsValue::new(true)
    );
    context.runtime().collect().unwrap();

    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Completed);
    wait.assert_pending();
    assert_guest_pending(&target, &mut context);
}

#[test]
fn late_native_adoption_retains_an_already_queued_guest_assimilation_obligation() {
    let (mut context, executor) = context_with_manual_executor();
    let thenable = context
        .eval(Source::from_bytes(
            "globalThis.lateOuterEntered = false;
             globalThis.lateInnerEntered = false;
             ({ then(resolve, reject) {
                 lateOuterEntered = true;
                 const source = Promise.resolve({ then(_) { lateInnerEntered = true; } });
                 globalThis.lateSource = source;
                 source.then(resolve, reject);
             } })",
        ))
        .unwrap();
    let target = ready_promise(thenable, &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let outer_status = invoke(executor.take_promise(), &mut context);
    assert_eq!(
        context
            .eval(Source::from_bytes("lateOuterEntered"))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context
            .eval(Source::from_bytes("lateInnerEntered"))
            .unwrap(),
        JsValue::new(false)
    );
    let source = JsPromise::from_object(
        context
            .eval(Source::from_bytes("lateSource"))
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(source.state().unwrap(), PromiseState::Pending);
    assert_guest_pending(&target, &mut context);
    wait.assert_pending();

    // Promise.resolve queued this real job before source.then installed the
    // resolving handlers that make its completion necessary for the target.
    let inner_assimilation = executor.take_promise();
    let inner_status = inner_assimilation.observer();
    assert_eq!(inner_status.status().unwrap(), JobStatus::Pending);
    drop(inner_assimilation);

    assert_eq!(
        inner_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(outer_status.status().unwrap(), JobStatus::Completed);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(source.state().unwrap(), PromiseState::Pending);
    assert_guest_pending(&target, &mut context);
    assert_eq!(
        context
            .eval(Source::from_bytes("lateInnerEntered"))
            .unwrap(),
        JsValue::new(false)
    );
}

#[test]
fn late_adoption_preserves_reaction_pair_after_original_assimilation_completed() {
    let (mut context, executor) = context_with_manual_executor();
    let (root, root_resolvers) = JsPromise::new_pending(context.access());
    context
        .global_object()
        .set(
            js_string!("adoptionRoot"),
            root.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let adopted = JsPromise::from_object(
        context
            .eval(Source::from_bytes(
                "globalThis.completedAssimilations = 0;
                 Promise.resolve({ then(resolve, reject) {
                     completedAssimilations += 1;
                     adoptionRoot.then(resolve, reject);
                 } })",
            ))
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    // Complete the guest-created source's assimilation before the native target
    // exists. Its live obligation is now the pending root's reaction pair.
    let initial_status = invoke(executor.take_promise(), &mut context);
    assert_eq!(adopted.state().unwrap(), PromiseState::Pending);
    assert_eq!(root.state().unwrap(), PromiseState::Pending);
    assert_eq!(
        context
            .eval(Source::from_bytes("completedAssimilations"))
            .unwrap(),
        JsValue::new(1)
    );

    let target = ready_promise(adopted.clone().into(), &mut context);
    let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
    watch_guest_rejection(&target, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    let adoption_status = invoke(executor.take_promise(), &mut context);
    wait.assert_pending();
    root_resolvers
        .resolve
        .call(&JsValue::undefined(), &[42.into()], context.access())
        .unwrap();
    assert_eq!(root.state().unwrap(), PromiseState::Fulfilled(42.into()));
    assert_eq!(adopted.state().unwrap(), PromiseState::Pending);
    assert_guest_pending(&target, &mut context);
    wait.assert_pending();

    let selected = executor.take_promise();
    let selected_status = selected.observer();
    assert_eq!(selected_status.status().unwrap(), JobStatus::Pending);
    drop(selected);

    assert_eq!(
        selected_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    for observer in [initial_status, producer_status, adoption_status] {
        assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    }
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(root.state().unwrap(), PromiseState::Fulfilled(42.into()));
    assert_eq!(adopted.state().unwrap(), PromiseState::Pending);
    assert_guest_pending(&target, &mut context);
    assert_eq!(
        context
            .eval(Source::from_bytes("completedAssimilations"))
            .unwrap(),
        JsValue::new(1)
    );
}

#[test]
fn explicit_executor_cancellation_terminates_native_wait_without_guest_rejection_or_extra_poll() {
    for started in [false, true] {
        let executor = Rc::new(SimpleJobExecutor::new());
        let mut context = Context::builder()
            .job_executor(executor.clone())
            .build()
            .unwrap();
        let factories = Rc::new(Cell::new(0));
        let polls = Rc::new(Cell::new(0));
        let factory_entries = factories.clone();
        let task_polls = polls.clone();
        let target = JsPromise::from_task(
            move |_| {
                factory_entries.set(factory_entries.get() + 1);
                Ok(ContextTask::from_poll(move |_, _| {
                    task_polls.set(task_polls.get() + 1);
                    Poll::Pending
                }))
            },
            context.access(),
        )
        .unwrap();
        let mut wait = NativeWait::subscribe_before_dispatch(&target, &mut context);
        watch_guest_rejection(&target, &mut context);
        if started {
            let holder = context.host_access();
            let mut drain = Box::pin(executor.clone().run_jobs_async(&holder));
            assert!(
                drain
                    .as_mut()
                    .poll(&mut TaskContext::from_waker(Waker::noop()))
                    .is_pending()
            );
            assert_eq!(factories.get(), 1);
            assert!(polls.get() > 0, "the retained native task actually started");
            drop(drain);
            drop(holder);
        }
        wait.assert_pending();
        assert_guest_pending(&target, &mut context);
        let polls_before_cancellation = polls.get();
        executor
            .get_cancellation_token()
            .store(true, Ordering::Relaxed);

        // This is the actual ordinary executor result, without a scope finalizer
        // capable of replacing the result returned by the driver.
        context.run_jobs().unwrap();
        wait.assert_retired(JobRetirement::Cancelled);
        assert_eq!(factories.get(), usize::from(started));
        assert_eq!(polls.get(), polls_before_cancellation);
        assert_guest_pending(&target, &mut context);
    }
}

#[derive(Default)]
struct FixtureLoader {
    modules: RefCell<Vec<(JsString, Module)>>,
    requests: RefCell<Vec<JsString>>,
}

impl ModuleLoader for FixtureLoader {
    fn load_imported_module(
        self: Rc<Self>,
        _: Referrer,
        request: ModuleRequest,
    ) -> ContextTask<Module> {
        let name = request.specifier().clone();
        self.requests.borrow_mut().push(name.clone());
        let module = self
            .modules
            .borrow()
            .iter()
            .find(|(specifier, _)| *specifier == name)
            .map(|(_, module)| module.clone())
            .expect("the fixture supplies every requested module");
        ContextTask::from_future(std::future::ready(Ok(module)))
    }
}

fn graph_fixture() -> (Context, Rc<ManualExecutor>, Rc<FixtureLoader>, JsPromise) {
    let executor = Rc::new(ManualExecutor::default());
    let loader = Rc::new(FixtureLoader::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone())
        .build()
        .unwrap();
    for name in [js_string!("first"), js_string!("second")] {
        let module =
            Module::parse(Source::from_bytes("export {};"), None, context.access()).unwrap();
        loader.modules.borrow_mut().push((name, module));
    }
    let graph = Module::parse(
        Source::from_bytes("import 'first'; import 'second';"),
        None,
        context.access(),
    )
    .unwrap();
    let promise = graph.load(context.access()).unwrap();
    (context, executor, loader, promise)
}

#[test]
fn one_loader_completion_does_not_settle_graph_and_dropping_the_other_wakes_original_waiter() {
    let (mut context, executor, loader, graph) = graph_fixture();
    let mut wait = NativeWait::subscribe_before_dispatch(&graph, &mut context);
    watch_guest_rejection(&graph, &mut context);
    let mut first = executor.take_async();
    let first_status = complete_ready(&mut first, &mut context);
    assert_eq!(*loader.requests.borrow(), [js_string!("first")]);
    assert_guest_pending(&graph, &mut context);
    wait.assert_pending();

    let second = executor.take_async();
    let second_status = second.observer();
    drop(second);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(first_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        second_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_eq!(*loader.requests.borrow(), [js_string!("first")]);
    assert_guest_pending(&graph, &mut context);
}

#[test]
fn owner_loss_of_remaining_required_loader_wakes_preexisting_graph_waiter() {
    let (mut context, executor, loader, graph) = graph_fixture();
    let _peer = Context::builder()
        .runtime(context.runtime().clone())
        .job_executor(executor.clone())
        .build()
        .unwrap();
    let mut wait = NativeWait::subscribe_before_dispatch(&graph, &mut context);
    let mut first = executor.take_async();
    let first_status = complete_ready(&mut first, &mut context);
    wait.assert_pending();
    assert_eq!(graph.state().unwrap(), PromiseState::Pending);
    let mut second = executor.take_async();
    prepare(&mut second, &mut context);
    let second_status = second.observer();
    assert_eq!(
        *loader.requests.borrow(),
        [js_string!("first"), js_string!("second")]
    );

    drop(context);
    wait.assert_retired(JobRetirement::OwnerDropped);
    assert_eq!(first_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        second_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::OwnerDropped)
    );
    assert_eq!(graph.state().unwrap(), PromiseState::Pending);
}

#[test]
fn graph_native_waiter_completes_only_after_both_required_loaders_publish_success() {
    let (mut context, executor, loader, graph) = graph_fixture();
    let mut wait = NativeWait::subscribe_before_dispatch(&graph, &mut context);
    let mut first = executor.take_async();
    let first_status = complete_ready(&mut first, &mut context);
    wait.assert_pending();
    assert_eq!(graph.state().unwrap(), PromiseState::Pending);
    let mut second = executor.take_async();
    let second_status = complete_ready(&mut second, &mut context);

    assert_eq!(
        *loader.requests.borrow(),
        [js_string!("first"), js_string!("second")]
    );
    assert_eq!(first_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(second_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        graph.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    wait.assert_fulfilled(JsValue::undefined());
}

#[test]
fn dynamic_import_native_waiter_observes_drop_of_selected_unstarted_module_reaction() {
    let executor = Rc::new(ManualExecutor::default());
    let loader = Rc::new(FixtureLoader::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let module = Module::parse(
        Source::from_bytes("export const value = 42;"),
        None,
        context.access(),
    )
    .unwrap();
    loader
        .modules
        .borrow_mut()
        .push((js_string!("entry"), module));
    let imported = JsPromise::from_object(
        context
            .eval(Source::from_bytes("import('entry')"))
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let mut wait = NativeWait::subscribe_before_dispatch(&imported, &mut context);
    watch_guest_rejection(&imported, &mut context);
    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut context);
    assert_eq!(*loader.requests.borrow(), [js_string!("entry")]);
    assert_guest_pending(&imported, &mut context);
    wait.assert_pending();
    let selected = executor.take_promise();
    let selected_status = selected.observer();

    drop(selected);
    wait.assert_retired(JobRetirement::Dropped);
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        selected_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    assert_guest_pending(&imported, &mut context);
}

#[test]
fn dynamic_import_committed_successor_survives_a_later_stop_in_its_loader() {
    use surgeist_js::{JsError, error::RuntimeLimitError, execution::ExecutionOptions};

    #[derive(Default)]
    struct InterruptingExecutor {
        queue: ManualExecutor,
        interrupt_next_promise: Cell<bool>,
        actual_stops: RefCell<Vec<JsError>>,
    }

    impl JobExecutor for InterruptingExecutor {
        fn enqueue_job(
            self: Rc<Self>,
            job: Job,
            mut access: ContextAccess<'_>,
        ) -> Result<JobAcceptance, JobPublicationError> {
            if let Err(cause) = job.check_publication(access.reborrow()) {
                return Err(JobPublicationError::new(job, cause));
            }
            let selected_reaction = matches!(&job, Job::PromiseJob(_));
            self.queue.jobs.borrow_mut().push_back(job);
            if selected_reaction && self.interrupt_next_promise.replace(false) {
                // Publication has already retained the successor. This reentry
                // uses the supplied capability and records a real budget stop.
                let error = access
                    .eval(Source::from_bytes("0"))
                    .expect_err("zero instructions must reject the nested evaluator");
                let cause = JobPublicationCause::try_new(error.clone(), access.reborrow())
                    .expect("the authored instruction stop is engine-valued");
                self.actual_stops.borrow_mut().push(error);
                return Ok(JobAcceptance::AcceptedWithError(cause));
            }
            Ok(JobAcceptance::Accepted)
        }

        fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
            panic!("this fixture dispatches captured jobs explicitly")
        }
    }

    let executor = Rc::new(InterruptingExecutor::default());
    let loader = Rc::new(FixtureLoader::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .module_loader(loader.clone())
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes("globalThis.importExecutions = 0"))
        .unwrap();
    let module = Module::parse(
        Source::from_bytes(
            "globalThis.importExecutions += 1;
             export const answer = 42;",
        ),
        None,
        context.access(),
    )
    .unwrap();
    loader
        .modules
        .borrow_mut()
        .push((js_string!("entry"), module));
    let imported = JsPromise::from_object(
        context
            .eval(Source::from_bytes("import('entry')"))
            .unwrap()
            .as_object()
            .unwrap(),
    )
    .unwrap();
    let mut wait = NativeWait::subscribe_before_dispatch(&imported, &mut context);
    watch_guest_rejection(&imported, &mut context);
    let mut producer = executor.queue.take_async();
    let producer_status = producer.observer();
    prepare(&mut producer, &mut context);
    executor.interrupt_next_promise.set(true);
    let actual_step = RefCell::new(None);
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |access| {
            let holder = access.into_host_access();
            *actual_step.borrow_mut() =
                Some(producer.run_step(&mut TaskContext::from_waker(Waker::noop()), &holder));
            Ok(())
        })
        .unwrap();
    let expected_stop = EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 0 });
    let JobStep::Complete(Err(actual_error)) = actual_step
        .into_inner()
        .expect("the loader job must actually be polled")
    else {
        panic!("the loader itself must return its retained stop")
    };
    assert_eq!(actual_error.as_engine(), Some(&expected_stop));
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&expected_stop)
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(executor.actual_stops.borrow().len(), 1);
    assert_eq!(
        executor.actual_stops.borrow()[0].as_engine(),
        Some(&expected_stop)
    );
    assert_eq!(
        producer_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Interrupted)
    );
    assert_eq!(*loader.requests.borrow(), [js_string!("entry")]);
    assert_eq!(
        context
            .eval(Source::from_bytes("importExecutions"))
            .unwrap(),
        JsValue::new(0)
    );
    assert_guest_pending(&imported, &mut context);
    // The predecessor's own interruption cannot revoke an already-published,
    // still-unstarted successor's real settlement responsibility.
    wait.assert_pending();

    let successor_status = invoke(executor.queue.take_promise(), &mut context);
    assert_eq!(
        context
            .eval(Source::from_bytes("importExecutions"))
            .unwrap(),
        JsValue::new(1)
    );
    assert_guest_pending(&imported, &mut context);
    wait.assert_pending();
    let fulfillment_status = invoke(executor.queue.take_promise(), &mut context);
    let PromiseState::Fulfilled(namespace) = imported.state().unwrap() else {
        panic!("the surviving successor must publish the imported namespace")
    };
    assert_eq!(
        namespace
            .as_object()
            .unwrap()
            .get(js_string!("answer"), context.access())
            .unwrap(),
        JsValue::new(42)
    );
    wait.assert_fulfilled(namespace);
    assert_eq!(successor_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(fulfillment_status.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        producer_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Interrupted)
    );
    assert_eq!(*loader.requests.borrow(), [js_string!("entry")]);
    assert_eq!(
        context
            .eval(Source::from_bytes("importExecutions"))
            .unwrap(),
        JsValue::new(1)
    );
}

#[test]
fn native_promise_future_rejects_a_foreign_receiving_runtime_with_a_type_error() {
    let (mut owner, executor) = context_with_manual_executor();
    let mut foreign = Context::default();
    let promise = ready_promise(42.into(), &mut owner);
    let mut owner_wait = NativeWait::subscribe_before_dispatch(&promise, &mut owner);

    let error = promise
        .clone()
        .into_js_future(foreign.access())
        .expect_err("a foreign receiving runtime cannot retain this promise");
    assert!(
        error.as_native().is_some_and(|native| native.is_type()),
        "foreign promise reception must return the ordinary TypeError: {error:?}"
    );
    assert_eq!(error.as_engine(), None);
    assert_eq!(promise.state().unwrap(), PromiseState::Pending);
    owner_wait.assert_pending();

    let mut producer = executor.take_async();
    let producer_status = complete_ready(&mut producer, &mut owner);
    owner_wait.assert_fulfilled(42.into());
    assert_eq!(promise.state().unwrap(), PromiseState::Fulfilled(42.into()));
    assert_eq!(producer_status.status().unwrap(), JobStatus::Completed);
}

#[test]
fn native_wait_registered_before_thenable_resolution_observes_assimilation_retirement() {
    let (mut context, executor) = context_with_manual_executor();
    let (promise, resolvers) = JsPromise::new_pending(context.access());
    let mut wait = NativeWait::subscribe_before_dispatch(&promise, &mut context);
    watch_guest_rejection(&promise, &mut context);
    let thenable = context
        .eval(Source::from_bytes(
            "globalThis.pendingThenableEntered = false;
             ({ then(resolve, reject) { pendingThenableEntered = true; } })",
        ))
        .unwrap();

    resolvers
        .resolve
        .call(&JsValue::undefined(), &[thenable], context.access())
        .unwrap();
    assert_guest_pending(&promise, &mut context);
    let assimilation = executor.take_promise();
    let assimilation_status = assimilation.observer();
    assert_eq!(assimilation_status.status().unwrap(), JobStatus::Pending);
    assert_eq!(
        context
            .eval(Source::from_bytes("pendingThenableEntered"))
            .unwrap(),
        JsValue::new(false)
    );
    // Do not poll the native future again between resolution and disposal:
    // its original subscription must survive the newly required native work.
    drop(assimilation);

    assert_eq!(
        assimilation_status.status().unwrap(),
        JobStatus::Retired(JobRetirement::Dropped)
    );
    wait.assert_retired(JobRetirement::Dropped);
    assert_guest_pending(&promise, &mut context);
    assert_eq!(
        context
            .eval(Source::from_bytes("pendingThenableEntered"))
            .unwrap(),
        JsValue::new(false)
    );
}
