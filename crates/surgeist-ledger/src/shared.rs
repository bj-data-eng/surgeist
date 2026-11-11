use crate::{
    Change, Checkpoint, Dimension, Error, IntervalReport, Limits, Operation, PhaseObservation,
    Report, Scope, ScopeEntry, Tariffs, Usage, profile::Profile,
};
use std::{
    ops::AsyncFnOnce,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

/// Cloneable accounting participants. The private lock is held only for Ledger
/// transitions; phase callbacks, future polling and awaiting always run unlocked.
/// Keep a participant outside cancelable work to retrieve accepted-prefix usage
/// and end checkpoints after cancellation and cleanup. Consumers own terminal
/// status and checkpoint completion. Clones keep the operation alive.
#[derive(Clone)]
pub struct SharedOperation<W: Dimension, R: Dimension> {
    operation: Arc<Mutex<Operation<W, R>>>,
}

impl<W: Dimension, R: Dimension> Operation<W, R> {
    /// Moves this operation into optional shared synchronization. No raw lock or
    /// replaceable mutable operation is exposed. Plain operations remain unshared.
    pub fn share(self) -> SharedOperation<W, R> {
        SharedOperation {
            operation: Arc::new(Mutex::new(self)),
        }
    }
}

impl<W: Dimension, R: Dimension> SharedOperation<W, R> {
    /// Requested Rust backing for a shared operation created with
    /// [`Operation::with_reserved_scopes`]. Includes the shared mutex body
    /// and reserved scope entries. Excludes allocator metadata, optional
    /// profiling, and any separately requested checkpoint storage.
    pub fn reserved_backing_bytes(max_scopes: usize) -> Option<usize> {
        if max_scopes == 0 {
            return None;
        }
        size_of::<ScopeEntry<W, R>>()
            .checked_mul(max_scopes)?
            .checked_add(size_of::<Mutex<Operation<W, R>>>())
    }

    fn lock(&self) -> MutexGuard<'_, Operation<W, R>> {
        self.operation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Records `value` as the largest reached by a gauge resource under one lock, so concurrent
    /// records of one gauge cannot release a stale amount.
    pub(crate) fn record_largest(
        &self,
        scope: Scope,
        resource: R,
        value: u64,
    ) -> Result<(), Error<W, R>> {
        self.lock().record_largest(scope, resource, value)
    }

    pub fn root(&self) -> Scope {
        self.lock().root()
    }
    pub fn child(
        &self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        self.lock().child(parent, name, limits, tariffs)
    }
    pub fn scope_name(&self, scope: Scope) -> Result<&'static str, Error<W, R>> {
        self.lock().scope_name(scope)
    }
    /// A copy of the caps configured for a scope; see [`Operation::limits`].
    pub fn limits(&self, scope: Scope) -> Result<Limits<W, R>, Error<W, R>> {
        self.lock().limits(scope).copied()
    }
    /// The most of a resource an acquisition at `scope` could take before a cap on its chain
    /// refuses it, read under one lock; see [`Operation::headroom`].
    pub fn headroom(&self, scope: Scope, resource: R) -> Result<u64, Error<W, R>> {
        self.lock().headroom(scope, resource)
    }
    /// Replaces the caps of a scope for later batches under one lock; see
    /// [`Operation::set_limits`].
    pub fn set_limits(&self, scope: Scope, limits: Limits<W, R>) -> Result<(), Error<W, R>> {
        self.lock().set_limits(scope, limits)
    }
    pub fn usage(&self, scope: Scope) -> Result<Usage<W, R>, Error<W, R>> {
        self.lock().usage(scope).copied()
    }
    /// Checks admission without reserving custody or changing observations.
    /// Concurrent admissions may invalidate this result before a later apply.
    pub fn check_batch(&self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        self.lock().check_batch(scope, changes)
    }

    pub fn apply(&self, scope: Scope, change: Change<W, R>) -> Result<(), Error<W, R>> {
        self.lock().apply(scope, change)
    }
    pub fn apply_batch(&self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        self.lock().apply_batch(scope, changes)
    }
    pub fn snapshot(&self) -> Result<Option<Report<W, R>>, Error<W, R>> {
        self.lock().snapshot()
    }
    pub fn begin_checkpoint(&self) -> Result<Checkpoint, Error<W, R>> {
        self.lock().begin_checkpoint()
    }
    /// Moves reserved interval storage without allocation after identity validation.
    pub fn end_checkpoint(
        &self,
        checkpoint: &Checkpoint,
    ) -> Result<IntervalReport<W, R>, Error<W, R>> {
        self.lock().end_checkpoint(checkpoint)
    }

    /// Begins an explicit observation through the operation's short private lock.
    /// The returned marker retains no lock; see [`Operation::begin_phase`].
    pub fn begin_phase(&self, scope: Scope) -> Result<PhaseObservation, Error<W, R>> {
        self.lock().begin_phase(scope)
    }

    /// Completes an observation through the operation's short private lock.
    ///
    /// Any participant of the same operation may complete it, including after
    /// [`Operation::share`]. A foreign observation is returned for retry without
    /// reading a clock or changing state; see [`Operation::end_phase`].
    pub fn end_phase(&self, observation: PhaseObservation) -> Result<(), PhaseObservation> {
        self.lock().end_phase(observation)
    }

    /// Runs a synchronous callback outside the lock and records its inclusive
    /// phase exactly once on normal return or unwinding. Nested calls use clones.
    pub fn with_phase<T>(&self, scope: Scope, run: impl FnOnce() -> T) -> Result<T, Error<W, R>> {
        let _phase = self.enter_phase(scope)?;
        Ok(run())
    }

    /// Starts on first poll, includes pending time, and ends exactly once on
    /// completion, unwinding or cancellation. An unpolled future records nothing.
    /// The callback uses cloned participants for independent admissions/phases.
    pub async fn with_async_phase<T>(
        &self,
        scope: Scope,
        run: impl AsyncFnOnce() -> T,
    ) -> Result<T, Error<W, R>> {
        let _phase = self.enter_phase(scope)?;
        Ok(run().await)
    }

    fn enter_phase(&self, scope: Scope) -> Result<SharedPhase<W, R>, Error<W, R>> {
        let operation = self.lock();
        operation.index(scope)?;
        let start = operation.profile.first().and_then(Profile::elapsed);
        Ok(SharedPhase {
            operation: self.clone(),
            scope,
            start,
        })
    }
}

struct SharedPhase<W: Dimension, R: Dimension> {
    operation: SharedOperation<W, R>,
    scope: Scope,
    start: Option<Duration>,
}

impl<W: Dimension, R: Dimension> Drop for SharedPhase<W, R> {
    fn drop(&mut self) {
        self.operation.lock().record_phase(self.scope, self.start);
    }
}
