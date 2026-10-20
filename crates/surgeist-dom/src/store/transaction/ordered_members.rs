//! Shared ordinary positions and group-qualified ordered participants.
//! A node may own separate member records in several groups. Only the existing
//! ChildPosition map establishes ordinary topology; shadow edges are reciprocal.
use super::super::*;
use super::KindRead;
use super::{NodeRead, slot_index::SlotIndex};
use std::cmp::Ordering;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) struct ChildPosition {
    pub key: Key,
    pub parent: Key,
    pub ordinal: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Mode {
    Ordinary,
    ShadowIncluding,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::store) enum Edge {
    Shadow,
    Child(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) struct ParentLink {
    pub parent: Key,
    pub edge: Edge,
}

#[derive(Clone, Copy)]
enum Read<'a> {
    Positions,
    Draft(&'a Draft),
    State(&'a State, DomOwnerId),
    Projected(&'a Draft, &'a source::AllocationIds),
}
#[derive(Clone, Copy)]
pub(in crate::store) struct OrderView<'a> {
    positions: &'a SlotIndex,
    read: Read<'a>,
    fields: Option<(Key, Option<NodeRead<'a>>)>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct OrderPair<'a> {
    pub before: OrderView<'a>,
    pub after: OrderView<'a>,
}
impl<'a> OrderView<'a> {
    pub fn positions(positions: &'a SlotIndex) -> Self {
        Self {
            positions,
            read: Read::Positions,
            fields: None,
        }
    }
    pub fn draft(draft: &'a Draft, positions: &'a SlotIndex) -> Self {
        Self {
            positions,
            read: Read::Draft(draft),
            fields: None,
        }
    }
    pub fn changed(mut self, key: Key, node: NodeRead<'a>) -> Self {
        self.fields = Some((key, Some(node)));
        self
    }
    pub fn removed(mut self, key: Key) -> Self {
        self.fields = Some((key, None));
        self
    }
    pub fn state(state: &'a State, owner: DomOwnerId) -> Self {
        Self {
            positions: &state.slot_index,
            read: Read::State(state, owner),
            fields: None,
        }
    }
    pub fn projected(
        draft: &'a Draft,
        positions: &'a SlotIndex,
        ids: &'a source::AllocationIds,
    ) -> Self {
        Self {
            positions,
            read: Read::Projected(draft, ids),
            fields: None,
        }
    }
    pub fn normalize(self, key: Key) -> Key {
        match self.read {
            Read::Positions => key,
            Read::Draft(draft) => draft.live_key(key),
            Read::State(..) => key,
            Read::Projected(draft, ids) => match draft.live_key(key) {
                Key::Created(index) => ids.get(index).map(Key::Existing).unwrap_or(key),
                Key::Existing(id) => Key::Existing(id),
            },
        }
    }
    pub fn position(self, key: Key) -> Option<ChildPosition> {
        self.positions.position(self.normalize(key))
    }
    pub fn node(self, key: Key) -> Result<NodeRead<'a>, DomError> {
        match self.read {
            Read::Positions => Err(DomError::UnavailableCreation),
            Read::Draft(draft) | Read::Projected(draft, _) => {
                let key = match (self.read, key) {
                    (Read::Projected(_, ids), Key::Existing(id)) => {
                        ids.original(id).map(Key::Created).unwrap_or(key)
                    }
                    _ => key,
                };
                let key = draft.live_key(key);
                if let Some((changed, node)) = self.fields
                    && draft.live_key(changed) == key
                {
                    return node.ok_or(DomError::StaleNode);
                }
                draft.node(key)
            }
            Read::State(state, owner) => match key {
                Key::Existing(id) => Ok(NodeRead::canonical(state.node(owner, id)?)),
                Key::Created(_) => Err(DomError::UnavailableCreation),
            },
        }
    }
    pub fn parent(
        self,
        key: Key,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<ParentLink>, DomError> {
        step()?;
        let key = self.normalize(key);
        if let Some(position) = self.position(key) {
            return Ok(Some(ParentLink {
                parent: position.parent,
                edge: Edge::Child(position.ordinal),
            }));
        }
        if mode == Mode::ShadowIncluding {
            let node = self.node(key)?;
            if matches!(node.kind(), KindRead::ShadowRoot(_))
                && let Some(host) = node.host()
            {
                let reciprocal = match self.node(host) {
                    Ok(node) => node.shadow_root().map(|root| self.normalize(root)) == Some(key),
                    Err(DomError::StaleNode) => false,
                    Err(error) => return Err(error),
                };
                if reciprocal {
                    return Ok(Some(ParentLink {
                        parent: self.normalize(host),
                        edge: Edge::Shadow,
                    }));
                }
            }
        }
        Ok(None)
    }
    pub fn root(
        self,
        mut key: Key,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Key, DomError> {
        key = self.normalize(key);
        while let Some(parent) = self.parent(key, mode, step)? {
            key = parent.parent;
        }
        Ok(key)
    }
    pub fn descendant(
        self,
        ancestor: Key,
        mut key: Key,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        let ancestor = self.normalize(ancestor);
        key = self.normalize(key);
        loop {
            step()?;
            if key == ancestor {
                return Ok(true);
            }
            match self.parent(key, mode, step)? {
                Some(parent) => key = parent.parent,
                None => return Ok(false),
            }
        }
    }
    pub fn compare(
        self,
        mut a: Key,
        mut b: Key,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Ordering, DomError> {
        a = self.normalize(a);
        b = self.normalize(b);
        if a == b {
            return Ok(Ordering::Equal);
        }
        let depth = |mut key: Key,
                     step: &mut dyn FnMut() -> Result<(), DomError>|
         -> Result<usize, DomError> {
            let mut depth = 0usize;
            while let Some(parent) = self.parent(key, mode, step)? {
                key = parent.parent;
                depth = depth
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
            Ok(depth)
        };
        let mut da = depth(a, step)?;
        let mut db = depth(b, step)?;
        while da > db {
            a = self
                .parent(a, mode, step)?
                .ok_or(DomError::HierarchyRequest)?
                .parent;
            da -= 1;
            if a == b {
                return Ok(Ordering::Greater);
            }
        }
        while db > da {
            b = self
                .parent(b, mode, step)?
                .ok_or(DomError::HierarchyRequest)?
                .parent;
            db -= 1;
            if a == b {
                return Ok(Ordering::Less);
            }
        }
        loop {
            let pa = self
                .parent(a, mode, step)?
                .ok_or(DomError::HierarchyRequest)?;
            let pb = self
                .parent(b, mode, step)?
                .ok_or(DomError::HierarchyRequest)?;
            if pa.parent == pb.parent {
                return Ok(pa.edge.cmp(&pb.edge));
            }
            a = pa.parent;
            b = pb.parent;
        }
    }
}

impl Draft {
    pub(in crate::store) fn order_view(&self) -> OrderView<'_> {
        self.slot_index.order(self)
    }
}

pub(in crate::store) struct Edit<'a> {
    pub registry: &'a bank::CustodyRegistry,
    pub admit: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    pub step: &'a mut dyn FnMut() -> Result<(), DomError>,
}

pub(in crate::store) trait Qualified: Clone + Eq + Send + Sync + 'static {
    fn key(&self) -> Key;
    fn usage(&self) -> Result<custody::Usage, DomError> {
        Ok(custody::Usage::new(size_of::<Self>(), 0))
    }
}
impl Qualified for Key {
    fn key(&self) -> Key {
        *self
    }
}
impl Qualified for ChildPosition {
    fn key(&self) -> Key {
        self.key
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::store) struct Map<T> {
    pub existing: bank::Bank<T>,
    pub created: bank::Bank<T>,
}
impl<T> Default for Map<T> {
    fn default() -> Self {
        Self {
            existing: Default::default(),
            created: Default::default(),
        }
    }
}
#[derive(Clone)]
pub(in crate::store) struct MapCustody<T> {
    pub existing: bank::BankCustody<T>,
    pub created: bank::BankCustody<T>,
}
impl<T> Default for MapCustody<T> {
    fn default() -> Self {
        Self {
            existing: Default::default(),
            created: Default::default(),
        }
    }
}
impl<T: Qualified> Map<T> {
    pub fn get(&self, key: Key) -> Option<&T> {
        let value = match key {
            Key::Existing(id) => self.existing.get(id.slot),
            Key::Created(index) => self.created.get(index),
        }?;
        (value.key() == key).then_some(value)
    }
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.existing.values().chain(self.created.values())
    }
    pub fn change(
        &mut self,
        control: &mut MapCustody<T>,
        key: Key,
        value: Option<T>,
        edit: &mut Edit<'_>,
    ) -> Result<bool, DomError> {
        (edit.step)()?;
        if self.get(key) == value.as_ref() {
            return Ok(false);
        }
        let (bank, custody, index) = match key {
            Key::Existing(id) => (&mut self.existing, &mut control.existing, id.slot),
            Key::Created(index) => (&mut self.created, &mut control.created, index),
        };
        if bank.get(index).is_some_and(|old| old.key() != key) {
            return Err(DomError::StaleNode);
        }
        let (next, next_custody) = if let Some(value) = value {
            let usage = value.usage()?;
            let record =
                bank::ControlledRecord::new(value, usage, edit.registry, &mut *edit.admit)?;
            bank.set_controlled(
                custody,
                index,
                record,
                false,
                bank::Totals {
                    bytes: usage.bytes(),
                    units: usage.units(),
                    occurrences: 1,
                },
                edit.registry,
                &mut *edit.admit,
            )?
        } else {
            let (next, next_custody, removed) =
                bank.remove_controlled(custody, index, edit.registry, &mut *edit.admit)?;
            drop(removed);
            (next, next_custody)
        };
        drop(std::mem::replace(bank, next));
        *custody = next_custody;
        edit.registry.reconcile_releases();
        Ok(true)
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<MapCustody<T>, DomError> {
        let existing = self.existing.adopt_controlled(
            registry,
            |value| {
                step()?;
                value.usage()
            },
            &mut *admit,
        )?;
        let created = self.created.adopt_controlled(
            registry,
            |value| {
                step()?;
                value.usage()
            },
            admit,
        )?;
        Ok(MapCustody { existing, created })
    }
    #[cfg(test)]
    pub fn heap_usage(&self) -> Result<custody::Usage, DomError> {
        custody::Usage::new(self.existing.totals().bytes, self.existing.totals().units)
            .checked_add(custody::Usage::new(
                self.created.totals().bytes,
                self.created.totals().units,
            ))?
            .checked_add(custody::Usage::new(self.existing.index_charge().bytes, 0))?
            .checked_add(custody::Usage::new(self.created.index_charge().bytes, 0))
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let describe = |v: &T| {
            let u = v.usage()?;
            Ok(bank::Totals {
                bytes: u.bytes(),
                units: u.units(),
                occurrences: 1,
            })
        };
        let mut graph = self.existing.inspect_graph(describe)?;
        graph.merge(&self.created.inspect_graph(describe)?)?;
        Ok(graph)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemberId(usize);
/// The header is owned by its exact family group, together with a matching
/// Members version. Links are reached only through this header, never by node
/// identity alone; duplicate node membership in different groups is valid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct RootHeader(Option<MemberId>);
impl RootHeader {
    pub fn is_empty(self) -> bool {
        self.0.is_none()
    }
}
#[derive(Clone, Copy)]
pub(in crate::store) enum Membership {
    Insert(Key),
    Remove(Key),
}
#[derive(Clone, Copy)]
enum MemberOrder<'a> {
    Tree(OrderView<'a>, Mode),
    Identity,
}
impl MemberOrder<'_> {
    fn normalize(self, key: Key) -> Key {
        match self {
            Self::Tree(view, _) => view.normalize(key),
            Self::Identity => key,
        }
    }
    fn compare(
        self,
        a: Key,
        b: Key,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Ordering, DomError> {
        match self {
            Self::Tree(view, mode) => view.compare(a, b, mode, step),
            Self::Identity => {
                step()?;
                let coordinate = |key| match key {
                    Key::Existing(id) => (0, id.owner.get(), id.slot, id.generation),
                    Key::Created(index) => (1, 0, index, 0),
                };
                Ok(coordinate(a).cmp(&coordinate(b)))
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Link {
    key: Key,
    left: Option<MemberId>,
    right: Option<MemberId>,
    height: u8,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct Members {
    nodes: bank::Bank<Link>,
}

#[cfg(test)]
pub(in crate::store) struct InspectedMember {
    pub key: Key,
    pub left: Option<Key>,
    pub right: Option<Key>,
    pub parent: Option<Key>,
    pub height: u8,
}

#[cfg(test)]
impl Members {
    /// Inspect physical link order without consulting any production cursor or
    /// topology comparator. Tests derive expected membership independently.
    pub fn inspect_group(&self, root: RootHeader) -> Result<Vec<Key>, DomError> {
        fn visit(
            members: &Members,
            id: Option<MemberId>,
            remaining: &mut usize,
            output: &mut Vec<Key>,
        ) -> Result<(), DomError> {
            let Some(id) = id else { return Ok(()) };
            *remaining = remaining
                .checked_sub(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let value = members
                .nodes
                .get(id.0)
                .ok_or(DomError::RepresentationExhausted)?;
            visit(members, value.left, remaining, output)?;
            output.push(value.key);
            visit(members, value.right, remaining, output)
        }
        let mut output = Vec::new();
        let mut remaining = self.nodes.iter().count();
        visit(self, root.0, &mut remaining, &mut output)?;
        Ok(output)
    }
    pub fn inspect_root(&self, root: RootHeader) -> Option<Key> {
        root.0
            .map(|id| self.nodes.get(id.0).expect("raw ordered root").key)
    }
    pub fn inspect_summary(&self) -> (bank::Totals, bank::IndexCharge) {
        (self.nodes.totals(), self.nodes.index_charge())
    }
    pub fn inspect_tree(&self, root: RootHeader) -> Vec<InspectedMember> {
        let mut result = Vec::new();
        let mut pending = Vec::new();
        if let Some(root) = root.0 {
            pending.push((root, None));
        }
        while let Some((id, parent)) = pending.pop() {
            let value = self.nodes.get(id.0).expect("raw ordered edge");
            result.push(InspectedMember {
                key: value.key,
                left: value
                    .left
                    .map(|id| self.nodes.get(id.0).expect("raw left edge").key),
                right: value
                    .right
                    .map(|id| self.nodes.get(id.0).expect("raw right edge").key),
                parent,
                height: value.height,
            });
            if let Some(left) = value.left {
                pending.push((left, Some(value.key)))
            }
            if let Some(right) = value.right {
                pending.push((right, Some(value.key)))
            }
        }
        result
    }
}
#[derive(Clone, Default)]
pub(in crate::store) struct MembersCustody {
    nodes: bank::BankCustody<Link>,
}
pub(in crate::store) struct Change {
    pub members: Members,
    pub custody: MembersCustody,
    pub root: RootHeader,
    pub previous: bool,
}
impl Members {
    fn get(
        &self,
        id: MemberId,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Link, DomError> {
        step()?;
        self.nodes.get(id.0).copied().ok_or(DomError::StaleNode)
    }
    fn height(&self, id: Option<MemberId>) -> u8 {
        id.and_then(|id| self.nodes.get(id.0))
            .map_or(0, |v| v.height)
    }
    fn put(
        &mut self,
        custody: &mut MembersCustody,
        id: MemberId,
        link: Option<Link>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        (edit.step)()?;
        if self.nodes.get(id.0).copied() == link {
            return Ok(());
        }
        let (nodes, controls) = if let Some(link) = link {
            let record = bank::ControlledRecord::new(
                link,
                custody::Usage::new(size_of::<Link>(), 0),
                edit.registry,
                &mut *edit.admit,
            )?;
            self.nodes.set_controlled(
                &custody.nodes,
                id.0,
                record,
                false,
                bank::Totals {
                    bytes: size_of::<Link>(),
                    units: 0,
                    occurrences: 1,
                },
                edit.registry,
                &mut *edit.admit,
            )?
        } else {
            let (nodes, controls, removed) = self.nodes.remove_controlled(
                &custody.nodes,
                id.0,
                edit.registry,
                &mut *edit.admit,
            )?;
            drop(removed);
            (nodes, controls)
        };
        drop(std::mem::replace(&mut self.nodes, nodes));
        custody.nodes = controls;
        edit.registry.reconcile_releases();
        Ok(())
    }
    fn save(
        &mut self,
        custody: &mut MembersCustody,
        id: MemberId,
        mut link: Link,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        link.height = self
            .height(link.left)
            .max(self.height(link.right))
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        self.put(custody, id, Some(link), edit)
    }
    fn rotate_left(
        &mut self,
        custody: &mut MembersCustody,
        id: MemberId,
        edit: &mut Edit<'_>,
    ) -> Result<MemberId, DomError> {
        let mut a = self.get(id, edit.step)?;
        let b_id = a.right.ok_or(DomError::StaleNode)?;
        let mut b = self.get(b_id, edit.step)?;
        a.right = b.left;
        self.save(custody, id, a, edit)?;
        b.left = Some(id);
        self.save(custody, b_id, b, edit)?;
        Ok(b_id)
    }
    fn rotate_right(
        &mut self,
        custody: &mut MembersCustody,
        id: MemberId,
        edit: &mut Edit<'_>,
    ) -> Result<MemberId, DomError> {
        let mut a = self.get(id, edit.step)?;
        let b_id = a.left.ok_or(DomError::StaleNode)?;
        let mut b = self.get(b_id, edit.step)?;
        a.left = b.right;
        self.save(custody, id, a, edit)?;
        b.right = Some(id);
        self.save(custody, b_id, b, edit)?;
        Ok(b_id)
    }
    fn balance(
        &mut self,
        custody: &mut MembersCustody,
        id: MemberId,
        edit: &mut Edit<'_>,
    ) -> Result<MemberId, DomError> {
        let mut node = self.get(id, edit.step)?;
        let delta = i16::from(self.height(node.left)) - i16::from(self.height(node.right));
        if delta > 1 {
            let left = self.get(node.left.ok_or(DomError::StaleNode)?, edit.step)?;
            if self.height(left.left) < self.height(left.right) {
                node.left = Some(self.rotate_left(custody, node.left.unwrap(), edit)?);
                self.save(custody, id, node, edit)?;
            }
            self.rotate_right(custody, id, edit)
        } else if delta < -1 {
            let right = self.get(node.right.ok_or(DomError::StaleNode)?, edit.step)?;
            if self.height(right.right) < self.height(right.left) {
                node.right = Some(self.rotate_right(custody, node.right.unwrap(), edit)?);
                self.save(custody, id, node, edit)?;
            }
            self.rotate_left(custody, id, edit)
        } else {
            self.save(custody, id, node, edit)?;
            Ok(id)
        }
    }
    fn insert(
        &mut self,
        custody: &mut MembersCustody,
        root: Option<MemberId>,
        key: Key,
        order: MemberOrder<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<MemberId, DomError> {
        let Some(id) = root else {
            let id = MemberId(self.nodes.lowest_vacant().unwrap_or(self.nodes.len()));
            self.put(
                custody,
                id,
                Some(Link {
                    key,
                    left: None,
                    right: None,
                    height: 1,
                }),
                edit,
            )?;
            return Ok(id);
        };
        let mut node = self.get(id, edit.step)?;
        match order.compare(key, node.key, edit.step)? {
            Ordering::Equal => return Ok(id),
            Ordering::Less => node.left = Some(self.insert(custody, node.left, key, order, edit)?),
            Ordering::Greater => {
                node.right = Some(self.insert(custody, node.right, key, order, edit)?)
            }
        }
        self.save(custody, id, node, edit)?;
        self.balance(custody, id, edit)
    }
    fn minimum(
        &self,
        mut id: MemberId,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<MemberId, DomError> {
        loop {
            match self.get(id, step)?.left {
                Some(left) => id = left,
                None => return Ok(id),
            }
        }
    }
    fn remove(
        &mut self,
        custody: &mut MembersCustody,
        root: Option<MemberId>,
        key: Key,
        order: MemberOrder<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<Option<MemberId>, DomError> {
        let Some(id) = root else { return Ok(None) };
        let mut node = self.get(id, edit.step)?;
        match order.compare(key, node.key, edit.step)? {
            Ordering::Less => node.left = self.remove(custody, node.left, key, order, edit)?,
            Ordering::Greater => node.right = self.remove(custody, node.right, key, order, edit)?,
            Ordering::Equal => match (node.left, node.right) {
                (None, right) => {
                    self.put(custody, id, None, edit)?;
                    return Ok(right);
                }
                (left, None) => {
                    self.put(custody, id, None, edit)?;
                    return Ok(left);
                }
                (Some(_), Some(right)) => {
                    let successor = self.get(self.minimum(right, edit.step)?, edit.step)?.key;
                    node.key = successor;
                    node.right = self.remove(custody, node.right, successor, order, edit)?;
                }
            },
        }
        self.save(custody, id, node, edit)?;
        self.balance(custody, id, edit).map(Some)
    }
    pub fn first(
        &self,
        root: RootHeader,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        root.0
            .map(|id| {
                self.minimum(id, step)
                    .and_then(|id| self.get(id, step).map(|v| v.key))
            })
            .transpose()
    }
    fn contains_by(
        &self,
        root: RootHeader,
        key: Key,
        order: MemberOrder<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        let mut next = root.0;
        while let Some(id) = next {
            let node = self.get(id, step)?;
            match order.compare(key, node.key, step)? {
                Ordering::Equal => return Ok(node.key == order.normalize(key)),
                Ordering::Less => next = node.left,
                Ordering::Greater => next = node.right,
            }
        }
        Ok(false)
    }
    pub fn after(
        &self,
        root: RootHeader,
        key: Key,
        order: OrderView<'_>,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.after_by(root, key, MemberOrder::Tree(order, mode), step)
    }
    /// Enumerate a group whose members can have disconnected ordinary roots.
    /// Identity ordering is deliberately independent of tree topology.
    pub fn after_identity(
        &self,
        root: RootHeader,
        key: Key,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.after_by(root, key, MemberOrder::Identity, step)
    }
    fn after_by(
        &self,
        root: RootHeader,
        key: Key,
        order: MemberOrder<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let mut next = root.0;
        let mut result = None;
        while let Some(id) = next {
            let node = self.get(id, step)?;
            if order.compare(node.key, key, step)? == Ordering::Greater {
                result = Some(node.key);
                next = node.left
            } else {
                next = node.right
            }
        }
        Ok(result)
    }
    pub fn lower_bound(
        &self,
        root: RootHeader,
        key: Key,
        order: OrderView<'_>,
        mode: Mode,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let mut next = root.0;
        let mut result = None;
        while let Some(id) = next {
            let node = self.get(id, step)?;
            if order.compare(node.key, key, mode, step)? != Ordering::Less {
                result = Some(node.key);
                next = node.left
            } else {
                next = node.right
            }
        }
        Ok(result)
    }
    pub fn change(
        &self,
        custody: &MembersCustody,
        root: RootHeader,
        membership: Membership,
        order: OrderView<'_>,
        mode: Mode,
        edit: Edit<'_>,
    ) -> Result<Change, DomError> {
        self.change_by(
            custody,
            root,
            membership,
            MemberOrder::Tree(order, mode),
            edit,
        )
    }
    /// Change a group ordered solely by exact generation-qualified identity.
    pub fn change_identity(
        &self,
        custody: &MembersCustody,
        root: RootHeader,
        membership: Membership,
        edit: Edit<'_>,
    ) -> Result<Change, DomError> {
        self.change_by(custody, root, membership, MemberOrder::Identity, edit)
    }
    fn change_by(
        &self,
        custody: &MembersCustody,
        root: RootHeader,
        membership: Membership,
        order: MemberOrder<'_>,
        mut edit: Edit<'_>,
    ) -> Result<Change, DomError> {
        let (key, present) = match membership {
            Membership::Insert(key) => (key, true),
            Membership::Remove(key) => (key, false),
        };
        let previous = self.contains_by(root, key, order, edit.step)?;
        let mut next = Change {
            members: self.clone(),
            custody: custody.clone(),
            root,
            previous,
        };
        if previous != present {
            next.root = RootHeader(if present {
                Some(
                    next.members
                        .insert(&mut next.custody, root.0, key, order, &mut edit)?,
                )
            } else {
                next.members
                    .remove(&mut next.custody, root.0, key, order, &mut edit)?
            });
        }
        Ok(next)
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<MembersCustody, DomError> {
        Ok(MembersCustody {
            nodes: self.nodes.adopt_controlled(
                registry,
                |_| {
                    step()?;
                    Ok(custody::Usage::new(size_of::<Link>(), 0))
                },
                admit,
            )?,
        })
    }
    #[cfg(test)]
    pub fn heap_usage(&self) -> Result<custody::Usage, DomError> {
        custody::Usage::new(self.nodes.totals().bytes, 0)
            .checked_add(custody::Usage::new(self.nodes.index_charge().bytes, 0))
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        self.nodes.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Link>(),
                units: 0,
                occurrences: 1,
            })
        })
    }
}
