use super::*;
use crate::{
    ContextAccess, JsData, Module, NativeFunction, Source,
    error::{
        DiagnosticEngine, DiagnosticKind, EngineError, EngineResourceError, JobError, PanicError,
    },
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobFaultCause, JobFaultOwner, JobObserver,
        JobPublicationCause, JobPublicationError, JobRetirement, JobStatus,
    },
    object::builtins::{JsFuture, JsPromise},
    property::Attribute,
};
use std::{
    cell::Cell,
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll, Waker},
};
const FINITE: u64 = 100_000;
type EarlyReactionFailure = (ReactionLifecycle, JsError, Rc<Cell<usize>>);
thread_local! {
    static INPUTS: RefCell<VecDeque<JsValue>> = const { RefCell::new(VecDeque::new()) };
    static EARLY: RefCell<Option<EarlyReactionFailure>> = const { RefCell::new(None) };
}
struct Inputs;
impl Drop for Inputs {
    fn drop(&mut self) {
        INPUTS.with(|slot| slot.borrow_mut().clear());
        EARLY.with(|slot| drop(slot.borrow_mut().take()));
    }
}
#[derive(HeapTrace)]
struct StoredRestoreCount(#[heap_trace(skip)] Rc<Cell<usize>>);
struct RestoreCount(Rc<Cell<usize>>);
impl crate::native_function::NativeCapture for RestoreCount {
    type Stored = StoredRestoreCount;
    fn store(&self) -> Self::Stored {
        StoredRestoreCount(self.0.clone())
    }
    fn restore(stored: &Self::Stored) -> Self {
        stored.0.set(stored.0.get() + 1);
        Self(stored.0.clone())
    }
}
struct RestoreHooks(Rc<Cell<usize>>);
impl crate::context::HostHooks for RestoreHooks {
    fn make_job_callback(
        &self,
        callback: crate::object::JsFunction,
        _: ContextAccess<'_>,
    ) -> crate::job::JobCallback {
        crate::job::JobCallback::new(callback, RestoreCount(self.0.clone()))
    }
}
#[derive(JsData, HeapTrace)]
struct Leaf(#[heap_trace(skip)] Rc<Cell<usize>>);
impl Drop for Leaf {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[derive(Default)]
struct Executor {
    queue: RefCell<VecDeque<Job>>,
    inline_first: Cell<bool>,
    tail_error: RefCell<Option<JsError>>,
    observers: RefCell<Vec<JobObserver>>,
}
impl JobExecutor for Executor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Job::PromiseJob(reaction) = &job else {
            panic!("the import-free fixture owns actual promise reactions")
        };
        self.observers.borrow_mut().push(reaction.observer());
        if self.inline_first.replace(false) {
            let Job::PromiseJob(reaction) = job else {
                unreachable!()
            };
            let error = match reaction.call(access.reborrow()) {
                JobCall::Complete(result) => result.unwrap_err(),
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("the real earlier callback must enter: {error:?}")
                }
            };
            return Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error, access).unwrap(),
            ));
        }
        self.queue.borrow_mut().push_back(job);
        let error = self.tail_error.borrow_mut().take();
        match error {
            Some(error) => Ok(JobAcceptance::AcceptedWithError(
                JobPublicationCause::try_new(error, access).unwrap(),
            )),
            None => Ok(JobAcceptance::Accepted),
        }
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("the fixture dispatches exact owned wrappers explicitly")
    }
}
struct Fixture {
    context: Context,
    executor: Rc<Executor>,
    source: JsPromise,
    resolvers: ResolvingFunctions,
    source_wait: JsFuture,
    modules: Vec<Module>,
    results: Vec<JsPromise>,
    waits: Vec<JsFuture>,
    drops: Vec<Rc<Cell<usize>>>,
    restores: Rc<Cell<usize>>,
    _prefix: Option<JsPromise>,
    _inputs: Inputs,
}
impl Fixture {
    fn new(count: usize, early: bool) -> Self {
        assert!(INPUTS.with(|slot| slot.borrow().is_empty()));
        assert!(EARLY.with(|slot| slot.borrow().is_none()));
        let executor = Rc::new(Executor::default());
        let restores = Rc::new(Cell::new(0));
        let builder = Context::builder()
            .job_executor(executor.clone())
            .host_hooks(Rc::new(RestoreHooks(restores.clone())));
        #[cfg(feature = "fuzz")]
        let builder = builder.instructions_remaining(1_000_000);
        let mut context = builder.build().unwrap();
        for name in ["ownerEntered", "ownerAfter"] {
            context
                .register_global_property(crate::JsString::from(name), 0, Attribute::all())
                .unwrap();
        }
        context
            .register_global_builtin_callable(
                js_string!("takeOwnerLeaf"),
                0,
                NativeFunction::from_fn_ptr(|_, _, _| {
                    Ok(INPUTS.with(|slot| slot.borrow_mut().pop_front().unwrap()))
                }),
            )
            .unwrap();
        let (source, resolvers) = JsPromise::new_pending(context.access());
        context
            .register_global_property(js_string!("ownerGate"), source.clone(), Attribute::all())
            .unwrap();
        let prefix = early.then(|| {
            let callback = NativeFunction::from_fn_ptr(|_, _, _| {
                let (lifecycle, error, effects) =
                    EARLY.with(|slot| slot.borrow_mut().take().unwrap());
                effects.set(effects.get() + 1);
                // The real Promise owner observes an actual required-action
                // failure on this already-prepared lifecycle, not a phase write.
                lifecycle
                    .preparation_failed(&error)
                    .map_err(JsError::from)?;
                Err(error)
            })
            .to_js_function(context.realm())
            .unwrap();
            source.then(Some(callback), None, context.access()).unwrap()
        });
        let mut modules = Vec::new();
        let mut results = Vec::new();
        let mut waits = Vec::new();
        let mut drops = Vec::new();
        for _ in 0..count {
            let dropped = Rc::new(Cell::new(0));
            let leaf =
                JsObject::from_proto_and_data(context.runtime(), None, Leaf(dropped.clone()))
                    .unwrap();
            INPUTS.with(|slot| slot.borrow_mut().push_back(leaf.into()));
            drops.push(dropped);
            let module = Module::parse(Source::from_bytes(
                "globalThis.ownerEntered += 1; function consumeLeaf(leaf, value) { globalThis.ownerAfter += 1; } consumeLeaf(takeOwnerLeaf(), await ownerGate);"
            ), None, context.access()).unwrap();
            assert_eq!(
                module.load(context.access()).unwrap().state().unwrap(),
                PromiseState::Fulfilled(JsValue::undefined())
            );
            module.link(context.access()).unwrap();
            let result = context
                .with_execution(ExecutionOptions::new(FINITE), |access| {
                    module.evaluate(access)
                })
                .unwrap()
                .result
                .unwrap();
            let mut wait = result.clone().into_js_future(context.access()).unwrap();
            assert!(poll(&mut wait).is_pending());
            modules.push(module);
            results.push(result);
            waits.push(wait);
        }
        // Native observation preserves its real delivery .then registration.
        // Register it last so authored prefix/module order is explicit.
        let mut source_wait = source.clone().into_js_future(context.access()).unwrap();
        assert!(poll(&mut source_wait).is_pending());
        assert!(INPUTS.with(|slot| slot.borrow().is_empty()));
        assert!(executor.queue.borrow().is_empty());
        context.runtime().collect().unwrap();
        for count in &drops {
            assert_eq!(count.get(), 0);
        }
        Self {
            context,
            executor,
            source,
            resolvers,
            source_wait,
            modules,
            results,
            waits,
            drops,
            restores,
            _prefix: prefix,
            _inputs: Inputs,
        }
    }
    fn after(&mut self) -> JsValue {
        self.context
            .global_object()
            .get(js_string!("ownerAfter"), self.context.access())
            .unwrap()
    }
}
fn poll(wait: &mut JsFuture) -> Poll<JsResult<JsValue>> {
    Pin::new(wait).poll(&mut TaskContext::from_waker(Waker::noop()))
}
fn task_fault(error: &JsError) -> crate::job::JobFault {
    let Some(EngineError::Job(JobError::Fault(fault))) = error.as_engine() else {
        panic!("the actual checked root must return its typed owner fault")
    };
    assert_eq!(fault.owner(), JobFaultOwner::Task);
    assert_eq!(fault.cause(), JobFaultCause::InaccessibleRegistry);
    *fault
}

#[test]
fn actual_setup_root_barrier_refusal_retains_its_error_for_every_source_pending_module() {
    let mut fixture = Fixture::new(3, false);
    let source = JsObject::clone(&fixture.source);
    let heap = fixture.context.runtime().heap.clone();
    let association = Promise::native_association(&source).unwrap();
    let setup = ReactionRestoration::new(&source, association.as_ref());
    let before_restore = fixture.restores.get();
    let refused = heap.read(|_| {
        heap.with_pending_edges(ReactionSourceRoots(source.clone()), |_| -> () {
            panic!("a refused actual setup barrier must not detach, restore or enter its publication body")
        })
    }).unwrap();
    let Err(access) = refused else {
        panic!("the real setup barrier requires mutable heap access")
    };
    assert_eq!(access, surgeist_js_gc::heap::AccessError::HeapBorrowed);
    // The exact outer setup path runs only after the legitimate heap loan ends.
    // A returned error must drive its module failure owner, not merely make the
    // Await links unavailable without giving their native observers a cause.
    let actual = setup
        .finish(Err::<(), JsError>(reaction_access_error(access)))
        .unwrap_err();
    let fault = task_fault(&actual);
    assert_eq!(fixture.restores.get(), before_restore);
    assert!(fixture.executor.queue.borrow().is_empty());
    assert!(fixture.executor.observers.borrow().is_empty());
    let Poll::Ready(Err(source_error)) = poll(&mut fixture.source_wait) else {
        panic!("failed setup must retire the original source native wait")
    };
    assert_eq!(
        source_error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::Interrupted))
    );
    for index in 0..fixture.modules.len() {
        let Poll::Ready(Err(error)) = poll(&mut fixture.waits[index]) else {
            panic!(
                "returned setup error must not strand an actual source-retained module native wait Pending"
            )
        };
        let Some(EngineError::ModuleInterrupted(native)) = error.as_engine() else {
            panic!("each actual module failure owner must retain the returned setup error")
        };
        assert_eq!(
            native.cause().kind(),
            DiagnosticKind::Engine(DiagnosticEngine::Job(JobError::Fault(fault)))
        );
        assert_eq!(
            fixture.results[index].state().unwrap(),
            PromiseState::Pending
        );
        let later = fixture
            .context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                fixture.modules[index].evaluate(access)
            })
            .unwrap();
        assert_eq!(later.report.instructions().unwrap(), 0);
        let later = later.result.unwrap_err();
        let Some(EngineError::ModuleInterrupted(later)) = later.as_engine() else {
            panic!("setup failure remains terminal")
        };
        assert_eq!(later.identity(), native.identity());
    }
    fixture.context.runtime().collect().unwrap();
    for count in &fixture.drops {
        assert_eq!(count.get(), 1);
    }
    assert_eq!(fixture.after(), JsValue::new(0));
}

#[test]
fn actual_lifecycle_root_contention_cleans_prepared_prefix_current_record_and_stored_tail() {
    let mut fixture = Fixture::new(3, false);
    let source = JsObject::clone(&fixture.source);
    let heap = fixture.context.runtime().heap.clone();
    let association = Promise::native_association(&source).unwrap();
    let actual = heap
        .with_pending_edges(ReactionSourceRoots(source.clone()), |_| {
            let mut restoration = ReactionRestoration::new(&source, association.as_ref());
            restoration.detach().unwrap();
            let records = restoration.restore(ReactionType::Fulfill).unwrap();
            assert_eq!(records.len(), 4); // Three module Await records + native delivery.
            let mut preparation = ReactionPreparation {
                remaining: records.into_iter(),
                prepared: Vec::new(),
                current: ReactionJobPreparation::new(),
                association: association.as_ref(),
            };
            preparation.current.live = preparation.remaining.next();
            new_promise_reaction_job(
                &mut preparation.current,
                JsValue::new(7),
                &mut fixture.context,
            )
            .unwrap();
            let prefix = preparation.current.take_prepared().unwrap();
            let prefix_observer = prefix.job.observer();
            preparation.prepared.push(prefix);
            preparation.current.live = preparation.remaining.next();
            assert_eq!(preparation.remaining.as_slice().len(), 2); // Module + native delivery.
            // Exercise the canonical checked lifecycle-root front itself. The full
            // factory's earlier realm conversion is intentionally outside this loan.
            let root_error = heap
                .read(
                    |_| match preparation.current.live.as_ref().unwrap().lifecycle.hold() {
                        Err(error) => error,
                        Ok(_) => panic!(
                            "the actual root cannot acquire mutable heap access during this read"
                        ),
                    },
                )
                .unwrap();
            task_fault(&root_error);
            // The real heap loan ends before any owner invalidation, callbacks,
            // final native drops or notices. Both actual guards own their full sets.
            let error = preparation.fail(root_error);
            assert_eq!(
                prefix_observer.status(),
                Ok(JobStatus::Retired(JobRetirement::Dropped))
            );
            assert_eq!(prefix_observer.fault(), None);
            restoration.finish(Err::<(), _>(error)).unwrap_err()
        })
        .unwrap();
    let fault = task_fault(&actual);
    assert!(fixture.executor.queue.borrow().is_empty());
    assert!(fixture.executor.observers.borrow().is_empty());
    let Poll::Ready(Err(source_error)) = poll(&mut fixture.source_wait) else {
        panic!("actual source preparation must not abandon its original native waiter")
    };
    assert_eq!(
        source_error.as_engine(),
        Some(&EngineError::JobRetired(JobRetirement::Interrupted))
    );
    for index in 0..fixture.modules.len() {
        let Poll::Ready(Err(error)) = poll(&mut fixture.waits[index]) else {
            panic!("all actual module tails must observe preparation failure")
        };
        let Some(EngineError::ModuleInterrupted(native)) = error.as_engine() else {
            panic!("required module action failure must be retained")
        };
        assert_eq!(
            native.cause().kind(),
            DiagnosticKind::Engine(DiagnosticEngine::Job(JobError::Fault(fault)))
        );
        assert_eq!(
            fixture.results[index].state().unwrap(),
            PromiseState::Pending
        );
        let observed = fixture
            .context
            .with_execution(ExecutionOptions::new(FINITE), |access| {
                fixture.modules[index].evaluate(access)
            })
            .unwrap();
        assert_eq!(observed.report.instructions().unwrap(), 0);
        let observed = observed.result.unwrap_err();
        let Some(EngineError::ModuleInterrupted(observed)) = observed.as_engine() else {
            panic!("the failed module stays terminal")
        };
        assert_eq!(observed.identity(), native.identity());
    }
    fixture.context.runtime().collect().unwrap();
    for count in &fixture.drops {
        assert_eq!(count.get(), 1);
    }
    assert_eq!(fixture.after(), JsValue::new(0));
}

#[test]
fn an_earlier_actual_callback_retires_a_prepared_tail_before_its_late_accepted_error_receipt() {
    let mut fixture = Fixture::new(1, true);
    let effects = Rc::new(Cell::new(0));
    let first: JsError = PanicError::new("first actual prepared-tail retirement")
        .with_source(JsNativeError::range().with_message("original owner cause"))
        .into();
    let lifecycle = {
        let source = fixture.source.borrow();
        let StoredPromiseState::Pending {
            fulfill_reactions, ..
        } = &source.data().state
        else {
            panic!("actual source remains pending")
        };
        assert_eq!(fulfill_reactions.len(), 3); // Earlier callback, module, native delivery.
        fulfill_reactions[1].lifecycle.clone()
    };
    EARLY.with(|slot| {
        assert!(
            slot.replace(Some((lifecycle, first.clone(), effects.clone())))
                .is_none()
        )
    });
    fixture.executor.inline_first.set(true);
    *fixture.executor.tail_error.borrow_mut() = Some(EngineResourceError::Allocation.into());
    let actual = fixture
        .resolvers
        .resolve
        .call(
            &JsValue::undefined(),
            &[JsValue::new(7)],
            fixture.context.access(),
        )
        .unwrap_err();
    assert_eq!(actual, first);
    assert_eq!(effects.get(), 1);
    assert!(EARLY.with(|slot| slot.borrow().is_none()));
    assert_eq!(
        fixture.source.state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(7))
    );
    assert_eq!(
        poll(&mut fixture.source_wait),
        Poll::Ready(Ok(JsValue::new(7)))
    );
    assert_eq!(
        fixture.executor.observers.borrow()[0].status(),
        Ok(JobStatus::Retired(JobRetirement::Interrupted))
    );
    let observer = fixture.executor.observers.borrow()[1].clone();
    assert_eq!(observer.status(), Ok(JobStatus::Pending));
    assert_eq!(observer.fault(), None);
    let Poll::Ready(Err(error)) = poll(&mut fixture.waits[0]) else {
        panic!("the actual retired prepared module tail must notify its preexisting native wait")
    };
    let Some(EngineError::ModuleInterrupted(cause)) = error.as_engine() else {
        panic!("the original owner error must seal the module")
    };
    let DiagnosticKind::Engine(DiagnosticEngine::Panic { message, .. }) = cause.cause().kind()
    else {
        panic!("the earlier full cause must not become the later allocation error")
    };
    assert_eq!(message, "first actual prepared-tail retirement");
    let job = fixture.executor.queue.borrow_mut().pop_front().unwrap();
    let stopped = fixture
        .context
        .with_execution(ExecutionOptions::new(FINITE), |access| {
            let Job::PromiseJob(job) = job else {
                panic!("actual saved tail")
            };
            match job.call(access) {
                JobCall::Complete(result) => result,
                JobCall::Deferred { job, error } => {
                    drop(job);
                    panic!("the admitted saved action reports its module cause: {error:?}")
                }
            }
        })
        .unwrap()
        .result
        .unwrap_err();
    let Some(EngineError::ModuleInterrupted(stopped)) = stopped.as_engine() else {
        panic!("late accepted receipt cannot revive retired storage")
    };
    assert_eq!(stopped.identity(), cause.identity());
    assert_eq!(
        observer.status(),
        Ok(JobStatus::Retired(JobRetirement::Interrupted))
    );
    assert_eq!(observer.fault(), None);
    assert_eq!(effects.get(), 1);
    assert_eq!(fixture.after(), JsValue::new(0));
    fixture.context.runtime().collect().unwrap();
    assert_eq!(fixture.drops[0].get(), 1);
}
