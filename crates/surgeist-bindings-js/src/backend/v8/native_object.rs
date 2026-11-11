use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::rc::Rc;

use crate::callback_object::CallbackObjectIdentity;
use crate::{Error, ExceptionPhase, Root};

use super::{Entered, ObjectPrototype, Roots, prototype_value};

struct Entry {
    object: v8::Weak<v8::Object>,
    retired: Rc<Cell<bool>>,
    payload: Rc<dyn Any>,
}

struct ResolvedObject<'scope> {
    index: u32,
    object: v8::Local<'scope, v8::Object>,
    entry: Rc<Entry>,
}

struct PreparedObject {
    index: u32,
    root: Root,
    entry: Rc<Entry>,
}

#[derive(Default)]
pub(super) struct NativeObjects {
    next: Cell<u32>,
    entries: RefCell<BTreeMap<u32, Rc<Entry>>>,
}

impl NativeObjects {
    #[cfg(test)]
    pub(super) fn entries_count(&self) -> usize {
        self.entries.borrow().len()
    }

    pub(super) const fn storage_bytes() -> usize {
        size_of::<(u32, Rc<Entry>)>()
            + size_of::<Entry>()
            + size_of::<Cell<bool>>()
            + size_of::<Rc<Cell<bool>>>()
    }

    pub(super) fn reap(&self, roots: &Roots) {
        let mut last = 0;
        loop {
            let retired = {
                let Ok(mut entries) = self.entries.try_borrow_mut() else {
                    roots.conflict();
                    return;
                };
                let Some(index) = entries
                    .range((Excluded(last), Unbounded))
                    .find_map(|(index, entry)| entry.retired.get().then_some(*index))
                else {
                    return;
                };
                last = index;
                entries.remove(&index)
            };
            // Destructors run outside the registry loan and SDK callback.
            drop(retired);
        }
    }
}

impl Entered<'_, '_, '_> {
    pub(super) fn project_payload(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn Any>,
    ) -> Result<Root, Error> {
        let PreparedObject { index, root, entry } = self.prepare_payload(prototype, payload)?;
        self.roots
            .native_objects
            .entries
            .try_borrow_mut()
            .map_err(|_| self.roots.conflict())?
            .insert(index, entry);
        Ok(root)
    }

    pub(super) fn project_pinned_payload(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn Any>,
        backing: Box<dyn Any>,
    ) -> Result<Rc<CallbackObjectIdentity>, Error> {
        let prepared = self.prepare_payload(prototype, payload)?;
        let holder = self.pin_callback_object(&prepared.root, self.realm, backing)?;
        let roots = &self.roots;
        match roots.native_objects.entries.try_borrow_mut() {
            Ok(mut entries) => {
                entries.insert(prepared.index, prepared.entry);
                Ok(holder)
            }
            Err(_) => {
                let error = roots.conflict();
                // Neither the holder nor the object has escaped. Drop the
                // exact persistent entry before releasing its backing.
                let _ = roots.callback_objects.release(roots, &holder);
                Err(error)
            }
        }
    }

    fn prepare_payload(
        &mut self,
        prototype: ObjectPrototype<'_>,
        payload: Rc<dyn Any>,
    ) -> Result<PreparedObject, Error> {
        self.attempt(ExceptionPhase::Prototype, |scope, roots, realm, _| {
            let bank = &roots.native_objects;
            let index = bank
                .next
                .get()
                .checked_add(1)
                .ok_or(Error::IdentityExhausted)?;
            bank.next.set(index);
            let template = v8::ObjectTemplate::new(scope);
            if !template.set_internal_field_count(2) {
                return Err(Error::BackendUnavailable);
            }
            let object = template
                .new_instance(scope)
                .ok_or(Error::BackendUnavailable)?;
            let prototype = prototype_value(scope, roots, prototype)?;
            let identity = v8::Integer::new_from_unsigned(scope, index);
            let references = v8::Object::new(scope);
            let null = v8::null(scope);
            if references.set_prototype(scope, null.into()) != Some(true)
                || object.set_prototype(scope, prototype) != Some(true)
                || !object.set_internal_field(0, identity.into())
                || !object.set_internal_field(1, references.into())
            {
                return Err(Error::BackendUnavailable);
            }
            let root = roots.retain(scope, realm, object.into())?;
            let retired = Rc::new(Cell::new(false));
            let signal = retired.clone();
            let object = v8::Weak::with_guaranteed_finalizer(
                scope,
                object,
                Box::new(move || signal.set(true)),
            );
            Ok(PreparedObject {
                index,
                root,
                entry: Rc::new(Entry {
                    object,
                    retired,
                    payload,
                }),
            })
        })
    }

    pub(super) fn native_payload(&mut self, root: &Root) -> Result<Option<Rc<dyn Any>>, Error> {
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, _, _| {
            Ok(find_native(scope, roots, root)?.map(|resolved| resolved.entry.payload.clone()))
        })
    }

    pub(super) fn native_index(
        &mut self,
        root: &Root,
        brand: TypeId,
    ) -> Result<Option<u32>, Error> {
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, _, _| {
            Ok(find_native(scope, roots, root)?
                .filter(|resolved| resolved.entry.payload.as_ref().type_id() == brand)
                .map(|resolved| resolved.index))
        })
    }

    pub(super) fn open_native_index(
        &mut self,
        index: u32,
        brand: TypeId,
    ) -> Result<Option<Root>, Error> {
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, realm, _| {
            let Some(entry) = find_entry(roots, index)? else {
                return Ok(None);
            };
            if entry.payload.as_ref().type_id() != brand {
                return Err(Error::ExpectedNativeObject);
            }
            entry
                .object
                .to_local(scope)
                .map(|object| roots.retain(scope, realm, object.into()))
                .transpose()
        })
    }

    pub(super) fn read_native_reference(
        &mut self,
        root: &Root,
        brand: TypeId,
        slot: u32,
    ) -> Result<Option<Root>, Error> {
        self.attempt(ExceptionPhase::PropertyRead, |scope, roots, realm, _| {
            let references = reference_holder(scope, roots, root, brand)?;
            let value = references
                .get_index(scope, slot)
                .ok_or(Error::BackendUnavailable)?;
            if value.is_undefined() {
                return Ok(None);
            }
            roots.retain(scope, realm, value).map(Some)
        })
    }

    pub(super) fn write_native_reference(
        &mut self,
        root: &Root,
        brand: TypeId,
        slot: u32,
        reference: Option<&Root>,
    ) -> Result<(), Error> {
        self.attempt(ExceptionPhase::PropertyWrite, |scope, roots, _, _| {
            let references = reference_holder(scope, roots, root, brand)?;
            let accepted = match reference {
                Some(reference) => {
                    let value = roots.local(scope, reference)?;
                    if !value.is_object() {
                        return Err(Error::ExpectedObject);
                    }
                    references.set_index(scope, slot, value)
                }
                None => references.delete_index(scope, slot),
            };
            if accepted == Some(true) {
                Ok(())
            } else {
                Err(Error::BackendUnavailable)
            }
        })
    }
}

fn reference_holder<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    root: &Root,
    brand: TypeId,
) -> Result<v8::Local<'s, v8::Object>, Error> {
    let Some(ResolvedObject { object, entry, .. }) = find_native(scope, roots, root)? else {
        return Err(Error::ExpectedNativeObject);
    };
    if entry.payload.as_ref().type_id() != brand {
        return Err(Error::ExpectedNativeObject);
    }
    object
        .get_internal_field(scope, 1)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or(Error::BackendUnavailable)
}

fn find_native<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    roots: &Roots,
    root: &Root,
) -> Result<Option<ResolvedObject<'s>>, Error> {
    let value = roots.local(scope, root)?;
    if value.is_proxy() {
        return Ok(None);
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Ok(None);
    };
    let Some(index) = object
        .get_internal_field(scope, 0)
        .and_then(|value| v8::Local::<v8::Uint32>::try_from(value).ok())
    else {
        return Ok(None);
    };
    let index = index.value();
    let Some(entry) = find_entry(roots, index)? else {
        return Ok(None);
    };
    Ok(entry
        .object
        .to_local(scope)
        .filter(|original| *original == object)
        .map(|_| ResolvedObject {
            index,
            object,
            entry,
        }))
}

fn find_entry(roots: &Roots, index: u32) -> Result<Option<Rc<Entry>>, Error> {
    Ok(roots
        .native_objects
        .entries
        .try_borrow()
        .map_err(|_| roots.conflict())?
        .get(&index)
        .cloned())
}
