//! Execution identity and its sole terminal-failure authority.
//!
//! This state owns only Rust cleanup registrations, without SDK handles,
//! contexts, callbacks or native authority. Cleanup can record a failure here
//! without entering the engine from a Rust destructor.

use std::cell::{Cell, OnceCell};
use std::rc::{Rc, Weak};

use crate::Error;
use crate::conversion::registry::{CleanupCustody, CleanupRecord, Registrations};
use crate::conversion::{CleanupFailure, CleanupFault, CleanupPrimary, CleanupStatus};

#[derive(Clone)]
enum Failure {
    Host(Error),
    Cleanup(CleanupFailure),
    PreparationCleanup(crate::preparation::CleanupFailure),
}

pub(crate) struct OwnerState {
    pub(crate) preparation: OnceCell<Rc<crate::preparation::Account>>,
    id: u64,
    closed: Cell<bool>,
    failure: Cell<Option<Failure>>,
    boundaries: Cell<usize>,
    cleanup: Cell<CleanupStatus>,
    conversion_cleanup: Registrations,
}

impl OwnerState {
    pub(crate) fn new(id: u64) -> Self {
        Self {
            preparation: OnceCell::new(),
            id,
            closed: Cell::new(false),
            failure: Cell::new(None),
            boundaries: Cell::new(0),
            cleanup: Cell::new(CleanupStatus::default()),
            conversion_cleanup: Registrations::new(),
        }
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn available(&self) -> Result<(), Error> {
        match self.failure() {
            Some(Failure::Host(error)) => Err(error),
            Some(Failure::Cleanup(cleanup)) => {
                let error = Error::ConversionCleanup(cleanup);
                if self.boundaries.get() == 0 && !std::thread::panicking() {
                    self.failure.set(Some(Failure::Host(error.clone())));
                }
                Err(error)
            }
            Some(Failure::PreparationCleanup(cleanup)) => {
                let error = Error::PreparationCleanup(cleanup);
                if self.boundaries.get() == 0 && !std::thread::panicking() {
                    self.failure.set(Some(Failure::Host(error.clone())));
                }
                Err(error)
            }
            None if self.closed.get() => Err(Error::BackendUnavailable),
            None => Ok(()),
        }
    }

    /// Admit custody of a selected cause while the active boundary is still
    /// choosing its result. Pending cleanup continues to refuse execution.
    pub(crate) fn check_host_failure_report(&self) -> Result<(), Error> {
        if self.boundaries.get() != 0
            && !self.closed.get()
            && matches!(
                self.failure(),
                Some(Failure::Cleanup(_) | Failure::PreparationCleanup(_))
            )
        {
            return Ok(());
        }
        self.available()
    }

    fn failure(&self) -> Option<Failure> {
        let failure = self.failure.take();
        let result = failure.clone();
        self.failure.set(failure);
        result
    }

    fn qualify_error(&self, error: Error) -> Error {
        match &error {
            Error::HostFailure(failure) if failure.owner != self.id => Error::ForeignOwner,
            Error::PreparationCleanup(failure) if failure.owner != self.id => Error::ForeignOwner,
            Error::ConversionCleanup(failure) if !failure.belongs_to(self.id) => {
                Error::ForeignOwner
            }
            _ => error,
        }
    }

    /// Records the first failure without touching an SDK handle. The backend
    /// requests its interrupt when it next observes this terminal state.
    pub(crate) fn fail(&self, error: Error) -> Error {
        match self.failure() {
            Some(Failure::Host(first)) => first,
            // A refused continuation reports immutable cleanup facts, but
            // does not select the outward result before the actual exit.
            Some(Failure::Cleanup(cleanup)) if matches!(&error, Error::ConversionCleanup(failure) if failure.same_fault(&cleanup)) => {
                error
            }
            Some(Failure::PreparationCleanup(cleanup)) if matches!(&error, Error::PreparationCleanup(failure) if failure.same_fault(&cleanup)) => {
                error
            }
            _ => {
                let error = self.qualify_error(error);
                self.failure.set(Some(Failure::Host(error.clone())));
                error
            }
        }
    }

    /// An enclosing host operation supplied a cause before running its
    /// completion steps. A failure reached during those steps still stops
    /// execution, but the enclosing operation's cause precedes it.
    pub(crate) fn select_prior_host_failure(&self, failure: crate::HostFailure) {
        self.failure
            .set(Some(Failure::Host(Error::HostFailure(failure))));
    }

    pub(crate) fn record_cleanup(&self, cleanup: CleanupFault) {
        self.record_conversion_failure(CleanupFailure::from_fault(cleanup));
    }

    pub(crate) fn register_conversion_cleanup(
        &self,
        custody: Weak<dyn CleanupCustody>,
        record: Rc<dyn CleanupRecord>,
    ) {
        if record.owner() != self.id {
            self.fail(Error::ForeignOwner);
            return;
        }
        self.conversion_cleanup.register(custody, record);
    }

    pub(crate) fn reap_conversion_cleanup(&self) {
        self.conversion_cleanup.reap();
    }

    pub(crate) fn record_host_cleanup(&self, record: Rc<dyn CleanupRecord>) {
        if record.owner() != self.id {
            self.fail(Error::ForeignOwner);
            return;
        }
        let Some(stage) = record.stage() else {
            self.fail(Error::RegistryConflict);
            return;
        };
        if record.error().is_none() {
            self.fail(Error::RegistryConflict);
            return;
        }
        self.record_conversion_failure(CleanupFailure::from_host(stage, record));
    }

    fn record_conversion_failure(&self, cleanup: CleanupFailure) {
        self.cleanup
            .set(self.cleanup.get().record(cleanup.cleanup()));
        if self.failure().is_none() {
            if self.boundaries.get() == 0 && !std::thread::panicking() {
                self.failure
                    .set(Some(Failure::Host(Error::ConversionCleanup(cleanup))));
            } else {
                self.failure.set(Some(Failure::Cleanup(cleanup)));
            }
        }
    }

    pub(crate) fn cleanup_status(&self) -> CleanupStatus {
        self.cleanup.get()
    }

    pub(crate) fn record_preparation_cleanup(&self, cleanup: crate::preparation::CleanupFailure) {
        if self.failure().is_none() {
            if self.boundaries.get() == 0 && !std::thread::panicking() {
                self.failure
                    .set(Some(Failure::Host(Error::PreparationCleanup(cleanup))));
            } else {
                self.failure.set(Some(Failure::PreparationCleanup(cleanup)));
            }
        }
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
                    Err(Error::ConversionCleanup(cleanup))
                }
                Err(error) => Err(combine(self.qualify_error(error), cleanup)),
            },
            Some(Failure::PreparationCleanup(mut cleanup)) => match result {
                Ok(value) => {
                    drop(value);
                    Err(Error::PreparationCleanup(cleanup))
                }
                Err(Error::Exception(exception)) => {
                    cleanup.primary = Some(CleanupPrimary::Exception(exception));
                    Err(Error::PreparationCleanup(cleanup))
                }
                Err(Error::CallbackPanicked) => {
                    cleanup.primary = Some(CleanupPrimary::CallbackPanicked);
                    Err(Error::PreparationCleanup(cleanup))
                }
                Err(error) => Err(self.qualify_error(error)),
            },
            None => result.map_err(|error| self.qualify_error(error)),
        }
    }

    pub(crate) fn finalize<T>(&self, result: Result<T, Error>) -> Result<T, Error> {
        let result = self.operation_result(result);
        if matches!(
            self.failure(),
            Some(Failure::Cleanup(_) | Failure::PreparationCleanup(_))
        ) && let Err(error) = &result
        {
            self.failure.set(Some(Failure::Host(error.clone())));
        }
        result
    }

    pub(crate) fn close(&self) {
        self.closed.set(true);
    }
}

fn combine(error: Error, mut cleanup: CleanupFailure) -> Error {
    match error {
        Error::Exception(exception) => {
            cleanup.primary = Some(CleanupPrimary::Exception(exception));
            Error::ConversionCleanup(cleanup)
        }
        Error::CallbackPanicked => {
            cleanup.primary = Some(CleanupPrimary::CallbackPanicked);
            Error::ConversionCleanup(cleanup)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::CleanupStage;

    #[derive(Debug)]
    struct OriginalFault(u64);

    impl std::fmt::Display for OriginalFault {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "original fault {}", self.0)
        }
    }

    impl std::error::Error for OriginalFault {}

    #[derive(Debug)]
    struct Record {
        owner: u64,
        error: Option<OriginalFault>,
        stage: Option<CleanupStage>,
    }

    impl CleanupRecord for Record {
        fn owner(&self) -> u64 {
            self.owner
        }

        fn error(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.error
                .as_ref()
                .map(|error| error as &dyn std::error::Error)
        }

        fn stage(&self) -> Option<CleanupStage> {
            self.stage
        }
    }

    struct Custody;
    impl CleanupCustody for Custody {}

    fn record(owner: u64, fault: u64, stage: CleanupStage) -> Rc<dyn CleanupRecord> {
        Rc::new(Record {
            owner,
            error: Some(OriginalFault(fault)),
            stage: Some(stage),
        })
    }

    fn source(failure: &CleanupFailure) -> u64 {
        failure
            .host_error()
            .unwrap()
            .downcast_ref::<OriginalFault>()
            .unwrap()
            .0
    }

    #[test]
    fn pending_host_cleanup_keeps_first_source_and_selected_callback_panic() {
        let owner = Rc::new(OwnerState::new(7));
        let boundary = Boundary::enter(owner.clone()).unwrap();
        owner.record_host_cleanup(record(7, 11, CleanupStage::StorageRelease));
        owner.record_host_cleanup(record(7, 12, CleanupStage::PhaseCompletion));
        assert!(owner.check_host_failure_report().is_ok());
        let pending = match owner.available() {
            Err(Error::ConversionCleanup(failure)) => failure,
            other => panic!("unexpected pending result: {other:?}"),
        };
        assert_eq!(source(&pending), 11);
        assert!(pending.primary().is_none());
        assert!(matches!(
            owner.fail(Error::ConversionCleanup(pending.clone())),
            Error::ConversionCleanup(_)
        ));
        assert!(matches!(owner.failure(), Some(Failure::Cleanup(_))));
        let selected = match boundary.finish::<()>(Err(Error::CallbackPanicked)) {
            Err(Error::ConversionCleanup(failure)) => failure,
            other => panic!("unexpected selected result: {other:?}"),
        };
        assert!(matches!(
            selected.primary(),
            Some(CleanupPrimary::CallbackPanicked)
        ));
        assert_eq!(source(&selected), 11);
        assert_eq!(owner.cleanup_status().failures(), Some(2));
        assert_eq!(
            owner.cleanup_status().first(),
            Some(CleanupFault::HostStorageRelease)
        );
    }

    #[test]
    fn pending_cleanup_attaches_the_preselected_exception_identity() {
        let owner = Rc::new(OwnerState::new(7));
        let boundary = Boundary::enter(owner.clone()).unwrap();
        // This fixture supplies an already selected engine-free identity; the
        // test exercises result selection, not creation of a guest exception.
        let original = crate::Exception {
            phase: crate::ExceptionPhase::Invoke,
            value: crate::Root(Rc::new(crate::RootIdentity {
                realm: crate::RealmKey { owner: 7, index: 1 },
                index: 2,
                _backing: None,
            })),
            rule: None,
        };
        owner.record_host_cleanup(record(7, 11, CleanupStage::PhaseCompletion));
        let selected = match boundary.finish::<()>(Err(Error::Exception(original.clone()))) {
            Err(Error::ConversionCleanup(failure)) => failure,
            other => panic!("unexpected selected result: {other:?}"),
        };
        let Some(CleanupPrimary::Exception(exception)) = selected.primary() else {
            panic!("the selected guest cause was not preserved");
        };
        assert!(Rc::ptr_eq(&exception.value().0, &original.value().0));
        assert_eq!(source(&selected), 11);
    }

    #[test]
    fn first_host_refusal_wins_while_secondary_and_late_cleanup_still_count() {
        let owner = OwnerState::new(7);
        assert!(matches!(owner.fail(Error::Allocation), Error::Allocation));
        owner.record_host_cleanup(record(7, 11, CleanupStage::StorageRelease));
        owner.record_cleanup(CleanupFault::PhaseOwnerMismatch);
        owner.close();
        owner.record_host_cleanup(record(7, 12, CleanupStage::PhaseCompletion));
        assert!(matches!(owner.available(), Err(Error::Allocation)));
        assert_eq!(owner.cleanup_status().failures(), Some(3));
        assert_eq!(
            owner.cleanup_status().first(),
            Some(CleanupFault::HostStorageRelease)
        );
        assert!(matches!(
            owner.finalize::<()>(Err(Error::CallbackPanicked)),
            Err(Error::Allocation)
        ));
    }

    #[test]
    fn hosted_cleanup_report_from_a_gone_foreign_owner_is_rejected() {
        let original = OwnerState::new(7);
        original.record_host_cleanup(record(7, 11, CleanupStage::StorageRelease));
        let report = match original.available() {
            Err(Error::ConversionCleanup(failure)) => failure,
            other => panic!("unexpected original result: {other:?}"),
        };
        drop(original);
        let receiving = OwnerState::new(8);
        assert!(matches!(
            receiving.fail(Error::ConversionCleanup(report)),
            Error::ForeignOwner
        ));
        let receiving = OwnerState::new(8);
        receiving.record_host_cleanup(record(7, 11, CleanupStage::StorageRelease));
        assert!(matches!(receiving.available(), Err(Error::ForeignOwner)));
        assert_eq!(receiving.cleanup_status().failures(), Some(0));
    }

    #[test]
    fn uninitialized_host_cleanup_record_is_a_registry_refusal_without_a_fake_stage() {
        for (stage, error) in [
            (None, Some(OriginalFault(11))),
            (Some(CleanupStage::StorageRelease), None),
        ] {
            let owner = OwnerState::new(7);
            owner.record_host_cleanup(Rc::new(Record {
                owner: 7,
                error,
                stage,
            }));
            assert!(matches!(owner.available(), Err(Error::RegistryConflict)));
            assert_eq!(owner.cleanup_status().failures(), Some(0));
        }
    }

    #[test]
    fn owner_registration_holds_only_records_and_reaps_expired_custody() {
        let owner = OwnerState::new(7);
        let custody: Rc<dyn CleanupCustody> = Rc::new(Custody);
        let retained = record(7, 11, CleanupStage::StorageRelease);
        let observed = Rc::downgrade(&retained);
        owner.register_conversion_cleanup(Rc::downgrade(&custody), retained);
        owner.reap_conversion_cleanup();
        assert!(observed.upgrade().is_some());
        drop(custody);
        owner.reap_conversion_cleanup();
        assert!(observed.upgrade().is_none());

        let custody: Rc<dyn CleanupCustody> = Rc::new(Custody);
        let retained = record(7, 12, CleanupStage::PhaseCompletion);
        let observed = Rc::downgrade(&retained);
        owner.register_conversion_cleanup(Rc::downgrade(&custody), retained);
        drop(owner);
        assert!(observed.upgrade().is_none());
        assert_eq!(Rc::strong_count(&custody), 1);
    }
}
