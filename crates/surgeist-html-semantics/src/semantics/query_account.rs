//! The ledger accounting of one semantic session.
//!
//! The session's limits are the caps of one ledger scope, `html-query`. A second
//! scope, `html-query-observed`, has no caps and projects nothing into the first:
//! it records every reservation the allocator actually made before the capped
//! scope admits or refuses it, so a refusal still reports the capacity that was
//! taken. The typed usage reads work from the capped scope and the resources from
//! the observed one.

use super::*;
use crate::accounting::Ledger;
pub(super) use crate::accounting::wide;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlChange, HtmlLedgerOutcome,
    HtmlLimits, HtmlResource, HtmlWork, aggregate_tariffs, classify,
};
use surgeist_ledger::{Change, Operation, Scope, Tariff, Tariffs};

pub(super) use crate::accounting::LedgerError;

/// The ledger dimension of a record resource of the session.
pub(super) fn record_dimension(resource: HtmlSemanticResource) -> HtmlResource {
    match resource {
        HtmlSemanticResource::ScratchRecords => HtmlResource::ScratchRecords,
        HtmlSemanticResource::NestedOutputRecords => HtmlResource::NestedOutputRecords,
        _ => HtmlResource::OutputRecords,
    }
}

fn limits(limits: HtmlSemanticLimits) -> HtmlLimits {
    HtmlLimits::from_fn(
        |work| match work {
            HtmlWork::SemanticStep => wide(limits.work),
            _ => u64::MAX,
        },
        |resource| match resource {
            HtmlResource::OutputRecords => wide(limits.output_records),
            HtmlResource::ScratchRecords => wide(limits.scratch_records),
            HtmlResource::AllocatedBytes => wide(limits.allocated_bytes),
            HtmlResource::NestedOutputRecords => wide(limits.nested_output_records),
            _ => u64::MAX,
        },
    )
}

/// The typed stop of a ledger error: a refused cap keeps its resource, its limit and the total the
/// change would have reached, and every other failure is the accounting fault.
pub(super) fn stop_kind(error: LedgerError) -> HtmlSemanticFailureKind {
    use HtmlSemanticFailureKind as Kind;
    let refusal = match classify(error) {
        HtmlLedgerOutcome::Fault(fault) => return Kind::Accounting(fault),
        HtmlLedgerOutcome::Exhausted(_) => return Kind::RepresentationExhausted,
        HtmlLedgerOutcome::Cap(refusal) => refusal,
    };
    let resource = match refusal.change() {
        Change::Work(HtmlWork::SemanticStep, _) => HtmlSemanticResource::Work,
        Change::Acquire(HtmlResource::OutputRecords, _) => HtmlSemanticResource::OutputRecords,
        Change::Acquire(HtmlResource::ScratchRecords, _) => HtmlSemanticResource::ScratchRecords,
        Change::Acquire(HtmlResource::AllocatedBytes, _) => HtmlSemanticResource::AllocatedBytes,
        Change::Acquire(HtmlResource::NestedOutputRecords, _) => {
            HtmlSemanticResource::NestedOutputRecords
        }
        _ => return Kind::Accounting(HtmlAccountingFault::UnexpectedChange),
    };
    match refusal.figures() {
        Ok(figures) => Kind::LimitExceeded {
            resource,
            limit: figures.limit,
            required: figures.required,
        },
        Err(_) => Kind::RepresentationExhausted,
    }
}

/// The operation and scopes of one session.
pub(super) struct QueryAccount {
    operation: Ledger,
    caps: Scope,
    observed: Scope,
}

impl QueryAccount {
    /// The scopes of a session: below the host's scope when it supplies one, else in an operation
    /// of the session's own whose root carries the caps.
    pub(super) fn new(
        session: HtmlSemanticLimits,
        accounting: Option<&HtmlHostScope>,
    ) -> Result<Self, LedgerError> {
        let (mut operation, caps, parent) = match accounting {
            Some(host) => {
                let mut operation = Ledger::Shared(host.ledger().clone());
                let caps = operation.child(
                    host.scope(),
                    HtmlCapScope::Query.name(),
                    limits(session),
                    aggregate_tariffs(),
                )?;
                (operation, caps, host.scope())
            }
            None => {
                let operation = Operation::new(HtmlCapScope::Query.name(), limits(session), 2)?;
                let caps = operation.root();
                (Ledger::Owned(operation), caps, caps)
            }
        };
        let observed = operation.child(
            parent,
            HtmlAuxiliaryScope::QueryObserved.name(),
            HtmlLimits::unlimited(),
            Tariffs::from_fn(
                |work| Tariff::new(work, 0),
                |resource| Tariff::new(resource, 0),
            ),
        )?;
        Ok(Self {
            operation,
            caps,
            observed,
        })
    }

    /// Points the account at scopes of another operation, so that every later read of it fails
    /// as the read of a ledger with a latched defect does.
    #[cfg(test)]
    pub(super) fn point_at_another_operation(&mut self) {
        let other = Operation::<HtmlWork, HtmlResource>::new("other", HtmlLimits::unlimited(), 1)
            .expect("a one-scope operation");
        self.caps = other.root();
        self.observed = other.root();
    }

    /// Admits `work` steps at the capped scope.
    pub(super) fn work(&mut self, work: usize) -> Result<(), LedgerError> {
        self.operation
            .apply(self.caps, Change::Work(HtmlWork::SemanticStep, wide(work)))
    }

    /// Checks a batch against the capped scope without recording it.
    pub(super) fn check(&mut self, batch: &[HtmlChange]) -> Result<(), LedgerError> {
        self.operation.check_batch(self.caps, batch)
    }

    /// Records a batch the allocator already made: the observed scope takes it unconditionally,
    /// then the capped scope admits or refuses it.
    pub(super) fn observe(&mut self, batch: &[HtmlChange]) -> Result<(), LedgerError> {
        self.operation.apply_batch(self.observed, batch)?;
        self.operation.apply_batch(self.caps, batch)
    }

    /// Returns scratch backing: the observed scope gives back `records`, which it must hold, then
    /// the capped scope gives back what it holds of them. After a refused overshoot the capped
    /// scope holds less than the observed one.
    pub(super) fn release_scratch(&mut self, records: usize) -> Result<(), LedgerError> {
        self.operation.apply(
            self.observed,
            Change::Release(HtmlResource::ScratchRecords, wide(records)),
        )?;
        let held = self
            .operation
            .usage(self.caps)?
            .live(HtmlResource::ScratchRecords);
        self.operation.apply(
            self.caps,
            Change::Release(HtmlResource::ScratchRecords, held.min(wide(records))),
        )
    }

    /// The ledger and the capped and observed scopes, for an operation that charges in them
    /// itself.
    pub(super) fn scopes(&mut self) -> (&mut Ledger, Scope, Scope) {
        (&mut self.operation, self.caps, self.observed)
    }

    /// The typed usage.
    pub(super) fn read(&self) -> Result<HtmlSemanticUsage, LedgerError> {
        let read = |value: u64| usize::try_from(value).unwrap_or(usize::MAX);
        let work = self.operation.read(self.caps, |capped| {
            read(capped.work(HtmlWork::SemanticStep))
        })?;
        self.operation
            .read(self.observed, |observed| HtmlSemanticUsage {
                work,
                output_records: read(observed.live(HtmlResource::OutputRecords)),
                scratch_records: read(observed.live(HtmlResource::ScratchRecords)),
                allocated_bytes: read(observed.live(HtmlResource::AllocatedBytes)),
                peak_scratch_records: read(observed.peak(HtmlResource::ScratchRecords)),
                nested_output_records: read(observed.live(HtmlResource::NestedOutputRecords)),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::accounting::HtmlAccountingFault as Fault;
    use surgeist_ledger::{Defect, Refusal, RefusalReason};

    fn limits(scratch: usize, bytes: usize) -> HtmlSemanticLimits {
        HtmlSemanticLimits::new(100, 100, scratch, 0, bytes, 100)
    }

    fn scratch(records: u64) -> [HtmlChange; 2] {
        [
            Change::Acquire(HtmlResource::ScratchRecords, records),
            Change::Acquire(HtmlResource::AllocatedBytes, records * 8),
        ]
    }

    fn live(account: &QueryAccount, scope: Scope, resource: HtmlResource) -> u64 {
        account
            .operation
            .read(scope, |usage| usage.live(resource))
            .unwrap()
    }

    #[test]
    fn a_refused_overshoot_is_observed_and_each_scope_gives_back_its_own_amount() {
        let mut account = QueryAccount::new(limits(2, 1000), None).unwrap();
        // Two records fit, then the allocator exposes three more: the caps refuse them but the
        // observed scope has taken them.
        account.observe(&scratch(2)).unwrap();
        let refused = account.observe(&scratch(3)).unwrap_err();
        assert_eq!(
            stop_kind(refused),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::ScratchRecords,
                limit: 2,
                required: 5,
            }
        );
        assert_eq!(account.read().unwrap().scratch_records, 5);
        assert_eq!(
            live(&account, account.caps, HtmlResource::ScratchRecords),
            2
        );
        assert_eq!(
            live(&account, account.observed, HtmlResource::ScratchRecords),
            5
        );
        // The backing of the two vectors comes back in either order, and both scopes end at zero.
        account.release_scratch(3).unwrap();
        account.release_scratch(2).unwrap();
        for scope in [account.caps, account.observed] {
            assert_eq!(live(&account, scope, HtmlResource::ScratchRecords), 0);
        }
        assert_eq!(account.read().unwrap().scratch_records, 0);
        assert_eq!(account.read().unwrap().peak_scratch_records, 5);
    }

    #[test]
    fn a_release_that_matches_no_reservation_is_refused_and_changes_nothing() {
        let mut account = QueryAccount::new(limits(10, 1000), None).unwrap();
        account.observe(&scratch(2)).unwrap();
        let before = account.read().unwrap();
        let refused = account.release_scratch(3).unwrap_err();
        assert!(matches!(
            refused,
            LedgerError::Refused(Refusal {
                reason: RefusalReason::Underflow,
                ..
            })
        ));
        assert_eq!(account.read().unwrap(), before);
        assert_eq!(
            live(&account, account.caps, HtmlResource::ScratchRecords),
            2
        );
    }

    #[test]
    fn bytes_are_refused_after_records_and_keep_their_own_limit() {
        let mut account = QueryAccount::new(limits(10, 30), None).unwrap();
        account.observe(&scratch(2)).unwrap();
        // Records fit; the bytes of three more records do not.
        let refused = account.check(&scratch(3)).unwrap_err();
        assert_eq!(
            stop_kind(refused),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::AllocatedBytes,
                limit: 30,
                required: 40,
            }
        );
        // Both over: the records are named first.
        let refused = account.check(&scratch(20)).unwrap_err();
        assert!(matches!(
            stop_kind(refused),
            HtmlSemanticFailureKind::LimitExceeded {
                resource: HtmlSemanticResource::ScratchRecords,
                ..
            }
        ));
    }

    #[test]
    fn every_ledger_error_maps_to_one_typed_stop() {
        use HtmlSemanticFailureKind as Kind;
        let scope = QueryAccount::new(limits(1, 1), None).unwrap().caps;
        let refusal = |reason, change| {
            LedgerError::Refused(Refusal {
                scope,
                item: 0,
                change,
                current: 4,
                reason,
            })
        };
        let step = Change::Work(HtmlWork::SemanticStep, 3);
        assert_eq!(
            stop_kind(refusal(RefusalReason::Limit { limit: 5 }, step)),
            Kind::LimitExceeded {
                resource: HtmlSemanticResource::Work,
                limit: 5,
                required: 7,
            }
        );
        assert_eq!(
            stop_kind(refusal(RefusalReason::Overflow, step)),
            Kind::RepresentationExhausted
        );
        assert_eq!(
            stop_kind(refusal(RefusalReason::Underflow, step)),
            Kind::Accounting(Fault::ReleaseUnderflow)
        );
        // A cap refusal that names a dimension the session does not report.
        assert_eq!(
            stop_kind(refusal(
                RefusalReason::Limit { limit: 5 },
                Change::Acquire(HtmlResource::TreeDepth, 1)
            )),
            Kind::Accounting(Fault::UnexpectedChange)
        );
        assert_eq!(
            stop_kind(LedgerError::Allocation),
            Kind::Accounting(Fault::Allocation)
        );
        for (error, fault) in [
            (LedgerError::ForeignScope, Fault::ForeignScope),
            (
                LedgerError::ScopeLimit { limit: 2 },
                Fault::ScopeLimit { limit: 2 },
            ),
            (LedgerError::IdentityExhausted, Fault::IdentityExhausted),
            (LedgerError::TimingUnavailable, Fault::TimingUnavailable),
            (LedgerError::StaleCheckpoint, Fault::Checkpoint),
            (
                LedgerError::Defect(Defect::ScopeEntry),
                Fault::Defect(Defect::ScopeEntry),
            ),
        ] {
            assert_eq!(stop_kind(error), Kind::Accounting(fault));
        }
    }
}
