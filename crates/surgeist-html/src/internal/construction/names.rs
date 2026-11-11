//! The interned names of one construction storage.
//!
//! A storage keeps one entry for every distinct element or attribute name it has been given: the
//! exact [`QualifiedName`], the [`ElementKey`] of that name, computed once when the entry is made,
//! and the keyed hash of the local name. An element record holds a handle on its entry, so two
//! elements of one name share one allocation and element-kind checks read the key.
//!
//! The table is append-only: an entry is never removed or changed, so a handle on it stays valid
//! for as long as any record holds it. Equality and hashing of [`QualifiedName`] compare the
//! components, so a name taken from one table equals the same name taken from another table or
//! made on its own.

use super::*;
use crate::internal::values::{
    Namespace,
    element_key::ElementKey,
    name_index::{
        HtmlNameIndex, HtmlNameIndexHasher, HtmlNameIndexSlot, HtmlNameIndexSlots,
        HtmlNameIndexTable,
    },
};

/// The probe steps a lookup or a placement in the index is charged before it runs, whatever the
/// process's hash key.
///
/// The index holds at most half its slots and probes linearly, so with uniform hashes the mean
/// number of slots a probe examines is `(1 + 1/(1 - a)^2) / 2` = 2.5 for a miss and
/// `(1 + 1/(1 - a)) / 2` = 1.5 for a hit at the greatest load `a` = 1/2. Eight is more than three
/// times the larger mean. A simulation of tables from 16 to 256 slots filled to that load found
/// more than eight slots examined, counting the empty slot that ends a miss, for at most 0.5% of
/// the hits and 3.4% of the misses, and a table is usually fuller than a quarter and emptier than
/// a half. A probe that examines more slots is charged one visit for each step beyond the
/// allowance, before the step. Two distinct names with an equal 64-bit hash are compared, and that
/// comparison is charged as one. The work of a lookup therefore varies with the key only for a
/// chain of more than eight slots or for such a pair of names.
pub(crate) const PROBE_ALLOWANCE: usize = 8;

/// One interned name.
#[derive(Eq, PartialEq)]
pub(crate) struct NameEntry {
    name: QualifiedName,
    key: ElementKey,
    hash: u64,
}
/// An entry debugs as its name: the key is a function of the name, and the hash depends on the
/// process.
impl fmt::Debug for NameEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.name, f)
    }
}
impl NameEntry {
    /// The entry of `name`, whose local name hashes to `hash`. The key is computed here.
    pub(crate) fn new(name: QualifiedName, hash: u64) -> Self {
        let key = ElementKey::from_name(&name);
        Self { name, key, hash }
    }
    /// The exact name.
    pub(crate) fn name(&self) -> &QualifiedName {
        &self.name
    }
    /// The key of the name, as [`ElementKey::from_name`] computes it.
    pub(crate) fn key(&self) -> ElementKey {
        self.key
    }
    /// The keyed hash of the local name.
    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }
    /// The bytes of an entry's allocation: the entry and the counts of its shared handle.
    pub(crate) const ALLOCATION_BYTES: usize = size_of::<Self>() + 2 * size_of::<usize>();
}

/// The components that identify a name when it is looked up.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NameKey<'a> {
    pub(crate) namespace: &'a Namespace,
    pub(crate) prefix: Option<&'a DocumentString>,
    pub(crate) local_name: &'a [u16],
}
impl<'a> NameKey<'a> {
    /// The key of an existing name.
    pub(crate) fn of(name: &'a QualifiedName) -> Self {
        Self {
            namespace: name.namespace(),
            prefix: name.prefix(),
            local_name: name.local_name().code_units(),
        }
    }
    /// The code units a comparison of this key with `entry` reads: a pair of equal length that
    /// does not share its allocation is read in full, and any other pair is told apart by its
    /// length or by the allocation it shares.
    pub(crate) fn comparison_units(&self, entry: &NameEntry) -> usize {
        fn pair(left: Option<&DocumentString>, right: Option<&DocumentString>) -> usize {
            match (left, right) {
                (Some(left), Some(right))
                    if left.len() == right.len() && !left.shares_storage_with(right) =>
                {
                    left.len()
                }
                _ => 0,
            }
        }
        let local = entry.name.local_name().code_units();
        let local = if local.len() == self.local_name.len() {
            local.len()
        } else {
            0
        };
        local
            .saturating_add(pair(self.namespace.uri(), entry.name.namespace().uri()))
            .saturating_add(pair(self.prefix, entry.name.prefix()))
    }
    /// Whether `entry` is the name of this key.
    pub(crate) fn is(&self, entry: &NameEntry) -> bool {
        entry.name.local_name().code_units() == self.local_name
            && entry.name.namespace().uri() == self.namespace.uri()
            && entry.name.prefix() == self.prefix
    }
}

/// The names of one storage and the table that finds them.
#[derive(Debug)]
pub(crate) struct NameTable {
    /// The entries in the order they were made.
    pub(in crate::internal::construction) entries: Vec<Arc<NameEntry>>,
    /// The slots of the index; empty until the first entry is made.
    pub(in crate::internal::construction) slots: Vec<HtmlNameIndexSlot>,
    pub(in crate::internal::construction) index: HtmlNameIndex,
    hasher: HtmlNameIndexHasher,
}
impl NameTable {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            slots: Vec::new(),
            index: HtmlNameIndex::new(HtmlNameIndexSlots::for_entries(0)),
            hasher: HtmlNameIndexHasher::new(),
        }
    }
    /// The keyed hash of a local name.
    pub(crate) fn hash(&self, local_name: &[u16]) -> u64 {
        self.hasher.hash(local_name)
    }
    /// The number of entries.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
    /// The entry at `position`, or `None` past the last.
    pub(crate) fn get(&self, position: usize) -> Option<&Arc<NameEntry>> {
        self.entries.get(position)
    }
    /// The slot geometry the index needs to hold one more entry, or `None` when it has room.
    /// A table that is full doubles the entries it holds, with room for at least eight.
    pub(crate) fn growth(&self) -> Option<HtmlNameIndexSlots> {
        let capacity = if self.slots.is_empty() {
            0
        } else {
            self.index.slots().capacity()
        };
        let needed = self.entries.len().checked_add(1)?;
        (needed > capacity)
            .then(|| HtmlNameIndexSlots::for_entries(capacity.saturating_mul(2).max(8).max(needed)))
    }
    /// The index lent its slots, or `None` when the table has no slots yet.
    pub(crate) fn lend(&mut self) -> Result<Option<HtmlNameIndexTable<'_>>, HtmlConstructionCause> {
        if self.slots.is_empty() {
            return Ok(None);
        }
        HtmlNameIndexTable::new(&mut self.index, &mut self.slots)
            .map(Some)
            .map_err(|_| HtmlConstructionCause::IndexUnavailable)
    }
}
