//! Exact named Slot groups and direct named-host slottables. Names are cached
//! only for actual Slot/host relationships; unchanged ordinary nodes add none.
use super::super::*;
use super::exact_forest::{self, Forest, KeyView, Suffix};
use super::ordered_members::{
    Edit, Map, MapCustody, Members, MembersCustody, Membership, Mode, OrderPair, OrderView,
    Qualified, RootHeader,
};
use super::{NodeChange, RelationChange};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Name {
    scope: Key,
    value: Option<DocumentString>,
    usage: custody::Usage,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Fact {
    key: Key,
    slot: Option<Name>,
    light: Option<Name>,
}
impl Qualified for Fact {
    fn key(&self) -> Key {
        self.key
    }
    fn usage(&self) -> Result<custody::Usage, DomError> {
        let mut usage = custody::Usage::new(size_of::<Self>(), 0);
        for name in [&self.slot, &self.light].into_iter().flatten() {
            usage = usage.checked_add(name.usage)?;
        }
        Ok(usage)
    }
}
impl Fact {
    fn name(&self, kind: usize) -> Option<&Name> {
        if kind == 0 {
            self.slot.as_ref()
        } else {
            self.light.as_ref()
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Group {
    representative: Key,
    members: RootHeader,
}
#[derive(Clone, Debug, Default)]
pub(in crate::store) struct Index {
    facts: Map<Fact>,
    forest: Forest<Group>,
    roots: [exact_forest::Root; 2],
    members: Members,
}
#[derive(Clone, Default)]
pub(in crate::store) struct Custody {
    facts: MapCustody<Fact>,
    forest: exact_forest::Custody<Group>,
    members: MembersCustody,
}
#[derive(Clone, Default)]
struct Version {
    value: Index,
    dirty: Map<Key>,
    custody: Custody,
    dirty_custody: MapCustody<Key>,
}
pub(in crate::store) struct Names {
    base: Index,
    base_custody: Custody,
    current: Version,
}
pub(in crate::store) struct Plan {
    next: Version,
}
pub(in crate::store) struct Undo {
    before: Version,
}
pub(in crate::store) struct Finished {
    pub value: Index,
    pub custody: Custody,
}
#[cfg(test)]
pub(in crate::store) struct NameInspection {
    pub scope: Key,
    pub value: Option<DocumentString>,
}
#[cfg(test)]
pub(in crate::store) struct FactInspection {
    pub key: Key,
    pub slot: Option<NameInspection>,
    pub light: Option<NameInspection>,
}
#[cfg(test)]
pub(in crate::store) struct GroupInspection {
    pub kind: usize,
    pub scope: Key,
    pub value: Option<DocumentString>,
    pub members: Vec<Key>,
}
impl Plan {
    pub fn commit(self, names: &mut Names) -> Undo {
        Undo {
            before: std::mem::replace(&mut names.current, self.next),
        }
    }
}
impl Undo {
    pub fn restore(self, names: &mut Names) {
        names.current = self.before
    }
}
#[derive(Default)]
struct Keys {
    values: Map<Key>,
    custody: MapCustody<Key>,
}
struct Reconcile(bank::CustodyRegistry);
impl Drop for Reconcile {
    fn drop(&mut self) {
        self.0.reconcile_releases()
    }
}

fn words(key: Key) -> [u64; 3] {
    match key {
        Key::Existing(id) => [0, id.slot as u64, id.generation],
        Key::Created(index) => [1, index as u64, 0],
    }
}
fn encoded(name: &Name) -> KeyView<'_, 3> {
    KeyView {
        words: words(name.scope),
        used: 3,
        suffix: Suffix::Utf16(name.value.as_ref().map_or(&[], DocumentString::code_units)),
    }
}
fn read_name(
    kind: &DomNodeKind,
    attribute: &str,
    cached: Option<&Name>,
    reuse: bool,
    step: &mut dyn FnMut() -> Result<(), DomError>,
) -> Result<(Option<DocumentString>, custody::Usage), DomError> {
    if reuse && let Some(cached) = cached {
        return Ok((cached.value.clone(), cached.usage));
    }
    let name = crate::slot::name_admitted(kind, attribute, &mut || step())?
        .filter(|name| !name.is_empty())
        .cloned();
    let mut payload = source::storage::Payload::default();
    if let Some(name) = &name {
        payload.string(name)?;
    }
    Ok((name, custody::Usage::new(payload.bytes, payload.units)))
}
fn fact(
    order: OrderView<'_>,
    key: Key,
    cached: Option<&Fact>,
    reuse: bool,
    step: &mut dyn FnMut() -> Result<(), DomError>,
) -> Result<Option<Fact>, DomError> {
    let key = order.normalize(key);
    let node = order.node(key)?;
    let slot = if crate::slot::is_slot(node.kind) {
        let (value, usage) = read_name(
            node.kind,
            "name",
            cached.and_then(|v| v.slot.as_ref()),
            reuse,
            step,
        )?;
        Some(Name {
            scope: order.root(key, Mode::Ordinary, step)?,
            value,
            usage,
        })
    } else {
        None
    };
    let mut light = None;
    if crate::slot::is_slottable(node.kind)
        && let Some(position) = order.position(key)
    {
        let parent = order.node(position.parent)?;
        if let Some(shadow) = parent.shadow_root {
            let root = order.node(shadow)?;
            if matches!(root.kind,DomNodeKind::ShadowRoot(state) if state.slot_assignment()==crate::HtmlSlotAssignment::Named)
                && root.host.map(|host| order.normalize(host)) == Some(position.parent)
            {
                let (value, usage) = read_name(
                    node.kind,
                    "slot",
                    cached.and_then(|v| v.light.as_ref()),
                    reuse,
                    step,
                )?;
                light = Some(Name {
                    scope: position.parent,
                    value,
                    usage,
                });
            }
        }
    }
    Ok((slot.is_some() || light.is_some()).then_some(Fact { key, slot, light }))
}
impl Index {
    fn membership(
        &mut self,
        custody: &mut Custody,
        value: &Fact,
        source: &Map<Fact>,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        for kind in 0..2 {
            let Some(name) = value.name(kind) else {
                continue;
            };
            let resolve = |group: Group| {
                source
                    .get(group.representative)
                    .and_then(|fact| fact.name(kind))
                    .map(encoded)
                    .ok_or(DomError::RepresentationExhausted)
            };
            let previous =
                self.forest
                    .lookup(self.roots[kind], &encoded(name), resolve, edit.step)?;
            let header = previous.map_or_else(RootHeader::default, |group| group.members);
            let changed = self.members.change(
                &custody.members,
                header,
                if present {
                    Membership::Insert(value.key)
                } else {
                    Membership::Remove(value.key)
                },
                order,
                Mode::Ordinary,
                Edit {
                    registry: edit.registry,
                    admit: edit.admit,
                    step: edit.step,
                },
            )?;
            self.members = changed.members;
            custody.members = changed.custody;
            let next = if let Some(representative) = self.members.first(changed.root, edit.step)? {
                Some(Group {
                    representative,
                    members: changed.root,
                })
            } else {
                None
            };
            let changed = self.forest.change(
                &custody.forest,
                self.roots[kind],
                &encoded(name),
                next,
                resolve,
                exact_forest::Edit {
                    registry: edit.registry,
                    admit: edit.admit,
                    step: edit.step,
                },
            )?;
            self.forest = changed.forest;
            custody.forest = changed.custody;
            self.roots[kind] = changed.root;
        }
        Ok(())
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Custody, DomError> {
        Ok(Custody {
            facts: self.facts.adopt_custody(registry, admit, step)?,
            forest: self.forest.adopt_custody(registry, admit, step)?,
            members: self.members.adopt_custody(registry, admit, step)?,
        })
    }
    fn first_name(
        &self,
        scope: Key,
        value: Option<&DocumentString>,
        kind: usize,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let query = KeyView {
            words: words(scope),
            used: 3,
            suffix: Suffix::Utf16(value.map_or(&[], DocumentString::code_units)),
        };
        let group = self.forest.lookup(
            self.roots[kind],
            &query,
            |group| {
                self.facts
                    .get(group.representative)
                    .and_then(|fact| fact.name(kind))
                    .map(encoded)
                    .ok_or(DomError::RepresentationExhausted)
            },
            step,
        )?;
        group
            .map(|group| self.members.first(group.members, step))
            .transpose()
            .map(Option::flatten)
    }
    pub fn named_slot(
        &self,
        root: Key,
        node: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        #[cfg(test)]
        let mut observation =
            scale_metrics::ENABLED.then(crate::slot::discovery_observation::Lookup::named);
        let mut step = || {
            work()?;
            #[cfg(test)]
            if scale_metrics::ENABLED
                && let Some(observation) = &mut observation
            {
                observation.read_candidate();
            }
            Ok(())
        };
        step()?;
        let fact = self
            .facts
            .get(node)
            .and_then(|fact| fact.light.as_ref())
            .ok_or(DomError::RepresentationExhausted)?;
        self.first_name(root, fact.value.as_ref(), 0, &mut step)
    }
    pub fn visit_light(
        &self,
        slot: Key,
        host: Key,
        order: OrderView<'_>,
        work: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(Key) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        #[cfg(test)]
        let mut observation =
            scale_metrics::ENABLED.then(crate::slot::discovery_observation::Lookup::host);
        let mut step = || {
            work()?;
            #[cfg(test)]
            if scale_metrics::ENABLED
                && let Some(observation) = &mut observation
            {
                observation.read_candidate();
            }
            Ok(())
        };
        step()?;
        let name = self
            .facts
            .get(slot)
            .and_then(|fact| fact.slot.as_ref())
            .ok_or(DomError::RepresentationExhausted)?;
        if self.first_name(name.scope, name.value.as_ref(), 0, &mut step)? != Some(slot) {
            return Ok(());
        }
        let query = KeyView {
            words: words(host),
            used: 3,
            suffix: Suffix::Utf16(name.value.as_ref().map_or(&[], DocumentString::code_units)),
        };
        let group = self.forest.lookup(
            self.roots[1],
            &query,
            |group| {
                self.facts
                    .get(group.representative)
                    .and_then(|fact| fact.light.as_ref())
                    .map(encoded)
                    .ok_or(DomError::RepresentationExhausted)
            },
            &mut step,
        )?;
        let Some(group) = group else { return Ok(()) };
        let mut current = self.members.first(group.members, &mut step)?;
        while let Some(key) = current {
            step()?;
            visit(key)?;
            current = self
                .members
                .after(group.members, key, order, Mode::Ordinary, &mut step)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        // Physical descriptors deliberately bypass Name.usage and the record
        // cache: the oracle owns the expected receipt calculation.
        let describe = |fact: &Fact| {
            let mut bytes = size_of::<Fact>();
            let mut units = 0usize;
            for name in [&fact.slot, &fact.light].into_iter().flatten() {
                if let Some(value) = &name.value {
                    let capacity = value.shared_code_units().capacity();
                    bytes = bytes
                        .checked_add(size_of::<Vec<u16>>())
                        .and_then(|v| {
                            capacity
                                .checked_mul(size_of::<u16>())
                                .and_then(|n| v.checked_add(n))
                        })
                        .ok_or(DomError::RepresentationExhausted)?;
                    units = units
                        .checked_add(capacity)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
            Ok(bank::Totals {
                bytes,
                units,
                occurrences: 1,
            })
        };
        let mut graph = self.facts.existing.inspect_graph(describe)?;
        graph.merge(&self.facts.created.inspect_graph(describe)?)?;
        graph.merge(&self.forest.inspect_graph()?)?;
        graph.merge(&self.members.inspect_graph()?)?;
        Ok(graph)
    }
    #[cfg(test)]
    pub fn inspect_summaries(&self) -> Vec<(bank::Totals, bank::IndexCharge)> {
        vec![
            (
                self.facts.existing.totals(),
                self.facts.existing.index_charge(),
            ),
            (
                self.facts.created.totals(),
                self.facts.created.index_charge(),
            ),
            self.forest.inspect_summaries(),
            self.members.inspect_summary(),
        ]
    }
    #[cfg(test)]
    pub fn inspect_facts(&self) -> Vec<FactInspection> {
        let name = |value: &Name| NameInspection {
            scope: value.scope,
            value: value.value.clone(),
        };
        self.facts
            .values()
            .map(|fact| FactInspection {
                key: fact.key,
                slot: fact.slot.as_ref().map(name),
                light: fact.light.as_ref().map(name),
            })
            .collect()
    }
    #[cfg(test)]
    pub fn inspect_groups(&self) -> Result<Vec<GroupInspection>, DomError> {
        let mut output = Vec::new();
        for kind in 0..2 {
            let mut pending = self.roots[kind].into_iter().collect::<Vec<_>>();
            while let Some(index) = pending.pop() {
                match self
                    .forest
                    .node(index)
                    .ok_or(DomError::RepresentationExhausted)?
                {
                    exact_forest::Node::Branch { children, .. } => pending.extend(children),
                    exact_forest::Node::Leaf(group) => {
                        let name = self
                            .facts
                            .get(group.representative)
                            .and_then(|fact| fact.name(kind))
                            .ok_or(DomError::RepresentationExhausted)?;
                        output.push(GroupInspection {
                            kind,
                            scope: name.scope,
                            value: name.value.clone(),
                            members: self.members.inspect_group(group.members)?,
                        });
                    }
                }
            }
        }
        Ok(output)
    }
}
impl Names {
    pub fn capture(base: &Index, custody: &Custody) -> Self {
        Self {
            base: base.clone(),
            base_custody: custody.clone(),
            current: Version {
                value: base.clone(),
                custody: custody.clone(),
                ..Default::default()
            },
        }
    }
    pub fn mark_published(&mut self, value: &Index, custody: &Custody) {
        self.base = value.clone();
        self.base_custody = custody.clone();
        self.current.dirty = Default::default();
        self.current.dirty_custody = Default::default();
    }
    pub fn named_slot(
        &self,
        root: Key,
        node: Key,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current.value.named_slot(root, node, step)
    }
    pub fn visit_light(
        &self,
        slot: Key,
        host: Key,
        order: OrderView<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
        visit: &mut dyn FnMut(Key) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.current
            .value
            .visit_light(slot, host, order, step, visit)
    }
    fn update(
        &self,
        draft: &Draft,
        keys: Keys,
        order: OrderPair<'_>,
        override_node: Option<NodeChange<'_>>,
        reordered: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut next = self.current.clone();
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        let before = self.current.value.facts.clone();
        for &key in keys.values.values() {
            if let Some(old) = before.get(key) {
                next.value.membership(
                    &mut next.custody,
                    old,
                    &before,
                    false,
                    order.before,
                    &mut edit,
                )?;
            }
        }
        let mut changed = false;
        for &key in keys.values.values() {
            let node_change = override_node.filter(|change| draft.live_key(change.key) == key);
            let reuse = match node_change {
                Some(NodeChange {
                    before: Some(before),
                    after: Some(after),
                    ..
                }) => match (before.kind, after.kind) {
                    (DomNodeKind::Element(a), DomNodeKind::Element(b)) => {
                        a.attributes_backing().same_storage(b.attributes_backing())
                    }
                    _ => false,
                },
                None => true,
                _ => false,
            };
            let value = if node_change.is_some_and(|change| change.after.is_none()) {
                None
            } else {
                fact(order.after, key, before.get(key), reuse, edit.step)?
            };
            changed |= next
                .value
                .facts
                .change(&mut next.custody.facts, key, value, &mut edit)?;
            next.dirty
                .change(&mut next.dirty_custody, key, Some(key), &mut edit)?;
        }
        let facts = next.value.facts.clone();
        for &key in keys.values.values() {
            if let Some(value) = facts.get(key) {
                next.value.membership(
                    &mut next.custody,
                    value,
                    &facts,
                    true,
                    order.after,
                    &mut edit,
                )?;
            }
        }
        Ok(
            (changed || (reordered && keys.values.values().next().is_some()))
                .then_some(Plan { next }),
        )
    }
    pub fn prepare_node(
        &self,
        draft: &Draft,
        change: NodeChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        let key = draft.live_key(change.key);
        let shadow_changed = change.before.and_then(|n| n.shadow_root)
            != change.after.and_then(|n| n.shadow_root)
            || change.before.and_then(|n| n.host) != change.after.and_then(|n| n.host);
        let indexed = self.current.value.facts.get(key).is_some();
        let relevant = change
            .after
            .is_some_and(|node| crate::slot::is_slot(node.kind))
            || order.after.position(key).is_some_and(|p| {
                order
                    .after
                    .node(p.parent)
                    .is_ok_and(|node| node.shadow_root.is_some())
            });
        if !indexed && !relevant && !shadow_changed {
            return Ok(None);
        }
        let mut keys = Keys::default();
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        keys.values
            .change(&mut keys.custody, key, Some(key), &mut edit)?;
        // A host/root attachment is the selected relationship that creates or
        // removes host-name groups. Its direct children are processed once.
        let before = change.before;
        let after = change.after;
        if shadow_changed {
            for node in [before, after].into_iter().flatten() {
                if node.shadow_root.is_some() {
                    for child in node.children.iter() {
                        let key = draft.live_key(child);
                        keys.values
                            .change(&mut keys.custody, key, Some(key), &mut edit)?;
                    }
                }
                if matches!(node.kind, DomNodeKind::ShadowRoot(_))
                    && let Some(host) = node.host
                {
                    // Destroy removes every ordinary child before retiring
                    // its closure in preorder. A host can therefore already
                    // be retired when its ShadowRoot loses this last link.
                    let host = match draft.node(host) {
                        Ok(host) => host,
                        Err(DomError::StaleNode) => continue,
                        Err(error) => return Err(error),
                    };
                    for child in host.children.iter() {
                        let key = draft.live_key(child);
                        keys.values
                            .change(&mut keys.custody, key, Some(key), &mut edit)?;
                    }
                }
            }
        }
        self.update(draft, keys, order, Some(change), false, cancel)
    }
    pub fn prepare_relation(
        &self,
        draft: &Draft,
        change: RelationChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        if !matches!(change.field, super::RelationField::Children) {
            return Ok(None);
        }
        let mut keys = Keys::default();
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        let end = change
            .index
            .checked_add(change.removed)
            .ok_or(DomError::IndexSize)?;
        for key in (change.index..end)
            .filter_map(|i| change.before.get(i))
            .chain(change.added.iter().copied())
        {
            let key = draft.live_key(key);
            let light = self
                .current
                .value
                .facts
                .get(key)
                .is_some_and(|fact| fact.light.is_some())
                || order.after.position(key).is_some_and(|p| {
                    order
                        .after
                        .node(p.parent)
                        .is_ok_and(|node| node.shadow_root.is_some())
                });
            let root = order.before.root(key, Mode::Ordinary, edit.step)?;
            let mut current = draft.slot_index.index().lower_bound(root, key, edit.step)?;
            let contains_slot = match current {
                Some(member) => order
                    .before
                    .descendant(key, member, Mode::Ordinary, edit.step)?,
                None => false,
            };
            if !light && !contains_slot {
                continue;
            }
            keys.values
                .change(&mut keys.custody, key, Some(key), &mut edit)?;
            while let Some(member) = current {
                if !order
                    .before
                    .descendant(key, member, Mode::Ordinary, edit.step)?
                {
                    break;
                }
                keys.values
                    .change(&mut keys.custody, member, Some(member), &mut edit)?;
                current = draft.slot_index.index().next(root, member, edit.step)?;
            }
        }
        self.update(draft, keys, order, None, true, cancel)
    }
    pub fn prepare_visibility(
        &self,
        draft: &Draft,
        key: Key,
        was: bool,
        is: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        let key = draft.live_key(key);
        if was == is || self.current.value.facts.get(key).is_none() {
            return Ok(None);
        }
        let mut next = self.current.clone();
        next.dirty.change(
            &mut next.dirty_custody,
            key,
            Some(key),
            &mut Edit {
                registry: draft.custody_registry(),
                admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
                step: &mut || draft.source_step(cancel),
            },
        )?;
        Ok(Some(Plan { next }))
    }
    pub fn finish(
        &self,
        draft: &Draft,
        mapper: &impl Fn(Key) -> Option<NodeId>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let mut next = Version {
            value: self.base.clone(),
            custody: self.base_custody.clone(),
            ..Default::default()
        };
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        for &key in self.current.dirty.values() {
            if let Some(id) = mapper(key)
                && let Some(old) = self.base.facts.get(Key::Existing(id))
            {
                next.value.membership(
                    &mut next.custody,
                    old,
                    &self.base.facts,
                    false,
                    order.before,
                    &mut edit,
                )?;
            }
        }
        for &key in self.current.dirty.values() {
            if let Some(id) = mapper(key) {
                let public = Key::Existing(id);
                let value = if let Some(current) = self.current.value.facts.get(draft.live_key(key))
                {
                    fact(order.after, public, Some(current), true, edit.step)?
                } else {
                    None
                };
                next.value
                    .facts
                    .change(&mut next.custody.facts, public, value, &mut edit)?;
            }
        }
        let facts = next.value.facts.clone();
        for &key in self.current.dirty.values() {
            if let Some(id) = mapper(key)
                && let Some(value) = facts.get(Key::Existing(id))
            {
                next.value.membership(
                    &mut next.custody,
                    value,
                    &facts,
                    true,
                    order.after,
                    &mut edit,
                )?;
            }
        }
        Ok(Finished {
            value: next.value,
            custody: next.custody,
        })
    }
}
