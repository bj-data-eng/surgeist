//! Small bounded lookup memoization retaining each actual resource key.

pub(crate) struct LruCache<K, V> {
    entries: Vec<(K, V)>,
    capacity: usize,
}

impl<K: PartialEq, V> LruCache<K, V> {
    pub(crate) fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a memoization cache needs a retained slot");
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    pub(crate) fn entry(&mut self, key: K, make: impl FnOnce() -> V) -> &V {
        let entry = if let Some(index) = self.entries.iter().position(|(stored, _)| stored == &key)
        {
            self.entries.remove(index)
        } else {
            (key, make())
        };
        if self.entries.len() == self.capacity {
            self.entries.pop();
        }
        self.entries.insert(0, entry);
        &self.entries[0].1
    }
}

#[cfg(test)]
mod tests {
    use super::LruCache;

    #[test]
    fn lookup_retains_real_keys_and_evicts_only_least_recent() {
        let mut cache = LruCache::new(2);
        assert_eq!(*cache.entry(String::from("first"), || 1), 1);
        assert_eq!(*cache.entry(String::from("second"), || 2), 2);
        assert_eq!(
            *cache.entry(String::from("first"), || panic!("retained")),
            1
        );
        assert_eq!(*cache.entry(String::from("third"), || 3), 3);
        assert_eq!(*cache.entry(String::from("first"), || panic!("recent")), 1);
        assert_eq!(*cache.entry(String::from("second"), || 4), 4);
    }
}
