pub(crate) mod v8;

use crate::{Callback, Error, Primitive, Realm, Root, ValueKind};

/// Heap lifecycle, separate from operations on an already entered realm.
pub(crate) trait Engine {
    fn create_realm(&mut self) -> Result<Realm, Error>;
    fn with_realm(
        &mut self,
        realm: &Realm,
        action: &mut dyn FnMut(&mut dyn Operations),
    ) -> Result<(), Error>;
    fn checkpoint(&mut self) -> Result<(), Error>;
    fn memory_pressure(&mut self);
    fn reap(&mut self);
}

/// Engine-neutral primitives in one selected realm. Both host entries and
/// native callbacks borrow this same adapter; neither can reenter the owner.
pub(crate) trait Operations {
    fn check_active(&self) -> Result<(), Error>;
    fn validate_root(&self, root: &Root) -> Result<(), Error>;
    fn evaluate(&mut self, source: &[u16]) -> Result<Root, Error>;
    fn string(&mut self, units: &[u16]) -> Result<Root, Error>;
    fn primitive(&mut self, primitive: Primitive) -> Result<Root, Error>;
    fn kind(&mut self, value: &Root) -> Result<ValueKind, Error>;
    fn is_callable(&mut self, value: &Root) -> Result<bool, Error>;
    fn type_error(&mut self, message: &[u16]) -> Result<Root, Error>;
    fn function(&mut self, callback: &Callback) -> Result<Root, Error>;
    fn global(&mut self) -> Result<Root, Error>;
    fn get(&mut self, object: &Root, name: &[u16]) -> Result<Root, Error>;
    fn set(&mut self, object: &Root, name: &[u16], value: &Root) -> Result<(), Error>;
    fn call(&mut self, function: &Root, receiver: &Root, arguments: &[Root])
    -> Result<Root, Error>;
    fn to_utf16(&mut self, value: &Root) -> Result<Vec<u16>, Error>;
    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error>;
    fn to_number(&mut self, value: &Root) -> Result<f64, Error>;
    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error>;
}
