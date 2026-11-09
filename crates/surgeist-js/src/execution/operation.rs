use std::{
    fmt,
    future::Future,
    ops::AsyncFnOnce,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

use surgeist_ledger::PhaseObservation;

use super::{
    Execution, ExecutionOptions, ExecutionOutcome, ExecutionPhase, ExecutionReceipt,
    ExecutionReport, ExecutionSetupError,
};
use crate::{Context, ContextAccess, JsResult, error::EngineResourceError, vm::CallCheckpoint};

/// A controlled callback reserving its Context until completion or drop.
///
/// Setup succeeds before the callback is invoked. The same operation remains
/// attached across suspension, preserving its limits and first host stop.
/// Retain a [`Self::receipt`] to inspect admitted work after cancellation or Rust
/// unwinding. Dropping an unpolled task invokes no callback and records no phase.
/// Rust panic payloads propagate unchanged; panic-abort cannot publish a report.
///
/// This future is Unpin even when its privately pinned callback future is not.
/// It makes no promise of external Context access between polls. Its lifetime
/// observation includes waiting; VM producer phases separately observe active
/// slices and scheduling waits, not CPU time or arbitrary host future internals.
#[must_use = "poll the task or cancel it to obtain its execution report"]
pub struct ExecutionTask<'ctx, T> {
    future: Option<Pin<Box<dyn Future<Output = ExecutionOutcome<T>> + 'ctx>>>,
    receipt: ExecutionReceipt,
}

impl<T> fmt::Debug for ExecutionTask<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecutionTask")
            .field("receipt", &self.receipt)
            .finish_non_exhaustive()
    }
}

impl<T> ExecutionTask<'_, T> {
    /// Retains access to the task's immutable terminal report without a snapshot.
    pub fn receipt(&self) -> ExecutionReceipt {
        self.receipt.clone()
    }

    /// Drops inner work and its invocation state before returning the report.
    /// A task already completed keeps its Completed disposition.
    /// Host destructor panics propagate; a separately retained receipt can still
    /// observe abandonment during ordinary unwinding.
    pub fn cancel(self) -> ExecutionReport {
        let Self { future, receipt } = self;
        drop(future);
        receipt
            .report()
            .expect("disposing a task publishes its report")
    }
}

impl<T> Future for ExecutionTask<'_, T> {
    type Output = ExecutionOutcome<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let task = self.get_mut();
        let result = catch_unwind(AssertUnwindSafe(|| {
            task.future
                .as_mut()
                .expect("a terminated task cannot be polled")
                .as_mut()
                .poll(cx)
        }));
        match result {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(outcome)) => {
                drop(task.future.take());
                Poll::Ready(outcome)
            }
            Err(payload) => {
                // Dispose a poisoned callback before rethrowing, even if the
                // caller catches the panic and retains this outer task.
                drop(task.future.take());
                resume_unwind(payload)
            }
        }
    }
}

struct Invocation<'ctx> {
    context: &'ctx mut Context,
    checkpoint: Option<CallCheckpoint>,
    lifetime: Option<PhaseObservation>,
    armed: bool,
}

impl<'ctx> Invocation<'ctx> {
    fn new(
        context: &'ctx mut Context,
        options: ExecutionOptions,
    ) -> Result<Self, ExecutionSetupError> {
        if context.execution.is_some() {
            return Err(ExecutionSetupError::AlreadyActive);
        }
        let execution = Execution::new(options)?;
        let checkpoint = context.vm.call_checkpoint();
        context.execution = Some(execution);
        Ok(Self {
            context,
            checkpoint: Some(checkpoint),
            lifetime: None,
            armed: true,
        })
    }

    fn receipt(&self) -> ExecutionReceipt {
        self.context
            .execution
            .as_ref()
            .expect("the invocation owns execution")
            .prepared_report
            .receipt()
    }

    fn begin(&mut self) {
        self.lifetime = self.context.begin_execution_phase(ExecutionPhase::Lifetime);
    }

    fn cleanup(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            self.context.vm.restore_call(checkpoint);
        }
        self.context.end_execution_phase(self.lifetime.take());
    }

    fn complete<T>(mut self, result: JsResult<T>) -> ExecutionOutcome<T> {
        // T and a later error can own arbitrary host destructors. Keep both the
        // first stop and this finalizer installed until disposal has succeeded.
        let result = if self
            .context
            .execution
            .as_ref()
            .expect("the invocation owns execution")
            .stop()
            .is_some()
        {
            drop(result);
            None
        } else {
            Some(result)
        };
        self.cleanup();
        let mut execution = self
            .context
            .execution
            .take()
            .expect("the invocation owns execution");
        let result = match execution.stop.take() {
            Some(stop) => Err(stop),
            None => result.expect("an unstopped invocation retains its result"),
        };
        self.armed = false;
        execution.finish(result)
    }
}

impl Drop for Invocation<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.cleanup();
            let execution = self
                .context
                .execution
                .take()
                .expect("the invocation owns execution");
            self.armed = false;
            drop(execution.abandon());
        }
    }
}

impl Context {
    /// Runs a callback with one work budget and optional operation profiling.
    ///
    /// Synchronous engine reentry shares the active operation. A nested
    /// controlled owner returns [`ExecutionSetupError::AlreadyActive`] before
    /// invoking its callback. The result owns the exact first retained host
    /// stop, even when the callback swallowed it. Committed effects and jobs
    /// are not rolled back. A scope includes only work the callback selects;
    /// evaluating source does not implicitly drain scheduled jobs.
    ///
    /// ```
    /// use surgeist_js::{Context, JsValue, Source, execution::ExecutionOptions};
    /// let mut context = Context::default();
    /// let outcome = context.with_execution(ExecutionOptions::new(128), |mut context| {
    ///     context.eval(Source::from_bytes("40 + 2"))
    /// })?;
    /// assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    /// assert!(outcome.report.instructions().unwrap() > 0);
    /// # Ok::<(), surgeist_js::execution::ExecutionSetupError>(())
    /// ```
    ///
    /// The callback cannot take, replace or mutably project the reserved owner:
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.with_execution(ExecutionOptions::new(128), |mut access| {
    ///     let _: Context = std::mem::take(&mut access);
    ///     Ok(())
    /// });
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.with_execution(ExecutionOptions::new(128), |mut access| {
    ///     let _: Context = std::mem::replace(&mut access, Context::default());
    ///     Ok(())
    /// });
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.with_execution(ExecutionOptions::new(128), |mut access| {
    ///     let _: &mut Context = &mut *access;
    ///     Ok(())
    /// });
    /// ```
    ///
    /// The callback's short access cannot escape in its generic output or in
    /// external storage:
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.with_execution(ExecutionOptions::new(128), |access| Ok(access));
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let mut saved = None;
    /// let _ = owner.with_execution(ExecutionOptions::new(128), |access| {
    ///     saved = Some(access);
    ///     Ok(())
    /// });
    /// drop(saved);
    /// ```
    pub fn with_execution<T>(
        &mut self,
        options: ExecutionOptions,
        run: impl for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
    ) -> Result<ExecutionOutcome<T>, ExecutionSetupError> {
        let mut invocation = Invocation::new(self, options)?;
        invocation.begin();
        let result = run(invocation.context.access());
        Ok(invocation.complete(result))
    }

    /// Prepares a controlled async callback and reserves this Context exclusively.
    ///
    /// The callback begins on first poll. One operation spans all its awaits;
    /// dropping or cancelling the task disposes its inner work before publishing
    /// an abandoned report to retained receipts. An unpolled drop records zero
    /// callback work and no phase. Report storage is prepared during setup.
    ///
    /// This composes directly with [`crate::Script::evaluate_async`]. A host may
    /// explicitly include job draining in its callback; jobs do not automatically
    /// inherit a stopped source operation after that operation ends.
    ///
    /// ```
    /// use surgeist_js::{Context, Source, execution::{ExecutionOptions, ExecutionTermination}};
    /// let mut context = Context::default();
    /// let task = context.execution_task(ExecutionOptions::new(128), async |mut context| {
    ///     context.eval(Source::from_bytes("40 + 2"))
    /// })?;
    /// let receipt = task.receipt();
    /// assert!(receipt.report().is_none());
    /// let report = task.cancel(); // No poll: the callback never ran.
    /// assert_eq!(report.instructions().unwrap(), 0);
    /// assert!(matches!(report.termination(), ExecutionTermination::Abandoned { first_stop: None }));
    /// assert!(receipt.report().is_some());
    /// # Ok::<(), surgeist_js::execution::ExecutionSetupError>(())
    /// ```
    ///
    /// Async callbacks cannot take, replace or mutably project the owner that
    /// the task reserves across suspension:
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.execution_task(ExecutionOptions::new(128), async |mut access| {
    ///     let _: Context = std::mem::take(&mut access);
    ///     Ok(())
    /// });
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.execution_task(ExecutionOptions::new(128), async |mut access| {
    ///     let _: Context = std::mem::replace(&mut access, Context::default());
    ///     Ok(())
    /// });
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.execution_task(ExecutionOptions::new(128), async |mut access| {
    ///     let _: &mut Context = &mut *access;
    ///     Ok(())
    /// });
    /// ```
    ///
    /// The lent access cannot outlive its callback through a generic result or
    /// an external slot, even after the task is dropped:
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let _ = owner.execution_task(ExecutionOptions::new(128), async |access| Ok(access));
    /// ```
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, execution::ExecutionOptions};
    /// let mut owner = Context::default();
    /// let mut saved = None;
    /// let task = owner.execution_task(ExecutionOptions::new(128), async |access| {
    ///     saved = Some(access);
    ///     Ok(())
    /// }).unwrap();
    /// drop(task);
    /// drop(saved);
    /// ```
    pub fn execution_task<'ctx, T, F>(
        &'ctx mut self,
        options: ExecutionOptions,
        run: F,
    ) -> Result<ExecutionTask<'ctx, T>, ExecutionSetupError>
    where
        T: 'ctx,
        F: for<'call> AsyncFnOnce(ContextAccess<'call>) -> JsResult<T> + 'ctx,
    {
        let invocation = Invocation::new(self, options)?;
        let receipt = invocation.receipt();
        // Capture one whole tuple: even unpolled disposal drops the callback's
        // host captures before the invocation finalizer publishes its report.
        let prepared = (run, invocation);
        let future = async move {
            let (run, mut invocation) = prepared;
            invocation.begin();
            let result = run(invocation.context.access()).await;
            invocation.complete(result)
        };
        Ok(ExecutionTask {
            future: Some(Box::pin(future)),
            receipt,
        })
    }

    /// Starts observing an execution phase. A ledger failure is retained as the
    /// operation's stop and returned by its next admission, its check and its
    /// result, so no observation is returned for it.
    pub(crate) fn begin_execution_phase(
        &mut self,
        phase: ExecutionPhase,
    ) -> Option<PhaseObservation> {
        let execution = self.execution.as_mut()?;
        let index = match phase {
            ExecutionPhase::Lifetime => 0,
            ExecutionPhase::VmActive => 1,
            ExecutionPhase::VmWaiting => 2,
            ExecutionPhase::JobPreparation => 3,
            ExecutionPhase::JobInvocation => 4,
            ExecutionPhase::JobPoll => 5,
            ExecutionPhase::GeneratorRequest => 6,
            ExecutionPhase::ModuleMetadata => 7,
            ExecutionPhase::JobClaim => 8,
        };
        match execution
            .ledger
            .begin_phase(execution.execution_phases[index])
        {
            Ok(observation) => Some(observation),
            Err(error) => {
                execution.retain_stop(&super::ledger_error(error));
                None
            }
        }
    }

    /// Completes an observation. A ledger failure is retained as the operation's
    /// stop, as for `begin_execution_phase`.
    pub(crate) fn end_execution_phase(&mut self, observation: Option<PhaseObservation>) {
        if let Some(observation) = observation
            && let Some(execution) = self.execution.as_mut()
            && execution.ledger.end_phase(observation).is_err()
        {
            execution.retain_stop(&EngineResourceError::LedgerInvariant.into());
        }
    }

    pub(crate) fn with_execution_phase<T>(
        &mut self,
        phase: ExecutionPhase,
        run: impl FnOnce(&mut Context) -> T,
    ) -> T {
        if self.execution.is_none() {
            return run(self);
        }
        let observation = self.begin_execution_phase(phase);
        let mut active = self.guard(move |context| context.end_execution_phase(observation));
        run(&mut active)
    }
}
