//! The ECMAScript `WeakRef` builtin and shared storage for weak collections.

use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    rc::Rc,
};

use surgeist_js_gc::heap;

use crate::{JsResult, JsValue, Runtime, js_error, object::JsObject, value::StoredValue};

mod weak_ref;

pub(crate) use weak_ref::WeakRef;

/// Storage shared by weak collections. A key's identity includes its heap and
/// generation, so a newly allocated object cannot match a reclaimed key.
#[derive(surgeist_js_macros::JsData)]
pub(crate) struct WeakObjectMap<V> {
    entries: Rc<RefCell<HashMap<heap::ObjectId, WeakEntry<V>>>>,
}

struct WeakEntry<V> {
    value: V,
    // Dropping the entry cancels its registration on the key.
    _retirement: Rc<dyn Fn()>,
}

impl<V: 'static> WeakObjectMap<V> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Rc::default(),
        }
    }

    pub(super) fn contains(&self, key: &JsObject) -> bool {
        self.entries.borrow().contains_key(&key.identity())
    }

    pub(super) fn insert(&self, key: &JsObject, value: V) {
        let id = key.identity();
        let mut entries = self.entries.borrow_mut();
        match entries.entry(id) {
            Entry::Occupied(mut entry) => {
                let replaced = std::mem::replace(&mut entry.get_mut().value, value);
                drop(entries);
                drop(replaced);
            }
            Entry::Vacant(entry) => {
                let entries = Rc::downgrade(&self.entries);
                // Retirement runs after tracing and sweeping. The observer touches
                // only the table, which can be updated while the object is read.
                let retirement: Rc<dyn Fn()> = Rc::new(move || {
                    if let Some(entries) = entries.upgrade() {
                        let retired = entries.borrow_mut().remove(&id);
                        drop(retired);
                    }
                });
                key.observe_retirement(&retirement);
                entry.insert(WeakEntry {
                    value,
                    _retirement: retirement,
                });
            }
        }
    }

    pub(super) fn remove(&self, key: &JsObject) -> bool {
        let removed = self.entries.borrow_mut().remove(&key.identity());
        removed.is_some()
    }
}

impl WeakObjectMap<StoredValue> {
    pub(super) fn get(&self, key: &JsObject) -> Option<JsValue> {
        let value = self
            .entries
            .borrow()
            .get(&key.identity())
            .map(|entry| entry.value.clone());
        // The caller holds roots for the map and key. Install the result's
        // root after releasing the table borrow.
        value.map(StoredValue::into_value)
    }
}

impl heap::Trace for WeakObjectMap<StoredValue> {
    fn trace(&self, edges: &mut heap::EdgeVisitor<'_>) -> Result<(), heap::AccessError> {
        let entries = self
            .entries
            .try_borrow()
            .map_err(|_| heap::AccessError::PayloadBorrowed)?;
        for (&key, entry) in entries.iter() {
            if let StoredValue::Object(object) = &entry.value {
                edges.ephemeron(key, object.identity());
            }
        }
        Ok(())
    }
}

impl heap::Trace for WeakObjectMap<()> {
    fn trace(&self, _: &mut heap::EdgeVisitor<'_>) -> Result<(), heap::AccessError> {
        Ok(())
    }
}

pub(super) fn require_same_runtime(runtime: &Runtime, target: &JsObject) -> JsResult<()> {
    if !runtime.same_heap(&target.runtime()) {
        return Err(js_error!(TypeError: "weak reference crosses JavaScript runtimes"));
    }
    Ok(())
}
