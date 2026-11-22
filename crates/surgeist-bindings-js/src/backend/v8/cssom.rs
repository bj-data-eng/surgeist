//! Private V8 lowering for authored CSSOM wrapper custody.

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

fn carrier(value: v8::Local<v8::Value>) -> Option<v8::Local<v8::Object>> {
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
    let carrier = carrier(object).ok_or(Error::ExpectedObject)?;
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

type Projected<'s> = (v8::Local<'s, v8::Object>, Rc<dyn std::any::Any>);

pub(super) fn projection<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    object: v8::Local<'s, v8::Value>,
) -> Result<Option<Projected<'s>>, Error> {
    let Some(carrier) = carrier(object) else {
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

pub(super) fn live_projection(
    roots: &Roots,
    identity: Projection,
) -> Result<Option<Rc<CallbackEntry>>, Error> {
    let Some(entry) = callback_entry(roots, identity.id)? else {
        return Ok(None);
    };
    if entry.lease.realm != identity.realm
        || !matches!(entry.lease.callback, NativeCallback::Projection(_))
    {
        return Err(roots.conflict());
    }
    Ok((!entry.object.is_empty()).then_some(entry))
}

pub(super) fn reopen<'s>(
    scope: &v8::PinScope<'s, '_>,
    roots: &Roots,
    identity: Projection,
) -> Result<Option<v8::Local<'s, v8::Value>>, Error> {
    Ok(live_projection(roots, identity)?.and_then(|entry| entry.object.to_local(scope)))
}
