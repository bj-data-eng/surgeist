use std::time::Duration;

use crate::{Error, Operation, Scope, profile::Profile};

/// An unfinished phase observation bound to its originating operation.
///
/// This marker holds no operation borrow or lock. The consumer may reenter the
/// operation between [`Operation::begin_phase`] and [`Operation::end_phase`].
/// Nested, overlapping and out-of-order completions are inclusive observations.
/// Moving the operation, including into [`Operation::share`], preserves identity.
///
/// Only explicit completion records a phase. Dropping this marker discards the
/// observation, and finishing the operation does not manufacture its end time.
/// Consumers needing completion on unwind or cancellation must own a guard that
/// ends their marker before finishing the operation. Borrowed [`crate::Phase`]
/// and the shared callback helpers retain their automatic completion behavior.
#[derive(Debug)]
#[must_use = "end the observation with its operation to record its phase"]
pub struct PhaseObservation {
    scope: Scope,
    start: Option<Duration>,
}

impl<const W: usize, const R: usize> Operation<W, R> {
    /// Captures a phase's scope and optional configured-clock start sample.
    ///
    /// No operation borrow is retained, no registry entry is allocated, and no
    /// admission or phase completion is recorded. Profiling off and untimed
    /// modes read no clock. End the observation explicitly to include its phase.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ForeignScope`] before reading a clock if the scope
    /// belongs to another operation.
    pub fn begin_phase(&self, scope: Scope) -> Result<PhaseObservation, Error<W, R>> {
        self.index(scope)?;
        Ok(PhaseObservation {
            scope,
            start: self.profile.first().and_then(Profile::elapsed),
        })
    }

    /// Consumes this operation's observation and records one phase completion.
    ///
    /// Uses the existing observer, filtering, bounded event storage and checkpoint
    /// clipping. Completion performs no admission or allocation. An observation
    /// cannot be duplicated or completed twice through safe public construction.
    ///
    /// # Errors
    ///
    /// Returns the original unfinished observation if it belongs to another
    /// operation. That rejection reads no clock and changes no accounting or
    /// observations. The caller can retry completion on its original owner.
    pub fn end_phase(&mut self, observation: PhaseObservation) -> Result<(), PhaseObservation> {
        if self.index(observation.scope).is_err() {
            return Err(observation);
        }
        self.record_phase(observation.scope, observation.start);
        Ok(())
    }
}
