use std::collections::{HashMap, TryReserveError};

use crate::{LayoutCacheStoreEntryOf, LayoutScalar};

struct Store<Node, S: LayoutScalar> {
    order: usize,
    entry: LayoutCacheStoreEntryOf<Node, S>,
}

/// Ordered cache writes for ordinary computation and its disposable trials.
/// The index keeps replacement local to the node. Publication consumes the
/// retained last writes in their original order across nodes.
pub(super) struct CacheStores<Node, S: LayoutScalar> {
    by_node: HashMap<usize, Vec<Store<Node, S>>>,
    next: usize,
}

impl<Node: Copy, S: LayoutScalar> CacheStores<Node, S> {
    pub(super) fn new() -> Self {
        Self {
            by_node: HashMap::new(),
            next: 0,
        }
    }

    pub(super) fn store(
        &mut self,
        node_index: usize,
        entry: LayoutCacheStoreEntryOf<Node, S>,
    ) -> Result<(), TryReserveError> {
        if !self.by_node.contains_key(&node_index) {
            self.by_node.try_reserve(1)?;
        }
        let stores = self.by_node.entry(node_index).or_default();
        let order = self.next;
        let next = order.checked_add(1).expect("cache store order exhausted");
        if let Some(previous) = stores.iter_mut().find(|previous| {
            previous.entry.input() == entry.input() && previous.entry.context() == entry.context()
        }) {
            *previous = Store { order, entry };
        } else {
            stores.try_reserve(1)?;
            stores.push(Store { order, entry });
        }
        self.next = next;
        Ok(())
    }

    pub(super) fn remove(&mut self, node_index: usize) {
        self.by_node.remove(&node_index);
    }

    pub(super) fn into_ordered(self) -> Vec<LayoutCacheStoreEntryOf<Node, S>> {
        let mut stores: Vec<_> = self.by_node.values().flatten().collect();
        stores.sort_unstable_by_key(|store| store.order);
        stores.into_iter().map(|store| store.entry).collect()
    }
}
