//! Publication observations and ownership of work rejected before acceptance.

use super::Job;
use crate::{Context, ContextAccess, JsError, JsResult};
use std::{error::Error, fmt};

/// The disposition of a job whose input custody has been consumed.
///
/// This observation does not authorize invocation, cancellation or replay.
#[must_use = "inspect accepted failure and explicit discard"]
#[derive(Debug)]
pub enum JobAcceptance {
    /// The executor retained the job or completed its inline action normally.
    Accepted,
    /// Acceptance occurred before this engine failure was observed.
    ///
    /// A valid continuation may already have been transferred. This result
    /// cannot revoke that transfer or return an already consumed callback.
    AcceptedWithError(JobPublicationCause),
    /// The executor deliberately disposed the job under its configured policy.
    Discarded,
}

impl JobAcceptance {
    /// Finishes a producer that treats configured discard as successful policy.
    ///
    /// Producers with resources tied to a live accepted job must inspect
    /// [`Self::Discarded`] before using this reduction.
    ///
    /// # Errors
    /// Returns the exact accepted engine failure, without revoking accepted work.
    pub fn into_result(self) -> JsResult<()> {
        match self {
            Self::Accepted | Self::Discarded => Ok(()),
            Self::AcceptedWithError(cause) => Err(cause.into_error()),
        }
    }
}

/// A validated, receiving-runtime-admitted uncatchable publication cause.
#[must_use]
#[derive(Debug)]
pub struct JobPublicationCause {
    error: JsError,
}

impl JobPublicationCause {
    /// Admits an engine error at the actual receiving context.
    ///
    /// An earlier retained operation stop takes precedence. Foreign diagnostic
    /// leaves are masked by the runtime's normal error-admission rules.
    ///
    /// # Errors
    /// Returns the admitted error if it remains catchable. A catchable callback
    /// result is not an engine publication failure.
    pub fn try_new(error: JsError, context: ContextAccess<'_>) -> Result<Self, JsError> {
        let context = context.into_owner();
        let error = match context.check_execution_stop() {
            Ok(()) => error,
            Err(stop) => stop,
        };
        let error = context.runtime().admit_error(error);
        if error.as_engine().is_some() {
            Ok(Self { error })
        } else {
            Err(error)
        }
    }

    /// Borrows the full admitted cause without reconstructing its diagnostics.
    pub fn error(&self) -> &JsError {
        &self.error
    }

    /// Moves the full admitted cause to its next owner.
    pub fn into_error(self) -> JsError {
        self.error
    }

    pub(crate) fn from_receiving(
        error: crate::error::RealmOwnershipMismatch,
        context: &Context,
    ) -> Self {
        Self {
            error: error.into_publication(),
        }
        .admit(context)
    }

    pub(crate) fn from_engine(error: crate::error::EngineError, context: &Context) -> Self {
        Self {
            error: error.into(),
        }
        .admit(context)
    }

    pub(crate) fn admit(self, context: &Context) -> Self {
        let error = match context.check_execution_stop() {
            Ok(()) => self.error,
            Err(stop) => stop,
        };
        Self {
            error: context.runtime().admit_error(error),
        }
    }
}

impl fmt::Display for JobPublicationCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

impl Error for JobPublicationCause {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Owns the exact original job rejected before acceptance or invocation.
///
/// Dropping this error deliberately disposes that job. Extract it to retry
/// without replacing its callback, observer, realm or cancellation token.
#[must_use = "retry the original job or deliberately dispose it"]
#[derive(Debug)]
pub struct JobPublicationError {
    job: Job,
    cause: JobPublicationCause,
}

impl JobPublicationError {
    /// Couples an unaccepted original input with its validated receiving cause.
    pub fn new(job: Job, cause: JobPublicationCause) -> Self {
        Self { job, cause }
    }

    /// Borrows the original unaccepted input.
    pub fn job(&self) -> &Job {
        &self.job
    }

    /// Borrows its validated failure.
    pub fn cause(&self) -> &JobPublicationCause {
        &self.cause
    }

    /// Returns the original job for retry or explicit disposal and its cause.
    pub fn into_parts(self) -> (Job, JobPublicationCause) {
        (self.job, self.cause)
    }

    /// Deliberately disposes the rejected input and moves its exact cause.
    pub fn into_error(self) -> JsError {
        let (job, cause) = self.into_parts();
        drop(job);
        cause.into_error()
    }

    pub(crate) fn admit(self, context: &Context) -> Self {
        Self {
            job: self.job,
            cause: self.cause.admit(context),
        }
    }
}

impl fmt::Display for JobPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.cause.fmt(formatter)
    }
}

impl Error for JobPublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.cause)
    }
}

/// Completes ordinary producer publication, disposing only an actual rejection.
pub(crate) fn finish_publication(
    result: Result<JobAcceptance, JobPublicationError>,
) -> JsResult<()> {
    match result {
        Ok(acceptance) => acceptance.into_result(),
        Err(error) => Err(error.into_error()),
    }
}
