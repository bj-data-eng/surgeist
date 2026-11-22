//! Property handlers for an ordinary native collection object.

use v8::{
    Array, Boolean, Function, IndexedPropertyHandlerConfiguration, Integer, Intercepted, Local,
    Name, NamedPropertyHandlerConfiguration, Object, ObjectTemplate, PinScope, PropertyAttribute,
    PropertyCallbackArguments, PropertyDescriptor, PropertyHandlerFlags, ReturnValue, String,
    Value,
};

pub(super) const READ_FIELD: usize = 2;
pub(super) const KEYS_FIELD: usize = 3;
pub(super) const FIELD_COUNT: usize = 4;

pub(super) fn install(template: Local<'_, ObjectTemplate>) {
    template.set_indexed_property_handler(
        IndexedPropertyHandlerConfiguration::new()
            .getter(get_index)
            .query(query_index)
            .descriptor(describe_index)
            .enumerator(enumerate_indices)
            .setter(set_index)
            .definer(define_index)
            .deleter(delete_index),
    );
    template.set_named_property_handler(
        NamedPropertyHandlerConfiguration::new()
            .getter(get_name)
            .query(query_name)
            .descriptor(describe_name)
            .enumerator(enumerate_names)
            .setter(set_name)
            .definer(define_name)
            .deleter(delete_name)
            .flags(
                PropertyHandlerFlags::NON_MASKING | PropertyHandlerFlags::ONLY_INTERCEPT_STRINGS,
            ),
    );
}

fn callback<'s>(
    scope: &mut PinScope<'s, '_>,
    holder: Local<'s, Object>,
    field: usize,
) -> Option<Local<'s, Function>> {
    let value = Local::<Value>::try_from(holder.get_internal_field(scope, field)?).ok()?;
    Local::<Function>::try_from(value).ok()
}

fn read<'s>(
    scope: &mut PinScope<'s, '_>,
    holder: Local<'s, Object>,
    key: Local<'s, Value>,
    presence_only: bool,
) -> Option<Local<'s, Value>> {
    let read = callback(scope, holder, READ_FIELD)?;
    if presence_only {
        let presence = v8::Boolean::new(scope, true);
        read.call(scope, holder.into(), &[key, presence.into()])
    } else {
        read.call(scope, holder.into(), &[key])
    }
}

fn project_value<'s>(
    observed: Option<Local<'s, Value>>,
    mut result: ReturnValue<'_>,
) -> Intercepted {
    match observed {
        Some(value) if value.is_undefined() => Intercepted::kNo,
        Some(value) => {
            result.set(value);
            Intercepted::kYes
        }
        None => Intercepted::kYes,
    }
}

fn get_index<'s>(
    scope: &mut PinScope<'s, '_>,
    index: u32,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_>,
) -> Intercepted {
    let key = Integer::new_from_unsigned(scope, index);
    project_value(read(scope, args.holder(), key.into(), false), result)
}

fn get_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_>,
) -> Intercepted {
    project_value(read(scope, args.holder(), name.into(), false), result)
}

fn project_attributes(
    exists: Option<Local<'_, Value>>,
    attributes: PropertyAttribute,
    mut result: ReturnValue<'_, Integer>,
) -> Intercepted {
    match exists {
        Some(present) if present.is_true() => {
            result.set_uint32(attributes.as_u32());
            Intercepted::kYes
        }
        Some(_) => Intercepted::kNo,
        None => Intercepted::kYes,
    }
}

fn query_index<'s>(
    scope: &mut PinScope<'s, '_>,
    index: u32,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Integer>,
) -> Intercepted {
    let key = Integer::new_from_unsigned(scope, index);
    let exists = read(scope, args.holder(), key.into(), true);
    project_attributes(exists, PropertyAttribute::READ_ONLY, result)
}

fn query_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Integer>,
) -> Intercepted {
    let exists = read(scope, args.holder(), name.into(), true);
    let attributes = PropertyAttribute::READ_ONLY | PropertyAttribute::DONT_ENUM;
    project_attributes(exists, attributes, result)
}

fn descriptor<'s>(
    scope: &mut PinScope<'s, '_>,
    args: PropertyCallbackArguments<'s>,
    key: Local<'s, Value>,
    enumerable: bool,
    mut result: ReturnValue<'_>,
) -> Intercepted {
    let Some(value) = read(scope, args.holder(), key, false) else {
        return Intercepted::kYes;
    };
    if value.is_undefined() {
        return Intercepted::kNo;
    }
    let (Some(val), Some(write), Some(enumerate), Some(configure)) = (
        String::new(scope, "value"),
        String::new(scope, "writable"),
        String::new(scope, "enumerable"),
        String::new(scope, "configurable"),
    ) else {
        return Intercepted::kYes;
    };
    let names = [val.into(), write.into(), enumerate.into(), configure.into()];
    let values = [
        value,
        v8::Boolean::new(scope, false).into(),
        v8::Boolean::new(scope, enumerable).into(),
        v8::Boolean::new(scope, true).into(),
    ];
    let null = v8::null(scope);
    result.set(Object::with_prototype_and_properties(scope, null.into(), &names, &values).into());
    Intercepted::kYes
}

fn describe_index<'s>(
    scope: &mut PinScope<'s, '_>,
    index: u32,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_>,
) -> Intercepted {
    let key = Integer::new_from_unsigned(scope, index);
    descriptor(scope, args, key.into(), true, result)
}

fn describe_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_>,
) -> Intercepted {
    descriptor(scope, args, name.into(), false, result)
}

fn enumerate<'s>(
    scope: &mut PinScope<'s, '_>,
    args: PropertyCallbackArguments<'s>,
    named: bool,
    mut result: ReturnValue<'_, Array>,
) {
    let holder = args.holder();
    let Some(keys) = callback(scope, holder, KEYS_FIELD) else {
        return;
    };
    let observed = if named {
        let named = v8::Boolean::new(scope, true);
        keys.call(scope, holder.into(), &[named.into()])
    } else {
        keys.call(scope, holder.into(), &[])
    };
    if let Some(observed) = observed
        && let Some(array) = snapshot_keys(scope, holder, observed, named)
    {
        result.set(array);
    }
}

fn snapshot_keys<'s>(
    scope: &mut PinScope<'s, '_>,
    holder: Local<'s, Object>,
    observed: Local<'s, Value>,
    named: bool,
) -> Option<Local<'s, Array>> {
    let Ok(observed) = Local::<Array>::try_from(observed) else {
        invalid_keys(scope);
        return None;
    };
    let length = observed.length();
    if usize::try_from(length).map_or(true, |length| {
        crate::backend::validate_array_length(length).is_err()
    }) {
        invalid_keys(scope);
        return None;
    }

    // V8's key accumulator assumes correctly typed elements. Copy the
    // checked values into a fresh array so an accessor cannot change a
    // previously validated entry or substitute a different backing.
    let result = Array::new(scope, 0);
    for index in 0..length {
        let value = observed.get_index(scope, index)?;
        if (named && !value.is_string()) || (!named && !value.is_uint32()) {
            invalid_keys(scope);
            return None;
        }
        // For all-key reflection, V8 consumes named interceptor results
        // without applying its usual non-masking lookup rule.
        if named && !holder.has_own_property(scope, Local::<Name>::try_from(value).ok()?)? {
            continue;
        }
        let key = Integer::new_from_unsigned(scope, result.length()).to_string(scope)?;
        if result.create_data_property(scope, key.into(), value) != Some(true) {
            return None;
        }
    }
    Some(result)
}

fn invalid_keys(scope: &mut PinScope<'_, '_>) {
    if let Some(message) = String::new(scope, "Invalid native collection keys") {
        let error = v8::Exception::type_error(scope, message);
        scope.throw_exception(error);
    }
}

fn enumerate_indices<'s>(
    scope: &mut PinScope<'s, '_>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Array>,
) {
    enumerate(scope, args, false, result);
}

fn enumerate_names<'s>(
    scope: &mut PinScope<'s, '_>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Array>,
) {
    enumerate(scope, args, true, result);
}

fn set_index<'s>(
    _: &mut PinScope<'s, '_>,
    _: u32,
    _: Local<'s, Value>,
    _: PropertyCallbackArguments<'s>,
    mut result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    result.set_bool(false);
    Intercepted::kYes
}

fn define_index<'s>(
    _: &mut PinScope<'s, '_>,
    _: u32,
    _: &PropertyDescriptor,
    _: PropertyCallbackArguments<'s>,
    mut result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    result.set_bool(false);
    Intercepted::kYes
}

fn delete_index<'s>(
    scope: &mut PinScope<'s, '_>,
    index: u32,
    args: PropertyCallbackArguments<'s>,
    mut result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    let key = Integer::new_from_unsigned(scope, index);
    if let Some(present) = read(scope, args.holder(), key.into(), true) {
        result.set_bool(!present.is_true());
    }
    Intercepted::kYes
}

fn refuse_visible_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    args: PropertyCallbackArguments<'s>,
    mut result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    match read(scope, args.holder(), name.into(), true) {
        Some(present) if present.is_true() => {
            result.set_bool(false);
            Intercepted::kYes
        }
        Some(_) => Intercepted::kNo,
        None => Intercepted::kYes,
    }
}

fn set_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    _: Local<'s, Value>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    refuse_visible_name(scope, name, args, result)
}

fn define_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    _: &PropertyDescriptor,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    refuse_visible_name(scope, name, args, result)
}

fn delete_name<'s>(
    scope: &mut PinScope<'s, '_>,
    name: Local<'s, Name>,
    args: PropertyCallbackArguments<'s>,
    result: ReturnValue<'_, Boolean>,
) -> Intercepted {
    refuse_visible_name(scope, name, args, result)
}
