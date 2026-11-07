//! Fallible compiler-owned indexing. Keys own their admitted hash and equality;
//! the index owns slot probes, storage and staged publication.

use super::control::{CompilerControl, CompilerWork, payload_bytes, reserve, size_overflow};
use crate::JsResult;
use std::mem::size_of;

pub(super) trait CompilerKey {
    /// Equal keys must produce equal hashes. Both operations preserve every
    /// semantic key field and return immediately on the first refused request.
    fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64>;
    fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool>;
}

#[derive(Debug)]
struct Entry<K, V> {
    hash: u64,
    key: K,
    value: V,
}

/// No deletion means an empty slot always terminates a search. Entries retain
/// stable indices when a replacement slot array is prepared. Replacing a value
/// retains the original key, like the compiler's former map insertion.
#[derive(Debug)]
pub(super) struct CompilerIndex<K, V> {
    entries: Vec<Entry<K, V>>,
    slots: Vec<Option<usize>>,
}

impl<K, V> Default for CompilerIndex<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            slots: Vec::new(),
        }
    }
}

fn home(hash: u64, slots: usize) -> usize {
    hash as usize & (slots - 1)
}

/// Find a vacant slot while preparing replacement metadata. No key hashing or
/// equality occurs: the caller already established that the key is absent.
fn vacant(
    slots: &[Option<usize>],
    hash: u64,
    control: &mut CompilerControl<'_>,
) -> JsResult<usize> {
    let mut slot = home(hash, slots.len());
    loop {
        control.admit(CompilerWork::IndexProbe)?;
        if slots[slot].is_none() {
            return Ok(slot);
        }
        slot = (slot + 1) & (slots.len() - 1);
    }
}

enum Search {
    Found(usize),
    Vacant(usize),
}

impl<K: CompilerKey, V> CompilerIndex<K, V> {
    fn search(&self, key: &K, hash: u64, control: &mut CompilerControl<'_>) -> JsResult<Search> {
        if self.slots.is_empty() {
            return Ok(Search::Vacant(0));
        }
        let mut slot = home(hash, self.slots.len());
        loop {
            control.admit(CompilerWork::IndexProbe)?;
            let Some(index) = self.slots[slot] else {
                return Ok(Search::Vacant(slot));
            };
            control.admit(CompilerWork::Read {
                bytes: size_of::<u64>(),
            })?;
            let entry = &self.entries[index];
            if entry.hash == hash && entry.key.try_eq(key, control)? {
                return Ok(Search::Found(index));
            }
            slot = (slot + 1) & (self.slots.len() - 1);
        }
    }

    pub(super) fn get(&self, key: &K, control: &mut CompilerControl<'_>) -> JsResult<Option<&V>> {
        let hash = key.try_hash(control)?;
        Ok(match self.search(key, hash, control)? {
            Search::Found(index) => Some(&self.entries[index].value),
            Search::Vacant(_) => None,
        })
    }

    pub(super) fn contains_key(
        &self,
        key: &K,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<bool> {
        self.get(key, control).map(|value| value.is_some())
    }

    pub(super) fn insert(
        &mut self,
        key: K,
        value: V,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<Option<V>> {
        let hash = key.try_hash(control)?;
        let vacant_slot = match self.search(&key, hash, control)? {
            Search::Found(index) => {
                control.admit(CompilerWork::IndexWrite)?;
                control.admit(CompilerWork::Copy {
                    bytes: size_of::<V>(),
                })?;
                return Ok(Some(std::mem::replace(
                    &mut self.entries[index].value,
                    value,
                )));
            }
            Search::Vacant(slot) => slot,
        };
        let length = self
            .entries
            .len()
            .checked_add(1)
            .ok_or_else(size_overflow)?;
        // Keep at least one quarter of the slots vacant, including at least one
        // terminating slot. All slot lengths are powers of two and start at 8.
        let grow = self.slots.is_empty() || length > self.slots.len() - self.slots.len() / 4;
        let mut replacement = if grow {
            let count = if self.slots.is_empty() {
                8
            } else {
                self.slots.len().checked_mul(2).ok_or_else(size_overflow)?
            };
            let mut slots = Vec::new();
            reserve(&mut slots, count, control)?;
            while slots.len() < count {
                let chunk = (count - slots.len()).min(256);
                control.admit(CompilerWork::Copy {
                    bytes: payload_bytes::<Option<usize>>(chunk)?,
                })?;
                slots.resize(slots.len() + chunk, None);
            }
            for (index, entry) in self.entries.iter().enumerate() {
                control.admit(CompilerWork::Read {
                    bytes: size_of::<u64>(),
                })?;
                let slot = vacant(&slots, entry.hash, control)?;
                control.admit(CompilerWork::IndexWrite)?;
                slots[slot] = Some(index);
            }
            Some(slots)
        } else {
            None
        };
        let slot = match &replacement {
            Some(slots) => vacant(slots, hash, control)?,
            None => vacant_slot,
        };
        if length > self.entries.capacity() {
            let capacity = self
                .entries
                .capacity()
                .checked_mul(2)
                .ok_or_else(size_overflow)?
                .max(8)
                .max(length);
            let additional = capacity - self.entries.len();
            reserve(&mut self.entries, additional, control)?;
        }
        control.admit(CompilerWork::IndexWrite)?;
        control.admit(CompilerWork::Copy {
            bytes: size_of::<Entry<K, V>>(),
        })?;
        control.admit(CompilerWork::IndexWrite)?;
        // All rejection and allocation has finished. Only move the entry and
        // replace its already prepared slot metadata from this point onward.
        let index = self.entries.len();
        self.entries.push(Entry { hash, key, value });
        if let Some(slots) = &mut replacement {
            slots[slot] = Some(index);
        } else {
            self.slots[slot] = Some(index);
        }
        if let Some(slots) = replacement {
            self.slots = slots;
        }
        Ok(None)
    }

    /// Inspect retained values without an unadmitted collection or hidden
    /// iterator scan. The callback shares this compilation's admission.
    pub(super) fn visit_values(
        &self,
        control: &mut CompilerControl<'_>,
        mut visit: impl FnMut(&V, &mut CompilerControl<'_>) -> JsResult<()>,
    ) -> JsResult<()> {
        for entry in &self.entries {
            control.admit(CompilerWork::IndexProbe)?;
            control.admit(CompilerWork::Read {
                bytes: size_of::<V>(),
            })?;
            visit(&entry.value, control)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecompiler::control::CompilerDepth;

    #[derive(Clone, Debug)]
    struct Key {
        bytes: Vec<u8>,
        hash: u64,
    }

    impl Key {
        fn new(value: u8) -> Self {
            Self {
                bytes: vec![value],
                hash: 0,
            }
        }
    }

    impl CompilerKey for Key {
        fn try_hash(&self, control: &mut CompilerControl<'_>) -> JsResult<u64> {
            control.admit(CompilerWork::Read {
                bytes: self.bytes.len(),
            })?;
            Ok(self.hash)
        }

        fn try_eq(&self, other: &Self, control: &mut CompilerControl<'_>) -> JsResult<bool> {
            if self.bytes.len() != other.bytes.len() {
                return Ok(false);
            }
            for (left, right) in self.bytes.iter().zip(&other.bytes) {
                control.admit(CompilerWork::Read { bytes: 2 })?;
                if left != right {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }

    fn baseline(count: u8) -> CompilerIndex<Key, u32> {
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        let mut index = CompilerIndex::default();
        for key in 0..count {
            assert_eq!(
                index
                    .insert(Key::new(key), u32::from(key) + 10, &mut control)
                    .unwrap(),
                None
            );
        }
        index
    }

    #[test]
    fn colliding_keys_survive_growth_and_replace_only_the_equal_value() {
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        let mut index = baseline(96);
        assert_eq!(
            index.insert(Key::new(47), 999, &mut control).unwrap(),
            Some(57)
        );
        for key in 0..96 {
            assert_eq!(
                index.get(&Key::new(key), &mut control).unwrap().copied(),
                Some(if key == 47 { 999 } else { u32::from(key) + 10 })
            );
        }
        assert!(!index.contains_key(&Key::new(255), &mut control).unwrap());
        let mut values = Vec::new();
        index
            .visit_values(&mut control, |value, _| {
                values.push(*value);
                Ok(())
            })
            .unwrap();
        assert_eq!(values.len(), 96);
        assert_eq!(values.iter().filter(|&&value| value == 999).count(), 1);
    }

    #[test]
    fn probes_include_colliding_candidates_and_the_terminating_vacancy() {
        let index = baseline(2);
        let depth = CompilerDepth::new();
        for (key, expected, expected_probes) in [(1, Some(11), 2), (2, None, 3)] {
            let mut probes = 0;
            let mut admit = |work| {
                if matches!(work, CompilerWork::IndexProbe) {
                    probes += 1;
                }
                Ok(())
            };
            let result = index.get(
                &Key::new(key),
                &mut CompilerControl::new(Some(&mut admit), &depth),
            );
            assert_eq!(result.unwrap().copied(), expected);
            assert_eq!(probes, expected_probes);
        }
    }

    #[test]
    fn every_growth_and_replacement_refusal_preserves_prior_mappings() {
        for key in [6, 3] {
            let mut completed = baseline(6);
            let depth = CompilerDepth::new();
            let mut count = 0;
            let mut count_work = |_| {
                count += 1;
                Ok(())
            };
            completed
                .insert(
                    Key::new(key),
                    999,
                    &mut CompilerControl::new(Some(&mut count_work), &depth),
                )
                .unwrap();
            for limit in 0..count {
                let mut index = baseline(6);
                let mut accepted = 0;
                let mut refused = false;
                let mut admit = |_| {
                    assert!(!refused, "work continued after the first refusal");
                    if accepted == limit {
                        refused = true;
                        Err(size_overflow())
                    } else {
                        accepted += 1;
                        Ok(())
                    }
                };
                let error = index
                    .insert(
                        Key::new(key),
                        999,
                        &mut CompilerControl::new(Some(&mut admit), &depth),
                    )
                    .unwrap_err();
                assert_eq!(error, size_overflow());
                assert_eq!(accepted, limit);
                let mut control = CompilerControl::new(None, &depth);
                for original in 0..6 {
                    assert_eq!(
                        index
                            .get(&Key::new(original), &mut control)
                            .unwrap()
                            .copied(),
                        Some(u32::from(original) + 10)
                    );
                }
                assert!(index.get(&Key::new(6), &mut control).unwrap().is_none());
                assert_eq!(
                    index.insert(Key::new(key), 999, &mut control).unwrap(),
                    if key == 3 { Some(13) } else { None }
                );
                assert_eq!(index.get(&Key::new(key), &mut control).unwrap(), Some(&999));
            }
        }
    }

    #[test]
    fn large_slot_initialization_can_stop_between_bounded_chunks() {
        let mut index = baseline(192);
        let depth = CompilerDepth::new();
        let chunk_bytes = 256 * size_of::<Option<usize>>();
        let mut chunks = 0;
        let mut refused = false;
        let mut admit = |work| {
            assert!(!refused, "work continued after slot initialization refusal");
            if let CompilerWork::Copy { bytes } = work {
                assert!(bytes <= chunk_bytes);
                if bytes == chunk_bytes {
                    chunks += 1;
                    if chunks == 2 {
                        refused = true;
                        return Err(size_overflow());
                    }
                }
            }
            Ok(())
        };
        let error = index
            .insert(
                Key::new(192),
                999,
                &mut CompilerControl::new(Some(&mut admit), &depth),
            )
            .unwrap_err();
        assert_eq!(error, size_overflow());
        assert_eq!(chunks, 2);
        let mut control = CompilerControl::new(None, &depth);
        for key in 0..192 {
            assert_eq!(
                index.get(&Key::new(key), &mut control).unwrap().copied(),
                Some(u32::from(key) + 10)
            );
        }
        assert!(index.get(&Key::new(192), &mut control).unwrap().is_none());
        assert_eq!(
            index.insert(Key::new(192), 999, &mut control).unwrap(),
            None
        );
        assert_eq!(index.get(&Key::new(192), &mut control).unwrap(), Some(&999));
    }

    #[test]
    fn long_key_hash_and_equality_requests_reach_the_shared_control() {
        let key = Key {
            bytes: vec![7; 4096],
            hash: 9,
        };
        let depth = CompilerDepth::new();
        let mut index = CompilerIndex::default();
        index
            .insert(key.clone(), 42, &mut CompilerControl::new(None, &depth))
            .unwrap();
        let mut reads = 0;
        let mut admit = |work| {
            if let CompilerWork::Read { bytes } = work {
                reads += bytes;
            }
            Ok(())
        };
        assert_eq!(
            index
                .get(&key, &mut CompilerControl::new(Some(&mut admit), &depth))
                .unwrap(),
            Some(&42)
        );
        // One complete hash input, one cached hash and all equal unit pairs.
        assert_eq!(reads, 4096 + size_of::<u64>() + 4096 * 2);
        let mut stopped = false;
        let mut refuse_hash = |_| {
            assert!(!stopped);
            stopped = true;
            Err(size_overflow())
        };
        assert_eq!(
            index
                .get(
                    &key,
                    &mut CompilerControl::new(Some(&mut refuse_hash), &depth)
                )
                .unwrap_err(),
            size_overflow()
        );
    }

    #[test]
    fn value_visitor_stops_before_the_next_entry_after_callback_refusal() {
        let index = baseline(5);
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        let mut visited = Vec::new();
        let result = index.visit_values(&mut control, |value, _| {
            visited.push(*value);
            if visited.len() == 3 {
                Err(size_overflow())
            } else {
                Ok(())
            }
        });
        assert_eq!(result.unwrap_err(), size_overflow());
        assert_eq!(visited, [10, 11, 12]);
    }
}
