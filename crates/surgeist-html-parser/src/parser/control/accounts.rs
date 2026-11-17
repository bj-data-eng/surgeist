//! The parse controller's accounts and their scopes.
//!
//! A parse that selected neither profiling nor a host operation charges a
//! ledger [`Chain`], which keeps the two parse scopes in plain counters and
//! admits batches by the rules of an operation. A parse that selected profiling
//! or runs in a host's operation charges a shared ledger operation. Both forms
//! charge the same two scopes, so the same batches reach the same admission
//! rules and the same refusals.

use crate::accounting::{Lease, LedgerError, ParsePhase, PhaseScopes, SetupFault};
use crate::{HtmlParseLimits, HtmlParseResource};
use std::sync::Arc;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlChange, HtmlLimits, HtmlResource, HtmlResourceTable, HtmlWork,
    HtmlWorkTable, charging_tariffs, fault_of,
};
use surgeist_ledger::{Chain, Change, Dimension, PhaseObservation, Scope, SharedOperation};

#[cfg(test)]
mod chain_tests;

/// The two scopes of a parse that nothing observes: the cap scope, which
/// carries the parse limits and totals the credits, and the charging scope
/// below it, which records every work category and every resource without a cap.
pub(crate) type OwnedChain = Chain<HtmlWork, HtmlResource, 1>;

/// The chain of a parse under `limits`, with the scopes, caps and tariffs of a
/// parse's own operation (`accounting::private`).
pub(crate) fn owned_chain(limits: HtmlLimits) -> Result<OwnedChain, HtmlAccountingFault> {
    Chain::new(limits, [(HtmlLimits::unlimited(), charging_tariffs())]).map_err(fault_of)
}

/// What a parse can read of a scope: its work totals and the live and peak
/// value of each resource.
pub(crate) trait Reading {
    fn work(&self, dimension: HtmlWork) -> u64;
    fn live(&self, dimension: HtmlResource) -> u64;
    fn peak(&self, dimension: HtmlResource) -> u64;
}

impl Reading for surgeist_ledger::Usage<HtmlWork, HtmlResource> {
    fn work(&self, dimension: HtmlWork) -> u64 {
        Self::work(self, dimension)
    }
    fn live(&self, dimension: HtmlResource) -> u64 {
        Self::live(self, dimension)
    }
    fn peak(&self, dimension: HtmlResource) -> u64 {
        Self::peak(self, dimension)
    }
}

/// The values a parse reads of a scope, copied out of it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Readings {
    work: HtmlWorkTable<u64>,
    live: HtmlResourceTable<u64>,
    peak: HtmlResourceTable<u64>,
}

impl Readings {
    pub(crate) fn capture(reading: &dyn Reading) -> Self {
        let mut copy = Self {
            work: HtmlWorkTable::splat(0),
            live: HtmlResourceTable::splat(0),
            peak: HtmlResourceTable::splat(0),
        };
        for &dimension in HtmlWork::ALL {
            *HtmlWork::get_mut(&mut copy.work, dimension) = reading.work(dimension);
        }
        for &dimension in HtmlResource::ALL {
            *HtmlResource::get_mut(&mut copy.live, dimension) = reading.live(dimension);
            *HtmlResource::get_mut(&mut copy.peak, dimension) = reading.peak(dimension);
        }
        copy
    }
    pub(crate) fn work(&self, dimension: HtmlWork) -> u64 {
        *HtmlWork::get(&self.work, dimension)
    }
    pub(crate) fn live(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(&self.live, dimension)
    }
    pub(crate) fn peak(&self, dimension: HtmlResource) -> u64 {
        *HtmlResource::get(&self.peak, dimension)
    }
}

/// What a parse asked of its accounts, for tests of how much accounting a
/// document costs.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AdmissionCounts {
    /// Batches admitted or checked through the accounts.
    pub(crate) batches: u64,
    /// Calls into a `surgeist-ledger` operation after setup: admissions,
    /// checks and reads.
    pub(crate) ledger_calls: u64,
}

#[cfg(test)]
std::thread_local! {
    static LAST_PARSE: std::cell::Cell<AdmissionCounts> =
        const { std::cell::Cell::new(AdmissionCounts { batches: 0, ledger_calls: 0 }) };
}

/// The counts of the accounts most recently dropped on this thread, which are
/// those of the last parse that ran here.
#[cfg(test)]
pub(crate) fn last_parse_counts() -> AdmissionCounts {
    LAST_PARSE.get()
}

pub(crate) enum Ledger {
    /// The parse's own scopes, which nothing else observes: plain counters.
    Owned(Box<OwnedChain>),
    /// An operation shared with a host or a profile.
    Shared(SharedOperation<HtmlWork, HtmlResource>),
}

impl Ledger {
    fn apply(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        match self {
            // An owned parse charges only its charging scope, the leaf of
            // its chain.
            Self::Owned(chain) => chain.apply_batch(changes),
            Self::Shared(operation) => operation.apply_batch(scope, changes),
        }
    }
    fn check(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        match self {
            // As in `apply`: the charging scope is the only one charged.
            Self::Owned(chain) => chain.check_batch(changes),
            Self::Shared(operation) => operation.check_batch(scope, changes),
        }
    }
    /// Reads a scope's usage without copying it for an owned parse.
    fn read<T>(
        &self,
        scope: Scope,
        read: impl FnOnce(&dyn Reading) -> T,
    ) -> Result<T, LedgerError> {
        match self {
            Self::Owned(chain) => chain.usage(scope).map(|usage| read(usage)),
            Self::Shared(operation) => operation.usage(scope).map(|usage| read(&usage)),
        }
    }
    fn limits(&self, scope: Scope) -> Result<HtmlLimits, LedgerError> {
        match self {
            Self::Owned(chain) => chain.limits(scope).copied(),
            Self::Shared(operation) => operation.limits(scope),
        }
    }
}

pub(crate) struct Accounts {
    ledger: Ledger,
    /// Carries the caps and the credit total.
    cap: Scope,
    /// Receives every resource and gauge, and the work of the parse when phases
    /// are not observed.
    charging: Scope,
    /// The scope that receives work-only batches.
    work: Scope,
    phases: Option<PhaseScopes>,
    observation: Option<PhaseObservation>,
    /// Held so that the supplied operation is released into when the last holder
    /// of the parse's accounting drops.
    _lease: Option<Arc<Lease>>,
    #[cfg(test)]
    batches: std::cell::Cell<u64>,
    #[cfg(test)]
    ledger_calls: std::cell::Cell<u64>,
}

impl Accounts {
    /// The accounts of a parse that keeps its figures in counters of its own,
    /// or that charges the operation the lease names.
    pub(crate) fn new(
        limits: &HtmlParseLimits,
        lease: Option<&Arc<Lease>>,
    ) -> Result<Self, SetupFault> {
        let Some(lease) = lease else {
            let chain = owned_chain(limits.ledger_limits()).map_err(SetupFault::from)?;
            let (cap, charging) = (chain.root(), chain.leaf());
            return Ok(Self {
                ledger: Ledger::Owned(Box::new(chain)),
                cap,
                charging,
                work: charging,
                phases: None,
                observation: None,
                _lease: None,
                #[cfg(test)]
                batches: std::cell::Cell::new(0),
                #[cfg(test)]
                ledger_calls: std::cell::Cell::new(0),
            });
        };
        let bound = lease.bind(limits)?;
        Ok(Self {
            ledger: Ledger::Shared(bound.ledger),
            cap: bound.cap,
            charging: bound.charging,
            work: bound.charging,
            phases: bound.phases,
            observation: None,
            _lease: Some(lease.clone()),
            #[cfg(test)]
            batches: std::cell::Cell::new(0),
            #[cfg(test)]
            ledger_calls: std::cell::Cell::new(0),
        })
    }

    /// Counts a batch that reached the accounts, and the ledger call it makes.
    #[cfg(test)]
    fn count_batch(&self) {
        self.batches.set(self.batches.get().saturating_add(1));
        self.count_call();
    }
    /// Counts a call into the ledger operation, which an owned parse does not
    /// make.
    #[cfg(test)]
    fn count_call(&self) {
        if matches!(self.ledger, Ledger::Shared(_)) {
            self.ledger_calls
                .set(self.ledger_calls.get().saturating_add(1));
        }
    }

    pub(crate) fn work(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        #[cfg(test)]
        self.count_batch();
        self.ledger.apply(self.work, changes)
    }
    pub(crate) fn charge(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        #[cfg(test)]
        self.count_batch();
        self.ledger.apply(self.charging, changes)
    }
    pub(crate) fn check(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        #[cfg(test)]
        self.count_batch();
        self.ledger.check(self.charging, changes)
    }
    pub(crate) fn live(&self, resource: HtmlResource) -> Result<u64, LedgerError> {
        #[cfg(test)]
        self.count_call();
        self.ledger
            .read(self.charging, |usage| usage.live(resource))
    }
    /// The credit total of the parse.
    pub(crate) fn credits(&self) -> Result<u64, LedgerError> {
        #[cfg(test)]
        self.count_call();
        self.ledger
            .read(self.cap, |usage| usage.work(HtmlWork::Credits))
    }
    /// The cap of a parse resource, read from the scope that carries it.
    pub(crate) fn limit(&self, resource: HtmlParseResource) -> Result<u64, LedgerError> {
        #[cfg(test)]
        self.count_call();
        let limits = self.ledger.limits(self.cap)?;
        Ok(match resource.dimension() {
            Some(dimension) => limits.live(dimension),
            None => limits.work(HtmlWork::Credits),
        })
    }
    /// The usage of the charging scope with the credit total of the cap scope.
    pub(crate) fn usage(&self) -> Result<(Readings, u64), LedgerError> {
        #[cfg(test)]
        self.count_call();
        let usage = self.ledger.read(self.charging, Readings::capture)?;
        Ok((usage, self.credits()?))
    }
    /// The value a change counts in `scope`: its work total or its live value.
    pub(crate) fn value_at(&self, scope: Scope, change: HtmlChange) -> Result<u64, LedgerError> {
        #[cfg(test)]
        self.count_call();
        self.ledger.read(scope, |usage| match change {
            Change::Work(dimension, _) => usage.work(dimension),
            Change::Acquire(dimension, _) | Change::Release(dimension, _) => usage.live(dimension),
        })
    }
    /// Whether a refusal names one of the parse's own scopes, whose figures the
    /// parser rebuilds, rather than a scope above them.
    pub(crate) fn owns(&self, scope: Scope) -> bool {
        scope == self.cap || scope == self.charging
    }

    /// Points the phases at other scopes, for a test of a failing observation.
    #[cfg(test)]
    pub(crate) fn replace_phases_for_test(&mut self, phases: PhaseScopes) {
        self.phases = Some(phases);
    }

    /// Ends the observation in progress and starts observing `phase`, whose
    /// scope receives the work that follows. Without phase scopes nothing
    /// changes.
    pub(crate) fn enter(&mut self, phase: ParsePhase) -> Result<(), LedgerError> {
        let (Some(phases), Ledger::Shared(ledger)) = (self.phases, &self.ledger) else {
            return Ok(());
        };
        if let Some(observation) = self.observation.take() {
            // An observation of another operation cannot occur: this control
            // began it on this ledger.
            let _ = ledger.end_phase(observation);
        }
        let scope = phases.scope(phase);
        self.observation = Some(ledger.begin_phase(scope)?);
        self.work = scope;
        Ok(())
    }
}

impl Drop for Accounts {
    fn drop(&mut self) {
        #[cfg(test)]
        LAST_PARSE.set(AdmissionCounts {
            batches: self.batches.get(),
            ledger_calls: self.ledger_calls.get(),
        });
        if let (Some(observation), Ledger::Shared(ledger)) = (self.observation.take(), &self.ledger)
        {
            let _ = ledger.end_phase(observation);
        }
    }
}
