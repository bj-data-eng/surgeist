//! Engine-neutral custody for a host-created promise and its settlement.

use crate::{Error, Root, Scope, Value};

/// Settlement authority for one host-created promise. It belongs to the
/// original heap and is consumed by settlement.
pub struct PromiseResolver {
    root: Root,
}

impl<'scope> Scope<'scope> {
    /// Create an intrinsic pending promise and retain its settlement authority.
    /// Producer admission follows the resolver root until actual reaping.
    pub fn promise_resolver(&mut self) -> Result<PromiseResolver, Error> {
        let root = self.operations.new_promise_resolver()?;
        Ok(PromiseResolver { root })
    }

    /// Open the exact promise belonging to this resolver.
    pub fn resolver_promise(&mut self, resolver: &PromiseResolver) -> Result<Value<'scope>, Error> {
        let root = self.operations.resolver_promise(&resolver.root)?;
        Ok(self.value(root))
    }

    /// Open an intrinsic promise with root backing from a current native read.
    /// This inspects the existing resolver and performs no guest operation.
    pub fn resolver_promise_funded<T: crate::preparation::Retention>(
        &mut self,
        resolver: &PromiseResolver,
        controller: &crate::preparation::Controller<T>,
        fund: impl FnOnce(crate::preparation::Request) -> Result<T, Error>,
    ) -> Result<Value<'scope>, Error> {
        self.operations.validate_root(&resolver.root)?;
        let backing = self.fund_root(controller, fund)?;
        let root = self
            .operations
            .resolver_promise_funded(&resolver.root, backing)?;
        Ok(self.value(root))
    }

    /// Resolve the promise with ordinary JavaScript thenable assimilation.
    /// A then getter can execute guest code before this call returns.
    pub fn resolve_promise(
        &mut self,
        resolver: PromiseResolver,
        value: &Value<'scope>,
    ) -> Result<(), Error> {
        self.operations
            .settle_promise(&resolver.root, &value.root, false)
    }

    /// Reject the promise with the original value. No checkpoint is run.
    pub fn reject_promise(
        &mut self,
        resolver: PromiseResolver,
        reason: &Value<'scope>,
    ) -> Result<(), Error> {
        self.operations
            .settle_promise(&resolver.root, &reason.root, true)
    }
}
