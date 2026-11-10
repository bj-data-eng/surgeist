//! Execution identity and its sole terminal-failure authority.
//!
//! This state owns no SDK handle, context, callback or registry. Cleanup can
//! record a failure here without entering the engine from a Rust destructor.

use std::cell::Cell;
use std::rc::Rc;

use crate::Error;
use crate::conversion::{CleanupFailure, CleanupFault, CleanupPrimary, CleanupStatus};

#[derive(Clone)]
enum Failure {
    Host(Error),
    Cleanup(CleanupFault),
}

pub(crate) struct OwnerState {
    id: u64,
    closed: Cell<bool>,
    failure: Cell<Option<Failure>>,
    boundaries: Cell<usize>,
    cleanup: Cell<CleanupStatus>,
}

impl OwnerState {
    pub(crate) fn new(id: u64) -> Self {
        Self {
            id,
            closed: Cell::new(false),
            failure: Cell::new(None),
            boundaries: Cell::new(0),
            cleanup: Cell::new(CleanupStatus::default()),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn available(&self) -> Result<(), Error> {
        match self.failure() {
            Some(Failure::Host(error)) => Err(error),
            Some(Failure::Cleanup(cleanup)) => {
                let error = cleanup_error(None, cleanup);
                if self.boundaries.get() == 0 && !std::thread::panicking() {
                    self.failure.set(Some(Failure::Host(error.clone())));
                }
                Err(error)
            }
            None if self.closed.get() => Err(Error::BackendUnavailable),
            None => Ok(()),
        }
    }

    fn failure(&self) -> Option<Failure> {
        let failure = self.failure.take();
        let result = failure.clone();
        self.failure.set(failure);
        result
    }

    /// Records the first failure without touching an SDK handle. The backend
    /// requests its interrupt when it next observes this terminal state.
    pub(crate) fn fail(&self, error: Error) -> Error {
        match self.failure() {
            Some(Failure::Host(first)) => first,
            // A refused continuation reports immutable cleanup facts, but
            // does not select the outward result before the actual exit.
            Some(Failure::Cleanup(cleanup)) if matches!(&error, Error::ConversionCleanup(failure) if failure.cleanup() == cleanup) => {
                error
            }
            _ => {
                self.failure.set(Some(Failure::Host(error.clone())));
                error
            }
        }
    }

    pub(crate) fn record_cleanup(&self, cleanup: CleanupFault) {
        self.cleanup.set(self.cleanup.get().record(cleanup));
        if self.failure().is_none() {
            if self.boundaries.get() == 0 && !std::thread::panicking() {
                self.failure
                    .set(Some(Failure::Host(cleanup_error(None, cleanup))));
            } else {
                self.failure.set(Some(Failure::Cleanup(cleanup)));
            }
        }
    }

    pub(crate) fn cleanup_status(&self) -> CleanupStatus {
        self.cleanup.get()
    }

    /// Describes a local result without choosing a still-active outer exit.
    pub(crate) fn operation_result<T>(&self, result: Result<T, Error>) -> Result<T, Error> {
        match self.failure() {
            Some(Failure::Host(first)) => {
                drop(result);
                Err(first)
            }
            Some(Failure::Cleanup(cleanup)) => match result {
                Ok(value) => {
                    drop(value);
                    Err(cleanup_error(None, cleanup))
                }
                Err(error) => Err(combine(error, cleanup)),
            },
            None => result,
        }
    }

    pub(crate) fn finalize<T>(&self, result: Result<T, Error>) -> Result<T, Error> {
        let result = self.operation_result(result);
        if matches!(self.failure(), Some(Failure::Cleanup(_)))
            && let Err(error) = &result
        {
            self.failure.set(Some(Failure::Host(error.clone())));
        }
        result
    }

    pub(crate) fn close(&self) {
        self.closed.set(true);
    }
}

fn cleanup_error(primary: Option<CleanupPrimary>, cleanup: CleanupFault) -> Error {
    Error::ConversionCleanup(CleanupFailure { primary, cleanup })
}

fn combine(error: Error, cleanup: CleanupFault) -> Error {
    match error {
        Error::Exception(exception) => {
            cleanup_error(Some(CleanupPrimary::Exception(exception)), cleanup)
        }
        Error::CallbackPanicked => cleanup_error(Some(CleanupPrimary::CallbackPanicked), cleanup),
        // Keep an already described primary; the first cleanup remains causal.
        Error::ConversionCleanup(failure) => Error::ConversionCleanup(failure),
        other => other,
    }
}

/// Actual host/callback result selection, including local Drop and reaping.
/// It retains no SDK handle and leaves pending cleanup unselected on unwind.
pub(crate) struct Boundary {
    owner: Rc<OwnerState>,
}

impl Boundary {
    pub(crate) fn enter(owner: Rc<OwnerState>) -> Result<Self, Error> {
        owner.available()?;
        let count = owner
            .boundaries
            .get()
            .checked_add(1)
            .ok_or_else(|| owner.fail(Error::IdentityExhausted))?;
        owner.boundaries.set(count);
        Ok(Self { owner })
    }

    pub(crate) fn finish<T>(self, result: Result<T, Error>) -> Result<T, Error> {
        self.owner.finalize(result)
    }
}

impl Drop for Boundary {
    fn drop(&mut self) {
        match self.owner.boundaries.get().checked_sub(1) {
            Some(count) => self.owner.boundaries.set(count),
            None => {
                self.owner.fail(Error::RegistryConflict);
            }
        }
    }
}
