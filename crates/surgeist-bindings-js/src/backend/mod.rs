pub(crate) mod v8;

use crate::{
    Callback, ConstructorCallback, Error, FunctionMetadata, Intrinsic, Primitive,
    PrototypeResolver, Realm, Root, ValueKind, WellKnownSymbol,
};

#[derive(Clone, Copy)]
pub(crate) enum PropertyKey<'a> {
    String(&'a [u16]),
    Value(&'a Root),
}

pub(crate) enum PropertyDescriptor<'a> {
    Data {
        value: &'a Root,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: Option<&'a Root>,
        set: Option<&'a Root>,
        enumerable: bool,
        configurable: bool,
    },
}

pub(crate) enum ObjectPrototype<'a> {
    Default,
    Null,
    Object(&'a Root),
}

// Embedding policy for the pinned backend, not the ECMAScript array limit.
// fixed-array-base.h permits 16 Mi elements in its lower-limits SDK build and
// 128 Mi otherwise. The minimum is safe for every supported pinned build mode.
const MAX_ARRAY_ELEMENTS: usize = 16 * 1024 * 1024;

pub(crate) fn validate_array_length(length: usize) -> Result<(), Error> {
    if length > MAX_ARRAY_ELEMENTS {
        Err(Error::ArrayTooLong)
    } else {
        Ok(())
    }
}

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
    fn constructor(
        &mut self,
        callback: &ConstructorCallback,
        metadata: FunctionMetadata<'_>,
    ) -> Result<Root, Error>;
    fn constructor_prototype(
        &mut self,
        new_target: &Root,
        fallback: &PrototypeResolver,
    ) -> Result<Root, Error>;
    fn global(&mut self) -> Result<Root, Error>;
    fn get(&mut self, object: &Root, key: PropertyKey<'_>) -> Result<Root, Error>;
    fn set(&mut self, object: &Root, key: PropertyKey<'_>, value: &Root) -> Result<(), Error>;
    fn define_property(
        &mut self,
        object: &Root,
        key: PropertyKey<'_>,
        descriptor: PropertyDescriptor<'_>,
    ) -> Result<(), Error>;
    fn object(&mut self, prototype: ObjectPrototype<'_>) -> Result<Root, Error>;
    fn set_prototype(&mut self, object: &Root, prototype: ObjectPrototype<'_>)
    -> Result<(), Error>;
    fn array(&mut self, values: &[Root]) -> Result<Root, Error>;
    fn well_known_symbol(&mut self, symbol: WellKnownSymbol) -> Result<Root, Error>;
    fn intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Root, Error>;
    fn call(&mut self, function: &Root, receiver: &Root, arguments: &[Root])
    -> Result<Root, Error>;
    fn to_utf16(&mut self, value: &Root) -> Result<Vec<u16>, Error>;
    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error>;
    fn to_number(&mut self, value: &Root) -> Result<f64, Error>;
    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error>;
}
