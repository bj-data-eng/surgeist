//! Private V8 lowering for authored CSSOM wrapper custody and indexed objects.

use super::*;

pub(super) fn values<'s, const N: usize>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    inputs: &[&Root],
) -> Result<[v8::Local<'s, v8::Value>; N], Error> {
    let mut output = [v8::undefined(scope).into(); N];
    let slots = output
        .get_mut(..inputs.len())
        .ok_or(Error::TooManyArguments)?;
    for (slot, input) in slots.iter_mut().zip(inputs) {
        *slot = roots.local(scope, input)?;
    }
    Ok(output)
}

pub(super) fn capture_reflect<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    reflect: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Array>, Error> {
    let mut methods = [v8::undefined(scope).into(); IndexedTrap::ALL.len()];
    for (entry, trap) in methods.iter_mut().zip(IndexedTrap::ALL) {
        *entry = pristine_method(scope, reflect, trap.name())?.into();
    }
    Ok(v8::Array::new_with_elements(scope, &methods))
}

fn reflection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    intrinsics: v8::Local<'s, v8::Object>,
    trap: IndexedTrap,
    arguments: &[v8::Local<'s, v8::Value>],
) -> Result<v8::Local<'s, v8::Value>, Error> {
    let array = captured_value(scope, intrinsics, INDEXED_REFLECT_FIELD)?;
    let array = v8::Local::<v8::Array>::try_from(array).map_err(|_| Error::BackendUnavailable)?;
    let method = array
        .get_index(scope, trap as u32)
        .ok_or(Error::BackendUnavailable)?;
    let method =
        v8::Local::<v8::Function>::try_from(method).map_err(|_| Error::BackendUnavailable)?;
    let receiver = v8::undefined(scope);
    method
        .call(scope, receiver.into(), arguments)
        .ok_or(Error::BackendUnavailable)
}

fn carrier<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Option<v8::Local<'s, v8::Object>> {
    let value = if let Ok(proxy) = v8::Local::<v8::Proxy>::try_from(value) {
        if proxy.is_revoked() {
            return None;
        }
        proxy.get_handler(scope)
    } else {
        value
    };
    if value.is_proxy() {
        return None;
    }
    v8::Local::<v8::Object>::try_from(value).ok()
}

fn callback_entry(roots: &Roots, id: u32) -> Result<Option<Rc<CallbackEntry>>, Error> {
    Ok(roots
        .callbacks
        .try_borrow()
        .map_err(|_| roots.conflict())?
        .get(&id)
        .filter(|entry| !entry.retired.get())
        .cloned())
}

pub(super) fn project<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    realm: RealmKey,
    intrinsics: v8::Local<'s, v8::Object>,
    object: &Root,
    state: Rc<dyn std::any::Any>,
    links: &[&Root],
) -> Result<Projection, Error> {
    let object = roots.local(scope, object)?;
    let carrier = carrier(scope, object).ok_or(Error::ExpectedObject)?;
    let key = v8::Local::new(scope, &roots.projection_key);
    match carrier.has_private(scope, key) {
        Some(false) => (),
        Some(true) => return Err(Error::PropertyRejected),
        None => return Err(Error::BackendUnavailable),
    }
    let id = roots
        .next_callback
        .get()
        .checked_add(1)
        .ok_or(Error::IdentityExhausted)?;
    roots.next_callback.set(id);
    let fields = 4_usize
        .checked_add(links.len())
        .ok_or(Error::TooManyArguments)?;
    let context = scope.get_current_context();
    let holder = super::managed_holder(scope, object, context, intrinsics, fields)?;
    let id_value = v8::Integer::new_from_unsigned(scope, id);
    if !holder.set_internal_field(3, id_value.into()) {
        return Err(Error::BackendUnavailable);
    }
    for (index, link) in links.iter().enumerate() {
        let link = roots.local(scope, link)?;
        if !holder.set_internal_field(4 + index, link.into()) {
            return Err(Error::BackendUnavailable);
        }
    }
    if carrier.set_private(scope, key, holder.into()) != Some(true) {
        return Err(Error::BackendUnavailable);
    }
    let lease = Rc::new(CallbackLease {
        callback: NativeCallback::Projection(state),
        realm,
    });
    roots.register_callback(scope, id, object, lease)?;
    Ok(Projection { realm, id })
}

pub(super) fn projection<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    object: v8::Local<'s, v8::Value>,
) -> Result<Option<(v8::Local<'s, v8::Object>, Rc<dyn std::any::Any>)>, Error> {
    let Some(carrier) = carrier(scope, object) else {
        return Ok(None);
    };
    let key = v8::Local::new(scope, &roots.projection_key);
    if !carrier
        .has_private(scope, key)
        .ok_or(Error::BackendUnavailable)?
    {
        return Ok(None);
    }
    let holder = carrier
        .get_private(scope, key)
        .ok_or(Error::BackendUnavailable)?;
    let holder = v8::Local::<v8::Object>::try_from(holder).map_err(|_| roots.conflict())?;
    if !captured_value(scope, holder, 0)?.strict_equals(object) {
        return Ok(None);
    }
    let id = captured_value(scope, holder, 3)?;
    let id = v8::Local::<v8::Uint32>::try_from(id)
        .map_err(|_| roots.conflict())?
        .value();
    let entry = callback_entry(roots, id)?.ok_or_else(|| roots.conflict())?;
    let NativeCallback::Projection(state) = &entry.lease.callback else {
        return Err(roots.conflict());
    };
    Ok(Some((holder, state.clone())))
}

pub(super) fn reopen<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    identity: Projection,
) -> Result<Option<v8::Local<'s, v8::Value>>, Error> {
    let Some(entry) = callback_entry(roots, identity.id)? else {
        return Ok(None);
    };
    if entry.lease.realm != identity.realm
        || !matches!(entry.lease.callback, NativeCallback::Projection(_))
    {
        return Err(roots.conflict());
    }
    Ok(entry.object.to_local(scope))
}

pub(super) fn indexed_proxy<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    prototype: &Root,
    handlers: &[&Root; 9],
) -> Result<v8::Local<'s, v8::Proxy>, Error> {
    let prototype = roots.local(scope, prototype)?;
    let target = v8::Object::with_prototype_and_properties(scope, prototype, &[], &[]);
    let null = v8::null(scope);
    let handler = v8::Object::with_prototype_and_properties(scope, null.into(), &[], &[]);
    // Define traps through V8's normal property path. The bulk constructor
    // fills a dictionary directly, which misses the engine's interesting-name
    // bookkeeping for fast Proxy get, including Array iterator reads.
    for (trap, value) in IndexedTrap::ALL.into_iter().zip(handlers) {
        let name = v8::String::new(scope, trap.name()).ok_or(Error::BackendUnavailable)?;
        let value = roots.local(scope, value)?;
        if handler.create_data_property(scope, name.into(), value) != Some(true) {
            return Err(Error::BackendUnavailable);
        }
    }
    v8::Proxy::new(scope, target, handler).ok_or(Error::BackendUnavailable)
}

pub(super) fn indexed_key(
    scope: &v8::PinScope,
    key: v8::Local<v8::Value>,
) -> Result<Option<u32>, Error> {
    if key.is_symbol() {
        return Ok(None);
    }
    let key = v8::Local::<v8::String>::try_from(key).map_err(|_| Error::ExpectedPropertyKey)?;
    let mut units = [0; 10];
    let Some(units) = units.get_mut(..key.length()) else {
        return Ok(None);
    };
    key.write_v2(scope, 0, units, v8::WriteFlags::empty());
    if units.is_empty() || (units.len() > 1 && units.first() == Some(&u16::from(b'0'))) {
        return Ok(None);
    }
    let mut index = 0_u32;
    for &unit in units.iter() {
        let Some(digit) = unit
            .checked_sub(u16::from(b'0'))
            .filter(|digit| *digit < 10)
        else {
            return Ok(None);
        };
        let Some(value) = index
            .checked_mul(10)
            .and_then(|index| index.checked_add(u32::from(digit)))
        else {
            return Ok(None);
        };
        index = value;
    }
    Ok((index != u32::MAX).then_some(index))
}

fn descriptor_fields<'s>(
    scope: &v8::PinScope<'s, '_>,
    fields: &[(&str, v8::Local<'s, v8::Value>)],
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let null = v8::null(scope);
    let object = v8::Object::with_prototype_and_properties(scope, null.into(), &[], &[]);
    for &(key, value) in fields {
        let key = v8::String::new(scope, key).ok_or(Error::BackendUnavailable)?;
        if object.create_data_property(scope, key.into(), value) != Some(true) {
            return Err(Error::BackendUnavailable);
        }
    }
    Ok(object)
}

pub(super) fn descriptor<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let yes = v8::Boolean::new(scope, true).into();
    let no = v8::Boolean::new(scope, false).into();
    descriptor_fields(
        scope,
        &[
            ("value", value),
            ("writable", no),
            ("enumerable", yes),
            ("configurable", yes),
        ],
    )
}

pub(super) fn fallback<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    intrinsics: v8::Local<'s, v8::Object>,
    trap: IndexedTrap,
    arguments: &[&Root],
) -> Result<v8::Local<'s, v8::Value>, Error> {
    let count = arguments.len();
    let mut values = values::<4>(scope, roots, arguments)?;
    let arguments = values.get_mut(..count).ok_or(Error::TooManyArguments)?;
    match trap {
        IndexedTrap::Define => {
            let descriptor = arguments.get_mut(2).ok_or(Error::BackendUnavailable)?;
            let input = v8::Local::<v8::Object>::try_from(*descriptor)
                .map_err(|_| Error::BackendUnavailable)?;
            let null = v8::null(scope);
            let output = v8::Object::with_prototype_and_properties(scope, null.into(), &[], &[]);
            for field in [
                "value",
                "writable",
                "get",
                "set",
                "enumerable",
                "configurable",
            ] {
                let key = v8::String::new(scope, field).ok_or(Error::BackendUnavailable)?;
                if input
                    .has_own_property(scope, key.into())
                    .ok_or(Error::BackendUnavailable)?
                {
                    let value = input
                        .get(scope, key.into())
                        .ok_or(Error::BackendUnavailable)?;
                    if output.create_data_property(scope, key.into(), value) != Some(true) {
                        return Err(Error::BackendUnavailable);
                    }
                }
            }
            *descriptor = output.into();
        }
        _ => (),
    }
    reflection(scope, intrinsics, trap, arguments)
}

pub(super) fn own_keys<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    intrinsics: v8::Local<'s, v8::Object>,
    target: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Array>, Error> {
    let existing = reflection(scope, intrinsics, IndexedTrap::Keys, &[target])?;
    v8::Local::<v8::Array>::try_from(existing).map_err(|_| Error::BackendUnavailable)
}

pub(super) fn keys<'s>(
    scope: &v8::PinScope<'s, '_>,
    existing: v8::Local<'s, v8::Array>,
    length: u32,
) -> Result<v8::Local<'s, v8::Array>, Error> {
    let count = length
        .checked_add(existing.length())
        .ok_or(Error::ArrayTooLong)?;
    super::super::validate_array_length(count as usize)?;
    let output = v8::Array::new(
        scope,
        i32::try_from(count).map_err(|_| Error::ArrayTooLong)?,
    );
    for index in 0..count {
        let property = v8::Integer::new_from_unsigned(scope, index)
            .to_string(scope)
            .ok_or(Error::BackendUnavailable)?;
        let value = if index < length {
            property.into()
        } else {
            existing
                .get_index(scope, index - length)
                .ok_or(Error::BackendUnavailable)?
        };
        if output.create_data_property(scope, property.into(), value) != Some(true) {
            return Err(Error::BackendUnavailable);
        }
    }
    Ok(output)
}

pub(super) fn prototype_step<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    target: v8::Local<'s, v8::Value>,
    candidate: v8::Local<'s, v8::Value>,
) -> Result<Result<Option<v8::Local<'s, v8::Value>>, ()>, Error> {
    let Ok(object) = v8::Local::<v8::Object>::try_from(candidate) else {
        return Ok(Ok(None));
    };
    let object = if let Ok(proxy) = v8::Local::<v8::Proxy>::try_from(candidate) {
        if projection(scope, roots, candidate)?.is_none() {
            // The ordinary cycle test stops at an object with custom [[GetPrototypeOf]].
            return Ok(Ok(None));
        }
        v8::Local::<v8::Object>::try_from(proxy.get_target(scope))
            .map_err(|_| Error::BackendUnavailable)?
    } else {
        object
    };
    if target.strict_equals(object.into()) {
        return Ok(Err(()));
    }
    let prototype = object
        .get_prototype(scope)
        .ok_or(Error::BackendUnavailable)?;
    Ok(Ok(Some(prototype)))
}
