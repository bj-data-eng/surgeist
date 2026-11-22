//! Native Error objects and engine-private DOMException profile and slots.

use std::{
    cell::{OnceCell, RefCell},
    collections::BTreeMap,
    rc::Weak,
};

use boa_engine::{
    Context, Finalize, JsData, JsObject, JsResult, JsString, JsValue, Trace,
    builtins::error::Error as NativeError,
    object::builtins::JsWeakMap,
    property::{PropertyDescriptor, PropertyKey},
    realm::WeakRealm,
};

use crate::{
    Error, ExceptionPhase, Root,
    backend::{DomExceptionProfile, ObjectPrototype, Operations},
    conversion::ConversionAccount,
    dom_exception::Slot,
};

use super::{Entered, Handle, State};

#[derive(Default)]
pub(super) struct DomExceptions {
    // Boa's ephemeron keeps slots alive exactly while the branded object lives.
    // The owner-private map has neither a prototype nor a realm to retain.
    slots: OnceCell<JsWeakMap>,
    accounts: RefCell<BTreeMap<u64, (WeakRealm, Weak<ConversionAccount>)>>,
}

impl DomExceptions {
    fn slots(&self) -> Result<JsWeakMap, Error> {
        if let Some(slots) = self.slots.get() {
            return Ok(slots.clone());
        }
        let slots = JsWeakMap::with_null_proto();
        self.slots
            .set(slots.clone())
            .map_err(|_| Error::RegistryConflict)?;
        Ok(slots)
    }

    pub(super) fn reap(&self, state: &State) {
        if let Ok(mut accounts) = self.accounts.try_borrow_mut() {
            accounts.retain(|_, (realm, _)| realm.upgrade().is_some());
        } else {
            state.conflict();
        }
    }
}

#[derive(Trace, Finalize, JsData)]
struct Slots {
    name: JsString,
    message: JsString,
    code: u16,
}

impl Slots {
    fn read(&self, slot: Slot) -> JsValue {
        match slot {
            Slot::Name => self.name.clone().into(),
            Slot::Message => self.message.clone().into(),
            Slot::Code => u32::from(self.code).into(),
        }
    }
}

// The traced constructor keeps the existing callback registry's funded
// conversion account live. The Rust index owns only a weak account edge.
#[derive(Trace, Finalize, JsData)]
struct Profile {
    prototype: JsObject,
    _constructor: JsObject,
}

fn fresh_error(context: &mut Context) -> JsResult<JsObject> {
    // The intrinsic constructor has an immutable own prototype, so building
    // with no arguments creates Error data without consulting guest state.
    let constructor = context.intrinsics().constructors().error().constructor();
    constructor.construct(&[], None, context)
}

impl Entered<'_> {
    pub(super) fn boa_type_error(&mut self, message: &[u16]) -> Result<Root, Error> {
        self.validate_string_length(message.len())?;
        self.attempt_value(ExceptionPhase::Coerce, |context| {
            let constructor = context
                .intrinsics()
                .constructors()
                .type_error()
                .constructor();
            let message = JsString::from(message).into();
            constructor
                .construct(&[message], None, context)
                .map(Into::into)
        })
    }

    pub(super) fn boa_native_error(&mut self) -> Result<Root, Error> {
        self.attempt_value(ExceptionPhase::Invoke, |context| {
            fresh_error(context).map(Into::into)
        })
    }

    pub(super) fn boa_dom_exception_profile(
        &mut self,
    ) -> Result<Option<DomExceptionProfile>, Error> {
        self.check_active()?;
        let prepared = self.state.prepare(self.realm)?;
        let realm = self.state.lookup_realm(self.realm)?;
        let profile = realm.host_defined();
        let accounts = self
            .state
            .dom_exceptions
            .accounts
            .try_borrow()
            .map_err(|_| self.state.conflict())?;
        let (prototype, account) = match (profile.get::<Profile>(), accounts.get(&self.realm.index))
        {
            (None, None) => return Ok(None),
            (Some(profile), Some((_, account))) => {
                let account = account
                    .upgrade()
                    .ok_or_else(|| self.state.owner.fail(Error::BackendUnavailable))?;
                (profile.prototype.clone(), account)
            }
            _ => return Err(self.state.owner.fail(Error::BackendUnavailable)),
        };
        if !prototype.is::<NativeError>() {
            return Err(self.state.owner.fail(Error::BackendUnavailable));
        }
        Ok(Some(DomExceptionProfile {
            prototype: prepared.finish(Handle::Value(prototype.into()))?,
            account,
        }))
    }

    pub(super) fn boa_publish_dom_exception(
        &mut self,
        key: Option<&[u16]>,
        constructor: &Root,
        prototype: &Root,
        account: Weak<ConversionAccount>,
    ) -> Result<(), Error> {
        self.validate_root(constructor)?;
        self.validate_root(prototype)?;
        if let Some(key) = key {
            self.validate_string_length(key.len())?;
        }
        let state = self.state.clone();
        let realm_key = self.realm;
        let realm = state.lookup_realm(realm_key)?;
        let global = self.context.global_object();
        let prepared = key.map(|_| state.prepare(realm_key)).transpose()?;
        if realm.host_defined().get::<Profile>().is_some() {
            return Err(Error::PropertyRejected);
        }
        let mut accounts = state
            .dom_exceptions
            .accounts
            .try_borrow_mut()
            .map_err(|_| state.conflict())?;
        if account.strong_count() == 0 || accounts.contains_key(&realm_key.index) {
            return Err(state.owner.fail(Error::BackendUnavailable));
        }
        let constructor = self.object_value(constructor)?;
        let prototype = self.object_value(prototype)?;
        let publication = if let Some((key, prepared)) = key.zip(prepared) {
            let key = PropertyKey::from(JsString::from(key));
            match global.own_descriptor(&key, self.context) {
                Ok(None) => {}
                Ok(Some(_)) => return Err(Error::PropertyRejected),
                Err(error) => return self.capture(ExceptionPhase::DefineProperty, error, prepared),
            }
            let descriptor = PropertyDescriptor::builder()
                .value(constructor.clone())
                .writable(true)
                .enumerable(false)
                .configurable(true)
                .build();
            Some((key, descriptor, prepared))
        } else {
            None
        };
        state.owner.available()?;
        accounts.insert(realm_key.index, (realm.downgrade(), account));
        drop(accounts);
        realm.host_defined_mut().insert(Profile {
            prototype,
            _constructor: constructor,
        });
        let Some((key, descriptor, prepared)) = publication else {
            return Ok(());
        };
        let result = global.define_own_property(&key, descriptor, self.context);
        if !matches!(result, Ok(true)) {
            let retired = realm.host_defined_mut().remove::<Profile>();
            drop(retired);
            state
                .dom_exceptions
                .accounts
                .try_borrow_mut()
                .map_err(|_| state.conflict())?
                .remove(&realm_key.index);
        }
        state.owner.available()?;
        match result {
            Ok(true) => Ok(()),
            Ok(false) => Err(Error::PropertyRejected),
            Err(error) => self.capture(ExceptionPhase::DefineProperty, error, prepared),
        }
    }

    pub(super) fn boa_create_dom_exception(
        &mut self,
        prototype: &Root,
        name: &Root,
        message: &Root,
        code: u16,
    ) -> Result<Root, Error> {
        self.validate_root(prototype)?;
        let prepared = self.state.prepare(self.realm)?;
        let name = self
            .value(name)?
            .as_string()
            .ok_or(Error::BackendUnavailable)?;
        let message = self
            .value(message)?
            .as_string()
            .ok_or(Error::BackendUnavailable)?;
        let slots = self.state.dom_exceptions.slots()?;
        let result = (|| -> JsResult<JsObject> {
            let error = fresh_error(self.context)?;
            let record = JsObject::new(
                self.context.root_shape(),
                None,
                Slots {
                    name,
                    message,
                    code,
                },
            );
            slots.set(&error, record.upcast().into(), self.context)?;
            Ok(error)
        })();
        self.state.owner.available()?;
        let error = match result {
            Ok(error) => error,
            Err(error) => return self.capture(ExceptionPhase::Invoke, error, prepared),
        };
        let error = prepared.finish(Handle::Value(error.into()))?;
        self.set_prototype(&error, ObjectPrototype::Object(prototype))?;
        Ok(error)
    }

    pub(super) fn boa_dom_exception_slot(
        &mut self,
        receiver: &Root,
        slot: Slot,
    ) -> Result<Option<Root>, Error> {
        self.validate_root(receiver)?;
        let prepared = self.state.prepare(self.realm)?;
        let Some(receiver) = self.value(receiver)?.as_object() else {
            return Ok(None);
        };
        // Boa's native data test does not walk prototypes or unwrap proxies.
        if !receiver.is::<NativeError>() {
            return Ok(None);
        }
        let Some(slots) = self.state.dom_exceptions.slots.get().cloned() else {
            return Ok(None);
        };
        let result = slots.get(&receiver, self.context);
        self.state.owner.available()?;
        let record = match result {
            Ok(record) => record,
            Err(error) => return self.capture(ExceptionPhase::PropertyRead, error, prepared),
        };
        if record.is_undefined() {
            return Ok(None);
        }
        let record = record.as_object().ok_or(Error::BackendUnavailable)?;
        let slot = record
            .downcast_ref::<Slots>()
            .ok_or(Error::BackendUnavailable)?
            .read(slot);
        prepared.finish(Handle::Value(slot)).map(Some)
    }
}
