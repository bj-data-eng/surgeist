//! The one name a ledger condition has in every HTML crate: the accounting
//! fault, the classification of a ledger error, and the figures of a refusal.

use super::{HtmlChange, HtmlResource, HtmlWork};
use std::{error::Error, fmt};
use surgeist_ledger::{Change, Defect, Error as LedgerError, Refusal, RefusalReason, Scope};

/// A ledger failure that is not a refusal at a cap the consumer reports.
///
/// Every HTML crate reports a ledger condition under the same variant: a
/// release that matches nothing held is [`Self::ReleaseUnderflow`], a ledger
/// that cannot allocate is [`Self::Allocation`], a scope of another operation is
/// [`Self::ForeignScope`] and an operation whose scope bound is spent is
/// [`Self::ScopeLimit`]. [`classify`] derives the fault from a ledger error.
///
/// A refusal at a cap is not a fault: it is a [`HtmlCapRefusal`], which the
/// consumer reports as its typed limit stop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum HtmlAccountingFault {
    /// A release returned more than the scope held.
    ReleaseUnderflow,
    /// The ledger could not allocate its storage.
    Allocation,
    /// The supplied scope belongs to another operation.
    ForeignScope,
    /// The operation's scope bound leaves no room for the scopes the consumer
    /// creates.
    ScopeLimit {
        /// The operation's scope bound.
        limit: usize,
    },
    /// The ledger's identities are exhausted.
    IdentityExhausted,
    /// Timed profiling needs a clock the platform does not supply.
    TimingUnavailable,
    /// A checkpoint operation failed. The HTML crates open no checkpoint, so
    /// only a ledger that reports one for another reason produces it.
    Checkpoint,
    /// The ledger refused or was asked for a change that the consumer does not
    /// charge: a refusal that names a dimension the consumer does not report, a
    /// refusal where the consumer charges none, such as the creation of a scope,
    /// or a charge batch the consumer does not account.
    UnexpectedChange,
    /// The ledger found a broken invariant of its own. It stays latched in the
    /// operation.
    Defect(Defect),
}

impl fmt::Display for HtmlAccountingFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseUnderflow => {
                formatter.write_str("a release returned more than the scope held")
            }
            Self::Allocation => formatter.write_str("the ledger could not allocate its storage"),
            Self::ForeignScope => {
                formatter.write_str("the supplied scope belongs to another operation")
            }
            Self::ScopeLimit { limit } => write!(
                formatter,
                "the operation's scope bound of {limit} leaves no room for the scopes"
            ),
            Self::IdentityExhausted => formatter.write_str("the ledger's identities are exhausted"),
            Self::TimingUnavailable => formatter.write_str("timed profiling needs a host clock"),
            Self::Checkpoint => formatter.write_str("a ledger checkpoint operation failed"),
            Self::UnexpectedChange => {
                formatter.write_str("the ledger saw a change that the consumer does not charge")
            }
            Self::Defect(defect) => write!(formatter, "the ledger found a defect: {defect}"),
        }
    }
}

impl Error for HtmlAccountingFault {}

/// A refusal at a cap: the change that reached the limit of a scope.
///
/// The figures are in the units of the scope that refused. A consumer reads
/// [`Self::change`] to name the resource, [`Self::limit`] for the cap and
/// [`Self::figures`] or [`Self::required`] for the total the change would have
/// reached. An amount that does not fit is representation exhaustion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCapRefusal {
    scope: Scope,
    item: usize,
    change: HtmlChange,
    current: u64,
    limit: u64,
}

/// The limit and the required total of a refusal, narrowed to the platform's
/// `usize`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlRefusalFigures {
    /// The cap that refused.
    pub limit: usize,
    /// The total the refused change would have reached.
    pub required: usize,
}

impl HtmlCapRefusal {
    /// The scope that refused.
    pub const fn scope(&self) -> Scope {
        self.scope
    }

    /// The index of the refused change in its batch.
    pub const fn item(&self) -> usize {
        self.item
    }

    /// The refused change.
    pub const fn change(&self) -> HtmlChange {
        self.change
    }

    /// The amount of the refused change.
    pub const fn amount(&self) -> u64 {
        match self.change {
            Change::Work(_, amount) | Change::Acquire(_, amount) | Change::Release(_, amount) => {
                amount
            }
        }
    }

    /// The value of the scope the change was applied to, including the batch
    /// prefix before the change.
    pub const fn current(&self) -> u64 {
        self.current
    }

    /// The cap that refused.
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// The total the change would have reached: the value it was applied to plus
    /// its amount. A total beyond `u64` is not representable.
    pub fn required(&self) -> Result<u64, HtmlChargeOverflow> {
        self.current
            .checked_add(self.amount())
            .ok_or(HtmlChargeOverflow)
    }

    /// The limit and the required total as platform sizes. A figure beyond
    /// `usize` is not representable.
    pub fn figures(&self) -> Result<HtmlRefusalFigures, HtmlChargeOverflow> {
        let limit = usize::try_from(self.limit).map_err(|_| HtmlChargeOverflow)?;
        let required = usize::try_from(self.required()?).map_err(|_| HtmlChargeOverflow)?;
        Ok(HtmlRefusalFigures { limit, required })
    }
}

/// A ledger error split by what a consumer does next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlLedgerOutcome {
    /// A cap refused a change: the consumer reports its typed limit stop.
    Cap(HtmlCapRefusal),
    /// The arithmetic of a change overflowed `u64`: the consumer reports
    /// representation exhaustion of the resource the change names.
    Exhausted(HtmlChange),
    /// Any other ledger failure: the consumer carries the fault.
    Fault(HtmlAccountingFault),
}

/// Classifies a ledger error. This is the one place that maps the ledger's
/// error shapes onto the HTML crates' names.
pub fn classify(error: LedgerError<HtmlWork, HtmlResource>) -> HtmlLedgerOutcome {
    HtmlLedgerOutcome::Fault(match error {
        LedgerError::Refused(refusal) => return refused(refusal),
        LedgerError::Allocation => HtmlAccountingFault::Allocation,
        LedgerError::ForeignScope => HtmlAccountingFault::ForeignScope,
        LedgerError::ScopeLimit { limit } => HtmlAccountingFault::ScopeLimit { limit },
        LedgerError::IdentityExhausted => HtmlAccountingFault::IdentityExhausted,
        LedgerError::TimingUnavailable => HtmlAccountingFault::TimingUnavailable,
        LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured => HtmlAccountingFault::Checkpoint,
        LedgerError::Defect(defect) => HtmlAccountingFault::Defect(defect),
    })
}

fn refused(refusal: Refusal<HtmlWork, HtmlResource>) -> HtmlLedgerOutcome {
    match refusal.reason {
        RefusalReason::Limit { limit } => HtmlLedgerOutcome::Cap(HtmlCapRefusal {
            scope: refusal.scope,
            item: refusal.item,
            change: refusal.change,
            current: refusal.current,
            limit,
        }),
        RefusalReason::Overflow => HtmlLedgerOutcome::Exhausted(refusal.change),
        RefusalReason::Underflow => HtmlLedgerOutcome::Fault(HtmlAccountingFault::ReleaseUnderflow),
    }
}

/// A conversion amount that does not fit the ledger's `u64`, or a figure that
/// does not fit the platform's `usize`.
///
/// Callers stop with their own representation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlChargeOverflow;

impl fmt::Display for HtmlChargeOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("accounting amount is not representable")
    }
}

impl Error for HtmlChargeOverflow {}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_ledger::Operation;

    fn refusal(current: u64, amount: u64, limit: u64) -> HtmlCapRefusal {
        let operation = Operation::<HtmlWork, HtmlResource>::new(
            "scope",
            surgeist_ledger::Limits::unlimited(),
            1,
        )
        .unwrap();
        HtmlCapRefusal {
            scope: operation.root(),
            item: 0,
            change: Change::Acquire(HtmlResource::ManagedEdges, amount),
            current,
            limit,
        }
    }

    #[test]
    fn a_required_total_beyond_u64_is_not_representable() {
        let refusal = refusal(u64::MAX, 1, 3);
        assert_eq!(refusal.required(), Err(HtmlChargeOverflow));
        assert_eq!(refusal.figures(), Err(HtmlChargeOverflow));
    }

    #[test]
    fn figures_narrow_the_limit_and_the_required_total_together() {
        let refusal = refusal(4, 3, 5);
        assert_eq!(
            refusal.figures(),
            Ok(HtmlRefusalFigures {
                limit: 5,
                required: 7
            })
        );
        let wide = refusal_with(u64::MAX - 1, 1);
        assert_eq!(wide.required(), Ok(u64::MAX));
        // The narrowing fails exactly when `usize` is narrower than `u64`.
        assert_eq!(wide.figures().is_ok(), usize::BITS >= 64);
    }

    fn refusal_with(current: u64, amount: u64) -> HtmlCapRefusal {
        refusal(current, amount, u64::MAX - 1)
    }
}
