use super::RegisterOperand;
use crate::builtins::promise::{CommittedReactions, ReactionCustody};
use crate::job::{JobFault, JobFaultCause, JobFaultOwner};
use crate::module::lifecycle::{EvaluationInvocation, ModuleAction};
use crate::vm::call_frame::InvocationId;
use crate::{
    Context, JsArgs, JsResult, JsValue,
    builtins::{
        Promise,
        async_generator::AsyncGenerator,
        generator::{GeneratorContext, StoredGeneratorContext},
        promise::PromiseCapability,
    },
    js_string,
    native_function::NativeFunction,
    object::FunctionObjectBuilder,
    vm::{CompletionRecord, GeneratorResumeKind, opcode::Operation},
};
use std::{
    cell::{Cell, RefCell},
    ops::ControlFlow,
    rc::{Rc, Weak},
};
use surgeist_js_gc::heap::{AccessError, ObjectId, RecordRef, Rooted, Trace};

#[derive(surgeist_js_gc::heap::Trace)]
struct AwaitState(Rc<AwaitData>);
#[derive(Trace)]
struct AwaitData {
    continuation: RefCell<Option<StoredGeneratorContext>>,
    successor: RefCell<Option<AwaitRef>>,
    #[heap_trace(skip)]
    phase: Cell<AwaitPhase>,
    #[heap_trace(skip)]
    fault: Cell<Option<JobFault>>,
    #[heap_trace(skip)]
    invocation: InvocationId,
    evaluation: Option<Rc<EvaluationInvocation>>,
}
#[derive(Clone, Trace)]
pub(crate) struct AwaitRef {
    edge: RecordRef<AwaitState>,
    #[heap_trace(skip)]
    data: Weak<AwaitData>,
}
impl std::fmt::Debug for AwaitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Await").field(&self.edge.identity()).finish()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AwaitPhase {
    Prepared,
    SourcePending,
    ExecutorAccepted,
    PolicyDiscarded,
    Uncommitted,
    Unavailable,
    Consumed,
    Retired,
}
impl AwaitPhase {
    fn available(self) -> bool {
        matches!(
            self,
            Self::Prepared | Self::SourcePending | Self::ExecutorAccepted
        )
    }
    fn qualified(self) -> bool {
        matches!(self, Self::SourcePending | Self::ExecutorAccepted)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContinuationFinish {
    Owning,
    QualifiedSuspension,
    ConsumedWithQualifiedSuccessor,
    Consumed,
    Unavailable,
}
impl ContinuationFinish {
    pub(crate) fn is_transferred(self) -> bool {
        matches!(
            self,
            Self::QualifiedSuspension | Self::ConsumedWithQualifiedSuccessor
        )
    }
}
fn await_fault(cause: JobFaultCause) -> JobFault {
    JobFault::new(JobFaultOwner::Task, cause)
}
fn await_access(error: AccessError) -> JobFault {
    await_fault(match error {
        AccessError::HeapBorrowed | AccessError::PayloadBorrowed => {
            JobFaultCause::InaccessibleRegistry
        }
        AccessError::ForeignHeap | AccessError::StaleHandle | AccessError::WrongType => {
            JobFaultCause::OwnerMismatch
        }
    })
}
impl AwaitData {
    fn record_fault(&self, error: JobFault) -> JobFault {
        let first = self.fault.get().unwrap_or(error);
        self.fault.set(Some(first));
        first
    }
    fn check(&self) -> Result<(), JobFault> {
        self.fault.get().map_or(Ok(()), Err)
    }
    fn abandon(&self, reference: &AwaitRef, phase: AwaitPhase) -> Result<(), JobFault> {
        self.phase.set(phase);
        if let Some(evaluation) = &self.evaluation
            && let Err(error) = evaluation.evaluation.consume(reference)
        {
            self.record_fault(error);
        }
        let owned = match self.continuation.try_borrow_mut() {
            Ok(mut slot) => slot.take(),
            Err(_) => {
                self.record_fault(await_fault(JobFaultCause::InaccessibleRegistry));
                None
            }
        };
        // Availability and the exact module link are coherent before arbitrary
        // captured host destructors run. Attempt both cleanup parts on fault.
        drop(owned);
        self.check()
    }
}
impl AwaitRef {
    pub(crate) fn identity(&self) -> ObjectId {
        self.edge.identity()
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.edge == other.edge
    }
    pub(crate) fn root(&self) -> JsResult<AwaitRoot> {
        let capture = self
            .edge
            .upgrade()
            .map_err(await_access)?
            .ok_or_else(|| await_fault(JobFaultCause::OwnerMismatch))?;
        Ok(AwaitRoot {
            capture,
            reference: self.clone(),
        })
    }
    pub(crate) fn phase(&self) -> Result<AwaitPhase, JobFault> {
        let data = self
            .data
            .upgrade()
            .ok_or_else(|| await_fault(JobFaultCause::OwnerMismatch))?;
        data.check()?;
        Ok(data.phase.get())
    }
    pub(crate) fn preparation_failed(&self) -> Result<(), JobFault> {
        let data = self
            .data
            .upgrade()
            .ok_or_else(|| await_fault(JobFaultCause::OwnerMismatch))?;
        if data.phase.get().qualified() {
            data.abandon(self, AwaitPhase::Unavailable)?;
        }
        data.check()
    }
    pub(crate) fn retire(&self) {
        if let Some(data) = self.data.upgrade()
            && let Err(error) = data.abandon(self, AwaitPhase::Retired)
        {
            data.fault.set(Some(data.fault.get().unwrap_or(error)));
        }
    }
    pub(crate) fn matches_evaluation(
        &self,
        expected: &crate::module::lifecycle::EvaluationRef,
    ) -> Result<bool, JobFault> {
        let data = self
            .data
            .upgrade()
            .ok_or_else(|| await_fault(JobFaultCause::OwnerMismatch))?;
        data.check()?;
        Ok(data
            .evaluation
            .as_ref()
            .is_some_and(|actual| actual.evaluation.same(expected)))
    }
}
/// A rooting-only lease. Drop has no phase/disposal action.
#[derive(Clone)]
pub(crate) struct AwaitRoot {
    capture: Rooted<AwaitState>,
    reference: AwaitRef,
}
impl std::fmt::Debug for AwaitRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.reference, f)
    }
}
impl AwaitRoot {
    pub(crate) fn reference(&self) -> &AwaitRef {
        &self.reference
    }
    pub(crate) fn phase(&self) -> AwaitPhase {
        self.capture.0.phase.get()
    }
    pub(crate) fn invocation(&self) -> InvocationId {
        self.capture.0.invocation
    }
    pub(crate) fn qualify(&self, receipt: &CommittedReactions) -> JsResult<()> {
        self.capture.0.check()?;
        if receipt.await_identity() != Some(self.reference.identity()) {
            return Err(await_fault(JobFaultCause::OwnerMismatch).into());
        }
        let phase = match (self.phase(), receipt.custody()) {
            (AwaitPhase::Prepared, ReactionCustody::SourcePending) => AwaitPhase::SourcePending,
            (AwaitPhase::SourcePending, ReactionCustody::SourcePending) => {
                AwaitPhase::SourcePending
            }
            (
                AwaitPhase::Prepared | AwaitPhase::SourcePending | AwaitPhase::ExecutorAccepted,
                ReactionCustody::ExecutorAccepted,
            ) => AwaitPhase::ExecutorAccepted,
            (
                AwaitPhase::Prepared | AwaitPhase::SourcePending | AwaitPhase::ExecutorAccepted,
                ReactionCustody::PolicyDiscarded,
            ) => AwaitPhase::PolicyDiscarded,
            (AwaitPhase::ExecutorAccepted, ReactionCustody::SourcePending) => {
                return Err(await_fault(JobFaultCause::StateInconsistency).into());
            }
            (negative, _) => negative,
        };
        self.capture.0.phase.set(phase);
        Ok(())
    }
    pub(crate) fn uncommitted(&self) -> JsResult<()> {
        self.capture.0.check()?;
        if self.phase() == AwaitPhase::Prepared {
            self.capture.0.phase.set(AwaitPhase::Uncommitted);
        }
        Ok(())
    }
    pub(crate) fn unavailable(&self) -> JsResult<()> {
        self.reference.preparation_failed().map_err(Into::into)
    }
    pub(crate) fn dispose(&self, entered: bool) -> Result<(), JobFault> {
        if self.phase().qualified() || (entered && self.phase() == AwaitPhase::Prepared) {
            self.capture
                .0
                .abandon(&self.reference, AwaitPhase::Unavailable)?;
        }
        self.capture.0.check()
    }
    fn successor(&self) -> JsResult<Option<AwaitRef>> {
        self.capture.0.check()?;
        Ok(self
            .capture
            .0
            .successor
            .try_borrow()
            .map_err(|_| {
                self.capture
                    .0
                    .record_fault(await_fault(JobFaultCause::InaccessibleRegistry))
            })?
            .clone())
    }
    fn next(reference: Option<AwaitRef>) -> JsResult<Option<AwaitRef>> {
        reference
            .map(|reference| reference.root()?.successor())
            .transpose()
            .map(Option::flatten)
    }
    // Floyd detection bounds corrupt lineage without allocating a visited set.
    // Each query walks only consumed successor edges to the current suspension;
    // this custody work is not a claim that continuation tariffs are complete.
    fn walk(&self, reject: Option<ObjectId>) -> JsResult<bool> {
        let mut current = self.successor()?;
        let mut slow = current.clone();
        let mut fast = current.clone();
        while let Some(reference) = current {
            if reject == Some(reference.identity()) {
                return Err(await_fault(JobFaultCause::StateInconsistency).into());
            }
            let root = reference.root()?;
            root.capture.0.check()?;
            if reject.is_none() && root.phase().qualified() {
                return Ok(true);
            }
            if reject.is_none() && root.phase() != AwaitPhase::Consumed {
                return Ok(false);
            }
            current = root.successor()?;
            slow = Self::next(slow)?;
            fast = Self::next(Self::next(fast)?)?;
            if let (Some(a), Some(b)) = (&slow, &fast)
                && a.same(b)
            {
                return Err(await_fault(JobFaultCause::StateInconsistency).into());
            }
        }
        Ok(false)
    }
    pub(crate) fn record_successor(&self, successor: &AwaitRoot) -> JsResult<()> {
        self.capture.0.check()?;
        if matches!(
            self.phase(),
            AwaitPhase::Retired
                | AwaitPhase::Unavailable
                | AwaitPhase::PolicyDiscarded
                | AwaitPhase::Uncommitted
        ) {
            return Ok(());
        }
        if self.phase() != AwaitPhase::Consumed || self.reference.same(successor.reference()) {
            return Err(await_fault(JobFaultCause::StateInconsistency).into());
        }
        successor.walk(Some(self.reference.identity()))?;
        if successor.finish()?.is_transferred() {
            *self.capture.0.successor.try_borrow_mut().map_err(|_| {
                self.capture
                    .0
                    .record_fault(await_fault(JobFaultCause::InaccessibleRegistry))
            })? = Some(successor.reference.clone());
        }
        Ok(())
    }
    pub(crate) fn finish(&self) -> JsResult<ContinuationFinish> {
        self.capture.0.check()?;
        Ok(match self.phase() {
            AwaitPhase::SourcePending | AwaitPhase::ExecutorAccepted => {
                ContinuationFinish::QualifiedSuspension
            }
            AwaitPhase::Consumed if self.walk(None)? => {
                ContinuationFinish::ConsumedWithQualifiedSuccessor
            }
            AwaitPhase::Consumed => ContinuationFinish::Consumed,
            _ => ContinuationFinish::Unavailable,
        })
    }
}
pub(crate) struct PreparedAwaitRef {
    root: AwaitRoot,
}
impl PreparedAwaitRef {
    pub(crate) fn reference(&self) -> &AwaitRef {
        self.root.reference()
    }
}

impl AwaitState {
    fn resume(
        &self,
        state: AwaitRoot,
        value: JsValue,
        kind: GeneratorResumeKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if let Some(evaluation) = &self.0.evaluation {
            if let Some(cause) = evaluation.evaluation.cause() {
                state.reference.retire();
                return Err(crate::error::EngineError::ModuleInterrupted(cause.external()).into());
            }
            evaluation
                .evaluation
                .validate_invocation(evaluation.invocation)?;
        }
        self.0.check()?;
        if !self.0.phase.get().available() {
            return Err(await_fault(JobFaultCause::StateInconsistency).into());
        }
        let mut continuation = self
            .0
            .continuation
            .try_borrow_mut()
            .map_err(|_| {
                self.0
                    .record_fault(await_fault(JobFaultCause::InaccessibleRegistry))
            })?
            .take()
            .ok_or_else(|| await_fault(JobFaultCause::StateInconsistency))?
            .to_context();
        self.0.phase.set(AwaitPhase::Consumed);
        if let Some(evaluation) = &self.0.evaluation {
            evaluation.evaluation.consume(state.reference())?;
        }
        let generator = continuation
            .async_generator_object()?
            .map(|object| {
                object.downcast::<AsyncGenerator>().map_err(|_| {
                    crate::JsError::from(crate::error::EngineError::InvalidEntryPermit)
                })
            })
            .transpose()?;
        let result = continuation.resume_from_await(Some(value), kind, context, state);
        if let Some(generator) = generator {
            AsyncGenerator::finish_resume(&generator, &continuation, result)?;
        } else {
            result.consume()?;
        }
        Ok(JsValue::undefined())
    }
}

/// `Await` implements the Opcode Operation for `Opcode::Await`
///
/// Operation:
///  - Stops the current Async function and schedules it to resume later.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Await;

impl Await {
    #[inline(always)]
    pub(super) fn operation(
        value: RegisterOperand,
        context: &mut Context,
    ) -> ControlFlow<CompletionRecord> {
        let value = context.vm.get_register(value.into());
        let promise = match Promise::promise_resolve(
            &context.intrinsics().constructors().promise().constructor(),
            value.clone(),
            context,
        )
        .and_then(|promise| {
            promise
                .downcast::<Promise>()
                .map_err(|_| crate::JsError::from(crate::error::EngineError::InvalidEntryPermit))
        }) {
            Ok(promise) => promise,
            Err(error) => return context.handle_error(error),
        };
        let return_value = context
            .vm
            .get_promise_capability()
            .ok()
            .map(|cap| JsValue::from(cap.promise))
            .unwrap_or_default();
        let invocation = context.vm.frame().invocation();
        let data = Rc::new(AwaitData {
            continuation: RefCell::new(None),
            successor: RefCell::new(None),
            phase: Cell::new(AwaitPhase::Prepared),
            fault: Cell::new(None),
            invocation,
            evaluation: context.vm.frame().module_evaluation.clone(),
        });
        let capture = match context.runtime().heap.allocate(AwaitState(data.clone())) {
            Ok(capture) => capture,
            Err(_) => {
                return context.handle_error(crate::error::EngineResourceError::Allocation.into());
            }
        };
        let reference = AwaitRef {
            edge: capture.record_ref(),
            data: Rc::downgrade(&data),
        };
        let root = AwaitRoot {
            capture: capture.clone(),
            reference: reference.clone(),
        };
        let prepared = PreparedAwaitRef { root: root.clone() };
        let action = match data
            .evaluation
            .as_ref()
            .map(|_| {
                let Some(crate::vm::ActiveRunnable::Module(module)) =
                    &context.vm.frame().active_runnable
                else {
                    return Err(crate::JsError::from(
                        crate::error::EngineError::InvalidEntryPermit,
                    ));
                };
                ModuleAction::resume(module, reference.clone())
            })
            .transpose()
        {
            Ok(action) => action,
            Err(error) => return context.handle_error(error),
        };
        let lifecycle =
            match crate::builtins::promise::ReactionLifecycle::for_await(&prepared, action) {
                Ok(lifecycle) => lifecycle,
                Err(error) => return context.handle_error(error),
            };
        let reaction = |rejected: bool| {
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_builtin_with_captures(
                    |_, args, (captures, rejected): &(Rooted<AwaitState>, bool), context| {
                        let state = AwaitRoot {
                            capture: captures.clone(),
                            reference: AwaitRef {
                                edge: captures.record_ref(),
                                data: Rc::downgrade(&captures.0),
                            },
                        };
                        captures.resume(
                            state,
                            args.get_or_undefined(0).clone(),
                            if *rejected {
                                GeneratorResumeKind::Throw
                            } else {
                                GeneratorResumeKind::Normal
                            },
                            context,
                        )
                    },
                    (capture.clone(), rejected),
                ),
            )
            .name(js_string!())
            .length(1)
            .build()
        };
        let on_fulfilled = match reaction(false) {
            Ok(value) => value,
            Err(error) => return context.handle_error(error),
        };
        let on_rejected = match reaction(true) {
            Ok(value) => value,
            Err(error) => return context.handle_error(error),
        };
        let generator = match GeneratorContext::from_current(context, None) {
            Ok(generator) => generator,
            Err(error) => return context.handle_error(error),
        };
        let stored = generator.stored();
        match data.continuation.try_borrow_mut() {
            Ok(mut slot) => *slot = Some(stored),
            Err(_) => {
                let error = data.record_fault(await_fault(JobFaultCause::InaccessibleRegistry));
                if context.vm.frame_mut().restore_stack().is_err() {
                    data.record_fault(await_fault(JobFaultCause::StateInconsistency));
                    return ControlFlow::Break(CompletionRecord::Throw(error.into()));
                }
                context.vm.stack.append(generator.stack);
                return context.handle_error(error.into());
            }
        }
        if let Some(evaluation) = &data.evaluation
            && let Err(error) = evaluation.evaluation.suspended(reference.clone())
        {
            data.record_fault(error);
            if context.vm.frame_mut().restore_stack().is_err() {
                data.record_fault(await_fault(JobFaultCause::StateInconsistency));
                return ControlFlow::Break(CompletionRecord::Throw(error.into()));
            }
            context.vm.stack.append(generator.stack);
            return context.handle_error(error.into());
        }
        let outcome = Promise::perform_promise_then_with_lifecycle(
            &promise,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            lifecycle,
            context,
        )
        .into_outcome();
        let (uncommitted, result) = match outcome {
            crate::builtins::promise::RegistrationOutcome::Uncommitted(error) => {
                // Qualification failure never authorizes a captured stack replay.
                match root.uncommitted() {
                    Ok(()) => (true, Err(error)),
                    Err(_) => (false, Err(error)),
                }
            }
            crate::builtins::promise::RegistrationOutcome::Committed(committed) => {
                (false, committed.into_result())
            }
        };
        if let Err(error) = context
            .vm
            .frame_mut()
            .set_await_finish(invocation, root.clone())
        {
            return ControlFlow::Break(CompletionRecord::Throw(result.err().unwrap_or(error)));
        }
        if let Err(mut error) = result {
            if uncommitted && root.phase() == AwaitPhase::Uncommitted {
                let owned = match data.continuation.try_borrow_mut() {
                    Ok(mut slot) => slot.take(),
                    Err(_) => return ControlFlow::Break(CompletionRecord::Throw(error)),
                };
                if owned.is_some() {
                    if let Some(evaluation) = &data.evaluation
                        && evaluation.evaluation.consume(&reference).is_err()
                    {
                        return ControlFlow::Break(CompletionRecord::Throw(error));
                    }
                    if context.vm.frame_mut().restore_stack().is_err() {
                        return ControlFlow::Break(CompletionRecord::Throw(error));
                    }
                    context.vm.stack.append(generator.stack);
                    return context.handle_error(error);
                }
            }
            match root.finish() {
                Ok(finish) if finish.is_transferred() => {
                    if let Some(origin) = context.vm.frame().await_origin().cloned()
                        && origin.record_successor(&root).is_err()
                    {
                        return ControlFlow::Break(CompletionRecord::Throw(error));
                    }
                    if !context.retain_committed_reaction_failure(&mut error) {
                        return ControlFlow::Break(CompletionRecord::Throw(error));
                    }
                }
                _ => return ControlFlow::Break(CompletionRecord::Throw(error)),
            }
        } else if let Some(origin) = context.vm.frame().await_origin().cloned()
            && let Err(error) = origin.record_successor(&root)
        {
            return ControlFlow::Break(CompletionRecord::Throw(error));
        }
        context.vm.set_return_value(return_value);
        context.handle_yield()
    }
}

impl Operation for Await {
    const NAME: &'static str = "Await";
    const INSTRUCTION: &'static str = "INST - Await";
    const COST: u8 = 5;
}

/// `CreatePromiseCapability` implements the Opcode Operation for `Opcode::CreatePromiseCapability`
///
/// Operation:
///  - Create a promise capacity for an async function.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CreatePromiseCapability;

impl CreatePromiseCapability {
    #[inline(always)]
    pub(super) fn operation((): (), context: &mut Context) -> JsResult<()> {
        let promise_capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )?;

        context.vm.set_promise_capability(promise_capability)
    }
}

impl Operation for CreatePromiseCapability {
    const NAME: &'static str = "CreatePromiseCapability";
    const INSTRUCTION: &'static str = "INST - CreatePromiseCapability";
    const COST: u8 = 8;
}

#[cfg(test)]
mod owner_tests;
