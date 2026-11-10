//! Owner-local functional qualification for the new reaction custody API.

use super::*;
use crate::{
    Source,
    builtins::promise::{ReactionCustody, ReactionLifecycle, RegistrationOutcome},
    error::{EngineError, JobError},
    job::{Job, JobAcceptance, JobCall, JobExecutor, JobPublicationError, JobStatus},
    object::builtins::JsPromise,
};
use std::collections::VecDeque;

// This is a phase-only VM-owner fixture. It deliberately contains no runnable
// bytecode storage and makes no claim about consumption/resumption correctness.
// Prepared is a legal state before installation; terminal Consumed owns no old
// storage. Production transitions are exercised below, except explicitly named
// terminal/corruption baselines used to test reconciliation and fault reporting.
fn prepared(context: &Context) -> PreparedAwaitRef {
    let data = Rc::new(AwaitData {
        continuation: RefCell::new(None),
        successor: RefCell::new(None),
        phase: Cell::new(AwaitPhase::Prepared),
        fault: Cell::new(None),
        invocation: context.vm.frame().invocation(),
        evaluation: None,
    });
    let capture = context
        .runtime()
        .heap
        .allocate(AwaitState(data.clone()))
        .unwrap();
    let reference = AwaitRef {
        edge: capture.record_ref(),
        data: Rc::downgrade(&data),
    };
    PreparedAwaitRef {
        root: AwaitRoot { capture, reference },
    }
}

fn pending_receipt(
    witness: &PreparedAwaitRef,
    context: &mut Context,
) -> (JsPromise, CommittedReactions) {
    let (source, _resolvers) = JsPromise::new_pending(context.access()).unwrap();
    let lifecycle = ReactionLifecycle::for_await(witness, None).unwrap();
    let registration =
        Promise::perform_promise_then_with_lifecycle(&source, None, None, None, lifecycle, context);
    let RegistrationOutcome::Committed(receipt) = registration.into_outcome() else {
        panic!("the real pending pair must commit")
    };
    assert!(matches!(receipt.custody(), ReactionCustody::SourcePending));
    (source, receipt)
}

fn assert_fault(error: &crate::JsError, cause: JobFaultCause) {
    let Some(EngineError::Job(JobError::Fault(fault))) = error.as_engine() else {
        panic!("expected typed owner fault, got {error:?}")
    };
    assert_eq!(fault.owner(), JobFaultOwner::Task);
    assert_eq!(fault.cause(), cause);
}

#[test]
fn wrong_await_receipt_refuses_without_changing_either_capture() {
    let mut context = Context::builder().build().unwrap();
    let original = prepared(&context);
    let other = prepared(&context);
    let (_source, receipt) = pending_receipt(&original, &mut context);
    assert_eq!(
        receipt.await_identity(),
        Some(original.reference().identity())
    );

    let error = other.root.qualify(&receipt).unwrap_err();
    assert_fault(&error, JobFaultCause::OwnerMismatch);
    assert_eq!(original.root.phase(), AwaitPhase::SourcePending);
    assert_eq!(other.root.phase(), AwaitPhase::Prepared);
    assert!(original.root.capture.0.continuation.borrow().is_none());
    assert!(other.root.capture.0.continuation.borrow().is_none());
    assert!(original.root.finish().unwrap().is_transferred());
    assert!(!other.root.finish().unwrap().is_transferred());
}

#[test]
fn late_real_receipt_keeps_retired_and_unavailable_captures_negative() {
    for retired in [false, true] {
        let mut context = Context::builder().build().unwrap();
        let witness = prepared(&context);
        let (_source, receipt) = pending_receipt(&witness, &mut context);
        if retired {
            witness.reference().retire();
        } else {
            witness.root.dispose(false).unwrap();
        }
        let expected = if retired {
            AwaitPhase::Retired
        } else {
            AwaitPhase::Unavailable
        };
        assert_eq!(witness.root.phase(), expected);

        witness.root.qualify(&receipt).unwrap();
        assert_eq!(witness.root.phase(), expected);
        assert!(matches!(receipt.custody(), ReactionCustody::SourcePending));
        assert_eq!(
            witness.root.finish().unwrap(),
            ContinuationFinish::Unavailable
        );
        assert!(witness.root.capture.0.continuation.borrow().is_none());
    }
}

#[test]
fn late_real_receipt_cannot_recreate_a_consumed_baseline() {
    let mut context = Context::builder().build().unwrap();
    let witness = prepared(&context);
    let (_source, receipt) = pending_receipt(&witness, &mut context);
    // Private owner fixture: model an already consumed capture with no old
    // continuation. The stimulus is the late receipt, not this setup assignment.
    witness.root.capture.0.phase.set(AwaitPhase::Consumed);

    witness.root.qualify(&receipt).unwrap();
    witness.root.dispose(true).unwrap();
    assert_eq!(witness.root.phase(), AwaitPhase::Consumed);
    assert_eq!(witness.root.finish().unwrap(), ContinuationFinish::Consumed);
    assert!(witness.root.capture.0.continuation.borrow().is_none());
}

#[derive(Default)]
struct HeldExecutor(RefCell<VecDeque<Job>>);
impl JobExecutor for HeldExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut context: crate::ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(context.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        self.0.borrow_mut().push_back(job);
        Ok(JobAcceptance::Accepted)
    }

    fn run_jobs(self: Rc<Self>, _: crate::ContextAccess<'_>) -> JsResult<()> {
        panic!("this test explicitly dispatches the one retained reaction")
    }
}

fn held_context(executor: Rc<HeldExecutor>) -> Context {
    let builder = Context::builder().job_executor(executor);
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

#[test]
fn dropping_producer_roots_keeps_the_accepted_job_available_until_dispatch() {
    let executor = Rc::new(HeldExecutor::default());
    let mut context = held_context(executor.clone());
    let handler = context.eval(Source::from_bytes(
        "globalThis.ownerDeliveries = 0; value => { globalThis.ownerDeliveries += 1; return value; }",
    )).unwrap().as_callable().unwrap();
    let handler = crate::object::JsFunction::from_object(handler).unwrap();
    let witness = prepared(&context);
    let reference = witness.reference().clone();
    let producer_hold = reference.root().unwrap();
    let (source, resolvers) = JsPromise::new_pending(context.access()).unwrap();
    let lifecycle = ReactionLifecycle::for_await(&witness, None).unwrap();
    Promise::perform_promise_then_with_lifecycle(
        &source,
        Some(handler),
        None,
        None,
        lifecycle,
        &mut context,
    )
    .into_result()
    .unwrap();
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[7.into()], context.access())
        .unwrap();
    assert_eq!(producer_hold.phase(), AwaitPhase::ExecutorAccepted);

    drop(producer_hold);
    drop(witness);
    drop(resolvers);
    drop(source);
    context.runtime().collect().unwrap();
    {
        let queued_root = reference
            .root()
            .expect("the accepted job owns its actual Await root");
        assert_eq!(queued_root.phase(), AwaitPhase::ExecutorAccepted);
        assert!(queued_root.finish().unwrap().is_transferred());
    }
    let job = executor.0.borrow_mut().pop_front().unwrap();
    let Job::PromiseJob(job) = job else {
        panic!("settlement publishes its actual reaction")
    };
    let observer = job.observer();
    assert_eq!(observer.status().unwrap(), JobStatus::Pending);
    match job.call(context.access()) {
        JobCall::Complete(result) => {
            result.unwrap();
        }
        JobCall::Deferred { error, .. } => panic!("fresh dispatch must remain possible: {error:?}"),
    }
    assert_eq!(observer.status().unwrap(), JobStatus::Completed);
    assert_eq!(
        context.eval(Source::from_bytes("ownerDeliveries")).unwrap(),
        1.into()
    );
    assert!(executor.0.borrow().is_empty());
    context.runtime().collect().unwrap();
    assert!(
        reference.root().is_err(),
        "producer and consumed job roots both release"
    );
}

#[test]
fn old_pending_receipt_cannot_downgrade_actual_executor_acceptance() {
    let executor = Rc::new(HeldExecutor::default());
    let mut context = held_context(executor.clone());
    let witness = prepared(&context);
    let (source, resolvers) = JsPromise::new_pending(context.access()).unwrap();
    let lifecycle = ReactionLifecycle::for_await(&witness, None).unwrap();
    let registration = Promise::perform_promise_then_with_lifecycle(
        &source,
        None,
        None,
        None,
        lifecycle,
        &mut context,
    );
    let RegistrationOutcome::Committed(old_pending) = registration.into_outcome() else {
        panic!("the real pending pair must commit")
    };
    assert!(matches!(
        old_pending.custody(),
        ReactionCustody::SourcePending
    ));
    // An idempotent observation of that same pending commit is harmless.
    witness.root.qualify(&old_pending).unwrap();
    assert_eq!(witness.root.phase(), AwaitPhase::SourcePending);

    resolvers
        .resolve
        .call(&JsValue::undefined(), &[7.into()], context.access())
        .unwrap();
    assert_eq!(witness.root.phase(), AwaitPhase::ExecutorAccepted);
    let error = witness.root.qualify(&old_pending).unwrap_err();
    assert_fault(&error, JobFaultCause::StateInconsistency);
    assert_eq!(witness.root.phase(), AwaitPhase::ExecutorAccepted);
    assert_eq!(
        witness.root.finish().unwrap(),
        ContinuationFinish::QualifiedSuspension
    );
    assert!(matches!(
        old_pending.custody(),
        ReactionCustody::SourcePending
    ));
    let queued = executor.0.borrow_mut().pop_front();
    assert!(matches!(queued, Some(Job::PromiseJob(_))));
    drop(queued);
}

#[test]
fn consumed_baseline_needs_a_distinct_current_qualified_successor() {
    let mut context = Context::builder().build().unwrap();
    let old = prepared(&context);
    let successor = prepared(&context);
    // Valid phase-only terminal fixture, not a simulated runnable continuation.
    old.root.capture.0.phase.set(AwaitPhase::Consumed);
    assert_eq!(old.root.finish().unwrap(), ContinuationFinish::Consumed);
    assert!(!old.root.finish().unwrap().is_transferred());

    old.root.record_successor(&successor.root).unwrap();
    assert!(
        !old.root.finish().unwrap().is_transferred(),
        "Prepared is not proof"
    );
    let (_source, receipt) = pending_receipt(&successor, &mut context);
    successor.root.qualify(&receipt).unwrap();
    old.root.record_successor(&successor.root).unwrap();
    assert_eq!(
        old.root.finish().unwrap(),
        ContinuationFinish::ConsumedWithQualifiedSuccessor
    );

    successor.reference().retire();
    assert_eq!(old.root.finish().unwrap(), ContinuationFinish::Consumed);
    assert!(
        !old.root.finish().unwrap().is_transferred(),
        "a historical qualification is not current proof"
    );
}

#[test]
fn self_lineage_and_a_corrupted_cycle_return_typed_faults() {
    let context = Context::builder().build().unwrap();
    let first = prepared(&context);
    let second = prepared(&context);
    first.root.capture.0.phase.set(AwaitPhase::Consumed);
    second.root.capture.0.phase.set(AwaitPhase::Consumed);

    let error = first.root.record_successor(&first.root).unwrap_err();
    assert_fault(&error, JobFaultCause::StateInconsistency);
    assert_eq!(first.root.phase(), AwaitPhase::Consumed);
    assert_eq!(second.root.phase(), AwaitPhase::Consumed);

    // Explicit private corruption fixture; no production constructor creates it.
    // The behavior under test is bounded, typed diagnosis rather than panic/loop.
    *first.root.capture.0.successor.borrow_mut() = Some(second.reference().clone());
    *second.root.capture.0.successor.borrow_mut() = Some(first.reference().clone());
    let error = first.root.finish().unwrap_err();
    assert_fault(&error, JobFaultCause::StateInconsistency);
    assert_eq!(first.root.phase(), AwaitPhase::Consumed);
    assert_eq!(second.root.phase(), AwaitPhase::Consumed);
}
