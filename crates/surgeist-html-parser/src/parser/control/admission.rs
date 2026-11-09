//! Admission of the controller's charges through its ledger operation, and the
//! mapping of a ledger refusal back to the parse stop the charge reports.
//!
//! The ledger decides whether a batch is admitted. A refusal leaves usage, peaks
//! and custody unchanged, so the stop's figures are read from the ledger after
//! the refusal and are the figures from before the batch.
use super::*;
#[cfg(test)]
use surgeist_html::accounting::HtmlCustody;
use surgeist_ledger::Dimension;

/// How a refusal at one of the parse's own scopes is reported.
#[derive(Clone, Copy)]
pub(super) struct Site {
    /// The invariant that a release beyond the held custody is.
    pub(super) underflow: HtmlParseInvariant,
    /// The requested change reported for [`HtmlParseResource::ManagedCapacityBytes`]
    /// when it differs from the refused amount: the net growth of a replacement.
    pub(super) growth: Option<u64>,
}

impl Site {
    /// A release beyond what is held is a release underflow.
    pub(super) const RELEASE: Self = Self {
        underflow: HtmlParseInvariant::ReleaseUnderflow,
        growth: None,
    };
    /// A release of an old layout beyond what is held disagrees with the
    /// capacity the caller was charged.
    pub(super) const REPLACEMENT: Self = Self {
        underflow: HtmlParseInvariant::CapacityMismatch,
        growth: None,
    };
    pub(super) const fn growth(self, growth: u64) -> Self {
        Self {
            growth: Some(growth),
            ..self
        }
    }
}

/// The parse resource a ledger change names; a work change names the credit
/// total.
fn resource_of(change: HtmlChange) -> Option<HtmlParseResource> {
    match change {
        Change::Work(..) => Some(HtmlParseResource::WorkCredits),
        Change::Acquire(dimension, _) | Change::Release(dimension, _) => {
            HtmlParseResource::from_dimension(dimension)
        }
    }
}

/// The amount a change carries in the units of the scope that refused it.
const fn amount_of(change: HtmlChange) -> u64 {
    match change {
        Change::Work(_, amount) | Change::Acquire(_, amount) | Change::Release(_, amount) => amount,
    }
}

impl ParseControl {
    pub(super) fn accounts_mut(&mut self) -> Result<&mut Accounts, HtmlParseStop> {
        if self.accounts.is_none() {
            // Setup latched its fault when it failed, so the stop exists.
            return Err(match &self.stopped {
                Some(first) => first.reason,
                None => HtmlParseStop::InternalInvariant(HtmlParseInvariant::LedgerState),
            });
        }
        match self.accounts.as_mut() {
            Some(accounts) => Ok(accounts),
            None => Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::LedgerState,
            )),
        }
    }

    /// Starts observing a parse phase, whose scope receives the work that
    /// follows, when the parse observes phases. A ledger failure stops the parse
    /// at its next admission.
    pub(crate) fn enter_phase(&mut self, phase: ParsePhase) {
        let Some(accounts) = self.accounts.as_mut() else {
            return;
        };
        if let Err(error) = accounts.enter(phase) {
            self.ledger_stop(error, Site::RELEASE);
        }
    }

    /// Applies a batch of work to the scope that receives work.
    pub(super) fn admit_work(&mut self, changes: &[HtmlChange]) -> Result<(), HtmlParseStop> {
        let result = self.accounts_mut()?.work(changes);
        result.map_err(|error| self.work_stop(error, changes))
    }

    /// Applies a batch of resource changes to the charging scope.
    pub(super) fn admit_resources(
        &mut self,
        changes: &[HtmlChange],
        site: Site,
    ) -> Result<(), HtmlParseStop> {
        let result = self.accounts_mut()?.charge(changes);
        result.map_err(|error| self.ledger_stop(error, site))
    }

    /// Checks a batch of resource changes the way admission would, committing
    /// nothing.
    pub(super) fn check_resources(
        &mut self,
        changes: &[HtmlChange],
        site: Site,
    ) -> Result<(), HtmlParseStop> {
        let result = self.accounts_mut()?.check(changes);
        result.map_err(|error| self.ledger_stop(error, site))
    }

    /// The live value of a resource.
    pub(super) fn live(&mut self, resource: HtmlParseResource) -> Result<u64, HtmlParseStop> {
        let Some(dimension) = resource.dimension() else {
            let credits = self.accounts_mut()?.credits();
            return credits.map_err(|error| self.ledger_stop(error, Site::RELEASE));
        };
        let live = self.accounts_mut()?.live(dimension);
        live.map_err(|error| self.ledger_stop(error, Site::RELEASE))
    }

    /// The cap of a resource, read from the ledger scope that carries it.
    pub(super) fn limit(&mut self, resource: HtmlParseResource) -> Result<u64, HtmlParseStop> {
        let limit = self.accounts_mut()?.limit(resource);
        limit.map_err(|error| self.ledger_stop(error, Site::RELEASE))
    }

    /// Records `value` as the largest reached by a gauge resource, in a batch
    /// with `changes`, the gauge last.
    pub(super) fn gauge_changes(
        &mut self,
        resource: HtmlResource,
        value: u64,
    ) -> Result<[HtmlChange; 2], HtmlParseStop> {
        let held = self.accounts_mut()?.live(resource);
        let held = held.map_err(|error| self.ledger_stop(error, Site::RELEASE))?;
        Ok(record_largest(resource, held, value))
    }

    /// The stop for a ledger error that refused a batch of resource changes.
    pub(super) fn ledger_stop(&mut self, error: LedgerError, site: Site) -> HtmlParseStop {
        match classify(error) {
            Classified::Fault(fault) => self.latch_stop(HtmlParseStop::Accounting(fault)),
            Classified::Refusal(refusal) => self.refusal_stop(refusal, site),
        }
    }

    fn refusal_stop(
        &mut self,
        refusal: Refusal<HtmlWork, HtmlResource>,
        site: Site,
    ) -> HtmlParseStop {
        let Some(resource) = resource_of(refusal.change) else {
            return self.fail_invariant(HtmlParseInvariant::LedgerState);
        };
        let owned = self
            .accounts
            .as_ref()
            .is_some_and(|accounts| accounts.owns(refusal.scope));
        let amount = amount_of(refusal.change);
        match refusal.reason {
            RefusalReason::Overflow => self.stop_representation(resource),
            RefusalReason::Underflow => {
                self.latch_stop(HtmlParseStop::InternalInvariant(site.underflow))
            }
            RefusalReason::Limit { limit } if !owned => {
                // A scope above the parse's own refused the amount it received.
                self.latch_stop(HtmlParseStop::LimitExceeded {
                    resource,
                    limit,
                    current: refusal.current,
                    requested_delta: amount,
                })
            }
            RefusalReason::Limit { limit } => {
                // The refused batch changed nothing, so the live value now is
                // the value before the batch.
                let current = match self.live(resource) {
                    Ok(current) => current,
                    Err(stop) => return stop,
                };
                let requested_delta = match resource {
                    HtmlParseResource::TreeDepth => amount.saturating_sub(current),
                    HtmlParseResource::ManagedCapacityBytes => site.growth.unwrap_or(amount),
                    _ => amount,
                };
                self.latch_stop(HtmlParseStop::LimitExceeded {
                    resource,
                    limit,
                    current,
                    requested_delta,
                })
            }
        }
    }

    /// The stop for a ledger error that refused a work batch. The ledger names
    /// one item with the prefix of the batch as its current value, while the
    /// parser checks the credit cap once for the whole batch: it reports the
    /// credits before the batch and the sum of every item, and reports an
    /// arithmetic overflow of any item before a cap.
    fn work_stop(&mut self, error: LedgerError, changes: &[HtmlChange]) -> HtmlParseStop {
        let refusal = match classify(error) {
            Classified::Fault(fault) => {
                return self.latch_stop(HtmlParseStop::Accounting(fault));
            }
            Classified::Refusal(refusal) => refusal,
        };
        let owned = self
            .accounts
            .as_ref()
            .is_some_and(|accounts| accounts.owns(refusal.scope));
        if !owned {
            return self.refusal_stop(refusal, Site::RELEASE);
        }
        let Some((usage, credits)) = self
            .accounts
            .as_ref()
            .and_then(|accounts| accounts.usage().ok())
        else {
            return self.fail_invariant(HtmlParseInvariant::LedgerState);
        };
        let mut totals = [0_u64; HtmlWork::ALL.len()];
        for (total, &dimension) in totals.iter_mut().zip(HtmlWork::ALL) {
            *total = usage.work(dimension);
        }
        let mut increment = 0_u64;
        for &change in changes {
            let Change::Work(dimension, units) = change else {
                return self.fail_invariant(HtmlParseInvariant::LedgerState);
            };
            let slot = HtmlWork::ALL
                .iter()
                .position(|candidate| *candidate == dimension)
                .and_then(|index| totals.get_mut(index));
            let (Some(slot), Some(total)) = (slot, increment.checked_add(units)) else {
                return self.stop_representation(HtmlParseResource::WorkCredits);
            };
            let Some(next) = slot.checked_add(units) else {
                return self.stop_representation(HtmlParseResource::WorkCredits);
            };
            *slot = next;
            increment = total;
        }
        if credits.checked_add(increment).is_none() {
            return self.stop_representation(HtmlParseResource::WorkCredits);
        }
        match refusal.reason {
            RefusalReason::Limit { limit } => self.latch_stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                limit,
                current: credits,
                requested_delta: increment,
            }),
            RefusalReason::Overflow => self.stop_representation(HtmlParseResource::WorkCredits),
            RefusalReason::Underflow => self.latch_stop(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::LedgerState,
            )),
        }
    }

    /// The stop for a refused storage batch built from `requests`. An overflow
    /// of any request, in request order, precedes a cap in the parser.
    pub(super) fn storage_stop(
        &mut self,
        error: LedgerError,
        requests: &[(HtmlParseResource, u64)],
    ) -> HtmlParseStop {
        if let Some(resource) = self.storage_overflow(requests) {
            return self.stop_representation(resource);
        }
        self.ledger_stop(error, Site::RELEASE)
    }

    /// The first request, in order, whose live value plus the requests so far
    /// does not fit in a `u64`.
    pub(super) fn storage_overflow(
        &self,
        requests: &[(HtmlParseResource, u64)],
    ) -> Option<HtmlParseResource> {
        let (usage, _) = self.accounts.as_ref()?.usage().ok()?;
        let mut values = ResourceTable::filled(0_u64);
        let mut deltas = ResourceTable::filled(0_u64);
        for resource in RESOURCES {
            if let Some(dimension) = resource.dimension() {
                *values.get_mut(resource) = usage.live(dimension);
            }
        }
        for &(resource, delta) in requests {
            let (Some(value), Some(sum)) = (
                values.copy(resource).checked_add(delta),
                deltas.copy(resource).checked_add(delta),
            ) else {
                return Some(resource);
            };
            *values.get_mut(resource) = value;
            *deltas.get_mut(resource) = sum;
        }
        None
    }
}

#[cfg(test)]
impl ParseControl {
    /// Moves the live value of a resource to `value` by an acquisition or a
    /// release, as a test that stages a state needs.
    pub(crate) fn set(&mut self, resource: HtmlParseResource, value: u64) {
        let dimension = resource.dimension().expect("a resource");
        let live = self.live(resource).expect("a readable ledger");
        let change = if value >= live {
            Change::Acquire(dimension, value - live)
        } else {
            Change::Release(dimension, live - value)
        };
        self.accounts_mut()
            .expect("accounts")
            .charge(&[change])
            .expect("a staged live value inside the caps");
    }
    /// Lowers one cap of a control that runs in its own operation. The ledger
    /// fixes caps when a scope is created, so the control's operation is
    /// rebuilt with the new cap and the work, live values and peaks so far are
    /// replayed into it. A cap below a live value cannot be replayed, except for
    /// the last value of a gauge, which replays as nothing held; a peak above
    /// the lowered cap is not replayed.
    pub(crate) fn test_limit(&mut self, resource: HtmlParseResource, limit: u64) {
        let usage = self.usage();
        let limits = self.configured.with_cap(resource, limit);
        let mut rebuilt = Accounts::new(&limits, None).expect("a private operation");
        for category in HtmlParseWorkCategory::ALL {
            rebuilt
                .work(&[category.change(usage.work(category))])
                .expect("replayed work");
        }
        for parse in RESOURCES {
            let Some(dimension) = parse.dimension() else {
                continue;
            };
            let reading = usage.resource(parse);
            let cap = limits.values().copy(parse);
            // The last value of a gauge is not custody: above the lowered cap it
            // is replayed as nothing held. A peak above the cap is not replayed.
            let live = match reading.current <= cap || dimension.custody() != HtmlCustody::Gauge {
                true => reading.current,
                false => 0,
            };
            let excess = if reading.high_water <= cap {
                reading.high_water.saturating_sub(live)
            } else {
                0
            };
            rebuilt
                .charge(&[
                    Change::Acquire(dimension, live),
                    Change::Acquire(dimension, excess),
                    Change::Release(dimension, excess),
                ])
                .expect("replayed live value and peak under the lowered cap");
        }
        self.configured = limits;
        self.accounts = Some(rebuilt);
    }
}
