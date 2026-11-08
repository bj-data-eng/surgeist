//! Constructor-owned logical work and fallible ordered indexing.

#[cfg(test)]
mod tests;

use super::{ImportAttribute, ModuleRequest};
use crate::{Context, JsError, JsResult, JsString, error::EngineResourceError};
use std::{
    hash::{Hash, Hasher},
    mem::size_of,
    path::{Path, PathBuf},
    rc::Rc,
};
use surgeist_js_ast::{declaration::ImportAttribute as AstAttribute, operations::NameWork};
use surgeist_js_gc::heap::{AccessError, EdgeVisitor, Trace};
use surgeist_js_interner::{Interner, Sym};
use surgeist_js_string::{StringError, StringWork};

#[derive(Clone, Copy)]
pub(crate) enum Work {
    Visit,
    Probe,
    Record,
    Write,
    Capacity(usize),
    Copy(usize),
    Relocate(usize),
    Publication,
    String(StringWork),
    Name(NameWork),
    Checkpoint,
}
pub(crate) const ORIGINS: [&str; 14] = [
    "metadata-visit",
    "metadata-probe",
    "metadata-record",
    "metadata-write",
    "metadata-capacity",
    "metadata-copy",
    "metadata-relocate",
    "metadata-publication",
    "metadata-string-scan",
    "metadata-string-copy",
    "metadata-string-allocate",
    "metadata-name-visit",
    "metadata-name-record",
    "metadata-name-comparison",
];
impl Work {
    pub(crate) fn origin_units(self) -> Option<(usize, u64)> {
        Some(match self {
            Self::Visit => (0, 1),
            Self::Probe => (1, 1),
            Self::Record => (2, 1),
            Self::Write => (3, 1),
            Self::Capacity(n) => (4, n as u64),
            Self::Copy(n) => (5, n as u64),
            Self::Relocate(n) => (6, n as u64),
            Self::Publication => (7, 1),
            Self::String(StringWork::Scan { units }) => (8, units as u64),
            Self::String(StringWork::Copy { units }) => (9, units as u64),
            Self::String(StringWork::Allocate { bytes }) => (10, bytes as u64),
            Self::Name(NameWork::Visit) => (11, 1),
            Self::Name(NameWork::Record) => (12, 1),
            Self::Name(NameWork::Comparison) => (13, 1),
            Self::Checkpoint | Self::String(StringWork::Checkpoint) => return None,
            Self::String(StringWork::FastFloat { .. }) => {
                unreachable!("metadata does not parse numbers")
            }
        })
    }
}

pub(super) fn bytes<T>(count: usize) -> JsResult<usize> {
    count
        .checked_mul(size_of::<T>())
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or_else(|| EngineResourceError::SizeOverflow.into())
}
fn string_error(error: StringError<JsError>) -> JsError {
    match error {
        StringError::Refused(error) => error,
        StringError::SizeOverflow => EngineResourceError::SizeOverflow.into(),
        StringError::Allocation(_) => EngineResourceError::Allocation.into(),
    }
}

/// Logical reservations are independent of allocator over-allocation.
#[derive(Debug)]
pub(super) struct Buffer<T> {
    values: Vec<T>,
    logical_capacity: usize,
}
impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            logical_capacity: 0,
        }
    }
}
impl<T> Buffer<T> {
    pub(super) fn into_vec(self) -> Vec<T> {
        self.values
    }
}
impl<T> std::ops::Deref for Buffer<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T> std::ops::DerefMut for Buffer<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}
impl<T: Trace> Trace for Buffer<T> {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.values.trace(edges)
    }
}

/// Only a temporary Context borrow; no operation state escapes construction.
pub(super) struct Control<'a> {
    context: &'a mut Context,
}
impl<'a> Control<'a> {
    pub(super) fn new(context: &'a mut Context) -> Self {
        Self { context }
    }
    pub(super) fn context(&self) -> &Context {
        self.context
    }
    pub(super) fn admit(&mut self, work: Work) -> JsResult<()> {
        self.context.admit_module_metadata(work)
    }
    pub(super) fn header<T>(&mut self) -> JsResult<()> {
        self.admit(Work::Record)?;
        self.admit(Work::Capacity(size_of::<T>()))
    }
    /// Known extents reserve exactly their requested logical capacity.
    pub(super) fn reserve_exact<T>(
        &mut self,
        values: &mut Buffer<T>,
        additional: usize,
    ) -> JsResult<()> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or(EngineResourceError::SizeOverflow)?;
        if needed > values.logical_capacity {
            self.reserve_to(values, needed)?;
        }
        Ok(())
    }
    /// Unknown append streams use deterministic amortized growth, independent
    /// of allocator over-allocation. Index entries share this append policy.
    pub(super) fn reserve<T>(&mut self, values: &mut Buffer<T>, additional: usize) -> JsResult<()> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or(EngineResourceError::SizeOverflow)?;
        if needed > values.logical_capacity {
            let doubled = values
                .logical_capacity
                .checked_mul(2)
                .ok_or(EngineResourceError::SizeOverflow)?;
            self.reserve_to(values, needed.max(4).max(doubled))?;
        }
        Ok(())
    }
    fn reserve_to<T>(&mut self, values: &mut Buffer<T>, target: usize) -> JsResult<()> {
        bytes::<T>(target)?;
        self.admit(Work::Capacity(bytes::<T>(
            target - values.logical_capacity,
        )?))?;
        // Vec may relocate all initialized values. This is conservative
        // logical prepayment, independent of allocator movement/capacity.
        self.admit(Work::Relocate(bytes::<T>(values.len())?))?;
        values
            .values
            .try_reserve_exact(target - values.len())
            .map_err(|_| EngineResourceError::Allocation)?;
        values.logical_capacity = target;
        Ok(())
    }
    pub(super) fn push<T>(&mut self, values: &mut Buffer<T>, value: T) -> JsResult<()> {
        self.reserve(values, 1)?;
        self.admit(Work::Record)?;
        self.admit(Work::Write)?;
        values.values.push(value);
        Ok(())
    }
    /// A fresh empty slot's Record credit includes its initialization. Later
    /// index mappings receive Write credits separately.
    pub(super) fn initialize<T>(&mut self, values: &mut Buffer<T>, value: T) -> JsResult<()> {
        self.reserve(values, 1)?;
        self.admit(Work::Record)?;
        values.values.push(value);
        Ok(())
    }
    pub(super) fn copy<T: Clone>(&mut self, value: &T) -> JsResult<T> {
        self.admit(Work::Copy(size_of::<T>()))?;
        Ok(value.clone())
    }
    pub(super) fn path(&mut self, path: Option<&Path>) -> JsResult<Option<PathBuf>> {
        path.map(|path| {
            let bytes = path.as_os_str().len();
            self.admit(Work::Capacity(bytes))?;
            self.admit(Work::Copy(bytes))?;
            Ok(path.to_path_buf())
        })
        .transpose()
    }
    pub(super) fn string(&mut self, symbol: Sym, interner: &Interner) -> JsResult<JsString> {
        let units = interner.resolve_expect(symbol).utf16();
        JsString::try_from_utf16(units, &mut |work| self.admit(Work::String(work)))
            .map_err(string_error)
    }
    pub(super) fn request(
        &mut self,
        specifier: Sym,
        attributes: &[AstAttribute],
        interner: &Interner,
    ) -> JsResult<ModuleRequest> {
        let mut owned = Buffer::default();
        self.reserve_exact(&mut owned, attributes.len())?;
        for attribute in attributes {
            self.admit(Work::Visit)?;
            let key = self.string(attribute.key(), interner)?;
            let value = self.string(attribute.value(), interner)?;
            self.push(&mut owned, ImportAttribute::new(key, value))?;
        }
        sort_attributes(&mut owned, self)?;
        let specifier = self.string(specifier, interner)?;
        self.header::<Vec<ImportAttribute>>()?;
        self.admit(Work::Record)?;
        Ok(ModuleRequest {
            specifier,
            attributes: Rc::new(owned.into_vec()),
        })
    }
}

fn sort_attributes(values: &mut [ImportAttribute], control: &mut Control<'_>) -> JsResult<()> {
    fn sift(
        values: &mut [ImportAttribute],
        mut root: usize,
        end: usize,
        control: &mut Control<'_>,
    ) -> JsResult<()> {
        loop {
            let Some(left) = root
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n < end)
            else {
                return Ok(());
            };
            let mut child = left;
            if left + 1 < end {
                control.admit(Work::Probe)?;
                if values[left]
                    .key
                    .as_str()
                    .try_cmp(values[left + 1].key.as_str(), &mut |w| {
                        control.admit(Work::String(w))
                    })
                    .map_err(string_error)?
                    .is_lt()
                {
                    child += 1;
                }
            }
            control.admit(Work::Probe)?;
            if !values[root]
                .key
                .as_str()
                .try_cmp(values[child].key.as_str(), &mut |w| {
                    control.admit(Work::String(w))
                })
                .map_err(string_error)?
                .is_lt()
            {
                return Ok(());
            }
            control.admit(Work::Write)?;
            values.swap(root, child);
            root = child;
        }
    }
    for root in (0..values.len() / 2).rev() {
        sift(values, root, values.len(), control)?;
    }
    for end in (1..values.len()).rev() {
        control.admit(Work::Write)?;
        values.swap(0, end);
        sift(values, 0, end, control)?;
    }
    Ok(())
}

pub(super) trait Key: Hash + Eq {
    fn hash_admitted(&self, control: &mut Control<'_>) -> JsResult<u64>;
    fn eq_admitted(&self, other: &Self, control: &mut Control<'_>) -> JsResult<bool>;
}
impl Key for JsString {
    fn hash_admitted(&self, control: &mut Control<'_>) -> JsResult<u64> {
        let mut hash = rustc_hash::FxHasher::default();
        self.as_str()
            .try_hash(&mut hash, &mut |w| control.admit(Work::String(w)))
            .map_err(string_error)?;
        Ok(hash.finish())
    }
    fn eq_admitted(&self, other: &Self, control: &mut Control<'_>) -> JsResult<bool> {
        self.as_str()
            .try_eq(other.as_str(), &mut |w| control.admit(Work::String(w)))
            .map_err(string_error)
    }
}
impl Key for ModuleRequest {
    fn hash_admitted(&self, control: &mut Control<'_>) -> JsResult<u64> {
        let mut hash = rustc_hash::FxHasher::default();
        self.specifier
            .as_str()
            .try_hash(&mut hash, &mut |w| control.admit(Work::String(w)))
            .map_err(string_error)?;
        control.admit(Work::Probe)?;
        hash.write_usize(self.attributes.len());
        for attribute in self.attributes.iter() {
            control.admit(Work::Visit)?;
            for string in [&attribute.key, &attribute.value] {
                string
                    .as_str()
                    .try_hash(&mut hash, &mut |w| control.admit(Work::String(w)))
                    .map_err(string_error)?;
            }
        }
        Ok(hash.finish())
    }
    fn eq_admitted(&self, other: &Self, control: &mut Control<'_>) -> JsResult<bool> {
        if !self.specifier.eq_admitted(&other.specifier, control)? {
            return Ok(false);
        }
        control.admit(Work::Probe)?;
        if self.attributes.len() != other.attributes.len() {
            return Ok(false);
        }
        for (left, right) in self.attributes.iter().zip(other.attributes.iter()) {
            control.admit(Work::Visit)?;
            if !left.key.eq_admitted(&right.key, control)?
                || !left.value.eq_admitted(&right.value, control)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Debug, Trace)]
pub(super) struct Entry<K, V> {
    hash: u64,
    key: K,
    value: V,
}
#[derive(Debug)]
pub(super) struct Index<K, V = ()> {
    entries: Buffer<Entry<K, V>>,
    slots: Buffer<Option<usize>>,
}
impl<K: Trace, V: Trace> Trace for Index<K, V> {
    fn trace(&self, edges: &mut EdgeVisitor<'_>) -> Result<(), AccessError> {
        self.entries.trace(edges)
    }
}
impl<K, V> Default for Index<K, V> {
    fn default() -> Self {
        Self {
            entries: Buffer::default(),
            slots: Buffer::default(),
        }
    }
}
impl<K, V> Index<K, V> {
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(super) fn iter(&self) -> impl ExactSizeIterator<Item = &K> {
        self.entries.iter().map(|entry| &entry.key)
    }
}
impl<'a, K, V> IntoIterator for &'a Index<K, V> {
    type Item = &'a K;
    type IntoIter = std::iter::Map<std::slice::Iter<'a, Entry<K, V>>, fn(&'a Entry<K, V>) -> &'a K>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().map(|entry| &entry.key)
    }
}
impl<K: Key, V> Index<K, V> {
    fn search(
        &self,
        key: &K,
        hash: u64,
        control: &mut Control<'_>,
    ) -> JsResult<Result<usize, usize>> {
        if self.slots.is_empty() {
            return Ok(Err(0));
        }
        let mut slot = hash as usize & (self.slots.len() - 1);
        loop {
            control.admit(Work::Probe)?;
            let Some(index) = self.slots[slot] else {
                return Ok(Err(slot));
            };
            control.admit(Work::Probe)?;
            let entry = &self.entries[index];
            if entry.hash == hash && entry.key.eq_admitted(key, control)? {
                return Ok(Ok(index));
            }
            slot = (slot + 1) & (self.slots.len() - 1);
        }
    }
    pub(super) fn get(&self, key: &K, control: &mut Control<'_>) -> JsResult<Option<&V>> {
        let hash = key.hash_admitted(control)?;
        Ok(self
            .search(key, hash, control)?
            .ok()
            .map(|index| &self.entries[index].value))
    }
    pub(super) fn insert(&mut self, key: K, value: V, control: &mut Control<'_>) -> JsResult<()> {
        control.admit(Work::Record)?; // Includes duplicate insertion attempts.
        let hash = key.hash_admitted(control)?;
        if self.search(&key, hash, control)?.is_ok() {
            return Ok(());
        }
        let needed = self
            .entries
            .len()
            .checked_add(1)
            .ok_or(EngineResourceError::SizeOverflow)?;
        if needed > self.slots.len() / 2 {
            let count = self
                .slots
                .len()
                .max(4)
                .checked_mul(2)
                .ok_or(EngineResourceError::SizeOverflow)?;
            let mut slots = Buffer::default();
            control.reserve_exact(&mut slots, count)?;
            for _ in 0..count {
                control.initialize(&mut slots, None)?;
            }
            for (index, entry) in self.entries.iter().enumerate() {
                let mut slot = entry.hash as usize & (count - 1);
                loop {
                    control.admit(Work::Probe)?;
                    if slots[slot].is_none() {
                        break;
                    }
                    slot = (slot + 1) & (count - 1);
                }
                control.admit(Work::Write)?;
                slots[slot] = Some(index);
            }
            self.slots = slots;
        }
        control.reserve(&mut self.entries, 1)?;
        let slot = self
            .search(&key, hash, control)?
            .expect_err("key remains absent");
        control.admit(Work::Write)?;
        control.admit(Work::Write)?;
        self.slots[slot] = Some(self.entries.len());
        self.entries.values.push(Entry { hash, key, value });
        Ok(())
    }
    /// Later lifecycle callers retain their existing ordinary lookup boundary.
    pub(super) fn contains(&self, key: &K) -> bool {
        if self.slots.is_empty() {
            return false;
        }
        let mut hash = rustc_hash::FxHasher::default();
        key.hash(&mut hash);
        let hash = hash.finish();
        let mut slot = hash as usize & (self.slots.len() - 1);
        loop {
            let Some(index) = self.slots[slot] else {
                return false;
            };
            let entry = &self.entries[index];
            if entry.hash == hash && entry.key == *key {
                return true;
            }
            slot = (slot + 1) & (self.slots.len() - 1);
        }
    }
}

#[cfg(test)]
mod cost_tests;
