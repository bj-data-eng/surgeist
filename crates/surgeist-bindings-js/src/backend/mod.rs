pub(crate) mod v8;

use std::{any::Any, rc::Rc};

use crate::conversion::ConversionAccount;
use crate::{
    Callback, ConstructorCallback, Error, FunctionMetadata, Intrinsic, Primitive,
    PrototypeResolver, Realm, RealmKey, Root, ValueKind, WellKnownSymbol,
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

pub(crate) enum PropertyDefinition {
    AllowExisting,
    RequireAbsent,
}

pub(crate) enum ObjectPrototype<'a> {
    Default,
    Null,
    Object(&'a Root),
}

/// One completed ToString, retained while its exact UTF-16 extent is admitted
/// and copied. The immutable string cannot reenter guest coercion during copy.
pub(crate) struct CoercedString {
    value: Root,
    length: usize,
}

impl CoercedString {
    pub(crate) fn len(&self) -> usize {
        self.length
    }
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
    fn owner_state(&self) -> std::rc::Rc<crate::owner::OwnerState>;
    fn create_realm(&mut self) -> Result<Realm, Error>;
    fn with_realm(
        &mut self,
        realm: &Realm,
        action: &mut dyn FnMut(&mut dyn Operations) -> Result<(), Error>,
    ) -> Result<(), Error>;
    fn checkpoint(&mut self) -> Result<(), Error>;
    fn memory_pressure(&mut self);
    fn realm_storage_bytes(&self) -> usize;
    fn managed_object_storage_bytes(&self) -> usize;
    fn release_managed_object(&mut self, holder: Root) -> Result<(), Error>;
    fn reap(&mut self);
}

pub(crate) struct DomExceptionProfile {
    pub(crate) prototype: Root,
    pub(crate) account: std::rc::Rc<ConversionAccount>,
}

#[derive(Clone, Copy)]
pub(crate) struct Projection {
    pub(crate) realm: RealmKey,
    pub(crate) id: u32,
}

pub(crate) enum PrototypeStep {
    Complete,
    Cycle,
    Next(Root),
}

#[derive(Clone, Copy)]
pub(crate) enum IndexedTrap {
    Get,
    Has,
    Descriptor,
    Define,
    Set,
    Delete,
    Keys,
    PreventExtensions,
    SetPrototype,
}

impl IndexedTrap {
    pub(crate) const ALL: [Self; 9] = [
        Self::Get,
        Self::Has,
        Self::Descriptor,
        Self::Define,
        Self::Set,
        Self::Delete,
        Self::Keys,
        Self::PreventExtensions,
        Self::SetPrototype,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Has => "has",
            Self::Descriptor => "getOwnPropertyDescriptor",
            Self::Define => "defineProperty",
            Self::Set => "set",
            Self::Delete => "deleteProperty",
            Self::Keys => "ownKeys",
            Self::PreventExtensions => "preventExtensions",
            Self::SetPrototype => "setPrototypeOf",
        }
    }

    pub(crate) const fn arity(self) -> u32 {
        match self {
            Self::Get | Self::Define => 3,
            Self::Set => 4,
            Self::Has | Self::Descriptor | Self::Delete | Self::SetPrototype => 2,
            Self::Keys | Self::PreventExtensions => 1,
        }
    }
}

/// Engine-neutral primitives in one selected realm. Both host entries and
/// native callbacks borrow this same adapter; neither can reenter the owner.
pub(crate) trait Operations {
    fn realm_key(&self) -> RealmKey;
    fn owner_state(&self) -> std::rc::Rc<crate::owner::OwnerState>;
    fn refuse(&self, error: Error) -> Error;
    fn check_host_failure_report(&self) -> Result<(), Error>;
    fn check_active(&self) -> Result<(), Error>;
    fn validate_root(&self, root: &Root) -> Result<(), Error>;
    fn evaluate(&mut self, source: &[u16]) -> Result<Root, Error>;
    fn validate_string_length(&self, units: usize) -> Result<(), Error>;
    fn string(&mut self, units: &[u16]) -> Result<Root, Error>;
    fn primitive(&mut self, primitive: Primitive) -> Result<Root, Error>;
    fn kind(&mut self, value: &Root) -> Result<ValueKind, Error>;
    fn is_callable(&mut self, value: &Root) -> Result<bool, Error>;
    fn type_error(&mut self, message: &[u16]) -> Result<Root, Error>;
    fn native_error(&mut self) -> Result<Root, Error>;
    fn dom_exception_profile(&mut self) -> Result<Option<DomExceptionProfile>, Error>;
    fn publish_dom_exception(
        &mut self,
        key: &[u16],
        constructor: &Root,
        prototype: &Root,
        account: std::rc::Weak<ConversionAccount>,
    ) -> Result<(), Error>;
    fn create_dom_exception(
        &mut self,
        prototype: &Root,
        name: &Root,
        message: &Root,
        code: u16,
    ) -> Result<Root, Error>;
    fn dom_exception_slot(
        &mut self,
        receiver: &Root,
        slot: crate::dom_exception::Slot,
    ) -> Result<Option<Root>, Error>;
    fn function(
        &mut self,
        callback: &Callback,
        metadata: Option<FunctionMetadata<'_>>,
        capture: Option<u32>,
    ) -> Result<Root, Error>;
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
        definition: PropertyDefinition,
    ) -> Result<(), Error>;
    fn object(&mut self, prototype: ObjectPrototype<'_>) -> Result<Root, Error>;
    fn managed_object(&mut self) -> Result<Root, Error>;
    fn projection_storage_bytes(&self) -> usize;
    fn project(
        &mut self,
        object: &Root,
        state: Rc<dyn Any>,
        links: &[&Root],
    ) -> Result<Projection, Error>;
    fn projection(&mut self, object: &Root) -> Result<Option<Rc<dyn Any>>, Error>;
    fn projection_link(&mut self, object: &Root, index: usize) -> Result<Root, Error>;
    fn projection_is_live(&self, identity: Projection) -> Result<bool, Error>;
    fn reopen_projection(&mut self, identity: Projection) -> Result<Option<Root>, Error>;
    fn cssom_prototypes(&mut self, prototypes: &[&Root]) -> Result<(), Error>;
    fn cssom_prototype(&mut self, index: u32) -> Result<Root, Error>;
    fn indexed_proxy(&mut self, prototype: &Root, handlers: &[&Root; 9]) -> Result<Root, Error>;
    fn indexed_key(&mut self, key: &Root) -> Result<Option<u32>, Error>;
    fn indexed_fallback(&mut self, trap: IndexedTrap, arguments: &[&Root]) -> Result<Root, Error>;
    fn indexed_own_keys(&mut self, target: &Root) -> Result<(Root, u32), Error>;
    fn indexed_keys(&mut self, own_keys: &Root, length: u32) -> Result<Root, Error>;
    fn indexed_prototype_step(
        &mut self,
        target: &Root,
        candidate: &Root,
    ) -> Result<PrototypeStep, Error>;
    fn indexed_descriptor(&mut self, value: &Root) -> Result<Root, Error>;

    fn open_managed_object(&mut self, holder: &Root) -> Result<Root, Error>;
    fn set_prototype(&mut self, object: &Root, prototype: ObjectPrototype<'_>)
    -> Result<(), Error>;
    fn array(&mut self, values: &[Root]) -> Result<Root, Error>;
    fn well_known_symbol(&mut self, symbol: WellKnownSymbol) -> Result<Root, Error>;
    fn intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Root, Error>;
    fn call(&mut self, function: &Root, receiver: &Root, arguments: &[Root])
    -> Result<Root, Error>;
    fn coerce_string(&mut self, value: &Root) -> Result<CoercedString, Error>;
    /// Copies the entire string into an exactly sized, caller-owned buffer.
    fn copy_string(&mut self, string: &CoercedString, units: &mut [u16]) -> Result<(), Error>;
    fn to_utf16(&mut self, value: &Root) -> Result<Vec<u16>, Error> {
        let string = self.coerce_string(value)?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(string.len())
            .map_err(|_| Error::Allocation)?;
        units.resize(string.len(), 0);
        self.copy_string(&string, &mut units)?;
        Ok(units)
    }
    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error>;
    fn to_number(&mut self, value: &Root) -> Result<f64, Error>;
    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error>;
}
