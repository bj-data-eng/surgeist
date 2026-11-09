//! Where capture, import and semantic queries charge their work and resources.
//!
//! The crate charges every work step and resource through a `surgeist-ledger`
//! operation of the HTML schema pair, `HtmlWork` and `HtmlResource`. Without
//! accounting supplied, each capture builder and each session charges an
//! operation of its own that nothing else observes. A host that attributes HTML
//! work to its own operation supplies a [`HtmlSemanticAccounting`], which names a
//! shared operation and a scope of it.
//!
//! # Scope tree
//!
//! Under the supplied scope a capture builder creates `html-capture`, which
//! carries the caps of `HtmlCaptureLimits`; a construction import creates
//! `html-capture-import` below the same scope; a session creates `html-query`,
//! which carries the caps of `HtmlSemanticLimits`, and `html-query-observed`.
//! The observed scope has no caps and projects nothing into the supplied scope.
//! It records every reservation the allocator actually made before `html-query`
//! admits or refuses it, so a refusal still reports the capacity that was taken
//! and the supplied scope sees only admitted amounts.
//!
//! ```text
//! supplied scope                  (the host's)
//! ├── html-capture                caps of the capture limits; aggregate tariffs
//! ├── html-capture-import         caps of the import limits; aggregate tariffs
//! ├── html-query                  caps of the session limits; aggregate tariffs
//! └── html-query-observed         unlimited; scale zero into the supplied scope
//! ```
//!
//! A host operation never frees a scope: each builder, import and session adds
//! its scopes, so an operation that runs several sizes its scope bound for all of
//! them.
//!
//! A refusal by a cap of the supplied scope or of an ancestor stops the operation
//! with the typed stop that carries that scope's limit and value. The tariffs
//! between the supplied scope and the scopes above it must map each dimension to
//! itself, because the refusal names the dimension of the charged change.

use std::{error::Error as StdError, fmt};
use surgeist_html::accounting::{HtmlChange, HtmlLimits, HtmlResource, HtmlTariffs, HtmlWork};
use surgeist_ledger::{Charge, Defect, Error, Operation, Scope, SharedOperation, Usage};

/// The ledger error type of the HTML schema pair.
pub(crate) type LedgerError = Error<HtmlWork, HtmlResource>;

/// A ledger failure that is not a refusal of a charged change.
///
/// A refusal at a cap is reported as the typed limit stop of the operation that
/// charged. Every other ledger error stops that operation with one of these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlSemanticAccountingFault {
    /// The supplied scope belongs to another operation.
    ForeignScope,
    /// The operation's scope bound leaves no room for the scopes of the operation.
    ScopeLimit {
        /// The operation's scope bound.
        limit: usize,
    },
    /// The ledger could not allocate its storage.
    Allocation,
    /// The ledger's identities are exhausted.
    IdentityExhausted,
    /// Timed profiling needs a clock the platform does not supply.
    TimingUnavailable,
    /// A checkpoint operation failed. Nothing here opens a checkpoint, so only a
    /// ledger that reports one for another reason produces it.
    Checkpoint,
    /// The ledger refused a change where nothing here charges one, such as the
    /// creation of a scope, or a refusal that names a dimension the operation does
    /// not report.
    UnexpectedRefusal,
    /// The ledger found a broken invariant of its own. It stays latched in the
    /// operation.
    Defect(Defect),
}

impl fmt::Display for HtmlSemanticAccountingFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignScope => f.write_str("the supplied scope belongs to another operation"),
            Self::ScopeLimit { limit } => write!(
                f,
                "the operation's scope bound of {limit} leaves no room for the scopes"
            ),
            Self::Allocation => f.write_str("the ledger could not allocate its storage"),
            Self::IdentityExhausted => f.write_str("the ledger's identities are exhausted"),
            Self::TimingUnavailable => f.write_str("timed profiling needs a host clock"),
            Self::Checkpoint => f.write_str("a ledger checkpoint operation failed"),
            Self::UnexpectedRefusal => {
                f.write_str("the ledger refused a change where nothing charges one")
            }
            Self::Defect(defect) => write!(f, "the ledger found a defect: {defect}"),
        }
    }
}

impl StdError for HtmlSemanticAccountingFault {}

/// The fault of a ledger error that is not a refusal of a charged change.
pub(crate) fn fault(error: LedgerError) -> HtmlSemanticAccountingFault {
    match error {
        Error::Refused(_) => HtmlSemanticAccountingFault::UnexpectedRefusal,
        Error::ForeignScope => HtmlSemanticAccountingFault::ForeignScope,
        Error::ScopeLimit { limit } => HtmlSemanticAccountingFault::ScopeLimit { limit },
        Error::Allocation => HtmlSemanticAccountingFault::Allocation,
        Error::IdentityExhausted => HtmlSemanticAccountingFault::IdentityExhausted,
        Error::TimingUnavailable => HtmlSemanticAccountingFault::TimingUnavailable,
        Error::ForeignCheckpoint
        | Error::StaleCheckpoint
        | Error::CheckpointLimit { .. }
        | Error::CheckpointsConfigured => HtmlSemanticAccountingFault::Checkpoint,
        Error::Defect(defect) => HtmlSemanticAccountingFault::Defect(defect),
    }
}

/// Where a capture builder or a session charges: a scope of an operation the host
/// owns.
///
/// Without one, each builder and session charges an operation of its own. The
/// scope tree and the refusals a host scope reports are described in the
/// [module documentation](self).
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
