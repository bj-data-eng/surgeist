//! Implements a set type that preserves insertion order.

use crate::{JsData, JsValue, builtins::map::ordered_map::MapKey};
use indexmap::IndexSet;
use std::{fmt::Debug, rc::Rc};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace as HeapTrace};

/// A type wrapping `indexmap::IndexSet`
#[derive(Default, JsData)]
pub(crate) struct OrderedSet {
    inner: IndexSet<MapKey>,
    iteration_lock: Rc<()>,
    empty_count: usize,
}

impl HeapTrace for OrderedSet {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        for value in &self.inner {
            if let MapKey::Key(value) = value {
                HeapTrace::trace(value, edges)?;
            }
        }
        Ok(())
    }
}

impl Clone for OrderedSet {
    fn clone(&self) -> Self {
        let mut snapshot = Self {
            inner: self.inner.clone(),
            iteration_lock: Rc::new(()),
            empty_count: self.empty_count,
        };
        snapshot.compact_if_unlocked();
        snapshot
    }
}

impl Debug for OrderedSet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.inner.fmt(formatter)
    }
}

impl OrderedSet {
    /// Creates a new empty `OrderedSet`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new empty `OrderedSet` with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: IndexSet::with_capacity(capacity),
            iteration_lock: Rc::new(()),
            empty_count: 0,
        }
    }

    /// Return the number of elements in the set, including empty elements.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn full_len(&self) -> usize {
        self.inner.len()
    }

    /// Return the number of elements in the set.
    ///
    /// Computes in **O(1)** time.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len() - self.empty_count
    }

    /// Appends the value if the set does not already contain it.
    ///
    /// Computes in **O(1)** time (amortized average).
    pub(super) fn add(&mut self, value: JsValue) {
        self.compact_if_unlocked();
        self.inner.insert(MapKey::Key(value.into_stored()));
    }

    /// Delete the `value` from the set and return true if successful
    ///
    /// Return `false` if `value` is not in set.
    ///
    /// Computes in **O(n)** time (average).
    pub fn delete(&mut self, value: &JsValue) -> bool {
        let value = value.clone().into_stored();
        if self.is_locked() {
            if !self.inner.contains(&value) {
                return false;
            }
            self.inner.insert(MapKey::Empty(self.empty_count));
            self.empty_count += 1;
            self.inner.swap_remove(&value)
        } else {
            self.compact_if_unlocked();
            self.inner.shift_remove(&value)
        }
    }

    /// Removes all elements while preserving active iterator positions.
    pub fn clear(&mut self) {
        let len = self.inner.len();
        self.inner.clear();
        if self.is_locked() {
            self.inner.extend((0..len).map(MapKey::Empty));
            self.empty_count = len;
        } else {
            self.inner.shrink_to_fit();
            self.empty_count = 0;
        }
    }

    /// Checks if a given value is present in the set
    ///
    /// Return `true` if `value` is present in set, false otherwise.
    ///
    /// Computes in **O(n)** time (average).
    #[must_use]
    pub fn contains(&self, value: &JsValue) -> bool {
        self.inner.contains(&value.clone().into_stored())
    }

    /// Get a value by index.
    /// Valid indices are 0 <= `index` < `self.full_len()`.
    /// Computes in O(1) time.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<JsValue> {
        if let MapKey::Key(value) = self.inner.get_index(index)? {
            Some(value.to_value())
        } else {
            None
        }
    }

    /// Return an iterator over the values of the set, in their order
    pub fn iter(&self) -> impl Iterator<Item = JsValue> {
        self.inner.iter().filter_map(|v| {
            if let MapKey::Key(v) = v {
                Some(v.to_value())
            } else {
                None
            }
        })
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
            self.inner.retain(|key| matches!(key, MapKey::Key(_)));
            self.empty_count = 0;
        }
    }
}
