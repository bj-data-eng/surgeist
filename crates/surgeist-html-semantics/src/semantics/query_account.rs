//! The ledger accounting of one semantic session.
//!
//! The session's limits are the caps of one ledger scope, `html-query`. A second
//! scope, `html-query-observed`, has no caps and projects nothing into the first:
//! it records every reservation the allocator actually made before the capped
//! scope admits or refuses it, so a refusal still reports the capacity that was
//! taken. The typed usage reads work from the capped scope and the resources from
//! the observed one.

use super::*;
use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlWork};
use surgeist_ledger::{Change, Error, Operation, RefusalReason, Scope, Tariff, Tariffs};

pub(super) type LedgerError = Error<HtmlWork, HtmlResource>;

/// Widens a platform amount; an amount beyond `u64` saturates and every finite cap refuses it.
pub(super) fn wide(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

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
/// change would have reached.
pub(super) fn stop_kind(error: LedgerError) -> HtmlSemanticFailureKind {
    use HtmlSemanticFailureKind as Kind;
    let LedgerError::Refused(refusal) = error else {
        return match error {
            LedgerError::Allocation => Kind::AllocationFailed,
            _ => Kind::ProtocolViolated,
        };
    };
    let RefusalReason::Limit { limit } = refusal.reason else {
        return match refusal.reason {
            RefusalReason::Overflow => Kind::RepresentationExhausted,
            _ => Kind::ProtocolViolated,
        };
    };
    let (resource, amount) = match refusal.change {
        Change::Work(HtmlWork::SemanticStep, amount) => (HtmlSemanticResource::Work, amount),
        Change::Acquire(HtmlResource::OutputRecords, amount) => {
            (HtmlSemanticResource::OutputRecords, amount)
        }
        Change::Acquire(HtmlResource::ScratchRecords, amount) => {
            (HtmlSemanticResource::ScratchRecords, amount)
        }
        Change::Acquire(HtmlResource::AllocatedBytes, amount) => {
            (HtmlSemanticResource::AllocatedBytes, amount)
        }
        Change::Acquire(HtmlResource::NestedOutputRecords, amount) => {
            (HtmlSemanticResource::NestedOutputRecords, amount)
        }
        _ => return Kind::ProtocolViolated,
    };
    match (
        usize::try_from(limit),
        refusal
            .current
            .checked_add(amount)
            .and_then(|required| usize::try_from(required).ok()),
    ) {
        (Ok(limit), Some(required)) => Kind::LimitExceeded {
            resource,
            limit,
            required,
        },
        _ => Kind::RepresentationExhausted,
    }
}

/// The operation and scopes of one session.
pub(super) struct QueryAccount {
    operation: Operation<HtmlWork, HtmlResource>,
    caps: Scope,
    observed: Scope,
}

impl QueryAccount {
    pub(super) fn new(session: HtmlSemanticLimits) -> Result<Self, LedgerError> {
        let mut operation = Operation::new("html-query", limits(session), 2)?;
        let caps = operation.root();
        let observed = operation.child(
            caps,
            "html-query-observed",
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

    /// Admits `work` steps at the capped scope.
    pub(super) fn work(&mut self, work: usize) -> Result<(), LedgerError> {
        self.operation
            .apply(self.caps, Change::Work(HtmlWork::SemanticStep, wide(work)))
    }

    /// Checks `work` steps against the capped scope without recording them.
    pub(super) fn check_work(&mut self, work: usize) -> Result<(), LedgerError> {
        self.operation.check_batch(
            self.caps,
            &[Change::Work(HtmlWork::SemanticStep, wide(work))],
        )
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

    /// Records a batch in the observed scope only.
    pub(super) fn observe_only(&mut self, batch: &[HtmlChange]) -> Result<(), LedgerError> {
        self.operation.apply_batch(self.observed, batch)
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

    /// The typed usage.
    pub(super) fn read(&self) -> Result<HtmlSemanticUsage, LedgerError> {
        let capped = self.operation.usage(self.caps)?;
        let observed = self.operation.usage(self.observed)?;
        let read = |value: u64| usize::try_from(value).unwrap_or(usize::MAX);
        Ok(HtmlSemanticUsage {
            work: read(capped.work(HtmlWork::SemanticStep)),
            output_records: read(observed.live(HtmlResource::OutputRecords)),
            scratch_records: read(observed.live(HtmlResource::ScratchRecords)),
            allocated_bytes: read(observed.live(HtmlResource::AllocatedBytes)),
            peak_scratch_records: read(observed.peak(HtmlResource::ScratchRecords)),
            nested_output_records: read(observed.live(HtmlResource::NestedOutputRecords)),
        })
    }
}
