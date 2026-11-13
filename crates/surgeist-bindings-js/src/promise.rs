//! Engine-neutral custody for a host-created promise and its settlement.

use std::any::Any;

use crate::{Error, Root, Scope, Value};

/// Settlement authority for one host-created promise. It belongs to the
/// original heap and is consumed by settlement.
pub struct PromiseResolver {
    root: Root,
}

impl<'scope> Scope<'scope> {
    /// Create an intrinsic pending promise and retain its settlement authority.
    /// The admitted backing follows the resolver's managed root.
    pub fn promise_resolver(&mut self, backing: Box<dyn Any>) -> Result<PromiseResolver, Error> {
        let root = self.operations.new_promise_resolver()?;
        self.operations.retain_root_backing(&root, backing)?;
        Ok(PromiseResolver { root })
    }

    /// Open the exact promise belonging to this resolver.
    pub fn resolver_promise(&mut self, resolver: &PromiseResolver) -> Result<Value<'scope>, Error> {
        let root = self.operations.resolver_promise(&resolver.root)?;
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
