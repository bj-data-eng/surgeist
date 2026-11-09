//! Where capture, import, fact constructors and semantic queries charge their work and resources.
//!
//! Each charges an operation of its own unless the host supplies a
//! [`HtmlHostScope`](surgeist_html::accounting::HtmlHostScope); the schema module documents the
//! scopes each opens below it.

use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlTariffs, HtmlWork};
use surgeist_ledger::{Charge, Error, Operation, Scope, SharedOperation, Usage};

/// The ledger error type of the HTML schema pair.
pub(crate) type LedgerError = Error<HtmlWork, HtmlResource>;

/// Widens a platform amount; an amount beyond `u64` saturates and every finite cap refuses it.
pub(crate) fn wide(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// The operation a builder or session charges: its own, or the host's.
pub(crate) enum Ledger {
    Owned(Operation<HtmlWork, HtmlResource>),
    Shared(SharedOperation<HtmlWork, HtmlResource>),
}

impl Ledger {
    pub(crate) fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: HtmlLimits,
        tariffs: HtmlTariffs,
    ) -> Result<Scope, LedgerError> {
        match self {
            Self::Owned(operation) => operation.child(parent, name, limits, tariffs),
            Self::Shared(operation) => operation.child(parent, name, limits, tariffs),
        }
    }

    #[cfg(test)]
    pub(crate) fn scope_name(&self, scope: Scope) -> Result<&'static str, LedgerError> {
        match self {
            Self::Owned(operation) => operation.scope_name(scope),
            Self::Shared(operation) => operation.scope_name(scope),
        }
    }

    pub(crate) fn apply(&mut self, scope: Scope, change: HtmlChange) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.apply(scope, change),
            Self::Shared(operation) => operation.apply(scope, change),
        }
    }

    pub(crate) fn apply_batch(
        &mut self,
        scope: Scope,
        batch: &[HtmlChange],
    ) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.apply_batch(scope, batch),
            Self::Shared(operation) => operation.apply_batch(scope, batch),
        }
    }

    pub(crate) fn check_batch(
        &mut self,
        scope: Scope,
        batch: &[HtmlChange],
    ) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.check_batch(scope, batch),
            Self::Shared(operation) => operation.check_batch(scope, batch),
        }
    }

    /// Reads the accepted usage of a scope. An owned operation lends its usage; a shared one
    /// copies it under its lock.
    pub(crate) fn read<T>(
        &self,
        scope: Scope,
        read: impl FnOnce(&Usage<HtmlWork, HtmlResource>) -> T,
    ) -> Result<T, LedgerError> {
        match self {
            Self::Owned(operation) => operation.usage(scope).map(read),
            Self::Shared(operation) => operation.usage(scope).map(|usage| read(&usage)),
        }
    }

    /// The accepted usage of a scope, as a copy.
    pub(crate) fn usage(&self, scope: Scope) -> Result<Usage<HtmlWork, HtmlResource>, LedgerError> {
        match self {
            Self::Owned(operation) => operation.usage(scope).copied(),
            Self::Shared(operation) => operation.usage(scope),
        }
    }

    /// Records `value` as the largest reached by a gauge resource of `scope`. A shared operation
    /// does it under one lock.
    pub(crate) fn record_largest(
        &mut self,
        scope: Scope,
        resource: HtmlResource,
        value: u64,
    ) -> Result<(), LedgerError> {
        match self {
            Self::Owned(operation) => operation.charge_at(scope).record_largest(resource, value),
            Self::Shared(operation) => operation.charge_at(scope).record_largest(resource, value),
        }
    }
}

impl surgeist_ledger::ScopeLedger<HtmlWork, HtmlResource> for Ledger {
    fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: HtmlLimits,
        tariffs: HtmlTariffs,
    ) -> Result<Scope, LedgerError> {
        Ledger::child(self, parent, name, limits, tariffs)
    }

    fn check_batch(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        Ledger::check_batch(self, scope, changes)
    }

    fn apply_batch(&mut self, scope: Scope, changes: &[HtmlChange]) -> Result<(), LedgerError> {
        Ledger::apply_batch(self, scope, changes)
    }

    fn usage(&self, scope: Scope) -> Result<Usage<HtmlWork, HtmlResource>, LedgerError> {
        self.read(scope, |usage| *usage)
    }
}
