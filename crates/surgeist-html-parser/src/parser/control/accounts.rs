//! The parse controller's ledger operation and its scopes.
//!
//! A parse owns its operation outright unless it selected profiling or runs in a
//! host's operation, in which case it charges a shared operation. The owned form
//! takes no lock on the admission path. Both forms charge the same scopes, so the
//! same batches reach the same admission rules.

use crate::accounting::{Lease, LedgerError, ParsePhase, PhaseScopes, SetupFault, private};
use crate::{HtmlParseLimits, HtmlParseResource};
use std::sync::Arc;
use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlWork};
use surgeist_ledger::{Change, Operation, PhaseObservation, Scope, SharedOperation, Usage};

pub(crate) type LedgerUsage = Usage<HtmlWork, HtmlResource>;

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
    Owned(Operation<HtmlWork, HtmlResource>),
    Shared(SharedOperation<HtmlWork, HtmlResource>),
}

impl Ledger {
    fn apply(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.apply_batch(scope, changes),
            Self::Shared(operation) => operation.apply_batch(scope, changes),
        }
    }
    fn check(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.check_batch(scope, changes),
            Self::Shared(operation) => operation.check_batch(scope, changes),
        }
    }
    /// Reads a scope's usage without copying it for an owned operation.
    fn read<T>(
        &self,
        scope: Scope,
        read: impl FnOnce(&LedgerUsage) -> T,
    ) -> Result<T, LedgerError> {
        match self {
            Self::Owned(operation) => operation.usage(scope).map(read),
            Self::Shared(operation) => operation.usage(scope).map(|usage| read(&usage)),
        }
    }
    fn limits(&self, scope: Scope) -> Result<HtmlLimits, LedgerError> {
        match self {
            Self::Owned(operation) => operation.limits(scope).copied(),
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
    /// The accounts of a parse that runs in its own operation, or in the
    /// operation the lease names.
    pub(crate) fn new(
        limits: &HtmlParseLimits,
        lease: Option<&Arc<Lease>>,
    ) -> Result<Self, SetupFault> {
        let Some(lease) = lease else {
            let (operation, cap, charging) = private(limits).map_err(SetupFault::from)?;
            return Ok(Self {
                ledger: Ledger::Owned(operation),
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
    /// Counts a call into the ledger operation.
    #[cfg(test)]
    fn count_call(&self) {
        self.ledger_calls
            .set(self.ledger_calls.get().saturating_add(1));
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
    pub(crate) fn usage(&self) -> Result<(LedgerUsage, u64), LedgerError> {
        #[cfg(test)]
        self.count_call();
        let usage = self.ledger.read(self.charging, |usage| *usage)?;
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
