//! The SDK templates that give an attached Window its immutable prototype chain.

use std::rc::Rc;

use crate::{Error, RealmKey};

pub(super) struct Templates<'scope> {
    window: v8::Local<'scope, v8::FunctionTemplate>,
    named_properties: v8::Local<'scope, v8::FunctionTemplate>,
    event_target: v8::Local<'scope, v8::FunctionTemplate>,
}

struct NamedPropertiesReady;

impl<'scope> Templates<'scope> {
    pub(super) fn new(scope: &mut v8::PinScope<'scope, '_, ()>) -> Result<Self, Error> {
        let event_target = v8::FunctionTemplate::new(scope, reject_construction);
        event_target.prototype_template(scope).set_immutable_proto();

        let named_properties = v8::FunctionTemplate::new(scope, reject_construction);
        named_properties.inherit(event_target);
        let names = named_properties.prototype_template(scope);
        names.set_immutable_proto();
        names.set_named_property_handler(
            v8::NamedPropertyHandlerConfiguration::new()
                .definer(reject_definition)
                .deleter(reject_deletion),
        );
        names.set_indexed_property_handler(
            v8::IndexedPropertyHandlerConfiguration::new()
                .definer(reject_index_definition)
                .deleter(reject_index_deletion),
        );
        install_tag(scope, names, "WindowProperties")?;

        let window = v8::FunctionTemplate::new(scope, reject_construction);
        window.inherit(named_properties);
        window.set_class_name(v8::String::new(scope, "Window").ok_or(Error::BackendUnavailable)?);
        window.instance_template(scope).set_immutable_proto();
        let prototype = window.prototype_template(scope);
        prototype.set_immutable_proto();
        install_tag(scope, prototype, "Window")?;
        Ok(Self {
            window,
            named_properties,
            event_target,
        })
    }

    pub(super) fn global_template(
        &self,
        scope: &v8::PinScope<'scope, '_, ()>,
    ) -> v8::Local<'scope, v8::ObjectTemplate> {
        self.window.instance_template(scope)
    }

    pub(super) fn open(
        self,
        scope: &mut v8::PinScope<'scope, '_>,
    ) -> Result<(v8::Local<'scope, v8::Object>, v8::Local<'scope, v8::Object>), Error> {
        let window = self
            .window
            .get_function(scope)
            .ok_or(Error::BackendUnavailable)?;
        let names = self
            .named_properties
            .get_function(scope)
            .ok_or(Error::BackendUnavailable)?;
        let event_target = self
            .event_target
            .get_function(scope)
            .ok_or(Error::BackendUnavailable)?;
        let prototype = v8::String::new(scope, "prototype").ok_or(Error::BackendUnavailable)?;
        let window = window
            .get(scope, prototype.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or(Error::BackendUnavailable)?;
        let names = names
            .get(scope, prototype.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or(Error::BackendUnavailable)?;
        let constructor_name =
            v8::String::new(scope, "constructor").ok_or(Error::BackendUnavailable)?;
        // The SDK inserts a constructor on every function template prototype.
        // The named properties object has no corresponding interface object.
        if names.delete(scope, constructor_name.into()) != Some(true)
            || scope
                .get_current_context()
                .set_slot(Rc::new(NamedPropertiesReady))
                .is_some()
        {
            return Err(Error::BackendUnavailable);
        }
        let target = event_target
            .get(scope, prototype.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or(Error::BackendUnavailable)?;
        if target.delete(scope, constructor_name.into()) != Some(true)
            || window.delete(scope, constructor_name.into()) != Some(true)
        {
            return Err(Error::BackendUnavailable);
        }
        Ok((window, target))
    }
}

pub(super) fn qualify_event_target<'scope>(
    scope: &mut v8::PinScope<'scope, '_>,
    constructor: v8::Local<'scope, v8::Value>,
    prototype: v8::Local<'scope, v8::Value>,
    constructor_key: v8::Local<'scope, v8::Private>,
    realm: RealmKey,
) -> Result<(), Error> {
    let proxy =
        v8::Local::<v8::Proxy>::try_from(constructor).map_err(|_| Error::PropertyRejected)?;
    if proxy.is_revoked() {
        return Err(Error::PropertyRejected);
    }
    let handler = v8::Local::<v8::Object>::try_from(proxy.get_handler(scope))
        .map_err(|_| Error::PropertyRejected)?;
    let target = v8::Local::<v8::Function>::try_from(proxy.get_target(scope))
        .map_err(|_| Error::PropertyRejected)?;
    // Inspect the actual private handler and target. A guest proxy cannot
    // obtain the handler or synthesize this private property.
    if handler.is_proxy()
        || !handler
            .get_private(scope, constructor_key)
            .is_some_and(|value| value.strict_equals(target.into()))
        || target
            .get_creation_context(scope)
            .and_then(|context| context.get_slot::<RealmKey>())
            .map(|key| *key)
            != Some(realm)
    {
        return Err(Error::PropertyRejected);
    }
    let global = scope.get_current_context().global(scope);
    let prototype =
        v8::Local::<v8::Object>::try_from(prototype).map_err(|_| Error::PropertyRejected)?;
    if !matches_data_property(scope, global, "EventTarget", constructor, true)?
        || !matches_data_property(scope, target.into(), "prototype", prototype.into(), false)?
        || !matches_data_property(scope, prototype, "constructor", constructor, true)?
    {
        return Err(Error::PropertyRejected);
    }
    Ok(())
}

fn matches_data_property<'scope>(
    scope: &mut v8::PinScope<'scope, '_>,
    object: v8::Local<'scope, v8::Object>,
    property: &str,
    expected: v8::Local<'scope, v8::Value>,
    writable_and_configurable: bool,
) -> Result<bool, Error> {
    let key = v8::String::new(scope, property).ok_or(Error::BackendUnavailable)?;
    let descriptor = object
        .get_own_property_descriptor(scope, key.into())
        .ok_or(Error::BackendUnavailable)?;
    let Ok(descriptor) = v8::Local::<v8::Object>::try_from(descriptor) else {
        return Ok(false);
    };
    let key = v8::String::new(scope, "value").ok_or(Error::BackendUnavailable)?;
    if descriptor.has_own_property(scope, key.into()) != Some(true)
        || !descriptor
            .get(scope, key.into())
            .is_some_and(|value| value.strict_equals(expected))
    {
        return Ok(false);
    }
    for (flag, expected) in [
        ("writable", writable_and_configurable),
        ("configurable", writable_and_configurable),
        ("enumerable", false),
    ] {
        let key = v8::String::new(scope, flag).ok_or(Error::BackendUnavailable)?;
        let value = descriptor
            .get(scope, key.into())
            .ok_or(Error::BackendUnavailable)?;
        if (expected && !value.is_true()) || (!expected && !value.is_false()) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn install_tag(
    scope: &mut v8::PinScope<'_, '_, ()>,
    template: v8::Local<'_, v8::ObjectTemplate>,
    name: &str,
) -> Result<(), Error> {
    let key = v8::Symbol::get_to_string_tag(scope);
    let value = v8::String::new(scope, name).ok_or(Error::BackendUnavailable)?;
    template.set_with_attr(
        key.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    );
    Ok(())
}

fn reject_construction<'scope>(
    scope: &mut v8::PinScope<'scope, '_>,
    _: v8::FunctionCallbackArguments<'scope>,
    _: v8::ReturnValue<'scope>,
) {
    if let Some(message) = v8::String::new(scope, "Illegal constructor") {
        let exception = v8::Exception::type_error(scope, message);
        scope.throw_exception(exception);
    }
}

fn reject_definition<'scope>(
    _: &mut v8::PinScope<'scope, '_>,
    _: v8::Local<'scope, v8::Name>,
    _: &v8::PropertyDescriptor,
    _: v8::PropertyCallbackArguments<'scope>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn reject_deletion<'scope>(
    scope: &mut v8::PinScope<'scope, '_>,
    _: v8::Local<'scope, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'scope>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    if arguments
        .holder()
        .get_creation_context(scope)
        .is_some_and(|context| context.get_slot::<NamedPropertiesReady>().is_none())
    {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn reject_index_definition<'scope>(
    _: &mut v8::PinScope<'scope, '_>,
    _: u32,
    _: &v8::PropertyDescriptor,
    _: v8::PropertyCallbackArguments<'scope>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn reject_index_deletion<'scope>(
    _: &mut v8::PinScope<'scope, '_>,
    _: u32,
    _: v8::PropertyCallbackArguments<'scope>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    result.set_bool(false);
    v8::Intercepted::kYes
}
