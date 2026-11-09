use super::*;
use crate::{
    ContextAccess, Source,
    context::HostHooks,
    error::{EngineError, JobError},
    execution::ExecutionOptions,
    job::{
        Job, JobAcceptance, JobCallback, JobExecutor, JobFaultCause, JobFaultOwner,
        JobPublicationError,
    },
    object::builtins::JsPromise,
};
use std::{cell::Cell, collections::VecDeque};

#[derive(Default)]
struct RegistrationHostCounts {
    made: Cell<usize>,
    called: Cell<usize>,
    notices: Cell<usize>,
}
impl HostHooks for RegistrationHostCounts {
    fn make_job_callback(&self, callback: JsFunction, _: ContextAccess<'_>) -> JobCallback {
        self.made.set(self.made.get() + 1);
        JobCallback::new(callback, ())
    }
    fn call_job_callback(
        &self,
        job: &JobCallback,
        this: &JsValue,
        args: &[JsValue],
        context: ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        self.called.set(self.called.get() + 1);
        job.callback().call(this, args, context)
    }
    fn promise_rejection_tracker(
        &self,
        _: &JsObject<Promise>,
        _: OperationType,
        _: ContextAccess<'_>,
    ) {
        self.notices.set(self.notices.get() + 1);
    }
}
#[derive(Default)]
struct RegistrationQueue {
    publications: Cell<usize>,
    jobs: RefCell<VecDeque<Job>>,
}
impl JobExecutor for RegistrationQueue {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(access.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.publications.set(self.publications.get() + 1);
        self.jobs.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }
    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
        panic!("pending handler-free registration does not dispatch jobs")
    }
}

#[test]
fn whole_untracked_registration_keeps_a_real_root_barrier_refusal_typed_and_uncommitted() {
    let queue = Rc::new(RegistrationQueue::default());
    let hooks = Rc::new(RegistrationHostCounts::default());
    let builder = Context::builder()
        .job_executor(queue.clone())
        .host_hooks(hooks.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    // Use an ordinary JS constructor for a real plain pending representation.
    // Introduce no native observation, handlers or new target before the loan.
    let source_value = context
        .eval_with_budget(
            Source::from_bytes("new Promise(() => {})"),
            ExecutionOptions::new(100_000),
        )
        .unwrap()
        .result
        .unwrap();
    let source = JsPromise::from_object(source_value.as_object().unwrap()).unwrap();
    let source_object = JsObject::clone(&source);
    assert!(
        Promise::native_association(&source_object)
            .unwrap()
            .is_none()
    );
    {
        let borrowed = source_object.borrow().unwrap();
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &borrowed.data().state
        else {
            panic!("the actual JS-created source must be pending")
        };
        assert!(fulfill_reactions.is_empty());
        assert!(reject_reactions.is_empty());
        assert!(!borrowed.data().handled);
    }
    assert_eq!(queue.publications.get(), 0);
    assert!(queue.jobs.borrow().is_empty());
    assert_eq!(hooks.made.get(), 0);
    assert_eq!(hooks.called.get(), 0);
    assert_eq!(hooks.notices.get(), 0);
    let heap = context.runtime().heap.clone();
    // No handlers, result capability or native target need rooting before the
    // actual barrier. The real source/runtime identity checks and Untracked
    // hold are valid while this legitimate shared heap loan is held.
    let registration = heap
        .read(|_| Promise::perform_promise_then(&source_object, None, None, None, &mut context))
        .unwrap();
    // Inspect/dispose the returned owner only AFTER the heap loan has ended.
    let RegistrationOutcome::Uncommitted(error) = registration.into_outcome() else {
        panic!("the refused real barrier cannot install either reaction")
    };
    let Some(EngineError::Job(JobError::Fault(fault))) = error.as_engine() else {
        panic!(
            "an internal registration root refusal must remain a typed engine fault, got {error:?}"
        )
    };
    assert_eq!(fault.owner(), JobFaultOwner::Task);
    assert_eq!(fault.cause(), JobFaultCause::InaccessibleRegistry);
    assert!(!error.is_catchable());
    assert!(
        Promise::native_association(&source_object)
            .unwrap()
            .is_none()
    );
    {
        let borrowed = source_object.borrow().unwrap();
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &borrowed.data().state
        else {
            panic!("setup refusal cannot publish a guest result")
        };
        assert!(fulfill_reactions.is_empty());
        assert!(reject_reactions.is_empty());
        assert!(!borrowed.data().handled);
    }
    assert_eq!(queue.publications.get(), 0);
    assert!(queue.jobs.borrow().is_empty());
    assert_eq!(hooks.made.get(), 0);
    assert_eq!(hooks.called.get(), 0);
    assert_eq!(hooks.notices.get(), 0);
    // The same actual source can commit normally after the lawful loan ends;
    // no fabricated phase, association or stale error is needed for retry.
    let control = Promise::perform_promise_then(&source_object, None, None, None, &mut context);
    let RegistrationOutcome::Committed(control) = control.into_outcome() else {
        panic!("ordinary unborrowed registration must install its real pending pair")
    };
    assert_eq!(control.custody(), ReactionCustody::SourcePending);
    assert_eq!(control.await_identity(), None);
    control.into_result().unwrap();
    assert!(
        Promise::native_association(&source_object)
            .unwrap()
            .is_none()
    );
    {
        let borrowed = source_object.borrow().unwrap();
        let StoredPromiseState::Pending {
            fulfill_reactions,
            reject_reactions,
        } = &borrowed.data().state
        else {
            panic!("ordinary registration retains its pending source")
        };
        assert_eq!(fulfill_reactions.len(), 1);
        assert_eq!(reject_reactions.len(), 1);
        assert!(borrowed.data().handled);
        for record in fulfill_reactions.iter().chain(reject_reactions) {
            assert!(record.handler.is_none());
            assert!(record.promise_capability.is_none());
            assert!(record.native_target.is_none());
            assert!(record.native_result_target.is_none());
        }
    }
    assert_eq!(queue.publications.get(), 0);
    assert!(queue.jobs.borrow().is_empty());
    assert_eq!(hooks.made.get(), 0);
    assert_eq!(hooks.called.get(), 0);
    assert_eq!(hooks.notices.get(), 0);
}
