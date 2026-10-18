//! Exact maintained native-control participants. The mutable owner changes
//! selected paired bank paths; immutable snapshots contain only derived values.
//!
//! Patricia branches store the first differing bit, so lookup visits only a
//! bounded key path. UTF-16 names are compared exactly, including lone surrogates
//! and prefixes. A group leaf names one of its actual members; it never copies
//! the group's string or searches unrelated groups.
use super::super::*;
use super::NodeFields;

#[derive(Clone, Debug)]
struct Facts {
    key: Key,
    document: Key,
    root: Key,
    owner: Option<Key>,
    form_present: bool,
    radio: bool,
    name: Option<DocumentString>,
    name_usage: custody::Usage,
}

impl Facts {
    fn usage(&self) -> Result<custody::Usage, DomError> {
        custody::Usage::new(size_of::<Self>(), 0).checked_add(self.name_usage)
    }
    fn same(&self, other: &Self) -> bool {
        self.key == other.key
            && self.document == other.document
            && self.root == other.root
            && self.owner == other.owner
            && self.form_present == other.form_present
            && self.radio == other.radio
            && match (&self.name, &other.name) {
                (Some(a), Some(b)) => Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()),
                (None, None) => true,
                _ => false,
            }
    }
    fn eligible(&self, kind: GroupKind) -> bool {
        match kind {
            GroupKind::Document | GroupKind::Tree => self.form_present,
            GroupKind::Radio => self.radio && self.name.as_ref().is_some_and(|v| !v.is_empty()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum IndexNode {
    Branch { bit: usize, children: [usize; 2] },
    Group { representative: Key, members: usize },
    Member(Key),
}

#[derive(Clone, Copy)]
enum GroupKind {
    Document = 0,
    Tree = 1,
    Radio = 2,
}
const GROUPS: [GroupKind; 3] = [GroupKind::Document, GroupKind::Tree, GroupKind::Radio];

/// Lease-free canonical derived state. Full Key equality qualifies every fact.
#[derive(Clone, Debug, Default)]
pub(in crate::store) struct ControlIndex {
    owner: Option<DomOwnerId>,
    existing: bank::Bank<Facts>,
    created: bank::Bank<Facts>,
    nodes: bank::Bank<IndexNode>,
    roots: [Option<usize>; 3],
}

#[derive(Clone, Default)]
pub(in crate::store) struct ControlCustody {
    existing: bank::BankCustody<Facts>,
    created: bank::BankCustody<Facts>,
    nodes: bank::BankCustody<IndexNode>,
}

#[derive(Clone, Default)]
struct Version {
    index: ControlIndex,
    dirty_existing: bank::Bank<Key>,
    dirty_created: bank::Bank<Key>,
    custody: ControlCustody,
    dirty_existing_custody: bank::BankCustody<Key>,
    dirty_created_custody: bank::BankCustody<Key>,
}

pub(in crate::store) struct Controls {
    base: ControlIndex,
    current: Version,
    base_custody: ControlCustody,
}

pub(in crate::store) struct Finished {
    pub value: ControlIndex,
    pub custody: ControlCustody,
}

pub(in crate::store) struct ControlPlan {
    next: Version,
    retain_undo: bool,
}
pub(in crate::store) struct ControlUndo {
    before: Version,
}
impl ControlUndo {
    pub fn restore(self, draft: &mut Draft) {
        draft.controls.current = self.before;
    }
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        // Shared roots are charged by their actual controlled graph, while
        // this inline inverse is already inside the journal backing.
        0
    }
}

struct Reconcile(bank::CustodyRegistry);
impl Drop for Reconcile {
    fn drop(&mut self) {
        self.0.reconcile_releases();
    }
}

struct Pair<T> {
    values: bank::Bank<T>,
    custody: bank::BankCustody<T>,
}
impl<T> Drop for Pair<T> {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.values));
        drop(std::mem::take(&mut self.custody));
    }
}

struct Editing<'a> {
    registry: &'a bank::CustodyRegistry,
    admit: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    step: &'a mut dyn FnMut() -> Result<(), DomError>,
}

fn set<T>(
    values: &mut bank::Bank<T>,
    controls: &mut bank::BankCustody<T>,
    index: usize,
    value: T,
    usage: custody::Usage,
    edit: &mut Editing<'_>,
) -> Result<(), DomError> {
    (edit.step)()?;
    let value = bank::ControlledRecord::new(value, usage, edit.registry, &mut *edit.admit)?;
    let (next, custody) = values.set_controlled(
        controls,
        index,
        value,
        false,
        bank::Totals {
            bytes: usage.bytes(),
            units: usage.units(),
            occurrences: 1,
        },
        edit.registry,
        &mut *edit.admit,
    )?;
    drop(Pair {
        values: std::mem::replace(values, next),
        custody: std::mem::replace(controls, custody),
    });
    edit.registry.reconcile_releases();
    Ok(())
}
fn remove<T>(
    values: &mut bank::Bank<T>,
    controls: &mut bank::BankCustody<T>,
    index: usize,
    edit: &mut Editing<'_>,
) -> Result<(), DomError> {
    (edit.step)()?;
    let (next, custody, removed) =
        values.remove_controlled(controls, index, edit.registry, &mut *edit.admit)?;
    drop(Pair {
        values: std::mem::replace(values, next),
        custody: std::mem::replace(controls, custody),
    });
    drop(removed);
    edit.registry.reconcile_releases();
    Ok(())
}

// All Existing keys belong to ControlIndex.owner. The encoded slot/generation
// is therefore collision-free within this store; leaf equality still checks
// the complete owner-qualified Key before a fact can be consumed.
fn key_words(key: Key) -> [u64; 3] {
    match key {
        Key::Existing(id) => [0, id.slot as u64, id.generation],
        Key::Created(index) => [1, index as u64, 0],
    }
}
struct Encoded<'a> {
    words: [u64; 7],
    count: usize,
    name: &'a [u16],
}
impl<'a> Encoded<'a> {
    fn member(key: Key) -> Self {
        let mut words = [0; 7];
        words[..3].copy_from_slice(&key_words(key));
        Self {
            words,
            count: 3,
            name: &[],
        }
    }
    fn group(fact: &'a Facts, kind: GroupKind) -> Self {
        let mut words = [0; 7];
        match kind {
            GroupKind::Document => {
                words[..3].copy_from_slice(&key_words(fact.document));
                Self {
                    words,
                    count: 3,
                    name: &[],
                }
            }
            GroupKind::Tree => {
                words[..3].copy_from_slice(&key_words(fact.root));
                Self {
                    words,
                    count: 3,
                    name: &[],
                }
            }
            GroupKind::Radio => {
                words[..3].copy_from_slice(&key_words(fact.root));
                if let Some(owner) = fact.owner {
                    words[3] = 1;
                    words[4..7].copy_from_slice(&key_words(owner));
                }
                Self {
                    words,
                    count: 7,
                    name: fact.name.as_ref().map_or(&[], |v| v.code_units()),
                }
            }
        }
    }
    fn bit(&self, bit: usize) -> usize {
        let fixed = self.count * 64;
        if bit < fixed {
            return ((self.words[bit / 64] >> (63 - bit % 64)) & 1) as usize;
        }
        // UTF-16 + 1 uses seventeen bits, reserving zero as a terminator.
        let position = bit - fixed;
        let symbol = self
            .name
            .get(position / 17)
            .map_or(0, |unit| u32::from(*unit) + 1);
        ((symbol >> (16 - position % 17)) & 1) as usize
    }
    fn difference(
        &self,
        other: &Self,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        assert_eq!(self.count, other.count, "one trie has one key shape");
        for i in 0..self.count {
            step()?;
            let difference = self.words[i] ^ other.words[i];
            if difference != 0 {
                return Ok(Some(i * 64 + difference.leading_zeros() as usize));
            }
        }
        for position in 0..=self.name.len().min(other.name.len()) {
            step()?;
            let a = self.name.get(position).map_or(0, |v| u32::from(*v) + 1);
            let b = other.name.get(position).map_or(0, |v| u32::from(*v) + 1);
            if a != b {
                let offset = position
                    .checked_mul(17)
                    .and_then(|v| v.checked_add((a ^ b).leading_zeros() as usize - 15))
                    .ok_or(DomError::RepresentationExhausted)?;
                return self
                    .count
                    .checked_mul(64)
                    .and_then(|v| v.checked_add(offset))
                    .map(Some)
                    .ok_or(DomError::RepresentationExhausted);
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Copy)]
enum Trie {
    Group(GroupKind),
    Members,
}
impl ControlIndex {
    fn fact(&self, key: Key) -> Option<&Facts> {
        let fact = match key {
            Key::Existing(id) => self.existing.get(id.slot),
            Key::Created(index) => self.created.get(index),
        }?;
        (fact.key == key).then_some(fact)
    }
    fn leaf_key(&self, node: usize, trie: Trie) -> Encoded<'_> {
        match (*self.nodes.get(node).expect("live control trie edge"), trie) {
            (IndexNode::Member(key), Trie::Members) => Encoded::member(key),
            (IndexNode::Group { representative, .. }, Trie::Group(kind)) => Encoded::group(
                self.fact(representative)
                    .expect("group representative remains a member"),
                kind,
            ),
            _ => unreachable!("control trie leaf shape"),
        }
    }
    fn leaf(
        &self,
        root: usize,
        target: &Encoded<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        let mut current = root;
        loop {
            step()?;
            match self.nodes.get(current).expect("live control trie path") {
                IndexNode::Branch { bit, children } => current = children[target.bit(*bit)],
                _ => return Ok(current),
            }
        }
    }
    fn group(
        &self,
        fact: &Facts,
        kind: GroupKind,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        let Some(root) = self.roots[kind as usize] else {
            return Ok(None);
        };
        let target = Encoded::group(fact, kind);
        let leaf = self.leaf(root, &target, step)?;
        Ok(target
            .difference(&self.leaf_key(leaf, Trie::Group(kind)), step)?
            .is_none()
            .then_some(leaf))
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<ControlCustody, DomError> {
        let _cleanup = Reconcile(registry.clone());
        let existing = self.existing.adopt_controlled(
            registry,
            |fact| {
                step()?;
                fact.usage()
            },
            &mut *admit,
        )?;
        let created = self.created.adopt_controlled(
            registry,
            |fact| {
                step()?;
                fact.usage()
            },
            &mut *admit,
        )?;
        let nodes = self.nodes.adopt_controlled(
            registry,
            |_| {
                step()?;
                Ok(custody::Usage::new(size_of::<IndexNode>(), 0))
            },
            admit,
        )?;
        Ok(ControlCustody {
            existing,
            created,
            nodes,
        })
    }

    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let facts = |fact: &Facts| {
            let mut payload = source::storage::Payload::default();
            if let Some(name) = &fact.name {
                payload.string(name)?;
            }
            Ok(bank::Totals {
                bytes: size_of::<Facts>()
                    .checked_add(payload.bytes)
                    .ok_or(DomError::RepresentationExhausted)?,
                units: payload.units,
                occurrences: 1,
            })
        };
        let mut graph = self.existing.inspect_graph(facts)?;
        graph.merge(&self.created.inspect_graph(facts)?)?;
        graph.merge(&self.nodes.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<IndexNode>(),
                units: 0,
                occurrences: 1,
            })
        })?)?;
        Ok(graph)
    }
    #[cfg(test)]
    pub fn inspect_summaries(&self) -> [(bank::Totals, bank::IndexCharge); 3] {
        [
            (self.existing.totals(), self.existing.index_charge()),
            (self.created.totals(), self.created.index_charge()),
            (self.nodes.totals(), self.nodes.index_charge()),
        ]
    }
    #[cfg(test)]
    pub fn inspect_facts(&self) -> Vec<ControlFactInspection> {
        self.existing
            .values()
            .chain(self.created.values())
            .map(|fact| ControlFactInspection {
                key: fact.key,
                document: fact.document,
                root: fact.root,
                owner: fact.owner,
                form_present: fact.form_present,
                radio: fact.radio,
                name: fact.name.clone(),
            })
            .collect()
    }
    #[cfg(test)]
    pub fn inspect_groups(&self) -> Vec<ControlGroupInspection> {
        fn leaves(nodes: &bank::Bank<IndexNode>, root: usize) -> Vec<IndexNode> {
            let mut pending = vec![root];
            let mut seen = std::collections::BTreeSet::new();
            let mut leaves = Vec::new();
            while let Some(index) = pending.pop() {
                assert!(seen.insert(index), "acyclic raw control trie");
                match *nodes.get(index).expect("actual control edge") {
                    IndexNode::Branch { children, .. } => {
                        pending.extend(children.into_iter().rev())
                    }
                    leaf => leaves.push(leaf),
                }
            }
            leaves
        }
        let mut result = Vec::new();
        for kind in GROUPS {
            if let Some(root) = self.roots[kind as usize] {
                for leaf in leaves(&self.nodes, root) {
                    let IndexNode::Group {
                        representative,
                        members,
                    } = leaf
                    else {
                        panic!("group trie leaf")
                    };
                    let members = leaves(&self.nodes, members)
                        .into_iter()
                        .map(|leaf| match leaf {
                            IndexNode::Member(key) => key,
                            _ => panic!("member trie leaf"),
                        })
                        .collect();
                    result.push(ControlGroupInspection {
                        kind: match kind {
                            GroupKind::Document => ControlGroupKind::Document,
                            GroupKind::Tree => ControlGroupKind::Tree,
                            GroupKind::Radio => ControlGroupKind::Radio,
                        },
                        representative,
                        members,
                    });
                }
            }
        }
        result
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct ControlFactInspection {
    pub key: Key,
    pub document: Key,
    pub root: Key,
    pub owner: Option<Key>,
    pub form_present: bool,
    pub radio: bool,
    pub name: Option<DocumentString>,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum ControlGroupKind {
    Document,
    Tree,
    Radio,
}
#[cfg(test)]
pub(in crate::store) struct ControlGroupInspection {
    pub kind: ControlGroupKind,
    pub representative: Key,
    pub members: Vec<Key>,
}

impl Version {
    fn node(
        &mut self,
        index: usize,
        value: IndexNode,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        set(
            &mut self.index.nodes,
            &mut self.custody.nodes,
            index,
            value,
            custody::Usage::new(size_of::<IndexNode>(), 0),
            edit,
        )
    }
    fn allocate(&mut self, value: IndexNode, edit: &mut Editing<'_>) -> Result<usize, DomError> {
        let index = self
            .index
            .nodes
            .lowest_vacant()
            .unwrap_or(self.index.nodes.len());
        self.node(index, value, edit)?;
        Ok(index)
    }
    fn remove_node(&mut self, index: usize, edit: &mut Editing<'_>) -> Result<(), DomError> {
        remove(&mut self.index.nodes, &mut self.custody.nodes, index, edit)
    }
    fn link(
        &mut self,
        parent: Option<(usize, usize)>,
        child: usize,
        root: &mut Option<usize>,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        if let Some((index, side)) = parent {
            let IndexNode::Branch { bit, mut children } =
                *self.index.nodes.get(index).expect("parent branch")
            else {
                unreachable!()
            };
            children[side] = child;
            self.node(index, IndexNode::Branch { bit, children }, edit)
        } else {
            *root = Some(child);
            Ok(())
        }
    }
    fn insert(
        &mut self,
        mut root: Option<usize>,
        value: IndexNode,
        trie: Trie,
        edit: &mut Editing<'_>,
    ) -> Result<usize, DomError> {
        let target = match (&value, trie) {
            (IndexNode::Member(key), Trie::Members) => Encoded::member(*key),
            (IndexNode::Group { representative, .. }, Trie::Group(kind)) => Encoded::group(
                self.index.fact(*representative).expect("new group fact"),
                kind,
            ),
            _ => unreachable!("new trie leaf"),
        };
        let Some(original) = root else {
            return self.allocate(value, edit);
        };
        let leaf = self.index.leaf(original, &target, edit.step)?;
        let Some(bit) = target.difference(&self.index.leaf_key(leaf, trie), edit.step)? else {
            return Ok(original);
        };
        let side = target.bit(bit);
        let mut current = original;
        let mut parent = None;
        loop {
            (edit.step)()?;
            match *self.index.nodes.get(current).expect("insertion path") {
                IndexNode::Branch {
                    bit: previous,
                    children,
                } if previous < bit => {
                    let side = target.bit(previous);
                    parent = Some((current, side));
                    current = children[side];
                }
                _ => break,
            }
        }
        let leaf = self.allocate(value, edit)?;
        let mut children = [current; 2];
        children[side] = leaf;
        let branch = self.allocate(IndexNode::Branch { bit, children }, edit)?;
        self.link(parent, branch, &mut root, edit)?;
        Ok(root.expect("inserted root"))
    }
    fn delete(
        &mut self,
        mut root: Option<usize>,
        target: &Encoded<'_>,
        trie: Trie,
        edit: &mut Editing<'_>,
    ) -> Result<Option<usize>, DomError> {
        let Some(original) = root else {
            return Ok(None);
        };
        let mut current = original;
        let mut parent = None;
        let mut grandparent = None;
        loop {
            (edit.step)()?;
            match *self.index.nodes.get(current).expect("deletion path") {
                IndexNode::Branch { bit, children } => {
                    grandparent = parent;
                    let side = target.bit(bit);
                    parent = Some((current, side));
                    current = children[side];
                }
                _ => break,
            }
        }
        if target
            .difference(&self.index.leaf_key(current, trie), edit.step)?
            .is_some()
        {
            return Ok(root);
        }
        if let Some((index, side)) = parent {
            let IndexNode::Branch { children, .. } = *self.index.nodes.get(index).expect("parent")
            else {
                unreachable!()
            };
            self.link(grandparent, children[1 - side], &mut root, edit)?;
            self.remove_node(index, edit)?;
        } else {
            root = None;
        }
        self.remove_node(current, edit)?;
        Ok(root)
    }
    fn membership(
        &mut self,
        fact: &Facts,
        kind: GroupKind,
        present: bool,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        if !fact.eligible(kind) {
            return Ok(());
        }
        let group = self.index.group(fact, kind, edit.step)?;
        if present {
            if let Some(index) = group {
                let IndexNode::Group {
                    representative,
                    members,
                } = *self.index.nodes.get(index).expect("group")
                else {
                    unreachable!()
                };
                let members = self.insert(
                    Some(members),
                    IndexNode::Member(fact.key),
                    Trie::Members,
                    edit,
                )?;
                self.node(
                    index,
                    IndexNode::Group {
                        representative,
                        members,
                    },
                    edit,
                )?;
            } else {
                let members = self.allocate(IndexNode::Member(fact.key), edit)?;
                self.index.roots[kind as usize] = Some(self.insert(
                    self.index.roots[kind as usize],
                    IndexNode::Group {
                        representative: fact.key,
                        members,
                    },
                    Trie::Group(kind),
                    edit,
                )?);
            }
        } else if let Some(index) = group {
            let IndexNode::Group {
                representative,
                members,
            } = *self.index.nodes.get(index).expect("group")
            else {
                unreachable!()
            };
            match self.delete(
                Some(members),
                &Encoded::member(fact.key),
                Trie::Members,
                edit,
            )? {
                Some(members) => {
                    let representative = if representative == fact.key {
                        first_member(&self.index.nodes, members, edit.step)?
                    } else {
                        representative
                    };
                    self.node(
                        index,
                        IndexNode::Group {
                            representative,
                            members,
                        },
                        edit,
                    )?;
                }
                None => {
                    self.index.roots[kind as usize] = self.delete(
                        self.index.roots[kind as usize],
                        &Encoded::group(fact, kind),
                        Trie::Group(kind),
                        edit,
                    )?;
                }
            }
        }
        Ok(())
    }
    fn qualify(&mut self, fact: &Facts) -> Result<(), DomError> {
        for key in [
            Some(fact.key),
            Some(fact.document),
            Some(fact.root),
            fact.owner,
        ]
        .into_iter()
        .flatten()
        {
            if let Key::Existing(id) = key {
                if self.index.owner.is_some_and(|owner| owner != id.owner) {
                    return Err(DomError::ForeignOwner);
                }
                self.index.owner = Some(id.owner);
            }
        }
        Ok(())
    }
    fn replace(
        &mut self,
        key: Key,
        next: Option<Facts>,
        edit: &mut Editing<'_>,
    ) -> Result<bool, DomError> {
        if let Some(next) = &next {
            self.qualify(next)?;
        }
        let old = match key {
            Key::Existing(id) => self
                .index
                .existing
                .controlled_record(&self.custody.existing, id.slot),
            Key::Created(index) => self
                .index
                .created
                .controlled_record(&self.custody.created, index),
        };
        if old.as_ref().is_some_and(|old| old.value().key != key) {
            return Err(DomError::StaleNode);
        }
        if match (&old, &next) {
            (Some(a), Some(b)) => a.value().same(b),
            (None, None) => true,
            _ => false,
        } {
            return Ok(false);
        }
        if let Some(old) = &old {
            for kind in GROUPS {
                self.membership(old.value(), kind, false, edit)?;
            }
        }
        match key {
            Key::Existing(id) => {
                if let Some(next) = &next {
                    set(
                        &mut self.index.existing,
                        &mut self.custody.existing,
                        id.slot,
                        next.clone(),
                        next.usage()?,
                        edit,
                    )?;
                } else if old.is_some() {
                    remove(
                        &mut self.index.existing,
                        &mut self.custody.existing,
                        id.slot,
                        edit,
                    )?;
                }
            }
            Key::Created(index) => {
                if let Some(next) = &next {
                    set(
                        &mut self.index.created,
                        &mut self.custody.created,
                        index,
                        next.clone(),
                        next.usage()?,
                        edit,
                    )?;
                } else if old.is_some() {
                    remove(
                        &mut self.index.created,
                        &mut self.custody.created,
                        index,
                        edit,
                    )?;
                }
            }
        }
        if let Some(next) = &next {
            for kind in GROUPS {
                self.membership(next, kind, true, edit)?;
            }
        }
        Ok(true)
    }
    fn dirty(&mut self, key: Key, edit: &mut Editing<'_>) -> Result<(), DomError> {
        let (values, custody, index) = match key {
            Key::Existing(id) => (
                &mut self.dirty_existing,
                &mut self.dirty_existing_custody,
                id.slot,
            ),
            Key::Created(index) => (
                &mut self.dirty_created,
                &mut self.dirty_created_custody,
                index,
            ),
        };
        if values.get(index) == Some(&key) {
            return Ok(());
        }
        set(
            values,
            custody,
            index,
            key,
            custody::Usage::new(size_of::<Key>(), 0),
            edit,
        )
    }
}

fn first_member(
    nodes: &bank::Bank<IndexNode>,
    mut root: usize,
    step: &mut dyn FnMut() -> Result<(), DomError>,
) -> Result<Key, DomError> {
    loop {
        step()?;
        match nodes.get(root).expect("member tree") {
            IndexNode::Branch { children, .. } => root = children[0],
            IndexNode::Member(key) => return Ok(*key),
            _ => unreachable!("member leaf"),
        }
    }
}

/// The selected semantic/custody aliases remain stable while original
/// controllers change the live index. No unrelated participant is copied.
pub(in crate::store) struct Participants {
    nodes: bank::Bank<IndexNode>,
    root: Option<usize>,
    custody: bank::BankCustody<IndexNode>,
    registry: bank::CustodyRegistry,
}
impl Drop for Participants {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.nodes));
        drop(std::mem::take(&mut self.custody));
        self.registry.reconcile_releases();
    }
}
pub(in crate::store) struct Members<'a> {
    nodes: &'a bank::Bank<IndexNode>,
    root: Option<usize>,
    next: Option<Key>,
}
impl Participants {
    pub fn cursor(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Members<'_>, DomError> {
        Ok(Members {
            nodes: &self.nodes,
            root: self.root,
            next: self
                .root
                .map(|root| first_member(&self.nodes, root, &mut step))
                .transpose()?,
        })
    }
}
impl Members<'_> {
    pub fn next(
        &mut self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let Some(value) = self.next else {
            return Ok(None);
        };
        let target = Encoded::member(value);
        let mut current = self.root.expect("nonempty members");
        let mut successor = None;
        loop {
            step()?;
            match self.nodes.get(current).expect("member successor path") {
                IndexNode::Branch { bit, children } => {
                    let side = target.bit(*bit);
                    if side == 0 {
                        successor = Some(children[1]);
                    }
                    current = children[side];
                }
                IndexNode::Member(_) => break,
                _ => unreachable!("member leaf"),
            }
        }
        self.next = successor
            .map(|root| first_member(self.nodes, root, &mut step))
            .transpose()?;
        Ok(Some(value))
    }
}

impl Controls {
    pub fn capture(base: &ControlIndex, custody: &ControlCustody) -> Self {
        Self {
            base: base.clone(),
            current: Version {
                index: base.clone(),
                custody: custody.clone(),
                ..Version::default()
            },
            base_custody: custody.clone(),
        }
    }
    pub fn mark_published(&mut self, base: &ControlIndex, custody: &ControlCustody) {
        self.base = base.clone();
        self.base_custody = custody.clone();
        self.current.dirty_existing = bank::Bank::default();
        self.current.dirty_created = bank::Bank::default();
        self.current.dirty_existing_custody = bank::BankCustody::default();
        self.current.dirty_created_custody = bank::BankCustody::default();
    }
    pub fn finish(
        &self,
        mapper: impl Fn(Key) -> Option<NodeId>,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Finished, DomError> {
        let _cleanup = Reconcile(registry.clone());
        let mut output = Version {
            index: self.base.clone(),
            custody: self.base_custody.clone(),
            ..Version::default()
        };
        let mut edit = Editing {
            registry,
            admit: &mut admit,
            step: &mut step,
        };
        for key in self
            .current
            .dirty_existing
            .values()
            .chain(self.current.dirty_created.values())
            .copied()
        {
            (edit.step)()?;
            let Some(id) = mapper(key) else { continue };
            let mapped = self
                .current
                .index
                .fact(key)
                .map(|fact| -> Result<Facts, DomError> {
                    let map = |key| mapper(key).map(Key::Existing).ok_or(DomError::StaleNode);
                    Ok(Facts {
                        key: Key::Existing(id),
                        document: map(fact.document)?,
                        root: map(fact.root)?,
                        owner: fact.owner.map(map).transpose()?,
                        ..fact.clone()
                    })
                })
                .transpose()?;
            output.replace(Key::Existing(id), mapped, &mut edit)?;
        }
        Ok(Finished {
            value: output.index,
            custody: output.custody,
        })
    }
}

fn listed(kind: &DomNodeKind) -> bool {
    matches!(kind, DomNodeKind::Element(element) if element.native_control().is_some_and(|(_,kind)|kind.listed()))
}
fn attributes_same(a: &DomNodeKind, b: &DomNodeKind) -> bool {
    match (a, b) {
        (DomNodeKind::Element(a), DomNodeKind::Element(b)) => Arc::ptr_eq(
            a.attributes().shared_attributes(),
            b.attributes().shared_attributes(),
        ),
        _ => true,
    }
}
impl Draft {
    fn control_facts(
        &self,
        key: Key,
        fields: &NodeFields,
        root: Key,
        reuse_attributes: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Facts>, DomError> {
        if !listed(&fields.kind) {
            return Ok(None);
        }
        let old = self.controls.current.index.fact(key);
        let DomNodeKind::Element(element) = &fields.kind else {
            unreachable!()
        };
        let input = element.input_state();
        let (form_present, name, name_usage) = if reuse_attributes && let Some(old) = old {
            (old.form_present, old.name.clone(), old.name_usage)
        } else {
            let mut form_present = false;
            let mut name = None;
            for attribute in element.attributes().iter() {
                self.source_step(cancel)?;
                if attribute.name().namespace().uri().is_some() {
                    continue;
                }
                if self.source_ascii(attribute.name().local_name(), "form", cancel)? {
                    form_present = true;
                }
                if input.is_some()
                    && self.source_ascii(attribute.name().local_name(), "name", cancel)?
                {
                    name = Some(attribute.value().clone());
                }
            }
            let mut payload = source::storage::Payload::default();
            if let Some(name) = &name {
                payload.string(name)?;
            }
            (
                form_present,
                name,
                custody::Usage::new(payload.bytes, payload.units),
            )
        };
        Ok(Some(Facts {
            key,
            document: self.live_key(fields.document),
            root: self.live_key(root),
            owner: fields.control_owner.map(|key| self.live_key(key)),
            form_present,
            radio: input
                .is_some_and(|input| input.input_type() == surgeist_html::HtmlInputType::Radio),
            name,
            name_usage,
        }))
    }
    pub(in crate::store) fn prepare_control_fields(
        &self,
        key: Key,
        fields: &NodeFields,
        cancel: &AtomicBool,
    ) -> Result<Option<ControlPlan>, DomError> {
        let key = self.live_key(key);
        let old = self.node(key)?;
        let parent_changed = old.parent != fields.parent;
        if !parent_changed && !listed(old.kind) && !listed(&fields.kind) {
            return Ok(None);
        }
        let previous_root = if parent_changed {
            Some(crate::slot::root(self, key, cancel)?)
        } else {
            None
        };
        let root = if parent_changed {
            fields
                .parent
                .map(|parent| crate::slot::root(self, parent, cancel))
                .transpose()?
                .unwrap_or(key)
        } else {
            self.controls.current.index.fact(key).map_or_else(
                || crate::slot::root(self, key, cancel),
                |fact| Ok(fact.root),
            )?
        };
        // The ordinary root of a moved subtree changes at the actual parent
        // write, before family insertion/removing controllers observe it.
        let moved = previous_root.is_some_and(|old| self.live_key(old) != self.live_key(root));
        if !moved && !listed(old.kind) && !listed(&fields.kind) {
            return Ok(None);
        }
        let next = self.control_facts(
            key,
            fields,
            root,
            attributes_same(old.kind, &fields.kind),
            cancel,
        )?;
        if !moved
            && match (self.controls.current.index.fact(key), &next) {
                (Some(a), Some(b)) => a.same(b),
                (None, None) => true,
                _ => false,
            }
        {
            return Ok(None);
        }
        let retain_undo = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut candidate = self.controls.current.clone();
        let mut edit = Editing {
            registry: self.custody_registry(),
            admit: &mut |usage| self.admit_custody_usage(usage, cancel),
            step: &mut || self.source_step(cancel),
        };
        if candidate.replace(key, next, &mut edit)? {
            candidate.dirty(key, &mut edit)?;
        }
        if moved {
            let mut child = crate::slot::next(self, key, key, cancel)?;
            while let Some(descendant) = child {
                self.source_step(cancel)?;
                let descendant = self.live_key(descendant);
                if let Some(fact) = candidate.index.fact(descendant) {
                    let next = Facts {
                        root: self.live_key(root),
                        ..fact.clone()
                    };
                    if candidate.replace(descendant, Some(next), &mut edit)? {
                        candidate.dirty(descendant, &mut edit)?;
                    }
                }
                child = crate::slot::next(self, key, descendant, cancel)?;
            }
        }
        Ok(Some(ControlPlan {
            next: candidate,
            retain_undo,
        }))
    }
    pub(in crate::store) fn prepare_control_created(
        &self,
        key: Key,
        fields: &NodeFields,
        cancel: &AtomicBool,
    ) -> Result<Option<ControlPlan>, DomError> {
        let Some(facts) = self.control_facts(key, fields, key, false, cancel)? else {
            return Ok(None);
        };
        let retain_undo = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut next = self.controls.current.clone();
        let mut edit = Editing {
            registry: self.custody_registry(),
            admit: &mut |usage| self.admit_custody_usage(usage, cancel),
            step: &mut || self.source_step(cancel),
        };
        next.replace(key, Some(facts), &mut edit)?;
        next.dirty(key, &mut edit)?;
        Ok(Some(ControlPlan { next, retain_undo }))
    }
    pub(in crate::store) fn prepare_control_retired(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<ControlPlan>, DomError> {
        let key = self.live_key(key);
        if self.controls.current.index.fact(key).is_none() {
            return Ok(None);
        }
        let retain_undo = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut next = self.controls.current.clone();
        let mut edit = Editing {
            registry: self.custody_registry(),
            admit: &mut |usage| self.admit_custody_usage(usage, cancel),
            step: &mut || self.source_step(cancel),
        };
        next.replace(key, None, &mut edit)?;
        next.dirty(key, &mut edit)?;
        Ok(Some(ControlPlan { next, retain_undo }))
    }
    pub(in crate::store) fn prepare_control_visibility(
        &self,
        key: Key,
        was_private: bool,
        is_private: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<ControlPlan>, DomError> {
        let key = self.live_key(key);
        if was_private == is_private || self.controls.current.index.fact(key).is_none() {
            return Ok(None);
        }
        let retain_undo = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut next = self.controls.current.clone();
        let mut edit = Editing {
            registry: self.custody_registry(),
            admit: &mut |usage| self.admit_custody_usage(usage, cancel),
            step: &mut || self.source_step(cancel),
        };
        next.dirty(key, &mut edit)?;
        Ok(Some(ControlPlan { next, retain_undo }))
    }
    pub(in crate::store) fn commit_control_plan(&mut self, plan: ControlPlan) {
        let before = std::mem::replace(&mut self.controls.current, plan.next);
        if plan.retain_undo {
            self.journal
                .push_prepared(super::Undo::Controls(ControlUndo { before }));
        }
    }
    fn control_participants(
        &self,
        fact: &Facts,
        kind: GroupKind,
        cancel: &AtomicBool,
    ) -> Result<Participants, DomError> {
        let group = self
            .controls
            .current
            .index
            .group(fact, kind, &mut || self.source_step(cancel))?;
        let root = group.map(|index| {
            match self
                .controls
                .current
                .index
                .nodes
                .get(index)
                .expect("selected control group")
            {
                IndexNode::Group { members, .. } => *members,
                _ => unreachable!(),
            }
        });
        Ok(Participants {
            nodes: self.controls.current.index.nodes.clone(),
            root,
            custody: self.controls.current.custody.nodes.clone(),
            registry: self.custody_registry().clone(),
        })
    }
    pub(in crate::store) fn control_id_participants(
        &self,
        source: Key,
        same_tree: bool,
        cancel: &AtomicBool,
    ) -> Result<Participants, DomError> {
        let source = self.live_key(source);
        let node = self.node(source)?;
        let fact = Facts {
            key: source,
            document: self.live_key(node.document),
            root: if same_tree {
                self.live_key(crate::slot::root(self, source, cancel)?)
            } else {
                source
            },
            owner: None,
            form_present: true,
            radio: false,
            name: None,
            name_usage: custody::Usage::ZERO,
        };
        self.control_participants(
            &fact,
            if same_tree {
                GroupKind::Tree
            } else {
                GroupKind::Document
            },
            cancel,
        )
    }
    pub(in crate::store) fn control_radio_participants(
        &self,
        key: Key,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<Participants, DomError> {
        let key = self.live_key(key);
        let cached = self
            .controls
            .current
            .index
            .fact(key)
            .ok_or(DomError::StaleNode)?;
        let fact = Facts {
            key,
            document: cached.document,
            root: cached.root,
            owner: cached.owner,
            form_present: false,
            radio: true,
            name: Some(name.clone()),
            name_usage: custody::Usage::ZERO,
        };
        self.control_participants(&fact, GroupKind::Radio, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(
        key: usize,
        document: usize,
        root: usize,
        owner: Option<usize>,
        form_present: bool,
        radio: bool,
        name: Option<&[u16]>,
    ) -> Facts {
        let name = name.map(|name| DocumentString::from_utf16(name.to_vec()));
        let mut payload = source::storage::Payload::default();
        if let Some(name) = &name {
            payload.string(name).unwrap();
        }
        Facts {
            key: Key::Created(key),
            document: Key::Created(document),
            root: Key::Created(root),
            owner: owner.map(Key::Created),
            form_present,
            radio,
            name,
            name_usage: custody::Usage::new(payload.bytes, payload.units),
        }
    }
    fn registry() -> bank::CustodyRegistry {
        bank::CustodyRegistry::new(&custody::Ledger::default(), |_| Ok(())).unwrap()
    }
    fn replace(
        version: &mut Version,
        key: Key,
        next: Option<Facts>,
        registry: &bank::CustodyRegistry,
    ) {
        let mut candidate = version.clone();
        let mut edit = Editing {
            registry,
            admit: &mut |_| Ok(()),
            step: &mut || Ok(()),
        };
        candidate.replace(key, next, &mut edit).unwrap();
        candidate.dirty(key, &mut edit).unwrap();
        *version = candidate;
        registry.reconcile_releases();
    }
    fn selected(
        version: &Version,
        fact: &Facts,
        kind: GroupKind,
        registry: &bank::CustodyRegistry,
    ) -> (Vec<Key>, usize) {
        let mut work = 0;
        let mut step = || {
            work += 1;
            Ok(())
        };
        let group = version.index.group(fact, kind, &mut step).unwrap();
        let root = group.map(|index| match version.index.nodes.get(index).unwrap() {
            IndexNode::Group { members, .. } => *members,
            _ => unreachable!(),
        });
        let participants = Participants {
            nodes: version.index.nodes.clone(),
            root,
            custody: version.custody.nodes.clone(),
            registry: registry.clone(),
        };
        let mut cursor = participants.cursor(&mut step).unwrap();
        let mut keys = Vec::new();
        while let Some(key) = cursor.next(&mut step).unwrap() {
            keys.push(key)
        }
        (keys, work)
    }

    #[test]
    fn form_attribute_presence_selects_all_listed_subjects_in_the_exact_event_scope() {
        let registry = registry();
        let mut version = Version::default();
        let rows = [
            fact(10, 0, 1, None, true, false, None),
            fact(11, 0, 1, None, false, false, None),
            fact(12, 0, 2, None, true, false, None),
            fact(13, 3, 1, None, true, false, None),
        ];
        for row in &rows {
            replace(&mut version, row.key, Some(row.clone()), &registry)
        }
        assert_eq!(
            selected(&version, &rows[0], GroupKind::Tree, &registry).0,
            vec![Key::Created(10), Key::Created(13)]
        );
        assert_eq!(
            selected(&version, &rows[0], GroupKind::Document, &registry).0,
            vec![Key::Created(10), Key::Created(12)]
        );
        // Membership uses presence, with no string or current-owner filtering.
        let mut next = rows[1].clone();
        next.form_present = true;
        next.owner = Some(Key::Created(99));
        replace(&mut version, next.key, Some(next), &registry);
        assert_eq!(
            selected(&version, &rows[0], GroupKind::Tree, &registry).0,
            vec![Key::Created(10), Key::Created(11), Key::Created(13)]
        );
    }

    #[test]
    fn radio_groups_are_exact_for_utf16_prefixes_actual_type_owner_and_ordinary_root() {
        let registry = registry();
        let mut version = Version::default();
        let target = fact(10, 0, 1, None, false, true, Some(&[0xd800]));
        let same = fact(11, 0, 1, None, false, true, Some(&[0xd800]));
        let rows = [
            target.clone(),
            same,
            fact(12, 0, 1, None, false, true, Some(&[0xd800, 0])),
            fact(13, 0, 1, Some(4), false, true, Some(&[0xd800])),
            fact(14, 0, 2, None, false, true, Some(&[0xd800])),
            fact(15, 0, 1, None, false, false, Some(&[0xd800])),
            fact(16, 0, 1, None, false, true, Some(&[])),
        ];
        for row in rows {
            replace(&mut version, row.key, Some(row), &registry)
        }
        assert_eq!(
            selected(&version, &target, GroupKind::Radio, &registry).0,
            vec![Key::Created(10), Key::Created(11)]
        );
        for i in 0..256 {
            let name = [i, 0xdc00];
            let row = fact(100 + i as usize, 0, 1, None, false, true, Some(&name));
            replace(&mut version, row.key, Some(row), &registry)
        }
        let (keys, work) = selected(&version, &target, GroupKind::Radio, &registry);
        assert_eq!(keys, vec![Key::Created(10), Key::Created(11)]);
        assert!(
            work < 80,
            "selected exact group must not visit 256 unrelated groups: {work}"
        );
    }

    #[test]
    fn selected_members_survive_live_group_changes_and_last_representative_removal() {
        let registry = registry();
        let mut version = Version::default();
        let a = fact(10, 0, 1, None, true, true, Some(&[103]));
        let b = fact(11, 0, 1, None, true, true, Some(&[103]));
        for row in [&a, &b] {
            replace(&mut version, row.key, Some(row.clone()), &registry)
        }
        let before = version.clone();
        replace(&mut version, a.key, None, &registry);
        assert_eq!(
            selected(&version, &b, GroupKind::Radio, &registry).0,
            vec![b.key]
        );
        assert_eq!(
            selected(&before, &a, GroupKind::Radio, &registry).0,
            vec![a.key, b.key]
        );
        replace(&mut version, b.key, None, &registry);
        assert!(
            selected(&version, &a, GroupKind::Document, &registry)
                .0
                .is_empty()
        );
        assert_eq!(version.index.nodes.record_count(), 0);
        let graph = version.index.inspect_graph().unwrap();
        assert_eq!(graph.index_charge().bytes, 0);
        assert_eq!(graph.record_totals().occurrences, 0);
        assert_eq!(
            selected(&before, &a, GroupKind::Tree, &registry).0,
            vec![a.key, b.key]
        );
    }

    #[test]
    fn full_generations_and_store_ownership_are_checked_before_reusing_a_fact_slot() {
        let registry = registry();
        let mut version = Version::default();
        let owner = DomOwnerId::allocate().unwrap();
        let foreign = DomOwnerId::allocate().unwrap();
        let a = Key::Existing(NodeId {
            owner,
            slot: 4,
            generation: 7,
        });
        let b = Key::Existing(NodeId {
            owner,
            slot: 4,
            generation: 8,
        });
        let mut row = fact(4, 0, 1, None, true, false, None);
        row.key = a;
        replace(&mut version, a, Some(row.clone()), &registry);
        let mut candidate = version.clone();
        let mut edit = Editing {
            registry: &registry,
            admit: &mut |_| Ok(()),
            step: &mut || Ok(()),
        };
        let mut changed = row.clone();
        changed.key = b;
        assert_eq!(
            candidate.replace(b, Some(changed.clone()), &mut edit),
            Err(DomError::StaleNode)
        );
        assert!(version.index.fact(b).is_none());
        replace(&mut version, a, None, &registry);
        replace(&mut version, b, Some(changed), &registry);
        let mut bad = row;
        bad.key = Key::Existing(NodeId {
            owner: foreign,
            slot: 5,
            generation: 0,
        });
        let mut callbacks = 0;
        let mut candidate = version.clone();
        let mut edit = Editing {
            registry: &registry,
            admit: &mut |_| {
                callbacks += 1;
                Ok(())
            },
            step: &mut || Ok(()),
        };
        assert_eq!(
            candidate.replace(bad.key, Some(bad), &mut edit),
            Err(DomError::ForeignOwner)
        );
        assert_eq!(callbacks, 0);
    }

    #[test]
    fn refused_new_index_storage_keeps_the_previous_groups_and_work_spent() {
        let registry = registry();
        let mut version = Version::default();
        let a = fact(10, 0, 1, None, true, true, Some(&[103]));
        replace(&mut version, a.key, Some(a.clone()), &registry);
        let before = registry.total();
        let b = fact(11, 0, 2, None, true, true, Some(&[104]));
        let mut work = 0;
        let mut calls = 0;
        let mut attempt = version.clone();
        let mut edit = Editing {
            registry: &registry,
            admit: &mut |_| {
                calls += 1;
                if calls == 4 {
                    Err(DomError::AllocationFailed)
                } else {
                    Ok(())
                }
            },
            step: &mut || {
                work += 1;
                Ok(())
            },
        };
        assert_eq!(
            attempt.replace(b.key, Some(b), &mut edit),
            Err(DomError::AllocationFailed)
        );
        drop(attempt);
        registry.reconcile_releases();
        assert!(work > 0);
        assert_eq!(registry.total(), before);
        assert_eq!(
            selected(&version, &a, GroupKind::Radio, &registry).0,
            vec![a.key]
        );
    }

    #[test]
    fn exact_name_capacity_admits_the_index_and_one_unit_short_preserves_the_original_root() {
        let registry = registry();
        let before = registry.total();
        let mut original = Version::default();
        let mut units = Vec::with_capacity(128);
        units.push(103);
        let extent = units.capacity();
        let name = DocumentString::from_utf16(units);
        let row = Facts {
            key: Key::Created(10),
            document: Key::Created(0),
            root: Key::Created(1),
            owner: None,
            form_present: true,
            radio: true,
            name: Some(name),
            name_usage: custody::Usage::new(
                size_of::<Vec<u16>>() + extent * size_of::<u16>(),
                extent,
            ),
        };
        let mut attempted = original.clone();
        let limit = before.units() + extent - 1;
        let mut edit = Editing {
            registry: &registry,
            admit: &mut |next| admit(next.units(), limit, DomResource::SourceRetainedUnits),
            step: &mut || Ok(()),
        };
        assert_eq!(
            attempted.replace(row.key, Some(row.clone()), &mut edit),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedUnits,
                limit
            })
        );
        drop(attempted);
        registry.reconcile_releases();
        assert_eq!(registry.total(), before);
        assert!(original.index.inspect_facts().is_empty());
        let limit = before.units() + extent;
        let mut edit = Editing {
            registry: &registry,
            admit: &mut |next| admit(next.units(), limit, DomResource::SourceRetainedUnits),
            step: &mut || Ok(()),
        };
        original
            .replace(row.key, Some(row.clone()), &mut edit)
            .unwrap();
        assert_eq!(registry.total().units(), limit);
        assert_eq!(
            selected(&original, &row, GroupKind::Radio, &registry).0,
            vec![row.key]
        );
        let independent = original.index.inspect_graph().unwrap();
        assert_eq!(independent.record_totals().units, extent);
    }

    #[test]
    fn tree_selection_prunes_other_ordinary_trees_while_document_selection_visits_its_actual_subjects()
     {
        let registry = registry();
        let mut version = Version::default();
        let target = fact(10, 0, 1, None, true, false, None);
        replace(&mut version, target.key, Some(target.clone()), &registry);
        for i in 0..128 {
            let row = fact(100 + i, 0, 100 + i, None, true, false, None);
            replace(&mut version, row.key, Some(row), &registry)
        }
        let (tree, tree_work) = selected(&version, &target, GroupKind::Tree, &registry);
        assert_eq!(tree, vec![target.key]);
        assert!(
            tree_work < 80,
            "other ordinary trees are excluded before discovery: {tree_work}"
        );
        let (document, document_work) = selected(&version, &target, GroupKind::Document, &registry);
        let expected = std::iter::once(target.key)
            .chain((100..228).map(Key::Created))
            .collect::<Vec<_>>();
        assert_eq!(document, expected);
        assert!(
            document_work > document.len(),
            "the actual participant cursor is metered"
        );
    }

    fn limits() -> DomLimits {
        DomLimits::new(128, 65536, 256, 0, 0, 8, 2_000_000).unwrap()
    }
    fn source_limits() -> crate::DomSourceLimits {
        crate::DomSourceLimits::new(50_000_000, 1_000_000, 100_000_000, 64, 4096, 4096, 1)
    }
    fn source_draft(dom: &Dom, cancel: &AtomicBool) -> Draft {
        let batch = DomBatch::new();
        Draft::new_source(
            dom,
            batch.owner.clone(),
            &batch.commands,
            source_limits(),
            cancel,
        )
        .unwrap()
        .0
    }
    fn checkpoint(draft: &mut Draft, cancel: &AtomicBool) -> source::checkpoint::Checkpoint {
        draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, cancel)
            .unwrap()
    }
    fn publish_phase(draft: &mut Draft, dom: &mut Dom, cancel: &AtomicBool) {
        let prepared = draft.finish(dom, cancel).unwrap();
        let mut publication = dom.prepare_publication(prepared, cancel).unwrap();
        let rebase = draft.prepare_rebase(&mut publication, cancel).unwrap();
        let committed = dom.publish_prepared_with_proof(publication);
        draft.commit_publication(rebase, &committed.committed);
    }
    fn assert_radio_group(draft: &Draft, key: Key, expected: &[Key], cancel: &AtomicBool) {
        let name = DocumentString::from("g");
        let selected = draft
            .control_radio_participants(key, &name, cancel)
            .unwrap();
        let mut cursor = selected.cursor(|| draft.source_step(cancel)).unwrap();
        let mut actual = Vec::new();
        while let Some(key) = cursor.next(|| draft.source_step(cancel)).unwrap() {
            actual.push(key)
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn radio_query_uses_the_maintained_root_without_revisiting_unchanged_ancestors() {
        for depth in [1, 96] {
            let cancel = AtomicBool::new(false);
            let mut dom = Dom::new(limits()).unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let mut parent = batch.create_element_ns(&document, crate::Namespace::html(), "div");
            batch.insert_before(&parent, &document, None);
            for _ in 1..depth {
                let child = batch.create_element_ns(&document, crate::Namespace::html(), "div");
                batch.insert_before(&child, &parent, None);
                parent = child;
            }
            let radio = batch.create_element_ns(&document, crate::Namespace::html(), "input");
            batch.set_attribute(&radio, "type", "radio");
            batch.set_attribute(&radio, "name", "g");
            batch.insert_before(&radio, &parent, None);
            let created = dom.apply(batch, &cancel).unwrap();
            let radio = Key::Existing(created.created(&radio).unwrap().unwrap());
            let draft = source_draft(&dom, &cancel);
            let before = draft.source_work.load(Ordering::Relaxed);
            assert_radio_group(&draft, radio, &[radio], &cancel);
            let work = draft.source_work.load(Ordering::Relaxed) - before;
            assert!(
                work < 80,
                "selected singleton radio query spent {work} work at depth {depth}"
            );
        }
    }

    #[test]
    fn actual_parent_owner_name_and_type_changes_restore_the_immediately_previous_source_index() {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        let other = batch.create_element_ns(&document, crate::Namespace::html(), "form");
        let a = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        let b = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        for key in [&a, &b] {
            batch.set_attribute(key, "type", "radio");
            batch.set_attribute(key, "name", "g");
            batch.insert_before(key, &root, None);
        }
        batch.insert_before(&root, &document, None);
        let created = dom.apply(batch, &cancel).unwrap();
        let document = created.created(&document).unwrap().unwrap();
        let root = created.created(&root).unwrap().unwrap();
        let other = created.created(&other).unwrap().unwrap();
        let a = created.created(&a).unwrap().unwrap();
        let b = created.created(&b).unwrap().unwrap();
        let mut draft = source_draft(&dom, &cancel);
        let a = draft.resolve(a.into()).unwrap();
        let b = draft.resolve(b.into()).unwrap();
        let other = draft.resolve(other.into()).unwrap();
        assert_radio_group(&draft, a, &[a, b], &cancel);
        let outer = checkpoint(&mut draft, &cancel);
        draft
            .with_node_fields(b, &cancel, |fields| fields.control_owner = Some(other))
            .unwrap();
        assert_radio_group(&draft, a, &[a], &cancel);
        let inner = checkpoint(&mut draft, &cancel);
        draft.pre_insert(b, other, None, limits(), &cancel).unwrap();
        assert_radio_group(&draft, a, &[a], &cancel);
        assert_radio_group(&draft, b, &[b], &cancel);
        draft
            .set_attribute(b, None, &"name".into(), &"h".into(), limits(), &cancel)
            .unwrap();
        assert_radio_group(&draft, b, &[], &cancel);
        draft
            .set_attribute(
                b,
                None,
                &"type".into(),
                &"checkbox".into(),
                limits(),
                &cancel,
            )
            .unwrap();
        assert!(
            matches!(&draft.node(b).unwrap().kind,DomNodeKind::Element(element) if element.input_state().unwrap().input_type()==surgeist_html::HtmlInputType::Checkbox)
        );
        draft
            .set_attribute(b, None, &"name".into(), &"g".into(), limits(), &cancel)
            .unwrap();
        assert_radio_group(&draft, b, &[], &cancel);
        inner.restore(&mut draft);
        assert_radio_group(&draft, a, &[a], &cancel);
        let spent = draft.source_work.load(Ordering::Relaxed);
        outer.restore(&mut draft);
        assert_radio_group(&draft, a, &[a, b], &cancel);
        assert!(draft.source_work.load(Ordering::Relaxed) >= spent);
        assert_eq!(draft.node(a).unwrap().document, Key::Existing(document));
        assert_eq!(draft.node(a).unwrap().parent, Some(Key::Existing(root)));
    }

    #[test]
    fn a_private_control_survives_a_real_hidden_publication_then_exposes_and_rolls_back_after_publication()
     {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(limits()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let other_document = batch.create_document();
        let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        let prototype = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        batch.set_attribute(&prototype, "type", "radio");
        batch.set_attribute(&prototype, "name", "g");
        batch.insert_before(&prototype, &root, None);
        batch.insert_before(&root, &document, None);
        let created = dom.apply(batch, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let other_document = created.document(&other_document).unwrap();
        let root = created.created(&root).unwrap().unwrap();
        let prototype = created.created(&prototype).unwrap().unwrap();
        let mut draft = source_draft(&dom, &cancel);
        let kind = draft.node(Key::Existing(prototype)).unwrap().kind.clone();
        let target = draft
            .allocate(kind, Key::Existing(document.node()), limits(), &cancel)
            .unwrap();
        draft.mark_source_private(target, &cancel).unwrap();
        publish_phase(&mut draft, &mut dom, &cancel);
        assert!(draft.is_private(target));
        assert!(draft.node(target).is_ok());
        assert_eq!(dom.state.controls.inspect_facts().len(), 1);
        assert_radio_group(&draft, target, &[target], &cancel);
        draft.expose_source_subtree(target, &cancel).unwrap();
        draft
            .pre_insert(target, Key::Existing(root), None, limits(), &cancel)
            .unwrap();
        publish_phase(&mut draft, &mut dom, &cancel);
        let published = draft.published_id(target).unwrap();
        assert!(dom.snapshot().node(published).is_ok());
        assert_eq!(dom.state.controls.inspect_facts().len(), 2);
        assert_radio_group(&draft, target, &[Key::Existing(prototype), target], &cancel);
        // Private front doors accept the canonical alias of an original
        // Created Source key, just as Draft::node and the mutation barrier do.
        let alias_checkpoint = checkpoint(&mut draft, &cancel);
        draft
            .set_input_checked(Key::Existing(published), true, &cancel)
            .unwrap();
        assert!(
            draft.input_state(target).unwrap().flags.checkedness(),
            "canonical and original Source aliases must skip the same radio self"
        );
        alias_checkpoint.restore(&mut draft);
        let outer = checkpoint(&mut draft, &cancel);
        draft
            .adopt(
                target,
                Key::Existing(other_document.node()),
                limits(),
                &cancel,
            )
            .unwrap();
        assert_radio_group(&draft, target, &[target], &cancel);
        let inner = checkpoint(&mut draft, &cancel);
        draft
            .set_attribute(
                target,
                None,
                &"name".into(),
                &"other".into(),
                limits(),
                &cancel,
            )
            .unwrap();
        assert_radio_group(&draft, target, &[], &cancel);
        inner.restore(&mut draft);
        assert_radio_group(&draft, target, &[target], &cancel);
        let spent = draft.source_work.load(Ordering::Relaxed);
        outer.restore(&mut draft);
        assert_eq!(
            draft.node(target).unwrap().document,
            Key::Existing(document.node())
        );
        assert_radio_group(&draft, target, &[Key::Existing(prototype), target], &cancel);
        assert!(draft.source_work.load(Ordering::Relaxed) >= spent);
    }

    #[test]
    fn actual_event_reentry_preserves_callback_radio_groups_when_the_original_source_resumes() {
        use crate::{DomSourceEdit, DomSourceEventOutcome, DomSourceProgress, DomSourceRequest};
        let cancel = AtomicBool::new(false);
        let configured = limits().with_dispatch_custody(
            crate::DomDispatchCustodyLimits::new(8, 64, 2_000_000, 2_000_000, 1_000_000).unwrap(),
        );
        let mut dom = Dom::new(configured).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let a = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        let b = batch.create_element_ns(&document, crate::Namespace::html(), "input");
        for key in [&a, &b] {
            batch.set_attribute(key, "type", "radio");
            batch.set_attribute(key, "name", "g");
            batch.insert_before(key, &root, None);
        }
        batch.set_input_checked(&a, true);
        batch.insert_before(&dialog, &root, None);
        batch.insert_before(&root, &document, None);
        let created = dom.apply(batch, &cancel).unwrap();
        let dialog = created.created(&dialog).unwrap().unwrap();
        let a = created.created(&a).unwrap().unwrap();
        let b = created.created(&b).unwrap().unwrap();
        let executor = dom.bind_source_event_executor(&cancel).unwrap();
        let request = {
            let mut lease = dom
                .begin_source_edit(DomSourceEdit::show_dialog(dialog), source_limits(), &cancel)
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual beforetoggle observation")
            };
            request
        };
        let permit = executor.begin_dispatch(&dom, &request, &cancel).unwrap();
        let scope = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        let mut callback = DomBatch::new();
        callback.set_attribute(b, "name", "other");
        callback.set_input_checked(b, true);
        dom.apply(callback, &cancel).unwrap();
        let checked = |snapshot: &DomSnapshot, key| {
            snapshot
                .node(key)
                .unwrap()
                .as_input()
                .unwrap()
                .state()
                .flags()
                .checkedness()
        };
        assert!(checked(&dom.snapshot(), a));
        assert!(checked(&dom.snapshot(), b));
        let callback_stamp = dom.snapshot().stamp();
        drop(scope);
        let mut permit = Some(permit);
        let mut publication = Some(
            executor
                .seal_completed(&dom, &mut permit, DomSourceEventOutcome::Canceled, &cancel)
                .unwrap(),
        );
        {
            let mut lease = dom
                .resume_source_edit(request.operation(), source_limits(), &cancel)
                .unwrap();
            lease
                .accept_source_event(&request, &mut publication, &cancel)
                .unwrap();
            assert!(matches!(
                lease.advance(&cancel).unwrap(),
                DomSourceProgress::Complete(_)
            ));
        }
        assert!(permit.is_none());
        assert!(publication.is_none());
        assert_eq!(dom.snapshot().stamp(), callback_stamp);
        let draft = source_draft(&dom, &cancel);
        assert_radio_group(&draft, Key::Existing(a), &[Key::Existing(a)], &cancel);
        let mut normalize = DomBatch::new();
        normalize.set_input_checked(a, true);
        dom.apply(normalize, &cancel).unwrap();
        assert!(
            checked(&dom.snapshot(), b),
            "the callback's distinct actual group survives Source reentry"
        );
    }
}
