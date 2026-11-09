use crate::{Change, Dimension, Error, Limits, Operation, Scope, SharedOperation, Tariffs, Usage};

/// An operation a component creates its own scopes in, and admits batches to.
///
/// A component that must observe the usage of scopes it created, check a batch before it
/// applies it, and open child scopes below a scope its caller supplied takes a `ScopeLedger`
/// instead of an operation. The stock implementations are an owned [`Operation`], a
/// [`SharedOperation`] and a mutable reference to either. Dispatch is static. Every method has
/// the meaning of the operation method of the same name, and an implementation outside this
/// crate forwards each method to such an operation.
pub trait ScopeLedger<W: Dimension, R: Dimension> {
    /// Opens a child scope; see [`Operation::child`].
    fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>>;

    /// Checks a batch without recording it; see [`Operation::check_batch`].
    fn check_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>>;

    /// Admits a batch; see [`Operation::apply_batch`].
    fn apply_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>>;

    /// A copy of the accepted usage of a scope; see [`Operation::usage`].
    fn usage(&self, scope: Scope) -> Result<Usage<W, R>, Error<W, R>>;
}

impl<W: Dimension, R: Dimension> ScopeLedger<W, R> for Operation<W, R> {
    fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        Operation::child(self, parent, name, limits, tariffs)
    }

    fn check_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        Operation::check_batch(self, scope, changes)
    }

    fn apply_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        Operation::apply_batch(self, scope, changes)
    }

    fn usage(&self, scope: Scope) -> Result<Usage<W, R>, Error<W, R>> {
        Operation::usage(self, scope).copied()
    }
}

impl<W: Dimension, R: Dimension> ScopeLedger<W, R> for SharedOperation<W, R> {
    fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        SharedOperation::child(self, parent, name, limits, tariffs)
    }

    fn check_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        SharedOperation::check_batch(self, scope, changes)
    }

    fn apply_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        SharedOperation::apply_batch(self, scope, changes)
    }

    fn usage(&self, scope: Scope) -> Result<Usage<W, R>, Error<W, R>> {
        SharedOperation::usage(self, scope)
    }
}

impl<W: Dimension, R: Dimension, L: ScopeLedger<W, R> + ?Sized> ScopeLedger<W, R> for &mut L {
    fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        (**self).child(parent, name, limits, tariffs)
    }

    fn check_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        (**self).check_batch(scope, changes)
    }

    fn apply_batch(&mut self, scope: Scope, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        (**self).apply_batch(scope, changes)
    }

    fn usage(&self, scope: Scope) -> Result<Usage<W, R>, Error<W, R>> {
        (**self).usage(scope)
    }
}
