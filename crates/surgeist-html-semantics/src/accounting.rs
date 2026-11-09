//! Where capture, import and semantic queries charge their work and resources.

use std::fmt;
use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlTariffs, HtmlWork};
use surgeist_ledger::{Charge, Error, Operation, Scope, SharedOperation, Usage};

/// The ledger error type of the HTML schema pair.
pub(crate) type LedgerError = Error<HtmlWork, HtmlResource>;

/// Widens a platform amount; an amount beyond `u64` saturates and every finite cap refuses it.
pub(crate) fn wide(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Where a capture builder or a session charges: a scope of an operation the host
/// owns.
///
/// The crate charges every work step and resource through a `surgeist-ledger`
/// operation of the HTML schema pair, `HtmlWork` and `HtmlResource`. Without
/// accounting supplied, each capture builder and each session charges an
/// operation of its own that nothing else observes. A host that attributes HTML
/// work to its own operation supplies this value, which names a
/// shared operation and a scope of it.
///
/// # Scope tree
///
/// Under the supplied scope a capture builder creates `html-capture`, which
/// carries the caps of `HtmlCaptureLimits`; a construction import creates
/// `html-capture-import` below the same scope; a session creates `html-query`,
/// which carries the caps of `HtmlSemanticLimits`, and `html-query-observed`.
/// The observed scope has no caps and projects nothing into the supplied scope.
/// It records every reservation the allocator actually made before `html-query`
/// admits or refuses it, so a refusal still reports the capacity that was taken
/// and the supplied scope sees only admitted amounts.
///
/// ```text
/// supplied scope                  (the host's)
/// ├── html-capture                caps of the capture limits; aggregate tariffs
/// ├── html-capture-import         caps of the import limits; aggregate tariffs
/// ├── html-query                  caps of the session limits; aggregate tariffs
/// └── html-query-observed         unlimited; scale zero into the supplied scope
/// ```
///
/// A host operation never frees a scope: each builder, import and session adds
/// its scopes, so an operation that runs several sizes its scope bound for all of
/// them.
///
/// A refusal by a cap of the supplied scope or of an ancestor stops the operation
/// with the typed stop that carries that scope's limit and value. The tariffs
/// between the supplied scope and the scopes above it must map each dimension to
/// itself, because the refusal names the dimension of the charged change.
///
/// A session sizes a numeric materialization by the allocated bytes its own
/// limits leave. A lower cap of the supplied scope or of an ancestor refuses the
/// capacity when the session records it, after the allocation.
#[derive(Clone)]
pub struct HtmlSemanticAccounting {
    pub(crate) ledger: SharedOperation<HtmlWork, HtmlResource>,
    pub(crate) scope: Scope,
}

impl HtmlSemanticAccounting {
    /// Charges below `scope` of `ledger`, an operation of the HTML schema pair that
    /// the host owns.
    pub fn host(ledger: SharedOperation<HtmlWork, HtmlResource>, scope: Scope) -> Self {
        Self { ledger, scope }
    }
}

impl fmt::Debug for HtmlSemanticAccounting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlSemanticAccounting")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
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
