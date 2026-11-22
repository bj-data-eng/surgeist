use super::*;
use crate::backend::{
    CoercedString, ObjectPrototype, PropertyDefinition, PropertyDescriptor, PropertyKey,
};
use crate::preparation::{self, Admitted, BufferKind, Work};
use crate::{Intrinsic, Primitive, ValueKind, WellKnownSymbol};
use boa_engine::{
    JsObject, JsString, JsSymbol, JsValue, Source,
    job::{Job, PromiseJob},
    object::builtins::JsArray,
    property::{PropertyDescriptor as BoaPropertyDescriptor, PropertyKey as BoaPropertyKey},
    script::Script,
    value::JsVariant,
};

impl Entered<'_> {
    fn property_key(&self, key: PropertyKey<'_>) -> Result<BoaPropertyKey, Error> {
        match key {
            PropertyKey::String(units) => {
                self.validate_string_length(units.len())?;
                Ok(JsString::from(units).into())
            }
            PropertyKey::Value(value) => match self.value(value)?.variant() {
                JsVariant::String(string) => Ok(string.into()),
                JsVariant::Symbol(symbol) => Ok(BoaPropertyKey::Symbol(symbol)),
                _ => Err(Error::ExpectedPropertyKey),
            },
        }
    }

    fn accessor_value(&self, value: Option<&Root>) -> Result<JsValue, Error> {
        let value = match value {
            Some(value) => self.value(value)?,
            None => JsValue::undefined(),
        };
        if value.is_undefined() || value.is_callable() {
            Ok(value)
        } else {
            Err(Error::ExpectedAccessor)
        }
    }

    fn object_prototype(&self, prototype: ObjectPrototype<'_>) -> Result<Option<JsObject>, Error> {
        Ok(match prototype {
            ObjectPrototype::Default => Some(
                self.context
                    .intrinsics()
                    .constructors()
                    .object()
                    .prototype(),
            ),
            ObjectPrototype::Null => None,
            ObjectPrototype::Object(prototype) => Some(self.object_value(prototype)?),
        })
    }

    fn outbound_values(&self, values: &[Root]) -> Result<Admitted<Vec<JsValue>>, Error> {
        let owner = &self.state.owner;
        let kind = BufferKind::OutboundLocals;
        let mut result = preparation::buffer::<JsValue>(owner, kind, values.len())?;
        for value in values {
            preparation::charge(owner, Work::BufferElement(kind))?;
            result.value.push(self.value(value)?);
        }
        Ok(result)
    }
}

fn primitive_value(primitive: Primitive) -> JsValue {
    match primitive {
        Primitive::Undefined => JsValue::undefined(),
        Primitive::Null => JsValue::null(),
        Primitive::Boolean(value) => JsValue::new(value),
        Primitive::Number(value) => JsValue::new(value),
    }
}

fn matches_data(
    owner: &JsObject,
    name: &'static str,
    value: JsValue,
    mutable: bool,
    context: &mut Context,
) -> JsResult<bool> {
    let descriptor = owner.own_descriptor(&JsString::from(name).into(), context)?;
    Ok(descriptor.is_some_and(|descriptor| {
        descriptor
            .value()
            .is_some_and(|actual| JsValue::strict_equals(actual, &value))
            && descriptor.writable() == Some(mutable)
            && descriptor.configurable() == Some(mutable)
            && descriptor.enumerable() == Some(false)
    }))
}

impl Operations for Entered<'_> {
    fn realm_key(&self) -> RealmKey {
        self.realm
    }

    fn owner_state(&self) -> Rc<OwnerState> {
        self.state.owner.clone()
    }

    fn refuse(&self, error: Error) -> Error {
        self.state.owner.fail(error)
    }

    fn check_host_failure_report(&self) -> Result<(), Error> {
        self.state.owner.check_host_failure_report()
    }

    fn check_active(&self) -> Result<(), Error> {
        self.state.owner.available()
    }

    fn validate_root(&self, root: &Root) -> Result<(), Error> {
        self.check_active()?;
        self.state.entry(root).map(|_| ())
    }

    fn window_interface(&mut self) -> Result<WindowInterface, Error> {
        let window = self
            .state
            .realms
            .try_borrow()
            .map_err(|_| self.state.conflict())?
            .get(&self.realm.index)
            .and_then(|record| record.window.clone())
            .ok_or(Error::BackendUnavailable)?;
        Ok(WindowInterface {
            prototype: self.root(window.prototype.into())?,
            event_target_prototype: self.root(window.event_target.into())?,
        })
    }

    fn install_window_name_reader(&mut self, reader: &Root) -> Result<(), Error> {
        self.check_active()?;
        let reader = self.object_value(reader)?;
        let names = self
            .state
            .realms
            .try_borrow()
            .map_err(|_| self.state.conflict())?
            .get(&self.realm.index)
            .and_then(|record| record.window.as_ref())
            .map(|window| window.names.clone())
            .ok_or(Error::BackendUnavailable)?;
        if boa_engine::object::host::install_window_name_reader(&names, reader) {
            Ok(())
        } else {
            Err(Error::PropertyRejected)
        }
    }

    fn install_window_index_reader(&mut self, reader: &Root) -> Result<(), Error> {
        self.check_active()?;
        if !self
            .state
            .realms
            .try_borrow()
            .map_err(|_| self.state.conflict())?
            .get(&self.realm.index)
            .is_some_and(|record| record.window.is_some())
        {
            return Err(Error::BackendUnavailable);
        }
        let reader = self.object_value(reader)?;
        let proxy = self.context.global_this();
        if boa_engine::object::host::install_window_index_reader(&proxy, reader) {
            Ok(())
        } else {
            Err(Error::PropertyRejected)
        }
    }

    fn prepare_same_origin_window(&mut self, payload: Weak<dyn Any>) -> Result<Realm, Error> {
        self.check_active()?;
        let origin = boa_engine::object::host::window_proxy_origin(&self.context.global_this())
            .ok_or(Error::BackendUnavailable)?;
        let backing = self
            .state
            .owner
            .preparation
            .get()
            .map(|account| {
                let bytes = State::realm_storage_bytes()
                    .checked_add(size_of::<Guard>())
                    .ok_or(Error::Allocation)?;
                let guard = account.admit(Request::Realm { bytes })?;
                Ok::<_, Error>(RealmBacking::new(guard))
            })
            .transpose()?;
        self.state
            .create_realm(self.context, Some(origin), backing, Some(payload))
    }

    fn child_window_payload(&self, value: &Root) -> Result<Option<Rc<dyn Any>>, Error> {
        self.check_active()?;
        let value = self.value(value)?;
        let Some(proxy) = value.as_object() else {
            return Ok(None);
        };
        if boa_engine::object::host::window_proxy_origin(&proxy).is_none() {
            return Ok(None);
        }
        let realm = proxy.creation_realm().ok_or(Error::RegistryConflict)?;
        let realms = self
            .state
            .realms
            .try_borrow()
            .map_err(|_| self.state.conflict())?;
        if &realm == self.context.realm() {
            return Ok(realms
                .get(&self.realm.index)
                .and_then(|record| record.child_payload.as_ref())
                .and_then(Weak::upgrade));
        }
        for record in realms.values() {
            preparation::charge(&self.state.owner, Work::RealmRecord)?;
            if record.realm.upgrade().as_ref() == Some(&realm) {
                return Ok(record.child_payload.as_ref().and_then(Weak::upgrade));
            }
        }
        Err(Error::RegistryConflict)
    }

    fn install_child_window(
        &mut self,
        realm: &Realm,
        install: &mut dyn FnMut(&mut dyn Operations) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.check_active()?;
        let key = realm.0.key;
        let realm = self.state.lookup_realm(key)?;
        let frame = EnterRealm::new(self.context, realm);
        install(&mut Entered {
            context: &mut *frame.context,
            state: self.state.clone(),
            realm: key,
        })?;
        let window = self
            .state
            .realms
            .try_borrow_mut()
            .map_err(|_| self.state.conflict())?
            .get_mut(&key.index)
            .ok_or_else(|| self.state.conflict())?
            .window
            .take();
        drop(window);
        Ok(())
    }

    fn qualify_window_event_target(
        &mut self,
        constructor: &Root,
        prototype: &Root,
    ) -> Result<(), Error> {
        self.check_active()?;
        let parent = self.object_value(constructor)?;
        let prototype = self.object_value(prototype)?;
        let reserved = self
            .state
            .realms
            .try_borrow()
            .map_err(|_| self.state.conflict())?
            .get(&self.realm.index)
            .and_then(|record| {
                record
                    .window
                    .as_ref()
                    .map(|window| window.event_target.clone())
            })
            .ok_or(Error::BackendUnavailable)?;
        if prototype != reserved || !self.state.callables.is_constructor(&parent, self.realm)? {
            return Err(Error::PropertyRejected);
        }
        let global = self.context.global_object();
        let valid = self.attempt(ExceptionPhase::PropertyRead, |context| {
            Ok(
                matches_data(&global, "EventTarget", parent.clone().into(), true, context)?
                    && matches_data(
                        &parent,
                        "prototype",
                        prototype.clone().into(),
                        false,
                        context,
                    )?
                    && matches_data(&prototype, "constructor", parent.into(), true, context)?,
            )
        })?;
        if !valid {
            return Err(Error::PropertyRejected);
        }
        let Some(record) = self
            .state
            .realms
            .try_borrow_mut()
            .map_err(|_| self.state.conflict())?
            .get_mut(&self.realm.index)
            .map(|record| record.window.take())
        else {
            return Err(self.state.conflict());
        };
        drop(record);
        Ok(())
    }

    fn managed_object(&mut self) -> Result<Root, Error> {
        self.new_handle(|context| {
            let object = JsObject::with_object_proto(context.intrinsics());
            Ok(Handle::Managed(object, context.realm().clone()))
        })
    }

    fn open_managed_object(&mut self, holder: &Root) -> Result<Root, Error> {
        self.validate_root(holder)?;
        let entry = self.state.entry(holder)?;
        let Some(Handle::Managed(object, realm)) = entry.handle.get() else {
            return Err(self.state.conflict());
        };
        if &self.state.lookup_realm(holder.0.realm)? != realm {
            return Err(self.state.conflict());
        }
        self.root(object.clone().into())
    }

    fn evaluate(&mut self, source: &[u16]) -> Result<Root, Error> {
        self.validate_string_length(source.len())?;
        let script = self.attempt(ExceptionPhase::Compile, |context| {
            Script::parse(Source::from_utf16(source), None, context)
        })?;
        self.attempt_value(ExceptionPhase::Evaluate, |context| script.evaluate(context))
    }

    fn enqueue_microtask(&mut self, function: &Root) -> Result<(), Error> {
        self.check_active()?;
        let function = self
            .value(function)?
            .as_callable()
            .ok_or(Error::ExpectedFunction)?;
        let realm = self.context.realm().clone();
        // Raw microtasks share the promise FIFO. Ordinary guest throws
        // complete this callback; engine failures stop the checkpoint.
        self.context
            .enqueue_job(Job::PromiseJob(PromiseJob::with_realm(
                move |context| {
                    function
                        .call(&JsValue::undefined(), &[], context)
                        .or_else(|error| {
                            if error.as_engine().is_some() {
                                Err(error)
                            } else {
                                Ok(JsValue::undefined())
                            }
                        })
                },
                realm,
            )));
        Ok(())
    }

    fn validate_string_length(&self, units: usize) -> Result<(), Error> {
        // Match Boa's String builtin bound. Leave space for the heap allocation
        // header even on targets whose allocator layout uses a smaller limit.
        let limit = (u32::MAX as usize).min((isize::MAX as usize / 2) - 64);
        if units > limit {
            return Err(Error::StringTooLong);
        }
        Ok(())
    }

    fn root_storage_bytes(&self) -> usize {
        State::root_storage_bytes()
    }

    fn string_funded(
        &mut self,
        units: &[u16],
        backing: crate::preparation::Guard,
    ) -> Result<Root, Error> {
        self.check_active()?;
        self.validate_string_length(units.len())?;
        self.funded(JsString::from(units).into(), backing)
    }

    fn primitive_funded(
        &mut self,
        primitive: Primitive,
        backing: crate::preparation::Guard,
    ) -> Result<Root, Error> {
        self.check_active()?;
        self.funded(primitive_value(primitive), backing)
    }

    fn string(&mut self, units: &[u16]) -> Result<Root, Error> {
        self.validate_string_length(units.len())?;
        self.attempt_value(ExceptionPhase::Coerce, |_| Ok(JsString::from(units).into()))
    }

    fn primitive(&mut self, primitive: Primitive) -> Result<Root, Error> {
        self.root(primitive_value(primitive))
    }

    fn kind(&mut self, value: &Root) -> Result<ValueKind, Error> {
        Ok(match self.value(value)?.variant() {
            JsVariant::Undefined => ValueKind::Undefined,
            JsVariant::Null => ValueKind::Null,
            JsVariant::Boolean(_) => ValueKind::Boolean,
            JsVariant::Integer32(_) | JsVariant::Float64(_) => ValueKind::Number,
            JsVariant::BigInt(_) => ValueKind::BigInt,
            JsVariant::String(_) => ValueKind::String,
            JsVariant::Symbol(_) => ValueKind::Symbol,
            JsVariant::Object(_) => ValueKind::Object,
        })
    }

    fn is_callable(&mut self, value: &Root) -> Result<bool, Error> {
        Ok(self.value(value)?.is_callable())
    }

    fn is_constructor(&mut self, value: &Root) -> Result<bool, Error> {
        Ok(self.value(value)?.is_constructor())
    }

    fn global(&mut self) -> Result<Root, Error> {
        let global = self.context.global_this();
        self.root(global.into())
    }

    fn get(&mut self, object: &Root, key: PropertyKey<'_>) -> Result<Root, Error> {
        let object = self.object_value(object)?;
        let key = self.property_key(key)?;
        self.attempt_value(ExceptionPhase::PropertyRead, |context| {
            object.get(key, context)
        })
    }

    fn set(&mut self, object: &Root, key: PropertyKey<'_>, value: &Root) -> Result<(), Error> {
        self.validate_root(object)?;
        self.validate_root(value)?;
        let object = self.object_value(object)?;
        let key = self.property_key(key)?;
        let value = self.value(value)?;
        if !self.attempt(ExceptionPhase::PropertyWrite, |context| {
            object.set(key, value, false, context)
        })? {
            return Err(Error::PropertyRejected);
        }
        Ok(())
    }

    fn define_property(
        &mut self,
        object: &Root,
        key: PropertyKey<'_>,
        descriptor: PropertyDescriptor<'_>,
        definition: PropertyDefinition,
    ) -> Result<(), Error> {
        let object = self.object_value(object)?;
        let key = self.property_key(key)?;
        let builder = BoaPropertyDescriptor::builder();
        let (builder, enumerable, configurable) = match descriptor {
            PropertyDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => (
                builder.value(self.value(value)?).writable(writable),
                enumerable,
                configurable,
            ),
            PropertyDescriptor::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => (
                builder
                    .get(self.accessor_value(get)?)
                    .set(self.accessor_value(set)?),
                enumerable,
                configurable,
            ),
        };
        let descriptor = builder
            .enumerable(enumerable)
            .configurable(configurable)
            .build();
        let owner = self.state.owner.clone();
        if !self.attempt(ExceptionPhase::DefineProperty, |context| {
            if matches!(definition, PropertyDefinition::RequireAbsent) {
                if object.own_descriptor(&key, context)?.is_some() {
                    return Ok(false);
                }
                // The own-property probe can run a proxy trap. The enclosing
                // attempt reports its host refusal before interpreting false.
                if owner.available().is_err() {
                    return Ok(false);
                }
            }
            object.define_own_property(&key, descriptor, context)
        })? {
            return Err(Error::PropertyRejected);
        }
        Ok(())
    }

    fn object(&mut self, prototype: ObjectPrototype<'_>) -> Result<Root, Error> {
        let prepared = self.state.prepare(self.realm)?;
        let prototype = self.object_prototype(prototype)?;
        let object = JsObject::new(
            self.context.root_shape(),
            prototype,
            boa_engine::builtins::object::OrdinaryObject,
        );
        prepared.finish(Handle::Value(object.upcast().into()))
    }

    fn set_prototype(
        &mut self,
        object: &Root,
        prototype: ObjectPrototype<'_>,
    ) -> Result<(), Error> {
        let object = self.object_value(object)?;
        let prototype = self.object_prototype(prototype)?;
        if !self.attempt(ExceptionPhase::Prototype, |context| {
            object.set_internal_prototype(prototype, context)
        })? {
            return Err(Error::PropertyRejected);
        }
        Ok(())
    }

    fn array(&mut self, values: &[Root]) -> Result<Root, Error> {
        self.check_active()?;
        crate::backend::validate_array_length(values.len())?;
        let prepared = self.state.prepare(self.realm)?;
        let elements = self.outbound_values(values)?;
        let object =
            elements.map(|values| JsObject::from(JsArray::from_iter(values, self.context)));
        prepared.finish(Handle::Value(object.value.into()))
    }

    fn well_known_symbol(&mut self, symbol: WellKnownSymbol) -> Result<Root, Error> {
        let symbol = match symbol {
            WellKnownSymbol::Iterator => JsSymbol::iterator(),
            WellKnownSymbol::ToStringTag => JsSymbol::to_string_tag(),
            WellKnownSymbol::Unscopables => JsSymbol::unscopables(),
        };
        self.root(symbol.into())
    }

    fn intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Root, Error> {
        let objects = self.context.intrinsics().objects();
        let function = match intrinsic {
            Intrinsic::ArrayEntries => objects.array_prototype_entries(),
            Intrinsic::ArrayKeys => objects.array_prototype_keys(),
            Intrinsic::ArrayValues => objects.array_prototype_values(),
            Intrinsic::ArrayForEach => objects.array_prototype_for_each(),
        };
        self.root(function.into())
    }

    fn call(
        &mut self,
        function: &Root,
        receiver: &Root,
        arguments: &[Root],
        enter: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<Root, Error> {
        self.check_active()?;
        if arguments.len() > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        self.validate_root(function)?;
        self.validate_root(receiver)?;
        let prepared = self.state.prepare(self.realm)?;
        let function = self
            .value(function)?
            .as_callable()
            .ok_or(Error::ExpectedFunction)?;
        let receiver = self.value(receiver)?;
        let arguments = self.outbound_values(arguments)?;
        enter()?;
        self.finish_value(prepared, ExceptionPhase::Invoke, |context| {
            function.call(&receiver, &arguments.value, context)
        })
    }

    fn construct_without_arguments(&mut self, constructor: &Root) -> Result<Root, Error> {
        self.validate_root(constructor)?;
        let prepared = self.state.prepare(self.realm)?;
        let constructor = self
            .value(constructor)?
            .as_constructor()
            .ok_or(Error::ExpectedConstructor)?;
        self.finish_value(prepared, ExceptionPhase::Invoke, |context| {
            constructor.construct(&[], None, context).map(JsValue::from)
        })
    }

    fn coerce_string(&mut self, value: &Root) -> Result<CoercedString, Error> {
        let value = self.value(value)?;
        let mut length = 0;
        let value = self.attempt_value(ExceptionPhase::Coerce, |context| {
            let string = value.to_string(context)?;
            length = string.len();
            Ok(string.into())
        })?;
        Ok(CoercedString { value, length })
    }

    fn copy_string(&mut self, string: &CoercedString, units: &mut [u16]) -> Result<(), Error> {
        let value = self
            .value(&string.value)?
            .as_string()
            .ok_or(Error::BackendUnavailable)?;
        if units.len() != string.len() || units.len() != value.len() {
            return Err(Error::BackendUnavailable);
        }
        for (unit, source) in units.iter_mut().zip(value.iter()) {
            *unit = source;
        }
        Ok(())
    }

    fn to_boolean(&mut self, value: &Root) -> Result<bool, Error> {
        Ok(self.value(value)?.to_boolean())
    }

    fn to_number(&mut self, value: &Root) -> Result<f64, Error> {
        let value = self.value(value)?;
        self.attempt(ExceptionPhase::Coerce, |context| value.to_number(context))
    }

    fn strict_equals(&mut self, left: &Root, right: &Root) -> Result<bool, Error> {
        Ok(self.value(left)?.strict_equals(&self.value(right)?))
    }

    fn function(
        &mut self,
        callback: &crate::Callback,
        declared: Option<crate::backend::DeclaredCallable<'_>>,
    ) -> Result<crate::Root, crate::Error> {
        self.register_boa_callable(
            super::functions::BoaCallable::Function {
                callback: callback.clone(),
                maximum_arguments: declared.map(|shape| shape.maximum_arguments),
            },
            declared.map(|shape| shape.metadata),
        )
    }

    fn constructor(
        &mut self,
        callback: &crate::ConstructorCallback,
        metadata: crate::FunctionMetadata<'_>,
    ) -> Result<crate::Root, crate::Error> {
        self.register_boa_callable(
            super::functions::BoaCallable::Constructor(callback.clone()),
            Some(metadata),
        )
    }

    fn constructor_prototype(
        &mut self,
        new_target: &crate::Root,
        fallback: &crate::PrototypeResolver,
    ) -> Result<crate::Root, crate::Error> {
        let result = (|| {
            self.check_active()?;
            let target = self
                .value(new_target)?
                .as_object()
                .filter(boa_engine::JsObject::is_constructor)
                .ok_or(crate::Error::ExpectedFunction)?;
            let prototype = self.attempt_value(crate::ExceptionPhase::Prototype, |context| {
                target.get(boa_engine::js_string!("prototype"), context)
            })?;
            if self.value(&prototype)?.is_object() {
                return Ok(prototype);
            }
            drop(prototype);

            let target_realm = self.attempt(crate::ExceptionPhase::Prototype, |context| {
                target.callable_realm(context)
            })?;
            let guard = super::EnterRealm::new(self.context, target_realm);
            let mut entered = Entered {
                realm: self.state.current_key(guard.context)?,
                state: self.state.clone(),
                context: &mut *guard.context,
            };
            let result = fallback(&mut crate::Scope {
                operations: &mut entered,
                invariant: std::marker::PhantomData,
            })
            .map(|value| value.root);
            let prototype = self.state.owner.operation_result(result)?;
            if !self.state.value(&prototype)?.is_object() {
                return Err(crate::Error::ExpectedObject);
            }
            Ok(prototype)
        })();
        match result {
            Err(error @ crate::Error::Exception(_)) => Err(error),
            Err(error) => Err(self.refuse(error)),
            success => success,
        }
    }

    fn native_object_storage_bytes(&self) -> usize {
        native::NativeObjects::storage_bytes()
    }

    fn native_reference_storage_bytes(&self) -> usize {
        size_of::<(u32, JsObject)>()
    }

    fn project_native_object(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn std::any::Any>,
        slot: Option<super::NativeProjectionSlot<'_>>,
    ) -> Result<Root, Error> {
        self.project_native(prototype, payload, slot)
    }

    fn project_retained_native_object(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn std::any::Any>,
        backing: Box<dyn std::any::Any>,
        slot: Option<NativeProjectionSlot<'_>>,
        funding: Option<crate::preparation::Guard>,
    ) -> Result<
        (
            Rc<crate::callback_object::CallbackObjectIdentity>,
            u32,
            Root,
        ),
        Error,
    > {
        self.project_retained_native(prototype, payload, backing, slot, funding)
    }

    fn project_retained_native_object_in_prototype_realm(
        &mut self,
        prototype: &Rc<crate::callback_object::CallbackObjectIdentity>,
        realm: RealmKey,
        payload: Rc<dyn std::any::Any>,
        backing: Box<dyn std::any::Any>,
        slot: Option<NativeProjectionSlot<'_>>,
        funding: Option<crate::preparation::Guard>,
    ) -> Result<
        (
            Rc<crate::callback_object::CallbackObjectIdentity>,
            u32,
            Root,
        ),
        Error,
    > {
        self.project_retained_native_in_prototype_realm(
            prototype, realm, payload, backing, slot, funding,
        )
    }

    fn discard_unpublished_native_object(
        &mut self,
        index: u32,
        holder: &Rc<crate::callback_object::CallbackObjectIdentity>,
        root: &Root,
    ) {
        self.abort_native(index, holder, root);
    }

    fn native_object(&mut self, object: &Root) -> Result<Option<Rc<dyn std::any::Any>>, Error> {
        self.native_payload(object)
    }

    fn native_object_is_live(&self, index: u32, brand: std::any::TypeId) -> Result<bool, Error> {
        self.native_is_live(index, brand)
    }

    fn native_object_index(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
    ) -> Result<Option<u32>, Error> {
        self.native_index(object, brand)
    }

    fn open_native_object(
        &mut self,
        index: u32,
        brand: std::any::TypeId,
    ) -> Result<Option<Root>, Error> {
        self.reopen_native(index, brand)
    }

    fn clear_native_object_reference(
        &mut self,
        index: u32,
        brand: std::any::TypeId,
        slot: u32,
    ) -> Result<(), Error> {
        self.clear_native_reference(index, brand, slot)
    }

    fn native_object_reference(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
        slot: u32,
    ) -> Result<Option<Root>, Error> {
        self.read_native_reference(object, brand, slot)
    }

    fn set_native_object_reference(
        &mut self,
        object: &Root,
        brand: std::any::TypeId,
        slot: u32,
        reference: Option<&Root>,
    ) -> Result<(), Error> {
        self.write_native_reference(object, brand, slot, reference)
    }

    fn callback_object_realm(&mut self, value: &Root) -> Result<RealmKey, Error> {
        self.observe_callback_object(value)
    }

    fn with_callback_object_realm(
        &mut self,
        object: &Root,
        realm: RealmKey,
        action: &mut dyn FnMut(&mut dyn Operations) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.enter_callback_object(object, realm, |entered| action(entered))
    }

    fn callback_object_storage_bytes(&self) -> usize {
        callback_objects::CallbackObjects::storage_bytes()
    }

    fn retain_callback_object(
        &mut self,
        object: &Root,
        realm: RealmKey,
        backing: Box<dyn std::any::Any>,
    ) -> Result<Rc<crate::callback_object::CallbackObjectIdentity>, Error> {
        self.pin_callback_object(object, realm, backing)
    }

    fn validate_callback_object(
        &self,
        holder: &Rc<crate::callback_object::CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.check_active()?;
        self.state.callbacks.validate(&self.state, holder)
    }

    fn open_callback_object(
        &mut self,
        holder: &Rc<crate::callback_object::CallbackObjectIdentity>,
    ) -> Result<Root, Error> {
        self.check_active()?;
        let object = self.state.callbacks.open(&self.state, holder)?;
        self.root(object.into())
    }

    fn release_callback_object(
        &mut self,
        holder: &Rc<crate::callback_object::CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.state.callbacks.release(&self.state, holder)
    }

    fn open_callback_object_funded(
        &mut self,
        holder: &Rc<crate::callback_object::CallbackObjectIdentity>,
        backing: crate::preparation::Guard,
    ) -> Result<Root, Error> {
        self.check_active()?;
        let object = self.state.callbacks.open(&self.state, holder)?;
        self.funded(object.into(), backing)
    }

    fn new_promise_resolver(&mut self) -> Result<Root, Error> {
        self.new_handle(|context| {
            let (promise, resolving) =
                boa_engine::object::builtins::JsPromise::new_pending(context);
            Ok(Handle::Promise(promise, resolving))
        })
    }

    fn resolver_promise(&mut self, resolver: &Root) -> Result<Root, Error> {
        self.validate_root(resolver)?;
        let (promise, _) = self.state.promise(resolver)?;
        self.root(promise.into())
    }

    fn resolver_promise_funded(
        &mut self,
        resolver: &Root,
        backing: crate::preparation::Guard,
    ) -> Result<Root, Error> {
        self.validate_root(resolver)?;
        let (promise, _) = self.state.promise(resolver)?;
        self.funded(promise.into(), backing)
    }

    fn settle_promise(&mut self, resolver: &Root, value: &Root, reject: bool) -> Result<(), Error> {
        self.validate_root(resolver)?;
        self.validate_root(value)?;
        let realm = resolver.0.realm;
        let boa_realm = self.state.lookup_realm(realm)?;
        let (_, resolving) = self.state.promise(resolver)?;
        let value = self.value(value)?;
        let function = if reject {
            resolving.reject
        } else {
            resolving.resolve
        };
        self.in_boa_realm(realm, boa_realm, |entered| {
            entered.attempt(ExceptionPhase::Invoke, |context| {
                function
                    .call(&boa_engine::JsValue::undefined(), &[value], context)
                    .map(|_| ())
            })
        })
    }

    fn type_error(&mut self, message: &[u16]) -> Result<Root, Error> {
        self.boa_type_error(message)
    }

    fn native_error(&mut self) -> Result<Root, Error> {
        self.boa_native_error()
    }

    fn dom_exception_profile(
        &mut self,
    ) -> Result<Option<crate::backend::DomExceptionProfile>, Error> {
        self.boa_dom_exception_profile()
    }

    fn publish_dom_exception(
        &mut self,
        key: Option<&[u16]>,
        constructor: &Root,
        prototype: &Root,
        account: std::rc::Weak<crate::conversion::ConversionAccount>,
    ) -> Result<(), Error> {
        self.boa_publish_dom_exception(key, constructor, prototype, account)
    }

    fn create_dom_exception(
        &mut self,
        prototype: &Root,
        name: &Root,
        message: &Root,
        code: u16,
    ) -> Result<Root, Error> {
        self.boa_create_dom_exception(prototype, name, message, code)
    }

    fn dom_exception_slot(
        &mut self,
        receiver: &Root,
        slot: crate::dom_exception::Slot,
    ) -> Result<Option<Root>, Error> {
        self.boa_dom_exception_slot(receiver, slot)
    }

    fn cssom_prototypes(&mut self, prototypes: &[&Root]) -> Result<(), Error> {
        self.check_active()?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(prototypes.len())
            .map_err(|_| Error::Allocation)?;
        for value in prototypes {
            values.push(self.object_value(value)?);
        }
        let mut host = self.context.realm().host_defined_mut();
        if host.has::<cssom::CssomPrototypes>() {
            return Err(Error::PropertyRejected);
        }
        host.insert(cssom::CssomPrototypes(values));
        Ok(())
    }

    fn cssom_prototype(&mut self, index: u32) -> Result<Option<Root>, Error> {
        self.check_active()?;
        let index = usize::try_from(index).map_err(|_| Error::BackendUnavailable)?;
        let value = self
            .context
            .realm()
            .host_defined()
            .get::<cssom::CssomPrototypes>()
            .and_then(|prototypes| prototypes.0.get(index))
            .cloned();
        value.map(|value| self.root(value.into())).transpose()
    }
}
