use crate::{Error, Exception};

use super::LedgerError;

/// Specified Web IDL validation rules that throw an intrinsic TypeError.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    Enumeration(&'static str),
    ByteString,
    IntegerRange,
    FloatingRange,
}

/// Cleanup activity that failed after its value or phase was finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupStage {
    StorageRelease,
    PhaseCompletion,
}

/// An immutable cleanup fault, independent of the selected primary failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupFault {
    StorageRelease(LedgerError),
    PhaseLedger(LedgerError),
    PhaseOwnerMismatch,
}

impl CleanupFault {
    pub fn stage(self) -> CleanupStage {
        match self {
            Self::StorageRelease(_) => CleanupStage::StorageRelease,
            Self::PhaseLedger(_) | Self::PhaseOwnerMismatch => CleanupStage::PhaseCompletion,
        }
    }
    pub fn ledger_error(self) -> Option<LedgerError> {
        match self {
            Self::StorageRelease(error) | Self::PhaseLedger(error) => Some(error),
            Self::PhaseOwnerMismatch => None,
        }
    }
}

/// Selected failure that preceded fatal cleanup at an execution exit.
#[derive(Clone, Debug)]
pub enum CleanupPrimary {
    Exception(Exception),
    CallbackPanicked,
}

/// Immutable facts of a cleanup-triggered refusal. An actual execution exit
/// attaches its selected primary; an earlier refused continuation can report
/// the cleanup alone. Neither report is subsequently mutated. Constructing
/// this inline report allocates nothing and never coerces a guest value.
#[derive(Clone, Debug)]
pub struct CleanupFailure {
    pub(crate) primary: Option<CleanupPrimary>,
    pub(crate) cleanup: CleanupFault,
}

impl CleanupFailure {
    pub fn primary(&self) -> Option<&CleanupPrimary> {
        self.primary.as_ref()
    }
    pub fn cleanup(&self) -> CleanupFault {
        self.cleanup
    }
}

/// Primary classification; cleanup can additionally make any result terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    TypeError(Rule),
    GuestException,
    HostRefusal,
    Cleanup,
}

/// Conversion failure propagated naturally through a public callback's `?`.
/// Normal guest failures remain `Error::Exception`; fatal cleanup is an
/// uncatchable host failure retaining the exact selected exception separately.
#[derive(Clone, Debug)]
pub struct Failure(pub(crate) Error);

impl Failure {
    pub fn kind(&self) -> FailureKind {
        match self.exception() {
            Some(exception) => exception
                .conversion_rule()
                .map_or(FailureKind::GuestException, FailureKind::TypeError),
            None if matches!(self.0, Error::ConversionCleanup(_)) => FailureKind::Cleanup,
            None => FailureKind::HostRefusal,
        }
    }
    pub fn exception(&self) -> Option<&Exception> {
        match &self.0 {
            Error::Exception(exception) => Some(exception),
            Error::ConversionCleanup(failure) => match failure.primary() {
                Some(CleanupPrimary::Exception(exception)) => Some(exception),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn cleanup_failure(&self) -> Option<&CleanupFailure> {
        match &self.0 {
            Error::ConversionCleanup(failure) => Some(failure),
            _ => None,
        }
    }
    pub fn error(&self) -> &Error {
        &self.0
    }
}

impl From<Failure> for Error {
    fn from(failure: Failure) -> Self {
        failure.0
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for Failure {}
