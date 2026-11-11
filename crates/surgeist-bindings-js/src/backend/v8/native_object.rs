use std::any::Any;
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
            if !template.set_internal_field_count(1) {
                return Err(Error::BackendUnavailable);
            }
            let object = template
                .new_instance(scope)
                .ok_or(Error::BackendUnavailable)?;
            let prototype = prototype_value(scope, roots, prototype)?;
            let identity = v8::Integer::new_from_unsigned(scope, index);
            if object.set_prototype(scope, prototype) != Some(true)
                || !object.set_internal_field(0, identity.into())
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
            let entry = roots
                .native_objects
                .entries
                .try_borrow()
                .map_err(|_| roots.conflict())?
                .get(&index.value())
                .cloned();
            let Some(entry) = entry else {
                return Ok(None);
            };
            Ok(entry
                .object
                .to_local(scope)
                .filter(|original| *original == object)
                .map(|_| entry.payload.clone()))
        })
    }
}
