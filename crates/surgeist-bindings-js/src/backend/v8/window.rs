//! The SDK templates that give an attached Window its immutable prototype chain.

use std::rc::Rc;

use crate::Error;

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
    _: v8::PropertyCallbackArguments<'scope>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    if scope
        .get_current_context()
        .get_slot::<NamedPropertiesReady>()
        .is_none()
    {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
}
