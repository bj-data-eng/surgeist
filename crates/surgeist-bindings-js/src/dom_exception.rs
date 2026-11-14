//! Web IDL DOMException installation and native exception names.

use std::mem::size_of;
use std::rc::{Rc, Weak};

use surgeist_ledger::{Limits, Operation, SharedOperation, Tariffs};

use crate::conversion::{
    ArgumentValue, ConversionAccount, Resource, Work,
    declarations::{dom_exception_constructor_0_message, dom_exception_constructor_0_name},
};
use crate::{
    Arguments, Callback, Completion, ConstructorArguments, ConstructorCallback,
    ConstructorCompletion, Error, FunctionMetadata, Primitive, PropertyDescriptor, PropertyKey,
    Root, Scope, Value, WellKnownSymbol,
};

mod declarations;
pub use declarations::DomExceptionName;
use declarations::legacy_code;
use declarations::{
    ARITY, CAPTURED_ARGUMENTS, CODE, CONSTANTS, GET_CODE, GET_MESSAGE, GET_NAME, INTERFACE,
    MESSAGE, NAME,
};

/// Cumulative constructor argument decisions, cumulative copied UTF-16 units,
/// and live Rust conversion buffer bytes for one DOMException installation.
/// Copy work includes both directions and native exception construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomExceptionLimits {
    argument_decisions: u64,
    copied_units: u64,
    buffer_bytes: u64,
}

impl DomExceptionLimits {
    /// Sets the three independent conversion limits.
    pub const fn new(argument_decisions: u64, copied_units: u64, buffer_bytes: u64) -> Self {
        Self {
            argument_decisions,
            copied_units,
            buffer_bytes,
        }
    }
}

impl Default for DomExceptionLimits {
    fn default() -> Self {
        Self::new(2_000_000, 16_777_216, 262_144)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Slot {
    Name,
    Message,
    Code,
}

const PROTOTYPE: &[u16] = &[112, 114, 111, 116, 111, 116, 121, 112, 101];
const CONSTRUCTOR: &[u16] = &[99, 111, 110, 115, 116, 114, 117, 99, 116, 111, 114];

struct ConstructorState<Storage> {
    conversion: Rc<ConversionAccount>,
    _backing: Storage,
}

impl<Storage> ConstructorState<Storage> {
    fn construct<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error> {
        let message = dom_exception_constructor_0_message();
        let name = dom_exception_constructor_0_name();
        let (message, name) = self.conversion.run(scope, |conversion| {
            let message = conversion.argument(&message, arguments.get(message.index()))?;
            let name = conversion.argument(&name, arguments.get(name.index()))?;
            Ok((message, name))
        })?;
        let (ArgumentValue::Present(message), ArgumentValue::Present(name)) =
            (message.get(), name.get())
        else {
            return Err(Error::BackendUnavailable);
        };
        let prototype = arguments.select_prototype(scope)?;
        let name = name.as_code_units();
        create(
            scope,
            &self.conversion,
            &prototype.prototype().root,
            name,
            message.as_code_units(),
        )
        .map(ConstructorCompletion::Return)
    }
}

/// Requested Rust constructor behavior, target, conversion account and shared
/// Ledger bodies, including the profile's account reference and two reserved
/// scopes. The native adapter keeps this backing admitted while the realm
/// profile or constructor remains reachable. Returns `None` on size overflow.
pub fn constructor_storage_bytes<Storage>() -> Option<usize> {
    size_of::<ConstructorState<Storage>>()
        .checked_add(ConstructorCallback::behavior_storage_bytes())?
        .checked_add(size_of::<ConversionAccount>())?
        .checked_add(size_of::<Weak<ConversionAccount>>())?
        .checked_add(ConversionAccount::retained_body_bytes())?
        .checked_add(SharedOperation::<Work, Resource>::reserved_backing_bytes(
            2,
        )?)
}

/// Installs DOMException before a host exposes an authenticated realm to guest
/// execution. The realm and constructor keep their interface prototype alive
/// through engine-owned edges. The supplied backing funds the Rust constructor
/// state until its callback is retired.
///
/// # Errors
/// An existing own global DOMException property or an already installed realm
/// profile refuses installation with [`Error::PropertyRejected`]. Resource
/// refusals retain their original error. The backend prepares the private
/// profile before publishing the global property and discards that profile
/// when publication is refused.
pub fn install<Storage: 'static>(
    scope: &mut Scope<'_>,
    limits: DomExceptionLimits,
    backing: Storage,
) -> Result<(), Error> {
    if scope.operations.dom_exception_profile()?.is_some() {
        return Err(Error::PropertyRejected);
    }
    let limits = Limits::from_fn(
        |work| match work {
            Work::Argument | Work::Value => limits.argument_decisions,
            Work::CodeUnit => limits.copied_units,
            _ => 0,
        },
        |_| limits.buffer_bytes,
    );
    let ledger = Operation::with_reserved_scopes("dom-exception", limits, 2)
        .map_err(Error::ConversionLedger)?
        .share();
    let parent = ledger.root();
    let conversion = Rc::new(ConversionAccount::new(
        scope,
        ledger,
        parent,
        limits,
        Tariffs::identity(),
    )?);
    // The constructor owns the account and its debit together. Publication
    // only needs a weak reference, including on an early installation refusal.
    let account = Rc::downgrade(&conversion);
    let state = ConstructorState {
        conversion,
        _backing: backing,
    };
    let prototype_root = scope.operations.native_error()?;
    let prototype = scope.value(prototype_root);
    let callback = ConstructorCallback::bounded(
        CAPTURED_ARGUMENTS,
        interface_prototype,
        move |scope, arguments| state.construct(scope, arguments),
    );
    let constructor = scope.constructor(
        &callback,
        FunctionMetadata {
            name: INTERFACE,
            length: ARITY,
        },
    )?;
    scope.define_property(
        &constructor,
        PropertyKey::String(PROTOTYPE),
        PropertyDescriptor::Data {
            value: &prototype,
            writable: false,
            enumerable: false,
            configurable: false,
        },
    )?;
    for (key, getter, callback) in [
        (
            NAME,
            GET_NAME,
            Callback::new(|scope, arguments| read_slot(scope, arguments, Slot::Name)),
        ),
        (
            MESSAGE,
            GET_MESSAGE,
            Callback::new(|scope, arguments| read_slot(scope, arguments, Slot::Message)),
        ),
        (
            CODE,
            GET_CODE,
            Callback::new(|scope, arguments| read_slot(scope, arguments, Slot::Code)),
        ),
    ] {
        let getter = scope.fixed_arity_function(
            &callback,
            FunctionMetadata {
                name: getter,
                length: 0,
            },
        )?;
        scope.define_property(
            &prototype,
            PropertyKey::String(key),
            PropertyDescriptor::Accessor {
                get: Some(&getter),
                set: None,
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    let tag_key = scope.well_known_symbol(WellKnownSymbol::ToStringTag)?;
    let tag = scope.string(INTERFACE)?;
    scope.define_property(
        &prototype,
        PropertyKey::Value(&tag_key),
        PropertyDescriptor::Data {
            value: &tag,
            writable: false,
            enumerable: false,
            configurable: true,
        },
    )?;
    for &(name, code) in CONSTANTS {
        let value = scope.primitive(Primitive::Number(f64::from(code)))?;
        for object in [&constructor, &prototype] {
            scope.define_property(
                object,
                PropertyKey::String(name),
                PropertyDescriptor::Data {
                    value: &value,
                    writable: false,
                    enumerable: true,
                    configurable: false,
                },
            )?;
        }
    }
    scope.define_property(
        &prototype,
        PropertyKey::String(CONSTRUCTOR),
        PropertyDescriptor::Data {
            value: &constructor,
            writable: true,
            enumerable: false,
            configurable: true,
        },
    )?;
    scope
        .operations
        .publish_dom_exception(INTERFACE, &constructor.root, &prototype.root, account)
}

fn read_slot<'scope>(
    scope: &mut Scope<'scope>,
    arguments: Arguments<'scope>,
    slot: Slot,
) -> Result<Completion<'scope>, Error> {
    match scope
        .operations
        .dom_exception_slot(&arguments.receiver().root, slot)?
    {
        Some(value) => Ok(Completion::Return(scope.value(value))),
        None => Ok(Completion::Throw(scope.type_error(&[])?)),
    }
}

fn interface_prototype<'scope>(scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
    let profile = scope
        .operations
        .dom_exception_profile()?
        .ok_or(Error::DomExceptionUnavailable)?;
    Ok(scope.value(profile.prototype))
}

pub(super) fn create_native<'scope>(
    scope: &mut Scope<'scope>,
    name: DomExceptionName,
    message: &[u16],
) -> Result<Value<'scope>, Error> {
    let profile = scope
        .operations
        .dom_exception_profile()?
        .ok_or(Error::DomExceptionUnavailable)?;
    create(
        scope,
        &profile.account,
        &profile.prototype,
        name.code_units(),
        message,
    )
}

fn create<'scope>(
    scope: &mut Scope<'scope>,
    account: &ConversionAccount,
    prototype: &Root,
    name: &[u16],
    message: &[u16],
) -> Result<Value<'scope>, Error> {
    let code = legacy_code(name);
    let (name, message) = account.run(scope, |conversion| {
        let name = conversion.copy_to_js(name)?;
        let message = conversion.copy_to_js(message)?;
        Ok((name, message))
    })?;
    let root = scope
        .operations
        .create_dom_exception(prototype, &name.root, &message.root, code)?;
    Ok(scope.value(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_storage_includes_behavior_and_shared_ledger_with_two_reserved_scopes() {
        let ledger = SharedOperation::<Work, Resource>::reserved_backing_bytes(2).unwrap();
        let expected = size_of::<ConstructorState<[u8; 3]>>()
            + size_of::<ConversionAccount>()
            + size_of::<Weak<ConversionAccount>>()
            + ConversionAccount::retained_body_bytes()
            + size_of::<crate::ConstructorBehavior>()
            + ledger;
        assert_eq!(constructor_storage_bytes::<[u8; 3]>(), Some(expected));
    }
}
