//! Execution identity and its sole terminal-failure authority.
//!
//! This state owns no SDK handle, context, callback or registry. Cleanup can
//! record a failure here without entering the engine from a Rust destructor.

use std::cell::Cell;

use crate::Error;

pub(crate) struct OwnerState {
    id: u64,
    closed: Cell<bool>,
    failure: Cell<Option<Error>>,
}

impl OwnerState {
    pub(crate) fn new(id: u64) -> Self {
        Self {
            id,
            closed: Cell::new(false),
            failure: Cell::new(None),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn available(&self) -> Result<(), Error> {
        let failure = self.failure.take();
        let result = failure.clone();
        self.failure.set(failure);
        if let Some(error) = result {
            Err(error)
        } else if self.closed.get() {
            Err(Error::BackendUnavailable)
        } else {
            Ok(())
        }
    }

    /// Records the first failure without touching an SDK handle. The backend
    /// requests its interrupt when it next observes this terminal state.
    pub(crate) fn fail(&self, error: Error) -> Error {
        let first = self.failure.take().unwrap_or(error);
        self.failure.set(Some(first.clone()));
        first
    }

    pub(crate) fn close(&self) {
        self.closed.set(true);
    }
}
