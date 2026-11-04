//! Keys met so far by a producer's duplicate check.
//!
//! A check that compared each fact with every earlier one took work quadratic
//! in the facts. The set is sized once by the number of facts, which the
//! producer's capacity check has already bounded, and tests each fact once.

use std::{collections::HashSet, hash::Hash};

pub(crate) struct Seen<K>(HashSet<K>);

impl<K: Hash + Eq> Seen<K> {
    /// Charges the fill of `facts` entries through `charge`, then allocates room
    /// for them; `exhausted` is the producer's refusal when allocation fails.
    pub(crate) fn new<E>(
        facts: usize,
        charge: impl FnOnce(usize) -> Result<(), E>,
        exhausted: E,
    ) -> Result<Self, E> {
        charge(facts)?;
        let mut keys = HashSet::new();
        keys.try_reserve(facts).map_err(|_| exhausted)?;
        Ok(Self(keys))
    }

    /// Whether `key` is new to the check.
    pub(crate) fn first(&mut self, key: K) -> bool {
        self.0.insert(key)
    }
}
