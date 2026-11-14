use std::rc::Rc;

use crate::{Error, Exception};

use super::{LedgerError, LocalRefusal, registry};

/// Specified Web IDL validation rules that throw an intrinsic TypeError.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    Enumeration(&'static str),
    ByteString,
    IntegerRange,
    FloatingRange,
    Sequence,
    Dictionary(&'static str),
    CallbackInterface(&'static str),
    Interface(&'static str),
    RequiredMember(&'static str),
    RequiredArgument(&'static str),
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
    LocalStorageRelease(LocalRefusal),
    HostStorageRelease,
    PhaseLedger(LedgerError),
    PhaseOwnerMismatch,
    HostPhaseCompletion,
}

impl CleanupFault {
    pub fn stage(self) -> CleanupStage {
        match self {
            Self::StorageRelease(_) | Self::LocalStorageRelease(_) | Self::HostStorageRelease => {
                CleanupStage::StorageRelease
            }
            Self::PhaseLedger(_) | Self::PhaseOwnerMismatch | Self::HostPhaseCompletion => {
                CleanupStage::PhaseCompletion
            }
        }
    }
    pub fn ledger_error(self) -> Option<LedgerError> {
        match self {
            Self::StorageRelease(error) | Self::PhaseLedger(error) => Some(error),
            Self::LocalStorageRelease(_)
            | Self::HostStorageRelease
            | Self::PhaseOwnerMismatch
            | Self::HostPhaseCompletion => None,
        }
    }

    pub fn local_error(self) -> Option<LocalRefusal> {
        match self {
            Self::LocalStorageRelease(error) => Some(error),
            Self::StorageRelease(_)
            | Self::HostStorageRelease
            | Self::PhaseLedger(_)
            | Self::PhaseOwnerMismatch
            | Self::HostPhaseCompletion => None,
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
    cause: CleanupCause,
}

/// Hosted custody and local fault payloads occupy mutually exclusive variants.
#[derive(Clone, Debug)]
enum CleanupCause {
    StorageRelease(LedgerError),
    LocalStorageRelease(LocalRefusal),
    PhaseLedger(LedgerError),
    PhaseOwnerMismatch,
    HostStorageRelease,
    HostPhaseCompletion,
    Host {
        stage: CleanupStage,
        record: Rc<dyn registry::CleanupRecord>,
    },
}

impl CleanupFailure {
    pub(crate) fn from_fault(cleanup: CleanupFault) -> Self {
        let cause = match cleanup {
            CleanupFault::StorageRelease(error) => CleanupCause::StorageRelease(error),
            CleanupFault::LocalStorageRelease(error) => CleanupCause::LocalStorageRelease(error),
            CleanupFault::HostStorageRelease => CleanupCause::HostStorageRelease,
            CleanupFault::PhaseLedger(error) => CleanupCause::PhaseLedger(error),
            CleanupFault::PhaseOwnerMismatch => CleanupCause::PhaseOwnerMismatch,
            CleanupFault::HostPhaseCompletion => CleanupCause::HostPhaseCompletion,
        };
        Self {
            primary: None,
            cause,
        }
    }

    pub(crate) fn from_host(stage: CleanupStage, record: Rc<dyn registry::CleanupRecord>) -> Self {
        Self {
            primary: None,
            cause: CleanupCause::Host { stage, record },
        }
    }

    pub fn primary(&self) -> Option<&CleanupPrimary> {
        self.primary.as_ref()
    }
    pub fn cleanup(&self) -> CleanupFault {
        match &self.cause {
            CleanupCause::StorageRelease(error) => CleanupFault::StorageRelease(*error),
            CleanupCause::LocalStorageRelease(error) => CleanupFault::LocalStorageRelease(*error),
            CleanupCause::PhaseLedger(error) => CleanupFault::PhaseLedger(*error),
            CleanupCause::PhaseOwnerMismatch => CleanupFault::PhaseOwnerMismatch,
            CleanupCause::HostStorageRelease
            | CleanupCause::Host {
                stage: CleanupStage::StorageRelease,
                ..
            } => CleanupFault::HostStorageRelease,
            CleanupCause::HostPhaseCompletion
            | CleanupCause::Host {
                stage: CleanupStage::PhaseCompletion,
                ..
            } => CleanupFault::HostPhaseCompletion,
        }
    }

    /// The original typed host cleanup source, including after its execution
    /// owner has gone. Reading it enters no engine and performs no coercion.
    pub fn host_error(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            CleanupCause::Host { record, .. } => record.error(),
            _ => None,
        }
    }

    pub(crate) fn same_fault(&self, other: &Self) -> bool {
        self.cleanup() == other.cleanup()
            && match (&self.cause, &other.cause) {
                (
                    CleanupCause::Host { record: left, .. },
                    CleanupCause::Host { record: right, .. },
                ) => Rc::ptr_eq(left, right),
                (CleanupCause::Host { .. }, _) | (_, CleanupCause::Host { .. }) => false,
                _ => true,
            }
    }

    pub(crate) fn belongs_to(&self, owner: u64) -> bool {
        match &self.cause {
            CleanupCause::Host { record, .. } => record.owner() == owner,
            _ => true,
        }
    }
}

impl std::fmt::Display for CleanupFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self.cleanup().stage() {
            CleanupStage::StorageRelease => "Web IDL conversion storage release failed",
            CleanupStage::PhaseCompletion => "Web IDL conversion phase completion failed",
        })
    }
}

impl std::error::Error for CleanupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause {
            CleanupCause::StorageRelease(error) | CleanupCause::PhaseLedger(error) => Some(error),
            CleanupCause::LocalStorageRelease(error) => Some(error),
            CleanupCause::Host { record, .. } => record.error(),
            CleanupCause::HostStorageRelease
            | CleanupCause::PhaseOwnerMismatch
            | CleanupCause::HostPhaseCompletion => None,
        }
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
            None if matches!(
                self.0,
                Error::ConversionCleanup(_) | Error::PreparationCleanup(_)
            ) =>
            {
                FailureKind::Cleanup
            }
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
            Error::PreparationCleanup(failure) => match failure.primary() {
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

impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Record {
        owner: u64,
        error: std::io::Error,
    }

    impl registry::CleanupRecord for Record {
        fn owner(&self) -> u64 {
            self.owner
        }

        fn error(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.error)
        }

        fn stage(&self) -> Option<CleanupStage> {
            Some(CleanupStage::StorageRelease)
        }
    }

    fn hosted(owner: u64) -> CleanupFailure {
        CleanupFailure::from_host(
            CleanupStage::StorageRelease,
            Rc::new(Record {
                owner,
                error: std::io::Error::from(std::io::ErrorKind::BrokenPipe),
            }),
        )
    }

    #[test]
    fn typed_host_source_survives_owner_and_original_report_destruction() {
        let owner = crate::owner::OwnerState::new(7);
        let original = hosted(owner.id());
        let retained = original.clone();
        drop(original);
        drop(owner);
        let source = retained
            .host_error()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap();
        assert_eq!(source.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(
            std::error::Error::source(&retained)
                .unwrap()
                .is::<std::io::Error>()
        );
        assert!(retained.belongs_to(7));
        assert!(!retained.belongs_to(8));
    }

    #[test]
    fn failure_error_chain_reaches_the_original_typed_host_cleanup_source() {
        let failure = Failure(Error::ConversionCleanup(hosted(7)));
        let binding_error = std::error::Error::source(&failure).unwrap();
        assert!(binding_error.is::<Error>());
        let cleanup = binding_error.source().unwrap();
        assert!(cleanup.is::<CleanupFailure>());
        let original = cleanup.source().unwrap();
        assert_eq!(
            original.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::BrokenPipe
        );
    }

    #[test]
    fn hosted_fault_identity_uses_the_record_and_shared_faults_keep_summary_equality() {
        let first = hosted(7);
        let mut selected = first.clone();
        selected.primary = Some(CleanupPrimary::CallbackPanicked);
        assert!(first.same_fault(&selected));
        assert!(!first.same_fault(&hosted(7)));
        let shared = CleanupFailure::from_fault(CleanupFault::PhaseOwnerMismatch);
        let mut selected = shared.clone();
        selected.primary = Some(CleanupPrimary::CallbackPanicked);
        assert!(shared.same_fault(&selected));
        assert!(shared.belongs_to(8));
        assert!(!first.same_fault(&CleanupFailure::from_fault(
            CleanupFault::HostStorageRelease
        )));
    }

    #[test]
    fn ledger_and_local_cleanup_sources_preserve_their_original_error_types() {
        let ledger =
            CleanupFailure::from_fault(CleanupFault::StorageRelease(LedgerError::Allocation));
        assert_eq!(
            ledger.cleanup().ledger_error(),
            Some(LedgerError::Allocation)
        );
        assert_eq!(ledger.cleanup().local_error(), None);
        assert_eq!(
            std::error::Error::source(&ledger)
                .unwrap()
                .downcast_ref::<LedgerError>(),
            Some(&LedgerError::Allocation)
        );
        let local = super::super::local::LocalState::new(surgeist_ledger::Limits::unlimited())
            .release(1)
            .unwrap_err();
        let failure = CleanupFailure::from_fault(CleanupFault::LocalStorageRelease(local));
        assert_eq!(failure.cleanup().stage(), CleanupStage::StorageRelease);
        assert_eq!(failure.cleanup().ledger_error(), None);
        assert_eq!(failure.cleanup().local_error(), Some(local));
        assert_eq!(
            std::error::Error::source(&failure)
                .unwrap()
                .downcast_ref::<LocalRefusal>(),
            Some(&local)
        );
        assert_eq!(
            CleanupFault::HostPhaseCompletion.stage(),
            CleanupStage::PhaseCompletion
        );
    }
}
