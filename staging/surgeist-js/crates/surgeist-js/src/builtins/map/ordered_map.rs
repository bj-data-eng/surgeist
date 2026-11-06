//! Implements a map type that preserves insertion order.

use crate::{JsData, JsResult, JsValue, Runtime, js_error, value::StoredValue};
use indexmap::{Equivalent, IndexMap};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace as HeapTrace};

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum MapKey {
    Key(StoredValue),
    Empty(usize), // Necessary to ensure empty keys are still unique.
}

// This ensures that a MapKey::Key(value) hashes to the same as value. The derived PartialEq implementation still holds.
impl Hash for MapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Key(v) => v.hash(state),
            Self::Empty(e) => e.hash(state),
        }
    }
}

impl Equivalent<MapKey> for StoredValue {
    fn equivalent(&self, key: &MapKey) -> bool {
        match key {
            MapKey::Key(v) => v == self,
            MapKey::Empty(_) => false,
        }
    }
}

/// A structure wrapping `indexmap::IndexMap`.
#[derive(JsData)]
pub(crate) struct OrderedMap {
    map: IndexMap<MapKey, Option<StoredValue>>,
    // Every active iteration holds a clone of this token. Dropping it is
    // sufficient to release the lock, even during heap retirement.
    iteration_lock: Rc<()>,
    empty_count: usize,
}

impl HeapTrace for OrderedMap {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        for (key, value) in &self.map {
            if let MapKey::Key(key) = key {
                HeapTrace::trace(key, edges)?;
            }
            HeapTrace::trace(value, edges)?;
        }
        Ok(())
    }
}

impl Debug for OrderedMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.map.fmt(formatter)
    }
}

impl Default for OrderedMap {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderedMap {
    /// Creates a new empty `OrderedMap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            iteration_lock: Rc::new(()),
            empty_count: 0,
        }
    }

    /// Creates a new empty `OrderedMap` with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(capacity),
            iteration_lock: Rc::new(()),
            empty_count: 0,
        }
    }

    /// Return the number of key-value pairs in the map, including empty values.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn full_len(&self) -> usize {
        self.map.len()
    }

    /// Gets the number of key-value pairs in the map, not including empty values.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len() - self.empty_count
    }

    /// Insert a key-value pair in the map.
    ///
    /// If an equivalent key already exists in the map: the key remains and
    /// retains its place in the order, and its corresponding value is updated.
    /// Otherwise the new key-value pair is appended.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub(super) fn insert(&mut self, key: JsValue, value: JsValue) {
        self.compact_if_unlocked();
        self.map
            .insert(MapKey::Key(key.into_stored()), Some(value.into_stored()));
    }

    /// Removes the entry for `key` and returns whether it existed.
    ///
    /// An active iterator keeps the entry's position as a tombstone.
    /// Otherwise, following entries shift left while keeping their order.
    ///
    /// Computes in **O(n)** time (average).
    pub(super) fn remove(&mut self, key: &JsValue) -> bool {
        let key = key.clone().into_stored();
        if self.is_locked() {
            if !self.map.contains_key(&key) {
                return false;
            }
            self.map.insert(MapKey::Empty(self.empty_count), None);
            self.empty_count += 1;
            self.map.swap_remove(&key).is_some()
        } else {
            self.compact_if_unlocked();
            self.map.shift_remove(&key).is_some()
        }
    }

    /// Removes all entries while preserving active iterator positions.
    pub fn clear(&mut self) {
        let len = self.map.len();
        self.map.clear();
        if self.is_locked() {
            self.map
                .extend((0..len).map(|index| (MapKey::Empty(index), None)));
            self.empty_count = len;
        } else {
            self.map.shrink_to_fit();
            self.empty_count = 0;
        }
    }

    /// Returns the value stored for `key`, or `None` if it is absent.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get(&self, key: &JsValue) -> Option<JsValue> {
        self.map
            .get(&key.clone().into_stored())
            .and_then(Option::as_ref)
            .map(StoredValue::to_value)
    }

    /// Get a key-value pair by index.
    ///
    /// Valid indices are `0 <= index < self.full_len()`.
    ///
    /// Computes in O(1) time.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<(JsValue, JsValue)> {
        if let (MapKey::Key(key), Some(value)) = self.map.get_index(index)? {
            Some((key.to_value(), value.to_value()))
        } else {
            None
        }
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter(&self) -> impl Iterator<Item = (JsValue, JsValue)> {
        self.map.iter().filter_map(|o| {
            if let (MapKey::Key(key), Some(value)) = o {
                Some((key.to_value(), value.to_value()))
            } else {
                None
            }
        })
    }

    /// Return `true` if an equivalent to `key` exists in the map.
    ///
    /// Computes in **O(1)** time (average).
    #[must_use]
    pub fn contains_key(&self, key: &JsValue) -> bool {
        self.map.contains_key(&key.clone().into_stored())
    }

    pub(crate) fn lock(&mut self) -> Rc<()> {
        self.compact_if_unlocked();
        Rc::clone(&self.iteration_lock)
    }

    fn is_locked(&self) -> bool {
        Rc::strong_count(&self.iteration_lock) > 1
    }

    fn compact_if_unlocked(&mut self) {
        if self.empty_count != 0 && !self.is_locked() {
            self.map.retain(|key, _| matches!(key, MapKey::Key(_)));
            self.empty_count = 0;
        }
    }
}

/// Rejects values whose object edge cannot be traced by the owning heap.
pub(crate) fn require_collection_value(runtime: &Runtime, value: &JsValue) -> JsResult<()> {
    if let Some(object) = value.as_object()
        && !runtime.same_heap(&object.runtime())
    {
        return Err(js_error!(TypeError: "collection value crosses JavaScript runtimes"));
    }
    Ok(())
}
