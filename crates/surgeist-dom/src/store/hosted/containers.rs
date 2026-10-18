//! All actual navigable containers in canonical shadow-including tree order.
//! Null content and graph-parent mismatch remain participants; the query owner
//! qualifies those facts against the current hosted graph.
use super::super::*;
use transaction::ordered_members::{
    Edit, Map, MapCustody, Members, MembersCustody, Membership, Mode, OrderPair, OrderView,
    Qualified, RootHeader,
};
use transaction::{NodeChange, NodeRead, RelationChange, RelationField};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Root {
    key: Key,
    member: RootHeader,
}
impl Qualified for Root {
    fn key(&self) -> Key {
        self.key
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct ContainerIndex {
    roots: Map<Root>,
    members: Members,
    facts: Map<Key>,
}
#[derive(Clone, Default)]
pub(in crate::store) struct ContainerCustody {
    roots: MapCustody<Root>,
    members: MembersCustody,
    facts: MapCustody<Key>,
}
#[derive(Clone, Default)]
struct Version {
    index: ContainerIndex,
    custody: ContainerCustody,
    dirty: Map<Key>,
    dirty_custody: MapCustody<Key>,
}
pub(in crate::store) struct Containers {
    base: ContainerIndex,
    base_custody: ContainerCustody,
    current: Version,
}
pub(in crate::store) struct ContainerPlan {
    next: Version,
}
pub(in crate::store) struct ContainerUndo {
    before: Version,
}
pub(in crate::store) struct ContainerFinished {
    pub index: ContainerIndex,
    pub custody: ContainerCustody,
}
impl ContainerPlan {
    pub fn commit(self, target: &mut Containers) -> ContainerUndo {
        ContainerUndo {
            before: std::mem::replace(&mut target.current, self.next),
        }
    }
}
impl ContainerUndo {
    pub fn restore(self, target: &mut Containers) {
        target.current = self.before;
    }
}
fn participant(node: NodeRead<'_>) -> bool {
    matches!(node.kind, DomNodeKind::Element(element) if element.child_navigable().is_some())
}
fn hosted_work<T>(
    work: &crate::hosted::Work,
    cancel: &AtomicBool,
    run: impl FnOnce(&mut dyn FnMut() -> Result<(), DomError>) -> Result<T, DomError>,
) -> Result<T, crate::hosted::DomHostedError> {
    let mut original = None;
    let result = run(&mut || {
        work.step(cancel).map_err(|error| {
            original = Some(error);
            DomError::Cancelled
        })
    });
    result.map_err(|error| original.unwrap_or_else(|| error.into()))
}
impl ContainerIndex {
    fn header(&self, root: Key) -> RootHeader {
        self.roots
            .get(root)
            .map_or_else(RootHeader::default, |r| r.member)
    }
    pub fn first(
        &self,
        root: Key,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        step()?;
        self.members.first(self.header(root), step)
    }
    pub fn after(
        &self,
        root: Key,
        key: Key,
        order: OrderView<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        step()?;
        self.members
            .after(self.header(root), key, order, Mode::ShadowIncluding, step)
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<ContainerCustody, DomError> {
        Ok(ContainerCustody {
            roots: self.roots.adopt_custody(registry, admit, step)?,
            members: self.members.adopt_custody(registry, admit, step)?,
            facts: self.facts.adopt_custody(registry, admit, step)?,
        })
    }
    pub fn first_hosted(
        &self,
        root: Key,
        work: &crate::hosted::Work,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, crate::hosted::DomHostedError> {
        hosted_work(work, cancel, |step| self.first(root, step))
    }
    pub fn after_hosted(
        &self,
        root: Key,
        key: Key,
        order: OrderView<'_>,
        work: &crate::hosted::Work,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, crate::hosted::DomHostedError> {
        hosted_work(work, cancel, |step| self.after(root, key, order, step))
    }
    #[cfg(test)]
    pub fn retained_usage(&self) -> Result<custody::Usage, DomError> {
        self.roots
            .heap_usage()?
            .checked_add(self.members.heap_usage()?)?
            .checked_add(self.facts.heap_usage()?)
    }
    fn change(
        &mut self,
        custody: &mut ContainerCustody,
        key: Key,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<bool, DomError> {
        let root = order.root(key, Mode::ShadowIncluding, edit.step)?;
        let changed = self.members.change(
            &custody.members,
            self.header(root),
            if present {
                Membership::Insert(key)
            } else {
                Membership::Remove(key)
            },
            order,
            Mode::ShadowIncluding,
            Edit {
                registry: edit.registry,
                admit: edit.admit,
                step: edit.step,
            },
        )?;
        let different = changed.previous != present;
        self.members = changed.members;
        custody.members = changed.custody;
        if different {
            self.roots.change(
                &mut custody.roots,
                root,
                (!changed.root.is_empty()).then_some(Root {
                    key: root,
                    member: changed.root,
                }),
                edit,
            )?;
        }
        Ok(different)
    }
}
impl Version {
    fn dirty(&mut self, key: Key, edit: &mut Edit<'_>) -> Result<(), DomError> {
        self.dirty
            .change(&mut self.dirty_custody, key, Some(key), edit)?;
        Ok(())
    }
    fn fact(
        &mut self,
        key: Key,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        if self.index.facts.get(key).is_some() == present {
            return Ok(());
        }
        self.index
            .change(&mut self.custody, key, present, order, edit)?;
        self.index
            .facts
            .change(&mut self.custody.facts, key, present.then_some(key), edit)?;
        self.dirty(key, edit)
    }
    fn relocate_selected(
        &mut self,
        selected: &Map<Key>,
        order: OrderPair<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<bool, DomError> {
        for &key in selected.values() {
            self.index
                .change(&mut self.custody, key, false, order.before, edit)?;
        }
        for &key in selected.values() {
            self.index
                .change(&mut self.custody, key, true, order.after, edit)?;
            self.dirty(key, edit)?;
        }
        Ok(selected.values().next().is_some())
    }
}
fn select(
    source: &ContainerIndex,
    subtree: Key,
    order: OrderView<'_>,
    selected: &mut Map<Key>,
    custody: &mut MapCustody<Key>,
    edit: &mut Edit<'_>,
) -> Result<(), DomError> {
    // Destroy removes ordinary child edges and participant facts before the
    // incoming stale shadow reference is cleared. That later cleanup has no
    // surviving subtree participant to relocate; other read failures propagate.
    match order.node(subtree) {
        Err(DomError::StaleNode) if source.facts.get(order.normalize(subtree)).is_none() => {
            return Ok(());
        }
        Err(error) => return Err(error),
        Ok(_) => {}
    }
    let root = order.root(subtree, Mode::ShadowIncluding, edit.step)?;
    let header = source.header(root);
    let mut cursor =
        source
            .members
            .lower_bound(header, subtree, order, Mode::ShadowIncluding, edit.step)?;
    while let Some(key) = cursor {
        if !order.descendant(subtree, key, Mode::ShadowIncluding, edit.step)? {
            break;
        }
        cursor = source.after(root, key, order, edit.step)?;
        selected.change(custody, key, Some(key), edit)?;
    }
    Ok(())
}
impl Containers {
    pub fn capture(index: &ContainerIndex, custody: &ContainerCustody) -> Self {
        Self {
            base: index.clone(),
            base_custody: custody.clone(),
            current: Version {
                index: index.clone(),
                custody: custody.clone(),
                ..Default::default()
            },
        }
    }
    fn prepare(
        &self,
        draft: &Draft,
        cancel: &AtomicBool,
        apply: impl FnOnce(&mut Version, &mut Edit<'_>) -> Result<bool, DomError>,
    ) -> Result<Option<ContainerPlan>, DomError> {
        let mut next = self.current.clone();
        let result = apply(
            &mut next,
            &mut Edit {
                registry: draft.custody_registry(),
                admit: &mut |u| draft.admit_custody_usage(u, cancel),
                step: &mut || draft.source_step(cancel),
            },
        );
        match result {
            Ok(changed) => Ok(changed.then_some(ContainerPlan { next })),
            Err(error) => {
                drop(next);
                draft.custody_registry().reconcile_releases();
                Err(error)
            }
        }
    }
    pub fn prepare_node(
        &self,
        draft: &Draft,
        change: NodeChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<ContainerPlan>, DomError> {
        let old = change.before.is_some_and(participant);
        let new = change.after.is_some_and(participant);
        let old_shadow = change.before.and_then(|n| n.shadow_root);
        let new_shadow = change.after.and_then(|n| n.shadow_root);
        let old_host = change.before.and_then(|n| n.host);
        let new_host = change.after.and_then(|n| n.host);
        if old == new && old_shadow == new_shadow && old_host == new_host {
            return Ok(None);
        }
        self.prepare(draft, cancel, |next, edit| {
            let key = draft.live_key(change.key);
            let mut selected = Map::default();
            let mut custody = MapCustody::default();
            if old_shadow != new_shadow {
                for shadow in old_shadow
                    .into_iter()
                    .chain(new_shadow.filter(|s| Some(*s) != old_shadow))
                {
                    select(
                        &self.current.index,
                        draft.live_key(shadow),
                        order.before,
                        &mut selected,
                        &mut custody,
                        edit,
                    )?;
                }
            }
            if old_host != new_host && change.before.is_some() && change.after.is_some() {
                select(
                    &self.current.index,
                    key,
                    order.before,
                    &mut selected,
                    &mut custody,
                    edit,
                )?;
            }
            let mut changed = next.relocate_selected(&selected, order, edit)?;
            if old != new {
                next.fact(key, new, if new { order.after } else { order.before }, edit)?;
                changed = true;
            }

            Ok(changed)
        })
    }
    pub fn prepare_relation(
        &self,
        draft: &Draft,
        change: RelationChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<ContainerPlan>, DomError> {
        if !matches!(change.field, RelationField::Children)
            || (change.removed == 0 && change.added.is_empty())
        {
            return Ok(None);
        }
        self.prepare(draft, cancel, |next, edit| {
            let mut selected = Map::default();
            let mut custody = MapCustody::default();
            for offset in 0..change.removed {
                let key = change
                    .before
                    .get(change.index + offset)
                    .ok_or(DomError::IndexSize)?;
                select(
                    &self.current.index,
                    draft.live_key(key),
                    order.before,
                    &mut selected,
                    &mut custody,
                    edit,
                )?;
            }
            for &key in change.added {
                select(
                    &self.current.index,
                    draft.live_key(key),
                    order.before,
                    &mut selected,
                    &mut custody,
                    edit,
                )?;
            }
            let changed = next.relocate_selected(&selected, order, edit)?;

            Ok(changed)
        })
    }
    pub fn prepare_visibility(
        &self,
        draft: &Draft,
        key: Key,
        was_private: bool,
        is_private: bool,
        _order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<ContainerPlan>, DomError> {
        let key = draft.live_key(key);
        if was_private == is_private || self.current.index.facts.get(key).is_none() {
            return Ok(None);
        }
        self.prepare(draft, cancel, |next, edit| {
            next.dirty(key, edit)?;
            Ok(true)
        })
    }
    pub fn finish(
        &self,
        draft: &Draft,
        mapper: &impl Fn(Key) -> Option<NodeId>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<ContainerFinished, DomError> {
        let mut output = ContainerFinished {
            index: self.base.clone(),
            custody: self.base_custody.clone(),
        };
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |u| draft.admit_custody_usage(u, cancel),
            step: &mut || draft.source_step(cancel),
        };
        // Remove every affected old member under its original topology before
        // inserting any member under the projected final topology.
        for &key in self.current.dirty.values() {
            (edit.step)()?;
            if let Some(id) = draft.published_id(key) {
                let old = Key::Existing(id);
                if self.base.facts.get(old).is_some() {
                    output.index.change(
                        &mut output.custody,
                        old,
                        false,
                        order.before,
                        &mut edit,
                    )?;
                    output
                        .index
                        .facts
                        .change(&mut output.custody.facts, old, None, &mut edit)?;
                }
            }
        }
        for &key in self.current.dirty.values() {
            (edit.step)()?;
            if self.current.index.facts.get(key).is_some()
                && let Some(id) = mapper(key)
            {
                let new = Key::Existing(id);
                output
                    .index
                    .change(&mut output.custody, new, true, order.after, &mut edit)?;
                output
                    .index
                    .facts
                    .change(&mut output.custody.facts, new, Some(new), &mut edit)?;
            }
        }
        Ok(output)
    }
    pub fn mark_published(&mut self, index: &ContainerIndex, custody: &ContainerCustody) {
        self.base = index.clone();
        self.base_custody = custody.clone();
        self.current.dirty = Map::default();
        self.current.dirty_custody = MapCustody::default();
    }
}
#[cfg(test)]
impl ContainerIndex {
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let mut graph = self.roots.inspect_graph()?;
        graph.merge(&self.members.inspect_graph()?)?;
        graph.merge(&self.facts.inspect_graph()?)?;
        Ok(graph)
    }
    pub fn inspect_groups(&self) -> Result<Vec<(Key, Vec<Key>)>, DomError> {
        self.roots
            .values()
            .map(|r| {
                self.members
                    .inspect_group(r.member)
                    .map(|members| (r.key, members))
            })
            .collect()
    }
    pub fn inspect_facts(&self) -> Vec<Key> {
        self.facts.values().copied().collect()
    }
    pub fn inspect_members(&self) -> Vec<(Key, Key)> {
        self.roots
            .values()
            .flat_map(|r| {
                self.members
                    .inspect_tree(r.member)
                    .into_iter()
                    .map(move |m| (r.key, m.key))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
