use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, btree_map::Entry as MapEntry};
use std::ops::Bound::{Excluded, Included};
use std::rc::Rc;

use boa_engine::gc::{Gc, GcRefCell};
use boa_engine::object::{
    JsPrototype, WeakJsObject,
    builtins::JsArray,
    host::{IndexedProperties, readonly_indexed, readonly_indexed_source},
};
use boa_engine::{
    Context, Finalize, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue, Trace,
};

use crate::backend::{NativeProjectionSlot, ObjectPrototype, validate_array_length};
use crate::callback_object::CallbackObjectIdentity;
use crate::preparation::Guard;
use crate::{Error, RealmKey, Root};

use super::{Entered, Handle, State};

struct Entry {
    object: WeakJsObject,
    payload: Rc<dyn Any>,
    realm: RealmKey,
}

struct Live {
    object: JsObject,
    payload: Rc<dyn Any>,
    realm: RealmKey,
}

#[derive(Default)]
pub(super) struct NativeObjects {
    next: Cell<u32>,
    entries: RefCell<BTreeMap<u32, Entry>>,
}

impl NativeObjects {
    pub(super) const fn storage_bytes() -> usize {
        size_of::<(u32, Entry)>()
    }

    fn next(&self) -> Result<u32, Error> {
        let index = self
            .next
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        self.next.set(index);
        Ok(index)
    }

    fn insert(&self, state: &State, index: u32, entry: Entry) -> Result<(), Error> {
        let mut entries = self
            .entries
            .try_borrow_mut()
            .map_err(|_| state.conflict())?;
        let MapEntry::Vacant(slot) = entries.entry(index) else {
            return Err(state.conflict());
        };
        slot.insert(entry);
        Ok(())
    }

    fn remove(&self, state: &State, index: u32) -> Result<(), Error> {
        // Payload destruction can release other native custody.
        let removed = self
            .entries
            .try_borrow_mut()
            .map_err(|_| state.conflict())?
            .remove(&index);
        drop(removed);
        Ok(())
    }

    fn live(&self, state: &State, index: u32) -> Result<Option<Live>, Error> {
        let entries = self.entries.try_borrow().map_err(|_| state.conflict())?;
        Ok(entries.get(&index).and_then(|entry| {
            entry.object.upgrade().map(|object| Live {
                object,
                payload: entry.payload.clone(),
                realm: entry.realm,
            })
        }))
    }

    fn canonical(
        &self,
        state: &State,
        object: &JsObject,
        index: u32,
    ) -> Result<Option<Live>, Error> {
        Ok(self
            .live(state, index)?
            .filter(|live| JsObject::equals(object, &live.object)))
    }

    pub(super) fn creation_realm(
        &self,
        state: &State,
        object: &JsObject,
    ) -> Result<Option<RealmKey>, Error> {
        let Some(data) = native_data(object) else {
            return Ok(None);
        };
        Ok(self
            .canonical(state, object, data.index)?
            .map(|live| live.realm))
    }

    pub(super) fn reap(&self, state: &State) {
        let end = self.next.get();
        let mut last = 0;
        loop {
            let retired = {
                let Ok(mut entries) = self.entries.try_borrow_mut() else {
                    state.conflict();
                    return;
                };
                let Some(index) = entries
                    .range((Excluded(last), Included(end)))
                    .find_map(|(&index, entry)| entry.object.upgrade().is_none().then_some(index))
                else {
                    return;
                };
                last = index;
                entries.remove(&index)
            };
            // Do not run the payload destructor inside a collector callback or registry loan.
            drop(retired);
        }
    }
}

#[derive(Trace, Finalize)]
struct NativeData {
    index: u32,
    references: GcRefCell<BTreeMap<u32, JsObject>>,
}

// The engine traces reference slots through the actual guest wrapper. The
// registry owns only a weak guest identity and a separately admitted Rust payload.
#[derive(Trace, Finalize, JsData)]
struct NativeCarrier(Gc<NativeData>);

#[derive(Trace, Finalize)]
struct Collection {
    native: Gc<NativeData>,
    read: JsObject,
    keys: JsObject,
}

fn native_data(object: &JsObject) -> Option<Gc<NativeData>> {
    if let Some(carrier) = object.downcast_ref::<NativeCarrier>() {
        return Some(Gc::clone(&carrier.0));
    }
    readonly_indexed_source::<Collection>(object).map(|collection| Gc::clone(&collection.native))
}

impl Collection {
    fn read(
        &self,
        holder: &JsObject,
        key: JsValue,
        presence: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let receiver = JsValue::from(holder.clone());
        if presence {
            self.read
                .call(&receiver, &[key, JsValue::from(true)], context)
        } else {
            self.read.call(&receiver, &[key], context)
        }
    }

    fn keys(
        &self,
        holder: &JsObject,
        named: bool,
        context: &mut Context,
    ) -> JsResult<(JsArray, u32)> {
        let receiver = JsValue::from(holder.clone());
        let observed = if named {
            self.keys.call(&receiver, &[JsValue::from(true)], context)?
        } else {
            self.keys.call(&receiver, &[], context)?
        };
        let array = JsArray::from_object(observed.as_object().ok_or_else(invalid_keys)?)
            .map_err(|_| invalid_keys())?;
        let length = u32::try_from(array.length(context)?).map_err(|_| invalid_keys())?;
        validate_array_length(length as usize).map_err(|_| invalid_keys())?;
        Ok((array, length))
    }

    fn numeric_keys(&self, holder: &JsObject, context: &mut Context) -> JsResult<Vec<u32>> {
        let (array, length) = self.keys(holder, false, context)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(length as usize)
            .map_err(|_| invalid_keys())?;
        for offset in 0..length {
            let value = array.get(offset, context)?;
            let number = value.as_number().ok_or_else(invalid_keys)?;
            if !(0.0..=f64::from(u32::MAX)).contains(&number) || number.fract() != 0.0 {
                return Err(invalid_keys());
            }
            indices.push(number as u32);
        }
        Ok(indices)
    }
}

fn invalid_keys() -> boa_engine::JsError {
    JsNativeError::typ()
        .with_message("Invalid native collection keys")
        .into()
}

impl IndexedProperties for Collection {
    const HAS_NAMED_PROPERTIES: bool = true;

    fn index_count(&self, holder: &JsObject, context: &mut Context) -> JsResult<u32> {
        self.numeric_keys(holder, context)
            .and_then(|keys| u32::try_from(keys.len()).map_err(|_| invalid_keys()))
    }

    fn indexed_keys(&self, holder: &JsObject, context: &mut Context) -> JsResult<Vec<u32>> {
        self.numeric_keys(holder, context)
    }

    fn indexed_value(
        &self,
        holder: &JsObject,
        index: u32,
        context: &mut Context,
    ) -> JsResult<Option<JsValue>> {
        let value = self.read(holder, JsValue::from(index), false, context)?;
        Ok((!value.is_undefined()).then_some(value))
    }

    fn index_present(
        &self,
        holder: &JsObject,
        index: u32,
        context: &mut Context,
    ) -> JsResult<bool> {
        let present = self.read(holder, JsValue::from(index), true, context)?;
        Ok(present.as_boolean() == Some(true))
    }

    fn named_keys(&self, holder: &JsObject, context: &mut Context) -> JsResult<Vec<JsString>> {
        let (array, length) = self.keys(holder, true, context)?;
        let mut names = Vec::new();
        names
            .try_reserve_exact(length as usize)
            .map_err(|_| invalid_keys())?;
        for offset in 0..length {
            names.push(
                array
                    .get(offset, context)?
                    .as_string()
                    .ok_or_else(invalid_keys)?,
            );
        }
        Ok(names)
    }

    fn named_value(
        &self,
        holder: &JsObject,
        name: &JsString,
        context: &mut Context,
    ) -> JsResult<Option<JsValue>> {
        let value = self.read(holder, JsValue::from(name.clone()), false, context)?;
        Ok((!value.is_undefined()).then_some(value))
    }

    fn named_present(
        &self,
        holder: &JsObject,
        name: &JsString,
        context: &mut Context,
    ) -> JsResult<bool> {
        let present = self.read(holder, JsValue::from(name.clone()), true, context)?;
        Ok(present.as_boolean() == Some(true))
    }
}

struct Prepared {
    index: u32,
    root: Root,
    entry: Entry,
}

struct Resolved {
    data: Gc<NativeData>,
    payload: Rc<dyn Any>,
}

impl Entered<'_> {
    fn native_prototype(&mut self, prototype: ObjectPrototype<'_>) -> Result<JsPrototype, Error> {
        match prototype {
            ObjectPrototype::Default => Ok(Some(
                self.context
                    .intrinsics()
                    .constructors()
                    .object()
                    .prototype(),
            )),
            ObjectPrototype::Null => Ok(None),
            ObjectPrototype::Object(root) => self.object_value(root).map(Some),
        }
    }

    fn prepare_native(
        &self,
        prototype: JsPrototype,
        realm: RealmKey,
        payload: Rc<dyn Any>,
        slot: Option<NativeProjectionSlot<'_>>,
        funding: Option<Guard>,
    ) -> Result<Prepared, Error> {
        self.state.owner.available()?;
        let index = self.state.native.next()?;
        let mut references = BTreeMap::new();
        if let Some(NativeProjectionSlot::Traced(slot, reference)) = slot.as_ref() {
            references.insert(
                *slot,
                self.state
                    .value(reference)?
                    .as_object()
                    .ok_or(Error::ExpectedObject)?,
            );
        }
        let native = Gc::new(NativeData {
            index,
            references: GcRefCell::new(references),
        });
        let object = match slot {
            Some(NativeProjectionSlot::Collection { read, keys }) => {
                let callable = |root| {
                    self.state
                        .value(root)?
                        .as_object()
                        .filter(JsObject::is_callable)
                        .ok_or(Error::ExpectedObject)
                };
                readonly_indexed(
                    prototype,
                    Collection {
                        native,
                        read: callable(read)?,
                        keys: callable(keys)?,
                    },
                )
            }
            _ => JsObject::from_proto_and_data(prototype, NativeCarrier(native)),
        };
        let entry = Entry {
            object: object.downgrade(),
            payload,
            realm,
        };
        let root = self
            .state
            .retain_funded(realm, Handle::Value(object.into()), funding)?;
        Ok(Prepared { index, root, entry })
    }

    pub(super) fn project_native(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn Any>,
        slot: Option<NativeProjectionSlot<'_>>,
    ) -> Result<Root, Error> {
        let prototype = self.native_prototype(prototype)?;
        let Prepared { index, root, entry } =
            self.prepare_native(prototype, self.realm, payload, slot, None)?;
        if let Err(error) = self.state.native.insert(&self.state, index, entry) {
            let _ = self.state.discard_root(&root);
            return Err(error);
        }
        Ok(root)
    }

    pub(super) fn project_retained_native(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn Any>,
        backing: Box<dyn Any>,
        funding: Option<Guard>,
    ) -> Result<(Rc<CallbackObjectIdentity>, u32, Root), Error> {
        let prototype = self.native_prototype(prototype)?;
        let prepared = self.prepare_native(prototype, self.realm, payload, None, funding)?;
        self.pin_native(prepared, self.realm, backing)
    }

    pub(super) fn project_retained_native_in_prototype_realm(
        &mut self,
        prototype: &Rc<CallbackObjectIdentity>,
        realm: RealmKey,
        payload: Rc<dyn Any>,
        backing: Box<dyn Any>,
        funding: Option<Guard>,
    ) -> Result<(Rc<CallbackObjectIdentity>, u32, Root), Error> {
        self.state.owner.available()?;
        let (prototype, actual) = self
            .state
            .callbacks
            .open_with_realm(&self.state, prototype)?;
        if self.state.realm_key(&actual)? != realm {
            return Err(Error::CallbackRealmUnavailable);
        }
        let prepared = self.prepare_native(Some(prototype), realm, payload, None, funding)?;
        self.pin_native(prepared, realm, backing)
    }

    fn pin_native(
        &mut self,
        prepared: Prepared,
        realm: RealmKey,
        backing: Box<dyn Any>,
    ) -> Result<(Rc<CallbackObjectIdentity>, u32, Root), Error> {
        let Prepared { index, root, entry } = prepared;
        // Pinning checks the exact native entry's creation realm first. The
        // object and entry remain private until both admissions succeed.
        if let Err(error) = self.state.native.insert(&self.state, index, entry) {
            let _ = self.state.discard_root(&root);
            return Err(error);
        }
        let holder = match self.pin_callback_object(&root, realm, backing) {
            Ok(holder) => holder,
            Err(error) => {
                let _ = self.state.native.remove(&self.state, index);
                let _ = self.state.discard_root(&root);
                return Err(error);
            }
        };
        Ok((holder, index, root))
    }

    pub(super) fn abort_native(
        &mut self,
        index: u32,
        holder: &Rc<CallbackObjectIdentity>,
        root: &Root,
    ) {
        let _ = self.state.callbacks.release(&self.state, holder);
        let _ = self.state.native.remove(&self.state, index);
        let _ = self.state.discard_root(root);
    }

    fn resolve_native(&mut self, root: &Root) -> Result<Option<Resolved>, Error> {
        self.state.owner.available()?;
        let Some(object) = self.state.value(root)?.as_object() else {
            return Ok(None);
        };
        let Some(data) = native_data(&object) else {
            return Ok(None);
        };
        Ok(self
            .state
            .native
            .canonical(&self.state, &object, data.index)?
            .map(|live| Resolved {
                data,
                payload: live.payload,
            }))
    }

    pub(super) fn native_payload(&mut self, root: &Root) -> Result<Option<Rc<dyn Any>>, Error> {
        Ok(self.resolve_native(root)?.map(|resolved| resolved.payload))
    }

    pub(super) fn native_index(
        &mut self,
        root: &Root,
        brand: TypeId,
    ) -> Result<Option<u32>, Error> {
        Ok(self
            .resolve_native(root)?
            .filter(|resolved| resolved.payload.as_ref().type_id() == brand)
            .map(|resolved| resolved.data.index))
    }

    pub(super) fn reopen_native(
        &mut self,
        index: u32,
        brand: TypeId,
    ) -> Result<Option<Root>, Error> {
        self.state.owner.available()?;
        let Some(live) = self.state.native.live(&self.state, index)? else {
            return Ok(None);
        };
        if live.payload.as_ref().type_id() != brand {
            return Err(Error::ExpectedNativeObject);
        }
        self.root(live.object.into()).map(Some)
    }

    fn native_references(&mut self, root: &Root, brand: TypeId) -> Result<Gc<NativeData>, Error> {
        self.resolve_native(root)?
            .filter(|resolved| resolved.payload.as_ref().type_id() == brand)
            .map(|resolved| resolved.data)
            .ok_or(Error::ExpectedNativeObject)
    }

    pub(super) fn read_native_reference(
        &mut self,
        root: &Root,
        brand: TypeId,
        slot: u32,
    ) -> Result<Option<Root>, Error> {
        let data = self.native_references(root, brand)?;
        let value = data
            .references
            .try_borrow()
            .map_err(|_| self.state.conflict())?
            .get(&slot)
            .cloned();
        value.map(|value| self.root(value.into())).transpose()
    }

    pub(super) fn write_native_reference(
        &mut self,
        root: &Root,
        brand: TypeId,
        slot: u32,
        reference: Option<&Root>,
    ) -> Result<(), Error> {
        let data = self.native_references(root, brand)?;
        let reference = reference
            .map(|reference| self.object_value(reference))
            .transpose()?;
        let mut references = data
            .references
            .try_borrow_mut()
            .map_err(|_| self.state.conflict())?;
        match reference {
            Some(reference) => {
                references.insert(slot, reference);
            }
            None => {
                references.remove(&slot);
            }
        }
        Ok(())
    }

    pub(super) fn clear_native_reference(
        &mut self,
        index: u32,
        brand: TypeId,
        slot: u32,
    ) -> Result<(), Error> {
        // This is terminal-safe cleanup: it creates no root and enters no guest call.
        let Some(live) = self.state.native.live(&self.state, index)? else {
            return Ok(());
        };
        if live.payload.as_ref().type_id() != brand {
            return Err(Error::ExpectedNativeObject);
        }
        let data = native_data(&live.object).ok_or_else(|| self.state.conflict())?;
        data.references
            .try_borrow_mut()
            .map_err(|_| self.state.conflict())?
            .remove(&slot);
        Ok(())
    }
}
