//! Independently shared indices and records for one immutable state capture.
//!
//! First mutation copies the touched HashMap's index (capacity, keys and Arc
//! handles), then only the changed record. Other indices and records remain
//! shared. A changed public owned data enum may still copy its own Vec payload;
//! this is deliberately not a persistent map or a general transaction overlay.
use std::{collections::HashMap, hash::Hash, ops::Index, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct SharedTable<K, V>(Arc<HashMap<K, Arc<V>>>);

impl<K: Eq + Hash, V> SharedTable<K, V> {
    pub(crate) fn new() -> Self {
        Self(Arc::new(HashMap::new()))
    }
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key).map(Arc::as_ref)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(key, value)| (key, value.as_ref()))
    }
    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.0.values().map(Arc::as_ref)
    }
}
impl<K: Eq + Hash + Clone, V: Clone> SharedTable<K, V> {
    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        // Missing lookups do not detach an index. The caller admits semantic
        // changes before requesting mutable access to a present record.
        if !self.0.contains_key(key) {
            return None;
        }
        Arc::make_mut(&mut self.0).get_mut(key).map(Arc::make_mut)
    }
    pub(crate) fn insert(&mut self, key: K, value: V) {
        Arc::make_mut(&mut self.0).insert(key, Arc::new(value));
    }
}
impl<K: Eq + Hash + Clone, V> SharedTable<K, V> {
    pub(crate) fn remove(&mut self, key: &K) {
        // Retained captures keep their original index and record. Removing a
        // current entry copies only the index; no retired payload clone escapes.
        if self.0.contains_key(key) {
            Arc::make_mut(&mut self.0).remove(key);
        }
    }
}
impl<K: Eq + Hash, V> Index<&K> for SharedTable<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.0[key].as_ref()
    }
}
