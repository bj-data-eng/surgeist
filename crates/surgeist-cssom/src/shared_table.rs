//! Independently shared indices and records for one immutable state capture.
//!
//! First mutation copies the touched HashMap's index (capacity, keys and Arc
//! handles), then only the changed record. Other indices and records remain
//! shared. A changed public owned data enum may still copy its own Vec payload;
//! this is deliberately not a persistent map or a general transaction overlay.
//!
//! Every mutation goes through [`SharedTable::insert`], [`SharedTable::insert_shared`],
//! [`SharedTable::remove`] or [`SharedTable::get_mut`], so a table knows which
//! records changed since the figures kept with its state last accounted for it:
//! the first time a key is touched after a settle, the table journals what the record
//! the key held then contributed ([`Journaled::before`]), or that it held none. The
//! journal keeps that contribution and not the record, so a record that is changed in
//! place stays unshared and is not copied. The figures of the state replace what each
//! journaled record contributed by what the key holds now, and take the journal
//! ([`SharedTable::take_journal`]). A table whose journal is empty holds exactly the
//! records the figures account for.
use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    ops::Index,
    sync::Arc,
};

/// A record that says what it contributes to the figures of its state, before it changes.
pub(crate) trait Journaled {
    /// What the record contributed, which the figures can take out again.
    type Before;
    /// What the record contributes now.
    fn before(&self) -> Self::Before;
}

pub(crate) struct SharedTable<K, V: Journaled> {
    index: Arc<HashMap<K, Arc<V>>>,
    /// The keys touched since the last settle, in the order they were first
    /// touched, with what the record each held then contributed.
    journal: Vec<(K, Option<V::Before>)>,
    touched: HashSet<K>,
}

impl<K: Clone, V: Journaled> Clone for SharedTable<K, V>
where
    V::Before: Clone,
{
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            journal: self.journal.clone(),
            touched: self.touched.clone(),
        }
    }
}

impl<K: fmt::Debug, V: Journaled + fmt::Debug> fmt::Debug for SharedTable<K, V>
where
    V::Before: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedTable")
            .field("index", &self.index)
            .field("journal", &self.journal)
            .finish_non_exhaustive()
    }
}

impl<K: Eq + Hash, V: Journaled> SharedTable<K, V> {
    pub(crate) fn new() -> Self {
        Self {
            index: Arc::new(HashMap::new()),
            journal: Vec::new(),
            touched: HashSet::new(),
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }
    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).map(Arc::as_ref)
    }
    pub(crate) fn get_shared(&self, key: &K) -> Option<&Arc<V>> {
        self.index.get(key)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.index.iter().map(|(key, value)| (key, value.as_ref()))
    }
    #[cfg(test)]
    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.index.values().map(Arc::as_ref)
    }
    /// Journals every key as one the table held no record for, in the order of `serial`.
    #[cfg(test)]
    pub(crate) fn journal_all(&mut self, serial: impl Fn(&K) -> u64)
    where
        K: Clone,
    {
        let mut keys = self.index.keys().cloned().collect::<Vec<_>>();
        keys.sort_by_key(|key| serial(key));
        self.touched = keys.iter().cloned().collect();
        self.journal = keys.into_iter().map(|key| (key, None)).collect();
    }
    /// Whether no key has been touched since the journal was last taken.
    pub(crate) fn is_settled(&self) -> bool {
        self.journal.is_empty()
    }
}
impl<K: Eq + Hash + Clone, V: Journaled> SharedTable<K, V> {
    /// Takes the journal: each key touched since it was last taken, in the order
    /// it was first touched, with what the record it held then contributed.
    pub(crate) fn take_journal(&mut self) -> Vec<(K, Option<V::Before>)> {
        // The set gives its storage back, so a state cloned later does not copy it.
        self.touched = HashSet::new();
        std::mem::take(&mut self.journal)
    }
    /// Journals `key` with `before` unless it has been touched since the journal was taken.
    fn touch(&mut self, key: &K, before: impl FnOnce() -> Option<V::Before>) {
        if !self.touched.contains(key) {
            self.touched.insert(key.clone());
            self.journal.push((key.clone(), before()));
        }
    }
}
impl<K: Eq + Hash + Clone, V: Journaled + Clone> SharedTable<K, V> {
    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        // Missing lookups do not detach an index. The caller admits semantic
        // changes before requesting mutable access to a present record.
        if !self.index.contains_key(key) {
            return None;
        }
        if !self.touched.contains(key) {
            let before = self.index.get(key).map(|record| record.before());
            self.touch(key, || before);
        }
        Arc::make_mut(&mut self.index)
            .get_mut(key)
            .map(Arc::make_mut)
    }
    pub(crate) fn insert(&mut self, key: K, value: V) {
        self.insert_shared(key, Arc::new(value));
    }
    pub(crate) fn insert_shared(&mut self, key: K, value: Arc<V>) {
        let replaced = Arc::make_mut(&mut self.index).insert(key.clone(), value);
        self.touch(&key, || replaced.map(|record| record.before()));
    }
}
impl<K: Eq + Hash + Clone, V: Journaled> SharedTable<K, V> {
    pub(crate) fn remove(&mut self, key: &K) {
        // Retained captures keep their original index and record. Removing a
        // current entry copies only the index; no retired payload clone escapes.
        if self.index.contains_key(key) {
            let removed = Arc::make_mut(&mut self.index).remove(key);
            self.touch(key, || removed.map(|record| record.before()));
        }
    }
}
impl<K: Eq + Hash, V: Journaled> Index<&K> for SharedTable<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &Self::Output {
        self.index[key].as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record whose contribution is its text, as the journal keeps it.
    impl Journaled for String {
        type Before = String;
        fn before(&self) -> String {
            self.clone()
        }
    }

    fn table() -> SharedTable<u32, String> {
        let mut table = SharedTable::new();
        table.insert(1, "one".to_owned());
        table.insert(2, "two".to_owned());
        table.take_journal();
        table
    }

    #[test]
    fn the_first_touch_of_a_key_journals_the_record_it_held_and_later_touches_do_not() {
        let mut table = table();
        assert!(table.is_settled());
        table.get_mut(&1).unwrap().push('!');
        table.get_mut(&1).unwrap().push('?');
        table.insert(1, "uno".to_owned());
        let journal = table.take_journal();
        assert_eq!(journal.len(), 1);
        assert_eq!(journal[0].0, 1);
        assert_eq!(journal[0].1.as_deref(), Some("one"));
        assert_eq!(table.get(&1).map(String::as_str), Some("uno"));
        assert!(table.is_settled());
    }

    #[test]
    fn a_key_added_or_removed_is_journaled_with_what_it_held() {
        let mut table = table();
        table.insert(3, "three".to_owned());
        table.remove(&2);
        // A key that is added and removed again leaves a journal entry of no record.
        table.insert(4, "four".to_owned());
        table.remove(&4);
        let journal = table.take_journal();
        let held = |key: u32| {
            journal
                .iter()
                .find(|(found, _)| *found == key)
                .map(|(_, before)| before.as_deref())
        };
        assert_eq!(held(3), Some(None));
        assert_eq!(held(2), Some(Some("two")));
        assert_eq!(held(4), Some(None));
        // The order is the order of the first touches.
        assert_eq!(
            journal.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            [3, 2, 4]
        );
    }

    #[test]
    fn a_record_that_is_changed_in_place_is_not_copied_for_the_journal() {
        let mut table = table();
        let record = Arc::as_ptr(table.get_shared(&1).unwrap());
        for round in 0..3 {
            table.get_mut(&1).unwrap().push('!');
            // Each round touches the key anew, as each settle of a batch does.
            assert_eq!(table.take_journal().len(), 1, "round {round}");
        }
        assert_eq!(Arc::as_ptr(table.get_shared(&1).unwrap()), record);
        // A record a capture still shares is copied, once.
        let capture = table.clone();
        table.get_mut(&1).unwrap().push('?');
        assert_ne!(Arc::as_ptr(table.get_shared(&1).unwrap()), record);
        assert_eq!(capture.get(&1).map(String::as_str), Some("one!!!"));
    }

    #[test]
    fn a_lookup_of_a_missing_key_and_a_removal_of_one_touch_nothing() {
        let mut table = table();
        assert!(table.get_mut(&9).is_none());
        table.remove(&9);
        assert!(table.is_settled());
    }

    #[test]
    fn a_clone_carries_the_journal_with_the_records() {
        let mut table = table();
        table.get_mut(&2).unwrap().push('!');
        let clone = table.clone();
        assert!(!clone.is_settled());
        assert_eq!(clone.get(&2).map(String::as_str), Some("two!"));
    }
}
