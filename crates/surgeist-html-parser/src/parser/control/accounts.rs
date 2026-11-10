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
use surgeist_ledger::{Change, Dimension, Operation, PhaseObservation, Scope, SharedOperation, Usage};

pub(crate) type LedgerUsage = Usage<HtmlWork, HtmlResource>;

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
    /// Prototype: work accumulated locally and not yet in the ledger.
    pending: [u64; 8],
    pending_sum: u64,
    credit_base: u64,
    credit_limit: u64,
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
            let credit_limit = operation
                .limits(cap)
                .map_or(0, |limits| limits.work(HtmlWork::Credits));
            return Ok(Self {
                pending: [0; 8],
                pending_sum: 0,
                credit_base: 0,
                credit_limit,
                ledger: Ledger::Owned(operation),
                cap,
                charging,
                work: charging,
                phases: None,
                observation: None,
                _lease: None,
            });
        };
        let bound = lease.bind(limits)?;
        Ok(Self {
            pending: [0; 8],
            pending_sum: 0,
            credit_base: 0,
            credit_limit: 0,
            ledger: Ledger::Shared(bound.ledger),
            cap: bound.cap,
            charging: bound.charging,
            work: bound.charging,
            phases: bound.phases,
            observation: None,
            _lease: Some(lease.clone()),
        })
    }

    pub(crate) fn work(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        const BLOCK: u64 = 1 << 16;
        const FLUSH: u64 = 1 << 20;
        if matches!(self.ledger, Ledger::Owned(_)) {
            let mut sum = 0_u64;
            let mut local = true;
            for change in changes {
                match *change {
                    Change::Work(dimension, amount) if (dimension as usize) < 8 => {
                        match sum.checked_add(amount) {
                            Some(next) => sum = next,
                            None => local = false,
                        }
                    }
                    _ => local = false,
                }
            }
            if local
                && self
                    .credit_base
                    .saturating_add(self.pending_sum)
                    .saturating_add(sum)
                    .saturating_add(BLOCK)
                    <= self.credit_limit
            {
                for change in changes {
                    if let Change::Work(dimension, amount) = *change {
                        self.pending[dimension as usize] += amount;
                    }
                }
                self.pending_sum += sum;
                if self.pending_sum >= FLUSH {
                    self.flush_work()?;
                }
                return Ok(());
            }
            self.flush_work()?;
        }
        self.ledger.apply(self.work, changes)
    }
    /// Moves locally accumulated work into the ledger.
    pub(crate) fn flush_work(&mut self) -> Result<(), LedgerError> {
        if self.pending_sum == 0 {
            return Ok(());
        }
        let mut batch = [Change::Work(HtmlWork::InputUnitRead, 0); 8];
        let mut count = 0;
        for &dimension in &HtmlWork::ALL[..8] {
            let amount = self.pending[dimension as usize];
            if amount != 0 {
                batch[count] = Change::Work(dimension, amount);
                count += 1;
            }
        }
        self.pending = [0; 8];
        self.credit_base = self.credit_base.saturating_add(self.pending_sum);
        self.pending_sum = 0;
        self.ledger.apply(self.work, &batch[..count])
    }
    pub(crate) fn charge(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        self.ledger.apply(self.charging, changes)
    }
    pub(crate) fn check(&mut self, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        self.ledger.check(self.charging, changes)
    }
    pub(crate) fn live(&self, resource: HtmlResource) -> Result<u64, LedgerError> {
        self.ledger
            .read(self.charging, |usage| usage.live(resource))
    }
    /// The credit total of the parse.
    pub(crate) fn credits(&self) -> Result<u64, LedgerError> {
        self.ledger
            .read(self.cap, |usage| usage.work(HtmlWork::Credits))
    }
    /// The cap of a parse resource, read from the scope that carries it.
    pub(crate) fn limit(&self, resource: HtmlParseResource) -> Result<u64, LedgerError> {
        let limits = self.ledger.limits(self.cap)?;
        Ok(match resource.dimension() {
            Some(dimension) => limits.live(dimension),
            None => limits.work(HtmlWork::Credits),
        })
    }
    /// The usage of the charging scope with the credit total of the cap scope.
    pub(crate) fn usage(&self) -> Result<(LedgerUsage, u64), LedgerError> {
        let usage = self.ledger.read(self.charging, |usage| *usage)?;
        Ok((usage, self.credits()?))
    }
    /// The value a change counts in `scope`: its work total or its live value.
    pub(crate) fn value_at(&self, scope: Scope, change: HtmlChange) -> Result<u64, LedgerError> {
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
        if let (Some(observation), Ledger::Shared(ledger)) = (self.observation.take(), &self.ledger)
        {
            let _ = ledger.end_phase(observation);
        }
    }
}
