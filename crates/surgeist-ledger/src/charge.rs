use crate::{Change, Dimension, Error, Operation, Scope, SharedOperation};

/// Where a bounded operation charges the work it is about to do.
///
/// A component that owns a bounded operation takes a `Charge` instead of an
/// operation and a scope. The caller decides what charging means: the stock
/// implementations apply the changes to a scope of an [`Operation`] or a
/// [`SharedOperation`], and a consumer with its own admission rules, such as a
/// latched first stop, implements the trait over them and maps the refusal into
/// its own stop type.
///
/// Dispatch is static and every call carries a whole batch, so an operation
/// charges once per batch of changes rather than once per change.
pub trait Charge<W: Dimension, R: Dimension> {
    /// The typed stop a refused or failed charge reports.
    type Stop;

    /// Applies one ordered batch. The batch is all or nothing: a refusal leaves
    /// the usage the charge reads as it was before the call.
    fn charge(&mut self, batch: &[Change<W, R>]) -> Result<(), Self::Stop>;

    /// Records `value` as the largest reached by a resource that is a gauge
    /// rather than live custody. One batch releases the value held and acquires
    /// `value`, so the cap is checked against the new value and the peak keeps
    /// the largest. A refusal leaves the held value as it was.
    fn record_largest(&mut self, resource: R, value: u64) -> Result<(), Self::Stop>;
}

/// An owned [`Operation`] and the scope that receives its charges.
///
/// Created by [`Operation::charge_at`]. `Stop` is the ledger [`Error`].
pub struct ScopeCharge<'a, W: Dimension, R: Dimension> {
    operation: &'a mut Operation<W, R>,
    scope: Scope,
}

impl<W: Dimension, R: Dimension> Operation<W, R> {
    /// Charges to `scope` of this operation through the [`Charge`] trait.
    pub fn charge_at(&mut self, scope: Scope) -> ScopeCharge<'_, W, R> {
        ScopeCharge {
            operation: self,
            scope,
        }
    }
}

impl<W: Dimension, R: Dimension> Charge<W, R> for ScopeCharge<'_, W, R> {
    type Stop = Error<W, R>;

    fn charge(&mut self, batch: &[Change<W, R>]) -> Result<(), Self::Stop> {
        self.operation.apply_batch(self.scope, batch)
    }

    fn record_largest(&mut self, resource: R, value: u64) -> Result<(), Self::Stop> {
        self.operation.record_largest(self.scope, resource, value)
    }
}

/// A [`SharedOperation`] and the scope that receives its charges.
///
/// Created by [`SharedOperation::charge_at`]. `Stop` is the ledger [`Error`].
pub struct SharedScopeCharge<'a, W: Dimension, R: Dimension> {
    operation: &'a SharedOperation<W, R>,
    scope: Scope,
}

impl<W: Dimension, R: Dimension> SharedOperation<W, R> {
    /// Charges to `scope` of this operation through the [`Charge`] trait.
    pub fn charge_at(&self, scope: Scope) -> SharedScopeCharge<'_, W, R> {
        SharedScopeCharge {
            operation: self,
            scope,
        }
    }
}

impl<W: Dimension, R: Dimension> Charge<W, R> for SharedScopeCharge<'_, W, R> {
    type Stop = Error<W, R>;

    fn charge(&mut self, batch: &[Change<W, R>]) -> Result<(), Self::Stop> {
        self.operation.apply_batch(self.scope, batch)
    }

    fn record_largest(&mut self, resource: R, value: u64) -> Result<(), Self::Stop> {
        self.operation.record_largest(self.scope, resource, value)
    }
}
