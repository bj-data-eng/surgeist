//! Keys met so far by a producer's duplicate check.
//!
//! A check that compared each fact with every earlier one took work quadratic
//! in the facts. The set is sized once by the number of facts, which the
//! producer's capacity check has already bounded, and tests each fact once.

use std::{
    collections::{HashSet, TryReserveError},
    hash::Hash,
};

pub(crate) struct Seen<K>(HashSet<K>);

impl<K: Hash + Eq> Seen<K> {
    pub(crate) fn new(facts: usize) -> Result<Self, TryReserveError> {
        let mut keys = HashSet::new();
        keys.try_reserve(facts)?;
        Ok(Self(keys))
    }

    /// Whether `key` is new to the check.
    pub(crate) fn first(&mut self, key: K) -> bool {
        self.0.insert(key)
    }
}
