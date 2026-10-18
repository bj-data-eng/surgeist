//! Exact qualified controller projections. Keys use bounded native chunks;
//! held leaves stay installed but inactive; permanent retirement prunes empty paths.
use super::*;
use bank::owned::{ChunkBank, EmptyPath};
const MAX_WORDS: usize = 96;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct KeyWords {
    words: [usize; MAX_WORDS],
    len: usize,
}
impl KeyWords {
    fn new() -> Self {
        Self {
            words: [0; MAX_WORDS],
            len: 0,
        }
    }
    fn word(&mut self, value: usize) {
        assert!(value <= u16::MAX as usize);
        self.words[self.len] = value;
        self.len += 1;
    }
    fn u64(&mut self, value: u64) {
        for shift in [48, 32, 16, 0] {
            self.word(((value >> shift) & 0xffff) as usize);
        }
    }
    fn usize(&mut self, value: usize) {
        for chunk in (0..usize::BITS.div_ceil(16)).rev() {
            self.word((value >> (chunk * 16)) & 0xffff);
        }
    }
    fn identity(&mut self, source: DomHostedSourceId, slot: usize, generation: u64) {
        self.u64(source.serial);
        self.usize(slot);
        self.u64(generation);
    }
    fn operation(&mut self, id: DomHostedOperationId) {
        self.identity(id.source, id.slot, id.generation);
    }
    pub fn retry(creation: DomHostedCreation, history: Option<DomHostedOperationId>) -> Self {
        let mut key = Self::new();
        let document = creation.document.node();
        key.usize(document.slot);
        key.u64(document.generation);
        for (source, slot, generation) in [
            (
                creation.host.source,
                creation.host.slot,
                creation.host.generation,
            ),
            (
                creation.context.source,
                creation.context.slot,
                creation.context.generation,
            ),
            (
                creation.global.source,
                creation.global.slot,
                creation.global.generation,
            ),
            (
                creation.entry.source,
                creation.entry.slot,
                creation.entry.generation,
            ),
            (
                creation.navigable.source,
                creation.navigable.slot,
                creation.navigable.generation,
            ),
        ] {
            key.identity(source, slot, generation);
        }
        key.word(usize::from(history.is_some()));
        if let Some(id) = history {
            key.operation(id);
        }
        key
    }
    pub fn cleanup(creation: DomHostedCreation) -> Self {
        let mut key = Self::new();
        let id = creation.document.node();
        key.usize(id.slot);
        key.u64(id.generation);
        let id = creation.global;
        key.identity(id.source, id.slot, id.generation);
        key
    }
    pub fn history(id: DomHostedOperationId) -> Self {
        let mut key = Self::new();
        key.operation(id);
        key
    }
    fn row(mut self, slot: usize) -> Self {
        self.usize(slot);
        self
    }
}
#[derive(Default)]
struct Node {
    children: ChunkBank<Node>,
    // Installed identity survives temporary invisibility. Another operation's
    // permanent retirement therefore cannot discard a held restoration path.
    value: Option<DomHostedOperationId>,
    active: bool,
}
impl Node {
    fn active(&self) -> bool {
        self.active || self.children.active_count() != 0
    }
    fn empty(&self) -> bool {
        self.value.is_none() && self.children.retained_count() == 0
    }
}
#[derive(Default)]
pub(super) struct Index {
    root: Node,
}
pub(super) struct Plan {
    key: KeyWords,
    paths: Vec<Option<EmptyPath<Node>>>,
    _source_body: Option<source::checkpoint::Charge>,
    _producer: Option<Retained>,
}
impl Index {
    pub fn prepare(
        &self,
        key: KeyWords,
        slot: usize,
        admission: &mut impl FnMut(bank::IndexCharge) -> Result<Option<Retained>, DomHostedError>,
        formation: Option<&source::formation::Formation>,
        work: &Work,
        cancel: &AtomicBool,
    ) -> Result<Plan, DomHostedError> {
        let key = key.row(slot);
        let requested = key
            .len
            .checked_mul(size_of::<Option<EmptyPath<Node>>>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        let producer = admission(bank::IndexCharge {
            bytes: requested,
            pages: 0,
        })?;
        let body = formation.map(|f| {
            f.split_payload(source::storage::Payload {
                bytes: requested,
                units: 0,
            })
        });
        let mut plan = Plan {
            key,
            paths: Vec::new(),
            _source_body: body,
            _producer: producer,
        };
        plan.paths
            .try_reserve_exact(key.len)
            .map_err(|_| DomHostedError::AllocationFailed)?;
        let actual = plan
            .paths
            .capacity()
            .checked_mul(size_of::<Option<EmptyPath<Node>>>())
            .ok_or(DomHostedError::RepresentationExhausted)?;
        #[cfg(test)]
        scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes: actual });
        let extra = actual
            .checked_sub(requested)
            .ok_or(DomHostedError::RepresentationExhausted)?;
        if extra != 0 {
            let producer = admission(bank::IndexCharge {
                bytes: extra,
                pages: 0,
            })?;
            let body = formation.map(|f| {
                f.split_payload(source::storage::Payload {
                    bytes: extra,
                    units: 0,
                })
            });
            match (&mut plan._producer, producer) {
                (Some(first), Some(extra)) => {
                    first.append(extra).expect("one original hosted producer")
                }
                (None, None) => {}
                _ => unreachable!("one producer domain"),
            }
            if let Some(body) = body {
                plan._source_body
                    .as_mut()
                    .expect("one Source domain")
                    .merge(body);
            }
        }
        // Both initialization and subsequent path construction are prepaid;
        // refusal leaves data-first temporary owners to reclaim their backing.
        for _ in 0..key.len {
            work.step(cancel)?;
            if let Some(formation) = formation {
                formation.step(cancel)?;
            }
        }
        plan.paths.resize_with(key.len, || None);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            let mut copies = crate::store::CopyObservation::default();
            copies.extended::<Option<EmptyPath<Node>>>(key.len);
        }
        let mut current = Some(&self.root);
        let empty = ChunkBank::<Node>::default();
        for (depth, &word) in key.words[..key.len].iter().enumerate() {
            let children = current.map_or(&empty, |node| &node.children);
            plan.paths[depth] = Some(children.reserve(
                u16::try_from(word).expect("qualified native chunk"),
                &mut *admission,
            )?);
            current = children.get(word);
        }
        Ok(plan)
    }
    pub fn install(&mut self, mut plan: Plan, id: DomHostedOperationId) -> bank::IndexCharge {
        let mut moved = bank::IndexCharge::default();
        let mut node = &mut self.root;
        for (depth, &word) in plan.key.words[..plan.key.len].iter().enumerate() {
            if node.children.get(word).is_none() {
                let retained = node.children.insert(
                    plan.paths[depth]
                        .take()
                        .expect("independent complete projection path"),
                    Node::default(),
                );
                moved.bytes = moved
                    .bytes
                    .checked_add(retained.bytes)
                    .expect("moved admitted subset");
                moved.pages = moved
                    .pages
                    .checked_add(retained.pages)
                    .expect("moved admitted subset");
                node.children.set_active(word, false);
            }
            node = &mut node.children[word];
        }
        assert!(
            node.value.is_none(),
            "original projection destination vacant"
        );
        self.set(plan.key, id, true);
        drop(plan); // Actual unused spares and Vec die before either domain debit.
        moved
    }
    fn set(&mut self, key: KeyWords, id: DomHostedOperationId, active: bool) {
        fn write(node: &mut Node, words: &[usize], id: DomHostedOperationId, active: bool) -> bool {
            if let Some((&word, rest)) = words.split_first() {
                let child = node
                    .children
                    .get_mut(word)
                    .expect("installed or held projection path");
                let active = write(child, rest, id, active);
                node.children.set_active(word, active);
            } else {
                assert!(
                    node.value.is_none_or(|original| original == id),
                    "same original installed identity"
                );
                node.value = Some(id);
                node.active = active;
            }
            node.active()
        }
        write(&mut self.root, &key.words[..key.len], id, active);
    }
    pub fn remove(&mut self, key: KeyWords, id: DomHostedOperationId) {
        self.set(key.row(id.slot), id, false);
    }
    pub fn restore(&mut self, key: KeyWords, id: DomHostedOperationId) {
        self.set(key.row(id.slot), id, true);
    }
    pub fn retire(&mut self, key: KeyWords, id: DomHostedOperationId) {
        fn retire(node: &mut Node, words: &[usize], id: DomHostedOperationId) -> bool {
            if let Some((&word, rest)) = words.split_first() {
                let child = node
                    .children
                    .get_mut(word)
                    .expect("original allowed projection path");
                let empty = retire(child, rest, id);
                let active = child.active();
                if empty {
                    node.children
                        .purge(u16::try_from(word).expect("qualified chunk"));
                } else {
                    node.children.set_active(word, active);
                }
            } else {
                assert!(
                    node.value == Some(id),
                    "retire the original qualified identity"
                );
                node.value = None;
                node.active = false;
            }
            node.empty()
        }
        let key = key.row(id.slot);
        retire(&mut self.root, &key.words[..key.len], id);
    }
    pub fn first(&self, key: KeyWords) -> Option<DomHostedOperationId> {
        let mut node = &self.root;
        for &word in &key.words[..key.len] {
            node = node.children.get(word)?;
        }
        loop {
            if node.active {
                return node.value;
            }
            let (index, _) = node.children.first_active()?;
            node = node.children.get(index)?;
        }
    }
    #[cfg(test)]
    pub fn inspect_storage_bytes(&self) -> usize {
        fn walk(node: &Node) -> usize {
            node.children.inspect_storage_bytes() + node.children.values().map(walk).sum::<usize>()
        }
        walk(&self.root)
    }
}
impl Plan {
    pub fn component_bytes(&self) -> usize {
        self.paths
            .iter()
            .flatten()
            .map(|path| path.charge().bytes)
            .sum()
    }
    pub fn backing_bytes(&self) -> usize {
        self.component_bytes() + self.paths.capacity() * size_of::<Option<EmptyPath<Node>>>()
    }
    #[cfg(test)]
    pub fn inspect_storage_bytes(&self) -> usize {
        self.paths.capacity() * size_of::<Option<EmptyPath<Node>>>()
            + self
                .paths
                .iter()
                .flatten()
                .map(EmptyPath::inspect_storage_bytes)
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod source_tests;
